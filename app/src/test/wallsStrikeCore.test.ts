// Walls-only city strike in the mock (core combat::city_can_fire): only walled cities fire, and
// the UI says which ones can.
import { describe, expect, it } from 'vitest';
import { cityCanFire } from '../combat';
import { cityFireStatus } from '../format';
import { wallsFixture } from '../test/wallsFixture';
import { FixtureClient } from './fixtureEngine';

function load() {
  const w = wallsFixture();
  const client = new FixtureClient();
  client.load(JSON.stringify(w.fixture));
  return { ...w, client };
}

describe('walls-only city strike', () => {
  it('needs Walls and HP', () => {
    expect(cityCanFire(true, 1)).toBe(true);
    expect(cityCanFire(false, 200)).toBe(false);
    expect(cityCanFire(true, 0)).toBe(false);
  });

  it('says which cities can fire, naming the Alpine Redoubt', () => {
    const { client, walled, open, bastion, own } = load();
    const v = client.view(0);
    const status = (id: number) => cityFireStatus(v, v.cities.find((c) => c.id === id)!);
    expect(status(walled)).toEqual({ fires: true, text: 'Walls: fires at units within 2 tiles' });
    expect(status(bastion)).toEqual({ fires: true, text: 'Alpine Redoubt: fires at units within 2 tiles' });
    expect(status(own)).toMatchObject({ fires: true });
    expect(status(open)).toEqual({ fires: false, text: "No Walls: can't fire" });
  });

  it('only the walled city shoots, and only its forecast carries a city shot', () => {
    const { client, walled, open, nearWalled, nearOpen } = load();
    const v = client.view(0);
    const pos = (id: number) => v.cities.find((c) => c.id === id)!.pos;
    expect(client.forecast(0, nearWalled, pos(walled), 'Melee')?.city_strike).toMatchObject({ city: walled });
    expect(client.forecast(0, nearOpen, pos(open), 'Melee')?.city_strike).toBeUndefined();
    const events = client.endTurn({}).events;
    const shots = events.filter((e) => e.type === 'Battle' && typeof e.attacker !== 'number');
    const hit = shots.map((e) => e.type === 'Battle' && e.defender.kind === 'Unit' && e.defender.unit);
    // The walled city and the Alpine Redoubt (at the Scout) fire; the unwalled city does not.
    expect(hit).toContain(nearWalled);
    expect(hit).not.toContain(nearOpen);
    expect(shots).toHaveLength(2);
  });
});

// The tour fixtures load into the real core: Ruhr Foundries count, an Heir shows his `heir_of`.
import { describe, expect, it } from 'vitest';
import { coreFixture } from './coreFixture';
import { forgeHeirFixture } from './forgeHeirFixture';

describe('forgeHeirFixture', () => {
  it('builds five German cities, four with a Ruhr Foundry', () => {
    const { fixture, capital } = forgeHeirFixture('Germany');
    const own = fixture.cities.filter((c) => c.owner === 0);
    expect(own).toHaveLength(5);
    expect(fixture.turn).toBeGreaterThanOrEqual(100);
    expect(fixture.players[0].techs).toContain('IronWorking');
    expect(own.every((c) => c.population >= 6)).toBe(true);
    expect(own.filter((c) => c.buildings.includes('MachineShop'))).toHaveLength(4);
    expect(own.find((c) => c.id === capital)!.buildings).toEqual([]);
    expect(coreFixture(fixture)).toContain('"MachineShop"');
  });
  it('puts the Heir at home or in a rival’s hands, always as Byzantium’s', () => {
    for (const [state, owner] of [['own', 0], ['captured', 1]] as const) {
      const { fixture } = forgeHeirFixture('Byzantium', state);
      const heir = fixture.units.find((u) => u.kind === 'Heir')!;
      expect([heir.owner, heir.heir_of]).toEqual([owner, 0]);
      expect(JSON.parse(coreFixture(fixture)).units[heir.id].heir_of).toBe(0);
    }
    expect(forgeHeirFixture('Byzantium', 'none').fixture.units.some((u) => u.kind === 'Heir')).toBe(false);
  });
});

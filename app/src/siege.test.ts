import { describe, expect, it } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import { siegeFixture } from './test/siegeFixture';
import { alarmedCities, captureForecast, citiesUnderAttack, cityHits, forceText } from './siege';
import { pendingDecisions, nextDecision } from './turnFlow';
import { contextFromViews, formatEvents, groupLines } from './format';
import { fromOffset } from './hex';
import { useGame } from './store';
import { unitStatsFor } from './data';
import type { Event, PlayerView, UnitKind, UnitView } from './types';

function setup() {
  const client = new FixtureClient();
  const s = siegeFixture();
  client.load(JSON.stringify(s.fixture));
  return { client, view: client.view(0), ...s };
}

/** A copy of the view with the first own city's HP and the attackers' count changed. */
function variant(view: PlayerView, patch: { hp?: number; keep?: number; attackedLastTurn?: boolean; add?: { kind: UnitKind; at: [number, number]; owner?: number }[] }): PlayerView {
  const own = view.cities.find((c) => c.owner === view.me)!;
  const enemies = view.units.filter((u) => u.owner !== view.me && u.military);
  const kept = new Set(enemies.slice(0, patch.keep ?? enemies.length).map((u) => u.id));
  const extra = (patch.add ?? []).map((a, i): UnitView => {
    const stats = unitStatsFor('Byzantium', a.kind);
    return { ...enemies[0], id: 900 + i, kind: a.kind, owner: a.owner ?? enemies[0].owner, pos: fromOffset(...a.at), strength: stats.strength, ranged: stats.ranged, range: stats.range, military: stats.military };
  });
  return { ...view,
    cities: view.cities.map((c) => c.id !== own.id ? c : { ...c, hp: patch.hp ?? c.hp, detail: c.detail ? { ...c.detail, attacked_last_turn: patch.attackedLastTurn ?? c.detail.attacked_last_turn } : c.detail }),
    units: [...view.units.filter((u) => u.owner === view.me || !u.military || kept.has(u.id)), ...extra] };
}

describe('cities under attack', () => {
  it('flags a city with enemies beside it and one that took damage last turn, with the attackers named', () => {
    const { view, own } = setup();
    const alarms = citiesUnderAttack(view, null);
    expect(alarms).toHaveLength(1);
    expect(alarms[0]).toMatchObject({ kind: 'attack', cityId: own, name: 'Waypost Seven', hp: 120, maxHp: 200, attackedLastTurn: true, force: '3 Spearmen' });
    expect(alarms[0].attackers).toHaveLength(3);
    expect(alarmedCities(alarms).get(own)).toBeDefined();
  });

  it('is quiet with no enemy near and no damage last turn', () => {
    const { view } = setup();
    expect(citiesUnderAttack(variant(view, { keep: 0, attackedLastTurn: false }), null)).toEqual([]);
  });

  it('still flags a city hit last turn when nothing stands beside it now, saying so', () => {
    const { view } = setup();
    const [alarm] = citiesUnderAttack(variant(view, { keep: 0 }), null);
    expect(alarm).toMatchObject({ kind: 'attack', attackers: [], attackedLastTurn: true });
    expect(alarm.forecast).toMatchObject({ level: 'quiet', turns: null });
  });

  it('ignores peace, scouts and the viewer\'s own team', () => {
    const { view } = setup();
    const peace = { ...view, diplomacy: { ...view.diplomacy, relations: view.diplomacy.relations.map((r) => ({ ...r, state: 'Peace' as const })) } };
    expect(citiesUnderAttack(variant(peace, { attackedLastTurn: false }), null)).toEqual([]);
    const scouts = variant(view, { keep: 0, attackedLastTurn: false, add: [{ kind: 'Scout', at: [3, 4] }, { kind: 'Scout', at: [4, 5] }] });
    expect(citiesUnderAttack(scouts, null)).toEqual([]);
  });

  it('forecasts the capture from public numbers: falls next turn when one turn of hits is enough', () => {
    const { view, own } = setup();
    const weak = variant(view, { hp: 40 });
    const { forecast, perTurn } = captureForecast(weak, own, citiesUnderAttack(weak, null)[0].attackers);
    expect(forecast).toMatchObject({ turns: 1, level: 'now', text: 'Falls next turn unless reinforced' });
    expect(perTurn).toBeGreaterThan(40);
    expect(forecast.detail).toContain('3 Spearmen hit for about');
    expect(forecast.detail).toContain('Nothing garrisons it');
  });

  it('counts the turns it holds, and a garrison adds the turns it takes to fall', () => {
    const { view, own, attackers } = setup();
    const open = captureForecast(view, own, view.units.filter((u) => attackers.includes(u.id)));
    expect(open.forecast.turns).toBe(2);
    expect(open.forecast).toMatchObject({ level: 'soon', text: 'Holds about 2 turns unless reinforced' });
    const ownCity = view.cities.find((c) => c.id === own)!;
    const garrisoned = { ...view, units: [...view.units, { ...view.units.find((u) => u.owner === view.me)!, id: 800, kind: 'Soldier' as const, military: true, strength: 9, ranged: 0, range: 0, hp: 100, fortify: 2, pos: ownCity.pos }] };
    const guard = captureForecast(garrisoned, own, garrisoned.units.filter((u) => attackers.includes(u.id)));
    expect(guard.forecast.turns!).toBeGreaterThan(open.forecast.turns!);
    expect(guard.forecast.detail).toContain('must fall before it is taken');
  });

  it('says ranged units cannot take a city, and when it drops to 0 HP', () => {
    const { view, own } = setup();
    const rangers = variant(view, { keep: 0, hp: 30, add: [{ kind: 'Ranger', at: [5, 5] }, { kind: 'Ranger', at: [4, 4] }] });
    const alarm = citiesUnderAttack(rangers, null)[0];
    expect(alarm.attackers).toHaveLength(2);
    expect(alarm.forecast.turns).toBeNull();
    expect(alarm.forecast.text).toMatch(/Drops to 0 HP next turn; no melee unit to take it yet/);
    expect(alarm.cityId).toBe(own);
    const atZero = citiesUnderAttack(variant(view, { keep: 0, hp: 0, add: [{ kind: 'Ranger', at: [5, 5] }, { kind: 'Ranger', at: [4, 4] }] }), null)[0];
    expect(atZero.forecast).toMatchObject({ short: 'at 0 HP', text: 'At 0 HP; no melee unit to take it yet' });
  });

  it('warns when an army of two or more closes within three tiles, but not for a lone unit', () => {
    const { view } = setup();
    const near = variant(view, { keep: 0, attackedLastTurn: false, add: [{ kind: 'Soldier', at: [6, 5] }, { kind: 'Soldier', at: [6, 6] }] });
    const [alarm] = citiesUnderAttack(near, null);
    expect(alarm).toMatchObject({ kind: 'approach', force: '2 Spearmen' });
    expect(alarm.forecast.text).toContain('Reaches it');
    expect(alarmedCities([alarm]).size).toBe(0);
    const lone = variant(view, { keep: 0, attackedLastTurn: false, add: [{ kind: 'Soldier', at: [6, 5] }] });
    expect(citiesUnderAttack(lone, null)).toEqual([]);
    const far = variant(view, { keep: 0, attackedLastTurn: false, add: [{ kind: 'Soldier', at: [8, 5] }, { kind: 'Soldier', at: [8, 6] }] });
    expect(citiesUnderAttack(far, null)).toEqual([]);
  });

  it('puts the city that falls soonest first, attacks before warnings', () => {
    const { view } = setup();
    const alarms = citiesUnderAttack(variant(view, { hp: 40 }), null);
    expect(alarms[0].forecast.level).toBe('now');
    expect(alarms.map((a) => a.kind)).toEqual(['attack']);
  });

  it('is a pending decision until the player opens the city this turn, and then keeps showing', () => {
    const { view, own } = setup();
    const first = pendingDecisions(view, []);
    expect(first[0]).toMatchObject({ kind: 'siege', id: own });
    expect(first[0].label).toContain('Waypost Seven is under attack');
    expect(nextDecision(first, null)).toBe(first[0]);
    const opened = pendingDecisions(view, [], [], { humans: [0], orders: [[]], errors: [[]], siegeSeen: { 0: { turn: view.turn, cities: [own] } } });
    expect(opened.some((d) => d.kind === 'siege')).toBe(false);
    // Last turn's opening does not count.
    const stale = pendingDecisions(view, [], [], { humans: [0], orders: [[]], errors: [[]], siegeSeen: { 0: { turn: view.turn - 1, cities: [own] } } });
    expect(stale.some((d) => d.kind === 'siege')).toBe(true);
    expect(citiesUnderAttack(view, null)).toHaveLength(1);
  });

  it('opening the city from anywhere answers the decision for this turn only', () => {
    const { client, view, own } = setup();
    useGame.setState(useGame.getInitialState(), true);
    useGame.setState({ client, humans: [0], views: [view], screen: 'game', seat: 0, orders: [[]], errors: [[]], ready: [false] });
    const open = () => pendingDecisions(view, [], [], useGame.getState()).some((d) => d.kind === 'siege');
    expect(open()).toBe(true);
    useGame.getState().nextDecision();
    expect(useGame.getState().selection).toEqual({ kind: 'city', id: own });
    expect(useGame.getState().siegeSeen[0]).toEqual({ turn: view.turn, cities: [own] });
    expect(open()).toBe(false);
    // Selecting it again does not duplicate the record, and a later turn starts unopened.
    useGame.getState().select({ kind: 'city', id: own });
    expect(useGame.getState().siegeSeen[0].cities).toEqual([own]);
    expect(pendingDecisions({ ...view, turn: view.turn + 1 }, [], [], useGame.getState()).some((d) => d.kind === 'siege')).toBe(true);
  });

  it('writes the unit force in words', () => {
    expect(forceText([{ kind: 'Soldier' }, { kind: 'Soldier' }, { kind: 'Soldier' }])).toBe('3 Spearmen');
    expect(forceText([{ kind: 'Soldier' }, { kind: 'Ranger' }])).toBe('a Spearman and an Archer');
    expect(forceText([{ kind: null }])).toBe('an unseen unit');
  });
});

describe('the turn report line', () => {
  let city = 0;
  const battle = (attacker: number, damage: number, hpAfter: number, extra: Partial<Extract<Event, { type: 'Battle' }>> = {}): Event => ({
    type: 'Battle', attacker, attacker_owner: 1, defender: { kind: 'City', city, owner: 0 }, at: fromOffset(3, 5), attacker_at: fromOffset(4, 5),
    attacker_strength: { total: 9, parts: [] }, defender_strength: { total: 16, parts: [] }, damage_to_attacker: 10, damage_to_defender: damage,
    attacker_roll: 100, defender_roll: 100, attacker_hp_after: 90, defender_hp_after: hpAfter, ranged: false, meeting: false, ...extra,
  });

  it('sums a city\'s hits into one before-and-after line, first in the report', () => {
    const { client, attackers, view, own } = setup();
    city = own;
    const [a, b, c] = attackers;
    // One stage's attacks share a snapshot, so each reports the same HP left.
    const events: Event[] = [battle(a, 27, 120), battle(b, 27, 120), battle(c, 26, 120),
      { type: 'UnitDestroyed', unit: 99, owner: 1, unit_kind: 'Soldier', at: fromOffset(1, 1) }];
    const ctx = contextFromViews(variant(client.view(0), { hp: 200 }), view);
    const lines = formatEvents(events, ctx);
    expect(lines[0]).toMatchObject({ category: 'attack', tone: 'bad', weight: 2, icon: 'ui-siege' });
    expect(lines[0].text).toBe('Waypost Seven was attacked: 200 → 120 HP by 3 Spearmen from Byzantium');
    expect(lines[0].hp).toEqual([{ text: '200 → 120 HP', tone: 'bad' }]);
    expect(lines[0].at).toEqual(view.cities.find((x) => x.name === 'Waypost Seven')!.pos);
    expect(groupLines(lines)[0].category).toBe('attack');
    expect(cityHits(events, (o) => o === 0, () => 'Soldier', () => 200)[0]).toMatchObject({ before: 200, after: 120, lost: 80 });
    // Without the earlier view, the start is what is left plus every hit; only an overshoot makes that more than it really had.
    expect(cityHits(events, (o) => o === 0, () => 'Soldier')[0]).toMatchObject({ before: 200, after: 120, lost: 80 });
  });

  it('says a city that reached 0 HP, ignores attacks on other empires\' cities and the city\'s own shots', () => {
    const { client, attackers, view, own } = setup();
    city = own;
    const events: Event[] = [battle(attackers[0], 120, 0),
      battle(attackers[1], 30, 70, { defender: { kind: 'City', city: 5, owner: 2 } }),
      battle(attackers[2], 10, 50, { attacker: { city: own } as unknown as number, attacker_owner: 0 })];
    const lines = formatEvents(events, contextFromViews(client.view(0), view)).filter((l) => l.category === 'attack');
    expect(lines).toHaveLength(1);
    expect(lines[0].text).toBe('Waypost Seven was attacked: 120 → 0 HP by a Spearman from Byzantium, and it is at 0 HP');
  });
});

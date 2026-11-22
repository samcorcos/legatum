import { beforeAll, describe, expect, it } from 'vitest';
import { addCity, addUnit, arenaView } from './test/arenaView';
import { aircraftNeedingOrders, airHeadline, airOverlayFor, airState, fightersOnGuard, guardNote, noTargetReason, pickerCandidates, rebaseOptions, readyNotice, reconBlocker, reconTargets, spotsIn, type Hold } from './airWing';
import { nextDecision, pendingDecisions } from './turnFlow';
import { momentsForTurn } from './celebrate/moments';
import { contextFromViews, formatEvent } from './format';
import { distance, fromOffset, tileIndex } from './hex';
import { describeOrder } from './orders';
import type { AircraftView, CityView, Event, Order, PlayerView } from './types';

let base: PlayerView;
beforeAll(async () => {
  base = await arenaView();
  // Keep a single Athens for the possessive-copy fixture regardless of lobby rotation.
  const rival = base.players.find((p) => p.id === 2)!;
  const athens = base.players.find((p) => p.faction === 'Athens');
  if (athens) athens.faction = rival.faction;
  rival.faction = 'Athens';
});

const HOME = fromOffset(8, 8);
/** Bern with a Fighter (id 50) and a Bomber (id 51) in the hangar; an enemy Spearman `tiles` east of it, when given. */
function wing(tiles: number | null = null, extra: (v: PlayerView, city: CityView) => void = () => {}) {
  const v = JSON.parse(JSON.stringify(base)) as PlayerView;
  const city = addCity(v, 0, 'Bern', HOME);
  v.aircraft = [
    { id: 50, owner: 0, kind: 'Fighter', city: city.id, hp: 100 },
    { id: 51, owner: 0, kind: 'Bomber', city: city.id, hp: 100 },
  ] satisfies AircraftView[];
  if (tiles !== null) addUnit(v, 2, 'Spearman', fromOffset(8 + tiles, 8));
  extra(v, city);
  return { v, city, fighter: v.aircraft[0], bomber: v.aircraft[1] };
}
const held = (v: PlayerView, ...planes: number[]): Hold => ({ turn: v.turn, planes });

describe('what an aircraft is doing, in plain words', () => {
  it('a Fighter with nothing to strike is on guard, and still needs orders until it is answered', () => {
    const { v, city, fighter } = wing();
    const state = airState(v, city, fighter, [], undefined);
    expect(state).toMatchObject({ kind: 'guard', targets: 0, held: false });
    expect(airHeadline(fighter, city, state)).toBe('On guard: covers 4 tiles around Bern');
    // Unanswered it is a decision (like any idle unit); "stay on guard" moves it to This Turn.
    expect(fightersOnGuard(v, [], undefined)).toEqual([]);
    expect(aircraftNeedingOrders(v, [], undefined).map((a) => [a.plane.id, a.targets])).toEqual([[50, 0], [51, 0]]);
    expect(fightersOnGuard(v, [], held(v, 50)).map((g) => g.plane.id)).toEqual([50]);
    expect(aircraftNeedingOrders(v, [], held(v, 50, 51))).toEqual([]);
  });

  it('sleeping keeps an aircraft quiet on later turns until an enemy comes in reach', () => {
    const { v, city, fighter, bomber } = wing();
    const asleep: Hold = { turn: v.turn - 3, planes: [], asleep: [50, 51] };
    expect(airState(v, city, fighter, [], asleep)).toMatchObject({ kind: 'guard', held: true, asleep: true });
    expect(airState(v, city, bomber, [], asleep)).toMatchObject({ kind: 'idle', held: true, asleep: true });
    expect(airHeadline(bomber, city, airState(v, city, bomber, [], asleep))).toBe('Waiting: wakes when an enemy is within reach');
    expect(aircraftNeedingOrders(v, [], asleep)).toEqual([]);
    expect(fightersOnGuard(v, [], asleep).map((g) => g.plane.id)).toEqual([50]);
    // An enemy in reach wakes it.
    const near = wing(2);
    expect(airState(near.v, near.city, near.bomber, [], asleep)).toMatchObject({ kind: 'ready', targets: 1 });
    expect(aircraftNeedingOrders(near.v, [], asleep).map((a) => a.plane.id)).toEqual([50, 51]);
  });

  it('a Fighter reaches 5 tiles, a Bomber 8: an enemy at 6 tiles is only the Bomber\'s target', () => {
    const near = wing(5), far = wing(6);
    expect(airState(near.v, near.city, near.fighter, [], undefined)).toMatchObject({ kind: 'ready', targets: 1 });
    expect(airHeadline(near.fighter, near.city, airState(near.v, near.city, near.fighter, [], undefined))).toBe('Ready: 1 target within 5 tiles');
    expect(airState(far.v, far.city, far.fighter, [], undefined)).toMatchObject({ kind: 'guard', targets: 0 });
    expect(airState(far.v, far.city, far.bomber, [], undefined)).toMatchObject({ kind: 'ready', targets: 1 });
    expect(airHeadline(far.bomber, far.city, airState(far.v, far.city, far.bomber, [], undefined))).toBe('Ready: 1 target within 8 tiles');
    const beyond = wing(9);
    expect(airState(beyond.v, beyond.city, beyond.bomber, [], undefined)).toMatchObject({ kind: 'idle', targets: 0 });
    expect(airHeadline(beyond.bomber, beyond.city, airState(beyond.v, beyond.city, beyond.bomber, [], undefined))).toBe('No targets within 8 tiles');
  });

  it('a teammate\'s unit is never a target, and the reason names what is missing', () => {
    const { v, city, bomber } = wing(null, (view) => { addUnit(view, 1, 'Spearman', fromOffset(10, 8)); });
    expect(airState(v, city, bomber, [], undefined)).toMatchObject({ kind: 'idle', targets: 0 });
    expect(noTargetReason(v, city, 'Bomber')).toBe('No enemy unit or city is visible within 8 tiles of Bern.');
  });

  it('a rival you are not at war with is named as the reason', () => {
    const { v, city } = wing(2, (view) => {
      view.diplomacy = { ...view.diplomacy, legacy: false, relations: [{ a: 0, b: view.players.find((p) => p.id === 2)!.team, state: 'Peace' }] } as PlayerView['diplomacy'];
    });
    expect(airState(v, city, v.aircraft![1], [], undefined)).toMatchObject({ kind: 'idle', targets: 0 });
    expect(noTargetReason(v, city, 'Bomber')).toMatch(/not at War with you: declare War to strike/);
  });

  it('spared barbarians are in reach but ask for no answer, and the reason says so', () => {
    const { v, city, fighter, bomber } = wing(null, (view) => {
      view.players.find((p) => p.id === view.me)!.faction = 'Celts';
      addUnit(view, 2, 'Spearman', fromOffset(10, 8));
      view.units[view.units.length - 1].owner = 255;
    });
    expect(airState(v, city, fighter, [], undefined)).toMatchObject({ kind: 'guard', targets: 0 });
    expect(airState(v, city, bomber, [], undefined)).toMatchObject({ kind: 'idle', targets: 0 });
    expect(noTargetReason(v, city, 'Fighter')).toBe("Only barbarians are within 5 tiles of Bern, and they don't attack you. You can still strike them.");
  });

  it('the headline never says "No targets" over a list of at-Peace targets (8.9.6 UI review)', () => {
    const { v, city, bomber } = wing(2, (view) => {
      view.diplomacy = { ...view.diplomacy, legacy: false, relations: [{ a: 0, b: view.players.find((p) => p.id === 2)!.team, state: 'Peace' }] } as PlayerView['diplomacy'];
    });
    const state = airState(v, city, bomber, [], undefined);
    const headline = airHeadline(bomber, city, state, v);
    expect(headline).toMatch(/ in range, but you are at Peace$/);
    expect(headline).not.toMatch(/No targets/);
  });

  it('an ordered strike or rebase is its own state, and a Fighter that strikes says it will not intercept', () => {
    const { v, city, fighter, bomber } = wing(2);
    const strike: Order = { type: 'AirStrike', plane: 50, target: fromOffset(10, 8) };
    const state = airState(v, city, fighter, [strike], undefined);
    expect(state.kind).toBe('ordered');
    expect(airHeadline(fighter, city, state)).toBe('Striking this turn, so it will not intercept');
    expect(airHeadline(bomber, city, airState(v, city, bomber, [{ type: 'AirStrike', plane: 51, target: fromOffset(10, 8) }], undefined))).toBe('Striking this turn');
    // An ordered Fighter is not on guard.
    expect(fightersOnGuard(v, [strike], undefined)).toEqual([]);
  });

  it('the guard note says whether an empire at war could send Bombers near, from public facts', () => {
    const quiet = wing(null, (view) => { addCity(view, 2, 'Karakorum', fromOffset(15, 8)); });
    expect(guardNote(quiet.v, quiet.city)).toMatch(/No empire at War with you has Bombers within reach/);
    const flight = (quiet.v.research.catalog.find((t) => t.tech === 'Flight')?.era ?? 4) + 1;
    const threat = wing(null, (view) => {
      addCity(view, 2, 'Karakorum', fromOffset(15, 8));
      view.players.find((p) => p.id === 2)!.era = flight;
    });
    expect(guardNote(threat.v, threat.city)).toMatch(/Karakorum, 7 tiles away\) is at War with you and can build Bombers/);
  });
});

describe('Recon', () => {
  it('lights every on-map tile from 1 to the aircraft\'s strike range of its city', () => {
    const { v, city, fighter, bomber } = wing();
    const f = reconTargets(v, city, fighter.kind);
    const b = reconTargets(v, city, bomber.kind);
    expect(f.length).toBe(3 * 5 * 6);
    expect(b.length).toBe(3 * 8 * 9);
    expect([...f, ...b].every((at) => distance(at, city.pos) >= 1 && distance(at, city.pos) <= (f.includes(at) ? 5 : 8))).toBe(true);
    const open = airOverlayFor(v, city.id, { plane: 51, what: 'recon', hover: null })!;
    expect(open.pickKind).toBe('recon');
    expect(open.candidates).toEqual(b);
    expect(pickerCandidates(v, city, bomber, 'recon').map((c) => c.at)).toEqual(b);
  });

  it('tiles off the map are not offered', () => {
    const { v, city, bomber } = wing(null, (_view, c) => { c.pos = fromOffset(1, 1); });
    const targets = reconTargets(v, city, bomber.kind);
    expect(targets.length).toBeLessThan(3 * 8 * 9);
    expect(targets.every((at) => tileIndex(at, v.width, v.height) >= 0)).toBe(true);
  });

  it('an ordered Recon is its own state: the Fighter still guards, the Bomber scouts', () => {
    const { v, city, fighter, bomber } = wing();
    const order = (plane: number): Order => ({ type: 'AirRecon', plane, target: fromOffset(12, 8) });
    expect(airState(v, city, fighter, [order(50)], undefined).kind).toBe('ordered');
    expect(airHeadline(fighter, city, airState(v, city, fighter, [order(50)], undefined))).toBe('Scouting this turn; it still guards');
    expect(airHeadline(bomber, city, airState(v, city, bomber, [order(51)], undefined))).toBe('Scouting this turn');
    expect(aircraftNeedingOrders(v, [order(50)], undefined).map((a) => a.plane.id)).toEqual([51]);
    expect(describeOrder(order(51), v)).toMatch(/^Bomber: recon of /);
  });

  it('says why it is unavailable', () => {
    const { v, city, bomber } = wing();
    expect(reconBlocker(v, city, bomber, [])).toBeNull();
    expect(reconBlocker(v, city, bomber, [{ type: 'AirStrike', plane: 51, target: fromOffset(10, 8) }])).toContain('already has an order this turn');
    expect(reconBlocker({ ...v, finished: true }, city, bomber, [])).toContain('game is over');
  });
});

describe('the Return queue', () => {
  it('every idle Bomber or Fighter needs orders like a unit, with or without a target in reach', () => {
    const { v } = wing(2);
    const list = pendingDecisions(v, [], []);
    expect(list.filter((d) => d.kind === 'air').map((d) => d.id)).toEqual([50, 51]);
    expect(list.find((d) => d.kind === 'air')).toMatchObject({ kind: 'air', city: v.cities[0].id, label: 'Fighter in Bern: 1 target within 5 tiles, strike or stay on guard' });
    expect(list.filter((d) => d.kind === 'air')[1].label).toBe('Bomber in Bern: 1 target within 8 tiles, strike or hold');
    // Nothing in reach: it is still in the queue, so the player knows it exists.
    const idle = pendingDecisions(wing().v, [], []).filter((d) => d.kind === 'air');
    expect(idle.map((d) => d.id)).toEqual([50, 51]);
    expect(idle.map((d) => d.label)).toEqual(['Fighter in Bern: needs orders', 'Bomber in Bern: needs orders']);
  });

  it('holding an aircraft answers it for this turn only', () => {
    const { v } = wing(2);
    const answered = pendingDecisions(v, [], [], { humans: [0], orders: [[]], errors: [[]], airHeld: { 0: held(v, 50) } });
    expect(answered.filter((d) => d.kind === 'air').map((d) => d.id)).toEqual([51]);
    const stale = pendingDecisions(v, [], [], { humans: [0], orders: [[]], errors: [[]], airHeld: { 0: { turn: v.turn - 1, planes: [50, 51] } } });
    expect(stale.filter((d) => d.kind === 'air').map((d) => d.id)).toEqual([50, 51]);
    // The held Fighter is on guard (This Turn lists it); the held Bomber simply holds.
    expect(fightersOnGuard(v, [], held(v, 50)).map((g) => g.plane.id)).toEqual([50]);
  });

  it('an order answers it, and an invalid order does not hide it from the player', () => {
    const { v } = wing(2);
    const strike: Order = { type: 'AirStrike', plane: 51, target: fromOffset(10, 8) };
    expect(pendingDecisions(v, [strike], []).filter((d) => d.kind === 'air').map((d) => d.id)).toEqual([50]);
    expect(pendingDecisions(v, [strike], [{ index: 0, message: 'out of range' }]).filter((d) => d.kind === 'air').map((d) => d.id)).toEqual([50, 51]);
  });

  it('a rebase answers it too', () => {
    const { v } = wing();
    expect(pendingDecisions(v, [{ type: 'Rebase', plane: 51, to: 7 }], []).filter((d) => d.kind === 'air').map((d) => d.id)).toEqual([50]);
  });

  it('Return goes to the aircraft after units and before idle cities', () => {
    const { v } = wing(2);
    const list = pendingDecisions(v, [], []);
    const first = nextDecision(list, null)!;
    expect(first.kind).toBe(list[0].kind);
    expect(list.map((d) => d.kind).indexOf('air')).toBeLessThan(list.findIndex((d) => d.kind === 'city') === -1 ? 99 : list.findIndex((d) => d.kind === 'city'));
  });

  it('a finished game and rival aircraft ask nothing', () => {
    const { v } = wing(2);
    expect(aircraftNeedingOrders({ ...v, finished: true }, [], undefined)).toEqual([]);
    const rival = JSON.parse(JSON.stringify(v)) as PlayerView;
    rival.aircraft = rival.aircraft!.map((a) => ({ ...a, owner: 2 }));
    expect(aircraftNeedingOrders(rival, [], undefined)).toEqual([]);
  });
});

describe('the map', () => {
  it('shows stealth strike and guard ranges on the same city overlay', () => {
    const { v, city } = wing(2);
    v.aircraft![0].kind = 'StealthBomber';
    v.aircraft![1].kind = 'StealthFighter';
    expect(airOverlayFor(v, city.id, { plane: null, what: null, hover: null })!.rings.map((r) => [r.radius, r.label])).toEqual([
      [10, 'Stealth Bomber strikes 10 tiles'],
      [7, 'Stealth Fighter strikes 7 tiles'],
      [5, 'Stealth Fighter guards 5 tiles'],
    ]);
  });
  it('rings what the city\'s aircraft cover: the Bomber 8, the Fighter\'s guard 4 and strike 5', () => {
    const { v, city } = wing(2);
    const overlay = airOverlayFor(v, city.id, { plane: null, what: null, hover: null })!;
    expect(overlay.rings.map((r) => [r.radius, r.line])).toEqual([[8, 'solid'], [5, 'dotted'], [4, 'dashed']]);
    expect(overlay.rings.map((r) => r.label)).toEqual(['Bomber strikes 8 tiles', 'Fighter strikes 5 tiles', 'Fighter guards 4 tiles']);
    expect(overlay.pickKind).toBeNull();
    // Only a Bomber in the hangar: one ring.
    v.aircraft = v.aircraft!.filter((a) => a.kind === 'Bomber');
    expect(airOverlayFor(v, city.id, { plane: null, what: null, hover: null })!.rings.map((r) => r.radius)).toEqual([8]);
    // A city with no aircraft, or one that is not yours, draws nothing.
    expect(airOverlayFor(wing().v, 999, { plane: null, what: null, hover: null })).toBeNull();
  });

  it('an open strike picker rings its candidates: the Fighter\'s are the ones within 5', () => {
    const { v, city } = wing(2, (view) => { addUnit(view, 2, 'Spearman', fromOffset(15, 8)); });
    const f = airOverlayFor(v, city.id, { plane: 50, what: 'strike', hover: null })!;
    const b = airOverlayFor(v, city.id, { plane: 51, what: 'strike', hover: null })!;
    expect(f.pickKind).toBe('strike');
    expect(f.candidates).toEqual([fromOffset(10, 8)]);
    expect(b.candidates).toEqual([fromOffset(10, 8), fromOffset(15, 8)]);
    expect(pickerCandidates(v, city, v.aircraft![0], 'rebase')).toEqual([]);
  });

  it('rebase options include own cities at any distance', () => {
    const { v, city, bomber } = wing();
    const far = addCity(v, 0, 'Distant', fromOffset(60, 8));
    expect(rebaseOptions(v, city).map((option) => option.city.id)).toContain(far.id);
    expect(pickerCandidates(v, city, bomber, 'rebase').map((c) => c.city?.id)).toContain(far.id);
  });

  it('rebase options count hangar aircraft and valid queued rebases, while explaining full cities', () => {
    const { v, city, bomber } = wing();
    const destination = addCity(v, 0, 'Emain Macha', fromOffset(12, 8));
    v.aircraft!.push({ id: 52, owner: 0, kind: 'Bomber', city: destination.id, hp: 100 });
    const queued: Order[] = [{ type: 'Rebase', plane: 50, to: destination.id }, { type: 'Rebase', plane: 51, to: destination.id }];
    const options = rebaseOptions(v, city, queued, [{ index: 1 }]);
    expect(options.find((option) => option.city.id === destination.id)).toMatchObject({ city: destination, count: 2, queued: 1, cap: 2, available: false });
    expect(rebaseOptions(v, city, queued).find((option) => option.city.id === destination.id)).toMatchObject({ count: 3, queued: 2, available: false });
    expect(pickerCandidates(v, city, bomber, 'rebase', queued)).toEqual([]);
  });
});

describe('the moment an aircraft is ready', () => {
  it('states the job in one plain line', () => {
    expect(readyNotice('Fighter', 'Bern')).toMatchObject({ kicker: 'Aircraft ready · Bern', title: 'Fighter ready', effect: 'On guard it intercepts enemy air strikes within 4 tiles of its city, except a Stealth Bomber’s: the strike fails and the two aircraft trade damage. Order it to strike enemies within 5 tiles instead; that uses its turn.' });
    expect(readyNotice('Bomber', 'Bern')).toMatchObject({ kicker: 'Aircraft ready · Bern', title: 'Bomber ready', effect: 'Order it to strike enemies within 8 tiles of Bern each turn, with no retaliation and +36 against city HP. A Fighter on guard or an Air Defence can intercept the strike.' });
  });

  it('completing one is a card for you, with its art; ground units still get none', () => {
    const { v, city } = wing();
    const done = (unit: 'Fighter' | 'Bomber' | 'Spearman'): Event => ({ type: 'ProductionCompleted', city: city.id, owner: 0, item: { kind: 'Unit', unit } });
    const [card] = momentsForTurn([done('Fighter')], 0, v, v);
    expect(card).toMatchObject({ tier: 'notable', kicker: 'Aircraft ready · Bern', title: 'Fighter ready', at: city.pos, unit: { kind: 'Fighter' } });
    expect(card.effect).toContain('Order it to strike enemies within 5 tiles instead');
    expect(momentsForTurn([done('Bomber')], 0, v, v)[0].effect).toContain('each turn');
    expect(momentsForTurn([done('Spearman')], 0, v, v)).toEqual([]);
    // A rival's aircraft is not announced to you.
    expect(momentsForTurn([{ ...done('Fighter'), owner: 2 } as Event], 0, v, v)).toEqual([]);
  });
});

describe('the turn report', () => {
  const duel = (at: PlayerView['cities'][number], striker: number, strikerOwner: number, fighterOwner: number): Event => ({
    type: 'Battle', attacker: striker, attacker_owner: strikerOwner, defender: { kind: 'Unit', unit: 77, owner: fighterOwner, unit_kind: 'Fighter' },
    at: at.pos, attacker_strength: { total: 36, parts: [['Bomber', 36]] }, defender_strength: { total: 58, parts: [['Fighter', 58]] },
    damage_to_attacker: 21, damage_to_defender: 12, attacker_roll: 90, defender_roll: 100, attacker_hp_after: 79, defender_hp_after: 88, ranged: false, meeting: false,
  } as Event);

  it('says a Fighter of yours intercepted, with the damage, and that the strike did not land', () => {
    const { v, city } = wing();
    const ctx = { ...contextFromViews(v, v), unitKind: (id: number) => (id === 61 ? 'Bomber' as const : null) };
    const line = formatEvent(duel(city, 61, 2, 0), ctx);
    expect(line.text).toMatch(/^Your Fighter from Bern intercepted Athens' Bomber: the strike did not land\.$/);
    expect(line.tone).toBe('good');
    expect(line.details[0]).toContain('traded damage');
    expect(line.hp?.map((h) => h.text)).toEqual(['−12 HP', '−21 HP']);
  });

  it('says your Bomber was intercepted, and names an unseen striker as an air strike', () => {
    const { v, city } = wing();
    const ctx = { ...contextFromViews(v, v), unitKind: (id: number) => (id === 51 ? 'Bomber' as const : null) };
    const mine = formatEvent(duel(city, 51, 0, 2), ctx);
    expect(mine.text).toMatch(/^Your Bomber was intercepted by Athens' Fighter from Bern: the strike did not land\.$/);
    expect(mine.tone).toBe('bad');
    expect(formatEvent(duel(city, 61, 2, 0), contextFromViews(v, v)).text).toMatch(/^Your Fighter from Bern intercepted Athens' air strike: the strike did not land\.$/);
  });

  it('a finished aircraft carries its job on the report line', () => {
    const { v, city } = wing();
    const line = formatEvent({ type: 'ProductionCompleted', city: city.id, owner: 0, item: { kind: 'Unit', unit: 'Fighter' } }, contextFromViews(v, v));
    expect(line.details[0]).toContain('intercepts enemy air strikes within 4 tiles');
  });
});

describe('a strike on the turn war is declared', () => {
  /** A rival at Peace with a Spearman 2 tiles east, and the War declaration core offers (it starts this turn). */
  function peace() {
    return wing(2, (view) => {
      const team = view.players.find((p) => p.id === 2)!.team;
      view.diplomacy = {
        ...view.diplomacy, legacy: false, relations: [{ a: view.team, b: team, state: 'Peace' }],
        actions: [{ order: { type: 'DeclareWar', against: team, breach: false }, effective_turn: view.turn, cost: 0, betrayal: false }],
      } as unknown as PlayerView['diplomacy'];
    });
  }
  const declare = (v: PlayerView): Order => ({ type: 'DeclareWar', against: v.players.find((p) => p.id === 2)!.team, breach: false });

  it('a target at Peace is not strikeable until War is queued, then it is, in the same turn', () => {
    const { v, city, bomber } = peace();
    expect(airState(v, city, bomber, [], undefined)).toMatchObject({ kind: 'idle', targets: 0 });
    expect(noTargetReason(v, city, 'Bomber')).toMatch(/not at War with you: declare War to strike\. The strike can go in the same turn/);
    // The picker still offers the tile, marked as needing a declaration.
    const [offered] = pickerCandidates(v, city, bomber, 'strike');
    expect(offered).toMatchObject({ at: fromOffset(10, 8), peaceTeam: v.players.find((p) => p.id === 2)!.team });
    const orders = [declare(v)];
    expect(airState(v, city, bomber, orders, undefined)).toMatchObject({ kind: 'ready', targets: 1 });
    expect(spotsIn(v, city, 5, orders).hostile[0]).toMatchObject({ surprise: true });
    expect(pickerCandidates(v, city, bomber, 'strike', orders)[0].peaceTeam).toBeUndefined();
    expect(airOverlayFor(v, city.id, { plane: 51, what: 'strike', hover: null }, orders)!.candidates).toEqual([fromOffset(10, 8)]);
  });

  it('a War that only starts next turn does not open the strike', () => {
    const { v, city, bomber } = peace();
    (v.diplomacy as unknown as { actions: { effective_turn: number }[] }).actions[0].effective_turn = v.turn + 1;
    expect(airState(v, city, bomber, [declare(v)], undefined)).toMatchObject({ kind: 'idle', targets: 0 });
  });
});

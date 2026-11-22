// Playtest G2: a melee attack targets the unit, not its tile. WASM checks for pursuit,
// the Peace block report and the surprise-War attack. G18: an attack order is one attack and
// lasts one turn. G20: a declaration at plain Peace takes effect this turn. G21: a unit attacked
// while pulling back takes the hit and completes its step.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event, Hex, Order } from '../types';
import { fromOffset, neighbors, within } from '../hex';
import { PEACE_BLOCK_PREFIX } from '../format';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture(peace = false) {
  const setup = makeStandardSetup('7', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, faction: 'Switzerland', team: i, is_human: true }));
  const s = createState(setup);
  for (const relation of s.diplomacy!.relations) relation.state = peace ? 'Peace' : 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.gold = 10000; p.explored.fill(0); spawnUnit(s, p.id, 'Settler', fromOffset(2 + 12 * p.id, 10), false); }
  return s;
}
/** Straight line a -> m -> b along direction 0, so the first step toward b is always m. */
function line(start: Hex) {
  const m = neighbors(start)[0];
  return { a: start, m, b: neighbors(m)[0] };
}
const relevant = (es: Event[]) => es.filter((e) => ['OrderFailed', 'UnitMoved', 'Battle', 'UnitDestroyed', 'MoveBlocked', 'Bounce', 'SettlerCaptured', 'AttackPursuit', 'WarDeclared'].includes(e.type));
function play(s: MState, turns: Record<number, Order[]>[]) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  const reports: Event[][] = [];
  for (const orders of turns) {
    for (const p of s.players) (expect(fixtureClient.validate(p.id, orders[p.id] ?? [])).toMatchSnapshot(), void (core.validate(p.id, orders[p.id] ?? [])));
    const actual = fixtureClient.endTurn(orders).events, expected = core.endTurn(orders).events;
    (expect(relevant(actual)).toMatchSnapshot(), void (relevant(expected))); reports.push(expected);
    for (const p of s.players) {
      const units = (c: FixtureClient | WasmClient) => c.view(p.id).units.filter((u) => u.owner === p.id)
        .map((u) => ({ id: u.id, pos: u.pos, hp: u.hp, goto: u.goto, path: u.path ?? [], attack_target: u.attack_target ?? null, needs_orders: !!u.needs_orders }))
        .sort((a, b) => a.id - b.id);
      (expect(units(fixtureClient)).toMatchSnapshot(), void (units(core)));
    }
    fixtureClient.load(fixtureClient.save()); core.load(core.save());
  }
  return { fixtureClient, core, reports };
}
const move = (unit: number, to: Hex): Order => ({ type: 'MoveUnit', unit, to });
const pursuit = (es: Event[], unit: number) => es.find((e) => e.type === 'AttackPursuit' && e.unit === unit);
const battlesOn = (es: Event[], unit: number) => es.flatMap((e) => e.type === 'Battle' && e.defender.kind === 'Unit' && e.defender.unit === unit ? [e.at] : []);

describe.skipIf(!available)('G2 attack pursuit core behavior', () => {
  beforeAll(async () => {
    const pkg = '../wasm-pkg/legatum.js', mod = await import(/* @vite-ignore */ pkg);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('a target that sidesteps within reach is attacked where it ends', () => {
    const s = fixture(); const { a, m, b } = line(fromOffset(4, 5)); const c = neighbors(m)[1];
    const soldier = spawnUnit(s, 0, 'Soldier', a, false), scout = spawnUnit(s, 1, 'Scout', b, false);
    const { reports } = play(s, [{ 0: [move(soldier.id, b)], 1: [move(scout.id, c)] }]);
    expect(battlesOn(reports[0], scout.id)).toEqual([c]);
    expect(pursuit(reports[0], soldier.id)).toMatchObject({ outcome: 'Attacked', target_at: c });
  });

  it('G18: a target that outruns the attacker is followed, then the order ends', () => {
    const s = fixture(); const { a, b } = line(fromOffset(3, 5)); const far = neighbors(neighbors(b)[0])[0];
    const soldier = spawnUnit(s, 0, 'Soldier', a, false), scout = spawnUnit(s, 1, 'Scout', b, false);
    const { core, reports } = play(s, [{ 0: [move(soldier.id, b)], 1: [move(scout.id, far)] }, {}]);
    expect(pursuit(reports[0], soldier.id)).toMatchObject({ outcome: 'Followed', target_at: far, target_kind: 'Scout' });
    expect(battlesOn(reports[0], scout.id)).toEqual([]);
    expect(battlesOn(reports[1], scout.id)).toEqual([]);
    expect(reports[0].filter((e) => e.type === 'UnitMoved' && e.unit === soldier.id).at(-1)).toMatchObject({ to: b });
    expect(core.view(0).units.find((u) => u.id === soldier.id)).toMatchObject({ pos: b, goto: null });
  });

  it('G18: an attack whose defender survives is one attack; the attacker needs orders', () => {
    const s = fixture(); const here = fromOffset(5, 5), there = neighbors(here)[0];
    s.terrain[idxOf(s, there)] = 'Hills';
    const soldier = spawnUnit(s, 0, 'Soldier', here, false), guard = spawnUnit(s, 1, 'Soldier', there, false);
    guard.fortify = 6;
    const { core, reports } = play(s, [{ 0: [move(soldier.id, there)] }, {}, {}]);
    expect(battlesOn(reports[0], guard.id)).toEqual([there]);
    expect(core.view(0).units.find((u) => u.id === soldier.id)).toMatchObject({ pos: here, goto: null });
    expect([...battlesOn(reports[1], guard.id), ...battlesOn(reports[2], guard.id)]).toEqual([]);
  });

  it('G18: an attack out of this turn\'s reach approaches, then ends', () => {
    const s = fixture(); const start = fromOffset(3, 5);
    let target = start; for (let i = 0; i < 4; i++) target = neighbors(target)[0];
    const soldier = spawnUnit(s, 0, 'Soldier', start, false), guard = spawnUnit(s, 1, 'Soldier', target, false);
    spawnUnit(s, 0, 'Scout', neighbors(neighbors(target)[2])[2], false);
    const { reports } = play(s, [{ 0: [move(soldier.id, target)] }, {}]);
    expect(pursuit(reports[0], soldier.id)).toMatchObject({ outcome: 'OutOfReach', target_at: target });
    expect(battlesOn(reports[1], guard.id)).toEqual([]);
  });

  it('G18: a meeting that both survive ends both moves', () => {
    const s = fixture();
    const a = spawnUnit(s, 0, 'Soldier', fromOffset(3, 5), false), b = spawnUnit(s, 1, 'Soldier', fromOffset(5, 5), false);
    const { core, reports } = play(s, [{ 0: [move(a.id, fromOffset(9, 5))], 1: [move(b.id, fromOffset(0, 5))] }]);
    expect(reports[0].filter((e) => e.type === 'Battle' && e.meeting)).toHaveLength(1);
    expect(core.view(0).units.find((u) => u.id === a.id)?.goto).toBeNull();
    expect(core.view(1).units.find((u) => u.id === b.id)?.goto).toBeNull();
  });

  it('G21: a unit attacked while pulling back takes the hit without fortify, then completes its step', () => {
    const s = fixture(); const here = fromOffset(5, 5), there = neighbors(here)[0], away = neighbors(neighbors(there)[0])[0];
    s.terrain[idxOf(s, there)] = 'Hills';
    const soldier = spawnUnit(s, 0, 'Soldier', here, false), ranger = spawnUnit(s, 1, 'Ranger', there, false);
    ranger.fortify = 6;
    const { core, reports } = play(s, [{ 0: [move(soldier.id, there)], 1: [move(ranger.id, away)] }]);
    const hit = reports[0].find((e) => e.type === 'Battle' && e.attacker === soldier.id);
    expect(hit).toMatchObject({ defender_leaving: true, at: there });
    expect(hit && hit.type === 'Battle' && hit.defender_strength.parts.some(([label]) => label.includes('Fortif'))).toBe(false);
    expect(reports[0].some((e) => e.type === 'MoveBlocked' && e.unit === ranger.id)).toBe(false);
    expect(core.view(1).units.find((u) => u.id === ranger.id)?.pos).not.toEqual(there);
    expect(core.view(0).units.find((u) => u.id === soldier.id)).toMatchObject({ pos: here, goto: null });
  });

  it('a target that leaves sight ends the order: no dangling route', () => {
    const s = fixture(); const { a, b } = line(fromOffset(2, 5));
    const gone = neighbors(neighbors(neighbors(b)[0])[0])[0];
    const soldier = spawnUnit(s, 0, 'Soldier', a, false), scout = spawnUnit(s, 1, 'Scout', b, false);
    const { core, reports } = play(s, [{ 0: [move(soldier.id, b)], 1: [move(scout.id, gone)] }]);
    expect(pursuit(reports[0], soldier.id)).toMatchObject({ outcome: 'LostSight', target_at: null });
    const v = core.view(0).units.find((u) => u.id === soldier.id)!;
    expect([v.goto, v.attack_target ?? null]).toEqual([null, null]);
  });

  it('a pursuit never attacks an escort it was not ordered to attack', () => {
    const s = fixture(); const { a, m, b } = line(fromOffset(4, 5)); const c = neighbors(m)[1];
    const soldier = spawnUnit(s, 0, 'Soldier', a, false), settler = spawnUnit(s, 1, 'Settler', b, false);
    spawnUnit(s, 1, 'Soldier', c, false);
    const { reports } = play(s, [{ 0: [move(soldier.id, b)], 1: [move(settler.id, c)] }]);
    expect(reports[0].some((e) => e.type === 'Battle')).toBe(false);
    expect(pursuit(reports[0], soldier.id)).toMatchObject({ outcome: 'Followed', target_at: c });
  });

  it('a move onto a unit of a team at Peace names it instead of a silent bounce', () => {
    const s = fixture(true); const here = fromOffset(5, 5), there = neighbors(here)[0];
    const soldier = spawnUnit(s, 0, 'Soldier', here, false), scout = spawnUnit(s, 1, 'Scout', there, false);
    const { reports } = play(s, [{ 0: [move(soldier.id, there)], 1: [move(scout.id, neighbors(there)[1])] }]);
    expect(reports[0]).toContainEqual(expect.objectContaining({ type: 'MoveBlocked', unit: soldier.id, reason: `${PEACE_BLOCK_PREFIX}${s.players[1].name}'s Scout` }));
    expect(pursuit(reports[0], soldier.id)).toBeUndefined();
  });

  it('G20: declaring War at plain Peace attacks this turn', () => {
    const s = fixture(true); const here = fromOffset(5, 5), there = neighbors(here)[0], step = neighbors(there)[1];
    const soldier = spawnUnit(s, 0, 'Soldier', here, false), scout = spawnUnit(s, 1, 'Scout', there, false);
    const { reports } = play(s, [{ 0: [{ type: 'DeclareWar', against: 1, breach: false }, move(soldier.id, there)], 1: [move(scout.id, step)] }]);
    expect(reports[0].find((e) => e.type === 'WarDeclared')).toMatchObject({ relation: { effective_turn: s.turn } });
    expect(battlesOn(reports[0], scout.id)).toHaveLength(1);
    expect(reports[0].some((e) => e.type === 'AttackPursuit' && e.outcome === 'AwaitingWar')).toBe(false);
    expect(reports[0].findIndex((e) => e.type === 'WarDeclared')).toBeLessThan(reports[0].findIndex((e) => e.type === 'Battle'));
  });

  it('war on an ally holds the attack this turn, then attacks the unit where it went', () => {
    const s = fixture(true); const here = fromOffset(5, 5), there = neighbors(here)[0], step = neighbors(there)[1];
    for (const r of s.diplomacy!.relations) { r.state = 'Alliance'; r.security = { id: 1, started_turn: 0, protected_until: 0 }; }
    const soldier = spawnUnit(s, 0, 'Soldier', here, false), scout = spawnUnit(s, 1, 'Scout', there, false);
    const declare: Record<number, Order[]> = { 0: [{ type: 'DeclareWar', against: 1, breach: true }, move(soldier.id, there)], 1: [move(scout.id, step)] };
    const held = play(s, [declare]);
    for (const c of [held.fixtureClient, held.core]) {
      expect(c.view(0).units.find((u) => u.id === soldier.id)).toMatchObject({ pos: here, goto: step, attack_target: scout.id });
    }
    const { reports } = play(s, [declare, {}]);
    expect(reports[0].some((e) => e.type === 'MoveBlocked' || e.type === 'OrderFailed')).toBe(false);
    expect(pursuit(reports[0], soldier.id)).toMatchObject({ outcome: 'AwaitingWar', target_at: step });
    expect(reports[0].some((e) => e.type === 'UnitMoved' && e.unit === soldier.id)).toBe(false);
    expect(reports[0]).toContainEqual(expect.objectContaining({ type: 'WarDeclared' }));
    expect(battlesOn(reports[1], scout.id)).toEqual([step]);
  });

  it('G20: declaring War with a move onto their city marches this turn and attacks it once', () => {
    const s = fixture(true); const cityAt = fromOffset(10, 5);
    s.cities.push({ id: 0, owner: 1, founder: 1, name: 'Redoubt', pos: cityAt, population: 3, food: 0, prod: 0, producing: null, bought: null,
      buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, originalCapital: true, foundedTurn: 1, resistance: 0 });
    s.nextCityId = 1;
    for (const h of within(cityAt, 1)) { const i = idxOf(s, h); s.owner[i] = 1; s.tileCity[i] = 0; }
    const here = neighbors(neighbors(cityAt)[3])[3];
    const soldier = spawnUnit(s, 0, 'Soldier', here, false);
    const { reports } = play(s, [{ 0: [{ type: 'DeclareWar', against: 1, breach: false }, move(soldier.id, cityAt)] }, {}, {}]);
    expect(reports[0].some((e) => e.type === 'MoveBlocked' || e.type === 'OrderFailed')).toBe(false);
    expect(reports[0].some((e) => e.type === 'UnitMoved' && e.unit === soldier.id)).toBe(true);
    expect(reports.flat().filter((e) => e.type === 'Battle' && e.attacker === soldier.id && e.defender.kind === 'City')).toHaveLength(1);
  });
});

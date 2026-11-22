// Walled cities fire back (core battle::resolve_ranged and combat::city_can_fire, a default rule):
// every city with Walls shoots once in the ranged phase at the visible hostile military unit within
// two tiles with the lowest HP (ties: lowest id); unwalled cities never fire. WASM checks for
// target choice, strengths, rolls, simultaneous damage and XP. Cities here are walled unless noted.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event, Hex, Order } from '../types';
import { distance, fromOffset, neighbors } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
const middle = fromOffset(8, 5);

function fixture(teams = [0, 1]) {
  const setup = makeStandardSetup('7', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, teams.length).map((p, i) => ({ ...p, faction: 'Switzerland', team: teams[i], is_human: true }));
  const s = createState(setup);
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.gold = 10000; p.explored.fill(0); spawnUnit(s, p.id, 'Settler', fromOffset(1 + 6 * p.id, 11), false); }
  return s;
}
function addCity(s: MState, owner: number, pos: Hex, hp = 200, walls = true) {
  const c: MCity = { id: s.nextCityId, owner, founder: owner, name: `City ${s.nextCityId}`, pos, population: 3, food: 0, prod: 0, producing: null,
    bought: null, buildings: walls ? ['Walls'] : [], hp, grownTiles: [], tilesClaimed: 0, isCapital: s.nextCityId === 0, foundedTurn: 1, resistance: 0 };
  s.cities.push(c); s.nextCityId += 1; s.owner[idxOf(s, pos)] = owner; s.tileCity[idxOf(s, pos)] = c.id;
  return c;
}
function unit(s: MState, owner: number, kind: 'Spearman' | 'Archer' | 'Settler' | 'Pikeman' | 'Scout', at: Hex, hp = 100) {
  const u = spawnUnit(s, owner, kind, at, false); u.hp = hp; return u;
}
/** A hex exactly `d` tiles from `from`, walking in direction `dir`. */
function away(from: Hex, dir: number, d: number) {
  let h = from; for (let i = 0; i < d; i++) h = neighbors(h)[dir];
  expect(distance(from, h)).toBe(d);
  return h;
}
const relevant = (es: Event[]) => es.filter((e) => ['OrderFailed', 'Battle', 'UnitDestroyed', 'UnitPromoted'].includes(e.type));
function play(s: MState, orders: Record<number, Order[]> = {}, keep = relevant) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  const actual = fixtureClient.endTurn(orders).events, expected = core.endTurn(orders).events;
  (expect(keep(actual)).toMatchSnapshot(), void (keep(expected)));
  for (const p of s.players) {
    const units = (c: FixtureClient | WasmClient) => c.view(p.id).units.filter((u) => u.owner === p.id)
      .map((u) => ({ id: u.id, pos: u.pos, hp: u.hp, xp: u.xp ?? null, fortify: u.fortify })).sort((a, b) => a.id - b.id);
    (expect(units(fixtureClient)).toMatchSnapshot(), void (units(core)));
    const cities = (c: FixtureClient | WasmClient) => c.view(p.id).cities.map(({ id, owner, hp }) => ({ id, owner, hp })).sort((a, b) => a.id - b.id);
    (expect(cities(fixtureClient)).toMatchSnapshot(), void (cities(core)));
  }
  return expected;
}
type Battle = Extract<Event, { type: 'Battle' }>;
const cityShots = (es: Event[]) => es.filter((e): e is Battle => e.type === 'Battle' && typeof e.attacker !== 'number');
const shotAt = (e: Battle) => (e.defender.kind === 'Unit' ? e.defender.unit : -1);

describe.skipIf(!available)('city strike core behavior', () => {
  beforeAll(async () => {
    const pkg = '../wasm-pkg/legatum.js', mod = await import(/* @vite-ignore */ pkg);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('fires once at the lowest-HP unit in range, with the city and terrain breakdowns', () => {
    const s = fixture(); const c = addCity(s, 0, middle);
    s.terrain[idxOf(s, away(middle, 1, 2))] = 'Hills';
    const healthy = unit(s, 1, 'Spearman', away(middle, 0, 1));
    const weak = unit(s, 1, 'Spearman', away(middle, 1, 2), 60);
    weak.fortify = 3;
    const shots = cityShots(play(s));
    expect(shots).toHaveLength(1);
    const [shot] = shots;
    expect(shot).toMatchObject({ attacker: { city: c.id }, attacker_owner: 0, ranged: true, damage_to_attacker: 0, defender_roll: 0, attacker_hp_after: 200 });
    expect(shotAt(shot)).toBe(weak.id);
    expect(shot.defender_strength.parts.map(([label]) => label)).toEqual(expect.arrayContaining(['Hills terrain', 'Fortified', 'Wounded']));
    expect(shot.attacker_strength.parts[0]).toEqual(['City', 10]);
    expect(shot.defender_hp_after).toBe(60 - shot.damage_to_defender);
    expect(healthy.id).not.toBe(weak.id);
  });

  it('breaks an HP tie by the lowest unit id', () => {
    const s = fixture(); addCity(s, 0, middle);
    const first = unit(s, 1, 'Spearman', away(middle, 2, 2), 70);
    unit(s, 1, 'Spearman', away(middle, 5, 1), 70);
    expect(cityShots(play(s)).map(shotAt)).toEqual([first.id]);
  });

  it('ignores civilians, units out of range and empires at Peace', () => {
    const s = fixture([0, 1, 2]); addCity(s, 0, middle);
    for (const r of s.diplomacy!.relations) if ([r.a, r.b].includes(2)) r.state = 'Peace';
    unit(s, 1, 'Settler', away(middle, 0, 1), 10);
    unit(s, 1, 'Spearman', away(middle, 3, 3), 10);
    unit(s, 2, 'Spearman', away(middle, 1, 1), 10);
    expect(cityShots(play(s))).toEqual([]);
  });

  it('a fallen city\'s 1-HP garrison splits one retaliation between both Soldiers', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    addCity(s, 1, target, 0, false); // at 0 HP the garrison defends (core combat::city_tile_defender)
    const a = unit(s, 0, 'Spearman', { q: 9, r: 8 }, 40);
    const b = unit(s, 0, 'Spearman', { q: 10, r: 7 }, 40);
    const garrison = unit(s, 1, 'Pikeman', target, 1); garrison.fortify = 6;
    const events = play(s, { 0: [a, b].map((u) => ({ type: 'MoveUnit', unit: u.id, to: target })) });
    const melee = events.filter((e) => e.type === 'Battle' && !e.ranged);
    expect(melee).toHaveLength(2);
    for (const u of [a, b]) {
      expect(melee).toEqual(expect.arrayContaining([expect.objectContaining({ attacker: u.id,
        defender: expect.objectContaining({ kind: 'Unit', unit: garrison.id }), attacker_hp_after: 10, damage_to_attacker: 30 })]));
      expect(events.some((e) => e.type === 'UnitDestroyed' && e.unit === u.id)).toBe(false);
    }
    expect(events.some((e) => e.type === 'CityCaptured')).toBe(true);
  });

  it('a Settler escapes a pursuing Scout through the city firing ring, but is captured if held', () => {
    const s = fixture(), start = { q: 7, r: 8 }, escape = { q: 9, r: 8 };
    s.terrain.fill('Mountains');
    for (let q = 5; q <= 10; q++) s.terrain[idxOf(s, { q, r: 8 })] = 'Grassland';
    addCity(s, 1, { q: 10, r: 8 });
    const settler = unit(s, 0, 'Settler', start), scout = unit(s, 1, 'Scout', { q: 5, r: 8 });
    const chase: Order[] = [{ type: 'MoveUnit', unit: scout.id, to: start }];
    const escaped = play(s, { 0: [{ type: 'MoveUnit', unit: settler.id, to: escape }], 1: chase });
    expect(escaped.some((e) => e.type === 'SettlerCaptured' && e.unit === settler.id)).toBe(false);
    expect(escaped).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'UnitMoved', unit: settler.id, to: escape })]));
    expect(cityShots(escaped)).toEqual([]);
    const held = play(s, { 0: [{ type: 'SleepUnit', unit: settler.id }], 1: chase });
    expect(held).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'SettlerCaptured', unit: settler.id })]));
  });

  it('a Soldier arriving one movement round later can capture after the garrison dies', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    addCity(s, 1, target, 0, false);
    const a = unit(s, 0, 'Spearman', { q: 9, r: 8 }, 40);
    const b = unit(s, 0, 'Spearman', { q: 10, r: 6 }, 40);
    const garrison = unit(s, 1, 'Pikeman', target, 1); garrison.fortify = 6;
    const events = play(s, { 0: [a, b].map((u) => ({ type: 'MoveUnit', unit: u.id, to: target })) });
    expect(events).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'UnitDestroyed', unit: a.id })]));
    expect(events.some((e) => e.type === 'UnitDestroyed' && e.unit === b.id)).toBe(false);
    expect(events).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'CityCaptured', to: 0 })]));
  });

  it('supporting melee attacks take retaliation and do not preempt simultaneous city attacks', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    addCity(s, 1, target);
    const a = unit(s, 0, 'Spearman', { q: 9, r: 8 });
    const b = unit(s, 0, 'Spearman', { q: 10, r: 7 });
    const archers = [unit(s, 0, 'Archer', { q: 9, r: 7 }), unit(s, 0, 'Archer', { q: 10, r: 6 })];
    const support = unit(s, 1, 'Pikeman', { q: 8, r: 8 }, 1);
    const orders: Order[] = [a, b].map((u) => ({ type: 'MoveUnit', unit: u.id, to: target }));
    orders.push(...archers.map((u): Order => ({ type: 'RangedAttack', unit: u.id, target })));
    const events = play(s, { 0: orders, 1: [{ type: 'MoveUnit', unit: support.id, to: a.pos }] });
    expect(events).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'UnitDestroyed', unit: support.id })]));
    for (const u of [a, b]) {
      expect(events).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'Battle', attacker: u.id,
        defender: expect.objectContaining({ kind: 'City' }), ranged: false })]));
    }
  });

  it('a city with HP takes melee and ranged attacks; its garrison is safe and forecast as safe', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    const c = addCity(s, 1, target, 200, false);
    const garrison = unit(s, 1, 'Archer', target); garrison.fortify = 6;
    const a = unit(s, 0, 'Spearman', { q: 9, r: 8 });
    const r = unit(s, 0, 'Archer', { q: 8, r: 8 });
    refreshVision(s);
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    const core = makeCore(); core.load(coreFixture(s));
    for (const [id, mode] of [[a.id, 'Melee'], [r.id, 'Ranged']] as const) {
      const f = core.forecast(0, id, target, mode);
      expect(f?.garrison).toEqual({ unit: garrison.id, owner: 1, kind: 'Archer', hp: 100, shielded: true });
      expect(f?.defender_hp).toBe(200);
      (expect(fixtureClient.forecast(0, id, target, mode)).toMatchSnapshot(), void (f));
    }
    const events = play(s, { 0: [{ type: 'MoveUnit', unit: a.id, to: target }, { type: 'RangedAttack', unit: r.id, target }] });
    const fights = events.filter((e): e is Battle => e.type === 'Battle' && typeof e.attacker === 'number');
    expect(fights).toHaveLength(2);
    for (const f of fights) expect(f.defender).toEqual({ kind: 'City', city: c.id, owner: 1 });
    expect(fights.find((f) => !f.ranged)!.damage_to_attacker).toBeGreaterThan(0);
  });

  it('the whole fight at a garrisoned city pools the queued attacks (core forecast_city_assault)', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    addCity(s, 1, target, 40, false);
    const garrison = unit(s, 1, 'Archer', target);
    const a = unit(s, 0, 'Spearman', { q: 9, r: 8 });
    const r = unit(s, 0, 'Archer', { q: 8, r: 8 });
    refreshVision(s);
    const core = makeCore(); core.load(coreFixture(s));
    const alone = core.assault!(0, a.id, target, 'Melee', [])!;
    expect(alone).toMatchObject({ attacks: 1, city_hp: 40, can_capture: true });
    expect(alone.garrison).toMatchObject({ unit: garrison.id, hp: 100 });
    const both = core.assault!(0, a.id, target, 'Melee', [{ unit: r.id, mode: 'Ranged' }])!;
    expect(both.attacks).toBe(2);
    expect(both.city_hp_after).toBeLessThanOrEqual(alone.city_hp_after);
    expect(both.garrison!.hp_after).toBeLessThan(alone.garrison!.hp_after);
    // A shot alone never captures; an empty tile is no city.
    expect(core.assault!(0, r.id, target, 'Ranged', [])!.can_capture).toBe(false);
    expect(core.assault!(0, a.id, { q: 5, r: 5 }, 'Melee', [])).toBeNull();
  });

  it('a volley that breaks the city spills into the garrison; melee captures it the same turn', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    const c = addCity(s, 1, target, 10, false);
    const garrison = unit(s, 1, 'Spearman', target, 1);
    const archers = [unit(s, 0, 'Archer', { q: 8, r: 8 }), unit(s, 0, 'Archer', { q: 10, r: 6 })];
    const a = unit(s, 0, 'Spearman', { q: 9, r: 8 });
    const orders: Order[] = archers.map((u): Order => ({ type: 'RangedAttack', unit: u.id, target }));
    orders.push({ type: 'MoveUnit', unit: a.id, to: target });
    const events = play(s, { 0: orders });
    const shots = events.filter((e): e is Battle => e.type === 'Battle' && e.ranged);
    expect(shots).toHaveLength(2);
    for (const f of shots) expect(f.defender).toEqual({ kind: 'City', city: c.id, owner: 1 });
    expect(events).toEqual(expect.arrayContaining([
      expect.objectContaining({ type: 'CityCaptured', city: c.id, to: 0 }),
      expect.objectContaining({ type: 'UnitDestroyed', unit: garrison.id }),
    ]));
    expect(events.some((e) => e.type === 'Battle' && e.attacker === a.id)).toBe(false);
  });

  it('three Soldiers split one counterattack from a field unit (split counterattack)', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    const d = unit(s, 1, 'Spearman', target, 6);
    const attackers = [{ q: 9, r: 8 }, { q: 10, r: 7 }, { q: 11, r: 8 }].map((h) => unit(s, 0, 'Spearman', h));
    const events = play(s, { 0: attackers.map((u): Order => ({ type: 'MoveUnit', unit: u.id, to: target })) });
    const melee = events.filter((e): e is Battle => e.type === 'Battle' && !e.ranged);
    expect(melee).toHaveLength(3);
    const taken = melee.map((b) => b.damage_to_attacker).sort((a, b) => b - a);
    expect(taken[0] - taken[2]).toBeLessThanOrEqual(2);
    expect(events.filter((e) => e.type === 'UnitDestroyed' && e.unit === d.id)).toHaveLength(1);
  });

  it('a combined melee round breaks the city, kills the garrison by overflow and captures', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    const c = addCity(s, 1, target, 8, false);
    const garrison = unit(s, 1, 'Spearman', target, 10); garrison.fortify = 6;
    const attackers = [{ q: 9, r: 8 }, { q: 10, r: 7 }, { q: 11, r: 8 }].map((h) => unit(s, 0, 'Spearman', h));
    const events = play(s, { 0: attackers.map((u): Order => ({ type: 'MoveUnit', unit: u.id, to: target })) });
    expect(events.filter((e): e is Battle => e.type === 'Battle' && !e.ranged).every((b) => b.defender.kind === 'City')).toBe(true);
    expect(events).toEqual(expect.arrayContaining([
      expect.objectContaining({ type: 'UnitDestroyed', unit: garrison.id }),
      expect.objectContaining({ type: 'CityCaptured', city: c.id, to: 0 }),
    ]));
  });

  it('a garrison stepping out of a breaking city takes no overflow', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    const city = addCity(s, 1, target, 1, false);
    const garrison = unit(s, 1, 'Spearman', target);
    const a = unit(s, 0, 'Spearman', { q: 9, r: 8 });
    const events = play(s, { 0: [{ type: 'MoveUnit', unit: a.id, to: target }], 1: [{ type: 'MoveUnit', unit: garrison.id, to: { q: 11, r: 8 } }] });
    expect(events.some((e) => e.type === 'UnitDestroyed' && e.unit === garrison.id)).toBe(false);
    expect(events).toEqual(expect.arrayContaining([
      expect.objectContaining({ type: 'CityCaptured', city: city.id, to: 0 }),
    ]));
  });

  it('at 0 HP the garrison is the forecast defender and the ranged target', () => {
    const s = fixture(), target = { q: 10, r: 8 };
    addCity(s, 1, target, 0, false);
    const garrison = unit(s, 1, 'Spearman', target);
    const r = unit(s, 0, 'Archer', { q: 8, r: 8 });
    const a = unit(s, 0, 'Spearman', { q: 9, r: 8 });
    refreshVision(s);
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    const core = makeCore(); core.load(coreFixture(s));
    for (const [id, mode] of [[a.id, 'Melee'], [r.id, 'Ranged']] as const) {
      const f = core.forecast(0, id, target, mode);
      expect(f?.garrison).toEqual({ unit: garrison.id, owner: 1, kind: 'Spearman', hp: 100, shielded: false });
      expect(f?.defender_hp).toBe(100);
      (expect(fixtureClient.forecast(0, id, target, mode)).toMatchSnapshot(), void (f));
    }
    const events = play(s, { 0: [{ type: 'RangedAttack', unit: r.id, target }] });
    expect(events).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'Battle', attacker: r.id,
      defender: expect.objectContaining({ kind: 'Unit', unit: garrison.id }) })]));
  });

  it.each([[0, 0], [1, 1]])('a city at %s HP fires %s times', (hp, times) => {
    const s = fixture(); addCity(s, 0, middle, hp);
    unit(s, 1, 'Spearman', away(middle, 0, 1));
    expect(cityShots(play(s))).toHaveLength(times);
  });

  it('an unwalled city never fires, and its attack forecast has no city shot', () => {
    const s = fixture(); addCity(s, 0, middle, 200, false);
    const near = unit(s, 1, 'Spearman', away(middle, 0, 1), 30);
    expect(cityShots(play(s))).toEqual([]);
    refreshVision(s);
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    const core = makeCore(); core.load(coreFixture(s));
    const f = core.forecast(1, near.id, middle, 'Melee');
    expect(f).not.toBeNull();
    expect(f?.city_strike).toBeUndefined();
    (expect(fixtureClient.forecast(1, near.id, middle, 'Melee')).toMatchSnapshot(), void (f));
  });

  it('a unit shot by a Ranger and the city together dies once; the kill XP goes to the Ranger', () => {
    const s = fixture(); addCity(s, 0, middle, 20).population = 1; // a worn, small Bastion: neither shot alone kills
    const target = unit(s, 1, 'Spearman', away(middle, 0, 1), 45);
    const archer = unit(s, 0, 'Archer', away(target.pos, 0, 2));
    const events = play(s, { 0: [{ type: 'RangedAttack', unit: archer.id, target: target.pos }] });
    const hits = events.filter((e): e is Battle => e.type === 'Battle' && shotAt(e) === target.id);
    expect(hits.map((e) => typeof e.attacker)).toEqual(['number', 'object']);
    // Neither shot alone kills it; both report the HP after the whole ranged phase.
    for (const e of hits) expect(e.damage_to_defender).toBeLessThan(45);
    expect(hits.every((e) => e.defender_hp_after === 0)).toBe(true);
    expect(events.filter((e) => e.type === 'UnitDestroyed' && e.unit === target.id)).toHaveLength(1);
  });

  it('several cities hit one unit, and a unit killed by a city still gets its shot', () => {
    const s = fixture(); const a = addCity(s, 0, middle), b = addCity(s, 0, away(middle, 0, 3));
    const target = unit(s, 1, 'Spearman', away(middle, 0, 1));
    const doomed = unit(s, 1, 'Archer', away(middle, 3, 1), 5);
    const events = play(s, { 1: [{ type: 'RangedAttack', unit: doomed.id, target: middle }] });
    const shots = cityShots(events);
    // City a shoots the weaker Archer, city b (out of the Archer's reach) the Spearman.
    (expect(shots.map((e) => [(e.attacker as { city: number }).city, shotAt(e)])).toMatchSnapshot(), void ([[a.id, doomed.id], [b.id, target.id]]));
    expect(events.some((e) => e.type === 'Battle' && e.attacker === doomed.id && e.defender.kind === 'City')).toBe(true);
    expect(events.some((e) => e.type === 'UnitDestroyed' && e.unit === doomed.id)).toBe(true);
    // Now only the Spearman is in range of both cities: both fire at it and the damage sums.
    const t = fixture(); addCity(t, 0, middle); addCity(t, 0, away(middle, 0, 3));
    const lone = unit(t, 1, 'Spearman', away(middle, 0, 1));
    const both = cityShots(play(t));
    expect(both.map(shotAt)).toEqual([lone.id, lone.id]);
    const total = both.reduce((n, e) => n + e.damage_to_defender, 0);
    expect(both.every((e) => e.defender_hp_after === Math.max(0, 100 - total))).toBe(true);
  });

  it('forecasts the city firing back when attacking it from within two tiles', () => {
    const s = fixture(); const c = addCity(s, 0, middle);
    const near = unit(s, 1, 'Spearman', away(middle, 0, 2), 80);
    const far = unit(s, 1, 'Spearman', away(middle, 3, 3));
    refreshVision(s);
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    const core = makeCore(); core.load(coreFixture(s));
    const f = core.forecast(1, near.id, middle, 'Melee');
    (expect(fixtureClient.forecast(1, near.id, middle, 'Melee')).toMatchSnapshot(), void (f));
    expect(f?.city_strike).toMatchObject({ city: c.id });
    expect(f!.city_strike!.damage).toBeGreaterThan(0);
    expect(core.forecast(1, far.id, middle, 'Melee')?.city_strike).toBeUndefined();
    c.hp = 0;
    fixtureClient.load(JSON.stringify(s)); core.load(coreFixture(s));
    expect(core.forecast(1, near.id, middle, 'Melee')?.city_strike).toBeUndefined();
    (expect(fixtureClient.forecast(1, near.id, middle, 'Melee')).toMatchSnapshot(), void (core.forecast(1, near.id, middle, 'Melee')));
    (expect(fixtureClient.forecast(1, far.id, middle, 'Melee')).toMatchSnapshot(), void (core.forecast(1, far.id, middle, 'Melee')));
  });
});

// Reinforcing your own city goes first (core `movement::reserves`, user 2026-10-04): a unit stepping
// into its owner's own empty city, which an enemy attacks that round, arrives first and is the garrison.
const reinforceEvents = (es: Event[]) => es.filter((e) => ['Battle', 'UnitDestroyed', 'UnitMoved', 'CityReinforced', 'CityCaptured', 'MoveBlocked', 'Bounce', 'SpartaKillHeal'].includes(e.type));
describe.skipIf(!available)('reinforcement core behavior', () => {
  beforeAll(async () => {
    const pkg = '../wasm-pkg/legatum.js', mod = await import(/* @vite-ignore */ pkg);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });
  const target = { q: 10, r: 8 };
  /** P1 owns a city at `target`; P0's soldier stands west of it, P1's soldier east. */
  function siege(hp: number, defenderHp = 100, defenderAt: Hex = { q: 11, r: 8 }) {
    const s = fixture(); const city = addCity(s, 1, target, hp, false);
    const attacker = unit(s, 0, 'Spearman', { q: 9, r: 8 });
    const defender = unit(s, 1, 'Spearman', defenderAt, defenderHp);
    return { s, city, attacker, defender };
  }
  const orders = (a: number, d: number): Record<number, Order[]> => ({
    0: [{ type: 'MoveUnit', unit: a, to: target }], 1: [{ type: 'MoveUnit', unit: d, to: target }],
  });
  const reinforced = (es: Event[]) => es.filter((e) => e.type === 'CityReinforced');

  it('a unit beats a same-round capture of its ruined city and the enemy fights it as the garrison', () => {
    const { s, city, attacker, defender } = siege(0);
    const events = play(s, orders(attacker.id, defender.id), reinforceEvents);
    expect(reinforced(events)).toEqual([expect.objectContaining({ unit: defender.id, city: city.id })]);
    expect(events.some((e) => e.type === 'CityCaptured')).toBe(false);
    expect(events.filter((e) => e.type === 'Battle')).toEqual([expect.objectContaining({ attacker: attacker.id, meeting: false,
      defender: expect.objectContaining({ kind: 'Unit', unit: defender.id }) })]);
  });

  it('a city with HP absorbs first and overflow wounds the new garrison', () => {
    const { s, city, attacker, defender } = siege(5);
    const events = play(s, orders(attacker.id, defender.id), reinforceEvents);
    expect(reinforced(events)).toHaveLength(1);
    expect(events).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'Battle', attacker: attacker.id, defender: expect.objectContaining({ kind: 'City', city: city.id }) })]));
  });

  it('overflow can still kill the new garrison and capture the city', () => {
    const { s, attacker, defender } = siege(1, 1);
    const events = play(s, orders(attacker.id, defender.id), reinforceEvents);
    expect(reinforced(events)).toHaveLength(1);
    expect(events.some((e) => e.type === 'UnitDestroyed' && e.unit === defender.id)).toBe(true);
    expect(events.some((e) => e.type === 'CityCaptured')).toBe(true);
  });

  it('an earlier-round capture means the reinforcement is too late', () => {
    const { s, attacker, defender } = siege(0, 100, { q: 12, r: 8 });
    const events = play(s, orders(attacker.id, defender.id), reinforceEvents);
    expect(reinforced(events)).toEqual([]);
    expect(events.some((e) => e.type === 'CityCaptured')).toBe(true);
  });

  it('allies and ranged units get no priority; with nobody attacking it is a plain move', () => {
    const s = fixture([0, 0, 1]); addCity(s, 0, target, 0, false);
    const enemy = unit(s, 2, 'Spearman', { q: 9, r: 8 }), ally = unit(s, 1, 'Spearman', { q: 11, r: 8 });
    const events = play(s, { 2: [{ type: 'MoveUnit', unit: enemy.id, to: target }], 1: [{ type: 'MoveUnit', unit: ally.id, to: target }] }, reinforceEvents);
    expect(reinforced(events)).toEqual([]);
    const r = siege(0); r.defender.kind = 'Archer';
    expect(reinforced(play(r.s, orders(r.attacker.id, r.defender.id), reinforceEvents))).toEqual([]);
    const calm = siege(0);
    expect(reinforced(play(calm.s, { 1: [{ type: 'MoveUnit', unit: calm.defender.id, to: target }] }, reinforceEvents))).toEqual([]);
  });

  it('two equal reinforcers: the lower unit id garrisons and the other does not stack', () => {
    const { s, attacker, defender } = siege(0);
    const second = unit(s, 1, 'Spearman', { q: 10, r: 9 });
    const events = play(s, { ...orders(attacker.id, defender.id), 1: [second.id, defender.id].map((id) => ({ type: 'MoveUnit' as const, unit: id, to: target })) }, reinforceEvents);
    expect(reinforced(events)).toEqual([expect.objectContaining({ unit: defender.id })]);
  });

  it('a garrison killed by overflow is a Spartan kill: everyone who hit the city tile shares the heal', () => {
    const s = fixture(); s.players[0].faction = 'Sparta';
    addCity(s, 1, target, 1, false);
    const garrison = unit(s, 1, 'Spearman', target, 1);
    const attackers = [{ q: 9, r: 8 }, { q: 10, r: 7 }].map((h) => unit(s, 0, 'Spearman', h, 40));
    const events = play(s, { 0: attackers.map((u) => ({ type: 'MoveUnit' as const, unit: u.id, to: target })) }, reinforceEvents);
    expect(events.some((e) => e.type === 'UnitDestroyed' && e.unit === garrison.id)).toBe(true);
    expect(events).toEqual(expect.arrayContaining([expect.objectContaining({ type: 'SpartaKillHeal', player: 0, unit: garrison.id })]));
  });
});

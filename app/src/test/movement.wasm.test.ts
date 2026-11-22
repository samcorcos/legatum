import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event, Hex, Order, UnitKind } from '../types';
import { fromOffset, neighbors } from '../hex';
import { WasmClient } from '../wasmClient';
import { previewRoute } from '../path';
import { unitsNeedingOrders } from '../orders';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { buildView } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
const middle = fromOffset(5, 5);
function fixture(teams = [0, 1, 2], seed = '7') {
  const setup = makeStandardSetup(seed, 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, teams.length).map((p, i) => ({ ...p, faction: 'Switzerland', team: teams[i], is_human: true }));
  const s = createState(setup);
  // Combat scenarios assume War explicitly; Peace scenarios below override this.
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.gold = 10000; p.explored.fill(0); spawnUnit(s, p.id, 'Settler', fromOffset(2 + 6 * p.id, 10), false); }
  return s;
}
function addCity(s: MState, owner: number, hp = 0, pos = middle) {
  const c: MCity = { id: 1, owner, founder: owner, name: 'Redoubt', pos, population: 3, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 };
  s.cities.push(c); s.nextCityId = 1; s.owner[idxOf(s, pos)] = owner; s.tileCity[idxOf(s, pos)] = c.id;
  return c;
}
function mover(s: MState, owner: number, kind: UnitKind, from: Hex, to: Hex, hp = 100) {
  const u = spawnUnit(s, owner, kind, from, false); u.goto = to; u.hp = hp; return u;
}
function play(s: MState, turns: Record<number, Order[]>[] = [{}]) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  const reports: Event[][] = [];
  for (const orders of turns) {
    fixtureClient.endTurn(orders);
    reports.push(core.endTurn(orders).events);
    fixtureClient.load(fixtureClient.save()); core.load(core.save());
  }
  return { fixtureClient, core, reports };
}

it('fixtureClient route views copy both the array and its hexes', () => {
  const s = fixture(); const u = mover(s, 0, 'Soldier', neighbors(middle)[0], neighbors(middle)[3]); u.path = [middle, u.goto!];
  refreshVision(s); const view = buildView(s, 0), shown = view.units.find((v) => v.id === u.id)!;
  shown.path![0].q += 1; shown.path!.pop();
  expect(u.path).toEqual([middle, u.goto]);
});

describe.skipIf(!available)('movement WASM behavior', () => {
  beforeAll(async () => {
    const pkg = '../wasm-pkg/legatum.js', mod = await import(/* @vite-ignore */ pkg);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });
  it('Mongol Scouts and Settlers obey the closed Peace border (fixtureClient and core agree)', () => {
    const s = fixture([0, 1]); s.players[0].faction = 'Mongols'; s.setup.players[0].faction = 'Mongols';
    for (const r of s.diplomacy!.relations) r.state = 'Peace';
    for (const p of s.players) p.explored.fill(1);
    for (let row = 0; row < 12; row++) for (let col = 5; col <= 8; col++) s.owner[idxOf(s, fromOffset(col, row))] = 1;
    const goal = (row: number) => fromOffset(6, row);
    const soldier = spawnUnit(s, 0, 'Soldier', fromOffset(3, 2), false);
    const scout = spawnUnit(s, 0, 'Scout', fromOffset(3, 4), false);
    const settler = spawnUnit(s, 0, 'Settler', fromOffset(3, 6), false);
    const orders: Order[] = [
      { type: 'MoveUnit', unit: soldier.id, to: goal(2) },
      { type: 'MoveUnit', unit: scout.id, to: goal(4) },
      { type: 'MoveUnit', unit: settler.id, to: goal(6) },
    ];
    const { fixtureClient, core } = play(s, [{ 0: orders.slice(1) }]);
    for (const c of [fixtureClient, core]) {
      const at = (id: number) => c.view(0).units.find((u) => u.id === id)!.pos;
      expect(c.view(0).tiles[idxOf(s, at(soldier.id))].owner).not.toBe(1);
      expect(c.view(0).tiles[idxOf(s, at(scout.id))].owner).not.toBe(1);
      expect(c.view(0).tiles[idxOf(s, at(settler.id))].owner).not.toBe(1);
    }
    for (const client of [fixtureClient, core]) {
      const message = client.validate(0, orders.slice(0, 1)).map((e) => e.message).join();
      expect(message).toContain('current borders');
    }
  });
  it('constrained routes keep their destination on the first block by an ally on a later step', () => {
    const s = fixture([0, 0, 1]);
    for (const p of s.players) p.explored.fill(1);
    const goal = fromOffset(7, 4), blocked = fromOffset(5, 4);
    const u = mover(s, 0, 'Settler', fromOffset(3, 4), goal);
    const ally = mover(s, 1, 'Settler', fromOffset(5, 5), blocked);
    // A second ally parked beyond it: crossing both costs more than a Settler's moves, so the route cannot hop.
    spawnUnit(s, 1, 'Settler', fromOffset(6, 4), false);
    const { fixtureClient, core, reports } = play(s, [{
      0: [{ type: 'MoveUnitPath', unit: u.id, to: goal, path: [4, 5, 6, 7].map((col) => fromOffset(col, 4)), stop_on_block: true }],
      1: [{ type: 'MoveUnit', unit: ally.id, to: blocked }],
    }]);
    for (const client of [fixtureClient, core]) {
      const stopped = client.view(0).units.find((v) => v.id === u.id)!;
      expect(stopped.pos).toEqual(fromOffset(4, 4));
      expect(stopped.goto).toEqual(goal);
      expect(stopped.needs_orders).toBe(true);
    }
    expect(reports.flat().filter((e) => e.type === 'UnitMoved' && e.unit === u.id)).toHaveLength(1);
  });
  it('a confirmed route across one ally that stays waits for the next turn, then hops over it', () => {
    const s = fixture([0, 0, 1]);
    for (const p of s.players) p.explored.fill(1);
    const goal = fromOffset(7, 4), blocked = fromOffset(5, 4);
    const u = mover(s, 0, 'Settler', fromOffset(3, 4), goal);
    const ally = mover(s, 1, 'Settler', fromOffset(5, 5), blocked);
    const { core, reports } = play(s, [{
      0: [{ type: 'MoveUnitPath', unit: u.id, to: goal, path: [4, 5, 6, 7].map((col) => fromOffset(col, 4)), stop_on_block: true }],
      1: [{ type: 'MoveUnit', unit: ally.id, to: blocked }],
    }, {}]);
    // Turn 1: one step, then the ally stands on the next tile and the hop (two moves) does not fit the one left.
    expect(reports[0].filter((e) => e.type === 'UnitMoved' && e.unit === u.id)).toHaveLength(1);
    expect(reports[0].some((e) => e.type === 'MoveBlocked' && e.unit === u.id)).toBe(false);
    // Turn 2: the hop over the ally, landing beyond it.
    expect(reports[1]).toContainEqual(expect.objectContaining({ type: 'UnitMoved', unit: u.id, from: fromOffset(4, 4), to: fromOffset(6, 4) }));
    expect(core.view(0).units.find((v) => v.id === u.id)!.pos).toEqual(fromOffset(6, 4));
  });
  it.each([1, 2, 3])('constrained routes blocked by an allied settler request orders and recover (turn %s)', (turns) => {
    const s = fixture([0, 0, 1]);
    const start = neighbors(middle)[0];
    const u = spawnUnit(s, 0, 'Settler', start, false);
    const ally = spawnUnit(s, 1, 'Settler', middle, false);
    const order: Order = { type: 'MoveUnitPath', unit: u.id, to: middle, path: [middle], stop_on_block: true };
    const { fixtureClient, core, reports } = play(s, [{ 0: [order] }, ...Array.from({ length: turns - 1 }, () => ({}))]);
    expect(reports.flat().some((e) => e.type === 'OrderFailed')).toBe(false);
    expect(reports[0]).toContainEqual({ type: 'MoveBlocked', unit: u.id, owner: 0, at: start, toward: middle, reason: expect.stringMatching(/^blocked by .+'s Settler, which did not move$/) });
    for (const client of [fixtureClient, core]) {
      const stopped = client.view(0).units.find((v) => v.id === u.id)!;
      expect(stopped.pos).toEqual(start);
      expect(stopped.needs_orders).toBe(true);
      expect(stopped.goto).toEqual(turns === 1 ? middle : null);
      expect(stopped.path ?? []).toEqual(turns === 1 ? [middle] : []);
      expect(client.view(1).units.find((v) => v.id === ally.id)!.pos).toEqual(middle);
      const saved = Object.values(JSON.parse(client.save()).units as Record<string, { id: number; path_stop_on_block?: boolean; blocked_turns?: number }>).find((v) => v.id === u.id)!;
      expect(!!saved.path_stop_on_block).toBe(turns === 1);
      expect(saved.blocked_turns ?? 0).toBe(turns === 1 ? 1 : 0);
    }
    if (turns >= 2) expect(reports[1]).toContainEqual(expect.objectContaining({ type: 'MoveBlocked', unit: u.id, reason: `move stopped after 2 blocked turns: blocked by ${s.players[1].name}'s Settler at 5,5` }));
    if (turns === 3) expect(reports[2].some((e) => e.type === 'MoveBlocked' && e.unit === u.id)).toBe(false);
  });
  it.each(['border', 'civilian', 'soldier'] as const)('confirmed routes respect Peace %s through save/load', (blocker) => {
    const s = fixture(); for (const r of s.diplomacy!.relations) r.state = 'Peace';
    const start = fromOffset(3, 4), goal = fromOffset(7, 4), blocked = fromOffset(4, 4);
    const u = mover(s, 0, 'Soldier', start, goal); u.path = [4, 5, 6, 7].map((col) => fromOffset(col, 4));
    if (blocker === 'border') s.owner[idxOf(s, blocked)] = 1;
    else spawnUnit(s, 1, blocker === 'civilian' ? 'Settler' : 'Soldier', blocked, false);
    const { core, reports } = play(s, [{}, {}, {}, {}]);
    expect(core.view(0).units.find((v) => v.id === u.id)!.pos).toEqual(goal);
    expect(reports.flat().some((e) => e.type === 'Battle' || e.type === 'SettlerCaptured')).toBe(false);
    expect(reports.flat().flatMap((e) => e.type === 'UnitMoved' && e.unit === u.id ? [e.to] : [])).not.toContainEqual(blocked);
  });
  it('a move blocked two turns by a peaceful unit stops, names the blocker and asks for orders', () => {
    const s = fixture(); for (const r of s.diplomacy!.relations) r.state = 'Peace';
    const scout = spawnUnit(s, 1, 'Scout', middle, false);
    const u = mover(s, 0, 'Trooper', neighbors(middle)[0], middle);
    const { fixtureClient, core, reports } = play(s, [{}, {}, {}]);
    const stop = (es: Event[]) => es.filter((e) => e.type === 'MoveBlocked' && e.unit === u.id && e.reason.startsWith('move stopped'));
    expect(stop(reports[0])).toEqual([]);
    const [col, row] = [5, 5];
    expect(stop(reports[1])).toEqual([expect.objectContaining({ reason: `move stopped after 2 blocked turns: blocked by ${s.players[1].name}'s Scout at ${col},${row}` })]);
    expect(scout.pos).toEqual(middle);
    for (const c of [fixtureClient, core]) {
      const v = c.view(0).units.find((x) => x.id === u.id)!;
      expect(v.goto).toBeNull();
      expect(v.needs_orders).toBe(true);
    }
    expect(stop(reports[2])).toEqual([]);
  });
  it('rejects a newly confirmed route across known closed territory', () => {
    const s = fixture(); for (const r of s.diplomacy!.relations) r.state = 'Peace';
    const u = mover(s, 0, 'Soldier', neighbors(middle)[0], neighbors(middle)[3]); u.goto = null;
    s.owner[idxOf(s, middle)] = 1;
    const { reports } = play(s, [{ 0: [{ type: 'MoveUnitPath', unit: u.id, to: neighbors(middle)[3], path: [middle, neighbors(middle)[3]] }] }]);
    expect(reports[0]).toContainEqual(expect.objectContaining({ type: 'OrderFailed', player: 0 }));
  });
  it('neutral arrivals never stack or fight: the stronger side moves in (rules 51)', () => {
    const s = fixture(); for (const r of s.diplomacy!.relations) r.state = 'Peace';
    const a = mover(s, 0, 'Soldier', neighbors(middle)[0], middle);
    const b = mover(s, 1, 'Settler', neighbors(middle)[3], middle);
    const { core, reports } = play(s);
    expect(reports[0].some((e) => e.type === 'Bounce' || e.type === 'Battle' || e.type === 'SettlerCaptured')).toBe(false);
    expect(core.view(0).units.find((u) => u.id === a.id)!.pos).toEqual(middle);
    expect(core.view(1).units.find((u) => u.id === b.id)!.pos).toEqual(b.pos);
  });
  it.each(['enemy', 'city', 'civilian', 'friend', 'settler'] as const)('repaths confirmed routes around %s through save/load', (blocker) => {
    const s = fixture(); const u = mover(s, 0, blocker === 'settler' ? 'Settler' : 'Soldier', fromOffset(3, 4), fromOffset(7, 4));
    u.path = [4, 5, 6, 7].map((col) => fromOffset(col, 4));
    if (blocker === 'city') addCity(s, 1, 200, u.path[0]);
    else spawnUnit(s, blocker === 'friend' ? 0 : 1, blocker === 'civilian' ? 'Settler' : 'Soldier', u.path[0], false);
    const { core, reports } = play(s, [{}, {}, {}, {}]);
    expect(core.view(0).units.find((v) => v.id === u.id)!.pos).toEqual(u.goto);
    // Passing an enemy city at War draws its fire (cities fire back); the route never attacks.
    expect(reports.flat().some((e) => (e.type === 'Battle' && typeof e.attacker === 'number') || e.type === 'SettlerCaptured')).toBe(false);
  });
  it.each(['city', 'unit', 'civilian'] as const)('rejects known intermediate hostile %s', (blocker) => {
    const s = fixture(); const u = mover(s, 0, 'Soldier', neighbors(middle)[0], neighbors(middle)[3]);
    u.goto = null;
    if (blocker === 'city') addCity(s, 1, 200); else spawnUnit(s, 1, blocker === 'unit' ? 'Soldier' : 'Settler', middle, false);
    const { reports } = play(s, [{ 0: [{ type: 'MoveUnitPath', unit: u.id, to: neighbors(middle)[3], path: [middle, neighbors(middle)[3]] }] }]);
    expect(reports[0].some((e) => e.type === 'OrderFailed')).toBe(true);
  });
  it.each([false, true])('sequences own traffic, including a single-hex choke (%s)', (choke) => {
    const s = fixture(); const starts = [neighbors(middle)[0], neighbors(middle)[5]], goals = [neighbors(middle)[2], neighbors(middle)[3]];
    const a = mover(s, 0, 'Ranger', starts[0], goals[0]), b = mover(s, 0, 'Ranger', starts[1], goals[1]);
    if (choke) { s.terrain.fill('Mountains'); for (const h of [...starts, ...goals, middle, ...s.units.filter((u) => u.kind === 'Settler').map((u) => u.pos)]) s.terrain[idxOf(s, h)] = 'Grassland'; }
    const { core } = play(s);
    expect([a, b].map((u) => core.view(0).units.find((v) => v.id === u.id)!.pos)).toEqual(goals);
  });
  it('preserves simultaneous friendly cycles', () => {
    const s = fixture(); const tiles = [middle, neighbors(middle)[0], neighbors(middle)[1]];
    const us = tiles.map((h, i) => mover(s, 0, 'Soldier', h, tiles[(i + 1) % 3]));
    const { core } = play(s);
    expect(us.map((u) => core.view(0).units.find((v) => v.id === u.id)!.pos)).toEqual(us.map((u) => u.goto));
  });
  for (const kind of ['Soldier', 'Ranger', 'Marksman', 'CompositeArcher'] as const) {
    it.each(['none', 'incoming', 'garrison'] as const)(`${kind} city entry against %s`, (defender) => {
      const s = fixture(); addCity(s, 1);
      const a = mover(s, 0, kind, neighbors(middle)[0], middle);
      if (defender !== 'none') {
        const b = spawnUnit(s, 1, 'Soldier', defender === 'incoming' ? neighbors(middle)[3] : middle, false); b.hp = 1;
        if (defender === 'incoming') b.goto = middle;
      }
      const { core, reports } = play(s);
      expect(core.view(0).cities.find((c) => c.id === 1)!.owner).toBe(kind === 'Soldier' ? 0 : 1);
      if (kind !== 'Soldier') expect(reports[0]).toContainEqual(expect.objectContaining({ type: 'MoveBlocked', unit: a.id, reason: "Ranged units can't capture cities; move a melee unit in" }));
    });
  }
  it.each([1, 7, 42, '18446744073709551615'])('teammate HP and seeded capture arbitration, seed %s', (seed) => {
    for (const hp of [[100, 100], [100, 60], [60, 100]]) {
      const s = fixture([0, 0, 1], String(seed)); addCity(s, 2);
      const a = mover(s, 0, 'Soldier', neighbors(middle)[0], middle, hp[0]), b = mover(s, 1, 'Soldier', neighbors(middle)[3], middle, hp[1]);
      const { core, reports } = play(s);
      const owner = core.view(0).cities.find((c) => c.id === 1)!.owner;
      expect(owner).not.toBe(2);
      if (hp[0] !== hp[1]) (expect(owner).toMatchSnapshot(), void (hp[0] > hp[1] ? a.owner : b.owner));
      expect(reports[0].some((e) => e.type === 'Battle' || e.type === 'Bounce')).toBe(false);
    }
  });
  it('three-way meetings apply every fight from the same snapshot', () => {
    const s = fixture();
    for (let owner = 0; owner < 3; owner++) mover(s, owner, 'Soldier', neighbors(middle)[owner * 2], middle);
    const { reports } = play(s);
    expect(reports[0].filter((e) => e.type === 'Battle')).toHaveLength(3);
  });
  it('updates a captured city out of sight without exposing private details', () => {
    const s = fixture(); addCity(s, 1);
    mover(s, 0, 'Soldier', neighbors(middle)[0], middle);
    const { fixtureClient, core } = play(s);
    for (const client of [fixtureClient, core]) {
      expect(client.view(0).cities.find((c) => c.id === 1)).toMatchObject({ owner: 0, visible: true });
      expect(client.view(1).cities.find((c) => c.id === 1)).toMatchObject({ owner: 0, visible: false, detail: null });
    }
  });
  it('p84: five attackers take the city and the four that stayed outside are ready for orders', () => {
    const s = fixture(); addCity(s, 1);
    spawnUnit(s, 1, 'Soldier', middle, false).hp = 1;
    const attackers = neighbors(middle).slice(0, 5).map((h) => mover(s, 0, 'Soldier', h, middle));
    const { fixtureClient, core } = play(s);
    for (const client of [fixtureClient, core]) {
      expect(client.view(0).cities.find((c) => c.id === 1)!.owner).toBe(0);
      const units = client.view(0).units.filter((u) => attackers.some((a) => a.id === u.id));
      expect(units.filter((u) => u.pos.q === middle.q && u.pos.r === middle.r)).toHaveLength(1);
      for (const u of units) {
        expect(u.goto).toBeNull(); expect(u.path ?? []).toEqual([]); expect(!!u.needs_orders).toBe(u.pos.q !== middle.q || u.pos.r !== middle.r);
      }
      // The four outside need orders; the one inside holds the captured city (occupation lock) and never asks.
      const needing = unitsNeedingOrders(client.view(0), []).map((u) => u.id);
      const inside = units.find((u) => u.pos.q === middle.q && u.pos.r === middle.r)!;
      for (const a of attackers) expect(needing.includes(a.id)).toBe(a.id !== inside.id);
    }
  });
  it.each(['teammate', 'ally', 'rival'] as const)('p84: a %s capturing the city ends a distant confirmed route into it', (who) => {
    const s = fixture(who === 'teammate' ? [0, 0, 1] : [0, 1, 2]);
    if (who === 'ally') s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!.state = 'Alliance';
    const victim = 2;
    addCity(s, victim); spawnUnit(s, victim, 'Soldier', middle, false).hp = 1;
    mover(s, 1, 'Soldier', neighbors(middle)[0], middle);
    const far = mover(s, 0, 'Soldier', fromOffset(1, 5), middle);
    far.path = [fromOffset(2, 5), fromOffset(3, 5), middle]; far.path_stop_on_block = true;
    // A rival (not allied, not a teammate) keeps its own war order at the city.
    const { fixtureClient, core } = play(s);
    for (const client of [fixtureClient, core]) {
      expect(client.view(1).cities.find((c) => c.id === 1)!.owner).toBe(1);
      const u = client.view(0).units.find((v) => v.id === far.id)!;
      if (who === 'rival') expect(u.goto).not.toBeNull();
      else { expect(u.goto).toBeNull(); expect(u.path ?? []).toEqual([]); expect(u.needs_orders).toBe(true); }
    }
  });
  it('p84: an attack order on a unit that died before movement ends with needs_orders', () => {
    const s = fixture(); const [a, b] = [neighbors(middle)[0], neighbors(neighbors(middle)[3])[3]];
    const scout = spawnUnit(s, 1, 'Scout', b, false);
    const u = mover(s, 0, 'Soldier', a, b); u.attack_target = scout.id;
    s.units.splice(s.units.indexOf(scout), 1);
    const { fixtureClient, core } = play(s);
    for (const client of [fixtureClient, core]) {
      const v = client.view(0).units.find((x) => x.id === u.id)!;
      expect(v.goto).toBeNull(); expect(v.attack_target ?? null).toBeNull(); expect(v.needs_orders).toBe(true);
    }
  });
  it('swaps are one meeting and the survivor advances', () => {
    const s = fixture(); const a = mover(s, 0, 'Trooper', middle, neighbors(middle)[0]); mover(s, 1, 'Soldier', a.goto!, middle, 1);
    const { reports, core } = play(s);
    expect(reports[0].filter((e) => e.type === 'Battle')).toHaveLength(1);
    expect(core.view(0).units.find((u) => u.id === a.id)!.pos).toEqual(a.goto);
  });
  it('a blocked owner does not retry on an unrelated player advance', () => {
    const s = fixture(); spawnUnit(s, 0, 'Soldier', middle, false);
    mover(s, 0, 'Soldier', neighbors(middle)[0], middle); mover(s, 0, 'Soldier', neighbors(middle)[3], middle);
    mover(s, 1, 'Soldier', fromOffset(14, 4), fromOffset(16, 4)); play(s);
  });
  // Playtest-6.2 "The Last Bridge": city c, a friend m beside it, a free tile f beside it, and
  // a soldier r touching both. Every rotation, so the hex-order tie-break meets the friend.
  const lastBridge = (d: number) => {
    const c = middle, m = neighbors(c)[d], f = neighbors(c)[(d + 1) % 6];
    return { c, m, f, r: { q: m.q + f.q - c.q, r: m.r + f.r - c.r } };
  };
  it.each([0, 1, 2, 3, 4, 5])('the previewed attack routes around a friend attacking in place (rotation %s)', (d) => {
    const { c, m, f, r } = lastBridge(d);
    const s = fixture(); addCity(s, 1, 200, c);
    for (const p of s.players) p.explored.fill(1);
    const friend = spawnUnit(s, 0, 'Soldier', m, false), right = spawnUnit(s, 0, 'Soldier', r, false);
    refreshVision(s);
    const friendOrder: Order = { type: 'MoveUnit', unit: friend.id, to: c };
    const preview = previewRoute(buildView(s, 0), r, c, [friendOrder], right.id);
    expect(preview.path).toEqual([f, c]);
    expect(preview.risk).toBeNull();
    const { core, reports } = play(s, [{ 0: [friendOrder, { type: 'MoveUnitPath', unit: right.id, to: c, path: preview.path! }] }]);
    const attackers = reports[0].flatMap((e) => e.type === 'Battle' ? [e.attacker] : []);
    expect(attackers).toEqual(expect.arrayContaining([friend.id, right.id]));
    expect(reports[0].some((e) => (e.type === 'MoveBlocked' && e.unit === right.id) || e.type === 'Bounce')).toBe(false);
    expect(core.view(0).units.find((u) => u.id === right.id)!.pos).toEqual(f);
  });
  it.each([0, 3])('a stale confirmed route through a friend attacking in place is rerouted, not bounced (rotation %s)', (d) => {
    const { c, m, f, r } = lastBridge(d);
    const s = fixture(); addCity(s, 1, 200, c);
    for (const p of s.players) p.explored.fill(1);
    const friend = spawnUnit(s, 0, 'Soldier', m, false), right = spawnUnit(s, 0, 'Soldier', r, false);
    const { core, reports } = play(s, [{ 0: [{ type: 'MoveUnit', unit: friend.id, to: c }, { type: 'MoveUnitPath', unit: right.id, to: c, path: [m, c] }] }]);
    expect(reports[0].some((e) => e.type === 'Battle' && e.attacker === right.id)).toBe(true);
    expect(core.view(0).units.find((u) => u.id === right.id)!.pos).toEqual(f);
  });
  it.each([0, 1, 2, 3, 4, 5])('a moving friend is a tie-break, not a wall (rotation %s)', (d) => {
    const { c, m, f, r } = lastBridge(d);
    const s = fixture(); addCity(s, 1, 200, c);
    for (const p of s.players) p.explored.fill(1);
    const friend = mover(s, 0, 'Soldier', m, neighbors(m)[d]), right = mover(s, 0, 'Soldier', r, c);
    const { core } = play(s);
    expect(core.view(0).units.find((u) => u.id === right.id)!.pos).toEqual(f);
    expect(core.view(0).units.find((u) => u.id === friend.id)!.pos).toEqual(neighbors(m)[d]);
  });
  it('a row ordered onto one city takes distinct approach tiles and nobody bounces', () => {
    const s = fixture(); addCity(s, 1, 200, middle);
    const row = [-1, 0, 1].map((k) => ({ q: middle.q - 1 + k, r: middle.r + 2 }));
    const ids = row.map((h) => mover(s, 0, 'Soldier', h, middle).id);
    const { core, reports } = play(s, [{}, {}]);
    expect(reports.flat().some((e) => (e.type === 'MoveBlocked' && ids.includes(e.unit)) || e.type === 'Bounce')).toBe(false);
    expect(reports[0].filter((e) => e.type === 'Battle' && ids.includes(e.attacker as number))).toHaveLength(2);
    expect(new Set(ids.map((id) => JSON.stringify(core.view(0).units.find((u) => u.id === id)!.pos))).size).toBe(3);
  });
  /** A one-tile corridor along row 4 with a Settler of each player parked far off it. */
  const corridor = () => {
    const s = fixture();
    for (let col = 0; col < 20; col++) for (let row = 0; row < 12; row++) if (row !== 4) s.terrain[idxOf(s, fromOffset(col, row))] = 'Mountains';
    s.units = []; s.nextUnitId = 0;
    for (const p of s.players) spawnUnit(s, p.id, 'Settler', fromOffset(14 + p.id, 4), false);
    return s;
  };
  it('two parked friends in a row cannot be crossed on one move: the unit waits beside them and names the first', () => {
    const s = corridor();
    const a = mover(s, 0, 'Soldier', fromOffset(2, 4), fromOffset(8, 4));
    spawnUnit(s, 0, 'Soldier', fromOffset(3, 4), false);
    spawnUnit(s, 0, 'Soldier', fromOffset(4, 4), false);
    const { core, reports } = play(s);
    expect(reports[0]).toContainEqual(expect.objectContaining({ type: 'MoveBlocked', unit: a.id, reason: expect.stringMatching(/^blocked by .+'s Spearman, which did not move$/) }));
    expect(core.view(0).units.find((u) => u.id === a.id)!.goto).toEqual(fromOffset(8, 4));
  });
  it('a unit passes through a parked friend in one hop and never ends on it (Civ 6, user 2026-10-21)', () => {
    const s = corridor();
    const a = mover(s, 0, 'Soldier', fromOffset(2, 4), fromOffset(9, 4));
    spawnUnit(s, 0, 'Soldier', fromOffset(3, 4), false);
    const { core, reports } = play(s, [{}, {}]);
    expect(reports[0]).toContainEqual(expect.objectContaining({ type: 'UnitMoved', unit: a.id, from: fromOffset(2, 4), to: fromOffset(4, 4) }));
    expect(core.view(0).units.find((u) => u.id === a.id)!.pos).toEqual(fromOffset(6, 4));
  });
  it('a unit passes through its own garrisoned city', () => {
    const s = corridor(); addCity(s, 0, 200, fromOffset(4, 4));
    spawnUnit(s, 0, 'Soldier', fromOffset(4, 4), false);
    const a = mover(s, 0, 'Soldier', fromOffset(3, 4), fromOffset(9, 4));
    const { core } = play(s);
    expect(core.view(0).units.find((u) => u.id === a.id)!.pos).toEqual(fromOffset(5, 4));
  });
  it('a hop that does not fit the moves left waits on the last free tile it passed', () => {
    const s = corridor();
    const a = mover(s, 0, 'Soldier', fromOffset(0, 4), fromOffset(9, 4));
    spawnUnit(s, 0, 'Soldier', fromOffset(2, 4), false);
    const { core } = play(s, [{}, {}]);
    expect(core.view(0).units.find((u) => u.id === a.id)!.pos).toEqual(fromOffset(3, 4));
  });
  it('a move that would end on a parked friend stops beside it', () => {
    const s = corridor();
    const a = mover(s, 0, 'Soldier', fromOffset(2, 4), fromOffset(3, 4));
    spawnUnit(s, 0, 'Soldier', fromOffset(3, 4), false);
    const { core, reports } = play(s);
    expect(reports[0]).toContainEqual(expect.objectContaining({ type: 'MoveBlocked', unit: a.id }));
    expect(core.view(0).units.find((u) => u.id === a.id)!.pos).toEqual(fromOffset(2, 4));
  });
  it('the preview crosses a parked friend and the order resolves as previewed', () => {
    const s = corridor();
    const a = spawnUnit(s, 0, 'Soldier', fromOffset(2, 4), false);
    spawnUnit(s, 0, 'Soldier', fromOffset(3, 4), false);
    for (const p of s.players) p.explored.fill(1);
    refreshVision(s);
    const goal = fromOffset(8, 4), preview = previewRoute(buildView(s, 0), a.pos, goal, [], a.id);
    expect(preview.path?.[0]).toEqual(fromOffset(3, 4));
    expect(preview.risk).toBeNull();
    const { core } = play(s, [{ 0: [{ type: 'MoveUnitPath', unit: a.id, to: goal, path: preview.path! }] }]);
    expect(core.view(0).units.find((u) => u.id === a.id)!.pos).toEqual(fromOffset(4, 4));
  });
  it('forecasts zero damage from an ungarrisoned defeated city', () => {
    const s = fixture(); addCity(s, 1); const a = mover(s, 0, 'Soldier', neighbors(middle)[0], middle);
    refreshVision(s); const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s)); const core = makeCore(); core.load(coreFixture(s));
    (expect(fixtureClient.forecast(0, a.id, middle, 'Melee')).toMatchSnapshot(), void (core.forecast(0, a.id, middle, 'Melee')));
    expect(core.forecast(0, a.id, middle, 'Melee')!.expected_damage_to_attacker).toBe(0);
  });
});

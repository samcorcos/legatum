// Plain sight, fixtureClient vs real WASM (user decision 2026-10-23): every unit on a visible tile is seen
// by everyone, Celtic archers included. Unit movement, simultaneous meetings, ranged orders and city garrisons all treat them like any unit.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event, Faction, Hex, Order, PlayerView, UnitKind } from '../types';
import { fromOffset, neighbors } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState, MUnit } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
const at = (col: number, row: number) => fromOffset(col, row);

/** P0 Celts (team 0), P1 `rival` (team 1), optionally P2 (team 2); everyone at War unless changed. */
function fixture(rival: Faction = 'Switzerland', players = 2) {
  const setup = makeStandardSetup('7', 'Celts', rival, 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, players).map((p, i) => ({ ...p, faction: i === 0 ? 'Celts' : i === 1 ? rival : 'India', team: i, is_human: true }));
  const s = createState(setup);
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.explored.fill(0); spawnUnit(s, p.id, 'Settler', at(2 + 6 * p.id, 11), false); }
  return s;
}
function unit(s: MState, owner: number, kind: UnitKind, pos: Hex, goto: Hex | null = null): MUnit {
  const u = spawnUnit(s, owner, kind, pos, false); u.goto = goto; return u;
}
function city(s: MState, owner: number, pos: Hex): MCity {
  const c: MCity = { id: ++s.nextCityId, owner, founder: owner, name: `C${s.nextCityId}`, pos, population: 3, food: 0, prod: 0, producing: null,
    bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 };
  s.cities.push(c);
  for (const h of [pos, ...neighbors(pos)]) { const i = idxOf(s, h); if (i >= 0) { s.owner[i] = owner; s.tileCity[i] = c.id; } }
  return c;
}
const RELEVANT = ['OrderFailed', 'UnitMoved', 'Battle', 'UnitDestroyed', 'MoveBlocked', 'Bounce', 'SettlerCaptured'];
const relevant = (es: Event[]) => es.filter((e) => RELEVANT.includes(e.type));
/** What each player sees of every unit. */
function seen(v: PlayerView) {
  return v.units.map((u) => ({ id: u.id, owner: u.owner, pos: u.pos, hp: u.hp, goto: u.goto, needs: !!u.needs_orders }))
    .sort((a, b) => a.id - b.id);
}
function play(s: MState, turns: Record<number, Order[]>[] = [{}]) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  const ids = s.players.map((p) => p.id);
  for (const p of ids) (expect(seen(fixtureClient.view(p))).toMatchSnapshot(), void (seen(core.view(p))));
  const reports: Event[][] = [];
  for (const orders of turns) {
    for (const p of ids) (expect(fixtureClient.validate(p, orders[p] ?? [])).toMatchSnapshot(), void (core.validate(p, orders[p] ?? [])));
    const a = fixtureClient.endTurn(orders), b = core.endTurn(orders);
    (expect(relevant(a.events)).toMatchSnapshot(), void (relevant(b.events)));
    for (const p of ids) {
      (expect(relevant(a.events_by_player?.[p] ?? [])).toMatchSnapshot(), void (relevant(b.events_by_player?.[p] ?? [])));
      (expect(seen(fixtureClient.view(p))).toMatchSnapshot(), void (seen(core.view(p))));
    }
    reports.push(b.events);
    fixtureClient.load(fixtureClient.save()); core.load(core.save());
  }
  return { fixtureClient, core, reports };
}
const sees = (v: PlayerView, id: number) => v.units.some((u) => u.id === id);

describe.skipIf(!available)('Celtic archers in plain sight, core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('shows a Celtic archer on a visible tile to a rival, like any unit', () => {
    const s = fixture();
    const h = unit(s, 0, 'Archer', at(5, 5));
    unit(s, 1, 'Spearman', at(4, 5));
    const { fixtureClient, core } = play(s);
    for (const c of [fixtureClient, core]) expect(sees(c.view(1), h.id)).toBe(true);
  });

  it('fights a unit that walks into a Celtic archer: a battle, no halt', () => {
    const s = fixture();
    const h = unit(s, 0, 'Archer', at(5, 5));
    unit(s, 1, 'Spearman', at(4, 5), at(5, 5));
    const { reports } = play(s);
    expect(reports[0].some((e) => e.type === 'Battle' && e.defender.kind === 'Unit' && e.defender.unit === h.id)).toBe(true);
  });

  it('turns a swap or a shared destination into a meeting engagement', () => {
    const s = fixture();
    unit(s, 0, 'Archer', at(5, 5), at(4, 5));
    unit(s, 1, 'Spearman', at(4, 5), at(5, 5));
    const tile = neighbors(at(12, 5))[0], start = neighbors(tile)[0];
    unit(s, 0, 'Archer', at(12, 5), tile);
    unit(s, 1, 'Spearman', start, tile);
    const { reports } = play(s);
    expect(reports[0].filter((e) => e.type === 'Battle' && e.meeting)).toHaveLength(2);
  });

  it('accepts a ranged order on a visible Celtic archer exactly like one on any enemy unit', () => {
    const s = fixture();
    unit(s, 0, 'Archer', at(5, 5));
    const archer = unit(s, 1, 'Archer', at(7, 5));
    refreshVision(s);
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    const core = makeCore(); core.load(coreFixture(s));
    const shot = (target: Hex): Order[] => [{ type: 'RangedAttack', unit: archer.id, target }];
    for (const c of [fixtureClient, core]) {
      expect(c.validate(1, shot(at(5, 5)))).toEqual([]);
      expect(c.validate(1, shot(at(6, 4)))).toHaveLength(1);
    }
    (expect(fixtureClient.validate(1, shot(at(6, 4))).map((e) => e.index)).toMatchSnapshot(), void (core.validate(1, shot(at(6, 4))).map((e) => e.index)));
  });

  it('a melee attack on a city a Celtic archer garrisons hits the city while it has HP', () => {
    const s = fixture();
    city(s, 0, at(6, 5));
    const h = unit(s, 0, 'Archer', at(6, 5));
    const soldier = unit(s, 1, 'Spearman', at(5, 5));
    const { reports } = play(s, [{ 1: [{ type: 'MoveUnit', unit: soldier.id, to: at(6, 5) }] }]);
    // The city shields its garrison (Civ 6): the archer is not fought.
    expect(reports[0].some((e) => e.type === 'Battle' && e.defender.kind === 'City')).toBe(true);
    expect(reports[0].some((e) => e.type === 'Battle' && e.defender.kind === 'Unit' && e.defender.unit === h.id)).toBe(false);
  });

  it('a melee attack on a fallen city fights its Celtic archer garrison', () => {
    const s = fixture();
    city(s, 0, at(6, 5)).hp = 0;
    const h = unit(s, 0, 'Archer', at(6, 5));
    const soldier = unit(s, 1, 'Spearman', at(5, 5));
    const { reports } = play(s, [{ 1: [{ type: 'MoveUnit', unit: soldier.id, to: at(6, 5) }] }]);
    expect(reports[0].some((e) => e.type === 'Battle' && e.defender.kind === 'Unit' && e.defender.unit === h.id)).toBe(true);
  });
});

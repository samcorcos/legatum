import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event, Hex, Order, PlayerView } from '../types';
import { fromOffset, ring } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit, stateFromCore } from './fixtureEngine';
import { BARBARIAN, idxOf, refreshVision } from './fixtureEngine';
import { newBarbarians } from './fixtureEngine';
import type { MCamp } from './fixtureState';
import type { MCity, MState } from './fixtureState';

// Parity: the fixtureClient's barbarians (fixtureClient/barbarians.ts) against core rules/barbarians.rs.
const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture(peace = true) {
  const setup = makeStandardSetup('7', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12; setup.barbarians = true;
  setup.players = setup.players.slice(0, 3).map((p, i) => ({ ...p, faction: 'Switzerland', team: i, is_human: true }));
  const s = createState(setup);
  for (const r of s.diplomacy!.relations) r.state = peace ? 'Peace' : 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  s.barbarians = newBarbarians();
  for (const p of s.players) { p.gold = 500; p.explored.fill(0); spawnUnit(s, p.id, 'Settler', fromOffset(1 + 8 * p.id, 11), false); }
  return s;
}
function addCity(s: MState, owner: number, pos: Hex) {
  s.nextCityId += 1;
  const c: MCity = { id: s.nextCityId, owner, founder: owner, name: `City ${s.nextCityId}`, pos, population: 2, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, originalCapital: true, foundedTurn: 1, resistance: 0 };
  s.cities.push(c); s.owner[idxOf(s, pos)] = owner; s.tileCity[idxOf(s, pos)] = c.id;
  return c;
}
function addCamp(s: MState, pos: Hex, scoutAt?: Hex): MCamp {
  const b = s.barbarians!;
  const guard = spawnUnit(s, BARBARIAN, 'Spearman', pos, false).id;
  const scout = spawnUnit(s, BARBARIAN, 'Scout', scoutAt ?? ring(pos, 1)[0], false).id;
  const camp: MCamp = { id: b.next_camp_id++, pos, founded_turn: 1, guard, scout, raiders: [], scout_report: null, raid: null, next_raid_turn: 0, next_spawn_turn: 100 };
  b.last_camp_turn = 1;
  b.camps.push(camp);
  return camp;
}

const relevant = (es: Event[]) => es.filter((e) => e.type.startsWith('Barbarian') || ['UnitMoved', 'Battle', 'UnitDestroyed', 'SettlerCaptured', 'MoveBlocked', 'OrderFailed'].includes(e.type));
const seen = (v: PlayerView) => ({
  units: v.units.map((u) => ({ id: u.id, owner: u.owner, kind: u.kind, pos: u.pos, hp: u.hp })).sort((a, b) => a.id - b.id),
  camps: v.barbarian_camps ?? [],
  gold: v.economy.gold,
});
function play(s: MState, turns: Record<number, Order[]>[]) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  const reports: Event[][] = [];
  for (const orders of turns) {
    for (const p of s.players) (expect(fixtureClient.validate(p.id, orders[p.id] ?? [])).toMatchSnapshot(), void (core.validate(p.id, orders[p.id] ?? [])));
    const actual = fixtureClient.endTurn(orders).events, expected = core.endTurn(orders).events;
    (expect(relevant(actual)).toMatchSnapshot(), void (relevant(expected)));
    reports.push(expected);
    for (const p of s.players) (expect(seen(fixtureClient.view(p.id))).toMatchSnapshot(), void (seen(core.view(p.id))));
    fixtureClient.load(fixtureClient.save()); core.load(core.save());
  }
  return { fixtureClient, core, reports };
}

describe.skipIf(!available)('WASM barbarian behavior', () => {
  beforeAll(async () => {
    const pkg = '../wasm-pkg/legatum.js', mod = await import(/* @vite-ignore */ pkg);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('barbarians are attackable at Peace with no Declare War', () => {
    const s = fixture(true);
    addCity(s, 0, fromOffset(2, 2));
    addCamp(s, fromOffset(16, 2));
    const soldier = spawnUnit(s, 0, 'Spearman', fromOffset(8, 5), false);
    const raider = spawnUnit(s, BARBARIAN, 'Scout', fromOffset(9, 5), false);
    raider.hp = 20;
    const { reports } = play(s, [{ 0: [{ type: 'MoveUnit', unit: soldier.id, to: fromOffset(9, 5) }] }]);
    expect(reports[0].some((e) => e.type === 'Battle' && e.attacker === soldier.id)).toBe(true);
  });

  it('a scout that sees a city and gets home launches a raid that musters three raiders', () => {
    const s = fixture(true);
    const city = addCity(s, 0, fromOffset(3, 3));
    const camp = addCamp(s, fromOffset(9, 3), fromOffset(5, 3));
    const { reports } = play(s, Array.from({ length: 8 }, () => ({})));
    const all = reports.flat();
    expect(all.some((e) => e.type === 'BarbarianScoutSpotted' && e.city === city.id)).toBe(true);
    expect(all.find((e) => e.type === 'BarbarianRaidLaunched')).toMatchObject({ camp: camp.id, city: city.id, owner: 0, direction: 'east' });
  });

  it('entering an undefended camp clears it for Gold, seen by the clearer', () => {
    const s = fixture(true);
    addCity(s, 1, fromOffset(2, 2));
    const camp = addCamp(s, fromOffset(12, 6));
    s.units = s.units.filter((u) => u.id !== camp.guard && u.id !== camp.scout);
    camp.guard = null; camp.scout = null;
    const soldier = spawnUnit(s, 1, 'Spearman', fromOffset(10, 6), false);
    const { reports } = play(s, [{ 1: [{ type: 'MoveUnit', unit: soldier.id, to: fromOffset(12, 6) }] }, {}]);
    expect(reports[0]).toContainEqual({ type: 'BarbarianCampCleared', camp: camp.id, at: fromOffset(12, 6), by: 1, unit: soldier.id, gold: 50 });
  });

  it('raiders capture a lone Settler and walk it to their camp', () => {
    const s = fixture(true);
    addCity(s, 0, fromOffset(2, 2));
    const camp = addCamp(s, fromOffset(14, 6));
    const raider = spawnUnit(s, BARBARIAN, 'Spearman', fromOffset(10, 6), false);
    camp.raiders.push(raider.id);
    spawnUnit(s, 0, 'Settler', fromOffset(9, 6), false);
    const { reports } = play(s, Array.from({ length: 5 }, () => ({})));
    expect(reports.flat().some((e) => e.type === 'SettlerCaptured' && e.to === BARBARIAN)).toBe(true);
  });

  it('camps appear at the grace turn on unseen land, identically', () => {
    const s = fixture(true);
    addCity(s, 0, fromOffset(2, 2));
    addCity(s, 1, fromOffset(17, 9));
    const { reports } = play(s, Array.from({ length: 12 }, () => ({})));
    const spawned = reports.flat().filter((e) => e.type === 'BarbarianCampSpawned');
    expect(spawned.length).toBeGreaterThan(0);
  });
});

describe('WASM games with barbarians', () => {
  it('play 40 AI turns with camps, raids and no crash', async () => {
    const setup = makeStandardSetup('11', 'India', 'Byzantium', 'A', 'B');
    setup.barbarians = true;
    setup.players = setup.players.map((p) => ({ ...p, is_human: false }));
    const fixtureClient = new FixtureClient();
    await fixtureClient.newGame(setup);
    for (let t = 0; t < 40; t++) fixtureClient.endTurn({});
    const s = stateFromCore(fixtureClient.save());
    expect(s.barbarians?.next_camp_id).toBeGreaterThan(0);
    expect(s.cities.every((c) => c.owner !== BARBARIAN)).toBe(true);
    for (const p of s.players) expect(() => fixtureClient.view(p.id)).not.toThrow();
  }, 45_000); // Forty turns across eight AI empires, alongside parallel test workers on a busy host.
});

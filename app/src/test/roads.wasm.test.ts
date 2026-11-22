import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { fromOffset } from '../hex';
import type { Terrain, UnitKind } from '../types';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { ROAD_NODE } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit, stateFromCore } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

/** A 36x24 map with a ridge, a lake, hills and forest, and eight cities of two empires. */
function world(): MState {
  const setup = makeStandardSetup('4417', 'Mongols', 'India', 'A', 'B');
  setup.width = 36; setup.height = 24;
  setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team, is_human: true }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const r of setup.height === 24 ? [...Array(24).keys()] : []) {
    const set = (c: number, t: Terrain) => { s.terrain[idxOf(s, fromOffset(c, r))] = t; };
    if (r !== 11 && r !== 12) set(17, r % 3 === 0 ? 'Ocean' : 'Mountains');
    if (r > 3 && r < 9) { set(9, 'Hills'); set(10, 'Forest'); set(26, 'Hills'); }
  }
  for (const [c, r] of [[6, 14], [7, 15], [27, 6], [28, 7]] as const) s.terrain[idxOf(s, fromOffset(c, r))] = 'Hills';
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'Peace';
  for (const p of s.players) p.explored.fill(1);
  return s;
}

function city(s: MState, owner: number, col: number, row: number, extra: Partial<MCity> = {}) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${s.nextCityId}`, pos: fromOffset(col, row),
    population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: s.cities.every((o) => o.owner !== owner), foundedTurn: 1, resistance: 0, ...extra,
  };
  s.cities.push(c);
  s.owner[idxOf(s, c.pos)] = owner; s.tileCity[idxOf(s, c.pos)] = c.id;
  return c;
}

function hallWorld() {
  const s = world();
  const hall = city(s, 0, 5, 12, { buildings: ['YamRelayStation'] });
  s.players[0].hallCity = hall.id;
  for (const [o, c, r] of [[0, 12, 5], [0, 12, 19], [1, 30, 4], [1, 31, 12], [1, 29, 20], [0, 24, 10], [1, 4, 3]] as const) city(s, o, c, r);
  spawnUnit(s, 0, 'Settler', fromOffset(1, 22), false);
  spawnUnit(s, 1, 'Settler', fromOffset(34, 22), false);
  return { s, hall };
}

function pair(s: MState) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}

const roadsOf = (c: FixtureClient | WasmClient, p = 0) => c.view(p).tiles.map((t) => t.road ?? 0);
const laid = (es: { type: string }[]) => es.filter((e) => e.type === 'RoadsLaid');

describe.skipIf(!available)('roads: WASM behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('a new Hall lays the same roads, announced once, and a later city joins the same way', () => {
    const { s } = hallWorld();
    const { fixtureClient, core } = pair(s);
    const a = fixtureClient.endTurn({}), b = core.endTurn({});
    (expect(laid(a.events)).toMatchSnapshot(), void (laid(b.events)));
    expect(laid(b.events)).toHaveLength(1);
    const roads = roadsOf(core);
    expect(roads.filter((r) => r).length).toBeGreaterThan(40);
    (expect(roadsOf(fixtureClient)).toMatchSnapshot(), void (roads));
    (expect(roadsOf(fixtureClient, 1)).toMatchSnapshot(), void (roadsOf(core, 1)));
    for (const c of s.cities) expect(roads[idxOf(s, c.pos)] & ROAD_NODE).toBe(ROAD_NODE);
    // Every road link is mutual.
    fixtureClient.load(fixtureClient.save()); core.load(core.save());
    expect(laid(fixtureClient.endTurn({}).events)).toEqual([]);
    expect(laid(core.endTurn({}).events)).toEqual([]);
  });

  it('cities founded after the Hall are joined the next turn, identically', () => {
    const { s } = hallWorld();
    const { fixtureClient, core } = pair(s);
    fixtureClient.endTurn({}); core.endTurn({});
    fixtureClient.load(fixtureClient.save()); core.load(core.save());
    const ms = stateFromCore(fixtureClient.save());
    city(ms, 1, 21, 20); city(ms, 0, 33, 16);
    fixtureClient.load(JSON.stringify(ms));
    core.load(coreFixture(ms));
    fixtureClient.endTurn({}); core.endTurn({});
    (expect(roadsOf(fixtureClient)).toMatchSnapshot(), void (roadsOf(core)));
  });

  it('units use roads the same way: cost 1 on road tiles and +1 move from a road start', () => {
    const { s } = hallWorld();
    const { fixtureClient, core } = pair(s);
    fixtureClient.endTurn({}); core.endTurn({});
    fixtureClient.load(fixtureClient.save()); core.load(core.save());
    const ms = stateFromCore(fixtureClient.save());
    const a = spawnUnit(ms, 0, 'Spearman', ms.cities.find((c) => c.name === 'C1')!.pos, false);
    const b = spawnUnit(ms, 1, 'Scout' as UnitKind, ms.cities.find((c) => c.name === 'C5')!.pos, false);
    a.goto = ms.cities.find((c) => c.name === 'C4')!.pos;
    b.goto = ms.cities.find((c) => c.name === 'C2')!.pos;
    fixtureClient.load(JSON.stringify(ms));
    core.load(coreFixture(ms));
    for (let t = 0; t < 4; t++) {
      (expect(fixtureClient.endTurn({}).events.filter((e) => e.type === 'UnitMoved')).toMatchSnapshot(), void (core.endTurn({}).events.filter((e) => e.type === 'UnitMoved')));
      fixtureClient.load(fixtureClient.save()); core.load(core.save());
    }
    const pos = (c: FixtureClient | WasmClient) => c.view(0).units.map((u) => [u.id, u.pos]);
    (expect(pos(fixtureClient)).toMatchSnapshot(), void (pos(core)));
  });

  it('Roadworks link the cities within 8 hexes identically, and the build is offered and priced the same', () => {
    const s = world();
    const home = city(s, 1, 8, 6, { buildings: ['Roadworks'] });
    for (const [o, c, r] of [[0, 14, 6], [1, 8, 13], [0, 28, 20], [1, 3, 9]] as const) city(s, o, c, r);
    s.players[1].techs = ['Masonry', 'Engineering'];
    s.players[1].gold = 1000;
    spawnUnit(s, 0, 'Settler', fromOffset(1, 22), false);
    spawnUnit(s, 1, 'Settler', fromOffset(34, 22), false);
    const { fixtureClient, core } = pair(s);
    expect(fixtureClient.endTurn({}).events.filter((e) => e.type === 'RoadsLaid')).toEqual([]);
    core.endTurn({});
    (expect(roadsOf(fixtureClient)).toMatchSnapshot(), void (roadsOf(core)));
    const roads = roadsOf(core);
    expect(roads.filter((r) => r).length).toBeGreaterThan(8);
    expect(roads[idxOf(s, s.cities[0].pos)] & ROAD_NODE).toBe(ROAD_NODE);
    // The far city is out of range of every Roadworks.
    expect(roads[idxOf(s, s.cities[3].pos)]).toBe(0);
    (expect(fixtureClient.view(1).economy.gold).toMatchSnapshot(), void (core.view(1).economy.gold));
    expect(home.id).toBe(1);
  });

  it('Roadworks are a Field Logistics build with the same price and reasons in fixtureClient and core', () => {
    const s = world();
    const a = city(s, 0, 8, 6);
    city(s, 1, 28, 20);
    s.players[0].gold = 1000;
    spawnUnit(s, 0, 'Settler', fromOffset(1, 22), false);
    spawnUnit(s, 1, 'Settler', fromOffset(34, 22), false);
    for (const techs of [[], ['Masonry', 'Engineering']] as const) {
      s.players[0].techs = [...techs];
      const { fixtureClient, core } = pair(s);
      const orders = [{ type: 'SetProduction', city: a.id, item: { kind: 'Building', building: 'Roadworks' } }] as const;
      (expect(fixtureClient.validate(0, [...orders])).toMatchSnapshot(), void (core.validate(0, [...orders])));
      const list = (c: FixtureClient | WasmClient) => {
        const d = c.view(0).cities.find((x) => x.id === a.id)!.detail!;
        const pick = (o: { item: { kind: string; building?: string } }) => o.item.kind === 'Building' && o.item.building === 'Roadworks';
        return { options: d.options.filter(pick), unavailable: d.unavailable.filter(pick) };
      };
      (expect(list(fixtureClient)).toMatchSnapshot(), void (list(core)));
    }
  });
});

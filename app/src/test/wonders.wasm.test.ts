import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { distance, fromOffset, hexAt, tileIndex, within } from '../hex';
import { WONDER_INFO } from '../wonders';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';

import { cityYields, refreshVision } from './fixtureEngine';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
beforeAll(async () => {
  if (!available) return;
  const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeCore = () => new WasmClient(mod.Game);
});

it('places a stable, separated set on suitable terrain outside every start', () => {
  const setup = makeStandardSetup('909', 'India', 'Germany', 'A', 'B');
  const world = createState(setup);
  expect(createState(setup).wonder).toEqual(world.wonder);
  const placed = world.wonder!.flatMap((kind, i) => kind ? [{ kind, at: hexAt(i, setup.width), i }] : []);
  expect(placed).toHaveLength(6);
  for (const { kind, at, i } of placed) {
    expect(WONDER_INFO[kind].terrain).toContain(world.terrain[i]);
    expect(world.ruin[i]).toBe(0);
    expect(world.resource![i]).toBeNull();
    expect(world.players.every(({ start }) => distance(start, at) >= 4)).toBe(true);
    expect(placed.every((other) => other.i === i || distance(other.at, at) >= 5)).toBe(true);
  }
});

it.skipIf(!available)('matches WASM wonder visibility, city bonus and one-time discovery', () => {
  const setup = makeStandardSetup('909', 'India', 'Germany', 'A', 'B');
  setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.wonder!.fill(null); s.wondersDiscovered = [];
  s.owner.fill(-1); s.tileCity.fill(-1); s.ruin.fill(0);
  const center = fromOffset(7, 7), landmark = fromOffset(8, 7);
  const index = tileIndex(landmark, s.width, s.height);
  s.terrain[index] = 'Desert'; s.wonder![index] = 'GrandCanyon';
  s.cities.push({ id: 1, owner: 0, founder: 0, name: 'Home', pos: center, population: 3,
    food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: 200, grownTiles: [],
    tilesClaimed: 0, isCapital: true, originalCapital: true, foundedTurn: 1, resistance: 0 });
  s.nextCityId = 1;
  for (const h of within(center, 1)) {
    const i = tileIndex(h, s.width, s.height);
    s.owner[i] = 0; s.tileCity[i] = 1;
  }
  expect(s.tileCity[index]).toBe(1);
  spawnUnit(s, 0, 'Scout', fromOffset(7, 6), false);
  const rival = fromOffset(20, 8);
  s.terrain[tileIndex(rival, s.width, s.height)] = 'Grassland';
  spawnUnit(s, 1, 'Settler', rival, false);
  for (const p of s.players) p.explored.fill(0);
  s.knownOwner = {}; s.knownTerrain = {}; s.knownRuins = {};
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  expect(fixtureClient.debugState().wonder?.[index]).toBe('GrandCanyon');
  expect(cityYields(fixtureClient.debugState(), fixtureClient.debugState().cities[0]).sources.some((source) => source.label.includes('Grand Canyon'))).toBe(true);
  const core = makeCore(); core.load(coreFixture(s));
  const mockTile = fixtureClient.view(0).tiles[index], coreTile = core.view(0).tiles[index];
  expect(fixtureClient.debugState().wonder?.[index]).toBe('GrandCanyon');
  expect(fixtureClient.debugState().tileCity[index]).toBe(1);
  expect(mockTile.wonder).toBe('GrandCanyon');
  expect(mockTile.wonder).toBe(coreTile.wonder);
  (expect(fixtureClient.view(0).cities[0].detail?.yields.sources).toMatchSnapshot(), void (cityYields(fixtureClient.debugState(), fixtureClient.debugState().cities[0]).sources));
  (expect(fixtureClient.view(0).cities[0].detail?.yields.sources).toMatchSnapshot(), void (core.view(0).cities[0].detail?.yields.sources));
  (expect(fixtureClient.view(0).cities[0].detail?.yields.total).toMatchSnapshot(), void (core.view(0).cities[0].detail?.yields.total));
  const a = fixtureClient.endTurn({}), b = core.endTurn({});
  (expect(a.events.filter((e) => e.type === 'WonderDiscovered')).toMatchSnapshot(), void (b.events.filter((e) => e.type === 'WonderDiscovered')));
  expect(a.events.filter((e) => e.type === 'WonderDiscovered')).toHaveLength(1);
  expect(fixtureClient.endTurn({}).events.filter((e) => e.type === 'WonderDiscovered')).toEqual([]);
  expect(core.endTurn({}).events.filter((e) => e.type === 'WonderDiscovered')).toEqual([]);
});

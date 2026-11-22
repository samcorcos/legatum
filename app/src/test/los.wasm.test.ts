import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Hex, Order, Terrain } from '../types';
import { losBlocker } from '../los';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { MState } from './fixtureState';

const from: Hex = { q: 6, r: 5 };
const straight: Hex = { q: 8, r: 5 };
const far: Hex = { q: 9, r: 5 };
const middle: Hex = { q: 7, r: 5 };
const edge: Hex = { q: 7, r: 6 };
const edgeA: Hex = { q: 7, r: 5 };
const edgeB: Hex = { q: 6, r: 6 };
const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture(to: Hex, source: Terrain, barriers: [Hex, Terrain][], city = false): MState {
  const setup = makeStandardSetup('7301', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: true }));
  const s = createState(setup);
  for (const r of s.diplomacy!.relations) r.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.terrain[idxOf(s, from)] = source;
  for (const [h, terrain] of barriers) s.terrain[idxOf(s, h)] = terrain;
  for (const p of s.players) { p.explored.fill(0); spawnUnit(s, p.id, 'Settler', { q: 1 + 12 * p.id, r: 10 }, false); }
  if (city) {
    s.cities.push({ id: 0, owner: 0, founder: 0, name: 'High city', pos: from, population: 3, food: 0, prod: 0,
      producing: null, bought: null, buildings: ['Walls'], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 });
    s.nextCityId = 1; s.owner[idxOf(s, from)] = 0; s.tileCity[idxOf(s, from)] = 0;
  } else spawnUnit(s, 0, 'Ranger', from, false);
  spawnUnit(s, 1, 'Soldier', to, false);
  refreshVision(s);
  return s;
}

function pair(s: MState) {
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  const visible = (client: FixtureClient | WasmClient) => client.view(0).tiles.map((t) => t.visible);
  (expect(visible(fixtureClient)).toMatchSnapshot(), void (visible(core)));
  return { fixtureClient, core };
}

const cases: [string, Hex, Terrain, [Hex, Terrain][], boolean][] = [
  ['low Forest blocks', straight, 'Grassland', [[middle, 'Forest']], false],
  ['hill sees over Forest', straight, 'Hills', [[middle, 'Forest']], true],
  ['hill cannot see over Mountains', straight, 'Hills', [[middle, 'Mountains']], false],
  ['edge with one mountain stays open', edge, 'Grassland', [[edgeA, 'Mountains']], true],
  ['edge with both mountains closes', edge, 'Grassland', [[edgeA, 'Mountains'], [edgeB, 'Mountains']], false],
];

describe.skipIf(!available)('terrain LOS core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it.each(cases)('%s', (_label, to, source, barriers, clear) => {
    const s = fixture(to, source, barriers);
    const { fixtureClient, core } = pair(s);
    const ranger = s.units.find((u) => u.kind === 'Ranger')!;
    const order: Order = { type: 'RangedAttack', unit: ranger.id, target: to };
    const mo = fixtureClient.validate(0, [order]), co = core.validate(0, [order]);
    (expect(mo.map((e) => e.message.includes('line of sight'))).toMatchSnapshot(), void (co.map((e) => e.message.includes('line of sight'))));
    expect(mo.length === 0).toBe(clear);
    const a = fixtureClient.endTurn({ 0: [order] }).events.filter((e) => e.type === 'Battle');
    const b = core.endTurn({ 0: [order] }).events.filter((e) => e.type === 'Battle');
    (expect(a).toMatchSnapshot(), void (b));
    expect(a.length > 0).toBe(clear);
  });

  it('walled city fires at visible targets over Forest, Hills and Mountains', () => {
    for (const terrain of ['Forest', 'Hills', 'Mountains'] as const) {
      const s = fixture(straight, 'Grassland', [[middle, terrain]], true);
      spawnUnit(s, 0, 'Scout', { q: 8, r: 6 }, false); // City team's spotter sees through the blocker.
      refreshVision(s);
      const { fixtureClient, core } = pair(s);
      for (const client of [fixtureClient, core]) {
        expect(client.view(0).tiles[idxOf(s, straight)].visible).toBe(true);
        const shots = client.endTurn({}).events.filter((e) => e.type === 'Battle');
        expect(shots).toHaveLength(1);
      }
    }
  });

  it('a hill adds one sight to a unit and a city', () => {
    const low = pair(fixture(far, 'Grassland', []));
    const high = pair(fixture(far, 'Hills', []));
    const index = idxOf(fixture(far, 'Hills', []), far);
    expect(low.core.view(0).tiles[index].visible).toBe(false);
    expect(high.core.view(0).tiles[index].visible).toBe(true);
    const cityLow = pair(fixture({ q: 10, r: 5 }, 'Grassland', [], true));
    const cityHigh = pair(fixture({ q: 10, r: 5 }, 'Hills', [], true));
    const cityIndex = idxOf(fixture({ q: 10, r: 5 }, 'Hills', [], true), { q: 10, r: 5 });
    expect(cityLow.core.view(0).tiles[cityIndex].visible).toBe(false);
    expect(cityHigh.core.view(0).tiles[cityIndex].visible).toBe(true);
  });

  it('owned tiles and their one-tile ring stay visible through blocking terrain', () => {
    const s = fixture(far, 'Grassland', [[{ q: 15, r: 5 }, 'Hills'], [{ q: 16, r: 5 }, 'Forest']]);
    s.owner[idxOf(s, { q: 15, r: 5 })] = 0;
    spawnUnit(s, 1, 'Marksman', { q: 16, r: 5 }, false);
    refreshVision(s);
    const { fixtureClient, core } = pair(s);
    for (const client of [fixtureClient, core]) {
      expect(client.view(0).tiles[idxOf(s, { q: 15, r: 5 })].visible).toBe(true);
      expect(client.view(0).tiles[idxOf(s, { q: 16, r: 5 })].visible).toBe(true);
      expect(client.view(0).units.some((u) => u.owner === 1 && u.pos.q === 16 && u.pos.r === 5)).toBe(true);
    }
  });

  it.each([
    ['Mongols', 2],
    ['Switzerland', 2],
  ] as const)('%s Scout sight is %i in core', (faction, expected) => {
    const s = fixture(far, 'Grassland', []);
    s.players[0].faction = faction;
    s.units = s.units.filter((u) => !(u.owner === 0 && u.kind === 'Ranger'));
    const scout = spawnUnit(s, 0, 'Scout', from, false);
    refreshVision(s);
    const { fixtureClient, core } = pair(s);
    (expect(fixtureClient.view(0).units.find((u) => u.id === scout.id)?.sight).toMatchSnapshot(), void (expected));
    (expect(core.view(0).units.find((u) => u.id === scout.id)?.sight).toMatchSnapshot(), void (expected));
  });
});

it('edge interpolation treats one blocker as open and both as closed', () => {
  expect(losBlocker(from, edge, (h) => h.q === edgeA.q && h.r === edgeA.r ? 'Mountains' : 'Grassland')).toBeNull();
  expect(losBlocker(from, edge, (h) => h.q === edgeA.q && h.r === edgeA.r || h.q === edgeB.q && h.r === edgeB.r ? 'Mountains' : 'Grassland')).not.toBeNull();
});

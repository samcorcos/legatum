import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { fromOffset, tileIndex } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf } from './fixtureEngine';
import { createState, makeStandardSetup } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function world(): { state: MState; city: MCity; site: ReturnType<typeof fromOffset> } {
  const setup = makeStandardSetup('8508', 'India', 'Switzerland', 'A', 'B');
  setup.width = 24; setup.height = 16;
  setup.players = setup.players.slice(0, 3).map((p, team) => ({ ...p, team }));
  const state = createState(setup);
  state.units = []; state.cities = []; state.nextCityId = 1;
  state.terrain.fill('Grassland'); state.ruin.fill(0);
  state.owner.fill(-1); state.tileCity.fill(-1);
  for (const p of state.players) p.explored.fill(0);
  state.knownOwner = {}; state.knownCities = {}; state.knownRuins = {};
  const site = fromOffset(10, 8);
  const i = idxOf(state, site);
  const city: MCity = {
    id: 1, owner: 1, founder: 1, name: 'Waypost Seven', pos: site,
    population: 3, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: false, foundedTurn: 1, resistance: 0,
  };
  state.cities.push(city); state.owner[i] = 1; state.tileCity[i] = city.id;
  state.players[0].explored[i] = 1;
  state.knownOwner[0] = Array(state.terrain.length).fill(-1);
  state.knownOwner[0][i] = 1;
  state.knownCities[0] = { [city.id]: structuredClone(city) };
  return { state, city, site };
}

function pair(state: MState) {
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(state));
  const core = makeCore(); core.load(coreFixture(state));
  return { fixtureClient, core };
}

function publicView(client: FixtureClient | WasmClient, site: ReturnType<typeof fromOffset>) {
  const view = client.view(0);
  return {
    tile: view.tiles[tileIndex(site, view.width, view.height)],
    cities: view.cities.map((c) => ({ id: c.id, owner: c.owner, name: c.name, visible: c.visible,
      population: c.population, hp: c.hp, walls: c.walls })),
    units: view.units,
  };
}

describe.skipIf(!available)('fogged public borders and cities: core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('updates a captured city and border without revealing private facts or units', () => {
    const { state, city, site } = world();
    city.owner = 2; city.name = 'New Waypost'; city.population = 7; city.hp = 12;
    city.buildings.push('Walls'); state.owner[idxOf(state, site)] = 2;
    const { fixtureClient, core } = pair(state);
    (expect(publicView(fixtureClient, site)).toMatchSnapshot(), void (publicView(core, site)));
    expect(publicView(core, site)).toMatchObject({
      tile: { explored: true, visible: false, owner: 2 },
      cities: [{ id: 1, owner: 2, name: 'New Waypost', visible: false, population: 3, hp: 200, walls: false }],
      units: [],
    });
  });

  it('shows a new city on explored land, hides one on unexplored land, and removes razed cities', () => {
    const { state, city, site } = world();
    state.knownCities![0] = {};
    const next = fromOffset(19, 12);
    state.cities.push({ ...city, id: 2, name: 'Uncharted', pos: next });
    state.nextCityId = 2;
    state.tileCity[idxOf(state, next)] = 2;
    state.owner[idxOf(state, next)] = 1;
    let clients = pair(state);
    (expect(publicView(clients.fixtureClient, site)).toMatchSnapshot(), void (publicView(clients.core, site)));
    expect(publicView(clients.core, site).cities).toMatchObject([{ id: 1, population: 1, hp: 200 }]);
    state.cities = state.cities.filter((c) => c.id !== city.id);
    state.tileCity[idxOf(state, site)] = -1; state.owner[idxOf(state, site)] = -1;
    clients = pair(state);
    (expect(publicView(clients.fixtureClient, site)).toMatchSnapshot(), void (publicView(clients.core, site)));
    expect(publicView(clients.core, site)).toMatchObject({ tile: { owner: null }, cities: [] });
  });
});

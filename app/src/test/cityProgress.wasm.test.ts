import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { FixtureClient } from './fixtureEngine';
import { WasmClient } from '../wasmClient';
import { turnFlowFixture } from '../test/turnFlowFixture';
import { coreFixture } from '../test/coreFixture';
import type { GameClient } from '../client';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
beforeAll(async () => {
  if (!available) return;
  const pkg = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ pkg);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeCore = () => new WasmClient(mod.Game);
});

/** A population-3 city on desert, starving from `stored` Food. */
function starving(stored: number) {
  const fixture = turnFlowFixture();
  fixture.terrain.fill('Desert');
  fixture.resource?.fill(null);
  const city = fixture.cities.find((c) => c.owner === 0)!;
  Object.assign(city, { population: 3, food: stored });
  return { fixture, id: city.id };
}

const start = (engine: 'fixtureClient' | 'wasm', fixture: ReturnType<typeof turnFlowFixture>): GameClient => {
  const client = engine === 'fixtureClient' ? new FixtureClient() : makeCore();
  client.load(engine === 'fixtureClient' ? JSON.stringify(fixture) : coreFixture(fixture));
  return client;
};
const detail = (client: GameClient, id: number) => client.view(0).cities.find((c) => c.id === id)!.detail!;

it.skipIf(!available).each([0, 4, 9])('core forecast starvation and population unlocks alike (%s stored Food)', (stored) => {
  const { fixture, id } = starving(stored);
  const fixtureClient = start('fixtureClient', fixture), core = start('wasm', fixture);
  const forecast = detail(core, id).turns_to_starve;
  expect(forecast).toBeGreaterThan(0);
  expect(detail(fixtureClient, id).turns_to_starve).toBe(forecast);
  (expect(detail(fixtureClient, id).population_unlocks).toMatchSnapshot(), void (detail(core, id).population_unlocks));
  expect(detail(core, id).population_unlocks).toContainEqual({ population: 2, item: { kind: 'Unit', unit: 'Settler' } });
  for (const client of [fixtureClient, core]) {
    let turns = 0;
    while (client.view(0).cities.find((c) => c.id === id)!.population === 3) {
      const events = client.endTurn({}).events;
      turns++;
      expect(turns).toBeLessThanOrEqual(forecast!);
      if (turns === forecast) expect(events).toContainEqual(expect.objectContaining({ type: 'CityStarved', city: id, population: 2 }));
    }
    expect(turns).toBe(forecast);
  }
});

it.skipIf(!available)('a fed or population-1 city has no starvation forecast in either engine', () => {
  const { fixture, id } = starving(0);
  fixture.cities.find((c) => c.id === id)!.population = 1;
  for (const engine of ['fixtureClient', 'wasm'] as const) expect(detail(start(engine, fixture), id).turns_to_starve ?? null).toBeNull();
});

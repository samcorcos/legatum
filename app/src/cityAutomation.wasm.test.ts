import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from './wasmClient';
import { coreFixture } from './test/coreFixture';
import { wallsFixture } from './test/wallsFixture';
import type { Order } from './types';
import type { CityAutomationPlan } from './automation';

const available = existsSync(fileURLToPath(new URL('./wasm-pkg/legatum.js', import.meta.url)));
let wasm: { city_automation_orders(view: string, orders: string): string };
let client: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = './wasm-pkg/legatum.js';
  const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('./wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  wasm = mod;
  client = new WasmClient(mod.Game);
});

it.skipIf(!available)('WASM generates ordinary production orders from the owner view and leaves manual queues first', () => {
  const setup = wallsFixture();
  const city = setup.fixture.cities.find((c) => c.id === setup.own)!;
  city.producing = null;
  city.productionQueue = [];
  client.load(coreFixture(setup.fixture));
  const view = client.view(0);
  const setting: Order = { type: 'SetCityAutomation', city: city.id, focus: 'Balanced' };
  const plan = (orders: Order[]) => JSON.parse(wasm.city_automation_orders(JSON.stringify(view), JSON.stringify(orders))) as CityAutomationPlan;
  const chosen = plan([setting]);
  expect(chosen.orders.some((o) => o.type === 'SetProduction' && o.city === city.id)).toBe(true);
  expect(chosen.orders.every((o) => o.type !== 'BuyItem')).toBe(true);
  expect(chosen.unchosen_cities).toEqual([]);
  expect(chosen).toEqual(plan([setting]));
  expect(plan([setting, { type: 'SetProductionQueue', city: city.id, queue: [{ kind: 'Unit', unit: 'Scout' }] }])).toEqual({ orders: [], unchosen_cities: [] });
  const rival = client.view(1);
  expect(rival.cities.find((c) => c.id === city.id)?.detail).toBeFalsy();
  expect(JSON.parse(wasm.city_automation_orders(JSON.stringify(rival), '[]'))).toEqual({ orders: [], unchosen_cities: [] });
});

it.skipIf(!available)('WASM reports an empty or finishing delegated city when every option is excluded', () => {
  const setup = wallsFixture();
  client.load(coreFixture(setup.fixture));
  const view = client.view(0);
  const city = view.cities.find((c) => c.id === setup.own)!;
  city.detail!.build_automation = 'Science';
  city.detail!.producing = null;
  city.detail!.production_queue = [];
  city.detail!.options = city.detail!.options.filter((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman');
  expect(city.detail!.options).toHaveLength(1);
  const plan = (orders: Order[]) => JSON.parse(wasm.city_automation_orders(JSON.stringify(view), JSON.stringify(orders))) as CityAutomationPlan;
  expect(plan([])).toEqual({ orders: [], unchosen_cities: [city.id] });
  expect(plan([{ type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }])).toEqual({ orders: [], unchosen_cities: [] });
  city.detail!.producing = { kind: 'Unit', unit: 'Scout' };
  city.detail!.turns_to_complete = 1;
  expect(plan([]).unchosen_cities).toEqual([city.id]);
});

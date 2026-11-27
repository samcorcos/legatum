import { expect, it } from 'vitest';
import { addCityAutomatedOrders, AUTOMATION_NO_CHOICE, cityAutomationPreview, cityAutomationFocus, setCityAutomation } from './automation';
import { loadWasmModule } from './wasmClient';
import { FixtureClient } from './test/fixtureEngine';
import { coreFixture } from './test/coreFixture';
import { wallsFixture } from './test/wallsFixture';

it('automation is one economic toggle, including a saved legacy focus', () => {
  const view = { cities: [{ id: 1, detail: { build_automation: 'Military' } }] } as unknown as import('./types').PlayerView;
  expect(cityAutomationFocus(1, view, [])).toBe('Economy');
  expect(cityAutomationFocus(1, view, setCityAutomation([], 1, null))).toBeNull();
  expect(cityAutomationFocus(1, view, setCityAutomation([], 1, 'Economy'))).toBe('Economy');
});

it('uses the exact loaded chooser for preview and preserves a manual order when it cannot choose', async () => {
  expect(AUTOMATION_NO_CHOICE).toContain("Choose this turn's build");
  await loadWasmModule();
  const setup = wallsFixture();
  const client = new FixtureClient();
  client.load(coreFixture(setup.fixture));
  const view = client.view(0);
  const city = view.cities.find((c) => c.id === setup.own)!;
  city.detail!.build_automation = 'Economy';
  city.detail!.producing = null;
  city.detail!.production_queue = [];
  city.detail!.options = city.detail!.options.filter((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman');
  expect(city.detail!.options).toHaveLength(1);
  expect(cityAutomationPreview(view, [])?.unchosen_cities).toEqual([city.id]);
  expect(await addCityAutomatedOrders(view, [])).toEqual({ orders: [], unchosen_cities: [city.id] });
  const manual = [{ type: 'SetProduction' as const, city: city.id, item: { kind: 'Unit' as const, unit: 'Spearman' as const } }];
  expect(await addCityAutomatedOrders(view, manual)).toEqual({ orders: manual, unchosen_cities: [] });
});

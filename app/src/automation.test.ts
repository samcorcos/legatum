import { expect, it } from 'vitest';
import { addCityAutomatedOrders, AUTOMATION_NO_CHOICE, cityAutomationPreview, lastAutomationFocus, nextAutomationFocus, setCityAutomation } from './automation';
import { loadWasmModule } from './wasmClient';
import { FixtureClient } from './test/fixtureEngine';
import { coreFixture } from './test/coreFixture';
import { wallsFixture } from './test/wallsFixture';

it('I uses the last chosen focus and Shift+I steps through the focuses', () => {
  setCityAutomation([], 1, 'Military');
  expect(lastAutomationFocus()).toBe('Military');
  expect(nextAutomationFocus(null)).toBe('Military');
  expect(nextAutomationFocus('Balanced')).toBe('Military');
  expect(nextAutomationFocus('Military')).toBe('Science');
  expect(nextAutomationFocus('Gold')).toBe('Balanced');
});

it('uses the exact loaded chooser for preview and preserves a manual order when it cannot choose', async () => {
  expect(AUTOMATION_NO_CHOICE).toContain("Choose this turn's build");
  await loadWasmModule();
  const setup = wallsFixture();
  const client = new FixtureClient();
  client.load(coreFixture(setup.fixture));
  const view = client.view(0);
  const city = view.cities.find((c) => c.id === setup.own)!;
  city.detail!.build_automation = 'Science';
  city.detail!.producing = null;
  city.detail!.production_queue = [];
  city.detail!.options = city.detail!.options.filter((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman');
  expect(city.detail!.options).toHaveLength(1);
  expect(cityAutomationPreview(view, [])?.unchosen_cities).toEqual([city.id]);
  expect(await addCityAutomatedOrders(view, [])).toEqual({ orders: [], unchosen_cities: [city.id] });
  const manual = [{ type: 'SetProduction' as const, city: city.id, item: { kind: 'Unit' as const, unit: 'Spearman' as const } }];
  expect(await addCityAutomatedOrders(view, manual)).toEqual({ orders: manual, unchosen_cities: [] });
});

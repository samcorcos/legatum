import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { contextFromViews, formatEvents } from '../format';
import { citiesUnderAttack, hitsForTurn } from '../siege';
import { coreFixture } from './coreFixture';
import { siegeFixture } from './siegeFixture';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let core: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  core = new WasmClient(mod.Game);
});

it.skipIf(!available)('core says a city was hit last turn, and the alerts need no turn report to know it', () => {
  const s = siegeFixture();
  core.load(coreFixture(s.fixture));
  const view = core.view(0);
  const city = view.cities.find((c) => c.id === s.own)!;
  expect(city.detail?.attacked_last_turn).toBe(true);
  const [alarm] = citiesUnderAttack(view, null);
  expect(alarm).toMatchObject({ kind: 'attack', cityId: s.own, force: '3 Spearmen', attackedLastTurn: true, hit: null });
  expect(alarm.forecast.text).toBe('Holds about 2 turns unless reinforced');
});

it.skipIf(!available)('the AI really attacks an unwalled city, and the report leads with it', () => {
  const s = siegeFixture({ fresh: true });
  core.load(coreFixture(s.fixture));
  const before = core.view(0);
  const result = core.endTurn({});
  const after = core.view(0);
  const events = result.events_by_player?.[0] ?? result.events;
  const lines = formatEvents(events, contextFromViews(before, after));
  expect(lines[0], 'the attack leads the report').toMatchObject({ category: 'attack', tone: 'bad' });
  expect(lines[0].text).toMatch(/^Waypost Seven was attacked: 200 → \d+ HP by 3 Spearmen from /);
  // Only the city's own record is left once the turn report is gone (a loaded game).
  expect(hitsForTurn(events, before, after)[0]).toMatchObject({ before: 200 });
  expect(hitsForTurn(events, before, after)[0].lost).toBeGreaterThan(0);
});

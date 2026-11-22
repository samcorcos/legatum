import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { captureOrdersFixture } from '../src/test/captureOrdersFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };

async function hoverForecast(page: import('@playwright/test').Page, hex: Hex) {
  const card = page.locator('.forecast');
  await expect.poll(async () => {
    const point = await page.evaluate((target) => {
      const p = (window as unknown as GameWindow).legatumMap.toScreen(target);
      const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
      return { x: p.x + b.left, y: p.y + b.top };
    }, hex);
    await page.mouse.move(point.x, point.y);
    return card.isVisible();
  }).toBe(true);
  return card;
}

test('a Celtic capture reports the sack and Gold carried off', async ({ page }) => {
  const { state, attackers } = captureOrdersFixture();
  state.players[0].faction = 'Celts';
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 1,
      names: ['Player 1'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(state));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }, true), attackers[0]);
  const forecast = await hoverForecast(page, state.cities[0].pos);
  await expect(forecast.getByTestId('forecast-sack')).toHaveText('Sack: +180 Gold (60 per population)');
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect(report).toContainText('sacked');
  await expect(report).toContainText(/carried off \d+ Gold/);
  const event = await page.evaluate(() => {
    const s = (window as unknown as { legatum: { getState(): { report?: { events: { type: string; sack_gold?: number }[][] } } } }).legatum.getState();
    return s.report?.events.flat().find((e) => e.type === 'CityRazed' && e.sack_gold !== undefined)?.sack_gold ?? null;
  });
  // 3 population x 60: exactly what the forecast showed, whatever the victim's treasury holds.
  expect(event).toBe(180);
});

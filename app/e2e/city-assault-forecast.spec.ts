// The combined city-attack forecast on the WASM core (user, 2026-10-19, T187 Lugano): one card with the city's HP, the garrison under it,
// whether the city is captured this turn and your unit's HP, pooling the attacks you have queued on the same city.
// ASSAULT_SHOTS=<dir> picks where the screenshots go.
import type { Page } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { assaultFixture } from '../src/test/assaultFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const fx = assaultFixture();
const shots = process.env.ASSAULT_SHOTS ? path.resolve(process.env.ASSAULT_SHOTS) : path.resolve('../.progress/art/assault');

test.beforeEach(async ({ page }) => {
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2,
      names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(fx.fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await mkdir(shots, { recursive: true });
});

type Case = typeof fx.open;
async function cityPoint(page: Page, c: Case) {
  const at = () => page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const p = (window as unknown as GameWindow).legatumMap.toScreen(view.cities.find((c) => c.id === id)!.pos);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) };
  }, c.city);
  let last = await at();
  await expect.poll(async () => { const now = await at(); const still = now.x === last.x && now.y === last.y; last = now; return still; }).toBe(true);
  return last;
}

async function hoverAssault(page: Page, c: Case) {
  await expect.poll(async () => {
    const at = await cityPoint(page, c);
    await page.mouse.move(at.x + 3, at.y + 3);
    await page.mouse.move(at.x, at.y, { steps: 3 });
    return page.getByTestId('forecast-assault').isVisible();
  }, { intervals: [500, 1000, 1000, 2000] }).toBe(true);
}

for (const [kind, c] of [['unwalled', fx.open], ['walled', fx.walled]] as const) {
  for (const [width, height] of [[1440, 900], [1000, 600]] as const) {
    test(`one card shows the city, the garrison, the capture and your unit, pooling queued attacks: ${kind} city at ${width}x${height}`, async ({ page }) => {
      await page.setViewportSize({ width, height });
      await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }, true), c.first);
      await hoverAssault(page, c);
      const block = page.getByTestId('forecast-assault');
      await expect(page.getByTestId('forecast-assault-city')).toContainText(`${c.hp} →`);
      await expect(page.getByTestId('forecast-assault-garrison')).toContainText('garrison');
      await expect(page.getByTestId('forecast-attacker-hp')).toContainText('→');
      const alone = (await page.getByTestId('forecast-assault-garrison').innerText()).match(/→\s*(\d+)/)![1];
      await expect(block).toContainText('The whole fight at');
      await page.screenshot({ path: path.join(shots, `assault-${kind}-alone-${width}.png`) });
      // Queue the second Spearman on the same city: the card pools both, and more of the excess reaches the garrison.
      await page.evaluate(({ unit, to }) => {
        (window as unknown as GameWindow).legatum.getState().editOrders((o) => [...o, { type: 'MoveUnit', unit, to }]);
      }, { unit: c.second, to: c.pos });
      await expect(block).toContainText('2 attacks this turn');
      const pooled = (await page.getByTestId('forecast-assault-garrison').innerText()).match(/→\s*(\d+)/)![1];
      expect(Number(pooled)).toBeLessThan(Number(alone));
      await expect(page.getByTestId('forecast-assault-capture')).toBeVisible();
      // The whole card is inside the window, never cut off.
      await page.waitForTimeout(200);
      const card = page.locator('.forecast');
      const box = (await card.boundingBox())!;
      expect(box.y).toBeGreaterThanOrEqual(0);
      expect(box.y + box.height).toBeLessThanOrEqual(height);
      expect(await card.evaluate((el) => el.scrollHeight <= el.clientHeight + 1)).toBe(true);
      await page.screenshot({ path: path.join(shots, `assault-${kind}-pooled-${width}.png`) });
    });
  }
}

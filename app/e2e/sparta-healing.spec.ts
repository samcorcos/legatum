import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { promotionFixture } from '../src/test/promotionFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const setup = promotionFixture();
setup.fixture.players[0].faction = 'Sparta';
setup.fixture.units.find((unit) => unit.id === setup.pikeman)!.hp = 40;

async function select(page: Page, id: number) {
  await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    s.select({ kind: 'unit', id });
    s.jumpTo(view.units.find((u) => u.id === id)!.pos);
  }, id);
}

async function hexPoint(page: Page, id: number) {
  await expect.poll(() => page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const p = (window as unknown as GameWindow).legatumMap.toScreen(view.units.find((u) => u.id === id)!.pos);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    // Focus keeps the selected unit clear of panels, so it need not be at the canvas center.
    return Math.max(0, -p.x, p.x - b.width, -p.y, p.y - b.height);
  }, id)).toBe(0);
  return page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const p = (window as unknown as GameWindow).legatumMap.toScreen(view.units.find((u) => u.id === id)!.pos);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: p.x + b.left, y: p.y + b.top };
  }, id);
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2,
      names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(setup.fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
});

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`Sparta Fortify heals at the normal rate and shows kill healing at ${size.width}x${size.height}`, async ({ page }, testInfo) => {
    await page.setViewportSize(size);
    await select(page, setup.pikeman);
    const heal = page.locator('.unit-panel button.act', { hasText: /^Fortify/ });
    await expect(heal).toBeVisible();
    await expect(heal).toBeEnabled();
    await expect(page.locator('.unit-panel .act-reason', { hasText: 'No healing' })).toHaveCount(0);
    await expect(page.locator('.unit-panel .up-hints')).toContainText('heal up to 10 HP');
    await page.screenshot({ path: testInfo.outputPath(`sparta-heal-${size.width}x${size.height}.png`), animations: 'disabled' });

    await page.mouse.move(5, size.height / 2);
    const at = await hexPoint(page, setup.enemy);
    await page.mouse.move(at.x, at.y);
    const card = page.locator('.forecast');
    await expect(card).toBeVisible();
    await expect(card).toContainText('A kill heals your surviving attackers +100 HP, shared');
    await page.screenshot({ path: testInfo.outputPath(`sparta-forecast-${size.width}x${size.height}.png`), animations: 'disabled' });

    // Pressing F (the map hover is off the enemy now) queues Fortify, which heals at the normal rate.
    await page.mouse.move(5, size.height / 2);
    await select(page, setup.pikeman);
    await page.keyboard.press('f');
    await expect.poll(() => page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.orders[s.seat].filter((o) => o.type === 'SleepUnit').length;
    })).toBe(1);
  });
}

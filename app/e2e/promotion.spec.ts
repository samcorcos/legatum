import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { promotionFixture } from '../src/test/promotionFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const setup = promotionFixture();

async function select(page: Page, id: number) {
  await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    s.select({ kind: 'unit', id });
    s.jumpTo(view.units.find((u) => u.id === id)!.pos);
  }, id);
}

async function hexPoint(page: Page, id: number, centered: number) {
  // The camera glides to a selection; wait until the selected unit is centred.
  await expect.poll(() => page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const p = (window as unknown as GameWindow).legatumMap.toScreen(view.units.find((u) => u.id === id)!.pos);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return Math.hypot(p.x - b.width / 2, p.y - b.height / 2);
  }, centered)).toBeLessThan(1);
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

test('the unit panel shows level, XP and promoted strength; enemies show their level only', async ({ page }) => {
  await select(page, setup.pikeman);
  const rank = page.locator('.unit-panel .up-rank');
  await expect(rank).toContainText('Level 3 · +30%');
  await expect(rank).toContainText('102/150 XP');
  await expect(rank.locator('.rank-insignia g')).toHaveCount(3);
  // Pikeman base 32 (the Medieval step of the era curve), +30%: 32 * 130 / 100 = 41 (rounded down).
  await expect(page.locator('.unit-panel .up-stat', { hasText: 'Strength' }).locator('b')).toHaveText('41');

  await select(page, setup.archer);
  await expect(page.locator('.unit-panel .up-rank')).toContainText('49/90 XP');
  await expect(page.locator('.unit-panel .up-stat', { has: page.locator('.lbl', { hasText: /^Ranged$/ }) }).locator('b')).toContainText('18');

  await select(page, setup.enemy);
  await expect(page.locator('.unit-panel .up-rank')).toHaveText(/Level 2 · \+20%/);
  await expect(page.locator('.unit-panel .up-rank')).not.toContainText('XP');
});

test('the combat forecast shows each side\'s promotion as its own line', async ({ page }) => {
  await select(page, setup.pikeman);
  const at = await hexPoint(page, setup.enemy, setup.pikeman);
  await page.mouse.move(at.x, at.y);
  const card = page.locator('.forecast');
  await expect(card).toBeVisible();
  await expect(card.locator('.forecast-bd')).toContainText('Level 3: +30%');
  await expect(card.locator('.forecast-bd')).toContainText('Level 2: +20%');
  await expect(card.locator('.fside-name .rank-insignia')).toHaveCount(2);
});

test('the tile tooltip shows the same promoted unit badge', async ({ page }) => {
  await select(page, setup.pikeman);
  const at = await hexPoint(page, setup.pikeman, setup.pikeman);
  await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
  await page.mouse.move(at.x, at.y);
  const row = page.locator('.tilecard-unit').first();
  await expect(row).toBeVisible();
  await expect(row.locator('.rank-insignia g')).toHaveCount(3);
});

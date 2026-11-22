import { revealSidebar } from './sidebarAssertions';
// Playtest-6.2 "The Last Bridge": an attack preview routes to the free tile beside the city,
// never through a friend that stays to attack (an attack cannot start two tiles away), a route
// through a parked friend is clear (it is passed through, never ended on), and a route that may
// still halt on a friend that must move out says so. Real app on the WASM core, at the user's desktop sizes.
import type { Page } from '@playwright/test';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex, Order } from '../src/types';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { lastBridgeFixture } from '../src/test/lastBridgeFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const setup = lastBridgeFixture();
const shots = process.env.LEGATUM_SHOTS ?? path.resolve('../.progress/art/pathfinding');
const label = process.env.LEGATUM_SHOT_LABEL ?? 'after';

async function selectUnit(page: Page, id: number) {
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }, true), id);
}

/** Screen point of a hex once the camera has finished gliding to the selection. */
async function hexPoint(page: Page, hex: Hex) {
  const at = () => page.evaluate((hex) => {
    const p = (window as unknown as GameWindow).legatumMap.toScreen(hex);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) };
  }, hex);
  let last = await at();
  await expect.poll(async () => { const now = await at(); const still = now.x === last.x && now.y === last.y; last = now; return still; }).toBe(true);
  return last;
}

async function hoverUntil(page: Page, hex: Hex, shown: ReturnType<Page['locator']>) {
  await expect.poll(async () => {
    const at = await hexPoint(page, hex);
    await page.mouse.move(at.x + 3, at.y + 3);
    await page.mouse.move(at.x, at.y, { steps: 3 });
    return shown.isVisible();
  }, { intervals: [500, 1000, 1000, 2000] }).toBe(true);
}

const orders = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  return s.orders[s.seat] as Order[];
});

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
  test(`the rightmost Spearman attacks Last Bridge from the free tile at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    // The two Spearmen beside the city attack it from where they stand.
    await page.evaluate(({ ids, to }) => (window as unknown as GameWindow).legatum.getState().editOrders(
      (o) => [...o, ...ids.map((unit) => ({ type: 'MoveUnit' as const, unit, to }))]), { ids: setup.attackers, to: setup.cityPos });
    await selectUnit(page, setup.rightmost);
    await hoverUntil(page, setup.cityPos, page.locator('.forecast'));
    await page.waitForTimeout(400);
    await page.screenshot({ path: path.join(shots, `last-bridge-${label}-${size.width}.png`) });
    await expect(page.getByTestId('route-warning')).toHaveCount(0);
    const at = await hexPoint(page, setup.cityPos);
    await page.mouse.click(at.x, at.y, { button: 'right' });
    // The confirmed route, with the card out of the way: reselect it, pointer on the top bar.
    await page.mouse.move(size.width / 2, 20);
    await selectUnit(page, setup.rightmost);
    await hexPoint(page, setup.cityPos);
    await page.waitForTimeout(600);
    await page.screenshot({ path: path.join(shots, `last-bridge-${label}-confirmed-${size.width}.png`) });
    await expect.poll(async () => (await orders(page)).find((o) => 'unit' in o && o.unit === setup.rightmost)).toEqual(
      { type: 'MoveUnitPath', unit: setup.rightmost, to: setup.cityPos, path: [setup.free, setup.cityPos] });
    await revealSidebar(page, 'turn-more');
    await expect(page.getByRole('button', { name: /^Pending orders/ })).not.toContainText('issue');
  });
}

test('a route through a parked friend is clear, and one through a friend that must move out warns', async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 600 });
  await selectUnit(page, setup.behind);
  const warning = page.getByTestId('route-warning');
  // Civ 6: a unit walks through its own stationary friend and lands beyond it, so no warning.
  await hoverUntil(page, setup.passGoal, page.locator('.tilecard'));
  await expect(warning).toHaveCount(0);
  await page.waitForTimeout(400);
  await page.screenshot({ path: path.join(shots, `pass-through-${label}-1000.png`) });
  // Ordering the friend on down the pass turns it into a "may be blocked" warning.
  await page.evaluate(({ unit, to }) => (window as unknown as GameWindow).legatum.getState().editOrders(
    (o) => [...o, { type: 'MoveUnit' as const, unit, to }]), { unit: setup.parked, to: { q: setup.passGoal.q + 4, r: setup.passGoal.r } });
  await selectUnit(page, setup.behind);
  await hoverUntil(page, setup.passGoal, warning);
  await expect(warning).toHaveText('Path may be blocked by your Spearman: it must move out first.');
});

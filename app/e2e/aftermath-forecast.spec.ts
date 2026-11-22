// The attack forecast covers the aftermath (p881 horseman, 2026-10-11), in the real app on the
// WASM core, built from the user's own turn-29 situation: the Horseman's charge on the fortified
// Archer shows what waits on the Archer's tile, says it assumes fortification, and fits at the
// user's laptop size (1000x600) and at 1440x900.
import type { Page } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { classBonusStrength } from '../src/combat';
import { unitStatsFor } from '../src/data';
import { p881Scene, type P881Variant } from '../src/test/p881Scene';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const shots = path.resolve('../.progress/art/aftermath');

async function open(page: Page, variant: P881Variant) {
  const scene = p881Scene(variant);
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 29,
      names: ['Sam'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(scene.fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(/\d+/);
  await mkdir(shots, { recursive: true });
  return scene;
}

async function selectUnit(page: Page, id: number) {
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }, true), id);
}

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

/** Hover the tile until the forecast card shows (a hover can land while the camera still glides). */
async function hoverForecast(page: Page, hex: Hex) {
  const card = page.locator('.forecast');
  await expect.poll(async () => {
    const at = await hexPoint(page, hex);
    await page.mouse.move(at.x + 3, at.y + 3);
    await page.mouse.move(at.x, at.y, { steps: 3 });
    return card.isVisible();
  }, { intervals: [500, 1000, 1000, 2000] }).toBe(true);
  await page.waitForTimeout(250);
  return card;
}

for (const [width, height] of [[1440, 900], [1000, 600]] as const) {
  test(`turn 29: the card names the walls, says it assumes fortification, and fits at ${width}x${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    const scene = await open(page, 'fog');
    await selectUnit(page, scene.horseman);
    const card = await hoverForecast(page, scene.archerTile);
    const after = page.getByTestId('forecast-aftermath');
    await expect(after).toContainText("If it destroys the Archer");
    await expect(after).toContainText("Nara's walls (about −");
    await expect(after).toContainText('Units you cannot see in Nara are not counted.');
    await expect(after).toHaveAttribute('data-risk', 'Exposed');
    await expect(page.getByTestId('forecast-fortify')).toContainText('Assumes the Archer stays fortified (+6 Defense). Firing loses it: chance above.');
    // The forecast still says 0% to kill, as the user saw, and now says why and what changes.
    await expect(card).toContainText('0%');
    const box = (await card.boundingBox())!;
    expect(box.y).toBeGreaterThanOrEqual(0);
    expect(box.y + box.height).toBeLessThanOrEqual(height);
    expect(await card.evaluate((el) => el.scrollHeight <= el.clientHeight + 1)).toBe(true);
    // The aftermath and the assumption are not the quiet notes that short windows hide.
    await expect(after).toBeVisible();
    await expect(page.getByTestId('forecast-fortify')).toBeVisible();
    await page.mouse.move(2, height - 2);
    await hoverForecast(page, scene.archerTile);
    await page.screenshot({ path: path.join(shots, `turn29-${width}.png`) });
  });

  test(`with the garrison in sight the severe aftermath leads the card at ${width}x${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    const scene = await open(page, 'seen');
    await selectUnit(page, scene.horseman);
    const card = await hoverForecast(page, scene.archerTile);
    const after = page.getByTestId('forecast-aftermath');
    // Exposed since the stronger riders (2026-10-31; Likely lost before), and still severe.
    await expect(after).toHaveAttribute('data-risk', 'Exposed');
    await expect(after).toContainText('2 Spearmen');
    await expect(after).toContainText(`+${classBonusStrength(unitStatsFor('India', 'Soldier').strength)} vs mounted`);
    // Directly under the header, above the two sides.
    const head = (await card.locator('.forecast-head').boundingBox())!, line = (await after.boundingBox())!, sides = (await card.locator('.forecast-sides').boundingBox())!;
    expect(line.y).toBeGreaterThanOrEqual(head.y + head.height - 1);
    expect(line.y + line.height).toBeLessThanOrEqual(sides.y + 1);
    const box = (await card.boundingBox())!;
    expect(box.y + box.height).toBeLessThanOrEqual(height);
    expect(await card.evaluate((el) => el.scrollHeight <= el.clientHeight + 1)).toBe(true);
    await page.screenshot({ path: path.join(shots, `seen-${width}.png`) });
  });
}

// The turn-30 retreat: the user's first step went sideways along the enemy screen and a Spearman
// caught the unit leaving. A move ordered now takes a first step known enemies cannot strike.
for (const [width, height] of [[1440, 900], [1000, 600]] as const) {
  test(`turn 30: ordering the retreat takes a safe first step at ${width}x${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    const scene = await open(page, 'retreat');
    await selectUnit(page, scene.horseman);
    // Beside the user's own target (17,11), which the unit panel covers at 1000x600; the plain route starts the same way.
    const target = { q: 18, r: 10 };
    const at = await hexPoint(page, target);
    await page.mouse.move(at.x + 2, at.y + 2);
    await page.mouse.move(at.x, at.y, { steps: 3 });
    // A safe route exists with the same turns, so there is nothing to warn about.
    await expect(page.getByTestId('route-warning')).toHaveCount(0);
    await page.waitForTimeout(250);
    await page.screenshot({ path: path.join(shots, `retreat-preview-${width}.png`) });
    // The selection camera can finish gliding while the screenshot is captured.
    const clickAt = await hexPoint(page, target);
    await page.mouse.click(clickAt.x, clickAt.y, { button: 'right' });
    const order = await page.evaluate((unit) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.orders[s.seat].find((o) => 'unit' in o && o.unit === unit) ?? null;
    }, scene.horseman);
    expect(order).toMatchObject({ type: 'MoveUnitPath', to: target });
    const route = (order as unknown as { path: Hex[] }).path;
    // The user's route began (18,13): a Spearman at Nara reaches it in one step. The safe one does not.
    expect(route[0]).not.toEqual({ q: 18, r: 13 });
    expect(route.at(-1)).toEqual(target);
    expect(route.length).toBeLessThanOrEqual(4);
  });
}

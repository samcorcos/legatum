// Walls-only city strike, in the real app on the WASM core: walls drawn on the map, the city
// panel says whether a city can fire, and attack forecasts show city fire only for walled cities.
import type { Page } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { wallsFixture } from '../src/test/wallsFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const setup = wallsFixture();
const shots = path.resolve('../.progress/art/walls');

async function selectCity(page: Page, id: number) {
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), id);
}

async function selectUnit(page: Page, id: number) {
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }, true), id);
}

/** Screen point of a city once the camera has finished gliding to the selection. */
async function cityPoint(page: Page, id: number) {
  const at = () => page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const p = (window as unknown as GameWindow).legatumMap.toScreen(view.cities.find((c) => c.id === id)!.pos);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) };
  }, id);
  let last = await at();
  await expect.poll(async () => { const now = await at(); const still = now.x === last.x && now.y === last.y; last = now; return still; }).toBe(true);
  return last;
}

/** Hover a city until its attack forecast shows: a hover can land while the camera still glides
 * to the new selection, on the tile beside the city. */
async function hoverForecast(page: Page, id: number, shown: ReturnType<Page['locator']>) {
  await expect.poll(async () => {
    const at = await cityPoint(page, id);
    await page.mouse.move(at.x + 3, at.y + 3);
    await page.mouse.move(at.x, at.y, { steps: 3 });
    return shown.isVisible();
  }, { intervals: [500, 1000, 1000, 2000] }).toBe(true);
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
  await mkdir(shots, { recursive: true });
});

test('the city panel says which cities can fire, naming the Alpine Redoubt', async ({ page }) => {
  const fire = page.getByTestId('city-fire');
  await selectCity(page, setup.walled);
  await expect(fire).toHaveText('Walls: fires at units within 2 tiles');
  await selectCity(page, setup.open);
  await expect(fire).toHaveText("No Walls: can't fire");
  await selectCity(page, setup.bastion);
  await expect(fire).toHaveText('Alpine Redoubt: fires at units within 2 tiles');
  await selectCity(page, setup.own);
  await expect(fire).toHaveText('Walls: fires at units within 2 tiles');
});

test('attack forecasts show city fire only for walled cities', async ({ page }) => {
  await selectUnit(page, setup.nearOpen);
  const card = page.locator('.forecast');
  await hoverForecast(page, setup.open, page.getByTestId('forecast-no-city-fire'));
  await expect(page.getByTestId('forecast-no-city-fire')).toContainText('cannot fire back: it has no Walls');
  await expect(card).not.toContainText('will fire back');
  await page.screenshot({ path: path.join(shots, 'forecast-unwalled-1440.png') });

  await selectUnit(page, setup.nearWalled);
  await hoverForecast(page, setup.walled, page.getByTestId('forecast-city-fire'));
  await expect(page.getByTestId('forecast-city-fire')).toContainText('fires first');
  await expect(page.getByTestId('forecast-no-city-fire')).toHaveCount(0);
  await page.screenshot({ path: path.join(shots, 'forecast-walled-1440.png') });
});

// Playtest-6 H1: the walls' shot was a grey footnote while the headline said "100 → 63". The
// headline HP result counts every consequence, city fire included, and the card fits at 1000x600.
for (const [width, height] of [[1440, 900], [1000, 600]] as const) {
  test(`the walled-city forecast headline includes the city's shot at ${width}×${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await selectUnit(page, setup.nearWalled);
    const card = page.locator('.forecast');
    await hoverForecast(page, setup.walled, page.getByTestId('forecast-city-fire'));
    // "100 → 20 HP" with "−37 city fire, −43 attack" under it: the headline is the sum.
    const hp = (await page.getByTestId('forecast-attacker-hp').innerText()).match(/(\d+)\s*→\s*(\d+)/)!.map(Number);
    const split = (await card.locator('.fside-split').innerText()).match(/−(\d+) city fire, −(\d+) attack/)!.map(Number);
    expect(split[1]).toBeGreaterThan(0);
    expect(hp[2]).toBe(hp[1] - split[1] - split[2]);
    // The red risk row sits above the odds, not among the footnotes.
    const risk = (await page.getByTestId('forecast-city-fire').boundingBox())!, odds = (await card.locator('.forecast-odds').boundingBox())!;
    expect(risk.y).toBeLessThan(odds.y);
    // The whole card is inside the window.
    await page.waitForTimeout(200);
    const box = (await card.boundingBox())!;
    expect(box.y).toBeGreaterThanOrEqual(0);
    expect(box.y + box.height).toBeLessThanOrEqual(height);
    expect(await card.evaluate((el) => el.scrollHeight <= el.clientHeight + 1)).toBe(true);
  });
}

for (const [name, width, height] of [['1440', 1440, 900], ['1000', 1000, 600]] as const) {
  test(`walls are drawn on the map at ${name}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await selectCity(page, setup.walled);
    await cityPoint(page, setup.walled);
    await expect(page.getByTestId('city-fire').first()).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: path.join(shots, `walls-${name}.png`) });
    // The Switzerland' Alpine Redoubt is drawn as a star fort.
    await selectCity(page, setup.bastion);
    await cityPoint(page, setup.bastion);
    await page.screenshot({ path: path.join(shots, `bastion-${name}.png`) });
  });
}

import { expectRaceWhole } from './tour/helpers';
import type { Browser, Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { auditOverlaySlots } from './layoutAudit';

// Playtest 3, G5: a Civ-style minimap. Desktop and tablets get a corner panel under or beside the
// turn dock.

type Cam = { x: number; y: number; zoom: number };
type GameWindow = Window & { legatum: typeof useGame; legatumMap: { camera: Cam } };

type Watch = (context: import('@playwright/test').BrowserContext) => void;

async function start(browser: Browser, baseURL: string | undefined, watchErrors: Watch, size: { width: number; height: number }, touch = false) {
  const context = await browser.newContext({ baseURL, viewport: size, hasTouch: touch, isMobile: touch && size.width < 700 });
  watchErrors(context);
  await context.addInitScript(() => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  const page = await context.newPage();
  await page.goto('/?client=wasm');
  const press = (name: RegExp) => touch ? page.getByRole('button', { name }).tap() : page.getByRole('button', { name }).click();
  await press(/^Single Player/);
  // A new lobby has 6 empires; these checks are about a full table of 8, so add the last two.
  for (let i = 0; i < 2; i++) await press(/^\+ Add an AI empire/);
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await press(/^Start game/);
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  return { context, page };
}

/** Overlay slots that intersect, ignoring the full-screen map canvas layers the HUD floats over on desktop and tablet. */
const overlaps = async (page: Page) => (await auditOverlaySlots(page))
  .filter((i) => !/^(map-viewport|counter-flight|map-compass)\b/.test(i.where) && !/^(map-viewport|counter-flight|map-compass):/.test(i.detail));

async function expectOwnRaceRow(page: Page, _options: { allRows: boolean }) {
  await expect(page.locator('.vp-row')).toHaveCount(8);
  await expect(page.getByTestId('minimap-canvas')).toBeVisible();
  await expectRaceWhole(page);
}

const camera = (page: Page) => page.evaluate(() => ({ ...(window as unknown as GameWindow).legatumMap.camera }));

test('desktop minimap: visible in the corner, click and drag move the map, shortcuts still work, collapse is remembered', async ({ browser, baseURL, watchErrors }) => {
  const { context, page } = await start(browser, baseURL, watchErrors, { width: 1440, height: 900 });
  try {
    const panel = page.getByRole('region', { name: 'Minimap' });
    const canvas = panel.getByTestId('minimap-canvas');
    await expect(canvas).toBeVisible();
    // Bottom-right corner, under the turn dock, and clear of every other overlay.
    const box = (await panel.boundingBox())!;
    expect(box.x + box.width).toBeGreaterThan(1440 - 24);
    expect(box.y + box.height).toBeLessThanOrEqual(900 - 8);
    const dock = (await page.locator('[data-overlay-slot="turn-dock"]').boundingBox())!;
    expect(dock.y + dock.height).toBeLessThanOrEqual(box.y);
    expect(await overlaps(page)).toEqual([]);
    await expect(canvas).toHaveCSS('cursor', 'pointer');

    // Click: the main map jumps there.
    const c = (await canvas.boundingBox())!;
    const before = await camera(page);
    await canvas.click({ position: { x: c.width * 0.08, y: c.height * 0.5 } });
    await expect.poll(async () => (await camera(page)).x).toBeLessThan(before.x - 200);
    const left = await camera(page);

    // Drag: the map follows the pointer across the minimap.
    await page.mouse.move(c.x + c.width * 0.1, c.y + c.height * 0.5);
    await page.mouse.down();
    await page.mouse.move(c.x + c.width * 0.5, c.y + c.height * 0.5, { steps: 4 });
    const mid = await camera(page);
    await page.mouse.move(c.x + c.width * 0.9, c.y + c.height * 0.5, { steps: 4 });
    await page.mouse.up();
    const right = await camera(page);
    expect(mid.x).toBeGreaterThan(left.x + 100);
    expect(right.x).toBeGreaterThan(mid.x + 100);

    // The minimap never keeps the keyboard: the map has focus and the arrow keys still pan.
    await expect(page.locator('[data-game-map]')).toBeFocused();
    await page.keyboard.down('ArrowUp');
    await page.waitForTimeout(250);
    await page.keyboard.up('ArrowUp');
    expect((await camera(page)).y).toBeLessThan(right.y - 20);

    // Collapse is remembered on this device.
    await panel.getByRole('button', { name: 'Hide minimap' }).click();
    await expect(canvas).toHaveCount(0);
    await expect(panel.getByRole('button', { name: 'Show minimap' })).toBeVisible();
    await page.reload();
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1');
    await expect(page.getByRole('button', { name: 'Show minimap' })).toBeVisible();
    await page.getByRole('button', { name: 'Show minimap' }).click();
    await expect(page.getByTestId('minimap-canvas')).toBeVisible();
  } finally {
    await context.close();
  }
});

for (const { size, touch, allRows } of [
  { size: { width: 1440, height: 900 }, touch: false, allRows: true },
  { size: { width: 1024, height: 768 }, touch: true, allRows: false },
]) {
  test(`${size.width}×${size.height} with the minimap: your own race row is never clipped${allRows ? ', and all eight rows fit' : ''}`, async ({ browser, baseURL, watchErrors }) => {
    const { context, page } = await start(browser, baseURL, watchErrors, size, touch);
    try {
      await expectOwnRaceRow(page, { allRows });
      expect(await overlaps(page)).toEqual([]);
    } finally {
      await context.close();
    }
  });
}

test('desktop minimap: the viewport frame follows panning and zooming of the main map', async ({ browser, baseURL, watchErrors }) => {
  const { context, page } = await start(browser, baseURL, watchErrors, { width: 1440, height: 900 });
  try {
    const canvas = page.getByTestId('minimap-canvas');
    await expect(canvas).toBeVisible();
    const pixels = () => canvas.evaluate((el: HTMLCanvasElement) => el.toDataURL());
    const first = await pixels();
    await page.locator('[data-game-map]').focus();
    await page.keyboard.down('ArrowRight');
    await page.waitForTimeout(300);
    await page.keyboard.up('ArrowRight');
    await expect.poll(pixels).not.toBe(first);
    const panned = await pixels();
    const map = (await page.locator('[data-game-map]').boundingBox())!;
    await page.mouse.move(map.x + map.width / 2, map.y + map.height / 2);
    await page.mouse.wheel(0, 400);
    await expect.poll(pixels).not.toBe(panned);
  } finally {
    await context.close();
  }
});

for (const size of [{ width: 1024, height: 768 }, { width: 1366, height: 1024 }]) {
  test(`tablet ${size.width}×${size.height}: the minimap sits beside the turn dock without overlapping anything`, async ({ browser, baseURL, watchErrors }) => {
    const { context, page } = await start(browser, baseURL, watchErrors, size, true);
    try {
      const panel = page.getByRole('region', { name: 'Minimap' });
      await expect(panel.getByTestId('minimap-canvas')).toBeVisible();
      const box = (await panel.boundingBox())!;
      expect(box.y + box.height).toBeLessThanOrEqual(size.height - 8);
      expect(await overlaps(page)).toEqual([]);
      // The end-turn action is untouched in the corner.
      await expect(page.locator('.seatbar .cta').first()).toBeInViewport();
      const before = await camera(page);
      const c = (await panel.getByTestId('minimap-canvas').boundingBox())!;
      // Bottom-left: the hide toggle sits over the top-right corner.
      await page.touchscreen.tap(c.x + c.width * 0.1, c.y + c.height * 0.85);
      await expect.poll(async () => { const now = await camera(page); return Math.hypot(now.x - before.x, now.y - before.y); }).toBeGreaterThan(100);
    } finally {
      await context.close();
    }
  });
}

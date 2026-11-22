// The Settle lens shows every legal city site on an absolute five-band scale (user, 2026-10-11, playtest 8.8.1),
// with the relative top-three markers on top. The busy art-review map (eight empires, borders, fog, wonders and
// resources) is the stage; the real WASM core scores it. Shots: SETTLE_SHOTS=<dir>, default test-results/settle-lens.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { settleLensFixture } from '../src/test/settleLensFixture';
import { showMapOptions } from './overlayHelpers';

type Tile = { index: number; band?: string; rank?: number };
type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; lensTiles: Tile[]; toScreen(h: { q: number; r: number }): { x: number; y: number } } };
const out = process.env.SETTLE_SHOTS ? path.resolve(process.env.SETTLE_SHOTS) : path.resolve('test-results/settle-lens');
const BANDS = ['Poor', 'Fair', 'Good', 'Great', 'Exceptional'];

const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0, strictPort: false } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('Vite did not bind');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

const { rich, indiaThreeHexSite, ...setup } = settleLensFixture();

async function open(page: Page) {
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(setup.fixture) });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

/** Select the Settler (the lens turns on by itself), centre it and set the zoom. */
async function frameSettler(page: Page, zoom: number) {
  await page.evaluate(({ id, at, zoom }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select({ kind: 'unit', id });
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { id: setup.settler!, at: setup.settlerAt!, zoom });
  await page.waitForTimeout(600);
}

async function park(page: Page) { await page.mouse.move(2, 2); await page.waitForTimeout(250); }
async function shot(page: Page, size: string, name: string) {
  await park(page);
  await mkdir(`${out}/${size}`, { recursive: true });
  await page.screenshot({ path: `${out}/${size}/${name}.png`, animations: 'disabled' });
}
const hoverHex = async (page: Page, hex: { q: number; r: number }) => {
  const at = await page.evaluate((hex) => {
    const p = (window as unknown as MapWindow).legatumMap.toScreen(hex), b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: b.left + p.x, y: b.top + p.y };
  }, hex);
  await page.mouse.move(at.x - 4, at.y - 4); await page.mouse.move(at.x, at.y);
  await page.waitForTimeout(900);
};

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    test('Settle tints every legal site by an absolute band, with a legend and top-three markers', async ({ page }) => {
      test.setTimeout(120_000);
      const size = `${width}x${height}`;
      await open(page);
      await frameSettler(page, 0.9);
      // The legend: five bands, in order, each with its colour, and the fixed-scale promise.
      await expect(page.locator('.settle-scale [data-settle-band]')).toHaveText(BANDS.map((b) => new RegExp(`^${b}`)));
      await expect(page.locator('.settle-legend')).toContainText('Better places to found a city are darker green');
      await expect(page.locator('.settle-legend')).not.toContainText('greener');
      // No dots and no numbers: the swatches are one green from faint to deep, each with its name.
      await expect(page.locator('.settle-pips, [data-testid="settle-floor"]')).toHaveCount(0);
      await expect(page.locator('.settle-scale')).not.toContainText(/\d/);
      const swatches = await page.locator('.settle-scale .settle-swatch').evaluateAll((els) => els.map((el) => getComputedStyle(el).backgroundColor.match(/[\d.]+/g)!.slice(0, 3).map(Number)));
      expect(swatches).toHaveLength(5);
      const lightness = swatches.map(([r, g, b]) => 0.2126 * r + 0.7152 * g + 0.0722 * b);
      for (let i = 0; i < 4; i++) expect(lightness[i], `${BANDS[i]} is lighter than ${BANDS[i + 1]}`).toBeGreaterThan(lightness[i + 1]);
      for (const [r, g, b] of swatches) expect(g, 'every swatch is green').toBeGreaterThan(Math.max(r, b));
      // The key for the top three sits under the scale, in the same words as the marks on the map.
      await expect(page.getByTestId('settle-top-key')).toContainText('Top three');
      // The map lens and the core's inspected site blocker agree on every legal tile.
      const verdict = await page.evaluate(async () => {
        const w = window as unknown as MapWindow;
        const view = w.legatum.getState().views![0];
        const path = '/src/settlement.ts';
        const { knownLegalSite } = await import(/* @vite-ignore */ path) as typeof import('../src/settlement');
        const hexAt = (i: number) => ({ q: i % view.width - Math.floor(Math.floor(i / view.width) / 2), r: Math.floor(i / view.width) });
        const legal = view.tiles.map((_, i) => i).filter((i) => knownLegalSite(view, hexAt(i)));
        const tinted = w.legatumMap.lensTiles.filter((t) => t.band).map((t) => t.index).sort((a, b) => a - b);
        const tiles = w.legatumMap.lensTiles;
        return { legal, tinted, bands: [...new Set(tiles.map((t) => t.band))], ranks: tiles.filter((t) => t.rank).map((t) => t.rank).sort(), hasMapRiskMark: tiles.some((t) => 'risk' in t) };
      });
      expect(verdict.legal.length).toBeGreaterThan(100);
      expect(verdict.tinted).toEqual(verdict.legal);
      expect(verdict.bands.length).toBeGreaterThanOrEqual(3);
      expect(rich, 'the staged rich site found open land').not.toBeNull();
      expect(verdict.bands).toContain('Exceptional');
      expect(verdict.ranks).toEqual([1, 2, 3]);
      expect(verdict.hasMapRiskMark).toBe(false);
      // The taller legend must not squeeze the Settler's primary action out of view (the unit panel yields height to the dock).
      const found = await page.evaluate(() => {
        const b = [...document.querySelectorAll<HTMLButtonElement>('.unit-panel button')].find((x) => /^Found City/.test(x.textContent ?? ''))!;
        const r = b.getBoundingClientRect(), hit = (x: number, y: number) => b.contains(document.elementFromPoint(x, y));
        const d = document.querySelector('.map-lenses')!.getBoundingClientRect();
        return { whole: hit(r.left + r.width / 2, r.top + 2) && hit(r.left + r.width / 2, r.bottom - 2), overlapsDock: r.left < d.right && r.right > d.left && r.top < d.bottom && r.bottom > d.top };
      });
      expect(found.whole, 'Found City is fully visible').toBe(true);
      expect(found.overlapsDock).toBe(false);
      await shot(page, size, '01-settle-near');
      // Hover a tinted tile: a plain sentence leads, the key numbers follow, and the arithmetic stays behind Details.
      const site = await page.evaluate(() => {
        const w = window as unknown as MapWindow;
        const view = w.legatum.getState().views![0];
        const t = w.legatumMap.lensTiles.find((t) => t.band && !t.rank)!;
        return { q: t.index % view.width - Math.floor(Math.floor(t.index / view.width) / 2), r: Math.floor(t.index / view.width) };
      });
      await frameSettler(page, 0.9);
      await page.evaluate((hex) => (window as unknown as MapWindow).legatum.getState().jumpTo(hex), site);
      await page.waitForTimeout(500);
      await hoverHex(page, site);
      const card = page.locator('.settlement-card');
      await expect(card).toBeVisible();
      await expect(card).toContainText(/^(Poor|Fair|Good|Great|Exceptional) spot for a city: /);
      await expect(card).toContainText('Nearby land: ');
      await expect(card).not.toContainText(/points|outer ring|×|quality|\bhex/i);
      await expect(card.locator('[data-settle-route]')).toBeVisible();
      await expect(card).not.toContainText('greener');
      const box = await card.boundingBox();
      expect(box!.x).toBeGreaterThanOrEqual(0); expect(box!.x + box!.width).toBeLessThanOrEqual(width);
      expect(box!.y).toBeGreaterThanOrEqual(0); expect(box!.y + box!.height).toBeLessThanOrEqual(height);
      await mkdir(`${out}/${size}`, { recursive: true });
      await page.screenshot({ path: `${out}/${size}/02-hover-site.png` });
      // The staged rich site: the top band, a wonder and strategic resources, in plain numbers.
      await page.evaluate((hex) => (window as unknown as MapWindow).legatum.getState().jumpTo(hex), rich!);
      await page.waitForTimeout(500);
      await hoverHex(page, rich!);
      await expect(card).toContainText(/^Exceptional spot for a city( \(unconfirmed\))?: /);
      await expect(card).toContainText(/Uluru 1 tile away: the city that owns it gets \+25% /);
      await expect(card).toContainText(/Mount Fuji 2 tiles away: the city that owns it gets \+25% /);
      await page.screenshot({ path: `${out}/${size}/02b-hover-exceptional.png` });
      // The click-through keeps the same plain lead and puts every weight and point behind a Details toggle.
      await showMapOptions(page);
      await page.locator('[data-settle-rank="1"]').click();
      const inspector = page.getByLabel('Settlement suitability').last();
      await expect(inspector.locator('[data-settle-band]')).toContainText('spot for a city');
      const details = inspector.locator('.settlement-breakdown');
      await expect(details.locator('summary')).toHaveText('Details');
      expect(await details.evaluate((el) => (el as HTMLDetailsElement).open)).toBe(false);
      expect((await inspector.locator('[data-settle-band], .settlement-reasons, .settlement-route').allTextContents()).join(' ')).not.toMatch(/outer ring|×|points/);
      await shot(page, size, '02c-inspect-closed');
      await details.locator('summary').click();
      await expect(inspector).toContainText('Site quality:');
      await shot(page, size, '02d-inspect-details');
      await page.getByRole('button', { name: 'Close tile actions' }).click();
      // An illegal land tile says why: a tile beside a city is too close.
      const near = await page.evaluate(() => {
        const w = window as unknown as MapWindow;
        const view = w.legatum.getState().views![0];
        const city = view.cities.find((c) => c.owner === view.me)!;
        return { q: city.pos.q + 1, r: city.pos.r };
      });
      await page.evaluate((hex) => (window as unknown as MapWindow).legatum.getState().jumpTo(hex), near);
      await page.waitForTimeout(500);
      await hoverHex(page, near);
      await expect(page.getByTestId('tile-note')).toContainText(/^Cannot settle here: Too close to .+; cities must be at least 3 tiles apart/);
      await page.screenshot({ path: `${out}/${size}/03-hover-illegal.png` });
      expect(indiaThreeHexSite).not.toBeNull();
      await page.evaluate((hex) => (window as unknown as MapWindow).legatum.getState().jumpTo(hex), indiaThreeHexSite!);
      await page.waitForTimeout(500);
      await hoverHex(page, indiaThreeHexSite!);
      await expect(page.locator('.settlement-card')).toBeVisible();
      await expect(page.getByTestId('tile-note')).toHaveCount(0);
      // Zoomed out the tint stays.
      await frameSettler(page, 0.45);
      await shot(page, size, '04-settle-out');
      await frameSettler(page, 1.5);
      await shot(page, size, '05-settle-close');
      // The legend and the top-three buttons still fit at this window size.
      const dock = await page.locator('.map-lenses').boundingBox();
      expect(dock!.y + dock!.height).toBeLessThanOrEqual(height);
      expect(dock!.y).toBeGreaterThan(0);
      await expect(page.locator('[data-settle-rank]')).toHaveText(['Best spot', '#2', '#3']);
    });
  });
}

test.describe('hidden-city founding gap', () => {
  test.use({ viewport: { width: 1000, height: 600 } });
  test('removes a newly blocked explored site from the Settle lens and gives its anonymous tile-card reason', async ({ page }) => {
    test.setTimeout(120_000);
    await open(page);
    await frameSettler(page, 0.9);
    const target = rich!;
    await page.evaluate((hex) => (window as unknown as MapWindow).legatum.getState().jumpTo(hex), target);
    await page.waitForTimeout(500);
    await hoverHex(page, target);
    await expect(page.locator('.settlement-card')).toBeVisible();
    const index = await page.evaluate((hex) => {
      const w = window as unknown as MapWindow;
      const state = w.legatum.getState();
      const view = state.views![0];
      const index = hex.r * view.width + hex.q + Math.floor(hex.r / 2);
      state.set({ views: [{ ...view, city_gap_blocked: [...(view.city_gap_blocked ?? []), hex] }, ...state.views!.slice(1)] });
      return index;
    }, target);
    await expect.poll(() => page.evaluate((i) => (window as unknown as MapWindow).legatumMap.lensTiles.some((t) => t.index === i && !!t.band), index)).toBe(false);
    await hoverHex(page, target);
    await expect(page.locator('.settlement-card')).toHaveCount(0);
    await expect(page.getByTestId('tile-note')).toHaveText('Too close to another city to found one here');
  });
});

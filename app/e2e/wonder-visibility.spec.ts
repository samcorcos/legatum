// A natural wonder shows its bonus and never hides anything: in every lens and the default view its painting
// stays under the neighbours' yield icons and resource medallions, its "+25% [yield]" badge is on the tile, and
// hovering it says what it gives, who holds it and what is still on offer. Also the tile card's order: wonder,
// then resource, then terrain. Screenshots land in WONDER_SHOTS (default test-results/wonder-visibility).
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { setOverlay, showMapOptions } from './overlayHelpers';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { wonderVisibilityFixture } from '../src/test/wonderVisibilityFixture';
import { neighbors } from '../src/hex';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: { q: number; r: number }): { x: number; y: number } } };
const setup = wonderVisibilityFixture();
const wonder0 = setup.wonder;
const SHOTS = process.env.WONDER_SHOTS ?? 'test-results/wonder-visibility';

const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ cacheDir: 'test-results/wonder-visibility/vite-cache', server: { host: '127.0.0.1', port: 0 } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('Vite did not bind');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }] as const;
const LENSES = ['Default', 'Yields', 'Resources', 'Settle', 'Borders', 'Movement', 'Threat', 'Discovery'] as const;

async function load(page: Page, size: { width: number; height: number }) {
  const errors: string[] = [];
  pageErrors.set(page, errors);
  page.on('pageerror', (e) => errors.push(`${e.message} ${e.stack?.split('\n').slice(0, 4).join(' / ') ?? ''}`));
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  await page.setViewportSize({ width: size.width, height: size.height });
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(setup.fixture) });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await page.evaluate((at) => {
    const w = window as unknown as MapWindow;
    w.legatum.getState().jumpTo(at);
    w.legatumMap.camera.zoom = 0.9;
    w.legatumMap.redraw();
  }, wonder0);
  await page.waitForTimeout(650);
}

/** A spot over plain panel text, so no control's tooltip or the map's tile card leaks into a shot. */
const park = (page: Page) => page.mouse.move(page.viewportSize()!.width < 1200 ? 820 : 1263, page.viewportSize()!.width < 1200 ? 355 : 345);

const pageErrors = new WeakMap<Page, string[]>();
/** One view per name: a lens alone, or (Yields, Resources, Borders) the plain map with that one overlay on. */
async function pickLens(page: Page, lens: string) {
  const overlay = ['Yields', 'Resources', 'Borders'].includes(lens);
  const radio = page.getByRole('radiogroup', { name: 'Map lens', exact: true }).getByRole('radio', { name: overlay ? 'Default' : lens, exact: true });
  try {
    await showMapOptions(page);
    await radio.click();
    await showMapOptions(page);
    await expect(radio).toBeChecked();
    for (const name of ['Yields', 'Resources', 'Borders']) await setOverlay(page, name, name === lens);
  } catch (e) {
    await mkdir(SHOTS, { recursive: true });
    await page.screenshot({ path: join(SHOTS, `failure-${lens.toLowerCase()}.png`) });
    throw new Error(`${(e as Error).message}\npage errors: ${(pageErrors.get(page) ?? []).join(' | ')}`);
  }
  await park(page);
}

/**
 * Draws the view twice, with the wonder and without it (everything else equal), and reports what the map shows at the six
 * neighbours: their yield-icon pixels, their resource medallions and the wonder's own badge.
 */
async function measure(page: Page, ring: { q: number; r: number }[], withResources: boolean) {
  return page.evaluate(async ({ ring, at, withResources }) => {
    const w = window as unknown as MapWindow;
    const store = w.legatum;
    const seat = store.getState().seat;
    const original = store.getState().views!;
    const base = structuredClone(original[seat]);
    const width = base.width, height = base.height;
    const idx = (h: { q: number; r: number }) => { const row = h.r, col = h.q + Math.floor((row - (row & 1)) / 2); return col < 0 || row < 0 || col >= width || row >= height ? -1 : row * width + col; };
    const wonderTile = idx(at);
    // In sight, so nothing is drawn faded and the icons keep their exact inks.
    for (const h of [at, ...ring]) { const i = idx(h); if (i >= 0 && base.tiles[i].explored) base.tiles[i].visible = true; }
    if (withResources) for (const h of ring) { const i = idx(h); if (i >= 0 && base.tiles[i].explored && !base.tiles[i].wonder) base.tiles[i].resource = 'Wheat'; }
    const apply = async (view: typeof base) => {
      const views = [...store.getState().views!]; views[seat] = view;
      store.setState({ views });
      w.legatumMap.redraw();
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
      await new Promise((r) => setTimeout(r, 120));
    };
    const canvas = document.querySelector('canvas[data-game-map]') as HTMLCanvasElement;
    const ctx = canvas.getContext('2d')!;
    const k = canvas.width / canvas.clientWidth;
    const r = 36 * w.legatumMap.camera.zoom;
    const read = (x: number, y: number, hw: number, hh: number) => ctx.getImageData(Math.max(0, Math.round((x - hw) * k)), Math.max(0, Math.round((y - hh) * k)), Math.max(1, Math.round(2 * hw * k)), Math.max(1, Math.round(2 * hh * k)));
    const INK = [[78, 118, 46], [150, 80, 32], [128, 98, 13], [35, 107, 146]];
    const near = (a: number[], b: number[], t: number) => Math.abs(a[0] - b[0]) <= t && Math.abs(a[1] - b[1]) <= t && Math.abs(a[2] - b[2]) <= t;
    const yieldPixels = (h: { q: number; r: number }) => {
      const p = w.legatumMap.toScreen(h);
      const d = read(p.x, p.y + r * 0.64, r * 0.45, r * 0.22).data;
      let n = 0; for (let i = 0; i < d.length; i += 4) if (INK.some((c) => near([d[i], d[i + 1], d[i + 2]], c, 10))) n++;
      return n;
    };
    const disc = (h: { q: number; r: number }) => {
      const p = w.legatumMap.toScreen(h);
      const d = read(p.x, p.y - r * 0.68, 3, 3).data;
      const m = [0, 0, 0]; const n = d.length / 4;
      for (let i = 0; i < d.length; i += 4) { m[0] += d[i] / n; m[1] += d[i + 1] / n; m[2] += d[i + 2] / n; }
      return m;
    };
    const badgePixels = () => {
      const p = w.legatumMap.toScreen(at);
      const d = read(p.x, p.y - r * 0.46, 14, 6).data;
      let n = 0; for (let i = 0; i < d.length; i += 4) if (near([d[i], d[i + 1], d[i + 2]], [247, 241, 227], 32)) n++;
      return n;
    };
    const rows = ring.map((h) => ({ h, i: idx(h) })).filter(({ i }) => i >= 0 && base.tiles[i].explored && !base.tiles[i].wonder);
    const snap = () => ({ yields: rows.map(({ h }) => yieldPixels(h)), discs: rows.map(({ h }) => disc(h)), badge: badgePixels() });
    await apply(base);
    const withWonder = snap();
    const bare = structuredClone(base); bare.tiles[wonderTile].wonder = null;
    await apply(bare);
    const without = snap();
    await apply(original[seat]);
    return { withWonder, without, tiles: rows.length };
  }, { ring, at: wonder0, withResources });
}

for (const size of SIZES) {
  test(`the wonder tile in every lens at ${size.name}: badge on the tile, nothing of the neighbours hidden`, async ({ page }) => {
    test.setTimeout(120_000);
    await load(page, size);
    await mkdir(SHOTS, { recursive: true });
    const ring = neighbors(wonder0);
    const wonderAt = await page.evaluate((h) => (window as unknown as MapWindow).legatumMap.toScreen(h), wonder0);
    const box = (await page.locator('canvas[data-game-map]').boundingBox())!;
    const clip = { x: Math.max(0, box.x + wonderAt.x - 150), y: Math.max(0, box.y + wonderAt.y - 110), width: 300, height: 220 };
    for (const lens of LENSES) {
      await pickLens(page, lens);
      // Resource medallions on every neighbour, so their top slots (the ones the painting overhangs) are drawn.
      // The release check runs many browser workers at once. A view swap can be observed before
      // the map's next paint under load; retry the complete with/without comparison, never a pixel.
      await expect.poll(async () => {
        const m = await measure(page, ring, lens === 'Default' || lens === 'Resources');
        const problems: string[] = [];
        if (m.tiles <= 2) problems.push(`${lens}: only ${m.tiles} explored neighbours`);
        if (m.withWonder.badge <= 60) problems.push(`${lens}: wonder badge pixels ${m.withWonder.badge}`);
        if (lens === 'Yields') {
          if (m.without.yields.reduce((a, b) => a + b, 0) <= 20) problems.push('Yields: the neighbours show no yield icons to compare');
          m.withWonder.yields.forEach((n, i) => { if (n < Math.floor(m.without.yields[i] * 0.95)) problems.push(`Yields: neighbour ${i} icon pixels ${n} < ${m.without.yields[i]}`); });
        }
        if (lens === 'Default' || lens === 'Resources') {
          m.withWonder.discs.forEach((rgb, i) => rgb.forEach((c, ch) => { if (Math.abs(c - m.without.discs[i][ch]) >= 6) problems.push(`${lens}: neighbour ${i} medallion channel ${ch}: ${c.toFixed(0)} vs ${m.without.discs[i][ch].toFixed(0)}`); }));
        }
        return problems;
      }, { timeout: 15_000, intervals: [250, 500, 1000] }).toEqual([]);
      await park(page);
      await page.waitForTimeout(150);
      await page.screenshot({ path: join(SHOTS, `${size.name}-${lens.toLowerCase()}.png`), animations: 'disabled' });
      await page.screenshot({ path: join(SHOTS, `${size.name}-${lens.toLowerCase()}-crop.png`), clip, animations: 'disabled' });
    }
  });

  test(`hovering the wonder explains it at ${size.name}`, async ({ page }) => {
    await load(page, size);
    await mkdir(SHOTS, { recursive: true });
    const at = await page.evaluate((h) => (window as unknown as MapWindow).legatumMap.toScreen(h), wonder0);
    const box = (await page.locator('canvas[data-game-map]').boundingBox())!;
    await page.mouse.move(box.x + at.x - 4, box.y + at.y + 3);
    await page.mouse.move(box.x + at.x, box.y + at.y);
    const card = page.locator('.tilecard');
    await expect(card).toBeVisible();
    await expect(card.getByTestId('wonder-bonus')).toContainText(/The city that owns this tile gets \+25% (Science|Production|Food|Gold)\./);
    await expect(card.getByTestId('wonder-owner')).toContainText(/Unclaimed|Held by/);
    await expect(card.getByTestId('wonder-discovery')).toContainText(/Discovery reward: (\d+ (Science|Gold), paid once to the first empire to see it\.|already paid)/);
    const r = (await card.boundingBox())!;
    expect(r.x).toBeGreaterThanOrEqual(0); expect(r.y).toBeGreaterThanOrEqual(0);
    expect(r.x + r.width).toBeLessThanOrEqual(size.width); expect(r.y + r.height).toBeLessThanOrEqual(size.height);
    await page.screenshot({ path: join(SHOTS, `${size.name}-wonder-tooltip.png`), animations: 'disabled' });
  });

  test(`a resource leads its tile card at ${size.name}`, async ({ page }) => {
    await load(page, size);
    await mkdir(SHOTS, { recursive: true });
    const hex = neighbors(wonder0)[3];
    await page.evaluate(async (h) => {
      const w = window as unknown as MapWindow;
      const s = w.legatum.getState(), seat = s.seat;
      const v = structuredClone(s.views![seat]);
      const row = h.r, col = h.q + Math.floor((row - (row & 1)) / 2);
      const t = v.tiles[row * v.width + col];
      t.terrain = 'Forest'; t.resource = 'Iron'; t.owner = null; t.wonder = null;
      const views = [...s.views!]; views[seat] = v; w.legatum.setState({ views }); w.legatumMap.redraw();
    }, hex);
    await page.waitForTimeout(300);
    const at = await page.evaluate((h) => (window as unknown as MapWindow).legatumMap.toScreen(h), hex);
    const box = (await page.locator('canvas[data-game-map]').boundingBox())!;
    await page.mouse.move(box.x + at.x - 4, box.y + at.y + 3);
    await page.mouse.move(box.x + at.x, box.y + at.y);
    const card = page.locator('.tilecard');
    await expect(card).toBeVisible();
    await expect(card.locator('> :first-child')).toHaveAttribute('data-testid', 'tile-resource');
    const art = (await card.locator('.tilecard-lead img').boundingBox())!;
    expect(art.width).toBeGreaterThanOrEqual(40);
    await expect(card.locator('.tilecard-lead-name')).toHaveText('Iron');
    await expect(card.locator('.tilecard-lead-kind')).toHaveText('Strategic resource');
    await expect(card.locator('.tilecard-benefits li')).toHaveCount(2);
    await expect(card.getByTestId('tile-improvement')).toContainText('Unclaimed: bring it inside your borders');
    await expect(card.locator('.tilecard-terrain')).toHaveText('Forest');
    const r = (await card.boundingBox())!;
    expect(r.y).toBeGreaterThanOrEqual(0); expect(r.y + r.height).toBeLessThanOrEqual(size.height);
    expect(r.x + r.width).toBeLessThanOrEqual(size.width);
    await page.screenshot({ path: join(SHOTS, `${size.name}-resource-tooltip.png`), animations: 'disabled' });
  });

  test(`a city that owns the wonder lists it, with the amount, at ${size.name}`, async ({ page }) => {
    await load(page, size);
    await mkdir(SHOTS, { recursive: true });
    // Hand the wonder tile to my capital, as the core would (territory, owner, and the breakdown source it adds).
    const city = await page.evaluate((h) => {
      const w = window as unknown as MapWindow;
      const s = w.legatum.getState(), seat = s.seat;
      const v = structuredClone(s.views![seat]);
      const row = h.r, col = h.q + Math.floor((row - (row & 1)) / 2);
      const t = v.tiles[row * v.width + col];
      const mine = v.cities.find((c) => c.owner === v.me && c.detail)!;
      t.owner = v.me; t.visible = true;
      mine.detail!.territory.push(h);
      mine.detail!.yields.sources.push({ label: 'Great Barrier Reef (+25%)', yields: { food: 0, production: 0, gold: 0, science: 3 } });
      mine.detail!.yields.total.science += 3;
      const views = [...s.views!]; views[seat] = v; w.legatum.setState({ views });
      s.select({ kind: 'city', id: mine.id });
      return { id: mine.id, name: mine.name, pos: mine.pos };
    }, wonder0);
    const row = page.getByTestId('city-wonders');
    await expect(row).toBeVisible();
    await expect(row).toContainText('Great Barrier Reef');
    await expect(row).toContainText('+25% Science');
    await expect(row).toContainText('+3');
    // The breakdown names it as its own line.
    await page.locator('.cp-economy > summary').click();
    // A click pins the full breakdown (a hover shows only its one-line summary).
    await page.locator('.cp-yield').nth(3).click();
    const line = page.locator('.bd-row[data-wonder="GreatBarrierReef"]');
    await expect(line).toContainText('Great Barrier Reef +25%');
    await expect(line).toContainText('+3');
    await page.screenshot({ path: join(SHOTS, `${size.name}-city-breakdown.png`), animations: 'disabled' });
    // And the tile's own card says who holds it (the city panel is closed first: at 1440 it would cover the tile).
    await page.evaluate(() => (window as unknown as MapWindow).legatum.getState().select(null));
    await park(page);
    await page.evaluate((h) => { const w = window as unknown as MapWindow; w.legatum.getState().jumpTo(h); w.legatumMap.camera.zoom = 0.9; w.legatumMap.redraw(); }, wonder0);
    await page.waitForTimeout(650);
    const at = await page.evaluate((h) => (window as unknown as MapWindow).legatumMap.toScreen(h), wonder0);
    const box = (await page.locator('canvas[data-game-map]').boundingBox())!;
    await page.mouse.move(box.x + at.x - 4, box.y + at.y + 3);
    await page.mouse.move(box.x + at.x, box.y + at.y);
    await expect(page.getByTestId('wonder-owner')).toContainText(`Held by your city ${city.name}: +3 Science each turn.`);
    await page.screenshot({ path: join(SHOTS, `${size.name}-owned-wonder-tooltip.png`), animations: 'disabled' });
  });
}

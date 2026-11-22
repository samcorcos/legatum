// Distinctness review shots (user, 2026-10-11: "Coal and iron look way too similar... All the icons, miniatures,
// etc need to be visually distinct"): a staged map with every resource side by side (Iron beside Coal), and rows of
// same-class units across eras, at normal and zoomed-out zoom, 1000x600 and 1440x900.
// ART_DISTINCT_SHOTS=<dir> picks where the shots go. Shots only.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';
import { fromOffset, hexEq } from '../src/hex';
import { TECH_NAME } from '../src/data';
import { spawnUnit } from '../src/test/fixtureEngine';
import { idxOf, refreshVision } from '../src/test/fixtureEngine';
import type { Hex, Resource, UnitKind } from '../src/types';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void } };
const out = process.env.ART_DISTINCT_SHOTS ? path.resolve(process.env.ART_DISTINCT_SHOTS) : path.resolve('test-results/art-distinct');

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

const RESOURCES: Resource[] = ['Iron', 'Coal', 'Copper', 'Wheat', 'Cattle', 'Stone'];
const ROWS: UnitKind[][] = [
  ['Horseman', 'Cataphract', 'Cuirassier', 'Knight', 'Cavalry'],
  ['Ranger', 'CompositeArcher', 'Marksman', 'Arquebusier'],
  ['Soldier', 'Phalangite', 'Trooper', 'AssaultTrooper', 'Rifleman'],
];

/** The art-map fixture plus a staged strip: the six resources in a row, three rows of units, all explored and known. */
function staged() {
  const setup = mapArtFixture();
  const g = setup.fixture;
  g.resource = g.resource ?? new Array(g.width * g.height).fill(null);
  // Rust-generated starting positions vary. Reserve a clear, central strip
  // instead of relying on six naturally passable neighbours of the capital.
  const candidates = Array.from({ length: g.height - 6 }, (_, row) =>
    Array.from({ length: g.width - 5 }, (_, col) => ({ col, row }))).flat()
    .sort((a, b) => Math.abs(a.col + 3 - g.width / 2) + Math.abs(a.row + 3 - g.height / 2)
      - Math.abs(b.col + 3 - g.width / 2) - Math.abs(b.row + 3 - g.height / 2));
  const rowsOfTiles = candidates.map(({ col, row }) =>
    Array.from({ length: 4 }, (_, r) => Array.from({ length: 6 }, (_, k) => fromOffset(col + k, row + r * 2))))
    .find((rows) => rows.flat().every((h) => !g.units.some((u) => hexEq(u.pos, h)) && !g.cities.some((c) => hexEq(c.pos, h))));
  if (!rowsOfTiles) throw new Error('No clear art strip on the core map');
  for (const h of rowsOfTiles.flat()) { const i = idxOf(g, h); g.terrain[i] = 'Plains'; g.ruin[i] = 0; g.resource[i] = null; }
  RESOURCES.forEach((res, k) => { const h = rowsOfTiles[0][k]; if (h) g.resource![idxOf(g, h)] = res; });
  ROWS.forEach((kinds, r) => kinds.forEach((kind, k) => { const h = rowsOfTiles[r + 1][k]; if (h) spawnUnit(g, 0, kind, h, false); }));
  g.players[0].techs = Object.keys(TECH_NAME) as typeof g.players[0]['techs'];
  refreshVision(g);
  g.players[0].explored.fill(1);
  return { setup, fixture: g, rows: rowsOfTiles, strip: rowsOfTiles.map((r) => r[3]) };
}

const s = staged();
test('the staged rows are long enough', () => {
  expect(s.rows[0].length).toBeGreaterThanOrEqual(6);
  expect(s.rows[1].length).toBeGreaterThanOrEqual(5);
  expect(s.rows[2].length).toBeGreaterThanOrEqual(4);
  expect(s.rows[3].length).toBeGreaterThanOrEqual(5);
});

async function open(page: Page) {
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(s.fixture) });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function frame(page: Page, at: Hex, zoom: number) {
  await page.evaluate(({ at, zoom }) => {
    const w = window as unknown as MapWindow;
    const st = w.legatum.getState();
    st.set({ techOpen: false });
    st.select(null);
    st.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom });
  await expect.poll(() => page.evaluate((at) => {
    const m = (window as unknown as { legatumMap: { toScreen(h: Hex): { x: number; y: number } } }).legatumMap;
    const r = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    const p = m.toScreen(at);
    return Math.hypot(p.x - r.width / 2, p.y - r.height / 2) < 6;
  }, at), { timeout: 5000 }).toBe(true).catch(() => undefined);
  // Park the strip in the left third, clear of the race panel, the dock and the minimap on the right.
  await page.evaluate((zoom) => {
    const w = window as unknown as MapWindow;
    const cam = w.legatumMap.camera as unknown as { x: number; y: number; zoom: number };
    cam.x += (window.innerWidth / 2 - window.innerWidth * 0.33) / zoom;
    w.legatumMap.redraw();
  }, zoom);
  await page.waitForTimeout(600);
}

/** A tight crop around the staged tiles (the full viewport is mostly panels and other empires). */
async function shot(page: Page, size: string, name: string, tiles: Hex[], zoom: number) {
  await mkdir(`${out}/${size}`, { recursive: true });
  const vp = page.viewportSize()!;
  const pts = await page.evaluate((tiles) => {
    const m = (window as unknown as { legatumMap: { toScreen(h: { q: number; r: number }): { x: number; y: number } } }).legatumMap;
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return tiles.map((t) => { const p = m.toScreen(t); return { x: p.x + b.left, y: p.y + b.top }; });
  }, tiles);
  const pad = 62 * zoom + 18;
  const x0 = Math.max(0, Math.min(...pts.map((p) => p.x)) - pad), x1 = Math.min(vp.width, Math.max(...pts.map((p) => p.x)) + pad);
  const y0 = Math.max(0, Math.min(...pts.map((p) => p.y)) - pad), y1 = Math.min(vp.height, Math.max(...pts.map((p) => p.y)) + pad);
  await page.screenshot({ path: `${out}/${size}/${name}.png`, animations: 'disabled', clip: { x: x0, y: y0, width: x1 - x0, height: y1 - y0 } });
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height }, deviceScaleFactor: 2 });
    test('resources and unit classes side by side', async ({ page }) => {
      test.setTimeout(120_000);
      const size = `${width}x${height}`;
      await open(page);
      const row = async (k: number, zoom: number, name: string) => { await frame(page, s.strip[k], zoom); await shot(page, size, name, s.rows[k], zoom); };
      await row(0, 0.9, '01-resources-normal');
      await row(0, 1.4, '02-resources-near');
      await row(0, 0.5, '03-resources-small');
      await row(0, 0.3, '04-resources-far');
      await row(1, 0.9, '05-mounted-normal');
      await row(1, 0.3, '06-mounted-far');
      await row(2, 0.9, '07-ranged-normal');
      await row(2, 0.3, '08-ranged-far');
      await row(3, 0.9, '09-melee-normal');
      await row(3, 0.3, '10-melee-far');
    });
  });
}

// Units on the organic terrain (2026-10-11): the painted miniature must stand out from forest, range, hills, desert and
// grass, at normal and far zoom, at 1000x600 and 1440x900. Two jobs:
//  - review shots (UNIT_SHOTS=<dir>, UNIT_PREFIX=before|after, optional UNIT_QUERY=terrain=s2&organic=1): a clip around
//    one Soldier on each terrain, plus a forest cluster of four seats, a Scout and a Settler, at both zooms;
//  - a contrast check: a Soldier alone in a deep wood must differ in luminance from the art around it (its base, and the
//    pool of vellum either side of the figure, against the bare terrain just outside its halo), by at least the thresholds
//    calibrated on the shots.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';
import { spawnUnit } from '../src/test/fixtureEngine';
import { hexEq, neighbors, within } from '../src/hex';
import type { Hex } from '../src/types';
import type { UnitKind } from '../src/types';
import { lensRadio, setOverlay } from './overlayHelpers';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { x: number; y: number; zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const shots = process.env.UNIT_SHOTS ? path.resolve(process.env.UNIT_SHOTS) : null;
const prefix = process.env.UNIT_PREFIX ?? 'after';
const query = (process.env.UNIT_QUERY ?? '').replace(/,/g, '&');

const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    // Concurrent Playwright workers must not write into the same Vite optimizer cache.
    const cacheDir = await mkdtemp(path.join(tmpdir(), 'legatum-terrain-vite-'));
    let server: Awaited<ReturnType<typeof createServer>> | undefined;
    try {
      server = await createServer({ cacheDir, server: { host: '127.0.0.1', port: 0 } });
      await server.listen();
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('Vite did not bind');
      await use(`http://127.0.0.1:${address.port}`);
    } finally {
      await server?.close();
      await rm(cacheDir, { recursive: true, force: true });
    }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

// Pin a deep-wood scene under the current map rules for the contrast check below.
const SEED = '2914831';
const hexOf = (col: number, row: number): Hex => ({ q: col - Math.floor((row - (row & 1)) / 2), r: row });

const TERRAINS = ['Forest', 'Mountains', 'Hills', 'Desert', 'Grassland'] as const;

function scene() {
  const g = mapArtFixture(SEED).fixture;
  const W = g.width, H = g.height;
  const at = (h: Hex) => { const r = h.r, c = h.q + Math.floor((r - (r & 1)) / 2); return c < 0 || r < 0 || c >= W || r >= H ? -1 : r * W + c; };
  const terrainAt = (h: Hex): string | null => { const i = at(h); return i < 0 ? null : g.terrain[i]; };
  const taken = (h: Hex) => g.units.some((u) => hexEq(u.pos, h)) || g.cities.some((c) => hexEq(c.pos, h)) || g.resource?.[at(h)] || g.wonder?.[at(h)] || g.ruin?.[at(h)];
  const used: Hex[] = [];
  /** The tile of a terrain with the most same-terrain tiles within two steps and no city, unit or resource near, far from tiles already used. */
  const interior = (t: string, minGap = 6): Hex => {
    let best: Hex | null = null, bestScore = -1;
    for (let r = 4; r < H - 4; r++) for (let c = 4; c < W - 4; c++) {
      const h = hexOf(c, r);
      if (terrainAt(h) !== t || taken(h) || used.some((u) => Math.max(Math.abs(u.q - h.q), Math.abs(u.r - h.r), Math.abs(u.q + u.r - h.q - h.r)) < minGap)) continue;
      if (within(h, 2).some((n) => g.cities.some((cc) => hexEq(cc.pos, n)))) continue;
      const score = within(h, 2).filter((n) => terrainAt(n) === t).length;
      if (score > bestScore) { bestScore = score; best = h; }
    }
    if (!best) throw new Error(`no ${t} tile`);
    used.push(best);
    return best;
  };
  const spots: Record<string, Hex> = {};
  for (const t of TERRAINS) { spots[t] = interior(t); spawnUnit(g, 0, 'Soldier', spots[t], false); }
  // A forest cluster: seats 1-3 soldiers, a Scout and a Settler of seat 0, side by side in one wood, so owner colours and unit kinds can be compared.
  const deep = interior('Forest');
  const ring = neighbors(deep).filter((n) => terrainAt(n) === 'Forest' && !taken(n));
  const cluster: [number, UnitKind][] = [[0, 'Soldier'], [1, 'Soldier'], [2, 'Soldier'], [3, 'Soldier'], [0, 'Scout'], [0, 'Settler']];
  const clusterAt: Hex[] = [];
  cluster.forEach(([seat, kind], k) => {
    // The Settler shares the first Soldier's tile, so a two-unit stack is shot as well.
    const h = k === 0 || kind === 'Settler' ? deep : ring[k - 1];
    if (!h) return;
    clusterAt.push(h);
    spawnUnit(g, seat, kind, h, false);
  });
  // A Settler beside a wood, for the Settle lens.
  const settler = spawnUnit(g, 0, 'Settler', interior('Plains', 4), false);
  // A lone Soldier in the deepest remaining wood, for the contrast check.
  const lone = interior('Forest');
  spawnUnit(g, 0, 'Soldier', lone, false);
  const ownerScout = interior('Desert');
  const enemyScout = spawnUnit(g, 2, 'Scout', ownerScout, false);
  // A Scout of seat 0 beside it, so the enemy's tile is in sight and its tile card names its owner.
  spawnUnit(g, 0, 'Scout', neighbors(ownerScout)[0], false);
  return { g, spots, deep, lone, settler, ownerScout, enemyScout };
}

const sc = scene();
const game = coreFixture(sc.g);

async function open(page: Page, q: string) {
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game });
  await page.goto(`/?client=wasm${q ? `&${q}` : ''}`);
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function frame(page: Page, at: Hex, zoom: number, select?: number) {
  await page.evaluate(({ at, zoom, select }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(select ? { kind: 'unit', id: select } : null);
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom, select });
  await page.mouse.move(60, (page.viewportSize()?.height ?? 900) - 107);
  await page.waitForTimeout(1200);
}

async function centreOf(page: Page, at: Hex) {
  return page.evaluate((at) => {
    const m = (window as unknown as MapWindow).legatumMap;
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    const p = m.toScreen(at);
    return { x: p.x + b.left, y: p.y + b.top };
  }, at);
}

async function clip(page: Page, name: string, at: Hex, half = { w: 150, h: 110 }, out = shots) {
  if (!out) return;
  const c = await centreOf(page, at);
  await mkdir(out, { recursive: true });
  const vp = page.viewportSize()!;
  const x = Math.max(0, Math.min(vp.width - half.w * 2, c.x - half.w)), y = Math.max(0, Math.min(vp.height - half.h * 2, c.y - half.h));
  await page.screenshot({ path: `${out}/${name}.png`, clip: { x, y, width: half.w * 2, height: half.h * 2 }, animations: 'disabled' });
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });

    // Retina, like the user's laptop: the shots are what the user sees.
    test.describe('shots', () => { test.use({ deviceScaleFactor: 2 }); test('units on every terrain', async ({ page }) => {
      test.skip(!shots, 'shots only: set UNIT_SHOTS');
      test.setTimeout(180_000);
      await open(page, query);
      for (const [zoomName, zoom] of [['normal', 0.9], ['far', 0.4]] as const) {
        for (const t of TERRAINS) {
          await frame(page, sc.spots[t], zoom);
          await clip(page, `units-${prefix}-${t.toLowerCase()}-${width}x${height}-${zoomName}`, sc.spots[t]);
        }
        await frame(page, sc.deep, zoom);
        await clip(page, `units-${prefix}-forest-seats-${width}x${height}-${zoomName}`, sc.deep, { w: 200, h: 140 });
      }
    }); });

    // The terrain art fades under the lenses that paint their own tint and icons (round 2 follow-up).
    test.describe('lens shots', () => { test.use({ deviceScaleFactor: 2 }); test('terrain art fades under Settle and a little under the Yields overlay', async ({ page }) => {
      test.skip(!shots, 'shots only: set UNIT_SHOTS');
      test.setTimeout(180_000);
      await open(page, query);
      await mkdir(shots!, { recursive: true });
      const lens = async (name: string) => {
        const overlay = ['Yields', 'Resources'].includes(name);
        await lensRadio(page, overlay ? 'Default' : name).click();
        for (const o of ['Yields', 'Resources']) await setOverlay(page, o, o === name);
        await page.mouse.move(60, (page.viewportSize()?.height ?? 900) - 107);
        await page.waitForTimeout(1500);
      };
      for (const name of ['Default', 'Settle', 'Resources', 'Yields']) {
        await frame(page, sc.deep, 0.9, name === 'Settle' ? sc.settler.id : undefined);
        await lens(name);
        await page.screenshot({ path: `${shots}/lens-${prefix}-${name.toLowerCase()}-${width}x${height}.png`, animations: 'disabled' });
      }
      await lens('Default');
    }); });

    test('a unit in a wood stands out from the art around it', async ({ page }) => {
      test.setTimeout(120_000);
      await open(page, query);
      await frame(page, sc.lone, 0.9);
      const c = await centreOf(page, sc.lone);
      const res = await page.evaluate(async ({ cx, cy }) => {
        const spritesPath = '/src/map/sprites.ts', gridPath = '/src/map/grid.ts';
        const m = await import(/* @vite-ignore */ spritesPath) as typeof import('../src/map/sprites');
        const grid = await import(/* @vite-ignore */ gridPath) as typeof import('../src/map/grid');
        const w = grid.tokenHalf(grid.HEX_SIZE * (window as unknown as MapWindow).legatumMap.camera.zoom);
        // The map's own canvas, read pixel for pixel.
        const canvas = document.querySelector('canvas[data-game-map]') as HTMLCanvasElement;
        const b = canvas.getBoundingClientRect();
        const k = canvas.width / b.width;
        const g = canvas.getContext('2d')!;
        const lum = (x: number, y: number) => {
          const d = g.getImageData(Math.round((x - b.left) * k), Math.round((y - b.top) * k), 1, 1).data;
          const f = (v: number) => { const s = v / 255; return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4; };
          return 0.2126 * f(d[0]) + 0.7152 * f(d[1]) + 0.0722 * f(d[2]);
        };
        const geo = m.miniatureGeometry(w);
        const mean = (pts: [number, number][]) => pts.reduce((a, [x, y]) => a + lum(x, y), 0) / pts.length;
        const disc = (x0: number, y0: number, rx: number, ry: number, from = 0) => {
          const pts: [number, number][] = [];
          for (let y = -ry; y <= ry; y += 1) for (let x = -rx; x <= rx; x += 1) {
            const d = (x / rx) ** 2 + (y / ry) ** 2;
            if (d <= 1 && d >= from) pts.push([x0 + x, y0 + y]);
          }
          return pts;
        };
        // The art around the unit: a ring of bare terrain beyond the halo.
        const R = geo.halo + 3;
        const ring = disc(cx, cy + geo.baseDy * 0.4, R + 14, R + 14, ((R) / (R + 14)) ** 2);
        const base = disc(cx, cy + geo.baseDy, geo.baseRx, geo.baseRy);
        // The pool either side of the figure, where the painting is bare and only the unit's own halo shows.
        const flank = [...disc(cx - geo.flankDx, cy, geo.flankR, geo.flankR), ...disc(cx + geo.flankDx, cy, geo.flankR, geo.flankR)];
        return { ring: mean(ring), base: mean(base), flank: mean(flank) };
      }, { cx: c.x, cy: c.y });
      const contrast = (a: number, b: number) => (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
      const out = { ring: res.ring, base: contrast(res.base, res.ring), flank: contrast(res.flank, res.ring) };
      console.log(`UNIT-CONTRAST ${width}x${height} ${JSON.stringify(out)}`);
      expect(out.base).toBeGreaterThanOrEqual(UNIT_BASE_CONTRAST);
      expect(out.flank).toBeGreaterThanOrEqual(UNIT_HALO_CONTRAST);
    });

    test('an enemy Scout names its empire and shows the empire emblem at normal and far zoom', async ({ page }) => {
      await open(page, query);
      for (const [zoomName, zoom] of [['normal', 0.9], ['far', 0.4]] as const) {
        await frame(page, sc.ownerScout, zoom);
        await page.mouse.move((await centreOf(page, sc.ownerScout)).x, (await centreOf(page, sc.ownerScout)).y);
        const line = page.locator('.tilecard-unit');
        await expect(line).toContainText('Spartan Scout');
        await expect(line).toContainText('Ancient era');
        await expect(line).toContainText('100 HP');
        await expect(line).toContainText('strength 5');
        await expect(line.locator('.faction-mark')).toHaveAttribute('aria-label', 'Sparta');
        await clip(page, `owner-emblem-${width}x${height}-${zoomName}`, sc.ownerScout, { w: 220, h: 150 }, shots ?? 'test-results/unit-owner-identity');
      }
    });
  });
}

/** Calibrated on the deep-wood shots before and after the fix (see the console line `UNIT-CONTRAST`). */
const UNIT_BASE_CONTRAST = 1.3;
const UNIT_HALO_CONTRAST = 1.8;

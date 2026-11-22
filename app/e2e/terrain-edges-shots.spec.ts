// Review shots only (EDGE_SHOTS=<dir>, EDGE_PREFIX=before|after, EDGE_SAVE=<path to a .legatum save>, EDGE_NAME=<label>):
// edge hexes of every painted feature in a real save, read-only, at both window sizes with a 2x crop.
import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { x: number; y: number; zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const shots = process.env.EDGE_SHOTS ? path.resolve(process.env.EDGE_SHOTS) : null;
const prefix = process.env.EDGE_PREFIX ?? 'after';
const name = process.env.EDGE_NAME ?? 'save';
const raw = shots ? readFileSync(process.env.EDGE_SAVE!, 'utf8') : '';
const turn = shots ? String(JSON.parse(raw).turn) : '';

const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ cacheDir: 'test-results/terrain-edges/vite-cache', server: { host: '127.0.0.1', port: 0 } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('Vite did not bind');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height }, deviceScaleFactor: 2 });
    test('feature edges', async ({ page }) => {
      test.skip(!shots, 'shots only: set EDGE_SHOTS');
      test.setTimeout(240_000);
      await page.addInitScript((save) => {
        localStorage.setItem('legatum.autosave.v1', save);
        localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
      }, raw);
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Continue/ }).click();
      await expect(page.getByTestId('turn-number')).toHaveText(turn, { timeout: 90_000 });
      await expect(page.locator('.loading-screen')).toHaveCount(0);
      await page.evaluate(() => document.fonts.ready.then(() => undefined));
      await mkdir(shots!, { recursive: true });
      // Per feature: an explored, all-neighbours-explored tile of that terrain with one or two open (other-terrain land) neighbours, and one with three or more, the closest to the player's first city.
      const targets = await page.evaluate(() => {
        const s = (window as unknown as MapWindow).legatum.getState() as unknown as { seat: number; views: { width: number; height: number; tiles: { terrain: string; explored: boolean }[]; cities: { owner: number; pos: Hex }[]; me: number }[] };
        const v0 = s.views[s.seat] ?? s.views[0];
        const v = v0;
        const W = v.width, H = v.height;
        const cube = (h: Hex) => ({ c: h.q + Math.floor((h.r - (h.r & 1)) / 2), r: h.r });
        const own = v.cities.find((c) => c.owner === v.me)?.pos ?? { q: 0, r: 0 };
        const o = cube(own);
        const nb = (c: number, r: number): [number, number][] => (r & 1
          ? [[c + 1, r], [c + 1, r - 1], [c, r - 1], [c - 1, r], [c, r + 1], [c + 1, r + 1]]
          : [[c + 1, r], [c, r - 1], [c - 1, r - 1], [c - 1, r], [c - 1, r + 1], [c, r + 1]]);
        const t = (c: number, r: number) => (c < 0 || r < 0 || c >= W || r >= H ? null : v.tiles[r * W + c]);
        const out: Record<string, Hex> = {};
        for (const kind of ['Hills', 'Forest', 'Mountains', 'Desert', 'Tundra']) {
          const cands: { c: number; r: number; open: number; d: number }[] = [];
          for (let r = 1; r < H - 1; r++) for (let c = 1; c < W - 1; c++) {
            const me = t(c, r)!;
            if (me.terrain !== kind || !me.explored) continue;
            const n = nb(c, r).map(([a, b]) => t(a, b));
            if (n.some((x) => !x || !x.explored)) continue;
            const open = n.filter((x) => x!.terrain !== kind && x!.terrain !== 'Ocean' && x!.terrain !== 'Coast').length;
            if (open >= 1) cands.push({ c, r, open, d: Math.hypot(c - o.c, r - o.r) });
          }
          cands.sort((a, b) => a.d - b.d);
          const hex = (x: { c: number; r: number }) => ({ q: x.c - Math.floor((x.r - (x.r & 1)) / 2), r: x.r });
          const few = cands.find((x) => x.open <= 2);
          const many = cands.find((x) => x.open >= 3);
          if (few) out[`${kind.toLowerCase()}-edge`] = hex(few);
          if (many) out[`${kind.toLowerCase()}-exposed`] = hex(many);
        }
        return out;
      });
      console.log('targets', JSON.stringify(targets));
      for (const [label, at] of Object.entries(targets)) {
        await page.evaluate((at) => {
          const w = window as unknown as MapWindow;
          const s = w.legatum.getState();
          s.set({ techOpen: false });
          s.select(null);
          s.jumpTo(at);
          w.legatumMap.redraw();
        }, at);
        await page.mouse.move(60, height - 107);
        await page.waitForTimeout(1500);
        const c = await page.evaluate((at) => {
          const m = (window as unknown as MapWindow).legatumMap;
          const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
          const p = m.toScreen(at);
          return { x: p.x + b.left, y: p.y + b.top };
        }, at);
        const base = `${shots}/${name}-${prefix}-${label}-${width}x${height}`;
        await page.screenshot({ path: `${base}.png`, animations: 'disabled' });
        const half = { w: 150, h: 110 };
        const x = Math.max(0, Math.min(width - half.w * 2, c.x - half.w)), y = Math.max(0, Math.min(height - half.h * 2, c.y - half.h));
        await page.screenshot({ path: `${base}-crop2x.png`, clip: { x, y, width: half.w * 2, height: half.h * 2 }, animations: 'disabled' });
      }
    });
  });
}

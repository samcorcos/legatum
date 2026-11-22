// Review shots only (MAP_FIX_SHOTS=<dir>, MAP_FIX_PREFIX=before|after): a grown city (pop 15, Walls, Industrial era)
// beside its farms, and a lone mountain plus a two-mountain group on open land, at normal and far zoom, 1000x600 and 1440x900.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';
import { idxOf } from '../src/test/fixtureEngine';
import { distance, hexEq, within } from '../src/hex';
import { TECHS, TECH_ERA, moveCost } from '../src/data';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const shots = process.env.MAP_FIX_SHOTS ? path.resolve(process.env.MAP_FIX_SHOTS) : null;
const prefix = process.env.MAP_FIX_PREFIX ?? 'after';

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

function scene() {
  const setup = mapArtFixture();
  const g = setup.fixture;
  const home = g.cities.find((c) => c.id === setup.capital)!;
  // Seat 0 in the Industrial era, a metropolis capital with Walls, a mid-sized second city.
  g.players[0].techs = TECHS.filter((t) => TECH_ERA[t] < 4);
  home.population = 15;
  home.buildings = [...home.buildings, 'Walls'];
  const own = g.cities.filter((c) => c.owner === 0 && c.id !== home.id);
  if (own[0]) own[0].population = 9;
  // Keep the town's neighbourhood clear of the counter parade, so the art itself is judged.
  g.units = g.units.filter((u) => g.cities.every((c) => distance(c.pos, u.pos) > 2));
  // Farms around the capital (the improvement art a city must not be paler than).
  for (const h of within(home.pos, 2)) { const i = idxOf(g, h); if (i >= 0 && !hexEq(h, home.pos) && moveCost(g.terrain[i]) !== null && g.owner[i] === 0) (g.improved ??= Array(g.terrain.length).fill(0))[i] = 1; }
  // A lone mountain and a two-peak group on open land, clear of cities and units.
  const open = (h: Hex) => { const i = idxOf(g, h); return i >= 0 && ['Grassland', 'Plains', 'Desert'].includes(g.terrain[i]) && !g.cities.some((c) => distance(c.pos, h) < 2) && !g.units.some((u) => hexEq(u.pos, h)); };
  const clear = (h: Hex) => within(h, 1).every((n) => open(n));
  const ring = within(home.pos, 9).filter((h) => distance(home.pos, h) >= 3).sort((a, b) => distance(home.pos, a) - distance(home.pos, b));
  const lone = ring.find((h) => clear(h) && within(h, 2).every((n) => g.terrain[idxOf(g, n)] !== 'Mountains'))!;
  g.terrain[idxOf(g, lone)] = 'Mountains';
  const pairA = ring.find((h) => distance(h, lone) >= 4 && clear(h) && within(h, 1).some((n) => !hexEq(n, h) && clear(n)) && within(h, 3).every((n) => g.terrain[idxOf(g, n)] !== 'Mountains'))!;
  const pairB = within(pairA, 1).find((n) => !hexEq(n, pairA) && clear(n))!;
  g.terrain[idxOf(g, pairA)] = 'Mountains';
  g.terrain[idxOf(g, pairB)] = 'Mountains';
  return { setup, home: home.pos, second: own[0]?.pos ?? null, lone, pair: pairA };
}

async function open(page: Page, game: string) {
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function frame(page: Page, at: Hex, zoom: number, width: number) {
  await page.evaluate(({ at, zoom }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(null);
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom });
  await page.mouse.move(width / 2, 12);
  await page.waitForTimeout(1500);
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    test('cities and lone mountains', async ({ page }) => {
      test.skip(!shots, 'shots only: set MAP_FIX_SHOTS');
      test.setTimeout(180_000);
      const s = scene();
      console.log('scene', JSON.stringify({ home: s.home, lone: s.lone, pair: s.pair }));
      await open(page, coreFixture(s.setup.fixture));
      await mkdir(shots!, { recursive: true });
      const size = `${width}x${height}`;
      const take = async (name: string, at: Hex) => {
        await page.screenshot({ path: `${shots}/${prefix}-${name}-${size}.png`, animations: 'disabled' });
        const c = await page.evaluate((at) => { const m = (window as unknown as MapWindow).legatumMap; const b = document.querySelector('[data-game-map]')!.getBoundingClientRect(); const p = m.toScreen(at); return { x: p.x + b.left, y: p.y + b.top }; }, at);
        const half = { w: 160, h: 120 };
        const x = Math.max(0, Math.min(width - half.w * 2, c.x - half.w)), y = Math.max(0, Math.min(height - half.h * 2, c.y - half.h));
        await page.screenshot({ path: `${shots}/${prefix}-${name}-${size}-crop.png`, clip: { x, y, width: half.w * 2, height: half.h * 2 }, animations: 'disabled' });
      };
      await frame(page, s.home, 0.9, width); await take('city-normal', s.home);
      await frame(page, s.home, 0.66, width); await take('city-mid', s.home);
      await frame(page, s.home, 0.28, width); await take('city-far', s.home);
      if (s.second) { await frame(page, s.second, 0.9, width); await take('city2-normal', s.second); }
      await frame(page, s.lone, 0.9, width); await take('mountain-lone-normal', s.lone);
      await frame(page, s.lone, 1.6, width); await take('mountain-lone-near', s.lone);
      await frame(page, s.lone, 0.28, width); await take('mountain-lone-far', s.lone);
      await frame(page, s.pair, 0.9, width); await take('mountain-pair-normal', s.pair);
    });
  });
}

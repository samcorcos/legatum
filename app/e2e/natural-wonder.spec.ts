import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: { q: number; r: number }): { x: number; y: number } } };
const setup = mapArtFixture();
const wonderIndex = setup.fixture.wonder!.findIndex((w) => w !== null);
const wonder0 = (() => {
  const row = Math.floor(wonderIndex / setup.fixture.width);
  const col = wonderIndex % setup.fixture.width;
  return { q: col - Math.floor((row - (row & 1)) / 2), r: row };
})();

const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ cacheDir: 'test-results/natural-wonder/vite-cache', server: { host: '127.0.0.1', port: 0 } });
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
for (const size of SIZES) {
  test(`natural wonder on the strategic map at ${size.name}`, async ({ page }) => {
    await page.setViewportSize({ width: size.width, height: size.height });
    expect(wonderIndex).toBeGreaterThanOrEqual(0);
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
    await mkdir('test-results/natural-wonder', { recursive: true });
    await page.screenshot({ path: `test-results/natural-wonder/${size.name}.png`, animations: 'disabled' });
  });
}

// All eight landmarks side by side on the map, so each cutout's edge is judged against real terrain.
test('all eight natural wonders on the map at 1440x900', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  const fx = mapArtFixture();
  const W = fx.fixture.width;
  const all = ['MountFuji', 'VictoriaFalls', 'GrandCanyon', 'GiantsCauseway', 'Uluru', 'HaLongBay', 'IguazuFalls', 'GreatBarrierReef'] as const;
  const wonder = fx.fixture.wonder!.map(() => null) as (typeof all[number] | null)[];
  all.forEach((w, k) => { wonder[wonderIndex + (k % 4) * 3 + Math.floor(k / 4) * 3 * W] = w; });
  fx.fixture.wonder = wonder;
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(fx.fixture) });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await page.evaluate((at) => {
    const w = window as unknown as MapWindow;
    w.legatum.getState().jumpTo({ q: at.q + 5, r: at.r + 3 });
    w.legatumMap.camera.zoom = 0.9;
    w.legatumMap.redraw();
  }, wonder0);
  await page.waitForTimeout(650);
  await mkdir('test-results/natural-wonder', { recursive: true });
  await page.screenshot({ path: 'test-results/natural-wonder/all-eight-1440x900.png', animations: 'disabled' });
});

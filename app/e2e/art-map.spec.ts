// Map art review (art pass 3): the same busy map (src/test/mapArtFixture.ts) at normal and
// zoomed-out zoom, 1000x600 and 1440x900, in the default and Settle lenses and with the Resources and Yields overlays.
// ART_MAP_SHOTS=<dir> picks where the shots go. Shots only: the map tour asserts nothing about pixels.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';
import { lensRadio, setOverlay, showMapOptions } from './overlayHelpers';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void } };
const out = process.env.ART_MAP_SHOTS ? path.resolve(process.env.ART_MAP_SHOTS) : path.resolve('test-results/art-map');

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

const setup = mapArtFixture();

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

/** Centre the camera on a hex (or a unit/city by id) at a zoom, with nothing selected unless asked. */
async function frame(page: Page, at: { q: number; r: number } | null, zoom: number, select?: { kind: 'unit' | 'city'; id: number }) {
  await page.evaluate(({ at, zoom, select }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(select ?? null);
    if (at) s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom, select });
  if (at) await expect.poll(() => page.evaluate((at) => {
    const m = (window as unknown as { legatumMap: { toScreen(h: { q: number; r: number }): { x: number; y: number } } }).legatumMap;
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    const p = m.toScreen(at);
    return Math.hypot(p.x - b.width / 2, p.y - b.height / 2) < 6;
  }, at), { timeout: 5000 }).toBe(true).catch(() => undefined);
  await page.waitForTimeout(500);
}

/** A lens alone, or (Yields, Resources, Borders) the plain map with that one overlay on. */
async function lens(page: Page, name: string) {
  const overlay = ['Yields', 'Resources', 'Borders'].includes(name);
  await showMapOptions(page);
  await lensRadio(page, overlay ? 'Default' : name).click();
  for (const o of ['Yields', 'Resources', 'Borders']) await setOverlay(page, o, o === name);
  await page.waitForTimeout(500);
}

async function shot(page: Page, size: string, name: string) {
  await mkdir(`${out}/${size}`, { recursive: true });
  await page.screenshot({ path: `${out}/${size}/${name}.png`, animations: 'disabled' });
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    test('map art tiers and lenses', async ({ page }) => {
      test.setTimeout(120_000);
      const size = `${width}x${height}`;
      await open(page);
      const home = await page.evaluate((id) => { const v = (window as unknown as MapWindow).legatum.getState().views![0]; return v.cities.find((c) => c.id === id)!.pos; }, setup.capital);
      await frame(page, home, 0.9);
      await shot(page, size, '01-normal');
      if (setup.parade[3]) { await frame(page, setup.parade[3], 1.7); await shot(page, size, '02-counters-near'); }
      if (setup.parade[3]) { await frame(page, setup.parade[3], 0.9); await shot(page, size, '12-seats-normal'); }
      if (setup.parade[7]) { await frame(page, setup.parade[7], 0.9); await shot(page, size, '13-seats-normal-b'); }
      if (setup.camp) { await frame(page, setup.camp, 1.1); await shot(page, size, '03-camp'); }
      await frame(page, home, 0.45); await shot(page, size, '04-out-0.45');
      await frame(page, home, 0.28); await shot(page, size, '05-out-0.28');
      await lens(page, 'Resources');
      await frame(page, home, 0.9); await shot(page, size, '06-resources');
      await frame(page, home, 0.45); await shot(page, size, '07-resources-out');
      await lens(page, 'Yields');
      await frame(page, home, 1.1); await shot(page, size, '08-yields');
      if (setup.settler && setup.settlerAt) {
        await frame(page, setup.settlerAt, 0.9, { kind: 'unit', id: setup.settler });
        await lens(page, 'Settle');
        await shot(page, size, '09-settle');
        await frame(page, setup.settlerAt, 0.45, { kind: 'unit', id: setup.settler }); await shot(page, size, '10-settle-out');
      }
      await lens(page, 'Default');
      await frame(page, home, 2.0); await shot(page, size, '11-india-capital-near');
    });
  });
}

// All eight seats side by side, military and civilian, on map parchment at normal size: each must be identifiable without its number.
test('seat colour strip', async ({ page }) => {
  await open(page);
  await page.evaluate(async () => {
    const spritesPath = '/src/map/sprites.ts', themePath = '/src/theme.ts', stylePath = '/src/map/style.ts';
    const { drawToken, drawPip } = await import(/* @vite-ignore */ spritesPath) as typeof import('../src/map/sprites');
    const { mapPlayerColor } = await import(/* @vite-ignore */ stylePath) as typeof import('../src/map/style');
    const { FACTION_PALETTE, setSeatFactions } = await import(/* @vite-ignore */ themePath) as typeof import('../src/theme');
    setSeatFactions(Object.keys(FACTION_PALETTE) as (keyof typeof FACTION_PALETTE)[]);
    const c = document.createElement('canvas');
    c.width = 9 * 64 + 20; c.height = 200;
    Object.assign(c.style, { position: 'fixed', left: '20px', top: '20px', zIndex: '99999', background: '#a9b98a', border: '1px solid #000' });
    document.body.appendChild(c);
    const g = c.getContext('2d')!;
    g.fillStyle = '#b7c28f'; g.fillRect(0, 0, c.width, 100);
    g.fillStyle = '#d9cc98'; g.fillRect(0, 100, c.width, 100);
    for (let seat = 0; seat < 9; seat++) {
      drawToken(g, 'atlas', 'Soldier', mapPlayerColor(seat), true, 42 + seat * 64, 36, 16, 1, seat);
      drawToken(g, 'atlas', 'Settler', mapPlayerColor(seat), false, 42 + seat * 64, 70, 11, 1, seat);
      drawToken(g, 'atlas', 'Soldier', mapPlayerColor(seat), true, 42 + seat * 64, 136, 11, 1, seat);
      drawPip(g, 'atlas', seat, true, 42 + seat * 64, 176, 6, 1);
    }
  });
  await page.waitForTimeout(500);
  await mkdir(out, { recursive: true });
  await page.locator('canvas').last().screenshot({ path: `${out}/seat-strip.png` });
});

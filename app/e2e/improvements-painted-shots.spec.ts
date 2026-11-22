// Visual review of every painted improvement on a real WASM map, with its city and usual terrain.
// Runs in cloud Chromium against the built release app; outputs normal and strategic zoom crops for side-by-side review.
import type { Page } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { test, expect } from './fixtures';
import type { useGame } from '../src/store';
import type { Resource, TechId, Terrain } from '../src/types';
import { coreFixture } from '../src/test/coreFixture';
import { improvementCity } from '../src/test/improvementsFixture';
import { idxOf } from '../src/test/fixtureEngine';
import { ring } from '../src/hex';

const out = path.resolve('../docs/design/improvements-painted');
const cases: { name: string; resource: Resource; terrain: Terrain; upgraded?: boolean }[] = [
  { name: 'farm', resource: 'Wheat', terrain: 'Plains' },
  { name: 'irrigated-farm', resource: 'Wheat', terrain: 'Plains', upgraded: true },
  { name: 'pasture', resource: 'Cattle', terrain: 'Grassland' },
  { name: 'ranch', resource: 'Cattle', terrain: 'Grassland', upgraded: true },
  { name: 'quarry', resource: 'Stone', terrain: 'Hills' },
  { name: 'deep-quarry', resource: 'Stone', terrain: 'Hills', upgraded: true },
  { name: 'mine', resource: 'Copper', terrain: 'Hills' },
  { name: 'deep-mine', resource: 'Copper', terrain: 'Hills', upgraded: true },
  { name: 'saltpetre-works', resource: 'Niter', terrain: 'Plains' },
  { name: 'oil-well', resource: 'Oil', terrain: 'Plains' },
  { name: 'quartz-mine', resource: 'Silicon', terrain: 'Hills' },
];

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: { q: number; r: number }): { x: number; y: number } } };

function fixture(c: (typeof cases)[number]) {
  const state = improvementCity();
  const home = state.cities[0].pos;
  state.resource = state.terrain.map(() => null);
  state.improved = state.terrain.map(() => 0);
  state.units = [];
  const at = ring(home, 1)[3];
  const i = idxOf(state, at);
  state.terrain[i] = c.terrain;
  state.resource[i] = c.resource;
  state.improved[i] = 1;
  const techs: TechId[] = ['Education', 'Industrialization', 'Electricity'];
  if (c.upgraded) techs.push('CropRotation', 'IronWorking');
  state.players[0].techs.push(...techs);
  return { game: coreFixture(state), at };
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
  await page.evaluate(() => {
    const s = (window as unknown as MapWindow).legatum.getState();
    s.set({ techOpen: false });
    s.select(null);
  });
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function capture(page: Page, name: string, at: { q: number; r: number }, zoom: number) {
  await page.evaluate(({ at, zoom }) => {
    const w = window as unknown as MapWindow;
    w.legatum.getState().jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom });
  await page.waitForTimeout(750); // allow painted assets to decode and the map to repaint
  const clip = await page.evaluate((at) => {
    const w = window as unknown as MapWindow;
    const v = w.legatum.getState().views![w.legatum.getState().seat];
    const city = v.cities.find((c) => c.owner === v.me)!;
    const map = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    const tile = w.legatumMap.toScreen(at), town = w.legatumMap.toScreen(city.pos);
    const centerX = map.left + (tile.x + town.x) / 2;
    const centerY = map.top + (tile.y + town.y) / 2;
    const width = 560, height = 420;
    const x = Math.max(0, Math.min(window.innerWidth - width, centerX - width / 2));
    const y = Math.max(0, Math.min(window.innerHeight - height, centerY - height / 2));
    return { x, y, width, height, tile: { x: map.left + tile.x, y: map.top + tile.y }, town: { x: map.left + town.x, y: map.top + town.y } };
  }, at);
  expect(clip.tile.x).toBeGreaterThan(clip.x + 20);
  expect(clip.tile.x).toBeLessThan(clip.x + clip.width - 20);
  expect(clip.tile.y).toBeGreaterThan(clip.y + 20);
  expect(clip.tile.y).toBeLessThan(clip.y + clip.height - 20);
  expect(clip.town.x).toBeGreaterThan(clip.x + 20);
  expect(clip.town.x).toBeLessThan(clip.x + clip.width - 20);
  expect(clip.town.y).toBeGreaterThan(clip.y + 20);
  expect(clip.town.y).toBeLessThan(clip.y + clip.height - 20);
  await page.screenshot({ path: `${out}/${name}.png`, clip: { x: clip.x, y: clip.y, width: clip.width, height: clip.height }, animations: 'disabled' });
  await page.screenshot({ path: `${out}/${name}-detail.png`, clip: { x: clip.x + 80, y: clip.y + 90, width: 360, height: 240 }, animations: 'disabled' });
}

for (const upgraded of [false, true]) {
  test(`${upgraded ? 'upgraded' : 'base'} painted improvements beside their city at normal and far zoom`, async ({ browser, baseURL }) => {
    test.setTimeout(180_000);
    await mkdir(out, { recursive: true });
    for (const c of cases.filter((v) => !!v.upgraded === upgraded)) {
      const context = await browser.newContext({ baseURL, viewport: { width: 1000, height: 600 } });
      try {
        const page = await context.newPage();
        const { game, at } = fixture(c);
        await open(page, game);
        await capture(page, `${c.name}-normal`, at, 1.25);
        await capture(page, `${c.name}-far`, at, 0.46);
      } finally { await context.close(); }
    }
  });
}

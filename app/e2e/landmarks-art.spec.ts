// Review the three finished Great Works on real terrain, beside their cities, in Chromium.
// Run on the cloud box; LANDMARK_SHOTS names the directory returned with the screenshots.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { coreFixture } from '../src/test/coreFixture';
import { citiesFixture } from '../src/test/citiesFixture';
import { distance, hexAt } from '../src/hex';
import type { useGame } from '../src/store';
import type { BuildingKind, Hex } from '../src/types';

const output = path.resolve(process.env.LANDMARK_SHOTS ?? '../docs/design/landmarks');
const works: [BuildingKind, string][] = [
  ['GreatLibrary', 'great-library'],
  ['GrandBazaar', 'grand-bazaar'],
  ['RoyalArsenal', 'royal-arsenal'],
];
const fixture = citiesFixture();
fixture.turn = 12;
const owners = fixture.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id);

type MapWindow = Window & {
  legatum: typeof useGame;
  legatumMap: { camera: { zoom: number }; redraw(): void };
};

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

async function open(page: Page, work: number): Promise<Hex> {
  const state = citiesFixture();
  state.turn = 12;
  const [kind] = works[work];
  const city = state.cities.find((c) => c.id === owners[work].id)!;
  city.buildings.push(kind);
  state.players[0].greatWorkCompleted = kind;
  state.players[0].greatWorkHolder = 0;
  const tile = state.tileCity.map((owner, index) =>
    owner === city.id && state.terrain[index] !== 'Ocean' && distance(city.pos, hexAt(index, state.width)) > 0
      ? index : -1).filter((index) => index >= 0)
    .sort((a, b) => distance(city.pos, hexAt(b, state.width)) - distance(city.pos, hexAt(a, state.width)))[0] ?? -1;
  if (tile < 0) throw new Error(`No landmark tile for ${city.name}`);
  state.landmark = Array(state.terrain.length).fill(null);
  state.landmark[tile] = { building: kind, city: city.id, of: city.owner };
  const site = hexAt(tile, state.width);
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(state) });
  await page.goto('/?client=wasm');
  const loadError = await page.evaluate(async () => {
    const { createClient } = await import('../src/client');
    const save = JSON.parse(localStorage.getItem('legatum.autosave.v1')!);
    try { (await createClient('wasm')).load(save.game); return null; }
    catch (error) { return String(error); }
  });
  expect(loadError, 'landmark fixture must load in the current engine').toBeNull();
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  return site;
}

test.use({ viewport: { width: 1440, height: 900 }, browserName: 'chromium' });

for (const [i, [kind, name]] of works.entries()) {
  test(`${name} beside its city at normal and far zoom`, async ({ page }) => {
    const site = await open(page, i);
    const city = owners[i];
    const landmark = await page.evaluate(async ({ id, kind }) => {
      const w = window as unknown as MapWindow;
      const view = w.legatum.getState().views![0];
      const { landmarksOf } = await import('../src/map/landmarks');
      return landmarksOf(view).find((l) => l.city === id && l.building === kind)?.hex ?? null;
    }, { id: city.id, kind });
    expect(landmark, `${name} should use its chosen wonder tile`).toEqual(site);
    await mkdir(output, { recursive: true });
    for (const [tier, zoom] of [['normal', 1.15], ['far', 0.45]] as const) {
      await page.evaluate(({ pos, zoom }) => {
        const w = window as unknown as MapWindow;
        const s = w.legatum.getState();
        s.select(null);
        s.jumpTo(pos);
        w.legatumMap.camera.zoom = zoom;
        w.legatumMap.redraw();
      }, { pos: city.pos, zoom });
      await page.waitForTimeout(800);
      await page.screenshot({ path: path.join(output, `${name}-${tier}.png`), clip: { x: 355, y: 150, width: 730, height: 570 }, animations: 'disabled' });
    }
  });
}

// Chromium review captures for the three late resource medallions and their improvement art.
// RESOURCE_ART_SHOTS=../docs/design/resource-art/era-resources npx playwright test e2e/resource-art-review.spec.ts --project=game
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { coreFixture } from '../src/test/coreFixture';
import { improvementCity } from '../src/test/improvementsFixture';
import { eraTechs, RULES } from '../src/data';
import { ring } from '../src/hex';
import { idxOf, refreshVision } from '../src/test/fixtureEngine';
import type { Hex, Resource, Terrain } from '../src/types';
import type { useGame } from '../src/store';

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

const out = path.resolve(process.env.RESOURCE_ART_SHOTS ?? '../docs/design/resource-art/era-resources');
const s = improvementCity({ improved: false });
const city = s.cities[0];
s.units = []; // Keep the review tiles clear of the fixture's nearby unit stack.
const futureResource = RULES.supply.eraResources.at(-1) as Resource;
const sites: { name: string; resource: Resource; terrain: Terrain; hex: Hex }[] = [
  { name: 'niter', resource: 'Niter', terrain: 'Plains', hex: ring(city.pos, 2)[0] },
  { name: 'oil', resource: 'Oil', terrain: 'Plains', hex: ring(city.pos, 2)[4] },
  { name: 'silicon', resource: futureResource, terrain: 'Hills', hex: ring(city.pos, 2)[9] },
];
for (const site of sites) {
  const i = idxOf(s, site.hex);
  s.terrain[i] = site.terrain;
  s.resource![i] = site.resource;
  s.improved![i] = 1;
}
s.players[0].techs.push(...[0, 1, 2, 3, 4, 5].flatMap(eraTechs));
refreshVision(s);
const game = coreFixture(s);

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };

async function frame(page: Page, hex: Hex, zoom: number) {
  await page.evaluate(({ hex, zoom }) => {
    const w = window as unknown as MapWindow;
    const st = w.legatum.getState();
    st.set({ techOpen: false });
    st.select(null);
    st.jumpTo(hex);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { hex, zoom });
  await page.waitForTimeout(700);
}

async function capture(page: Page, hex: Hex, name: string, span: number) {
  const at = await page.evaluate((hex) => {
    const w = window as unknown as MapWindow;
    const box = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    const p = w.legatumMap.toScreen(hex);
    return { x: box.left + p.x, y: box.top + p.y };
  }, hex);
  const clip = { x: Math.max(0, at.x - span / 2), y: Math.max(0, at.y - span / 2), width: span, height: span };
  await mkdir(out, { recursive: true });
  await page.screenshot({ path: `${out}/${name}.png`, clip, animations: 'disabled' });
  expect(at.x).toBeGreaterThan(0);
  expect(at.y).toBeGreaterThan(0);
}

test.use({ viewport: { width: 1440, height: 900 }, browserName: 'chromium' });
test('late resource medallions and their engraved improvements on the map', async ({ page }) => {
  test.skip(!process.env.RESOURCE_ART_SHOTS, 'opt-in art review capture');
  test.setTimeout(120_000);
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  for (const site of sites) {
    await frame(page, site.hex, 1.0);
    await capture(page, site.hex, `${site.name}-medallion-map`, 180);
    await frame(page, site.hex, 1.8);
    await capture(page, site.hex, `${site.name}-improvement-map`, 240);
  }
});

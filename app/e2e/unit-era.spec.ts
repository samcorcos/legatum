// Era-distinct units (user, 2026-10-11: the Hoplite and the Phalangite "looked nearly identical"): your
// Spartans against Athens (src/test/unitEraFixture.ts), every era of each class, at normal, close and far
// zoom, 1000x600 and 1440x900, with nothing selected (era plaques only), with the Hoplite selected (strength
// chips on the enemies it could attack this turn) and the attack forecast on the Phalangite beside it.
// UNIT_ERA_SHOTS=<dir> picks where the shots go. It checks the forecast headline and that the chips stay
// off units out of reach; the rest is shots for review.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { coreFixture } from '../src/test/coreFixture';
import { unitEraFixture } from '../src/test/unitEraFixture';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const out = process.env.UNIT_ERA_SHOTS ? path.resolve(process.env.UNIT_ERA_SHOTS) : path.resolve('test-results/unit-era');

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

const setup = unitEraFixture();

async function open(page: Page) {
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 60, names: ['Leonidas', 'Pericles'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(setup.fixture) });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('60');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function frame(page: Page, at: Hex, zoom: number, selectHoplite = true) {
  await page.evaluate(({ at, zoom, id }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(id === null ? null : { kind: 'unit', id });
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom, id: selectHoplite ? setup.hoplite : null });
  await expect.poll(() => page.evaluate((at) => {
    const m = (window as unknown as MapWindow).legatumMap;
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    const p = m.toScreen(at);
    return Math.hypot(p.x - b.width / 2, p.y - b.height / 2) < 6;
  }, at), { timeout: 5000 }).toBe(true).catch(() => undefined);
  const park = await page.getByText('This Turn', { exact: true }).boundingBox();
  if (park) await page.mouse.move(park.x + park.width / 2, park.y + park.height / 2);
  await page.waitForTimeout(600);
}

async function shot(page: Page, size: string, name: string) {
  await mkdir(`${out}/${size}`, { recursive: true });
  await page.screenshot({ path: `${out}/${size}/${name}.png`, animations: 'disabled' });
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    test('era plaques, strength chips and the forecast headline', async ({ page }) => {
      test.setTimeout(180_000);
      const size = `${width}x${height}`;
      await open(page);
      await frame(page, setup.centre, 1.0, false); await shot(page, size, '01-plaques-normal');
      await frame(page, setup.centre, 1.7, false); await shot(page, size, '02-plaques-close');
      await frame(page, setup.centre, 1.0); await shot(page, size, '03-selected-normal');
      await frame(page, setup.centre, 1.7); await shot(page, size, '04-selected-close');
      await frame(page, setup.centre, 0.55); await shot(page, size, '05-selected-far');
      await frame(page, setup.centre, 1.0);
      const p = await page.evaluate((h) => (window as unknown as MapWindow).legatumMap.toScreen(h), setup.phalangiteAt);
      const b = await page.locator('[data-game-map]').boundingBox();
      await expect.poll(async () => {
        await page.mouse.move(b!.x + p.x + 3, b!.y + p.y + 3);
        await page.mouse.move(b!.x + p.x, b!.y + p.y, { steps: 3 });
        return page.locator('.forecast').isVisible();
      }, { intervals: [500, 1000, 1000, 2000] }).toBe(true);
      await expect(page.getByTestId('forecast-era')).toHaveText(/^Phalangite, one era newer: 25 vs your 22\. It outclasses you\.$/);
      await page.waitForTimeout(400);
      await shot(page, size, '06-forecast');
    });
  });
}

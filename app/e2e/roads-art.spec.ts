// Road art review: the Courier Hall's roads on a generated map (src/test/roadsFixture.ts), at normal
// and zoomed-out zoom. ROADS_SHOTS=<dir> picks where the shots go. Shots only.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { roadsFixture } from '../src/test/roadsFixture';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void }; };
const out = process.env.ROADS_SHOTS ? path.resolve(process.env.ROADS_SHOTS) : path.resolve('test-results/roads-art');
const [W, H] = (process.env.ROADS_VIEWPORT ?? '390x760').split('x').map(Number);

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

async function open(page: Page, f: ReturnType<typeof roadsFixture>) {
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 40, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(f.fixture) });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('40');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function frame(page: Page, at: { q: number; r: number }, zoom: number) {
  await page.evaluate(({ at, zoom }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(null);
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom });
  await page.waitForTimeout(700);
}

const arrangements = [
  { name: 'duel-small', far: 0.3, opts: { players: 2, width: 40, height: 26, seed: '12', extra: 8 } },
  { name: 'four-standard', far: 0.26, opts: { players: 4, width: 48, height: 32, seed: '31337', extra: 16 } },
  { name: 'eight-large', far: 0.2, opts: { players: 8, width: 60, height: 40, seed: '777', extra: 30 } },
];

test.use({ viewport: { width: W, height: H } });
for (const a of arrangements) {
  test(`roads ${a.name}`, async ({ page }) => {
    test.setTimeout(180_000);
    const f = roadsFixture(a.opts);
    await open(page, f);
    await mkdir(out, { recursive: true });
    for (const [tag, zoom] of [['near', 0.8], ['far', a.far]] as const) {
      await frame(page, tag === 'near' ? f.hallPos : f.center, zoom);
      await page.screenshot({ path: `${out}/${a.name}-${tag}.png`, animations: 'disabled' });
    }
  });
}

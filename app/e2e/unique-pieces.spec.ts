// Unique pieces must differ (user, 2026-10-17): in the real app, against the real core.
//   - Sparta: two Hoplites side by side. With one selected, the attack forecast lists the phalanx as its own line in "Your strength".
// At 1000x600 and 1440x900. UNIQUE_PIECE_SHOTS=<dir> picks where the shots go.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { coreFixture } from '../src/test/coreFixture';
import { phalanxFixture } from '../src/test/uniquePiecesFixture';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const out = process.env.UNIQUE_PIECE_SHOTS ? path.resolve(process.env.UNIQUE_PIECE_SHOTS) : path.resolve('test-results/unique-pieces');

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

async function open(page: Page, game: string, turn: number, names: [string, string]) {
  await page.addInitScript(({ game, turn, names }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names, savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn, names });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function shot(page: Page, size: string, name: string) {
  await mkdir(`${out}/${size}`, { recursive: true });
  await page.screenshot({ path: `${out}/${size}/${name}.png`, animations: 'disabled' });
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });

    test('the forecast lists the Hoplite phalanx as its own line', async ({ page }) => {
      test.setTimeout(120_000);
      const size = `${width}x${height}`;
      const setup = phalanxFixture();
      await open(page, coreFixture(setup.fixture), 40, ['Leonidas', 'Pericles']);
      await page.evaluate(({ at, id }) => {
        const w = window as unknown as MapWindow;
        const s = w.legatum.getState();
        s.set({ techOpen: false });
        s.select({ kind: 'unit', id });
        s.jumpTo(at);
        w.legatumMap.camera.zoom = 1.3;
        w.legatumMap.redraw();
      }, { at: setup.centre, id: setup.hoplite });
      await page.waitForTimeout(600);
      const p = await page.evaluate((h) => (window as unknown as MapWindow).legatumMap.toScreen(h), setup.enemyAt);
      const b = await page.locator('[data-game-map]').boundingBox();
      await expect.poll(async () => {
        await page.mouse.move(b!.x + p.x + 3, b!.y + p.y + 3);
        await page.mouse.move(b!.x + p.x, b!.y + p.y, { steps: 3 });
        return page.locator('.forecast').isVisible();
      }, { intervals: [500, 1000, 1000, 2000] }).toBe(true);
      const card = page.locator('.forecast');
      await expect(card).toContainText('Phalanx: next to another Hoplite');
      await expect(card.locator('.forecast-bd')).toContainText('+5');
      // The line belongs to the attacker only: the Athenian Spearman has no Hoplite beside it.
      const theirs = await card.locator('.forecast-bd > *').nth(1).innerText();
      expect(theirs).not.toContain('Phalanx');
      // The card stays inside the window.
      const box = (await card.boundingBox())!;
      expect(box.y).toBeGreaterThanOrEqual(0);
      expect(box.y + box.height).toBeLessThanOrEqual(height + 0.5);
      await page.waitForTimeout(300);
      await shot(page, size, '01-phalanx-forecast');
    });

  });
}

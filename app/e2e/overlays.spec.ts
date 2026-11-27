import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';
import { lensRadio, overlayBox, showMapOptions } from './overlayHelpers';

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

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test.describe(`${size.width}x${size.height}`, () => {
    test.use({ viewport: size });
    test('Yields stacks over Default and Settle and survives reload', async ({ page }) => {
      const game = coreFixture(mapArtFixture().fixture);
      await page.addInitScript((game) => {
        localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Player'], savedAt: new Date().toISOString(), game }));
        localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
      }, game);
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Continue/ }).click();
      await expect(page.getByTestId('turn-number')).toHaveText('12');
      for (let i = 0; i < 4 && await page.locator('.moment-card').first().isVisible(); i++) await page.keyboard.press('Escape');
      await showMapOptions(page);
      await expect(page.locator('.lens-bar button')).toHaveText(['Default', 'Settle', 'YieldsY']);
      await expect(overlayBox(page, 'Yields')).not.toBeChecked();
      await overlayBox(page, 'Yields').click();
      await expect(page.locator('.lens-yield-line')).toContainText('One icon per point');
      for (const lens of ['Settle', 'Default']) {
        await showMapOptions(page);
        await lensRadio(page, lens).click();
        await showMapOptions(page);
        await expect(overlayBox(page, 'Yields')).toBeChecked();
      }
      await page.reload();
      await page.getByRole('button', { name: /^Continue/ }).click();
      await showMapOptions(page);
      await expect(overlayBox(page, 'Yields')).toBeChecked();
      expect(await page.evaluate(() => JSON.parse(localStorage.getItem('legatum.map-overlays.v1')!))).toEqual({ yields: true });
      await overlayBox(page, 'Yields').click();
      await expect(overlayBox(page, 'Yields')).not.toBeChecked();
    });
  });
}

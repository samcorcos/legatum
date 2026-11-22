import { endPlanning } from './turnActions';
import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';

// Other suites serve the production build. Feedback diagnostics deliberately exist only in dev.
const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0, strictPort: false } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('Vite did not bind a port');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

// Phone width (390) returns with the portrait layout (plans/mobile.md increment 2).
for (const viewport of [{ width: 1024, height: 1366 }]) {
  test.describe(`feel at ${viewport.width}px`, () => {
    test.use({ viewport, hasTouch: true });
    test('logs one queue and ready from direct research intent', async ({ page }) => {
      const errors: string[] = [];
      page.on('pageerror', (error) => errors.push(error.message));
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Single Player/ }).click();
      await page.getByRole('button', { name: /^Start game/ }).click();
      await expect(page.getByTestId('turn-number')).toHaveText('1');
      await page.getByRole('button', { name: /^Research/ }).click();
      await page.evaluate(() => { window.__feelLog = []; });
      await page.locator('.era-card[data-tech="Irrigation"]').getByTestId('research-now').tap();
      await expect.poll(() => page.evaluate(() => window.__feelLog?.map((e) => e.name))).toEqual(['queue']);
      await page.getByRole('button', { name: 'Close research', exact: true }).tap();
      await expect.poll(() => page.evaluate(() => performance.now() - window.__feelLog!.at(-1)!.at)).toBeGreaterThanOrEqual(80);
      // On a touch screen the desktop "Ready anyway" link is replaced by the touch turn controls
      // (touch.css hides it); their End Turn runs the same readyAndPass handler.
      await endPlanning(page);
      await expect(page.getByTestId('turn-number')).toHaveText('2');
      const calls = await page.evaluate(() => window.__feelLog!.map((e) => e.name));
      for (const name of ['queue', 'ready']) expect(calls.filter((call) => call === name)).toHaveLength(1);
      expect(errors).toEqual([]);
    });
  });
}

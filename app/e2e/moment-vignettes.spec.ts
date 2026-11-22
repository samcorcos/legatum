import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';

const shots = path.resolve('../.progress/art/moment-vignettes');
const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0, strictPort: false } });
    await server.listen();
    try {
      await server.warmupRequest('/src/main.tsx');
      await server.warmupRequest('/src/test/celebrationFixture.ts');
      await server.waitForRequestsIdle();
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('No Vite address');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    for (const [kind, image] of [
      ['founded', 'founded'], ['ruin-gold', 'ruin-explored'],
      ['greatwork', 'greatwork'], ['captured', 'captured'], ['lost', 'lost'],
      ['tech-partner', 'tech'],
    ] as const) {
      test(`${kind} shows its painting, emblem, title and actions`, async ({ page }) => {
        await page.addInitScript(() => {
          if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
        });
        await page.goto('/?client=wasm');
        await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
        await page.evaluate(async (reward) => {
          const momentsFile = '/src/celebrate/moments.ts';
          const { DURATION } = await import(/* @vite-ignore */ momentsFile);
          DURATION.notable = DURATION.major = DURATION.legendary = 120_000;
          const fixtureFile = '/src/test/celebrationFixture.ts';
          const { rewardFixture } = await import(/* @vite-ignore */ fixtureFile);
          await rewardFixture(reward);
        }, kind);
        const card = page.locator('.moment-card');
        await expect(card).toBeVisible();
        await expect(card.locator('.moment-vignette')).toHaveAttribute('src', new RegExp(`moment-${image}-104\\.png`));
        await expect(card.locator('[data-emblem]')).toBeVisible();
        await expect(card.locator('h2')).toBeVisible();
        await expect(card.getByRole('button', { name: 'Show on map' })).toBeVisible();
        await expect(card.getByRole('button', { name: 'Turn report' })).toBeVisible();
        await mkdir(shots, { recursive: true });
        await page.screenshot({ path: path.join(shots, `${kind}-${width}x${height}.png`) });
      });
    }
  });
}

// Faction sheet review (art pass): every faction's sheet at 1000x600 and 1440x900, shot from the
// real picker. FACTION_SHOTS=<dir> picks where the shots go, FACTION_SHOTS_TAG the file prefix,
// and FACTION_SHOTS_FACTIONS selects a comma-separated subset for focused reviews.
// Shots only; the layout spec asserts the fit.
import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { FACTIONS, FACTION_INFO } from '../src/data';

const out = process.env.FACTION_SHOTS ? path.resolve(process.env.FACTION_SHOTS) : path.resolve('test-results/faction-sheet');
const tag = process.env.FACTION_SHOTS_TAG ?? 'shot';
const shotFactions = process.env.FACTION_SHOTS_FACTIONS?.split(',') ?? ['Random', ...FACTIONS];

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
  test(`faction sheet shots at ${size.width}x${size.height}`, async ({ page }) => {
    test.skip(!process.env.FACTION_SHOTS, 'set FACTION_SHOTS=<dir> to write the review shots');
    test.setTimeout(120_000);
    await mkdir(out, { recursive: true });
    await page.setViewportSize(size);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    await page.evaluate(() => document.fonts.ready.then(() => undefined));
    if (size.width === 1000) {
      const gallery = stage.locator('.fs-gallery');
      const galleryOverflow = await gallery.evaluate((el) => el.scrollHeight - el.clientHeight);
      console.log(`gallery overflow ${size.width}: ${galleryOverflow}`);
      expect(galleryOverflow).toBeLessThanOrEqual(0);
    }
    for (const faction of ['Random', ...FACTIONS] as const) {
      if (!shotFactions.includes(faction)) continue;
      const name = faction === 'Random' ? 'Random' : FACTION_INFO[faction].name;
      // A click chooses the faction and shows its sheet; the dialog stays open.
      await page.getByRole('option', { name }).click();
      await expect(stage.getByTestId('faction-sheet').locator('.fs-sheet-name')).toContainText(name);
      if (faction === 'India') {
        const pairs = stage.getByTestId('faction-sheet').locator('.fs-pairs');
        await expect(pairs).toContainText('Cities can be founded 3 tiles apart, closer than anyone else.');
        await expect(pairs).toContainText('A city gains 7% Food and Production for every other Indian city within 5 tiles.');
      }
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(500);
      const sheet = stage.getByTestId('faction-sheet');
      const over = await sheet.evaluate((el) => el.scrollHeight - el.clientHeight);
      const room = await sheet.evaluate((el) => {
        const facts = el.querySelector('.fs-facts')?.getBoundingClientRect(), box = el.getBoundingClientRect();
        return facts ? `${Math.round(box.bottom - facts.bottom)}` : '-';
      });
      console.log(`overflow ${size.width} ${faction}: ${over} (room under the facts ${room})`);
      await page.screenshot({ path: path.join(out, `${tag}-${faction.toLowerCase()}-${size.width}.png`) });
    }
  });
}

test('India faction sheet explains dense founding and the nearby-city bonus', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
  const stage = page.locator('dialog[data-testid="faction-stage"][open]');
  await expect(stage).toBeVisible();
  await page.getByRole('option', { name: FACTION_INFO.India.name }).click();
  const pairs = stage.getByTestId('faction-sheet').locator('.fs-pairs');
  await expect(pairs).toContainText('Cities can be founded 3 tiles apart, closer than anyone else.');
  await expect(pairs).toContainText('A city gains 7% Food and Production for every other Indian city within 5 tiles.');
});

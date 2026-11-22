// Text-artifact review shots (user, 2026-10-11: ",." after the Celts' piece cost): the piece card of
// several peoples and one whole sheet, at 1000x600 and 1440x900, from the real picker. Asserts that the
// card text carries no stray punctuation. The data-level guard is src/ui/textArtifacts.test.tsx; the
// screen tour asserts the same on every screen it visits.
import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { FACTION_INFO } from '../src/data';
import { auditBlocks } from '../src/test/textArtifacts';
import { visibleTextBlocks } from './layoutAudit';

const out = path.resolve('test-results/text-artifacts');

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
  test(`piece cards read cleanly at ${size.width}x${size.height}`, async ({ page }) => {
    test.setTimeout(120_000);
    await mkdir(out, { recursive: true });
    await page.setViewportSize(size);
    await page.goto('/');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    await page.evaluate(() => document.fonts.ready.then(() => undefined));
    for (const faction of ['Celts', 'Mongols', 'Switzerland', 'Byzantium', 'Sparta', 'Athens', 'Germany', 'India'] as const) {
      const name = FACTION_INFO[faction].name;
      // A click chooses the faction and shows its sheet; the dialog stays open.
      await page.getByRole('option', { name }).click();
      await expect(stage.getByTestId('faction-sheet').locator('.fs-sheet-name')).toContainText(name);
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(400);
      const card = stage.getByTestId('faction-sheet').locator('.pc');
      await expect(card.first()).toBeVisible();
      await card.first().screenshot({ path: path.join(out, `piece-${faction.toLowerCase()}-${size.width}.png`) });
      if (faction === 'Celts') await page.screenshot({ path: path.join(out, `sheet-celts-${size.width}.png`) });
      const artifacts = auditBlocks(`${faction} sheet`, await visibleTextBlocks(page));
      expect(artifacts, artifacts.join('\n')).toEqual([]);
      // A special unit or building card carries rule sentences with complete punctuation.
      const text = await card.locator('.pc-text').allInnerTexts();
      for (const t of text) expect(t).toMatch(/^[A-Z0-9+][^]*[.)]$/);
    }
  });
}

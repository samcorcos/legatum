// Switzerland's tolls and interest in plain words (user, 2026-10-17: "'per complete pair' is not normal
// language"; interest has no cap): the faction sheet from the real picker, the Guide, and the Gold
// breakdown's interest line. Set SWISS_SHOTS to a folder to keep a screenshot of each at both laptop sizes.
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import type { useGame } from '../src/store';

type GameWindow = Window & { legatum: typeof useGame };

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shots = process.env.SWISS_SHOTS;
const SENTENCE = 'Each city earns +1 Gold for every 2 population. Every 100 Gold in your treasury earns +1 Gold a turn in interest.';

async function chooseSwitzerland(page: Page) {
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
  const stage = page.locator('dialog[data-testid="faction-stage"][open]');
  await expect(stage).toBeVisible();
  await page.getByRole('option', { name: 'Switzerland' }).click();
  return stage;
}

for (const size of SIZES) {
  test.describe(`Switzerland's income at ${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    test('the faction sheet says tolls and interest in plain words, with no maximum, and fits whole', async ({ page }) => {
      const stage = await chooseSwitzerland(page);
      const sheet = stage.getByTestId('faction-sheet');
      await expect(sheet.locator('.fs-sheet-name')).toContainText('Switzerland');
      await expect(sheet).toContainText(SENTENCE);
      await expect(sheet).not.toContainText(/complete pair|up to \+5|max 5|Gold held adds/i);
      expect(await sheet.evaluate((el) => el.scrollHeight - el.clientHeight)).toBeLessThanOrEqual(1);
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(500);
      if (shots) await page.screenshot({ path: join(shots, `swiss-sheet-${size.width}.png`) });
    });

    test('the Gold breakdown lists interest as its own line', async ({ page }) => {
      const stage = await chooseSwitzerland(page);
      await page.keyboard.press('Enter');
      await expect(stage).toBeHidden();
      await page.getByLabel('Map seed', { exact: true }).fill('424242');
      await page.getByRole('button', { name: /^Start game/ }).click();
      await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
      await expect(page.locator('.loading-screen')).toHaveCount(0);
      // Stage a hoard of 1,400 Gold on the live view (render only): 14 Gold a turn, the same line the core emits.
      await page.evaluate(() => {
        const store = (window as unknown as GameWindow).legatum, s = store.getState();
        const v = structuredClone(s.views![s.seat]);
        v.economy.gold = 1_400;
        v.economy.gold_per_turn = { total: v.economy.gold_per_turn.total + 14, parts: [...v.economy.gold_per_turn.parts.filter(([label]) => label !== 'Switzerland: interest'), ['Switzerland: interest', 14]] };
        store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) });
      });
      await page.locator('.gold-stat').click();
      const tip = page.locator('.tip-bubble').filter({ hasText: 'Switzerland: interest' }).first();
      await expect(tip).toBeVisible();
      await expect(tip).not.toContainText(/treasury interest/i);
      const row = tip.locator('tr, .bd-row, li, div').filter({ hasText: /^Switzerland: interest/ }).first();
      await expect(row).toContainText('14');
      await page.waitForTimeout(450);
      if (shots) await page.screenshot({ path: join(shots, `swiss-gold-breakdown-${size.width}.png`) });
    });
  });
}

test('the Guide uses the same words', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^How to Play/ }).click();
  const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
  await expect(guide).toBeVisible();
  const chapters = guide.getByRole('navigation', { name: 'Guide chapters' }).getByRole('button');
  for (const label of ['Cities & research', 'The peoples']) {
    await chapters.filter({ hasText: label }).click();
    await expect(guide).toContainText(SENTENCE);
    await expect(guide).not.toContainText(/up to 5 Gold interest|one per 100 Gold held|complete pair|max 5/);
  }
});

// The race fixture exercises the score ledger's ownership, empty and hold states at both review sizes.
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { progressFixture } from '../src/test/progressFixture';
import { stageHold, stageHoldings } from './vpStage';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => { localStorage.setItem('legatum.dev.no-era-hud', '1'); });
});

async function twoEmpires(page: Page) {
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(progressFixture().fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
}

async function eightEmpires(page: Page) {
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  // A new lobby has 6 empires; this table is the full 8, so add the last two.
  for (let i = 0; i < 2; i++) await page.getByRole('button', { name: /^\+ Add an AI empire/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

async function openLedger(page: Page) {
  await page.locator('button.vh-chip').click();
  const sheet = page.getByTestId('objective-sheet');
  await expect(sheet.locator('.sl-table')).toBeVisible();
  return sheet;
}

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  for (const scenario of [{ name: '2 empires', start: twoEmpires, rows: 2 }, { name: '8 empires', start: eightEmpires, rows: 8 }]) {
    test.describe(`ledger holdings, ${scenario.name} at ${size.width}x${size.height}`, () => {
      test.use({ viewport: size });

      test('each way to score shows its holder, empty socket and your goal progress', async ({ page }) => {
        await scenario.start(page);
        await stageHoldings(page);
        const sheet = await openLedger(page);
        await expect(sheet.locator('.sl-col')).toHaveCount(scenario.rows);
        await expect(sheet.locator('.sl-col.mine')).toHaveCount(1);
        const work = sheet.locator('.sl-row[data-row="GreatWork"]');
        await expect(work.locator('td.mine .sl-token')).toHaveCount(1);
        await expect(work.locator('td:not(.mine) .sl-socket')).toHaveCount(scenario.rows - 1);
        await expect(sheet.locator('.sl-row[data-row="goal"] td.mine [role="progressbar"], .sl-row[data-row="held-goal"] td.mine [role="progressbar"]')).toBeVisible();
        await expect(sheet.locator('.sl-row.k-era .sl-token').first()).toBeVisible();
        await expect(sheet.locator('.sl-row.k-era .sl-socket').first()).toBeVisible();
        await expect(sheet).not.toContainText(/fastest|recommend|suggest/i);
      });

      test('a team at its target shows both countdowns under the right VP totals', async ({ page }) => {
        await scenario.start(page);
        await stageHoldings(page);
        await stageHold(page, { mine: true });
        const chip = page.locator('button.vh-chip');
        await expect(chip).toContainText('Score ledger');
        await expect(page.locator('.race-headline')).toContainText(/win(s)? in 1/);
        const sheet = await openLedger(page);
        await expect(sheet.locator('.sl-col.mine .sl-hold')).toContainText('win in 1');
        await expect(sheet.locator('.sl-col:not(.mine) .sl-hold')).toContainText('Wins in 3');
        await expect(sheet.locator('.sl-col.mine .sl-col-vp')).toContainText('/10');
        await expect(sheet.locator('.sl-col:not(.mine) .sl-col-vp').first()).toContainText('/10');
        await page.keyboard.press('Escape');
        await expect(sheet).toHaveCount(0);
      });
    });
  }
}

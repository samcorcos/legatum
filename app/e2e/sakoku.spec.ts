import { expectRaceWhole } from './tour/helpers';
import { revealSidebar } from './sidebarAssertions';
// The reserved Japan Sakoku rule in a legacy scenario: its action row and confirm, a closed country's public count in a rival's
// race row, and the map's closed-country border. Checked at the user's laptop
// (1000x600) and 1440x900. Screenshots land in test-results/sakoku/<size>/.
import { mkdir } from 'node:fs/promises';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import type { useGame } from '../src/store';
import { sakokuCoreSave, sakokuRivalCoreSave } from '../src/test/goalActionsFixture';
import { endPlanning } from './turnActions';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }];

async function load(page: Page, game: string, turn: number) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

async function shot(page: Page, size: { name: string }, name: string) {
  await page.mouse.move(5, 300);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(400);
  const dir = `test-results/sakoku/${size.name}/`;
  await mkdir(dir, { recursive: true });
  await page.screenshot({ path: `${dir}${name}.png`, animations: 'disabled' });
}

for (const size of SIZES) {
  test.describe(`Sakoku at ${size.name}`, () => {
    test.use({ viewport: { width: size.width, height: size.height } });

    test('an empire with the legacy Sakoku rule closes the country, sees its count, and the row stays whole', async ({ page }) => {
      test.setTimeout(120_000);
      const state = () => page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return { view: s.views![s.seat], orders: s.orders[s.seat] }; });
      await load(page, sakokuCoreSave(), 60);
      await revealSidebar(page, 'goal');
      const row = page.locator('[data-goal-action="Sakoku"]');
      await expect(row).toBeVisible();
      await expectRaceWhole(page);
      await expect(row.getByRole('button', { name: /^Close the country/ })).toBeVisible();
      await shot(page, size, '1-open');
      await row.getByRole('button', { name: /^Close the country/ }).click();
      const dialog = page.locator('dialog[data-goal-dialog="Sakoku"][open]');
      await expect(dialog).toBeVisible();
      // The confirm fits the window with its primary visible, and names every block.
      const box = (await dialog.boundingBox())!;
      expect(box.y).toBeGreaterThanOrEqual(0);
      expect(box.y + box.height).toBeLessThanOrEqual(size.height + 1);
      await expect(dialog.getByRole('button', { name: 'Close the country' })).toBeInViewport();
      await expect(dialog.locator('[data-testid="sakoku-blocks"] tbody th')).toHaveText(['War', 'Alliances', 'Tech diffusion', 'Foreign units']);
      await shot(page, size, '2-confirm');
      await page.keyboard.press('Enter');
      await expect(dialog).toHaveCount(0);
      await expect(row).toContainText('Ordered');
      expect((await state()).orders).toEqual([{ type: 'Action', action: { action: 'Sakoku', close: true } }]);
      await shot(page, size, '3-ordered');
      await endPlanning(page);
      await expect(page.getByTestId('turn-number')).toHaveText('61');
      await expect(page.locator('[data-testid="closed-chip"]').first()).toHaveText('1/30');
      await expect(row.getByRole('button', { name: /^Reopen the country/ })).toBeVisible();
      await expectRaceWhole(page);
      // The map draws the closed border from the public count.
      await page.evaluate(() => { const g = (window as unknown as GameWindow).legatum.getState(); g.select(null); });
      await shot(page, size, '4-closed');
    });

    test('a rival unit inside the closed empire does not break the held vow: the count and palisade', async ({ page }) => {
      test.setTimeout(120_000);
      await load(page, sakokuRivalCoreSave(true), 100);
      await expect(page.locator('[data-testid="closed-chip"]').first()).toHaveText('holding');
      await expectRaceWhole(page);
      await page.evaluate(() => {
        const g = (window as unknown as GameWindow).legatum.getState(), v = g.views![g.seat];
        // Our Soldier standing inside India's land: the eastmost unit we have.
        const inside = v.units.filter((u) => u.owner === v.me).sort((a, b) => b.pos.q - a.pos.q)[0];
        g.select(null);
        g.jumpTo(inside.pos);
      });
      await shot(page, size, '5-rival-unit-inside');
    });

    test('a rival sees a held vow: the chip says holding', async ({ page }) => {
      test.setTimeout(120_000);
      await load(page, sakokuRivalCoreSave(false), 100);
      await expect(page.locator('[data-testid="closed-chip"]').first()).toHaveText('holding');
      await expectRaceWhole(page);
      await shot(page, size, '6-rival-holding');
    });
  });
}

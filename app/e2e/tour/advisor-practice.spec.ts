// The advisor once the routine steps are known (moves, Fortify, Skip, Auto-explore, End turn, Close Research):
// Settings shows "Show every step again", and in a game the card still shows builds and research but stays quiet about routine steps.
import { test, expect } from '../fixtures';
import { SIZES, shot, type GameWindow } from './helpers';
import { setMapSeed } from '../mapSeed';

const KNOWN = { enabled: true, seen: ['welcome', 'report'], practice: { move: 3, fortify: 3, skip: 3, explore: 3, endTurn: 3, closeResearch: 3 } };

test.describe.configure({ mode: 'parallel' });
for (const size of SIZES) test.describe(`advisor practice ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('settings with and without practice', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.evaluate(() => localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: true, seen: [] })));
    await page.reload();
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await expect(page.getByRole('dialog', { name: 'Settings', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Show every step again' })).toHaveCount(0);
    await shot(page, size, '01-settings-no-practice');
    await page.evaluate((k) => localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify(k)), KNOWN);
    await page.reload();
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const again = page.getByRole('button', { name: 'Show every step again' });
    await expect(again).toBeVisible();
    await shot(page, size, '02-settings-practice');
    await again.hover();
    await shot(page, size, '03-settings-practice-hover');
    await again.focus();
    await page.keyboard.press('Tab');
    await page.keyboard.press('Shift+Tab');
    await shot(page, size, '04-settings-practice-focus');
    await again.click();
    await expect(again).toHaveCount(0);
    await expect(page.getByText('Every step will show again.')).toBeVisible();
    await shot(page, size, '05-settings-after-reset');
  });

  test('a turn with every routine known', async ({ page }) => {
    test.setTimeout(120_000);
    await page.addInitScript((k) => { if (!sessionStorage.getItem('seeded')) { localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify(k)); sessionStorage.setItem('seeded', '1'); } }, KNOWN);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await setMapSeed(page, '424242');
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await stage.getByRole('option', { name: /Athens/ }).click();
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    const card = page.getByTestId('advisor');
    await expect(card).toHaveAttribute('data-advisor-id', /^city:/);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '10-build-still-shown');
    const cityId = Number((await card.getAttribute('data-advisor-id'))!.split(':')[1]);
    await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), cityId);
    await expect(card).toContainText('Step 2 of 2');
    const name = /Build on (.+)\./.exec((await card.locator('.advisor-step strong').textContent())!)?.[1];
    await page.locator('.city-panel').getByRole('button', { name: `Build ${name}`, exact: true }).click();
    const tech = page.locator('dialog.tech-modal[open]');
    if (!(await tech.isVisible())) {
      await expect(card).toHaveAttribute('data-advisor-id', 'research');
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, '11-research-still-shown');
      await page.keyboard.press('t');
    }
    await expect(tech).toBeVisible();
    await expect(card).toHaveAttribute('data-advisor-id', 'research');
    await tech.getByRole('button', { name: 'Research now' }).first().click();
    // Close Research is known, so the card does not point at Close.
    await expect(page.locator('[data-testid="advisor"][data-advisor-id="closeResearch"]')).toHaveCount(0);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '12-research-chosen-no-close-pointer');
    await page.keyboard.press('Escape');
    await expect(tech).toBeHidden();
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '13-after-research');
    // With every unit ordered, End turn is known: no card.
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      s.editOrders((orders) => [...orders, ...v.units.filter((u) => u.owner === v.me && !orders.some((o) => 'unit' in o && o.unit === u.id))
        .map((u) => ({ type: 'SkipUnit' as const, unit: u.id }))]);
    });
    await expect(card).toHaveCount(0);
    await shot(page, size, '14-all-set-no-card');
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('turn-number')).toHaveText('2', { timeout: 60_000 });
    await page.waitForTimeout(1500);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '15-turn-two');
  });
});

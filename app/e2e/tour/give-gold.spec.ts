// Co-op Gold gifts (rules 50): the diplomacy panel's "Your team" section in each state (default amount with its
// forecast, an amount out of range, no amount, queued with Undo), the pending-orders line and the turn report lines.
// Run alone: `npx playwright test e2e/tour/give-gold`.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { capitalsFixture } from '../../src/test/capitalsFixture';
import type { Event } from '../../src/types';
import { SIZES, loadFixture, shot, stageHallReport, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`give gold ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('your team: amount, forecast, out of range, queued, order line and report lines', async ({ page }) => {
    await loadFixture(page, coreFixture(capitalsFixture().fixture), 2);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ diplomacyOpen: true }));
    const panel = page.getByRole('complementary', { name: 'Diplomacy inbox' });
    const team = panel.getByTestId('team-gifts');
    await expect(team).toBeVisible();
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'g1-diplomacy-top');
    await team.scrollIntoViewIfNeeded();
    await shot(page, size, 'g2-team-valid-forecast');
    const input = team.getByRole('spinbutton');
    await input.fill('9999');
    await expect(team.getByRole('alert')).toBeVisible();
    await shot(page, size, 'g3-team-out-of-range');
    await input.fill('');
    await expect(input).toHaveValue('');
    await shot(page, size, 'g4-team-empty');
    await input.fill('120');
    await team.getByTestId('give-gold').click();
    await expect(team.getByTestId('gift-queued')).toBeVisible();
    await shot(page, size, 'g5-team-queued-undo');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ diplomacyOpen: false }));
    await page.locator('summary', { hasText: 'Turn details' }).click();
    const toggle = page.locator('.orders-toggle');
    if (await toggle.getAttribute('aria-expanded') === 'false') await toggle.click();
    await page.locator('#pending-orders').scrollIntoViewIfNeeded();
    // The gift is a known order kind: the pending-orders line names it, never "Unknown order".
    await expect(page.locator('#pending-orders')).toContainText(/Give 120 Gold to (the )?[A-Z]/);
    await expect(page.locator('#pending-orders')).not.toContainText('Unknown order');
    await shot(page, size, 'g6-order-list-gift');
    const errors = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.errors[s.seat]; });
    expect.soft(errors, 'the gift order validates').toEqual([]);

    const events: Event[] = [
      { type: 'GoldGiven', from: 0, to: 4, amount: 120 },
      { type: 'GoldGiven', from: 4, to: 0, amount: 35 },
    ];
    await stageHallReport(page, events);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
    const report = page.getByRole('complementary', { name: 'Turn report' });
    await expect(report).toContainText('gave you 35 Gold');
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'g7-report-gift-lines');
  });
});

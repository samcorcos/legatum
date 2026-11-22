import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { sakokuState, swarmState, wildState } from '../../src/test/goalActionsFixture';
import { SIZES, loadFixture, shot } from './helpers';
import { expectSidebarGlance } from '../sidebarAssertions';

for (const size of SIZES) test.describe(`visible faction actions ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('Aztecs: blocked and ready', async ({ page }) => {
    const blocked = swarmState();
    for (const c of blocked.cities.filter((c) => c.owner === 0)) c.population = Math.min(c.population, 8);
    await loadFixture(page, coreFixture(blocked), blocked.turn);
    const button = page.locator('[data-goal-action="Swarm"] .goal-action-btn');
    await expect(button).toBeDisabled();
    await expect(page.locator('[data-goal-action="Swarm"]')).toContainText('21 of 30 population to spare');
    await expectSidebarGlance(page);
    await button.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-aztecs-blocked');

    const ready = swarmState();
    await loadFixture(page, coreFixture(ready), ready.turn);
    await expect(button).toBeEnabled();
    // One offer: the dock's primary names the action and what it scores, filled (not the outlined stepped-back style), with the only Return hint.
    const cta = page.locator('.right-sidebar .cta');
    await expect(cta).toContainText('Kindle the New Fire');
    await expect(cta).toContainText('+2 VP, kept for good');
    await expect(cta).not.toHaveClass(/stepped/);
    await expect(page.getByTestId('goal-ready-card')).toHaveCount(0);
    await expect(page.locator('.right-sidebar .kbd', { hasText: /^Return$/ })).toHaveCount(1);
    await expect(page.locator('.right-sidebar').getByText(/is ready/)).toHaveCount(0);
    await expectSidebarGlance(page);
    await button.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '02-aztecs-ready');
    await cta.click();
    await expect(page.locator('dialog[data-goal-dialog="Swarm"][open]')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(cta).not.toContainText('Kindle the New Fire');
    await expect(button).toBeEnabled();
  });

  test('legacy Sakoku: the former Japan action is visible', async ({ page }) => {
    const state = sakokuState();
    await loadFixture(page, coreFixture(state), state.turn);
    const button = page.locator('[data-goal-action="Sakoku"] .goal-action-btn');
    await expect(button).toBeEnabled();
    await button.scrollIntoViewIfNeeded();
    await shot(page, size, '03-legacy-sakoku');
  });

  test('Celts: warbands action is visible', async ({ page }) => {
    const state = wildState();
    await loadFixture(page, coreFixture(state), state.turn);
    const button = page.locator('[data-goal-action="CallTheWild"] .goal-action-btn');
    await expect(button).toBeEnabled();
    await button.scrollIntoViewIfNeeded();
    await shot(page, size, '04-celts-war-bands');
  });
});

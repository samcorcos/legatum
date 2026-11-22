// Screen tour, Training mode (2026-11-07): the top-bar chip while he plans, the progress card
// when End Turn waits for the AI's battle search, and the turn report's line on what it found.
// Run it alone with `npx playwright test e2e/tour/training`. Shots land in
// test-results/screen-tour/<size>/training/.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { siegeFixture } from '../../src/test/siegeFixture';
import { SIZES, dismissCelebrations, loadFixture, openReport, scrollWithinReport, shot } from './helpers';

test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('training: chip while planning, progress card after End Turn, report line after', async ({ page }) => {
    test.setTimeout(180_000);
    // Three enemy Soldiers at your city, and their empire is a Training AI: it studies the siege.
    const setup = siegeFixture({ fresh: true });
    const rival = setup.fixture.setup.players[1] as { difficulty?: string; search?: boolean };
    rival.difficulty = 'Hard';
    rival.search = true;
    // A long cap, and the search held until the card has been photographed.
    await page.addInitScript(() => {
      localStorage.setItem('legatum.training.capMs', '60000');
      (window as unknown as { legatumTrainingHold?: boolean }).legatumTrainingHold = true;
    });
    await loadFixture(page, coreFixture(setup.fixture), 2);

    const chip = page.getByTestId('training-chip');
    await expect(chip).toHaveText('Training · 1');
    await expect(chip).toHaveAttribute('aria-label', 'Training: studying 1 battle');
    await shot(page, size, '01-training-chip');

    await endPlanning(page);
    const card = page.getByTestId('training-card');
    await expect(card).toBeVisible();
    await expect(card).toContainText('Training: the AI is studying 1 battle');
    await expect(card).toContainText('Attack on Waypost Seven (1 of 1)');
    await page.waitForTimeout(1500);
    await shot(page, size, '02-training-card');
    await page.evaluate(() => { (window as unknown as { legatumTrainingHold?: boolean }).legatumTrainingHold = false; });

    await expect(page.getByTestId('turn-number')).toHaveText('3', { timeout: 60_000 });
    await dismissCelebrations(page);
    const report = await openReport(page);
    const line = report.getByText(/^Training: the AI (changed its plan|studied)/);
    await expect(line).toBeVisible();
    await scrollWithinReport(line);
    await shot(page, size, '03-training-report');

    // Every searched battle was logged (the browser build keeps the lines in memory).
    const lines = await page.evaluate(() => (window as unknown as { legatumTrainingLog?: string[] }).legatumTrainingLog ?? []);
    expect(lines.some((l) => JSON.parse(l).type === 'decision')).toBe(true);
  });
});

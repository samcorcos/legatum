import { expect, type Page } from '@playwright/test';

/** A new game opens with its founded capital selected and production as its first choice. */
export async function chooseCapitalProduction(page: Page): Promise<string> {
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await expect(page.locator('.seatbar .cta')).toContainText('Choose production');
  if (!await page.locator('.city-panel').isVisible()) await page.locator('.seatbar .cta').click();
  await expect(page.locator('.city-panel')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Build Settler', exact: true })).toHaveAttribute('aria-disabled', 'true');
  const production = page.locator('.city-panel .production-choice:not(.unavailable) .prod').first();
  const choice = (await production.getAttribute('aria-label'))!.replace(/^Build /, '');
  await production.click();
  await expect(page.locator('.city-panel')).toHaveCount(0);
  // The opening flow immediately offers research after production is chosen.
  // Leave that decision pending for specs that exercise the rest of the turn.
  const research = page.getByRole('dialog', { name: /Technologies|Research/ });
  await expect(research).toBeVisible();
  await research.getByRole('button', { name: 'Close research' }).click();
  return choice;
}

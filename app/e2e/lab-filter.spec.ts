// The Scenario Lab's filter: typing anywhere on the list narrows the cards, Esc clears it.
import { test, expect } from './fixtures';

test('typing filters the scenario cards and Esc clears the filter', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
  const cards = page.locator('[data-testid^="lab-card-"]');
  const all = await cards.count();
  expect(all).toBeGreaterThan(5);
  await page.keyboard.type('nicomedia');
  await expect(page.getByTestId('lab-filter')).toHaveValue('nicomedia');
  await expect(page.getByRole('option', { name: /Nicomedia/ }).first()).toBeVisible();
  expect(await cards.count()).toBeLessThan(all);
  await expect(page.getByTestId('lab-filter-count')).toContainText(`of ${all}`);
  await page.keyboard.type('zzzz');
  await expect(page.getByTestId('lab-filter-empty')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByTestId('lab-filter')).toHaveValue('');
  await expect(cards).toHaveCount(all);
  await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
  // Return opens the briefing of the first match.
  await page.keyboard.type('nicomedia');
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('lab-briefing')).toBeVisible();
});

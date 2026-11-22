import { test, expect } from './fixtures';

test('the Mongol faction sheet states the Medieval-era Siege limit in one plain sentence', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
  const stage = page.locator('dialog[data-testid="faction-stage"][open]');
  await page.getByRole('option', { name: 'Mongols' }).click();
  const sheet = stage.getByTestId('faction-sheet');
  await expect(sheet).toContainText('buy no Siege before the Medieval era.');
  await expect(sheet).not.toContainText('capture a city with a Workshop');
  await expect(sheet).toContainText('raise no Walls, Castles or Star Forts');
});

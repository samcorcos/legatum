import type { Page } from '@playwright/test';

/** Open the lobby's Advanced settings (closed by default) the way a player does. */
export async function openAdvanced(page: Page): Promise<void> {
  const details = page.locator('details.lobby-advanced');
  if (!(!(await details.evaluate((d) => (d as HTMLDetailsElement).open)))) {} else { await details.locator('> summary').click(); }
}

/** Type a map seed in the lobby: it lives under Advanced. */
export async function setMapSeed(page: Page, seed: string): Promise<void> {
  await openAdvanced(page);
  await page.getByLabel('Map seed', { exact: true }).fill(seed);
}

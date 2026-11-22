import { expect, type Page } from '@playwright/test';

/** Escape closes the current turn's interstitial queue, as it does for a player. */
export async function dismissInterstitials(page: Page) {
  const screen = page.getByTestId('alarm-screen');
  if (await screen.isVisible()) {
    await page.keyboard.press('Escape');
    await expect(screen).toHaveCount(0);
  }
}

/** A declared war is part of the turn's story; acknowledge it before using the report or dock. */
export async function acknowledgeWarDeclaredByYou(page: Page) {
  const screen = page.getByTestId('alarm-screen');
  await expect(screen).toHaveAttribute('data-alarm', 'warByYou');
  const show = screen.locator('.alarm-diplomacy', { hasText: 'Show on map' });
  await expect(screen.locator('.alarm-primary')).toHaveAccessibleName(/^Continue/);
  await expect(show).not.toHaveClass(/primary/);
  await expect(show).toBeVisible();
  await show.click();
  await expect(screen).toHaveCount(0);
}

/** Explicitly accept any pending decisions through the same desktop/touch flow as a player. */
export async function endPlanning(page: Page) {
  await expect(page.locator('[data-overlay-slot="turn-dock"]')).toBeVisible();
  // Wait for the turn dock: counting its CTA before the first render misses pending decisions.
  await expect(page.locator('.seatbar .cta').first()).toBeVisible();
  const pending = await page.locator('.seatbar .cta.todo').count() > 0;
  const touch = page.locator('.touch-turn-controls');
  if (await touch.isVisible()) await touch.getByRole('button', { name: /^(End Turn|Ready)$/ }).click();
  else await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn|Ready\b)/ }).click();
  if (pending) {
    const dialog = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
    await expect(dialog).toBeVisible();
    await expect(dialog.locator('li').first()).toBeVisible();
    await dialog.getByRole('button', { name: 'End turn anyway', exact: true }).click();
  }
}

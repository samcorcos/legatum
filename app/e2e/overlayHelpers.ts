import { expect, type Page } from '@playwright/test';

/** A selected unit collapses map options to a Lens toggle. Open it before using its controls. */
export async function showMapOptions(page: Page) {
  const toggle = page.getByRole('button', { name: /^Lens: / });
  if (await toggle.isVisible() && await toggle.getAttribute('aria-expanded') !== 'true') await toggle.click();
}

/** Yields checkbox in the map lens bar. */
export const overlayBox = (page: Page, name: string) =>
  page.locator('.map-lenses').getByRole('checkbox', { name: new RegExp(`^${name}`) });

/** The panel's lens radios (one at a time). */
export const lensRadio = (page: Page, name: string | RegExp) =>
  page.getByRole('radiogroup', { name: 'Map lens', exact: true }).getByRole('radio', { name: typeof name === 'string' ? new RegExp(`^${name}$`) : name });

export async function setOverlay(page: Page, name: string, on: boolean) {
  await showMapOptions(page);
  const box = overlayBox(page, name);
  if ((await box.getAttribute('aria-checked')) !== String(on)) await box.click();
  await expect(box).toHaveAttribute('aria-checked', String(on));
}

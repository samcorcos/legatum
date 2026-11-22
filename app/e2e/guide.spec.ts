import { endPlanning } from './turnActions';
import { chooseCapitalProduction } from './opening';
import { FACTIONS } from '../src/data';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';

const guide = (page: Page) => page.getByRole('dialog', { name: 'How to Play', exact: true });

async function openSolo(page: Page) {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
}

test('guide chapters, focus loop and shortcuts preserve the current screen', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^How to Play/ }).click();
  await expect(guide(page)).toBeVisible();
  const chapters = guide(page).getByRole('navigation', { name: 'Guide chapters' }).getByRole('button');
  await expect(chapters).toHaveCount(8);
  for (const label of ['Your first turns', 'Cities & research', 'Simultaneous combat', 'The victory race', 'The peoples', 'Controls & saves', 'Credits']) {
    await chapters.filter({ hasText: label }).click();
    await expect(guide(page).getByRole('heading', { name: label, exact: true })).toBeVisible();
  }
  await chapters.filter({ hasText: 'The peoples' }).click();
  await expect(guide(page).getByRole('heading', { level: 3 })).toHaveCount(FACTIONS.length);
  await expect(guide(page)).toContainText('Peoples with different ways to shape history.');
  await expect(guide(page)).toContainText('Ruhr Foundry replaces Forge');
  for (let i = 0; i < 16; i++) {
    await page.keyboard.press('Tab');
    expect(await page.evaluate(() => !!document.activeElement?.closest('dialog'))).toBe(true);
  }
  await page.keyboard.press('Shift+Tab');
  expect(await page.evaluate(() => !!document.activeElement?.closest('dialog'))).toBe(true);
  await page.keyboard.press('Escape');
  await expect(guide(page)).not.toBeVisible();

  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('button', { name: 'Close settings', exact: true }).click();
  await page.keyboard.type('?');
  await expect(guide(page)).toBeVisible();
  await page.setViewportSize({ width: 600, height: 720 });
  const bounds = await guide(page).boundingBox();
  expect(bounds!.width).toBeLessThanOrEqual(600);
  expect(await guide(page).getByRole('region').evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true);
  await page.keyboard.press('Escape');
  await page.setViewportSize({ width: 1440, height: 1100 });
  await page.getByRole('button', { name: /^Single Player/ }).click();
  const seed = page.getByLabel('Map seed', { exact: true });
  await seed.fill('42');
  await seed.press('End');
  await page.keyboard.type('?');
  await expect(guide(page)).not.toBeVisible();
  await seed.fill('4242');
  await seed.blur();
  await page.keyboard.type('?');
  await expect(guide(page)).toBeVisible();
  await page.keyboard.type('?');
  await expect(guide(page)).not.toBeVisible();
  await expect(seed).toHaveValue('4242');
});

test('in-game guide contains keys; hints dismiss, persist, disable and reset', async ({ page }) => {
  await openSolo(page);
  const hint = page.getByTestId('first-game-hint');
  await expect(hint).toContainText('Pick production');
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await page.getByRole('button', { name: 'How to Play (?)', exact: true }).click();
  await guide(page).getByRole('region').focus();
  for (const key of ['Enter', 'b', 'f', 't', '1', 'Space']) await page.keyboard.press(key);
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await expect(page.locator('.sidebar-turn-more > summary')).toContainText('0 orders');
  await page.keyboard.press('Escape');
  const dismiss = page.getByRole('button', { name: 'Dismiss hint: Pick production' });
  await dismiss.focus();
  await page.keyboard.press('Enter');
  await expect(hint).toContainText('Choose your first research');
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await page.getByRole('button', { name: 'Dismiss hint: Choose your first research' }).click();
  await expect(hint).not.toBeVisible();
  await chooseCapitalProduction(page);
  // Production alone leaves units and research pending; the ready hint must wait.
  await expect(hint).not.toBeVisible();
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  if (await page.getByRole('button', { name: 'Close report', exact: true }).isVisible()) await page.getByRole('button', { name: 'Close report', exact: true }).click();

  await page.reload();
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(hint).not.toBeVisible();
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await page.getByRole('button', { name: 'Reset hints', exact: true }).click();
  await expect(hint).toContainText('Choose your first research');
  await page.getByLabel('First-game hints', { exact: true }).uncheck();
  await expect(hint).not.toBeVisible();
  await page.reload();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await expect(page.getByLabel('First-game hints', { exact: true })).not.toBeChecked();
  await page.getByRole('button', { name: 'Reset hints', exact: true }).click();
  await expect(page.getByLabel('First-game hints', { exact: true })).toBeChecked();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(hint).toContainText('Choose your first research');
});

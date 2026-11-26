import { FACTIONS } from '../src/data';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { openAdvanced, setMapSeed } from './mapSeed';

const guide = (page: Page) => page.getByRole('dialog', { name: 'How to Play', exact: true });

async function openSolo(page: Page) {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await setMapSeed(page, '424242');
  await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
}

test('guide chapters, focus loop and shortcuts preserve the current screen', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^How to Play/ }).click();
  await expect(guide(page)).toBeVisible();
  const chapters = guide(page).getByRole('navigation', { name: 'How to Play chapters' }).getByRole('button');
  await expect(chapters).toHaveCount(8);
  for (const label of ['Your first turns', 'Cities & research', 'Simultaneous combat', 'The victory race', 'The peoples', 'Controls & saves', 'Credits']) {
    await chapters.filter({ hasText: label }).click();
    await expect(guide(page).getByRole('heading', { name: label, exact: true })).toBeVisible();
  }
  await chapters.filter({ hasText: 'The peoples' }).click();
  await expect(guide(page).getByRole('heading', { level: 3 })).toHaveCount(FACTIONS.length);
  await expect(guide(page)).toContainText('Every faction is playable, with a rule it bends and a weakness that pays for it.');
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
  await openAdvanced(page);
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

test('in-game guide contains keys; advisor moves on and its switch persists', async ({ page }) => {
  await openSolo(page);
  const advisor = page.getByTestId('advisor');
  await expect(advisor).toHaveAttribute('data-advisor-id', 'welcome');
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await expect(advisor).toHaveCount(0);
  await page.getByRole('button', { name: 'How to Play (?)', exact: true }).click();
  await guide(page).getByRole('region').focus();
  for (const key of ['Enter', 'b', 'f', 't', '1', 'Space']) await page.keyboard.press(key);
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await expect(page.locator('.sidebar-turn-more > summary')).toContainText('0 orders');
  await page.keyboard.press('Escape');
  await expect(advisor).toHaveAttribute('data-advisor-id', 'welcome');
  await page.keyboard.press('Enter');
  await expect(advisor).not.toHaveAttribute('data-advisor-id', 'welcome');
  const first = await advisor.getAttribute('data-advisor-id');
  await advisor.getByRole('button', { name: 'Not now' }).click();
  await expect(advisor).not.toHaveAttribute('data-advisor-id', first!);
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await page.getByLabel('Show the advisor').uncheck();
  await expect(advisor).toHaveCount(0);
  await page.reload();
  // The game in progress resumes; the switch stayed off across the reload.
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await expect(advisor).toHaveCount(0);
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await expect(page.getByLabel('Show the advisor')).not.toBeChecked();
  await page.getByLabel('Show the advisor').check();
  await page.keyboard.press('Escape');
  await expect(advisor).toBeVisible();
});

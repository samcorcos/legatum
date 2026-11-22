// Team map pings in a hot seat game: the idle and armed control, your own ping, the teammate's view with the named
// Go to ping button, and the Guide paragraph. Run alone: `npx playwright test e2e/tour/team-ping`.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { SIZES, shot } from './helpers';

async function startHotSeat(page: Page) {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Multiplayer/ }).click();
  await page.getByRole('button', { name: /^Hot Seat/ }).click();
  const small = page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ });
  await small.click();
  await expect(small).toBeChecked();
  await page.getByLabel('Map seed', { exact: true }).fill('9001');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  const gotIt = page.getByRole('button', { name: 'Got it', exact: true });
  if (await gotIt.count()) await gotIt.click();
}

for (const size of SIZES) test.describe(`team ping ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('idle, armed with a ghost ring, own ping, teammate view and Guide', async ({ page }) => {
    await startHotSeat(page);
    const button = page.getByTestId('ping-button');
    await expect(page.getByTestId('ping-control')).toBeInViewport({ ratio: 1 });
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'p1-idle');
    // Armed: ink fill, Cancel ping with Esc, the instruction, and the hovered tile shows a ghost ring (no route or select ring).
    await button.click();
    await expect(page.getByTestId('ping-hint')).toBeVisible();
    const map = (await page.locator('[data-game-map]').boundingBox())!;
    await page.mouse.move(map.x + map.width * 0.5, map.y + map.height * 0.68);
    await shot(page, size, 'p2-armed-ghost');
    await page.mouse.click(map.x + map.width * 0.45, map.y + map.height * 0.5);
    await expect(button).toHaveText(/^Ping\s*P$/);
    await page.mouse.move(5, size.height / 2);
    await page.waitForTimeout(2800); // past the pulse: the still ring
    await shot(page, size, 'p3-own-ping');
    // The teammate takes the device: the ping and a named, outlined way to it.
    await page.keyboard.press('2');
    const go = page.getByTestId('ping-go');
    await expect(go).toBeInViewport({ ratio: 1 });
    await expect(go).toContainText('pinged · Go to ping');
    await expect(button).toHaveAttribute('aria-pressed', 'false');
    await page.waitForTimeout(300);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'p4-teammate-go-to-ping');
    // An armed Ping does not survive the next hand-over.
    await button.click();
    await page.keyboard.press('1');
    await expect(page.getByTestId('ping-button')).toHaveAttribute('aria-pressed', 'false');
  });

  test('the Guide paragraph sits after the Alliance and Peace rules', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /Your first turns/ }).click();
    const para = guide.locator('.guide-page p', { hasText: 'With a human teammate you can ping.' });
    await para.scrollIntoViewIfNeeded();
    await expect(para).toBeVisible();
    await expect(para.locator('xpath=preceding-sibling::p[1]')).toContainText('A Peace offer is between two teams.');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'p5-guide-paragraph');
  });
});

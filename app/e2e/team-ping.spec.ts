import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';

// Team pings in a hot seat game: the Ping control shows with a human teammate, a click on the map pings that
// tile, and the teammate sees the ring (and a way to it) when they take the device.
const SHOTS = process.env.LEGATUM_SHOTS;

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

const pingCount = (page: Page) => page.evaluate(() => (window as unknown as { legatum: { getState(): { pings: unknown[] } } }).legatum.getState().pings.length);

for (const [w, h] of [[1000, 600], [1440, 900]]) {
  test(`a ping reaches the teammate who takes the device at ${w}x${h}`, async ({ page }) => {
    await page.setViewportSize({ width: w, height: h });
    await startHotSeat(page);
    const control = page.getByTestId('ping-control');
    await expect(control).toBeInViewport({ ratio: 1 });
    const button = page.getByTestId('ping-button');
    await expect(button).toHaveText(/^Ping\s*P$/);
    if (SHOTS) await page.screenshot({ path: `${SHOTS}/ping-control-${w}x${h}.png` });
    await button.click();
    await expect(button).toHaveText(/^Cancel ping\s*Esc$/);
    await expect(page.getByTestId('ping-hint')).toHaveText('Click a tile to ping it');
    await expect(button).toHaveAttribute('aria-pressed', 'true');
    // Click the map a little off centre: the tile there is the ping.
    const map = page.locator('[data-game-map]');
    const box = (await map.boundingBox())!;
    await page.mouse.click(box.x + box.width * 0.45, box.y + box.height * 0.5);
    await expect(button).toHaveText(/^Ping\s*P$/);
    expect(await pingCount(page)).toBe(1);
    if (SHOTS) await page.screenshot({ path: `${SHOTS}/ping-live-${w}x${h}.png` });
    // The teammate takes the device: the ring and a way to it are there.
    await page.keyboard.press('2');
    await expect(page.getByTestId('ping-go')).toHaveText(/pinged · Go to ping$/);
    await expect(page.getByTestId('ping-go')).toBeInViewport({ ratio: 1 });
    await page.waitForTimeout(700);
    if (SHOTS) await page.screenshot({ path: `${SHOTS}/ping-teammate-${w}x${h}.png` });
  });
}

test('a solo game has no Ping control', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await expect(page.getByTestId('ping-control')).toHaveCount(0);
});

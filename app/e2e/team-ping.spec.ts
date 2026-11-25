import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';

// Team pings in an online game: the Ping control shows with a human teammate, a click on the map pings that
// tile, and the teammate's browser shows the ring with a way to it. Hot seat is gone (2026-11-23), so the host
// and guest are two browser contexts joined through the real relay under wrangler dev.
const SHOTS = process.env.LEGATUM_SHOTS;

async function startOnline(host: Page, guest: Page) {
  await host.goto('/?client=wasm');
  await host.getByRole('button', { name: /^Multiplayer/ }).click();
  await host.getByLabel('Your name in the game list').fill('Ada');
  await host.getByRole('button', { name: /^Host Game/ }).click();
  await expect(host.getByRole('heading', { name: 'Host Game', exact: true })).toBeVisible();
  const small = host.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ });
  await small.click();
  await expect(small).toBeChecked();
  await host.getByLabel('Map seed', { exact: true }).fill('9001');

  await guest.goto('/?client=wasm');
  await guest.getByRole('button', { name: /^Multiplayer/ }).click();
  await guest.getByRole('button', { name: /^Join Game/ }).click();
  const row = guest.locator('.online-room').filter({ hasText: 'Ada' });
  await expect(row).toHaveCount(1, { timeout: 20_000 });
  await row.getByRole('button', { name: /^Join/ }).click();
  await guest.getByRole('row').nth(2).getByRole('button', { name: 'Take seat', exact: true }).click();
  await guest.getByLabel('Your name', { exact: true }).fill('Alice');
  await guest.getByRole('button', { name: /^Ready Lock/ }).click();

  const start = host.getByRole('button', { name: /^Start game/ });
  await expect(start).toBeEnabled();
  await start.click();
  for (const page of [host, guest]) {
    await expect(page.getByTestId('turn-number')).toHaveText('1');
    const gotIt = page.getByRole('button', { name: 'Got it', exact: true });
    if (await gotIt.count()) await gotIt.click();
  }
}

const pingCount = (page: Page) => page.evaluate(() => (window as unknown as { legatum: { getState(): { pings: unknown[] } } }).legatum.getState().pings.length);

for (const [w, h] of [[1000, 600], [1440, 900]]) {
  test(`a ping reaches the teammate's browser at ${w}x${h}`, async ({ page: host, guestPage: guest, relay: _relay }) => {
    test.setTimeout(150_000);
    await host.setViewportSize({ width: w, height: h });
    await guest.setViewportSize({ width: w, height: h });
    await startOnline(host, guest);

    const control = host.getByTestId('ping-control');
    await expect(control).toBeInViewport({ ratio: 1 });
    const button = host.getByTestId('ping-button');
    await expect(button).toHaveText(/^Ping\s*P$/);
    if (SHOTS) await host.screenshot({ path: `${SHOTS}/ping-control-${w}x${h}.png` });
    await button.click();
    await expect(button).toHaveText(/^Cancel ping\s*Esc$/);
    await expect(host.getByTestId('ping-hint')).toHaveText('Click a tile to ping it');
    await expect(button).toHaveAttribute('aria-pressed', 'true');
    // Click the map a little off centre: the tile there is the ping.
    const map = host.locator('[data-game-map]');
    const box = (await map.boundingBox())!;
    await host.mouse.click(box.x + box.width * 0.45, box.y + box.height * 0.5);
    await expect(button).toHaveText(/^Ping\s*P$/);
    expect(await pingCount(host)).toBe(1);
    if (SHOTS) await host.screenshot({ path: `${SHOTS}/ping-live-${w}x${h}.png` });

    // The teammate's browser shows who pinged and a way to it.
    await expect(guest.getByTestId('ping-go')).toHaveText(/Ada pinged · Go to ping$/, { timeout: 20_000 });
    await expect(guest.getByTestId('ping-go')).toBeInViewport({ ratio: 1 });
    if (SHOTS) await guest.screenshot({ path: `${SHOTS}/ping-teammate-${w}x${h}.png` });
  });
}

test('a solo game has no Ping control', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await expect(page.getByTestId('ping-control')).toHaveCount(0);
});

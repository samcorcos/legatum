import { testWithFailingRequests as test, expect } from '../fixtures';
import { SIZES, shot } from './helpers';

for (const size of SIZES) test(`multiplayer turn timer ${size.name}`, async ({ page: host, browser, baseURL, relay }) => {
  test.setTimeout(120_000);
  expect(relay).toBeTruthy();
  await host.setViewportSize(size);
  const guestContext = await browser.newContext({ baseURL, viewport: size });
  const guest = await guestContext.newPage();
  try {
    for (const p of [host, guest]) await p.addInitScript(() => {
      localStorage.setItem('legatum.intro.v1', 'off');
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await host.goto('/');
    await host.getByRole('button', { name: /^Multiplayer/ }).click();
    await host.getByRole('button', { name: /^Host Game/ }).click();
    await expect(host.getByLabel('Turn timer', { exact: true })).toHaveValue('0');
    await host.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
    await host.getByLabel('Turn timer', { exact: true }).selectOption('30');
    await shot(host, size, '01-host-timer');
    const roomName = await host.locator('.invite-name').innerText();
    await guest.goto('/');
    await guest.getByRole('button', { name: /^Multiplayer/ }).click();
    await guest.getByRole('button', { name: /^Join Game/ }).click();
    await guest.locator('.online-room').filter({ hasText: roomName }).getByRole('button', { name: /^Join/ }).click();
    await expect(guest.getByLabel('Turn timer', { exact: true })).toHaveValue('30');
    await expect(guest.getByLabel('Turn timer', { exact: true })).toBeDisabled();
    await guest.getByRole('row').nth(2).getByRole('button', { name: 'Take seat', exact: true }).click();
    await guest.getByRole('button', { name: /^Ready Lock/ }).click();
    await guest.getByLabel('Turn timer', { exact: true }).scrollIntoViewIfNeeded();
    await shot(guest, size, '02-guest-timer');
    await host.getByRole('button', { name: /^Start game/ }).click();
    for (const p of [host, guest]) {
      await expect(p.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
      await expect(p.getByRole('timer')).toContainText('Turn ends in 0:');
    }
    await shot(host, size, '03-countdown');
    await expect(host.getByTestId('turn-number')).toHaveText('2', { timeout: 45_000 });
    await expect(guest.getByTestId('turn-number')).toHaveText('2');
    await expect(host.getByRole('timer')).toContainText('Turn ends in 0:');
    await host.setViewportSize({ width: 390, height: 844 });
    await expect(host.locator('.phone-turn-dock')).toBeVisible();
    await host.waitForTimeout(400);
    await host.screenshot({ path: `test-results/turn-timer-phone-${size.name}.png` });
    await expect(host.getByRole('timer').first()).toBeVisible();
  } finally { await guestContext.close(); }
});

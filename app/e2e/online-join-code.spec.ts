import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Locator, Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { chooseCapitalProduction } from './opening';
import { endPlanning } from './turnActions';

// A host locks its game with a join code and removes a player (plans/online-play.md): the real relay under
// wrangler dev, the host's engine in its own tab, one guest in a second browser context.
// Set JOIN_CODE_SHOTS=<folder> to also save screenshots at both laptop sizes.

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shots = process.env.JOIN_CODE_SHOTS;
if (shots) mkdirSync(shots, { recursive: true });

async function snap(page: Page, name: string, show?: Locator) {
  if (!shots) return;
  const keep = page.viewportSize();
  for (const size of SIZES) {
    await page.setViewportSize(size);
    await page.waitForTimeout(700);
    await show?.scrollIntoViewIfNeeded();
    await page.screenshot({ path: join(shots, `${name}-${size.width}x${size.height}.png`) });
  }
  if (keep) await page.setViewportSize(keep);
}

async function hostLocked(host: Page, code: string) {
  await host.goto('/');
  await host.getByRole('button', { name: /^Multiplayer/ }).click();
  await host.getByLabel('Your name', { exact: true }).fill('Ada');
  await host.getByLabel('Join code (optional)').fill(code);
  await snap(host, 'host-game-join-code');
  await host.getByRole('button', { name: /^Host Game/ }).click();
  await expect(host.getByRole('heading', { name: 'Game room', exact: true })).toBeVisible();
  const small = host.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ });
  await small.click();
  await host.getByLabel('Map seed', { exact: true }).fill('20240926');
}

test('Online: a locked game asks the guest for its code, and the host can remove the player', async ({ page: host, guestPage: guest, relay: _relay }) => {
  test.setTimeout(180_000);
  await hostLocked(host, 'Wolf42');

  await guest.goto('/');
  await guest.getByRole('button', { name: /^Multiplayer/ }).click();
  await guest.getByRole('button', { name: /^Join Game/ }).click();
  const row = guest.locator('.online-room').filter({ hasText: 'Ada' });
  await expect(row).toHaveCount(1, { timeout: 20_000 });
  await expect(row).toContainText('Locked');
  await snap(guest, 'lobby-locked');
  await row.getByRole('button', { name: /^Join/ }).click();
  await expect(guest.getByRole('heading', { name: 'Game room', exact: true })).toBeVisible();

  // The seat button waits for the code; a wrong one is refused in plain words.
  const take = guest.getByRole('row').nth(2).getByRole('button', { name: 'Take seat', exact: true });
  await expect(guest.getByLabel('Join code', { exact: true })).toBeVisible();
  await expect(take).toBeDisabled();
  await snap(guest, 'guest-code-prompt');
  await guest.getByLabel('Join code', { exact: true }).fill('nope99');
  await take.click();
  await expect(guest.getByText('That join code is not right.')).toBeVisible();
  await snap(guest, 'guest-code-wrong');
  await expect(host.getByRole('row').nth(2)).not.toContainText('Alice');

  await guest.getByLabel('Join code', { exact: true }).fill('wolf42');
  await guest.getByLabel('Join code', { exact: true }).press('Tab');
  await take.click();
  await guest.getByLabel('Your name', { exact: true }).fill('Alice');
  await guest.getByRole('button', { name: /^Ready Lock/ }).click();

  // The host sees Remove on the guest's seat, asks first, and Keep leaves the player.
  const seat = host.getByRole('row').nth(2);
  await expect(seat).toContainText('Alice');
  await snap(host, 'host-seats-remove');
  await seat.getByRole('button', { name: 'Remove Alice' }).click();
  await expect(seat.getByRole('button', { name: 'Keep' })).toBeFocused();
  await seat.getByRole('button', { name: 'Keep' }).click();
  await expect(seat).toContainText('Alice');
  await seat.getByRole('button', { name: 'Remove Alice' }).click();
  await snap(host, 'host-seats-remove-ask');
  await seat.getByRole('button', { name: 'Remove', exact: true }).click();

  // The seat stays closed: the guest is told, and cannot take it again.
  await expect(guest.getByText('The host removed you from this game.')).toBeVisible();
  await expect(seat).toContainText('disconnected');
  await expect(seat.getByRole('button', { name: 'Remove Alice' })).toHaveCount(0);
  await expect(guest.getByRole('row').nth(2).getByRole('button', { name: 'Take seat', exact: true })).toHaveCount(0);

  // The host opens the seat again and the guest can take it with the code.
  host.once('dialog', (d) => void d.accept());
  await seat.getByRole('button', { name: 'Open seat' }).click();
  await guest.getByRole('row').nth(2).getByRole('button', { name: 'Take seat', exact: true }).click();
  await expect(guest.getByLabel('Your name', { exact: true })).toBeVisible();
});

test('Online: removing a player in a running game does not hold up the turn', async ({ page: host, guestPage: guest, relay: _relay }) => {
  test.setTimeout(180_000);
  await hostLocked(host, 'Wolf42');
  await guest.goto('/');
  await guest.getByRole('button', { name: /^Multiplayer/ }).click();
  await guest.getByRole('button', { name: /^Join Game/ }).click();
  await guest.locator('.online-room').filter({ hasText: 'Ada' }).getByRole('button', { name: /^Join/ }).click();
  await guest.getByLabel('Join code', { exact: true }).fill('WOLF42');
  await guest.getByRole('row').nth(2).getByRole('button', { name: 'Take seat', exact: true }).click();
  await guest.getByRole('button', { name: /^Ready Lock/ }).click();
  const start = host.getByRole('button', { name: /^Start game/ });
  await expect(start).toBeEnabled();
  await start.click();
  await expect(host.getByTestId('turn-number')).toHaveText('1');
  await expect(guest.getByTestId('turn-number')).toHaveText('1');

  // Remove is rare, so it waits behind Turn details (the fold opens by itself only for a lost seat).
  const fold = host.locator('.sidebar-turn-more');
  if (!(await fold.evaluate((el) => (el as HTMLDetailsElement).open))) await fold.locator('summary').first().click();
  const seat = host.getByTestId('online-seat-1');
  await expect(seat).toContainText('online');
  await snap(host, 'host-game-seats-remove', seat.getByRole('button', { name: /^Remove / }));
  await seat.getByRole('button', { name: /^Remove / }).click();
  await seat.getByRole('button', { name: 'Remove', exact: true }).click();
  await expect(guest.getByText('The host removed you from this game.')).toBeVisible();
  await expect(seat).toContainText('not connected');

  // The host alone can finish the turn.
  await chooseCapitalProduction(host);
  await endPlanning(host);
  await expect(host.getByTestId('turn-number')).toHaveText('2', { timeout: 60_000 });
});

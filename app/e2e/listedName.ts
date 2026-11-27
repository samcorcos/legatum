import type { Page } from '@playwright/test';
import { expect } from './fixtures';

/** The name the public game list shows for the game this host page opened, read from its game room. */
export async function listedGameName(host: Page): Promise<string> {
  const name = host.locator('.invite-name');
  await expect(name).toBeVisible({ timeout: 20_000 });
  const text = ((await name.textContent()) ?? '').trim();
  if (!text) throw new Error('the game room does not name the listed game');
  return text;
}

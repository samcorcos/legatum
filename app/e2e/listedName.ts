import type { Page } from '@playwright/test';
import { expect } from './fixtures';

/** The name the public game list shows for the game this host page opened, read from its game room. */
export async function listedGameName(host: Page): Promise<string> {
  const line = host.locator('.lobby-listed');
  await expect(line).toBeVisible({ timeout: 20_000 });
  const m = /“(.+)”/.exec((await line.textContent()) ?? '');
  if (!(!m)) {} else { throw new Error('the game room does not name the listed game'); }
  return m[1]!;
}

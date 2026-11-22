// An older rules version is visible in the save list and cannot be resumed.
// The stored save must remain byte-for-byte untouched.
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { wallsFixture } from '../src/test/wallsFixture';

test('an older save explains itself in the save list and remains on disk', async ({ page }: { page: Page }) => {
  const game = coreFixture(wallsFixture().fixture).replace(/"save_version":\d+/, '"save_version":5');
  const save = JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2,
    names: ['Ada'], playtest: 'Playtest 8.9.19', savedAt: new Date().toISOString(), game });
  await page.addInitScript((contents) => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.autosave.v1', contents);
  }, save);
  await page.goto('/?client=wasm');
  const saved = page.getByRole('button', { name: /^Saved game \(older version\)/ });
  await expect(saved).toBeDisabled();
  await expect(page.getByRole('alert')).toContainText("This game was saved by Playtest 8.9.19 with older rules and can't continue. Starting a new game replaces the autosave.");
  await expect(page.getByRole('button', { name: /^Continue/ })).toHaveCount(0);
  await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('legatum.autosave.v1'))).toBe(save);
  await expect(page.locator('.loading-screen')).toHaveCount(0);
});

import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';

// Training (Hard, with each battle studied from the true state) is a research tool. A production
// build never offers it to players (2026-11-21): the lobby lists Easy, Normal and Hard only, and
// a browser that remembers Training from an older build opens on Hard.
test.use({ viewport: { width: 1000, height: 600 } });

const level = (page: Page) => page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true });

async function openLobby(page: Page, url: string) {
  await page.goto(url);
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
}

async function expectThreeLevels(page: Page) {
  await expect(level(page).getByRole('radio')).toHaveText([/^Easy/, /^Normal/, /^Hard/]);
  await expect(page.getByTestId('ai-level-note')).toHaveCount(0);
  const seats = page.getByRole('combobox', { name: /^Seat \d+ AI level$/ });
  expect(await seats.count()).toBeGreaterThanOrEqual(3);
  for (const seat of await seats.all()) {
    await expect(seat.getByRole('option')).toHaveText(['Easy', 'Normal', 'Hard']);
  }
  await expect(level(page).getByRole('radio', { name: /Training/ })).toHaveCount(0);
}

test('the lobby offers Easy, Normal and Hard only, even with the developer switch in the URL', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('legatum.dev.training', '1'));
  await openLobby(page, '/?client=wasm&training=1');
  await expectThreeLevels(page);
});

for (const stored of ['Training', 'HardSearch']) {
  test(`a remembered ${stored} level opens as Hard and starts a game without the search flag`, async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(String(e)));
    await page.addInitScript((level) => {
      if (!sessionStorage.getItem('seeded')) localStorage.setItem('legatum.lobby.difficulty', JSON.stringify({ level, seats: { 'solo:2': level } }));
      sessionStorage.setItem('seeded', '1');
    }, stored);
    await openLobby(page, '/?client=wasm');
    await expectThreeLevels(page);
    await expect(level(page).getByRole('radio', { name: /^Hard/ })).toBeChecked();
    for (const seat of await page.getByRole('combobox', { name: /^Seat \d+ AI level$/ }).all()) await expect(seat).toHaveValue('Hard');
    await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1');
    const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('legatum.autosave.v1') ?? 'null'));
    expect(JSON.parse(saved.game).setup.players.filter((p: { search?: boolean }) => p.search)).toHaveLength(0);
    expect(errors).toEqual([]);
  });
}

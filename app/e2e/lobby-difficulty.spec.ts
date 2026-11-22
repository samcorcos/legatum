import { test, expect } from './fixtures';
import { revealSidebar } from './sidebarAssertions';

test.use({ viewport: { width: 1000, height: 600 } });

test('an 8.9.11 Hard profile starts a game with every AI on Hard', async ({ page }) => {
  await page.addInitScript(() => {
    const oldGame = { setup: { players: [{ is_human: true }, { is_human: false, difficulty: 'Hard' }, { is_human: false, difficulty: 'Hard' }] } };
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', savedAt: '2026-10-25T12:00:00.000Z', turn: 20, names: ['Player 1'], game: JSON.stringify(oldGame) }));
  });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  await expect(page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true }).getByRole('radio', { name: /^Hard\s+Ruthless/ })).toBeChecked();
  const seats = page.getByRole('combobox', { name: /^Seat \d+ AI level$/ });
  await expect(seats).toHaveCount(3);
  for (const seat of await seats.all()) await expect(seat).toHaveValue('Hard');

  await page.getByRole('button', { name: /^Start game/ }).click();
  // The old save that carried the Hard profile is still on disk, so the lobby asks before replacing it.
  await page.getByTestId('replace-game').getByRole('button', { name: 'Start new game' }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  // The new game's own players (its autosave now lives in the browser's database, not the old localStorage slot).
  const difficulties = await page.evaluate(() => {
    const s = (window as unknown as { legatum: { getState(): { seat: number; views: { players: { is_human: boolean; difficulty?: string }[] }[] } } }).legatum.getState();
    return s.views[s.seat].players.filter((player) => !player.is_human).map((player) => player.difficulty);
  });
  expect(difficulties).toEqual(['Hard', 'Hard', 'Hard']);
  await revealSidebar(page, 'standings');
  await expect(page.locator('.vp-ai-level')).toHaveCount(3);
  await expect(page.locator('.vp-ai-level').first()).toHaveText('AI · Hard');
  await page.getByRole('button', { name: /^Diplomacy( \(\d+\))?$/ }).click();
  await expect(page.locator('.empire-pick-level')).toHaveCount(3);
  await expect(page.locator('.empire-pick-level').first()).toHaveText('AI · Hard');
  await page.getByRole('button', { name: 'Close diplomacy' }).click();
  await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'Score ledger', exact: true }).click();
  await expect(page.locator('.sl-ai-level')).toHaveCount(3);
});

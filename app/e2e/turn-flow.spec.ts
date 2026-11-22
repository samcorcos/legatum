import { revealSidebar } from './sidebarAssertions';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { turnFlowFixture } from '../src/test/turnFlowFixture';
import { coreFixture } from '../src/test/coreFixture';

test.beforeEach(async ({ page }) => {
  const fixture = turnFlowFixture();
  fixture.players[1].isHuman = false;
  fixture.setup.players[1].is_human = false;
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2,
      names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.locator('[data-game-map]').focus();
});

test('Return cycles units, production and research while decisions remain', async ({ page }) => {
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Spearman');
  await page.keyboard.press('Enter');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await page.keyboard.press('Enter');
  await expect(page.locator('.city-panel')).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
  await page.keyboard.press('Escape');
  for (let i = 0; i < 12; i++) {
    // Dialog controls keep native Enter; close research to continue the gameplay cycle.
    if (await page.locator('dialog.tech-modal').isVisible()) await page.keyboard.press('Escape');
    await page.locator('[data-game-map]').focus();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    await expect(page.locator('dialog.end-turn-confirmation')).toHaveCount(0);
  }
});

test('Shift+Return reviews every pending decision; Return on its default control goes to the first', async ({ page }) => {
  await page.keyboard.press('Shift+Enter');
  const dialog = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('Spearman #0: needs orders');
  await expect(dialog).toContainText('Scout #4: needs orders');
  await expect(dialog).toContainText(/Choose what .+ builds/);
  await expect(dialog).toContainText('Choose research');
  await expect(dialog.getByRole('button', { name: 'Give the Spearman orders' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(dialog).toHaveCount(0);
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Spearman');
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Shift+Enter');
  await dialog.getByRole('button', { name: 'End turn anyway', exact: true }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('3');
});

test('Space lasts one turn; Fortify persists and stays out of the cycle', async ({ page }) => {
  await page.keyboard.press('Space');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await page.keyboard.press('f');
  await expect(page.locator('.unit-panel')).toHaveCount(0);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Spearman');
  await page.keyboard.press('Enter');
  await expect(page.locator('.city-panel')).toBeVisible();
  // Autosave is an output readback: no store injection or debug mutations.
  const units = await page.evaluate(() => JSON.parse(JSON.parse(localStorage.getItem('legatum.autosave.v1')!).game).units);
  expect(units['0'].standing_order).toBeUndefined();
  expect(units['4'].standing_order).toBe('Sleep');
});

test('Return ends the turn when all decisions are handled, and focused controls keep native Space', async ({ page }) => {
  const skip = page.getByRole('button', { name: /^Skip this turn/ });
  await skip.focus();
  await page.keyboard.press('Space');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await skip.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.unit-panel')).toHaveCount(0);
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Enter');
  await page.locator('.city-panel .production-choice:not(.unavailable) button.prod:enabled').first().click();
  const research = page.locator('dialog.tech-modal');
  await expect(page.locator('.city-panel')).toHaveCount(0);
  await expect(research).toBeVisible();
  await research.locator('.era-card.available').first().getByTestId('research-now').click();
  await page.keyboard.press('Escape');
  await expect(page.locator('.seatbar .cta.ready')).toBeVisible();
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  await expect(page.locator('dialog.end-turn-confirmation')).toHaveCount(0);
});

test('mouse Build advances to research and the city remains reachable from Cities', async ({ page }) => {
  await revealSidebar(page, 'turn-more');
  await page.locator('.city-list summary').click();
  const city = page.locator('.city-list-items button').first();
  await city.click();
  const build = page.getByRole('button', { name: 'Build Spearman', exact: true });
  await build.click();
  await expect(page.locator('.city-panel')).toHaveCount(0);
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.keyboard.press('Escape');
  await city.click();
  await expect(build).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByRole('button', { name: 'Queue Spearman', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: /^Buy Spearman for/ })).toBeVisible();
});

test('Fortify heals while the unit rests and stays in effect at full HP, out of the cycle', async ({ page }) => {
  await page.keyboard.press('f');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('f');
  await expect(page.locator('.unit-panel')).toHaveCount(0);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  await expect(page.locator('.seatbar .cta')).toContainText('Choose production');
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('4');
  const units = await page.evaluate(() => JSON.parse(JSON.parse(localStorage.getItem('legatum.autosave.v1')!).game).units);
  expect(units['0'].hp).toBe(100);
  expect(units['0'].standing_order).toBe('Sleep');
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.unit-panel')).toHaveCount(0);
});


test('the turn button counts the selected last idle unit, then the selected idle city', async ({ page }) => {
  await expect(page.locator('.seatbar .cta')).toContainText('2 units need orders');
  await page.keyboard.press('Space');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await expect(page.locator('.seatbar .cta')).toContainText('1 unit needs orders');
  await page.locator('.seatbar .cta').click();
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await page.keyboard.press('Space');
  await expect(page.locator('.seatbar .cta')).toContainText('Choose production');
  await page.locator('.seatbar .cta').click();
  await expect(page.locator('.city-panel')).toBeVisible();
  await expect(page.locator('.seatbar .cta')).toContainText('Choose production');
  const production = page.locator('.city-panel .production-choice:not(.unavailable) button.prod:enabled').first();
  await production.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.city-panel')).toHaveCount(0);
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
  await page.keyboard.press('Escape');
  if (await page.locator('dialog.tech-modal').isVisible()) await page.keyboard.press('Escape');
  await expect(page.locator('[data-game-map]')).toBeFocused();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
});

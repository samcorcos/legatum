import { test, expect } from '../fixtures';
import { revealSidebar } from '../sidebarAssertions';
import { TECHS } from '../../src/data';
import { coreFixture } from '../../src/test/coreFixture';
import { swarmState } from '../../src/test/goalActionsFixture';
import { SIZES, loadFixture, selectThing, shot, stageHallReport, frameHall } from './helpers';

for (const size of SIZES) test.describe(`Aztecs ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('faction sheet uses the portrait and explains the strength and price', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    await page.getByRole('option', { name: /^Aztecs / }).click();
    const sheet = page.getByRole('dialog', { name: 'Choose your empire' }).getByTestId('faction-sheet');
    await expect(sheet).toContainText('Flower Wars');
    await expect(sheet).toContainText('Early Tanks, Tanks and later armour are allowed');
    await expect(sheet).toContainText('Offer 30 population');
    expect(await sheet.evaluate((el) => el.scrollWidth <= el.clientWidth + 1)).toBe(true);
    await shot(page, size, '01-faction-sheet');
  });

  test('permanent horse exclusions, thirty torches and the Flower Wars population popup', async ({ page }) => {
    const s = swarmState(); s.players[0].techs = [...TECHS];
    await loadFixture(page, coreFixture(s), s.turn);
    const city = s.cities[0];
    await selectThing(page, 'city', city.id);
    const panel = page.locator('.city-panel');
    for (const unit of ['Horseman', 'Knight', 'Cataphract', 'Cuirassier'])
      await expect(panel.locator(`.production-choice[data-item="u:${unit}"]`)).toHaveCount(0);
    await expect(panel.getByRole('note').filter({ hasText: 'Aztecs train no horse-mounted units (Early Tanks, Tanks and later armour allowed)' })).toHaveCount(1);
    await shot(page, size, '02-city-no-horse-cards');
    await page.keyboard.press('Escape');
    await revealSidebar(page, 'goal');
    await page.locator('.goal-action-btn', { hasText: 'Kindle the New Fire' }).click();
    const dialog = page.locator('dialog[data-goal-dialog="Swarm"][open]');
    await expect(dialog.locator('.new-fire-torches > *')).toHaveCount(30);
    await expect(dialog.getByRole('button', { name: 'Kindle the New Fire', exact: true })).toBeVisible();
    await shot(page, size, '03-new-fire');
    await page.keyboard.press('Escape');
    await frameHall(page, city.pos, 1);
    await stageHallReport(page, [{ type: 'FlowerWars', city: city.id, owner: 0, population: city.population + 1 }]);
    const flowerCard = page.locator('.moment-card.notable');
    await expect(flowerCard).toContainText('Flower Wars');
    await expect(flowerCard).toContainText('gains 1 population');
    await expect(flowerCard.locator('.moment-vignette')).toHaveAttribute('src', /moment-growth-104/);
    await shot(page, size, '04-flower-wars');
  });
});

// Review shots for the city and glossary copy pass at both laptop and large-window sizes.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { progressFixture } from '../../src/test/progressFixture';
import { postGameSave } from '../../src/test/postGameFixture';
import { tileIndex, within } from '../../src/hex';
import { SIZES, aim, loadFixture, selectThing, shot } from './helpers';

for (const size of SIZES) test.describe(`text city terms ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('city growth and starvation explanations, and Hills tile card', async ({ page }) => {
    const setup = progressFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await selectThing(page, 'city', setup.capital);
    await page.getByTestId('city-growth').click();
    await expect(page.locator('.tip-bubble').filter({ hasText: 'The first growth costs' })).toBeVisible();
    await shot(page, size, 'city-growth-rule');
    await page.keyboard.press('Escape');

    await selectThing(page, 'city', setup.second);
    await page.getByTestId('city-growth').click();
    await expect(page.locator('.tip-bubble').filter({ hasText: 'Build a Farm or a Granary' })).toBeVisible();
    await shot(page, size, 'city-starvation-advice');
    await page.keyboard.press('Escape');

    const city = setup.fixture.cities.find((c) => c.id === setup.second)!;
    const hill = within(city.pos, 2).find((h) => setup.fixture.terrain[tileIndex(h, setup.fixture.width, setup.fixture.height)] === 'Hills'
      && !setup.fixture.units.some((u) => u.pos.q === h.q && u.pos.r === h.r));
    expect(hill).toBeDefined();
    await aim(page, hill!, page.locator('.tilecard'));
    await expect(page.locator('.tilecard')).toContainText('defence');
    await shot(page, size, 'hills-tile-defence');
  });

  test('Byzantium Levy chip', async ({ page }) => {
    const setup = progressFixture();
    setup.fixture.players[0].faction = 'Byzantium';
    setup.fixture.setup.players[0].faction = 'Byzantium';
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const levy = page.locator('.levy-counter');
    await expect(levy).toContainText('0 / 3');
    await levy.hover();
    await expect(page.locator('.tip-bubble').filter({ hasText: 'Military units 0 / 3' })).toBeVisible();
    await shot(page, size, 'levy-military-units');
  });

  test('post-game army column', async ({ page }) => {
    const { game, turn } = postGameSave('swiss-win');
    await loadFixture(page, game, turn);
    const reveal = page.getByTestId('game-reveal');
    await reveal.click({ position: { x: 6, y: 6 } });
    await expect(reveal).toHaveAttribute('data-settled', 'true');
    await page.locator('.gr-actions .btn.primary').click();
    await page.locator('[role="tab"][data-tab="stats"]').click();
    await expect(page.getByTestId('postgame-report')).toContainText('Army: highest army points reached');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'postgame-army-points');
  });
});

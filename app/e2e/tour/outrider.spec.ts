// Screen tour: a distant melee attack forecasts the fight and warns that a moving horse escapes.
// Run alone with `npx playwright test e2e/tour/outrider`. Shots land in test-results/screen-tour/<size>/outrider/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { spawnUnit, refreshVision } from '../../src/test/fixtureEngine';
import { fromOffset, distance } from '../../src/hex';
import { SIZES, aim, centred, hexPoint, loadFixture, posOf, selectThing, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('outrider: a distant attack shows its escape risk, an adjacent attack does not', async ({ page }) => {
    const setup = wallsFixture();
    const f = setup.fixture;
    const far = spawnUnit(f, 1, 'Horseman', fromOffset(14, 9), false);
    const lone = spawnUnit(f, 0, 'Spearman', fromOffset(14, 5), false);
    const near = spawnUnit(f, 1, 'Horseman', fromOffset(15, 5), false);
    refreshVision(f);
    expect(distance(f.units.find((u) => u.id === lone.id)!.pos, f.units.find((u) => u.id === near.id)!.pos)).toBe(1);
    const apart = spawnUnit(f, 0, 'Spearman', fromOffset(12, 9), false);
    refreshVision(f);
    await loadFixture(page, coreFixture(f), 2);
    // Two tiles away: the fight is available, with its escape condition on the card.
    await selectThing(page, 'unit', apart.id);
    await centred(page, apart.id);
    await page.mouse.move(5, size.height / 2);
    const p = await hexPoint(page, await posOf(page, 'unit', far.id));
    await page.mouse.move(p.x + 3, p.y + 3); await page.mouse.move(p.x, p.y, { steps: 3 });
    await page.waitForTimeout(800);
    await expect(page.getByTestId('forecast-outrider')).toHaveText('If the Horseman moves this turn, it gets away.');
    await shot(page, size, '01-outrider-distant-forecast');
    await page.mouse.click(p.x, p.y, { button: 'right' });
    await page.waitForTimeout(600);
    await expect(page.locator('.toast')).toHaveCount(0);
    await shot(page, size, '02-outrider-ordered');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ error: 'That order cannot be placed.' }));
    const toast = await page.locator('.game-error-toast').boundingBox();
    const callToAction = await page.locator('.right-sidebar .cta').boundingBox();
    expect(toast && callToAction && toast.x + toast.width <= callToAction.x, 'toast stays in the map lane').toBe(true);
    await shot(page, size, '02b-toast-clear-of-orders');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ error: null }));
    // Already beside it: the normal forecast.
    await page.mouse.move(5, size.height / 2);
    await selectThing(page, 'unit', lone.id);
    await centred(page, lone.id);
    await aim(page, await posOf(page, 'unit', near.id), page.locator('.forecast'));
    await expect(page.getByTestId('forecast-outrider')).toHaveCount(0);
    await shot(page, size, '03-outrider-adjacent-forecast');
  });
});

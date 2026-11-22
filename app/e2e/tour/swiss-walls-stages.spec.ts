import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`Swiss Walls stages ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the star fort grows through all three stages into the finished overlay', async ({ page }) => {
    const setup = wallsFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await page.mouse.move(5, size.height / 2);
    for (const [name, stored] of [['stage-1', 200], ['stage-2', 500], ['stage-3', 800], ['finished', 1000]] as const) {
      await page.evaluate(({ id, stored }) => {
        const w = window as unknown as GameWindow & { legatumMap: { redraw(): void } };
        const game = w.legatum, state = game.getState(), views = [...state.views!], v = views[state.seat];
        const city = v.cities.find((c) => c.id === id)!;
        views[state.seat] = { ...v,
          players: v.players.map((p) => p.id === city.owner ? { ...p, faction: 'Switzerland' as const } : p),
          cities: v.cities.map((c) => c.id === id ? { ...c, walls: stored === 1000,
            detail: c.detail && { ...c.detail, producing: stored === 1000 ? null : { kind: 'Building' as const, building: 'Walls' as const }, production_stored: stored, production_cost: 1000 } } : c),
        };
        game.setState({ views });
        game.getState().jumpTo(city.pos);
        w.legatumMap.redraw();
      }, { id: setup.own, stored });
      await expect(page.locator('[data-game-map]')).toBeVisible();
      await shot(page, size, `swiss-walls-${name}`);
    }
  });
});

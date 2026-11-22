import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { ring, tileIndex } from '../../src/hex';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`landmarks on the map ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('all painted works and the Courier Hall occupy visible tiles at normal zoom', async ({ page }) => {
    const s = improvementCity({ techs: ['IronWorking'] });
    const city = s.cities[0];
    await loadFixture(page, coreFixture(s), 12);
    const sites = [ring(city.pos, 2)[1], ring(city.pos, 2)[4], ring(city.pos, 2)[8], ring(city.pos, 2)[10]];
    await page.evaluate(({ sites, indices, cityId }) => {
      const w = window as unknown as GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };
      const game = w.legatum;
      const state = game.getState();
      const views = [...state.views!];
      const v = views[state.seat];
      const tiles = [...v.tiles];
      for (const [i, building] of (['GreatLibrary', 'GrandBazaar', 'RoyalArsenal', 'YamRelayStation'] as const).entries()) {
        const n = indices[i];
        tiles[n] = { ...tiles[n], terrain: 'Plains', resource: null, landmark: { building, city: cityId, of: v.me } };
      }
      views[state.seat] = { ...v, tiles };
      game.setState({ views, techOpen: false });
      w.legatumMap.camera.zoom = 0.9;
      game.getState().select(null);
      game.getState().jumpTo(sites[0]);
      w.legatumMap.redraw();
    }, { sites, indices: sites.map((h) => tileIndex(h, s.width, s.height)), cityId: city.id });
    await expect(page.locator('[data-game-map]')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'great-works-on-map');
    for (const [i, name] of ['great-library', 'grand-bazaar', 'royal-arsenal', 'courier-hall'].entries()) {
      await page.evaluate((h) => (window as unknown as GameWindow).legatum.getState().jumpTo(h), sites[i]);
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, name);
    }
    // The counter moves beside the first landmark on two successive map frames; its painting must stay anchored.
    for (const [frame, beside] of [ring(sites[0], 1)[0], ring(sites[0], 1)[1]].entries()) {
      await page.evaluate(({ at, beside }) => {
        const w = window as unknown as GameWindow & { legatumMap: { redraw(): void } };
        const game = w.legatum, state = game.getState(), views = [...state.views!], v = views[state.seat];
        const unit = v.units.find((u) => u.owner === v.me);
        if (!unit) throw new Error('landmark tour needs an own unit');
        views[state.seat] = { ...v, units: v.units.map((u) => u.id === unit.id ? { ...u, pos: beside } : u) };
        game.setState({ views });
        game.getState().jumpTo(at);
        w.legatumMap.redraw();
      }, { at: sites[0], beside });
      await shot(page, size, `landmark-unit-frame-${frame + 1}`);
    }
  });
});

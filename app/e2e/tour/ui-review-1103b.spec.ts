// UI review of train-1103, second pass: the capture and loss cards of an ordinary city that holds a world wonder.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { ring, tileIndex } from '../../src/hex';
import { SIZES, frameHall, loadFixture, shot, stageHallReport, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`ui review 1103b ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  for (const kind of ['captured', 'lost'] as const) test(`wonder city ${kind}`, async ({ page }) => {
    const s = improvementCity({ techs: ['Masonry'] });
    const city = s.cities[0];
    await loadFixture(page, coreFixture(s), 12);
    await page.evaluate(({ index, cityId }) => {
      const game = (window as unknown as GameWindow).legatum;
      const state = game.getState();
      const views = [...state.views!];
      const v = views[state.seat];
      const tiles = [...v.tiles];
      tiles[index] = { ...tiles[index], terrain: 'Plains', resource: null, landmark: { building: 'Pyramids', city: cityId, of: v.me } };
      views[state.seat] = { ...v, tiles };
      game.setState({ views, techOpen: false });
    }, { index: tileIndex(ring(city.pos, 2)[1], s.width, s.height), cityId: city.id });
    await frameHall(page, city.pos, 1.1);
    const [from, to] = kind === 'captured' ? [1, 0] : [0, 1];
    await stageHallReport(page, [
      { type: 'CityCaptured', city: city.id, from, to },
      { type: 'GreatWorkLost', player: from, of: from, building: 'Pyramids', vp: 2 },
      { type: 'GreatWorkCompleted', player: to, of: from, building: 'Pyramids', vp: 2, again: true },
    ]);
    await page.waitForTimeout(600);
    await shot(page, size, `m-${kind}-card`);
    const card = page.locator('.moment-card').first();
    await expect(card).toBeVisible();
    await page.keyboard.press('l'); // the card holds the report back; L opens it
    await shot(page, size, `m-${kind}-report`);
  });
});

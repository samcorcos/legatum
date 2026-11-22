// UI review of train-1103: the shots the tour was missing for world wonders (map art beside the painted works, tile card,
// moments for completion, capture and loss, the raze confirm in a wonder city, the expanded ledger row, the Guide chapter)
// and the Aztec rule note in view. Run alone: `npx playwright test e2e/tour/ui-review-1103`.
import { test, expect } from '../fixtures';
import { TECHS } from '../../src/data';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { swarmState } from '../../src/test/goalActionsFixture';
import { ring, tileIndex } from '../../src/hex';
import { openLateRace } from '../lateRaceSave';
import { SIZES, aim, frameHall, loadFixture, openHall, selectThing, shot, stageHallReport, type GameWindow } from './helpers';


for (const size of SIZES) test.describe(`ui review 1103 ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('world wonder landmarks beside the painted works, at three zooms, plus the tile card', async ({ page }) => {
    test.setTimeout(150_000);
    const s = improvementCity({ techs: ['IronWorking', 'Masonry'] });
    const city = s.cities[0];
    await loadFixture(page, coreFixture(s), 12);
    const sites = [ring(city.pos, 2)[1], ring(city.pos, 2)[4], ring(city.pos, 2)[8], ring(city.pos, 2)[10], ring(city.pos, 3)[6]];
    await page.evaluate(({ indices, cityId }) => {
      const game = (window as unknown as GameWindow).legatum;
      const state = game.getState();
      const views = [...state.views!];
      const v = views[state.seat];
      const tiles = [...v.tiles];
      for (const [i, building] of (['Pyramids', 'HagiaSophia', 'EiffelTower', 'IterReactor', 'RelayUplink'] as const).entries()) {
        const n = indices[i];
        tiles[n] = { ...tiles[n], terrain: 'Plains', resource: null, landmark: { building, city: cityId, of: v.me } };
      }
      views[state.seat] = { ...v, tiles };
      game.setState({ views, techOpen: false });
    }, { indices: sites.map((h) => tileIndex(h, s.width, s.height)), cityId: city.id });
    await frameHall(page, city.pos, 0.9);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'w1-all-landmarks-normal');
    await frameHall(page, city.pos, 0.4);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'w2-all-landmarks-far');
    for (const [i, name] of ['pyramids', 'hagia-sophia', 'eiffel-tower', 'iter-reactor', 'great-library-painted'].entries()) {
      await frameHall(page, sites[i], 1.3);
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `w3-close-${name}`);
    }
    await frameHall(page, sites[0], 1);
    await aim(page, sites[0], page.locator('.tilecard'));
    await shot(page, size, 'w4-pyramids-tile-card');
  });

  test('wonder moments: completed, captured by you, taken from you', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await openHall(page, 'buildable');
    await stageHallReport(page, [
      { type: 'ProductionCompleted', city: f.b, owner: 0, item: { kind: 'Wonder', building: 'Pyramids', tile: { q: f.hallPos.q + 1, r: f.hallPos.r } } },
      { type: 'GreatWorkCompleted', player: 0, of: 0, building: 'Pyramids', vp: 2, again: false },
    ]);
    await page.waitForTimeout(600);
    await shot(page, size, 'm1-wonder-completed-moment');
  });

  test('wonder capture: the captor and the loser', async ({ page, browser }) => {
    test.setTimeout(180_000);
    const f = await openHall(page, 'captured');
    await frameHall(page, f.hallPos, 1.25);
    await stageHallReport(page, [
      { type: 'CityCaptured', city: f.b, from: f.glassSeat, to: 0 },
      { type: 'GreatWorkLost', player: f.glassSeat, of: f.glassSeat, building: 'Pyramids', vp: 2 },
      { type: 'GreatWorkCompleted', player: 0, of: f.glassSeat, building: 'Pyramids', vp: 2, again: true },
    ]);
    await page.waitForTimeout(600);
    await shot(page, size, 'm2-wonder-city-captured-by-you');
    const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await context.newPage();
      const l = await openHall(p, 'lost');
      await frameHall(p, l.hallPos, 1.25);
      await stageHallReport(p, [
        { type: 'CityCaptured', city: l.b, from: 0, to: l.otherSeat },
        { type: 'GreatWorkLost', player: 0, of: 0, building: 'Pyramids', vp: 2 },
        { type: 'GreatWorkCompleted', player: l.otherSeat, of: 0, building: 'Pyramids', vp: 2, again: true },
      ]);
      await p.waitForTimeout(600);
      await shot(p, size, 'm3-wonder-city-taken-from-you');
    } finally { await context.close(); }
  });

  test('raze confirm in a captured city that holds a wonder', async ({ page }) => {
    const s = improvementCity({ techs: ['Masonry'] });
    const city = s.cities[0];
    city.founder = 1; city.isCapital = false;
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
    await selectThing(page, 'city', city.id);
    const raze = page.getByTestId('city-raze');
    await raze.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'r1-raze-control-wonder-city');
    await raze.getByRole('button', { name: 'Raze city' }).click();
    await page.getByTestId('raze-confirm').scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'r2-raze-confirm-wonder-city');
  });

  test('the ledger world wonder row opened, and the Guide chapter', async ({ page }) => {
    await openLateRace(page);
    await page.getByRole('button', { name: /^Every way to score/ }).click();
    const sheet = page.getByTestId('objective-sheet');
    await expect(sheet.locator('.sl-table')).toBeVisible();
    const row = sheet.locator('tr.sl-row.k-work').first().locator('button.sl-row-btn');
    if (await row.getAttribute('aria-expanded') !== 'true') await row.click();
    await row.scrollIntoViewIfNeeded();
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'l1-ledger-world-wonder-open');
    await page.keyboard.press('Escape');
  });

  test('the Guide world wonders section', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /The victory race/ }).click();
    const h = guide.locator('.guide-page').getByRole('heading', { name: /^World wonders/ }).first();
    await h.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'g1-guide-world-wonders');
    await guide.locator('.guide-page').evaluate((el, height) => { el.scrollTop += height * 0.5; }, size.height);
    await shot(page, size, 'g2-guide-world-wonders-rest');
  });

  test('Aztec city: the rule note in view', async ({ page }) => {
    const s = swarmState(); s.players[0].techs = [...TECHS];
    await loadFixture(page, coreFixture(s), s.turn);
    await selectThing(page, 'city', s.cities[0].id);
    const note = page.locator('.city-panel').getByRole('note').filter({ hasText: 'Aztecs train no horse-mounted units' });
    await note.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'a1-aztec-rule-note');
  });
});

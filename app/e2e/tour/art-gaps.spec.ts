// Screen tour, the art gap sweep (PR #107): the Random faction sheet's painting, a built wonder tile's card, and the
// faction goal moments that now show their building's painting. Run it alone with `npx playwright test e2e/tour/art-gaps`.
// Shots land in test-results/screen-tour/<size>/art-gaps/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { ring, tileIndex } from '../../src/hex';
import type { Event } from '../../src/types';
import { SIZES, hexPoint, loadFixture, openHall, shot, showMoment, stageHallReport, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`art gaps ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the faction sheet with Random chosen shows its painting', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    await stage.locator('[role=option][data-faction="Random"], [role=option]:has-text("Random")').first().click();
    await expect(stage.locator('.fs-sheet-name')).toContainText('Random');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-random-sheet');
    // A painted people beside it, for the set.
    await page.getByRole('option', { name: /Sparta/ }).click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '02-sparta-sheet');
  });

  test('a built wonder tile card leads with the painting', async ({ page }) => {
    const s = improvementCity({ techs: ['IronWorking'] });
    const city = s.cities[0];
    await loadFixture(page, coreFixture(s), 12);
    const sites = [ring(city.pos, 2)[1], ring(city.pos, 2)[4], ring(city.pos, 2)[8], ring(city.pos, 2)[10]];
    const kinds = ['GreatLibrary', 'GrandBazaar', 'RoyalArsenal', 'YamRelayStation'] as const;
    await page.evaluate(({ indices, cityId, kinds }) => {
      const w = window as unknown as GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };
      const game = w.legatum, state = game.getState(), views = [...state.views!], v = views[state.seat], tiles = [...v.tiles];
      for (const [i, building] of kinds.entries()) {
        const n = indices[i];
        tiles[n] = { ...tiles[n], terrain: 'Plains', resource: null, landmark: { building, city: cityId, of: v.me } } as typeof tiles[number];
      }
      views[state.seat] = { ...v, tiles };
      game.setState({ views, techOpen: false });
      w.legatumMap.camera.zoom = 1.1;
      game.getState().select(null);
      w.legatumMap.redraw();
    }, { indices: sites.map((h) => tileIndex(h, s.width, s.height)), cityId: city.id, kinds: [...kinds] });
    for (const [i, name] of ['great-library', 'grand-bazaar', 'royal-arsenal', 'yam-relay-station'].entries()) {
      await page.evaluate((h) => (window as unknown as GameWindow).legatum.getState().jumpTo(h), sites[i]);
      const spot = await hexPoint(page, sites[i]);
      await page.mouse.move(spot.x - 3, spot.y - 3);
      await page.mouse.move(spot.x, spot.y);
      await expect(page.getByTestId('tile-landmark')).toBeVisible();
      await shot(page, size, `1${i}-tile-card-${name}`);
    }
  });

  test('faction goal moments show the building painting', async ({ page }) => {
    test.setTimeout(120_000);
    await openHall(page, 'own');
    const cases: [string, string, Event[]][] = [
      ['20-yam-built', 'The Yam Network', [{ type: 'FactionGoalCompleted', player: 0, faction: 'Mongols', vp: 2 }]],
      ['21-yam-again', 'The Yam Relay Station stands again', [{ type: 'FactionGoalCompleted', player: 0, faction: 'Mongols', vp: 2, again: true }]],
      ['22-diwan', 'The Imperial Diwan', [{ type: 'FactionGoalCompleted', player: 0, faction: 'India', vp: 2 }]],
      ['23-sparta-veterans', 'The veterans are honoured', [
        { type: 'VeteranRetired', player: 0, unit: 3, kind: 'Spearman', city: 1, retired: 3, of: 3 },
        { type: 'FactionGoalCompleted', player: 0, faction: 'Sparta', vp: 2 }]],
    ];
    for (const [name, title, events] of cases) {
      await stageHallReport(page, events);
      await showMoment(page, title);
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, name);
      const box = await page.locator('.moment-card .moment-wonder-art').first().boundingBox();
      expect(box, `${title} has a painting`).toBeTruthy();
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ report: null }));
      await page.evaluate(() => document.querySelector<HTMLButtonElement>('.moment-lane button[aria-label="Dismiss celebration"]')?.click());
    }
  });
});

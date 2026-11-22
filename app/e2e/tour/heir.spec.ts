// Screen tour, the Heir's public birth (real engine): a rival's Byzantium trains its Heir out of sight and every empire is told (the card names the
// city and offers the map; the Heir wears a halo and a tag in the fog and a gilt ring on the minimap; the tile card says what capturing him
// scores); and the Byzantine view of the same birth (the Heir card with the city in its kicker). Run alone with
// `npx playwright test e2e/tour/heir`. Shots land in test-results/screen-tour/<size>/heir/.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { heirState } from '../../src/test/heirState';
import { OWN_HEIR_BIRTH_TITLE, RAIDER_SACK_KICKER } from '../../src/celebrate/moments';
import type { Event } from '../../src/types';
import { SIZES, aim, loadFixture, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

type MapWindow = GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('a rival\'s Heir is born in fog: the card, then the halo, the minimap dot and the tile card', async ({ page }) => {
    test.setTimeout(180_000);
    await page.clock.install();
    const fx = heirState('rival');
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await page.mouse.move(5, size.height / 2);
    // Hold the short-lived story while this tour reads and photographs it on slow engines.
    await page.clock.pauseAt(await page.evaluate(() => Date.now()) + 2000);
    await endPlanning(page);
    // Turn resolution is scheduled 30 ms later so its busy overlay can paint.
    await page.clock.runFor(100);
    const card = page.locator('.moment-card', { hasText: 'Heir born' });
    await expect(card).toContainText('Heir born', { timeout: 30_000 });
    await expect(card).toContainText('Byzantium’s Heir');
    await expect(card).toContainText('Born in Constantinople');
    await expect(card).toContainText('capture him to score 2 VP');
    // The buttons stay on screen at every window size.
    const area = (await card.boundingBox())!;
    const show = card.getByRole('button', { name: 'Show on map' });
    const box = (await show.boundingBox())!;
    expect(box.y + box.height).toBeLessThanOrEqual(Math.min(area.y + area.height, size.height) + 0.5);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-rival-heir-card');
    // Show on map centres the camera on the Heir, who stands in the fog.
    await show.click();
    await page.keyboard.press('Escape');
    await page.clock.resume();
    await expect(card).toHaveCount(0);
    const heir = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; const h = v.units.find((u) => u.kind === 'HeirToTheThrone')!; return { pos: h.pos, visible: v.tiles[h.pos.r * v.width + h.pos.q]?.visible ?? false }; });
    expect(heir.visible).toBe(false);
    await page.evaluate(() => { const w = window as unknown as MapWindow; w.legatumMap.camera.zoom = 1.4; w.legatumMap.redraw(); });
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: false }));
    await page.waitForTimeout(900);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '02-heir-in-fog-map');
    // Nothing selected: with a unit of yours selected, hovering a rival at Peace shows the Peace hint instead.
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    const heirTile = page.getByTestId('heir-tile');
    await aim(page, heir.pos, heirTile);
    await expect(heirTile).toContainText('Be at War, then capture him to score 2 VP');
    await expect(heirTile).not.toContainText("Byzantium's Heir");
    await page.waitForTimeout(300);
    await shot(page, size, '03-heir-tile-card');
    await page.evaluate(() => { const w = window as unknown as MapWindow; w.legatumMap.camera.zoom = 0.5; w.legatumMap.redraw(); });
    await page.mouse.move(5, size.height / 2);
    await page.waitForTimeout(500);
    await shot(page, size, '04-heir-far-zoom');
  });

  test('your own Heir is born: one Heir to the Throne card that names the city', async ({ page }) => {
    test.setTimeout(180_000);
    const fx = heirState('own');
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await page.mouse.move(5, size.height / 2);
    await endPlanning(page);
    const card = page.locator('.moment-card', { hasText: OWN_HEIR_BIRTH_TITLE });
    await expect(card).toContainText('Your Heir is born in Constantinople', { timeout: 30_000 });
    await expect(card).toContainText('his position is public');
    await expect(card).toContainText('The promise has a face. Keep him.');
    await expect(page.locator('.moment-card')).toHaveCount(1);
    const area = (await card.boundingBox())!;
    const show = (await card.getByRole('button', { name: 'Show on map' }).boundingBox())!;
    expect(show.y + show.height).toBeLessThanOrEqual(Math.min(area.y + area.height, size.height) + 0.5);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '05-own-heir-card');
  });

  test('a raider sees the city sack card', async ({ page }) => {
    const fx = heirState('rival');
    fx.state.players[0].faction = 'Celts';
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    const events: Event[] = [{ type: 'CityRazed', city: fx.rivalCity, from: 2, by: 0, at: fx.state.cities.find((c) => c.id === fx.rivalCity)!.pos, sack_gold: 80 }];
    await page.evaluate(({ events }) => {
      const st = (window as unknown as GameWindow).legatum.getState();
      const view = st.views![st.seat];
      st.set({ report: { turn: view.turn, hash: 'sack', events: [events], lines: [[]] }, reportOpen: false });
    }, { events });
    const card = page.locator('.moment-card', { hasText: RAIDER_SACK_KICKER });
    await expect(card).toContainText('Gold taken:', { timeout: 30_000 });
    await shot(page, size, '06-raider-city-sacked-card');
  });
});

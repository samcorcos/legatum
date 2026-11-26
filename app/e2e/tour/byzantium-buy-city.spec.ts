// Screen tour, Byzantium's Buy a city and the fall of Constantinople (real engine): the Byzantine player's tile card and city
// panel on a city it can buy (price with every line, one primary "Make an offer"), the queued offer; the seller's view of a
// running offer (the banner's countdown chip, the city panel, the turn report's notice, the capital's public line); and the
// faction sheet. Run alone with `npx playwright test e2e/tour/byzantium-buy-city`. Shots land in
// test-results/screen-tour/<size>/byzantium-buy-city/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { buyCityState } from '../../src/test/buyCityState';
import { SIZES, aim, loadFixture, openReport, primaries, selectThing, shot, stageHallReport, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

type MapWindow = GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };
const cityPos = (page: import('@playwright/test').Page, id: number) => page.evaluate((id) => {
  const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat].cities.find((c) => c.id === id)!.pos;
}, id);
const zoom = (page: import('@playwright/test').Page, z: number) => page.evaluate((z) => { const w = window as unknown as MapWindow; w.legatumMap.camera.zoom = z; w.legatumMap.redraw(); }, z);

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('Byzantium: the price on the tile card, the offer on the city panel, then queued', async ({ page }) => {
    test.setTimeout(120_000);
    const fx = buyCityState('buyer');
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await selectThing(page, 'city', fx.pylos);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await zoom(page, 1.3);
    const card = page.getByTestId('tile-city-buy');
    await aim(page, await cityPos(page, fx.pylos), card);
    await expect(card).toContainText(/^ ?Buy for \d+ Gold \(Starting price 150/);
    await shot(page, size, '01-byzantium-tile-card-buyable');
    // The capital's public line, from its own player's side.
    await selectThing(page, 'city', fx.constantinople);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    const fall = page.getByTestId('tile-capital-fall');
    await aim(page, await cityPos(page, fx.constantinople), fall);
    await expect(fall).toContainText('Lose this city and you lose every city');
    await shot(page, size, '02-byzantium-own-capital-tile-card');
    await selectThing(page, 'city', fx.pylos);
    await page.mouse.move(5, size.height / 2);
    const offer = page.getByTestId('buy-city-offer');
    await expect(offer).toHaveText(/^Make an offer \(\d+ Gold\)$/);
    await offer.scrollIntoViewIfNeeded();
    // Pillar 9: the offer is the one primary; the turn's call to action steps back.
    expect(await primaries(page)).toBe(1);
    await shot(page, size, '03-byzantium-city-panel-offer');
    await offer.click();
    await expect(page.getByTestId('buy-city-queued')).toContainText('Queued: your offer for Pylos');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '04-byzantium-offer-queued');
  });

  test('the seller: the banner countdown, the city panel, the turn report notice and the capital line', async ({ page }) => {
    test.setTimeout(120_000);
    const fx = buyCityState('seller');
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await stageHallReport(page, fx.events);
    const report = await openReport(page);
    await expect(report).toContainText('Byzantium is buying Pylos for 600 Gold, at the end of turn 14. Move military units in or build Walls to raise the price, or declare War.');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '05-seller-turn-report-notice');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: false }));
    await selectThing(page, 'city', fx.pylos);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await zoom(page, 1.3);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '06-banner-being-bought');
    const offerLine = page.getByTestId('tile-city-offer');
    await aim(page, await cityPos(page, fx.pylos), offerLine);
    await expect(offerLine).toContainText('Byzantium is buying Pylos');
    await shot(page, size, '07-seller-tile-card');
    await selectThing(page, 'city', fx.pylos);
    await page.mouse.move(5, size.height / 2);
    const panel = page.getByTestId('city-offer');
    await panel.scrollIntoViewIfNeeded();
    await expect(page.getByTestId('city-offer-options')).toHaveText('Move military units in or build Walls to raise the price, or declare War.');
    await shot(page, size, '08-seller-city-panel');
    await selectThing(page, 'city', fx.constantinople);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    const fall = page.getByTestId('tile-capital-fall');
    await aim(page, await cityPos(page, fx.constantinople), fall);
    await expect(fall).toHaveText(/Take it and you take every Byzantine city\./);
    await shot(page, size, '09-rival-byzantine-capital-tile-card');
  });

  test('the faction sheet: Buy a city and the fall of Constantinople', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    await page.getByRole('option', { name: /^Byzantium / }).click();
    const sheet = page.getByRole('dialog', { name: 'Choose your empire' }).getByTestId('faction-sheet');
    await expect(sheet).toContainText('Buy a rival’s city with Gold.');
    await expect(sheet).toContainText('unless the Heir shelters in another one');
    expect(await sheet.evaluate((el) => el.scrollWidth <= el.clientWidth + 1)).toBe(true);
    await page.mouse.move(5, size.height - 5);
    await shot(page, size, '10-faction-sheet');
  });
});

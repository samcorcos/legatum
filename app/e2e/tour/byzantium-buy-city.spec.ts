// Screen tour, Byzantium's Buy a city and the fall of Constantinople (real engine): the Byzantine player's tile card and city
// panel on a city it can buy (price with every line, one primary "Make an offer"), the queued offer; the seller's view of a
// running offer (the banner's countdown chip, the city panel, the turn report's notice, the capital's public line); and the
// faction sheet. Run alone with `npx playwright test e2e/tour/byzantium-buy-city`. Shots land in
// test-results/screen-tour/<size>/byzantium-buy-city/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { buyCityState } from '../../src/test/buyCityState';
import { SIZES, aim, loadFixture, parkMouse, primaries, selectThing, shot, stageHallReport, type GameWindow } from './helpers';

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
    await expect(card).toHaveText(/^ ?Buy for \d+ Gold\. Click the city to make an offer\.$/);
    await shot(page, size, '01-byzantium-tile-card-buyable');
    // The capital's public line, from its own player's side.
    await selectThing(page, 'city', fx.constantinople);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    const fall = page.getByTestId('tile-capital-fall');
    await aim(page, await cityPos(page, fx.constantinople), fall);
    await expect(fall).toContainText('Lose this city and you lose every city');
    await shot(page, size, '02-byzantium-own-capital-tile-card');
    await selectThing(page, 'city', fx.pylos);
    await parkMouse(page);
    const offer = page.getByTestId('buy-city-offer');
    await expect(offer).toHaveText(/^Make an offer \(\d+ Gold\)$/);
    await shot(page, size, '03a-byzantium-city-panel-as-opened');
    await offer.scrollIntoViewIfNeeded();
    // Pillar 9: the offer is the one primary; the turn's call to action steps back.
    expect(await primaries(page)).toBe(1);
    await shot(page, size, '03-byzantium-city-panel-offer');
    await offer.click();
    await expect(page.getByTestId('buy-city-queued')).toContainText('Queued: your offer for Pylos');
    await parkMouse(page);
    await shot(page, size, '04-byzantium-offer-queued');
  });

  test('the seller: the banner countdown, the city panel, the turn report notice and the capital line', async ({ page }) => {
    test.setTimeout(120_000);
    const fx = buyCityState('seller');
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await stageHallReport(page, fx.events);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
    const report = page.getByRole('complementary', { name: 'Turn report' });
    await expect(report).toContainText('Byzantium will buy Pylos for 600 Gold at the end of turn 14.');
    await parkMouse(page);
    await shot(page, size, '05-seller-turn-report-notice');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: false }));
    await selectThing(page, 'city', fx.pylos);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await zoom(page, 1.3);
    await parkMouse(page);
    await shot(page, size, '06-banner-being-bought');
    const offerLine = page.getByTestId('tile-city-offer');
    await aim(page, await cityPos(page, fx.pylos), offerLine);
    await expect(offerLine).toContainText('Byzantium pays you 600 Gold for Pylos at the end of turn 14. To keep it, move');
    await shot(page, size, '07-seller-tile-card');
    await selectThing(page, 'city', fx.pylos);
    await parkMouse(page);
    const panel = page.getByTestId('city-offer');
    await panel.scrollIntoViewIfNeeded();
    await expect(panel).toContainText('Byzantium pays you 600 Gold for Pylos at the end of turn 14. To keep it, move');
    await expect(panel).toContainText('or declare War.');
    await expect(page.getByTestId('city-offer-declare-war')).toBeVisible();
    await shot(page, size, '08-seller-city-panel');
    // The warning is an alarm at the top of This Turn, whatever is selected.
    const alarm = page.getByTestId('offer-alerts');
    await expect(alarm).toContainText('Byzantium is buying Pylos');
    await alarm.scrollIntoViewIfNeeded();
    await shot(page, size, '08a-seller-this-turn-alarm');
    await selectThing(page, 'city', fx.constantinople);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    const fall = page.getByTestId('tile-capital-fall');
    await aim(page, await cityPos(page, fx.constantinople), fall);
    await expect(fall).toHaveText(/Take it and you take every Byzantine city\./);
    await shot(page, size, '09-rival-byzantine-capital-tile-card');
  });

  test('Byzantium blocked: short of Gold, at War, and a capital', async ({ page }) => {
    test.setTimeout(120_000);
    const fx = buyCityState('buyer');
    fx.state.players[fx.byzantium].gold = 120;
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await selectThing(page, 'city', fx.pylos);
    await parkMouse(page);
    await page.getByTestId('city-buy').scrollIntoViewIfNeeded();
    await shot(page, size, '11-byzantium-city-panel-short-of-gold');
    await selectThing(page, 'city', fx.sparta);
    await parkMouse(page);
    await page.getByTestId('city-buy').scrollIntoViewIfNeeded().catch(() => {});
    await shot(page, size, '12-byzantium-rival-capital-not-for-sale');
  });

  test('Byzantium blocked: at War with the owner', async ({ page }) => {
    test.setTimeout(120_000);
    const fx = buyCityState('buyer');
    const r = fx.state.diplomacy!.relations.find((r) => r.a === Math.min(fx.byzantium, fx.seller) && r.b === Math.max(fx.byzantium, fx.seller))!;
    Object.assign(r, { state: 'War', effective_turn: 10, version: 1, war_id: 1, declarers: [fx.seller] });
    fx.state.diplomacy!.next_war_id = 2;
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await selectThing(page, 'city', fx.pylos);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await zoom(page, 1.3);
    const card = page.getByTestId('tile-city-buy');
    await aim(page, await cityPos(page, fx.pylos), card);
    await expect(card).toContainText('Make Peace with Sparta to buy it.');
    await shot(page, size, '13-byzantium-tile-card-at-war');
    await selectThing(page, 'city', fx.pylos);
    await parkMouse(page);
    await page.getByTestId('city-buy').scrollIntoViewIfNeeded();
    await shot(page, size, '14-byzantium-city-panel-at-war');
  });

  test('Byzantium: its own offer running', async ({ page }) => {
    test.setTimeout(120_000);
    const fx = buyCityState('buyer');
    fx.state.players[fx.byzantium].cityOffer = { city: fx.pylos, seller: fx.seller, escrow: 600, base: 480, started: 11, ends_turn: 14 };
    fx.state.players[fx.byzantium].gold -= 600;
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await selectThing(page, 'city', fx.pylos);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await zoom(page, 1.3);
    await parkMouse(page);
    await shot(page, size, '15-buyer-banner-being-bought');
    const offerLine = page.getByTestId('tile-city-offer');
    await aim(page, await cityPos(page, fx.pylos), offerLine);
    await shot(page, size, '16-buyer-tile-card');
    await selectThing(page, 'city', fx.pylos);
    await parkMouse(page);
    await page.getByTestId('city-offer').scrollIntoViewIfNeeded();
    await shot(page, size, '17-buyer-city-panel-offer-running');
  });

  for (const who of ['buyer', 'seller'] as const) test(`the advisor: the ${who}'s tip`, async ({ page }) => {
    test.setTimeout(120_000);
    // The first-game hints are done (a turn-12 game), so the advisor is on to its strategy tips.
    await page.addInitScript(() => {
      const set = Storage.prototype.setItem;
      Storage.prototype.setItem = function (k: string, v: string) {
        if (k === 'legatum.first-game-hints.v2') { const p = JSON.parse(v); p.seen = [...new Set([...(p.seen ?? []), 'welcome', 'report', 'race'])]; v = JSON.stringify(p); }
        return set.call(this, k, v);
      };
    });
    const fx = buyCityState(who);
    await loadFixture(page, coreFixture(fx.state), fx.turn, true);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await parkMouse(page);
    await shot(page, size, who === 'buyer' ? '18-advisor-buy-tip' : '19-advisor-offer-tip');
  });

  test('the faction sheet: Buy a city and the fall of Constantinople', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    await page.getByRole('option', { name: /^Byzantium / }).click();
    const sheet = page.getByRole('dialog', { name: 'Choose your empire' }).getByTestId('faction-sheet');
    await expect(sheet).toContainText('Buy a rival’s city with Gold.');
    await expect(sheet).toContainText('unless your Heir is in another of your cities');
    expect(await sheet.evaluate((el) => el.scrollWidth <= el.clientWidth + 1)).toBe(true);
    await page.mouse.move(5, size.height - 5);
    await shot(page, size, '10-faction-sheet');
  });
});

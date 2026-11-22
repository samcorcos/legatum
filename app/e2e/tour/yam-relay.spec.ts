// Screen tour, the Yam Relay Station and roads. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/yam-relay`. Shots land in test-results/screen-tour/<size>/yam-relay/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { roadsFixture } from '../../src/test/roadsFixture';
import { hallFixture } from '../../src/test/hallFixture';
import { SIZES, closeReportIfOpen, dismissCelebrations, expectRaceWhole, frameHall, hexPoint, loadFixture, openHall, selectThing, shot, stageHallReport, type GameWindow } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });



  // Faction goals 5: the Mongols' Courier Hall (Portolan rose landmark, build card, held goal, capture).
  test('Yam Relay Station: held in your own cities (a walled one too), at normal and far zoom', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await openHall(page, 'own');
    await frameHall(page, f.hallPos, 1.25);
    await shot(page, size, '95a-hall-own-map');
    await frameHall(page, f.hallPos, 1.7);
    await shot(page, size, '95b-hall-own-map-close');
    await frameHall(page, f.hallPos, 0.55);
    await shot(page, size, '95c-hall-own-map-far');
    // The Hall's own city lists the standing building and its chosen tile.
    await selectThing(page, 'city', f.b);
    const built = page.locator('.city-panel .cp-foot');
    await built.scrollIntoViewIfNeeded();
    await expect(built).toContainText('Yam Relay Station');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '95d-hall-city-card-built');
    // Standings now fold into the sidebar's shared scroller. Open them to inspect the held token.
    await page.locator('.sidebar-standings > summary').click();
    await page.locator('.panel.vp .vhc-tok[data-held]').first().hover();
    await expect(page.locator('.vhc-pop .vh-card.k-goal .vh-held')).toContainText('You hold the Yam Relay Station');
    await shot(page, size, '95e-hall-race-token-card');
    await page.mouse.move(5, size.height / 2);
    // The goal slip on the objectives sheet says where it stands and shows it on the map.
    await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'All objectives', exact: true }).click();
    const sheet = page.getByTestId('objective-sheet');
    await expect(sheet).toBeVisible();
    await sheet.locator('.sl-row.k-heldGoal button').click();
    const locate = sheet.locator('#sl-detail-held-goal .sl-goals li.me').getByRole('button', { name: 'Show on map' });
    await expect(locate).toBeVisible();
    await shot(page, size, '95f-hall-goal-slip');
    await locate.click();
    await expect(sheet).toBeHidden();
  });

  test('Yam Relay Station roads: drawn from the rose city at both zooms, on the tile card and in a move preview', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await openHall(page, 'own');
    await frameHall(page, f.hallPos, 1.25);
    await shot(page, size, '96a-roads-near');
    await frameHall(page, f.hallPos, 0.55);
    await shot(page, size, '96b-roads-far');
    await frameHall(page, f.hallPos, 1.25);
    // A road tile beside a city: its card says Road and what it costs.
    const road = await page.evaluate((hall) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      const near = v.tiles.map((t, i) => ({ t, i })).filter(({ t }) => t.road && t.terrain && t.terrain !== 'Ocean')
        .map(({ i }) => ({ q: (i % v.width) - ((Math.floor(i / v.width) - (Math.floor(i / v.width) & 1)) / 2), r: Math.floor(i / v.width) }))
        .filter((h) => !v.cities.some((c) => c.pos.q === h.q && c.pos.r === h.r) && Math.max(Math.abs(h.q - hall.q), Math.abs(h.r - hall.r)) <= 3)
        .sort((a, b) => Math.abs(a.q - hall.q) + Math.abs(a.r - hall.r) - (Math.abs(b.q - hall.q) + Math.abs(b.r - hall.r)));
      return near[0];
    }, f.hallPos);
    expect(road, 'a road tile near the Hall').toBeTruthy();
    const spot = await hexPoint(page, road);
    await page.mouse.move(spot.x - 3, spot.y - 3);
    await page.mouse.move(spot.x, spot.y);
    await expect(page.locator('.tilecard')).toContainText(/Road · move cost 1/);
    await shot(page, size, '96c-road-tile-card');
    // Your Scout previews a trip along the road.
    await selectThing(page, 'unit', f.scout);
    await page.mouse.move(spot.x - 3, spot.y - 3);
    await page.mouse.move(spot.x, spot.y);
    await shot(page, size, '96d-road-move-preview');
  });

  test('Roadworks: the build card, then the roads it lays to every city within 8 hexes', async ({ page }) => {
    test.setTimeout(150_000);
    const offer = roadsFixture({ hall: false, works: 'offer', players: 2, width: 40, height: 26, seed: '12', extra: 8 });
    await loadFixture(page, coreFixture(offer.fixture), 40);
    await selectThing(page, 'city', offer.hall.id);
    const card = page.locator('.city-panel .production-choice[data-item="b:Roadworks"]');
    await card.scrollIntoViewIfNeeded();
    await expect(card).toContainText('Roadworks');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '97a-roadworks-build-card');
    const built = roadsFixture({ hall: false, works: 'built', players: 2, width: 40, height: 26, seed: '12', extra: 8 });
    const p2 = await page.context().newPage();
    await p2.setViewportSize({ width: size.width, height: size.height });
    await loadFixture(p2, coreFixture(built.fixture), 40);
    await frameHall(p2, built.hallPos, 0.9);
    await shot(p2, size, '97b-roadworks-roads');
    await p2.close();
  });

  test('Yam Relay Station: on a walled city the plaque sits inside the wall ring', async ({ page }) => {
    const f = await openHall(page, 'ownWalled');
    await frameHall(page, f.hallPos, 1.7);
    await shot(page, size, '95g-hall-walled-city-close');
    await frameHall(page, f.hallPos, 1.25);
    await shot(page, size, '95h-hall-walled-city');
  });

  test('Yam Relay Station: a rival\'s Hall seen from another empire, with its open road', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await openHall(page, 'rival');
    await frameHall(page, f.hallPos, 1.25);
    await expectRaceWhole(page);
    await shot(page, size, '95i-hall-rival-map');
    await frameHall(page, f.hallPos, 0.55);
    await shot(page, size, '95j-hall-rival-map-far');
    // Your Scout on their land: the open road, not a closed border.
    await selectThing(page, 'unit', f.scout);
    await expect(page.getByTestId('open-road-note')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '95k-hall-open-road-unit');
    await page.locator('.sidebar-standings > summary').click();
    await page.locator('.panel.vp .vhc-tok[data-held]').first().hover();
    await page.mouse.move(5, size.height / 2);
    await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'All objectives', exact: true }).click();
    await expect(page.getByTestId('objective-sheet')).toBeVisible();
    await shot(page, size, '95l-hall-rival-goal-slip');
  });

  test('Yam Relay Station: captured by you, the moment and the report line', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await openHall(page, 'captured');
    await frameHall(page, f.hallPos, 1.25);
    await shot(page, size, '95m-hall-captured-map');
    await stageHallReport(page, [
      { type: 'CityCaptured', city: f.b, from: f.glassSeat, to: 0 },
      { type: 'FactionGoalLost', player: f.glassSeat, faction: 'Mongols', vp: 2 },
      { type: 'FactionGoalCompleted', player: 0, faction: 'Mongols', vp: 2, again: true },
    ]);
    await expect(page.locator('.moment-card').first()).toContainText('The Yam Relay Station is taken');
    await shot(page, size, '95n-hall-capture-moment');
    await dismissCelebrations(page);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
    await expect(page.getByRole('complementary', { name: 'Turn report' })).toContainText('You took the Yam Relay Station');
    await shot(page, size, '95o-hall-capture-report');
  });

  test('Yam Relay Station: taken from you, the build card says retake the city', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await openHall(page, 'lost');
    await frameHall(page, f.hallPos, 1.25);
    await shot(page, size, '95p-hall-lost-map');
    await stageHallReport(page, [
      { type: 'CityCaptured', city: f.b, from: 0, to: f.otherSeat },
      { type: 'FactionGoalLost', player: 0, faction: 'Mongols', vp: 2 },
      { type: 'FactionGoalCompleted', player: f.otherSeat, faction: 'Mongols', vp: 2, again: true },
    ]);
    await expect(page.locator('.moment-card').first()).toContainText('The Yam Relay Station is taken from you');
    await shot(page, size, '95q-hall-lost-moment');
    await dismissCelebrations(page);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
    await expect(page.getByRole('complementary', { name: 'Turn report' })).toContainText('took the Yam Relay Station');
    await shot(page, size, '95r-hall-lost-report');
    await closeReportIfOpen(page);
    await selectThing(page, 'city', f.a);
    const locked = page.locator('.city-panel [aria-label="Yam Relay Station"] .hint');
    await locked.scrollIntoViewIfNeeded();
    await expect(locked).toContainText('retake the city instead');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '95s-hall-build-card-retake');
  });

  test('Yam Relay Station: the build card (available, locked while building elsewhere) and its completion moment', async ({ page, browser }) => {
    test.setTimeout(180_000);
    const f = hallFixture('buildable');
    f.fixture.players[0].gold = 2000;
    await loadFixture(page, coreFixture(f.fixture), 12);
    await selectThing(page, 'city', f.a);
    const card = page.locator('.city-panel .production-choice[data-item^="w:CourierHall:"]').first();
    await card.scrollIntoViewIfNeeded();
    await expect(card.locator('.prod-cost')).toContainText(/\d{3}/);
    await expect(card).toContainText('every empire may cross all of the holder\'s land at Peace');
    await expect(card.locator('.btn-buy')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '95t-hall-build-card-available');
    // One card, not one per tile (Civ's district placement): Build lights the eligible tiles on the map.
    const yam = page.locator('.city-panel [aria-label="Yam Relay Station"]');
    await expect(yam.locator('.production-choice')).toHaveCount(1);
    await expect(card.locator('.prod-where')).toHaveText('Choose its tile on the map');
    await card.locator('button.prod').click();
    await expect(page.getByTestId('city-placement')).toBeVisible();
    // Escape cancels the placement; Build opens it again.
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('city-placement')).toBeHidden();
    await yam.locator('button.queue-add').click();
    await expect(page.getByTestId('city-placement')).toContainText('Queue Yam Relay Station');
    await page.getByTestId('city-placement').getByRole('button', { name: /Cancel.*Esc/ }).click();
    await expect(page.getByTestId('city-placement')).toBeHidden();
    await expect(card.locator('.btn-buy')).toBeEnabled();
    await card.locator('.btn-buy').click();
    await expect(page.getByTestId('city-placement')).toContainText('Buy Yam Relay Station');
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('city-placement')).toBeHidden();
    await yam.locator('button.prod').click();
    await expect(page.getByTestId('city-placement')).toBeVisible();
    const lit = await page.evaluate(() => (window as unknown as { legatumMap: { place: { candidates: { q: number; r: number }[] } | null } }).legatumMap.place?.candidates ?? []);
    expect(lit.length, 'eligible tiles are lit').toBeGreaterThan(1);
    const pick = lit[0];
    // Centre the pick without dropping the city (frameHall deselects), so the placement stays open.
    await page.evaluate((hex) => (window as unknown as GameWindow).legatum.getState().jumpTo(hex), pick);
    await expect(page.getByTestId('city-placement')).toBeVisible();
    const spot = await hexPoint(page, pick);
    await page.mouse.move(spot.x - 3, spot.y - 3);
    await page.mouse.move(spot.x, spot.y);
    await expect(page.locator('.tilecard')).toContainText('Click to place it here');
    await shot(page, size, '95t2-hall-placing');
    await page.mouse.click(spot.x, spot.y);
    // Build moves on to the next decision, so come back to the city: the card now names its tile and cost.
    await selectThing(page, 'city', f.a);
    await expect(yam.locator('.production-choice')).toHaveAttribute('data-item', `w:CourierHall:${pick.q},${pick.r}`);
    await expect(yam.locator('.prod-where.site-cost')).toContainText('gives up');
    // The placed tile stays marked on the map.
    expect(await page.evaluate((hex) => (window as unknown as { legatumMap: { improve?: { sites: { q: number; r: number }[] } | null } }).legatumMap.improve?.sites.some((h) => h.q === hex.q && h.r === hex.r), pick)).toBe(true);
    for (const line of await yam.locator('.prod-where').all()) expect(await line.evaluate((el) => el.scrollWidth <= el.clientWidth + 1), 'a card line is whole').toBe(true);
    const title = await yam.locator('button.prod').getAttribute('title');
    expect(title!.split('Only Mongols can build it').length - 1, 'the card copy appears once in the tooltip').toBe(1);
    await yam.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '95t3-hall-placed');
    // The placed tile keeps its ring on the map while the city is open: centre it and look.
    await page.evaluate((hex) => (window as unknown as GameWindow).legatum.getState().jumpTo(hex), pick);
    await hexPoint(page, pick);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '95t4-hall-placed-ring');
    // The panel's head names what is now being produced: with a placed tile it must not be cut.
    await page.locator('.city-panel').evaluate((el) => { el.scrollTop = 0; });
    // The name of what is being produced ("Yam Relay Station W") is whole inside its label, beside the turns.
    const label = await page.locator('.cp-production .cp-now-name').evaluate((el) => ({ nameRight: el.getBoundingClientRect().right, labelRight: el.closest('.pbar-label')!.getBoundingClientRect().right, over: el.scrollWidth - el.clientWidth }));
    expect(label.nameRight, 'the producing name is not cut by its label').toBeLessThanOrEqual(label.labelRight + 1);
    expect(label.over).toBeLessThanOrEqual(0);
    await shot(page, size, '95t5-hall-placed-panel-top');
    await stageHallReport(page, [
      { type: 'ProductionCompleted', city: f.b, owner: 0, item: { kind: 'Wonder', building: 'CourierHall', tile: { q: f.hallPos.q + 1, r: f.hallPos.r } } },
      { type: 'FactionGoalCompleted', player: 0, faction: 'Mongols', vp: 2 },
    ]);
    await expect(page.locator('.moment-card').first()).toContainText('The Yam Network');
    await shot(page, size, '95u-hall-built-moment');
    const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await context.newPage();
      const g = await openHall(p, 'building');
      await selectThing(p, 'city', g.a);
      const locked = p.locator('.city-panel [aria-label="Yam Relay Station"] .hint');
      await locked.scrollIntoViewIfNeeded();
      await expect(locked).toContainText('already being built');
      await p.mouse.move(5, size.height / 2);
      await shot(p, size, '95v-hall-build-card-locked-building');
    } finally { await context.close(); }
  });
});

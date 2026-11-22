import { revealSidebar } from '../sidebarAssertions';
// Screen tour, walls, sieges, forecasts and unit promotion. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/combat`. Shots land in test-results/screen-tour/<size>/combat/.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { cityShieldFixture } from '../../src/test/cityShieldFixture';
import { combinedAssaultFixture } from '../../src/test/combinedAssaultFixture';
import { reinforcementFixture } from '../../src/test/reinforcementFixture';
import { siegeFixture } from '../../src/test/siegeFixture';
import { promotionFixture } from '../../src/test/promotionFixture';
import { contextFromViews, formatEvents } from '../../src/format';
import { SIZES, aim, centred, closeReportIfOpen, dismissCelebrations, expectDockWhole, expectRaceWhole, expectWhole, hexPoint, loadFixture, openReport, posOf, primaries, scrollWithinReport, selectThing, shot, type GameWindow } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });


  test('walls: city panels and attack forecasts against walled and unwalled cities', async ({ page }) => {
    const setup = wallsFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const fire = page.getByTestId('city-fire').first();
    await selectThing(page, 'city', setup.walled);
    await expect(fire).toContainText('Walls: fires');
    // An attack at hand is the only primary: the turn's call to action steps back.
    await expect(page.getByRole('button', { name: /^Attack with/ })).toBeVisible();
    expect(await primaries(page)).toBe(1);
    await revealSidebar(page, 'standings');
    // Race rows: the roundel keeps its own column, clear of the name, and the name clear of the tokens.
    const clash = await page.locator('.vp-row').evaluateAll((rows) => rows.some((r) => {
      const who = r.querySelector('.vhc-who')!.getBoundingClientRect(), name = r.querySelector('.vp-label')!.getBoundingClientRect(), holds = r.querySelector('.vhc-holds')!.getBoundingClientRect();
      return who.width < 20 || who.right > name.left + 1 || (holds.width > 0 && name.right > holds.left + 1 && Math.abs(name.top - holds.top) < 4);
    }));
    expect(clash).toBe(false);
    await expectRaceWhole(page);
    await shot(page, size, '19-city-walled');
    await selectThing(page, 'city', setup.open);
    await expect(fire).toContainText('No Walls');
    await shot(page, size, '20-city-unwalled');

    const forecast = page.locator('.forecast');
    await selectThing(page, 'unit', setup.nearOpen);
    await centred(page, setup.nearOpen);
    await aim(page, await posOf(page, 'city', setup.open), forecast);
    await expectWhole(forecast);
    await shot(page, size, '21-forecast-unwalled');
    await page.mouse.move(5, size.height / 2);
    await selectThing(page, 'unit', setup.nearWalled);
    await centred(page, setup.nearWalled);
    await aim(page, await posOf(page, 'city', setup.walled), forecast);
    await expectWhole(forecast);
    await shot(page, size, '22-forecast-walled');
  });

  test('city shield: forecasts against a garrisoned unwalled city', async ({ page }) => {
    const setup = cityShieldFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const forecast = page.locator('.forecast');
    const garrison = page.getByTestId('forecast-assault-garrison');
    for (const [unit, name] of [[setup.nearOpen, '21c-forecast-garrisoned-melee'], [setup.archer, '21d-forecast-garrisoned-ranged']] as const) {
      await selectThing(page, 'unit', unit);
      await centred(page, unit);
      // The Archer aims a ranged strike (the Ranged Attack action, R).
      if (unit === setup.archer) await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().setMode('ranged'));
      await aim(page, await posOf(page, 'city', setup.open), forecast);
      // One card shows the city, the garrison under it and whether the city is captured this turn (user, 2026-10-19, T187 Lugano).
      await expect(garrison).toContainText('Archer garrison');
      await expect(page.getByTestId('forecast-assault-city')).toContainText('→');
      await expect(page.getByTestId('forecast-assault-capture')).toContainText(unit === setup.archer ? 'A shot cannot capture a city.' : 'not captured this turn');
      if (unit === setup.archer) await expect(forecast).toContainText('Ranged strike');
      await expectWhole(forecast);
      await shot(page, size, name);
      await page.mouse.move(5, size.height / 2);
    }
    // A walled, garrisoned city: the headline carries both the city's shot and the shield.
    await selectThing(page, 'unit', setup.nearWalled);
    await centred(page, setup.nearWalled);
    await aim(page, await posOf(page, 'city', setup.walled), forecast);
    await expect(garrison).toContainText('Spearman garrison');
    await expect(page.getByTestId('forecast-city-fire')).toBeVisible();
    await expect(page.getByTestId('forecast-attacker-hp')).toBeVisible();
    await expectWhole(forecast);
    await shot(page, size, '21e-forecast-walled-garrisoned');
  });

  test('combined assault: a shared counterattack forecast, then the combined-assault report', async ({ page }) => {
    test.setTimeout(90_000);
    const setup = combinedAssaultFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    // One Spearman is already ordered into the city; the other aims at it.
    await page.evaluate(({ unit, city }) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      const to = v.cities.find((c) => c.id === city)!.pos;
      s.editOrders((o) => [...o, { type: 'MoveUnit', unit, to }]);
    }, { unit: setup.nearOpen, city: setup.open });
    const forecast = page.locator('.forecast');
    await selectThing(page, 'unit', setup.second);
    await centred(page, setup.second);
    await aim(page, await posOf(page, 'city', setup.open), forecast);
    await expect(page.getByTestId('forecast-counter-split')).toContainText(/^1 other also attacking: you take about −\d+ instead of −\d+\.$/);
    await expect(page.getByTestId('forecast-assault')).toContainText('2 attacks this turn');
    await expect(page.getByTestId('forecast-assault-garrison')).toContainText('garrison');
    await expectWhole(forecast);
    await shot(page, size, '21f-forecast-combined-assault');
    await page.mouse.move(5, size.height / 2);
    await page.evaluate(({ unit, city }) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      const to = v.cities.find((c) => c.id === city)!.pos;
      s.editOrders((o) => [...o, { type: 'MoveUnit', unit, to }]);
    }, { unit: setup.second, city: setup.open });
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3');
    await dismissCelebrations(page);
    const report = await openReport(page);
    await expect(report).toContainText('combined assault by your 2 Spearmen');
    await expect(report).toContainText(/its counterattack was split/i);
    await scrollWithinReport(report.getByText(/combined assault by your 2 Spearmen/).first());
    await shot(page, size, '21g-report-combined-assault');
  });

  test('reinforcement: the tile-card note on your threatened empty city, then the report line', async ({ page }) => {
    test.setTimeout(90_000);
    const setup = reinforcementFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await selectThing(page, 'unit', setup.reinforcer);
    await centred(page, setup.reinforcer);
    const note = page.getByTestId('reinforcement-note');
    // A pointer over the city (any move onto it shows the card): wiggle until the note appears.
    const at = await posOf(page, 'city', setup.own);
    await expect.poll(async () => {
      const p = await hexPoint(page, at);
      await page.mouse.move(p.x + 3, p.y + 3);
      await page.mouse.move(p.x, p.y);
      return note.isVisible();
    }).toBe(true);
    await expect(note).toContainText('gets there first and becomes its garrison');
    await shot(page, size, '21h-forecast-reinforcement-note');
    await page.mouse.move(5, size.height / 2);
    await page.evaluate(({ unit, city }) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      const to = v.cities.find((c) => c.id === city)!.pos;
      s.editOrders((o) => [...o, { type: 'MoveUnit', unit, to }]);
    }, { unit: setup.reinforcer, city: setup.own });
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3');
    await dismissCelebrations(page);
    const report = await openReport(page);
    await expect(report).toContainText('reached');
    await expect(report).toContainText('first and');
    await scrollWithinReport(report.getByText(/first and/).first());
    // Park the pointer on a neutral spot: a hover tooltip must not leak into the baseline.
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '21i-report-reinforcement');
  });

  test('city under attack: the This Turn alert, the alarmed banner at both zooms, the first report line and the early warning', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = siegeFixture({ fresh: true });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const at = await posOf(page, 'city', setup.own);
    await page.evaluate((hex) => { const s = (window as unknown as GameWindow).legatum.getState(); s.set({ techOpen: false }); s.jumpTo(hex); }, at);
    await hexPoint(page, at);
    await revealSidebar(page, 'turn-more');
    const card = page.locator(`.seatbar .siege-card[data-city="${setup.own}"]`);
    await expect(card).toContainText('Waypost Seven is under attack');
    await expect(card.getByRole('button', { name: 'Go to Waypost Seven' })).toBeVisible();
    await expect(page.locator('.seatbar .cta .cta-label')).toHaveText('City under attack');
    await expectDockWhole(page);
    await page.mouse.move(5, size.height / 2);
    expect(await primaries(page), 'one primary action: the turn call to action names the city').toBe(1);
    await shot(page, size, '21j-city-under-attack-this-turn');
    await page.evaluate(() => { const m = (window as unknown as { legatumMap: { camera: { zoom: number }; redraw(): void } }).legatumMap; m.camera.zoom = 0.5; m.redraw(); });
    await hexPoint(page, at);
    await page.waitForTimeout(400);
    await shot(page, size, '21k-city-under-attack-far-zoom');
    await page.evaluate(() => { const m = (window as unknown as { legatumMap: { camera: { zoom: number }; redraw(): void } }).legatumMap; m.camera.zoom = 1.25; m.redraw(); });
    await card.getByRole('button', { name: 'Go to Waypost Seven' }).click();
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3');
    await dismissCelebrations(page);
    const report = await openReport(page);
    await expect(report.locator('.report-group').first()).toHaveAttribute('data-category', 'attack');
    await expect(report.locator('.rline').first()).toContainText(/Waypost Seven was attacked: 200 → \d+ HP by 3 Spearmen/);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '21l-report-under-attack-first-line');
    await closeReportIfOpen(page);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    // The battle's damage numbers float over the map for a few seconds: shoot after they clear.
    await page.waitForTimeout(4000);
    await expect(card.locator('.siege-lost')).toHaveText(/^−\d+ HP last turn$/);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '21m-city-attacked-aftermath');
  });

  test('city under attack: the early warning for an army three tiles out', async ({ page }) => {
    test.setTimeout(90_000);
    const setup = siegeFixture({ approach: true });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const at = await posOf(page, 'city', setup.own);
    await page.evaluate((hex) => { const s = (window as unknown as GameWindow).legatum.getState(); s.set({ techOpen: false }); s.jumpTo(hex); }, at);
    await hexPoint(page, at);
    await expect(page.locator(`.seatbar .siege-card[data-city="${setup.own}"][data-kind="approach"]`)).toContainText('Army closing on Waypost Seven');
    await expectDockWhole(page);
    await shot(page, size, '21n-army-approaching-warning');
  });

  test('promotion: unit rank and the promoted combat forecast', async ({ page }) => {
    const setup = promotionFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await selectThing(page, 'unit', setup.pikeman);
    await expect(page.locator('.unit-panel .up-rank')).toBeVisible();
    await shot(page, size, '23-promotion');
    await centred(page, setup.pikeman);
    await aim(page, await posOf(page, 'unit', setup.enemy), page.locator('.forecast'));
    await shot(page, size, '24-promotion-forecast');
  });

  test('Sparta: capture heals nearby units (turn report)', async ({ page }) => {
    const setup = wallsFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views![s.seat];
    });
    const events = [{ type: 'SpartaConquest' as const, city: setup.own, player: 0, units_healed: 3 }];
    const report = { turn: 2, hash: '', events: [events], moments: [[]], alarms: [[]], lines: [formatEvents(events, contextFromViews(view, view))] };
    // No moment is staged for this synthetic event, so open the report directly rather than via
    // the moment lane's "Turn report" link (openReport's usual path).
    await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [], reportOpen: true }), report);
    const opened = page.getByRole('complementary', { name: 'Turn report' });
    await expect(opened).toBeVisible();
    await expect(opened).toContainText('healed by the conquest');
    await expect(opened).toContainText('wounded units healed within two tiles');
    await shot(page, size, '72-sparta-capture-heal-report');
  });
});

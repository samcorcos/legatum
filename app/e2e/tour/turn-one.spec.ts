// Screen tour, the first turns, cities, production and the turn report. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/turn-one`. Shots land in test-results/screen-tour/<size>/turn-one/.
import type { Hex } from '../../src/types';
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { idxOf, spawnUnit } from '../../src/test/fixtureEngine';
import { distance, fromOffset, within } from '../../src/hex';
import { researchFixture } from '../../src/test/researchFixture';
import { progressFixture } from '../../src/test/progressFixture';
import { hallFixture } from '../../src/test/hallFixture';
import { indiaCityClusterFixture } from '../../src/test/indiaCityClusterFixture';
import { improvementCity, UPGRADE_TECHS } from '../../src/test/improvementsFixture';
import { turnReportFixture } from '../../src/test/turnReportFixture';
import { supplyCity } from '../../src/test/supplyFixture';
import { diplomacyState } from '../../src/test/diplomacyFixture';
import { SIZES, closeReportIfOpen, dismissCelebrations, expectReasonsFit, hexPoint, loadFixture, mapSettled, openReport, selectThing, shot, type GameWindow } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('first contact at Peace: the hint warns of a same-turn declaration', async ({ page }) => {
    const setup = wallsFixture();
    for (const relation of setup.fixture.diplomacy!.relations) relation.state = 'Peace';
    await loadFixture(page, coreFixture(setup.fixture), 2, true);
    const hint = page.getByTestId('first-game-hint');
    for (let i = 0; i < 3; i++) {
      await expect(hint).toBeVisible();
      const title = await hint.locator('h3').innerText();
      if (title === 'A rival in sight') break;
      await page.getByRole('button', { name: `Dismiss hint: ${title}` }).click();
      await expect(hint.locator('h3')).not.toHaveText(title);
    }
    await expect(hint).toContainText('A rival in sight');
    await expect(hint).toContainText('A rival can declare War and attack in the same turn.');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'first-contact-peace');
  });


  test('ordinary Walls build card quotes 100 Production', async ({ page }) => {
    const setup = wallsFixture();
    const own = setup.fixture.cities.find((c) => c.id === setup.own)!;
    own.buildings = [];
    own.producing = null;
    setup.fixture.players[0].techs.push('Masonry');
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await selectThing(page, 'city', setup.own);
    const card = page.locator('.city-panel .production-choice', { hasText: 'Walls' }).first();
    await expect(card).toContainText('100');
    await card.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '11a-walls-build-100');
  });

  test('Swiss Alpine Redoubt build card quotes 120 Production', async ({ page }) => {
    const setup = wallsFixture();
    const own = setup.fixture.cities.find((c) => c.id === setup.own)!;
    own.buildings = [];
    own.producing = null;
    setup.fixture.players[0].faction = 'Switzerland';
    setup.fixture.players[0].techs.push('Masonry');
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await selectThing(page, 'city', setup.own);
    const card = page.locator('.city-panel .production-choice', { hasText: 'Alpine Redoubt' }).first();
    await expect(card).toContainText('120');
    await card.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '11b-redoubt-build-120');
  });

  test('city panel: the queue section, the in-place Queued confirmation, and healing', async ({ page, browser }) => {
    test.setTimeout(180_000);
    // Your own walled city, idle with nothing built, as in queue-click.spec.ts.
    const setup = wallsFixture();
    Object.assign(setup.fixture.cities.find((c) => c.id === setup.own)!, { population: 3, buildings: [], producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [] });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await selectThing(page, 'city', setup.own);
    const panel = page.locator('.city-panel');
    const cards = panel.locator('.production-choice:not([data-unavailable])');
    await cards.nth(0).getByRole('button', { name: /^Queue / }).click();
    await cards.nth(1).getByRole('button', { name: /^Queue / }).click();
    await expect(cards.nth(1).locator('.queue-add')).toContainText('Queued #2');
    await shot(page, size, '11b-city-queued-flash');
    await cards.nth(2).getByRole('button', { name: /^Queue / }).click();
    await expect(cards.nth(2).locator('.queue-add')).toHaveText(/^\+ Queue$/, { timeout: 3000 });
    await panel.evaluate((el) => { el.scrollTop = 0; });
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '11c-city-queue-section');

    // A damaged city of yours: it heals, or says in red why not.
    for (const [name, blocked] of [['19b-city-healing', false], ['19c-city-not-healing', true]] as const) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        const hurt = wallsFixture();
        hurt.fixture.cities.find((c) => c.id === hurt.own)!.hp = 90;
        if (blocked) {
          spawnUnit(hurt.fixture, 1, 'Soldier', fromOffset(4, 5), false);
        }
        await loadFixture(p, coreFixture(hurt.fixture), 2);
        await selectThing(p, 'city', hurt.own);
        await expect(p.locator('.city-panel')).toContainText(blocked ? 'Not healing' : '+20 a turn');
        // The status sits under the head and stays on screen as the panel opens (an idle city's picker reveal must not scroll it away).
        await expect(p.getByTestId('city-heal')).toBeInViewport({ ratio: 1 });
        await p.mouse.move(5, size.height / 2);
        await shot(p, size, name);
      } finally { await context.close(); }
    }
  });

  test('settle lens with no Settler on the map: the explored land is washed and the best sites are marked', async ({ page }) => {
    // Playtest 8.4: pressing Settle with no Settler selected used to draw nothing.
    await loadFixture(page, coreFixture(hallFixture('buildable').fixture), 12);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await page.getByRole('radio', { name: 'Settle', exact: true }).click();
    await expect(page.locator('[data-settle-rank]')).toHaveCount(3);
    await page.mouse.move(500, 20);
    await mapSettled(page);
    await shot(page, size, '05c-settle-lens-no-settler');
  });

  test('first turns: capital choice, unit, report, production, research, race, menu', async ({ page }) => {
    test.setTimeout(240_000);
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: true, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    const small = page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ });
    await small.click();
    await page.getByLabel('Map seed', { exact: true }).fill('424242');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await expect(page.getByTestId('first-game-hint')).toContainText('Pick production');
    const opening = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      const mine = v.units.filter((u) => u.owner === v.me);
      return { capital: v.cities.find((c) => c.owner === v.me && c.is_capital)!.id, scout: mine.find((u) => u.kind === 'Scout')?.id ?? null };
    });
    await selectThing(page, 'city', opening.capital);
    await shot(page, size, '05-first-turn');
    // The Settle lens still advises from the capital when no Settler is on the map.
    const lensToggle = page.getByRole('button', { name: /^Lens: / });
    if (await lensToggle.isVisible()) await lensToggle.click();
    await page.getByRole('radio', { name: 'Settle', exact: true }).click();
    await expect(page.getByRole('button', { name: /^Lens: Settle/ })).toBeVisible();
    await shot(page, size, '05b-settle-lens-capital');

    if (opening.scout !== null) {
      await selectThing(page, 'unit', opening.scout);
      await expect(page.locator('.unit-panel')).toContainText('Scout');
      await shot(page, size, '06-unit-selected');
    }

    // End the turn with capital production and the Scout idle: the pending-decisions card.
    await expect.poll(() => page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      return v.cities.filter((c) => c.owner === v.me).length;
    })).toBe(1);
    await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn|Ready\b)/ }).click();
    const pending = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
    await expect(pending).toBeVisible();
    await shot(page, size, '07-end-turn-pending');
    await pending.getByRole('button', { name: 'End turn anyway', exact: true }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('2', { timeout: 60_000 });

    // A routine first turn has no founding celebration, but its report remains available.
    await page.getByRole('button', { name: 'Menu', exact: true }).click();
    await page.getByRole('button', { name: 'Turn report (L)', exact: true }).click();
    await expect(page.getByRole('complementary', { name: 'Turn report' })).toBeVisible();
    await shot(page, size, '09-turn-report');
    await closeReportIfOpen(page);

    // The capital still needs production after the first turn was ended with decisions pending.
    const city = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      return v.cities.find((c) => c.owner === v.me)!.id;
    });
    await selectThing(page, 'city', city);
    await expect(page.locator('.city-panel')).toBeVisible();
    await shot(page, size, '10-production-picker');
    await shot(page, size, '10-production-picker-no-recommendation');
    await expect(page.getByTestId('first-settler-hint')).toHaveCount(0);
    await expect(page.locator('.city-panel .prod-recommended')).toHaveCount(0);
    await page.locator('.city-panel').evaluate((el) => { el.scrollTop = 0; });
    // Idle, nothing queued yet: the banner's production meter draws the eye with its "choose" state.
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await mapSettled(page);
    await shot(page, size, '10a-map-banner-idle');
    await selectThing(page, 'city', city);
    await expect(page.locator('.city-panel')).toBeVisible();
    // A population-1 capital can't build a Settler: the card is greyed out, and clicking it only says why.
    const settler = page.locator('.city-panel .production-choice[data-item="u:Settler"]');
    await expect(settler.getByRole('button', { name: 'Build Settler', exact: true })).toHaveAttribute('aria-disabled', 'true');
    await settler.getByRole('button', { name: 'Build Settler', exact: true }).click({ force: true });
    await expect(settler.locator('.production-notice')).toContainText('needs 2 population');
    await settler.scrollIntoViewIfNeeded();
    await shot(page, size, '10b-settler-unavailable');
    await page.locator('.city-panel').getByRole('button', { name: 'Build Scout', exact: true }).click();
    if (await page.locator('dialog.tech-modal').isVisible()) await page.getByRole('button', { name: 'Close research' }).click();
    await expect.poll(() => page.evaluate((id) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.orders[s.seat].some((o) => o.type === 'SetProductionQueue' && o.city === id);
    }, city)).toBe(true);
    await selectThing(page, 'city', city);
    // One glyph, one meaning: the red diamond is NEW only. Pending-until-Ready is a hollow ring with a visible
    // "pending" word, and "needs a decision" (the top bar's Choose a technology) is a round "!" badge.
    await expect(page.locator('.city-panel .pending-label')).toHaveText('pending');
    expect(await page.locator('.city-panel .pending-dot').evaluate((el) => { const c = getComputedStyle(el); return [c.backgroundColor, c.borderRadius]; })).toEqual(['rgba(0, 0, 0, 0)', '50%']);
    expect(await page.locator('.research-chip.needs').evaluate((el) => getComputedStyle(el, '::after').content)).toBe('"!"');
    await shot(page, size, '11-city-panel');

    // Research: a current technology plus a queue.
    await page.locator('.topbar .research-chip').click();
    const research = page.locator('dialog.tech-modal');
    await expect(research).toBeVisible();
    await research.locator('.era-card[data-tech="Writing"]').getByTestId('research-now').click();
    await expect(research.getByLabel('Research queue', { exact: true })).toContainText('Writing');
    // The queue head's text and the first slot are whole at both sizes (8.9.4 review B3).
    const cut = await research.locator('.research-queue').evaluate((queue) => [...queue.querySelectorAll<HTMLElement>('.research-queue-title, .research-queue-hint, .research-queue-total, li:first-child .rq-name, li:first-child .rq-arrival')]
      .filter((e) => getComputedStyle(e).display !== 'none' && e.scrollWidth > e.clientWidth + 1).map((e) => `${e.className}: ${e.textContent}`));
    expect(cut, 'queue head text and slot 1 are not truncated').toEqual([]);
    await shot(page, size, '12-research');
    await research.getByRole('button', { name: 'Close research', exact: true }).click();
    await expect(research).toHaveCount(0);

    // Victory race, expanded.
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'Score ledger', exact: true }).click();
    await expect(page.getByTestId('objective-sheet')).toBeVisible();
    await shot(page, size, '13-vp-race');
    await page.getByRole('button', { name: 'Close objectives', exact: true }).click();
    await expect(page.getByTestId('objective-sheet')).toHaveCount(0);

    // The game menu (in-game settings live here).
    await page.getByRole('button', { name: 'Menu', exact: true }).click();
    await expect(page.locator('#game-menu')).toBeVisible();
    await shot(page, size, '14-menu');
    // The troubleshooting tools under Advanced (Export game history, the engine and state hash).
    await page.locator('#game-menu .menu-advanced summary').click();
    await expect(page.getByRole('button', { name: 'Export game history', exact: true })).toBeVisible();
    await page.locator('#game-menu .menu-advanced').scrollIntoViewIfNeeded();
    await shot(page, size, '14b-menu-advanced');
    await page.keyboard.press('Escape');
  });

  test('production banking: banked progress, switching, and a city that must choose', async ({ page, browser }) => {
    // Your city builds a Spearman (12 so far) with a Settler (38) and a Granary (20) banked from earlier switches.
    const setup = wallsFixture();
    const own = setup.fixture.cities.find((c) => c.id === setup.own)!;
    Object.assign(own, { population: 3, producing: { kind: 'Unit', unit: 'Soldier' }, prod: 12, productionQueue: [],
      productionBanked: [{ item: { kind: 'Unit', unit: 'Settler' }, progress: 38 }, { item: { kind: 'Building', building: 'Granary' }, progress: 20 }] });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await selectThing(page, 'city', setup.own);
    const panel = page.locator('.city-panel');
    await expect(panel.getByTestId('city-banked')).toContainText('Settler 38/');
    await expect(panel.locator('.production-choice[data-item="u:Settler"] .prod-cost')).toContainText('38/');
    await shot(page, size, '28-production-banked');
    // Switching to the Settler: the Spearman's progress stays banked, the Settler resumes from its own.
    await panel.getByRole('button', { name: 'Build Settler', exact: true }).click();
    await selectThing(page, 'city', setup.own);
    await expect(panel.getByTestId('production-switch')).toContainText('keeps the 12 Production already put into the Spearman');
    await expect(panel.getByTestId('production-switch')).toContainText('Settler resumes at 38/');
    await shot(page, size, '28b-production-switch');

    // The city shrank to 1 and banked its Settler: it asks for a new build and says until when.
    const parked = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p2 = await parked.newPage();
      const again = wallsFixture();
      const city = again.fixture.cities.find((c) => c.id === again.own)!;
      Object.assign(city, { population: 1, producing: null, prod: 0, food: 0, productionQueue: [],
        productionBanked: [{ item: { kind: 'Unit', unit: 'Settler' }, progress: 38 }] });
      await loadFixture(p2, coreFixture(again.fixture), 2);
      await selectThing(p2, 'city', again.own);
      await expect(p2.locator('.city-panel').getByTestId('production-choose')).toContainText('keeps the 38 Production already put into the Settler until');
      await shot(p2, size, '29-production-choose');
    } finally { await parked.close(); }
  });

  test('progress: map banner meters, city bars and milestone notices', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = progressFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    // Your banners carry growth (the territory bar) and production meters at normal zoom; no city is open.
    await page.evaluate((id) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      s.select(null);
      s.jumpTo(v.cities.find((c) => c.id === id)!.pos);
    }, setup.capital);
    await shot(page, size, '46-map-banners');
    await selectThing(page, 'city', setup.second);
    await expect(page.getByTestId('city-growth')).toContainText('Starving');
    await shot(page, size, '47-city-starving');
    await selectThing(page, 'city', setup.capital);
    await expect(page.getByTestId('city-growth')).toContainText(/Grows to 2 · claims .* · unlocks Settler/);
    await expect(page.getByTestId('city-growth')).toContainText('1 turn');
    await shot(page, size, '48-city-progress');
    // Next turn the capital grows (unlocking the Settler) and claims a tile.
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3', { timeout: 60_000 });
    await expect(page.locator('.moment-card').first()).toContainText('Unlocks Settler');
    await shot(page, size, '49-growth-notice');
    await dismissCelebrations(page);
    const report = await openReport(page);
    await expect(report).toContainText('grew to 2 population · unlocks Settler');
    await expect(report).toContainText(/claimed an? \w+ tile/);
    await shot(page, size, '50-growth-report');
  });

  test('city-built improvements: the picker group lights its tile; the improved land at both zoom tiers', async ({ page }) => {
    await loadFixture(page, coreFixture(improvementCity({ techs: UPGRADE_TECHS })), 12);
    const city = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; return v.cities.find((c) => c.owner === v.me)!.id; });
    await selectThing(page, 'city', city);
    const group = page.getByTestId('improvements-group');
    await group.scrollIntoViewIfNeeded();
    await expect(group).toBeVisible();
    await expect(group).toContainText('6 resource tiles unimproved');
    // An upgraded row: the number already includes the known upgrade, so the line doesn't say "known".
    const firstImprovement = group.locator('.production-choice').first();
    await firstImprovement.locator('.production-more > summary').click();
    await expect(firstImprovement.locator('.production-more-text')).toContainText('when built');
    await expect(firstImprovement.locator('.production-more-text')).not.toContainText('known');
    await group.locator('.production-choice').nth(2).locator('button.prod').hover();
    await shot(page, size, '51-improvements-picker');
    await group.locator('.production-choice').nth(1).scrollIntoViewIfNeeded();
    await shot(page, size, '54-improvements-upgraded-row');
    // A click on an eligible tile on the map lights its row.
    const row = group.locator('.production-choice', { hasText: 'Stone' });
    const tile = await row.getAttribute('data-item');
    const [q, r] = tile!.split(':')[2].split(',').map(Number);
    await page.mouse.move(5, 5);
    await page.evaluate((hex) => (window as unknown as GameWindow).legatum.getState().jumpTo(hex), { q, r });
    const at = await hexPoint(page, { q, r });
    await page.mouse.click(at.x, at.y);
    await expect(row).toHaveAttribute('data-picked', 'true');
    await shot(page, size, '55-improvement-map-click-lights-row');
    // Build it: Now producing says which tile.
    await row.locator('button.prod').click();
    await selectThing(page, 'city', city);
    await expect(page.getByTestId('city-production')).toContainText('Quarry');
    await shot(page, size, '56-improvement-queued');
    // A dormant tile's card: one status line, from the same check as the map.
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    const dormant = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      const c = v.cities.find((x) => x.owner === v.me)!; const o = c.detail!.options.find((x) => x.item.kind === 'Improvement' && x.item.improvement === 'Mine' && x.improvement?.resource === 'Iron')!;
      return (o.item as { tile: Hex }).tile; });
    const spot = await hexPoint(page, dormant);
    await page.mouse.move(spot.x - 3, spot.y - 3);
    await page.mouse.move(spot.x, spot.y);
    await expect(page.getByTestId('tile-improvement')).toContainText(/Its Production bonus is dormant: build the Mine from .+'s production list/);
    await expect(page.locator('.tilecard')).not.toContainText('already counts');
    await shot(page, size, '57-improvement-tile-card');
    // The same city after its resources are improved: upgraded forms, then zoomed out (the badge tier).
    const p2 = await page.context().newPage();
    await p2.setViewportSize({ width: size.width, height: size.height });
    await loadFixture(p2, coreFixture(improvementCity({ improved: true, techs: UPGRADE_TECHS })), 12);
    await p2.evaluate(() => { const w = window as unknown as GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } }; const s = w.legatum.getState(), v = s.views![s.seat]; s.select(null); s.jumpTo(v.cities.find((c) => c.owner === v.me)!.pos); w.legatumMap.camera.zoom = 1.25; w.legatumMap.redraw(); });
    await p2.waitForTimeout(600);
    await shot(p2, size, '52-improved-land');
    await p2.evaluate(() => { const w = window as unknown as { legatumMap: { camera: { zoom: number }; redraw(): void } }; w.legatumMap.camera.zoom = 0.55; w.legatumMap.redraw(); });
    await p2.waitForTimeout(600);
    await shot(p2, size, '53-improved-land-zoomed-out');
    await p2.close();
    // The report line: a city one turn from finishing a Farm.
    const p3 = await page.context().newPage();
    await p3.setViewportSize({ width: size.width, height: size.height });
    await loadFixture(p3, coreFixture(improvementCity({ techs: UPGRADE_TECHS, finishing: true })), 12);
    await endPlanning(p3);
    await expect(p3.getByTestId('turn-number')).toHaveText('13', { timeout: 60_000 });
    await dismissCelebrations(p3);
    const report = await openReport(p3);
    await expect(report).toContainText(/completed Farm, 1\stile/);
    await report.getByText(/completed Farm, 1\stile/).scrollIntoViewIfNeeded();
    await shot(p3, size, '58-improvement-report');
    await p3.close();
  });

  test('India city cluster map art, zoomed in and out', async ({ page }) => {
    const setup = indiaCityClusterFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await page.evaluate((id) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      s.select(null);
      s.jumpTo(v.cities.find((c) => c.id === id)!.pos);
    }, setup.capital);
    await mapSettled(page);
    const setZoom = (z: number) => page.evaluate((zoom) => {
      const m = (window as unknown as { legatumMap: { camera: { zoom: number }; redraw(): void } }).legatumMap;
      m.camera.zoom = zoom;
      m.redraw();
    }, z);
    // Zoomed out: the Indian cities remain readable without connecting map ornament.
    await setZoom(0.55);
    await shot(page, size, '80-india-cities-zoomed-out');
    // Zoomed in: ordinary historical city art at the capital.
    await setZoom(2.1);
    await shot(page, size, '81-india-capital-zoomed-in');
  });

  test('turn report history: reopen with Last turn and step back to a combat turn', async ({ page }) => {
    test.setTimeout(90_000);
    const setup = turnReportFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await page.evaluate(({ ranger, enemy }) => {
      const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
      const target = view.units.find((u) => u.id === enemy)!.pos;
      s.editOrders((o) => [...o, { type: 'RangedAttack', unit: ranger, target }]);
    }, { ranger: setup.ranger, enemy: setup.enemy });
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3');
    await openReport(page);
    await closeReportIfOpen(page);
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('4');
    await dismissCelebrations(page);
    await closeReportIfOpen(page);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await page.locator('.seatbar .last-turn').click();
    const report = page.getByRole('complementary', { name: 'Turn report' });
    await expect(report).toBeVisible();
    await report.getByRole('button', { name: /^Previous turn/ }).click();
    await expect(report.locator('.report-kicker')).toHaveText('1 turn ago');
    await shot(page, size, '25-turn-report-history');
  });

  test('Turn 1: the first city panel shows its build choice at scroll 0', async ({ page }) => {
    // The first city a player ever sees: turn 1, population 1, nothing queued yet.
    const first = researchFixture('India');
    first.turn = 1;
    const capital = first.cities.find((c) => c.owner === 0 && c.isCapital)!;
    Object.assign(capital, { population: 1, buildings: [], producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [] });
    await loadFixture(page, coreFixture(first), 1);
    await selectThing(page, 'city', capital.id);
    const panel = page.locator('.city-panel');
    await expect(panel).toBeVisible();
    await panel.evaluate((el) => { el.scrollTop = 0; });
    const fits = await panel.evaluate((el) => {
      const box = el.getBoundingClientRect(), head = el.querySelector('.cp-head')!.getBoundingClientRect();
      return [...el.querySelectorAll('.production-choice')].some((c) => { const r = c.getBoundingClientRect(); return r.top >= head.bottom && r.bottom <= box.bottom; });
    });
    expect(fits).toBe(true);
    await shot(page, size, '11a-turn1-city-panel');
  });
  test('strategic supply: the chip tooltip in a shortage, an idle Factory in the city panel and a greyed-out Forge card', async ({ page }) => {
    test.setTimeout(150_000);
    await loadFixture(page, coreFixture(supplyCity('short')), 12);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    // Both chips show a shortage with the alert mark and the idle count; the top bar still fits.
    await expect(page.getByTestId('supply-Iron')).toHaveText('0/2');
    await expect(page.getByTestId('supply-Coal')).toHaveText('0/0');
    expect(await page.locator('.topbar').evaluate((el) => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(0);
    await page.locator('.supply-stat.supply-iron').click();
    const tip = page.locator('.supply-tip');
    await expect(tip).toBeVisible();
    await expect(tip.locator('.bd-title').first()).toHaveText('2 of 2 Iron at work, 2 idle');
    await expect(tip).toContainText('Short of Iron: 2 buildings or units are idle');
    await expect(tip).toContainText('Needs 4, supplies 2');
    await expect(tip).toContainText('Short by 2');
    await expect(tip).not.toContainText(/in use|users?\b/);
    await expect(tip).toContainText(/Iron(?: with a Mine)?, (?:\d+ tiles? [NESW]+ of |in your land)/);
    await expect(tip).toContainText('Forge in');
    await expect(tip).toContainText('Idle, newest first');
    // Every idle entry can be found on the map.
    await expect(tip.getByRole('button', { name: 'Show on map' }).first()).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '98a-supply-chip-shortage');
    await page.keyboard.press('Escape');
    // Esc closes the tooltip and leaves a focus frame on the chip.
    await expect(tip).toBeHidden();
    await expect(page.locator('.supply-stat.supply-iron[data-esc-focus]')).toHaveCount(1);
    // An idle unit shows the strength it fights with, what it lost and why; the map marks it with the alert badge.
    const idleUnit = await page.evaluate(() => { const st = (window as unknown as GameWindow).legatum.getState(), v = st.views![st.seat]; return v.units.find((u) => u.unsupplied)?.id ?? -1; });
    expect(idleUnit).toBeGreaterThan(-1);
    await selectThing(page, 'unit', idleUnit);
    await expect(page.getByTestId('unit-unsupplied')).toContainText('Idle: you are short of Iron');
    await expect(page.getByTestId('unit-strength-was')).toHaveText(/^was \d+: no Iron$/);
    await expect(page.getByTestId('unit-strength-idle')).toHaveText(/^\d+$/);
    const idleAction = page.locator('.unit-panel .act-pair .act-wrap').first();
    await expect(idleAction).not.toHaveClass(/primary/);
    await expect(page.locator('.unit-panel button.primary')).toHaveCount(1);
    const keyFits = await idleAction.evaluate((el) => {
      const button = el.querySelector('.act')!.getBoundingClientRect();
      const key = el.querySelector('.kbd')!.getBoundingClientRect();
      return key.left >= button.left && key.right <= button.right;
    });
    expect(keyFits, 'Fortify key hint stays inside its button').toBe(true);
    await expectReasonsFit(page);
    if (size.width === 1000) {
      const stats = await page.locator('.unit-panel .up-stats').evaluate((el) => {
        const scroller = el.closest('.unit-panel-body')!;
        const bounds = scroller.getBoundingClientRect();
        const labels = [...el.querySelectorAll('.lbl')].filter((label) => /^(HP|Moves)$/i.test(label.textContent?.trim() ?? ''))
          .map((label) => { const b = label.getBoundingClientRect(); return { top: b.top, bottom: b.bottom }; });
        return { labels, top: bounds.top, bottom: bounds.bottom, scroll: scroller.scrollTop };
      });
      expect(stats.labels).toHaveLength(2);
      for (const label of stats.labels) {
        expect(label.top).toBeGreaterThanOrEqual(stats.top - 1);
        expect(label.bottom).toBeLessThanOrEqual(stats.bottom + 1);
      }
      expect(stats.scroll).toBe(0);
    }
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '98d-idle-unit-panel');
    // The idle Factory says why, next to the building list.
    const city = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; return v.cities.find((c) => c.owner === v.me)!.id; });
    await selectThing(page, 'city', city);
    const note = page.locator('.idle-building-note').first();
    await note.scrollIntoViewIfNeeded();
    await expect(note).toContainText('The Factory is idle: you have no Coal. It yields nothing until you get some.');
    await expect(page.locator('.cp-foot .idle-building')).toContainText('Idle: no Coal');
    await shot(page, size, '98b-idle-factory-city-panel');
    // A city short of Iron: the Forge card is greyed out with the core's reason where its price would be.
    const p2 = await page.context().newPage();
    await p2.setViewportSize({ width: size.width, height: size.height });
    await loadFixture(p2, coreFixture(supplyCity('blocked')), 12);
    await p2.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    const city2 = await p2.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; return v.cities.find((c) => c.owner === v.me)!.id; });
    await selectThing(p2, 'city', city2);
    const forge = p2.locator('.production-choice.supply-blocked').filter({ hasText: 'Forge' });
    await expect(forge).toHaveCount(1);
    await expect(forge).toContainText('No Iron to spare: all 2 of your Iron are in use. Bring another Iron deposit inside your borders, or build a Mine on one for +1 Iron.');
    await expect(forge.locator('.prod-supply-need')).toContainText('Uses 1 Iron');
    // Buying is blocked too, so the button carries no price.
    await expect(forge.getByRole('button', { name: /^Buy Forge/ })).toBeDisabled();
    await forge.scrollIntoViewIfNeeded();
    await p2.mouse.move(5, size.height / 2);
    await shot(p2, size, '98c-forge-card-blocked');
    await p2.close();
  });

  test('Buy a tile: the city panel button, the priced placement on the map, and the queued purchase', async ({ page }) => {
    await loadFixture(page, coreFixture(diplomacyState(12, { met: true })), 12);
    const target = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      const city = v.cities.find((c) => c.owner === v.me)!;
      s.set({ techOpen: false });
      s.select({ kind: 'city', id: city.id }, true);
      s.jumpTo(city.pos);
      return { tile: city.detail!.buyable_tiles[0].tile, price: city.detail!.buyable_tiles[0].price };
    });
    const panel = page.locator('.city-panel');
    const button = panel.getByRole('button', { name: /^Buy a tile · from/ });
    await expect(button).toBeVisible();
    // The button sits in the panel's first screen; the shot records whether it needs a scroll.
    await expect(button).toBeInViewport({ ratio: 1 });
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '11d-city-buy-tile-button');
    await button.click();
    await expect(page.getByTestId('city-placement')).toBeVisible();
    const point = await hexPoint(page, target.tile);
    await page.mouse.move(point.x, point.y);
    await expect(page.locator('.tilecard')).toContainText(`Click to buy for ${target.price} Gold`);
    await shot(page, size, '11e-city-buy-tile-placement');
    await page.mouse.click(point.x, point.y);
    await expect(panel.getByTestId('city-buy-tile').locator('.cost-chip')).toContainText(`${target.price}`);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '11f-city-buy-tile-queued');
  });

  // The control's states in a city that is building (the common case, picker below the fold) at scroll 0. The two refusals that
  // core words (already bought, no tile in reach) are patched into the view: the sentence under test is core's, covered by Rust.
  test('Buy a tile in a building city: as opened, mixed affordability, already bought, no tile in reach, item queued', async ({ page }) => {
    const s = diplomacyState(12, { met: true });
    Object.assign(s.cities[0], { producing: { kind: 'Unit', unit: 'Soldier' }, prod: 12 });
    s.players[0].gold = 50;
    s.players[0].explored.fill(1);
    // The city already owns its west half of ring 2, so ring 2 (50 Gold) and ring 3 (60 Gold) are both buyable.
    for (const h of within(s.cities[0].pos, 2).filter((h) => distance(h, s.cities[0].pos) === 2).slice(0, 6)) { const i = idxOf(s, h); s.owner[i] = 0; s.tileCity[i] = 0; }
    await loadFixture(page, coreFixture(s), 12);
    const info = await page.evaluate(() => {
      const w = (window as unknown as GameWindow).legatum.getState(), v = w.views![w.seat];
      const city = v.cities.find((c) => c.owner === v.me)!;
      w.set({ techOpen: false });
      w.select({ kind: 'city', id: city.id }, true);
      w.jumpTo(city.pos);
      const tiles = city.detail!.buyable_tiles;
      return { id: city.id, cheap: tiles[0].tile, dear: tiles.find((t) => t.price > v.economy.gold)!.tile };
    });
    const panel = page.locator('.city-panel');
    const away = () => page.mouse.move(5, size.height / 2);
    // New objects all the way down, so the panel's memoised previews see the change.
    const setDetail = (change: { buy_tile_blocker?: string | null; empty?: boolean }) => page.evaluate(({ id, change }) => {
      const w = (window as unknown as GameWindow).legatum.getState();
      const win = window as unknown as { __tiles?: unknown };
      const views = w.views!.map((v, i) => i !== w.seat ? v : { ...v, cities: v.cities.map((c) => {
        if (c.id !== id || !c.detail) return c;
        if (!win.__tiles) win.__tiles = c.detail.buyable_tiles;
        return { ...c, detail: { ...c.detail, buy_tile_blocker: change.buy_tile_blocker ?? null, buyable_tiles: change.empty ? [] : win.__tiles as typeof c.detail.buyable_tiles } };
      }) });
      w.set({ views });
    }, { id: info.id, change });
    const patch = (blocker: string | null, empty: boolean) => setDetail({ buy_tile_blocker: blocker, empty });
    const restore = () => setDetail({});

    await expect(panel.getByRole('button', { name: /^Buy a tile · from/ })).toBeInViewport({ ratio: 1 });
    await away();
    await shot(page, size, '11g-buy-tile-building-city-as-opened');

    await panel.getByRole('button', { name: /^Buy a tile/ }).click();
    const dear = await hexPoint(page, info.dear);
    await page.mouse.move(dear.x, dear.y);
    await expect(page.getByTestId('tile-refusal')).toContainText('short');
    await shot(page, size, '11h-buy-tile-mixed-affordability-refusal');
    const cheap = await hexPoint(page, info.cheap);
    await page.mouse.move(cheap.x, cheap.y);
    await expect(page.locator('.tilecard')).toContainText('Click to buy for');
    await shot(page, size, '11i-buy-tile-mixed-affordability-hover');
    await page.keyboard.press('Escape');

    await patch('Astra already bought something this turn', false);
    await expect(panel.locator('.city-borders-reason')).toHaveText('Already bought this turn');
    await away();
    await shot(page, size, '11j-buy-tile-already-bought');

    await patch('Astra has no free tile to buy within 3 tiles of the city.', true);
    await expect(panel.locator('.city-borders-reason')).toHaveText('No free tile in reach');
    await away();
    await shot(page, size, '11k-buy-tile-no-tile-in-reach');

    await restore();
    await page.evaluate((id) => {
      (window as unknown as GameWindow).legatum.getState().editOrders((o) => [...o, { type: 'BuyItem', city: id, item: { kind: 'Unit', unit: 'Soldier' } }], 'purchase');
    }, info.id);
    await expect(panel.locator('.city-borders-reason')).toHaveText(/^Cancel the .+ purchase first$/);
    await away();
    await shot(page, size, '11l-buy-tile-item-queued');
  });
});

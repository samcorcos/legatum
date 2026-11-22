import { revealSidebar } from '../sidebarAssertions';
// Screen tour, diplomacy, treaties, war declarations and alarms. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/diplomacy-war`. Shots land in test-results/screen-tour/<size>/diplomacy-war/.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { callToArmsCoreSave, leaderAllianceOfferCoreSave } from '../../src/test/allianceFixture';
import { TECHS } from '../../src/data';
import { diplomacyCoreSave, diplomacyState } from '../../src/test/diplomacyFixture';
import { WAR_PROMPT, warPromptCoreSave } from '../../src/test/warPromptFixture';
import { warDeclaredCoreSave } from '../../src/test/warDeclaredFixture';
import { ALARM_EVENTS, ALARM_TURN, alarmCoreSave } from '../../src/test/alarmFixture';
import { alarmsForTurn } from '../../src/celebrate/alarms';
import { contextFromViews, formatEvents } from '../../src/format';
import { SIZES, centred, dismissCelebrations, hexPoint, loadFixture, openReport, primaries, selectThing, shot, type GameWindow } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });


  test('diplomacy: inbox, alliance offer and declaring war', async ({ page, browser }) => {
    test.setTimeout(120_000);
    await loadFixture(page, diplomacyCoreSave(16, { met: true }), 16);
    await revealSidebar(page, 'standings');
    await page.getByRole('button', { name: /^Diplomacy( \(\d+\))?$/ }).click();
    const inbox = page.getByRole('complementary', { name: 'Diplomacy', exact: true });
    await expect(inbox).toBeVisible();
    await expect(inbox.getByRole('button', { name: 'Declare War', exact: true })).toBeVisible();
    await shot(page, size, '16-diplomacy-inbox');

    // An incoming Alliance offer: one decision card.
    const offer = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p2 = await offer.newPage();
      const state = diplomacyState(10);
      state.diplomacy!.next_proposal_id = 1;
      state.diplomacy!.proposals = [{ id: 1, from: 1, to: 0, proposer: 1, kind: 'Alliance', created_turn: 9, respond_by: 12, relation_version: 0 }];
      await loadFixture(p2, coreFixture(state), 10);
      await p2.getByRole('button', { name: /Respond to offer/ }).first().click();
      await expect(p2.locator('.offer-card')).toBeVisible();
      // An Alliance offer never prints the Peace rule ("Accepting ends your wars...").
      await expect(p2.locator('.offer-card')).not.toContainText('Accepting ends your wars');
      await expect(p2.locator('[data-testid=offer-agree]')).toHaveCount(0);
      // The pinned treaty emblem overhangs the portrait: the leader's quote below it must clear it.
      const emblem = (await p2.locator('.offer-card .offer-emblem').boundingBox())!, quote = (await p2.locator('.offer-card .offer-quote').boundingBox())!;
      expect(quote.y, 'the quote starts below the treaty emblem').toBeGreaterThanOrEqual(emblem.y + emblem.height - 0.5);
      await shot(p2, size, '17-alliance-offer');
    } finally { await offer.close(); }

    // An ally's war: a call to arms is one decision card, Join primary and Decline beside it.
    const call = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p3 = await call.newPage();
      await loadFixture(p3, callToArmsCoreSave(), 16);
      await expect(p3.locator('.seatbar .cta')).toHaveClass(/cta-call/);
      await p3.getByRole('button', { name: /Answer call to arms/ }).first().click();
      await expect(p3.locator('.call-card')).toBeVisible();
      await expect(p3.locator('.call-card .offer-headline')).toContainText(/Your ally .+ calls you to war with .+/);
      await expect(p3.locator('.call-card .btn.primary')).toHaveCount(1);
      await expect(p3.locator('.call-card .btn.primary')).toContainText(/Join the war on/);
      const colors = await p3.locator('.seatbar .cta').evaluate((el) => ({ seat: getComputedStyle(el).backgroundColor, join: getComputedStyle(document.querySelector('.call-card .btn.primary')!).backgroundColor }));
      expect(colors.seat).not.toBe(colors.join);
      await shot(p3, size, '17b-call-to-arms');
    } finally { await call.close(); }

    // Declaring war: order a Spearman onto a unit of a team at Peace.
    const war = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p3 = await war.newPage();
      await loadFixture(p3, warPromptCoreSave(), 1);
      const soldier = await p3.evaluate((at) => {
        const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
        return v.units.find((u) => u.owner === v.me && u.pos.q === at.q && u.pos.r === at.r)!.id;
      }, WAR_PROMPT.soldier);
      await selectThing(p3, 'unit', soldier);
      const at = await hexPoint(p3, WAR_PROMPT.scout);
      await p3.mouse.move(at.x, at.y);
      await expect(p3.locator('.peace-hint')).toBeVisible();
      await shot(p3, size, '18a-peace-hint');
      await p3.mouse.click(at.x, at.y, { button: 'right' });
      const dialog = p3.getByRole('dialog', { name: 'Declare War on Switzerland?' });
      await expect(dialog).toBeVisible();
      await shot(p3, size, '18-declare-war');
    } finally { await war.close(); }
  });

  test('treaties: first contact in the inbox, then an accepted Alliance and Peace in the report', async ({ page }) => {
    // First contact: a freshly met empire's row opens with its leader, quote and relation emblem.
    await loadFixture(page, diplomacyCoreSave(3, { met: true }), 3);
    await revealSidebar(page, 'standings');
    await page.getByRole('button', { name: /^Diplomacy( \(\d+\))?$/ }).click();
    await expect(page.getByRole('complementary', { name: 'Diplomacy', exact: true }).locator('.relation-head')).toBeVisible();
    await shot(page, size, '16b-first-contact-inbox');

    // An accepted Alliance and a Peace, as one turn's report (the real formatting, fed synthetic events).
    await page.goto('about:blank');
    await loadFixture(page, alarmCoreSave(), ALARM_TURN);
    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views![s.seat];
    });
    const rel = (a: number, b: number, state: 'Peace' | 'Alliance') => ({ a, b, state, security: state === 'Alliance' ? { id: 3, started_turn: ALARM_TURN, protected_until: ALARM_TURN + 10 } : null,
      effective_turn: ALARM_TURN, guarantee_until: ALARM_TURN + 10, version: 3, war_id: null, declarers: [] });
    const events = [
      { type: 'ProposalAccepted' as const, effective_turn: ALARM_TURN, proposal: { id: 4, from: 1, to: 0, proposer: 1, kind: 'Alliance' as const, created_turn: ALARM_TURN - 2, respond_by: ALARM_TURN + 1, relation_version: 1 } },
      { type: 'TreatyActivated' as const, relation: rel(0, 1, 'Alliance') },
      { type: 'PeaceEstablished' as const, relation: rel(0, 3, 'Peace') },
    ];
    const report = { turn: ALARM_TURN, hash: '', events: [events], moments: [[]], alarms: [[]], lines: [formatEvents(events, contextFromViews(view, view))] };
    await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [], reportOpen: true }), report);
    await expect(page.getByRole('complementary', { name: 'Turn report' })).toContainText('Alliance');
    await shot(page, size, '17b-treaty-accepted-report');
  });

  test('a rival two VP from victory: the near-victory card, then the report line', async ({ page }) => {
    await loadFixture(page, alarmCoreSave(), ALARM_TURN);
    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views![s.seat];
    });
    // A rival: a team with no Alliance with you.
    const rival = view.standings.find((x) => x.team !== view.team && !view.diplomacy.relations.some((r) => r.state === 'Alliance' && (r.a === x.team || r.b === x.team) && (r.a === view.team || r.b === view.team)))!.team;
    const standing = (vp: number) => ({ ...view, standings: view.standings.map((x) => x.team === rival ? { ...x, vp, target: 10 } : x) });
    const before = standing(7), after = standing(8);
    const alarms = alarmsForTurn([], before, after);
    expect(alarms.map((a) => a.kind)).toEqual(['victoryNear']);
    const report = { turn: ALARM_TURN, hash: '', events: [[]], moments: [[]], alarms: [alarms], lines: [alarms.map((a) => a.line)] };
    await page.evaluate(({ report, after }) => {
      const g = (window as unknown as GameWindow).legatum, st = g.getState();
      g.setState({ views: st.views!.map((v, i) => (i === st.seat ? after : v)) });
      g.getState().set({ report, alarmSeen: [], alarmStep: {}, reportOpen: false });
    }, { report, after });
    const alarm = page.getByTestId('alarm-screen');
    await expect(alarm).toHaveAttribute('data-armed', '');
    await expect(alarm.locator('h1')).toContainText('2 VP from victory: 8 of 10');
    await expect(alarm.getByTestId('rival-win')).toContainText('from winning.');
    await expect(alarm.locator('.btn.primary')).toHaveCount(1);
    await expect(alarm.locator('.alarm-primary')).toHaveAccessibleName(/^(Declare war on|See the race)/);
    await shot(page, size, '30b-near-victory');
    await page.keyboard.press('Escape');
    await expect(alarm).toHaveCount(0);
    await page.mouse.move(1, 330);
    await shot(page, size, '30bb-near-victory-race-panel');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
    await expect(page.getByRole('complementary', { name: 'Turn report' })).toContainText('2 VP from victory: 8 of 10');
    await shot(page, size, '30c-near-victory-report');
  });

  test('war declared on you: the full-screen declaration, then the map, report and race', async ({ page }) => {
    await loadFixture(page, warDeclaredCoreSave(), 1);
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    const alarm = page.getByTestId('alarm-screen');
    await expect(alarm).toHaveAttribute('data-armed', '');
    await shot(page, size, '30-war-declared');
    const primary = alarm.locator('.alarm-primary');
    await expect(primary).toHaveAccessibleName(/^Show on map/);
    await primary.click();
    await expect(alarm).toHaveCount(0);
    await dismissCelebrations(page);
    await openReport(page);
    await shot(page, size, '31-war-declared-after');
  });

  test('lighter alarms: a broken treaty, a lost city and an ally\'s war', async ({ page }) => {
    await loadFixture(page, alarmCoreSave(), ALARM_TURN);
    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views![s.seat];
    });
    const alarm = page.getByTestId('alarm-screen');
    for (const [kind, name, primary] of [['breach', '41-alarm-broken-treaty', /^Continue/], ['cityLost', '42-alarm-city-lost', /^Show Brasslock/],
      ['allyWar', '43-alarm-ally-war', /^Continue/]] as const) {
      // The turn's report as applyTurn builds it, from the real alarm and report formatting.
      const events = ALARM_EVENTS[kind];
      const report = { turn: ALARM_TURN, hash: '', events: [events], moments: [[]], alarms: [alarmsForTurn(events, view, view)],
        lines: [formatEvents(events, contextFromViews(view, view))] };
      expect(report.alarms[0].map((a) => a.kind)).toEqual([kind]);
      await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [], alarmStep: {}, reportOpen: false }), report);
      await expect(alarm).toHaveAttribute('data-alarm', kind);
      await expect(alarm).toHaveAttribute('data-armed', '');
      if (kind === 'breach') await expect(alarm).toContainText('broke the Alliance with you early');
      if (kind === 'cityLost') await expect(alarm.locator('.alarm-fact.strong .alarm-fact-text')).toContainText('seized 80 Gold');
      await shot(page, size, name);
      const action = alarm.locator('.alarm-primary');
      await expect(action).toHaveAccessibleName(primary);
      await action.click();
      await expect(alarm).toHaveCount(0);
    }
  });

  test('attack at Peace: foreign city card, ranged War prompt and the Fix order card', async ({ page }) => {
    const CITY = { q: 19, r: 8 }; // Switzerland' capital Harmonia; the Ranger stands 2 tiles west.
    await loadFixture(page, warPromptCoreSave(false, true), 1);
    const ranger = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      return v.units.find((u) => u.owner === v.me && u.kind === 'Ranger')!.id;
    });
    await selectThing(page, 'unit', ranger);
    await centred(page, ranger);
    const city = await hexPoint(page, CITY);
    await page.mouse.click(city.x, city.y);
    const card = page.getByTestId('foreign-city-actions');
    await expect(card).toBeVisible();
    // The attack is the card's one primary; a rival's capture bounty is a prize, not a loss.
    expect(await primaries(page)).toBe(1);
    await expect(page.locator('.seatbar .cta')).toHaveClass(/\bstepped\b/);
    await expect(page.locator('.cp-bounty')).toHaveClass(/\bgain\b/);
    await shot(page, size, '37-foreign-city-card');

    const dialog = page.getByRole('dialog', { name: 'Declare War on Switzerland?' });
    await card.getByRole('button', { name: /^Attack with/ }).click();
    await expect(dialog).toBeVisible();
    await shot(page, size, '38-ranged-war-prompt');
    await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
    await expect(dialog).toHaveCount(0);

    // A stale ranged order on a team at Peace: the Fix order call to action.
    await page.evaluate(({ ranger, city }) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      s.editOrders((o) => [...o, { type: 'RangedAttack', unit: ranger, target: city }]);
      s.select({ kind: 'unit', id: ranger });
    }, { ranger, city: CITY });
    // A broken order is the next decision: "Fix order" with its reason, and the one primary.
    await expect(page.locator('.seatbar .cta .cta-label')).toHaveText('Fix order');
    await expect(page.locator('.seatbar .cta .cta-sub')).toContainText('declare War first');
    // The unit's status names the target and is never healthy green while the order can't resolve.
    await expect(page.locator('.unit-panel .up-status')).toHaveText(/^Ranged attack on Harmonia · won’t resolve$/i);
    await expect(page.locator('.unit-panel .up-status')).toHaveClass(/\bbad\b/);
    await expect(page.locator('.unit-panel').getByTestId('unit-order-error')).toContainText('declare War first');
    expect(await primaries(page)).toBe(1);
    await shot(page, size, '39-fix-order');
    // Fixing it opens the same War prompt; confirming keeps the attack and says so.
    await page.locator('.seatbar .cta').click();
    await expect(dialog).toBeVisible();
    await dialog.locator('.btn.primary').click();
    await expect(dialog).toHaveCount(0);
    // The race says what the city card says, and the notice keeps clear of the race title.
    await revealSidebar(page, 'standings');
    await expect(page.locator('.vp-row .vhc-rel.declared')).toBeVisible();
    await expect(page.locator('.vp-row .vhc-rel.declared')).toHaveAttribute('data-relation', 'War declared');
    const notice = (await page.locator('.decision-notice').boundingBox())!, title = (await page.locator('.vp .ph').boundingBox())!;
    expect(notice.x + notice.width <= title.x || notice.y >= title.y + title.height).toBe(true);
    await shot(page, size, '40-war-declared-notice');
  });

  test('Great Works placement: Build lights the eligible tiles, hover says what a tile gives up, then the placed card', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = wallsFixture();
    Object.assign(setup.fixture.cities.find((c) => c.id === setup.own)!, { population: 8, producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [], buildings: ['Autoworks'] });
    setup.fixture.players[0].gold = 100_000;
    setup.fixture.players[0].techs = [...TECHS];
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    await selectThing(page, 'city', setup.own);
    const works = page.locator('.city-panel .cp-great-works');
    const card = works.locator('.production-choice[data-item^="w:"]').first();
    await expect(works.locator('.production-choice')).toHaveCount(7);
    await card.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '86c0-great-works-list-open');
    await card.locator('button.prod').click();
    const placement = page.getByTestId('city-placement');
    await expect(placement).toContainText(/Choose a tile for .*: click a highlighted tile near /);
    await expect(placement.getByRole('button', { name: /Cancel.*Esc/ })).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '86c-great-works-placing');
    await page.keyboard.press('Escape');
    await expect(placement).toBeHidden();
    await expect(card).toBeInViewport();
    await shot(page, size, '86c1-great-works-esc-cancelled');
    await card.locator('button.prod').click();
    await expect(placement).toBeVisible();
    const lit = await page.evaluate(() => (window as unknown as { legatumMap: { place: { candidates: { q: number; r: number }[] } | null } }).legatumMap.place?.candidates ?? []);
    expect(lit.length, 'eligible tiles are lit').toBeGreaterThan(1);
    const pick = lit[0];
    await page.evaluate((hex) => (window as unknown as GameWindow).legatum.getState().jumpTo(hex), pick);
    const spot = await hexPoint(page, pick);
    await page.mouse.move(spot.x - 3, spot.y - 3);
    await page.mouse.move(spot.x, spot.y);
    await expect(page.locator('.tilecard')).toContainText('Click to place it here');
    await expect(page.locator('.tilecard')).toContainText('gives up');
    await shot(page, size, '86d-great-works-placing-hover');
    await page.mouse.click(spot.x, spot.y);
    await expect(placement).toBeHidden();
    await expect(page.locator('.city-panel .cp-great-works')).toBeVisible();
    await expect(card).toBeInViewport();
    await selectThing(page, 'city', setup.own);
    await page.locator('.city-panel .cp-great-works .production-choice').first().scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '86e-great-works-placed');
    // The placed tile keeps its ring on the map while the city is open.
    await page.evaluate((hex) => (window as unknown as GameWindow).legatum.getState().jumpTo(hex), pick);
    await hexPoint(page, pick);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '86f-great-works-placed-ring');
  });

  test('Great Works have no Buy control for any faction and a free Alliance offer against the leader', async ({ page, browser }) => {
    // The Great Works section in a big city that meets the Foundry's requirement, every tech known.
    for (const [faction, name] of [['Switzerland', '86a-great-works-switzerland'], ['India', '86b-great-works-india']] as const) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        const setup = wallsFixture();
        Object.assign(setup.fixture.cities.find((c) => c.id === setup.own)!, { population: 8, producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [], buildings: ['Autoworks'] });
        setup.fixture.players[0].faction = faction;
        setup.fixture.players[0].gold = 100_000;
        setup.fixture.players[0].techs = [...TECHS];
        await loadFixture(p, coreFixture(setup.fixture), 2);
        await p.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
        await selectThing(p, 'city', setup.own);
        const works = p.locator('.city-panel .cp-great-works');
        // The section stays open; bring its first available card into view for the tour shot.
        const first = works.locator('.production-choice[data-item^="w:"]').first();
        await first.scrollIntoViewIfNeeded();
        await expect(works.locator('.hint')).toHaveCount(1);
        await expect(works.locator('.hint')).toContainText('Wonders are built with Production only');
        await expect(first.locator('.prod-buy-why')).toHaveCount(0);
        await expect(first.locator('.btn-buy')).toHaveCount(0);
        await p.mouse.move(5, size.height / 2);
        await shot(p, size, name);
      } finally { await context.close(); }
    }
    // An Alliance offer shows its upkeep.
    await loadFixture(page, leaderAllianceOfferCoreSave(), 16);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    await expect(page.locator('.offer-card')).toContainText('Upkeep');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '87-alliance-offer');
  });
});

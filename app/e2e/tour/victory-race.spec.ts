import { revealSidebar } from '../sidebarAssertions';
// Screen tour, the victory race, holdings and objective cards. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/victory-race`. Shots land in test-results/screen-tour/<size>/victory-race/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { progressFixture } from '../../src/test/progressFixture';
import { stageHold, stageHoldings } from '../vpStage';
import { ALARM_TURN, alarmCoreSave } from '../../src/test/alarmFixture';
import { eliminatedCoreSave, ELIMINATED_TURN } from '../../src/test/eliminatedFixture';
import { holdBroken, holdStarted, stageHoldReport } from '../holdFixture';
import { gameOverCoreSave } from '../../src/test/gameOverFixture';
import { contextFromViews, formatEvents } from '../../src/format';
import { factionGoalRule } from '../../src/data';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('victory reveal is already settled with Map animations Off', async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem('legatum.combat-fx', JSON.stringify({ enabled: false })));
    await loadFixture(page, gameOverCoreSave('victory'), 164);
    const reveal = page.getByTestId('game-reveal');
    await expect(reveal).toHaveAttribute('data-settled', 'true');
    await expect(page.locator('.go-row b').first()).not.toHaveText(/^0\/10$/);
    await shot(page, size, '44d-victory-animations-off');
  });


  test('victory hold: the alarm for a rival, for you and for an ally, and the broken-hold notice', async ({ page }) => {
    await loadFixture(page, alarmCoreSave(), ALARM_TURN);
    const alarm = page.getByTestId('alarm-screen');
    for (const [team, name] of [[3, '44-alarm-hold-rival'], [0, '44b-alarm-hold-self'], [1, '44c-alarm-hold-ally']] as const) {
      await stageHoldReport(page, [holdStarted(team)], { holding: { team, held: 1, needed: 10 } });
      await expect(alarm).toHaveAttribute('data-alarm', 'victoryHold');
      await expect(alarm).toHaveAttribute('data-armed', '');
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, name);
      if (team === 3) {
        // A rival's alarm leads with Declare war while it is legal; Esc dismisses it.
        await expect(alarm.locator('.alarm-primary')).toHaveAccessibleName(/^Declare war on/);
        await page.keyboard.press('Escape');
      } else await alarm.getByRole('button', { name: /^See the race/ }).click();
      await expect(alarm).toHaveCount(0);
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ vpExpanded: false }));
    }
    await stageHoldReport(page, [holdBroken(3)], { reportOpen: true });
    await expect(page.getByRole('complementary', { name: 'Turn report' }).locator('.rline', { hasText: /was broken after 6 of 10 turns/ })).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '45-hold-broken-report');
  });

  test('VP race: your own Byzantium goal says whether the Heir is trained and who holds him', async ({ page }) => {
    await loadFixture(page, coreFixture(researchFixture('Byzantium')), 2);
    await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'Score ledger', exact: true }).click();
    const sheet = page.getByTestId('objective-sheet');
    await expect(sheet).toBeVisible();
    const goal = sheet.locator('.sl-row.k-heldGoal');
    await expect(goal).toContainText('The Heir to the Throne');
    await goal.locator('button').click();
    await expect(sheet.locator('#sl-detail-held-goal')).toContainText('Train the Heir and keep him');
    await expect(sheet.locator('#sl-detail-held-goal')).toContainText('Nobody holds the Heir now');
    await shot(page, size, '75-vp-byzantium-goal-expanded');
  });

  test('an eliminated empire leaves the race and the diplomacy list (it stays only in history)', async ({ page }) => {
    await loadFixture(page, eliminatedCoreSave(), ELIMINATED_TURN);
    const race = page.getByRole('region', { name: 'Victory race', exact: true });
    // Six empires, one knocked out: five rows, the fallen Athens among none of them.
    await expect(race.locator('.vp-row')).toHaveCount(5);
    await expect(race).not.toContainText('Athens');
    await shot(page, size, '76-eliminated-race');
    await race.getByRole('button', { name: 'Score ledger', exact: true }).click();
    await expect(page.getByTestId('objective-sheet')).toBeVisible();
    await expect(page.getByTestId('objective-sheet')).not.toContainText('Athens');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '77-eliminated-race-expanded');
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('objective-sheet')).toHaveCount(0);
    await revealSidebar(page, 'standings');
    await page.getByRole('button', { name: /^Diplomacy( \(\d+\))?$/ }).click();
    const empires = page.getByRole('list', { name: 'Empires' });
    await expect(empires.locator('.empire-row')).toHaveCount(4);
    await expect(empires).not.toContainText('Athens');
    await expect(page.locator('.betrayal-line')).toHaveCount(0);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '78-eliminated-diplomacy');
  });

  // The victory race as holdings (round-2 C): tokens per empire, the Nobody row, and a token's card.
  test('victory race holdings with 2 empires, and a token card', async ({ page }) => {
    await page.addInitScript(() => { localStorage.setItem('legatum.dev.no-era-hud', '1'); });
    await loadFixture(page, coreFixture(progressFixture().fixture), 2);
    await stageHoldings(page);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '79a-vp-holdings-2-empires');
    await revealSidebar(page, 'standings');
    await page.locator('.vp-row.mine .vhc-tok.live').first().hover();
    await expect(page.locator('.vhc-pop')).toBeVisible();
    await shot(page, size, '79b-vp-token-card-title-2-empires');
    await page.mouse.move(5, size.height / 2);
    await page.locator('.vhc-nobody .vhc-open-tok').last().hover();
    await expect(page.locator('.vhc-pop .vh-card.own-open')).toBeVisible();
    await shot(page, size, '79c-vp-token-card-nobody-2-empires');
  });

  test('victory race holdings with 8 empires, and a token card', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('legatum.dev.no-era-hud', '1');
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByLabel('Map seed', { exact: true }).fill('424242');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await stageHoldings(page);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '79d-vp-holdings-8-empires');
    await stageHold(page, { mine: true });
    await shot(page, size, '79g-vp-hold-countdown-8-empires');
    await page.locator('.vp-row:not(.mine) .vhc-tok').first().hover();
    await expect(page.locator('.vhc-pop')).toBeVisible();
    await shot(page, size, '79e-vp-token-card-rival-8-empires');
    await page.mouse.move(5, size.height / 2);
    await revealSidebar(page, 'standings');
    await page.locator('.vp-row.mine .vhc-goal-open').first().hover();
    await expect(page.locator('.vhc-pop .vh-card.k-goal')).toBeVisible();
    // Your own goal's meter comes from the core: a card for your own goal carries it.
    await expect(page.locator('.vhc-pop .vh-meter')).toHaveCount(1);
    await shot(page, size, '79f-vp-token-card-own-goal-8-empires');
  });

  // The score ledger opens from the top bar's VP chip, in both scenarios.
  test('score ledger with 2 empires, from the VP chip', async ({ page }) => {
    await loadFixture(page, coreFixture(progressFixture().fixture), 2);
    await stageHoldings(page);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '80a-vp-chip-2-empires');
    await page.locator('button.vh-chip').click();
    await expect(page.getByTestId('objective-sheet')).toBeVisible();
    await page.mouse.move(5, size.height - 5);
    await expect(page.locator('.sl-col')).toHaveCount(2);
    await shot(page, size, '80b-score-ledger-2-empires');
  });

  test('score ledger with 8 empires, from the VP chip', async ({ page }) => {
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    // A new lobby has 6 empires; this shot is the full table of 8, so add the last two.
    for (let i = 0; i < 2; i++) await page.getByRole('button', { name: /^\+ Add an AI empire/ }).click();
    await page.getByLabel('Map seed', { exact: true }).fill('424242');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await stageHoldings(page);
    await page.locator('button.vh-chip').click();
    await expect(page.getByTestId('objective-sheet')).toBeVisible();
    await page.mouse.move(5, size.height - 5);
    await expect(page.locator('.sl-col')).toHaveCount(8);
    await shot(page, size, '80c-score-ledger-8-empires');
  });

  test('Switzerland hoard: the live goal meter held and lost, and the goal-lost report line', async ({ page }) => {
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByLabel('Map seed', { exact: true }).fill('424242');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    // Render-only staging: you are the Switzerland, with the hoard held (5,040 of 5,000) or lost (4,760).
    const stage = (held: boolean) => page.evaluate((held) => {
      const store = (window as unknown as GameWindow).legatum, st = store.getState();
      const v = structuredClone(st.views![st.seat]);
      v.players = v.players.map((p) => (p.id === v.me ? { ...p, faction: 'Switzerland' as const, faction_goal_done: held, vp_sources: held ? [[{ kind: 'FactionGoal' as const, of: v.me }, 2] as [{ kind: 'FactionGoal'; of: number }, number]] : [] } : p));
      // The HUD shows the same Gold and the same 2 VP as the card.
      v.economy = { ...v.economy, gold: held ? 5040 : 4760 };
      v.standings = v.standings.map((t) => (t.team === v.team ? { ...t, vp: held ? 2 : 0 } : t));
      v.faction_goal = { value: held ? 5040 : 4760, target: 5000, label: held ? '5,040 of 5,000 Gold' : '4,760 of 5,000 Gold', live: true, done: held };
      store.setState({ views: st.views!.map((x, i) => (i === st.seat ? v : x)), seat: st.seat });
    }, held);
    await page.mouse.move(5, size.height / 2);
    await stage(true);
    await revealSidebar(page, 'standings');
    await page.locator('.vp-row.mine [data-source]').first().hover();
    await expect(page.locator('.vhc-pop .vh-card.k-goal')).toContainText(factionGoalRule('Switzerland'));
    await expect(page.locator('.vhc-pop .vh-card.k-goal')).not.toContainText(/Quick|Standard/);
    await expect(page.locator('.vhc-pop .vh-card.k-goal')).toContainText('lost the turn it falls below 5,000');
    await expect(page.locator('.vhc-pop [role="progressbar"]')).toHaveAttribute('aria-valuenow', '5000');
    await shot(page, size, '79h-switzerland-hoard-held');
    await page.mouse.move(5, size.height / 2);
    await stage(false);
    await revealSidebar(page, 'standings');
    await page.locator('.vp-row.mine .vhc-goal-open').first().hover();
    await expect(page.locator('.vhc-pop .vh-card.k-goal')).toContainText('4,760 of 5,000 Gold');
    await shot(page, size, '79i-switzerland-hoard-lost');
    await page.mouse.move(5, size.height / 2);
    // The turn report: the loss is a bad line for you, then the hoard returns as a plain line.
    const view = await page.evaluate(() => { const st = (window as unknown as GameWindow).legatum.getState(); return st.views![st.seat]; });
    const events = [
      { type: 'FactionGoalLost' as const, player: view.me, faction: 'Switzerland' as const, vp: 2 },
      { type: 'FactionGoalCompleted' as const, player: view.me, faction: 'Switzerland' as const, vp: 2, again: true },
    ];
    const report = { turn: view.turn, hash: '', events: [events], moments: [[]], alarms: [[]], lines: [formatEvents(events, contextFromViews(view, view))] };
    await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [], reportOpen: true }), report);
    const reportPanel = page.getByRole('complementary', { name: 'Turn report' });
    await expect(reportPanel).toContainText('lost The Reserve');
    await expect(reportPanel).toContainText('regained The Reserve');
    await shot(page, size, '79j-goal-lost-report-line');
  });
});

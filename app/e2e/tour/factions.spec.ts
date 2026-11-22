import { revealSidebar } from '../sidebarAssertions';
// Screen tour, faction goals and signature mechanics. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/factions`. Shots land in test-results/screen-tour/<size>/factions/.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { stageHeirHolder } from '../vpStage';
import { forgeHeirFixture } from '../../src/test/forgeHeirFixture';
import { broadcastCoreSave, freeTechsCoreSave, retireCoreSave, sakokuCoreSave, wildClearedCoreSave, wildCoreSave } from '../../src/test/goalActionsFixture';
import { SIZES, aim, centred, dismissCelebrations, expectDockWhole, expectRaceWhole, loadFixture, openReport, primaries, selectThing, shot, showMoment, type GameWindow } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });


  // Germany's live Foundry goal and Byzantium's Heir.
  test('Germany: five Ruhr Foundries score the live goal', async ({ page }) => {
    const forge = forgeHeirFixture('Germany');
    await loadFixture(page, coreFixture(forge.fixture), forge.fixture.turn);
    await selectThing(page, 'city', forge.capital);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '90a-germany-city-without-foundry');
    await selectThing(page, 'city', forge.forged[0]);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '90b-germany-city-with-foundry');
    await revealSidebar(page, 'standings');
    await page.locator('.vp-row.mine .vhc-goal-open').first().hover();
    await expect(page.locator('.vhc-pop .vh-card.k-goal .vh-meter')).toContainText('4 of 5 Ruhr Foundries');
    await shot(page, size, '90c-germany-goal-meter');
    await page.mouse.move(5, size.height / 2);
    await stageHeirHolder(page, { holder: 2, of: 1 });
    await shot(page, size, '90d-race-heir-held-by-rival');
    await page.locator('.panel.vp .vhc-tok[data-held]').first().hover();
    await expect(page.locator('.vhc-pop .vh-card.k-goal .vh-held')).toBeVisible();
    await shot(page, size, '90e-race-heir-card');
  });

  test('Byzantium: the Heir card, the Heir at home, and the Heir captured by a rival', async ({ page, browser }) => {
    test.setTimeout(180_000);
    const none = forgeHeirFixture('Byzantium', 'none');
    await loadFixture(page, coreFixture(none.fixture), 2);
    await selectThing(page, 'city', none.capital);
    const card = page.locator('.city-panel .production-choice[data-item="u:HeirToTheThrone"]');
    await card.scrollIntoViewIfNeeded();
    await expect(card).toBeVisible();
    await expect(card.locator('.btn-buy')).toBeDisabled();
    const meta = await page.locator('.city-panel .cp-meta > *').evaluateAll((els) => els.map((el) => {
      const r = el.getBoundingClientRect(); return { left: r.left, right: r.right, top: r.top, bottom: r.bottom };
    }));
    for (let i = 1; i < meta.length; i++) {
      if (meta[i].top < meta[i - 1].bottom) expect(meta[i].left - meta[i - 1].right, 'city stats have visible space').toBeGreaterThanOrEqual(6);
    }
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '91a-byzantium-heir-card');
    await revealSidebar(page, 'standings');
    await page.locator('.vp-row.mine .vhc-goal-open').first().hover();
    await expect(page.locator('.vhc-pop .vh-card.k-goal .vh-held')).toBeVisible();
    await shot(page, size, '91f-byzantium-heir-goal-card-nobody');

    for (const [heir, id] of [['own', '91b-heir-own-map-and-panel'], ['captured', '91c-heir-captured-map-and-panel']] as const) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        const fx = forgeHeirFixture('Byzantium', heir);
        await loadFixture(p, coreFixture(fx.fixture), 2);
        await selectThing(p, 'unit', fx.heirUnit!);
        await expect(p.getByTestId('heir-note')).toBeVisible();
        await p.mouse.move(5, size.height / 2);
        await shot(p, size, id);
        if (heir === 'own') {
          // Disband asks first and says what it forfeits.
          await p.locator('.unit-panel button.act', { hasText: /^Disband/ }).click();
          await expect(p.getByTestId('heir-disband-confirm')).toContainText('forfeit 2 VP');
          await p.mouse.move(5, size.height / 2);
          await shot(p, size, '91d-heir-disband-confirm');
          await p.getByRole('button', { name: 'Keep him' }).click();
          await selectThing(p, 'city', fx.capital);
          const blocked = p.locator('.city-panel .production-choice[data-item="u:HeirToTheThrone"]');
          await blocked.scrollIntoViewIfNeeded();
          await expect(blocked).toContainText('The Heir to the Throne still lives');
          await p.mouse.move(5, size.height / 2);
          await shot(p, size, '91e-heir-card-blocked');
        } else {
          await stageHeirHolder(p, { holder: 1, of: 0 });
          await p.mouse.move(5, size.height / 2);
          await shot(p, size, '91g-race-own-heir-held-by-rival');
          // A military unit beside him: his tile offers the capture.
          await selectThing(p, 'unit', fx.spearman!);
          const capture = p.getByTestId('heir-capture');
          await aim(p, fx.heirHex!, capture);
          await expect(capture).toHaveText('Capture the Heir (+2 VP)');
          await shot(p, size, '91h-heir-capture-tile-card');
          // Zoomed out, units are pips: the Heir's wears a gilt ring.
          await p.mouse.move(size.width / 2, size.height / 2);
          for (let i = 0; i < 3; i++) await p.mouse.wheel(0, 300);
          await p.mouse.move(5, size.height / 2);
          await shot(p, size, '91i-heir-pip-zoomed-out');
        }
      } finally { await context.close(); }
    }
  });

  test('Sparta 7b: Agoge in the production picker, older-save message', async ({ browser }) => {
    // Agoge: an Sparta city whose picker offers it (needs Chivalry).
    const ctx = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p2 = await ctx.newPage();
      const w = wallsFixture();
      w.fixture.players[0].faction = 'Sparta';
      w.fixture.players[0].techs = [...w.fixture.players[0].techs, 'Chivalry'];
      const city = w.fixture.cities.find((c) => c.id === w.own)!;
      Object.assign(city, { population: 3, producing: null, prod: 0, productionQueue: [], productionBanked: [], buildings: [] });
      await loadFixture(p2, coreFixture(w.fixture), 2);
      await selectThing(p2, 'city', w.own);
      const agoge = p2.locator('.city-panel .production-choice[data-item="b:Agoge"]');
      await agoge.scrollIntoViewIfNeeded();
      await expect(agoge).toBeVisible();
      await shot(p2, size, '67-sparta-war-shrine-picker');
    } finally { await ctx.close(); }

    // The title screen explains an older save that no longer loads, in plain words.
    const old = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p3 = await old.newPage();
      const game = coreFixture(wallsFixture().fixture).replace(/"save_version":\d+/, '"save_version":5');
      const save = JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], playtest: 'Playtest 8.9.20', savedAt: new Date().toISOString(), game });
      await p3.addInitScript((save) => { if (location.protocol.startsWith('http')) localStorage.setItem('legatum.autosave.v1', save); }, save);
      await p3.goto('/?client=wasm');
      await expect(p3.getByRole('button', { name: /^Saved game \(older version\)/ })).toBeDisabled();
      await shot(p3, size, '68-old-save-message');
    } finally { await old.close(); }
  });

  test('goal actions: Sakoku and the broadcast confirms, the received broadcast and its picker', async ({ page, browser }) => {
    test.setTimeout(240_000);
    const state = () => page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return { view: s.views![s.seat], orders: s.orders[s.seat] }; });
    await loadFixture(page, sakokuCoreSave(), 60);
    // The race panel's own-goal action, where the goals live.
    await revealSidebar(page, 'goal');
    const row = page.locator('[data-goal-action="Sakoku"]');
    await expect(row).toBeVisible();
    await expect(row.getByRole('button', { name: /^Close the country/ })).toBeEnabled();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '85-sakoku-action-row');
    await row.getByRole('button', { name: /^Close the country/ }).click();
    const dialog = page.locator('dialog[data-goal-dialog="Sakoku"][open]');
    await expect(dialog).toBeVisible();
    // The confirm lists exactly what closing blocks.
    await expect(dialog.locator('[data-testid="sakoku-blocks"] tbody th')).toHaveText(['War', 'Alliances', 'Tech diffusion', 'Foreign units']);
    await expect(dialog.getByRole('button', { name: 'Close the country' })).toBeFocused();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '86-sakoku-confirm');
    await dialog.getByRole('button', { name: 'Close the country' }).click();
    await expect(dialog).toHaveCount(0);
    await expect(row).toContainText('Ordered');
    expect((await state()).orders).toEqual([{ type: 'Action', action: { action: 'Sakoku', close: true } }]);
    await page.mouse.move(5, size.height / 2);
    await expectRaceWhole(page);
    await expectDockWhole(page);
    await shot(page, size, '87-sakoku-ordered');
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('61');
    {
      const { view } = await state();
      const me = view.players.find((p) => p.id === view.me)!;
      expect(me.sakoku_turns).toBe(1);
      expect(me.faction_goal_done).toBe(false);
      // Closed: the race row carries the public count, and the action row offers the reopening.
      await expect(page.locator('[data-testid="closed-chip"]').first()).toHaveText('1/30');
      await expect(page.locator('[data-goal-action="Sakoku"] button.goal-action-btn')).toHaveText(/Reopen the country/);
      await page.mouse.move(5, size.height / 2);
      await expectRaceWhole(page);
      await shot(page, size, '88-sakoku-closed');
    }

    const other = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p2 = await other.newPage();
      await loadFixture(p2, broadcastCoreSave(), 90);
      await revealSidebar(p2, 'goal');
      const r2 = p2.locator('[data-goal-action="Broadcast"]');
      await expect(r2.getByRole('button', { name: /^Share the Learning/ })).toBeEnabled();
      await r2.getByRole('button', { name: /^Share the Learning/ }).click();
      const d2 = p2.locator('dialog[data-goal-dialog="Broadcast"][open]');
      await expect(d2).toBeVisible();
      await expect(d2.getByRole('listitem').filter({ hasText: 'Celts' })).toHaveCount(1);
      await p2.mouse.move(5, size.height / 2);
      await shot(p2, size, '88-broadcast-confirm');
      await d2.getByRole('button', { name: 'Share the Learning' }).click();
      await expect(r2).toContainText('Ordered');
    } finally { await other.close(); }

    const rival = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p3 = await rival.newPage();
      await loadFixture(p3, freeTechsCoreSave(), 91);
      const prompt = p3.getByTestId('free-techs-prompt');
      await expect(prompt).toHaveAttribute('data-armed', '');
      await expect(prompt).toContainText('Choose 2 technologies');
      await p3.mouse.move(5, size.height / 2);
      await shot(p3, size, '89-free-techs-prompt');
      await prompt.getByRole('button', { name: /^Choose 2 technologies/ }).click();
      await expect(prompt).toHaveCount(0);
      const picker = p3.getByTestId('free-tech-picker');
      await expect(picker).toBeVisible();
      await expect(picker.getByRole('button', { name: /^Take 2 technologies/ })).toBeDisabled();
      await picker.locator('.ftp-tech.open').first().click();
      await picker.locator('.ftp-tech.open').first().click();
      await p3.mouse.move(5, size.height / 2);
      await shot(p3, size, '90-free-tech-picker');
      await expect(picker.getByRole('button', { name: /^Take 2 technologies/ })).toBeEnabled();
      await picker.getByRole('button', { name: /^Take 2 technologies/ }).click();
      await expect(picker).toHaveCount(0);
      const { orders } = await p3.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return { orders: s.orders[s.seat] }; });
      expect(orders[0]).toMatchObject({ type: 'Action', action: { action: 'ChooseFreeTechs' } });
      await endPlanning(p3);
      await expect(p3.getByTestId('turn-number')).toHaveText('92');
      const known = await p3.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat].research.known.length; });
      expect(known).toBe(5); // three known plus the two free picks (two of the era's five were unknown)
      // Three were granted but only two were left in the Ancient era: finishing it enters the Classical era, where the third is still owed.
      await expect(p3.getByTestId('free-techs-prompt')).toContainText('Choose 1 technology');
    } finally { await rival.close(); }
  });

  test('Sparta retires veterans (valid cities marked, confirm, the goal completes) and the Celts call the Wild', async ({ page, browser }) => {
    test.setTimeout(180_000);
    const units = () => page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      return v.units.filter((u) => u.owner === v.me).map((u) => ({ id: u.id, level: u.level ?? 0, hp: u.hp, pos: u.pos }));
    });
    await loadFixture(page, retireCoreSave(1), 130);
    await revealSidebar(page, 'goal');
    const row = page.locator('[data-goal-action="Retire"]');
    await expect(row).toContainText('1 of 3 veterans retired');
    // A veteran waiting in a city without an Agoge: the panel says why, and the map marks where he can retire.
    const all = await units();
    const away = all.find((u) => u.level === 5)!, ready = all.find((u) => u.level === 4 && u.hp === 100)!;
    await selectThing(page, 'unit', away.id);
    await centred(page, away.id);
    const retireBtn = page.locator('.unit-panel button.act', { hasText: /^Retire this veteran/ });
    await expect(retireBtn).toBeDisabled();
    await expect(page.locator('.unit-panel .act-reason', { hasText: 'no Agoge' })).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await expectRaceWhole(page);
    await shot(page, size, '93a-retire-veteran-away-shrine-city-marked');
    // A veteran standing in the Agoge city: the action is offered.
    await selectThing(page, 'unit', ready.id);
    await centred(page, ready.id);
    await expect(retireBtn).toBeEnabled();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '93b-retire-veteran-offered');
    await retireBtn.click();
    const dialog = page.locator('dialog[data-goal-dialog="RetireVeteran"][open]');
    await expect(dialog).toBeVisible();
    await expect(dialog.getByTestId('retire-progress-line')).toHaveText('2 of 3');
    await expect(dialog.getByRole('button', { name: 'Retire this veteran' })).toBeFocused();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '93c-retire-confirm');
    await dialog.getByRole('button', { name: 'Retire this veteran' }).click();
    await expect(dialog).toHaveCount(0);
    await expect(page.locator('.unit-panel .up-status')).toContainText('Retiring with honour');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '93d-retire-ordered');
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('131');
    await dismissCelebrations(page);
    await expect(page.locator('[data-goal-action="Retire"]')).toContainText('2 of 3 veterans retired');
    await expect(page.locator('.unit-panel')).toHaveCount(0);

    // The third retirement completes the goal: the legendary card, then the finished goal.
    const done = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await done.newPage();
      await loadFixture(p, retireCoreSave(2), 130);
      const vet = await p.evaluate(() => {
        const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
        return v.units.find((u) => u.owner === v.me && (u.level ?? 0) >= 4 && v.actions!.retire!.veterans.find((x) => x.unit === u.id)?.blocker === null)!.id;
      });
      await selectThing(p, 'unit', vet);
      await p.locator('.unit-panel button.act', { hasText: /^Retire this veteran/ }).click();
      const d2 = p.locator('dialog[data-goal-dialog="RetireVeteran"][open]');
      await expect(d2.getByTestId('retire-progress-line')).toHaveText('3 of 3');
      await expect(d2).toContainText('the last veteran owed');
      await p.mouse.move(5, size.height / 2);
      await shot(p, size, '93e-retire-last-veteran-confirm');
      await d2.getByRole('button', { name: 'Retire this veteran' }).click();
      await endPlanning(p);
      await expect(p.getByTestId('turn-number')).toHaveText('131');
      await showMoment(p, 'The veterans are honoured');
      await p.mouse.move(5, size.height / 2);
      await shot(p, size, '93f-retire-goal-complete-moment');
      await dismissCelebrations(p);
      const flag = await p.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; return v.players.find((x) => x.id === v.me)!.faction_goal_done; });
      expect(flag).toBe(true);
      await expect(p.locator('[data-goal-action]')).toHaveCount(0);
      await openReport(p);
      await expect(p.getByRole('complementary', { name: 'Turn report' })).toContainText('3 of 3 toward Honour the Veterans');
      await shot(p, size, '93g-retire-goal-report');
    } finally { await done.close(); }

    // The Celts: Call the Wild in the race panel, the confirm, the camps ghosted on the map.
    const wild = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await wild.newPage();
      await loadFixture(p, wildCoreSave(), 100);
      await revealSidebar(p, 'goal');
      const wrow = p.locator('[data-goal-action="CallTheWild"]');
      await expect(wrow).toContainText('0 of 3 cities have a wild camp');
      await p.mouse.move(5, size.height / 2);
      await expectRaceWhole(p);
      await expectDockWhole(p);
      await shot(p, size, '93h-wild-action-row');
      await wrow.getByRole('button', { name: /^Raise the Warbands/ }).click();
      const wd = p.locator('dialog[data-goal-dialog="CallTheWild"][open]');
      await expect(wd.locator('tbody tr')).toHaveCount(3);
      await expect(wd.getByRole('button', { name: 'Raise the Warbands' })).toBeFocused();
      await p.mouse.move(5, size.height / 2);
      await shot(p, size, '93i-wild-confirm');
      await wd.getByRole('button', { name: 'Raise the Warbands' }).click();
      await expect(wrow).toContainText('Ordered');
      await p.evaluate(() => { const g = (window as unknown as GameWindow).legatum.getState(); g.select(null); g.jumpTo({ q: 9, r: 11 }); });
      await p.mouse.move(5, size.height / 2);
      await shot(p, size, '93j-wild-ordered-camps-marked');
      await endPlanning(p);
      await expect(p.getByTestId('turn-number')).toHaveText('101');
      await showMoment(p, 'The warbands gather');
      await p.mouse.move(5, size.height / 2);
      await shot(p, size, '93k-wild-called-moment');
      await dismissCelebrations(p);
      await expect(wrow).toContainText('3 of 3 cities have a wild camp');
      await p.mouse.move(5, size.height / 2);
      await shot(p, size, '93l-wild-camps-standing');
    } finally { await wild.close(); }

    // A rival clears a camp: the Celts lose the points, with a sober card and a report line.
    const cleared = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await cleared.newPage();
      await loadFixture(p, wildClearedCoreSave(), 101);
      await endPlanning(p);
      await expect(p.getByTestId('turn-number')).toHaveText('102');
      await showMoment(p, 'A warband camp is cleared');
      await p.mouse.move(5, size.height / 2);
      expect(await primaries(p), 'one primary with a moment card up').toBe(1);
      await shot(p, size, '93m-wild-camp-cleared-moment');
      await dismissCelebrations(p);
      await expect(p.locator('[data-goal-action="CallTheWild"]')).toContainText('2 of 3 cities have a wild camp');
      await openReport(p);
      await expect(p.getByRole('complementary', { name: 'Turn report' })).toContainText('Raise the Warbands again');
      await shot(p, size, '93n-wild-camp-cleared-report');
    } finally { await cleared.close(); }
  });
});

// Screens changed on train-1101d that had no tour shots: the city panel's yields row, Now producing with its stop button and
// Compare cities; Heal beside Fortify, the healing unit, and "healed to full, needs orders" in the queue and report; a move
// that follows a friend through a choke; the original-capital VP (forecast foot, event card, report line and ledger row);
// and the research screen once every technology is known (the Science choice). Run alone: `npx playwright test e2e/tour/train-1101d`.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { TECHS } from '../../src/data';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { turnFlowFixture } from '../../src/test/turnFlowFixture';
import { pathDetourFixture } from '../../src/test/pathDetourFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { refreshVision } from '../../src/test/fixtureEngine';
import type { Event, Hex, Order } from '../../src/types';
import { SIZES, aim, centred, closeReportIfOpen, dismissCelebrations, hexPoint, loadFixture, posOf, selectThing, shot, showMoment, stageHallReport, type GameWindow } from './helpers';

async function reportNow(page: Page) {
  await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect(report).toBeVisible();
  return report;
}

const park = (page: Page, size: { width: number; height: number }) => page.mouse.move(5, size.height / 2);

for (const size of SIZES) test.describe(`train 1101d ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('city panel: yields row, Now producing with stop, Compare cities, improvement gains', async ({ page }) => {
    const s = improvementCity({ techs: ['Masonry', 'IronWorking'] });
    s.cities[0].producing = { kind: 'Building', building: 'Granary' };
    s.cities[0].prod = 12;
    s.cities[0].productionQueue = [{ kind: 'Unit', unit: 'Soldier' }] as never;
    await loadFixture(page, coreFixture(s), 12);
    await selectThing(page, 'city', s.cities[0].id);
    const panel = page.locator('.city-panel');
    await expect(panel.getByTestId('city-yields')).toBeVisible();
    await park(page, size);
    await shot(page, size, 'd01-city-yields-producing');
    // Small controls are at least 24 px each way (stop, Compare cities).
    for (const control of [panel.getByTestId('production-stop'), panel.getByRole('button', { name: /Compare cities/ })]) {
      if (!(await control.isVisible())) continue;
      const box = (await control.boundingBox())!;
      expect(box.height, 'click target height').toBeGreaterThanOrEqual(24);
      expect(box.width, 'click target width').toBeGreaterThanOrEqual(24);
    }
    // Hover the stop button (its tooltip says what stopping does).
    const stop = panel.getByTestId('production-stop');
    if (await stop.isVisible()) {
      await stop.hover();
      await page.waitForTimeout(700);
      await shot(page, size, 'd02-city-stop-hover');
      await stop.click();
      await park(page, size);
      await shot(page, size, 'd03-city-after-stop');
    }
    // The yields row hover (Production breakdown).
    await panel.locator('.cp-yields .cp-yield').nth(1).hover();
    await page.waitForTimeout(500);
    await shot(page, size, 'd04-city-yield-hover');
    await park(page, size);
    // Improvement cards: the Quarry and Mine summary lines.
    const quarry = panel.locator('.production-choice', { hasText: /Quarry/ }).first();
    if (await quarry.count()) {
      await quarry.scrollIntoViewIfNeeded();
      await park(page, size);
      await shot(page, size, 'd05-city-improvement-cards');
    }
    const mine = panel.locator('.production-choice', { hasText: /Mine, Iron/ }).first();
    if (await mine.count()) {
      await mine.scrollIntoViewIfNeeded();
      await park(page, size);
      await shot(page, size, 'd06-city-improvement-iron-mine');
    }
    // Compare cities opens the overview.
    await panel.evaluate((el) => { el.scrollTop = 0; });
    await panel.getByRole('button', { name: /Compare cities/ }).click();
    await expect(page.getByTestId('cities-overview')).toBeVisible();
    await park(page, size);
    await shot(page, size, 'd07-compare-cities');
  });

  test('heal: Heal beside Fortify, the healing unit, healed to full in the queue and report', async ({ page }) => {
    const f = turnFlowFixture();
    f.players[1].isHuman = false;
    f.setup.players[1].is_human = false;
    await loadFixture(page, coreFixture(f), 2);
    await selectThing(page, 'unit', 0);
    await expect(page.locator('.unit-panel button.act', { hasText: /^Heal/ })).toBeVisible();
    await park(page, size);
    await shot(page, size, 'd10-unit-heal-offered');
    await page.locator('.unit-panel button.act', { hasText: /^Heal/ }).hover();
    await page.waitForTimeout(700);
    await shot(page, size, 'd11-unit-heal-tooltip');
    await page.locator('.unit-panel button.act', { hasText: /^Heal/ }).click();
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const o = s.orders[s.seat].find((o) => o.type === 'SleepUnit')!;
      s.select({ kind: 'unit', id: (o as { unit: number }).unit });
    });
    await park(page, size);
    // The standing order is visible in the pair: a filled accent with a check, on computed style (a pair rule once hid .act.active).
    const healBtn = page.locator('.unit-panel button.act', { hasText: /^Heal/ });
    const fortifyBtn = page.locator('.unit-panel button.act', { hasText: /^Fortify/ });
    await expect(healBtn).toHaveAttribute('aria-pressed', 'true');
    const fill = (b: typeof healBtn) => b.evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(await fill(healBtn), 'the active order is filled differently from the idle one').not.toBe(await fill(fortifyBtn));
    expect(await fill(healBtn), 'the active fill is opaque').not.toMatch(/rgba\(.*, 0\)$|transparent/);
    await expect(healBtn.locator('.action-check')).toBeVisible();
    await shot(page, size, 'd12-unit-healing');
    // Close-up of the healing marker on the map.
    await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); s.select(null); });
    await page.evaluate(() => { const m = (window as unknown as GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } }).legatumMap; m.camera.zoom = 2.2; m.redraw(); });
    await hexPoint(page, await posOf(page, 'unit', 0));
    await park(page, size);
    await shot(page, size, 'd13-map-healing-marker');
  });

  test('heal: a standing Heal and a standing Fortify from an earlier turn use one sentence, in neutral ink', async ({ page }) => {
    for (const [order, text, shotName] of [['Heal', 'Healing until full health, then asks for orders', 'd12b-unit-healing-standing'], ['Sleep', 'Fortified until woken', 'd12c-unit-fortified-standing']] as const) {
      const f = turnFlowFixture();
      f.players[1].isHuman = false;
      f.setup.players[1].is_human = false;
      // The order was given on an earlier turn: it is the unit's standing order, with no pending order this turn.
      f.units.find((u) => u.id === 0)!.standing_order = order;
      await loadFixture(page, coreFixture(f), 2);
      await selectThing(page, 'unit', 0);
      await park(page, size);
      const status = page.locator('.unit-panel .up-status');
      await expect(status).toHaveText(text, { ignoreCase: true });
      await expect(status).toHaveClass(/standing/);
      // Neutral ink, not the accent red of "Needs orders" (computed, since both are text colours on the same card).
      const [ink, accent] = await status.evaluate((el) => {
        const probe = (v: string) => { const d = document.createElement('i'); d.style.color = `var(${v})`; document.body.append(d); const c = getComputedStyle(d).color; d.remove(); return c; };
        return [getComputedStyle(el).color, probe('--accent')];
      });
      expect(ink, 'a standing order is not drawn in the needs-orders colour').not.toBe(accent);
      await shot(page, size, shotName);
    }
  });

  test('heal: a unit that healed to full returns to the queue and the report says so', async ({ page }) => {
    const f = turnFlowFixture();
    f.players[1].isHuman = false;
    f.setup.players[1].is_human = false;
    f.units.find((u) => u.id === 0)!.hp = 100;
    await loadFixture(page, coreFixture(f), 2);
    const at = await posOf(page, 'unit', 0);
    const events: Event[] = [{ type: 'UnitHealed', unit: 0, owner: 0, unit_kind: 'Soldier', at } as Event];
    await stageHallReport(page, events);
    await dismissCelebrations(page);
    await park(page, size);
    await shot(page, size, 'd14-healed-to-full-queue');
    const report = await reportNow(page);
    await expect(report).toContainText('healed to full');
    await park(page, size);
    await shot(page, size, 'd15-healed-to-full-report');
    await closeReportIfOpen(page);
  });

  test('move preview: a unit follows a friend through a choke', async ({ page }) => {
    const setup = pathDetourFixture();
    const f = setup.fixture;
    const leader = f.units.find((u) => u.id === setup.attacker)!;
    const follower = f.units.find((u) => u.id === setup.left)!;
    leader.pos = { q: 20, r: 15 };
    follower.pos = { q: 20, r: 14 };
    refreshVision(f);
    await loadFixture(page, coreFixture(f), 38);
    const pending: Order[] = [{ type: 'MoveUnitPath', unit: setup.attacker, to: { q: 22, r: 13 }, path: [setup.gap, { q: 22, r: 13 }] }];
    await page.evaluate((orders) => (window as unknown as GameWindow).legatum.getState().editOrders(() => orders), pending);
    await selectThing(page, 'unit', setup.left);
    await centred(page, setup.left);
    const target: Hex = setup.target;
    await aim(page, target, page.locator('.forecast'));
    await page.waitForTimeout(400);
    await shot(page, size, 'd20-move-follows-friend');
  });

  test('capital VP: forecast foot, event card, report line', async ({ page }) => {
    const setup = wallsFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const forecast = page.locator('.forecast');
    await selectThing(page, 'unit', setup.nearWalled);
    await centred(page, setup.nearWalled);
    await aim(page, await posOf(page, 'city', setup.walled), forecast);
    await page.waitForTimeout(300);
    await shot(page, size, 'd30-forecast-capital-vp');
    await page.mouse.move(5, size.height / 2);
    const city = await page.evaluate((id) => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat].cities.find((c) => c.id === id)!; }, setup.walled);
    const events: Event[] = [
      { type: 'CityCaptured', city: setup.walled, from: 1, to: 0 } as unknown as Event,
      { type: 'CapitalTaken', captor: 0, capital: setup.walled, name: city.name } as Event,
    ];
    // The staged event must agree with the public score already visible behind its card.
    await page.evaluate(({ id, name }) => {
      const store = (window as unknown as GameWindow).legatum, st = store.getState();
      const views = st.views!.map((v) => ({ ...v,
        players: v.players.map((p) => p.id === 0
          ? { ...p, capitals_taken: [[id, name]], vp: (p.vp ?? 0) + 1, vp_sources: [...(p.vp_sources ?? []), [{ kind: 'CapitalTaken', capital: id }, 1]] } : p),
        standings: v.standings.map((row) => row.members.includes(0) ? { ...row, vp: row.vp + 1 } : row),
      }));
      store.setState({ views: views as never });
    }, { id: setup.walled, name: city.name });
    await stageHallReport(page, events);
    await showMoment(page, city.name);
    await park(page, size);
    await shot(page, size, 'd31-capital-taken-card');
    await dismissCelebrations(page);
    const report = await reportNow(page);
    await expect(report).toContainText('original capital');
    await park(page, size);
    await shot(page, size, 'd32-capital-taken-report');
    await closeReportIfOpen(page);
    // A rival's capture of a third empire's capital: the card from the rival's side.
    const rival: Event[] = [
      { type: 'CityCaptured', city: setup.open, from: 1, to: 2 } as unknown as Event,
      { type: 'CapitalTaken', captor: 2, capital: setup.open, name: 'Rival capital' } as Event,
    ];
    await stageHallReport(page, rival);
    await showMoment(page, 'Rival capital');
    await park(page, size);
    await shot(page, size, 'd33-capital-taken-rival-card');
    await dismissCelebrations(page);
    // The ledger with a capital taken.
    await page.locator('button.vh-chip').click();
    const sheet = page.getByTestId('objective-sheet');
    await expect(sheet.locator('.sl-table')).toBeVisible();
    await sheet.locator('[data-row="capitals-held"]').scrollIntoViewIfNeeded();
    await park(page, size);
    await shot(page, size, 'd34-ledger-capitals-held');
  });

  test('capital VP: the forecast line on a rival original capital', async ({ page }) => {
    const setup = wallsFixture();
    // The fixture carries the original-capital flag into the core, so the screen is staged by the rules, not by patching the view.
    setup.fixture.cities.find((c) => c.id === setup.walled)!.originalCapital = true;
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const forecast = page.locator('.forecast');
    await selectThing(page, 'unit', setup.nearWalled);
    await centred(page, setup.nearWalled);
    await aim(page, await posOf(page, 'city', setup.walled), forecast);
    await page.waitForTimeout(300);
    await shot(page, size, 'd35-forecast-capital-vp-staged');
  });

  test('research: every technology known, the Science choice', async ({ page }) => {
    const f = researchFixture('India');
    f.players[0].techs = [...TECHS];
    f.players[0].research = null;
    f.players[0].research_queue = [];
    await loadFixture(page, coreFixture(f), 2);
    await page.locator('.topbar .research-chip').click();
    const research = page.locator('dialog.tech-modal');
    await expect(research).toBeVisible();
    await park(page, size);
    await shot(page, size, 'd40-research-all-known');
    const prod = research.getByRole('button', { name: /^Production/ });
    if (await prod.isVisible()) { await prod.click(); await park(page, size); await shot(page, size, 'd41-research-science-to-production'); }
  });

  test('research: the Future era cards', async ({ page }) => {
    const f = researchFixture('India');
    f.players[0].techs = TECHS.slice(0, 30);
    await loadFixture(page, coreFixture(f), 2);
    await page.locator('.topbar .research-chip').click();
    const research = page.locator('dialog.tech-modal');
    await expect(research).toBeVisible();
    await park(page, size);
    await shot(page, size, 'd42-research-future-era');
    await research.locator('.research-scroll').evaluate((el) => { el.scrollTop = 400; });
    await park(page, size);
    await shot(page, size, 'd43-research-future-era-scrolled');
  });

  test('research: the Modern era cards', async ({ page }) => {
    const f = researchFixture('Germany');
    f.players[0].techs = TECHS.slice(0, 25);
    await loadFixture(page, coreFixture(f), 2);
    await page.locator('.topbar .research-chip').click();
    await expect(page.locator('dialog.tech-modal')).toBeVisible();
    await park(page, size);
    await shot(page, size, 'd44-research-modern-era');
  });

  test('research: the Medieval and Renaissance era cards', async ({ page }) => {
    for (const [n, name] of [[10, 'd45-research-medieval-era'], [15, 'd46-research-renaissance-era'], [20, 'd47-research-industrial-era'], [5, 'd48-research-classical-era']] as const) {
      const ctx = await page.context().browser()!.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await ctx.newPage();
        const f = researchFixture('Athens');
        f.players[0].techs = TECHS.slice(0, n);
        await loadFixture(p, coreFixture(f), 2);
        await p.locator('.topbar .research-chip').click();
        await expect(p.locator('dialog.tech-modal')).toBeVisible();
        await park(p, size);
        await shot(p, size, name);
      } finally { await ctx.close(); }
    }
  });
});

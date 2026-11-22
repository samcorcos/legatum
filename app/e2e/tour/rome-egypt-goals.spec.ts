// Screen tour, Rome's Colonia Victrix and Egypt's Royal Tomb: the goal row in each state, the level 3 announcement,
// the Settle lens's gilt rings, a rival's view of each goal city, the tomb's work, the faction sheets and the Guide.
// Run alone with `npx playwright test e2e/tour/rome-egypt-goals`. Shots land in test-results/screen-tour/<size>/rome-egypt-goals/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { goalCitiesFixture, type GoalCityMode } from '../../src/test/goalCitiesFixture';
import { revealSidebar } from '../sidebarAssertions';
import { SIZES, aim, dismissCelebrations, frameHall, hexPoint, loadFixture, selectThing, shot, stageHallReport, type GameWindow } from './helpers';
import type { Browser, Page } from '@playwright/test';

test.describe.configure({ mode: 'parallel' });

async function open(page: Page, mode: GoalCityMode) {
  const f = goalCitiesFixture(mode);
  await loadFixture(page, coreFixture(f.fixture), f.fixture.turn);
  await dismissCelebrations(page);
  return f;
}

// Park the pointer on the sidebar's own top-left corner, where no card opens (the map would open a tile card).
async function away(page: Page, h: number) {
  const bar = page.locator('.right-sidebar');
  const box = await bar.count() ? await bar.boundingBox() : null;
  await page.mouse.move(box ? box.x + 2 : 5, box ? box.y + 2 : h / 2);
}

// One more state in a fresh window of the same size.
async function withMode(browser: Browser, size: { width: number; height: number }, mode: GoalCityMode, fn: (p: Page, f: ReturnType<typeof goalCitiesFixture>) => Promise<void>) {
  const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
  try {
    const p = await context.newPage();
    await fn(p, await open(p, mode));
  } finally { await context.close(); }
}

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('Rome: the goal row before, ready and held', async ({ page, browser }) => {
    test.setTimeout(150_000);
    await open(page, 'romeBefore');
    const row = page.locator('[data-goal-action="Colonia"]');
    await expect(row).toHaveAttribute('data-colonia-state', 'before');
    await expect(row).toContainText('level 2 of 3');
    await row.scrollIntoViewIfNeeded();
    await away(page, size.height);
    await shot(page, size, 'r01-rome-row-before');
    for (const [mode, state, name] of [['romeReady', 'ready', 'r02-rome-row-ready'], ['romeHeld', 'held', 'r03-rome-row-held'], ['romeLost', 'lost', 'r04-rome-row-lost']] as const) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        await open(p, mode);
        const r = p.locator('[data-goal-action="Colonia"]');
        await expect(r).toHaveAttribute('data-colonia-state', state);
        await r.scrollIntoViewIfNeeded();
        await away(p, size.height);
        await shot(p, size, name);
      } finally { await context.close(); }
    }
  });

  test('Rome: the level 3 announcement, the Settle lens, the site card and the found button', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await open(page, 'romeReady');
    // The dock offers the goal once, as This Turn's next step.
    const cta = page.locator('.right-sidebar .cta');
    // Nothing selected: the dock's offer is the turn's one primary.
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ selection: null }));
    await expect(cta).toContainText('Veteran ready: Colonia Victrix');
    await expect(cta).toContainText('Select the veteran');
    await away(page, size.height);
    await shot(page, size, 'r05-rome-level3-announcement');
    await selectThing(page, 'unit', f.veteran!);
    // Selecting a founder turns the Settle lens on; open its legend.
    const settle = page.getByRole('radio', { name: 'Settle', exact: true });
    if (!await settle.isVisible()) await page.getByRole('button', { name: /^Lens: Settle/ }).click();
    await expect(settle).toBeChecked();
    await expect(page.getByText(/Gilt ring/).first()).toBeVisible();
    await away(page, size.height);
    await shot(page, size, 'r06-rome-settle-lens-rings');
    const spot = await hexPoint(page, f.coloniaPos);
    await page.mouse.move(spot.x - 3, spot.y - 3);
    await page.mouse.move(spot.x, spot.y);
    await expect(page.getByText(/Founding here makes your Colonia Victrix/).first()).toBeVisible();
    await shot(page, size, 'r07-rome-site-hover-card');
    await away(page, size.height);
    const closeLens = page.getByRole('button', { name: 'Close map options' });
    if (await closeLens.isVisible()) await closeLens.click();
    const found = page.getByRole('button', { name: /Found Colonia Victrix/ });
    await expect(found).toBeVisible();
    await found.scrollIntoViewIfNeeded();
    await away(page, size.height);
    await shot(page, size, 'r08-rome-found-button');
  });

  test('Rome: a rival sees the Colonia on the map, on hover, in a forecast, in a card and in the race', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await open(page, 'romeRival');
    await frameHall(page, f.coloniaPos, 1.25);
    await shot(page, size, 'r09-rival-colonia-banner');
    await frameHall(page, f.coloniaPos, 0.55);
    await shot(page, size, 'r10-rival-colonia-far');
    await frameHall(page, f.coloniaPos, 1.25);
    const spot = await hexPoint(page, f.coloniaPos);
    await page.mouse.move(spot.x - 3, spot.y - 3);
    await page.mouse.move(spot.x, spot.y);
    await expect(page.locator('.tilecard')).toContainText('Colonia Victrix');
    await shot(page, size, 'r11-rival-colonia-hover');
    await selectThing(page, 'unit', f.attacker!);
    const forecast = page.locator('.forecast');
    await aim(page, f.coloniaPos, forecast);
    await expect(forecast).toContainText('Colonia Victrix');
    await shot(page, size, 'r12-rival-colonia-forecast');
    await away(page, size.height);
    await page.keyboard.press('Escape');
    await stageHallReport(page, [{ type: 'FactionGoalCompleted', player: f.goalSeat, faction: 'Rome', vp: 2 }]);
    await expect(page.locator('.moment-card').first()).toContainText('Colonia Victrix');
    await shot(page, size, 'r13-rival-colonia-public-card');
    await dismissCelebrations(page);
    await revealSidebar(page, 'standings');
    await page.locator('.panel.vp .vhc-tok[data-held]').first().hover();
    await expect(page.locator('.vhc-pop .vh-card.k-goal .vh-held')).toContainText('Colonia Victrix');
    await shot(page, size, 'r14-rival-colonia-race-row');
  });

  test('Egypt: the Build the Royal Tomb action and its confirm', async ({ page }) => {
    await open(page, 'egyptIdle');
    const row = page.locator('[data-goal-action="Tomb"]');
    await row.scrollIntoViewIfNeeded();
    await away(page, size.height);
    await shot(page, size, 'e01-egypt-tomb-action');
    await row.locator('.goal-action-btn').click();
    const dialog = page.locator('dialog[data-goal-dialog="Tomb"][open]');
    await expect(dialog).toBeVisible();
    await shot(page, size, 'e02-egypt-tomb-confirm');
  });

  test('Egypt mid-work: the count, the city panel, the cities overview and the pause', async ({ page }) => {
    test.setTimeout(150_000);
    const f = await open(page, 'egyptWork');
    await expect(page.getByTestId('tomb-readout')).toBeVisible();
    const row = page.locator('[data-goal-action="Tomb"]');
    await expect(row).toContainText('6 of 15');
    await row.scrollIntoViewIfNeeded();
    await away(page, size.height);
    await shot(page, size, 'e03-egypt-work-topbar-sidebar');
    await selectThing(page, 'city', f.b);
    await expect(page.locator('.city-panel')).toContainText('Royal Tomb');
    await away(page, size.height);
    await shot(page, size, 'e04-egypt-work-city-panel');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ selection: null, citiesOpen: true }));
    await expect(page.getByTestId('cities-tomb')).toBeVisible();
    await shot(page, size, 'e05-egypt-work-cities-overview');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ citiesOpen: false }));
    await row.locator('.goal-action-btn').click();
    await expect(page.locator('dialog[data-goal-dialog="TombPause"][open]')).toBeVisible();
    await shot(page, size, 'e06-egypt-pause-confirm');
  });

  test('Egypt seen by a rival: the work under way, then the standing tomb', async ({ page, browser }) => {
    test.setTimeout(150_000);
    const f = await open(page, 'egyptRival');
    await stageHallReport(page, [{ type: 'TombStarted', player: f.goalSeat, turns: 6 }]);
    await expect(page.locator('.moment-card').first()).toContainText('Royal Tomb');
    await shot(page, size, 'e07-rival-tomb-started-card');
    await dismissCelebrations(page);
    await revealSidebar(page, 'standings');
    // A short window folds the rows: open them all so Egypt's row and its count show.
    const more = page.locator('.panel.vp').getByRole('button', { name: /more empires?$/ });
    if (await more.isVisible()) await more.click();
    const chip = page.getByTestId('tomb-chip').first();
    await chip.scrollIntoViewIfNeeded();
    await expect(chip).toHaveText('6/15');
    await away(page, size.height);
    await shot(page, size, 'e08-rival-tomb-work-race');
    const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await context.newPage();
      const g = await open(p, 'tombRival');
      await frameHall(p, g.capitalPos, 1.25);
      await shot(p, size, 'e09-rival-tomb-banner');
      await frameHall(p, g.capitalPos, 0.55);
      await shot(p, size, 'e10-rival-tomb-far');
      await frameHall(p, g.capitalPos, 1.25);
      const spot = await hexPoint(p, g.capitalPos);
      await p.mouse.move(spot.x - 3, spot.y - 3);
      await p.mouse.move(spot.x, spot.y);
      await expect(p.locator('.tilecard')).toContainText('Royal Tomb');
      await shot(p, size, 'e11-rival-tomb-hover');
      await away(p, size.height);
      await revealSidebar(p, 'standings');
      await p.locator('.panel.vp .vhc-tok[data-held]').first().hover();
      await expect(p.locator('.vhc-pop .vh-card.k-goal .vh-held')).toContainText('Royal Tomb');
      await shot(p, size, 'e12-rival-tomb-race-row');
    } finally { await context.close(); }
  });

  test('Rome: your own Colonia, a third empire holding it, and a veteran with no site or off the rings', async ({ page, browser }) => {
    test.setTimeout(240_000);
    const f = await open(page, 'romeHeld');
    await frameHall(page, f.coloniaPos, 1.25);
    await shot(page, size, 'r15-own-colonia-banner');
    await stageHallReport(page, [{ type: 'FactionGoalCompleted', player: 0, faction: 'Rome', vp: 2 }]);
    await expect(page.locator('.moment-card').first()).toContainText('Colonia Victrix');
    await shot(page, size, 'r16-own-colonia-founded-card');
    await withMode(browser, size, 'coloniaThird', async (p, g) => {
      await frameHall(p, g.coloniaPos, 1.25);
      await shot(p, size, 'r17-third-empire-holds-colonia-banner');
      await revealSidebar(p, 'standings');
      await p.locator('.panel.vp .vhc-tok[data-held]').first().hover();
      await expect(p.locator('.vhc-pop .vh-card.k-goal .vh-held')).toContainText('Colonia Victrix');
      await shot(p, size, 'r18-third-empire-holds-colonia-race');
    });
    for (const [mode, name] of [['romeNoSite', 'r19-rome-no-site-lens'], ['romeOffRing', 'r20-rome-veteran-off-ring']] as const) {
      await withMode(browser, size, mode, async (p, g) => {
        await selectThing(p, 'unit', g.veteran!);
        const settle = p.getByRole('radio', { name: 'Settle', exact: true });
        if (!await settle.isVisible()) await p.getByRole('button', { name: /^Lens: Settle/ }).click();
        await expect(settle).toBeChecked();
        if (mode === 'romeNoSite') await expect(p.getByTestId('settle-colonia-none')).toContainText('No site you know founds your Colonia Victrix');
        else await expect(p.getByTestId('settle-colonia-key')).toBeVisible();
        await away(p, size.height);
        await shot(p, size, name);
      });
    }
  });

  test('Egypt: the planned tomb on the map, the row paused, standing and lost, and your own tomb', async ({ page, browser }) => {
    test.setTimeout(240_000);
    const f = await open(page, 'egyptWork');
    // Your capital: the dashed tomb seal, the capital star and the tomb count in place of production.
    await frameHall(page, f.capitalPos, 1.25);
    await shot(page, size, 'e13-own-tomb-site-banner');
    await withMode(browser, size, 'egyptRival', async (p, g) => {
      await frameHall(p, g.capitalPos, 1.25);
      await shot(p, size, 'e14-rival-tomb-site-banner');
      const spot = await hexPoint(p, g.capitalPos);
      await p.mouse.move(spot.x - 3, spot.y - 3);
      await p.mouse.move(spot.x, spot.y);
      await expect(p.getByTestId('tile-goal-tomb-site')).toContainText("Egypt's Royal Tomb will stand here in 9 turns: 2 VP to whoever holds the city.");
      await shot(p, size, 'e15-rival-tomb-site-hover');
    });
    await withMode(browser, size, 'egyptRivalPaused', async (p, g) => {
      await frameHall(p, g.capitalPos, 1.25);
      const spot = await hexPoint(p, g.capitalPos);
      await p.mouse.move(spot.x - 3, spot.y - 3);
      await p.mouse.move(spot.x, spot.y);
      await expect(p.getByTestId('tile-goal-tomb-site')).toContainText("Egypt's Royal Tomb will stand here: 2 VP to whoever holds the city. Paused at 6 of 15 turns.");
      await shot(p, size, 'e21-rival-paused-tomb-site-hover');
    });
    for (const [mode, name] of [['egyptPaused', 'e16-egypt-row-paused'], ['egyptHeld', 'e17-egypt-row-standing'], ['tombLost', 'e18-egypt-row-rival-holds']] as const) {
      await withMode(browser, size, mode, async (p) => {
        const row = p.locator('[data-goal-action="Tomb"]');
        await expect(row).toBeVisible();
        await row.scrollIntoViewIfNeeded();
        await away(p, size.height);
        await shot(p, size, name);
      });
    }
    await withMode(browser, size, 'egyptHeld', async (p, g) => {
      await frameHall(p, g.capitalPos, 1.25);
      await shot(p, size, 'e19-own-tomb-banner');
      await stageHallReport(p, [{ type: 'FactionGoalCompleted', player: 0, faction: 'Egypt', vp: 2 }]);
      await expect(p.locator('.moment-card').first()).toContainText('Royal Tomb');
      await shot(p, size, 'e20-own-tomb-stands-card');
    });
  });

  test('the faction sheets and the Guide entries', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    for (const [faction, name] of [['Rome', 'g01-sheet-rome'], ['Egypt', 'g02-sheet-egypt']] as const) {
      await page.getByRole('option', { name: faction }).click();
      await page.evaluate(() => document.fonts.ready.then(() => undefined));
      await away(page, size.height);
      await shot(page, size, name);
    }
    await page.keyboard.press('Escape');
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /The victory race/ }).click();
    for (const [lead, name] of [['Colonia Victrix (Rome):', 'g03-guide-colonia'], ['The Royal Tomb (Egypt):', 'g04-guide-tomb'], ['A gilt-ringed seal', 'g05-guide-seals']] as const) {
      const p = guide.locator('.guide-page p', { hasText: lead });
      await p.scrollIntoViewIfNeeded();
      await expect(p).toBeVisible();
      await away(page, size.height);
      await shot(page, size, name);
    }
  });
});

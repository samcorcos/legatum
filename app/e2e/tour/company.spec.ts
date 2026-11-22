// Screen tour, Mercenary Companies (Byzantium, the real engine): the hire section in the city panel (stocked, one row chosen, a thin
// treasury), Companies on the map, in the unit panel and on a tile card, a rival's roster on its diplomacy row, and the faction sheet.
// Run it alone with `npx playwright test e2e/tour/company`. Shots land in test-results/screen-tour/<size>/company/.
import type { Hex } from '../../src/types';
import { FACTION_SHEETS } from '../../src/factionSheets';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { companyState } from '../../src/test/companyState';
import { SIZES, aim, loadFixture, selectThing, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

type MapWindow = GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };

/** Scrolls the city panel so the shop's heading sits just under the sticky head. */
const showShop = (page: import('@playwright/test').Page) => page.evaluate(() => {
  const panel = document.querySelector<HTMLElement>('.city-panel')!;
  const shop = document.querySelector<HTMLElement>('[data-testid="company-shop"]')!;
  const head = document.querySelector<HTMLElement>('.city-panel .cp-head, .city-panel [data-sticky-head]')?.getBoundingClientRect().height ?? 130;
  panel.scrollTop += shop.getBoundingClientRect().top - panel.getBoundingClientRect().top - head - 30;
});

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the hire section: stocked, one row chosen, then a thin treasury', async ({ page, browser }) => {
    test.setTimeout(180_000);
    const fx = companyState(400);
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await selectThing(page, 'city', fx.city);
    const shop = page.getByTestId('company-shop');
    await expect(shop).toBeVisible();
    await shop.scrollIntoViewIfNeeded();
    await showShop(page);
    // Sparta, India and the rival Byzantium each sell their own special unit, one row apiece.
    expect(await shop.getByTestId('company-row').count()).toBe(3);
    // The real shop: every row is priced by the core, and a row you can afford has a live Hire button.
    const first = shop.getByTestId('company-row').first();
    await expect(first.getByRole('button', { name: /^Hire an? .* Company from .* for \d+ Gold$/ })).toBeEnabled();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-hire-section');
    for (const row of await shop.getByTestId('company-row').all()) expect(await row.evaluate((el) => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(0);
    await first.getByRole("button", { name: /^Hire an? / }).click();
    await expect(page.locator('.purchase-status')).toContainText('to spend');
    await expect(page.locator('[data-testid="company-row"].chosen')).toHaveCount(1);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '02-hire-chosen');
    // A thin treasury: the dear rows are greyed with the reason under each, and no price on a dead button.
    const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await context.newPage();
      const poor = companyState(60);
      await loadFixture(p, coreFixture(poor.state), poor.turn);
      await selectThing(p, 'city', poor.city);
      const poorShop = p.getByTestId('company-shop');
      await poorShop.scrollIntoViewIfNeeded();
      await showShop(p);
      await expect(poorShop.locator('.company-why').first()).toContainText('Need');
      await expect(poorShop.locator('.company-why').first()).toContainText('more');
      await p.mouse.move(5, size.height / 2);
      await shot(p, size, '03-hire-thin-treasury');
    } finally { await context.close(); }
    const lean = companyState(80);
    await loadFixture(page, coreFixture(lean.state), lean.turn);
    await selectThing(page, 'city', lean.city);
    const leanShop = page.getByTestId('company-shop');
    await leanShop.scrollIntoViewIfNeeded();
    await showShop(page);
    await expect(leanShop.locator('.company-impact.danger').first()).toContainText('At your current income, treasury cannot pay');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '04-hire-negative-income');
  });

  test('the map and the unit panel: a Company wears its seller and its countdown', async ({ page }) => {
    test.setTimeout(180_000);
    const fx = companyState(400);
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await selectThing(page, 'unit', fx.week);
    await page.evaluate(() => { const w = window as unknown as MapWindow; w.legatumMap.camera.zoom = 1.5; w.legatumMap.redraw(); });
    await expect(page.getByTestId('company-tag')).toContainText('Hired from Sparta: 7 turns of contract left, upkeep 2');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '05-unit-panel-week');
    await selectThing(page, 'unit', fx.last);
    await expect(page.getByTestId('company-tag')).toContainText('Hired from India: goes home this turn');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '06-unit-panel-last-turn');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '07-map-companies');
    await page.evaluate(() => { const w = window as unknown as MapWindow; w.legatumMap.camera.zoom = 0.5; w.legatumMap.redraw(); });
    await page.waitForTimeout(500);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '08-map-companies-far');
    // A rival's Company: the same public countdown on its tile card.
    await page.evaluate(() => { const w = window as unknown as MapWindow; w.legatumMap.camera.zoom = 1.5; w.legatumMap.redraw(); });
    await selectThing(page, 'city', fx.city);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    await page.evaluate((h) => (window as unknown as GameWindow).legatum.getState().jumpTo(h), fx.hex.rival as Hex);
    const company = page.getByTestId('tile-company');
    await aim(page, fx.hex.rival as Hex, company);
    await expect(company).toContainText('Hired from Sparta: 19 turns of contract left');
    await page.waitForTimeout(300);
    await shot(page, size, '09-tile-card-rival-company');
  });

  test('the roster: your Companies beside the levy, and a rival\'s on its diplomacy row', async ({ page }) => {
    test.setTimeout(180_000);
    const fx = companyState(400);
    await loadFixture(page, coreFixture(fx.state), fx.turn);
    await selectThing(page, 'city', fx.city);
    const roster = page.getByTestId('company-roster').first();
    await expect(roster).toContainText('Companies: 2 · upkeep 4 Gold a turn · next goes home this turn');
    await roster.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '10-roster-city-panel');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ diplomacyOpen: true, diplomacyTeam: 3 }));
    const rival = page.locator('.empire-row.open [data-testid="company-roster"]');
    await expect(rival).toContainText('Companies: 1 · upkeep 2 Gold a turn · next goes home in 19 turns');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '11-roster-rival-row');
  });

  test('the Byzantium faction sheet states the Company rule and how rivals counter it', async ({ page }) => {
    test.setTimeout(120_000);
    await page.addInitScript(() => { if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] })); });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    await page.getByRole('option', { name: 'Byzantium' }).click();
    const sheet = stage.getByTestId('faction-sheet');
    await expect(sheet.locator('.fs-sheet-name')).toContainText('Byzantium');
    await expect(sheet).toContainText('Hire a Company');
    await expect(sheet).toContainText(FACTION_SHEETS.Byzantium.counteredBy!);
    await expect(sheet).not.toContainText('Imperial Scriptorium');
    await expect(sheet).not.toContainText(/tribute|bought peace/i);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '12-faction-sheet');
    expect(await sheet.evaluate((el) => el.scrollHeight - el.clientHeight)).toBeLessThanOrEqual(0);
  });
});

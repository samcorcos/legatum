// The production picker under per-item banking (Civ 6): greyed-out items say why and add no
// order; banked progress shows on the cards. Runs the real engine (WASM) on an injected save.
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { wallsFixture } from '../src/test/wallsFixture';
import { neighbors, tileIndex } from '../src/hex';
import { TECHS } from '../src/data';

type GameWindow = Window & { legatum: typeof useGame };

async function openCity(page: Page, edit: (city: ReturnType<typeof wallsFixture>['fixture']['cities'][number]) => void, turn = 2, editState?: (fixture: ReturnType<typeof wallsFixture>['fixture']) => void) {
  const setup = wallsFixture();
  edit(setup.fixture.cities.find((c) => c.id === setup.own)!);
  editState?.(setup.fixture);
  setup.fixture.turn = turn;
  const game = coreFixture(setup.fixture);
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.setViewportSize({ width: 1000, height: 600 });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
  await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    s.set({ techOpen: false });
    s.select({ kind: 'city', id }, true);
  }, setup.own);
  return page.locator('.city-panel');
}

const productionOrders = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  return s.orders[s.seat].filter((o) => o.type === 'SetProduction' || o.type === 'SetProductionQueue' || o.type === 'BuyItem');
});

test('a population-1 city greys out its Settler, says why with the growth forecast, and clicking it adds no order', async ({ page }) => {
  const panel = await openCity(page, (c) => Object.assign(c, { population: 1, producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [] }));
  const card = panel.locator('.production-choice[data-item="u:Settler"]');
  const build = card.getByRole('button', { name: 'Build Settler', exact: true });
  await expect(card).toHaveAttribute('data-unavailable', 'true');
  await expect(build).toHaveAttribute('aria-disabled', 'true');
  await expect(card.locator('.prod-unavailable')).toHaveText(/^Needs 2 population · .+ grows to 2 in \d+ turns?$/);
  await expect(card.locator('.prod-effect')).toHaveText('Founds a city. Costs 1 population.');
  // Looks disabled before hover: dashed frame, no "Build →", not-allowed cursor, Buy disabled.
  const look = await build.evaluate((el) => ({ border: getComputedStyle(el).borderTopStyle, cursor: getComputedStyle(el).cursor }));
  expect(look).toEqual({ border: 'dashed', cursor: 'not-allowed' });
  await expect(card.locator('.prod-go')).toHaveText('Not yet');
  await expect(card.locator('.btn-buy')).toBeDisabled();
  // aria-disabled controls stay clickable for the mouse (to explain); Playwright needs force for them.
  await build.click({ force: true });
  await card.getByRole('button', { name: 'Queue Settler', exact: true }).click({ force: true });
  expect(await productionOrders(page)).toEqual([]);
  await expect(card.locator('.production-notice')).toHaveText(/^Can't build Settler yet: needs 2 population\. .+ grows to 2 in \d+ turns?\.$/);
  // The panel stays open on the refusal (Build of a valid item would advance to the next decision).
  await expect(panel).toBeVisible();
});

test('banked progress shows on its card and in the city, and switching says what stays banked', async ({ page }) => {
  const panel = await openCity(page, (c) => Object.assign(c, { population: 3, producing: { kind: 'Unit', unit: 'Soldier' }, prod: 12, productionQueue: [],
    productionBanked: [{ item: { kind: 'Unit', unit: 'Settler' }, progress: 38 }] }));
  await expect(panel.locator('.production-choice[data-item="u:Settler"] .prod-cost')).toHaveText(/^38\/\d+ banked$/);
  await expect(panel.locator('.production-choice[data-item="u:Soldier"] .prod-cost')).toHaveText(/^12\/\d+ so far$/);
  await expect(panel.getByTestId('city-banked')).toHaveText(/^Banked Settler 38\/\d+$/);
  await panel.getByRole('button', { name: 'Build Settler', exact: true }).click();
  expect(await productionOrders(page)).toEqual([expect.objectContaining({ type: 'SetProductionQueue', queue: [{ kind: 'Unit', unit: 'Settler' }] })]);
  await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    s.select({ kind: 'city', id: v.cities.find((c) => c.owner === v.me)!.id }, true);
  });
  await expect(panel.getByTestId('production-switch')).toHaveText(/^.+ keeps the 12 Production already put into the Spearman; Settler resumes at 38\/\d+\.$/);
});

test('Walls Buy is disabled with the Production-only reason while Build remains available', async ({ page }) => {
  const panel = await openCity(page, (c) => { c.buildings = []; }, 2, (f) => { f.players[0].techs.push('Masonry'); });
  const card = panel.locator('.production-choice[data-item="b:Walls"]');
  const buy = card.locator('.btn-buy');
  await expect(buy).toBeDisabled();
  await expect(card.locator('.production-refusal')).toHaveText("Walls must be built with Production; they can't be bought.");
  await expect(card.getByRole('button', { name: 'Build Walls' })).toBeEnabled();
  await buy.click({ force: true });
  expect(await productionOrders(page)).toEqual([]);
});

test('turn 1 at 1000x600: an idle capital opens on its build choice, and a build card shows even at scroll 0', async ({ page }) => {
  // The first city a player sees: population 1, nothing queued, no Walls (wallsFixture's capital has a Walls and Citadel).
  const panel = await openCity(page, (c) => Object.assign(c, { population: 1, buildings: [], producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [] }), 1);
  const geometry = () => panel.evaluate((el) => {
    const box = el.getBoundingClientRect(), head = el.querySelector('.cp-head')!.getBoundingClientRect();
    const title = el.querySelector('.cp-choose-title')!.getBoundingClientRect();
    const cards = [...el.querySelectorAll('.production-choice')].map((c) => c.getBoundingClientRect());
    return { scrollTop: el.scrollTop, panelBottom: box.bottom, headBottom: head.bottom, title: { top: title.top, bottom: title.bottom }, cards: cards.map((c) => ({ top: c.top, bottom: c.bottom })) };
  });
  // An idle city with nothing queued brings "Choose what X builds" up under the sticky head, with a card in view.
  const arrived = await geometry();
  expect(arrived.title.top).toBeGreaterThanOrEqual(arrived.headBottom - 1);
  expect(arrived.cards.some((c) => c.top >= arrived.headBottom && c.bottom <= arrived.panelBottom)).toBe(true);
  // The tile line is labelled (gross tile yields, not income) and the claim rule is a tooltip, not a paragraph.
  await expect(panel.getByTestId('city-tiles')).toHaveText('Tiles');
  await expect(panel.getByTestId('claim-rule')).toHaveCount(0);
  // Even with the panel scrolled to the top, the title and a whole build card fit under the territory bar.
  await panel.evaluate((el) => { el.scrollTop = 0; });
  const top = await geometry();
  expect(top.scrollTop).toBe(0);
  expect(top.cards.some((c) => c.bottom <= top.panelBottom && c.top >= top.headBottom)).toBe(true);
  await page.screenshot({ path: 'test-results/production-picker/turn1-city-scroll0.png' });
});

test('turn 1: the Improvements group comes after Buildings, and the Units cards sit at scroll 0', async ({ page }) => {
  const setup = wallsFixture();
  const city = setup.fixture.cities.find((c) => c.id === setup.own)!;
  Object.assign(city, { population: 1, producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [] });
  const f = setup.fixture;
  f.resource ??= f.terrain.map(() => null);
  const near = neighbors(city.pos)[0];
  f.resource[tileIndex(near, f.width, f.height)] = 'Wheat';
  f.turn = 1;
  await page.addInitScript(({ game }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 1, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(f) });
  await page.setViewportSize({ width: 1000, height: 600 });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await page.evaluate((id) => { const s = (window as unknown as GameWindow).legatum.getState(); s.set({ techOpen: false }); s.select({ kind: 'city', id }, true); }, setup.own);
  const panel = page.locator('.city-panel');
  const group = page.getByTestId('improvements-group');
  await expect(group).toBeAttached();
  // At scroll 0 the first build cards are in view; the group comes after the Buildings heading in the page.
  await panel.evaluate((el) => { el.scrollTop = 0; });
  await expect(panel.locator('.production-choice').first()).toBeInViewport();
  const order = await panel.evaluate((el) => {
    const pos = (n: Element | null) => n ? [...el.querySelectorAll('*')].indexOf(n) : -1;
    const heads = [...el.querySelectorAll('.cp-sub')];
    return { buildings: pos(heads.find((h) => h.textContent === 'Buildings') ?? null), group: pos(el.querySelector('[data-testid="improvements-group"]')) };
  });
  expect(order.group).toBeGreaterThan(order.buildings);
  await page.waitForTimeout(400);
  await page.screenshot({ path: 'test-results/production-picker-turn1.png', animations: 'disabled' });
  await group.scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await page.screenshot({ path: 'test-results/production-picker-turn1-group.png', animations: 'disabled' });
});

for (const faction of ['Switzerland', 'India'] as const) {
  test(`${faction}: the world wonders section states its Production-only rule and fits at 1000x600`, async ({ page }) => {
    const panel = await openCity(page,
      (c) => Object.assign(c, { population: 8, producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [], buildings: ['Autoworks'] }),
      2,
      (f) => { f.players[0].faction = faction; f.players[0].gold = 100_000; f.players[0].techs = [...TECHS]; });
    const works = panel.locator('.cp-great-works');
    await expect(works.getByRole('heading', { name: /World wonders/ })).toBeVisible();
    await expect(works.locator('.production-choice')).toHaveCount(7);
    await expect(works).not.toContainText('No Gold purchases');
    const site = works.locator('.production-choice.wonder-site').first();
    const cardFit = await site.evaluate((row) => {
      const card = row.querySelector('button.prod')!;
      const box = (el: Element) => el.getBoundingClientRect();
      const effect = row.querySelector('.prod-effect')!;
      const cost = box(card.querySelector('.prod-cost')!), turns = box(card.querySelector('.prod-turns')!), action = box(card.querySelector('.prod-go')!), summary = box(effect);
      // A wonder is a normal build card: the action on the first row, cost and turns on the second, then the summary line.
      return { summaryAfterAction: summary.top >= Math.max(action.bottom, cost.bottom, turns.bottom) - 1, costBeforeAction: cost.top >= action.bottom - 1,
        turnsBeforeAction: turns.top >= action.bottom - 1, fitsSideways: row.scrollWidth - row.clientWidth <= 1 && card.scrollWidth - card.clientWidth <= 1,
        summaryWhole: new Set([...effect.getClientRects()].map((r) => Math.round(r.top))).size <= 2 && getComputedStyle(effect).textOverflow !== 'ellipsis',
        positions: [cost.top, cost.bottom, turns.top, turns.bottom, action.top, action.bottom, summary.top, summary.bottom] };
    });
    expect(cardFit.summaryAfterAction && cardFit.costBeforeAction && cardFit.turnsBeforeAction && cardFit.fitsSideways && cardFit.summaryWhole,
      `World wonder card positions: ${cardFit.positions.join(', ')}`).toBe(true);
    await expect(works.locator('.prod-buy-why')).toHaveCount(0);
    await expect(works.locator('.btn-buy')).toHaveCount(0);
    await works.locator('.hint').hover();
    await expect(works.locator('.hint')).toContainText('Wonders are built with Production only');
    await page.mouse.move(5, 5);
    for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
      await page.setViewportSize({ width, height });
      await site.scrollIntoViewIfNeeded();
      await expect(site).toBeVisible();
      await page.screenshot({ path: `test-results/wonder-card-${faction.toLowerCase()}-${width}x${height}.png`, animations: 'disabled' });
    }
  });
}

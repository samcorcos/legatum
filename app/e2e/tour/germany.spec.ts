// Screen tour, the Germany rework (2026-11-05 and 2026-11-15): the Ruhr Foundry in the Forge slot (+6 Production), double
// strategic supply and no movement penalty. Every surface the rework touched: the build card, the Iron Working unlock
// card, the supply chip and its tooltip, a German Iron deposit's tile card, the unit panel's Moves tooltip, a city with
// a Ruhr Foundry (its Food breakdown) and the Factory card that upgrades it.
// Run alone with `npx playwright test e2e/tour/germany`; shots land in test-results/screen-tour/<size>/germany/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { ring } from '../../src/hex';
import { coreFixture } from '../../src/test/coreFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { spawnUnit, resolveFixture } from '../../src/test/fixtureEngine';
import { momentsForTurn } from '../../src/celebrate/moments';
import { contextFromViews, formatEvents } from '../../src/format';
import type { TechId } from '../../src/types';
import { SIZES, loadFixture, selectThing, shot, hexPoint, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

const park = (page: Page) => page.mouse.move(2, page.viewportSize()!.height - 2);

/** A German capital of population 5 with a Workshop, its own Iron deposit and two Pikemen. */
function germanCity(opts: { foundry?: boolean; factory?: boolean; techs?: TechId[] } = {}) {
  const s = improvementCity({ techs: ['IronWorking', 'Mathematics', 'Engineering', 'Feudalism', ...(opts.techs ?? [])] });
  s.players[0].faction = 'Germany';
  const c = s.cities[0];
  c.buildings.push(opts.factory ? 'Autoworks' : opts.foundry ? 'MachineShop' : 'Workshop');
  const spots = ring(c.pos, 1);
  const units = [spots[1], spots[2]].map((at) => spawnUnit(s, 0, 'Trooper', at, false));
  const after = resolveFixture(s);
  for (const p of s.players) p.supply = after.players[p.id].supply;
  const iron = ring(c.pos, 3)[2];
  return { s, city: c, units, iron };
}

for (const size of SIZES) test.describe(`germany ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('build card, supply chip, Iron tile card and unit panel', async ({ page }) => {
    test.setTimeout(150_000);
    const g = germanCity();
    await loadFixture(page, coreFixture(g.s), 12);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    // The supply chip and its tooltip.
    await park(page);
    await shot(page, size, 'g01-topbar-supply-chip');
    await page.locator('.supply-stat.supply-iron').click();
    await expect(page.locator('.supply-tip')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'g02-supply-chip-tooltip');
    await page.keyboard.press('Escape');
    // The city's build list with the Ruhr Foundry card.
    await selectThing(page, 'city', g.city.id);
    const card = page.locator('.city-panel .production-choice[data-item="b:MachineShop"]').first();
    await card.scrollIntoViewIfNeeded();
    await expect(card.locator('.prod-effect')).toContainText('−1');
    await park(page);
    await shot(page, size, 'g03-ruhr-foundry-build-card');
    const more = card.getByRole('button', { name: /More/ }).first();
    if (await more.count()) {
      await more.click();
      await card.scrollIntoViewIfNeeded();
      await park(page);
      await shot(page, size, 'g04-ruhr-foundry-build-card-more');
    }
    // The Iron deposit's tile card.
    await page.keyboard.press('Escape');
    const pt = await hexPoint(page, g.iron);
    await page.mouse.move(pt.x, pt.y);
    await page.waitForTimeout(700);
    await shot(page, size, 'g05-iron-tile-card');
    // A German Pikeman's panel and its Moves tooltip.
    await selectThing(page, 'unit', g.units[0].id);
    await park(page);
    await shot(page, size, 'g06-unit-panel');
    await page.locator('.unit-panel .up-stat').filter({ hasText: /Moves/i }).first().hover();
    await page.waitForTimeout(600);
    await shot(page, size, 'g07-unit-moves-tooltip');
  });

  test('a city with a Ruhr Foundry: Food breakdown and the Factory card', async ({ page }) => {
    test.setTimeout(150_000);
    const g = germanCity({ foundry: true, techs: ['Industrialization', 'Metallurgy'] });
    g.city.population = 7;
    await loadFixture(page, coreFixture(g.s), 12);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    await selectThing(page, 'city', g.city.id);
    await park(page);
    await shot(page, size, 'g08-city-with-foundry');
    await page.locator('[data-testid="city-yields"] .cp-yield').first().hover();
    await expect(page.locator('.tip-bubble:not([hidden])')).toContainText('Industry: -2 Food (Workshop, Ruhr Foundry)');
    await page.waitForTimeout(600);
    await shot(page, size, 'g09-city-food-hover');
    await page.locator('[data-testid="city-yields"] .cp-yield').nth(1).hover();
    await page.waitForTimeout(600);
    await shot(page, size, 'g10-city-production-hover');
    const factory = page.locator('.city-panel .production-choice[data-item="b:Autoworks"]').first();
    if (await factory.count()) {
      await expect(factory.locator('.prod-effect')).toContainText('−1');
      await factory.scrollIntoViewIfNeeded();
      await park(page);
      await shot(page, size, 'g11-factory-card');
    } else test.info().annotations.push({ type: 'missing', description: 'Factory card' });
    await page.locator('.supply-stat.supply-iron').click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'g12-supply-tooltip-with-foundry');
  });

  test('a Factory city retains the Foundry cost and goal', async ({ page }) => {
    test.setTimeout(150_000);
    const g = germanCity({ factory: true, techs: ['Industrialization', 'Metallurgy'] });
    g.city.population = 7;
    await loadFixture(page, coreFixture(g.s), 12);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    await selectThing(page, 'city', g.city.id);
    await expect(page.locator('.city-panel .cp-foot')).toContainText('Factory');
    await page.locator('[data-testid="city-yields"] .cp-yield').first().hover();
    await expect(page.locator('.tip-bubble:not([hidden])')).toContainText('Industry: -3 Food (Workshop, Ruhr Foundry, Factory)');
    await page.locator('.city-panel .cp-foot').scrollIntoViewIfNeeded();
    await park(page);
    await shot(page, size, 'g13-city-with-foundry-and-factory');
    // The goal counts the Forge-slot building wherever it stands: one Foundry city, with its Factory, is 1 of 5.
    await expect(page.getByText('Five Ruhr Foundries').first().locator('xpath=..')).toContainText('1/5');
    await page.locator('.supply-stat.supply-iron').click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'g14-supply-tooltip-foundry-and-factory');
  });

  test('tech unlock cards: Iron Working, Industrialization and Replaceable Parts', async ({ page }) => {
    test.setTimeout(150_000);
    await loadFixture(page, coreFixture(researchFixture('Germany')), 2);
    for (const tech of ['IronWorking', 'Industrialization', 'ReplaceableParts', 'Machinery'] as TechId[]) {
      await page.evaluate(() => {
        const s = (window as unknown as GameWindow).legatum.getState();
        s.set({ views: s.views!.map((v, i) => i === s.seat ? { ...v, research: { ...v.research, current: 'Currency', turns_left: 12 } } : v) });
      });
      const v = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; });
      const events = [{ type: 'TechResearched' as const, player: v.me, tech }];
      const report = { turn: v.turn - 1, hash: '', events: [events], moments: [momentsForTurn(events, v.me, null, v)], alarms: [[]], lines: [formatEvents(events, contextFromViews(v, v))] };
      await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [] }), report);
      await expect(page.locator('.moment-card.unlock')).toBeVisible();
      await park(page);
      await shot(page, size, `g20-unlock-${tech.toLowerCase()}`);
    }
  });

  test('the Guide and the peoples chapter for Germany', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await expect(guide).toBeVisible();
    for (const [button, needle, name] of [
      [/Cities & research/, 'Ruhr Foundry replaces the Forge', 'g30-guide-cities'],
      [/Cities & research/, 'German deposits supply twice as much', 'g31-guide-supply'],
      [/The victory race/, 'Five Ruhr Foundries (Germany)', 'g32-guide-goal'],
      [/The peoples/, 'Germany', 'g33-guide-peoples'],
      [/The peoples/, 'Ruhr Foundry replaces Forge', 'g34-guide-peoples-germany-card'],
    ] as const) {
      await guide.getByRole('button', { name: button }).click();
      const target = guide.locator('.guide-page').getByText(needle).first();
      if (await target.count()) await target.scrollIntoViewIfNeeded();
      else test.info().annotations.push({ type: 'missing', description: needle });
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, name);
    }
  });
});

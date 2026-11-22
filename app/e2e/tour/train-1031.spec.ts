// Screen tour, the train-1031 surfaces (UI review 2026-10-31): city automation (panel row, I hotkey, map tags, Cities
// overview selection), nested upgrade rows, the German Ruhr Foundry and aircraft Oil cards, Artillery's indirect fire,
// the Mongol Siege reason, the relations tab and a viewer eliminated mid-game.
// Run alone with `npx playwright test e2e/tour/train-1031`; shots land in test-results/screen-tour/<size>/train-1031/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { revealSidebar } from '../sidebarAssertions';
import { endPlanning } from '../turnActions';
import { TECHS } from '../../src/data';
import { fromOffset } from '../../src/hex';
import { coreFixture } from '../../src/test/coreFixture';
import { citiesFixture } from '../../src/test/citiesFixture';
import { spawnUnit, refreshVision } from '../../src/test/fixtureEngine';
import { allianceCoreSave } from '../../src/test/allianceFixture';
import { eliminatedState, ELIMINATED_TURN } from '../../src/test/eliminatedFixture';
import { SIZES, loadFixture, selectThing, shot, frameHall, dismissCelebrations, closeReportIfOpen, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

const park = (page: Page) => page.mouse.move(2, page.viewportSize()!.height - 2);

for (const size of SIZES) test.describe(`train-1031 ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('city automation: panel row, I hotkey, Cities overview selection, map tags, the pick after a turn', async ({ page }) => {
    test.setTimeout(180_000);
    const f = citiesFixture();
    const mine = f.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id);
    await loadFixture(page, coreFixture(f), 2);
    await selectThing(page, 'city', mine[3].id); // Geneva, idle
    await park(page);
    await expect(page.getByRole('combobox', { name: /Build automatically in/ })).toBeVisible();
    await shot(page, size, '01-city-panel-automation-off');
    await page.getByRole('combobox', { name: /Build automatically in/ }).selectOption('Science');
    await park(page);
    await shot(page, size, '02-city-panel-automation-science');
    await selectThing(page, 'city', mine[0].id); // Bern, building a Market
    await page.locator('[data-game-map]').focus();
    await page.keyboard.press('i');
    await park(page);
    await shot(page, size, '03-city-panel-after-I-hotkey');
    await page.keyboard.press('Escape');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ citiesOpen: true }));
    const sheet = page.getByTestId('cities-overview');
    await expect(sheet).toBeVisible();
    await park(page);
    await shot(page, size, '04-cities-overview-automation');
    await sheet.getByRole('checkbox', { name: /^Select (Basel|Lugano)$/ }).first().check();
    await sheet.getByRole('checkbox', { name: 'Select Lugano' }).check();
    await sheet.getByRole('combobox', { name: 'Building focus for selected cities' }).selectOption('Gold');
    await park(page);
    await shot(page, size, '05-cities-overview-two-selected');
    await sheet.getByRole('button', { name: 'Automate', exact: true }).click();
    await park(page);
    await shot(page, size, '06-cities-overview-automated');
    const overflow = await sheet.evaluate((el) => { const s = el.querySelector('.co-scroll')!; return s.scrollWidth - s.clientWidth; });
    expect.soft(overflow, 'cities table has no sideways scroll').toBeLessThanOrEqual(1);
    await page.keyboard.press('Escape');
    await frameHall(page, mine[0].pos, 1);
    await park(page);
    await shot(page, size, '07-map-automation-tags-near');
    await frameHall(page, mine[0].pos, 0.45);
    await park(page);
    await shot(page, size, '08-map-automation-tags-far');
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3', { timeout: 60_000 });
    await dismissCelebrations(page);
    await closeReportIfOpen(page);
    await selectThing(page, 'city', mine[3].id);
    await park(page);
    await shot(page, size, '09-city-panel-auto-pick-after-turn');
  });

  test('build list: nested upgrades early, then a German city with every technology (Ruhr Foundry, aircraft Oil, Artillery)', async ({ page, browser }) => {
    test.setTimeout(180_000);
    const early = citiesFixture();
    const geneva = early.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id)[3];
    early.players[0].techs.push('IronWorking', 'Mathematics', 'Engineering', 'Feudalism', 'Machinery');
    await loadFixture(page, coreFixture(early), 2);
    await selectThing(page, 'city', geneva.id);
    const nest = page.locator('.city-panel .upgrade-nest').first();
    await nest.scrollIntoViewIfNeeded();
    const upgradeArt = nest.locator('.upgrade-detail .item-art').first();
    await expect(upgradeArt).toBeVisible();
    expect(await upgradeArt.evaluate((image) => image.getBoundingClientRect().width)).toBeGreaterThanOrEqual(24);
    await park(page);
    await shot(page, size, '10-nested-upgrades');
    await nest.locator('.upgrade-detail').first().hover();
    await page.waitForTimeout(500);
    await shot(page, size, '11-nested-upgrade-hover');

    const ctx = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await ctx.newPage();
      const g = citiesFixture('Germany');
      g.players[0].techs = [...TECHS];
      const bern = g.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id)[0];
      bern.population = 8; bern.buildings = ['Laboratory', 'Workshop'];
      const art = spawnUnit(g, 0, 'Artillery', fromOffset(5, 5), false);
      refreshVision(g);
      await loadFixture(p, coreFixture(g), 2);
      await selectThing(p, 'city', bern.id);
      for (const [key, name] of [['b:MachineShop', '12-germany-ruhr-foundry'], ['u:Fighter', '13-aircraft-oil'], ['u:Artillery', '14-artillery-card']] as const) {
        const card = p.locator(`.city-panel .production-choice[data-item="${key}"]`).first();
        if (!await card.count()) { test.info().annotations.push({ type: 'missing', description: key }); continue; }
        await card.scrollIntoViewIfNeeded();
        await park(p);
        await shot(p, size, name);
      }
      await p.keyboard.press('Escape');
      await selectThing(p, 'unit', art.id);
      await park(p);
      await shot(p, size, '15-artillery-unit-panel');
    } finally { await ctx.close(); }
  });

  test('Mongol Siege before Medieval, the relations tab, and a viewer eliminated mid-game', async ({ page, browser }) => {
    test.setTimeout(180_000);
    const m = citiesFixture('Mongols');
    m.players[0].techs.push('Mathematics');
    const soldier = spawnUnit(m, 0, 'Soldier', fromOffset(5, 5), false);
    refreshVision(m);
    await loadFixture(page, coreFixture(m), 2);
    await selectThing(page, 'unit', soldier.id);
    await park(page);
    await shot(page, size, '16-mongol-siege-reason');

    const ctx = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await ctx.newPage();
      await loadFixture(p, allianceCoreSave(), 50);
      await revealSidebar(p, 'standings');
      await p.getByRole('button', { name: /^Diplomacy inbox/ }).click();
      const panel = p.getByRole('complementary', { name: 'Diplomacy inbox' });
      await park(p);
      await shot(p, size, '17-diplomacy-inbox-tab');
      await panel.getByRole('tab', { name: 'Relations' }).click();
      await park(p);
      await shot(p, size, '18-diplomacy-relations-tab');
      await panel.locator('.relations-cell').first().hover();
      await p.waitForTimeout(400);
      await shot(p, size, '19-diplomacy-relations-hover');
    } finally { await ctx.close(); }

    const ctx2 = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await ctx2.newPage();
      const s = eliminatedState();
      const gone = new Set(s.cities.filter((c) => c.owner === 0).map((c) => c.id));
      s.cities = s.cities.filter((c) => c.owner !== 0);
      for (let i = 0; i < s.tileCity.length; i++) if (gone.has(s.tileCity[i])) { s.tileCity[i] = -1; s.owner[i] = -1; }
      s.units = s.units.filter((u) => u.owner !== 0);
      s.players[0].alive = false;
      await loadFixture(p, coreFixture(s), ELIMINATED_TURN);
      await p.waitForTimeout(1500);
      await park(p);
      await shot(p, size, '20-viewer-eliminated-midgame');
      const reveal = p.getByTestId('game-reveal');
      if (await reveal.isVisible()) {
        await reveal.click({ position: { x: 6, y: 6 } });
        await park(p);
        await shot(p, size, '21-viewer-eliminated-settled');
      }
    } finally { await ctx2.close(); }
  });
});

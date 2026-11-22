// Screen tour, city automation focuses (PR #98, 2026-11-22): the city panel's Automate menu (Off, Economy, Military,
// Gold project, Science project, Food project; Egypt: Grow), its reason line and the idle "Chooses at Ready" heading,
// the Projects list with the Food project, the Cities sheet's focus menu, and the focus icons on map banners.
// Run alone with `npx playwright test e2e/tour/city-automation`; shots land in test-results/screen-tour/<size>/city-automation/.
import { writeFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { citiesFixture } from '../../src/test/citiesFixture';
import { SIZES, loadFixture, selectThing, shot, frameHall, dismissCelebrations, closeReportIfOpen, type Size } from './helpers';

test.describe.configure({ mode: 'parallel' });

const park = (page: Page) => page.mouse.move(2, page.viewportSize()!.height - 2);
const FOCUSES = ['', 'Economy', 'Military', 'Gold', 'Science', 'Food'] as const;
const auto = (page: Page) => page.getByRole('combobox', { name: /^Automate / });

/** The select's native title and the reason line can't show in a screenshot: record them beside the shots. */
async function note(page: Page, size: Size, name: string) {
  const data = await page.evaluate(() => ({
    title: document.querySelector('.cp-automation select')?.getAttribute('title'),
    options: [...document.querySelectorAll('.cp-automation select option')].map((o) => o.textContent),
    heading: document.querySelector('.cp-choose-title')?.textContent,
    reason: document.querySelector('.cp-automation-reason')?.textContent,
  }));
  const dir = fileURLToPath(new URL(`../../test-results/screen-tour/${size.name}/city-automation/`, import.meta.url));
  await mkdir(dir, { recursive: true });
  await writeFile(`${dir}${name}.text.json`, JSON.stringify(data, null, 2));
}

for (const size of SIZES) test.describe(`city-automation ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('panel: every focus on an idle city, a busy city, the Projects list', async ({ page }) => {
    test.setTimeout(180_000);
    const f = citiesFixture();
    const mine = f.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id);
    await loadFixture(page, coreFixture(f), 2);
    await selectThing(page, 'city', mine[3].id); // Geneva, idle
    for (const focus of FOCUSES) {
      await auto(page).selectOption(focus);
      await park(page);
      await shot(page, size, `a-idle-${focus || 'Off'}`);
      await note(page, size, `a-idle-${focus || 'Off'}`);
    }
    const projects = page.locator('.city-panel .cp-sub').filter({ hasText: /^Projects$/ });
    await projects.scrollIntoViewIfNeeded();
    await park(page);
    await shot(page, size, 'b-projects-list');
    const food = page.locator('.city-panel .production-choice[data-item*="Food"] button.prod').first();
    if (await food.count()) { await food.hover(); await page.waitForTimeout(700); await shot(page, size, 'b-projects-food-tip'); }
    await selectThing(page, 'city', mine[0].id); // Bern, building a Market
    await auto(page).selectOption('Gold');
    await park(page);
    await shot(page, size, 'c-busy-gold');
    await note(page, size, 'c-busy-gold');
    await auto(page).selectOption('Military');
    await park(page);
    await shot(page, size, 'c-busy-military');
    await note(page, size, 'c-busy-military');
  });

  test('Egypt: Grow in the menu, Projects list without a Food project', async ({ page }) => {
    test.setTimeout(150_000);
    const f = citiesFixture('Egypt');
    const mine = f.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id);
    await loadFixture(page, coreFixture(f), 2);
    await selectThing(page, 'city', mine[3].id);
    await auto(page).selectOption('Food');
    await park(page);
    await shot(page, size, 'd-egypt-grow');
    await note(page, size, 'd-egypt-grow');
    const projects = page.locator('.city-panel .cp-sub').filter({ hasText: /^(Projects|Growth)$/ }).first();
    await projects.scrollIntoViewIfNeeded();
    await park(page);
    await shot(page, size, 'd-egypt-projects');
  });

  test('Cities sheet, map banners near and far, the pick after a turn', async ({ page }) => {
    test.setTimeout(200_000);
    const f = citiesFixture();
    const mine = f.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id);
    for (const c of mine) c.producing = null; // every city idle, so each banner shows its focus
    await loadFixture(page, coreFixture(f), 2);
    const plan: [number, string][] = [[0, 'Economy'], [1, 'Military'], [2, 'Gold'], [3, 'Science']];
    for (const [i, focus] of plan) { await selectThing(page, 'city', mine[i].id); await auto(page).selectOption(focus); }
    await page.keyboard.press('Escape');
    await page.evaluate(() => (window as unknown as { legatum: { getState(): { set(p: object): void } } }).legatum.getState().set({ citiesOpen: true }));
    const sheet = page.getByTestId('cities-overview');
    await expect(sheet).toBeVisible();
    await sheet.getByRole('checkbox', { name: 'Select Lugano' }).check();
    await sheet.getByRole('combobox', { name: 'Building focus for selected cities' }).selectOption('Food');
    await park(page);
    await shot(page, size, 'e-cities-sheet-food-selected');
    const opts = await sheet.locator('select option').allTextContents();
    const dir = fileURLToPath(new URL(`../../test-results/screen-tour/${size.name}/city-automation/`, import.meta.url));
    await writeFile(`${dir}e-cities-sheet.options.json`, JSON.stringify(opts));
    await sheet.getByRole('button', { name: 'Automate', exact: true }).click();
    await park(page);
    await shot(page, size, 'e-cities-sheet-automated');
    await page.keyboard.press('Escape');
    await frameHall(page, mine[2].pos, 1);
    await park(page);
    await shot(page, size, 'f-map-focus-icons-near');
    await frameHall(page, mine[2].pos, 0.6);
    await park(page);
    await shot(page, size, 'f-map-focus-icons-mid');
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3', { timeout: 60_000 });
    await dismissCelebrations(page);
    await closeReportIfOpen(page);
    for (const i of [0, 1, 4]) {
      await selectThing(page, 'city', mine[i].id);
      await park(page);
      await shot(page, size, `g-after-turn-${mine[i].name}`);
      await note(page, size, `g-after-turn-${mine[i].name}`);
    }
    await frameHall(page, mine[2].pos, 1);
    await park(page);
    await shot(page, size, 'g-map-after-turn');
  });
});

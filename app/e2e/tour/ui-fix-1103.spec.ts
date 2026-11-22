// Fixes from the train-1103 UI review (ui-fix-1103): the build list at 1000x600 (long names fit their column), the objective deck
// cards in plain words, and a Germany Age card that names the ordinary Factory. The other fixed screens (Alliance offer at Peace,
// world wonders drawer, raze confirm, wonder moments, tile card, ledger header, Sparta clock, old-save notice, Keshig sheet) are in
// their own domains and in ui-review-1103. Run alone: `npx playwright test e2e/tour/ui-fix-1103`.
import { test, expect } from '../fixtures';
import { TECHS } from '../../src/data';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { withV2 } from '../../src/test/vpV2Fixture';
import { SIZES, loadFixture, selectThing, shot, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`ui fix 1103 ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the building list: Roadworks and Library fit their name column', async ({ page }) => {
    const s = improvementCity({ techs: [...TECHS] });
    s.players[0].faction = 'Byzantium';
    s.cities[0].buildings.push('Workshop', 'Market');
    await loadFixture(page, coreFixture(s), 12);
    await selectThing(page, 'city', s.cities[0].id);
    const names = page.locator('.city-panel .production-choice .prod-name');
    const cut = await names.evaluateAll((els) => els.map((el) => ({ text: el.textContent?.trim(), over: el.scrollWidth - el.clientWidth })).filter((x) => x.over > 0));
    expect(cut, 'no card name spills out of its column').toEqual([]);
    const road = page.locator('.city-panel .production-choice', { hasText: 'Roadworks' }).first();
    await road.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'b1-build-list-roadworks');
    await page.locator('.city-panel .production-choice', { hasText: 'Library' }).first().scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'b2-build-list-library');
  });

  /** The live view of the fixture game, staged with the v2 era numbers (render only), then changed by `edit`. */
  async function stage(page: import('@playwright/test').Page, edit: (v: ReturnType<typeof withV2>) => void) {
    const base = await page.evaluate(() => { const st = (window as unknown as GameWindow).legatum.getState(); return structuredClone(st.views![st.seat]); });
    const view = withV2(base, { era: 2, scored: [] });
    edit(view);
    await page.evaluate((v) => { const store = (window as unknown as GameWindow).legatum, st = store.getState(); store.setState({ views: st.views!.map((x, i) => (i === st.seat ? v : x)) }); }, view);
  }

  test('World Age cards say what they cost in plain words', async ({ page }) => {
    const s = improvementCity({ techs: ['Masonry'] });
    await loadFixture(page, coreFixture(s), 12);
    // One card per Age, as a real game draws: Breach for Age II, Veterans for Age III.
    await stage(page, (v) => {
      v.eras = v.eras!.filter((e) => e.era === 2 || e.era === 3);
      v.eras[0].objectives = [{ slot: 0, family: 'Fight', text: 'Breach: capture a walled city', target: 1, objective: { kind: 'Breach' } }];
      v.eras[1].objectives = [{ slot: 0, family: 'Fight', text: 'Veterans: earn 4 promotions in battle (a promotion is a level a unit gains from combat experience after this Age began)', target: 4, objective: { kind: 'Veterans', n: 4 } }];
      for (const e of v.eras) e.scored_by = [];
      v.objective_progress = [
        { era: 2, slot: 0, current: 0, target: 1, entered: true, era_scored: false, ready: false },
        { era: 3, slot: 0, current: 2, target: 4, entered: true, era_scored: false, ready: false },
      ];
    });
    await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'Score ledger', exact: true }).click();
    const sheet = page.getByTestId('objective-sheet');
    const open = async (era: number) => {
      const row = sheet.locator(`.sl-row[data-row="era-${era}"] button`);
      if (await row.getAttribute('aria-expanded') === 'false') await row.click();
      return sheet.locator(`#sl-detail-era-${era}`);
    };
    const veterans = await open(3);
    await expect(veterans).toContainText('Veterans: earn 4 promotions in battle (a promotion is a level a unit gains from combat experience after this Age began)');
    // The capital and wonder exception is printed only under the two capture cards.
    await expect(veterans).not.toContainText('never count capitals or wonder cities');
    const detail = await open(2);
    // The status line states the rule once; the fine print no longer repeats it in the plural.
    await expect(detail).toContainText('Every empire sees this card the turn the first empire enters the era, and it never expires.');
    await expect(detail).not.toContainText('and they never expire');
    await expect(detail).toContainText('Each card counts only what you do after its Age begins');
    await expect(detail).toContainText('Humble the mighty and Breach never count capitals or wonder cities.');
    await expect(detail).toContainText('Walled cities captured since the Age began: 0 of 1');
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd1-objective-deck-cards');
  });

  test('a Germany Age card names the ordinary Factory', async ({ page }) => {
    const s = improvementCity({ techs: ['Masonry'] });
    s.players[0].faction = 'Germany';
    await loadFixture(page, coreFixture(s), 12);
    await stage(page, (v) => {
      v.eras = v.eras!.filter((e) => e.era === 2);
      v.eras[0].objectives[0] = { slot: 0, family: 'Build', text: 'Add a Factory to 3 more cities', target: 3, objective: { kind: 'Buildings', building: 'Factory', cities: 3 } };
      v.objective_progress = [{ era: 2, slot: 0, current: 1, target: 3, entered: true, era_scored: false }];
    });
    await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'Score ledger', exact: true }).click();
    const sheet = page.getByTestId('objective-sheet');
    const era = sheet.locator('.sl-row[data-row="era-2"] button');
    if (await era.getAttribute('aria-expanded') === 'false') await era.click();
    await expect(sheet.locator('#sl-detail-era-2')).toContainText('Add a Factory to 3 more cities');
    await expect(sheet.getByText('Five Ruhr Foundries').first()).toBeVisible();
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'g1-germany-objective-and-goal-ledger');
  });
});

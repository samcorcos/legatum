// Screen tour, Egypt's Grow (the Nile, 2026-11-22): an Egyptian city's panel idle, building something and running
// Grow, with a German city as the control (its Industry projects section is unchanged).
// Run alone with `npx playwright test e2e/tour/egypt-grow`; shots land in test-results/screen-tour/<size>/egypt-grow/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import type { ProductionItem } from '../../src/types';
import { SIZES, loadFixture, selectThing, shot } from './helpers';

test.describe.configure({ mode: 'parallel' });

const park = (page: Page) => page.mouse.move(2, page.viewportSize()!.height - 2);

function city(faction: 'Egypt' | 'Germany', producing: ProductionItem | null) {
  const s = improvementCity({ improved: true });
  s.players[0].faction = faction;
  if (faction === 'Germany') s.cities[0].buildings.push('Workshop');
  s.cities[0].producing = producing;
  return { s, city: s.cities[0] };
}

const STATES: { name: string; faction: 'Egypt' | 'Germany'; producing: ProductionItem | null }[] = [
  { name: 'e01-egypt-idle', faction: 'Egypt', producing: null },
  { name: 'e02-egypt-building', faction: 'Egypt', producing: { kind: 'Building', building: 'Granary' } as ProductionItem },
  { name: 'e03-egypt-grow', faction: 'Egypt', producing: { kind: 'Project', project: 'Grow' } },
  { name: 'e04-germany-control', faction: 'Germany', producing: { kind: 'Building', building: 'Granary' } as ProductionItem },
];

for (const size of SIZES) test.describe(`egypt-grow ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  for (const st of STATES) test(st.name, async ({ page }) => {
    test.setTimeout(120_000);
    const g = city(st.faction, st.producing);
    await loadFixture(page, coreFixture(g.s), 12);
    await selectThing(page, 'city', g.city.id);
    await expect(page.locator('.city-panel')).toBeVisible();
    await park(page);
    await shot(page, size, `${st.name}-top`);
    const sub = page.locator('.city-panel .cp-sub').filter({ hasText: st.faction === 'Egypt' ? /^Growth$/ : /Industry projects/ }).first();
    if (await sub.count()) {
      await sub.scrollIntoViewIfNeeded();
      await park(page);
      await shot(page, size, `${st.name}-projects`);
      const btn = page.locator('.city-panel .project-choice button.prod').first();
      await btn.hover();
      await page.waitForTimeout(700);
      await shot(page, size, `${st.name}-project-tip`);
    }
    const note = page.getByTestId('egypt-construction');
    if (await note.count()) { await note.scrollIntoViewIfNeeded(); await park(page); await shot(page, size, `${st.name}-nile-note`); }
    // The growth tooltip (Food breakdown).
    await page.mouse.move(size.width - 5, size.height / 2); await page.waitForTimeout(300);
    const growth = page.getByTestId('city-growth').first();
    if (await growth.count()) { await growth.scrollIntoViewIfNeeded(); await growth.hover(); await page.waitForTimeout(700); await shot(page, size, `${st.name}-growth-tip`);
      await growth.click(); await page.waitForTimeout(500); await shot(page, size, `${st.name}-growth-breakdown`); }
  });
});

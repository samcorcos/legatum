// Screen tour, Egypt's Grow (the Nile, 2026-11-22): an Egyptian city's panel idle, building something and running
// Grow, with a German city as the control (its Industry projects section is unchanged).
// Run alone with `npx playwright test e2e/tour/egypt-grow`; shots land in test-results/screen-tour/<size>/egypt-grow/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import type { ProductionItem } from '../../src/types';
import { SIZES, aim, loadFixture, selectThing, shot, type GameWindow } from './helpers';
import { setOverlay } from '../overlayHelpers';
import { ring, within } from '../../src/hex';
import { idxOf } from '../../src/test/fixtureEngine';

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

// The Nile on the map (2026-11-22): Egypt sees double Food and no Production on the Yields overlay and the tile card,
// Stone says a Quarry gives it nothing, and its build list offers no Quarry. India is the control.
const MAP_CASES: { name: string; faction: 'Egypt' | 'India' }[] = [
  { name: 'e10-egypt-map', faction: 'Egypt' },
  { name: 'e11-india-map-control', faction: 'India' },
];
for (const size of SIZES) test.describe(`egypt-grow map ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  for (const mc of MAP_CASES) test(mc.name, async ({ page }) => {
    test.setTimeout(150_000);
    const s = improvementCity({ improved: false });
    s.players[0].faction = mc.faction;
    const c = s.cities[0];
    const wheat = ring(c.pos, 1)[0], stone = ring(c.pos, 2)[7], forest = within(c.pos, 3).find((h) => s.terrain[idxOf(s, h)] === 'Forest')!;
    await loadFixture(page, coreFixture(s), 12);
    const look = async (h: { q: number; r: number }, label: string) => {
      await page.evaluate((h) => { const st = (window as unknown as GameWindow).legatum.getState(); st.set({ techOpen: false }); st.select(null); st.jumpTo(h); }, h);
      await aim(page, h, page.locator('.tilecard'));
      await page.waitForTimeout(400);
      await shot(page, size, `${mc.name}-${label}`);
    };
    await look(wheat, 'tile-wheat');
    await look(stone, 'tile-stone');
    await look(forest, 'tile-forest');
    await setOverlay(page, 'Yields', true);
    await page.evaluate((h) => { const st = (window as unknown as GameWindow).legatum.getState(); st.select(null); st.jumpTo(h); }, c.pos);
    await park(page); await page.waitForTimeout(600);
    await shot(page, size, `${mc.name}-yields-overlay`);
    await setOverlay(page, 'Yields', false);
    await selectThing(page, 'city', c.id);
    await expect(page.locator('.city-panel')).toBeVisible();
    const imp = page.locator('.city-panel .production-choice[data-item^="i:"]');
    if (await imp.count()) {
      await imp.first().scrollIntoViewIfNeeded(); await park(page);
      await shot(page, size, `${mc.name}-improvements`);
      await imp.first().hover(); await page.waitForTimeout(700);
      await shot(page, size, `${mc.name}-improvement-tip`);
    }
    const quarries = await page.locator('.city-panel .production-choice[data-item^="i:Quarry"]').count();
    if (mc.faction === 'Egypt') expect(quarries).toBe(0); else expect(quarries).toBeGreaterThan(0);
  });
});

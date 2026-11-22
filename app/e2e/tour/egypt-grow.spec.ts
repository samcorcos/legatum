// Screen tour, Egypt's Grow (the Nile, 2026-11-22): an Egyptian city's panel idle, building something and running
// Grow, with a German city as the control (its Industry projects section is unchanged).
// Run alone with `npx playwright test e2e/tour/egypt-grow`; shots land in test-results/screen-tour/<size>/egypt-grow/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import type { ProductionItem } from '../../src/types';
import { SIZES, aim, loadFixture, mapSettled, selectThing, shot, type GameWindow } from './helpers';
import { setOverlay, showMapOptions } from '../overlayHelpers';
import { fromOffset, ring, within } from '../../src/hex';
import { idxOf, spawnUnit } from '../../src/test/fixtureEngine';

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

// The Nile on the Settle lens (2026-11-22, PR #18): an Egyptian Settler's site card reads Food only ("plenty of Food nearby")
// and the Details breakdown shows doubled Food and no Production. India is the control (Food and Production).
type LensWindow = GameWindow & { legatumMap: { lensTiles: { index: number; band?: string; rank?: number }[] } };
const SETTLE_CASES: { name: string; faction: 'Egypt' | 'India' }[] = [
  { name: 'e20-egypt-settle', faction: 'Egypt' },
  { name: 'e21-india-settle-control', faction: 'India' },
];
for (const size of SIZES) test.describe(`egypt-grow settle ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  for (const sc of SETTLE_CASES) test(sc.name, async ({ page }) => {
    test.setTimeout(150_000);
    const s = improvementCity({ improved: false });
    s.players[0].faction = sc.faction;
    // Varied land to the east so Food and Production differ from site to site.
    s.terrain.forEach((t, i) => {
      const col = i % s.width, row = Math.floor(i / s.width);
      if (col < 9 || col > 15 || t !== 'Grassland') return;
      if ((col + row) % 5 === 0) s.terrain[i] = 'Hills'; else if ((col * 3 + row) % 7 === 0) s.terrain[i] = 'Plains'; else if ((col + 2 * row) % 9 === 0) s.terrain[i] = 'Forest';
    });
    s.terrain.forEach((_, i) => { if (i % s.width <= 15) s.players[0].explored[i] = 1; });
    const settler = spawnUnit(s, 0, 'Settler', fromOffset(10, 8), false);
    await loadFixture(page, coreFixture(s), 12);
    await selectThing(page, 'unit', settler.id);
    await expect(page.locator('.unit-panel')).toContainText('Settler');
    await expect(page.locator('[data-settle-rank]')).toHaveCount(3);
    await park(page); await mapSettled(page);
    await shot(page, size, `${sc.name}-lens`);
    // Hover the best site, then a lower-ranked tinted one.
    const sites = await page.evaluate(() => {
      const w = window as unknown as LensWindow, v = w.legatum.getState().views![0];
      const at = (i: number) => ({ q: i % v.width - Math.floor(Math.floor(i / v.width) / 2), r: Math.floor(i / v.width) });
      const tiles = w.legatumMap.lensTiles;
      return { best: at(tiles.find((t) => t.rank === 1)!.index), other: at(tiles.find((t) => t.band && !t.rank && (t.band === 'Fair' || t.band === 'Poor'))?.index ?? tiles.find((t) => t.band && !t.rank)!.index) };
    });
    for (const [label, h] of [['hover-best', sites.best], ['hover-other', sites.other]] as const) {
      await page.evaluate((h) => (window as unknown as GameWindow).legatum.getState().jumpTo(h), h);
      await aim(page, h, page.locator('.settlement-card'));
      await page.waitForTimeout(500);
      const text = await page.locator('.settlement-card').innerText();
      if (sc.faction === 'Egypt') expect(text).not.toContain('Production');
      await shot(page, size, `${sc.name}-${label}`);
    }
    // Click through to the inspector and open Details.
    await showMapOptions(page);
    await page.locator('[data-settle-rank="1"]').click();
    const inspector = page.getByLabel('Settlement suitability').last();
    await expect(inspector.locator('[data-settle-band]')).toContainText('spot for a city');
    await park(page);
    await shot(page, size, `${sc.name}-inspect`);
    await inspector.locator('.settlement-breakdown summary').click();
    await expect(inspector).toContainText('Site quality:');
    await shot(page, size, `${sc.name}-inspect-details`);
  });
});

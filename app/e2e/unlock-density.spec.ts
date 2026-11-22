// Dense unlock cards on short screens (astra full review 2026-10-31 §3 rank 1): Machinery unlocks Hagia Sophia,
// Gunpowder unlocks two units and reveals Niter; Iron Working, Metallurgy, Industrialization (Coal), Combustion (Oil) and Autonomous Systems (Silicon) are also dense. Every row must sit inside the viewport and inside the card's scrolling body (rectangles, not
// DOM visibility), with the buttons pinned in view, at the user's 1000x600 laptop and at 1440x900.
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import { TECHS, TECH_ERA } from '../src/data';
import type { Faction, TechId } from '../src/types';

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shotDir = process.env.LEGATUM_SHOT_DIR ?? 'test-results';

async function finishTech(page: Page, faction: Faction, tech: TechId) {
  const state = researchFixture(faction);
  state.players[0].techs = TECHS.filter((t) => t !== tech && TECH_ERA[t] <= TECH_ERA[tech]);
  state.players[0].research = tech;
  state.players[0].science = 5000;
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(state));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await endPlanning(page);
}

for (const size of SIZES) test.describe(`unlock density ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  for (const [faction, tech] of [['India', 'Machinery'], ['India', 'Gunpowder'], ['Mongols', 'Gunpowder'], ['India', 'IronWorking'], ['India', 'Metallurgy'], ['India', 'Industrialization'], ['India', 'Combustion'], ['India', 'AutonomousSystems']] as const) test(`${tech} (${faction}): every row and both buttons are inside the viewport`, async ({ page }) => {
    await finishTech(page, faction, tech);
    const card = page.locator('.moment-card.unlock');
    await expect(card).toBeVisible();
    await page.mouse.move(size.width - 2, size.height - 2);
    await page.waitForTimeout(1600);
    await page.screenshot({ path: `${shotDir}/unlock-${tech}-${faction}-${size.width}x${size.height}.png`, animations: 'disabled' });
    // Machinery shows the painted wonder.
    if (tech === 'Machinery' && faction === 'India') {
      await expect(card.locator('.unlock-tile').filter({ hasText: 'Hagia Sophia' }).locator('.unlock-art')).toHaveCount(1);
      await expect(card.locator('.unlock-tile').filter({ hasText: 'Hagia Sophia' }).locator('.unlock-frame .mask-icon')).toHaveCount(0);
    }
    const rows = await card.locator('.unlock-tile').count();
    expect(rows).toBeGreaterThan(1);
    const body = (await card.locator('.unlock-body').boundingBox())!;
    const bodyScroll = await card.locator('.unlock-body').evaluate((el) => el.scrollHeight - el.clientHeight);
    for (let i = 0; i < rows; i++) {
      const r = (await card.locator('.unlock-tile').nth(i).boundingBox())!;
      expect(r.y, `row ${i} top in viewport`).toBeGreaterThanOrEqual(0);
      expect(r.y + r.height, `row ${i} bottom in viewport`).toBeLessThanOrEqual(size.height);
      expect(r.y + r.height, `row ${i} bottom inside the card body`).toBeLessThanOrEqual(body.y + body.height + 0.5);
    }
    expect(bodyScroll, 'the body needs no scrolling to show every row').toBeLessThanOrEqual(1);
    for (const name of [/Show |Continue|Choose/, /Later/]) {
      // The last research of the Future era has no "Later" (nothing is queued behind it).
      const button = card.getByRole('button', { name }).first();
      const b = (await button.count()) ? await button.boundingBox() : null;
      if (b) { expect(b.y).toBeGreaterThanOrEqual(0); expect(b.y + b.height).toBeLessThanOrEqual(size.height); }
    }
  });
});

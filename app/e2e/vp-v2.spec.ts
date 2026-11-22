import { expectRaceWhole } from './tour/helpers';
import { revealSidebar } from './sidebarAssertions';
// VP v2 in the real app (Rust core through WASM): the objectives sheet, decadence and
// the This Turn objective row. The v2 numbers that need a long game (an era entered, progress)
// are staged render-only on the live view with `withV2`. Set VP_V2_SHOTS to a folder to
// keep screenshots of each surface at both laptop sizes.
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';

// These specs are about the rest of the race panel (tokens, the Nobody row): the "How you win" block is left out (era-objectives.spec.ts covers it).
test.beforeEach(async ({ page }) => { await page.addInitScript(() => { try { localStorage.setItem('legatum.dev.no-era-hud', '1'); } catch { /* storage blocked */ } }); });
import type { useGame } from '../src/store';
import { withV2, type V2Options } from '../src/test/vpV2Fixture';
import { decadencePercent } from '../src/data';
import { endPlanning } from './turnActions';
import { chooseCapitalProduction } from './opening';
import { stageCityDecadence } from './vpStage';
import type { PlayerView } from '../src/types';

type GameWindow = Window & { legatum: typeof useGame };

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shots = process.env.VP_V2_SHOTS;
const stagedDecadence = decadencePercent(60, { decadenceSlope: 45 });

async function startEightEmpires(page: Page) {
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

/** Stage v2 numbers on the live view (render only). */
async function stage(page: Page, options: V2Options = {}) {
  const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
  const view: PlayerView = withV2(base, options);
  await page.evaluate((v) => { const store = (window as unknown as GameWindow).legatum, s = store.getState(); store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) }); }, view);
  await expect(page.getByRole('region', { name: 'Victory race', exact: true })).toBeVisible();
}

async function shot(page: Page, name: string, size: { width: number }) {
  await page.mouse.move(4, 300);
  // Let the sheet's fade-in finish.
  await page.waitForTimeout(350);
  if (shots) await page.screenshot({ path: join(shots, `${name}-${size.width}.png`) });
}

for (const size of SIZES) {
  test.describe(`VP v2 at ${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    test('the ledger lists each begun Age’s card, scorers and your progress', async ({ page }) => {
      await startEightEmpires(page);
      await page.locator('button.vh-chip').click();
      const sheet = page.getByTestId('objective-sheet');
      await expect(sheet.locator('.sl-row.k-work')).toHaveCount(1);
      // No Age has begun on turn 1: its card stays hidden until the first empire enters the era.
      await expect(sheet.locator('.sl-row.k-era')).toHaveCount(0);
      await expect(sheet).not.toContainText(/Developed Empire|Six Technologies|Twenty Population|Most Populous|Most Territory/);
      await page.keyboard.press('Escape');
      await stage(page, { era: 2, scored: [] });
      await page.locator('button.vh-chip').click();
      const era2 = sheet.locator('.sl-row[data-row="era-2"]');
      await expect(era2.locator('.sl-status')).toContainText('open to every empire');
      if (await era2.locator('button').getAttribute('aria-expanded') === 'false') await era2.locator('button').click();
      await expect(sheet.locator('#sl-detail-era-2')).toContainText('2 of 4');
      await expect(sheet.locator('#sl-detail-era-2 .vh-marker')).toHaveCount(1);
      await expect(sheet).not.toContainText(/fastest|recommend|suggest/i);
      const fit = await sheet.evaluate((el) => {
        const r = el.getBoundingClientRect(), body = el.querySelector('.sl-body')!;
        return { inside: r.left >= -1 && r.top >= -1 && r.right <= innerWidth + 1 && r.bottom <= innerHeight + 1, noHorizontal: body.scrollWidth - body.clientWidth <= 1 };
      });
      expect(fit).toEqual({ inside: true, noHorizontal: true });
      await shot(page, 'objectives-sheet', size);
      await page.keyboard.press('Escape');
      await stage(page, { era: 3, scored: [2] });
      await page.locator('button.vh-chip').click();
      await expect(sheet.locator('.sl-row[data-row="era-2"] .sl-status')).toContainText('scored');
      await expect(sheet.locator('.sl-row[data-row="era-2"] td.mine .sl-token')).toHaveCount(1);
      await page.keyboard.press('Escape');
      await stage(page, { era: 7, scored: [2, 3, 4] });
      await page.locator('button.vh-chip').click();
      // Six Ages have begun in the staged view.
      await expect(sheet.locator('.sl-row.k-era')).toHaveCount(6);
      const future = sheet.locator('.sl-row[data-row="era-7"]');
      await expect(future.locator('.sl-name')).toContainText('Future');
      const futureButton = future.locator('button');
      if (await futureButton.getAttribute('aria-expanded') === 'false') await futureButton.click();
      await expect(sheet.locator('#sl-detail-era-7 .vh-obj')).toHaveCount(3);
      const reach = await sheet.locator('.sl-body').evaluate((body) => {
        const lastRow = body.querySelector('.sl-row[data-row="era-7"]')!;
        lastRow.scrollIntoView({ block: 'end' });
        const r = body.getBoundingClientRect(), last = lastRow.getBoundingClientRect();
        return last.bottom <= r.bottom + 1 && last.top >= r.top - 1 && body.scrollWidth - body.clientWidth <= 1;
      });
      expect(reach).toBe(true);
      await shot(page, 'objectives-sheet-six-eras', size);
    });

    test('a held VP adds no Gold line', async ({ page }) => {
      await startEightEmpires(page);
      await stage(page, { era: 2, scored: [] });
      await expect(page.getByTestId('objective-progress')).toHaveCount(0);
      // Holding live VP does not add a Gold breakdown line.
      await page.locator('.gold-stat').click();
      await expect(page.locator('.tip-bubble .bd-row').filter({ hasText: /^Overreach:/ })).toHaveCount(0);
      await page.waitForTimeout(450);
      if (shots) await page.screenshot({ path: join(shots, `gold-breakdown-${size.width}.png`) });
    });

    test('a lead over the average empire shows as a lead chip and a decadence chip in the race, a note in the Gold breakdown and a list in the sheet', async ({ page }) => {
      await startEightEmpires(page);
      await stage(page, { era: 2, scored: [], lead: 60, rivalLead: 120 });
      const race = page.getByRole('region', { name: 'Victory race', exact: true });
      await expect(race.locator('.vp-row.mine').getByTestId('lead-chip')).toHaveText('+60%');
      await expect(race.getByTestId('lead-chip')).toHaveCount(2);
      // One rate on every yield: its own red chip with an arrow, in the same two rows.
      await expect(race.locator('.vp-row.mine').getByTestId('decadence-chip')).toHaveText(`${stagedDecadence}%`);
      await expect(race.getByTestId('decadence-chip')).toHaveCount(2);
      await expect(race.locator('.vp-row.mine').getByTestId('decadence-chip').locator('.icon')).toHaveCount(1);
      await expectRaceWhole(page);
      await revealSidebar(page, 'standings');
      await race.locator('.vp-row.mine').getByTestId('lead-chip').hover();
      await shot(page, 'v3-race-panel-lead', size);
      await page.locator('.gold-stat').click();
      const tip = page.locator('.tip-bubble').filter({ hasText: 'Decadence:' }).first();
      await expect(tip.getByTestId('decadence-note')).toContainText(`your Power is 60% above the average empire’s, so your Production, Gold and Science are cut by ${stagedDecadence}% and your cities grow ${stagedDecadence}% slower`);
      await expect(tip.getByTestId('decadence-note')).toContainText('Food itself is never cut');
      await page.waitForTimeout(400);
      if (shots) await page.screenshot({ path: join(shots, `v3-gold-hover-decadence-${size.width}.png`) });
      await page.keyboard.press('Escape');
      await page.locator('button.vh-chip').click();
      const sheet = page.getByTestId('objective-sheet');
      await expect(sheet.locator('.sl-col.mine .sl-decadence')).toContainText(`decadence −${stagedDecadence}%`);
      await expect(sheet.locator('.sl-col .sl-decadence')).toHaveCount(2);
      await expect(sheet.locator('.sl-col.mine .sl-decadence')).toHaveAttribute('title', /Power .*60% above the average empire/);
      await shot(page, 'v3-objectives-sheet-overreach', size);
    });

    test('a city’s breakdowns show each Decadence line with its reason, the growth bar carries the Food part, and one note explains the rate', async ({ page }) => {
      await startEightEmpires(page);
      await chooseCapitalProduction(page);
      await revealSidebar(page, 'turn-more');
      await expect(page.getByRole('button', { name: /^Pending orders/ })).toContainText('1');
      await endPlanning(page);
      await expect(page.getByTestId('turn-number')).toHaveText('2', { timeout: 30_000 });
      if (await page.locator('.moment-card, [data-celebrate-delta]').count()) await page.keyboard.press('Escape');
      await stage(page, { era: 2, lead: 60 });
      await stageCityDecadence(page, stagedDecadence);
      await page.evaluate(() => {
        const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
        s.set({ techOpen: false });
        s.select({ kind: 'city', id: v.cities.find((c) => c.owner === v.me)!.id }, true);
      });
      const panel = page.locator('.city-panel');
      // The yields sit in the economy details, closed until you open them.
      if (!(await panel.locator('details.cp-economy').evaluate((d) => (d as HTMLDetailsElement).open))) await panel.locator('details.cp-economy > summary').click();
      await expect(panel.locator('.cp-yield').first()).toBeVisible();
      const reasons = { production: 'of Production: sloth', gold: 'of Gold: luxury', science: 'of Science: complacency' } as const;
      // Food has no line: its hover points to the growth bar, whose breakdown carries the Food part.
      await panel.locator('.cp-yield').nth(0).click();
      const food = page.locator('.tip-bubble:not([hidden])').filter({ hasText: 'Decadence also slows growth: see the city’s growth bar.' }).first();
      await expect(food).toBeVisible();
      await expect(food).not.toContainText('Decadence −');
      await page.keyboard.press('Escape');
      await panel.getByTestId('city-growth').click();
      const growth = page.locator('.tip-bubble:not([hidden])').filter({ hasText: 'Decadence: growth needs +' }).first();
      await expect(growth).toBeVisible();
      await expect(growth.getByTestId('decadence-note')).toContainText(`your cities grow ${stagedDecadence}% slower`);
      expect(await growth.evaluate((el) => { const r = el.getBoundingClientRect(); return r.top >= -1 && r.bottom <= innerHeight + 1 && r.right <= innerWidth + 1; })).toBe(true);
      // The pinned popover scrolls inside past its cap: the decadence note must not be cut off.
      expect(await growth.evaluate((el) => el.scrollHeight <= el.clientHeight + 1)).toBe(true);
      await shot(page, 'v3-city-growth-decadence', size);
      await page.keyboard.press('Escape');
      let n = 0;
      for (const [i, reason] of Object.values(reasons).entries()) {
        await panel.locator('.cp-yield').nth(i + 1).click();
        const tip = page.locator('.tip-bubble:not([hidden])').filter({ hasText: `Decadence −${stagedDecadence}% ${reason}` }).first();
        await expect(tip).toBeVisible();
        await expect(tip.getByTestId('decadence-note')).toContainText(`are cut by ${stagedDecadence}%`);
        await expect(tip.getByTestId('decadence-note')).toContainText('no city starves');
        // The note sits inside the viewport under the table.
        const inside = await tip.evaluate((el) => { const r = el.getBoundingClientRect(); return r.top >= -1 && r.bottom <= innerHeight + 1 && r.left >= -1 && r.right <= innerWidth + 1; });
        expect(inside).toBe(true);
        if (i === 0 || i === 2) await shot(page, `v3-city-breakdown-${['production', 'gold', 'science'][i]}`, size);
        await page.keyboard.press('Escape');
        n++;
      }
      expect(n).toBe(3);
    });
  });
}

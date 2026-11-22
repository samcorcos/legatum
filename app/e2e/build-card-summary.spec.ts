import { mkdirSync } from 'node:fs';
import path from 'node:path';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import { TECHS } from '../src/data';
import { loadFixture, selectThing } from './tour/helpers';

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }] as const;
const SHOTS = path.resolve('test-results/build-card-text');

for (const size of SIZES) {
  test.describe(`${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    for (const era of ['modern', 'ancient'] as const) {
      test(`${era} production list keeps descriptions behind More`, async ({ page }) => {
        const state = researchFixture('India');
        const city = state.cities.find((c) => c.owner === 0 && c.isCapital)!;
        city.buildings = [];
        if (era === 'modern') state.players[0].techs = TECHS.slice(0, TECHS.indexOf('Flight') + 1);
        await loadFixture(page, coreFixture(state), 2);
        await selectThing(page, 'city', city.id);
        const panel = page.locator('.city-panel');
        const focus = panel.locator('.production-choice').first();
        await expect(focus).toBeVisible();
        await panel.evaluate((el) => {
          const target = el.querySelector('.cp-choose-title')!.getBoundingClientRect();
          const head = el.querySelector('.cp-head')!.getBoundingClientRect();
          el.scrollTop += target.top - head.bottom - 8;
        });
        await page.evaluate(() => document.fonts.ready.then(() => undefined));
        await page.mouse.move(1, 330);
        await page.waitForTimeout(400);
        const dir = path.join(SHOTS, 'after');
        mkdirSync(dir, { recursive: true });
        await page.screenshot({ path: path.join(dir, `${era}-${size.width}x${size.height}.png`), animations: 'disabled' });

        const fighter = panel.locator('.production-choice[data-item="u:Fighter"]');
        if (era === 'modern') {
          await expect(fighter.locator('.prod-effect')).toContainText('Guards 4 tiles');
          for (const item of ['Fighter', 'Bomber']) {
            const summary = panel.locator(`.production-choice[data-item="u:${item}"] .production-more > summary`);
            expect(await summary.evaluate((el) => el.scrollWidth <= el.clientWidth + 1), `${item} summary fits at ${size.width}x${size.height}`).toBe(true);
          }
          await expect(fighter.locator('.production-more-text')).toBeHidden();
          const more = fighter.locator('.production-more > summary');
          await more.focus();
          await page.keyboard.press('Enter');
          await expect(fighter.locator('.production-more-text')).toBeVisible();
          await expect(fighter.locator('.production-more-text')).toContainText('Wait stops it asking each turn');
          await more.click();
          await expect(fighter.locator('.production-more-text')).toBeHidden();
          await fighter.getByRole('button', { name: 'Build Fighter' }).hover();
          await expect(page.locator('[role="tooltip"]:not([hidden])')).toContainText('Wait stops it asking each turn');
        }
        // More shares the summary's first line; a long summary wraps whole onto a second, never an ellipsis, and never leaves More alone.
        const lines = await panel.locator('.production-choice .production-more > summary').evaluateAll((els) => els.filter((el) => el.getBoundingClientRect().height > 0).map((el) => {
          const style = getComputedStyle(el), effect = el.querySelector('.prod-effect')!, toggle = el.querySelector('.more-toggle')!.getBoundingClientRect();
          const rows = new Set([...effect.getClientRects()].map((r) => Math.round(r.top))).size;
          const first = [...effect.getClientRects()][0];
          return { text: effect.textContent, rows, ellipsis: getComputedStyle(effect).textOverflow === 'ellipsis', toggleOnFirstLine: toggle.top < first.bottom && toggle.bottom > first.top,
            contentHeight: el.getBoundingClientRect().height - Number.parseFloat(style.paddingTop) - Number.parseFloat(style.paddingBottom), lineHeight: Number.parseFloat(style.lineHeight) };
        }));
        expect(lines.every(({ rows, ellipsis, toggleOnFirstLine, contentHeight, lineHeight }) => rows <= 2 && !ellipsis && toggleOnFirstLine && contentHeight <= rows * lineHeight + 3), JSON.stringify(lines)).toBe(true);
        // Locked upgrades nest under their prerequisite now and no longer have a card of their own, so the single-line upgrade
        // cards (7 of 37 at 1000x600) left the count: the share dropped from 51% to 40%.
        expect(lines.filter((l) => l.rows === 1).length, `a good share of summaries share one line with More; two-line ones: ${JSON.stringify(lines.filter((l) => l.rows > 1).map((l) => l.text))}`).toBeGreaterThanOrEqual(lines.length * 0.4);
      });
    }
  });
}

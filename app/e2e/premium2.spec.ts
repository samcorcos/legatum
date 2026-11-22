// The score ledger keeps its heading and rows readable while its body scrolls.
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { progressFixture } from '../src/test/progressFixture';
import { stageHoldings } from './vpStage';

async function loadSave(page: Page, game: string, turn: number) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test(`the score ledger keeps its head whole while its body scrolls, and its scoring marks remain visible (${width}x${height})`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await loadSave(page, coreFixture(progressFixture().fixture), 2);
    await stageHoldings(page);
    // The sidebar has one scroll area, with the race headline pinned above its details.
    const race = page.locator('.panel.vp');
    const scroll = page.locator('.right-sidebar .sidebar-scroll');
    await scroll.evaluate((el) => { el.scrollTop = el.scrollHeight; });
    const headlineFit = await scroll.evaluate((el) => {
      const area = el.getBoundingClientRect(), headline = el.querySelector('.race-headline')!.getBoundingClientRect();
      return { top: headline.top - area.top, bottom: headline.bottom - area.bottom };
    });
    expect(headlineFit.top).toBeGreaterThanOrEqual(-1);
    expect(headlineFit.bottom).toBeLessThanOrEqual(1);
    await scroll.evaluate((el) => { el.scrollTop = 0; });
    await race.getByRole('button', { name: 'Score ledger', exact: true }).click();
    const sheet = page.getByTestId('objective-sheet');
    await expect(sheet).toBeVisible();
    const head = sheet.locator('.sl-head'), body = sheet.locator('.sl-body');
    // At the top and the very bottom of the body: the head is whole, inside the sheet, and not covered.
    for (const top of [0, 1e6]) {
      await body.evaluate((el, y) => { el.scrollTop = y; }, top);
      const [h, p] = await Promise.all([head.boundingBox(), sheet.boundingBox()]);
      expect(h!.y).toBeGreaterThanOrEqual(p!.y);
      expect(h!.height).toBeGreaterThan(24);
      expect(await head.evaluate((el) => { const r = el.getBoundingClientRect(); return el.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)); })).toBe(true);
    }
    await body.evaluate((el) => { el.scrollTop = 0; });
    // Every scoring source stays a complete row; held and open marks retain their distinct shapes.
    await expect(sheet.locator('.sl-row.k-goal')).toHaveCount(1);
    await expect(sheet.locator('.sl-row.k-heldGoal')).toHaveCount(1);
    await expect(sheet.locator('.sl-row.k-capital')).toHaveCount(1);
    await expect(sheet.locator('.sl-row.k-work')).toHaveCount(1);
    await expect(sheet.locator('.sl-row.k-era')).toHaveCount(2);
    await expect(sheet.locator('.sl-row.k-work .sl-socket').first()).toBeVisible();
    await expect(sheet.locator('.sl-row.k-era .sl-socket').first()).toBeVisible();
    const last = sheet.locator('.sl-row.k-era').last();
    await last.scrollIntoViewIfNeeded();
    const bounds = await Promise.all([last.boundingBox(), body.boundingBox()]);
    expect(bounds[0]!.y + bounds[0]!.height).toBeLessThanOrEqual(bounds[1]!.y + bounds[1]!.height + 1);
    // Close is in view, and Escape gives the map back.
    await expect(sheet.getByRole('button', { name: 'Close objectives' })).toBeInViewport();
    await page.keyboard.press('Escape');
    await expect(sheet).toHaveCount(0);
  });
}

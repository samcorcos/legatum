// The top bar at its most crowded: Iron and Coal revealed, a five-digit treasury, and either
// "Choose a technology" or the treasury warning. No two cells (or their visible text) may overlap,
// nothing may spill out of the bar, and on desktop the research text stays whole at 1440.
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { improvementCity } from '../src/test/improvementsFixture';
import { coreFixture } from '../src/test/coreFixture';

type GameWindow = Window & { legatum: typeof useGame };

async function crowdedTopBar(page: Page, touch: boolean) {
  const fixture = improvementCity({ techs: ['IronWorking'] });
  fixture.players[0].gold = 12345;
  fixture.players[0].research = null;
  fixture.players[0].research_queue = [];
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 12,
      names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
  }, coreFixture(fixture));
  await page.goto('/?client=wasm');
  const start = async (name: RegExp) => { const b = page.getByRole('button', { name }); if (touch) await b.tap(); else await b.click(); };
  await start(/^Continue/);
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  // The widest chips: Iron short (wants 12 of 10 supplied, 2 idle) and Coal with room to spare, staged in the view (render only).
  await stageSupply(page);
  await expect(page.getByTestId('supply-Iron')).toHaveText('0/10');
  await expect(page.getByTestId('supply-Niter')).toHaveText('2/3');
  await expect(page.getByTestId('supply-Coal')).toHaveText('1/4');
  await expect(page.getByTestId('supply-Oil')).toHaveText('1/2');
  const resourceMarks = page.locator('.supply-stat .supply-resource-icon');
  await expect(resourceMarks).toHaveCount(4);
  await expect(resourceMarks.nth(0)).toBeVisible();
  await expect(resourceMarks.nth(1)).toBeVisible();
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

/** Stage a shortage of Iron and a healthy Coal supply, in the Industrial era, in the local view (render only, no orders). */
async function stageSupply(page: Page) {
  await page.evaluate(() => {
    const store = (window as unknown as GameWindow).legatum, s = store.getState(), v = s.views![s.seat];
    const views = [...s.views!];
    // The Industrial era (4, 0-based) emphasizes Coal; all revealed resources remain accessible.
    views[s.seat] = { ...v, research: { ...v.research, era: 4 }, supply: [
      { resource: 'Iron', revealed_by: 'IronWorking', revealed: true, supplied: { total: 10, parts: [['Iron near Home', 5], ['Iron near Away, with a Mine', 5]] },
        used: { total: 12, parts: [['Forge in Home', 1], ['Pikeman ×11', 11]] }, idle: ['Pikeman', 'Pikeman'], held: [], spare: -2 },
      { resource: 'Niter', revealed_by: 'Gunpowder', revealed: true, supplied: { total: 3, parts: [['Niter near Home', 3]] },
        used: { total: 1, parts: [['Bank in Home', 1]] }, idle: [], held: [], spare: 2 },
      { resource: 'Coal', revealed_by: 'Industrialization', revealed: true, supplied: { total: 4, parts: [['Coal near Home', 4]] },
        used: { total: 3, parts: [['Factory in Home', 1], ['Factory in Away', 2]] }, idle: [], held: [], spare: 1 },
      { resource: 'Oil', revealed_by: 'Combustion', revealed: true, supplied: { total: 2, parts: [['Oil near Away', 2]] },
        used: { total: 1, parts: [['Research Lab in Away', 1]] }, idle: [], held: [], spare: 1 },
    ] };
    store.setState({ views });
  });
}

/** Stage the treasury warning in the local view (render only, no orders). */
async function stageTreasuryWarning(page: Page) {
  await page.evaluate(() => {
    const store = (window as unknown as GameWindow).legatum, s = store.getState(), v = s.views![s.seat];
    const views = [...s.views!];
    views[s.seat] = { ...v, economy: { ...v.economy, bankruptcy: { gold: { total: -12, parts: [['Treasury', 1], ['Income', -13]] }, disband: null } } };
    store.setState({ views });
  });
}

/** Overlaps between top bar cells, by box and by visible text (clipped by overflow-hiding ancestors). */
function topBarProblems(page: Page) {
  return page.evaluate(() => {
    const bar = document.querySelector<HTMLElement>('.topbar')!;
    const bb = bar.getBoundingClientRect();
    type Box = { l: number; r: number; t: number; b: number };
    const cells = [...bar.querySelectorAll<HTMLElement>('.brand, .tb-stats > :not(.supplies):not(.research-slot), .supply-stat, .research-chip, .tb-right > *')]
      .filter((e) => e.getClientRects().length > 0);
    const texts = (cell: HTMLElement) => {
      const out: Box[] = [];
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT);
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!n.textContent?.trim()) continue;
        const range = document.createRange();
        range.selectNodeContents(n);
        for (const r of range.getClientRects()) {
          let box: Box | null = { l: r.left, r: r.right, t: r.top, b: r.bottom };
          for (let a = n.parentElement; a && a !== bar.parentElement; a = a.parentElement) {
            const cs = getComputedStyle(a);
            if (cs.display === 'none' || cs.visibility === 'hidden') { box = null; break; }
            if (cs.overflowX !== 'visible') { const ar = a.getBoundingClientRect(); box.l = Math.max(box.l, ar.left); box.r = Math.min(box.r, ar.right); }
          }
          if (box && box.r - box.l > 0.5) out.push(box);
        }
      }
      return out;
    };
    const name = (e: Element) => `${e.className} "${e.textContent?.trim().slice(0, 24)}"`;
    const data = cells.map((e) => { const b = e.getBoundingClientRect(); return { n: name(e), box: { l: b.left, r: b.right, t: b.top, b: b.bottom }, texts: texts(e) }; });
    const overlaps = (a: Box, b: Box) => Math.min(a.r, b.r) - Math.max(a.l, b.l) > 1 && Math.min(a.b, b.b) - Math.max(a.t, b.t) > 1;
    const problems = new Set<string>();
    for (const d of data) {
      if (d.box.l < bb.left - 1 || d.box.r > bb.right + 1 || d.box.t < bb.top - 1 || d.box.b > bb.bottom + 1) problems.add(`outside the bar: ${d.n}`);
    }
    for (let i = 0; i < data.length; i++) for (let j = i + 1; j < data.length; j++) {
      const a = data[i], b = data[j];
      if (overlaps(a.box, b.box)) problems.add(`cells overlap: ${a.n} × ${b.n}`);
      if (a.texts.some((ta) => b.texts.some((tb) => overlaps(ta, tb)))) problems.add(`text overlaps: ${a.n} × ${b.n}`);
    }
    return [...problems];
  });
}

/** Research text shown in full (no ellipsis) in the desktop research cell. */
function researchTruncated(page: Page) {
  return page.locator('.topbar').evaluate((bar) => [...bar.querySelectorAll<HTMLElement>('.research-chip b, .research-chip-plan > span')]
    .some((e) => e.scrollWidth > e.clientWidth));
}

const sizes = [
  { width: 1440, height: 900, touch: false },
  // The user's laptop: the Cities button shares the bar, and the research cell still shows its longest prompt whole.
  { width: 1000, height: 600, touch: false },
  { width: 1280, height: 800, touch: false },
  { width: 1024, height: 768, touch: false },
];
for (const size of sizes) for (const look of ['atlas'] as const) test.describe(`${look} ${size.width}x${size.height}`, () => {
  test.use({ viewport: { width: size.width, height: size.height }, hasTouch: size.touch, isMobile: size.touch });
  test('top bar cells never overlap with Iron and Coal revealed', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    await crowdedTopBar(page, size.touch);
    const phone = size.width < 700;
    if (!phone) await expect(page.locator('.research-chip b')).toHaveText('Choose a technology');
    await expect.poll(() => topBarProblems(page)).toEqual([]);
    if (!phone) await expect.poll(() => researchTruncated(page)).toBe(false);
    if (size.width === 1000) {
      expect(await page.getByTestId('supply-Iron').evaluate((el) => getComputedStyle(el).fontSize)).toBe('0px');
      await page.locator('.supply-iron').hover();
      await expect(page.locator('.tip-summary').filter({ hasText: 'Iron: 0/10 spare/total' })).toBeVisible();
      await page.mouse.move(5, size.height / 2);
    }
    if (size.width === 1440) {
      expect(await page.getByTestId('supply-Iron').evaluate((el) => parseFloat(getComputedStyle(el).fontSize))).toBeGreaterThan(0);
    }
    // Iron and Coal read apart at a glance on a desktop bar: painted art at 24 px (8.9.4 review), and the Technologies
    // button is the first thing the bar drops (the research cell opens the same dialog).
    if (size.width >= 1280) {
      for (const mark of await page.locator('.supply-stat .supply-resource-icon').all()) {
        const box = (await mark.boundingBox())!;
        expect(box.width, 'a supply mark is at least 24 px').toBeGreaterThanOrEqual(24);
        expect(box.height).toBeGreaterThanOrEqual(24);
      }
      const names = await page.locator('.supply-stat .supply-resource-icon').evaluateAll((els) => els.map((e) => (e as HTMLImageElement).alt));
      expect(names).toEqual(['Iron', 'Niter', 'Coal', 'Oil']);
    }

    await stageTreasuryWarning(page);
    if (phone) {
      // No room for a warning cell: the Gold cell carries the alarm, its tip the details.
      await expect(page.locator('.topbar .gold-stat .gold-warn-mark')).toBeVisible();
      await expect(page.locator('.topbar .gold-warning')).toHaveCount(0);
    } else {
      await expect(page.locator('.topbar .gold-warning')).toContainText('Treasury −12 Gold');
    }
    await expect.poll(() => topBarProblems(page)).toEqual([]);
    if (!phone) await expect.poll(() => researchTruncated(page)).toBe(false);
    // At 1440 the resource names and research label survive even with the warning showing.
    if (size.width === 1440) {
      await expect(page.locator('.topbar .supply-name').first()).toBeVisible();
      await expect(page.locator('.topbar .research-label')).toBeVisible();
    }
    expect(errors).toEqual([]);
  });
});

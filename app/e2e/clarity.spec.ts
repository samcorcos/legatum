import { revealSidebar } from './sidebarAssertions';
import { expectRaceWhole } from './tour/helpers';
import { coreFixture } from '../src/test/coreFixture';
import { explorationFixture } from '../src/test/explorationFixture';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';

// In-game clarity regressions from the first playtest (plans/playtest-feedback.md A6, A7, A10–A12,
// A14, A15, F4, F10): notes never drawn over each other, tooltips that never block or cover what
// they explain, dialogs that fit a laptop screen, and disabled actions that say why.

type Box = { left: number; top: number; right: number; bottom: number; text: string };
// Atlas at laptop and desktop sizes.
const VIEWPORTS = [{ width: 1280, height: 800, look: 'atlas' }, { width: 1440, height: 900, look: 'atlas' }];

async function startSolo(page: Page, { empires = 4 } = {}) {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  if (empires === 4) {
    await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  }
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  const dismiss = page.getByRole('button', { name: /^Dismiss hint/ });
  if (await dismiss.count()) await dismiss.click();
}

async function selectOwn(page: Page, kind: string) {
  const found = await page.evaluate((k) => {
    const s = (window as unknown as { legatum: { getState(): any } }).legatum.getState();
    const v = s.views[s.seat];
    const target = k === 'city' ? v.cities.find((c: any) => c.owner === v.me) : v.units.find((u: any) => u.owner === v.me && u.kind === k);
    if (target) s.select({ kind: k === 'city' ? 'city' : 'unit', id: target.id }, true);
    return !!target;
  }, kind);
  expect(found, `own ${kind} to select`).toBe(true);
}

const boxes = (page: Page, selector: string) => page.locator(selector).evaluateAll((els) => els
  .filter((el) => (el as HTMLElement).offsetParent !== null)
  .flatMap((el) => [...el.getClientRects()].slice(0, 1).map((r) => ({ left: r.left, top: r.top, right: r.right, bottom: r.bottom, text: (el.textContent ?? '').trim().slice(0, 40) }))));

const rectOf = async (page: Page, selector: string): Promise<Box> => {
  const r = await page.locator(selector).first().evaluate((el) => { const b = el.getBoundingClientRect(); return { left: b.left, top: b.top, right: b.right, bottom: b.bottom }; });
  return { ...r, text: selector };
};

/** Hover like a person (pointer moves, then rests past the show delay) and return the visible tooltip. */
async function hoverTip(page: Page, selector: string) {
  const target = page.locator(selector).first();
  await target.hover();
  await page.waitForTimeout(650);
  const tip = page.locator('[role="tooltip"]:not([hidden])');
  await expect(tip).toHaveCount(1);
  return tip;
}

const intersects = (a: Box, b: Box) => a.left < b.right - 1 && b.left < a.right - 1 && a.top < b.bottom - 1 && b.top < a.bottom - 1;

for (const viewport of VIEWPORTS) {
  test.describe(`clarity at ${viewport.width}×${viewport.height} (${viewport.look})`, () => {
    test.use({ viewport: { width: viewport.width, height: viewport.height } });

    test('unit notes in the side panel never overlap each other and stay on screen (A6)', async ({ page }) => {
      await startSolo(page);
      const assertNotes = async (kind: string) => {
        await selectOwn(page, kind);
        await expect(page.locator('.unit-panel')).toBeVisible();
        const notes = await boxes(page, '.up-hints > .hint');
        for (const n of notes) {
          expect(n.top, `${n.text} on screen`).toBeGreaterThanOrEqual(0);
          expect(n.bottom, `${n.text} on screen`).toBeLessThanOrEqual(viewport.height);
        }
        for (let i = 0; i < notes.length; i++) for (let j = i + 1; j < notes.length; j++) {
          expect(intersects(notes[i], notes[j]), `"${notes[i].text}" overlaps "${notes[j].text}"`).toBe(false);
        }
      };
      for (const kind of ['Scout', 'Soldier']) await assertNotes(kind);
      // A saved expansion position still exercises Settler notes, even though new games have no Settler.
      await page.addInitScript((game) => localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 1, names: ['Ada'], savedAt: new Date().toISOString(), game })), coreFixture(explorationFixture()));
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Continue/ }).click();
      await assertNotes('Settler');
    });

    test('every disabled action shows its reason beside the button (A15, F10)', async ({ page }) => {
      await startSolo(page);
      await selectOwn(page, 'Soldier');
      const disabled = page.locator('.unit-panel button.act:disabled');
      expect(await disabled.count(), 'the Spearman cannot upgrade on turn 1').toBeGreaterThan(0);
      const reasons: string[] = [];
      for (const button of await disabled.all()) {
        const reason = await button.evaluate((el) => {
          const ids = (el.getAttribute('aria-describedby') ?? '').split(' ');
          const r = ids.map((id) => document.getElementById(id)).find((n) => n?.classList.contains('act-reason'));
          if (!r) return null;
          const box = r.getBoundingClientRect();
          return { text: r.textContent ?? '', visible: box.width > 0 && box.height > 0 && getComputedStyle(r).visibility !== 'hidden' && !r.closest('[role="tooltip"]') };
        });
        expect(reason, `${await button.textContent()} names why it is disabled`).not.toBeNull();
        expect(reason!.visible).toBe(true);
        expect(reason!.text.trim().length).toBeGreaterThan(0);
        reasons.push(reason!.text);
      }
      // The founded capital puts the opening Spearman in friendly territory;
      // the remaining upgrade blocker is its technology requirement.
      expect(reasons).toContain('Research Mathematics first');
    });

    test('the research dialog fits, its forecast shows in full, and capital Science is available (A11, A12)', async ({ page }) => {
      await startSolo(page);
      await page.locator('.topbar .research-chip').click();
      await expect(page.locator('.research-no-science')).toHaveCount(0);
      await page.getByRole('button', { name: 'Close research', exact: true }).click();
      await page.locator('.topbar .research-chip').click();
      const dialog = await rectOf(page, 'dialog.tech-modal');
      expect(dialog.top).toBeGreaterThanOrEqual(0);
      expect(dialog.bottom).toBeLessThanOrEqual(viewport.height);
      // Inspect the era card with the longest forecast (Writing: a building with city forecasts).
      await page.locator('.era-card[data-tech="Writing"] .era-details-chevron').click();
      const inDialog = async (selector: string) => {
        const r = await rectOf(page, selector);
        expect(r.top, `${selector} inside the dialog`).toBeGreaterThanOrEqual(dialog.top);
        expect(r.bottom, `${selector} inside the dialog`).toBeLessThanOrEqual(dialog.bottom);
      };
      await inDialog('.research-detail');
      await inDialog('.research-detail h3');
      await inDialog('.era-badge');
      await inDialog('.tech-modal .modal-foot');
      // The forecast stays above the footer; longer forecasts scroll in the screen's body.
      const finalLine = page.locator('.research-detail-columns p, .research-detail-columns summary').last();
      await finalLine.scrollIntoViewIfNeeded();
      const last = await finalLine.boundingBox();
      const detail = await rectOf(page, '.research-detail');
      expect(last!.y + last!.height).toBeLessThanOrEqual(detail.bottom + 1);
      const scroller = await rectOf(page, '.research-scroll');
      expect(last!.y + last!.height, 'the forecast ends inside the scrolling body').toBeLessThanOrEqual(scroller.bottom + 1);
      await expect(page.locator('.research-no-science')).toHaveCount(0);
    });

    test('the production chooser states the decision and its options look clickable before hover (A7)', async ({ page }) => {
      await startSolo(page);
      await selectOwn(page, 'city');
      const name = await page.locator('.cp-name').textContent();
      await expect(page.locator('.cp-choose-title')).toHaveText(`Choose what ${name} builds`);
      await page.mouse.move(viewport.width / 2, viewport.height - 4);
      const rows = page.locator('.city-panel .production-choice:not(.unavailable) button.prod');
      for (const row of (await rows.all()).slice(0, 3)) {
        const look = await row.evaluate((el) => {
          const cs = getComputedStyle(el);
          return { cursor: cs.cursor, border: parseFloat(cs.borderTopWidth), color: cs.borderTopColor, chevron: el.querySelector('.prod-go')?.textContent };
        });
        expect(look.cursor).toBe('pointer');
        expect(look.border).toBeGreaterThan(0);
        expect(look.color).not.toBe('rgba(0, 0, 0, 0)');
        expect(look.chevron).toBe('Build →');
      }
      // Build moves on to the next decision (Civ 6); reopening the city shows the pick
      // marked on its row until the turn resolves.
      const picked = page.locator('.city-panel').getByRole('button', { name: (await rows.first().getAttribute('aria-label'))!, exact: true });
      await rows.first().click();
      await expect(page.locator('.city-panel')).toHaveCount(0);
      if (await page.locator('dialog.tech-modal').isVisible()) await page.keyboard.press('Escape');
      await selectOwn(page, 'city');
      await expect(picked).toHaveAttribute('aria-pressed', 'true');
      await expect(picked).toHaveClass(/pending/);
      await expect(picked).toContainText('Chosen');
      // The city's header line never clips inside the panel.
      const panel = await rectOf(page, '.city-panel');
      for (const r of await boxes(page, '.cp-meta > *')) {
        expect(r.left, r.text).toBeGreaterThanOrEqual(panel.left);
        expect(r.right, r.text).toBeLessThanOrEqual(panel.right);
      }
    });

    test('tooltips wait, never take the pointer, and never cover their control or End turn (F4)', async ({ page }) => {
      await startSolo(page);
      await selectOwn(page, 'Scout');
      // A pass across the controls shows nothing: tooltips wait for the pointer to rest.
      await page.locator('.unit-panel button.act').first().hover();
      // Short on purpose: the tooltip opens 450 ms after the pointer rests, and a loaded machine (six workers, a build
      // alongside) can stall this wait past that, which then reads as a tooltip that did not wait.
      await page.waitForTimeout(40);
      await expect(page.locator('[role="tooltip"]:not([hidden])')).toHaveCount(0);
      // A race token opens its source's card the same way (the card is a tooltip: it waits, is pointer-transparent, covers neither the token nor the turn dock).
      for (const selector of ['.unit-panel button.act', '.vp-row.mine [data-source]', '.topbar .stat']) {
        await page.mouse.move(2, viewport.height / 2);
        await page.waitForTimeout(500);
        if (selector.includes('.vp-row')) await revealSidebar(page, 'standings');
        const tip = await hoverTip(page, selector);
        expect(await tip.evaluate((el) => getComputedStyle(el).pointerEvents)).toBe('none');
        const bubble = await rectOf(page, '[role="tooltip"]:not([hidden])');
        expect(intersects(bubble, await rectOf(page, selector)), `tooltip covers ${selector}`).toBe(false);
        expect(intersects(bubble, await rectOf(page, '.seatbar')), `tooltip for ${selector} covers the This Turn panel`).toBe(false);
        // The pointer passes straight through a tooltip to whatever lies under it.
        const cx = (bubble.left + bubble.right) / 2, cy = (bubble.top + bubble.bottom) / 2;
        expect(await page.evaluate(([x, y]) => !!document.elementFromPoint(x, y)?.closest('[role="tooltip"]'), [cx, cy])).toBe(false);
      }
      // Rich breakdowns open on click as a pinned popover with a close control; Escape dismisses.
      await page.locator('.topbar .stat').first().click();
      const pinned = page.getByRole('dialog', { name: 'Breakdown', exact: true });
      await expect(pinned).toContainText('Gold per turn');
      await expect(pinned.getByRole('button', { name: 'Close breakdown', exact: true })).toBeVisible();
      await page.keyboard.press('Escape');
      await expect(pinned).toHaveCount(0);
    });
  });
}

test.describe('the race panel with eight empires at 1280×800', () => {
  test.use({ viewport: { width: 1280, height: 800 } });
  test('shows its whole footer and its notes never cover the This Turn panel (A10)', async ({ page }) => {
    await startSolo(page, { empires: 8 });
    await expectRaceWhole(page);
    await revealSidebar(page, 'standings');
    await hoverTip(page, '.vp-row.mine [data-source]');
    expect(intersects(await rectOf(page, '[role="tooltip"]:not([hidden])'), await rectOf(page, '.seatbar'))).toBe(false);
  });
});

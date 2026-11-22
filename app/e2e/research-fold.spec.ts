// The research screen at the laptop size: a technology card's name, art and both buttons are in view together without scrolling,
// on turn 1 (idle queue, no Science yet, the longest head) and with a queue filled. The queue strip stays one slot per technology
// (research-click.spec.ts pins that nothing moves when picking). Set R_SHOTS to keep screenshots.
import { join } from 'node:path';
import type { Locator, Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import { withV2 } from '../src/test/vpV2Fixture';
import type { useGame } from '../src/store';

type GameWindow = Window & { legatum: typeof useGame };

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shots = process.env.R_SHOTS;

async function turnOne(page: Page) {
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.locator('.topbar .research-chip').click();
  const d = page.locator('dialog.tech-modal');
  await expect(d).toBeVisible();
  await page.waitForTimeout(400);
  return d;
}

async function midGame(page: Page) {
  const f = researchFixture('India');
  f.players[0].research = null as never;
  f.players[0].research_queue = [] as never;
  const game = coreFixture(f);
  await page.addInitScript(({ game }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.locator('.topbar .research-chip').click();
  const d = page.locator('dialog.tech-modal');
  await expect(d).toBeVisible();
  await page.waitForTimeout(400);
  return d;
}

/** The first card sits whole inside the scrolling body: its head (name), the art and both buttons. */
async function expectFirstCardWhole(d: Locator, page: Page, name: string, width: number, card = '.era-card.available', buttons = ['research-now', 'queue-add']) {
  await page.mouse.move(2, 2);
  await page.waitForTimeout(300);
  if (shots) await page.screenshot({ path: join(shots, `research-${name}-${width}.png`) });
  const g = await d.evaluate((dialog, [cardSelector, ids]) => {
    const scroll = dialog.querySelector('.research-scroll')!.getBoundingClientRect();
    const card = dialog.querySelector(cardSelector)!;
    const inside = (el: Element | null) => { if (!el) return false; const b = el.getBoundingClientRect(); return b.top >= scroll.top - 0.5 && b.bottom <= scroll.bottom + 0.5; };
    return {
      scroll: { top: Math.round(scroll.top), bottom: Math.round(scroll.bottom), h: Math.round(scroll.height), card: Math.round(card.getBoundingClientRect().height) },
      nameIn: inside(card.querySelector('.era-card-name')),
      artIn: inside(card.querySelector('.era-unlock-art, .era-stepping')),
      buttonsIn: ids.map((id) => inside(card.querySelector(`[data-testid="${id}"]`))),
    };
  }, [card, buttons] as [string, string[]]);
  expect.soft(g.nameIn, `the card name is in view: ${JSON.stringify(g.scroll)}`).toBe(true);
  expect.soft(g.artIn, 'the card art is in view').toBe(true);
  expect.soft(g.buttonsIn, `the card's buttons (${buttons.join(', ')}) are in view`).toEqual(buttons.map(() => true));
}

for (const size of SIZES) {
  test.describe(`research screen fold at ${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    test('turn 1: the first card is whole at scroll 0 under the longest head and an idle queue strip', async ({ page }) => {
      const d = await turnOne(page);
      const queueAll = d.getByTestId('queue-all');
      await expect(queueAll).toHaveClass(/\bprimary\b/);
      const actionStyle = await queueAll.evaluate((button) => ({
        height: button.getBoundingClientRect().height,
        background: getComputedStyle(button).backgroundColor,
      }));
      expect(actionStyle.height, 'Queue all has a clear button-sized target').toBeGreaterThanOrEqual(32);
      expect(actionStyle.background, 'Queue all uses the primary button fill').not.toBe('rgba(0, 0, 0, 0)');
      await expectFirstCardWhole(d, page, 'turn1', size.width);
    });

    test('mid-era with the objectives header (this era and the next): the first card is whole at scroll 0', async ({ page }) => {
      await turnOne(page).then((d) => d.page().keyboard.press('Escape'));
      // A game under way: Science is flowing (no "No Science yet" note in the head), and Ages have begun with progress on their cards.
      const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
      const view = withV2(base, { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [3, 4], '2:2': [0, 1] } });
      view.economy.science_per_turn.total = 4;
      await page.evaluate((v) => {
        const store = (window as unknown as GameWindow).legatum, s = store.getState();
        store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) });
      }, view);
      await page.locator('.topbar .research-chip').click();
      const d = page.locator('dialog.tech-modal');
      await expect(d.getByTestId('research-objectives-current')).toContainText('Score 1 VP this Age');
      await expectFirstCardWhole(d, page, 'mid-era', size.width);
    });

    test('a queue with three picks: the first card is still whole, and the strip keeps one slot per technology', async ({ page }) => {
      const d = await midGame(page);
      await expectFirstCardWhole(d, page, 'idle', size.width);
      const strip = d.locator('.research-queue');
      const before = (await strip.boundingBox())!;
      for (const tech of ['Masonry', 'Writing', 'Currency']) await d.locator(`.era-card[data-tech="${tech}"]`).click({ position: { x: 60, y: 24 }, modifiers: tech === 'Masonry' ? [] : ['Shift'] });
      await expect(d.locator('.research-queue li')).toHaveCount(3);
      const after = (await strip.boundingBox())!;
      expect(Math.abs(after.height - before.height), 'the strip does not change height').toBeLessThan(0.75);
      await expectFirstCardWhole(d, page, 'queued', size.width, '.era-card.queued', ['research-now']);
    });

    test('one queued technology: the queue header has no overlapping neighbours', async ({ page }) => {
      const d = await midGame(page);
      await d.locator('.era-card[data-tech="Masonry"]').click({ position: { x: 60, y: 24 }, modifiers: ['Shift'] });
      await expect(d.locator('.research-queue li')).toHaveCount(1);
      const overlaps = await d.locator('.research-queue-head').evaluate((head) => {
        const children = [...head.children].filter((el) => {
          const style = getComputedStyle(el), box = el.getBoundingClientRect();
          return style.display !== 'none' && box.width > 0 && box.height > 0;
        }).map((el) => ({
          name: el.className || el.textContent?.trim(),
          box: (() => { const r = el.getBoundingClientRect(); return { left: r.left, right: r.right, top: r.top, bottom: r.bottom }; })(),
        }));
        const hits: string[] = [];
        for (let i = 0; i < children.length; i++) for (let j = i + 1; j < children.length; j++) {
          const a = children[i], b = children[j];
          if (Math.min(a.box.right, b.box.right) - Math.max(a.box.left, b.box.left) > 1 &&
              Math.min(a.box.bottom, b.box.bottom) - Math.max(a.box.top, b.box.top) > 1) hits.push(`${a.name} overlaps ${b.name}`);
        }
        return hits;
      });
      expect(overlaps, `queue header children overlap at ${size.width}x${size.height}`).toEqual([]);
      // The head's text and the first slot are whole: the summary is one short line ("Done T20"), nothing is cut (8.9.4 review).
      const cut = await d.locator('.research-queue').evaluate((queue) => [...queue.querySelectorAll<HTMLElement>('.research-queue-title, .research-queue-hint, .research-queue-total, li:first-child .rq-name, li:first-child .rq-arrival')]
        .filter((e) => getComputedStyle(e).display !== 'none' && e.scrollWidth > e.clientWidth + 1).map((e) => `${e.className}: ${e.textContent}`));
      expect(cut, 'queue head text and slot 1 are not truncated').toEqual([]);
      await expect(d.getByTestId('queue-total')).toHaveText(/^Done T\d+$/);
      const total = (await d.getByTestId('queue-total').boundingBox())!;
      expect(total.height, 'the summary is one line').toBeLessThan(24);
    });
  });
}

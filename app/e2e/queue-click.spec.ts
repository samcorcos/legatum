// Clicking + Queue confirms in place: nothing under the pointer moves, so a second click in a row (on the
// same card or the next one) lands on its target. The queue is its own visible, editable section.
// Runs the real engine (WASM) on an injected save.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Locator, Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { wallsFixture } from '../src/test/wallsFixture';

type GameWindow = Window & { legatum: typeof useGame };

async function openCity(page: Page, size: { width: number; height: number }) {
  const setup = wallsFixture();
  Object.assign(setup.fixture.cities.find((c) => c.id === setup.own)!, { population: 3, buildings: [], producing: null, prod: 0, food: 0, productionQueue: [], productionBanked: [] });
  setup.fixture.turn = 2;
  const game = coreFixture(setup.fixture);
  await page.addInitScript(({ game }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game });
  await page.setViewportSize(size);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    s.set({ techOpen: false });
    s.select({ kind: 'city', id }, true);
  }, setup.own);
  return page.locator('.city-panel');
}

const queueOrder = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  const order = s.orders[s.seat].find((o) => o.type === 'SetProductionQueue');
  const queue = order && order.type === 'SetProductionQueue' ? order.queue : [];
  return queue.map((i) => i.kind === 'Unit' ? `u:${i.unit}` : i.kind === 'Building' ? `b:${i.building}` : i.kind);
});

/** The same box, give or take sub-pixel rounding from scrolling. */
const same = (a: { x: number; y: number; width: number; height: number } | null, b: typeof a, what: string) => {
  expect(a && b, what).toBeTruthy();
  for (const k of ['x', 'y', 'width', 'height'] as const) expect(Math.abs(a![k] - b![k]), `${what}: ${k}`).toBeLessThan(0.75);
};

/** The order badge sits in the icon column: it must not touch any text line of its card (it once overprinted "No upkeep"). */
async function expectBadgeClear(card: Locator) {
  const badge = (await card.locator('.queue-badge').first().boundingBox())!;
  for (const text of await card.locator('.prod-effect, .prod-cost, .prod-turns, .prod-upkeep').all()) {
    const box = await text.boundingBox();
    if (!box) continue;
    const sameRow = badge.y < box.y + box.height && box.y < badge.y + badge.height;
    const overlaps = badge.x < box.x + box.width && box.x < badge.x + badge.width && sameRow;
    expect(overlaps, `the queue badge overlaps ${await text.evaluate((el) => el.className)}`).toBe(false);
    if (sameRow) expect(box.x - (badge.x + badge.width), 'a visible gap between the badge and the text').toBeGreaterThanOrEqual(3);
  }
}

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`two Queue clicks in a row both register and nothing moves between them at ${size.width}x${size.height}`, async ({ page }) => {
    const panel = await openCity(page, size);
    const cards = panel.locator('.production-choice:not([data-unavailable])');
    expect(await cards.count()).toBeGreaterThanOrEqual(3);
    const [first, second, third] = [cards.nth(0), cards.nth(1), cards.nth(2)];
    const firstItem = (await first.getAttribute('data-item'))!, secondItem = (await second.getAttribute('data-item'))!;
    await expect(panel.getByTestId('production-queue')).toContainText('add from the list below');
    // The panel now opens with growth and progress above the picker, so at 1000x600 the second card starts below the fold:
    // put the first card just under the sticky city head so all three are in view and a click never has to scroll (a scroll is not a layout shift).
    await panel.evaluate((el, id) => {
      const card = el.querySelector(`.production-choice[data-item="${id}"]`)!;
      const edge = el.querySelector('.cp-head')!.getBoundingClientRect().bottom;
      el.scrollTo({ top: el.scrollTop + card.getBoundingClientRect().top - edge - 8, behavior: 'instant' });
    }, firstItem);
    const box = (card: typeof first) => card.boundingBox();
    const before = { first: await box(first), second: await box(second), third: await box(third) };

    await first.getByRole('button', { name: /^Queue /, exact: false }).click();
    // The confirmation is on the button, the card keeps its place and height, and no line was inserted.
    await expect(first.locator('.queue-add')).toContainText('Queued #1');
    await expect(first.locator('.production-notice')).toHaveCount(0);
    await expect(first.locator('.queue-badge')).toHaveText('1');
    await expectBadgeClear(first);
    // The first queued item is the one in Now producing, so the list below starts at #2: no row yet.
    await expect(panel.getByTestId('production-queue').locator('li')).toHaveCount(0);
    const between = { first: await box(first), second: await box(second), third: await box(third) };
    for (const k of ['first', 'second', 'third'] as const) same(between[k], before[k], `${k} card moved after the first click`);

    // The very next click, with no wait, on the neighbouring card.
    await second.getByRole('button', { name: /^Queue /, exact: false }).click();
    await expect(second.locator('.queue-add')).toContainText('Queued #2');
    const after = { first: await box(first), second: await box(second), third: await box(third) };
    for (const k of ['first', 'second', 'third'] as const) same(after[k], before[k], `${k} card moved after the second click`);
    await expect(panel.getByTestId('production-queue').locator('li')).toHaveCount(1);
    expect(await queueOrder(page)).toEqual([firstItem, secondItem]);

    // The button settles back after a moment, at the same size.
    await expect(first.locator('.queue-add')).toHaveText(/^\+ Queue$/, { timeout: 3000 });
    same(await box(third), before.third, 'third card after the button settled');

    // The queue section is plainly editable: icon, name, turns, reorder and remove, all visible without hover.
    const row = panel.getByTestId('production-queue').locator('li').nth(0);
    await expect(row.locator('.icon').first()).toBeVisible();
    await expect(row.locator('.queue-turns')).toHaveText(/turn|—/);
    for (const name of [/^Move .* up$/, /^Move .* down$/, /^Remove .* from queue$/]) await expect(row.getByRole('button', { name })).toBeVisible();
    await panel.getByRole('button', { name: /^Move .* up$/ }).nth(0).click();
    expect(await queueOrder(page)).toEqual([secondItem, firstItem]);
    await panel.getByRole('button', { name: /^Remove .* from queue$/ }).first().click();
    await expect(panel.getByTestId('production-queue').locator('li')).toHaveCount(0);
  });
}

test('queueing the same unit twice counts up on its card and the queue, with no inserted line', async ({ page }) => {
  const panel = await openCity(page, { width: 1000, height: 600 });
  const scout = panel.locator('.production-choice[data-item="u:Scout"]');
  await scout.scrollIntoViewIfNeeded();
  const next = panel.locator('.production-choice').nth(1);
  const before = await next.boundingBox();
  await scout.getByRole('button', { name: 'Queue Scout' }).click();
  await scout.getByRole('button', { name: 'Queue Scout' }).click();
  await expect(scout.locator('.queue-add')).toContainText('Queued ×2');
  await expect(scout.locator('.queue-badge')).toHaveText('×2');
  await expectBadgeClear(scout);
  same(await next.boundingBox(), before, 'next card');
  await expect(panel.getByTestId('production-queue').locator('li')).toHaveCount(1);
});

// Screenshots of the city panel for the UI reviewer: idle, two queued, a long queue, after reordering and removing.
// Saved to app/test-results/queue-shots/<size>/<NN-state>.png (looked at, never compared).
const shotsRoot = fileURLToPath(new URL('../test-results/queue-shots/', import.meta.url));
for (const size of [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }]) {
  test(`city panel queue states at ${size.name}`, async ({ page }) => {
    const panel = await openCity(page, size);
    const dir = `${shotsRoot}${size.name}/`;
    await mkdir(dir, { recursive: true });
    const snap = async (name: string) => {
      await page.evaluate(() => document.fonts.ready.then(() => undefined));
      await page.waitForTimeout(1800); // the "Queued #N" confirmation has settled
      await page.mouse.move(2, 2);
      await page.screenshot({ path: `${dir}${name}.png`, animations: 'disabled' });
    };
    const cards = panel.locator('.production-choice:not([data-unavailable])');
    const queueNth = async (n: number) => cards.nth(n).getByRole('button', { name: /^Queue /, exact: false }).click();
    await snap('01-idle');
    await queueNth(0); await queueNth(1);
    await snap('02-two-queued');
    await queueNth(2); await queueNth(0); await queueNth(1);
    await snap('03-five-queued');
    await panel.getByRole('button', { name: /^Move .* up$/ }).nth(2).click();
    await panel.getByRole('button', { name: /^Remove .* from queue$/ }).nth(3).click();
    await snap('04-reordered-and-removed');
    await panel.evaluate((el) => { el.scrollTop = 0; });
    await snap('05-scrolled-top');
  });
}

// Shared helpers for the screen tour domains in this directory (see e2e/tour/*.spec.ts).
// Each domain spec is a slice of the old single screen-tour spec; every shot and assertion is kept.
import { mkdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Locator, Page } from '@playwright/test';
import type { useGame } from '../../src/store';
import type { Hex } from '../../src/types';
import { test, expect } from '../fixtures';
import { auditLayout, auditOverlaySlots, visibleTextBlocks, type LayoutIssue } from '../layoutAudit';
import { auditBlocks } from '../../src/test/textArtifacts';
import { coreFixture } from '../../src/test/coreFixture';
import { hallFixture, type HallMode } from '../../src/test/hallFixture';
import { momentsForTurn } from '../../src/celebrate/moments';
import { contextFromViews, formatEvents } from '../../src/format';
import type { Event } from '../../src/types';
import { basename } from 'node:path';

export type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };

export const SIZES = [
  { name: '1000x600', width: 1000, height: 600 },
  { name: '1440x900', width: 1440, height: 900 },
] as const;
export type Size = (typeof SIZES)[number];

/** Reasons must wrap inside their own panel instead of relying on a title tooltip or an ellipsis. */
export async function expectReasonsFit(page: Page) {
  const issues = await page.locator('.unit-panel .act-reason, .unit-panel .up-status-why, .city-panel [data-testid*="reason"]').evaluateAll((els) => els.map((el) => ({
    text: el.textContent?.trim(), overflow: el.scrollWidth - el.clientWidth, ellipsis: getComputedStyle(el).textOverflow,
  })).filter((x) => x.overflow > 1 || x.ellipsis === 'ellipsis'));
  expect(issues, 'reason text wraps without overflow or an ellipsis').toEqual([]);
}


// LEGATUM_TOUR_OUT moves the shots (the release check runs Chromium and WebKit side by side and each wipes its own output directory).
const outRoot = process.env.LEGATUM_TOUR_OUT ? `${process.env.LEGATUM_TOUR_OUT.replace(/\/$/, '')}/` : fileURLToPath(new URL('../../test-results/screen-tour/', import.meta.url));

/** Save one numbered screen, plus its layout-audit findings (recorded, never asserted). Its visible text is asserted artifact-free. */
export async function shot(page: Page, size: Size, name: string) {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  // Let entrance transitions and camera glides finish; screenshots also freeze CSS animations.
  await page.waitForTimeout(400);
  // One directory per window size, then one per domain (the spec file's name): domains run alone and never overwrite each other.
  const dir = `${outRoot}${size.name}/${basename(test.info().file, '.spec.ts')}/`;
  await mkdir(dir, { recursive: true });
  await page.screenshot({ path: `${dir}${name}.png`, fullPage: true, animations: 'disabled' });
  let issues: LayoutIssue[] = notes.get(page) ?? [];
  notes.delete(page);
  try {
    issues = [...issues, ...await auditOverlaySlots(page), ...await auditLayout(page, { minTarget: 24 })];
  } catch (e) {
    issues = [{ kind: 'offscreen', where: 'audit', detail: `audit failed: ${String(e)}` }];
  }
  const file = `${dir}${name}.audit.json`;
  await writeFile(file, JSON.stringify(issues, null, 2));
  // The artifact guard (legatum-ui: build UI text from structured data): no screen prints a stray join like ",." or "undefined".
  const artifacts = auditBlocks(`${size.name}/${name}`, await visibleTextBlocks(page));
  expect.soft(artifacts, artifacts.join('\n')).toEqual([]);
}

/** Every folded standing can be scrolled fully into view; no inner height cap strands a row. */
export async function expectRaceWhole(page: Page) {
  const sidebar = page.locator('.right-sidebar');
  const scroll = sidebar.locator('.sidebar-scroll');
  const detail = sidebar.locator('.sidebar-standings');
  const wasOpen = await detail.evaluate(el => (el as HTMLDetailsElement).open);
  const top = await scroll.evaluate(el => el.scrollTop);
  if (!wasOpen) await detail.locator('summary').click();
  for (const row of await sidebar.locator('.vp-row').all()) {
    await row.evaluate(el => el.scrollIntoView({ block: 'center' }));
    const b = (await row.boundingBox())!, area = (await scroll.boundingBox())!;
    expect(b.y).toBeGreaterThanOrEqual(area.y);
    expect(b.y + b.height).toBeLessThanOrEqual(area.y + area.height + 1);
    expect(await row.evaluate(el => el.scrollWidth <= el.clientWidth + 1)).toBe(true);
  }
  if (!wasOpen) { await detail.locator('summary').evaluate(el => el.scrollIntoView({ block: 'center' })); await detail.locator('summary').click(); }
  await scroll.evaluate((el, top) => { el.scrollTop = top; }, top);
}

/** The primary stays outside scrolling content, even with expanded turn details. */
export async function expectDockWhole(page: Page) {
  const cta = page.locator('.seatbar .cta');
  const b = (await cta.boundingBox())!;
  expect(b.y).toBeGreaterThan(0);
  expect(b.y + b.height).toBeLessThanOrEqual(page.viewportSize()!.height);
  expect(await cta.evaluate(el => { const b = el.getBoundingClientRect(); return el.contains(document.elementFromPoint(b.x + b.width / 2, b.bottom - 2)); })).toBe(true);
}

export async function loadFixture(page: Page, game: string, turn: number, hints = false) {
  await page.addInitScript(({ game, turn, hints }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: hints, seen: [] }));
  }, { game, turn, hints });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  // WebKit can spend several seconds compiling WASM and painting a large saved game on the box.
  // The turn number, rather than the Continue click, is the ready signal for the fixture.
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 60_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

export async function selectThing(page: Page, kind: 'unit' | 'city', id: number) {
  await page.evaluate(({ kind, id }) => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    s.set({ techOpen: false });
    s.select({ kind, id }, true);
    s.jumpTo((kind === 'unit' ? v.units : v.cities).find((x) => x.id === id)!.pos);
  }, { kind, id });
}

/** Screen point of a hex once the camera glide has settled. */
export async function hexPoint(page: Page, hex: Hex) {
  // jumpTo animates the camera for 380 ms. Two immediate reads can agree before its first
  // animation frame on WebKit, leaving a hover aimed at the tile's old screen position.
  await page.waitForTimeout(400);
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))));
  const at = () => page.evaluate((hex) => {
    const p = (window as unknown as GameWindow).legatumMap.toScreen(hex);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) };
  }, hex);
  let last = await at();
  await expect.poll(async () => { const now = await at(); const still = now.x === last.x && now.y === last.y; last = now; return still; }, { intervals: [100] }).toBe(true);
  return last;
}

export const posOf = (page: Page, kind: 'unit' | 'city', id: number) => page.evaluate(({ kind, id }) => {
  const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
  return (kind === 'unit' ? v.units : v.cities).find((x) => x.id === id)!.pos;
}, { kind, id });

/** Findings noticed while driving a screen, saved into that screen's audit file. */
const notes = new WeakMap<Page, LayoutIssue[]>();

/** Hover a target until its card (the forecast) shows.
 * If something covers the target's centre (e.g. a city banner at small zooms), record it and
 * aim at the lower part of the hex instead, as a player would. */
export async function aim(page: Page, hex: Hex, card: Locator) {
  let p = await hexPoint(page, hex);
  const cover = await page.evaluate(({ x, y }) => {
    const el = document.elementFromPoint(x, y);
    if (!el || el.closest('[data-game-map]')) return null;
    return `${el.tagName.toLowerCase()}.${[...el.classList].join('.')} "${(el.textContent ?? '').trim().slice(0, 40)}"`;
  }, p);
  if (cover) {
    notes.set(page, [...notes.get(page) ?? [], { kind: 'overlap', where: `target hex ${hex.q},${hex.r}`, detail: `centre covered by ${cover}` }]);
    const below = await hexPoint(page, { q: hex.q, r: hex.r + 1 });
    p = { x: Math.round(p.x + (below.x - p.x) * 0.4), y: Math.round(p.y + (below.y - p.y) * 0.4) };
  } else {
    // A city banner (drawn on the canvas, above units) over the hex's centre: its token is raised to the banner's
    // top edge (render.ts::bannerLift), so aim at the token's head just above the banner, as a player would.
    const banner = await page.evaluate(({ x, y }) => {
      const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
      const hits = (window as unknown as { legatumMap: { banners?: { cover: { x: number; y: number; w: number; h: number }; parts: { x: number; y: number; w: number; h: number }[]; far: boolean }[] } }).legatumMap.banners ?? [];
      const hit = hits.findLast((h) => !h.far && h.parts.some((r) => x - b.left >= r.x && x - b.left <= r.x + r.w && y - b.top >= r.y && y - b.top <= r.y + r.h));
      return hit ? Math.round(hit.cover.y + b.top) : null;
    }, p);
    if (banner !== null) p = { x: p.x, y: banner - 4 };
  }
  // A hover can land before the map has taken the new selection: wiggle until the card shows.
  await expect.poll(async () => {
    await page.mouse.move(p.x + 3, p.y + 3);
    await page.mouse.move(p.x, p.y, { steps: 3 });
    return card.isVisible();
  }, { intervals: [500, 1000, 1000, 2000] }).toBe(true);
}

/** Move the mouse off the map (onto the top bar) so no hover card or select ring is in the next shot. */
export async function parkMouse(page: Page) {
  await page.mouse.move(20, 25);
}

/** Wait until the camera has centred the selected unit (selection jumps glide the camera). */
export async function centred(page: Page, id: number) {
  await expect.poll(() => page.evaluate((id) => {
    const w = window as unknown as GameWindow, s = w.legatum.getState(), v = s.views![s.seat];
    const p = w.legatumMap.toScreen(v.units.find((u) => u.id === id)!.pos);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return Math.hypot(p.x - b.width / 2, p.y - b.height / 2);
  }, id)).toBeLessThan(2);
}

/** Pillar 9: the visible primary actions (filled buttons, and the turn's call to action unless it stepped back). */
export const primaries = (page: Page) => page.locator('.btn.primary:visible, .act.primary:visible, .cta.todo:not(.stepped):visible, .cta.ready:not(.stepped):visible').count();

/** Skip any celebration through its normal dismissal, then open the turn report. */
export async function openReport(page: Page) {
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect.poll(async () => {
    const open = page.getByTestId('moment-lane').getByRole('button', { name: 'Turn report', exact: true });
    if (await open.isVisible()) await open.click();
    return report.isVisible();
  }).toBe(true);
  // Opening the report used to scroll the app root (.game was overflow:hidden), which shifted the whole HUD up
  // 78 px in the 21g shot at 1440x900. .game is overflow:clip now, so it can never scroll; this guards it.
  expect(await page.evaluate(() => [...document.querySelectorAll('.game')].map((el) => el.scrollTop))).toEqual([0]);
  return report;
}

/** Bring a report line into view by scrolling the report's own list, never the page: Playwright's
 * scrollIntoViewIfNeeded also scrolls the app root, which shifted the whole screen in the shot. */
export async function scrollWithinReport(target: Locator) {
  await target.evaluate((el) => {
    const body = el.closest('.report-body') as HTMLElement | null;
    if (!body) return;
    body.scrollTop += el.getBoundingClientRect().top - body.getBoundingClientRect().top - 8;
  });
}

/** A hover card never hides part of itself behind its height cap: its last line ends inside the card. */
export async function expectWhole(card: Locator) {
  const m = await card.evaluate((el) => {
    const box = el.getBoundingClientRect();
    let last = [...el.children].filter((c) => (c.textContent ?? '').trim() && getComputedStyle(c).display !== 'none').pop()!;
    // A closed disclosure shows only its summary.
    const closed = last.matches('details:not([open])') ? last : last.querySelector('details:not([open])');
    if (closed) last = closed.querySelector('summary')!;
    // The last line's own text box (its padding may sit under the cap; the words may not).
    const range = document.createRange();
    range.selectNodeContents(last);
    return { room: box.bottom - range.getBoundingClientRect().bottom, text: ((last as HTMLElement).innerText ?? '').slice(-60) };
  });
  expect(m.room, `forecast card cut off by ${m.room}px; last line "${m.text}"`).toBeGreaterThanOrEqual(0);
}

/** Wait until the map has stopped resizing (panels and the camera glide have settled). */
export async function mapSettled(page: Page) {
  let last = 0;
  await expect.poll(async () => {
    const h = (await page.locator('[data-game-map]').boundingBox())!.height;
    const still = h === last; last = h;
    return still;
  }, { intervals: [250] }).toBe(true);
}

export async function closeReportIfOpen(page: Page) {
  const close = page.getByRole('button', { name: 'Close report', exact: true });
  if (await close.isVisible()) await close.click();
}

/** Dismiss celebration cards until the one with `text` is showing (several moments can queue for one turn). */
export async function showMoment(page: Page, text: string) {
  const card = page.locator('.moment-card', { hasText: text });
  const dismiss = page.getByRole('button', { name: 'Dismiss celebration' });
  for (let i = 0; i < 14 && !await card.first().isVisible(); i++) {
    if (!await dismiss.first().isVisible()) break;
    await dismiss.first().dispatchEvent('click');
  }
  await expect(card.first()).toBeVisible();
}

export async function dismissCelebrations(page: Page) {
  // The card can advance on its own timer between isVisible() and dispatchEvent(). In that race,
  // dispatchEvent waits for a new matching button even though the celebration queue is already empty.
  // Read and click the currently rendered control in one page task, stopping when the UI has none.
  for (let i = 0; i < 6; i++) {
    const clicked = await page.evaluate(() => {
      const button = document.querySelector<HTMLButtonElement>('.moment-lane button[aria-label="Dismiss celebration"]');
      if (!button) return false;
      button.click();
      return true;
    });
    if (!clicked) break;
  }
}


/** Frame the Courier Hall's city at a zoom: the camera jumps there, then the zoom is set and the glide settles. */
export async function frameHall(page: Page, hex: Hex, zoom: number) {
  await page.evaluate(({ hex, zoom }) => {
    const w = window as unknown as GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(null);
    s.jumpTo(hex);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { hex, zoom });
  await hexPoint(page, hex);
  await page.waitForTimeout(500);
}

/** Stage last turn's Hall events as the turn report and its moments, for the fixture's viewer. */
export async function stageHallReport(page: Page, events: Event[]) {
  const v = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; });
  const report = { turn: v.turn - 1, hash: '', events: [events], moments: [momentsForTurn(events, v.me, null, v)], alarms: [[]], lines: [formatEvents(events, contextFromViews(v, v))] };
  await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [] }), report);
}

export async function openHall(page: Page, mode: HallMode) {
  const f = hallFixture(mode);
  await loadFixture(page, coreFixture(f.fixture), 12);
  return f;
}

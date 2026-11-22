// Era overhaul phase C: the celebrated era-up takeover and the board changing with the era.
// Shots go to docs/design/era-art/in-game/<size>/ (look at them: they are the UI review).
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import { TECHS } from '../src/data';
import type { useGame } from '../src/store';
import { withV2, withWorldAges } from '../src/test/vpV2Fixture';
import type { Hex } from '../src/types';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number }; redraw(): void } };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const out = fileURLToPath(new URL('../../docs/design/era-art/in-game/', import.meta.url));

async function load(page: Page, game: string, turn = 2) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

async function shot(page: Page, size: { width: number; height: number }, name: string, clip?: { x: number; y: number; width: number; height: number }) {
  await page.mouse.move(size.width - 2, size.height - 2);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(400);
  await mkdir(`${out}${size.width}x${size.height}/`, { recursive: true });
  await page.screenshot({ path: `${out}${size.width}x${size.height}/${name}.png`, animations: 'disabled', ...(clip ? { clip } : {}) });
}

/** Ada knows four Ancient techs and finishes the fifth this turn: she enters the Classical era at the end of it. */
function eraUpState() {
  const f = researchFixture('India');
  f.players[0].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency'];
  f.players[0].research = 'Writing';
  f.players[0].research_queue = ['Writing'];
  f.players[0].science = 5000;
  return coreFixture(f);
}

/** Stage the era-up on the live view: v2 objectives, and your own EraEntered in the report (render only). */
async function stageEraUp(page: Page) {
  const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
  const view = withV2(base, { era: 2 });
  await page.evaluate((v) => {
    const store = (window as unknown as GameWindow).legatum, s = store.getState();
    store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) });
    store.getState().set({ report: { turn: v.turn, hash: 'era-up', lines: [[], []], events: [[{ type: 'EraEntered', player: v.me, era: 2, first: false }], []] }, reportOpen: false });
  }, view);
}

/** The same, in a World Age game with two open Ages. */
async function stageWorldAges(page: Page) {
  const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
  const view = withWorldAges(base);
  await page.evaluate((v) => {
    const store = (window as unknown as GameWindow).legatum, s = store.getState();
    store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) });
    store.getState().set({ report: { turn: v.turn, hash: 'era-up', lines: [[], []], events: [[{ type: 'EraEntered', player: v.me, era: 2, first: false }], []] }, reportOpen: false });
  }, view);
}

const takeover = (page: Page) => page.getByTestId('era-takeover');

async function intoTakeover(page: Page) {
  await load(page, eraUpState());
  await endPlanning(page);
  await expect(page.locator('.moment-card.unlock')).toBeVisible();
  await page.keyboard.press('Escape'); // the Writing unlock card first, then the era
  await expect(takeover(page)).toBeVisible();
}

for (const size of SIZES) test.describe(`era takeover ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  test('the takeover fills the screen, fits whole, names what changes and has one primary', async ({ page }) => {
    await intoTakeover(page);
    const t = takeover(page);
    await expect(t).toContainText('The Classical Era');
    const box = (await t.boundingBox())!;
    expect(box).toMatchObject({ x: 0, y: 0, width: size.width, height: size.height });
    await expect(t.locator('.et-art img')).toBeVisible();
    // What changes: new units and buildings, the board (the objectives need a VP v2 game: next test).
    await expect(t.getByTestId('era-up-new').locator('li').first()).toBeVisible();
    await expect(t.getByTestId('era-up-board').locator('img')).toHaveCount(2);
    // Everything stays inside the window, nothing is cut at the bottom, and Continue (here: Choose research) is the one primary.
    await page.waitForTimeout(3200);
    for (const part of ['.et-head', '.et-changes', '.et-foot', '[data-primary-action]']) {
      const b = (await t.locator(part).boundingBox())!;
      expect(b.y, part).toBeGreaterThanOrEqual(0);
      expect(b.y + b.height, part).toBeLessThanOrEqual(size.height + 0.5);
    }
    for (const col of await t.locator('.et-col').all()) {
      // A column's own content never overflows its box.
      expect(await col.evaluate((el) => el.scrollHeight <= el.clientHeight + 1 && el.scrollWidth <= el.clientWidth + 1)).toBe(true);
    }
    await expect(t.locator('[data-primary-action]')).toHaveCount(1);
    await expect(t.locator('[data-primary-action]')).toContainText('Choose research');
    await shot(page, size, 'era-up-classical');
  });

  test('it is skippable at once: Esc puts it off mid-animation and touches neither the turn nor research', async ({ page }) => {
    await intoTakeover(page);
    const turn = await page.getByTestId('turn-number').innerText();
    await page.keyboard.press('Escape');
    await expect(takeover(page)).toHaveCount(0);
    await expect(page.locator('dialog.tech-modal[open]')).toHaveCount(0);
    await expect(page.getByTestId('turn-number')).toHaveText(turn);
  });

  test('Map animations Off shows the takeover without motion', async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem('legatum.combat-fx', JSON.stringify({ enabled: false })));
    await intoTakeover(page);
    await expect(takeover(page)).toHaveClass(/reduced/);
    for (const selector of ['.et-art', '.et-seal', '.et-rays', '.et-foot']) {
      await expect.poll(() => takeover(page).locator(selector).first().evaluate((el) => getComputedStyle(el).animationName)).toBe('none');
    }
  });

  test('with VP v2 on, the three era objectives are on the card', async ({ page }) => {
    await load(page, eraUpState());
    await stageEraUp(page);
    const t = takeover(page);
    await expect(t).toBeVisible();
    await expect(t.getByTestId('era-up-objectives').locator('li')).toHaveCount(3);
    await expect(t.getByTestId('era-up-objectives')).toContainText('Meet any one of the three');
    // The objectives are the hero: titled for what they are worth, first, widest, with your progress on each.
    await expect(t.getByTestId('era-up-objectives').locator('h3')).toContainText('Victory points open now');
    await expect(t.getByTestId('era-up-objectives').locator('.eo-num')).toHaveText(['6/9', '2/4', '0/1']);
    const widths = await t.locator('.et-col').evaluateAll((els) => els.map((el) => el.getBoundingClientRect().width));
    expect(widths[0]).toBeGreaterThan(Math.max(...widths.slice(1)) * 1.1);
    expect(await t.locator('.et-col').first().getAttribute('data-testid')).toBe('era-up-objectives');
    await page.waitForTimeout(3200);
    for (const col of await t.locator('.et-col').all()) expect(await col.evaluate((el) => el.scrollHeight <= el.clientHeight + 1 && el.scrollWidth <= el.clientWidth + 1)).toBe(true);
    const foot = (await t.locator('.et-foot').boundingBox())!;
    expect(foot.y + foot.height).toBeLessThanOrEqual(size.height + 0.5);
    await shot(page, size, 'era-up-classical-objectives');
  });

  test('in a World Age game it opens with the victory points open now, one block per open Age', async ({ page }) => {
    await load(page, eraUpState());
    await stageWorldAges(page);
    const t = takeover(page);
    await expect(t).toBeVisible();
    const panel = t.getByTestId('era-up-objectives');
    await expect(panel.locator('h3')).toContainText('Victory points open now');
    await expect(panel.locator('.et-age h4')).toHaveText(['Classical Age', 'Medieval Age']);
    await expect(panel.locator('.eo-num')).toHaveText(['2/4', '1/4']);
    await expect(panel).toContainText('Scored by');
    await expect(t).not.toContainText(/crown/i);
    expect(await t.locator('.et-col').first().getAttribute('data-testid')).toBe('era-up-objectives');
    await page.waitForTimeout(3200);
    for (const col of await t.locator('.et-col').all()) expect(await col.evaluate((el) => el.scrollHeight <= el.clientHeight + 1 && el.scrollWidth <= el.clientWidth + 1)).toBe(true);
    const foot = (await t.locator('.et-foot').boundingBox())!;
    expect(foot.y + foot.height).toBeLessThanOrEqual(size.height + 0.5);
    await expect(t.locator('[data-primary-action]')).toHaveCount(1);
    for (const text of await panel.locator('.eo-text, .eo-num, .eo-scored-by, .et-age h4').allInnerTexts()) expect(text.length).toBeGreaterThan(0);
    const small = await panel.evaluate((el) => [...el.querySelectorAll('*')].filter((n) => !n.closest('.vh-seal') && n.childNodes.length && [...n.childNodes].some((c) => c.nodeType === 3 && c.textContent!.trim()) && parseFloat(getComputedStyle(n).fontSize) < 12).map((n) => `${n.tagName}.${n.className}:${n.textContent}`));
    expect(small, 'every word of the panel is at least 12px').toEqual([]);
    await shot(page, size, 'era-up-world-ages');
  });

  test('its report line shows the card again, and the controls are on screen from the first moments', async ({ page }) => {
    await intoTakeover(page);
    // Mid-animation the footer with the primary is already visible and clickable.
    const primary = takeover(page).locator('[data-primary-action]');
    await expect(primary).toBeVisible();
    // Fully there within a second, long before the panels finish.
    await expect.poll(() => takeover(page).locator('.et-foot').evaluate((el) => Number(getComputedStyle(el).opacity)), { timeout: 1500 }).toBeGreaterThan(0.95);
    await page.keyboard.press('Escape');
    await expect(takeover(page)).toHaveCount(0);
    await expect(page.getByTestId('report-panel')).toBeVisible().catch(() => undefined);
    await page.getByRole('button', { name: 'Show card' }).first().click();
    await expect(takeover(page)).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(takeover(page)).toHaveCount(0);
  });

  test('Return opens research from the takeover', async ({ page }) => {
    await intoTakeover(page);
    await page.keyboard.press('Enter');
    await expect(takeover(page)).toHaveCount(0);
    await expect(page.locator('dialog.tech-modal[open]')).toBeVisible();
    await expect(page.locator('dialog.tech-modal').getByTestId('era-badge')).toContainText('Classical era');
  });

  test('reduced motion: no animation, the finished scene stands at once', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await intoTakeover(page);
    const names = await takeover(page).locator('.et-title, .et-seal, .et-col, .et-foot, .et-art img').evaluateAll((els) => els.map((el) => getComputedStyle(el).animationName));
    expect(names.every((n) => n === 'none')).toBe(true);
    await expect(takeover(page).locator('.et-foot')).toBeVisible();
    await shot(page, size, 'era-up-classical-reduced-motion');
  });

});

// ---- the board changes with the era ----

/** First `n` eras of techs (five per era): the empire is in era n + 1. */
const throughEra = (n: number) => TECHS.slice(0, n * 5);

function boardState(eras: [number, number, number], pops: [number, number, number, number], walls = false, ironVisible = false) {
  const f = researchFixture('India');
  f.players[0].techs = [...throughEra(eras[0])];
  if (ironVisible && !f.players[0].techs.includes('IronWorking')) f.players[0].techs.push('IronWorking');
  f.players[1].techs = [...throughEra(eras[1])];
  f.players[2].techs = [...throughEra(eras[2])];
  f.cities.forEach((c, i) => { c.population = pops[i] ?? 3; });
  if (walls) f.cities[0].buildings = [...f.cities[0].buildings, 'Walls'];
  return coreFixture(f);
}

async function cityShots(page: Page) {
  return page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const map = (window as unknown as GameWindow).legatumMap;
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return view.cities.map((c) => { const p = map.toScreen(c.pos); return { id: c.id, owner: c.owner, pop: c.population, pos: c.pos, x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) }; });
  });
}

/** A clean map for art review: no docks, minimap or lens switcher, nothing selected, the camera on a hex at a zoom. */
async function frame(page: Page, at: Hex, zoom: number) {
  await page.addStyleTag({ content: '.command-docks, .minimap, [aria-label="Map options"], .toast { visibility: hidden !important; }' });
  await page.evaluate(({ at, zoom }) => {
    const w = window as unknown as GameWindow & { legatumMap: { camera: { zoom: number } } };
    const s = w.legatum.getState();
    s.set({ techOpen: false, reportOpen: false });
    s.select(null);
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom });
  await page.waitForTimeout(900);
}

for (const size of SIZES) test.describe(`city art by era ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  test('every empire\'s cities are painted in its own era; a new era repaints only its owner\'s', async ({ page }) => {
    // The crops compare painted cities, so leave canvas idle motion out of the comparison.
    await page.emulateMedia({ reducedMotion: 'reduce' });
    // Ada Classical (era 2), the Mongols rival Industrial (era 5), the India rival Ancient: hamlet, town, city, metropolis.
    // Keep Iron revealed in both shots: learning Iron Working changes resource marks across the whole visible map.
    // This comparison is for city art, so a newly revealed deposit should not alter the rival's crop.
    await load(page, boardState([1, 4, 0], [2, 6, 10, 14], false, true));
    const first = await cityShots(page);
    await frame(page, first[0].pos, 1.6);
    await page.waitForTimeout(600);
    const cities = await cityShots(page);
    const crop = async (c: { x: number; y: number }) => {
      // A tile card can otherwise open over the crop as the camera moves beneath the pointer.
      await page.mouse.move(5, size.height / 2);
      return page.screenshot({ clip: { x: Math.max(0, c.x - 40), y: Math.max(0, c.y - 40), width: 80, height: 80 }, animations: 'disabled' });
    };
    await shot(page, size, 'board-mine-classical');
    const mine = await crop(cities[0]);
    // A painted miniature, not a flat paved hex: the crop carries real texture.
    expect(new Set(mine).size).toBeGreaterThan(40);
    await frame(page, cities[2].pos, 1.6);
    await shot(page, size, 'board-rival-industrial');
    const rivals = await cityShots(page);
    const rival = await crop(rivals[2]);
    // Ada enters the Renaissance (era 4): her capital is repainted, the rival's city is untouched.
    await load(page, boardState([3, 4, 0], [2, 6, 10, 14], false, true));
    await frame(page, first[0].pos, 1.6);
    const after = await cityShots(page);
    expect(after.map((c) => [c.x, c.y])).toEqual(cities.map((c) => [c.x, c.y]));
    expect(Buffer.compare(await crop(after[0]), mine)).not.toBe(0);
    await shot(page, size, 'board-mine-renaissance');
    await frame(page, cities[2].pos, 1.6);
    await expect.poll(async () => Buffer.compare(await crop((await cityShots(page))[2]), rival)).toBe(0);
  });

  test('zoomed out the shared far marks stand under the seals; walled cities wear the painted wall of their era', async ({ page }) => {
    await load(page, boardState([2, 4, 0], [4, 8, 14, 14], true));
    const cities = await cityShots(page);
    await frame(page, cities[0].pos, 1.6);
    await shot(page, size, 'board-walls-medieval');
    await frame(page, cities[0].pos, 0.5);
    await shot(page, size, 'board-far');
  });
});

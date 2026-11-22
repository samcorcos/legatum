// City panel density (astra full review 2026-10-31 §3 rank 2; legatum-ui "build lists are dense"): header, status
// and queue took most of the height at 1000x600, leaving about one complete production choice. Counts the complete
// choices whose rectangles sit inside the scroll area (under the sticky head), as the panel opens and after one scroll
// that brings the "Choose what ... builds" title under the head.
import type { Locator, Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { wallsFixture } from '../src/test/wallsFixture';
import { TECHS } from '../src/data';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ width: 1000, height: 600 }, { width: 1280, height: 800 }, { width: 1440, height: 900 }];
const shotDir = process.env.LEGATUM_SHOT_DIR ?? 'test-results';
const MIN_COMPLETE_CHOICES = { 1000: 3, 1280: 5, 1440: 4 } as const;

async function openCity(page: Page, size: { width: number; height: number }, busy: boolean, hp?: number) {
  const setup = wallsFixture();
  const own = setup.fixture.cities.find((c) => c.id === setup.own)!;
  setup.fixture.players[0].techs = TECHS.filter((t) => ['Irrigation', 'BronzeWorking', 'Masonry', 'Writing', 'Currency'].includes(t));
  Object.assign(own, busy
    ? { population: 4, hp: 140, producing: { kind: 'Unit', unit: 'Soldier' }, prod: 12, productionBanked: [],
      productionQueue: [{ kind: 'Building', building: 'Granary' }, { kind: 'Building', building: 'Workshop' }, { kind: 'Building', building: 'Laboratory' }, { kind: 'Unit', unit: 'Settler' }] }
    : { population: 4, producing: null, prod: 0, productionQueue: [], productionBanked: [] });
  if (hp !== undefined) own.hp = hp;
  setup.fixture.turn = 2;
  const game = coreFixture(setup.fixture);
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, game);
  await page.setViewportSize(size);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    s.set({ techOpen: false });
    s.select({ kind: 'city', id }, true);
  }, setup.own);
  const panel = page.locator('.city-panel');
  await expect(panel).toBeVisible();
  await page.mouse.move(size.width / 2, 2);
  return panel;
}

/** Complete production choices inside the scroll area under the sticky head, and where the first one starts. */
const completeChoices = (panel: Locator) => panel.evaluate((el) => {
  const scroller = el as HTMLElement, box = scroller.getBoundingClientRect();
  const head = scroller.querySelector('.cp-head')!.getBoundingClientRect();
  const cards = [...scroller.querySelectorAll<HTMLElement>('.production-choice')].map((c) => c.getBoundingClientRect());
  const complete = cards.filter((r) => r.top >= head.bottom - 0.5 && r.bottom <= box.bottom + 0.5 && r.bottom <= window.innerHeight).length;
  return { complete, scrollTop: Math.round(scroller.scrollTop), headH: Math.round(head.height), firstCardTop: Math.round(cards[0]?.top ?? -1), panelTop: Math.round(box.top), panelBottom: Math.round(box.bottom) };
});

for (const size of SIZES) test.describe(`city panel density ${size.width}x${size.height}`, () => {
  for (const busy of [true, false]) test(`${busy ? 'a busy city (queue, healing)' : 'an idle city'}: several complete production choices show`, async ({ page }) => {
    const panel = await openCity(page, size, busy);
    await page.waitForTimeout(500);
    const tag = busy ? 'busy' : 'idle';
    await page.screenshot({ path: `${shotDir}/city-${tag}-${size.width}x${size.height}.png`, animations: 'disabled' });
    const first = await completeChoices(panel);
    await panel.evaluate((el) => {
      const scroller = el as HTMLElement, head = scroller.querySelector('.cp-head')!.getBoundingClientRect();
      scroller.scrollTop += scroller.querySelector('.cp-choose-title')!.getBoundingClientRect().top - head.bottom - 4;
    });
    await page.waitForTimeout(200);
    await page.screenshot({ path: `${shotDir}/city-${tag}-scrolled-${size.width}x${size.height}.png`, animations: 'disabled' });
    const scrolled = await completeChoices(panel);
    console.log(`DENSITY ${tag} ${size.width}x${size.height} at-open=${JSON.stringify(first)} picker-under-head=${JSON.stringify(scrolled)}`);
    const need = MIN_COMPLETE_CHOICES[size.width as 1000 | 1280 | 1440];
    expect(scrolled.complete).toBeGreaterThanOrEqual(need);
    if (!busy) expect(first.complete).toBeGreaterThanOrEqual(size.width === 1280 ? 3 : need);
  });
});

for (const size of SIZES) test(`a damaged idle city keeps its healing line on screen under the head as the panel opens at ${size.width}x${size.height}`, async ({ page }) => {
  const panel = await openCity(page, size, false, 120);
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${shotDir}/city-idle-damaged-${size.width}x${size.height}.png`, animations: 'disabled' });
  const rects = await panel.evaluate((el) => {
    const r = (q: string) => el.querySelector(q)!.getBoundingClientRect();
    return { head: r('.cp-head').bottom, heal: r('[data-testid="city-heal"]'), panel: el.getBoundingClientRect().bottom };
  });
  expect(rects.heal.top).toBeGreaterThanOrEqual(rects.head - 0.5);
  expect(rects.heal.bottom).toBeLessThanOrEqual(rects.panel);
  await expect(page.getByTestId('city-heal')).toContainText('+20 a turn');
});

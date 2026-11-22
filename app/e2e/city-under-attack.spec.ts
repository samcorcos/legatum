import { revealSidebar } from './sidebarAssertions';
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { siegeFixture } from '../src/test/siegeFixture';

// A city under attack is never a surprise (user, 2026-10-08, playtest Switzerland T59): three
// Spearmen stood beside Waypost Seven and the only sign was a thin red bar. This Turn now leads
// with the city, its HP, the attackers and a capture forecast; the map banner is alarmed; the turn
// report's first line says what the attack cost. At the user's laptop window and a desktop one.
type GameWindow = Window & { legatum: typeof useGame; legatumMap: { siege: number[]; toScreen(hex: Hex): { x: number; y: number }; camera: { zoom: number }; redraw(): void } };

const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }] as const;
const shots = process.env.SIEGE_SHOTS;
const SIEGE_RED = [193, 39, 27];

async function start(page: Page, opts: { fresh?: boolean; approach?: boolean } = { fresh: true }) {
  const setup = siegeFixture(opts);
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(setup.fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  return setup;
}

/** Shot into test-results/city-under-attack/<size>/ when SIEGE_SHOTS is set (for the UI review). */
async function shot(page: Page, size: string, name: string) {
  if (!shots) return;
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(300);
  await page.screenshot({ path: `test-results/city-under-attack/${size}/${name}.png`, animations: 'disabled' });
}

/** Pixels near the siege red inside a square around a hex: the alarmed banner, ring, badge and bar. */
const redAround = (page: Page, hex: Hex, radius = 90) => page.evaluate(({ hex, radius, red }) => {
  const w = window as unknown as GameWindow;
  const canvas = document.querySelector<HTMLCanvasElement>('[data-game-map] canvas, canvas[data-game-map]') ?? document.querySelector('canvas')!;
  const rect = canvas.getBoundingClientRect(), k = canvas.width / rect.width;
  const p = w.legatumMap.toScreen(hex);
  const x0 = Math.max(0, Math.round((p.x - radius) * k)), y0 = Math.max(0, Math.round((p.y - radius * 1.4) * k));
  const data = canvas.getContext('2d')!.getImageData(x0, y0, Math.round(radius * 2 * k), Math.round(radius * 2.4 * k)).data;
  let n = 0;
  for (let i = 0; i < data.length; i += 4) if (Math.abs(data[i] - red[0]) < 14 && Math.abs(data[i + 1] - red[1]) < 14 && Math.abs(data[i + 2] - red[2]) < 14) n++;
  return n;
}, { hex, radius, red: SIEGE_RED });

const cityHex = (page: Page, id: number) => page.evaluate((id) => {
  const s = (window as unknown as GameWindow).legatum.getState();
  return s.views![s.seat].cities.find((c) => c.id === id)!.pos;
}, id);

async function shownReport(page: Page) {
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect.poll(async () => {
    const dismiss = page.getByRole('button', { name: 'Dismiss celebration' });
    if (await dismiss.first().isVisible()) await dismiss.first().dispatchEvent('click');
    const open = page.getByTestId('moment-lane').getByRole('button', { name: 'Turn report', exact: true });
    if (await open.isVisible()) await open.click();
    return report.isVisible();
  }).toBe(true);
  return report;
}

for (const size of SIZES) test.describe(`city under attack ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('This Turn leads with the city, the banner is alarmed, then the report leads with the attack', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = await start(page);
    const dock = page.locator('.seatbar');
    await revealSidebar(page, 'turn-more');
    const card = dock.locator(`.siege-card[data-city="${setup.own}"]`);

    await revealSidebar(page, 'turn-more');
    // 1. This Turn: top of the dock, above everything routine.
    await expect(card).toBeVisible();
    await expect(card).toContainText('Waypost Seven is under attack');
    await expect(card.locator('.siege-num')).toHaveText('200/200 HP');
    await expect(card.locator('.siege-who')).toContainText(/3 Spearmen from .+: about \d+ HP a turn/);
    await expect(card.locator('.siege-forecast')).toHaveText(/^(Falls next turn|Holds about \d+ turns?) unless reinforced$/);
    await expect(card.getByRole('button', { name: 'Go to Waypost Seven' })).toBeVisible();
    const top = await dock.locator('.turn-body').evaluate((body) => [...body.children].find((c) => c.matches('.siege-alerts, .sidebar-turn-more, .seats, .city-list, .orders-box'))?.className);
    expect(top, 'the alert is the first thing in the dock body').toBe('siege-alerts');
    const sliced = await card.evaluate((el) => {
      const body = el.closest('.turn-body')!, a = el.getBoundingClientRect(), b = body.getBoundingClientRect();
      return { room: b.bottom - a.bottom, scrollTop: body.scrollTop };
    });
    expect(sliced.scrollTop).toBe(0);
    // 2. The turn's one primary action names it and goes there.
    const cta = dock.locator('.cta');
    await expect(cta).toHaveClass(/todo/);
    await expect(cta.locator('.cta-label')).toHaveText('City under attack');
    await expect(cta.locator('.cta-sub')).toContainText('Waypost Seven');
    expect(await page.locator('.btn.primary:visible, .act.primary:visible, .cta.todo:not(.stepped):visible, .cta.ready:not(.stepped):visible').count(), 'one primary action').toBe(1);

    // 3. The banner on the map is alarmed, at normal and far zoom.
    const pos = await cityHex(page, setup.own);
    await page.evaluate(({ hex }) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      s.set({ techOpen: false });
      s.jumpTo(hex);
    }, { hex: pos });
    await page.waitForTimeout(700);
    expect(await page.evaluate(() => (window as unknown as GameWindow).legatumMap.siege)).toEqual([setup.own]);
    expect(await redAround(page, pos), 'the banner wears the siege red').toBeGreaterThan(150);
    // With the camera elsewhere the minimap still rings the city.
    const mini = await page.getByTestId('minimap-canvas').evaluate((c: HTMLCanvasElement) => {
      const d = c.getContext('2d')!.getImageData(0, 0, c.width, c.height).data;
      let n = 0;
      for (let i = 0; i < d.length; i += 4) if (Math.abs(d[i] - 193) < 14 && Math.abs(d[i + 1] - 39) < 14 && Math.abs(d[i + 2] - 27) < 14) n++;
      return n;
    });
    expect(mini, 'the minimap rings the city under attack').toBeGreaterThan(10);
    await shot(page, size.name, '01-this-turn-and-banner');
    await page.evaluate(() => { const m = (window as unknown as GameWindow).legatumMap; m.camera.zoom = 0.5; m.redraw(); });
    await page.waitForTimeout(500);
    expect(await redAround(page, pos, 40), 'zoomed out, the seal wears it too').toBeGreaterThan(40);
    await shot(page, size.name, '02-far-zoom');
    await page.evaluate(() => { const m = (window as unknown as GameWindow).legatumMap; m.camera.zoom = 1.25; m.redraw(); });

    // 4. Go to city: selects it, centres it, and answers the decision.
    await card.getByRole('button', { name: 'Go to Waypost Seven' }).click();
    await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().selection)).toEqual({ kind: 'city', id: setup.own });
    await expect(page.getByRole('heading', { name: /Waypost Seven/ }).first()).toBeVisible();
    // The alert stays while the siege lasts.
    await expect(card).toBeVisible();

    // 5. The attack resolves: the report's first line says what it cost.
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3');
    const report = await shownReport(page);
    const first = report.locator('.report-group').first();
    await expect(first).toHaveAttribute('data-category', 'attack');
    const line = first.locator('.rline').first();
    await expect(line).toContainText(/Waypost Seven was attacked: 200 → \d+ HP by 3 Spearmen from /);
    await expect(line).toHaveAttribute('data-tone', 'bad');
    await expect(line.locator('.hp-delta.bad')).toHaveText(/^200 → \d+ HP$/);
    await expect(first.locator('.rline-show-map').first()).toBeVisible();
    await shot(page, size.name, '03-report-first-line');
    await first.locator('.rline-show-map').first().click();
    await report.getByRole('button', { name: 'Close report' }).click();

    // 6. Next turn: the card shows what was lost and the forecast from the new HP.
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
    const next = dock.locator('.siege-card').first();
    await expect(next).toBeVisible();
    await expect(next.locator('.siege-lost')).toHaveText(/^−\d+ HP last turn$/);
    await expect(next.locator('.siege-num')).toHaveText(/^\d+\/200 HP$/);
    await expect(dock.locator('.cta .cta-label')).toHaveText('City under attack');
    expect(await page.evaluate(() => (window as unknown as GameWindow).legatumMap.siege)).toEqual([setup.own]);
    await page.waitForTimeout(600);
    await shot(page, size.name, '04-after-the-attack');
  });

  test('a loaded game with no report still shows the siege, and reduced motion holds the ring still', async ({ page }) => {
    test.setTimeout(90_000);
    await page.emulateMedia({ reducedMotion: 'reduce' });
    const setup = await start(page, {});
    await revealSidebar(page, 'turn-more');
    const card = page.locator(`.seatbar .siege-card[data-city="${setup.own}"]`);
    await expect(card).toBeVisible();
    // A save has no turn report: the card says only what the city's own record says.
    await expect(card.locator('.siege-lost')).toHaveText('Took damage last turn');
    await expect(card.locator('.siege-num')).toHaveText('120/200 HP');
    await expect(card.locator('.siege-forecast')).toHaveText('Holds about 2 turns unless reinforced');
    await expect(card.locator('.siege-head')).toContainText(/Waypost Seven is under attack · .+/);
    await expect(card.locator('.siege-compact-facts')).toHaveText('120/200 HP · Took damage last turn · Falls in 2 turns');
    expect(await card.locator('.siege-compact-facts').evaluate((el) => el.scrollWidth <= el.clientWidth), 'the deciding facts fit on the compact line').toBe(true);
    const pos = await cityHex(page, setup.own);
    await page.evaluate(({ hex }) => (window as unknown as GameWindow).legatum.getState().jumpTo(hex), { hex: pos });
    await page.waitForTimeout(700);
    const a = await redAround(page, pos);
    await page.waitForTimeout(450);
    const b = await redAround(page, pos);
    expect(a, 'the alarm shows').toBeGreaterThan(150);
    expect(b, 'and does not pulse').toBe(a);
    await shot(page, size.name, '05-loaded-game-120hp');
  });

  test('Map animations Off holds the siege ring still', async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem('legatum.combat-fx', JSON.stringify({ enabled: false })));
    const setup = await start(page, {});
    const pos = await cityHex(page, setup.own);
    await page.evaluate(({ hex }) => (window as unknown as GameWindow).legatum.getState().jumpTo(hex), { hex: pos });
    await page.waitForTimeout(700);
    await expect.poll(() => redAround(page, pos), 'the alarm finishes painting').toBeGreaterThan(150);
    const a = await redAround(page, pos);
    await page.waitForTimeout(450);
    expect(await redAround(page, pos), 'the alarm stays still with Map animations Off').toBe(a);
  });

  test('an army closing on a city is an early warning in This Turn, not a siege', async ({ page }) => {
    test.setTimeout(90_000);
    const setup = await start(page, { approach: true });
    const dock = page.locator('.seatbar');
    await revealSidebar(page, 'turn-more');
    const card = dock.locator(`.siege-card[data-city="${setup.own}"]`);
    await expect(card).toBeVisible();
    await expect(card).toHaveAttribute('data-kind', 'approach');
    await expect(card).toContainText('Army closing on Waypost Seven');
    await expect(card.locator('.siege-head')).toHaveText('Army closing on Waypost Seven · 2 turns away');
    await expect(card.locator('.siege-who')).toContainText(/2 Spearmen from .+ within 3 tiles/);
    await expect(card.locator('.siege-forecast')).toContainText('Reaches it');
    // Not a siege: the banner stays calm and nothing becomes a pending decision.
    expect(await page.evaluate(() => (window as unknown as GameWindow).legatumMap.siege)).toEqual([]);
    await expect(dock.locator('.cta .cta-label')).not.toHaveText('City under attack');
    await shot(page, size.name, '06-army-approaching');
  });
});

import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Event, Hex } from '../src/types';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { turnReportFixture } from '../src/test/turnReportFixture';

// Playtest-5: a ranged unit was hit, the report flashed by, and there was no obvious way back to
// it or to where the shot came from. The report now has a Last turn control (and L), keeps recent
// turns, and a combat line centres the map with an arrow from the attacker.
type GameWindow = Window & {
  legatum: typeof useGame;
  legatumMap: { toScreen(hex: Hex): { x: number; y: number }; reportArrow: { from: Hex | null; to: Hex; alpha: number } | null };
};
const setup = turnReportFixture();
const shots = process.env.REPORT_SHOTS;

async function start(page: Page) {
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2,
      names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(setup.fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
}

/** The report, opened if the turn's celebration is holding it back. */
async function shownReport(page: Page) {
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect.poll(async () => {
    const open = page.getByTestId('moment-lane').getByRole('button', { name: 'Turn report', exact: true });
    if (await open.isVisible()) await open.click();
    return report.isVisible();
  }).toBe(true);
  return report;
}

/** Order the Ranger to fire on the enemy Soldier, end the turn, and return the turn and the shot. */
async function fireAndEndTurn(page: Page) {
  await page.evaluate(({ ranger, enemy }) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const target = view.units.find((u) => u.id === enemy)!.pos;
    s.set({ techOpen: false });
    s.editOrders((o) => [...o, { type: 'RangedAttack', unit: ranger, target }]);
  }, { ranger: setup.ranger, enemy: setup.enemy });
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  return page.evaluate((ranger) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    const shot = s.report!.events[s.seat].find((e) => e.type === 'Battle' && e.attacker === ranger) as Extract<Event, { type: 'Battle' }>;
    return { turn: s.report!.turn, from: shot.attacker_at!, to: shot.at };
  }, setup.ranger);
}

test('end a turn with combat, close the report, reopen it from Last turn, step back, and locate the attacker', async ({ page }) => {
  await start(page);
  // Before any turn resolves there is nothing to reopen: This Turn keeps its subtitle.
  await expect(page.locator('.seatbar .last-turn')).toHaveCount(0);
  const shot = await fireAndEndTurn(page);
  const ranger = await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.views![s.seat].units.find((u) => u.id === id)?.pos ?? null;
  }, setup.ranger);
  expect(shot.from).toEqual(ranger);

  let report = await shownReport(page);
  const line = report.locator('.rline', { hasText: 'Your Archer bombarded' });
  await expect(line).toHaveAttribute('data-tone', 'good');
  await expect(line).toContainText(/from 2 tiles [A-Z]+: −\d+ HP/);
  await expect(line.locator('.hp-delta.good')).toHaveText(/^−\d+ HP$/);
  await report.getByRole('button', { name: 'Close report' }).click();
  await expect(report).toBeHidden();

  // A second, quieter turn: the combat turn is now one step back.
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('4');
  if (await page.getByRole('complementary', { name: 'Turn report' }).isVisible()) {
    await page.getByRole('button', { name: 'Close report' }).click();
  }
  const lastTurn = page.locator('.seatbar .last-turn');
  await expect(lastTurn).toContainText('Last turn');
  await expect(lastTurn).toContainText('L');
  await lastTurn.click();
  report = await shownReport(page);
  await expect(report).not.toHaveAttribute('data-report-turn', String(shot.turn));
  await report.getByRole('button', { name: /^Previous turn/ }).click();
  await expect(report).toHaveAttribute('data-report-turn', String(shot.turn));
  await expect(report.locator('.report-kicker')).toHaveText('1 turn ago');
  await expect(report.locator('.chip[data-category="combat"]')).toContainText('good');

  // Click the combat line: the map centres on the target and draws an arrow from the Archer.
  await report.locator('.rline', { hasText: 'Your Archer bombarded' }).locator('.rline-text').click();
  expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().focus?.hex)).toEqual(shot.to);
  await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatumMap.reportArrow)).toMatchObject({ from: shot.from, to: shot.to });
  // The target ends up on screen, clear of the report.
  await expect.poll(() => page.evaluate((to) => {
    const p = (window as unknown as GameWindow).legatumMap.toScreen(to);
    const map = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    const r = document.querySelector('.report')!.getBoundingClientRect();
    const x = p.x + map.left, y = p.y + map.top;
    const onMap = x > map.left && x < map.right && y > map.top && y < map.bottom;
    const underReport = x > r.left && x < r.right && y > r.top && y < r.bottom;
    return onMap && !underReport;
  }, shot.to)).toBe(true);
  if (shots) await page.screenshot({ path: `${shots}/report-arrow-1440x1100.png` });
  // The arrow fades after a few seconds.
  await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatumMap.reportArrow), { timeout: 8000 }).toBeNull();
  // L closes and reopens on the latest turn.
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('l');
  await expect(report).toBeHidden();
  await page.keyboard.press('l');
  await expect(report).not.toHaveAttribute('data-report-turn', String(shot.turn));
});

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`the report and its Last turn control fit at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await start(page);
    const shot = await fireAndEndTurn(page);
    const report = await shownReport(page);
    await expect(report).toHaveAttribute('data-report-turn', String(shot.turn));
    const lastTurn = page.locator('.seatbar .last-turn');
    await expect(lastTurn).toBeVisible();
    // Neither the stepper nor the close button is covered.
    for (const sel of ['.report-step >> nth=0', '[aria-label="Close report"]']) {
      await expect.poll(() => report.locator(sel).evaluate((el) => {
        const b = el.getBoundingClientRect();
        return el.contains(document.elementFromPoint(b.left + b.width / 2, b.top + b.height / 2));
      })).toBe(true);
    }
    await report.locator('.rline', { hasText: 'Your Archer bombarded' }).locator('.rline-text').click();
    if (shots) {
      await page.waitForTimeout(700);
      await page.screenshot({ path: `${shots}/report-${size.width}x${size.height}.png` });
    }
  });
}

// The occupation lock experiment as the player meets it (UI review, rules 27): the capture forecast that warns the capturer will be
// locked, the locked unit's panel, the frozen city's panel and pending list, and the World Ages ledger opened from "See how you win".
// Run alone: `npx playwright test e2e/tour/occupation-lock`.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { CAPTURE_AT, captureOrdersFixture } from '../../src/test/captureOrdersFixture';
import { endPlanning } from '../turnActions';
import { SIZES, hexPoint, loadFixture, selectThing, shot, type GameWindow } from './helpers';

/** The capture fixture: five Spearmen around a fallen city held by one wounded garrison. */
function lockedCapture() {
  const { state, attackers } = captureOrdersFixture();
  return { game: coreFixture(state), attackers };
}

/** The turn's story arrives as a moment card, then the turn report: close both so the shots show the panels and the map. */
async function clearTurnStory(page: Page) {
  const moment = page.locator('.moment-card'), report = page.getByRole('button', { name: 'Close report' });
  for (let i = 0; i < 8; i++) {
    if (await moment.isVisible()) await page.keyboard.press('Escape');
    else if (await report.isVisible()) { await report.click(); return; }
    await page.waitForTimeout(400);
  }
}

for (const size of SIZES) test.describe(`occupation lock ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the capture forecast warns that the capturer is locked and the city frozen', async ({ page }) => {
    const { game, attackers } = lockedCapture();
    await loadFixture(page, game, 1);
    await selectThing(page, 'unit', attackers[0]);
    const at = await hexPoint(page, CAPTURE_AT);
    await page.mouse.move(at.x, at.y);
    const warning = page.getByTestId('forecast-occupation-lock');
    await expect(warning).toBeVisible();
    await expect(warning).toContainText('stays as its garrison, holding the city down until Peace with');
    await expect(warning).toContainText('the city builds and buys nothing until then');
    await shot(page, size, 'a1-capture-forecast-lock-warning');
  });

  test('a locked unit wears a padlock on the map and its tile card says why', async ({ page }) => {
    const { game } = lockedCapture();
    await loadFixture(page, game, 1);
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    await clearTurnStory(page);
    const held = await page.evaluate((at) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      return v.units.find((u) => u.owner === v.me && u.pos.q === at.q && u.pos.r === at.r)!.id;
    }, CAPTURE_AT);
    await selectThing(page, 'unit', held);
    await page.keyboard.press('Escape');
    const token = await hexPoint(page, CAPTURE_AT);
    const clip = { x: Math.max(0, token.x - 70), y: Math.max(0, token.y - 70), width: 140, height: 140 };
    await page.mouse.move(5, size.height / 2);
    await page.waitForTimeout(3500);
    const withLock = await page.screenshot({ clip });
    await shot(page, size, 'b0-locked-token-on-map');
    await page.mouse.move(token.x, token.y);
    await expect(page.getByTestId('tile-lock')).toHaveText(/^\s*Garrison of Vestige · held here until Peace with /);
    await page.mouse.move(5, size.height / 2);
    // Lifting the lock from the view must change the pixels around the token: the padlock is drawn, not implied.
    await page.evaluate((id) => {
      const g = (window as unknown as GameWindow).legatum, st = g.getState(), v = structuredClone(st.views![st.seat]);
      v.units.find((u) => u.id === id)!.occupation_lock = null;
      g.setState({ views: st.views!.map((x, i) => (i === st.seat ? v : x)) });
    }, held);
    await page.waitForTimeout(300);
    expect(withLock.equals(await page.screenshot({ clip })), 'the locked token draws a padlock').toBe(false);
    // Zoomed out, the padlock stays on the pip.
    await page.evaluate((id) => {
      const g = (window as unknown as GameWindow).legatum, st = g.getState(), v = structuredClone(st.views![st.seat]);
      v.units.find((u) => u.id === id)!.occupation_lock = (v.me + 1) as never;
      g.setState({ views: st.views!.map((x, i) => (i === st.seat ? v : x)) });
    }, held);
    await page.mouse.move(size.width / 2, size.height / 2);
    for (let i = 0; i < 8; i++) await page.mouse.wheel(0, 300);
    await page.mouse.move(5, size.height / 2);
    await page.waitForTimeout(800);
    await shot(page, size, 'b0b-locked-token-zoomed-out');
  });

  test('a locked unit looks locked, and a frozen city is never offered as a decision', async ({ page }) => {
    const { game, attackers } = lockedCapture();
    await loadFixture(page, game, 1);
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    const held = await page.evaluate((at) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      const unit = v.units.find((u) => u.owner === v.me && u.pos.q === at.q && u.pos.r === at.r)!;
      return { unit: unit.id, city: v.cities.find((c) => c.pos.q === at.q && c.pos.r === at.r)!.id, locked: unit.occupation_lock };
    }, CAPTURE_AT);
    expect(held.locked, 'the capturing unit is locked').not.toBeNull();
    // The turn's story (the city taken moment, then the report) is read first; close it so the shots show the panels.
    await clearTurnStory(page);

    // The unit card: the garrison line says it once, with the reason under it; the unit fortifies by itself and the card offers no Move, Disband, Fortify or Skip.
    await selectThing(page, 'unit', held.unit);
    const card = page.locator('.unit-panel');
    await expect(card.getByTestId('occupation-lock')).toHaveText(/^\s*Garrisoning Vestige: holding the city down until Peace with /);
    await expect(card.getByTestId('occupation-lock-note')).toContainText('people will not accept your rule while');
    await expect(card.locator('.up-status')).toHaveText('Fortified until woken');
    for (const name of [/^Move/, /^Disband/, /^Fortify/, /^Skip/]) await expect(card.getByRole('button', { name })).toHaveCount(0);
    await expect(card).not.toContainText('Needs orders');
    // It is never in the queue, and nothing it could be asked is pending.
    expect(await page.evaluate((id) => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; const u = v.units.find((x) => x.id === id)!; return [u.standing_order, !!u.needs_orders]; }, held.unit)).toEqual(['Sleep', false]);
    // The card fits the window: nothing scrolls.
    expect(await card.evaluate((el) => el.scrollHeight - el.clientHeight), 'the unit card does not scroll').toBeLessThanOrEqual(1);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'b1-locked-unit-panel');

    // A right-click move is refused at the click, with the reason, and nothing is queued.
    // Right of the hex's centre: the city's banner (drawn above units, and taking their clicks) reaches the centre itself.
    const away = await hexPoint(page, { q: CAPTURE_AT.q + 1, r: CAPTURE_AT.r });
    const home = await hexPoint(page, CAPTURE_AT);
    await page.mouse.click(Math.round(away.x + (away.x - home.x) * 0.3), away.y, { button: 'right' });
    await expect(page.locator('.order-error, [role="alert"]').filter({ hasText: 'is holding Vestige down and cannot leave until Peace with' }).first()).toBeVisible();
    expect(await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.orders[s.seat].length; })).toBe(0);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'b2-locked-rightclick-refused');

    // The frozen city: the build list is replaced by the Occupied note, nothing to build or buy, and no "No production chosen".
    await selectThing(page, 'city', held.city);
    const panel = page.locator('.city-panel');
    await expect(panel.getByTestId('city-occupied')).toContainText('Occupied: Vestige builds and buys nothing until Peace with');
    await expect(panel.locator('.production-choice')).toHaveCount(0);
    await expect(panel.getByRole('button', { name: /^(Build|Buy|Queue)\b/ })).toHaveCount(0);
    await expect(panel).not.toContainText('add from the list below');
    await panel.getByTestId('city-occupied').scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'c1-frozen-city-panel');

    // The city card on the map says it is occupied, not that no production is chosen.
    await clearTurnStory(page);
    const city = await hexPoint(page, CAPTURE_AT);
    await page.mouse.move(city.x, city.y);
    await expect(page.getByTestId('tile-city-production')).toContainText('Occupied: Vestige builds and buys nothing until Peace with');
    await expect(page.locator('.tilecard')).not.toContainText('No production chosen');
    await shot(page, size, 'c2-frozen-city-tile-card');

    // The end-turn dialog never asks what the frozen city builds, and never complains about the locked unit.
    await page.mouse.move(5, size.height / 2);
    await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn|Ready\b)/ }).click();
    const dialog = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
    if (await dialog.isVisible()) {
      await expect(dialog).not.toContainText('Choose what Vestige builds');
      await expect(dialog).not.toContainText('cannot leave');
      await shot(page, size, 'c3-end-turn-dialog');
    }
    expect(attackers.length).toBeGreaterThan(1);
  });
});

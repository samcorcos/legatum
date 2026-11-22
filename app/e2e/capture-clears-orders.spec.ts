// Playtest 8.4, turn 31 (user): after a city fell, the units that did not move in still showed a
// "move into this city" order. They must come back as ready for orders, with no arrow left.
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { CAPTURE_AT, captureOrdersFixture } from '../src/test/captureOrdersFixture';

type GameWindow = Window & { legatum: typeof useGame };

test('units outside a just-captured city are ready for orders, without stale move arrows', async ({ page }) => {
  const { state, attackers } = captureOrdersFixture();
  await page.setViewportSize({ width: 1000, height: 600 });
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 1,
      names: ['Player 1'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(state));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  const mine = await page.evaluate((ids) => {
    const g = (window as unknown as GameWindow).legatum.getState();
    const view = g.views![g.seat];
    return view.units.filter((u) => ids.includes(u.id)).map((u) => ({ id: u.id, pos: u.pos, goto: u.goto, needs: u.needs_orders, lock: u.occupation_lock }));
  }, attackers);
  const inside = mine.filter((u) => u.pos.q === CAPTURE_AT.q && u.pos.r === CAPTURE_AT.r);
  expect(inside).toHaveLength(1);
  for (const u of mine) expect(u.goto, `unit ${u.id} still has an order`).toBeNull();
  // The soldier that took the city is its garrison under the occupation lock (rules 27+): it holds the city and needs no orders.
  expect(inside[0].lock, 'the captor is locked in the city').not.toBeNull();
  // The pending-decisions list of the end-turn confirmation counts every other soldier (an AI peace offer takes the CTA).
  await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn|Ready\b)/ }).click();
  const dialog = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
  await expect(dialog).toBeVisible();
  await expect(dialog.locator('li', { hasText: 'needs orders' })).toHaveCount(attackers.length - inside.length);
  await page.keyboard.press('Escape');
  const outside = mine.find((u) => !inside.includes(u))!;
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }, true), outside.id);
  await expect(page.locator('.unit-panel')).toContainText('Needs orders');
  await expect(page.locator('.unit-panel')).not.toContainText('Moving to');
});

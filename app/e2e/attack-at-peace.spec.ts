import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { warPromptCoreSave } from '../src/test/warPromptFixture';

// Playtest 6: an Archer ordered to fire on a city of an empire at Peace showed an order icon on the
// map, then "Fix unit order: no enemy unit or city to attack there". Civ-style, attacking a team at
// Peace now asks "Declare War on …?" first: Return declares and keeps the attack (the surprise war
// opens before ranged fire), Esc drops it. Clicking a foreign city opens its card with Declare War,
// diplomacy, and "Attack with" the unit that was selected. The Fix-order card's sentence wraps
// inside the card at the user's window sizes.
type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const shots = process.env.ATTACK_SHOTS;
const CITY = { q: 19, r: 8 }; // Switzerland' capital Harmonia, fromOffset(23, 8); the Archer stands 2 tiles west.

async function start(page: Page) {
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 1, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, warPromptCoreSave(false, true));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  return page.evaluate((city) => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    const u = v.units.find((u) => u.owner === v.me && u.kind === 'Archer')!;
    s.set({ techOpen: false });
    s.select({ kind: 'unit', id: u.id }); s.jumpTo(city);
    return u.id;
  }, CITY);
}

async function mapPoint(page: Page, hex: Hex) {
  // jumpTo animates the camera for 380 ms. Two early reads can agree before its first frame.
  await page.waitForTimeout(400);
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))));
  let last = { x: NaN, y: NaN };
  await expect.poll(async () => {
    const p = await page.evaluate((hex) => {
      const p = (window as unknown as GameWindow).legatumMap.toScreen(hex);
      const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
      return { x: p.x + b.left, y: p.y + b.top };
    }, hex);
    const still = Math.hypot(p.x - last.x, p.y - last.y) < 0.5;
    last = p;
    return still;
  }).toBe(true);
  return last;
}

const orders = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  return s.orders[s.seat];
});
const errors = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  return s.errors[s.seat];
});
const settle = (page: Page) => page.evaluate(() => Promise.all(document.getAnimations()
  .filter((a) => a.effect?.getTiming().iterations !== Infinity).map((a) => a.finished.catch(() => null))));

async function aimAtCity(page: Page) {
  await page.getByRole('button', { name: /^Ranged Attack/ }).click();
  const city = await mapPoint(page, CITY);
  await page.mouse.click(city.x, city.y, { button: 'right' });
}

test('ranged attack on a city at Peace asks to declare War; Esc drops it, Return declares and fires this turn', async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 600 });
  const archer = await start(page);
  await aimAtCity(page);
  const dialog = page.getByRole('dialog', { name: 'Declare War on Switzerland?' });
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText("can't fire on their Harmonia yet");
  await expect(dialog).toContainText('War starts now.');
  await expect(dialog).toContainText('fires on their Harmonia this turn');
  await expect(dialog).toContainText('Costs no Gold.');
  await expect(dialog.locator('.btn.primary')).toHaveAccessibleName('Declare War and attack');
  // Nothing looks accepted while the decision is open.
  expect(await orders(page)).toEqual([]);
  if (shots) { await settle(page); await page.screenshot({ path: `${shots}/prompt-ranged-1000x600.png` }); }
  await page.keyboard.press('Escape');
  await expect(dialog).toHaveCount(0);
  expect(await orders(page)).toEqual([]);

  await aimAtCity(page);
  await expect(dialog).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(dialog).toHaveCount(0);
  await expect.poll(() => orders(page)).toEqual([
    { type: 'DeclareWar', against: 1, breach: false },
    { type: 'RangedAttack', unit: archer, target: CITY },
  ]);
  expect(await errors(page)).toEqual([]);
  await expect(page.locator('.decision-notice')).toContainText('War declared on Switzerland; the attack goes in this turn');
  if (shots) await page.screenshot({ path: `${shots}/declared-1000x600.png` });
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  const events = await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.report?.events[s.seat] ?? [];
  });
  expect(events.some((e) => e.type === 'Battle' && e.attacker === archer && e.defender.kind === 'City')).toBe(true);
});

test('an empty tile is refused with a reason and nothing is queued', async ({ page }) => {
  await start(page);
  await page.getByRole('button', { name: /^Ranged Attack/ }).click();
  const empty = await mapPoint(page, { q: CITY.q - 1, r: CITY.r + 1 });
  await page.mouse.click(empty.x, empty.y, { button: 'right' });
  await expect(page.locator('.toast.bad')).toContainText('Nothing to fire on there');
  expect(await orders(page)).toEqual([]);
});

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`Fix order card wraps inside the dock at ${size.width}x${size.height}; fixing it asks about War, Esc removes the order`, async ({ page }) => {
    await page.setViewportSize(size);
    const archer = await start(page);
    // A stale order (e.g. queued before a treaty, or from an older build) that can't resolve.
    await page.evaluate(({ archer, city }) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      s.editOrders((o) => [...o, { type: 'RangedAttack', unit: archer, target: city }]);
      s.select({ kind: 'unit', id: archer });
    }, { archer, city: CITY });
    const cta = page.locator('.seatbar .cta');
    await expect(cta.locator('.cta-label')).toHaveText('Fix order');
    await expect(cta).toContainText(/Archer #\d+: you are not at War with its owner; declare War first to attack there\./);
    await expect(cta).toHaveClass(/cta-long/);
    // Sentence case body text, inside the notched button, clear of the key hint.
    const fit = await cta.evaluate((button) => {
      const box = button.getBoundingClientRect();
      const sub = button.querySelector('.cta-sub')!;
      const kbd = button.querySelector('.kbd');
      const inside = (r: DOMRect) => r.left >= box.left - 0.5 && r.right <= box.right + 0.5 && r.top >= box.top - 0.5 && r.bottom <= box.bottom + 0.5;
      const range = document.createRange(); range.selectNodeContents(sub);
      const lines = [...range.getClientRects()];
      const style = getComputedStyle(sub);
      return { text: lines.every(inside), kbd: !kbd || inside(kbd.getBoundingClientRect()), transform: style.textTransform, size: parseFloat(style.fontSize), spacing: style.letterSpacing };
    });
    expect(fit).toMatchObject({ text: true, kbd: true, transform: 'none' });
    expect(fit.size).toBeGreaterThanOrEqual(13);
    expect(['normal', '0px']).toContain(fit.spacing);
    if (shots) await page.locator('.seatbar').screenshot({ path: `${shots}/fix-card-${size.width}x${size.height}.png` });
    if (shots) await page.screenshot({ path: `${shots}/fix-card-full-${size.width}x${size.height}.png` });
    await cta.click();
    const dialog = page.getByRole('dialog', { name: 'Declare War on Switzerland?' });
    await expect(dialog).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(dialog).toHaveCount(0);
    await expect.poll(() => orders(page)).toEqual([]);
    await expect(cta).not.toContainText('Fix order');
  });
}

test('clicking a foreign city opens its card: Declare War asks first, Attack with the selected Ranger', async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 600 });
  const archer = await start(page);
  // With the Archer selected, a left-click on the city opens the city's card (not an order).
  const city = await mapPoint(page, CITY);
  await page.mouse.click(city.x, city.y);
  const card = page.getByTestId('foreign-city-actions');
  await expect(card).toBeVisible();
  await expect(page.locator('.city-panel .cp-name')).toHaveText('Harmonia');
  await expect(card).toContainText('At Peace with Switzerland');
  await expect(card.getByRole('button', { name: 'Diplomacy with Switzerland' })).toBeVisible();
  await expect(card.getByRole('button', { name: 'Declare War…' })).toBeVisible();
  const attack = card.getByRole('button', { name: /^Attack with Archer…$/ });
  await expect(attack).toHaveClass(/primary/);
  await expect(card.locator('.btn.primary')).toHaveCount(1);
  if (shots) { await settle(page); await page.screenshot({ path: `${shots}/city-card-1000x600.png` }); }
  // Declare War alone: the same confirmation.
  await card.getByRole('button', { name: 'Declare War…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Declare War on Switzerland?' });
  await expect(dialog).toBeVisible();
  await expect(dialog.locator('.btn.primary')).toHaveAccessibleName('Declare War');
  if (shots) { await settle(page); await page.screenshot({ path: `${shots}/prompt-city-1000x600.png` }); }
  await page.keyboard.press('Escape');
  expect(await orders(page)).toEqual([]);
  // Attack with the Archer: confirm with Return.
  await attack.click();
  await expect(dialog).toBeVisible();
  await expect(dialog.locator('.btn.primary')).toHaveAccessibleName('Declare War and attack');
  await page.keyboard.press('Enter');
  await expect.poll(() => orders(page)).toEqual([
    { type: 'DeclareWar', against: 1, breach: false },
    { type: 'RangedAttack', unit: archer, target: CITY },
  ]);
  // Reopened, the card now says War is declared and attacks without asking.
  await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    s.select({ kind: 'city', id: v.cities.find((c) => c.name === 'Harmonia')!.id });
  });
  await expect(card).toContainText('War declared on Switzerland this turn; it starts this turn.');
  await expect(card.getByRole('button', { name: 'Declare War…' })).toHaveCount(0);
  // Diplomacy opens on that empire.
  await card.getByRole('button', { name: 'Diplomacy with Switzerland' }).click();
  await expect(page.locator('.diplomacy-panel .empire-row.open')).toContainText('Switzerland');
  await expect(page.locator('.diplomacy-panel .empire-row.open')).toBeInViewport();
});

test('1440x900: the War prompt and city card', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await start(page);
  const city = await mapPoint(page, CITY);
  await page.mouse.click(city.x, city.y);
  await expect(page.getByTestId('foreign-city-actions')).toBeVisible();
  if (shots) { await settle(page); await page.screenshot({ path: `${shots}/city-card-1440x900.png` }); }
  await page.getByTestId('foreign-city-actions').getByRole('button', { name: /^Attack with/ }).click();
  await expect(page.getByRole('dialog', { name: 'Declare War on Switzerland?' })).toBeVisible();
  if (shots) { await settle(page); await page.screenshot({ path: `${shots}/prompt-ranged-1440x900.png` }); }
});

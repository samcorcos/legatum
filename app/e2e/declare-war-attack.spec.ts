import { revealSidebar } from './sidebarAssertions';
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { acknowledgeWarDeclaredByYou, endPlanning } from './turnActions';
import { PURSUIT, WAR_PROMPT, warPromptCoreSave } from '../src/test/warPromptFixture';

// Playtest G2: ordering a Spearman onto a unit of a team at Peace used to be a silent bounce.
// Now the map says so on hover and a click asks to declare War; confirming queues the
// declaration and the attack, which follows the unit if it moves. G20: with no treaty the War
// starts at once and the attack resolves the same turn. G18: it is one attack, then the
// Spearman needs orders.
type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const shots = process.env.G2_SHOTS;

async function start(page: Page, war = false, ranged = false) {
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 1, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, warPromptCoreSave(war, ranged));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  const soldier = await page.evaluate((at) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const u = view.units.find((u) => u.owner === view.me && u.pos.q === at.q && u.pos.r === at.r)!;
    s.set({ techOpen: false });
    s.select({ kind: 'unit', id: u.id }); s.jumpTo(u.pos);
    return u.id;
  }, (war ? PURSUIT : WAR_PROMPT).soldier);
  return soldier;
}

test('G20 ranged attack can target a city after the Soldier declares surprise War in the same turn', async ({ page }) => {
  await start(page, false, true);
  const scout = await mapPoint(page, WAR_PROMPT.scout);
  await page.mouse.click(scout.x, scout.y, { button: 'right' });
  await page.getByRole('dialog', { name: 'Declare War on Switzerland?' })
    .getByRole('button', { name: 'Declare War and attack' }).click();
  const city = { q: 19, r: 8 }; // fromOffset(23, 8)
  const ranger = await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    const u = v.units.find((u) => u.owner === v.me && u.kind === 'Archer')!;
    s.select({ kind: 'unit', id: u.id }); s.jumpTo(u.pos);
    return u.id;
  });
  await page.getByRole('button', { name: /^Ranged Attack/ }).click();
  const target = await mapPoint(page, city);
  await page.mouse.click(target.x, target.y, { button: 'right' });
  await expect.poll(async () => (await orders(page)).some((o) => o.type === 'RangedAttack' && o.unit === ranger)).toBe(true);
  await revealSidebar(page, 'turn-more');
  await expect(page.getByRole('button', { name: /^Pending orders/ })).not.toContainText('issue');
  await expect(page.locator('.seatbar .cta')).not.toContainText('unit needs orders');
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  const result = await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.report?.events[s.seat] ?? [];
  });
  expect(result.some((e) => e.type === 'Battle' && e.attacker === ranger && e.defender.kind === 'City')).toBe(true);
});

async function mapPoint(page: Page, hex: Hex) {
  // jumpTo animates the camera; wait until the point settles.
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

async function afterConfirm(page: Page, soldier: number) {
  expect(await orders(page)).toEqual([
    { type: 'DeclareWar', against: 1, breach: false },
    { type: 'MoveUnit', unit: soldier, to: WAR_PROMPT.scout },
  ]);
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await acknowledgeWarDeclaredByYou(page);
  // The report says what the Spearman is doing instead of a silent bounce.
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect.poll(async () => {
    // A moment card holds the report back; L opens it (the card has no Turn report button).
    if (await page.getByTestId('moment-lane').locator('.moment-card').isVisible()) await page.keyboard.press('l');
    return report.isVisible();
  }).toBe(true);
  // War started this turn and the attack resolved now: the Scout was hit (standing or pulling
  // back), or it slipped out of reach or sight. The report says which.
  await expect(report).toContainText('War: you and Switzerland · started this turn');
  await expect(report).toContainText(/Your Spearman (attacked Switzerland's Scout|hit the retreating Scout)|Switzerland's Scout slipped away; your Spearman (followed but couldn't reach it|lost sight of it)/);
  const aim = await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const u = view.units.find((u) => u.id === id)!;
    return { goto: u.goto, attack: u.attack_target ?? null };
  }, soldier);
  // One attack, this turn only: nothing is left standing for next turn.
  expect(aim).toEqual({ goto: null, attack: null });
  if (shots) await page.screenshot({ path: `${shots}/report-${page.viewportSize()!.width}.png` });
}

test('G2 desktop: hover says At Peace; right-click asks to declare War; confirm queues War and the attack', async ({ page }) => {
  const soldier = await start(page);
  const at = await mapPoint(page, WAR_PROMPT.scout);
  await page.mouse.move(at.x, at.y);
  const hint = page.locator('.peace-hint');
  await expect(hint).toContainText("At Peace with Switzerland. Declare War to attack.");
  await expect(page.locator('.map-canvas')).toHaveClass(/is-peaceful/);
  if (shots) await page.screenshot({ path: `${shots}/hover-desktop.png` });
  // Cancel does nothing.
  await page.mouse.click(at.x, at.y, { button: 'right' });
  const dialog = page.getByRole('dialog', { name: 'Declare War on Switzerland?' });
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('Costs no Gold.');
  // The target's leader sits beside the decision, under a red ribbon.
  await expect(dialog.locator('.war-prompt-hero .pf > img')).toHaveCount(1);
  await expect(dialog.locator('.war-prompt-hero .rb')).toHaveText('Switzerland');
  await expect(dialog).toContainText('War starts now. Your Spearman attacks this turn.');
  await expect(dialog).toContainText('pursues it');
  await expect(dialog.locator('.btn.primary')).toHaveCount(1);
  await expect(dialog.locator('.btn.primary')).toHaveAccessibleName('Declare War and attack');
  // G16: a surprise war at Peace spends nothing, so its button carries no cost chip.
  await expect(dialog.locator('.btn.primary .cost-chip')).toHaveCount(0);
  if (shots) { await dialog.evaluate((el) => Promise.all(el.getAnimations().map((a) => a.finished))); await page.screenshot({ path: `${shots}/dialog-desktop.png` }); }
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(dialog).toHaveCount(0);
  expect(await orders(page)).toEqual([]);
  // G19: D cancels too (declaring has no key).
  await page.mouse.click(at.x, at.y, { button: 'right' });
  await expect(dialog).toBeVisible();
  await page.keyboard.press('a');
  await expect(dialog).toBeVisible();
  await page.keyboard.press('d');
  await expect(dialog).toHaveCount(0);
  expect(await orders(page)).toEqual([]);
  // Confirm.
  await page.mouse.click(at.x, at.y, { button: 'right' });
  await dialog.getByRole('button', { name: 'Declare War and attack', exact: true }).click();
  // G17: the decision is confirmed over the map, with Undo.
  await expect(page.locator('.decision-notice')).toContainText('War declared on Switzerland');
  await afterConfirm(page, soldier);
});

test('G2 at War: an attack on a unit follows it, and the report says what happened', async ({ page }) => {
  const soldier = await start(page, true);
  const at = await mapPoint(page, PURSUIT.scout);
  await page.mouse.move(at.x, at.y);
  await expect(page.locator('.forecast-pursuit')).toContainText('One attack this turn, then your Spearman stays where it fought and needs orders.');
  await expect(page.locator('.forecast-pursuit')).toContainText('moves this turn, your Spearman pursues it');
  if (shots) await page.screenshot({ path: `${shots}/forecast-war.png` });
  await page.mouse.click(at.x, at.y, { button: 'right' });
  await expect(page.getByRole('dialog')).toHaveCount(0);
  const queued = await orders(page);
  expect(queued).toHaveLength(1);
  expect(queued[0]).toMatchObject({ unit: soldier, to: PURSUIT.scout });
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect.poll(async () => {
    // A moment card holds the report back; L opens it (the card has no Turn report button).
    if (await page.getByTestId('moment-lane').locator('.moment-card').isVisible()) await page.keyboard.press('l');
    return report.isVisible();
  }).toBe(true);
  // Either the Scout was hit (standing or pulling back), or it slipped away and the report says so.
  await expect(report).toContainText(/Your Spearman (attacked Switzerland's Scout|hit the retreating Scout)|Switzerland's Scout slipped away; your Spearman (followed but couldn't reach it|lost sight)/);
  const aim = await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const u = view.units.find((u) => u.id === id);
    return { goto: u?.goto ?? null, attack: u?.attack_target ?? null };
  }, soldier);
  // G18: the attack was one attack, this turn only.
  expect(aim).toEqual({ goto: null, attack: null });
  if (shots) await page.screenshot({ path: `${shots}/report-war.png` });
});

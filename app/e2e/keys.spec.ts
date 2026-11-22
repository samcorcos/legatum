import { revealSidebar } from './sidebarAssertions';
import { endPlanning } from './turnActions';
import { chooseCapitalProduction } from './opening';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';

// The user's report: "sometimes when I press the arrows the map doesn't move, and sometimes when I
// press Return the turn doesn't end." Clicked buttons kept focus and the turn report blocked keys.

async function startSolo(page: Page) {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
}

const cameraX = (page: Page) => page.evaluate(() => (window as unknown as { legatumMap: { camera: { x: number } } }).legatumMap.camera.x);

/** The camera is clamped to explored land, so try both directions. */
async function pans(page: Page) {
  for (const key of ['ArrowRight', 'ArrowLeft']) {
    const before = await cameraX(page);
    await page.keyboard.down(key);
    await page.waitForTimeout(250);
    await page.keyboard.up(key);
    if ((await cameraX(page)) !== before) return true;
  }
  return false;
}

test('arrows pan and Return selects decisions after mouse clicks, with the turn report open, and after research', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await startSolo(page);
  await page.getByRole('button', { name: /^Dismiss hint/ }).click();
  await chooseCapitalProduction(page);
  expect(await pans(page), 'arrows pan after choosing capital production').toBe(true);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');

  // Open the routine first-turn report from the menu; it must not block panning or Enter.
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await page.getByRole('button', { name: 'Last turn report (L)', exact: true }).click();
  await expect(page.getByRole('complementary', { name: 'Turn report' })).toBeVisible();
  expect(await pans(page), 'arrows pan with the turn report open').toBe(true);

  // Change production with the mouse; arrows still pan.
  await revealSidebar(page, 'turn-more');
  await page.locator('.city-list > summary').click();
  await revealSidebar(page, 'turn-more');
  await page.locator('.city-list-items button').first().click();
  await page.locator('.city-panel .production-choice:not(.unavailable) button.prod').nth(1).click();
  // Build advances to the next decision, which may open research and pause map keys.
  if (await page.locator('dialog.tech-modal').isVisible()) await page.keyboard.press('Escape');
  expect(await pans(page), 'arrows pan after clicking a production option').toBe(true);

  // Click a tech, close research with Escape: Return cycles instead of re-opening research.
  await page.locator('.topbar .research-chip').click();
  await page.locator('.era-card.available').first().getByTestId('research-now').click();
  await page.keyboard.press('Escape');
  await expect(page.locator('dialog.tech-modal')).toHaveCount(0);
  // Escape closes the report first, then Return selects the next decision.
  if (await page.getByRole('complementary', { name: 'Turn report' }).count()) {
    await page.keyboard.press('Escape');
    await expect(page.getByRole('complementary', { name: 'Turn report' })).toHaveCount(0);
  }
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  // Leave the next research decision pending so Shift+Enter reviews it.
  await page.locator('.topbar .research-chip').click();
  await page.locator('.research-queue li').first().getByRole('button', { name: /^Remove .* from the queue$/ }).click();
  await page.getByRole('button', { name: 'Close research', exact: true }).click();
  await page.keyboard.press('Shift+Enter');
  const confirmation = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
  await expect(confirmation).toBeVisible();
  await confirmation.getByRole('button', { name: 'End turn anyway', exact: true }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  await expect(page.locator('dialog.tech-modal')).toHaveCount(0);
});

// Playtest 3 (G15): "when I press return [with] the action modal or the 'what happened' screen [open]
// ... I kind of want those to close so that I can see the whole map."
test('Return first closes what covers the map, then selects the next decision (G15)', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await startSolo(page);
  await page.getByRole('button', { name: /^Dismiss hint/ }).click();
  await chooseCapitalProduction(page);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await page.getByRole('button', { name: 'Last turn report (L)', exact: true }).click();
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect(report).toBeVisible();
  const selection = () => page.evaluate(() => JSON.stringify((window as unknown as { legatum: { getState(): { selection: unknown } } }).legatum.getState().selection));
  const before = await selection();

  // The first Return closes the report and does nothing else: no decision, no end of turn.
  await page.keyboard.press('Enter');
  await expect(report).toHaveCount(0);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  expect(await selection()).toBe(before);
  await expect(page.locator('dialog.tech-modal')).toHaveCount(0);

  // The next Return is the primary call to action again: it goes to the next decision.
  await page.keyboard.press('Enter');
  await expect.poll(async () => (await selection()) !== before || await page.locator('dialog.tech-modal').isVisible()).toBe(true);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  if (await page.locator('dialog.tech-modal').isVisible()) {
    // Research is a surface over the map too: Return closes it.
    await page.keyboard.press('Enter');
    await expect(page.locator('dialog.tech-modal')).toHaveCount(0);
  }

  // The diplomacy inbox and the objective sheet close on Return as well.
  await revealSidebar(page, 'standings');
  await page.getByRole('button', { name: /^Diplomacy inbox/ }).click();
  await expect(page.getByRole('complementary', { name: 'Diplomacy inbox' })).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('complementary', { name: 'Diplomacy inbox' })).toHaveCount(0);
  await page.getByRole('button', { name: 'All objectives', exact: true }).click();
  await expect(page.getByTestId('objective-sheet')).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('objective-sheet')).toHaveCount(0);
  // The top bar's VP chip opens the same sheet, and Escape closes it.
  await page.getByRole('button', { name: /^Every way to score/ }).click();
  await expect(page.getByTestId('objective-sheet')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByTestId('objective-sheet')).toHaveCount(0);
  await expect(page.getByTestId('turn-number')).toHaveText('2');

  // With nothing covering the map, Shift+Return still reviews End Turn.
  await page.keyboard.press('Shift+Enter');
  await expect(page.getByRole('dialog', { name: 'End turn with decisions pending?' })).toBeVisible();
});

// The user's report (2026-11-11): "i might click on a unit, then right click to tell it to move, but right as i
// click, the screen re-centers on the unit, which causes my move location click to be the wrong location."
test('the map does not shift under a mouse that is already moving to the next click', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await startSolo(page);
  await page.getByRole('button', { name: /^Dismiss hint/ }).click();
  type W = { legatum: { getState(): { seat: number; views: { me: number; units: { owner: number; pos: { q: number; r: number } }[] }[]; select(s: null): void } };
    legatumMap: { camera: { x: number; y: number; zoom: number }; toScreen(h: { q: number; r: number }): { x: number; y: number }; redraw(): void } };
  // Put the unit at the left edge of the map, where the opening unit panel crowds it and the layout nudge wants to move the camera.
  const unitAtEdge = () => page.evaluate(() => {
    const w = window as unknown as W, s = w.legatum.getState();
    s.select(null);
    const v = s.views[s.seat], at = v.units.find((u) => u.owner === v.me)!.pos;
    const cam = w.legatumMap.camera, p = w.legatumMap.toScreen(at);
    cam.x += (p.x - 40) / cam.zoom;
    w.legatumMap.redraw();
    return { ...w.legatumMap.toScreen(at), cam: { x: cam.x, y: cam.y } };
  });
  const camera = () => page.evaluate(() => { const c = (window as unknown as W).legatumMap.camera; return { x: c.x, y: c.y }; });
  const box = (await page.locator('[data-game-map]').boundingBox())!;

  // A still mouse: the nudge reveals the unit as before.
  let at = await unitAtEdge();
  await page.mouse.click(box.x + at.x, box.y + at.y);
  await page.waitForTimeout(700);
  expect(await camera(), 'a still mouse lets the layout nudge run').not.toEqual(at.cam);

  // A mouse on its way to the destination: the camera stays put.
  await page.keyboard.press('Escape');
  at = await unitAtEdge();
  await page.mouse.click(box.x + at.x, box.y + at.y);
  await page.mouse.move(box.x + at.x + 260, box.y + at.y + 40, { steps: 6 });
  await page.waitForTimeout(700);
  expect(await camera(), 'the camera holds while the player aims').toEqual(at.cam);
});

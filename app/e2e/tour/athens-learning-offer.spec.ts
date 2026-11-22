// UI review of claude/project-thread-u7x359: the dock offers Athens' Share the Learning only when its VP would reach the
// victory target, never on a fresh turn 1; the sidebar goal button stays. Run alone:
// `npx playwright test e2e/tour/athens-learning-offer`. Shots land in test-results/screen-tour/<size>/athens-learning-offer/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { broadcastState } from '../../src/test/goalActionsFixture';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`athens learning offer ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('a fresh Athenian game: turn 1 leads with the ordinary turn, the goal waits in the sidebar', async ({ page }) => {
    test.setTimeout(120_000);
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await stage.locator('[role=option][data-faction="Athens"]').click();
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    const cta = page.locator('.right-sidebar .cta');
    await expect(cta).not.toContainText('Share the Learning');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-fresh-athens-turn1-dock');
    const row = page.locator('[data-goal-action="Broadcast"]');
    if (await row.count()) {
      await row.scrollIntoViewIfNeeded();
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, '02-fresh-athens-turn1-goal-row');
    }
    console.log(`[${size.name}] turn-1 CTA: ${(await cta.innerText()).replace(/\s+/g, ' ')}`);
    console.log(`[${size.name}] goal row: ${(await row.count()) ? (await row.innerText()).replace(/\s+/g, ' ') : '(none)'}`);
  });

  for (const short of [3, 2] as const) test(`Athens ${short} VP short: the dock ${short === 2 ? 'offers' : 'does not offer'} Share the Learning`, async ({ page }) => {
    const state = broadcastState();
    await loadFixture(page, coreFixture(state), state.turn);
    await page.evaluate((short) => {
      const g = (window as unknown as GameWindow).legatum;
      const s = g.getState();
      const views = s.views!.map((v) => ({ ...v, standings: v.standings.map((t) => t.team === v.team ? { ...t, vp: t.target - short } : t) }));
      g.setState({ views });
    }, short);
    const cta = page.locator('.right-sidebar .cta');
    if (short === 2) {
      await expect(cta).toContainText('Share the Learning');
      await expect(cta).not.toHaveClass(/stepped/);
    } else await expect(cta).not.toContainText('Share the Learning');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, `0${short === 2 ? 4 : 3}-athens-${short}-short-dock`);
    console.log(`[${size.name}] ${short} short CTA: ${(await cta.innerText()).replace(/\s+/g, ' ')}`);
    const row = page.locator('[data-goal-action="Broadcast"]');
    await expect(row.locator('.goal-action-btn')).toBeEnabled();
    await row.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, `0${short === 2 ? 4 : 3}b-athens-${short}-short-goal-row`);
    if (short === 2) {
      await cta.click();
      await expect(page.locator('dialog[data-goal-dialog="Broadcast"][open]')).toBeVisible();
      await shot(page, size, '05-athens-near-victory-confirm');
    }
  });
});

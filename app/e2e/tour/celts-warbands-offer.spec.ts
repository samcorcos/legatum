// UI review of claude/project-thread-u7x359: the dock offers the Celts' Raise the Warbands only when its VP would bring
// the team within 1 VP of victory, never on a fresh turn 1; the sidebar goal button stays. Run alone:
// `npx playwright test e2e/tour/celts-warbands-offer`. Shots land in test-results/screen-tour/<size>/celts-warbands-offer/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { wildState } from '../../src/test/goalActionsFixture';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`celts warbands offer ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('a fresh Celtic game: turn 1 leads with the ordinary turn, the goal waits in the sidebar', async ({ page }) => {
    test.setTimeout(120_000);
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await stage.locator('[role=option][data-faction="Celts"]').click();
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    const cta = page.locator('.right-sidebar .cta');
    await expect(cta).not.toContainText('Raise the Warbands');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-fresh-celts-turn1-dock');
    const row = page.locator('[data-goal-action="CallTheWild"]');
    if (await row.count()) {
      await row.scrollIntoViewIfNeeded();
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, '02-fresh-celts-turn1-goal-row');
    }
    console.log(`[${size.name}] turn-1 CTA: ${(await cta.innerText()).replace(/\s+/g, ' ')}`);
    console.log(`[${size.name}] goal row: ${(await row.count()) ? (await row.innerText()).replace(/\s+/g, ' ') : '(none)'}`);
  });

  for (const short of [10, 3] as const) test(`Celts ${short} VP short: the dock ${short === 3 ? 'offers' : 'does not offer'} the warbands`, async ({ page }) => {
    const state = wildState();
    await loadFixture(page, coreFixture(state), state.turn);
    await page.evaluate((short) => {
      const g = (window as unknown as GameWindow).legatum;
      const s = g.getState();
      const views = s.views!.map((v) => ({ ...v, standings: v.standings.map((t) => t.team === v.team ? { ...t, vp: t.target - short } : t) }));
      g.setState({ views });
    }, short);
    const cta = page.locator('.right-sidebar .cta');
    if (short === 3) {
      await expect(cta).toContainText('Raise the Warbands');
      await expect(cta).not.toHaveClass(/stepped/);
    } else await expect(cta).not.toContainText('Raise the Warbands');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, `0${short === 3 ? 4 : 3}-celts-${short}-short-dock`);
    console.log(`[${size.name}] ${short} short CTA: ${(await cta.innerText()).replace(/\s+/g, ' ')}`);
    const row = page.locator('[data-goal-action="CallTheWild"]');
    await expect(row.locator('.goal-action-btn')).toBeEnabled();
    await row.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, `0${short === 3 ? 4 : 3}b-celts-${short}-short-goal-row`);
    if (short === 3) {
      await cta.click();
      await expect(page.locator('dialog[data-goal-dialog="CallTheWild"][open]')).toBeVisible();
      await shot(page, size, '05-celts-near-victory-confirm');
    }
  });
});

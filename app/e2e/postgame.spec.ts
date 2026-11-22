import { revealSidebar } from './sidebarAssertions';
// The end of a game (legatum-ui "The end of a game is an event"): the staged reveal and the post-game report,
// shot at the user's laptop (1000x600) and 1440x900, with fit assertions: the dialog sits inside the window,
// nothing under it scrolls (the report's own lists scroll inside their panels), and the primary is on screen.
// Fixtures are real end states of the user's own games with the core's recorded history
// (src/test/fixtures/postgame-*.json).
import type { Page } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { test, expect } from './fixtures';
import { postGameSave, type PostGameFixture } from '../src/test/postGameFixture';
import type { useGame } from '../src/store';

type PlayTimeWindow = Window & { legatum: typeof useGame };

const out = path.resolve('test-results/postgame');
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];

async function load(page: Page, f: PostGameFixture) {
  const { game, turn } = postGameSave(f);
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Player 1'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await expect(page.locator('dialog.gameover')).toBeVisible();
}

async function snap(page: Page, name: string, size: { width: number; height: number }) {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.mouse.move(size.width - 2, size.height - 2);
  await page.waitForTimeout(300);
  await mkdir(out, { recursive: true });
  await page.screenshot({ path: path.join(out, `${name}-${size.width}x${size.height}.png`) });
}

async function fits(page: Page, size: { width: number; height: number }) {
  const dialog = page.locator('dialog.gameover');
  const box = (await dialog.boundingBox())!;
  expect(box.y, 'dialog top inside the window').toBeGreaterThanOrEqual(0);
  expect(box.y + box.height, 'dialog bottom inside the window').toBeLessThanOrEqual(size.height + 0.5);
  expect(await dialog.evaluate((el) => el.scrollHeight - el.clientHeight), 'nothing scrolls inside the dialog').toBeLessThanOrEqual(1);
  const primary = dialog.locator('.btn.primary');
  await expect(primary).toHaveCount(1);
  const pb = (await primary.boundingBox())!;
  expect(pb.y + pb.height, 'the primary is on screen').toBeLessThanOrEqual(size.height);
}

for (const size of SIZES) {
  test.describe(`${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    for (const f of ['swiss-win', 'rival-win'] as PostGameFixture[]) {
      test(`end of game: ${f}`, async ({ page }) => {
        test.setTimeout(90_000);
        await page.clock.install();
        await load(page, f);
        const reveal = page.getByTestId('game-reveal');
        await expect(reveal).toHaveAttribute('data-settled', 'false');
        // Mid-sequence (the crest pressed on, the title engraving in): CSS animations are not on Playwright's
        // clock, so they are paused at 2.2 s for the shot, then the sequence runs on and settles.
        await page.evaluate(() => document.getAnimations().forEach((a) => { a.pause(); a.currentTime = 2_200; }));
        await page.screenshot({ path: path.join(out, `${f}-reveal-mid-${size.width}x${size.height}.png`) });
        await page.evaluate(() => document.getAnimations().forEach((a) => a.play()));
        await page.clock.runFor(6_200);
        await expect(reveal).toHaveAttribute('data-settled', 'true');
        await snap(page, `${f}-reveal`, size);
        await fits(page, size);
        await expect(page.locator('.gr-actions .btn.primary')).toHaveText('See the game report');
        await page.evaluate(() => {
          const client = (window as unknown as PlayTimeWindow).legatum.getState().client!;
          client.archive = () => JSON.stringify({ turn_times: [
            { turn: 1, ended_at: '2026-10-18T12:00:00Z', active_seconds: 180, era: 1, session_id: 'first' },
            { turn: 2, ended_at: '2026-10-18T12:02:00Z', active_seconds: 180, era: 1, session_id: 'second' },
          ] });
        });
        await page.locator('.gr-actions .btn.primary').click();
        const report = page.getByTestId('postgame-report');
        await expect(report).toBeVisible();
        await expect(page.locator('.pg-actions .btn.primary')).toHaveText('Return to title');
        await snap(page, `${f}-report-race`, size);
        await fits(page, size);
        // Point at the chart: every empire's VP on that turn.
        const chart = page.locator('.pg-race .hc svg');
        const cb = (await chart.boundingBox())!;
        await page.mouse.move(cb.x + cb.width * 0.55, cb.y + cb.height * 0.4);
        await expect(page.locator('.hc-readout')).toBeVisible();
        await page.screenshot({ path: path.join(out, `${f}-report-race-hover-${size.width}x${size.height}.png`) });
        for (const [tab, name] of [['growth', 'growth'], ['timeline', 'timeline'], ['stats', 'stats']] as const) {
          await page.locator(`[role="tab"][data-tab="${tab}"]`).click();
          if (tab === 'stats') await expect(page.locator('.pg-play-time')).toHaveText('Played: 6 min over 2 sessions · about 3.0 min a turn');
          await snap(page, `${f}-report-${name}`, size);
          await fits(page, size);
        }
        await page.locator('[role="tab"][data-tab="growth"]').click();
        await page.getByRole('radio', { name: /Military/ }).click();
        await snap(page, `${f}-report-growth-military`, size);
        // Closed to look at the map, the turn dock's primary reopens the report.
        await page.getByRole('button', { name: 'Keep looking at the map' }).click();
        await expect(page.locator('dialog.gameover')).toHaveCount(0);
        {
          const race = page.locator('.panel.vp');
          await revealSidebar(page, 'standings');
          await expect(race.locator('.vp-row').first()).toBeVisible();
          await expect(race).not.toContainText(/wins this turn|Next: .* era/i);
        }
        // The game is decided: no unit awaits orders and none can be given, and no goal reads past its target (8.9.4 review).
        {
          const hud = page.locator('.game');
          // A selected unit stays readable but awaits nothing (no panel at all is fine too).
          await expect(hud.getByText(/awaiting orders/i)).toHaveCount(0);
          await expect(hud.locator('.unit-panel .actions')).toHaveCount(0);
          const gold = await hud.innerText().then((t) => [...t.matchAll(/([\d,]+) of ([\d,]+) Gold/g)]);
          for (const m of gold) expect(Number(m[1].replace(/,/g, '')), `"${m[0]}" is capped at its target`).toBeLessThanOrEqual(Number(m[2].replace(/,/g, '')));
        }
        await expect(page.locator('.seatbar .cta')).toContainText('Game report');
        await snap(page, `${f}-map-after`, size);
        await page.locator('.seatbar .cta').click();
        await expect(report).toBeVisible();
      });
    }

    test('end of game: reduced motion stands finished at once', async ({ page }) => {
      await page.emulateMedia({ reducedMotion: 'reduce' });
      await load(page, 'swiss-win');
      await expect(page.getByTestId('game-reveal')).toHaveAttribute('data-settled', 'true');
      await expect(page.locator('.go-row b').first()).toHaveAttribute('aria-label', /^\d+ of \d+ VP$/);
    });
  });
}

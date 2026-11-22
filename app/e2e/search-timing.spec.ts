import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { chooseCapitalProduction } from './opening';

// How long End Turn takes in the real app (the WASM engine in the page, one thread) for plain
// Hard and Training, by player count. A measurement, not a gate: it runs only with
//   LEGATUM_SEARCH_TIMING=1 npx playwright test e2e/search-timing.spec.ts --project=game
// (TURNS=n sets the length, default 70). The human seat chooses capital production and ends turns;
// "turn ms" is the click on End Turn until the next turn number shows, so it includes the
// harness's own round trips (see the plain Hard row for that floor).
test.skip(process.env.LEGATUM_SEARCH_TIMING !== '1', 'measurement only');
test.use({ viewport: { width: 1000, height: 600 } });

const turns = Number(process.env.TURNS ?? 70);
const sizes = [
  { name: '1v1', seats: 2, map: /^Small/ },
  { name: '4 players', seats: 4, map: /^Small/ },
  { name: '6 players', seats: 6, map: /^Standard/ },
  { name: '8 players', seats: 8, map: /^Standard/ },
];

async function finished(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const s = (window as unknown as { legatum: { getState(): { views?: { finished?: boolean }[]; seat: number } } }).legatum.getState();
    return Boolean(s.views?.[s.seat]?.finished);
  });
}

for (const size of sizes) {
  for (const level of ['Hard', 'Training'] as const) {
    test(`${size.name}, ${level}`, async ({ page }, info) => {
      test.setTimeout(30 * 60_000);
      // First-game hint cards cover the End Turn button after a few dozen turns; they aren't what's measured.
      await page.addInitScript(() => localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] })));
      // Training is offered only by a development build with the switch on (`npm run dev`), never by `app/dist`.
      await page.goto(level === 'Training' ? '/?client=wasm&training=1' : '/?client=wasm');
      await page.getByRole('button', { name: /^Single Player/ }).click();
      await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: size.map }).click();
      for (let seat = size.seats + 1; seat <= 4; seat++) await page.getByLabel(`Seat ${seat} type`, { exact: true }).selectOption('closed');
      await page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true })
        .getByRole('radio', { name: level === 'Hard' ? /^Hard/ : /^Training/ }).click();
      await page.getByLabel('Map seed', { exact: true }).fill('7100003');
      await page.getByRole('button', { name: /^Start game/ }).click();
      await expect(page.getByTestId('turn-number')).toHaveText('1');
      await chooseCapitalProduction(page);

      const ms: number[] = [];
      for (let turn = 2; turn <= turns + 1; turn++) {
        const start = Date.now();
        try {
          const watchdog = new Promise<never>((_, reject) => setTimeout(() => reject(new Error('no new turn within 60 s')), 60_000));
          await Promise.race([
            (async () => {
              await endPlanning(page);
              await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 60_000 });
            })(),
            watchdog,
          ]);
        } catch (error) {
          // The game ended or a card is in the way: keep what was measured and show why.
          await info.attach('stalled.png', { body: await page.screenshot(), contentType: 'image/png' });
          console.log(`stalled before turn ${turn}: ${String(error).split('\n')[0]}`);
          break;
        }
        ms.push(Date.now() - start);
        if (process.env.TIMING_LOG) console.log(`turn ${turn}: ${ms[ms.length - 1]} ms`);
        if (await finished(page)) break;
        // A war declared on the human, celebrations and the like hold the turn until dismissed
        // (they mount a moment after the turn number changes).
        await page.waitForTimeout(300);
        for (let i = 0; i < 5 && (await page.locator('.moment-card, [data-celebrate-delta], dialog[open]').count() || await page.getByRole('button', { name: /^To arms/ }).count()); i++) {
          await page.keyboard.press('Escape');
          await page.waitForTimeout(100);
        }
      }
      const sorted = [...ms].sort((a, b) => a - b);
      const q = (p: number) => sorted[Math.floor((sorted.length - 1) * p)];
      const row = { players: size.name, level, turns: ms.length, meanMs: Math.round(ms.reduce((a, b) => a + b, 0) / ms.length), p50: q(0.5), p95: q(0.95), maxMs: sorted[sorted.length - 1], over1s: ms.filter((m) => m > 1000).length, over2s: ms.filter((m) => m > 2000).length };
      console.log(`SEARCH-TIMING ${JSON.stringify(row)}`);
      await info.attach('timing.json', { body: JSON.stringify({ ...row, ms }), contentType: 'application/json' });
    });
  }
}

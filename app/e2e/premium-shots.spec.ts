// Premium pass shots (docs: legatum-ui "Premium tiers"): the screens each premium increment touches,
// at the user's laptop (1000x600) and 1440x900, for before/after contact sheets. Shots only, plus
// "fits" assertions: the dialog sits inside the window and nothing under it scrolls.
// PREMIUM_SHOTS=<dir> picks where they go, PREMIUM_TAG the file prefix (default "shot").
import type { Page } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { test, expect } from './fixtures';
import { gameOverCoreSave, type GameOverKind } from '../src/test/gameOverFixture';
import { FACTIONS, FACTION_INFO } from '../src/data';

const out = process.env.PREMIUM_SHOTS ? path.resolve(process.env.PREMIUM_SHOTS) : path.resolve('test-results/premium');
const tag = process.env.PREMIUM_TAG ?? 'shot';
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];

async function loadSave(page: Page, game: string, turn: number) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

async function snap(page: Page, name: string, size: { width: number; height: number }) {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.mouse.move(size.width - 2, size.height - 2);
  await page.waitForTimeout(700);
  await mkdir(out, { recursive: true });
  await page.screenshot({ path: path.join(out, `${tag}-${name}-${size.width}.png`), animations: 'disabled' });
}

for (const size of SIZES) {
  test.describe(`${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    for (const kind of ['victory', 'team-victory', 'rival-victory', 'defeat'] as GameOverKind[]) {
      test(`game over: ${kind}`, async ({ page }) => {
        await loadSave(page, gameOverCoreSave(kind), 164);
        const dialog = page.locator('dialog.gameover');
        await expect(dialog).toBeVisible();
        await snap(page, `gameover-${kind}`, size);
        const box = (await dialog.boundingBox())!;
        expect(box.y, 'dialog top inside the window').toBeGreaterThanOrEqual(0);
        expect(box.y + box.height, 'dialog bottom inside the window').toBeLessThanOrEqual(size.height);
        expect(await dialog.evaluate((el) => el.scrollHeight - el.clientHeight), 'nothing scrolls inside the dialog').toBeLessThanOrEqual(1);
        const primary = dialog.getByRole('button').first();
        const pb = (await primary.boundingBox())!;
        expect(pb.y + pb.height).toBeLessThanOrEqual(size.height);
      });
    }

    test('lobby: every faction on a seat row fits the row and the panel', async ({ page }) => {
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Single Player/ }).click();
      const pick = async (seat: number, faction: (typeof FACTIONS)[number]) => {
        await page.getByRole('button', { name: new RegExp(`^Seat ${seat} faction:`) }).click();
        await page.getByRole('option', { name: FACTION_INFO[faction].name }).click();
        await page.getByRole('button', { name: 'Close faction picker' }).click();
      };
      // A new lobby has six seats; add the last two so all eight (the most it allows) are checked: the first eight
      // peoples fill them, then the rest take seat 1 in turn.
      for (let i = 0; i < 2; i++) await page.getByRole('button', { name: '+ Add an AI empire' }).click();
      const SEATS = 8;
      const check = () => page.evaluate(() => {
        const out: string[] = [];
        const card = document.querySelector('.seat-table')!.closest('.lobby-card')!.getBoundingClientRect();
        for (const row of document.querySelectorAll<HTMLElement>('.seat-row:not(.head)')) {
          const r = row.getBoundingClientRect();
          if (row.scrollWidth > row.clientWidth + 1) out.push(`row overflows (${row.scrollWidth} > ${row.clientWidth})`);
          if (r.right > card.right + 1) out.push('row passes the panel edge');
          const button = row.querySelector<HTMLElement>('.faction-current');
          if (!button) continue;
          const b = button.getBoundingClientRect();
          for (const el of button.querySelectorAll<HTMLElement>('.faction-current-name, .faction-current-identity')) {
            const e = el.getBoundingClientRect();
            if (e.right > b.right - 4 || e.bottom > b.bottom + 0.5 || el.scrollWidth > el.clientWidth + 1) out.push(`"${el.textContent}" is clipped in its button`);
          }
        }
        return out;
      });
      for (const [i, faction] of FACTIONS.slice(0, SEATS).entries()) await pick(i + 1, faction);
      await page.evaluate(() => document.fonts.ready.then(() => undefined));
      const issues = await check();
      for (const faction of FACTIONS.slice(SEATS)) {
        await pick(1, faction);
        issues.push(...await check());
      }
      expect(issues).toEqual([]);
      await snap(page, 'lobby-all-factions', size);
    });
  });
}

// The user's turn-203 save (8.9.2, 2026-10-17: "This view is too scrunched at this point... I can't scroll or see anything"):
// Modern era scored, Sparta's side leading, 8 empires, a city under siege. A gzipped copy of the save in
// ~/LegatumRuns/bug-saves is committed beside this file; specs load it as the autosave, so nothing here writes a save.
import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { expect } from './fixtures';

const save = () => gunzipSync(readFileSync(fileURLToPath(new URL('./saves/p892-race-panel-t203.current.legatum.gz', import.meta.url)))).toString('utf8');

/** Continue from the late-game save (turn 203). */
export async function openLateRace(page: Page) {
  await page.addInitScript((save) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', save);
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, save());
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('203', { timeout: 90_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.mouse.move(4, 300);
  await page.waitForTimeout(600);
}

/** The race panel's rows: how many sit whole in the rows box and are really in view (not under your pinned row), and the scroll state. */
export const raceRows = (page: Page) => page.getByRole('region', { name: 'Victory race', exact: true }).evaluate((el) => {
  const rows = el.querySelector<HTMLElement>('.vp-rows')!, box = rows.getBoundingClientRect();
  const all = [...rows.querySelectorAll<HTMLElement>(':scope > .vp-row')];
  const inView = (r: HTMLElement) => {
    const b = r.getBoundingClientRect();
    if (b.top < box.top - 1 || b.bottom > box.bottom + 1) return false;
    // Whatever is on top at the row's top and bottom edges is the row itself, not a pinned row over it (a row half under the pinned one fails).
    const on = (y: number) => !!document.elementFromPoint(b.left + 14, y)?.closest('.vp-row')?.isSameNode(r);
    return on(b.top + 3) && on(b.bottom - 3);
  };
  const more = el.querySelector('.vp-more');
  return {
    total: all.length,
    whole: all.filter(inView).length,
    names: all.filter(inView).map((r) => r.querySelector('.vp-team-name')?.textContent ?? ''),
    scrollTop: Math.round(rows.scrollTop), scrolls: rows.scrollHeight > rows.clientHeight + 1,
    more: more?.textContent?.trim() ?? '',
    panelBottom: Math.round(el.getBoundingClientRect().bottom),
    mineInView: all.some((r) => r.classList.contains('mine') && inView(r)),
  };
});

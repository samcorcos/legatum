import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';
import { moveCost } from '../src/data';
import { distance, tileIndex, within } from '../src/hex';
import type { Faction, Hex } from '../src/types';

const out = path.resolve('../docs/design/insignia/us-enlisted');
const phase = process.env.INSIGNIA_PHASE === 'before' ? 'before' : 'after';
const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0 } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('Vite did not bind');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

const cases: { slug: string; faction: Faction; xp: number; level: number }[] = [
  { slug: 'india-level-1', faction: 'India', xp: 16, level: 1 },
  { slug: 'india-level-2', faction: 'India', xp: 46, level: 2 },
  { slug: 'india-level-3', faction: 'India', xp: 90, level: 3 },
  { slug: 'india-level-4', faction: 'India', xp: 150, level: 4 },
  { slug: 'india-level-5', faction: 'India', xp: 226, level: 5 },
  { slug: 'sparta-level-6', faction: 'Sparta', xp: 318, level: 6 },
];

function staged(faction: Faction, xp: number) {
  const scene = mapArtFixture();
  scene.fixture.players[0].faction = faction;
  scene.fixture.setup.players[0].faction = faction;
  const veteran = scene.fixture.units.find((u) => u.owner === 0 && u.kind === 'Soldier' && u.pos.q === scene.parade[0].q && u.pos.r === scene.parade[0].r);
  if (!veteran) throw new Error('Art fixture has no own parade soldier');
  veteran.kind = 'FieldCannon';
  veteran.xp = xp;
  const home = scene.fixture.cities.find((c) => c.owner === 0)!;
  const open = within(home.pos, 12).find((pos) => {
    const i = tileIndex(pos, scene.fixture.width, scene.fixture.height);
    return distance(pos, home.pos) >= 6 && i >= 0 && moveCost(scene.fixture.terrain[i]) !== null
      && !scene.fixture.units.some((u) => u.id !== veteran.id && distance(u.pos, pos) < 3)
      && !scene.fixture.cities.some((c) => distance(c.pos, pos) < 3)
      && !scene.fixture.resource?.[i] && !scene.fixture.wonder?.[i] && !scene.fixture.ruin[i];
  });
  if (!open) throw new Error('Art fixture has no clear tile for the veteran');
  veteran.pos = open;
  return { game: coreFixture(scene.fixture), id: veteran.id, pos: veteran.pos };
}

async function frame(page: Page, pos: Hex, id: number, zoom: number, selected: boolean) {
  await page.evaluate(({ pos, zoom, id, selected }) => {
    const w = window as unknown as Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void } };
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(selected ? { kind: 'unit', id } : null);
    s.jumpTo(pos);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { pos, zoom, id, selected });
  await page.waitForTimeout(500);
  await page.mouse.move(1, 1);
}

for (const c of cases) for (const [width, height] of [[1000, 600], [1440, 900]]) {
  test(`${c.slug} at ${width}x${height}: normal, far, panel`, async ({ page }) => {
    test.setTimeout(120_000);
    await page.setViewportSize({ width, height });
    const { game, id, pos } = staged(c.faction, c.xp);
    await page.addInitScript(({ game }) => {
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, { game });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('12');
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await page.evaluate(() => document.fonts.ready.then(() => undefined));
    const level = await page.evaluate((id) => {
      const w = window as unknown as Window & { legatum: typeof useGame };
      return w.legatum.getState().views![0].units.find((u) => u.id === id)?.level;
    }, id);
    expect(level).toBe(c.level);
    await mkdir(out, { recursive: true });
    const stem = `${phase}-${c.slug}-${width}x${height}`;
    await frame(page, pos, id, 1.1, false);
    await page.screenshot({ path: path.join(out, `${stem}-normal.png`), animations: 'disabled' });
    await frame(page, pos, id, 0.4, false);
    await page.screenshot({ path: path.join(out, `${stem}-far.png`), animations: 'disabled' });
    await frame(page, pos, id, 1.1, true);
    await expect(page.locator('.up-rank')).toBeVisible();
    await expect(page.locator('.up-rank svg')).toHaveCount(1);
    if (phase === 'after') {
      await expect(page.locator('.up-rank .rank-insignia g')).toHaveCount(c.level);
      await expect(page.locator('.up-rank .rank-insignia path[d^="M3 "][d*="Q12"]')).toHaveCount(2 * Math.max(0, c.level - 3));
      if (c.level === 6) await expect(page.locator('.up-rank .rank-insignia path[d="M9 21 L12 16 L15 21"]')).toHaveCount(2);
    }
    await page.screenshot({ path: path.join(out, `${stem}-panel.png`), animations: 'disabled' });
    if (phase === 'after') {
      await frame(page, pos, id, 1.1, false);
      const point = await page.evaluate((pos) => {
        const w = window as unknown as Window & { legatumMap: { toScreen(h: Hex): { x: number; y: number } } };
        const box = document.querySelector('[data-game-map]')!.getBoundingClientRect();
        const p = w.legatumMap.toScreen(pos);
        return { x: box.x + p.x, y: box.y + p.y };
      }, pos);
      await page.mouse.move(point.x, point.y);
      const row = page.locator('.tilecard-unit').filter({ hasText: 'Field Cannon' }).first();
      await expect(row).toBeVisible();
      await expect(row.locator('svg')).toHaveCount(1);
      await expect(row.locator('.rank-insignia g')).toHaveCount(c.level);
    }
  });
}

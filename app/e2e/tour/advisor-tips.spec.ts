// Situational advisor tips (ui/advisorTips.ts, PR 162): Spend your Gold, Buy the Iron tile (both steps, then placement),
// Found a city near the Iron and Build up your army. A tip with a ring is dismissed by Not now; one without, by Got it.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { SIZES, shot, type GameWindow } from './helpers';
import type { PlayerView } from '../../src/types';

const KNOWN = { enabled: true, seen: ['welcome', 'report', 'race'], practice: { move: 3, fortify: 3, skip: 3, explore: 3, endTurn: 3, closeResearch: 3 } };

async function start(page: Page) {
  await page.addInitScript((k) => { if (!sessionStorage.getItem('seeded')) { localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify(k)); sessionStorage.setItem('seeded', '1'); } }, KNOWN);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
  const stage = page.locator('dialog[data-testid="faction-stage"][open]');
  await stage.locator('[role=option][data-faction="Athens"]').click();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

/** Edit the seat's view in place (turn, Gold, supply...) the way advisorTips.test.ts does. */
async function edit(page: Page, kind: 'gold' | 'buy' | 'found' | 'army') {
  return page.evaluate((kind) => {
    const g = (window as unknown as GameWindow).legatum;
    const s = g.getState();
    const v: PlayerView = structuredClone(s.views![s.seat]);
    v.turn = 30;
    v.economy = { ...v.economy, gold: kind === 'gold' ? 400 : 120 };
    v.supply = [];
    const city = v.cities.find((c) => c.owner === v.me)!;
    const rival = v.players.find((p) => p.id !== v.me && p.team !== v.team)!;
    const w = v.width;
    const at = (i: number) => { const r = Math.floor(i / w), q = i % w; return { q: q - Math.floor(r / 2), r }; };
    const dist = (a: { q: number; r: number }, b: { q: number; r: number }) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    if (kind === 'buy' || kind === 'found') {
      const buy = city.detail?.buyable_tiles ?? [];
      let idx = -1;
      if (kind === 'buy' && buy[0]) idx = v.tiles.findIndex((_, i) => { const p = at(i); return p.q === buy[0].tile.q && p.r === buy[0].tile.r; });
      if (idx < 0) idx = v.tiles.findIndex((t, i) => t.explored && t.owner === null && dist(at(i), city.pos) === 4);
      v.tiles[idx] = { ...v.tiles[idx], resource: 'Iron' };
      v.supply = [{ resource: 'Iron', revealed_by: 'BronzeWorking', revealed: true, supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, spare: 0 } as unknown as NonNullable<PlayerView['supply']>[number]];
      if (kind === 'found') city.detail = { ...city.detail!, buyable_tiles: [] };
    }
    if (kind === 'army') {
      v.players = v.players.map((p) => p.id === v.me ? { ...p, army: 12 } : p.id === rival.id ? { ...p, army: 40 } : p);
      const near = v.tiles.findIndex((_, i) => dist(at(i), city.pos) === 5);
      v.cities.push({ ...city, id: 7001, owner: rival.id, name: 'Sardis', pos: at(near), detail: null });
    }
    g.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) });
    s.select(null);
    return { city: city.id, name: city.name };
  }, kind);
}

const park = (page: Page, h: number) => page.mouse.move(5, h / 2);

test.describe.configure({ mode: 'parallel' });
for (const size of SIZES) test.describe(`advisor tips ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('spend your Gold, then Not now', async ({ page }) => {
    test.setTimeout(120_000);
    await start(page);
    await edit(page, 'gold');
    const card = page.locator('[data-testid="advisor"][data-advisor-id="tip:gold"]');
    await expect(card).toBeVisible();
    await park(page, size.height);
    await shot(page, size, '01-gold-tip');
    await card.getByRole('button', { name: 'Not now' }).hover();
    await shot(page, size, '02-gold-tip-hover-not-now');
    await card.getByRole('button', { name: 'Not now' }).click();
    await expect(card).toHaveCount(0);
    await park(page, size.height);
    await shot(page, size, '03-gold-tip-dismissed');
  });

  test('buy the Iron tile: both steps and placement', async ({ page }) => {
    test.setTimeout(120_000);
    await start(page);
    const { city } = await edit(page, 'buy');
    const card = page.locator('[data-testid="advisor"][data-advisor-id="tip:resource:Iron"]');
    await expect(card).toBeVisible();
    await park(page, size.height);
    await shot(page, size, '04-buy-tile-step1-city');
    await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }), city);
    await expect(page.locator('.city-panel')).toBeVisible();
    await park(page, size.height);
    await shot(page, size, '05-buy-tile-step2-button');
    const btn = page.locator('.city-panel button', { hasText: /^Buy a tile/ }).first();
    await btn.click();
    // Placement keeps the tip, on its last step: the ring is on the Iron tile itself.
    await expect(card).toBeVisible();
    await expect(card).toContainText('the Iron tile');
    await expect(page.getByTestId('advisor-ring')).toBeVisible();
    await park(page, size.height);
    await shot(page, size, '06-buy-tile-placement');
  });

  test('found a city near the Iron', async ({ page }) => {
    test.setTimeout(120_000);
    await start(page);
    await edit(page, 'found');
    await expect(page.locator('[data-testid="advisor"][data-advisor-id="tip:resource:Iron"]')).toBeVisible();
    await park(page, size.height);
    await shot(page, size, '07-found-city-iron');
  });

  test('build up your army', async ({ page }) => {
    test.setTimeout(120_000);
    await start(page);
    await edit(page, 'army');
    const card = page.locator('[data-testid="advisor"][data-advisor-id="tip:army"]');
    await expect(card).toBeVisible();
    await expect(card.getByRole('button', { name: 'Got it' })).toBeVisible();
    await park(page, size.height);
    await shot(page, size, '08-army-tip');
  });
});

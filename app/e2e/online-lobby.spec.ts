import type { Page, Route } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { E2E_RELAY, expect, testWithFailingRequests as test } from './fixtures';

// The online lobby (plans/online-play.md): the game list, with the relay's /lobby answered by the test.
// Joining and playing through the real relay is game.spec.ts's online test. The list's error state makes
// the browser log a failed request, so this file runs without the console-error watch.
// Set ONLINE_LOBBY_SHOTS=<folder> to also save screenshots of each state at both laptop sizes.

type Relay = { setRooms(r: unknown[]): void; setFail(f: boolean): void };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shots = process.env.ONLINE_LOBBY_SHOTS;
if (shots) mkdirSync(shots, { recursive: true });

/** The build and rules this page runs, filled in by openLobby, so a listed game matches unless a test says otherwise. */
let mine = { build: '', rules_version: 0, protocol: 1 };
const room = (over: Record<string, unknown> = {}) => ({
  room: 'room-aaaaaaaa', name: 'Ada', ...mine,
  players: 2, humans: 2, open_seats: 1, phase: 'staging', turn: 0, updated_ms: 1, ...over,
});

/** Answers the relay's game list for this page. */
async function fakeLobby(page: Page): Promise<Relay> {
  let rooms: unknown[] = [];
  let fail = false;
  await page.route(`${E2E_RELAY}/lobby`, (route: Route) =>
    route.fulfill(fail
      ? { status: 500, contentType: 'application/json', headers: { 'access-control-allow-origin': '*' }, body: '{"error":"down"}' }
      : { status: 200, contentType: 'application/json', headers: { 'access-control-allow-origin': '*' }, body: JSON.stringify({ rooms }) }));
  return { setRooms: (r) => { rooms = r; }, setFail: (f) => { fail = f; } };
}

async function openLobby(page: Page, _relay: Relay, host = false) {
  await page.goto('/');
  await page.getByRole('button', { name: /^Multiplayer/ }).click();
  if (!host) await page.getByRole('button', { name: /^Join Game/ }).click();
}

async function snap(page: Page, name: string) {
  if (!shots) return;
  for (const size of SIZES) {
    await page.setViewportSize(size);
    await page.waitForTimeout(150);
    await page.screenshot({ path: join(shots, `${name}-${size.width}x${size.height}.png`) });
  }
}

/** Nothing on the page may be cut off at either laptop size. */
async function expectNoClipping(page: Page) {
  for (const size of SIZES) {
    await page.setViewportSize(size);
    const clipped = await page.evaluate(() =>
      [...document.querySelectorAll('.online-room, .online-room *, .online-empty, .online-empty *, .setup-lede')]
        .filter((el) => el.scrollWidth > el.clientWidth + 1 && getComputedStyle(el).overflow !== 'visible')
        .map((el) => el.className));
    expect(clipped).toEqual([]);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  }
}

test.describe('online lobby', () => {
  let relay: Relay;
  test.beforeEach(async ({ page }) => {
    relay = await fakeLobby(page);
    await page.goto('/');
    mine = await page.evaluate(() => {
      const info = (window as unknown as { legatum: typeof import('../src/store').useGame }).legatum.getState().online!;
      return { build: info.build, rules_version: info.rules_version, protocol: info.protocol };
    });
  });

  test('lists games, blocks a version mismatch and refreshes by itself', async ({ page }) => {
    relay.setRooms([
      room(),
      room({ room: 'room-bbbbbbbb', name: 'Bo from Zürich', build: '8.9.70', phase: 'playing', turn: 14, players: 3, open_seats: 1 }),
    ]);
    await openLobby(page, relay);
    const rows = page.locator('.online-room');
    await expect(rows).toHaveCount(2);
    await expect(rows.nth(0)).toContainText('2 players · 1 open seat');
    await expect(rows.nth(0)).toContainText('Waiting in lobby');
    await expect(rows.nth(1)).toContainText('Turn 14');
    await expect(rows.nth(1)).toContainText(`This game is on version 8.9.70 and you have ${mine.build}. Update the older one to play together.`);
    await expect(rows.nth(0).getByRole('button', { name: /^Join/ })).toBeEnabled();
    await expect(rows.nth(1).getByRole('button', { name: /^Join/ })).toBeDisabled();
    await expectNoClipping(page);
    await snap(page, 'list');
    // A new game appears without any click.
    relay.setRooms([room({ room: 'room-cccccccc', name: 'Cy' }), room(), room({ room: 'room-bbbbbbbb', name: 'Bo', phase: 'playing', turn: 14 })]);
    await expect(rows).toHaveCount(3, { timeout: 8000 });
  });

  test('shows the empty state', async ({ page }) => {
    await openLobby(page, relay);
    await expect(page.getByText('No games are being hosted right now.')).toBeVisible();
    await expectNoClipping(page);
    await snap(page, 'empty');
  });

  test('shows a plain error, then recovers on its own', async ({ page }) => {
    relay.setFail(true);
    await openLobby(page, relay);
    await expect(page.getByText('The list of games could not be loaded.')).toBeVisible();
    await expect(page.getByText('Trying again')).toBeVisible();
    await expectNoClipping(page);
    await snap(page, 'error');
    relay.setFail(false);
    relay.setRooms([room()]);
    await expect(page.locator('.online-room')).toHaveCount(1, { timeout: 8000 });
  });

  test('Back returns to the title', async ({ page }) => {
    await openLobby(page, relay);
    await page.getByRole('button', { name: /Back/ }).click();
    await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  });
});

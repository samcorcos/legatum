import { test, expect, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { createServer } from 'node:net';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
// @ts-expect-error plain .mjs helper without types
import { startFakeRelay } from '../scripts/fake-relay.mjs';

// The online lobby (plans/online-play.md) against a fake relay, with the desktop commands mocked.
// Set ONLINE_LOBBY_SHOTS=<folder> to also save screenshots of each state at both laptop sizes.

type Relay = { url: string; port: number; close(): Promise<void>; setRooms(r: unknown[]): void; setFail(f: boolean): void };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shots = process.env.ONLINE_LOBBY_SHOTS;
if (shots) mkdirSync(shots, { recursive: true });

const room = (over: Record<string, unknown> = {}) => ({
  room: 'room-aaaaaaaa', name: 'Ada', build: '8.9.74', rules_version: 12, protocol: 1,
  players: 2, humans: 2, open_seats: 1, phase: 'staging', turn: 0, updated_ms: 1, ...over,
});

async function openLobby(page: Page, relay: Relay, host = false) {
  await page.addInitScript((info) => {
    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string) => {
        if (cmd === 'online_info') return info;
        throw new Error(`unmocked command ${cmd}`);
      },
    };
  }, { relay: relay.url, build: '8.9.74', rules_version: 12, protocol: 1 });
  await page.goto(`${relay.url}/`);
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
  test.beforeEach(async () => {
    relay = await startFakeRelay({ rooms: [] });
  });
  test.afterEach(async () => {
    await relay.close();
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
    await expect(rows.nth(1)).toContainText('This game is on version 8.9.70 and you have 8.9.74. Update the older one to play together.');
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

// Join through the relay into a real staging room, bridged to the fake LAN server.
test.describe('joining through the relay', () => {
  let lan: ChildProcess;
  let lanBase: string;
  let relay: Relay;
  const root = fileURLToPath(new URL('../', import.meta.url));

  test.beforeAll(async () => {
    const probe = createServer();
    await new Promise<void>((r) => probe.listen(0, '127.0.0.1', r));
    const port = (probe.address() as { port: number }).port;
    await new Promise<void>((r) => probe.close(() => r()));
    lanBase = `http://127.0.0.1:${port}`;
    lan = spawn('node', ['scripts/fake-lan-server.mjs', '--port', String(port)], { cwd: root, stdio: 'pipe' });
    await expect.poll(() => fetch(`${lanBase}/api/status`).then((r) => r.status).catch(() => 0)).toBe(200);
    const seat = (kind: string, name: string) => ({ kind, name, faction: 'Random', team: 0, difficulty: 'Normal' });
    const made = await fetch(`${lanBase}/api/staging`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ seed: '1', width: 40, height: 24, turn_limit: null, seats: [seat('host', 'Ada'), seat('open', 'Player 2'), seat('ai', 'AI 1')] }),
    });
    expect(made.status).toBe(200);
    relay = await startFakeRelay({ upstream: lanBase, rooms: [room({ room: 'room-live' })] });
  });
  test.afterAll(async () => {
    await relay?.close();
    lan?.kill();
  });

  test('Join lands a guest in the host\'s game room', async ({ page }) => {
    const sockets: string[] = [];
    page.on('websocket', (ws) => sockets.push(ws.url()));
    await openLobby(page, relay);
    await page.locator('.online-room').getByRole('button', { name: /^Join/ }).click();
    await expect(page.getByRole('heading', { name: 'Join Game' })).toBeVisible({ timeout: 10_000 });
    await expect.poll(() => sockets.some((u) => u.endsWith('/rooms/room-live/ws'))).toBe(true);
    // Leaving goes back to the title with no room kept.
    await page.getByRole('button', { name: /Back/ }).first().click();
    await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  });
});

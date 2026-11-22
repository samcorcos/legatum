import { endPlanning } from './turnActions';
import { serverBinary } from './fixtures';
import { test, expect, type BrowserContext, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'node:net';
import { once } from 'node:events';

let server: ChildProcess;
let base: string;
let saves: string;
const root = fileURLToPath(new URL('../../', import.meta.url));
const query = process.env.LEGATUM_STAGING_QUERY ?? '';

test.beforeAll(async () => {
  const socket = createServer();
  socket.listen(0, '127.0.0.1');
  await once(socket, 'listening');
  const port = (socket.address() as { port: number }).port;
  await new Promise<void>((resolve) => socket.close(() => resolve()));
  saves = mkdtempSync(join(tmpdir(), 'legatum-staging-e2e-'));
  base = `http://127.0.0.1:${port}`;
  server = spawn(serverBinary(), ['--port', String(port), '--static', join(root, 'app/dist'), '--saves', saves, '--archive-dir', join(saves, 'archive')], { stdio: 'pipe' });
  let log = '';
  server.stderr?.on('data', (chunk) => { log += chunk; });
  await expect.poll(async () => {
    if (server.exitCode !== null) throw new Error(`Server exited: ${log}`);
    return fetch(`${base}/api/status`).then((r) => r.status).catch(() => 0);
  }).toBe(200);
});

test.afterAll(async () => {
  if (server && server.exitCode === null) {
    const exited = once(server, 'exit');
    server.kill();
    await exited;
  }
  if (saves) rmSync(saves, { recursive: true, force: true });
});

const status = async () => (await fetch(`${base}/api/status`)).json();
const startButton = (page: Page) => page.getByRole('button', { name: /^Start game/ });
const readyButton = (page: Page) => page.getByRole('button', { name: /^Ready Lock/ });

// WebKit (unlike Chromium) surfaces the browser's own ResizeObserver backpressure notice as an
// uncaught page error; it's benign browser noise, not an app fault (see e2e/fixtures.ts).
const watchPageErrors = (page: Page, errors: string[]) => page.on('pageerror', (e) => {
  if (e.message === 'ResizeObserver loop completed with undelivered notifications.') return;
  errors.push(e.message);
});

async function multiplayer(page: Page, suffix = query) {
  await page.goto(`${base}/${suffix}`);
  await page.getByRole('button', { name: /^Multiplayer/ }).click();
}

// The private subscription is only accepted from loopback; public status omits the seed.
async function hostSeed(host: Page) {
  return host.evaluate(() => new Promise<string>((resolve, reject) => {
    const socket = new WebSocket(`ws://${location.host}/ws`);
    socket.onopen = () => socket.send(JSON.stringify({ type: 'staging_watch', host: true }));
    socket.onerror = () => { socket.close(); reject(new Error('Host subscription failed')); };
    socket.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (message.type === 'staging') { socket.close(); resolve(message.state.seed); }
    };
  }));
}

async function hostRoom(host: Page) {
  await multiplayer(host);
  await host.getByRole('button', { name: /^Host Game/ }).click();
  await host.getByLabel('Map seed').fill('5');
  await expect.poll(() => hostSeed(host)).toBe('5');
  expect((await status()).staging.config.seed).toBeUndefined();
  expect((await status()).staging.seed).toBeUndefined();
  await expect.poll(async () => (await status()).staging?.occupants[0]?.connected).toBe(true);
}

async function joinRoom(page: Page, seat: number, name: string, lanGuest = false) {
  if (lanGuest) await page.goto(`${base}/${query}`);
  else await multiplayer(page);
  await page.getByRole('button', { name: /^Join Game/ }).click();
  await page.getByRole('row').nth(seat + 1).getByRole('button', { name: 'Take seat', exact: true }).click();
  await page.getByLabel('Your name').fill(name);
  await expect.poll(async () => (await status()).staging?.occupants[seat]?.name).toBe(name);
}

// Both contexts use loopback in CI. Show the guest the LAN status flag; the Rust
// protocol tests separately exercise release authorization with LAN/loopback connection identities.
async function guestStatus(context: BrowserContext) {
  await context.route('**/api/status', async (route) => {
    const response = await route.fetch();
    await route.fulfill({ response, json: { ...await response.json(), is_local: false } });
  });
}

test('a new guest context reclaims its empire after the host opens the disconnected game seat', async ({ browser }, testInfo) => {
  const hostContext = await browser.newContext();
  let guestContext = await browser.newContext();
  const errors: string[] = [];
  try {
    await guestStatus(guestContext);
    const host = await hostContext.newPage();
    const guest = await guestContext.newPage();
    watchPageErrors(host, errors);
    watchPageErrors(guest, errors);
    await hostRoom(host);
    await joinRoom(guest, 1, 'Alice', true);
    await guest.getByRole('button', { name: /^Your faction/ }).click();
    // Clicking a faction in the gallery picks it; Close then closes the picker and keeps the choice.
    await guest.getByRole('option', { name: /Byzantium/ }).click();
    await guest.getByRole('button', { name: 'Close faction picker' }).click();
    await readyButton(guest).click();
    await expect(startButton(host)).toBeEnabled();
    await startButton(host).click();
    await expect(host.locator('.turn b')).toHaveText('1');
    await expect(guest.locator('.turn b')).toHaveText('1');
    const credential = await guest.evaluate(() => JSON.parse(sessionStorage.getItem('legatum.lan.seat')!));
    const hostCredential = await host.evaluate(() => sessionStorage.getItem('legatum.lan.seat'));

    // Reloading this tab keeps its credential and automatically returns to its empire.
    await guest.reload();
    await expect(guest.locator('.turn b')).toHaveText('1');
    expect(await guest.evaluate(() => JSON.parse(sessionStorage.getItem('legatum.lan.seat')!))).toEqual(credential);
    expect(await guest.evaluate(() => localStorage.getItem('legatum.lan.seat'))).toBeNull();
    await expect(guest.getByRole('button', { name: 'Open seat', exact: true })).toHaveCount(0);

    await guestContext.close();
    await expect.poll(async () => (await status()).seats[1].connected).toBe(false);
    guestContext = await browser.newContext();
    await guestStatus(guestContext);
    const newcomer = await guestContext.newPage();
    watchPageErrors(newcomer, errors);
    await newcomer.goto(`${base}/${query}`);
    await newcomer.getByRole('button', { name: /^Join Game/ }).click();
    expect(await newcomer.evaluate(() => sessionStorage.getItem('legatum.lan.seat'))).toBeNull();
    const empire = newcomer.locator('.join-seat').filter({ hasText: 'Alice' });
    await expect(empire).toBeDisabled();
    await expect(empire).toContainText('Reserved');
    await expect(empire).toContainText('Ask the host to open this seat');
    const openSeat = host.getByTestId('lan-seat-1').getByRole('button', { name: 'Open seat', exact: true });
    await expect(openSeat).toBeVisible();
    await host.screenshot({ path: testInfo.outputPath('host-open-seat.png'), fullPage: true });
    host.once('dialog', async (dialog) => {
      expect(dialog.message()).toBe("Let another device take over Alice's empire?");
      await dialog.dismiss();
    });
    await openSeat.click();
    await expect(empire).toBeDisabled();
    expect((await status()).seats[1].reserved).toBe(true);
    host.once('dialog', (dialog) => dialog.accept());
    await openSeat.click();
    await expect.poll(async () => (await status()).seats[1].reserved).toBe(false);
    await expect(empire).toBeEnabled();
    await expect(empire).toContainText('Rejoin as Alice');
    expect(await host.evaluate(() => sessionStorage.getItem('legatum.lan.seat'))).toBe(hostCredential);
    await empire.click();
    await expect(newcomer.locator('.turn b')).toHaveText('1');
    await expect(newcomer.getByTestId('lan-seat-1')).toContainText('You');
    const replacement = await newcomer.evaluate(() => JSON.parse(sessionStorage.getItem('legatum.lan.seat')!));
    expect(replacement).toMatchObject({ seat: 1, name: 'Alice', gameId: credential.gameId });
    expect(replacement.token).not.toBe(credential.token);
    expect((await status()).seats[1]).toMatchObject({ player: 1, name: 'Alice', faction: 'Byzantium', connected: true, ready: false, orders: 0 });
    await newcomer.screenshot({ path: testInfo.outputPath('rejoined-empire.png'), fullPage: true });
    expect(errors).toEqual([]);
  } finally {
    await hostContext.close();
    await guestContext.close();
  }
});

test('the host opens a disconnected lobby reservation without toggling its seat type', async ({ browser }) => {
  const hostContext = await browser.newContext();
  let guestContext = await browser.newContext();
  try {
    await guestStatus(guestContext);
    const host = await hostContext.newPage();
    const guest = await guestContext.newPage();
    await hostRoom(host);
    await joinRoom(guest, 1, 'Alice', true);
    await guestContext.close();
    await expect.poll(async () => (await status()).staging.occupants[1]?.connected).toBe(false);
    guestContext = await browser.newContext();
    await guestStatus(guestContext);
    const newcomer = await guestContext.newPage();
    await newcomer.goto(`${base}/${query}`);
    await newcomer.getByRole('button', { name: /^Join Game/ }).click();
    const row = newcomer.getByRole('row').nth(2);
    await expect(row).toContainText('Reserved — ask the host to open this seat');
    await expect(row.getByRole('button', { name: 'Take seat', exact: true })).toHaveCount(0);
    host.once('dialog', (dialog) => dialog.accept());
    await host.getByRole('row').nth(2).getByRole('button', { name: 'Open seat', exact: true }).click();
    await expect(host.getByLabel('Seat 2 type')).toHaveValue('human');
    await row.getByRole('button', { name: 'Take seat', exact: true }).click();
    await expect(newcomer.getByLabel('Your name')).toBeVisible();
    expect((await status()).staging.occupants[0]).toMatchObject({ connected: true, is_host: true });
    await host.getByRole('button', { name: '← Back', exact: true }).click();
    await expect.poll(async () => (await status()).staging).toBeNull();
  } finally {
    await hostContext.close();
    await guestContext.close();
  }
});

test('two contexts create, reclaim after reload, acknowledge settings, re-ready, start and play', async ({ browser }, testInfo) => {
  const hostContext = await browser.newContext();
  const guestContext = await browser.newContext();
  try {
    const host = await hostContext.newPage();
    const guest = await guestContext.newPage();
    const errors: string[] = [];
    watchPageErrors(host, errors);
    watchPageErrors(guest, errors);
    await hostRoom(host);
    await joinRoom(guest, 1, 'Alice');
    await guest.getByRole('button', { name: /^Your faction/ }).click();
    // Clicking a faction in the gallery picks it; Close then closes the picker and keeps the choice.
    await guest.getByRole('option', { name: /Byzantium/ }).click();
    await guest.getByRole('button', { name: 'Close faction picker' }).click();
    await readyButton(guest).click();
    await expect(startButton(host)).toBeEnabled();

    await guest.reload();
    await expect(guest.getByLabel('Your name')).toHaveValue('Alice');
    await expect(guest.getByLabel('Map seed')).toHaveValue('Hidden');
    await expect(guest.getByRole('button', { name: /^Your faction/ })).toContainText('Byzantium');
    await readyButton(guest).click();
    await expect(startButton(host)).toBeEnabled();

    // Hold the settings request beyond the debounce to prove both windows of the race.
    let release!: () => void;
    const gate = new Promise<void>((resolve) => { release = resolve; });
    let held = false;
    let starts = 0;
    host.on('request', (r) => { if (r.url().endsWith('/api/staging/start')) starts++; });
    await host.route('**/api/staging', async (route) => {
      if (route.request().method() === 'POST' && route.request().postDataJSON().seed === '6') {
        held = true;
        await gate;
      }
      await route.continue();
    });
    await host.getByLabel('Map seed').fill('6');
    expect(await startButton(host).isDisabled()).toBe(true);
    await startButton(host).evaluate((button: HTMLButtonElement) => button.click());
    await expect.poll(() => held).toBe(true);
    expect(starts).toBe(0);
    expect(await hostSeed(host)).toBe('5');
    expect((await status()).staging.config.seed).toBeUndefined();
    await expect(startButton(host)).toBeDisabled();
    release();
    await expect.poll(() => hostSeed(host)).toBe('6');
    await expect(guest.getByLabel('Map seed')).toHaveValue('Hidden');
    await expect(readyButton(guest)).toBeVisible();
    await expect(startButton(host)).toBeDisabled();
    await readyButton(guest).click();
    await expect(startButton(host)).toBeEnabled();
    await host.screenshot({ path: testInfo.outputPath('ready-room.png'), fullPage: true });
    await startButton(host).click();
    await expect(host.locator('.turn b')).toHaveText('1');
    await expect(guest.locator('.turn b')).toHaveText('1');
    const save = await (await fetch(`${base}/api/save`)).json();
    // Saves carry the u64 seed as a decimal string so JavaScript never rounds it.
    expect(save.setup.seed).toBe('6');
    expect(save.setup.players[1]).toMatchObject({ name: 'Alice', faction: 'Byzantium', is_human: true });
    await endPlanning(host);
    expect((await status()).turn).toBe(1);
    await endPlanning(guest);
    await expect(host.locator('.turn b')).toHaveText('2');
    await expect(guest.locator('.turn b')).toHaveText('2');
    await guest.screenshot({ path: testInfo.outputPath('guest-first-turn.png'), fullPage: true });
    expect(errors).toEqual([]);
  } finally {
    await hostContext.close();
    await guestContext.close();
  }
});

test('removing an occupied row preserves the adjacent player slot', async ({ browser }) => {
  const hostContext = await browser.newContext();
  const guestContext = await browser.newContext();
  try {
    const host = await hostContext.newPage();
    const alice = await guestContext.newPage();
    const bob = await guestContext.newPage();
    await hostRoom(host);
    await host.getByLabel('Seat 3 type').selectOption('human');
    await expect.poll(async () => (await status()).staging.config.seats[2].kind).toBe('open');
    await joinRoom(alice, 1, 'Alice');
    await joinRoom(bob, 2, 'Bob');
    await readyButton(alice).click();
    await readyButton(bob).click();
    await expect(startButton(host)).toBeEnabled();
    await host.getByLabel('Remove seat 2', { exact: true }).click();
    await expect(host.getByLabel('Seat 2 type')).toHaveValue('closed');
    await expect(host.getByLabel('Seat 3 type')).toHaveValue('human');
    await expect(alice.getByText('The host changed your seat.')).toBeVisible();
    await expect(bob.getByLabel('Your name')).toHaveValue('Bob');
    const room = (await status()).staging;
    expect(room.config.seats).toHaveLength(8);
    expect(room.occupants[1]).toBeNull();
    expect(room.occupants[2]).toMatchObject({ name: 'Bob', connected: true, ready: false });
    await expect(startButton(host)).toBeDisabled();
    await readyButton(bob).click();
    await expect(startButton(host)).toBeEnabled();
    await startButton(host).click();
    await expect(bob.locator('.turn b')).toHaveText('1');
    const save = await (await fetch(`${base}/api/save`)).json();
    expect(save.setup.players[1].name).toBe('Bob');
  } finally {
    await hostContext.close();
    await guestContext.close();
  }
});

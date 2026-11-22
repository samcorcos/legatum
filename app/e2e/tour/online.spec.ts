// Screen tour, online co-op in the browser: the Multiplayer menu, the game list in every network state,
// the host's game room as the host and a guest see it, and both players on turn one (top bar, seat bar,
// game menu). Run it alone with `npx playwright test e2e/tour/online`. Needs app/dist built for the
// e2e relay (LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh --skip-wasm); the host and
// guest test starts the real relay under wrangler dev (fixtures.ts). Shots land in
// test-results/screen-tour/<size>/online/.
import type { Page, Route } from '@playwright/test';
import { E2E_RELAY, expect, testWithFailingRequests } from '../fixtures';
import { dismissInterstitials, endPlanning } from '../turnActions';
import { chooseCapitalProduction } from '../opening';
import { SIZES, shot, type GameWindow } from './helpers';

const noHints = (p: Page) => p.addInitScript(() => {
  if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
});

async function multiplayer(page: Page) {
  await page.goto('/');
  await page.getByRole('button', { name: /^Multiplayer/ }).click();
}

const cors = { 'access-control-allow-origin': '*' };

for (const size of SIZES) testWithFailingRequests.describe(`online screens ${size.name}`, () => {
  testWithFailingRequests.use({ viewport: { width: size.width, height: size.height } });

  testWithFailingRequests('multiplayer menu: normal, no relay, opening, failed', async ({ page }) => {
    await noHints(page);
    await multiplayer(page);
    await expect(page.getByRole('button', { name: /^Host Game/ })).toBeVisible();
    await page.mouse.move(5, 5);
    await shot(page, size, '20-online-menu');

    // A build with no relay offers hot seat only.
    await page.getByRole('button', { name: /Back/ }).click();
    await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum; s.setState({ online: { ...s.getState().online!, relay: null } }); });
    await page.getByRole('button', { name: /^Multiplayer/ }).click();
    await shot(page, size, '21-online-menu-no-relay');

    // Opening: the relay never answers the room request.
    const slow = await page.context().newPage();
    await slow.route(`${E2E_RELAY}/rooms`, () => { /* hang */ });
    await multiplayer(slow);
    await slow.getByRole('button', { name: /^Host Game/ }).click();
    await expect(slow.getByText('Opening your game…')).toBeVisible();
    await slow.mouse.move(5, 5);
    await shot(slow, size, '22-online-menu-opening');

    // Failed: the relay refuses.
    const bad = await page.context().newPage();
    await bad.route(`${E2E_RELAY}/rooms`, (r: Route) => r.abort());
    await multiplayer(bad);
    await bad.getByRole('button', { name: /^Host Game/ }).click();
    await expect(bad.getByText(/Could not open your game/)).toBeVisible();
    await bad.mouse.move(5, 5);
    await shot(bad, size, '23-online-menu-failed');
  });

  testWithFailingRequests('game list: loading, empty, error, with rooms', async ({ page }) => {
    let rooms: unknown[] = [];
    let mode: 'ok' | 'fail' | 'hang' = 'hang';
    await page.route(`${E2E_RELAY}/lobby`, (route: Route) => {
      if (mode === 'hang') return; // loading
      return route.fulfill(mode === 'fail'
        ? { status: 500, contentType: 'application/json', headers: cors, body: '{"error":"down"}' }
        : { status: 200, contentType: 'application/json', headers: cors, body: JSON.stringify({ rooms }) });
    });
    await multiplayer(page);
    const mine = await page.evaluate(() => {
      const info = (window as unknown as GameWindow).legatum.getState().online!;
      return { build: info.build, rules_version: info.rules_version, protocol: info.protocol };
    });
    const room = (over: Record<string, unknown> = {}) => ({
      room: 'room-aaaaaaaa', name: 'Ada', ...mine, players: 2, humans: 2, open_seats: 1, phase: 'staging', turn: 0, updated_ms: 1, ...over,
    });
    await page.getByRole('button', { name: /^Join Game/ }).click();
    await page.mouse.move(5, 5);
    await shot(page, size, '30-list-loading');

    mode = 'ok';
    await page.getByRole('button', { name: /Back/ }).click();
    await page.getByRole('button', { name: /^Multiplayer/ }).click();
    await page.getByRole('button', { name: /^Join Game/ }).click();
    await expect(page.getByText('No games are being hosted right now.')).toBeVisible();
    await page.mouse.move(5, 5);
    await shot(page, size, '31-list-empty');

    mode = 'fail';
    await page.getByRole('button', { name: /Back/ }).click();
    await page.getByRole('button', { name: /^Multiplayer/ }).click();
    await page.getByRole('button', { name: /^Join Game/ }).click();
    await expect(page.getByText('The list of games could not be loaded.')).toBeVisible();
    await page.mouse.move(5, 5);
    await shot(page, size, '32-list-error');

    mode = 'ok';
    rooms = [
      room(),
      room({ room: 'room-a2', name: 'Player' }),
      room({ room: 'room-a3', name: 'Player' }),
      room({ room: 'room-bbbbbbbb', name: 'Bo from Zürich', build: '8.9.70', phase: 'playing', turn: 14, players: 3 }),
      room({ room: 'room-cccccccc', name: 'Cy', phase: 'playing', turn: 31, players: 4, open_seats: 0 }),
      room({ room: 'room-dddddddd', name: '', rules_version: mine.rules_version - 1 }),
    ];
    await page.getByRole('button', { name: /Back/ }).click();
    await page.getByRole('button', { name: /^Multiplayer/ }).click();
    await page.getByRole('button', { name: /^Join Game/ }).click();
    await expect(page.locator('.online-room')).toHaveCount(6, { timeout: 8000 });
    await page.mouse.move(5, 5);
    await shot(page, size, '33-list-rooms');

    // Join a room with no host behind it: the failure shows in its row.
    await page.locator('.online-room').first().getByRole('button', { name: /^Join/ }).click();
    await expect(page.locator('.online-room').first().getByRole('alert')).toBeVisible({ timeout: 15_000 });
    await page.mouse.move(5, 5);
    await shot(page, size, '34-list-join-error');
  });
});

// Without the console-error watch: the last shot closes the host's tab, and the guest's status checks
// then get the relay's 503 (host away) on purpose.
for (const size of SIZES) testWithFailingRequests.describe(`online co-op ${size.name}`, () => {
  testWithFailingRequests.use({ viewport: { width: size.width, height: size.height } });

  testWithFailingRequests('host room, guest room, both on turn one, game menu', async ({ page: host, browser, baseURL, relay }) => {
    testWithFailingRequests.setTimeout(240_000);
    const guestContext = await browser.newContext({ baseURL, viewport: { width: size.width, height: size.height } });
    const guest = await guestContext.newPage();
    expect(relay).toBeTruthy();
    // The relay keeps rooms for a while after a run: a unique name finds this run's room.
    const hostName = `Ada ${String(Date.now()).slice(-5)}`;
    await noHints(host);
    await noHints(guest);
    await multiplayer(host);
    await host.getByLabel('Your name in the game list').fill(hostName);
    await host.getByRole('button', { name: /^Host Game/ }).click();
    await expect(host.getByRole('heading', { name: 'Host Game', exact: true })).toBeVisible();
    await expect(host.getByText('Waiting for every online seat to be taken and Ready.', { exact: true })).toBeVisible();
    await host.mouse.move(5, 5);
    await shot(host, size, '40-host-room-waiting');
    const small = host.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ });
    await small.click();
    await host.getByLabel('Map seed', { exact: true }).fill('20240926');
    await host.evaluate(() => { (document.activeElement as HTMLElement | null)?.blur(); for (const el of document.querySelectorAll('*')) if (el.scrollTop) el.scrollTop = 0; });

    await multiplayer(guest);
    await guest.getByRole('button', { name: /^Join Game/ }).click();
    const row = guest.locator('.online-room').filter({ hasText: hostName });
    await expect(row).toHaveCount(1, { timeout: 20_000 });
    await guest.mouse.move(5, 5);
    await shot(guest, size, '41-guest-list-live');
    await row.getByRole('button', { name: /^Join/ }).click();
    await expect(guest.getByRole('heading', { name: 'Join Game', exact: true })).toBeVisible();
    await guest.mouse.move(5, 5);
    await shot(guest, size, '42-guest-room-arrived');
    await guest.getByRole('row').nth(2).getByRole('button', { name: 'Take seat', exact: true }).click();
    await guest.getByLabel('Your name', { exact: true }).fill('Alice');
    await guest.mouse.move(5, 5);
    await shot(guest, size, '43-guest-room-seated');
    await guest.getByRole('button', { name: /^Ready Lock/ }).click();
    const start = host.getByRole('button', { name: /^Start game/ });
    await expect(start).toBeEnabled();
    await guest.mouse.move(5, 5);
    await shot(guest, size, '44-guest-room-ready');
    await host.mouse.move(5, 5);
    await shot(host, size, '45-host-room-ready');

    await start.click();
    for (const p of [host, guest]) {
      await expect(p.getByTestId('turn-number')).toHaveText('1', { timeout: 90_000 });
      await expect(p.locator('.loading-screen')).toHaveCount(0);
      await dismissInterstitials(p);
    }
    await host.mouse.move(size.width / 2, size.height - 4);
    await shot(host, size, '50-host-turn-one');
    await guest.mouse.move(size.width / 2, size.height - 4);
    await shot(guest, size, '51-guest-turn-one');

    // The connection chip's tooltip, as each side sees it.
    await host.locator('.topbar .net').hover();
    await host.waitForTimeout(500);
    await shot(host, size, '52-host-net-tip');
    await guest.locator('.topbar .net').hover();
    await guest.waitForTimeout(500);
    await shot(guest, size, '53-guest-net-tip');

    // The game menu.
    for (const [p, name] of [[host, '54-host-game-menu'], [guest, '55-guest-game-menu']] as const) {
      await p.getByRole('button', { name: 'Menu', exact: true }).click();
      await p.waitForTimeout(300);
      await shot(p, size, name);
      await p.keyboard.press('Escape');
      await p.waitForTimeout(300);
      if (await p.locator('#game-menu').isVisible()) {
        console.log(`[tour] ${name}: Escape left the game menu open`);
        await p.mouse.click(size.width / 2, size.height / 2 + 120);
      }
      await expect(p.locator('#game-menu')).toHaveCount(0);
    }

    // The seat list (Turn details) as each side sees it.
    for (const [p, name] of [[host, '55b-host-seats'], [guest, '55c-guest-seats']] as const) {
      const details = p.locator('details.sidebar-turn-more').first();
      if (!(await details.evaluate((d) => (d as HTMLDetailsElement).open))) await details.locator(':scope > summary').click();
      await p.mouse.move(size.width / 2, size.height - 4);
      await shot(p, size, name);
    }

    // The host is Ready, waiting on the guest.
    await chooseCapitalProduction(host);
    await endPlanning(host);
    await expect(host.getByRole('button', { name: /^Waiting for Alice/ })).toBeVisible();
    await host.mouse.move(size.width / 2, size.height - 4);
    await shot(host, size, '56-host-waiting-for-guest');
    await guest.mouse.move(size.width / 2, size.height - 4);
    await shot(guest, size, '57-guest-host-ready');

    // The host closes the tab: what the guest sees.
    await host.close();
    await guest.waitForTimeout(8000);
    await guest.mouse.move(size.width / 2, size.height - 4);
    await shot(guest, size, '58-guest-host-gone');
    await guestContext.close();
  });
});

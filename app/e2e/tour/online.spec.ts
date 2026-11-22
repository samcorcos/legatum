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

  // Every refusal the relay can give to a new game, as the Multiplayer menu shows it. The relay's real answers are
  // staged, with Retry-After readable only where the relay exposes it (a cross-origin page sees no other header).
  const REFUSALS = [
    { id: '24a-refused-too-fast', status: 429, error: 'You are opening games too quickly. Try again in a few minutes.', headers: { 'retry-after': '300', 'access-control-expose-headers': 'Retry-After' }, sub: 'Try again in 5 minutes', retry: false },
    { id: '24b-refused-too-many', status: 429, error: 'You already have 2 games open. Close one before hosting another.', headers: {}, sub: 'Close one of your games first', retry: false },
    { id: '24c-refused-full', status: 503, error: 'This lobby is full. Try again in a few minutes.', headers: {}, sub: 'Try again', retry: true },
    { id: '24d-refused-closed', status: 503, error: 'Hosting new games is paused. Try again later.', headers: {}, sub: 'Paused for now', retry: false },
  ] as const;
  const refuse = (page: Page, r: { status: number; error: string; headers: Record<string, string> }) =>
    page.route(`${E2E_RELAY}/rooms`, (route: Route) => route.request().method() === 'OPTIONS'
      ? route.fulfill({ status: 204, headers: { ...cors, 'access-control-allow-headers': 'content-type', 'access-control-allow-methods': 'POST, OPTIONS' } })
      : route.fulfill({ status: r.status, contentType: 'application/json', headers: { ...cors, ...r.headers }, body: JSON.stringify({ error: r.error }) }));
  /** The menu stays inside the window and above the title footer, whatever lines are showing. */
  const expectMenuAboveFooter = async (page: Page, what: string) => {
    const box = await page.evaluate(() => ({
      menu: document.querySelector('.title-menu')!.getBoundingClientRect().bottom,
      foot: document.querySelector('.title-foot')!.getBoundingClientRect().top,
      height: window.innerHeight,
    }));
    console.log(`[tour] ${what}: menu bottom ${Math.round(box.menu)}, footer top ${Math.round(box.foot)}, window ${box.height}`);
    expect(box.menu, `${what}: the menu ends above the title footer`).toBeLessThanOrEqual(box.foot);
    expect(box.menu, `${what}: the menu is inside the window`).toBeLessThanOrEqual(box.height);
  };

  testWithFailingRequests('multiplayer menu: five host refusals, and the worst case with a game open', async ({ page }) => {
    await noHints(page);
    for (const r of REFUSALS) {
      const p = await page.context().newPage();
      await refuse(p, r);
      await multiplayer(p);
      await p.getByRole('button', { name: /^Host Game/ }).click();
      await expect(p.locator('.title-notice')).toHaveText(r.error.includes('2 games') ? 'Your internet connection already has 2 games open. Close one before hosting another.' : /.+/);
      expect(await p.getByRole('button', { name: /^Host Game/ }).evaluate((b) => b.classList.contains('primary'))).toBe(r.retry);
      expect(await p.getByRole('button', { name: /^Join Game/ }).evaluate((b) => b.classList.contains('primary'))).toBe(!r.retry);
      await expect(p.getByRole('button', { name: /^Host Game/ })).toContainText(r.sub);
      await p.mouse.move(5, 5);
      await expectMenuAboveFooter(p, r.id);
      await shot(p, size, r.id);
    }
    // The relay is unreachable: Try again is true, and Host Game stays the primary.
    const off = await page.context().newPage();
    await off.route(`${E2E_RELAY}/rooms`, (r: Route) => r.abort());
    await multiplayer(off);
    await off.getByRole('button', { name: /^Host Game/ }).click();
    await expect(off.getByRole('button', { name: /^Host Game/ })).toContainText('Try again');
    await off.mouse.move(5, 5);
    await shot(off, size, '24e-refused-offline');

    // Worst case: a game is open (Resume row), a two-line notice, and the name note, all at once.
    const worst = await page.context().newPage();
    await refuse(worst, REFUSALS[0]);
    await multiplayer(worst);
    await worst.evaluate(() => { (window as unknown as GameWindow).legatum.setState({ views: [{ turn: 14 }] as never }); });
    await worst.getByLabel('Your name in the game list').fill('Ada \u{1F600}');
    await worst.getByRole('button', { name: /^Host Game/ }).click();
    await expect(worst.locator('.title-notice')).toBeVisible();
    await worst.mouse.move(5, 5);
    await expect(worst.locator('.host-name .host-name-note')).toHaveText('Shown in the game list as “Ada”.');
    await expectMenuAboveFooter(worst, '25a worst case (Resume + notice + name note)');
    await shot(worst, size, '25a-menu-worst-case');
    await worst.getByLabel('Your name in the game list').fill('\u{1F600}\u{1F600}');
    await expect(worst.locator('.host-name .host-name-note')).toHaveText('Listed as “Game” and four characters.');
    await expectMenuAboveFooter(worst, '25b worst case, nothing showable');
    await shot(worst, size, '25b-menu-name-note-empty');

    // The name note with a game open and no other message.
    const note = await page.context().newPage();
    await multiplayer(note);
    await note.evaluate(() => { (window as unknown as GameWindow).legatum.setState({ views: [{ turn: 14 }] as never }); });
    await note.getByLabel('Your name in the game list').fill('Ada \u{1F600} of Rome');
    await note.mouse.move(5, 5);
    await expectMenuAboveFooter(note, '25c name note, game open');
    await shot(note, size, '25c-menu-name-note-game-open');
  });

  // The two ways a guest's connection can be stopped, on the Join screen with six seats: the notice sits under the
  // header at scroll 0, never below the seats.
  for (const [id, message] of [
    ['26a-join-too-many', 'Too many people on your internet connection have joined this game. Close any extra windows, then pick your seat again.'],
    ['26b-join-still-too-fast', 'You are still sending too much, too fast. Wait a minute, then pick your seat again.'],
  ] as const) testWithFailingRequests(`join screen, six seats: ${id}`, async ({ page }) => {
    await noHints(page);
    await page.goto('/');
    await page.evaluate((error) => {
      const factions = ['Athens', 'Byzantium', 'Celts', 'Mongols', 'India', 'Germany'] as const;
      const seats = factions.map((faction, i) => ({ seat: i, player: i, name: `Player ${i + 1}`, faction, connected: i === 1, reserved: false, ready: false, orders: 0 }));
      (window as unknown as GameWindow).legatum.setState({
        screen: 'join', error,
        net: { status: { mode: 'server', has_game: true, turn: 1, seats, is_local: false }, mySeat: null, seats, conn: 'closed' },
      } as never);
    }, message);
    await expect(page.getByRole('heading', { name: 'Choose your seat' })).toBeVisible();
    await page.mouse.move(5, 5);
    const alert = page.getByRole('alert').filter({ hasText: message });
    await expect(alert).toBeVisible();
    const box = (await alert.boundingBox())!;
    const first = (await page.locator('.join-seat').first().boundingBox())!;
    expect(box.y + box.height, 'the notice is in the window at scroll 0').toBeLessThanOrEqual(size.height);
    expect(box.y + box.height, 'the notice sits above the seats').toBeLessThanOrEqual(first.y);
    await shot(page, size, id);
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
    testWithFailingRequests.setTimeout(420_000);
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

    // The host saves the online game to a file (Save to file…, host only).
    await host.getByRole('button', { name: 'Menu', exact: true }).click();
    const download = host.waitForEvent('download');
    await host.getByRole('button', { name: 'Save to file…', exact: true }).click();
    const savePath = testWithFailingRequests.info().outputPath('online-save.legatum.json');
    await (await download).saveAs(savePath);

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

    // Host from a save: a new tab picks the file on the Multiplayer screen, then the guest rejoins.
    const host2Context = await browser.newContext({ baseURL, viewport: { width: size.width, height: size.height } });
    const host2 = await host2Context.newPage();
    await noHints(host2);
    await multiplayer(host2);
    await host2.getByLabel('Your name in the game list').fill(hostName);
    // A file that is not a save first: the refusal.
    const junkPath = testWithFailingRequests.info().outputPath('junk.json');
    (await import('node:fs')).writeFileSync(junkPath, '{"hello":1}');
    let chooser = host2.waitForEvent('filechooser');
    await host2.getByRole('button', { name: /^Host from a save/ }).click();
    await (await chooser).setFiles(junkPath);
    await host2.waitForTimeout(800);
    await host2.mouse.move(5, 5);
    await shot(host2, size, '60-host-from-save-bad-file');
    chooser = host2.waitForEvent('filechooser');
    await host2.getByRole('button', { name: /^Host from a save/ }).click();
    await (await chooser).setFiles(savePath);
    await expect(host2.getByTestId('turn-number')).toBeVisible({ timeout: 90_000 });
    await host2.waitForTimeout(1500);
    await dismissInterstitials(host2);
    await host2.mouse.move(size.width / 2, size.height - 4);
    await shot(host2, size, '61-host-from-save-turn');
    await host2.getByRole('button', { name: 'Menu', exact: true }).click();
    await host2.waitForTimeout(300);
    await shot(host2, size, '62-host-from-save-menu');
    await host2.keyboard.press('Escape');

    const guest2Context = await browser.newContext({ baseURL, viewport: { width: size.width, height: size.height } });
    const guest2 = await guest2Context.newPage();
    await noHints(guest2);
    await multiplayer(guest2);
    await guest2.getByRole('button', { name: /^Join Game/ }).click();
    const row2 = guest2.locator('.online-room').filter({ hasText: hostName });
    await expect(row2.first()).toBeVisible({ timeout: 20_000 });
    await guest2.mouse.move(5, 5);
    await shot(guest2, size, '63-guest-list-resumed');
    await row2.first().getByRole('button', { name: /^Join/ }).click();
    await guest2.waitForTimeout(3000);
    await guest2.mouse.move(5, 5);
    await shot(guest2, size, '64-guest-resumed-arrived');
    await guest2.getByRole('button', { name: /Rejoin as Alice/ }).click();
    await expect(guest2.getByTestId('turn-number')).toBeVisible({ timeout: 60_000 });
    await guest2.waitForTimeout(1500);
    await dismissInterstitials(guest2);
    await guest2.mouse.move(size.width / 2, size.height - 4);
    await shot(guest2, size, '65-guest-resumed-seated');
    await host2.mouse.move(size.width / 2, size.height - 4);
    await shot(host2, size, '66-host-resumed-guest-back');
    await guest2Context.close();
    await host2Context.close();
    await guestContext.close();
  });
});

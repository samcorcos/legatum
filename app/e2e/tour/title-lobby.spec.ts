// Screen tour, title, lobby, LAN, hot seat and scenarios. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/title-lobby`. Shots land in test-results/screen-tour/<size>/title-lobby/.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { SIZES, shot } from './helpers';
import { spawn, type ChildProcess } from 'node:child_process';
import { createServer } from 'node:net';
import { fileURLToPath } from 'node:url';
// @ts-expect-error plain .mjs helper without types
import { startFakeRelay } from '../../scripts/fake-relay.mjs';
import { test as plain, type Page } from '@playwright/test';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });


  test('title, settings, lobby, faction picker and guide', async ({ page }) => {
    await page.goto('/?client=wasm');
    await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
    await shot(page, size, '01-title');

    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await expect(page.getByRole('button', { name: 'Close settings', exact: true })).toBeVisible();
    await shot(page, size, '02-title-settings');
    await page.getByRole('button', { name: 'Close settings', exact: true }).click();

    await page.getByRole('button', { name: /^How to Play/ }).click();
    await expect(page.getByRole('dialog', { name: 'How to Play', exact: true })).toBeVisible();
    await shot(page, size, '15-guide');
    // The chapters that carry the hold-to-win rule and every faction goal.
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /The victory race/ }).click();
    await page.mouse.move(5, 5);
    await shot(page, size, '15b-guide-victory-race');
    await guide.getByRole('button', { name: /The peoples/ }).click();
    await page.mouse.move(5, 5);
    await shot(page, size, '15c-guide-eight-heirs');
    await page.keyboard.press('Escape');

    await page.getByRole('button', { name: /^Single Player/ }).click();
    await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
    await shot(page, size, '03-lobby');
    // The all-AI control offers Easy, Normal and Hard only (Training is a research tool, 2026-11-21),
    // then the seat list with the level name closed in every seat (nothing clipped).
    const levels = page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true });
    await expect(levels.getByRole('radio')).toHaveCount(3);
    await levels.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03d-lobby-levels');
    await page.getByRole('combobox', { name: 'Seat 2 AI level', exact: true }).scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03e-lobby-seat-list');
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    // Every seat has its own (closed) picker dialog in the DOM; scope to the one that's open.
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    // The picker now changes the seat as you click; remember the default so the tour can put it back.
    const defaultFaction = (await stage.locator('[role=option][aria-selected=true]').getAttribute('data-faction'))!;
    await shot(page, size, '04-faction-picker');
    // A selected faction's full sheet: lore, pips, strengths paired with their price, the unique
    // piece, and how you win/lose. Clicking an entry chooses it; the dialog stays open.
    await page.getByRole('option', { name: /Sparta/ }).click();
    await expect(stage.getByTestId('faction-sheet')).toContainText('Sparta');
    await shot(page, size, '04b-faction-sheet');
    // Every gallery option's sheet, Random included (premium bar: all of them must reach the reference).
    // Clicking a tile chooses it and shows its sheet; the dialog stays open until Close.
    const names = await stage.locator('.fs-tile-name').allTextContents();
    for (const [i, label] of names.entries()) {
      const id = (await stage.getByRole('option').nth(i).getAttribute('data-faction'))!.toLowerCase();
      await stage.getByRole('option').nth(i).click();
      await expect(stage.locator('.fs-sheet-name')).toContainText(label.trim());
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `04c-sheet-${String(i + 1).padStart(2, '0')}-${id}`);
    }
    await stage.locator(`[role=option][data-faction="${defaultFaction}"]`).click();
    await page.keyboard.press('Escape');
    await expect(stage).toHaveCount(0);
    // Map size snaps the empire count: Small trims to 4, Large fills all 8 seat rows.
    const mapSize = page.getByRole('radiogroup', { name: 'Map size', exact: true });
    await mapSize.getByRole('radio', { name: /^Small/ }).click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03f-lobby-small-snapped');
    await mapSize.getByRole('radio', { name: /^Large/ }).click();
    await page.locator('.seat-row:not(.head)').last().scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03g-lobby-large-eight-seats');
  });

  test('LAN: host room, then the host and guest on turn one', async ({ page: host, guestPage: guest }) => {
    for (const p of [host, guest]) await p.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await host.goto('/');
    await host.getByRole('button', { name: /^Multiplayer/ }).click();
    await host.getByRole('button', { name: /^Host Game/ }).click();
    await expect(host.getByRole('heading', { name: 'Host Game', exact: true })).toBeVisible();
    await host.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
    await host.getByLabel('Map seed', { exact: true }).fill('20240926');
    await guest.goto('/');
    await guest.getByRole('button', { name: /^Multiplayer/ }).click();
    await guest.getByRole('button', { name: /^Join Game/ }).click();
    await guest.getByRole('button', { name: 'Take seat', exact: true }).first().click();
    await guest.getByLabel('Your name', { exact: true }).fill('Alice');
    await guest.getByRole('button', { name: /^Ready Lock/ }).click();
    const start = host.getByRole('button', { name: /^Start game/ });
    await expect(start).toBeEnabled();
    // Filling the seed scrolled the setup down to it: show the seat table, as a host sees it on arrival.
    await host.evaluate(() => { (document.activeElement as HTMLElement | null)?.blur(); for (const el of document.querySelectorAll('*')) if (el.scrollTop) el.scrollTop = 0; });
    await shot(host, size, '03b-lan-host-room');
    await shot(guest, size, '03c-lan-guest-room');
    await start.click();
    await expect(host.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(guest.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(host.locator('.loading-screen')).toHaveCount(0);
    await shot(host, size, '05c-lan-host-first-turn');
  });

  test('hot seat: two humans on turn one', async ({ page }) => {
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Multiplayer/ }).click();
    await page.getByRole('button', { name: /^Hot Seat/ }).click();
    await expect(page.getByRole('heading', { name: 'Hot Seat', exact: true })).toBeVisible();
    await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
    await page.getByLabel('Map seed', { exact: true }).fill('9001');
    await expect(page.getByLabel('Seat 2 type', { exact: true })).toHaveValue('human');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await shot(page, size, '05b-hotseat-first-turn');
  });

  test('six empires: the default single-player race on turn one', async ({ page }) => {
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByLabel('Map seed', { exact: true }).fill('424242');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await shot(page, size, '05d-six-empires-first-turn');
  });

  test('scenarios: list, briefing, objective bar, result and feedback', async ({ page }) => {
    await page.addInitScript(() => {
      if (!location.protocol.startsWith('http')) return;
      (globalThis as { legatumScenarioOverrides?: unknown }).legatumScenarioOverrides = { 'war-sparta-nicaea': { objective: { turns: 1 } } };
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
    await page.mouse.move(2, 2);
    await shot(page, size, '32-scenarios');
    for (const [slug, title] of [['celts-reserve', 'The Unwalled Reserve'], ['india-niter', 'An Army Without Niter']] as const) {
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
      await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
      const option = page.getByRole('option', { name: new RegExp(title) });
      await option.scrollIntoViewIfNeeded();
      await page.mouse.move(2, 2);
      await shot(page, size, `32b-scenario-card-${slug}`);
      await option.click();
      await expect(page.getByTestId('lab-briefing')).toBeVisible();
      await page.mouse.move(2, 2);
      await shot(page, size, `32c-scenario-briefing-${slug}`);
    }
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
    await page.getByRole('option', { name: /Nicaea Strike/ }).click();
    await expect(page.getByTestId('lab-briefing')).toBeVisible();
    await page.mouse.move(2, 2);
    await shot(page, size, '33-scenario-briefing');
    await page.getByRole('button', { name: /^Begin Sparta's Nicaea Strike/ }).click();
    await expect(page.getByTestId('scenario-bar')).toBeVisible();
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await shot(page, size, '34-scenario-in-play');
    // The objective's city is ringed on the map; Show on map centres on it.
    await page.getByTestId('scenario-show-target').click();
    await page.waitForTimeout(500);
    await shot(page, size, '34b-scenario-target-on-map');
    await page.getByTestId('scenario-bar').locator('.sb-main').click();
    await expect(page.locator('#scenario-details')).toBeVisible();
    await shot(page, size, '35-scenario-objective-details');
    await page.getByTestId('scenario-bar').locator('.sb-main').click();
    await endPlanning(page);
    await expect(page.getByTestId('scenario-result')).toBeVisible();
    await expect(page.getByTestId('scenario-saved')).not.toHaveText('Saving your run…');
    await page.mouse.move(2, 2);
    await shot(page, size, '36-scenario-result');
    await page.getByTestId('scenario-feel').click();
    await expect(page.getByTestId('scenario-feedback')).toBeVisible();
    await page.mouse.move(2, 2);
    await shot(page, size, '36b-scenario-feedback');
    const feedback = page.getByTestId('scenario-feedback');
    await feedback.getByRole('radiogroup', { name: 'Fun' }).getByRole('radio', { name: '4' }).click();
    await feedback.getByRole('radiogroup', { name: 'Fair' }).getByRole('radio', { name: '3' }).click();
    await feedback.getByRole('radio', { name: 'Too easy' }).click();
    await feedback.locator('textarea').fill('Four Hoplites at the gate felt too easy.');
    await page.mouse.move(2, 2);
    await shot(page, size, '36c-scenario-feedback-filled');
    await page.getByTestId('scenario-save-feedback').click();
    await expect(page.getByRole('heading', { name: 'Thank you' })).toBeVisible();
    await expect(page.getByTestId('scenario-saved')).not.toHaveText('Saving your run…');
    await page.mouse.move(2, 2);
    await shot(page, size, '36d-scenario-thanks');
    // Back on the list the card shows your best result.
    await page.getByRole('button', { name: /^Back to scenarios/ }).click();
    await page.mouse.move(2, 2);
    await shot(page, size, '36e-scenarios-after-play');
  });

  test('scenarios: the objective is marked on the map from turn 1, even in fog', async ({ page }) => {
    await page.addInitScript(() => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
    await page.getByRole('option', { name: /Nicaea Strike/ }).click();
    await expect(page.getByTestId('scenario-briefing-target')).toContainText(/Take \w+: \d+ tiles? /);
    await shot(page, size, '37-scenario-briefing-target');
    await page.getByRole('button', { name: /^Begin Sparta's Nicaea Strike/ }).click();
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await expect(page.getByTestId('scenario-target')).toContainText('Take');
    await page.getByTestId('scenario-show-target').click();
    await page.waitForTimeout(500);
    await shot(page, size, '38-scenario-target-in-fog');
  });
});

// Online play (desktop app): the Multiplayer menu, the lobby list and the game room, against a fake relay
// with the desktop commands mocked.
type Relay = { url: string; close(): Promise<void>; setRooms(r: unknown[]): void; setFail(f: boolean): void; control(c: Record<string, unknown>): void };
const onlineRoom = (over: Record<string, unknown> = {}) => ({
  room: 'room-aaaaaaaa', name: 'Ada', build: '8.9.74', rules_version: 12, protocol: 1,
  players: 2, humans: 2, open_seats: 1, phase: 'staging', turn: 0, updated_ms: 1, ...over,
});

async function desktop(page: Page, relay: Relay | null, host: 'ok' | 'hang' | 'fail' = 'ok') {
  await page.addInitScript(({ info, mode }) => {
    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string) => {
        if (cmd === 'online_info') return info;
        if (cmd === 'online_host_start') {
          if (mode === 'hang') return new Promise(() => {});
          if (mode === 'fail') throw new Error('no');
          return { url: 'http://127.0.0.1:1/', room: null };
        }
        throw new Error(`unmocked ${cmd}`);
      },
    };
  }, { info: { relay: relay ? relay.url : null, build: '8.9.74', rules_version: 12, protocol: 1 }, mode: host });
}

// Plain Playwright test: the shared fixture fails on any console error, and these screens show 404/503 on purpose.
for (const size of SIZES) plain.describe(`online screens ${size.name}`, () => {
  plain.use({ viewport: { width: size.width, height: size.height } });
  let relay: Relay;
  plain.beforeEach(async () => { relay = await startFakeRelay({ rooms: [] }); });
  plain.afterEach(async () => { await relay.close(); });
  const menu = async (page: Page) => {
    await page.goto(`${relay.url}/`);
    await page.getByRole('button', { name: /^Multiplayer/ }).click();
    await expect(page.getByRole('button', { name: /^Host Game/ })).toBeVisible();
  };

  plain('multiplayer menu: normal, no relay, starting, failed', async ({ page }) => {
    await desktop(page, relay);
    await menu(page);
    await shot(page, size, '20-online-menu');
    await expect(page.locator('.title-actions .primary')).toContainText('Join Game');

    const bare = await page.context().newPage();
    await desktop(bare, null);
    await menu(bare);
    await shot(bare, size, '21-online-menu-no-relay');
    await expect(bare.getByRole('button', { name: /^Join Game/ })).toBeDisabled();

    const slow = await page.context().newPage();
    await desktop(slow, relay, 'hang');
    await menu(slow);
    await slow.getByRole('button', { name: /^Host Game/ }).click();
    await expect(slow.getByText('Opening your game…')).toBeVisible();
    await expect(slow.getByRole('button', { name: /^Join Game/ })).toBeDisabled();
    await shot(slow, size, '22-online-menu-starting');

    const bad = await page.context().newPage();
    await desktop(bad, relay, 'fail');
    await menu(bad);
    await bad.getByRole('button', { name: /^Host Game/ }).click();
    await expect(bad.getByText('Could not open your game.')).toBeVisible();
    await expect(bad.getByRole('button', { name: /^Host Game/ })).toHaveClass(/primary/);
    await shot(bad, size, '23-online-menu-failed');
  });

  plain('lobby: loading, list with mismatch and full rows, joining, join error', async ({ page }) => {
    relay.setRooms([
      onlineRoom(),
      onlineRoom({ room: 'room-bbbbbbbb', name: 'Bo from Zürich', build: '8.9.70', phase: 'playing', turn: 14, players: 3 }),
      onlineRoom({ room: 'room-cccccccc', name: 'Cy', phase: 'playing', turn: 31, players: 4, open_seats: 0 }),
      onlineRoom({ room: 'room-dddddddd', name: '', rules_version: 11 }),
    ]);
    relay.control({ lobbyDelay: 1500 });
    await desktop(page, relay);
    await menu(page);
    await page.getByRole('button', { name: /^Join Game/ }).click();
    await expect(page.getByText('Looking for games…')).toBeVisible();
    await shot(page, size, '30-lobby-loading');
    relay.control({ lobbyDelay: 0 });
    await expect(page.locator('.online-room')).toHaveCount(4, { timeout: 8000 });
    await expect(page.getByText('4 players · Full')).toBeVisible();
    await expect(page.getByText('Unnamed game')).toBeVisible();
    await page.mouse.move(5, size.height - 5);
    await shot(page, size, '31-lobby-list');

    relay.control({ statusDelay: 2500 });
    await page.locator('.online-room').first().getByRole('button', { name: /^Join/ }).click();
    await expect(page.getByRole('button', { name: 'Joining…' })).toBeVisible();
    await shot(page, size, '32-lobby-joining');

    // No host behind the room: the failure shows in the row that failed.
    await expect(page.locator('.online-room').first().getByRole('alert')).toContainText('The host is not connected right now.', { timeout: 10_000 });
    await shot(page, size, '33-lobby-join-error');
    await page.keyboard.press('Escape');
    await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  });

  plain('game room with the host away stays on screen', async ({ page }) => {
    plain.setTimeout(90_000);
    const root = fileURLToPath(new URL('../../', import.meta.url));
    const probe = createServer();
    await new Promise<void>((r) => probe.listen(0, '127.0.0.1', r));
    const port = (probe.address() as { port: number }).port;
    await new Promise<void>((r) => probe.close(() => r()));
    const lanBase = `http://127.0.0.1:${port}`;
    const lan: ChildProcess = spawn('node', ['scripts/fake-lan-server.mjs', '--port', String(port)], { cwd: root, stdio: 'pipe' });
    const live = await (async () => {
      await expect.poll(() => fetch(`${lanBase}/api/status`).then((r) => r.status).catch(() => 0)).toBe(200);
      const seat = (kind: string, name: string) => ({ kind, name, faction: 'Random', team: 0, difficulty: 'Normal' });
      await fetch(`${lanBase}/api/staging`, {
        method: 'POST', headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ seed: '1', width: 40, height: 24, turn_limit: null, seats: [seat('host', 'Ada'), seat('open', 'Player 2'), seat('ai', 'AI 1')] }),
      });
      return startFakeRelay({ upstream: lanBase, rooms: [onlineRoom({ room: 'room-live' })] }) as Promise<Relay>;
    })();
    try {
      await desktop(page, live);
      await page.goto(`${live.url}/`);
      await page.getByRole('button', { name: /^Multiplayer/ }).click();
      await page.getByRole('button', { name: /^Join Game/ }).click();
      await page.locator('.online-room').getByRole('button', { name: /^Join/ }).click();
      await expect(page.getByRole('heading', { name: 'Join Game' })).toBeVisible({ timeout: 10_000 });
      await expect(page.getByRole('button', { name: /^Take seat/ }).first()).toBeVisible();
      await shot(page, size, '40-game-room');
      live.control({ away: true });
      const notice = page.locator('.host-away');
      await expect(notice).toContainText('The host is not connected right now.', { timeout: 15_000 });
      await expect(notice).toContainText('Waiting for the host to come back…');
      const box = (await notice.boundingBox())!;
      expect(box.y + box.height, 'the notice is inside the window').toBeLessThanOrEqual(size.height);
      await shot(page, size, '41-game-room-host-away');
    } finally {
      await live.close();
      lan.kill();
    }
  });
});

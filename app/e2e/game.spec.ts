import { revealSidebar } from './sidebarAssertions';
import { dismissInterstitials, endPlanning } from './turnActions';
import { chooseCapitalProduction } from './opening';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';

async function configureLobby(page: Page, seed: string) {
  const small = page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ });
  await small.click();
  await expect(small).toBeChecked();
  // Choosing Small trims to four empires, which keeps the real AI/WASM turns quick.
  const hard = page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true }).getByRole('radio', { name: /^Hard Ruthless/ });
  await hard.click();
  await expect(hard).toBeChecked();
  for (const level of await page.getByRole('combobox', { name: /^Seat \d+ AI level$/ }).all()) await expect(level).toHaveValue('Hard');
  await page.getByLabel('Map seed', { exact: true }).fill(seed);
}

async function expectTurn(page: Page, turn: number) {
  const counter = page.getByTestId('turn-number');
  await expect(counter).toBeVisible();
  await expect(counter).toHaveText(String(turn));
}

async function startSolo(page: Page, seed: string) {
  await page.goto('/?client=wasm');
  await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
  await configureLobby(page, seed);
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expectTurn(page, 1);
}

async function endTurn(page: Page, nextTurn: number) {
  // This is the UI's explicit way to leave optional unit/research choices idle.
  await dismissInterstitials(page);
  await endPlanning(page);
  await expectTurn(page, nextTurn);
  await dismissInterstitials(page);
}

/** Celebrations hold the report until they end; a player's Escape skips them, as here. */
async function skipCelebrations(page: Page) {
  if (await page.locator('.moment-card, [data-celebrate-delta]').count()) await page.keyboard.press('Escape');
}

function turnReport(page: Page) {
  return page.getByRole('complementary').filter({ has: page.getByRole('heading', { name: 'What happened', exact: true }) });
}

async function autosave(page: Page): Promise<{ client: string; turn: number; game: string; savedAt: string }> {
  // Read the actual persisted artifact; never seed or drive the game via its store.
  return page.evaluate(() => JSON.parse(localStorage.getItem('legatum.autosave.v1') ?? 'null'));
}

test('Single Player: choose capital production and play six turns with VP visible', async ({ page }) => {
  test.setTimeout(90_000);
  await startSolo(page, '424242');
  const first = await chooseCapitalProduction(page);
  await revealSidebar(page, 'turn-more');
  await expect(page.getByRole('button', { name: /^Pending orders/ })).toContainText('1');
  await page.getByRole('button', { name: /^Pending orders/ }).click();
  await expect(page.getByText(new RegExp(`: 1\\. ${first}$`))).toBeVisible();
  await page.getByRole('button', { name: /^Pending orders/ }).click();
  await endTurn(page, 2);
  if (await page.getByRole('button', { name: 'Close report', exact: true }).isVisible()) await page.getByRole('button', { name: 'Close report', exact: true }).click();

  await page.getByRole('button', { name: /^Research/ }).click();
  await page.locator('.era-card[data-tech="Irrigation"]').getByTestId('research-now').click();
  await expect(page.locator('.research-queue li[data-tech="Irrigation"]')).toHaveCount(1);
  await page.getByRole('button', { name: 'Close research', exact: true }).click();
  const city = page.locator('.city-list-items button').first();
  await revealSidebar(page, 'turn-more');
  if (!await city.isVisible()) await page.locator('.city-list summary').click();
  await city.click();
  await expect(page.getByRole('button', { name: `Build ${first}`, exact: true })).toHaveAttribute('aria-pressed', 'true');

  const vp = page.getByRole('region', { name: 'Victory race', exact: true });
  for (let turn = 3; turn <= 7; turn++) {
    await endTurn(page, turn);
    await expect(vp).toBeVisible();
    await expect(vp.locator('.race-headline')).toBeVisible();
    await expect(vp).toContainText('/10');
    if (turn === 3) await expect(page.getByRole('button', { name: `Build ${first}`, exact: true })).toHaveAttribute('aria-pressed', 'true');
  }
  expect(await autosave(page)).toMatchObject({ client: 'wasm', turn: 7 });
});

test('Single Player: the turn-one capital build is reachable at 1000×600', async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 600 });
  await startSolo(page, '424242');
  const build = page.locator('.city-panel .production-choice:not(.unavailable) .prod').first();
  await expect(build).toBeInViewport();
  await expect.poll(() => build.evaluate((button) => {
    const box = button.getBoundingClientRect();
    return button.contains(document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2));
  })).toBe(true);
  await chooseCapitalProduction(page);
  await revealSidebar(page, 'turn-more');
  await expect(page.getByRole('button', { name: /^Pending orders/ })).toContainText('1');
});

test('Autosave: reload and Continue restore the same turn and exact game state', async ({ page }) => {
  await startSolo(page, '131313');
  await chooseCapitalProduction(page);
  await endTurn(page, 2);
  await endTurn(page, 3);
  const before = await autosave(page);
  expect(before).toMatchObject({ client: 'wasm', turn: 3 });
  expect(before.game.length).toBeGreaterThan(0);

  await page.reload();
  const resume = page.getByRole('button', { name: /^Continue/ });
  await expect(resume).toBeEnabled();
  await expect(resume).toContainText('turn 3');
  await resume.click();
  await expectTurn(page, before.turn);

  // Continue writes a fresh autosave after loading. Wait for that ownership
  // handoff, then compare engine JSON as a string to preserve its u64 values.
  await expect.poll(async () => (await autosave(page)).savedAt).not.toBe(before.savedAt);
  const resumed = await autosave(page);
  expect(resumed.game).toBe(before.game);
  expect(resumed.turn).toBe(before.turn);
  await endTurn(page, 4);
  expect(await autosave(page)).toMatchObject({ client: 'wasm', turn: 4 });
});

test('Hot Seat: both humans plan before the shared turn resolves', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Multiplayer/ }).click();
  await page.getByRole('button', { name: /^Hot Seat/ }).click();
  await expect(page.getByRole('heading', { name: 'Hot Seat', exact: true })).toBeVisible();
  await configureLobby(page, '9001');
  await expect(page.getByLabel('Seat 2 type', { exact: true })).toHaveValue('human');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expectTurn(page, 1);

  // Each local seat has a selection button (faction crest text, the player's name, then the
  // order count) and a separate Ready toggle named "<player>: ready".
  await revealSidebar(page, 'turn-more');
  const firstSeat = page.getByRole('button', { name: /\bPlayer 1 \d+ orders?$/ });
  const secondSeat = page.getByRole('button', { name: /\bPlayer 2 \d+ orders?$/ });
  const firstReady = page.getByRole('button', { name: 'Player 1: ready', exact: true });
  const secondReady = page.getByRole('button', { name: 'Player 2: ready', exact: true });
  await expect(firstSeat).toHaveAttribute('aria-pressed', 'true');
  await expect(secondSeat).toHaveAttribute('aria-pressed', 'false');
  await chooseCapitalProduction(page);
  await endPlanning(page);
  // Hot seat privacy: Player 2's empire stays covered until Player 2 takes the device.
  const pass = page.getByRole('dialog', { name: 'Pass to Player 2' });
  await expect(pass).toBeVisible();
  await expect(pass.getByRole('button', { name: "I'm Player 2: start planning" })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(pass).toHaveCount(0);
  await expect(secondSeat).toHaveAttribute('aria-pressed', 'true');
  await expect(firstSeat).toHaveAttribute('aria-pressed', 'false');
  await expect(firstSeat).toContainText('1 order');
  await expect(firstReady).toHaveAttribute('aria-pressed', 'true');
  await expect(firstReady).toHaveText('Ready');
  await expect(secondReady).toHaveAttribute('aria-pressed', 'false');
  await expectTurn(page, 1);

  await chooseCapitalProduction(page);
  await expect(secondSeat).toContainText('1 order');
  await endTurn(page, 2);
  await skipCelebrations(page);
  // Both capital production orders resolved in the same shared turn.
  expect(await page.evaluate(() => {
    const s = (window as unknown as { legatum: typeof import('../src/store').useGame }).legatum.getState();
    return s.views?.slice(0, 2).map((v) => v.cities.find((c) => c.owner === v.me)?.detail?.producing ?? null);
  })).toEqual([expect.any(Object), expect.any(Object)]);
  for (const ready of [firstReady, secondReady]) {
    await expect(ready).toHaveAttribute('aria-pressed', 'false');
    await expect(ready).toHaveText('Planning');
  }
});

// Online co-op end to end: the host's tab runs the game (the WASM host) and reaches the guest's browser
// through the real relay under wrangler dev. No game server anywhere.
test('Online: host in one browser, a guest joins from the game list, both Ready reach turn 2', async ({ page: host, guestPage: guest, relay }) => {
  test.setTimeout(150_000);
  const tunnel: string[] = [];
  host.on('websocket', (ws) => tunnel.push(ws.url()));
  await host.goto('/');
  await host.getByRole('button', { name: /^Multiplayer/ }).click();
  await host.getByLabel('Your name in the game list').fill('Ada');
  await host.getByRole('button', { name: /^Host Game/ }).click();
  await expect(host.getByRole('heading', { name: 'Host Game', exact: true })).toBeVisible();
  // The host holds one tunnel to the relay, and plays its own seat inside the tab.
  expect(tunnel.filter((u) => u.startsWith(relay.replace(/^http/, 'ws')))).toEqual([expect.stringMatching(/\/rooms\/[^/]+\/host\?key=/)]);
  await configureLobby(host, '20240926');
  const start = host.getByRole('button', { name: /^Start game/ });
  // The room is open and synced, but the open online seat blocks the start.
  await expect(host.getByText('Waiting for every online seat to be taken and Ready.', { exact: true })).toBeVisible();
  await expect(start).toBeDisabled();

  await guest.goto('/');
  await guest.getByRole('button', { name: /^Multiplayer/ }).click();
  await guest.getByRole('button', { name: /^Join Game/ }).click();
  const row = guest.locator('.online-room').filter({ hasText: 'Ada' });
  await expect(row).toHaveCount(1, { timeout: 20_000 });
  await row.getByRole('button', { name: /^Join/ }).click();
  await expect(guest.getByRole('heading', { name: 'Join Game', exact: true })).toBeVisible();

  // The guest sees the host's settings: Seat 2 is the one open seat.
  await expect(guest.getByLabel('Map seed', { exact: true })).toHaveValue('Hidden');
  await guest.getByRole('row').nth(2).getByRole('button', { name: 'Take seat', exact: true }).click();
  await guest.getByLabel('Your name', { exact: true }).fill('Alice');
  await guest.getByRole('button', { name: /^Ready Lock/ }).click();
  await expect(guest.getByRole('button', { name: /^Ready/ })).toBeVisible();
  await expect(start).toBeEnabled();
  await start.click();
  await expectTurn(host, 1);
  await expectTurn(guest, 1);
  await expect(host.getByTestId('online-seat-1')).toContainText('Alice');
  await expect(host.getByTestId('online-seat-1')).toContainText('online');
  await expect(guest.getByTestId('online-seat-0')).toContainText('online');

  await chooseCapitalProduction(host);
  await chooseCapitalProduction(guest);
  await endPlanning(host);
  await expect(host.getByRole('button', { name: /^Waiting for Alice/ })).toBeVisible();
  await expect(guest.getByTestId('online-seat-0')).toContainText('Ready');
  await expectTurn(host, 1);
  await expectTurn(guest, 1);
  await endPlanning(guest);
  await expectTurn(host, 2);
  await expectTurn(guest, 2);
  for (const page of [host, guest]) {
    await dismissInterstitials(page);
    await skipCelebrations(page);
    // Both team capitals remain visible; each page sees its own build in progress.
    await expect(page.locator('.city-list-items button')).toHaveCount(2);
    expect(await page.evaluate(() => {
      const s = (window as unknown as { legatum: typeof import('../src/store').useGame }).legatum.getState();
      const v = s.views![s.seat];
      return v.cities.find((c) => c.owner === v.me)?.detail?.producing ?? null;
    })).toEqual(expect.any(Object));
    await expect(page.getByTestId('online-seat-0')).toContainText('Planning');
    await expect(page.getByTestId('online-seat-1')).toContainText('Planning');
    await expect(page.getByRole('region', { name: 'Victory race', exact: true })).toBeVisible();
  }
});


test('Research: a queue of current-era technologies advances through saved turns', async ({ page }, testInfo) => {
  test.setTimeout(120_000);
  await startSolo(page, '737373');
  await chooseCapitalProduction(page);
  await endTurn(page, 2);
  if (await page.getByRole('button', { name: 'Close report', exact: true }).isVisible()) await page.getByRole('button', { name: 'Close report', exact: true }).click();
  await page.locator('.topbar .research-chip').click();
  const dialog = page.getByRole('dialog', { name: 'Research', exact: true });
  await dialog.locator('.era-card[data-tech="BronzeWorking"]').getByTestId('research-now').click();
  await dialog.locator('.era-card[data-tech="Writing"]').getByTestId('queue-add').click();
  await expect(dialog.locator('.era-card[data-tech="BronzeWorking"] [data-testid="era-researching"]')).toBeVisible();
  await expect(dialog.locator('.era-card[data-tech="Writing"] [data-testid="research-order-badge"]')).toHaveText('Queued #2');
  await expect(dialog.getByLabel('Research queue', { exact: true })).toContainText('Writing');
  // A third tech queued, then dropped again from its card.
  await dialog.locator('.era-card[data-tech="Irrigation"]').getByTestId('queue-add').click();
  await expect(dialog.locator('.era-card[data-tech="Irrigation"] [data-testid="research-order-badge"]')).toHaveText('Queued #3');
  await dialog.locator('.research-queue li[data-tech="Irrigation"]').getByRole('button', { name: 'Remove Irrigation from the queue' }).click();
  await expect(dialog.getByTestId('research-order-badge')).toHaveCount(1);
  // The population-1 capital already produces +2 Science, so the queue has a finite ETA.
  await expect(dialog.locator('.research-queue li[data-tech="Writing"] [data-testid="queue-arrival"]')).toHaveText(/^Turn \d+ · in \d+ turns?$/);
  await page.screenshot({ path: testInfo.outputPath('research-queue.png') });
  await dialog.getByRole('button', { name: 'Close research', exact: true }).click();
  await expect(page.locator('.research-slot')).toContainText('Writing');
  await endTurn(page, 3);
  await expect.poll(async () => JSON.parse((await autosave(page)).game).players[0].research_queue).toEqual(['BronzeWorking', 'Writing']);
  await page.reload();
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expectTurn(page, 3);
  let turn = 3;
  while (turn < 45 && JSON.parse((await autosave(page)).game).players[0].researching !== 'Writing') {
    await endTurn(page, ++turn);
    await expect.poll(async () => (await autosave(page)).turn).toBe(turn);
  }
  expect(JSON.parse((await autosave(page)).game).players[0].researching).toBe('Writing');
  await page.locator('.topbar .research-chip').click();
  await expect(dialog.locator('.era-card.known[data-tech="BronzeWorking"]')).toBeVisible();
  await expect(dialog.locator('.era-card[data-tech="Writing"] [data-testid="era-researching"]')).toBeVisible();
  await expect(dialog.locator('.era-card[data-tech="Writing"] [data-testid="era-when"]')).toContainText('Turn');
  await expect(dialog.getByTestId('research-order-badge')).toHaveCount(0);
  await dialog.getByRole('button', { name: 'Close research', exact: true }).click();
  while (turn < 65 && JSON.parse((await autosave(page)).game).players[0].researching !== null) {
    await endTurn(page, ++turn);
    await expect.poll(async () => (await autosave(page)).turn).toBe(turn);
  }
  await skipCelebrations(page);
  await expect(turnReport(page)).toContainText('Research queue finished: choose next');
  expect(JSON.parse((await autosave(page)).game).players[0].research_queue).toEqual([]);
});

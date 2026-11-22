// The hosted web build ("Play in your browser"), played the way a visitor gets it: app/dist from
// scripts/build-web.sh on a static server with no backend (serve.mjs). Everything a visitor can
// reach must work without a server: any console error, failed request, request to another host or
// request for the game server's /api or /ws fails the test. A build with LEGATUM_GAMES_URL also sends each
// game to that upload Worker (src/platform/gameUpload.ts): the test answers those itself, so a test game never
// lands in the real bucket, and checks that the game was sent.
import { readFileSync } from 'node:fs';
import { expect, test, type Page } from '@playwright/test';
import { dismissInterstitials, endPlanning } from '../e2e/turnActions';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';

type GameWindow = Window & {
  legatum: typeof useGame;
  legatumMap: { toScreen(hex: Hex): { x: number; y: number }; redraw(): void };
};

/** The game-upload Worker the build under test was built with, if any (scripts/build-web.sh). */
const gamesOrigin = process.env.LEGATUM_GAMES_URL ? new URL(process.env.LEGATUM_GAMES_URL).origin : null;

/** scripts/build-web.sh bakes in the live relay unless LEGATUM_RELAY_URL=none; then the title offers hot seat only. */
const relayBuilt = process.env.LEGATUM_RELAY_URL !== 'none';

const version = (JSON.parse(readFileSync(new URL('../package.json', import.meta.url), 'utf8')) as { version: string }).version;

/** What goes wrong on a static host: collected for the whole test, asserted at the end. */
function watch(page: Page, origin: string) {
  const problems: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') problems.push(`console: ${message.text()} (${message.location().url})`);
  });
  page.on('pageerror', (error) => problems.push(`uncaught: ${error.message}`));
  page.on('requestfailed', (request) => problems.push(`failed: ${request.url()} ${request.failure()?.errorText}`));
  page.on('request', (request) => {
    const url = new URL(request.url());
    if (url.protocol === 'data:' || url.protocol === 'blob:') return;
    if (url.origin === gamesOrigin) return;
    if (url.origin !== origin) problems.push(`request to another host: ${request.url()}`);
    else if (/^\/(api|ws)(\/|$)/.test(url.pathname)) problems.push(`request for the game server: ${url.pathname}`);
  });
  page.on('response', (response) => {
    if (response.status() >= 400) problems.push(`HTTP ${response.status()}: ${response.url()}`);
  });
  return problems;
}

/** Close whatever the turn opened (alarms, the report, research, celebrations), as Escape does for a player. */
async function clear(page: Page) {
  await dismissInterstitials(page);
  for (let i = 0; i < 6; i++) {
    const open = await page.locator('dialog[open], .moment-card, [data-celebrate-delta], [data-testid="alarm-screen"]').count();
    const report = page.getByRole('button', { name: 'Close report', exact: true });
    if (await report.isVisible()) await report.click();
    else if (open) await page.keyboard.press('Escape');
    else break;
    await page.waitForTimeout(150);
  }
}

async function endTurn(page: Page) {
  const counter = page.getByTestId('turn-number');
  const turn = Number(await counter.textContent());
  await clear(page);
  await endPlanning(page);
  await expect(counter).toHaveText(String(turn + 1), { timeout: 60_000 });
  await clear(page);
}

/** The human's cities and Settler, read from the same view the screen draws. */
const empire = (page: Page) => page.evaluate(() => {
  const state = (window as unknown as GameWindow).legatum.getState();
  const view = state.views![0];
  const me = view.me;
  const cities = view.cities.filter((c) => c.owner === me);
  const settler = view.units.find((u) => u.owner === me && u.kind === 'Settler') ?? null;
  const producing = cities[0]?.detail?.producing;
  const sites = settler && state.client!.settlementSites ? state.client!.settlementSites(view, settler.id, []) : [];
  const reachable = sites.filter((s) => s.travel_turns !== null).sort((a, b) => a.travel_turns! - b.travel_turns! || b.score - a.score);
  return {
    cities: cities.length,
    capital: cities[0]?.id ?? null,
    buildingSettler: producing?.kind === 'Unit' && producing.unit === 'Settler',
    settler: settler && { id: settler.id, pos: settler.pos },
    site: reachable[0]?.pos ?? null,
    scouts: view.units.filter((u) => u.owner === me && u.kind === 'Scout').map((u) => u.id),
  };
});

const ENGINE = /\/assets\/legatum_bg-[\w-]+\.wasm$/;

test('a slow engine download shows the loading plate, then the title', async ({ page, baseURL }) => {
  const problems = watch(page, new URL(baseURL!).origin);
  // Hold the engine until the plate has been seen, so the check never races a fast connection.
  let release!: () => void;
  const held = new Promise<void>((resolve) => { release = resolve; });
  await page.route(ENGINE, async (route) => { await held; await route.continue(); });
  await page.goto('/');
  const plate = page.getByTestId('boot-plate');
  await expect(plate).toBeVisible();
  await expect(plate).toContainText('LEGATUM');
  await expect(plate).toContainText('Loading the game…');
  release();
  await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible({ timeout: 60_000 });
  await expect(plate).toHaveCount(0);
  expect(problems).toEqual([]);
});

test('an engine that cannot download shows Reload, never a blank page', async ({ page }) => {
  await page.route(ENGINE, (route) => route.abort());
  await page.goto('/');
  const card = page.getByTestId('crash-screen');
  await expect(card).toBeVisible({ timeout: 60_000 });
  await expect(card.getByRole('heading', { name: 'The game could not load' })).toBeVisible();
  const reload = card.getByRole('button', { name: /^Reload/ });
  await expect(reload).toBeFocused();
  // With the connection back, Return reloads into the game.
  await page.unroute(ENGINE);
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible({ timeout: 60_000 });
});

test('a visitor plays: title, a people, turns, a second city, save and load, with no server', async ({ page, baseURL, request }) => {
  test.setTimeout(300_000);
  const problems = watch(page, new URL(baseURL!).origin);
  const uploads: string[] = [];
  if (gamesOrigin) {
    await page.route(`${gamesOrigin}/**`, (route) => {
      if (route.request().method() === 'PUT') uploads.push(new URL(route.request().url()).pathname);
      return route.fulfill({
        status: route.request().method() === 'OPTIONS' ? 204 : 200,
        headers: { 'Access-Control-Allow-Origin': '*', 'Access-Control-Allow-Methods': 'PUT, OPTIONS', 'Access-Control-Allow-Headers': 'Content-Type' },
        body: route.request().method() === 'OPTIONS' ? '' : '{}',
      });
    });
  }

  // The host: security headers, the engine's MIME type, a real 404, the link-preview card.
  const index = await request.get('/');
  expect(index.headers()['content-security-policy']).toContain("script-src 'self' 'wasm-unsafe-eval'");
  expect(index.headers()['x-content-type-options']).toBe('nosniff');
  const html = await index.text();
  expect(html).toContain('<meta name="twitter:card" content="summary_large_image" />');
  const image = /<meta property="og:image" content="(https:\/\/[^"]+)\/og\.jpg" \/>/.exec(html);
  expect(image, 'og:image is an absolute https address').not.toBeNull();
  expect(html).not.toContain('%SITE_URL%');
  expect((await request.get('/og.jpg')).headers()['content-type']).toBe('image/jpeg');
  const wasm = /legatum_bg-[\w-]+\.wasm/.exec(await (await request.get(`/${/assets\/index-[\w-]+\.js/.exec(html)![0]}`)).text());
  const engine = await request.get(`/assets/${wasm![0]}`);
  expect(engine.headers()['content-type']).toBe('application/wasm');
  expect(engine.headers()['cache-control']).toContain('immutable');
  const missing = await request.get('/no-such-page');
  expect(missing.status()).toBe(404);
  expect(await missing.text()).toContain('There is nothing at this address');
  expect((await request.get('/api/status')).status()).toBe(404);
  expect(await (await request.get('/THIRD_PARTY_LICENSES.txt')).text()).toContain('SIL OPEN FONT LICENSE Version 1.1');

  // Title: the build label carries the app version; online play is offered when the build has a relay.
  await page.addInitScript(() => localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] })));
  await page.goto('/');
  await expect(page).toHaveTitle(/^Legatum/);
  await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
  await expect(page.locator('.title-foot')).toContainText(`Version ${version}`);
  await page.getByRole('button', { name: /^Multiplayer/ }).click();
  if (relayBuilt) {
    await expect(page.getByRole('button', { name: /^Host Game/ })).toBeEnabled();
    await expect(page.getByRole('button', { name: /^Join Game/ })).toBeEnabled();
  } else {
    await expect(page.getByRole('button', { name: /^Host Game/ })).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Join Game/ })).toHaveCount(0);
  }
  await expect(page.getByRole('button', { name: /^Hot Seat/ })).toBeEnabled();
  await page.getByRole('button', { name: /Back/ }).click();

  // The licence notices open inside the Credits.
  await page.getByRole('button', { name: /^How to Play/ }).click();
  await page.getByRole('navigation', { name: 'Guide chapters' }).getByRole('button', { name: /Credits/ }).click();
  await expect(page.getByText('not affiliated with or endorsed by Take-Two Interactive or Firaxis Games')).toBeVisible();
  await page.getByText('Open-source licences', { exact: true }).click();
  await expect(page.locator('.guide-licences pre')).toContainText('Legatum: third-party licence notices');
  await page.getByRole('button', { name: 'Close guide' }).click();

  // Single player on a small map, as Athens.
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
  await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  await page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true }).getByRole('radio', { name: /^Easy/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
  const picker = page.locator('dialog[data-testid="faction-stage"][open]');
  await picker.getByRole('option', { name: /Athens/ }).click();
  await picker.getByRole('button', { name: 'Close faction picker' }).click();
  await expect(page.getByRole('button', { name: /^Seat 1 faction:/ })).toContainText('Athens');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });

  // Play until a second city stands: grow the capital, build a Settler, walk it to a site, found.
  const capital = (await empire(page)).capital!;
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), capital);
  await page.locator('.city-panel .production-choice:not(.unavailable) .prod').first().click();
  let founded = false;
  for (let turn = 1; turn <= 60 && !founded; turn++) {
    await clear(page);
    const now = await empire(page);
    if (now.cities >= 2) { founded = true; break; }
    // Scouts explore on their own, so the Settler has somewhere known to go.
    for (const id of now.scouts) {
      await page.evaluate((unit) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id: unit }, true), id);
      const explore = page.getByRole('button', { name: /^Auto-explore/ }).first();
      if (await explore.isVisible() && await explore.isEnabled()) await explore.click();
    }
    if (now.settler) {
      await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }, true), now.settler.id);
      const found = page.getByRole('button', { name: /^Found City/ }).first();
      if (await found.isVisible() && await found.isEnabled()) await found.click();
      else if (now.site) {
        // Right-click the site on the map: the Settler's move order, as a player gives it.
        const at = await page.evaluate((pos) => {
          const win = window as unknown as GameWindow;
          win.legatum.getState().jumpTo(pos);
          win.legatumMap.redraw();
          return win.legatumMap.toScreen(pos);
        }, now.site);
        await page.mouse.click(at.x, at.y, { button: 'right' });
      }
    } else if (!now.buildingSettler && now.capital !== null) {
      await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), now.capital);
      const settler = page.getByRole('button', { name: 'Build Settler', exact: true });
      if (await settler.isVisible() && await settler.getAttribute('aria-disabled') !== 'true') await settler.click();
    }
    await endTurn(page);
  }
  expect(founded, 'a second city was founded within 60 turns').toBe(true);
  const turn = Number(await page.getByTestId('turn-number').textContent());
  expect(turn).toBeGreaterThan(5);

  // Save to file is a download in a browser; Load Game reads it back to the same turn.
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Save to file…', exact: true }).click();
  const file = await (await download).path();
  expect(readFileSync(file, 'utf8')).toContain('"legatum-save"');

  // Orders entered this turn survive a reload: they are kept a moment after each change, beside the autosave.
  await clear(page);
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), (await empire(page)).capital!);
  const choices = page.locator('.city-panel .production-choice:not(.unavailable) .prod');
  const orders = () => page.evaluate(() => JSON.stringify((window as unknown as GameWindow).legatum.getState().orders[0]));
  for (let i = (await choices.count()) - 1; i >= 0 && (await orders()) === '[]'; i--) await choices.nth(i).click();
  const entered = await orders();
  expect(entered, 'an order was entered this turn').not.toBe('[]');
  await expect.poll(() => page.evaluate(() => new Promise<boolean>((resolve) => {
    const open = indexedDB.open('legatum-autosave-v1');
    open.onerror = () => resolve(false);
    open.onsuccess = () => {
      const db = open.result;
      const get = db.transaction('orders').objectStore('orders').get('current');
      get.onsuccess = () => { db.close(); resolve(!!get.result?.text); };
      get.onerror = () => { db.close(); resolve(false); };
    };
  })), { message: 'the pending orders are saved' }).toBe(true);

  // The autosave survives a reload: Continue returns to the same turn with both cities, and the orders.
  await page.reload();
  const resume = page.getByRole('button', { name: /^Continue/ });
  await expect(resume).toBeEnabled();
  await expect(resume).toContainText(`turn ${turn}`);
  await resume.click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 60_000 });
  expect((await empire(page)).cities).toBe(2);
  expect(await orders(), 'the orders entered before the reload are still there').toBe(entered);
  await endTurn(page);

  // A newest save that will not open is skipped: Continue offers the turn before it.
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn + 1));
  await expect.poll(() => page.evaluate(() => new Promise<number>((resolve) => {
    const open = indexedDB.open('legatum-autosave-v1');
    open.onsuccess = () => {
      const db = open.result;
      const all = db.transaction('saves').objectStore('saves').getAll();
      all.onsuccess = () => { db.close(); resolve((all.result as unknown[]).length); };
    };
  })), { message: 'the new turn is saved beside the old one' }).toBeGreaterThan(1);
  await page.evaluate(() => new Promise<void>((resolve, reject) => {
    const open = indexedDB.open('legatum-autosave-v1');
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const db = open.result;
      const tx = db.transaction('saves', 'readwrite');
      const store = tx.objectStore('saves');
      const all = store.getAll();
      all.onsuccess = () => {
        const newest = (all.result as { turn: number }[]).sort((a, b) => b.turn - a.turn)[0];
        store.put({ ...newest, text: '{"format":"legatum-save"' });
      };
      tx.oncomplete = () => { db.close(); resolve(); };
      tx.onerror = () => reject(tx.error);
    };
  }));
  await page.reload();
  await expect(page.locator('.title-notice')).toContainText(`Your latest save (turn ${turn + 1}) could not be opened. Continue opens turn ${turn} instead.`);
  await expect(resume).toContainText(`turn ${turn}`);
  await resume.click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 60_000 });
  expect((await empire(page)).cities).toBe(2);
  await endTurn(page);

  // Load the downloaded file from the title screen.
  await page.reload();
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: /^Load Game/ }).click();
  await (await chooser).setFiles(file);
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 60_000 });
  expect((await empire(page)).cities).toBe(2);

  expect(problems, 'no console errors, failed requests, or requests beyond the static files').toEqual([]);
  if (gamesOrigin) expect(uploads.length, 'the game was sent to the upload Worker').toBeGreaterThan(0);
});

import { spawn } from 'node:child_process';
import { access, mkdtemp, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { test as base, expect, type BrowserContext, type Page } from '@playwright/test';
// @ts-expect-error plain .mjs helper without types
import { startStaticServer } from '../e2e-hosted/serve.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));

/**
 * The relay the e2e build talks to: the real one (infra/relay) under `wrangler dev`, started by the
 * `relay` fixture for the tests that play online. Build the app for it with
 *   LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh
 */
// LEGATUM_E2E_RELAY points the online tests at a deployed relay instead (a live check before a release):
//   LEGATUM_E2E_RELAY=https://relay.playlegatum.com, with app/dist built for that relay.
export const E2E_RELAY = process.env.LEGATUM_E2E_RELAY ?? 'http://127.0.0.1:8787';
const BUILD_HINT = `Build first: LEGATUM_RELAY_URL=${E2E_RELAY} scripts/build-web.sh (needs the WASM engine; add --skip-wasm to reuse it).`;

/**
 * A new game opens with the people's introduction (celebrate/PeopleIntro.tsx), which waits for Begin. Every test
 * starts without it, so a test about anything else never has to press it; e2e/tour/people-intro.spec.ts turns it back on.
 */
export const skipPeopleIntro = () => {
  try { if (!(localStorage.getItem('legatum.intro.v1') === null)) {} else { localStorage.setItem('legatum.intro.v1', 'off'); } } catch { /* storage is blocked: the intro shows */ }
  // Research opens on the tech tree; the specs written for the era cards start on "This era".
  // e2e/research-tree.spec.ts sets the tree back.
  try { if (!(localStorage.getItem('legatum.research-view.v1') === null)) {} else { localStorage.setItem('legatum.research-view.v1', 'era'); } } catch { /* storage is blocked: the tree shows */ }
};

/**
 * Mid-game fixtures leave other empires unmet, so their first resolved turn meets them and opens the first-contact
 * plate (celebrate/MeetingTakeover.tsx), which waits for Continue. A test about anything else has it closed at once,
 * as a player would; e2e/tour/meeting.spec.ts keeps it with `test.use({ meetings: 'show' })`.
 */
export const closeMeetingPlates = () => {
  const close = () => {
    for (const plate of document.querySelectorAll<HTMLElement>('[data-testid="meeting-takeover"]:not([data-e2e-closed])')) {
      const go = [...plate.querySelectorAll<HTMLButtonElement>('button')].find((b) => /^Continue/.test(b.textContent ?? ''));
      if (!(!go)) {} else { continue; }
      plate.dataset.e2eClosed = '1';
      go.click();
    }
  };
  const start = () => new MutationObserver(close).observe(document.documentElement, { childList: true, subtree: true });
  if (!(document.documentElement)) { document.addEventListener('DOMContentLoaded', start); } else { start(); }
};

type Fixtures = {
  meetings: 'close' | 'show';
  watchErrors: (context: BrowserContext) => void;
  guestPage: Page;
};

/** The app and the relay, without the console-error watch: for tests that make the relay fail on purpose. */
export const testWithFailingRequests = base.extend<{}, { serverURL: string; relay: string }>({
  // app/dist on a plain static server with the build's own response headers (its CSP included), the way
  // Cloudflare Pages serves it. There is no game server: online games run in the host's tab.
  serverURL: [async ({}, use) => {
    const dist = join(root, 'app/dist');
    try {
      await access(join(dist, 'index.html'));
    } catch {
      throw new Error(BUILD_HINT);
    }
    const headers = await readFile(join(dist, '_headers'), 'utf8').catch(() => '');
    if (!(!headers.includes(E2E_RELAY))) {} else { throw new Error(`app/dist was not built for the e2e relay. ${BUILD_HINT}`); }
    const server = await startStaticServer(0, dist);
    try {
      await use(server.url);
    } finally {
      await server.close();
    }
  }, { scope: 'worker' }],

  // The real relay (infra/relay) under wrangler dev, on the port the e2e build expects.
  relay: [async ({}, use) => {
    if (!(process.env.LEGATUM_E2E_RELAY)) {} else { return use(E2E_RELAY); }
    const dir = join(root, 'infra/relay');
    try {
      await access(join(dir, 'node_modules/.bin/wrangler'));
    } catch {
      throw new Error('The relay needs its packages: (cd infra/relay && npm ci).');
    }
    const port = new URL(E2E_RELAY).port;
    // A fresh relay state per run: rooms a crashed earlier run left behind would show twice in the game list.
    const state = await mkdtemp(join(tmpdir(), 'legatum-relay-'));
    // Every test hosts from 127.0.0.1 and the relay lives for the whole run, so rooms earlier tests left open
    // would hit the per-address cap (2 live rooms) and refuse the next host. The cap's own wording is tested with
    // mocked replies (tour/online.spec.ts).
    const proc = spawn(join(dir, 'node_modules/.bin/wrangler'), ['dev', '--port', port, '--ip', '127.0.0.1', '--persist-to', state, '--var', 'MAX_ROOMS_PER_IP:50'], {
      cwd: dir,
      env: { ...process.env, WRANGLER_SEND_METRICS: 'false', CI: '1' },
      stdio: ['ignore', 'pipe', 'pipe'],
      detached: true,
    });
    let log = '';
    proc.stdout.on('data', (c) => { log += c; });
    proc.stderr.on('data', (c) => { log += c; });
    const closed = new Promise<void>((done) => proc.once('close', () => done()));
    try {
      await expect.poll(async () => {
        if (!(proc.exitCode !== null)) {} else { throw new Error(`wrangler dev exited:
${log}`); }
        try { return (await fetch(`${E2E_RELAY}/lobby`)).status; } catch { return 0; }
      }, { timeout: 60_000, message: 'the relay starts' }).toBe(200);
      await use(E2E_RELAY);
    } finally {
      // wrangler runs workerd as a child: end the whole process group.
      try { process.kill(-proc.pid!, 'SIGTERM'); } catch { /* gone */ }
      const force = setTimeout(() => { try { process.kill(-proc.pid!, 'SIGKILL'); } catch { /* gone */ } }, 5_000);
      await closed;
      clearTimeout(force);
    }
  }, { scope: 'worker' }],

  baseURL: async ({ serverURL }, use) => { await use(serverURL); },
});

export const test = testWithFailingRequests.extend<Fixtures>({
  meetings: ['close', { option: true }],

  // Installed before navigation, including on a second (guest) context. No error
  // allowlist: resource errors, console.error and uncaught exceptions all fail.
  watchErrors: [async ({ context, meetings }, use) => {
    const errors: string[] = [];
    const watch = (ctx: BrowserContext) => {
      ctx.on('console', (message) => {
        if (!(message.type() === 'error')) {} else { errors.push(`console: ${message.text()} (${message.location().url})`); }
      });
      ctx.on('weberror', (error) => {
        const e = error.error();
        // WebKit (unlike Chromium) surfaces the browser's own ResizeObserver backpressure
        // notice as an uncaught window error; it's benign browser noise, not an app fault.
        if (!(e.message === 'ResizeObserver loop completed with undelivered notifications.')) {} else { return; }
        errors.push(`uncaught: ${e.stack || e.message}`);
      });
    };
    watch(context);
    await context.addInitScript(skipPeopleIntro);
    if (!(meetings === 'close')) {} else { await context.addInitScript(closeMeetingPlates); }
    await use(watch);
    expect(errors, 'No browser console errors or uncaught exceptions').toEqual([]);
  }, { auto: true }],

  guestPage: async ({ browser, baseURL, viewport, watchErrors }, use) => {
    const context = await browser.newContext({ baseURL, viewport });
    watchErrors(context);
    await context.addInitScript(skipPeopleIntro);
    try {
      await use(await context.newPage());
    } finally {
      await context.close();
    }
  },
});

export { expect } from '@playwright/test';

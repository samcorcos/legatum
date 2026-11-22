import { spawn } from 'node:child_process';
import { access, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { test as base, expect, type BrowserContext, type Page } from '@playwright/test';

const root = fileURLToPath(new URL('../../', import.meta.url));

/** The release server binary: LEGATUM_SERVER_BIN, else <CARGO_TARGET_DIR or target>/release/legatum-server. */
export function serverBinary(): string {
  if (process.env.LEGATUM_SERVER_BIN) return process.env.LEGATUM_SERVER_BIN;
  return join(resolve(root, process.env.CARGO_TARGET_DIR ?? 'target'), 'release/legatum-server');
}

type Fixtures = {
  watchErrors: (context: BrowserContext) => void;
  guestPage: Page;
};

export const test = base.extend<Fixtures, { serverURL: string }>({
  // Use the real release server for static files too: /api/status is real, and no
  // request interception or fake server can accidentally make the LAN test pass.
  serverURL: [async ({}, use) => {
    const binary = serverBinary();
    const staticDir = join(root, 'app/dist');
    try {
      await access(binary);
      await access(join(staticDir, 'index.html'));
    } catch {
      throw new Error('Build first: scripts/build-wasm.sh, (cd app && npm run build), cargo build --release -p legatum-server. See app/README.md.');
    }
    const saves = await mkdtemp(join(tmpdir(), 'legatum-e2e-'));
    // Port 0 lets the OS reserve a free port without a probe/bind race.
    const server = spawn(binary, ['--port', '0', '--static', staticDir, '--saves', saves, '--archive-dir', join(saves, 'archive')], {
      cwd: root,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let log = '';
    const closed = new Promise<void>((resolve) => server.once('close', () => resolve()));
    try {
      const url = await new Promise<string>((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error(`Server did not start:\n${log}`)), 30_000);
        const fail = (reason: unknown) => {
          clearTimeout(timer);
          reject(new Error(`Server failed (${String(reason)}):\n${log}`));
        };
        server.once('error', fail);
        server.once('exit', (code, signal) => fail(signal ?? code));
        server.stderr.on('data', (chunk) => { log += chunk.toString(); });
        server.stdout.on('data', (chunk) => {
          log += chunk.toString();
          const match = log.match(/Host: (http:\/\/localhost:\d+)/);
          if (match) {
            clearTimeout(timer);
            resolve(match[1]);
          }
        });
      });
      // The listener is bound before the startup line, but check HTTP readiness
      // as well so the first navigation doesn't depend on process timing.
      await expect.poll(async () => {
        try { return (await fetch(`${url}/api/status`)).status; }
        catch { return 0; }
      }).toBe(200);
      await use(url);
    } finally {
      server.kill('SIGTERM');
      const force = setTimeout(() => server.kill('SIGKILL'), 5_000);
      await closed;
      clearTimeout(force);
      await rm(saves, { recursive: true, force: true });
    }
  }, { scope: 'worker' }],

  baseURL: async ({ serverURL }, use) => { await use(serverURL); },

  // Installed before navigation, including on the second LAN context. No error
  // allowlist: resource errors, console.error and uncaught exceptions all fail.
  watchErrors: [async ({ context }, use) => {
    const errors: string[] = [];
    const watch = (ctx: BrowserContext) => {
      ctx.on('console', (message) => {
        if (message.type() === 'error') errors.push(`console: ${message.text()} (${message.location().url})`);
      });
      ctx.on('weberror', (error) => {
        const e = error.error();
        // WebKit (unlike Chromium) surfaces the browser's own ResizeObserver backpressure
        // notice as an uncaught window error; it's benign browser noise, not an app fault.
        if (e.message === 'ResizeObserver loop completed with undelivered notifications.') return;
        errors.push(`uncaught: ${e.stack || e.message}`);
      });
    };
    watch(context);
    await use(watch);
    expect(errors, 'No browser console errors or uncaught exceptions').toEqual([]);
  }, { auto: true }],

  guestPage: async ({ browser, baseURL, viewport, watchErrors }, use) => {
    const context = await browser.newContext({ baseURL, viewport });
    watchErrors(context);
    try {
      await use(await context.newPage());
    } finally {
      await context.close();
    }
  },
});

export { expect } from '@playwright/test';

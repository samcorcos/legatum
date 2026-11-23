# Legatum app

The React/Vite web UI. The Rust engine runs inside it as WASM (`crates/wasm`, built into `src/wasm-pkg`). The game ships only as a static web app: there is no desktop app and no server program.

## Web (development)

```sh
scripts/build-wasm.sh        # from the repo root; builds src/wasm-pkg (git-ignored)
scripts/build-wasm.sh && cd app && npm ci && npm run dev   # http://localhost:5173
```

## Unit tests

```sh
npx playwright install chromium   # once: src/artDistinct.test.ts renders the art in headless Chromium
npm test                          # Vitest; also needs python3 (the art check runs tools/art/distinctness.py)
```

## Hosted web build

```sh
scripts/build-web.sh         # from the repo root: WASM, licence notices, the UI, host files
cd app && npm run e2e:hosted # plays a short game against dist/ on a plain static server (e2e-hosted/); with LEGATUM_GAMES_URL set it answers the upload requests itself and checks one was sent
```

This is what `.github/workflows/deploy-web.yml` publishes to Cloudflare Pages. The build asks no server for anything except the relay (online games) and the optional game upload (`src/platform/gameUpload.ts`, on when `scripts/build-web.sh` is given `LEGATUM_GAMES_URL`; see [infra/games](../infra/games/README.md)). `hosted/_headers` and `hosted/404.html` are copied into `dist/`. See [docs/ops/releasing.md](../docs/ops/releasing.md).

## App icon

Every icon (the Atlas Seal: the wordmark's star-crowned A in an engraved roundel) comes from one script, `tools/design/app-icon/build.mjs`. Run `node tools/design/app-icon/build.mjs` from the repository root after changing it: it rewrites `public/favicon.svg` and the PNG favicons and `apple-touch-icon.png`.

## Online games

Online games run in the host's browser tab and use the relay in `infra/relay` (see [plans/online-play.md](../plans/online-play.md)). There is no server program to run.

## Browser end-to-end tests

From the repository root (Rust, `wasm-pack`, Node/npm and `python3` required):

```sh
export PATH="$HOME/.cargo/bin:$PATH"
scripts/build-wasm.sh
(cd infra/relay && npm ci)
LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh --skip-wasm
cd app
npm ci
npx playwright install chromium
npm run e2e
```

The suite runs against the static `app/dist`, so after app changes rebuild it with
`LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh` (add `--skip-wasm` to reuse the
engine build); rebuild WASM after changing Rust code. `npm test` remains the Vitest suite;
`npm run build` type-checks the Playwright tests. `npm run test:e2e` rebuilds `dist`
for the local relay first, then runs the suite.

`e2e/` holds over a hundred spec files (plus the screen tour in `e2e/tour/`).
The whole suite runs on one worker and takes ten minutes or more, so while
working run the files your change touches, for example
`npx playwright test e2e/game.spec.ts`. `e2e/game.spec.ts` holds the core flows:
Single Player (Small map, Hard AI, production, more turns, and the VP track),
autosave/reload/Continue with an exact game-state comparison,
online host/join in separate browser contexts. Every spec
fails on any browser console error or uncaught exception, including errors from
a second online context.

The suite uses Chromium, fixed planet seeds, one worker, and no retries. Local
games explicitly use the real WASM client. The fixture serves `app/dist` on a plain static server with the build's own
response headers, the way Cloudflare Pages does, and starts the real relay
(`infra/relay`) under `wrangler dev` on `127.0.0.1:8787` for the online tests (see
`e2e/fixtures.ts`; `LEGATUM_E2E_RELAY` points them at a deployed relay instead).
Online games run in the host's tab, so no game server is involved. Both are
stopped at teardown. No existing host, user saves, or mock server is used.

Use `npm run e2e -- --headed` to watch, or `npm run e2e -- --grep Online` to run just
the network flows. Failure screenshots and traces are under `test-results/`;
open a trace with `npx playwright show-trace <path-to-trace.zip>`.

In a restricted workspace, caches can be kept inside the ignored `target/`
directory: set `WASM_PACK_CACHE="$PWD/target/wasm-pack-cache"` before building
WASM from the root, and `npm_config_cache="$PWD/../target/npm-cache"` plus
`PLAYWRIGHT_BROWSERS_PATH="$PWD/../target/playwright"` when working in `app/`.
Use the same browser-cache setting for installation and test runs.

The runner must be allowed to launch Chromium and bind local server sockets.
On macOS, `bootstrap_check_in ... Permission denied` during browser launch is
a sandbox restriction before any flow runs; run the commands in a terminal or
CI environment that permits browser processes. Changing cache paths does not
resolve that restriction.

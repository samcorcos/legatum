# Legatum app

The React/Vite web UI. The Rust engine runs inside it as WASM (`crates/wasm`, built into `src/wasm-pkg`), so the same UI runs in a browser for testing and in the Tauri desktop app.

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
scripts/build-web.sh         # from the repo root: WASM, licence notices, the UI with VITE_LEGATUM_HOSTED=1, host files
cd app && npm run e2e:hosted # plays a short game against dist/ on a plain static server (e2e-hosted/)
```

This is what `.github/workflows/deploy-web.yml` publishes to Cloudflare Pages. `src/platform/hosted.ts` is the one switch: the hosted build asks no server for anything except the optional game upload (`src/platform/gameUpload.ts`, on when `scripts/build-web.sh` is given `LEGATUM_GAMES_URL`; see [infra/games](../infra/games/README.md)), and the Multiplayer menu explains that network games need the host program. `hosted/_headers` and `hosted/404.html` are copied into `dist/`. See [docs/ops/releasing.md](../docs/ops/releasing.md).

## Mac desktop app

One command from a clean checkout (needs Rust with `wasm-pack`, Node/npm, and the Xcode command line tools):

```sh
scripts/build-mac-app.sh          # add --dmg to also produce a .dmg
open app/src-tauri/target/release/bundle/macos/Legatum.app
```

It builds the WASM engine with the dedicated `wasm-release` profile (fat LTO, one codegen unit, and `wasm-opt -O3`), then runs `npx tauri build`, which builds the web UI (`npm run build` into `app/dist`) and bundles it into `Legatum.app` (about 5 MB). The first build compiles Tauri and takes a couple of minutes; later builds are faster.

To iterate with hot reload inside the native window: `cd app && npm run tauri:dev` (after `scripts/build-wasm.sh`).

How it fits together:

- `src-tauri/` is its own Cargo workspace (not part of the root engine workspace). It is a thin shell: one window (1440x900, min 1000x600) showing `app/dist`. The game runs in the webview as WASM. `archive_stage` encodes a recording and returns a staged archive path and bytes. `archive_commit` reads and stores a staged recording. `scenario_save_run` validates and saves a scenario run JSON file. Autosave uses the filesystem plugin's scoped file access.
- The CSP in `src-tauri/tauri.conf.json` allows `'wasm-unsafe-eval'` for the engine and inline styles for React. Fonts are bundled through `@fontsource`, so the app works offline.
- Save/Load use native dialogs in Tauri (`.legatum` JSON files; older `.legatum.json` downloads also load). Browser builds keep download/upload. File access is limited to paths selected in a dialog, plus writing `autosave.legatum` in the app data directory (`~/Library/Application Support/com.legatum.game/` on macOS). Natively the app also writes to `~/LegatumData/` (`%APPDATA%\Legatum` where `HOME` is unset): the local game archive in `archive/` (override with `--archive-dir` or `LEGATUM_ARCHIVE`), scenario runs in `scenarios/`, and `app-info.json`. If the archive folder cannot be opened, the game runs without it and logs one line. Autosave mirrors localStorage there in the background; Continue still uses localStorage, and the disk copy can be recovered with Load Game. All file I/O preserves the save text, including the opaque engine string containing `u64` values.
- Every icon (the Atlas Seal: the wordmark's star-crowned A in an engraved roundel) comes from one script, `tools/design/app-icon/build.mjs`. Run `node tools/design/app-icon/build.mjs` from the repository root after changing it: it rewrites `public/favicon.svg` (also served by the build dashboard), the PNG favicons and `apple-touch-icon.png`, and `src-tauri/icons/` (via `npx tauri icon`, then a hand-built `icon.icns` whose 16 and 32 px slots use a bolder small-size drawing). It deletes the Android, iOS and Windows Store sets `tauri icon` also writes, since we don't ship them yet.
- A local build is unsigned. On another Mac, Gatekeeper blocks it on first open: right-click > Open, or `xattr -dr com.apple.quarantine Legatum.app`. The signed, notarized release is built by `.github/workflows/release-mac.yml` ([docs/ops/releasing.md](../docs/ops/releasing.md)): hardened runtime, the empty `src-tauri/Entitlements.plist`, and `src-tauri/tauri.dist.conf.json` merged over `tauri.conf.json` for what only a distributed build has (the disk image and the licence notices as a bundle resource).
- `scripts/build-mac-app.sh` first writes `public/THIRD_PARTY_LICENSES.txt` (`tools/release/third-party-licenses.mjs`); Credits in the Guide reads it.

## iOS Simulator app

Needs Xcode with an iOS Simulator runtime, plus everything the Mac build needs. From the repo root:

```sh
scripts/build-ios-sim.sh                        # builds app/src-tauri/gen/apple/build/arm64-sim/Legatum.app
scripts/build-ios-sim.sh --run                  # ...then boots "iPad Pro 11-inch (M4)", installs and launches it
scripts/build-ios-sim.sh --run "iPhone 15 Pro"  # any simulator name from `xcrun simctl list devices`
```

It builds the WASM engine, generates the Xcode project with `npx tauri ios init --ci` if `src-tauri/gen/apple` is missing (the first run adds the iOS rustup targets and installs `xcodegen` and `libimobiledevice` with Homebrew), then runs `npx tauri ios build --debug --target aarch64-sim --ci`, which builds the web UI and the Rust shell for the Apple Silicon simulator. Add `--release` for an optimized shell. Simulator builds are unsigned and need no Apple team; real devices need signing configured in Xcode.

- `src-tauri/gen/` is generated and git-ignored. iOS settings live in `src-tauri/tauri.ios.conf.json` (merged over `tauri.conf.json`: one window with no size minimums, iOS 16+) and `src-tauri/Info.ios.plist` (merged into the generated Info.plist: iPad is landscape-only and full screen; iPhone allows portrait and landscape). Delete `gen/apple` to regenerate it after upgrading Tauri.
- The same bundle identifier (`com.legatum.game`), CSP and capabilities as desktop. Save and Load use the iOS Files pickers, the autosave copy goes to the app's `Library/Application Support/com.legatum.game/`, and Quit is hidden (iOS apps don't quit themselves).
- Screenshot a running simulator with `xcrun simctl io <udid> screenshot out.png` (landscape iPad screenshots come out rotated; fix with `sips -r 270 out.png`).

## LAN play

LAN hosting is currently a separate `legatum-server` binary that serves `app/dist` and runs the authoritative game; players join in a browser. Embedding that server in the desktop app is a later step and not part of this build.

## Browser end-to-end tests

From the repository root (Rust, `wasm-pack`, Node/npm and `python3` required):

```sh
export PATH="$HOME/.cargo/bin:$PATH"
scripts/build-wasm.sh
cargo build --release -p legatum-server
cd app
npm ci
npx playwright install chromium
npm run build
npm run e2e
```

After app changes, run `npm run build` before `npm run e2e`; rebuild WASM or the
server after changing their Rust code. `npm test` remains the Vitest suite; the
build also type-checks the Playwright tests. `npm run test:e2e` rebuilds the
release server first, then runs the suite.

`e2e/` holds over a hundred spec files (plus the screen tour in `e2e/tour/`).
The whole suite runs on one worker and takes ten minutes or more, so while
working run the files your change touches, for example
`npx playwright test e2e/game.spec.ts`. `e2e/game.spec.ts` holds the core flows:
Single Player (Small map, Hard AI, production, more turns, and the VP track),
localStorage autosave/reload/Continue with an exact game-state comparison,
two-human Hot Seat, and LAN host/join in separate browser contexts. Every spec
fails on any browser console error or uncaught exception, including errors from
a second LAN context.

The suite uses Chromium, fixed planet seeds, one worker, and no retries. Local
games explicitly use the real WASM client. The fixture starts
`target/release/legatum-server --port 0 --static app/dist --saves <temp-dir>`
(or `LEGATUM_SERVER_BIN`); the OS chooses a free port. This real server serves
the built UI for every test and resolves the LAN games. It is stopped and its
temporary saves removed at teardown. No existing host, user saves, or mock
server is used.

Use `npm run e2e -- --headed` to watch, or `npm run e2e -- --grep LAN` to run just
the network flows. Failure screenshots and traces are under `test-results/`;
open a trace with `npx playwright show-trace <path-to-trace.zip>`.

In a restricted workspace, caches can be kept inside the ignored `target/`
directory: set `WASM_PACK_CACHE="$PWD/target/wasm-pack-cache"` before building
WASM from the root, and `npm_config_cache="$PWD/../target/npm-cache"` plus
`PLAYWRIGHT_BROWSERS_PATH="$PWD/../target/playwright"` when working in `app/`.
Use the same browser-cache setting for installation and test runs.

The runner must be allowed to launch Chromium and bind a local server socket.
On macOS, `bootstrap_check_in ... Permission denied` during browser launch is
a sandbox restriction before any flow runs; run the commands in a terminal or
CI environment that permits browser processes. Changing cache paths does not
resolve that restriction.

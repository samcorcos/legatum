# Releasing Legatum

Two things reach players: the web build ("Play in your browser") and the signed Mac app. Both are built by
GitHub Actions from this repository. No secret value is written here or anywhere in the repository; secrets live
only in the repository's Actions settings (Settings → Secrets and variables → Actions).

## The web build (Cloudflare Pages)

`.github/workflows/deploy-web.yml` runs on every push to `master` and on a manual run. It builds the engine as
WebAssembly and the web UI (`scripts/build-web.sh`), plays a short game against the result on a plain static
server (`npm run e2e:hosted` in `app/`), creates the Cloudflare Pages project if it does not exist, and deploys
`app/dist` as the production deployment. It never runs for pull requests. Without the two secrets it passes with
a notice and deploys nothing, so forks stay green.

### One-time setup

1. Create a Cloudflare account (the free plan is enough) and note the **Account ID** (dashboard → Workers &
   Pages, right-hand column).
2. Create an API token (My Profile → API Tokens → Create Token → Custom token) with one permission:
   **Account → Cloudflare Pages → Edit**, limited to that account.
3. Add the repository secrets and, if needed, the variables below.
4. Run the workflow (Actions → Deploy web → Run workflow) or push to `master`. The run's summary shows the
   address.
5. If the address is not `https://legatum-game.pages.dev`, correct it in the two places it is written: `homepage` in
   `app/package.json` (the link-preview tags are built from it) and the "Play in your browser" link in
   `README.md`. Then check a preview of the link with a card validator before announcing it.

| Name | Kind | What it is |
| --- | --- | --- |
| `CLOUDFLARE_API_TOKEN` | secret | The API token from step 2. It can edit Pages projects in that one account and nothing else. |
| `CLOUDFLARE_ACCOUNT_ID` | secret | The Cloudflare account that owns the Pages project. |
| `CLOUDFLARE_PAGES_PROJECT` | variable, optional | The Pages project name. Default `legatum-game` (`legatum` belongs to someone else). Project names are unique across Cloudflare: if `legatum-game` is taken, creation fails with a clear message; set this to a free name. The address becomes `https://<name>.pages.dev`. |
| `LEGATUM_SITE_URL` | variable, optional | The public address used in the link-preview tags, for a custom domain. Default: `homepage` in `app/package.json`. |
| `LEGATUM_GAMES_URL` | variable, optional | The game-upload Worker (`infra/games`, deployed separately). Set, every game played on the site is uploaded there and the page may connect to it; unset, nothing is uploaded. A custom domain must also be added to `ALLOWED_ORIGINS` in `infra/games/wrangler.jsonc`, or the Worker refuses its uploads (they stay queued in the browser). |

### What the hosted build is

Static files only. `VITE_LEGATUM_HOSTED=1` (`app/src/platform/hosted.ts`) makes the UI skip every request to
`legatum-server`, and the Multiplayer menu says that Host Game and Join Game need the host program. Single
player, hot seat, scenarios, autosave (browser storage) and Save to file / Load Game (download and upload) work.
`app/hosted/_headers` sets the security headers and long caching for the hashed files; `app/hosted/404.html`
answers unknown addresses. The title screen shows `Version <app version> · web <commit>`.

With `LEGATUM_GAMES_URL` set, the build uploads every game's record (names removed) to that Worker, the
title screen says so in one line, and `_headers` adds the Worker to `connect-src`. See `infra/games/README.md`.

To build and check it locally:

```sh
scripts/build-web.sh
cd app && npm run e2e:hosted
```

## The Mac app (signed and notarized)

`.github/workflows/release-mac.yml` builds a universal app (Apple Silicon and Intel), signs it with a Developer
ID Application certificate under the hardened runtime, has Apple notarize the app and the disk image, staples
both, verifies them (`codesign --verify --deep --strict`, `spctl`, `stapler validate`, both architectures, the
app inside the mounted image) and attaches `Legatum-<version>-mac-universal.dmg` to a **draft** GitHub release.
It never publishes. If any signing secret is missing it stops at the start with a message naming it.

### One-time setup

1. Enrol in the Apple Developer Program.
2. Create a **Developer ID Application** certificate (Xcode → Settings → Accounts → Manage Certificates, or
   developer.apple.com → Certificates). "Apple Development" and "Mac App Distribution" certificates do not work
   for a download outside the App Store.
3. In Keychain Access, export that certificate together with its private key as a `.p12` with a password, then
   base64-encode it: `base64 -i certificate.p12 | pbcopy`.
4. Create an app-specific password for your Apple ID at account.apple.com (Sign-In and Security → App-Specific
   Passwords).
5. Add the five secrets.

| Secret | What it is |
| --- | --- |
| `APPLE_CERTIFICATE` | The Developer ID Application certificate and its private key: the `.p12` file, base64-encoded. |
| `APPLE_CERTIFICATE_PASSWORD` | The password chosen when exporting the `.p12`. |
| `APPLE_ID` | The Apple ID (email) that submits the app for notarization. |
| `APPLE_PASSWORD` | An app-specific password for that Apple ID. Not the account password. |
| `APPLE_TEAM_ID` | The 10-character team identifier (developer.apple.com → Membership details). |

The signing identity is read from the certificate during the run; no name is stored in the repository.

### Cutting a release

1. Set the same version in `app/src-tauri/tauri.conf.json`, `app/package.json` and `app/src-tauri/Cargo.toml`
   (`app/src/version.test.ts` fails when they differ), and merge that to `master`.
2. Either push a tag (`git tag v0.1.0 && git push origin v0.1.0`) or run the workflow by hand with the version.
   A manual run builds the commit it is started on and creates the tag when the draft is published.
3. When the run is green, open the draft release, download the disk image, and try it on a Mac that has never
   run the app: it should open with no Gatekeeper warning beyond "downloaded from the internet".
4. Publish the draft by hand.
5. First release only: check that the Mac line in `README.md` (at the `<!-- mac-download -->` marker) links to
   `https://github.com/samcorcos/legatum/releases/latest`. It already does; the link works once this release is published.

Re-running for the same version replaces the files on the draft. It refuses to touch a published release.

### What the app is signed with

- Hardened runtime on, and `app/src-tauri/Entitlements.plist`, which is empty on purpose: the WebAssembly
  engine runs inside WKWebView's own process, so the app needs no JIT entitlement. The file says what would
  justify adding one.
- Bundle identifier `com.legatum.game`. It names the folder saves live in
  (`~/Library/Application Support/com.legatum.game/`), so changing it after a release orphans players' autosaves.
- No updater is configured: players get a new version by downloading it.

## Third-party licence notices

Every distributed build carries `THIRD_PARTY_LICENSES.txt`, written by `tools/release/third-party-licenses.mjs`
from the locked dependency graph: the crates in the build, the production npm packages and the two bundled fonts
with their SIL Open Font License texts. The web build serves it at `/THIRD_PARTY_LICENSES.txt`; the Mac app has
it in `Contents/Resources/`; both show it in the game under How to Play → Credits → Open-source licences. A
crate that ships no licence file gets the standard text from `tools/release/license-texts/`; the script fails
and names the crate when that folder has no text for its licence.

## Local, unsigned builds

`scripts/build-mac-app.sh [--dmg]` still builds an unsigned app for development and playtests; Gatekeeper asks
for right-click → Open on another Mac. Playtest builds and their checks are described in the
`legatum-workflow` skill and `tools/release/check.sh`.

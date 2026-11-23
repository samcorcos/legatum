# Releasing Legatum

One thing reaches players: the web build ("Play in your browser") at https://playlegatum.com (a custom domain on the Cloudflare Pages project `legatum-game`; https://legatum-game.pages.dev serves the same deploy and stays, because a browser keeps saves per address and games begun there live there), with the
online lobby's relay (`infra/relay`) beside it. Both are deployed by GitHub Actions from this repository. No secret value is written here or anywhere in the repository; secrets live
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
5. If the address is not `https://playlegatum.com`, correct it in the two places it is written: `homepage` in
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

Static files only; there is no game server: the engine runs in the player's browser as WebAssembly. Online play
runs in the host player's tab and goes through the relay (`infra/relay`, `plans/online-play.md`):
`scripts/build-web.sh` bakes the relay's address in (`LEGATUM_RELAY_URL`, default the live relay, `none` for a build
without online play) and writes it into the page's `connect-src`. Single player, online co-op,
autosave (browser storage) and Save to file / Load Game (download and upload) work.
`app/hosted/_headers` sets the security headers and long caching for the hashed files; `app/hosted/404.html`
answers unknown addresses. The title screen shows `Version <app version> · web <commit>`, and a production build adds ` · Last updated <date time>` on the same line: the build's UTC time (`__BUILD_TIME__` in `app/vite.config.ts`, `LEGATUM_BUILD_TIME` overrides it), shown in the player's own time zone (`app/src/buildTime.ts`). Each kept build (`tools/release/keep-builds.mjs`) carries its own time. Dev builds leave it off.

With `LEGATUM_GAMES_URL` set, the build uploads every game's record (names removed) to that Worker, the
title screen says so in one line, and `_headers` adds the Worker to `connect-src`. See `infra/games/README.md`.

To build and check it locally:

```sh
scripts/build-web.sh
cd app && npm run e2e:hosted
```

When the build has `LEGATUM_GAMES_URL`, the hosted test answers requests to that Worker itself (no test game reaches the real bucket) and fails if no upload was sent. To check that path locally, build with any URL, for example `LEGATUM_GAMES_URL=https://legatum-games.example.workers.dev scripts/build-web.sh`, then run `npm run e2e:hosted`.

### Games left open across a deploy

Every built file is named by its content, and Pages serves only the newest deployment, so a deploy would take away
the art (and lazy code) that a tab opened before it still asks for: the player sees broken images. The build keeps
them instead. With `LEGATUM_CARRY_ASSETS_FROM` set (the deploy workflow sets the live site; `scripts/deploy-preview.sh`
sets the preview's own address), `scripts/build-web.sh` runs `tools/release/carry-assets.mjs`, which copies into
`app/dist/assets` every file a deployed build used in the last 30 days (listed with that day in the site's
`asset-history.json`, which the build writes for the next deploy). A production deploy run by hand must set it too:
`LEGATUM_CARRY_ASSETS_FROM=https://legatum-game.pages.dev scripts/build-web.sh`.

### Every game finishes on the version it started on

A deploy can also change the rules (`RULES_VERSION`), and a build refuses a save made with other rules. So every
build stays playable for 30 days after a newer one replaces it. `scripts/build-web.sh` runs
`tools/release/keep-builds.mjs`, which:

- gives the build an id (the commit, plus a hash of any uncommitted changes; `LEGATUM_BUILD_ID` overrides it) and
  publishes it twice: at the site root (the newest version) and at `v/<id>/`, a copy of its page with
  `<base href="../../">`, so its scripts, art, terrain and sounds load from the root. Same site, so the same autosaves.
- copies the pages of the earlier builds the live site lists (from `LEGATUM_CARRY_ASSETS_FROM`) and writes
  `versions.json`: every build the site serves, newest first, with its rules version, online protocol and title-screen
  label. A build drops out 30 days after it was replaced, the same window `carry-assets.mjs` keeps its files.
- copies back root files (terrain, wonders, audio) that an earlier build used and this one dropped.

The game reads `versions.json` (`app/src/platform/versions.ts`). Saves record the build that wrote them.

- **Continue or Load Game with a save from other rules** opens it in the build that made it, or the newest build with
  its rules: an autosave by its address (`v/<id>/?continue=<game>`, the same browser database), a chosen file
  handed over through browser storage (`?open=save`). Only when no kept build has its rules does the game refuse it.
- **Join Game** on a game whose host runs another kept build opens that build and joins (`?join=<room>`).
- **Between games**, a tab whose build is not the newest says so on Single Player's sub-line ("New games start on the
  newest version"), and Single Player and Multiplayer open the newest version (`/?menu=solo` or `multi`). A
  game in progress is never interrupted.

Builds from before `versions.json` existed can be added once by their Pages deployment address (from
`wrangler pages deployment list --project-name legatum-game`), with the rules version they ran:
`LEGATUM_KEEP_LEGACY="https://<deployment>.legatum-game.pages.dev=<id>:<rules>"`. Their files are crawled into the
build too. Such a build ignores the `?continue` address, so the player presses Continue there.

Limits: the site's `_headers` (the Content-Security-Policy) is the newest build's, so an older build whose relay or
upload Worker address changed cannot reach the old one. A save made with rules the site no longer serves is refused
as before.

Every web build, and CI's app build, also runs `tools/release/check-dist-refs.mjs`: it fails if the JS, CSS or HTML
points to a file the build does not contain (a link to a folder, such as `/`, counts when the folder has an `index.html`).

## Third-party licence notices

Every distributed build carries `THIRD_PARTY_LICENSES.txt`, written by `tools/release/third-party-licenses.mjs`
from the locked dependency graph: the crates in the build, the production npm packages and the two bundled fonts
with their SIL Open Font License texts. The web build serves it at `/THIRD_PARTY_LICENSES.txt` and shows it in
the game under How to Play → Credits → Open-source licences. A
crate that ships no licence file gets the standard text from `tools/release/license-texts/`; the script fails
and names the crate when that folder has no text for its licence.

## Playtest builds

A playtest is the web build of a branch; its checks are described in the `legatum-workflow` skill and
`tools/release/check.sh`.

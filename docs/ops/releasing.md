# Releasing Legatum

One thing reaches players: the web build ("Play in your browser") at https://legatum-game.pages.dev, with the
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
5. If the address is not `https://legatum-game.pages.dev`, correct it in the two places it is written: `homepage` in
   `app/package.json` (the link-preview tags are built from it) and the "Play in your browser" link in
   `README.md`. Then check a preview of the link with a card validator before announcing it.

| Name | Kind | What it is |
| --- | --- | --- |
| `CLOUDFLARE_API_TOKEN` | secret | The API token from step 2. It can edit Pages projects in that one account and nothing else. |
| `CLOUDFLARE_ACCOUNT_ID` | secret | The Cloudflare account that owns the Pages project. |
| `CLOUDFLARE_PAGES_PROJECT` | variable, optional | The Pages project name. Default `legatum-game` (`legatum` belongs to someone else). Project names are unique across Cloudflare: if `legatum-game` is taken, creation fails with a clear message; set this to a free name. The address becomes `https://<name>.pages.dev`. |
| `LEGATUM_SITE_URL` | variable, optional | The public address used in the link-preview tags, for a custom domain. Default: `homepage` in `app/package.json`. |

### What the hosted build is

Static files only: the engine runs in the player's browser as WebAssembly. Single player, hot seat, online games
through the relay, scenarios, autosave (browser storage) and Save to file / Load Game (download and upload) work.
`app/hosted/_headers` sets the security headers and long caching for the hashed files; `app/hosted/404.html`
answers unknown addresses. The title screen shows `Version <app version> · web <commit>`.

To build and check it locally:

```sh
scripts/build-web.sh
cd app && npm run e2e:hosted
```

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

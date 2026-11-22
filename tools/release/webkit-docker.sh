#!/usr/bin/env bash
# Playwright WebKit for the screen tour and the menu spec, inside a Linux container (never on the macOS host).
#
#   tools/release/webkit-docker.sh <worktree> [extra playwright args...]
#
# Why: Playwright WebKit on macOS captures screenshots through WindowServer, which aborted three
# times and logged the user out (docs/learnings.md, 2026-10-07). app/playwright.config.ts refuses
# WebKit on darwin; this is the only way to run it. Safari is WebKit, so WebKit coverage of
# e2e/tour and e2e/exit-to-title.spec.ts still matters (a WebKit-only blur bug).
# tools/release/check.sh calls this as its WebKit step.
#
# Inputs : <worktree>/app/dist (build it first: LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh) and infra/relay.
# Outputs: <worktree>/app/test-results (results and screen-tour screenshots), or $LEGATUM_WEBKIT_RESULTS when set
#          (tools/release/check.sh sets app/test-results-webkit because it runs this beside Chromium, which wipes
#          and rewrites app/test-results). Exit code is Playwright's.
# How    : image mcr.microsoft.com/playwright:v<version>-noble, <version> read from app/package-lock.json and
#          required to equal app/package.json. Docker Desktop must be installed (it is started if not running).
#          - No game server: the e2e fixture serves app/dist statically and runs the relay (infra/relay) under
#            `wrangler dev` inside the container, so the container needs only the dist and the relay's packages.
#          - node_modules: `npm ci` inside Docker volume legatum-webkit-nm-<hash of package-lock.json>;
#            the host's node_modules is never mounted or touched.
#          - The worktree is mounted read-only; app/ is copied into the container (minus node_modules)
#            and only app/test-results is writable from the host.
# Limits : --cpus=4, one named container (legatum-webkit-<pid>-<time>), stopped only by this script on exit/signal.
# Extra args go to `playwright test` (default reporter is list); env LEGATUM_WEBKIT_SPECS overrides the spec list.
set -uo pipefail
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

here="$(cd "$(dirname "$0")" && pwd)"
W="$(cd "${1:?usage: webkit-docker.sh <worktree> [playwright args...]}" && pwd)"; shift
specs="${LEGATUM_WEBKIT_SPECS:-e2e/tour e2e/exit-to-title.spec.ts}"
workers="${LEGATUM_WEBKIT_WORKERS:-1}"
cache="${LEGATUM_WEBKIT_CACHE:-$HOME/.cache/legatum-webkit}"
log() { echo "[webkit-docker] $*"; }
die() { echo "[webkit-docker] $*" >&2; exit 2; }

command -v docker >/dev/null || die "docker is not installed"
if ! docker info >/dev/null 2>&1; then
  log "starting Docker Desktop"; open -a Docker 2>/dev/null || true
  for _ in $(seq 1 60); do docker info >/dev/null 2>&1 && break; sleep 3; done
  docker info >/dev/null 2>&1 || die "Docker is not running"
fi
[ -f "$W/app/dist/index.html" ] || die "$W/app/dist is missing: build the e2e web dist first (LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh)"

# Playwright version: package-lock and package.json must agree exactly.
pw="$(node -e 'console.log(require(process.argv[1]).packages["node_modules/@playwright/test"].version)' "$W/app/package-lock.json")"
want="$(node -e 'console.log(require(process.argv[1]).devDependencies["@playwright/test"].replace(/^[\^~]/,""))' "$W/app/package.json")"
[ -n "$pw" ] && [ "$pw" = "$want" ] || die "Playwright version mismatch: package-lock $pw vs package.json $want"
image="mcr.microsoft.com/playwright:v$pw-noble"
docker image inspect "$image" >/dev/null 2>&1 || { log "pulling $image"; docker pull --platform linux/arm64 "$image" >/dev/null || die "pull failed"; }

mkdir -p "$cache"
lock="$cache/lock"
acquire() { # one builder at a time across agents; a dead holder's lock is stolen
  while ! mkdir "$lock" 2>/dev/null; do
    pid="$(cat "$lock/pid" 2>/dev/null || true)"
    if [ -n "$pid" ] && ! kill -0 "$pid" 2>/dev/null; then rm -rf "$lock"; continue; fi
    sleep 3
  done
  echo $$ > "$lock/pid"
}
release() { [ "$(cat "$lock/pid" 2>/dev/null)" = "$$" ] && rm -rf "$lock"; return 0; }
name="legatum-webkit-$$-$(date +%s)"
cleanup() { release; docker stop -t 5 "$name" >/dev/null 2>&1 || true; }
trap cleanup EXIT; trap 'exit 130' INT TERM

acquire

# 2. Linux node_modules in a volume keyed by package-lock.json.
nm="legatum-webkit-nm-$(shasum "$W/app/package-lock.json" "$W/app/package.json" | shasum | cut -c1-12)"
if ! docker run --rm -v "$nm":/w/node_modules "$image" test -f /w/node_modules/.legatum-ci-ok 2>/dev/null; then
  log "npm ci into volume $nm"
  docker run --rm --cpus=4 --name "$name-npm" -v "$W/app/package.json":/w/package.json:ro -v "$W/app/package-lock.json":/w/package-lock.json:ro \
    -v "$nm":/w/node_modules -w /w "$image" bash -euc 'npm ci --no-audit --no-fund && touch node_modules/.legatum-ci-ok' \
    || { docker volume rm "$nm" >/dev/null 2>&1; die "npm ci failed"; }
fi
# Relay packages (wrangler) in a second volume keyed by its lockfile.
rnm="legatum-webkit-relay-nm-$(shasum "$W/infra/relay/package-lock.json" "$W/infra/relay/package.json" | shasum | cut -c1-12)"
if ! docker run --rm -v "$rnm":/w/node_modules "$image" test -f /w/node_modules/.legatum-ci-ok 2>/dev/null; then
  log "npm ci (relay) into volume $rnm"
  docker run --rm --cpus=4 --name "$name-rnpm" -v "$W/infra/relay/package.json":/w/package.json:ro -v "$W/infra/relay/package-lock.json":/w/package-lock.json:ro \
    -v "$rnm":/w/node_modules -w /w "$image" bash -euc 'npm ci --no-audit --no-fund && touch node_modules/.legatum-ci-ok' \
    || { docker volume rm "$rnm" >/dev/null 2>&1; die "relay npm ci failed"; }
fi
release

# 3. The run.
results="${LEGATUM_WEBKIT_RESULTS:-$W/app/test-results}"
mkdir -p "$results"
surf() { command -v ioclasscount >/dev/null && ioclasscount IOSurface 2>/dev/null | awk '{print $NF}'; }
log "image $image, node_modules $nm, workers: $workers, specs: $specs"
log "IOSurface before: $(surf)"
start=$(date +%s)
docker run --rm --init --ipc=host --cpus=4 --name "$name" \
  -v "$W":/src:ro -v "$nm":/work/app/node_modules -v "$rnm":/work/infra/relay/node_modules -v "$results":/work/app/test-results \
  -e LEGATUM_E2E_BROWSER="${LEGATUM_E2E_BROWSER:-webkit}" -e LEGATUM_WEBKIT_WORKERS="$workers" -e CI=1 -e SPECS="$specs" -e LEGATUM_WEBKIT_TIMEOUT="${LEGATUM_WEBKIT_TIMEOUT:-}" "$image" bash -euc '
    tar -C /src --exclude=app/node_modules --exclude=app/test-results --exclude=app/test-results-webkit --exclude=app/playwright-report -cf - app | tar -C /work -xf -
    tar -C /src --exclude=infra/relay/node_modules -cf - infra/relay | tar -C /work -xf -
    cd /work/app
    exec npx playwright test $SPECS --workers="$LEGATUM_WEBKIT_WORKERS" --reporter=list --timeout="${LEGATUM_WEBKIT_TIMEOUT:-120000}" "$@"
  ' bash "$@"
rc=$?
log "IOSurface after: $(surf)"
log "exit $rc after $(( $(date +%s) - start ))s; results in $results"
exit "$rc"

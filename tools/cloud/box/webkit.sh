#!/usr/bin/env bash
# Playwright WebKit natively on the box (Linux, no VM, never the macOS host): the screen tour and the exit-to-title
# spec, as tools/release/webkit-docker.sh runs them. Runs in a slot at the ref being tested. Results: app/test-results.
#
#   LEGATUM_WEBKIT_WORKERS    Playwright workers (default 8; the measured table is in the legatum-workflow skill)
#   LEGATUM_WEBKIT_PREBUILT=1 skip the WASM and web dist builds (the caller has just built this tree: the release check)
#   LEGATUM_WEBKIT_OUTPUT     Playwright output directory under app/ (default test-results); the screen-tour shots go to
#                             <output>/screen-tour via LEGATUM_TOUR_OUT, so a Chromium run beside it cannot wipe them
#   LEGATUM_WEBKIT_SPECS      spec list (default: the screen tour, e2e/tour, and e2e/exit-to-title.spec.ts)
set -uo pipefail
. /data/bin/budget.sh && box_core_budget_apply   # this job's share of the cores (see budget.sh)
specs="${LEGATUM_WEBKIT_SPECS:-e2e/tour e2e/exit-to-title.spec.ts}"
specs="${specs//,/ }"
workers="${LEGATUM_WEBKIT_WORKERS:-8}"
output="${LEGATUM_WEBKIT_OUTPUT:-test-results}"
set -e
if [ "${LEGATUM_WEBKIT_PREBUILT:-0}" != 1 ]; then
  scripts/build-wasm.sh
  LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh --skip-wasm
fi
cd app
set +e
LEGATUM_TOUR_OUT="$PWD/$output/screen-tour" LEGATUM_E2E_BROWSER="${LEGATUM_E2E_BROWSER:-webkit}" CI=1 \
  npx playwright test $specs --workers="$workers" --output="$output" --reporter=list --timeout="${LEGATUM_WEBKIT_TIMEOUT:-120000}" "$@"
exit $?

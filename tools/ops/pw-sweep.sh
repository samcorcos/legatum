#!/usr/bin/env bash
# Measure Playwright wall time at several worker counts on one machine (run it on the cloud box, in a slot):
#
#   tools/cloud/remote.sh <worktree> --npm --artifact .progress/pw-sweep -- bash tools/ops/pw-sweep.sh chromium 12 24 36 48
#   tools/cloud/remote.sh <worktree> --npm --artifact .progress/pw-sweep -- bash tools/ops/pw-sweep.sh webkit 4 8 16 24
#
# Builds the WASM package and the static e2e web dist (relay on 127.0.0.1:8787) once, then runs the same Playwright command at each worker
# count and writes one TSV row each to .progress/pw-sweep/<engine>.tsv: workers, wall seconds, load average before and
# after, counts, failures by class, screenshots written. The specs are whatever LEGATUM_SWEEP_SPECS says (default: all
# game specs for chromium; the screen tour and exit-to-title for webkit, as the release check runs them).
set -uo pipefail
engine="${1:?engine: chromium or webkit}"; shift
[ "$#" -ge 1 ] || { echo "usage: pw-sweep.sh chromium|webkit WORKERS..." >&2; exit 2; }
out=".progress/pw-sweep"; mkdir -p "$out"
scripts/build-wasm.sh
LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh --skip-wasm
if [ "$engine" = webkit ]; then
  specs="${LEGATUM_SWEEP_SPECS:-e2e/tour e2e/exit-to-title.spec.ts}"; timeout=120000
else
  specs="${LEGATUM_SWEEP_SPECS:-}"; timeout=90000
fi
tsv="$out/$engine.tsv"
[ -f "$tsv" ] || printf 'engine\tworkers\twall_s\tload_before\tload_after\tpassed\tfailed\tflaky\tclasses\tshots\n' > "$tsv"
cd app
for w in "$@"; do
  rm -rf test-results
  json="../$out/$engine-w$w.json"
  l0="$(cut -d' ' -f1 /proc/loadavg)"
  t0=$SECONDS
  LEGATUM_E2E_BROWSER="$engine" CI=1 PLAYWRIGHT_JSON_OUTPUT_NAME="$json" \
    nice -n 5 npx playwright test --project=game --workers="$w" --timeout="$timeout" --reporter=json $specs > "../$out/$engine-w$w.stdout" 2> "../$out/$engine-w$w.stderr"
  wall=$((SECONDS - t0)); l1="$(cut -d' ' -f1 /proc/loadavg)"
  sum="$(python3 ../tools/ops/pw-summary.py "$json" --workers "$w")"
  echo "$sum" | tee "../$out/$engine-w$w.summary"
  read -r pass fail flaky < <(echo "$sum" | sed -n 's/^pw-summary: expected=\([0-9]*\) unexpected=\([0-9]*\) flaky=\([0-9]*\).*/\1 \2 \3/p' | head -1)
  classes="$(echo "$sum" | sed -n 's/^pw-summary: failure classes: //p' | head -1)"
  shots="$(find test-results/screen-tour -name '*.png' 2>/dev/null | wc -l | tr -d ' ')"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$engine" "$w" "$wall" "$l0" "$l1" "${pass:-?}" "${fail:-?}" "${flaky:-?}" "${classes:--}" "$shots" >> "../$tsv"
done
cat "../$tsv"

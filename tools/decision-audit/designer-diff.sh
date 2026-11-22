#!/usr/bin/env bash
# One recorded game, its recording engine, and a paired Hard decision diff.
# designer-diff.sh SAVE.legatum ENGINE_TAG docs/playtests/diffs/GAME-decision-diff.md
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
[ "$#" -eq 3 ] || { echo "usage: $0 SAVE.legatum ENGINE_TAG REPORT.md" >&2; exit 2; }
save="$1"; engine="$2"; report="$3"
test -f "$save"
probe="$(mktemp -d "${TMPDIR:-/tmp}/legatum-diff-probe.XXXXXX")"
trap 'rm -rf "$probe"' EXIT
python3 "$root/tools/decision-audit/build_tape.py" "$probe" "$save" >/dev/null
recorded_engine="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["saves"][0]["engine_commit"] or "")' "$probe/tape-info.json")"
if [ -n "$recorded_engine" ] && [ "$(git -C "$root" rev-parse "$engine^{commit}")" != "$recorded_engine" ]; then
  echo "engine tag does not match save metadata" >&2; exit 2
fi
end="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["last"]+1)' "$probe/tape-info.json")"
raw="$(mktemp -d "${TMPDIR:-/tmp}/legatum-designer-diff.XXXXXX")"
echo "decision diff raw workspace: $raw" >&2
"$root/tools/decision-audit/run.sh" --out "$raw" --engine "$engine" --label "designer-diff-$engine" "$save" -- \
  --top 30 --salts 48 --end-turn "$end" --prod-detail --settle-detail --peace-detail --war-detail --army-detail
if [ "$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["hash_mismatches"])' "$raw/fidelity.json")" != 0 ]; then
  echo "skipping $save: replay hash mismatches (details: $raw/fidelity.json)" >&2
  exit 3
fi
attribution="${report%.md}.attribution.json"
if [ -f "$attribution" ]; then
  python3 "$root/tools/decision-audit/diff_report.py" "$raw" --out "$report" --engine "$engine" --attribution "$attribution"
else
  python3 "$root/tools/decision-audit/diff_report.py" "$raw" --out "$report" --engine "$engine"
fi
echo "wrote $report and ${report%.md}.json" >&2
"$root/tools/decision-audit/designer-diff-v2.sh" "$save" "$engine" "${report%.md}"

#!/usr/bin/env bash
# Event-window and each rival-seat analysis for one recorded game.
# designer-diff-v2.sh SAVE.legatum ENGINE_TAG REPORT-PREFIX
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
[ "$#" -eq 3 ] || { echo "usage: $0 SAVE.legatum ENGINE_TAG REPORT-PREFIX" >&2; exit 2; }
save="$1"; engine="$2"; prefix="$3"
probe="$(mktemp -d "${TMPDIR:-/tmp}/legatum-v2-probe.XXXXXX")"
raw="${DIFF_V2_RAW:-$(mktemp -d "${TMPDIR:-/tmp}/legatum-v2.XXXXXX")}"
trap 'rm -rf "$probe"' EXIT
echo "v2 raw workspace: $raw" >&2
python3 "$root/tools/decision-audit/build_tape.py" "$probe" "$save" >/dev/null
read -r recorded end human < <(python3 - "$probe/tape-info.json" <<'PY'
import json,sys
x=json.load(open(sys.argv[1])); print(x['saves'][0]['engine_commit'] or '-',x['last']+1,x['human'][0]['player'])
PY
)
if [ "$recorded" != - ] && [ "$(git -C "$root" rev-parse "$engine^{commit}")" != "$recorded" ]; then
  echo "engine tag does not match save metadata" >&2; exit 2
fi
if [ ! -f "$raw/windows/value.jsonl" ]; then
  "$root/tools/decision-audit/run.sh" --out "$raw/windows" --engine "$engine" --label "diff-windows-$engine" "$save" -- \
    --seat "$human" --event-windows 5 --salts 48 --end-turn "$end"
fi
python3 "$root/tools/decision-audit/v2_report.py" "$raw/windows" --mode window --out "${prefix}-windows.md"
rival_files=()
seats=()
while read -r seat; do seats+=("$seat"); done < <(python3 - "$probe/tape-info.json" "$human" <<'PY'
import json,sys
for seat in json.load(open(sys.argv[1]))['seats']:
    if seat['player'] != int(sys.argv[2]): print(seat['player'])
PY
)
for seat in "${seats[@]}"; do
  if [ ! -f "$raw/rival-$seat/value.jsonl" ]; then
    "$root/tools/decision-audit/run.sh" --out "$raw/rival-$seat" --engine "$engine" --label "diff-rival-$seat-$engine" "$save" -- \
      --seat "$seat" --target-seat "$human" --rival --top 8 --salts 48 --end-turn "$end"
  fi
  python3 "$root/tools/decision-audit/v2_report.py" "$raw/rival-$seat" --mode rival --out "$raw/rival-$seat.md"
  rival_files+=("$raw/rival-$seat.json")
done
python3 "$root/tools/decision-audit/combine_rivals.py" "${prefix}-rivals.md" "${rival_files[@]}"
echo "wrote ${prefix}-windows.md and ${prefix}-rivals.md" >&2

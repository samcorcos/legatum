#!/usr/bin/env bash
# Decision audit of a recorded game, in one command: what Hard (or any decision source) would have
# ordered in his seat each turn, how that differs from what he did, and what the difference is worth.
#
#   tools/decision-audit/run.sh --out DIR [--engine COMMIT|here] [--local] SAVE [LATER_SAVE ...] [-- AUDIT_OPTIONS]
#
# SAVEs are the saves of one game, oldest first (a game the app reloaded mid-way spans two archives;
# the later one's AI memory restore is applied automatically). The engine defaults to the last save's
# `engine_commit`: tapes replay exactly only on the rules that recorded them, and its Hard is the AI he
# played. `--engine here` uses this checkout (for a candidate gene, or a newer Hard on a tape that still
# replays: check `hash_mismatches` and `ai_strategic_order_mismatches` in DIR/fidelity.json).
# AUDIT_OPTIONS go to `crates/sim/examples/decision_audit.rs` (see its header): `--alt params:FILE`
# for a candidate gene, `--alt orders:FILE.jsonl` for any other player, `--every`, `--turns`, `--salts`,
# `--horizons`, `--dice fixed`, `--diff-only`, `--continue hard,params:FILE,...`, `--memory-reset`.
# `--event-windows K` values event-selected category windows. `--rival --seat N --target-seat H
# --top 8` values alternatives from rival N's own view against human H. `--fixture-turns` exports
# hash-verified state and AI memory for ignored decisive-turn regression tests.
# Files named in them are shipped if given as local paths
# under DIR's inputs (pass them with `--ship FILE`, then refer to them as `audit-in/<basename>`).
# Example: --ship tools/decision-audit/continuations/economic.json \
#   --ship tools/decision-audit/continuations/conquest.json SAVE -- \
#   --continue hard,params:audit-in/economic.json,params:audit-in/conquest.json --memory-reset
# Runs on the cloud box (`tools/cloud/remote.sh`); `--local` runs here (small diff-only checks only).
# DIR gets diff.jsonl, value.jsonl, fidelity.json, tape-info.json and report.md.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
die() { echo "decision-audit: $*" >&2; exit 2; }
out=""; engine=""; local_run=0; saves=(); ship=(); label="decision-audit"
while [ $# -gt 0 ]; do
  case "$1" in
    --out) out="${2:?}"; shift 2;;
    --engine) engine="${2:?}"; shift 2;;
    --local) local_run=1; shift;;
    --ship) ship+=("${2:?}"); shift 2;;
    --label) label="${2:?}"; shift 2;;
    --) shift; break;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 0;;
    -*) die "unknown option $1";;
    *) saves+=("$1"); shift;;
  esac
done
[ -n "$out" ] || die "--out DIR is required"
[ ${#saves[@]} -gt 0 ] || die "give at least one save"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
tmp="$(mktemp -d)"
cleanup() { [ -d "$tmp/eng" ] && git -C "$root" worktree remove --force "$tmp/eng" >/dev/null 2>&1; rm -rf "$tmp"; }
trap cleanup EXIT
python3 "$root/tools/decision-audit/build_tape.py" "$tmp/in" "${saves[@]}" > /dev/null
cp "$tmp/in/tape-info.json" "$out/"
for f in "${ship[@]+"${ship[@]}"}"; do cp "$f" "$tmp/in/"; done
mem=""
initial=""
[ ! -f "$tmp/in/initial-state.json" ] || initial=" --initial-state audit-in/initial-state.json"
for m in "$tmp/in"/memory-T*.json; do
  [ -e "$m" ] || continue
  t="$(basename "$m" .json)"; t="${t#memory-T}"
  mem="$mem --memory $t=audit-in/$(basename "$m")"
done
if [ -z "$engine" ]; then
  engine="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["saves"][-1]["engine_commit"])' "$tmp/in/tape-info.json")"
fi
if [ "$engine" = here ]; then
  src="$root"
else
  git -C "$root" cat-file -e "$engine^{commit}" 2>/dev/null || die "engine commit $engine is not in this repository"
  git -C "$root" worktree add --detach "$tmp/eng" "$engine" >/dev/null
  if git -C "$root" cat-file -e "$engine:crates/sim/examples/decision_audit.rs" 2>/dev/null; then
    cp "$root/crates/sim/examples/decision_audit.rs" "$tmp/eng/crates/sim/examples/decision_audit.rs"
  else
    python3 "$root/tools/decision-audit/legacy_adapter.py" \
      "$root/crates/sim/examples/decision_audit.rs" "$tmp/eng/crates/sim/examples/decision_audit.rs" "$root" "$engine"
  fi
  python3 "$root/tools/decision-audit/patch_recording_engine.py" "$tmp/eng"
  src="$tmp/eng"
fi
rm -rf "$src/audit-in"; cp -R "$tmp/in" "$src/audit-in"
cmd="tools/sim/build-slot.sh -- cargo build --release -q -p legatum-sim --example decision_audit && target/release/examples/decision_audit --tape audit-in/tape.json --out audit-out$initial$mem $*"
echo "decision-audit: engine $engine; $cmd"
if [ "$local_run" = 1 ]; then
  (cd "$src" && CARGO_BUILD_JOBS=4 bash -euo pipefail -c "$cmd")
  cp "$src/audit-out/"* "$out/"
  rm -rf "$src/audit-out"
else
  "$root/tools/cloud/remote.sh" "$src" --with-uncommitted --label "$label" --artifact-dir "$tmp/res" --artifact audit-out -- \
    bash -euo pipefail -c "$cmd"
  cp "$tmp/res/audit-out/"* "$out/"
fi
[ "$engine" = here ] && rm -rf "$root/audit-in"
python3 "$root/tools/decision-audit/report.py" "$out" > "$out/report.md" || echo "decision-audit: report.py failed; raw data in $out" >&2
echo "decision-audit: results in $out (report.md)"

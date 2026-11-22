#!/usr/bin/env bash
# Replay a save on the engine it was recorded on and dump the full game state at chosen turns, for
# `cut.sh --state` (a save from an older engine cannot be replayed by the current one: its rules
# differ, so the replay becomes another game). Every turn is checked against the save's recorded hash.
#
#   tools/scenarios/extract-state.sh SAVE --out DIR (--turns 150,160 | --every 3 --from 150 --to 200) [--engine COMMIT] [--audit-plans]
#
# The engine commit comes from the save's own metadata, or pass --engine (a commit of this repository whose
# rules played the game; the log says how many replayed turns differed). It builds on the cloud box, in a
# temporary worktree. DIR gets T<turn>.json for each turn and summary.jsonl: one line per turn with the
# cities, units, wars, the coalition target and who is alive, so you can choose the turn to cut.
# Selected turns also include T<turn>-p<id>.view.json; events.jsonl records every event.
# --audit-plans adds original-brain order comparisons and pre-plan memory in plans.jsonl.
# Needs an engine from 8.9 on (coalitions); for older ones use tools/scenarios/extract-war-states.sh.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
die() { echo "extract-state.sh: $*" >&2; exit 2; }
[ $# -ge 1 ] || { sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 2; }
save="$1"; shift
[ -f "$save" ] || die "no such save: $save"
out=""; turns=""; every=1; from=1; to=100000; engine=""; audit_plans=""
while [ $# -gt 0 ]; do
  case "$1" in
    --out) out="${2:?}"; shift 2;;
    --turns) turns="${2:?}"; shift 2;;
    --every) every="${2:?}"; shift 2;;
    --from) from="${2:?}"; shift 2;;
    --to) to="${2:?}"; shift 2;;
    --audit-plans) audit_plans="audit-plans"; shift;;
    --engine) engine="${2:?}"; shift 2;;
    *) die "unknown option $1";;
  esac
done
[ -n "$out" ] || die "--out DIR is required"
if [ -z "$turns" ]; then
  [ "$to" != 100000 ] || die "give --turns, or --every N --from A --to B"
  turns="$(python3 -c 'import sys; print(",".join(str(t) for t in range(int(sys.argv[1]), int(sys.argv[2]) + 1, int(sys.argv[3]))))' "$from" "$to" "$every")"
fi
[[ "$turns" =~ ^[0-9]+(,[0-9]+)*$ ]] || die "turns must be comma-separated positive integers"
if [ -z "$engine" ]; then
  engine="$(python3 -c 'import json,sys; print(json.loads(json.load(open(sys.argv[1]))["game"])["archive"]["metadata"]["engine_commit"])' "$save")"
fi
git -C "$root" cat-file -e "$engine^{commit}" 2>/dev/null || die "engine commit $engine is not in this repository"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
tmp="$(mktemp -d)"
trap 'git -C "$root" worktree remove --force "$tmp/eng" >/dev/null 2>&1 || true; rm -rf "$tmp"' EXIT
git -C "$root" worktree add --detach "$tmp/eng" "$engine" >/dev/null
cp "$root/tools/scenarios/state_extract.rs" "$tmp/eng/crates/wasm/examples/war_extract.rs"
cp "$save" "$tmp/eng/save.legatum"
echo "extract-state.sh: replaying on engine $engine, dumping turns $turns"
"$root/tools/cloud/remote.sh" "$tmp/eng" --with-uncommitted --label "extract-state" --artifact-dir "$tmp/out" --artifact states -- \
  bash -euo pipefail -c "tools/sim/build-slot.sh -- cargo build --release -q -p legatum-wasm --example war_extract && target/release/examples/war_extract save.legatum states $turns $audit_plans" | tee "$tmp/log"
cp "$tmp/out/states/"* "$out/"
grep -q ' 0 hash mismatches' "$tmp/log" || die "the replay differs from the save (see the line above): $engine is not the engine this game was played on. Try --engine with another commit near the save's date."
echo "extract-state.sh: states in $out (summary.jsonl lists every turn)"

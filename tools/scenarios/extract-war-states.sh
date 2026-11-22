#!/usr/bin/env bash
# Rebuild the start states of the war scenarios (scenarios/war/) from the user's playtest saves.
#
# Each save is replayed with the engine it was recorded on (a detached worktree at that commit
# with tools/scenarios/war_extract.rs dropped in as an example), every turn is checked against
# the recorded state hash, and the full state is written before the scenario's start turn.
# The saves are only read: copies are made first. Then `legatum-sim war-build --states DIR`
# converts the states to the current rules and writes scenarios/war/*.json.
#
# usage: tools/scenarios/extract-war-states.sh OUT_DIR [NAME ...]   (names: only those saves)
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
out="${1:?usage: extract-war-states.sh OUT_DIR [NAME ...]}"
shift
only=" $* "
mkdir -p "$out"
out="$(cd "$out" && pwd)"
work="$out/.engines"
mkdir -p "$work"
renice 15 -p $$ >/dev/null 2>&1 || true

# name | save | engine commit (the save's recorded engine) | start turns | [extra]
# The optional fifth field is `memory`, which also replays the AI brains (tools/scenarios/war_memory.rs) and writes
# memory-T<turn>.json beside each state (every AI seat's saved memory, which war-build attaches), and/or
# `seat:first-last`, which records that seat's orders for the replayed script (both: `memory,0:101-120`).
saves() {
  cat <<LIST
pt6|$HOME/LegatumData/playtest-6-final-saves/autosave.legatum|a05bf36d|25
o86|$HOME/LegatumRuns/bug-saves/playtest-8.6-sparta-win.legatum|a39d0928|65
o84|$HOME/LegatumRuns/bug-saves/playtest-8.4-sparta-win.legatum|900d58c4|51
t85|$HOME/LegatumRuns/bug-saves/playtest-8.5-switzerland-win.legatum|d5205047|46
p892|$HOME/LegatumRuns/bug-saves/p892-mongol-assault-t149-1416.legatum|d50e709cd|136
s892|$HOME/LegatumRuns/bug-saves/p892-sparta-crush-t87.legatum|28be168b|18
a1025|$HOME/LegatumRuns/bug-saves/athens-win-swiss-tepid-20261025.legatum|8e9433e5c|180,220|memory
r1026|$HOME/LegatumRuns/bug-saves/staging-ring-chur-20261026.legatum|0b2ff473c|143|memory
jp|$HOME/LegatumRuns/bug-saves/japan-win-bombers-20261026.legatum|d1233d48|210,219|0:210-221
sj|$HOME/LegatumRuns/bug-saves/sparta-crushing-japan-20261025-0841.legatum|8e9433e5|142
ab|$HOME/LegatumRuns/bug-saves/athens-win-bombers-20261027.legatum|a58e80856|101,150,168|memory,0:101-177
sw|$HOME/LegatumRuns/bug-saves/sparta-win-20261027.legatum|a58e80856|112|memory,0:112-125
s915|$HOME/LegatumRuns/bug-saves/swiss-win-2026-10-28/autosave.legatum|060f78ef4|137|memory,0:137-180
m917|$HOME/LegatumRuns/bug-saves/mongols-hollow-8917-20261030.legatum|0cf2a61d3|76|memory
LIST
}

build_engine() {
  local commit="$1" tree="$work/engine-$1"
  if [ ! -x "$work/war_extract-$commit" ]; then
    [ -d "$tree" ] || git -C "$root" worktree add --detach "$tree" "$commit" >/dev/null
    cp "$root/tools/scenarios/war_extract.rs" "$tree/crates/wasm/examples/war_extract.rs"
    # One target directory per commit: a shared one silently reuses the first commit's binary.
    (cd "$tree" && CARGO_TARGET_DIR="$work/target-$commit" cargo build --release -q -p legatum-wasm --example war_extract)
    cp "$work/target-$commit/release/examples/war_extract" "$work/war_extract-$commit"
  fi
}

build_memory() {
  local commit="$1" tree="$work/engine-$1"
  if [ ! -x "$work/war_memory-$commit" ]; then
    cp "$root/tools/scenarios/war_memory.rs" "$tree/crates/wasm/examples/war_memory.rs"
    (cd "$tree" && CARGO_TARGET_DIR="$work/target-$commit" cargo build --release -q -p legatum-wasm --example war_memory)
    cp "$work/target-$commit/release/examples/war_memory" "$work/war_memory-$commit"
  fi
}

saves | while IFS='|' read -r name save commit turns extra; do
  memory="" orders=""
  for part in ${extra//,/ }; do
    if [ "$part" = memory ]; then memory=memory; else orders="$part"; fi
  done
  [ "$only" = "  " ] || [[ "$only" == *" $name "* ]] || continue
  build_engine "$commit"
  cp "$save" "$work/$name.legatum"
  "$work/war_extract-$commit" "$work/$name.legatum" "$out/$name" "$turns" "${orders:-}"
  if [ "$memory" = memory ]; then
    build_memory "$commit"
    "$work/war_memory-$commit" "$work/$name.legatum" "$out/$name" "$turns"
  fi
done
echo "states in $out; next: legatum-sim war-build --states $out"

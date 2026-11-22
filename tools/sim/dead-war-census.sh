#!/usr/bin/env bash
# Dead-war census (run on the box: tools/cloud/remote.sh <worktree> --with-uncommitted --artifact OUT -- tools/sim/dead-war-census.sh OUT [games] [params.json ...]):
# eight-player Hard games per arm (the built-in AI, then each params file), tapes replayed by the
# war_peace_metrics example (wars nobody fights, how long they last, Peace offers refused). Every
# arm plays the same seeds. Writes OUT/<arm>.json.
set -euo pipefail
out="$1"; games="${2:-128}"; shift 2 || true
seed=7600001
mkdir -p "$out"
cargo build --release -p legatum-sim --bin legatum-sim --example war_peace_metrics 2>&1 | grep -E '^error' -A8 | head -40 || true
dir="$(mktemp -d)"; trap 'rm -rf "$dir"' EXIT
run() { # name extra-args...
  local name="$1"; shift
  /usr/bin/time -p target/release/legatum-sim --players 8 --difficulty hard --barbarians true --games "$games" --seed "$seed" --threads 20 \
    --tape-out "$dir/$name/tapes" --out-dir "$dir/$name/out" --out "$dir/$name/replays" --label "deadwar-$name" "$@" > "$dir/$name.log" 2>&1 || { tail -20 "$dir/$name.log"; exit 1; }
  tail -4 "$dir/$name.log"
  target/release/examples/war_peace_metrics "$dir/$name/tapes" 20 10 > "$out/$name.json"
  echo "=== $name"; cat "$out/$name.json"
}
run base
for params in "$@"; do
  run "$(basename "$params" .json)" --ai-params "$params"
done

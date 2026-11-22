#!/usr/bin/env bash
# The learned value network pipeline (AI war plan step 3; docs/sim-results/value-net/README.md).
# Each command runs ON THE BUILD BOX, through remote.sh (the Mac stays idle):
#
#   tools/cloud/remote.sh . --artifact out/value-smoke -- bash tools/sim/value-net.sh smoke [GAMES]
#       build, the value_net/winchance unit tests, then GAMES (default 384) Hard self-play games
#       (8,8,8,4 players, every 10 turns) -> out/value-smoke
#   tools/cloud/remote.sh . --artifact out/value-time -- bash tools/sim/value-net.sh time
#       the unit tests (golden vectors), then the cost per estimate and per whole-lobby turn, gene off
#       against on, one thread per game
#   tools/cloud/remote.sh . --artifact out/value-league -- bash tools/sim/value-net.sh league [GAMES]
#       GAMES (default 512) 8-player Hard games, learned_value=1 in half the seats against plain Hard
#   tools/cloud/remote.sh . --artifact out/value-fit -- bash tools/sim/value-net.sh train SRC [LABEL]
#       train on SRC (a run directory or an s3:// prefix the box can read, i.e. under box/) in a cached
#       venv; writes out/value-fit/{value_net.json,fit.json}. Needs python3-venv, which the box lacks
#       today (2026-10-31): train on the Mac instead (README).
#   tools/cloud/remote.sh . --artifact out/value-regret -- bash tools/sim/value-net.sh regret [GAMES] [SAMPLES]
#       a small run of the regret harness (value_regret); the real run goes to spot (README)
#   ... value-net.sh smoke-train [GAMES]   smoke, then train on it (checks the box's training path)
#
# The full training run goes to spot with tools/cloud/run-batch.sh --bin value_data (README).
set -euo pipefail
cd "$(dirname "$0")/../.."
cmd="${1:?usage: value-net.sh smoke|time|league|train|smoke-train ...}"
cores="$(nproc 2>/dev/null || sysctl -n hw.ncpu)"
case "$cmd" in
  smoke)
    games="${2:-384}"
    cargo build --release -p legatum-sim --example value_data 2>&1 | tail -1
    cargo test --release -p legatum-ai --lib -- value_net winchance 2>&1 | tail -4
    rm -rf out/value-smoke; mkdir -p out/value-smoke
    RAYON_NUM_THREADS="$cores" target/release/examples/value_data --games "$games" --seed 8800001 \
      --players 8,8,8,4 --every 10 --label smoke --out-dir out/value-smoke
    ls -la out/value-smoke
    ;;
  time)
    cargo test --release -p legatum-ai --lib -- value_net winchance 2>&1 | tail -4
    cargo build --release -p legatum-sim --example value_net_time 2>&1 | tail -1
    rm -rf out/value-time; mkdir -p out/value-time
    pids=()
    for s in 7000001 7000002 7000003 7000004 7000005 7000006; do
      RAYON_NUM_THREADS=1 target/release/examples/value_net_time --seeds "$s" --turns 260 \
        > "out/value-time/$s.txt" & pids+=($!)
    done
    for p in "${pids[@]}"; do wait "$p"; done
    cat out/value-time/*.txt
    ;;
  league)
    games="${2:-512}"
    cargo build --release -p legatum-sim --bin legatum-sim 2>&1 | tail -1
    rm -rf out/value-league; mkdir -p out/value-league
    printf '{"schema":1,"name":"learned-value","decisions":{"learned_value":1}}\n' > out/value-league/on.json
    printf '{"schema":1,"name":"builtin"}\n' > out/value-league/off.json
    target/release/legatum-sim --players 8 --difficulty hard --games "$games" --seed 8900001 \
      --ai-params out/value-league/on.json --opponent-params out/value-league/off.json \
      --label value-league --out-dir out/value-league 2>&1 | tail -5
    ls out/value-league
    ;;
  train)
    src="${2:?train needs a run directory or an s3:// prefix}"
    venv="${LEGATUM_VALUE_VENV:-$HOME/.cache/legatum-value-venv}"
    [ -x "$venv/bin/pip" ] || python3 -m venv "$venv" || {
      echo "value-net: no python3 venv here (the box lacks python3-venv); train on the Mac instead (README)" >&2; exit 3; }
    "$venv/bin/pip" install -q -r tools/sim/requirements-value.txt
    data="$src"
    case "$src" in
      s3://*) data=out/value-data; mkdir -p "$data"
        aws s3 sync --only-show-errors "$src" "$data" --exclude '*' --include '*value-*';;
    esac
    rm -rf out/value-fit; mkdir -p out/value-fit
    "$venv/bin/python" tools/sim/train-value.py --data "$data" --data-label "${3:-$src}" --threads "$cores" \
      --out out/value-fit/value_net.json --report out/value-fit/fit.json 2>&1 | grep -v ' epoch '
    ;;
  regret)
    games="${2:-8}"; samples="${3:-4}"
    cargo build --release -p legatum-sim --example value_regret 2>&1 | tail -1
    rm -rf out/value-regret; mkdir -p out/value-regret
    RAYON_NUM_THREADS="$cores" target/release/examples/value_regret --games "$games" --seed 8880001 \
      --samples "$samples" --label smoke --out-dir out/value-regret
    python3 tools/sim/value-regret-report.py out/value-regret/value-regret-smoke.jsonl
    ;;
  smoke-train)
    bash "$0" smoke "${2:-384}"
    bash "$0" train out/value-smoke "value-smoke: ${2:-384} Hard self-play games from seed 8800001, players 8,8,8,4, every 10 turns"
    ;;
  *) echo "unknown command $cmd" >&2; exit 2;;
esac

#!/usr/bin/env bash
# One-command card loop over every Lab card, on this Mac: build the simulator, score Hard on each
# card, search every failing card and write one task per divergence to
# docs/sim-results/ai-quality/card-loop-queue/ (see tools/sim/card-loop-README.md).
# Arguments go to tools/sim/card_loop_nightly.py (e.g. --score-only, --cards ID ..., --max-evals N).
# Reads designer runs from ~/LegatumData/scenarios but never writes there.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
tools/sim/build-slot.sh -- cargo build --release -q -p legatum-sim
python3 tools/sim/card_loop_nightly.py "$@"

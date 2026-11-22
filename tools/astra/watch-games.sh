#!/usr/bin/env bash
# Poll finished human archives and review a daily sample of 20 simulated games.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
# Its own build directory: sharing target/ with the lead's test runs caused E0460 artifact clashes.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/analysis}"
exec python3 "$root/tools/astra/ai_analysis.py" watch "$@"

#!/usr/bin/env bash
# Run all known-gap decision fixtures, or the one whose Rust test name is supplied.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
tools/sim/build-slot.sh -- cargo test -p legatum-sim --test decisive_turns --profile release-checked "${1:-}" -- --ignored --nocapture

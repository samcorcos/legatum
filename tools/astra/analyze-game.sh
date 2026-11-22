#!/usr/bin/env bash
# Verified replay -> bounded Astra review -> validated high-confidence queue proposals.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
exec python3 "$root/tools/astra/ai_analysis.py" review "$@"

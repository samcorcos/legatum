#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
exec python3 "$(dirname "$0")/pw-bridge.py" "$@"

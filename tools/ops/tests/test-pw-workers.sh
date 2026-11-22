#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")/.." && pwd)/pw-workers.sh"

failures=0
while IFS=' ' read -r cores load expected; do
  [ -n "${cores:-}" ] || continue
  actual="$(pw_workers "$cores" "$load")"
  if [ "$actual" != "$expected" ]; then
    echo "FAIL cores=$cores load=$load expected=$expected got=$actual"
    failures=$((failures + 1))
  else
    echo "PASS cores=$cores load=$load -> $actual"
  fi
done <<'CASES'
1 0 2
4 0 2
12 0 4
24 0 8
64 0 21
96 0 24
16 8 4
16 12 2
16 100 2
CASES

[ "$failures" -eq 0 ] || exit 1

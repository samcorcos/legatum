#!/usr/bin/env bash
# Negative-path test for tools/release/export-public.sh: --push must never be combinable with --no-scan,
# and the rejection must happen before anything is exported or cloned.
# Usage: tools/release/test-export-public.sh
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
script="$here/export-public.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
fail=0

check_rejected() {
  local name="$1"; shift
  local out rc
  # PUBLIC_REPO points at a path that cannot exist, so any attempt to reach the push stage would fail elsewhere.
  out="$(PUBLIC_REPO="$work/no-such-repo.git" "$script" --out "$work/out" "$@" 2>&1)"; rc=$?
  if [ "$rc" = 2 ] && grep -q 'cannot be combined' <<<"$out" && [ ! -e "$work/out" ]; then
    echo "ok: $name"
  else
    echo "FAIL: $name (exit $rc, out dir exists: $([ -e "$work/out" ] && echo yes || echo no))" >&2; echo "$out" >&2; fail=1
  fi
}

check_rejected "--push --no-scan" --push --no-scan
check_rejected "--no-scan --push" --no-scan --push
check_rejected "--push --no-scan with --ref" --ref HEAD --push --no-scan

if "$script" --help 2>&1 | grep -q 'cannot be combined with --push'; then echo "ok: --help documents the rule"
else echo "FAIL: --help does not document the rule" >&2; fail=1; fi

exit "$fail"

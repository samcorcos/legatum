#!/usr/bin/env bash
# budget.sh: a job's share of the cores follows slots held + queued, and older jobs get overridden.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"; trap 'kill $(jobs -p) 2>/dev/null || true; rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin" "$tmp/slots"
cat > "$tmp/bin/flock" <<'PERL'
#!/usr/bin/env perl
use strict; use warnings; use Fcntl qw(LOCK_EX LOCK_NB LOCK_UN);
my $mode = ''; $mode = shift @ARGV if @ARGV && $ARGV[0] =~ /^-/;
my $fd = shift @ARGV; open(my $lock, "<&=$fd") or die "open fd $fd: $!\n";
my $op = $mode eq '-u' ? LOCK_UN : LOCK_EX | ($mode eq '-n' ? LOCK_NB : 0);
exit(flock($lock, $op) ? 0 : 1);
PERL
printf '#!/bin/sh\necho 64\n' > "$tmp/bin/nproc"; chmod +x "$tmp/bin/flock" "$tmp/bin/nproc"
export PATH="$tmp/bin:$PATH" SLOTDIR="$tmp/slots" SLOTS=12 DATA="$tmp"
fail() { echo "FAIL: $*" >&2; exit 1; }
# Run inside a box job (verify's tools-check), the environment already carries that job's own budget and sizes, which would
# make every case below see an "older job" and keep them: start from a clean environment.
unset LEGATUM_CORE_BUDGET BOX_CARGO_JOBS RAYON_NUM_THREADS NEXTEST_TEST_THREADS RUST_TEST_THREADS BINARYEN_CORES CARGO_BUILD_JOBS \
  LEGATUM_VITEST_WORKERS VITEST_MAX_WORKERS LEGATUM_PW_WORKERS LEGATUM_VERIFY_JOBS LEGATUM_VERIFY_TEST_THREADS \
  LEGATUM_VERIFY_SIM_THREADS LEGATUM_VERIFY_PW_WORKERS LEGATUM_VERIFY_VITEST_WORKERS
budget() { ( . "$here/budget.sh"; "$@" >/dev/null 2>&1; box_core_budget_apply >/dev/null; echo "$LEGATUM_CORE_BUDGET $BOX_CARGO_JOBS $NEXTEST_TEST_THREADS $BINARYEN_CORES" ); }
hold() { # hold <count>: that many slot locks held by a background process
  for k in $(seq 1 "$1"); do ( exec 9>"$SLOTDIR/s$k.lock"; flock 9; sleep 30 ) & disown; done; sleep 0.8
}
printf '1\n' > "$SLOTDIR/serving"; printf '1\n' > "$SLOTDIR/next-ticket"
got="$(budget true)"; [ "$got" = "20 20 20 10" ] || fail "an empty box gives a lone job 20 cores: $got"
hold 12
got="$(budget true)"; [ "$got" = "6 6 6 3" ] || fail "12 held slots give 6 cores each: $got"
printf '50\n' > "$SLOTDIR/next-ticket"
got="$(budget true)"; [ "$got" = "6 6 6 3" ] || fail "a long queue never goes under SLOTS-based sharing: $got"
got="$( export BOX_CARGO_JOBS=24 RUST_TEST_THREADS=8; . "$here/budget.sh"; box_core_budget_apply >/dev/null; echo "$LEGATUM_CORE_BUDGET $BOX_CARGO_JOBS")"
[ "$got" = "6 6" ] || fail "an older job's cargo jobs=24 must be overridden: $got"
got="$( export LEGATUM_CORE_BUDGET=9 BOX_CARGO_JOBS=3; . "$here/budget.sh"; box_core_budget_apply >/dev/null; echo "$LEGATUM_CORE_BUDGET $BOX_CARGO_JOBS $NEXTEST_TEST_THREADS")"
[ "$got" = "9 3 9" ] || fail "a job started by the new box-job keeps its budget and explicit settings: $got"
got="$( . "$here/budget.sh"; box_core_budget_apply >/dev/null; box_verify_sizes; echo "$LEGATUM_VERIFY_JOBS $LEGATUM_VERIFY_TEST_THREADS $LEGATUM_VERIFY_PW_WORKERS")"
[ "$got" = "2 6 2" ] || fail "verify sizes from a budget of 6: $got"
echo "budget test passed: lone job 20 cores, full box 6, older jobs overridden, explicit settings kept"

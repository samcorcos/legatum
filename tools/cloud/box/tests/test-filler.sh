#!/usr/bin/env bash
# The idle filler drains its queue in priority order, collects outputs and DONE, and honours cancel (local store, no S3,
# no systemd scope).
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"; trap 'kill $(jobs -p) 2>/dev/null || true; rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin" "$tmp/store/pending" "$tmp/data/bin"
cat > "$tmp/bin/flock" <<'PERL'
#!/usr/bin/env perl
use strict; use warnings; use Fcntl qw(LOCK_EX LOCK_NB LOCK_UN);
my $mode = ''; $mode = shift @ARGV if @ARGV && $ARGV[0] =~ /^-/;
my $fd = shift @ARGV; open(my $lock, "<&=$fd") or die "open fd $fd: $!\n";
my $op = $mode eq '-u' ? LOCK_UN : LOCK_EX | ($mode eq '-n' ? LOCK_NB : 0);
exit(flock($lock, $op) ? 0 : 1);
PERL
chmod +x "$tmp/bin/flock"
mkdir -p "$tmp/slots"
export PATH="$tmp/bin:$PATH" FILLER_SLOTDIR="$tmp/slots" SLOTS=2 DATA="$tmp/data" FILLER_LOCAL_STORE="$tmp/store" FILLER_NO_SCOPE=1 FILLER_SKIP_BUILD=1 \
  FILLER_IDLE_ROUNDS=2 FILLER_POLL_SECONDS=0.2 FILLER_WATCH_SECONDS=0.2
fail() { echo "FAIL: $*" >&2; exit 1; }
spec() { # id priority script
  jq -n --arg id "$1" --argjson p "$2" --arg s "$3" '{id:$id, priority:$p, argv:["bash","-c",$s], inputs:{"in/arms.json":"{\"a\":1}"}, outputs:["out/r.txt"]}' > "$tmp/store/pending/$1.json"
}
spec late 50 'mkdir -p out; echo late > out/r.txt; cat in/arms.json >> out/r.txt'
spec first 5 'mkdir -p out; echo first > out/r.txt; sleep 0.3'
spec boom 70 'exit 3'
"$here/box-filler" run > "$tmp/daemon.log" 2>&1 || { cat "$tmp/daemon.log"; fail "daemon failed"; }
for i in first late boom; do [ -f "$tmp/store/results/$i/DONE" ] || { cat "$tmp/daemon.log"; fail "no DONE for $i"; }; done
[ "$(cat "$tmp/store/results/first/out/r.txt")" = first ] || fail "output of first not uploaded"
grep -q '"a":1' "$tmp/store/results/late/out/r.txt" || fail "the inline input did not reach the batch"
grep -q '^rc=0 ' "$tmp/store/results/first/DONE" || fail "rc=0 expected for first"
grep -q '^rc=3 ' "$tmp/store/results/boom/DONE" || fail "rc=3 expected for boom"
order="$(grep -o 'start [a-z]*' "$tmp/daemon.log" | tr '\n' ' ')"
[ "$order" = "start first start late start boom " ] || fail "priority order wrong: $order"
"$here/box-filler" run > "$tmp/daemon1b.log" 2>&1
grep -q 'start ' "$tmp/daemon1b.log" && fail "a finished batch ran again (the box role cannot delete pending specs)"
echo "PASS: batches run in priority order with inputs, outputs, DONE and the exit code"

jq -n '{id:"slow", priority:1, argv:["bash","-c","sleep 60"], outputs:[]}' > "$tmp/store/pending/slow.json"
"$here/box-filler" run > "$tmp/daemon2.log" 2>&1 &
d=$!
for _ in $(seq 1 50); do grep -q 'start slow' "$tmp/daemon2.log" 2>/dev/null && break; sleep 0.2; done
mkdir -p "$tmp/store/cancel"; : > "$tmp/store/cancel/slow"
wait "$d"
grep -q '^rc=cancelled ' "$tmp/store/results/slow/DONE" || { cat "$tmp/daemon2.log"; fail "cancel did not stop the batch"; }
echo "PASS: a cancel marker stops the running batch"

# A claim left by an earlier daemon instance (reboot, crash) with no DONE is abandoned work and runs again.
spec abandoned 1 'echo again > out.txt'
mkdir -p "$tmp/store/running"; printf 'old-instance' > "$tmp/store/running/abandoned"
"$here/box-filler" run > "$tmp/daemon3.log" 2>&1
grep -q '^rc=0 ' "$tmp/store/results/abandoned/DONE" || { cat "$tmp/daemon3.log"; fail "an abandoned claim was not picked up again"; }
echo "PASS: a claim from an earlier daemon instance is rerun"

# No filler work while the slots are full: with both slots held it must not start, and it starts once one is free.
spec gated 1 'echo ran > out.txt'
( exec 9>"$tmp/slots/s1.lock"; flock 9; exec 8>"$tmp/slots/s2.lock"; flock 8; sleep 3 ) &
sleep 0.5
FILLER_IDLE_ROUNDS=30 "$here/box-filler" run > "$tmp/daemon4.log" 2>&1 &
d4=$!
sleep 1.5
grep -q 'start gated' "$tmp/daemon4.log" && fail "the filler started a batch while every slot was held"
wait "$d4"
grep -q '^rc=0 ' "$tmp/store/results/gated/DONE" || { cat "$tmp/daemon4.log"; fail "the gated batch never ran after the slots freed"; }
echo "PASS: the filler waits while every slot is held and runs when two are free"

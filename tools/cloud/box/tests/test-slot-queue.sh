#!/usr/bin/env bash
# Local-only FIFO and dead-waiter checks for slot-queue.sh.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
holder_pid=""
cleanup() {
  [ -z "$holder_pid" ] || kill "$holder_pid" 2>/dev/null || true
  rm -rf "$tmp"
}
trap cleanup EXIT

# macOS has flock(2) but does not ship the Linux flock command. This shim
# applies the same lock to the inherited descriptor for local test runs.
mkdir -p "$tmp/bin"
cat > "$tmp/bin/flock" <<'PERL'
#!/usr/bin/env perl
use strict;
use warnings;
use Fcntl qw(LOCK_EX LOCK_NB LOCK_UN);
my $mode = '';
$mode = shift @ARGV if @ARGV && $ARGV[0] =~ /^-/;
my $fd = shift @ARGV;
open(my $lock, "<&=$fd") or die "open fd $fd: $!\n";
my $op = $mode eq '-u' ? LOCK_UN : LOCK_EX | ($mode eq '-n' ? LOCK_NB : 0);
exit(flock($lock, $op) ? 0 : 1);
PERL
chmod +x "$tmp/bin/flock"
PATH="$tmp/bin:$PATH"
export PATH
. "$here/slot-queue.sh"
log() { echo "$*" >> "$tmp/log"; }
wait_for_tickets() {
  local expected="$1" tries=0 count
  while :; do
    count="$(find "$SLOTDIR" -maxdepth 1 -name 'ticket.*' -type f | wc -l | tr -d ' ')"
    [ "$count" -ge "$expected" ] && return 0
    tries=$((tries + 1))
    [ "$tries" -lt 1200 ] || { echo "timed out waiting for $expected queue tickets" >&2; exit 1; }
    sleep 0.05
  done
}
start_holder() {
  local dir="$1"
  (
    exec 9>"$dir/s1.lock"
    flock 9
    : > "$dir/holder.ready"
    while [ ! -e "$dir/release" ]; do sleep 0.05; done
  ) &
  holder_pid=$!
  for _ in $(seq 1 200); do [ -e "$dir/holder.ready" ] && return 0; sleep 0.05; done
  echo "slot holder failed to start" >&2
  exit 1
}
release_holder() {
  : > "$SLOTDIR/release"
  wait "$holder_pid"
  holder_pid=""
}

SLOTDIR="$tmp/fifo-slots"
SLOTS=1
SLOT_POLL_SECONDS=1
export SLOTDIR SLOTS SLOT_POLL_SECONDS
mkdir -p "$SLOTDIR"
start_holder "$SLOTDIR"
for name in A B C; do
  SLOT_QUEUE="$here/slot-queue.sh" ACQUIRED="$tmp/acquired" WAITER_NAME="$name" \
    bash -c '. "$SLOT_QUEUE"; pick_slot; echo "$WAITER_NAME" >> "$ACQUIRED"' &
  sleep 1
done
wait_for_tickets 3
release_holder
for pid in $(jobs -p); do wait "$pid"; done
[ "$(cat "$tmp/acquired")" = $'A\nB\nC' ] || {
  echo "waiters acquired out of order: $(tr '\n' ' ' < "$tmp/acquired")" >&2
  exit 1
}

# A killed waiter leaves a ticket behind; the next live waiter must pass it.
SLOTDIR="$tmp/dead-slots"
export SLOTDIR
mkdir -p "$SLOTDIR"
start_holder "$SLOTDIR"
SLOT_QUEUE="$here/slot-queue.sh" DEAD_ACQUIRED="$tmp/dead-acquired" \
  bash -c '. "$SLOT_QUEUE"; pick_slot; echo "dead waiter unexpectedly acquired a slot" >> "$DEAD_ACQUIRED"' &
dead_pid=$!
wait_for_tickets 1
kill -KILL "$dead_pid"
wait "$dead_pid" 2>/dev/null || true
SLOT_QUEUE="$here/slot-queue.sh" AFTER_DEAD="$tmp/after-dead" \
  bash -c '. "$SLOT_QUEUE"; pick_slot; echo acquired > "$AFTER_DEAD"' &
next_pid=$!
# Not wait_for_tickets 2: the live waiter removes the dead one's ticket as soon as it runs, so on a fast machine
# the count never reaches 2 (the box). The ticket counter says the second waiter has queued.
tries=0
while [ "$(cat "$SLOTDIR/next-ticket" 2>/dev/null || echo 1)" -lt 3 ]; do
  tries=$((tries + 1))
  [ "$tries" -lt 1200 ] || { echo "timed out waiting for the second waiter to take a ticket" >&2; exit 1; }
  sleep 0.05
done
release_holder
wait "$next_pid"
[ "$(cat "$tmp/after-dead")" = acquired ]
[ ! -e "$tmp/dead-acquired" ]
# Transition from 5 to 12 slots: an old-style head waiter (live ticket, no v2 marker) only ever tries s1..LEGACY_SLOTS.
# A new waiter behind it must still get a free overflow slot instead of waiting for the old one.
SLOTDIR="$tmp/legacy-slots"
export SLOTDIR
mkdir -p "$SLOTDIR"
sleep 120 & legacy_pid=$!
printf '1\n' > "$SLOTDIR/serving"; printf '2\n' > "$SLOTDIR/next-ticket"; printf '%s\n' "$legacy_pid" > "$SLOTDIR/ticket.1"
start_holder "$SLOTDIR"          # s1 is held; the old head waiter can never advance
LEGACY_SLOTS=1 SLOTS=2 SLOT_QUEUE="$here/slot-queue.sh" bash -c '. "$SLOT_QUEUE"; pick_slot; echo "$SLOT" > "'"$tmp"'/overflow"' &
overflow_pid=$!
for _ in $(seq 1 100); do [ -s "$tmp/overflow" ] && break; sleep 0.1; done
[ "$(cat "$tmp/overflow" 2>/dev/null)" = s2 ] || { kill "$legacy_pid" 2>/dev/null; echo "overflow slot not taken behind an old-style head (got: $(cat "$tmp/overflow" 2>/dev/null))" >&2; exit 1; }
wait "$overflow_pid"
kill "$legacy_pid" 2>/dev/null || true
release_holder
echo "slot queue tests passed: FIFO order A, B, C; killed waiter skipped; overflow slots usable behind an old-style head"

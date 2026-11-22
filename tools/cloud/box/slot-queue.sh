#!/usr/bin/env bash
# FIFO slot assignment for box jobs. SLOTDIR, SLOTS, PREFER_PATH and log are
# supplied by box-job (and can be supplied by local tests).

_slot_queue_log() {
  if declare -F log >/dev/null; then log "$*"; else echo "$*"; fi
}

_slot_queue_skip_dead() {
  local serving="$1" next="$2" pid
  while [ "$serving" -lt "$next" ]; do
    if [ -r "$SLOTDIR/ticket.$serving" ]; then
      read -r pid < "$SLOTDIR/ticket.$serving" || pid=""
      kill -0 "$pid" 2>/dev/null && break
    fi
    rm -f "$SLOTDIR/v2.$serving"
    rm -f "$SLOTDIR/ticket.$serving"   # a dead waiter's ticket would otherwise stay forever (hundreds had piled up)
    serving=$((serving + 1))
    printf '%s\n' "$serving" > "$SLOTDIR/serving.tmp"
    mv -f "$SLOTDIR/serving.tmp" "$SLOTDIR/serving"
  done
}

_slot_queue_position() {
  local serving="$1" ticket="$2" n pid position=0
  for ((n = serving; n <= ticket; n++)); do
    [ -r "$SLOTDIR/ticket.$n" ] || continue
    read -r pid < "$SLOTDIR/ticket.$n" || pid=""
    if kill -0 "$pid" 2>/dev/null; then position=$((position + 1)); fi
  done
  printf '%s' "$position"
}

# 0 when a live waiter with a v2 marker (it can use every slot) is queued ahead of ticket $2, from $1 on.
_slot_queue_new_ahead() {
  local serving="$1" ticket="$2" n pid
  for ((n = serving; n < ticket; n++)); do
    [ -e "$SLOTDIR/v2.$n" ] && [ -r "$SLOTDIR/ticket.$n" ] || continue
    read -r pid < "$SLOTDIR/ticket.$n" || pid=""
    kill -0 "$pid" 2>/dev/null && return 0
  done
  return 1
}

_slot_queue_ordinal() {
  case "$1" in
    *1[123]) printf '%sth' "$1" ;;
    *1) printf '%sst' "$1" ;;
    *2) printf '%snd' "$1" ;;
    *3) printf '%srd' "$1" ;;
    *) printf '%sth' "$1" ;;
  esac
}

_slot_queue_advance() {
  local ticket="$1" serving next
  exec 8>"$SLOTDIR/queue.lock"
  flock 8
  serving="$(cat "$SLOTDIR/serving")"
  next="$(cat "$SLOTDIR/next-ticket")"
  _slot_queue_skip_dead "$serving" "$next"
  serving="$(cat "$SLOTDIR/serving")"
  if [ "$serving" -eq "$ticket" ]; then
    rm -f "$SLOTDIR/ticket.$ticket"
    serving=$((serving + 1))
    printf '%s\n' "$serving" > "$SLOTDIR/serving.tmp"
    mv -f "$SLOTDIR/serving.tmp" "$SLOTDIR/serving"
  fi
  flock -u 8
  exec 8>&-
}

pick_slot() { # sets SLOT and retains fd 9 on the selected slot lock
  local ticket serving next position waited=0 last_logged=-60 k order name busy
  local poll="${SLOT_POLL_SECONDS:-5}" legacy="${LEGACY_SLOTS:-5}" new_ahead
  # Transition: waiters that were queued before SLOTS went from 5 to 12 run the old code and never look past s5, and
  # strict FIFO makes the head waiter block everyone behind it. A waiter with no new-style waiter ahead of it may
  # therefore take s6.. while the head is an old one; once the old ones drain this never fires.
  mkdir -p "$SLOTDIR"
  exec 8>"$SLOTDIR/queue.lock"
  flock 8
  [ -f "$SLOTDIR/next-ticket" ] || printf '1\n' > "$SLOTDIR/next-ticket"
  [ -f "$SLOTDIR/serving" ] || printf '1\n' > "$SLOTDIR/serving"
  next="$(cat "$SLOTDIR/next-ticket")"
  ticket="$next"
  # Never via $(...): a command substitution runs in a subshell whose BASHPID is dead
  # at once, so every waiter looked dead, was skipped, and the queue starved (2026-10-11).
  printf '%s\n' "${BASHPID:-$$}" > "$SLOTDIR/ticket.$ticket"
  : > "$SLOTDIR/v2.$ticket"   # waiters from before 12 slots only ever try s1..LEGACY_SLOTS; this one tries all
  printf '%s\n' "$((next + 1))" > "$SLOTDIR/next.tmp"
  mv -f "$SLOTDIR/next.tmp" "$SLOTDIR/next-ticket"
  flock -u 8
  exec 8>&-

  while :; do
    exec 8>"$SLOTDIR/queue.lock"
    flock 8
    serving="$(cat "$SLOTDIR/serving")"
    next="$(cat "$SLOTDIR/next-ticket")"
    _slot_queue_skip_dead "$serving" "$next"
    serving="$(cat "$SLOTDIR/serving")"
    position="$(_slot_queue_position "$serving" "$ticket")"
    new_ahead=1; _slot_queue_new_ahead "$serving" "$ticket" || new_ahead=0
    flock -u 8
    exec 8>&-

    # -ge, not -eq: if serving ever passes a live ticket, that waiter is at the head, not stranded.
    if [ "$serving" -ge "$ticket" ]; then
      order=""
      # Keep the warm-worktree preference for the head waiter.
      if [ -n "${PREFER_PATH:-}" ]; then
        for k in $(seq 1 "$SLOTS"); do [ -e "$SLOTDIR/s$k/$PREFER_PATH" ] && order="$order s$k"; done
      fi
      for k in $(seq 1 "$SLOTS"); do case " $order " in *" s$k "*) ;; *) order="$order s$k";; esac; done
      busy=1
      for name in $order; do
        exec 9>"$SLOTDIR/$name.lock"
        if flock -n 9; then
          SLOT="$name"
          _slot_queue_advance "$ticket"
          rm -f "$SLOTDIR/ticket.$ticket" "$SLOTDIR/v2.$ticket"
          return 0
        fi
      done
    else
      busy=1
      if [ "$new_ahead" = 0 ] && [ "$SLOTS" -gt "$legacy" ]; then
        for k in $(seq $((legacy + 1)) "$SLOTS"); do
          exec 9>"$SLOTDIR/s$k.lock"
          if flock -n 9; then
            SLOT="s$k"
            # Not the head: leave serving alone; skip_dead steps over this removed ticket later.
            rm -f "$SLOTDIR/ticket.$ticket" "$SLOTDIR/v2.$ticket"
            return 0
          fi
        done
      fi
    fi

    if [ "$busy" = 1 ] && [ $((waited - last_logged)) -ge 60 ]; then
      _slot_queue_log "waiting for a slot: $(_slot_queue_ordinal "${position:-1}") in queue"
      last_logged="$waited"
    fi
    sleep "$poll"
    waited=$((waited + poll))
  done
}

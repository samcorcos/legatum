#!/usr/bin/env bash
# Keeps the Codex CLI workers busy (Astra, Sol or Luna per task). Two loops, started with:
#   tools/astra/runner.sh queue <k> # worker k: works through .progress/astra-queue/pending/*.md in lane k
#   tools/astra/runner.sh reviewer  # reviews every new origin/master commit range
#
# Queue task file format: first lines are headers, then a blank line, then the prompt.
#   mode: review | write        (review = read-only; write = own git worktree on branch astra/<name>)
#   name: short-kebab-name
#   model: astra | sol | luna   (optional; default sol). luna = gpt-6-luna; astra and sol = gpt-6-sol,
#          with astra defaulting to high effort and sol to medium; used for implementation, analysis and reviews.
#   effort: low | medium | high (optional; default astra=high, sol=medium, luna=low)
# Outputs: .reviews/astra-<name>.md (Astra's final message), logs in .progress/astra.log.
# GitHub is the source of truth (user, 2026-11-21): lanes branch from a freshly fetched origin/master and
# a finished write task's branch is pushed to origin (never force) for a PR into master. Nothing is merged
# automatically: the lead (Claude) reviews every result and merges the PR once CI and reviews pass.
set -uo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
# An Xcode update with an unaccepted license breaks /usr/bin/git, cc and ld machine-wide (2026-10-04).
# The Command Line Tools are licensed separately, so fall back to them until the user accepts.
if /usr/bin/git --version 2>&1 | grep -q "license"; then export DEVELOPER_DIR=/Library/Developer/CommandLineTools; fi
queue="$root/.progress/astra-queue"
mkdir -p "$queue/pending" "$queue/running" "$queue/done" "$queue/hold" "$root/.reviews"
log() { echo "$(date +%FT%T) | $*" >> "$root/.progress/astra.log"; }
# The base every lane starts from: origin/master after a fetch. Falls back to local master when there
# is no origin remote (an offline or private checkout).
base_ref=master
refresh_base() { # sets base_ref; never fails the caller
  if git remote get-url origin >/dev/null 2>&1; then
    local i
    for i in 1 2 3; do git fetch -q origin master >/dev/null 2>&1 && break; sleep $((i * 5)); done
    git rev-parse -q --verify origin/master >/dev/null 2>&1 && { base_ref=origin/master; return 0; }
  fi
  base_ref=master
}
commit_leftovers() { # $1 worktree, $2 branch, $3 task name: commit what Codex could not
  # The Codex sandbox can't write git metadata (index.lock, FETCH_HEAD) for a worktree whose gitdir lives in the
  # main checkout (2026-11-21), so a task's edits may be uncommitted. The runner, outside the sandbox, commits them.
  [ "$(git -C "$1" symbolic-ref -q --short HEAD 2>/dev/null)" = "$2" ] || return 0
  [ -n "$(git -C "$1" status --porcelain --untracked-files=normal 2>/dev/null)" ] || return 0
  git -C "$1" add -A >/dev/null 2>&1 && git -C "$1" commit -q -m "Codex task $3: work left uncommitted in the sandbox" >/dev/null 2>&1 \
    && log "committed uncommitted work on $2" || log "commit of leftovers failed on $2"
}
push_branch() { # $1 worktree, $2 branch: push new commits for a PR; never force
  git remote get-url origin >/dev/null 2>&1 || return 0
  [ -n "$(git -C "$1" rev-list "$base_ref..$2" 2>/dev/null | head -1)" ] || { log "no commits to push on $2"; return 0; }
  if git -C "$1" push -q -u origin "$2" >/dev/null 2>&1; then log "pushed $2 to origin (open a PR into master)"
  else log "push failed for $2 (push it by hand)"; fi
}
# Loops restart themselves between tasks when this script changes, so tooling fixes reach the
# workers without killing them (bash keeps executing the old copy otherwise). $1 = loop args.
self="$root/tools/astra/runner.sh"; self_mtime="$(stat -f %m "$self")"
reload_if_changed() {
  [ "$(stat -f %m "$self" 2>/dev/null)" = "$self_mtime" ] && return
  bash -n "$self" 2>/dev/null || return # never exec into a half-written or broken script
  log "runner.sh changed; restarting loop ($*)"; exec bash "$self" "$@"
}

# Models (user, 2026-10-24: "use astra... it's stronger for things that require max intelligence"): astra = gpt-6-astra (max-intelligence reviews and audits), sol = gpt-6-sol, luna = gpt-6-luna.
model_slug() { case "$1" in luna) echo gpt-6-luna ;; astra) echo gpt-6-astra ;; *) echo gpt-6-sol ;; esac; }
default_effort() { case "$1" in astra) echo high ;; luna) echo low ;; *) echo medium ;; esac; }

# Heavy CPU stays off the Mac (user, 2026-10-11: the laptop's battery sat at 7-8% under load 78). Write tasks get a
# PATH of shims (tools/astra/lane-bin) that send cargo test/nextest/clippy/check, npx vitest/tsc/playwright and
# npm test/build to the cloud build box, and scripts such as repin.sh and build-wasm.sh offload themselves
# (LEGATUM_OFFLOAD=1; tools/cloud/offload.sh). Codex runs every command through `zsh -lc`, which reorders PATH, so
# ZDOTDIR points at tools/astra/lane-zsh to put the shims back first. Box unreachable: the commands fall back to
# running here at low priority (the whole codex process tree starts under nice -n 15, because a Codex sandbox denies
# setpriority, so a command cannot nice itself). Kill switch: `touch .progress/astra-queue/no-offload`.
lane_env=() # set by run_codex: env assignments for write tasks (read-only tasks run nothing heavy)
set_lane_env() { # $1 sandbox
  lane_env=()
  [ "$1" = workspace-write ] || return 0
  lane_env=(LEGATUM_OFFLOAD=1 LEGATUM_LANE_NICED=1 "LEGATUM_LANE_BIN=$root/tools/astra/lane-bin" "ZDOTDIR=$root/tools/astra/lane-zsh" "PATH=$root/tools/astra/lane-bin:$PATH")
}

run_codex() { # $1 sandbox, $2 dir, $3 out file, $4 prompt, $5 model key (astra|sol|luna), $6 effort
  local key="${5:-sol}"
  set_lane_env "$1"
  python3 "$root/tools/ops/timed.py" codex "$key" --label "$(basename "$3" .md)" -- \
  nice -n 15 env ${lane_env[@]+"${lane_env[@]}"} codex exec -s "$1" -C "$2" -m "$(model_slug "$key")" \
    -c "model_reasoning_effort=\"${6:-$(default_effort "$key")}\"" \
    -c 'sandbox_workspace_write.writable_roots=["'"$HOME"'/.cargo","'"$HOME"'/LegatumRuns","'"$HOME"'/.cache/huggingface","/tmp","'"$root"'/.git","'"$root"'/.progress"]' \
    -c 'sandbox_workspace_write.network_access=true' \
    -o "$3" "$4" > "$3.log" 2>&1 &
  local pid=$!
  echo "$pid" > "$3.pid" # the reaper checks this PID; pgrep on the huge argv missed live tasks
  wait "$pid"; local rc=$?
  rm -f "$3.pid"
  return "$rc"
}

# Lanes (2026-10-04): each queue worker owns one persistent, locked worktree
# (.claude/worktrees/lane-<k>) and switches it to each task's branch, so builds are incremental
# instead of cold (a fresh worktree compiles the whole workspace in 3 profiles, ~26 GB of target).
# A branch already checked out in an older per-task worktree keeps using that worktree.
lane=""
prepare_lane() { # $1 task name; sets prepared_wt on success, lane_setup_error on failure
  local name="$1" branch="astra/$1" wt existing status
  prepared_wt=""; lane_setup_error=""
  existing="$(git worktree list --porcelain | awk -v b="refs/heads/$branch" '/^worktree /{w=substr($0,10)} $0=="branch " b {print w}')" || { lane_setup_error="git worktree list failed"; return 1; }
  if [ -n "$existing" ]; then
    if ! git -C "$existing" merge -q "$base_ref" >/dev/null 2>&1; then
      git -C "$existing" merge --abort >/dev/null 2>&1 || true
      lane_setup_error="merge $base_ref into existing worktree failed ($existing)"; return 1
    fi
    prepared_wt="$existing"; return 0
  fi
  wt="$root/.claude/worktrees/lane-$lane"
  if [ ! -d "$wt" ]; then
    git worktree add -q --detach "$wt" "$base_ref" >&2 || { lane_setup_error="git worktree add failed ($wt)"; return 1; }
    git worktree lock --reason "astra lane $lane (persistent)" "$wt" >&2 || { lane_setup_error="git worktree lock failed ($wt)"; return 1; }
  fi
  if [ ! -e "$wt/app/node_modules" ]; then
    ln -s "$root/app/node_modules" "$wt/app/node_modules" || { lane_setup_error="node_modules symlink failed ($wt)"; return 1; }
  fi
  # Never lose the previous task's leftovers: commit them on its branch (or a rescue branch).
  status="$(git -C "$wt" status --porcelain --untracked-files=normal)" || { lane_setup_error="git status failed ($wt)"; return 1; }
  if [ -n "$status" ]; then
    git -C "$wt" symbolic-ref -q HEAD >/dev/null || git -C "$wt" checkout -q -b "lane-$lane-rescue-$(date +%Y%m%d%H%M%S)" >&2 || { lane_setup_error="rescue branch checkout failed ($wt)"; return 1; }
    git -C "$wt" add -A >&2 && git -C "$wt" commit -q -m "WIP left in lane $lane before switching tasks" >&2 || { lane_setup_error="rescue commit failed ($wt)"; return 1; }
  fi
  if git show-ref -q --verify "refs/heads/$branch"; then
    git -C "$wt" checkout -q "$branch" >&2 || { lane_setup_error="branch checkout failed ($branch)"; return 1; }
    if ! git -C "$wt" merge -q "$base_ref" >/dev/null 2>&1; then
      git -C "$wt" merge --abort >/dev/null 2>&1 || true
      lane_setup_error="merge $base_ref failed ($branch)"; return 1
    fi
  else
    git -C "$wt" checkout -q -b "$branch" "$base_ref" >&2 || { lane_setup_error="branch creation failed ($branch)"; return 1; }
  fi
  prepared_wt="$wt"
}

queue_loop() {
  lane="${1:?usage: runner.sh queue <lane number>}"
  while true; do
    reload_if_changed queue "$lane"
    # Lanes above the number in $queue/max-lanes finish their current task, then take no new ones.
    if [ "$lane" -gt "$(cat "$queue/max-lanes" 2>/dev/null || echo 99)" ]; then sleep 60; continue; fi
    task="$(ls -1tr "$queue/pending"/*.md 2>/dev/null | head -1)"
    if [ -z "$task" ]; then sleep 30; continue; fi
    # Never take a task while git is unusable (2026-10-04: an Xcode update left its license
    # unaccepted, every git call failed, and one worker failed the whole queue in 6 seconds).
    if ! git rev-parse -q --verify master >/dev/null 2>&1; then
      log "lane $lane paused: git unusable ($(git --version 2>&1 | head -1 | cut -c1-90))"; sleep 300; continue
    fi
    base="$(basename "$task")"
    # mv is atomic: if another worker grabbed this task first, skip it.
    mv "$task" "$queue/running/$base" 2>/dev/null || continue
    task="$queue/running/$base"
    touch "$task" # the reaper leaves tasks younger than 5 minutes alone (lane preparation)
    started=$SECONDS
    mode="$(sed -n 's/^mode: *//p' "$task" | head -1)"
    name="$(sed -n 's/^name: *//p' "$task" | head -1)"
    model="$(sed -n 's/^model: *//p' "$task" | head -1)"; model="${model:-sol}"
    effort="$(sed -n 's/^effort: *//p' "$task" | head -1)"; effort="${effort:-$(default_effort "$model")}"
    prompt="$(awk 'f{print} /^$/{f=1}' "$task")"
    out="$root/.reviews/astra-$name.md"
    log "started task: $name ($mode, $(model_slug "$model") $effort)"
    if [ "$mode" = "write" ]; then
      refresh_base
      if ! prepare_lane "$name"; then
        mv "$task" "$queue/hold/$base"
        log "lane $lane setup failed for $name: $lane_setup_error → hold/"
        sleep 300; continue
      fi
      wt="$prepared_wt"
      full="$prompt

You are working in your own git worktree ($wt) on branch astra/$name, branched from $base_ref. While iterating, run only the tests your change touches. Run the full checks once, when you are done: the ones your task lists, or else, if you changed Rust code, cargo fmt --all, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace --profile release-checked at the end only (overflow checks and debug assertions stay enabled in this optimized profile). Wrap Cargo builds, tests and clippy runs in tools/sim/build-slot.sh -- <cargo command>. Heavy commands run off this laptop (its battery is low): cargo test/nextest/clippy/check, npx vitest/tsc/playwright, npm test and npm run build, tools/sim/repin.sh and scripts/build-wasm.sh are forwarded to a cloud build box automatically and behave exactly like local runs (same output, same exit code, the files they change come back), so use them as usual and add no workarounds; if the box is unreachable they fall back to running here at low priority. Plain cargo build, cargo run and any binary you build or run yourself still run on this laptop, so keep them small. Sims, benchmarks and anything that takes more than a few minutes go to AWS spot (tools/sim/hard-vs-human.sh runs in the cloud by default; tools/cloud/run-batch.sh), never a long local run, and never --local unless a task tells you to. To run some other command on the box and bring its outputs back: tools/cloud/remote.sh . --with-uncommitted --artifact <path> -- <command>. If $base_ref has moved by then (it is fetched before your task starts; don't fetch inside the sandbox), merge it and resolve conflicts (regenerate pins and fixtures with the documented commands, never by hand), then re-run only the determinism and pin tests: the landing re-runs every check on the merged code. Commit your work on this branch (if git can't write its index or lock files in your sandbox, leave the changes uncommitted: the runner commits them when you finish) with messages ending in the line 'Co-Authored-By: $(model_slug "$model") (Codex) <noreply@openai.com>'. Do not merge into master and do not push; the runner pushes your branch for a pull request and the lead reviews it. Do NOT edit docs/learnings.md or docs/learnings/ (parallel edits conflict); put non-obvious learnings in your final message instead. Before designing or changing rules, AI or UI, read the relevant project skills in .claude/skills/ (legatum-rules for how the game works; legatum-design and legatum-ui for the user's preferences and rejected ideas); keep the matching file under legatum-rules/references/ (see its index.md) updated for any rule you change. Your final message: what you changed, exact check results, anything uncertain."
      run_codex workspace-write "$wt" "$out" "$full" "$model" "$effort"
      wstatus=$?
      commit_leftovers "$wt" "astra/$name" "$name"
      push_branch "$wt" "astra/$name"
      (exit "$wstatus")
    else
      run_codex read-only "$root" "$out" "$prompt" "$model" "$effort"
    fi
    status=$?
    # Model capacity errors are transient: re-queue (the branch keeps its work) and back off,
    # up to 3 tries per task, instead of filing the task as failed.
    if [ "$status" -ne 0 ] && tail -c 4000 "$out.log" 2>/dev/null | grep -q "at capacity"; then
      tries=$(grep -c "capacity retry: $name" "$root/.progress/astra.log" 2>/dev/null)
      if [ "${tries:-0}" -lt 3 ]; then
        mv "$task" "$queue/pending/$base"
        log "capacity retry: $name (exit $status; try $((tries + 1))/3) → pending/; lane $lane backs off 10 min"
        sleep 600; continue
      fi
    fi
    # Circuit breaker: a task failing within 2 minutes is an environment fault, not the task's.
    # Park it in hold/ and pause this worker instead of draining the queue.
    if [ "$status" -ne 0 ] && [ $((SECONDS - started)) -lt 120 ]; then
      mv "$task" "$queue/hold/$base"
      log "fast failure: $name (exit $status in $((SECONDS - started))s) → hold/; lane $lane pauses 5 min"
      sleep 300; continue
    fi
    mv "$task" "$queue/done/$base"
    log "finished task: $name (exit $status) → .reviews/astra-$name.md"
  done
}

changes_code() { # $1 commit range; true if it touches anything but docs, skills, agents and plans
  ! git diff --quiet "$1" -- . ':(exclude)docs' ':(exclude).claude/skills' ':(exclude).claude/agents' ':(exclude)plans' ':(exclude)CLAUDE.md' 2>/dev/null
}

reviewer_loop() {
  state="$root/.progress/astra-last-reviewed"
  while true; do
    reload_if_changed reviewer
    refresh_base
    head="$(git rev-parse --verify -q "$base_ref")"
    last="$(cat "$state" 2>/dev/null || echo "")"
    if [ -n "$head" ] && [ "$head" != "$last" ]; then
      range="${last:+$last..}$head"
      [ -z "$last" ] && range="$head~5..$head"
      # Skip ranges that change only prose (2026-10-04 audit: 28 of 52 reviews that day were
      # docs/skills-only, ~570K Sol tokens, one minor finding). Code, tools and hooks are reviewed.
      if ! changes_code "$range"; then
        log "skipped prose-only $base_ref range ($range)"; echo "$head" > "$state"; sleep 60; continue
      fi
      short="$(git rev-parse --short "$head")"
      out="$root/.reviews/astra-master-$short.md"
      log "reviewing new $base_ref commits ($range)"
      run_codex read-only "$root" "$out" "You are an independent code reviewer for Legatum, a deterministic Rust 4X game engine (crates/core, crates/ai, crates/sim, crates/wasm) with a web UI (app/). Contract: plans/first-playable.md. Review ONLY the changes in \`git diff $range\` (use git log/diff to read them; read surrounding code as needed). Report real defects only: rule bugs vs the contract, determinism hazards (floats, HashMap iteration, u64->usize casts, call-order-dependent RNG, native vs wasm32 differences), panics reachable from valid input, broken tests or checks, security issues. For each: file:line, the concrete failure scenario, and the minimal fix. If you find nothing real, say 'No defects found.' No style comments. Be concise." sol medium
      echo "$head" > "$state"
      log "master review done → .reviews/astra-master-$short.md"
    fi
    sleep 60
  done
}

reaper_loop() {
  # Moves running tasks whose codex process has exited (e.g. after a worker restart) to done.
  while true; do
    reload_if_changed reaper
    for task in "$queue/running"/*.md; do
      [ -e "$task" ] || continue
      [ -n "$(find "$task" -mmin -5 2>/dev/null)" ] && continue # still being prepared
      name="$(sed -n 's/^name: *//p' "$task" | head -1)"
      pidf="$root/.reviews/astra-$name.md.pid"
      [ -f "$pidf" ] && kill -0 "$(cat "$pidf")" 2>/dev/null && continue # live codex: never reap
      if ! pgrep -f "astra-$name.md" > /dev/null; then # reads only; never kills
        sleep 5
        pgrep -f "astra-$name.md" > /dev/null && continue
        mv "$task" "$queue/done/" 2>/dev/null && log "finished task: $name (reaped) → .reviews/astra-$name.md"
      fi
    done
    sleep 30
  done
}

case "${1:-}" in
  analysis) exec "$root/tools/astra/watch-games.sh" "${@:2}" ;;
  reaper) reaper_loop ;;
  queue) queue_loop "${2:-}" ;;
  reviewer) reviewer_loop ;;
  *) echo "usage: $0 queue|reviewer|reaper|analysis [watcher options]" >&2; exit 1 ;;
esac

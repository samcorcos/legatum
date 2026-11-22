#!/bin/bash
# Run (or reuse) a legatum-sim baseline on a given master commit, shared by every agent.
#
#   tools/sim/cached-baseline.sh [--commit <sha>] [--build-only] -- <legatum-sim args without --out-dir/--label>
#   --build-only     build (or reuse) master's sim binary, print its path and stop; no arguments needed
#
# Agents kept re-measuring master on the same seeds in parallel. This builds master's sim once per
# version of the simulation code and stores each distinct argument set once
# (~/LegatumRuns/baseline-cache/code-<key>/<args-hash>/). The key hashes crates/, Cargo.toml and
# Cargo.lock, so the many master commits that only touch docs, skills, tools or the app reuse the
# previous binary and results instead of rebuilding. Builds share one persistent, locked worktree
# (~/LegatumRuns/baseline-src) so cargo only recompiles crates that changed. Concurrent callers
# wait for the first run instead of duplicating it; runs take thread tokens from tools/sim/slot.sh.
# Prints the result directory on stdout; reports are inside it.
set -uo pipefail
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
commit=""
build_only=0
while [[ "${1:-}" == --commit || "${1:-}" == --build-only ]]; do
  if [[ "$1" == "--commit" ]]; then commit="$2"; shift 2; else build_only=1; shift; fi
done
[[ "${1:-}" == "--" ]] && shift
[[ $build_only -eq 1 || $# -gt 0 ]] || { echo "usage: $0 [--commit <sha>] [--build-only] -- <legatum-sim args>" >&2; exit 2; }
commit="$(git -C "$repo" rev-parse "${commit:-master}")"
code="code-$(git -C "$repo" rev-parse "$commit:crates" "$commit:Cargo.toml" "$commit:Cargo.lock" |
  shasum -a 256 | cut -c1-16)"

root="${LEGATUM_RUNS:-$HOME/LegatumRuns}"
bin_dir="$root/baseline-bin/$code"
bin="$bin_dir/legatum-sim"
key="$(printf '%s\n' "$@" | shasum -a 256 | cut -c1-16)"
out="$root/baseline-cache/$code/$key"

# Locks are directories holding a subdirectory named after the owner's pid; a dead owner's lock
# is broken, so a killed caller can't leave everyone else waiting forever.
lock() {
  until mkdir "$1" 2>/dev/null; do
    owner="$(ls "$1" 2>/dev/null | head -1)"
    if [[ -n "$owner" ]] && ! kill -0 "$owner" 2>/dev/null; then rmdir "$1/$owner" "$1" 2>/dev/null; fi
    sleep 5
  done
  mkdir "$1/$$"
}
unlock() { rmdir "$1/$$" "$1"; }

# 1. Build master's sim once per code version.
if [[ ! -x "$bin" ]]; then
  mkdir -p "$root/baseline-bin"
  lock "$root/baseline-build.lock"
  if [[ ! -x "$bin" ]]; then
    src="$root/baseline-src"
    if [[ -d "$src" ]]; then
      # The shared worktree belongs to whichever checkout created it. On the cloud box every job slot is its own
      # clone, so the commit may not be in that repository yet: fetch it from this one.
      git -C "$src" cat-file -e "$commit^{commit}" 2>/dev/null || git -C "$src" fetch -q "$repo" "$commit" >&2
      git -C "$src" checkout -q --detach --force "$commit" >&2
    else
      git -C "$repo" worktree add -q --detach "$src" "$commit" >&2 &&
        git -C "$repo" worktree lock --reason "shared baseline build (tools/sim/cached-baseline.sh)" "$src" >&2
    fi &&
      (cd "$src" && nice -n 10 cargo build -q --release -p legatum-sim >&2) &&
      mkdir -p "$bin_dir" && cp "$src/target/release/legatum-sim" "$bin.tmp" && mv "$bin.tmp" "$bin" &&
      echo "$commit" > "$bin_dir/commit"
    status=$?
    unlock "$root/baseline-build.lock"
    [[ $status -eq 0 ]] || { echo "build of $commit failed" >&2; exit 1; }
  else
    unlock "$root/baseline-build.lock"
  fi
fi

if [[ $build_only -eq 1 ]]; then echo "$bin"; exit 0; fi

# 2. Run each distinct argument set once.
if [[ -f "$out/.done" ]]; then echo "$out"; exit 0; fi
mkdir -p "$root/baseline-cache/$code"
lock "$out.lock"
if [[ ! -f "$out/.done" ]]; then
  mkdir -p "$out"
  printf '%s\n' "$@" > "$out/args.txt"
  echo "$commit" > "$out/commit"
  nice -n 10 "$repo/tools/sim/slot.sh" -- "$bin" "$@" --label "baseline-$key" --out-dir "$out" > "$out/run.log" 2>&1
  status=$?
  [[ $status -eq 0 ]] && touch "$out/.done"
  unlock "$out.lock"
  [[ $status -eq 0 ]] || { echo "baseline run failed; see $out/run.log" >&2; exit 1; }
else
  unlock "$out.lock"
fi
echo "$out"

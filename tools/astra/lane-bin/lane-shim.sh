#!/usr/bin/env bash
# PATH shim for Codex lanes: cargo, npx, npm and wasm-pack are symlinks to this file (tools/astra/runner.sh puts this
# directory first on PATH for every write task, and tools/astra/lane-zsh/ keeps it first in Codex's login shell).
# Heavy verification commands run on the cloud build box through tools/cloud/offload.sh and behave exactly like the
# local ones (same output stream, same exit code, changed files applied); everything else runs here, niced.
#
#   cargo test|nextest|clippy|check     -> box
#   npx vitest|tsc|playwright, npm test | run build | run e2e | run test:e2e   -> box (the box rebuilds the WASM package)
#   everything else (build, run, fmt, wasm-pack, npm ci, ...)  -> local, nice -n 15
# Why not cargo build / cargo run: they produce binaries this Mac runs afterwards, and the Linux build is no use here.
# Sims belong on AWS spot (tools/cloud/run-batch.sh); scripts whose outputs matter (repin.sh, build-wasm.sh,
# hard-vs-human.sh) offload themselves. Off switch: LEGATUM_OFFLOAD unset/0, or the file .progress/astra-queue/no-offload.
set -uo pipefail
tool="$(basename "$0")"
bin_dir="$(cd "$(dirname "$0")" && pwd)"
export LEGATUM_LANE_BIN="$bin_dir"
offload="$bin_dir/../../cloud/offload.sh"

real_path() {
  local out="" d IFS=:
  for d in $PATH; do [ "$d" = "$bin_dir" ] || out="${out:+$out:}$d"; done
  printf '%s' "$out"
}
niced_exec() { # exec at low priority; the Codex sandbox denies setpriority, so lanes start under nice -n 15 (LEGATUM_LANE_NICED=1)
  if [ "${LEGATUM_LANE_NICED:-0}" = 1 ] || [ -n "$(nice -n 15 true 2>&1)" ]; then exec "$@"; else exec nice -n 15 "$@"; fi
}
run_local() { local p; p="$(real_path)"; PATH="$p"; niced_exec "$tool" "$@"; }

[ "${LEGATUM_OFFLOAD:-0}" = 1 ] && [ -z "${LEGATUM_ON_BOX:-}" ] || { p="$(real_path)"; PATH="$p" exec "$tool" "$@"; }

first_word() { # first argument that is not a flag or a +toolchain
  local a
  for a in "$@"; do case "$a" in -*|+*) ;; *) printf '%s' "$a"; return;; esac; done
}

case "$tool" in
  cargo)
    case "$(first_word "$@")" in test|nextest|clippy|check) ;; *) run_local "$@";; esac
    # Apple targets cannot build on the Linux box.
    case "$PWD $*" in *apple*|*darwin*) run_local "$@";; esac
    exec "$offload" -- cargo "$@" ;;
  npx)
    case "$(first_word "$@")" in
      vitest|tsc) exec "$offload" --prep wasm -- npx "$@" ;;
      playwright) exec "$offload" --prep web --artifact app/test-results -- npx "$@" ;;
      *) run_local "$@" ;;
    esac ;;
  npm)
    case "$(first_word "$@")" in
      test) exec "$offload" --prep wasm -- npm "$@" ;;
      run|run-script)
        shift_args=("$@"); script=""; seen=0
        for a in "${shift_args[@]}"; do
          if [ "$seen" = 1 ]; then case "$a" in -*) ;; *) script="$a"; break;; esac
          else case "$a" in run|run-script) seen=1;; esac; fi
        done
        case "$script" in
          build|test) exec "$offload" --prep wasm -- npm "$@" ;;
          e2e|test:e2e) exec "$offload" --prep web --artifact app/test-results -- npm "$@" ;;
          *) run_local "$@" ;;
        esac ;;
      *) run_local "$@" ;;
    esac ;;
  *) run_local "$@" ;;
esac

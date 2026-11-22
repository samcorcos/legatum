#!/usr/bin/env bash
# Make the morning report and the narrated film of an evolution run.
#   tools/film/make-film.sh docs/sim-results/evolve-<run-id> [render options]
# Render options pass through to tools/film/render.mjs: --no-llm (template story), --fresh-story,
# --voice <name>, --workers <n>, --out <dir>. Resumable: voice clips, replays and rendered scenes
# are cached in ~/LegatumData/films/cache/<run-id>/, so a rerun only redoes what changed.
# Output: ~/LegatumData/films/<run-id>.mp4 plus .srt, .vtt, .jpg, .story.md and .script.json.
set -euo pipefail
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
run_dir="${1:?usage: tools/film/make-film.sh <run-dir> [render options]}"
shift
cd "$repo"
command -v ffmpeg >/dev/null || { echo "ffmpeg is required (brew install ffmpeg)" >&2; exit 1; }
command -v say >/dev/null || { echo "macOS say is required for narration" >&2; exit 1; }
if [ ! -x "${LEGATUM_SIM:-target/release/legatum-sim}" ]; then
  cargo build --release -p legatum-sim
fi
node tools/film/report.mjs "$run_dir"
node tools/film/render.mjs "$run_dir" "$@"

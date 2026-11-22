#!/usr/bin/env bash
# Re-record and rebuild every piece of README media from the running app, in one command:
#   docs/media/src/refresh.sh                 record all clips, then rebuild docs/media
#   docs/media/src/refresh.sh --no-capture    rebuild docs/media from the masters already recorded
#   MEDIA_ONLY=battle,factions docs/media/src/refresh.sh    re-record only those clips, then rebuild
# Masters (1920x1080, 60 fps) go to MEDIA_WORK (default ~/LegatumData/media-masters); they are not committed.
# Needs: the Rust toolchain and wasm-pack, Node 22 with app dependencies, Playwright's Chromium, ffmpeg and img2webp.
# Set LEGATUM_WASM_NO_OPT=1 where wasm-opt cannot be downloaded. Recording takes about an hour on four cores.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
work="${MEDIA_WORK:-$HOME/LegatumData/media-masters}"
capture=1
[ "${1:-}" = "--no-capture" ] && capture=0
masters="$work/masters"
mkdir -p "$masters"
cd "$root"
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

if [ "$capture" = 1 ]; then
  [ -f app/src/wasm-pkg/legatum_bg.wasm ] || LEGATUM_PLAYTEST_PROFILE=1 scripts/build-wasm.sh
  [ -d app/node_modules ] || (cd app && npm ci)
  (cd app && npm run build)
  (cd app && MEDIA_OUT="$masters" npx playwright test -c playwright.media.config.ts launch.media.ts --trace off)
fi

out="$root/docs/media"
# The teaser: captions, then two passes to the README's size budget (1920x1080, 60 fps, with the soundtrack).
node "$here/captions.mjs" "$work/captions"
node "$here/teaser.mjs" "$masters" "$work/captions" "$out/legatum-teaser.mp4" --kbps "${TEASER_KBPS:-1350}"

# README loops: name, master, start, seconds, width, fps, quality, optional crop (w:h:x:y in the 1920x1080 master).
loop() {
  local name="$1" clip="$2" ss="$3" t="$4" w="$5" fps="$6" q="$7" crop="${8:-}" src="$masters/$2.mp4"
  if [ -n "$crop" ]; then
    ffmpeg -y -loglevel error -ss "$ss" -t "$t" -i "$src" -vf "crop=$crop" -c:v libx264 -crf 12 -preset fast "$work/crop-$name.mp4"
    "$here/webp.sh" "$work/crop-$name.mp4" "$out/$name.webp" 0 "$t" "$w" "$fps" "$q"
  else
    "$here/webp.sh" "$src" "$out/$name.webp" "$ss" "$t" "$w" "$fps" "$q"
  fi
}
# shellcheck source=/dev/null
. "$here/loops.sh"
# The teaser's preview (the first seconds, silent) for the README.
"$here/webp.sh" "$out/legatum-teaser.mp4" "$out/teaser-preview.webp" 0 8 720 8 36

node "$here/banner.mjs"
node "$here/diagrams.mjs"
ls -l "$out"/*.mp4 "$out"/*.webp "$out"/*.jpg | awk '{printf "%8.2f MB  %s\n", $5/1048576, $9}'

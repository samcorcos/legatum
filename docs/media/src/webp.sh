#!/usr/bin/env bash
# Encode a README loop as an animated WebP from a master capture:
#   webp.sh <in.mp4> <out.webp> [start=0] [duration=6] [width=720] [fps=12] [quality=60]
# About a third of the size of the same loop as a GIF, with full colour (the painted map bands badly in a 256-colour palette).
# Uses img2webp when it is installed, otherwise ffmpeg's libwebp_anim encoder.
set -euo pipefail
in="${1:?in.mp4}"; out="${2:?out.webp}"; ss="${3:-0}"; t="${4:-6}"; w="${5:-720}"; fps="${6:-12}"; q="${7:-60}"
if command -v img2webp >/dev/null; then
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  ffmpeg -y -loglevel error -ss "$ss" -t "$t" -i "$in" -vf "fps=$fps,scale=$w:-1:flags=lanczos" "$tmp/%04d.png"
  img2webp -loop 0 -d "$((1000 / fps))" -lossy -q "$q" "$tmp"/*.png -o "$out" >/dev/null
else
  ffmpeg -y -loglevel error -ss "$ss" -t "$t" -i "$in" -an -vf "fps=$fps,scale=$w:-1:flags=lanczos" -c:v libwebp_anim -lossless 0 -q:v "$q" -compression_level 6 -loop 0 "$out"
fi
ls -l "$out" | awk '{printf "%s %.2f MB\n", $9, $5/1048576}'

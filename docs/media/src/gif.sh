#!/usr/bin/env bash
# Encode a README loop from a master capture: gif.sh <in.mp4> <out.gif> [start=0] [duration=6] [width=720] [fps=12] [colors=128]
# Two-pass palette (per-clip stats, rectangle diffs) keeps the painted map smooth at a small size.
set -euo pipefail
in="${1:?in.mp4}"; out="${2:?out.gif}"; ss="${3:-0}"; t="${4:-6}"; w="${5:-720}"; fps="${6:-12}"; colors="${7:-128}"
ffmpeg -y -loglevel error -ss "$ss" -t "$t" -i "$in" -vf "fps=$fps,scale=$w:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=$colors:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=4:diff_mode=rectangle" -loop 0 "$out"
ls -l "$out" | awk '{printf "%s %.2f MB\n", $9, $5/1048576}'

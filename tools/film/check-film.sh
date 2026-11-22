#!/usr/bin/env bash
# Check a rendered film: streams, duration, black frames, and a contact sheet of stills to look at.
#   tools/film/check-film.sh ~/LegatumData/films/<run-id>.mp4 [out-dir] [n-stills=8]
set -euo pipefail
film="${1:?usage: tools/film/check-film.sh <film.mp4> [out-dir] [n]}"
out="${2:-$(dirname "$film")/stills}"
n="${3:-8}"
mkdir -p "$out"
ffprobe -v error -show_entries format=duration:stream=codec_name,profile,level,width,height,r_frame_rate,pix_fmt,color_range,sample_rate,channels -of compact "$film"
dur="$(ffprobe -v error -show_entries format=duration -of default=nw=1:nk=1 "$film")"
echo "black frames (>0.25 s):"
ffmpeg -hide_banner -nostats -i "$film" -vf "blackdetect=d=0.25:pix_th=0.06" -an -f null - 2>&1 | grep -o "black_start.*" || echo "  none"
base="$(basename "$film" .mp4)"
inputs=()
for i in $(seq 1 "$n"); do
  t="$(awk -v d="$dur" -v i="$i" -v n="$n" 'BEGIN { printf "%.2f", d * (i - 0.5) / n }')"
  ffmpeg -y -loglevel error -ss "$t" -i "$film" -frames:v 1 -q:v 3 "$out/$base-$i.jpg"
  echo "still $i at ${t}s: $out/$base-$i.jpg"
  inputs+=(-i "$out/$base-$i.jpg")
done
layout=""
for i in $(seq 0 $((n - 1))); do
  x=$(( i % 2 )); y=$(( i / 2 ))
  xs="0"; [ "$x" = 1 ] && xs="w0"
  ys="0"; [ "$y" -gt 0 ] && ys="$(printf 'h0+%.0s' $(seq 1 "$y") | sed 's/+$//')"
  layout="$layout${layout:+|}${xs}_${ys}"
done
ffmpeg -y -loglevel error "${inputs[@]}" -filter_complex "xstack=inputs=$n:layout=$layout,scale=1600:-1" "$out/$base-sheet.jpg"
echo "sheet: $out/$base-sheet.jpg"

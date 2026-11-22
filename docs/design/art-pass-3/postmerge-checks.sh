#!/usr/bin/env bash
# Run on the cloud box. Preserve all statuses; a visual similarity failure must not skip pin checks.
set -uo pipefail
out=docs/design/art-pass-3
python3 tools/art/distinctness.py --render --check --report "$out/distinctness.md" --sheet "$out/distinctness-pairs.png" 2>&1 | tee "$out/distinctness.log"
art_rc=${PIPESTATUS[0]}
tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked -- pinned 2>&1 | tee "$out/pins.log"
pins_rc=${PIPESTATUS[0]}
tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked --test determinism 2>&1 | tee "$out/determinism.log"
det_rc=${PIPESTATUS[0]}
printf 'distinctness=%s\npins=%s\ndeterminism=%s\n' "$art_rc" "$pins_rc" "$det_rc" > "$out/postmerge-status.txt"
if (( art_rc || pins_rc || det_rc )); then exit 1; fi

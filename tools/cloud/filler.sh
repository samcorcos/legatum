#!/usr/bin/env bash
# Queue idle-filler work for the cloud build box (tools/cloud/box/box-filler): pure-CPU batches that run at the lowest
# CPU priority on whatever cores the real jobs leave, so the 64 cores are never idle while there is work to do.
#
#   tools/cloud/filler.sh submit SPEC.json [--commit REF]   queue a batch (starts the box and the filler daemon)
#   tools/cloud/filler.sh ensure                            start the box and the daemon (after queueing by hand)
#   tools/cloud/filler.sh status                            queue, running batch and recent results
#   tools/cloud/filler.sh fetch ID [DIR]                    copy results/<ID>/ (outputs, output.log, DONE) to DIR (default .)
#   tools/cloud/filler.sh cancel ID                         stop a queued or running batch
#
# Spec (JSON). A batch is one command that is pure CPU, deterministic and safe to kill and rerun:
#   {"id": "climb-g7", "priority": 10,                lower number runs first (default 50)
#    "commit": "<sha>",                               optional: build at this commit (submit --commit REF ships the objects)
#    "build": ["cargo","build","--profile","sim","-p","legatum-sim","--example","milestone_suite"],   run in the checkout
#    "argv": ["target/sim/examples/milestone_suite","--ai-params","arms.json","--out-dir","out"],
#    "inputs": {"arms.json": "<inline JSON or text>"},  written into the batch directory before argv runs
#    "env": {"RAYON_NUM_THREADS": "64"},               default RAYON_NUM_THREADS is every core; the filler yields anyway
#    "outputs": ["out/milestones.json"],               relative to the batch directory; uploaded to results/<id>/
#    "timeout_s": 7200}
# argv runs in /data/filler/run/<id>; an argv[0] starting with "target/" is taken from the checkout's target/. The
# checkout and its target/ persist, so a commit is built once and every later batch at it starts at once.
# Queue by hand with `aws s3 cp spec.json s3://<sims bucket>/box/filler/pending/<id>.json` plus `filler.sh ensure`.
# Results: s3://<sims bucket>/box/filler/results/<id>/ ending with DONE (rc= start= end= wall_s= cores=).
set -euo pipefail
. "$(dirname "$0")/box-lib.sh"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
bucket="$(box_bucket)"
base="s3://$bucket/box/filler"

ensure() { box_up; box_sync; box_ssh '/data/bin/box-filler ensure'; }

cmd="${1:-status}"; [ $# -eq 0 ] || shift
case "$cmd" in
  submit)
    spec="${1:?usage: filler.sh submit SPEC.json [--commit REF]}"; shift
    ref=""; [ "${1:-}" != --commit ] || ref="${2:?--commit needs a ref}"
    id="$(jq -r '.id // empty' "$spec")"; [ -n "$id" ] || die "the spec needs an id"
    tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
    cp "$spec" "$tmp/spec.json"
    if [ -n "$ref" ]; then
      sha="$(git -C "$root" rev-parse --verify "$ref^{commit}")" || die "cannot resolve $ref"
      box_up; box_sync
      neg=()
      while read -r s; do [ -z "$s" ] || { git -C "$root" cat-file -e "$s^{commit}" 2>/dev/null && neg+=("^$s"); }; done < <(box_ssh '/data/bin/box-job known')
      git -C "$root" update-ref "refs/filler/$sha" "$sha"
      if git -C "$root" bundle create "$tmp/in.bundle" "refs/filler/$sha" ${neg[@]+"${neg[@]}"} >/dev/null 2>"$tmp/b.err"; then
        awsx s3 cp --no-progress "$tmp/in.bundle" "$base/bundles/$sha.bundle" >/dev/null
        jq --arg c "$sha" --arg b "bundles/$sha.bundle" '.commit = $c | .bundle_key = $b' "$tmp/spec.json" > "$tmp/spec2.json"
      else
        grep -q "empty bundle" "$tmp/b.err" || { cat "$tmp/b.err" >&2; die "git bundle failed"; }
        jq --arg c "$sha" '.commit = $c' "$tmp/spec.json" > "$tmp/spec2.json"   # the box already has the objects
      fi
      mv "$tmp/spec2.json" "$tmp/spec.json"
    fi
    # The box role cannot delete: a resubmitted id needs its old DONE, claim and cancel marker gone first.
    awsx s3 rm --recursive --only-show-errors "$base/results/$id/" >/dev/null 2>&1 || true
    awsx s3 rm --only-show-errors "$base/running/$id" >/dev/null 2>&1 || true
    awsx s3 rm --only-show-errors "$base/cancel/$id" >/dev/null 2>&1 || true
    awsx s3 cp --no-progress "$tmp/spec.json" "$base/pending/$id.json" >/dev/null
    ensure
    echo "queued $id (results: $base/results/$id/)"
    ;;
  ensure) ensure ;;
  status)
    box_up; box_sync
    box_ssh '/data/bin/box-filler status'
    ;;
  fetch)
    id="${1:?usage: filler.sh fetch ID [DIR]}"; dest="${2:-.}"
    mkdir -p "$dest"
    awsx s3 sync --no-progress --only-show-errors "$base/results/$id/" "$dest/"
    if [ -f "$dest/DONE" ]; then
      cat "$dest/DONE"
      for k in pending/$id.json running/$id cancel/$id; do awsx s3 rm --only-show-errors "$base/$k" >/dev/null 2>&1 || true; done
    else echo "$id has no DONE yet"; fi
    ;;
  cancel)
    id="${1:?usage: filler.sh cancel ID}"
    t="$(mktemp)"; : > "$t"
    awsx s3 cp --no-progress "$t" "$base/cancel/$id" >/dev/null   # stops it if it is running
    if ! awsx s3 ls "$base/running/$id" 2>/dev/null | grep -q .; then   # not started: mark it finished so it never starts
      printf 'rc=cancelled start=0 end=0 wall_s=0 cores=0\n' > "$t"; awsx s3 cp --no-progress "$t" "$base/results/$id/DONE" >/dev/null
    fi
    rm -f "$t"
    echo "cancel requested for $id"
    ;;
  *) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 2;;
esac

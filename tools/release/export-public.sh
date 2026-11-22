#!/usr/bin/env bash
# Build the curated public tree from a commit of this (private, full-history) repository.
#
#   tools/release/export-public.sh [--ref REF] [--out DIR] [--push] [--no-scan]
#
#   --ref REF   commit to export (default: HEAD of the checkout this script lives in).
#   --out DIR   where the tree goes (default: $TMPDIR/legatum-public). Emptied first.
#   --push      commit the tree on branch `staging` of $PUBLIC_REPO (default github.com/samcorcos/legatum), one
#               commit on top of that repository's `master`, and force-push `staging`. Never touches `master`, never
#               changes the repository's visibility.
#   --no-scan   skip gitleaks and the personal-data scan, for a local dry run only. It cannot be combined with --push:
#               the scans are on by default and failing them blocks --push.
#
# Everything tracked is exported except EXCLUDE, the unreferenced art masters (tools/release/runtime-assets.py) and
# large design images. A left-out file that exported code or tests still name comes back (referenced-excluded.py). Decisions, kept in one place (designer, 2026-10-23: publish the development layer too):
#   - The development layer is public: `.claude/` (skills, agents, hooks), `CLAUDE.md`, `tools/` (astra, ops, cloud,
#     film, evolve, ...), `infra/`, `plans/`, `docs/status`, `docs/ops`, `docs/learnings.md`. What is scrubbed or left
#     out: credentials, the AWS account ID (the bucket is resolved from the account at run time), IPs, home paths,
#     tfvars and state, personal details, and the private design document's link. The personal-data scan
#     (scrub.py) and gitleaks enforce that; findings are fixed at the source on master.
#   - Left out as run output and agent scratch: `.reviews/`, `.progress/`, and `docs/sim-results/` (800 MB) except the
#     methodology READMEs and the dated reports the exported docs link to.
#   - Files from tools/release/public/ are copied over the tree last (`.gitignore`, `AGENTS.md`, `docs/README.md`, and a
#     README stub that yields to a real README.md on master).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
ref=HEAD
out="${TMPDIR:-/tmp}/legatum-public"
push=0
scan=1
while [ $# -gt 0 ]; do
  case "$1" in
    --ref) ref="${2:?--ref needs a commit}"; shift 2;;
    --out) out="${2:?--out needs a directory}"; shift 2;;
    --push) push=1; shift;;
    --no-scan) scan=0; shift;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 0;;
    *) echo "unknown argument: $1" >&2; exit 2;;
  esac
done
if [ "$push" = 1 ] && [ "$scan" = 0 ]; then
  echo "export: --push cannot be combined with --no-scan; publishing always runs the scans" >&2; exit 2
fi
PUBLIC_REPO="${PUBLIC_REPO:-https://github.com/samcorcos/legatum.git}"
commit="$(git -C "$repo" rev-parse --verify "$ref^{commit}")"
# What must not appear in the public tree: the generic patterns in scrub-patterns.txt plus, when it exists, the owner's
# untracked .private/scrub-patterns.txt (scrub.py finds it; .private.example/ explains the format). No value to hide
# is ever written in this script. Fix findings at the source on master, not here.
pat="$(python3 -I "$here/scrub.py" pattern)"

INCLUDE=(.)
EXCLUDE=(
  .reviews .progress docs/sim-results .claude/settings.local.json .claude/worktrees
  app/e2e-qa app/test-results app/dist app/node_modules
  scripts/summarize-tech-bicycle.py scripts/summarize-tech-bicycle-2.py scripts/summarize-tech-bicycle-2-tuning.py
  scripts/summarize-tech-full.py scripts/test_summarize_tech_bicycle.py scripts/sparta-balance-report.py
  scripts/ruin-reward-report.py scripts/summarize-capital-science.py scripts/summarize-difficulty.py
  scripts/coalition-report.py scripts/threat_report.py scripts/perf-full-tree.mjs
  docs/design/art-audit docs/design/art-distinctness docs/design/japan-hive docs/design/siege-engineer
  docs/media/PLAN.md docs/open-source docs/status/oss-readiness-2026-11-02.md
  docs/design/diplomacy-design.md docs/design/tech-tree-design.md docs/design/factions-draft-astra.md
  docs/design/factions-original-three-astra.md docs/design/art-audit-2026-10-10.md
)
SIM_KEEP=(
  docs/sim-results/ai-quality/README.md docs/sim-results/hard-vs-human/README.md docs/sim-results/perf/README.md
)
# Only the paths that exist in the commit; git archive fails on a missing pathspec.
spec=()
for p in "${INCLUDE[@]}"; do spec+=("$p"); done
for p in "${EXCLUDE[@]}"; do spec+=(":(exclude)$p"); done

rm -rf "$out"; mkdir -p "$out"
echo "export: $commit -> $out"
git -C "$repo" archive "$commit" -- "${spec[@]}" | tar -x -C "$out"
keep=()
for p in "${SIM_KEEP[@]}"; do [ -n "$(git -C "$repo" ls-tree --name-only "$commit" -- "$p")" ] && keep+=("$p"); done
[ ${#keep[@]} -gt 0 ] && git -C "$repo" archive "$commit" -- "${keep[@]}" | tar -x -C "$out"

# Art masters the app never loads (source PNGs, review sheets, retired variants) stay in the archive.
keep="$(mktemp)"; python3 -I "$here/runtime-assets.py" "$out" > "$keep"
dropped=0
while IFS= read -r f; do
  if ! grep -qxF "$f" "$keep"; then rm -f "$out/$f"; dropped=$((dropped + 1)); fi
done < <(cd "$out" && find app/src/assets -type f \( -name '*.png' -o -name '*.jpg' -o -name '*.jpeg' -o -name '*.webp' \))
rm -f "$keep"
echo "export: dropped $dropped unreferenced raster assets from app/src/assets"

# Design review images: keep the small ones, the markdown that cites them stays.
if [ -d "$out/docs/design" ]; then
  find "$out/docs/design" -type f \( -name '*.png' -o -name '*.jpg' -o -name '*.jpeg' -o -name '*.webp' -o -name '*.gif' \) -size +120k -delete
  find "$out/docs/design" -type f -name '*.pdf' -delete
fi
find "$out" -type f \( -name '.DS_Store' -o -name '*.tsbuildinfo' \) -delete

# Overlay: public-only files (README stub, AGENTS.md, ...). Files in the commit's own tree win for README.md.
if git -C "$repo" cat-file -e "$commit:tools/release/public" 2>/dev/null; then
  ov="$(mktemp -d)"
  git -C "$repo" archive "$commit" tools/release/public | tar -x -C "$ov"
  (cd "$ov/tools/release/public" && find . -type f | while IFS= read -r f; do
    if [ "$f" = ./README.md ] && [ -f "$out/README.md" ]; then continue; fi
    mkdir -p "$out/$(dirname "$f")"; cp "$f" "$out/$f"
  done)
  rm -rf "$ov"
fi

# Reports and notes that the exported docs link to (dated sim-results reports and the like) come along, one level deep.
for _ in 1; do
  extra=()
  while IFS= read -r f; do
    case "$f" in
      docs/sim-results/*.md|docs/sim-results/*/*.md|docs/analysis/*.md|docs/archive/*.md)
        if [ -n "$(git -C "$repo" ls-tree --name-only "$commit" -- "$f")" ] && [ ! -e "$out/$f" ]; then extra+=("$f"); fi;;
    esac
  done < <(python3 -I "$here/check-links.py" "$out" --missing-paths)
  [ ${#extra[@]} -gt 0 ] || break
  git -C "$repo" archive "$commit" -- "${extra[@]}" | tar -x -C "$out"
  echo "export: pulled in ${#extra[@]} linked documents"
done

# Left-out files that exported code still names (include_str! fixtures, default params, files tests open) come back,
# unless they are over 10 MB or hold personal data: those print a warning and stay out.
extra=()
while IFS= read -r f; do [ -n "$f" ] && extra+=("$f"); done < <(git -C "$repo" ls-tree -r --name-only "$commit" \
  | python3 -I "$here/referenced-excluded.py" "$out" --commit "$commit" --repo "$repo" --scan-pattern "$pat")
if [ ${#extra[@]} -gt 0 ]; then
  git -C "$repo" archive "$commit" -- "${extra[@]}" | tar -x -C "$out"
  echo "export: pulled in ${#extra[@]} files the exported code references"
fi

# Text transform: the private design document's link (a claude.ai artifact) is not published.
find "$out" -type f \( -name '*.md' -o -name '*.json' -o -name '*.sh' -o -name '*.py' \) -not -path '*/node_modules/*' -print0 \
  | xargs -0 perl -pi -e 's#https://claude\.ai/[^\s)>\x27"`]+#(a private document, not published)#g'

# Large-file guard: nothing over 10 MB ships by accident.
big="$(find "$out" -type f -size +10M | sed "s|^$out/||")"
if [ -n "$big" ]; then echo "export: FAIL: files over 10 MB:" >&2; echo "$big" >&2; exit 1; fi

fail=0
if [ "$scan" = 1 ]; then
  echo "export: scanning for personal data"
  if python3 -I "$here/scrub.py" scan "$out" > "$out/../legatum-scan.txt"; then
    echo "export: personal-data scan clean"
  else
    n="$(wc -l < "$out/../legatum-scan.txt")"
    echo "export: personal-data scan: $n hits (first 25; full list in $(cd "$out/.." && pwd)/legatum-scan.txt)" >&2
    head -25 "$out/../legatum-scan.txt" | cut -c1-200 >&2
    fail=1
  fi
  if command -v gitleaks >/dev/null; then
    gitleaks dir "$out" --no-banner --redact --exit-code 1 >"$out/../legatum-gitleaks.txt" 2>&1 \
      && echo "export: gitleaks clean" || { echo "export: gitleaks FOUND SECRETS:" >&2; cat "$out/../legatum-gitleaks.txt" >&2; fail=1; }
  else
    echo "export: gitleaks is not installed (brew install gitleaks); secret scan SKIPPED" >&2; fail=1
  fi
fi

files="$(find "$out" -type f | wc -l | tr -d ' ')"
size="$(du -sk "$out" | awk '{printf "%.1f MB", $1/1024}')"
echo "export: $files files, $size"
echo "export: largest files:"
python3 - "$out" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])
largest = sorted(
    ((path.stat().st_size, path.relative_to(root)) for path in root.rglob("*") if path.is_file()),
    key=lambda item: (-item[0], str(item[1])),
)[:8]
for size, path in largest:
    print(f"  {size / 1e6:6.2f} MB  {path}")
PY

if [ "$push" = 1 ]; then
  if [ "$fail" != 0 ]; then echo "export: scans failed; refusing to push" >&2; exit 1; fi
  work="$(mktemp -d)"
  git clone --quiet --branch master "$PUBLIC_REPO" "$work/repo"
  cd "$work/repo"
  git checkout --quiet -B staging origin/master
  git rm -rq --ignore-unmatch . >/dev/null 2>&1 || true
  (cd "$out" && tar -c .) | tar -x -C .
  git add -A
  short="$(git -C "$repo" rev-parse --short "$commit")"
  GIT_AUTHOR_NAME="Sam Corcos" GIT_AUTHOR_EMAIL="2626231+samcorcos@users.noreply.github.com" \
  GIT_COMMITTER_NAME="Sam Corcos" GIT_COMMITTER_EMAIL="2626231+samcorcos@users.noreply.github.com" \
    git commit --quiet -m "Public snapshot of the Legatum game ($short)

Curated export of the private archive at $short (tools/release/export-public.sh).

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
  git push --force-with-lease --quiet origin staging
  echo "export: pushed staging: $(git rev-parse HEAD)"
  rm -rf "$work"
fi
[ "$fail" = 0 ] || { echo "export: scans reported findings (exit 1)" >&2; exit 1; }

#!/usr/bin/env bash
# Cut a Scenario Lab scenario from any save, add it to the title screen's list and measure the Hard
# AI's result from the same start. See scenarios/README.md.
#
#   tools/scenarios/cut.sh SAVE --inspect [--from A] [--to B]
#   tools/scenarios/cut.sh --state STATE.json --seat K --id ID ...   (a state dumped by extract-state.sh, for a
#       save recorded on an older engine; --turn is not needed)
#   tools/scenarios/cut.sh --card SCENARIO.json --question TEXT --blurb TEXT [--ask TEXT] [--order K] [--runs 50]
#       (list an existing scenario file, such as one in scenarios/war/, and measure its Hard AI baseline)
#   tools/scenarios/cut.sh SAVE --turn N --seat K --id ID --title TEXT --question TEXT --blurb TEXT \
#       --briefing TEXT --objective SPEC --turns N [--minutes M] [--order K] [--ask TEXT] \
#       [--runs 50] [--replace] [--allow-mismatch] [--no-baseline] [--local]
#
# The save is only read (a copy goes to the box). It must have been recorded on the current engine
# (or pass --allow-mismatch). The work runs on the cloud box: building `legatum-sim`, replaying the
# save to turn N, writing scenarios/lab/ID.json, then `legatum-sim scenario ... --record` (the Hard AI
# plays your seat from the same start; its result is written into the file for the result card).
# With the box unavailable (exit 3) rerun with --local, which builds and runs on this Mac, niced.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
here="$(cd "$(dirname "$0")" && pwd)"

die() { echo "cut.sh: $*" >&2; exit 2; }
[ $# -ge 1 ] || { sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 2; }
case "$1" in -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 0;; esac
card=""; save=""; state=""
if [ "$1" = "--state" ]; then
  state="${2:?--state needs a state file}"; shift 2
  [ -f "$state" ] || die "no such state file: $state"
elif [ "$1" = "--card" ]; then
  card="${2:?--card needs a scenario file}"; shift 2
  [ -f "$card" ] || die "no such scenario file: $card"
  card="$(cd "$(dirname "$card")" && pwd)/$(basename "$card")"
  case "$card" in "$root"/*) card="${card#"$root"/}";; *) die "the scenario file must be inside this checkout";; esac
else
  save="$1"; shift
  [ -f "$save" ] || die "no such save: $save"
fi

cut_args=(); id=""; runs=50; baseline=1; local_run=0; inspect=0
while [ $# -gt 0 ]; do
  case "$1" in
    --inspect) inspect=1; cut_args+=("$1"); shift;;
    --id) id="${2:?--id needs a value}"; cut_args+=("$1" "$2"); shift 2;;
    --runs) runs="${2:?}"; shift 2;;
    --no-baseline) baseline=0; shift;;
    --local) local_run=1; shift;;
    --replace|--allow-mismatch) cut_args+=("$1"); shift;;
    --turn|--seat|--title|--question|--blurb|--briefing|--ask|--requires|--rusher|--war|--storm|--rush-target|--opponent|--objective|--turns|--minutes|--order|--from|--to)
      [ $# -ge 2 ] || die "$1 needs a value"; cut_args+=("$1" "$2"); shift 2;;
    *) die "unknown option $1";;
  esac
done
if [ -n "$card" ]; then
  id="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["id"])' "$root/$card")"
  cut_args=(--card "$card" ${cut_args[@]+"${cut_args[@]}"})
elif [ "$inspect" = 0 ]; then
  [ -n "$id" ] || die "--id is required"
  case "$id" in *[!a-z0-9-]*) die "--id may use lowercase letters, digits and dashes only";; esac
fi

rel=".cut-tmp/run-$$"
work="$root/$rel"
mkdir -p "$work"
trap 'rm -rf "$work"; rmdir "$root/.cut-tmp" 2>/dev/null || true' EXIT
[ -z "$save" ] || cp "$save" "$work/save.legatum"
[ -z "$state" ] || cp "$state" "$work/state.json"

{
  echo '#!/usr/bin/env bash'
  echo 'set -euo pipefail'
  printf 'cargo run -q --release -p legatum-sim -- scenario-cut'
  [ -z "$save" ] || printf " --save $rel/save.legatum"
  [ -z "$state" ] || printf " --state $rel/state.json"
  for a in ${cut_args[@]+"${cut_args[@]}"}; do printf ' %q' "$a"; done
  echo
  if [ "$inspect" = 0 ] && [ "$baseline" = 1 ]; then
    printf 'cargo run -q --release -p legatum-sim -- scenario %q --runs %q --difficulty hard --threads 16 --record\n' "${card:-scenarios/lab/$id.json}" "$runs"
  fi
} > "$work/run.sh"

if [ "$local_run" = 1 ]; then
  renice 15 -p $$ >/dev/null 2>&1 || true
  (cd "$root" && bash "$rel/run.sh")
  echo "cut.sh: done locally"
else
  artifacts=()
  if [ "$inspect" = 0 ]; then
    artifacts+=(--artifact "${card:-scenarios/lab/$id.json}")
    [ "$baseline" = 1 ] && artifacts+=(--artifact "docs/sim-results/scenarios/$id-baseline.json" --artifact "docs/sim-results/scenarios/$id-baseline.md")
  fi
  "$root/tools/cloud/remote.sh" "$root" --with-uncommitted --label "cut-${id:-inspect}" --artifact-dir "$work/out" ${artifacts[@]+"${artifacts[@]}"} -- bash "$rel/run.sh"
  if [ "$inspect" = 0 ]; then
    mkdir -p "$root/scenarios/lab" "$root/docs/sim-results/scenarios"
    cp "$work/out/${card:-scenarios/lab/$id.json}" "$root/${card:-scenarios/lab/$id.json}"
    if [ "$baseline" = 1 ]; then
      cp "$work/out/docs/sim-results/scenarios/$id-baseline.json" "$work/out/docs/sim-results/scenarios/$id-baseline.md" "$root/docs/sim-results/scenarios/"
    fi
  fi
fi

if [ "$inspect" = 0 ]; then
  if [ "$baseline" = 1 ]; then
    measured_commit="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("engine_commit", "unknown"))' "$root/docs/sim-results/scenarios/$id-baseline.json")"
    python3 "$root/tools/sim/freshness.py" stamp --commit "$measured_commit" \
      "docs/sim-results/scenarios/$id-baseline.json" "docs/sim-results/scenarios/$id-baseline.md"
  fi
  python3 - "$root/${card:-scenarios/lab/$id.json}" <<'PY'
import json, sys
f = json.load(open(sys.argv[1]))
b = f.get("baseline")
print(f"cut.sh: {f['id']} r{f['revision']} written; {f['faction']} seat {f['seat']}; objective: {f['objective']['summary']}")
if b:
    print(f"cut.sh: Hard AI over {b['runs']} runs: median score {b['median']}, met {b.get('win_percent')}%, median turns {b.get('median_turns')}")
else:
    print("cut.sh: no baseline recorded (--no-baseline or the run failed)")
req = (f.get("lab") or {}).get("requires")
if req:
    print(f"cut.sh: held back from the Scenarios screen until: {req}. Delete lab.requires in the JSON when it lands.")
else:
    print("cut.sh: it is listed in the Scenarios screen after the next app build.")
print("cut.sh: commit scenarios/lab/ and docs/sim-results/scenarios/.")
PY
fi

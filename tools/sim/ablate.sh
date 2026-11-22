#!/usr/bin/env bash
# Faction-rule ablation census: for one faction, a paired cloud census of "control" against each of the
# faction's rules switched off (legatum-sim --rule-off), same seeds, then a report.
#
#   tools/sim/ablate.sh <faction> [--games 4096] [--seed 9311003] [--rules a,b] [--parallel 3]
#                       [--type c8g.4xlarge]
#                       [--no-build] [--control-dir RUN] [--dry-run] [--date YYYY-MM-DD]
#
# <faction>     germany, switzerland, india, byzantium, celts, mongols, sparta, athens or aztecs. The rules
#               are the faction's rows in the `faction_rules!` table of crates/core/src/experiments.rs
#               (`python3 -I tools/sim/ablate_report.py rules <faction>` lists them).
# --rules       only these rules (snake_case names, comma-separated); default every rule of the faction.
# --parallel    runs in flight at once (each is one spot server; default 3). A launch
#               refused for spot capacity is retried every 60 s.
# --type        Graviton spot instance type forwarded to run-batch.sh (default c8g.16xlarge).
# --no-build    use the binary already in target/cloud/linux-arm64 (the default builds the current worktree on
#               the build box with tools/cloud/build-linux.sh; one control per engine commit is enough).
# --control-dir reuse a control run of the SAME binary and seeds (a run dir under ~/LegatumRuns/cloud).
# --dry-run     build nothing and call no AWS: print each run's plan (run-batch.sh --dry-run) and stop.
#
# Each run: tools/cloud/run-batch.sh --count 1 --max-hours 1 -- --players 8 --difficulty hard --games N
# --seed S --label ablate-<faction>-<arm> [--rule-off <rule>]  (one batch per arm; arms differ only by the flag).
# Only spot batches through run-batch.sh are launched; no other AWS command is run.
# Report: docs/sim-results/faction-ablation/<faction>-<date>/README.md (tools/sim/ablate_report.py).
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

faction=""; games=4096; seed=9311003; rules=""; parallel=3; type="c8g.16xlarge"; build=1; control_dir=""; dry=0; date="$(date +%F)"
while [ $# -gt 0 ]; do
  case "$1" in
    --games) games="${2:?}"; shift 2;;
    --seed) seed="${2:?}"; shift 2;;
    --rules) rules="${2:?}"; shift 2;;
    --parallel) parallel="${2:?}"; shift 2;;
    --type) type="${2:?}"; shift 2;;
    --no-build) build=0; shift;;
    --control-dir) control_dir="${2:?}"; shift 2;;
    --dry-run) dry=1; shift;;
    --date) date="${2:?}"; shift 2;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 0;;
    -*) echo "ablate: unknown option $1" >&2; exit 2;;
    *) [ -z "$faction" ] || { echo "ablate: unexpected argument $1" >&2; exit 2; }; faction="$1"; shift;;
  esac
done
[ -n "$faction" ] || { echo "usage: tools/sim/ablate.sh <faction> [options] (see --help)" >&2; exit 2; }
faction="$(printf '%s' "$faction" | tr 'A-Z' 'a-z')"

table="$(python3 -I tools/sim/ablate_report.py rules "$faction")"   # exits with the valid factions on a typo
names=""
for n in $(printf '%s\n' "$table" | awk '{print $1}'); do
  if [ -z "$rules" ]; then names="$names $n"; else
    case ",$rules," in *",$n,"*) names="$names $n";; esac
  fi
done
if [ -n "$rules" ]; then
  for want in $(printf '%s' "$rules" | tr ',' ' '); do
    printf '%s\n' "$table" | awk '{print $1}' | grep -qx "$want" \
      || { echo "ablate: $want is not a $faction rule. Valid:" >&2; printf '%s\n' "$table" | awk '{print "  " $1}' >&2; exit 2; }
  done
fi
[ -n "$names" ] || { echo "ablate: no rules selected" >&2; exit 2; }
echo "ablate: $faction: control + $(echo $names | wc -w | tr -d ' ') rules:$names"

commit="$(git rev-parse --short=9 HEAD)"; git diff --quiet HEAD -- 2>/dev/null || commit="$commit-dirty"
if [ "$build" = 1 ] && [ "$dry" = 0 ]; then
  tools/cloud/build-linux.sh
elif [ "$dry" = 0 ]; then
  [ -x target/cloud/linux-arm64/legatum-sim ] || { echo "ablate: no binary in target/cloud/linux-arm64 (drop --no-build)" >&2; exit 1; }
fi
[ "$dry" = 1 ] || commit="$(sed -n 's/^commit //p' target/cloud/linux-arm64/BUILD_INFO 2>/dev/null | head -1)"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# run_arm <arm> [sim args...]: one spot batch; writes the run dir to $tmp/<arm>.dir, the log to $tmp/<arm>.log
run_arm() {
  local arm="$1"; shift
  local label="ablate-$faction-$arm" attempt=0
  while :; do
    if tools/cloud/run-batch.sh --type "$type" --count 1 --max-hours 3 $([ "$dry" = 1 ] && echo --dry-run) -- \
        --players 8 --difficulty hard --games "$games" --seed "$seed" --label "$label" "$@" > "$tmp/$arm.log" 2>&1; then
      break
    fi
    if grep -qE "spot vCPUs|no capacity" "$tmp/$arm.log" && [ "$attempt" -lt 60 ]; then
      attempt=$((attempt + 1)); echo "ablate: $arm waiting for spot capacity (try $attempt)" >&2; sleep 60; continue
    fi
    echo "ablate: $arm FAILED; see $tmp/$arm.log" >&2; cat "$tmp/$arm.log" >&2; cp "$tmp/$arm.log" "/tmp/ablate-$faction-$arm.log" 2>/dev/null || true
    return 1
  done
  sed -n 's/^results: \([^ ]*\) .*/\1/p' "$tmp/$arm.log" > "$tmp/$arm.dir"
  echo "ablate: $arm done: $(cat "$tmp/$arm.dir")"
}

pids=""; fail=0
reap() {   # drop finished runs from $pids, recording failures
  local keep="" p
  for p in $pids; do
    if kill -0 "$p" 2>/dev/null; then keep="$keep $p"; else wait "$p" || fail=1; fi
  done
  pids="$keep"
}
launch() {   # launch <arm> [sim args...], at most $parallel at once
  reap
  while [ "$(echo $pids | wc -w | tr -d ' ')" -ge "$parallel" ]; do sleep 5; reap; done
  run_arm "$@" &
  pids="$pids $!"
}

if [ -z "$control_dir" ]; then launch control; fi
for n in $names; do launch "$n" --rule-off "$n"; done
for p in $pids; do wait "$p" || fail=1; done
[ "$fail" = 0 ] || { echo "ablate: a run failed" >&2; exit 1; }
[ "$dry" = 0 ] || { echo "ablate: dry run, no report"; for a in control $names; do echo "--- $a"; grep -E "remote command|batch " "$tmp/$a.log" 2>/dev/null | head -3; done; exit 0; }

[ -n "$control_dir" ] || control_dir="$(cat "$tmp/control.dir")"
out="docs/sim-results/faction-ablation/$faction-$date"
mkdir -p "$out"
args=()
for n in $names; do args+=(--arm "$n=$(cat "$tmp/$n.dir")"); done
python3 -I tools/sim/ablate_report.py report --faction "$faction" --control "$control_dir" "${args[@]}" \
  --out "$out/README.md" --commit "$commit" --seed "$seed" --games "$games" --date "$date"
python3 tools/sim/freshness.py stamp --commit "$commit" "$out/README.md"
echo "ablate: wrote $out/README.md"

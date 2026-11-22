#!/usr/bin/env bash
# Run a big simulation batch on spot Graviton servers, then fetch and merge the results.
#
#   tools/cloud/run-batch.sh [options] -- <legatum-sim args>
#
#   --type T         instance type (default c8g.16xlarge, 64 vCPUs; c7g.16xlarge is the older, cheaper fallback)
#   --count N        servers (default 1; at most $LEGATUM_CLOUD_MAX_SERVERS, default 4)
#   --max-hours H    hard limit per server (default 2, at most 12). Every server schedules its own
#                    `shutdown -h` for this long after boot, before anything else runs.
#   --align K        round each server's game count to a multiple of K (default 1)
#   --yes            go ahead even when the worst-case cost is above $LEGATUM_CLOUD_CONFIRM_USD (default 10)
#   --dry-run        no AWS calls: print the plan and cost, write the user-data under target/cloud/dry-run
#   --env K=V        set an environment variable for legatum-sim on the servers (repeatable;
#                    letters, digits and _ in K, no spaces in V)
#   --bin NAME       run this binary from target/cloud/linux-arm64 instead of legatum-sim (an
#                    example built with LEGATUM_CLOUD_EXAMPLES; it must ignore the --games,
#                    --seed, --label, --out-dir, --out and --status-file flags appended here, or
#                    use them). Its output files are fetched as they are, without merge.py.
#   --ai-params FILE upload this local AI params file and pass its server-side copy to the sim
#   --opponent-params FILE  the same for the sim's --opponent-params: the params of half the seats
#                    (a head-to-head league against --ai-params, or against the built-in AI without it)
#   --input-file NAME=FILE  upload a read-only input for a custom --bin at /tmp/input-NAME
#
# The sim args are legatum-sim's own, split across servers by game count and seed range: `--games N`
# is required (total), `--seed S` is the first seed, `--label L` names the run. Do not pass --out-dir,
# --out or --status-file (set here). Relative paths land in the server's output directory and are
# uploaded. Flags whose schedule depends on the game index (--strategy-matchups, --duel-matrix) cannot
# be split and are refused; for --mirror and the rotations pick --align carefully (README).
#
#   tools/cloud/run-batch.sh --count 2 --max-hours 3 -- --games 20000 --seed 7000001 \
#       --players 2 --label duel-check
#   tools/cloud/run-batch.sh --dry-run --bin seat_run --input-file tape=game.json \
#       --input-file states=states.jsonl -- --tape /tmp/input-tape --states /tmp/input-states \
#       --games 2 --label seat-check
#
# Register it with the watchdog by running it under bgjob.sh, named cloud-<anything>:
#   tools/ops/bgjob.sh cloud-duel-check --log .progress/cloud-duel-check.log -- tools/cloud/run-batch.sh ...
#
# Guardrails, each independent: self-destruct timer in user-data; instance-initiated shutdown
# terminates; Ctrl-C and any error here terminate the batch's servers; every server is tagged with
# batch, owner and expiry; the AWS-side reaper and the monthly budget (infra/cloud-sims).
set -euo pipefail
. "$(dirname "$0")/lib.sh"

# Keep timing best-effort: an unavailable ledger must not change cloud launch output or status.
timing_file="$(mktemp 2>/dev/null || true)"
timing_file_after="$(mktemp 2>/dev/null || true)"
timing_label="cloud;usd=unknown"
timing_ref="unknown"
timing_load="$(sysctl -n vm.loadavg 2>/dev/null | awk '{print $2}' || true)"; [ -n "$timing_load" ] || timing_load=0
if [ -n "$timing_file" ] && [ -n "$timing_file_after" ]; then times > "$timing_file"; fi
SECONDS=0
_timing_exit() {
  local exit_code="${1:-$?}" cpu
  trap - EXIT
  if [ -n "$timing_file" ] && [ -n "$timing_file_after" ]; then
    times > "$timing_file_after" 2>/dev/null || true
    cpu="$(awk 'FNR==2 { s=0; for (i=1; i<=2; i++) { split($i, a, "m"); sub("s", "", a[2]); s += a[1]*60 + a[2] } print s }' "$timing_file" "$timing_file_after" 2>/dev/null | awk 'NR==1{a=$1} NR==2{printf "%.1f", $1-a}')"
  else cpu=0; fi
  [ -n "$cpu" ] || cpu=0
  python3 "$(dirname "$0")/../ops/timed.py" --record cloud run-batch "$timing_label" "$timing_ref" "$exit_code" "$SECONDS" "$cpu" "$timing_load" >/dev/null 2>&1 || true
  rm -f "$timing_file" "$timing_file_after" 2>/dev/null || true
  exit "$exit_code"
}
trap _timing_exit EXIT

type="c8g.16xlarge"; count=1; max_hours=2; align=1; yes=0; dry=0; sim_env=""; bin="legatum-sim"; ai_params=""; opponent_params=""; input_files=(); input_names=()
while [ $# -gt 0 ]; do
  case "$1" in
    --type) type="${2:?--type needs a value}"; shift 2;;
    --count) count="${2:?--count needs a value}"; shift 2;;
    --max-hours) max_hours="${2:?--max-hours needs a value}"; shift 2;;
    --align) align="${2:?--align needs a value}"; shift 2;;
    --yes) yes=1; shift;;
    --bin) bin="${2:?--bin needs a value}"; shift 2
      case "$bin" in ''|*[!A-Za-z0-9_-]*) die "--bin takes a binary name";; esac;;
    --ai-params)
      [ -z "$ai_params" ] || die "--ai-params may be specified only once"
      ai_params="${2:?--ai-params needs a value}"
      [ -n "$ai_params" ] || die "--ai-params needs a non-empty file path"
      shift 2;;
    --opponent-params)
      [ -z "$opponent_params" ] || die "--opponent-params may be specified only once"
      opponent_params="${2:?--opponent-params needs a value}"
      [ -n "$opponent_params" ] || die "--opponent-params needs a non-empty file path"
      shift 2;;
    --input-file)
      spec="${2:?--input-file needs NAME=FILE}"
      name="${spec%%=*}"; file="${spec#*=}"
      case "$name" in ''|*[!A-Za-z0-9_-]*) die "--input-file name must be letters, digits, _ or -";; esac
      [ "$name" != "$spec" ] && [ -n "$file" ] || die "--input-file needs NAME=FILE"
      for used_name in "${input_names[@]+"${input_names[@]}"}"; do [ "$used_name" != "$name" ] || die "duplicate --input-file name $name"; done
      input_names+=("$name"); input_files+=("$file"); shift 2;;
    --dry-run) dry=1; shift;;
    --env)
      case "${2:-}" in [A-Za-z_]*=*) ;; *) die "--env needs K=V";; esac
      case "$2" in *[[:space:]\'\"\$\`]*) die "--env value must not contain spaces or quotes";; esac
      sim_env="$sim_env $2"; shift 2;;
    --) shift; break;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 0;;
    *) die "unknown option $1 (sim args go after --)";;
  esac
done

# ---- validate ------------------------------------------------------------------------------
case "$count" in ''|*[!0-9]*|0) die "--count must be a positive integer";; esac
case "$align" in ''|*[!0-9]*|0) die "--align must be a positive integer";; esac
max_servers="${LEGATUM_CLOUD_MAX_SERVERS:-4}"
[ "$count" -le "$max_servers" ] || die "--count $count is above LEGATUM_CLOUD_MAX_SERVERS ($max_servers)"
case "$max_hours" in ''|*[!0-9.]*) die "--max-hours must be a number";; esac
awk -v h="$max_hours" 'BEGIN{exit !(h>=0.25 && h<=12)}' || die "--max-hours must be between 0.25 and 12"
vcpus="$(vcpus_of "$type")"
case "$type" in c7g.*|c8g.*|c6g.*|m7g.*|m8g.*) ;; *) die "type $type is not an allowed Graviton type (the launcher's IAM policy refuses it too)";; esac

games=""; seed=1; label="cloud"; rest=()
while [ $# -gt 0 ]; do
  case "$1" in
    --games) games="${2:?--games needs a value}"; shift 2;;
    --games=*) games="${1#*=}"; shift;;
    --seed) seed="${2:?--seed needs a value}"; shift 2;;
    --seed=*) seed="${1#*=}"; shift;;
    --label) label="${2:?--label needs a value}"; shift 2;;
    --label=*) label="${1#*=}"; shift;;
    --out-dir|--out-dir=*|--out|--out=*|--status-file|--status-file=*) die "do not pass $1: the server's output paths are set by run-batch.sh";;
    --ai-params|--ai-params=*) die "pass the local params file before -- using --ai-params FILE";;
    --opponent-params|--opponent-params=*) die "pass the local params file before -- using --opponent-params FILE";;
    --strategy-matchups|--duel-matrix)
      # One server plays games 0..N-1 from the first seed, exactly the local schedule.
      [ "$count" -eq 1 ] || die "$1 schedules by game index and cannot be split across servers; run it on one server (--count 1) or locally"
      rest+=("$1"); shift;;
    --replay|--replay-export|--tape-in) die "$1 reads local files and does not belong in a cloud batch";;
    --mirror|--difficulty-rotation*|--search-rotation*) echo "cloud: note: $1 schedules by game index; slices are only valid at --align multiples of its period (README)" >&2; rest+=("$1"); shift;;
    *) rest+=("$1"); shift;;
  esac
done
case "$games" in ''|*[!0-9]*|0) die "the sim args must include --games N";; esac
case "$seed" in ''|*[!0-9]*) die "--seed must be a non-negative integer";; esac
case "$label" in ''|*[!A-Za-z0-9._-]*) die "--label may contain only letters, digits, . _ -";; esac
[ "$games" -ge $(( count * align )) ] || die "--games $games is too few for $count servers at --align $align"

# ---- slice the games ---------------------------------------------------------------------------
units=$(( games / align )); extra=$(( games % align ))
slice_games=(); slice_seeds=(); next_seed="$seed"
for ((i = 0; i < count; i++)); do
  u=$(( units / count )); [ "$i" -lt $(( units % count )) ] && u=$(( u + 1 ))
  g=$(( u * align )); [ "$i" -eq $(( count - 1 )) ] && g=$(( g + extra ))
  slice_games+=("$g"); slice_seeds+=("$next_seed"); next_seed=$(( next_seed + g ))
done

# ---- cost estimate -------------------------------------------------------------------------------
case "$type" in   # approximate on-demand $/vCPU-hour: the ceiling a spot price can reach
  c7g.*) od_vcpu=0.03625;; c8g.*) od_vcpu=0.03987;; c6g.*) od_vcpu=0.0340;; m7g.*) od_vcpu=0.0408;; m8g.*) od_vcpu=0.0449;;
esac
od_hour="$(awk -v v="$vcpus" -v p="$od_vcpu" 'BEGIN{printf "%.2f", v*p}')"
batch="${label}-$(date +%m%d-%H%M%S)"
owner="${USER:-unknown}@$(hostname -s)"
mins="$(awk -v h="$max_hours" 'BEGIN{printf "%d", h*60 + 0.5}')"

if [ "$dry" = 0 ]; then
  require_profile
  bucket="$(bucket_name)"
  spot_rows="$(awsx ec2 describe-spot-price-history --instance-types "$type" --product-descriptions "Linux/UNIX" \
    --start-time "$(date -u +%FT%TZ)" --query 'SpotPriceHistory[].[AvailabilityZone,SpotPrice]' --output text | sort -k2 -n)"
  [ -n "$spot_rows" ] || die "no spot price for $type in $REGION (type not offered here?)"
  spot_hour="$(echo "$spot_rows" | head -1 | awk '{printf "%.4f", $2}')"
else
  bucket="legatum-sims-DRYRUN"
  spot_hour="${LEGATUM_SPOT_PRICE:-$(awk -v h="$od_hour" 'BEGIN{printf "%.4f", h*0.35}')}"   # dry run: ~65% off on-demand
  spot_rows="dry-run-az $spot_hour"
fi
# +0.005 $/h: the public IPv4 address each server needs to reach S3 without a NAT gateway.
hour_each="$(awk -v s="$spot_hour" 'BEGIN{printf "%.3f", s+0.005}')"
est_worst="$(awk -v e="$hour_each" -v n="$count" -v h="$max_hours" 'BEGIN{printf "%.2f", e*n*h}')"
est_ceiling="$(awk -v o="$od_hour" -v n="$count" -v h="$max_hours" 'BEGIN{printf "%.2f", (o+0.005)*n*h}')"

echo "batch $batch: $games games ($label) from seed $seed on $count x $type ($vcpus vCPUs each), max $max_hours h each"
for ((i = 0; i < count; i++)); do echo "  server $i: ${slice_games[$i]} games, seeds ${slice_seeds[$i]}..$(( ${slice_seeds[$i]} + ${slice_games[$i]} - 1 ))"; done
echo "  spot now \$$spot_hour/h per server (+\$0.005 IPv4): about \$$hour_each/h each, \$$(awk -v e="$hour_each" -v n="$count" 'BEGIN{printf "%.2f", e*n}')/h in total"
echo "  cost if every server runs the full $max_hours h: \$$est_worst (a batch that finishes early costs less; ceiling if spot jumped to on-demand: \$$est_ceiling)"
echo "  monthly cap: monthly_budget_usd in infra/cloud-sims (budget alerts and launch ban)"

confirm_usd="${LEGATUM_CLOUD_CONFIRM_USD:-10}"
if awk -v w="$est_worst" -v c="$confirm_usd" 'BEGIN{exit !(w>c)}' && [ "$yes" = 0 ] && [ "$dry" = 0 ]; then
  die "worst case \$$est_worst is above \$$confirm_usd: pass --yes to go ahead"
fi

# ---- spot headroom for the build box ---------------------------------------------------------------------
# The persistent build box (infra/cloud-box) is a spot instance in the same vCPU quota. Keep room for it: refuse a
# launch that would leave fewer than LEGATUM_CLOUD_BOX_RESERVE_VCPUS (default 64) spot vCPUs free while the box is
# stopped. LEGATUM_CLOUD_SPOT_QUOTA is the account's spot quota (448 on 2026-10-10; the launcher cannot read it).
if [ "$dry" = 0 ]; then
  reserve="${LEGATUM_CLOUD_BOX_RESERVE_VCPUS:-64}"
  quota="${LEGATUM_CLOUD_SPOT_QUOTA:-448}"
  if [ "$reserve" -gt 0 ]; then
    used=0
    for t in $(awsx ec2 describe-instances --filters "Name=instance-lifecycle,Values=spot" "Name=instance-state-name,Values=pending,running" \
        --query 'Reservations[].Instances[].InstanceType' --output text); do used=$(( used + $(vcpus_of "$t") )); done
    box_running="$(awsx ec2 describe-instances --filters "Name=tag:project,Values=legatum-box" "Name=instance-state-name,Values=pending,running" \
        --query 'length(Reservations[].Instances[])' --output text 2>/dev/null || echo 0)"
    [ "${box_running:-0}" != 0 ] && held=0 || held="$reserve"
    need=$(( count * vcpus ))
    if [ $(( used + held + need )) -gt "$quota" ]; then
      die "spot vCPUs: $used in use + $need requested + $held reserved for the build box > quota $quota. Retry when servers finish, use fewer servers, or set LEGATUM_CLOUD_BOX_RESERVE_VCPUS=0 to ignore the box"
    fi
  fi
fi

# ---- binaries and user-data --------------------------------------------------------------------
bin_dir="$CLOUD_ROOT/target/cloud/linux-arm64"
[ -x "$bin_dir/$bin" ] || die "no Linux binary $bin: run tools/cloud/build-linux.sh first"
bin_sum="$(shasum -a 256 "$bin_dir/$bin" | cut -c1-16)"
bin_key="bin/$bin_sum/$bin.gz"
if [ -n "$ai_params" ]; then
  [ -f "$ai_params" ] && [ -r "$ai_params" ] || die "AI params file is not a readable regular file: $ai_params"
  params_sum="$(shasum -a 256 "$ai_params" | cut -c1-16)"
  params_key="ai-params/$params_sum/params"
  params_remote="/tmp/ai-params-$params_sum"
fi
if [ -n "$opponent_params" ]; then
  [ -f "$opponent_params" ] && [ -r "$opponent_params" ] || die "opponent params file is not a readable regular file: $opponent_params"
  opp_sum="$(shasum -a 256 "$opponent_params" | cut -c1-16)"
  opp_key="ai-params/$opp_sum/params"
  opp_remote="/tmp/opp-params-$opp_sum"
fi
input_keys=(); input_downloads=""
for ((j = 0; j < ${#input_files[@]}; j++)); do
  file="${input_files[$j]}"; name="${input_names[$j]}"
  [ -f "$file" ] && [ -r "$file" ] || die "input file is not readable: $file"
  sum="$(shasum -a 256 "$file" | cut -c1-16)"
  key="inputs/$sum/$name"; input_keys+=("$key")
  input_downloads="${input_downloads}aws s3 cp --no-progress \"s3://\$BUCKET/$key\" /tmp/input-$name
"
done
commit="$(sed -n 's/^commit //p' "$bin_dir/BUILD_INFO" 2>/dev/null | head -1)"
timing_label="$label;usd=$est_worst"
timing_ref="${commit:-unknown}"

# The sim command line shared by every slice; --games, --seed and --label are added per slice.
sim_args=""
for a in ${rest[@]+"${rest[@]}"}; do sim_args="$sim_args $(printf '%q' "$a")"; done
[ -z "$ai_params" ] || sim_args="$sim_args --ai-params $params_remote"
[ -z "$opponent_params" ] || sim_args="$sim_args --opponent-params $opp_remote"

user_data_dir="$CLOUD_ROOT/target/cloud/dry-run"
[ "$dry" = 1 ] && mkdir -p "$user_data_dir" || user_data_dir="$(mktemp -d)"

write_user_data() {   # write_user_data <slice index> <file>
  local i="$1" f="$2"
  cat > "$f" <<UDATA
#!/bin/bash
# legatum-sims batch $batch, server $i. The self-destruct comes first, before anything can fail.
shutdown -h +$mins "legatum-sims self-destruct after $max_hours h"
exec > /var/log/legatum-batch.log 2>&1
set -x
export AWS_DEFAULT_REGION=$REGION
BUCKET=$bucket
PREFIX=batches/$batch/slice-$i
START=\$(date +%s)
RC=99
finish() {
  # Always: upload whatever exists, then a DONE marker, then power off (which terminates the server).
  END=\$(date +%s)
  gzip -f /work/out/*.json 2>/dev/null
  aws s3 sync --no-progress /work/out "s3://\$BUCKET/\$PREFIX/out"
  aws s3 cp --no-progress /var/log/legatum-batch.log "s3://\$BUCKET/\$PREFIX/batch.log"
  printf 'rc=%s\nstart=%s\nend=%s\n' "\$RC" "\$START" "\$END" > /tmp/DONE
  aws s3 cp --no-progress /tmp/DONE "s3://\$BUCKET/\$PREFIX/DONE"
  shutdown -h now
}
trap finish EXIT
mkdir -p /work/out/replays
cd /work/out || exit 1
aws s3 cp --no-progress "s3://\$BUCKET/$bin_key" /tmp/$bin.gz
gunzip -f /tmp/$bin.gz
chmod +x /tmp/$bin
${ai_params:+aws s3 cp --no-progress "s3://\$BUCKET/$params_key" "$params_remote"}
${opponent_params:+aws s3 cp --no-progress "s3://\$BUCKET/$opp_key" "$opp_remote"}
$input_downloads
set +e
${sim_env:+env$sim_env }/tmp/$bin$sim_args --games ${slice_games[$i]} --seed ${slice_seeds[$i]} --label $label-s$i \\
  --out-dir /work/out --out /work/out/replays --status-file /work/out/status.json
RC=\$?
exit 0
UDATA
}

for ((i = 0; i < count; i++)); do write_user_data "$i" "$user_data_dir/user-data-$i.sh"; done

if [ "$dry" = 1 ]; then
  echo "S3 binary key: s3://$bucket/$bin_key"
  [ -z "$ai_params" ] || echo "S3 AI params key: s3://$bucket/$params_key"
  [ -z "$opponent_params" ] || echo "S3 opponent params key: s3://$bucket/$opp_key"
  for ((i = 0; i < count; i++)); do
    echo "remote command $i: ${sim_env:+env$sim_env }/tmp/$bin$sim_args --games ${slice_games[$i]} --seed ${slice_seeds[$i]} --label $label-s$i --out-dir /work/out --out /work/out/replays --status-file /work/out/status.json"
  done
  echo "dry run: no AWS calls made. User-data for each server is in $user_data_dir"
  exit 0
fi

# ---- launch (from here on, failures and Ctrl-C terminate the batch) -----------------------------------
launched=()
finished=0
cleanup() {
  local rc=$?
  trap - INT TERM EXIT
  if [ "$finished" = 0 ]; then
    echo "cloud: stopping batch $batch (exit $rc)" >&2
    ids="$(awsx ec2 describe-instances --filters "Name=tag:batch,Values=$batch" "Name=instance-state-name,Values=pending,running,stopping,stopped" \
      --query 'Reservations[].Instances[].InstanceId' --output text 2>/dev/null || true)"
    ids="$ids ${launched[*]-}"
    if [ -n "${ids// /}" ]; then
      # shellcheck disable=SC2086
      awsx ec2 terminate-instances --instance-ids $ids >/dev/null 2>&1 || echo "cloud: TERMINATE FAILED, run tools/cloud/kill-all.sh $batch" >&2
    fi
  fi
  _timing_exit "$rc"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "uploading binary ($bin_key)"
if ! awsx s3api head-object --bucket "$bucket" --key "$bin_key" >/dev/null 2>&1; then
  gzip -c "$bin_dir/$bin" > "$user_data_dir/$bin.gz"
  awsx s3 cp --no-progress "$user_data_dir/$bin.gz" "s3://$bucket/$bin_key"
fi
if [ -n "$ai_params" ]; then
  echo "uploading AI params ($params_key)"
  awsx s3 cp --no-progress "$ai_params" "s3://$bucket/$params_key"
fi
if [ -n "$opponent_params" ]; then
  echo "uploading opponent params ($opp_key)"
  awsx s3 cp --no-progress "$opponent_params" "s3://$bucket/$opp_key"
fi
for ((j = 0; j < ${#input_files[@]}; j++)); do
  echo "uploading input ${input_names[$j]} (${input_keys[$j]})"
  awsx s3 cp --no-progress "${input_files[$j]}" "s3://$bucket/${input_keys[$j]}"
done

ami="$(awsx ssm get-parameter --name /aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64 --query Parameter.Value --output text)"
sg="$(awsx ec2 describe-security-groups --filters "Name=group-name,Values=$SECURITY_GROUP_NAME" --query 'SecurityGroups[0].GroupId' --output text)"
[ -n "$sg" ] && [ "$sg" != None ] || die "security group $SECURITY_GROUP_NAME not found: has infra/cloud-sims been applied in $REGION?"
subnets="$(awsx ec2 describe-subnets --filters Name=default-for-az,Values=true --query 'Subnets[].[AvailabilityZone,SubnetId]' --output text)"

launch_epoch="$(date -u +%s)"
expiry="$(date -u -r $(( launch_epoch + mins * 60 + 300 )) +%FT%TZ)"   # + 5 min boot margin; the reaper adds its own grace
{
  echo "batch $batch"; echo "commit ${commit:-unknown}"; echo "type $type x $count"; echo "max_hours $max_hours"
  echo "owner $owner"; echo "launched $(date -u +%FT%TZ)"; echo "expiry $expiry"; echo "sim_args $sim_args --games $games --seed $seed --label $label"
} > "$user_data_dir/BATCH_INFO"
awsx s3 cp --no-progress "$user_data_dir/BATCH_INFO" "s3://$bucket/batches/$batch/BATCH_INFO" >/dev/null

# Servers go to the cheapest AZ first; a capacity error moves on to the next one.
azs="$(echo "$spot_rows" | awk '{print $1}')"
ids=()
for ((i = 0; i < count; i++)); do
  id=""
  for az in $azs; do
    subnet="$(echo "$subnets" | awk -v az="$az" '$1 == az {print $2}')"
    [ -n "$subnet" ] || continue
    if id="$(awsx ec2 run-instances --image-id "$ami" --instance-type "$type" --count 1 \
        --subnet-id "$subnet" --security-group-ids "$sg" --iam-instance-profile "Name=$INSTANCE_PROFILE" \
        --instance-market-options 'MarketType=spot,SpotOptions={SpotInstanceType=one-time,InstanceInterruptionBehavior=terminate}' \
        --instance-initiated-shutdown-behavior terminate \
        --metadata-options 'HttpTokens=required,HttpEndpoint=enabled' \
        --block-device-mappings 'DeviceName=/dev/xvda,Ebs={VolumeSize=30,VolumeType=gp3,DeleteOnTermination=true}' \
        --tag-specifications \
          "ResourceType=instance,Tags=[{Key=project,Value=$TAG_PROJECT},{Key=batch,Value=$batch},{Key=owner,Value=$owner},{Key=expiry,Value=$expiry},{Key=slice,Value=$i},{Key=Name,Value=legatum-sims-$batch-$i}]" \
          "ResourceType=volume,Tags=[{Key=project,Value=$TAG_PROJECT},{Key=batch,Value=$batch}]" \
        --user-data "file://$user_data_dir/user-data-$i.sh" \
        --query 'Instances[0].InstanceId' --output text 2>"$user_data_dir/launch-err.txt")"; then
      break
    fi
    echo "  $az: $(tr '\n' ' ' < "$user_data_dir/launch-err.txt" | cut -c1-200)" >&2
    id=""
  done
  [ -n "$id" ] || die "no capacity (or permission) for server $i in any AZ"
  launched+=("$id"); ids+=("$id")
  echo "  server $i: $id ($az)"
done
echo "launched ${#ids[@]} server(s); expiry $expiry. Waiting (Ctrl-C terminates them)."

# ---- wait -------------------------------------------------------------------------------------------
deadline=$(( launch_epoch + mins * 60 + 900 ))
failed=0
done_slices=" "
while :; do
  for ((i = 0; i < count; i++)); do
    case "$done_slices" in *" $i "*) continue;; esac
    if awsx s3api head-object --bucket "$bucket" --key "batches/$batch/slice-$i/DONE" >/dev/null 2>&1; then
      done_slices="$done_slices$i "
      echo "  server $i finished ($(date +%T))"
    fi
  done
  [ "$(echo "$done_slices" | wc -w | tr -d ' ')" -ge "$count" ] && break
  # A server that is gone without a DONE marker was interrupted (spot reclaim) or crashed hard.
  alive="$(awsx ec2 describe-instances --instance-ids "${ids[@]}" --query 'Reservations[].Instances[?State.Name!=`terminated`].InstanceId' --output text 2>/dev/null | wc -w | tr -d ' ')"
  if [ "$alive" = 0 ]; then
    sleep 20   # the DONE marker is uploaded just before the server powers off
    for ((i = 0; i < count; i++)); do
      case "$done_slices" in *" $i "*) ;; *)
        awsx s3api head-object --bucket "$bucket" --key "batches/$batch/slice-$i/DONE" >/dev/null 2>&1 \
          && done_slices="$done_slices$i " || { echo "  server $i ended without finishing (spot interruption?): games ${slice_games[$i]} from seed ${slice_seeds[$i]} are missing" >&2; failed=1; };;
      esac
    done
    break
  fi
  [ "$(date -u +%s)" -lt "$deadline" ] || { echo "cloud: batch past its deadline" >&2; failed=1; break; }
  sleep 30
done

# ---- fetch, merge, report -------------------------------------------------------------------------------
dest="$RESULTS_ROOT/$batch"
mkdir -p "$dest"
awsx s3 sync --no-progress "s3://$bucket/batches/$batch" "$dest" >/dev/null
awsx ec2 terminate-instances --instance-ids "${ids[@]}" >/dev/null 2>&1 || true   # idempotent; already gone is fine
if [ "$bin" != legatum-sim ]; then
  ls "$dest"/slice-*/out/* >/dev/null 2>&1 || { echo "cloud: no output came back; see $dest/slice-*/batch.log" >&2; failed=1; }
elif ls "$dest"/slice-*/out/*.json.gz >/dev/null 2>&1; then
  python3 "$CLOUD_ROOT/tools/cloud/merge.py" "$dest" || failed=1
else
  echo "cloud: no reports came back; see $dest/slice-*/batch.log" >&2; failed=1
fi

total_secs=0; bad=0
for ((i = 0; i < count; i++)); do
  f="$dest/slice-$i/DONE"; [ -f "$f" ] || continue
  rc="$(sed -n 's/^rc=//p' "$f")"; end="$(sed -n 's/^end=//p' "$f")"
  [ "$rc" = 0 ] || { echo "  server $i: $bin exited $rc (see $dest/slice-$i/batch.log)" >&2; bad=1; }
  total_secs=$(( total_secs + end - launch_epoch ))
done
hours_used="$(awk -v s="$total_secs" 'BEGIN{printf "%.2f", s/3600}')"
cost_used="$(awk -v h="$hours_used" -v e="$hour_each" 'BEGIN{printf "%.2f", h*e}')"
echo "server-hours used: $hours_used (about \$$cost_used at the launch-time spot price; the AWS bill is authoritative)"
echo "results: $dest (merged.json.gz, per-server reports under slice-*/out)"
finished=1
trap - EXIT
[ "$failed" = 0 ] && [ "$bad" = 0 ] || { echo "batch $batch FAILED (partial results above); check tools/cloud/status.sh" >&2; exit 1; }
echo "batch $batch done"

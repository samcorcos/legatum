# shellcheck shell=bash disable=SC2034
# Shared by tools/cloud/*.sh (sourced, not run). Bash 3.2 compatible (macOS default).
#
# Configuration comes from the environment, else from the git-ignored .private/cloud.env of this checkout or of the
# main checkout (KEY=value lines; .private.example/cloud.env shows the shape). Nothing here reads ~/.aws files itself,
# and no account-specific name is written in the repository.
#   LEGATUM_AWS_PROFILE      AWS CLI profile of the sims launcher IAM user (infra/cloud-sims `launcher_user_name`).
#                            No default: unset means the AWS CLI's own default credential chain (AWS_PROFILE and so on).
#   LEGATUM_CLOUD_REGION     default us-east-1 (must match infra/cloud-sims `region`)
#   LEGATUM_CLOUD_BUCKET     default legatum-sims-<account id> (the bucket infra/cloud-sims creates)
#   LEGATUM_CLOUD_RESULTS    default ~/LegatumRuns/cloud

CLOUD_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# cloud_private_env: take LEGATUM_* settings the environment does not already have from .private/cloud.env.
cloud_private_env() {
  local f line key main
  main="$(git -C "$CLOUD_ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; main="${main%/.git}"
  for f in "$CLOUD_ROOT/.private/cloud.env" "${main:+$main/.private/cloud.env}"; do
    [ -n "$f" ] && [ -f "$f" ] || continue
    while IFS= read -r line; do
      case "$line" in
        LEGATUM_[A-Z_]*=*) key="${line%%=*}"; [ -n "${!key:-}" ] || export "$key=${line#*=}";;
      esac
    done < "$f"
    return 0
  done
}
cloud_private_env
LEGATUM_AWS_PROFILE="${LEGATUM_AWS_PROFILE:-}"
REGION="${LEGATUM_CLOUD_REGION:-us-east-1}"
TAG_PROJECT="legatum-sims"
INSTANCE_PROFILE="legatum-sims-server"
SECURITY_GROUP_NAME="legatum-sims-server"
RESULTS_ROOT="${LEGATUM_CLOUD_RESULTS:-$HOME/LegatumRuns/cloud}"

die() { echo "cloud: $*" >&2; exit 1; }

# awsx <aws args...>: the AWS CLI pinned to the sims profile (when one is configured) and region.
awsx() { aws ${LEGATUM_AWS_PROFILE:+--profile "$LEGATUM_AWS_PROFILE"} --region "$REGION" "$@"; }

require_profile() {
  command -v aws >/dev/null || die "aws CLI not installed"
}

# bucket_name: LEGATUM_CLOUD_BUCKET, else legatum-sims-<account id>.
bucket_name() {
  if [ -n "${LEGATUM_CLOUD_BUCKET:-}" ]; then echo "$LEGATUM_CLOUD_BUCKET"; return; fi
  local acct
  acct="$(awsx sts get-caller-identity --query Account --output text)" || die "cannot read the account id (set LEGATUM_AWS_PROFILE in the environment or .private/cloud.env; current: ${LEGATUM_AWS_PROFILE:-the default credential chain})"
  echo "legatum-sims-$acct"
}

# vcpus_of <instance type>: from the size name; refuses sizes it does not know (an unknown size has no cost estimate).
vcpus_of() {
  case "${1#*.}" in
    medium) echo 1;; large) echo 2;; xlarge) echo 4;; 2xlarge) echo 8;; 4xlarge) echo 16;;
    8xlarge) echo 32;; 12xlarge) echo 48;; 16xlarge) echo 64;; 24xlarge) echo 96;; 32xlarge) echo 128;;
    48xlarge) echo 192;; metal) echo 64;;
    *) die "unknown instance size in '$1'";;
  esac
}

# running_servers: tab-separated "id  type  launch-time  expiry  batch  owner  state" for every
# non-terminated legatum-sims server, one per line.
running_servers() {
  awsx ec2 describe-instances \
    --filters "Name=tag:project,Values=$TAG_PROJECT" "Name=instance-state-name,Values=pending,running,stopping,stopped" \
    --query 'Reservations[].Instances[].[InstanceId,InstanceType,LaunchTime,Tags[?Key==`expiry`]|[0].Value,Tags[?Key==`batch`]|[0].Value,Tags[?Key==`owner`]|[0].Value,State.Name]' \
    --output text
}

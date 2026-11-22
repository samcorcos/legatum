#!/usr/bin/env bash
# terraform for the build box with its state outside the checkout (worktrees get deleted):
#   infra/cloud-box/tf.sh plan -out plan.bin | apply plan.bin | output | ...
set -euo pipefail
cd "$(dirname "$0")"
state="${LEGATUM_BOX_TFSTATE:-$HOME/.legatum-box/terraform.tfstate}"
mkdir -p "$(dirname "$state")"
terraform init -input=false -reconfigure -backend-config="path=$state" >/dev/null
exec terraform "$@"

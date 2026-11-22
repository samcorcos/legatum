#!/usr/bin/env bash
# Linux ARM sim binary for the cloud servers (Amazon Linux 2023 glibc), built in Docker on the box with a
# per-slot target volume. The box itself is Ubuntu 24.04 (glibc 2.39), whose binaries would not run on the
# servers, hence the container. Output: target/cloud/linux-arm64 in the slot (the Mac fetches it as an artifact).
set -euo pipefail
export LEGATUM_CLOUD_LOCAL_BUILD=1
examples="${LEGATUM_CLOUD_EXAMPLES:-}"; export LEGATUM_CLOUD_EXAMPLES="${examples//,/ }"   # remote.sh --env values cannot hold spaces
export LEGATUM_CLOUD_TARGET_VOLUME="legatum-sim-cargo-target-${LEGATUM_BOX_SLOT:-s1}"
. /data/bin/budget.sh && box_core_budget_apply   # this job's share of the cores (see budget.sh)
export CARGO_BUILD_JOBS="${BOX_CARGO_JOBS:-24}"
exec tools/cloud/build-linux.sh

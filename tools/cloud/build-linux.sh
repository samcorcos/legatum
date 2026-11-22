#!/usr/bin/env bash
# Build legatum-sim and the sim_bench example for Linux ARM (Graviton), sim profile, from the Mac.
#
#   tools/cloud/build-linux.sh            # -> target/cloud/linux-arm64/{legatum-sim,sim_bench,BUILD_INFO}
#
# How: inside a linux/arm64 Amazon Linux 2023 container (the servers' OS), with Docker Desktop
# running (it is native on Apple silicon, no emulation). The source is mounted read-only; cargo's
# target dir and registry live in named Docker volumes, so rebuilds are incremental and the Mac's
# target/ is untouched. The binaries are built with the same rustc version as this Mac.
# Docker Desktop's memory limit must allow a fat-LTO build (8 GB worked); raise it if the linker is killed.
# Set LEGATUM_CLOUD_TARGET_VOLUME to a worktree-specific Docker volume when builds from several
# checkouts overlap; sharing one target volume can link a dependency from another checkout.
# By default the build runs on the cloud build box (tools/cloud/box.sh; the Mac stays free) and the same
# target/cloud/linux-arm64 comes back. Set LEGATUM_CLOUD_LOCAL_BUILD=1 to force the local Docker build below; it is also
# the fallback when the box cannot be used at all (remote.sh exit 3). A build that fails on the box fails here.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

if [ "${LEGATUM_CLOUD_LOCAL_BUILD:-0}" != 1 ]; then
  box_env=()
  [ -z "${LEGATUM_CLOUD_EXAMPLES:-}" ] || box_env=(--env "LEGATUM_CLOUD_EXAMPLES=$(printf '%s' "$LEGATUM_CLOUD_EXAMPLES" | tr ' ' ',')")
  rc=0
  "$root/tools/cloud/remote.sh" "$root" --with-uncommitted --label "build-sim" --artifact target/cloud/linux-arm64 \
    --artifact-dir "$root" ${box_env[@]+"${box_env[@]}"} -- bash /data/bin/build-sim.sh || rc=$?
  if [ "$rc" -eq 0 ]; then
    cat "$root/target/cloud/linux-arm64/BUILD_INFO"; echo "built $root/target/cloud/linux-arm64 on the build box"; exit 0
  elif [ "$rc" -ne 3 ]; then
    exit "$rc"
  fi
  if [ "${LEGATUM_CLOUD_REQUIRE_BOX:-0}" = 1 ]; then
    echo "build-linux: cloud build box unavailable; local fallback disabled" >&2
    exit 3
  fi
  echo "build-linux: the build box is unavailable, building locally in Docker" >&2
fi

command -v docker >/dev/null || { echo "build-linux: docker not installed" >&2; exit 1; }
docker info >/dev/null 2>&1 || { echo "build-linux: Docker is not running (open -a Docker)" >&2; exit 1; }

rust_version="${RUST_VERSION:-$(rustc --version | awk '{print $2}')}"
image="legatum-sim-build:rust-$rust_version"
out="$root/target/cloud/linux-arm64"
target_volume="${LEGATUM_CLOUD_TARGET_VOLUME:-legatum-sim-cargo-target}"
mkdir -p "$out"

echo "== image $image"
docker build --platform linux/arm64 --build-arg "RUST_VERSION=$rust_version" \
  -t "$image" -f tools/cloud/Dockerfile.build tools/cloud

commit="$(git rev-parse --short=8 HEAD)"
dirty=""; git diff --quiet HEAD -- 2>/dev/null || dirty="-dirty"
extra="${LEGATUM_CLOUD_EXAMPLES:-}"
case "$extra" in *[!A-Za-z0-9_\ ]*) echo "build-linux: LEGATUM_CLOUD_EXAMPLES takes example names only" >&2; exit 1;; esac
echo "== cargo build --profile sim (commit $commit$dirty${extra:+, examples: $extra})"
docker run --rm --platform linux/arm64 \
  -v "$root":/src:ro \
  -v legatum-sim-cargo-registry:/root/.cargo/registry \
  -v "$target_volume":/target \
  -v "$out":/out \
  -e CARGO_TARGET_DIR=/target \
  -e CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}" \
  -e EXTRA="$extra" \
  "$image" bash -euc '
    cargo build --locked --profile sim -p legatum-sim --bin legatum-sim --example sim_bench \
      $(for e in $EXTRA; do printf -- "--example %s " "$e"; done)
    cp /target/sim/legatum-sim /target/sim/examples/sim_bench /out/
    for e in $EXTRA; do cp "/target/sim/examples/$e" /out/; done
    ldd /out/legatum-sim | grep -v "=>.*/lib" || true
    ldd --version | head -1
  '

printf 'commit %s%s\nrustc %s\nbuilt %s\ntarget aarch64-unknown-linux-gnu (Amazon Linux 2023 container), profile sim\n' \
  "$commit" "$dirty" "$rust_version" "$(date -u +%FT%TZ)" > "$out/BUILD_INFO"
ls -l "$out"
echo "built $out/legatum-sim and $out/sim_bench"

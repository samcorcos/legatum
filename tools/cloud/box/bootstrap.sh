#!/usr/bin/env bash
# Runs ON the build box as the ubuntu user (via `tools/cloud/box.sh prepare`). Idempotent: every step checks
# before it installs, so re-running only fills gaps. The toolchain lives on /data (rustup, cargo, node, Playwright
# browsers, docker), so replacing the instance costs only the apt packages and a re-run of this script.
set -euo pipefail
DATA=/data
log() { echo "[bootstrap $(date +%T)] $*"; }
[ -d "$DATA/jobs" ] || { echo "bootstrap: $DATA is not set up (user-data did not finish?)" >&2; exit 1; }
RUST_VERSION="${RUST_VERSION:?RUST_VERSION is required}"
export DEBIAN_FRONTEND=noninteractive

log "apt packages"
sudo apt-get -o DPkg::Lock::Timeout=300 update -qq
sudo apt-get -o DPkg::Lock::Timeout=300 install -y -qq build-essential pkg-config libssl-dev cmake git curl unzip zip jq rsync python3 \
  ca-certificates docker.io xfsprogs util-linux time htop >/dev/null

log "directories and limits"
sudo mkdir -p "$DATA/LegatumRuns" "$DATA/slots" "$DATA/jobs/running" "$DATA/npm-cache" "$DATA/ms-playwright"
sudo chown -R ubuntu:ubuntu "$DATA" 2>/dev/null || true
[ -e "$HOME/LegatumRuns" ] || ln -s "$DATA/LegatumRuns" "$HOME/LegatumRuns"
printf '* soft nofile 1048576\n* hard nofile 1048576\n' | sudo tee /etc/security/limits.d/legatum.conf >/dev/null

log "idle watcher (repo copy)"
sudo install -m 755 "$DATA/bin/idle-watch.sh" /usr/local/bin/legatum-idle-watch

log "docker (data-root on /data)"
sudo mkdir -p /etc/docker "$DATA/docker"
echo '{"data-root": "/data/docker", "log-driver": "json-file", "log-opts": {"max-size": "10m", "max-file": "2"}}' | sudo tee /etc/docker/daemon.json >/dev/null
sudo systemctl enable --now docker
sudo systemctl restart docker
sudo usermod -aG docker ubuntu

if ! command -v aws >/dev/null; then
  log "aws cli v2"
  tmp="$(mktemp -d)"; curl -fsSL "https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip" -o "$tmp/awscli.zip"
  unzip -q "$tmp/awscli.zip" -d "$tmp"; sudo "$tmp/aws/install" --update; rm -rf "$tmp"
fi

acct="$(aws sts get-caller-identity --query Account --output text --region us-east-1)"
cat > "$DATA/bin/config" <<CFG
BUCKET=legatum-sims-$acct
REGION=us-east-1
SLOTS=12
CFG

export CARGO_HOME="$DATA/cargo" RUSTUP_HOME="$DATA/rustup"
export PATH="$CARGO_HOME/bin:$DATA/node/bin:$PATH"
if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
  log "rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
fi
if ! rustup toolchain list | grep -q "^$RUST_VERSION"; then
  log "rust $RUST_VERSION"
  rustup toolchain install "$RUST_VERSION" --profile minimal -c rustfmt -c clippy -t wasm32-unknown-unknown
fi
rustup default "$RUST_VERSION" >/dev/null

if ! command -v wasm-pack >/dev/null; then
  log "wasm-pack (cargo install, a few minutes)"
  cargo install wasm-pack --locked
fi

if [ ! -x "$DATA/node/bin/node" ]; then
  log "node 22 LTS"
  base="https://nodejs.org/dist/latest-v22.x"
  file="$(curl -fsSL "$base/SHASUMS256.txt" | awk '/linux-arm64\.tar\.xz$/{print $2}')"
  mkdir -p "$DATA/node" && curl -fsSL "$base/$file" | tar xJ -C "$DATA/node" --strip-components=1
fi
node --version

if [ -f "$DATA/bootstrap-src/app/package-lock.json" ]; then
  log "Playwright system libraries and browsers (from the repo's lockfile)"
  mkdir -p "$DATA/bootstrap-app" && cp "$DATA/bootstrap-src/app/package.json" "$DATA/bootstrap-src/app/package-lock.json" "$DATA/bootstrap-app/"
  ( cd "$DATA/bootstrap-app" && npm ci --no-audit --no-fund >/dev/null \
    && sudo env "PATH=$PATH" PLAYWRIGHT_BROWSERS_PATH="$DATA/ms-playwright" npx playwright install-deps chromium webkit \
    && PLAYWRIGHT_BROWSERS_PATH="$DATA/ms-playwright" npx playwright install chromium webkit )
fi

date -u +%FT%TZ > "$DATA/.bootstrapped"
log "done: rust $(rustc --version), node $(node --version), $(docker --version 2>/dev/null || sudo docker --version)"

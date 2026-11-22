#!/usr/bin/env bash
# Build the WASM API for the web UI into app/src/wasm-pkg (git-ignored).
# Usage: scripts/build-wasm.sh [--dev]
# Set LEGATUM_WASM_NO_OPT=1 when the host cannot execute wasm-opt.
# Set LEGATUM_PLAYTEST_PROFILE=1 for the faster thin-LTO playtest profile (wasm-playtest); the default stays wasm-release.
set -euo pipefail
# Codex lanes run this on the cloud build box (what it changes comes back); see tools/cloud/offload.sh.
_self="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
if [ "${LEGATUM_OFFLOAD:-0}" = 1 ] && [ -z "${LEGATUM_ON_BOX:-}" ] && [ -x "$(dirname "$0")/../tools/cloud/offload.sh" ]; then
  exec "$(dirname "$0")/../tools/cloud/offload.sh" --artifact app/src/wasm-pkg -- "$_self" "$@"
fi
# Build timings are recorded only on a machine that keeps them: when the timings folder already
# exists, or LEGATUM_TIMINGS=1 opts in. A fresh clone never writes outside the repository.
_timings_dir="${LEGATUM_TIMINGS_DIR:-$HOME/LegatumRuns/timings}"
if [ "${LEGATUM_TIMED_BUILD_WASM:-0}" != 1 ] && [ -f "$(dirname "$0")/../tools/ops/timed.py" ] \
  && { [ "${LEGATUM_TIMINGS:-0}" = 1 ] || [ -d "$_timings_dir" ]; }; then
  profile_label=wasm-release
  [ "${LEGATUM_PLAYTEST_PROFILE:-0}" != 1 ] || profile_label=wasm-playtest
  [ "${1:-}" != "--dev" ] || profile_label=dev
  exec python3 "$(dirname "$0")/../tools/ops/timed.py" build build-wasm --label "profile=$profile_label" -- \
    env LEGATUM_TIMED_BUILD_WASM=1 "$0" "$@"
fi
cd "$(dirname "$0")/.."
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

profile_args=(--profile wasm-release)
if [ "${LEGATUM_PLAYTEST_PROFILE:-0}" = 1 ]; then profile_args=(--profile wasm-playtest); fi
if [ "${1:-}" = "--dev" ]; then profile_args=(--dev); fi

out_dir=app/src/wasm-pkg
cache_dir="${LEGATUM_WASM_CACHE_DIR:-}"
cache_key=""
cache_entry=""

# A curated source export has no .git metadata to enumerate exact cache inputs.
if [ -n "$cache_dir" ] && ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "WASM exact-input cache disabled: no Git metadata" >&2
  cache_dir=""
fi

if [ -n "$cache_dir" ]; then
  mkdir -p "$cache_dir"
  cache_key="$(
    {
      printf 'profile=%s\nno_opt=%s\n' "${profile_args[*]}" "${LEGATUM_WASM_NO_OPT:-0}"
      python3 - <<'PY'
import hashlib
import json
import pathlib
import subprocess

digest = hashlib.sha256()
try:
    metadata = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--filter-platform", "wasm32-unknown-unknown"],
        check=True,
        capture_output=True,
        text=True,
    )
    workspace = json.loads(metadata.stdout)
    packages = {package["id"]: package for package in workspace["packages"]}
    nodes = {node["id"]: node for node in workspace["resolve"]["nodes"]}
    wasm = next(package for package in workspace["packages"] if package["name"] == "legatum-wasm")
    todo = [wasm["id"]]
    compiled = set()
    while todo:
        package_id = todo.pop()
        if package_id in compiled:
            continue
        compiled.add(package_id)
        for dependency in nodes[package_id]["deps"]:
            if any(kind["kind"] in (None, "build") for kind in dependency["dep_kinds"]):
                todo.append(dependency["pkg"])

    crate_dirs = set()
    workspace_members = set(workspace["workspace_members"])
    root = pathlib.Path.cwd()
    for package_id in compiled & workspace_members:
        manifest = pathlib.Path(packages[package_id]["manifest_path"])
        crate_dir = manifest.parent.resolve().relative_to(root.resolve()).as_posix()
        if crate_dir.startswith("crates/"):
            crate_dirs.add(crate_dir)
    if "crates/wasm" not in crate_dirs:
        raise ValueError("legatum-wasm crate directory is outside the crate scope")
except Exception:
    # A failed/incomplete metadata query must preserve the old conservative scope.
    crate_dirs = None

paths = subprocess.check_output(
    ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"]
).split(b"\0")
for raw in sorted(set(path for path in paths if path)):
    path = pathlib.Path(raw.decode())
    name = path.as_posix()
    in_crate_scope = name.startswith("crates/") and (
        crate_dirs is None or any(name.startswith(crate_dir + "/") for crate_dir in crate_dirs)
    )
    if not (
        name in ("Cargo.toml", "Cargo.lock")
        or name == "scripts/build-wasm.sh"
        or in_crate_scope
        or name.startswith(".cargo/")
        or path.name in ("rust-toolchain", "rust-toolchain.toml")
    ):
        continue
    digest.update(name.encode())
    digest.update(b"\0")
    try:
        digest.update(path.read_bytes())
    except FileNotFoundError:
        digest.update(b"<deleted>")
    digest.update(b"\0")
print(digest.hexdigest())
PY
      rustc -Vv
      wasm-pack --version
      wasm_bindgen_version="$(wasm-bindgen --version 2>/dev/null || echo managed-by-wasm-pack)"
      wasm_opt_version="$(wasm-opt --version 2>/dev/null || echo managed-by-wasm-pack)"
      printf 'wasm-bindgen=%s\nwasm-opt=%s\n' "$wasm_bindgen_version" "$wasm_opt_version"
      if [ "${LEGATUM_WASM_NO_OPT:-0}" = 1 ]; then
        printf '%s\n' 'wasm-opt=disabled'
      fi
      env | LC_ALL=C sort | grep -E '^(CARGO|RUST|LEGATUM_WASM|CC=|CXX=|AR=|CFLAGS=|CXXFLAGS=|LDFLAGS=|PKG_CONFIG)' || true
    } | sha256sum | awk '{print $1}'
  )"
  cache_entry="$cache_dir/$cache_key"
  exec 9>"$cache_dir/$cache_key.lock"
  flock 9
fi

cache_manifest() {
  ( cd "$1" && find . -type f -print0 | sort -z | xargs -0 sha256sum )
}

if [ -n "$cache_entry" ] && [ -d "$cache_entry/pkg" ] && [ -f "$cache_entry/manifest" ]; then
  if cache_manifest "$cache_entry/pkg" | cmp -s "$cache_entry/manifest" -; then
    rm -rf "$out_dir"
    mkdir -p "$out_dir"
    cp -a "$cache_entry/pkg/." "$out_dir/"
    echo "restored optimized WASM package from exact-input cache ($cache_key)"
    exit 0
  fi
  echo "discarding invalid optimized WASM cache entry ($cache_key)" >&2
  rm -rf "$cache_entry"
fi

rm -rf "$out_dir"
mkdir -p "$out_dir"
# --out-dir is relative to the crate directory.
if [ "${LEGATUM_WASM_NO_OPT:-0}" = 1 ]; then
  wasm-pack build crates/wasm --target web "${profile_args[@]}" --no-opt --out-dir ../../app/src/wasm-pkg --out-name legatum
else
  wasm-pack build crates/wasm --target web "${profile_args[@]}" --out-dir ../../app/src/wasm-pkg --out-name legatum
fi

if [ -n "$cache_entry" ]; then
  cache_tmp="$(mktemp -d "$cache_dir/.tmp-$cache_key.XXXXXX")"
  mkdir -p "$cache_tmp/pkg"
  cp -a "$out_dir/." "$cache_tmp/pkg/"
  cache_manifest "$cache_tmp/pkg" > "$cache_tmp/manifest"
  printf '%s\n' "$cache_key" > "$cache_tmp/key"
  mv "$cache_tmp" "$cache_entry"
fi

echo "built app/src/wasm-pkg (import init, { Game } from './wasm-pkg/legatum.js')"

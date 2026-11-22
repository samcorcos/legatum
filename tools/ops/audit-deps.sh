#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

command -v cargo-audit >/dev/null 2>&1 || { echo "audit-deps: cargo-audit is not installed (cargo install cargo-audit --locked)" >&2; exit 2; }
command -v npm >/dev/null 2>&1 || { echo "audit-deps: npm is not installed" >&2; exit 2; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
vulnerabilities=0

echo '=== Vulnerabilities ==='
for lockfile in Cargo.lock; do
  echo "--- cargo audit: $lockfile ---"
  out="$tmp/audit-${lockfile//\//_}.json"
  cargo audit --file "$lockfile" --json >"$out" 2>"$tmp/cargo.err" || true
  python3 - "$out" "$lockfile" <<'PY' || exit 2
import json, sys
path, lockfile = sys.argv[1:]
try:
    report = json.load(open(path))
except (OSError, json.JSONDecodeError) as exc:
    print(f"Could not parse cargo-audit output for {lockfile}: {exc}")
    raise SystemExit(2)
vulns = report.get("vulnerabilities", {}).get("list", [])
for item in vulns:
    open(path + ".vulnerable", "w").close()
    advisory = item.get("advisory", {})
    package = item.get("package", {})
    print(f"{package.get('name')} {package.get('version')}: {advisory.get('id')} {advisory.get('title', '')}")
if not vulns:
    print("No vulnerable crates reported.")
PY
done

echo '--- npm audit: app/ (registry) ---'
(cd app && npm audit --json >"$tmp/npm.json") 2>"$tmp/npm.err" || true
python3 - "$tmp/npm.json" <<'PY' || exit 2
import json, sys
try:
    report = json.load(open(sys.argv[1]))
except (OSError, json.JSONDecodeError) as exc:
    print(f"Could not parse npm audit output: {exc}")
    raise SystemExit(2)
vulns = report.get("vulnerabilities", {})
found = False
for name, item in vulns.items():
    if item.get("severity") in {"critical", "high", "moderate", "low"}:
        open(sys.argv[1] + ".vulnerable", "w").close()
        found = True
        print(f"{name}: {item.get('severity')} ({item.get('isDirect') and 'direct' or 'transitive'})")
if not found:
    print("No vulnerable npm packages reported.")
PY

echo '=== Unmaintained crates ==='
for lockfile in Cargo.lock; do
  echo "--- cargo audit: $lockfile ---"
  out="$tmp/audit-${lockfile//\//_}.json"
  python3 - "$out" <<'PY'
import json, sys
try:
    report = json.load(open(sys.argv[1]))
except (OSError, json.JSONDecodeError):
    raise SystemExit(0)
warnings = report.get("warnings", {}).get("unmaintained", [])
for item in warnings:
    package = item.get("package", {})
    advisory = item.get("advisory", {})
    name = package.get("name", "")
    # proc-macro-error is a known informational warning from Linux GTK dependencies.
    note = " (informational: Linux GTK dependency)" if name == "proc-macro-error" else ""
    print(f"{name} {package.get('version')}: {advisory.get('id', '')}{note}")
if not warnings:
    print("No unmaintained crates reported.")
PY
done

if compgen -G "$tmp/*.vulnerable" >/dev/null; then exit 1; fi
exit 0

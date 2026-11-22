#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
PW_REQUEST_DIR="$(cd "$(dirname "$0")" && pwd)"
export PW_REQUEST_DIR
exec python3 - "$@" <<'PY'
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid

parser = argparse.ArgumentParser(usage="pw-request.sh <worktree> <spec...> [-g grep] [--project game]")
parser.add_argument("worktree")
parser.add_argument("specs", nargs="+")
parser.add_argument("-g", "--grep")
parser.add_argument("--project")
args = parser.parse_args()
common = subprocess.run(["git", "-C", os.environ["PW_REQUEST_DIR"], "rev-parse", "--path-format=absolute", "--git-common-dir"],
                        check=True, capture_output=True, text=True).stdout.strip()
queue = Path(os.environ.get("LEGATUM_PW_BRIDGE_DIR") or Path(common).parent / ".progress/pw-bridge")
requests = queue / "requests"
requests.mkdir(parents=True, exist_ok=True)
request_id = uuid.uuid4().hex
request = {"id": request_id, "worktree": args.worktree, "specs": args.specs}
if args.grep is not None:
    request["grep"] = args.grep
if args.project is not None:
    request["project"] = args.project
staging = queue / f".{request_id}.tmp"
staging.write_text(json.dumps(request) + "\n")
os.replace(staging, requests / f"{request_id}.json")
result = queue / "results" / request_id
print(f"Playwright request {request_id}; result: {result}", flush=True)
deadline = time.monotonic() + 45 * 60
while time.monotonic() < deadline:
    summary = result / "summary.json"
    if summary.is_file():
        payload = json.loads(summary.read_text())
        print(json.dumps(payload, indent=2))
        print(f"Output: {result / 'output.log'}")
        if (result / "shots").is_dir():
            print(f"Screenshots: {result / 'shots'}")
        sys.exit(0 if payload.get("exit_code") == 0 and payload.get("failed") == 0 else 1)
    time.sleep(15)
print(f"Timed out waiting for request {request_id}; inspect {requests / (request_id + '.json')}", file=sys.stderr)
sys.exit(124)
PY

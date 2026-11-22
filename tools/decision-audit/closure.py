#!/usr/bin/env python3
"""Re-run known-gap turn fixtures against a newer engine (current master by default)."""
import argparse
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from patterns import write_table

ROOT = Path(__file__).resolve().parents[2]
LEDGER = ROOT / "docs/playtests/diffs/patterns.jsonl"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("pattern_id", nargs="?", help="limit to one pattern")
    ap.add_argument("--engine", default="master", help="git revision to check (default: current master)")
    ap.add_argument("--write", action="store_true", help="record observed closure state in patterns.jsonl")
    args = ap.parse_args()
    records = [json.loads(s) for s in LEDGER.read_text().splitlines()]
    selected = [r for r in records if r["fixtures"] and (args.pattern_id is None or r["pattern_id"] == args.pattern_id)]
    if not selected:
        ap.error("no fixtures for this pattern")
    revision = subprocess.check_output(["git", "rev-parse", args.engine], cwd=ROOT, text=True).strip()
    with tempfile.TemporaryDirectory(prefix="legatum-closure-") as tmp:
        checkout = Path(tmp) / "engine"
        subprocess.run(["git", "worktree", "add", "--detach", str(checkout), revision], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
        try:
            test_dir = checkout / "crates/sim/tests"
            shutil.copy2(ROOT / "crates/sim/tests/decisive_turns.rs", test_dir / "decisive_turns.rs")
            shutil.copytree(ROOT / "crates/sim/tests/fixtures/decisive", test_dir / "fixtures/decisive", dirs_exist_ok=True)
            # One Cargo invocation covers all selected fixtures. This command is forwarded to
            # the build box by build-slot, even though the source is a temporary master worktree.
            cmd = ["tools/sim/build-slot.sh", "--", "cargo", "test", "-p", "legatum-sim",
                   "--test", "decisive_turns", "--profile", "release-checked", "--", "--ignored", "--nocapture"]
            run = subprocess.run(cmd, cwd=checkout, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            print(run.stdout)
            found = dict(re.findall(r"test (\w+) \.\.\. (ok|FAILED)", run.stdout))
            if not found:
                raise SystemExit(f"test runner failed before reporting fixture results (exit {run.returncode})")
            print("| Pattern | Engine | Closure | Fixtures |")
            print("|---|---|---|---|")
            for row in selected:
                outcomes = [found.get(name, "missing") for name in row["fixtures"]]
                status = "closed" if all(x == "ok" for x in outcomes) else "open" if all(x in ("ok", "FAILED") for x in outcomes) else "unverified"
                print(f"| {row['pattern_id']} | `{revision[:12]}` | {status} | {', '.join(f'{n}: {x}' for n,x in zip(row['fixtures'],outcomes))} |")
                if args.write:
                    row["closure"] = {"engine": revision, "status": status}
            if args.write:
                LEDGER.write_text("\n".join(json.dumps(row, separators=(",", ":")) for row in records) + "\n")
                write_table(records)
            if any(found.get(name) == "missing" for row in selected for name in row["fixtures"]):
                raise SystemExit("some selected fixture results were missing")
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(checkout)], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)


if __name__ == "__main__": main()

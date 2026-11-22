#!/usr/bin/env python3
"""Inventory simulation results and check the small, explicit headline set.

Provenance is a sidecar for the tree. Its default `unknown` covers legacy files;
individual entries are added only when there is evidence for a measured source.
`stamp` is for newly measured outputs, `backfill` uses recorded commits only.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
RESULTS = ROOT / "docs/sim-results"
SIDECAR = RESULTS / "provenance.json"
HEADLINES = RESULTS / "headline.json"
UNKNOWN = {"engine_commit": "unknown", "rules_version": "unknown", "ai_defaults_sha256": "unknown"}
RESULT_SUFFIXES = (".json", ".jsonl", ".json.gz", ".jsonl.gz", ".csv", ".csv.gz", ".md", ".png", ".svg", ".html", ".txt", ".log", ".log.gz", ".zst")


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def rules_version(commit):
    try:
        source = git("show", f"{commit}:crates/core/src/lib.rs")
    except subprocess.CalledProcessError:
        return "unknown"
    match = re.search(r"pub const RULES_VERSION: u32 = (\d+);", source)
    return int(match.group(1)) if match else "unknown"


def current_stamp(commit_arg=None):
    if commit_arg and (commit_arg.endswith("-dirty") or commit_arg == "unknown"):
        return dict(UNKNOWN)
    commit = git("rev-parse", f"{commit_arg or 'HEAD'}^{{commit}}")
    if not commit_arg and subprocess.run(["git", "diff", "--quiet", "HEAD", "--", "crates", "scenarios"], cwd=ROOT).returncode:
        raise ValueError("engine or scenario files are dirty; commit them before measuring")
    try:
        params = subprocess.check_output(["git", "show", f"{commit}:crates/ai/src/params.rs"], cwd=ROOT, stderr=subprocess.DEVNULL)
        defaults_hash = hashlib.sha256(params).hexdigest()
    except subprocess.CalledProcessError:
        defaults_hash = "unknown"
    return {"engine_commit": commit, "rules_version": rules_version(commit),
            "ai_defaults_sha256": defaults_hash}


def inventory():
    for path in sorted(RESULTS.rglob("*")):
        if not path.is_file() or path.name in {"provenance.json", "headline.json"}:
            continue
        if path.suffix == ".py" or path.suffix == ".sh" or "__pycache__" in path.parts:
            continue
        if path.name.endswith(RESULT_SUFFIXES):
            yield path.relative_to(ROOT).as_posix(), path


def ledger_versions(path):
    """A ledger is many measurements; its newest sidecar must not hide old rows."""
    if path.name not in {"history.jsonl", "ledger.jsonl"} or path.stat().st_size > 20_000_000:
        return None
    versions = []
    for line in path.read_text().splitlines():
        if line.strip():
            try:
                versions.append(json.loads(line).get("rules_version", "unknown"))
            except (ValueError, AttributeError):
                versions.append("unknown")
    return versions or None


def load_sidecar():
    return json.loads(SIDECAR.read_text()) if SIDECAR.exists() else {"schema": 1, "default": UNKNOWN, "files": {}}


def save_sidecar(data):
    SIDECAR.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def recorded_commit(path):
    candidates = [path] if path.suffix == ".md" else []
    candidates += [path.parent / "README.md", path.parent / "BUILD_INFO"]
    for candidate in candidates:
        if not candidate.exists() or candidate.stat().st_size > 300_000:
            continue
        content = candidate.read_text(errors="replace")
        match = re.search(r"(?:Engine commit|engine commit|^commit)\s*[` :]+([0-9a-f]{8,40})(?:`|\b)", content, re.M)
        if match:
            if content[match.end():].startswith("-dirty"):
                return None
            return match.group(1)
    if path.suffix == ".json" and path.stat().st_size < 10_000_000:
        try:
            data = json.loads(path.read_text())
            if isinstance(data, dict):
                value = data.get("engine_commit") or data.get("meta", {}).get("engine_commit")
                if isinstance(value, str) and re.fullmatch(r"[0-9a-f]{8,40}", value):
                    return value
        except (ValueError, AttributeError):
            pass
    return None


def backfill(data, paths):
    for name in paths:
        path = ROOT / name
        if not path.is_file():
            continue  # The first nightly run will create canonical headline files.
        commit = recorded_commit(path)
        if not commit:
            continue
        try:
            full = git("rev-parse", f"{commit}^{{commit}}")
        except subprocess.CalledProcessError:
            continue
        data["files"][name] = dict(current_stamp(full), source="recorded commit")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail for stale or unknown headline results")
    sub = parser.add_subparsers(dest="command")
    stamp = sub.add_parser("stamp", help="stamp newly measured paths with this checkout's engine")
    stamp.add_argument("--commit", help="actual binary source commit (dirty sources become unknown)")
    stamp.add_argument("paths", nargs="+")
    sub.add_parser("backfill", help="backfill headline paths from recorded README or BUILD_INFO commits")
    args = parser.parse_args()
    data = load_sidecar()
    headlines = json.loads(HEADLINES.read_text())["results"] if HEADLINES.exists() else []
    headline_paths = {row["path"] for row in headlines}
    if args.command == "stamp":
        value = current_stamp(args.commit)
        for name in args.paths:
            path = ROOT / name
            if not path.is_file() or not path.is_relative_to(RESULTS):
                parser.error(f"not a result file: {name}")
            data["files"][name] = dict(value, source="measured checkout")
        save_sidecar(data)
        return
    if args.command == "backfill":
        for name in headline_paths:
            if data["files"].get(name, {}).get("source") == "recorded commit":
                del data["files"][name]
        backfill(data, headline_paths)
        save_sidecar(data)
        return
    master = git("rev-parse", "master")
    master_version = rules_version(master)
    if master_version == "unknown":
        parser.error("cannot read master's RULES_VERSION")
    now = datetime.now(timezone.utc)
    failures = []
    seen = set()
    print(f"master {master[:12]} RULES_VERSION={master_version}")
    print("STATUS   RULES   AGE(d)  RESULT")
    for name, path in inventory():
        seen.add(name)
        stamp = data["files"].get(name, data.get("default", UNKNOWN))
        versions = ledger_versions(path) or [stamp.get("rules_version", "unknown")]
        statuses = ["UNKNOWN" if not isinstance(v, int) else "STALE" if v < master_version else "CURRENT" if v == master_version else "FUTURE" for v in versions]
        status = next((s for s in ("STALE", "UNKNOWN", "FUTURE") if s in statuses), "CURRENT")
        if status == "CURRENT" and stamp.get("engine_commit") == "unknown":
            status = "UNKNOWN"
        version = ",".join(str(v) for v in sorted(set(versions), key=str))
        days = (now - datetime.fromtimestamp(path.stat().st_mtime, timezone.utc)).total_seconds() / 86400
        print(f"{status:<8} {str(version):<7} {days:6.1f}  {name}")
        if name in headline_paths and status != "CURRENT":
            failures.append((name, status))
    for name in sorted(headline_paths - seen):
        failures.append((name, "MISSING"))
    if failures:
        print("headline issues: " + ", ".join(f"{name} ({status})" for name, status in failures), file=sys.stderr)
    if args.check and failures:
        sys.exit(1)


if __name__ == "__main__":
    main()

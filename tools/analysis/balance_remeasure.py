#!/usr/bin/env python3
"""Run paired balance cohorts, retaining commands, exit codes and compressed reports.

Defaults to two processes with four workers each; the configurable total is at
most eight workers. Every simulator is invoked with nice -n 10.
Receipts record observed priority because a managed launcher may already be nicer.
Outputs are append-only: an existing successful receipt is verified and reused.
"""
import argparse
import concurrent.futures
import datetime
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time


def sha256(path):
    with Path(path).open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def run_one(binary, output, variant, cohort, games, seed, flags, threads=4):
    label = f"{variant}-standard-{cohort}"
    folder = output / label
    folder.mkdir(parents=True, exist_ok=True)
    receipt_path = folder / "receipt.json"
    command = ["nice", "-n", "10", str(binary), "--brain", "scripted",
               "--difficulty", "hard", "--players", "8", "--map-size", "medium",
               "--threads", str(threads), "--games", str(games), "--seed", str(seed),
               "--label", label, "--out-dir", str(folder),
               "--status-file", str(folder / "status.json")]
    if flags:
        command += ["--rules-experiment", flags]
    binary_hash = sha256(binary)
    if receipt_path.exists():
        receipt = json.loads(receipt_path.read_text())
        assert receipt["exit_code"] == 0 and receipt["command"] == command
        assert receipt["binary_sha256"] == binary_hash
        assert sha256(folder / "report.json.gz") == receipt["report_sha256"]
        print(f"Verified existing {label}", flush=True)
        return receipt
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    start = time.monotonic()
    print(f"Starting {label}: {games} games from {seed}", flush=True)
    with (folder / "run.log").open("w") as log:
        with subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT) as process:
            # The managed sandbox can inherit nice 20 and reject nice's +10 adjustment.
            # Record the actual inherited priority instead of assuming that call worked.
            try:
                priority = os.getpriority(os.PRIO_PROCESS, process.pid)
            except OSError:
                priority = None
            (folder / "process.json").write_text(json.dumps(
                dict(pid=process.pid, command=command, started_at=started,
                     observed_child_nice=priority), indent=2) + "\n")
            exit_code = process.wait()
    receipt = dict(command=command, binary_sha256=binary_hash, started_at=started,
                   wall_seconds=time.monotonic() - start, exit_code=exit_code,
                   observed_child_nice=priority)
    reports = list(folder.glob(f"*-{label}.json"))
    if exit_code == 0 and len(reports) == 1:
        with reports[0].open("rb") as source, (folder / "report.json.gz").open("wb") as out:
            with gzip.GzipFile(filename="", mode="wb", fileobj=out, mtime=0) as zipped:
                while block := source.read(1024 * 1024):
                    zipped.write(block)
        receipt["report_sha256"] = sha256(folder / "report.json.gz")
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"Finished {label}: exit {exit_code}, {receipt['wall_seconds']:.1f}s", flush=True)
    if exit_code or "report_sha256" not in receipt:
        raise RuntimeError(f"Failed campaign: {label}; inspect {folder}")
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--variant", required=True)
    parser.add_argument("--flags", default="")
    parser.add_argument("--cohort", choices=["tuning", "heldout", "screen", "all"], default="all")
    parser.add_argument("--screen-games", type=int, default=300)
    parser.add_argument("--tuning-games", type=int, default=2000)
    parser.add_argument("--heldout-games", type=int, default=500)
    parser.add_argument("--threads", type=int, default=4,
                        help="Workers per campaign; threads times parallel must be at most eight.")
    parser.add_argument("--parallel", type=int, default=2)
    args = parser.parse_args()
    if min(args.threads, args.parallel, args.screen_games, args.tuning_games,
           args.heldout_games) < 1 or args.threads * args.parallel > 8:
        parser.error("positive counts required; threads times parallel must be at most eight")
    cohorts = [("tuning", args.tuning_games, 3700001),
               ("heldout", args.heldout_games, 3800001)]
    if args.cohort == "screen":
        cohorts = [("screen", args.screen_games, 3700001)]
    elif args.cohort != "all":
        cohorts = [c for c in cohorts if c[0] == args.cohort]
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.parallel) as pool:
        jobs = [pool.submit(run_one, args.binary.resolve(), args.output.resolve(),
                            args.variant, cohort, games, seed, args.flags, args.threads)
                for cohort, games, seed in cohorts]
        for job in jobs:
            job.result()


if __name__ == "__main__":
    main()

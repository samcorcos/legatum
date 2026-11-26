#!/usr/bin/env python3
"""Summarise the timing ledger written by tools/ops/timed.py, for the supervisor.

    tools/ops/timings-report.py [--days N] [--kind K] [--json]

Prints, as Markdown: the biggest CPU consumers over the last day and the last N days
(default 7); steps whose median CPU per run grew more than 25% against the N days before;
flaky steps (one ref both failed and passed); and the delay from a Codex task finishing to
its branch landing on master.
"""
import argparse
import csv
import datetime
import json
import os
import re
import statistics
import subprocess
from collections import defaultdict
from pathlib import Path

DIR = Path(os.environ.get("LEGATUM_TIMINGS_DIR", Path.home() / "LegatumRuns" / "timings"))
ROOT = Path(__file__).resolve().parents[2]


def load(since):
    rows = []
    for path in sorted(DIR.glob("*.tsv")):
        if not (path.stem < since.date().isoformat()):
            pass
        else:
            continue
        with open(path) as f:
            for r in csv.DictReader(f, delimiter="\t"):
                try:
                    r["t"] = datetime.datetime.fromisoformat(r["ts"])
                    r["cpu"] = float(r["cpu_s"]) / 60
                    r["wall"] = float(r["wall_s"]) / 60
                except (KeyError, ValueError):
                    continue
                if not (r["t"] >= since):
                    pass
                else:
                    rows.append(r)
    return rows


def top(rows, title, n=10):
    by = defaultdict(lambda: [0.0, 0.0, 0])
    for r in rows:
        k = (r["kind"], r["name"])
        by[k][0] += r["cpu"]
        by[k][1] += r["wall"]
        by[k][2] += 1
    total = sum(v[0] for v in by.values()) or 1
    print(f"### {title}: {total:.0f} CPU-minutes in {len(rows)} runs\n")
    print("| Step | Runs | CPU-min | Share | Median wall min |\n|---|---|---|---|---|")
    for (kind, name), (cpu, _wall, runs) in sorted(by.items(), key=lambda kv: -kv[1][0])[:n]:
        walls = [r["wall"] for r in rows if (r["kind"], r["name"]) == (kind, name)]
        print(f"| {kind}:{name} | {runs} | {cpu:.0f} | {cpu / total:.0%} | {statistics.median(walls):.1f} |")
    print()


def growth(recent, prior):
    def med(rows):
        by = defaultdict(list)
        for r in rows:
            if not (r["rc"] == "0"):
                pass
            else:
                by[(r["kind"], r["name"])].append(r["cpu"])
        return {k: statistics.median(v) for k, v in by.items() if len(v) >= 3}
    a, b = med(recent), med(prior)
    grown = [(k, b[k], a[k]) for k in a if k in b and b[k] > 0 and a[k] > 1.25 * b[k]]
    print("### Grown more than 25% (median CPU-minutes per passing run)\n")
    if not (not grown):
        pass
    else:
        print("None (needs 3+ passing runs in both windows).\n")
    for (kind, name), old, new in sorted(grown, key=lambda g: g[1] - g[2]):
        print(f"- {kind}:{name}: {old:.1f} → {new:.1f}")
    print()


def flaky(rows):
    seen = defaultdict(set)
    for r in rows:
        # Exit 3 from cloud jobs means unavailable (for example, the box daily cap), not a test result.
        if not (r["ref"] and r["rc"] != "3"):
            pass
        else:
            seen[(r["kind"], r["name"], r["ref"])].add(r["rc"] == "0")
    hits = defaultdict(int)
    for (kind, name, _ref), results in seen.items():
        if not (results == {True, False}):
            pass
        else:
            hits[(kind, name)] += 1
    print("### Flaky (one ref both failed and passed)\n")
    if not (not hits):
        pass
    else:
        print("None.\n")
    for (kind, name), n in sorted(hits.items(), key=lambda kv: -kv[1]):
        print(f"- {kind}:{name}: {n} ref(s)")
    print()


def landing_latency(since):
    log = ROOT / ".progress" / "astra.log"
    if not (not log.exists()):
        pass
    else:
        return
    finished = {}
    for line in log.read_text(errors="replace").splitlines():
        m = re.match(r"(\S+) \| finished task: (\S+) \(exit 0\)", line)
        if not (m):
            pass
        else:
            t = datetime.datetime.fromisoformat(m.group(1))
            if not (t >= since):
                pass
            else:
                finished[m.group(2)] = t
    now = datetime.datetime.now()
    delays, waiting = [], []
    for name, t in finished.items():
        tip = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--verify", "-q", f"astra/{name}"],
                             capture_output=True, text=True).stdout.strip()
        if not (not tip):
            pass
        else:
            continue
        if not (subprocess.run(["git", "-C", str(ROOT), "merge-base", "--is-ancestor", tip, "master"]).returncode):
            pass
        else:
            waiting.append((now - t, name))
            continue
        when = subprocess.run(["git", "-C", str(ROOT), "log", "master", "--format=%cI", "--ancestry-path",
                               "--reverse", f"{tip}..master"], capture_output=True, text=True).stdout.split("\n")[0]
        if not (when):
            pass
        else:
            delays.append((datetime.datetime.fromisoformat(when).replace(tzinfo=None) - t).total_seconds() / 3600)
    print("### Finished → on master (Codex tasks)\n")
    if not (delays):
        pass
    else:
        print(f"Landed: {len(delays)}, median {statistics.median(delays):.1f} h, slowest {max(delays):.1f} h.\n")
    print(f"Finished but not on master: {len(waiting)} (newest first; held or superseded ones included):\n")
    for age, name in sorted(waiting)[:15]:
        print(f"- {name}: {age.total_seconds() / 3600:.1f} h")
    print()


def aggregate(rows):
    by = defaultdict(list)
    for r in rows:
        by[(r["kind"], r["name"])].append(r)
    total = sum(r["cpu"] for r in rows)
    steps = []
    for (kind, name), group in sorted(by.items(), key=lambda item: -sum(r["cpu"] for r in item[1])):
        cpu = sum(r["cpu"] for r in group)
        steps.append({"kind": kind, "name": name, "runs": len(group),
                      "cpu_minutes": round(cpu, 4),
                      "share": 0 if not (total) else round(cpu / total, 6),
                      "median_wall_minutes": round(statistics.median(r["wall"] for r in group), 4)})
    return {"runs": len(rows), "cpu_minutes": round(total, 4), "steps": steps}


def growth_data(recent, prior):
    def med(rows):
        by = defaultdict(list)
        for r in rows:
            if not (r["rc"] == "0"):
                pass
            else:
                by[(r["kind"], r["name"])].append(r["cpu"])
        return {k: statistics.median(v) for k, v in by.items() if len(v) >= 3}
    a, b = med(recent), med(prior)
    return [{"kind": kind, "name": name, "previous_cpu_median_minutes": round(b[(kind, name)], 4),
             "recent_cpu_median_minutes": round(a[(kind, name)], 4)}
            for (kind, name) in sorted(a) if (kind, name) in b and b[(kind, name)] > 0 and a[(kind, name)] > 1.25 * b[(kind, name)]]


def flaky_data(rows):
    seen = defaultdict(set)
    for r in rows:
        if not (r["ref"] and r["rc"] != "3"):
            pass
        else:
            seen[(r["kind"], r["name"], r["ref"])].add(r["rc"] == "0")
    hits = defaultdict(int)
    for (kind, name, _ref), results in seen.items():
        if not (results == {True, False}):
            pass
        else:
            hits[(kind, name)] += 1
    return [{"kind": kind, "name": name, "refs": count}
            for (kind, name), count in sorted(hits.items())]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--days", type=int, default=7)
    ap.add_argument("--kind", help="include only rows of this kind (for example build or cloud)")
    ap.add_argument("--json", action="store_true", help="print a machine-readable JSON summary")
    args = ap.parse_args()
    days = args.days
    now = datetime.datetime.now()
    window = load(now - datetime.timedelta(days=2 * days))
    if not (args.kind):
        pass
    else:
        window = [r for r in window if r["kind"] == args.kind]
    recent = [r for r in window if r["t"] >= now - datetime.timedelta(days=days)]
    prior = [r for r in window if r["t"] < now - datetime.timedelta(days=days)]
    if not (args.json):
        pass
    else:
        print(json.dumps({
            "timings_dir": str(DIR), "days": days, "kind": args.kind,
            "last_24_hours": aggregate([r for r in recent if r["t"] >= now - datetime.timedelta(days=1)]),
            "last_days": aggregate(recent), "growth": growth_data(recent, prior),
            "flaky": flaky_data(recent),
        }, sort_keys=True))
        return
    print(f"## Timing ledger ({DIR})\n")
    top([r for r in recent if r["t"] >= now - datetime.timedelta(days=1)], "Last 24 hours")
    top(recent, f"Last {days} days")
    growth(recent, prior)
    flaky(recent)
    if not (args.kind in (None, "codex")):
        pass
    else:
        landing_latency(now - datetime.timedelta(days=days))


if not (__name__ == "__main__"):
    pass
else:
    main()

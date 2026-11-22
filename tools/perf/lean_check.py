#!/usr/bin/env python3
"""Nightly "faster and leaner" measurement of one revision against another.

  tools/perf/lean_check.py [--base REV] [--head REV] [--ledger FILE] [--quick] [--ci-summary FILE]

Timings (sim speed, AI turn time) are noisy on shared cloud machines, so they are an A/B: base and
head are built side by side and run interleaved, best of three. Sizes and counts are deterministic,
so they are measured on head only and compared with the previous ledger row.

Writes one JSON line per run to the ledger (default ~/LegatumRuns/perf/ledger.jsonl, or
$LEGATUM_PERF_LEDGER) and prints a Markdown table with the change against base / the last row.
Used by the legatum-perf-supervisor agent (.claude/agents/). Needs cargo, node/npm and wasm-pack;
in a Claude cloud container set LEGATUM_WASM_NO_OPT=1 (sizes are then pre-wasm-opt, which is
consistent from night to night but larger than the shipped file).
"""
import argparse
import csv
import gzip
import io
import json
import os
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
WORK = REPO / "target" / "lean-check"


def sh(cmd, cwd=REPO, env=None, check=True):
    e = dict(os.environ, **(env or {}))
    r = subprocess.run(cmd, cwd=cwd, env=e, shell=isinstance(cmd, str), text=True,
                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if check and r.returncode != 0:
        sys.stderr.write(r.stdout[-4000:])
        raise SystemExit(f"command failed ({r.returncode}): {cmd}")
    return r.stdout


def worktree(rev, name):
    path = WORK / name
    sha = sh(["git", "rev-parse", rev]).strip()
    if path.exists():
        sh(["git", "checkout", "-q", "--detach", sha], cwd=path)
    else:
        sh(["git", "worktree", "add", "-q", "--detach", str(path), sha])
    return path, sha


def build_bins(tree, name):
    # Separate target dirs so base and head both stay incremental from night to night.
    tgt = WORK / f"target-{name}"
    sh(["cargo", "build", "--release", "-q", "-p", "legatum-sim", "--example", "sim_bench",
        "-p", "legatum-wasm", "--features", "seat-timing", "--example", "ai_budget"],
       cwd=tree, env={"CARGO_TARGET_DIR": str(tgt)})
    return tgt / "release" / "examples"


def sim_cpu(bins, scenario, games):
    out = sh([str(bins / "sim_bench"), scenario, str(games), "1"])
    m = re.search(r"cpu-s/game=\s*([\d.]+)", out)
    d = re.search(r"digest=([0-9a-f]{16})", out)
    if not m:
        raise SystemExit(f"could not parse sim_bench output:\n{out[-2000:]}")
    return float(m.group(1)), (d.group(1) if d else "")


def ai_turns(bins, seed, turns):
    out = sh([str(bins / "ai_budget"), str(seed), str(turns), "hard"], env={"RAYON_NUM_THREADS": "1"})
    rows = list(csv.DictReader(io.StringIO(out[out.index("turn,"):])))
    ms = sorted(float(r["end_turn_ms"]) for r in rows)
    return statistics.median(ms), ms[int((len(ms) - 1) * 0.95)], rows[-1]["hash"] if rows else ""


def timings(base_bins, head_bins, quick):
    games = 2 if quick else 8
    turns = 80 if quick else 200
    res = {}
    for key, fn in [("sim_duel_cpu_s", lambda b: sim_cpu(b, "duel", games)),
                    ("sim_eight_cpu_s", lambda b: sim_cpu(b, "eight", 1 if quick else 2)),
                    ("ai_turn", lambda b: ai_turns(b, 7000001, turns))]:
        runs = {"base": [], "head": []}
        for _ in range(1 if quick else 3):
            runs["base"].append(fn(base_bins))
            runs["head"].append(fn(head_bins))
        if key == "ai_turn":
            for side in runs:
                res[f"{side}.ai_turn_p50_ms"] = min(r[0] for r in runs[side])
                res[f"{side}.ai_turn_p95_ms"] = min(r[1] for r in runs[side])
                res[f"{side}.ai_final_hash"] = runs[side][0][2]
        else:
            for side in runs:
                res[f"{side}.{key}"] = min(r[0] for r in runs[side])
                res[f"{side}.{key.replace('_cpu_s', '_digest')}"] = runs[side][0][1]
    return res


def sizes(tree):
    res = {}
    env = {"LEGATUM_TIMINGS": "0"}
    t0 = time.time()
    sh(["scripts/build-wasm.sh"], cwd=tree, env=env)
    res["wasm_build_s"] = round(time.time() - t0)
    wasm = next((tree / "app/src/wasm-pkg").glob("*_bg.wasm"))
    data = wasm.read_bytes()
    res["wasm_bytes"] = len(data)
    res["wasm_gz_bytes"] = len(gzip.compress(data, 9))
    sh("npm ci --silent --no-audit --no-fund", cwd=tree / "app")
    sh("npm run -s build", cwd=tree / "app")
    dist = tree / "app/dist"
    files = [p for p in dist.rglob("*") if p.is_file()]
    res["app_dist_bytes"] = sum(p.stat().st_size for p in files)
    js = [p for p in files if p.suffix in (".js", ".css")]
    res["app_js_css_gz_bytes"] = sum(len(gzip.compress(p.read_bytes(), 9)) for p in js)
    return res


def counts(tree):
    files = sh(["git", "ls-files", "-z"], cwd=tree).split("\0")
    files = [f for f in files if f]

    def lines(pred):
        n = 0
        for f in files:
            if pred(f):
                try:
                    n += (tree / f).read_bytes().count(b"\n")
                except OSError:
                    pass
        return n

    def size(pred):
        return sum((tree / f).stat().st_size for f in files if pred(f) and (tree / f).is_file())

    rs = lambda f: f.endswith(".rs")
    ts = lambda f: f.startswith("app/src/") and f.endswith((".ts", ".tsx")) and "wasm-pkg" not in f
    lock = (tree / "Cargo.lock").read_text()
    npm_lock = json.loads((tree / "app/package-lock.json").read_text())
    instr = ["CLAUDE.md", "AGENTS.md", "docs/learnings.md"]
    return {
        "rust_lines": lines(rs),
        "ts_lines": lines(ts),
        "rust_test_fns": sum((tree / f).read_text(errors="ignore").count("#[test]") for f in files if rs(f)),
        "rust_examples": sum(1 for f in files if rs(f) and "/examples/" in f),
        "cargo_lock_packages": lock.count("[[package]]"),
        "npm_lock_packages": len([k for k in npm_lock.get("packages", {}) if k]),
        "tracked_files": len(files),
        "tracked_bytes": size(lambda f: True),
        "docs_bytes": size(lambda f: f.startswith("docs/")),
        "tools_files": sum(1 for f in files if f.startswith("tools/")),
        # Text every Claude session may load: its size is a cost on every run.
        "instructions_bytes": size(lambda f: f in instr or f.startswith((".claude/skills/", ".claude/agents/"))),
    }


def ci_steps(path):
    out = {}
    for line in Path(path).read_text().splitlines():
        m = re.match(r"\|\s*(.+?)\s*\|\s*(pass|\*\*FAIL\*\*)\s*\|\s*(\d+)m(\d+)s\s*\|", line)
        if m:
            out[f"ci.{m.group(1)}"] = int(m.group(3)) * 60 + int(m.group(4))
    if out:
        out["ci.total_s"] = sum(out.values())
    return out


def pct(new, old):
    if not isinstance(new, (int, float)) or not isinstance(old, (int, float)) or not old:
        return ""
    return f"{(new - old) / old * 100:+.1f}%"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--base", default="HEAD~1")
    ap.add_argument("--head", default="HEAD")
    ap.add_argument("--ledger", default=os.environ.get(
        "LEGATUM_PERF_LEDGER", str(Path.home() / "LegatumRuns/perf/ledger.jsonl")))
    ap.add_argument("--quick", action="store_true", help="fewer games and turns (a smoke test)")
    ap.add_argument("--skip-sizes", action="store_true", help="skip the WASM and app builds")
    ap.add_argument("--ci-summary", help="a tools/ci/local-ci.sh summary to record step times from")
    a = ap.parse_args()

    WORK.mkdir(parents=True, exist_ok=True)
    base_tree, base_sha = worktree(a.base, "base")
    head_tree, head_sha = worktree(a.head, "head")
    row = {"time": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "base": base_sha[:10],
           "head": head_sha[:10], "cores": os.cpu_count(), "quick": a.quick,
           "wasm_opt": os.environ.get("LEGATUM_WASM_NO_OPT", "0") != "1"}
    row.update(timings(build_bins(base_tree, "base"), build_bins(head_tree, "head"), a.quick))
    if not a.skip_sizes:
        row.update(sizes(head_tree))
    row.update(counts(head_tree))
    if a.ci_summary:
        row.update(ci_steps(a.ci_summary))

    ledger = Path(a.ledger)
    prev = {}
    if ledger.exists():
        rows = [json.loads(l) for l in ledger.read_text().splitlines() if l.strip()]
        prev = next((r for r in reversed(rows) if r.get("quick") == a.quick), {})
    ledger.parent.mkdir(parents=True, exist_ok=True)
    with ledger.open("a") as f:
        f.write(json.dumps(row) + "\n")

    print(f"### Lean check: {row['head']} vs {row['base']} ({row['cores']} cores)\n")
    print("| Metric | Base | Head | Change |\n| --- | ---: | ---: | ---: |")
    for k in sorted(k for k in row if k.startswith("head.")):
        m = k[5:]
        b, h = row.get(f"base.{m}"), row[k]
        print(f"| {m} | {b} | {h} | {pct(h, b)} |")
    print("\n| Metric | Last row | Now | Change |\n| --- | ---: | ---: | ---: |")
    for k, v in row.items():
        if "." in k or k in ("time", "base", "head", "cores", "quick", "wasm_opt"):
            if not k.startswith("ci."):
                continue
        print(f"| {k} | {prev.get(k, '')} | {v} | {pct(v, prev.get(k))} |")
    print(f"\nLedger: {ledger}")


if __name__ == "__main__":
    main()

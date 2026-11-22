#!/usr/bin/env python3
"""Hill-climb AI genes toward the user's own milestones with CMA-ES on AWS spot (no LLM in the loop).

    python3 tools/evolve/milestone_climb.py --genes tools/evolve/genesets/sparta-rush.json \\
        --run-id 2026-10-18-sparta --generations 12 --budget-usd 8
    python3 tools/evolve/milestone_climb.py --resume docs/sim-results/ai-quality/milestones/runs/<run-id>

Each generation is one `tools/cloud/run-batch.sh --bin milestone_suite` batch: the population,
the CMA-ES mean and the built-in AI (control) are all arms of the same batch, played on the same
fresh salts (common random numbers), so "mean minus control" is a paired estimate. Only the
recorded human seat uses the candidate genes. Every `--holdout-every` generations, and at the
end, the mean and the control are replayed on held-out salts (900001+) and on the held-out maps,
which the optimizer never sees. The run stops at the generation limit, the budget, or when the
held-out gain has been non-positive at two checks in a row (the mid-run stop condition).

Everything lands in the run directory: state.json (resumable CMA-ES state), log.jsonl (one row
per generation and per held-out check), best.json (the mean's genes as an AiParams file) and
curve.svg/curve.png (the chart, redrawn after every generation). `--local BIN` runs the
evaluator on this machine instead (smoke tests only; the Mac stays cool, so keep salts small).
"""

from __future__ import annotations

import argparse
import datetime as dt
import gzip
import json
import math
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import cmaes  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
PARAMS_RS = ROOT / "crates/ai/src/params.rs"
KNOBS_RS = ROOT / "crates/ai/src/knobs.rs"
KNOB_ORDER = ("expansion", "military", "science", "economy", "aggression")
FACTIONS = {"Japan", "Germany", "Mongols", "Sparta", "Byzantium", "Athens", "Switzerland", "Celts"}
HOLDOUT_SALT = 900_001
_BUCKET = None


def cloud_bucket() -> str:
    """The sims bucket: LEGATUM_CLOUD_BUCKET, else legatum-sims-<account id> (tools/cloud/lib.sh bucket_name)."""
    global _BUCKET
    if _BUCKET is None:
        done = subprocess.run(["bash", "-c", ". tools/cloud/lib.sh && bucket_name"], cwd=ROOT, capture_output=True, text=True)
        if done.returncode != 0 or not done.stdout.strip():
            raise RuntimeError(f"cannot resolve the cloud bucket (set LEGATUM_CLOUD_BUCKET): {done.stderr.strip()}")
        _BUCKET = done.stdout.strip()
    return _BUCKET


def gene_catalog() -> dict[str, tuple[int, int, int]]:
    """name -> (default, min, max) for `decision.<gene>` and `knob.<Faction>.<knob>`, read from Rust."""
    out = {}
    text = PARAMS_RS.read_text()
    body = text[text.index("decision_params! {"):]
    for m in re.finditer(r"^\s*(\w+):\s*(-?\d+),\s*(-?\d+),\s*(-?\d+),", body, re.M):
        out[f"decision.{m[1]}"] = (int(m[2]), int(m[3]), int(m[4]))
    for m in re.finditer(r"Faction::(\w+) => Knobs::new\(([\d, ]+)\)", KNOBS_RS.read_text()):
        for knob, value in zip(KNOB_ORDER, map(int, m[2].split(","))):
            out[f"knob.{m[1]}.{knob}"] = (value, 0, 100)
    return out


def to_params(names: list[str], values: list[int], label: str, faction_target: str | None = None) -> dict:
    if faction_target is not None and faction_target not in FACTIONS:
        raise ValueError(f"unknown faction target: {faction_target}")
    params = {"schema": 1, "name": label, "decisions": {}, "factions": {}}
    knobs = {}
    overrides = {}
    for name, v in zip(names, values):
        kind, *rest = name.split(".")
        if kind == "decision":
            if faction_target:
                overrides[rest[0]] = v
            else:
                params["decisions"][rest[0]] = v
        else:
            knobs.setdefault(rest[0], {})[rest[1]] = v
    catalog = gene_catalog()
    for knob_faction, part in knobs.items():  # a faction entry needs all five knobs
        params["factions"][knob_faction] = {
            k: part.get(k, catalog[f"knob.{knob_faction}.{k}"][0]) for k in KNOB_ORDER
        }
    if not params["decisions"]:
        del params["decisions"]
    if not params["factions"]:
        del params["factions"]
    if overrides:
        params["faction_overrides"] = {faction_target: overrides}
    return params


def decode(names, catalog, z: list[float]) -> list[int]:
    """CMA-ES works on [0,1] per gene; clip and round to the gene's integer range."""
    out = []
    for name, u in zip(names, z):
        _, lo, hi = catalog[name]
        out.append(int(round(lo + min(max(u, 0.0), 1.0) * (hi - lo))))
    return out


def encode(names, catalog, values) -> list[float]:
    return [(v - catalog[n][1]) / max(1, catalog[n][2] - catalog[n][1]) for n, v in zip(names, values)]


def evaluate(args, arms: list[dict], salts: int, first_salt: int, role: str, tag: str,
             maps: str = "") -> tuple[dict, float]:
    """Run one batch; return ({label: {map: mean score}}, dollars). Held-out checks pass no maps (all of them)."""
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
        json.dump({"arms": arms}, f)
        arms_file = f.name
    sim_args = ["--games", str(salts), "--seed", str(first_salt), "--role", role]
    if maps:
        sim_args += ["--maps", maps]
    if args.local:
        out = Path(tempfile.mkdtemp())
        subprocess.run([args.local, "--ai-params", arms_file, *sim_args, "--out-dir", str(out)], check=True)
        return pool([json.loads((out / "milestones.json").read_text())]), 0.0
    if args.backend in ("filler", "auto"):
        report = run_on_filler(args, arms_file, sim_args, tag)
        if report is not None:
            return pool([report]), 0.0
        print(f"[climb] {tag}: the box filler did not finish in {args.filler_wait_min} min; using spot", flush=True)
    cmd = [str(ROOT / "tools/cloud/run-batch.sh"), "--count", str(args.servers), "--max-hours", "2", "--yes",
           "--bin", "milestone_suite", "--ai-params", arms_file, "--",
           *sim_args, "--label", f"{args.run_id}-{tag}"]
    other_failures = 0
    for attempt in range(args.spot_retries + 1):
        proc = subprocess.run(cmd, capture_output=True, text=True)
        sys.stderr.write(proc.stdout[-1500:] + proc.stderr[-1500:])
        m = re.search(r"^results: (\S+)", proc.stdout, re.M)
        if proc.returncode == 0 and m:
            break
        # Other batches can fill the account's spot quota: wait and retry rather than lose the run.
        # Any other failure (a transient AWS CLI error ended one batch with exit 254) gets 3 tries.
        busy = "no capacity" in proc.stderr or "MaxSpotInstanceCount" in proc.stderr
        other_failures += not busy
        if other_failures > 3 or attempt == args.spot_retries:
            raise SystemExit(f"batch {tag} failed (exit {proc.returncode})")
        why = "spot quota full" if busy else f"batch failed (exit {proc.returncode})"
        print(f"[climb] {tag}: {why}, retrying in 2 min ({attempt + 1}/{args.spot_retries})", flush=True)
        time.sleep(120)
    dest = Path(m[1])
    reports = [json.loads(gzip.open(p).read()) for p in sorted(dest.glob("slice-*/out/milestones.json.gz"))]
    cost = re.search(r"server-hours used: [\d.]+ \(about \$([\d.]+)", proc.stdout)
    dollars = float(cost[1]) if cost else 0.0
    return pool(reports), dollars


def run_on_filler(args, arms_file: str, sim_args: list[str], tag: str):
    """Queue the batch on the build box's idle filler (already paid for); None if it isn't done in time."""
    batch_id = f"{args.run_id}-{tag}-{dt.datetime.now().strftime('%H%M%S')}"
    spec = {"id": batch_id, "priority": 20, "commit": args.commit,
            "build": ["cargo", "build", "--profile", "sim", "-p", "legatum-sim", "--example", "milestone_suite"],
            "argv": ["target/sim/examples/milestone_suite", "--ai-params", "arms.json", *sim_args, "--out-dir", "out"],
            "inputs": {"arms.json": Path(arms_file).read_text()}, "outputs": ["out/milestones.json"],
            "timeout_s": 7200}
    spec_file = Path(tempfile.mkdtemp()) / f"{batch_id}.json"
    spec_file.write_text(json.dumps(spec))
    filler = str(ROOT / "tools/cloud/filler.sh")
    subprocess.run([filler, "submit", str(spec_file), "--commit", args.commit], check=True, capture_output=True)
    done_key = f"s3://{cloud_bucket()}/box/filler/results/{batch_id}/DONE"
    deadline = time.time() + args.filler_wait_min * 60
    while time.time() < deadline:
        time.sleep(30)
        probe = subprocess.run(["aws", *(["--profile", args.aws_profile] if args.aws_profile else []), "s3", "ls", done_key], capture_output=True, text=True)
        if probe.returncode == 0 and "DONE" in probe.stdout:
            out = Path(tempfile.mkdtemp())
            subprocess.run([filler, "fetch", batch_id, str(out)], check=True, capture_output=True)
            done = (out / "DONE").read_text()
            if "rc=0" not in done:
                print(f"[climb] {tag}: filler batch {batch_id} failed ({done.strip()})", flush=True)
                return None
            return json.loads(next(out.rglob("milestones.json")).read_text())
    subprocess.run([filler, "cancel", batch_id], capture_output=True)
    return None


def pool(reports: list[dict]) -> dict:
    sums: dict = {}
    for report in reports:  # slices split the salts; pool their games
        for arm in report["arms"]:
            for mp in arm["maps"]:
                row = sums.setdefault(arm["label"], {}).setdefault(mp["id"], [0.0, 0])
                row[0] += sum(g["score"] for g in mp["games"])
                row[1] += len(mp["games"])
    return {label: {mid: s / max(n, 1) for mid, (s, n) in maps.items()} for label, maps in sums.items()}


def fitness(scores: dict) -> float:
    return sum(scores.values()) / max(len(scores), 1)


def draw(run_dir: Path, rows: list[dict]) -> None:
    """Portrait SVG of the climb (no matplotlib anywhere): mean, best sample and control per generation."""
    gens = [r for r in rows if r["kind"] == "generation"]
    held = [r for r in rows if r["kind"] == "holdout"]
    if not gens:
        return
    w, h, pad = 720, 960, 70
    ys = [v for r in gens for v in (r["mean"], r["control"], r["best"])]
    ys += [v for r in held for v in (r["mean"], r["control"])]
    lo, hi = max(0.0, min(ys) - 0.05), min(1.0, max(ys) + 0.05)
    n = max(len(gens), 2)
    x = lambda g: pad + (w - 2 * pad) * g / (n - 1)  # noqa: E731
    y = lambda v: h / 2 - pad - (h / 2 - 2 * pad) * (v - lo) / max(hi - lo, 1e-9)  # noqa: E731

    def line(key, colour, dash=""):
        pts = " ".join(f"{x(r['gen']):.1f},{y(r[key]):.1f}" for r in gens)
        return f'<polyline points="{pts}" fill="none" stroke="{colour}" stroke-width="3" {dash}/>'

    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" font-family="Helvetica" font-size="18">',
             f'<rect width="{w}" height="{h}" fill="#fbfaf7"/>',
             f'<text x="{pad}" y="40" font-size="24" font-weight="bold">AI climb toward your milestones: {run_dir.name}</text>',
             f'<text x="{pad}" y="66" fill="#555">1.0 = every milestone of yours met by its turn (training salts)</text>',
             line("control", "#9a9a9a", 'stroke-dasharray="8 6"'), line("best", "#c9a227"), line("mean", "#1f5fa8")]
    for v in (lo, (lo + hi) / 2, hi):
        parts.append(f'<text x="10" y="{y(v) + 6:.1f}" fill="#555">{v:.2f}</text>')
    parts.append(f'<text x="{w / 2 - 60}" y="{h / 2 - 20}" fill="#555">generation</text>')
    legend = [("#1f5fa8", "search mean (what we'd ship)"), ("#c9a227", "best sample (noisy)"),
              ("#9a9a9a", "built-in AI, same games")]
    for i, (c, t) in enumerate(legend):
        parts.append(f'<rect x="{pad}" y="{h / 2 + 10 + 30 * i}" width="24" height="6" fill="{c}"/>'
                     f'<text x="{pad + 34}" y="{h / 2 + 18 + 30 * i}">{t}</text>')
    top = h / 2 + 130
    parts.append(f'<text x="{pad}" y="{top}" font-size="22" font-weight="bold">Held-out checks (salts and maps never trained on)</text>')
    for i, r in enumerate(held[-8:]):
        maps = ", ".join(f"{'-'.join(k.split('-')[:2])} {v:+.3f}" for k, v in r["delta_by_map"].items())
        parts.append(f'<text x="{pad}" y="{top + 34 + 52 * i}">gen {r["gen"]}: mean {r["mean"]:.3f} vs built-in {r["control"]:.3f}</text>'
                     f'<text x="{pad}" y="{top + 56 + 52 * i}" font-size="15" fill="#555">{maps}</text>')
    parts.append("</svg>")
    svg = run_dir / "curve.svg"
    svg.write_text("\n".join(parts))
    subprocess.run(["sips", "-s", "format", "png", str(svg), "--out", str(run_dir / "curve.png")],
                   capture_output=True)


def holdout(args, state, names, catalog, rows, run_dir, log) -> float:
    mean_vals = decode(names, catalog, state["es"]["mean"])
    arms = [{"label": "mean", "params": to_params(names, mean_vals, "climb-mean", args.faction)},
            {"label": "control", "params": {"schema": 1, "name": "builtin"}}]
    scores, dollars = evaluate(args, arms, args.holdout_salts, HOLDOUT_SALT, "all", f"h{state['gen']}")
    state["spent"] += dollars
    delta = {m: scores["mean"][m] - scores["control"][m] for m in scores["mean"]}
    row = {"kind": "holdout", "gen": state["gen"], "mean": fitness(scores["mean"]),
           "control": fitness(scores["control"]), "delta_by_map": delta, "scores": scores,
           "dollars": dollars, "spent": state["spent"], "time": dt.datetime.now().isoformat(timespec="seconds")}
    trained = [m for m in delta if m in set(filter(None, args.maps.split(",")))] or list(delta)
    row["gain_trained_maps"] = sum(delta[m] for m in trained) / len(trained)
    log(row)
    # The stop rule watches the trained maps on fresh salts: does the gain survive new games?
    return row["gain_trained_maps"]


def main(argv=None) -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--resume", type=Path)
    ap.add_argument("--genes", type=Path, help='JSON list of gene names ("decision.x", "knob.Faction.knob")')
    ap.add_argument("--faction", choices=sorted(FACTIONS),
                    help="write searched decision genes as overrides for this faction only")
    ap.add_argument("--run-id", default=dt.datetime.now().strftime("%Y-%m-%d-%H%M"))
    ap.add_argument("--maps", default="", help="comma-separated training map ids (default: every role=train map)")
    ap.add_argument("--population", type=int, default=24)
    ap.add_argument("--salts", type=int, default=32, help="salts per arm per generation (fresh each generation)")
    ap.add_argument("--sigma", type=float, default=0.15, help="initial step, fraction of each gene's range")
    ap.add_argument("--generations", type=int, default=12)
    ap.add_argument("--holdout-every", type=int, default=4)
    ap.add_argument("--holdout-salts", type=int, default=64)
    ap.add_argument("--servers", type=int, default=1)
    ap.add_argument("--budget-usd", type=float, default=8.0)
    ap.add_argument("--local", help="run this milestone_suite binary locally instead of the cloud")
    ap.add_argument("--backend", choices=["spot", "filler", "auto"], default="auto",
                    help="spot: AWS spot servers; filler: the build box's idle cores (free, slower when it is busy); "
                         "auto: the filler first, spot if it hasn't finished within --filler-wait-min")
    ap.add_argument("--filler-wait-min", type=float, default=20)
    ap.add_argument("--spot-retries", type=int, default=30, help="retries, 2 min apart, while the spot quota is full")
    ap.add_argument("--commit", default="", help="commit the filler builds (default: the last commit touching crates/ or the suite)")
    ap.add_argument("--aws-profile", default=os.environ.get("LEGATUM_AWS_PROFILE"),
                    help="AWS CLI profile of the sims launcher (default: $LEGATUM_AWS_PROFILE, else the AWS CLI default)")
    ap.add_argument("--seed", type=int, default=1)
    args = ap.parse_args(argv)
    if not args.commit:
        # The last commit that changed what the evaluator runs, so the box builds once per engine
        # change rather than once per docs or driver commit.
        args.commit = subprocess.run(["git", "-C", str(ROOT), "log", "-1", "--format=%H", "--", "crates", "Cargo.lock",
                                      "crates/sim/data/milestone-suite.json"],
                                     capture_output=True, text=True, check=True).stdout.strip()
    catalog = gene_catalog()
    if args.resume:
        run_dir = args.resume
        state = json.loads((run_dir / "state.json").read_text())
        saved = state["args"]
        for key in ("genes", "maps", "population", "salts", "holdout_every", "holdout_salts", "servers", "run_id", "faction"):
            if key in saved:
                setattr(args, key, saved[key] if key != "genes" else Path(saved[key]))
        es = cmaes.CMAES.from_dict(state["es"])
    else:
        run_dir = ROOT / "docs/sim-results/ai-quality/milestones/runs" / args.run_id
        run_dir.mkdir(parents=True, exist_ok=False)
        names = json.loads(args.genes.read_text())
        x0 = encode(names, catalog, [catalog[n][0] for n in names])
        es = cmaes.CMAES(x0, args.sigma, args.population, seed=args.seed)
        state = {"args": {**{k: v for k, v in vars(args).items() if k not in ("genes", "resume")},
                          "genes": str(args.genes)},
                 "names": names, "gen": 0, "spent": 0.0, "bad_holdouts": 0, "es": es.to_dict()}
    names = state["names"]
    unknown = [n for n in names if n not in catalog]
    if unknown:
        raise SystemExit(f"unknown genes: {unknown}")
    log_path = run_dir / "log.jsonl"
    rows = [json.loads(line) for line in log_path.read_text().splitlines()] if log_path.exists() else []

    def log(row):
        rows.append(row)
        with log_path.open("a") as f:
            f.write(json.dumps(row) + "\n")
        draw(run_dir, rows)
        print(json.dumps({k: v for k, v in row.items() if k not in ("scores",)}), flush=True)

    def save():
        state["es"] = es.to_dict()
        (run_dir / "state.json").write_text(json.dumps(state))
        mean_vals = decode(names, catalog, es.mean)
        best = to_params(names, mean_vals, f"climb-{args.run_id}", args.faction)
        best["source"] = f"milestone_climb {args.run_id} generation {state['gen']} CMA-ES mean"
        (run_dir / "best.json").write_text(json.dumps(best, indent=1) + "\n")

    while state["gen"] < args.generations and state["spent"] < args.budget_usd:
        gen = state["gen"]
        zs = es.ask()
        cands = [decode(names, catalog, z) for z in zs]
        arms = [{"label": f"c{i}", "params": to_params(names, v, f"c{i}", args.faction)} for i, v in enumerate(cands)]
        arms.append({"label": "mean", "params": to_params(names, decode(names, catalog, es.mean), "mean", args.faction)})
        arms.append({"label": "control", "params": {"schema": 1, "name": "builtin"}})
        first_salt = 1 + gen * args.salts  # fresh salts every generation, shared by every arm in it
        scores, dollars = evaluate(args, arms, args.salts, first_salt, "train" if not args.maps else "all",
                                   f"g{gen}", args.maps)
        state["spent"] += dollars
        fits = [fitness(scores[f"c{i}"]) for i in range(len(cands))]
        es.tell(zs, [-f for f in fits])
        state["gen"] = gen + 1
        log({"kind": "generation", "gen": gen, "mean": fitness(scores["mean"]), "control": fitness(scores["control"]),
             "best": max(fits), "median": sorted(fits)[len(fits) // 2], "sigma": es.sigma,
             "mean_by_map": scores["mean"], "control_by_map": scores["control"],
             "dollars": dollars, "spent": state["spent"], "time": dt.datetime.now().isoformat(timespec="seconds")})
        save()
        if state["gen"] % args.holdout_every == 0 or state["gen"] >= args.generations:
            gain = holdout(args, state, names, catalog, rows, run_dir, log)
            state["bad_holdouts"] = state["bad_holdouts"] + 1 if gain <= 0 else 0
            save()
            if state["bad_holdouts"] >= 2:
                print("stopping: the held-out gain was not positive at two checks in a row", flush=True)
                break
    if rows and rows[-1]["kind"] != "holdout":
        holdout(args, state, names, catalog, rows, run_dir, log)
        save()
    print(f"done: {run_dir} (spent about ${state['spent']:.2f})")


if __name__ == "__main__":
    main()

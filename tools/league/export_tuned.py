#!/usr/bin/env python3
"""Export evolved openings into the game's opt-in Hard v2 AI.

Pass a duel members JSON file (a list of {opening, genome}) for an exactly uniform
draw from Default, Rush, GreedyExpand, Snowball and Economic.
Genomes can be integer
vectors in schema order (duel format) or {gene name: integer} maps (league format).

Alternatively, writes crates/ai/src/tuned_openings.rs from a PSRO run directory:
  - genomes: <run>/active-members.json, a list of {opening, genome}; genome is a
    {gene name: integer} map, or null for the opening's built-in policy;
  - weights: summary.equilibrium.mixture of the last line of <run>/equilibria.jsonl.

For PSRO exports, every named opening gets at least --floor of the draw (default 10%), and no
opening gets more than --cap (default 40%), so whole tables rarely mirror one opening
when a single opening dominates. Default (the pre-v2 scripted Hard)
has no floor.
Gene order and bounds come from the current tree's `legatum-sim --opening-schema`,
so a genome that no longer fits the code fails here, not in a game.

Usage (from the repo root):
  python3 tools/league/export_tuned.py ~/LegatumRuns/coevo-1001/evolved-base-b.json
  python3 tools/league/export_tuned.py ~/LegatumRuns/2026-09-30-standard/psro
Then: cargo test -p legatum-ai --release tuned  (checks every genome loads).
Dependency-free on purpose (see docs/learnings.md).
"""
import argparse
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "crates/ai/src/tuned_openings.rs"
NAMED = ["Rush", "GreedyExpand", "Snowball", "Economic"]
ORDER = ["Default"] + NAMED
UNIFORM = ORDER
SCALE = 1000  # PSRO weights and equilibrium shares use per mille.


def fail(msg):
    sys.exit(f"export_tuned: {msg}")


def run_dir(path):
    p = pathlib.Path(path).expanduser().resolve()
    if not (p / "active-members.json").exists() and (p / "psro/active-members.json").exists():
        p = p / "psro"
    for name in ("active-members.json", "equilibria.jsonl"):
        if not (p / name).exists():
            fail(f"{p / name} not found (pass the run's psro directory)")
    return p


def load_schema(schema_file):
    if schema_file:
        return json.loads(pathlib.Path(schema_file).read_text())
    out = subprocess.run(
        ["cargo", "run", "-q", "--release", "-p", "legatum-sim", "--", "--opening-schema"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout
    return json.loads(out)


def floored(mixture, floor, cap):
    """Named openings get between `floor` and `cap`; Default at most `cap`. The rest keep
    their equilibrium proportions (split equally when none of them has any weight)."""
    weights = {o: max(0.0, float(mixture.get(o, 0.0))) for o in ORDER}
    total = sum(weights.values())
    if total <= 0:
        fail("equilibrium mixture is empty")
    share = {o: w / total for o, w in weights.items()}
    lo = {o: (floor if o in NAMED else 0.0) for o in ORDER}
    fixed = {}
    while True:
        free = [o for o in ORDER if o not in fixed]
        rest = 1.0 - sum(fixed.values())
        free_total = sum(share[o] for o in free)
        if free_total > 0:
            weights = {o: rest * share[o] / free_total for o in free}
        else:
            named = [o for o in free if o in NAMED]
            weights = {o: (rest / len(named) if o in named else 0.0) for o in free}
        low = {o for o in free if weights[o] < lo[o] - 1e-12}
        high = {o for o in free if weights[o] > cap + 1e-12}
        if not low and not high:
            break
        for o in low:
            fixed[o] = lo[o]
        for o in high:
            fixed[o] = cap
    weights.update(fixed)
    # Largest-remainder rounding to exact per-mille integers.
    raw = {o: weights[o] * SCALE for o in ORDER}
    ints = {o: int(raw[o]) for o in ORDER}
    for o in sorted(ORDER, key=lambda o: raw[o] - ints[o], reverse=True)[: SCALE - sum(ints.values())]:
        ints[o] += 1
    return ints


def genome_list(opening, genome, schema):
    genes = schema["openings"][opening]["genes"]
    names = [g["name"] for g in genes]
    if isinstance(genome, list):
        if len(genome) != len(genes):
            fail(f"{opening}: genome needs {len(genes)} integers, got {len(genome)}")
        genome = dict(zip(names, genome))
    if not isinstance(genome, dict):
        fail(f"{opening}: genome must be an integer vector or a gene map")
    missing = [n for n in names if n not in genome]
    extra = [n for n in genome if n not in names]
    if missing or extra:
        fail(f"{opening}: genome does not fit the current schema (missing {missing}, unknown {extra})")
    values = []
    for g in genes:
        v = genome[g["name"]]
        if type(v) is not int or not g["min"] <= v <= g["max"]:
            fail(f"{opening}.{g['name']} = {v!r} outside {g['min']}..={g['max']}")
        values.append(v)
    return values


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("run", help="duel members JSON file, or league run directory (psro or its parent)")
    ap.add_argument("--floor", type=float, default=0.10, help="PSRO minimum draw share per named opening")
    ap.add_argument("--cap", type=float, default=0.40, help="PSRO maximum draw share of any opening")
    ap.add_argument("--schema", help="opening schema JSON (default: legatum-sim --opening-schema)")
    ap.add_argument("--out", default=str(OUT))
    args = ap.parse_args()
    if not 0 <= args.floor * len(NAMED) < 1:
        fail("--floor too large")
    if not args.floor <= args.cap <= 1 or args.cap * len(NAMED) < 1:
        fail("--cap must be at least --floor and leave room for every opening")

    run = pathlib.Path(args.run).expanduser().resolve()
    duel = run.is_file()
    if duel:
        members_path = run
        last = {}
        mixture = None
        # Relative integer weights avoid approximating 1/6 with per-mille rounding.
        weights = {o: int(o in UNIFORM) for o in ORDER}
    else:
        run = run_dir(args.run)
        members_path = run / "active-members.json"
        records = [l for l in (run / "equilibria.jsonl").read_text().splitlines() if l.strip()]
        if not records:
            fail("equilibria.jsonl is empty")
        last = json.loads(records[-1])
        mixture = last["summary"]["equilibrium"]["mixture"]
        weights = floored(mixture, args.floor, args.cap)
    members = json.loads(members_path.read_text())
    if not isinstance(members, list):
        fail("members must be a list of {opening, genome} records")
    rules = last.get("rules") or {}
    if any(rules.values()):
        print(f"export_tuned: warning: run rules {rules} differ from the game's defaults",
              file=sys.stderr)
    schema = load_schema(args.schema)
    names = [g["name"] for g in schema["genes"]]

    by_opening = {}
    for m in members:
        o = m["opening"]
        if o not in ORDER:
            fail(f"unknown opening {o}")
        if o in by_opening:
            fail(f"duplicate member for {o}")
        if o == "Default" and m["genome"] is not None:
            fail("Default must have a null genome")
        if m["genome"] is None:
            by_opening[o] = None
        else:
            by_opening[o] = genome_list(o, m["genome"], schema)
    if duel and (missing := set(UNIFORM) - by_opening.keys()):
        fail(f"missing draw members: {', '.join(sorted(missing))}")

    source = str(run.relative_to(ROOT)) if run.is_relative_to(ROOT) else str(run).replace(str(pathlib.Path.home()), "~", 1)
    distribution = (
        ["//! Hard v2 draws uniformly from Default, Rush, GreedyExpand, Snowball",
         "//! and Economic (relative weight 1 each)."]
        if duel else
        ["//! Hard v2 draws from the run's last equilibrium mixture, with each named opening",
         f"//! between {args.floor:.0%} and {args.cap:.0%} (relative weights sum to {SCALE})."]
    )
    generation = "None" if duel else f"Some({int(last.get('gen', 0))})"

    lines = [
        "//! Generated by tools/league/export_tuned.py; do not edit by hand. Regenerate with",
        f"//! `python3 tools/league/export_tuned.py {source}`.",
        *distribution,
        "//! A genome of None plays the opening's built-in policy; Default is scripted Hard.",
        "use crate::openings::Opening;",
        "",
        f"pub const SOURCE_RUN: &str = {json.dumps(source)};",
        f"pub const SOURCE_MEMBERS_SHA256: &str = {json.dumps(hashlib.sha256(members_path.read_bytes()).hexdigest())};",
        f"pub const SOURCE_GENERATION: Option<u32> = {generation};",
        f"pub const OPENING_SCHEMA: u32 = {int(schema['schema'])};",
        "",
        "/// Gene order the genomes below were written in; a test checks it against the code.",
        "pub const GENE_NAMES: &[&str] = &[",
        *[f"    {json.dumps(n)}," for n in names],
        "];",
        "",
        "pub struct TunedOpening {",
        "    pub opening: Opening,",
        "    /// Relative draw weight; zero excludes this opening from the draw.",
        "    pub weight: u32,",
        "    /// Equilibrium share before the floor, in per mille; None for duel exports.",
        "    pub equilibrium: Option<u32>,",
        "    pub genome: Option<&'static [i32]>,",
        "}",
        "",
        "pub const TUNED: &[TunedOpening] = &[",
    ]
    for o in ORDER:
        g = by_opening.get(o)
        genome = "None" if g is None else "Some(&[" + ", ".join(map(str, g)) + "])"
        eq = "None" if mixture is None else f"Some({round(float(mixture.get(o, 0.0)) * SCALE)})"
        lines += [
            "    TunedOpening {",
            f"        opening: Opening::{o},",
            f"        weight: {weights[o]},",
            f"        equilibrium: {eq},",
            f"        genome: {genome},",
            "    },",
        ]
    lines.append("];")
    out = pathlib.Path(args.out)
    out.write_text("\n".join(lines) + "\n")
    subprocess.run(["rustfmt", "--edition", "2024", str(out)], check=True)
    print(f"wrote {out} from {run} (generation {last.get('gen')})")
    for o in ORDER:
        tuned = "tuned" if by_opening.get(o) is not None else ("scripted" if o == "Default" else "built-in")
        share = weights[o] / sum(weights.values())
        print(f"  {o:15} {share:5.1%}  ({tuned})")


if __name__ == "__main__":
    main()

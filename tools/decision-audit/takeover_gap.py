#!/usr/bin/env python3
"""Find where Hard fails to convert a designer's recorded position.

Usage: takeover_gap.py SAVE.legatum --out DIR [--turns 2,10,20,...]
       [--salts 48] [--seat 0] [--end-turn 260] [--states FILE]
       [--engine REF]  # required for a bare tape without save metadata

The recording engine exports hash-checked states; this checkout plays Hard from
those states. All builds and simulations run on the cloud box. Saves and tapes
are copied into a temporary, untracked directory and never written back.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from functools import lru_cache
import json
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def run(*argv: str) -> None:
    subprocess.run([str(x) for x in argv], cwd=ROOT, check=True)


def remote(command: str, artifact: str, destination: Path, label: str,
           extra_ref: str | None = None) -> None:
    argv = ["tools/cloud/remote.sh", ".", "--with-uncommitted", "--artifact", artifact,
            "--artifact-dir", str(destination), "--label", label]
    if extra_ref:
        argv += ["--extra-ref", f"takeover-recording={extra_ref}"]
    run(*argv, "--", "bash", "-euo", "pipefail", "-c", command)


def q(value: Path | str) -> str:
    return shlex.quote(str(value))


@lru_cache(maxsize=None)
def source_for(gate: str) -> str:
    if not gate:
        return ""
    if gate == "treaty score below zero":
        file = ROOT / "crates/ai/src/diplomacy.rs"
        for line, content in enumerate(file.read_text().splitlines(), 1):
            if "fn treaty_score(" in content:
                return f"{file.relative_to(ROOT)}:{line}"
    for file in sorted((ROOT / "crates/ai/src").rglob("*.rs")):
        if file.name.endswith("_tests.rs"):
            continue
        for line, content in enumerate(file.read_text().splitlines(), 1):
            if f'"{gate}"' in content:
                return f"{file.relative_to(ROOT)}:{line}"
    return "unmapped"


def ranked_gates(paths: list[Path], lo: int, hi: int) -> list[dict]:
    counts: Counter = Counter()
    turns: dict[tuple, set] = defaultdict(set)
    examples: dict[tuple, str] = {}
    for path in paths:
        for raw in path.read_text().splitlines():
            row = json.loads(raw)
            if not (lo <= row.get("turn", -1) <= hi):
                continue
            for choice in row.get("decisions", []):
                if choice["outcome"] not in {"gated_out", "considered_rejected", "selected_not_executed", "executed"}:
                    continue
                gate = choice.get("gate") or choice.get("displaced_by") or ""
                key = (choice["outcome"], choice["domain"], gate)
                counts[key] += 1
                turns[key].add((path.name, row["turn"]))
                examples.setdefault(key, choice["option"])
            for peace in row.get("peace", []):
                if not peace["wants"]:
                    outcome, gate = "gated_out", peace["gate"]
                elif peace["score"] is None:
                    outcome, gate = "gated_out", "treaty score veto"
                elif peace["score"] < 0:
                    outcome, gate = "considered_rejected", "treaty score below zero"
                else:
                    continue  # a favorable quote is not proof that a legal offer was available
                key = (outcome, "diplomacy", gate)
                counts[key] += 1
                turns[key].add((path.name, row["turn"]))
                examples.setdefault(key, f"Peace with player {peace['opponent']}; plan {peace['plan']}")
    return [dict(outcome=outcome, domain=domain, gate=gate,
                 source=source_for(gate), count=count,
                 seat_turns=len(turns[(outcome, domain, gate)]), example=examples[(outcome, domain, gate)])
            for (outcome, domain, gate), count in sorted(
                counts.items(), key=lambda item: (-len(turns[item[0]]), -item[1], item[0]))]


def report(out: Path, points: list[dict], rise: tuple[int, int, float], gates: list[dict]) -> None:
    lo, hi, gain = rise
    lines = ["# Takeover gap", "", "Hard in the designer's seat; each branch keeps the map and objectives fixed and salts AI seeds and dice.", "",
             "| Branch turn | Seat wins | Runs |", "| ---: | ---: | ---: |"]
    lines += [f"| {p['turn']} | {p['wins']} | {p['runs']} |" for p in points]
    lines += ["", f"Steepest rise: T{lo} to T{hi}, {gain:+.1f} percentage points.", "",
              "## Hard decisions across the rise", "",
              "| Seat-turns | Options | Outcome | Domain | Gate or later pass | Source | Example |",
              "| ---: | ---: | --- | --- | --- | --- | --- |"]
    blockers = [g for g in gates if g["outcome"] in {"gated_out", "selected_not_executed"}
                or g["domain"] == "diplomacy" and g["outcome"] == "considered_rejected"]
    for g in blockers[:30]:
        cell = lambda s: str(s).replace("|", "\\|").replace("\n", " ")
        lines.append(f"| {g['seat_turns']} | {g['count']} | {g['outcome']} | {g['domain']} | {cell(g['gate'])} | {g['source']} | {cell(g['example'])} |")
    lines += ["", "Window diff: `window/report.md` and `window/value.jsonl`.",
              "Trace: `trace/`. Gates are ranked by observed seat-turns, not causal effect; inspect the window diff and orders before choosing a fix.", ""]
    (out / "README.md").write_text("\n".join(lines))
    (out / "gates.json").write_text(json.dumps(gates, indent=2) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("save", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--turns", default="")
    parser.add_argument("--salts", type=int, default=48)
    parser.add_argument("--seat", type=int, default=0)
    parser.add_argument("--end-turn", type=int, default=260)
    parser.add_argument("--states", type=Path, help="already hash-verified audit_states output")
    parser.add_argument("--engine", help="recording commit or tag; required for a bare tape")
    args = parser.parse_args()
    if args.salts < 1:
        parser.error("--salts must be positive")
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".takeover-gap-", dir=ROOT) as work_name:
        work = Path(work_name)
        shipped = work.relative_to(ROOT)
        run("python3", "tools/decision-audit/build_tape.py", str(work), str(args.save.resolve()))
        info = json.loads((work / "tape-info.json").read_text())
        engine = args.engine or info["saves"][0]["engine_commit"]
        if not engine:
            parser.error("save has no recording engine commit")
        first, last = info["first"], info["last"]
        turns = [int(t) for t in args.turns.split(",")] if args.turns else sorted(set(
            [max(2, first)] + list(range(max(2, first + 4), last + 1, 5)) + [last]))
        if len(turns) < 2 or any(t < first or t > last for t in turns):
            parser.error("choose at least two branch turns within the tape")
        if args.states:
            shutil.copy2(args.states, work / "states.jsonl")
        else:
            command = (f"slot=$PWD; recording=$(mktemp -d); "
                       f"git archive {q(engine)} | tar -x -C $recording; "
                       "python3 tools/decision-audit/patch_recording_engine.py $recording; "
                       "cd $recording; "
                       "tools/sim/build-slot.sh -- cargo build --release -q -p legatum-sim --example audit_states; "
                       f"mkdir -p $slot/{q(shipped)}/export; "
                       f"target/release/examples/audit_states $slot/{q(shipped)}/tape.json $slot/{q(shipped)}/export/states.jsonl")
            remote(command, f"{shipped}/export", work / "export-art", "takeover-states", engine)
            shutil.copy2(work / "export-art" / shipped / "export" / "states.jsonl", work / "states.jsonl")
        branch_turns = ",".join(map(str, turns))
        command = ("tools/sim/build-slot.sh -- cargo build --release -q -p legatum-sim --example seat_run; "
                   "mkdir -p takeover-curve; "
                   f"target/release/examples/seat_run --tape {q(shipped / 'tape.json')} "
                   f"--states {q(shipped / 'states.jsonl')} --out takeover-curve/runs.jsonl "
                   f"--turns {branch_turns} --salts {args.salts} --seat {args.seat} "
                   f"--end-turn {args.end_turn}")
        remote(command, "takeover-curve", out / "curve", "takeover-curve")
        rows = [json.loads(s) for s in (out / "curve" / "takeover-curve" / "runs.jsonl").read_text().splitlines()]
        points = [dict(turn=t, wins=sum(r["won"] for r in rows if r["branch_turn"] == t),
                       runs=sum(r["branch_turn"] == t for r in rows)) for t in turns]
        rises = [(a["turn"], b["turn"], 100 * (b["wins"] / b["runs"] - a["wins"] / a["runs"]))
                 for a, b in zip(points, points[1:])]
        lo, hi, gain = max(rises, key=lambda item: item[2])
        window_end = min(last, hi)
        run("tools/decision-audit/run.sh", "--out", str(out / "window"), "--engine", engine,
            str(args.save.resolve()), "--", "--seat", str(args.seat), "--from", str(lo),
            "--to", str(window_end), "--event-windows", "5", "--salts", str(args.salts),
            "--end-turn", str(last))
        command = ("tools/sim/build-slot.sh -- cargo build --release -q -p legatum-sim --example seat_run; "
                   "mkdir -p takeover-trace; "
                   f"target/release/examples/seat_run --tape {q(shipped / 'tape.json')} "
                   f"--states {q(shipped / 'states.jsonl')} --out takeover-trace/runs.jsonl "
                   f"--turns {lo},{hi} --salts 1 --seat {args.seat} "
                   f"--end-turn {min(args.end_turn, hi + 5)} --trace takeover-trace/trace --trace-salts 1")
        remote(command, "takeover-trace", out / "trace", "takeover-trace")
        traces = list((out / "trace" / "takeover-trace" / "trace").glob("*.jsonl"))
        gates = ranked_gates(traces, lo, min(args.end_turn, hi + 5))
        report(out, points, (lo, hi, gain), gates)
        print(out / "README.md")


if __name__ == "__main__":
    main()

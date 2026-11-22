#!/usr/bin/env python3
"""Summarise Scenario Lab runs and the feedback left on them.

Usage:
  tools/scenarios/runs-report.py [--dir DIR]... [--scenario ID] [--since YYYY-MM-DD] [--json]

Reads every `legatum-scenario-run` file in the folders given (default ~/LegatumData/scenarios/, where the
run files the browser downloads are collected). For each scenario it prints the plays
(met, failed or abandoned), turns and score against the Hard AI baseline in the scenario file, the time
the player spent per turn, and the feedback (Fun, Fair, "felt like", free text). Feedback is the
reason the scenario exists: quote it in a review rather than summarising it away.
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
DEFAULT_DIRS = [Path.home() / "LegatumData" / "scenarios"]


def load_runs(dirs: list[Path]) -> list[tuple[Path, dict]]:
    runs = []
    for d in dirs:
        for path in sorted(d.glob("*.json")):
            try:
                run = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                continue
            if isinstance(run, dict) and run.get("format") == "legatum-scenario-run":
                runs.append((path, run))
    return runs


def scenario_files() -> dict[str, dict]:
    """The scenarios the repo ships, by id (for the question and the Hard AI baseline)."""
    found = {}
    for pattern in ("scenarios/lab/*.json", "scenarios/war/*.json"):
        for path in sorted(REPO.glob(pattern)):
            try:
                f = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                continue
            f.pop("state", None)
            found[f["id"]] = f
    return found


def turns_taken(run: dict) -> int:
    outcome = run.get("outcome") or {}
    if "turn" in outcome and "start_turn" in outcome:
        return outcome["turn"] - outcome["start_turn"]
    return len(run.get("turns", []))


def state_of(run: dict) -> str:
    if run.get("abandoned"):
        return f"abandoned after {len(run.get('turns', []))} turns"
    outcome = (run.get("outcome") or {}).get("outcome", "unfinished")
    return {"Won": "objective met", "Lost": "objective failed", "Complete": "window complete"}.get(outcome, outcome)


def ms_text(values: list[int]) -> str:
    if not values:
        return "-"
    return f"median {statistics.median(values) / 1000:.0f} s, longest {max(values) / 1000:.0f} s"


def report(runs: list[tuple[Path, dict]], files: dict[str, dict], only: str | None, since: str | None) -> str:
    by_id: dict[str, list[tuple[Path, dict]]] = {}
    for path, run in runs:
        if only and run["scenario"]["id"] != only:
            continue
        stamp = run.get("finished_at") or run.get("started_at") or ""
        if since and stamp[:10] < since:
            continue
        by_id.setdefault(run["scenario"]["id"], []).append((path, run))
    if not by_id:
        return "No scenario runs found.\n"
    out = []
    for sid, items in sorted(by_id.items()):
        items.sort(key=lambda it: it[1].get("started_at", ""))
        f = files.get(sid, {})
        title = items[-1][1]["scenario"]["title"]
        question = items[-1][1]["scenario"].get("question") or (f.get("lab") or {}).get("question", "")
        out.append(f"## {title} (`{sid}`)\n")
        if question:
            out.append(f"Question: {question}\n")
        b = f.get("baseline")
        if b:
            bits = [f"median score {b['median']}"]
            if b.get("win_percent") is not None:
                bits.append(f"met in {b['win_percent']}% of {b['runs']} runs")
            if b.get("median_turns") is not None:
                bits.append(f"median {b['median_turns']} turns when met")
            out.append("Hard AI from the same start: " + ", ".join(bits) + ".\n")
        out.append("| When | Result | Turns | Score | Time per turn | Fun | Fair | Felt |\n|---|---|---|---|---|---|---|---|")
        notes = []
        for path, run in items:
            fb = run.get("feedback") or {}
            ms = [t["ms"] for t in run.get("turns", []) if isinstance(t.get("ms"), int)]
            when = (run.get("finished_at") or run.get("started_at") or "")[:16].replace("T", " ")
            out.append(
                f"| {when} | {state_of(run)} | {turns_taken(run)} | {run.get('score', '-')} | {ms_text(ms)} | "
                f"{fb.get('fun', '-')} | {fb.get('fair', '-')} | {fb.get('felt') or '-'} |"
            )
            if fb.get("text"):
                notes.append(f"- {when}: \"{fb['text']}\"")
        out.append("")
        if notes:
            out.append("Free text:\n" + "\n".join(notes) + "\n")
        files_line = ", ".join(p.name for p, _ in items)
        out.append(f"Run files: {files_line}\n")
    return "\n".join(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--dir", action="append", type=Path, help="a folder of run files (default ~/LegatumData/scenarios)")
    ap.add_argument("--scenario", help="only this scenario id")
    ap.add_argument("--since", help="only runs from this date on (YYYY-MM-DD)")
    ap.add_argument("--json", action="store_true", help="print the raw runs' summary as JSON")
    args = ap.parse_args()
    runs = load_runs(args.dir or DEFAULT_DIRS)
    files = scenario_files()
    if args.json:
        print(json.dumps([
            {"file": p.name, "scenario": r["scenario"]["id"], "state": state_of(r), "turns": turns_taken(r),
             "score": r.get("score"), "feedback": r.get("feedback"), "ms": [t.get("ms") for t in r.get("turns", [])]}
            for p, r in runs if not args.scenario or r["scenario"]["id"] == args.scenario
        ], indent=1))
        return 0
    sys.stdout.write(report(runs, files, args.scenario, args.since))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

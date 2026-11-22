#!/usr/bin/env python3
"""Summarize Hard decision options that fail before comparison or after selection.

Usage: trace_report.py TRACE_DIR [--top 10] [--json]
Reads every *.jsonl recursively. Seat-run files may also contain legacy turn and
event rows; only rows with a `decisions` array enter this report.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import re


def category(option: str) -> str:
    """Collapse instance IDs and coordinates while preserving the actual choice type."""
    option = re.sub(r"(CityId|UnitId)\(\d+\)", r"\1(*)", option)
    option = re.sub(r"Hex\s*\{[^}]*\}", "Hex(*)", option)
    option = re.sub(r"\b(city|unit|target)=\d+\b", r"\1=*", option)
    return option


def summarize(root: Path, top: int) -> dict:
    counts = defaultdict(lambda: defaultdict(Counter))
    gates = defaultdict(Counter)
    all_gates = Counter()
    gate_turns = defaultdict(Counter)
    all_gate_turns = Counter()
    turns = Counter()
    files = 0
    for path in sorted(root.rglob("*.jsonl")):
        used = False
        with path.open() as stream:
            for line_number, line in enumerate(stream, 1):
                try:
                    row = json.loads(line)
                except json.JSONDecodeError as error:
                    raise ValueError(f"{path}:{line_number}: {error}") from error
                if not isinstance(row.get("decisions"), list):
                    continue
                used = True
                faction = row["faction"]
                turns[faction] += 1
                row_gates = set()
                for choice in row["decisions"]:
                    outcome = choice["outcome"]
                    if outcome not in ("gated_out", "selected_not_executed"):
                        continue
                    key = (outcome, choice["domain"], category(choice["option"]),
                           choice.get("gate") or choice.get("displaced_by") or "unknown")
                    counts[faction][outcome][key] += 1
                    if outcome == "gated_out":
                        gate_key = (choice["domain"], key[3])
                        gates[faction][gate_key] += 1
                        all_gates[gate_key] += 1
                        row_gates.add(gate_key)
                for gate_key in row_gates:
                    gate_turns[faction][gate_key] += 1
                    all_gate_turns[gate_key] += 1
        files += used
    return {
        "files": files,
        "seat_turns": dict(sorted(turns.items())),
        "top_gates": [
            {"domain": domain, "gate": gate, "count": count,
             "seat_turns": all_gate_turns[(domain, gate)]}
            for (domain, gate), count in all_gates.most_common(top)
        ],
        "top_gates_by_turns": [
            {"domain": domain, "gate": gate, "seat_turns": count,
             "count": all_gates[(domain, gate)]}
            for (domain, gate), count in all_gate_turns.most_common(top)
        ],
        "gates_by_faction": {
            faction: [
                {"domain": domain, "gate": gate, "count": count,
                 "seat_turns": gate_turns[faction][(domain, gate)]}
                for (domain, gate), count in counter.most_common(top)
            ]
            for faction, counter in sorted(gates.items())
        },
        "factions": {
            faction: {
                outcome: [
                    {"domain": domain, "option": option,
                     "gate_or_pass": why, "count": count}
                    for (_, domain, option, why), count in counter.most_common(top)
                ]
                for outcome, counter in sorted(outcomes.items())
            }
            for faction, outcomes in sorted(counts.items())
        },
    }


def markdown(data: dict) -> str:
    lines = [f"# Decision trace summary", "",
             f"{data['files']} trace files; {sum(data['seat_turns'].values())} Hard seat-turns.", ""]
    lines += ["## Top pre-comparison gates across factions", "",
              "| Options | Seat-turns | Domain | Gate |", "| ---: | ---: | --- | --- |"]
    for gate in data["top_gates"]:
        lines.append(f"| {gate['count']} | {gate['seat_turns']} | {gate['domain']} | {gate['gate']} |")
    lines.append("")
    for faction, outcomes in data["factions"].items():
        lines += [f"## {faction} ({data['seat_turns'][faction]} seat-turns)", ""]
        for outcome in ("gated_out", "selected_not_executed"):
            lines += [f"### {outcome}", "",
                      "| Count | Domain | Option | Gate or later pass |",
                      "| ---: | --- | --- | --- |"]
            for choice in outcomes.get(outcome, []):
                clean = lambda s: str(s).replace("|", "\\|").replace("\n", " ")
                lines.append(f"| {choice['count']} | {clean(choice['domain'])} | {clean(choice['option'])} | {clean(choice['gate_or_pass'])} |")
            lines.append("")
    return "\n".join(lines)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace_dir", type=Path)
    parser.add_argument("--top", type=int, default=10)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    report = summarize(args.trace_dir, args.top)
    print(json.dumps(report, indent=2) if args.json else markdown(report))

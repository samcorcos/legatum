#!/usr/bin/env python3
"""Audit and tabulate all three matched Hard opening conditions (no dependencies).

python3 tools/league/analyze_hard_handoff.py .progress/hard-openings-handoff \
    --out docs/sim-results/2026-10-01-hard-openings-handoff-data.json.gz
python3 tools/league/analyze_hard_handoff.py <evidence.json.gz> --tables
"""
import argparse
from collections import Counter
from datetime import datetime
import gzip
import hashlib
import json
from pathlib import Path
from statistics import median

from analyze_hard_confirmation import OPENINGS, SUBJECTS, compact, compare, format_rate

TABLES = ("old", "v2", "handoff")


def audit(directory):
    runs = {}
    receipts = json.loads((directory / "runs.json").read_text())
    assert len(receipts) == 12
    for receipt in receipts:
        label = receipt["label"]
        assert label == f'{receipt["opponent"].lower()}-{receipt["table"]}'
        assert label not in runs
        command = receipt["command"]
        assert ("--hard-v2-handoff" in command) == (receipt["table"] == "handoff")
        assert ("--hard-v2" in command) == (receipt["table"] == "v2")
        assert ("--legacy-hard" in command) == (receipt["table"] == "old")
        paths = list((directory / "raw" / label).glob("*.json"))
        assert len(paths) == 1, paths
        runs[label] = compact(receipt, paths[0])
    assert set(runs) == {f'{s.lower()}-{t}' for s in SUBJECTS for t in TABLES}
    source = json.loads((directory / "source.json").read_text())
    assert all(r["receipt"]["binary_sha256"] == source["binary_sha256"] for r in runs.values())
    # All three tables use the same seeds, factions, and challenger rotation. The
    # full-game and hand-off draws are identical, including across challenger types.
    comparisons = {}
    for subject in SUBJECTS:
        comparisons[subject] = {}
        for a, b in [("old", "v2"), ("old", "handoff"), ("v2", "handoff")]:
            comparisons[subject][f'{a}_vs_{b}'] = compare(
                runs[f'{subject.lower()}-{a}'], runs[f'{subject.lower()}-{b}'])
        reference = runs[f'{subject.lower()}-v2']["games"]
        for game, other in zip(reference, runs[f'{subject.lower()}-handoff']["games"], strict=True):
            assert [p["opening"] for p in game["players"]] == [p["opening"] for p in other["players"]]
    for table in TABLES:
        reference = runs[f'greedyexpand-{table}']["games"]
        for subject in ("snowball", "rush"):
            for a, b in zip(reference, runs[f'{subject}-{table}']["games"], strict=True):
                assert [p["faction"] for p in a["players"]] == [p["faction"] for p in b["players"]]
                assert [p["opening"] for p in a["players"] if p["id"] != a["subject_seat"]] == [
                    p["opening"] for p in b["players"] if p["id"] != b["subject_seat"]]
    members = (directory / "evolved-members.json").read_text()
    assert hashlib.sha256(members.encode()).hexdigest() == source["members_sha256"]
    data = {"format": 2, "source": source, "members_json": members,
            "runs": runs, "comparisons": comparisons}
    if "binary_reconciliation" in source:
        reconciliation = source["binary_reconciliation"]
        changed_at = datetime.fromisoformat(reconciliation["target_binary_ctime_utc"])
        assert source["binary_sha256"] == reconciliation["tested_binary_sha256"]
        for label, started in reconciliation["retained_launch_times"].items():
            run = runs[label]
            assert run["meta"]["started_at"] == started
            assert datetime.fromisoformat(started.replace('Z', '+00:00')) > changed_at
            assert run["receipt"]["initial_cached_binary_sha256"] == reconciliation["initial_binary_sha256"]
        assert len(reconciliation["retained_launch_times"]) == 11
        superseded = json.loads((directory / 'superseded/greedyexpand-old-compact.json').read_text())
        assert superseded["receipt"]["binary_sha256"] == reconciliation["initial_binary_sha256"]
        assert superseded["games"] == runs["greedyexpand-old"]["games"], 'inspect cross-build result differences'
        assert 'Frozen executable' in runs["greedyexpand-old"]["receipt"]["binary_hash_basis"]
        data["superseded_baseline"] = superseded
    return data


def leader_pressure(games):
    return (sum(g["diplomacy"]["leader_at_war_turns"] for g in games),
            sum(g["diplomacy"]["leader_eligible_turns"] for g in games))


def tables(data):
    runs = data["runs"]
    print('| Challenger | Old Hard | Full-game v2 | Hand-off v2 |')
    print('|---|---|---|---|')
    for subject in SUBJECTS:
        values = [format_rate(runs[f'{subject.lower()}-{t}']["summary"]["subject"]) for t in TABLES]
        print(f'| {subject} | ' + ' | '.join(values) + ' |')
    print('\n| Challenger | Comparison (A → B) | Change (pp) | A-only / B-only wins | Paired p |')
    print('|---|---|---:|---:|---:|')
    for subject in SUBJECTS:
        for pair in ("old_vs_v2", "old_vs_handoff", "v2_vs_handoff"):
            result = data["comparisons"][subject][pair]
            print(f'| {subject} | {pair.replace("_vs_", " → ")} | '
                  f'{result["delta_percentage_points"]:+.2f} | '
                  f'{result["old_only"]} / {result["v2_only"]} | {result["exact_two_sided_p"]:.4g} |')
    print('\n| Challenger | Table | Mean turns | Stalls | Wars/game | Declarations/game | AI declarations/game | City captures/game | Settler captures/game |')
    print('|---|---|---:|---:|---:|---:|---:|---:|---:|')
    for subject in SUBJECTS:
        for table in TABLES:
            run = runs[f'{subject.lower()}-{table}']
            s = run["summary"]
            print(f'| {subject} | {table} | {s["mean_turns"]:.2f} | '
                  f'{s["outcomes"].get("stalled", 0)} | {s["mean_war_starts"]:.2f} | '
                  f'{s["mean_declarations"]:.2f} | '
                  f'{s["mean_ai_declarations"]:.2f} | {s["mean_city_captures"]:.3f} | '
                  f'{s["mean_settler_captures"]:.3f} |')
    print('\n| Challenger | Old Hard: leader at war / eligible turns | Full-game v2 | Hand-off v2 |')
    print('|---|---|---|---|')
    for subject in SUBJECTS:
        values = []
        for table in TABLES:
            at_war, eligible = leader_pressure(runs[f'{subject.lower()}-{table}']["games"])
            values.append(f'{at_war:,}/{eligible:,} = {at_war/eligible:.1%}')
        print(f'| {subject} | ' + ' | '.join(values) + ' |')
    print('\n| Original AI opening | Old Hard: share of AI wins | Full-game v2 | Hand-off v2 |')
    print('|---|---|---|---|')
    for opening in OPENINGS:
        values = []
        for table in TABLES:
            counts = [runs[f'{s.lower()}-{table}']["summary"]["ai_openings"] for s in SUBJECTS]
            wins = sum(c.get(opening, {}).get("wins", 0) for c in counts)
            total = sum(v["wins"] for c in counts for v in c.values())
            values.append(f'{wins}/{total} = {wins/total:.1%}')
        print(f'| {opening} | ' + ' | '.join(values) + ' |')
    print('\n| Table / original AI opening | vs GreedyExpand | vs Snowball | vs Rush | vs Proxy |')
    print('|---|---|---|---|---|')
    for table in TABLES:
        for opening in (["Default"] if table == "old" else OPENINGS):
            values = []
            for subject in SUBJECTS:
                counts = runs[f'{subject.lower()}-{table}']["summary"]["ai_openings"]
                wins = counts[opening]["wins"]
                ai_wins = sum(c["wins"] for c in counts.values())
                values.append(f'{wins}/{ai_wins} = {wins/ai_wins:.1%}')
            print(f'| {table} / {opening} | ' + ' | '.join(values) + ' |')
    print('\n| Original opening | AI seats | Hand-offs | Turn min / median / max | Reasons |')
    print('|---|---:|---:|---|---|')
    for opening in OPENINGS:
        seats = [p for s in SUBJECTS for g in runs[f'{s.lower()}-handoff']["games"]
                 for p in g["players"] if p["id"] != g["subject_seat"] and p["opening"] == opening]
        records = [p["opening_handoff"] for p in seats if p["opening_handoff"] is not None]
        turns = [h["turn"] for h in records]
        span = f'{min(turns)} / {median(turns):g} / {max(turns)}' if turns else '—'
        reasons = ', '.join(f'{k}: {v}' for k, v in sorted(Counter(h["reason"] for h in records).items()))
        print(f'| {opening} | {len(seats)} | {len(records)} | {span} | {reasons or "Default from start"} |')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--tables", action="store_true")
    args = parser.parse_args()
    if args.tables:
        with gzip.open(args.input, "rt") as handle:
            data = json.load(handle)
    else:
        if args.out is None:
            parser.error("--out is required for raw-result audits")
        data = audit(args.input)
        args.out.write_bytes(gzip.compress((json.dumps(data, sort_keys=True) + '\n').encode(), mtime=0))
    tables(data)


if __name__ == "__main__":
    main()

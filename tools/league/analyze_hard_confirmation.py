#!/usr/bin/env python3
"""Audit the 2026-10-01 matched Hard benchmarks and retain compact per-game evidence.

python3 tools/league/analyze_hard_confirmation.py \
  .progress/hard-openings-confirmation --out /tmp/hard-confirmation.json.gz

The input directory contains runs.json (launch/exit receipts) and raw/<label>/*.json
from the simulator. Run with --tables to render Markdown tables from saved evidence.
Only the seven opponent seats contribute to opening win rates. Stalls remain in all
denominators and never count as wins. No third-party dependencies.
"""
import argparse
from collections import Counter, defaultdict
import gzip
import hashlib
import json
import math
from pathlib import Path

OPENINGS = ["Default", "Rush", "GreedyExpand", "Snowball", "Economic"]
SUBJECTS = ["GreedyExpand", "Snowball", "Rush", "Proxy"]
PLAYER_FIELDS = [
    "id", "faction", "team", "brain", "difficulty", "opening", "ai_primitives",
    "won", "rank", "vp", "cities", "alive", "wars_declared", "wars_with_battles",
    "cities_captured", "cities_lost", "settlers_captured_by", "settlers_lost",
]


def wilson(wins, total):
    z = 1.959963984540054
    p = wins / total
    denominator = 1 + z * z / total
    center = (p + z * z / (2 * total)) / denominator
    margin = z * math.sqrt(p * (1 - p) / total + z * z / (4 * total * total)) / denominator
    return [max(0, center - margin), min(1, center + margin)]


def rate(wins, total):
    return {"wins": wins, "total": total, "rate": wins / total,
            "wilson_95": wilson(wins, total)}


def compact(receipt, raw_path):
    raw_bytes = raw_path.read_bytes()
    raw = json.loads(raw_bytes)
    subject, table = receipt["opponent"], receipt["table"]
    assert table in ("old", "v2", "handoff")
    total = 128 if subject == "Proxy" else 400
    assert receipt["exit_code"] == 0, receipt
    assert receipt["command"][:3] == ["nice", "-n", "10"]
    assert receipt["observed_niceness"] >= 10
    assert raw["meta"]["threads"] == 8
    assert raw["meta"]["safety_cap"] == 1200
    assert raw["meta"]["turn_limit"] is None
    assert raw["meta"]["first_seed"] == "3500001"
    assert len(raw["games"]) == total
    assert raw["summary"]["panics"] == 0
    setup = raw["meta"]["first_setup"]
    assert (setup["width"], setup["height"]) == (48, 32)
    # Some boolean switches are serialized even when disabled.
    enabled_rules = {k: v for k, v in setup.get("rule_experiments", {}).items() if v is not False}
    assert enabled_rules == ({"hard_v2_ai": True} if table == "v2" else {})
    assert not setup.get("legacy_all_war", False)
    assert setup.get("barbarians", False) == (subject != "Proxy")
    if subject == "Proxy":
        assert raw["meta"]["proxy"] == {
            "seat": 0, "rotate": True, "difficulty": "Hard", "start": "peace", "version": 4,
        }
    games = []
    for i, game in enumerate(raw["games"]):
        assert game["index"] == i and game["seed"] == str(3500001 + i)
        assert game["panic"] is None and game["outcome"] in ("vp", "conquest", "stalled", "draw")
        assert [p["id"] for p in game["players"]] == list(range(8))
        assert len({p["faction"] for p in game["players"]}) == 8
        assert len({p["team"] for p in game["players"]}) == 8
        assert all(p["difficulty"] == "Hard" for p in game["players"])
        seat = i % 8
        for player in game["players"]:
            assert player["won"] == (player["id"] in game["winner_players"])
            if player["id"] == seat:
                if subject == "Proxy":
                    assert player["brain"] == "proxy"
                    assert game["telemetry"]["proxy"]["seat"] == seat
                else:
                    assert player["brain"] == f"opening:{subject}"
                    assert player["opening"] == subject and player["ai_primitives"]
            else:
                opening = player["opening"]
                assert opening in OPENINGS
                if table == "old":
                    assert opening == "Default"
                assert player["brain"] == (
                    "scripted" if opening == "Default" else
                    f'{"hard-v2-handoff" if table == "handoff" else "hard-v2"}:{opening}'
                )
                assert player["ai_primitives"] == (opening != "Default")
            handoff = player.get("opening_handoff")
            if table == "handoff" and player["id"] != seat and player["opening"] != "Default":
                if handoff is not None:
                    assert 2 <= handoff["turn"] <= min(80, game["turns"])
                    assert handoff["reason"] in (
                        "CityTarget", "ArmyTarget", "PhaseEnded", "TurnCap",  # Original evidence.
                        "CoreBuild", "SnowballReady", "RushCaptured", "RushAbandoned",
                    )
                    assert all(p["turn"] <= handoff["turn"] for p in player["phase_switches"])
                    if handoff["reason"] == "PhaseEnded":
                        assert player["phase_switches"][-1]["phase"] != "Opening"
                    if handoff["reason"] == "TurnCap":
                        assert handoff["turn"] == 80
                else:
                    assert not player["alive"] or game["turns"] < 80
            else:
                assert handoff is None
        telemetry = game["telemetry"]
        assert sum(p["wars_declared"] for p in game["players"]) == telemetry["diplomacy"]["declarations"]
        assert sum(p["cities_captured"] for p in game["players"]) == game["city_captures"]
        record = {k: game[k] for k in (
            "index", "seed", "outcome", "winner_players", "turns", "city_captures",
        )}
        record.update({"subject_seat": seat, "diplomacy": telemetry["diplomacy"],
                       "settler_captures": telemetry["settler_captures"],
                       "players": [{k: p[k] for k in PLAYER_FIELDS} for p in game["players"]]})
        # Keep old confirmation evidence byte-for-byte reproducible; new simulator
        # records also retain hand-off receipts and the preceding phase history.
        for player, original in zip(record["players"], game["players"], strict=True):
            if "opening_handoff" in original:
                player["opening_handoff"] = original["opening_handoff"]
                player["phase_switches"] = original["phase_switches"]
        games.append(record)
    counts = defaultdict(lambda: Counter(seats=0, wins=0))
    for game in games:
        for player in game["players"]:
            if player["id"] != game["subject_seat"]:
                counts[player["opening"]].update(seats=1, wins=int(player["won"]))
    assert sum(c["seats"] for c in counts.values()) == total * 7
    assert set(counts) == ({"Default"} if table == "old" else set(OPENINGS))
    assert set(Counter(g["subject_seat"] for g in games).values()) == {total // 8}
    if subject == "Proxy":
        pairs = Counter((g["subject_seat"], g["players"][g["subject_seat"]]["faction"]) for g in games)
        assert len(pairs) == 64 and set(pairs.values()) == {2}
    wins = sum(g["subject_seat"] in g["winner_players"] for g in games)
    summary = {
        "subject": rate(wins, total),
        "outcomes": dict(Counter(g["outcome"] for g in games)),
        "mean_turns": sum(g["turns"] for g in games) / total,
        "mean_war_starts": sum(g["diplomacy"]["war_starts"] for g in games) / total,
        "mean_declarations": sum(g["diplomacy"]["declarations"] for g in games) / total,
        "mean_city_captures": sum(g["city_captures"] for g in games) / total,
        "mean_settler_captures": sum(g["settler_captures"] for g in games) / total,
        "mean_ai_declarations": sum(p["wars_declared"] for g in games for p in g["players"]
                                    if p["id"] != g["subject_seat"]) / total,
        "mean_subject_rank": sum(g["players"][g["subject_seat"]]["rank"] for g in games) / total,
        "ai_openings": dict(counts),
        "stalled_seeds": [g["seed"] for g in games if g["outcome"] == "stalled"],
    }
    assert sum(c["wins"] for c in counts.values()) + wins == sum(len(g["winner_players"]) for g in games)
    return {"receipt": receipt, "meta": raw["meta"], "summary": summary, "games": games,
            "raw_sha256": hashlib.sha256(raw_bytes).hexdigest(),
            "proxy_summary": raw["summary"].get("proxy")}


def compare(old, v2):
    discordant = Counter(old_only=0, v2_only=0)
    for a, b in zip(old["games"], v2["games"], strict=True):
        assert (a["index"], a["seed"], a["subject_seat"]) == (b["index"], b["seed"], b["subject_seat"])
        assert [p["faction"] for p in a["players"]] == [p["faction"] for p in b["players"]]
        won_a = a["subject_seat"] in a["winner_players"]
        won_b = b["subject_seat"] in b["winner_players"]
        if won_a != won_b:
            discordant["old_only" if won_a else "v2_only"] += 1
    n = sum(discordant.values())
    p = min(1.0, 2 * sum(math.comb(n, k) for k in range(min(discordant.values()) + 1)) / 2 ** n)
    return {**discordant, "exact_two_sided_p": p,
            "delta_percentage_points": 100 * (v2["summary"]["subject"]["rate"] -
                                               old["summary"]["subject"]["rate"])}


def format_rate(value):
    lo, hi = value["wilson_95"]
    return (f'{value["wins"]}/{value["total"]} = {value["rate"]:.2%} '
            f'(95% CI {lo:.1%}–{hi:.1%})')


def tables(data):
    runs = data["runs"]
    print('| Challenger | vs old Hard | vs Hard v2 | Change (pp) | Old-only / v2-only wins; paired p |')
    print('|---|---|---|---:|---|')
    for subject in SUBJECTS:
        a, b = [runs[f'{subject.lower()}-{table}']["summary"]["subject"] for table in ("old", "v2")]
        c = data["comparisons"][subject]
        print(f'| {subject} | {format_rate(a)} | {format_rate(b)} | '
              f'{c["delta_percentage_points"]:+.2f} | {c["old_only"]} / {c["v2_only"]}; '
              f'p = {c["exact_two_sided_p"]:.4g} |')
    print('\n| Challenger | Table | Mean turns | Stalls | Wars/game | Declarations/game | AI declarations/game | City captures/game | Settler captures/game |')
    print('|---|---|---:|---:|---:|---:|---:|---:|---:|')
    for key in (f'{s.lower()}-{t}' for s in SUBJECTS for t in ("old", "v2")):
        run = runs[key]
        s, r = run["summary"], run["receipt"]
        print(f'| {r["opponent"]} | {r["table"]} | {s["mean_turns"]:.2f} | '
              f'{s["outcomes"].get("stalled", 0)} | {s["mean_war_starts"]:.2f} | '
              f'{s["mean_declarations"]:.2f} | {s["mean_ai_declarations"]:.2f} | '
              f'{s["mean_city_captures"]:.3f} | {s["mean_settler_captures"]:.3f} |')
    print('\n| Challenger | Old Hard: leader at war / eligible turns | Hard v2: leader at war / eligible turns |')
    print('|---|---|---|')
    for subject in SUBJECTS:
        values = []
        for table in ("old", "v2"):
            games = runs[f'{subject.lower()}-{table}']["games"]
            eligible = sum(g["diplomacy"]["leader_eligible_turns"] for g in games)
            war = sum(g["diplomacy"]["leader_at_war_turns"] for g in games)
            values.append(f'{war:,}/{eligible:,} = {war/eligible:.1%}')
        print(f'| {subject} | ' + ' | '.join(values) + ' |')
    print('\n| Table / AI opening | vs GreedyExpand | vs Snowball | vs Rush | vs Proxy |')
    print('|---|---|---|---|---|')
    for table, openings in [("old", ["Default"]), ("v2", OPENINGS)]:
        for opening in openings:
            values = []
            for subject in SUBJECTS:
                c = runs[f'{subject.lower()}-{table}']["summary"]["ai_openings"][opening]
                values.append(f'{c["wins"]}/{c["seats"]} = {c["wins"]/c["seats"]:.1%}')
            print(f'| {table} / {opening} | ' + ' | '.join(values) + ' |')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--tables", action="store_true")
    args = parser.parse_args()
    if args.tables:
        with gzip.open(args.input, "rt") as handle:
            tables(json.load(handle))
        return
    if args.out is None:
        parser.error("--out is required when auditing raw results")
    runs = {}
    for receipt in json.loads((args.input / "runs.json").read_text()):
        paths = list((args.input / "raw" / receipt["label"]).glob("*.json"))
        assert len(paths) == 1, paths
        runs[receipt["label"]] = compact(receipt, paths[0])
    assert len(runs) == 8
    assert len({r["receipt"]["binary_sha256"] for r in runs.values()}) == 1
    for table in ("old", "v2"):
        reference = runs[f'greedyexpand-{table}']["games"]
        for subject in ("snowball", "rush"):
            for a, b in zip(reference, runs[f'{subject}-{table}']["games"], strict=True):
                assert a["seed"] == b["seed"]
                assert [p["faction"] for p in a["players"]] == [p["faction"] for p in b["players"]]
                assert [p["opening"] for p in a["players"] if p["id"] != a["subject_seat"]] == [
                    p["opening"] for p in b["players"] if p["id"] != b["subject_seat"]
                ]
    source = json.loads((args.input / "source.json").read_text())
    assert all(r["receipt"]["binary_sha256"] == source["binary_sha256"] for r in runs.values())
    members = (args.input / "evolved-members.json").read_text()
    assert hashlib.sha256(members.encode()).hexdigest() == source["members_sha256"]
    data = {"format": 1, "source": source, "members_json": members, "runs": runs, "comparisons": {
        s: compare(runs[f'{s.lower()}-old'], runs[f'{s.lower()}-v2']) for s in SUBJECTS
    }}
    args.out.write_bytes(gzip.compress((json.dumps(data, sort_keys=True) + '\n').encode(), mtime=0))
    tables(data)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Summarize the retained hard-vs-human evidence (no third-party dependencies)."""
import argparse
import collections
import gzip
import json
import math
import statistics


def proxy(games):
    wins = [g for g in games if g["telemetry"]["proxy"]["seat"] in g["winner_players"]]
    n, k = len(games), len(wins)
    z = 1.959963984540054
    center = (k / n + z * z / (2 * n)) / (1 + z * z / n)
    half = z * math.sqrt(k / n * (1 - k / n) / n + z * z / (4 * n * n)) / (1 + z * z / n)
    diagnostics = [g["telemetry"]["proxy"]["diagnostics"] for g in games]
    return {
        "games": n, "wins": k, "win_percent": 100 * k / n,
        "wilson95_percent": [100 * (center - half), 100 * (center + half)],
        "outcomes": dict(collections.Counter(g["outcome"] for g in games)),
        "mean_turns": statistics.mean(g["turns"] for g in games),
        "captures": sum(g["telemetry"]["proxy"]["proxy_captures"] for g in games),
        "cities_lost": sum(d["cities_lost"] for d in diagnostics),
        "settlers_captured": sum(d["settlers_captured"] for d in diagnostics),
        "military_purchase_gold": sum(d["military_purchase_gold"] for d in diagnostics),
    }


def hard(games):
    wins = collections.Counter(f for g in games for f in g["winner_factions"])
    factions = sorted({p["faction"] for g in games for p in g["players"]})
    rates = {f: wins[f] / sum(p["faction"] == f for g in games for p in g["players"]) for f in factions}
    return {"games": len(games), "mean_turns": statistics.mean(g["turns"] for g in games),
            "outcomes": dict(collections.Counter(g["outcome"] for g in games)),
            "faction_wins": {f: wins[f] for f in factions},
            "faction_ratio": max(rates.values()) / min(rates.values()) if min(rates.values()) else None}


def paired_proxy(before, after):
    a = {g["seed"]: g for g in before}
    b = {g["seed"]: g for g in after}
    assert len(a) == len(before) == len(b) == len(after) and a.keys() == b.keys()
    counts = collections.Counter()
    for seed, old in a.items():
        new = b[seed]
        assert old["telemetry"]["proxy"]["seat"] == new["telemetry"]["proxy"]["seat"]
        assert [(p["id"], p["team"], p["faction"], p["difficulty"]) for p in old["players"]] == [
            (p["id"], p["team"], p["faction"], p["difficulty"]) for p in new["players"]]
        seat = old["telemetry"]["proxy"]["seat"]
        counts[(seat in old["winner_players"], seat in new["winner_players"])] += 1
    return {"both_win": counts[True, True], "before_only": counts[True, False],
            "after_only": counts[False, True], "neither_win": counts[False, False]}


def audit(rows):
    predicates = {
        "no_garrison": lambda c: c["garrison_hp"] is None,
        "at_most_one_local_defender": lambda c: c["local_defenders"] <= 1,
        "isolated_over_six_tiles": lambda c: c["nearest_other_city"] is None or c["nearest_other_city"] > 6,
        "within_five_turns_of_war": lambda c: c["war_age"] is not None and c["war_age"] <= 5,
        "prior_visible_buildup_three_turns": lambda c: c["warning_age"] is not None and c["warning_age"] >= 3,
        "affordable_defender_no_purchase": lambda c: c["affordable_defender"] and not c["bought"],
    }
    categories = {}
    for name, predicate in predicates.items():
        affected = [r for r in rows if any(predicate(c) for c in r["captures"])]
        categories[name] = {"captures": sum(predicate(c) for r in rows for c in r["captures"]),
                            "games": len(affected), "proxy_wins": sum(r["win"] for r in affected)}
    followup = [(r, c) for r in rows for c in r["captures"] if c["turn"] + 10 <= r["turns"]]
    return {
        "games": len(rows), "verified_turns": sum(r["turns"] for r in rows),
        "captures": sum(len(r["captures"]) for r in rows), "capture_categories": categories,
        "settlers": sum(len(r["settlers"]) for r in rows),
        "settlers_visible_raider": sum(s["visible_raider"] for r in rows for s in r["settlers"]),
        "settlers_no_escort": sum(not s["escort"] for r in rows for s in r["settlers"]),
        "settlers_at_peace": sum(s["peace"] for r in rows for s in r["settlers"]),
        "recaptures_within_ten": sum(c["original_loss"] is not None and c["turn"]-c["original_loss"][0]<=10 for r in rows for c in r["recaptures"]),
        "captures_with_ten_turn_followup": len(followup),
        "not_recovered_within_ten": sum(not any(
            x["city"] == c["city"] and c["turn"] < x["turn"] <= c["turn"] + 10
            for x in r["recaptures"]) for r, c in followup),
        "recapture_attack_turns": sum(len(r["recapture_attacks"]) for r in rows),
        "recapture_attack_turns_without_recovery": sum(
            not c["recovered"] for r in rows for c in r["recapture_attacks"]),
        "leader_peaces": sum(len(r.get("leader_peaces", [])) for r in rows),
        "leader_peaces_without_local_army_or_damage": sum(
            not p["visible_army_near_home"] and not p["damaged_home"]
            for r in rows for p in r.get("leader_peaces", [])),
        "counterfire_opportunities": sum(r.get("counterfire_opportunities", 0) for r in rows),
        "withheld_counterfire": sum(r.get("withheld_counterfire", 0) for r in rows),
        "withheld_wounded_counterfire": sum(r.get("withheld_wounded_counterfire", 0) for r in rows),
        "snowball_turns": sum(r["snowball_turns"] for r in rows),
        "snowball_no_coalition": sum(r["snowball_no_coalition"] for r in rows),
    }


def summarize(evidence):
    out = {}
    for label, games in evidence["runs"].items():
        out[label] = proxy(games) if games[0].get("telemetry", {}).get("proxy") else hard(games)
    for label, rows in evidence.get("audits", {}).items():
        out[label + "_audit"] = audit(rows)
    for before, after in evidence.get("matched_proxy_pairs", []):
        out[before + "_vs_" + after] = paired_proxy(evidence["runs"][before], evidence["runs"][after])
    if {"master_hard", "branch_hard", "branch_proxy_v6"} <= out.keys():
        old, new, v6 = (out[k] for k in ("master_hard", "branch_hard", "branch_proxy_v6"))
        schedules = {"branch_proxy_v6": (3600001, 400),
                     "master_hard": (3200001, 200), "branch_hard": (3200001, 200)}
        checks = {
            "specified_seeds": all(sorted(int(g["seed"]) for g in evidence["runs"][label])
                                   == list(range(seed, seed + count))
                                   for label, (seed, count) in schedules.items()),
            "v6_at_most_88_wins": v6["wins"] <= 88,
            "hard_mean_within_ten_percent": 0.9 * old["mean_turns"] <= new["mean_turns"] <= 1.1 * old["mean_turns"],
            "hard_stalls_not_up": new["outcomes"].get("stalled", 0) <= old["outcomes"].get("stalled", 0),
            "hard_faction_ratio_within_point_two": new["faction_ratio"] is not None
                and new["faction_ratio"] <= old["faction_ratio"] + 0.2,
            "no_panics": not any(g["panic"] for label in schedules for g in evidence["runs"][label]),
        }
        out["acceptance"] = {"passed": all(checks.values()), "checks": checks,
                             "hard_mean_change_percent": 100 * (new["mean_turns"] / old["mean_turns"] - 1)}
    return out


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", help="Retained evidence JSON or .json.gz bundle")
    args = parser.parse_args()
    with gzip.open(args.evidence, "rt") if args.evidence.endswith(".gz") else open(args.evidence) as source:
        print(json.dumps(summarize(json.load(source)), indent=2))

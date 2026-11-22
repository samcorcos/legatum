#!/usr/bin/env python3
"""Recompute matched playbook metrics from raw simulator reports or committed evidence.

Example: python3 tools/analysis/human_playbook_v6.py BASELINE.json.gz V6.json
Only reads observer reports; this is never imported by a policy or simulator.
"""
import argparse
import collections
import gzip
import hashlib
import json
import math
import pathlib
import statistics


def read(path):
    raw = pathlib.Path(path).read_bytes()
    return json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)


def games(report, policy="v5"):
    if "reports" in report:
        report = report["reports"][policy]
    return [dict(g, proxy=g.get("proxy") or g["telemetry"]["proxy"]) for g in report["games"]]


def metric(values):
    xs = list(values)
    return dict(n=len(xs), mean=statistics.mean(xs) if xs else None,
                median=statistics.median(xs) if xs else None, total=sum(xs))


def won(g):
    return g["proxy"]["seat"] in (g["winner_players"] or [])


def wilson(wins, n):
    if not n:
        return [None, None]
    z = 1.959963984540054
    p = wins / n
    center = (p + z*z/(2*n)) / (1 + z*z/n)
    half = z * math.sqrt(p*(1-p)/n + z*z/(4*n*n)) / (1 + z*z/n)
    return [center-half, center+half]


def summarize(gs):
    ps = [g["proxy"] for g in gs]
    ds = [p["diagnostics"] for p in ps]
    wins = sum(map(won, gs))
    result = dict(games=len(gs), wins=wins, rate=wins/len(gs), ci95=wilson(wins, len(gs)),
                  winning_turn=metric(g["turns"] for g in gs if won(g)),
                  outcomes=dict(collections.Counter(g["outcome"] for g in gs)),
                  captures=metric(p["proxy_captures"] for p in ps))
    for field in ["first_war", "first_capture", "settlers_captured", "settlers_lost",
                  "military_purchase_gold", "cities_lost", "city_fire_hits", "city_fire_kills",
                  "cities_zeroed_turns", "zeroed_without_capture_turns"]:
        result[field] = metric(d[field] for d in ds if d.get(field) is not None)
    result["third_founding"] = metric(d["foundings"][2] for d in ds if len(d["foundings"]) >= 3)
    result["siege_research"] = metric(d["first_research"]["SiegeEngineering"] for d in ds
                                      if "SiegeEngineering" in d["first_research"])
    result["siege_by_136"] = sum(d["first_research"].get("SiegeEngineering", 99999) <= 136 for d in ds)
    result["siege_by_200"] = sum(d["first_research"].get("SiegeEngineering", 99999) <= 200 for d in ds)
    result["no_capture_games"] = sum(p["proxy_captures"] == 0 for p in ps)
    result["two_capture_games"] = sum(p["proxy_captures"] >= 2 for p in ps)
    result["first_military_purchase"] = metric(min(map(int, d["military_purchases"]))
        for d in ds if d["military_purchases"])
    for field in ["production", "units_lost", "failed_orders", "disbands", "incidents"]:
        result[field] = dict(sum((collections.Counter(d.get(field, {})) for d in ds), collections.Counter()))
    result["military_purchases"] = metric(sum(d["military_purchases"].values()) for d in ds)
    result["purchase_bursts"] = {}
    for cap in [136, 1200]:
        bursts = [max([0] + [n+(d["military_purchases"].get(str(int(t)+1), 0) if int(t) < cap else 0)
                              for t, n in d["military_purchases"].items() if int(t) <= cap]) for d in ds]
        result["purchase_bursts"][cap] = {size: sum(b >= size for b in bursts) for size in [2, 4]}
    result["snapshots"] = {}
    for turn in [100, 136, 150, 200]:
        ss = [p["snapshots"][str(turn)] for p in ps if str(turn) in p["snapshots"]]
        result["snapshots"][turn] = dict(cities=metric(s["proxy_cities"] for s in ss),
            captures=metric(s["diagnostics"]["proxy_captures"] for s in ss),
            science=metric(s["diagnostics"]["proxy_science_per_turn"] for s in ss),
            gold=metric(s["diagnostics"]["proxy_gold"] for s in ss),
            income=metric(s["diagnostics"]["proxy_gold_per_turn"] for s in ss),
            tech_lead=metric(s["proxy_techs"]-max(s["ai_techs"].values()) for s in ss),
            cities_ge={n: sum(s["proxy_cities"] >= n for s in ss) for n in [5, 6, 8]},
            captures_ge={n: sum(s["diagnostics"]["proxy_captures"] >= n for s in ss) for n in [1, 2, 3]},
            with_siege=sum(s["diagnostics"]["proxy_units"].get("SiegeEngine", 0) > 0 for s in ss),
            units=dict(sum((collections.Counter(s["diagnostics"]["proxy_units"]) for s in ss), collections.Counter())))
    result["factions"] = {f: dict(games=sum(g["proxy"]["faction"] == f for g in gs),
                                    wins=sum(g["proxy"]["faction"] == f and won(g) for g in gs))
                          for f in sorted({p["faction"] for p in ps})}
    result["closing"] = {}
    for faction in result["factions"]:
        rows = []
        for g in gs:
            if g["proxy"]["faction"] != faction:
                continue
            telemetry = g.get("observer", g.get("telemetry", {}))
            seat = str(g["proxy"]["seat"])
            score = telemetry.get("final_scores", {}).get("players", {}).get(seat)
            if score is not None:
                rows.append((g, score, telemetry.get("faction_goal_turns", {}).get(seat)))
        goal_vp = {"Sparta": 4, "Byzantium": 3}.get(faction, 2)
        result["closing"][faction] = dict(
            observations=len(rows), goals=sum(score["faction_goals"] > 0 for _, score, _ in rows),
            goal_turn=metric(turn for _, _, turn in rows if turn is not None),
            # These are final-position deficits, not predicted counterfactual wins.
            losses_within_missing_goal_vp=sum(not won(g) and score["faction_goals"] == 0
                and 0 < 10-score["total"] <= goal_vp for g, score, _ in rows),
            losses_within_missing_work_vp=sum(not won(g) and score["milestones"]["RelayUplink"] == 0
                and 0 < 10-score["total"] <= 2 for g, score, _ in rows))
    result["cohorts_136"] = {}
    for name, predicate in [
        ("cities_ge_5", lambda s: s["proxy_cities"] >= 5),
        ("cities_lt_5", lambda s: s["proxy_cities"] < 5),
        ("captures_ge_2", lambda s: s["diagnostics"]["proxy_captures"] >= 2),
        ("no_captures", lambda s: s["diagnostics"]["proxy_captures"] == 0),
    ]:
        cohort = [g for g in gs if "136" in g["proxy"]["snapshots"]
                  and predicate(g["proxy"]["snapshots"]["136"])]
        result["cohorts_136"][name] = dict(games=len(cohort), wins=sum(map(won, cohort)))
    return result


def matched(a, b):
    assert len(a) == len(b), "unequal game counts"
    result = collections.Counter()
    for ga, gb in zip(a, b):
        assert ga["seed"] == gb["seed"], "unmatched seeds"
        assert ga["proxy"]["seat"] == gb["proxy"]["seat"], "unmatched seats"
        assert ga["proxy"]["faction"] == gb["proxy"]["faction"], "unmatched proxy factions"
        assert len(ga["players"]) == len(gb["players"]), "unmatched player counts"
        for pa, pb in zip(ga["players"], gb["players"]):
            for key in ["id", "team", "faction", "difficulty"]:
                assert pa[key] == pb[key], f"unmatched setup: {key}"
        result[f"{int(won(ga))}{int(won(gb))}"] += 1
    discordant = result["10"] + result["01"]
    result["exact_mcnemar_p"] = min(1, 2*sum(math.comb(discordant, k)
        for k in range(min(result["10"], result["01"])+1))/2**discordant) if discordant else 1
    return dict(result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline")
    parser.add_argument("candidate")
    args = parser.parse_args()
    a, b = games(read(args.baseline), "v5"), games(read(args.candidate), "v6")
    result = dict(baseline=summarize(a), candidate=summarize(b), paired=matched(a, b),
                  inputs={p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
                          for p in [args.baseline, args.candidate]})
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()

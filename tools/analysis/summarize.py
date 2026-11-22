#!/usr/bin/env python3
"""Summarize probe JSON + telemetry using Python's standard library only.

Faction intervals resample whole games, preserving the competing/repeated seats.
Wilson intervals are for one Bernoulli observation per game (not pooled seats).
All VP subtraction exercises hold the terminal state fixed; they are not replays.
"""

import argparse
from collections import Counter, defaultdict
import gzip
import json
import math
from pathlib import Path
import random
import statistics

FACTIONS = ("Switzerland", "Japan", "Byzantium")
# Telemetry recorded before the factions were renamed (tools/dev/rename-legacy-factions.py).
LEGACY_FACTIONS = {"Tollkeepers": "Switzerland", "Choir": "Japan", "Remnant": "Byzantium"}
TITLES = ("LargestEconomy", "MostTerritory", "StrongestArmy", "MostAdvanced", "MostPopulous")
MILESTONES = ("FourCities", "SixTechs", "TwentyPopulation", "RelayUplink")
SOURCES = TITLES + MILESTONES + ("Faction goal",)


def percentile(values, q):
    """The simulator's rounded index convention; q in [0, 1]."""
    values = sorted(values)
    return values[min(len(values) - 1, int(q * (len(values) - 1) + 0.5))] if values else None


def dist(values):
    values = list(values)
    if not values:
        return None
    return dict(n=len(values), mean=statistics.mean(values), min=min(values),
                p10=percentile(values, .1), p25=percentile(values, .25),
                median=percentile(values, .5), p75=percentile(values, .75),
                p90=percentile(values, .9), max=max(values))


def histogram_dist(h):
    return dist(v for k, n in h.items() for v in [int(k)] * n)


def wilson(wins, n):
    if not n:
        return None
    z = 1.959963984540054
    p = wins / n
    d = 1 + z * z / n
    center = (p + z * z / (2 * n)) / d
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return [max(0, center - half), min(1, center + half)]


def rate(wins, n):
    return dict(wins=wins, n=n, rate=wins / n if n else None, ci95=wilson(wins, n))


def load_telemetry(path):
    opener = gzip.open if str(path).endswith(".gz") else open
    with opener(path, "rt") as f:
        records = [json.loads(line) for line in f if line.strip()]
    for g in records:
        for p in g.get("players", ()):
            p["faction"] = LEGACY_FACTIONS.get(p.get("faction"), p.get("faction"))
    return records


def load_report(path):
    """Read a sim report JSON; large reports are committed as <name>.json.gz."""
    path = Path(path)
    if not path.exists() and path.with_name(path.name + ".gz").exists():
        return json.loads(gzip.decompress(path.with_name(path.name + ".gz").read_bytes()))
    return json.loads(path.read_text())


def verify(records, reference):
    assert len(records) == len(reference)
    for g, ref in zip(records, reference):
        for key in ("seed", "turns", "winner_team", "outcome", "eliminations"):
            assert g[key] == ref[key], (g["seed"], key, g[key], ref[key])
        assert len(g["players"]) == len(ref["players"])
        for p, rp in zip(g["players"], ref["players"]):
            for key, value in rp.items():
                assert p[key] == value, (g["seed"], p["id"], key)


def bootstrap_factions(records, replications):
    rows = []
    for g in records:
        row = []
        for f in FACTIONS:
            seats = [p for p in g["players"] if p["faction"] == f]
            row += [sum(p["team"] == g["winner_team"] for p in seats), len(seats)]
        rows.append(row)
    rng = random.Random(20260926)
    samples = [[] for _ in FACTIONS]
    differences = {"Switzerland-Japan": [], "Switzerland-Byzantium": [], "Japan-Byzantium": []}
    for _ in range(replications):
        totals = list(map(sum, zip(*rng.choices(rows, k=len(rows)))))
        rates = [totals[2 * i] / totals[2 * i + 1] for i in range(3)]
        for i, value in enumerate(rates):
            samples[i].append(value)
        for name, (a, b) in zip(differences, ((0, 1), (0, 2), (1, 2))):
            differences[name].append(rates[a] - rates[b])
    totals = list(map(sum, zip(*rows)))
    out = {}
    for i, f in enumerate(FACTIONS):
        w, n = totals[2 * i:2 * i + 2]
        out[f] = dict(wins=w, seats=n, rate=w / n,
                      ci95=[percentile(samples[i], .025), percentile(samples[i], .975)])
    return out, {k: [percentile(v, .025), percentile(v, .975)] for k, v in differences.items()}


def group_sources(p):
    src = p["sources"]
    return dict(titles=sum(src.get(s, 0) for s in TITLES),
                milestones=sum(src.get(s, 0) for s in MILESTONES),
                faction_goal=src.get("Faction goal", 0))


def summarize(records, bootstraps=10000):
    assert records
    counts = Counter()
    hists = defaultdict(Counter)
    players = []
    winners = []
    vp_winners = []
    margin_sources = Counter()
    terminal_gain = Counter()
    terminal_patterns = Counter()
    source_times = defaultdict(list)
    terminal_subtraction = Counter()
    for g in records:
        counts.update(g["counts"])
        for name, h in g["histograms"].items():
            hists[name].update(h)
        assert sum(g["histograms"]["battles_per_turn"].values()) == g["turns"]
        assert sum(int(k) * n for k, n in g["histograms"]["battles_per_turn"].items()) == g["counts"].get("battles", 0)
        assert len(g["captures"]) == g["counts"].get("captures", 0)
        players.extend(g["players"])
        for p in g["players"]:
            assert sum(p["sources"].values()) == p["vp"]
        for e in g["scoring_events"]:
            if e["source"] not in TITLES:
                f = g["players"][e["to"]]["faction"]
                source_times[e["source"]].append(e["turn"])
                if e["source"] == "Faction goal":
                    source_times[f + " goal"].append(e["turn"])
        if g["winner_team"] is None:
            continue
        w = next(p for p in g["players"] if p["team"] == g["winner_team"])
        winners.append(w)
        runner = max((p for p in g["players"] if p["id"] != w["id"]), key=lambda p: (p["vp"], p["population"], -p["id"]))
        for name, amount in group_sources(w).items():
            margin_sources[name] += amount - group_sources(runner)[name]
        if g["outcome"] == "vp":
            vp_winners.append(w)
            old = g["before_final"][w["id"]]
            gained = []
            for s in SOURCES:
                delta = w["sources"].get(s, 0) - old.get(s, 0)
                if delta > 0:
                    terminal_gain[s] += 1
                    gained.append(s)
            terminal_patterns[" + ".join(gained)] += 1
        for name in ("titles", "milestones", "faction_goal"):
            reduced = {p["id"]: p["vp"] - group_sources(p)[name] for p in g["players"]}
            if g["outcome"] == "vp" and reduced[w["id"]] < 10:
                terminal_subtraction[name + "_below_target"] += 1
            if reduced[w["id"]] > max(v for p, v in reduced.items() if p != w["id"]):
                terminal_subtraction[name + "_still_unique_leader"] += 1
    faction_rates, faction_diffs = bootstrap_factions(records, bootstraps)
    source_stats = {}
    for src in SOURCES:
        held = [p for p in players if src in p["sources"]]
        winning = [p for p in winners if src in p["sources"]]
        source_stats[src] = dict(holders=len(held), winner_holders=len(winning),
                                winner_vp_total=sum(p["sources"].get(src, 0) for p in winners),
                                all_vp_total=sum(p["sources"].get(src, 0) for p in players),
                                winner_share=len(winning) / len(winners) if winners else None,
                                win_rate_given_held=len(winning) / len(held) if held else None,
                                claim_turns=dist(source_times[src]))
    snow = [g for g in records if g["turn50"] is not None]
    unique, tied = [], []
    leader_counts = Counter()
    gap_counts = defaultdict(lambda: [0, 0])
    random_tie_credit = 0
    random_leader_chance = 0
    for g in snow:
        ranked = sorted(g["turn50"], key=lambda p: p["vp"], reverse=True)
        lead = [p["team"] for p in ranked if p["vp"] == ranked[0]["vp"]]
        won = g["winner_team"] in lead
        (unique if len(lead) == 1 else tied).append(won)
        leader_counts[len(lead)] += 1
        gap = ranked[0]["vp"] - ranked[1]["vp"]
        gap_counts[gap][0] += won
        gap_counts[gap][1] += 1
        random_tie_credit += won / len(lead)
        random_leader_chance += len(lead) / len(ranked)
    brain_stats = {}
    for brain in sorted({p["brain"] for p in players}):
        ps = [p for p in players if p["brain"] == brain]
        ws = [p for p in winners if p["brain"] == brain]
        seats_per_game = len(ps) / len(records)
        assert all(sum(p["brain"] == brain for p in g["players"]) == seats_per_game for g in records)
        brain_stats[brain] = dict(seats=len(ps), wins=len(ws), win_rate=len(ws) / len(ps),
                                 game_win=rate(len(ws), len(records)),
                                 seat_ci95=[x / seats_per_game for x in wilson(len(ws), len(records))],
                                 vp=dist(p["vp"] for p in ps), cities=dist(p["cities"] for p in ps),
                                 techs=dist(p["techs"] for p in ps), army=dist(p["army"] for p in ps),
                                 alive=sum(p["alive"] for p in ps))
    metrics = {}
    for metric in ("battles", "city_battles", "garrison_battles", "captures", "ranged_battles", "units_destroyed", "army_destroyed", "units_disbanded"):
        values = [g["counts"].get(metric, 0) for g in records]
        metrics[metric] = dict(total=sum(values), games_nonzero=sum(v > 0 for v in values), **dist(values))
    capture_spans = [c["turn"] - c["first_hit"] + 1 for g in records for c in g["captures"] if c["first_hit"] is not None]
    goal_factions = {}
    for f in FACTIONS:
        ps = [p for p in players if p["faction"] == f]
        ws = [p for p in winners if p["faction"] == f]
        goal_factions[f] = dict(seats=len(ps), completed=sum("Faction goal" in p["sources"] for p in ps),
                               winners=len(ws), winner_completed=sum("Faction goal" in p["sources"] for p in ws),
                               claim_turns=dist(source_times[f + " goal"]))
    return dict(
        n=len(records), outcomes=dict(Counter(g["outcome"] for g in records)),
        length=dist(g["turns"] for g in records),
        length_bins=dict(Counter(f"{((g['turns'] - 1) // 25) * 25 + 1}-{((g['turns'] - 1) // 25 + 1) * 25}" for g in records)),
        reached_turn_limit=sum(g["turns"] == 150 for g in records),
        length_by_outcome={o: dist(g["turns"] for g in records if g["outcome"] == o) for o in {g["outcome"] for g in records}},
        faction_rates=faction_rates, faction_difference_ci95=faction_diffs, brain=brain_stats,
        eliminations=dict(total=sum(len(g["eliminations"]) for g in records), games=rate(sum(bool(g["eliminations"]) for g in records), len(records))),
        vp=dict(winners=len(winners), vp_winners=len(vp_winners),
                vp_winners_without_relay=sum("RelayUplink" not in p["sources"] for p in vp_winners),
                vp_winner_title_counts=dict(Counter(sum(s in p["sources"] for s in TITLES) for p in vp_winners)),
                winner_total=sum(p["vp"] for p in winners), all_total=sum(p["vp"] for p in players),
                sources=source_stats, goals_by_faction=goal_factions,
                terminal_gains=dict(terminal_gain), terminal_patterns=dict(terminal_patterns.most_common()),
                terminal_subtraction=dict(terminal_subtraction),
                mean_margin_by_source={k: v / len(winners) for k, v in margin_sources.items()} if winners else None),
        snowball=dict(eligible=len(snow), missing=len(records) - len(snow),
                      unique_leader=rate(sum(unique), len(unique)),
                      tied_leader_set=rate(sum(tied), len(tied)),
                      any_leader=rate(sum(unique) + sum(tied), len(snow)),
                      random_pick_among_tied_leaders_win_rate=random_tie_credit / len(snow) if snow else None,
                      chance_winner_in_leader_set_if_uniform=random_leader_chance / len(snow) if snow else None,
                      leader_count_histogram=dict(leader_counts),
                      gap_win_rates={k: rate(*v) for k, v in sorted(gap_counts.items())}),
        combat=metrics, counts=dict(counts), histogram_distributions={k: histogram_dist(v) for k, v in hists.items()},
        first_battle=dist(g["first_battle"] for g in records if g["first_battle"] is not None),
        first_city_battle=dist(g["first_city_battle"] for g in records if g["first_city_battle"] is not None),
        capture_span=dist(capture_spans),
        captures_by_brain=dict(Counter(c["from_brain"] + "->" + c["to_brain"] for g in records for c in g["captures"])),
        cities_hit=sum(g["cities_hit"] for g in records),
        cities_reduced_to_zero=sum(g["cities_reduced_to_zero"] for g in records),
        distinct_captured_cities=sum(len({c["city"] for c in g["captures"]}) for g in records),
        final_unit_mix=dict(sum((Counter(p["units"]) for p in players), Counter())),
        army_final=dist(p["army"] for p in players),
        army_turn50=dist(p["army"] for g in snow for p in g["turn50"]),
        army_strength_final=dist(p["army_strength"] for p in players),
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("telemetry", type=Path)
    parser.add_argument("--reference", type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--games", type=int, help="Summarize first N games (paired experiment baseline).")
    parser.add_argument("--bootstraps", type=int, default=10000)
    args = parser.parse_args()
    records = load_telemetry(args.telemetry)
    assert args.bootstraps > 0
    if args.reference:
        reference = load_report(args.reference)["games"]
        verify(records, reference)
        print(f"Verified all final fields and eliminations for {len(records)} games against {args.reference}")
    if args.games:
        assert 0 < args.games <= len(records)
        records = records[:args.games]
    result = summarize(records, args.bootstraps)
    result["method"] = dict(bootstrap_replications=args.bootstraps, bootstrap_seed=20260926,
                            telemetry=str(args.telemetry), reference=str(args.reference) if args.reference else None,
                            final_records_verified=bool(args.reference))
    args.out.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(f"Wrote {args.out}")


if __name__ == "__main__":
    main()

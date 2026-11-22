#!/usr/bin/env python3
"""Reproduce the V10 iteration's cohort diagnostics and telemetry observations.

Individual faction intervals are Wilson 95% score intervals. Independent cohort
differences use Newcombe's Wilson construction; cohort tests use a Pearson
multinomial homogeneity test, retaining the no-winner category. A paired bundle
effect is a different estimand and uses the existing paired game analyzer.
"""
import argparse
from collections import Counter
import gzip
import hashlib
import json
import math
from pathlib import Path
import random
import statistics

from summarize_balance import (FACTIONS, compact, paired, ratio_interval, read,
                               summarize, wilson_interval)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(value, indent=2, allow_nan=False) + "\n"
    if path.suffix == ".gz":
        with path.open("wb") as raw:
            with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as out:
                out.write(text.encode())
    else:
        path.write_text(text)


def chi_square_tail(value, degrees):
    """Chi-square survival function for integer degrees, via gamma recurrence."""
    if degrees < 1 or value < 0:
        raise ValueError("positive degrees and nonnegative statistic required")
    x = value / 2
    shape = 0.5 if degrees % 2 else 1.0
    result = math.erfc(math.sqrt(x)) if degrees % 2 else math.exp(-x)
    while shape < degrees / 2:
        result += math.exp(-x) * x ** shape / math.gamma(shape + 1)
        shape += 1
    return min(1.0, result)


def homogeneity_statistic(cells, n, m):
    return sum((x - n * (x + y) / (n + m)) ** 2 / (n * (x + y) / (n + m))
               + (y - m * (x + y) / (n + m)) ** 2 / (m * (x + y) / (n + m))
               for x, y in cells if x + y)


def permutation_p(cells, n, m, repetitions=10000):
    """Condition on pooled outcomes and cohort sizes, including sparse cap cells."""
    rng = random.Random(3800001)
    totals = [x + y for x, y in cells]
    population = [category for category, total in enumerate(totals) for _ in range(total)]
    observed = homogeneity_statistic(cells, n, m)
    exceeded = 0
    for _ in range(repetitions):
        heldout = Counter(rng.sample(population, m))
        resample = [(total - heldout[i], heldout[i]) for i, total in enumerate(totals)]
        exceeded += homogeneity_statistic(resample, n, m) >= observed - 1e-12
    p = (exceeded + 1) / (repetitions + 1)
    return dict(p=p, repetitions=repetitions, seed=3800001,
                monte_carlo_se=math.sqrt(p * (1 - p) / repetitions))


def cohort_comparison(tuning, heldout):
    a, b = summarize(tuning), summarize(heldout)
    n, m = a["games"], b["games"]
    rows = {}
    cells = []
    for faction in FACTIONS:
        x, y = a["factions"][faction]["wins"], b["factions"][faction]["wins"]
        p, q = x / n, y / m
        lo_a, hi_a = wilson_interval(x, n)
        lo_b, hi_b = wilson_interval(y, m)
        difference = q - p
        lower = difference - math.hypot(hi_a - p, q - lo_b)
        upper = difference + math.hypot(p - lo_a, hi_b - q)
        pooled = (x + y) / (n + m)
        se = math.sqrt(pooled * (1 - pooled) * (1 / n + 1 / m))
        rows[faction] = dict(tuning_wins=x, heldout_wins=y,
                             tuning_rate=p, tuning_wilson_95_ci=[lo_a, hi_a],
                             heldout_rate=q, heldout_wilson_95_ci=[lo_b, hi_b],
                             delta_pp=100 * difference,
                             newcombe_95_ci_pp=[100 * lower, 100 * upper],
                             unadjusted_p=math.erfc(abs(difference / se) / math.sqrt(2)) if se else 1.0)
        cells.append((x, y))
    # Each game is one multinomial draw, including caps/draws with no winner.
    cells.append((n - sum(x for x, _ in cells), m - sum(y for _, y in cells)))
    cells = [(x, y) for x, y in cells if x + y]
    statistic = homogeneity_statistic(cells, n, m)
    previous = 0.0
    for rank, faction in enumerate(sorted(FACTIONS, key=lambda f: rows[f]["unadjusted_p"])):
        previous = max(previous, min(1.0, (len(FACTIONS) - rank) * rows[faction]["unadjusted_p"]))
        rows[faction]["holm_p_within_cohort_comparison"] = previous
    return dict(tuning_games=n, heldout_games=m, factions=rows,
                pearson_chi_square=statistic, degrees_of_freedom=len(cells) - 1,
                asymptotic_p=chi_square_tail(statistic, len(cells) - 1),
                permutation=permutation_p(cells, n, m),
                caution="Exploratory comparison of already inspected seed cohorts; faction p values corrected only within this comparison, not across variants or speeds.")


def distribution(values):
    return dict(reached=len(values), mean=statistics.mean(values) if values else None,
                median=statistics.median(values) if values else None,
                minimum=min(values) if values else None, maximum=max(values) if values else None)


def telemetry_diagnosis(report):
    summary = summarize(report)
    factions = {}
    for faction in FACTIONS:
        rows = []
        for game in report["games"]:
            player = next(p for p in game["players"] if p["faction"] == faction)
            pid, telemetry = str(player["id"]), game["telemetry"]
            assert "eliminations" in game and "investments" in telemetry
            death = next((e["turn"] for e in game["eliminations"] if e["player"] == player["id"]), None)
            assert player["alive"] == (death is None)
            rows.append(dict(seed=game["seed"], player=player,
                             won=player["id"] in game["winner_players"], death=death,
                             goal=telemetry["faction_goal_turns"].get(pid),
                             score=telemetry["final_scores"]["players"][pid],
                             builds=telemetry["investments"].get(pid, {}).get("completed", {}),
                             opening=telemetry["openings"][pid]))
        n = len(rows)
        builds, incidence, sources = Counter(), Counter(), Counter()
        for row in rows:
            builds.update(row["builds"])
            incidence.update(row["builds"].keys())
            sources.update(row["score"]["live_titles"])
            sources.update(row["score"]["milestones"])
            sources["FactionGoal"] += row["score"]["faction_goals"]
        groups = {}
        for name, subset in [("wins", [r for r in rows if r["won"]]),
                             ("losses", [r for r in rows if not r["won"]])]:
            groups[name] = dict(count=len(subset),
                                alive=sum(r["player"]["alive"] for r in subset),
                                completed_goal=sum(r["goal"] is not None for r in subset),
                                vp=distribution([r["player"]["vp"] for r in subset]),
                                final_cities=distribution([r["player"]["cities"] for r in subset]))
        factions[faction] = dict(
            games=n, deaths=distribution([r["death"] for r in rows if r["death"] is not None]),
            death_games=[dict(seed=r["seed"], turn=r["death"]) for r in rows if r["death"] is not None],
            goal_turns=distribution([r["goal"] for r in rows if r["goal"] is not None]),
            final_vp=distribution([r["player"]["vp"] for r in rows]),
            final_cities=distribution([r["player"]["cities"] for r in rows]),
            mean_final_vp_by_source={k: v / n for k, v in sorted(sources.items())},
            completed_items={k: dict(total=v, mean=v / n, empires=incidence[k]) for k, v in builds.most_common()},
            openings={key: distribution([r["opening"][key] for r in rows if r["opening"].get(key) is not None])
                      for key in ["first_soldier_turn", "first_settler_turn", "second_city_turn"]},
            groups=groups,
            three_markets_completed_without_goal=sum(r["builds"].get("Building { building: Market }", 0) >= 3 and r["goal"] is None for r in rows))
    return dict(games=summary["games"], factions=factions,
                definitions="Completed production events include purchases but exclude starting/captured items. Final titles are live holdings, not cumulative awards. Deaths are PlayerEliminated events. Null death/goal at game end is censored. Build totals do not identify co-location, active status, city spacing or population at completion.")


def slice_report(report, start, stop):
    """Keep a contiguous seed subcohort and rebuild the audited count metadata."""
    games = sorted(report["games"], key=lambda game: int(game["seed"]))[start:stop]
    assert games
    wins = Counter(f for game in games for f in game["winner_factions"])
    meta = dict(report["meta"], games=len(games), first_seed=games[0]["seed"])
    return dict(meta=meta, games=games,
                summary={"by_faction": {f: dict(wins=wins[f], seats=len(games)) for f in FACTIONS}})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    diagnose = sub.add_parser("diagnose")
    diagnose.add_argument("--previous", type=Path, required=True)
    diagnose.add_argument("--full-reports", type=Path, required=True)
    diagnose.add_argument("--output", type=Path, required=True)
    analyze = sub.add_parser("analyze")
    analyze.add_argument("--report", type=Path, required=True)
    analyze.add_argument("--output", type=Path, required=True)
    analyze.add_argument("--label", required=True)
    analyze.add_argument("--baseline", type=Path)
    args = parser.parse_args()
    if args.command == "diagnose":
        comparisons, observations = {}, {}
        for variant in ["baseline", "v10a", "v10b"]:
            for speed in ["standard"]:
                paths = [args.previous / f"{variant}-{speed}-{cohort}.json.gz" for cohort in ["tuning", "heldout"]]
                result = cohort_comparison(*(read(p) for p in paths))
                result["source_sha256"] = {str(p): digest(p) for p in paths}
                if variant != "baseline":
                    effects = [paired(read(args.previous / f"baseline-{speed}-{cohort}.json.gz"), read(path))
                               for cohort, path in zip(["tuning", "heldout"], paths)]
                    result["bundle_effect_by_cohort"] = dict(zip(["tuning", "heldout"], effects))
                    interaction = {}
                    for faction in FACTIONS:
                        first, second = (effect[faction] for effect in effects)
                        delta = second["delta_pp"] - first["delta_pp"]
                        half_width = math.hypot(
                            (first["paired_normal_95_ci_pp"][1] - first["paired_normal_95_ci_pp"][0]) / 2,
                            (second["paired_normal_95_ci_pp"][1] - second["paired_normal_95_ci_pp"][0]) / 2)
                        interaction[faction] = dict(heldout_minus_tuning_effect_pp=delta,
                                                    unadjusted_normal_95_ci_pp=[delta - half_width, delta + half_width])
                    result["bundle_by_cohort_interaction"] = interaction
                comparisons[f"{variant}-{speed}"] = result
        for speed in ["standard"]:
            path = args.full_reports / f"baseline-{speed}-tuning/report.json.gz"
            if not path.exists():
                path = args.full_reports / f"baseline-{speed}-tuning.json.gz"
            observations[speed] = dict(source_report_sha256=digest(path), **telemetry_diagnosis(read(path)))
        write_json(args.output / "cohort-diagnosis.json", comparisons)
        write_json(args.output / "telemetry-diagnosis.json", observations)
    else:
        report = read(args.report)
        summary = summarize(report)
        summary["ratio_bootstrap_95_interval"] = ratio_interval(summary)
        if args.baseline:
            summary["paired_vs_baseline"] = paired(read(args.baseline), report)
        if summary["games"] == 1000 and int(summary["first_seed"]) == 3800001:
            summary["heldout_blocks"] = {}
            for label, start, stop in [("previously_seen_500", 0, 500), ("extension_500", 500, 1000)]:
                part = summarize(slice_report(report, start, stop))
                part["ratio_bootstrap_95_interval"] = ratio_interval(part)
                summary["heldout_blocks"][label] = part
        projection = compact(report, args.report)
        # Preserve diagnostics, too, so the weak-faction narrative is auditable.
        for projected, game in zip(projection["games"], report["games"], strict=True):
            projected["eliminations"] = game["eliminations"]
            projected["telemetry"]["investments"] = game["telemetry"]["investments"]
        projection["projection"]["description"] += " Includes elimination and completed-investment telemetry."
        assert summarize(projection) == summarize(report)
        write_json(args.output / f"{args.label}.json.gz", projection)
        write_json(args.output / f"{args.label}-summary.json", summary)


if __name__ == "__main__":
    main()

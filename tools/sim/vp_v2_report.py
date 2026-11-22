#!/usr/bin/env python3
"""VP v2 metrics from sim reports (merged cloud reports or local reports).

  tools/sim/vp_v2_report.py LABEL=path/to/report.json[.gz][:N] [LABEL=path ...]  (:N keeps the first N seeds) [--json out.json]

Each report holds games whose telemetry carries `vp_race` (crates/sim/src/vp_race.rs). Prints a
markdown table with one column per label, and optionally writes every number as JSON.
Standard library only.
"""
import gzip
import json
import statistics
import sys
from collections import defaultdict

SNAP = (50, 100)


def load(path):
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt") as f:
        return json.load(f)


def pct(a, b):
    return round(100.0 * a / b, 1) if b else None


def mean(xs):
    return round(sum(xs) / len(xs), 2) if xs else None


def median(xs):
    return round(statistics.median(xs), 1) if xs else None


def era_stats(games, era_count):
    """Per era: share of empires that entered it who scored, share of all empires, and per-game
    guardrails (some empire scored; every empire scored)."""
    out = {}
    for era in range(2, era_count + 1):
        entered = scored = total = 0
        games_entered = games_some = games_all = 0
        for g in games:
            vr = (g.get("telemetry") or {}).get("vp_race")
            if not vr:
                continue
            seats = vr["eras"]
            n = len(seats)
            e = s = 0
            for seat, eras in seats.items():
                total += 1
                rec = eras.get(str(era))
                if rec:
                    e += 1
                    if rec.get("scored") is not None:
                        s += 1
            entered += e
            scored += s
            if e:
                games_entered += 1
                games_some += s > 0
                games_all += s == n
        out[era] = {
            "entered_share_of_empires": pct(entered, total),
            "scored_of_entered": pct(scored, entered),
            "scored_of_all": pct(scored, total),
            "games_entered": games_entered,
            "some_scored_share": pct(games_some, games_entered),
            "all_scored_share": pct(games_all, games_entered),
        }
    return out


def select(report, first=None):
    games = [g for g in report["games"] if not g.get("panic")]
    if first:
        # Paired comparison: only the first `first` seeds (a sweep cohort's seed range).
        lo = min(int(g["seed"]) for g in games)
        games = [g for g in games if int(g["seed"]) < lo + first]
    return games


def cohort(report):
    games = report["games"]
    vr = [(g, (g.get("telemetry") or {}).get("vp_race")) for g in games]
    vr = [(g, v) for g, v in vr if v]
    n = len(games)
    won = [g for g, _ in vr if g.get("winner_team") is not None]
    out = {"games": n, "with_vp_race": len(vr)}
    lengths = [g["turns"] for g in games]
    out["median_turns"] = median(lengths)
    out["p90_turns"] = sorted(lengths)[int(0.9 * (n - 1))] if n else None
    out["win_share"] = pct(len(won), n)
    out["stalled_share"] = pct(sum(1 for g in games if g.get("winner_team") is None), n)
    # Winner leads at snapshot.
    for t in SNAP:
        ok = tot = 0
        for g, v in vr:
            if g.get("winner_team") is None or g["turns"] < t:
                continue
            leaders = v["leaders"].get(str(t))
            if leaders is None:
                continue
            tot += 1
            ok += g["winner_team"] in leaders
        out[f"leader_wins_T{t}"] = pct(ok, tot)
        out[f"leader_wins_T{t}_n"] = tot
    lc = [v["lead_changes"] for _, v in vr]
    out["lead_changes_mean"] = mean(lc)
    out["lead_changes_median"] = median(lc)
    last = [v["last_lead_change"] for g, v in vr if v["lead_changes"] > 0]
    out["last_lead_change_median"] = median(last)
    frac = [v["last_lead_change"] / g["turns"] for g, v in vr if v["lead_changes"] > 0 and g["turns"]]
    out["last_lead_change_share_of_game"] = round(100 * statistics.median(frac), 1) if frac else None
    ls = sum(v["leader_samples"] for _, v in vr)
    out["leader_crowns_mean"] = round(sum(v["leader_crowns_sum"] for _, v in vr) / ls, 2) if ls else None
    samples = sum(v["samples"] for _, v in vr)
    out["all_three_crowns_share_of_samples"] = pct(sum(v["all_crowns_samples"] for _, v in vr), samples)
    out["crown_flips_per_game"] = mean([v["crown_flips"] for _, v in vr])
    out["leader_overreach_gold_mean"] = (
        round(sum(v["leader_overreach_sum"] for _, v in vr) / ls, 2) if ls else None
    )
    gold = sum(v["leader_gold_sum"] for _, v in vr)
    out["leader_overreach_share_of_gold"] = pct(sum(v["leader_overreach_sum"] for _, v in vr), gold)
    holder_turns = sum(v["crown_holder_turns"] for _, v in vr)
    out["overreach_per_holder_turn"] = (
        round(sum(v["overreach_total"] for _, v in vr) / holder_turns, 2) if holder_turns else None
    )
    out["broke_crown_holder_share"] = pct(sum(v["broke_crown_turns"] for _, v in vr), holder_turns)
    out["upkeep_disbands_per_game"] = mean(
        [sum((g.get("telemetry") or {}).get("upkeep_disbands", {}).values()) for g in games
         if isinstance((g.get("telemetry") or {}).get("upkeep_disbands", {}), dict)]
    )
    # Power spread between the strongest and the median empire, in percent.
    for t in (50, 100, 150):
        spreads, leads, dec = [], [], []
        for _, v in vr:
            snap = (v.get("power") or {}).get(str(t))
            if snap and snap["median"] > 0 and snap["alive"] >= 2:
                spreads.append(100.0 * (snap["first"] - snap["median"]) / snap["median"])
                leads.append(snap["top_lead_percent"])
                dec.append(snap.get("top_decadence_percent", 0))
        out[f"power_spread_T{t}"] = mean(spreads)
        out[f"power_spread_mean_T{t}"] = mean(
            [100.0 * (v["power"][str(t)]["first"] - v["power"][str(t)]["mean"]) / v["power"][str(t)]["mean"]
             for _, v in vr if (v.get("power") or {}).get(str(t)) and v["power"][str(t)].get("mean", 0) > 0 and v["power"][str(t)]["alive"] >= 2]
        )
        out[f"top_lead_p50_T{t}"] = median(leads)
        out[f"top_lead_p90_T{t}"] = (
            round(sorted(leads)[int(0.9 * (len(leads) - 1))], 1) if leads else None
        )
        out[f"top_decadence_p50_T{t}"] = median(dec)
        out[f"top_decadence_p90_T{t}"] = (
            round(sorted(dec)[int(0.9 * (len(dec) - 1))], 1) if dec else None
        )
    ts = sum(v.get("top_samples", 0) for _, v in vr)
    out["top_decadence_percent_mean"] = (
        round(sum(v.get("top_decadence_percent_sum", 0) for _, v in vr) / ts, 1) if ts else None
    )
    # Proportional decadence: the highest rate any empire paid on any turn (per game), and the runner-up's mean rate.
    seen = sorted(v.get("decadence_max_seen", 0) for _, v in vr)
    out["decadence_max_seen_max"] = seen[-1] if seen else None
    out["decadence_max_seen_p50"] = median(seen)
    out["decadence_max_seen_p90"] = seen[int(0.9 * (len(seen) - 1))] if seen else None
    out["second_decadence_percent_mean"] = (
        round(sum(v.get("second_decadence_percent_sum", 0) for _, v in vr) / ts, 1) if ts else None
    )
    dt = sum(v.get("decadence_empire_turns", 0) for _, v in vr)
    out["decadence_percent_mean_when_paid"] = (
        round(sum(v.get("decadence_percent_sum", 0) for _, v in vr) / dt, 1) if dt else None
    )
    # Mean decadence per faction over every living empire-sample: is the burden spread evenly?
    fsum, fn = defaultdict(int), defaultdict(int)
    for _, v in vr:
        for f, (total, n) in (v.get("decadence_by_faction") or {}).items():
            fsum[f] += total
            fn[f] += n
    out["decadence_by_faction"] = {f: round(fsum[f] / fn[f], 2) for f in sorted(fn) if fn[f]}
    out["overreach_disbands_per_game"] = mean([v.get("overreach_disbands", 0) for _, v in vr])
    out["overreach_disbands_per_vp_leader_game"] = mean(
        [v.get("overreach_disbands_vp_leader", 0) for _, v in vr]
    )
    out["overreach_disbands_per_top_power_game"] = mean(
        [v.get("overreach_disbands_top_power", 0) for _, v in vr]
    )
    era_count = max((int(e) for _, v in vr for eras in v["eras"].values() for e in eras), default=1)
    out["eras"] = era_stats([g for g, _ in vr], era_count)
    pooled = [v for e, v in out["eras"].items() if v["games_entered"]]
    n_pairs = sum(v["games_entered"] for v in pooled)
    out["eras_pooled"] = {
        "some_scored_share": pct(sum(v["games_entered"] * (v["some_scored_share"] or 0) / 100 for v in pooled), n_pairs),
        "all_scored_share": pct(sum(v["games_entered"] * (v["all_scored_share"] or 0) / 100 for v in pooled), n_pairs),
        "scored_of_entered": pct(
            sum(v["games_entered"] * (v["scored_of_entered"] or 0) for v in pooled), n_pairs * 100
        ),
    }
    # Per-faction win share (information only).
    wins, plays = defaultdict(int), defaultdict(int)
    for g in games:
        for p in g["players"]:
            plays[p["faction"]] += 1
        for f in g.get("winner_factions") or []:
            wins[f] += 1
    out["faction_win_share"] = {f: pct(wins[f], plays[f]) for f in sorted(plays)}
    return out


ROWS = [
    ("games", "Games"),
    ("median_turns", "Median game length (turns)"),
    ("p90_turns", "p90 game length"),
    ("win_share", "Games with a winner (%)"),
    ("stalled_share", "Stalled / no winner (%)"),
    ("leader_wins_T50", "T50 VP leader goes on to win (%)"),
    ("leader_wins_T100", "T100 VP leader goes on to win (%)"),
    ("lead_changes_mean", "VP lead changes per game (mean)"),
    ("lead_changes_median", "VP lead changes per game (median)"),
    ("last_lead_change_median", "Turn of last lead change (median)"),
    ("last_lead_change_share_of_game", "...as % of game length (median)"),
    ("leader_crowns_mean", "Titles held by the VP leader (3 crowns in v2, 5 titles legacy)"),
    ("all_three_crowns_share_of_samples", "Samples where one team holds every title in play (%)"),
    ("crown_flips_per_game", "Crown flips per game"),
    ("leader_overreach_gold_mean", "Overreach paid by the VP leader (Gold/turn)"),
    ("leader_overreach_share_of_gold", "...as % of its city Gold (%)"),
    ("overreach_per_holder_turn", "Overreach per crown-holder turn (Gold)"),
    ("broke_crown_holder_share", "Crown-holder turns with an empty treasury (%)"),
    ("upkeep_disbands_per_game", "Units disbanded for unpaid upkeep per game"),
    ("overreach_disbands_per_game", "Units deserted for unpaid overreach per game"),
    ("overreach_disbands_per_vp_leader_game", "...of the VP leader's team, per game"),
    ("overreach_disbands_per_top_power_game", "...of the top-Power empire, per game"),
    ("power_spread_T50", "Top Power over median Power at T50 (%)"),
    ("power_spread_T100", "Top Power over median Power at T100 (%)"),
    ("power_spread_T150", "Top Power over median Power at T150 (%)"),
    ("power_spread_mean_T100", "Top Power over mean Power at T100 (%)"),
    ("power_spread_mean_T150", "Top Power over mean Power at T150 (%)"),
    ("top_lead_p50_T100", "Top empire's lead at T100, median (%)"),
    ("top_lead_p90_T100", "Top empire's lead at T100, p90 (%)"),
    ("top_decadence_p50_T100", "Top empire's decadence at T100, median (% of every yield)"),
    ("top_decadence_p90_T100", "Top empire's decadence at T100, p90 (% of every yield)"),
    ("top_decadence_p50_T150", "Top empire's decadence at T150, median (% of every yield)"),
    ("top_decadence_percent_mean", "Top empire's decadence, mean over samples (%)"),
    ("second_decadence_percent_mean", "Runner-up's decadence, mean over samples (%)"),
    ("decadence_max_seen_p50", "Highest decadence rate seen in a game, median (%)"),
    ("decadence_max_seen_p90", "Highest decadence rate seen in a game, p90 (%)"),
    ("decadence_max_seen_max", "Highest decadence rate seen in any game (%)"),
    ("decadence_percent_mean_when_paid", "Decadence paid by empires above the median, mean (%)"),
    ("top_lead_p50_T150", "Top empire's lead at T150, median (%)"),
    ("top_lead_p90_T150", "Top empire's lead at T150, p90 (%)"),
]


def table(results):
    labels = list(results)
    lines = ["| Metric | " + " | ".join(labels) + " |", "|---|" + "---:|" * len(labels)]
    for key, name in ROWS:
        lines.append(f"| {name} | " + " | ".join(str(results[l].get(key)) for l in labels) + " |")
    for key, name in (
        ("some_scored_share", "Eras scored by at least one empire, pooled (%)"),
        ("all_scored_share", "Eras scored by every empire, pooled (%)"),
    ):
        lines.append(f"| {name} | " + " | ".join(str(results[l]["eras_pooled"][key]) for l in labels) + " |")
    eras = sorted({e for l in labels for e in results[l]["eras"]})
    for e in eras:
        for key, name in (
            ("scored_of_entered", "scored (of empires that entered)"),
            ("scored_of_all", "scored (of all empires)"),
            ("some_scored_share", "some empire scored (% of games that reached it)"),
            ("all_scored_share", "every empire scored (% of games that reached it)"),
        ):
            lines.append(
                f"| Era {e} {name} | "
                + " | ".join(str(results[l]["eras"].get(e, {}).get(key)) for l in labels)
                + " |"
            )
    return "\n".join(lines)


def main():
    args = sys.argv[1:]
    out_json = None
    if "--json" in args:
        i = args.index("--json")
        out_json = args[i + 1]
        del args[i : i + 2]
    if not args:
        sys.exit(__doc__)
    results = {}
    for a in args:
        label, _, spec = a.partition("=")
        # "pathA[:N]+pathB[:N]" pools several reports (e.g. the 4- and 8-empire lobbies).
        games = []
        for part in spec.split("+"):
            path, first = part, None
            if ":" in part:
                path, _, n = part.rpartition(":")
                first = int(n)
            games.extend(select(load(path), first))
        results[label] = cohort({"games": games})
    print(table(results))
    if out_json:
        with open(out_json, "w") as f:
            json.dump(results, f, indent=1, sort_keys=True)


if __name__ == "__main__":
    main()

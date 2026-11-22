#!/usr/bin/env python3
"""VP pacing metrics from `tools/sim/vp-pacing.sh` output.

  tools/sim/vp_pacing_report.py DIR [--json out.json] [--arms a,b,c]

DIR holds one directory per arm: the sim report (*.json) and air-usage.txt. Prints one markdown table
with a column per arm (game length, ending era, Modern-era time, air usage, race and VP-source mix).
Standard library only.
"""
import glob
import json
import os
import re
import statistics
import sys

# era_entries keys are the 1-based era entered minus one: key "5" is Modern (era 6), "6" the Future (era 7).
# Flight is a Modern technology (research_turns, per empire).


ERA_NAMES = {1: "Ancient", 2: "Classical", 3: "Medieval", 4: "Renaissance", 5: "Industrial", 6: "Modern", 7: "Future"}


def pctile(xs, p):
    xs = sorted(xs)
    return xs[min((p * (len(xs) - 1) + 50) // 100, len(xs) - 1)] if xs else None


def race_metrics(games):
    races = [(g, (g.get("telemetry") or {}).get("vp_race")) for g in games]
    races = [(g, race) for g, race in races if race]
    n = len(games)
    pct = lambda count, total: round(100 * count / total, 1) if total else None
    out = {
        "win_share": pct(sum(g.get("winner_team") is not None for g in games), n),
        "stalled_share": pct(sum(g.get("winner_team") is None for g in games), n),
        "lead_changes_mean": round(statistics.mean(r["lead_changes"] for _, r in races), 2) if races else None,
        "upkeep_disbands_per_game": round(statistics.mean(
            sum((g.get("telemetry") or {}).get("upkeep_disbands", {}).values()) for g in games), 2) if games else None,
    }
    for turn in (50, 100):
        eligible = [(g, r["leaders"].get(str(turn))) for g, r in races
                    if g.get("winner_team") is not None and g["turns"] >= turn]
        eligible = [(g, leaders) for g, leaders in eligible if leaders is not None]
        out[f"leader_wins_T{turn}"] = pct(sum(g["winner_team"] in leaders for g, leaders in eligible), len(eligible))
    changed = [r["last_lead_change"] / g["turns"] for g, r in races
               if r["lead_changes"] > 0 and g["turns"]]
    out["last_lead_change_share_of_game"] = round(100 * statistics.median(changed), 1) if changed else None
    samples = sum(r["leader_samples"] for _, r in races)
    out["leader_crowns_mean"] = round(sum(r["leader_crowns_sum"] for _, r in races) / samples, 2) if samples else None
    entered = scored = 0
    for _, race in races:
        for eras in race["eras"].values():
            for era, record in eras.items():
                if int(era) >= 2:
                    entered += 1
                    scored += record.get("scored") is not None
    out["objective_scored_of_entered"] = pct(scored, entered)
    return out


def arm_metrics(d):
    rep = json.load(open(glob.glob(os.path.join(d, "*.json"))[0]))
    games = [g for g in rep["games"] if not g.get("panic")]
    out = {"games": len(games)}
    base = race_metrics(games)
    for k in ("win_share", "stalled_share", "leader_wins_T50", "leader_wins_T100", "lead_changes_mean",
              "last_lead_change_share_of_game", "leader_crowns_mean",
              "upkeep_disbands_per_game"):
        out[k] = base.get(k)
    out["objective_scored_of_entered"] = base["objective_scored_of_entered"]
    lengths = [g["turns"] for g in games]
    out["turns_mean"] = round(statistics.mean(lengths), 1)
    for p in (10, 50, 90):
        out[f"turns_p{p}"] = pctile(lengths, p)
    out["turns_over_300"] = round(100 * sum(t > 300 for t in lengths) / len(lengths), 1)
    # World era at the end: the highest era any empire entered (named by 1-based era).
    ends, flight_to_end = {}, []
    for g in games:
        t = g.get("telemetry") or {}
        ee = t.get("era_entries") or {}
        top = max((int(e) for seat in ee.values() for e in seat), default=0) + 1
        ends[top] = ends.get(top, 0) + 1
        flights = [r["Flight"] for r in (t.get("research_turns") or {}).values() if "Flight" in r]
        if flights:
            flight_to_end.append(g["turns"] - min(flights))
    n = len(games)
    out["ends_in_era"] = {ERA_NAMES.get(e, e): round(100 * c / n, 1) for e, c in sorted(ends.items())}
    out["flight_to_end_p10"] = pctile(flight_to_end, 10)
    out["flight_to_end_p50"] = pctile(flight_to_end, 50)
    out["flight_to_end_p90"] = pctile(flight_to_end, 90)
    out["flight_to_end_ge60"] = round(100 * sum(t >= 60 for t in flight_to_end) / n, 1)
    out["flight_to_end_ge30"] = round(100 * sum(t >= 30 for t in flight_to_end) / n, 1)
    # Winner's VP mix at the end (winning team's final sources).
    mix = {"era": [], "goal": [], "great_work": [], "crowns": [], "total": []}
    for g in games:
        t = g.get("telemetry") or {}
        w = g.get("winner_team")
        fs = (t.get("final_scores") or {}).get("teams", {}).get(str(w)) if w is not None else None
        if not fs:
            continue
        mix["era"].append(fs["era_objectives"])
        mix["goal"].append(fs["faction_goals"])
        mix["great_work"].append(fs["great_works"])
        mix["crowns"].append(sum(fs["live_titles"].values()))
        mix["total"].append(fs["total"])
    for k, v in mix.items():
        out[f"winner_{k}_mean"] = round(statistics.mean(v), 2) if v else None
    # Holds
    started = broken = completed = 0
    for g in games:
        vh = (g.get("telemetry") or {}).get("victory_holds") or {}
        started += vh.get("started", 0)
        broken += vh.get("broken", 0)
        completed += vh.get("completed", 0)
    out["hold_break_share"] = round(100 * broken / max(1, broken + completed), 1)
    out["holds_per_game"] = round(started / n, 2)
    # Air usage (text from examples/air_usage.rs)
    txt = open(os.path.join(d, "air-usage.txt")).read()

    def grab(rx, cast=float):
        m = re.search(rx, txt)
        return cast(m.group(1)) if m else None

    out["air_flight_known_pct"] = grab(r"knew Flight: \d+ of \d+ \(([\d.]+)%")
    out["air_flight_median_turn"] = grab(r"first turn (\d+)\n", int)
    out["air_bomber_struck_pct"] = grab(r"Bomber: .*?strike ordered in \d+ of \d+ \(([\d.]+)%")
    out["air_bomber_strike_orders"] = grab(r"Bomber: .*?, (\d+) strike orders", int)
    out["air_fighter_struck_pct"] = grab(r"Fighter: .*?strike ordered in \d+ of \d+ \(([\d.]+)%")
    out["air_fighter_strike_orders"] = grab(r"Fighter: .*?, (\d+) strike orders", int)
    out["air_any_strike_pct"] = grab(r"any aircraft strike ordered in \d+ of \d+ \(([\d.]+)%")
    out["air_first_strike_median_turn"] = grab(r"any aircraft strike ordered in .*?median first turn (\d+)", int)
    out["air_strikes_landed_per_game"] = None
    landed = re.findall(r"(\d+) strikes landed\)", txt)
    out["air_strikes_landed_per_game"] = round(sum(int(x) for x in landed) / n, 1)
    out["air_interceptions"] = grab(r"\(total (\d+)\)", int)
    return out


ROWS = [
    ("games", "Games"),
    ("stalled_share", "No winner (%)"),
    ("turns_p10", "Game length p10"),
    ("turns_p50", "Game length median"),
    ("turns_p90", "Game length p90"),
    ("turns_mean", "Game length mean"),
    ("turns_over_300", "Games longer than 300 turns (%)"),
    ("ends_in_era", "Highest era entered by any empire when the game ends (%)"),
    ("air_flight_known_pct", "Games where some empire learns Flight (%)"),
    ("air_flight_median_turn", "Flight first learned, median turn"),
    ("flight_to_end_p10", "Turns from the first Flight to the end of the game, p10"),
    ("flight_to_end_p50", "...median"),
    ("flight_to_end_p90", "...p90"),
    ("flight_to_end_ge30", "Games with 30+ turns after Flight (%)"),
    ("flight_to_end_ge60", "Games with 60+ turns after Flight (%)"),
    ("air_first_strike_median_turn", "First aircraft strike, median turn"),
    ("air_any_strike_pct", "Games with an aircraft strike (%)"),
    ("air_bomber_strike_orders", "Bomber strike orders (all games)"),
    ("air_fighter_strike_orders", "Fighter strike orders (all games)"),
    ("air_strikes_landed_per_game", "Aircraft strikes landed per game"),
    ("air_interceptions", "Fighter interceptions (all games)"),
    ("leader_wins_T50", "T50 VP leader goes on to win (%)"),
    ("leader_wins_T100", "T100 VP leader goes on to win (%)"),
    ("lead_changes_mean", "VP lead changes per game"),
    ("last_lead_change_share_of_game", "Last lead change, % of game (median)"),
    ("hold_break_share", "Holds broken (% of resolved holds)"),
    ("holds_per_game", "Holds started per game"),
    ("objective_scored_of_entered", "Era objectives scored (% of empires entering the era)"),
    ("winner_total_mean", "Winner's VP at the end"),
    ("winner_era_mean", "...from era objectives"),
    ("winner_goal_mean", "...from the faction goal"),
    ("winner_great_work_mean", "...from the Great Work"),
    ("winner_crowns_mean", "...from crowns"),
    ("upkeep_disbands_per_game", "Units lost to unpaid upkeep per game"),
]


def main():
    args = sys.argv[1:]
    d = args[0]
    arms = sorted(a for a in os.listdir(d) if os.path.isdir(os.path.join(d, a)))
    if "--arms" in args:
        arms = args[args.index("--arms") + 1].split(",")
    res = {a: arm_metrics(os.path.join(d, a)) for a in arms}
    if "--json" in args:
        json.dump(res, open(args[args.index("--json") + 1], "w"), indent=1)
    print("| | " + " | ".join(arms) + " |")
    print("|---|" + "---:|" * len(arms))
    for key, label in ROWS:
        cells = []
        for a in arms:
            v = res[a].get(key)
            if isinstance(v, dict):
                v = " / ".join(f"{e}:{x}" for e, x in v.items())
            cells.append("-" if v is None else str(v))
        print(f"| {label} | " + " | ".join(cells) + " |")


if __name__ == "__main__":
    main()

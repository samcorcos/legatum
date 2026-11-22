#!/usr/bin/env python3
"""Victory-hold and game-length summary of a sim report, recomputed from its `games`.

  tools/cloud/hold_summary.py <report.json[.gz] | merged.json.gz>   # prints a markdown table
  tools/cloud/hold_summary.py --json <report>                       # prints the summary as JSON

A team wins by VP only after holding its VP target for N consecutive turn ends (10 Standard,
10 turns). The per-game facts are in games[].telemetry.victory_holds (absent in older reports and
on panics; those games are counted as excluded, never as zero holds). merge.py calls
`summarize` so a merged cloud batch carries `summary.victory_holds` and
`summary.length_by_outcome` computed over every slice's games; the same function reproduces the
single-run `summary.victory_holds` written by legatum-sim (same nearest-rank percentile).
Standard library only.
"""
import gzip
import json
import sys

OUTCOMES = ("vp", "conquest", "turn_limit", "draw", "stalled", "panic")


def load(path):
    opener = gzip.open if str(path).endswith(".gz") else open
    with opener(path, "rt") as f:
        return json.load(f)


def percentile(sorted_values, p):
    """Nearest rank, the simulator's convention: index = (p * (n - 1) + 50) // 100."""
    if not sorted_values:
        return None
    return sorted_values[min((p * (len(sorted_values) - 1) + 50) // 100, len(sorted_values) - 1)]


def turn_stats(values):
    values = sorted(values)
    if not values:
        return None
    return {
        "n": len(values),
        "mean": sum(values) / len(values),
        "median": percentile(values, 50),
        "min": values[0],
        "max": values[-1],
    }


def ratio(n, d):
    return n / d if d else 0.0


def summarize(games):
    s = {
        "observed_games": 0,
        "excluded_games_without_holds": 0,
        "turns_needed": None,
        "games_with_hold": 0,
        "started": 0,
        "broken": 0,
        "completed": 0,
        "unfinished": 0,
        "hold_wins": 0,
        "comebacks": 0,
        "wins_not_first_to_reach": 0,
    }
    reach_to_win, final_to_win, reach, win = [], [], [], []
    lengths = {o: [] for o in OUTCOMES}
    wins_by_faction = {}
    for g in games:
        outcome = g.get("outcome")
        if outcome != "panic":
            lengths.setdefault(outcome, []).append(g.get("turns", 0))
        if outcome != "panic":
            for faction in g.get("winner_factions") or []:
                wins_by_faction[faction] = wins_by_faction.get(faction, 0) + 1
        h = (g.get("telemetry") or {}).get("victory_holds")
        if h is None:
            s["excluded_games_without_holds"] += 1
            continue
        s["observed_games"] += 1
        if s["turns_needed"] is None:
            s["turns_needed"] = h["turns_needed"]
        s["games_with_hold"] += h["started"] > 0
        for k in ("started", "broken", "completed", "unfinished"):
            s[k] += h[k]
        w = h.get("hold_win")
        if outcome != "vp" or w is None:
            continue
        s["hold_wins"] += 1
        s["comebacks"] += w["earlier_broken_holds"] > 0
        s["wins_not_first_to_reach"] += not w["first_to_reach"]
        reach_to_win.append(max(0, w["win_turn"] - w["first_reached_turn"]))
        final_to_win.append(max(0, w["win_turn"] - w["final_hold_start_turn"]))
        reach.append(w["first_reached_turn"])
        win.append(w["win_turn"])
    s["break_rate"] = ratio(s["broken"], s["started"])
    s["break_rate_resolved"] = ratio(s["broken"], s["broken"] + s["completed"])
    s["first_reach_to_win"] = turn_stats(reach_to_win)
    s["final_hold_to_win"] = turn_stats(final_to_win)
    s["first_reach_turn"] = turn_stats(reach)
    s["win_turn"] = turn_stats(win)
    by_outcome = {o: turn_stats(v) for o, v in lengths.items() if v}
    total_wins = sum(wins_by_faction.values())
    share = {f: n / total_wins for f, n in sorted(wins_by_faction.items())} if total_wins else {}
    return {"victory_holds": s, "length_by_outcome": by_outcome, "win_share_by_faction": share}


def fmt(stats):
    if not stats:
        return "-"
    return f"{stats['mean']:.1f} / {stats['median']} / {stats['min']}-{stats['max']}"


def render(result, n_games):
    s = result["victory_holds"]
    out = [f"{n_games} games", "", "| Metric | Value |", "| --- | --- |"]
    total = sum(v["n"] for v in result["length_by_outcome"].values())
    mix = ", ".join(f"{o} {v['n']}" for o, v in result["length_by_outcome"].items())
    out.append(f"| Games finished or stopped (non-panic) | {total} ({mix}) |")
    out.append(f"| Hold turns needed | {s['turns_needed']} |")
    out.append(
        f"| Holds started / broken / completed / unfinished | {s['started']} / {s['broken']} / "
        f"{s['completed']} / {s['unfinished']} |"
    )
    out.append(
        f"| Break rate (broken / started; resolved only) | {100 * s['break_rate']:.1f}%; "
        f"{100 * s['break_rate_resolved']:.1f}% |"
    )
    out.append(f"| Hold wins | {s['hold_wins']} |")
    out.append(f"| First reach to win, turns (mean / median / min-max) | {fmt(s['first_reach_to_win'])} |")
    out.append(f"| Final hold start to win, turns | {fmt(s['final_hold_to_win'])} |")
    out.append(f"| Comebacks (winner broke an earlier hold) | {s['comebacks']} |")
    out.append(f"| Wins by a team not first to reach its target | {s['wins_not_first_to_reach']} |")
    out += ["", "| Outcome | Games | Length mean / median / min-max |", "| --- | --- | --- |"]
    for o, v in result["length_by_outcome"].items():
        out.append(f"| {o} | {v['n']} | {fmt(v)} |")
    if result["win_share_by_faction"]:
        out += ["", "| Faction | Share of winning seats |", "| --- | --- |"]
        for f, share in result["win_share_by_faction"].items():
            out.append(f"| {f} | {100 * share:.1f}% |")
    return "\n".join(out)


def main():
    args = [a for a in sys.argv[1:] if a != "--json"]
    if len(args) != 1:
        sys.exit(__doc__)
    games = load(args[0])["games"]
    result = summarize(games)
    if "--json" in sys.argv:
        print(json.dumps(result, indent=2))
    else:
        print(render(result, len(games)))


if __name__ == "__main__":
    main()

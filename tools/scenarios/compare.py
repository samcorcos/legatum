#!/usr/bin/env python3
"""Compare a human scenario run with the AI baseline and the scenario's predictions.

Usage:
  tools/scenarios/compare.py <run.json> [--baseline <id>-baseline.json] [--scenario scenarios/<id>.json]

The run file is what the browser downloads when a scenario ends. The baseline comes from
`legatum-sim scenario scenarios/<id>.json --runs 50 --difficulty hard` and defaults to
docs/sim-results/scenarios/<id>-baseline.json. Prints:
  1. the human score against the AI's score distribution;
  2. decision differences on the turns the baseline recorded (orders by unit, city, research and
     diplomacy), plus the key moments: first war, builds and unit targets;
  3. how the human and the AI measured up against the predictions written before play.
"""

from __future__ import annotations

import argparse
import statistics
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from scenario_lib import (  # noqa: E402
    DIPLOMATIC, REPO, World, decode_state, describe, find_scenario, is_economy, load_json, percentile_rank,
)


def header(title: str) -> None:
    print(f"\n## {title}\n")


def turns_word(n: int) -> str:
    return f"{n} turn{'' if n == 1 else 's'}"


def median(values: list[int]) -> str:
    """The median as a plain number ("350", or "352.5" between two runs)."""
    if not values:
        return "-"
    m = statistics.median(values)
    return str(int(m)) if m == int(m) else str(m)


# Orders that keep a unit where it is; a unit on a standing order needs no new one each turn.
HOLDING = {"fortify", "sleep", "skip", "heal"}


def score_section(run: dict, base: dict, scenario: dict) -> None:
    results = base["results"]
    scores = [r["score"] for r in results]
    human = run["score"]
    outcome = run["outcome"]["outcome"] if run.get("outcome") else "unfinished"
    s = base["scores"]
    header("Score")
    print(f"Human: {human} ({outcome}: {run['outcome']['reason'] if run.get('outcome') else 'no result'})")
    print(f"Hard AI over {len(scores)} runs: p10 {s['p10']}, median {s['median']}, p90 {s['p90']} "
          f"(min {s['min']}, max {s['max']}); objective met in {base['win_percent']}%")
    rank = percentile_rank(scores, human)
    better = sum(1 for x in scores if human > x)
    worse = sum(1 for x in scores if human < x)
    print(f"The human scored above {better} of {len(scores)} AI runs, below {worse} "
          f"(percentile {rank}).")
    if human > s["p90"]:
        print("=> The human beat almost every AI run: an AI improvement target. See the decisions below.")
    elif human < s["p10"]:
        print("=> The AI beat the human in almost every run.")
    # Same-seed comparison: run 0 used the scenario's own seed, as the human game did.
    zero = next((r for r in results if r["run"] == 0), None)
    if zero:
        print(f"AI run 0 (the human's own dice): {zero['score']} ({zero['outcome']}: {zero['reason']})")
    # Score lines side by side: the human against the AI's median value of each line.
    lines = run["outcome"]["score_lines"] if run.get("outcome") else []
    if lines:
        print("\n| Score line | Human | AI median |\n|---|---|---|")
        for i, line in enumerate(lines):
            ai = [r["score_lines"][i]["points"] for r in results if i < len(r["score_lines"])]
            print(f"| {line['label']} | {line['points']} | {median(ai)} |")
    # Score by turn: the human against the AI's median at the same game turn.
    by_turn: dict[int, list[int]] = defaultdict(list)
    for r in results:
        for p in r["timeline"]:
            by_turn[p["turn"]].append(p["score"])
    human_turns = [(run["initial"]["turn"], run["initial"]["score"])] + [(t["turn"] + 1, t["score"]) for t in run["turns"]]
    print("\nScore by turn (after each turn): " + ", ".join(
        f"T{turn} {score} vs AI {median(by_turn.get(turn, []))}" for turn, score in human_turns))


def order_table(world: World, orders: list[dict]) -> dict[str, list[str]]:
    table: dict[str, list[str]] = defaultdict(list)
    for o in orders:
        actor, action = describe(world, o)
        table[actor].append(action)
    return table


def decisions_section(run: dict, base: dict, world: World) -> None:
    header("Decisions, turn by turn")
    results = base["results"]
    n = len(results)
    ai_turns: dict[int, list[list[dict]]] = defaultdict(list)
    for r in results:
        for t in r["decisions"]:
            ai_turns[t["turn"]].append(t["orders"])
    compared = 0
    for t in run["turns"]:
        turn = t["turn"]
        if turn not in ai_turns:
            continue
        compared += 1
        print(f"### Turn {turn}\n")
        human = order_table(world, t["orders"])
        ai: dict[str, Counter] = defaultdict(Counter)
        for orders in ai_turns[turn]:
            for actor, actions in order_table(world, orders).items():
                for a in actions:
                    ai[actor][a] += 1
        for actor in sorted(set(human) | set(ai)):
            mine = human.get(actor, [])
            theirs = ai.get(actor, Counter())
            common = ", ".join(f"{a} ({c}/{n})" for a, c in theirs.most_common(3)) or "no order"
            if mine:
                agree = all(theirs.get(a, 0) * 2 > n for a in mine)
                mark = "same" if agree else "DIFFERENT"
                rare = [a for a in mine if theirs.get(a, 0) == 0]
                note = f" — the AI never did: {'; '.join(rare)}" if rare else ""
                print(f"- [{mark}] **{actor}**: human {'; '.join(mine)} | AI {common}{note}")
            else:
                top = theirs.most_common(1)
                mark = "same" if not top or top[0][0] in HOLDING else "DIFFERENT"
                print(f"- [{mark}] **{actor}**: human gave no new order (standing orders) | AI {common}")
        print()
    if not compared:
        print("No turns in common with the baseline's recorded decision turns.")


def key_moments(run: dict, base: dict, world: World) -> None:
    header("Key moments")
    results = base["results"]
    n = len(results)
    human_war = next((t["turn"] for t in run["turns"] if any(o["type"] == "DeclareWar" for o in t["orders"])), None)
    ai_wars = sorted(r["first_war_turn"] for r in results if r["first_war_turn"] is not None)
    ai_war_text = (f"{len(ai_wars)}/{n} runs, median turn {median(ai_wars)}" if ai_wars else f"never (0/{n} runs)")
    print(f"- First war declared: human {'turn ' + str(human_war) if human_war else 'never'} | AI {ai_war_text}")
    # Builds and research: the human's economy orders against what the AI ordered on the same turns.
    human_eco = [(t["turn"], o) for t in run["turns"] for o in t["orders"] if is_economy(o)]
    ai_eco: dict[int, Counter] = defaultdict(Counter)
    for r in results:
        for t in r["economy"]:
            for o in t["orders"]:
                actor, action = describe(world, o)
                ai_eco[t["turn"]][f"{actor}: {action}"] += 1
    if human_eco:
        print("- Builds and research (human, with how many AI runs did the same that turn):")
        for turn, o in human_eco:
            actor, action = describe(world, o)
            key = f"{actor}: {action}"
            print(f"  - T{turn} {key} — AI {ai_eco[turn].get(key, 0)}/{n}")
    else:
        print("- Builds and research: the human changed none.")
    first_ai = Counter()
    for r in results:
        for t in r["economy"][:3]:
            for o in t["orders"]:
                actor, action = describe(world, o)
                first_ai[f"T{t['turn']} {actor}: {action}"] += 1
    if first_ai:
        print("- The AI's most common early builds: " + "; ".join(f"{k} ({c}/{n})" for k, c in first_ai.most_common(5)))
    # Unit targets: where the human sent units, against where the AI sent the same units.
    human_moves = [(t["turn"], o) for t in run["turns"] for o in t["orders"] if o["type"] in ("MoveUnit", "MoveUnitPath", "RangedAttack")]
    if human_moves:
        print("- Unit targets (human, and the AI's most common target for that unit over the window):")
        ai_targets: dict[int, Counter] = defaultdict(Counter)
        for r in results:
            for t in r["decisions"]:
                for o in t["orders"]:
                    if o["type"] in ("MoveUnit", "MoveUnitPath", "RangedAttack"):
                        ai_targets[o["unit"]][describe(world, o)[1]] += 1
        for turn, o in human_moves[:12]:
            actor, action = describe(world, o)
            top = ai_targets[o["unit"]].most_common(1)
            print(f"  - T{turn} {actor}: {action} | AI: {top[0][0] + f' ({top[0][1]} orders across runs)' if top else 'no move'}")
    human_dip = [(t["turn"], o) for t in run["turns"] for o in t["orders"] if o["type"] in DIPLOMATIC]
    for turn, o in human_dip:
        print(f"- Diplomacy T{turn}: {describe(world, o)[1]}")


def predictions_section(run: dict, base: dict, scenario: dict) -> None:
    header("Against the predictions")
    p = scenario["predictions"]
    s = base["scores"]
    ai = p["ai_score"]
    within = ai["low"] <= s["median"] <= ai["high"]
    print(f"- Predicted AI median {ai['median']} ({ai['low']} to {ai['high']}), measured {s['median']} — "
          f"{'within' if within else 'OUTSIDE'} the predicted range.")
    if scenario["objective"].get("win"):
        print(f"- Predicted AI objective rate {ai['win_percent']}%, measured {base['win_percent']}%.")
    print(f"- Human scored {run['score']} against the AI's predicted {ai['median']}.")
    print(f"- Predicted good play: {p['good_play']}")
    print(f"- Predicted AI play: {p['ai_play']}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("run")
    ap.add_argument("--baseline")
    ap.add_argument("--scenario")
    args = ap.parse_args()
    run = load_json(args.run)
    if run.get("format") != "legatum-scenario-run":
        print(f"{args.run} is not a scenario run", file=sys.stderr)
        return 2
    sid = run["scenario"]["id"]
    scenario = load_json(args.scenario) if args.scenario else find_scenario(sid)
    base_path = Path(args.baseline) if args.baseline else REPO / "docs" / "sim-results" / "scenarios" / f"{sid}-baseline.json"
    base = load_json(base_path)
    world = World(decode_state(run["initial_state"]))
    print(f"# {scenario['title']} (`{sid}`): human run vs Hard AI baseline\n")
    print(f"Run: {args.run}\nStarted {run['started_at']}, finished {run.get('finished_at') or 'not finished'}, {turns_word(len(run['turns']))} played.")
    print(f"Baseline: {base_path} ({base['runs']} runs, {base['created_at']}, engine {base['engine_commit'][:10]}).")
    warnings = []
    if run["scenario"]["revision"] != base["scenario"]["revision"]:
        warnings.append(f"the run is revision {run['scenario']['revision']} but the baseline measured revision {base['scenario']['revision']}")
    if run["scenario"]["state_hash"] != base["scenario"]["state_hash"]:
        warnings.append("the run and the baseline started from different states")
    if run["scenario"]["turns"] != scenario["objective"]["turns"]:
        warnings.append(f"the run used a {turns_word(run['scenario']['turns'])} window; the scenario (and baseline) use {scenario['objective']['turns']}")
    for w in warnings:
        print(f"WARNING: {w}.")
    score_section(run, base, scenario)
    decisions_section(run, base, world)
    key_moments(run, base, world)
    predictions_section(run, base, scenario)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

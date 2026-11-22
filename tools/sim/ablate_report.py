#!/usr/bin/env python3
"""Faction-rule ablation: lists a faction's rules and writes the paired report.

  python3 -I tools/sim/ablate_report.py rules <faction>
      One line per rule of the faction: `<name> <strength|drawback>` (parsed from the
      `faction_rules!` table in crates/core/src/experiments.rs, the single source).

  python3 -I tools/sim/ablate_report.py report --faction germany --control RUN \
      --arm RULE=RUN [--arm RULE=RUN ...] --out README.md [--commit SHA] [--seed S] [--games N]
      Each RUN holds slice-*/out/*.json.gz from `tools/cloud/run-batch.sh -- legatum-sim ...`.
      The paired difference, its bootstrap interval and the win rate per seat come from
      docs/sim-results/ai-quality/swiss-playbook-1105/census.py (the same maths as the census).

Verdict, for the faction's win share with the rule off against the control (paired by seed):
  strength: pays when removing it lowers the share with the 95% CI excluding 0.
  drawback: pays when removing it raises the share with the CI excluding 0.
  backwards: significant in the wrong direction.
  dead: the CI includes 0 and |delta| < 1 pp.
  inconclusive: the CI includes 0 but |delta| >= 1 pp (too few games to say).
"""

import argparse
import importlib.util
import os
import re
import statistics
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
EXPERIMENTS = os.path.join(ROOT, "crates", "core", "src", "experiments.rs")
CENSUS = os.path.join(ROOT, "docs", "sim-results", "ai-quality", "swiss-playbook-1105", "census.py")

# Faction names as the sim writes them (`players[].faction`).
FACTION_NAMES = {
    "switzerland": "Switzerland",
    "india": "India",
    "byzantium": "Byzantium",
    "germany": "Germany",
    "celts": "Celts",
    "mongols": "Mongols",
    "sparta": "Sparta",
    "athens": "Athens",
    "aztecs": "Aztecs",
    "egypt": "Egypt",
    "rome": "Rome",
}


def rule_table(path=EXPERIMENTS):
    """[(name, faction, kind)] in declaration order, from the faction_rules! macro call."""
    with open(path) as source:
        text = source.read()
    block = text.split("faction_rules! {", 1)[1].split("\n}\n", 1)[0]
    rows = re.findall(r'\w+\s*(?:=\s*\d+\s*)?=>\s*\("([a-z0-9_]+)",\s*(\w+),\s*(Strength|Drawback)\)', block)
    return [(n, f.lower(), k.lower()) for n, f, k in rows]


def census():
    spec = importlib.util.spec_from_file_location("census", CENSUS)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verdict(kind, delta, lo, hi):
    """Verdict word for a paired difference `delta` (off minus control, pp) with CI [lo, hi]."""
    if lo <= 0 <= hi:
        return "dead" if abs(delta) < 1 else "inconclusive"
    lowers = hi < 0
    return "pays" if lowers == (kind == "strength") else "backwards"


def seats(games, faction):
    """{seed: seat record} of the faction's seats (one per game at most in a census)."""
    out = {}
    for seed, g in games.items():
        for p in g["players"]:
            if p["faction"] == faction:
                out[seed] = (g, p)
    return out


def median(xs):
    return statistics.median(xs) if xs else None


def german_growth(games):
    """Mean cities and population at T60/T120 across German seats with a recorded turn."""
    rows = seats(games, "Germany")
    out = {}
    for turn in (60, 120):
        samples = []
        for game, player in rows.values():
            flow = game.get("telemetry", {}).get("empire_flow", {}).get(str(player["id"]), {})
            cities = flow.get("city_count_by_turn", {}).get(str(turn))
            population = flow.get("population_by_turn", {}).get(str(turn))
            if cities is not None and population is not None:
                samples.append((cities, population))
        out[turn] = (len(samples), *(statistics.mean(values) for values in zip(*samples))) if samples else (0, None, None)
    return out


ALWAYS_ON = {
    "swiss_tolls", "swiss_interest", "swiss_cheap_buys", "swiss_redoubt",
    "swiss_unit_upkeep", "swiss_capture_bounty", "india_food", "india_network", "india_close_cities",
    "india_corruption", "india_remittance", "byzantium_levy", "byzantium_tagmatic",
    "celts_barbarian_peace", "mongols_moves",
    "mongols_commerce", "mongols_keshig", "mongols_no_terrain",
    "sparta_veterans", "sparta_no_wounds", "sparta_kill_heal", "sparta_phalanx", "sparta_premium",
    "athens_lyceum", "athens_ruins", "athens_weak_markets", "aztecs_flower_wars",
    "aztecs_no_horses",
}


def first_applies(rule, games, faction):
    """What telemetry says about when a rule first applies, from the control run's faction seats."""
    rows = seats(games, faction)
    n = len(rows)
    if not n:
        return "n/a"
    if rule in ("germany_ruhr_production", "germany_industry_food"):
        tech = "BronzeWorking" if rule == "germany_industry_food" else "IronWorking"
        turns = [
            g["telemetry"]["research_turns"].get(str(p["id"]), {}).get(tech)
            for g, p in rows.values()
        ]
        got = [t for t in turns if t is not None]
        label = "Bronze Working" if rule == "germany_industry_food" else "Iron Working"
        return f"{label}, median T{median(got):.0f} ({100 * len(got) // n}% of seats reach it)" if got else "never researched"
    if rule == "germany_double_supply":
        return "from the first revealed strategic deposit"
    if rule.endswith("_goal") and rule != "germany_goal":
        turns = [g["telemetry"]["faction_goal_turns"].get(str(p["id"])) for g, p in rows.values()]
        got = [t for t in turns if t is not None]
        return f"first scores median T{median(got):.0f} ({100 * len(got) // n}% of seats ever hold it)" if got else "never scored"
    if rule == "byzantium_companies":
        firsts = []
        for g, p in rows.values():
            hires = [h["turn"] for h in g["telemetry"].get("company_hires", []) if h["buyer"] == p["id"]]
            if hires:
                firsts.append(min(hires))
        return f"first hire median T{median(firsts):.0f} ({100 * len(firsts) // n}% of seats hire)" if firsts else "never hired"
    if rule in ("celts_sack", "celts_no_keep"):
        sacks = [g["telemetry"].get("celts_razes", {}).get(str(p["id"]), 0) for g, p in rows.values()]
        return f"first capture; {100 * sum(1 for x in sacks if x) // n}% of seats sack at least once ({statistics.mean(sacks):.1f} per seat)"
    if rule == "mongols_no_walls":
        ends = [g["telemetry"]["era_entries"].get(str(p["id"]), {}).get("2") for g, p in rows.values()]
        got = [t for t in ends if t is not None]
        return f"T1, ends on entering the Medieval era (median T{median(got):.0f})" if got else "T1, never ends"
    if rule in ALWAYS_ON:
        return "T1 (always on)"
    if rule == "germany_goal":
        turns = [g["telemetry"]["faction_goal_turns"].get(str(p["id"])) for g, p in rows.values()]
        got = [t for t in turns if t is not None]
        return f"first scores median T{median(got):.0f} ({100 * len(got) // n}% of seats ever hold it)" if got else "never scored"
    if rule == "germany_industry_projects":
        used = [sum(g["telemetry"]["germany_project_city_turns"].values()) for g, _ in rows.values()]
        share = 100 * sum(1 for u in used if u) // n
        return f"no first-turn telemetry; used in {share}% of games ({statistics.mean(used):.1f} city-turns per game, all Germany seats)"
    return "n/a (no telemetry)"


def report(args):
    c = census()
    faction = FACTION_NAMES[args.faction]
    table = {n: (f, k) for n, f, k in rule_table()}
    control = c.load(args.control)
    arms = []
    for spec in args.arm:
        name, _, run = spec.partition("=")
        if name not in table or table[name][0] != args.faction:
            sys.exit(f"{name} is not a {args.faction} rule (see `ablate_report.py rules {args.faction}`)")
        arms.append((name, run, c.load(run)))
    control_rate, control_n = c.rate(control, faction)
    lines = [
        f"# {faction} rule ablation ({args.date})",
        "",
        "Every faction rule must move its faction's win share: switching a strength off lowers it, switching a "
        "drawback off raises it (designer, 2026-11-05). Paired census: eight-empire Hard AI-only games, the same "
        f"seeds in every arm (first seed {args.seed}), win share per {faction} seat, the difference taken "
        "per game (rule off minus control) with a 95% bootstrap interval over games "
        "([census.py](../../ai-quality/swiss-playbook-1105/census.py)). The AI reads the switched rules too, so "
        "it never plans around a rule that is off.",
        "",
        f"Engine commit `{args.commit}`; each arm {args.games} games per run; control {len(control)} games, "
        f"{faction} seated in {control_n} of them.",
        "",
        "| rule | kind | share (control to off) | delta, pp (95% CI) | first applies | verdict |",
        "|---|---|---|---|---|---|",
    ]
    notes = []
    growth_rows = []
    for name, run, games in arms:
        kind = table[name][1]
        rate_off, n_off = c.rate(games, faction)
        delta, (lo, hi), n = c.paired(control, games, faction)
        word = verdict(kind, delta, lo, hi)
        lines.append(
            f"| `{name}` | {kind} | {100 * control_rate:.2f}% to {100 * rate_off:.2f}% | "
            f"{delta:+.2f} [{lo:+.2f}, {hi:+.2f}] | {first_applies(name, control, faction)} | **{word}** |"
        )
        notes.append(f"- `{name}`: {n} paired seats, {len(games)} games, run `{run}`")
        if args.faction == "germany" and name == "germany_industry_food":
            on_growth, off_growth = german_growth(control), german_growth(games)
            for turn in (60, 120):
                on_n, on_cities, on_pop = on_growth[turn]
                off_n, off_cities, off_pop = off_growth[turn]
                growth_rows.append(
                    f"| T{turn} | {on_cities:.2f} ({on_n}) | {off_cities:.2f} ({off_n}) | "
                    f"{on_pop:.2f} ({on_n}) | {off_pop:.2f} ({off_n}) |"
                    if on_n and off_n else f"| T{turn} | n/a | n/a | n/a | n/a |"
                )
    lines += [
        "",
        "Verdicts: **pays** (a strength whose removal costs share, or a drawback whose removal raises it, "
        "with the CI excluding 0), **dead** (the CI includes 0 and |delta| < 1 pp), **backwards** "
        "(significant in the wrong direction), **inconclusive** (the CI includes 0 but |delta| is 1 pp or more: "
        "too few games). One seat per game and a win share of 7 to 20% give a 95% interval of roughly 1 to 2 pp "
        "either way at this size, so a rule worth less than that cannot be told from dead here.",
        "",
        "## Runs",
        "",
        f"- control: `{args.control}`",
        *notes,
        "",
        f"Seeds {args.seed}.. ({args.games:,} games per run), `legatum-sim --players 8 --difficulty hard`, arms differ only "
        "by `--rule-off`.",
        "",
    ]
    if growth_rows:
        lines += [
            "## German cities and population",
            "",
            "Means across German seats with a post-resolution sample; parenthesized counts are the sampled seats.",
            "",
            "| turn | cities, rule on | cities, rule off | population, rule on | population, rule off |",
            "|---|---:|---:|---:|---:|",
            *growth_rows,
            "",
        ]
    with open(args.out, "w") as f:
        f.write("\n".join(lines))
    print("\n".join(lines))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("rules")
    r.add_argument("faction")
    p = sub.add_parser("report")
    p.add_argument("--faction", required=True)
    p.add_argument("--control", required=True)
    p.add_argument("--arm", action="append", required=True, metavar="RULE=RUN")
    p.add_argument("--out", required=True)
    p.add_argument("--commit", default="unknown")
    p.add_argument("--seed", default="9311003")
    p.add_argument("--games", type=int, default=4096)
    p.add_argument("--date", default="")
    args = ap.parse_args()
    if args.cmd == "rules":
        rows = [(n, k) for n, f, k in rule_table() if f == args.faction.lower()]
        if not rows:
            valid = sorted({f for _, f, _ in rule_table()})
            sys.exit(f"no rules for faction {args.faction!r} (valid: {', '.join(valid)})")
        for n, k in rows:
            print(n, k)
    else:
        report(args)


if __name__ == "__main__":
    main()

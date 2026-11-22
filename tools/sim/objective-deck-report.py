#!/usr/bin/env python3
"""Paired era/deck experiment report from four sim JSONs and two human-seat JSONs."""
import gzip
import json
import statistics as st
import sys
from collections import Counter, defaultdict


def read(path):
    with (gzip.open(path, "rt") if path.endswith(".gz") else open(path)) as f:
        return json.load(f)


def median(rows):
    return st.median(rows) if rows else 0


def pct(n, d):
    return f"{n / d:.1%}" if d else "n/a"


def seats(g):
    return g.get("telemetry", {}).get("empire_flow", {})


def capture_count(g, seat):
    t = g.get("telemetry") or {}
    return t.get("empire_flow", {}).get(str(seat), {}).get("captures", 0) + t.get("celts_razes", {}).get(str(seat), 0)


def arm(path):
    d = read(path)
    games = d["games"]
    finished = [g for g in games if g.get("winner_team") is not None]
    leader = [(g, (g.get("telemetry") or {}).get("vp_race", {}).get("leaders", {}).get("100", [])) for g in games if g["turns"] >= 100]
    leader = [(g, l) for g, l in leader if l]
    wars = [(g.get("telemetry") or {}).get("diplomacy", {}).get("declarations", 0) for g in games]
    leads = [(g.get("telemetry") or {}).get("vp_race", {}).get("lead_changes", 0) for g in games]
    factions = Counter(p["faction"] for g in games for p in g["players"])
    wins = Counter(f for g in finished for f in g.get("winner_factions", []))
    zero = sum(not any(capture_count(g, p) for p in g.get("winner_players", [])) for g in finished)
    winner_captures = [sum(capture_count(g, p) for p in g.get("winner_players", [])) for g in finished]
    parts = defaultdict(list)
    for g in finished:
        sources = (g.get("telemetry") or {}).get("final_scores", {}).get("teams", {}).get(str(g["winner_team"]), {})
        for key, value in (("cards", sources.get("era_objectives", 0)), ("wonders", sources.get("great_works", 0)), ("goals", sources.get("faction_goals", 0)), ("crowns", sum(sources.get("live_titles", {}).values()))):
            parts[key].append(value)
    return dict(games=games, median_turn=median([g["turns"] for g in games]), outcomes=Counter(g["outcome"] for g in games),
                leader=(sum(str(g["winner_team"]) in [str(x) for x in leaders] for g, leaders in leader), len(leader)),
                leads=median(leads), wars=st.mean(wars), zero=(zero, len(finished)), winner_captures=winner_captures, factions=factions, wins=wins,
                parts={k: median(v) for k, v in parts.items()})


def holds(a):
    per_era = defaultdict(lambda: [0, 0])
    incomes = defaultdict(list)
    durations = defaultdict(list)
    for g in a["games"]:
        vr = (g.get("telemetry") or {}).get("vp_race", {})
        for seat, entries in vr.get("eras", {}).items():
            for era_s, row in entries.items():
                era = int(era_s)
                drawn = vr.get("pool_drawn", {}).get(era_s, [])
                if drawn:
                    hold_idx = drawn[0]
                    met = vr.get("pool_met", {}).get(seat, {}).get(era_s, [])
                    if len(met) > hold_idx:
                        per_era[era][1] += 1
                        per_era[era][0] += met[hold_idx] is not None and met[hold_idx] <= row["entered"] + 25
                next_entry = entries.get(str(era + 1), {}).get("entered")
                if next_entry:
                    durations[era].append(next_entry - row["entered"])
                elif era == 7:
                    durations[era].append(g["turns"] - row["entered"])
                income = vr.get("entry_gold_income", {}).get(seat, {}).get(era_s)
                if income is not None:
                    incomes[era].append(income)
    return per_era, incomes, durations


def hold_cards(a):
    rows = defaultdict(lambda: [0, 0])
    for g in a["games"]:
        vr = (g.get("telemetry") or {}).get("vp_race", {})
        for seat, entries in vr.get("eras", {}).items():
            for era_s, row in entries.items():
                drawn = vr.get("pool_drawn", {}).get(era_s, [])
                if not drawn:
                    continue
                ix = drawn[0]
                name = vr["pool_text"][era_s][ix]
                met = vr.get("pool_met", {}).get(seat, {}).get(era_s, [])
                key = (int(era_s), name)
                rows[key][1] += 1
                rows[key][0] += len(met) > ix and met[ix] is not None and met[ix] <= row["entered"] + 25
    return rows


def scored(a, within=None):
    rows = defaultdict(lambda: [0, 0])
    for g in a["games"]:
        for entries in (g.get("telemetry") or {}).get("vp_race", {}).get("eras", {}).values():
            for era_s, row in entries.items():
                era = int(era_s)
                rows[era][1] += 1
                rows[era][0] += row.get("scored") is not None and (within is None or row["scored"] <= row["entered"] + within)
    return rows


def human(path):
    d = read(path)
    maps = d["maps"]
    return round(sum(row["win_rate"] * row["n"] for row in maps)), sum(row["n"] for row in maps)


def main():
    if len(sys.argv) != 7:
        raise SystemExit("usage: objective-deck-report.py era.json deck.json era-seek0.json deck-seek0.json era-human.json deck-human.json")
    paths = sys.argv[1:]
    era, deck, era0, deck0 = map(arm, paths[:4])
    print("# Objective deck first test (2026-11-01)\n")
    print("Paired 256-game, eight-Hard cohorts use identical seeds; `objectives=era` is the unchanged default. Human-seat benchmark uses identical fixed maps and salts in both arms.\n")
    print("| Measure | Era control | Deck |\n| --- | ---: | ---: |")
    for label, fn in (("Median turns", lambda a: f'{a["median_turn"]:g}'), ("Turn-100 VP leader eventually wins", lambda a: pct(*a["leader"])), ("Median lead changes", lambda a: f'{a["leads"]:g}'), ("Mean wars declared/game", lambda a: f'{a["wars"]:.2f}'), ("Wins with zero captured rival cities", lambda a: pct(*a["zero"])), ("Median rival cities captured by winner", lambda a: f'{median(a["winner_captures"]):g}'), ("Wins with at most two captures", lambda a: pct(sum(n <= 2 for n in a["winner_captures"]), len(a["winner_captures"])))):
        print(f"| {label} | {fn(era)} | {fn(deck)} |")
    print(f"| Human-seat win rate | {pct(*human(paths[4]))} | {pct(*human(paths[5]))} |")
    print("\nOutcomes: era " + str(dict(era["outcomes"])) + "; deck " + str(dict(deck["outcomes"])) + ".\n")
    print("## Winners' VP budget (median at finish)\n")
    print("| Source | Era | Deck |\n| --- | ---: | ---: |")
    for key in ("cards", "wonders", "goals", "crowns"):
        print(f"| {key.capitalize()} | {era['parts'].get(key, 0)} | {deck['parts'].get(key, 0)} |")
    print("\nSolo target 10 VP; pair target 20 VP.\n")
    print("## Hold thresholds and Spend prices\n")
    sought, incomes, durations = holds(deck)
    accidental, _, _ = holds(deck0)
    prices = [70, 170, 270, 400, 580, 800]
    print("| Era | Hold met ≤25 turns, seek 0 | Hold met ≤25 turns, seek 1 | Spend Gold | Median entry Gold/turn | Median era turns | Price / estimated era Gold |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for era_n, price in zip(range(2, 8), prices):
        income = median(incomes[era_n]); turns = median(durations[era_n]); estimate = income * turns
        print(f"| {era_n} | {pct(*accidental[era_n])} | {pct(*sought[era_n])} | {price} | {income} | {turns} | {price / estimate:.2f}× |" if estimate > 0 else f"| {era_n} | {pct(*accidental[era_n])} | {pct(*sought[era_n])} | {price} | {income} | {turns} | n/a |")
    print("\nHold percentages are for the drawn Hold card, among empires entering that era. Spend affordability is excluded because it does not score without the Claim action.\n")
    print("| Drawn Hold card | Seek 0: met ≤25 turns | Seek 1: met ≤25 turns |\n| --- | ---: | ---: |")
    passive_cards, pursued_cards = hold_cards(deck0), hold_cards(deck)
    for key in sorted(passive_cards | pursued_cards):
        print(f"| Era {key[0]}: {key[1]} | {pct(*passive_cards[key])} | {pct(*pursued_cards[key])} |")
    print()
    print("## Accidental era points\n")
    print("| Era | Era control, seek 0 | Deck, seek 0 |\n| --- | ---: | ---: |")
    era_accidental, deck_accidental = scored(era0), scored(deck0)
    for era_n in range(2, 8):
        print(f"| {era_n} | {pct(*era_accidental[era_n])} | {pct(*deck_accidental[era_n])} |")
    print("\nAccidental scoring means a point actually awarded at any time after entry with `objective_seek=0`; it includes offensive Fight captures, while the Hold-only table above isolates the 25-turn threshold requirement.\n")
    print("## Functional faction check\n")
    print("| Faction | Era win rate (wins/seats) | Deck win rate (wins/seats) |\n| --- | ---: | ---: |")
    for faction in sorted(era["factions"] | deck["factions"]):
        print(f"| {faction} | {pct(era['wins'][faction], era['factions'][faction])} ({era['wins'][faction]}/{era['factions'][faction]}) | {pct(deck['wins'][faction], deck['factions'][faction])} ({deck['wins'][faction]}/{deck['factions'][faction]}) |")
    print("\nFaction rates are functional smoke data at this sample size, not balance evidence.\n")
    print("Raw reports: `" + "`, `".join(paths) + "`.")


if __name__ == "__main__":
    main()

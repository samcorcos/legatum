#!/usr/bin/env python3
"""Build the climb program's targets: what the user actually achieved, per faction.

    python3 tools/ai-quality/climb-targets.py [--saves DIR] [--out-dir DIR]

Reads every game in the user's bug-saves folder (read only), groups the saves into games
(same map seed = same game; the save with the latest turn wins), and writes
`targets.json` (the climber reads it) and `targets.md` (people read it) into
docs/sim-results/ai-quality/climb-program/.

Per game and checkpoint (T50/T100/T150/T200) it records his cities and VP; also captures,
cities lost, rivals eliminated, his first war, Gold at T100 and the result. Numbers come
from the archive's per-turn telemetry (exact), from the chronicle where the save has one
(8.9.4 and later), from `players[].fight.captures` (8.8.1 and later), and otherwise from
telemetry city deltas with his FoundCity orders netted out (a lower bound: razes and
same-turn swaps are invisible). Each game says which source it used.

Games dropped: snippets shorter than MIN_TURNS (bug repro saves, not games he played).
Results the save can't show (the game was abandoned before the engine declared it) live
in RESULT_OVERRIDES with the reason and the document that states it.
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CHECKPOINTS = (50, 100, 150, 200)
MIN_TURNS = 30
GENERATED = "2026-10-23"

# engine name -> the name people use
DISPLAY = {
    "Switzerland": "Switzerland", "Japan": "Japan", "Byzantium": "Byzantium", "Germany": "Germany",
    "Celts": "Celts", "Mongols": "Mongols", "Sparta": "Sparta", "Athens": "Athens",
}

# file-name prefix -> playtest version (the engine_commit of a bug save can be older than the build)
PREFIX_VERSION = [
    (r"^playtest-(8\.\d)", None), (r"^p881", "8.8.1"), (r"^p82", "8.2"), (r"^p84", "8.4"),
    (r"^p85", "8.5"), (r"^p86", "8.6"), (r"^p89(\d)", "8.9.{0}"),
]
COMMIT_VERSION = {
    "6e3780b": "8.3", "900d58c": "8.4", "d520504": "8.5", "a39d092": "8.6", "0f917ec": "8.8.1",
    "28be168": "8.9.2", "dcb4f0d": "8.9.3", "eeccf48": "8.9.4", "64ad962": "8.9.5",
    "6328686": "8.9.6", "5637d81": "8.9.7", "7bc01cd": "8.2", "faecae9": "7a",
}

# seed -> (result, why). Only for games whose save isn't marked finished but whose end the user told us.
RESULT_OVERRIDES = {
    "2602274368712031": ("loss", "8.9.2 Swiss: abandoned at T220 with the Mongols and Sparta at 8 VP to his 5; "
                                 "he said 'the AI legitimately beat me' (docs/playtests/human-reasoning.md, 2026-10-17)"),
    "5355669152530667": ("loss", "8.6 Germany vs Mongols: 'about to lose another game' at T125, second Germany loss "
                                 "(docs/playtests/human-reasoning.md, 2026-10-09; docs/learnings.md)"),
}


def zstd_json(path: Path) -> dict:
    return json.loads(subprocess.run(["zstd", "-dc", str(path)], capture_output=True, check=True).stdout)


def load_game(path: Path):
    """-> (archive, state or None). State is None for tape-only archives."""
    if path.name.endswith(".tape.json.zst"):
        return zstd_json(path), None
    outer = json.loads(path.read_text())
    state = json.loads(outer["game"])
    return state["archive"], state


def version_of(file: str, commit: str) -> str:
    for pat, fmt in PREFIX_VERSION:
        m = re.match(pat, file)
        if m:
            return m.group(1) if fmt is None else fmt.format(*m.groups())
    return COMMIT_VERSION.get(commit[:7], commit[:7])


def game_id(file: str) -> str:
    stem = file.replace(".tape.json.zst", "").replace(".legatum", "")
    return re.sub(r"(-\d{4}){1,2}$", "", stem)


def median(values):
    vals = [v for v in values if v is not None]
    if not vals:
        return None
    m = statistics.median(vals)
    return int(m) if float(m).is_integer() else round(m, 1)


def seat_frame(frame, seat):
    return next(p for p in frame["players"] if p["player"] == seat)


def analyse(file: str, archive: dict, state: dict | None) -> dict:
    meta = archive["metadata"]
    seats = meta["seats"]
    seat = next(s["player"] for s in seats if s["human"])
    me_seat = seats[seat]
    tel = archive["telemetry"]
    last_turn = tel[-1]["turn"]
    by_turn = {fr["turn"]: fr for fr in tel}
    setup = archive["tape"]["setup"]
    ai = [s for s in setup["players"] if not s.get("is_human")]
    ai_diff = sorted({s.get("difficulty", "?") for s in ai})
    searching = any(s.get("search") for s in ai)

    def at(turn, key):
        fr = by_turn.get(turn)
        return seat_frame(fr, seat)[key] if fr else None

    cities = {str(t): at(t, "cities") for t in CHECKPOINTS}
    vp = {str(t): at(t, "vp") for t in CHECKPOINTS}

    first_war = next((fr["turn"] for fr in tel if seat_frame(fr, seat)["wars"]), None)

    # captures / lost / eliminated, best source first
    chronicle = (state or {}).get("history", {}).get("chronicle") if state else None
    fight = (state["players"][seat].get("fight") or {}).get("captures") if state else None
    taken = [e for e in chronicle or [] if e["kind"] in ("Captured", "Razed")]
    if chronicle is not None:
        captures = sum(1 for e in taken if e["by"] == seat)
        lost = sum(1 for e in taken if e["from"] == seat)
        cap_src = "chronicle"
        dead_by_him = {e["player"] for e in chronicle if e["kind"] == "Eliminated"
                       and any(t["turn"] == e["turn"] and t["by"] == seat and t["from"] == e["player"] for t in taken)}
    else:
        founds = {}
        for t in archive["tape"]["turns"]:
            founds[t["turn"]] = sum(1 for o in t["orders"].get(str(seat), [])
                                    if isinstance(o, dict) and o.get("type") == "FoundCity")
        gained = dropped = 0
        for t in range(1, last_turn + 1):
            x = at(t, "cities") - at(t - 1, "cities") - founds.get(t, 0)
            gained += max(x, 0)
            dropped += max(-x, 0)
        captures, lost, cap_src = gained, dropped, "telemetry-lower-bound"
        if fight is not None:
            captures, cap_src = fight, "fight-counter"
        # a rival that died while at war with him on a turn his city count rose
        dead_by_him = set()
        for t in range(1, last_turn + 1):
            prev, cur = by_turn[t - 1], by_turn[t]
            for p in cur["players"]:
                q = p["player"]
                if q == seat or p["alive"] or not seat_frame(prev, q)["alive"]:
                    continue
                if q in seat_frame(prev, seat)["wars"] and at(t, "cities") > at(t - 1, "cities"):
                    dead_by_him.add(q)
    final = tel[-1]
    dead_any = sum(1 for p in final["players"] if p["player"] != seat and not p["alive"])

    winner = meta.get("winner")
    finished = winner is not None
    note = None
    if finished:
        result = "win" if winner == seats[seat]["team"] else "loss"
    else:
        result = "unfinished"
    win_faction = next((s["faction"] for s in seats if s["team"] == winner), None) if finished else None
    victory = None
    if finished:
        victory = {"vp": "vp-hold", "elimination": "elimination"}.get(meta["outcome"], meta["outcome"])
    if str(meta["seed"]) in RESULT_OVERRIDES and not finished:
        result, note = RESULT_OVERRIDES[str(meta["seed"])]

    return {
        "id": game_id(file), "files": [file], "version": version_of(file, meta["engine_commit"]),
        "engine_commit": meta["engine_commit"][:9], "seed": str(meta["seed"]),
        "faction": me_seat["faction"], "players": len(seats),
        "map": [meta["map"]["width"], meta["map"]["height"]],
        "ai_difficulty": ai_diff[0] if len(ai_diff) == 1 else ai_diff, "ai_search": searching,
        "opponents": [s["faction"] for s in seats if not s["human"]],
        "result": result, "last_turn": last_turn,
        "win_turn": last_turn if result == "win" else None,
        "victory": victory if result in ("win", "loss") and finished else None,
        "winner_faction": win_faction, "note": note,
        "cities": cities, "vp": vp, "final_cities": at(last_turn, "cities"), "final_vp": at(last_turn, "vp"),
        "captures": captures, "lost": lost, "captures_source": cap_src,
        "eliminated": len(dead_by_him), "eliminated_any": dead_any,
        "first_war": first_war, "gold_100": at(100, "gold"),
        "small_4p": len(seats) == 4 and [meta["map"]["width"], meta["map"]["height"]] == [40, 26],
    }


def gather(saves: Path):
    groups: dict[str, list] = {}
    for path in sorted(saves.glob("*.legatum")) + sorted(saves.glob("*.tape.json.zst")):
        try:
            archive, state = load_game(path)
            meta = archive["metadata"]
        except (json.JSONDecodeError, KeyError, subprocess.CalledProcessError):
            continue
        last = archive["telemetry"][-1]["turn"] if archive.get("telemetry") else -1
        groups.setdefault(str(meta["seed"]), []).append((last, path.name.endswith(".legatum"), path, archive, state))
    games, dropped = [], []
    for seed, members in groups.items():
        members.sort(key=lambda m: (m[0], m[1]))
        last, _, path, archive, state = members[-1]
        files = [m[2].name for m in members]
        if last < MIN_TURNS:
            dropped.append({"seed": seed, "files": files, "last_turn": last,
                            "reason": f"shorter than {MIN_TURNS} turns (bug repro save)"})
            continue
        g = analyse(path.name, archive, state)
        g["files"] = [path.name] + [f for f in files if f != path.name]
        games.append(g)
    games.sort(key=lambda g: (version_key(g["version"]), g["id"]))
    return games, dropped


def version_key(v: str):
    return [int(x) if x.isdigit() else 0 for x in re.split(r"[.]", v.replace("7a", "7.0"))]


def summarise(games: list[dict], prefer_small: bool) -> dict:
    finished = [g for g in games if g["result"] in ("win", "loss")]
    wins = [g for g in games if g["result"] == "win"]
    out = {"games": len(games), "finished": len(finished), "wins": len(wins),
           "losses": len(finished) - len(wins),
           "win_rate": round(len(wins) / len(finished), 3) if finished else None,
           "small_4p_games": sum(g["small_4p"] for g in games),
           "small_4p_wins": sum(g["small_4p"] and g["result"] == "win" for g in games)}
    small = [g for g in games if g["small_4p"]]
    basis = {}

    def metric(getter, name):
        pool = [getter(g) for g in games]
        if prefer_small and any(v is not None for v in (getter(g) for g in small)):
            pool, basis[name] = [getter(g) for g in small], "small_4p"
        else:
            basis[name] = "all"
        return median(pool), sum(v is not None for v in pool)

    n = {}
    for key in ("cities", "vp"):
        out[key] = {}
        for t in CHECKPOINTS:
            out[key][str(t)], n[f"{key}_{t}"] = metric(lambda g, t=t, key=key: g[key][str(t)], f"{key}_{t}")
    for key, getter in (("captures", lambda g: g["captures"]), ("lost", lambda g: g["lost"]),
                        ("eliminated", lambda g: g["eliminated"]), ("first_war", lambda g: g["first_war"]),
                        ("win_turn", lambda g: g["win_turn"]), ("gold_100", lambda g: g["gold_100"])):
        out[key], n[key] = metric(getter, key)
    out["n"], out["basis"] = n, basis
    if prefer_small:
        recent = [g for g in games if version_key(g["version"]) >= [8, 9]]
        out["rules_8_9_median"] = {
            "games": len(recent),
            "cities": {str(t): median(g["cities"][str(t)] for g in recent) for t in CHECKPOINTS},
            "vp": {str(t): median(g["vp"][str(t)] for g in recent) for t in CHECKPOINTS},
            "captures": median(g["captures"] for g in recent), "first_war": median(g["first_war"] for g in recent),
            "win_turn": median(g["win_turn"] for g in recent),
        }
        out["all_games_median"] = {
            "cities": {str(t): median(g["cities"][str(t)] for g in games) for t in CHECKPOINTS},
            "vp": {str(t): median(g["vp"][str(t)] for g in games) for t in CHECKPOINTS},
            "captures": median(g["captures"] for g in games), "first_war": median(g["first_war"] for g in games),
            "win_turn": median(g["win_turn"] for g in games),
        }
    return out


def build(games, dropped) -> dict:
    factions = {}
    for f in sorted({g["faction"] for g in games}):
        factions[f] = summarise([g for g in games if g["faction"] == f], prefer_small=True)
        factions[f]["name"] = DISPLAY.get(f, f)
    overall = summarise(games, prefer_small=False)
    overall["small_4p"] = summarise([g for g in games if g["small_4p"]], prefer_small=False)
    return {
        "generated": GENERATED,
        "source": "tools/ai-quality/climb-targets.py: the user's saves in ~/LegatumRuns/bug-saves grouped into games by "
                  "map seed (latest save of each game); per-turn telemetry for cities, VP, Gold and wars; chronicle "
                  "(8.9.4+), fight counter (8.8.1+) or telemetry city deltas for captures; medians over wins and losses "
                  "(unfinished games count at the checkpoints they reached); faction medians prefer 4-player Small "
                  "(40x26) games when the faction has any",
        "games": games, "dropped": dropped, "factions": factions, "overall": overall,
    }


def fmt(v):
    return "-" if v is None else str(v)


def md_summary_table(s: dict) -> list[str]:
    rows = ["| Metric | T50 | T100 | T150 | T200 |", "| --- | --- | --- | --- | --- |",
            "| Cities (median) | " + " | ".join(fmt(s["cities"][str(t)]) for t in CHECKPOINTS) + " |",
            "| VP (median) | " + " | ".join(fmt(s["vp"][str(t)]) for t in CHECKPOINTS) + " |",
            "| Games behind it (basis) | " + " | ".join(
                f"{s['n'][f'cities_{t}']} ({s['basis'][f'cities_{t}']})" for t in CHECKPOINTS) + " |"]
    return rows


def md(data: dict) -> str:
    games = data["games"]
    o = data["overall"]
    L = [
        "# Climb targets: what the user achieved, per faction", "",
        f"Generated {data['generated']} by `python3 tools/ai-quality/climb-targets.py` from his saves in "
        "`~/LegatumRuns/bug-saves` (read only). The climber reads `targets.json` next to this page.", "",
        f"He played {o['games']} games we can use ({o['finished']} finished): "
        f"**{o['wins']} wins, {o['losses']} losses, win rate {round(o['win_rate'] * 100)}%** "
        f"(finished games only; {o['games'] - o['finished']} unfinished). "
        f"{o['small_4p']['games']} were 4-player Small maps "
        f"({o['small_4p']['wins']} won of {o['small_4p']['finished']} finished). "
        "Every AI was Hard.", "",
        "Medians include wins and losses. A median uses only the games that reached that turn "
        "(unfinished and short wins have no T150 or T200). `-` means no game reached it.", "",
        "## Overall", "", *md_summary_table(o), "",
        f"Captures (median) {fmt(o['captures'])}, cities lost {fmt(o['lost'])}, first war T{fmt(o['first_war'])}, "
        f"Gold at T100 {fmt(o['gold_100'])}, win turn {fmt(o['win_turn'])} (wins only).", "",
        "4-player Small games only: "
        f"cities {'/'.join(fmt(o['small_4p']['cities'][str(t)]) for t in CHECKPOINTS)}, "
        f"VP {'/'.join(fmt(o['small_4p']['vp'][str(t)]) for t in CHECKPOINTS)} at T50/T100/T150/T200; "
        f"captures {fmt(o['small_4p']['captures'])}, first war T{fmt(o['small_4p']['first_war'])}, "
        f"win turn {fmt(o['small_4p']['win_turn'])}.", "",
        "## Per faction", "",
        "Engine name (people's name). Medians prefer his 4-player Small games when the faction has any "
        "(the basis row says which, per checkpoint, so a row can switch from 4-player to 8-player games); the rows below the table give the all-games and the 8.9.x-only medians.", "",
    ]
    order = sorted(data["factions"].items(), key=lambda kv: -kv[1]["games"])
    for f, s in order:
        L += [f"### {f} ({s['name']}): {s['wins']} won of {s['finished']} finished "
              f"({s['games']} games, {s['small_4p_games']} on 4-player Small)", ""]
        L += md_summary_table(s)
        am = s["all_games_median"]
        L.append("| Cities, all games | " + " | ".join(fmt(am["cities"][str(t)]) for t in CHECKPOINTS) + " |")
        L.append("| VP, all games | " + " | ".join(fmt(am["vp"][str(t)]) for t in CHECKPOINTS) + " |")
        rm = s["rules_8_9_median"]
        if rm["games"]:
            L.append(f"| Cities, 8.9.x games only ({rm['games']}) | " + " | ".join(fmt(rm["cities"][str(t)]) for t in CHECKPOINTS) + " |")
            L.append("| VP, 8.9.x games only | " + " | ".join(fmt(rm["vp"][str(t)]) for t in CHECKPOINTS) + " |")
        L += ["", f"Captures {fmt(s['captures'])}, lost {fmt(s['lost'])}, eliminated {fmt(s['eliminated'])}, "
                  f"first war T{fmt(s['first_war'])}, Gold at T100 {fmt(s['gold_100'])}, win turn {fmt(s['win_turn'])} "
                  f"(wins only).", ""]
    L += ["## Every game", "",
          "Result: W win, L loss, ? unfinished (last turn in brackets). Cities and VP at T50/T100/T150/T200 "
          "(`-` = the game was over or the save ended earlier). Cap = captured by him, Lost = his cities taken, "
          "Elim = rivals he eliminated, War = his first war (declared or received). An asterisk on Cap or Lost "
          "means a lower bound (older saves have no event log). A dagger on the result means the game was abandoned "
          "and the result is what he told us (see Caveats).", "",
          "| Game | Ver | Faction | Map | Result | Cities | VP | Cap | Lost | Elim | War | Gold100 |",
          "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
    for g in games:
        res = {"win": "W", "loss": "L", "unfinished": "?"}[g["result"]] + f" ({g['last_turn']})"
        if g["win_turn"] is None and g["result"] == "loss" and g["note"]:
            res += " \u2020"
        star = "*" if g["captures_source"] == "telemetry-lower-bound" else ""
        L.append(f"| {g['id']} | {g['version']} | {g['faction']} | {g['players']}p {g['map'][0]}x{g['map'][1]} | {res} | "
                 + "/".join(fmt(g["cities"][str(t)]) for t in CHECKPOINTS) + " | "
                 + "/".join(fmt(g["vp"][str(t)]) for t in CHECKPOINTS)
                 + f" | {g['captures']}{star} | {g['lost']}{star} | {g['eliminated']} | {fmt(g['first_war'])} | {fmt(g['gold_100'])} |")
    L += ["", "Files per game (the first is the save the numbers come from; the rest are earlier saves of the same game, "
          "same seed, not counted separately):", ""]
    for g in games:
        extra = f" (+ {', '.join(g['files'][1:])})" if len(g["files"]) > 1 else ""
        L.append(f"- {g['id']}: `{g['files'][0]}`{extra}")
    L += ["", "Dropped:", ""]
    for d in data["dropped"]:
        L.append(f"- {', '.join(d['files'])}: {d['reason']} (last turn {d['last_turn']})")
    notes = [f"- {g['id']}: {g['note']}" for g in games if g["note"]]
    L += ["", "## Caveats", "",
          "- **Rules changed between versions.** The games run from 8.3 to 8.9.7; the AI, decadence, eras, walls and "
          "combat all moved. The early ones (8.3 to 8.6) are 2 to 4 player and mostly won by elimination or an early VP hold; "
          "treat them as what a human can do, not as what the current rules allow.",
          "- **Bug saves are not all full games.** Some games exist only as a mid-game bug save; those count as "
          "unfinished at their last turn, so they feed the early checkpoints only and never the win rate.",
          "- **Two losses are not in the engine's win/loss flag.** He abandoned the game before the end and the result "
          "comes from what he said at the time:", *["  " + n for n in notes],
          "- **Eight-player games** (the 48x32 maps of 8.9.2 to 8.9.6 and the 8.2 and 7a saves) have more room and "
          "more rivals; only 4-player Small games match the climber's setting.",
          "- **Small samples.** Most factions have one or two games; a faction win rate of 1.0 means 1 of 1.",
          "- **Captures, losses and eliminations.** Exact from 8.9.4 (chronicle) and the fight counter from 8.8.1 "
          "(razed cities count as captured). Earlier saves use telemetry city counts with his FoundCity orders netted out: "
          "it misses razes and same-turn swaps, so those numbers are lower bounds (marked *). Eliminations before 8.9.4 "
          "are estimated (a rival that died at war with him on a turn his city count rose); `eliminated_any` in the JSON "
          "counts every dead rival.",
          "- **Unrecorded games.** A third Germany game (he was stuck at 7 VP at T170, 2026-10-09) has no save. "
          "`~/LegatumData/scenarios/*.json` start from a mid-game state and are not counted.",
          "- **The human seat's difficulty field says Normal** in every save; it is a default, not a handicap. "
          "Every AI seat was Hard (the older games also ran Hard + Search).", ""]
    return "\n".join(L)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--saves", type=Path, default=Path.home() / "LegatumRuns/bug-saves")
    ap.add_argument("--out-dir", type=Path, default=ROOT / "docs/sim-results/ai-quality/climb-program")
    args = ap.parse_args()
    games, dropped = gather(args.saves)
    data = build(games, dropped)
    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "targets.json").write_text(json.dumps(data, indent=1) + "\n")
    (args.out_dir / "targets.md").write_text(md(data))
    o = data["overall"]
    print(f"{o['games']} games ({o['finished']} finished): {o['wins']} wins, win rate {o['win_rate']}; "
          f"dropped {len(dropped)}")
    for f, s in data["factions"].items():
        print(f"  {f}: {s['wins']}/{s['finished']} of {s['games']} games, 4p-small {s['small_4p_games']}, "
              f"cities {s['cities']} vp {s['vp']}")


if __name__ == "__main__":
    main()

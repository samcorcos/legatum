#!/usr/bin/env python3
"""Print or update the last-ten designer win rate from the decision-audit archive index."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INDEX = Path.home() / "LegatumData/archive/index.jsonl"
STATUS = ROOT / "docs/status/state-of-the-ai.md"
PREFIX = "Designer vs Hard, last 10 finished games:"


def line(index: Path) -> str:
    games = []
    for raw in index.read_text().splitlines():
        game = json.loads(raw)
        seats = game.get("seats", [])
        humans = [seat for seat in seats if seat.get("human")]
        rivals = [seat for seat in seats if not seat.get("human")]
        if (game.get("source") != "human-local" or not game.get("finished_at")
                or game.get("winner") is None or len(humans) != 1 or not rivals
                or any(seat.get("difficulty") != "Hard" for seat in rivals)):
            continue
        games.append(game)
    games.sort(key=lambda game: (game["finished_at"], game["game_id"]), reverse=True)
    recent = games[:10]
    wins = sum(game["winner"] == next(seat["team"] for seat in game["seats"] if seat["human"])
               for game in recent)
    end = recent[0]["finished_at"][:10] if recent else "none"
    return (f"- **{PREFIX}** {wins}/{len(recent)} "
            f"({wins / len(recent):.0%} as of {end}; "
            "source: `~/LegatumData/archive/index.jsonl`; "
            "run `python3 tools/decision-audit/designer_win_rate.py --write` to refresh)." if recent
            else f"- **{PREFIX}** no matching games in {index}.")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--index", type=Path, default=INDEX)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    result = line(args.index)
    if args.write:
        lines = STATUS.read_text().splitlines()
        matches = [i for i, row in enumerate(lines) if row.startswith(f"- **{PREFIX}**")]
        if len(matches) > 1:
            raise SystemExit("duplicate designer win-rate lines")
        if matches:
            lines[matches[0]] = result
        else:
            lines.insert(2, result)
        STATUS.write_text("\n".join(lines) + "\n")
    print(result)


if __name__ == "__main__":
    main()

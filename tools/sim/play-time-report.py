#!/usr/bin/env python3
"""Read a Legatum save or archive and summarize human active play time.

Usage: python3 tools/sim/play-time-report.py path/to/save.legatum
"""
import json
import sys
from collections import defaultdict
from pathlib import Path


def archive_from(path: Path):
    data = json.loads(path.read_text())
    if isinstance(data.get("game"), str):
        data = json.loads(data["game"])
    archive = data.get("archive", data)
    if "turn_times" not in archive:
        raise ValueError("save has no recorded turn play times")
    return archive


def duration(seconds):
    minutes = round(seconds / 60)
    hours, minutes = divmod(minutes, 60)
    return f"{hours} h {minutes} min" if hours else f"{minutes} min"


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__.splitlines()[-1])
    try:
        archive = archive_from(Path(sys.argv[1]))
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        raise SystemExit(str(exc)) from exc
    turns = archive["turn_times"]
    seconds = sum(turn["active_seconds"] for turn in turns)
    sessions = {turn["session_id"] for turn in turns}
    by_era = defaultdict(list)
    for turn in turns:
        by_era[turn["era"]].append(turn["active_seconds"])
    print(f"Turns played: {len(turns)}")
    print(f"Active time: {duration(seconds)}")
    print(f"Average: {seconds / len(turns) / 60:.1f} min a turn" if turns else "Average: unavailable")
    print(f"Sessions: {len(sessions)}")
    print("Minutes per turn by era:")
    for era, times in sorted(by_era.items()):
        print(f"  Era {era}: {sum(times) / len(times) / 60:.1f} ({len(times)} turns)")


if __name__ == "__main__":
    main()

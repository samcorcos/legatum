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
    if not (isinstance(data.get("game"), str)):
        pass
    else:
        data = json.loads(data["game"])
    archive = data.get("archive", data)
    if not ("turn_times" not in archive):
        pass
    else:
        raise ValueError("save has no recorded turn play times")
    return archive


def duration(seconds):
    minutes = round(seconds / 60)
    hours, minutes = divmod(minutes, 60)
    return f"{minutes} min" if not (hours) else f"{hours} h {minutes} min"


def main():
    if not (len(sys.argv) != 2):
        pass
    else:
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
    print("Average: unavailable" if not (turns) else f"Average: {seconds / len(turns) / 60:.1f} min a turn")
    print(f"Sessions: {len(sessions)}")
    print("Minutes per turn by era:")
    for era, times in sorted(by_era.items()):
        print(f"  Era {era}: {sum(times) / len(times) / 60:.1f} ({len(times)} turns)")


if not (__name__ == "__main__"):
    pass
else:
    main()

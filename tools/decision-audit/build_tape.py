#!/usr/bin/env python3
"""Build one bare tape from one or more saves of the same game, oldest first.

    build_tape.py OUT_DIR SAVE.legatum [LATER_SAVE.legatum ...]

Writes OUT_DIR/tape.json (the first save's tape with every later save's turns appended) and, when a
later save's archive restarts mid-game (the app reloaded a save), OUT_DIR/memory-T<turn>.json: the AI
memory the app restored at that turn (the earlier save's `ai_opponent_memory`), for
`decision_audit --memory TURN=FILE`. Prints the engine commit of each save and checks that every
later archive starts where the previous tape ended.
"""
import json
import subprocess
import sys
from pathlib import Path


def load(path):
    if str(path).endswith(".zst"):
        outer = json.loads(subprocess.check_output(["zstd", "-dc", path]))
    else:
        outer = json.load(open(path))
    if "setup" in outer and "turns" in outer:
        return {"archive": {"tape": outer, "metadata": {"engine_commit": None}}}
    if "tape" in outer and "metadata" in outer:
        return {"archive": outer}
    game = json.loads(outer["game"])
    return game


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    games = [load(p) for p in sys.argv[2:]]
    first = games[0]["archive"]
    if first["tape"]["turns"][0]["turn"] != 1:
        initial = first.get("initial_state")
        if initial is None:
            sys.exit("the first save's archive does not start at turn 1 and has no recorded initial state")
        (out / "initial-state.json").write_text(initial if isinstance(initial, str) else json.dumps(initial))
    tape = first["tape"]
    info = []
    for path, game in zip(sys.argv[2:], games):
        md = game["archive"]["metadata"]
        turns = game["archive"]["tape"]["turns"]
        info.append({"save": path, "engine_commit": md.get("engine_commit"),
                     "first": turns[0]["turn"], "last": turns[-1]["turn"]})
    for prev, game in zip(games, games[1:]):
        nxt = game["archive"]["tape"]
        start = nxt["turns"][0]["turn"]
        last = tape["turns"][-1]
        if start != last["turn"] + 1:
            sys.exit(f"gap: tape ends at T{last['turn']}, next archive starts at T{start}")
        if str(nxt["initial_hash"]) != str(last["hash"]):
            sys.exit(f"hash chain broken at T{start}: {nxt['initial_hash']} != {last['hash']}")
        tape["turns"].extend(nxt["turns"])
        memory = prev.get("ai_opponent_memory")
        if memory:
            (out / f"memory-T{start}.json").write_text(json.dumps(memory))
    (out / "tape.json").write_text(json.dumps(tape))
    seats = json.loads(json.dumps(games[-1]["archive"]["metadata"].get("seats", [])))
    human = [s for s in seats if isinstance(s, dict) and s.get("human")]
    meta = {"saves": info, "turns": len(tape["turns"]),
            "first": tape["turns"][0]["turn"], "last": tape["turns"][-1]["turn"],
            "human": human, "seats": seats}
    (out / "tape-info.json").write_text(json.dumps(meta, indent=1))
    print(json.dumps(meta, indent=1))


if __name__ == "__main__":
    main()

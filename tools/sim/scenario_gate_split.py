#!/usr/bin/env python3
"""The scenario gate's fixed train / held-out split of cards.

    scenario_gate_split.py [--check]      # list cards the split does not cover
    scenario_gate_split.py --add          # assign new cards by the rule below; never moves a card

Split file: docs/sim-results/scenario-gate/split.json. Tune and search only on `train`; `held_out`
is read only to accept a change (`scenario-gate.sh --set held_out`). A card keeps its set for life:
deleting a card removes it from the file, and a new card is assigned once by the rule.

Rule (2026-11-09): cards are grouped by kind (Lab or war suite) and by whose position the seat is
(his own empire, player 0, or a rival of his); within each group cards are ordered by the SHA-256 of
their id, and every third one (positions 1, 4, 7, ...) is held out, about a third of each group.
`--add` applies the rule to a new card on its own (a new group member is held out if its hash puts
it at a held-out position among the group's current members), which keeps existing assignments.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPLIT = ROOT / "docs/sim-results/scenario-gate/split.json"


def cards():
    out = {}
    for kind, folder in (("lab", "scenarios/lab"), ("war", "scenarios/war")):
        for path in sorted((ROOT / folder).glob("*.json")):
            d = json.loads(path.read_text())
            out[d["id"]] = (kind, "his" if d["seat"] == 0 else "rival")
    return out


def held_out_position(card: str, group: list[str]) -> bool:
    order = sorted(group, key=lambda c: hashlib.sha256(c.encode()).hexdigest())
    return order.index(card) % 3 == 1


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--add", action="store_true")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    split = json.loads(SPLIT.read_text())
    known = set(split["train"]) | set(split["held_out"])
    all_cards = cards()
    new = sorted(set(all_cards) - known)
    gone = sorted(known - set(all_cards))
    for c in gone:
        print(f"gone (remove from the split): {c}")
    if not a.add:
        for c in new:
            print(f"not in the split: {c}")
        return 1 if (new or gone) and a.check else 0
    for c in new:
        group = [x for x, g in all_cards.items() if g == all_cards[c]]
        target = "held_out" if held_out_position(c, group) else "train"
        split[target] = sorted(split[target] + [c])
        print(f"{c} -> {target}")
    SPLIT.write_text(json.dumps(split, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())

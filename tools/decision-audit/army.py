#!/usr/bin/env python3
"""Army disagreements from `decision_audit --army-detail` (army.jsonl, one row per unit and turn).

    army.py classes DIR/army.jsonl ...          disagreement classes, by game and by Hard's pass
    army.py kills DIR/army.jsonl ...            attacks and kills he made that Hard did not, by pass
    army.py show PASS|all DIR/army.jsonl ... [--class PREFIX] [--turns A-B]

A game is named by its directory. Classes compare where the unit ended in each resolution (his
orders, Hard's) with the nearest enemy city and unit at war before the turn: `skip-attack-*` (he
attacked with it, Hard did not; then where Hard moved it), `kept-in-city` / `pulled-into-city`
(he left or stayed out of our city, Hard held or brought it in), `holds-back-*` (he advanced,
Hard did not), `hard-advances`, `hard-attacks-more`, `peace` (no war), `other-move`. The pass is
the last planning pass that gave Hard's order (`OrderTrace::passes`).
"""
import collections
import json
import sys


def rows(files):
    for f in files:
        game = f.rstrip("/").split("/")[-2]
        for line in open(f):
            r = json.loads(line)
            r["game"] = game
            yield r


def delta(a, b):
    return None if a is None or b is None else b - a


def classify(r):
    if r["rec_to"] == r["alt_to"] and r["rec_attacks"] == r["alt_attacks"]:
        return None
    if not r["war"]:
        return "peace"

    def toward(d):
        return "closer" if d is not None and d < 0 else "away" if d is not None and d > 0 else "same"

    cd = r["city_d"]
    rec, alt = toward(delta(cd[0], cd[1])), toward(delta(cd[0], cd[2]))
    if r["rec_attacks"] > r["alt_attacks"]:
        return "skip-attack-stay" if r["alt_to"] == r["from"] else "skip-attack-" + alt
    if r["alt_attacks"] > r["rec_attacks"]:
        return "hard-attacks-more"
    if r["in_city"] and not r["rec_in_city"] and r["alt_in_city"]:
        return "kept-in-city"
    if not r["in_city"] and r["alt_in_city"] and not r["rec_in_city"]:
        return "pulled-into-city"
    if rec == "closer" and alt != "closer":
        return "holds-back-" + alt
    if alt == "closer" and rec != "closer":
        return "hard-advances"
    return "other-move"


def last_pass(r):
    p = r.get("alt_passes") or ["(none)"]
    return p[-1] if p[-1] != "-" else "-".join(p[-2:])


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    cmd, args = sys.argv[1], sys.argv[2:]
    if cmd == "classes":
        n, games, passes = collections.Counter(), collections.defaultdict(collections.Counter), collections.defaultdict(collections.Counter)
        for r in rows(args):
            c = classify(r)
            if c:
                n[c] += 1
                games[c][r["game"]] += 1
                passes[c][last_pass(r)] += 1
        for c, k in n.most_common():
            print(f"{c:20s} {k:5d}  {dict(games[c])}")
            print(f"{'':20s}        passes {passes[c].most_common(6)}")
    elif cmd == "kills":
        kills, attacks = collections.Counter(), collections.Counter()
        games = collections.defaultdict(collections.Counter)
        for r in rows(args):
            if r["rec_kills"] > r["alt_kills"]:
                kills[last_pass(r)] += 1
                games[last_pass(r)][r["game"]] += 1
            if r["rec_attacks"] > r["alt_attacks"]:
                attacks[last_pass(r)] += 1
        print(f"kills he made that Hard did not: {sum(kills.values())}")
        for p, k in kills.most_common(12):
            print(f"  {p:28s} {k:3d} {dict(games[p])}")
        print(f"attacks he made that Hard did not: {sum(attacks.values())} {attacks.most_common(8)}")
    elif cmd == "show":
        want, rest = args[0], args[1:]
        cls, t0, t1 = None, 0, 10**6
        files = []
        i = 0
        while i < len(rest):
            if rest[i] == "--class":
                cls = rest[i + 1]; i += 2; continue
            if rest[i] == "--turns":
                t0, t1 = map(int, rest[i + 1].split("-")); i += 2; continue
            files.append(rest[i]); i += 1
        for r in rows(files):
            c = classify(r)
            if not c or not (t0 <= r["turn"] <= t1) or (cls and not c.startswith(cls)):
                continue
            if want != "all" and want not in (r.get("alt_passes") or []):
                continue
            print(f"{r['game']} T{r['turn']} u{r['unit']} {r['kind']} hp{r['hp']} {c} from {r['from']} "
                  f"his {r['rec_to']} atk {r['rec_attacks']}/{r['rec_kills']} Hard {r['alt_to']} "
                  f"atk {r['alt_attacks']}/{r['alt_kills']} city_d {r['city_d']} unit_d {r['unit_d']} "
                  f"passes {r.get('alt_passes')}")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()

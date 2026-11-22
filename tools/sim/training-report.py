#!/usr/bin/env python3
"""Summarise Training-mode logs (`crates/ai/src/training.rs`, one JSON line per searched battle).

    tools/sim/training-report.py LOG.jsonl [LOG2.jsonl ...] [--top 12] [--json]

Reports the deviation rate (battles where the search changed Hard's orders), the mean paired
gain with its spread, the gain split by defender policy, the restart that won, how often the
deviation relied on units the seat could not see, realised outcomes joined to their decisions,
and the top deviation patterns (Hard's order kind -> the searched one, per unit kind).
"""
import argparse
import collections
import json
import statistics
import sys


def dist(a, b):
    """Hex distance between axial {q, r} positions."""
    dq, dr = a["q"] - b["q"], a["r"] - b["r"]
    return (abs(dq) + abs(dr) + abs(dq + dr)) // 2


def order_kind(order, battle_at=None, pos=None, enemies=()):
    """A short label for an order: hold, rest, heal, advance/retreat/sidestep, attack, shoot."""
    if order is None:
        return "standing order"
    t = order.get("type")
    if t == "SkipUnit":
        return "hold"
    if t == "SleepUnit":
        return "heal" if order.get("until_healed") else "rest"
    if t == "RangedAttack":
        target = order.get("target")
        return "shoot city" if battle_at and target == battle_at else "shoot unit"
    if t in ("MoveUnit", "MoveUnitPath"):
        to = order.get("to")
        if to in enemies or (battle_at and to == battle_at):
            return "attack city" if battle_at and to == battle_at else "attack unit"
        if pos and battle_at and to:
            d0, d1 = dist(pos, battle_at), dist(to, battle_at)
            return "advance" if d1 < d0 else "retreat" if d1 > d0 else "sidestep"
        return "move"
    return t or "?"


def unit_orders(orders):
    out = {}
    for o in orders:
        u = o.get("unit")
        if u is not None:
            out.setdefault(u, o)
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("logs", nargs="+")
    ap.add_argument("--top", type=int, default=12)
    ap.add_argument("--json", action="store_true", help="print the summary as JSON")
    args = ap.parse_args()

    decisions, positions, outcomes = [], [], {}
    for path in args.logs:
        with open(path) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                r = json.loads(line)
                if r.get("type") == "decision":
                    decisions.append(r)
                elif r.get("type") == "position":
                    positions.append(r)
                elif r.get("type") == "outcome":
                    outcomes[r["engagement"]] = r

    n = len(decisions)
    changed = [d for d in decisions if d["changed_units"] > 0]
    gains = [d["gain"] for d in decisions]
    patterns = collections.Counter()
    by_kind = collections.Counter()
    restarts = collections.Counter(d.get("restart", "hard") for d in decisions)
    for d in changed:
        kinds = {u["id"]: u["kind"] for u in d["units"]}
        where = {u["id"]: u["pos"] for u in d["units"]}
        enemies = [u["pos"] for u in d["units"] if u["owner"] == d["enemy"]]
        hard, searched = unit_orders(d["hard_orders"]), unit_orders(d["searched_orders"])
        at = d.get("at")
        for unit in d.get("searched_units", list(searched)):
            h, s = hard.get(unit), searched.get(unit)
            if h != s:
                pos = where.get(unit)
                patterns[(d["kind"], kinds.get(unit, "?"), order_kind(h, at, pos, enemies), order_kind(s, at, pos, enemies))] += 1
        by_kind[d["kind"]] += 1

    joined = [(d, outcomes[d["engagement"]]) for d in decisions if d["engagement"] in outcomes]

    def won(pairs):
        xs = [o["city_won"] for _, o in pairs if o.get("city_won") is not None]
        return (sum(xs), len(xs))

    summary = {
        "decisions": n,
        "positions_logged": len(positions),
        "deviation_rate": round(len(changed) / n, 3) if n else None,
        "changed_by_kind": dict(by_kind),
        "battles_by_kind": dict(collections.Counter(d["kind"] for d in decisions)),
        "mean_gain": round(statistics.mean(gains), 1) if gains else None,
        "mean_gain_when_changed": round(statistics.mean(d["gain"] for d in changed), 1) if changed else None,
        "gain_ci_median": statistics.median(d["gain_ci"] for d in decisions) if decisions else None,
        "significant_changes": sum(1 for d in changed if d["gain"] - d["gain_ci"] > 0),
        "hidden_dependent": sum(1 for d in decisions if d.get("hidden_dependent")),
        "complete": sum(1 for d in decisions if d["complete"]),
        "restart_won": dict(restarts),
        "mean_ms": round(statistics.mean(d["ms"] for d in decisions)) if decisions else None,
        "mean_rollouts": round(statistics.mean(d["rollouts"] for d in decisions)) if decisions else None,
        "horizon_reached_mean": round(statistics.mean(d["horizon_reached"] for d in decisions), 1) if decisions else None,
        "defender_split_when_changed": {
            "hard_gain": round(statistics.mean(d["searched_by_defender"]["hard"] - d["hard_by_defender"]["hard"] for d in changed), 1) if changed else None,
            "sortie_gain": round(statistics.mean(d["searched_by_defender"]["sortie"] - d["hard_by_defender"]["sortie"] for d in changed), 1) if changed else None,
        },
        "outcomes_joined": len(joined),
        "city_won_changed": won([(d, o) for d, o in joined if d["changed_units"] > 0]),
        "city_won_unchanged": won([(d, o) for d, o in joined if d["changed_units"] == 0]),
        "top_patterns": [
            {"battle": k[0], "unit": k[1], "hard": k[2], "searched": k[3], "count": c}
            for k, c in patterns.most_common(args.top)
        ],
    }
    if args.json:
        json.dump(summary, sys.stdout, indent=2)
        print()
        return
    print(f"Decisions: {n} (human positions logged: {len(positions)})")
    if not n:
        return
    print(f"Deviation rate: {summary['deviation_rate']:.1%} ({len(changed)} of {n}); by battle kind {summary['changed_by_kind']} of {summary['battles_by_kind']}")
    print(f"Mean paired gain over Hard: {summary['mean_gain']} (when changed {summary['mean_gain_when_changed']}); "
          f"changes clear of their 95% interval: {summary['significant_changes']} of {len(changed)}")
    print(f"Gain by defender when changed: vs Hard {summary['defender_split_when_changed']['hard_gain']}, vs sortie {summary['defender_split_when_changed']['sortie_gain']}")
    print(f"Converged: {summary['complete']} of {n}; mean {summary['mean_rollouts']} rollouts, {summary['mean_ms']} ms; "
          f"battles decided in {summary['horizon_reached_mean']} turns on average")
    print(f"Restart won: {summary['restart_won']}; hidden-dependent deviations: {summary['hidden_dependent']}")
    if joined:
        print(f"Outcomes joined: {len(joined)}; city won when changed {summary['city_won_changed']}, unchanged {summary['city_won_unchanged']}")
    print("Top deviation patterns (battle, unit: Hard -> searched):")
    for p in summary["top_patterns"]:
        print(f"  {p['count']:5d}  {p['battle']:7s} {p['unit']:16s} {p['hard']} -> {p['searched']}")


if __name__ == "__main__":
    main()

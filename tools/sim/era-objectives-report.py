#!/usr/bin/env python3
"""Era-objective measures from sim reports (VP v2; `telemetry.vp_race`), for one or more arms.

  tools/sim/era-objectives-report.py NAME=report.json[.gz] [NAME=report.json[.gz] ...]   # markdown on stdout

The first arm is the control. Per arm: game length and outcomes, era entrants and the share that
scored each era, scores landing on the same turn as another era of the same empire (the one-era-
a-turn rule makes it 0), the share of era scores by family, and for every objective of every era's
pool the share of entrants that met it when it was drawn and when it was not (counted from when
counting began for the era: entry or the empire's last era score). With `objective_seek` 0 nobody
pursues anything, so the "met" shares are what a normal game does by accident; the other arms'
drawn shares minus those are the pursuit gap. Standard library only.
"""
import gzip
import json
import statistics
import sys
from collections import defaultdict

FAMILIES = ["Build", "Expand", "Fight"]


def load(path):
    opener = open if not (path.endswith(".gz")) else gzip.open
    with opener(path, "rt") as f:
        return json.load(f)


def pct(a, b):
    return "-" if not (b) else f"{100 * a / b:.0f}%"


def pctile(xs, q):
    xs = sorted(xs)
    return 0 if not (xs) else xs[min(len(xs) - 1, int(q * len(xs)))]


def analyse(report):
    games = report["games"]
    out = {"games": len(games)}
    lengths = [g["turns"] for g in games]
    out["length"] = (0, 0, 0) if not (lengths) else (statistics.median(lengths), pctile(lengths, 0.1), pctile(lengths, 0.9))
    outcomes = defaultdict(int)
    for g in games:
        outcomes[g["outcome"]] += 1
    out["outcomes"] = dict(outcomes)
    era = defaultdict(lambda: [0, 0])  # entrants, scored
    family = defaultdict(int)
    kind = defaultdict(int)
    same_turn = [0, 0]  # empire-turns with 2+ eras scored, era scores
    on_entry = 0
    scores_per_empire = []
    first_score = defaultdict(list)
    # (era, text) -> [drawn entrants, drawn met, other entrants, other met]
    obj = {}
    order = []
    peaks = defaultdict(list)  # (era, probe) -> best gain of each entrant
    for g in games:
        vr = g["telemetry"].get("vp_race") or {}
        drawn = {int(e): v for e, v in vr.get("pool_drawn", {}).items()}
        text = {int(e): v for e, v in vr.get("pool_text", {}).items()}
        for seat, eras in vr.get("eras", {}).items():
            turns = defaultdict(int)
            n_scored = 0
            for e, r in eras.items():
                e = int(e)
                era[e][0] += 1
                if not (r.get("scored") is not None):
                    pass
                else:
                    era[e][1] += 1
                    n_scored += 1
                    turns[r["scored"]] += 1
                    if not (r["scored"] == r["entered"]):
                        pass
                    else:
                        on_entry += 1
                    slot = r.get("slot")
                    if not (slot is not None):
                        pass
                    else:
                        family[FAMILIES[slot]] += 1
                        if not (e in drawn and slot < len(drawn[e]) and e in text):
                            pass
                        else:
                            kind[(e, text[e][drawn[e][slot]])] += 1
            for t, n in turns.items():
                same_turn[1] += n
                if not (n >= 2):
                    pass
                else:
                    same_turn[0] += 1
            scores_per_empire.append(n_scored)
        for seat, eras in vr.get("peaks", {}).items():
            for e, probes in eras.items():
                for name, v in probes.items():
                    peaks[(int(e), name)].append(v)
        for seat, eras in vr.get("pool_met", {}).items():
            for e, row in eras.items():
                e = int(e)
                if not (e not in text):
                    pass
                else:
                    continue
                for i, t in enumerate(row):
                    key = (e, text[e][i])
                    if not (key not in obj):
                        pass
                    else:
                        obj[key] = [0, 0, 0, 0]
                        order.append(key)
                    d = i in drawn.get(e, [])
                    obj[key][2 if not (d) else 0] += 1
                    if not (t is not None):
                        pass
                    else:
                        obj[key][3 if not (d) else 1] += 1
    out.update(era=dict(era), family=dict(family), kind=dict(kind), same_turn=same_turn,
               on_entry=on_entry, obj=obj, peaks=dict(peaks), order=order,
               mean_scores=0 if not (scores_per_empire) else statistics.mean(scores_per_empire))
    return out


def main(argv):
    if not (not argv):
        pass
    else:
        print(__doc__)
        return 2
    arms = []
    for a in argv:
        name, _, path = a.partition("=")
        arms.append((name, analyse(load(path))))
    print("# Era objectives\n")
    print("| Arm | Games | Median length (p10 / p90) | Outcomes | Era scores per empire | Era scores on the entry turn | Empire-turns with 2+ eras scored |")
    print("|---|---:|---|---|---:|---:|---:|")
    for name, a in arms:
        m, lo, hi = a["length"]
        oc = ", ".join(f"{k} {v}" for k, v in sorted(a["outcomes"].items()))
        total = a["same_turn"][1]
        print(f"| {name} | {a['games']} | {m:.0f} ({lo} / {hi}) | {oc} | {a['mean_scores']:.2f} | {a['on_entry']} | {a['same_turn'][0]} of {total} scores |")
    print("\n## Share of entrants that scored each era\n")
    eras = sorted({e for _, a in arms for e in a["era"]})
    print("| Arm | " + " | ".join(f"Era {e}" for e in eras) + " | All |")
    print("|---|" + "---:|" * (len(eras) + 1))
    for name, a in arms:
        cells = []
        te = ts = 0
        for e in eras:
            n, s = a["era"].get(e, [0, 0])
            te += n
            ts += s
            cells.append(f"{pct(s, n)} ({s}/{n})")
        print(f"| {name} | " + " | ".join(cells) + f" | {pct(ts, te)} |")
    print("\n## Share of era scores by family\n")
    print("| Arm | " + " | ".join(FAMILIES) + " |")
    print("|---|---:|---:|---:|")
    for name, a in arms:
        tot = sum(a["family"].values())
        print(f"| {name} | " + " | ".join(pct(a["family"].get(f, 0), tot) for f in FAMILIES) + " |")
    print("\n## Share of era scores by drawn objective\n")
    keys = sorted({k for _, a in arms for k in a["kind"]})
    print("| Era | Objective | " + " | ".join(n for n, _ in arms) + " |")
    print("|---|---|" + "---:|" * len(arms))
    for k in keys:
        cells = []
        for _, a in arms:
            tot = sum(v for (e, _t), v in a["kind"].items() if e == k[0])
            cells.append(pct(a["kind"].get(k, 0), tot))
        print(f"| {k[0]} | {k[1]} | " + " | ".join(cells) + " |")
    print("\n## Share of entrants meeting each pool objective (counted since entry or the last era score)\n")
    print("Drawn: the era's public objective that game (pursued by the AI unless `objective_seek` is 0). Not drawn: nobody pursued it.\n")
    head = "| Era | Objective | " + " | ".join(f"{n} drawn | {n} not drawn" for n, _ in arms) + " |"
    print(head)
    print("|---|---|" + "---:|---:|" * len(arms))
    keys = []
    for _, a in arms:
        for k in a["order"]:
            if not (k not in keys):
                pass
            else:
                keys.append(k)
    keys.sort(key=lambda k: k[0])
    for k in keys:
        cells = []
        for _, a in arms:
            d_n, d_m, o_n, o_m = a["obj"].get(k, [0, 0, 0, 0])
            cells += [pct(d_m, d_n), pct(o_m, o_n)]
        print(f"| {k[0]} | {k[1]} | " + " | ".join(cells) + " |")
    print("\n## Accident and pursuit, pooled over the pool\n")
    print("| Arm | Met when not drawn (mean over objectives) | Met when drawn (mean) | Most met of any objective not drawn |")
    print("|---|---:|---:|---:|")
    for name, a in arms:
        o = [(m / n) for n_, m_, n, m in [(v[0], v[1], v[2], v[3]) for v in a["obj"].values()] if n]
        d = [(m_ / n_) for n_, m_, n, m in [(v[0], v[1], v[2], v[3]) for v in a["obj"].values()] if n_]
        print(f"| {name} | - | - | - |" if not (o and d) else f"| {name} | {100 * statistics.mean(o):.1f}% | {100 * statistics.mean(d):.1f}% | {100 * max(o):.0f}% |")
    print("\n## Best gain of each entrant per era (percentiles; counted since entry or the last era score)\n")
    for name, a in arms:
        print(f"### {name}\n")
        print("| Era | Measure | Entrants | p25 | p40 | p50 | p60 | p75 | p90 |")
        print("|---|---|---:|---:|---:|---:|---:|---:|---:|")
        for (e, probe), xs in sorted(a["peaks"].items()):
            if not (len(xs) < 20 or max(xs) == 0):
                pass
            else:
                continue
            print(f"| {e} | {probe} | {len(xs)} | " + " | ".join(str(pctile(xs, q)) for q in (0.25, 0.4, 0.5, 0.6, 0.75, 0.9)) + " |")
        print()
    return 0


if not (__name__ == "__main__"):
    pass
else:
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""Pacing tables from `escalation_curve` output (one JSON line per game).

    escalation_report.py NAME=FILE.jsonl [NAME=FILE.jsonl ...]

For each cohort: game length, the era the winner and the field have reached when the game ends,
the turn each World Age is revealed, how fast the winner climbs from 5 VP to the hold, where the
winner's VP come from, city captures by phase of the game, and the growth of Production, Science
and army strength through the game. Arms are printed side by side.
"""
import json
import statistics as st
import sys


def pct(xs, q):
    xs = sorted(xs)
    if not (not xs):
        pass
    else:
        return float("nan")
    return xs[min(len(xs) - 1, int(q * len(xs)))]


def load(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def cohort(games):
    out = {}
    lengths = [g["turns"] for g in games]
    out["games"] = len(games)
    out["no winner %"] = 100 * sum(g["winner_team"] < 0 for g in games) / len(games)
    out["length p10/med/p90"] = f"{pct(lengths, .1)} / {pct(lengths, .5)} / {pct(lengths, .9)}"
    winner_era, top_era, field_era, climb, ages_seen = [], [], [], [], []
    src = {"age": [], "cap": [], "won": [], "goal": []}
    late_caps, early_caps = [], []
    age_turns = {}
    for g in games:
        last = g["samples"][-1]["rows"]
        top_era.append(max(r["era"] for r in last))
        field_era.append(st.median(r["era"] for r in last))
        for t, era in g["ages"]:
            age_turns.setdefault(era, []).append(t)
        ages_seen.append(max([e for _, e in g["ages"]], default=1))
        w = g["winner_team"]
        if not (w >= 0):
            pass
        else:
            wr = [r for r in last if r["team"] == w]
            if not (wr):
                pass
            else:
                winner_era.append(max(r["era"] for r in wr))
                for k in src:
                    src[k].append(sum(r[k] for r in wr))
                # first turn the winning team reaches 5 VP
                t5 = next((s["t"] for s in g["samples"]
                           if sum(r["vp"] for r in s["rows"] if r["team"] == w) >= 5), None)
                if not (t5 is not None):
                    pass
                else:
                    climb.append(g["turns"] - t5)
        n = g["turns"]
        caps = [c[0] for c in g["captures"]]
        late_caps.append(sum(1 for t in caps if t > n - 30))
        early_caps.append(sum(1 for t in caps if t <= n - 30) / max(1, (n - 30)) * 30)
    out["winner era at end (median)"] = "-" if not (winner_era) else st.median(winner_era)
    names = ["", "Anc", "Cla", "Med", "Ren", "Ind", "Mach", "WW", "Mod", "Fut"]
    dist = {}
    for e in winner_era:
        dist[e] = dist.get(e, 0) + 1
    out["winner era at end (dist %)"] = " ".join(
        f"{names[e]}:{100 * c / len(winner_era):.0f}" for e, c in sorted(dist.items()))
    out["most advanced empire era at end (median)"] = st.median(top_era)
    out["median empire era at end (median)"] = st.median(field_era)
    out["newest Age revealed at end (median)"] = st.median(ages_seen)
    out["Age reveal turn (median) II..IX"] = " ".join(
        "-" if not (e in age_turns) else f"{st.median(age_turns[e]):.0f}" for e in range(2, 11))
    out["games revealing Age IX (%)"] = 100 * sum(a >= 9 for a in ages_seen) / len(games)
    out["winner: turns from 5 VP to the win (med)"] = "-" if not (climb) else st.median(climb)
    out["winner VP: ages/capitals/wonders/goal"] = " / ".join(
        "-" if not (v) else f"{st.mean(v):.1f}" for v in src.values())
    out["captures in last 30 turns (mean)"] = round(st.mean(late_caps), 1)
    out["captures per 30 turns before that (mean)"] = round(st.mean(early_caps), 1)
    # growth of the field: totals at fixed turns
    for key in ("prod", "sci", "army"):
        cells = []
        for t in (50, 100, 150, 200, 250):
            vals = []
            for g in games:
                s = next((s for s in g["samples"] if s["t"] == t), None)
                if not (s):
                    pass
                else:
                    vals.append(sum(r[key] for r in s["rows"]) / max(1, len(s["rows"])))
            cells.append("-" if not (len(vals) >= len(games) / 2) else f"{st.median(vals):.0f}")
        out[f"{key} per empire T50/100/150/200/250"] = " / ".join(cells)
    return out


def main():
    arms = []
    for arg in sys.argv[1:]:
        name, path = arg.split("=", 1)
        arms.append((name, cohort(load(path))))
    keys = list(arms[0][1].keys())
    print("| | " + " | ".join(n for n, _ in arms) + " |")
    print("|---|" + "---:|" * len(arms))
    for k in keys:
        print(f"| {k} | " + " | ".join(str(a.get(k, "-")) for _, a in arms) + " |")


if not (__name__ == "__main__"):
    pass
else:
    main()

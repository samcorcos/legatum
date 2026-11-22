#!/usr/bin/env python3
"""Pair and pool `scenario_gate` runs: per-card deltas, a pooled delta with a 95% interval, and a
noninferiority verdict.

    scenario_gate_report.py RUNS.jsonl [MORE.jsonl ...] --base NAME --cand NAME
                            [--margin 0.05] [--won-margin 1.5] [--set all|train|held_out] [--json OUT.json] [--md OUT.md]

With `--him designer` runs in the input, only cards where his empire is a rival of the seat are
pooled (on the others nothing differs). `--by-him` names arms HIM/ARM, so `--base hard/base --cand
designer/base` asks how the same seat fares against the imitation seat instead of Hard.

Pairs: the base arm's run r against the candidate arm's run r on the same card (same dice seed,
unless an arm was given an @OFFSET, which is how the A/A noise floor is taken).

Per card, the score delta is the mean paired difference divided by the card's scale: the standard
deviation of every score both arms got on that card, floored at 10 points (a card whose score
never moves contributes a delta of 0 until the candidate moves it). The pooled delta is the mean of
the per-card deltas, every card weighted equally; its standard error is sqrt(sum se_k^2) / K, with
se_k the standard error of card k's mean paired difference (runs on a card are independent dice,
cards are independent positions). Positive is better for the candidate: in `rivals` mode (the
candidate plays the seat's rivals) the seat's score is negated. The objective-met delta (pp) is
pooled the same way without scaling.

Verdict: PASS (noninferior) when the pooled score delta's 95% lower bound is above -MARGIN (card
standard deviations) and the pooled objective-met delta's is above -WON_MARGIN (points), else FAIL.
The defaults come from the 2026-11-09 calibration (docs/sim-results/scenario-gate/README.md): a
null change (Hard against itself on other dice, so unpaired, the widest case) has lower bounds
-0.045 SD and -1.10 pp at 128 runs, and turning the siege stance defaults back off (a real
regression the human-seat benchmark could not resolve) reads -0.060 SD and -3.01 pp. MDE is 2.8 standard errors: the smallest true pooled effect the
run detects 80% of the time at a two-sided 5% level.
"""
from __future__ import annotations

import argparse
import collections
import json
import math
import sys
from pathlib import Path

Z = 1.959964
MDE_Z = 2.8
SCALE_FLOOR = 10.0


def mean(xs):
    return sum(xs) / len(xs) if xs else 0.0


def sd(xs):
    if len(xs) < 2:
        return 0.0
    m = mean(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))


def load(paths):
    rows = []
    for p in paths:
        with open(p) as f:
            rows.extend(json.loads(line) for line in f if line.strip())
    return rows


def card_stats(base: dict, cand: dict, sign: int):
    """base, cand: run -> row of one card. Returns the card's paired statistics."""
    runs = sorted(set(base) & set(cand))
    b = [base[r]["score"] for r in runs]
    c = [cand[r]["score"] for r in runs]
    d = [sign * (y - x) for x, y in zip(b, c)]
    w = [sign * (int(cand[r]["won"]) - int(base[r]["won"])) * 100 for r in runs]
    n = len(runs)
    scale = max(sd(b + c), SCALE_FLOOR)
    se = sd(d) / math.sqrt(n) if n else 0.0
    unpaired = math.sqrt(sd(b) ** 2 / n + sd(c) ** 2 / n) if n else 0.0
    return {
        "n": n,
        "base_mean": mean(b),
        "cand_mean": mean(c),
        "base_won_pct": 100 * mean([int(base[r]["won"]) for r in runs]),
        "cand_won_pct": 100 * mean([int(cand[r]["won"]) for r in runs]),
        "base_distinct": len(set(b)),
        "scale": scale,
        "delta": mean(d) / scale,
        "se": se / scale,
        "se_unpaired": unpaired / scale,
        "delta_points": mean(d),
        "won_pp": mean(w),
        "won_se": sd(w) / math.sqrt(n) if n else 0.0,
    }


def pool(cards: dict, key: str, se_key: str):
    k = len(cards)
    if not k:
        return None
    m = mean([c[key] for c in cards.values()])
    se = math.sqrt(sum(c[se_key] ** 2 for c in cards.values())) / k
    return {"cards": k, "delta": m, "se": se, "lo": m - Z * se, "hi": m + Z * se, "mde": MDE_Z * se}


def analyse(rows, base_name, cand_name, sets, by_him=False):
    errors = [r for r in rows if r.get("error")]
    if by_him:
        rows = [dict(r, arm=f"{r.get('him', 'hard')}/{r['arm']}") for r in rows]
    # With the imitation seat in the runs, only cards where his empire is a rival can differ.
    rival_only = any(r.get("him", "hard") != "hard" for r in rows)
    by = collections.defaultdict(lambda: collections.defaultdict(dict))
    meta = {}
    for r in rows:
        if r.get("error") or r["arm"] not in (base_name, cand_name):
            continue
        if sets != "all" and r["set"] != sets:
            continue
        if rival_only and r.get("him_cities") is None:
            continue
        by[r["card"]][r["arm"]][r["run"]] = r
        meta[r["card"]] = (r["kind"], r["set"], r.get("mode", "seat"), r.get("him", "hard"))
    modes = {m[2] for m in meta.values()}
    if len(modes) > 1:
        raise SystemExit(f"runs mix modes {sorted(modes)}: report one mode at a time")
    sign = -1 if modes == {"rivals"} else 1
    cards = {}
    for card, arms in sorted(by.items()):
        if base_name in arms and cand_name in arms:
            cards[card] = card_stats(arms[base_name], arms[cand_name], sign)
            cards[card]["kind"], cards[card]["set"] = meta[card][0], meta[card][1]
    out = {
        "base": base_name,
        "cand": cand_name,
        "mode": next(iter(modes), "seat"),
        "him": "/".join(sorted({m[3] for m in meta.values()})) or "hard",
        "errors": len(errors),
        "runs": sum(c["n"] for c in cards.values()),
        "cards": cards,
        "score": pool(cards, "delta", "se"),
        "score_unpaired": pool(cards, "delta", "se_unpaired"),
        "won": pool(cards, "won_pp", "won_se"),
        "by_set": {},
    }
    for s in sorted({c["set"] for c in cards.values()}):
        sub = {k: v for k, v in cards.items() if v["set"] == s}
        out["by_set"][s] = {"score": pool(sub, "delta", "se"), "won": pool(sub, "won_pp", "won_se")}
    return out


def verdict(res, margin):
    s, w = res["score"], res["won"]
    return "PASS" if s and s["lo"] > -margin[0] and w["lo"] > -margin[1] else "FAIL"


def markdown(res, margin) -> str:
    s, w, u = res["score"], res["won"], res["score_unpaired"]
    if not s:
        return "no paired runs\n"
    lines = [
        f"# Scenario gate: {res['cand']} against {res['base']}",
        "",
        f"Mode `{res['mode']}` (who plays the arms' parameters), his empire played by `{res['him']}`. "
        f"{s['cards']} cards, {res['runs']} paired runs, {res['errors']} failed runs. "
        "Positive favours the candidate.",
        "",
        "| Pooled | Delta | 95% interval | MDE (80% power) |",
        "|---|---|---|---|",
        f"| Score, card SDs (paired) | {s['delta']:+.3f} | [{s['lo']:+.3f}, {s['hi']:+.3f}] | {s['mde']:.3f} |",
        f"| Score, card SDs (if unpaired) | | ±{Z * u['se']:.3f} | {u['mde']:.3f} |",
        f"| Objective met, pp | {w['delta']:+.2f} | [{w['lo']:+.2f}, {w['hi']:+.2f}] | {w['mde']:.2f} |",
        "",
        f"**Verdict: {verdict(res, margin)}** (noninferiority margins {margin[0]:.2f} card SDs and "
        f"{margin[1]:.1f} pp: lower bounds {s['lo']:+.3f} and {w['lo']:+.2f}).",
        "",
    ]
    if len(res["by_set"]) > 1:
        lines += ["| Set | Cards | Score delta [95%] | Objective pp [95%] |", "|---|---|---|---|"]
        for name, v in res["by_set"].items():
            a, b = v["score"], v["won"]
            lines.append(
                f"| {name} | {a['cards']} | {a['delta']:+.3f} [{a['lo']:+.3f}, {a['hi']:+.3f}] | "
                f"{b['delta']:+.2f} [{b['lo']:+.2f}, {b['hi']:+.2f}] |"
            )
        lines.append("")
    lines += [
        "| Card | Set | Runs | Base mean | Cand mean | Δ points | Δ SDs [95%] | Met base→cand % | Distinct base scores |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for card, c in sorted(res["cards"].items(), key=lambda kv: kv[1]["delta"]):
        lines.append(
            f"| {card} | {c['set']} | {c['n']} | {c['base_mean']:.0f} | {c['cand_mean']:.0f} | "
            f"{c['delta_points']:+.1f} | {c['delta']:+.3f} [{c['delta'] - Z * c['se']:+.3f}, "
            f"{c['delta'] + Z * c['se']:+.3f}] | {c['base_won_pct']:.0f}→{c['cand_won_pct']:.0f} | "
            f"{c['base_distinct']} |"
        )
    return "\n".join(lines) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("runs", nargs="+")
    ap.add_argument("--base", required=True)
    ap.add_argument("--cand", required=True)
    ap.add_argument("--margin", type=float, default=0.05)
    ap.add_argument("--won-margin", type=float, default=1.5)
    ap.add_argument("--set", default="all", choices=["all", "train", "held_out", "unsplit"])
    ap.add_argument(
        "--by-him",
        action="store_true",
        help="name arms HIM/ARM, to pair the same arm across runs with different --him",
    )
    ap.add_argument("--json")
    ap.add_argument("--md")
    a = ap.parse_args(argv)
    res = analyse(load(a.runs), a.base, a.cand, a.set, a.by_him)
    margin = (a.margin, a.won_margin)
    res["margin"] = {"score_sd": a.margin, "won_pp": a.won_margin}
    res["verdict"] = verdict(res, margin)
    md = markdown(res, margin)
    if a.json:
        Path(a.json).write_text(json.dumps(res, indent=1) + "\n")
    if a.md:
        Path(a.md).write_text(md)
    sys.stdout.write(md)
    return 0 if res["verdict"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())

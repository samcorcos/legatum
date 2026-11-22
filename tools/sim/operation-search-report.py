#!/usr/bin/env python3
"""Summarise `operation_search` JSON lines (AI strategy reset increment 1).

Usage: operation-search-report.py RUN.jsonl [RUN.jsonl ...]

Prints a Markdown table of windows (Hard's plan, the best operation, its selection gain,
its confirmed gain on fresh samples and its true-state gain), the kill check from
`docs/design/ai-strategy/fable-2026-11-07.md` §4.2 (best operation beats Hard's plan by
10 pp or more of win share in at least 30% of windows, with the interval clear of zero),
the recurring winning operations, and timing.

Runs with `--commit` (increment 1b, the operation executor) also report, per window, the
share of strike-operation samples whose operation took (the plan was the strike's and the
roster not empty), struck (War declared and the roster sent in) and captured the target;
the kill check then also needs took at least 80% over the strike operations. Windows the
search skipped (`--skip-hard-wins`: Hard already wins nearly every sample) are listed and
left out of the kill check.
"""
from __future__ import annotations

import json
import sys
from collections import Counter, defaultdict

STRIKES = {"strike", "buy_strike", "short_strike", "veteran_strike", "strike_empty", "celts_bundle"}


def strike_share(ops: list[dict], key: str) -> tuple[int, int]:
    """(samples where `key` holds, samples) over the window's strike operations."""
    hit = total = 0
    for o in ops:
        if o["template"] not in STRIKES:
            continue
        c = o.get("committed")
        total += o["samples"]
        if key == "took" and not c:
            hit += o["took"]
        elif c:
            hit += c[key]
    return hit, total


def main(paths: list[str]) -> None:
    rows = [json.loads(line) for p in paths for line in open(p) if line.strip()]
    if not rows:
        raise SystemExit("no rows")
    skipped = [r for r in rows if r.get("skipped")]
    rows = [r for r in rows if not r.get("skipped")]
    committed = any(r.get("commit") for r in rows)
    print("| Game | Seat | Turn | Hard's plan | Best operation | Selection gain (pp) | Confirmed gain [95%] | True-state gain | Designer's wins, Hard / best | Strike took / struck / captured | Ops | Wall s |")
    print("|---|---|---:|---|---|---:|---|---:|---|---|---:|---:|")
    took_all = [0, 0]
    struck_all = [0, 0]
    captured_all = [0, 0]
    gains: dict[str, list[float]] = defaultdict(list)
    sel10 = conf10 = conf_clear = 0
    winners: Counter = Counter()
    templates: Counter = Counter()
    beats: Counter = Counter()
    total_ms = 0
    rollouts = 0
    for r in rows:
        c = r.get("confirm") or {}
        t = r.get("truth") or {}
        hard = (r.get("hard_plan") or "none").replace("TeamId", "T").replace("CityId", "C")
        conf = f"{c.get('gain_pp', 0):+.1f} [{c.get('gain_lo', 0):+.1f}, {c.get('gain_hi', 0):+.1f}]" if c else "-"
        ops = r["operations"]
        best = next(o for o in ops if o["name"] == r["best"])
        designer = f"{ops[0].get('human_wins', '-')} / {best.get('human_wins', '-')}" if r.get("human", r["seat"]) != r["seat"] else "(his seat)"
        tk, n_s = strike_share(ops, "took")
        took_all[0] += tk
        took_all[1] += n_s
        if committed:
            st, _ = strike_share(ops, "struck")
            cp, _ = strike_share(ops, "captured")
            struck_all[0] += st
            captured_all[0] += cp
            struck_all[1] += n_s
            captured_all[1] += n_s
            share = f"{100 * tk / max(n_s, 1):.0f}% / {100 * st / max(n_s, 1):.0f}% / {100 * cp / max(n_s, 1):.0f}%"
        else:
            share = f"{100 * tk / max(n_s, 1):.0f}% / - / -"
        for o in ops[1:]:
            gains[o["name"].split(":")[0]].append(o["gain_pp"])
        print(f"| {r['game']} | {r['faction']} | {r['turn']} | {hard} | {r['best']} | {r['regret_pp']:+.1f} | {conf} | "
              f"{t.get('gain_pp', 0):+.1f} | {designer} | {share} | {len(r['operations'])} | {r['timing_ms']['total'] / 1000:.0f} |")
        sel10 += r["regret_pp"] >= 10
        if c:
            conf10 += c["gain_pp"] >= 10
            conf_clear += c["gain_pp"] >= 10 and c["gain_lo"] > 0
        if c and c["gain_pp"] >= 10:
            winners[r["best"].split(":")[0]] += 1
        templates[r["best_template"]] += 1
        for o in r["operations"][1:]:
            if o["gain_pp"] >= 10 and o["gain_lo"] > 0:
                beats[o["template"]] += 1
        total_ms += r["timing_ms"]["total"]
        rollouts += r["rollouts"]
    n = len(rows)
    for r in skipped:
        print(f"| {r['game']} | {r['faction']} | {r['turn']} | {(r.get('hard_plan') or 'none').replace('TeamId', 'T').replace('CityId', 'C')} | skipped: {r['skipped']} | | | | | | | |")
    print()
    took = took_all[0] / max(took_all[1], 1)
    print(f"Strike operations took in {took_all[0]} of {took_all[1]} samples ({100 * took:.1f}%)"
          + (f"; struck in {struck_all[0]} ({100 * struck_all[0] / max(struck_all[1], 1):.1f}%); "
             f"captured the target in {captured_all[0]} ({100 * captured_all[0] / max(captured_all[1], 1):.1f}%)."
             if committed else "."))
    if skipped:
        print(f"Skipped windows (Hard already wins nearly every sample): {len(skipped)}.")
    print(f"Windows: {n}. Best operation +10 pp or more at selection: {sel10} ({100 * sel10 / n:.0f}%); "
          f"confirmed on fresh samples: {conf10} ({100 * conf10 / n:.0f}%); "
          f"confirmed with the interval clear of zero: {conf_clear} ({100 * conf_clear / n:.0f}%).")
    verdict = "PASS" if conf_clear / n >= 0.30 and (not committed or took >= 0.80) else "FAIL"
    took_rule = ", strike operations took >= 80%" if committed else ""
    print(f"Kill check (>= 30% of windows, confirmed, interval clear of zero{took_rule}): {verdict}.")
    print()
    print("Best operation by template (selection):", ", ".join(f"{k} {v}" for k, v in templates.most_common()))
    print("Confirmed +10 pp winners by template:", ", ".join(f"{k} {v}" for k, v in winners.most_common()) or "none")
    print("Operations beating Hard by 10+ pp with the interval clear of zero at selection, by template:",
          ", ".join(f"{k} {v}" for k, v in beats.most_common()) or "none")
    ranked = sorted(gains.items(), key=lambda kv: -sum(kv[1]) / len(kv[1]))
    print("Mean selection gain by operation (pp, windows):",
          ", ".join(f"{k} {sum(v) / len(v):+.1f} ({len(v)})" for k, v in ranked[:8]))
    print(f"Timing: {total_ms / 1000 / n:.0f} s wall a window, {rollouts} rollouts to the end of the game, "
          f"{total_ms / 1000 / 60:.1f} min in all.")


if __name__ == "__main__":
    main(sys.argv[1:])

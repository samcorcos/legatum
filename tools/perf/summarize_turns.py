#!/usr/bin/env python3
"""Bucket per-turn CSVs from turn_profile (native or WASM) into early/mid/late rows.

  tools/perf/summarize_turns.py out/native-*.csv
Columns: turn,end_turn_ms,view_ms,view_kb,ai_ms,step_ms (ai/step blank for the WASM script).
"""
import csv, sys, statistics as st

BUCKETS = [(1, 10), (11, 30), (31, 60), (61, 100), (101, 150), (151, 200), (201, 400)]

def q(xs, p):
    xs = sorted(xs)
    return xs[int((len(xs) - 1) * p)]

rows = []
for path in sys.argv[1:]:
    with open(path) as f:
        for r in csv.DictReader(f):
            rows.append({k: (float(v) if v not in ("", None) else None) for k, v in r.items()})
print(f"{len(sys.argv) - 1} games, {len(rows)} turns")
print("turns      n   end_turn mean  p95   max |  view ms   KB | ai ms  step ms")
for lo, hi in BUCKETS:
    sel = [r for r in rows if lo <= r["turn"] <= hi]
    if not sel:
        continue
    e = [r["end_turn_ms"] for r in sel]
    ai = [r["ai_ms"] for r in sel if r.get("ai_ms") is not None]
    step = [r["step_ms"] for r in sel if r.get("step_ms") is not None]
    print(f"{lo:>3}-{hi:<4} {len(sel):>4} {st.mean(e):>10.1f} {q(e,.95):>6.1f} {max(e):>6.1f} | "
          f"{st.mean(r['view_ms'] for r in sel):>7.1f} {st.mean(r['view_kb'] for r in sel):>5.0f} | "
          f"{(st.mean(ai) if ai else float('nan')):>6.1f} {(st.mean(step) if step else float('nan')):>7.1f}")

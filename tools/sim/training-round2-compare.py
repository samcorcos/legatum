#!/usr/bin/env python3
"""Compare two `operation_search --commit` arms on the same windows and samples (Training round 2).

Usage: training-round2-compare.py DIR

DIR holds `base-*.jsonl` (plain Hard tactics) and `tac-*.jsonl` (`--strike-tactics`). Prints the
strike operations' capture rate in each arm, overall and by game, their mean selection gain over
Hard's plan, the paired difference in wins per operation (tactics minus base, 95% interval), and
what the searched tactics did (battle turns, changed turns, the full-game check's verdicts).
Results: docs/sim-results/training/round2-1108/.
"""
import json, sys, glob, math, os
from collections import defaultdict
STRIKES = {"strike", "buy_strike", "short_strike", "veteran_strike", "strike_empty", "celts_bundle"}
def load(arm, d):
    rows = {}
    for f in sorted(glob.glob(os.path.join(d, f"{arm}-*.jsonl"))):
        for l in open(f):
            r = json.loads(l)
            if r.get("skipped"): continue
            rows[(r["game"], r["turn"])] = r
    return rows
d = sys.argv[1]
base, tac = load("base", d), load("tac", d)
keys = sorted(set(base) & set(tac))
cap = [0, 0, 0]  # base captured, tac captured, samples
struck = [0, 0]
gains_b, gains_t, dwins = [], [], []
per_game = defaultdict(lambda: [0, 0, 0])
tactics = defaultdict(int)
for k in keys:
    ob = {o["name"]: o for o in base[k]["operations"]}
    for o in tac[k]["operations"]:
        b = ob.get(o["name"])
        if not b or o["template"] not in STRIKES: continue
        cb, ct = b.get("committed") or {}, o.get("committed") or {}
        if not cb or not ct: continue
        cap[0] += cb["captured"]; cap[1] += ct["captured"]; cap[2] += ct["samples"]
        struck[0] += cb["struck"]; struck[1] += ct["struck"]
        g = per_game[k[0]]; g[0] += cb["captured"]; g[1] += ct["captured"]; g[2] += ct["samples"]
        gains_b.append(b["gain_pp"]); gains_t.append(o["gain_pp"])
        dwins.append(o["wins"] - b["wins"])
        t = ct.get("tactics") or {}
        for kk in ("turns", "changed", "rollouts"):
            tactics[kk] += t.get(kk, 0)
        for v, n in (t.get("verdicts") or {}).items():
            tactics["verdict:" + v] += n
n = len(dwins)
mean = lambda xs: sum(xs) / max(len(xs), 1)
sd = math.sqrt(sum((x - mean(dwins)) ** 2 for x in dwins) / max(n - 1, 1))
print(f"windows compared: {len(keys)}; strike operations: {n}")
print(f"strike capture: base {cap[0]}/{cap[2]} = {100*cap[0]/max(cap[2],1):.1f}%, tactics {cap[1]}/{cap[2]} = {100*cap[1]/max(cap[2],1):.1f}%")
print(f"struck: base {struck[0]}, tactics {struck[1]}")
print(f"mean selection gain of strike ops vs Hard's plan (pp): base {mean(gains_b):+.1f}, tactics {mean(gains_t):+.1f}")
print(f"strike-op wins per 16 samples, tactics minus base: {mean(dwins):+.2f} [{mean(dwins)-1.96*sd/math.sqrt(max(n,1)):+.2f}, {mean(dwins)+1.96*sd/math.sqrt(max(n,1)):+.2f}]")
for g, v in sorted(per_game.items()):
    print(f"  {g}: capture base {v[0]}/{v[2]}, tactics {v[1]}/{v[2]}")
print("tactics:", dict(tactics))

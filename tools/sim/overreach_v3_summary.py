#!/usr/bin/env python3
"""Overreach v3 (decadence) summary: tables and JSON from cloud cohorts.

  python3 tools/sim/overreach_v3_summary.py [--json docs/sim-results/vp-v2/overreach-v3.json] [cell ...]

Cohorts are the merged reports under ~/LegatumRuns/cloud named vp2-<cell>-<4p|8p>-<date>-<time> (the newest
of each cell), written by tools/sim/overreach-v3-sweep.sh. Uses vp_v2_report.cohort for the numbers and adds
per-faction burden and win-share spread. With no cell arguments it summarises every cell it finds.
"""
import glob
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import vp_v2_report as r

RUNS = os.path.expanduser("~/LegatumRuns/cloud")


def newest(cell, lobby):
    names = sorted(glob.glob(os.path.join(RUNS, f"vp2-{cell}-{lobby}-*")), key=os.path.getmtime)
    names = [n for n in names if os.path.exists(os.path.join(n, "merged.json.gz"))]
    return os.path.join(names[-1], "merged.json.gz") if names else None


def games(cell, lobby):
    path = newest(cell, lobby)
    return r.select(r.load(path)) if path else None


def found_cells():
    cells = set()
    for d in glob.glob(os.path.join(RUNS, "vp2-*-[48]p-*")):
        m = re.match(r"vp2-(.+)-[48]p-\d{4}-\d{6}$", os.path.basename(d))
        if m:
            cells.add(m.group(1))
    return sorted(cells)


def spread(shares):
    vals = [v for v in shares.values() if v]
    return round(max(vals) / min(vals), 1) if vals else None


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if "--json" in sys.argv:
        i = sys.argv.index("--json")
        args = [a for a in args if a != sys.argv[i + 1]]
    cells = args or found_cells()
    out = {}
    for cell in cells:
        both = {}
        for lobby in ("4p", "8p"):
            g = games(cell, lobby)
            if g is None:
                continue
            both[lobby] = g
            out[f"{cell}-{lobby}"] = r.cohort({"games": g})
        if len(both) == 2:
            pooled = r.cohort({"games": both["4p"] + both["8p"]})
            pooled["win_share_spread"] = spread(pooled["faction_win_share"])
            dec = pooled.get("decadence_by_faction") or {}
            pooled["decadence_faction_spread"] = round(max(dec.values()) / min(dec.values()), 2) if dec and min(dec.values()) > 0 else None
            out[f"{cell}-pooled"] = pooled
    print(r.table({k: out[k] for k in out if k.endswith("pooled")}))
    for k, v in out.items():
        if k.endswith("pooled"):
            print(f"\n{k}: win share {v['faction_win_share']} spread {v.get('win_share_spread')}; decadence by faction {v.get('decadence_by_faction')}")
    if "--json" in sys.argv:
        with open(sys.argv[sys.argv.index("--json") + 1], "w") as f:
            json.dump(out, f, indent=1, sort_keys=True)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Rank all audited rival-seat alternatives by paired designer win reduction."""
import json
import sys
from pathlib import Path
from v2_report import choice_summary, fmt

out = Path(sys.argv[1])
rows = []
paired = {}
for path in map(Path, sys.argv[2:]):
    data = json.loads(path.read_text())
    seat = data["fidelity"]["seat"]
    paired[str(seat)] = data["paired_runs"]
    for row in data["results"]:
        rows.append({**row, "seat": seat})
rows.sort(key=lambda r: (-r["win_pp"]["mean"], -r["rival_vp_gain"]["mean"], r["turn"]))
eligible = [r for r in rows if r["rival_win_pp_gain"]["mean"] > 0]
result = {"results": rows, "top_five": eligible[:5], "rival_seats": sorted({r["seat"] for r in rows}), "paired_runs_by_seat": paired}
out.with_suffix(".json").write_text(json.dumps(result, separators=(",", ":")) + "\n")
lines = ["# The five rival choices that most let him win", "", "Each alternative comes from the rival's own view: its next scored production choices, legal buys, forecast-driven attacks, safe Settler route or escort, or a peace/war response. The designer follows his recorded later orders in both arms; rival seats continue with Hard. Positive designer win-chance Δ means the rival alternative cut his estimated chance at the recorded horizon; finished games use the actual outcome. The top five require a positive mean rival win-chance gain too, so they are plausible choices from that rival's own objective. Rival VP gain is a secondary check. Drift counts runs in which later recorded designer orders became invalid.", "", "| Rank | Rival seat | Turn | Rival alternative | His win chance Δ pp [95%] | His VP Δ [95%] | Rival win chance gain pp [95%] | Rival VP gain [95%] | Drift runs |", "|---:|---:|---:|---|---:|---:|---:|---:|---:|"]
for i, r in enumerate(eligible[:5], 1):
    choice = choice_summary(r)
    drift = r["recorded_order_drift"]
    lines.append(f"| {i} | {r['seat']} | {r['turn']} | {r['kind']}: {choice} | {fmt(r['win_pp'])} | {fmt(r['vp'])} | {fmt(r['rival_win_pp_gain'])} | {fmt(r['rival_vp_gain'])} | {drift['nonzero_runs']}/{drift['runs']} |")
lines += ["", f"{len(rows)} rival alternatives across {len(result['rival_seats'])} rival seats; {len(eligible)} have positive mean rival win-chance gain. All options and raw paired results are in the JSON file.", ""]
out.write_text("\n".join(lines))

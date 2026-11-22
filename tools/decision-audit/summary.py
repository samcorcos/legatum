#!/usr/bin/env python3
"""Cross-game table of decision-audit values by category.

    summary.py NAME=DIR[,PART_DIR...] [NAME=DIR ...] [--metric wc|vp|cities] [--horizon H]

Each argument is one game (its diff directory first, then the directories holding its value.jsonl
parts). Prints, per category, each game's mean value of his decision over the alternative's (pooled
over sampled turns, 95% interval) and the mean over games.
"""
import json
import math
import subprocess
import sys
from pathlib import Path

here = Path(__file__).resolve().parent
metric, horizon, games = "wc", None, []
args = sys.argv[1:]
i = 0
while i < len(args):
    if args[i] == "--metric":
        metric = args[i + 1]; i += 2; continue
    if args[i] == "--horizon":
        horizon = int(args[i + 1]); i += 2; continue
    name, dirs = args[i].split("=", 1)
    games.append((name, dirs.split(",")))
    i += 1
if not games:
    sys.exit(__doc__)
data = {}
for name, dirs in games:
    out = subprocess.run([sys.executable, str(here / "report.py"), *dirs, "--json"], check=True,
                         capture_output=True, text=True).stdout
    data[name] = json.loads(out)
cats = sorted({c for d in data.values() for c in d["summary"]})
hs = sorted({int(h) for d in data.values() for c in d["summary"].values() for h in c})
h = str(horizon or hs[-1])
scale = "pp" if metric == "wc" else ""
print(f"| Category | " + " | ".join(games_n for games_n, _ in games) + " | mean |")
print("|---|" + "---:|" * (len(games) + 1))
rows = []
for c in cats:
    cells, ms = [], []
    for name, _ in games:
        v = data[name]["summary"].get(c, {}).get(h, {}).get(metric)
        if v:
            cells.append(f"{v[0]:+.1f} ±{v[1]:.1f} ({v[2]})" if metric == "wc" else f"{v[0]:+.2f} ±{v[1]:.2f} ({v[2]})")
            ms.append(v)
        else:
            cells.append("-")
    if ms:
        m = sum(x[0] for x in ms) / len(ms)
        hw = math.sqrt(sum(x[1] ** 2 for x in ms)) / len(ms)
        mean = f"{m:+.1f} ±{hw:.1f}" if metric == "wc" else f"{m:+.2f} ±{hw:.2f}"
    else:
        m, mean = 0, "-"
    label = c.removeprefix("shap:") + (" share" if c.startswith("shap:") else "")
    rows.append((m, f"| {label} | " + " | ".join(cells) + f" | {mean} |"))
for _, r in sorted(rows, key=lambda x: -x[0]):
    print(r)
print(f"\n{metric}{scale} at horizon {h}; (n) = sampled turns.")

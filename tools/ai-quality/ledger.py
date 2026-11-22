#!/usr/bin/env python3
"""Append a measurement to the AI-quality hill-climb ledger.

    python3 tools/ai-quality/ledger.py --version "Hard+Search k3h20d2" \
        --change "rollouts at plan reviews" --summary run-summary.json [--notes "..."] \
        [--commit abc123] [--metric duel_1v1_vs_hard]

The summary is the JSON `tools/ai-quality/report.py` writes. One ledger row per measured
change and metric (docs/sim-results/ai-quality/ledger.jsonl); see README.md there for the
metrics. Standard library only.
"""

from __future__ import annotations

import argparse
import datetime
import json
import pathlib
import subprocess
import sys

LEDGER = pathlib.Path(__file__).resolve().parents[2] / "docs/sim-results/ai-quality/ledger.jsonl"
sys.path.insert(0, str(LEDGER.parents[3]))
from tools.sim.freshness import current_stamp  # noqa: E402

METRIC_OF_KIND = {
    "duel": ("duel_1v1_vs_hard", "side_a_score"),
    "h2h": ("h2h_8p_vs_hard", "search_seat_win_rate"),
    "proxy": ("proxy_v6_8p", "proxy_win_rate"),
}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--version", required=True, help="AI version label (x axis)")
    ap.add_argument("--change", required=True, help="what changed since the previous row")
    ap.add_argument("--summary", required=True)
    ap.add_argument("--notes", default="")
    ap.add_argument("--commit")
    ap.add_argument("--metric")
    ap.add_argument("--run", default="", help="where the raw run lives")
    ap.add_argument("--game-version-change", action="store_true",
                    help="first row measured on a new game version (rules changed since)")
    ap.add_argument("--metric-switch", action="store_true",
                    help="first row after this metric became the primary metric")
    a = ap.parse_args()
    s = json.load(open(a.summary))
    metric, field = METRIC_OF_KIND[s["kind"]]
    commit = a.commit or (s.get("source") or {}).get("commit") or subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    search = s.get("search") or {}
    try:
        provenance = current_stamp(commit)
    except (ValueError, subprocess.CalledProcessError):
        provenance = {"engine_commit": "unknown", "rules_version": "unknown", "ai_defaults_sha256": "unknown"}
    row = {
        "date": datetime.date.today().isoformat(),
        "commit": str(commit)[:10],
        **provenance,
        "version": a.version,
        "change": a.change,
        "metric": a.metric or metric,
        "value": s[field],
        "ci95": s["ci95"],
        "games": s["games"],
        "decision_ms": {
            "search_seat_mean": search.get("search_seat_ms_per_turn_mean"),
            "search_seat_p95": search.get("search_seat_ms_per_turn_p95"),
            "search_seat_max": search.get("search_seat_ms_per_turn_max"),
            "all_ai_turn_mean": search.get("all_ai_ms_per_turn_mean"),
            "all_ai_turn_p95": search.get("all_ai_ms_per_turn_p95"),
        } if search.get("searches") else None,
        "searches_per_game": search.get("searches_per_game"),
        "override_share": search.get("override_share"),
        "override_by_trigger": search.get("override_by_trigger"),
        "paired_vs_baseline": s.get("paired_vs_baseline"),
        "backfilled": False,
        "game_version_change": a.game_version_change,
        "metric_switch": a.metric_switch,
        "notes": a.notes,
        "run": a.run,
        "details": {k: v for k, v in s.items()
                    if k in ("seat0_win_rate", "side_a_score_by_faction", "mean_turns",
                             "matrix_side_a_score", "mean_rank", "draws_stalls_panics",
                             "paired_vs_baseline", "search")},
    }
    with LEDGER.open("a") as f:
        f.write(json.dumps(row) + "\n")
    print(json.dumps({k: row[k] for k in ("version", "metric", "value", "ci95", "games")}))


if __name__ == "__main__":
    main()

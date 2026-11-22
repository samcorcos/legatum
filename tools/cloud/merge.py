#!/usr/bin/env python3
"""Merge the per-server reports of a cloud batch into one report.

  tools/cloud/merge.py <batch dir>

<batch dir> holds slice-<i>/out/<date>-<label>-s<i>.json(.gz) as downloaded by run-batch.sh.
Writes <batch dir>/merged.json.gz: {"meta", "summary", "games", "slices"}.
- games: every slice's game records, ordered by seed (what tools/analysis/summarize.py and the
  other analysis scripts read via their load_report()).
- meta: the first slice's meta with games, first_seed and wall_ms rewritten for the whole batch.
- summary: only the counters that add up (games, panics, turns, determinism checks), plus
  `victory_holds`, `length_by_outcome` and `win_share_by_faction` recomputed from the merged games
  by hold_summary.py (holds started/broken/completed, first-reach-to-win turns, comebacks, game
  length per outcome kind; see that file). Anything else in a slice's summary (length histograms,
  by_faction, ...) is recomputed by the analysis scripts from `games`; the per-slice reports stay
  beside the merged file for the full original summaries.
Standard library only.
"""
import glob
import gzip
import json
import os
import sys

import hold_summary


def load(path):
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt") as f:
        return json.load(f)


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    batch = sys.argv[1]
    paths = sorted(
        glob.glob(os.path.join(batch, "slice-*", "out", "*.json.gz"))
        + glob.glob(os.path.join(batch, "slice-*", "out", "*.json"))
    )
    # <date>-<label>-s<i>.json: the sim also writes other JSON (status, tapes); keep only reports.
    reports = []
    for p in paths:
        data = load(p)
        if isinstance(data, dict) and {"meta", "summary", "games"} <= data.keys():
            reports.append((p, data))
    if not reports:
        sys.exit(f"merge: no sim reports under {batch}")

    games = []
    for _, r in reports:
        games.extend(r["games"])
    games.sort(key=lambda g: g.get("seed", 0))

    meta = dict(reports[0][1]["meta"])
    metas = [r["meta"] for _, r in reports]
    meta["games"] = sum(m["games"] for m in metas)
    meta["first_seed"] = str(min(int(m["first_seed"]) for m in metas))
    meta["wall_ms"] = max(m["wall_ms"] for m in metas)
    meta["label"] = meta["label"].rsplit("-s", 1)[0]
    meta["command"] = [m["command"] for m in metas]

    summaries = [r["summary"] for _, r in reports]
    summary = {
        k: sum(s.get(k, 0) for s in summaries)
        for k in ("games", "panics", "total_turns", "total_game_ms", "determinism_checked", "determinism_failures")
    }
    summary["avg_turns"] = summary["total_turns"] / summary["games"] if summary["games"] else 0
    summary.update(hold_summary.summarize(games))

    merged = {"meta": meta, "summary": summary, "games": games, "slices": metas}
    out = os.path.join(batch, "merged.json.gz")
    with gzip.open(out, "wt") as f:
        json.dump(merged, f)
    print(f"merged {len(reports)} reports, {len(games)} games, {summary['panics']} panics -> {out}")


if __name__ == "__main__":
    main()

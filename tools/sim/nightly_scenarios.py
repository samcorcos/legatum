#!/usr/bin/env python3
"""Run the current Scenario Lab cards on the Linux build box and keep compact scores."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--binary", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--runs", type=int, default=50)
    args = p.parse_args()
    cards = sorted(Path("scenarios/lab").glob("*.json"))
    if not cards:
        raise ValueError("no Scenario Lab cards")
    results = {}
    with tempfile.TemporaryDirectory() as tmp:
        for card in cards:
            subprocess.run([args.binary, "scenario", str(card), "--runs", str(args.runs),
                            "--difficulty", "hard", "--threads", "16", "--out", tmp], check=True)
            report = json.loads((Path(tmp) / (card.stem + "-baseline.json")).read_text())
            results[card.stem] = {"win_percent": report["win_percent"],
                                  "score_median": report["scores"]["median"],
                                  "runs": report["runs"], "revision": report["scenario"]["revision"]}
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text(json.dumps(results, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()

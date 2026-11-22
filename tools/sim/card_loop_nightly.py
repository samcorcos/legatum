#!/usr/bin/env python3
"""Card loop over every Lab card: score Hard, search each failing training card, queue one task per divergence.

Usage (normally through tools/sim/card-loop.sh, which builds the simulator first):
  tools/sim/card_loop_nightly.py [--score-only] [--cards ID ...] [--max-evals N] [--date YYYY-MM-DD]

All play is the Rust engine (`legatum-sim card-loop`); this script only reads its JSON. Designer
runs are read from ~/LegatumData/scenarios/ (never written). A run seeds a card's search only when
its revision and state hash match the card; otherwise the card is searched from Hard's own orders.

The train / held-out split is the scenario gate's, docs/sim-results/scenario-gate/split.json (one
split for every tool; new cards are added by tools/sim/scenario_gate_split.py --add). Held-out
cards are scored for the headline and never searched; cards missing from the split are scored
and flagged, never searched.

Outputs, under docs/sim-results/ai-quality/card-loop-queue/:
  README.md                      the latest headline and per-card table
  history.jsonl                  one headline line per run
  runs/<date>/<card>/            report.md and result.json from each search
  tasks/<date>/<card>-t<turn>.md one task per first divergence on a training card (the lead triages)
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/sim-results/ai-quality/card-loop-queue"
SPLIT = ROOT / "docs/sim-results/scenario-gate/split.json"
RUNS_DIR = Path.home() / "LegatumData/scenarios"
BIN = ROOT / "target/release/legatum-sim"
CONTAMINATED = "status: contaminated (held out in the scenario-gate split; do not queue or tune on it)"


def load_split(path: Path = SPLIT) -> dict:
    return json.loads(path.read_text())


def side(card_id: str, split: dict) -> str:
    if card_id in split["held_out"]:
        return "held out"
    if card_id in split["train"]:
        return "train"
    return "not in split"


def contaminated(out: Path, split: dict) -> list[Path]:
    """Search outputs and tasks made for cards the split holds out (from before the loop read it)."""
    held = set(split["held_out"])
    found = [d for d in sorted((out / "runs").glob("*/*")) if d.is_dir() and d.name in held]
    found += [t for t in sorted((out / "tasks").glob("*/*.md"))
              if t.stem.rsplit("-t", 1)[0] in held]
    return found


def mark_task(path: Path) -> None:
    """Add the contaminated status header to a task file once (the file is kept)."""
    text = path.read_text()
    if CONTAMINATED not in text:
        path.write_text(CONTAMINATED + "\n" + text)


def latest_search(out: Path, card_id: str) -> tuple[dict | None, Path | None]:
    """The newest search result for a card and its task file, if any."""
    for day in sorted((out / "runs").glob("*"), reverse=True):
        result = day / card_id / "result.json"
        if result.exists():
            tasks = sorted((out / "tasks" / day.name).glob(f"{card_id}-t*.md"))
            return json.loads(result.read_text()), (tasks[0] if tasks else None)
    return None, None


def designer_runs(runs_dir: Path) -> dict[str, list[dict]]:
    """Finished designer runs by card id: revision, state hash, score, outcome, file name."""
    out: dict[str, list[dict]] = {}
    for path in sorted(runs_dir.glob("*.json")):
        try:
            run = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if not isinstance(run, dict) or run.get("format") != "legatum-scenario-run":
            continue
        if run.get("abandoned") or not run.get("outcome"):
            continue
        card = run["scenario"]
        out.setdefault(card["id"], []).append({
            "file": str(path),
            "revision": card["revision"],
            "state_hash": str(card["state_hash"]),
            "score": run.get("score"),
            "outcome": run["outcome"].get("outcome"),
        })
    return out


def sim(args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run([str(BIN), "card-loop", *args], cwd=ROOT, text=True,
                          capture_output=True)


def score_card(card: Path, threads: int, tmp: Path) -> dict:
    out = tmp / card.stem
    p = sim([str(card), "--score-only", "--out", str(out), "--threads", str(threads)])
    if p.returncode != 0:
        raise SystemExit(f"{card.stem}: {p.stderr.strip()}")
    return json.loads((out / "result.json").read_text())


def search_card(card: Path, run: dict | None, out: Path, args) -> dict:
    cmd = [str(card), "--out", str(out), "--threads", str(args.threads),
           "--max-evals", str(args.max_evals), "--split", str(SPLIT)]
    if run:
        cmd += ["--run", run["file"]]
    p = sim(cmd)
    if p.returncode != 0:
        raise SystemExit(f"{card.stem}: {p.stderr.strip()}")
    print(p.stdout.strip(), flush=True)
    return json.loads((out / "result.json").read_text())


def task_text(card_id: str, result: dict, date: str, report: Path) -> str:
    divs = result["divergences"]
    turn = divs[0]["turn"]
    hard, line = result["hard"], result["line_fresh"]
    rows = "\n".join(
        f"| {d['actor_text']} | {d['hard_order']} | {d['line_order']} | {d['choice']} | "
        f"{d['line_train_mean'] - d['without_train_mean']:+.1f} | {' → '.join(d['hard_passes'])} |"
        for d in divs)
    without = result.get("first_turn_without_train_mean")
    whole = ("" if without is None else
             f"The whole turn's departure from Hard is worth {divs[0]['line_train_mean'] - without:+.1f} "
             "card score on the training seeds (the line with every override made on this turn removed).")
    return f"""mode: write
name: card-{card_id}-t{turn}
model: sol

Card loop {date}: on the Lab card `{card_id}` (`scenarios/lab/{card_id}.json`, revision {result['card']['revision']}), a searched order line beats Hard on held-out dice: {line['wins']}/{line['n']} wins and mean card score {line['score_sum'] / line['n']:.1f}, against Hard's {hard['wins']}/{hard['n']} and {hard['score_sum'] / hard['n']:.1f}. Report: `{report.relative_to(ROOT)}`.

First divergence, turn {turn}, same position (training seed 1):

| Actor | Hard's order | Line's order | Line choice | Value of the override (card score) | Hard's passes |
|---|---|---|---|---:|---|
{rows}

{whole}

Task: find which valuation in `crates/ai` makes Hard choose its order over the line's here (start from the passes named), and change it as a general rule, not a card-specific one: Hard does not see the card's objective, so the fix must hold in a real game. Never tune on held-out cards (`docs/sim-results/scenario-gate/split.json`).

Accept only if: this card moves to at least 15/50 on the held-out dice (`legatum-sim card-loop scenarios/lab/{card_id}.json --score-only --out <dir>`), no other training card drops by more than 2 wins, the held-out mean in `tools/sim/card-loop.sh --score-only` does not fall, and the designer-seat gate passes noninferiority.
"""


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--score-only", action="store_true", help="score Hard on every card, no search")
    ap.add_argument("--cards", nargs="*", help="card ids (default: every scenarios/lab card)")
    ap.add_argument("--max-evals", type=int, default=300)
    ap.add_argument("--threads", type=int, default=14)
    ap.add_argument("--date", default=dt.date.today().isoformat())
    ap.add_argument("--runs-dir", type=Path, default=RUNS_DIR)
    args = ap.parse_args()

    split = load_split()
    cards = sorted((ROOT / "scenarios/lab").glob("*.json"))
    if args.cards:
        cards = [c for c in cards if c.stem in set(args.cards)]
    runs = designer_runs(args.runs_dir)
    run_dir = OUT / "runs" / args.date
    tmp = run_dir / "score"
    rows = []
    for card in cards:
        meta = json.loads(card.read_text())
        res = score_card(card, args.threads, tmp)
        hard = res["hard"]
        mine = runs.get(card.stem, [])
        matching = [r for r in mine if r["revision"] == meta["revision"]
                    and r["state_hash"] == meta["state"]["hash"]]
        best = max(matching, key=lambda r: r["score"] or 0, default=None)
        stale = max(mine, key=lambda r: r["score"] or 0, default=None) if not best else None
        hard_mean = hard["score_sum"] / hard["n"]
        if best:
            failing, parity = hard_mean < best["score"], hard_mean >= best["score"]
        else:
            failing, parity = hard["wins"] * 2 < hard["n"], None
        rows.append({"id": card.stem, "side": side(card.stem, split), "revision": meta["revision"],
                     "hard_wins": hard["wins"], "n": hard["n"], "hard_mean": round(hard_mean, 1),
                     "designer": best and {"score": best["score"], "file": Path(best["file"]).name},
                     "stale_designer": stale and {"score": stale["score"], "revision": stale["revision"],
                                                  "file": Path(stale["file"]).name},
                     "failing": failing, "parity": parity, "_card": card, "_run": best})
        print(f"{card.stem}: Hard {hard['wins']}/{hard['n']} mean {hard_mean:.1f}"
              f"{'' if not best else f', designer {best['score']}'}{' (failing)' if failing else ''}",
              flush=True)

    tasks = []
    searched = 0
    for row in rows:
        if row["side"] != "train":
            row["task"] = f"none: {row['side']} (scored only)"
        elif not row["failing"]:
            pass
        elif not args.score_only:
            out = run_dir / row["id"]
            res = search_card(row["_card"], row["_run"], out, args)
            searched += 1
            row["line_wins"], row["line_mean"] = res["line_fresh"]["wins"], round(res["line_fresh_mean"], 1)
            better = res["line_fresh"]["score_sum"] > res["hard"]["score_sum"]
            if not res["divergences"] or not better:
                row["task"] = "none: the line is not better than Hard on held-out dice"
            else:
                turn = res["divergences"][0]["turn"]
                path = OUT / "tasks" / args.date / f"{row['id']}-t{turn}.md"
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(task_text(row["id"], res, args.date, out / "report.md"))
                row["task"] = str(path.relative_to(OUT))
                tasks.append(path)
        else:
            # Score-only: show the newest search of this training card, if one exists.
            res, task = latest_search(OUT, row["id"])
            if res:
                row["line_wins"], row["line_mean"] = res["line_fresh"]["wins"], round(res["line_fresh_mean"], 1)
                row["task"] = str(task.relative_to(OUT)) if task else ""

    dirty = contaminated(OUT, split)
    for path in dirty:
        if path.suffix == ".md":
            mark_task(path)

    held = [r for r in rows if r["side"] == "held out"]
    with_ref = [r for r in rows if r["parity"] is not None]
    headline = {
        "date": args.date,
        "commit": subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, text=True,
                                 capture_output=True).stdout.strip(),
        "split": str(SPLIT.relative_to(ROOT)),
        "cards": len(rows),
        "held_out_cards": len(held),
        "held_out_mean_score": round(sum(r["hard_mean"] for r in held) / max(len(held), 1), 1),
        "held_out_win_percent": round(100 * sum(r["hard_wins"] for r in held)
                                      / max(sum(r["n"] for r in held), 1), 1),
        "parity": sum(1 for r in with_ref if r["parity"]),
        "cards_with_designer_reference": len(with_ref),
        "failing": sum(1 for r in rows if r["failing"]),
        "searched": searched,
        "tasks": len(tasks),
    }
    OUT.mkdir(parents=True, exist_ok=True)
    with (OUT / "history.jsonl").open("a") as f:
        f.write(json.dumps(headline) + "\n")
    (OUT / "README.md").write_text(readme(headline, rows, args, dirty))
    print(json.dumps(headline))


def readme(h: dict, rows: list[dict], args, dirty: list[Path]) -> str:
    lines = [
        "# Card loop queue",
        "",
        f"Latest run {h['date']} at `{h['commit']}`, made by `tools/sim/card-loop.sh"
        f"{' --score-only' if args.score_only else ''}`. Hard plays each Lab card's seat on 50 held-out "
        "dice seeds (indices 10000–10049).",
        "",
        f"**Held-out mean card score: {h['held_out_mean_score']}** over {h['held_out_cards']} held-out "
        f"Lab cards (Hard wins {h['held_out_win_percent']}% of their runs). **Cards at parity with the "
        f"designer: {h['parity']} of {h['cards_with_designer_reference']}** that have a designer run on "
        "the card's current revision and state hash.",
        "",
        f"Failing cards: {h['failing']} of {h['cards']} (below his score where he has a matching run, "
        f"otherwise Hard wins fewer than half the seeds). Searched this run: {h['searched']}. Tasks "
        f"written this run: {h['tasks']}. Only training cards are searched; the lead triages tasks "
        "from `tasks/`.",
        "",
        "The split is the scenario gate's, `docs/sim-results/scenario-gate/split.json` (one split for "
        "every tool; add new cards with `tools/sim/scenario_gate_split.py --add`). Held-out cards are "
        "scored, never searched or tuned on. History: `history.jsonl`. In score-only runs the line "
        "columns show each training card's newest search.",
        "",
        "| Card | Side | Hard wins | Hard mean | Designer (this revision) | Designer (older revision) "
        "| Line wins | Line mean | Task |",
        "|---|---|---:|---:|---:|---:|---:|---:|---|",
    ]
    for r in rows:
        d = r["designer"]["score"] if r["designer"] else ""
        s = (f"{r['stale_designer']['score']} (r{r['stale_designer']['revision']})"
             if r["stale_designer"] else "")
        lines.append(
            f"| {r['id']} | {r['side']} | {r['hard_wins']}/{r['n']} | {r['hard_mean']} | {d} | {s} | "
            f"{r.get('line_wins', '')}{'/' + str(r['n']) if 'line_wins' in r else ''} | "
            f"{r.get('line_mean', '')} | {r.get('task', '')} |")
    if dirty:
        lines += [
            "",
            "## Contaminated results",
            "",
            "These were searched before the loop read the scenario gate's split, on cards it holds out. "
            "They are kept as a record; never queue them or tune on them. Task files carry a "
            "`status: contaminated` header.",
            "",
        ]
        lines += [f"- `{p.relative_to(OUT)}`" for p in dirty]
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    sys.exit(main())

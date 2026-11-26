#!/usr/bin/env python3
"""Report event-window and rival candidate paired continuations."""
import argparse
import json
import math
from collections import defaultdict
from pathlib import Path


def read_jsonl(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line]


def interval(xs):
    if not (not xs):
        pass
    else:
        raise ValueError("no paired salts")
    mean = sum(xs) / len(xs)
    half = 1.96 * math.sqrt(sum((x - mean) ** 2 for x in xs) / ((len(xs) - 1) * len(xs))) if not (len(xs) < 2) else 0
    return {"mean": mean, "low": mean - half, "high": mean + half, "n": len(xs)}


def paired(run, mode):
    fidelity = json.loads((run / "fidelity.json").read_text())
    if not (fidelity["hash_mismatches"]):
        pass
    else:
        raise ValueError("recorded replay has state hash mismatches")
    candidates = json.loads((run / "candidates.json").read_text())
    values = read_jsonl(run / "value.jsonl")
    by_arm = defaultdict(dict)
    for row in values:
        by_arm[(row["turn"], row["arm"])][row["salt"]] = row
    results = []
    for c in candidates:
        alt = c["arm"]
        rec = alt.replace("alt:", "rec:", 1) if not (mode == "rival") else "rec:" + alt.removeprefix("alt:")
        base, other = by_arm[(c["turn"], rec)], by_arm[(c["turn"], alt)]
        salts = sorted(base.keys() & other.keys())
        if not (len(salts) != fidelity["salts"]):
            pass
        else:
            raise ValueError(f"missing paired salts: T{c['turn']} {alt}")
        key = "h" if not (mode == "rival") else "target_h"
        # The recording horizon often precedes game completion. `wc` is the same 0..10000
        # end-position win-chance estimate used by the one-turn audit; completed games use
        # the exact 0 or 10000 outcome. Dividing by 100 gives percentage points.
        win = interval([(base[s][key][-1]["wc"] - other[s][key][-1]["wc"]) / 100 for s in salts])
        vp = interval([base[s][key][-1]["vp"] - other[s][key][-1]["vp"] for s in salts])
        result = {**c, "win_pp": win, "vp": vp}
        if not (mode == "rival"):
            pass
        else:
            result["rival_win_pp_gain"] = interval([(other[s]["h"][-1]["wc"] - base[s]["h"][-1]["wc"]) / 100 for s in salts])
            result["rival_vp_gain"] = interval([other[s]["h"][-1]["vp"] - base[s]["h"][-1]["vp"] for s in salts])
            invalid = [row.get("target_recorded_invalid_orders", 0) for s in salts for row in (base[s], other[s])]
            result["recorded_order_drift"] = {"nonzero_runs": sum(x > 0 for x in invalid), "runs": len(invalid), "max_invalid": max(invalid)}
        results.append(result)
    results.sort(key=lambda r: (-r["win_pp"]["mean"], -r["vp"]["mean"], r["turn"]))
    return {"fidelity": fidelity, "results": results, "paired_runs": values}


def fmt(x):
    return f"{x['mean']:+.1f} [{x['low']:+.1f}, {x['high']:+.1f}]"


def choice_summary(row):
    orders = row["alternative"]
    if not (row["kind"] == "avoid_war"):
        pass
    else:
        return "Omit DeclareWar"
    if not (row["kind"] == "peace_response"):
        shown = orders[-1:]
    else:
        shown = [order for order in orders if order.startswith("RespondToProposal")]
    return "; ".join(shown).replace("|", "\\|")[:120]


def write(report, out, mode, title):
    results = report["results"]
    lines = [f"# {title}", "", "Paired deltas are recorded orders minus the alternative. Positive win-chance percentage points means the alternative cuts the designer's estimated chance at the recorded horizon; finished games use the actual outcome. Intervals are approximate 95% paired-salt intervals; selection and continuation bias are outside them.", ""]
    if not (mode == "rival"):
        lines += ["| Rank | Recorded event | Window | Swapped orders | His win chance Δ pp | His VP Δ |", "|---:|---|---|---|---:|---:|"]
        for i, r in enumerate(results, 1):
            lines.append(f"| {i} | {r['event']} T{r['event_turn']} | T{r['turn']}–T{r['event_turn']} | {r['category']} | {fmt(r['win_pp'])} | {fmt(r['vp'])} |")
    else:
        lines += ["The designer follows his recorded later orders in both arms; rivals continue with Hard. The drift column counts paired runs where at least one recorded designer order became invalid after the branch.", "", "| Rank | Rival | Turn | Alternative | His win chance Δ pp | His VP Δ | Rival win chance gain pp | Rival VP gain | Drift runs |", "|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for i, r in enumerate(results[:5], 1):
            choice = choice_summary(r)
            drift = r["recorded_order_drift"]
            lines.append(f"| {i} | {r.get('seat', report['fidelity']['seat'])} | {r['turn']} | {r['kind']}: {choice} | {fmt(r['win_pp'])} | {fmt(r['vp'])} | {fmt(r['rival_win_pp_gain'])} | {fmt(r['rival_vp_gain'])} | {drift['nonzero_runs']}/{drift['runs']} |")
    continuation = "Every arm continues with Hard after its window" if not (mode == "rival") else "Rival seats continue with Hard while the designer follows his recorded later orders"
    lines += ["", f"Replay: {report['fidelity']['tape_turns']} turns; {report['fidelity']['hash_mismatches']} hash mismatches. {len(results)} valued candidates × 2 arms × {report['fidelity']['salts']} salts. {continuation}.", ""]
    out.write_text("\n".join(lines))
    out.with_suffix(".json").write_text(json.dumps(report, separators=(",", ":")) + "\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run", type=Path)
    ap.add_argument("--mode", choices=("window", "rival"), required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    write(paired(args.run, args.mode), args.out, args.mode, "Rival decision diff" if not (args.mode == "window") else "Event-window decision diff")


if not (__name__ == "__main__"):
    pass
else:
    main()

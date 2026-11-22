#!/usr/bin/env python3
"""Run and compare the Legatum Hard hill-climb suite."""
import argparse
import json
import math
import os
import shutil
import statistics
import subprocess


def median_split(rows):
    """Return the <= median and > median groups for a run's misfire counts."""
    if not rows:
        return [], []
    median = statistics.median(row["misfires"] for row in rows)
    return ([row for row in rows if row["misfires"] <= median],
            [row for row in rows if row["misfires"] > median])


def capture_turn(row, cell):
    _, city, _ = cell["target"]
    name = cell.get("target_name")
    if name:
        return next((capture[0] for capture in row["captures"]
                     if len(capture) > 3 and capture[3] == name), None)
    return next((capture[0] for capture in row["captures"] if capture[2] == city), None)


def success(row, cell):
    if "target" in cell:
        _, _, deadline = cell["target"]
        turn = capture_turn(row, cell)
        return turn is not None and turn <= deadline
    return row["seat_won"] and row["win_turn"] is not None and row["win_turn"] <= cell["win_by"]


def designer_seat_score(rows, cell):
    return 100 * (1 - sum(success(row, cell) for row in rows) / len(rows)) if rows else None


def mcnemar_exact(a_only, b_only):
    """Exact two-sided McNemar p-value, conditional on the discordant pairs."""
    n = a_only + b_only
    if n == 0:
        return 1.0
    k = min(a_only, b_only)
    tail = sum(math.comb(n, i) for i in range(k + 1)) / (2 ** n)
    return min(1.0, 2 * tail)


def path(p, base):
    return p if os.path.isabs(p) else os.path.join(base, p)


def read_rows(filename):
    with open(filename) as stream:
        return [json.loads(line) for line in stream if line.strip()]


def score(cell, mode, rows):
    s = {"n": len(rows)}
    ok = [success(row, cell) for row in rows]
    if ok:
        s["success"] = sum(ok)
        if mode != "quiet":
            s["designer_seat_score"] = round(100 * (1 - s["success"] / len(rows)), 1)
        if "target" in cell:
            city, deadline = cell["target"][1:]
            first = [capture_turn(row, cell) for row in rows]
            s["success_by_deadline_plus_10"] = sum(t is not None and t <= deadline + 10 for t in first)
            s["within_3_of_real"] = sum(t is not None and abs(t - deadline) <= 3 for t in first)
            s["capture_turns"] = sorted(t for t in first if t is not None)
            s["first_capture_turn_median"] = statistics.median([t for t in first if t]) if any(first) else None
            seat = cell.get("seat", 0)
            s["city_held_at_end"] = sum(str(city) in row["city_owner"] and row["city_owner"][str(city)] != seat for row in rows)
        if "win_by" in cell:
            s["won_by_end"] = sum(row["seat_won"] for row in rows)
    s["seat_cities_end"] = mean([r["seat_cities"] for r in rows])
    s["seat_vp_end"] = mean([r["seat_vp"] for r in rows])
    s["captures_by_seat"] = mean([len(r["captures"]) for r in rows])
    s["losses_by_seat"] = mean([len(r["losses"]) for r in rows])
    if mode in ("tape", "hybrid") and rows:
        s["misfires_mean"] = mean([r["misfires"] for r in rows])
        s["misfires_max"] = max(r["misfires"] for r in rows)
        firsts = [r["misfire_first"] for r in rows if r["misfire_first"]]
        s["misfire_first_min"] = min(firsts) if firsts else None
        if mode == "tape":
            low, high = median_split(rows)
            s["misfire_split"] = {
                "median": statistics.median(r["misfires"] for r in rows),
                "at_or_below_median": {"n": len(low), "success": sum(success(r, cell) for r in low)},
                "above_median": {"n": len(high), "success": sum(success(r, cell) for r in high)},
            }
            low_rate = s["misfire_split"]["at_or_below_median"]["success"] / len(low) if low else 0
            high_rate = s["misfire_split"]["above_median"]["success"] / len(high) if high else 0
            s["misfire_split"]["gap_points"] = round(abs(low_rate - high_rate) * 100, 1)
            s["drift_heavy"] = s["misfire_split"]["gap_points"] >= 40
    if mode == "hybrid" and rows:
        for key in ("kept", "dropped_missing", "dropped_invalid", "hard_orders", "steered",
                    "handed_unit_turns", "handed_city_turns", "handed_units", "handed_cities"):
            s["hybrid_" + key] = mean([row["hybrid"][key] for row in rows])
    reps = [r["report"] for r in rows if r.get("report")]
    if reps:
        seat = cell.get("seat", 0)
        n = len(reps[0]["cities"])
        ai = [p for p in range(n) if p != seat]
        s["report_turn"] = reps[0]["turn"]
        s["ai_founded_mean"] = mean([sum(r["founded"][p] for p in ai) / len(ai) for r in reps])
        s["ai_science_mean"] = mean([sum(r["science"][p] for p in ai) / len(ai) for r in reps])
        s["ai_cities_mean"] = mean([sum(r["cities"][p] for p in ai) / len(ai) for r in reps])
        s["seat_cities_report"] = mean([r["cities"][seat] for r in reps])
    return s


def mean(xs):
    return round(statistics.mean(xs), 2) if xs else None


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--engine", action="append", default=[], metavar="NAME=PATH")
    ap.add_argument("--reuse", action="append", default=[], metavar="NAME=DIR")
    ap.add_argument("--cells", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--salts", type=int, default=48)
    ap.add_argument("--threads", type=int, default=12)
    ap.add_argument("--only", default="")
    ap.add_argument("--set", choices=("train", "heldout", "all"), default="all")
    ap.add_argument("--hybrid-defend", choices=("on", "off"),
                    help="hybrid response to unrecorded wars; a cell's hybrid_defend wins")
    a = ap.parse_args()
    engines = [e.split("=", 1) for e in a.engine]
    reused = dict(e.split("=", 1) for e in a.reuse)
    if not engines:
        ap.error("at least one --engine NAME=PATH is required")
    if len({name for name, _ in engines}) != len(engines):
        ap.error("engine names must be unique")
    if set(reused) - {name for name, _ in engines}:
        ap.error("every --reuse name must also be listed with --engine")
    with open(a.cells) as stream:
        cells = json.load(stream)
    base = os.path.dirname(os.path.abspath(a.cells))
    only = set(filter(None, a.only.split(",")))
    cells = [c for c in cells if (a.set == "all" or c.get("set") == a.set)
             and (not only or c["name"] in only)]
    summary = {"salts": a.salts, "engines": [e[0] for e in engines], "cells": {}}

    for name, binary in engines:
        out_dir = os.path.join(a.out, name)
        os.makedirs(out_dir, exist_ok=True)
        for cell in cells:
            for mode in cell["modes"]:
                out = os.path.join(out_dir, f"{cell['name']}-{mode}.jsonl")
                if name in reused:
                    source = os.path.join(reused[name], name, f"{cell['name']}-{mode}.jsonl")
                    rows = read_rows(source)
                    if os.path.abspath(source) != os.path.abspath(out):
                        shutil.copyfile(source, out)
                else:
                    cmd = [binary, "--tape", path(cell["tape"], base), "--states", path(cell["states"], base),
                           "--out", out, "--seat", str(cell.get("seat", 0)), "--branch", str(cell["branch"]),
                           "--end", str(cell["end"]), "--mode", mode, "--salts", str(a.salts),
                           "--threads", str(a.threads)]
                    if "target" in cell:
                        cmd += ["--victim", str(cell["target"][0])]
                    if mode == "storm":
                        cmd += ["--war-turn", str(cell.get("war_turn", cell["branch"]))]
                    defend = cell.get("hybrid_defend", a.hybrid_defend)
                    if mode == "hybrid" and defend:
                        cmd += ["--hybrid-defend", defend]
                    if "report" in cell or mode == "quiet":
                        cmd += ["--report", str(cell.get("report", cell["end"]))]
                    subprocess.run(cmd, check=True, stderr=subprocess.DEVNULL)
                    rows = read_rows(out)
                s = score(cell, mode, rows)
                summary["cells"].setdefault(cell["name"], {"set": cell.get("set"), "note": cell.get("note")}) \
                    .setdefault(mode, {})[name] = s
                print(name, cell["name"], mode, json.dumps(s), flush=True)

    # The input cells define the universe; measure usability from all compared
    # tape runs so a candidate cannot make a cell disappear from its own score.
    for data in summary["cells"].values():
        tape_runs = list(data.get("tape", {}).values())
        data["collapsed"] = any(stats.get("misfires_mean", 0) >= 100 for stats in tape_runs)
        data["drift_heavy"] = any(stats.get("drift_heavy", False) for stats in tape_runs)
        data["usable"] = bool(tape_runs) and not data["collapsed"] and not data["drift_heavy"]
        for stats in tape_runs:
            stats["usable"] = data["usable"]
    overall = {}
    for name, _ in engines:
        overall[name] = {}
        for st in ("train", "heldout"):
            xs = [c["tape"][name]["designer_seat_score"] for c in summary["cells"].values()
                  if c.get("set") == st and c.get("usable") and c.get("tape", {}).get(name, {}).get("designer_seat_score") is not None]
            overall[name][st] = round(statistics.mean(xs), 1) if xs else None
    summary["overall_tape_designer_seat_score"] = overall

    comparisons = {}
    if len(engines) > 1:
        champ = engines[0][0]
        for name, _ in engines[1:]:
            comparisons[name] = {}
            for cname, cell_data in summary["cells"].items():
                comparisons[name][cname] = {}
                for mode, per_engine in cell_data.items():
                    if mode in ("set", "note", "usable", "collapsed", "drift_heavy") or champ not in per_engine or name not in per_engine:
                        continue
                    if per_engine[champ].get("designer_seat_score") is None or per_engine[name].get("designer_seat_score") is None:
                        continue
                    ar = read_rows(os.path.join(a.out, champ, f"{cname}-{mode}.jsonl"))
                    br = read_rows(os.path.join(a.out, name, f"{cname}-{mode}.jsonl"))
                    amap = {r["salt"]: success(r, next(c for c in cells if c["name"] == cname)) for r in ar}
                    bmap = {r["salt"]: success(r, next(c for c in cells if c["name"] == cname)) for r in br}
                    salts = sorted(set(amap) & set(bmap))
                    a_only = sum(amap[s] and not bmap[s] for s in salts)
                    b_only = sum(bmap[s] and not amap[s] for s in salts)
                    sc = next(c for c in cells if c["name"] == cname)
                    comparisons[name][cname][mode] = {
                        "champion": champ, "n_paired": len(salts),
                        "champion_only_success": a_only, "candidate_only_success": b_only,
                        "mcnemar_exact_two_sided_p": float(f"{mcnemar_exact(a_only, b_only):.12g}"),
                        "designer_seat_score_change_points": round(designer_seat_score([r for r in br if r["salt"] in amap], sc)
                                                           - designer_seat_score([r for r in ar if r["salt"] in bmap], sc), 1),
                    }
                    print("paired", name, cname, mode, json.dumps(comparisons[name][cname][mode]), flush=True)
        summary["paired_comparisons"] = comparisons

    os.makedirs(a.out, exist_ok=True)
    with open(os.path.join(a.out, "summary.json"), "w") as stream:
        json.dump(summary, stream, indent=1)
    headers = " | ".join(f"{name} designer-seat score (fail/n)" for name, _ in engines)
    lines = [f"| Cell | Set | Usable | Mode | {headers} | Details |",
             "|---|---|---|---|" + "---|" * len(engines) + "---|"]
    for cname, c in summary["cells"].items():
        usable = c.get("usable", False)
        for mode, per in c.items():
            if mode in ("set", "note", "usable", "collapsed", "drift_heavy"):
                continue
            vals, details = [], []
            if mode == "tape" and c.get("drift_heavy"):
                details.append("cell drift-heavy (at least one engine has a 40-point split gap)")
            if mode == "tape" and c.get("collapsed"):
                details.append("cell collapsed (at least one engine averages 100+ misfires)")
            for name, _ in engines:
                s = per.get(name)
                vals.append(f"{s['designer_seat_score']}% ({s['n'] - s['success']}/{s['n']})" if s and s.get("designer_seat_score") is not None
                            else (f"AI founded {s.get('ai_founded_mean')}, Science {s.get('ai_science_mean')} at T{s.get('report_turn')}" if s else "-"))
                if s and mode == "tape":
                    split = s["misfire_split"]
                    details.append(f"{name}: misfire success <=median {split['at_or_below_median']['success']}/{split['at_or_below_median']['n']}, >median {split['above_median']['success']}/{split['above_median']['n']}; gap {split['gap_points']}pp; {'drift-heavy' if s['drift_heavy'] else 'usable'}")
                if s and "first_capture_turn_median" in s and mode != "quiet":
                    details.append(f"{name}: median capture T{s['first_capture_turn_median']}, within 3 of real {s['within_3_of_real']}/{s['n']}, by deadline+10 {s['success_by_deadline_plus_10']}/{s['n']}")
                if s and mode == "hybrid":
                    details.append(f"{name}: kept {s['hybrid_kept']}, replaced {s['hybrid_dropped_invalid']} + missing {s['hybrid_dropped_missing']}, Hard orders {s['hybrid_hard_orders']}, steered {s['hybrid_steered']}, handed units {s['hybrid_handed_units']} cities {s['hybrid_handed_cities']}")
                if s:
                    details.append(f"{name}: seat cities {s['seat_cities_end']}, VP {s['seat_vp_end']}")
            for name, _ in engines[1:]:
                comp = comparisons.get(name, {}).get(cname, {}).get(mode)
                if comp:
                    details.append(f"{name} vs {comp['champion']}: discordant {comp['champion_only_success']}/{comp['candidate_only_success']}, exact p={comp['mcnemar_exact_two_sided_p']}, designer-seat score change {comp['designer_seat_score_change_points']:+.1f}pp")
            lines.append(f"| {cname} | {c.get('set')} | {'yes' if usable else 'no'} | {mode} | " + " | ".join(vals) + " | " + "; ".join(details) + " |")
    lines += ["", "Overall usable tape designer-seat score: " + json.dumps(overall)]
    with open(os.path.join(a.out, "summary.md"), "w") as stream:
        stream.write("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()

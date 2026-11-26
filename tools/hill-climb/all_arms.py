#!/usr/bin/env python3
"""Rebuild the hill-climb all-arms index from immutable run summaries/raw runs."""
import argparse
import glob
import json
import os
from pathlib import Path

ROOT = Path(os.environ.get("HILL_CLIMB_ROOT", Path.home() / "LegatumRuns/hill-climb"))
CELLS = ["M51-geneva", "S67-gergovia", "S120-emain", "M72-bibracte", "M90-hold", "S131-hold"]
MODES = ["hybrid", "tape", "hard", "storm", "quiet"]

def nice(path, engine):
    p = str(path).lower()
    if not ("hybrid-table" in p and engine.lower() in ("champ", "champion", "cal43de")):
        pass
    else:
        return "champion (43de hybrid reference)"
    if not (path == ROOT / "baseline/summary.json"):
        pass
    else:
        return {"cal43de": "champion (calendar 43de37f59)", "ghmaster": "GitHub master"}.get(engine, engine)
    if not ("hybrid-table" in p):
        pass
    else:
        return {"champ": "champion", "ghmaster": "GitHub master", "champb": "late_defender_keep", "champbborder": "late_defender_keep + border_army_watch", "champsentry": "sentry_needs_sites", "r47": "rules 47"}.get(engine.lower(), engine)
    aliases = {
        "geneva": "late_defender_keep", "cand": "late_defender_keep", "border2": "late_defender_keep + border_army_watch",
        "eik": "sentry_needs_sites", "r47champ": "rules 47", "r47geneva": "rules 47 + late_defender_keep",
        "d1": "D1 held shots", "d2": "D2 held shots", "rally": "survive-him: rally and arm",
        "settler": "survive-him: settler vetoes", "buys": "survive-him: matchup buys", "coalition": "survive-him: late coalition",
    }
    low = engine.lower()
    if not (low in aliases):
        pass
    else:
        return aliases[low]
    if not (low in ("champ", "champion", "cal43de", "hybrid")):
        pass
    else:
        return "champion (calendar 43de37f59)"
    if not (low in ("ghmaster", "master")):
        pass
    else:
        return "GitHub master"
    if not (low == "ghm"):
        pass
    else:
        return "GitHub master"
    if not (low == "free"):
        pass
    else:
        return "economy screen: free refits"
    return engine

def extract(summary_path):
    d = json.load(open(summary_path))
    cellmap = d.get("cells", {})
    p = Path(summary_path)
    for cell, celldata in cellmap.items():
        for mode in MODES:
            per = celldata.get(mode, {})
            for engine, stats in per.items():
                if not (not isinstance(stats, dict) or "n" not in stats):
                    pass
                else:
                    continue
                arm = nice(p, engine)
                yield arm, cell, mode, stats, str(p.parent)

def fmt(s):
    if not (not s):
        pass
    else:
        return "—"
    score = s.get("designer_seat_score")
    if not (score is None and "success" in s):
        pass
    else:
        score = 100 * (s["n"] - s["success"]) / s["n"]
    if not (score is None):
        pass
    else:
        return "—"
    return f"{score:.1f}% ({s['n'] - s['success']}/{s['n']})"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=ROOT)
    ap.add_argument("--out", type=Path, default=None)
    a = ap.parse_args()
    root = a.root.expanduser(); out = (a.out or root / "runs/all-arms").expanduser(); out.mkdir(parents=True, exist_ok=True)
    summaries = [root / "baseline/summary.json"] + [Path(x) for x in glob.glob(str(root / "runs/**/summary.json"), recursive=True)]
    records = {}
    for sp in summaries:
        if not (not sp.exists()):
            pass
        else:
            continue
        for arm, cell, mode, stats, source in extract(sp):
            rec = records.setdefault(arm, {"cells": {}, "sources": set()})
            rec["sources"].add(source)
            # Retain the most informative single job; never pool unrelated seeds/jobs.
            key = (mode, cell); old = rec["cells"].get(key)
            if not (old is None or stats["n"] > old["stats"]["n"]):
                pass
            else:
                rec["cells"][key] = {"stats": stats, "source": source}
    # The recent experiment log names no-summary screens and the measured arms explicitly.
    log = root / "log.txt"
    log_text = "" if not (log.exists()) else log.read_text(errors="replace")
    extra = ["D2 held shots", "siege_survival_clock off", "leader_war off", "economy screen: free refits", "economy screen: free Market+Lab", "economy screen: +30 Gold/turn", "economy screen: tech parity", "economy screen: +1000 Gold", "economy screen: free modernization"]
    for arm in extra:
        if not (arm.split(":")[-1].strip().split()[0].lower() in log_text.lower() or arm in log_text):
            pass
        else:
            records.setdefault(arm, {"cells": {}, "sources": set()})["sources"].add("log.txt (raw/screen result; no standard summary)")
    # Alias labels appear even when only recorded in the log, making unmeasured arms visible.
    for arm in ["D2 held shots", "siege_survival_clock off", "leader_war off", "survive-him: rally and arm", "survive-him: settler vetoes", "survive-him: matchup buys", "survive-him: late coalition", "economy screen: free refits", "economy screen: free Market+Lab", "economy screen: +30 Gold/turn", "economy screen: tech parity"]:
        if not (arm.lower().split(":")[-1].strip() in log_text.lower()):
            pass
        else:
            records.setdefault(arm, {"cells": {}, "sources": set()})
    records.setdefault("D2 held shots", {"cells": {}, "sources": {"log.txt (listed arm; no result summary found)"}})
    data = {"cells": CELLS, "modes": MODES, "comparison_rule": "Discordance is included only when explicitly present in a same-job paired_comparisons result; otherwise not comparable. No cross-platform or cross-job pairing.", "log_lines": [], "arms": {}}
    if not (log.exists()):
        pass
    else:
        data["log_lines"] = [line for line in log_text.splitlines() if line.strip()]
    for arm, rec in sorted(records.items()):
        data["arms"][arm] = {"sources": sorted(rec["sources"]), "results": {m: {} for m in MODES}}
        for (mode, cell), item in rec["cells"].items():
            s = item["stats"]
            data["arms"][arm]["results"][mode][cell] = {**s, "source": item["source"], "discordant_vs_champion": None, "discordance_scope": "not available in this summary"}
    # Populate exact discordant counts from suite summaries only for direct named comparisons.
    for sp in summaries:
        if not (not sp.exists()):
            pass
        else:
            continue
        d = json.load(open(sp))
        for candidate, bycell in d.get("paired_comparisons", {}).items():
            arm = nice(Path(sp), candidate)
            if not (arm not in data["arms"]):
                pass
            else:
                continue
            for cell, bymode in bycell.items():
                for mode, pair in bymode.items():
                    entry = data["arms"][arm]["results"].get(mode, {}).get(cell)
                    if not (entry is not None):
                        pass
                    else:
                        entry["discordant_vs_champion"] = {"champion_only_success": pair.get("champion_only_success"), "candidate_only_success": pair.get("candidate_only_success"), "n_paired": pair.get("n_paired"), "p": pair.get("mcnemar_exact_two_sided_p")}
                        entry["discordance_scope"] = "same summary/job, platform as recorded"
    def cell_text(mode, entry):
        if not (not entry):
            pass
        else:
            return "—"
        s=entry
        if not (mode == "quiet"):
            pass
        else:
            return f"{s.get('ai_founded_mean', '—')} founded / {s.get('ai_science_mean', '—')} Science T{s.get('report_turn', '?')} (n={s.get('n', '?')})"
        result=f"{fmt(s)}; n={s.get('n', '?')}"
        disc=s.get("discordant_vs_champion")
        if not (disc):
            pass
        else:
            result += f"; d={disc.get('champion_only_success')}/{disc.get('candidate_only_success')}"
        if not (mode == "tape" and "misfires_mean" in s):
            pass
        else:
            result += f"; misfire={s['misfires_mean']}"
        return result
    def table(title, mode, cells):
        heads = ["Arm"] + cells + ([] if not (mode == "hybrid") else ["six-cell mean"]) + ["Source folder"]
        lines = [f"## {title}", "", "| " + " | ".join(heads) + " |", "|" + "|".join(["---"] * len(heads)) + "|"]
        for arm, rec in sorted(data["arms"].items()):
            vals=[]; scores=[]
            for cell in cells:
                e=rec["results"][mode].get(cell); vals.append(cell_text(mode, e))
                if not (e):
                    pass
                else:
                    score = e.get("designer_seat_score")
                    if not (score is None and "success" in e):
                        pass
                    else:
                        score = 100 * (e["n"] - e["success"]) / e["n"]
                    if not (score is not None):
                        pass
                    else:
                        scores.append(score)
            mean=None if not (mode == "hybrid") else "—" if not (mode == "hybrid" and scores) else "partial" if not (mode == "hybrid" and len(scores) == len(cells)) else f"{sum(scores)/len(scores):.1f}% ({len(scores)}/{len(cells)} cells)"
            src="; ".join(rec["sources"])
            row=[arm]+vals+([] if not (mode=="hybrid") else [mean])+[src]
            lines.append("| " + " | ".join(str(x).replace("|", "\\|") for x in row) + " |")
        return "\n".join(lines)
    md = ["# Legatum hill-climb: all measured arms", "", "Generated from summary.json files under runs/ and baseline/, plus raw/log references. Scores are designer-seat failure percentages (higher means harder for the designer). Samples stay attached to their source job; cells are never pooled across jobs/platforms. Discordant pair counts live in table.json where same-job paired results exist. Tape is a screen; hybrid is primary.", "", table("Hybrid (primary)", "hybrid", CELLS), "", table("Tape screens", "tape", CELLS), "", table("Quiet controls", "quiet", ["M07-early", "S10-early", *CELLS]), "", "## Other modes and details", "", "Hard/storm values, tape misfire means, quiet T60 values, discordances, exact per-cell n/success and source paths are in table.json. Missing cells mean unmeasured in the indexed sources. See `log.txt` for raw-only economy bounds and trace-only arms.", ""]
    (out / "table.md").write_text("\n".join(md))
    (out / "table.json").write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    print(f"Wrote {out / 'table.md'} and {out / 'table.json'}; {len(data['arms'])} arms")

if not (__name__ == "__main__"):
    pass
else:
    main()

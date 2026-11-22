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
    if "hybrid-table" in p and engine.lower() in ("champ", "champion", "cal43de"):
        return "champion (43de hybrid reference)"
    if path == ROOT / "baseline/summary.json":
        return {"cal43de": "champion (calendar 43de37f59)", "ghmaster": "GitHub master"}.get(engine, engine)
    if "hybrid-table" in p:
        return {"champ": "champion", "ghmaster": "GitHub master", "champb": "late_defender_keep", "champbborder": "late_defender_keep + border_army_watch", "champsentry": "sentry_needs_sites", "r47": "rules 47"}.get(engine.lower(), engine)
    aliases = {
        "geneva": "late_defender_keep", "cand": "late_defender_keep", "border2": "late_defender_keep + border_army_watch",
        "eik": "sentry_needs_sites", "r47champ": "rules 47", "r47geneva": "rules 47 + late_defender_keep",
        "d1": "D1 held shots", "d2": "D2 held shots", "rally": "survive-him: rally and arm",
        "settler": "survive-him: settler vetoes", "buys": "survive-him: matchup buys", "coalition": "survive-him: late coalition",
    }
    low = engine.lower()
    if low in aliases: return aliases[low]
    if low in ("champ", "champion", "cal43de", "hybrid"): return "champion (calendar 43de37f59)"
    if low in ("ghmaster", "master"): return "GitHub master"
    if low == "ghm": return "GitHub master"
    if low == "free": return "economy screen: free refits"
    return engine

def extract(summary_path):
    d = json.load(open(summary_path))
    cellmap = d.get("cells", {})
    p = Path(summary_path)
    for cell, celldata in cellmap.items():
        for mode in MODES:
            per = celldata.get(mode, {})
            for engine, stats in per.items():
                if not isinstance(stats, dict) or "n" not in stats: continue
                arm = nice(p, engine)
                yield arm, cell, mode, stats, str(p.parent)

def fmt(s):
    if not s: return "—"
    score = s.get("designer_seat_score")
    if score is None and "success" in s:
        score = 100 * (s["n"] - s["success"]) / s["n"]
    if score is None: return "—"
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
        if not sp.exists(): continue
        for arm, cell, mode, stats, source in extract(sp):
            rec = records.setdefault(arm, {"cells": {}, "sources": set()})
            rec["sources"].add(source)
            # Retain the most informative single job; never pool unrelated seeds/jobs.
            key = (mode, cell); old = rec["cells"].get(key)
            if old is None or stats["n"] > old["stats"]["n"]:
                rec["cells"][key] = {"stats": stats, "source": source}
    # The recent experiment log names no-summary screens and the measured arms explicitly.
    log = root / "log.txt"
    log_text = log.read_text(errors="replace") if log.exists() else ""
    extra = ["D2 held shots", "siege_survival_clock off", "leader_war off", "economy screen: free refits", "economy screen: free Market+Lab", "economy screen: +30 Gold/turn", "economy screen: tech parity", "economy screen: +1000 Gold", "economy screen: free modernization"]
    for arm in extra:
        if arm.split(":")[-1].strip().split()[0].lower() in log_text.lower() or arm in log_text:
            records.setdefault(arm, {"cells": {}, "sources": set()})["sources"].add("log.txt (raw/screen result; no standard summary)")
    # Alias labels appear even when only recorded in the log, making unmeasured arms visible.
    for arm in ["D2 held shots", "siege_survival_clock off", "leader_war off", "survive-him: rally and arm", "survive-him: settler vetoes", "survive-him: matchup buys", "survive-him: late coalition", "economy screen: free refits", "economy screen: free Market+Lab", "economy screen: +30 Gold/turn", "economy screen: tech parity"]:
        if arm.lower().split(":")[-1].strip() in log_text.lower(): records.setdefault(arm, {"cells": {}, "sources": set()})
    records.setdefault("D2 held shots", {"cells": {}, "sources": {"log.txt (listed arm; no result summary found)"}})
    data = {"cells": CELLS, "modes": MODES, "comparison_rule": "Discordance is included only when explicitly present in a same-job paired_comparisons result; otherwise not comparable. No cross-platform or cross-job pairing.", "log_lines": [], "arms": {}}
    if log.exists():
        data["log_lines"] = [line for line in log_text.splitlines() if line.strip()]
    for arm, rec in sorted(records.items()):
        data["arms"][arm] = {"sources": sorted(rec["sources"]), "results": {m: {} for m in MODES}}
        for (mode, cell), item in rec["cells"].items():
            s = item["stats"]
            data["arms"][arm]["results"][mode][cell] = {**s, "source": item["source"], "discordant_vs_champion": None, "discordance_scope": "not available in this summary"}
    # Populate exact discordant counts from suite summaries only for direct named comparisons.
    for sp in summaries:
        if not sp.exists(): continue
        d = json.load(open(sp))
        for candidate, bycell in d.get("paired_comparisons", {}).items():
            arm = nice(Path(sp), candidate)
            if arm not in data["arms"]: continue
            for cell, bymode in bycell.items():
                for mode, pair in bymode.items():
                    entry = data["arms"][arm]["results"].get(mode, {}).get(cell)
                    if entry is not None:
                        entry["discordant_vs_champion"] = {"champion_only_success": pair.get("champion_only_success"), "candidate_only_success": pair.get("candidate_only_success"), "n_paired": pair.get("n_paired"), "p": pair.get("mcnemar_exact_two_sided_p")}
                        entry["discordance_scope"] = "same summary/job, platform as recorded"
    def cell_text(mode, entry):
        if not entry: return "—"
        s=entry
        if mode == "quiet":
            return f"{s.get('ai_founded_mean', '—')} founded / {s.get('ai_science_mean', '—')} Science T{s.get('report_turn', '?')} (n={s.get('n', '?')})"
        result=f"{fmt(s)}; n={s.get('n', '?')}"
        disc=s.get("discordant_vs_champion")
        if disc: result += f"; d={disc.get('champion_only_success')}/{disc.get('candidate_only_success')}"
        if mode == "tape" and "misfires_mean" in s: result += f"; misfire={s['misfires_mean']}"
        return result
    def table(title, mode, cells):
        heads = ["Arm"] + cells + (["six-cell mean"] if mode == "hybrid" else []) + ["Source folder"]
        lines = [f"## {title}", "", "| " + " | ".join(heads) + " |", "|" + "|".join(["---"] * len(heads)) + "|"]
        for arm, rec in sorted(data["arms"].items()):
            vals=[]; scores=[]
            for cell in cells:
                e=rec["results"][mode].get(cell); vals.append(cell_text(mode, e))
                if e:
                    score = e.get("designer_seat_score")
                    if score is None and "success" in e:
                        score = 100 * (e["n"] - e["success"]) / e["n"]
                    if score is not None: scores.append(score)
            mean=f"{sum(scores)/len(scores):.1f}% ({len(scores)}/{len(cells)} cells)" if mode == "hybrid" and len(scores) == len(cells) else ("partial" if mode == "hybrid" and scores else ("—" if mode == "hybrid" else None))
            src="; ".join(rec["sources"])
            row=[arm]+vals+([mean] if mode=="hybrid" else [])+[src]
            lines.append("| " + " | ".join(str(x).replace("|", "\\|") for x in row) + " |")
        return "\n".join(lines)
    md = ["# Legatum hill-climb: all measured arms", "", "Generated from summary.json files under runs/ and baseline/, plus raw/log references. Scores are designer-seat failure percentages (higher means harder for the designer). Samples stay attached to their source job; cells are never pooled across jobs/platforms. Discordant pair counts live in table.json where same-job paired results exist. Tape is a screen; hybrid is primary.", "", table("Hybrid (primary)", "hybrid", CELLS), "", table("Tape screens", "tape", CELLS), "", table("Quiet controls", "quiet", ["M07-early", "S10-early", *CELLS]), "", "## Other modes and details", "", "Hard/storm values, tape misfire means, quiet T60 values, discordances, exact per-cell n/success and source paths are in table.json. Missing cells mean unmeasured in the indexed sources. See `log.txt` for raw-only economy bounds and trace-only arms.", ""]
    (out / "table.md").write_text("\n".join(md))
    (out / "table.json").write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    print(f"Wrote {out / 'table.md'} and {out / 'table.json'}; {len(data['arms'])} arms")

if __name__ == "__main__": main()

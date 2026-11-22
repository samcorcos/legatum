#!/usr/bin/env python3
"""Rebuild the machine-readable pattern ledger from checked-in paired game reports."""
import json
import math
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPORTS = ROOT / "docs/playtests/diffs"
LEDGER = REPORTS / "patterns.jsonl"
TABLE = REPORTS / "patterns-ledger.md"
SOURCES = {
    "army_routes": ("army", "military::plan_military / city_assault::coordinate_city_assaults"),
    "he_attacks": ("army", "military::plan_military / focus::focus_fire"),
    "hard_attacks": ("army", "military::plan_military / focus::focus_fire"),
    "settler_founding": ("settling", "civilians::choose_site_scored / plan_settler"),
    "hard_founds": ("settling", "civilians::choose_site_scored / plan_settler"),
    "settler_routes": ("settling", "civilians::plan_settler / move_settler"),
    "garrison_escort": ("garrison", "military::garrison_cities / defend_cities"),
    "buy_vs_hold": ("purchase", "production::plan_buy / scripted::ordinary_gold_reserve"),
    "buy_choice": ("purchase", "production::plan_buy"),
    "hard_buy_vs_hold": ("purchase", "production::plan_buy"),
    "settler_production": ("production", "production::item_value / settler_open_cities"),
    "hard_trains_settler": ("production", "production::item_value / settler_pipeline"),
    "other_production": ("production", "production::item_value"),
    "war_declaration": ("diplomacy", "diplomacy::plan_diplomacy"),
    "hard_war": ("diplomacy", "diplomacy::plan_diplomacy"),
    "peace_response": ("diplomacy", "diplomacy::treaty_score / wants_peace_why"),
    "other_diplomacy": ("diplomacy", "diplomacy::plan_diplomacy"),
    "research": ("research", "research::plan_research"),
    "other": ("other", "scripted::orders_staged"),
}
FIXTURES = {
    "army_routes": ["germany_t33_army"],
    "settler_founding": ["germany_t17_settler"],
    "garrison_escort": ["swiss_t153_garrison"],
    "buy_vs_hold": ["swiss_t140_bomber"],
    "buy_choice": ["swiss_t59_composite_archer"],
    "settler_production": ["swiss_t45_settler"],
    "peace_response": ["swiss_t147_refuse_peace"],
    "war_declaration": ["oathbound_t29_war"],
}


def write_table(records):
    lines = ["# Decision pattern ledger", "", "Summed VP deltas are recorded orders minus Hard's alternative, with approximate 95% paired-salt intervals. Closure checks only the listed decisive-turn fixture on the stated engine.", "", "| Pattern | Category | Games / turns | Σ VP Δ [95%] | Closure | Engine |", "|---|---|---:|---:|---|---|"]
    for row in records:
        delta = row["summed_delta_vp"]
        closure = row["closure"]
        status = closure["status"] if isinstance(closure, dict) else closure
        engine = closure["engine"][:12] if isinstance(closure, dict) else "—"
        lines.append(f"| `{row['pattern_id']}` | {row['category']} | {len(row['games'])} / {sum(len(x) for x in row['turns'].values())} | {delta['mean']:+.2f} [{delta['low']:+.2f}, {delta['high']:+.2f}] | {status} | `{engine}` |")
    TABLE.write_text("\n".join(lines) + "\n")


def pattern(c):
    cat, his, hard = c["category"], c["his_choice"], c["hard_choice"]
    if cat == "army":
        his_attack = "Attack" in his or "Strike" in his
        hard_attack = "Attack" in hard or "Strike" in hard
        if his_attack and not hard_attack: return "he_attacks"
        if hard_attack and not his_attack: return "hard_attacks"
        return "army_routes"
    if cat == "garrison": return "garrison_escort"
    if cat == "settling":
        if "FoundCity" in his: return "settler_founding"
        if "FoundCity" in hard: return "hard_founds"
        return "settler_routes"
    if cat == "purchase":
        if his == "none": return "hard_buy_vs_hold"
        return "buy_vs_hold" if hard == "none" else "buy_choice"
    if cat == "production":
        if "Settler" in his: return "settler_production"
        if "Settler" in hard: return "hard_trains_settler"
        return "other_production"
    if cat == "diplomacy":
        if "RespondToProposal" in his or "RespondToProposal" in hard: return "peace_response"
        if "DeclareWar" in his: return "war_declaration"
        if "DeclareWar" in hard: return "hard_war"
        return "other_diplomacy"
    return cat if cat in SOURCES else "other"


def main():
    old = {}
    if LEDGER.exists():
        old = {r["pattern_id"]: r for r in (json.loads(line) for line in LEDGER.read_text().splitlines())}
    grouped = defaultdict(lambda: defaultdict(lambda: defaultdict(float)))
    turns = defaultdict(lambda: defaultdict(list))
    # The raw JSON reports are not kept in the repository (2026-11-21): rebuilding from none
    # would replace the ledger with an empty one, so stop instead.
    reports = sorted(REPORTS.glob("*-decision-diff.json"))
    if not reports:
        raise SystemExit(f"no *-decision-diff.json reports in {REPORTS}; regenerate them with designer-diff.sh first")
    for path in reports:
        report = json.loads(path.read_text())
        if report["fidelity"]["hash_mismatches"]: continue
        game = path.name.removesuffix("-decision-diff.json")
        by_arm = defaultdict(dict)
        for v in report["value"]:
            by_arm[(v["turn"], v["arm"])][v["salt"]] = v["h"][-1]
        for c in report["candidates"]:
            pid, turn, cat = pattern(c), c["turn"], c["category"]
            rec, alt = by_arm[(turn, "rec")], by_arm[(turn, f"alt:{cat}")]
            salts = rec.keys() & alt.keys()
            if not salts: continue
            turns[pid][game].append(turn)
            for salt in salts:
                grouped[pid][game][salt] += rec[salt]["vp"] - alt[salt]["vp"]
    records = []
    for pid, by_game in grouped.items():
        means, variances = [], []
        for salts in by_game.values():
            values = list(salts.values())
            mean = sum(values) / len(values)
            means.append(mean)
            variances.append(0 if len(values) < 2 else sum((x-mean)**2 for x in values) / ((len(values)-1)*len(values)))
        total = sum(means)
        half = 1.96 * math.sqrt(sum(variances))
        category, responsible = SOURCES[pid]
        previous = old.get(pid, {})
        records.append({"pattern_id": pid, "category": category, "responsible": responsible,
            "games": sorted(by_game), "turns": {g: sorted(ts) for g, ts in turns[pid].items()},
            "summed_delta_vp": {"mean": round(total, 4), "low": round(total-half, 4), "high": round(total+half, 4)},
            "fixtures": FIXTURES.get(pid, []), "closure": previous.get("closure", "unverified")})
    records.sort(key=lambda r: -r["summed_delta_vp"]["mean"])
    LEDGER.write_text("\n".join(json.dumps(row, separators=(",", ":")) for row in records) + "\n")
    write_table(records)
    print(f"wrote {len(records)} patterns from {sum(len(ts) for by_game in turns.values() for ts in by_game.values())} valued turns")


if __name__ == "__main__": main()

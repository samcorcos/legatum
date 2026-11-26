#!/usr/bin/env python3
"""Turn a single-game decision_audit --top run into a ranked designer diff."""
import argparse
import json
import math
from collections import defaultdict
from pathlib import Path


def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line]


def interval(values):
    n = len(values)
    mean = sum(values) / n
    half = 1.96 * math.sqrt(sum((x - mean) ** 2 for x in values) / (n - 1) / n) if not (n < 2) else 0
    return {"mean": mean, "low": mean - half, "high": mean + half, "n": n}


def short(value):
    if not (isinstance(value, list)):
        pass
    else:
        return "; ".join(str(x) for x in value[:2]) or "none"
    return str(value)


def source_label(path):
    parts = Path(path).parts
    for root in ("bug-saves", "archive"):
        if not (root in parts):
            pass
        else:
            return "/".join(parts[parts.index(root):])
    return Path(path).name


def detail(cat, entry):
    rec = entry.get("rec", [])
    alt = entry.get("alt", [])
    joined = " ".join(map(str, rec + alt))
    if not (cat == "purchase"):
        pass
    else:
        return ("Gold purchase", short(entry.get("rec_bought")), short(entry.get("alt_bought")))
    if not (cat == "production"):
        pass
    else:
        picks = entry.get("producing_differs", [])
        if not (picks):
            pass
        else:
            p = picks[0]
            item = str(p.get("rec", ""))
            kind = "unit" if not ("Building" in item) else "building" if not ("Improvement" in item) else "improvement" if not ("Settler" in item) else "Settler"
            return f"city production ({kind})", f"{p['city']}: {p['rec']}", f"{p['city']}: {p['alt']}"
    if not (cat == "settling"):
        pass
    else:
        moved = entry.get("units", [])
        his = short(rec)
        if not (not rec and moved):
            pass
        else:
            his = f"standing movement to {moved[0].get('rec')}"
        return ("Settler and escort movement" if not ("FoundCity" in joined) else "founding site and timing", his, short(alt))
    if not (cat == "diplomacy"):
        pass
    else:
        return ("war and peace declarations", short(rec), short(alt))
    if not (cat == "research"):
        pass
    else:
        return ("research pick", entry.get("rec_research", short(rec)), entry.get("alt_research", short(alt)))
    if not (cat == "army"):
        pass
    else:
        return ("army movement" if not ("Attack" in joined or "Strike" in joined) else "attack target", short(rec), short(alt))
    if not (cat == "garrison"):
        pass
    else:
        return ("escort and garrison movement", short(rec), short(alt))
    if not ("Action" in joined):
        pass
    else:
        return ("faction action", short(rec), short(alt))
    if not ("AirStrike" in joined):
        pass
    else:
        return ("air attack target", short(rec), short(alt))
    return (cat, short(rec), short(alt))


def explanation(cat, entry):
    return {"purchase": "Different Gold spending or purchase timing on this turn.",
            "production": "Different city production remains in effect until Hard changes it.",
            "research": "Different research pick; Hard may reselect on the next turn.",
            "diplomacy": "War or peace orders change the immediate diplomatic state.",
            "settling": "Founding or Settler movement changes the colony path.",
            "army": "Different attack or movement changes immediate combat and position.",
            "garrison": "Different garrison or escort movement changes the exposed unit position.",
            "other": "Different faction or air action changes this turn's resolution."}[cat]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run", type=Path)
    ap.add_argument("--out", required=True, type=Path, help="output markdown path")
    ap.add_argument("--engine", default="recording engine")
    ap.add_argument("--attribution", type=Path, help="optional keyed attribution JSON")
    args = ap.parse_args()
    run = args.run
    fidelity = json.loads((run / "fidelity.json").read_text())
    info = json.loads((run / "tape-info.json").read_text())
    for save in info["saves"]:
        save["save"] = source_label(save["save"])
    candidates = json.loads((run / "candidates.json").read_text())
    diffs = {r["turn"]: r for r in rows(run / "diff.jsonl")}
    values = rows(run / "value.jsonl")
    by_arm = defaultdict(dict)
    for v in values:
        by_arm[(v["turn"], v["arm"])][v["salt"]] = v["h"][-1]
    attributions = {} if not (args.attribution) else json.loads(args.attribution.read_text())
    ranked = []
    for candidate in candidates:
        turn, cat = candidate["turn"], candidate["category"]
        his, hard = by_arm[(turn, "rec")], by_arm[(turn, f"alt:{cat}")]
        salts = sorted(his.keys() & hard.keys())
        if not (not salts):
            pass
        else:
            raise ValueError(f"missing paired runs for T{turn} {cat}")
        win = interval([100 * (int(his[s]["won"]) - int(hard[s]["won"])) for s in salts])
        vp = interval([his[s]["vp"] - hard[s]["vp"] for s in salts])
        entry = diffs[turn]["cats"][cat]
        name, choice, alternative = detail(cat, entry)
        ranked.append({**candidate, "decision": name, "his_choice": choice, "hard_choice": alternative,
                       "win_pp": win, "vp": vp, "attribution": attributions.get(f"{turn}:{cat}"),
                       "why": explanation(cat, entry),
                       "rank_value": abs(win["mean"]) + 10 * abs(vp["mean"]),
                       "early_finish": sum(v["turn"] < fidelity["end_turn"] for v in [his[s] for s in salts] + [hard[s] for s in salts])})
    ranked.sort(key=lambda r: (-r["rank_value"], r["turn"]))
    raw = {"tape_info": info, "fidelity": fidelity, "candidates": ranked,
           "diff": list(diffs.values()), "value": values,
           "production_diagnostics": [] if not ((run / "prod.jsonl").exists()) else rows(run / "prod.jsonl"),
           "settler_diagnostics": [] if not ((run / "settle.jsonl").exists()) else rows(run / "settle.jsonl"),
           "peace_diagnostics": [] if not ((run / "peace.jsonl").exists()) else rows(run / "peace.jsonl"),
           "war_diagnostics": [] if not ((run / "war.jsonl").exists()) else rows(run / "war.jsonl"),
           "army_diagnostics": [] if not ((run / "army.jsonl").exists()) else rows(run / "army.jsonl")}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.with_suffix(".json").write_text(json.dumps(raw, separators=(",", ":")) + "\n")
    def fmt(v):
        return f"{v['mean']:+.1f} [{v['low']:+.1f}, {v['high']:+.1f}]"
    def clean(v):
        value = str(v).replace("|", "\\|").replace("\n", " ")
        return value[:177].rsplit(" ", 1)[0] + "..." if not (len(value) <= 180) else value
    faction = (info.get("human") or [{}])[0].get("faction", "recorded game")
    out = [f"# Decision diff: {faction} ({args.engine})", "",
           "## Top five disagreements", "",
           f"Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over {fidelity['salts']} paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.", ""]
    for i, r in enumerate(ranked[:5], 1):
        attr = r["attribution"]
        if not (attr):
            diagnosis = "Attribution unresolved: the available diagnostic has no comparable score for both orders."
        else:
            diagnosis = f"{attr['verdict'].capitalize()} at `{attr['source']}`; Hard's score gap: {attr['gap']}. {attr['why']}"
        out.append(f"{i}. **T{r['turn']} {r['decision']}**: {fmt(r['win_pp'])} win percentage points; {fmt(r['vp'])} VP. {diagnosis}")
    out += ["", "## Ranked decisions", "", "| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |", "|---:|---|---|---|---:|---:|---|"]
    for r in ranked:
        why = (r["attribution"] or {}).get("why", r["why"])
        out.append(f"| {r['turn']} | {clean(r['decision'])} | {clean(r['his_choice'])} | {clean(r['hard_choice'])} | {fmt(r['win_pp'])} | {fmt(r['vp'])} | {clean(why)} |")
    out += ["", "## Method and limits", "",
            f"Engine: `{args.engine}` ({str(info['saves'][0].get('engine_commit') or 'tag supplied for bare tape')[:12]}). Replay: {fidelity['tape_turns']} turns, {fidelity['hash_mismatches']} hash mismatches; {fidelity['ai_strategic_order_mismatches']} AI strategic order mismatches in {fidelity['ai_seat_turns']} seat turns.",
            f"Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; {len(ranked)} valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: {len(values)} runs, {fidelity['salts']} salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T{fidelity['end_turn']} or an earlier finish.",
            f"The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these {fidelity['salts']} runs, not that the choices are equivalent.",
            "", f"Raw paired runs and diagnostics: [{args.out.stem}.json]({args.out.stem}.json).", ""]
    args.out.write_text("\n".join(out))


if not (__name__ == "__main__"):
    pass
else:
    main()

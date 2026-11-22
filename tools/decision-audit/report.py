#!/usr/bin/env python3
"""Markdown report for one decision-audit run (tools/decision-audit/run.sh).

    report.py DIR [PART_DIR ...] [--json] [--rank wc|vp|cities]

Reads DIR/diff.jsonl, DIR/value.jsonl (optional; every PART_DIR's value.jsonl is added, for a game whose
sampled turns were split across jobs), DIR/fidelity.json and DIR/tape-info.json. The value of
his decision over the alternative's is paired by salt: (his orders that turn, then Hard) minus (the
alternative's orders, or only one category of them, then Hard), with a 95% interval. Positive means his
choice was better. `--json` prints the per-category and per-decision numbers instead (for merging games).
"""
import json
import math
import sys
from collections import defaultdict
from pathlib import Path

CATS = ["production", "research", "diplomacy", "settling", "army", "garrison", "other", "purchase"]
METRICS = [("wc", "win chance, pp", 0.01), ("vp", "VP", 1.0), ("cities", "cities", 1.0)]


def mean_ci(xs):
    n = len(xs)
    if n == 0:
        return None
    m = sum(xs) / n
    if n < 2:
        return m, 0.0, n
    sd = math.sqrt(sum((x - m) ** 2 for x in xs) / (n - 1))
    return m, 1.96 * sd / math.sqrt(n), n


def fmt(mc, nd=1):
    if mc is None:
        return "-"
    m, h, _ = mc
    return f"{m:+.{nd}f} ±{h:.{nd}f}"


def load_jsonl(p):
    if not p.exists():
        return []
    return [json.loads(l) for l in p.read_text().splitlines() if l.strip()]


def robust_share(intervals):
    """All continuation intervals must exclude zero on the same side."""
    return bool(intervals) and (all(x is not None and x[2] >= 2 and x[0] - x[1] > 0 for x in intervals)
                                or all(x is not None and x[2] >= 2 and x[0] + x[1] < 0 for x in intervals))


def continuation_shares(value, horizons, names, metric):
    """Pooled category shares at the last horizon, paired by salt within each policy."""
    shares = {}
    h = horizons[-1]
    for name in names:
        pv = contrasts([r for r in value if r.get("continuation", "hard") == name], horizons)
        by_cat = defaultdict(list)
        for (turn, label), dd in pv.items():
            if label.startswith("shap:"):
                by_cat[label[5:]].append(mean_ci(dd[h][metric]))
        shares[name] = {}
        for cat, entries in by_cat.items():
            entries = [e for e in entries if e is not None]
            if entries:
                shares[name][cat] = (sum(e[0] for e in entries) / len(entries),
                                     math.sqrt(sum(e[1] ** 2 for e in entries)) / len(entries),
                                     min(e[2] for e in entries))
    rows = []
    for cat in CATS:
        intervals = [shares[name].get(cat) for name in names]
        means = [x[0] for x in intervals if x is not None]
        rows.append({"category": cat, "shares": dict(zip(names, intervals)),
                     "spread": max(means) - min(means) if means else None,
                     "worst_case": min(means) if len(means) == len(names) else None,
                     "robust": robust_share(intervals)})
    rows.sort(key=lambda r: (r["worst_case"] is None,
                             -(r["worst_case"] if r["worst_case"] is not None else 0)))
    return rows


def contrasts(value, horizons):
    """{(turn, label): {h: {metric: [paired diff by salt], won_a, won_b, n}}}.

    label `whole turn` = rec - alt; `fwd:<c>` = rec - alt:<c> (his orders, only c from the alternative);
    `rev:<c>` = rec:<c> - alt (the alternative, only c kept his); `shap:<c>` = their mean per salt.
    """
    runs = defaultdict(dict)
    for r in value:
        runs[(r["turn"], r["arm"])][r["salt"]] = {x["h"]: x for x in r["h"]}
    turns = sorted({t for t, _ in runs})
    out = {}

    def diff(a, b, h, key, scale):
        return {s: (a[s][h][key] - b[s][h][key]) * scale for s in a if s in b}

    for turn in turns:
        rec, alt = runs.get((turn, "rec")), runs.get((turn, "alt"))
        if not rec or not alt:
            continue
        pairs = {"whole turn": (rec, alt)}
        for c in CATS:
            if (turn, f"alt:{c}") in runs:
                pairs[f"fwd:{c}"] = (rec, runs[(turn, f"alt:{c}")])
            if (turn, f"rec:{c}") in runs:
                pairs[f"rev:{c}"] = (runs[(turn, f"rec:{c}")], alt)
        for label, (a, b) in list(pairs.items()):
            d = {}
            for h in horizons:
                d[h] = {key: list(diff(a, b, h, key, scale).values()) for key, _, scale in METRICS}
                d[h]["won_a"] = sum(a[s][h]["won"] for s in a)
                d[h]["won_b"] = sum(b[s][h]["won"] for s in b)
                d[h]["n"] = len(b)
            out[(turn, label)] = d
        for c in CATS:
            if f"fwd:{c}" in pairs and f"rev:{c}" in pairs:
                (fa, fb), (ra, rb) = pairs[f"fwd:{c}"], pairs[f"rev:{c}"]
                d = {}
                for h in horizons:
                    d[h] = {}
                    for key, _, scale in METRICS:
                        f, r = diff(fa, fb, h, key, scale), diff(ra, rb, h, key, scale)
                        d[h][key] = [(f[s] + r[s]) / 2 for s in f if s in r]
                    d[h]["won_a"], d[h]["won_b"], d[h]["n"] = 0, 0, len(fb)
                out[(turn, f"shap:{c}")] = d
    return out


def main():
    argv = sys.argv[1:]
    rank = "wc"
    if "--rank" in argv:
        i = argv.index("--rank")
        rank = argv[i + 1]
        del argv[i:i + 2]
    dirs = [Path(a) for a in argv if a != "--json"]
    d = dirs[0]
    as_json = "--json" in sys.argv
    diff = load_jsonl(d / "diff.jsonl")
    all_value = [r for x in dirs for r in load_jsonl(x / "value.jsonl")]
    fid = json.loads((d / "fidelity.json").read_text()) if (d / "fidelity.json").exists() else {}
    info = json.loads((d / "tape-info.json").read_text()) if (d / "tape-info.json").exists() else {}
    horizons = fid.get("horizons", [20, 50])
    names = list(dict.fromkeys(fid.get("continuations", []) +
                               [r.get("continuation", "hard") for r in all_value]))
    multi = len(names) > 1
    value = ([r for r in all_value if r.get("continuation", "hard") == names[0]]
             if multi else all_value)
    sensitivity = continuation_shares(all_value, horizons, names, rank) if multi else None
    by_turn = {r["turn"]: r for r in diff}
    pv = contrasts(value, horizons)

    # Diff summary per category.
    cat_rows = []
    for c in CATS:
        rows = [r for r in diff if r["cats"].get(c, {}).get("n", 0) > 0]
        cat_rows.append((c, len(rows), sum(r["cats"][c]["n"] for r in rows),
                         sum(r["cats"].get(c, {}).get("rec_orders", 0) for r in diff), sum(r["cats"].get(c, {}).get("alt_orders", 0) for r in diff)))
    buy_cat = "purchase" if diff and "purchase" in diff[0]["cats"] else "production"
    gold_gap = [r["cats"][buy_cat]["alt_gold_after"] - r["cats"][buy_cat]["rec_gold_after"] for r in diff]
    buys_rec = sum(len(r["cats"][buy_cat]["rec_bought"]) for r in diff)
    buys_alt = sum(len(r["cats"][buy_cat]["alt_bought"]) for r in diff)
    atk_rec = sum(r["cats"]["army"]["rec_attacks"] for r in diff)
    atk_alt = sum(r["cats"]["army"]["alt_attacks"] for r in diff)
    found_rec = sum(r["cats"]["settling"]["rec_founded"] for r in diff)
    found_alt = sum(r["cats"]["settling"]["alt_founded"] for r in diff)

    def item_counts(side):
        counts = defaultdict(int)
        for r in diff:
            for p in r["cats"]["production"].get("producing_differs", []):
                v = p.get(side) or ""
                for k in ["Walls", "Settler", "Engineer", "Granary", "Market", "Library", "Monument", "Barracks"]:
                    if k in v:
                        counts[k] += 1
                if "Unit {" in v and "Settler" not in v and "Engineer" not in v and "Scout" not in v:
                    counts["military unit"] += 1
        return counts

    items_rec, items_alt = item_counts("rec"), item_counts("alt")

    # Value per contrast label, pooled over sampled turns.
    cat_value = defaultdict(lambda: defaultdict(list))
    decisions = []
    hl = horizons[-1]
    for (turn, label), dd in pv.items():
        for h in horizons:
            cat_value[label][h].append((turn, mean_ci(dd[h]["wc"]), mean_ci(dd[h]["vp"]), mean_ci(dd[h]["cities"])))
        decisions.append({"turn": turn, "label": label, "cat": label.split(":", 1)[-1],
                          **{f"wc{h}": mean_ci(dd[h]["wc"]) for h in horizons},
                          **{f"vp{h}": mean_ci(dd[h]["vp"]) for h in horizons},
                          **{f"cities{h}": mean_ci(dd[h]["cities"]) for h in horizons},
                          "won_a": dd[hl]["won_a"], "won_b": dd[hl]["won_b"], "n": dd[hl]["n"]})

    def pooled(entries, idx):
        # Mean over turns of per-turn means; the interval adds per-turn variances.
        ms = [e[idx] for e in entries if e[idx] is not None]
        if not ms:
            return None
        m = sum(x[0] for x in ms) / len(ms)
        h = math.sqrt(sum(x[1] ** 2 for x in ms)) / len(ms)
        return m, h, len(ms)

    summary = {}
    for cat, hs in cat_value.items():
        summary[cat] = {h: {"wc": pooled(hs[h], 1), "vp": pooled(hs[h], 2), "cities": pooled(hs[h], 3)} for h in horizons}

    if as_json:
        print(json.dumps({"info": info, "fidelity": fid, "summary": summary, "decisions": decisions,
                          **({"continuation_ranking": sensitivity} if multi else {}),
                          "diff": {"cats": cat_rows, "gold_gap_mean": sum(gold_gap) / max(1, len(gold_gap)),
                                   "buys": [buys_rec, buys_alt], "attacks": [atk_rec, atk_alt],
                                   "founded": [found_rec, found_alt], "items_rec": items_rec, "items_alt": items_alt}}))
        return

    human = (info.get("human") or [{}])[0]
    p = print
    p(f"# Decision audit: {human.get('faction', '?')} (seat {fid.get('seat', 0)}), alternative `{fid.get('alt', 'hard')}`\n")
    saves = ", ".join("{} (engine {}, T{}-T{})".format(Path(s["save"]).name, str(s["engine_commit"])[:9], s["first"], s["last"])
                      for s in info.get("saves", []))
    p(f"Saves: {saves}.  ")
    p(f"Replay: {fid.get('tape_turns')} turns, {fid.get('hash_mismatches')} hash mismatches; AI seat-turns {fid.get('ai_seat_turns')}, "
      f"{fid.get('ai_strategic_order_mismatches')} with strategic orders different from the tape (first {fid.get('first_ai_mismatch')}).  ")
    fids = [r["ai_fidelity"] for r in value if r.get("ai_fidelity")]
    if fids:
        p(f"Branch turn, salt 0: AI seats reproduced the tape's strategic orders in {sum(f[0] for f in fids)}/{sum(f[1] for f in fids)} seat-turns.  ")
    p(f"Value runs: {len(value)}, salts {fid.get('salts')}, horizons {horizons}, window {fid.get('window', 1)} turn(s), dice {'reseeded' if fid.get('reseed') else 'fixed'}.\n")
    if multi:
        p(f"The value and largest-decision tables below use `{names[0]}`; the policy-sensitivity table compares all continuations.\n")

    p("## Where the alternative disagrees (every turn)\n")
    p("| Category | Turns differing | Differences | His orders | Alt orders |\n|---|---:|---:|---:|---:|")
    for c, t, n, ro, ao in cat_rows:
        p(f"| {c} | {t}/{len(diff)} | {n} | {ro} | {ao} |")
    p("")
    p(f"- Gold after the turn, alternative minus his: mean {sum(gold_gap) / max(1, len(gold_gap)):+.0f}. Purchases: his {buys_rec}, alternative {buys_alt}.")
    p(f"- Attacks made that turn: his {atk_rec}, alternative {atk_alt}. Cities founded: his {found_rec}, alternative {found_alt}.")
    p(f"- Where the city's production differs, items chosen (his / alternative): " + ", ".join(
        f"{k} {items_rec.get(k, 0)}/{items_alt.get(k, 0)}" for k in sorted(set(items_rec) | set(items_alt))) + ".\n")

    if summary:
        p("## Value of his decision over the alternative's\n")
        p("Paired by salt, pooled over the sampled turns, 95% intervals; positive means his choice was better. "
          "`whole turn` = his orders against the alternative's. Per category: `fwd` = his orders with only that "
          "category from the alternative, `rev` = the alternative with only that category kept his, `share` = their mean "
          "(mixing two planners costs something by itself, which `fwd` and `rev` charge to opposite sides).\n")
        p("| Contrast | Turns | " + " | ".join(f"win chance pp @{h}" for h in horizons) + f" | VP @{hl} | cities @{hl} |")
        p("|---|---:|" + "---:|" * (len(horizons) + 2))
        def row(label, name):
            s_ = summary.get(label)
            if not s_ or not s_[hl]["wc"]:
                return
            p(f"| {name} | {s_[hl]['wc'][2]} | " + " | ".join(fmt(s_[h]["wc"]) for h in horizons)
              + f" | {fmt(s_[hl]['vp'], 2)} | {fmt(s_[hl]['cities'], 2)} |")
        row("whole turn", "**whole turn**")
        key = lambda c: -((summary.get(f"shap:{c}") or summary.get(f"fwd:{c}") or {hl: {"wc": (0, 0, 0)}})[hl]["wc"] or (0, 0, 0))[0]
        for c in sorted(CATS, key=key):
            row(f"shap:{c}", f"**{c}** share")
            row(f"fwd:{c}", f"{c} fwd")
            row(f"rev:{c}", f"{c} rev")
        p("")
        top = sorted([x for x in decisions if x[f"{rank}{hl}"] and not x["label"].startswith(("fwd", "rev"))],
                     key=lambda x: -x[f"{rank}{hl}"][0])[:15]
        p(f"## Largest single decisions (whole turn or category share, by {rank} @{hl})\n")
        p(f"| Turn | Contrast | win chance pp @{horizons[0]} | @{hl} | VP @{hl} | cities @{hl} | What he did (first turn) | What the alternative did |\n|---:|---|---:|---:|---:|---:|---|---|")
        for x in top:
            r = by_turn.get(x["turn"], {})
            cats = [x["cat"]] if x["cat"] in CATS else [c for c in CATS if r.get("cats", {}).get(c, {}).get("n", 0) > 0]
            what_rec = "; ".join(f"{c}: " + ", ".join(r["cats"][c].get("rec", [])[:3]) for c in cats if r.get("cats", {}).get(c, {}).get("rec"))
            what_alt = "; ".join(f"{c}: " + ", ".join(r["cats"][c].get("alt", [])[:3]) for c in cats if r.get("cats", {}).get(c, {}).get("alt"))
            p(f"| {x['turn']} | {x['label']} | {fmt(x[f'wc{horizons[0]}'])} | {fmt(x[f'wc{hl}'])} | {fmt(x[f'vp{hl}'], 2)} | {fmt(x[f'cities{hl}'], 2)} | {what_rec[:240]} | {what_alt[:240]} |")
        p("")
    p("## Per-turn disagreement (differences by category)\n")
    p("| Turn | Gold | Cities | VP | Mil | " + " | ".join(CATS) + " |\n|---:|---:|---:|---:|---:|" + "---:|" * len(CATS))
    for r in diff:
        if r["turn"] % 5 == 0:
            p(f"| {r['turn']} | {r['gold']} | {r['cities']} | {r['vp']} | {r['military']} | " + " | ".join(str(r["cats"][c]["n"]) for c in CATS) + " |")
    if multi:
        p(f"\n## Policy sensitivity: category shares by {rank} @{hl}\n")
        p("Each continuation uses the same branch arms and salts. Spread is the largest minus smallest "
          "mean share. Robust means every continuation's 95% interval excludes zero on the same side. "
          "Rows rank by the smallest (worst-case) mean share.\n")
        p("Invalid continuation orders across value runs: " + ", ".join(
            f"`{name}` {sum(r.get('continuation_invalid_orders', 0) for r in all_value if r.get('continuation', 'hard') == name)}"
            for name in names) + ".\n")
        p("| Category | " + " | ".join(names) + " | Spread | Worst-case share | Robust |")
        p("|---|" + "---:|" * (len(names) + 3))
        for r in sensitivity:
            p("| " + r["category"] + " | " + " | ".join(fmt(r["shares"][name], 2) for name in names)
              + " | " + (f'{r["spread"]:.2f}' if r["spread"] is not None else "-")
              + " | " + (f'{r["worst_case"]:+.2f}' if r["worst_case"] is not None else "-")
              + " | " + ("yes" if r["robust"] else "no") + " |")


if __name__ == "__main__":
    main()

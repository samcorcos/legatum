#!/usr/bin/env python3
"""Tables and charts for the siege sweep: can a walled city be taken, and by which army?

  siege-lab-report.py CELLS.csv[.gz] [--summary summary.json] [--out DIR]

Writes DIR/results.md (markdown tables) and four portrait PNG charts: options.png,
cost.png, archers.png and engineer.png. Standard library plus Pillow (the charts); no pandas.

Terms.
  cell       one starting situation (army, city, garrison, era, faction, ground), played once
             per dice seed (`runs`, 32 by default). Every policy within a seed faces the same dice.
  walled     the city has Walls.
  mix        melee: all melee. mixed: half melee, half archers. shooters: one melee and
             archers. siege: half melee, half siege engines (Catapult or Bombard). siege1:
             melee plus one siege engine.
  policy     storm: every melee and mounted unit walks in every turn. wait: shooters first,
             melee commits at half HP. ai: Hard's own tactics. "Best forced" is the better of
             storm and wait by capture rate, picked cell by cell (storm on a tie). Surround and
             focus are in the CSV but are not used here.
  capture    share of the dice seeds where the city falls.
  breach     share of the dice seeds where the city's HP reaches 0.

Rule variants, in plain words:
  base               today's rules
  r50                archers do half damage to a walled city's HP
  a50                archers and melee both do half damage to a walled city's HP
  a50_cat2           a50, and siege engines do double damage to city HP
  a50_cat2_spare     a50_cat2, and the city strike never shoots siege engines
  a50_cat2_mantlet   a50_cat2, and siege engines get +5 defence
  a50_eng            a50, and a Siege Engineer travels with the army: units next to it
                     ignore the half-damage penalty
  a50_eng2           a50_eng, and units next to the Engineer also ignore Walls' +10
  a25_eng            both classes do 25% damage without an Engineer; the Engineer lifts it
  r50_eng            only archers are penalised; the Engineer lifts it

Pooling: each cell is first reduced to one record (its best forced policy, or the AI's),
then the cell values are averaged, so every cell counts once whatever its army size or era.
"""
import argparse
import csv
import gzip
import json
from pathlib import Path

STRINGS = {"cell", "era", "faction", "mix", "garrison_type", "ground", "variant"}
METRICS = ("capture", "turns", "att_lost", "def_lost", "loss_cost", "breach", "breach_turns",
           "city_dmg_siege", "city_dmg_other", "siege_lost", "eng_lost", "def_cost")
VARIANTS = (
    ("base", "today's rules"),
    ("r50", "archers do half damage to a walled city's HP"),
    ("a50", "archers and melee both do half damage to a walled city's HP"),
    ("a50_cat2", "a50, and siege engines do double damage to city HP"),
    ("a50_cat2_spare", "a50_cat2, and the city strike never shoots siege engines"),
    ("a50_cat2_mantlet", "a50_cat2, and siege engines get +5 defence"),
    ("a50_eng", "a50, and a Siege Engineer travels with the army (units next to it ignore the penalty)"),
    ("a50_eng2", "a50_eng, and units next to the Engineer also ignore Walls' +10"),
    ("a25_eng", "both classes do 25% damage without an Engineer; the Engineer lifts it"),
    ("r50_eng", "only archers are penalised; the Engineer lifts it"),
)
VARIANT_NAME = dict(VARIANTS)
VARIANT_ORDER = [v for v, _ in VARIANTS]
VARIANT_SHORT = {
    "base": "today's rules",
    "r50": "archers half damage vs walls",
    "a50": "archers and melee half damage",
    "a50_cat2": "a50 + catapults double city damage",
    "a50_cat2_spare": "a50_cat2, city never shoots siege",
    "a50_cat2_mantlet": "a50_cat2, siege +5 defence",
    "a50_eng": "a50 + Siege Engineer",
    "a50_eng2": "a50 + Engineer, also ignores Walls",
    "a25_eng": "25% both, Engineer lifts it",
    "r50_eng": "archers half, Engineer lifts it",
}
MIXES = ("melee", "mixed", "shooters", "siege", "siege1")
MIX_TEXT = {
    "melee": "all melee",
    "mixed": "half melee, half archers",
    "shooters": "one melee, the rest archers",
    "siege": "half melee, half siege engines",
    "siege1": "melee plus one siege engine",
}
ERAS = ("ancient", "classical", "renaissance")
PAIR_ERAS = ("classical", "renaissance")  # tables 1 and 2, as specified
ENGINEER_VARIANTS = ("base", "a50", "a50_eng", "a50_eng2", "a25_eng", "r50_eng")
CATAPULT_VARIANTS = ("base", "a50", "a50_cat2", "a50_cat2_spare", "a50_cat2_mantlet")
GARRISON_VARIANTS = ("base", "a50", "a50_cat2", "a50_eng", "a50_eng2")
GARRISONS = (
    ("none", {"garrison": 0}),
    ("1 melee", {"garrison": 1, "garrison_type": "melee"}),
    ("1 ranged", {"garrison": 1, "garrison_type": "ranged"}),
    ("2 ranged", {"garrison": 2, "garrison_type": "ranged"}),
)
EMPTY = {"cells": 0, **{m: None for m in METRICS + ("army_cost", "dmg_total", "prod_per_100")}}


# ---------------------------------------------------------------- data

def load(path):
    opener = gzip.open if str(path).endswith(".gz") else open
    with opener(path, "rt", newline="") as fh:
        rows = list(csv.DictReader(fh))
    for r in rows:
        for k, v in list(r.items()):
            if k is None or k in STRINGS:
                continue
            if v in (None, ""):
                r[k] = None
            else:
                try:
                    r[k] = float(v)
                except ValueError:
                    pass
    return rows


def sel(rows, **conds):
    return [r for r in rows if all(r.get(k) == v for k, v in conds.items())]


def walled(rows):
    return sel(rows, walls=1)


def variants_in(rows):
    found = {r.get("variant") for r in rows} - {None, ""}
    return [v for v in VARIANT_ORDER if v in found] + sorted(found - set(VARIANT_ORDER))


def eras_in(rows, order=ERAS):
    found = {r.get("era") for r in rows}
    return [e for e in order if e in found]


def avg(xs):
    xs = [x for x in xs if x is not None]
    return sum(xs) / len(xs) if xs else None


def take(r, policy):
    rec = {m: r.get(f"{policy}_{m}") for m in METRICS}
    rec["army_cost"] = r.get("army_cost")
    return rec


def best_forced(r):
    """Storm or wait, whichever captures more in this cell; storm on a tie."""
    forced = [p for p in ("storm", "wait") if r.get(f"{p}_capture") is not None]
    if not forced:
        return None
    return take(r, max(forced, key=lambda p: (r[f"{p}_capture"], p == "storm")))


def summarize(rows, policy="best"):
    """Pool cells: one record per cell (best forced policy, or the AI's), then average."""
    recs = [best_forced(r) if policy == "best" else take(r, "ai") for r in rows]
    recs = [x for x in recs if x is not None]
    if not recs:
        return dict(EMPTY)
    out = {"cells": len(recs)}
    for m in METRICS + ("army_cost",):
        out[m] = avg([rec[m] for rec in recs])
    siege, other = out["city_dmg_siege"], out["city_dmg_other"]
    out["dmg_total"] = None if siege is None and other is None else (siege or 0) + (other or 0)
    cost, dmg = out["army_cost"], out["dmg_total"]
    out["prod_per_100"] = cost / (dmg / 100) if cost and dmg else None
    return out


# ---------------------------------------------------------------- markdown

def pct(x):
    return "-" if x is None else f"{100 * x:.0f}%"


def whole(x):
    return "-" if x is None else f"{x:.0f}"


def tenth(x):
    return "-" if x is None else f"{x:.1f}"


def md(headers, body):
    out = ["| " + " | ".join(headers) + " |", "|" + "|".join("---" for _ in headers) + "|"]
    out += ["| " + " | ".join(str(c) for c in row) + " |" for row in body]
    return "\n".join(out)


def scope(rows, summary, name, L):
    repeats = int(summary.get("repeats", max((r.get("runs") or 0) for r in rows)))
    cells = summary.get("cells", len(rows))
    if "fights" in summary:
        fights = f"{int(summary['fights']):,}"
    else:
        fights = f"{cells * repeats * 5:,} (estimated: cells x repeats x 5 policies)"
    wall = summary.get("wall_seconds")
    variants = summary.get("variants")
    if isinstance(variants, list):
        n_variants = len(variants)
    elif isinstance(variants, int):
        n_variants = variants
    else:
        n_variants = len(variants_in(rows))
    L.append("# Siege lab: can a walled city be taken, and by which army?\n")
    L.append(f"Generated by `tools/sim/siege-lab-report.py` from `{name}`. Terms and variant names "
             "are in that script's docstring; the key is below.\n")
    L.append(md(["", ""], [
        ["Cells (situations)", f"{int(cells):,}"],
        ["Dice seeds per cell", repeats],
        ["Fights", fights],
        ["Wall time", f"{wall:.0f} s" if wall is not None else "not recorded"],
        ["Variants", n_variants],
    ]))
    L.append("\n## Key\n")
    L.append("Capture figures use each cell's better forced policy (storm or wait). Blank or `-` "
             "means no cells for that combination. Tables 1 and 2 cover classical and renaissance "
             "only; the other tables pool every era in the file.\n")
    L.append(md(["Variant", "What it changes"],
                [[v, VARIANT_NAME.get(v, "not in the key")] for v in variants_in(rows)]))
    L.append("")
    L.append(md(["Army", "Units"], [[m, MIX_TEXT[m]] for m in MIXES]))


# ---------------------------------------------------------------- sections

def sec1(rows, L):
    base = walled(sel(rows, variant="base"))
    L.append("\n## 1. Diagnosis: walled cities under today's rules\n")
    L.append("Pooled over factions, army sizes and garrisons.\n")
    stats, dmg, per = [], [], []
    for era in PAIR_ERAS:
        for mix in MIXES:
            s = summarize(sel(base, era=era, mix=mix))
            if not s["cells"]:
                continue
            stats.append([era, mix, s["cells"], whole(s["army_cost"]), pct(s["capture"]), tenth(s["turns"]),
                          whole(s["loss_cost"]), tenth(s["siege_lost"])])
            dmg.append([era, mix, whole(s["city_dmg_siege"]), whole(s["city_dmg_other"]), whole(s["dmg_total"])])
            if mix in ("shooters", "siege", "siege1"):
                per.append([era, mix, whole(s["army_cost"]), whole(s["dmg_total"]), whole(s["prod_per_100"])])
    L.append("### 1a. Cost, capture and losses\n")
    L.append(md(["Era", "Army", "Cells", "Army cost", "Capture", "Turns to capture", "Attacker Production lost",
                 "Siege engines lost"], stats))
    L.append("\n### 1b. City-HP damage per fight (best forced policy)\n")
    L.append(md(["Era", "Army", "Siege dmg", "Other dmg", "Total dmg"], dmg))
    L.append("\n### 1c. Production per 100 city-HP damage\n")
    L.append("Army cost divided by the damage it does to a walled city, per 100 HP. Lower is better.\n")
    L.append(md(["Era", "Army", "Army cost", "Damage per fight", "Production per 100 HP"], per))


def sec2(rows, L):
    base = sel(rows, variant="base")
    L.append("\n## 2. Walls against no walls, today's rules\n")
    body = []
    for era in PAIR_ERAS:
        for mix in MIXES:
            op = summarize(sel(base, era=era, mix=mix, walls=0))
            wa = summarize(sel(base, era=era, mix=mix, walls=1))
            if not op["cells"] and not wa["cells"]:
                continue
            body.append([era, mix, pct(op["capture"]), whole(op["loss_cost"]), pct(wa["capture"]),
                         whole(wa["loss_cost"]), f"{op['cells']} / {wa['cells']}"])
    L.append(md(["Era", "Army", "Open: capture", "Open: Production lost", "Walls: capture",
                 "Walls: Production lost", "Cells (open / walls)"], body))


def grid(rs, L):
    overall, by_mix = [], []
    for v in variants_in(rs):
        vr = sel(rs, variant=v)
        s = summarize(vr)
        overall.append([v, s["cells"], pct(s["capture"]), tenth(s["turns"]), whole(s["loss_cost"]),
                        pct(s["breach"]), whole(s["def_cost"])])
        row = [v]
        for m in MIXES:
            mr = sel(vr, mix=m)
            row.append(pct(summarize(mr)["capture"]) if mr else "")
        by_mix.append(row)
    L.append(md(["Variant", "Cells", "Capture", "Turns to capture", "Attacker Production lost", "Breach",
                 "Defender cost"], overall))
    L.append("\nCapture by army:\n")
    L.append(md(["Variant", "melee", "mixed", "shooters", "siege", "siege1"], by_mix))


def sec3(rows, L):
    walled_rows = walled(rows)
    L.append("\n## 3. The option grid\n")
    L.append("Walled cities, pooled over eras, factions, army sizes and garrisons. Defender cost is the "
             "Production the defender spent (units lost plus units added mid-fight).\n")
    L.append("### 3a. Walled cities, every cell\n")
    grid(walled_rows, L)
    L.append("\n### 3b. Typical cell: 4 units, one ranged garrison\n")
    L.append("Same measures, restricted to 4-unit armies against a single shooter in a walled city.\n")
    grid(sel(walled_rows, n=4, garrison=1, garrison_type="ranged"), L)


def sec4(rows, L):
    shoot = sel(walled(rows), mix="shooters")
    L.append("\n## 4. Can archers alone still breach?\n")
    L.append("Shooter-heavy armies (one melee and archers) against walled cities.\n")
    body = []
    for v in variants_in(shoot):
        for era in eras_in(shoot):
            s = summarize(sel(shoot, variant=v, era=era))
            if not s["cells"]:
                continue
            body.append([v, era, s["cells"], pct(s["breach"]), tenth(s["breach_turns"]), pct(s["capture"]),
                         tenth(s["att_lost"])])
    L.append(md(["Variant", "Era", "Cells", "Breach", "Breach turn", "Capture", "Attackers lost (units)"], body))


def sec5(rows, L):
    walled_rows = walled(rows)
    L.append("\n## 5. The Siege Engineer\n")
    L.append("Walled cities, melee, mixed and shooter armies. The AI column is its own tactics under the same "
             "rules, next to the best forced policy. `Engineer lost` is the rate the Engineer was captured or "
             "destroyed; `-` where the rule set has no Engineer.\n")
    body = []
    for v in ENGINEER_VARIANTS:
        for era in eras_in(walled_rows):
            for mix in ("melee", "mixed", "shooters"):
                rs = sel(walled_rows, variant=v, era=era, mix=mix)
                if not rs:
                    continue
                s, ai = summarize(rs), summarize(rs, "ai")
                body.append([v, f"{era}, {mix}", pct(s["capture"]), pct(ai["capture"]), tenth(s["turns"]),
                             whole(s["loss_cost"]), pct(s["eng_lost"]), whole(s["def_cost"])])
    L.append(md(["Variant", "Era, army", "Capture (best forced)", "AI capture", "Turns to capture",
                 "Attacker Production lost", "Engineer lost", "Defender cost"], body))
    L.append("\nThe AI does not know the new rules, so its column shows how its own tactics fare under each "
             "variant, not how well it would play with the rule known.\n")


def sec6(rows, L):
    walled_rows = walled(rows)
    L.append("\n## 6. Catapult variants\n")
    L.append("Siege and siege1 armies against walled cities, by era.\n")
    body = []
    for v in CATAPULT_VARIANTS:
        for era in eras_in(walled_rows):
            for mix in ("siege", "siege1"):
                rs = sel(walled_rows, variant=v, era=era, mix=mix)
                if not rs:
                    continue
                s = summarize(rs)
                body.append([v, f"{era}, {mix}", pct(s["capture"]), tenth(s["turns"]), whole(s["loss_cost"]),
                             tenth(s["siege_lost"]), whole(s["city_dmg_siege"]), whole(s["city_dmg_other"])])
    L.append(md(["Variant", "Era, army", "Capture", "Turns to capture", "Attacker Production lost",
                 "Siege engines lost", "Siege dmg per fight", "Other dmg per fight"], body))


def sec7(rows, L):
    walled_rows = walled(rows)
    L.append("\n## 7. By garrison\n")
    L.append("Walled cities, pooled over eras, factions and armies. Capture uses each cell's best forced policy.\n")
    body = []
    for v in GARRISON_VARIANTS:
        row = [v]
        for _, conds in GARRISONS:
            rs = sel(walled_rows, variant=v, **conds)
            row.append(pct(summarize(rs)["capture"]) if rs else "-")
        body.append(row)
    L.append(md(["Variant"] + [name for name, _ in GARRISONS], body))


def sec8(rows, L):
    walled_rows = walled(rows)
    L.append("\n## 8. Attacker faction\n")
    L.append("Walled cities, pooled over eras, armies and garrisons.\n")
    body = []
    for v in variants_in(walled_rows):
        gen = summarize(sel(walled_rows, variant=v, faction="generic"))
        spa = summarize(sel(walled_rows, variant=v, faction="sparta"))
        body.append([v, pct(gen["capture"]), pct(spa["capture"]), whole(gen["loss_cost"]), whole(spa["loss_cost"])])
    L.append(md(["Variant", "Generic capture", "Sparta capture", "Generic Production lost",
                 "Sparta Production lost"], body))


# ---------------------------------------------------------------- charts

W = 1080
WHITE, INK, MUTED, GRID = (255, 255, 255), (28, 32, 40), (88, 96, 108), (226, 229, 234)
# Okabe-Ito palette: distinct for colour-blind readers.
MIX_COLOR = {"melee": (0, 114, 178), "mixed": (86, 180, 233), "shooters": (0, 158, 115),
             "siege": (213, 94, 0), "siege1": (230, 159, 0)}
ATTACKER, DEFENDER = (213, 94, 0), (0, 114, 178)
BREACH, CAPTURE = (230, 159, 0), (0, 114, 178)
BAR_H, STEP, BAR_W, BAR_X = 24, 36, 600, 210
CAP_COST = 2000


def font(size, bold=False):
    from PIL import ImageFont

    paths = (
        ["/System/Library/Fonts/Supplemental/Arial Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"]
        if bold
        else ["/System/Library/Fonts/Supplemental/Arial.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]
    )
    for p in paths:
        try:
            return ImageFont.truetype(p, size)
        except OSError:
            pass
    return ImageFont.load_default()


def fonts():
    return {"title": font(36, True), "head": font(28, True), "body": font(28), "small": font(24)}


def canvas(height=6000):
    from PIL import Image, ImageDraw

    img = Image.new("RGB", (W, height), WHITE)
    return img, ImageDraw.Draw(img)


def finish(img, y, path):
    img.crop((0, 0, W, min(img.height, y + 40))).save(path)


def line_h(d, fnt):
    _, top, _, bottom = d.textbbox((0, 0), "Ag", font=fnt)
    return bottom - top


def wrap(d, text, fnt, width):
    lines, line = [], ""
    for word in text.split():
        trial = f"{line} {word}".strip()
        if not line or d.textlength(trial, font=fnt) <= width:
            line = trial
        else:
            lines.append(line)
            line = word
    if line:
        lines.append(line)
    return lines


def para(d, x, y, text, fnt, fill, width):
    step = line_h(d, fnt) + 8
    for ln in wrap(d, text, fnt, width):
        d.text((x, y), ln, fill=fill, font=fnt)
        y += step
    return y


def title_block(d, f, title, sub):
    y = para(d, 40, 36, title, f["title"], INK, W - 80)
    return para(d, 40, y + 6, sub, f["small"], MUTED, W - 80) + 20


def legend(d, f, y, items):
    x = 40
    for label, color in items:
        tw = d.textlength(label, font=f["small"])
        if x > 40 and x + 34 + tw > W - 40:
            x, y = 40, y + 40
        d.rectangle([x, y + 5, x + 22, y + 27], fill=color)
        d.text((x + 32, y), label, fill=INK, font=f["small"])
        x += 32 + tw + 40
    return y + 52


def bar_at(d, f, y, frac, color, text):
    d.rectangle([BAR_X, y, BAR_X + BAR_W, y + BAR_H], fill=GRID)
    if frac is not None and frac > 0:
        d.rectangle([BAR_X, y, BAR_X + max(2, int(BAR_W * min(frac, 1.0))), y + BAR_H], fill=color)
    d.text((BAR_X + BAR_W + 14, y - 4), text, fill=INK, font=f["body"])


def bar_group(d, f, y, title, items):
    """items: (left label, fraction or None, colour, value text). None draws an empty track."""
    y = para(d, 40, y, title, f["head"], INK, W - 80) + 4
    for name, frac, color, text in items:
        d.text((40, y - 4), name, fill=INK, font=f["body"])
        bar_at(d, f, y, frac, color, text)
        y += STEP
    return y + 18


def chart_options(rows, path):
    f = fonts()
    img, d = canvas()
    walled_rows = walled(rows)
    y = title_block(d, f, "Taking a walled city: capture rate by rule variant and army",
                    "Best forced policy (storm or wait), pooled over eras, factions, army sizes and garrisons.")
    y = legend(d, f, y, [(m, MIX_COLOR[m]) for m in MIXES])
    for v in variants_in(walled_rows):
        vr = sel(walled_rows, variant=v)
        items = []
        for m in MIXES:
            mr = sel(vr, mix=m)
            cap = summarize(mr)["capture"] if mr else None
            items.append((m, cap, MIX_COLOR[m], pct(cap) if mr else "not run"))
        y = bar_group(d, f, y, f"{v}   {VARIANT_SHORT.get(v, '')}", items)
    finish(img, y, path)


def chart_cost(rows, path):
    f = fonts()
    img, d = canvas()
    walled_rows = walled(rows)
    y = title_block(d, f, "What a walled city costs each side",
                    "Attacker: Production lost per capture (Production lost divided by capture rate; bars stop "
                    "at 2000 and say so). Defender: Production spent per fight. Walled, best forced policy, pooled.")
    y = legend(d, f, y, [("attacker: Production lost per capture", ATTACKER),
                         ("defender: Production spent", DEFENDER)])
    for v in variants_in(walled_rows):
        s = summarize(sel(walled_rows, variant=v))
        per = None
        if s["capture"] and s["loss_cost"] is not None:
            per = s["loss_cost"] / s["capture"]
        if per is None:
            att_frac, att_text = 1.0, f">{CAP_COST} (no captures)"
        elif per > CAP_COST:
            att_frac, att_text = 1.0, f">{CAP_COST} (capped)"
        else:
            att_frac, att_text = per / CAP_COST, f"{per:.0f}"
        dfc = s["def_cost"]
        def_text = "-" if dfc is None else f"{dfc:.0f}"
        items = [
            ("attacker", att_frac, ATTACKER, att_text),
            ("defender", None if dfc is None else min(dfc, CAP_COST) / CAP_COST, DEFENDER, def_text),
        ]
        y = bar_group(d, f, y, f"{v}   {VARIANT_SHORT.get(v, '')}", items)
    finish(img, y, path)


def chart_archers(rows, path):
    f = fonts()
    img, d = canvas()
    shoot = sel(walled(rows), mix="shooters")
    y = title_block(d, f, "Can archers alone breach walls?",
                    "Shooter-heavy armies (one melee and archers) against walled cities, best forced policy.")
    y = legend(d, f, y, [("breach: city HP reaches 0", BREACH), ("capture: city taken", CAPTURE)])
    for v in variants_in(shoot):
        s = summarize(sel(shoot, variant=v))
        items = [
            ("breach", s["breach"], BREACH, pct(s["breach"])),
            ("capture", s["capture"], CAPTURE, pct(s["capture"])),
        ]
        y = bar_group(d, f, y, f"{v}   {VARIANT_SHORT.get(v, '')}", items)
    finish(img, y, path)


def chart_engineer(rows, path):
    f = fonts()
    img, d = canvas()
    walled_rows = walled(rows)
    three = ("melee", "mixed", "shooters")
    y = title_block(d, f, "The Siege Engineer",
                    "Walled cities, best forced policy. Bars: capture rate by army. Each line below: how often "
                    "the Engineer was captured or destroyed.")
    y = legend(d, f, y, [(m, MIX_COLOR[m]) for m in three])
    for v in [x for x in ENGINEER_VARIANTS if x in variants_in(walled_rows)]:
        vr = sel(walled_rows, variant=v)
        items = []
        for m in three:
            mr = sel(vr, mix=m)
            cap = summarize(mr)["capture"] if mr else None
            items.append((m, cap, MIX_COLOR[m], pct(cap) if mr else "not run"))
        y = bar_group(d, f, y, f"{v}   {VARIANT_SHORT.get(v, '')}", items)
        if v in ("base", "a50"):
            note = "No Engineer in these rules."
        else:
            armies = [r for r in vr if r.get("mix") in three]
            per = [f"{m} {pct(summarize(sel(armies, mix=m))['eng_lost'])}" for m in three if sel(armies, mix=m)]
            note = f"Engineer lost: {pct(summarize(armies)['eng_lost'])} of fights ({', '.join(per)})."
        y = para(d, 40, y - 8, note, f["small"], INK, W - 80) + 30
    finish(img, y, path)


# ---------------------------------------------------------------- main

def main():
    ap = argparse.ArgumentParser(description="Tables and charts for the siege sweep.")
    ap.add_argument("cells")
    ap.add_argument("--summary")
    ap.add_argument("--out", default=".")
    a = ap.parse_args()
    rows = load(a.cells)
    if not rows:
        raise SystemExit(f"no rows in {a.cells}")
    summary = json.loads(Path(a.summary).read_text()) if a.summary else {}
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    L = []
    scope(rows, summary, Path(a.cells).name, L)
    sec1(rows, L)
    sec2(rows, L)
    sec3(rows, L)
    sec4(rows, L)
    sec5(rows, L)
    sec6(rows, L)
    sec7(rows, L)
    sec8(rows, L)
    (out / "results.md").write_text("\n".join(L) + "\n")
    chart_options(rows, out / "options.png")
    chart_cost(rows, out / "cost.png")
    chart_archers(rows, out / "archers.png")
    chart_engineer(rows, out / "engineer.png")
    print(f"wrote {out / 'results.md'} and four charts in {out}")


if __name__ == "__main__":
    main()

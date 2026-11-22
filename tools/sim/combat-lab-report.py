#!/usr/bin/env python3
"""Tables and the portrait chart for the combat lab (crates/sim/examples/combat_lab.rs).

  combat-lab-report.py CELLS.csv[.gz] [--summary summary.json] [--out DIR]

Writes DIR/results.md (the tables) and DIR/forecast-vs-storm.png (a phone-width chart).
Standard library plus Pillow (the chart); no pandas.

Terms. A cell is one starting situation played many times with different dice seeds.
  forecast  what the AI's own `forecast_siege` expects from the start: capture within about
            four rounds of arriving, or none.
  storm     the forced policy: every melee and mounted unit walks into the city every turn.
  AI        Hard's own tactical planner.
  visible   the city is in the attackers' sight (march distance 1 or 2). At distance 3 the
            city is in fog and the forecast assumes one fortified Soldier holds it.
"""
import argparse
import collections
import csv
import gzip
import json
from pathlib import Path

STR = ("cell", "era", "faction", "mix", "garrison_type", "ground")
UNIT_COST = {  # Production cost of one unit by (era, class)
    ("ancient", "melee"): 40, ("ancient", "ranged"): 66, ("ancient", "mounted"): 55,
    ("classical", "melee"): 55, ("classical", "ranged"): 84, ("classical", "mounted"): 75,
    ("renaissance", "melee"): 105, ("renaissance", "ranged"): 136, ("renaissance", "mounted"): 139,
}
PEOPLE = {"generic": "generic", "sparta": "Sparta", "mongols": "Mongols"}


def load(path):
    opener = gzip.open if str(path).endswith(".gz") else open
    with opener(path, "rt") as fh:
        rows = list(csv.DictReader(fh))
    for r in rows:
        for k in list(r):
            if k not in STR:
                r[k] = float(r[k]) if r[k] != "" else None
    return rows


def mean(rs, k):
    xs = [r[k] for r in rs if r[k] is not None]
    return sum(xs) / len(xs) if xs else float("nan")


def pct(x):
    return "-" if x != x else f"{100 * x:.0f}%"


def army_cost(r):
    e, n = r["era"], int(r["n"])
    m, s, mo = (UNIT_COST[(e, c)] for c in ("melee", "ranged", "mounted"))
    mix = r["mix"]
    if mix == "melee":
        return n * m
    if mix == "mounted":
        return n * mo
    if mix == "mixed":
        melee = (n + 1) // 2
        return melee * m + (n - melee) * s
    return m + (n - 1) * s


def md(headers, body):
    out = ["| " + " | ".join(headers) + " |", "|" + "|".join("---" for _ in headers) + "|"]
    out += ["| " + " | ".join(str(c) for c in row) + " |" for row in body]
    return "\n".join(out)


def best(r):
    return max(r[k] for k in ("ai_capture", "storm_capture", "wait_capture", "surround_capture", "focus_capture"))


POLICIES = {
    "AI's own": "ai_capture", "Storm now": "storm_capture",
    "Wait for shooters": "wait_capture", "Surround first": "surround_capture",
    "Focus fire": "focus_capture",
}
POLICY_LOSSES = {
    "AI's own": "ai_loss_cost", "Storm now": "storm_loss_cost",
    "Wait for shooters": "wait_loss_cost", "Surround first": "surround_loss_cost",
    "Focus fire": "focus_loss_cost",
}


def policy_table(rows):
    keys = sorted({(r["mix"], int(r["walls"]), r["garrison_type"], "in sight" if r["fc_visible"] else "fog") for r in rows})
    body = []
    for mix, walls, garrison, sight in keys:
        rs = [r for r in rows if (r["mix"], int(r["walls"]), r["garrison_type"], "in sight" if r["fc_visible"] else "fog") == (mix, walls, garrison, sight)]
        rates = {name: mean(rs, field) for name, field in POLICIES.items()}
        winner = max(rates, key=rates.get)
        body.append([mix, "Walls" if walls else "Open", garrison, sight, len(rs), winner,
                     pct(rates[winner]), f"{mean(rs, POLICY_LOSSES[winner]):.0f}"])
    return md(["Army", "City", "Garrison", "Vision", "Cells", "Best policy", "Capture rate", "Losses (Production)"], body)


def confusion(rs):
    """Forecast against the best of five policies; winnable means any captures in half the runs."""
    win = [r for r in rs if best(r) >= 0.5]
    yes = [r for r in rs if r["fc_capture"] == 1]
    miss = sum(1 for r in win if r["fc_capture"] == 0)
    alarm = sum(1 for r in yes if best(r) < 0.5)
    return win, yes, miss, alarm


SLICES = [
    ("Army: melee only", lambda r: r["mix"] == "melee"),
    ("Army: melee and shooters", lambda r: r["mix"] == "mixed"),
    ("Army: shooter-heavy", lambda r: r["mix"] == "shooters"),
    ("Army: mounted", lambda r: r["mix"] == "mounted"),
    ("City: no Walls", lambda r: r["walls"] == 0),
    ("City: Walls", lambda r: r["walls"] == 1),
    ("Garrison: none", lambda r: r["garrison"] == 0),
    ("Garrison: 1 unit", lambda r: r["garrison"] == 1),
    ("Garrison: 2 units", lambda r: r["garrison"] == 2),
    ("Garrison: 3 units", lambda r: r["garrison"] == 3),
    ("Garrison: melee", lambda r: r["garrison_type"] == "melee"),
    ("Garrison: shooters", lambda r: r["garrison_type"] == "ranged"),
    ("City at 200 HP", lambda r: r["city_hp"] == 200),
    ("City at 100 HP", lambda r: r["city_hp"] == 100),
    ("Attacker: generic", lambda r: r["faction"] == "generic"),
    ("Attacker: Sparta", lambda r: r["faction"] == "sparta"),
    ("Attacker: Mongols", lambda r: r["faction"] == "mongols"),
    ("Era: ancient", lambda r: r["era"] == "ancient"),
    ("Era: classical", lambda r: r["era"] == "classical"),
    ("Era: renaissance", lambda r: r["era"] == "renaissance"),
    ("Ground: flat", lambda r: r["ground"] == "flat"),
    ("Ground: city on hills", lambda r: r["ground"] == "hills"),
    ("Ground: river with a ford", lambda r: r["ground"] == "river"),
    ("Start: adjacent (1)", lambda r: r["dist"] == 1),
    ("Start: 2 hexes", lambda r: r["dist"] == 2),
]


def slice_table(vis):
    body = []
    for name, sel in SLICES:
        rs = [r for r in vis if sel(r)]
        win, yes, miss, alarm = confusion(rs)
        body.append([
            name, len(rs), pct(mean(rs, "fc_capture")), pct(mean(rs, "storm_capture")), pct(mean(rs, "ai_capture")),
            f"{pct(miss / len(win)) if win else '-'} ({miss})", f"{pct(alarm / len(yes)) if yes else '-'} ({alarm})",
        ])
    return md(
        ["Slice", "Cells", "Forecast says capture", "Storm captures", "AI captures",
         "Missed wins: forecast says no, a policy wins", "False alarms: forecast says yes, neither wins"],
        body,
    )


def bucket(r):
    return "none" if r["fc_capture"] == 0 else int(r["fc_turns"])


def buckets():
    return [("none", "no capture"), (1, "turn 1"), (2, "turn 2"), (3, "turn 3"), (4, "turn 4"), (5, "turn 5+")]


def by_bucket(vis):
    out = collections.defaultdict(list)
    for r in vis:
        b = bucket(r)
        out["none" if b == "none" else min(b, 5)].append(r)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cells")
    ap.add_argument("--summary")
    ap.add_argument("--out", default=".")
    a = ap.parse_args()
    rows = load(a.cells)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    vis = [r for r in rows if r["fc_visible"] == 1]
    fog = [r for r in rows if r["fc_visible"] == 0]
    summary = json.loads(Path(a.summary).read_text()) if a.summary else {}
    L = []
    w = L.append
    runs = int(rows[0]["runs"])
    runner = summary.get("runner", f"{summary.get('threads', '?')} threads (cloud runner)")
    w("# Combat lab, stage 2: siege policies with a live defender\n")
    w("Generated by `tools/sim/combat-lab-report.py` from `cells.csv.gz`. Terms are in that script's header "
      "and in the README.\n")
    w("## Scope and speed\n")
    w(md(["", ""], [
        ["Cells (situations)", f"{len(rows):,} ({len(vis):,} with the city in sight, {len(fog):,} in fog)"],
        ["Dice seeds per cell and policy", runs],
        ["Fights", f"{summary.get('fights', len(rows) * runs * 5):,} (five policies per seed, same dice)"],
        ["Wall time", f"{summary.get('wall_seconds', float('nan')):.0f} s on {runner}"],
        ["Per thousand fights", f"{summary.get('wall_seconds_per_thousand_fights', float('nan')):.3f} s of wall time, "
                                f"{summary.get('cpu_ms_per_fight', float('nan')):.1f} ms of CPU per fight"],
    ]))
    w("\n## 1. Forecast error against the best policy (cities in sight)\n")
    win, yes, miss, alarm = confusion(vis)
    w(f"Of {len(vis):,} situations with the city in sight, the best of five policies captures in at "
      f"least half the runs in {len(win):,}. The forecast calls **{miss:,} of those ({pct(miss / len(win)) if win else 'n/a'}) "
      f"hopeless**. It says \"captures\" in {len(yes):,} situations, and in {alarm:,} of them "
      f"({pct(alarm / len(yes)) if yes else 'n/a'}) none of the five policies succeeds.\n")
    body = []
    bk = by_bucket(vis)
    for key, label in buckets():
        rs = bk.get(key, [])
        if not rs:
            continue
        body.append([label, len(rs), pct(mean(rs, "storm_capture")), pct(mean(rs, "ai_capture")), pct(mean(rs, "wait_capture")),
                     pct(mean(rs, "surround_capture")), pct(mean(rs, "focus_capture")),
                     pct(mean(rs, "commit_rate")), f"{mean(rs, 'storm_turns'):.1f}", f"{mean(rs, 'storm_loss_cost'):.0f}",
                     "-" if key == "none" else f"{mean(rs, 'fc_losses'):.0f}"])
    w(md(["Forecast says", "Cells", "Storm", "AI", "Wait", "Surround", "Focus", "AI commits on turn 1",
          "Storm turns (when it wins)", "Storm losses (Production)", "Forecast losses (Production)"], body))
    w("\nThe forecast looks about four rounds ahead of the army's arrival, and the AI commits when it sees a "
      "capture in three turns or fewer. Past that it mostly waits, though a storm still wins most of the "
      "\"turn 4\" and \"turn 5+\" situations.\n")
    w("## 2. Where the forecast is wrong\n")
    w(slice_table(vis))
    w("\n## 3. How good are its numbers when it does say \"captures\"?\n")
    tp = [r for r in vis if r["fc_capture"] == 1 and r["storm_capture"] >= 0.5]
    body = []
    for fac in ("generic", "sparta", "mongols"):
        rs = [r for r in tp if r["faction"] == fac]
        body.append([PEOPLE[fac], len(rs), f"{mean(rs, 'fc_turns'):.2f}", f"{mean(rs, 'storm_turns'):.2f}",
                     f"{mean(rs, 'fc_losses'):.0f}", f"{mean(rs, 'storm_loss_cost'):.0f}"])
    rs = tp
    body.append(["all", len(rs), f"{mean(rs, 'fc_turns'):.2f}", f"{mean(rs, 'storm_turns'):.2f}",
                 f"{mean(rs, 'fc_losses'):.0f}", f"{mean(rs, 'storm_loss_cost'):.0f}"])
    w(md(["Attacker", "Cells", "Forecast capture turn", "Storm capture turn", "Forecast losses (Production)",
          "Storm losses (Production)"], body))
    w("\nLosses count dead units whole and wounds at half their cost, the forecast's own rule. In these "
      "forecast-positive cells, the forecast is about half a turn earlier than the storm and understates "
      "its Production losses for all three factions. The live defender makes the actual price much higher.\n")
    w("## 4. Fog: the city is three hexes out and unseen\n")
    body = []
    for name, sel in [("All", lambda r: True), ("Garrison 1 melee, no Walls",
                      lambda r: r["garrison"] == 1 and r["garrison_type"] == "melee" and r["walls"] == 0),
                      ("Garrison 2 or 3", lambda r: r["garrison"] >= 2), ("Walls", lambda r: r["walls"] == 1)]:
        rs = [r for r in fog if sel(r)]
        body.append([name, len(rs), pct(mean(rs, "fc_capture")), pct(mean(rs, "storm_capture")),
                     pct(mean(rs, "ai_capture")), pct(mean(rs, "ai_engaged"))])
    w(md(["Fogged cities", "Cells", "Forecast says capture", "Storm captures", "AI captures", "AI ever attacks"], body))
    w("\nWith the city unseen the forecast assumes one fortified Soldier, whatever is really there, so it "
      "predicts the same answer for a city with no garrison and a city with three archers. Across these fog "
      f"cells, the AI attacked in {pct(mean(fog, 'ai_engaged'))} of runs and captured in {pct(mean(fog, 'ai_capture'))}.\n")
    w("## 5. A policy comparison: AI\u2019s own vs storm now, cities in sight\n")
    sk = [r for r in vis if r["storm_capture"] >= 0.8 and r["ai_capture"] <= 0.2]
    sure = [r for r in vis if r["storm_capture"] >= 0.8]
    better = [r for r in vis if r["storm_capture"] - r["ai_capture"] >= 0.25]
    worse = [r for r in vis if r["ai_capture"] - r["storm_capture"] >= 0.25]
    w(f"A storm captures in at least 80% of the runs in {len(sure):,} situations; the AI, in the same situation, "
      f"captures in at most 20% of them in **{len(sk):,} ({pct(len(sk) / len(sure)) if sure else 'n/a'})**. Storm beats the AI by "
      f"25 points or more in {len(better):,} situations; the AI beats storm by that much in {len(worse):,} "
      f"({collections.Counter(r['mix'] for r in worse).most_common(1)[0][0] if worse else 'no'} armies, mostly). "
      "Wait, surround and focus are compared in the best-policy table below.\n")
    body = []
    for walls in (0, 1):
        for gt in ("none", "melee", "ranged"):
            for label, ns in (("2 to 3 units", (2, 3)), ("4 to 6 units", (4, 6))):
                rs = [r for r in vis if r["mix"] in ("melee", "mounted") and r["walls"] == walls
                      and r["garrison_type"] == gt and r["n"] in ns and r["dist"] == 2
                      and r["ground"] == "flat" and r["city_hp"] == 200]
                if not rs:
                    continue
                body.append([("Walls" if walls else "No Walls"), {"none": "empty", "melee": "melee garrison 1-3",
                             "ranged": "shooter garrison 1-3"}[gt], label, len(rs),
                             pct(mean(rs, "storm_capture")), f"{mean(rs, 'storm_loss_cost'):.0f}",
                             pct(mean(rs, "ai_capture")), f"{mean(rs, 'ai_loss_cost'):.0f}"])
    w("Melee and mounted armies, start 2 hexes out on flat ground, city at 200 HP (all three factions and eras "
      "pooled; losses in Production):\n")
    w(md(["City", "Garrison", "Army", "Cells", "Storm captures", "Storm losses", "AI captures", "AI losses"], body))
    body = []
    for mixname, mixsel in [("Melee only", "melee"), ("Mounted", "mounted"), ("Melee and shooters", "mixed"),
                            ("Shooter-heavy", "shooters")]:
        rs = [r for r in vis if r["mix"] == mixsel]
        body.append([mixname, len(rs), pct(mean(rs, "storm_capture")), f"{mean(rs, 'storm_loss_cost'):.0f}",
                     pct(mean(rs, "ai_capture")), f"{mean(rs, 'ai_loss_cost'):.0f}"])
    w("\nBy army type (everything in sight, pooled):\n")
    w(md(["Army", "Cells", "Storm captures", "Storm losses", "AI captures", "AI losses"], body))
    w("\n## 6. Best policy by situation class\n")
    w("Capture rates pool eras, factions, city HP, march distance and terrain within each class. Every policy faces the same live defender and paired dice seeds. City-in-sight and fogged cells are separate classes.\n")
    w(policy_table(rows))
    w("\n## 7. The known Sparta case: 2 to 6 Hoplites against an unwalled city\n")
    w("Ancient era, flat ground, city at 200 HP, no Walls. Cells are army size x garrison; `m` is melee "
      "defenders, `r` shooters; start 1 or 2 hexes out. Forecast = the AI's own prediction (capture turn / losses in "
      "Production); `-` means no capture in its window. Cap = capture rate over the dice seeds; turns = mean "
      "capture turn; lost = mean attackers killed.\n")
    body = []
    for r in rows:
        if (r["era"] == "ancient" and r["faction"] == "sparta" and r["mix"] == "melee" and r["walls"] == 0
                and r["ground"] == "flat" and r["city_hp"] == 200 and r["dist"] in (1, 2)):
            fc = f"{int(r['fc_turns'])} / {int(r['fc_losses'])}" if r["fc_capture"] == 1 else "-"
            ai_t = "-" if r["ai_turns"] is None else f"{r['ai_turns']:.1f}"
            st_t = "-" if r["storm_turns"] is None else f"{r['storm_turns']:.1f}"
            body.append([int(r["n"]), (f"{int(r['garrison'])}{r['garrison_type'][0]}" if r["garrison"] else "0"),
                         int(r["dist"]), fc, pct(r["commit_rate"]), pct(r["ai_capture"]), ai_t,
                         f"{r['ai_att_lost']:.1f}", pct(r["storm_capture"]), st_t, f"{r['storm_att_lost']:.1f}"])
    w(md(["Hoplites", "Garrison", "Start", "Forecast (turn / losses)", "AI commits turn 1", "AI cap", "AI turns",
          "AI lost", "Storm cap", "Storm turns", "Storm lost"], body))
    w("\n## 8. Biggest forecast errors\n")
    w("Misses: cities in sight that a storm takes in at least 90% of the runs while the forecast says no, "
      "toughest city first, then smallest army. Walls count double.\n")
    misses = [r for r in vis if r["fc_capture"] == 0 and r["storm_capture"] >= 0.9]
    misses.sort(key=lambda r: (-(r["garrison"] + 2 * r["walls"]), r["n"], -r["storm_capture"], r["cell"]))
    seen, body = set(), []
    for r in misses:
        key = (r["faction"], r["mix"], r["era"])
        if key in seen:
            continue
        seen.add(key)
        body.append([r["cell"], pct(r["storm_capture"]), f"{r['storm_turns']:.1f}", f"{r['storm_att_lost']:.1f}",
                     pct(r["ai_capture"])])
        if len(body) == 12:
            break
    w(md(["Cell", "Storm captures", "Turns", "Attackers lost", "AI captures"], body))
    w("\nFalse alarms: the forecast says capture and neither policy captures in more than 20% of the runs, "
      "largest army first.\n")
    alarms = [r for r in vis if r["fc_capture"] == 1 and best(r) <= 0.2]
    alarms.sort(key=lambda r: (-r["n"], r["storm_capture"], r["cell"]))
    seen, body = set(), []
    for r in alarms:
        key = (r["faction"], r["mix"], r["ground"], r["walls"])
        if key in seen:
            continue
        seen.add(key)
        body.append([r["cell"], f"{int(r['fc_turns'])} / {int(r['fc_losses'])}", pct(r["storm_capture"]),
                     f"{r['storm_att_lost']:.1f}", pct(r["ai_capture"])])
        if len(body) == 12:
            break
    w(md(["Cell", "Forecast (turn / losses)", "Storm captures", "Storm attackers lost", "AI captures"], body))
    w("\n## 9. Rules the data points at\n")
    idx = {}
    for r in rows:
        k = (r["era"], r["mix"], r["n"], r["walls"], r["garrison"], r["garrison_type"], r["city_hp"], r["ground"], r["dist"])
        idx.setdefault(k, {})[r["faction"]] = r
    body = []
    for fac in ("sparta", "mongols"):
        pairs = [v for v in idx.values() if fac in v and "generic" in v and v["generic"]["fc_visible"] == 1]
        if pairs:
            body.append([f"{PEOPLE[fac]} minus generic",
                         f"{100 * sum(v[fac]['storm_capture'] - v['generic']['storm_capture'] for v in pairs) / len(pairs):+.0f} points",
                         f"{sum(v[fac]['storm_loss_cost'] - v['generic']['storm_loss_cost'] for v in pairs) / len(pairs):+.0f}",
                         f"{100 * sum(v[fac]['fc_capture'] - v['generic']['fc_capture'] for v in pairs) / len(pairs):+.0f} points"])
        else:
            body.append([f"{PEOPLE[fac]} minus generic", "n/a", "n/a", "n/a"])
    w(md(["Paired by situation", "Storm capture rate", "Storm losses (Production)", "Forecast says capture"], body))
    unw = [r for r in vis if r["mix"] in ("melee", "mounted") and r["walls"] == 0 and r["garrison"] <= 1]
    wal = [r for r in vis if r["mix"] in ("melee", "mounted") and r["walls"] == 1 and r["garrison"] <= 1]
    w(f"\nMelee and mounted armies of any size against an unwalled city with at most one defender: a storm "
      f"captures {pct(mean(unw, 'storm_capture'))} of the time and loses {mean(unw, 'storm_loss_cost'):.0f} "
      f"Production on average. Against the same garrison behind Walls: {pct(mean(wal, 'storm_capture'))} "
      f"and {mean(wal, 'storm_loss_cost'):.0f}. Walls roughly double the price of the rush; they do not stop "
      f"a large army.\n")
    (out / "results.md").write_text("\n".join(L) + "\n")
    chart(vis, out / "forecast-vs-storm.png")
    policy_chart(rows, out / "policies.png")
    print(f"wrote {out / 'results.md'}, {out / 'forecast-vs-storm.png'} and {out / 'policies.png'}")


# ---------------------------------------------------------------- chart

W_PX = 1080
INK, MUTED, GRID = (28, 32, 40), (96, 104, 116), (226, 229, 234)
STORM, AI, FORE = (222, 110, 36), (70, 112, 176), (60, 60, 64)
MISS, ALARM = (196, 52, 52), (126, 84, 170)


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


def chart(vis, path):
    from PIL import Image, ImageDraw

    h_title, h_panel = 190, 0
    # panel 1: forecast buckets
    bk = by_bucket(vis)
    p1 = [(label, bk[key]) for key, label in buckets() if bk.get(key)]
    # panel 2: error slices
    names = ["Army: melee only", "Army: mounted", "Army: melee and shooters", "Army: shooter-heavy", "City: no Walls",
             "City: Walls", "Garrison: 1 unit", "Garrison: 3 units", "Attacker: Sparta", "Attacker: Mongols",
             "Ground: river with a ford", "Ground: city on hills"]
    sel = dict(SLICES)
    p2 = [(n, [r for r in vis if sel[n](r)]) for n in names]
    # panel 3: policies
    groups = []
    for label, f in [
        ("Melee or mounted, no Walls, 1-3 melee garrison", lambda r: r["mix"] in ("melee", "mounted") and r["walls"] == 0 and r["garrison_type"] == "melee"),
        ("Melee or mounted, no Walls, 1-3 shooters", lambda r: r["mix"] in ("melee", "mounted") and r["walls"] == 0 and r["garrison_type"] == "ranged"),
        ("Melee or mounted, Walls, 1-3 melee garrison", lambda r: r["mix"] in ("melee", "mounted") and r["walls"] == 1 and r["garrison_type"] == "melee"),
        ("Melee or mounted, Walls, 1-3 shooters", lambda r: r["mix"] in ("melee", "mounted") and r["walls"] == 1 and r["garrison_type"] == "ranged"),
        ("Melee and shooters", lambda r: r["mix"] == "mixed"),
        ("Shooter-heavy", lambda r: r["mix"] == "shooters"),
        ("Sparta, melee only, no Walls", lambda r: r["faction"] == "sparta" and r["mix"] == "melee" and r["walls"] == 0 and r["garrison"] >= 1),
        ("Mongols, any army, Walls", lambda r: r["faction"] == "mongols" and r["walls"] == 1),
    ]:
        groups.append((label, [r for r in vis if f(r)]))
    height = 4000  # cropped to the content below
    img = Image.new("RGB", (W_PX, height), (255, 255, 255))
    d = ImageDraw.Draw(img)
    f_title, f_head, f_body, f_small = font(46, True), font(34, True), font(28), font(24)
    d.text((40, 36), "Combat lab: is the siege forecast right?", fill=INK, font=f_title)
    d.text((40, 100), f"{len(vis):,} city-capture situations with the city in sight,", fill=MUTED, font=f_body)
    d.text((40, 136), "each played 32 times with different dice", fill=MUTED, font=f_body)
    y = h_title

    def head(title, sub):
        nonlocal y
        d.text((40, y), title, fill=INK, font=f_head)
        d.text((40, y + 44), sub, fill=MUTED, font=f_small)
        y += 90

    def bar(x0, yy, width, value, color, label=None):
        d.rectangle([x0, yy, x0 + width, yy + 22], fill=GRID)
        if value > 0:
            d.rectangle([x0, yy, x0 + max(2, int(width * value)), yy + 22], fill=color)
        if label:
            d.text((x0 + width + 12, yy - 3), label, fill=INK, font=f_small)

    # panel 1
    head("1. When the forecast says \"captures in N turns\"",
         "what actually happened: forced storm (orange) and the AI's own play (blue)")
    for label, rs in p1:
        d.text((40, y + 8), f"{label}", fill=INK, font=f_body)
        d.text((40, y + 40), f"{len(rs):,} situations", fill=MUTED, font=f_small)
        s, a = mean(rs, "storm_capture"), mean(rs, "ai_capture")
        bar(330, y + 6, 560, s, STORM, pct(s))
        bar(330, y + 34, 560, a, AI, pct(a))
        y += 74
    d.text((40, y), "A perfect forecast would show 0% on the first row and 100% on every row below it.", fill=MUTED, font=f_small)
    y += 70
    # panel 2
    head("2. Where it is wrong",
         "red: forecast said no, a policy wins.  purple: forecast said yes, neither wins")
    for name, rs in p2:
        win, yes, miss, alarm = confusion(rs)
        d.text((40, y), name, fill=INK, font=f_body)
        m_, a_ = (miss / len(win) if win else 0), (alarm / len(yes) if yes else 0)
        bar(430, y + 4, 430, m_, MISS, pct(m_))
        bar(430, y + 30, 430, a_, ALARM, pct(a_))
        y += 66
    d.text((40, y), "Red: share of winnable situations the forecast calls hopeless. Purple: share of its wins neither policy gets.", fill=MUTED, font=f_small)
    y += 70
    # panel 3
    head("3. Storm now against the AI's own choice", "capture rate: storm (orange), AI (blue)")
    for label, rs in groups:
        d.text((40, y), label, fill=INK, font=f_body)
        s, a = mean(rs, "storm_capture"), mean(rs, "ai_capture")
        bar(40, y + 40, 600, s, STORM, f"{pct(s)}  loses {mean(rs, 'storm_loss_cost'):.0f}")
        bar(40, y + 64, 600, a, AI, f"{pct(a)}  loses {mean(rs, 'ai_loss_cost'):.0f}")
        y += 104
    d.text((40, y - 6), "Losses are Production per assault (a Spearman costs 40).", fill=MUTED, font=f_small)
    img = img.crop((0, 0, W_PX, y + 40))
    img.save(path)


def policy_chart(rows, path):
    from PIL import Image, ImageDraw

    colors = {
        "AI's own": AI, "Storm now": STORM, "Wait for shooters": (54, 153, 105),
        "Surround first": (126, 84, 170), "Focus fire": (207, 163, 42),
    }
    groups = []
    for mix in ("melee", "mixed", "shooters", "mounted"):
        for walls in (0, 1):
            rs = [r for r in rows if r["mix"] == mix and int(r["walls"]) == walls]
            if rs:
                groups.append((f"{mix.title()} | {'Walls' if walls else 'Open city'}", rs))
    width, row_h = 1080, 184
    height = 230 + len(groups) * row_h
    img = Image.new("RGB", (width, height), (255, 255, 255))
    d = ImageDraw.Draw(img)
    f_title, f_head, f_body, f_small = font(44, True), font(28, True), font(23), font(19)
    d.text((36, 24), "Combat lab: which siege policy wins?", fill=INK, font=f_title)
    d.text((36, 82), "Capture rate by attacker army and city fortification", fill=MUTED, font=f_body)
    x, y = 40, 132
    for name, color in colors.items():
        d.rectangle((x, y, x + 18, y + 18), fill=color)
        d.text((x + 25, y - 3), name, fill=INK, font=f_small)
        x += 200
    y = 185
    for label, rs in groups:
        rates = {name: mean(rs, field) for name, field in POLICIES.items()}
        winner = max(rates, key=rates.get)
        d.text((40, y), f"{label}  •  best: {winner} ({pct(rates[winner])})", fill=INK, font=f_head)
        d.text((40, y + 39), f"{len(rs):,} situations; live city production and defender turns", fill=MUTED, font=f_small)
        bx, by, bw = 40, y + 66, 690
        for name, rate in rates.items():
            d.text((bx, by), name, fill=INK, font=f_small)
            d.rectangle((bx + 170, by + 1, bx + 170 + bw, by + 19), fill=GRID)
            d.rectangle((bx + 170, by + 1, bx + 170 + max(1, int(bw * rate)), by + 19), fill=colors[name])
            d.text((bx + 870, by - 3), pct(rate), fill=INK, font=f_small)
            by += 21
        y += row_h
    img.crop((0, 0, width, min(height, y + 8))).save(path)


if __name__ == "__main__":
    main()

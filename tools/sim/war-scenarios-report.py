#!/usr/bin/env python3
"""Report, ledger and chart for the war-scenario suite (crates/sim/examples/war_suite.rs).

  war-scenarios-report.py REPORT.json [--ledger L.jsonl] [--table T.md] [--chart C.png] [--label X]
  war-scenarios-report.py --compare BASE.json CANDIDATE.json

The suite score is the mean goal-success rate over the scenarios; "human parity" counts the
scenarios where the AI met the goal in at least half the runs and no slower than the user.
--compare pairs two reports run by run (same scenarios and run indices, so the same seeds) and
prints the candidate-minus-base success delta per scenario with a bootstrap interval.
"""
import argparse
import json
import random
from pathlib import Path
import subprocess
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from freshness import current_stamp, UNKNOWN


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


PEOPLE = {
    "Switzerland": "Switzerland", "Mongols": "Mongols", "Sparta": "Sparta", "Germany": "Germany",
    "Japan": "Japan", "Byzantium": "Byzantium", "Celts": "Celts", "Athens": "Athens",
}


def load(path):
    return json.loads(Path(path).read_text())


def human_text(h):
    if not h:
        return "n/a"
    return f"{'met' if h['met'] else 'not met'} in {h['turns']} turns, {h['units_lost']} units lost"


def enemy_lost(s):
    """Mean combat units lost by the side at war with the seat, and the ratio to the seat's own losses."""
    if "enemy_units_lost_x100" not in s:
        return "-"
    enemy, own = s["enemy_units_lost_x100"], s["units_lost_x100"]
    ratio = f"{enemy / own:.1f}:1" if own else "n/a"
    return f"{enemy / 100:.2f} ({ratio})"


def table(report):
    lines = [
        "| Scenario | Faction | Goal met | Median turns (human) | Units lost (human) | Enemy units lost | Cities lost | Never declared | Bugs |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for s in report["scenarios"]:
        h = s.get("human") or {}
        med = "-" if s["median_turns"] is None else str(s["median_turns"])
        lines.append(
            f"| {s['title']} (`{s['id']}`) | {PEOPLE.get(s['faction'], s['faction'])} | {s['success_percent']}% | {med} ({h.get('turns', '-')}) "
            f"| {s['units_lost_x100'] / 100:.2f} ({h.get('units_lost', '-')}) | {enemy_lost(s)} | {s['cities_lost_x100'] / 100:.2f} "
            f"| {s['never_declared']}/{s['runs']} | {len(s['bugs'])} |"
        )
    lines.append("")
    lines.append(
        f"Suite: {report['suite_success_percent']}% mean goal success, {report['human_parity']}/{len(report['scenarios'])} "
        f"scenarios at human parity ({report['brain']}, {report['runs']} runs each, commit {git_commit()})."
    )
    return "\n".join(lines)


def git_commit():
    import subprocess
    try:
        return subprocess.run(["git", "rev-parse", "--short=8", "HEAD"], capture_output=True, text=True).stdout.strip()
    except OSError:
        return None


def ledger_row(report, label):
    try:
        provenance = current_stamp(report["engine_commit"])
    except (ValueError, subprocess.CalledProcessError):
        provenance = dict(UNKNOWN, engine_commit=report["engine_commit"])
    return {
        "label": label,
        "git_commit": git_commit(),
        **provenance,
        "brain": report["brain"],
        "candidate_params": report.get("candidate_params"),
        "params_seats": report.get("params_seats"),
        "runs": report["runs"],
        "suite_success_percent": report["suite_success_percent"],
        "human_parity": report["human_parity"],
        "scenarios": {
            s["id"]: {
                "success_percent": s["success_percent"],
                "median_turns": s["median_turns"],
                "units_lost_x100": s["units_lost_x100"],
                "enemy_units_lost_x100": s.get("enemy_units_lost_x100"),
                "cities_lost_x100": s["cities_lost_x100"],
                "bugs": len(s["bugs"]),
            }
            for s in report["scenarios"]
        },
    }


def chart(path, reports):
    """Portrait (1080 px wide) bars: goal success per scenario for each report, human marker."""
    from PIL import Image, ImageDraw

    base = reports[0]
    scen = base["scenarios"]
    colors = [(37, 99, 235), (234, 88, 12), (22, 163, 74)]
    W, row, top = 1080, 150, 250
    H = top + row * len(scen) + 110 + 42 * len(reports)
    img = Image.new("RGB", (W, H), (250, 249, 246))
    d = ImageDraw.Draw(img)
    d.text((60, 50), "War scenarios: AI vs the user", font=font(52, True), fill=(20, 20, 20))
    d.text((60, 120), "Share of runs that met the goal the user met in his own game", font=font(30), fill=(70, 70, 70))
    lx = 60
    for i, r in enumerate(reports):
        d.rectangle([lx, 180, lx + 30, 210], fill=colors[i % 3])
        name = r["brain"] + (f" + {r['candidate_params']}" if r.get("candidate_params") else "")
        d.text((lx + 42, 178), name, font=font(28), fill=(40, 40, 40))
        lx += 60 + d.textlength(name, font=font(28))
    x0, x1 = 60, W - 60
    for j, s in enumerate(scen):
        y = top + j * row
        h = s.get("human") or {}
        d.text((x0, y), s["title"], font=font(32, True), fill=(20, 20, 20))
        sub = (f"{PEOPLE.get(s['faction'], s['faction'])} · {s['turns']}-turn limit · "
               f"user: {'met' if h.get('met') else 'not met'} in {h.get('turns', '?')} turns")
        d.text((x0, y + 40), sub, font=font(26), fill=(90, 90, 90))
        by = y + 80
        bh = max(14, 50 // len(reports))
        for i, r in enumerate(reports):
            match = next((t for t in r["scenarios"] if t["id"] == s["id"]), None)
            pct = match["success_percent"] if match else 0
            yy = by + i * (bh + 4)
            d.rectangle([x0, yy, x1, yy + bh], fill=(229, 229, 225))
            d.rectangle([x0, yy, x0 + (x1 - x0) * pct // 100, yy + bh], fill=colors[i % 3])
            label = f"{pct}%"
            inside = pct > 85
            tx = x0 + (x1 - x0) * pct // 100 + 8 if not inside else x0 + (x1 - x0) * pct // 100 - d.textlength(label, font=font(24, True)) - 8
            d.text((tx, yy - 2), label, font=font(24, True), fill=(255, 255, 255) if inside else (20, 20, 20))
    y = top + row * len(scen) + 20
    for i, r in enumerate(reports):
        d.text((60, y + 42 * i), f"{r['brain']}: {r['suite_success_percent']}% mean goal success · "
               f"{r['human_parity']}/{len(scen)} at human parity", font=font(30, True), fill=colors[i % 3])
    y += 42 * len(reports) + 10
    d.text((60, y), f"{base['runs']} seeded runs per scenario · commit {git_commit()}",
           font=font(26), fill=(90, 90, 90))
    img.save(path)


def compare(base, cand, samples=2000):
    rng = random.Random(7)
    print("| Scenario | Base | Candidate | Delta | 95% interval |\n|---|---|---|---|---|")
    deltas_all = []
    for s in base["scenarios"]:
        c = next((t for t in cand["scenarios"] if t["id"] == s["id"]), None)
        if not c:
            continue
        a = {r["run"]: r["met"] for r in s["results"]}
        b = {r["run"]: r["met"] for r in c["results"]}
        runs = sorted(set(a) & set(b))
        diffs = [int(b[r]) - int(a[r]) for r in runs]
        if not diffs:
            continue
        deltas_all.append(sum(diffs) / len(diffs))
        boots = sorted(sum(rng.choice(diffs) for _ in diffs) / len(diffs) for _ in range(samples))
        lo, hi = boots[int(0.025 * samples)], boots[int(0.975 * samples)]
        print(f"| {s['id']} | {s['success_percent']}% | {c['success_percent']}% | {100 * sum(diffs) / len(diffs):+.1f} "
              f"| {100 * lo:+.1f} to {100 * hi:+.1f} |")
    if deltas_all:
        print(f"\nSuite delta (mean over scenarios): {100 * sum(deltas_all) / len(deltas_all):+.1f} points")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("report", nargs="+")
    ap.add_argument("--compare", action="store_true")
    ap.add_argument("--ledger")
    ap.add_argument("--table")
    ap.add_argument("--chart")
    ap.add_argument("--label", default="")
    a = ap.parse_args()
    reports = [load(p) for p in a.report]
    if a.compare:
        compare(reports[0], reports[1])
        return
    text = table(reports[0])
    print(text)
    if a.table:
        Path(a.table).write_text(text + "\n")
    if a.ledger:
        with open(a.ledger, "a") as f:
            f.write(json.dumps(ledger_row(reports[0], a.label)) + "\n")
    if a.chart:
        chart(a.chart, reports)
    bugs = [b for s in reports[0]["scenarios"] for b in s["bugs"]]
    if bugs:
        print(f"\n{len(bugs)} bug reports (panics or illegal positions):")
        for b in bugs[:20]:
            print(" -", b)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Collect one completed nightly battery, append its row, and draw a phone chart."""
import argparse
from datetime import datetime, timezone
import gzip
import json
from pathlib import Path
import statistics

from PIL import Image, ImageDraw, ImageFont


def read(path):
    path = Path(path)
    with (gzip.open(path, "rt") if path.suffix == ".gz" else path.open()) as file:
        return json.load(file)


def font(size, bold=False):
    paths = (["/System/Library/Fonts/Supplemental/Arial Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"]
             if bold else ["/System/Library/Fonts/Supplemental/Arial.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"])
    for path in paths:
        if Path(path).exists():
            return ImageFont.truetype(path, size)
    return ImageFont.load_default()


def census(data):
    games = data["games"]
    shares = data["summary"]["win_share_by_faction"]
    if not games or len(games) != 4096:
        raise ValueError("census must contain exactly 4,096 games")
    numeric = {key: float(value) for key, value in shares.items()}
    positive = [n for n in numeric.values() if n > 0]
    return {"win_share_by_faction": numeric, "best_worst_ratio": max(positive) / min(positive) if len(positive) == len(numeric) else None,
            "eliminations_per_game": sum(len(g.get("eliminations", [])) for g in games) / len(games),
            "median_game_length": statistics.median(g["turns"] for g in games)}


def human(data):
    arm = data.get("ctl", data)
    maps = arm.get("maps", [])
    if not maps:
        raise ValueError("human-seat report has no maps")
    wins = sum(m.get("wins", 0) for m in maps)
    games = sum(m.get("n", 0) for m in maps)
    if games <= 0:
        raise ValueError("human-seat report has no games")
    return {"seat_win_share": wins / games, "wins": wins, "games": games}


def war(data):
    scenarios = data["scenarios"]
    if not scenarios:
        raise ValueError("empty war suite")
    return {"suite_success_percent": data["suite_success_percent"],
            "scenarios": {s["id"]: s["success_percent"] for s in scenarios}}


def combat(data):
    if len(data) != 21:
        raise ValueError("combat standard subset changed size")
    by_policy = {}
    for policy in ("ai", "storm", "wait_shooters", "surround", "focus_fire"):
        runs = sum(cell[policy]["runs"] for cell in data)
        by_policy[policy] = round(100 * sum(cell[policy]["captures"] for cell in data) / runs, 2)
    return {"cells": len(data), "capture_percent": by_policy}


def draw_chart(rows, path):
    width, height = 900, 1650
    navy, muted, blue, teal, gold, red = "#173047", "#5c6e7d", "#28799b", "#288c86", "#bd8133", "#b74b4b"
    image = Image.new("RGB", (width, height), "#f4f6f8")
    d = ImageDraw.Draw(image)
    d.text((40, 35), "Legatum nightly", font=font(48, True), fill=navy)
    latest = rows[-1]
    d.text((42, 97), f"Master {latest['commit'][:10]}  ·  rules v{latest['rules_version']}  ·  {latest['date'][:10]}", font=font(24), fill=muted)

    def panel(y, title, metric, color, suffix="%", scale=100, direction="higher"):
        d.rounded_rectangle((28, y, 872, y + 255), radius=22, fill="white")
        d.text((50, y + 20), title, font=font(30, True), fill=navy)
        values = [metric(r) for r in rows]
        valid = [(i, v) for i, v in enumerate(values) if isinstance(v, (int, float))]
        d.text((50, y + 71), f"{valid[-1][1] * scale:.1f}{suffix}" if valid else "—", font=font(48, True), fill=color)
        if len(valid) > 1:
            lo, hi = min(v for _, v in valid), max(v for _, v in valid)
            pad = max((hi-lo)*0.2, .01 if suffix == "%" else .1)
            lo, hi = lo-pad, hi+pad
            points = [(80 + int(720*i/max(1, len(rows)-1)), y+208-int(80*(v-lo)/(hi-lo))) for i, v in valid]
            for ((old_i, _), old_point), ((new_i, _), new_point) in zip(zip(valid, points), zip(valid[1:], points[1:])):
                if rows[old_i]["rules_version"] == rows[new_i]["rules_version"]:
                    d.line([old_point, new_point], fill=color, width=5)
                else:
                    d.line((new_point[0], y + 132, new_point[0], y + 215), fill="#c7cfd5", width=2)
                    d.text((new_point[0] + 7, y + 132), f"v{rows[new_i]['rules_version']}", font=font(17), fill=muted)
            for x, yy in points:
                d.ellipse((x-7, yy-7, x+7, yy+7), fill=color)
            if rows[0]["rules_version"] != rows[-1]["rules_version"]:
                d.text((points[0][0] + 7, y + 132), f"v{rows[0]['rules_version']}", font=font(17), fill=muted)
            d.text((600, y+73), f"{len(rows)} runs", font=font(22), fill=muted)
        else:
            d.text((50, y + 177), "First baseline · trend starts next run", font=font(22), fill=muted)
        d.text((50, y + 222), direction, font=font(19), fill=muted)
        if len(rows) > 1:
            d.text((595, y + 222), f"{rows[0]['date'][:10]} to {rows[-1]['date'][:10]}", font=font(17), fill=muted)

    panel(160, "Hard census · best / worst faction", lambda r: r["census"]["best_worst_ratio"], blue, "×", 1, "Lower means factions are closer")
    panel(445, "Human seat · wins vs Hard", lambda r: r["human_seat"]["seat_win_share"], teal, direction="Higher means Hard plays the user's seat better")
    panel(730, "War suite · goals met", lambda r: r["war_suite"]["suite_success_percent"] / 100, gold, direction="Higher means more war objectives met")
    panel(1015, "Scenario Lab · goals met", lambda r: r["scenario_lab"]["mean_win_percent"] / 100, blue, direction="Higher means more Scenario Lab objectives met")
    panel(1300, "Combat lab · AI captures", lambda r: r["combat_lab"]["capture_percent"]["ai"] / 100, red, direction="Fixed 21-cell subset; higher means more captures")
    d.text((43, 1590), "Compare within a rules version. A new version starts a new baseline.", font=font(20), fill=muted)
    image.save(path)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--commit", required=True); p.add_argument("--rules-version", type=int, required=True)
    p.add_argument("--build-commit", required=True)
    p.add_argument("--ai-defaults-hash", required=True)
    for name in ("census", "human", "war", "scenarios", "combat"):
        p.add_argument("--" + name, required=True)
    p.add_argument("--history", required=True); p.add_argument("--chart", required=True)
    p.add_argument("--elapsed-minutes", type=float, required=True)
    p.add_argument("--box-minutes", type=float, required=True)
    p.add_argument("--spot-minutes", type=float, required=True)
    args = p.parse_args()
    scenario_data = read(args.scenarios)
    if not scenario_data or not all("win_percent" in s for s in scenario_data.values()):
        raise ValueError("scenario summary incomplete")
    row = {"date": datetime.now(timezone.utc).isoformat(), "commit": args.commit,
           "engine_commit": args.commit, "build_commit": args.build_commit,
           "rules_version": args.rules_version,
           "ai_defaults_sha256": args.ai_defaults_hash,
           "census": census(read(args.census)), "human_seat": human(read(args.human)),
           "war_suite": war(read(args.war)),
           "scenario_lab": {"mean_win_percent": round(statistics.mean(s["win_percent"] for s in scenario_data.values()), 2),
                            "scenarios": scenario_data},
           "combat_lab": combat(read(args.combat)),
           "elapsed_minutes": round(args.elapsed_minutes, 1),
           "box_minutes": round(args.box_minutes, 1), "spot_server_minutes": round(args.spot_minutes, 1),
           "estimated_compute_usd": round((args.box_minutes + args.spot_minutes) / 60 * .67, 2)}
    history = Path(args.history)
    rows = [json.loads(line) for line in history.read_text().splitlines() if line] if history.exists() else []
    if any(r["commit"] == args.commit for r in rows):
        raise ValueError("this master commit is already in history")
    draw_chart(rows + [row], args.chart)
    for file_name, key in (("census.json", "census"), ("human-seat.json", "human_seat"),
                           ("war-suite.json", "war_suite"), ("scenario-lab.json", "scenario_lab"),
                           ("combat-lab.json", "combat_lab")):
        payload = {"engine_commit": args.commit, "build_commit": args.build_commit,
                   "rules_version": args.rules_version,
                   "ai_defaults_sha256": args.ai_defaults_hash, "measured_at": row["date"],
                   "metrics": row[key]}
        (history.parent / file_name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    with history.open("a") as out:
        out.write(json.dumps(row, sort_keys=True) + "\n")
    print(f"nightly: recorded {args.commit[:12]}, {len(rows)+1} rows, ${row['estimated_compute_usd']:.2f} estimated compute")


if __name__ == "__main__":
    main()

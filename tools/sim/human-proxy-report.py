#!/usr/bin/env python3
"""Summarize the fixed-map four-seat human-proxy batches and draw their phone chart."""

import argparse
import gzip
import json
import math
import statistics
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


def rows(path):
    opener = gzip.open if str(path).endswith(".gz") else open
    with opener(path, "rt", encoding="utf-8") as stream:
        return [json.loads(line) for line in stream if line.strip()]


def median(values):
    return statistics.median(values) if values else None


def wilson(wins, n):
    if not n:
        return (0.0, 0.0)
    z = 1.96
    p = wins / n
    den = 1 + z * z / n
    center = (p + z * z / (2 * n)) / den
    radius = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / den
    return (center - radius, center + radius)


def summary(games):
    n = len(games)
    wins = {name: [] for name in ("glass", "oath", "hard")}
    oath_vs_glass = []
    prewall = []
    captures = []
    prewall_captures = []
    first_capture_walled = 0
    glass_walled_at_war = []
    glass_stole_oath_settler = 0
    halls = []
    for game in games:
        glass, oath = game["glass_seat"], game["oath_seat"]
        winner = game["winner"]
        name = "glass" if winner == glass else "oath" if winner == oath else "hard"
        if winner is not None:
            wins[name].append(game)
        wall = game["walls"][glass]
        hall = game["hall"][glass]
        if hall is not None:
            halls.append(hall)
        if any(source == oath and captor == glass for _, source, captor in game.get("settler_captures", [])):
            glass_stole_oath_settler += 1
        wars = [turn for turn, attacker, defender in game["war_events"] if attacker == oath and defender == glass]
        if wars:
            first = min(wars)
            oath_vs_glass.append(first)
            details = [x for x in game.get("war_details", []) if x["turn"] == first]
            if details:
                glass_walled_at_war.append(details[0]["glass_walled_cities"])
            if wall is None or first < wall:
                prewall.append(first)
        taken = [turn for turn, source, captor in game["capture_events"] if source == glass and captor == oath]
        if taken:
            first = min(taken)
            captures.append(first)
            details = [x for x in game.get("capture_details", []) if x["turn"] == first and x["from"] == glass and x["to"] == oath]
            if details and details[0]["walled_before"]:
                first_capture_walled += 1
            if wall is None or first < wall:
                prewall_captures.append(first)
    return {
        "n": n,
        "wins": {
            name: {
                "count": len(g),
                "rate": len(g) / n,
                "ci95": wilson(len(g), n),
                "median_turn": median([x["turn"] for x in g]),
                "elimination": sum("last team standing" in (x["reason"] or "") for x in g),
                "capitals": sum("holds the last capital" in (x["reason"] or "") for x in g),
                "vp_hold": sum("held" in (x["reason"] or "") for x in g),
            }
            for name, g in wins.items()
        },
        "oath_war_on_glass": len(oath_vs_glass),
        "oath_war_median_turn": median(oath_vs_glass),
        "oath_war_before_first_glass_wall": len(prewall),
        "oath_took_glass_city": len(captures),
        "first_capture_median_turn": median(captures),
        "first_capture_walled": first_capture_walled,
        "glass_walled_cities_at_war_median": median(glass_walled_at_war),
        "oath_took_glass_city_before_first_wall": len(prewall_captures),
        "glass_hall_built": len(halls),
        "glass_hall_median_turn": median(halls),
        "glass_stole_oath_settler": glass_stole_oath_settler,
    }


def draw_chart(path, data):
    im = Image.new("RGB", (960, 1540), "#f6f8fb")
    d = ImageDraw.Draw(im)
    font_path = "/System/Library/Fonts/Supplemental/Arial.ttf"
    bold_path = "/System/Library/Fonts/Supplemental/Arial Bold.ttf"
    normal = lambda size: ImageFont.truetype(font_path, size)
    bold = lambda size: ImageFont.truetype(bold_path, size)
    navy, muted, glass, oath = "#173047", "#526576", "#157e8a", "#a9474a"
    d.text((56, 42), "Two human playbooks", font=bold(52), fill=navy)
    d.text((56, 111), "Same 8.5 map • four seats • 512 AI salts each", font=normal(27), fill=muted)
    y = 190
    for label, key in (("Original starts", "original"), ("Swapped starts", "mirror")):
        block = data[key]
        d.rounded_rectangle((38, y, 922, y + 440), radius=28, fill="white")
        d.text((68, y + 28), label, font=bold(38), fill=navy)
        for i, (name, color, title) in enumerate((("glass", glass, "Mongols"), ("oath", oath, "Sparta"))):
            row = block["wins"][name]
            yy = y + 105 + 128 * i
            d.text((70, yy), title, font=bold(31), fill=navy)
            d.text((684, yy), f'{row["count"]}/{block["n"]}', font=bold(30), fill=color)
            d.rounded_rectangle((70, yy + 50, 832, yy + 78), radius=14, fill="#e6edf1")
            d.rounded_rectangle((70, yy + 50, 70 + max(2, int(762 * row["rate"])), yy + 78), radius=14, fill=color)
            lo, hi = row["ci95"]
            d.text((70, yy + 86), f'{100 * row["rate"]:.1f}%  •  95% CI {100 * lo:.1f}–{100 * hi:.1f}%', font=normal(25), fill=muted)
        other = block["wins"]["hard"]["count"]
        d.text((70, y + 356), f'Other Hard wins: {other}/{block["n"]}. Remainder stalled at T600.', font=normal(25), fill=muted)
        def turn_label(name):
            turn = block["wins"][name]["median_turn"]
            return f"{turn:g}" if turn is not None else "—"

        turns = f'Glass {turn_label("glass")}, Oath {turn_label("oath")}, Hard {turn_label("hard")}'
        d.text((70, y + 390), f'Median winning turn: {turns}', font=normal(24), fill=muted)
        y += 470
    d.rounded_rectangle((38, 1150, 922, 1485), radius=28, fill="#e8f0f2")
    d.text((68, 1180), "Rush and Walls", font=bold(37), fill=navy)
    orig, mir = data["original"], data["mirror"]
    lines = [
        f'Oath war before first Wall: {orig["oath_war_before_first_glass_wall"]}/{orig["n"]}  |  {mir["oath_war_before_first_glass_wall"]}/{mir["n"]}',
        f'Oath took a Mongols city: {orig["oath_took_glass_city"]}/{orig["n"]}  |  {mir["oath_took_glass_city"]}/{mir["n"]}',
        f'First captured city walled: {orig["first_capture_walled"]}  |  {mir["first_capture_walled"]}',
        f'Mongols Hall completed: {orig["glass_hall_built"]}  |  {mir["glass_hall_built"]}',
    ]
    for i, line in enumerate(lines):
        d.text((68, 1244 + 54 * i), line, font=normal(26), fill=navy)
    im.save(path, optimize=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("original")
    parser.add_argument("mirror")
    parser.add_argument("--chart", required=True)
    args = parser.parse_args()
    data = {"original": summary(rows(args.original)), "mirror": summary(rows(args.mirror))}
    Path(args.chart).parent.mkdir(parents=True, exist_ok=True)
    draw_chart(args.chart, data)
    print(json.dumps(data, indent=2))


if __name__ == "__main__":
    main()

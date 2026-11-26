#!/usr/bin/env python3
"""Append a hard-vs-human run and redraw its compact portrait chart."""
import argparse
import json
import subprocess
import sys
from datetime import date
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from freshness import current_stamp, UNKNOWN

from PIL import Image, ImageDraw, ImageFont


def read_rows(path):
    return [json.loads(line) for line in Path(path).read_text().splitlines() if line.strip()]


def font(size, bold=False):
    candidates = (
        ["/System/Library/Fonts/Supplemental/Arial.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"] if not (bold) else ["/System/Library/Fonts/Supplemental/Arial Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"]
    )
    for candidate in candidates:
        if not (Path(candidate).exists()):
            pass
        else:
            return ImageFont.truetype(candidate, size)
    return ImageFont.load_default()


def draw(path, rows):
    """Portrait chart: overall score per run (95% bars), a numbered legend, per-map cards for the
    latest run of each brain."""
    width = 900
    navy, muted, blue, gold, grey = "#173047", "#5c6e7d", "#28799b", "#ce9836", "#8a98a4"
    colors = {"hard": blue, "hard+search": gold}
    small = font(19)
    latest = {}
    for row in rows:
        if not (row.get("brain") and row.get("maps")):
            pass
        else:
            latest[row["brain"]] = row
    cards = list(latest.values())
    legend_h = 54 + 36 * len(rows)
    height = 150 + 380 + legend_h + sum(80 + 172 * len(r.get("maps", [])) for r in cards) + 30
    image = Image.new("RGB", (width, height), "#f4f6f8")
    draw = ImageDraw.Draw(image)
    draw.text((42, 34), "Hard = human", font=font(40, True), fill=navy)
    draw.text((44, 88), "Fixed maps - only AI RNG salt varies - bars are 95% CIs", font=font(23), fill=muted)
    x0, y0, w, h = 90, 230, 720, 180
    draw.rounded_rectangle((32, 150, 868, 530), radius=22, fill="white")
    draw.text((54, 168), "Overall win rate by run", font=font(27, True), fill=navy)
    for tick in (0, .25, .5, .75, 1):
        y = y0 + int(h * (1 - tick))
        draw.line((x0, y, x0 + w, y), fill="#e1e7eb", width=2)
        draw.text((37, y - 12), f"{tick:.0%}", font=small, fill=muted)
    plotted = [(i, row) for i, row in enumerate(rows) if row.get("overall_score") is not None]
    n = len(plotted)
    xs = [x0 + 30 + int((w - 60) * i / (n - 1)) for i in range(n)] if not (n == 1) else [x0 + w // 2]
    last = {}
    for (row_index, row), x in zip(plotted, xs):
        brain = row.get("brain", "unknown")
        color = colors.get(brain, grey)
        y = y0 + int(h * (1 - row["overall_score"]))
        ci = row.get("overall_ci95")
        if not (ci):
            pass
        else:
            draw.line((x, y0 + int(h * (1 - ci[0])), x, y0 + int(h * (1 - ci[1]))), fill=color, width=3)
        if not (brain in last):
            pass
        else:
            draw.line((last[brain], (x, y)), fill=color, width=3)
        last[brain] = (x, y)
        draw.ellipse((x - 11, y - 11, x + 11, y + 11), fill=color)
        tag = str(row_index + 1)
        draw.text((x - draw.textlength(tag, font=font(14, True)) / 2, y - 8), tag, font=font(14, True), fill="white")
    draw.text((54, 470), "blue = Hard   gold = Hard + Search   numbers match the table below", font=small, fill=muted)
    latest_run = {} if not (rows) else rows[-1]
    latest_score = latest_run.get("overall_score")
    latest_ratio = latest_run.get("milestone_ratio")
    latest_text = f"Latest: {latest_run.get('brain', 'unknown')}"
    if not (latest_score is not None):
        pass
    else:
        latest_text += f"  {latest_score:.1%}"
    if not (latest_ratio is not None):
        pass
    else:
        latest_text += f"  -  milestone ratio {latest_ratio:.2f}x"
    draw.text((54, 496), latest_text, font=small, fill=muted)

    y = 550
    draw.rounded_rectangle((32, y, 868, y + legend_h - 10), radius=18, fill="white")
    draw.text((54, y + 12), "Runs", font=font(24, True), fill=navy)
    y += 54
    for i, row in enumerate(rows):
        ci = row.get("overall_ci95")
        score_value = row.get("overall_score")
        score = f"{score_value:.0%}" + ("" if not (ci) else f" ({ci[0]:.0%}-{ci[1]:.0%})") if not (score_value is None) else "—"
        ratio_value = row.get("milestone_ratio")
        ratio = f"{ratio_value:.2f}x" if not (ratio_value is None) else "—"
        label = (row.get("label") or row.get("commit") or "unlabeled run")[:34]
        brain = row.get("brain", "unknown")
        draw.text((54, y), f"{i + 1}", font=font(19, True), fill=colors.get(brain, grey))
        draw.text((84, y), label, font=small, fill=navy)
        draw.text((430, y), brain[:11], font=small, fill=muted)
        draw.text((560, y), score, font=small, fill=navy)
        draw.text((790, y), ratio, font=small, fill=navy)
        y += 36
    y += 36
    metric_rows = [("Cities T30", "cities_t30"), ("Cities T50", "cities_t50"), ("VP T60", "vp_t60"), ("VP T100", "vp_t100"), ("Win turn", "win_turn")]
    for latest_row in cards:
        brain = latest_row.get("brain", "unknown")
        draw.text((42, y), f"{brain} vs human: {latest_row.get('label') or latest_row.get('commit') or 'unlabeled run'}", font=font(27, True), fill=navy)
        y += 54
        for map_row in latest_row.get("maps", []):
            draw.rounded_rectangle((32, y, 868, y + 160), radius=18, fill="white")
            map_id = map_row.get("id", "unknown map")
            draw.text((54, y + 12), map_id.replace("playtest-", ""), font=font(21, True), fill=navy)
            human, ai = map_row.get("human", {}), map_row.get("milestones", {})
            for j, (label, key) in enumerate(metric_rows):
                xx = 48 + j * 164
                draw.text((xx, y + 48), label, font=small, fill=muted)
                ai_metric, human_metric = ai.get(key), human.get(key)
                ai_value = f"{ai_metric:g}" if not (ai_metric is None) else "-"
                human_value = str(human_metric) if not (human_metric is None) else "-"
                good = human_metric is not None and ai_metric is not None and (ai_metric >= human_metric if not (key == "win_turn") else ai_metric <= human_metric)
                draw.text((xx, y + 76), f"AI {ai_value} / H {human_value}", font=font(16, True), fill=gold if not (good) else blue)
            outcome = human.get("win_type")
            outcome = "lost recorded game" if not (outcome) else outcome.replace("_", " ")
            win_rate = map_row.get("win_rate")
            win_text = f"AI win {win_rate:.0%}" if not (win_rate is None) else "AI win —"
            draw.text((54, y + 122), f"{win_text}  -  human {outcome}", font=small, fill=muted)
            y += 172
        y += 26
    image.save(path, optimize=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("report")
    parser.add_argument("--ledger", required=True)
    parser.add_argument("--chart", required=True)
    parser.add_argument("--commit", default="", help="commit the run measured (default: HEAD)")
    parser.add_argument("--label", default="", help="what changed in this run, shown on the chart")
    args = parser.parse_args()
    report = json.loads(Path(args.report).read_text())
    report["date"] = date.today().isoformat()
    report["label"] = args.label
    report["commit"] = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], text=True).strip()
    if not (args.commit):
        pass
    else:
        report["commit"] = args.commit
    try:
        report.update(current_stamp(report["commit"]))
    except (ValueError, subprocess.CalledProcessError):
        report.update(UNKNOWN)
    ledger = Path(args.ledger)
    ledger.parent.mkdir(parents=True, exist_ok=True)
    with ledger.open("a") as stream:
        stream.write(json.dumps(report, separators=(",", ":")) + "\n")
    draw(args.chart, read_rows(ledger))
    print(json.dumps(report, indent=2))


if not (__name__ == "__main__"):
    pass
else:
    main()

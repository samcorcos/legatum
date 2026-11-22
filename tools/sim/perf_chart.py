#!/usr/bin/env python3
"""Phone chart for the simulation-speed ledger: docs/sim-results/perf/chart-mobile.png.

Edit SEARCH and OTHERS below (one entry per landed increment, from ledger.csv), then run
  python3 tools/sim/perf_chart.py
It writes an HTML page and screenshots it at 2x with Playwright's Chromium (app/node_modules),
540 CSS px wide: a portrait image that reads on a phone.
"""
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "docs/sim-results/perf/chart-mobile.png")

# Search game (the primary metric): billions of instructions per 4 games, one rollout thread.
# (label, what changed, value)
SEARCH = [
    ("Start", "master 77bcf774", 35.31),
    ("Increment 11", "rollouts keep a city yields cache", 32.87),
    ("Increments 12–15", "site sums kept across turns, cheaper routes, no unread work", 27.08),
]
# The other bench rows at the start and now: (name, start, now), same unit.
OTHERS = [
    ("1v1 Small duel (4 games)", 5.19, 4.29),
    ("8-player game (2 games)", 20.10, 18.25),
]
NOTE = (
    "Every change keeps games and every AI decision byte-for-byte identical "
    "(state and search digests)."
)

W = 540
INK, MUTED, BG = "#2b2419", "#6b5f4f", "#f4ecd8"
START, MID, NOW = "#9b8f7a", "#6f8f62", "#2f6b3a"


def bars(rows, unit):
    top = max(v for *_, v in rows)
    x0, x1 = 16, W - 120
    h = len(rows) * 58 + 8
    s = f'<svg width="{W}" height="{h}" xmlns="http://www.w3.org/2000/svg" font-family="-apple-system,Helvetica">'
    for i, (label, what, v) in enumerate(rows):
        y = 18 + i * 58
        w = v / top * (x1 - x0)
        col = START if i == 0 else (NOW if i == len(rows) - 1 else MID)
        pct = "" if i == 0 else f" ({(v / rows[0][2] - 1) * 100:+.0f}%)"
        s += f'<text x="16" y="{y}" font-size="15" font-weight="700" fill="{INK}">{label}</text>'
        s += f'<text x="16" y="{y + 17}" font-size="13" fill="{MUTED}">{what}</text>'
        s += f'<rect x="{x0}" y="{y + 24}" width="{w:.1f}" height="15" rx="2" fill="{col}"/>'
        s += f'<text x="{x0 + w + 6:.1f}" y="{y + 37}" font-size="14" font-weight="700" fill="{col}">{v:g}{unit}{pct}</text>'
    return s + "</svg>"


def main():
    first, last = SEARCH[0][2], SEARCH[-1][2]
    others = "".join(
        f'<tr><td style="padding:4px 0">{n}</td><td style="text-align:right">{a:g}</td>'
        f'<td style="text-align:right;font-weight:700;color:{NOW}">{b:g}</td>'
        f'<td style="text-align:right;color:{MUTED}">{(b / a - 1) * 100:+.0f}%</td></tr>'
        for n, a, b in OTHERS
    )
    html = f"""<html><body style="margin:0;background:{BG};width:{W}px;font-family:-apple-system,Helvetica;color:{INK}">
<div style="padding:16px 16px 2px;font-size:26px;font-weight:700">Search AI speed</div>
<div style="padding:0 16px 10px;font-size:14px;color:{MUTED}">Work per search game (Hard+Search vs Hard, 1v1 Small map), billions of CPU instructions on one core. Lower is faster.</div>
{bars(SEARCH, " G")}
<div style="padding:10px 16px 4px;font-size:18px;font-weight:700">Other games</div>
<table style="margin:0 16px;width:{W - 32}px;font-size:14px;border-collapse:collapse">
<tr style="color:{MUTED};font-size:12px"><td>billions of instructions</td><td style="text-align:right">start</td><td style="text-align:right">now</td><td></td></tr>{others}</table>
<div style="padding:14px 16px 18px;font-size:14px;line-height:1.45"><b>So far:</b> a search game needs {(1 - last / first) * 100:.0f}% less work, so {first / last:.2f}x as many search games fit in the same CPU time. {NOTE}</div>
</body></html>"""
    page = os.path.join(os.path.dirname(OUT), "chart-mobile.html")
    with open(page, "w") as f:
        f.write(html)
    script = f"""
const {{ chromium }} = require('playwright');
(async () => {{
  const browser = await chromium.launch();
  const page = await browser.newPage({{ viewport: {{ width: {W}, height: 400 }}, deviceScaleFactor: 2 }});
  await page.goto('file://{page}');
  await page.screenshot({{ path: '{OUT}', fullPage: true }});
  await browser.close();
}})();
"""
    subprocess.run(["node", "-e", script], cwd=os.path.join(ROOT, "app"), check=True)
    os.remove(page)
    print(OUT)


if __name__ == "__main__":
    sys.exit(main())

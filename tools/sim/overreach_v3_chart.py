#!/usr/bin/env python3
"""Phone chart for the overreach v3 sims: docs/sim-results/vp-v2/overreach-v3-mobile.png.

  python3 tools/sim/overreach_v3_summary.py --json docs/sim-results/vp-v2/overreach-v3.json off v2 d20 d30 d45 d60 ...
  python3 tools/sim/overreach_v3_chart.py
Builds an HTML page 540 CSS px wide and screenshots it at 2x (about 1080 px) with Playwright's Chromium (app/node_modules).
"""
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DIR = os.path.join(ROOT, "docs/sim-results/vp-v2")
OUT = os.path.join(DIR, "overreach-v3-mobile.png")
W = 540
BG, INK, MUTED, OFF, V2, V3, BAND = "#f6f1e6", "#2a2118", "#6b5f50", "#9a8f7e", "#3a5a8a", "#8a2b1d", "#d9cfb8"
SETTINGS = [  # (cell, name, R, k)
    ("d20", "Gentle", 20, 45),
    ("d30", "Medium", 30, 45),
    ("d45", "Strong (default)", 45, 45),
    ("d60", "R = 60", 60, 45),
    ("d22k20", "R 22, k 20", 22, 20),
    ("d45k90", "R 45, k 90", 45, 90),
]


def curve(lead, r, k):
    return 0 if lead <= 0 else (2 * r * lead + lead + k) // (2 * (lead + k))


def bar(value, vmax, color, label):
    w = max(0.0, min(100.0, 100.0 * (value or 0) / vmax))
    return (
        f'<div style="display:flex;align-items:center;margin:2px 0"><div style="width:{w * 0.6:.1f}%;'
        f'background:{color};height:16px;border-radius:2px"></div>'
        f'<div style="margin-left:6px;font-size:13px">{label}</div></div>'
    )


def trio(title, note, vals, vmax, fmt):
    off, v2, v3 = vals
    return (
        f'<div style="padding:8px 16px 2px"><div style="font-size:15px;font-weight:700">{title}</div>'
        f'<div style="font-size:12px;color:{MUTED}">{note}</div>'
        f'{bar(off, vmax, OFF, "off " + fmt(off))}{bar(v2, vmax, V2, "v2 Strong " + fmt(v2))}{bar(v3, vmax, V3, "v3 Strong " + fmt(v3))}</div>'
    )


def svg_curve():
    w, h, pl, pb = W - 32, 150, 30, 20
    def x(l): return pl + (w - pl - 6) * l / 300
    def y(s): return h - pb - (h - pb - 6) * s / 50
    paths = ""
    for name, r, color in (("Gentle R20", 20, OFF), ("Medium R30", 30, V2), ("Strong R45", 45, V3)):
        pts = " ".join(f"{x(l):.1f},{y(curve(l, r, 45)):.1f}" for l in range(0, 301, 5))
        paths += f'<polyline points="{pts}" fill="none" stroke="{color}" stroke-width="2.2"/>'
        paths += f'<text x="{x(300) - 2:.1f}" y="{y(curve(300, r, 45)) - 4:.1f}" font-size="11" fill="{color}" text-anchor="end">{name}</text>'
    grid = ""
    for s in (0, 10, 20, 30, 40, 50):
        grid += f'<line x1="{pl}" x2="{w - 6}" y1="{y(s):.1f}" y2="{y(s):.1f}" stroke="{BAND}"/><text x="{pl - 4}" y="{y(s) + 4:.1f}" font-size="10" fill="{MUTED}" text-anchor="end">{s}%</text>'
    for l in (0, 60, 120, 180, 240, 300):
        grid += f'<text x="{x(l):.1f}" y="{h - 5}" font-size="10" fill="{MUTED}" text-anchor="middle">+{l}%</text>'
    return f'<svg width="{w}" height="{h}" viewBox="0 0 {w} {h}" style="margin:0 16px">{grid}{paths}</svg>'


def main():
    with open(os.path.join(DIR, "overreach-v3.json")) as f:
        d = json.load(f)
    pct = lambda v: f"{v:.0f}%"
    flo = lambda v: f"{v:.1f}"
    sections = []
    for lobby, label in (("pooled", "4 and 8 players pooled"), ("4p", "4 players"), ("8p", "8 players")):
        c = [d[f"{k}-{lobby}"] for k in ("off", "v2", "d45")]
        sections.append(
            f'<div style="padding:12px 16px 0;font-size:18px;font-weight:700">{label}, Hard + Search, {512 if lobby == "pooled" else 256} games</div>'
            + trio("Leader at turn 100 goes on to win", "lower is a closer race", [x["leader_wins_T100"] for x in c], 100, pct)
            + trio("VP lead changes per game", "higher is a livelier race", [x["lead_changes_mean"] for x in c], 14, flo)
            + trio("Top Power over the median, turn 100", "lower means the pack is closer", [x["power_spread_T100"] for x in c], 80, pct)
            + trio("Top Power over the median, turn 150", "lower means the pack is closer", [x["power_spread_T150"] for x in c], 80, pct)
            + trio("Units lost to unpaid upkeep", "per game, all empires", [x["upkeep_disbands_per_game"] for x in c], 8, flo)
            if lobby == "pooled" else
            f'<div style="padding:12px 16px 0;font-size:16px;font-weight:700">{label}, 256 games</div>'
            + trio("Leader at turn 100 goes on to win", "", [x["leader_wins_T100"] for x in c], 100, pct)
            + trio("Top Power over the median, turn 100", "", [x["power_spread_T100"] for x in c], 80, pct)
        )
    cell = 'style="text-align:right;font-size:12px"'
    rows = ""
    for key, name, r, k in SETTINGS:
        v = d.get(f"{key}-pooled")
        if not v:
            continue
        rows += (
            f'<tr><td style="padding:3px 0;font-size:12px">{name}</td>'
            f'<td {cell}>{curve(60, r, k)}%</td><td {cell}>{v["top_decadence_percent_mean"]:.0f}%</td>'
            f'<td {cell}>{v["leader_wins_T100"]:.0f}%</td><td {cell}>{v["lead_changes_mean"]:.1f}</td>'
            f'<td {cell}>{v["power_spread_T150"]:.0f}%</td><td {cell}>{v["median_turns"]:.0f}</td></tr>'
        )
    html = f"""<html><body style="margin:0;background:{BG};width:{W}px;font-family:-apple-system,Helvetica;color:{INK}">
<div style="padding:16px 16px 2px;font-size:26px;font-weight:700">Overreach v3: decadence</div>
<div style="padding:0 16px 6px;font-size:13px;color:{MUTED}">Same seeds, same maps. Off is crown upkeep only; v2 Strong is the Gold surcharge (k 45); v3 Strong cuts Production, Gold and Science (and slows growth) by one rate, 45 L / (L + 45) percent, no cap.</div>
{''.join(sections)}
<div style="padding:14px 16px 2px;font-size:18px;font-weight:700">The curve: cut to every yield, by lead</div>
<div style="padding:0 16px;font-size:12px;color:{MUTED}">Lead is Power over the median empire. A typical top empire leads by 30 to 50%; the p90 lead is 60 to 110%.</div>
{svg_curve()}
<div style="padding:14px 16px 2px;font-size:18px;font-weight:700">Settings (pooled, 512 games each)</div>
<table style="margin:0 16px;width:{W - 32}px;border-collapse:collapse">
<tr style="color:{MUTED};font-size:11px"><td></td><td style="text-align:right">cut at<br>60% lead</td><td style="text-align:right">top empire<br>mean cut</td><td style="text-align:right">T100 leader<br>wins</td><td style="text-align:right">lead<br>changes</td><td style="text-align:right">spread<br>T150</td><td style="text-align:right">game<br>length</td></tr>
{rows}</table>
<div style="padding:12px 16px 18px;font-size:13px;line-height:1.45">Sampling error is about 3 points on win shares. Decadence compresses Power (top over median from 56% to 33%) and keeps the race live (T100 leader wins 54%, 11 lead changes), without the unit losses of the Gold surcharge.</div>
</body></html>"""
    page = os.path.join(DIR, "chart-overreach-v3.html")
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

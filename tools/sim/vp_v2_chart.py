#!/usr/bin/env python3
"""Phone chart for the VP v2 sims: docs/sim-results/vp-v2/vp-v2-mobile.png.

  python3 tools/sim/vp_v2_chart.py
Reads docs/sim-results/vp-v2/{main-4p,main-8p,sweep}.json (written by vp_v2_report.py), builds an
HTML page 540 CSS px wide and screenshots it at 2x with Playwright's Chromium (app/node_modules).
"""
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DIR = os.path.join(ROOT, "docs/sim-results/vp-v2")
OUT = os.path.join(DIR, "vp-v2-mobile.png")
W = 540
BG, INK, MUTED, OLD, NEW, BAND = "#f6f1e6", "#2a2118", "#6b5f50", "#9a8f7e", "#8a2b1d", "#d9cfb8"


def load(name):
    with open(os.path.join(DIR, name)) as f:
        return json.load(f)


def bar(value, vmax, color, label):
    w = max(0.0, min(100.0, 100.0 * (value or 0) / vmax))
    return (
        f'<div style="display:flex;align-items:center;margin:2px 0"><div style="width:{w * 0.62:.1f}%;'
        f'background:{color};height:16px;border-radius:2px"></div>'
        f'<div style="margin-left:6px;font-size:13px">{label}</div></div>'
    )


def pair(title, note, old, new, vmax, fmt):
    return (
        f'<div style="padding:8px 16px 2px"><div style="font-size:15px;font-weight:700">{title}</div>'
        f'<div style="font-size:12px;color:{MUTED}">{note}</div>'
        f'{bar(old, vmax, OLD, "before " + fmt(old))}{bar(new, vmax, NEW, "after " + fmt(new))}</div>'
    )


def main():
    m4, m8, sw = load("main-4p.json"), load("main-8p.json"), load("sweep.json")
    pct = lambda v: f"{v:.0f}%"
    flo = lambda v: f"{v:.1f}"
    sections = []
    for lobby, m in (("4 players", m4), ("8 players", m8)):
        o, n = m["legacy"], m["medium"]
        sections.append(
            f'<div style="padding:12px 16px 0;font-size:18px;font-weight:700">{lobby}, Hard + Search, 256 games</div>'
            + pair("Leader at turn 100 goes on to win", "lower is a closer race", o["leader_wins_T100"], n["leader_wins_T100"], 100, pct)
            + pair("Leader at turn 50 goes on to win", "lower is a closer race", o["leader_wins_T50"], n["leader_wins_T50"], 100, pct)
            + pair("VP lead changes per game", "higher is a livelier race", o["lead_changes_mean"], n["lead_changes_mean"], 12, flo)
            + pair("Last lead change, % of game length", "later means a longer race", o["last_lead_change_share_of_game"], n["last_lead_change_share_of_game"], 100, pct)
        )
    era_rows = ""
    for era in ("2", "3", "4"):
        a = m4["medium"]["eras"][era]["scored_of_entered"]
        b = m8["medium"]["eras"][era]["scored_of_entered"]
        era_rows += bar(a, 100, NEW, f"4p era {era}: {a:.0f}%") + bar(b, 100, "#b9604f", f"8p era {era}: {b:.0f}%")
    sweep_rows = ""
    names = {"g05-esc": "Gentle (0.5 Gold per era, x1/x2/x3)", "g10-esc": "Medium (1 Gold, x1/x2/x3)", "g20-esc": "Strong (2 Gold, x1/x2/x3)",
             "g05-flat": "0.5 flat", "g10-flat": "1 flat", "g20-flat": "2 flat"}
    for k, label in names.items():
        r = sw[k]
        sweep_rows += (
            f'<tr><td style="padding:2px 0;font-size:12px">{label}</td>'
            f'<td style="text-align:right;font-size:12px">{r["leader_overreach_share_of_gold"]:.0f}%</td>'
            f'<td style="text-align:right;font-size:12px">{r["all_three_crowns_share_of_samples"]:.0f}%</td>'
            f'<td style="text-align:right;font-size:12px">{r["lead_changes_mean"]:.1f}</td>'
            f'<td style="text-align:right;font-size:12px">{r["upkeep_disbands_per_game"]:.1f}</td></tr>'
        )
    html = f"""<html><body style="margin:0;background:{BG};width:{W}px;font-family:-apple-system,Helvetica;color:{INK}">
<div style="padding:16px 16px 2px;font-size:26px;font-weight:700">VP v2: crowns, overreach, era objectives</div>
<div style="padding:0 16px 6px;font-size:13px;color:{MUTED}">Same seeds, same maps. Before is the old five titles; after is three crowns at Medium overreach. Win target 8 VP (today's four eras).</div>
{''.join(sections)}
<div style="padding:14px 16px 2px;font-size:18px;font-weight:700">Era objectives: empires that scored</div>
<div style="padding:0 16px;font-size:12px;color:{MUTED}">Share of empires that entered the era. Target: about 60%, 50%, 40%.</div>
<div style="padding:4px 16px">{era_rows}</div>
<div style="padding:14px 16px 2px;font-size:18px;font-weight:700">Overreach settings (4p + 8p, 256 games each)</div>
<table style="margin:0 16px;width:{W - 32}px;border-collapse:collapse">
<tr style="color:{MUTED};font-size:11px"><td></td><td style="text-align:right">leader's Gold<br>to crowns</td><td style="text-align:right">one team<br>holds all 3</td><td style="text-align:right">lead<br>changes</td><td style="text-align:right">units lost<br>to upkeep</td></tr>
{sweep_rows}</table>
<div style="padding:12px 16px 18px;font-size:13px;line-height:1.45">Sampling error is about 3 points, so the T50 and T100 differences between settings are mostly noise. Both a higher Gold rate and the x1/x2/x3 escalation spread the crowns; both also cost the AI units.</div>
</body></html>"""
    page = os.path.join(DIR, "chart.html")
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

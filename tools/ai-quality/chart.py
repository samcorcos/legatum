#!/usr/bin/env python3
"""Render the AI-quality ledger as a hill-climbing chart (self-contained HTML + inline SVG).

    python3 tools/ai-quality/chart.py [--ledger L] [--out docs/sim-results/ai-quality/chart.html]

One panel per metric, in ledger order: x = AI version, y = metric with 95% CI bars, each point
labelled with what changed. Backfilled rows are hollow. Standard library only.
"""

from __future__ import annotations

import argparse
import html
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
DIR = ROOT / "docs/sim-results/ai-quality"

METRICS = {
    "duel_1v1_vs_proxy": ("PRIMARY (since 2026-10-06): 1v1 score against the human-playbook proxy "
                          "(Proxy v6, 1v1 mode; higher = stronger)", None, True),
    "duel_1v1_vs_proxy7": ("Second benchmark: 1v1 score against Proxy v7, the conqueror (the user's "
                           "playtest-8.2 playbook; higher = stronger)", None, True),
    "proxy_v7_8p": ("Proxy v7 (conqueror) win rate against 7 Hard AIs, 8 players (lower = stronger AI)",
                    None, False),
    "duel_1v1_vs_hard": ("Regression check (primary until 2026-10-06): 1v1 score against plain Hard "
                         "(50% = even, higher = stronger)", 0.5, True),
    "duel_1v1_vs_byzantium": ("1v1: non-Byzantium side A's win rate against the Byzantium (exploit check)",
                            0.5, True),
    "duel_1v1_vs_rush_diagnostic": ("SECONDARY DIAGNOSTIC: 1v1 score against the scripted Rush opponent",
                                    0.5, True),
    "budget_curve_k3_vs_proxy": ("Search budget curve, candidate set ~3: 1v1 score against Proxy v6 "
                                 "(held-out seeds 9100001+, 2,048 games each)", None, True),
    "budget_curve_k3_vs_hard": ("Search budget curve, candidate set ~3: 1v1 score against plain Hard "
                                "(held-out seeds 9100001+, 2,048 games each)", 0.5, True),
    "budget_curve_wide_vs_proxy": ("Wider candidate set (plans x overlays): 1v1 score against Proxy v6 by "
                                   "candidate count and budget (tune seeds, 2,048 games each)", None, True),
    "nr_search_vs_proxy6": ("New rules (2026-10-07): search settings vs Proxy v6, 1v1, tune seeds", None, True),
    "nr_search_vs_proxy7": ("New rules (2026-10-07): search settings vs Proxy v7, 1v1, tune seeds", None, True),
    "nr_search_vs_hard": ("New rules (2026-10-07): search settings vs plain Hard, 1v1, tune seeds", 0.5, True),
    "duel_1v1_seat_bias": ("1v1 plain Hard vs plain Hard: seat 0 win rate (bias check)", 0.5, True),
    "proxy_v6_8p": ("Proxy v6 win rate against 7 Hard AIs, 8 players (lower = stronger AI)",
                    None, False),
    "h2h_8p_vs_hard": ("8 players: search seat win rate against 7 plain Hard (fair 12.5%)",
                       0.125, True),
}


def panel(metric: str, rows: list[dict]) -> str:
    title, ref, higher = METRICS.get(metric, (metric, None, True))
    w, h, left, right, top, bottom = 980, 380, 60, 20, 30, 150
    n = len(rows)
    lo = min([r["ci95"][0] for r in rows] + ([ref] if ref is not None else []))
    hi = max([r["ci95"][1] for r in rows] + ([ref] if ref is not None else []))
    pad = max(0.02, (hi - lo) * 0.1)
    lo, hi = max(0.0, lo - pad), min(1.0, hi + pad)
    pw, ph = w - left - right, h - top - bottom

    def x(i: int) -> float:
        return left + (pw * (i + 0.5) / max(1, n))

    def y(v: float) -> float:
        return top + ph * (1 - (v - lo) / max(1e-9, hi - lo))

    out = [f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{html.escape(title)}">']
    # Grid and axis labels.
    steps = 5
    for k in range(steps + 1):
        v = lo + (hi - lo) * k / steps
        out.append(f'<line class="grid" x1="{left}" x2="{w - right}" y1="{y(v):.1f}" y2="{y(v):.1f}"/>')
        out.append(f'<text class="tick" x="{left - 6}" y="{y(v) + 4:.1f}" text-anchor="end">{v:.0%}</text>')
    if ref is not None:
        out.append(f'<line class="ref" x1="{left}" x2="{w - right}" y1="{y(ref):.1f}" y2="{y(ref):.1f}"/>')
        out.append(f'<text class="tick" x="{w - right}" y="{y(ref) - 4:.1f}" text-anchor="end">reference {ref:.1%}</text>')
    pts = []
    for i, r in enumerate(rows):
        cx, cy = x(i), y(r["value"])
        if r.get("game_version_change") and i > 0:
            # Rows before this one were measured on an earlier game version.
            gx = (x(i - 1) + cx) / 2
            out.append(f'<line class="era" x1="{gx:.1f}" x2="{gx:.1f}" y1="{top}" y2="{top + ph}"/>')
            out.append(f'<text class="tick" x="{gx + 4:.1f}" y="{top + 10}">new game version →</text>')
        if r.get("metric_switch") and i > 0:
            # Rows before this one measured the metric as a secondary benchmark.
            sx = (x(i - 1) + cx) / 2
            out.append(f'<line class="switch" x1="{sx:.1f}" x2="{sx:.1f}" y1="{top}" y2="{top + ph}"/>')
            out.append(f'<text class="tick" x="{sx + 4:.1f}" y="{top + 22}">primary metric from here →</text>')
        pts.append(f"{cx:.1f},{cy:.1f}")
        tip = (f'{r["version"]} ({r["date"]}, {r["commit"]})\n{r["change"]}\n'
               f'{r["value"]:.1%} [{r["ci95"][0]:.1%}, {r["ci95"][1]:.1%}], {r["games"]} games'
               + (f'\n{r["notes"]}' if r.get("notes") else ""))
        out.append(f'<g><title>{html.escape(tip)}</title>')
        out.append(f'<line class="ci" x1="{cx:.1f}" x2="{cx:.1f}" y1="{y(r["ci95"][0]):.1f}" y2="{y(r["ci95"][1]):.1f}"/>')
        cls = "pt hollow" if r.get("backfilled") else "pt"
        out.append(f'<circle class="{cls}" cx="{cx:.1f}" cy="{cy:.1f}" r="5"/>')
        out.append(f'<text class="val" x="{cx + 8:.1f}" y="{cy - 6:.1f}">{r["value"]:.1%}</text></g>')
        label = html.escape(r["version"][:34])
        out.append(f'<text class="lab" transform="translate({cx:.1f},{top + ph + 12}) rotate(40)">{label}</text>')
    if len(pts) > 1:
        out.insert(1, f'<polyline class="line" points="{" ".join(pts)}"/>')
    out.append("</svg>")
    table = "".join(
        f'<tr><td>{i + 1}</td><td>{html.escape(r["version"])}</td><td>{html.escape(r["change"])}</td>'
        f'<td class="num">{r["value"]:.1%}</td><td class="num">{r["ci95"][0]:.1%} – {r["ci95"][1]:.1%}</td>'
        f'<td class="num">{r["games"]}</td><td>{"backfilled" if r.get("backfilled") else r["commit"]}</td>'
        f'<td>{html.escape(r.get("notes", ""))}</td></tr>'
        for i, r in enumerate(rows))
    arrow = "higher is better" if higher else "lower is better"
    return (f'<section><h2>{html.escape(title)}</h2><p class="sub">{arrow}; bars are 95% Wilson '
            f'intervals; hollow points are backfilled history; a dotted line marks a game-rules '
            f'change, so points on either side measure different game versions; a dashed line '
            f'marks where the metric became the primary one.</p>{"".join(out)}'
            f'<table><thead><tr><th>#</th><th>AI version</th><th>What changed</th><th>Value</th>'
            f'<th>95% CI</th><th>Games</th><th>Commit</th><th>Notes</th></tr></thead>'
            f'<tbody>{table}</tbody></table></section>')


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ledger", default=str(DIR / "ledger.jsonl"))
    ap.add_argument("--out", default=str(DIR / "chart.html"))
    a = ap.parse_args()
    rows = [json.loads(line) for line in open(a.ledger) if line.strip()]
    # A count metric recorded without an interval (e.g. disbands per game) plots as a point.
    for r in rows:
        r.setdefault("ci95", [r["value"], r["value"]])
    order = list(METRICS) + sorted({r["metric"] for r in rows} - set(METRICS))
    panels = "".join(panel(m, [r for r in rows if r["metric"] == m]) for m in order
                     if any(r["metric"] == m for r in rows))
    page = f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>AI Quality Ledger</title>
<style>
:root {{ --bg:#fbfaf7; --fg:#1d1d1b; --muted:#6b6a66; --grid:#e4e1da; --accent:#2f6fb0; --ref:#b0552f; }}
@media (prefers-color-scheme: dark) {{ :root {{ --bg:#17181a; --fg:#e8e6e1; --muted:#9a978f; --grid:#2c2e31; --accent:#7fb2e5; --ref:#e59a7f; }} }}
body {{ background:var(--bg); color:var(--fg); font:14px/1.45 system-ui, sans-serif; margin:0; padding:16px; }}
main {{ max-width:1000px; margin:0 auto; }}
h1 {{ font-size:22px; margin:8px 0 4px; }} h2 {{ font-size:16px; margin:28px 0 2px; }}
.sub {{ color:var(--muted); margin:0 0 8px; }}
svg {{ width:100%; height:auto; overflow:visible; }}
.grid {{ stroke:var(--grid); }} .era {{ stroke:var(--muted); stroke-dasharray:2 3; }} .switch {{ stroke:var(--ref); stroke-dasharray:6 3; }} .ref {{ stroke:var(--ref); stroke-dasharray:4 4; }}
.tick, .lab {{ fill:var(--muted); font-size:11px; }} .val {{ fill:var(--fg); font-size:11px; }}
.ci {{ stroke:var(--accent); stroke-width:2; }} .pt {{ fill:var(--accent); }}
.hollow {{ fill:var(--bg); stroke:var(--accent); stroke-width:2; }}
.line {{ fill:none; stroke:var(--accent); stroke-opacity:.35; stroke-width:1.5; }}
table {{ border-collapse:collapse; width:100%; font-size:12px; margin-top:8px; display:block; overflow-x:auto; }}
th, td {{ border-bottom:1px solid var(--grid); padding:4px 6px; text-align:left; vertical-align:top; }}
.num {{ text-align:right; white-space:nowrap; }}
</style></head><body><main>
<h1>AI quality: hill-climb ledger</h1>
<p class="sub">Generated from docs/sim-results/ai-quality/ledger.jsonl by tools/ai-quality/chart.py. Hover a point for its details.</p>
{panels}
</main></body></html>
"""
    pathlib.Path(a.out).write_text(page)
    print(f"wrote {a.out} ({len(rows)} rows)")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Plot search quality against compute budget as a phone-readable portrait chart.

    python3 tools/ai-quality/curve.py --spec curve.json --out docs/sim-results/ai-quality/<name>.png

The spec lists panels, each with points {label, budget, run, baseline?}: `run` is a duel run
(JSON or a cloud batch's merged.json.gz), `budget` the rollout turns per decision (0 = plain
Hard). Side A's score and its 95% Wilson interval are drawn against budget (log scale, with 0 at
the left edge); a paired difference against `baseline` is printed under each point when given.
Writes an SVG next to the PNG; the PNG is rendered with headless Chrome when it is installed.
Standard library only (plus Chrome for the PNG).
"""

from __future__ import annotations

import argparse
import html
import json
import math
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import report  # noqa: E402

CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"


def point(p: dict) -> dict:
    run = report.load(p["run"])
    s = report.duel(run)
    out = {"label": p["label"], "budget": p["budget"], "value": s["side_a_score"],
           "ci": s["ci95"], "games": s["games"]}
    if p.get("baseline"):
        out["paired"] = report.paired(run, report.load(p["baseline"]))
    return out


def panel(title: str, ref: float | None, pts: list[dict], w: int, h: int, top: int) -> str:
    left, right, ph = 110, 40, h - 230
    lo = min([q["ci"][0] for q in pts] + ([ref] if ref is not None else []))
    hi = max([q["ci"][1] for q in pts] + ([ref] if ref is not None else []))
    pad = max(0.01, (hi - lo) * 0.12)
    lo, hi = max(0.0, lo - pad), min(1.0, hi + pad)
    budgets = [q["budget"] for q in pts if q["budget"] > 0]
    bmin, bmax = (min(budgets), max(budgets)) if budgets else (1, 10)
    lmin, lmax = math.log10(bmin) - 0.35, math.log10(bmax) + 0.15
    pw = w - left - right

    def x(b: float) -> float:
        lb = math.log10(b) if b > 0 else lmin
        return left + pw * (lb - lmin) / max(1e-9, lmax - lmin)

    def y(v: float) -> float:
        return top + 70 + ph * (1 - (v - lo) / max(1e-9, hi - lo))

    o = [f'<text class="title" x="{left}" y="{top + 40}">{html.escape(title)}</text>']
    for k in range(6):
        v = lo + (hi - lo) * k / 5
        o.append(f'<line class="grid" x1="{left}" x2="{w - right}" y1="{y(v):.1f}" y2="{y(v):.1f}"/>')
        o.append(f'<text class="tick" x="{left - 12}" y="{y(v) + 9:.1f}" text-anchor="end">{v:.1%}</text>')
    if ref is not None:
        o.append(f'<line class="ref" x1="{left}" x2="{w - right}" y1="{y(ref):.1f}" y2="{y(ref):.1f}"/>')
    line = []
    for q in sorted(pts, key=lambda q: q["budget"]):
        cx, cy = x(q["budget"]), y(q["value"])
        line.append(f"{cx:.1f},{cy:.1f}")
        o.append(f'<line class="ci" x1="{cx:.1f}" x2="{cx:.1f}" y1="{y(q["ci"][0]):.1f}" y2="{y(q["ci"][1]):.1f}"/>')
        o.append(f'<circle class="pt" cx="{cx:.1f}" cy="{cy:.1f}" r="11"/>')
        o.append(f'<text class="val" x="{cx + 16:.1f}" y="{cy - 10:.1f}">{q["value"]:.1%}</text>')
        if "paired" in q:
            d = q["paired"]
            o.append(f'<text class="small" x="{cx + 16:.1f}" y="{cy + 22:.1f}">{d["mean_diff"] * 100:+.1f}</text>')
        o.append(f'<text class="lab" transform="translate({cx:.1f},{top + 90 + ph}) rotate(55)">{html.escape(q["label"])} ({q["budget"]})</text>')
    o.insert(0, f'<polyline class="line" points="{" ".join(line)}"/>')
    return "".join(o)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--spec", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    spec = json.load(open(a.spec))
    w, ph = 1080, 820
    panels = spec["panels"]
    h = 260 + ph * len(panels) + 120
    body = [f'<text class="h1" x="40" y="70">{html.escape(spec["title"])}</text>']
    for j, line in enumerate(spec.get("subtitle", "").split("\n")):
        body.append(f'<text class="sub" x="40" y="{115 + 34 * j}">{html.escape(line)}</text>')
    for i, p in enumerate(panels):
        pts = [point(q) for q in p["points"]]
        body.append(panel(p["title"], p.get("ref"), pts, w, ph, 170 + i * ph))
    for j, line in enumerate(spec.get("footer", "").split("\n")):
        body.append(f'<text class="sub" x="40" y="{h - 90 + 34 * j}">{html.escape(line)}</text>')
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">'
           '<style>text{font-family:Helvetica,Arial,sans-serif;fill:#1d1d1b}'
           '.h1{font-size:44px;font-weight:700}.sub{font-size:27px;fill:#555}.title{font-size:32px;font-weight:600}'
           '.tick{font-size:26px;fill:#666}.lab{font-size:24px;fill:#444}.val{font-size:28px;font-weight:600}'
           '.small{font-size:24px;fill:#2f6fb0}.grid{stroke:#e4e1da;stroke-width:2}'
           '.ref{stroke:#b0552f;stroke-dasharray:10 8;stroke-width:3}.ci{stroke:#2f6fb0;stroke-width:5}'
           '.pt{fill:#2f6fb0}.line{fill:none;stroke:#2f6fb0;stroke-opacity:.35;stroke-width:4}</style>'
           f'<rect width="{w}" height="{h}" fill="#fbfaf7"/>{"".join(body)}</svg>')
    out = pathlib.Path(a.out)
    svg_path = out.with_suffix(".svg")
    svg_path.write_text(svg)
    print(f"wrote {svg_path}")
    if out.suffix == ".png" and pathlib.Path(CHROME).exists():
        import tempfile
        profile = tempfile.mkdtemp(prefix="curve-chrome-")
        subprocess.run([CHROME, "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check", f"--user-data-dir={profile}",
                        f"--screenshot={out.resolve()}",
                        f"--window-size={w},{h}", "--hide-scrollbars", svg_path.resolve().as_uri()],
                       check=True, capture_output=True)
        print(f"wrote {out}")


if __name__ == "__main__":
    main()

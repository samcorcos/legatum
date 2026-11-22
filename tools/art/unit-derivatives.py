#!/usr/bin/env python3
"""Derived sizes of a painted unit miniature (app/src/assets/units), from its 256 master and its source raster.

  python3 tools/art/unit-derivatives.py [--panel] [--card] <slug>...   (no slugs: every unit with a -256.png)

--panel  <slug>-panel.png: 512 x 512, the source figure fitted into 18..494, bottom-aligned at 494, centred (the unit panel).
--card   <slug>-card.png: 64 x 64, a build-card thumbnail cropped to the figure (UI review 2026-10-11: tall figures drawn
         whole in a 30 px card slot were about 24 px tall). The crop is a square around the figure's upper body: the base is
         cut away, so a standing man fills the slot; wide subjects are cut to a square about the figure's mass.
Needs Pillow. A `<slug>-source.png` kept outside git is fetched first (tools/art/fetch-sources.py, LEGATUM_ART_SOURCES_URL).
"""
import argparse
import glob
import os
import sys
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import artsources

UNITS = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'app', 'src', 'assets', 'units')


def fit(img, box, size, bottom=True):
    a = img.getchannel('A').getbbox()
    fig = img.crop(a)
    k = min(box / fig.width, box / fig.height)
    fig = fig.resize((max(1, round(fig.width * k)), max(1, round(fig.height * k))), Image.LANCZOS)
    out = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    m = (size - box) // 2
    out.alpha_composite(fig, ((size - fig.width) // 2, (m + box - fig.height) if bottom else (size - fig.height) // 2))
    return out


def panel(slug):
    src = os.path.join(UNITS, f'{slug}-source.png')
    # An archived source (tools/art/sources-manifest.json) is fetched when missing; a unit with no source yet uses its 256.
    if not os.path.exists(src) and artsources.rel(src) in artsources.manifest_entries():
        artsources.ensure_or_exit(src)
    img = Image.open(src if os.path.exists(src) else os.path.join(UNITS, f'{slug}-256.png')).convert('RGBA')
    fit(img, 476, 512).save(os.path.join(UNITS, f'{slug}-panel.png'), optimize=True)


def card(slug):
    img = Image.open(os.path.join(UNITS, f'{slug}-256.png')).convert('RGBA')
    x0, y0, x1, y1 = img.getchannel('A').getbbox()
    w, h = x1 - x0, y1 - y0
    # Keep the top 80% of the figure (the stone base and feet go), as a square around its middle.
    keep = h * 0.8
    # Wide subjects (a pike fan, a gun on a rest, a galloping horse) are cut to a square about the figure's mass rather
    # than shrunk to their full width, so the body, not the weapon tip, fills the slot.
    side = max(keep, min(w, keep * 1.2)) * 1.04
    a = img.getchannel('A').crop((x0, y0, x1, y1))
    cols = [sum(a.crop((i, 0, i + 1, h)).getdata()) for i in range(w)]
    tot = sum(cols) or 1
    cx = x0 + sum(i * c for i, c in enumerate(cols)) / tot
    # A blocky subject (a tank hull) would fill the square edge to edge: widen the cut until at least 32% is clear, so the
    # card reads as a figure on the card and not as a filled box.
    while True:
        cxs = min(max(cx, x0 + side / 2), x1 - side / 2) if side < w else (x0 + x1) / 2
        top = y0 - (side - keep) / 2 if side > keep else y0
        crop = img.crop((round(cxs - side / 2), round(top), round(cxs + side / 2), round(top + side)))
        out = crop.resize((64, 64), Image.LANCZOS)
        clear = sum(1 for v in out.getchannel('A').getdata() if v == 0) / (64 * 64)
        if clear >= 0.32 or side >= max(w, h) * 1.3:
            break
        side *= 1.05
    # A one pixel clear border, so the corners are transparent however the figure is cut.
    px = out.load()
    for i in range(64):
        for p in ((i, 0), (i, 63), (0, i), (63, i)):
            r, g, b, _ = px[p]
            px[p] = (r, g, b, 0)
    out.save(os.path.join(UNITS, f'{slug}-card.png'), optimize=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--panel', action='store_true')
    ap.add_argument('--card', action='store_true')
    ap.add_argument('slugs', nargs='*')
    a = ap.parse_args()
    slugs = a.slugs or sorted(os.path.basename(p)[:-8] for p in glob.glob(os.path.join(UNITS, '*-256.png')))
    for s in slugs:
        if a.panel:
            panel(s)
        if a.card:
            card(s)
        print(s)


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Mechanical unit-art audit: finds copies and recolours across every unit asset (legatum-art, "A recolour is a placeholder").

    python3 tools/art/unit-audit.py [--top N] [--json OUT]       # needs numpy and Pillow

For every pair of painted miniatures (`app/src/assets/units/*-256.png`) it compares the figure with colour taken out:
the alpha mask (cropped to its bounding box, fitted into a square) as IoU, and the greyscale painting after histogram
matching (so a tint or a re-grade cannot hide a copy) as normalised correlation of the pixels and of the edge map,
inside the pair's shared mask. A recolour of one painting scores near 1.0 on all three; two different paintings on
the same stone base share a mask but not the edges. It also hashes every file (a byte-identical PNG, panel or source
is a copy) and compares icons and roundels by their drawing with comments, titles and whitespace removed.
"""
import argparse
import hashlib
import json
import os
import re
import sys
from itertools import combinations

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import artsources  # noqa: E402
UNITS = os.path.join(HERE, '..', '..', 'app', 'src', 'assets', 'units')
ICONS = os.path.join(HERE, '..', '..', 'app', 'src', 'assets', 'icons')
N = 128


def prep(path):
    im = Image.open(path).convert('RGBA')
    a = im.getchannel('A')
    box = a.getbbox()
    im = im.crop(box)
    k = N / max(im.size)
    im = im.resize((max(1, round(im.width * k)), max(1, round(im.height * k))), Image.LANCZOS)
    out = Image.new('RGBA', (N, N), (0, 0, 0, 0))
    out.alpha_composite(im, ((N - im.width) // 2, (N - im.height) // 2))
    arr = np.asarray(out).astype(np.float64)
    mask = arr[..., 3] > 128
    grey = arr[..., 0] * 0.299 + arr[..., 1] * 0.587 + arr[..., 2] * 0.114
    return mask, grey


def match(src, ref, m):
    """Histogram-match src's greyscale to ref's over the mask, so tint and grade do not matter."""
    s, r = src[m], ref[m]
    if len(s) < 50:
        return src
    order = np.argsort(s)
    out = np.empty_like(s)
    out[order] = np.sort(r)
    res = src.copy()
    res[m] = out
    return res


def ncc(a, b):
    a, b = a - a.mean(), b - b.mean()
    d = np.sqrt((a * a).sum() * (b * b).sum())
    return float((a * b).sum() / d) if d > 0 else 0.0


def edges(g):
    gx = np.zeros_like(g)
    gy = np.zeros_like(g)
    gx[:, 1:-1] = g[:, 2:] - g[:, :-2]
    gy[1:-1, :] = g[2:, :] - g[:-2, :]
    return np.hypot(gx, gy)


def compare(ma, ga, mb, gb):
    inter, union = (ma & mb), (ma | mb)
    iou = float(inter.sum() / max(1, union.sum()))
    if inter.sum() < 200:
        return iou, 0.0, 0.0
    gam = match(ga, gb, inter)
    return iou, ncc(gam[inter], gb[inter]), ncc(edges(gam)[inter], edges(gb)[inter])


def norm_svg(text):
    t = re.sub(r'<!--.*?-->', '', text, flags=re.S)
    t = re.sub(r'<title>.*?</title>', '', t, flags=re.S)
    t = re.sub(r'aria-label="[^"]*"', '', t)
    return re.sub(r'\s+', ' ', t).strip()


def sha(path):
    return hashlib.sha256(open(path, 'rb').read()).hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--top', type=int, default=40)
    ap.add_argument('--json')
    args = ap.parse_args()
    slugs = sorted(f[:-8] for f in os.listdir(UNITS) if f.endswith('-256.png'))
    data = {s: prep(os.path.join(UNITS, f'{s}-256.png')) for s in slugs}
    rows = []
    for a, b in combinations(slugs, 2):
        iou, c, e = compare(*data[a], *data[b])
        rows.append({'a': a, 'b': b, 'iou': round(iou, 3), 'ncc': round(c, 3), 'edge': round(e, 3)})
    rows.sort(key=lambda r: -(r['ncc'] + r['edge']))
    print('Miniature pairs by structure (ncc = histogram-matched greyscale, edge = edge-map correlation, iou = mask):')
    for r in rows[:args.top]:
        print(f"  {r['a']:24} {r['b']:24} iou {r['iou']:.2f}  ncc {r['ncc']:.2f}  edge {r['edge']:.2f}")
    suffixes = ['256.png', '64.png', '48.png', 'card.png', 'panel.png', 'source.png', 'roundel.svg']
    print('\nByte-identical assets across slugs (same suffix):')
    dup = {}
    archived = artsources.manifest_entries()
    for s in slugs:
        for suf in suffixes:
            p = os.path.join(UNITS, f'{s}-{suf}')
            if not os.path.exists(p) and artsources.rel(p) in archived:
                # An archived source needs no download: the manifest already holds its sha256.
                dup.setdefault((suf, archived[artsources.rel(p)]['sha256']), []).append(s)
            elif os.path.exists(p):
                key = hashlib.sha256(norm_svg(open(p).read()).encode()).hexdigest() if suf == 'roundel.svg' else sha(p)
                dup.setdefault((suf, key), []).append(s)
    for (suf, _), group in dup.items():
        if len(group) > 1:
            print(f'  {suf}: {", ".join(group)}')
    print('\nIcons with the same drawing (comments, titles and whitespace removed):')
    icons = {}
    for f in sorted(os.listdir(ICONS)):
        if f.startswith('unit-') and f.endswith('.svg'):
            icons.setdefault(norm_svg(open(os.path.join(ICONS, f)).read()), []).append(f)
    for group in icons.values():
        if len(group) > 1:
            print('  ' + ', '.join(group))
    if args.json:
        json.dump(rows, open(args.json, 'w'))


if __name__ == '__main__':
    sys.exit(main())

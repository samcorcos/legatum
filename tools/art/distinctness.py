#!/usr/bin/env python3
"""Pairwise lookalike check for every Legatum icon, medallion and miniature set (standard library only).

An asset must stay distinct from every other asset in its set at the size it is drawn, in greyscale and for
colour-blind players (legatum-art: "every icon must be distinct in its set"). The renderer
(tools/art/render-sets.mjs, headless Chromium) draws each asset at its display size on parchment; this script scores
each pair. Two kinds of set:

  colour  painted or multi-colour art (resources, miniatures, roundels, cities, wonders, portraits). The picture is
          lightly blurred (a one pixel shift is forgiven) and compared in CIE Lab over every pixel where either
          picture has content; the gap is the mean Delta E, taken in the worst way of seeing the pair: normal sight,
          protanopia, deuteranopia, tritanopia, or greyscale (lightness only). score = 1 - gap / 40.
  mono    single-colour icons (the 24-grid glyph sets, seals, emblems). Their ink is one colour, so only shape can
          differ: score = 0.6 * ssim + 0.4 * shape, ssim being structural similarity of the greyscale pictures over
          the windows where either has content and shape the silhouette intersection over union.

  gist    (painted sets with silhouettes) how alike the two figures' silhouettes are as a coarse 12 x 12 picture: what two
          upright spear-and-shield figures share at play size when colour and detail differ; a set may flag on it
          alone ("gist_threshold")

A pair at or above its set's threshold (tools/art/distinctness.json) is "too similar" and fails --check unless it is
listed in "accepted" with a reason. Thresholds are calibrated in that file's "note".

  python3 tools/art/distinctness.py [--render] [--cache DIR] [--check] [--report FILE] [--sheet FILE] [--top N]
"""
import argparse
import json
import math
import os
import struct
import subprocess
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
DEFAULT_CACHE = os.path.join(HERE, '.cache')
PAPER = (242, 234, 216)  # tokens.css --panel, the colour every asset sits on in the game's panels and map

# ---------------------------------------------------------------- PNG in and out


def read_png(path):
    """8-bit RGBA, non-interlaced PNG -> (width, height, list of rows of (r, g, b, a))."""
    with open(path, 'rb') as f:
        buf = f.read()
    pos, width, height, idat = 8, 0, 0, []
    while pos < len(buf):
        (length,) = struct.unpack('>I', buf[pos:pos + 4])
        kind = buf[pos + 4:pos + 8]
        data = buf[pos + 8:pos + 8 + length]
        if kind == b'IHDR':
            width, height = struct.unpack('>II', data[:8])
            if data[8] != 8 or data[9] != 6 or data[12] != 0:
                raise ValueError('%s: expected 8-bit RGBA, not interlaced' % path)
        elif kind == b'IDAT':
            idat.append(data)
        pos += 12 + length
    raw = zlib.decompress(b''.join(idat))
    stride = width * 4
    prev = bytearray(stride)
    rows = []
    for y in range(height):
        base = y * (stride + 1)
        ft = raw[base]
        line = bytearray(raw[base + 1:base + 1 + stride])
        if ft == 1:
            for x in range(4, stride):
                line[x] = (line[x] + line[x - 4]) & 255
        elif ft == 2:
            for x in range(stride):
                line[x] = (line[x] + prev[x]) & 255
        elif ft == 3:
            for x in range(stride):
                a = line[x - 4] if x >= 4 else 0
                line[x] = (line[x] + ((a + prev[x]) >> 1)) & 255
        elif ft == 4:
            for x in range(stride):
                a = line[x - 4] if x >= 4 else 0
                b = prev[x]
                c = prev[x - 4] if x >= 4 else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pred = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[x] = (line[x] + pred) & 255
        rows.append(line)
        prev = line
    return width, height, [[tuple(r[i:i + 4]) for i in range(0, stride, 4)] for r in rows]


def write_png(path, width, height, rows):
    """rows: list of lists of (r, g, b) -> an opaque PNG."""
    raw = b''.join(b'\x00' + bytes(v for px in row for v in px) for row in rows)

    def chunk(kind, data):
        body = kind + data
        return struct.pack('>I', len(data)) + body + struct.pack('>I', zlib.crc32(body) & 0xffffffff)
    with open(path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
                + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b''))


# ---------------------------------------------------------------- the picture of one asset


def crop_rows(rows, frac):
    n = len(rows)
    m = int(round(n * frac))
    return [r[m:len(r) - m] for r in rows[m:n - m]]


class Pic(object):
    """An asset composited on parchment: greyscale, rgb and alpha planes (alpha as 0..1)."""

    def __init__(self, rows):
        self.h = len(rows)
        self.w = len(rows[0])
        self.alpha = [[p[3] / 255.0 for p in r] for r in rows]
        self.rgb = []
        for r in rows:
            out = []
            for (R, G, B, A) in r:
                k = A / 255.0
                out.append((R * k + PAPER[0] * (1 - k), G * k + PAPER[1] * (1 - k), B * k + PAPER[2] * (1 - k)))
            self.rgb.append(out)
        self.gray = [[0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2] for p in r] for r in self.rgb]
        self.blur = blur3(self.gray)
        self.ablur = blur3(self.alpha, 1.0)
        self.mask = [[1 if a > 0.5 else 0 for a in r] for r in self.alpha]
        self.gist = None
        self.cover = sum(sum(r) for r in self.mask) / float(self.w * self.h)
        self.has_silhouette = 0.02 < self.cover < 0.9
        self.gist = gist(self)


def blur3(plane, scale=1.0):
    h, w = len(plane), len(plane[0])
    out = [[0.0] * w for _ in range(h)]
    k = ((1, 2, 1), (2, 4, 2), (1, 2, 1))
    for y in range(h):
        for x in range(w):
            s = t = 0.0
            for dy in (-1, 0, 1):
                yy = y + dy
                if yy < 0 or yy >= h:
                    continue
                for dx in (-1, 0, 1):
                    xx = x + dx
                    if xx < 0 or xx >= w:
                        continue
                    wgt = k[dy + 1][dx + 1]
                    s += plane[yy][xx] * wgt
                    t += wgt
            out[y][x] = s / t * scale
    return out


# ---------------------------------------------------------------- the metrics


def integral(plane):
    h, w = len(plane), len(plane[0])
    t = [[0.0] * (w + 1) for _ in range(h + 1)]
    for y in range(h):
        run = 0.0
        row, up, cur = plane[y], t[y], t[y + 1]
        for x in range(w):
            run += row[x]
            cur[x + 1] = up[x + 1] + run
    return t


def box(t, x, y, win):
    return t[y + win][x + win] - t[y][x + win] - t[y + win][x] + t[y][x]


class Prep(object):
    """Per-picture integral images, built once and reused for every pair."""

    def __init__(self, pic):
        self.pic = pic
        g = pic.blur
        self.i1 = integral(g)
        self.i2 = integral([[v * v for v in r] for r in g])
        self.ia = integral(pic.ablur)


def ssim(pa, pb):
    a, b = pa.pic, pb.pic
    n = min(a.w, a.h)
    win = max(3, int(round(n / 8.0)))
    if n < win:
        return 0.0
    c1, c2 = (0.01 * 255) ** 2, (0.03 * 255) ** 2
    prod = integral([[a.blur[y][x] * b.blur[y][x] for x in range(a.w)] for y in range(a.h)])
    area = float(win * win)
    total = weight = 0.0
    for y in range(a.h - win + 1):
        for x in range(a.w - win + 1):
            # Only windows where either picture has something in them (two blank corners are not "alike").
            content = max(box(pa.ia, x, y, win), box(pb.ia, x, y, win)) / area
            if content < 0.04:
                continue
            ma, mb = box(pa.i1, x, y, win) / area, box(pb.i1, x, y, win) / area
            va = max(0.0, box(pa.i2, x, y, win) / area - ma * ma)
            vb = max(0.0, box(pb.i2, x, y, win) / area - mb * mb)
            cov = box(prod, x, y, win) / area - ma * mb
            s = ((2 * ma * mb + c1) * (2 * cov + c2)) / ((ma * ma + mb * mb + c1) * (va + vb + c2))
            total += s * content
            weight += content
    return max(0.0, total / weight) if weight else 0.0


def iou(a, b):
    inter = union = 0
    for ra, rb in zip(a.mask, b.mask):
        for x, y in zip(ra, rb):
            if x or y:
                union += 1
                if x and y:
                    inter += 1
    return inter / float(union) if union else 0.0


# Machado, Oliveira and Fernandes 2009, severity 1.0, applied in linear light.
CVD = {
    'protan': ((0.152286, 1.052583, -0.204868), (0.114503, 0.786281, 0.099216), (-0.003882, -0.048116, 1.051998)),
    'deutan': ((0.367322, 0.860646, -0.227968), (0.280085, 0.672501, 0.047413), (-0.011820, 0.042940, 0.968881)),
    'tritan': ((1.255528, -0.076749, -0.178779), (-0.078411, 0.930809, 0.147602), (0.004733, 0.691367, 0.303900)),
}
_LIN = [((i / 255.0) / 12.92 if i / 255.0 <= 0.04045 else (((i / 255.0) + 0.055) / 1.055) ** 2.4) for i in range(256)]


def _to_srgb(v):
    v = min(1.0, max(0.0, v))
    return 255 * (v * 12.92 if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055)


def simulate(rgb, mode):
    if mode == 'normal':
        return rgb
    if mode == 'gray':
        g = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2]
        return (g, g, g)
    m = CVD[mode]
    lin = [_LIN[int(round(min(255, max(0, c))))] for c in rgb]
    return tuple(_to_srgb(sum(m[i][j] * lin[j] for j in range(3))) for i in range(3))


def hsv(rgb):
    r, g, b = [c / 255.0 for c in rgb]
    mx, mn = max(r, g, b), min(r, g, b)
    d = mx - mn
    if d == 0:
        h = 0.0
    elif mx == r:
        h = ((g - b) / d) % 6
    elif mx == g:
        h = (b - r) / d + 2
    else:
        h = (r - g) / d + 4
    return h * 60.0, (d / mx if mx else 0.0), mx


def lab(rgb):
    """sRGB (0..255) -> CIE Lab (D65)."""
    r, g, b = [_LIN[int(round(min(255, max(0, c))))] for c in rgb]
    x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047
    y = 0.2126 * r + 0.7152 * g + 0.0722 * b
    z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883

    def f(t):
        return t ** (1 / 3.0) if t > 0.008856 else 7.787 * t + 16 / 116.0
    fx, fy, fz = f(x), f(y), f(z)
    return 116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)


def lab_plane(pic, mode):
    """Blurred Lab planes (L, a, b) of the picture as seen in `mode`, flattened row-major."""
    L, A, B = [], [], []
    for y in range(pic.h):
        for x in range(pic.w):
            l, a, b = lab(simulate(pic.rgb[y][x], mode))
            L.append(l)
            A.append(a)
            B.append(b)

    def blur(flat):
        rows = [flat[i * pic.w:(i + 1) * pic.w] for i in range(pic.h)]
        return [v for r in blur3(rows) for v in r]
    return blur(L), blur(A), blur(B)


INK_L = 48.0  # a pixel darker than this (Lab L) is ink


def lab_distance(pa, pb, la, lb, ink=False):
    """Mean Delta E over the pixels where either picture has content (with ink=True: where either is inked, which
    is how a roundel's silhouette is compared without its shared plate and disc), plus the mean |dL| alone
    (greyscale sight)."""
    tot = tl = n = 0.0
    w = pa.pic.w
    for i in range(len(la[0])):
        y, x = divmod(i, w)
        if max(pa.pic.alpha[y][x], pb.pic.alpha[y][x]) < 0.05:
            continue
        if ink and min(la[0][i], lb[0][i]) > INK_L:
            continue
        dl = la[0][i] - lb[0][i]
        da = la[1][i] - lb[1][i]
        db = la[2][i] - lb[2][i]
        tot += math.sqrt(dl * dl + da * da + db * db)
        tl += abs(dl)
        n += 1
    return (tot / n, tl / n) if n else (0.0, 0.0)


def gist(pic):
    """The figure's silhouette as a coarse picture: alpha cropped to its bounding box and resampled to a 12 x 12 coverage
    grid, with the box's aspect ratio. This is what two upright figures with a vertical spear share at play size, even
    when their colours and fine detail differ. Returns (aspect, grid) or None when there is no silhouette."""
    if not pic.has_silhouette:
        return None
    ys = [y for y in range(pic.h) if any(a > 0.5 for a in pic.alpha[y])]
    xs = [x for x in range(pic.w) if any(pic.alpha[y][x] > 0.5 for y in range(pic.h))]
    if not ys or not xs:
        return None
    y0, y1, x0, x1 = ys[0], ys[-1] + 1, xs[0], xs[-1] + 1
    bw, bh = x1 - x0, y1 - y0
    n = 12
    grid = []
    for gy in range(n):
        row = []
        for gx in range(n):
            ya, yb = y0 + bh * gy / float(n), y0 + bh * (gy + 1) / float(n)
            xa, xb = x0 + bw * gx / float(n), x0 + bw * (gx + 1) / float(n)
            tot = cnt = 0
            for y in range(int(ya), max(int(ya) + 1, int(math.ceil(yb)))):
                for x in range(int(xa), max(int(xa) + 1, int(math.ceil(xb)))):
                    if 0 <= y < pic.h and 0 <= x < pic.w:
                        tot += 1 if pic.alpha[y][x] > 0.5 else 0
                        cnt += 1
            row.append(tot / float(cnt) if cnt else 0.0)
        grid.append(row)
    return bw / float(bh), grid


def gist_similarity(ga, gb):
    """1 = the same silhouette gist: aspect ratio agreement times one minus the mean coverage difference."""
    if ga is None or gb is None:
        return None
    ar = min(ga[0], gb[0]) / max(ga[0], gb[0])
    diff = sum(abs(a - b) for ra, rb in zip(ga[1], gb[1]) for a, b in zip(ra, rb)) / 144.0
    return ar * (1 - diff)


MODES = ('normal', 'protan', 'deutan', 'tritan', 'gray')
GRAY_SCALE = 1.35


def sight_gap(pa, pb, la, lb, ink=False):
    """The smallest mean Delta E between the two pictures over every way of seeing them: normal, protanopia,
    deuteranopia, tritanopia, and greyscale (lightness only, scaled by GRAY_SCALE because one channel always reads
    smaller than three). Returns (gap, mode of the smallest)."""
    best, which = 1e9, 'normal'
    for m in MODES:
        d, dl = lab_distance(pa, pb, la[m], lb[m], ink)
        v = dl * GRAY_SCALE if m == 'gray' else d
        if v < best:
            best, which = v, m
    return best, which


GAP_SCALE = 40.0  # a mean Delta E this large between two pictures is plainly different: similarity 0


def score_pair(pa, pb, la, lb, kind, ink=False):
    """-> dict of ssim, shape (or None), gap, gap_mode, score. Painted and multi-colour assets are scored on the
    gap alone (their fine texture makes SSIM meaningless at 24 px); single-colour icons on shape: SSIM and IoU."""
    s = ssim(pa, pb)
    sh = iou(pa.pic, pb.pic) if (pa.pic.has_silhouette and pb.pic.has_silhouette) else None
    if kind == 'mono':
        return {'ssim': s, 'shape': sh, 'gap': None, 'gap_mode': None, 'gist': None, 'score': 0.6 * s + 0.4 * sh if sh is not None else s}
    gap, mode = sight_gap(pa, pb, la, lb, ink)
    return {'ssim': s, 'shape': sh, 'gap': gap, 'gap_mode': mode, 'score': max(0.0, 1.0 - gap / GAP_SCALE),
            'gist': gist_similarity(pa.pic.gist, pb.pic.gist)}


# ---------------------------------------------------------------- sets


def load_set(cache, name, spec, size):
    ids = spec['ids']
    inner = 0.12 if spec['crop'] == 'inner' else 0.0
    d = os.path.join(cache, '%s@%d' % (name, size))
    pics = {}
    for i in ids:
        w, h, rows = read_png(os.path.join(d, i + '.png'))
        if inner:
            rows = crop_rows(rows, inner)
        pics[i] = Pic(rows)
    return pics


def compare_set(cache, name, spec, size):
    pics = load_set(cache, name, spec, size)
    preps = dict((i, Prep(p)) for i, p in pics.items())
    labs = {}
    if spec['kind'] == 'colour':
        labs = dict((i, dict((m, lab_plane(p, m)) for m in MODES)) for i, p in pics.items())
    ids = spec['ids']
    out = []
    for x in range(len(ids)):
        for y in range(x + 1, len(ids)):
            a, b = ids[x], ids[y]
            r = score_pair(preps[a], preps[b], labs.get(a), labs.get(b), spec['kind'], spec.get('focus') == 'ink')
            r.update({'set': name, 'size': size, 'a': a, 'b': b})
            out.append(r)
    return out


def threshold_for(cfg, name, kind):
    return cfg.get('sets', {}).get(name, {}).get('threshold', cfg['threshold'][kind])


def same_group(cfg, r):
    """Roundels carry the unit's class, not its era: two roundels of one class may look alike on purpose."""
    for g in cfg.get('groups', {}).get(r['set'], []):
        sa = set(r['a'].split('+'))
        sb = set(r['b'].split('+'))
        if sa <= set(g['members']) and sb <= set(g['members']):
            return {'why': 'same class by design: ' + g['why']}
    return None


def accepted(cfg, r):
    g = same_group(cfg, r)
    if g:
        return g
    for e in cfg.get('accepted', []):
        if e['set'] == r['set'] and sorted([e['a'], e['b']]) == sorted([r['a'], r['b']]):
            return e
    return None


def run(cache, cfg, only=None, reuse=False):
    manifest = json.load(open(os.path.join(cache, 'manifest.json')))
    rows = []
    saved = os.path.join(cache, 'results.json')
    for name, spec in manifest['sets'].items():
        if only and name not in only:
            continue
        for size in spec['sizes']:
            for r in (json.load(open(saved)).get('%s@%d' % (name, size)) if reuse and os.path.exists(saved) else compare_set(cache, name, spec, size)):
                r['kind'] = spec['kind']
                r['threshold'] = threshold_for(cfg, name, spec['kind'])
                gt = cfg.get('sets', {}).get(name, {}).get('gist_threshold')
                r['gist_threshold'] = gt
                r['flag'] = r['score'] >= r['threshold'] or bool(gt and r.get('gist') is not None and r['gist'] >= gt)
                r['accepted'] = accepted(cfg, r) if r['flag'] else None
                r['advisory'] = bool(cfg.get('sets', {}).get(name, {}).get('advisory'))
                rows.append(r)
    if not only:
        by = {}
        for r in rows:
            by.setdefault('%s@%d' % (r['set'], r['size']), []).append(dict((k, r[k]) for k in ('set', 'size', 'a', 'b', 'ssim', 'shape', 'gap', 'gap_mode', 'score', 'gist')))
        json.dump(by, open(saved, 'w'))
    return manifest, rows


def fmt1(v):
    return '  -  ' if v is None else '%.1f' % v


def fmt(v):
    return '  -  ' if v is None else '%.2f' % v


def line(r):
    mark = ' (accepted: %s)' % r['accepted']['why'] if r['accepted'] else ''
    return '%-20s %3dpx  %-26s %-26s score %.2f  ssim %s  shape %s  gist %s  gap %s%s%s' % (
        r['set'], r['size'], r['a'], r['b'], r['score'], fmt(r['ssim']), fmt(r['shape']), fmt(r.get('gist')), fmt1(r['gap']),
        (' [' + r['gap_mode'] + ']') if r['gap_mode'] else '', mark)


def write_report(path, manifest, rows, cfg):
    flagged = [r for r in rows if r['flag']]
    out = ['# Lookalike report', '',
           'Generated by `python3 tools/art/distinctness.py --render --report <file>` (see `tools/art/distinctness.json` for the '
           'thresholds and how they were calibrated). Score: 0 = nothing alike, 1 = the same picture; a pair at or over its '
           "set's threshold is flagged.", '',
           '| Set | Assets | Sizes (px) | Threshold | Pairs | Flagged | Accepted |', '| --- | --- | --- | --- | --- | --- | --- |']
    for name, spec in manifest['sets'].items():
        mine = [r for r in rows if r['set'] == name]
        npairs = len(spec['ids']) * (len(spec['ids']) - 1) // 2
        fl = [r for r in mine if r['flag']]
        out.append('| %s | %d | %s | %.2f | %d | %d | %d |' % (name, len(spec['ids']), ', '.join(map(str, spec['sizes'])),
                                                              threshold_for(cfg, name, spec['kind']), npairs, len(fl),
                                                              len([r for r in fl if r['accepted']])))
    out += ['', '## Flagged pairs', '', '| Set | Size | A | B | Score | SSIM | Shape | Gap (dE) | Verdict |', '| --- | --- | --- | --- | --- | --- | --- | --- | --- |']
    for r in sorted(flagged, key=lambda r: -r['score']):
        out.append('| %s | %d | %s | %s | %.2f | %s | %s | %s | %s |' % (
            r['set'], r['size'], r['a'], r['b'], r['score'], fmt(r['ssim']), fmt(r['shape']), fmt1(r['gap']),
            ('accepted: ' + r['accepted']['why']) if r['accepted'] else ('advisory (set is report-only)' if r['advisory'] else 'FAILS')))
    if not flagged:
        out.append('| none | | | | | | | | |')
    watched = cfg.get('watch', {})
    if watched:
        out += ['', '## Watch lists', '', 'Sibling groups named in tools/art/distinctness.json: every pair is listed whether or not it is flagged.', '']
        for name, groups in watched.items():
            for g in groups:
                out += ['### %s: %s' % (name, g['why']), '', '| A | B | Size | Score | Verdict |', '| --- | --- | --- | --- | --- |']
                for r in sorted([r for r in rows if r['set'] == name and r['a'] in g['members'] and r['b'] in g['members']], key=lambda r: -r['score']):
                    out.append('| %s | %s | %d | %.2f | %s |' % (r['a'], r['b'], r['size'], r['score'], 'flagged' if r['flag'] else 'ok'))
                out.append('')
    with open(path, 'w') as f:
        f.write('\n'.join(out) + '\n')


def write_sheet(path, cache, rows, limit):
    """One row per flagged pair, worst first: the two renders at 4x on parchment, side by side."""
    scale, gap = 4, 8
    picks = sorted([r for r in rows if r['flag']], key=lambda r: -r['score'])[:limit]
    if not picks:
        return
    imgs = []
    for n, r in enumerate(picks):
        print('sheet row %d: %s %dpx  %s  vs  %s  (score %.2f)' % (n, r['set'], r['size'], r['a'], r['b'], r['score']))
        ims = []
        for i in (r['a'], r['b']):
            w, h, px = read_png(os.path.join(cache, '%s@%d' % (r['set'], r['size']), i + '.png'))
            ims.append(Pic(px).rgb)
        imgs.append(ims)
    cell = max(len(p[0]) for p in imgs)
    W = (cell * scale) * 2 + gap * 3
    H = (cell * scale + gap) * len(imgs) + gap
    canvas = [[tuple(int(c) for c in (225, 215, 195))] * W for _ in range(H)]
    for n, (a, b) in enumerate(imgs):
        for k, img in enumerate((a, b)):
            ox, oy = gap + k * (cell * scale + gap), gap + n * (cell * scale + gap)
            for y, row in enumerate(img):
                for x, c in enumerate(row):
                    col = tuple(int(v) for v in c)
                    for dy in range(scale):
                        line_ = canvas[oy + y * scale + dy]
                        for dx in range(scale):
                            line_[ox + x * scale + dx] = col
    write_png(path, W, H, canvas)


def write_montage(path, cache, name, size, scale, cols):
    """Every asset of one set at `scale`x on parchment in a grid (row-major, in manifest order), for looking at."""
    manifest = json.load(open(os.path.join(cache, 'manifest.json')))
    ids = manifest['sets'][name]['ids']
    cells = [Pic(read_png(os.path.join(cache, '%s@%d' % (name, size), i + '.png'))[2]).rgb for i in ids]
    cell = size * scale + 6
    rows_n = (len(cells) + cols - 1) // cols
    W, H = cols * cell + 6, rows_n * cell + 6
    canvas = [[(225, 215, 195)] * W for _ in range(H)]
    for n, img in enumerate(cells):
        ox, oy = 6 + (n % cols) * cell, 6 + (n // cols) * cell
        for y, row in enumerate(img):
            for x, c in enumerate(row):
                col = tuple(int(v) for v in c)
                for dy in range(scale):
                    for dx in range(scale):
                        canvas[oy + y * scale + dy][ox + x * scale + dx] = col
    write_png(path, W, H, canvas)
    for n, i in enumerate(ids):
        print('%d,%d  %s' % (n // cols, n % cols, i))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--render', action='store_true', help='render the sets first (node tools/art/render-sets.mjs)')
    ap.add_argument('--cache', default=DEFAULT_CACHE)
    ap.add_argument('--check', action='store_true', help='exit 1 if any pair is too similar and not accepted')
    ap.add_argument('--report', help='write a Markdown report here')
    ap.add_argument('--sheet', help='write a contact sheet of the flagged pairs (worst first) as a PNG')
    ap.add_argument('--top', type=int, default=0, help='print the N most similar pairs of every set, flagged or not')
    ap.add_argument('--reuse', action='store_true', help='reuse the scores from the last full run (re-apply thresholds only)')
    ap.add_argument('--montage', help='NAME@SIZE: write every asset of that set as a grid PNG (needs --out), then exit')
    ap.add_argument('--out')
    ap.add_argument('--scale', type=int, default=6)
    ap.add_argument('--cols', type=int, default=8)
    ap.add_argument('--set', action='append', help='only this set (repeatable)')
    args = ap.parse_args(argv)
    cfg = json.load(open(os.path.join(HERE, 'distinctness.json')))
    if args.render or not os.path.exists(os.path.join(args.cache, 'manifest.json')):
        node = os.environ.get('LEGATUM_NODE', 'node')
        subprocess.check_call([node, os.path.join(HERE, 'render-sets.mjs'), '--out', args.cache])
    if args.montage:
        n, sz = args.montage.split('@')
        write_montage(args.out, args.cache, n, int(sz), args.scale, args.cols)
        return 0
    manifest, rows = run(args.cache, cfg, args.set, args.reuse)
    bad = [r for r in rows if r['flag'] and not r['accepted'] and not r['advisory']]
    if args.top:
        for name in manifest['sets']:
            mine = sorted([r for r in rows if r['set'] == name], key=lambda r: -r['score'])[:args.top]
            print('== %s (threshold %.2f)' % (name, threshold_for(cfg, name, manifest['sets'][name]['kind'])))
            for r in mine:
                print(('  FLAG ' if r['flag'] else '       ') + line(r))
    for r in sorted([r for r in rows if r['flag']], key=lambda r: -r['score']):
        print(('advisory ' if r['advisory'] and not r['accepted'] else 'FAIL ' if not r['accepted'] else 'ok   ') + line(r))
    if args.report:
        write_report(args.report, manifest, rows, cfg)
    if args.sheet:
        write_sheet(args.sheet, args.cache, rows, args.top or 24)
    print('%d sets, %d pair comparisons, %d flagged, %d accepted, %d failing' % (
        len(manifest['sets']), len(rows), len([r for r in rows if r['flag']]), len([r for r in rows if r['accepted']]), len(bad)))
    if args.check and bad:
        print('\nToo similar at display size. Repaint one of each pair (silhouette first, then colour family and value), or'
              ' accept the pair in tools/art/distinctness.json with a reason.', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())

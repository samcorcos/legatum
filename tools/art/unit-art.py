#!/usr/bin/env python3
"""Turn a generated unit miniature on a flat magenta field into the full unit asset set (app/src/assets/units).

    python3 tools/art/unit-art.py RAW.png hunter-killer [--cool N] [--light N] [--edge-key]
    python3 tools/art/unit-art.py --sheet OUT.png slug...      # side-by-side check sheet of the 256 masters on parchment

Writes <slug>-source.png (1254 px, real alpha), -256, -64, -48, then the panel portrait and build-card
crop through `unit-derivatives.py`. The magenta key and despill are `wonder-art.py`'s `key_magenta`,
which also pulls the alpha in a few pixels so the generator's pink glow never shows as a rim.
The roundel (`<slug>-roundel.svg`) is drawn by hand: ink silhouette only, inside the rim.
`--edge-key` keys the field with a global saturated-magenta test only (no median or core filters, a 2 px edge pull-in), for
paintings with blue-violet or red-pink cloth that the stock key (`wonder-art.py` `key_magenta`) punches holes in.
Needs Pillow.
"""
import importlib.util
import os
import sys

from PIL import Image, ImageChops, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
UNITS = os.path.join(HERE, '..', '..', 'app', 'src', 'assets', 'units')


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, os.path.join(HERE, file))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def fit(img, box, size, centre=True):
    x0, y0, x1, y1 = img.getchannel('A').getbbox()
    fig = img.crop((x0, y0, x1, y1))
    k = min(box / fig.width, box / fig.height)
    fig = fig.resize((max(1, round(fig.width * k)), max(1, round(fig.height * k))), Image.LANCZOS)
    out = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    out.alpha_composite(fig, ((size - fig.width) // 2, (size - fig.height) // 2))
    return out


def grade(img, cool, light):
    if not cool and not light:
        return img
    r, g, b, a = img.split()
    r = r.point(lambda v: max(0, min(255, int(v * (1 - cool * 0.08) * (1 + light)))))
    g = g.point(lambda v: max(0, min(255, int(v * (1 + light)))))
    b = b.point(lambda v: max(0, min(255, int(v * (1 + cool * 0.08) * (1 + light)))))
    return Image.merge('RGBA', (r, g, b, a))


def key_edge(raw):
    """Magenta field to alpha by the field's saturation alone, so cloth that merely leans violet or pink survives."""
    r, g, b = raw.convert('RGB').split()
    m = ImageChops.subtract(ImageChops.darker(r, b), g)
    alpha = m.point(lambda v: 255 if v < 50 else max(0, int(255 - (v - 50) * 255 / 40)))
    alpha = alpha.filter(ImageFilter.MinFilter(5)).filter(ImageFilter.GaussianBlur(1.0))
    edge = alpha.point(lambda a: 1 if 0 < a < 255 else 0)
    spill = ImageChops.multiply(m, edge)
    return Image.merge('RGBA', (ImageChops.subtract(r, spill), g, ImageChops.subtract(b, spill), alpha))


def build(raw_path, slug, cool=0.0, light=0.0, edge_key=False):
    wonder = load('wonder_art', 'wonder-art.py')
    raw = Image.open(raw_path)
    keyed = grade(key_edge(raw) if edge_key else wonder.key_magenta(raw), cool, light)
    out = lambda name: os.path.join(UNITS, f'{slug}-{name}.png')
    fit(keyed, 1240, 1254).save(out('source'), optimize=True)
    master = fit(keyed, 232, 256)
    master.save(out('256'), optimize=True)
    fit(master, 56, 64).save(out('64'), optimize=True)
    fit(master, 42, 48).save(out('48'), optimize=True)
    derive = load('unit_derivatives', 'unit-derivatives.py')
    derive.panel(slug)
    derive.card(slug)
    print('wrote', slug)


def sheet(path, slugs):
    size = 256
    sheet_img = Image.new('RGB', (size * len(slugs), size), (214, 203, 178))
    for i, s in enumerate(slugs):
        im = Image.open(os.path.join(UNITS, f'{s}-256.png')).convert('RGBA')
        sheet_img.paste(im, (i * size, 0), im)
    sheet_img.save(path)


if __name__ == '__main__':
    a = sys.argv[1:]
    if a and a[0] == '--sheet':
        sheet(a[1], a[2:])
    else:
        opts = {'--cool': 0.0, '--light': 0.0}
        rest = []
        edge = '--edge-key' in a
        a = [x for x in a if x != '--edge-key']
        i = 0
        while i < len(a):
            if a[i] in opts:
                opts[a[i]] = float(a[i + 1])
                i += 2
            else:
                rest.append(a[i])
                i += 1
        build(rest[0], rest[1], opts['--cool'], opts['--light'], edge)

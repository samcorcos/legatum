#!/usr/bin/env python3
"""Turn a generated unlock miniature on a flat magenta field into app/src/assets/unlocks/<slug>-{256,128,64}.png.

    python3 tools/art/unlock-art.py RAW.png building-supermarket

Keys the magenta field with `unit-art.py`'s edge key, crops to the painting and fits it the way the set sits:
about 238 px wide (the median building's width at 256, the ground patch nearly edge to edge), at most 200 px tall,
centred, its ground resting at y = 226. Not graded: unlock art keeps the painting's own colour (the map palette grade
is for landmarks on the map only, see card-grade.py).
"""
import importlib.util
import sys
from pathlib import Path

from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parents[2]
UNLOCKS = ROOT / "app/src/assets/unlocks"
_spec = importlib.util.spec_from_file_location("wonder_art", ROOT / "tools/art/wonder-art.py")
_wa = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_wa)
_uspec = importlib.util.spec_from_file_location("unit_art", ROOT / "tools/art/unit-art.py")
_ua = importlib.util.module_from_spec(_uspec)
_uspec.loader.exec_module(_ua)

WIDTH, HEIGHT, BOTTOM = 238, 200, 226


def enlarge(raw: Image.Image) -> Image.Image:
    """Crop to the subject (with a magenta margin) and scale it to about 1000 px before keying: the key pulls the
    edge in by a few pixels, which erases thin parts (pavilion columns, a sling) of a subject drawn small."""
    rgb = raw.convert("RGB")
    r, g, b = rgb.split()
    field = ImageChops.subtract(ImageChops.darker(r, b), g).point(lambda m: 255 if m > 60 else 0)
    box = ImageChops.invert(field).getbbox()
    pad = max(box[2] - box[0], box[3] - box[1]) // 8
    box = (max(0, box[0] - pad), max(0, box[1] - pad), min(rgb.width, box[2] + pad), min(rgb.height, box[3] + pad))
    sub = rgb.crop(box)
    k = 1000 / max(sub.size)
    return sub.resize((round(sub.width * k), round(sub.height * k)), Image.LANCZOS) if k > 1.2 else sub


def fit(raw: Image.Image) -> Image.Image:
    # unit-art.py's edge key (saturated-magenta test, 2 px pull-in): the stock key's 9 px erosion cuts thin columns
    # and ropes, leaving roof kiosks and slings floating.
    keyed = _ua.key_edge(enlarge(raw).convert("RGBA"))
    box = keyed.getchannel("A").point(lambda v: 255 if v > 24 else 0).getbbox()
    art = keyed.crop(box)
    scale = min(WIDTH / art.width, HEIGHT / art.height)
    art = art.resize((max(1, round(art.width * scale * 4)), max(1, round(art.height * scale * 4))), Image.LANCZOS)
    canvas = Image.new("RGBA", (1024, 1024), (0, 0, 0, 0))
    canvas.alpha_composite(art, ((1024 - art.width) // 2, BOTTOM * 4 - art.height))
    return canvas


def main(raw, slug):
    master = fit(Image.open(raw))
    for size in (256, 128, 64):
        master.resize((size, size), Image.LANCZOS).save(UNLOCKS / f"{slug}-{size}.png", optimize=True)
    print("wrote", slug)


if __name__ == "__main__":
    main(*sys.argv[1:3])

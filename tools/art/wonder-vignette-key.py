#!/usr/bin/env python3
"""Key the black ground of the natural-wonder vignettes to real alpha.

    python3 tools/art/wonder-vignette-key.py [--check] [FILE ...]     # default: app/public/wonders/*-a-512.png

The eight `-a-512.png` vignettes were generated as a painting cut to a hexagon or ragged shape on a solid
black square. The UI shows them as plain <img> on parchment (the tile card, the discovery card), where the
black square showed as a box. This script removes only the black that is connected to the picture's border:

  1. Flood-fill from every border pixel through near-black pixels (all channels <= KEY). Dark paint inside
     the picture (basalt, forest shadow, deep water) is not connected to the border through near-black, so it
     is never touched. The script reports how the keyed area changes with the threshold: a painting whose
     own darks leak shows a jump, and the run fails rather than eat the picture.
  2. Pull the edge in by one pixel (the antialiased rim is paint mixed with black, which would leave a dark
     fringe on parchment) and feather it, so the cut-out has a soft edge.
  3. Clear the colour of fully transparent pixels to black-transparent.

A file that already has transparent corners is left alone, so the script is safe to run twice.
--check changes nothing and exits 1 if any file still has an opaque corner.
"""

import sys
from collections import deque
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter

ROOT = Path(__file__).resolve().parents[2]
KEY = 14  # a pixel is ground when every channel is at or below this
PROBE = (8, 14, 24)  # thresholds compared to prove the ground is separable from the painting's own darks
MAX_SPREAD = 0.012  # the keyed share of the picture may differ by at most this between the probes
FEATHER = 1.1


def ground(im: Image.Image, key: int) -> Image.Image:
    """L mask (255 = ground): near-black pixels connected to the border."""
    w, h = im.size
    px = im.convert("RGB").load()
    dark = [[max(px[x, y]) <= key for x in range(w)] for y in range(h)]
    seen = [[False] * w for _ in range(h)]
    todo = deque()
    for x in range(w):
        todo.append((x, 0))
        todo.append((x, h - 1))
    for y in range(h):
        todo.append((0, y))
        todo.append((w - 1, y))
    while todo:
        x, y = todo.popleft()
        if x < 0 or y < 0 or x >= w or y >= h or seen[y][x] or not dark[y][x]:
            continue
        seen[y][x] = True
        todo.extend(((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))
    out = Image.new("L", (w, h), 0)
    out.putdata([255 if v else 0 for row in seen for v in row])
    return out


def share(mask: Image.Image) -> float:
    return mask.histogram()[255] / (mask.width * mask.height)


def opaque_corner(im: Image.Image) -> bool:
    a = im.convert("RGBA").getchannel("A")
    w, h = im.size
    return any(a.getpixel(p) > 0 for p in ((0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)))


def key_file(path: Path) -> str:
    im = Image.open(path).convert("RGBA")
    if not opaque_corner(im):
        return "already keyed"
    shares = [share(ground(im, k)) for k in PROBE]
    if max(shares) - min(shares) > MAX_SPREAD:
        raise SystemExit(f"{path.name}: the ground is not separable from the painting's darks (keyed share {shares} at thresholds {PROBE})")
    g = ground(im, KEY)
    # paint = not ground; pull the rim in by a pixel, then feather. MinFilter(3) erodes the paint by one pixel.
    paint = ImageChops.invert(g).filter(ImageFilter.MinFilter(3))
    soft = paint.filter(ImageFilter.GaussianBlur(FEATHER))
    # never make the inside more transparent than the eroded mask's blur, never the outside visible
    alpha = ImageChops.multiply(soft, ImageChops.invert(g))
    out = im.copy()
    out.putalpha(alpha)
    clear = Image.new("RGBA", im.size, (0, 0, 0, 0))
    out = Image.composite(out, clear, alpha.point(lambda v: 255 if v else 0))
    out.save(path, optimize=True)
    return f"keyed {shares[1] * 100:.1f}% of the square (spread {100 * (max(shares) - min(shares)):.2f} points across thresholds {PROBE})"


def main(argv):
    check = "--check" in argv
    files = [Path(a) for a in argv if not a.startswith("--")] or sorted((ROOT / "app/public/wonders").glob("*-a-512.png"))
    bad = 0
    for f in files:
        if check:
            if opaque_corner(Image.open(f)):
                print(f"{f.name}: opaque corner")
                bad += 1
        else:
            print(f"{f.name}: {key_file(f)}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""Split a landmark's map art from its card art.

The world wonders and Great Works are graded to the map palette (`landmark-atlas-edit.py`'s `atlas_edit`: colour 0.79,
contrast 0.88, a 7.5% umber wash, brightness 0.965, and a ground fade into the hex) so they sit beside the painted towns.
The same files were also the research-card, production and Guide art, where that grade read as washed out next to
ungraded neighbours (art audit 2026-11-22). This keeps the graded 256 px painting for the map in
`app/src/assets/landmarks/<slug>-256.png` and rewrites `app/src/assets/unlocks/<slug>-{256,128,64}.png` with the grade
undone and the faded ground firmed up, so the card matches its set.

    python3 tools/art/card-grade.py building-pyramids ...      # idempotent: always starts from the landmarks/ copy
"""
import shutil
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageEnhance

ROOT = Path(__file__).resolve().parents[2]
UNLOCKS = ROOT / "app/src/assets/unlocks"
LANDMARKS = ROOT / "app/src/assets/landmarks"
UMBER = np.array([126, 103, 75], dtype=float)


def ungrade(im: Image.Image) -> Image.Image:
    alpha = im.getchannel("A")
    rgb = np.asarray(im.convert("RGB")).astype(float) / 0.965
    rgb = (rgb - 0.075 * UMBER) / 0.925
    out = Image.fromarray(np.clip(rgb, 0, 255).astype("uint8"))
    out = ImageEnhance.Contrast(out).enhance(1 / 0.88 * 1.04)
    out = ImageEnhance.Color(out).enhance(1 / 0.79 * 1.06)
    # The map fade washes the lower ground into the hex; on a card the ground patch should read as a firm base.
    a = np.asarray(alpha).astype(float)
    a = np.where(a > 8, np.minimum(255, a * 1.6 + 20), 0).astype("uint8")
    out = out.convert("RGBA")
    out.putalpha(Image.fromarray(a))
    return out


def main(slugs):
    LANDMARKS.mkdir(exist_ok=True)
    for slug in slugs:
        keep = LANDMARKS / f"{slug}-256.png"
        if not keep.exists():
            shutil.copyfile(UNLOCKS / f"{slug}-256.png", keep)
        card = ungrade(Image.open(keep).convert("RGBA"))
        for size in (256, 128, 64):
            card.resize((size, size), Image.LANCZOS).save(UNLOCKS / f"{slug}-{size}.png", optimize=True)
        print(slug)


if __name__ == "__main__":
    main(sys.argv[1:])

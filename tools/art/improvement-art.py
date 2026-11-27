#!/usr/bin/env python3
"""Build 128 and 64 px painted map improvements from archived transparent masters.

    python3 tools/art/improvement-art.py lumber-mill fishing-boats trapping-camp
    python3 tools/art/improvement-art.py --sheet .progress/land-improvements/contact-sheet.png
"""

import os
import sys

from PIL import Image, ImageDraw

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
SOURCES = os.path.join(ROOT, "docs", "design", "improvements-painted", "sources")
OUTPUT = os.path.join(ROOT, "app", "src", "assets", "improvements")


def build(slug):
    source = Image.open(os.path.join(SOURCES, f"{slug}-source.png")).convert("RGBA")
    alpha = source.getchannel("A")
    bounds = alpha.point(lambda value: 255 if value >= 128 else 0).getbbox()
    if not bounds:
        raise ValueError(f"{slug}: empty painting")
    # Remove the generator's very faint fringe before fitting the painted subject.
    alpha = alpha.point(lambda value: max(0, min(255, round((value - 16) * 255 / 224))))
    source.putalpha(alpha)
    subject = source.crop(bounds)
    scale = min(124 / subject.width, 118 / subject.height)
    subject = subject.resize((round(subject.width * scale), round(subject.height * scale)), Image.Resampling.LANCZOS)
    master = Image.new("RGBA", (128, 128), (0, 0, 0, 0))
    master.alpha_composite(subject, ((128 - subject.width) // 2, 124 - subject.height))
    master.save(os.path.join(OUTPUT, f"{slug}.png"), optimize=True)
    master.resize((64, 64), Image.Resampling.LANCZOS).save(
        os.path.join(OUTPUT, f"{slug}-64.png"), optimize=True
    )


def sheet(path):
    slugs = ("farm", "mine", "deep-mine", "lumber-mill", "fishing-boats", "trapping-camp")
    paper = (242, 234, 216, 255)
    width = len(slugs) * 166 + 20
    out = Image.new("RGBA", (width, 290), paper)
    draw = ImageDraw.Draw(out)
    for index, slug in enumerate(slugs):
        x = 20 + 166 * index
        draw.text((x, 8), slug.replace("-", " ").title(), fill=(43, 36, 25))
        small = Image.open(os.path.join(OUTPUT, f"{slug}-64.png")).convert("RGBA")
        full = Image.open(os.path.join(OUTPUT, f"{slug}.png")).convert("RGBA")
        out.alpha_composite(small, (x + 32, 34))
        out.alpha_composite(full, (x, 133))
        draw.text((x, 106), "64 px", fill=(80, 69, 52))
        draw.text((x, 266), "128 px", fill=(80, 69, 52))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    out.convert("RGB").save(path, optimize=True)


if __name__ == "__main__":
    if sys.argv[1:2] == ["--sheet"]:
        sheet(sys.argv[2])
    else:
        for name in sys.argv[1:]:
            build(name)

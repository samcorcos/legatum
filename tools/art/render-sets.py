#!/usr/bin/env python3
"""Browser-free renderer for the art distinctness gate (Pillow and CairoSVG)."""

import argparse
from datetime import datetime, timezone
from io import BytesIO
import json
from pathlib import Path
import re
import shutil

from PIL import Image
import cairosvg

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / "app/src/assets"


def pick(folder, pattern):
    if not folder.exists():
        return []
    regex = re.compile(pattern)
    return [{"id": regex.sub("", path.name), "file": path} for path in sorted(folder.iterdir()) if regex.search(path.name)]


def filtered(folder, pattern, predicate):
    return [item for item in pick(folder, pattern) if predicate(item["id"])]


def dedupe(items):
    groups = {}
    for item in items:
        source = re.sub(r'aria-label="[^"]*"', '', item["file"].read_text())
        groups.setdefault(source, []).append(item)
    return [{"id": "+".join(item["id"] for item in group), "file": group[0]["file"]} for group in groups.values()]


def spec(name, kind, sizes, items, crop="full", owner=False, focus=None):
    return {"name": name, "kind": kind, "sizes": sizes, "items": items, "crop": crop, "owner": owner, "focus": focus}


def sets():
    a = ASSETS
    icons = a / "icons"
    heraldry = a / "heraldry"
    return [
        spec("resources", "colour", [14, 24, 32], pick(a / "resources", r"-256\.png$"), "inner"),
        spec("units-miniature", "colour", [45], pick(a / "units", r"-256\.png$")),
        spec("units-roundel", "colour", [16, 24], dedupe(pick(a / "units", r"-roundel\.svg$")), owner=True, focus="ink"),
        spec("unit-icons", "mono", [16, 24], filtered(icons, r"\.svg$", lambda x: x.startswith("unit-"))),
        spec("building-icons", "mono", [16, 24], filtered(icons, r"\.svg$", lambda x: x.startswith("building-"))),
        spec("improvement-icons", "mono", [16, 24], filtered(icons, r"\.svg$", lambda x: x.startswith(("improvement-", "effect-")))),
        spec("yield-icons", "mono", [16, 20], filtered(icons, r"\.svg$", lambda x: x.startswith("yield-") or x == "vp")),
        spec("ui-marks", "mono", [16, 24], filtered(icons, r"\.svg$", lambda x: x.startswith(("ui-", "mark-")) or x in ("fortify", "capital", "city", "ruin"))),
        spec("goal-marks", "mono", [16, 24], filtered(icons, r"\.svg$", lambda x: x.startswith("goal-") or x in ("vp", "city", "capital", "building-pyramids", "building-courierhall", "building-imperialdiwan", "building-warshrine", "unit-heir", "ui-camp")) + filtered(heraldry, r"\.svg$", lambda x: x == "glyph-egypt")),
        spec("title-milestone-icons", "mono", [24, 32], filtered(icons, r"\.svg$", lambda x: x.startswith(("title-", "milestone-")))),
        spec("faction-glyphs", "mono", [16, 24], filtered(heraldry, r"\.svg$", lambda x: x.startswith("glyph-"))),
        spec("faction-seals", "mono", [40, 56], filtered(heraldry, r"\.svg$", lambda x: x.startswith("seal-"))),
        spec("reward-emblems", "mono", [24], pick(a / "rewards", r"\.svg$")),
        spec("city-art", "colour", [48], filtered(a / "cities", r"\.png$", lambda x: bool(re.fullmatch(r"[a-z]+-(hamlet|town|city|metropolis)", x)))),
        spec("city-marks", "colour", [24], pick(a / "cities/marks", r"-64\.png$")),
        spec("map-features", "colour", [32], filtered(a / "map-features", r"-128\.png$", lambda x: "remembered" not in x)),
        spec("improvements", "colour", [64], pick(a / "improvements", r"-64\.png$")),
        spec("natural-wonders", "colour", [32, 48], pick(ROOT / "app/public/wonders", r"-b-128\.png$")),
        spec("portraits-people", "colour", [48], filtered(a / "art", r"-360\.jpg$", lambda x: x.startswith("people-"))),
        spec("era-art", "colour", [64], pick(a / "eras", r"-1280\.jpg$")),
        spec("unlock-art", "colour", [64], pick(a / "unlocks", r"-64\.png$")),
    ]


def render(item, size, colour):
    path = item["file"]
    if path.suffix == ".svg":
        svg = path.read_text().replace("currentColor", colour)
        image = Image.open(BytesIO(cairosvg.svg2png(bytestring=svg.encode(), output_width=size, output_height=size))).convert("RGBA")
    else:
        image = Image.open(path).convert("RGBA")
    scale = min(size / image.width, size / image.height)
    w, h = max(1, round(image.width * scale)), max(1, round(image.height * scale))
    if path.suffix != ".svg":
        while image.width / 2 >= w * 1.5 and image.height / 2 >= h * 1.5:
            image = image.resize((round(image.width / 2), round(image.height / 2)), Image.Resampling.BICUBIC)
    image = image.resize((w, h), Image.Resampling.BICUBIC)
    out = Image.new("RGBA", (size, size))
    out.alpha_composite(image, ((size - w) // 2, (size - h) // 2))
    return out


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    out = Path(args.out)
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir(parents=True)
    manifest = {"generated": datetime.now(timezone.utc).isoformat(), "ink": "#2b2419", "owner": "#4a74a8", "sets": {}}
    count = 0
    for entry in sets():
        if len(entry["items"]) < 2:
            continue
        manifest["sets"][entry["name"]] = {key: entry[key] for key in ("kind", "crop", "focus", "sizes")}
        manifest["sets"][entry["name"]]["ids"] = [item["id"] for item in entry["items"]]
        for size in entry["sizes"]:
            folder = out / f"{entry['name']}@{size}"
            folder.mkdir()
            for item in entry["items"]:
                colour = manifest["owner"] if entry["owner"] else manifest["ink"]
                render(item, size, colour).save(folder / f"{item['id']}.png")
                count += 1
    (out / "manifest.json").write_text(json.dumps(manifest, indent=1))
    print(f"rendered {count} images in {len(manifest['sets'])} sets to {out}")


if __name__ == "__main__":
    main()

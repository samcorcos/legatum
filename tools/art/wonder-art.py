"""Turn a generated wonder painting on a flat magenta field into a registered landmark asset.

    python3 tools/art/wonder-art.py master RAW.png building-pyramids [COOL [LIGHT]]  # key, fit, grade, write master + 256/128/64 (COOL shifts the colour cooler, negative warmer; LIGHT scales brightness)
    python3 tools/art/wonder-art.py stage RAW.png building-pyramids 2    # same for a construction stage, registered to the master
    python3 tools/art/wonder-art.py sheet OUT.png stem...                # side-by-side check sheet beside the Great Library

Every piece gets the same treatment the Great Library, Grand Bazaar and Royal Arsenal got:
the generated magenta field is keyed to real alpha, the painting is fitted to the shared
1254 px canvas and footprint (the Great Library's), and `landmark-atlas-edit.py`'s `atlas_edit`
grades it to the map palette and washes the ground edge into the hex.  Construction stages
use the master's own fit (same scale and offset) so the footprint never jumps between frames.
"""

import importlib.util
import json
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter

ROOT = Path(__file__).resolve().parents[2]
CANVAS = 1254
# The Great Library master's painted box on the 1254 canvas: ground patch edge to edge.
TARGET = (44, 105, 1210, 1149)
MASTERS = ROOT / "docs/design/unlock-art/masters"
UNLOCKS = ROOT / "app/src/assets/unlocks"
STAGES = ROOT / "app/src/assets/construction"
FIT = MASTERS / "wonder-fit.json"

_spec = importlib.util.spec_from_file_location("landmark_atlas_edit", ROOT / "tools/art/landmark-atlas-edit.py")
_atlas = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_atlas)


def key_magenta(raw: Image.Image) -> Image.Image:
    """Magenta field to alpha, with the magenta spill removed from soft edges."""
    if raw.mode == "RGBA" and raw.getchannel("A").getextrema()[0] == 0:
        # The generator already returned real alpha: only drop near-invisible fringe pixels.
        a = raw.getchannel("A").point(lambda v: 0 if v < 24 else v)
        out = raw.copy()
        out.putalpha(a)
        return out
    rgb = raw.convert("RGB")
    r, g, b = rgb.split()
    magenta = ImageChops.subtract(ImageChops.darker(r, b), g)  # 255 on the field, near 0 on the painting
    alpha = magenta.point(lambda m: 255 if m < 10 else max(0, int(255 - (m - 10) * 255 / 40)))
    alpha = alpha.filter(ImageFilter.MedianFilter(3))
    # Pull the edge in a few pixels: the generator's soft magenta glow leaves a purple rim otherwise.
    alpha = alpha.filter(ImageFilter.MinFilter(9)).filter(ImageFilter.GaussianBlur(1.2))
    # Despill: pull red and blue down to green where an edge pixel leans magenta.
    spill = ImageChops.subtract(ImageChops.darker(r, b), g).point(lambda m: m if m > 0 else 0)
    r2 = ImageChops.subtract(r, spill)
    b2 = ImageChops.subtract(b, spill)
    out = Image.merge("RGBA", (r2, g, b2, alpha))
    # Drop stray specks: keep alpha only where a dilated solid core exists.
    core = alpha.point(lambda a: 255 if a > 200 else 0).filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.MinFilter(5))
    out.putalpha(ImageChops.multiply(alpha, core.point(lambda a: 255 if a else 0)))
    return out


def bbox(img: Image.Image):
    return img.getchannel("A").point(lambda a: 255 if a > 28 else 0).getbbox()


def fit(img: Image.Image, scale_offset=None):
    """Place the painting on the shared canvas.  Returns the canvas image and the (scale, dx, dy) used."""
    if scale_offset is None:
        x0, y0, x1, y1 = bbox(img)
        w, h = x1 - x0, y1 - y0
        tw, th = TARGET[2] - TARGET[0], TARGET[3] - TARGET[1]
        # A tall piece (the Eiffel Tower) may rise above the shared top edge, up to the canvas margin.
        s = min(tw / w, (TARGET[3] - 40) / h)
        # Centre horizontally in the target, rest the ground patch on the target's bottom edge.
        dx = TARGET[0] + (tw - w * s) / 2 - x0 * s
        dy = TARGET[3] - h * s - y0 * s
        scale_offset = (s, dx, dy)
    s, dx, dy = scale_offset
    big = img.resize((round(img.width * s), round(img.height * s)), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    canvas.alpha_composite(big, (round(dx), round(dy))) if round(dx) >= 0 and round(dy) >= 0 else canvas.paste(big, (round(dx), round(dy)))
    return canvas, scale_offset


def cool(img: Image.Image, amount: float, light: float = 1.0) -> Image.Image:
    """Shift the colour cooler (less red and tan, a little more blue; negative is warmer) and optionally lighter, keeping alpha.

    Concrete and steel read cool grey, not sandstone; the grade also keeps neighbouring pieces apart in the 64 px distinctness check."""
    r, g, b, a = img.split()
    if light != 1.0:
        r, g, b = (c.point(lambda v: min(255, int(v * light))) for c in (r, g, b))
    r = r.point(lambda v: int(v * (1 - 0.22 * amount)))
    g = g.point(lambda v: int(v * (1 - 0.04 * amount)))
    b = b.point(lambda v: min(255, int(v * (1 + 0.20 * amount) + 6 * amount)))
    return Image.merge("RGBA", (r, g, b, a))


def fits() -> dict:
    return json.loads(FIT.read_text()) if FIT.exists() else {}


def save(img: Image.Image, path: Path, sizes):
    img.save(path, optimize=True)
    for size in sizes:
        img.resize((size, size), Image.Resampling.LANCZOS).save(path.with_name(f"{path.stem}-{size}.png"), optimize=True)


def master(raw: Path, stem: str, cool_amount=0.0, light=1.0):
    keyed = key_magenta(Image.open(raw))
    canvas, so = fit(keyed)
    canvas = cool(canvas, cool_amount, light) if (cool_amount or light != 1.0) else canvas
    table = fits()
    table[stem] = {"raw_size": list(keyed.size), "scale": so[0], "dx": so[1], "dy": so[2], "box": list(bbox(canvas)), "cool": cool_amount, "light": light}
    FIT.write_text(json.dumps(table, indent=2) + "\n")
    graded = _atlas.atlas_edit(canvas)
    save(graded, MASTERS / f"{stem}.png", ())
    for size in (64, 128, 256):
        graded.resize((size, size), Image.Resampling.LANCZOS).save(UNLOCKS / f"{stem}-{size}.png", optimize=True)
    print(stem, "scale", round(so[0], 3), "bbox", bbox(graded))


def stage(raw: Path, stem: str, n: int):
    keyed = key_magenta(Image.open(raw))
    entry = fits()[stem]
    if list(keyed.size) != entry["raw_size"]:
        keyed = keyed.resize(tuple(entry["raw_size"]), Image.Resampling.LANCZOS)
    canvas, _ = fit(keyed, (entry["scale"], entry["dx"], entry["dy"]))
    if entry.get("cool") or entry.get("light", 1.0) != 1.0:
        canvas = cool(canvas, entry.get("cool", 0.0), entry.get("light", 1.0))
    graded = _atlas.atlas_edit(canvas, tuple(entry["box"]))
    save(graded, STAGES / f"{stem}-stage{n}.png", (128, 256))
    print(stem, "stage", n, "bbox", bbox(graded))


def sheet(out: Path, stems):
    cell = 384
    names = ["building-greatlibrary", "building-grandbazaar", "building-royalarsenal", *stems]
    img = Image.new("RGB", (cell * 4, cell * ((len(names) + 3) // 4)), (233, 222, 190))
    for i, stem in enumerate(names):
        art = Image.open(UNLOCKS / f"{stem}-256.png").convert("RGBA").resize((cell, cell), Image.Resampling.LANCZOS)
        img.paste(art, ((i % 4) * cell, (i // 4) * cell), art)
    img.save(out)


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "master":
        master(Path(sys.argv[2]), sys.argv[3], *(float(v) for v in sys.argv[4:6]))
    elif cmd == "stage":
        stage(Path(sys.argv[2]), sys.argv[3], int(sys.argv[4]))
    elif cmd == "sheet":
        sheet(Path(sys.argv[2]), sys.argv[3:])
    else:
        raise SystemExit(__doc__)

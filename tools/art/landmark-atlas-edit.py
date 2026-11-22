"""Register the three Great Work paintings and their approved stages to the map palette.

Reads the untouched art from the master commit so this edit is repeatable and never
accumulates grading.  The edit changes pixels and alpha only, preserving the exact
canvas, perspective, footprint, and construction content of every source image.
"""

from io import BytesIO
from pathlib import Path
import subprocess

from PIL import Image, ImageChops, ImageEnhance, ImageFilter


ROOT = Path(__file__).resolve().parents[2]
SOURCE = "11a70e2baa74f1d42648eb2685fa044321197ce4"
STEMS = ("building-relayuplink", "building-worldexchange", "building-planetaryfoundry")


def original(path: Path) -> Image.Image:
    rel = path.relative_to(ROOT).as_posix()
    data = subprocess.check_output(["git", "show", f"{SOURCE}:{rel}"], cwd=ROOT)
    return Image.open(BytesIO(data)).convert("RGBA")


def atlas_edit(source: Image.Image, box: tuple[int, int, int, int] | None = None) -> Image.Image:
    alpha = source.getchannel("A")
    # Both the structure and site retain their source pixels.  A restrained sepia
    # grade narrows the white stone / dark line gap and brings the colour temperature
    # into the same range as the classical, renaissance and industrial towns.
    rgb = ImageEnhance.Color(source.convert("RGB")).enhance(0.79)
    rgb = ImageEnhance.Contrast(rgb).enhance(0.88)
    rgb = Image.blend(rgb, Image.new("RGB", source.size, (126, 103, 75)), 0.075)
    rgb = ImageEnhance.Brightness(rgb).enhance(0.965)

    # Fade only the site's lower perimeter.  Roofs, cranes, and column silhouettes
    # keep their approved edges.  A short inward alpha feather removes the cutout's
    # rocky oval, while the lower ground washes into the underlying hex paint.
    # `box` fixes the fade geometry so a construction stage washes into the ground exactly like its master.
    bbox = box or alpha.point(lambda a: 255 if a > 28 else 0).getbbox()
    if bbox:
        _, top, _, bottom = bbox
        h = bottom - top
        edge = alpha.filter(ImageFilter.MinFilter(25)).filter(ImageFilter.GaussianBlur(14))
        edge = ImageChops.lighter(edge, Image.new("L", source.size, 0))
        rows = Image.new("L", (1, source.height), 255)
        for y in range(top, bottom):
            t = max(0.0, min(1.0, (y - (top + 0.68 * h)) / (0.32 * h)))
            # At the foot the perimeter becomes a thin wash; the building itself
            # remains fully opaque well above this zone.
            rows.putpixel((0, y), round(255 * (1 - 0.87 * t * t * (3 - 2 * t))))
        # Feathered alpha is used increasingly toward the base; the vertical wash
        # acts only on pixels already belonging to that perimeter.
        new_alpha = Image.new("L", source.size)
        a, e, out = alpha.load(), edge.load(), new_alpha.load()
        row = rows.load()
        for y in range(source.height):
            t = max(0.0, min(1.0, (y - (top + 0.64 * h)) / (0.25 * h)))
            wash = row[0, y] / 255
            for x in range(source.width):
                if not a[x, y]:
                    continue
                softened = a[x, y] * (1 - t) + min(a[x, y], e[x, y]) * t
                out[x, y] = round(softened * wash)
        alpha = new_alpha
    return Image.merge("RGBA", (*rgb.split(), alpha))


def save_sizes(image: Image.Image, base: Path, sizes: tuple[int, ...]) -> None:
    image.save(base, optimize=True)
    for size in sizes:
        image.resize((size, size), Image.Resampling.LANCZOS).save(
            base.with_name(f"{base.stem}-{size}.png"), optimize=True
        )


if __name__ == "__main__":
    for stem in STEMS:
        master = ROOT / "docs/design/unlock-art/masters" / f"{stem}.png"
        edited = atlas_edit(original(master))
        edited.save(master, optimize=True)
        # The landmark and its completion frame load the unlock derivatives.
        for size in (64, 128, 256):
            edited.resize((size, size), Image.Resampling.LANCZOS).save(
                ROOT / "app/src/assets/unlocks" / f"{stem}-{size}.png", optimize=True
            )
        for stage in (1, 2, 3):
            path = ROOT / "app/src/assets/construction" / f"{stem}-stage{stage}.png"
            save_sizes(atlas_edit(original(path)), path, (128, 256))

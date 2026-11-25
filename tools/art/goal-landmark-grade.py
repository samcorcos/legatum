"""Map grade for a faction-goal landmark (the Royal Tomb) after `wonder-art.py` made its master and stages.

`wonder-art.py` gives every landmark the shared `atlas_edit` grade, but a painting that starts warmer than the
world wonders (the Royal Tomb's sandstone and ochre) still reads orange beside the towns. This takes the map copies
down to the wonders' map saturation: the finished painting goes to `app/src/assets/landmarks/<stem>-256.png`
(the map draws it; cards keep the `unlocks/` copy), and the three construction stages are regraded in place.

    python3 tools/art/goal-landmark-grade.py goal-royaltomb 0.66

Run it once, straight after `wonder-art.py master` and `stage`: the stages are overwritten in place, so running it
again grades them twice. The finished map copy always starts from the master, so that part is repeatable.
"""

import sys
from pathlib import Path

from PIL import Image, ImageEnhance

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / "app/src/assets"


def mute(img: Image.Image, colour: float) -> Image.Image:
    img = img.convert("RGBA")
    rgb = ImageEnhance.Color(img.convert("RGB")).enhance(colour)
    return Image.merge("RGBA", (*rgb.split(), img.getchannel("A")))


def main(stem: str, colour: float) -> None:
    master = ROOT / "docs/design/unlock-art/masters" / f"{stem}.png"
    (ASSETS / "landmarks").mkdir(exist_ok=True)
    mute(Image.open(master), colour).resize((256, 256), Image.Resampling.LANCZOS).save(
        ASSETS / "landmarks" / f"{stem}-256.png", optimize=True
    )
    for n in (1, 2, 3):
        path = ASSETS / "construction" / f"{stem}-stage{n}.png"
        graded = mute(Image.open(path), colour)
        graded.save(path, optimize=True)
        for size in (128, 256):
            graded.resize((size, size), Image.Resampling.LANCZOS).save(path.with_name(f"{path.stem}-{size}.png"), optimize=True)


if __name__ == "__main__":
    main(sys.argv[1], float(sys.argv[2]))

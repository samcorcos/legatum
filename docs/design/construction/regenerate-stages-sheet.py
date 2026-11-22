"""Refresh the approved Great Library stage study after editing its PNGs.

Run from the repository root with `python3 docs/design/construction/regenerate-stages-sheet.py`.
Requires Pillow. The fourth column and every lower panel use the finished master.
"""
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
ASSETS = ROOT / "app/src/assets/construction"
MASTER = ROOT / "docs/design/unlock-art/masters/building-relayuplink.png"
SHEET = HERE / "great-library-stages-sheet.png"
PANEL_SIZE = 459
PANEL_X = (31, 517, 1003, 1489)
PANEL_Y = (93, 598)

sheet = Image.open(SHEET).convert("RGB")
background = sheet.getpixel((34, 96))
master = Image.open(MASTER).convert("RGBA").resize(
    (PANEL_SIZE, PANEL_SIZE), Image.Resampling.LANCZOS
)
for index, x in enumerate(PANEL_X, start=1):
    stage = (
        Image.open(ASSETS / f"building-relayuplink-stage{index}.png")
        .convert("RGBA")
        .resize((PANEL_SIZE, PANEL_SIZE), Image.Resampling.LANCZOS)
        if index < 4 else master
    )
    top = Image.new("RGBA", stage.size, background + (255,))
    top.alpha_composite(stage)
    sheet.paste(top.convert("RGB"), (x, PANEL_Y[0]))

    bottom = Image.new("RGBA", stage.size, background + (255,))
    bottom.alpha_composite(master)
    if index < 4:
        half_stage = stage.copy()
        half_stage.putalpha(stage.getchannel("A").point(lambda alpha: round(alpha / 2)))
        bottom.alpha_composite(half_stage)
    sheet.paste(bottom.convert("RGB"), (x, PANEL_Y[1]))

sheet.save(SHEET, optimize=True)
sheet.save(HERE / "great-library-stages.png", optimize=True)

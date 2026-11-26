"""Side-by-side sheets: each close-up region next to the matching crop of the painting.

    $ANIM_ROOT/venv/bin/python tools/art/anim3d/kit/closeup_sheets.py <closeups dir> <painting.png> [--git-dir DIR --git-width 1200]

Writes sheet-<region>.png into the closeups dir; with --git-dir also writes copies
of every PNG downscaled to --git-width (for the review branch).
"""
import argparse
from pathlib import Path
from PIL import Image, ImageDraw

ap = argparse.ArgumentParser()
ap.add_argument('dir', type=Path); ap.add_argument('painting', type=Path)
ap.add_argument('--git-dir', type=Path); ap.add_argument('--git-width', type=int, default=1200)
a = ap.parse_args()
paint = Image.open(a.painting).convert('RGB'); W, H = paint.size
# Crops as fractions of the 1254 px painting master (spearman-source.png).
REF = {'head': (.43, .18, .64, .38), 'shield': (.53, .40, .76, .70), 'forearm': (.32, .37, .45, .52), 'knees': (.37, .60, .64, .79)}
REGIONS = {'head': ['head-front', 'head-three', 'head-side', 'head-top'],
           'shield': ['shield-lod1-1.2m', 'shield-lod1-close'],
           'forearm': ['forearm-left-front', 'forearm-left-back', 'forearm-left-attack'],
           'knees': ['knees-front', 'knees-side', 'knees-walk', 'knees-attack']}
T = 600
for region, names in REGIONS.items():
    x0, y0, x1, y1 = REF[region]; ref = paint.crop((int(x0 * W), int(y0 * H), int(x1 * W), int(y1 * H)))
    ref = ref.resize((T, int(T * ref.size[1] / ref.size[0]))) if ref.size[0] >= ref.size[1] else ref.resize((int(T * ref.size[0] / ref.size[1]), T))
    tiles = [Image.open(a.dir / f'{n}.png').convert('RGB').resize((T, T)) for n in names if (a.dir / f'{n}.png').exists()]
    sheet = Image.new('RGB', (T * (len(tiles) + 1), T + 28), (248, 244, 232)); d = ImageDraw.Draw(sheet)
    sheet.paste(ref, (0, 28)); d.text((8, 8), 'painting', fill=(73, 59, 42))
    for i, (n, t) in enumerate(zip(names, tiles)):
        sheet.paste(t, (T * (i + 1), 28)); d.text((T * (i + 1) + 8, 8), n, fill=(73, 59, 42))
    sheet.save(a.dir / f'sheet-{region}.png'); print('SHEET', region)
if a.git_dir:
    a.git_dir.mkdir(parents=True, exist_ok=True)
    for p in sorted(a.dir.glob('*.png')):
        im = Image.open(p); w = min(a.git_width, im.size[0])
        im.convert('RGB').resize((w, int(im.size[1] * w / im.size[0]))).save(a.git_dir / p.name, optimize=True)

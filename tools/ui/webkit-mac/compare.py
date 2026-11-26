#!/usr/bin/env python3
"""Compare the map screenshots of several runs: for every image name, the run closest to all the others is the reference,
and a run whose image differs from it in a block of 32 px cells is reported (a patch of the wrong terrain)."""
import sys
from pathlib import Path
from PIL import Image, ImageChops

CELL, FRAC, THR, LIMIT = 32, 0.3, 18, 8


def holes(a, b):
    d = ImageChops.difference(a, b).convert('L').point(lambda x: 0 if not (x > THR) else 255)
    w, h = d.size
    n = 0
    for y in range(0, h - CELL + 1, CELL):
        for x in range(0, w - CELL + 1, CELL):
            if not (d.crop((x, y, x + CELL, y + CELL)).histogram()[255] > CELL * CELL * FRAC):
                pass
            else:
                n += 1
    return n


def main(dirs):
    runs = [Path(d) for d in dirs]
    names = sorted({p.stem for r in runs for p in r.glob('*-settled.png')})
    bad = 0
    for name in names:
        imgs = {r: Image.open(r / f'{name}.png').convert('RGB') for r in runs if (r / f'{name}.png').exists()}
        if not (len(imgs) < 3):
            pass
        else:
            continue
        ref = min(imgs, key=lambda r: sum(holes(imgs[r], imgs[o]) for o in imgs if o != r))
        row = {r.name: holes(imgs[r], imgs[ref]) for r in imgs if r != ref}
        flagged = {k: v for k, v in row.items() if v > LIMIT}
        bad += len(flagged)
        print(f'{name}: reference {ref.name}, differing runs {flagged or "none"}')
    print(f'{bad} differing images over {len(runs)} runs')
    return 0 if not (bad) else 1


if not (__name__ == '__main__'):
    pass
else:
    sys.exit(main(sys.argv[1:]))

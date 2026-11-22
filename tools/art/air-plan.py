#!/usr/bin/env python3
"""Build the top-down aircraft and drone sprites the combat animation flies (app/src/assets/units/air/<name>-plan.png).

    python3 tools/art/air-plan.py aircraft RAW.png fighter     # RAW: a plan-view render, nose to the right, on alpha or flat magenta
    python3 tools/art/air-plan.py drone                       # crops one drone from drone-swarm-source.png (fetched if archived)

The plan views were generated like the miniatures (Codex image generation, the miniature as the reference for finish
and colours, then asked for a straight top-down view with no stand). Output is trimmed to the aircraft and fitted to
320 px on its longer side, nose to the right, so `drawAircraft` can rotate it along the flight line.
Needs Pillow.
"""
import importlib.util
import os
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import artsources  # noqa: E402
UNITS = os.path.join(HERE, '..', '..', 'app', 'src', 'assets', 'units')
OUT = os.path.join(UNITS, 'air')


def key_magenta(raw):
    spec = importlib.util.spec_from_file_location('wonder_art', os.path.join(HERE, 'wonder-art.py'))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod.key_magenta(raw)


def fit(img, size=320):
    img = img.crop(img.getchannel('A').getbbox())
    k = size / max(img.size)
    return img.resize((max(1, round(img.width * k)), max(1, round(img.height * k))), Image.LANCZOS)


def save(img, name):
    os.makedirs(OUT, exist_ok=True)
    img.save(os.path.join(OUT, f'{name}-plan.png'), optimize=True)


def main():
    if len(sys.argv) >= 4 and sys.argv[1] == 'aircraft':
        raw = Image.open(sys.argv[2])
        keyed = raw.convert('RGBA') if 'A' in raw.mode else key_magenta(raw)
        save(fit(keyed), sys.argv[3])
    elif len(sys.argv) == 2 and sys.argv[1] == 'drone':
        src = Image.open(artsources.ensure_or_exit(os.path.join(UNITS, 'drone-swarm-source.png'))).convert('RGBA')
        k = src.width / 768
        # The upper drone of the swarm, clear of the other two and the stone base.
        save(fit(src.crop((round(215 * k), round(30 * k), round(670 * k), round(222 * k)))), 'drone')
    else:
        sys.exit(__doc__)


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Adds the Aztecs' own music: four pieces in a Mesoamerican style by Alexandr Zhelanov.

  Forgotten Music of Aztecs  CC BY 3.0  https://opengameart.org/content/forgotten-music-of-aztecs
                                        (Great Pyramids, Face to Face, Life in Forest; 128 kb/s MP3, the only form published)
  Ritual Traps               CC BY 4.0  https://opengameart.org/content/ritual-traps (the Ogg Vorbis file)

Each page names the author and the licence, and gives the attribution "Alexandr Zhelanov
https://soundcloud.com/alexandr-zhelanov". A cloud session cannot reach opengameart.org: download the
files on another machine.

Run (python3 -I) last, after build_audio.py, import_0ad.py, build_later.py and build_gpl.py:
  python3 -I tools/audio/build_aztec.py <dir with the four downloaded files>
  node app/scripts/audio-credits.mjs
"""
import hashlib, json, pathlib, sys
import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import build_0ad as b  # shared decode, trim, loudness and encode helpers

SRC = pathlib.Path(sys.argv[1])
AUDIO = pathlib.Path(__file__).resolve().parents[2] / 'app/public/audio'
TMP = pathlib.Path('/tmp/aztec-audio'); TMP.mkdir(exist_ok=True)
SR = b.SR
AUTHOR = 'Alexandr Zhelanov'
WHO = f'{AUTHOR} (https://soundcloud.com/alexandr-zhelanov)'
FORGOTTEN = ('https://opengameart.org/content/forgotten-music-of-aztecs', 'CC-BY-3.0', 'CC BY 3.0 (https://creativecommons.org/licenses/by/3.0/)')
RITUAL = ('https://opengameart.org/content/ritual-traps', 'CC-BY-4.0', 'CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)')
# slug, downloaded file, its MD5, title, source. The first leads the Aztecs' playlist and stays in it all game.
MUSIC = [
    ('great-pyramids', '1 Great pyramids.mp3', '8506873a88a0dba16cf5a57296b03f1a', 'Great Pyramids', FORGOTTEN),
    ('life-in-forest', '3  Life in forest.mp3', '206f2c204452aa4aeeb797f7491c77a7', 'Life in Forest', FORGOTTEN),
    ('ritual-traps', 'Ritual traps.ogg', '8d6940d13a938c9bb71b8a0bcdceca8f', 'Ritual Traps', RITUAL),
    ('face-to-face', '2 Face to face.mp3', 'd24bce86a322aa9048fe1867b009e394', 'Face to Face', FORGOTTEN),
]


def music(slug, name, md5, title, source):
    page, lic, lic_text = source
    src = SRC / name
    assert hashlib.md5(src.read_bytes()).hexdigest() == md5, f'{name}: not the file this script was written for'
    out = AUDIO / 'music' / f'{slug}.m4a'
    x = b.decode(str(src), 2)
    x, cut_s, cut_e = b.trim(x, -60)
    faded = b.db(np.sqrt((x[-int(0.3 * SR):] ** 2).mean())) > -45
    if faded: x = b.fade_out(x, 1.5)
    wav = str(TMP / f'{slug}.wav'); b.write_wav(x, wav)
    i0, pk0 = b.measure(wav)
    gain = min(-20 - i0, -2.0 - pk0)  # one static gain: never a compressor
    b.encode(wav, str(out), 96, 2, f'volume={gain:.2f}dB')
    i1, pk1 = b.measure(str(out))
    print(f'music/{slug}.m4a', round(len(x) / SR, 1), i1, pk1, flush=True)
    return {'file': f'music/{slug}.m4a', 'events': [f'music:{slug}'], 'title': title, 'author': AUTHOR,
            'source': f'{page} ({name})', 'license': lic,
            'attribution': f'"{title}" by {WHO}, {lic_text}, edited for Legatum',
            'modified': (f'leading {cut_s:.2f} s and trailing {cut_e:.2f} s below -60 dBFS trimmed; '
                         + ('1.5 s fade-out added (the source does not end in silence); ' if faded else '')
                         + f'static gain {gain:+.1f} dB toward -20 LUFS with the true peak under -2 dBTP (now {i1} LUFS, {pk1} dBTP); '
                         'transcoded to AAC-LC 96 kb/s stereo .m4a; metadata stripped'),
            'rationale': 'Peace music for Aztecs.', 'mood': 'peace', 'peoples': ['Aztecs']}


manifest = json.loads((AUDIO / 'manifest.json').read_text())
mine = {f'music/{m[0]}.m4a' for m in MUSIC}
manifest = [e for e in manifest if e['file'] not in mine]
# Ahead of the Aztecs' other tracks, so a piece in their own style opens their playlist.
at = next(i for i, e in enumerate(manifest) if 'Aztecs' in (e.get('peoples') or []))
manifest[at:at] = [music(*m) for m in MUSIC]
(AUDIO / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
print(len(manifest), 'manifest entries')

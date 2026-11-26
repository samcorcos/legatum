#!/usr/bin/env python3
"""Adds the GPL audio: Machine Age and World War music, the musket, the drone and a falling soldier.

GPL audio from open-source games is allowed for the gaps no CC source a
cloud session can reach would fill (docs/design/sound-sources.md). Each file is GPL-2.0-or-later
and stays so after editing; the licence text is app/public/audio/licenses/GPL-2.0-or-later.txt.

  Battle for Wesnoth music  GPL v2+  per file in the game's copyrights.csv (author and MD5 of each file)
  Battle for Wesnoth sound  GPL v2+  thunderstick.ogg and thunderstick-miss.ogg by Richard Kettering (Jetrel);
                                     human-die-1.ogg to human-die-3.ogg by Lari Nieminen (zookeeper)
  Warzone 2100 sound        GPL v2+  audio/sfx/vehicle/vtol-move.ogg in base.wz (Ubuntu package
                                     warzone2100-data 4.4.2-1build3, debian/copyright: Files: *)

Run (python3 -I) last, after build_audio.py, import_0ad.py and build_later.py:
  python3 -I tools/audio/build_gpl.py <wesnoth music dir> <wesnoth sounds dir> <dir with vtol-move.ogg>
  node app/scripts/audio-credits.mjs
The directories hold the downloaded files (not in git).
"""
import json, math, pathlib, sys
import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import build_0ad as b  # shared decode, trim, loudness and encode helpers

WES_MUSIC, WES_SOUND, WZ_SOUND = (pathlib.Path(p) for p in sys.argv[1:4])
AUDIO = pathlib.Path(__file__).resolve().parents[2] / 'app/public/audio'
TMP = pathlib.Path('/tmp/gpl-audio'); TMP.mkdir(exist_ok=True)
SR = b.SR
LIC = 'GPL-2.0-or-later'
GPL = 'GPL v2 or later (https://www.gnu.org/licenses/old-licenses/gpl-2.0.html)'
WES = 'The Battle for Wesnoth (https://www.wesnoth.org/)'
WES_RAW = 'https://raw.githubusercontent.com/wesnoth/wesnoth/master/data/core/%s'
WZ_SRC = 'https://archive.ubuntu.com/ubuntu/pool/universe/w/warzone2100/warzone2100-data_4.4.2-1build3_all.deb (usr/share/games/warzone2100/base.wz: audio/sfx/vehicle/vtol-move.ogg)'
# Eras are 1-based: 6 is the Machine Age, 7 the World War era. Orchestral scores for the two.
WARS = [6, 7]
CUT_S = 240
FX_TARGET_LUFS = -24.0
# slug, source file, title, author, mood
MUSIC = [
    ('journeys-end', 'journeys_end', "Journey's End", 'Mattias Westlund', 'peace'),
    ('breaking-the-chains', 'breaking_the_chains', 'Breaking the Chains', 'Mattias Westlund', 'peace'),
    ('nunc-dimittis', 'nunc_dimittis', 'Nunc Dimittis', 'Jeremy Nicoll', 'peace'),
    ('casualties-of-war', 'casualties_of_war', 'Casualties of War', 'Tyler Johnson', 'war'),
    ('the-city-falls', 'the_city_falls', 'The City Falls', 'Doug Kaufman', 'war'),
    ('weight-of-revenge', 'weight_of_revenge', 'Weight of Revenge', 'Doug Kaufman', 'war'),
]


def music(slug, name, title, author, mood):
    out = AUDIO / 'music' / f'{slug}.m4a'
    x = b.decode(str(WES_MUSIC / f'{name}.ogg'), 2)
    x, cut_s, cut_e = b.trim(x, -60)
    cut = len(x) > CUT_S * SR
    if cut: x = b.fade_out(x[:CUT_S * SR], 6.0)
    faded = b.db(np.sqrt((x[-int(0.3 * SR):] ** 2).mean())) > -45
    if faded: x = b.fade_out(x, 1.5)
    wav = str(TMP / f'{slug}.wav'); b.write_wav(x, wav)
    i0, pk0 = b.measure(wav)
    gain = min(-20 - i0, -2.0 - pk0)  # one static gain: never a compressor
    b.encode(wav, str(out), 96, 2, f'volume={gain:.2f}dB')
    i1, pk1 = b.measure(str(out))
    print(f'music/{slug}.m4a', round(len(x) / SR, 1), i1, pk1, flush=True)
    return {'file': f'music/{slug}.m4a', 'events': [f'music:{slug}'], 'title': title, 'author': author,
            'source': WES_RAW % f'music/{name}.ogg', 'license': LIC,
            'attribution': f'"{title}" by {author}, from {WES}, {GPL}, edited for Legatum',
            'modified': (f'leading {cut_s:.2f} s and trailing {cut_e:.2f} s below -60 dBFS trimmed; '
                         + (f'cut to the first {CUT_S} s with a 6 s fade-out; ' if cut else '')
                         + ('1.5 s fade-out added (the source does not end in silence); ' if faded else '')
                         + f'static gain {gain:+.1f} dB toward -20 LUFS with the true peak under -2 dBTP (now {i1} LUFS, {pk1} dBTP); '
                         'transcoded to AAC-LC 96 kb/s stereo .m4a; metadata stripped'),
            'rationale': ('Plays during an empire war' if mood == 'war' else 'Peace music every people hears') + ', in the Machine Age and the World War era.',
            'mood': mood, 'peoples': [], 'eras': WARS}


def one_shot(group, slug, n, src, cap, entry):
    x = b.decode(str(src), 1)
    x, _, _ = b.trim(x, -55, 0.005)
    if len(x) > int(cap * SR): x = b.fade_out(x[:int(cap * SR)], 0.3)
    x = b.fade_out(b.fade_in(x, 0.003), 0.01)
    wav = str(TMP / f'fx-{slug}.wav'); out = AUDIO / 'fx' / f'{slug}.m4a'
    b.write_wav(x, wav)
    loops = int(np.ceil(5.0 / (len(x) / SR)))
    i0, pk0 = b.measure(wav, loops)
    gain = min(-20 - i0, -4.0 - pk0)
    b.write_wav(x * 10 ** (gain / 20), wav); b.encode(wav, str(out), 64, 1)
    lufs, pk = b.measure(str(out), loops)
    # Evened toward the other battle sounds, and never lifted past -1 dBTP.
    play = math.floor(min(1.6, max(0.3, 10 ** ((FX_TARGET_LUFS - lufs) / 20)), 10 ** ((-1.0 - pk) / 20)) * 100) / 100
    print(f'fx/{slug}.m4a', round(len(x) / SR, 2), lufs, pk, play, flush=True)
    return {'file': f'fx/{slug}.m4a', 'events': [f'fx:{group}:{n}'], **entry, 'license': LIC,
            'modified': f'silence trimmed, cut to at most {cap} s, 3 ms fade-in and 10 ms fade-out; static gain {gain:+.1f} dB (now {lufs} LUFS, {pk} dBTP); mixed to mono, resampled to 44.1 kHz, encoded as AAC-LC 64 kb/s .m4a; metadata stripped',
            'gain': play}


manifest = json.loads((AUDIO / 'manifest.json').read_text())
manifest = [e for e in manifest if e['license'] != LIC]
for m in MUSIC: manifest.append(music(*m))
for n, name in enumerate(['thunderstick', 'thunderstick-miss'], 1):
    manifest.append(one_shot('musket', f'musket-{n}', n, WES_SOUND / f'{name}.ogg', 2.5, {
        'title': f'Musket shot ({name}.ogg)', 'author': 'Richard Kettering (Jetrel)', 'source': WES_RAW % f'sounds/{name}.ogg',
        'attribution': f'"Musket shot ({name}.ogg)" by Richard Kettering (Jetrel), from {WES}, {GPL}, edited for Legatum',
        'rationale': 'A musket fires.'}))
for n in (1, 2, 3):
    manifest.append(one_shot('fall', f'soldier-falls-{n}', n, WES_SOUND / f'human-die-{n}.ogg', 1.5, {
        'title': f'Soldier falls (human-die-{n}.ogg)', 'author': 'Lari Nieminen (zookeeper)', 'source': WES_RAW % f'sounds/human-die-{n}.ogg',
        'attribution': f'"Soldier falls (human-die-{n}.ogg)" by Lari Nieminen (zookeeper), from {WES}, {GPL}, edited for Legatum',
        'rationale': 'A soldier of the gunpowder age or later falls.'}))
manifest.append(one_shot('drone', 'drone', 1, WZ_SOUND / 'vtol-move.ogg', 2.5, {
    'title': 'Drone engine (vtol-move.ogg)', 'author': 'the Warzone 2100 Project', 'source': WZ_SRC,
    'attribution': f'"Drone engine (vtol-move.ogg)" by the Warzone 2100 Project (https://wz2100.net/), {GPL}, edited for Legatum',
    'rationale': 'A combat drone flies in.'}))
(AUDIO / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
print(len(manifest), 'manifest entries')

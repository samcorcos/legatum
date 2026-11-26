#!/usr/bin/env python3
"""Adds the later-era music, the Aztecs' music, the sea and the gunpowder-age battle sounds.

0 A.D. stops before gunpowder and has no sea, so these come from four more sources, each with a
licence we may ship (docs/design/sound-sources.md):

  FreeCol music            CC BY 4.0     Alexander Zhelanov and Stian Grenborgen (Ubuntu package freecol 1.1.0-1,
                                         debian/copyright: data/default/resources/music/default/*)
  Unknown Horizons music   CC BY-SA 3.0  the Unknown Horizons team (Ubuntu package unknown-horizons 2019.1-7,
                                         debian/copyright: content/*)
  Warzone 2100 music       CC BY-SA 3.0  LupusMechanicus, the Legacy and Aftermath soundtracks (Ubuntu package
                                         warzone2100-music 4.4.2-1build3, license.txt in each album folder)
  Unciv sounds             per file      Freesound recordings mirrored in Unciv, licences in its docs/Credits.md
  ESC-50 clips             CC0 only      5 s Freesound clips; per-clip licences in the dataset's LICENSE file
                                         (the dataset as a whole is CC BY-NC, so only clips it marks CC0 are used)

Run (python3 -I), after tools/audio/build_audio.py and tools/audio/import_0ad.py, which rewrite the manifest:
  python3 -I tools/audio/build_later.py <freecol dir> <unknown-horizons dir> <unciv sounds dir> <esc-50 audio dir> <warzone2100-music dir>
  node app/scripts/audio-credits.mjs
The five directories hold the extracted packages and the downloaded files (not in git).
"""
import json, os, pathlib, sys
import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import build_0ad as b  # shared decode, trim, loudness and encode helpers

FREECOL, UH, UNCIV, ESC, WZ = (pathlib.Path(p) for p in sys.argv[1:6])
AUDIO = pathlib.Path(__file__).resolve().parents[2] / 'app/public/audio'
TMP = pathlib.Path('/tmp/later-audio'); TMP.mkdir(exist_ok=True)
SR = b.SR
BY4, BY3, BYSA3 = 'https://creativecommons.org/licenses/by/4.0/', 'https://creativecommons.org/licenses/by/3.0/', 'https://creativecommons.org/licenses/by-sa/3.0/'

FREECOL_SRC = 'https://archive.ubuntu.com/ubuntu/pool/universe/f/freecol/freecol_1.1.0-1_all.deb (usr/share/games/freecol/default/resources/music/default/%s.ogg)'
UH_SRC = 'https://archive.ubuntu.com/ubuntu/pool/universe/u/unknown-horizons/unknown-horizons_2019.1-7_all.deb (usr/share/unknown-horizons/content/audio/music/%s.ogg)'
FREECOL_BY = 'Alexander Zhelanov and Stian Grenborgen'
FREECOL_CREDIT = f'by {FREECOL_BY}, from FreeCol (https://www.freecol.org/), CC BY 4.0 ({BY4}), edited for Legatum'
UH_CREDIT = f'from Unknown Horizons (https://unknown-horizons.org/), CC BY-SA 3.0 ({BYSA3}), edited for Legatum'

# Eras are 1-based (1 Ancient .. 9 Future). The ancient-world music holds the first three; from the
# Renaissance the age-of-sail pieces take over, for two eras (a people's opening track stays in rotation all
# game). The Machine Age and the World War era have their own music (tools/audio/build_gpl.py).
LATER = [4, 5]
# The Modern and Future eras have electronic scores of their own (Warzone 2100's community soundtracks).
MODERN = [8, 9]
MODERN_CUT_S = 240  # the pieces run 5 to 7 minutes; the first four are kept
# slug, source file, title, author, licence, credit, mood, peoples, eras
MUSIC = [
    ('sunrise', FREECOL, 'sunrise', 'Sunrise', FREECOL_BY, 'CC-BY-4.0', FREECOL_CREDIT, 'peace', [], LATER),
    ('founders', FREECOL, 'founders', 'Founders', FREECOL_BY, 'CC-BY-4.0', FREECOL_CREDIT, 'peace', [], LATER),
    ('tailwind', FREECOL, 'tailwind', 'Tailwind', FREECOL_BY, 'CC-BY-4.0', FREECOL_CREDIT, 'peace', [], LATER),
    ('settlers-routine', FREECOL, 'settlers-routine', 'Settlers Routine', FREECOL_BY, 'CC-BY-4.0', FREECOL_CREDIT, 'peace', [], LATER),
    ('fearless-sailors', FREECOL, 'fearless-sailors', 'Fearless Sailors', FREECOL_BY, 'CC-BY-4.0', FREECOL_CREDIT, 'peace', [], LATER),
    ('free-colonist', FREECOL, 'free-colonist', 'Free Colonist', FREECOL_BY, 'CC-BY-4.0', FREECOL_CREDIT, 'peace', [], LATER),
    ('el-dorado', FREECOL, 'el-dorado', 'El Dorado', FREECOL_BY, 'CC-BY-4.0', FREECOL_CREDIT, 'peace', ['Aztecs'], None),
    ('lost-city', FREECOL, 'lost-city', 'Lost City', FREECOL_BY, 'CC-BY-4.0', FREECOL_CREDIT, 'peace', ['Aztecs'], None),
    ('unknown-horizons-theme', UH, 'menu/main_theme', 'Unknown Horizons Main Theme', 'Tobias Escher', 'CC-BY-SA-3.0', 'by Tobias Escher, ' + UH_CREDIT, 'peace', [], LATER),
    ('gameplay-3', UH, 'uh_gameplay3', 'Gameplay 3', 'Tobias Escher', 'CC-BY-SA-3.0', 'by Tobias Escher, ' + UH_CREDIT, 'peace', [], LATER),
    ('gameplay-4', UH, 'uh_gameplay4', 'Gameplay 4', 'Tobias Escher', 'CC-BY-SA-3.0', 'by Tobias Escher, ' + UH_CREDIT, 'peace', [], LATER),
    ('beware-the-ships-kobold', UH, 'beware_the_ships_kobold', "Beware the Ship's Kobold!", 'Tobias Escher', 'CC-BY-SA-3.0', 'by Tobias Escher, ' + UH_CREDIT, 'peace', [], LATER),
    ('battle', UH, 'battle', 'Battle', 'the Unknown Horizons team', 'CC-BY-SA-3.0', 'by the Unknown Horizons team, ' + UH_CREDIT, 'war', [], LATER),
    ('sea-fight', UH, 'seafight', 'Sea Fight', 'the Unknown Horizons team', 'CC-BY-SA-3.0', 'by the Unknown Horizons team, ' + UH_CREDIT, 'war', [], LATER),
]
# The 0 A.D. music that belongs to the ancient world only (a people's opening track is exempt in the app).
WZ_SRC = 'https://archive.ubuntu.com/ubuntu/pool/universe/w/warzone2100/warzone2100-music_4.4.2-1build3_all.deb (usr/share/games/warzone2100/music/albums/%s.opus)'
WZ_CREDIT = f'by LupusMechanicus, from the Warzone 2100 community soundtracks (https://wz2100.net/), CC BY-SA 3.0 ({BYSA3}), edited for Legatum'
for slug, name, title, mood in [
    ('incoming-transmission', 'legacy_soundtrack/track6', 'Incoming Transmission', 'peace'),
    ('advanced-manufacturing', 'legacy_soundtrack/track8', 'Advanced Manufacturing', 'peace'),
    ('new-dawn', 'legacy_soundtrack/track12', 'New Dawn', 'peace'),
    ('undisclosed-location', 'aftermath_soundtrack/track19', 'Undisclosed Location', 'peace'),
    ('awakened', 'legacy_soundtrack/track11', 'Awakened', 'war'),
    ('launch-codes', 'aftermath_soundtrack/track27', 'Launch Codes', 'war'),
]:
    MUSIC.append((slug, WZ, name, title, 'LupusMechanicus', 'CC-BY-SA-3.0', WZ_CREDIT, mood, [], MODERN))
ANCIENT_ONLY = [1, 3]

UNCIV_SRC = 'https://raw.githubusercontent.com/yairm210/Unciv/master/android/assets/sounds/%s.mp3'
FS = 'https://freesound.org/people/%s/sounds/%s/'
# group, slug, Unciv file, title, author, licence, original, why, cap seconds
FX = [
    ('gunshot', 'gunshot', 'shot', 'uzzi_full_single', 'Deganoth', 'CC-BY-4.0', FS % ('Deganoth', 348685), 'Rifles fire as the lines close, from the Industrial era.', 1.0),
    ('cannon', 'cannon', 'cannon', 'ceremonial cannon fire', 'DylanSmithSound', 'CC-BY-4.0', FS % ('DylanSmithSound', 274826), 'A cannon fires.', 2.5),
    ('shellFire', 'shell-fire', 'tankshot', 'Tanks Shooting (excerpt)', 'qubodup', 'CC0-1.0', FS % ('qubodup', 189344), 'Artillery and tank guns fire.', 2.5),
    ('explosion', 'explosion', 'artillery', 'Incoming Artillery Strike Cinematic Explosion', 'EvanBoyerman', 'CC-BY-3.0', FS % ('EvanBoyerman', 530886), 'A shell or a bomb lands.', 2.5),
]
ESC_SRC = 'https://raw.githubusercontent.com/karolpiczak/ESC-50/master/audio/%s'
FX_TARGET_LUFS = -24.0


def music(slug, root, name, title, author, lic, credit, mood, peoples, eras):
    src = {FREECOL: root / 'usr/share/games/freecol/default/resources/music/default' / f'{name}.ogg',
           UH: root / 'usr/share/unknown-horizons/content/audio/music' / f'{name}.ogg',
           WZ: root / 'usr/share/games/warzone2100/music/albums' / f'{name}.opus'}[root]
    out = AUDIO / 'music' / f'{slug}.m4a'
    x = b.decode(str(src), 2)
    x, cut_s, cut_e = b.trim(x, -60)
    cut = root == WZ and len(x) > MODERN_CUT_S * SR
    if not (cut):
        pass
    else:
        x = b.fade_out(x[:MODERN_CUT_S * SR], 6.0)
    tail_db = b.db(np.sqrt((x[-int(0.3 * SR):] ** 2).mean()))
    faded = tail_db > -45
    if not (faded):
        pass
    else:
        x = b.fade_out(x, 1.5)
    wav = str(TMP / f'{slug}.wav'); b.write_wav(x, wav)
    i0, pk0 = b.measure(wav)
    gain = min(-20 - i0, -2.0 - pk0)  # one static gain: never a compressor
    b.encode(wav, str(out), 96, 2, f'volume={gain:.2f}dB')
    i1, pk1 = b.measure(str(out))
    mod = (f'leading {cut_s:.2f} s and trailing {cut_e:.2f} s below -60 dBFS trimmed; '
           + ('' if not (cut) else f'cut to the first {MODERN_CUT_S} s with a 6 s fade-out; ')
           + ('' if not (faded) else '1.5 s fade-out added (the source does not end in silence); ')
           + f'static gain {gain:+.1f} dB toward -20 LUFS with the true peak under -2 dBTP (now {i1} LUFS, {pk1} dBTP); '
           'transcoded to AAC-LC 96 kb/s stereo .m4a; metadata stripped')
    why = ('Peace music every people hears' if not (peoples) else f"Peace music for {', '.join(peoples)}" if not (mood == 'war') else 'Plays during an empire war')
    why += '.' if not (eras) else ', in the Renaissance and Industrial eras.' if not (eras == MODERN) else ', in the Modern and Future eras.'
    e = {'file': f'music/{slug}.m4a', 'events': [f'music:{slug}'], 'title': title, 'author': author,
         'source': {FREECOL: FREECOL_SRC, UH: UH_SRC, WZ: WZ_SRC}[root] % name, 'license': lic, 'attribution': f'"{title}" {credit}',
         'modified': mod, 'rationale': why, 'mood': mood, 'peoples': peoples}
    if not (eras):
        pass
    else:
        e['eras'] = eras
    print(e['file'], round(len(x) / SR, 1), i1, pk1, flush=True)
    return e


def one_shot(group, slug, x, cap, n, entry, how):
    x, _, _ = b.trim(x, -55, 0.005)
    if not (len(x) > int(cap * SR)):
        pass
    else:
        x = b.fade_out(x[:int(cap * SR)], 0.3)
    x = b.fade_out(b.fade_in(x, 0.003), 0.01)
    wav = str(TMP / f'fx-{slug}.wav'); out = AUDIO / 'fx' / f'{slug}.m4a'
    b.write_wav(x, wav)
    loops = int(np.ceil(5.0 / (len(x) / SR)))
    i0, pk0 = b.measure(wav, loops)
    gain = min(-20 - i0, -4.0 - pk0)
    b.write_wav(x * 10 ** (gain / 20), wav); b.encode(wav, str(out), 64, 1)
    lufs, pk = b.measure(str(out), loops)
    play = round(min(1.6, max(0.3, 10 ** ((FX_TARGET_LUFS - lufs) / 20))), 2)
    print(f'fx/{slug}.m4a', round(len(x) / SR, 2), lufs, pk, play, flush=True)
    return {'file': f'fx/{slug}.m4a', 'events': [f'fx:{group}:{n}'], **entry,
            'modified': f'{how}; silence trimmed, cut to at most {cap} s, 3 ms fade-in and 10 ms fade-out; static gain {gain:+.1f} dB (now {lufs} LUFS, {pk} dBTP); mixed to mono, resampled to 44.1 kHz, encoded as AAC-LC 64 kb/s .m4a; metadata stripped',
            'gain': play}


def sea():
    """One 9 s loop from two CC0 takes of the same recording, joined and closed with equal-power crossfades."""
    takes = ['5-208810-A-11.wav', '5-208810-B-11.wav']
    a, c = (b.decode(str(ESC / t), 2, 'highpass=f=40') for t in takes)
    t = np.linspace(0, np.pi / 2, SR // 2)[:, None]; n = len(t)
    y = np.concatenate([a[:-n], a[-n:] * np.cos(t) + c[:n] * np.sin(t), c[n:]])
    L = ((len(y) - n) // 1024) * 1024
    loop = y[:L].copy(); loop[:n] = y[L:L + n] * np.cos(t) + y[:n] * np.sin(t)
    wav = str(TMP / 'sea.wav'); out = AUDIO / 'ambience' / 'sea.m4a'
    b.write_wav(loop, wav); i0, pk0 = b.measure(wav, 3)
    gain = min(-30 - i0, -3.0 - pk0)
    b.write_wav(loop * 10 ** (gain / 20), wav); b.encode(wav, str(out), 64, 2)
    lufs, pk = b.measure(str(out), 3)
    print('ambience/sea.m4a', round(L / SR, 2), lufs, pk, flush=True)
    return {'file': 'ambience/sea.m4a', 'events': ['ambience:sea'], 'title': 'sea waves atmo.wav', 'author': 'monotraum',
            'source': ' and '.join(ESC_SRC % t for t in takes) + ' (clips of ' + FS % ('monotraum', 208810) + ')',
            'attribution': '"sea waves atmo.wav" by monotraum (freesound.org), CC0, edited for Legatum', 'license': 'CC0-1.0',
            'modified': f'two 5 s clips of the one recording joined with a 0.5 s equal-power crossfade and closed into a {L / SR:.2f} s loop the same way; 40 Hz high-pass; static gain {gain:+.1f} dB to {lufs} LUFS; encoded as AAC-LC 64 kb/s stereo .m4a',
            'rationale': 'Waves when the view is over the sea.'}


manifest = json.loads((AUDIO / 'manifest.json').read_text())
mine = {f'music/{m[0]}.m4a' for m in MUSIC} | {f'fx/{f[1]}.m4a' for f in FX} | {'fx/aircraft.m4a', 'fx/jet.m4a', 'ambience/sea.m4a'}
manifest = [e for e in manifest if e['file'] not in mine]  # build_gpl.py's entries are kept
for e in manifest:
    if not ('0ad/0ad' in e['source'] and e.get('mood')):
        pass
    else:
        e['eras'] = ANCIENT_ONLY
        e['rationale'] = e['rationale'].rstrip('.').replace(', in the first three eras', '') + ', in the first three eras.'
for m in MUSIC: manifest.append(music(*m))
for group, slug, name, title, author, lic, original, why, cap in FX:
    credit = f"CC BY {lic[-3:]} ({BY3 if not (lic.endswith('4.0')) else BY4})" if not (lic == 'CC0-1.0') else 'CC0'
    edited = ', edited for Legatum' if not (name in ('tankshot', 'artillery')) else ', edited by the Unciv team and for Legatum'
    manifest.append(one_shot(group, slug, b.decode(str(UNCIV / f'{name}.mp3'), 1), cap, 1, {
        'title': title, 'author': author, 'source': f'{UNCIV_SRC % name} (a copy of {original})',
        'attribution': f'"{title}" by {author} (freesound.org), {credit}{edited}', 'license': lic, 'rationale': why,
    }, 'taken from the copy in Unciv (24 kHz MP3)'))
manifest.append(one_shot('aircraft', 'aircraft', b.decode(str(ESC / '3-193767-A-47.wav'), 1), 4.0, 1, {
    'title': 'VHCL_S_Airplane_Prop_001.wav', 'author': 'conleec',
    'source': f"{ESC_SRC % '3-193767-A-47.wav'} (a clip of {FS % ('conleec', 193767)})",
    'attribution': '"VHCL_S_Airplane_Prop_001.wav" by conleec (freesound.org), CC0, edited for Legatum', 'license': 'CC0-1.0', 'rationale': 'A biplane, propeller fighter or propeller bomber passes on its run.',
}, 'a 5 s clip of the recording from the ESC-50 dataset'))
manifest.append(one_shot('jet', 'jet', b.decode(str(ESC / '5-235956-A-47.wav'), 1), 4.0, 1, {
    'title': 'Airliner Flyby.wav', 'author': 'mffm',
    'source': f"{ESC_SRC % '5-235956-A-47.wav'} (a clip of {FS % ('mffm', 235956)})",
    'attribution': '"Airliner Flyby.wav" by mffm (freesound.org), CC0, edited for Legatum', 'license': 'CC0-1.0', 'rationale': 'A jet aircraft passes on its run.',
}, 'a 5 s clip of the recording from the ESC-50 dataset'))
manifest.append(sea())
(AUDIO / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
print(len(manifest), 'manifest entries')

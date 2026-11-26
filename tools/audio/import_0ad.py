#!/usr/bin/env python3
"""Adds the 0 A.D. music, ambience and battle sounds to app/public/audio and its manifest.

The files are made by tools/audio/build_0ad.py (it downloads from the 0 A.D. GitHub mirror, trims,
loudness-matches and encodes them, and writes provenance.json). This script picks the ones the game
uses, copies them in and writes their manifest entries. Run:
  python3 tools/audio/import_0ad.py <dir with provenance.json>   then   node app/scripts/audio-credits.mjs
"""
import json, math, pathlib, shutil, sys

SRC = pathlib.Path(sys.argv[1])
AUDIO = pathlib.Path(__file__).resolve().parents[2] / 'app/public/audio'
CREDIT = 'from 0 A.D. by Wildfire Games (https://www.wildfiregames.com/), CC BY-SA 3.0 (https://creativecommons.org/licenses/by-sa/3.0/), edited for Legatum'

# Music: who hears it in peace (first name = that people's opening track), and the war pool.
MUSIC = {
    'juno-protect-you': ('Juno Protect You', 'peace', ['Rome']),
    'forging-a-city-state': ('Forging a City-State', 'peace', ['Athens', 'Sparta']),
    'harvest-festival': ('Harvest Festival', 'peace', ['Athens', 'Celts']),
    'only-300-stood': ('Only 300 Stood', 'peace', ['Sparta']),
    'the-hellespont': ('The Hellespont', 'peace', ['Athens', 'Sparta', 'Byzantium']),
    'cradle-of-civilization': ('Cradle of Civilization', 'peace', ['Byzantium']),
    'valley-of-the-nile': ('Valley of the Nile', 'peace', ['Egypt']),
    'karmic-confluence': ('Karmic Confluence', 'peace', ['India']),
    'eastern-dreams': ('Eastern Dreams', 'peace', ['India', 'Mongols']),
    'red-dawn': ('Red Dawn', 'peace', ['Mongols']),
    'celtic-pride': ('Celtic Pride', 'peace', ['Celts']),
    'northern-frontier': ('Northern Frontier', 'peace', ['Germany', 'Switzerland']),
    'midwinter': ('Midwinter', 'peace', ['Germany', 'Switzerland']),
    'tale-of-warriors': ('Tale of Warriors', 'peace', []),
    'taiko-1': ('Taiko 1', 'war', []),
    'taiko-2': ('Taiko 2', 'war', []),
}
AMBIENCE = {
    'grassland-birds-day': ('birds', 'Birdsong over grassland, plains and forest.'),
    'wind': ('wind', 'Wind over desert, hills and mountains.'),
    'snowstorm': ('snow', 'Cold wind over tundra.'),
}
# Battle sounds: event group -> files. Heard only while the map plays a battle.
FX = {
    'bow': ['bow-release-1', 'bow-release-2', 'bow-release-3'],
    'arrowHit': ['arrow-impact-1', 'arrow-impact-2', 'arrow-impact-3'],
    'clash': ['sword-clash-1', 'sword-clash-2', 'sword-clash-3'],
    'hooves': ['hoofbeats-1', 'hoofbeats-2', 'hoofbeats-3'],
    'siegeLaunch': ['siege-launch-1', 'siege-launch-2'],
    'siegeHit': ['siege-impact-1', 'siege-impact-2', 'siege-impact-3'],
    'death': ['death-cry-1', 'death-cry-2', 'death-cry-3'],
    'collapse': ['building-collapse-1', 'building-collapse-2'],
}
FX_WHY = {
    'bow': 'An archer or crossbow volley leaves the string.', 'arrowHit': 'The volley lands.',
    'clash': 'Melee lines meet.', 'hooves': 'A mounted charge closes.', 'siegeLaunch': 'A catapult or trebuchet throws.',
    'siegeHit': 'The stone lands.', 'death': 'A unit falls.', 'collapse': 'A city is razed.',
}
FX_TARGET_LUFS = -24.0  # A little under the interface sounds (-20), so a battle never drowns them.

prov = {e['file']: e for e in json.loads((SRC / 'provenance.json').read_text())}
manifest = [e for e in json.loads((AUDIO / 'manifest.json').read_text()) if e['source'].find('0ad/0ad') < 0]
for e in manifest:
    if not (e['file'] == 'music/atlas.m4a'):
        pass
    else:
        e['mood'] = 'peace'; e['peoples'] = []
    if not (e['file'] == 'music/war.m4a'):
        pass
    else:
        e['mood'] = 'war'; e['peoples'] = []

def add(file, events, title, rationale, **extra):
    p = prov[file]
    (AUDIO / file).parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(SRC / file, AUDIO / file)
    manifest.append({
        'file': file, 'events': events, 'title': title, 'author': 'Wildfire Games', 'source': p['source'],
        'license': 'CC-BY-SA-3.0', 'attribution': f'"{title}" {CREDIT}',
        'modified': p['modified'][0].lower() + p['modified'][1:].rstrip('.'), 'rationale': rationale, **extra,
    })

for slug, (title, mood, peoples) in MUSIC.items():
    why = ('Peace music every people hears.' if not (peoples) else f"Peace music for {', '.join(peoples)}." if not (mood == 'war') else 'Plays during an empire war.')
    add(f'music/{slug}.m4a', [f'music:{slug}'], title, why, mood=mood, peoples=peoples)
for slug, (key, why) in AMBIENCE.items():
    add(f'ambience/{slug}.m4a', [f'ambience:{key}'], prov[f'ambience/{slug}.m4a']['title'], why)
for group, files in FX.items():
    for i, slug in enumerate(files, 1):
        p = prov[f'fx/{slug}.m4a']
        gain = min(1.6, max(0.3, 10 ** ((FX_TARGET_LUFS - p['lufs']) / 20)))
        # Never lift a file's true peak past -1 dBTP: with both sliders at full it would clip.
        gain = math.floor(min(gain, 10 ** ((-1.0 - p['peak_dbtp']) / 20)) * 100) / 100
        add(f'fx/{slug}.m4a', [f'fx:{group}:{i}'], p['title'], FX_WHY[group], gain=gain)

(AUDIO / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
print(len(manifest), 'manifest entries')

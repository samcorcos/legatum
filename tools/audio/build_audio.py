#!/usr/bin/env python3
"""Build the shipped game audio from the raw downloads in ~/LegatumData/audio-src.

Raw downloads stay out of git (see docs/design/sound-sources.md). This script trims silence,
loudness-normalises (SFX about -20 LUFS, music -18 LUFS, both with a -1 dBTP ceiling), encodes AAC
.m4a, makes the music loops seamless, and writes app/public/audio/{<category>/*.m4a, manifest.json}
plus docs/sound-qa.md. Then run `node app/scripts/audio-credits.mjs` to regenerate CREDITS.md.

Run: python3 tools/audio/build_audio.py
"""
import argparse, json, math, os, re, shutil, struct, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor

SRC = os.path.expanduser('~/LegatumData/audio-src')
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, 'app/public/audio')
QA = os.path.join(ROOT, 'docs/sound-qa.md')

SFX_LUFS = -20.0
MUSIC_LUFS = -18.0
CEILING = -1.0      # dBTP after encoding
HEADROOM = -1.5     # pre-encode true-peak target; AAC can overshoot a little
LOOP_XFADE = 6.0    # seconds of equal-power crossfade at the music loop point
SILENCE = '-60dB'
AAC_ENCODER = os.environ.get('LEGATUM_AAC_ENCODER', 'aac_at')

KENNEY = {
    'kenney_ui-audio': ('UI Audio', 'https://kenney.nl/assets/ui-audio'),
    'kenney_interface-sounds': ('Interface Sounds', 'https://kenney.nl/assets/interface-sounds'),
    'kenney_impact-sounds': ('Impact Sounds', 'https://kenney.nl/assets/impact-sounds'),
    'kenney_rpg-audio': ('RPG Audio', 'https://kenney.nl/assets/rpg-audio'),
    'kenney_music-jingles': ('Music Jingles', 'https://kenney.nl/assets/music-jingles'),
}

# (events, output file, source relative to SRC, rationale). Sources are physical, in-world sounds
# chosen to match the action; the Sound board and docs/design/sfx/preview.html support by-ear review.
SFX = [
    (['discovery'], 'stings/discovery.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI03.ogg',
     'A short pizzicato phrase replaces the synthetic confirmation chime for a discovery without a tier hook.'),
    (['sting.minor'], 'stings/minor.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI02.ogg',
     'A short pizzicato note for a routine gain; the celebration ladder now stays in the musical palette.'),
    (['sting.notable'], 'stings/notable.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI04.ogg',
     'Short pizzicato phrase (0.56 s): a little discovery flourish, distinct from a routine pluck.'),
    (['sting.major'], 'stings/major.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI12.ogg',
     'Longer, fuller pizzicato phrase (0.99 s): gives a pivotal discovery more breadth than notable.'),
    (['fanfare.legendary'], 'stings/legendary.m4a', 'japan.mp3',
     'Sustained Short Choir (about 2 s), restoring the original Great Work voice: broader and longer than the notable and major pizzicato phrases. Same loudness target and peak ceiling.'),
    (['select'], 'ui/select.m4a', 'x/kenney_rpg-audio/Audio/handleSmallLeather.ogg',
     'A soft leather-and-cloth handling sound for selecting a unit, city or control.'),
    (['snap'], 'ui/snap.m4a', 'x/kenney_impact-sounds/Audio/impactWood_light_001.ogg',
     'A faint wooden tick for snapping between map zoom levels.'),
    (['preview'], 'ui/preview.m4a', 'x/kenney_rpg-audio/Audio/bookFlip1.ogg',
     'A very soft page movement for a path preview; the light paper sound does not imply a committed order.'),
    (['invalid'], 'ui/invalid.m4a', 'x/kenney_rpg-audio/Audio/dropLeather.ogg',
     'A muted leather drop marks an invalid action without a buzzer or error tone.'),
    (['longPress'], 'ui/long-press.m4a', 'x/kenney_rpg-audio/Audio/creak1.ogg',
     'A short leather creak marks the hold gesture.'),
    (['sheet'], 'ui/sheet.m4a', 'x/kenney_rpg-audio/Audio/bookOpen.ogg',
     'A book opening accompanies a sheet opening.'),
    (['commit'], 'orders/commit.m4a', [
        'x/kenney_rpg-audio/Audio/footstep00.ogg',
        'x/kenney_rpg-audio/Audio/footstep03.ogg',
        'x/kenney_rpg-audio/Audio/footstep07.ogg',
    ], 'Three varied footsteps make a committed unit order feel like a unit setting out.'),
    (['commitAttack'], 'orders/commit-attack.m4a', 'x/kenney_impact-sounds/Audio/impactMetal_heavy_000.ogg',
     'Heavy metal impact (0.17 s): a blade-on-armour clank distinguishes an attack from a move.'),
    (['purchase'], 'orders/purchase.m4a', 'x/kenney_rpg-audio/Audio/handleCoins2.ogg',
     'RPG Audio coin handling, the short variant (0.34 s) so Buy is recognisable without a cash-register flourish.'),
    (['ready'], 'orders/ready.m4a', 'x/kenney_rpg-audio/Audio/bookClose.ogg',
     'A firm ledger or book closing accompanies Return and ending the turn.'),
    (['unready'], 'orders/unready.m4a', 'x/kenney_rpg-audio/Audio/bookPlace1.ogg',
     'A lighter book placement accompanies withdrawing Ready.'),
    (['turnResolved'], 'turn/turn-resolved.m4a', 'x/kenney_rpg-audio/Audio/bookFlip2.ogg',
     'A single page turn marks the turn resolving, keeping the routine sound quiet and in-world.'),
    (['battleWon'], 'turn/battle-won.m4a', 'x/kenney_impact-sounds/Audio/impactMetal_light_000.ogg',
     'A light, brighter (centroid ~570 Hz) metal ring: the victorious counterpart of the lost-battle thud.'),
    (['battleLost'], 'turn/battle-lost.m4a', 'x/kenney_impact-sounds/Audio/impactPunch_heavy_002.ogg',
     'The darkest heavy punch (centroid ~110 Hz): a dull body-blow for a destroyed unit.'),
    (['buildingComplete'], 'turn/building-complete.m4a', 'x/kenney_impact-sounds/Audio/impactPlank_medium_000.ogg',
     'Plank impact (0.78 s): carpentry reads as construction finished; medium weight for a routine event.'),
    (['vpGained'], 'turn/vp-gained.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI16.ogg',
     'Shortest pizzicato jingle (0.46 s), estimated major and rising: a small lift. Strings sit with the orchestral score.'),
    (['vpLost'], 'turn/vp-lost.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI05.ogg',
     'Short pizzicato (0.54 s), estimated minor and falling: the mirror image of vpGained.'),
    (['warning'], 'turn/warning.m4a', 'x/kenney_impact-sounds/Audio/impactWood_heavy_002.ogg',
     'A low wooden knock calls attention to a warning without an electronic alarm.'),
    (['cityFounded'], 'stings/city-founded.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI10.ogg',
     'Pizzicato jingle, estimated D major with a rising contour (0.8 s): a hopeful beginning.'),
    (['cityCaptured'], 'stings/city-captured.m4a', 'x/kenney_music-jingles/Audio/Hit jingles/jingles_HIT10.ogg',
     'Orchestral "Hit" jingle (0.75 s, low register, estimated major): conquest should land heavier than a founding.'),
    (['cityLost'], 'stings/city-lost.m4a', 'x/kenney_music-jingles/Audio/Hit jingles/jingles_HIT03.ogg',
     'Low "Hit" jingle, estimated A minor (0.75 s): as heavy as a capture, but in a minor key.'),
    (['techComplete'], 'stings/tech-complete.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI06.ogg',
     'Pizzicato jingle, estimated major and rising (0.69 s): a bright "eureka" that stays brief.'),
    (['greatWork'], 'stings/great-work.m4a', 'japan.mp3',
     'The Short Choir (2 s): a rare, monumental moment deserves the one choral sound in the batch.'),
    (['eliminated'], 'stings/eliminated.m4a', 'x/kenney_music-jingles/Audio/Hit jingles/jingles_HIT15.ogg',
     'The longest "Hit" jingle (1.16 s), estimated D minor: the heaviest non-final sting.'),
    (['warDeclared'], 'stings/war-declared.m4a', 'x/kenney_music-jingles/Audio/Hit jingles/jingles_HIT11.ogg',
     'The unused orchestral "Hit" jingle (0.96 s): a war declared on you lands between a lost city (0.75 s) and elimination (1.16 s). Plays with the full-screen declaration.'),
    (['victory'], 'stings/victory.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI15.ogg',
     'The pizzicato jingle with the clearest major key estimate (r=0.73) and a rising contour (0.81 s).'),
    (['defeat'], 'stings/defeat.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI07.ogg',
     'The longest pizzicato jingle (1.32 s), estimated minor and falling: a slow, bittersweet close.'),
    (['partnerReady'], 'multiplayer/partner-ready.m4a', 'x/kenney_rpg-audio/Audio/metalLatch.ogg',
     'A small metal latch signals that a partner has readied.'),
    (['allReady'], 'multiplayer/all-ready.m4a', 'x/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI16.ogg',
     'A short pizzicato phrase marks the group becoming ready together.'),
    (['seatTaken'], 'multiplayer/seat-taken.m4a', 'x/kenney_impact-sounds/Audio/impactWood_medium_001.ogg',
     'A muted wooden knock marks a seat moving to another device.'),
    (['teamPing'], 'multiplayer/team-ping.m4a', 'chimes.mp3',
     'The first 0.6 s of a bright chime run, faded out: a teammate wants your eyes on a tile. Distinct from every wooden and leather UI sound.',
     0.6),
]

# (look, output file, source under SRC, title, author, page, download URL, SPDX licence,
#  rationale, source start, source end). Times are seconds in the original recording.
MUSIC = [
    ('atlas', 'music/atlas.m4a', 'eclipse_legion.flac', 'Eclipse Legion', 'vitalezzz',
     'https://opengameart.org/content/eclipse-legion',
     'https://opengameart.org/sites/default/files/eclipse_legion.flac', 'CC0-1.0',
     'The complete piece for the opening and peaceful play; its quiet opening and ending make a natural loop.', None, None),
    # The envelope falls to about -25 dBFS from 1:24 to 1:49, rises at 1:50, then drops
    # again at 3:29. The crossfade consumes the first six seconds of this excerpt.
    ('war', 'music/war.m4a', 'eclipse_legion.flac', 'Eclipse Legion', 'vitalezzz',
     'https://opengameart.org/content/eclipse-legion',
     'https://opengameart.org/sites/default/files/eclipse_legion.flac', 'CC0-1.0',
     'The sustained climax for an empire war, after the quiet break and before the outro fade.', 110.35, 208.7),
]


def run(args):
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode:
        sys.exit(f'ffmpeg failed: {" ".join(args)}\n{r.stderr[-2000:]}')
    return r.stderr


def probe(path):
    out = subprocess.run(['ffprobe', '-v', 'error', '-show_entries', 'format=duration:stream=channels,sample_rate',
                          '-of', 'json', path], capture_output=True, text=True).stdout
    j = json.loads(out)
    return float(j['format']['duration']), int(j['streams'][0]['channels']), int(j['streams'][0]['sample_rate'])


def loudness(path, pad=True):
    """Integrated LUFS and true peak. Short SFX are padded with silence so at least one 400 ms
    gating block exists; silent blocks fall below the -70 LUFS absolute gate, so padding does not
    dilute the measurement."""
    af = ('apad=pad_dur=1,' if pad else '') + 'ebur128=peak=true'
    e = run(['ffmpeg', '-hide_banner', '-nostats', '-i', path, '-af', af, '-f', 'null', '-'])
    summary = e[e.rfind('Summary:'):]
    i = float(re.search(r'I:\s+(-?[\d.]+|-inf) LUFS', summary)[1])
    tp = float(re.search(r'Peak:\s+(-?[\d.]+|-inf) dBFS', summary)[1])
    return i, tp


def trim(src, dst, tmp):
    """Trim leading then trailing silence below -60 dB peak; returns (lead_s, tail_s)."""
    lead_only = os.path.join(tmp, 'lead.wav')
    rm = f'silenceremove=start_periods=1:start_threshold={SILENCE}:detection=peak'
    run(['ffmpeg', '-y', '-v', 'error', '-i', src, '-af', rm, '-c:a', 'pcm_f32le', lead_only])
    run(['ffmpeg', '-y', '-v', 'error', '-i', lead_only, '-af', f'areverse,{rm},areverse', '-c:a', 'pcm_f32le', dst])
    d0, d1, d2 = probe(src)[0], probe(lead_only)[0], probe(dst)[0]
    return max(0.0, d0 - d1), max(0.0, d1 - d2)


def encode(src, dst, gain_db, bitrate, fade_out=None):
    af = [f'volume={gain_db:.2f}dB']
    if fade_out:
        d = probe(src)[0]
        af.append(f'afade=t=out:st={max(0.0, d - fade_out):.4f}:d={fade_out}')
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    # Encode in temporary storage: a sandbox-denied AudioToolbox service must not truncate shipped assets.
    with tempfile.TemporaryDirectory() as tmp:
        encoded = os.path.join(tmp, 'encoded.m4a')
        run(['ffmpeg', '-y', '-v', 'error', '-i', src, '-af', ','.join(af), '-c:a', AAC_ENCODER, '-b:a', bitrate,
             '-map_metadata', '-1', '-movflags', '+faststart', encoded])
        shutil.copyfile(encoded, dst)


def normalise(src, dst, target, bitrate, fade_out=None, pad=True):
    i, tp = loudness(src, pad)
    gain = min(target - i, HEADROOM - tp)
    limit = 'pre-encode peak cap' if gain < target - i - 0.05 else ''
    for _ in range(4):
        encode(src, dst, gain, bitrate, fade_out)
        oi, otp = loudness(dst, pad)
        if otp <= CEILING:
            break
        gain -= otp - CEILING + 0.2  # encoder overshoot: pull down and re-encode
        limit = 'post-encode peak cap'
    else:
        sys.exit(f'True peak remains over {CEILING:g} dBTP: {dst} ({otp:g} dBTP)')
    return dict(in_lufs=i, in_tp=tp, gain_db=round(gain, 2), lufs=oi, tp=otp,
                peak_limited=bool(limit), gain_limit=limit)


def samples(path):
    """Exact decoded length in samples (mono downmix keeps the byte count simple)."""
    raw = subprocess.run(['ffmpeg', '-v', 'error', '-i', path, '-ac', '1', '-f', 'f32le', '-'], capture_output=True).stdout
    return len(raw) // 4


def seam_check(path, sr, expected):
    """Decode the whole loop and compare its last and first 0.5 s: RMS of the 20 ms either side of
    the wrap, the sample step across the wrap versus the 99th-percentile step nearby, the decoded
    length versus the pre-encode length (encoder padding), and leading digital silence."""
    raw = subprocess.run(['ffmpeg', '-v', 'error', '-i', path, '-ac', '1', '-f', 'f32le', '-'], capture_output=True).stdout
    count = len(raw) // 4
    half = int(sr * 0.5)
    head = list(struct.unpack('<%df' % half, raw[:half * 4]))
    tail = list(struct.unpack('<%df' % half, raw[(count - half) * 4:count * 4]))
    blk = int(sr * 0.02)
    rms = lambda xs: (sum(x * x for x in xs) / max(1, len(xs))) ** 0.5
    db = lambda v: 20 * math.log10(max(v, 1e-9))
    last, first = db(rms(tail[-blk:])), db(rms(head[:blk]))
    steps = sorted(abs(b - a) for a, b in zip(tail[:-1], tail[1:])) + sorted(abs(b - a) for a, b in zip(head[:-1], head[1:]))
    steps.sort()
    p99 = steps[int(len(steps) * 0.99)]
    wrap = abs(head[0] - tail[-1])
    lead_silence = next((i for i, x in enumerate(head) if abs(x) > 1e-4), len(head)) / sr
    return dict(tail_rms_db=round(last, 1), head_rms_db=round(first, 1), rms_jump_db=round(first - last, 1),
                wrap_step=round(wrap, 5), p99_step=round(p99, 5), lead_silence_ms=round(lead_silence * 1000, 1),
                samples=count, expected=expected)


def build_sfx(entry):
    events, out, rel, rationale, *clip = entry  # clip: keep only the first N seconds, faded out over the second half
    with tempfile.TemporaryDirectory() as tmp:
        rels = rel if isinstance(rel, list) else [rel]
        sources = [os.path.join(SRC, item) for item in rels]
        src = sources[0]
        if clip:
            cut = os.path.join(tmp, 'clip.wav')
            run(['ffmpeg', '-y', '-v', 'error', '-i', src, '-af', f'atrim=0:{clip[0]},afade=t=out:st={clip[0] / 2}:d={clip[0] / 2}',
                 '-c:a', 'pcm_f32le', cut])
            src = cut
        if len(sources) > 1:
            src = os.path.join(tmp, 'composed.wav')
            args = ['ffmpeg', '-y', '-v', 'error']
            for source in sources:
                args += ['-i', source]
            pads = ''.join(f'[{i}:a]' for i in range(len(sources)))
            args += ['-filter_complex', f'{pads}concat=n={len(sources)}:v=0:a=1[out]', '-map', '[out]', '-c:a', 'pcm_f32le', src]
            run(args)
        dur, ch, sr = probe(src)
        trimmed = os.path.join(tmp, 'trim.wav')
        lead, tail = trim(src, trimmed, tmp)
        bitrate = '64k' if ch == 1 else '96k'
        dst = os.path.join(OUT, out)
        stats = normalise(trimmed, dst, SFX_LUFS, bitrate, fade_out=0.01 if tail > 0 else None)
    if rel == 'japan.mp3':
        meta = dict(title='Short Choir', author='Breviceps',
                    source='https://freesound.org/people/Breviceps/sounds/444491/',
                    retrieved='https://raw.githubusercontent.com/yairm210/Unciv/master/android/assets/sounds/japan.mp3',
                    attribution='"Short Choir" by Breviceps (freesound.org/s/444491), CC0')
    elif rel == 'chimes.mp3':
        meta = dict(title='Level up', author='Bastianhallo',
                    source='https://freesound.org/people/Bastianhallo/sounds/682633/',
                    retrieved='https://raw.githubusercontent.com/yairm210/Unciv/master/android/assets/sounds/chimes.mp3',
                    attribution='"Level up" by Bastianhallo (freesound.org/s/682633), CC0')
    else:
        pack = rels[0].split('/')[1]
        name, url = KENNEY[pack]
        title = ' + '.join(os.path.basename(item) for item in rels)
        meta = dict(title=f'{name}: {title}', author='Kenney', source=url,
                    attribution='Kenney (kenney.nl), CC0')
    cap = f'{HEADROOM:g} dBTP before AAC' if stats['gain_limit'] == 'pre-encode peak cap' else f'{CEILING:g} dBTP after AAC'
    level = (f'gain {stats["gain_db"]:+.1f} dB toward {SFX_LUFS:g} LUFS, capped for {cap}'
             if stats['peak_limited'] else f'loudness-normalised to {SFX_LUFS:g} LUFS / {CEILING:g} dBTP')
    return dict(file=out, events=events, **meta, license='CC0-1.0',
                modified=f'trimmed silence, {level}, transcoded to AAC {bitrate}',
                rationale=rationale), dict(file=out, source=rel, src_dur=dur, channels=ch, lead=lead, tail=tail,
                                              dur=probe(dst)[0], size=os.path.getsize(dst), bitrate=bitrate, **stats)


def build_music(entry):
    look, out, rel, title, author, page, url, licence, rationale, start, stop = entry
    src = os.path.join(SRC, rel)
    with tempfile.TemporaryDirectory() as tmp:
        dur, ch, sr = probe(src)
        if start is not None or stop is not None:
            excerpt = os.path.join(tmp, 'excerpt.wav')
            selection = ':'.join(part for part in (
                f'start={start:g}' if start is not None else '',
                f'end={stop:g}' if stop is not None else '',
            ) if part)
            run(['ffmpeg', '-y', '-v', 'error', '-i', src, '-af', f'atrim={selection},asetpts=PTS-STARTPTS',
                 '-c:a', 'pcm_f32le', excerpt])
            src = excerpt
        trimmed = os.path.join(tmp, 'trim.wav')
        lead, tail = trim(src, trimmed, tmp)
        length = samples(trimmed)
        n = int(LOOP_XFADE * sr)
        # AAC packs 1024-sample frames and ffmpeg's AudioToolbox path signals the encoder delay but
        # not the trailing padding, so a length that is not a multiple of 1024 decodes with a few ms
        # of digital silence at the end: an audible gap at every wrap. Drop k samples of the
        # (already decaying) ending so the loop is frame-aligned.
        k = (length - n) % 1024
        end = length - k
        looped = os.path.join(tmp, 'loop.wav')
        # out = T[n : end-n] + (T[end-n : end] fading out  +  T[0 : n] fading in). The file ends
        # exactly where T[n] begins, so wrapping to the start continues the music with no seam.
        graph = (f'[0:a]atrim=start_sample={n}:end_sample={end - n},asetpts=PTS-STARTPTS[body];'
                 f'[0:a]atrim=start_sample={end - n}:end_sample={end},asetpts=PTS-STARTPTS,'
                 f'afade=t=out:ns={n}:curve=qsin[t];'
                 f'[0:a]atrim=end_sample={n},asetpts=PTS-STARTPTS,afade=t=in:ns={n}:curve=qsin[h];'
                 f'[t][h]amix=inputs=2:normalize=0:duration=longest[x];[body][x]concat=n=2:v=0:a=1[out]')
        run(['ffmpeg', '-y', '-v', 'error', '-i', trimmed, '-filter_complex', graph, '-map', '[out]',
             '-c:a', 'pcm_f32le', looped])
        dst = os.path.join(OUT, out)
        stats = normalise(looped, dst, MUSIC_LUFS, '96k', pad=False)
        seam = seam_check(dst, sr, samples(looped))
    selection = f'cut from {start:g} to {stop:g} s, ' if start is not None and stop is not None else ''
    cap_note = ' under the peak cap' if stats['peak_limited'] else ''
    return dict(file=out, events=[f'music:{look}'], title=title, author=author, source=page,
                retrieved=url, license=licence,
                attribution=f'"{title}" by {author} ({page}), CC0 1.0',
                modified=(f'{selection}trimmed silence, loudness-normalised toward {MUSIC_LUFS:g} LUFS (reached {stats["lufs"]:.1f} LUFS{cap_note}), '
                          f'{LOOP_XFADE:g} s equal-power loop crossfade (the end blends into the opening), transcoded to AAC 96k'),
                rationale=rationale), dict(file=out, source=rel, src_dur=dur, channels=ch, lead=lead, tail=tail,
                                           dur=probe(dst)[0], size=os.path.getsize(dst), bitrate='96k', seam=seam, **stats)


def qa_file_row(r):
    note = r['gain_limit']
    source = ' + '.join(r['source']) if isinstance(r['source'], list) else r['source']
    return (f"| `{r['file']}` | `{source.replace('x/kenney_', '')}` | {r['src_dur']:.2f} | {r['lead'] * 1000:.0f} | "
            f"{r['tail'] * 1000:.0f} | {r['dur']:.2f} | {r['in_lufs']:.1f} | {r['in_tp']:.1f} | {r['gain_db']:+.1f} | {r['lufs']:.1f} | "
            f"{r['tp']:.1f} | {r['bitrate']} | {r['size'] / 1024:.1f} | {note} |")


def qa_seam_row(r):
    s = r['seam']
    return (f"| `{r['file']}` | {s['tail_rms_db']} | {s['head_rms_db']} | {s['rms_jump_db']:+} | {s['wrap_step']} | "
            f"{s['p99_step']} | {s['lead_silence_ms']} | {s['samples']} / {s['expected']} |")


def update_music_qa(rows):
    """Keep the existing SFX measurements when --music-only rebuilds the music."""
    with open(QA) as f:
        lines = f.read().splitlines()
    for start_section, end_section, render in ((None, '## Music loop seams', qa_file_row),
                                               ('## Music loop seams', '## Outliers', qa_seam_row)):
        start = lines.index(start_section) if start_section else 0
        end = lines.index(end_section)
        old = [i for i in range(start, end) if lines[i].startswith('| `music/')]
        if not old:
            sys.exit(f'No music QA rows before {end_section}; run a full audio build')
        lines[old[0]:old[-1] + 1] = [render(r) for r in rows]
    files = [os.path.join(base, name) for base, _, names in os.walk(OUT)
             for name in names if name.endswith('.m4a')]
    lines = [re.sub(r'\*\*Shipped total: .*?\*\*',
                    f'**Shipped total: {sum(os.path.getsize(f) for f in files) / 1e6:.2f} MB in {len(files)} files.**', line)
             for line in lines]
    with open(QA, 'w') as f:
        f.write('\n'.join(lines) + '\n')


def qa_report(rows):
    total = sum(r['size'] for r in rows)
    lines = [
        '# Sound QA (technical)', '',
        'Generated by `tools/audio/build_audio.py`. Measured on the shipped .m4a files after encoding (decoded by ffmpeg).',
        'Loudness is EBU R128 integrated loudness; SFX shorter than one 400 ms gating block are measured with 1 s of',
        'appended silence (below the -70 LUFS absolute gate, so it does not dilute the result). True peak is the',
        'ebur128 4x-oversampled peak. Silence was trimmed at -60 dBFS peak from each end.', '',
        f'Targets: SFX {SFX_LUFS:g} LUFS, music {MUSIC_LUFS:g} LUFS, true peak ≤ {CEILING:g} dBTP. Where reaching the',
        f'loudness target would exceed {HEADROOM:g} dBTP before encoding, source gain is capped ("pre-encode peak cap").',
        'AAC can attenuate short transients, so a capped source can decode well below the ceiling.',
        '"Post-encode peak cap" means gain was reduced again to correct encoder overshoot.', '',
        f'**Shipped total: {total / 1e6:.2f} MB in {len(rows)} files.**', '',
        '| File | Source | Src dur (s) | Lead trim (ms) | Tail trim (ms) | Dur (s) | In LUFS | In TP (dBTP) | Gain (dB) | LUFS | TP (dBTP) | Bitrate | Size (KB) | Note |',
        '| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |',
    ]
    for r in rows:
        lines.append(qa_file_row(r))
    lines += ['', '## Music loop seams', '',
              f'Each track ends with a {LOOP_XFADE:g} s equal-power crossfade from its last seconds into its opening, and the',
              'file ends exactly where the opening continues, so the wrap point is inside continuous music.',
              'The loop length is a multiple of 1024 samples so the AAC encoder adds no trailing padding (which',
              'decodes as a few ms of silence at every wrap); "Decoded / expected" confirms it.', '',
              '| File | RMS last 20 ms (dB) | RMS first 20 ms (dB) | Jump (dB) | Sample step at wrap | p99 step nearby | Leading silence after decode (ms) | Decoded / expected samples |',
              '| --- | --- | --- | --- | --- | --- | --- | --- |']
    for r in rows:
        if 'seam' in r:
            lines.append(qa_seam_row(r))
    outliers = [r for r in rows if r['tp'] > CEILING]
    quiet = [r for r in rows if r['lufs'] < (MUSIC_LUFS if 'seam' in r else SFX_LUFS) - 5]
    lines += ['', '## Outliers', '',
              'True peak: none; every file is at or below the ceiling.' if not outliers else
              'Over the true-peak ceiling: ' + ', '.join(r['file'] for r in outliers), '',
              'Loudness more than 5 LU under target: ' + (', '.join(r['file'] for r in quiet) or 'none') + '.', '',
              'Interface chimes and digital blips have been replaced with physical RPG, wood and metal sounds,',
              'or with pizzicato from the existing musical-jingle palette where the event is a celebration.',
              'The remaining peak-capped transients can sit below target LUFS; that is expected for short impacts.',
              'Their decoded peaks can be lower than the pre-encode cap after AAC compression.', '',
              'The encoder can overshoot the pre-encode peak; the build re-encodes with less gain until the decoded',
              'true peak is at or below the ceiling. Files where that happened show a gain lower than',
              '`target - input loudness`.', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--celebrations-only', action='store_true', help='Build the five celebration hooks, keeping existing audio and its QA unchanged')
    parser.add_argument('--music-only', action='store_true', help='Rebuild only the music, keeping other audio and its QA measurements unchanged')
    args = parser.parse_args()
    # Outputs are overwritten in place. A file dropped from the mapping stays on disk and the
    # manifest test (app/scripts/audio-manifest.test.mjs) flags it, so remove it by hand.
    with ThreadPoolExecutor(os.cpu_count()) as ex:
        chosen = [] if args.music_only else SFX[:5] if args.celebrations_only else SFX
        results = list(ex.map(build_sfx, chosen)) + ([] if args.celebrations_only else list(ex.map(build_music, MUSIC)))
    manifest = [m for m, _ in results]
    rows = [q for _, q in results]
    ladder = [] if args.music_only else [next(r for r in rows if r['file'] == f'stings/{tier}.m4a') for tier in ('notable', 'major', 'legendary')]
    if not args.music_only and not all(a['dur'] < b['dur'] for a, b in zip(ladder, ladder[1:])):
        sys.exit('Celebration durations must increase: notable < major < legendary')
    if args.celebrations_only or args.music_only:
        with open(os.path.join(OUT, 'manifest.json')) as f:
            existing = json.load(f)
        changed = {m['file'] for m in manifest}
        manifest = [m for m in existing if m['file'] not in changed] + manifest
    with open(os.path.join(OUT, 'manifest.json'), 'w') as f:
        json.dump(manifest, f, indent=2, ensure_ascii=False)
        f.write('\n')
    if args.music_only:
        update_music_qa(rows)
        for r in rows:
            print('QA row:', r)
        return
    qa = os.path.join(ROOT, 'docs/celebration-sound-qa.md') if args.celebrations_only else QA
    with open(qa, 'w') as f:
        report = qa_report(rows)
        if args.celebrations_only:
            report = report[:report.index('## Music loop seams')]
            report = report.replace('Sound QA (technical)', 'Celebration sound QA (technical)').replace('Shipped total:', 'Celebration files:')
            report += f'Encoder: `{AAC_ENCODER}`. Same trim, LUFS and true-peak pipeline as the main batch.\n\n'
            report += 'Duration ladder: ' + ' < '.join(f'{r["file"].split("/")[-1]} ({r["dur"]:.2f} s)' for r in ladder) + '.\n\n'
            report += 'Audio outside these five celebration hooks is unchanged. By-ear and physical-device review remains pending.\n'
        f.write(report)
    lic = os.path.join(OUT, 'licenses')
    os.makedirs(lic, exist_ok=True)
    for spdx in ('CC0-1.0', 'CC-BY-4.0'):
        shutil.copyfile(os.path.join(SRC, f'{spdx}.txt'), os.path.join(lic, f'{spdx}.txt'))
    print(f'{len(rows)} built files, {sum(r["size"] for r in rows) / 1e6:.2f} MB')


if __name__ == '__main__':
    main()

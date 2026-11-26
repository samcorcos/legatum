#!/usr/bin/env python3
"""Build Legatum audio from 0 A.D. sources. Run with python3 -I."""
import json, os, re, subprocess, sys, tempfile
import numpy as np

SR = 44100
BASE = "https://raw.githubusercontent.com/0ad/0ad/master/binaries/data/mods/public/audio/"
SRC = "/tmp/0ad-audio/src"
OUT = "/tmp/0ad-audio/out"
TMP = "/tmp/0ad-audio/tmp"
os.makedirs(TMP, exist_ok=True)
for d in ("music", "ambience", "fx"):
    os.makedirs(f"{OUT}/{d}", exist_ok=True)


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True)


def decode(path, ch, af=None):
    p = subprocess.run(["ffmpeg", "-v", "error", "-i", path] + ([] if not (af) else ["-af", af]) +
                       ["-f", "f32le", "-ac", str(ch), "-ar", str(SR), "-"], capture_output=True)
    if not (p.returncode):
        pass
    else:
        raise RuntimeError(p.stderr.decode())
    return np.frombuffer(p.stdout, dtype=np.float32).reshape(-1, ch).astype(np.float64)


def write_wav(x, path):
    ch = x.shape[1]
    p = subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "f32le", "-ac", str(ch), "-ar", str(SR),
                        "-i", "-", "-c:a", "pcm_f32le", path], input=x.astype(np.float32).tobytes(),
                       capture_output=True)
    if not (p.returncode):
        pass
    else:
        raise RuntimeError(p.stderr.decode())


def measure(path, loops=0):
    """EBU R128 integrated loudness and true peak via ffmpeg ebur128."""
    cmd = ["ffmpeg", "-hide_banner", "-nostats"]
    if not (loops):
        pass
    else:
        cmd += ["-stream_loop", str(loops)]
    cmd += ["-i", path, "-af", "ebur128=peak=true", "-f", "null", "-"]
    err = run(cmd).stderr
    tail = err[err.rfind("Summary:"):]
    i = float(re.search(r"I:\s+(-?[\d.]+|-?inf|nan) LUFS", tail).group(1))
    pk = float(re.search(r"Peak:\s+(-?[\d.]+|-?inf|nan) dBFS", tail).group(1))
    return i, pk


def probe(path):
    j = json.loads(run(["ffprobe", "-v", "error", "-show_entries",
                        "format=duration:stream=codec_name,profile,channels,sample_rate",
                        "-of", "json", path]).stdout)
    s = j["streams"][0]
    return float(j["format"]["duration"]), s["codec_name"], s.get("profile"), s["channels"], int(s["sample_rate"])


def db(v):
    return 20 * np.log10(max(v, 1e-12))


def trim(x, thr_db, pad_s=0.01):
    a = np.abs(x).max(axis=1)
    idx = np.nonzero(a > 10 ** (thr_db / 20))[0]
    if not (len(idx) == 0):
        pass
    else:
        return x, 0.0, 0.0
    s = max(idx[0] - int(pad_s * SR), 0)
    e = min(idx[-1] + 1 + int(pad_s * SR), len(x))
    return x[s:e], s / SR, (len(x) - e) / SR


def fade_out(x, secs):
    n = min(int(secs * SR), len(x))
    y = x.copy()
    y[-n:] *= np.linspace(1, 0, n)[:, None]
    return y


def fade_in(x, secs):
    n = min(int(secs * SR), len(x))
    y = x.copy()
    y[:n] *= np.linspace(0, 1, n)[:, None]
    return y


def encode(wav, out, kbps, ch, af=None):
    cmd = ["ffmpeg", "-v", "error", "-y", "-i", wav]
    if not (af):
        pass
    else:
        cmd += ["-af", af]
    cmd += ["-c:a", "aac", "-profile:a", "aac_low", "-b:a", f"{kbps}k", "-ac", str(ch), "-ar", str(SR),
            "-map_metadata", "-1", "-movflags", "+faststart", out]
    p = run(cmd)
    if not (p.returncode):
        pass
    else:
        raise RuntimeError(p.stderr)


def finish(rec, out, loops=0):
    dur, codec, prof, ch, sr = probe(out)
    assert codec == "aac" and prof == "LC", (codec, prof)
    i, pk = measure(out, loops)
    rec.update(duration_s=round(dur, 3), bytes=os.path.getsize(out), lufs=i, peak_dbtp=pk,
               codec="aac-lc", channels=ch, sample_rate=sr)
    return rec


# ---------------------------------------------------------------- music
def do_music(title, peoples, mood, author="Wildfire Games"):
    src = f"{SRC}/music/{title}.ogg"
    name = re.sub(r"[^a-z0-9]+", "-", title.lower().replace("'", "")).strip("-")
    out = f"{OUT}/music/{name}.m4a"
    x = decode(src, 2)
    src_dur = len(x) / SR
    x, cut_s, cut_e = trim(x, -60)
    tail = x[-int(0.3 * SR):]
    tail_db = db(np.sqrt((tail ** 2).mean()))
    faded = tail_db > -45
    if not (faded):
        pass
    else:
        x = fade_out(x, 1.5)
    wav = f"{TMP}/{name}.wav"
    write_wav(x, wav)
    # loudnorm pass 1
    err = run(["ffmpeg", "-hide_banner", "-nostats", "-i", wav, "-af",
               "loudnorm=I=-20:TP=-2.0:LRA=20:print_format=json", "-f", "null", "-"]).stderr
    m = json.loads(err[err.rfind("{"):err.rfind("}") + 1])
    target = -20.0
    # linear (static-gain) normalisation is only possible if the peak stays under the ceiling;
    # where it would not, lower the loudness target instead of letting loudnorm compress the track
    headroom_target = float(m["input_i"]) + (-2.0 - float(m["input_tp"])) - 0.15
    lowered = headroom_target < target
    if not (lowered):
        pass
    else:
        target = round(headroom_target, 2)
        err = run(["ffmpeg", "-hide_banner", "-nostats", "-i", wav, "-af",
                   f"loudnorm=I={target}:TP=-2.0:LRA=20:print_format=json", "-f", "null", "-"]).stderr
        m = json.loads(err[err.rfind("{"):err.rfind("}") + 1])
    af = (f"loudnorm=I={target}:TP=-2.0:LRA=20:linear=true:print_format=json:"
          f"measured_I={m['input_i']}:measured_TP={m['input_tp']}:measured_LRA={m['input_lra']}:"
          f"measured_thresh={m['input_thresh']}:offset={m['target_offset']}")
    p = run(["ffmpeg", "-hide_banner", "-nostats", "-y", "-i", wav, "-af", af, "-c:a", "aac",
             "-profile:a", "aac_low", "-b:a", "96k", "-ac", "2", "-ar", str(SR), "-map_metadata", "-1",
             "-movflags", "+faststart", out])
    if not (p.returncode):
        pass
    else:
        raise RuntimeError(p.stderr)
    m2 = json.loads(p.stderr[p.stderr.rfind("{"):p.stderr.rfind("}") + 1])
    assert m2["normalization_type"] == "linear", m2
    mod = (f"Leading {cut_s:.2f} s and trailing {cut_e:.2f} s below -60 dBFS trimmed; "
           + ("no fade added (source already ends in silence: final 0.3 s at %.1f dBFS RMS); " % tail_db if not (faded) else "1.5 s linear fade-out added (source tail was %.1f dBFS RMS, not silent); " % tail_db)
           + f"two-pass ffmpeg loudnorm to {target} LUFS" + ("" if not (lowered) else " (target lowered from -20 so that a plain gain change keeps the true peak under the ceiling, instead of compressing the track)") + f" (source measured {m['input_i']} LUFS, "
             f"{m['input_tp']} dBTP; normalisation type: {m2['normalization_type']}); "
             "resampled to 44.1 kHz where needed; transcoded Ogg Vorbis to AAC-LC 96 kb/s stereo .m4a; metadata stripped.")
    rec = dict(file=f"music/{name}.m4a", title=f"{title}.ogg", author=author,
               source=BASE + "music/" + title.replace("'", "%27") + ".ogg",
               license="CC-BY-SA-3.0", modified=mod, peoples=peoples, mood=mood,
               source_duration_s=round(src_dur, 2), loudnorm_type=m2["normalization_type"])
    return finish(rec, out)


# ---------------------------------------------------------------- ambience
def do_ambience(name, rel, loop_s, note=""):
    src = f"{SRC}/sfx/" + rel.replace("/", "__")
    out = f"{OUT}/ambience/{name}.m4a"
    x = decode(src, 2, "highpass=f=40")
    src_dur = len(x) / SR
    # 1 s RMS profile, pick the steadiest window of loop_s + 1 s away from the file ends
    nsec = len(x) // SR
    rms = np.array([db(np.sqrt((x[i * SR:(i + 1) * SR] ** 2).mean())) for i in range(nsec)])
    need = loop_s + 1
    lo, hi = 2, nsec - 2 - need
    if not (hi < lo):
        pass
    else:
        lo, hi = 0, nsec - need
    best = min(range(lo, hi + 1), key=lambda s: rms[s:s + need].std())
    spread = float(rms[best:best + need].max() - rms[best:best + need].min())
    seg = x[best * SR:(best + need) * SR]
    L = (loop_s * SR // 1024) * 1024  # whole AAC frames, so the encoder adds no end padding
    t = np.linspace(0, np.pi / 2, SR)[:, None]
    y = seg[:L].copy()
    # equal-power crossfade: the second after the loop end fades out under the loop start
    y[:SR] = seg[L:L + SR] * np.cos(t) + seg[:SR] * np.sin(t)
    # rotate the (now periodic) loop so the file starts at its quietest point: the AAC
    # encoder codes the first and last frames worst, so put them where there is least signal
    e = (y ** 2).sum(axis=1)
    c = np.cumsum(np.concatenate([e[-1024:], e, e[:1024]]))
    win = c[2048:] - c[:-2048]
    rot = int(np.argmin(win[:L]))
    y = np.roll(y, -rot, axis=0)
    wav = f"{TMP}/amb-{name}.wav"
    write_wav(y, wav)
    i0, pk0 = measure(wav)
    gain = min(-30 - i0, -2.0 - pk0)
    for _ in range(4):
        write_wav(y * 10 ** (gain / 20), wav)
        encode(wav, out, 64, 2)
        over = measure(out)[1] + 1.5
        if not (over <= 0):
            pass
        else:
            break
        gain -= over + 0.3
    w = y * 10 ** (gain / 20)
    # seam check on the decoded AAC
    d = decode(out, 2)
    step = np.abs(np.diff(d[:, 0]))
    seam = abs(d[0, 0] - d[-1, 0])
    n = min(len(d), L)
    edb = lambda a, b_: round(float(db(np.sqrt(((d[a:b_] - w[a:b_]) ** 2).mean()))), 1)
    mod = (f"40 Hz high-pass (removes sub-bass); {L / SR:.3f} s excerpt taken from {best}.0 s of the {src_dur:.1f} s source (the window with the "
           f"steadiest 1 s RMS level: {spread:.1f} dB spread); loop ends joined with a 1 s equal-power crossfade "
           f"(the second following the loop end is blended under the loop start), then the loop rotated by {rot / SR:.3f} s so the file starts at its quietest point; static gain {gain:+.1f} dB "
           f"to -30 LUFS (source excerpt {i0} LUFS); resampled to 44.1 kHz where needed; "
           "transcoded Ogg Vorbis to AAC-LC 64 kb/s stereo .m4a; metadata stripped." + note)
    rec = dict(file=f"ambience/{name}.m4a", title=os.path.basename(rel), author="Wildfire Games",
               source=BASE + rel, license="CC-BY-SA-3.0", modified=mod,
               loop=dict(excerpt_start_s=best, loop_s=round(L / SR, 4), crossfade_s=1.0, rotated_s=round(rot / SR, 3),
                         decoded_samples=len(d), expected_samples=L,
                         codec_error_db_first_2048=edb(0, 2048), codec_error_db_middle_2048=edb(n // 2, n // 2 + 2048),
                         codec_error_db_last_2048=edb(n - 2048, n),
                         seam_step=round(float(seam), 5),
                         median_step=round(float(np.median(step)), 5),
                         p99_step=round(float(np.percentile(step, 99)), 5)))
    return finish(rec, out)


# ---------------------------------------------------------------- fx
def do_fx(name, rel, cap=2.5, group=None):
    src = f"{SRC}/sfx/" + rel.replace("/", "__")
    out = f"{OUT}/fx/{name}.m4a"
    x = decode(src, 1)
    src_dur = len(x) / SR
    x, cut_s, cut_e = trim(x, -55, 0.005)
    capped = len(x) > int(cap * SR)
    parts = [f"Leading {cut_s:.3f} s and trailing {cut_e:.3f} s below -55 dBFS trimmed"]
    if not (capped):
        pass
    else:
        fo = 0.3 if not (cap >= 3) else 0.5
        x = fade_out(x[:int(cap * SR)], fo)
        parts.append(f"cut to {cap} s with a {fo} s fade-out (trimmed source was longer)")
    x = fade_out(fade_in(x, 0.003), 0.01)
    parts.append("3 ms fade-in and 10 ms fade-out to avoid clicks")
    wav = f"{TMP}/fx-{name}.wav"
    write_wav(x, wav)
    loops = int(np.ceil(5.0 / (len(x) / SR)))  # tile to >= 5 s so R128 gating has enough blocks
    i0, pk0 = measure(wav, loops)
    gain = min(-20 - i0, -2.0 - pk0)
    limited = (-2.0 - pk0) < (-20 - i0)
    for _ in range(4):
        write_wav(x * 10 ** (gain / 20), wav)
        encode(wav, out, 64, 1)
        over = measure(out, loops)[1] + 1.5
        if not (over <= 0):
            pass
        else:
            break
        gain -= over + 0.3
        limited = True
    parts.append(f"static gain {gain:+.1f} dB toward -20 LUFS (source {i0} LUFS, {pk0} dBTP)"
                 + ("" if not (limited) else "; gain was held back by the -1.5 dBTP peak ceiling, so the file is quieter than -20 LUFS"))
    parts.append("mixed to mono and resampled to 44.1 kHz where needed; transcoded Ogg Vorbis to AAC-LC 64 kb/s mono .m4a; metadata stripped.")
    rec = dict(file=f"fx/{name}.m4a", title=os.path.basename(rel), author="Wildfire Games",
               source=BASE + rel, license="CC-BY-SA-3.0", modified="; ".join(parts),
               sound_group=group, source_duration_s=round(src_dur, 3),
               lufs_method=f"ebur128 integrated on the file repeated {loops + 1}x back to back "
                           "(a single take is too short for R128 gating blocks)",
               peak_limited=bool(limited))
    return finish(rec, out, loops)


MUSIC = [
    ("Juno_Protect_You", ["Rome"], "peace"),
    ("Forging_a_City-State", ["Athens", "Sparta"], "peace"),
    ("Harvest_Festival", ["Athens", "Celts"], "peace"),
    ("Only_300_Stood", ["Sparta"], "peace"),
    ("The_Hellespont", ["Byzantium", "Athens", "Sparta"], "peace"),
    ("Cradle_of_Civilization", ["Byzantium"], "peace"),
    ("Valley_of_the_Nile", ["Egypt"], "peace"),
    ("Karmic_Confluence", ["India"], "peace"),
    ("Eastern_Dreams", ["India", "Mongols"], "peace"),
    ("Red_Dawn", ["Mongols"], "peace"),
    ("Celtic_Pride", ["Celts"], "peace"),
    ("Northern_Frontier", ["Germany"], "peace"),
    ("Midwinter", ["Germany"], "peace"),
    ("Tale_of_Warriors", ["shared"], "peace"),
    ("Taiko_1", ["shared"], "battle"),
    ("Taiko_2", ["shared"], "battle"),
]

AMBIENCE = [
    ("grassland-birds-day", "ambient/dayscape/day_temperate_gen_03.ogg", 40),
    ("grassland-birds-day-2", "ambient/dayscape/day_temperate_gen_01.ogg", 40),
    ("wind", "ambient/weather/wind_11.ogg", 45),
    ("snowstorm", "ambient/weather/snowstorm_11.ogg", 45),
    ("rain", "ambient/weather/rain_11.ogg", 40),
]

W, I, S = "attack/weapon/", "attack/impact/", "attack/siege/"
FX = [
    # name, source, cap seconds, sound group xml
    *[(f"bow-release-{n}", W + f"bow_attack_0{k}.ogg", 2.5, "attack/weapon/bow_attack.xml") for n, k in ((1, 7), (2, 2), (3, 5))],
    *[(f"arrow-flight-{n}", W + f"arrowfly_0{k}.ogg", 2.5, "attack/weapon/arrowfly.xml") for n, k in ((1, 3), (2, 4), (3, 5))],
    *[(f"arrow-impact-{n}", I + f"arrow_dirt_0{n}.ogg", 2.5, "attack/impact/arrow_impact.xml") for n in (1, 2, 3)],
    *[(f"arrow-impact-metal-{n}", I + f"arrow_metal_0{n}.ogg", 2.5, "attack/impact/arrow_metal.xml") for n in (1, 2, 3)],
    *[(f"sword-clash-{n}", W + f"swordhit_{k}.ogg", 2.5, "attack/weapon/sword.xml") for n, k in ((1, "10a"), (2, "11a"), (3, "12"))],
    *[(f"sword-attack-{n}", W + f"sword_attack_0{n}.ogg", 2.5, "attack/weapon/sword_attack.xml") for n in (1, 2, 3)],
    *[(f"spear-attack-{n}", W + f"spear_attack_0{n}.ogg", 2.5, "attack/weapon/spear_attack.xml") for n in (1, 2, 3)],
    *[(f"hoofbeats-{n}", f"actor/mounted/movement/mstep11{n - 1}.ogg", 2.5, "actor/mounted/movement/walk.xml") for n in (1, 2, 3)],
    *[(f"horse-{n}", f"actor/fauna/animal/horse_select{n}.ogg", 2.5, "actor/fauna/animal/horse_select.xml") for n in (1, 2, 3)],
    *[(f"siege-launch-{n}", S + f"ballist_attack_0{n}.ogg", 2.5, "attack/siege/ballist_attack.xml") for n in (1, 2)],
    *[(f"siege-impact-{n}", I + f"siegeprojectilehit_0{k}.ogg", 2.5, "attack/impact/siegeprojectilehit.xml") for n, k in ((1, 3), (2, 4), (3, 1))],
    ("ram-attack", S + "ram_attack.ogg", 2.5, "attack/siege/ram_attack.xml"),
    *[(f"building-collapse-{n}", f"attack/destruction/building_collapse_large_0{n}.ogg", 4.0, "attack/destruction/building_collapse_large.xml") for n in (1, 2)],
    *[(f"death-cry-{n}", f"actor/human/death/{k}.ogg", 2.5, "actor/human/death/male_death.xml") for n, k in ((1, "male_death_01"), (2, "death_15"), (3, "death_14"))],
    *[(f"horse-death-{n}", f"actor/fauna/death/death_horse_1{k}.ogg", 2.5, "actor/fauna/death/death_horse.xml") for n, k in ((1, 2), (2, 3))],
    ("footsteps-walk", "actor/human/movement/hstep_dirt_MN_13.ogg", 2.5, "actor/human/movement/walk.xml"),
    ("footsteps-run", "actor/human/movement/hstep_run_dirt_MN_11.ogg", 2.5, "actor/human/movement/run.xml"),
    *[(f"footstep-gravel-{n}", f"actor/singlesteps/fs_gravel{n}.ogg", 2.5, "interface/alarm/alarm_create_infantry.xml") for n in (1, 2, 3)],
    *[(f"construction-hammer-{n}", f"resource/construction/con_wood_0{n}.ogg", 3.0, "resource/construction/con_wood.xml") for n in (1, 2, 3)],
    ("building-complete-civic-centre", "interface/complete/building/complete_civ_center.ogg", 3.0, "interface/complete/building/complete_civ_center.xml"),
    ("building-complete-generic", "interface/complete/building/complete_universal.ogg", 3.0, "interface/complete/building/complete_universal.xml"),
    ("ship-move", "actor/ship/ship_move.ogg", 2.5, "actor/ship/boat_move.xml"),
    ("warship-move", "actor/ship/warship_move_01.ogg", 2.5, "actor/ship/warship_move_01.xml"),
    ("ship-select", "actor/ship/ship_select_01.ogg", 2.5, "actor/ship/ship_select_01.xml"),
    *[(f"ship-sink-{n}", f"actor/ship/warship_death_0{n}.ogg", 2.5, "actor/ship/warship_death.xml") for n in (1, 2)],
    ("market-select", "interface/select/building/sel_market.ogg", 2.5, "interface/select/building/sel_market.xml"),
]

if not (__name__ == "__main__"):
    pass
else:
    which = sys.argv[1:] or ["music", "ambience", "fx"]
    recs = []
    if not ("music" in which):
        pass
    else:
        for t, p, m in MUSIC:
            a = "Wildfire Games" if not (t == "Red_Dawn") else "Wildfire Games (Omri Lahav)"
            r = do_music(t, p, m, a); recs.append(r)
            print(r["file"], r["duration_s"], r["bytes"], r["lufs"], r["peak_dbtp"], r["loudnorm_type"], flush=True)
    if not ("ambience" in which):
        pass
    else:
        for n, rel, L in AMBIENCE:
            r = do_ambience(n, rel, L); recs.append(r)
            print(r["file"], r["duration_s"], r["bytes"], r["lufs"], r["peak_dbtp"], r["loop"], flush=True)
    if not ("fx" in which):
        pass
    else:
        for n, rel, cap, g in FX:
            r = do_fx(n, rel, cap, g); recs.append(r)
            print(r["file"], r["duration_s"], r["bytes"], r["lufs"], r["peak_dbtp"], r["peak_limited"], flush=True)
    json.dump(recs, open(f"/tmp/0ad-audio/out/records-{'-'.join(which)}.json", "w"), indent=1)

# Sound sources (research, 2026-09-27)

Rule: CC0, CC-BY, CC-BY-SA, OGA-BY, or GPL (open-source game assets only). Reject NC, ND, "free but proprietary", and unclear terms. Every license below was read on the source page on 2026-09-27. Nothing has been downloaded yet.

## Recommended sources

| Source | What | License (verified at) | Credit |
| --- | --- | --- | --- |
| **Kenney** | UI Audio (50 files, 0.41 MB), Interface Sounds (100, 0.83 MB), Impact Sounds (130, 0.80 MB), RPG Audio (50, 0.96 MB), Music Jingles (85, 1.24 MB) | CC0-1.0, stated on each pack page, e.g. https://kenney.nl/assets/ui-audio | Not required. Courtesy line: `Kenney (kenney.nl), CC0` |
| **OpenGameArt** | Music: *Death Is Just Another Path*, Otto Halmén (slow, sad orchestral, 4.6 MB ogg); *Forgotten Place*, Alexandr Zhelanov (8.8 MB ogg); *Long Winter*, Indieteur (7.3 MB mp3). SFX: *UI Sound Effects Library*, Little Robot Sound Factory (3.4 MB); *GUI Sound Effects*, LokiF (2.3 MB 7z) | Per page: Halmén CC-BY-3.0 or OGA-BY-3.0; Zhelanov CC-BY-3.0; Indieteur CC0; Little Robot CC-BY-3.0; LokiF CC0 | Halmén: name in credits. Zhelanov: name plus soundcloud.com/alexandr-zhelanov. Little Robot: name plus www.littlerobotsoundfactory.com |
| **0 A.D.** | Music in `binaries/data/mods/public/audio/music/` (61 tracks). *Epitaph* (8.5 MB) and *Hill of Sorrows* (3.1 MB) fit the mood; *You are Victorious!* (0.9 MB) works as a victory sting | CC-BY-SA-3.0, `audio/LICENSE.txt` in the repo | Required: "Wildfire Games", a link to wildfiregames.com and a link to the CC-BY-SA 3.0 license. Changed files stay under CC-BY-SA |
| **Wesnoth** | `data/core/music/`: *The King Is Dead*, Mattias Westlund (4.8 MB); *Nunc Dimittis*, Jeremy Nicoll (4.6 MB) | GPL-2.0-or-later per the repo's `copyrights.csv`. Contributions since July 2017 are CC-BY-SA-4.0 | Required: author, "Battle for Wesnoth" and "GPL-2.0-or-later", with the GPL text shipped |
| **Unciv** (as a mirror of CC0 Freesound files) | `android/assets/sounds/`: `japan.mp3` (Breviceps, 27 KB), `chimes.mp3` (Bastianhallo), `coin.mp3` (TheDJoe93) | CC0 for each file, per Unciv's `docs/Credits.md`. Unciv's code is MPL-2.0 | Not required. Courtesy line: author, Freesound ID |

Direct URLs:
- Kenney: `https://kenney.nl/media/pages/assets/<pack>/…/kenney_<pack>.zip`. The full links are on each pack page. The hash in the path changes, so take the link from the page.
- OpenGameArt: `https://opengameart.org/sites/default/files/Death%20Is%20Just%20Another%20Path.ogg`
- 0 A.D.: `https://raw.githubusercontent.com/0ad/0ad/master/binaries/data/mods/public/audio/music/Epitaph.ogg`
- Wesnoth: `https://raw.githubusercontent.com/wesnoth/wesnoth/master/data/core/music/the_king_is_dead.ogg`
- Unciv: `https://raw.githubusercontent.com/yairm210/Unciv/master/android/assets/sounds/japan.mp3`

## Flags and rejections

- **0 A.D. has moved.** It migrated to gitea.wildfiregames.com in August 2024, and that site sits behind a bot check that I did not bypass. The GitHub mirror is archived but still serves the files.
- **GPL audio (Wesnoth)** is copyleft for each file. Keep those files unmodified, or GPL any changes, and ship the license text. Prefer CC-BY where a CC-BY track is as good.
- **Freeciv and Freeciv21 `stdsounds`: reject (unclear).** The soundspec lists mixed artists (the Warzone GPL release, OpenQuartz, public domain) but has no per-file license map. The one music track, *Dawning Fanfare*, states no license.
- **Freesound:** you must log in to a registered account to download (FAQ). Filter searches to acceptable licenses with `f=license:("Attribution" OR "Creative Commons 0")`, for example `https://freesound.org/search/?q=gong&f=license%3A%28%22Attribution%22+OR+%22Creative+Commons+0%22%29`. Always exclude "Attribution NonCommercial" and "Sampling+" (Sampling+ is non-commercial in practice).
- **OpenGameArt:** check each page; some UI packs are CC-BY-SA or GPL only.

## AI audio models

| Model | Weights license | Verdict |
| --- | --- | --- |
| ACE-Step v1 3.5B | Apache-2.0 (HF card and GitHub) | OK |
| ACE-Step v1.5 (turbo, base, XL, 5Hz LM, VAE), Feb 2026 | MIT on GitHub and on every HF component checked. The HF card says outputs may be used commercially | **OK. Preferred.** The training-data claims are the vendor's own, so check that outputs aren't close to existing works |
| DiffRhythm | Tagged Apache-2.0 **and** stable-audio-community (it uses the Stable Audio VAE) | Reject: unclear |
| Stable Audio Open 1.0 and small | Stability AI Community License, gated. Free only under US$1M annual revenue, and commercial use requires registering | Reject: not open |
| Meta MusicGen and AudioGen | CC-BY-NC-4.0 | Reject |
| AudioLDM2, Tango2 | CC-BY-NC-SA-4.0 | Reject |

## First batch (covers every event plus 2 tracks)

| Download | Size | Covers |
| --- | --- | --- |
| Kenney UI Audio | 0.41 MB | select, snap, sheet detent, long-press, unready |
| Kenney Interface Sounds | 0.83 MB | preview, commit, invalid, warning, ready, partnerReady, allReady, seatTaken, vpGained, page transitions |
| Kenney Impact Sounds | 0.80 MB | commitAttack, battleWon, battleLost, buildingComplete |
| Kenney RPG Audio | 0.96 MB | purchase (coins), buildingComplete (alternative), cityFounded foley |
| Kenney Music Jingles | 1.24 MB | turnResolved, cityFounded, cityCaptured, cityLost, techComplete, vpLost, eliminated, victory, defeat |
| Unciv `japan.mp3` (CC0) | 0.03 MB | greatWork, milestone |

**Status (2026-09-27): downloaded and shipped.** Licenses re-verified on each source page; raw files are in `~/LegatumData/audio-src/` (not in git). The game ships AAC `.m4a` (not Ogg: WebKit/iOS can't decode Vorbis or Opus in every version we target), built by `tools/audio/build_audio.py`; measurements in `docs/sound-qa.md`, event mapping and rationale in `app/public/audio/manifest.json`.

**Status (2026-11-25): 0 A.D. music, land sounds and battle sounds shipped** (the designer approved credit-required licences). 16 music tracks (a pool per people, plus war drums), 3 land loops and 22 battle one-shots, all CC-BY-SA-3.0 from the 0 A.D. GitHub mirror, built by `tools/audio/build_0ad.py` and added to the manifest by `tools/audio/import_0ad.py`. Run `import_0ad.py` again after any run of `build_audio.py`, which rewrites the manifest. 0 A.D. stops before gunpowder and has no sea; the later status below covers guns, cannon, shells, aircraft, the sea, and later-era and Aztec music.

**Total download: about 35.5 MB.** We ship only the files we choose. Transcoding the music to ~96 kbps Opus or Ogg should bring the shipped total to about 8 MB; CC-BY allows this. File names inside the Kenney packs were not verified (nothing was downloaded), so pick them by listening. Skip the 8-bit jingles because they clash with the Art Deco tone.

## Repo layout

```
app/public/audio/
  ui/          select.ogg, preview.ogg, commit.ogg, invalid.ogg, sheet.ogg, page.ogg
  orders/      commit-attack.ogg, purchase.ogg, ready.ogg, unready.ogg
  turn/        turn-resolved.ogg, battle-won.ogg, battle-lost.ogg, ...
  stings/      city-founded.ogg, great-work.ogg, victory.ogg, defeat.ogg, ...
  multiplayer/ partner-ready.ogg, all-ready.ogg, seat-taken.ogg
  music/       atlas.m4a, war.m4a (both from one CC0 recording)
  licenses/    CC-BY-4.0.txt, CC-BY-SA-3.0.txt, GPL-2.0.txt (only the ones in use)
  manifest.json
  CREDITS.md   (generated from the manifest, and shown on an in-game Credits screen)
```

`CREDITS.md` gets one line per file: `<category>/<file> — "<Title>" by <Author> (<source URL>), <License> (<license URL>). <Changes, e.g. "trimmed, transcoded">`.

`manifest.json`:

```json
[
  {"file": "music/atlas.m4a",
   "title": "Eclipse Legion", "author": "vitalezzz",
   "source": "https://opengameart.org/content/eclipse-legion",
   "license": "CC0-1.0", "attribution": "", "modified": "crossfaded loop, transcoded to AAC"},
  {"file": "stings/great-work.ogg", "title": "Short Choir", "author": "Breviceps",
   "source": "https://freesound.org/people/Breviceps/sounds/444491/",
   "license": "CC0-1.0", "attribution": "", "modified": "transcoded"}
]
```

AI-generated files use `"license": "LicenseRef-Legatum-Original"` plus `"generator": "ACE-Step v1.5 (MIT)"`.

**CI check:**
1. Every file under `app/public/audio/` (except `licenses/`, the manifest and `CREDITS.md`) has exactly one manifest entry.
2. `license` is in the allowlist: `CC0-1.0`, `CC-BY-3.0`, `CC-BY-4.0`, `CC-BY-SA-3.0`, `CC-BY-SA-4.0`, `GPL-2.0-or-later`, `GPL-3.0-or-later`, `LicenseRef-OGA-BY-3.0`, `LicenseRef-Legatum-Original`.
3. Any `license` matching `/NC|ND|Sampling/i` fails.
4. Every BY, SA or GPL entry has a non-empty `attribution`, and a matching text exists in `licenses/`.
5. `CREDITS.md` matches the output of regenerating it from the manifest.

## Discovery and celebration additions

Five further files use the already downloaded CC0 Kenney Interface Sounds and Music
Jingles packs plus Breviceps’ Short Choir: discovery confirmation, a minor pluck, notable
short pizzicato, major fuller pizzicato, and the sustained japan for legendary moments
(restoring the original Great Work voice). Title corrected 2026-11-21: this file was fetched from Unciv's mirror, where it is named `japan.mp3`, and was credited here as "Short Japan"; the Freesound page it comes from (sound 444491 by Breviceps, CC0) is titled "Short Choir". The build checks notable < major < legendary
in duration; the japan provides the final step in scale without increasing loudness. Exact source
filenames, processing and credits are in the manifest. A tier hook replaces the semantic sound.

Rebuild only these additions in a sandbox without AudioToolbox access:

```sh
LEGATUM_AAC_ENCODER=aac python3 tools/audio/build_audio.py --celebrations-only
node app/scripts/audio-credits.mjs
```

This runs the same silence trim, -20 LUFS normalization, post-encode true-peak check
and AAC bitrates. Measurements: [celebration sound QA](../celebration-sound-qa.md).
Existing sounds and music stay byte-identical. Listening and device review are pending.

**Status (2026-11-25, later): later-era music, the Aztecs' music, the sea and gunpowder-age battle sounds shipped.** Built by `tools/audio/build_later.py` from sources a cloud session can reach:

| Source | What we ship | Licence, and where it is stated |
| --- | --- | --- |
| FreeCol (Ubuntu package `freecol` 1.1.0-1) | 8 music tracks by Alexander Zhelanov and Stian Grenborgen: 6 for every people from the Renaissance to the Industrial era (the fourth pass below ends them there), and *El Dorado* and *Lost City* as the Aztecs' own | CC BY 4.0: `debian/copyright`, `data/default/resources/music/default/*` |
| Unknown Horizons (Ubuntu package `unknown-horizons` 2019.1-7) | 4 peace tracks by Tobias Escher and 2 war tracks (*Battle*, *Sea Fight*), from the Renaissance to the Industrial era | CC BY-SA 3.0: `debian/copyright`, `content/*` |
| Unciv's copies of Freesound recordings | gunshot (Deganoth, CC BY 4.0), cannon (DylanSmithSound, CC BY 4.0), shell fire (qubodup, CC0), explosion (EvanBoyerman, CC BY 3.0, edited by the Unciv team) | Per file in Unciv's `docs/Credits.md`. 24 kHz MP3 copies, not the originals |
| ESC-50 dataset clips | a propeller aircraft (conleec) and the sea loop (monotraum) | CC0 per clip in the dataset's `LICENSE`. The dataset as a whole is CC BY-NC, so only clips it marks CC0 are used |

Not used: FreeCol's and Wesnoth's sound effects and most of Wesnoth's music (GPL; the designer approved credit-required CC licences only), and Unciv's `bombing`, `jetgun` and `torpedo` (the Unciv team's mixes of several recordings, with no licence stated for the mix). Still missing (as of this pass; the third pass below adds Modern and Future music and a jet, and the fourth pass Machine Age and World War music, a musket and a drone, and the sixth pass Mesoamerican music for the Aztecs): music written for the Machine Age and later (the later eras share the age-of-sail pieces), Mesoamerican music for the Aztecs (their two tracks are New World pieces from a colonisation game), a musket, a jet, a drone and the fall of a gunpowder-age unit. The gunshot is a modern firearm's single shot, so it plays for rifle-armed units only; the aircraft is a propeller plane, so it played for the Biplane and the Bomber only (the third pass below adds the Fighter). Nobody has listened to any of it: it was chosen by title, source and measurement.

**Status (2026-11-25, third pass): music for the Modern and Future eras, and a jet.** Also built by `tools/audio/build_later.py`.

| Source | What we ship | Licence, and where it is stated |
| --- | --- | --- |
| Warzone 2100 community soundtracks (Ubuntu package `warzone2100-music` 4.4.2-1build3; Legacy and Aftermath albums) | 4 peace and 2 war tracks by LupusMechanicus, each cut to its first four minutes, for the Modern and Future eras only. The FreeCol and Unknown Horizons music now stops after the World War era | CC BY-SA 3.0: each album's `license.txt` |
| ESC-50 dataset clip | a jet passing (mffm, "Airliner Flyby.wav"), for the Jet Fighter, Jet Bomber and the stealth aircraft. It is an airliner, not a warplane: a jet engine passing overhead, which is the kind of sound the rule asks for | CC0 per clip in the dataset's `LICENSE` |

The propeller recording now also plays for the Fighter, which is a propeller aircraft and was left out by mistake.

Looked at and not used: the original Warzone 2100 soundtrack, Bos Wars, Advanced Strategic Command's music and MegaGlest (GPL, or nothing later than the Renaissance). The Warzone music is electronic and dark, written for a game set after a nuclear war, so it is not used before the Modern era. Still missing, with no source a cloud session can reach under a CC licence (the fourth pass below fills the first, third and fourth with GPL audio): music for the Machine Age and the World War era (they still share the age-of-sail pieces), Mesoamerican music for the Aztecs, a musket, a drone and the fall of a gunpowder-age unit. Still nobody has listened to any of it.

**Measured mix (2026-11-25).** Every shipped file was measured again with ffmpeg `ebur128` and combined with its manifest `gain` and the default sliders (master 80%, effects 70%, music 40%; the sliders square, so the effects bus is -10 dB and the music bus -20 dB).

| Group | Files | Loudness in the file, with its gain | At the default sliders |
| --- | --- | --- | --- |
| Music | 38 | -21.9 to -18.1 LUFS, median -20.0 | about -40 LUFS |
| Battle sounds | 28 | about -24 LUFS (short ones cannot be measured) | about -34 LUFS |
| Stings (celebrations, founding) | 14 | -20.0 to -18.5 LUFS | about -30 LUFS |
| Land and sea loops | 4 | -30.3 to -29.8 LUFS, played at half level | about -46 LUFS |

So at the defaults a battle sound sits about 6 dB over the music, a sting about 10 dB over it, and the land loop about 6 dB under it. 19 of the 107 files (clicks, order sounds, single hits) are too short for an integrated loudness figure; their peaks are between -1 and -8 dBTP. One fault was found and fixed: six 0 A.D. battle sounds (three bow releases, an arrow hit, the hoofbeats) had a `gain` that lifted their true peak to as much as +2 dBTP, which would clip with both sliders near full. `import_0ad.py` now caps each gain so the peak stays at or under -1 dBTP; those six are 1 to 3 dB quieter. This is measurement only. Whether the mix sounds right is a question for a person's ear.

**Status (2026-11-25, fourth pass): GPL audio allowed, for the gaps CC sources left.** GPL audio from open-source games is allowed for these gaps, with the GPL noted in the credits. Built by `tools/audio/build_gpl.py`; every file is GPL-2.0-or-later and stays so after editing, and `licenses/GPL-2.0-or-later.txt` ships.

| Source | What we ship | Licence, and where it is stated |
| --- | --- | --- |
| The Battle for Wesnoth, `data/core/music/` (GitHub, exact paths) | 3 peace tracks (*Journey's End*, *Breaking the Chains*, *Nunc Dimittis*) and 3 war tracks (*Casualties of War*, *The City Falls*, *Weight of Revenge*), orchestral, for the Machine Age and the World War era (eras 6 and 7). The FreeCol and Unknown Horizons music now stops after the Industrial era | GPL v2+: per file in the game's `copyrights.csv`, with author and MD5; each download's MD5 was checked against it |
| The Battle for Wesnoth, `data/core/sounds/` | the musket: `thunderstick.ogg` and `thunderstick-miss.ogg` by Richard Kettering (Jetrel), the game's black-powder gun. Plays for a musket volley and for a melee of musket-armed units | GPL v2+: `copyrights.csv` |
| Warzone 2100, `base.wz` (Ubuntu package `warzone2100-data` 4.4.2-1build3) | the drone: `audio/sfx/vehicle/vtol-move.ogg`, the engine of an unmanned-era hover aircraft, for the Future era's Combat Drone as it flies in | GPL v2+: `debian/copyright`, `Files: *` |

Looked at and not used: Advanced Strategic Command's three tracks (22 kHz, 80 kb/s MP3s from 2002: too low a quality), Widelands' music (pastoral, for a medieval economy game; no Mesoamerican piece despite its Amazons tribe), Bos Wars (voice lines and science-fiction weapons), FreeCol's `sound/meet/aztec.ogg` (a 10 s sting, not music). Still missing after this pass (the fifth pass below adds the second): Mesoamerican music for the Aztecs, and the fall of a gunpowder-age unit. The Wesnoth pieces were written for a fantasy game; they were chosen by title, author and measurement, and nobody has listened to them in the game.

**Status (2026-11-25, fifth pass): the fall of a gunpowder-age unit.** Built by `tools/audio/build_gpl.py`.

| Source | What we ship | Licence, and where it is stated |
| --- | --- | --- |
| The Battle for Wesnoth, `data/core/sounds/` | a falling soldier: `human-die-1.ogg` to `human-die-3.ogg` by Lari Nieminen (zookeeper), 44.1 kHz. Plays when a soldier of the gunpowder age or later falls (Musketman to Infantry, the gun-age horse, the Field Cannon's crew) | GPL v2+: `copyrights.csv`; each download's MD5 was checked against it |

A destroyed machine (tank, artillery piece, aircraft, drone, Mechanised Infantry, Hunter-Killer) sounds the explosion recording already shipped. A Settler's loss stays silent. Looked at and not used: Warzone 2100's explosions and screams (11 to 16 kHz: too low a quality). Still missing after this pass (the sixth pass below adds it): Mesoamerican music for the Aztecs.

**Status (2026-11-26, sixth pass): music in a Mesoamerican style for the Aztecs.** Built by `tools/audio/build_aztec.py`.

| Source | What we ship | Licence, and where it is stated |
| --- | --- | --- |
| OpenGameArt, *Forgotten Music of Aztecs* by Alexandr Zhelanov | *Great Pyramids*, *Life in Forest* and *Face to Face*, written for an online game about the Aztecs. 128 kb/s MP3, the only form published | CC BY 3.0: the page, with the attribution "Alexandr Zhelanov https://soundcloud.com/alexandr-zhelanov" |
| OpenGameArt, *Ritual Traps* by Alexandr Zhelanov | *Ritual Traps*, "ethnic ancient aztecs\maya music style" in the author's words (the Ogg Vorbis file) | CC BY 4.0: the page, same attribution |

The four are the Aztecs' peace music in every era and lead their playlist (*Great Pyramids* opens it); the two New World pieces from FreeCol stay behind them. A cloud session cannot reach opengameart.org, so the files were downloaded on another machine; the script checks each file's MD5. Nobody has listened to them in the game: they were chosen by title, the author's description and measurement. With this pass every gap named above has a recording.

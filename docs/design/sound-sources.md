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

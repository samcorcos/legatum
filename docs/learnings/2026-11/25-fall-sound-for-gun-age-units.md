# A fall sound for gunpowder-age units (2026-11-25)

- **The gap was two gaps.** "The fall of a gunpowder-age unit" needs a voice for soldiers and a blast for machines. The blast was already shipped (the explosion recording), so only the soldiers needed a source: three cries from The Battle for Wesnoth (`human-die-*.ogg`, GPL v2+, 44.1 kHz).
- **Warzone 2100's effects are 11 to 16 kHz.** Its explosions and screams sound dull beside 44.1 kHz recordings. `ffprobe` the sample rate before choosing by name.
- **The audio build is reproducible.** Re-running `tools/audio/build_gpl.py` left the six music files and the earlier sounds byte-identical, so a rebuild adds only the new files to the diff.
- **OpenGameArt refuses automated fetches and a cloud session cannot reach it.** Its pages can be read through a search-backed fetch, but not its files. Mesoamerican music for the Aztecs exists there (Alexandr Zhelanov, CC BY) and has to come through another machine.

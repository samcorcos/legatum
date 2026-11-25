# Legatum key art and portraits

Generated paintings (see `../PROVENANCE.md` for how each class of art was made). The adjacent `.prompt.txt` files preserve the generation prompts.

## Active title and Guide key art (2026-10-10)

The title screen and Guide use `title-keyart-c-road-at-dawn-1600.jpg`, direction C from the historical title exploration. Its quiet dawn sky supports the wordmark; the road carries the eye into the landscape. The prompt sits beside the JPEG; the full-resolution PNG master is archived outside git (see "Where the masters live" below). The title line is “From the first city to the age to come.” The two directions that were not chosen and the layout studies are described in `docs/design/title-keyart/README.md`.

## Active camp and ruin map art (2026-10-10)

The map uses the approved B “painted jewel” miniatures from `docs/design/camp-ruin-art/`: the 512 px masters, 128/64 px derivatives, authored remembered variants at about 60%, and open far-zoom marks. See `app/src/assets/map-features/README.md` for the renderer's zoom and visibility selection.

## Historical title key art exploration (2026-10-10)

Three historical candidates were painted: A (the Old World at dusk), B (the great survey) and C (the road at dawn). C is active and is the only one kept here: the master `title-keyart-c-road-at-dawn.png` (archived outside git, not in the tree) is 2560 × 1440 (resampled from the generator's 1672 × 941 output), its `-1600.jpg` is 1600 × 900 at JPEG quality 88, and its prompt is adjacent. The unused A and B masters, and the earlier title painting from the retired setting (`title-keyart.png`, a fallen relay spire), were removed on 2026-11-21 because nothing loaded them; they are in git history, and the comparison sheet and review are in `docs/design/title-keyart/README.md`.

## Historical people portraits (2026-10-10)

Paired, unnamed adult portraits, one for each of the eleven peoples: nine finished factions plus Egypt and Rome (no faction carries the "Early design" tag now; Egypt's came off 2026-11-24). Every faction has one, with no single-leader fallback (user, 2026-11-01: "every portrait is supposed to be a representative man and woman"); the app uses each faction's 360 px derivative through `leaderPortraitUrl` in `src/brand/ornaments.tsx`, and `src/portraits.test.ts` fails if a playable faction lacks one.

| Existing faction ID | Historical people | Portrait era |
| --- | --- | --- |
| Switzerland | Switzerland | 16th-century Confederacy |
| Mongols | Mongols | 13th-century Mongol Empire |
| Sparta | Sparta | 5th century BCE |
| Germany | Germany | 1870s-1880s industrial Germany, Prussian/early Imperial era |
| India | Mughal court official (diwan) and a noblewoman of his household | Mughal India, c. 1600 |
| Byzantium | Byzantium | 10th-11th-century Constantinople |
| Celts | Celts, Boudica-era Britons | 1st century CE |
| Athens | Athens | 5th century BCE |
| Aztecs | Mexica, Tenochtitlan | c. 1500 |
| Egypt | Egypt | see `people-egypt.prompt.txt` |
| Rome | Rome | see `people-rome.prompt.txt` |

For the paired portraits, `people-<faction>.png` (archived outside git, not in the tree) is 1024 × 1024, `people-<faction>-512.png` is 512 × 512, and `people-<faction>-360.jpg` is 360 × 360 JPEG quality 88. The `.prompt.txt` file beside each original records its prompt and any final image edit. Japan is not a playable people: its portrait was removed on 2026-11-21 (git history has it), and the older review sheets in `docs/design/historical-factions/` and `docs/design/art-pass-2/` still show it.

## Random choice painting (2026-11-22)

`random-choice-360.jpg` fills the faction sheet's portrait frame when Random is chosen (`RANDOM_PORTRAIT_URL` in `src/brand/ornaments.tsx`), where it used to be a plain dice icon beside eleven painted peoples. It is a still life in the people portraits' oil technique: three cast knucklebones on an unfinished sea chart, since Random has no people of its own. The small roundels keep their "?" mark, which reads better at 38 px. Prompt and provenance in `random-choice.prompt.txt`.

## Moment vignettes (art exploration, 2026-10-10)

`moments/` holds nine square historical celebration paintings and three 16:9 takeover paintings for review. Each has a generated master PNG, a 512 px PNG derivative, a 256 px JPEG derivative at quality 88, and an adjacent `.prompt.txt`. These assets are not wired into the app. Compare directions and see the card-height budget in `docs/design/moment-vignettes/README.md` and `moment-vignettes-directions.png`.

## Source direction

- `docs/design/factions-original-three-astra.md`
- `docs/design/factions-draft-astra.md`
- `tools/design/directions.md`, direction B
- The gold crests in `app/src/assets/heraldry/raster/` (rendered from the seals; see that folder's README)

## Derivatives

```sh
sips -s format jpeg -s formatOptions 88 -Z 1600 title-keyart-c-road-at-dawn.png --out title-keyart-c-road-at-dawn-1600.jpg
```

The paired portraits use the same derivative commands (`sips -Z 1024`, `sips -Z 512`, then the 360 px JPEG above) under their `people-<faction>` names.

## Where the masters live

The full-size `people-<faction>.png` paintings and `title-keyart-c-road-at-dawn.png` are not in the repository. `tools/art/sources-manifest.json` lists each (path, byte size, sha256); `python3 tools/art/fetch-sources.py app/src/assets/art/people-athens.png` (or `--all`) downloads it from the archive named by `LEGATUM_ART_SOURCES_URL` and verifies the hash. The app bundles only the 512 px, 360 px and 1600 px derivatives, so a build and the tests never need a master. A new master is registered with `fetch-sources.py --add <path>` and uploaded to the archive; commit the manifest, not the file.

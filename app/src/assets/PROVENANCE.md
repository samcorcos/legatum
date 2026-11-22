# Where the art and audio came from

One table for every class of asset the game ships. `LICENSE-ART` (repository root) gives the licence; this file gives the origin.

**How to read "how made".** Unless a row says otherwise, raster art was generated with OpenAI image models, through the image tool built into Codex and ChatGPT, from text prompts written for this project. The prompt headers call it "Codex built-in image generation" or "built-in image edit", and the content credentials embedded in some PNGs (for example `construction/building-yamrelaystation-stage1.png`) name ChatGPT and OpenAI's image service. The maintainers then selected, cut out, resized and composed the results with the scripts in `tools/art/`. The repository records no artwork made by a human illustrator or taken from another game or a stock library. "Code" means the file is written or generated as SVG in this repository, with no image model involved.

The date range is the range of dates the repository's own records give (prompt headers, READMEs, design notes). The git history was squashed into one snapshot, so it carries no per-file dates.

| Asset class | Path | How made | Prompts stored? | Dates on record |
| --- | --- | --- | --- | --- |
| Title key art | `app/src/assets/art/title-keyart-c-road-at-dawn.*` | Generated; master resampled to 2560 × 1440 (master archived outside git, `tools/art/sources-manifest.json`), JPEG derived | Yes, beside the JPEG | 2026-11-01 to 2026-11-16 |
| People portraits (eleven couples) | `app/src/assets/art/people-*` | Generated; 512 px and 360 px derivatives (the full-size masters are archived outside git, `tools/art/sources-manifest.json`). Six were repainted by image edit on 2026-11-22 (faces); the Spartan man's upper lip shaved by a second masked edit the same day | Yes, beside each master | 2026-11-01 to 2026-11-22 |
| Unit miniatures | `app/src/assets/units/` | Generated on a flat magenta field, keyed to alpha, resized (`-source`, archived outside git, `-256`, `-64`, `-48`, card and panel crops). A reference miniature from the same set was attached to each request for style. Roundels (`*-roundel.svg`) are code | Yes for most: beside the file or in `units/prompts/`. None for the top-down aircraft plan sprites in `units/air/` | 2026-11-01 to 2026-11-22 |
| Siege props | `app/src/assets/siege-props/` | Generated, keyed, resized | Yes, beside each source | Not recorded |
| City miniatures | `app/src/assets/cities/` | Generated as sets, cut out and resized | Set prompts: some beside the files, the rest in `docs/design/city-art/prompts/` | 2026-10-10 to 2026-11-16 |
| Construction stages (wonders and landmarks) | `app/src/assets/construction/` | Generated master on magenta, then each stage an image edit of that master; keyed and aligned by `tools/art/wonder-art.py` and `tools/art/landmark-atlas-edit.py` | Yes for most, beside the full-size images | 2026-10-24 to 2026-11-16 |
| Era illustrations | `app/src/assets/eras/` | Generated 1920 × 1080 masters; 1280 × 720 and 720 × 1280 crops | Yes, beside each master | 2026-11-16 |
| Unlock art (buildings, improvements, effects on tech cards) | `app/src/assets/unlocks/` | Generated, keyed, resized | Partly: a few beside the files, the rest in `docs/design/unlock-art/` (`buildings.md`, `buildings-2.md`, `tiles-effects.md`, `masters/`) | 2026-10-11 to 2026-11-22 |
| Landmark map copies | `app/src/assets/landmarks/` | The world wonders' and Great Works' unlock paintings as graded for the map (`tools/art/landmark-atlas-edit.py`, `wonder-art.py`); the card copies in `unlocks/` have that grade undone by `tools/art/card-grade.py` | As for unlock art | 2026-11-01; split 2026-11-22 |
| Resource medallions | `app/src/assets/resources/` | Generated; Niter, Oil and Silicon were generated with the existing medallions as visual references | Only for Niter, Oil and Silicon (beside the PNGs). The six originals have none; their review sheets are in `docs/design/resource-art/` | 2026-10-10 (six originals chosen), 2026-10-24 |
| Map improvements | `app/src/assets/improvements/` | Generated, cut out, resized to 128 and 64 px | No. Review notes in `docs/design/improvements-painted/` | Not recorded |
| Camp and ruin sprites | `app/src/assets/map-features/` | Generated (direction B of three), with remembered variants at reduced opacity | Yes, in `docs/design/camp-ruin-art/README.md` | 2026-10-10 |
| Terrain sprites | `app/public/terrain/` | Generated as sprite atlases, then sliced | One shared prompt: `docs/design/terrain-art/stylized-sprites.prompt.txt` | 2026-10-11 |
| Natural wonders | `app/public/wonders/` | Generated; the black ground keyed to alpha by `tools/art/wonder-vignette-key.py` (vignettes) and an earlier cut-out (sprites) | Yes, in `docs/design/natural-wonders/README.md` | Not recorded; vignettes keyed 2026-11-21 |
| Heraldry: seals and glyphs | `app/src/assets/heraldry/seal-*.svg`, `glyph-*.svg` | Code: `heraldry/src/build-seals.mjs` with the engraving kit `kit.mjs` | Not applicable | Redrawn 2026-11-21 |
| Heraldry: crests | `app/src/assets/heraldry/raster/crest-*-512.png` | Code: rendered from the seals by `heraldry/src/render-crests.mjs` (SVG lighting in Chromium). The earlier generated crests were replaced on 2026-11-21 | Not applicable | 2026-11-21 |
| Icons | `app/src/assets/icons/*.svg` | Code: hand-written SVG on a 24 px grid | Not applicable | 2026-10 to 2026-11-21 |
| Reward emblems | `app/src/assets/rewards/*.svg` | Code: `rewards/src/build-rewards.mjs` | Not applicable | Not recorded |
| App icon and favicons | `app/public/favicon*`, `app/public/apple-touch-icon.png` | Composed by `tools/design/app-icon/build.mjs` from a code-drawn seal (`seal.svg`) over a parchment texture (`parchment.jpg`), which that script records as generated with Codex image generation | No prompt for the parchment texture | Not recorded |
| Music and sound effects | `app/public/audio/` | Third-party recordings released under CC0 1.0 (Kenney, Breviceps, vitalezzz), trimmed and encoded by `tools/audio/build_audio.py` | Not applicable. Sources, authors and licences: `app/public/audio/CREDITS.md`, `manifest.json`, `licenses/` | See `CREDITS.md` |

## Gaps

- No prompt is stored for the map improvements, the six original resource medallions, the aircraft plan sprites or the parchment texture.
- Where a set was generated from one shared prompt (terrain, city sets, camps and ruins), there is one prompt for the set, not one per file.

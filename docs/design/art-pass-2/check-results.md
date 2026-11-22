# Check results

- `cd app && npx vitest run src/unlockArt.test.ts src/portraits.test.ts src/factionSheets.test.ts src/ui/factionSheetCards.test.tsx src/icons.test.ts src/factions.test.ts`: **6 files passed, 29 tests passed**, exit 0, 2.14 seconds on 2026-11-16. Includes the four requested files plus icons/factions for the new icons and Aztec asset route. Cloud upload job `20261116T124411Z-9811ec` timed out at 300 seconds before starting; the configured wrapper fell back to local low-priority execution. Subsequent city/construction work received separate file integrity checks; a targeted city/construction test run follows below.
- Derivative integrity: **196 images decoded and dimension-checked**, PASS. See `derivative-check.txt`.
- `git diff --check`: PASS.
- `tools/art/distinctness.py --render --check`: **FAIL, exit 1** on a fresh cloud render (`20261116T130501Z-8dce58`, commit `b2f540a62bad`). 604 images, 19 sets, 9,750 comparisons, 103 flagged, 34 accepted, **15 failing**; 54 other flags are advisory. Command took 71 seconds. Initial upload `20261116T124409Z-af717c` failed before execution; rerun from committed art succeeded. Later construction frames are not members of any distinctness set, so scored asset bytes are unchanged. Full report and 15 proposed entries are delivered.
- `tools/art/distinctness.json` and `docs/learnings.md`: unchanged.
- No Rust, gameplay, AI or UI changes. Only new-asset wiring is the Aztec raster crest route in `app/src/data.ts`.

- `cd app && npx vitest run src/map/cityArt.test.ts src/map/constructionFx.test.ts src/unlockArt.alpha.test.ts`: **3 files passed, 192 tests passed**, exit 0, 2.64 seconds on cloud job `20261116T131650Z-907ee7`, source commit `91e1086bbf63`. City lookup: 7; construction drawing/lookup: 5; existing construction dimensions/alpha: 180. New Machine/Contemporary frames also passed the separate 196-file derivative check.
- Combined test runs: **9 distinct files, 221 tests passed**. No Rust or unrelated full-app checks were run.
- Delivery integrity: all 66 figure inventory paths, all 52 changed targets/before/pair paths, and every report image link exist.
- Master remained `fb473936846e8`; no master merge was needed. The task base remains the calendar playtest commit `4246569c5894`, even though `claude/calendar` advanced concurrently.

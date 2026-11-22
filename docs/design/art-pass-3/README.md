# Third art pass — 2026-11-16

Art based on `claude/calendar` at `a6ae2f29b1ad`. No rules, runtime UI or acceptance thresholds changed. These are reviewed Codex paintings for the lead's final art approval, not a claim that every fault in the second pass is cleared.

## Changed

- All **54 unit miniatures** now use a plain warm cream stone disc, one low step. Rebuilt source/256/64/48/panel/card files, retaining filenames. Feet, hooves, tracks, gun trails and aircraft display-stand contacts were inspected in crops. Galloping and raised-foot action poses remain. The Scout's first base edit changed his leg and was rejected; a tight base crop preserved his original sandal positions. Howitzer's disc reaches its trail; Keshig's disc reaches the lowest rear hoof.
- **Pikeman and Barbarian Pikeman:** larger bodies with two-handed sloping pikes. Three figure attempts each: the first read as short spears, the second made the near shaft too thick, the third retained slender long shafts. They read better at 48px but remain smaller than short-weapon neighbours; projected pike length is still compressed. No historical full-length certification is claimed.
- **Phalangite:** one adult replaces three tiny figures. Both hands visibly grip one shaft; a small shield is strapped to the left forearm rather than centred on the chest. Sandals, greaves and the shaft were inspected separately. Sarissa perspective/length remains compressed.
- **Panzer:** added overlapping hull side skirts. Eight small road wheels remain visible below them; existing turret skirts, long barrel and two-chamber muzzle brake retained and cropped for inspection. This improves the Panzer IV detail set; it does not claim to eliminate Tank similarity.
- **Machine, Modern, Contemporary and Future walls:** four independent paintings and twelve structurally unfinished construction edits, each with 512/128/64 assets. Every stage uses its finished master's scale/translation. The Future gate was corrected from an inappropriate stone arch to a rectangular sliding gate and access ramp. These are in `assets/cities` and `assets/construction`, the actual wall overlay loader paths; no runtime wiring was needed. The existing loader test now expects the four supplied eras instead of Industrial fallback.
- Thin-arrow/pike alpha processing preserves the painted shafts without morphological widening; the standard minimum filter erased the Barbarian Archer's arrow. Its **bowstring is still wrong**.

## Still wrong or uncertain

1. **Barbarian Archer string:** tight crop request rejected by image service (`violence`). No image returned. Full repaint was not used to work around the filter. Original wrong string retained.
2. **Legionary pilum:** upper-weapon crop request rejected (`violence`). Its iron shank is still too short; accepted figure retained.
3. **Infantry legibility:** advancing WWII repaint rejected (`violence`). Existing figure retained with the new disc.
4. **Medjay:** the calendar source already has no hanging fist beside the shield. Close inspection shows the left forearm concealed by the shield, right-hand spear and bare feet. Retained; no claim that a hidden grip was made visible.
5. Remaining poor detail work not completed: Cataphract lance compression; Arquebusier's simplified matchlock; Reisläufer's short pike; Composite Archer proportions/costume; Cuirassier horse proportions; Eagle Warrior feathers; Gaesatae body paint; Aztec portrait hand/costume; Mughal market/diwan ground integration; Swiss castle cliff; Machinery/Mathematics ground patches; Modern era scene dating. Pose-only findings from the old review are superseded by the current skill and are not repaint jobs.
6. Wall sheets validate registration and city compositing with the renderer's shared canvas and city-then-wall layer order, at 128px and enlarged. They are **static composites, not live-game screenshots**. The existing renderer draws the entire wall overlay after the city, so back walls can occlude buildings. No layering code changed here.

## Review artifacts

- `all-units-48.png`: all 54 at actual map raster size, without enlargement.
- `before-after-01.png` through `before-after-09.png`: portrait phone sheets, six units each, with enlarged and actual-size comparisons.
- `crops/final-contacts-01.png` through `-09.png`: feet, hooves, trails, tracks and stand contacts after keying.
- `{machine,modern,contemporary,future}-wall-stages.png`: stage sequence and city composites.
- `crops/`: inspected base, weapon, hand and wall crops. Generation paths/prompts and rejected attempts are recorded in the JSON files; raw generation paths are local provenance. Shipped PNG masters are portable. `review-sheets.py` reconstructs before images from the calendar commit.
- `distinctness-lead.md`: proposed acceptance changes for the lead only. `tools/art/distinctness.json` and `docs/learnings.md` were not edited.

Check results are recorded in `check-results.md` and the accompanying logs. No automated score constitutes historical or aesthetic approval.

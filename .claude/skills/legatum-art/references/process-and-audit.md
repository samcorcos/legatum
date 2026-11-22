# Art: process, consistency audit and finished-art rule

The mocks-first process, the consistency checklist and its review additions, and the finished-art (no abstractions) rule with the first audit's status.

Moved verbatim from SKILL.md on 2026-11-22 (original order kept). The current rules are in SKILL.md; where this file disagrees with SKILL.md, SKILL.md wins.

## Process

**Natural wonders (lead choice, 2026-10-10; real-landmark update, 2026-10-10):** Use recognizable real-world landmarks from `docs/design/natural-wonders/`, chosen to read clearly on a hex map. The strategic map shows style B, an ink-and-wash landmark integrated with the terrain at normal zoom; the far tier uses a compact engraved mark distinct from a ruin diamond and resource dot. The tile card and first-discovery notice use style A, the painted vignette. The art stays below roads, banners and units on the atlas. A target player liked Civilization's recognizable strategic-map landmarks with bonuses, and the user chose real natural wonders over the invented concepts.

1. **Mocks first:** 2–3 variants as screenshots in the real map or screen context, at 1000x600 and 1440x900, before implementing any new visual feature. The user picks.
2. **Build to the house style,** reusing tokens, pen weights and icon grid.
3. **Review against this skill** from screenshots (headless Playwright only; never the app's browser pane). Codex-built art is never final until a Claude agent has looked at it. Every branch that changes art, or the names, kinds or wiring art depends on, gets the `legatum-art-reviewer` agent before it merges (user, 2026-11-22); the gate requires its marker. It judges changed art beside its set (`tools/art/art-review.py sheet`) and looks for art the branch should have changed but didn't (`art-review.py impact`: renamed or new kinds, wiring with no file, deleted art still listed). Every branch that changes art, or the names, kinds or wiring art depends on, gets the `legatum-art-reviewer` agent before it merges (user, 2026-11-22); the gate requires its marker. It judges changed art beside its set (`tools/art/art-review.py sheet`) and looks for art the branch should have changed but didn't (`art-review.py impact`: renamed or new kinds, wiring with no file, deleted art still listed).
4. **Record** the pick and any new rule here in the same change.

## Consistency audit (checklist)

Run before a playtest that adds art, and as a whole-game pass when asked. Inventory, then flag each item:
- the same concept drawn two ways (icon vs map sprite vs tooltip);
- icons off the 24 grid, filled where others are monoline, or with hard-coded colours;
- map sprites with their own line weights, saturated fills, gradients or shadows;
- overlays too opaque to read terrain through;
- elements that are noise or vanish at the zoomed-out tier;
- faction art that doesn't read at a glance or doesn't follow from the story;
- raster art with mismatched technique, palette or framing;
- placeholder or emoji art still shipping.
Report each finding with a screenshot path, the rule it breaks and a proposed fix; group fixes by effort.
- **Owned-but-unworked territory is player-tinted hatch** inside the border line (never neutral ink, which reads as fog). Below a zoom threshold it becomes a flat player wash, because constant-pixel hatching turns to noise.
- **Improvement art (review, 2026-10-06):** hair and thin pen weights, no solid black fills, and it must never read as a landmark (a quarry isn't an amphitheatre). Each tech upgrade gets a visible step. Zoomed out, an improvement "badge" is a different shape from the city seal and the resource dot.
- **Every lens is translucent,** including Borders, Threat, Discovery and Movement; audit them all, not just Settle and Resources. Improvement art may skip its fine strokes under a unit counter so the unit and the land don't merge (8.1 review).
- **Polish pass (8.1 review, `claude/p82-polish`):** the Borders lens is an owner wash at 0.3 (`lens.ownerWash`) with a hair-weight owner line; owner numerals sit only on border tiles and only at normal zoom (`nearOnly`), and the far tier is tint alone. Improvement art is clipped out of the unit counters' footprint (`counterFootprint` in `improvementArt.ts`) so a unit never merges with furrows or benches. Zoomed out, a dormant resource marker fades to 45% with a longer dash (3.5 on, 3.5 off) so it never reads as the solid ring of an improved one.
- **Dormant markers at the far tier keep at least about 55% opacity,** so they read on the palest terrain (hills, desert) (8.2 review).
- **Goals 3 art (2026-10-07):** `unit-heir` (crowned head over a cloak with the relay star cut in) and `building-engine` (engine block with three cylinders and a flywheel; not the Foundry's flame). On the map the Heir is a civilian chit with a gilt inner rim; held by a rival it carries Byzantium's glyph on a disc in Byzantium's seat colour at its lower-right corner, and his zoomed-out pip wears a gilt ring (`sprites.ts` `HeirMark`).
- **Decisions (user, 2026-10-10).** The Japan and elf items below are SUPERSEDED the same day by the historical re-theme. Factions are real peoples, portraits pair a man and a woman in each people's most recognisable era, and there's no fantasy or alien art.
  - **Resources:** painted medallions.
  - **Natural wonders:** ink-and-wash landmarks on the map, so they read as geography and stay distinct from the medallions, and painted vignettes on cards and notices (lead's call; the user liked both).
  - **Mongols elves:** being explored. A target player wants a "cute girl" faction (adult, warm and endearing, never sexualised) now that the Japan are insects.

- **New unlock paintings match their set (2026-11-07 art review):** fit the same footprint, earth base and frame coverage as the others in the set (within about 10%), and check at 64 px in greyscale beside the predecessor (Oil Well, Quartz Mine and Saltpetre Works landed paler and smaller than Deep Mine; follow-up).
- **Construction stages match the finished piece's colour, not only its outline** (the Swiss Redoubt's stage 3 is warm tan, the finished fort cool grey; follow-up).

## Finished art, not abstractions (user, 2026-10-10)
"A target player loves the art and beauty of those things, so we need to fix that... it's ok to use placeholders when we're iterating, but once we've nailed it down, we should make sure the art gets updated properly."
- **Placeholders are fine while a feature is iterating.** Once its design is settled (the user has approved it, or it has shipped in a playtest build with no open redesign), its art must be finished, painterly and covetable in the house style. A settled feature that still shows abstract shapes (cubes, plain glyphs) is a defect.
- **Cities:** art per era and per size tier, so cities visibly grow with population and change architecture each era (Civ style). **Done (2026-10-10, `claude/era-takeover`):** the 28 painted miniatures (`app/src/assets/cities/`) are wired into the map by `map/cityArt.ts` and `terrain.ts`; the abstract cubes are gone.
- **The art audit** is recurring: once per release (not weekly, since 2026-11-22 every art PR gets the art reviewer), the ops supervisor tours the screens and a resolved turn and lists every remaining placeholder, abstraction or set misfit, ranked by how visible it is in play, and queues the art.
- **Era advancement** gets celebration art: a full-screen takeover with music. The board changes too, as cities and units take on the new era's look.
- **Art audit status (first audit 2026-10-10):** it ranked 25 placeholders and abstractions (the report and its crops were working files and are not in the repository). Queued from it: `title-keyart-historical`, `terrain-feature-art`, `moment-vignettes`, `camp-ruin-map-art` (each starts with a comparison sheet, no wiring). Deferred until the era overhaul and historical rename land: units, techs, buildings, cities, Great Works, era and victory art, faction ornaments and crests. Re-run the audit after the era overhaul.

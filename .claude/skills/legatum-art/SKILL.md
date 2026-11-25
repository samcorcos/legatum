---
name: legatum-art
description: This skill should be used when creating, changing, generating or reviewing any Legatum art (icons, map sprites and terrain art, improvements, faction ornaments, heraldry, leader portraits, key art, celebration art), when writing prompts for image generation, or when auditing visual consistency across the game. It holds the art direction, the consistency rules and the user's art decisions.
---

# Legatum art

North star (user): refined and grand, "a real 4X that could succeed Civilization": sophisticated and serious, premium, ceremonial, historically weighty. Restrained materials, confident negative space. The map is a place, not a diagram. If a piece of art would look at home in a mobile casual game, a children's book or a clip-art set, it is wrong.

## Where the details already live (read before changing art)

| What | Where |
| --- | --- |
| Directions, wordmark, house star, crest frames | `tools/design/directions.md` |
| The two looks (Atlas, Court), type, celebration layer | `docs/design/presentation-variants.md` |
| Colour and type tokens (the only colour source) | `app/src/brand/tokens.css` |
| Icon grid and glyph rules | `app/src/assets/icons/README.md` (`preview.html` shows them all) |
| Seals, glyphs, raster crests | `app/src/assets/heraldry/README.md` |
| Portraits and key art, with prompts | `app/src/assets/art/README.md`, `*.prompt.txt` |
| UI placement and feedback rules | `legatum-ui` skill |

Don't copy those rules here; extend them there and keep this skill to direction and decisions.

## The house style

- **Map art is atlas ink on vellum:** engraved or pen-and-wash line, hatching for tone, muted earth pigments, one accent red (`--accent`). Think a surveyor's atlas or an engraved estate map, not a painted game board. No glossy gradients, bevels, drop-shadow stickers, cartoon outlines or saturated flat fills on the map.
- **One line language:** the same pen weights everywhere on the map at a given zoom (set in `style.ts`). New sprites reuse those weights, never their own.
- **Player colour is a tint, not a fill:** ornaments, borders and seals take the player colour through `currentColor` or a translucent wash; the terrain still reads underneath (overlays about 0.5–0.6 opacity). Territory is the exception to "only a thin accent line stronger" (revised 2026-10-09, after a target player could not tell whose land was whose): a territory border is a bold seat-colour line with an ink edge over an 0.18 tint, drawn like an atlas's coloured outline and wash. The terrain still reads under the tint; the line is the strong part.
- **Icons:** single-colour `currentColor` glyphs on the 24-unit grid, drawn as solid silhouettes with knocked-out detail (not monoline: that is what the set always was), readable at 16 px, one consistent metaphor per concept everywhere it appears (the same Food glyph in the yield bar, tooltips, city banner and tech card). Every unit, building tier and faction replacement piece has its own icon, and text glyphs (warning, check, cross, arrows, letters) are never used as icons: add a `ui-*` or `mark-*` icon.
- **Goal marks (art review of the Colonia Victrix and Royal Tomb marks, 2026-11-22; reviewer's rules, accepted by the lead):** a glyph inside a wreath or ring gives the inner subject at least 40% of the grid width, or only the ring survives at 16 px; a mark with a fine and a heavy variant uses the heavy one wherever it is drawn under 20 px, on the map or in the UI; a goal mark is checked for distinctness against its own faction's glyph, because they sit side by side on every banner. Owed once a goal's design is settled: its two-tone reward emblem for the celebration card, and a landmark with construction stages for a goal that "stands" in a city (the Royal Tomb's landmark and stages are done, 2026-11-25; its reward emblem is still owed). A faction-goal landmark drawn on the map takes the map grade like any wonder: a graded `landmarks/` copy and graded stages, with the ungraded copy for cards only. A painting that starts warmer than the wonders needs more than the shared `atlas_edit` grade: check its mean saturation against the wonders' map copies (about 0.31 to 0.40 at 256 px) and mute it with `tools/art/goal-landmark-grade.py`.
- **Paintings** (portraits, key art): the oil technique and parchment-compatible pigments of the current set; no text in the image; prompts saved next to the file.
- **Historical people portraits (user, 2026-10-10):** depict exactly two unnamed adults, one man and one woman, as representative people rather than named leaders. Both are half-length with faces large, distinct and readable at 128 px; keep both hands natural when visible. Use the established refined oil technique and parchment-compatible pigments, but let period clothing, materials and a recognizable historical setting carry the identity. No fantasy, sci-fi, caricature, stereotype exaggeration, text, watermark or additional people. Each portrait's prompt records its people and era; provide a 1024 px original, 512 px PNG and 360 px JPEG.
  - **Every faction ships a `people-<faction>` pair portrait in the same change that adds the faction; there is no single-leader fallback (user, 2026-11-01, after India showed Akbar alone).** India's pair is a Mughal court official (diwan) and a noblewoman of his household, c. 1600. The `leader-*` art and the loader fallback are deleted, and `src/portraits.test.ts` fails if a playable faction lacks its `people-<faction>` files.
- **Celebrations** are the one place for gold glow and particles (see presentation-variants).
- **Zoom tiers:** every map element defines what it shows at each zoom. Detail (hatching, small figures) at normal zoom; a simple, high-contrast mark when zoomed out. Nothing turns into noise or vanishes without a replacement.


## Who decides

- Art is the lead's call, end to end: audit, repaint, approve and ship without asking him; show the result, not options, unless a call changes the game's overall look (user, 2026-11-15; earlier 2026-10-05 "use your judgment on the art").
- Accepted look-alike pairs in `tools/art/distinctness.json` are not the lead's to extend alone; prefer repainting (user, 2026-11-14). Remove accepted pairs once repaints make them unnecessary (user, 2026-11-15).

## Current rules

- **Painted versus engraved:** things you act on (cities, units, resources, camps, ruins, improvements) are painted and pop; geography (terrain features, natural-wonder landmarks) is engraved or ink-and-wash and recedes (designer, 2026-10-10; improvements painted since user 2026-10-25).
- **Finished art, not abstractions:** placeholders are fine while a feature iterates; once its design is settled its art must be finished and painterly (user, 2026-10-10). A recolour of another piece is a placeholder, not art (user, 2026-11-14).
- **Distinctness:** every icon, medallion and miniature is distinct from others in its set at display size, in greyscale and for colour-blind players; the distinctness check must pass, and an in-play crop is judged as well as the score (user, 2026-10-11). Keep thresholds; fix by redrawing.
- **Units:** painted miniatures as a zoom hybrid (full figure on a base up close, class roundel with owner rim far out). Same-class units differ across eras by pose or weapon angle; an era-numeral plaque sits on the base (user, 2026-10-11). Roundels are class marks, shared within a class (user, 2026-11-15).
- **Figures:** one plain round cream stone base for every figure; realistic heroic adult proportions, never cartoonish (user, 2026-11-07); action poses welcome where they read better at 48 px (lead, delegated 2026-11-16). Review each held or worn item against its real form for that people and period in a crop; never repeat the painter's description (user, 2026-11-14/15).
- **A renamed unit gets new art** matching its new identity (user, 2026-11-07). Siege gear matches the unit's own era and stays inside its hex (user, 2026-10-28, 2026-10-19).
- **Portraits:** every faction ships a `people-<faction>` pair portrait, one man and one woman, unnamed, with no single-leader fallback (user, 2026-10-10, 2026-11-01).
- **Heraldry:** eleven peoples, one historically grounded charge each, three tiers from one drawing; no modern national flag or state emblem, no loaded sign, no copy of another game's device; no per-faction special cases in code; judge a crest at the size the app draws it (2026-11-21).
- **Zoom tiers:** up close painted detail; far out a simplified, heavier, high-contrast mark that keeps type and owner legible (user, 2026-10-10; far marks heavier, 2026-10-11).
- **Terrain:** organic S2 is the default look; forests and ranges read as regions; the grid stays visible inside them; lenses win over terrain art; units must pop on any terrain (user, 2026-10-11).
- **Map sprites and rasters need real alpha**, with a test for any raster shown as a plain `<img>` (2026-10-11, 2026-11-21).
- **Animation:** combat animation is engraved, not particles, and plays in rules order (approved 2026-10-24). Construction shows the actual piece in partial stages advancing with production; payoffs are restrained, never cartoon pops (user, 2026-10-24). Animations meet the painted bar too (user, 2026-11-02).
- **No real-world insignia or makers:** no national insignia on shared unit art (2026-11-02); never name or model a figure on a real maker's product (2026-11-21). Image generators add a people's modern state device unasked (a Swiss cross on a "Swiss" treasury, 2026-11-22): every faction-scene prompt says "no national cross, coat of arms or flag", and review crops shields over doors, banners and clothing for it.
- **Deleting art** removes every reference in the same change (distinctness entry, icon preview), after a search that includes computed names like `building-${kind}` (2026-11-06, 2026-11-21). A retired people leaves no files behind.
- **Provenance** has one home: `app/src/assets/PROVENANCE.md` (2026-11-21).
- **Art masters live outside git** (2026-11-22): unit `-source.png` rasters, full-size people paintings and key art are listed in `tools/art/sources-manifest.json` and fetched with `tools/art/fetch-sources.py`; add a new master with `--add`, upload it, and commit the manifest, not the file. The hosted build serves bundled PNG art as WebP.
- **Map grade is not card art:** a landmark graded for the map is not reused on research cards; every unlock piece stands on a ground patch; thin parts survive every size (key after enlarging, check islands with `art-review.py impact`); each people's portrait faces are distinct people (art audit, 2026-11-22). A tall subject is prompted by its real proportions and the set's camera, never "tall and slender", and an open lattice is keyed with `wonder-art.py --lattice`, then its outline checked at 256 px on a contrasting ground for notches, holes and cut-loose tips (Eiffel Tower repaint and art review, 2026-11-22).
- **No accidental symbols; art ahead of its kind (project art review, 2026-11-22):** a balance scale, or any upright with a crossbar, is prompted with a wide beam and large pans so it can't read as a cross at card size. Art painted before its game kind exists says so in its README row, and the branch that adds the kind owns the test and the in-panel check.
- **Marketing:** trailers make you want to play: real gameplay from real games, combat and empire at scale, and the whole game, not only war; smooth 60 fps capture; epic orchestral music, no harpsichord (user, 2026-10-24, 2026-11-02). The tagline is "Empires rise and fall. Some are remembered." Launch media is recorded from the running game (`docs/media/src/refresh.sh`).

- **Front-door media looks like the game, never like generic AI output (user, 2026-11-22: the README's "What the AI can see and do" diagram was "claude slop" and "our reviewer should have caught that").** The README, `docs/media/`, the link-preview card (`app/public/og.jpg`), the launch kit and trailer title cards get the same premium bar as the game. Reject on sight: eyebrow or kicker labels (small letter-spaced capitals over a headline: "THE AI DOES NOT CHEAT", "FOG OF WAR", "A HISTORICAL 4X ON A HEX MAP"); feature strips of filler ("OPEN SOURCE · RUST ENGINE · PLAY IN YOUR BROWSER"); boxes-and-arrows diagrams, flowcharts of rounded cards, funnels, pill chips and numbered circles; gradient or flat pastel cards; cards holding one line and empty space, or of uneven heights; a slogan headline that restates the paragraph beside it; decorative badges. Use, in this order: a real screenshot or clip from a real game; the painted key art with the wordmark and the tagline only; plain prose or a numbered list. A diagram ships only when neither a screenshot nor prose can show the thing, and then in the house style (engraved ink on parchment, the game's own typefaces), judged by the art reviewer. Clips that share a README table row have the same aspect ratio, and when text leaves an image its alt text changes in the same commit.

## Decided and rejected

- Rejected: "Real peoples. Simultaneous turns. An AI that never cheats." as a tagline (user, 2026-10-24, again 2026-11-21).
- Rejected: the three README diagrams (architecture, simultaneous turns, "AI does not cheat"), and the kicker line and feature strip on the banner, link card and trailer end card (user, 2026-11-22). The README says those things in prose and lists; the banner and link card carry the wordmark and tagline only (an unfurl already shows the domain); video end cards add PLAYLEGATUM.COM.
- Rejected: an era-metal ring on unit bases, and a raw strength number on every base (user, 2026-10-11).
- Rejected: construction and payoff animation rounds 1 and 2 (generic scaffold, image reveals, sunbursts) (user, 2026-10-24).
- Rejected: Landship as a name on screen; it shows as "Early Tank", painted as a WWI rhomboid tank (user, 2026-11-14).
- Superseded: the fantasy and sci-fi directions (Japan insects, Mongol elves, relay-star iconography) by the historical re-theme (user, 2026-10-10).
- Superseded: engraved-ink-only improvement drawings by painted improvements (user, 2026-10-25).

## Process

1. Mocks first for any new visual feature: 2-3 variants as screenshots in context at 1000x600 and 1440x900.
2. Build to the house style, reusing tokens, pen weights and the icon grid.
3. Review from headless Playwright screenshots against this skill, plus `tools/gates/art.py`; Codex-built art is never final until a Claude agent has looked at it. Every branch that changes art, or the names, kinds or wiring art depends on, gets the `legatum-art-reviewer` agent before it merges (user, 2026-11-22): it judges changed art beside its set (`tools/art/art-review.py sheet`) and lists art the branch should have changed (`art-review.py impact`).
4. When a rule change shows an unpainted item to more players (e.g. the Gold and Science projects, Germany-only until 2026-11-22; painted the same day as `project-gold`, `project-science` and `project-grow` in `app/src/assets/unlocks/`), the art review flags painting it as a follow-up.
5. Record any new rule here (one line) and its detail in the matching reference file in the same change.

Run the consistency audit (`references/process-and-audit.md`) once per release (since 2026-11-22 every art PR gets the art reviewer).

## References (open only when needed)

| File | Open it when |
| --- | --- |
| `references/process-and-audit.md` | Running the consistency audit or checking the finished-art rule and audit history |
| `references/decisions-2026-10.md` | Working on resource medallions, landmarks, improvements, icons, reward emblems, lenses, the far tier, portraits' use, the Courier Hall rose or roads |
| `references/direction-terrain-units.md` | Working on city or unit miniatures, terrain features and organic terrain, wonder map art, era distinctness, siege props |
| `references/marketing-and-animation.md` | Making trailers or README media, combat or construction animation, wonder or future-unit paintings, figure review, portrait fixes |
| `references/rulings-2026-11.md` | Heraldry, launch audit fixes, figure bases, the unit sweep rulings, keying thin weapons |

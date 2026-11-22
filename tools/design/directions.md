# Legatum art directions

North star (from the user): refined and grand, a real 4X that could succeed Civilization. That means premium, ceremonial, and historically weighty, with rich but restrained materials, beautiful type, and confident negative space. The map should feel like a place, not a diagram.

Mockups (static HTML at 1440x900, built from `tools/design/mockups/build/`):

| Direction | Mockup | Render |
| --- | --- | --- |
| A. Imperial Instrument | `mockups/variant-a.html` | `.progress/art/direction-a.png` |
| B. Atlas of Heirs | `mockups/variant-b.html` | `.progress/art/direction-b.png` |
| C. Heir's Court (recommended) | `mockups/variant-c.html`, `title-c.html`, `lobby-c.html` | `direction-c.png`, `title-c.png`, `lobby-c.png` |

Rebuild with `node tools/design/mockups/build/build.mjs a b c title-c lobby-c`. The map, scene, and content live in `build/shared.mjs`. Each `variant-*.mjs` holds one direction's map palette, tokens, and CSS.

---

## Shared rules (all directions)

### Brand system

- **The wordmark is the only way the name appears.** Use `app/src/assets/icons/logo-legatum.svg` (spaced geometric caps, an A holding a four-point star, a ruled underline with a star). Never set "Legatum" in a text font.
  - Title screen hero: the wordmark at 480–560 px wide, centered, over the world.
  - In game: the wordmark at the top left of the top bar, 24 px tall, in the direction's gold or ink.
  - Setup, lobby, and loading screens: the wordmark top left at 24 px. On loading screens it is centered at 320 px with a progress rule under it.
  - Victory and defeat screens: the wordmark small, above the result title.
  - Dashboard header (`tools/dashboard`): the wordmark at 20 px, in the same color tokens.
  - App icon: the chosen **Atlas Seal**, built from `tools/design/app-icon/seal.svg` and its parchment texture by `tools/design/app-icon/build.mjs`. Use this same seal for desktop icons and favicons in both looks; do not substitute a lone four-point star or a look-specific icon.
- **House ornament: the four-point star** from the A (path `M0 -5L.95 -.95L5 0L.95 .95L0 5L-.95 .95L-5 0L-.95 -.95Z`). It marks capitals, panel headers, the selected-row state in lists, and dividers. Use at most one star per panel header. Never use it as a bullet in running text.
- **Harmony with the wordmark.** The wordmark has wide Roman proportions and monoline geometric strokes. Each direction pairs it with a display face of the same lineage (A: Roman inscriptional capitals; B: Garamond; C: a Didone of the deco era, with a 1920s geometric sans). Tracked uppercase labels (0.14–0.32em) echo the wordmark's spacing.
- **One token source.** Each direction ships as one `tokens.css` (the `TOKENS_*` blocks in the variant files are the seed), consumed by the game UI, the dashboard, and the icon export script. No component defines its own colors.
- **Fonts are bundled**, not loaded from Google at runtime (the game makes no requests to font services). All faces used are OFL-licensed.
- **Neutral wording.** Name seats by player ("Ada", "Bo"), group them by team ("Team 1", "Your team"), and label the panel "Council" or "This Turn". Never write copy that assumes two particular people.

### Crests and small glyphs

A separate heraldry pass is replacing the faction emblems. Until then the current emblems sit inside the same frames, so the new crests drop in.

| Size | Where | A (Imperial Instrument) | B (Atlas of Heirs) | C (Heir's Court) |
| --- | --- | --- | --- | --- |
| 88–120 px | Setup, lobby, victory screen | Octagonal steel plate, 1 px gold rule, crest in the player color | Heater shield on vellum, ink outline, crest in the player color | Round medallion: gold rim, lapis field, player-color inner ring (see `lobby-c.png`) |
| 40 px | Seats panel, city header | Octagon, 1 px frame at 55% of the player color | Shield, ink outline | Medallion, 1 px gold with a player-color ring |
| 24–26 px | Victory race rows | Octagon | Shield | Medallion; team members overlap by 6 px |
| 12–16 px | Inline glyphs (yields, buttons, list rows) | Flat single-color icon, no frame | Same | Same |

At 24 px and below the crest must read as a silhouette. The heraldry pass should deliver a simplified small-size variant of each crest.

### Craft rules (all directions)

- Spacing is a strict 4/8 grid: 4, 8, 12, 16, 24, 32, 48. Panel padding is 16 (A) or 16–20 (B, C). Row heights are 24, 30/32, 56–64, or 76.
- Two type families per direction. All data numerals are lining and tabular. Set this with `font-feature-settings: "lnum" 1, "tnum" 1` on `body`, not with `font-variant-numeric` (the `font:` shorthand resets `font-variant-numeric` on every element that uses it).
- The transient [celebration layer and editorial film variant](../../docs/design/presentation-variants.md) are intentional, documented exceptions.
- No drop shadows on chrome. The only shadows allowed are the 1.5–2.5 px offset shadows under map tokens, which sit on terrain. No glows on chrome. No gradients on chrome, except a 2-color hard split for team bars.
- One primary action per region: the seat CTA (bottom right) and the contextual primary in the unit bar (for example Found City).
- Layout at 1440x900: 48–64 px top bar; left column 352 px (city panel, top 72–76); right column 320 px (victory race top, seats and CTA bottom); unit command bar centered on the map stage (384 → 1096 px, 104 px tall). The map stage between the columns stays clear.

---

## Map rendering (shared pipeline, per-direction palette)

The map makes a 4X feel grand, so all three directions use the same painterly pipeline (`renderMap` in `build/shared.mjs`) and differ only by palette and glyph ink. Terrain is static per game, so render layers 1–7 once per zoom level to an offscreen canvas and cache them. Only borders, fog, and overlays redraw each turn.

Layer order, bottom to top:

1. **Deep water.** A flat `deep` color, plus a low-frequency noise mottle (turbulence 0.011, grayscale, 16–22% opacity, overlay or soft-light).
2. **Shallows.** Coast-hex polygons in `shallow`, blurred 12 px, so depth fades smoothly away from land.
3. **Coastline and wave lines**, all derived from the land mask:
   - Land mask: land-hex polygons, blur 6.5 px, alpha threshold (`a' = 16a − 7.6`), then displacement by fractal noise (freq 0.045, scale 11 px). The result is an organic coast that still follows the hexes. On canvas: blur with `ctx.filter`, threshold alpha in one pass over the image data, and displace with a precomputed noise field.
   - Coast line: mask dilated by 1–1.1 px, minus the mask.
   - Two offshore wave lines: rings at 5–5.8 px and 10–10.7 px from the mask, at 40–70% and 20–35% opacity in `waveLine`.
4. **Land color field.** Hex polygons in the terrain color with ±4% per-hex jitter. Water hexes next to land are painted with a neighboring land color so the blur has no dark fringe. Blur 4–5 px so biomes blend like paint, then clip to the land mask.
5. **Relief.** A per-hex hillshade from the elevation gradient, lit from the north-west (+0.12 on hills, +0.25 on mountains). Draw it as `light` or `shadow` polygons, blur 9 px, at 32–36% opacity, clipped to land.
6. **Texture.** The same macro mottle at 28–35% opacity, plus fine grain (turbulence 0.9, black alpha) at 10–22%.
7. **Grid.** Draw each shared edge once, as 1 px lines. Land edges in `grid` (ink at 14–28% opacity); water edges fainter (5–10%).
8. **Terrain glyphs**, deterministic per hex:
   - Forest: 6–8 round canopies (r 3.6–5.8) sorted by y. Each canopy has a shadow ellipse offset (+1.6, +2.2), a dark crown, and a light highlight offset up and left.
   - Mountains: one or two peaks with a light west face, a dark east face, and a snow cap, a 0.9 px ridge line, and four hachures on the shadow face.
   - Hills: two or three mounds with a 1 px crest line and three hachures.
   - Desert: three dune ripples. Tundra: snow streaks. Grassland and plains: 3–4 small tufts. Ocean: sparse wave glyphs, about 22% of hexes.
   - In explored fog, glyphs drop to 75–85% opacity.
9. **Cities.** A paved hex (82% radius), a cluster of 5 + population roof blocks with hard shadows, and a tower on capitals. Ruins use `ruin.svg` in the direction's accent.
10. **Fog.** Hex-aligned, because it is game information:
    - Explored but not visible: a gray fill with `mix-blend-mode: saturation`, a `fog.tint` wash, and a 45° hatch (6 px pitch). Canvas has the same composite operation (`saturation`). The mockup values (55–80% desaturation) turned real maps, which are mostly remembered terrain, into gray murk; the app uses 20% desaturation, a 34% paper wash and a lighter hatch in Atlas (faded ink), and 35% with a 30% lapis wash in Court.
    - Unexplored: fill with the direction's `unexplored` treatment, blurred 1.5–2.5 px (polygons enlarged by 1 px so no terrain leaks). Draw a crisp 1 px edge where known meets unknown.
11. **Territory**, drawn above the fog tint so last-known borders stay in color:
    - A tint of 6–8% of the player color, clipped to the owned hexes.
    - An inner band 4–8 px wide, clipped inside the border: a flat band at 30–32% in A and C; a blurred watercolor wash at 42% in B.
    - A 1.3–1.6 px line on the inside of the boundary, so two neighbors' lines sit side by side instead of overlapping.
12. **Vignette** at 28–50% (sepia in B, lapis in C).
13. **Overlays.** Movement reach (a wash plus a dashed or dotted boundary), the planned path, the turn marker at the destination, the proposed city ring, the selection mark, then unit tokens and city labels (HTML or canvas text).

The world in the mockups is designed, not noise: a western ocean with a bay that reaches the capital, a mountain spine between rivals, an inland sea, and tundra in the north. The rank-based thresholds are 7% mountains and 13% hills of the land. Name the seas and regions on the map in the display face (italic for water, tracked capitals for land, at 50–75% opacity). Labels make it a place.

---

## A. Imperial Instrument

Machined brass on blued steel: an instrument of the fallen star empire. Chamfered plates, engraved gold hairlines, Roman capitals.

**Type.** Cinzel for titles, city and unit names, section heads, and the turn numeral. IBM Plex Sans for everything else.

| Token | Spec |
| --- | --- |
| `display-xl` | Cinzel 600 30/1, tracking 0.06em (city name, turn numeral) |
| `display-l` | Cinzel 600 20/1, tracking 0.06em (unit name, item in production) |
| `display-s` | Cinzel 600 13/1 or 11/1, tracking 0.2–0.24em, uppercase (panel titles, section heads) |
| `body` | Plex Sans 400 13/1.35; list items 14 |
| `data` | Plex Sans 600 14–18, tabular |
| `label` | Plex Sans 600 10/1, tracking 0.16em, uppercase, in `--gold` |

**Color.**

```
--steel-0:#0a0d11  --steel-1:#10151b (panel)  --steel-2:#151b22 (raised)  --steel-3:#1c242d (active)
--rule:#27303a  --rule-2:#364252  --gold-rule:rgba(201,166,92,.42)
--gold:#c9a65c  --gold-hi:#e6c883  --text:#ece6d6  --text-2:#aeb2b6  --muted:#78808a
--good:#86c46b  --bad:#df6a55
```

Gold is a signal color. Use it only for the primary action, the selected item, labels, and the frame hairline.

**Corners and borders.** Panels ("plates") have an 8 px 45° chamfer on all four corners (clip-path), a 1 px `--gold-rule` frame, and an opaque `--steel-1` fill. Buttons are chamfered on two opposite corners (6 px). Tooltips use a 6 px chamfer with a solid gold 1 px frame. Internal dividers are 1 px `--rule`.

**Components.**
- Panel: 40 px header with the star, a Cinzel 13 tracked title, and an optional right-hand link; then 1 px `--rule` sections.
- Button: 32 px tall (40 in the command bar), Plex 600 11 uppercase, tracking 0.12em, `--steel-2` fill with a `--rule-2` border. Primary: `--gold` fill with dark text. Danger: text only.
- List row: 32 px, 16 px side padding, columns icon / name / cost / turns. Selected row: `--steel-3` fill, a 2 px gold left bar, and the name in `--gold-hi`.
- Tooltip: Cinzel header with an icon and a hairline under it; 24 px rows with dotted leaders; a 32 px total row with the value in `--gold-hi` at 16; a muted note.
- Map tokens: chamfered steel plates, a 1.6 px rim in the player color, icon in the player color. Selection: gold corner brackets. City labels: a chamfered plate with the population cell in the player color and the name in Cinzel 12, tracked.

**Map palette.** Natural and slightly cool (ocean `#0f2335`, grass `#5f7c44`, forest `#435f38`). Cream coast line, dark unexplored with a faint gold engraving pattern, crisp flat border bands.

**Trade-offs.** The most legible and efficient of the three, and the easiest to build. It reads as precise and imperial, but the least ceremonial: close to a very good sci-fi strategy UI rather than a successor to Civ.

---

## B. Atlas of Heirs

A grand printed atlas: paper panels with ink double rules, a painted map, cartouche city labels, a compass rose, and italic sea names.

**Type.** Cormorant for display words and all italics. Libre Franklin for labels and every data numeral (Cormorant's figures are old-style and uneven in tables). The turn numeral stays in Cormorant as a ceremonial exception.

| Token | Spec |
| --- | --- |
| `display-xl` | Cormorant 600 44/1 (city name) |
| `display-l` | Cormorant 600 20–26/1.1 (panel titles, unit name, CTA) |
| `display-italic` | Cormorant italic 400–500 14–20 (subtitles, notes, sea names) |
| `body` | Franklin 400 13–14 |
| `data` | Franklin 600 15–17, tabular |
| `label` | Franklin 600 9–10, tracking 0.12–0.2em, uppercase |

**Color.**

```
--paper:#f2ead8  --paper-2:#e9dfc8 (active row)  --paper-3:#dfd2b5 (tracks)  --vellum:#f7f1e3 (tooltip, label)
--ink:#2b2419  --ink-2:#5b503f  --muted:#8a7c65  --rule:#bcac8e  --rule-2:#9d8c6c
--red:#9a3322 (the one accent: kicker, selection, route)  --good:#4f7f33  --bad:#a3392a
```

Yield colors are deepened for paper: food `#5e8c3b`, production `#b8662a`, gold `#a8841c`, science `#2c7fab`, influence `#8358bd`.

**Corners and borders.** Square corners everywhere. Panels have a 1 px ink border, a 3 px paper gap, and a 1 px `--rule-2` inner rule (the double rule, done with `box-shadow: inset`). The top bar has a double ink rule under it. The tooltip is vellum with one ink rule and a notched pointer.

**Components.**
- Panel: centered title in Cormorant 20 with an italic subtitle in `--muted`.
- Button: 32 px, 1 px ink border on vellum, Franklin 600 11 uppercase. Primary: ink fill with paper text.
- List (production): a table with 30 px rows, italic red group heads ("Units", "Buildings"), dotted leaders, and cost and turns right-aligned. Selected row: `--paper-2` fill and a 3 px ink bar.
- Tooltip: Cormorant 18 head, rows with leaders, Cormorant total, italic note.
- Map tokens: square wargame counters in the player color with a 1.2 px ink border, a paper inner rule, and a paper icon. Selection: a red double ring. The route is a red dashed curve with a roman-numeral turn marker. City labels: vellum cartouches with an ink double rule, diamond end-caps, small-caps names, a population square in the player color, and turns in italic.

**Map palette.** Light atlas tints (sea `#8fb1ba`, shallow `#bcd6d6`, grass `#b3bf85`, plains `#d8cb93`). Ink coast line with strong wave lines. Watercolor border washes. Unexplored is blank vellum with a fine hatch ("terra incognita"). Sepia vignette.

**Trade-offs.** The most distinctive direction and the strongest "place". It is closest to Civ 6's strategy view and old atlases. The light paper UI is heavy on large screens over long sessions, it pushes readable text colors darker, and it leans cozy-tabletop more than imperial-grand. It is also the hardest to keep crisp at small sizes.

---

## C. Heir's Court (recommended)

A court ceremony for heirs to a star empire: art deco and neo-classical, with deep lapis panels, gold-leaf linework, symmetric headers, and sunburst fans.

**Type.** Bodoni Moda for display at 17 px and up (names, numerals in headings, CTA, turn, tooltip total, italic notes). Josefin Sans for UI and all tracked labels. Its 1920s geometric caps are near-siblings of the wordmark. Josefin sits high on its baseline, so add 2–3 px top padding to small caps in boxes (as the mockups do).

| Token | Spec |
| --- | --- |
| `display-xl` | Bodoni Moda 500 46/1 (city name); 28–32 (turn numeral, lobby title) |
| `display-l` | Bodoni Moda 500 20–26/1 (unit, seat names, CTA, item in production) |
| `display-num` | Bodoni Moda 500 17–22 (VP, yields, tooltip total) |
| `display-italic` | Bodoni Moda italic 400 17–18 (display notes and sea names); smaller notes and controls use Josefin Sans at 14 px or larger |
| `body` | Josefin Sans 400 14–15 |
| `label` | Josefin Sans 600 9–12, tracking 0.2–0.32em, uppercase, in `--gold` |

**Color.**

```
--lapis-0:#0a1128 (bars, fields)  --lapis-1:#0f1a3a (panel)  --lapis-2:#15234c (raised)  --lapis-3:#1c2d60 (active)
--gold:#d4b46a  --gold-hi:#f0d894  --gold-dim:rgba(212,180,106,.4) (frames)  --gold-faint:rgba(212,180,106,.16) (inner rules)
--ivory:#f3ecda  --ivory-2:#c9c6c4  --muted:#8e93ad  --good:#8fcf72  --bad:#ef7f6a
```

Yield colors are brightened slightly for lapis: food `#8fcf72`, production `#eb9a57`, gold `#f0cf62`, science `#6cc0ea`, influence `#c29cf0`.

**Corners and borders.** Panels have deco notched corners: 10 px square notches, cut by clip-path on a 1 px `--gold-dim` frame. A second `--gold-faint` hairline sits 4 px inside; it stops at the notches, which gives the deco corner. Primary buttons use 8–10 px notches and a double inner rule. Selects and secondary buttons are square with a 1 px `--gold-faint` or `--gold-dim` border. Round forms are reserved for medallions (tokens and crests).

**Components.**
- Panel header: symmetric. An optional sunburst fan (the star over nine rays), then a rule with a diamond tip, a tracked Josefin 12 title, and a mirrored rule. Titles are always centered.
- Button: 32 px, Josefin 600 11 uppercase, tracking 0.16em, lapis-2 fill with a `--gold-dim` border. Primary: gold fill, lapis text, inset double rule, notched corners. Quiet: text only.
- List row: 30 px inside a 12 px panel inset; icon / name / cost / turns. Selected row: `--lapis-3` fill with top and bottom `--gold-dim` hairlines and the name in `--gold-hi`.
- Yields strip: five cells separated by short gold-faint dividers, Bodoni numerals, and the active cell marked with a gold diamond under it.
- Tooltip: 1 px gold frame on lapis-0; a centered tracked header; rows with dotted gold leaders; a "Per turn" total in Bodoni 20.
- Map tokens: round medallions (gold hairline rim, lapis field, a 2.4 px player-color ring, ivory icon); damage shows as a dark arc on the ring. Selection: a gold ring with four diamond studs. The path is a gold line ending in a diamond turn marker. City labels: a plaque with gold top and bottom rules, detached side ticks, a population diamond in the player color, a tracked Josefin name, and turns in Bodoni.
- Title screen (`title-c.png`): the world behind a lapis veil, a deco double frame with corner studs, the fan and the wordmark as hero, and a standard menu: Continue, Single Player, Multiplayer (Host · Join · Hot Seat), Load Game, Settings, Quit.
- Lobby (`lobby-c.png`): a seats table (number, crest medallion, player, seat type Human/AI/Closed, faction, team, difficulty, status), plus a world panel with a map preview, settings, Ready and Invite, and a notched primary "Begin the Game".

**Map palette.** Jewel-toned twilight (ocean `#0b2837`, grass `#577f47`, forest `#31593a`). A gold coast line and gold wave lines. Unexplored is lapis with a deco scale pattern. Flat border bands; lapis vignette.

**Trade-offs.** The most ceremonial and "grand". The dark jewel field makes the map colors sing and suits long sessions. The risks are ornament creep (limit it to one fan per screen region and one star per header), Bodoni's hairlines below 17 px (never use it there), and Josefin's small x-height at 12–13 px (keep body at 14 or larger and data in tabular numerals).

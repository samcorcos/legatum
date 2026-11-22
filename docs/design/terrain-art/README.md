# Terrain feature art directions

## Round 4: organic S2 is the default (2026-10-11)

The user saw the round 3 sheet and decided: "yes adopt the look" (organic S2 becomes the default for everyone); "yes you should still be able to see [the grid]" (it stays visible inside big woods and ranges, so tiles can be counted at a glance, not the near-invisible Civ style); and "the miniatures for units are now pretty hard to see, which is not ideal."

**What changed** (branch `claude/terrain-default`):
- **Default.** `app/src/map/terrainVariant.ts` now defaults to S2 with the organic regions on. `?terrain=current` brings back the old drawn glyphs and `?organic=0` gives S2 as one sprite per hex, for comparisons. Organic is also off if the sprites fail to load, so the map never goes blank.
- **Retired** (the README records them; their atlases stay in `source-atlases/`, round 1 sprites in `sprites/`, round 2 shots in `round2/`): round 1 A, B and A-light, and round 2 S1 and S3. Their code (`lightMask`, ridges, tinting) and about 4.7 MB of dev sprites are deleted from the app. The shipped S2 sprites moved from `app/public/dev-terrain/s2/` to `app/public/terrain/`. No settings or faction screen offers a terrain choice, so there was nothing to update there.
- **Grid.** Where a hex edge touches a wood, hills or a range, the grid is a cased line: a pale casing (`gridArt.light`, 2.6 CSS px) under an ink core (`gridArt.dark`, 1 px), numbers in `style.ts`. Open ground keeps the faint 0.14 line. It works at normal and far zoom.
- **Units pop.** Measured on five terrains at 1000x600 and 1440x900 and at far zoom (shots `round3/units-before-*` and `round3/units-after-*`). Four looks were compared: A (figure 15% larger, vellum pool), B (pool plus a firm owner-colour ring), C (a vellum coin instead of the stone base) and the old one. B won: the owner colour stays the dominant read of the base (C's pale coin let it wash out), and the vellum pool lifts the painted figure off dark woods, brown peaks and olive hills alike without a dark outline or sticker edge. The shipped look: the figure is 20% larger, a soft vellum pool (0.78 opacity at the centre, radius 0.72 of the figure's side) sits behind it, and its stone base wears the owner's colour as a 2.2 px ring between two ink rules. Health bars and status tags sit lower to clear the bigger base, two-unit stacks fan wider (1.16 token widths), and improvement art keeps a bigger clear footprint under counters. Far zoom keeps the roundel (it already read on every terrain); owner colours stay the colour-blind-safe seat palette. Cities, resources and wonders are drawn as before and still read.
- **Lenses.** Under Settle, Resources and Yields the feature art (sprites and region wash) fades to 38% (`ART_FADE` in `terrain.ts`, `TerrainCache.setArtFade`), so the lens tint and icons win; the biome colour stays and chunks repaint in place.
- **Test.** `e2e/terrain-units.spec.ts` shoots the units and asserts that a Soldier alone in a deep wood differs from the art around it: the unit's base against the bare terrain outside its halo (at least 1.3:1) and the vellum pool beside the figure against the same terrain (at least 1.8:1). Calibration: the old look measured about 1.0 for the pool, the new look is above the threshold (see the numbers in the learnings entry).

**Next art, queued for Luna** (`.progress/astra-queue/pending/`, S2 style, real alpha, 256 and 128 px): `terrain-single-trees` (16 single trees, `app/public/terrain/trees/`), `terrain-wood-edge` (4 edge strips, `edge/`) and `terrain-massif-peaks` (4 multi-hex massifs, `massif/`). Wiring hook when they land: `organicTerrain.ts` `planOrganic` picks the sprite for each plan element (`Forest`/`Mountains` kinds today), and `terrainVariant.ts` `loadTerrainSprites` is where the new folders are loaded. Single trees replace some cluster sprites at a wood's edge, strips follow the region outline, and a massif sprite replaces the spine peaks of a range of 8 or more tiles. Not wired yet: the tracker has the item.

## Round 3: organic regions (2026-10-11)

The user asked for terrain that reads as landscape: "a forest that is multiple tiles should feel like it is a forest covering multiple tiles, not a bunch of forest tiles pushed together", and a 12-tile mountain range should feel different from two or three mountains. Round 3 is a modifier on any sprite direction: `?terrain=s2&organic=1` (also worked on `s1`, `s3`, `a`, `b`; ignored on `current` and `a-light`). Superseded the same day by round 4 below: organic S2 is the shipped default.

The phone sheet is [`round3-organic.png`](round3-organic.png): S2 as is on the left, S2 organic on the right, at normal and far zoom, a large forest and a small grove close up, a 19-tile range and a 2-tile range close up, the Settle lens, and the 1000x600 window. Individual shots are in [`round3/`](round3/) as `<s2|s2o>-<normal|far|forest|grove|range|smallrange|settle|resources|small>.jpg`, plus `s1o-normal.jpg` and `s3o-normal.jpg` (the modifier is style-agnostic). The map is generated seed 2914823 (the `mapArtFixture` with a seed): a 128-tile forest, a 19-tile range, a 6-tile range, a 2-tile range and a 2-tile grove. It was found by scanning 400 seeds for the largest forest and range with small ones on the same map. Shots come from [`round3/terrain-round3-shots.spec.ts.txt`](round3/terrain-round3-shots.spec.ts.txt), run on the cloud box (copy it to `app/e2e/`, build WASM, `TERRAIN_SHOTS=<dir> npx playwright test e2e/terrain-round3-shots.spec.ts --project=game`).

**How it works** (`app/src/map/organicTerrain.ts`, wired in `terrain.ts`):
- **Regions.** Connected components per feature type (forest, hills, mountains), computed from explored tiles only. Each tile records its distance in hex steps to its region's edge. A range of hills or mountains larger than 3 tiles gets a spine: its diameter path, plus a branch to the farthest flank in ranges of 8 or more.
- **Forests and hills of 4+ tiles** scatter overlapping sprite clusters on a world-space lattice that ignores hex edges (at normal zoom; zoomed out, every tile keeps one per-hex far mark over the region's wash, because a scatter of small far marks faded into dashes). Points near the region's edge drop out by a noise threshold, so the outline wanders inside the hex. Density and size grow with distance from the edge, and species follow low-frequency patches (one part of a wood leans conifer, another broadleaf), never a per-hex shuffle. A wood gets a soft canopy-shade underlay, deeper toward its interior and clipped to the region's own (feathered, wobbly) outline, so it never darkens the open tile next to it.
- **Groves and ranges of 1 to 3 tiles** keep the classic one-cluster-per-hex look, so a small grove still reads as a grove and two peaks as two peaks.
- **Ranges of 4+ tiles:** peak size follows depth and the spine, and a saddle peak sits on every spine link, so the range runs on without gaps. Ranges of 8+ are massifs: bigger spine peaks, low outer peaks (foothills) on margin tiles, and a stone-grey wash under the range.
- **Hills** are a firmer, warmer value (the sprite multiplied by `#c4ab80`) on a light earth wash, so they separate from desert and plains (the round 2 follow-up).
- **Biome boundaries** in the blended colour field follow noise-nudged hex outlines (corners and edge points move up to about a sixth of a hex) (shared corners and edge points get the same nudge from their world position, so there are no gaps) instead of hard hexes.
- **The grid.** The classic veil left a thin sliver between explored hexes that, once blurred, drew a pale line along every hex edge over the art: that line is what cut a wood into tiles. Organic cuts explored hexes out as one union, and draws the faint `grid` line (alpha 0.14) over the art instead, as the only grid line.

**What worked**
- A big wood now reads as one forest: continuous canopy across hex edges, a darker interior and a ragged edge. Beside it, a 2-tile grove and the classic clusters elsewhere read as small woods. This is the clearest win and visible at every zoom.
- The firmer hills fix the round 2 weakness: rolling hills are now distinguishable from desert at a glance.
- Saddles and the stone wash join a range's peaks into a ridge. The 19-tile range reads as one continuous range where S2 shows separate triangles.
- Every surface stays on top: cities, borders, units, resources and lenses are drawn after the terrain cache. Settle keeps its Great/Good/Poor read; under Resources the green rings stay clear.
- It is style-agnostic: S1 and S3 organic look right with no per-style code.

**What didn't**
- **Massifs are only half there.** Generated ranges are mostly one hex wide, so few tiles have interior depth, and peaks can only grow along the spine (capped at 1.24x on edge tiles, so a peak never covers a lowland hex). A 19-tile range reads as one connected range, but not yet as a single grand massif. That needs multi-hex peak sprites (prompts below), or a generator that makes wider ranges.
- **The canopy is a little uniform.** Clusters at 0.8 scale on a 0.86-hex lattice give 3 to 4 clusters per interior hex; from a distance the interior is texture more than trees. The underlay helps it read as a mass, and it is quieter than S1. Single-tree sprites would let the edge thin into individual trees.
- **Far zoom gains little.** A first version scattered small far marks across each region. The UI review found they faded into dashes, and hills read as hatching. Far zoom now draws S2's per-hex far marks over a half-strength wash, so it looks like S2 far with darker masses.
- **The hex grid is fainter.** Without the veil slivers the grid is only the 0.14 ink line. Tiles stay countable at normal zoom (the line runs over the wood) but are less prominent than in S2. Hover and every lens still outline tiles.
- **Lens alpha.** Under Settle a wood is darker (the canopy wash multiplies under the tint). Lowering terrain art alpha under lenses (the round 2 follow-up) is not done; it needs the lens state in the terrain cache key.

**Legibility checks** (`organicTerrain.test.ts`, 19 tests):
- every forest, hill and mountain tile keeps at least one sprite centred inside its own hex, and no sprite centre lies on a tile of another type;
- edge peaks stay classic size (at most 1.06x), apart from a massif's mid-spine peaks (at most 1.3x);
- a wood thins from its interior to its edge, and a grove keeps one cluster per hex;
- a massif's largest peak is more than 1.15x the largest of a pair of peaks;
- city tiles get no art, and forests next to tundra take the conifer sprite.

**Fog of war.** Regions are computed only from explored tiles, so a known forest tile next to the unknown is an edge, and the wood thins there. Unit tests show that different terrain in unexplored tiles changes nothing: the same regions, edge depths, spines and sprite plan. No sprite is anchored or centred on an unexplored tile, and a tile's organic signature changes when exploration reveals more of its region, so its chunks repaint. The underlay is built from explored tiles, and the unexplored veil is still drawn on top.

**Determinism.** Every random choice is hashed from tile or lattice coordinates, and the plan is a pure function of the explored map, so there is no shimmer and every chunk agrees. Tests check that the plan is identical across rebuilds.

**Performance** (Large map, 60x40; headless Chromium on the cloud box, software rendering, so absolute times are high and only the comparison matters; three 183-frame pan sweeps from a cold cache; `perf-*.json` in `round3/`):

| | current | S2 | S2 organic |
|---|---|---|---|
| Cache build (ms) | 158 | 156 | 200 |
| Chunk render work, normal sweep (ms) | 1,997 | 1,972 | 2,047 (+4%) |
| Chunk render work, far sweep (ms) | 2,657 | 2,598 | 2,606 |
| Frame draw, mean / max, normal (ms) | 12.1 / 94 | 12.0 / 93 | 12.3 / 104 |
| Frame interval p50 / p95, normal (ms) | 27 / 68 | 26 / 63 | 26 / 69 |

The regions and both sprite plans are computed once per map update. That adds about 45 ms (up to 65 ms in other runs) when a turn changes the explored map. Chunk rendering costs 4–12% more at normal zoom across three runs; run-to-run noise was 7–10%. Pan and zoom still only blit cached chunks, so frame time is unchanged within noise.

**UI review** (`legatum-ui-reviewer`, 2026-10-11): no blockers; it recommends merging as a dev toggle, not as the default. The forest request is met. The range request is mostly met: the arc reads as one ridge, but the one-hex-wide part under Eleusis is still separate triangles. Its three majors were fixed in this branch and reshot: weak far marks (now per-hex), the wash staining neighbouring open tiles (now clipped), and open ground still reading as hex cells (stronger boundary wobble, plus roomier hill clusters). Still open:
- the grid nearly vanishes inside big woods (ask the user: countable, or Civ-style near-invisible?);
- under the Settle lens a wood is darker;
- the hill field can read as a texture close to the claimed-land hatch.

**Recommendation: adopt organic on top of S2** (`?terrain=s2&organic=1` as the candidate look). It does what the user asked for forests, firms up hills, and costs nothing visible in legibility or speed. Before shipping it as the default:
1. the designer and a playtester look at the sheet;
2. paint the next art increment, below;
3. check it on a few more generated maps.
Keep the toggle until then. If the user finds the wood too uniform, the next step is art, not code.

**Next art increment (prompts; not queued, pending the user's pick).** All three in the S2 flat-illustrated style (one shade plane, crisp silhouettes, the S2 palette), transparent PNG with real alpha, 256 px masters plus 128 px derivatives:
- *Single trees:* "Transparent PNG sprite atlas, 4x4 grid, sixteen single trees for a historical strategy map in the S2 flat illustrated style: eight conifers and eight broadleaf trees of varied height and crown shape, each alone and centred in its cell with transparent gutters, one flat shade plane and a dark outline, base at the bottom of the cell. No ground, shadow plate, text or fantasy."
- *Wood-edge sprites:* "Transparent PNG, 2x2 grid, four forest-edge strips in the S2 style: a dense band of overlapping crowns along the top that thins downward into two or three scattered trees, so it can sit at the edge of a large wood. No ground or background."
- *Massif peaks:* "Transparent PNG, 2x2 grid, four broad multi-peak massifs in the S2 style, each about two and a half hexes wide and one and a half tall: a high central summit with snow, overlapping lower shoulders and foothills to both sides, flat grey-and-snow shading, dark outline, base along the bottom of the cell. No background, text or fantasy."

## Round 2: stylized directions

The user asked for three middle-ground options after round 1: more dimensional and beautiful than the
current glyphs, while keeping the feature silhouettes and terrain colors easy to tell apart. The
static, mobile-portrait sprite comparison is [`stylized-sprites.png`](stylized-sprites.png). It shows
all four woodland, hill and mountain variants for each direction on parchment. The game can render
each direction with `?terrain=s1`, `?terrain=s2` or `?terrain=s3`; the default remains `current`.

- **S1 · Painted toy:** softly modeled painted trees, broad rounded hills and bold snow-capped peaks.
- **S2 · Flat illustrated:** crisp board-game shapes with one shade plane and strong silhouettes.
- **S3 · Watercolour sprite:** sparse organic pigment, light ink edges and controlled wash texture.

Each direction has four transparent 256 px masters per feature and 128 px map derivatives. The
fourth woodland sprite is sparse tundra conifers; a forest touching tundra selects it. Each also has
four 64 px far marks, derived from its own sprites so fine detail drops away at distance. Feature
clusters are centered and leave substantial open ground. These are comparison directions, not a
settled art selection; the lead will judge them in the live map.

Generated transparent 1024 × 1536 source atlases are preserved in `source-atlases/`. The generation
briefs are in [`stylized-sprites.prompt.txt`](stylized-sprites.prompt.txt).

Round 1 was an art exploration only; its PNG comparison sheet is [`terrain-art-directions.png`](terrain-art-directions.png), and its four expanded adjacency boards are [`terrain-adjacency-normal-A.png`](terrain-adjacency-normal-A.png), [`terrain-adjacency-far-A.png`](terrain-adjacency-far-A.png), [`terrain-adjacency-normal-B.png`](terrain-adjacency-normal-B.png), and [`terrain-adjacency-far-B.png`](terrain-adjacency-far-B.png). Round 2 adds three wired options for review in the live map.

## Round 2 in the real renderer (2026-10-11)

Real-renderer screenshots (headless Chromium, 1440 x 900) of one map view in `current`, `s1`, `s2` and `s3`. The comparison sheet for a phone is [`round2-in-game.png`](round2-in-game.png) (normal and far zoom per variant); [`round2-in-game-detail.png`](round2-in-game-detail.png) adds a hills-and-mountains view and the Settle and Resources lenses. Individual shots (JPEG, q90) are in [`round2/`](round2/) as `<variant>-<normal|far|east|close|settle|resources>.jpg`: `normal` is the scene on the sheet, `far` is 0.4 zoom, `east` is the hills and mountains of the east, `close` is 1.4 zoom on the Old Canopy and its range.

How they were shot: `scripts/build-wasm.sh`, then the spec in [`round2/terrain-round2-shots.spec.ts.txt`](round2/terrain-round2-shots.spec.ts.txt) (copy it to `app/e2e/terrain-round2-shots.spec.ts`; it starts Vite, loads the `mapArtFixture` map as a save and sets `?terrain=`). Run `TERRAIN_SHOTS=<dir> npx playwright test e2e/terrain-round2-shots.spec.ts --project=game` with Node 22. The scene is the fixture map at turn 12 (eight empires, borders, units, resources, ruins, natural wonders), centred on Lugano and the Old Canopy at 0.66 zoom (hex radius about 24 px, still the normal tier), with forest, tundra, desert, plains, grassland, coast, mountains, two cities and the Great Barrier Reef, Mount Fuji, Uluru and Victoria Falls in view. The game has no rivers (no river terrain or river art exists), so none could be shot. The scene has no hills in its middle, only along the right edge, so the `east` shot is where hills should be judged.

**Current (baseline).** Quiet and clear, but flat: circles for forest, triangles for mountains and arcs for hills are readable only because colour does most of the work. Hills are the weakest type (faint arcs on plains), and nothing here is more beautiful than the base map. It sits perfectly under painted cities and units, and lenses are untouched.

**S1, painted toy.** The most dimensional and most "finished" art, and the best mountains: grey-and-snow peaks that are unmistakable at normal zoom and read as a range at far zoom. The cost is weight. A dense wood is a dark, high-contrast mass with orange autumn trees mixed in, so the eye goes to it before it goes to the units or the city, which is the opposite of the zoom-tier rule (geography should recede). Hills are dense rows of olive ridges that become a busy wallpaper across a whole region. Terrain types are distinct, but through contrast, not restraint. Under Settle the woods turn muddy brown-orange and fight the Great/Good/Poor tint; under Resources the dark woods still compete with the medallions. This is the likeliest to repeat the round 1 complaint of being too busy.

**S2, flat illustrated.** The best balance of the three. Conifers and broadleaf are told apart by silhouette alone, woods are a mid green that stays quieter than the unit counters, mountains are crisp grey peaks, and tundra reads as pale conifers. Because the shading is one plane, a region of forest looks like a place, not texture, so adjacent types separate at a glance and the painted cities, borders, units and resource medallions stay the first read. The weakness is hills: pastel rolling bands with low contrast that are legible as hills next to plains but hard to tell from the edge of the desert wash in a mixed view, and a little flat next to the painted figures. Under the lenses it is acceptable: the Settle tint dominates and the forest silhouettes are still readable; Resources keeps its green rings clear, though woods stay a bit darker than the wash around them.

**S3, watercolour sprite.** The softest and prettiest at a glance, closest to Civ 6's painted-but-not-a-picture feel, and the quietest: sparse trees on a wash with a light ink contour, so units and borders pop. It sits best under painted cities and units and under both lenses: the tint dominates and the sprites fade to texture. The price is distinctness. Hills are sand-coloured ridges nearly the same value as desert and plains, mountains are pale and small, and at far zoom the marks are faint and fall back toward the base map's blur. A player has to look, which is the round 1 complaint ("hard to tell them apart") from the other side.

**Recommendation: S2.** The designer's round 1 verdict was that A, B and A-light were too busy and too hard to tell apart; S2 is the only direction that is clearly easier to read than the busy sets and still clearly more beautiful than `current`. Two follow-ups before wiring it: give hills a firmer ridge and shade value so they separate from desert and plains (borrow S1's hill contrast, not its density), and cut sprite alpha under Settle and Resources toward S3's level so the lens tint wins. S3 is the fallback if the designer and a playtester prefer quiet over distinct; S1's mountains are the best of the three and could be borrowed into S2. The dev toggle `?terrain=` stays until the user picks. These judgments come from one fixture map; check the pick on a generated map with a river-free coast, a Tundra border and a real city skyline before shipping.

## Round 1 recommendation (superseded by round 2: the user found A, B and A-light too busy)

Choose **A, Engraved atlas** for the settled map. It uses the house line language consistently, keeps snow as paper, and makes tree species and hill folds read at the usual 22–40 px hex radius. The adjacent range and wood hold their individual hex structure without looking like stickers. The 64 px marks are kept separate for the far tier.

B is a strong alternative if the lead prioritizes continuous watercolor landforms. Its softened edges help a range or wood merge across hexes, especially at normal zoom. The wash also raises the risk of muddy terrain colors under Settle and Resources and may soften the tile boundaries players use for planning. The sheet shows both; this recommendation is a design call, not a request to wire A now.

## Round 1 sprite inventory

Each direction has four 512 × 512 transparent master PNGs per feature type, with 128 × 128 and 64 × 64 derivatives beside them:

- `sprites/A|B/mountains/{512,128,64}/mountain-01..04.png`
- `sprites/A|B/hills/{512,128,64}/downs-01..04.png`
- `sprites/A|B/forests/{512,128,64}/forest-mixed-01..03.png` and `forest-tundra-conifers.png`
- `sprites/far/mountain-1..4.png`, `downs-1..4.png`, and `forest-1..4.png` are the simplified, high-contrast 64 px marks for the far tier.

The forest set has three mixed woodland silhouettes and one conifer-only tundra edge/scrub treatment. Desert dunes are left for a later exploration. The six 2 × 2 generated source sheets were split into their sprite variants, clipped to a pointy-top hex footprint, and palette-quantized. PNG alpha is retained; the pigment colors come from `ATLAS` in `app/src/map/style.ts` only. Each normal sprite stays inside one hex; B has a slight alpha feather at its edge.

## Prompts

The following prompts were used to create the 2 × 2 source sheets. The rendered master sprites are the resulting art, cropped to one cell each and retinted to the exact `style.ts` palette.

### A · mountains

> Transparent PNG sprite atlas for a serious historical strategy game's map, direction A: engraved atlas. Square 2 × 2 grid, four separate mountain range miniatures, one centered in each equal cell, transparent gutters, each fits one point-up hex. Real-world European ranges, asymmetrical overlapping ridgelines with receding peaks, natural erosion, snow left as paper, warm lit vellum ridge and fine sepia hachures on the shadow flank, contour lines at the base. Fine nineteenth-century survey atlas engraving, four varied silhouettes, not copies. Palette: `#2b2419`, `#5b503f`, `#f2ead8`, `#f7f1e3`, `#9d8c6c`, `#c2b194`, `#f1e8d4`, `#a8967a`, `#fcf9f1`. Transparent outside the landform; no ground plate, text, symbols, sticker shadow, gradient, cartoon or fantasy; no tent-like triangular peaks. Crisp enough to read at 64 px.

### A · forests

> Transparent square PNG, exact 2 × 2 atlas with four different small forest groves, one per cell and generous transparent margins. Engraved historical atlas trees, not clip art: natural irregular clusters of distinct individual tree forms, conifer and broadleaf mixed in the first three variations, a sparse conifer-only tundra grove with low frost scrub in the fourth. Varied silhouettes, spacing, fine trunks/crowns, short ground-shadow strokes and small hachures. No rounded green balls, bright detached dots or lollipop forms. Muted palette from `#2b2419`, `#5b503f`, `#f2ead8`, `#f7f1e3`, `#9d8c6c`, `#5b7649`, `#8aa56b`. No background, plate, label, outline sticker, effect or fantasy; transparent around each grove; readable at 64 px.

### A · hills

> Transparent square 2 × 2 PNG atlas, four isolated miniature European rolling downs or low ridges, one per cell. Each fits one point-up hex, with transparent gutters. Nineteenth-century survey atlas engraving: broad, gently overlapping asymmetrical ridges, stepped contour curves, fine short lee-slope hachures and lightly washed warm earth with vellum-lit slopes. Four varied silhouettes: paired rounded downs, long escarpment, folded low ridge, three rolling hills. No bare arcs or mountain triangles. Restrained palette from `#2b2419`, `#5b503f`, `#f2ead8`, `#f7f1e3`, `#9d8c6c`, `#cdb283`, `#7c6547`, `#4a3920`. No background, text, hard enclosing outline, sticker shadow or fantasy.

### B · mountains

> Transparent square 2 × 2 PNG atlas, four distinct small real-European mountain range miniatures, one per cell and contained within a point-up hex. Loose traditional watercolor wash with light fine pen ink, layered asymmetrical ridgelines and receding foothills, warm vellum lit slopes, hatched umber shadow flanks, snow left as paper. Four different natural silhouettes; no isolated triangular tent peaks. Soft watercolor edges fade into transparency so adjacent hex sprites can read as one range. Restrained palette from `#2b2419`, `#5b503f`, `#f2ead8`, `#f7f1e3`, `#9d8c6c`, `#c2b194`, `#a8967a`, `#fcf9f1`. No bright color, text, label, background, border, glow or fantasy.

### B · forests

> Transparent square 2 × 2 PNG atlas with four small irregular forest groves, each within one cell and with large transparent margins. Loose restrained watercolor wash and light fine ink linework, real European woods. Mixed conifer/broadleaf in the first three variations and conifer-only tundra wood with low frost scrub in the fourth. Natural patchy canopy, visible trunks and branches, sparse ground-shadow strokes. Feathered edges fade into alpha so adjacent groves visually join into a wood. No circles, balls, lollipops, buttons or detached dots. Muted vellum, umber and forest greens from `#2b2419`, `#5b503f`, `#f2ead8`, `#f7f1e3`, `#9d8c6c`, `#5b7649`, `#8aa56b`. No backdrop, label, sticker outline or fantasy.

### B · hills

> Transparent square 2 × 2 PNG atlas, four distinct rolling hill/downs miniatures, one per cell, contained within a point-up hex and separated by transparent gutters. Loose watercolor wash with fine light ink contours; historical European downs and low escarpments with gently rolling asymmetrical terrain, overlapping ridge folds and delicate lee-slope contour hatching. Four silhouettes: paired low mounds, long chalk escarpment, undulating ridges, broad folded downs. Soft feathered edges fade to alpha so adjacent hexes blend. Muted earth palette from `#2b2419`, `#5b503f`, `#f2ead8`, `#f7f1e3`, `#9d8c6c`, `#cdb283`, `#7c6547`, `#4a3920`. No background, text, hard outline, sticker shadow, mountain triangles or fantasy.

## Static composition notes

`terrain-art-directions.png` compares before/A/B at normal and far scales. The adjacency boards show a flat five-hex range, a 3 × 3 wood, and a coastal sequence of downs and woods, with map overlays above the features. The base images are the supplied real map crops in `docs/design/art-audit/`.

The requested tour screenshots were absent, so I re-ran `npx playwright test e2e/screen-tour.spec.ts`. All 118 cases stopped before app startup because this environment denied Chromium's macOS `MachPortRendezvousServer` bootstrap (`Permission denied (1100)`, process exit `SIGTRAP`). No tour screenshots were produced. The static composites therefore use the supplied real audit crops rather than claiming to be fresh 1440 × 900 tour captures. The boards are art direction mocks, not renderer screenshots; the unit counter, Eboracum banner and round resource medallion are composed for legibility review.

## Wiring estimate for the lead

The current `TerrainCache.renderChunk()` calls `drawGlyphs()` inside each 512 px chunk; towns, fog, borders and the unexplored veil follow it. A sprite direction can replace only the `Forest`, `Mountains` and `Hills` branches in `drawGlyphs()` while preserving that layer order. Load/decode the selected direction's images once (for example as `ImageBitmap`s), choose the sprite variant with the existing stable tile-coordinate hash, and draw it under the tile's hex clip. Use `isFar(hexRadiusPx)` from `grid.ts` to draw the `sprites/far/` mark below the far threshold; at normal zoom draw the 512 or 128 source chosen for the active cache level. The images are baked into the existing chunks, so pan/zoom still blits cached chunks and no per-frame sprite work is needed. If direction selection ever becomes runtime-configurable, include it in cache invalidation or the chunk key. Keep the UI tokens, units, banners, resources and lenses above terrain in their existing render passes.

Approximate decoded RGBA memory per sprite is 1 MiB at 512, 64 KiB at 128, and 16 KiB at 64. Twelve sprites × all three sizes is about 13 MiB for one direction; both directions resident would be about 26 MiB, plus about 0.2 MiB for the twelve far marks. Load only the selected direction and far marks at runtime. These decoded sources do not allocate extra chunk canvases; the existing cache can hold up to 110 512 × 512 RGBA chunks (about 110 MiB of pixel storage, before browser/GPU copies), and that remains the larger memory cost.

## Tradeoffs to review when wiring

| Concern | A · Engraved atlas | B · Wash and ink |
|---|---|---|
| Noise and repetition | Four silhouettes per type; stable coordinate hashing avoids flicker. More hatching can shimmer below the far threshold, so replace detail with a mark there. | Feathered patches reduce obvious hex repetition and connect ranges/woods. Soft overlapping pigment can blur tile boundaries if multiple sprites stack. |
| Memory | Same source footprint as B. Runtime target is about 13 MiB decoded for a selected direction. | Same footprint; alpha feathering does not materially change decoded size. |
| Unit/city/resource overlays | Fine lines keep the unit chip, city banner and medallion distinct. Preserve their current layers above `TerrainCache`. | Broad pigment washes can lower contrast behind these pieces; keep the current overlay ordering and test the palest terrain and darkest counters. |
| Settle / Resources lenses | Hatch and line detail should survive the translucent washes; test the 0.5 Settle wash and 0.3 Resources dim values in `style.ts`. | The washes may merge with lens tints or the parchment dim. Test both lenses and reduce sprite wash alpha if the map's existing lens tokens no longer read. |
| Historical fit | Reads like a surveyor's map of real landforms; safest match for the settled atlas. | Still historical, with a more painterly watercolor character and softer hex boundaries. |

Do not settle the final in-game scale or lens opacity from this sheet alone. The lead should wire the chosen direction in an isolated art branch, shoot normal/far and Settle/Resources screenshots, and review those before merge.

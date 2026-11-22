# City art directions

![Three Ancient city sprite directions and the atlas metropolis across six eras](city-art-directions.png)

Art exploration for the directions below; the chosen direction A (full set) is wired into the map, see "Full city set". The comparison uses the art-audit map screenshot at normal zoom; the far-zoom panels scale that same crop down to illustrate the proposed sprite footprint, rather than claiming to be a second live screen-tour capture. The existing screenshot preserves the city's banner and the blue seat border/wash around the tile.

## Current map drawing

`app/src/map/terrain.ts` draws each explored city as a parchment-colored paved hex with 5 to 17 terracotta roof rectangles (five plus up to twelve for visible population). Each block gets a small offset shadow and lighter roof strip. Capitals receive a tiny tower; walls are a separate rampart pass. The draw order puts the city banner above this terrain art. At far zoom, `app/src/map/render.ts` uses the city seal and places its name nearby instead of drawing the town. The visual is deterministic and legible, but its repeated rectangular roofs do not communicate period architecture.

## Directions

### A · Painted isometric miniature

A clustered, hand-painted model with warm roofs, limestone, trees and a rising building count. It reads as a place quickly and carries the richest surface detail. The larger tiers create a strong growth payoff.

**Pros:** warm, covetable miniature quality; distinctive skyline and building density; the population step is immediately visible.

**Cons:** it leans toward a toy-like 3D miniature and competes with the atlas map language; foliage and roof detail will collapse into texture at 64 px; deep ground shadows can muddy borders and nearby resource marks. Keep the future production art grounded, restrained and more planar than this study.

### B · Atlas ink and wash — strongest direction

A compact town in engraved sepia contours, hatching and translucent earth-color washes. This best extends the existing natural-wonder landmark language and the map's shared ink-and-vellum palette, while still letting roofs, gates and civic buildings carry era identity.

**Pros:** strongest continuity with the map; terrain and seat-color territory remain visible; architectural silhouettes can change without changing the map's visual grammar; the six-era family demonstrates that continuity.

**Cons:** fine hatching and individual roofs blur at far zoom; the generated examples are still more dimensional and detailed than the map's pen-weight rules allow. The shipped far tier should be deliberately simplified to a high-contrast town silhouette or seal in the same ink language, not a downsampled copy of the near sprite. The smallest hamlet also needs a recognizable shape after simplification.

### C · Illuminated-map vignette

A manuscript-like walled-town emblem, with small gold and blue accents. The tiers grow by adding a clear wall, gate towers and skyline landmarks.

**Pros:** the wall gives city and metropolis a strong far-zoom silhouette; population growth is obvious; the gold detail feels ceremonial and collectible.

**Cons:** gilding and blue heraldic details move away from the map's ink-and-wash style; the gates, flags and frames can read like a faction badge or landmark; the extra decoration competes with the banner and colored border. Reserve the richer illumination for cards or historical map plates if the map treatment stays Atlas.

## Full city set · direction A

The selected direction is A: a restrained painted isometric miniature, with a low shared footprint, warm roofs and stone, and a visible increase in building density. The set covers Ancient, Classical, Medieval, Renaissance, Industrial, Modern and near-future Future architecture. Every era has hamlet (population 1–3), town (4–7), city (8–12) and metropolis (13+) art. Future is a hopeful, human-scale 2040s city with gardens, clean transit and solar roofs.

The production files are under `app/src/assets/cities/`: `<era>-<tier>.png` is the transparent 512 px master, with `-128.png` and `-64.png` derivatives beside it. **Wired (era overhaul phase C, branch `claude/era-takeover`):** `app/src/map/cityArt.ts` loads them and `app/src/map/terrain.ts` paints each city from its owner's era (public `PlayerPublic.era`) and size tier, so a city changes look the turn its owner enters the next era and a rival's city shows the rival's era. A missing file falls back to the nearest finished era, then the nearest size (never a cube); `missingCityArt()` lists any gap (none today). The portrait, map-colour comparison sheet is [`city-art-full-set.png`](city-art-full-set.png). The portrait, map-colour comparison sheet is [`city-art-full-set.png`](city-art-full-set.png).

Direction A's architectural detail becomes texture at the smallest zoom. The separate `marks/<tier>-mark-128.png` and `marks/<tier>-mark-64.png` sprites are the high-contrast far-zoom city marks; they communicate population size through the building cluster and central landmark. They are shared across eras.

Optional defenses are separate transparent overlays: `ancient-walls`, `classical-walls`, `medieval-walls`, `renaissance-walls` (the angular star fort) and `industrial-walls`. Each has 512, 128 and 64 px files. Modern and Future have no wall overlay in this set.

The generated, uncropped tier sheets are kept in `sources/cities/`; the base and era-specific generation prompts are recorded in [`prompts/a-full-set.txt`](prompts/a-full-set.txt). Each sheet has a 2×2 layout in hamlet, town, city, metropolis order. The extracted sprites share a centered 512 px canvas and common city footprint.

## Banner, border and zoom fit

- Keep art centered in the paved city hex, with a low profile so capitals and later-era towers do not push into the banner.
- Preserve the current draw order: city art under banner, units, labels and map overlays. Borders stay readable around the art and keep their player color as a wash/edge rather than a sprite fill.
- At normal zoom, use enough architectural grouping to show population growth and era. At far zoom, use a purpose-drawn silhouette sized around the existing city seal; remove tiny roofs, windows and hatching.
- For a final art pass, use the line weights and earth pigments from `app/src/map/style.ts`; avoid heavy drop shadows, saturated fills and opaque tile stickers. Keep the city's terrain visible between buildings.
- The 64 px far marks are distinct from the detailed 64 px city derivatives; the renderer selects them below the same hex size where banners become seals (`isFar`, 22 px per hex radius). Do not use the detailed downsample at far zoom.
- Fortifications are optional overlays and should only be shown when the game state calls for them. Keep their transparent centers so city art remains visible below.
- Pure-red generation fringe and disconnected sheet fragments were removed during extraction. Do not treat red as a game accent.

## Sources

- Map context: [`art-audit/city-town-cubes.png`](../art-audit/city-town-cubes.png)
- Natural-wonder atlas references: [`natural-wonders/`](../natural-wonders/)
- Map renderer: [`app/src/map/terrain.ts`](../../../app/src/map/terrain.ts), [`app/src/map/render.ts`](../../../app/src/map/render.ts), [`app/src/map/style.ts`](../../../app/src/map/style.ts)

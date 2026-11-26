# A pale rectangle round the pointer after a wheel zoom (2026-11-26)

Seen on the live site: a rectangle of the map, round the selected unit and the pointer, lighter than the land
about it and not following hex edges. It stayed until something else redrew the map.

Cause. A frame where only the tile under the pointer changed repaints a few boxes (`hoverClipBoxes`), and it runs
the whole `draw` inside that clip, the terrain cache included. A wheel zoom holds its chunk level for 250 ms and
then settles on the level the zoom wants. When the frame that settled it was a clipped one, the boxes were drawn
at the new level and the rest of the canvas kept the old level, scaled. Nothing asked for another frame, because
every chunk of the new level was already cached.

Before terrain painted between frames (PR 309) a waiting terrain made every frame a full one, so the settling
frame was never clipped. Since then a waiting terrain only steps, and a hover frame goes ahead of the step.

Fix. `TerrainCache.relevelled` says the last draw drew another level than the one before; `terrainLanded()` in
`render.ts` now answers for that as well as for a finished chunk, and `MapCanvas` follows any clipped frame that
changed the terrain with a full one.

Reproduced by moving the pointer in a circle for 700 ms after each wheel notch across a level boundary and
comparing the canvas with a forced full redraw: 9 of 14 tries differed before, 0 of 14 after. The same steps are
now a test in `app/e2e/map-chunk-cache.spec.ts` (fails without the fix).

The rule: a frame that draws inside a clip must not change anything the whole canvas shows. Whatever a clipped
frame can change outside its boxes (a chunk landing, the level) has to ask for a full frame.

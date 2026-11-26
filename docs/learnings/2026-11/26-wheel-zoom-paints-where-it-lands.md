# A wheel zoom painted every zoom level it passed through (2026-11-26)

Measured in headless Chromium (no GPU, so software canvases) on a turn-280 save at 1366x768: eight wheel notches
60 ms apart, in and then out, then five notches 400 ms apart.

What it did before:

- Zooming in from level 2 to level 6 painted 58 to 69 chunks, at levels 3, 4, 5 and 6, to end on a view of 9.
  Each level the gesture went past got a view of chunks, and a ring round it.
- Zooming out painted 51 chunks to end on a view of 6: a two-chunk ring at each level on the way.

What changed (`app/src/map/terrain.ts`, `chunkCache.ts`):

- Past the range a held level scales to, the cache goes straight toward the level the zoom wants
  (`gestureLevel`), at most `FALLBACK_SPAN` levels at once zooming in (the farthest a coarser level may stand in
  from), with no limit zooming out.
- Zooming in paints no ring until the zoom rests. It reveals no new ground, and a ring is three to five times the
  chunks of the view.
- Where a chunk stage is slow (`SLOW_STAGE_MS`, a machine drawing canvases on the CPU), a zoom gesture paints no
  chunks at all. A chunk there takes longer than the gesture stays at its level, so the cached levels were already
  what the player saw. The level the zoom lands on is painted when it rests, 250 ms after the last notch.

Result, same save, before to after:

| Gesture | Chunks painted | Long-task time |
| --- | --- | --- |
| In, 8 notches | 69 to 33 | 117 to 57 ms |
| Out, 8 notches | 51 to 20 | 577 to 60 ms |
| In, 5 slow notches | 95 to 57 | 282 to 355 ms (noise: 5 and 6 tasks near 60 ms) |
| In, 4x CPU throttle | 32 to 27 | 2,465 to 1,732 ms |
| Out, 4x CPU throttle | 23 to 20 | 4,208 to 3,239 ms |

What is left at 4x throttle is not chunk painting: each wheel notch redraws the whole map, 150 to 230 ms a frame
there. A cheaper frame during a gesture (scaling the last frame, then one sharp redraw at rest) is the next step.

A machine with a graphics card keeps its sharper behaviour: it still paints the held level and the ring while
zooming out, since a chunk there is 5 to 10 ms.

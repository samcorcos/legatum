# A terrain chunk was one long task on a machine without a graphics card (2026-11-26)

Players' telemetry: terrain painting is 43 to 48% of long-task time at every stage of a game, and worst on small
canvases, which is where Chrome draws canvases on the CPU. A report of "very slow UI" on a laptop matched it.

Reproduced in headless Chromium (no GPU, so software canvases) with Chrome's CPU throttling, on a turn-121 and a
turn-280 save, jumping the camera and stepping the zoom (`tools/perf/terrain-fill.cjs`).

What was slow, in the order it mattered:

- **One chunk was one task.** A chunk is 5 to 10 ms with a graphics card and 90 to 200 ms without. The cache
  painted at least one whole chunk per frame, so every frame of a fill was a long task.
- **Chrome queues canvas calls.** Drawing onto a canvas costs almost nothing until the canvas is read. A chunk's
  glyphs, sprites, towns and territory were therefore paid in the frame that first showed the chunk, on top of the
  map's own redraw. Timers round the drawing code read near zero; a CPU profile put the time under `(program)`.
  Splitting work across frames does nothing until each part is made to run: using the canvas as the source of a
  1 px `drawImage` runs its queue without reading pixels back (a readback stalls a real GPU).
- **A redraw of the whole map per chunk.** On the same machine a full map frame is 60 to 90 ms, most of it after
  the script returns (the browser rastering the main canvas). One redraw per finished chunk cost as much as the
  chunks.
- **Stand-in layers drawn across the whole view** while any chunk was missing, though the finished chunks are
  opaque and cover them.
- **The unexplored veil** was filled, blurred and blitted for every chunk, including chunks with nothing
  unexplored in reach (a third of a chunk's cost there).

What changed (`app/src/map/terrain.ts`, `MapCanvas.tsx`):

- A chunk is painted by a generator, a stage at a time, onto a canvas the cache does not show until it is whole
  (a repaint swaps canvases). One chunk is in flight at a time, because the scratch canvases are shared.
- When only the terrain wants a frame, the map loop paints a stage (`stepTerrain`) and does not draw the map. It
  draws when a chunk lands; on a slow machine, at most every 200 ms while more of the view is still to come.
- "Slow" is measured, not guessed: a stage over 5 ms. A machine with a graphics card never takes the slow paths
  (no forced queue runs, no batching), so it behaves as before.
- Stand-ins are clipped to the gaps; a chunk whose veil is empty skips it.

Measured (long tasks while the view fills; before, after):

| Save, CPU slowdown | Long-task time | Longest task | Tasks over 200 ms |
| --- | --- | --- | --- |
| T121, 1x | 1,782 ms, 541 ms | 213 ms, 107 ms | 1, 0 |
| T121, 2x | 5,977 ms, 1,818 ms | 334 ms, 171 ms | 4, 0 |
| T121, 4x | 10,999 ms, 5,792 ms | 519 ms, 206 ms | 24, 1 |
| T280, 1x | 2,614 ms, 1,071 ms | 265 ms, 71 ms | 2, 0 |
| T280, 2x | 8,517 ms, 3,859 ms | 478 ms, 132 ms | 6, 0 |
| T280, 4x | 15,010 ms, 9,618 ms | 666 ms, 184 ms | 30, 0 |

The settled map is the same picture: 64 views (two saves, four zooms) compared pixel for pixel against master.
Time to fill a view is unchanged except at 4x on the late save (14 s to 18 s over the ten steps): the pauses are
shorter and there are more of them.

Still there: at 4x a full map frame is itself a long task (60 to 90 ms), whatever the terrain does. The next cut
is drawing only the rectangle a new chunk covers.

Gotchas:

- Anything that changes what a chunk is painted from (a new view, the art fade, a payoff hold) must drop the
  chunk in flight, or it lands half old and half new.
- Sprites that arrive while a chunk is in flight are checked when it lands, since the chunk is not in the cache
  for the arrival to find.
- `e2e/map-chunk-cache.spec.ts` (nightly) is the behaviour check for this cache. In the cloud container two of its
  six tests fail on master too (the 4x zoom-out time limit, and the late city art wait), so compare against a
  master run rather than against green.

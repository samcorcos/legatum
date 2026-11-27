# The selection mark's repaint box must cover its widest frame

The 2D map repaints only the selection ring's box on pulse-only frames (`ambientClipRects` with the ring `draw` returns).
A mark that grows (the selection burst) must report its widest reach for every frame the burst plays
(`render.ts::selectionReach`), or the clipped frames leave stale arcs of the burst outside the old box. The frame after the
burst ends is clipped to the previous, wide box, so the last arc is cleaned up without a full redraw.

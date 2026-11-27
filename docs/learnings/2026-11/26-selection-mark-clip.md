## 2026-11-26: Selection repaint boxes cover the widest burst frame

- Cause: The 2D map repaints only the selection ring's box on pulse-only frames (`ambientClipRects` with the ring returned by `draw`). A growing selection burst can extend beyond that box.
- Evidence: `app/src/map/render.ts::selectionReach` includes the burst's widest scale in the ring's reported reach. `app/src/map/MapCanvas.tsx` retains the previous ring box for the next clipped frame.
- What changed: Keep the repaint box wide through the burst. The first frame after it ends repaints that previous box, clearing the last arc without a full-map redraw.

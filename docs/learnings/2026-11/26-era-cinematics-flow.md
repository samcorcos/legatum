# Era cinematics: free AI video beats hand-built VFX (2026-11-26)

- Three days of free tooling (Blender camera moves, Mantaflow smoke, painted flame cards, an After Effects-style composite over the era painting) never produced something that read as a cut scene: animating a still painting reads as a still painting with effects. Two 8 s shots from Google Flow's free daily credits did, on the first try.
- Google Flow's free tier: 50 credits a day, no rollover; Omni Flash 1.2 at 720p is 3 credits a second (24 for an 8 s shot), so two shots a day. Downloads above 720p are subscriber-only. Video for non-subscribers may be refused in the US afternoon.
- A film inside a takeover needs its own key handling: a capture-phase `keydown` listener on `window` with `stopImmediatePropagation()` runs before `MomentLane`'s held-moment handler, so Escape ends the film without closing the card. Its `done` callback must be stable (`useCallback`), or the listener re-binds every render.
- e2e specs that drive the era takeover must skip the film first (`skipEraFilm` in `e2e/turnActions.ts`); the film covers the card's buttons.
- Chromium in Playwright plays the H.264 film, so the e2e check that `currentTime` advances is real.

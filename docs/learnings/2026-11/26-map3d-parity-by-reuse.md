# 3D map parity: reuse the 2D drawing, project it

Date: 2026-11-26

- The fastest route to 2D/3D parity was not re-modelling marks in three.js. The 2D sprite functions (banners, meters, badges, lens chips, yield icons, route ink) already draw to a 2D context; giving `drawLens` and `drawYieldIcons` an optional `at(index)` projection and `hexPx` let a transparent canvas over the WebGL scene draw them at the projected tile centres. Text stays crisp and the look stays identical.
- Hover cards needed their state lifted out of `MapCanvas` first: `useBattlePreview`, `useMapLens` and `map/orderMarks.ts` now feed both maps. Anything left inside `MapCanvas` is invisible to 3D.
- Turn playback reuses `map/combatFx.ts`: `fxUnits(plan, t)` gives positions and shifts in 2D world px; divide by `PX` (`HEX_W`) for 3D x/z. The store's `anim.start` is a `performance.now()` stamp, so the 3D scene runs on the same clock as the battle sound.
- Gotchas: run the `setAnim` effect after the view effect (the view rebuild clears `units` and would drop the ghost figures), and place a falling ghost from its plan position every frame. Subtracting the sink from the current y accumulates and buries it within a second.
- Shooting playback under swiftshader is too slow to catch a 0.5 s fall; stage the plan with `start` in the future to check the ghost stands, then let it run.

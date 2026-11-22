# Great Works on the map

The three finished Great Works and their approved construction stages were
edited as aligned raster art on 2026-10-25. The edited masters are in
`../unlock-art/masters/`; the map draws the graded `-256` copies in `app/src/assets/landmarks/`
(`landmarkArtUrl`; the research cards use the ungraded unlock copies, see
`app/src/assets/unlocks/README.md`) for finished landmarks and the construction derivatives for the build sequence.
`tools/art/landmark-atlas-edit.py` reads the prior approved pixels from the
recorded base commit and applies the same muted earth grade and ground-edge
feather to all twelve full-size frames. The building content, camera and
footprint are unchanged.

The review images in this directory put each finished landmark beside a city
on the actual rendered map, at normal (1.15) and far (0.45) zoom. They were
captured with Chromium on the cloud build box by
`app/e2e/landmarks-art.spec.ts`. Far zoom replaces the painting with the
existing gilt-ringed icon badge, so the badge and city seal should be checked
as a pair.

The only other faction-goal map landmark currently drawn by `app/src/map/` is
the Mongols' Courier Hall rose. It is drawn from the shared atlas palette in
`map/yamRelayStation.ts`, rather than from a building PNG or construction-stage set.
Other faction-goal buildings are represented in their cities but have no
separate landmark tile.

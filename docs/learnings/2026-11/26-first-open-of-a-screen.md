# A screen's first open was nobody's number (2026-11-26)

The tech tree's first open fetched 74 pictures (2.6 MB) and took 2.4 s to fill in on a 10 Mbit/s
connection; a player saw the thumbnails trickle in. Nothing measured it: the nightly check timed play
(frames, long tasks, turns), and the surface map had listed "what a first game downloads, or when" as
unwatched since 2026-11-22 without anyone taking the row.

Causes, all the same shape:

- No small picture was fetched before its screen opened. Every image is its own content-hashed file, so a
  screen with 70 thumbnails is 70 requests on first open.
- The tree asked for 128 px unlock art in 34 px boxes, and a research effect that shows a unit fell through
  to the 256 px miniature (38 of the tree's pictures were over 2.5 times their box).
- Round faction portraits (18 to 52 px, on screen all game in the rivals table) used the 360 px painting,
  60 to 80 KB each. They now have a 112 px file.

Fix: `app/src/artWarm.ts` fetches the small paintings (unit cards and thumbs, unlock and resource art at
64 px, about 1.2 MB) in idle time, four at a time, once a game is on screen, and skips it when the browser
asks to save data. The tree uses the 64 px files. After: the tree's first open fetches nothing and settles
in 0.2 s.

Watcher: `tools/perf/first-open.cjs` opens each screen once on a fresh, throttled browser and reports files,
KB, time until every picture has loaded, and pictures much larger than their box. `lean_check.py` runs it
nightly and flags a screen past 40 files, 500 KB, 1 s or any oversize picture. A new screen gets a row in
the probe's `SCREENS` list in the PR that adds it.

Rule of thumb: a picture's file should be at most about twice its box in CSS pixels times the screen's
pixel ratio. When a new art family ships, ship the size its smallest slot needs.

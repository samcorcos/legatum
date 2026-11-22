# Performance surface map

Everything Legatum does that costs a player time, memory, battery or data, with what watches each today. The
perf supervisor (`.claude/agents/legatum-perf-supervisor.md`, "Surface sweep") owns this page: once a week it
takes rows marked **unwatched**, measures them, and either records a number and a watcher or opens a fix.

Why this page exists (user, 2026-11-22): "there is surface area beyond what we have defined now that could be
optimized for performance." Every lag found so far sat in a place no benchmark looked at: hover redraws, the
upload replaying the whole game, the terrain cache after each turn. A benchmark answers the question somebody
thought to ask; this page is the list of questions nobody has asked yet.

How to read a row: **Trigger** is what makes the work run. **Grows with** is what makes it cost more (the
"costs that grow with the game" test). **Watched by** names the number that would catch a regression, or says
unwatched. First seeded 2026-11-22 from a code survey (`addEventListener`, `requestAnimationFrame`,
`setInterval`, storage, `fetch`, `measureTurn` in `app/src`); the survey commands are in the agent file.

## 1. Opening the game (before the first turn)

| Surface | Trigger | Grows with | Watched by |
| --- | --- | --- | --- |
| Download: one 3.9 MB script (unminified size, no code splitting), the engine WASM, 0.4 MB of CSS | first visit, every deploy | features added | `lean_check.py` sizes (bytes only) |
| About 930 images and 4 MB of audio in the build | first visit; each era, city and unit seen | art added | **unwatched**: nobody knows what a first game downloads, or when |
| Time from the link to a playable turn 1 (parse, WASM compile, new-game setup, first map draw) | every visit | bundle and WASM size | **unwatched** in players' games; `tools/perf/app-load.sh` exists, last run 2026-10 |
| Continue: load a save (`loadClient`, `afterLoad`, first autosave) | Continue, rewind | turns played, save size | `?perf=1` marks only; not in uploads |
| Title screen art and its animation loop | sitting on the title screen | nothing | **unwatched** (battery on a laptop) |

## 2. Every frame and every pointer move

| Surface | Trigger | Grows with | Watched by |
| --- | --- | --- | --- |
| Map draw (full and clipped frames) | camera, hover, pulses, selection | units, cities, pixels on screen | `frame_times` map draws; `battle-hover.cjs` |
| Terrain chunk cache (render, stale repaint, field rebuild) | zoom, pan to new ground, each turn | explored land, device pixel ratio | `frame_times` long tasks; `window.legatumMap.terrain` counters |
| Minimap and world backdrop loops | every frame while the map is up | map size | **unwatched** |
| React renders on hover and selection (top bar width check, panels, tooltips) | pointer move, select | cities, panel content | `frame_times` input lag only; no per-component number |
| Idle cost: frames drawn when nothing changes | a tab left open | nothing (should be zero) | **unwatched** (battery, fan) |
| Combat effects, celebrations, the era reveal, score ledger animation | battles, wonders, era change | units in the fight | **unwatched** |
| Sound: decode and mix | first gesture, every cue | cues per turn | **unwatched** |

## 3. Every turn

| Surface | Trigger | Grows with | Watched by |
| --- | --- | --- | --- |
| Engine step for the whole lobby (`end_turn` in WASM, on the main thread) | End Turn | units, cities, AI empires alive | `native-perf.sh`, `turn_profile`, `growth_profile`; in the browser `?perf=1` only |
| Reading views, applying the turn, autosave, archive, upload copy | End Turn | view size (1.2 MB late), turns played | `growth_profile`; `?perf=1` |
| Turn playback and the turn report | after End Turn | events that turn | **unwatched** |
| Order validation while planning (`validate` per candidate in purchase previews, path search, where-to-build) | opening a panel, hovering a route | cities, candidates, map size | **unwatched** |
| City and unit automation run in the page | End Turn, toggling auto | cities, units on auto | **unwatched** in the browser |
| Lists that only grow (report, notices, history, archive) | every turn | turns played | `growth_profile` (archive only) |

## 4. Memory and storage

| Surface | Trigger | Grows with | Watched by |
| --- | --- | --- | --- |
| JS heap and WASM memory over a whole game | playing | turns, views kept | `growth_profile` (native); one Mac sample 2026-11-22 (flat, 54 to 60 MB) |
| Image and canvas memory (terrain chunks at each zoom level, decoded sprites) | zooming, eras seen | device pixel ratio, levels visited | **unwatched** (the usual cause of a mobile tab being killed) |
| Browser storage: autosave, archive, preferences (22 files touch it) | every turn, settings | save size, games kept | **unwatched**: no check against the storage quota |

## 5. Network

| Surface | Trigger | Grows with | Watched by |
| --- | --- | --- | --- |
| Game upload to R2 | every few turns | record size | `growth_profile` (CPU of the copy); bytes sent **unwatched** |
| Online games: relay socket, host in the browser, lobby polling every few seconds | multiplayer | players, message size | **unwatched** |
| Guide content and sound fetched on demand | opening the Guide, first cue | nothing | **unwatched** |

## 6. Phones and tablets

| Surface | Trigger | Grows with | Watched by |
| --- | --- | --- | --- |
| Everything above on a phone CPU and GPU, on WebKit | playing on a phone | all of it | **unwatched**: every measurement so far is desktop Chromium; uploads carry no device class by design, only canvas size |

## 7. Outside the player's browser

| Surface | Trigger | Grows with | Watched by |
| --- | --- | --- | --- |
| Simulation speed and AI turn time (native) | sims, the hill climb | rules and AI code | `lean_check.py` nightly |
| CI and local test time | every PR | tests added | `lean_check.py --ci-summary`; hosted run times |
| Workers (games upload and read API, relay): CPU time and request count per game | every game | players | **unwatched**: Cloudflare's dashboard, never read |
| Deploy size and time (about 1,000 files) | every merge | art | **unwatched** |
| Repo, instruction and docs weight | every session | time | `lean_check.py` nightly |

## Sweep log

One line per sweep: date, the rows taken, what was measured, what changed in the table.

- 2026-11-22: page seeded. No rows measured yet beyond what the day's fixes covered (hover redraw, upload
  replay, terrain cache after a turn).

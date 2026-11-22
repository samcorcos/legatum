# Turn latency and app size, 2026-10-23 (baseline, master 13492d27)

What a player waits on after End Turn, on a Standard 8-empire map (48x32): seat 0 is a human
whose orders come from a Hard scripted brain, seats 1-7 are Hard AI, barbarians on. Seeds
7000001-7000004 (401, 210, 352 and 281 turns; 1,240 turns in all). Measured on the shared
64-core cloud box (load 60-70 on 64 cores, so single-thread numbers carry a few ms of noise),
single thread, `--release`.

Tools: `crates/wasm/examples/turn_profile.rs` (native; `tools/perf/native-turns.sh`),
`scripts/perf-wasm-turns.mjs` (the same games replayed through the shipped WASM API in Node/V8;
`tools/perf/wasm-turns.sh`), `tools/perf/summarize_turns.py`, `tools/perf/native-perf.sh` (perf
record with caller trees), `tools/perf/wasm-lto.sh`, `tools/perf/app-load.sh`.

## Live AI speed budget, 2026-10-27 (master a58e80856, playtest 8.9.13)

The plan's budget (`plans/ai-war.md`) is whole-lobby turn p95 under 1 s on the designer's laptop.
This section measures what 8.9.13 spends, for an 8-empire Standard game with every seat AI (no
human seat, so no human view), 6 seeds (7000001-7000006), played from turn 1 to turn 260 (two
seeds end early by victory, so T250 holds four seeds). Native is `--release` on one thread; WASM is
the shipped `wasm-release` build run in Node 23.6 (V8) on the same games (the state hash matched on
all 2,858 turns). Apple M3 Max, load average 11-13 from other jobs. Windows are the turn +/- 10;
cells are median / p95 ms. *Seat* is one AI seat's planning for one turn (all 8 seats pooled);
*lobby* is the whole End Turn (every seat's planning, resolution, history). Tools:
`crates/wasm/examples/ai_budget.rs`, `scripts/perf-wasm-budget.mjs`, `tools/perf/ai-budget.sh`,
`tools/perf/budget_summary.py`; the `seat-timing` feature of `legatum-wasm` records the per-seat
times. The full write-up, with the headroom arithmetic, is in
[`../ai-war/baseline.md`](../ai-war/baseline.md).

| Config | Window | Seat | Seat, only turns it searched | Whole lobby |
| --- | --- | --- | --- | --- |
| Native, Hard | T50 / T150 / T250 | 0.6 / 1.0, 1.6 / 3.5, 2.2 / 4.1 | n/a | 6.3 / 7.3, 16.5 / 19.9, 22.8 / 27.0 |
| WASM, Hard | T50 / T150 / T250 | 0.7 / 1.3, 1.8 / 4.0, 2.5 / 4.8 | n/a | 8.1 / 9.3, 19.1 / 23.8, 25.8 / 31.1 |
| Native, Hard+Search | T50 / T150 / T250 | 0.6 / 211, 1.7 / 659, 2.5 / 1,470 | 214 / 396, 723 / 1,153, 1,464 / 3,093 | 178 / 576, 19 / 1,684, 978 / 3,768 |
| WASM, Hard+Search | T50 / T150 / T250 | 0.7 / 275, 2.0 / 796, 2.7 / 1,765 | 291 / 524, 877 / 1,378, 1,711 / 3,320 | 225 / 763, 22 / 1,978, 1,161 / 4,916 |

Plain Hard's whole lobby is 31 ms p95 in WASM at T250 (3% of the budget). Hard+Search with every
seat searching is 2.0 s p95 at T150 and 4.9 s at T250 in WASM, over 1 s on 24% and 52% of those
turns: a seat searches on about 10% of its turns, a search costs 0.3 / 0.9 / 1.7 s in WASM at
T50 / T150 / T250, and up to 4 seats search on one turn. The head-room left for new search under
the 1 s p95 (two humans reserved at 20 ms) is about 950 ms per turn in WASM at T150 and T250,
about 155 ms per seat if all 6 AI seats use it on the same turn; divide by about 1.3 for an
M1-class laptop.

## Summary of the changes landed 2026-10-24

Five performance branches landed together (current era, supply check, End Turn redundancy,
the WASM build profile and the browser measurements). Each branch's own A/B (below) was measured against
a different master on a loaded shared box, so the per-branch figures are not additive. The
combined effect was measured once, on the same box at the same time: master `03fa1d23b` (before
any of them) against the landed branch, the same four native games (seeds 7000001-7000004, 1,240
turns, single thread, `tools/perf/native-turns.sh`). Native `end_turn` mean ms per turn; `view(0)`
is the UI's call after End Turn, now served from the view cached by `end_turn`.

| Turns | End Turn before | End Turn after | view(0) before | view(0) after |
| --- | ---: | ---: | ---: | ---: |
| 1-10 | 3.9 | 3.6 | 0.5 | 0.0 |
| 11-30 | 5.8 | 5.2 | 0.6 | 0.0 |
| 31-60 | 9.3 | 7.7 | 1.0 | 0.0 |
| 61-100 | 14.1 | 10.7 | 1.5 | 0.0 |
| 101-150 | 20.4 | 14.3 | 2.4 | 0.0 |
| 151-200 | 27.4 | 18.1 | 3.3 | 0.1 |
| 201-400 (mean) | 36.1 | 21.7 | 3.9 | 0.0 |
| 201-400 (p95 / max) | 42.8 / 45.2 | 25.4 / 29.2 | | |

A late turn, End Turn plus the follow-up view, went from 40.0 ms to 21.7 ms native (a 46% cut).
AI planning in a late turn (it builds the same city options) went from 23.9 to 14.4 ms and
rules resolution from 3.6 to 3.1 ms. Every
report, view and state hash was identical before and after on the A/B games, and no pin or
tape moved.

The WASM engine was not re-measured as a whole. Its own A/Bs, each against its own baseline,
are: the shipped build profile (fat LTO, one codegen unit, `wasm-opt -O3`) took turns 201+ from
54.3 to 43.2 ms mean (p95 63.3 to 50.9 ms) and the `.wasm` from 7.0 to 6.7 MB; the single-hash,
cached-view and cheaper-visibility changes took WASM End Turn plus `view(0)` from 59.8 to 52.0
ms. In the browser, the one app change (skipping a Chromium-only canvas readback) cut sampled
`getImageData` time on a late turn from 136 to 23 ms, but end-to-end click-to-paint was not
shown to improve (see the browser section at the end).

## End Turn, ms per turn (mean / p95 / max); view(0) is the UI's follow-up call

| Turns | Native end_turn | Native view(0) | WASM end_turn | WASM view(0) | View JSON |
| --- | --- | --- | --- | --- | --- |
| 1-10 | 5.7 / 8.3 / 12 | 0.6 | 24 / 103 / 158 (JIT warm-up) | 2.2 | 195 KB |
| 11-30 | 7.2 / 12 / 14 | 1.0 | 17 / 21 / 29 | 2.1 | 227 KB |
| 31-60 | 12.9 / 12 / 355 | 1.0 | 21 / 26 / 119 | 2.3 | 273 KB |
| 61-100 | 15.2 / 18 / 61 | 1.5 | 27 / 34 / 41 | 3.3 | 341 KB |
| 101-150 | 22.1 / 26 / 112 | 2.5 | 33 / 40 / 53 | 4.7 | 464 KB |
| 151-200 | 29.2 / 39 / 171 | 3.4 | 42 / 54 / 69 | 6.1 | 571 KB |
| 201+ | 40.8 / 59 / 176 | 4.6 | 54 / 63 / 87 | 7.5 | 661 KB |

Native splits a late turn (201+) into AI planning 27.8 ms, rules resolution 4.6 ms, and the
rest (about 8 ms) in `end_turn_json`'s own work: the human's full view built before and after
the turn, event visibility, hashing and JSON. The UI then builds the human view a third time
for `view(0)` (4.6 ms native, 7.5 ms WASM) and parses it (about 5 ms for 661 KB).

The engine is not what a player feels at 8 Hard empires: even the 201+ bucket averages 54 ms in
WASM. Spikes (a few turns per game) reach 90-350 ms.

## Removing repeated hash and human view work (2026-10-23)

The same four games and 440 turns in the 201+ bucket, measured on the shared cloud box after
each change. Each cell is mean ms per turn. `view(0)` is the app's call after End Turn; the
combined column is their sum. These are separate runs under varying box load, so small changes
in `end_turn` alone are noisy (AI planning and rules resolution also move).

| Version | Native end_turn | Native view(0) | Native combined | WASM end_turn | WASM view(0) | WASM combined |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Before, `f5d8bb796bdc` | 37.0 | 4.1 | 41.1 | 52.5 | 7.3 | 59.8 |
| Hash reused from recording | 35.6 | 4.1 | 39.7 | 51.3 | 7.4 | 58.7 |
| Post-turn human view cached | 42.2 | 0.2 | 42.4 | 53.4 | 1.4 | 54.8 |
| Pre-turn visibility uses map detail | 36.8 | 0.1 | 36.9 | 50.5 | 1.5 | 52.0 |

The view cache moves one serialization into `end_turn`, while eliminating the next full view
build. In the native cache run, AI planning plus rules resolution rose from 28.1 to 31.9 ms;
the combined native row therefore overstates the cost of the cache. The final row uses only
the map fields required by event visibility for the pre-turn snapshot; the post-turn human view
remains full and supplies the cached JSON. No AI code changed. Four 60-turn, four-player games
produced byte-identical report JSON, view JSON and hashes at every turn before and after each
change. `sim_bench views` stayed at duel `8241d9e04803f628`, eight `0999e782e1e69c0d`.

## Where the CPU goes (perf, native, whole game, frame pointers on)

| Share | Item |
| --- | --- |
| 20.6% inclusive (16.4% self + 3.8% closure) | `resources::deposits`: a full map scan building a `Vec<Deposit>` for every call of `supplied` |
| 21.3% inclusive | `resources::spare` (= `supplied` - `in_use` - `reserved`), reached from `shortage_after` for every production option's `start_check` and `buy_check` in `view::city_detail` (29% of the profile: decision and human views) |
| 5.9% self | `eras::current_era`: re-walks the tech table per call (callers: `city_detail` production options, `production_check`) |
| 3.7% inclusive | `resources::current_users`: allocates and sorts a Vec of every supply user just so `in_use` can count one resource |
| 6.2% inclusive | `GameState::hash` (`Recording::observe`, the report's hash, `Game::hash()`: up to three per turn) |
| about 15% | allocator (glibc malloc/free/realloc here; dlmalloc in WASM) |

So about 30% of all turn CPU is the strategic-supply check recomputed from scratch per
production option.

## Strategic supply in city options (2026-10-23)

Compared the branch's pre-change source (`ef0068047efe`, after merging `claude/perf`) and the
view-scoped supply change in one cloud-box slot. Both runs used `tools/perf/native-turns.sh`,
seeds 7000001-7000004, 1,240 turns, and the same instrumented `turn_profile` example. Each
cell is native `end_turn` mean / p95 / max in milliseconds. The box was shared and its load
varied during the two runs, especially in the after run's p95 and max.

| Turns | Before | After |
| --- | --- | --- |
| 1-10 | 4.1 / 4.5 / 4.7 | 4.8 / 7.3 / 9.1 |
| 11-30 | 6.2 / 7.7 / 8.1 | 6.4 / 8.0 / 13.1 |
| 31-60 | 9.8 / 12.2 / 12.9 | 10.5 / 16.9 / 24.3 |
| 61-100 | 14.9 / 18.6 / 20.9 | 14.9 / 23.2 / 65.8 |
| 101-150 | 21.4 / 26.1 / 27.8 | 19.1 / 26.0 / 71.2 |
| 151-200 | 28.2 / 37.2 / 41.2 | 25.8 / 36.4 / 156.4 |
| 201+ | 37.5 / 44.8 / 46.8 | 32.8 / 60.3 / 116.5 |

The late-turn mean fell 4.7 ms (12.5%). A separate pre-change run in a quieter slot was
30.7 ms for 201+, which illustrates why these wall-clock measurements need the same-run A/B.
The original 40.8 ms baseline at the top of this report was measured on an older master.

| `native-perf.sh` symbol (whole-process share) | Fresh before | After |
| --- | --- | --- |
| `resources::supplied`, self | 4.30% | 1.44% |
| `resources::in_use`, self | 1.67% | below 0.4% report cutoff |
| `resources::spare`, inclusive | 8.99% | below 2% report cutoff |
| `resources::deposits` | inlined into `supplied`; no separate row | no call from `supplied`; no row |

The direct tile fold and user count removed list allocation and sorting, but alone changed
the 201+ mean only 30.7 → 30.3 ms in an earlier A/B. The remaining hot path justified
one `ViewSupply` per player view: its supply, use and reservation counts are reused by every
buy and start check in that view. Order validation and turn resolution still calculate from
the current state. Debug builds assert that a cached spare count equals the uncached result.
The four games' 1,240 per-turn hashes, both `sim_bench views` digests, and the 151-hash
`search_hashes` stream were identical before and after.

## WASM build profile (same replayed games, Node/V8)

| Turns | default `wasm-pack --release` | fat LTO + 1 codegen unit | change |
| --- | --- | --- | --- |
| 61-100 | 26.8 | 20.5 | -23% |
| 101-150 | 33.1 | 27.2 | -18% |
| 151-200 | 42.2 | 34.1 | -19% |
| 201+ | 53.8 | 43.1 | -20% |
| `.wasm` size | 7,020,583 B | 6,702,359 B | -4.5% |

## Shipped WASM profile, 2026-10-23

`scripts/build-wasm.sh` now selects the dedicated `wasm-release` Cargo profile: fat LTO,
one codegen unit, Cargo opt-level 3, and `wasm-opt -O3`. Native release builds and tests keep
their previous profile. On the same four replayed games (1,240 turns), the final Node-target
build compares with plain `wasm-pack --release` as follows. Times are milliseconds per turn.

| Turns | Plain release mean | Shipped profile mean | Change |
| --- | ---: | ---: | ---: |
| 61-100 | 26.8 | 21.8 | -18.7% |
| 101-150 | 33.9 | 27.8 | -18.0% |
| 151-200 | 43.4 | 35.0 | -19.4% |
| 201+ | 54.3 | 43.2 | -20.4% |
| 201+ p95 | 63.3 | 50.9 | -19.6% |
| 201+ max | 85.6 | 73.0 | -14.7% |
| `.wasm` size | 7,020,487 B | 6,699,470 B | -4.6% |

The final Node-target clean build took 307 seconds on the cloud box. The shipped web-target
build took 306 seconds and produced a 6,699,440-byte `legatum_bg.wasm`.

The `tools/perf/wasm-profile-matrix.sh` trial used the same orders and compared both Cargo
opt levels with both Binaryen passes. The four Node replays were run sequentially on the shared
box; other jobs still varied the load.

| Cargo opt-level | `wasm-opt` | `.wasm` bytes | 201+ mean ms | 201+ p95 ms |
| --- | --- | ---: | ---: | ---: |
| 3 | `-O` | 6,702,265 | 45.6 | 53.8 |
| 3 | `-O3` | 6,699,470 | 43.5 | 50.8 |
| 2 | `-O` | 6,267,480 | 44.4 | 52.0 |
| 2 | `-O3` | 6,266,319 | 82.2 | 214.4 |

The last replay was strongly contended (its early-turn timings also more than doubled), so
that row does not establish an intrinsic slowdown. Opt-level 3 with `-O3` was the fastest
usable late-turn result and was smaller than the default LTO output.

## App bundle and load (release web build, Chromium on the box, localhost, 1000x600)

| Item | Size |
| --- | --- |
| `legatum_bg.wasm` | 7.0 MB (2.4 MB gzip) |
| main JS chunk `index-*.js` | 2.2 MB (0.92 MB gzip), one chunk for every screen; `mockClient` is already split (200 KB) |
| CSS | 329 KB (64 KB gzip) |
| `dist/` total | 60 MB, of which images 51 MB |
| Title screen (navigation to Single Player button) | 404-429 ms warm, 845-860 ms on the first runs; DOMContentLoaded 180-220 ms, load 295-350 ms; JS heap 10 MB |

## Current-era lookup, 2026-10-23

The same four native games (1,240 turns; 440 in the 201+ bucket) measured before and after the
change on the cloud box. A temporary counter in `turn_profile` recorded 2,479,479
`current_era` calls over turns 201-300 of seed 7000001, or 24,795 per turn across the game and
its shadow replay. The lookup now walks the known-tech prefix once; `city_detail` also reuses
the owner's era across its production and unavailable-option checks. `crates/ai` was unchanged.

| Measure | Before | After |
| --- | ---: | ---: |
| Native End Turn, turns 201+, mean / p95 / max ms | 36.9 / 44.2 / 45.4 | 34.9 / 41.2 / 43.6 |
| Native view(0), turns 201+, mean ms | 4.1 | 3.9 |
| AI planning, turns 201+, mean ms | 24.3 | 22.7 |
| `eras::current_era`, native `perf` self CPU | 5.93% | 0.50% |
| `sim_bench views` duel / eight digests | `8241d9e04803f628` / `0999e782e1e69c0d` | identical |

The wall-clock timings share a loaded box and can move with other jobs; the profile share and
unchanged digests provide independent checks of the targeted cost and behavior.

## Browser app measurements and canvas readback, 2026-10-24

The full release-build Chromium results, commands, 25-turn End Turn buckets, panel timings, and
300-turn memory series are in `app.md` (raw run output, not in the public repository). The box rasterizes the map in software; the map
row below is JavaScript draw time per frame, not only the FPS overlay.

| App metric | Before | After canvas readback change |
| --- | ---: | ---: |
| New Game click to painted turn 1 | 1,108 ms | 1,543 ms |
| Continue turn-200 save click to painted turn 201 | 1,937 ms | 2,462 ms |
| Late-map JavaScript draw per frame | 2.3 ms mean | 2.4 ms mean, 3.3 ms p95 |
| Cities / research / diplomacy / where-to-build first paint | 195 / 160 / 111 / 149 ms | 208 / 174 / 158 / 197 ms |
| End Turn click to painted number, turns 1–25 | 393 ms mean | 425 ms mean |
| End Turn click to painted number, turns 201–225 | 625 ms mean | 649 ms mean |
| Main-thread long-task time, turns 201–225 | 570 ms mean | 578 ms mean |
| Post-GC JS heap, turn 301 | 18.3 MB | 18.6 MB |
| WASM linear memory, turn 301 | 40.4 MB | 40.4 MB |
| Late-turn Chrome profile `getImageData` samples | 136 ms | 23 ms |

The 113 ms sampled readback reduction is the concrete CPU win. End-to-end app times were slower
in the complete after run on the shared box; the earlier 25-turn fresh-server control was 401 to
391 ms. Neither comparison proves a player-facing speedup. The UI store keeps 20 reports while
the engine's replay archive and WASM memory high-water mark grow across the game. No AI or rules
logic changed.

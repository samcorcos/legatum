# Simulation speed ledger

How fast the engine and the scripted AI play whole games, one row per change. The search AI's
rollouts multiply this loop (player_view → brain.orders → resolve_turn), so it is the number to
move.

## Current baseline (2026-10-31, cloud box, `--profile sim`)

Measured exact master `09ca07e6df6eead0b6d0ca29bee1e0cfb0d2c459` before the UI
changes on this branch. The box reported 64 cores and a one-minute load average of 44.4
after the build, falling to 42.7 by the small-map run. CPU time and digests are useful for
future same-workload comparisons; wall times and latency depend on box load. The older
2026-10-06 instruction counts below are a different metric and are not converted to CPU time.

Commands (seeds start at 7,000,001; `rollout`, `latency`, and `views` use their fixed ranges):

```
tools/sim/build-slot.sh -- cargo build --profile sim -p legatum-sim --example sim_bench
target/sim/examples/sim_bench duel 16 1
target/sim/examples/sim_bench eight 2 1
target/sim/examples/sim_bench small 4 1
LEGATUM_ROLLOUT_THREADS=1 target/sim/examples/sim_bench search 4 1
target/sim/examples/sim_bench rollout
target/sim/examples/sim_bench latency
target/sim/examples/sim_bench views
```

These ran through `tools/cloud/remote.sh master`; the raw output is in
`~/LegatumRuns/box/20261031T143840Z-791941/artifacts/perf-baseline-1031.log`.

| Mode | Games / turns | Wall; CPU | CPU s/game | Digest |
| --- | ---: | ---: | ---: | --- |
| Standard duel, Hard vs Hard | 16 / 3,870 | 21.23s; 21.24s | 1.327 | `075afcbffab3ea33` |
| Eight-player Standard, all Hard | 2 / 382 | 5.08s; 5.08s | 2.539 | `c5c3ab80ce7904f8` |
| Small-map Standard duel, Hard vs Hard | 4 / 831 | 3.14s; 3.14s | 0.786 | `0d8a4649e9737fbb` |
| Small-map Search vs Hard, one rollout thread | 4 / 790 | 65.32s; 65.32s | 16.330 | `94294ccf8f73f9ec` |

Rollout workload: eight starting games per scenario, six rollouts each, 20 turns per rollout,
one thread. Times include the current AI and engine; they are not a same-code speedup against
the 2026-10-18 section.

| Scenario | Turns/s | ms/turn | Clone µs | Digest |
| --- | ---: | ---: | ---: | --- |
| Duel from turn 30 | 638 | 1.567 | 11.9 | `bad714b643c4af3e` |
| Small duel from turn 60 | 494 | 2.023 | 9.2 | `6ac175f18d859e45` |
| Eight players from turn 100 | 68 | 14.646 | 56.7 | `62e2be50e14f877a` |

Single-game latency (wall time; each parallel/sequential pair ended in the same state):

| Scenario and seed | Parallel seats; slowest AI turn | Sequential seats; slowest AI turn |
| --- | ---: | ---: |
| Duel, 7,000,001 | 0.86s; 12ms | 1.32s; 15ms |
| Eight players, 7,000,001 | 0.80s; 4ms | 2.40s; 19ms |
| Eight players, 7,000,002 | 0.86s; 7ms | 2.64s; 24ms |

Full PlayerView JSON digests: duel `c10c1643714af74d` (four games × 80 turns);
eight players `f1a80a6e737a6107` (one game × 150 turns). The changed digests against older
sections record code and AI changes, not a performance comparison.

## How to measure

`rollout` is the search AI's workload: from a mid-game state (duel turn 30, eight-player turn
100) it times `GameState::clone` and 20-turn forward runs with fresh scripted brains.

```
cargo build --profile sim -p legatum-sim --example sim_bench   # fat LTO, ~10% faster; or --release
tools/sim/build-pgo.sh   # sim profile + profile-guided optimization, another 3-8% (target/pgo/sim/)
nice target/sim/examples/sim_bench all
nice target/release/examples/sim_bench all          # both scenarios, 1 thread then all cores
nice target/release/examples/sim_bench duel 32 1    # one scenario, 32 games, 1 thread
LEGATUM_ROLLOUT_THREADS=1 nice target/release/examples/sim_bench search   # the search game, one thread
```

| Scenario | Setup |
| --- | --- |
| `duel` | 1v1, Hard vs Hard, barbarians on, seeds 7,000,001.. (32 games single-threaded, 16 per thread in parallel) |
| `eight` | 8 players, Standard, all Hard, barbarians on, same seeds (8 games single-threaded, 3 per thread in parallel) |
| `small` | 1v1 Standard on the Small map (40x26), Hard vs Hard, same seeds (the AI-quality ledger's duel) |
| `search` | **The primary metric since 2026-10-06.** The `small` duel with seat 0 Hard+Search at `SearchConfig::default()` plus fitted plan weights (k3 h20 d4 w) and seat 1 plain Hard, seeds 7,000,001.. (4 games). Its digest covers every final state, turn count and search decision (each search's options, sample results and choice). Rollouts run on the rollout pool: `LEGATUM_ROLLOUT_THREADS=1` for instruction counts, all cores for wall time |

Map size does not scale with player count: the default map (48x32, the lobby's Medium) is used
for every player count, so a duel plays on the same map as eight players. Duels end early
(about 70 turns on average; an eight-player Standard game runs about 145).

The bench uses the runner's faction shuffle and the shared `Session` loop, without runner
telemetry, tape or archive.

## Where things stand: the search pass (2026-10-06, increments 11–15)

Billions of instructions, one thread (`--release`; search games with one rollout thread).
Digests unchanged by every row.

| | master 77bcf774 | increment 11 | now (increment 15) | change |
| --- | --- | --- | --- | --- |
| Search game, 4 games (primary) | 35.31 | 32.87 | 27.08 | −23.3% |
| Duel, 16 games | 15.10 | 15.10 | 12.36 | −18.1% |
| Small duel, 4 games | 5.19 | 5.20 | 4.29 | −17.3% |
| 8 players, 2 games | 20.10 | 20.09 | 18.25 | −9.2% |

Timed at load 16: 0.736 → 0.567 CPU s per search game on one rollout thread (1.30x).

Where a search game's time goes (one rollout thread): about 89% inside rollouts, which are
ordinary scripted turns from determinized states (planning about 55%, views 20%,
`resolve_turn` 18%), and about 11% in the real game's own turns. After increment 15 the
largest items are route searches inside closures: engine movement rounds 11.5%, the AI's
movement reconciliation 10%, campaign staging 6%, strike forces 5%, city detail 7%.

Phone chart: `chart-mobile.png` (`python3 tools/sim/perf_chart.py`; edit its data first).

## Where things stood after increment 2 (2026-10-06, `--profile sim`, load 6)

| | master a59968be | increment 2 | |
| --- | --- | --- | --- |
| Quick duel, games/s on 16 cores | 32 | 108 | target 100: met |
| Quick duel, games/s on one core | 3.3 | 8.6 | |
| 8-player Standard, CPU s per game | 3.5 | 1.23 | target under 1 |
| Small-map Standard duel, CPU s per game | 0.60 | 0.23 | |

Rows marked "Timed" carry wall-clock numbers; the others are instruction and cycle counts.

## Human-facing work in a simulated turn (2026-10-06)

Measured as the instructions removed when a simulation stops building it, with games and
every AI decision byte-identical (state digests unchanged, repin clean):

| Human-facing work | Share of a sim turn (duel / small duel / 8 players) |
| --- | --- |
| Labelled yield sources and focus reasons in every city detail, build preview and tech unlock; tech unlocks' growth forecasts; tech income-change previews | 6.4% / 7.9% / 5.1% |
| A full view (city details, research, economy, forecasts) built by the engine just to check each player's move orders against their map | 4.1% / 1.7% / 4.2% |
| legatum-sim runner telemetry (reports, opening and primitive observers) | under 1% |
| Viewer-independent view parts (titles, standings, players) rebuilt for every seat | 0.2% / 0.3% / 2.2% |

Earlier rows already removed the largest label costs on the totals-only paths (yields built
without labels for callers that read totals; labels allocated only when kept), so the shares
above are what was left at increment 3.

How it works: `ViewDetail::Decisions` (AI seats, in `Session::ai_orders` on both native and
WASM, and in the runner) leaves out fields no brain reads, and `ViewDetail::Map` is the
engine's order-check view. A human's view is always `ViewDetail::Full`. Before leaving out
another field, grep `crates/ai` (the search AI included) for every read of it; the bench
digests and `tools/sim/repin.sh` must not move. Some human-looking text is read by the AI and
must stay: the Byzantium salvage planner parses the "Tech diffusion (N ...)" line of tech costs,
and buy and production blockers are checked.

After this, the time in an eight-player Standard turn is about 70% scripted AI planning, 16%
building the AI seats' views (city details, tech unlocks, development forecasts, diplomacy),
and 13% rules resolution (movement about half).

## Columns

- `duel_ginstr`, `eight_ginstr`, `small_ginstr` and the matching `*_gcycles`: billions of
  instructions retired and cycles on one thread for 16 duels, 2 eight-player games and 4
  small-map duels (`/usr/bin/time -l sim_bench duel 16 1`, `... eight 2 1`, `... small 4 1`).
  Instructions barely move with machine load, so they are the row-to-row comparison; cycles
  track time on a P-core. Rows without timings were measured this way only; timings are taken
  when an increment lands. The digests in those rows are for these runs. Merging master's
  Byzantium squatter pricing (a59968be) made games longer and changed every digest, so it has its
  own baseline row; compare rows only within one baseline.

- `duel_games_s_1core`, `eight_games_s_1core`: games per second on one thread (wall time).
- `*_allcores`: games per second across all 16 cores. This depends on machine load (other
  agents' sims and builds), so `load_1m` records the 1-minute load average at the start.
- `duel_ms_turn`, `eight_ms_turn`: CPU milliseconds per turn, single-threaded.
- `eight_cpu_s_game`: CPU seconds per eight-player game, single-threaded (target: under 1).
- `search_ginstr`, `search_gcycles`: the same for 4 search games with one rollout thread
  (`LEGATUM_ROLLOUT_THREADS=1 /usr/bin/time -l sim_bench search 4 1`); `search_cpu_s_game` and
  `search_wall_s_game` (all rollout threads) on timed rows; `digest_search` covers every search
  decision too.
- `digest_duel`, `digest_eight`: digest of every final state hash and turn count from the
  single-threaded run. Every optimization must leave both unchanged: a different digest means
  behaviour changed.

`sim_bench views` prints a digest of every seat's PlayerView JSON on every turn of four duels
(80 turns) and one eight-player game (150 turns): view-building changes must leave it unchanged
too (master 709b4c39: duel `a145027d324bc6d0`, eight `790e9612cde74b3b`).

Timings on this shared machine are noisy (E-cores and load). For A/B comparisons of a change,
`/usr/bin/time -l` on the bench reports instructions retired and cycles elapsed, which barely
move with load.

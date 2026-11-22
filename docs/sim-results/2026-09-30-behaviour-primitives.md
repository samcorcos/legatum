# Behaviour primitives verification — 2026-09-30

Branch `astra/behaviour-primitives` is based on the completed `astra/opening-levers` commit `5808ca5d41460cb4182cfdbe6499765dcfe38d92`. It adds nine opt-in primitives, 23 bounded appended genes (48 total, schema 2), `--ai-primitives`, and per-seat diagnostics. Rules, canonical save structures, and committed fixtures are unchanged.

The [policy and counter definitions](../archive/opening-league.md#composable-behaviour-primitives-schema-2) describe the thresholds and priority interactions. [Machine-readable analysis](2026-09-30-behaviour-primitives-analysis.json) includes the command, denominators, aggregate counters, Scout checkpoints, and check results. The complete per-game report (`2026-09-30-behaviour-primitives-smoke.json.gz`, on the `evolution/league-phase2` branch, kept out of master for size) retains the dense Scout series and all individual counters.

## 100-game Quick smoke

Hard AI, barbarians on, default rules, seeds 9300001–9300100, four workers. Every seat used primitives, including Default. No optional rule levers, forced turn limit, or custom genome. All 100 games reached a VP victory: **zero stalls and zero panics**, average 138.6 turns, range 85–241, under the ordinary 600-turn safety cap. Wall time 56.3 seconds.

```sh
nice -n 10 target/release/legatum-sim --games 100 --seed 9300001 --speed quick --difficulty hard --threads 4 --ai-primitives --seat-opening all=DefendedExpand --seat-opening 0=Default --seat-opening 1=Rush --seat-opening 2=GreedyExpand --seat-opening 4=Snowball --seat-opening 5=Tall --seat-opening 6=Economic --label behaviour-primitives-smoke --out-dir target/primitives-checks/smoke
```

| Opening | Seat appearances | Wins | Win rate | Settlers captured / lost | Wars declared / fought | Coalition march turns |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Default | 100 | 21 | 21% | 37 / 69 | 71 / 52 | 87 |
| Rush | 100 | 0 | 0% | 24 / 35 | 89 / 50 | 120 |
| GreedyExpand | 100 | 24 | 24% | 32 / 59 | 66 / 51 | 42 |
| DefendedExpand | 200 | 23 | 11.5% | 64 / 109 | 148 / 86 | 142 |
| Snowball | 100 | 15 | 15% | 31 / 50 | 69 / 37 | 74 |
| Tall | 100 | 2 | 2% | 23 / 11 | 68 / 43 | 103 |
| Economic | 100 | 15 | 15% | 29 / 66 | 65 / 33 | 91 |

| Opening | Escorted / unescorted Settler turns | Mean Scouts after T100 (observed seats) | Gold spent | Maximum treasury peak | Sieges abandoned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Default | 1562 / 1583 | 1.05 (99) | 36599 | 1282 | 0 |
| Rush | 818 / 585 | 0.95 (99) | 11959 | 712 | 0 |
| GreedyExpand | 1379 / 1629 | 0.97 (99) | 53113 | 1110 | 6 |
| DefendedExpand | 2367 / 2761 | 0.99 (198) | 68244 | 1054 | 4 |
| Snowball | 1302 / 1589 | 1.05 (99) | 45978 | 1595 | 4 |
| Tall | 617 / 462 | 1.00 (99) | 29111 | 1067 | 3 |
| Economic | 1303 / 1490 | 1.01 (99) | 63489 | 1238 | 1 |

These are fixed seat assignments; DefendedExpand occupies two seats. Win-rate denominators are seat appearances, not games. The T100 sample excludes the one game that ended before then; the full report includes zero Scout counts, including eliminated seats. This is execution evidence, not a paired improvement or balance study. Rush won zero games and Tall won two: tuning, rotated matchups and held-out seeds remain necessary before asserting strategic diversity.

## Verification

All commands were prefixed with `nice -n 10`. The environment already inherits niceness 20, confirmed in a child process; attempts to add another 10 report `EPERM` and retain 20. Cargo used four build jobs; workspace tests used `RUST_TEST_THREADS=2 RAYON_NUM_THREADS=2`.

| Check | Result |
| --- | --- |
| `cargo fmt --all` and `cargo fmt --all --check` | Exit 0 |
| `cargo clippy --workspace --all-targets -j 4 -- -D warnings` | Exit 0, no warnings |
| `cargo test --workspace --release --no-fail-fast -j 4` | Exit 0; 1,049 passed, 0 failed, 4 existing ignored tests |
| `cargo test --workspace -j 4` | Exit 0; 1,049 passed, 0 failed, 4 existing ignored tests |
| Fresh Default AI tape comparison | All five tapes byte-identical: Standard mixed seeds max−2 through max, 40 turns; Hard Quick seeds 930001–930002, full games of 126/148 turns |
| Additional inherited Default AI pins | Passed; three Standard mixed 80-turn parent tape hashes |
| Primitive flag determinism | Exit 0; 3/3 full Hard Quick games, seeds 9400001–9400003, 141/137/154 turns |
| Tuned opening genome and all rule flags | Passed native determinism and save/load at every tape boundary; includes an evolved primitive Scout quota |
| `wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node` | Exit 0, release plus wasm-opt, with `CARGO_BUILD_JOBS=4 WASM_PACK_CACHE=/tmp/legatum-primitives-wasm-pack-cache` |
| `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node` | Exit 0; all 10 committed tapes matched |
| `node scripts/wasm-tape-check.mjs target/primitives-checks/flagged-tapes target/wasm-node` | Exit 0; all three flagged tapes / 432 turns matched |
| `git diff --check` | Exit 0; no edits to core, WASM, committed fixtures or `docs/learnings.md` |

The targeted tests cover triggered and untriggered behavior for all nine primitives, with additional legal-declaration, siege peace-response, opt-in and genome-boundary coverage. Observer tests cover capture direction, actual spending, dense Scout samples, one battle count per declared war, actual quota movement, and siege capture/casualty exclusions. The existing 20-game Quick opening smoke also passes.

The first complete test run exposed one old phase fixture that assumed a distant army qualified for conquest. The fixture now asserts that it does not, then stages a sufficient local force and checks that conquest does qualify. Both final check logs use that corrected test. The first WASM build could not write its default tool cache outside the sandbox; changing only `WASM_PACK_CACHE` to a writable temporary directory completed the optimized build.

The four pre-existing ignored tests are `tests::midgame_speed`, `paired_evaluation`, `many_games_summary`, and `recorded_corpus_states_and_serialized_views_match`. No ignore annotation was added.

## Boundaries

All primitive decisions use PlayerView knowledge. The separate observer can see resolution events/state only to measure outcomes; it never informs a brain. A successful declaration is not a battle, a dispatch order is not movement, and an unescorted turn is not necessarily a mistake when its threat threshold or wait timeout permits it.

Opt-in observation memory, like the parent opening phase memory, lives in the brain instance. Order tapes replay exactly, but a fresh brain loaded midway from a bare save cannot recover earlier sightings or Settler waits. The policy schema changed from 25 to 48 fields; old explicit opening genomes require the appended defaults. Default saves and no-flag AI tapes retain their old bytes.

# Balance tuning 1: opening production and city healing

2026-09-26. B8a. Adopt a **3-Production floor for the city center plus worked tiles**, reduce the first trained Settler from **80 to 75 Production**, and reduce **CITY_HEAL from 10 to 5**. Keep CITY_HP at 200. The final 1,000-game run has **81.1% VP-target victories, a 130-turn median, and 0.789 captures/game**, versus 79.0%, 134 turns, and 0.630 before. Both requested simulation pacing limits pass. No VP target, tech cost, AI knob, or other AI source changed.

The full workspace check is **not green**: faster production exposes one failure in the unchanged AI integration test, and this sandbox prevents five server protocol tests from binding localhost sockets. The complete run has 222 passing tests, six failures, and two pre-existing ignored tests. Core, simulator, replay fixtures, WASM, server unit tests, formatting, and strict Clippy pass. Details and exact failure evidence appear below; the AI failure needs the parallel AI task or lead's review before integration.

The baseline is the worktree's starting commit `3519f6d983fd3af34e5bac9bd1cb48e34fbb8dab`. The earlier [probe](balance-probe-1.md) measured `f59bb2546a7cdc4df4120b81bbcc5baaf2edb058`. This base already includes later AI campaigns, Japan aggression 70, and rule fixes, so the old probe's 0.091 captures/game is not this experiment's control. Every comparison here uses the same current AI, eight independent empires, 48×32 maps, seeds 1–1000, and a 150-turn limit. `git diff --exit-code -- crates/ai` passes.

The production floor is the small formula change in [economy.rs](../../crates/core/src/economy.rs), controlled by `CITY_MIN_PRODUCTION` in [data.rs](../../crates/core/src/data.rs). It adds only the shortfall to three after center/terrain/worked-tile production, before buildings and faction penalties. Workshops still add their full two Production. Distant Japan cities still lose 25%, and resistance still suppresses all output. The yield breakdown labels the added source “Minimum city production.” Existing center yields remain 2/1/1 and Soldier cost remains 40.

The floor addresses both founding-default Balanced focus and the scripted AI's usual early Growth focus. On the 48 unmodified generated starts used by the regression tests, original Balanced production is 1/2/3 in 11/5/32 capitals; Growth production is 1/2/3 in 37/5/6. A constant center bonus of +1 would leave the slowest starts at two. A bonus of +2 would raise 32/48 Balanced starts to five. A floor of three preserves already productive tiles and avoids raising all mature cities' output. Its tradeoff is that small terrain improvements below the floor initially contribute no extra Production; production above three and building bonuses remain useful.

At three Production, the original 80-cost Settler quotes 27 turns, outside the 25-turn upper bound. Seventy-five is the largest cost that fits; subsequent Settlers still add 20 each. The alternative with fewer changed constants, a floor of four and the original 80-cost Settler, was also checked in an isolated copy. Its initial quotes pass, but actual growth makes the Soldier finish in nine turns for seed 2, seat 4, below the requested 10-turn lower bound. The adopted floor of three plus 75-cost Settler passes both the quotes and actual economic-phase completion checks across all 48 starts.

[Opening tests](../../crates/core/tests/opening_pacing.rs) call `new_game(GameSetup::all_ai(seed))` for seeds 1, 2, 3, 7, 42, and 100 and found every capital at its real starting Settler position. They alter no terrain, population, buildings, or starting resources. Both Balanced and Growth yield three Production on turn one in all 48 cases, giving 14-turn Soldier and 25-turn first-Settler estimates at that initial rate. Separate independent builds run the real growth, production, and border phases with Growth focus, zero stored production, and no gold purchases: Soldiers finish in 10–14 turns and Settlers in 19–25. The Settler population gate is exercised too. These are durations for the first **trained** unit of each kind, not a sequential Soldier-then-Settler build order or the AI's calendar turn of completion. New games already grant one Soldier. Subsequent AI choices, production focus, purchases, or buildings can change durations.

Representative rows from the [complete before/after opening records](../sim-results/2026-09-26-tuning-1-openings.csv):

| Seed / seat | Terrain | Growth Production before → after | Soldier turns before → after | Settler turns before → after |
| --- | --- | --- | --- | --- |
| 1 / 0 | Grassland | 1 → 3 | 40 → 14 | 53 → 25 |
| 1 / 2 | Grassland | 1 → 3 | 19 → 14 | 31 → 25 |
| 2 / 2 | Plains | 2 → 3 | 20 → 14 | 40 → 25 |
| 2 / 4 | Grassland | 3 → 3 | 10 → 10 | 20 → 19 |
| 2 / 5 | Plains | 3 → 3 | 12 → 12 | 22 → 21 |
| 42 / 0 | Grassland | 1 → 3 | 26 → 14 | 36 → 25 |

Four matched rule sets separate the city experiments:

| Rules | Production floor | Settler base cost | CITY_HEAL | CITY_HP |
| --- | --- | --- | --- | --- |
| Before | None | 80 | 10 | 200 |
| Production only | 3 | 75 | 10 | 200 |
| Healing 5, adopted final | 3 | 75 | 5 | 200 |
| HP 160, independent alternative | 3 | 75 | 10 | 160 |

Each set has a 1,000-game simulator run and a separate 1,000-game replay through the existing read-only [telemetry harness](../../tools/analysis/README.md). A fifth, final `tuning-1-after` simulator run confirms all 1,000 final player records, outcomes, winners, turn counts, and elimination sequences against the healing-5 telemetry. Thus 5,000 CLI games and 4,000 telemetry games were executed, but these are repeated experiments on 1,000 seeds, not 9,000 independent maps. No run panicked or remained unfinished. The regular 1,000-game runs did not record per-turn hashes; final-record agreement and the separate cross-target tapes are distinct checks.

| Rules | VP target | Turn-limit winner | Draw | Conquest / unfinished / panic |
| --- | --- | --- | --- | --- |
| Before | 790 (79.0%) | 150 (15.0%) | 60 (6.0%) | 0 / 0 / 0 |
| Production only | 812 (81.2%) | 119 (11.9%) | 69 (6.9%) | 0 / 0 / 0 |
| Healing 5 (adopted) | 811 (81.1%) | 129 (12.9%) | 60 (6.0%) | 0 / 0 / 0 |
| HP 160 (not adopted) | 843 (84.3%) | 117 (11.7%) | 40 (4.0%) | 0 / 0 / 0 |

| Rules | Mean | Min | P10 | P25 | Median | P75 | P90 | Max | Reached turn 150 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Before | 129.10 | 63 | 95 | 121 | 134 | 146 | 150 | 150 | 218 |
| Production only | 126.39 | 59 | 90 | 118 | 130 | 143 | 150 | 150 | 194 |
| Healing 5 (adopted) | 126.31 | 59 | 90 | 119 | 130 | 144 | 150 | 150 | 196 |
| HP 160 (not adopted) | 125.89 | 59 | 90 | 119 | 130 | 142 | 150 | 150 | 165 |

| Rules | 51–75 | 76–100 | 101–125 | 126–150 |
| --- | --- | --- | --- | --- |
| Before | 13 | 107 | 207 | 673 |
| Production only | 34 | 106 | 241 | 619 |
| Healing 5 (adopted) | 36 | 104 | 243 | 617 |
| HP 160 (not adopted) | 36 | 108 | 242 | 614 |

Percentiles use the simulator's rounded order statistic, `floor(q*(n-1)+0.5)`. Reaching turn 150 includes VP victories on that turn, explaining why it exceeds turn-limit winners plus draws. Faster production alone reduces mean duration by 2.712 turns, paired 95% CI [−4.285, −1.128], while leaving the median at 130. Healing changes the mean by only −0.081 [−0.529, +0.371] relative to production alone. No compensating VP or research change is warranted: the accepted median stays within 110–150 and VP victories exceed 70%.

Faction rates use wins divided by **initial seats**, retaining draws and eliminated empires in the denominator. Brackets are 95% percentile bootstrap intervals, 10,000 resamples of whole games with bootstrap seed 20260926; this preserves competition and repeated factions within each game.

| Faction | Before | Production only | Healing 5 (adopted) | HP 160 (not adopted) |
| --- | --- | --- | --- | --- |
| Switzerland | 337/2666 = 12.64% [11.52, 13.73] | 316/2666 = 11.85% [10.75, 12.95] | 312/2666 = 11.70% [10.60, 12.80] | 333/2666 = 12.49% [11.39, 13.55] |
| Japan | 289/2667 = 10.84% [9.81, 11.90] | 289/2667 = 10.84% [9.79, 11.90] | 296/2667 = 11.10% [10.03, 12.17] | 297/2667 = 11.14% [10.09, 12.22] |
| Byzantium | 314/2667 = 11.77% [10.69, 12.87] | 326/2667 = 12.22% [11.17, 13.34] | 332/2667 = 12.45% [11.37, 13.57] | 330/2667 = 12.37% [11.32, 13.47] |

These exploratory intervals have no multiple-comparison adjustment. All three final pairwise faction-difference intervals include zero. The largest observed final gap is Byzantium minus Japan, 1.35 percentage points; this population does not establish a faction imbalance that warrants another change.

| Measure | Before | Production only | Healing 5 (adopted) | HP 160 (not adopted) |
| --- | --- | --- | --- | --- |
| Battles/game | 214.20 | 250.27 | 248.28 | 259.17 |
| Direct city-HP battles/game | 24.21 | 25.19 | 25.20 | 27.92 |
| Captures, total / per game | 630 / 0.630 | 647 / 0.647 | 789 / 0.789 | 802 / 0.802 |
| Games with a capture | 426/1000 | 426/1000 | 488/1000 | 505/1000 |
| Distinct captured cities | 591 | 605 | 735 | 766 |
| Distinct cities reduced to zero HP | 897 | 907 | 1084 | 1192 |
| Eliminations | 38 | 38 | 43 | 40 |
| Capture span P10 / median / P90 | 6 / 17 / 42 | 5 / 17 / 48 | 5 / 18 / 48 | 5 / 16 / 45 |
| Ranged share of direct city battles | 85.48% | 86.11% | 84.64% | 85.16% |

The [paired comparisons](../sim-results/2026-09-26-tuning-1-comparisons.json) isolate each city change against the production-only control. Healing five raises captures by **0.142/game [0.097, 0.187]**, a 21.9% increase, and the share of games with a capture by **6.2 percentage points [3.8, 8.6]**. Distinct captured cities increase by 0.130/game [0.090, 0.168]. VP share changes by −0.1 points [−2.0, +1.8]. This supports adopting healing five without changing the HP pool. The full final-versus-before capture increase is 0.159/game [0.082, 0.238], or 25.2%.

HP 160 also has positive evidence: **+0.155 captures/game [0.092, 0.219]**, +7.9 points of games with captures [4.4, 11.5], and +3.1 points of VP victories [0.4, 5.9]. It passes both pacing gates. It is **deferred, not declared ineffective**: the direct alternative comparison is +0.013 captures/game [−0.052, +0.080] over healing five, and combining both reductions has not been tested. Retaining 200 HP preserves the healthy city's damage budget while the narrower healing change already produces a measured capture increase. The number of distinct zero-HP cities left uncaptured is 302 with production alone, 349 with healing five, and 426 with HP 160; removing HP still leaves substantial melee-follow-through work.

The mechanical effect is targeted: a full-health strength-15 Ranger firing at a population-five walled city (strength 30) deals 16 damage at the central roll. With no adjacent enemy military, the first-turn net city loss rises from six to eleven when healing falls from ten to five. Adjacent military already suppresses healing. Ranged fire still cannot capture a city. In these runs the ranged fraction of direct city attacks falls from 86.11% to 84.64% and more distinct cities are actually captured; the benefit is not solely extra zero-HP events. This does not rule out an unsupported-bombardment problem in human play. Capture spans run from the first battle on the tile, including garrison fights and interruptions; their median rising from 17 to 18 is not evidence that continuous sieges became slower or faster.

Snowballing is measured **after resolving turn 50**. All games survive that long. Draws count as no leader win. Unique-leader and tied-leader cases remain separate; brackets below are game-level 95% Wilson intervals.

| Turn-50 measure | Before | Production only | Healing 5 (adopted) | HP 160 (not adopted) |
| --- | --- | --- | --- | --- |
| Unique leader eventually wins | 365/826 = 44.19% [40.84, 47.59] | 353/819 = 43.10% [39.75, 46.52] | 364/819 = 44.44% [41.08, 47.87] | 369/818 = 45.11% [41.73, 48.53] |
| Winner in tied leader set, tied games only | 108/174 = 62.07% [54.67, 68.94] | 109/181 = 60.22% [52.95, 67.07] | 106/181 = 58.56% [51.28, 65.49] | 113/182 = 62.09% [54.86, 68.82] |
| Winner in any leader set, all games | 473/1000 = 47.30% [44.22, 50.40] | 462/1000 = 46.20% [43.13, 49.30] | 470/1000 = 47.00% [43.92, 50.10] | 482/1000 = 48.20% [45.12, 51.30] |
| Uniform random choice among tied leaders: win rate | 41.28% | 39.91% | 40.92% | 41.65% |

The final unique leader's win rate is 44.44% versus 44.19% before; these observations do not indicate a new decisive turn-50 snowball. The rates are descriptive persistence measures, not a causal effect of having VP.

The simulator runs executed, with constants set as in the table before each build:

```sh
cargo run --release -p legatum-sim -- --games 1000 --seed 1 --label tuning-1-before
cargo run --release -p legatum-sim -- --games 1000 --seed 1 --label tuning-1-production
cargo run --release -p legatum-sim -- --games 1000 --seed 1 --label tuning-1-heal5
cargo run --release -p legatum-sim -- --games 1000 --seed 1 --label tuning-1-hp160
cargo run --release -p legatum-sim -- --games 1000 --seed 1 --label tuning-1-after
```

The first command ran on the unmodified base commit. The accepted source reproduces `heal5`/`after`; revert healing to ten for `production`, then set HP to 160 for its independent alternative. Build `tools/analysis/Cargo.toml` with `cargo build --release --locked --manifest-path tools/analysis/Cargo.toml` after each rule change, then run `tools/analysis/target/release/balance-probe --games 1000 --seed 1 --out <label>-telemetry.jsonl`. No AI override flags were supplied.

[Before report](../sim-results/2026-09-26-tuning-1-before.md), [production report](../sim-results/2026-09-26-tuning-1-production.md), [healing report](../sim-results/2026-09-26-tuning-1-heal5.md), [HP report](../sim-results/2026-09-26-tuning-1-hp160.md), and [final report](../sim-results/2026-09-26-tuning-1-after.md) have adjacent raw simulator JSON. Each of the four rule sets also has `-analysis.json` and losslessly compressed `-telemetry.jsonl.gz`. All compressed payloads round-trip exactly, and every telemetry terminal record is checked against its simulator JSON. [The manifest](../sim-results/2026-09-26-tuning-1-manifest.json) records experiment settings and artifact checksums.

To recalculate a final summary and its CIs from the retained data:

```sh
python3 tools/analysis/summarize.py docs/sim-results/2026-09-26-tuning-1-heal5-telemetry.jsonl.gz --reference docs/sim-results/2026-09-26-tuning-1-after.json --out /tmp/tuning-1-final-analysis.json
```

The extra paired intervals resample the per-seed differences, using the existing `tools/analysis/finalize.py::mean_interval` method (10,000 draws, seed 20260926). For example:

```python
import sys
from pathlib import Path
sys.path.insert(0, "tools/analysis")
from summarize import load_telemetry
from finalize import mean_interval
root = Path("docs/sim-results")
b = load_telemetry(root / "2026-09-26-tuning-1-production-telemetry.jsonl.gz")
v = load_telemetry(root / "2026-09-26-tuning-1-heal5-telemetry.jsonl.gz")
assert [g["seed"] for g in b] == [g["seed"] for g in v]
print(mean_interval([len(y["captures"]) - len(x["captures"]) for x, y in zip(b, v)]))
```

Fixtures were intentionally regenerated under the accepted rules using the exact [README procedure](../../crates/sim/tests/fixtures/README.md):

```sh
cargo run --release -p legatum-sim -- --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports
cargo test -p legatum-sim --test determinism
cargo test --release -p legatum-sim --test determinism
wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node
node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node
```

All three setups and initial hashes are unchanged; all 120 turn hashes change, beginning with turn-one city production. Later recorded AI orders change as the economy develops differently. This is expected rules-induced divergence, not a change in seed serialization or state hashing. Native debug and release each pass all six fixture tests. Optimized WASM matches all three tapes, including initial state, every turn, and continuation after save/load. The first `wasm-pack` attempt failed to write its cache outside the sandbox; copying the installed cache to `target/tuning-1-checks/wasm-pack-cache` and setting `WASM_PACK_CACHE` to that writable path made the same optimized build pass. No manifest or optimization setting was changed.

The larger README-linked S8 check also passes with the new rules: 30 games, seeds 1000–1029, up to 80 turns, **2,394 recorded turns**, identical in native debug, native release, and optimized WASM, including save/load. The exact commands use `--label tuning-1-tapes --tape-out target/tuning-1-tapes --out-dir target/tuning-1-checks`, native `--tape-in target/tuning-1-tapes`, and `node scripts/wasm-tape-check.mjs target/tuning-1-tapes target/wasm-node`.

[Check transcript](../sim-results/2026-09-26-tuning-1-checks.log):

| Check | Exact result |
| --- | --- |
| `cargo fmt --all` | Exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0, no warnings |
| `cargo test -p legatum-core` | Exit 0: 149 unit tests + 2 opening tests passed |
| `cargo test --workspace` | Exit 101, stops at AI integration failure |
| `cargo test --workspace --no-fail-fast` | Exit 101: 222 passed, 6 failed, 2 ignored; zero doctests |
| Native debug / release fixture tests | Exit 0 each: 6 passed, 0 failed, 0 ignored |
| Optimized WASM fixture replay | Exit 0: 3 tapes, 120 turn hashes matched |
| Additional native debug / release / WASM replay | Exit 0 each: 30 tapes, 2,394 turn hashes matched |
| Simulator batches | All five exit 0, 1,000 games each, zero panics |
| Telemetry verification | All four populations match; final rerun also matches healing-5 telemetry |
| AI source diff | Empty |

The six complete-suite failures are explicit boundaries of this handoff:

- `crates/ai/tests/full_game.rs::scripted_orders_all_validate_and_succeed_in_a_real_game`: seed 12, turn 56, players 0 and 1, Settlers 48 and 49 both try to found within three hexes in the same turn. Their orders pass validation individually and both fail during simultaneous resolution. The same test passes on an isolated archive of the original base commit, and fails with the same conflict under the production-only change (healing still ten). No AI code, defaults, or test filter was changed to hide this newly exposed case. The AI task needs to handle or deliberately classify rival simultaneous founding conflicts.
- All five `crates/server/tests/protocol.rs` tests fail at line 27, binding `127.0.0.1:0`, with `PermissionDenied: Operation not permitted`. They cannot exercise the protocol in this sandbox. All six server unit tests pass. Re-run the protocol tests where localhost binding is allowed.

Environment: Rust/Cargo 1.98.1, aarch64-apple-darwin, Node v26.10.0, wasm-pack 0.15.0, Python 3.14.7. The six opening seeds and one AI family are limited evidence for human game feel. Eight AI free-for-all teams do not validate the standard two-human cooperative setup. The accepted siege change and the production floor's reduced early terrain differentiation need human acceptance. The final re-run is a reproducibility check on the same seeds, not an independent balance-validation sample. Both durability reductions together remain untested.

Direct staging is blocked by the sandbox: Git cannot create `.git/worktrees/astra-balance-core-feel/index.lock`. The shared branch cannot be updated here. A commit bundle with the requested co-author trailer is prepared under `target/tuning-1-handoff/` for the lead to import; the edited worktree is retained and no merge into master was attempted.

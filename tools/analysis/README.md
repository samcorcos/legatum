# Balance probe tooling

For Pillar 10 strategy classification, forced-policy tournaments and the reusable Rust
guardrail, see [strategy diversity](../../docs/strategy-diversity.md). The independent
`strategy_diversity.py` audit accepts a plain/gzipped `legatum-sim --strategy-matchups`
report, reconciles all assigned matchup scores from per-game player records, and prints
95% bootstrap intervals using whole mirrored seed pairs. It uses only the standard library:

```sh
python3 tools/analysis/strategy_diversity.py \
  docs/sim-results/2026-09-27-strategy-diversity-forced.json.gz
```

Analysis only. This separate Cargo workspace calls the existing core, player views, brain constructors, and seed derivation; it does not alter gameplay or the production workspace manifest. The report is [balance-probe-1.md](../../docs/analysis/balance-probe-1.md).

The supplied results use source commit `f59bb2546a7cdc4df4120b81bbcc5baaf2edb058`, Rust/Cargo 1.98.1, and Python 3.14.7. Python scripts use only the standard library. Run commands from the repository root. The analysis crate has its own lockfile.

Probe runs use Standard pace with no turn limit. `--safety-cap` defaults to 1,200 turns; if the game is unfinished at the cap, it records `outcome: "stalled"` and `finished: false` without awarding victory. An optional `--turn-limit N` lets the core finish by VP at turn N; it is independent of the safety cap. Both limits accept 1 through 4294967294. Telemetry records the fixed Standard pace, optional turn limit, and effective safety cap.

Reproduce the requested simulator batches:

```sh
cargo run --release -p legatum-sim -- --games 1000 --seed 1 --label probe-1000
cargo run --release -p legatum-sim -- --games 500 --seed 5000 --brain mixed --label probe-mixed
```

Collect telemetry and the supplementary controls (substitute the simulator reports' UTC date for `2026-09-26` on a new run):

```sh
cargo build --release --locked --manifest-path tools/analysis/Cargo.toml
tools/analysis/target/release/balance-probe --games 1000 --seed 1 --out docs/sim-results/2026-09-26-probe-1000-telemetry.jsonl
tools/analysis/target/release/balance-probe --games 500 --seed 5000 --brain mixed --out docs/sim-results/2026-09-26-probe-mixed-telemetry.jsonl
tools/analysis/target/release/balance-probe --games 200 --seed 1 --aggression 75 --out docs/sim-results/2026-09-26-probe-aggression75-telemetry.jsonl
tools/analysis/target/release/balance-probe --games 200 --seed 1 --military 65 --out docs/sim-results/2026-09-26-probe-military65-telemetry.jsonl
tools/analysis/target/release/balance-probe --games 200 --seed 1 --aggression 75 --military 65 --out docs/sim-results/2026-09-26-probe-both-telemetry.jsonl
tools/analysis/target/release/balance-probe --games 500 --seed 5000 --brain mixed --random-even --out docs/sim-results/2026-09-26-probe-mixed-swapped-telemetry.jsonl
python3 tools/analysis/finalize.py --date 2026-09-26
```

`finalize.py` compresses each JSONL losslessly with a reproducible gzip header, checks every baseline game's final fields and elimination events against the original simulator JSON, recalculates aggregates and paired bootstrap intervals, and writes a SHA-256 manifest. It can also regenerate all analysis JSON directly from the supplied `.jsonl.gz` files. A checksum manifest checks artifact integrity; it is not a game-state determinism test. Original simulator wall times and timestamps will change on reruns.

To inspect one telemetry population without re-running games:

```sh
python3 tools/analysis/summarize.py docs/sim-results/2026-09-26-probe-1000-telemetry.jsonl.gz --reference docs/sim-results/2026-09-26-probe-1000.json --out /tmp/legatum-probe-1000-analysis.json
```

Run the Python regression tests (including the all-draw seed 11 case and CLI JSON output):

```sh
python3 -B -m unittest tools.analysis.test_summarize -v
```

Each JSONL record contains seed, experimental settings, final hash, final player facts and VP sources, standings after resolving turn 50, VP sources before the last turn, scoring events, captures, eliminations, action/event counts, and histogram bins. Final hashes are recorded for future replay comparison; the requested simulator runs did not record per-turn hashes. A failure in the harness aborts that population instead of silently excluding games.

Definitions:

- A battle is one `Event::Battle`, including scout combat, ranged fire, and meeting engagements. Several can occur in one turn. A city battle damages the city itself; a garrison battle hits a unit on a city tile.
- An army is Soldier + Ranger + Trooper, excluding Scouts and Settlers. Army snapshots include zero armies and eliminated seats. Per-turn army histograms include only living empires; position shares weight each living combat unit equally for each resolved turn.
- `*_army_order_*` counts submitted orders, not successful actions. A persistent movement order can resolve into multiple movement events. Counts of failed orders and bounces are retained, but the telemetry does not attribute each failure to a planner stage.
- Threat counts reproduce the current planner's visible enemy military weight within three hexes of any own city. They diagnose the marching guard, not a counterfactual count of otherwise successful marches.
- Capture spans run from the first battle on that city tile (including garrisons) to capture, inclusive. They include interruptions and, for recaptures, earlier owners' battles. They are not continuous siege duration estimates.
- `city_zero_hp_battles` counts battle events ending with the city at zero HP; simultaneous events can share the same post-stage HP. `cities_reduced_to_zero` counts distinct city IDs per game. Healing can restore HP before the end-turn snapshot.
- Title acquisition on the winning turn means a positive change in that winner's title VP. Several source categories can change together. Static VP subtraction changes every player's score in the same terminal state, without replaying strategy, extending the game, or resolving a hypothetical tie.
- Percentiles use the simulator's rounded index `floor(q * (n - 1) + 0.5)`, rather than interpolation. Faction win-rate confidence intervals resample games, preserving all eight competing seats and repeated factions. Other binary intervals use Wilson scores at the game level; fixed four-seat brain rates divide those endpoints by four. All intervals are 95%, with no multiple-comparison adjustment.
- Bootstrap replications: 10,000; seed: 20260926. Knob differences are paired by game seed. For the parity control, each seed's two brain assignments are averaged before resampling, preserving dependence.

Checks performed on the analysis tooling in addition to the required project checks:

```sh
cargo fmt --manifest-path tools/analysis/Cargo.toml --check
cargo clippy --offline --manifest-path tools/analysis/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path tools/analysis/Cargo.toml
python3 -m py_compile tools/analysis/summarize.py tools/analysis/finalize.py
```

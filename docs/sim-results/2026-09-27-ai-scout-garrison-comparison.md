# AI scout, garrison and group-attack regression comparison

The final comparison uses **500 paired Quick seeds, 700001–700500**, after merging the capital-science change from master. Each game has eight Normal scripted empires, one seat per faction, on the same 48×32 map setup. No turn limit; safety cap 600. All selected baseline games and all candidate games pass per-turn deterministic replay and final save/load checks.

## Revisions and control

- Control: the first 500 records of the committed [capital-science Quick cohort](2026-09-27-capital-science-quick.md), source `4cb53642408154d0bc2f260ae405e3f5fe68e2b9`. The original report has 1,000 games; every baseline statistic below is recomputed from indices 0–499 only. Its [source manifest](2026-09-27-capital-science-source.json.gz) records source and binary hashes.
- Candidate: `63308cbee1bf74f928c008e47f97c05f8f287107`, clean at run start, including master `6e499d1bf6d2d42a411417d5ebea0d154fa1a577`. Four control threads, eight candidate threads.

The control’s core, AI and WASM source matches that master revision byte-for-byte. The later runner change adds an optional analysis observer, which is disabled in both cohorts; its per-game runner and summary code are identical. Candidate vs merged master changes AI decisions only, with identical core, WASM, simulation and dependency files. These comparisons were checked against Git and the published source manifest. Earlier runs under the old science rules are excluded from this comparison.

## Stalls and length

| Metric | Control | Candidate |
| --- | ---: | ---: |
| Stalls at 600 turns | 1 | 0 |
| Panics | 0 | 0 |
| Determinism passes | 500/500 | 500/500 |
| Observed mean turns | 156.216 | 151.542 |
| Minimum | 67 | 74 |
| P10 | 122 | 122 |
| Median | 149 | 147 |
| P90 | 196 | 183 |
| Maximum | 600 | 386 |

Observed mean length changes by **-4.674 turns (-2.99%)**. Lengths include any safety-cap stops; those are censored observations, not completed games. Only first-play observations count; deterministic replays are not extra games.

Outcomes: control `{'vp': 499, 'stalled': 1}`; candidate `{'vp': 500}`. Stalled seeds: control `['700347']`; candidate `[]`.

## Faction ratio

Wins / all 500 initial seats per faction, including eliminated empires and capped games.

| Faction | Control wins / 500 | Control rate | Candidate wins / 500 | Candidate rate | Change (pp) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Japan | 40 | 8.0% | 39 | 7.8% | -0.2 |
| Germany | 82 | 16.4% | 79 | 15.8% | -0.6 |
| Mongols | 82 | 16.4% | 116 | 23.2% | +6.8 |
| Sparta | 32 | 6.4% | 25 | 5.0% | -1.4 |
| Byzantium | 57 | 11.4% | 45 | 9.0% | -2.4 |
| Athens | 20 | 4.0% | 32 | 6.4% | +2.4 |
| Switzerland | 83 | 16.6% | 72 | 14.4% | -2.2 |
| Celts | 103 | 20.6% | 92 | 18.4% | -2.2 |

Control max/min: **5.150×** (Celts / Athens).

Candidate max/min: **4.640×** (Mongols / Sparta).

The observed faction spread **improves in this cohort**. These are development-seed self-play results. They do not establish the ≤1.5× pillar 8 criterion at both speeds on larger held-out cohorts, or strength against a human/Proxy opponent. No faction knobs or rule bonuses were changed by this AI patch.

## Tactical scenarios and checks

Eight new scenarios bring `crates/ai/tests/tactical_combat.rs` to 18 tests. They resolve real turns from fogged views and verify legal orders and save/load stability: one-turn scout escape with exhausted exploration or enemy movement reach; safe and partially safe retreats on costly terrain; settlers escaping a blocked home route or sharing a friendly garrison tile; healthy and wounded garrison sorties; buying a defender when fortification hides an inability to counterattack; three-Soldier group attacks including three-move Mongols infantry; and rejecting distant support as an immediate fighting force. The original five added regressions fail before the fix. Response assertions cover Normal and Hard; Easy retains its deliberate missed-order policy.

- `git merge master`: passed without conflicts; final integration is `63308cb`, including `6e499d1`. No merge into master.
- `cargo fmt --all`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, no warnings.
- `cargo test --workspace`: exit 0; **555 passed, 0 failed, 4 ignored**, across 36 test/doc-test targets. The ignored cases are existing opt-in evaluations/benchmarks.
- `git diff --check`: exit 0.

Checks and the final candidate measurement were rerun after integrating capital science. Master supplies its updated fixed determinism fixtures; this AI patch does not change engine rules or regenerate fixtures.

## Reproduction and artifacts

Built separately at each recorded source revision:

```sh
cargo build --release -p legatum-sim
./target/release/legatum-sim --games 1000 --speed quick --seed 700001 --determinism --threads 4 --label capital-science-quick --status-file target/capital-science-quick-status.json
./target/release/legatum-sim --games 500 --seed 700001 --speed quick --determinism --threads 8 --label ai-scout-garrison-merged-quick --status-file /tmp/astra-ai-scout-garrison-merged-status.json
```

Select `baseline["games"][:500]` for the paired control. Quantiles use the runner’s rounded-index convention over sorted observed lengths.

- [Control raw JSON, 1,000 games](2026-09-27-capital-science-quick.json.gz); select indices 0–499. Decompressed SHA-256 `7692daf67e7c6b8f4bed2c34f4d5ea1816e6f889729f7338d0b34f91245482f6`.
- [Candidate report](2026-09-27-ai-scout-garrison-merged-quick.md) and [raw JSON](2026-09-27-ai-scout-garrison-merged-quick.json.gz). Decompressed SHA-256 `dc0e5cc93f56a2dc5b69fc20b9963aa91a71d40c65f7834ad99e306017dd3bce`. Compression was verified byte-for-byte.

Retreat exposure conservatively estimates visible attack reach from range, movement and known connectivity. Hidden reinforcements and simultaneous orders remain unknown; an attempted retreat can still be pinned by a melee attack. Route contention remains subject to the existing final movement de-duplication. No legal escape or affordable purchase can be invented when none exists.

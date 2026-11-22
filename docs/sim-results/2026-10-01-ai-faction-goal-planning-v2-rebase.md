# Faction-goal planning v2 on integrated master

The integration fails the **Mongols win-rate gate: 64/600 (10.67%) → 47/600 (7.83%)**. All other acceptance gates pass. This report compares fresh runs of master `b824eb6` and the combined branch; the earlier branch's passing result is historical. No balance values or policy parameters were retuned for this rebase.

## Integration

The starting master is `b824eb697b48f4dbff455ef597bd026a008733a2`. It already contains opening closing `11b111a`, city-fire awareness `187f165`, Tall VP planning `30922b6`, and city-assault review fixes `b824eb6`. Revision `39199cc` is incremental, so its required goal-planning parent `5086beb` was cherry-picked first (as `f97a3cb`), followed by `39199cc` (as `faa52a5`).

- `military.rs`: retain master's city-shot and simultaneous assault forecasts; reserve the branch's Sparta home guards and expedition before allocating the remaining army. Remove the superseded Sparta routine and its unused `BuildingKind` import.
- `scripted.rs`: preserve Tall growth state and the two cached faction plans. Keep Tall's gold floor inside `ordinary_gold_reserve()`, then apply the maximum of that floor and faction-goal savings. The Monument savings exception consequently cannot bypass Tall's reserve.
- Preserve master's civilian-specific exposure checks, coordinated assaults, persistent opening closing, and Tall investments. Keep active opening production priority while inserting faction-goal planning before the shared victory planner.
- Preserve both historical fixture explanations and append the fresh combined-behavior recording. No engine, faction-rule, serialization, `CLAUDE.md`, or `docs/learnings.md` changes.

[Source checks](2026-10-01-ai-faction-goal-planning-v2-rebase/integration-source-check.json) verify that the core goal-planning implementation and tests match `39199cc`, while the audited assault, closing, Tall and difficulty modules match master. [Provenance](2026-10-01-ai-faction-goal-planning-v2-rebase/provenance.json) records source and binary SHA-256 values. The baseline was built from a `git archive` of the pinned master; both simulations used frozen executable copies.

## Matched 600-seed comparison

Both executables ran seeds **3300001–3300600**, eight independent players (one of each faction), **Standard**, **scripted Hard**, Default openings, barbarians enabled, unchanged default rules, no turn limit, and the 1,200-turn safety cap. Every cohort command explicitly includes `--difficulty hard`. Each uses four workers; results are sorted and compared by seed, never by completion-order progress totals.

The [analysis script](2026-10-01-ai-faction-goal-planning-v2-rebase/analyze.py) asserts the exact seed set, setup, seat/faction/team pairing, all eight Hard scripted Default players, and absence of opt-in primitives. Goals count telemetry's banked `faction_goal_turns`, including eliminated players; wins count `winner_players`. Every percentage has 600 appearances as its denominator.

| Faction | Goals: master → combined | Wins: master → combined |
| --- | ---: | ---: |
| Japan | 176/600 (29.33%) → 183/600 (30.50%) | 69/600 (11.50%) → 81/600 (13.50%) |
| Germany | 250/600 (41.67%) → 238/600 (39.67%) | 38/600 (6.33%) → 40/600 (6.67%) |
| Mongols | 218/600 (36.33%) → 220/600 (36.67%) | 64/600 (10.67%) → 47/600 (7.83%) |
| Sparta | 82/600 (13.67%) → 363/600 (60.50%) | 61/600 (10.17%) → 77/600 (12.83%) |
| Byzantium | 593/600 (98.83%) → 584/600 (97.33%) | 105/600 (17.50%) → 99/600 (16.50%) |
| Athens | 413/600 (68.83%) → 402/600 (67.00%) | 76/600 (12.67%) → 70/600 (11.67%) |
| Switzerland | 82/600 (13.67%) → 71/600 (11.83%) | 86/600 (14.33%) → 84/600 (14.00%) |
| Celts | 261/600 (43.50%) → 265/600 (44.17%) | 101/600 (16.83%) → 102/600 (17.00%) |

| Acceptance gate | Master | Combined | Result |
| --- | ---: | ---: | --- |
| Mongols goals ≥ master | 218 | 220 | PASS |
| Mongols wins ≥ master | 64 | 47 | FAIL |
| Sparta wins ≥ 72 (12%) | 61 | 77 | PASS |
| Best/worst ratio ≤ master | 2.7632× | 2.5500× | PASS |
| No stalls or panics | {'vp': 600}; 0 panics | {'vp': 600}; 0 panics | PASS |

Mean length: **249.37 → 242.69 turns**. Full records are retained as [master](2026-10-01-ai-faction-goal-planning-v2-rebase/master.json.gz) and [combined](2026-10-01-ai-faction-goal-planning-v2-rebase/branch.json.gz); [analysis.json](2026-10-01-ai-faction-goal-planning-v2-rebase/analysis.json) also includes per-faction gained/lost seed lists.

The prior acceptance result did not survive composition with the newer master. This rebase preserves both requested policies without further tuning. The Mongols win regression remains a review blocker; its cause has not been isolated.

Mongols gains 89 and loses 87 goal completions, but gains only 27 wins while losing 44. It loses fewer settlers (275 → 199). In 28 of those 44 lost wins, Mongols still completes its goal in both versions; 12 lose the goal and four complete it in neither. Replacement winners span all seven other factions, with only four going to Sparta. The [lost-win summary](2026-10-01-ai-faction-goal-planning-v2-rebase/mongols-lost-win-summary.json) records every seed. These observations show why goal completion alone is insufficient evidence of strength; they do not isolate the causal mechanism of the win regression.

## Re-recorded fixtures and tapes

Both executables independently generated the three 40-turn mixed Normal/random fixtures, the same seeds at 80 turns, and Hard seed 3200001 at 160 turns. All setups and initial hashes match. [The complete comparison](2026-10-01-ai-faction-goal-planning-v2-rebase/fixture-comparison.json) records each changed order/hash turn and the first differing orders.

| Recording | Changed orders | Changed hashes | First difference | Combined tape FNV-1a |
| --- | ---: | ---: | --- | ---: |
| MAX-2 / MAX-1 / MAX, 40 turns | 0 / 0 / 0 | 0 / 0 / 0 | none | unchanged |
| MAX-2, 80 turns | 2 | 13 | turn 68 | 15915321984924008312 |
| MAX-1, 80 turns | 0 | 0 | none | 5205266159733089792 |
| MAX, 80 turns | 0 | 0 | none | 15256394797693125308 |
| Hard 3200001, 160 turns | 57 | 77 | turn 84 | 8031244405549724855 |

At MAX-2 turn 68, Sparta seat 2 builds a Granary instead of taking the old blanket Walls bonus before it has two homes. This retains the earlier v2 golden. At Hard turn 84, Sparta seat 5 commits city 10 to Walls instead of Workshop for its coordinated home plan. Later orders diverge as the goal plan interacts with the preserved combat policy. The Hard tape was recorded with new AI orders, and its native generator assertion was updated with it. The 40-turn files remain byte-identical after regeneration.

The seven manual scenario generators were rerun and are all byte-identical: research queue, both diplomacy slices, production queue, movement path, promotion, and alliance. [Receipts](2026-10-01-ai-faction-goal-planning-v2-rebase/manual-tape-comparison.json) record the generator commands and comparisons. Historical save/archive inputs stay frozen.

Six earlier diagnostic seeds were re-recorded in full on both binaries: 3300002, 3300026, 3300033, 3300045, 3300073, and 3300077. Their complete player records and telemetry are checked against the corresponding 600-game cohort. [Selected replay results](2026-10-01-ai-faction-goal-planning-v2-rebase/selected-replay-results.json) retain both recoveries and losses. In the combined run, seed 3300045 completes Mongols's goal on turn 214 and Sparta wins on 344; seed 3300073 completes Mongols's goal on 150 and wins; seed 3300077 completes on 165 but loses, while master completes on 161 and wins. These are paired trajectories, not independent causal estimates.

Native and optimized WASM replay both pass **30 recordings / 4,571 turns**, checking the initial state, every turn hash, and save/load at every boundary. This includes all eleven committed fixtures (615 turns), both sets of three 80-turn goldens, twelve full-game tapes, and the baseline 160-turn Hard tape.

## Check results

[Validation receipts](2026-10-01-ai-faction-goal-planning-v2-rebase/validation-receipts.json) retain exact commands, exit codes, durations, environment and compressed logs, including unsuccessful initial attempts.

| Check | Final result |
| --- | --- |
| `cargo fmt --all` | Exit **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit **0** |
| `cargo test --workspace --release --no-fail-fast` | Exit **0**; 1,152 passed, 0 failed, 4 ignored |
| `cargo test --workspace` | Exit **0**; 1,152 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | Exit **0**; 14 passed, 0 failed, 0 ignored |
| `scripts/build-wasm.sh` | Exit **0** |
| Optimized Node WASM build | Exit **0** |
| `cd app && npx tsc --noEmit && npx vitest run --maxWorkers=4` | Exit **0**; TypeScript passed; 105 Vitest files, 1,381 passed, 1 skipped |
| `git diff --check b824eb6` | Exit **0** |
| Conflict-marker scan | Exit **0** |
| Native and WASM replay checks | All twelve commands exit **0**; 30 recordings / 4,571 turns per target |

The first web WASM build exited 1 because `wasm-pack` attempted to use its unwritable shared tool cache. Copying the existing cache into the worktree and setting `WASM_PACK_CACHE=target/goal-planning-rebase/wasm-pack-cache` allowed the unchanged release build script to finish, including `wasm-opt`. No optimization was disabled.

The first full release suite exited **101**: its 1,152 executed tests passed (four ignored), but the server doctest hit `E0460` after a concurrent example build replaced the shared, unhashed `legatum_wasm.rlib`. This is the target-directory race already documented in `docs/learnings.md`. The complete required release command was rerun without competing release builds; the table above reports its actual final process exit.

The first debug workspace command also completed 1,152 tests successfully (four ignored), then exited **1** at the same server doctest after the overlapping standalone determinism build. The full `cargo test --workspace` command was rerun serially as well. Both initial failures remain in the receipts; no tests were filtered or disabled for the reruns.

## Reproduction and limits

Build the pinned master from an isolated checkout/archive and the combined branch, then copy each executable before later builds. Run each executable with:

```sh
<binary> --games 600 --seed 3300001 --players 8 --speed standard \
  --brain scripted --difficulty hard --threads 4 \
  --label <master-or-branch> --out-dir <separate-output-directory>
```

The fixture commands are in `crates/sim/tests/fixtures/README.md`; exact invocations also appear in the retained receipts. Recompute the acceptance table with `python3 docs/sim-results/2026-10-01-ai-faction-goal-planning-v2-rebase/analyze.py`. A failed acceptance gate intentionally returns exit 1. Serialize native example builds and the full Cargo suites within a target directory.

These are the same seeds used in the earlier policy tuning, not an independent held-out estimate. The 600-game Default-opening cohort does not measure Tall or non-Default opening strength; their preserved behavior is supported by source comparison and regression tests. No claim of perfect settler safety or per-seed improvement is made. Nothing was merged into master.

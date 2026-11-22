# AI faction-goal planning: matched Hard comparison

The combined policy improves Sparta but regresses Mongols and widens the measured win-rate spread. It is **not an overall balance improvement**. The implementation and the negative Mongols result are retained for review; no rule or faction-value change is included.

Frozen master: `42f5620b7e7ff0516d748c1c744e914b8e860cc8`. Branch base: `9d75a9dce45cd23cd27e82a2bd30bdfb368d77f7`; these differ only in `docs/learnings.md`. Both already include V9, so the older diagnosis describes different faction rules. This branch changes AI policy only; `crates/core`, `crates/wasm`, the workspace manifest and lockfile are unchanged from frozen master.

The shared victory planner chooses a complete pairwise-eight-hex Mongols trio, including replacement sites after three unsuitable Markets. It prices missing Markets, Workshops, settlers, travel and escorts, preserves feasible paid settler destinations, permits escape from blocked destinations, and stops after the actual award. A complete-trio search can replace an unusable cheap pair. Goal spending preserves the existing opening sequence and basic industry/defense.

Sparta shares its two home cities and distinct guards/veteran across construction, military reservations and diplomacy. It buys or constructs missing pieces through normal quotes, reserves, upkeep and resource limits. Guards are reserved before routine attacks once both Walls are built or committed; emergencies retain priority. The veteran recovers below 80 HP on home territory and follows an explicit path outside known city fire. Expedition declarations use legal actions, prefer existing wars, retain the two-war ceiling and reject treaty breaches. These policies live in the shared planner, including Default scripted and opening brains, without an opt-in switch.

## Matched results

600 games per executable, seeds **3300001–3300600**, eight independent players with one of each faction, **Standard**, **scripted Hard**, Default openings. No turn limit or rule experiment; default barbarians and city strike are on. The safety cap is 1,200 turns. Both simulation cohorts ran serially with `--threads 8`. Every player record was checked for Hard/scripted/Default, and all 600 seed/seat/faction/team assignments match.

All percentages use 600 appearances per faction, including any capped games. Goal completion is membership in telemetry's `faction_goal_turns`, including goals banked before elimination; wins are membership in `winner_players`.

| Faction | Master goals | Branch goals | Master wins | Branch wins |
| --- | ---: | ---: | ---: | ---: |
| Japan | 185/600 (30.83%) | 184/600 (30.67%) | 81/600 (13.50%) | 73/600 (12.17%) |
| Germany | 259/600 (43.17%) | 243/600 (40.50%) | 43/600 (7.17%) | 41/600 (6.83%) |
| Mongols | 224/600 (37.33%) | 180/600 (30.00%) | 55/600 (9.17%) | 30/600 (5.00%) |
| Sparta | 74/600 (12.33%) | 388/600 (64.67%) | 51/600 (8.50%) | 90/600 (15.00%) |
| Byzantium | 594/600 (99.00%) | 587/600 (97.83%) | 95/600 (15.83%) | 100/600 (16.67%) |
| Athens | 413/600 (68.83%) | 390/600 (65.00%) | 75/600 (12.50%) | 68/600 (11.33%) |
| Switzerland | 95/600 (15.83%) | 83/600 (13.83%) | 96/600 (16.00%) | 93/600 (15.50%) |
| Celts | 272/600 (45.33%) | 254/600 (42.33%) | 104/600 (17.33%) | 105/600 (17.50%) |

Best/worst win-rate ratio: **2.4186× → 3.5000×**. Master: Celts / Germany; branch: Celts / Mongols. Mean length is 249.44 → 241.91 turns, including caps. Outcomes: master `{'vp': 600}`; branch `{'vp': 600}`. Panics: 0 → 0. Capped branch seeds: none.

Mongols: 82 matched seeds gain a completion and 126 lose one. Among completed goals only, median goal turn is 198.0 → 199.5. Win-rate change: -4.17 percentage points; approximate paired 95% interval [-6.82, -1.51] points.

Sparta: 323 matched seeds gain a completion and 9 lose one. Among completed goals only, median goal turn is 260.0 → 167.5. Win-rate change: +6.50 percentage points; approximate paired 95% interval [+3.83, +9.17] points.

The intervals use the standard error of each matched seed's −1/0/+1 outcome difference. Both faction policies change together; this does not isolate either policy from changed competition. The cohort was used during implementation, so it is not a held-out balance result.

An earlier candidate spent on goals before preserving the opening and economic/defensive foundation. It failed the unchanged 20-game opening guardrail with two caps; its 600-game comparison also worsened the best/worst ratio to 3.2121× (Mongols: 32.50% goals, 5.50% wins). The final policy preserves those foundations and waits to reserve guards until the Walls are committed. The final opening cohort has one cap, seed 8120013, within the existing allowance; frozen master has zero in that cohort. No guardrail was weakened. The earlier debug run was interrupted after the revision and is not counted as a pass.

## Reproduction and evidence

Executables were frozen before comparison. Master was built from `git archive` of the SHA above in `/tmp`; branch was copied to a task-specific executable. Its AI source hashes were verified against the final files. Hashes and exact commands are in [provenance](2026-10-01-ai-faction-goal-planning/provenance.json) and [simulation receipts](2026-10-01-ai-faction-goal-planning/simulation-receipts.json).

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
# Run separately against frozen master and the frozen branch executable:
nice -n 10 <executable> --games 600 --seed 3300001 --brain scripted \
  --difficulty hard --players 8 --speed standard --threads 8 \
  --label <label> --out-dir <reports> --status-file <progress.json>
```

**Environment limitation:** every simulation command used `nice -n 10`, but this sandbox rejected `setpriority` with `Operation not permitted`; the command continued. Effective OS niceness could not be verified. The simulation worker limit was eight.

[Full master records](2026-10-01-ai-faction-goal-planning/master.json.gz), [full branch records](2026-10-01-ai-faction-goal-planning/branch.json.gz), and [calculated metrics](2026-10-01-ai-faction-goal-planning-analysis.json) retain the comparison. Run [analyze.py](2026-10-01-ai-faction-goal-planning/analyze.py) from the repository root to recalculate it from the retained compressed records.

## Fixtures and focused coverage

All three 40-turn mixed-brain tapes and seven manual scenario tapes were regenerated; all ten are byte-identical to master. The 80-turn Default mixed-brain goldens change in MAX−2 only: orders first change at turn 68, when Sparta chooses a Granary rather than the old blanket Walls preference before a two-home plan exists. Two order maps and 13 turn hashes change. MAX−1 and MAX remain identical. Frozen master independently reproduces all old digests; the final executable produces the new expected digests. Details are in the [fixture README](../../crates/sim/tests/fixtures/README.md) and [comparison](2026-10-01-ai-faction-goal-planning/fixture-comparison.json).

The six old/new 80-turn tapes are retained under this report's `master-golden-80/` and `branch-golden-80/` directories. Additional full-game scripted Hard tapes reproduce:

- Seed **3300011**: Mongols winner, 268 turns.
- Seed **3300002**: Sparta winner, 276 turns.

Both target factions complete their goals in both selected games. Winners, every player record and all telemetry exactly match the original cohort records. These are replay checks, not extra games in the statistics; see [replay receipts](2026-10-01-ai-faction-goal-planning/selected-replays.json).

Focused tests cover complete-trio replacement after three Markets, no feasible third site, blocked destinations, targeted Market investment, Workshop/garrison funding, exact two-Wall construction, protected home guards, reservation only after Walls commitment, 79-HP expedition recovery, release after goal completion, routing around city fire, unknown-route rejection, legal wars without borrowing guards or breaking pacts, and real engine goal completion at full HP.

## Checks

All commands below completed with **exit code 0** on the final source. [Validation receipts and compressed logs](2026-10-01-ai-faction-goal-planning/validation-receipts.json) retain the exact invocations and counts.

| Check | Result |
| --- | --- |
| `cargo fmt --all` | Exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --workspace --release --no-fail-fast` | Exit 0; 1,107 passed, 0 failed, 4 ignored |
| `cargo test --workspace` | Exit 0; 1,107 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | Exit 0; 13 passed |
| `scripts/build-wasm.sh` | Exit 0; release web package built |
| `wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node` | Exit 0 |
| `node scripts/wasm-tape-check.mjs … target/wasm-node` | Three exit-0 runs: 15 tapes, 1,239 turns, every initial/turn/save-load hash matches |
| Native replay of the three 80-turn and two full-game tapes | Two exit-0 runs; 784 additional turns |
| `git diff --check` | Exit 0 |
| Conflict-marker scan | None in changed/new text files |
| Engine scope and frozen-source audit | Core/wasm/manifests/lockfile unchanged; every frozen AI source hash matches |

The four ignored tests are the existing opt-in `midgame_speed`, `paired_evaluation`, `many_games_summary`, and `recorded_corpus_states_and_serialized_views_match` checks. No new ignore or relaxed assertion was introduced. The opening guardrail passes in both debug and release; its [separate 20-game comparison](2026-10-01-ai-faction-goal-planning/opening-guardrail.json) retains the one branch cap and zero master caps.

Build/test limits were `CARGO_BUILD_JOBS=4`, `RUST_TEST_THREADS=8`, and `RAYON_NUM_THREADS=8`. WASM commands used `WASM_PACK_CACHE="$PWD/target/faction-goal-checks/wasm-pack-cache"` because the default cache is outside the writable sandbox. This changes only the tool cache location; WASM optimization remains enabled.

Non-obvious observations: V9 allows Soldiers as home guards and gives Sparta military base healing of only **2 HP at home and 0 in the field** (an Aid Station raises it to 15). The expedition needs an 80-HP recovery policy. An initial one-ring enemy claim has no tile outside range-two city fire; a protected expedition needs an expanded border. Earlier completion among successful goals alone does not establish a higher overall win rate.

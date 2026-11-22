# V9 re-land on current master: scripted Hard balance

Baseline: `11db0dacdffefae9c035c1dd17ff7cc249a066ca` (master at task start).
Candidate: faithful re-land of `b25bbbe82541bdd592c7052dbf2fb8d0f3acf0f8`
on `astra/reland-balance-v9`, preserving the baseline's subsequent gameplay changes.

The fixed tuning campaigns fail pillar 8 at both speeds: Quick 2.306× and Standard
2.441×, versus 3.016× and 2.595× on the starting master. No further faction balance
changes were made in response to these results. The 500-game held-out cohorts also
fail: Quick 2.730× and Standard 1.813× (master: 4.478× and 3.355×). All 10,000
games are complete; all eight campaign processes exited 0.

Master advanced during this campaign through the real-Rush merge `744458a` and
results commit `114fcc672f64b81249cc22b42624929bf137bdd4`. This branch and the
before/after comparison remain pinned to the exact task-start master above. Those
later commits are outside this validation; this report makes no claim to have
benchmarked or integrated their AI changes.

## Implementation and conflict resolution

The original V9 rules and included AI policies were cherry-picked without committing,
then reconciled with current master. Current diplomacy and immediate surprise wars,
Iron/Coal costs, promotions, barbarians, default city strike and identity telemetry
remain in place. In particular, upgrade tests retain their Iron funding; the V9
Gold and Production surcharges do not replace resource costs. The tile card keeps
current resource-aware, viewer-faction yield presentation, using the V9 terrain math.

| Faction | V9 behavior restored |
| --- | --- |
| Switzerland | Existing income/purchase advantages; an additional 1 Gold per complete pair of paid military units, before Logistics. |
| Japan | Existing Influence/Chorus Hall strength; distant-city penalty starts beyond 6 hexes from the owned capital. |
| Byzantium | Goal awards 3 VP; ordinary buildings cost 120%; ruin and Legionary strengths retained. |
| Germany | Cheap new buildings and full Foundry output retained; every unit costs +30 base Production. |
| Celts | Hills/Cairn Singer retained; Grassland Food is 1 and ordinary infantry melee is reduced by 2. |
| Mongols | Existing mobility/no-terrain-defense identity; active Market-family cities at least 8 hexes from the owned capital earn +2 Gold. |
| Sparta | Oathbearer strength 28; Trooper line costs +80 base Production; healthy Soldiers can guard the two walled homes, but the expedition remains a healthy veteran on actual War territory; goal awards 4 VP; military healing is 2 at home, 15 at an active owned Aid Station, zero in the field. |
| Athens | Unchanged. |

The included V9 planning changes reserve an Sparta expedition, permit a prepared
legal border war without requiring a city invasion, keep Mongols network repair
active beyond a close third city, budget paired Swiss upkeep, and value the
actual faction goal awards. They are part of the requested commit, not new tuning.
The remaining Sparta and Mongols goal-planning gaps are AI-side; importing these
partial policies does not establish that either end-to-end objective is solved.

The former TypeScript client mirrors the rule changes. `rules.generated.json` now exports
faction goal VP and Japan range from Rust; the offline UI uses those values. Existing
WASM parity coverage includes costs, healing, scoring, settlement, resources,
combat and full-tree forecasts. Telemetry uses each faction's actual VP award.

The contextual diagnosis was read from the main checkout's
`.reviews/astra-faction-balance-diagnosis.md`; a checksummed copy is retained as
`diagnosis-review.md` in the external evidence bundle. Its earlier 200-game sample
used Normal, so its win rates are not substituted for the Hard measurements here.

## Fixed experiment

Each variant runs 2,000 tuning games and 500 held-out games at each speed: 10,000
games total. The cohorts were fixed before results; neither cohort was used to tune.
The independent audit verified all 80,000 player records as scripted Hard, each
faction once per game, identical paired seed/seat assignments, reconciled VP ledgers
and zero panics. Outcomes: 9,991 VP victories, eight safety-cap stops and one draw.
Every held-out game ended in a VP victory. Exact capped seeds are retained in
`campaign-audit.json` and `summary.json`.

| Cohort | Seeds, inclusive | Games per speed per variant |
| --- | --- | ---: |
| Tuning | 10010001–10012000 | 2,000 |
| Held-out | 10110001–10110500 | 500 |

Every game has eight independent AI empires, one of each faction with seed-shuffled
seats, a medium 48×32 map, current default diplomacy/resources/promotions/barbarians/
city strike, no rule experiments, no custom genome or opening, no Hard v2, and no
turn-limit victory. Both variants use the same seeds and faction seats at both speeds.
The default safety caps remain 600 Quick / 1,200 Standard turns. Capped games stay
in the all-game denominator and award no win. Goal-completion and terminal-observation
statistics include capped games as censored observations; completion-turn medians
include only goals actually completed.

The master executable was built and copied before the cherry-pick. V9 was built
and copied after resolving the gameplay changes. Each campaign invokes its immutable
executable with `nice -n 10`, four Rayon workers; at most two campaigns run together
(eight simulation workers total). Example (repeat for each speed/variant/cohort):

```sh
nice -n 10 /tmp/reland-v9/bin/v9-sim \
  --brain scripted --difficulty hard --players 8 --map-size medium --threads 4 \
  --games 2000 --seed 10010001 --speed quick --label v9-quick-tuning \
  --out-dir /tmp/reland-v9/v9/quick-tuning \
  --status-file /tmp/reland-v9/v9/quick-tuning/status.json
```

For held-out runs change `--games` to 500, `--seed` to 10110001 and the label/directory
to `held-out`. For baseline use `bin/master-sim`; for Standard use `--speed standard`.
`command.json` beside each report retains exact arguments, executable SHA-256,
timestamps, duration and exit status. Simulator checkout metadata describes the
working checkout at launch, not the copied binary; the frozen executable receipts
and source hashes identify the actual comparison.

Environment limitations: the filesystem policy denied creation of the requested
`~/LegatumData/sim-artifacts/reland-v9/`, so artifacts are staged at
`/tmp/reland-v9/`. `nice` was invoked, but macOS denied `setpriority`; effective
niceness 10 is not verified. No permission workaround was used.

## Results

Win rates below are wins divided by all scheduled games, including safety-cap stops
and draws. Each faction appears once per game. The ratio is the highest faction win
rate divided by the lowest; the same denominator cancels in that ratio.
These are finite-sample results for this scripted Hard policy, not estimates of human play.

| Speed / cohort | Games per variant | Master best / worst | Master ratio | V9 best / worst | V9 ratio | V9 pillar 8 |
| --- | ---: | --- | ---: | --- | ---: | --- |
| Quick tuning | 2,000 | Switzerland 19.45% / Sparta 6.45% | 3.016× | Byzantium 19.95% / Germany 8.65% | 2.306× | FAIL |
| Quick held-out | 500 | Switzerland 20.60% / Sparta 4.60% | 4.478× | Byzantium 20.20% / Germany 7.40% | 2.730× | FAIL |
| Standard tuning | 2,000 | Switzerland 19.20% / Sparta 7.40% | 2.595× | Celts 18.55% / Germany 7.60% | 2.441× | FAIL |
| Standard held-out | 500 | Switzerland 20.80% / Sparta 6.20% | 3.355× | Byzantium 17.40% / Germany 9.60% | 1.813× | FAIL |

### Quick, tuning (2,000 games per variant)

| Faction | Master wins | Master win rate | V9 wins | V9 win rate | Change, percentage points |
| --- | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 389 | 19.45% | 289 | 14.45% | -5.00 |
| Japan | 287 | 14.35% | 217 | 10.85% | -3.50 |
| Byzantium | 263 | 13.15% | 399 | 19.95% | +6.80 |
| Germany | 231 | 11.55% | 173 | 8.65% | -2.90 |
| Celts | 328 | 16.40% | 299 | 14.95% | -1.45 |
| Mongols | 152 | 7.60% | 177 | 8.85% | +1.25 |
| Sparta | 129 | 6.45% | 201 | 10.05% | +3.60 |
| Athens | 217 | 10.85% | 243 | 12.15% | +1.30 |

Master: capped games 4, draws 0, mean 136.15 turns. V9: capped games 1, draws 1, mean 130.18 turns.

### Quick, held-out (500 games per variant)

| Faction | Master wins | Master win rate | V9 wins | V9 win rate | Change, percentage points |
| --- | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 103 | 20.60% | 69 | 13.80% | -6.80 |
| Japan | 65 | 13.00% | 54 | 10.80% | -2.20 |
| Byzantium | 62 | 12.40% | 101 | 20.20% | +7.80 |
| Germany | 65 | 13.00% | 37 | 7.40% | -5.60 |
| Celts | 77 | 15.40% | 87 | 17.40% | +2.00 |
| Mongols | 40 | 8.00% | 48 | 9.60% | +1.60 |
| Sparta | 23 | 4.60% | 41 | 8.20% | +3.60 |
| Athens | 65 | 13.00% | 63 | 12.60% | -0.40 |

Master: capped games 0, draws 0, mean 135.14 turns. V9: capped games 0, draws 0, mean 131.36 turns.

### Standard, tuning (2,000 games per variant)

| Faction | Master wins | Master win rate | V9 wins | V9 win rate | Change, percentage points |
| --- | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 384 | 19.20% | 287 | 14.35% | -4.85 |
| Japan | 304 | 15.20% | 238 | 11.90% | -3.30 |
| Byzantium | 185 | 9.25% | 270 | 13.50% | +4.25 |
| Germany | 245 | 12.25% | 152 | 7.60% | -4.65 |
| Celts | 354 | 17.70% | 371 | 18.55% | +0.85 |
| Mongols | 158 | 7.90% | 224 | 11.20% | +3.30 |
| Sparta | 148 | 7.40% | 212 | 10.60% | +3.20 |
| Athens | 221 | 11.05% | 244 | 12.20% | +1.15 |

Master: capped games 1, draws 0, mean 260.31 turns. V9: capped games 2, draws 0, mean 252.90 turns.

### Standard, held-out (500 games per variant)

| Faction | Master wins | Master win rate | V9 wins | V9 win rate | Change, percentage points |
| --- | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 104 | 20.80% | 59 | 11.80% | -9.00 |
| Japan | 77 | 15.40% | 63 | 12.60% | -2.80 |
| Byzantium | 46 | 9.20% | 87 | 17.40% | +8.20 |
| Germany | 59 | 11.80% | 48 | 9.60% | -2.20 |
| Celts | 78 | 15.60% | 78 | 15.60% | +0.00 |
| Mongols | 42 | 8.40% | 49 | 9.80% | +1.40 |
| Sparta | 31 | 6.20% | 54 | 10.80% | +4.60 |
| Athens | 63 | 12.60% | 62 | 12.40% | -0.20 |

Master: capped games 0, draws 0, mean 256.65 turns. V9: capped games 0, draws 0, mean 246.53 turns.

### Goal and pacing diagnostics

These are observed outcomes of the whole V9 bundle, not isolated causal estimates
for individual rules. Goal completion is counted across every scheduled game;
completion medians exclude non-completions. Second-city medians use actual founding
events, not captures.

Quick tuning:

| Faction | Goal completion, master → V9 | Goal median turn, master → V9 | Second-city median, master → V9 |
| --- | ---: | ---: | ---: |
| Switzerland | 24.55% → 14.90% | 146 → 145 | 28 → 28 |
| Japan | 22.20% → 20.05% | 118 → 118 | 27 → 26 |
| Byzantium | 99.55% → 99.50% | 77 → 77 | 27 → 27 |
| Germany | 57.95% → 47.90% | 115 → 116 | 35 → 38 |
| Celts | 33.10% → 31.60% | 103 → 101.5 | 28 → 27 |
| Mongols | 17.15% → 23.15% | 105 → 107 | 32 → 32 |
| Sparta | 34.55% → 14.70% | 122 → 131 | 28 → 28 |
| Athens | 67.20% → 64.50% | 101 → 102 | 28 → 28 |

Standard tuning:

| Faction | Goal completion, master → V9 | Goal median turn, master → V9 | Second-city median, master → V9 |
| --- | ---: | ---: | ---: |
| Switzerland | 23.65% → 14.35% | 281 → 282 | 54 → 54 |
| Japan | 36.35% → 31.50% | 219 → 217 | 47 → 47 |
| Byzantium | 99.10% → 99.00% | 154 → 155 | 54 → 54 |
| Germany | 54.45% → 44.30% | 231 → 233 | 68 → 73 |
| Celts | 46.10% → 45.15% | 201 → 198 | 54 → 50 |
| Mongols | 34.95% → 39.70% | 199 → 197 | 61 → 60 |
| Sparta | 41.35% → 15.25% | 226 → 250 | 52 → 52 |
| Athens | 74.00% → 72.15% | 195 → 192 | 54 → 54 |

### What the Quick failure means

The 1.5× requirement is a pairwise spread limit, not a separate target of exactly
12.5% for every faction. In Quick tuning, Byzantium (19.95%), Celts (14.95%) and
Switzerland (14.45%) each exceed 1.5× Germany's 8.65%. Mongols is almost as low
at 8.85%. V9 improves the overall spread from 3.016× to 2.306× but does not pass.

Byzantium completes its goal in 99.5% of V9 Quick games, at median turn 77 among
completions. Its extra permanent VP is therefore a highly reliable payoff: with
all five milestone points, the goal leaves two live titles needed instead of three.
That is consistent with the win-rate increase despite the deeper building surcharge;
the bundled experiment does not isolate the VP change from the other rules/policies.

Germany's second founding moves from median turn 35 to 38, while goal completion
falls from 58.0% to 47.9%. Its cheap buildings and Foundry strength are intact, but
V9 deepens every unit's cost and the surrounding race ends sooner (mean Quick game
length 136.15 → 130.18 turns). These observations support an expansion/paid-goal
timing problem; they do not prove the unit surcharge alone caused the decline.

Sparta improves from 6.45% to 10.05% wins while completing its goal much less
frequently (34.55% → 14.70%; median completed-goal turn 122 → 131). Recorded Trooper
production completions plus upgrades fall from 7.80 to 0.55 per empire. The +80
base Production surcharge now coexists with Iron costs and harsh field recovery.
Of the 2,000 V9 empires, 872 fail the goal and complete fewer than two Walls builds.
The larger award pays when achieved, but the full preparation/health/expedition plan
remains unreliable. Walls build counts are evidence of investment, not a substitute
for checking captured or active walled cities.

Mongols improves goal completion from 17.15% to 23.15%, but 874 V9 empires complete
at least three Markets without earning the goal. V9's network-size fix and distant
commerce are partial improvements; usable pairwise spacing, continued ownership,
active Markets and the cost of reaching/defending them still need coordinated planning.

### What the Standard failure means

Standard tuning also fails: Celts wins 18.55% and Germany 7.60%, a 2.441× spread
versus master's 2.595×. Celts, Switzerland (14.35%), Byzantium (13.50%),
Athens (12.20%) and Japan (11.90%) each exceed 1.5× Germany's rate.
Sparta (10.60%) and Mongols (11.20%) improve but still trail Celts by more
than 1.5×. These are relative pairwise failures, not eight independent balance targets.

Celts's goal remains frequent (46.10% → 45.15%), and its second founding is earlier
(54 → 50 median turns), despite the Grassland/infantry drawbacks. At the final
observation, its Most Populous holdings fall (772 → 580 of 2,000), while Largest
Economy rises (308 → 408) and Strongest Army rises (339 → 377). The population
drawback has a visible associated effect, but it does not remove Celts's overall
winning advantage in this whole-bundle comparison. These terminal holdings are
not a causal decomposition of wins.

Germany's goal completion falls from 54.45% to 44.30%; second founding shifts
68 → 73 median turns, and completed goals shift 231 → 233 while mean game length
falls 260.31 → 252.90 turns. The same paid expansion/goal timing concern seen in
Quick is present at Standard. Byzantium's reliable goal (99.00%, median turn 155)
is consistent with its win increase from 9.25% to 13.50%, but it is not the Standard
tuning leader. A Quick-only assessment would conceal that speed difference.

Sparta again wins more while completing its objective less: 7.40% → 10.60% wins,
41.35% → 15.25% goal completion, and median completed-goal turn 226 → 250.
Trooper production plus upgrades fall from 8.21 to 0.46 per empire; 807 V9 empires
fail the objective and complete fewer than two Walls builds. Mongols goal completion
improves 34.95% → 39.70%, yet 642 empires complete at least three Markets without
earning it. The diagnosis review's coordinated goal-planning gaps remain AI-side
at both speeds; neither count alone identifies the missing condition in every game.

### Held-out confirmation

Neither held-out cohort passes. Quick improves from master's 4.478× to V9's
2.730×: Byzantium wins 20.20% and Germany 7.40%. Byzantium, Celts (17.40%),
Switzerland (13.80%) and Athens (12.60%) each exceed 1.5× Germany.

Standard improves from 3.355× to 1.813×: Byzantium wins 17.40% and Germany 9.60%;
Celts wins 15.60%, and Mongols is nearly as low as Germany at 9.80%. Both Byzantium
and Celts exceed 1.5× Germany. The Standard leader switches from Celts in tuning to
Byzantium in held-out, so the identity of the strongest faction is less stable than
the failure itself. All four V9 cohorts exceed 1.5×, and Germany is last in all four.
The held-out samples contain 500 games each, with no draws, safety-cap stops or panics;
they confirm the empirical failure without implying exact population win rates.

### Candidates for a separate follow-up; none implemented here

The diagnosis review's Sparta and Mongols planning gaps are **AI-side**.
The next Sparta candidate is an explicit paid plan for two Walls, two protected
healthy home guards and one 80+ HP veteran expedition, including Iron/Gold/Production
and recovery, legal expedition wars, and city-shot avoidance. V9 already supplies
Soldier home guards, the 4-VP payoff, stronger Oathbearers and part of the expedition
policy; repeating those changes is not the next experiment.

For Mongols, keep the plan active until the actual goal is complete, evaluate a
feasible trio with its Markets, travel and escort costs together, and repair it after
close settlements or losses. V9 already removes the old three-city cutoff and pays
+2 distant Market-family Gold; that is not evidence of a complete coordinated plan.

The review also proposes doubled Byzantium barbarian-camp Gold (25 → 50 Quick,
50 → 100 Standard) with safe camp-expedition AI. Treat that as a deferred payoff
hypothesis: the current Hard Quick results already put Byzantium first, so these
results do not support adding that buff now. Its reviewed 3-VP goal / 120% building
tradeoff is already present.

For a persistent Celts or Japan lead, the review's drawback directions are stronger
non-hill/ordinary-infantry dependence and a tighter cost for distant Japan colonies,
while preserving the hill/ranged and Influence strengths. V9 already applies the
review's specific Grassland/infantry and six-hex values; any deeper version requires
a new experiment. The review gives no next numerical Germany proposal beyond V9;
its present weakness calls for a separate examination of the paid Complete City
payoff and expansion opportunity cost, not an unmeasured adjustment in this re-land.

The review's matched city-strike on/off comparison, separately for Normal and
explicit Hard, would help distinguish the city-shot forecasting gap from faction
rule effects. No city-strike ablation or additional tuning was performed here.

## Fixture provenance

V9 changes rules and AI decisions, and its engine source changes the conservative
rules fingerprint. All ten committed tape generators were rerun under the current
rules; only three tapes changed. Tape state hashes do not themselves contain the
source fingerprint. All setups, faction seats and initial hashes remain unchanged.
The mixed-brain fixtures deliberately retain Normal scripted seats; they are engine
replay tests, separate from the explicitly Hard balance campaign.

| Fixture | Changed turn hashes | First hash change | Changed order maps | First order change | Reason |
| --- | ---: | ---: | ---: | ---: | --- |
| MAX-2, seed 18446744073709551613 | 40/40 | 1 | 15/40 | 24 | V9 faction economics/combat resolve differently and later alter scripted planning. |
| MAX-1, seed 18446744073709551614 | 30/40 | 4 | 0/40 | none | Identical orders resolve under V9 rule values; no AI tape change is needed. |
| MAX, seed 18446744073709551615 | 40/40 | 1 | 20/40 | 15 | V9 resolution feeds back into scripted decisions. |
| Research queue | 0/120 | none | 0/120 | none | Generator output byte-identical; retained. |
| Diplomacy slice 1 | 0/12 | none | 0/12 | none | Generator output byte-identical; retained. |
| Diplomacy slice 2 | 0/15 | none | 0/15 | none | Generator output byte-identical; retained. |
| Production queue / purchases | 0/150 | none | 0/150 | none | Generator output byte-identical; retained. |
| Movement path | 0/8 | none | 0/8 | none | Generator output byte-identical; retained. |
| Promotion | 0/12 | none | 0/12 | none | Generator output byte-identical; retained. |
| Diplomacy alliance | 0/18 | none | 0/18 | none | Generator output byte-identical; retained. |

The generated UI rules snapshot changes only to add the exported V9 goal awards and
Japan range. The historical compressed settler/save/archive fixtures are not rewritten.

The newer `runner::tests::default_ai_tapes_are_pinned` golden values cover the same
three seeds for 80 mixed-brain Standard turns. The frozen master executable
independently reproduced the old full-tape FNV-1a values, and the frozen V9 executable
reproduced the new values below. Their change is caused by the intentional V9
resolution/planning changes, not by the source fingerprint itself. `golden-80/`
retains both sets of tapes and commands. The first release check found this one
unrefreshed golden test (exit 101); the already-compiled debug run also used the old
expectations and exited 101 on the same test. The expectations were updated after
this audit, and both full suites then passed against the final source.

| Seed | Master full-tape digest | V9 full-tape digest |
| --- | --- | --- |
| MAX-2 | `15030415405075546679` | `5639197132889317457` |
| MAX-1 | `6699008936867188249` | `5205266159733089792` |
| MAX | `7820597218741664499` | `15256394797693125308` |

V9's original route regression adaptation remains: it checks the recorded historical
three-waypoint circuit and current legal forward progress, while continuing its five
route-only turns if the larger VP awards end the historical late-game state early.
This is documented beside that fixture and does not change production victory rules.

Generation commands are in `crates/sim/tests/fixtures/README.md`, with exact invocations
and comparison rows in external `fixture-commands.json` and `fixture-diff.json`.
All ten tapes cover 455 turns, with save/load at every boundary.

## Checks

Final receipts live in `checks/`. The first release/debug runs exposed only the old
80-turn golden expectation described above. The first app run exposed one outdated
Celts Grassland + Wheat expectation (3 Food → 2 = 1 terrain + 1 resource); that test
now documents the intentional V9 change. No additional gameplay adjustment was made.

Initial WASM builds generated working bindings but exited 1 when `wasm-pack` tried
to write its tool cache outside the writable roots. Setting
`WASM_PACK_CACHE=/tmp/reland-v9/wasm-pack-cache` and copying the existing tool cache
there allowed the unchanged build script and `wasm-opt` optimization to complete.
All final WASM/app checks below ran on those optimized outputs.

| Required check | Final result |
| --- | --- |
| `cargo fmt --all` | Exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0. |
| `cargo test --workspace --release --no-fail-fast` | Exit 0; 1,074 passed, 0 failed, 4 ignored. Full exit code checked. |
| `cargo test --workspace` | Exit 0; 1,074 passed, 0 failed, 4 ignored. |
| `cargo test -p legatum-sim --test determinism` | Exit 0; 13 passed. |
| `scripts/build-wasm.sh` | Exit 0, including `wasm-opt`, with writable tool cache. |
| Node WASM build + `scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node` | Exit 0; 10 tapes / 455 turns and every save/load boundary. |
| WASM replay of refreshed 80-turn goldens | Exit 0; 3 tapes / 240 turns. |
| `cd app && npx tsc --noEmit` | Exit 0. |
| `cd app && npx vitest run --maxWorkers=4` | Exit 0; 105 files passed, 1,372 tests passed, 1 intentionally skipped. Mock/WASM parity tests ran. |
| `cd app && npm run build` | Exit 0. |
| `git diff --check`; conflict-marker / unmerged-entry scan | Exit 0 for diff checks; no conflict markers or unmerged entries. |

Rust's four ignored tests are the existing performance/paired-evaluation/corpus
checks (`midgame_speed`, `paired_evaluation`, `many_games_summary`, and
`recorded_corpus_states_and_serialized_views_match`). The one app skip is the mock
iteration of the WASM-only recording lifecycle test, not missing WASM availability.


## Artifacts

Requested directory: `~/LegatumData/sim-artifacts/reland-v9/`. Actual staging directory: `/tmp/reland-v9/` (filesystem restriction described above).

| Variant | Executable SHA-256 | Rules fingerprint |
| --- | --- | --- |
| master | `6dd4968a94fac481b6e2148eb8ea0672ceadb8ad85b731139f7c9fa056f182ba` | `577f40efdbfb8260b8e33fb2cc89b3f0a9393a7d9210888e6e2d9a5fed847fe1` |
| v9 | `6d87ff8d46023048e7449bec2e21a464a9f57f34966bd28036a842a74565ed9c` | `5ec8840623aaf69fd2f2895eae3b0666c0e70cbc2b8239a0ed7aa4bc2705a351` |

Both fingerprint strings were verified inside their respective frozen executables.
`v9-sources.json` pins the source at benchmark compilation; the subsequent `runner.rs`
edit changes only its test golden expectations and comment, not gameplay or this
rules fingerprint. `summary.json` retains the independently aggregated cohort results;
raw JSON/Markdown reports, command receipts, logs, source diff and fixture audits stay
outside Git.

Evidence bundle: `/tmp/reland-v9/reland-v9-artifacts.tar.gz`
(75,728,548 bytes, 72.22 MiB). It contains
177 evidence files plus `manifest.json` and `SHA256SUMS`;
all 179 archived files were read back and matched to their SHA-256 values.
Build-tool caches are excluded. `package-receipt.json` records the bundle receipt.

- Bundle SHA-256: `9b176499e43b8b06a0d52436f0d48990596cb739022f980250ab89d2675eeef3`.
- Manifest SHA-256: `59d3a06ef6ba790a0f59dce51594cc7c27d027432b66476c5c99e793954a79ec`.
- `SHA256SUMS` SHA-256: `1b4bf5563e5a6654538cb8ab9af08b216c5cff237c005f3f7b43991a074a6edc`.

`campaign-audit.json` records the 10,000-game reconciliation and eight-worker peak;
`source-and-check-audit.json` records the unchanged production source and required
check exits. The bundle also preserves the diagnosis review and compiler/runtime
versions. The requested durable artifact location still requires a permitted copy
from this staging directory.

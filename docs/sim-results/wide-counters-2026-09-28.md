# Structural costs of going wide — 2026-09-28

Recommend **no default-rule change**. None of the four isolated structural costs or either
combination achieves the requested counter structure. Wide remains ahead of every other
forced policy on point estimates, and has no demonstrated clear counter in any condition.
There is therefore no evidence-backed smallest successful lever set to recommend.

The study schedules **4,200 games**, with zero panics. The common completed cohort has
**570 games per condition**, and **34–40 games / 17–20 complete mirrored maps per matchup**.
Fifteen maps are excluded across all conditions because at least one orientation was capped
somewhere. Full matrices, counter lists, faction wins/seats, policy behavior, length ranges
and paired intervals are in the generated appendix below.

## Results and recommendation

All rows fail both requested graph conditions: every policy has a clear counter = **No**;
none beats every other policy on point estimates = **No**. Wide is the point-dominant policy
in every row. Entropy alone would miss this failure.

| Condition | Wide / Economic placement | Winner entropy assigned / observed | Faction best/worst | Mean / median turns | Wide seat win rate |
| --- | ---: | ---: | ---: | ---: | ---: |
| control | 60.8% | 0.9040 / 0.7251 | 6.318× | 142.6 / 137 | 22.70% |
| settler-escalation | 55.8% | 0.9239 / 0.7116 | 4.571× | 147.1 / 142 | 19.77% |
| city-research | 59.7% | 0.9153 / 0.7341 | 5.417× | 145.6 / 139 | 21.81% |
| city-upkeep | 57.3% | 0.9275 / 0.7536 | 8.222× | 150.6 / 144 | 19.01% |
| expansion-coalitions | 60.3% | 0.9083 / 0.7188 | 5.231× | 144.5 / 138 | 21.81% |
| settler-research | 54.0% | 0.9223 / 0.7227 | 4.778× | 148.6 / 143 | 19.77% |
| settler-coalitions | 57.3% | 0.9256 / 0.7291 | 4.062× | 148.4 / 143 | 19.64% |

The stronger Settler ramp is the most promising single lever for further investigation,
not a release recommendation: it lowers Wide's score against all five opponents and improves
the faction ratio from 6.318× to 4.571×, but Wide still has no counter. The existing default
already has a rising Settler price; simply adding that familiar mechanic was not a missing
feature. This trial strengthens it.

`settler-research` combines `settler-escalation,city-research`. It was selected from the
completed cost trials: both improved the faction ratio individually, while upkeep failed
that screen. It produces the closest Wide/Economic matchup, **54.0%**, with a descriptive
95% interval of **[49.6%, 58.4%]** for Wide. An uncertain advantage is not an Economic counter.
Wide still scores **90.0% versus Rush**, **63.4% versus Tall**, **63.2% versus Science**, and
**70.8% versus Turtle**. Economic also loses its *clear* counter in this sample; this is not
a successful cycle.

`settler-coalitions` combines `settler-escalation,expansion-coalitions`. It tests the strongest
cost lever with a different mechanism after the coalition-only result passed the faction
regression screen. It has the best faction ratio, **4.062×**, but Wide remains clearly ahead
of Economic at **57.3%**, as well as ahead of all other policies. Neither two-lever set
justifies its extra rule complexity as a Pillar 10 solution.

Reject `city-upkeep` from a default recommendation at this setting. Its faction ratio
worsens to **8.222×**; Germany wins 148 of 570 seats while Sparta wins 18. It increases
captures from **207 to 340**, yet does not create a Wide counter. Wide's mean peak cities
also rises from **4.37 to 4.56**. The other candidates improve the faction ratio on point
estimates, but every ratio remains well above **1.5×**. These are descriptive screening
results, not confidence-qualified proof of faction improvement or regression.

Wide remains competitive as an assigned policy: every treatment gives it **19.0–21.8%**
seat wins versus **22.7%** in control (equal-share reference: 12.5%), about **4.0–4.6** peak
cities, and over **99.7%** survival. It is not suppressed into an uncompetitive policy.
However, **0 of 784 assigned Wide seats per condition** meet classifier v1's four-cities-by-
turn-60 criterion, including in control. The baseline opening evidence contains only one
to three founded cities by that horizon. The strongest claim supported here is about the
six forced policies, not optimal human strategies or a validated observed Wide archetype.
Do not silently change the classifier to make the experiment succeed.

The recommendation is to keep all levers opt-in. Higher coefficients, other thresholds,
other pairings, Standard speed, optimized policies and held-out seeds are untested. A stronger
Settler ramp merits further study if the lead wants to continue; it does not meet the stated
acceptance criteria on this evidence. No design decision was recorded on the lead's behalf.

### Paired change in Wide / Economic placement

These are treatment minus control, in percentage points. Every row uses the same 20 maps /
40 games in this matchup; none of those maps was censored. Other paired cells follow in the
appendix and the external JSON.

| Condition | Change | Paired descriptive 95% interval |
| --- | ---: | ---: |
| settler-escalation | -5.0 | [-9.7, -0.5] |
| city-research | -1.1 | [-4.8, +2.2] |
| city-upkeep | -3.5 | [-7.1, -0.2] |
| expansion-coalitions | -0.5 | [-3.7, +3.0] |
| settler-research | -6.8 | [-11.0, -2.8] |
| settler-coalitions | -3.4 | [-7.1, -0.2] |

### Coalition exposure

The city-based trigger was exercised. With the isolated coalition flag, it selects a target
different from the VP-only rule on **10,427 resolved turns**. More targeting and fighting
still does not translate into a counter structure.

| Condition | Eligible target turns | Target at war | Target in combat | All city captures |
| --- | ---: | ---: | ---: | ---: |
| control | 14,614 | 5,167 | 2,053 | 207 |
| expansion-coalitions | 24,545 | 8,151 | 3,388 | 203 |
| settler-coalitions | 23,576 | 8,199 | 3,438 | 185 |

These counters select the target in the resolved state. Wars may predate its selection,
and a lost city can remove a former target from that state. They measure exposure, not
proof that an explicit coalition caused a battle or a capture. The table's capture count
includes all captures, independently of target selection.

## Scope and frozen rules

This is a Pillar 10 experiment, with Pillar 8 measured alongside it. Defaults are unchanged.
The lead retains the rule decision and its design-log entry. All eight experimental flags
are off unless explicitly enabled; the four earlier counter levers remain available.

The study integrates `astra/strategy-counters` (`2596764`) with master through
`cd86292e163182f7987ca888dbbf46f7787d653c`, including diplomacy, coalitions, Proxy v3,
the current AI fixes and Settler detour safety. Merge commits are `4ae8407` and `af55001`.
The replay identity fix and the branch's forced-Rush opening are retained. The single
diplomacy text conflict retains both coalition disengagement and Rush's longer campaign
window. Two newer test-only runner literals also needed default experiment fields.

Every game starts at Peace under current diplomacy, with eight separate teams and Normal
scripted AI. Proxy v3 is preserved by the merge but is not a contestant in this forced-policy
round robin. The six existing policies are Rush, Wide, Tall, Science, Turtle and Economic.
Diplomatic remains an uncalibrated, inactive classifier hook and has no forced policy;
these results cannot establish counters for a Diplomatic strategy.

## Interventions

| Flag | Exact isolated intervention |
| --- | --- |
| `settler-escalation` | Master already charges 75 base Production plus 20 for each previously trained Settler. Increase the step to 60: with the normal base, 75, 135, 195, 255, … Production. Faction base cost applies first, escalation next, and speed last. The starting Settler is uncounted; trained and purchased Settlers share the existing counter. |
| `city-research` | For `c` currently owned cities, add `ceil(diffused price × 0.05 × max(c−2, 0))` Science before speed scaling. Captured and resisting cities count. Stored Science is retained; losing a city reduces subsequent quotes. |
| `city-upkeep` | Each city costs `max(c−2, 0)` Gold per turn: total `c × max(c−2, 0)`. At 1/2/3/4/5/6 cities, administration costs 0/0/3/8/15/24 Gold per turn. It is separate from military upkeep and is not discounted by Strategic Logistics. Existing bankruptcy rules apply. |
| `expansion-coalitions` | Add `floor(10 × max(c−2m, 0) / m)` to integer team VP progress, where `m` is permanent-team membership including eliminated members. Apply the existing coalition thresholds to that pressure: strictly ahead and either ≥80, or ≥60 with a ≥15 lead. Equal pressure has no target. Coalition exit checks use the same pressure. |

These are familiar mechanical forms, not reproductions of Civilization's exact numerical
balance. The coalition lever uses public city counts, not hidden territory or city locations.
It changes the trigger and its exit conditions, not military strength, war legality or a
policy label. Ordinary VP, scoring, faction traits and normal pact cutoffs are unchanged.
Costs, purchase quotes, research previews, net income and bankruptcy forecasts share the
engine calculation. New false flags are omitted from old experimental encodings.

## Method and interpretation

Each condition schedules 600 Quick games on the default 48×32 map: 15 unordered policy pairs, 40 games / 20 mirrored
maps per pair, seeds 982001–982300. A lobby has four empires of each policy; its second
orientation swaps every seat's policy while retaining the seed, faction and starting position.
The complete seed schedule is reused in every condition. These are development seeds,
not held-out confirmation. No artificial victory turn limit is imposed; the safety cap is
600 Quick turns. Capped games are censored, not losses or draws.

The analyzer independently reconstructs each raw matrix and reconciles it with the Rust
summary. It excludes an entire mirrored map from every condition if either orientation
is unfinished in any condition, and refuses fewer than 16 complete maps / 32 games in
any cell. Its JSON includes worst-case score bounds for the excluded maps. Complete-case
results do not assume that censoring is random.

Cells are relative-placement scores: ahead plus half ties, averaged equally over games.
The winning team ranks first, then survivors by VP, then eliminated empires by elimination
turn. They are not duel win rates. A clear counter requires a descriptive 95% bootstrap
interval excluding 50%, resampling whole mirrored maps (10,000 replicates, RNG seed 972001).
Treatment-minus-control intervals are paired on those maps. These exploratory intervals
are not corrected for multiple comparisons. A policy can be undominated on point estimates
without having a demonstrated counter; the report checks both conditions separately.

Winner entropy is normalized Shannon entropy over six assigned policies or seven observed
labels including Generalist. Zero-win labels stay in the denominator. Faction rates are
winning seats / eligible seats, with equal faction exposure; best/worst is their rate ratio.
The ≤1.5× target is reported separately from deterioration versus the control. This forced
Quick tournament is not the roughly 2,000-game-per-speed plus held-out ordinary-AI campaign
required to establish production Pillar 8 balance.

Viability is checked through Wide's placement row, winning seats, survival, final/peak
city counts and observed behavior. Positive wins alone do not prove that a Wide strategy
was executed. Classifier v1 requires four cities founded by Quick turn 60; it is not
recalibrated for these experiments. Policies are frozen rather than retuned for each cost.


### Completion and common sample

| Condition | Scheduled | VP finishes | Safety-cap stops | Common eligible |
| --- | ---: | ---: | ---: | ---: |
| control | 600 | 599 | 1 | 570 |
| settler-escalation | 600 | 598 | 2 | 570 |
| city-research | 600 | 599 | 1 | 570 |
| city-upkeep | 600 | 596 | 4 | 570 |
| expansion-coalitions | 600 | 595 | 5 | 570 |
| settler-research | 600 | 596 | 4 | 570 |
| settler-coalitions | 600 | 594 | 6 | 570 |

All 23 safety-cap stops are retained in raw reports, with their indices also in the analysis
JSON. Shared exclusion removes both orientations of 15 maps from every condition, leaving
3,990 eligible game observations across the seven conditions. No extra blocks were needed.
The excluded pre-freeze 30-game build check and two all-flag replay checks are not study
samples. Complete-case filtering changes the baseline numbers from the intermediate
cost-only comparisons; the tables here all use the final seven-condition cohort.

| Matchup | Eligible games / mirrored maps |
| --- | ---: |
| economic / rush | 38 / 19 |
| economic / science | 34 / 17 |
| economic / tall | 38 / 19 |
| economic / turtle | 38 / 19 |
| economic / wide | 40 / 20 |
| rush / science | 40 / 20 |
| rush / tall | 40 / 20 |
| rush / turtle | 40 / 20 |
| rush / wide | 40 / 20 |
| science / tall | 36 / 18 |
| science / turtle | 34 / 17 |
| science / wide | 38 / 19 |
| tall / turtle | 36 / 18 |
| tall / wide | 38 / 19 |
| turtle / wide | 40 / 20 |

Each faction has 570 eligible seats. Policy exposures vary slightly after shared censoring;
the appendix reports their exact denominators. Entropies describe empirical winner shares,
without adjusting those shares for the remaining differences in policy exposure.

## Artifacts and reproduction

New raw JSON, tapes, executable snapshots, bootstrap output, source patch and check logs
are outside Git in `/tmp/legatum-sim-artifacts/wide-counters-2026-09-28/`. Historical files
already committed on `astra/strategy-counters` are retained by the merge. The new report
does not add a second set of large raw artifacts to Git.

Frozen study executable SHA-256:
`206faeea8f4978ca9af47611c31b2f1de29603d175afe47b5f24a446dd5579dc`.
It was built from the pending implementation on `af55001`; the archived `study-source.patch`
has SHA-256 `56c65fe4583e8426de95feb3f85a9327fec658c1fc9bab0eb50e817fb5a7e317`.
The later runner-literal fix only fills test setup defaults and does not change the study
executable's gameplay. Each condition has an external manifest with its exact command,
executable hash, report hash, runtime and CLI exit code. Run-start Git metadata alone is
not the executable's build identity.

Build this branch and run each condition with the same schedule, using a fresh output
directory for a rerun:

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
RUN=/tmp/legatum-sim-artifacts/wide-counters-rerun
./target/release/legatum-sim --strategy-matchups --games 600 --seed 982001 \
  --speed quick --threads 4 --label control --out-dir "$RUN/control" \
  --diversity-check --diversity-basis assigned --diversity-min-matchup-games 32
# Repeat with separate labels/output directories and one flag at a time:
# --rules-experiment settler-escalation
# --rules-experiment city-research
# --rules-experiment city-upkeep
# --rules-experiment expansion-coalitions
```

Guardrail exit 1 denotes a measured diversity failure, separately from panics or test
failures. The raw reports and manifests distinguish these outcomes.

The two combination flags are:

```sh
# label settler-research:
# --rules-experiment settler-escalation,city-research
# label settler-coalitions:
# --rules-experiment settler-escalation,expansion-coalitions
```

Reconstruct the full paired audit from the original artifacts:

```sh
RUN=/tmp/legatum-sim-artifacts/wide-counters-2026-09-28
python3 tools/analysis/strategy_counters.py \
  control="$RUN/control/2026-09-28-control.json" \
  settler-escalation="$RUN/settler-escalation/2026-09-28-settler-escalation.json" \
  city-research="$RUN/city-research/2026-09-28-city-research.json" \
  city-upkeep="$RUN/city-upkeep/2026-09-28-city-upkeep.json" \
  expansion-coalitions="$RUN/expansion-coalitions/2026-09-28-expansion-coalitions.json" \
  settler-research="$RUN/settler-research/2026-09-28-settler-research.json" \
  settler-coalitions="$RUN/settler-coalitions/2026-09-28-settler-coalitions.json" \
  --replicates 10000 --markdown "$RUN/results-appendix.md" > "$RUN/analysis.json"
```

All seven simulation commands return **1** because their assigned-policy diversity guardrail
fails; all reports contain **zero panics**. The independent analyzer returns **0**, validating
identities, raw matrices and the minimum sample size. Its exit code does not endorse a rule.

| Raw report condition | SHA-256 |
| --- | --- |
| control | `9ca16d66b2852ecae3f5c7c5fdf50a4a390489655af00ef993d04a326d3694ab` |
| settler-escalation | `06279c36f98c73b51bd719b549cbe5b1b279508d3cfef041ec356b71323417d6` |
| city-research | `8e829f0b737aef0512f3fd280b2bf51d65f35cbc3d91b596e43d09c6561a6b2b` |
| city-upkeep | `ff8309bb841a693460d97f44890608951544fce5d6febbd09e4b053c0316cbc2` |
| expansion-coalitions | `d7fe591b5d7c069c25295ed43fbb95c79b1598585c4abf3be1dce070c8a43bb5` |
| settler-research | `9d1f10f0c93b30ca8cfc07134cf99c65e40c787dc32b60f0827a8eed360de103` |
| settler-coalitions | `01a216f97e7eef883c5529eb0d27958b2e20f2b12804344db94eff1644c3a1f7` |

Paired analysis JSON SHA-256: `64a50b86464bee545e6e7a2d7be77b551f76a93fa2c0d20fa42db340edaa2677`.

## Validation

| Check | Exact result |
| --- | --- |
| `cargo fmt --all` | Exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 after fixing the two merged test literals. |
| `cargo test --workspace --release --no-fail-fast` | **Exit 0; 915 passed, 0 failed, 4 ignored.** |
| `cargo test --workspace` | **Exit 0; 915 passed, 0 failed, 4 ignored.** |
| `cargo test -p legatum-sim --test determinism` | **Exit 0; 12 passed, 0 failed.** Existing fixtures are unchanged. |
| Experimental full-game determinism and tape readback | Exit 0; 2 games / 280 turns with all four new flags, initial/every-turn/save-load hashes match. |
| Analyzer regression on prior study control | Exit 0; original matrix, entropies, faction counts, game lengths, Rush metrics and capture totals unchanged. |
| `git diff --check`; conflict-marker scan | Clean before commit. |

The four ignored workspace checks were already opt-in: AI midgame speed, the paired AI
release evaluation, the many-games summary, and the explicit cross-build performance
corpus. No additional test was ignored. Build jobs were limited to four and test/Rayon
threads to two for the workspace test runs. Separate passing contract tests cover sequential
Settler purchase quotes, ownership/diffusion/speed-dependent research prices and actual
completion, city upkeep payment and bankruptcy forecasts, coalition team normalization,
fogged-view agreement and save/load. The existing replay-export test verifies that the two
mirrored orientations retain distinct replay identities.

Default fixture replay passed without regeneration. `docs/learnings.md` and `CLAUDE.md`
were not edited. This branch is for lead review; it was not merged into master.

## Generated paired results

Cells are row half-tie finish scores (%), not duel win rates. Cells have at least 32 games / 16 complete mirrored maps. Clear counters require a descriptive map-bootstrap 95% interval excluding 50%; no multiplicity correction. The JSON contains all intervals and paired deltas.

### control

Scheduled 600; eligible 570; censored 1; common excluded maps 15. Assigned winner entropy **0.9040**; observed **0.7251**. Length mean **142.6**, median **137.0**, range 95–340 turns. Captures: 207.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 12.0 | 19.7 | 26.4 | 18.7 | 18.6 |
| wide | 88.0 | — | 68.0 | 70.1 | 70.7 | 60.8 |
| tall | 80.3 | 32.0 | — | 54.0 | 49.2 | 37.7 |
| science | 73.6 | 29.9 | 46.0 | — | 48.3 | 39.2 |
| turtle | 81.3 | 29.3 | 50.8 | 51.7 | — | 34.5 |
| economic | 81.4 | 39.2 | 62.3 | 60.8 | 65.5 | — |

Point-estimate dominant policies: wide. Every policy has a clear counter: no.

| Assigned policy | Wins / seats | Seat win rate | Mean final / peak cities | Observed matching policy |
| --- | ---: | ---: | ---: | ---: |
| rush | 12 / 792 | 1.52% | 2.49 / 2.55 | 24.6% |
| wide | 178 / 784 | 22.70% | 4.35 / 4.37 | 0.0% |
| tall | 93 / 752 | 12.37% | 2.60 / 2.62 | 70.1% |
| science | 79 / 728 | 10.85% | 2.80 / 2.84 | 50.8% |
| turtle | 74 / 752 | 9.84% | 2.85 / 2.87 | 10.0% |
| economic | 134 / 752 | 17.82% | 3.27 / 3.29 | 35.0% |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 57 / 570 | 10.00% |
| Germany | 139 / 570 | 24.39% |
| Mongols | 33 / 570 | 5.79% |
| Sparta | 22 / 570 | 3.86% |
| Byzantium | 49 / 570 | 8.60% |
| Athens | 89 / 570 | 15.61% |
| Switzerland | 78 / 570 | 13.68% |
| Celts | 103 / 570 | 18.07% |

Faction max/min: **6.318×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, wide |
| turtle | economic, wide |
| economic | wide |

### settler-escalation

Scheduled 600; eligible 570; censored 2; common excluded maps 15. Assigned winner entropy **0.9239**; observed **0.7116**. Length mean **147.1**, median **142.0**, range 93–283 turns. Captures: 190.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 14.7 | 22.7 | 24.4 | 22.1 | 19.7 |
| wide | 85.3 | — | 64.4 | 64.9 | 65.2 | 55.8 |
| tall | 77.3 | 35.6 | — | 55.0 | 49.2 | 42.1 |
| science | 75.6 | 35.1 | 45.0 | — | 46.0 | 40.0 |
| turtle | 77.9 | 34.8 | 50.8 | 54.0 | — | 34.5 |
| economic | 80.3 | 44.2 | 57.9 | 60.0 | 65.5 | — |

Point-estimate dominant policies: wide. Every policy has a clear counter: no.

| Assigned policy | Wins / seats | Seat win rate | Mean final / peak cities | Observed matching policy |
| --- | ---: | ---: | ---: | ---: |
| rush | 16 / 792 | 2.02% | 2.45 / 2.51 | 24.1% |
| wide | 155 / 784 | 19.77% | 3.96 / 3.98 | 0.0% |
| tall | 96 / 752 | 12.77% | 2.60 / 2.62 | 74.5% |
| science | 82 / 728 | 11.26% | 2.77 / 2.81 | 51.4% |
| turtle | 81 / 752 | 10.77% | 2.85 / 2.87 | 9.3% |
| economic | 140 / 752 | 18.62% | 3.19 / 3.21 | 29.9% |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 50 / 570 | 8.77% |
| Germany | 128 / 570 | 22.46% |
| Mongols | 31 / 570 | 5.44% |
| Sparta | 28 / 570 | 4.91% |
| Byzantium | 57 / 570 | 10.00% |
| Athens | 84 / 570 | 14.74% |
| Switzerland | 90 / 570 | 15.79% |
| Celts | 102 / 570 | 17.89% |

Faction max/min: **4.571×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -1.2 | [-5.2, +2.9] |
| economic / science | -0.7 | [-4.4, +2.9] |
| economic / tall | -4.4 | [-9.3, +1.4] |
| economic / turtle | +0.0 | [-4.4, +4.8] |
| economic / wide | +5.0 | [+0.5, +9.7] |
| rush / science | -2.0 | [-6.1, +2.8] |
| rush / tall | +3.0 | [-0.9, +7.7] |
| rush / turtle | +3.4 | [-0.4, +7.5] |
| rush / wide | +2.7 | [-0.2, +5.4] |
| science / tall | -1.0 | [-3.7, +1.6] |
| science / turtle | -2.2 | [-6.8, +2.5] |
| science / wide | +5.2 | [+1.3, +9.0] |
| tall / turtle | +0.0 | [-5.9, +5.6] |
| tall / wide | +3.6 | [-2.7, +9.8] |
| turtle / wide | +5.5 | [+0.3, +10.3] |

### city-research

Scheduled 600; eligible 570; censored 1; common excluded maps 15. Assigned winner entropy **0.9153**; observed **0.7341**. Length mean **145.6**, median **139.0**, range 95–487 turns. Captures: 216.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 10.6 | 22.7 | 28.4 | 19.5 | 18.8 |
| wide | 89.4 | — | 67.9 | 69.6 | 72.0 | 59.7 |
| tall | 77.3 | 32.1 | — | 52.7 | 51.0 | 38.9 |
| science | 71.6 | 30.4 | 47.3 | — | 45.0 | 38.8 |
| turtle | 80.5 | 28.0 | 49.0 | 55.0 | — | 36.6 |
| economic | 81.2 | 40.3 | 61.1 | 61.2 | 63.4 | — |

Point-estimate dominant policies: wide. Every policy has a clear counter: no.

| Assigned policy | Wins / seats | Seat win rate | Mean final / peak cities | Observed matching policy |
| --- | ---: | ---: | ---: | ---: |
| rush | 16 / 792 | 2.02% | 2.53 / 2.60 | 24.6% |
| wide | 171 / 784 | 21.81% | 4.36 / 4.38 | 0.0% |
| tall | 96 / 752 | 12.77% | 2.62 / 2.64 | 70.3% |
| science | 77 / 728 | 10.58% | 2.82 / 2.85 | 50.0% |
| turtle | 75 / 752 | 9.97% | 2.88 / 2.89 | 10.1% |
| economic | 135 / 752 | 17.95% | 3.31 / 3.33 | 36.3% |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 56 / 570 | 9.82% |
| Germany | 130 / 570 | 22.81% |
| Mongols | 28 / 570 | 4.91% |
| Sparta | 24 / 570 | 4.21% |
| Byzantium | 55 / 570 | 9.65% |
| Athens | 93 / 570 | 16.32% |
| Switzerland | 85 / 570 | 14.91% |
| Celts | 99 / 570 | 17.37% |

Faction max/min: **5.417×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, turtle, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -0.2 | [-5.0, +4.0] |
| economic / science | +0.5 | [-2.8, +4.0] |
| economic / tall | -1.2 | [-3.0, +0.8] |
| economic / turtle | -2.1 | [-6.0, +1.3] |
| economic / wide | +1.1 | [-2.2, +4.8] |
| rush / science | +2.0 | [-1.6, +6.2] |
| rush / tall | +3.0 | [-0.9, +7.4] |
| rush / turtle | +0.9 | [-1.8, +3.4] |
| rush / wide | -1.3 | [-3.0, +0.2] |
| science / tall | +1.3 | [-1.2, +3.9] |
| science / turtle | -3.2 | [-6.8, +0.5] |
| science / wide | +0.5 | [-1.7, +2.6] |
| tall / turtle | +1.8 | [-0.1, +3.8] |
| tall / wide | +0.1 | [-3.5, +3.6] |
| turtle / wide | -1.2 | [-4.6, +2.2] |

### city-upkeep

Scheduled 600; eligible 570; censored 4; common excluded maps 15. Assigned winner entropy **0.9275**; observed **0.7536**. Length mean **150.6**, median **144.0**, range 103–490 turns. Captures: 340.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 18.8 | 18.4 | 31.5 | 20.9 | 16.1 |
| wide | 81.2 | — | 67.6 | 73.9 | 67.2 | 57.3 |
| tall | 81.6 | 32.4 | — | 53.6 | 50.7 | 40.8 |
| science | 68.5 | 26.1 | 46.4 | — | 50.6 | 40.3 |
| turtle | 79.1 | 32.8 | 49.3 | 49.4 | — | 39.1 |
| economic | 83.9 | 42.7 | 59.2 | 59.7 | 60.9 | — |

Point-estimate dominant policies: wide. Every policy has a clear counter: no.

| Assigned policy | Wins / seats | Seat win rate | Mean final / peak cities | Observed matching policy |
| --- | ---: | ---: | ---: | ---: |
| rush | 17 / 792 | 2.15% | 2.64 / 2.72 | 24.5% |
| wide | 149 / 784 | 19.01% | 4.48 / 4.56 | 0.0% |
| tall | 99 / 752 | 13.16% | 2.61 / 2.63 | 71.7% |
| science | 79 / 728 | 10.85% | 2.84 / 2.88 | 52.3% |
| turtle | 84 / 752 | 11.17% | 2.88 / 2.90 | 8.8% |
| economic | 142 / 752 | 18.88% | 3.32 / 3.34 | 40.6% |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 37 / 570 | 6.49% |
| Germany | 148 / 570 | 25.96% |
| Mongols | 33 / 570 | 5.79% |
| Sparta | 18 / 570 | 3.16% |
| Byzantium | 62 / 570 | 10.88% |
| Athens | 69 / 570 | 12.11% |
| Switzerland | 90 / 570 | 15.79% |
| Celts | 113 / 570 | 19.82% |

Faction max/min: **8.222×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | +2.5 | [-0.3, +5.8] |
| economic / science | -1.1 | [-3.5, +1.7] |
| economic / tall | -3.0 | [-6.3, +0.3] |
| economic / turtle | -4.7 | [-8.1, -1.2] |
| economic / wide | +3.5 | [+0.2, +7.1] |
| rush / science | +5.1 | [+0.3, +10.0] |
| rush / tall | -1.2 | [-5.5, +2.6] |
| rush / turtle | +2.3 | [-0.6, +5.6] |
| rush / wide | +6.9 | [+3.3, +10.4] |
| science / tall | +0.4 | [-3.0, +3.9] |
| science / turtle | +2.3 | [-3.5, +8.4] |
| science / wide | -3.9 | [-8.1, +0.4] |
| tall / turtle | +1.5 | [-1.6, +4.9] |
| tall / wide | +0.4 | [-4.1, +4.8] |
| turtle / wide | +3.5 | [-1.0, +8.0] |

### expansion-coalitions

Scheduled 600; eligible 570; censored 5; common excluded maps 15. Assigned winner entropy **0.9083**; observed **0.7188**. Length mean **144.5**, median **138.0**, range 96–333 turns. Captures: 203.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 11.6 | 22.1 | 30.5 | 16.1 | 16.5 |
| wide | 88.4 | — | 69.0 | 69.9 | 72.5 | 60.3 |
| tall | 77.9 | 31.0 | — | 51.6 | 49.0 | 37.9 |
| science | 69.5 | 30.1 | 48.4 | — | 47.8 | 37.9 |
| turtle | 83.9 | 27.5 | 51.0 | 52.2 | — | 34.0 |
| economic | 83.5 | 39.7 | 62.1 | 62.1 | 66.0 | — |

Point-estimate dominant policies: wide. Every policy has a clear counter: no.

| Assigned policy | Wins / seats | Seat win rate | Mean final / peak cities | Observed matching policy |
| --- | ---: | ---: | ---: | ---: |
| rush | 13 / 792 | 1.64% | 2.53 / 2.59 | 24.6% |
| wide | 171 / 784 | 21.81% | 4.47 / 4.49 | 0.0% |
| tall | 92 / 752 | 12.23% | 2.63 / 2.64 | 68.5% |
| science | 79 / 728 | 10.85% | 2.84 / 2.88 | 50.0% |
| turtle | 75 / 752 | 9.97% | 2.88 / 2.90 | 10.0% |
| economic | 140 / 752 | 18.62% | 3.34 / 3.36 | 37.1% |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 59 / 570 | 10.35% |
| Germany | 136 / 570 | 23.86% |
| Mongols | 35 / 570 | 6.14% |
| Sparta | 26 / 570 | 4.56% |
| Byzantium | 48 / 570 | 8.42% |
| Athens | 82 / 570 | 14.39% |
| Switzerland | 79 / 570 | 13.86% |
| Celts | 105 / 570 | 18.42% |

Faction max/min: **5.231×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | +2.1 | [-0.6, +4.4] |
| economic / science | +1.4 | [-0.7, +3.2] |
| economic / tall | -0.2 | [-3.5, +3.5] |
| economic / turtle | +0.5 | [-2.4, +3.6] |
| economic / wide | +0.5 | [-3.0, +3.7] |
| rush / science | +4.1 | [-1.4, +9.1] |
| rush / tall | +2.4 | [-1.6, +6.2] |
| rush / turtle | -2.6 | [-6.6, +0.9] |
| rush / wide | -0.3 | [-3.0, +2.7] |
| science / tall | +2.3 | [-0.8, +5.7] |
| science / turtle | -0.5 | [-3.0, +1.7] |
| science / wide | +0.2 | [-2.8, +3.0] |
| tall / turtle | -0.2 | [-2.4, +1.5] |
| tall / wide | -1.0 | [-3.7, +2.0] |
| turtle / wide | -1.8 | [-5.4, +1.6] |

### settler-research

Scheduled 600; eligible 570; censored 4; common excluded maps 15. Assigned winner entropy **0.9223**; observed **0.7227**. Length mean **148.6**, median **143.0**, range 96–466 turns. Captures: 181.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 10.0 | 20.5 | 26.6 | 20.8 | 18.3 |
| wide | 90.0 | — | 63.4 | 63.2 | 70.8 | 54.0 |
| tall | 79.5 | 36.6 | — | 52.6 | 52.6 | 43.9 |
| science | 73.4 | 36.8 | 47.4 | — | 50.8 | 39.3 |
| turtle | 79.2 | 29.2 | 47.4 | 49.2 | — | 36.5 |
| economic | 81.7 | 46.0 | 56.1 | 60.7 | 63.5 | — |

Point-estimate dominant policies: wide. Every policy has a clear counter: no.

| Assigned policy | Wins / seats | Seat win rate | Mean final / peak cities | Observed matching policy |
| --- | ---: | ---: | ---: | ---: |
| rush | 14 / 792 | 1.77% | 2.42 / 2.48 | 24.1% |
| wide | 155 / 784 | 19.77% | 4.00 / 4.02 | 0.0% |
| tall | 98 / 752 | 13.03% | 2.59 / 2.61 | 74.3% |
| science | 90 / 728 | 12.36% | 2.78 / 2.81 | 51.0% |
| turtle | 78 / 752 | 10.37% | 2.84 / 2.85 | 9.4% |
| economic | 135 / 752 | 17.95% | 3.23 / 3.25 | 30.9% |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 43 / 570 | 7.54% |
| Germany | 129 / 570 | 22.63% |
| Mongols | 34 / 570 | 5.96% |
| Sparta | 27 / 570 | 4.74% |
| Byzantium | 56 / 570 | 9.82% |
| Athens | 85 / 570 | 14.91% |
| Switzerland | 90 / 570 | 15.79% |
| Celts | 106 / 570 | 18.60% |

Faction max/min: **4.778×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, wide |
| turtle | economic, wide |
| economic | None demonstrated |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | +0.2 | [-3.4, +4.0] |
| economic / science | -0.1 | [-4.7, +4.4] |
| economic / tall | -6.2 | [-10.8, -0.7] |
| economic / turtle | -2.1 | [-7.2, +3.0] |
| economic / wide | +6.8 | [+2.8, +11.0] |
| rush / science | +0.2 | [-4.6, +5.1] |
| rush / tall | +0.8 | [-4.1, +5.7] |
| rush / turtle | +2.1 | [-2.0, +5.9] |
| rush / wide | -2.0 | [-5.3, +1.2] |
| science / tall | +1.4 | [-2.0, +4.9] |
| science / turtle | +2.6 | [-1.5, +7.0] |
| science / wide | +6.8 | [+3.0, +10.4] |
| tall / turtle | +3.4 | [-0.7, +7.6] |
| tall / wide | +4.6 | [-1.5, +10.6] |
| turtle / wide | -0.1 | [-4.5, +4.1] |

### settler-coalitions

Scheduled 600; eligible 570; censored 6; common excluded maps 15. Assigned winner entropy **0.9256**; observed **0.7291**. Length mean **148.4**, median **143.0**, range 97–595 turns. Captures: 185.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 11.8 | 21.0 | 25.9 | 21.5 | 20.2 |
| wide | 88.2 | — | 63.5 | 61.5 | 66.6 | 57.3 |
| tall | 79.0 | 36.5 | — | 52.9 | 48.8 | 44.3 |
| science | 74.1 | 38.5 | 47.1 | — | 48.1 | 39.8 |
| turtle | 78.5 | 33.4 | 51.2 | 51.9 | — | 36.7 |
| economic | 79.8 | 42.7 | 55.7 | 60.2 | 63.3 | — |

Point-estimate dominant policies: wide. Every policy has a clear counter: no.

| Assigned policy | Wins / seats | Seat win rate | Mean final / peak cities | Observed matching policy |
| --- | ---: | ---: | ---: | ---: |
| rush | 16 / 792 | 2.02% | 2.48 / 2.53 | 24.1% |
| wide | 154 / 784 | 19.64% | 4.02 / 4.04 | 0.0% |
| tall | 97 / 752 | 12.90% | 2.58 / 2.60 | 71.3% |
| science | 88 / 728 | 12.09% | 2.82 / 2.85 | 51.9% |
| turtle | 78 / 752 | 10.37% | 2.88 / 2.90 | 8.2% |
| economic | 137 / 752 | 18.22% | 3.21 / 3.23 | 29.5% |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 48 / 570 | 8.42% |
| Germany | 130 / 570 | 22.81% |
| Mongols | 32 / 570 | 5.61% |
| Sparta | 35 / 570 | 6.14% |
| Byzantium | 47 / 570 | 8.25% |
| Athens | 87 / 570 | 15.26% |
| Switzerland | 88 / 570 | 15.44% |
| Celts | 103 / 570 | 18.07% |

Faction max/min: **4.062×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -1.6 | [-7.0, +3.5] |
| economic / science | -0.6 | [-4.6, +3.4] |
| economic / tall | -6.6 | [-10.5, -2.7] |
| economic / turtle | -2.2 | [-7.0, +3.2] |
| economic / wide | +3.4 | [+0.2, +7.1] |
| rush / science | -0.5 | [-5.3, +4.4] |
| rush / tall | +1.3 | [-4.4, +7.0] |
| rush / turtle | +2.8 | [-1.6, +7.3] |
| rush / wide | -0.2 | [-3.7, +3.7] |
| science / tall | +1.1 | [-2.3, +4.5] |
| science / turtle | -0.2 | [-5.1, +4.9] |
| science / wide | +8.6 | [+4.6, +13.0] |
| tall / turtle | -0.4 | [-4.3, +3.6] |
| tall / wide | +4.5 | [-0.7, +9.5] |
| turtle / wide | +4.1 | [-1.3, +9.5] |

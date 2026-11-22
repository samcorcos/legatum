# Strategy counters — 2026-09-28

Promote no rule lever from this study. None of the four isolated levers or either tested combination produces the requested counter structure. Wide remains ahead of every other forced policy, with no demonstrated counter. The best/worst faction win-rate ratio is also far above 1.5×. Default rules remain unchanged; these are experiments for the lead to review, not recorded design decisions.

This is a negative result for these scripted policies and tested settings. It is not a proof that a competent human Rush cannot work, that these are the optimal policies, or that no other rule combination can produce a cycle.

## Baseline after diplomacy

The checkout matched master **`ad2e9c5`** when work began and contains the diplomacy merge `5ef02f0`. The baseline executable includes only the replay-export fix `8eeff2e`; that fix does not change gameplay. Current Peace/pact rules are active, not the Proxy benchmark’s legacy all-war mode.

The requested initial 480-game rerun changed the diagnosis from the September 27 study: **Wide**, not Economic, is now the dominant candidate. Its half-tie placement scores were 77.1% versus Rush, 70.3% versus Tall, 69.9% versus Science, 67.6% versus Turtle and 56.1% versus Economic. Assigned winner entropy was 0.9445. There were 478 VP victories and two safety-cap stops, with no panics. Sixty additional games increased every scheduled matchup to 36 games.

Diplomatic remains excluded. Classifier v1 has a dormant hook, but the observer still supplies `diplomacy: null` for that classifier, and `Strategy::ALL` still has six policies. Separate pact telemetry does not establish the required allied turns and coalitions led. Activating a new classification or inventing a diplomatic policy would require separate calibration.

## Rush audit before rule comparisons

The original post-diplomacy Rush rarely executed the opening its name implied. An initial revision capped the army, reserved ranged slots, changed research timing, chose nearby attainable cities for declarations, avoided early neighbour pacts and allowed campaigns more time before routine peace. It failed its empirical audit. A second revision added a funded opening shared by production and purchase choices: a second melee unit, Workshop and Market, then ranged support and a bounded field force. Settlers wait until after Quick turn 60; times scale with game speed. Emergency defense and immediate victory actions retain priority. These changes affect only the opt-in forced Rush policy, not ordinary faction AI.

Rangers are available from turn 1. Ballistics unlocks Troopers. The initial assumption that earlier Ballistics supplied ranged units was wrong; the revised production sequence is the relevant change. The final research sequence is Fabrication → Soil Reclamation → Exchange → Fortification → Ballistics → Field Logistics.

The final policy was piloted on 30 excluded games at seeds 971001–971015 before freezing it for the rule trials: 8/40 Rush seats met classifier v1, with 2.4 initiated early battles per Rush seat. The full comparable panel is more informative:

| Policy | Matching Rush label | Mean early initiated battles | Mean early military completed cost | Captures by Rush seats |
| --- | ---: | ---: | ---: | ---: |
| Master policy | 14 / 720 | 0.31 | 126.9 | 269 |
| Rejected first revision (`67d6631`) | 3 / 720 | 0.23 | 127.6 | 171 |
| Funded opening (`91fc827`) | 186 / 720 | 3.30 | 175.6 | 191 |

Rush now attacks substantially earlier and realizes its assigned label more often, but its placement and capture efficiency remain weak. In fact, its placement worsens against all five opponents. This audit establishes better execution of a Rush opening, **not validated optimal play**. That distinction limits any claim about the rules; the revised policy should remain an experimental benchmark until replay analysis establishes stronger play. The independent behavioral labels and raw investment telemetry are retained instead of treating assignment as successful execution.

A subsequent replay audit exactly reproduced control game 0 (seed 972001, Rush/Wide; excluded from additional sample counts). Athens Rush queued its second melee unit on turn 2, Workshop on 14, Market on 34 and first Ranger on 45, then declared war on 100. Japan Rush declared war on 39; it and Germany Wide submitted their first ranged-attack orders on turn 50. Three of the four Rush seats recorded zero initiated attacks by turn 60, and the game ended with zero captures. This one-seed illustration confirms that the opening is being executed but unevenly reaches combat; it does not diagnose a universal rule bottleneck. The [order tape](strategy-counters-2026-09-28-rush-inspection.tape.json.gz) preserves every order and state hash.

## Isolated rules and results

Each rule is opt-in through `--rules-experiment`; absent flags preserve the existing setup serialization, hashes and rules. Rule values travel with setups, saves and order tapes. No rule reads an assigned policy or classifier label. Faction stats, unique units, yields and drawbacks are unchanged, and the city-strength adjustments add to the existing faction-specific bonuses.

| Lever | Exact intervention |
| --- | --- |
| `frontier-cities` | Unwalled population 1/2/3 cities lose 12/8/4 intrinsic defense strength; population ≥4 and walled cities are unaffected. |
| `market-plunder` | Capturing an unwalled city dismantles each Market-family building and awards twice its nominal, speed-scaled production cost in gold. A base Market pays 120 Quick gold. Repeated capture cannot pay again without rebuilding. |
| `wall-masonry` | Walls add eight city defense strength in addition to their existing faction and Citadel contributions. This affects city HP defense, not the garrison unit. |
| `combined` | All three preceding levers together; not a separate rule. |
| `early-siege` | The existing Siege Engine becomes available at Ballistics instead of Siege Engineering. Cost, stats, upkeep and target selection are unchanged. |
| `all-four` | All four levers together, using the same fixed values and Rush policy; this is a condition label, not a separate CLI flag. |

The first three are local, familiar city-defense/infrastructure tradeoffs. The fourth tests earlier access to an existing military role. There is no gold-interest mechanic in the current economy, so no interest cap was implemented.

| Condition | Assigned winner entropy | Faction max/min | Mean / median turns | Captures | All policies have a clear counter? |
| --- | ---: | ---: | ---: | ---: | --- |
| master | 0.9438 | 4.000× | 150.8 / 144 | 296 | No |
| control | 0.9062 | 5.040× | 146.9 / 143 | 199 | No |
| frontier-cities | 0.9066 | 6.632× | 147.4 / 142 | 204 | No |
| market-plunder | 0.9084 | 5.167× | 146.9 / 142 | 201 | No |
| wall-masonry | 0.9158 | 4.960× | 147.3 / 143 | 197 | No |
| combined | 0.9131 | 6.350× | 147.4 / 142 | 206 | No |
| early-siege | 0.9092 | 4.846× | 146.5 / 141 | 191 | No |
| all-four | 0.8984 | 6.150× | 146.9 / 141 | 208 | No |

Every condition still has Wide as its point-estimate dominant policy. Full matrices, faction wins/seats, winner entropies, length ranges, clear-counter lists and paired changes follow below. “Clear” here means a descriptive map-bootstrap 95% interval excludes 50%; these exploratory intervals are not corrected for multiple comparisons.

The small-city penalty, market plunder and both combinations worsen the faction ratio relative to the frozen Rush control on their point estimates. Stronger walls and earlier siege improve that ratio, but remain far from 1.5× and do not solve strategic dominance. Every candidate's ratio also exceeds the original master policy's 4.0×. The policy revisions illustrate why changing the measurement policy must be kept separate from a rule effect: faction results changed even with unchanged rules. These ratios describe the forced-policy tournament, not ordinary faction-AI release balance.

The final four-lever combination also leaves Wide dominant, with a faction ratio of 6.150× and mean length 146.9 turns. It does not rescue the counter structure.

The siege intervention was exercised: across the same 720 Rush seats, completed Siege Engines increased from **427 to 1,098**, and their ranged attacks increased from **691 to 3,001**. Rush captures fell from **191 to 184**. This tests actual additional siege use, although it does not establish that the units were used well. The prescribed Rush opening prioritizes melee and Rangers through turn 60 and does not explicitly plan an early Siege Engine. The isolated siege treatment leaves total initiated attacks through turn 60 unchanged at **2,374**. Its later additional siege use is therefore not a validation of an optimized early-siege Rush.

## Interpretation and decision boundary

- Do not select a default-rule change from these results. No tested set achieves the requested structure, so there is no evidence-backed “smallest successful set” to recommend. Two-lever subsets and other values remain untested; this is not an impossibility claim.
- Weakening intrinsic city defense does not weaken a garrison: the existing combat rules hit the military unit first. Siege also hits a garrison first, and its +12 city bonus applies only when it reaches city HP. This explains why city-defense tuning is an indirect test of early conquest; it does not by itself prove which bottleneck dominates.
- Prioritize replay-level analysis of failed Rush conversions and alternative competent Rush openings before another broad rule search. Earlier attacks with fewer successful captures are insufficient grounds to conclude that human military play is intrinsically weak.
- Preserve faction asymmetry. The existing best/worst ratio is already above target, and preserving stat tables alone does not establish balanced outcomes. No candidate qualifies for a release recommendation here. The roughly 2,000-game Quick/Standard and held-out validation required for a Pillar 8 claim was not performed; a failing Quick counter graph already rejects these candidates.

## Pairing, censoring and uncertainty

Each condition schedules **540 Quick games**, 270 maps, 15 unordered pairs and 36 games / 18 mirrored maps per pair. Each lobby has four empires of each assigned policy, with eight separate teams. The second orientation swaps every policy while retaining seed, faction and starting position. Seeds are **972001–972270**. The same panel is reused for every treatment; this is development evidence, not a held-out confirmation.

The first baseline and failed policy trial were recorded as 480 games at seed 972001 plus 60 at seed 972241. Both shard boundaries are complete 30-game tournament blocks, so this is the same schedule as a single 540-game run. The analyzer retains shard metadata and assigns study indices in concatenation order.

No artificial victory turn limit is imposed. The safety cap is 600 Quick turns. Unfinished games are not losses or draws. The analyzer removes an entire mirrored map from **all** conditions if either orientation is censored in any condition; it refuses fewer than 16 complete maps per matchup. The JSON also bounds placement scores by assigning every excluded map wholly either way, rather than assuming censoring is random.

All nine conditions had the same two safety-cap stops (study game indices 171 and 411, seeds 972086 and 972206), with zero panics. Removing both orientations of those maps leaves **536 eligible games per condition**: **32 Tall/Turtle games** and **36 games for each other matchup**, or at least 16 complete mirrored maps in every cell. Each faction has 536 seats in each condition. The nine scheduled conditions total **4,860 games**, excluding the 30-game pilot and later reproduction checks.

Placements compare winning teams first, then survivors by VP, then eliminated empires by elimination turn, preserving ties. Each game contributes equal weight over its cross-policy seat pairs. Scores are ahead + half ties, not duel win rates. Bootstrap resampling uses whole mirrored maps (10,000 replicates, RNG seed 972001), including paired treatment-minus-control differences. Winner entropy uses fixed vocabularies: six assigned policies, seven observed labels including Generalist. Faction rates are winning seats / seats; all factions have equal exposure within the retained paired cohort.

## Reproduction and artifacts

The replay fix is `8eeff2e`; the first policy trial is `67d6631`, with the final funded opening in `91fc827`. The first three levers and runner integration are in `91d412b`. Commit `f154d43` adds early siege; when false that field is omitted so earlier three-lever tape encoding is preserved. No default-rule tuning was committed.

The raw `source` field records the worktree at **run start**, not the executable build revision. Several executables were built from pending edits and then frozen before those edits were committed; the baseline extra block also used the older frozen binary while the worktree had advanced. Consequently the raw dirty revision field alone does not identify the gameplay code. The implementation commits above identify the completed changes. These are SHA-256 hashes of the frozen executables used:

| Executable / conditions | SHA-256 |
| --- | --- |
| Master baseline, including its extra block | `6813de67c96ba806af5e803ab34ee57e001dd38f5ccdddeef92dd8485cf061f7` |
| Rejected policy revision, including its extra block | `cc96820fba9e5694d9927b5c529c8cd65592b1775712a05e2fffe14259ac896d` |
| Funded Rush pilot, control, first three isolated levers and combination | `2be3ba42ab2f2ba6acf429094f710d173d2de1c0a9316f8de88764bdacb78525` |
| Early siege, all-four combination and final reproduction checks | `86bffd86b9e8f6f2987d9c07678a4c662b02f785388d78ef74398d822e18d4f6` |

As a cross-check, the final executable reproduced all 30 first-block control records and all 30 first-block combined-rule records exactly except elapsed wall time. All **360 non-Rush records** also match the master baseline, excluding elapsed wall time and shard-local indices. These checks support policy isolation and preservation of the earlier experiments after adding the fourth flag. They are not a claim that the run-start metadata embeds build provenance.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo run --release -p legatum-sim -- \
  --strategy-matchups --games 540 --seed 972001 --speed quick --threads 6 \
  --label counters-control --diversity-check --diversity-basis assigned
# Repeat the identical command/seed schedule with a distinct label and one of:
# --rules-experiment frontier-cities
# --rules-experiment market-plunder
# --rules-experiment wall-masonry
# --rules-experiment frontier-cities,market-plunder,wall-masonry
# --rules-experiment early-siege
# --rules-experiment frontier-cities,market-plunder,wall-masonry,early-siege
```

Guardrail exit 1 is expected for these diversity failures; it is separate from panic counts in the reports. Raw reports contain the exact command, setup, source revision/dirty state, telemetry and Rust summaries. Binary snapshots were kept fixed during each batch. The analyzer independently reconstructs each raw shard’s matrix and verifies it against Rust before filtering the common cohort.

- [Paired audit JSON](strategy-counters-2026-09-28-analysis.json).
- [Master](strategy-counters-2026-09-28-master.json.gz) and [additional baseline block](strategy-counters-2026-09-28-master-extra.json.gz).
- [Rejected policy trial](strategy-counters-2026-09-28-policy-v2.json.gz) and [additional block](strategy-counters-2026-09-28-policy-v2-extra.json.gz).
- [Excluded Rush pilot](strategy-counters-2026-09-28-rush-pilot.json.gz).
- [control raw report](strategy-counters-2026-09-28-control.json.gz).
- [frontier-cities raw report](strategy-counters-2026-09-28-frontier-cities.json.gz).
- [market-plunder raw report](strategy-counters-2026-09-28-market-plunder.json.gz).
- [wall-masonry raw report](strategy-counters-2026-09-28-wall-masonry.json.gz).
- [combined raw report](strategy-counters-2026-09-28-combined.json.gz).
- [early-siege raw report](strategy-counters-2026-09-28-early-siege.json.gz).
- [all-four raw report](strategy-counters-2026-09-28-all-four.json.gz).

The following reproduces the complete paired audit from the committed artifacts. Comma-separated files combine shards; the first label is always the paired-difference control.

```sh
python3 tools/analysis/strategy_counters.py \
  control=docs/sim-results/strategy-counters-2026-09-28-control.json.gz \
  master=docs/sim-results/strategy-counters-2026-09-28-master.json.gz,docs/sim-results/strategy-counters-2026-09-28-master-extra.json.gz \
  policy-v2=docs/sim-results/strategy-counters-2026-09-28-policy-v2.json.gz,docs/sim-results/strategy-counters-2026-09-28-policy-v2-extra.json.gz \
  frontier-cities=docs/sim-results/strategy-counters-2026-09-28-frontier-cities.json.gz \
  market-plunder=docs/sim-results/strategy-counters-2026-09-28-market-plunder.json.gz \
  wall-masonry=docs/sim-results/strategy-counters-2026-09-28-wall-masonry.json.gz \
  combined=docs/sim-results/strategy-counters-2026-09-28-combined.json.gz \
  early-siege=docs/sim-results/strategy-counters-2026-09-28-early-siege.json.gz \
  all-four=docs/sim-results/strategy-counters-2026-09-28-all-four.json.gz \
  --replicates 10000 --markdown /tmp/strategy-counters-tables.md \
  > /tmp/strategy-counters-analysis.json
```

## Replay correction and checks

Replay filenames and deduplication now include the native game index. Mirrored games sharing a seed survive direct export and tape-directory import; importing one tape preserves its original index. True identity collisions are rejected before output mutation. CLI regression tests cover all of those paths.

| Check | Result |
| --- | --- |
| `cargo fmt --all` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace --release` | PASS — 837 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | PASS — 11 passed, 0 failed, 0 ignored |
| `cargo test --workspace` | PASS — 837 passed, 0 failed, 4 ignored |

Both workspace runs include the mirrored replay export regression and the opt-in rule tests. The four existing ignored tests are the midgame speed benchmark, paired AI-v2 evaluation, many-games summary, and explicit cross-build observation corpus check. They were not silently counted as passes.

Additional verification: the independent Python audit reconciled all nine conditions with the Rust matrices, verified source checksums and the minimum paired sample sizes, and checked report links and equal faction denominators. The final executable also reproduced 60 control/combined records and one full inspected order tape as described above.

The canonical design document was inaccessible through the available tools. This study follows the user’s brief and checked-in pillars and rules; the lead retains the design decision. `CLAUDE.md` and `docs/learnings.md` were not edited.

## Generated paired results

Cells are row half-tie finish scores (%), not duel win rates. Cells have at least 32 games / 16 complete mirrored maps. Clear counters require a descriptive map-bootstrap 95% interval excluding 50%; no multiplicity correction. The JSON contains all intervals and paired deltas.

### control

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.9062**; observed **0.7643**. Length mean **146.9**, median **143.0**, range 93–475 turns. Captures: 199.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 12.2 | 27.3 | 23.2 | 16.4 | 15.0 |
| wide | 87.8 | — | 68.8 | 70.1 | 66.4 | 56.6 |
| tall | 72.7 | 31.2 | — | 54.3 | 52.9 | 40.5 |
| science | 76.8 | 29.9 | 45.7 | — | 51.9 | 42.9 |
| turtle | 83.6 | 33.6 | 47.1 | 48.1 | — | 41.4 |
| economic | 85.0 | 43.4 | 59.5 | 57.1 | 58.6 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 32 / 536 | 5.97% |
| Germany | 126 / 536 | 23.51% |
| Mongols | 25 / 536 | 4.66% |
| Sparta | 29 / 536 | 5.41% |
| Byzantium | 63 / 536 | 11.75% |
| Athens | 69 / 536 | 12.87% |
| Switzerland | 89 / 536 | 16.60% |
| Celts | 103 / 536 | 19.22% |

Faction max/min: **5.040×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

### master

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.9438**; observed **0.7360**. Length mean **150.8**, median **144.0**, range 93–475 turns. Captures: 296.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 22.5 | 33.4 | 40.3 | 34.8 | 22.3 |
| wide | 77.5 | — | 68.8 | 70.1 | 66.4 | 56.6 |
| tall | 66.6 | 31.2 | — | 54.3 | 52.9 | 40.5 |
| science | 59.7 | 29.9 | 45.7 | — | 51.9 | 42.9 |
| turtle | 65.2 | 33.6 | 47.1 | 48.1 | — | 41.4 |
| economic | 77.7 | 43.4 | 59.5 | 57.1 | 58.6 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 39 / 536 | 7.28% |
| Germany | 116 / 536 | 21.64% |
| Mongols | 33 / 536 | 6.16% |
| Sparta | 29 / 536 | 5.41% |
| Byzantium | 65 / 536 | 12.13% |
| Athens | 69 / 536 | 12.87% |
| Switzerland | 76 / 536 | 14.18% |
| Celts | 109 / 536 | 20.34% |

Faction max/min: **4.000×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -7.3 | [-12.5, -2.1] |
| economic / science | +0.0 | [+0.0, +0.0] |
| economic / tall | +0.0 | [+0.0, +0.0] |
| economic / turtle | +0.0 | [+0.0, +0.0] |
| economic / wide | +0.0 | [+0.0, +0.0] |
| rush / science | +17.1 | [+9.9, +25.3] |
| rush / tall | +6.1 | [-2.8, +14.6] |
| rush / turtle | +18.4 | [+14.1, +23.0] |
| rush / wide | +10.2 | [+6.0, +14.1] |
| science / tall | +0.0 | [+0.0, +0.0] |
| science / turtle | +0.0 | [+0.0, +0.0] |
| science / wide | +0.0 | [+0.0, +0.0] |
| tall / turtle | +0.0 | [+0.0, +0.0] |
| tall / wide | +0.0 | [+0.0, +0.0] |
| turtle / wide | +0.0 | [+0.0, +0.0] |

### policy-v2

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.9449**; observed **0.7152**. Length mean **148.2**, median **143.0**, range 93–475 turns. Captures: 176.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 21.2 | 30.1 | 35.4 | 36.3 | 20.3 |
| wide | 78.8 | — | 68.8 | 70.1 | 66.4 | 56.6 |
| tall | 69.9 | 31.2 | — | 54.3 | 52.9 | 40.5 |
| science | 64.6 | 29.9 | 45.7 | — | 51.9 | 42.9 |
| turtle | 63.7 | 33.6 | 47.1 | 48.1 | — | 41.4 |
| economic | 79.7 | 43.4 | 59.5 | 57.1 | 58.6 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 33 / 536 | 6.16% |
| Germany | 127 / 536 | 23.69% |
| Mongols | 23 / 536 | 4.29% |
| Sparta | 29 / 536 | 5.41% |
| Byzantium | 64 / 536 | 11.94% |
| Athens | 69 / 536 | 12.87% |
| Switzerland | 82 / 536 | 15.30% |
| Celts | 109 / 536 | 20.34% |

Faction max/min: **5.522×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -5.3 | [-10.3, -0.9] |
| economic / science | +0.0 | [+0.0, +0.0] |
| economic / tall | +0.0 | [+0.0, +0.0] |
| economic / turtle | +0.0 | [+0.0, +0.0] |
| economic / wide | +0.0 | [+0.0, +0.0] |
| rush / science | +12.2 | [+5.9, +18.3] |
| rush / tall | +2.8 | [-5.2, +11.6] |
| rush / turtle | +19.9 | [+15.2, +24.7] |
| rush / wide | +8.9 | [+5.2, +12.9] |
| science / tall | +0.0 | [+0.0, +0.0] |
| science / turtle | +0.0 | [+0.0, +0.0] |
| science / wide | +0.0 | [+0.0, +0.0] |
| tall / turtle | +0.0 | [+0.0, +0.0] |
| tall / wide | +0.0 | [+0.0, +0.0] |
| turtle / wide | +0.0 | [+0.0, +0.0] |

### frontier-cities

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.9066**; observed **0.7421**. Length mean **147.4**, median **142.0**, range 93–475 turns. Captures: 204.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 11.3 | 22.5 | 21.5 | 22.6 | 18.1 |
| wide | 88.7 | — | 68.6 | 70.2 | 65.2 | 56.9 |
| tall | 77.5 | 31.4 | — | 54.3 | 52.9 | 40.5 |
| science | 78.5 | 29.8 | 45.7 | — | 51.9 | 42.9 |
| turtle | 77.4 | 34.8 | 47.1 | 48.1 | — | 41.4 |
| economic | 81.9 | 43.1 | 59.5 | 57.1 | 58.6 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 37 / 536 | 6.90% |
| Germany | 126 / 536 | 23.51% |
| Mongols | 19 / 536 | 3.54% |
| Sparta | 32 / 536 | 5.97% |
| Byzantium | 66 / 536 | 12.31% |
| Athens | 65 / 536 | 12.13% |
| Switzerland | 83 / 536 | 15.49% |
| Celts | 108 / 536 | 20.15% |

Faction max/min: **6.632×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -3.0 | [-6.8, +0.7] |
| economic / science | +0.0 | [+0.0, +0.0] |
| economic / tall | -0.1 | [-0.3, +0.0] |
| economic / turtle | +0.0 | [+0.0, +0.0] |
| economic / wide | -0.3 | [-0.8, +0.0] |
| rush / science | -1.6 | [-8.1, +5.1] |
| rush / tall | -4.9 | [-11.9, +2.1] |
| rush / turtle | +6.2 | [+1.4, +10.9] |
| rush / wide | -1.0 | [-5.0, +3.1] |
| science / tall | +0.0 | [+0.0, +0.0] |
| science / turtle | +0.0 | [+0.0, +0.0] |
| science / wide | -0.2 | [-0.5, +0.0] |
| tall / turtle | +0.0 | [+0.0, +0.0] |
| tall / wide | +0.2 | [+0.0, +0.5] |
| turtle / wide | +1.2 | [+0.0, +3.6] |

### market-plunder

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.9084**; observed **0.7655**. Length mean **146.9**, median **142.0**, range 93–475 turns. Captures: 201.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 12.0 | 28.0 | 23.7 | 16.7 | 14.5 |
| wide | 88.0 | — | 68.8 | 70.1 | 66.4 | 56.6 |
| tall | 72.0 | 31.2 | — | 54.3 | 52.9 | 40.5 |
| science | 76.3 | 29.9 | 45.7 | — | 51.9 | 42.9 |
| turtle | 83.3 | 33.6 | 47.1 | 48.1 | — | 41.4 |
| economic | 85.5 | 43.4 | 59.5 | 57.1 | 58.6 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 34 / 536 | 6.34% |
| Germany | 124 / 536 | 23.13% |
| Mongols | 24 / 536 | 4.48% |
| Sparta | 27 / 536 | 5.04% |
| Byzantium | 64 / 536 | 11.94% |
| Athens | 69 / 536 | 12.87% |
| Switzerland | 90 / 536 | 16.79% |
| Celts | 104 / 536 | 19.40% |

Faction max/min: **5.167×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | +0.5 | [-0.5, +2.0] |
| economic / science | +0.0 | [+0.0, +0.0] |
| economic / tall | +0.0 | [+0.0, +0.0] |
| economic / turtle | +0.0 | [+0.0, +0.0] |
| economic / wide | +0.0 | [+0.0, +0.0] |
| rush / science | +0.5 | [+0.0, +1.5] |
| rush / tall | +0.6 | [-3.4, +4.5] |
| rush / turtle | +0.3 | [+0.0, +0.7] |
| rush / wide | -0.3 | [-1.0, +0.3] |
| science / tall | +0.0 | [+0.0, +0.0] |
| science / turtle | +0.0 | [+0.0, +0.0] |
| science / wide | +0.0 | [+0.0, +0.0] |
| tall / turtle | +0.0 | [+0.0, +0.0] |
| tall / wide | +0.0 | [+0.0, +0.0] |
| turtle / wide | +0.0 | [+0.0, +0.0] |

### wall-masonry

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.9158**; observed **0.7605**. Length mean **147.3**, median **143.0**, range 93–475 turns. Captures: 197.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 11.5 | 29.7 | 23.6 | 20.7 | 16.8 |
| wide | 88.5 | — | 68.8 | 70.1 | 66.4 | 56.6 |
| tall | 70.3 | 31.2 | — | 54.3 | 52.9 | 40.5 |
| science | 76.4 | 29.9 | 45.7 | — | 51.9 | 42.9 |
| turtle | 79.3 | 33.6 | 47.1 | 48.1 | — | 41.4 |
| economic | 83.2 | 43.4 | 59.5 | 57.1 | 58.6 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 33 / 536 | 6.16% |
| Germany | 124 / 536 | 23.13% |
| Mongols | 25 / 536 | 4.66% |
| Sparta | 29 / 536 | 5.41% |
| Byzantium | 64 / 536 | 11.94% |
| Athens | 66 / 536 | 12.31% |
| Switzerland | 88 / 536 | 16.42% |
| Celts | 107 / 536 | 19.96% |

Faction max/min: **4.960×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -1.7 | [-6.5, +2.4] |
| economic / science | +0.0 | [+0.0, +0.0] |
| economic / tall | +0.0 | [+0.0, +0.0] |
| economic / turtle | +0.0 | [+0.0, +0.0] |
| economic / wide | +0.0 | [+0.0, +0.0] |
| rush / science | +0.4 | [-2.3, +3.1] |
| rush / tall | +2.3 | [+0.3, +5.3] |
| rush / turtle | +4.3 | [+1.8, +7.0] |
| rush / wide | -0.8 | [-2.1, +0.4] |
| science / tall | +0.0 | [+0.0, +0.0] |
| science / turtle | +0.0 | [+0.0, +0.0] |
| science / wide | +0.0 | [+0.0, +0.0] |
| tall / turtle | +0.0 | [+0.0, +0.0] |
| tall / wide | +0.0 | [+0.0, +0.0] |
| turtle / wide | +0.0 | [+0.0, +0.0] |

### combined

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.9131**; observed **0.7500**. Length mean **147.4**, median **142.0**, range 93–475 turns. Captures: 206.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 13.0 | 24.0 | 21.1 | 22.1 | 17.8 |
| wide | 87.0 | — | 68.6 | 70.2 | 65.2 | 56.9 |
| tall | 76.0 | 31.4 | — | 54.3 | 52.9 | 40.5 |
| science | 78.9 | 29.8 | 45.7 | — | 51.9 | 42.9 |
| turtle | 77.9 | 34.8 | 47.1 | 48.1 | — | 41.4 |
| economic | 82.2 | 43.1 | 59.5 | 57.1 | 58.6 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 35 / 536 | 6.53% |
| Germany | 127 / 536 | 23.69% |
| Mongols | 20 / 536 | 3.73% |
| Sparta | 33 / 536 | 6.16% |
| Byzantium | 72 / 536 | 13.43% |
| Athens | 66 / 536 | 12.31% |
| Switzerland | 76 / 536 | 14.18% |
| Celts | 107 / 536 | 19.96% |

Faction max/min: **6.350×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -2.8 | [-6.6, +1.6] |
| economic / science | +0.0 | [+0.0, +0.0] |
| economic / tall | -0.1 | [-0.3, +0.0] |
| economic / turtle | +0.0 | [+0.0, +0.0] |
| economic / wide | -0.3 | [-0.8, +0.0] |
| rush / science | -2.1 | [-7.7, +3.4] |
| rush / tall | -3.4 | [-10.5, +4.6] |
| rush / turtle | +5.7 | [+1.5, +10.1] |
| rush / wide | +0.8 | [-3.8, +5.5] |
| science / tall | +0.0 | [+0.0, +0.0] |
| science / turtle | +0.0 | [+0.0, +0.0] |
| science / wide | -0.2 | [-0.5, +0.0] |
| tall / turtle | +0.0 | [+0.0, +0.0] |
| tall / wide | +0.2 | [+0.0, +0.5] |
| turtle / wide | +1.2 | [+0.0, +3.6] |

### early-siege

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.9092**; observed **0.7590**. Length mean **146.5**, median **141.0**, range 93–475 turns. Captures: 191.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 9.8 | 22.0 | 20.3 | 18.2 | 16.1 |
| wide | 90.2 | — | 68.7 | 70.6 | 65.5 | 56.4 |
| tall | 78.0 | 31.3 | — | 54.3 | 52.9 | 40.5 |
| science | 79.7 | 29.4 | 45.7 | — | 51.9 | 42.9 |
| turtle | 81.8 | 34.5 | 47.1 | 48.1 | — | 41.6 |
| economic | 83.9 | 43.6 | 59.5 | 57.1 | 58.4 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 34 / 536 | 6.34% |
| Germany | 126 / 536 | 23.51% |
| Mongols | 26 / 536 | 4.85% |
| Sparta | 27 / 536 | 5.04% |
| Byzantium | 68 / 536 | 12.69% |
| Athens | 68 / 536 | 12.69% |
| Switzerland | 83 / 536 | 15.49% |
| Celts | 104 / 536 | 19.40% |

Faction max/min: **4.846×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -1.1 | [-5.8, +3.1] |
| economic / science | +0.0 | [+0.0, +0.0] |
| economic / tall | -0.1 | [-0.3, +0.0] |
| economic / turtle | -0.2 | [-0.5, +0.0] |
| economic / wide | +0.2 | [-1.0, +1.6] |
| rush / science | -2.9 | [-6.6, +0.9] |
| rush / tall | -5.3 | [-10.2, -1.0] |
| rush / turtle | +1.8 | [-1.5, +4.9] |
| rush / wide | -2.4 | [-5.6, +0.3] |
| science / tall | +0.0 | [+0.0, +0.0] |
| science / turtle | +0.0 | [+0.0, +0.0] |
| science / wide | -0.5 | [-1.2, +0.0] |
| tall / turtle | +0.0 | [+0.0, +0.0] |
| tall / wide | +0.1 | [-0.5, +0.8] |
| turtle / wide | +0.9 | [+0.0, +2.6] |

### all-four

Scheduled 540; eligible 536; censored 2; common excluded maps 2. Assigned winner entropy **0.8984**; observed **0.7571**. Length mean **146.9**, median **141.0**, range 93–475 turns. Captures: 208.

| Row / opponent | rush | wide | tall | science | turtle | economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rush | — | 12.5 | 21.4 | 20.1 | 20.7 | 17.4 |
| wide | 87.5 | — | 68.6 | 70.3 | 65.3 | 56.6 |
| tall | 78.6 | 31.4 | — | 54.3 | 52.9 | 40.5 |
| science | 79.9 | 29.7 | 45.7 | — | 51.9 | 42.9 |
| turtle | 79.3 | 34.7 | 47.1 | 48.1 | — | 41.6 |
| economic | 82.6 | 43.4 | 59.5 | 57.1 | 58.4 | — |

| Faction | Wins / seats | Win rate |
| --- | ---: | ---: |
| Japan | 35 / 536 | 6.53% |
| Germany | 123 / 536 | 22.95% |
| Mongols | 20 / 536 | 3.73% |
| Sparta | 30 / 536 | 5.60% |
| Byzantium | 68 / 536 | 12.69% |
| Athens | 66 / 536 | 12.31% |
| Switzerland | 85 / 536 | 15.86% |
| Celts | 109 / 536 | 20.34% |

Faction max/min: **6.150×** (target ≤1.5×).

| Policy | Clear counters in this sample |
| --- | --- |
| rush | economic, science, tall, turtle, wide |
| wide | None demonstrated |
| tall | economic, wide |
| science | economic, tall, wide |
| turtle | economic, wide |
| economic | wide |

Paired score change versus the experiment control (percentage points):

| Row / opponent | Change | Paired 95% interval |
| --- | ---: | ---: |
| economic / rush | -2.4 | [-6.6, +2.2] |
| economic / science | +0.0 | [+0.0, +0.0] |
| economic / tall | -0.1 | [-0.3, +0.0] |
| economic / turtle | -0.2 | [-0.5, +0.0] |
| economic / wide | +0.0 | [-1.3, +1.5] |
| rush / science | -3.1 | [-9.3, +3.2] |
| rush / tall | -5.9 | [-14.8, +2.8] |
| rush / turtle | +4.3 | [-1.4, +10.4] |
| rush / wide | +0.3 | [-4.4, +4.7] |
| science / tall | +0.0 | [+0.0, +0.0] |
| science / turtle | +0.0 | [+0.0, +0.0] |
| science / wide | -0.3 | [-0.7, +0.2] |
| tall / turtle | +0.0 | [+0.0, +0.0] |
| tall / wide | +0.2 | [-0.4, +0.9] |
| turtle / wide | +1.1 | [+0.0, +3.4] |

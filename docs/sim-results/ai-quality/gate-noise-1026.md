# Gate noise audit, October 26

**The gate is repeatable, but the old comparisons mixed protocols and over-read sampling noise.** The 342 and 399 master scores came from different engine revisions; the 381 run used unreseeded dice. At one frozen commit, identical settings and binary produced **399/1,024 twice: 0.00 pp run-to-run spread and 1,024/1,024 identical full game records**. Different salts still matter: four disjoint master blocks ranged from 358 to 399 wins, a **4.00 pp** spread.

**New decisions:** allow the 10-turn truce, Strongest Army measure and walls shooting over terrain **with monitoring**. Hold the current Modern auto-breach and latest aircraft implementations for AI adaptation. Retain all five designer rule decisions. Nothing is merged into master.

## Individual rule results

Each row compares the candidate with the matching prefix of the same frozen
control. Intervals include the planned three-look, five-rule correction. “Allow
with monitoring” follows the designer-rule policy; it does **not** prove that the
rule is harmless or within the noninferiority margin.

| Rule | Games per arm | Master / candidate wins | Paired change | Adjusted interval | New implementation decision |
| --- | ---: | ---: | ---: | ---: | --- |
| 10-turn truce | 4,096 | 1531 / 1484 | -1.15 pp | [-3.62, +1.32] pp | Allow with monitoring |
| Strongest Army measure | 4,096 | 1531 / 1510 | -0.51 pp | [-3.09, +2.07] pp | Allow with monitoring |
| Walls shooting over terrain | 4,096 | 1531 / 1494 | -0.90 pp | [-3.42, +1.61] pp | Allow with monitoring |
| Modern auto-breach | 4,096 | 1531 / 1374 | -3.83 pp | [-5.76, -1.91] pp | Hold implementation; adapt AI |
| Aircraft Oil, latest full fix | 1,024 | 399 / 228 | -16.70 pp | [-21.09, -12.31] pp | Hold implementation; adapt AI |

The aircraft full-branch comparison does not isolate the Oil requirement from
repaired Search air orders. Preserve the Oil rule, hold this combined branch for
AI adaptation/benchmark repair, and use a matched-AI Oil/free comparison to
attribute its effect. Earlier matched-AI evidence is discussed below. Individual
acceptance also does not approve a bundle automatically.

[All final comparisons](gate-noise-1026/rule-summary.json) include the descriptive
95% intervals. Every planned look is retained in each rule's `*-comparisons.json`;
compressed `*-result.json.gz` files retain win vectors and game/binary/source
hashes. These are benchmark snapshots of the requested branches, not landing
branches with their functional checks and generated pins refreshed.

## Protocol and decision

`tools/sim/quality_gate.py` prepares priority-35 box filler specifications, runs one binary at a time, saves full per-game records and binary/source provenance, and compares each candidate with a frozen master control. The control source is `d3d5dc112`; this is unchanged runtime code from the later 399-win controls. The five latest rule branches are each applied separately to this source, without merging rules together. Only Rust source/example changes enter these benchmark-only source commits; no generated pin or fixture is edited. They are not landing branches.

| Requested source branch | Source tip | Benchmark snapshot on frozen master |
| --- | --- | --- |
| `astra/peace-cooling-off` | `f19806b51` | `cbe213ae5` |
| `astra/fix-strongest-army-measure` | `c11e66fad` | `578deef3c` |
| `astra/city-strike-no-los` | `9e3ba6463` | `15a86c8cb` |
| `astra/modern-auto-siege` | `ab6792591` | `d4f71cc3b` |
| `astra/fix-aircraft-oil-gate` (latest `aircraft-use-oil` fix) | `16cb30575` | `9689a55e2` |

The eight fixed maps have equal weight. The human seat uses Hard with builtin parameters; the original opponents use their recorded Hard difficulty **with Search enabled**. “All seats” applies parameters to every seat; it does not disable opponent Search. The exact map setups, arms JSON, reseeding, salt ranges, and stopping plan are in [protocol.json](gate-noise-1026/protocol.json); original and benchmark source revisions are in [sources.json](gate-noise-1026/sources.json).

Each candidate is checked at 128, 256 and 512 salts per map (1,024, 2,048 and 4,096 games per arm). The frozen control is generated through 512 salts once; a second process repeats its first 128 salts using the same commit and binary. Candidate preparation refuses to proceed unless the per-game hashes and binary hashes agree. Candidate runs append disjoint salt ranges instead of rerunning the prefix. Environment variables that could activate experiments are removed; dice reseeding is explicitly enabled. No local simulator fallback exists. The complete control ran on box filler at priority 35. The repeat was queued at the same priority, then cancelled after higher-priority work delayed it; after verifying budget headroom, the repeat and candidate simulations used bounded Spot batches. Both reproducibility controls use 48 Rayon threads; candidate batches use 64 threads. Game RNG is keyed by map/seat/salt, not scheduling: the first 1,024-game control at 48 threads matched every historical record from the 64-core spot run. Worker metadata/specs retain the thread count.

For each map and salt, let `d = candidate_win − control_win`, taking values −1, 0, +1. The effect is the equally weighted map mean. Fixed-map stratified variance is `sum(sample_variance(d_map)) / (8² × salts)`. We also estimate variance from the salt-wise averages across all maps, and use the larger standard error. This preserves the old salt-cluster sensitivity without allowing accidental cross-map covariance cancellation to shrink the decision interval.

Descriptive intervals use ±1.96 standard errors. Decisions use normal-approximation intervals with Bonferroni correction for **three looks × five candidates**, family alpha 0.05 (critical z ≈ 2.935). This controls repeated peeking and multiplicity to the accuracy of the underlying normal approximation; it is not an exact finite-sample confidence sequence. Fewer than 20 discordant pairs or zero estimated variance forces an unresolved interval rather than spurious zero uncertainty. Constant ±1 differences are the exception: a conservative Hoeffding bound over independent salt averages detects an across-the-board regression without estimating variance. This edge-case guard does not change any observed campaign comparison.

- Upper adjusted bound below −1.5 pp: **clear harm; hold the implementation and fix AI adaptation**. This does not reverse the designer’s rule decision.
- Lower adjusted bound above −1.5 pp: **noninferior; stop and allow the rule**.
- Otherwise continue to the next planned look; at 4,096, **ship with monitoring**, explicitly retaining uncertainty.

This policy is for already-decided game rules. It is not a reason to ship speculative AI optimizations without evidence. A crash, invalid order, broken scenario or failed functional test remains a separate correctness issue. These simulations do not replace branch verification or prove that a bundle of individually acceptable rules is acceptable; the eventual integration needs its own paired test.

## Why master moved

| Report | Control source | Binary SHA-256 prefix | Game dice | Wins / 1,024 |
| --- | --- | --- | --- | ---: |
| Rule bundle | `59b839f4` | `bde3d8eed7a6f36f` | reseeded per salt | 342 |
| Landable bundle | `0a73cfc5` | `bcde52b875c77656` | reseeded per salt | 399 |
| Raze | recorded batch `a76aa7621`, same runtime as `0a73cfc5` | `bcde52b875c77656` | **not reseeded** | 381 |
| Modern auto-breach | `acc7ddfc0`, same runtime as `0a73cfc5` | `bcde52b875c77656` | reseeded per salt | 399 |

Every cohort uses the same eight map fixtures and salt indices 1–128; the two-server runs concatenate 1–64 and 65–128. Seeds here identify AI salts, not freshly generated maps. Reseeding changes combat, movement and barbarian RNG after map creation while retaining the same map. The Raze batch log downloads the **same binary** as the Landable control and does not export `LEGATUM_HVH_RESEED_DICE`; its command is `--brain hard --all-seats --games 128 --seed 1`. The other controls explicitly enable reseeding and pass default parameters. Search remains on for the recorded opponents in both CLI paths. Reseeding is a substantial change within individual maps: Sparta 8.6 goes from 92/128 wins with reseeding to 32/128 without it, while Sparta 8.4 moves from 79 to 110. The modest aggregate difference hides those opposing changes.

Between `59b839f4` and `0a73cfc5`, runtime changes include Copper/Iron reveal, natural-wonder tile rules and AI valuation, held/public Great Works, and pre-Modern cavalry Outrider. The benchmark harness change only adds Great Work VP to telemetry; the maps do not change. Thus the 57-win increase is engine drift, not evidence of an unstable identical experiment. The 18-win difference on the later runtime comes with a dice-reseeding change and a default-constructor entry-point change. `params::empty_params_are_the_builtin_ai` and constructor inspection establish equivalent defaults; the arms/native parameterized controls match outcome-for-outcome. The retained logs identify reseeding as the meaningful settings difference, rather than different opponents or salts. No per-change attribution within the engine bundle is claimed.

[Historical outcomes and original artifact hashes](gate-noise-1026/historical-controls.json) preserve the per-map win vectors used for this comparison. Original command/build logs remain under `~/LegatumRuns/gate-rules-1025/`, `~/LegatumRuns/gate-landable-1025/`, and the cloud batch directories named in that JSON.

## Measured salt noise

The frozen control is **1,531/4,096 (37.38%)**. Its four disjoint 1,024-game blocks
(salts 1–128, 129–256, 257–384, 385–512) scored **399, 358, 383, 391**: a 4.00 pp
range and a sample standard deviation of **1.73 pp**. Four blocks are too few to
estimate tail behavior reliably; the per-salt estimates below use all 512 salts.
The same-input repetition is a separate determinism measurement, not a fifth
independent block. At frozen commit `d3d5dc112`, binary
`95a7043e8189daf51339c765a8711a2afcde2e80fa3c10c9994ab6fc3f6424a5`,
both executions scored **399/1,024**, with **all 1,024 full game-record hashes
identical** and **0.00 pp run-to-run spread**. Both used 48 threads, the same
arms, maps, salts and reseeding; the first ran on box filler and the second on
Spot. [Repeatability proof](gate-noise-1026/repeatability.json) and
[repeat outcomes/provenance](gate-noise-1026/repeat-result.json.gz) are retained.
Two executions support determinism for this cohort; they do not establish that
all future runtimes, platforms or settings are deterministic.

| Fixed map | Master wins / 512 | Win rate | Per-salt win variance |
| --- | ---: | ---: | ---: |
| 8.3-switzerland-win | 54 | 10.55% | 0.0945 |
| 8.4-sparta-win | 301 | 58.79% | 0.2427 |
| 8.4-japan-win | 313 | 61.13% | 0.2381 |
| 8.4-celts-win | 16 | 3.12% | 0.0303 |
| 8.5-mongols-win | 23 | 4.49% | 0.0430 |
| 8.5-switzerland-win | 186 | 36.33% | 0.2318 |
| 8.6-germany-loss | 286 | 55.86% | 0.2470 |
| 8.6-sparta-win | 352 | 68.75% | 0.2153 |

These variances are for Bernoulli win outcomes, not uncertainty about the map's
identity. The equally weighted fixed-map control SE at 4,096 is **0.640 pp**;
clustering by salt gives **0.689 pp**, which the conservative decision calculation
uses. Scaling that sampling variance gives:

| Games per arm | Control SE | Descriptive 95% half-width | 80%-power detectable difference, independent arms |
| ---: | ---: | ---: | ---: |
| 1,024 | 1.38 pp | 2.70 pp | 5.46 pp |
| 2,048 | 0.97 pp | 1.91 pp | 3.86 pp |
| 4,096 | 0.69 pp | 1.35 pp | 2.73 pp |

The final paired-rule table uses each rule's actual paired disagreement variance;
it should be smaller when common salts correlate the two outcomes. There is no
single universal paired noise value: a rule that changes many game trajectories
has a different variance from a mostly inactive rule. MDE is a normal-approximation
planning estimate for 80% power at a two-sided 5% test, `(1.96 + 0.842) × SE`.
For a harm floor at −1.5 pp, detecting harm requires that separation **below the
floor**, not merely a 1.5 pp difference from zero. Sequential/family-adjusted MDEs
are larger and are retained in each comparison artifact.

[Noise calculations](gate-noise-1026/control-noise.json) and the
[frozen control outcomes and provenance](gate-noise-1026/control-result.json.gz)
are retained with this report. Full telemetry remains in the filler artifacts.

### Paired sensitivity actually observed

The table projects each rule's final observed paired variance to each sample size;
these are planning estimates, not additional simulation runs. “Adjusted” uses
the three-look/five-rule critical value; the other columns use a nominal two-sided
5% test. All values are percentage points, with 80% target power.

| Rule | MDE at 1,024 | MDE at 2,048 | MDE at 4,096 | Adjusted MDE at 4,096 |
| --- | ---: | ---: | ---: | ---: |
| 10-turn truce | 4.71 | 3.33 | 2.36 | 3.18 |
| Strongest Army measure | 4.92 | 3.48 | 2.46 | 3.32 |
| Walls shooting over terrain | 4.80 | 3.39 | 2.40 | 3.23 |
| Modern auto-breach | 3.67 | 2.60 | 1.84 | 2.48 |
| Aircraft Oil, latest full fix | 4.19 | 2.96 | 2.09 | 2.82 |

Per-map paired variance of `candidate_win − control_win` at each rule's final
look is below. The rule table above gives the sample sizes; variances are in
squared win-indicator units. Differences across maps and rules are substantial.

| Map | Truce | Army | Walls | Modern | Aircraft |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8.3-switzerland-win | 0.1252 | 0.1251 | 0.1702 | 0.0567 | 0.0521 |
| 8.4-sparta-win | 0.3914 | 0.3952 | 0.3761 | 0.2331 | 0.2589 |
| 8.4-japan-win | 0.4149 | 0.4867 | 0.4265 | 0.2592 | 0.2694 |
| 8.4-celts-win | 0.0625 | 0.0587 | 0.0604 | 0.0622 | 0.0393 |
| 8.5-mongols-win | 0.0763 | 0.0724 | 0.0567 | 0.0294 | 0.0312 |
| 8.5-switzerland-win | 0.3606 | 0.3852 | 0.4243 | 0.2787 | 0.2510 |
| 8.6-germany-loss | 0.4185 | 0.4109 | 0.4149 | 0.2703 | 0.3108 |
| 8.6-sparta-win | 0.3826 | 0.3816 | 0.3554 | 0.2192 | 0.2327 |

[Fresh-salt sensitivity](gate-noise-1026/fresh-salt-sensitivity.json) separately
reports salts after the previously inspected first 128. It is descriptive only
and does not add another stopping test. On those 3,072 fresh games, the effects
are **truce −0.91 pp, Army +0.16 pp, walls −0.10 pp and Modern −3.71 pp**.
Modern's fresh-only descriptive 95% interval is [−5.19, −2.23] pp; its negative
result is not solely driven by the previously inspected block. Aircraft stopped
before additional salts were needed under the predeclared rule.

## Interactions and what the gate measures

The old rule bundle's −2.15 pp interval [−5.08, +0.88] did not establish harm beyond the floor. The later five-rule bundle was different: −5.66 pp [−9.18, −2.34], so its unadjusted interval did lie below −1.5. That supports concern about that bundle, not blame for every constituent. The later isolated peace, army and strike estimates were −1.86, −2.54 and −3.32 pp; all three upper 95% bounds exceeded −1.5. Adaptive addition/removal followed by a point cutoff cannot identify an interaction. Changes to diplomacy, city fire and scoring can interact, but demonstrating one requires the paired interaction contrast `AB − A − B + control`, with its own uncertainty, at one frozen source and protocol.

This metric is Hard's **relative win share against Search**, not a direct measure of absolute AI competence or human enjoyment. The latest aircraft fix restores opponent air orders previously omitted by Search/Opening wrappers, in addition to charging Oil and improving air planning. Its historical matched-AI counterfactual (same repaired AI, Oil versus free aircraft) found −0.39 pp [−2.64, +1.86]; that is inconclusive for harm, even though the complete branch had a much larger negative master comparison. The current audit measures the requested latest branch versus frozen master; it cannot attribute all that branch's change to Oil. Preserve the designer's Oil decision and distinguish the implementation/benchmark repair from reversing that rule. Original evidence: `astra/fix-aircraft-oil-gate:docs/sim-results/ai-quality/aircraft-oil-fix/README.md`.

The eight maps are a small, purposive set of saved human games. Salt intervals describe those fixed maps, not new-map generalization. Salt indices 1–128 have also been inspected during previous branch selection; the later 384 salts extend coverage, but these are not wholly untouched confirmatory data. A future daily campaign should predeclare a fresh salt block (`prepare --salt-start N`), retain fixed weights, and reserve additional maps for confirmation. A control reused across candidates is still a sample: pairing includes its uncertainty, and shared-control correlation is why the family correction does not assume independent candidates.

## Budget and reproducibility

At **2026-10-26 09:56 UTC**, a read of Cost Explorer and the account budget showed the month's
spend with room left under the monthly cap. The budget had last updated hours earlier, so it is
not an instant meter. Reserve $25 for billing lag and other work rather than treating all of the
remaining headroom as available. The control used the existing build box's filler at priority 35; its 4,096 games
and build took 4,598 seconds. After a higher-priority filler queue delayed the
repeat, the audit used the authorized Spot fallback. A campaign ledger reserves
each launch at the on-demand price plus IPv4 for its full 30-minute maximum,
with a **$25 task ceiling**, in addition to the $25 reserve for billing lag and other work.
Thus the verified spend plus both reserves stays below the monthly cap. Each instance
has shutdown and reaper deadlines. The completed Spot runs used approximately **4.82 server-hours**,
about **$3.17** at launch-time prices including IPv4.
The cumulative worst-case reservation was **$23.01**.
[Compute records](gate-noise-1026/compute.json) retain each batch, runtime, price
and instance ID. This is an estimate through each DONE marker; shutdown/S3 costs
and the authoritative AWS bill may differ. The existing box was shared, so its
4,598-second control allocation and 280-second cancelled repeat/build allocation
are reported separately from incremental Spot cost.
The final [billing read](gate-noise-1026/billing-final.json), at **11:55 UTC**,
still reports $340.615 with the same delayed update timestamp; it is not a final
invoice for this campaign. The [instance check](gate-noise-1026/instances.json)
found **no task instances active**: 17 terminated and the earliest no longer
listed. Other workloads and the shared build box were left untouched. Across
control, repeat and candidates, **22,528 game records** were completed and
validated. No AWS settings or monthly cap were changed.

Reproduce a campaign (use a new directory/id for a different frozen source):

```sh
python3 tools/sim/quality_gate.py prepare --directory RUN --commit MASTER_SHA \
  --id DAY-00-control --role control
# Submit the printed spec with tools/cloud/filler.sh submit SPEC --commit MASTER_SHA.
# Fetch completed results to RUN/control; repeat the same source with --role repeat
# and fetch to RUN/repeat. Then prepare each candidate with its exact source SHA:
python3 tools/sim/quality_gate.py prepare --directory RUN --commit CANDIDATE_SHA \
  --id DAY-rule --role candidate
# Submit/fetch, then inspect look-128.json, look-256.json and look-512.json.
python3 tools/sim/quality_gate.py analyze --protocol RUN/protocol.json \
  --control RUN/control/result.json --candidate RUN/rule/result.json
```

For a budget-authorized Spot run, build the exact source on the box, retain its
commit/binary/map SHA-256 manifest, and use `tools/cloud/run-batch.sh` with
`--max-hours 0.5`, the frozen `arms.json`, `LEGATUM_HVH_RESEED_DICE=1`, and
disjoint salts. Import only completed slices:

```sh
python3 tools/sim/quality_gate.py ingest --protocol RUN/protocol.json \
  --manifest RUN/rule-manifest.json --chunks RUN/rule-chunks.json \
  --out-dir RUN/rule
```

The manifest contains `commit`, `binary_sha256`, `maps_sha256`, `profile`,
`rustc`, and `machine`; each chunk names its `games_file`, `report_file`,
`log_file`, `done_file`, `salt_start` and `salts`. The importer checks the native
command and report against the frozen protocol, rejects failed/duplicate/missing
slices, and retains artifact hashes. Analyze after each planned cumulative look;
launch the next disjoint block only when the result is `continue`. The importer
does not launch instances or authorize spending.

Only complete matching protocols may share a control. A new master engine, map fixture, search setting, parameter arm, reseeding policy or salt schedule requires a new campaign. Freeze the control for the day and compare every rule against it; record later master drift instead of silently replacing it halfway through selection.

## Verification

The gate's 21 focused unit tests pass, including stopping decisions, paired
variance, invalid provenance, replication failure, salt overlap, family limits
and concurrent registration. Python compilation passes. Master `a6fc53bed` was
merged into this branch; its changes since the frozen control are documentation
only. The requested post-merge checks ran on the build box through
`tools/sim/build-slot.sh`, using `release-checked`: **12/12 determinism tests and
3/3 pin tests passed**, both commands exited 0 and returned no changed files.
[Commands and log hashes](gate-noise-1026/checks.json) are retained. This branch
changes gate tooling and audit evidence, not Rust game rules or AI code. Offline
analysis of the archived records reproduces all five decisions; source hashes,
record counts, report links and `git diff --check` also pass.

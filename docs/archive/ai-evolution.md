> Archived 2026-11-22: kept for history; it no longer describes the project.

# AI evolution: decision genes, paired selection, and held-out adoption gates

`tools/evolve/` tunes the scripted AI's decisions with CMA-ES. The harness never changes the
rules or installs its result as the default AI. `best-params.json` is the **paired-selected
incumbent**, not the final CMA-ES mean or the largest historical training score.

## Run it

Build from the reviewed current master rules, including diplomacy and Peace starts:

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
python3 tools/evolve/evolve.py --hours 7
```

Defaults: six new candidates per generation, **512 Proxy + 128 league games per candidate**,
plus one fresh incumbent evaluation with the same budget. Two simulators share the thread budget.
Every ten generations the incumbent gets a 512 + 128 fixed-seed checkpoint and a coach cycle.
The wall-clock budget reserves time for final validation; the 2,000-game Standard self-play gate
can take substantially longer than the old closing validation.

`--nice` is on by default. The harness lowers its scheduling priority to niceness 10, inherited
by simulators, detector replays, coach subprocesses, and film generation. `--no-nice` opts out.
If the OS refuses the priority change (as this restricted macOS session does), the harness logs
that failure, records the actual priority, and caps simulator threads at half the machine's
logical cores. `--threads 8 --parallel 2` explicitly leaves more room for interactive play.
Niceness yields to competing work; it is not a fixed CPU utilization limit.

Useful options: `--population`, `--proxy-games`, `--league-games`, `--bootstrap` (256 by default),
`--bench-proxy-games`, `--bench-league-games`, `--validate-games` (512 games),
`--balance-games` (2,000 per AI), `--coach-every`, `--no-coach`, `--no-film`, and
`--no-queue`. Small samples are useful for smoke tests, but cannot pass the adoption gates.

SIGTERM/Ctrl-C marks the run stopped. Resume with:

```sh
python3 tools/evolve/evolve.py --resume docs/sim-results/evolve-<run-id> --hours 7
```

Candidates, bootstrap draws, and cached games are deterministic on resume. The simulator binary is copied into the data directory before any game; rebuilds in the
main checkout cannot change an active run. Its hash also prevents mixing old caches with new
rules or AI code on resume. Start a new run after rebuilding.
Version 1 runs cannot resume in version 2: both the genome and selection contract changed.
Large caches and tapes live in `~/LegatumData/evolve/<run-id>` (or `--data`), outside git.

## Noise and selection

The first overnight run, `evolve-2026-09-28-0313`, used 16 candidates with 128 Proxy + 32 league
games each. Its fixed checkpoint had only 64 Proxy games: at a 59% win rate its binomial SE is
about **0.062**, not enough to distinguish the reported 0.59 → 0.50 → 0.64 checkpoints reliably.
The earlier documentation's 0.035 SE for 128 games was too small: near 50% it is **0.044**.

Re-scoring all sixteen generation-zero candidate caches with 256 bootstrap replicates measured
**median full-fitness SE 0.06665**, range **0.06402–0.07637**. This includes the survival term,
league term, faction/entropy/dominance penalties, and length/stall penalties. It is not just a
binomial interval on Proxy wins. The bootstrap samples whole games, preserving correlated seats,
and stratifies league draws by opponent. No model calls or new games are needed to reproduce it:

```sh
python3 tools/evolve/calibrate.py \
  ~/LegatumData/evolve/2026-09-28-0313/gens/g0 \
  --proxy-games 512 --league-games 128 --out /tmp/evolution-noise.json
```

The projected SE at the new fourfold game budget and the measured current-rules calibration are
recorded in [the v2 evidence report](../sim-results/2026-09-28-evolution-v2.md). Projection resamples
old games; current-rules measurement uses new games. Neither proves that an evolved AI is better.
Small improvements remain unresolved: the harness reports that uncertainty instead of promoting
an apparent win from an easier seed block.

- All candidates and the incumbent use the same **new seed block and league opponents** each
  generation. This common-random-number pairing was already present between candidates in v1;
  v2 extends it to the incumbent and computes paired uncertainty.
- CMA-ES recombines by **rank**, with log rank weights. The harness explicitly passes ranks after
  applying the distance-to-box penalty. Fitness magnitude cannot give a lucky candidate extra
  recombination weight.
- The incumbent's old score is discarded each generation. A challenger replaces it only if the
  lower paired bootstrap-normal confidence bound on its fitness improvement exceeds zero, with
  Bonferroni correction across that generation's population (family-wise alpha 0.05).
- This gate controls comparisons within a generation, not unlimited sequential experiments.
  Independent final validation is still mandatory. Bootstrap intervals are approximate, and
  discontinuous penalties can make the distribution non-normal.
- `generations.jsonl` records every candidate's SE/95% interval, rank, the freshly evaluated
  incumbent, and all paired deltas/SEs/adjusted intervals. `best` remains the raw generation leader
  for existing readers; it is not the export/adoption choice.
- Niche representatives are refreshed by generation. Historical scores from different seed
  blocks are not compared to keep a stale MAP-Elites winner forever.

A generation now simulates 7 × 640 = **4,480 games**, versus 2,560 previously. The current-rules
stock calibration measured **1,710.8 CPU-seconds per 640-game evaluation**, so a default generation
costs about **11,976 CPU-seconds**. At 14 effective cores the ideal estimate is 4.2 generations/hour;
allowing for seven evaluations on two workers, bootstrap, and checkpoints gives **about 3.5–4
generations/hour on an otherwise available 16-core Mac**. The eight-thread niceness fallback is
roughly half that rate. Shared load can reduce it much further. The old 9.8/hour estimate used
older rules and smaller candidate samples. See the evidence report for both measurements.

The same-rules stock calibration's full-fitness SE decreased from **0.0723 at 128+32** to
**0.0417 at 512+128**. It measures one stock candidate; future genotypes can have different
variance. `wall_s`, `cpu_s`, and the digest's observed generations/hour expose actual throughput.
Final validation also consumes the overnight budget, separately from generations.

## Genome and pillar 6

There are **71 genes**: the original five personality knobs for each of eight factions, plus
31 shared decision genes. Keeping the tactical/macro genes shared avoids multiplying a scarce
sampling budget across 248 separate faction-specific thresholds. Faction personalities and
faction-specific policies remain intact.

| Decisions | Genes / effect |
| --- | --- |
| Retreat and holding | recovery HP, sustainable incoming/retaliation ratio, doomed-garrison damage threshold |
| Attacking and bombardment response | local attack-ratio multiplier, counterattack/sortie maximum loss and sufficient win probabilities |
| Staging and campaigns | forecast damage safety multiplier for an exposed coordinated assault; campaign commitment ratio multiplier |
| Settlers and escorts | minimum site score, maximum home distance, travel penalty, destination commitment multiplier/bonus, commitment timeout, escort home/search distances |
| Purchasing | deficit reserve runway, surplus-unit upkeep runway and required gold multiple, minimum discretionary purchase value, approaching-enemy radius |
| Research | future-unlock divisor and military/economic/science value weights |
| Production | military, settler, and building value weights |
| Leader pressure | Hard/other-difficulty public-VP trigger and Hard's riskier siege trigger |

Defaults and bounded integer metadata live in the `decision_params!` table in
`crates/ai/src/params.rs`; `--print-ai-params` prints the values and `--print-ai-genes` prints the
catalog. Python normalizes each gene using its own bounds and checks its decision allowlist
against that catalog before a run. Genes reach real planner comparisons and scores; they do not
change combat forecasts, production prices, tech costs, yields, movement rules, or vision.

`war_commit_ratio` uses integer hundredths: 100–250 means 1.00–2.50 times the existing
personality-derived campaign threshold. Its default of 100 preserves that threshold and does
not alter local attacks or sorties. `settle_patience` is 4–16 turns (default 10). After that age
an unfounded settler prefers a safe legal site reachable within two
movement phases, while continuing its normal safe travel if no nearby site exists. These
defaults are part of the stock policy; an empty params file does not opt into another AI.

`AiParams` accepts an optional `decisions` object, with stock defaults for omitted fields.
Schema 1 personality files remain valid. `deny_unknown_fields` covers every typed level, and
both parsers reject unknown genes and out-of-range values. `rules_cannot_be_smuggled_in` covers
rule/yield/cost/vision injection. `legatum-core` still has no dependency on the AI.

The empty file `{"schema":1,"name":"builtin"}` and the explicit default file reproduce stock
**serialized tapes exactly** under Standard rules at all three difficulties (80-turn test cases).
A separate test changes only decision genes, proves a tape differs, and verifies deterministic
replay. Existing tactical, macro, faction, and determinism tests exercise the stock decisions.

This is a decision search space, not an implementation of every coach proposal. For example,
exposing a sortie threshold can improve a bombardment response, but cannot invent a new
multi-turn behavior. Behavior proposals still require separate code and A/B review.

## Rules, opponents, fitness, and seeds

Proxy v2 runs explicitly pass **`--proxy-start war`**, with rotated seat/faction and seven Hard
candidate seats. The simulator report must record Proxy version 2 and `war`. Proxy v2 was
calibrated without diplomacy; Peace starts are a different benchmark. Ordinary league and
self-play games use current **Peace starts** and the existing diplomacy AI. The harness checks
that their recorded setup is not legacy all-war. Start mode, source metadata, and simulator hash
are recorded, rather than inferred from a default CLI argument.

Fitness remains:

```
(1 - Proxy win rate) + 0.3 * AI survival to turn 150 + 0.5 * league win rate
- capped faction-ratio penalty - low-archetype-entropy penalty
- dominant-archetype penalty - stalls/panics penalty - implausible-length penalty
```

The exact coefficients are in `metrics.DEFAULT_FITNESS` and `run.json.config.fitness`. Faction
shrinkage is a training aid only. Final balance uses raw rates and simultaneous confidence
bounds. The league starts with stock Hard plus frozen files in `crates/ai/params`; it adds recent
candidates and niche representatives. Every generation plays stock plus rotating opponents.
Even game allocations alternate the four candidate and four opponent seats and sum to exactly
`--league-games`, including when that budget is not divisible by the number of league members.

| Purpose | First seed | Use |
| --- | ---: | --- |
| Training Proxy | 100001 | offset by generation × Proxy games |
| Training league | 500001 | offset by generation × league games |
| Fixed checkpoint Proxy / league | 900001 / 920001 | same seeds at every checkpoint |
| Held-out Proxy / league | 950001 / 970001 | final comparison only; paired stock/evolved |
| Held-out homogeneous self-play | 990001 | final faction/diversity comparison only |

The training loop refuses to cross a seed partition. Validation sizes must fit their disjoint
20,000-seed blocks. All training, benchmark, and validation simulations use Standard rules.

## Final validation and adoption

The selected incumbent is frozen **before** reading held-out results. The harness runs:

1. Stock and evolved versus Proxy v2, on identical 950001+ seeds, with each win rate's Wilson 95%
   interval and a paired improvement interval from simultaneous Wilson bounds on discordant
   outcomes. Identical outcomes produce an inconclusive interval, not false certainty.
2. Evolved versus stock in a Peace-start league, 970001+, alternating four seats per side. Report
   evolved wins over all requested games and a Wilson interval; stalls/draws do not disappear
   from the denominator.
3. Separate stock-only and evolved-only Peace self-play on 990001+, 2,000 games each. Report
   raw faction win rates, Bonferroni simultaneous faction intervals and a conservative upper
   bound on best/worst ratio, archetype and winning-strategy entropy, whole-game bootstrap
   intervals for winner entropy, matchup counts, dominance, stalls, and panics.

The digest and `run.json.final_validation.verdict` say **adopt** only if Standard simulations
establish a Proxy improvement and a Peace-league advantage at 95% confidence, the evolved faction-ratio
upper bound is ≤1.5 with eight factions and ≥2,000 self-play games, winning-strategy entropy's
lower bound is ≥0.55, every real classifier archetype has a sampled counter (≥20 pairs), no
archetype is dominant, and no relevant game stalls or panics. These are conservative review
criteria, not proof of universal balance. The classifier currently reports Rush, Wide, Tall,
Science, Turtle, Economic, and residual Generalist; it has no separate Diplomatic label.

Missing, skipped, failed, underpowered, or inconclusive validation means **don't-adopt**, with
specific reasons. A smoke run should get that verdict. Even an adopt verdict is a recommendation
for the lead; the harness never modifies stock defaults, merges, or deploys.

## Coach and artifacts

The coach retains its existing workflow: archive checkpoint tapes, sample incident-rich games,
run the detector, make one read-only model call, validate evidence, and fall back to a rule-based
coach on failure. `--no-coach` chooses that fallback directly. High-confidence new-behavior
proposals may enter the local Astra queue unless `--no-queue`; tunable-parameter and rule-change
proposals stay in the digest. A checkpoint uses the selected incumbent, so coach observations
refer to the genome actually eligible for export.

The small log contract is [ai-evolution-log.md](ai-evolution-log.md). Data files under `--data`
include state, params, compressed candidate caches, reports, and archived tapes. Keep those and
large generated reports/films out of git; commit only compact evidence needed for review.

## Checks

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release --no-fail-fast
cargo test -p legatum-sim --test determinism
cargo test --workspace
python3 -m unittest discover -s tools/evolve -p 'test_*.py'
```

Python tests cover CMA-ES, mixed gene bounds, rule injection, whole-game SE and paired draws,
unchanged outcomes, adoption gates, niceness denial, fresh incumbent scoring, league allocation,
run/resume, coach validation, digest, and existing reader compatibility.

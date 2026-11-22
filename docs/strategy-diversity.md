# Measuring strategic diversity

This is a Pillar 10 diagnostic, not a balance change or a claim that a scripted policy is
optimal. Ordinary sims now report observed archetypes. Experiments additionally report
assigned biases and the assignment-to-observation confusion table. Never substitute a
brain's name for evidence that it executed its strategy.

## Classifier v2

`legatum_sim::diversity::classify` consumes integer `Evidence`, also stored under
`games[].telemetry.strategy.empires`. `games[].players[].archetype` is its result. No
winner, faction, hidden intent, or assigned bias is an input. The observer never writes
game state or consumes game RNG. The tape/save formats and core rules are unchanged.

The early window is resolved turns 1–120; the development window ends at
turn 240. Development counts and
the last alive snapshot are frozen at that horizon or the empire's earlier elimination /
game end. Later conquest or a winning end state cannot erase a losing empire's opening.
Very short games can leave a strategy undeveloped: Generalist is a real residual category.

Production shares use completed items weighted by their **base tuning cost**,
including purchases once at completion. They exclude starting/captured units, unfinished
queues, settler surcharges, faction discounts, and upgrades. These are investment proxies,
not an accounting of actual production or gold spent. Building upgrades count in their
base family. Gold is gross city yield before empire upkeep. Only founded cities count
toward Wide; `early_peak_cities` separately includes captures.

The first matching row wins; this precedence is part of version 1:

| Archetype | Deterministic condition |
| --- | --- |
| Diplomatic (inactive hook) | At least 20 allied turns and one coalition led, from real diplomacy telemetry. Current observer leaves `diplomacy: null`; embassies do not count. |
| Rush | At least 3 early initiated battles (excluding meeting engagements), at least 120 listed military production, and military at least 50% of early completed cost. Scouts do not count as military production; their initiated battles can count. |
| Wide | At least 4 cities founded by the early horizon, including the capital. |
| Science | At least 2 cities with a Laboratory-family building in the development snapshot, and that family at least 25% of development completed cost. |
| Turtle/Wonder | At least 2 walled cities in the development snapshot, plus either a completed Great Work or Walls-family buildings at least 10% of development completed cost. |
| Economic | At least 2 cities with a Market-family building, plus either that family at least 20% of development completed cost or gross gold yield at least 6 per city. |
| Generalist | None of the above. Includes mixed plans, interrupted plans, and insufficient development. |

These thresholds are interpretable initial hypotheses, not learned labels. They can merge
hybrid strategies and miss failed attempts. Inspect the raw evidence and confusion table
before attributing a result to game design. If thresholds change, increment the classifier
version; do not silently relabel earlier studies.

## Report definitions

Both Markdown and JSON (`summary.diversity`) contain:

- `observed`: win rate (winning seats / seats), player frequency, winner frequency,
  normalized winner entropy, cross-archetype matchups, and faction-by-archetype wins/seats.
- `assigned`: the same metrics for the five forced policies, when present. A partially
  assigned game is excluded from this view rather than changing its player denominator.
- `assigned_vs_observed`: counts for auditing whether a forced policy realizes its label.
- `observed_check` and `assigned_check`: guardrail verdict, reasons, missing pairs,
  dominant candidates, policies without an observed counter, never-winners, and unobserved
  policies. Never-winner means sampled with zero wins; unobserved is separate.

Only finished, non-panicked games enter diversity metrics. Draws contribute all seats and
matchups but no winners. Stalls and missing/legacy labels are excluded and counted, not
treated as losses or Generalist. Winning teams contribute one winning seat per member;
the standard sim is eight single-empire teams. Determinism replays are not extra samples.

Entropy is Shannon `-sum(p ln p) / ln(K)`, with fixed K = 6 observed categories (including
Generalist) or K = 5 assigned policies. Zero-win categories stay in K. Diplomatic is excluded
until real evidence activates it. No winners gives null, never a fabricated 0 or 1.

For each game with X and Y, compare every cross-team X/Y seat pair. The game contributes
one unit of weight, shared equally among its pairs; duplicating X seats cannot give that
game extra weight. Rank winning-team members first, then surviving empires by final VP,
then eliminated empires by later elimination turn. Equal ranks tie, with no player-ID
tiebreak. Same-team pairs are excluded. JSON includes pair counts, co-occurrence games,
strict `ahead_rate`, `tie_rate`, and `score = ahead + ties/2`. Empty cells have null rates.
Markdown shows strict ahead / tie percentages and game counts. A score over 50% indicates
an observed advantage, but this is a multiplayer association, not a causal duel result.

## Forced scripted policies

`--brain scripted --strategies rush,science` alternates two biases across all eight seats.
Accepts 1, 2, 4 or 8 comma-separated values. Biases run at Normal difficulty and cannot be
combined with difficulty rotation or another brain mode. Each uses the same fogged view,
legal options, prices, tactical planner, emergency defense, and immediate-victory logic.
They change only preferences; they do not guarantee a particular outcome or observed label.

| Bias | Expansion / military / science / economy / aggression knobs | City target | 4x production preference | 4x research preference |
| --- | --- | ---: | --- | --- |
| Rush | 20 / 100 / 20 / 30 / 100 | 3 | Combat units; settlers receive 0.5x | Ballistics, Field Logistics, Combined Arms, Siege Engineering |
| Wide | 100 / 35 / 35 / 65 / 35 | 8 | Settlers | Soil Reclamation, Civic Networks, Public Administration |
| Science | 25 / 25 / 100 / 55 / 10 | 3 | Laboratory family | Signal Theory, Academies, Research Networks, Predictive Computation |
| Turtle | 0 / 65 / 65 / 65 / 0 | 3 | Walls family and Great Works | Fortification, Mutual Defense, Signal Theory, Civic Networks, Relay Restoration |
| Economic | 40 / 30 / 35 / 100 / 20 | 4 | Market family | Exchange, Commercial Law, Federated Markets |

Rush
bypasses the default opening rule that suspends army production for the first settler.
Otherwise zero-value/illegal choices remain excluded. Shared faction goals and tactical
responses can override a bias. The existing inheritance sensitivity experiment uses the
unbiased brain, so biased seats are excluded from that experiment, not compared to the
wrong counterfactual.

`--strategy-matchups` schedules all 10 unordered pairs, with four independent empires of
each policy in an eight-player free-for-all. Every pair plays two games on the same seed,
faction assignment, and starting positions, swapping every seat's bias in game two. Games
must be a positive multiple of 20. Game i uses `first_seed + floor(i/2)` (wrapping u64),
with pair `floor(i/2) % 10` in the order shown above. Each faction has exactly equal exposure
to both policies within every seed pair. Mirrored games are related samples, not two
independent maps. The rest of a four-vs-four lobby can change pairwise standings.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo run --release -p legatum-sim -- \
  --strategy-matchups --games 480 --seed 972001 --threads 6 \
  --label strategy-diversity --diversity-check --diversity-basis assigned
```

## Reusable guardrail

Tools can depend on the `legatum-sim` library and call
`diversity::check_diversity(&summary, GuardrailConfig::default())`. The summary and verdict
are serializable, including deserialization from the report JSON. The function never exits
the process. The CLI writes all reports before returning:

- 0: pass (or guardrail not requested).
- 1: any sampled strategy scores above 50% against every other sufficiently sampled
  strategy, or normalized winner entropy is below the configured minimum; also used for
  existing simulator execution/determinism failures.
- 2: insufficient evidence to pass—no winners, fewer than two active strategies, or any
  required pair has fewer than the configured number of finished co-occurrence games.

A demonstrated failure takes precedence over insufficient coverage. The default minimum
is 20 games per pair; this is a sample floor, not a significance test. Every category in the
summary is checked, including Generalist in observed reports. Missing categories cannot
be silently dropped to make a check pass.
`--diversity-basis observed` is the default; `assigned` and `both` are also available.
Use `--diversity-min-matchup-games N` and `--diversity-min-entropy H` to override limits.
Invalid, infinite, or NaN thresholds are rejected.

The `diversity_check` example re-evaluates saved JSON using the public function, without
running games or changing the source report. It checks both available bases, prints the
new verdicts as JSON, and can render standalone diversity Markdown:

```sh
cargo run -p legatum-sim --example diversity_check -- report.json \
  --min-entropy 0.80 --min-matchup-games 20 --markdown /tmp/diversity-audit.md
```

The proposed entropy floor is **0.80**, subject to the dated study's calibration. For five
policies this corresponds to at least about 3.6 equally common winning policies in effective
diversity, rather than requiring a perfectly even five-way distribution. It is an alarm for
concentration, not proof that all policies are viable. A high entropy can coexist with a
Condorcet winner, which is why the pairwise test is separate. Equal or uncertain matchups
can leave a policy without a demonstrated counter even when neither guardrail fails.

Design responses belong with the lead. This measurement does not tune or change game rules.

The initial calibration, 480-game controlled tournament, ordinary-AI comparison, and
candidate lead experiments are in [the 2026-09-27 study](sim-results/strategy-diversity-2026-09-27.md).

## Counter experiments (2026-09-28)

The post-diplomacy counter study is in
[the counter experiment report](sim-results/strategy-counters-2026-09-28.md).
Classifier v2 still leaves Diplomatic inactive: the simulation reports diplomacy separately,
but it does not yet supply the classifier's allied-turn and coalition-lead evidence, and
there is no forced Diplomatic policy. Do not infer that label from pacts or Embassy buildings.

The opt-in Rush policy now has an opening sequence in addition to the preference multipliers
above. Through turn 120, it delays settlers, funds a Workshop and
Market after its second melee unit, and reserves room for two ranged units in a five-unit
force (one garrison plus four field troops). Research follows Fabrication, Soil Reclamation,
Exchange, Fortification, Ballistics, then Field Logistics. Legal options, income, emergency
defense, immediate victory opportunities and fog still apply. Later production retains a
bounded field contingent and mixed composition. The policy avoids early pacts with observed
nearby rivals, chooses attainable nearby cities when declaring war, and gives a funded war
24 turns before routine peace consideration; endangered empires can still seek peace.
Rangers are available from turn 1; Ballistics unlocks Troopers, not Rangers.
These are scripted decisions, not extra resources or changes to ordinary faction AI.
The study found more early attacks but worse placements after this revision. Treat it as
an experimental opening, not a validated stronger opponent or proof of competent Rush play.

The following CLI switches are **experimental only** and have no default effect. They can
be combined with a comma-separated list. Their values are saved in `GameSetup.rule_experiments`,
so state hashes, save/load and replay resolution agree; absence means the unchanged rules.

```sh
legatum-sim --strategy-matchups --games 540 --seed 972001 \
  --threads 3 --rules-experiment frontier-cities,market-plunder,wall-masonry \
  --diversity-check --diversity-basis assigned --label counters-combined
```

- `frontier-cities`: an unwalled city below population four loses four intrinsic defense
  strength per missing population (−12 / −8 / −4 at populations 1 / 2 / 3).
- `market-plunder`: capturing an unwalled city dismantles each Market-family building and
  awards the captor gold equal to twice that building's listed production
  cost. Walls protect the market. The consumed building cannot pay again on recapture
  unless rebuilt. The rule uses actual infrastructure, never an assigned strategy label.
- `wall-masonry`: Walls add eight city strength, on top of the existing faction-specific
  Wall and Citadel contributions. This applies to city HP defense against melee and ranged
  attacks, including siege; it does not strengthen the unit garrison.
- `early-siege`: unlocks the existing Siege Engine at Ballistics instead of Siege Engineering.
  Production cost, movement, attack strengths, upkeep and garrison targeting stay unchanged.
  Legal production and the research unlock preview use the same experimental prerequisite.

Combat previews, public city strength, tactical planning and resolution share the same
experimental math. All faction abilities, unique unit stats, yields and drawbacks are retained.
Preserving those formulas does **not** prove balanced faction outcomes; compare win rates
separately before recommending any rule. No gold-interest lever was implemented because
there is no treasury-interest income in the current game.

`tools/analysis/strategy_counters.py` independently reconstructs and audits each raw shard's
matrix against Rust, then compares a common set of complete mirrored maps across runs.
Its first argument is the experiment control; additional arguments are `label=report.json`
or `label=report.json.gz`. Comma-separated files form consecutive complete tournament blocks.
It reports censored game identities, common map exclusions, worst-case score bounds for
those exclusions, map-cluster bootstrap intervals, paired score changes, both winner
entropies, faction win rates and game lengths. At least 16 complete maps per matchup must
remain; otherwise analysis fails and every treatment needs more scheduled blocks. This
complete-case comparison is descriptive, not a claim that censoring is random.

Replay exports now retain the native game index in both filenames and index metadata:
`<label>-game-00001-seed-972001.json`. A tape imported alone retains the same index parsed
from its native filename. Two orientations sharing a seed are distinct; a duplicate
(label, game index, seed) is still rejected. Unindexed legacy tapes retain seed-only names.

## Structural expansion experiments (2026-09-28)

The [wide-counter study](sim-results/wide-counters-2026-09-28.md) adds four independent
`--rules-experiment` flags, also off and omitted from serialized experiment objects by
default. The preceding four flags remain supported. No flag depends on a policy label.

- `settler-escalation`: increases the existing +40 Production per previously trained
  Settler to +120. The starting Settler is free and uncounted; trained and purchased
  Settlers use the existing counter. The faction price applies and nothing follows it.
  Purchase quotes and the price increase for sequential purchases share this step.
- `city-research`: adds 5% per currently owned city beyond two to the diffusion-adjusted
  research price, rounding the surcharge up. Captured and resisting
  cities count. Losing a city reduces future quotes; stored Science is unchanged.
- `city-upkeep`: each currently owned city costs `max(city count - 2, 0)` Gold per turn.
  Total administration is therefore 0/0/3/8/15/24 Gold at 1/2/3/4/5/6 cities. It is
  separate from military upkeep, gets no Strategic Logistics discount, and uses the
  existing bankruptcy rules. Per-turn income and upkeep are fixed amounts.
- `expansion-coalitions`: adds `floor(10 * max(cities - 2 * members, 0) / members)` to
  the team's integer VP progress percentage when selecting coalition targets. Members
  means permanent-team members, including eliminated members. Existing 60-with-15-lead
  and 80-with-strict-lead thresholds and equal-pressure tie protection remain. Coalition
  exit checks use the same pressure; ordinary VP and pact cutoffs retain their meaning.
  City counts are already public; no hidden terrain, city positions or policy labels
  enter the calculation. The public reason gives the pressure breakdown.

The analyzer also reports assigned-policy wins, observed-policy realization, final/peak
city counts, survival, and coalition trigger/combat telemetry. `leader_*` counters retain
their VP-only definition; `coalition_*` counters follow the active rule. A coalition
target being at war is exposure, not proof that an explicit coalition caused that war.
Keep new raw reports, tapes, executables and bootstrap JSON outside Git, under
`/tmp/legatum-sim-artifacts/<run>/`; commit the evidence summary and reproduction commands.

# Duel opening experiments

Pure Python standard library. Build the simulator on this branch, then run from
the repository root:

```sh
cargo build --release -p legatum-sim
python3 tools/duel/duel.py --run .progress/duel-default --seeds 100
python3 tools/duel/duel.py --run .progress/duel-tuned --seeds 100 \
  --members ~/LegatumRuns/2026-09-30-standard/psro/active-members.json
python3 tools/duel/duel.py --run .progress/duel-rush --seeds 100 \
  --opening-genome Rush=/absolute/path/rush.json
```

The only game pace is Standard; the defaults are small maps, Hard scripted AI, barbarians on,
and a seeded faction draw. `Default` uses the scripted Hard, with its existing
behaviour. Other openings use their built-in policies unless overridden. Files
can be simulator integer vectors or named gene objects (schema order is retained,
unknown keys and invalid values fail). `--members` accepts the league manifest;
null genomes mean built-in. Relative genome paths resolve beside the manifest.
The manifest and genome files are snapshotted by value at startup.

`--seeds 100` means **200 games per unordered pair**: 100 maps, each with both
orientations. All 28 pairs including self-play produce **5,600 games** and fill
the 7×7 matrix. Opposite cells describe the same games from the other opening's
perspective. Every pair uses the same seeds, improving comparisons across the
matrix. `--openings Rush Economic` runs a subset for a smoke check. The same-name
matchups play the same genome on both sides; the native simulator keys overrides
by opening, so it cannot pit two Rush genomes against each other in one game.

Workers never exceed `max(1, cpu_count()-2)`; `--workers` can lower that limit.
Each native batch uses one thread. Children request nice 10, or retain inherited
lower priority (for example nice 20). Where the host denies priority changes,
they retain the inherited priority and the run records that value. Each invocation has a `--timeout` (900 s by
default); interrupting a run terminates its owned simulator process groups. No
process-name killing is used. Run directories have an exclusive writer lock, a
frozen binary with SHA-256, configuration checks, and atomic compressed batch
caches; rerunning the same command reuses completed games. Changed scientific
inputs require a new run directory. A timeout/error fails the run, never turns a
missing game into a loss.

Outputs are `matrix.md`, `summary.json`, `matrix-history.jsonl`, and
`games.jsonl.gz`. All native seat fields are preserved, including opening, VP,
win, checkpoints and phases. Per-game reports retain turns, victory type, outcome,
and mirror orientation. Full native telemetry is also kept in the batch cache.

## Interpretation

The row opening scores 1 for a win, 0 for a loss, and 0.5 for a completed draw.
If either orientation stalls at the safety cap, **the whole map is dropped** from
that pair's estimate; attempted games, stalled games, dropped maps and retained
maps are explicit. Rates therefore condition on completed maps when stalls occur.
No turn-limit wins are accepted. Diagonal rates are 50% by self-play symmetry.

The 95% confidence intervals use finite-sample Hoeffding bounds on independent
map means, not 200 independent Bernoulli games. They are deliberately
conservative. Direction tests use exact two-sided sign tests on map scores
relative to 0.5 (tied maps contribute no direction), with Holm correction across
all off-diagonal unordered pairs. If completed draws produce quarter-valued map
scores, a bounded-mean Hoeffding test replaces the sign test (which would then
test the median rather than the mean). A cycle needs three winning edges significant
at family-wise 0.05, with at least eight retained maps each. The star is the
paired direction test; its stronger use of ties can detect an edge even when the
conservative interval includes 50%. These tests do not establish a counter in
eight-player games. The four intended counters explicitly named in the task are
reported as supported, reversed, or not established.

The `validity` section in `summary.json` and `matrix.md` checks every included
opening (including Default): it must significantly beat at least one other
opening, must not significantly beat all others, and must average at least 0.35
against the others. This is an equally weighted mean of off-diagonal scores;
missing matchups fail the mean check. The report lists significant wins/losses,
each criterion's result, and a VALID / NOT VALID verdict with failure reasons.
It reuses the existing Holm tests and cycle results; a triangle is reported as
evidence, not an additional validity requirement.

The separate `identity` section reports observed behavior for each opening. All
thresholds and their rationale live in [`identity.json`](identity.json). It starts
with a required share of 60%: Rush declares war **and** reaches or attacks an enemy
city by Standard turn 80; GreedyExpand founds three cities by turn 120 with a light
military budget; Snowball needs expansion and production, and Economic needs Markets and gold yield. Default is an unconstrained
anchor (N/A). These initial constraints need calibration; they do not assert that
the current built-ins already behave as named or that satisfying them wins games.

Turns use the fixed Standard pace. The native strategy observer records founding and military investment up
to turn 120 and development yields up to turn 240 at Standard. Military investment
is measured in listed production costs, excluding Scouts; it is a proxy
for military commitment, not surviving unit count. Development yields use the last living snapshot up to the horizon, even if play
ends earlier. Science and gold are gross per-turn city yields, not treasury totals.

Identity rates count **seat-games**, retaining losses, draws and stalls, with
both seats counted in self-play. Unlike win estimates, identity never drops a
map because one orientation stalled. A null event timestamp means the event
never happened; an absent field means old/missing telemetry. Missing evidence
never earns a pass. The JSON includes known failures, unknown games, missing
fields, failed checks and a pass-rate upper bound; the matrix flags missing
coverage alongside FAIL. Phase labels, wars against barbarians and late captures
cannot substitute for early opponent-city pressure.

Three observer-only native seat fields supply previously missing evidence:
`first_opponent_war_turn`, `first_opponent_city_pressure_turn`, and
`guarded_expansion_turns`. Pressure excludes Scouts and requires either adjacency
while at war, an actual unit attack on the opponent's city (including an attacker
that dies), or occupation of an already-zero-HP city even without a new battle.
A guarded expansion has another owned city and a friendly
non-Scout military unit within one hex when founding resolves. Existing lifetime
escort counters include the starting Settler and late travel, so they cannot prove
early defended expansion. These fields never feed back into orders, RNG or state.

The Nash solver tries multiplicative replicator dynamics then support
enumeration. It certifies the empirical matrix with deviation regret ≤1e-6;
missing cells produce an explicitly unconfirmed equilibrium. A pure equilibrium
on a noisy empirical matrix is not statistical proof of dominance.

## Coevolution

```sh
python3 tools/duel/duel_evolve.py --run .progress/duel-evolve \
  --generations 24 --population 6 --train-seeds 24 --holdout-seeds 100
# Start from the league's current members instead:
python3 tools/duel/duel_evolve.py --run .progress/duel-evolve-tuned \
  --members ~/LegatumRuns/2026-09-30-standard/psro/active-members.json
# Prioritize difficult opponents and the least valid selected opening:
python3 tools/duel/duel_evolve.py --run .progress/duel-pfsp \
  --opponent-weighting pfsp --targets Rush,Snowball,Economic --target-order worst-first
# Preserve each opening's measured identity during tuning (off by default):
python3 tools/duel/duel_evolve.py --run .progress/duel-identity --identity-gates
```

Generations rotate through all six tunable openings; Default remains a fixed
anchor. Each opening keeps a separate CMA-ES optimizer in normalized [0,1] gene
space, with bounded integer decoding and a boundary penalty. Other openings get
uniform coverage; opponents are drawn 50% from current active members and 50%
from their hall of fame. The hall starts with built-ins and the supplied members,
retains every accepted version, and is actually sampled in subsequent games.
Historical versions of the target itself are excluded because overrides are
keyed by opening name.

`--opponent-weighting uniform` preserves the original seeded sampling. With
`pfsp`, each distinct current or historical opponent version gets weight
`(1 - p)^2 + 0.05`, where `p` is the target's score in the latest committed
matrix. The opening-keyed matrix supplies the current member's score for hall
versions too; unobserved scores use `p = 0.5`. Versions appearing in both the
current roster and hall are counted once. Training and holdout independently
sample this same weighted pool; `opponent_weights` records its members, scores
(null if unobserved), and weights in each generation record.

`--targets Opening,...` limits evolution, keeping all other members as
opponents. The default includes every tunable opening in member order; an
explicit list sets the round-robin order. `--target-order worst-first` instead
chooses the selected opening failing the most validity criteria in the committed
matrix, breaking ties by lowest mean score (unobserved first), then name. The
default remains `round-robin`. Worst-first respects `--max-consecutive` (default
2) and selects every target at least once in each sliding `--fairness-window`
(default twice the number of targets). It reserves slots for targets approaching
their deadlines, then keeps the worst-first ordering among eligible targets.
The window must be at least the target count; a single target is exempt from the
consecutive limit. Scheduling uses committed checkpoint history across resumes.
These options are pinned in `config.json`; missing options in older configs take
their defaults, and changing saved values requires a fresh run directory.

All training candidates and the incumbent use the same maps, mirror pairs and
opponents. A stalled map scores zero in training. Exactly one training-selected
candidate sees the next fresh held-out block, paired against a freshly evaluated
incumbent with identical opponents and factions. Seed high bits separate training,
holdout and matrix domains; generations never reuse training or holdout maps.

The win-improvement promotion path requires at least eight retained maps, a mean
score improvement greater than `--min-delta` (0.0025), and a two-sided **exact
paired sign-flip test at fixed alpha 0.01**. Both mirrored games are averaged
before comparing policies. Any map
with a stall in either candidate or incumbent is dropped from both sides. A
candidate also cannot increase the number of held-out stalls. The logged normal
99% effect interval is descriptive; the exact randomization test controls the
acceptance decision (under paired sign-exchangeability). Alpha is per comparison,
not a family-wise guarantee across a long search. Use fresh final validation for
any genome proposed for adoption. Tuning never changes the game defaults.

`--identity-gates` shapes training fitness as `score - boundary_penalty -
fitness_penalty * max(0, required_share - pass_rate)`. The configured penalty is
initially 1000. Candidates gain credit for approaching their identity even when
all candidates fail the gate. CMA-ES minimizes the negative of this fitness, and
only the highest-fitness training candidate sees holdout.

With identity gates enabled, promotion requires the candidate's **held-out**
identity pass rate to be at least `min(required_share, incumbent held-out pass
rate)`. A failing incumbent can improve without clearing the whole gate at once;
an incumbent already passing the gate must keep passing. In addition to the
win-improvement path, a candidate can promote by significantly increasing a
failing incumbent's identity pass rate, provided held-out win delta is at least
-0.02, at least eight completed maps remain, and held-out stalls do not increase.
Identity improvement uses a one-sided exact paired sign-flip test at alpha 0.01
on pass-count differences per map. Both mirror orientations stay together;
stalls and missing telemetry remain in the identity denominator, with missing
evidence earning no pass. This is a per-test threshold under paired
sign-exchangeability, not a family-wise guarantee across both promotion paths
or generations.

Each generation records training gate rates, fitness and penalties, both
held-out gate reports, the identity regression floor, the exact identity test,
and the promotion reason (`wins`, `identity`, or null). The identity flag is off
by default; reports still measure identity when it is off. Both the flag and the
complete threshold file are pinned in `config.json`; changing them requires a
fresh run directory.

`generations.jsonl` logs every candidate, fitness, held-out comparison, opponents,
and promotion. `matrix-history.jsonl` holds the initial matrix and one entry per
generation; unchanged generations reuse their last measured matrix. Accepted
members trigger a fresh matrix (unchanged matchups reuse exact cached games).
`state.json` and `checkpoints/` retain CMA covariance/RNG, active members and hall
of fame. Journals are reconstructed from the committed checkpoints on resume;
`--generations` may increase. `active-members.json`, `hall-of-fame.json`, and
`genomes/` expose the accepted policies.

`tools/duel` reuses the shared `tools/evolve/cmaes.py`; its original source was
`evolution/league-phase2:tools/evolve/cmaes.py`. `equilibrium.py` is copied from
that branch's `tools/league/equilibrium.py`. Source revision:
`0a4e26b076f1ace3d04d8fed7105058ee53c8c3f`.

## Native flags and checks

```sh
target/release/legatum-sim --players 2 --map-size small \
  --games 3 --mirror --factions Japan,Celts --barbarians true \
  --seat-opening 0=Rush --seat-opening 1=GreedyExpand \
  --determinism --out-dir .progress/duel-smoke
python3 -m unittest discover -s tools/duel -p 'test_*.py'
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release
cargo test --workspace
```

Native defaults remain eight players and 48×32 (`medium`); `small` is 40×26 and
`large` is 60×40. Counts 2–8 work with every preset. Faction names are
case-insensitive, exactly one per seat; repeats allow same-faction tests. Without
`--factions`, the historical shuffle draws distinct factions from the same seeded
stream. `--mirror` reverses active opening/parameter/strategy assignments while
holding the setup, map, factions and seat difficulties fixed. For more than two
seats it reverses the active seat order. It requires scripted brains and does
not combine with Proxy or the older eight-player strategy matchup scheduler.
`--games N --mirror` writes 2N separate games and uniquely indexed tapes. Proxy
and the calibrated strategy scheduler retain their eight-player restriction.

`turns` already existed and is preserved. The additive `victory_type` is `vp`,
`elimination`, `turn_limit` (only when separately requested), or null for a draw,
stall or panic; existing `outcome` distinguishes these. `mirrored: true` marks the
second orientation. Default setup encoding, game rules, order tapes and RNG
sequences remain unchanged.

# Strategic diversity — 2026-09-27

Pillar 10 is **not supported by this experiment**. The Economic scripted policy has an observed finish-order advantage against every other forced policy, so it has no observed counter in the tested set. All six policies win games. Winner entropy is high (0.9713), showing why varied winners alone are insufficient to detect a dominant policy.

This is a result about these scripted policies on Quick, eight-empire free-for-all games. It does not establish that Economic is unbeatable by a competent human, a stronger Rush policy, a different lobby composition, or an unimplemented diplomatic strategy. Observed behavior gives a different alarm—Wide—which illustrates why assigned intent and realized play must be reported separately.

## Experiment and reproducibility

- Main study: **480 Quick games**, 240 map seeds, 15 unordered policy pairs, **32 games / 16 mirrored seed pairs per matchup**. Four seats use each policy; every empire remains on its own team. The second game reverses every seat's policy while preserving map, faction, and starting position. Each policy gets 640 seats, including exactly 80 seats of each faction.
- Ordinary-AI comparison: **120 Quick games**, 960 seats, unmodified Normal scripted faction personalities. This is a separate population, not a held-out replication of the forced tournament.
- All **600 games ended by VP victory**. Zero panics, draws, conquest victories, explicit turn-limit wins, or safety-cap stops. No artificial turn limit; safety cap 600.
- Main seeds 972001–972240; comparison seeds 973001–973120. A separate 30-game pilot at 971001–971015 informed the initial classifier; its games are excluded here. The final classifier and biases are documented in [strategy-diversity.md](../strategy-diversity.md).
- The run-start checkout was `1a1ade2` with this branch's implementation uncommitted (`dirty: true` in JSON). The guardrails were subsequently re-evaluated with the final public API, including Generalist; per-game measurements did not change. `diversity_audit` records that command in each JSON. No game-rule, tape-format, or determinism-fixture changes were made.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo run --release -p legatum-sim -- \
  --strategy-matchups --games 480 --seed 972001 --speed quick --threads 6 \
  --label strategy-diversity-forced --diversity-check --diversity-basis assigned
cargo run --release -p legatum-sim -- \
  --games 120 --seed 973001 --speed quick --threads 3 \
  --label strategy-diversity-natural --diversity-check
```

Both commands return **1 for a diversity alarm**, not an execution failure. Reports are written before the exit. The ordinary comparison also has insufficient coverage in 9 of its 21 observed-category pairs; its low entropy is already a measured failure. The forced study covers every assigned and observed pair above the 20-game floor.

Full generated reports retain per-faction wins, all denominators, strict ahead/tie matrices, and the assignment/observation confusion table:

- [Forced tournament Markdown](2026-09-27-strategy-diversity-forced.md) and [full JSON, gzip](2026-09-27-strategy-diversity-forced.json.gz).
- [Ordinary-AI Markdown](2026-09-27-strategy-diversity-natural.md) and [full JSON, gzip](2026-09-27-strategy-diversity-natural.json.gz).
- [Independent matrix audit and paired uncertainty](strategy-diversity-2026-09-27-analysis.json).

## Assigned policies: wins and execution

Win rate is winning seats / seats. Winner frequency is wins / 480. Every policy has 16.67% of all seats. Realization is the share of assigned seats that independently meet that policy's behavioral classifier; missing realization is not relabeled as success.

| Assigned policy | Wins / seats | Win rate | Winner frequency | Realized matching label |
| --- | ---: | ---: | ---: | ---: |
| Rush | 51 / 640 | 7.97% | 10.62% | 194 / 640 (30.3%) |
| Wide | 112 / 640 | 17.50% | 23.33% | 203 / 640 (31.7%) |
| Tall | 82 / 640 | 12.81% | 17.08% | 427 / 640 (66.7%) |
| Science | 76 / 640 | 11.88% | 15.83% | 357 / 640 (55.8%) |
| Turtle/Wonder | 48 / 640 | 7.50% | 10.00% | 312 / 640 (48.8%) |
| Economic | 111 / 640 | 17.34% | 23.12% | 239 / 640 (37.3%) |

Economic has 111 wins and Wide 112; a finish-order advantage is not the same statistic as first-place frequency. Economic can place ahead of more Wide empires while a particular Wide empire wins the whole game.

## Forced-policy matchup matrix

Cell = **strict ahead % (half-tie score %)** for row versus column. Every cell has 32 games, 512 cross-policy seat pairs, and 16 independent map seeds. Each game has equal weight. Rank: winning team first; survivors by VP; eliminated empires by later elimination. Equal ranks tie. Half-tie score = ahead + ties/2; the guardrail calls an advantage above 50%. These are multiplayer placements, not duel win rates.

| Row vs column | Rush | Wide | Tall | Science | Turtle/Wonder | Economic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 33.8 (39.2) | 20.7 (26.3) | 29.1 (34.0) | 37.3 (43.6) | 24.4 (29.2) |
| Wide | 55.5 (60.8) | — | 44.3 (52.4) | 46.1 (52.1) | 51.6 (57.2) | 36.1 (42.8) |
| Tall | 68.2 (73.7) | 39.5 (47.6) | — | 43.4 (51.6) | 57.0 (62.8) | 30.5 (39.1) |
| Science | 61.1 (66.0) | 42.0 (47.9) | 40.2 (48.4) | — | 48.4 (57.0) | 29.3 (37.5) |
| Turtle/Wonder | 50.2 (56.4) | 37.1 (42.8) | 31.4 (37.2) | 34.4 (43.0) | — | 29.5 (35.2) |
| Economic | 66.0 (70.8) | 50.6 (57.2) | 52.3 (60.9) | 54.3 (62.5) | 59.2 (64.8) | — |

Observed counters to each policy, using half-tie score:

| Policy | Policies that finish ahead more often than behind |
| --- | --- |
| Rush | Wide, Tall, Science, Turtle/Wonder, Economic |
| Wide | Economic |
| Tall | Wide, Economic |
| Science | Wide, Tall, Economic |
| Turtle/Wonder | Wide, Tall, Science, Economic |
| Economic | **None observed** |

**Dominant candidate and policy without an observed counter: Economic. Never-winning policies: none.** Rush and Turtle/Wonder have the lowest win rates, but both win and Turtle counters Rush. Diplomacy is untested and unavailable; it must not be listed as a strategy that never wins.

## Uncertainty

Economic's 95% descriptive bootstrap intervals remain above 50% against each opponent. The bootstrap resamples whole mirrored seed pairs, not the 512 correlated seat comparisons. There are only 16 independent seeds per matchup; intervals do not adjust for multiple comparisons or choosing the strongest policy after observing the results.

| Economic against | Half-tie score | Paired bootstrap 95% interval |
| --- | ---: | ---: |
| Rush | 70.8% | 64.6–76.6% |
| Science | 62.5% | 59.3–65.5% |
| Tall | 60.9% | 55.7–66.1% |
| Turtle/Wonder | 64.8% | 60.3–69.5% |
| Wide | 57.2% | 52.0–62.1% |

The uncertainty audit reconstructs placements from per-game records and reconciles all 15 pair means and sample counts against Rust's summary, then uses 10,000 bootstrap replicates with fixed analysis seed 972001:

```sh
python3 tools/analysis/strategy_diversity.py \
  docs/sim-results/2026-09-27-strategy-diversity-forced.json.gz
```

The controlled study is balanced across factions and seats, but each lobby contains only two policies. A six-way mix, different map settings, Standard speed, stronger scripted policies, and held-out seeds remain untested. No claim of human-level AI competence follows from these self-play results; the [Proxy benchmark plan](../ai-competitiveness.md) remains relevant.

## Observed behavior and entropy calibration

| Population | Winner entropy | Effective equally frequent categories | Dominant alarm | Never-winning observed categories |
| --- | ---: | ---: | --- | --- |
| Forced: assigned policies | 0.9713 | 5.70 / 6 | Economic | None |
| Forced: observed behavior | 0.8708 | 5.44 / 7 | Wide | None |
| Ordinary AI: observed behavior | 0.6836 | 3.78 / 7 | None established; incomplete coverage | None |

In the forced games, observed Wide wins 113 of its 286 seats (39.5%) and has no observed counter. This is a behavioral association: reaching four founded cities by turn 60 is itself evidence of a successful opening. It does not establish that assigning Wide causes the advantage. Economic assignments are frequently realized as Tall, Wide, or Generalist; only 37.3% meet the Economic label. Rush realization is 30.3%, Wide 31.7%, and Turtle 48.8%. Policy quality and classifier thresholds must therefore be audited before attributing the alarms to engine balance.

In ordinary play, Generalist wins 58 games and Wide 39: together **97/120 (80.8%)** of winners. Generalist is 506/960 (52.7%) of all players. All seven categories still win at least once, but rare Science/Economic encounters leave 9 pairs below the coverage floor. A coarse residual label can hide distinct plans; the current classifier is an interpretable first measurement, not ground-truth strategy recognition.

Propose a **0.80 normalized entropy floor** and **20 finished games per pair** as an initial regression alarm. The floor distinguishes the ordinary sample's concentrated 0.684 from the forced study's 0.971 assigned / 0.871 observed diversity. It corresponds to about 4.19 equally frequent winning policies out of six (or 4.74 observed categories out of seven). It deliberately permits unequal strengths. This calibration uses the present data, not an independent validation set; re-evaluate it after classifier or policy revisions. The dominance check remains necessary: both forced populations exceed the entropy floor and still trigger a pairwise alarm.

The pilot showed that a 20% defense-spending cutoff missed wall-heavy Turtle play because walls are cheap relative to later development. Version 1 uses 10% plus at least two walled cities, or two walled cities plus a Great Work. Production shares are nominal base-cost proxies, not actual expenditure. Late plans and hybrids can remain Generalist because evidence freezes at Quick turn 120. No policy or game-rule tuning was performed after the main study began.

## Candidate experiments for the lead

1. Audit Rush execution before changing combat balance. Only 30.3% of assigned Rush seats meet the early-attack/investment threshold. Check contact timing, reinforcement purchases, ranged harassment, and target selection against the Proxy. Repeat the Economic/Rush pair with a competent attacker.
2. Test an economic vulnerability that creates a counter. Candidate levers are the timing of gold-to-defense conversion, the opportunity cost of reinvestment, and exposure of economic infrastructure or future trade routes to raids. Measure whether a committed attacker can exploit that exposure while economic investment retains a worthwhile payoff.
3. Test expansion versus concentrated development. Candidate levers are settlement payback, distance/administration costs, defender travel time, and the timing of tall/science score opportunities. Use assigned-policy experiments to distinguish a real tradeoff from the observational advantage of already having expanded successfully.
4. Add coalition measurement when diplomacy lands. Leader coalitions may supply a counter missing from the current policy set. Fill the real alliance/coalition telemetry hook, add a policy, version the classifier, and rerun; embassy construction alone is not diplomacy evidence.

No game rules were changed to address these findings. Faction balance, player agency, and human acceptance need separate checks before choosing any design lever.

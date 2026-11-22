# Scenario gate

A fast A/B gate for AI changes built from short scenarios instead of full games (review
`docs/status/fable-review-2026-11-09/2-training.md`, section 2 and increment D). A tool the lead
runs; it is not part of the merge gates.

```sh
tools/sim/scenario-gate.sh CANDIDATE.json                 # held-out cards, candidate in the seat
tools/sim/scenario-gate.sh CANDIDATE.json --set train     # the cards you may tune on
tools/sim/scenario-gate.sh CANDIDATE.json --mode rivals   # candidate plays the seat's rivals
tools/sim/scenario-gate.sh --aa                           # Hard against itself on other dice
```

`CANDIDATE.json` is an `AiParams` gene file (as for `--ai-params` everywhere else); `--base FILE`
replaces the built-in Hard as the baseline. Exit 0 on PASS, 1 on FAIL; the run directory holds
`runs.jsonl`, `report.md` and `report.json`.

## What it does

- **Cards:** every Lab card (`scenarios/lab/*.json`) and war scenario (`scenarios/war/*.json`), 44
  on 2026-11-09. A war file cut from the same position as a Lab card is played once.
- **Runs:** each card is played `--runs` times (default 480, sized for the 15 held-out cards) per arm. Run `r` uses the same dice
  seed in both arms (the derivation of `legatum-sim scenario` and the war suite), so the arms pair run
  by run. The seat plays Hard; the rivals keep the card's difficulty; nobody searches; a war
  scenario's script (his recorded orders) and a card's rusher still apply. The engine:
  `crates/sim/examples/scenario_gate.rs` (Rust, rayon over all runs); the statistics:
  `tools/sim/scenario_gate_report.py` (Python, aggregation only).
- **Score:** the card's own score (`ActiveScenario::status`, what the app shows the player) and
  whether the objective was met. Per card, the delta is the mean paired score difference divided by
  the card's score standard deviation (both arms, floored at 10 points), so cards with different
  score scales pool. The pooled delta is the unweighted mean over cards, with standard error
  sqrt(Σ se_k²)/K. The objective-met delta (points) pools the same way.
- **Verdict:** noninferiority. PASS when the pooled score delta's 95% lower bound is above −0.05
  card SDs **and** the objective-met delta's is above −1.5 pp. Calibrated below on all 44 cards at
  128 runs: the A/A run (unpaired, the widest a null change gets) passes with lower bounds −0.045
  and −1.10; siege-off fails with −0.060 and −3.01. On the 15 held-out cards alone the A/A
  interval at 128 runs is too wide for these margins (score lower bound −0.076), which is why the
  wrapper's default is 480 runs (checked below). The margins are the lead's to tighten as the run
  count grows.

## Resolution (2026-11-09, master `916e35431`)

One run of three arms on all 44 cards, 128 runs each: Hard (`base`), Hard on other dice
(`alt`, the A/A arm, run offset 100,000) and `candidates/siege-off.json` (the four siege-stance
genes that went on by default on 11-08, turned back off). 16,896 card runs in 903 s wall on this Mac
with 16 threads while other jobs held the load average at 100 to 160; 0.26 CPU-s per card run when
measured on a quieter machine (176 runs in 5.4 s, 46 CPU-s).

| Arm against Hard | Score delta, card SDs [95%] | Objective met, pp [95%] | MDE score / objective |
|---|---|---|---|
| A/A (same AI, other dice; unpaired) | −0.011 [−0.045, +0.023] | −0.37 [−1.10, +0.36] | 0.049 / 1.04 |
| siege-off (paired) | **−0.042 [−0.060, −0.025]** | **−2.50 [−3.01, −1.99]** | 0.025 / 0.73 |

- **The noise floor is honest:** the A/A interval covers zero on both measures.
- **Pairing halves the noise:** for siege-off the paired standard error is 0.0089 SD against
  0.017 if the runs were independent, because most cards play the same game until the candidate's
  orders first differ.
- **It sees a change the full-game gate could not:** the same four genes read +1.46 pp
  [−0.05, +2.98] on the human-seat benchmark (2,048 games per arm, z ≈ 1.9, not significant);
  here the reverse change reads z ≈ 4.7 on score and z ≈ 9.6 on the objective. Per card it is
  concentrated: Hollow Swiss T76 92% → 0% met, Swiss tepid T220 96% → 62%, and two cards improve
  with the stance off (Athens break the hold +0.74 SD, Mongols at Chur 16% → 34%), the trade the
  gate exists to show.
- **Smallest effect it detects in about 15 minutes on 16 cores:** a two-arm run gets about 190
  runs per card per arm in 15 minutes even on this loaded machine (the three-arm run did 128 in
  15); at that size the paired MDE (80% power, 5% two-sided) is about **0.02 card SDs on score and
  0.6 pp on objectives met**. On a quiet machine (0.26 CPU-s per run, about 60 runs/s) the same 15
  minutes buy about 600 runs per card per arm, MDE about 0.012 SD and 0.33 pp.
- **The human-seat benchmark in the same 15 minutes:** 2,048 full games per arm; its printed 95%
  half-width is about 1.9 pp, so its MDE is about **2.7 pp** of win rate, against a median change of
  0.65 pp (72 seat-only ledger rows). And those games are not independent (37 of 64 salts of a map
  were the same game on fixed dice, learnings 11-08), so its true MDE is larger.

**What the numbers do not say.** A card SD is not a win rate: 0.02 SD on cards and 2.7 pp of
full-game wins are different units, and nobody has calibrated how a card delta converts into wins
against him. Treat the gate as a sensitive detector of behaviour changes in his positions, with the
full-game benchmark (weekly) as the guardrail that the detected changes add up to winning. The
objective-met column is closer to wins, and its MDE is still 4 times smaller.

**Not independent everywhere:** 8 cards have 1 to 3 distinct Hard scores over 128 seeds (for
example `celts-steal-a-settler` is 150 in every run): the dice do not reach the decision. They add
nothing to the noise and can only move when a candidate changes the play. The pooled interval is
correct for them (their variance is zero), but they dilute the pooled delta: a change that only
touches war cards reads smaller pooled than per card. Read the per-card table.

**The default run, checked end to end:** `tools/sim/scenario-gate.sh --aa` (15 held-out cards,
480 runs per arm, 14,400 card runs) took 1,625 s on this Mac at a load average near 200 (other
agents' sims), and passes: score −0.010 [−0.042, +0.021] card SDs, objectives −0.31 pp
[−0.87, +0.26] (`calibration-1109/aa-held-out-480.md`). On a quiet 16-core machine the same run is
about 4 minutes. On the held-out half alone siege-off reads +0.022 SD [−0.007, +0.052] and
−1.04 pp [−1.84, −0.24] at 128 runs: its large losses are on train cards (Hollow Swiss T76), so the
held-out verdict rests on the objective column. Run `--set train` as well when a change targets a
kind of position.

## Held-out split

Fixed on 2026-11-09 in `split.json` (29 train, 15 held out; `celts-parked-at-st-gallen` added to train 2026-11-10, now 30 train) by `tools/sim/scenario_gate_split.py`:
cards are grouped by kind (Lab, war) and by whose position the seat holds (his empire or a rival of
his), ordered by the SHA-256 of their id, and every third is held out. Tune, search and read
per-card results only on `--set train`; accept on `--set held_out` (the wrapper's default). A card
keeps its set for life; `scenario_gate_split.py --add` assigns a new card by the same rule and
never moves an old one, and `--check` lists cards the split misses. Deleting a card (the Lab's
"scenarios are disposable" rule) shrinks the held-out set, so prefer cutting new cards to deleting
held-out ones while the gate is in use.

## The imitation seat (opponent column)

`legatum_ai::designer` (`crates/ai/src/designer.rs`, genes in `designer.json`) is a
measurement-only brain that plays like the designer, so the gate can ask "does the candidate do
better against him" and not only against Hard's mirror. No app game, difficulty or default uses it.
`--him designer` puts it in his empire (player 0 in every card cut from his games) on the 27
cards where his empire is a rival of the seat; on the other 17 the seat *is* his position and the
column has nothing to change, so the report leaves them out.

**What it is (the simplest thing that works, v2):** Hard with a gene set for his recurring plays
from `docs/playtests/human-reasoning.md` and the coach playbook (early strike on an open neighbour,
follow-ups city by city, paying through Walls for a prize, Settler snipes, focus fire, commitment,
decisive purchase, frontier Walls, fast expansion), plus two order-level habits his tapes show:
Walls first in every city past turn 25, and no routine purchases (only aircraft, and anything for a
city with an enemy army at war within three hexes: "cash saved up to buy walls and a ranger if I
get surprise attacked").

**How well it predicts him** (`crates/sim/examples/designer_predict.rs`: a shadow brain sees his
seat's view of every recorded turn of a tape, on the tape's hash-verified states, and its orders
are compared with his; files in `imitation-1109/`). Training tape Aztecs 8.9.29 (108 turns) and
the held-out tape Celts 8.9.26 (135 turns; a designer-seat held-out tape, never used to build it):

| Held-out Celts 8.9.26: his decisions predicted | Hard | Imitation |
|---|---|---|
| Production (60 orders) | 17 (28%) | **23 (38%)** |
| Founding a city (17) | 12 (70%) | 12 (70%) |
| Military moves to the same tile (67) | 9 (13%; 25% within 2) | 9 (13%; 22% within 2) |
| Settler moves (29) | 2 (6%) | 2 (6%) |
| Research (6) | 2 | 2 |
| War declaration (1), ranged attack (1), purchase (1) | 0 | 0 |
| Purchases it made that he did not | 82 | **56** |
| All his decisions | 42 of 182 (23%) | 48 of 182 (26%) |

On the training tape: production 19% → 36%, purchases he did not make 64 → 12, everything else
equal. **The gene set changes no prediction at all** (the `designer-genes` source predicts exactly
what Hard does on both tapes): only the order-level habits move anything. His tapes are mostly
standing orders (cities on automation, units on paths), so each category is a small sample.

**Is it harder for Hard than Hard is?** Hard in the seat, 27 rival cards, 128 paired runs, his
empire played by Hard and by the imitation (`hard-seat-vs-imitation-v2.md`):

| His empire played by | Seat score, card SDs [95%] | Seat objectives met, pp [95%] |
|---|---|---|
| Imitation v1 (no purchases at all) | +0.047 [+0.018, +0.076] (easier) | +0.46 [−0.06, +0.99] |
| Imitation v2 (emergency purchases kept) | −0.012 [−0.040, +0.016] | −0.75 [−1.32, −0.18] |

v1 was *easier* to beat than Hard (war-india-sparta 59% → 88% met, Swiss tepid T180 +1.3 SD): with
no emergency purchases his Gold never turned into defenders. v2 is about as hard as Hard: slightly
fewer objectives met for the seat, no difference on the 9 held-out rival cards. **Honest verdict:
it imitates two of his habits, not his strength.** He has won his last 10 games against Hard; this
seat does not beat Hard more than Hard does. What it lacks is his decisive tactics (where and when
he strikes, focus and timing of captures), which neither the genes nor the habits express; the
next step is the per-situation imitation the review describes (his recorded orders as candidates in
the positions where they apply).

**The column works as wiring:** siege-off against Hard, on the same 27 cards, reads −0.062 SD and
−4.40 pp with Hard in his empire and −0.057 SD [−0.082, −0.032] and −3.53 pp [−4.15, −2.91] with
the imitation (`siege-off-vs-imitation-v2.md`).

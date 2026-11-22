# Hard Mongols: the designer's mounted horde (2026-11-07)

**Source snapshot:** The 4,096-seat arms, human-seat gate, and Keshig ablation used engine commit `6740948d0`. A later master merge brought RULES_VERSION 19, including removal of Mongol Peace passage. As directed for landing, the merged branch is repinned and reruns deterministic checks; the performance estimates below have not been remeasured on those newer rules.

The 8.9.25 coach review found a plan the AI does not play: the designer fielded nine Keshigs by T80, four upgraded from promoted Horsemen, fired 94 shots, made 17 charges, and captured five cities by riding a Keshig into a city reduced to 0 HP. He lost three military units. In 452 AI-only Mongol seats, Hard fielded 0.7 Keshigs a game, fired 0.03 Keshig shots at cities, and left a Keshig idle in 54% of its contact turns. A screened bonus-only mix switch changed wins by −0.1 percentage points in 900 paired seats. Source: [coach review](../../../playtests/reviews/2026-11-06-8.9.25-mongols.md) and [ideas #47/#48](../ideas.md).

## Intervention

`mongol_horde` is a decision gene, 0–3, shared default 0. It has no effect outside Hard Mongols. Level 1 fills the war quota and early counter with the best buildable mounted unit, keeps melee for a threatened city's garrison, values the Keshig itself, and advances Iron Working after three mounted units. Level 2 upgrades Horsemen into Keshigs with available Gold and spare Iron before the Forge's future yield or the ordinary Gold runway, preserving promotions; a wounded Keshig next to an enemy retreats before an assault can consume it. Level 3 adds the bow siege: Keshigs take firing tiles two hexes from a walled city, shoot while it has HP, and charge to capture at 0 HP. Against an unwalled city they may charge directly. All are AI order choices; combat rules are unchanged.

The probe is [`mongol_horde_probe.rs`](../../../../crates/sim/examples/mongol_horde_probe.rs), adapted from the coach's diagnostic. Each game contains exactly one Mongol seat; the other seven factions are shuffled from the remaining eight. The same seeds and seat assignments are used in every arm. The probe counts completed and bought units, upgrades, battles by attack mode and target, shots at standing Walls, captures by unit kind, losses, contact idles, first Iron Working, and winner faction. [`analyze.py`](analyze.py) verifies paired seed and seat identity, then computes per-seat means, conditional per-faction win shares, the best/worst ratio, and paired normal-approximation 95% intervals for Mongol win-share changes. Games run on AWS spot through [`run-arms.sh`](run-arms.sh) and `tools/cloud/run-batch.sh`; no custom runs root is set. The fixed binary was built from `6740948d0`, SHA-256 `3b85126d59c4c33e1ccfcb3bcf9337de4f4921946e51307b19e29c84bd8682a2`.

## Paired AI-only seats

The held-out level-2 and level-3 batches were relaunched with identical seed, binary, and parameters after their first spot-capacity requests were refused. Their successful batch labels end in `-retry`; [`run-arms.sh`](run-arms.sh) now retries capacity refusals automatically.

| Seed | Level | Mongol win share | Paired change vs 0 | Keshigs fielded | Shots / charges | Captures | Best/worst |
|---|---:|---:|---:|---:|---:|---:|---:|
| 9311003 | 0 | 16.87% (691/4096) | — | 0.766 | 0.652 / 0.575 | 2.680 | 2.289 |
| 9311003 | 1 | 18.21% (746/4096) | +1.34 pp [0.07, 2.61] | 1.457 | 1.364 / 1.350 | 2.866 | 2.674 |
| 9311003 | 2 | 18.26% (748/4096) | +1.39 pp [0.10, 2.68] | 1.546 | 1.422 / 1.484 | 2.893 | 2.545 |
| 9311003 | **3** | **18.38% (753/4096)** | **+1.51 pp [0.21, 2.82]** | 1.535 | 1.484 / 1.369 | 2.860 | 2.429 |
| 8241001 | 0 | 17.48% (716/4096) | — | 0.732 | 0.605 / 0.565 | 2.639 | 2.024 |
| 8241001 | 1 | 19.19% (786/4096) | +1.71 pp [0.41, 3.01] | 1.438 | 1.278 / 1.273 | 2.853 | 2.336 |
| 8241001 | 2 | 19.56% (801/4096) | +2.08 pp [0.78, 3.37] | 1.552 | 1.300 / 1.363 | 2.865 | 2.287 |
| 8241001 | **3** | **19.68% (806/4096)** | **+2.20 pp [0.90, 3.49]** | 1.544 | 1.387 / 1.231 | 2.843 | 2.279 |

Level 3 leads both seeds in Mongol wins and causes the least faction-spread worsening of the enabled levels. Pooled, it wins 1,559/8,192 Mongol seats (19.03%) against 1,407/8,192 (17.18%), a paired +1.86 pp [0.94, 2.77]. The best/worst ratio still rises from 2.289 to 2.429 and from 2.024 to 2.279. That is a balance concern, not evidence to increase Keshig strength. The 95% intervals are descriptive paired normal intervals over the fixed seeds; the level comparison itself is exploratory.

| Seed | Faction | L0 | L1 | L2 | L3 |
|---|---|---:|---:|---:|---:|
| 9311003 | Athens | 14.10% | 13.24% | 13.26% | 13.49% |
| | Aztecs | 13.93% | 14.38% | 14.10% | 13.99% |
| | Byzantium | 11.81% | 11.95% | 12.20% | 11.90% |
| | Celts | 11.83% | 11.38% | 10.94% | 10.91% |
| | Germany | 15.75% | 15.42% | 15.53% | 15.45% |
| | India | 10.24% | 9.94% | 10.13% | 10.38% |
| | Mongols | 16.87% | 18.21% | 18.26% | 18.38% |
| | Sparta | 9.97% | 10.36% | 10.08% | 9.61% |
| | Switzerland | 7.37% | 6.81% | 7.18% | 7.57% |
| 8241001 | Athens | 13.28% | 13.25% | 12.75% | 12.63% |
| | Aztecs | 12.34% | 12.37% | 12.21% | 12.37% |
| | Byzantium | 11.33% | 11.16% | 11.28% | 11.50% |
| | Celts | 10.29% | 11.02% | 10.94% | 11.19% |
| | Germany | 16.80% | 16.47% | 16.19% | 15.75% |
| | India | 10.99% | 10.20% | 10.29% | 9.90% |
| | Mongols | 17.48% | 19.19% | 19.56% | 19.68% |
| | Sparta | 10.55% | 9.54% | 9.65% | 9.74% |
| | Switzerland | 8.64% | 8.21% | 8.55% | 8.64% |

The level 3 arm fields 6.44/6.57 Horsemen and upgrades 0.328/0.335 to Keshigs per seat (primary/held out), compared with 3.05/3.07 Horsemen and 0.172/0.169 upgrades at level 0. It fires 0.142/0.150 shots at standing Walls against 0.018/0.022, and fields 0.748/0.750 Keshigs at T75 against 0.281/0.272. Keshig captures barely change (0.032/0.035 to 0.043/0.046); military losses rise from 16.40/16.12 to 17.57/17.45 as the army fights more. Median Iron Working remains T43 in every arm: its new research priority did not advance the median completion turn. Spearmen fall from 10.54/10.40 to 8.92/8.68 per seat, but remain common through the threatened-city and Gold defense paths. Keshig contact idles remain 50.8%/53.6% at level 3 against 50.8%/51.1%; the horde changes composition and siege decisions, not the general contact-idle behavior. These averages span varied maps and are not directly comparable to the designer's one game, but they plainly do not reproduce his nine-Keshig, 94-shot attack. Further means and exact per-faction shares are in [`summary.json`](summary.json).

## Designer's seat, T2 and T31

The tape and hash-verified state stream from `~/LegatumRuns/coach-tools/coach-8925-mongols/` were copied into a temporary worktree folder for `seat_run`; the live save directory was read only. Forty-eight salts per turn and level compare the current Hard brain from each branch turn to `mongol_horde=1,2,3` under the same dice and AI seeds. The replay's historical pre-branch positions are read from the coach's exported states; the continuation uses this branch's rules.

| Branch turn | Level 0 | Level 1 | Level 2 | Level 3 |
|---|---:|---:|---:|---:|
| T2 seat wins / 48 | 3 | 7 | **9** | 6 |
| T31 seat wins / 48 | 20 | 30 | **31** | 29 |
| T2 final cities (mean) | 4.50 | 6.08 | **6.88** | 6.02 |
| T31 final cities (mean) | 9.65 | 11.90 | 12.12 | **12.27** |

The original coach's stock-Hard baseline from T2 was 4/48 on an earlier master snapshot; this branch's paired control is 3/48. Level 2's paired T31 win change is +22.9 percentage points [4.2, 41.6] by the normal approximation over 48 salts; its T2 change is +12.5 [−1.3, 26.3]. Level 3 did not add seat wins to level 2 on this tape. Receipt: `~/LegatumRuns/mongol-horde-1107/docs/sim-results/ai-quality/mongol-horde-1107/seat-input/runs.jsonl` (384 continuations, 31 historical states hash-verified through T31).

## Rivals' counters

`byzantium-empty-horde` and `celts-stop-the-horde` are rival-seat scenarios, not tuning targets for Mongol decisions. The coach's 50-run baseline had Hard take the target in 0/50 in each; Byzantium lost one of its own cities in 44/50, while Celts declared in 34/50 at median T89. These failures belong to idea #48, the counter playbook.

On this branch, 50 runs per control and level-3 arm reproduce the unmet objective. The Byzantium control loses a starting city in 44/50 and the level-3 arm in 42/50; both take Dyrrachium in 0/50 ([off](scenarios/byzantium-empty-horde-l0/byzantium-empty-horde-baseline.md), [level 3](scenarios/byzantium-empty-horde-l3/byzantium-empty-horde-baseline.md)). The Celtic control and level-3 arm are identical: both fail to take Almaliq in 50/50, declare in 23/50 at median T90, and lose a starting city in 2/50 ([off](scenarios/celts-stop-the-horde-l0/celts-stop-the-horde-baseline.md), [level 3](scenarios/celts-stop-the-horde-l3/celts-stop-the-horde-baseline.md)). The Celtic declaration rate differs from the earlier coach snapshot, so the within-branch pairing is the relevant comparison. Only these small reports were retained; the generated per-run JSON and logs were not committed.

## Human-seat gate and Keshig ablation

The selected level was checked in `hard_vs_human --arms` seat-only and rivals-only arms (256 paired salts × 8 maps, seed 310001, reseeded dice, one binary built from `6740948d0`). [`human-arms.json`](human-arms.json) sets the control gene to 0 in every seat. The [paired quality-gate comparison](human-gate.json) uses `tools/sim/quality_gate.py::compare` with a two-arm family and a −1.5 pp noninferiority floor.

| Arm | Human seat wins / 2,048 | Change vs gene off, pp [95%] | Gate |
|---|---:|---:|---|
| Gene off, all seats | 472 | — | control |
| Level 3, rivals only | 469 | −0.15 [−1.04, +0.75] | noninferior |
| Level 3, player seat only | **502** | **+1.46 [+0.69, +2.23]** | noninferior |

The seat-only gain is entirely on the Mongol map, 112 to 142 wins of 256 salts. The rivals arm changes two maps (Swiss: 15 to 7 human wins; Mongol: 112 to 117), with a net near zero. This supports the Mongol Hard builtin at level 3, while leaving the shared gene default 0. Batch: `~/LegatumRuns/cloud/mongol-horde-human-1107-043338`.

The paired [`mongols_keshig` re-ablation](ablation.md) uses level 3 in both arms, 4,096 Hard games, 3,655 paired Mongol seats, and switches off only the faction rule. Mongol share is 18.03% with the Keshig to 17.95% with an ordinary, cheaper Cataphract: −0.08 pp [−1.23, +1.18], classified **dead** at this sample size. The bow and its price largely cancel in this census; this is not a demonstration that the bow is useless in the designer's concentrated army. A subsequent balance experiment can vary price or Iron demand, but this branch changes no Keshig rule.

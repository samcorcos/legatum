# Placement by neighbour, 2026-11-07

**Status:** `site_denial` 100 is on for Hard (branch `claude/settle-strategy`); `site_aggressor_risk` and `site_backfill` were measured and then removed from the code (git history, `0db34c07b`, keeps them). The sections below describe them as tested.

The designer's own account of how he places cities is in `docs/playtests/human-reasoning.md` (2026-11-07). He values a site by its yields. He also weighs the risk from a specific aggressive neighbour: he won't settle toward Sparta unless an army goes with the Settler, Walls can follow fast, or Sparta is busy in another war. He also values denial: Biel and Bern cut India off from land and Niter it needed later. The earlier attempts were flat. `land_contest` credited contested land toward every neighbour, and `settler_safety` applied the same safety rule everywhere. Here each term depends on the specific neighbour.

## Mechanism (`crates/ai/src/site_strategy.rs`, Hard only, valuation only)

Each term adjusts the value `civilians.rs::choose_site_scored` gives a preferred site, after `land_contest_value`. None applies to the first city.

- **`site_aggressor_risk` N:** the site loses N% of a full loss. The full loss is 250% of the site's value: the Settler and city, the same city in the rival's hands, and the war it invites. That loss is weighted by the worst neighbour's chance (0 to 90%) of taking the city before its Walls rise.
  - **What raises the chance:** the neighbour's aggression, the sum of:
    - a public record: 20 per war declared, 15 per recent capture, 5 per older capture, 25 per elimination, and 20 for a recent surprise war;
    - a weak prior: its faction's aggression knob divided by 4, or 15 for a human.

    An empire at war with us counts 100. The chance then grows with closeness to its nearest known city (100% at 4 hexes, 25% at 9 to 10, or 25% further out if its army is in sight), with its era lead and Strongest Army lead, and with the Walls window.
  - **What lowers it:** our troops within 3 hexes of the site or beside the Settler (a third of the chance when they out-power its visible army within 8 hexes), Gold to buy a defender (two thirds), and the neighbour being at war with someone else (half).
  - **Walls cannot be bought.** `economy::is_unbuyable` covers every Walls tier, so "Gold for Walls" becomes Gold for a defender.
- **`site_denial` N:** inside the nearest rival's Settler reach (twice its city gap plus 2), a site gains up to N% of its value, scaled by closeness to that rival. Two things count:
  - the "lane": open land within two city gaps of the site that is farther from the rival than the site and nearer the site than the rival (18 tiles is a full lane);
  - unowned strategic deposits within the work radius, 30% each, at most 2.

  The aggressor risk prices the extra exposure, so the trade-off is the sum of the two terms.
- **`site_backfill` N:** a site loses N% of its value now when it meets all three conditions:
  - it is out of every known rival's reach plus 2;
  - all land within 4 hexes is explored;
  - it is nearer our cities than any visible rival troop.

## Gates

### His two Swiss seats (Hard plays his seat from his recorded position)

The 8.9.24 tape and states are the swiss-gap copies. The 8.9.26 win save was copied to `~/LegatumRuns/settle-strategy-1107/s8926/` (SHA-256 `7db209a1…`) and replayed on its recording engine `514dbb8f`: 144 hash-verified states. The runs used 48 paired salts per branch turn, branch turns T2/T30/T40, and `seat-run.sh` from aztecs-playbook-1105. Full tables: [seat-v2.md](seat-v2.md). The v1 table ([seat-v1.md](seat-v1.md)) used a stronger temperament prior (knob/2).

| Arm (pooled over 2 maps x 3 branch turns, 288 runs) | Seat wins | Paired +/- vs control | Cities lost within 30 turns of founding, per run | Hard sites within 1 hex of one of his | His later sites Hard also founds |
|---|---:|---:|---:|---:|---:|
| control | 83 | - | 0.92 | 70.1% | 41.3% |
| `site_denial` 100 | 86 | +42/-39 | 0.82 | 65.1% | 40.2% |
| `site_aggressor_risk` 50 | 64 | +27/-46 | 0.86 | 68.6% | 40.1% |
| `site_aggressor_risk` 100 | 73 | +34/-44 | 0.75 | 61.5% | 34.8% |
| risk 50 + denial 100 | 74 | +32/-41 | 0.88 | 69.4% | 40.8% |
| `site_backfill` 50 (v1) | 82 | +11/-12 | 0.99 | - | - |

- The risk gene does what it says: fewer young cities are lost at 100 (0.92 to 0.75). But the seat wins less, and Hard founds farther from where he founded. Settling away from Sparta costs land and tempo that the safety does not repay.
- On 8.9.26 he settled toward India (Biel, Bern), so a risk penalty moves Hard away from his sites. At T30 on 8.9.26, his sites fall from 90% to 60% of Hard's foundings at risk 100. The decision diff measures how close Hard's choices are to his, not whether the gene is right.
- Denial leaves the seat and the agreement with his sites roughly where they were.

### AI-only census (4,096 eight-player Hard games per arm, gene in every seat, paired by seed)

[census-*.json](.) from `census.py`; win share = wins / seats.

| Arm | Seed | Best/worst | Swiss | Young cities lost per seat | Paired young losses per game |
|---|---|---|---:|---:|---:|
| control | 9311003 | Mongols 17.67 / Swiss 7.06 = 2.50 | 7.06% | 0.633 | - |
| `site_denial` 100 | 9311003 | 17.78 / 7.17 = 2.48 | 7.17% | 0.626 | -0.05 [-0.17, +0.06] |
| `site_aggressor_risk` 100 | 9311003 | 16.83 / 7.78 = 2.16 | 7.78% | 0.658 | +0.20 [+0.08, +0.32] |
| control | 8241001 (held out) | Mongols 18.12 / Swiss 7.05 = 2.57 | 7.05% | 0.652 | - |
| `site_denial` 100 | 8241001 | 17.93 / 8.32 = 2.16 | 8.32% (+1.27 pp [+0.16, +2.38]) | 0.622 | -0.24 [-0.35, -0.12] |
| `site_aggressor_risk` 100 | 8241001 | 18.09 / 7.85 = 2.30 | 7.85% | 0.651 | -0.00 [-0.12, +0.12] |

No faction's paired change is outside noise on both seeds. Celts rose 1.40 [0.09, 2.71] with denial on the main seed and were unchanged held out. 0 panics in all six batches.

### Human-seat gate (8 maps x 256 salts, independent dice, seed 310001, one binary)

[human-seat-v2.json](human-seat-v2.json), [human-seat-v3.json](human-seat-v3.json) from `gate.py`. The control won 505 of 2,048 games. A lower seat win rate in a rivals arm means harder rivals.

| Candidate | Rivals-only seat wins (pp) | Seat-only seat wins (pp) |
|---|---|---|
| `site_denial` 100 | 497, -0.39 [-2.47, +1.69] | 526, +1.03 [-0.77, +2.82] |
| `site_denial` 200 | 487, -0.88 [-2.96, +1.20] | 507, +0.10 [-1.79, +1.98] |
| `site_aggressor_risk` 100 | 452, -2.59 [-4.73, -0.45] (harder) | 415, -4.39 [-6.23, -2.55] (worse) |
| `site_aggressor_risk` 25 | 494, -0.54 [-2.35, +1.27] | 504, -0.05 [-1.59, +1.49] |
| risk 25 + denial 100 | 506, +0.05 [-2.01, +2.10] | 523, +0.88 [-0.96, +2.72] |

## Decision

- **`site_denial` 100 on for Hard.**
  - The seat is noninferior: +1.03 pp, lower bound -0.77 against the -1.5 floor.
  - The rivals are not softer: -0.39 harder, an inconclusive interval.
  - The census ratio improved on both seeds (2.50 to 2.48 and 2.57 to 2.16). Swiss rose on both (+0.11, +1.27).
  - Cities lost young fell, clearly on the held-out seed.
  - His seats were neutral.
- **`site_aggressor_risk` off.**
  - At 100 it made rivals harder in the benchmark, but Hard in the seat lost 4.4 pp and won less on his own maps.
  - At 25 it did nothing measurable. The safety it buys does not repay the land it gives up, as the flat `settler_safety` found.
  - Removed from the code: off by default with no gain is dead code.
- **`site_backfill` off, then removed from the code.** Its conditions (no rival in reach, every tile within 4 explored) rarely hold while Settlers are still choosing. On his seats at 50 it changed almost nothing.

## What is left

- The risk gene trades land for safety at a bad rate. A version that only defers the risky site, founding it later behind an escort or Walls, instead of discounting it now, is untested. The designer's exceptions are "an army goes with it" and "once I can defend it". That suggests timing and escorts, not avoidance.
- Information value (his hill lookouts at chokepoints) is not built.
- Agreement with his founding sites did not rise with any gene. The metric is coarse, because his sites are dense on both maps, and both maps are Swiss.

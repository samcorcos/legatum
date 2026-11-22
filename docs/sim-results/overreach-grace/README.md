# Crown upkeep grace (2026-10-11)

Rule (user, 2026-10-11): a crown costs upkeep only once its current holder has held it for more than 30 turns, and only while the holder is above the median living empire (the decadence test). Decadence is unchanged (it needs a full 30-turn Power window). `crown-upkeep-after=0` restores the old charge from the turn after a crown is won (the control; the median test stays on). The AI gene `crown_free_aware` (default 1) is the AI's knowledge of the free turns; 0 judges every crown by its full eventual cost, as before.

Binary: the branch `claude/overreach-grace`; master control: `a98efd68` built separately. All on AWS spot.

## Human-seat benchmark (hard_vs_human, brain hard, 256 salts x 8 maps = 2,048 games per arm)

| Arm | Rules | Seat AI | Rival AI | Seat win rate |
|---|---|---|---|---|
| M | master | master | master | 26.8% (24.9-28.5) |
| A | upkeep from turn after win, median test | unaware | unaware | 26.1% |
| A' | new | unaware | unaware | 26.5% |
| B | new | aware | aware | 27.3% (25.4-29.0) |
| Xs | new | unaware | aware | 26.6% (seat-only arm) |
| Xo | new | aware | unaware | 25.2% (opponent-only arm) |

Paired, salt bootstrap: B minus M +0.5 pp (95% -2.1 to +3.0); A' minus M -0.3 (-2.8 to +2.1). Seat awareness (B minus Xs) +0.7 pp; rival awareness (B minus Xo) +2.1 pp, i.e. aware rivals did not become harder. Nothing exceeds noise. Hard + Search, 64 salts: M 21.7% vs B 26.2% (+4.5 pp, -0.2 to +9.0). Per map (hard, M to B): Switzerland 8.3 2 to 0, Sparta 8.4 49 to 54, Japan 41 to 37, Celts 38 to 35, Mongols 1 to 1, Switzerland 8.5 45 to 55, Germany loss 24 to 14, Sparta 8.6 16 to 21 (per-map swings of 5 to 10 points exist; the combined number does not move).

## VP race (Hard + Search, 256 games each at 4 and 8 empires, pooled)

| | master (overreach v3 alone) | grace |
|---|---|---|
| T100 leader wins | 47.7% | 50.9% |
| T50 leader wins | 49.5% | 47.7% |
| VP lead changes per game | 11.4 | 9.7 |
| Samples with one team holding all 3 crowns | 3.0% | 8.1% |
| Crown flips per game | 55.6 | 43.7 |
| Leader's overreach, Gold per turn / share of its city Gold | 9.8 / 35.8% | 4.4 / 16.8% |
| Units deserted for unpaid overreach per game | 4.5 | 1.9 |
| Median game length | 255 | 251 |

The rubber band is weaker: crowns stick with the leader (fewer flips, more all-crown samples) and the race has about 15% fewer lead changes. Top Power over the median at T150 rose from 29.9% to 34.7%. This is the price of free crowns and is within what the user chose.

## What still bites early (grace, mean per game at the end of the stated turn)

| Turn | Empires paying crown upkeep (of 4 / of 8) | Their upkeep as share of their city Gold | Empires with decadence | Mean / max decadence |
|---|---|---|---|---|
| 10, 20 | 0 / 0 | - | 0 / 0 | - |
| 30 (applies from turn 31) | 0 / 0 | - | 1.9 / 3.6 | 8.5% / 26% |
| 31 | 0.13 / 0.05 | 53% / 45% | 1.9 / 3.6 | 8.5% / 26% |
| 40 | 0.36 / 0.20 | 58% / 44% | 1.9 / 3.7 | 9.6% / 28% |
| 60 | 0.58 / 0.30 | 46% / 34% | 1.9 / 3.7 | 10.3% / 31% |

Nothing penalises a lead before turn 31. From turn 31 decadence starts at once at its full rate (the top empire goes from 0 to about 26% in one turn; it then stays flat, no later spike), and a crown held since the opening is charged from turn 31 for an empire above the median.

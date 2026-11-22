# Siege forecast calibration and stance guards (2026-11-08): on by default

Branch `claude/siege-forecast-calibration`, cut from `claude/hard-breach`. Step 1 after increment 3: calibrate `forecast_siege` against the combat lab.

## What was wrong
`opportunity.rs::forecast_siege_rounds` had every enemy unit attack the force every round, the besieged city's own garrison included: a melee garrison charged out at the weakest attacker. Traced on the hills cell, the garrison took about 25 HP a round off one attacker, so a three-Soldier storm the search wins 7 times in 8 came out as "no capture". In the lab, Hard garrisons made **0 melee sorties in 403 siege turns** (42 fights, plain Hard; field defenders made 2 melee attacks in total).

## Change (gene `forecast_garrison_holds`; first measured off, now on at 1, see the end)
- With the gene on, the besieged city's garrison holds the city and only shoots, if it can.
- With it on, `siege_stance`'s storm also brings the force units that can't reach the city this turn to their assault tiles, unless an enemy field unit is within two tiles of them.

## Gates

| Arm | Lab (21 × 8) | Lab, held-out dice (seeds 2000+, × 16) | War suite (34 × 32, seat) | Designer seat, 96 salts, all [95%] |
|---|---|---|---|---|
| Control | 29.8% | 28.9% | 45% | 45.8% |
| `forecast_garrison_holds` 1 | 29.8% (no fight changes) | | **+2.1**: Tepid T220 62 → 96%, T220 fresh 62 → 100%; none worse | −0.3 [−2.7, +2.1]; held out +0.8 [−2.8, +4.4] |
| `siege_stance` 6 alone | 34.5% | | | |
| **`forecast_garrison_holds` 1 + `siege_stance` 6** | **41.7%** (24 fights only the candidate took, 4 only plain Hard took; Production lost 283 → 308) | **41.1%** (44 against 3) | **+5.1**: Hollow Swiss T76 0 → 100%, both Tepid T220s 62 → 100%; none significantly worse; worst Celts wild hold −9.4 [−21.9, +3.1] | −0.1 [−2.5, +2.3]; training +0.5, held out −0.9 [−4.5, +2.7]; Sparta 8.9.26 +5.2 [+0.1, +10.3], Mongols 8.9.25 −4.2 [−10.4, +2.0], India −2.5 |

The control for the designer seat is the 96-salt control at `16dd54e8c`. With these genes off the code takes the same path, so the control carries over.

- The lab passes 38% only with both genes. Garrison-holds alone changes no lab fight (Hard's own passes use other forecasts), and the longer window alone stays at 34.5%. Window 8 gives the same 41.7% as window 6.
- **The designer seat doesn't move** for either arm. Following the coordinator's rule, `siege_stance` 4 wasn't re-tested on top, and every default stayed 0 at this point. Later the same day the guarded arm below was gated and shipped on (see its section); `breach` stays 0.

## Non-inferiority check: 200 salts and the human seat

The coordinator's ship rule for the combined arm: the all-tapes lower bound must be at least −2.0, the held-out lower bound at least −3.0, and the human seat-only result must not be significantly worse.

| Measure | Control | Combined arm | Delta [95%] | Bar |
|---|---:|---:|---|---|
| Designer seat, all tapes (7,000 runs a side) | 45.8% | 45.6% | −0.1 [−1.8, +1.5] | pass |
| Training | 37.0% | 37.9% | +0.9 [−1.2, +3.0] | |
| **Held out** | 57.4% | 55.9% | **−1.5 [−4.0, +1.0]** | **fail** (needs ≥ −3.0) |
| Mongols 8.9.25 (held out) | 42.1% | 38.4% | −3.7 [−8.0, +0.6] | |
| India 8.9.25 | 13.8% | 11.4% | −2.4 [−5.3, +0.5] | |
| Sparta 8.9.26 | 76.5% | 80.8% | +4.3 [+0.7, +7.9] | |
| Human seat, seat only (8 maps × 256) | 22.75% | 24.12% | +1.37 [−0.34, +3.08] | pass |
| Human seat, rivals only | 22.75% | 20.90% | −1.86 [−3.71, +0.05] (rivals harder) | |

The 200-salt control is a fresh run at `f22b7bf69` with the genes off. **The held-out bar fails, so the defaults stay 0.** The drop is concentrated in Mongols 8.9.25 and India 8.9.25, the same two tapes that were negative at 96 salts.


## The cost on Mongols 8.9.25 and India 8.9.25, traced

The same two seeds and turns, seat only (`seat_run --continue hard,params:FILE`, 200 salts, paired on the dice). Here only the designer's seat plays the genes, so rivals getting stronger can't explain it: the seat itself got worse. Files: `trace-*.jsonl.gz`.

| Start | Ungated arm | Mechanism | Guard | Guarded arm |
|---|---|---|---|---|
| Mongols 8.9.25 T60 (63 of the 98 losses) | −22.0 [−30.4, −13.6] | **Horde overcommit away from a threatened home.** At T64 the Cataphracts left the front at (30–32, 11–15) to storm a city, one died at T65, and a young city behind them fell at T66 (salt 13, traced). | `stance_home_guard` N: units within N hexes of an own city under real threat (more than half its strength, as `siege_war` reads it) stay out of the stance, unless the city is also within N of that home | g3, not target-aware: +1.0. g6, target-aware: −11.0 [−18.8, −3.2] (still a cost; see below) |
| India 8.9.25 T80 (44 of the 103 losses) | −12.5 [−19.2, −5.8] | **The calibrated forecast leaked into the plan layer and turned a dead war into a pressing one.** With the control, Hard signed Peace with team 1 at T84 ("dead war" +30). With the arm, "truce forgoes pressing" −25 replaced it, Hard refused, kept attacking at (20, 9) and lost 3 of 6 units by T88 (salt 4, traced). `forecast_garrison_holds` alone: −12.5. `siege_stance` 6 alone: −4.5, not significant. | `forecast_garrison_holds` 1 is now stance-only (2 keeps the old reach); `stance_plan_war` 1: the stance only takes on the plan's Campaign or Contain enemy, or the campaign or conquest city | −3.0 [−9.9, +3.9] (g6 not target-aware); +1.5 [−7.2, +10.2] (g6 target-aware) |

The home guard has a trade-off. Without the target-aware exception, any radius 2 to 6 cost Hollow Swiss T76 (100% → 3%) and the war suite fell to +0.2 to +2.4. With it, the war suite keeps +5.6, India is fixed, and Mongols T60 keeps some of its cost.

## The guarded arm (now the default): `forecast_garrison_holds` 1, `siege_stance` 6, `stance_home_guard` 6, `stance_plan_war` 1

| Gate | Result | Ship rule |
|---|---|---|
| Combat lab | 29.8% → **41.7%** (24 fights only the candidate took, 4 only plain Hard took) | |
| War suite (34 × 32, seat) | **+5.6**: Hollow Swiss T76 0 → 100%, both Tepid T220s 62 → 100%, Mongol bombers +9.4; none significantly worse; worst Celts wild hold −3.1 [−9.4, 0.0] | |
| Designer seat, 200 salts, all tapes | 45.8% → 47.0%, **+1.2 [−0.5, +2.9]** | lower bound ≥ −2.0: pass |
| Held out | 57.4% → 58.0%, **+0.5 [−2.0, +3.0]** | lower bound ≥ −3.0: pass |
| Training | 37.0% → 38.7%, +1.7 [−0.4, +3.8] | |
| By tape | Sparta 8.9.26 +4.2 [+0.6, +7.8], Germany 8.9.27 +2.5 [+0.3, +4.7], Mongols 8.9.25 −2.1 [−6.4, +2.2], India 8.9.25 −1.4 [−4.4, +1.6] | |
| Human seat, seat only | 22.75% → 24.22%, **+1.46 [−0.05, +2.98]** | not significantly worse: pass |
| Human seat, rivals only | −1.86 [−3.61, −0.10]: the rivals got harder | |

The designer-seat control is the 200-salt run at `f22b7bf69` (genes off; the code path is the same). The human-seat control is the calibration round's control batch, same seeds. **Everything passes, so the four genes are on by default** (`breach` stays 0).

## Reading
The lab and the war suite agree: Hard was leaving winnable storms on the table because its forecast priced a garrison sortie that Hard defenders never make. Taken alone, the calibration and a longer stance window cost the designer seat on two tapes, and the traces show two different mechanisms: horde overcommit away from a threatened home, and a stance keeping a dead war going. Two narrow guards and a stance-only scope for the calibration remove most of that cost. The guarded arm passes the non-inferiority rule, and the designer seat even rises (+1.2). Mongols 8.9.25 still reads −2.1 (not significant). Most of that is the T60 start, where the target-aware home guard lets the horde storm a city 4 to 6 hexes from a threatened young city.

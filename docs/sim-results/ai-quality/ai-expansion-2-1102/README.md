# AI expansion, round 2 (2026-11-02, branch `claude/ai-expansion-2`)

Round 1 (`../ai-expansion-1102/`) left Hard on the designer's 8.9.23 Germany map at a 3rd and 4th city by T30 and T47 and 5.2 cities at T81 (his: T22, T31 and 9). This round looked for the next binding constraint with `expansion_probe` (Hard in every seat of his save, 64 to 128 reseeded games per arm) and found that decision rules are no longer the main limit.

## What the probe shows

- **Hard's own Settler pace now matches his to T31.** The capital starts its first Settler at T6 and keeps a chain (55% of its Production-turns to T60 are Settlers). His own Settlers founded at T22 and T31; his 2nd city (T19) was Byzantium's Settler, taken in a T16 surprise war.
- **Lifting every expansion gate at once barely helps.** The arm with `settler_priority` 4000, `settler_pipeline` 4, `settler_open_cities` 8, a site floor of 20, reach 14 and `colony_compound` 300 reaches 5.72 cities at T81 (base 5.20). Single levers: pipeline 3 +0.16 to +0.24, `settler_open_cities` 8 -0.03, site floor 20 -0.03, reach 14 -0.11, `military_scout` 100 +0.08, 300 -0.03, `settler_priority` 2000 +0.03.
- **The land is the limit.** At T40, T60 and T80 about 7 good legal sites lie within 8 hexes of Hard's cities in the true state, but its view knows 5.6, 4.2 and 3.4. 60 to 78 land tiles there are unexplored, all reachable, a third of the hidden site tiles near a barbarian camp. Its Scouts die out (1.2 alive at T40, 0.4 at T80). Games where the home land is explored end with only 0.35 more cities at T81. In his game Byzantium stayed on one city; in Hard's games it founds 4 more by T81, and 4 of his 9 cities lie on the land toward Byzantium.
- **Economy per city is not behind.** Hard: Production 54 at T50 and 93 at T81 on 3.6 and 5.3 cities. Him: 73 on 4 and 141 on 9. Hard improves few tiles (1.8 at T50, 3.7 at T81; he queued 14 improvements by T81) but out-produces him per city at T81 (17.6 against 15.7). Gold is not his method: he held 475 and spent little.
- Mid-game Settlers wait: 16% of Settler-turns at T41 to T60 and 27% at T61 to T81 have no site (the production gate counts sites the site search later drops). In 64 games, Settlers stood in view next to a seat unit at Peace on 18 turns; 6 were taken.

## Tried and rejected

| Change | Result on his map | Decision |
|---|---|---|
| `home_scout`: frontier near home weighs revealed tiles more; a missing Scout is worth 15 per unknown home tile | T81 5.20 to 4.56, fog unchanged | Reverted (not on the branch) |
| `garrison_by_risk`: skip the troop-per-city quota where nothing threatens | No change (army at T50 unchanged; the quota rarely bound) | Reverted |
| `settler_slow_turns` 12: another city's Settler 12+ turns from done leaves the gate's pipeline count | T81 +0.05, T100 +0.07 (128 games); with `settler_pipeline` 3 +0.21 / +0.20 | Gene kept, **default 0** after the gate below |

## Human-seat gate

`hard_vs_human`, seed 310001, 256 reseeded salts x 8 maps (2,048 games per arm), plain Hard rivals, one spot server (0.30 server-hours), binary at `dff52577`. Control: round 1's genes on, `settler_slow_turns` 0, `settler_pipeline` 2 in every seat. `_riv` and `_seat` set the other seats to the control through `other_params`. Delta: change in the designer's-seat win rate ([compare.py](compare.py), [comparison.json](comparison.json), [arms.json](arms.json)); batch `~/LegatumRuns/cloud/aiexp2-gate-1102-140436`.

| Arm | Seat wins | Delta pp [95%] | Rival cities T50 / T100 | Rival Settlers lost per game |
|---|---:|---:|---:|---:|
| ctl | 425 | | 3.06 / 5.32 | 0.50 |
| slow, rivals only | 457 | +1.56 [-0.15, +3.22] | 3.07 / 5.41 | 0.56 |
| slow, all seats | 463 | +1.86 [+0.10, +3.66] | 3.07 / 5.41 | 0.56 |
| slow + pipeline 3, rivals only | 460 | +1.71 [-0.24, +3.71] | 3.08 / 5.53 | 0.57 |
| slow + pipeline 3, all seats | 439 | +0.68 [-1.22, +2.59] | 3.08 / 5.54 | 0.56 |
| slow + pipeline 3, seat only | 405 | -0.98 [-2.69, +0.73] | 3.06 / 5.33 | 0.53 |

More Settlers from more cities made the rivals found more cities and still lose more to the designer's seat, and made Hard in the seat weaker. Neither ships.

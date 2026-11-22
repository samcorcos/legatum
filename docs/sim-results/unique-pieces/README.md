# Unique pieces that differ: Hoplite phalanx and Imperial Scriptorium (2026-10-17/18)

Branch `claude/unique-pieces`. Rules: a Spartan Hoplite gets +5 strength next to another Hoplite of its empire (`data::HOPLITE_PHALANX_STRENGTH`); Byzantium's Imperial Scriptorium makes techs a rival knows 10 percentage points cheaper, inside the 50% diffusion cap (`data::SCRIPTORIUM_DIFFUSION`). AI genes `phalanx_formation` and `scriptorium_value` (default 1) let the benchmark separate the rules from the AI's use of them.

## Choosing +5

`combat_scenarios --phalanx` (`phalanx-scan.txt`): the Spartan line against other armies for strengths 0, 3, 4, 5, 6 and 8. Only one row is not saturated by Spartan wound immunity and kill healing: three Hoplites against three Phalangites (one era ahead), 44% at +0, 51% at +3, 53% at +4 and +5, 55% at +6, 56% at +8 (Hard policy). A Hoplite is 22 with its promotion level and a Phalangite 26, so +5 makes a pair 27, level with the next era's melee unit. Pairs against plain Spearmen did not move beyond noise, and two Hoplites beat one Composite Archer 96 to 98% at every strength, so the "counter-class unit one era ahead" check cannot be made in an arena.

## Human-seat benchmark (hard_vs_human, brain hard, 128 salts x 8 maps = 1,024 games per arm; AWS spot)

Control is the master binary of the same day (`ed5960428`); the candidate adds the rules and both genes. Arms are `arms.json` (the genes off for "unaware"). Win rate of the recorded human seat:

| Arm | Rules | Seat AI | Rival AI | Seat win rate | vs master (paired 95%) |
|---|---|---|---|---|---|
| Master | old | master | master | 34.9% | |
| Unaware | new | unaware | unaware | 37.1% | +2.2 (-0.4 to +4.9) |
| Seat-only | new | aware | unaware | 37.9% | +3.0 (+0.7 to +5.5) |
| Opponents-only | new | unaware | aware | 37.7% | +2.8 (+0.2 to +5.5) |
| Both aware | new | aware | aware | 38.7% | +3.8 (+1.4 to +6.2) |

Seat-only (aware seat minus unaware seat, rivals unaware) is +0.8 pp: the AI uses the rules without losing. Opponents-only is +0.6 pp for the seat, so aware rivals did not become harder. The rules themselves help the seat, mostly on the Sparta maps.

The first formation rule ranked staging and assault spots next to a Hoplite ahead of terrain and distance. Against the merged master it cut the 8.4 Sparta seat from 116 to 64 wins of 128 (seat-only 31.7%, -3.1 pp, CI -6.1 to -0.2). The shipped version is a tie-break after distance and exposure and recovers it (117 of 128). The earlier run against the older master had shown only -5% on that map: re-measure an AI change after every merge of master.

## Faction win share (8 Hard AIs, 4,096 games each, seed 2027000001, fair share 12.5%)

| Faction | Master | Candidate |
|---|---|---|
| Sparta (Sparta) | 16.4% | 16.0% |
| Byzantium (Byzantium) | 20.7% | 22.1% |
| Others (Switzerland, Japan, Germany, Mongols, Athens, Celts) | 12.7, 2.6, 10.6, 11.1, 16.7, 9.3 | 12.4, 2.1, 10.8, 10.8, 16.8, 8.8 |

Sparta moves -0.4 pp (noise). Byzantium moves +1.5 pp (about 2.3 standard errors), from a faction already above fair share. Balance does not gate landings while the AI is weak (user, 2026-10-04); the numbers are for the record.

## Cost

Five hard_vs_human batches and ten 8-player batches on one c8g.16xlarge spot server each: about 5 server-hours, under $4.

# Attack together, finish the fallen (2026-11-03, branch `claude/fix-joint-assault`)

From the 8.9.25 Celts coach review (`docs/playtests/reviews/2026-11-03-8.9.25-celts.md`, ideas #42): the Aztecs, with four times his army, walked about 15 units one or two at a time into his walled towns over 50 turns at War, and twice a lone wounded Mongol unit left a Celtic city at 0 HP with nobody to finish it.

## What Hard did, and why

`decision_audit --diff-only --army-detail` on his tape (the AI seats reproduce the recording: 0 hash and 0 strategic-order mismatches), Aztec seat T78-T140 and Mongol seat T86-T158, gives the planning pass behind every unit order.

- **The trickle is the siege pass.** Of the 45 Aztec moves that ended within two hexes of a Celtic city, 35 came from `siege.rs::siege_war` (23 from the early pass, 12 from the late one); the rest were counterattacks (4), the campaign pass (3) and the opportunity strike (2). `siege_city` sends every free unit within `siege_reach` (8) to a firing tile (shooters) or a two-hex screen tile (melee), accepting the city's strike so long as the unit survives its strongest roll. The far units arrive a turn or two after the near ones, one at a time, and the city plus one Marksman or Arquebusier in Lugdunum focus each one down in 1 to 3 turns (the review's table: 5 to 9 Aztec units within five hexes of Lugdunum, 0 to 2 within two).
- **Colony 7 at 0 HP (T92) was abandoned, not unreachable.** On T93 the Mongol campaign was withdrawn (`withdraw_failed_campaign`, the second military pass) while two full-HP Rangers stood two hexes from the city and the Cataphract beside it; the Marksman inside sat at 61 HP after two shots. `retake_weakened_city` runs later and needs a home city within 6 and a charger at 50+ HP.
- **Colony 10 at 0 HP (T153) was unreachable.** One Cavalry at 16 HP brought it down; the next Mongol unit was four hexes away holding its own city, and the Marksman inside was at full HP.

## Genes (`crates/ai/src/params.rs`)

- **`finish_fallen` 50 (on).** `military.rs::finish_fallen_cities`, run at the start of `withdraw_failed_campaign`: a visible enemy city at war at or below this percent of its HP, with a free unit of ours within four hexes, gets the city-assault coordinator (`city_assault.rs::coordinate_city_assaults_on`, its multi-turn capture forecast) before any withdrawal takes the units, whatever campaign is running.
- **`joint_assault` 0 (off; 1 or 2).** `siege.rs::siege_city` and `military.rs::press_strike`: against a city that strikes (Walls, HP left), the units that can be in range this turn (within one move) step in only when `opportunity.rs::forecast_siege` of that group on its tiles, with our Bombers in reach, takes the city; otherwise every planned unit stages three or four hexes out (a safe tile, else the nearest free one outside every walled city's range). Siege tiles one move away are preferred and tiles in another walled city's range are refused. 2 also sends a force in when every planned unit steps in on this turn, whatever the forecast.
- **`joint_window` 0.** With `joint_assault`, the rounds its forecast plays (0: the siege forecast's window). 6 and 8 changed nothing in his game.

## Gate 1: his game (32 runs a row; `crates/sim/examples/joint_assault_gate.rs`, [gate1-config.json](gate1-config.json), [gate1-results.json](gate1-results.json), [summarize.py](summarize.py))

Branches from the recorded state with every AI's memory restored, as the coach runner does. Hard seat: Hard in his seat, dice reseeded per salt. Tape seat: his recorded orders, AI seeds salted (75 of his orders fail per run as the game drifts, the same in every arm). Per run, against his cities: a *lone exposure* is a unit standing in a walled city's strike range at the start of a turn with no other unit of its owner there; a *lone hit* is a city strike on such a unit.

| T83-T133, Aztecs | Lone exposures | Lone hits | Units lost within 2 | Hits on his cities | His cities taken |
|---|---:|---:|---:|---:|---:|
| Hard seat, control | 7.22 | 7.22 | 4.62 | 1.53 | 0.06 |
| Hard seat, `finish_fallen` 50 | 7.50 | 7.53 | 4.69 | 1.56 | 0.06 |
| Hard seat, + `joint_assault` 2 | 5.53 | 5.53 | 3.28 | 0.81 | 0.03 |
| Tape seat, control | 25.78 | 20.72 | 10.12 | 30.72 | 2.94 |
| Tape seat, `finish_fallen` 50 | 25.78 | 20.72 | 10.12 | 30.72 | 2.94 |
| Tape seat, + `joint_assault` 2 | 17.25 | 12.16 | 6.69 | 14.28 | 1.16 |

`joint_assault` 1 read 5.47 / 3.22 losses (Hard seat) and 19.94 / 15.81 / 6.62 / 1.66 taken (tape seat).

| 0-HP cities | Control | `finish_fallen` 50 |
|---|---|---|
| Colony 7 from T93, Hard in his seat | taken 13 of 32 by T97 | **taken 32 of 32**, on T93 |
| Colony 7 from T93, his orders | 0 of 32 (2 Mongol losses) | 0 of 32 (3 Mongol losses) |
| Colony 10 from T154 (both seats) | 0 of 32 | 0 of 32 (one 16-HP Cavalry, nothing in reach) |

## Gate 2: scenarios

- **Lab `byzantium-celtic-walls`** (100 runs, `legatum-sim scenario`): 0/100 in every arm ([off](lab-off.md), [joint 1 + finish](lab-joint1.md), [joint 2 + finish](lab-joint2.md)). Byzantium never assaults: its plan is Contain on the Celts until Peace at T137, and at most three of its units come within eight hexes of Colony 7 (the gate runner in `--scenario` mode, 32 runs). The limit there is the war plan, not the assault.
- **War suite** (64 runs, all seats; `docs/sim-results/hard-vs-human/war-scenarios/runs/`, ledger rows `joint-ctl`, `joint-on-all`, `joint2-on-all`, `finish-on-all`, `joint2-only-all`): control 36% mean (12/33 at parity); `finish_fallen` alone 36% (12/33), no scenario moved beyond noise; `joint_assault` (1 or 2) 35-36%: Mongols against the Bomber 35% to 57% [+8, +36], Tepid Swiss T220 64% to 40% [-39, -6] and its fresh-memory twin 64% to 43% (counting the Swiss Bombers in the forecast did not recover it).
- **Attack check** (`war-scenarios.sh --check`, `war-golden-horde-thin`): 0 of 64 on this branch's control and with the genes. The check already reads 0% at the branch base, below its 60% floor (see the report).

## Gate 3: human-seat benchmark

`hard_vs_human`, seed 310001, 256 reseeded salts x 8 maps (2,048 games per arm), plain Hard rivals, one spot server each. Control `ctl`: both genes 0 in every seat (468 seat wins in both batches). Delta: change in the designer's-seat win rate; on a rivals-only arm negative means stronger rivals ([compare.py](compare.py)).

| Arm | Seat wins | Delta pp [95%] | Batch |
|---|---:|---:|---|
| `joint_assault` 1 + `finish_fallen` 50, rivals only | 467 | -0.05 [-1.86, +1.71] | `joint-gate1-1103-1103-214301` ([arms](arms.json), [comparison](comparison.json)) |
| the same, seat only | 461 | -0.34 [-1.90, +1.17] | same |
| `finish_fallen` 50, rivals only | 482 | +0.68 [-0.68, +2.05] | `joint-gate2-1103-1103-222409` ([arms](arms2.json), [comparison](comparison2.json)) |
| `finish_fallen` 50, seat only | 484 | +0.78 [-0.39, +1.95] | same |
| `joint_assault` 2, rivals only | 494 | +1.27 [-0.54, +3.03] | same |

## Decision

- **`finish_fallen` 50 on:** it fixes the T93 case (13 to 32 of 32), costs nothing on the war suite, and the benchmark rivals-only reading (+0.68) is inside the noise. Pins: unchanged on the branch base; after merging master (`68f670acc`) the scripted Hard city-fire tape moved (repinned). The T93 result is the same on the merged code. Extended verify passed (`tools/cloud/verify-remote.sh claude/fix-joint-assault --profile extended`).
- **`joint_assault` off.** It does what the review asked for the trickle (lone hits -24% with Hard in his seat, -41% against his orders; units lost near his cities -29% and -34%), but it does not make the Aztecs take more: against his recorded orders they take 1.2 of his cities instead of 2.9, because waiting outside the range leaves the wear-down to nobody. The benchmark leans the same way (+1.27 rivals-only, not significant), and the Tepid Swiss T220 scenario loses 23 points. Turning the trickle into a capture needs the next step: a siege target the joint group can reach (Bibracte's ring has three free reachable tiles, so a joint group of more than three can never form there, yet the siege pass keeps choosing it), and the war plan bringing the army (in the Lab scenario Byzantium never comes).

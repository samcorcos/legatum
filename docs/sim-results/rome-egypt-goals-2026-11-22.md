# Rome's Colonia and Egypt's royal tomb: goals on against goals off (2026-11-22)

**Result.** The two held-city goals (rules 54; measured on the branch when it sat on rules 51) barely move the win table. Over 320 paired eight-seat Hard games, Egypt wins 20.0% with its goal against 19.4% without it (+0.6 points, paired 95% interval ±3.4), and Rome 1.6% against 0.9% (+0.6, ±1.5). Egypt raises its tomb in 35% of games and wins while holding it in 21 of its 64 wins. Rome founds the Colonia in 2 of 320 games: its veterans rarely reach level 3, and when they do they are usually held in a city they captured. No game's winner changed because of a goal's 2 VP. Rules 52, `legatum-sim`, scripted Hard in every seat, medium map, `--rotate-factions`.

**Setup.** Same seeds, seats and faction rotation in both arms: seeds 9000-9319 (game *i* plays seed 9000 + *i*), factions Egypt, Athens, Aztecs, Rome, Switzerland, Germany, Mongols, Sparta, rotated one seat per game. 320 games per arm, about 90 minutes of wall clock for both on 2 threads each.

- Arm A, goals on (the branch default): commit 61ce740, the final Colonia AI below.
- Arm B, goals off (`--rule-off rome_goal,egypt_goal`): commit c1d4b45. Every change between c1d4b45 and 61ce740 runs only while the Colonia is open (`colonia_on`), which this switch closes, so arm B plays identically on either commit. The decision trace rows added in 61ce740 are written only with `--decision-trace`.

```
legatum-sim --players 8 --difficulty hard \
  --factions Egypt,Athens,Aztecs,Rome,Switzerland,Germany,Mongols,Sparta --rotate-factions \
  --games 320 --seed 9000 --threads 2 --label armA --out-dir .
legatum-sim --players 8 --difficulty hard \
  --factions Egypt,Athens,Aztecs,Rome,Switzerland,Germany,Mongols,Sparta --rotate-factions \
  --games 320 --seed 9000 --threads 2 --rule-off rome_goal,egypt_goal --label armB --out-dir .
```

## Win share

| Faction | Goals on | Goals off | Difference |
|---|---|---|---|
| Egypt | 20.0% (64) | 19.4% (62) | +0.6 (paired 95% ±3.4) |
| Mongols | 18.8% (60) | 19.7% (63) | −0.9 |
| Germany | 15.3% (49) | 13.1% (42) | +2.2 |
| Athens | 13.4% (43) | 15.6% (50) | −2.2 |
| Aztecs | 12.8% (41) | 15.6% (50) | −2.8 |
| Switzerland | 9.7% (31) | 7.2% (23) | +2.5 |
| Sparta | 8.4% (27) | 8.4% (27) | 0 |
| Rome | 1.6% (5) | 0.9% (3) | +0.6 (paired 95% ±1.5) |

Best/worst ratio: 12.8 with goals on (Egypt 20.0% / Rome 1.6%), 21.0 with goals off (Mongols 19.7% / Rome 0.9%). The pillar-8 target is 1.5; Rome's weakness on this pool is the whole gap and the goal does not close it. The same faction won 233 of the 320 paired games. Both arms: 318 VP wins and 2 capital wins; median game length 225 turns (goals on) and 227 (goals off). Unpaired intervals for the two goal factions are ±6.2 (Egypt) and ±1.7 (Rome); pairing on seed removes about half of the noise.

## Egypt's royal tomb (goals on, 320 Egypt seats)

| Measure | Value |
|---|---|
| Seats that started the work | 173 (54%), median first start T188 |
| Tombs raised | 112 (35%), median T209 |
| Held by Egypt at the end | 84 |
| Held by a rival at the end | 24 |
| Captured at least once | 26 (no tomb was razed and rebuilt) |
| Median turns Egypt held a raised tomb | 27.5 |
| Work turns per starting seat | 11.4 |
| Pauses per starting seat | 0.53 |
| Egypt wins while holding its tomb | 21 of 64 |

## Rome's Colonia Victrix (goals on, 320 Rome seats)

| Measure | Value |
|---|---|
| Seats with a level-2 melee unit while the Colonia was open | 39 (12%), 48 units, median first T159 |
| Turns those seats had a level-2 unit | median 24, longest 72 |
| Seats with a level-3 melee unit | 5 (1.6%), median first T206, 55 seat-turns |
| Of those seat-turns, every level-3 unit held by the occupation lock | 35 |
| Of those seat-turns, no legal Colonia site on the map | 0 |
| Colonias founded | 2 (T221, T224), both held to the end, never captured |
| Rome wins | 5, none while holding a Colonia |

The five level-3 seats: game 95 (locked 32 of 33 turns), 118 (founded T224), 176 (locked 2 of 2), 184 (founded T221), 304 (locked 1 of 1).

## Games decided by a goal

A game counts when the winner held its own goal city at the end and won by 2 VP or less. Four games qualify, all Egypt wins with the tomb (games 5, 200, 220 and 289, margins 1 to 2 VP). In all four Egypt also won the paired goals-off game, so no winner in this sample owes its win to the goal's 2 VP. Two more games ended within 2 VP while a rival held a goal city; the telemetry does not say which rival, so they are not attributed.

## Why Rome's veteran never founded (traces of games 95 and 118)

Replayed with `--decision-trace` as single games (`--games 1 --seed 9095` with the factions rotated right by 7, Rome in seat 2; `--seed 9118` rotated right by 6, Rome in seat 1), one `colonia` trace row per level-3 unit per turn. On c1d4b45:

- **Game 118**: the veteran (level 3 from T212) was wounded on 21 of its 25 turns, and founding needs full health. The war passes moved it nearly every turn, so it hung at 86 to 96 HP for eight turns instead of healing. On the 4 turns it was whole (3 of them needed as a threatened city's defence), the site search showed 29 to 30 candidates and none a Colonia site: the search used the settler list that drops sites with border risk (an unseen rival city could stand within the spacing gap), and every site within two tiles of a rival's land carries that flag.
- **Game 95**: the veteran (level 3 from T163) took Cologne and the occupation lock held it there until the war ended. On the turns a Colonia site passed every gate, the founder's walk was ordered and then dropped (`drop_leaving_orders_of_locked_units`); on the other turns it failed "cannot leave" (it was the threatened city's defence) or was wounded.

Fixes in 61ce740 (one test each in `goal_colonia_tests.rs`): the site search uses the full legal list (arriving reveals the land; an illegal site drops out by itself); a wounded level-3 veteran with a site, free to leave and out of enemy reach, is claimed by the Colonia pass first and rests; a locked veteran is not a founder. The hold-risk judgement is unchanged. With them, game 118 founds at T224 and holds the Colonia to the end; game 95 still cannot (locked 32 of 33 turns).

## TOMB_TURNS sensitivity

`goals::TOMB_TURNS` (15) set to 10 and to 20 in two scratch binaries built from 61ce740 (the change was never committed), 160 games each on seeds 9000-9159, otherwise the arm A command with `--games 160 --threads 1`. The 15-turn row is arm A's first 160 games.

| Tomb turns | Egypt win | Started | Raised (of 160 seats) | Raised / started | Median raise | Held at end | Rival holds at end | Pauses per start | Wins holding it |
|---|---|---|---|---|---|---|---|---|---|
| 10 | 23.1% (37) | 81 | 56 (35%) | 69% | T198 | 41 | 11 | 0.43 | 10 |
| 15 | 21.2% (34) | 81 | 51 (32%) | 63% | T204 | 40 | 8 | 0.49 | 7 |
| 20 | 22.5% (36) | 81 | 42 (26%) | 52% | T210 | 33 | 4 | 0.59 | 7 |

The work starts on the same 81 seats at every length (median T189): the AI's start decision does not read the length. A longer count raises fewer tombs (69% to 52% of starts) and later, but the ones raised are captured less, so the tombs held at the end fall only from 41 to 33. Egypt's win share does not follow the length (95% interval about ±6.5 points at 160 games), so the count's length is not a balance lever at this sample size.

## Round 3: the founder kept out of the occupation lock

**Change** (commits 115df24 and 9b0deab, gene `rome_colonia`, AI only). While the Colonia is open, `unit_order` keeps the would-be founder (`colonia_reserved`: every level-3 melee veteran, or the best veteran below level 3 that is being kept) from any move that ends in a city whose occupation lock would hold it: a capture, or entering an occupied city of Rome's. If another free Roman melee unit can reach the city this turn, it takes the move instead. If not, and another unit is within two turns, an enemy capture waits one turn (never two in a row). Otherwise the founder captures. The founder's own Colonia walk never ends in such a city, and the modernization return no longer cancels the founder's healing rest. Decision-trace rows (`colonia`, outcome `stood_aside` or `allowed`) record each choice.

**Baseline.** The branch now carries three merges of master, which changed the games themselves. On the merged tree, games 95 and 118 no longer produce a Roman veteran of level 2 or higher, so their replays say nothing about this change. To separate the change from the merge, the same 320 games were run on the merged tree before the change: "Merged base", commit d84fc4c, Rules 52. Round 3 is commit 9b0deab. Both used the arm A command with goals on, seeds 9000-9319, in four chunks of 80 games (`--games 80 --seed 9000/9080/9160/9240`; 80 is a multiple of 8, so the rotation is unchanged).

| Measure | Arm A (round 2, pre-merge) | Merged base | Round 3 |
|---|---|---|---|
| Rome win | 1.6% (5) | 2.2% (7) | 1.2% (4) |
| Rome cities captured per seat | 1.83 | 1.93 | 1.76 |
| Rome seats with a level-2 unit | 39 | 35 | 25 |
| Rome seats with a level-3 unit | 5 | 2 | 4 |
| Level-3 seat-turns | 55 | 56 | 130 |
| Of those, all level-3 units locked | 35 | 51 | 30 |
| Colonias founded | 2 | 0 | 1 |
| Colonias held at the end | 2 | 0 | 1 |
| Egypt win | 20.0% (64) | 17.8% (57) | 20.3% (65) |
| Tombs started / raised / held at end | 173 / 112 / 84 | 179 / 104 / 80 | 168 / 104 / 78 |

Paired against the merged base:
- Rome's captures per seat: −0.17 (95% interval ±0.18).
- Rome's win share: −0.9 points (±1.6).
- Egypt's win share: +2.5 points (±1.9). Egypt's AI did not change, so this comes from Rome's games playing out differently; treat it as noise until a second pool repeats it.

The lock share of level-3 turns fell from 91% (51 of 56) to 23% (30 of 130). Of round 3's four level-3 seats:
- Game 301 founded the Colonia at T205, held it 57 turns and won.
- Game 291 was locked for 11 of 30 turns.
- Game 211 was locked for 18 of 18.
- Game 131 was locked for 1 of 1.

In games 211 and 291 the trace shows the founder took cities when no other unit was within reach ("no other unit can take the city"); that trade is intended.

Seats that reached level 2 fell from 35 to 25. A kept veteran that no longer captures cities earns XP more slowly. Captures per seat are down too, though within noise.

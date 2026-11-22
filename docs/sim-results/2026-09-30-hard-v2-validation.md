# Hard v2 validation (2026-09-30, Standard)

**Decision (2026-09-30): playtest-4 keeps the scripted Hard for Hard seats and ships city strike. Hard v2 stays in the code behind the `hard-v2-ai` switch.**

**Verdict: do not ship Hard v2 as the Hard AI.** It is not harder for a strong player; by every challenger measure it is easier than the old Hard. It fails ship criterion (a) and misses (b) on the point estimate; (c) passes with the 25% cap.

Hard v2 (`crates/ai/src/hard_v2.rs`): each Hard seat draws an opening from the Standard opening league's final equilibrium (`~/LegatumRuns/2026-09-30-standard/psro`, generation 30: DefendedExpand 73%, Tall 14%, Economic 13%) with every named opening floored at 10%, plays its tuned genome with primitives, and uses the opening phase machine with a 2-turn Defend hysteresis. City strike is on in every run (the new default rule). "Old Hard" is the pre-v2 scripted Hard (`--legacy-hard all`). Runs used 14 threads, niced; commands are at the end.

## Ship criteria

| Criterion | 40% cap | 25% cap (committed) | Result |
|---|---|---|---|
| (a) Snowball challenger wins no more vs Hard v2 than vs old Hard | 14.0% → 15.2% (p = 0.65) | 14.0% → 15.0% (p = 0.74) | no worse within noise, but higher |
| (a) GreedyExpand challenger wins no more vs Hard v2 than vs old Hard | 13.5% → 17.0% (p = 0.14) | 13.5% → 18.2% (p = 0.05) | **fail** |
| (b) One Hard v2 seat vs 7 old Hard: parity (12.5%) or better | 10.8% (CI 8.1–14.2) | 10.8% (CI 8.1–14.2) | **below parity** (within noise) |
| (c) No more stalls than old Hard | 1 vs 1 | 1 vs 1 | pass |
| (c) Average length ≤ 110% of old Hard | 295 vs 262 (+12.5%) | 285 vs 262 (+8.8%) | fail at 40%, pass at 25% |
| (d) The user's playtest-3 map and seat | seat 0 won 0/56 vs old Hard, 7–15/56 vs Hard v2 | 0/56 vs old Hard, 9–11/56 vs Hard v2 | Hard v2 is much easier here |

The one extra fix allowed was the cap: 40% → 25% fixed game length but made the GreedyExpand challenger's advantage larger.

## Why Hard v2 is easier

Old Hard fights more and gangs up on the leader. On all-AI tables old Hard declares 20.2 wars per game against 12.6–13.3 for Hard v2. On the playtest map, seven old Hard AIs declared war on the Snowball seat 25.6 times per game, against 11.8–14.3 for Hard v2, and that seat never won against old Hard. The league tuned openings against each other for their own win rate; nothing in its fitness rewards stopping a runaway leader, and the opening policies (Expand/Tall/Economic strategies) are less aggressive than the scripted Hard's. Hard v2 is the better table for strategic variety (six openings, each winning 8–23% of seats), not for difficulty.

## Committed configuration: 10% floor, 25% cap

Draw weights: DefendedExpand 25%, Tall 23.4%, Economic 21.6%, Rush 10%, GreedyExpand 10%, Snowball 10%.

### (a) Challenger vs table

One seat per game (rotated, seat = game index mod 8) plays the Hard v2 tuned opening; the other seven are old Hard or Hard v2. Same seeds for both tables. Lower challenger win rate = harder table.

| Challenger | vs 7 old Hard | vs 7 Hard v2 | Paired: won only vs old / only vs v2 | Challenger avg rank old → v2 |
|---|---|---|---|---|
| Snowball | 56/400 = 14.0% (95% CI 10.9–17.7) | 60/400 = 15.0% (95% CI 11.8–18.8) | 40 / 44 (sign test p = 0.74) | 4.02 → 3.81 |
| GreedyExpand | 54/400 = 13.5% (95% CI 10.5–17.2) | 73/400 = 18.2% (95% CI 14.8–22.3) | 33 / 52 (sign test p = 0.05) | 3.85 → 3.52 |

### (b) One Hard v2 seat vs 7 old Hard

Hard v2 won 43/400 = 10.8% (95% CI 8.1–14.2); parity is 12.5%. Stalls: 1.

| v2 opening | Games | Won |
|---|---|---|
| Economic | 99 | 11 (11.1%) |
| DefendedExpand | 96 | 10 (10.4%) |
| Tall | 84 | 4 (4.8%) |
| GreedyExpand | 45 | 3 (6.7%) |
| Rush | 41 | 5 (12.2%) |
| Snowball | 35 | 10 (28.6%) |

### (c) All-Hard-v2 tables vs all-old-Hard tables (same seeds)

| Table | Games | Avg turns | p90 turns | Stalls | City captures/game | Best/worst faction |
|---|---|---|---|---|---|---|
| All Hard v2 | 400 | 285 | 355 | 1 | 0.23 | 3.42x |
| All old Hard | 400 | 262 | 307 | 1 | 0.21 | 3.52x |

Faction win rates: v2: Switzerland 22.2%, Japan 15.5%, Germany 15.0%, Celts 13.0%, Athens 10.5%, Sparta 8.5%, Byzantium 8.5%, Mongols 6.5%; old: Switzerland 22.0%, Japan 17.5%, Celts 14.8%, Germany 14.0%, Byzantium 9.2%, Athens 8.0%, Mongols 8.0%, Sparta 6.2%

| Opening (all-v2 tables) | Share of seats | Win rate per seat | Phase switches mean / max |
|---|---|---|---|
| DefendedExpand | 25.0% | 86/799 = 10.8% | 6.6 / 33 |
| Tall | 23.8% | 99/761 = 13.0% | 4.3 / 25 |
| Economic | 21.1% | 53/675 = 7.9% | 2.4 / 27 |
| Snowball | 10.5% | 54/336 = 16.1% | 6.6 / 32 |
| GreedyExpand | 10.3% | 75/329 = 22.8% | 6.1 / 38 |
| Rush | 9.4% | 32/300 = 10.7% | 5.6 / 25 |

### (d) The playtest-3 map and seat

Same map, factions and starts as the user's playtest-3 game; seat 0 (the user's Switzerland) is played by the Proxy benchmark brain or the tuned Snowball opening, the other seven by old Hard or Hard v2; 56 brain-seed salts each.

| Seat 0 brain | Table | Games | Seat-0 team won | Seat-0 final VP | Seat-0 cities lost | Wars declared on seat 0's team per game | Avg turns |
|---|---|---|---|---|---|---|---|
| proxy | old Hard | 56 | 0 (0.0%) | 1.09 | 0.02 | 1.61 | 257 |
| proxy | Hard v2 | 56 | 9 (16.1%) | 3.91 | 0.36 | 3.21 | 295 |
| snowball | old Hard | 56 | 0 (0.0%) | 6.66 | 0.07 | 25.59 | 335 |
| snowball | Hard v2 | 56 | 11 (19.6%) | 6.84 | 0.32 | 14.32 | 321 |

## First try: 10% floor, 40% cap

Draw weights: DefendedExpand 40%, Tall 15.6%, Economic 14.4%, Rush 10%, GreedyExpand 10%, Snowball 10%.

### (a) Challenger vs table

One seat per game (rotated, seat = game index mod 8) plays the Hard v2 tuned opening; the other seven are old Hard or Hard v2. Same seeds for both tables. Lower challenger win rate = harder table.

| Challenger | vs 7 old Hard | vs 7 Hard v2 | Paired: won only vs old / only vs v2 | Challenger avg rank old → v2 |
|---|---|---|---|---|
| Snowball | 56/400 = 14.0% (95% CI 10.9–17.7) | 61/400 = 15.2% (95% CI 12.1–19.1) | 36 / 41 (sign test p = 0.65) | 4.02 → 3.74 |
| GreedyExpand | 54/400 = 13.5% (95% CI 10.5–17.2) | 68/400 = 17.0% (95% CI 13.6–21.0) | 31 / 45 (sign test p = 0.14) | 3.85 → 3.74 |

### (b) One Hard v2 seat vs 7 old Hard

Hard v2 won 43/400 = 10.8% (95% CI 8.1–14.2); parity is 12.5%. Stalls: 1.

| v2 opening | Games | Won |
|---|---|---|
| DefendedExpand | 141 | 18 (12.8%) |
| Tall | 68 | 6 (8.8%) |
| Economic | 63 | 7 (11.1%) |
| GreedyExpand | 45 | 3 (6.7%) |
| Snowball | 42 | 4 (9.5%) |
| Rush | 41 | 5 (12.2%) |

### (c) All-Hard-v2 tables vs all-old-Hard tables (same seeds)

| Table | Games | Avg turns | p90 turns | Stalls | City captures/game | Best/worst faction |
|---|---|---|---|---|---|---|
| All Hard v2 | 400 | 295 | 375 | 1 | 0.29 | 2.16x |
| All old Hard | 400 | 262 | 307 | 1 | 0.21 | 3.52x |

Faction win rates: v2: Switzerland 17.2%, Celts 17.0%, Japan 13.0%, Athens 12.2%, Germany 12.2%, Mongols 10.8%, Byzantium 9.2%, Sparta 8.0%; old: Switzerland 22.0%, Japan 17.5%, Celts 14.8%, Germany 14.0%, Byzantium 9.2%, Athens 8.0%, Mongols 8.0%, Sparta 6.2%

| Opening (all-v2 tables) | Share of seats | Win rate per seat | Phase switches mean / max |
|---|---|---|---|
| DefendedExpand | 40.8% | 137/1307 = 10.5% | 6.6 / 38 |
| Tall | 15.4% | 64/494 = 13.0% | 4.5 / 33 |
| Economic | 13.8% | 27/440 = 6.1% | 2.7 / 35 |
| Snowball | 10.3% | 67/330 = 20.3% | 6.3 / 27 |
| GreedyExpand | 10.3% | 73/329 = 22.2% | 6.4 / 32 |
| Rush | 9.4% | 31/300 = 10.3% | 5.6 / 25 |

### (d) The playtest-3 map and seat

| Seat 0 brain | Table | Games | Seat-0 team won | Seat-0 final VP | Seat-0 cities lost | Wars declared on seat 0's team per game | Avg turns |
|---|---|---|---|---|---|---|---|
| proxy | old Hard | 56 | 0 (0.0%) | 1.09 | 0.02 | 1.61 | 257 |
| proxy | Hard v2 | 56 | 7 (12.5%) | 3.75 | 0.43 | 2.82 | 308 |
| snowball | old Hard | 56 | 0 (0.0%) | 6.66 | 0.07 | 25.59 | 335 |
| snowball | Hard v2 | 56 | 15 (26.8%) | 7.36 | 0.43 | 11.75 | 305 |

## Method and commands

Every run: `legatum-sim --threads 14 --speed standard --difficulty hard`, 400 games, barbarians on, city strike on (default). Seeds 9910001+ for (a) and (b) (the same maps for both tables and the challenger rotation), 9920001+ for (c).

```sh
# (a) challenger, seat = game index mod 8, tuned genome from tuned_openings.rs
legatum-sim ... --seed 9910001 --challenger Snowball --legacy-hard all   # vs old Hard
legatum-sim ... --seed 9910001 --challenger Snowball                     # vs Hard v2
legatum-sim ... --seed 9910001 --challenger GreedyExpand [--legacy-hard all]
# (b) one Hard v2 seat, rotated
legatum-sim ... --seed 9910001 --hard-v2-rotation
# (c) whole tables
legatum-sim ... --seed 9920001
legatum-sim ... --seed 9920001 --legacy-hard all
# (d) the playtest-3 map and seat 0 (Switzerland), 56 salts per row
cargo run --release -p legatum-sim --example playtest_bench -- salts ~/LegatumData/playtest3-final-1849.legatum <proxy|snowball> <hard|hardv2> 56
```

The paired column in (a) counts seeds where the challenger won against only one of the two tables; the p value is an exact two-sided sign test on those pairs. (d) re-plays seat 0 with a brain: the user's recorded orders no longer replay once city strike changes the game (the tape diverges at turn 13). GreedyExpand's genome is its built-in one (the league kept it untuned).

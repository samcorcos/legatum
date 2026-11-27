# Simulation: india-office-upkeep-full (2026-11-26)

1000 games, first seed 20261126, brain `scripted`, turn limit off, safety cap 1200, 6 threads, started 2026-11-26T23:27:25Z. Wall time 1028.7s.

Source checkout at run start: `unavailable`; dirty: unknown. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--games","1000","--seed","20261126","--threads","6","--difficulty","hard","--rules-experiment","india-office-upkeep","--rules-experiment","india-office-full","--label","india-office-upkeep+full"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 1000 of 1000 |
| Game length, turns (mean / median / min-max) | 248.3 / 240 / 143-500 |
| Win reason mix | VP hold 96.0%, Capitals 3.6%, Turn limit 0.3%, Draw 0.1% |
| Median end turn | 240 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 3.6% / 96.0% / 0.3% / 0.1% |
| Games reaching the World War era (any empire) | 995 of 1000 (99.5%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1446 / 409 / 964 / 73 |
| Hold break rate | 28.3% of started; 29.8% of resolved (broken / (broken + completed)) |
| Hold wins | 960 |
| First reach to win, turns (mean / median / min-max) | 13.0 / 9 / 9-281 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 141 |
| Wins by a team another team beat to its target | 72 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 960 | 245.6 / 238 / 143-482 |
| Capitals | 36 | 290.2 / 272 / 207-493 |
| Turn limit | 3 | 500.0 / 500 / 500-500 |
| Draw | 1 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 104 | 10.4% |
| India | 88 | 8.8% |
| Byzantium | 88 | 8.8% |
| Germany | 138 | 13.8% |
| Celts | 160 | 16.0% |
| Mongols | 71 | 7.1% |
| Sparta | 106 | 10.6% |
| Athens | 137 | 13.7% |
| Aztecs | 107 | 10.7% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 960 | 96.0% |
| Conquest | 0 | 0.0% |
| Capitals | 36 | 3.6% |
| Turn limit | 3 | 0.3% |
| Draw | 1 | 0.1% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 899 | 104 | 11.6% |
| India | 895 | 88 | 9.8% |
| Byzantium | 878 | 88 | 10.0% |
| Germany | 880 | 138 | 15.7% |
| Celts | 896 | 160 | 17.9% |
| Mongols | 877 | 71 | 8.1% |
| Sparta | 891 | 106 | 11.9% |
| Athens | 895 | 137 | 15.3% |
| Aztecs | 889 | 107 | 12.0% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 8000 | 999 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 899 | 104 | 11.6% |
| India | Hard | 895 | 88 | 9.8% |
| Byzantium | Hard | 878 | 88 | 10.0% |
| Germany | Hard | 880 | 138 | 15.7% |
| Celts | Hard | 896 | 160 | 17.9% |
| Mongols | Hard | 877 | 71 | 8.1% |
| Sparta | Hard | 891 | 106 | 11.9% |
| Athens | Hard | 895 | 137 | 15.3% |
| Aztecs | Hard | 889 | 107 | 12.0% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 899 | 130 | 14.5% | 113 | 146 | 192 | 253 | 487 |
| India | 895 | 417 | 46.6% | 42 | 90 | 131 | 217 | 349 |
| Byzantium | 878 | 601 | 68.5% | 69 | 84 | 108 | 158 | 284 |
| Germany | 880 | 276 | 31.4% | 96 | 128 | 172 | 232 | 465 |
| Celts | 896 | 646 | 72.1% | 31 | 184 | 210 | 210 | 309 |
| Mongols | 877 | 127 | 14.5% | 119 | 156 | 201 | 250 | 326 |
| Sparta | 891 | 180 | 20.2% | 127 | 168 | 213 | 285 | 377 |
| Athens | 895 | 332 | 37.1% | 114 | 175 | 255 | 255 | 255 |
| Aztecs | 889 | 330 | 37.1% | 125 | 198 | 225 | 240 | 314 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 143 | 193 | 240 | 307 | 500 |
| Winning team VP | 2 | 10 | 10 | 13 | 17 |
| First Soldier completed (turn, per empire) | 6 | 8 | 21 | 64 | 264 |
| First Settler completed (turn, per empire) | 3 | 14 | 16 | 32 | 205 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 8 | 18 | 23 | 46 | 247 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 7671 | 329 |
| First Settler completed | 7654 | 346 |
| First city founded | 8000 | 0 |
| Second city founded | 7568 | 432 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 2009 |
| 1 | 1209 |
| 2 | 942 |
| 3 | 737 |
| 4 | 579 |
| 5 | 490 |
| 6 | 382 |
| 7 | 268 |
| 8 | 185 |
| 9 | 148 |
| 10 | 518 |
| 11 | 270 |
| 12 | 146 |
| 13 | 75 |
| 14 | 22 |
| 15 | 12 |
| 16 | 7 |
| 17 | 1 |

## End state

- Average game length: 248.3 turns
- Average cities per empire: 4.30
- Average techs per empire: 31.03
- Average eliminations per game: 2.67

## Strategy diversity: observed behavior (classifier v2)

1000 finished games; 8000 player seats; 999 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4426** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 3364 | 207 | 6.2% | 42.0% | 20.7% |
| Wide | 2542 | 701 | 27.6% | 31.8% | 70.2% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 12 | 0 | 0.0% | 0.1% | 0.0% |
| Economic | 641 | 91 | 14.2% | 8.0% | 9.1% |
| Generalist | 1441 | 0 | 0.0% | 18.0% | 0.0% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 19.4% / 3.6% (959) | n/a / n/a (0) | 33.3% / 4.9% (12) | 22.6% / 6.8% (460) | 82.0% / 2.0% (867) |
| Wide | 77.0% / 3.6% (959) | — | n/a / n/a (0) | 72.2% / 5.6% (12) | 57.7% / 5.0% (451) | 95.7% / 0.7% (855) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 61.8% / 4.9% (12) | 22.2% / 5.6% (12) | n/a / n/a (0) | — | 25.0% / 0.0% (4) | 95.0% / 5.0% (10) |
| Economic | 70.5% / 6.8% (460) | 37.2% / 5.0% (451) | n/a / n/a (0) | 75.0% / 0.0% (4) | — | 94.4% / 2.6% (389) |
| Generalist | 16.0% / 2.0% (867) | 3.6% / 0.7% (855) | n/a / n/a (0) | 0.0% / 5.0% (10) | 3.0% / 2.6% (389) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 8 / 257 | 87 / 378 | 0 / 0 | 0 / 0 | 9 / 119 | 0 / 145 |
| India | 30 / 500 | 58 / 233 | 0 / 0 | 0 / 1 | 0 / 5 | 0 / 156 |
| Byzantium | 1 / 15 | 79 / 403 | 0 / 0 | 0 / 4 | 8 / 124 | 0 / 332 |
| Germany | 17 / 339 | 114 / 303 | 0 / 0 | 0 / 3 | 7 / 44 | 0 / 191 |
| Celts | 17 / 267 | 134 / 409 | 0 / 0 | 0 / 0 | 9 / 87 | 0 / 133 |
| Mongols | 46 / 677 | 22 / 79 | 0 / 0 | 0 / 0 | 3 / 12 | 0 / 109 |
| Sparta | 55 / 693 | 34 / 149 | 0 / 0 | 0 / 4 | 17 / 34 | 0 / 11 |
| Athens | 10 / 268 | 102 / 303 | 0 / 0 | 0 / 0 | 25 / 125 | 0 / 199 |
| Aztecs | 23 / 348 | 71 / 285 | 0 / 0 | 0 / 0 | 13 / 91 | 0 / 165 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Turtle/Wonder, Generalist. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 999 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 15289/1000 | 15.289 | 1000/1000 (100.0%) |
| Peace offers | 99158/1000 | 99.158 | 1000/1000 (100.0%) |
| Peace agreements | 12235/1000 | 12.235 | 997/1000 (99.7%) |
| Battles | 1248384/1000 | 1248.384 | 1000/1000 (100.0%) |
| City captures | 23689/1000 | 23.689 | 1000/1000 (100.0%) |
| Settler captures | 4279/1000 | 4.279 | 960/1000 (96.0%) |
| Eliminations | 2665/1000 | 2.665 | 975/1000 (97.5%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 6680/8000 | 6680/8000 | 3220/999 |
| Faction goals | 4918/8000 | 4918/8000 | 2024/999 |
| Era objectives | 14293/8000 | 14293/8000 | 4080/999 |
| Capitals taken | 2653/8000 | 2653/8000 | 1532/999 |
| Total VP | 28544/8000 | 28544/8000 | 10856/999 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 89/723 (12.3%) | 1 |
| Tied leaders (winner in set) | 263/277 (94.9%) | 0 |
| Any leader set | 352/1000 (35.2%) | 1 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 7536/8000 | 5 | 20 | 29 | 33 | 70 | -2.2 (999) |
| Medieval | 7303/8000 | 31 | 58 | 62 | 66 | 130 | -2.0 (999) |
| Renaissance | 7232/8000 | 65 | 85 | 90 | 98 | 169 | -3.7 (999) |
| Industrial | 7010/8000 | 89 | 112 | 121 | 136 | 349 | -6.0 (999) |
| Machine Age | 6378/8000 | 109 | 141 | 153 | 174 | 444 | -8.4 (999) |
| World War | 5159/8000 | 128 | 167 | 182 | 207 | 499 | -10.7 (990) |
| Modern | 3505/8000 | 152 | 192 | 212 | 243 | 457 | -11.2 (888) |
| Future | 1861/8000 | 176 | 217 | 245 | 267 | 427 | -9.1 (600) |

Research idle turns (techs left, no research set): 2, of which stalled (two or more in a row): 0.

## Tech pace against the calendar

Median resolved turn each technology was learned, the display year of that turn (`calendar::calendar_label`) and the year the technology really arrived (n/a when no single year is established). Empires that never learned it are left out.

| Era | Technology | Learned / seats | Median turn | Median year | Real year |
| --- | --- | --- | --- | --- | --- |
| Ancient | Irrigation | 7961/8000 | 9 | 3173 BC | 6000 BC |
| Ancient | Bronze Working | 7800/8000 | 18 | 2242 BC | 3300 BC |
| Ancient | Masonry | 7997/8000 | 12 | 2863 BC | 3000 BC |
| Ancient | Currency | 7601/8000 | 27 | 1311 BC | 600 BC |
| Ancient | Writing | 7633/8000 | 25 | 1518 BC | 3200 BC |
| Classical | Mathematics | 7337/8000 | 45 | 251 BC | 300 BC |
| Classical | Engineering | 7319/8000 | 51 | 50 AD | n/a |
| Classical | Iron Working | 7380/8000 | 38 | 601 BC | 1200 BC |
| Classical | Code of Laws | 7313/8000 | 59 | 450 AD | 1754 BC |
| Classical | Philosophy | 7305/8000 | 61 | 530 AD | 500 BC |
| Medieval | Feudalism | 7279/8000 | 75 | 950 AD | n/a |
| Medieval | Machinery | 7287/8000 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 6445/8000 | 81 | 1130 AD | n/a |
| Medieval | Education | 7248/8000 | 86 | 1280 AD | 1088 AD |
| Medieval | Crop Rotation | 7234/8000 | 89 | 1370 AD | n/a |
| Renaissance | Gunpowder | 7211/8000 | 97 | 1470 AD | n/a |
| Renaissance | Metallurgy | 7151/8000 | 108 | 1580 AD | n/a |
| Renaissance | Military Engineering | 7170/8000 | 105 | 1550 AD | n/a |
| Renaissance | Banking | 7058/8000 | 118 | 1680 AD | 1397 AD |
| Renaissance | Printing Press | 7023/8000 | 119 | 1690 AD | 1440 AD |
| Industrial | Rifling | 6640/8000 | 143 | 1816 AD | n/a |
| Industrial | Military Science | 6818/8000 | 134 | 1783 AD | n/a |
| Industrial | Ballistics | 6695/8000 | 145 | 1824 AD | n/a |
| Industrial | Industrialization | 6914/8000 | 130 | 1768 AD | 1769 AD |
| Industrial | Railways | 6382/8000 | 153 | 1854 AD | 1825 AD |
| Machine Age | Replaceable Parts | 5892/8000 | 168 | 1889 AD | 1800 AD |
| Machine Age | Combustion | 6151/8000 | 162 | 1882 AD | 1876 AD |
| Machine Age | Steel | 5851/8000 | 167 | 1887 AD | 1856 AD |
| Machine Age | Flight | 5460/8000 | 176 | 1898 AD | 1903 AD |
| Machine Age | Electricity | 5208/8000 | 181 | 1903 AD | 1879 AD |
| World War | Mass Mobilisation | 4340/8000 | 197 | 1924 AD | n/a |
| World War | Rocketry | 4071/8000 | 201 | 1929 AD | 1926 AD |
| World War | Armoured Warfare | 3837/8000 | 207 | 1938 AD | 1916 AD |
| World War | Air Power | 3693/8000 | 210 | 1942 AD | n/a |
| World War | Electronics | 4809/8000 | 189 | 1912 AD | n/a |
| Modern | Mechanised Warfare | 2552/8000 | 226 | 1976 AD | n/a |
| Modern | Composite Armour | 2321/8000 | 232 | 1990 AD | n/a |
| Modern | Guided Missiles | 2178/8000 | 238 | 2005 AD | n/a |
| Modern | Jet Engines | 2107/8000 | 240 | 2010 AD | 1939 AD |
| Modern | Green Revolution | 3007/8000 | 220 | 1962 AD | n/a |
| Future | Autonomous Systems | 1259/8000 | 256 | 2032 AD | n/a |
| Future | Robotics | 1053/8000 | 260 | 2036 AD | n/a |
| Future | Fusion Power | 993/8000 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 1473/8000 | 254 | 2030 AD | n/a |
| Future | Recursive AI | 899/8000 | 256 | 2032 AD | n/a |

## Metric definitions

One Event::Battle, including Scout encounters, ranged shots and meeting engagements; not a war, siege or submitted attack order.

City captures use the existing GameRecord.city_captures count of Event::CityCaptured (recaptures count again). Settler captures count Event::SettlerCaptured separately; neither is inferred from ownership or unit losses.

Count the existing GameRecord.eliminations entries, one per Event::PlayerEliminated; telemetry.elimination_count is their length. Never infer from final alive flags.

Final-state VP: scored World Age cards and Celtic-sacked rival capitals are banked; held rival capitals count for their holder; Great Works and faction goals use their held core VpSource entries for the current holder. Player sources reconcile to core vp_total; team sources sum each member once and reconcile to core team_vp. Player, team and winning-team populations are separate views, never added together. Eliminated members retain banked VP, as in core scoring.

After resolution AND scoring of turn 50, when GameState.turn is 51, before turn-51 orders. Rank all initial TEAMS by core team_vp (sum every member once), matching the probe's all-seat population. Equal VP preserves the entire leader set: no population or ID tie-break. Games ending on turn 50 are eligible; games ending earlier are excluded. Turn-75 replay comeback data is independent.

Per initial empire, record the resolved turn (turn 1 is the first turn) of its first Event::ProductionCompleted for Soldier and Settler, and its second Event::CityFounded. Starting units and captured units do not count as production; purchases count only when production actually completes. City captures do not count as founding, and losing a city does not reset the founding count. Null means never reached by the end of observed play. Distributions pool reached empires only, using the report's existing percentile convention; reached and never counts include eliminated empires and safety-cap stops. Cap stops are censored, not proof a milestone could never happen. Only first plays count; panics and legacy records without opening telemetry are excluded, not never observations.

A team wins by VP only after standing at its VP target at data::VP_HOLD_TURNS consecutive turn ends (10 Standard); the turn it reaches the target is the first held turn. Started = Event::VictoryHoldStarted; broken = Event::VictoryHoldBroken (dropped below the target or was eliminated); completed = the hold reached the full count (a VP win, or a draw when several teams complete with identical standings); unfinished = still running when the game ended otherwise (conquest, turn limit, safety cap). started = broken + completed + unfinished. Break rate = broken / started, and broken / (broken + completed) over resolved holds. First reach to win = win turn minus the first turn the winner ever stood at its target (includes broken holds); final hold start to win is always turns_needed - 1. A comeback is a hold win by a team that broke an earlier hold of its own; wins not first to reach are hold wins where another team first stood at its target strictly earlier. Only games with games[].telemetry.victory_holds count; panics and older reports are excluded, not zero. Outcome `vp` means a completed hold: a winner at or above its target at the turn limit without a completed hold is `turn_limit`.

Event totals and incidence use first plays with telemetry, including runner-stopped games; panics and legacy records without telemetry are excluded, not zeros. Terminal score populations use finished games only. Snowball cohorts use finished games with a turn-50 snapshot and at least one team; unique and tied leaders are separate. Draws stay in each eligible denominator and count as no leader win. Unfinished games have no eventual result and are excluded. Empty denominators are reported as 0/0 (no rate).

Definitions reuse [balance-probe-1](../analysis/balance-probe-1.md) and [the probe's definitions](../../tools/analysis/README.md); this report is a fresh simulation sample, not a rerun of those campaigns.

## Promotions

1000 games with promotion telemetry. Promotions are UnitPromoted events by the level reached; final units are military units alive when the game ended.

| Level | Promotions / game | Final units / game |
| --- | --- | --- |
| 0 | 0.00 | 63.68 |
| 1 | 72.41 | 14.62 |
| 2 | 27.98 | 5.73 |
| 3 | 7.75 | 1.35 |
| 4 | 1.58 | 0.28 |
| 5 | 0.34 | 0.08 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 20261126 | 362 | 2165 | 47 | 4 | 3 |
| 1 | 20261127 | 229 | 1048 | 13 | 3 | 3 |
| 2 | 20261128 | 277 | 1660 | 14 | 8 | 1 |
| 3 | 20261129 | 287 | 1198 | 37 | 1 | 2 |
| 4 | 20261130 | 247 | 1447 | 23 | 3 | 2 |
| 5 | 20261131 | 295 | 1351 | 39 | 5 | 5 |
| 6 | 20261132 | 287 | 1553 | 18 | 2 | 3 |
| 7 | 20261133 | 311 | 1597 | 38 | 2 | 5 |
| 8 | 20261134 | 244 | 1096 | 19 | 6 | 3 |
| 9 | 20261135 | 247 | 1583 | 26 | 11 | 3 |
| 10 | 20261136 | 286 | 1360 | 29 | 6 | 2 |
| 11 | 20261137 | 278 | 1028 | 26 | 2 | 3 |
| 12 | 20261138 | 186 | 1033 | 22 | 2 | 3 |
| 13 | 20261139 | 238 | 1101 | 23 | 2 | 2 |
| 14 | 20261140 | 357 | 1862 | 38 | 4 | 3 |
| 15 | 20261141 | 265 | 1407 | 26 | 5 | 3 |
| 16 | 20261142 | 245 | 1239 | 28 | 4 | 2 |
| 17 | 20261143 | 250 | 1498 | 27 | 10 | 3 |
| 18 | 20261144 | 278 | 1146 | 29 | 3 | 4 |
| 19 | 20261145 | 318 | 1466 | 43 | 3 | 4 |
| 20 | 20261146 | 215 | 1035 | 15 | 1 | 2 |
| 21 | 20261147 | 326 | 1822 | 32 | 7 | 3 |
| 22 | 20261148 | 364 | 1908 | 47 | 3 | 4 |
| 23 | 20261149 | 333 | 2052 | 26 | 1 | 3 |
| 24 | 20261150 | 246 | 1413 | 27 | 4 | 1 |
| 25 | 20261151 | 242 | 1059 | 24 | 0 | 2 |
| 26 | 20261152 | 184 | 982 | 23 | 4 | 4 |
| 27 | 20261153 | 198 | 889 | 15 | 9 | 1 |
| 28 | 20261154 | 187 | 1087 | 19 | 8 | 1 |
| 29 | 20261155 | 204 | 859 | 23 | 7 | 2 |
| 30 | 20261156 | 287 | 1855 | 25 | 1 | 2 |
| 31 | 20261157 | 210 | 1069 | 22 | 2 | 3 |
| 32 | 20261158 | 246 | 1330 | 25 | 5 | 4 |
| 33 | 20261159 | 289 | 1405 | 36 | 3 | 3 |
| 34 | 20261160 | 188 | 785 | 17 | 3 | 3 |
| 35 | 20261161 | 211 | 834 | 15 | 4 | 2 |
| 36 | 20261162 | 324 | 1276 | 31 | 0 | 3 |
| 37 | 20261163 | 252 | 1337 | 32 | 4 | 2 |
| 38 | 20261164 | 199 | 803 | 21 | 4 | 3 |
| 39 | 20261165 | 303 | 1984 | 20 | 4 | 3 |
| 40 | 20261166 | 167 | 925 | 13 | 6 | 2 |
| 41 | 20261167 | 193 | 945 | 24 | 4 | 4 |
| 42 | 20261168 | 200 | 1308 | 23 | 3 | 4 |
| 43 | 20261169 | 228 | 1336 | 21 | 12 | 2 |
| 44 | 20261170 | 257 | 1334 | 15 | 5 | 3 |
| 45 | 20261171 | 220 | 886 | 16 | 6 | 2 |
| 46 | 20261172 | 259 | 1222 | 30 | 2 | 3 |
| 47 | 20261173 | 238 | 1031 | 20 | 3 | 3 |
| 48 | 20261174 | 210 | 1080 | 24 | 2 | 3 |
| 49 | 20261175 | 232 | 1337 | 20 | 1 | 3 |
| 50 | 20261176 | 252 | 1377 | 36 | 8 | 2 |
| 51 | 20261177 | 247 | 1224 | 22 | 4 | 2 |
| 52 | 20261178 | 242 | 1088 | 18 | 0 | 2 |
| 53 | 20261179 | 222 | 1014 | 18 | 3 | 2 |
| 54 | 20261180 | 203 | 789 | 10 | 2 | 3 |
| 55 | 20261181 | 216 | 888 | 16 | 5 | 2 |
| 56 | 20261182 | 236 | 1026 | 25 | 4 | 1 |
| 57 | 20261183 | 287 | 1561 | 34 | 1 | 2 |
| 58 | 20261184 | 234 | 1150 | 26 | 1 | 3 |
| 59 | 20261185 | 250 | 1044 | 21 | 3 | 2 |
| 60 | 20261186 | 227 | 1063 | 19 | 3 | 3 |
| 61 | 20261187 | 271 | 1099 | 30 | 5 | 3 |
| 62 | 20261188 | 339 | 2048 | 39 | 1 | 3 |
| 63 | 20261189 | 369 | 2264 | 30 | 6 | 4 |
| 64 | 20261190 | 272 | 1347 | 34 | 9 | 3 |
| 65 | 20261191 | 500 | 2699 | 25 | 7 | 4 |
| 66 | 20261192 | 271 | 1513 | 37 | 2 | 2 |
| 67 | 20261193 | 276 | 1677 | 20 | 1 | 3 |
| 68 | 20261194 | 200 | 927 | 17 | 2 | 3 |
| 69 | 20261195 | 198 | 973 | 10 | 5 | 2 |
| 70 | 20261196 | 193 | 836 | 13 | 7 | 2 |
| 71 | 20261197 | 229 | 1282 | 17 | 3 | 4 |
| 72 | 20261198 | 216 | 984 | 11 | 4 | 1 |
| 73 | 20261199 | 323 | 1581 | 33 | 6 | 4 |
| 74 | 20261200 | 192 | 981 | 16 | 3 | 1 |
| 75 | 20261201 | 240 | 1136 | 24 | 7 | 2 |
| 76 | 20261202 | 262 | 1576 | 32 | 10 | 0 |
| 77 | 20261203 | 308 | 1562 | 43 | 9 | 4 |
| 78 | 20261204 | 365 | 1637 | 22 | 8 | 6 |
| 79 | 20261205 | 201 | 867 | 17 | 3 | 0 |
| 80 | 20261206 | 222 | 838 | 23 | 12 | 2 |
| 81 | 20261207 | 257 | 1045 | 15 | 9 | 3 |
| 82 | 20261208 | 207 | 1012 | 24 | 7 | 1 |
| 83 | 20261209 | 208 | 1127 | 20 | 2 | 1 |
| 84 | 20261210 | 282 | 1430 | 43 | 6 | 5 |
| 85 | 20261211 | 218 | 1213 | 28 | 3 | 3 |
| 86 | 20261212 | 241 | 1080 | 17 | 2 | 3 |
| 87 | 20261213 | 246 | 868 | 20 | 7 | 2 |
| 88 | 20261214 | 270 | 993 | 25 | 6 | 3 |
| 89 | 20261215 | 212 | 1168 | 21 | 6 | 2 |
| 90 | 20261216 | 245 | 1198 | 30 | 7 | 3 |
| 91 | 20261217 | 274 | 1349 | 35 | 5 | 4 |
| 92 | 20261218 | 213 | 1008 | 28 | 3 | 4 |
| 93 | 20261219 | 241 | 1088 | 9 | 3 | 4 |
| 94 | 20261220 | 218 | 1008 | 22 | 2 | 2 |
| 95 | 20261221 | 273 | 1236 | 31 | 8 | 3 |
| 96 | 20261222 | 209 | 1080 | 23 | 5 | 3 |
| 97 | 20261223 | 241 | 956 | 19 | 3 | 2 |
| 98 | 20261224 | 382 | 2054 | 30 | 7 | 5 |
| 99 | 20261225 | 322 | 1978 | 40 | 6 | 2 |
| 100 | 20261226 | 184 | 825 | 20 | 4 | 3 |
| 101 | 20261227 | 267 | 1436 | 36 | 5 | 3 |
| 102 | 20261228 | 219 | 1051 | 10 | 4 | 1 |
| 103 | 20261229 | 254 | 1157 | 22 | 3 | 3 |
| 104 | 20261230 | 354 | 1936 | 36 | 4 | 5 |
| 105 | 20261231 | 199 | 692 | 12 | 1 | 2 |
| 106 | 20261232 | 215 | 1237 | 23 | 4 | 2 |
| 107 | 20261233 | 272 | 1335 | 27 | 2 | 2 |
| 108 | 20261234 | 207 | 655 | 22 | 9 | 4 |
| 109 | 20261235 | 229 | 1036 | 21 | 2 | 3 |
| 110 | 20261236 | 181 | 746 | 9 | 3 | 2 |
| 111 | 20261237 | 265 | 1382 | 31 | 2 | 4 |
| 112 | 20261238 | 241 | 1084 | 40 | 5 | 4 |
| 113 | 20261239 | 267 | 1580 | 22 | 3 | 2 |
| 114 | 20261240 | 227 | 1267 | 15 | 2 | 2 |
| 115 | 20261241 | 236 | 1020 | 15 | 0 | 4 |
| 116 | 20261242 | 291 | 1609 | 20 | 4 | 3 |
| 117 | 20261243 | 226 | 1131 | 17 | 5 | 2 |
| 118 | 20261244 | 217 | 1291 | 22 | 6 | 3 |
| 119 | 20261245 | 240 | 1183 | 12 | 3 | 2 |
| 120 | 20261246 | 230 | 1124 | 20 | 6 | 3 |
| 121 | 20261247 | 202 | 757 | 14 | 4 | 2 |
| 122 | 20261248 | 283 | 1566 | 36 | 5 | 2 |
| 123 | 20261249 | 237 | 1095 | 22 | 0 | 1 |
| 124 | 20261250 | 203 | 929 | 18 | 6 | 3 |
| 125 | 20261251 | 222 | 1085 | 13 | 1 | 3 |
| 126 | 20261252 | 227 | 906 | 32 | 9 | 4 |
| 127 | 20261253 | 224 | 926 | 16 | 2 | 2 |
| 128 | 20261254 | 234 | 1229 | 18 | 7 | 4 |
| 129 | 20261255 | 216 | 1087 | 23 | 2 | 3 |
| 130 | 20261256 | 228 | 1239 | 28 | 1 | 4 |
| 131 | 20261257 | 237 | 1143 | 22 | 1 | 3 |
| 132 | 20261258 | 272 | 1071 | 13 | 1 | 3 |
| 133 | 20261259 | 176 | 753 | 14 | 5 | 1 |
| 134 | 20261260 | 279 | 1564 | 33 | 3 | 2 |
| 135 | 20261261 | 500 | 2016 | 64 | 5 | 5 |
| 136 | 20261262 | 212 | 988 | 24 | 1 | 1 |
| 137 | 20261263 | 193 | 948 | 14 | 1 | 2 |
| 138 | 20261264 | 192 | 942 | 20 | 5 | 3 |
| 139 | 20261265 | 224 | 1086 | 20 | 5 | 3 |
| 140 | 20261266 | 234 | 1339 | 23 | 8 | 2 |
| 141 | 20261267 | 280 | 1206 | 32 | 5 | 5 |
| 142 | 20261268 | 226 | 1125 | 24 | 3 | 0 |
| 143 | 20261269 | 247 | 1311 | 27 | 6 | 4 |
| 144 | 20261270 | 234 | 1004 | 22 | 1 | 1 |
| 145 | 20261271 | 277 | 1335 | 21 | 2 | 2 |
| 146 | 20261272 | 273 | 1153 | 21 | 8 | 3 |
| 147 | 20261273 | 243 | 1078 | 23 | 1 | 4 |
| 148 | 20261274 | 243 | 1168 | 22 | 4 | 2 |
| 149 | 20261275 | 234 | 1182 | 28 | 6 | 1 |
| 150 | 20261276 | 279 | 1047 | 10 | 8 | 1 |
| 151 | 20261277 | 221 | 1100 | 18 | 6 | 3 |
| 152 | 20261278 | 240 | 1357 | 34 | 6 | 2 |
| 153 | 20261279 | 213 | 1169 | 29 | 10 | 1 |
| 154 | 20261280 | 242 | 1068 | 30 | 5 | 3 |
| 155 | 20261281 | 236 | 1407 | 35 | 15 | 2 |
| 156 | 20261282 | 304 | 1431 | 35 | 5 | 3 |
| 157 | 20261283 | 225 | 1069 | 25 | 3 | 1 |
| 158 | 20261284 | 394 | 1939 | 25 | 6 | 5 |
| 159 | 20261285 | 236 | 1250 | 29 | 10 | 3 |
| 160 | 20261286 | 173 | 854 | 17 | 4 | 3 |
| 161 | 20261287 | 270 | 1488 | 23 | 3 | 1 |
| 162 | 20261288 | 238 | 1270 | 29 | 3 | 2 |
| 163 | 20261289 | 228 | 1161 | 17 | 1 | 3 |
| 164 | 20261290 | 298 | 1638 | 35 | 5 | 3 |
| 165 | 20261291 | 270 | 1390 | 27 | 6 | 3 |
| 166 | 20261292 | 251 | 1303 | 32 | 5 | 4 |
| 167 | 20261293 | 223 | 1179 | 23 | 2 | 2 |
| 168 | 20261294 | 226 | 1287 | 17 | 5 | 4 |
| 169 | 20261295 | 363 | 1995 | 46 | 2 | 4 |
| 170 | 20261296 | 337 | 1867 | 40 | 6 | 2 |
| 171 | 20261297 | 220 | 781 | 21 | 1 | 3 |
| 172 | 20261298 | 236 | 1056 | 27 | 3 | 3 |
| 173 | 20261299 | 330 | 1534 | 34 | 5 | 4 |
| 174 | 20261300 | 269 | 1595 | 28 | 12 | 3 |
| 175 | 20261301 | 243 | 1217 | 30 | 0 | 2 |
| 176 | 20261302 | 195 | 969 | 17 | 5 | 3 |
| 177 | 20261303 | 237 | 1083 | 28 | 0 | 3 |
| 178 | 20261304 | 246 | 880 | 7 | 1 | 5 |
| 179 | 20261305 | 299 | 2003 | 26 | 8 | 1 |
| 180 | 20261306 | 340 | 1374 | 36 | 2 | 6 |
| 181 | 20261307 | 214 | 953 | 25 | 2 | 2 |
| 182 | 20261308 | 281 | 1385 | 34 | 7 | 2 |
| 183 | 20261309 | 251 | 1122 | 30 | 4 | 4 |
| 184 | 20261310 | 247 | 1585 | 33 | 6 | 3 |
| 185 | 20261311 | 214 | 1085 | 25 | 5 | 3 |
| 186 | 20261312 | 276 | 1440 | 18 | 5 | 4 |
| 187 | 20261313 | 263 | 1384 | 32 | 7 | 3 |
| 188 | 20261314 | 208 | 1046 | 18 | 7 | 2 |
| 189 | 20261315 | 402 | 2095 | 32 | 10 | 5 |
| 190 | 20261316 | 202 | 870 | 13 | 7 | 4 |
| 191 | 20261317 | 242 | 1243 | 16 | 4 | 3 |
| 192 | 20261318 | 268 | 1443 | 35 | 2 | 4 |
| 193 | 20261319 | 318 | 1385 | 27 | 1 | 2 |
| 194 | 20261320 | 226 | 1076 | 23 | 4 | 1 |
| 195 | 20261321 | 235 | 1170 | 31 | 6 | 3 |
| 196 | 20261322 | 259 | 1035 | 27 | 1 | 2 |
| 197 | 20261323 | 194 | 997 | 10 | 2 | 3 |
| 198 | 20261324 | 225 | 975 | 20 | 2 | 2 |
| 199 | 20261325 | 211 | 1025 | 30 | 9 | 3 |
| 200 | 20261326 | 245 | 1179 | 31 | 3 | 1 |
| 201 | 20261327 | 227 | 933 | 23 | 4 | 2 |
| 202 | 20261328 | 213 | 1020 | 17 | 2 | 3 |
| 203 | 20261329 | 237 | 1394 | 18 | 3 | 3 |
| 204 | 20261330 | 201 | 862 | 19 | 3 | 1 |
| 205 | 20261331 | 299 | 1330 | 38 | 2 | 5 |
| 206 | 20261332 | 296 | 1403 | 18 | 4 | 3 |
| 207 | 20261333 | 186 | 787 | 14 | 6 | 0 |
| 208 | 20261334 | 221 | 1217 | 24 | 4 | 3 |
| 209 | 20261335 | 343 | 1969 | 37 | 5 | 2 |
| 210 | 20261336 | 251 | 1370 | 16 | 12 | 2 |
| 211 | 20261337 | 259 | 1389 | 30 | 15 | 4 |
| 212 | 20261338 | 249 | 1175 | 12 | 5 | 0 |
| 213 | 20261339 | 208 | 909 | 14 | 6 | 1 |
| 214 | 20261340 | 233 | 764 | 8 | 3 | 2 |
| 215 | 20261341 | 182 | 720 | 18 | 7 | 4 |
| 216 | 20261342 | 435 | 2155 | 56 | 5 | 5 |
| 217 | 20261343 | 210 | 1011 | 23 | 7 | 1 |
| 218 | 20261344 | 233 | 1096 | 17 | 3 | 2 |
| 219 | 20261345 | 306 | 1628 | 33 | 5 | 2 |
| 220 | 20261346 | 276 | 1605 | 40 | 3 | 5 |
| 221 | 20261347 | 340 | 1886 | 38 | 5 | 3 |
| 222 | 20261348 | 228 | 1082 | 22 | 5 | 3 |
| 223 | 20261349 | 215 | 892 | 12 | 6 | 3 |
| 224 | 20261350 | 207 | 1095 | 22 | 4 | 5 |
| 225 | 20261351 | 305 | 1423 | 46 | 6 | 4 |
| 226 | 20261352 | 219 | 1025 | 23 | 1 | 1 |
| 227 | 20261353 | 231 | 1277 | 20 | 5 | 3 |
| 228 | 20261354 | 234 | 981 | 20 | 2 | 4 |
| 229 | 20261355 | 253 | 1416 | 21 | 0 | 3 |
| 230 | 20261356 | 325 | 1387 | 22 | 1 | 6 |
| 231 | 20261357 | 176 | 872 | 12 | 3 | 2 |
| 232 | 20261358 | 224 | 1094 | 15 | 7 | 4 |
| 233 | 20261359 | 233 | 1180 | 28 | 4 | 3 |
| 234 | 20261360 | 202 | 811 | 15 | 4 | 4 |
| 235 | 20261361 | 202 | 611 | 6 | 3 | 2 |
| 236 | 20261362 | 321 | 1993 | 51 | 4 | 3 |
| 237 | 20261363 | 276 | 1456 | 26 | 7 | 3 |
| 238 | 20261364 | 212 | 862 | 21 | 6 | 4 |
| 239 | 20261365 | 335 | 1677 | 27 | 4 | 5 |
| 240 | 20261366 | 228 | 1267 | 27 | 7 | 3 |
| 241 | 20261367 | 277 | 1622 | 26 | 3 | 4 |
| 242 | 20261368 | 229 | 1224 | 26 | 3 | 3 |
| 243 | 20261369 | 245 | 1078 | 24 | 3 | 2 |
| 244 | 20261370 | 212 | 990 | 26 | 8 | 3 |
| 245 | 20261371 | 210 | 868 | 20 | 1 | 1 |
| 246 | 20261372 | 176 | 763 | 11 | 3 | 2 |
| 247 | 20261373 | 237 | 1350 | 32 | 1 | 3 |
| 248 | 20261374 | 394 | 2741 | 16 | 3 | 5 |
| 249 | 20261375 | 222 | 959 | 16 | 3 | 2 |
| 250 | 20261376 | 242 | 1414 | 27 | 0 | 2 |
| 251 | 20261377 | 185 | 878 | 15 | 4 | 3 |
| 252 | 20261378 | 309 | 1865 | 23 | 6 | 3 |
| 253 | 20261379 | 234 | 1008 | 17 | 2 | 2 |
| 254 | 20261380 | 248 | 1390 | 26 | 11 | 3 |
| 255 | 20261381 | 306 | 1386 | 19 | 5 | 4 |
| 256 | 20261382 | 221 | 849 | 22 | 1 | 4 |
| 257 | 20261383 | 267 | 1284 | 30 | 4 | 5 |
| 258 | 20261384 | 246 | 1616 | 25 | 4 | 3 |
| 259 | 20261385 | 216 | 927 | 12 | 7 | 4 |
| 260 | 20261386 | 285 | 1292 | 23 | 7 | 2 |
| 261 | 20261387 | 253 | 1150 | 33 | 9 | 1 |
| 262 | 20261388 | 206 | 825 | 15 | 4 | 4 |
| 263 | 20261389 | 259 | 1445 | 23 | 4 | 3 |
| 264 | 20261390 | 194 | 841 | 24 | 2 | 3 |
| 265 | 20261391 | 200 | 966 | 17 | 5 | 2 |
| 266 | 20261392 | 193 | 789 | 14 | 0 | 3 |
| 267 | 20261393 | 276 | 1458 | 26 | 7 | 1 |
| 268 | 20261394 | 229 | 1295 | 35 | 7 | 2 |
| 269 | 20261395 | 229 | 1206 | 19 | 7 | 0 |
| 270 | 20261396 | 236 | 1076 | 33 | 3 | 3 |
| 271 | 20261397 | 219 | 1213 | 22 | 5 | 3 |
| 272 | 20261398 | 324 | 1849 | 29 | 2 | 4 |
| 273 | 20261399 | 217 | 1094 | 28 | 5 | 1 |
| 274 | 20261400 | 213 | 683 | 19 | 1 | 3 |
| 275 | 20261401 | 187 | 883 | 19 | 6 | 2 |
| 276 | 20261402 | 225 | 1110 | 19 | 4 | 2 |
| 277 | 20261403 | 285 | 1465 | 41 | 7 | 4 |
| 278 | 20261404 | 231 | 1419 | 27 | 2 | 3 |
| 279 | 20261405 | 256 | 1198 | 21 | 1 | 2 |
| 280 | 20261406 | 217 | 1205 | 22 | 9 | 1 |
| 281 | 20261407 | 240 | 1277 | 22 | 3 | 2 |
| 282 | 20261408 | 254 | 1447 | 38 | 8 | 2 |
| 283 | 20261409 | 283 | 1469 | 19 | 8 | 3 |
| 284 | 20261410 | 273 | 1537 | 21 | 4 | 5 |
| 285 | 20261411 | 228 | 1199 | 31 | 4 | 1 |
| 286 | 20261412 | 316 | 1929 | 31 | 0 | 0 |
| 287 | 20261413 | 262 | 1394 | 23 | 2 | 2 |
| 288 | 20261414 | 264 | 1380 | 23 | 4 | 2 |
| 289 | 20261415 | 281 | 1633 | 30 | 4 | 2 |
| 290 | 20261416 | 275 | 1676 | 18 | 2 | 5 |
| 291 | 20261417 | 180 | 776 | 13 | 3 | 2 |
| 292 | 20261418 | 271 | 1351 | 27 | 1 | 3 |
| 293 | 20261419 | 309 | 1843 | 32 | 6 | 3 |
| 294 | 20261420 | 257 | 1206 | 29 | 7 | 2 |
| 295 | 20261421 | 215 | 1138 | 23 | 9 | 0 |
| 296 | 20261422 | 190 | 759 | 15 | 0 | 1 |
| 297 | 20261423 | 258 | 1517 | 20 | 6 | 2 |
| 298 | 20261424 | 334 | 1413 | 34 | 1 | 5 |
| 299 | 20261425 | 246 | 1467 | 16 | 1 | 2 |
| 300 | 20261426 | 271 | 1380 | 35 | 6 | 4 |
| 301 | 20261427 | 345 | 2259 | 43 | 2 | 3 |
| 302 | 20261428 | 250 | 1545 | 37 | 6 | 3 |
| 303 | 20261429 | 237 | 1401 | 13 | 3 | 2 |
| 304 | 20261430 | 192 | 915 | 14 | 7 | 2 |
| 305 | 20261431 | 269 | 1212 | 26 | 6 | 4 |
| 306 | 20261432 | 248 | 816 | 24 | 3 | 3 |
| 307 | 20261433 | 229 | 1157 | 40 | 3 | 3 |
| 308 | 20261434 | 196 | 852 | 20 | 4 | 3 |
| 309 | 20261435 | 208 | 1223 | 18 | 2 | 3 |
| 310 | 20261436 | 217 | 1208 | 30 | 1 | 2 |
| 311 | 20261437 | 340 | 1794 | 38 | 4 | 2 |
| 312 | 20261438 | 226 | 1073 | 13 | 3 | 3 |
| 313 | 20261439 | 172 | 814 | 14 | 3 | 2 |
| 314 | 20261440 | 374 | 1878 | 35 | 7 | 4 |
| 315 | 20261441 | 273 | 1197 | 14 | 3 | 1 |
| 316 | 20261442 | 313 | 1216 | 30 | 5 | 3 |
| 317 | 20261443 | 235 | 1075 | 16 | 3 | 2 |
| 318 | 20261444 | 214 | 1092 | 24 | 8 | 3 |
| 319 | 20261445 | 239 | 1420 | 35 | 2 | 4 |
| 320 | 20261446 | 269 | 1680 | 31 | 2 | 4 |
| 321 | 20261447 | 194 | 1089 | 20 | 1 | 3 |
| 322 | 20261448 | 189 | 925 | 14 | 2 | 1 |
| 323 | 20261449 | 209 | 1130 | 14 | 3 | 0 |
| 324 | 20261450 | 222 | 1068 | 24 | 7 | 2 |
| 325 | 20261451 | 256 | 1289 | 25 | 10 | 2 |
| 326 | 20261452 | 240 | 1212 | 22 | 7 | 4 |
| 327 | 20261453 | 303 | 1550 | 32 | 0 | 4 |
| 328 | 20261454 | 317 | 1937 | 33 | 5 | 2 |
| 329 | 20261455 | 213 | 870 | 28 | 8 | 1 |
| 330 | 20261456 | 210 | 1098 | 20 | 2 | 3 |
| 331 | 20261457 | 258 | 1145 | 31 | 6 | 4 |
| 332 | 20261458 | 236 | 1250 | 27 | 9 | 2 |
| 333 | 20261459 | 227 | 1181 | 32 | 3 | 3 |
| 334 | 20261460 | 293 | 1331 | 27 | 5 | 3 |
| 335 | 20261461 | 178 | 761 | 11 | 4 | 2 |
| 336 | 20261462 | 315 | 1406 | 47 | 1 | 2 |
| 337 | 20261463 | 235 | 1432 | 16 | 3 | 1 |
| 338 | 20261464 | 199 | 981 | 17 | 10 | 2 |
| 339 | 20261465 | 229 | 1183 | 19 | 10 | 1 |
| 340 | 20261466 | 222 | 1144 | 20 | 4 | 3 |
| 341 | 20261467 | 292 | 1533 | 37 | 5 | 5 |
| 342 | 20261468 | 194 | 921 | 20 | 5 | 2 |
| 343 | 20261469 | 315 | 1873 | 36 | 5 | 3 |
| 344 | 20261470 | 296 | 1133 | 26 | 2 | 3 |
| 345 | 20261471 | 198 | 891 | 21 | 10 | 1 |
| 346 | 20261472 | 284 | 1566 | 16 | 3 | 2 |
| 347 | 20261473 | 198 | 931 | 15 | 3 | 1 |
| 348 | 20261474 | 228 | 1042 | 21 | 6 | 2 |
| 349 | 20261475 | 230 | 1233 | 25 | 4 | 2 |
| 350 | 20261476 | 248 | 1232 | 21 | 14 | 1 |
| 351 | 20261477 | 339 | 1852 | 34 | 0 | 3 |
| 352 | 20261478 | 223 | 961 | 22 | 6 | 4 |
| 353 | 20261479 | 255 | 1509 | 17 | 4 | 3 |
| 354 | 20261480 | 310 | 1577 | 37 | 6 | 3 |
| 355 | 20261481 | 245 | 1389 | 24 | 2 | 3 |
| 356 | 20261482 | 311 | 1597 | 27 | 9 | 4 |
| 357 | 20261483 | 191 | 899 | 18 | 4 | 3 |
| 358 | 20261484 | 221 | 1032 | 23 | 6 | 1 |
| 359 | 20261485 | 241 | 1720 | 39 | 1 | 1 |
| 360 | 20261486 | 277 | 1072 | 25 | 4 | 2 |
| 361 | 20261487 | 231 | 1147 | 21 | 4 | 2 |
| 362 | 20261488 | 217 | 1248 | 22 | 0 | 2 |
| 363 | 20261489 | 233 | 1091 | 16 | 7 | 2 |
| 364 | 20261490 | 237 | 979 | 23 | 2 | 4 |
| 365 | 20261491 | 263 | 1304 | 28 | 4 | 4 |
| 366 | 20261492 | 269 | 1137 | 21 | 7 | 3 |
| 367 | 20261493 | 233 | 1258 | 16 | 1 | 3 |
| 368 | 20261494 | 303 | 1403 | 36 | 11 | 5 |
| 369 | 20261495 | 208 | 1161 | 12 | 10 | 0 |
| 370 | 20261496 | 193 | 889 | 20 | 3 | 3 |
| 371 | 20261497 | 255 | 1299 | 16 | 4 | 4 |
| 372 | 20261498 | 228 | 1274 | 35 | 3 | 4 |
| 373 | 20261499 | 179 | 875 | 23 | 4 | 2 |
| 374 | 20261500 | 272 | 1357 | 30 | 1 | 4 |
| 375 | 20261501 | 177 | 841 | 12 | 5 | 1 |
| 376 | 20261502 | 290 | 1624 | 20 | 3 | 5 |
| 377 | 20261503 | 188 | 833 | 13 | 2 | 4 |
| 378 | 20261504 | 234 | 1348 | 25 | 12 | 3 |
| 379 | 20261505 | 223 | 1119 | 19 | 6 | 2 |
| 380 | 20261506 | 379 | 2123 | 50 | 9 | 4 |
| 381 | 20261507 | 203 | 912 | 19 | 5 | 2 |
| 382 | 20261508 | 310 | 1449 | 28 | 7 | 3 |
| 383 | 20261509 | 188 | 1005 | 16 | 6 | 2 |
| 384 | 20261510 | 234 | 1105 | 27 | 2 | 3 |
| 385 | 20261511 | 216 | 1029 | 35 | 6 | 4 |
| 386 | 20261512 | 238 | 1193 | 31 | 7 | 2 |
| 387 | 20261513 | 220 | 1063 | 27 | 3 | 2 |
| 388 | 20261514 | 253 | 1349 | 28 | 2 | 4 |
| 389 | 20261515 | 203 | 859 | 7 | 4 | 3 |
| 390 | 20261516 | 259 | 1766 | 16 | 12 | 2 |
| 391 | 20261517 | 179 | 726 | 14 | 7 | 1 |
| 392 | 20261518 | 271 | 1464 | 18 | 5 | 0 |
| 393 | 20261519 | 288 | 1076 | 32 | 6 | 2 |
| 394 | 20261520 | 232 | 878 | 16 | 2 | 4 |
| 395 | 20261521 | 179 | 813 | 12 | 6 | 2 |
| 396 | 20261522 | 206 | 1119 | 26 | 8 | 3 |
| 397 | 20261523 | 217 | 1162 | 25 | 6 | 2 |
| 398 | 20261524 | 197 | 695 | 17 | 7 | 1 |
| 399 | 20261525 | 226 | 1140 | 20 | 6 | 1 |
| 400 | 20261526 | 256 | 1031 | 16 | 2 | 5 |
| 401 | 20261527 | 246 | 1369 | 15 | 6 | 1 |
| 402 | 20261528 | 265 | 1009 | 18 | 4 | 3 |
| 403 | 20261529 | 233 | 1004 | 19 | 7 | 3 |
| 404 | 20261530 | 201 | 931 | 18 | 4 | 2 |
| 405 | 20261531 | 238 | 1190 | 26 | 1 | 3 |
| 406 | 20261532 | 240 | 1182 | 20 | 6 | 2 |
| 407 | 20261533 | 196 | 945 | 11 | 5 | 1 |
| 408 | 20261534 | 195 | 942 | 18 | 10 | 0 |
| 409 | 20261535 | 267 | 1422 | 43 | 5 | 4 |
| 410 | 20261536 | 179 | 1117 | 18 | 0 | 1 |
| 411 | 20261537 | 233 | 1177 | 26 | 8 | 3 |
| 412 | 20261538 | 254 | 1160 | 22 | 9 | 3 |
| 413 | 20261539 | 257 | 1060 | 32 | 4 | 3 |
| 414 | 20261540 | 283 | 1230 | 27 | 2 | 4 |
| 415 | 20261541 | 223 | 1396 | 25 | 1 | 2 |
| 416 | 20261542 | 224 | 1205 | 30 | 2 | 2 |
| 417 | 20261543 | 238 | 1176 | 19 | 2 | 3 |
| 418 | 20261544 | 230 | 1183 | 27 | 2 | 2 |
| 419 | 20261545 | 191 | 924 | 8 | 5 | 3 |
| 420 | 20261546 | 258 | 1253 | 35 | 9 | 3 |
| 421 | 20261547 | 225 | 1419 | 25 | 1 | 2 |
| 422 | 20261548 | 227 | 1088 | 23 | 2 | 1 |
| 423 | 20261549 | 250 | 1722 | 18 | 6 | 1 |
| 424 | 20261550 | 271 | 1677 | 20 | 1 | 3 |
| 425 | 20261551 | 268 | 1343 | 22 | 4 | 1 |
| 426 | 20261552 | 299 | 1877 | 38 | 5 | 3 |
| 427 | 20261553 | 311 | 2173 | 25 | 1 | 3 |
| 428 | 20261554 | 240 | 1385 | 29 | 4 | 3 |
| 429 | 20261555 | 251 | 1233 | 22 | 6 | 4 |
| 430 | 20261556 | 213 | 1024 | 12 | 2 | 3 |
| 431 | 20261557 | 216 | 979 | 27 | 5 | 5 |
| 432 | 20261558 | 271 | 1553 | 24 | 3 | 2 |
| 433 | 20261559 | 247 | 1342 | 24 | 5 | 2 |
| 434 | 20261560 | 201 | 1336 | 19 | 0 | 2 |
| 435 | 20261561 | 255 | 1290 | 30 | 3 | 1 |
| 436 | 20261562 | 345 | 1483 | 31 | 4 | 4 |
| 437 | 20261563 | 202 | 980 | 16 | 7 | 3 |
| 438 | 20261564 | 256 | 1298 | 27 | 3 | 1 |
| 439 | 20261565 | 248 | 1268 | 22 | 6 | 2 |
| 440 | 20261566 | 259 | 1054 | 22 | 9 | 3 |
| 441 | 20261567 | 211 | 915 | 19 | 9 | 3 |
| 442 | 20261568 | 264 | 1413 | 21 | 7 | 0 |
| 443 | 20261569 | 214 | 1061 | 25 | 4 | 5 |
| 444 | 20261570 | 285 | 1540 | 31 | 6 | 3 |
| 445 | 20261571 | 229 | 1243 | 23 | 10 | 2 |
| 446 | 20261572 | 258 | 1421 | 34 | 11 | 1 |
| 447 | 20261573 | 288 | 1750 | 50 | 2 | 3 |
| 448 | 20261574 | 248 | 994 | 19 | 5 | 4 |
| 449 | 20261575 | 257 | 1296 | 26 | 5 | 5 |
| 450 | 20261576 | 276 | 1421 | 27 | 6 | 5 |
| 451 | 20261577 | 215 | 1003 | 22 | 4 | 4 |
| 452 | 20261578 | 426 | 2577 | 34 | 8 | 4 |
| 453 | 20261579 | 280 | 1427 | 29 | 7 | 3 |
| 454 | 20261580 | 368 | 1899 | 44 | 7 | 4 |
| 455 | 20261581 | 179 | 854 | 17 | 5 | 1 |
| 456 | 20261582 | 241 | 1359 | 11 | 5 | 3 |
| 457 | 20261583 | 271 | 1351 | 19 | 6 | 3 |
| 458 | 20261584 | 301 | 1654 | 37 | 6 | 2 |
| 459 | 20261585 | 206 | 1065 | 29 | 1 | 0 |
| 460 | 20261586 | 179 | 917 | 14 | 2 | 1 |
| 461 | 20261587 | 251 | 1321 | 20 | 2 | 2 |
| 462 | 20261588 | 206 | 1002 | 21 | 0 | 3 |
| 463 | 20261589 | 254 | 1390 | 19 | 3 | 3 |
| 464 | 20261590 | 202 | 976 | 14 | 3 | 3 |
| 465 | 20261591 | 284 | 1676 | 31 | 0 | 3 |
| 466 | 20261592 | 235 | 1427 | 26 | 3 | 2 |
| 467 | 20261593 | 241 | 1583 | 23 | 6 | 2 |
| 468 | 20261594 | 234 | 1281 | 20 | 2 | 4 |
| 469 | 20261595 | 235 | 1331 | 29 | 4 | 3 |
| 470 | 20261596 | 213 | 894 | 23 | 5 | 3 |
| 471 | 20261597 | 412 | 2748 | 35 | 1 | 3 |
| 472 | 20261598 | 259 | 1213 | 28 | 10 | 2 |
| 473 | 20261599 | 235 | 1186 | 16 | 1 | 3 |
| 474 | 20261600 | 242 | 1169 | 22 | 1 | 2 |
| 475 | 20261601 | 306 | 1714 | 32 | 6 | 3 |
| 476 | 20261602 | 180 | 772 | 17 | 2 | 3 |
| 477 | 20261603 | 293 | 1259 | 34 | 2 | 4 |
| 478 | 20261604 | 242 | 1172 | 15 | 5 | 5 |
| 479 | 20261605 | 240 | 1437 | 29 | 4 | 3 |
| 480 | 20261606 | 246 | 1314 | 27 | 8 | 2 |
| 481 | 20261607 | 214 | 904 | 19 | 6 | 2 |
| 482 | 20261608 | 240 | 1312 | 18 | 3 | 2 |
| 483 | 20261609 | 218 | 1011 | 17 | 3 | 1 |
| 484 | 20261610 | 199 | 935 | 23 | 5 | 2 |
| 485 | 20261611 | 277 | 1302 | 29 | 10 | 5 |
| 486 | 20261612 | 156 | 513 | 9 | 1 | 2 |
| 487 | 20261613 | 258 | 1134 | 19 | 2 | 4 |
| 488 | 20261614 | 329 | 1608 | 30 | 5 | 2 |
| 489 | 20261615 | 206 | 971 | 11 | 9 | 2 |
| 490 | 20261616 | 291 | 1518 | 36 | 7 | 3 |
| 491 | 20261617 | 252 | 1208 | 16 | 2 | 4 |
| 492 | 20261618 | 183 | 920 | 20 | 2 | 2 |
| 493 | 20261619 | 232 | 1087 | 31 | 5 | 3 |
| 494 | 20261620 | 206 | 1186 | 27 | 2 | 2 |
| 495 | 20261621 | 259 | 1364 | 31 | 0 | 1 |
| 496 | 20261622 | 205 | 660 | 12 | 3 | 2 |
| 497 | 20261623 | 252 | 1415 | 25 | 2 | 2 |
| 498 | 20261624 | 214 | 932 | 16 | 8 | 2 |
| 499 | 20261625 | 204 | 995 | 14 | 7 | 1 |
| 500 | 20261626 | 276 | 990 | 28 | 0 | 3 |
| 501 | 20261627 | 197 | 922 | 20 | 1 | 2 |
| 502 | 20261628 | 268 | 1134 | 18 | 3 | 4 |
| 503 | 20261629 | 198 | 849 | 18 | 2 | 1 |
| 504 | 20261630 | 276 | 1417 | 16 | 6 | 5 |
| 505 | 20261631 | 229 | 1122 | 19 | 0 | 2 |
| 506 | 20261632 | 219 | 1000 | 25 | 2 | 2 |
| 507 | 20261633 | 242 | 1415 | 20 | 7 | 1 |
| 508 | 20261634 | 192 | 1079 | 20 | 3 | 1 |
| 509 | 20261635 | 244 | 1204 | 26 | 6 | 3 |
| 510 | 20261636 | 257 | 1133 | 26 | 4 | 2 |
| 511 | 20261637 | 236 | 1151 | 23 | 4 | 4 |
| 512 | 20261638 | 273 | 1922 | 32 | 1 | 0 |
| 513 | 20261639 | 236 | 1149 | 24 | 5 | 2 |
| 514 | 20261640 | 255 | 1460 | 22 | 1 | 1 |
| 515 | 20261641 | 277 | 1530 | 27 | 2 | 5 |
| 516 | 20261642 | 227 | 1304 | 14 | 7 | 2 |
| 517 | 20261643 | 259 | 1391 | 22 | 4 | 3 |
| 518 | 20261644 | 248 | 867 | 13 | 0 | 2 |
| 519 | 20261645 | 176 | 949 | 16 | 3 | 4 |
| 520 | 20261646 | 270 | 1146 | 29 | 8 | 5 |
| 521 | 20261647 | 212 | 840 | 25 | 5 | 2 |
| 522 | 20261648 | 248 | 1098 | 23 | 5 | 3 |
| 523 | 20261649 | 231 | 1273 | 28 | 9 | 3 |
| 524 | 20261650 | 199 | 1135 | 19 | 5 | 1 |
| 525 | 20261651 | 294 | 1521 | 28 | 5 | 3 |
| 526 | 20261652 | 233 | 1399 | 11 | 5 | 4 |
| 527 | 20261653 | 236 | 1476 | 27 | 1 | 3 |
| 528 | 20261654 | 290 | 1150 | 18 | 5 | 2 |
| 529 | 20261655 | 236 | 1447 | 21 | 6 | 1 |
| 530 | 20261656 | 171 | 767 | 16 | 5 | 2 |
| 531 | 20261657 | 258 | 1058 | 17 | 3 | 3 |
| 532 | 20261658 | 184 | 732 | 10 | 3 | 1 |
| 533 | 20261659 | 255 | 1220 | 23 | 1 | 1 |
| 534 | 20261660 | 210 | 676 | 21 | 6 | 3 |
| 535 | 20261661 | 249 | 1217 | 21 | 6 | 1 |
| 536 | 20261662 | 294 | 1582 | 31 | 0 | 3 |
| 537 | 20261663 | 212 | 1063 | 28 | 6 | 2 |
| 538 | 20261664 | 305 | 1678 | 42 | 5 | 6 |
| 539 | 20261665 | 260 | 1374 | 39 | 9 | 1 |
| 540 | 20261666 | 246 | 1281 | 38 | 6 | 3 |
| 541 | 20261667 | 230 | 1109 | 34 | 1 | 4 |
| 542 | 20261668 | 256 | 1229 | 15 | 4 | 2 |
| 543 | 20261669 | 169 | 784 | 16 | 4 | 1 |
| 544 | 20261670 | 227 | 957 | 19 | 8 | 4 |
| 545 | 20261671 | 267 | 1398 | 32 | 7 | 4 |
| 546 | 20261672 | 247 | 1118 | 23 | 3 | 2 |
| 547 | 20261673 | 266 | 1681 | 25 | 6 | 5 |
| 548 | 20261674 | 223 | 1324 | 31 | 7 | 3 |
| 549 | 20261675 | 230 | 1290 | 20 | 5 | 2 |
| 550 | 20261676 | 228 | 1227 | 8 | 4 | 4 |
| 551 | 20261677 | 201 | 892 | 23 | 3 | 3 |
| 552 | 20261678 | 227 | 1287 | 14 | 4 | 1 |
| 553 | 20261679 | 173 | 902 | 9 | 4 | 1 |
| 554 | 20261680 | 294 | 1533 | 30 | 9 | 3 |
| 555 | 20261681 | 170 | 632 | 4 | 2 | 1 |
| 556 | 20261682 | 260 | 945 | 20 | 8 | 2 |
| 557 | 20261683 | 191 | 977 | 20 | 0 | 4 |
| 558 | 20261684 | 161 | 730 | 12 | 1 | 0 |
| 559 | 20261685 | 185 | 814 | 12 | 4 | 3 |
| 560 | 20261686 | 194 | 1054 | 25 | 3 | 2 |
| 561 | 20261687 | 301 | 1780 | 33 | 2 | 3 |
| 562 | 20261688 | 321 | 1676 | 29 | 3 | 2 |
| 563 | 20261689 | 217 | 1261 | 24 | 2 | 2 |
| 564 | 20261690 | 294 | 1351 | 13 | 6 | 4 |
| 565 | 20261691 | 279 | 1028 | 29 | 5 | 5 |
| 566 | 20261692 | 193 | 1151 | 26 | 3 | 1 |
| 567 | 20261693 | 289 | 1713 | 12 | 3 | 4 |
| 568 | 20261694 | 169 | 574 | 14 | 2 | 3 |
| 569 | 20261695 | 175 | 572 | 15 | 2 | 3 |
| 570 | 20261696 | 202 | 1090 | 23 | 3 | 4 |
| 571 | 20261697 | 321 | 1785 | 15 | 6 | 4 |
| 572 | 20261698 | 209 | 932 | 20 | 7 | 2 |
| 573 | 20261699 | 216 | 948 | 31 | 2 | 4 |
| 574 | 20261700 | 220 | 1059 | 27 | 4 | 3 |
| 575 | 20261701 | 301 | 1451 | 16 | 2 | 3 |
| 576 | 20261702 | 274 | 1525 | 32 | 3 | 4 |
| 577 | 20261703 | 197 | 1040 | 27 | 3 | 1 |
| 578 | 20261704 | 230 | 1152 | 28 | 7 | 3 |
| 579 | 20261705 | 256 | 1353 | 29 | 6 | 3 |
| 580 | 20261706 | 307 | 1732 | 42 | 3 | 3 |
| 581 | 20261707 | 196 | 844 | 25 | 2 | 3 |
| 582 | 20261708 | 191 | 817 | 11 | 2 | 2 |
| 583 | 20261709 | 237 | 1277 | 22 | 1 | 3 |
| 584 | 20261710 | 285 | 1231 | 26 | 9 | 4 |
| 585 | 20261711 | 267 | 1507 | 33 | 11 | 4 |
| 586 | 20261712 | 242 | 1174 | 26 | 5 | 3 |
| 587 | 20261713 | 294 | 1454 | 25 | 2 | 3 |
| 588 | 20261714 | 252 | 1475 | 29 | 2 | 4 |
| 589 | 20261715 | 200 | 755 | 11 | 4 | 2 |
| 590 | 20261716 | 365 | 1812 | 23 | 3 | 4 |
| 591 | 20261717 | 222 | 1178 | 15 | 3 | 3 |
| 592 | 20261718 | 224 | 1191 | 23 | 4 | 2 |
| 593 | 20261719 | 186 | 948 | 20 | 2 | 2 |
| 594 | 20261720 | 267 | 1556 | 16 | 5 | 3 |
| 595 | 20261721 | 183 | 831 | 15 | 4 | 3 |
| 596 | 20261722 | 265 | 1249 | 26 | 1 | 3 |
| 597 | 20261723 | 197 | 1129 | 27 | 7 | 2 |
| 598 | 20261724 | 294 | 1559 | 27 | 2 | 5 |
| 599 | 20261725 | 279 | 1722 | 26 | 4 | 1 |
| 600 | 20261726 | 266 | 1238 | 41 | 5 | 2 |
| 601 | 20261727 | 221 | 1083 | 21 | 5 | 1 |
| 602 | 20261728 | 348 | 2077 | 47 | 1 | 3 |
| 603 | 20261729 | 218 | 919 | 16 | 3 | 3 |
| 604 | 20261730 | 245 | 1169 | 30 | 3 | 4 |
| 605 | 20261731 | 284 | 1527 | 30 | 4 | 4 |
| 606 | 20261732 | 222 | 1018 | 23 | 3 | 1 |
| 607 | 20261733 | 238 | 1013 | 25 | 4 | 3 |
| 608 | 20261734 | 331 | 1613 | 26 | 4 | 1 |
| 609 | 20261735 | 315 | 1905 | 27 | 11 | 4 |
| 610 | 20261736 | 177 | 486 | 6 | 1 | 2 |
| 611 | 20261737 | 220 | 1125 | 27 | 2 | 2 |
| 612 | 20261738 | 193 | 1043 | 18 | 3 | 0 |
| 613 | 20261739 | 194 | 959 | 15 | 5 | 2 |
| 614 | 20261740 | 207 | 1101 | 19 | 0 | 4 |
| 615 | 20261741 | 271 | 1335 | 19 | 6 | 2 |
| 616 | 20261742 | 248 | 1327 | 30 | 4 | 2 |
| 617 | 20261743 | 247 | 1421 | 15 | 6 | 2 |
| 618 | 20261744 | 189 | 815 | 16 | 3 | 4 |
| 619 | 20261745 | 239 | 1211 | 26 | 4 | 5 |
| 620 | 20261746 | 269 | 1191 | 19 | 3 | 1 |
| 621 | 20261747 | 241 | 1071 | 16 | 6 | 3 |
| 622 | 20261748 | 288 | 1391 | 45 | 5 | 4 |
| 623 | 20261749 | 231 | 1276 | 20 | 7 | 2 |
| 624 | 20261750 | 203 | 971 | 20 | 2 | 3 |
| 625 | 20261751 | 326 | 1812 | 27 | 6 | 5 |
| 626 | 20261752 | 258 | 1555 | 14 | 4 | 4 |
| 627 | 20261753 | 273 | 1368 | 13 | 7 | 2 |
| 628 | 20261754 | 231 | 1120 | 9 | 4 | 5 |
| 629 | 20261755 | 167 | 854 | 13 | 3 | 1 |
| 630 | 20261756 | 184 | 875 | 13 | 1 | 2 |
| 631 | 20261757 | 289 | 1648 | 27 | 3 | 2 |
| 632 | 20261758 | 272 | 1589 | 24 | 12 | 3 |
| 633 | 20261759 | 275 | 1239 | 16 | 8 | 2 |
| 634 | 20261760 | 232 | 895 | 15 | 0 | 3 |
| 635 | 20261761 | 292 | 1320 | 40 | 1 | 5 |
| 636 | 20261762 | 208 | 660 | 11 | 2 | 3 |
| 637 | 20261763 | 169 | 676 | 11 | 4 | 1 |
| 638 | 20261764 | 269 | 1459 | 31 | 2 | 2 |
| 639 | 20261765 | 293 | 1531 | 33 | 5 | 2 |
| 640 | 20261766 | 356 | 1764 | 26 | 1 | 3 |
| 641 | 20261767 | 251 | 1629 | 41 | 8 | 2 |
| 642 | 20261768 | 248 | 1194 | 20 | 6 | 2 |
| 643 | 20261769 | 281 | 1582 | 22 | 2 | 3 |
| 644 | 20261770 | 208 | 950 | 18 | 11 | 3 |
| 645 | 20261771 | 183 | 709 | 16 | 3 | 2 |
| 646 | 20261772 | 226 | 1252 | 15 | 10 | 3 |
| 647 | 20261773 | 256 | 1093 | 26 | 4 | 4 |
| 648 | 20261774 | 219 | 1115 | 19 | 3 | 1 |
| 649 | 20261775 | 289 | 1517 | 24 | 1 | 5 |
| 650 | 20261776 | 279 | 1590 | 34 | 4 | 4 |
| 651 | 20261777 | 184 | 654 | 16 | 4 | 2 |
| 652 | 20261778 | 223 | 1071 | 23 | 4 | 3 |
| 653 | 20261779 | 253 | 1030 | 33 | 6 | 3 |
| 654 | 20261780 | 229 | 1037 | 14 | 3 | 2 |
| 655 | 20261781 | 229 | 1290 | 30 | 3 | 3 |
| 656 | 20261782 | 253 | 1131 | 30 | 5 | 5 |
| 657 | 20261783 | 237 | 1358 | 23 | 11 | 0 |
| 658 | 20261784 | 249 | 1171 | 23 | 4 | 2 |
| 659 | 20261785 | 387 | 1948 | 48 | 2 | 5 |
| 660 | 20261786 | 174 | 895 | 18 | 3 | 1 |
| 661 | 20261787 | 276 | 1489 | 33 | 3 | 1 |
| 662 | 20261788 | 281 | 1532 | 30 | 1 | 4 |
| 663 | 20261789 | 235 | 1058 | 22 | 5 | 3 |
| 664 | 20261790 | 169 | 872 | 12 | 4 | 1 |
| 665 | 20261791 | 224 | 1157 | 15 | 3 | 3 |
| 666 | 20261792 | 232 | 1146 | 19 | 2 | 3 |
| 667 | 20261793 | 209 | 791 | 17 | 3 | 1 |
| 668 | 20261794 | 500 | 2453 | 66 | 9 | 5 |
| 669 | 20261795 | 343 | 1718 | 22 | 5 | 5 |
| 670 | 20261796 | 303 | 1573 | 28 | 6 | 4 |
| 671 | 20261797 | 258 | 1550 | 18 | 4 | 2 |
| 672 | 20261798 | 171 | 875 | 18 | 6 | 2 |
| 673 | 20261799 | 180 | 651 | 16 | 3 | 2 |
| 674 | 20261800 | 265 | 1057 | 26 | 10 | 5 |
| 675 | 20261801 | 178 | 725 | 13 | 3 | 2 |
| 676 | 20261802 | 201 | 1014 | 17 | 2 | 2 |
| 677 | 20261803 | 232 | 1377 | 34 | 4 | 1 |
| 678 | 20261804 | 254 | 1478 | 32 | 4 | 1 |
| 679 | 20261805 | 229 | 1137 | 14 | 2 | 3 |
| 680 | 20261806 | 183 | 958 | 14 | 0 | 2 |
| 681 | 20261807 | 235 | 1262 | 27 | 2 | 3 |
| 682 | 20261808 | 282 | 1185 | 39 | 6 | 4 |
| 683 | 20261809 | 233 | 1334 | 22 | 12 | 1 |
| 684 | 20261810 | 217 | 861 | 27 | 3 | 2 |
| 685 | 20261811 | 361 | 1759 | 36 | 4 | 3 |
| 686 | 20261812 | 228 | 829 | 21 | 7 | 2 |
| 687 | 20261813 | 230 | 1017 | 23 | 1 | 3 |
| 688 | 20261814 | 247 | 1256 | 18 | 6 | 3 |
| 689 | 20261815 | 220 | 999 | 19 | 1 | 2 |
| 690 | 20261816 | 263 | 1287 | 26 | 2 | 2 |
| 691 | 20261817 | 276 | 1463 | 31 | 2 | 4 |
| 692 | 20261818 | 267 | 1742 | 32 | 5 | 1 |
| 693 | 20261819 | 301 | 1250 | 28 | 3 | 5 |
| 694 | 20261820 | 226 | 842 | 15 | 5 | 2 |
| 695 | 20261821 | 185 | 623 | 17 | 5 | 3 |
| 696 | 20261822 | 239 | 1209 | 27 | 4 | 5 |
| 697 | 20261823 | 202 | 1003 | 25 | 5 | 3 |
| 698 | 20261824 | 196 | 710 | 15 | 5 | 2 |
| 699 | 20261825 | 256 | 1482 | 30 | 4 | 2 |
| 700 | 20261826 | 246 | 1354 | 32 | 3 | 4 |
| 701 | 20261827 | 482 | 2280 | 57 | 2 | 5 |
| 702 | 20261828 | 365 | 1881 | 36 | 4 | 2 |
| 703 | 20261829 | 319 | 1413 | 19 | 4 | 3 |
| 704 | 20261830 | 248 | 1384 | 21 | 2 | 3 |
| 705 | 20261831 | 248 | 1381 | 25 | 3 | 3 |
| 706 | 20261832 | 280 | 1482 | 24 | 2 | 2 |
| 707 | 20261833 | 173 | 905 | 11 | 2 | 2 |
| 708 | 20261834 | 240 | 1099 | 14 | 7 | 0 |
| 709 | 20261835 | 258 | 1374 | 13 | 4 | 2 |
| 710 | 20261836 | 280 | 1282 | 26 | 2 | 1 |
| 711 | 20261837 | 284 | 1738 | 40 | 3 | 4 |
| 712 | 20261838 | 164 | 634 | 8 | 6 | 1 |
| 713 | 20261839 | 253 | 1379 | 22 | 3 | 2 |
| 714 | 20261840 | 232 | 982 | 11 | 1 | 4 |
| 715 | 20261841 | 241 | 1297 | 19 | 4 | 3 |
| 716 | 20261842 | 276 | 1507 | 24 | 5 | 2 |
| 717 | 20261843 | 244 | 1268 | 22 | 7 | 3 |
| 718 | 20261844 | 227 | 1038 | 20 | 7 | 2 |
| 719 | 20261845 | 270 | 1570 | 34 | 2 | 0 |
| 720 | 20261846 | 205 | 902 | 15 | 3 | 3 |
| 721 | 20261847 | 187 | 1024 | 17 | 6 | 2 |
| 722 | 20261848 | 222 | 722 | 22 | 2 | 4 |
| 723 | 20261849 | 242 | 1232 | 35 | 7 | 1 |
| 724 | 20261850 | 247 | 1273 | 24 | 7 | 2 |
| 725 | 20261851 | 260 | 1393 | 24 | 3 | 2 |
| 726 | 20261852 | 226 | 1202 | 23 | 6 | 2 |
| 727 | 20261853 | 188 | 1000 | 14 | 2 | 2 |
| 728 | 20261854 | 279 | 1514 | 36 | 6 | 4 |
| 729 | 20261855 | 302 | 2231 | 32 | 1 | 0 |
| 730 | 20261856 | 406 | 2002 | 63 | 7 | 4 |
| 731 | 20261857 | 195 | 914 | 12 | 3 | 1 |
| 732 | 20261858 | 244 | 1410 | 17 | 1 | 3 |
| 733 | 20261859 | 257 | 1402 | 19 | 0 | 1 |
| 734 | 20261860 | 206 | 1044 | 17 | 7 | 1 |
| 735 | 20261861 | 230 | 1318 | 22 | 3 | 3 |
| 736 | 20261862 | 246 | 1364 | 31 | 5 | 1 |
| 737 | 20261863 | 342 | 1883 | 29 | 2 | 6 |
| 738 | 20261864 | 271 | 1250 | 35 | 2 | 4 |
| 739 | 20261865 | 379 | 1775 | 42 | 2 | 4 |
| 740 | 20261866 | 201 | 1055 | 13 | 5 | 1 |
| 741 | 20261867 | 285 | 1668 | 20 | 7 | 1 |
| 742 | 20261868 | 261 | 1246 | 21 | 6 | 3 |
| 743 | 20261869 | 272 | 1237 | 27 | 2 | 1 |
| 744 | 20261870 | 281 | 1220 | 22 | 1 | 4 |
| 745 | 20261871 | 210 | 1087 | 15 | 2 | 1 |
| 746 | 20261872 | 240 | 1239 | 20 | 2 | 2 |
| 747 | 20261873 | 196 | 1049 | 26 | 3 | 3 |
| 748 | 20261874 | 332 | 2194 | 23 | 5 | 4 |
| 749 | 20261875 | 277 | 1182 | 26 | 2 | 2 |
| 750 | 20261876 | 294 | 1725 | 48 | 4 | 4 |
| 751 | 20261877 | 159 | 644 | 10 | 1 | 1 |
| 752 | 20261878 | 189 | 844 | 12 | 9 | 1 |
| 753 | 20261879 | 290 | 1674 | 27 | 4 | 4 |
| 754 | 20261880 | 285 | 1811 | 21 | 7 | 3 |
| 755 | 20261881 | 290 | 1253 | 34 | 4 | 4 |
| 756 | 20261882 | 218 | 1346 | 19 | 0 | 2 |
| 757 | 20261883 | 249 | 1163 | 25 | 5 | 3 |
| 758 | 20261884 | 287 | 1625 | 43 | 7 | 3 |
| 759 | 20261885 | 225 | 1183 | 29 | 6 | 3 |
| 760 | 20261886 | 233 | 1138 | 32 | 2 | 3 |
| 761 | 20261887 | 369 | 1771 | 37 | 3 | 2 |
| 762 | 20261888 | 201 | 1041 | 21 | 1 | 1 |
| 763 | 20261889 | 247 | 1489 | 23 | 2 | 3 |
| 764 | 20261890 | 296 | 1409 | 22 | 7 | 6 |
| 765 | 20261891 | 275 | 1205 | 31 | 6 | 2 |
| 766 | 20261892 | 224 | 886 | 20 | 3 | 1 |
| 767 | 20261893 | 293 | 1511 | 34 | 4 | 5 |
| 768 | 20261894 | 227 | 1096 | 10 | 8 | 3 |
| 769 | 20261895 | 276 | 1389 | 33 | 3 | 5 |
| 770 | 20261896 | 186 | 772 | 15 | 3 | 1 |
| 771 | 20261897 | 226 | 1081 | 22 | 3 | 2 |
| 772 | 20261898 | 228 | 985 | 20 | 4 | 2 |
| 773 | 20261899 | 314 | 1808 | 24 | 0 | 2 |
| 774 | 20261900 | 288 | 1193 | 20 | 9 | 5 |
| 775 | 20261901 | 281 | 1176 | 37 | 4 | 4 |
| 776 | 20261902 | 282 | 1687 | 22 | 15 | 2 |
| 777 | 20261903 | 359 | 1790 | 35 | 8 | 4 |
| 778 | 20261904 | 298 | 1971 | 41 | 8 | 2 |
| 779 | 20261905 | 207 | 928 | 22 | 2 | 2 |
| 780 | 20261906 | 284 | 1383 | 18 | 4 | 2 |
| 781 | 20261907 | 232 | 1100 | 27 | 9 | 3 |
| 782 | 20261908 | 228 | 1122 | 18 | 8 | 3 |
| 783 | 20261909 | 164 | 840 | 16 | 3 | 2 |
| 784 | 20261910 | 229 | 1188 | 25 | 4 | 2 |
| 785 | 20261911 | 245 | 1211 | 29 | 1 | 4 |
| 786 | 20261912 | 355 | 1588 | 32 | 1 | 4 |
| 787 | 20261913 | 218 | 680 | 11 | 11 | 1 |
| 788 | 20261914 | 256 | 1396 | 17 | 6 | 3 |
| 789 | 20261915 | 212 | 1011 | 22 | 3 | 2 |
| 790 | 20261916 | 213 | 977 | 19 | 2 | 1 |
| 791 | 20261917 | 255 | 1405 | 37 | 2 | 1 |
| 792 | 20261918 | 259 | 855 | 18 | 6 | 2 |
| 793 | 20261919 | 235 | 1230 | 17 | 4 | 1 |
| 794 | 20261920 | 245 | 1432 | 17 | 2 | 3 |
| 795 | 20261921 | 238 | 1043 | 18 | 3 | 2 |
| 796 | 20261922 | 239 | 1170 | 26 | 7 | 5 |
| 797 | 20261923 | 264 | 1340 | 17 | 2 | 3 |
| 798 | 20261924 | 209 | 1061 | 27 | 5 | 2 |
| 799 | 20261925 | 259 | 1254 | 32 | 1 | 3 |
| 800 | 20261926 | 294 | 1557 | 25 | 2 | 3 |
| 801 | 20261927 | 285 | 1441 | 27 | 10 | 2 |
| 802 | 20261928 | 181 | 809 | 14 | 2 | 4 |
| 803 | 20261929 | 225 | 1027 | 18 | 0 | 2 |
| 804 | 20261930 | 271 | 1274 | 17 | 2 | 4 |
| 805 | 20261931 | 266 | 1190 | 22 | 5 | 4 |
| 806 | 20261932 | 263 | 1513 | 22 | 1 | 4 |
| 807 | 20261933 | 306 | 1749 | 37 | 7 | 4 |
| 808 | 20261934 | 225 | 1102 | 12 | 3 | 3 |
| 809 | 20261935 | 221 | 1145 | 17 | 8 | 4 |
| 810 | 20261936 | 239 | 1240 | 24 | 5 | 1 |
| 811 | 20261937 | 239 | 1459 | 26 | 2 | 3 |
| 812 | 20261938 | 241 | 1318 | 29 | 3 | 2 |
| 813 | 20261939 | 206 | 752 | 23 | 2 | 3 |
| 814 | 20261940 | 254 | 1314 | 23 | 3 | 2 |
| 815 | 20261941 | 232 | 1149 | 31 | 6 | 3 |
| 816 | 20261942 | 228 | 1306 | 17 | 5 | 3 |
| 817 | 20261943 | 238 | 1101 | 39 | 3 | 5 |
| 818 | 20261944 | 249 | 1510 | 18 | 3 | 5 |
| 819 | 20261945 | 295 | 1752 | 21 | 5 | 5 |
| 820 | 20261946 | 243 | 1227 | 16 | 6 | 3 |
| 821 | 20261947 | 218 | 895 | 24 | 4 | 4 |
| 822 | 20261948 | 248 | 1429 | 15 | 3 | 1 |
| 823 | 20261949 | 211 | 1133 | 17 | 4 | 1 |
| 824 | 20261950 | 339 | 1696 | 27 | 0 | 4 |
| 825 | 20261951 | 295 | 1342 | 38 | 6 | 4 |
| 826 | 20261952 | 213 | 832 | 14 | 1 | 2 |
| 827 | 20261953 | 222 | 1063 | 17 | 5 | 2 |
| 828 | 20261954 | 185 | 861 | 24 | 1 | 3 |
| 829 | 20261955 | 215 | 1290 | 24 | 9 | 2 |
| 830 | 20261956 | 199 | 822 | 21 | 4 | 4 |
| 831 | 20261957 | 287 | 1373 | 33 | 0 | 4 |
| 832 | 20261958 | 231 | 1183 | 29 | 1 | 3 |
| 833 | 20261959 | 205 | 800 | 19 | 3 | 2 |
| 834 | 20261960 | 250 | 1070 | 14 | 3 | 4 |
| 835 | 20261961 | 233 | 989 | 17 | 2 | 2 |
| 836 | 20261962 | 254 | 1101 | 24 | 5 | 3 |
| 837 | 20261963 | 237 | 1098 | 21 | 5 | 3 |
| 838 | 20261964 | 240 | 1416 | 39 | 4 | 2 |
| 839 | 20261965 | 223 | 1000 | 15 | 4 | 1 |
| 840 | 20261966 | 253 | 1209 | 25 | 4 | 1 |
| 841 | 20261967 | 186 | 663 | 8 | 3 | 1 |
| 842 | 20261968 | 314 | 1736 | 31 | 7 | 4 |
| 843 | 20261969 | 250 | 1605 | 30 | 3 | 1 |
| 844 | 20261970 | 197 | 1117 | 13 | 3 | 4 |
| 845 | 20261971 | 213 | 1098 | 16 | 3 | 3 |
| 846 | 20261972 | 250 | 1369 | 12 | 4 | 1 |
| 847 | 20261973 | 242 | 1243 | 23 | 3 | 1 |
| 848 | 20261974 | 344 | 2022 | 32 | 4 | 3 |
| 849 | 20261975 | 207 | 947 | 9 | 7 | 2 |
| 850 | 20261976 | 172 | 691 | 19 | 5 | 3 |
| 851 | 20261977 | 234 | 1398 | 36 | 4 | 2 |
| 852 | 20261978 | 373 | 2132 | 44 | 6 | 4 |
| 853 | 20261979 | 222 | 852 | 15 | 1 | 3 |
| 854 | 20261980 | 240 | 1308 | 18 | 7 | 2 |
| 855 | 20261981 | 189 | 900 | 17 | 5 | 2 |
| 856 | 20261982 | 269 | 1138 | 20 | 3 | 5 |
| 857 | 20261983 | 272 | 1374 | 38 | 7 | 3 |
| 858 | 20261984 | 224 | 827 | 22 | 0 | 3 |
| 859 | 20261985 | 228 | 1456 | 20 | 10 | 3 |
| 860 | 20261986 | 233 | 1101 | 18 | 3 | 4 |
| 861 | 20261987 | 233 | 1559 | 22 | 3 | 2 |
| 862 | 20261988 | 227 | 1434 | 18 | 6 | 3 |
| 863 | 20261989 | 189 | 799 | 17 | 4 | 0 |
| 864 | 20261990 | 203 | 937 | 17 | 1 | 3 |
| 865 | 20261991 | 185 | 765 | 25 | 3 | 1 |
| 866 | 20261992 | 249 | 1266 | 32 | 0 | 4 |
| 867 | 20261993 | 199 | 824 | 20 | 7 | 1 |
| 868 | 20261994 | 258 | 1278 | 9 | 3 | 4 |
| 869 | 20261995 | 192 | 767 | 18 | 1 | 2 |
| 870 | 20261996 | 213 | 1278 | 24 | 10 | 0 |
| 871 | 20261997 | 181 | 1015 | 12 | 7 | 1 |
| 872 | 20261998 | 233 | 1271 | 23 | 2 | 3 |
| 873 | 20261999 | 214 | 967 | 13 | 3 | 2 |
| 874 | 20262000 | 182 | 976 | 10 | 4 | 0 |
| 875 | 20262001 | 221 | 1067 | 21 | 4 | 1 |
| 876 | 20262002 | 279 | 1055 | 17 | 10 | 3 |
| 877 | 20262003 | 266 | 1472 | 26 | 8 | 3 |
| 878 | 20262004 | 211 | 999 | 17 | 1 | 2 |
| 879 | 20262005 | 272 | 1476 | 29 | 6 | 5 |
| 880 | 20262006 | 192 | 784 | 9 | 3 | 3 |
| 881 | 20262007 | 181 | 655 | 12 | 7 | 1 |
| 882 | 20262008 | 196 | 1197 | 10 | 2 | 2 |
| 883 | 20262009 | 500 | 2328 | 67 | 6 | 6 |
| 884 | 20262010 | 361 | 1911 | 10 | 8 | 1 |
| 885 | 20262011 | 250 | 1142 | 15 | 4 | 1 |
| 886 | 20262012 | 239 | 927 | 28 | 7 | 4 |
| 887 | 20262013 | 277 | 1189 | 25 | 5 | 4 |
| 888 | 20262014 | 143 | 508 | 9 | 2 | 2 |
| 889 | 20262015 | 312 | 1957 | 39 | 1 | 4 |
| 890 | 20262016 | 230 | 1165 | 22 | 1 | 3 |
| 891 | 20262017 | 205 | 1033 | 19 | 5 | 3 |
| 892 | 20262018 | 246 | 914 | 11 | 1 | 3 |
| 893 | 20262019 | 235 | 1336 | 25 | 2 | 3 |
| 894 | 20262020 | 255 | 1211 | 29 | 4 | 3 |
| 895 | 20262021 | 219 | 1126 | 23 | 5 | 2 |
| 896 | 20262022 | 244 | 828 | 18 | 5 | 4 |
| 897 | 20262023 | 299 | 1662 | 36 | 4 | 4 |
| 898 | 20262024 | 434 | 2221 | 38 | 7 | 2 |
| 899 | 20262025 | 272 | 1148 | 19 | 6 | 4 |
| 900 | 20262026 | 219 | 954 | 26 | 3 | 2 |
| 901 | 20262027 | 343 | 1670 | 26 | 9 | 3 |
| 902 | 20262028 | 209 | 757 | 13 | 3 | 3 |
| 903 | 20262029 | 331 | 2053 | 26 | 6 | 4 |
| 904 | 20262030 | 253 | 1267 | 31 | 6 | 2 |
| 905 | 20262031 | 315 | 2243 | 30 | 6 | 3 |
| 906 | 20262032 | 275 | 1205 | 22 | 7 | 3 |
| 907 | 20262033 | 254 | 1176 | 34 | 4 | 2 |
| 908 | 20262034 | 243 | 1091 | 25 | 5 | 4 |
| 909 | 20262035 | 295 | 1953 | 33 | 2 | 2 |
| 910 | 20262036 | 261 | 1584 | 22 | 4 | 3 |
| 911 | 20262037 | 233 | 1339 | 25 | 5 | 1 |
| 912 | 20262038 | 399 | 1971 | 61 | 4 | 4 |
| 913 | 20262039 | 279 | 1829 | 26 | 1 | 3 |
| 914 | 20262040 | 218 | 1231 | 13 | 7 | 2 |
| 915 | 20262041 | 245 | 1170 | 18 | 4 | 3 |
| 916 | 20262042 | 219 | 1087 | 28 | 4 | 2 |
| 917 | 20262043 | 280 | 1696 | 34 | 9 | 2 |
| 918 | 20262044 | 272 | 1504 | 20 | 3 | 4 |
| 919 | 20262045 | 283 | 1566 | 38 | 4 | 6 |
| 920 | 20262046 | 233 | 1420 | 24 | 1 | 2 |
| 921 | 20262047 | 240 | 942 | 12 | 4 | 2 |
| 922 | 20262048 | 239 | 1025 | 22 | 3 | 3 |
| 923 | 20262049 | 269 | 1160 | 26 | 3 | 2 |
| 924 | 20262050 | 208 | 1170 | 20 | 4 | 2 |
| 925 | 20262051 | 218 | 892 | 22 | 1 | 2 |
| 926 | 20262052 | 305 | 1790 | 25 | 6 | 2 |
| 927 | 20262053 | 222 | 1155 | 6 | 2 | 4 |
| 928 | 20262054 | 188 | 820 | 10 | 2 | 1 |
| 929 | 20262055 | 167 | 821 | 10 | 2 | 2 |
| 930 | 20262056 | 227 | 1083 | 22 | 2 | 3 |
| 931 | 20262057 | 253 | 917 | 22 | 1 | 3 |
| 932 | 20262058 | 231 | 1243 | 26 | 1 | 3 |
| 933 | 20262059 | 190 | 722 | 16 | 4 | 3 |
| 934 | 20262060 | 265 | 1582 | 31 | 1 | 1 |
| 935 | 20262061 | 230 | 955 | 20 | 1 | 4 |
| 936 | 20262062 | 276 | 1634 | 32 | 3 | 2 |
| 937 | 20262063 | 308 | 1462 | 45 | 5 | 3 |
| 938 | 20262064 | 493 | 2876 | 51 | 2 | 4 |
| 939 | 20262065 | 296 | 1529 | 28 | 4 | 5 |
| 940 | 20262066 | 289 | 1581 | 37 | 10 | 3 |
| 941 | 20262067 | 247 | 1593 | 41 | 0 | 3 |
| 942 | 20262068 | 214 | 1069 | 19 | 6 | 1 |
| 943 | 20262069 | 265 | 1373 | 24 | 1 | 2 |
| 944 | 20262070 | 224 | 1120 | 26 | 2 | 2 |
| 945 | 20262071 | 207 | 1150 | 20 | 3 | 1 |
| 946 | 20262072 | 161 | 774 | 10 | 5 | 3 |
| 947 | 20262073 | 359 | 1550 | 18 | 3 | 1 |
| 948 | 20262074 | 269 | 1451 | 31 | 0 | 4 |
| 949 | 20262075 | 263 | 1171 | 17 | 10 | 2 |
| 950 | 20262076 | 258 | 1376 | 23 | 6 | 2 |
| 951 | 20262077 | 259 | 1571 | 36 | 5 | 3 |
| 952 | 20262078 | 214 | 746 | 12 | 4 | 4 |
| 953 | 20262079 | 212 | 1011 | 24 | 5 | 2 |
| 954 | 20262080 | 340 | 1993 | 43 | 5 | 3 |
| 955 | 20262081 | 243 | 1415 | 29 | 3 | 4 |
| 956 | 20262082 | 263 | 1166 | 18 | 16 | 3 |
| 957 | 20262083 | 234 | 1057 | 19 | 7 | 3 |
| 958 | 20262084 | 357 | 2188 | 44 | 2 | 2 |
| 959 | 20262085 | 216 | 1214 | 17 | 8 | 4 |
| 960 | 20262086 | 221 | 1056 | 11 | 7 | 2 |
| 961 | 20262087 | 217 | 1070 | 31 | 2 | 2 |
| 962 | 20262088 | 228 | 1191 | 22 | 4 | 3 |
| 963 | 20262089 | 221 | 947 | 17 | 3 | 3 |
| 964 | 20262090 | 244 | 1328 | 15 | 2 | 3 |
| 965 | 20262091 | 222 | 1083 | 13 | 3 | 3 |
| 966 | 20262092 | 236 | 1297 | 15 | 1 | 1 |
| 967 | 20262093 | 227 | 1240 | 21 | 2 | 3 |
| 968 | 20262094 | 285 | 1699 | 40 | 6 | 2 |
| 969 | 20262095 | 281 | 1451 | 21 | 4 | 2 |
| 970 | 20262096 | 253 | 975 | 29 | 1 | 4 |
| 971 | 20262097 | 275 | 1211 | 20 | 0 | 3 |
| 972 | 20262098 | 244 | 1431 | 35 | 6 | 2 |
| 973 | 20262099 | 311 | 1830 | 16 | 1 | 1 |
| 974 | 20262100 | 220 | 961 | 12 | 5 | 2 |
| 975 | 20262101 | 306 | 1702 | 36 | 8 | 3 |
| 976 | 20262102 | 215 | 1039 | 9 | 3 | 2 |
| 977 | 20262103 | 193 | 916 | 10 | 6 | 3 |
| 978 | 20262104 | 185 | 1078 | 11 | 3 | 0 |
| 979 | 20262105 | 215 | 930 | 21 | 1 | 3 |
| 980 | 20262106 | 281 | 1726 | 27 | 6 | 1 |
| 981 | 20262107 | 230 | 1109 | 19 | 7 | 4 |
| 982 | 20262108 | 222 | 976 | 26 | 1 | 3 |
| 983 | 20262109 | 238 | 1092 | 20 | 4 | 1 |
| 984 | 20262110 | 368 | 2124 | 37 | 13 | 3 |
| 985 | 20262111 | 298 | 1741 | 29 | 0 | 3 |
| 986 | 20262112 | 182 | 796 | 10 | 4 | 1 |
| 987 | 20262113 | 194 | 1058 | 21 | 1 | 3 |
| 988 | 20262114 | 201 | 903 | 12 | 5 | 3 |
| 989 | 20262115 | 227 | 1156 | 18 | 1 | 2 |
| 990 | 20262116 | 476 | 2212 | 37 | 10 | 4 |
| 991 | 20262117 | 246 | 1386 | 12 | 3 | 3 |
| 992 | 20262118 | 325 | 1707 | 27 | 10 | 3 |
| 993 | 20262119 | 315 | 1356 | 13 | 5 | 2 |
| 994 | 20262120 | 194 | 973 | 22 | 4 | 3 |
| 995 | 20262121 | 233 | 1109 | 17 | 3 | 3 |
| 996 | 20262122 | 225 | 886 | 14 | 3 | 1 |
| 997 | 20262123 | 230 | 1271 | 18 | 5 | 2 |
| 998 | 20262124 | 306 | 1226 | 21 | 2 | 3 |
| 999 | 20262125 | 292 | 1398 | 21 | 8 | 5 |

## Throughput

- Turns simulated: 248253 (first play of each game)
- Aggregate: 241.3 turns/s over 1028.7s wall
- Per game thread: 42.3 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

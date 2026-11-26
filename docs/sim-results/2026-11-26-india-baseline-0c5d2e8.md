# Simulation: india-baseline-0c5d2e8 (2026-11-26)

1000 games, first seed 20261126, brain `scripted`, turn limit off, safety cap 1200, 4 threads, started 2026-11-26T22:30:20Z. Wall time 2346.1s.

Source checkout at run start: `0c5d2e8f05b65f08db2319b210af40dfe36974b9`; dirty: no. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--games","1000","--seed","20261126","--threads","4","--difficulty","hard","--label","india-baseline-0c5d2e8"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 1000 of 1000 |
| Game length, turns (mean / median / min-max) | 252.4 / 242 / 153-500 |
| Win reason mix | VP hold 95.8%, Capitals 3.6%, Turn limit 0.5%, Draw 0.1% |
| Median end turn | 242 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 3.6% / 95.8% / 0.5% / 0.1% |
| Games reaching the World War era (any empire) | 993 of 1000 (99.3%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1565 / 513 / 964 / 88 |
| Hold break rate | 32.8% of started; 34.7% of resolved (broken / (broken + completed)) |
| Hold wins | 958 |
| First reach to win, turns (mean / median / min-max) | 13.7 / 9 / 9-281 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 156 |
| Wins by a team another team beat to its target | 101 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 958 | 248.9 / 240 / 153-476 |
| Capitals | 36 | 302.4 / 289 / 232-493 |
| Turn limit | 5 | 500.0 / 500 / 500-500 |
| Draw | 1 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 100 | 10.0% |
| India | 12 | 1.2% |
| Byzantium | 94 | 9.4% |
| Germany | 148 | 14.8% |
| Celts | 178 | 17.8% |
| Mongols | 77 | 7.7% |
| Sparta | 126 | 12.6% |
| Athens | 150 | 15.0% |
| Aztecs | 114 | 11.4% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 958 | 95.8% |
| Conquest | 0 | 0.0% |
| Capitals | 36 | 3.6% |
| Turn limit | 5 | 0.5% |
| Draw | 1 | 0.1% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 899 | 100 | 11.1% |
| India | 895 | 12 | 1.3% |
| Byzantium | 878 | 94 | 10.7% |
| Germany | 880 | 148 | 16.8% |
| Celts | 896 | 178 | 19.9% |
| Mongols | 877 | 77 | 8.8% |
| Sparta | 891 | 126 | 14.1% |
| Athens | 895 | 150 | 16.8% |
| Aztecs | 889 | 114 | 12.8% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 8000 | 999 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 899 | 100 | 11.1% |
| India | Hard | 895 | 12 | 1.3% |
| Byzantium | Hard | 878 | 94 | 10.7% |
| Germany | Hard | 880 | 148 | 16.8% |
| Celts | Hard | 896 | 178 | 19.9% |
| Mongols | Hard | 877 | 77 | 8.8% |
| Sparta | Hard | 891 | 126 | 14.1% |
| Athens | Hard | 895 | 150 | 16.8% |
| Aztecs | Hard | 889 | 114 | 12.8% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 899 | 130 | 14.5% | 114 | 150 | 191 | 268 | 487 |
| India | 895 | 454 | 50.7% | 42 | 90 | 129 | 213 | 404 |
| Byzantium | 878 | 623 | 71.0% | 69 | 85 | 110 | 166 | 305 |
| Germany | 880 | 293 | 33.3% | 97 | 127 | 172 | 239 | 465 |
| Celts | 896 | 655 | 73.1% | 31 | 180 | 210 | 210 | 311 |
| Mongols | 877 | 142 | 16.2% | 119 | 160 | 204 | 276 | 490 |
| Sparta | 891 | 212 | 23.8% | 131 | 164 | 211 | 289 | 492 |
| Athens | 895 | 359 | 40.1% | 114 | 170 | 255 | 255 | 255 |
| Aztecs | 889 | 356 | 40.0% | 124 | 190 | 225 | 246 | 489 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 153 | 194 | 242 | 319 | 500 |
| Winning team VP | 3 | 10 | 10 | 13 | 17 |
| First Soldier completed (turn, per empire) | 6 | 8 | 21 | 64 | 315 |
| First Settler completed (turn, per empire) | 3 | 14 | 16 | 32 | 205 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 8 | 18 | 23 | 46 | 215 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 7675 | 325 |
| First Settler completed | 7652 | 348 |
| First city founded | 8000 | 0 |
| Second city founded | 7564 | 436 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 1971 |
| 1 | 1190 |
| 2 | 1007 |
| 3 | 722 |
| 4 | 621 |
| 5 | 440 |
| 6 | 351 |
| 7 | 286 |
| 8 | 196 |
| 9 | 157 |
| 10 | 542 |
| 11 | 283 |
| 12 | 124 |
| 13 | 66 |
| 14 | 25 |
| 15 | 12 |
| 16 | 5 |
| 17 | 2 |

## End state

- Average game length: 252.4 turns
- Average cities per empire: 4.25
- Average techs per empire: 31.38
- Average eliminations per game: 2.62

## Strategy diversity: observed behavior (classifier v2)

1000 finished games; 8000 player seats; 999 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4288** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 3296 | 190 | 5.8% | 41.2% | 19.0% |
| Wide | 2602 | 722 | 27.7% | 32.5% | 72.3% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 9 | 0 | 0.0% | 0.1% | 0.0% |
| Economic | 660 | 86 | 13.0% | 8.2% | 8.6% |
| Generalist | 1433 | 1 | 0.1% | 17.9% | 0.1% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 20.1% / 3.4% (965) | n/a / n/a (0) | 35.2% / 3.7% (9) | 25.2% / 5.3% (486) | 82.6% / 2.4% (876) |
| Wide | 76.5% / 3.4% (965) | — | n/a / n/a (0) | 77.8% / 0.0% (9) | 56.9% / 4.5% (483) | 95.1% / 1.4% (866) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 61.1% / 3.7% (9) | 22.2% / 0.0% (9) | n/a / n/a (0) | — | 0.0% / 0.0% (4) | 78.6% / 7.1% (7) |
| Economic | 69.5% / 5.3% (486) | 38.6% / 4.5% (483) | n/a / n/a (0) | 100.0% / 0.0% (4) | — | 93.5% / 2.6% (418) |
| Generalist | 15.0% / 2.4% (876) | 3.5% / 1.4% (866) | n/a / n/a (0) | 14.3% / 7.1% (7) | 3.9% / 2.6% (418) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 8 / 245 | 82 / 382 | 0 / 0 | 0 / 0 | 10 / 115 | 0 / 157 |
| India | 4 / 451 | 8 / 271 | 0 / 0 | 0 / 0 | 0 / 10 | 0 / 163 |
| Byzantium | 1 / 14 | 87 / 407 | 0 / 0 | 0 / 5 | 6 / 124 | 0 / 328 |
| Germany | 19 / 343 | 121 / 306 | 0 / 0 | 0 / 2 | 8 / 42 | 0 / 187 |
| Celts | 16 / 270 | 152 / 402 | 0 / 0 | 0 / 0 | 10 / 92 | 0 / 132 |
| Mongols | 44 / 676 | 27 / 79 | 0 / 0 | 0 / 0 | 6 / 14 | 0 / 108 |
| Sparta | 72 / 695 | 36 / 147 | 0 / 0 | 0 / 1 | 18 / 40 | 0 / 8 |
| Athens | 9 / 261 | 123 / 311 | 0 / 0 | 0 / 0 | 17 / 126 | 1 / 197 |
| Aztecs | 17 / 341 | 86 / 297 | 0 / 0 | 0 / 1 | 11 / 97 | 0 / 153 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Turtle/Wonder. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 999 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 15502/1000 | 15.502 | 1000/1000 (100.0%) |
| Peace offers | 101377/1000 | 101.377 | 1000/1000 (100.0%) |
| Peace agreements | 12437/1000 | 12.437 | 998/1000 (99.8%) |
| Battles | 1250656/1000 | 1250.656 | 1000/1000 (100.0%) |
| City captures | 23606/1000 | 23.606 | 1000/1000 (100.0%) |
| Settler captures | 4222/1000 | 4.222 | 958/1000 (95.8%) |
| Eliminations | 2625/1000 | 2.625 | 972/1000 (97.2%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 6546/8000 | 6546/8000 | 3106/999 |
| Faction goals | 5248/8000 | 5248/8000 | 2154/999 |
| Era objectives | 14276/8000 | 14276/8000 | 4044/999 |
| Capitals taken | 2596/8000 | 2596/8000 | 1512/999 |
| Total VP | 28666/8000 | 28666/8000 | 10816/999 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 106/725 (14.6%) | 1 |
| Tied leaders (winner in set) | 262/275 (95.3%) | 0 |
| Any leader set | 368/1000 (36.8%) | 1 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 7536/8000 | 5 | 20 | 29 | 33 | 70 | -2.0 (999) |
| Medieval | 7304/8000 | 31 | 58 | 62 | 66 | 130 | -1.8 (999) |
| Renaissance | 7235/8000 | 65 | 84 | 90 | 98 | 168 | -3.3 (999) |
| Industrial | 7041/8000 | 89 | 112 | 121 | 136 | 349 | -5.8 (999) |
| Machine Age | 6474/8000 | 109 | 141 | 153 | 174 | 444 | -8.3 (999) |
| World War | 5295/8000 | 128 | 167 | 182 | 208 | 499 | -10.3 (990) |
| Modern | 3603/8000 | 152 | 192 | 213 | 246 | 457 | -11.2 (881) |
| Future | 1981/8000 | 176 | 217 | 247 | 268 | 494 | -8.7 (615) |

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
| Classical | Engineering | 7318/8000 | 51 | 50 AD | n/a |
| Classical | Iron Working | 7380/8000 | 38 | 601 BC | 1200 BC |
| Classical | Code of Laws | 7312/8000 | 59 | 450 AD | 1754 BC |
| Classical | Philosophy | 7304/8000 | 61 | 530 AD | 500 BC |
| Medieval | Feudalism | 7277/8000 | 75 | 950 AD | n/a |
| Medieval | Machinery | 7289/8000 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 6446/8000 | 81 | 1130 AD | n/a |
| Medieval | Education | 7253/8000 | 86 | 1280 AD | 1088 AD |
| Medieval | Crop Rotation | 7236/8000 | 89 | 1370 AD | n/a |
| Renaissance | Gunpowder | 7216/8000 | 97 | 1470 AD | n/a |
| Renaissance | Metallurgy | 7153/8000 | 108 | 1580 AD | n/a |
| Renaissance | Military Engineering | 7186/8000 | 105 | 1550 AD | n/a |
| Renaissance | Banking | 7087/8000 | 117 | 1670 AD | 1397 AD |
| Renaissance | Printing Press | 7050/8000 | 119 | 1690 AD | 1440 AD |
| Industrial | Rifling | 6697/8000 | 143 | 1816 AD | n/a |
| Industrial | Military Science | 6849/8000 | 134 | 1783 AD | n/a |
| Industrial | Ballistics | 6732/8000 | 145 | 1824 AD | n/a |
| Industrial | Industrialization | 6934/8000 | 130 | 1768 AD | 1769 AD |
| Industrial | Railways | 6479/8000 | 153 | 1854 AD | 1825 AD |
| Machine Age | Replaceable Parts | 5969/8000 | 168 | 1889 AD | 1800 AD |
| Machine Age | Combustion | 6240/8000 | 162 | 1882 AD | 1876 AD |
| Machine Age | Steel | 5910/8000 | 166 | 1886 AD | 1856 AD |
| Machine Age | Flight | 5567/8000 | 177 | 1899 AD | 1903 AD |
| Machine Age | Electricity | 5351/8000 | 182 | 1904 AD | 1879 AD |
| World War | Mass Mobilisation | 4456/8000 | 197 | 1924 AD | n/a |
| World War | Rocketry | 4200/8000 | 202 | 1931 AD | 1926 AD |
| World War | Armoured Warfare | 3982/8000 | 207 | 1938 AD | 1916 AD |
| World War | Air Power | 3838/8000 | 211 | 1944 AD | n/a |
| World War | Electronics | 4947/8000 | 189 | 1912 AD | n/a |
| Modern | Mechanised Warfare | 2682/8000 | 228 | 1981 AD | n/a |
| Modern | Composite Armour | 2456/8000 | 233 | 1993 AD | n/a |
| Modern | Guided Missiles | 2310/8000 | 240 | 2010 AD | n/a |
| Modern | Jet Engines | 2230/8000 | 243 | 2014 AD | 1939 AD |
| Modern | Green Revolution | 3129/8000 | 221 | 1964 AD | n/a |
| Future | Autonomous Systems | 1378/8000 | 256 | 2032 AD | n/a |
| Future | Robotics | 1168/8000 | 263 | 2039 AD | n/a |
| Future | Fusion Power | 1136/8000 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 1612/8000 | 256 | 2032 AD | n/a |
| Future | Recursive AI | 1027/8000 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 64.16 |
| 1 | 71.39 | 14.21 |
| 2 | 27.96 | 5.66 |
| 3 | 8.07 | 1.31 |
| 4 | 1.69 | 0.30 |
| 5 | 0.38 | 0.08 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 20261126 | 229 | 1207 | 15 | 5 | 3 |
| 1 | 20261127 | 233 | 1018 | 15 | 7 | 3 |
| 2 | 20261128 | 277 | 1660 | 14 | 8 | 1 |
| 3 | 20261129 | 287 | 1198 | 37 | 1 | 2 |
| 4 | 20261130 | 225 | 1285 | 28 | 3 | 3 |
| 5 | 20261131 | 194 | 797 | 15 | 4 | 1 |
| 6 | 20261132 | 208 | 1019 | 22 | 2 | 0 |
| 7 | 20261133 | 331 | 1809 | 40 | 2 | 4 |
| 8 | 20261134 | 244 | 1096 | 19 | 6 | 3 |
| 9 | 20261135 | 250 | 1435 | 15 | 6 | 5 |
| 10 | 20261136 | 286 | 1360 | 29 | 6 | 2 |
| 11 | 20261137 | 281 | 1187 | 21 | 2 | 2 |
| 12 | 20261138 | 274 | 1504 | 30 | 5 | 4 |
| 13 | 20261139 | 226 | 988 | 14 | 8 | 1 |
| 14 | 20261140 | 258 | 1527 | 44 | 6 | 4 |
| 15 | 20261141 | 195 | 904 | 3 | 7 | 2 |
| 16 | 20261142 | 266 | 1231 | 21 | 4 | 2 |
| 17 | 20261143 | 191 | 1001 | 15 | 6 | 1 |
| 18 | 20261144 | 278 | 1146 | 29 | 3 | 4 |
| 19 | 20261145 | 318 | 1466 | 43 | 3 | 4 |
| 20 | 20261146 | 234 | 1070 | 14 | 2 | 1 |
| 21 | 20261147 | 457 | 2358 | 46 | 15 | 3 |
| 22 | 20261148 | 250 | 1223 | 26 | 3 | 3 |
| 23 | 20261149 | 254 | 1431 | 18 | 2 | 2 |
| 24 | 20261150 | 291 | 1632 | 23 | 4 | 0 |
| 25 | 20261151 | 242 | 1059 | 24 | 0 | 2 |
| 26 | 20261152 | 212 | 1309 | 19 | 7 | 4 |
| 27 | 20261153 | 219 | 952 | 10 | 6 | 3 |
| 28 | 20261154 | 237 | 1212 | 28 | 9 | 1 |
| 29 | 20261155 | 209 | 834 | 12 | 7 | 1 |
| 30 | 20261156 | 500 | 3111 | 56 | 0 | 4 |
| 31 | 20261157 | 215 | 1186 | 29 | 2 | 2 |
| 32 | 20261158 | 274 | 1423 | 27 | 5 | 3 |
| 33 | 20261159 | 204 | 835 | 23 | 4 | 1 |
| 34 | 20261160 | 221 | 1178 | 17 | 3 | 2 |
| 35 | 20261161 | 211 | 834 | 15 | 4 | 2 |
| 36 | 20261162 | 231 | 990 | 23 | 1 | 4 |
| 37 | 20261163 | 252 | 1337 | 32 | 4 | 2 |
| 38 | 20261164 | 220 | 901 | 26 | 3 | 3 |
| 39 | 20261165 | 297 | 1618 | 19 | 3 | 4 |
| 40 | 20261166 | 167 | 925 | 13 | 6 | 2 |
| 41 | 20261167 | 193 | 945 | 24 | 4 | 4 |
| 42 | 20261168 | 206 | 1199 | 22 | 3 | 3 |
| 43 | 20261169 | 310 | 2082 | 25 | 10 | 2 |
| 44 | 20261170 | 364 | 1791 | 20 | 5 | 4 |
| 45 | 20261171 | 197 | 710 | 14 | 8 | 3 |
| 46 | 20261172 | 268 | 1351 | 26 | 4 | 1 |
| 47 | 20261173 | 249 | 1022 | 21 | 3 | 4 |
| 48 | 20261174 | 227 | 952 | 26 | 7 | 6 |
| 49 | 20261175 | 260 | 1366 | 22 | 1 | 5 |
| 50 | 20261176 | 177 | 962 | 26 | 2 | 3 |
| 51 | 20261177 | 281 | 1381 | 25 | 7 | 3 |
| 52 | 20261178 | 242 | 1088 | 18 | 0 | 2 |
| 53 | 20261179 | 269 | 1453 | 34 | 6 | 3 |
| 54 | 20261180 | 203 | 789 | 10 | 2 | 3 |
| 55 | 20261181 | 213 | 1065 | 25 | 5 | 3 |
| 56 | 20261182 | 274 | 1236 | 24 | 9 | 2 |
| 57 | 20261183 | 273 | 1425 | 22 | 1 | 2 |
| 58 | 20261184 | 234 | 1150 | 26 | 1 | 3 |
| 59 | 20261185 | 250 | 1044 | 21 | 3 | 2 |
| 60 | 20261186 | 218 | 943 | 22 | 4 | 2 |
| 61 | 20261187 | 271 | 1099 | 30 | 5 | 3 |
| 62 | 20261188 | 339 | 2048 | 39 | 1 | 3 |
| 63 | 20261189 | 299 | 1907 | 29 | 4 | 3 |
| 64 | 20261190 | 272 | 1347 | 34 | 9 | 3 |
| 65 | 20261191 | 274 | 1337 | 24 | 8 | 4 |
| 66 | 20261192 | 271 | 1513 | 37 | 2 | 2 |
| 67 | 20261193 | 220 | 1341 | 13 | 0 | 4 |
| 68 | 20261194 | 303 | 1744 | 23 | 2 | 1 |
| 69 | 20261195 | 235 | 1230 | 20 | 8 | 1 |
| 70 | 20261196 | 193 | 836 | 13 | 7 | 2 |
| 71 | 20261197 | 281 | 1520 | 18 | 5 | 4 |
| 72 | 20261198 | 252 | 1174 | 17 | 4 | 2 |
| 73 | 20261199 | 323 | 1581 | 33 | 6 | 4 |
| 74 | 20261200 | 242 | 1391 | 17 | 3 | 2 |
| 75 | 20261201 | 198 | 827 | 17 | 8 | 4 |
| 76 | 20261202 | 234 | 1295 | 28 | 7 | 0 |
| 77 | 20261203 | 277 | 1408 | 13 | 12 | 2 |
| 78 | 20261204 | 365 | 1637 | 22 | 8 | 6 |
| 79 | 20261205 | 309 | 1770 | 37 | 5 | 5 |
| 80 | 20261206 | 271 | 1127 | 31 | 7 | 2 |
| 81 | 20261207 | 195 | 597 | 3 | 3 | 3 |
| 82 | 20261208 | 259 | 1396 | 23 | 6 | 3 |
| 83 | 20261209 | 250 | 1228 | 24 | 1 | 1 |
| 84 | 20261210 | 194 | 936 | 26 | 6 | 2 |
| 85 | 20261211 | 210 | 1111 | 30 | 0 | 1 |
| 86 | 20261212 | 241 | 1080 | 17 | 2 | 3 |
| 87 | 20261213 | 246 | 868 | 20 | 7 | 2 |
| 88 | 20261214 | 270 | 993 | 25 | 6 | 3 |
| 89 | 20261215 | 270 | 1431 | 44 | 2 | 4 |
| 90 | 20261216 | 295 | 1561 | 36 | 5 | 1 |
| 91 | 20261217 | 286 | 1376 | 24 | 2 | 3 |
| 92 | 20261218 | 208 | 911 | 20 | 3 | 2 |
| 93 | 20261219 | 241 | 1088 | 9 | 3 | 4 |
| 94 | 20261220 | 195 | 896 | 17 | 2 | 1 |
| 95 | 20261221 | 273 | 1236 | 31 | 8 | 3 |
| 96 | 20261222 | 332 | 2113 | 49 | 8 | 4 |
| 97 | 20261223 | 203 | 788 | 18 | 3 | 3 |
| 98 | 20261224 | 213 | 948 | 16 | 6 | 2 |
| 99 | 20261225 | 220 | 1183 | 27 | 5 | 1 |
| 100 | 20261226 | 209 | 1088 | 17 | 3 | 2 |
| 101 | 20261227 | 313 | 1434 | 34 | 6 | 2 |
| 102 | 20261228 | 219 | 1051 | 10 | 4 | 1 |
| 103 | 20261229 | 254 | 1157 | 22 | 3 | 3 |
| 104 | 20261230 | 340 | 1702 | 33 | 10 | 1 |
| 105 | 20261231 | 199 | 692 | 12 | 1 | 2 |
| 106 | 20261232 | 182 | 791 | 17 | 4 | 3 |
| 107 | 20261233 | 272 | 1335 | 27 | 2 | 2 |
| 108 | 20261234 | 207 | 655 | 22 | 9 | 4 |
| 109 | 20261235 | 269 | 1281 | 18 | 2 | 2 |
| 110 | 20261236 | 181 | 746 | 9 | 3 | 2 |
| 111 | 20261237 | 265 | 1382 | 31 | 2 | 4 |
| 112 | 20261238 | 241 | 1110 | 33 | 4 | 2 |
| 113 | 20261239 | 203 | 964 | 19 | 3 | 2 |
| 114 | 20261240 | 256 | 1569 | 35 | 6 | 2 |
| 115 | 20261241 | 236 | 1020 | 15 | 0 | 4 |
| 116 | 20261242 | 290 | 1144 | 25 | 0 | 3 |
| 117 | 20261243 | 226 | 1131 | 17 | 5 | 2 |
| 118 | 20261244 | 212 | 1119 | 25 | 4 | 1 |
| 119 | 20261245 | 275 | 1535 | 25 | 1 | 3 |
| 120 | 20261246 | 234 | 1166 | 15 | 4 | 3 |
| 121 | 20261247 | 219 | 819 | 16 | 3 | 2 |
| 122 | 20261248 | 313 | 2126 | 32 | 6 | 2 |
| 123 | 20261249 | 273 | 1250 | 31 | 1 | 1 |
| 124 | 20261250 | 225 | 1036 | 25 | 5 | 3 |
| 125 | 20261251 | 282 | 1516 | 18 | 1 | 1 |
| 126 | 20261252 | 222 | 1002 | 27 | 6 | 3 |
| 127 | 20261253 | 224 | 926 | 16 | 2 | 2 |
| 128 | 20261254 | 249 | 1220 | 16 | 5 | 4 |
| 129 | 20261255 | 216 | 1087 | 23 | 2 | 3 |
| 130 | 20261256 | 255 | 1273 | 29 | 0 | 2 |
| 131 | 20261257 | 277 | 1360 | 17 | 1 | 2 |
| 132 | 20261258 | 272 | 1071 | 13 | 1 | 3 |
| 133 | 20261259 | 290 | 1405 | 28 | 8 | 2 |
| 134 | 20261260 | 233 | 1226 | 28 | 4 | 2 |
| 135 | 20261261 | 500 | 2016 | 64 | 5 | 5 |
| 136 | 20261262 | 208 | 881 | 22 | 2 | 2 |
| 137 | 20261263 | 235 | 1254 | 25 | 0 | 2 |
| 138 | 20261264 | 194 | 964 | 23 | 4 | 3 |
| 139 | 20261265 | 224 | 1086 | 20 | 5 | 3 |
| 140 | 20261266 | 250 | 1307 | 32 | 8 | 4 |
| 141 | 20261267 | 280 | 1206 | 32 | 5 | 5 |
| 142 | 20261268 | 226 | 1125 | 24 | 3 | 0 |
| 143 | 20261269 | 321 | 1605 | 28 | 8 | 5 |
| 144 | 20261270 | 234 | 1004 | 22 | 1 | 1 |
| 145 | 20261271 | 277 | 1335 | 21 | 2 | 2 |
| 146 | 20261272 | 268 | 1334 | 26 | 7 | 2 |
| 147 | 20261273 | 262 | 1109 | 33 | 2 | 4 |
| 148 | 20261274 | 317 | 1719 | 26 | 8 | 5 |
| 149 | 20261275 | 219 | 1023 | 23 | 5 | 2 |
| 150 | 20261276 | 256 | 1093 | 17 | 7 | 2 |
| 151 | 20261277 | 240 | 1052 | 22 | 6 | 4 |
| 152 | 20261278 | 185 | 932 | 20 | 12 | 2 |
| 153 | 20261279 | 249 | 1449 | 13 | 7 | 1 |
| 154 | 20261280 | 262 | 1167 | 34 | 5 | 4 |
| 155 | 20261281 | 223 | 1378 | 25 | 1 | 2 |
| 156 | 20261282 | 218 | 969 | 18 | 2 | 1 |
| 157 | 20261283 | 215 | 934 | 18 | 3 | 2 |
| 158 | 20261284 | 286 | 1187 | 18 | 6 | 3 |
| 159 | 20261285 | 236 | 1250 | 29 | 10 | 3 |
| 160 | 20261286 | 170 | 609 | 15 | 3 | 0 |
| 161 | 20261287 | 256 | 1299 | 25 | 5 | 2 |
| 162 | 20261288 | 273 | 1803 | 35 | 6 | 5 |
| 163 | 20261289 | 228 | 1161 | 17 | 1 | 3 |
| 164 | 20261290 | 399 | 2034 | 50 | 4 | 3 |
| 165 | 20261291 | 251 | 1270 | 22 | 8 | 2 |
| 166 | 20261292 | 251 | 1303 | 32 | 5 | 4 |
| 167 | 20261293 | 182 | 761 | 14 | 1 | 1 |
| 168 | 20261294 | 243 | 1113 | 20 | 2 | 3 |
| 169 | 20261295 | 347 | 1613 | 27 | 3 | 4 |
| 170 | 20261296 | 403 | 2563 | 39 | 6 | 3 |
| 171 | 20261297 | 220 | 781 | 21 | 1 | 3 |
| 172 | 20261298 | 292 | 1692 | 23 | 2 | 5 |
| 173 | 20261299 | 330 | 1534 | 34 | 5 | 4 |
| 174 | 20261300 | 227 | 1495 | 22 | 2 | 2 |
| 175 | 20261301 | 292 | 1568 | 27 | 2 | 5 |
| 176 | 20261302 | 250 | 1154 | 26 | 7 | 4 |
| 177 | 20261303 | 281 | 1310 | 25 | 1 | 5 |
| 178 | 20261304 | 246 | 880 | 7 | 1 | 5 |
| 179 | 20261305 | 299 | 2003 | 26 | 8 | 1 |
| 180 | 20261306 | 357 | 1609 | 36 | 3 | 5 |
| 181 | 20261307 | 214 | 953 | 25 | 2 | 2 |
| 182 | 20261308 | 233 | 1127 | 33 | 4 | 3 |
| 183 | 20261309 | 256 | 1167 | 41 | 6 | 5 |
| 184 | 20261310 | 238 | 1450 | 21 | 3 | 2 |
| 185 | 20261311 | 214 | 1085 | 25 | 5 | 3 |
| 186 | 20261312 | 260 | 1488 | 17 | 6 | 5 |
| 187 | 20261313 | 263 | 1384 | 32 | 7 | 3 |
| 188 | 20261314 | 194 | 1014 | 18 | 4 | 1 |
| 189 | 20261315 | 402 | 2095 | 32 | 10 | 5 |
| 190 | 20261316 | 224 | 1012 | 20 | 6 | 4 |
| 191 | 20261317 | 219 | 1040 | 32 | 5 | 4 |
| 192 | 20261318 | 306 | 1486 | 12 | 10 | 5 |
| 193 | 20261319 | 201 | 648 | 13 | 2 | 2 |
| 194 | 20261320 | 226 | 1076 | 23 | 4 | 1 |
| 195 | 20261321 | 235 | 1170 | 31 | 6 | 3 |
| 196 | 20261322 | 233 | 1039 | 19 | 1 | 1 |
| 197 | 20261323 | 288 | 1345 | 15 | 3 | 5 |
| 198 | 20261324 | 260 | 1128 | 23 | 2 | 4 |
| 199 | 20261325 | 238 | 1164 | 24 | 5 | 2 |
| 200 | 20261326 | 283 | 1888 | 35 | 5 | 2 |
| 201 | 20261327 | 213 | 1107 | 15 | 2 | 1 |
| 202 | 20261328 | 288 | 1414 | 29 | 6 | 4 |
| 203 | 20261329 | 425 | 2568 | 28 | 3 | 2 |
| 204 | 20261330 | 201 | 862 | 19 | 3 | 1 |
| 205 | 20261331 | 224 | 1072 | 28 | 2 | 2 |
| 206 | 20261332 | 296 | 1403 | 18 | 4 | 3 |
| 207 | 20261333 | 235 | 1150 | 24 | 3 | 2 |
| 208 | 20261334 | 289 | 1666 | 29 | 6 | 3 |
| 209 | 20261335 | 262 | 1534 | 44 | 6 | 2 |
| 210 | 20261336 | 210 | 1137 | 19 | 10 | 3 |
| 211 | 20261337 | 264 | 1385 | 24 | 10 | 5 |
| 212 | 20261338 | 458 | 2236 | 24 | 8 | 0 |
| 213 | 20261339 | 208 | 909 | 14 | 6 | 1 |
| 214 | 20261340 | 233 | 764 | 8 | 3 | 2 |
| 215 | 20261341 | 204 | 849 | 19 | 8 | 5 |
| 216 | 20261342 | 229 | 1113 | 22 | 2 | 1 |
| 217 | 20261343 | 238 | 1139 | 25 | 7 | 4 |
| 218 | 20261344 | 242 | 1111 | 25 | 4 | 4 |
| 219 | 20261345 | 207 | 816 | 14 | 3 | 2 |
| 220 | 20261346 | 253 | 1423 | 22 | 1 | 3 |
| 221 | 20261347 | 252 | 1481 | 14 | 3 | 2 |
| 222 | 20261348 | 228 | 1082 | 22 | 5 | 3 |
| 223 | 20261349 | 232 | 948 | 18 | 5 | 3 |
| 224 | 20261350 | 255 | 994 | 29 | 5 | 6 |
| 225 | 20261351 | 261 | 1193 | 23 | 9 | 4 |
| 226 | 20261352 | 224 | 1061 | 20 | 4 | 2 |
| 227 | 20261353 | 392 | 2225 | 38 | 7 | 3 |
| 228 | 20261354 | 224 | 723 | 25 | 4 | 3 |
| 229 | 20261355 | 201 | 970 | 19 | 0 | 2 |
| 230 | 20261356 | 277 | 1302 | 24 | 3 | 5 |
| 231 | 20261357 | 153 | 705 | 12 | 3 | 1 |
| 232 | 20261358 | 224 | 1094 | 15 | 7 | 4 |
| 233 | 20261359 | 233 | 1180 | 28 | 4 | 3 |
| 234 | 20261360 | 202 | 811 | 15 | 4 | 4 |
| 235 | 20261361 | 404 | 1803 | 26 | 5 | 5 |
| 236 | 20261362 | 271 | 1658 | 27 | 3 | 1 |
| 237 | 20261363 | 256 | 1392 | 17 | 7 | 4 |
| 238 | 20261364 | 212 | 862 | 21 | 6 | 4 |
| 239 | 20261365 | 317 | 1539 | 23 | 6 | 5 |
| 240 | 20261366 | 268 | 1618 | 28 | 4 | 2 |
| 241 | 20261367 | 279 | 1468 | 23 | 1 | 4 |
| 242 | 20261368 | 214 | 995 | 17 | 1 | 1 |
| 243 | 20261369 | 206 | 886 | 18 | 4 | 2 |
| 244 | 20261370 | 282 | 1431 | 42 | 6 | 2 |
| 245 | 20261371 | 199 | 710 | 15 | 0 | 2 |
| 246 | 20261372 | 339 | 1514 | 24 | 3 | 4 |
| 247 | 20261373 | 216 | 1068 | 17 | 2 | 1 |
| 248 | 20261374 | 394 | 2741 | 16 | 3 | 5 |
| 249 | 20261375 | 222 | 959 | 16 | 3 | 2 |
| 250 | 20261376 | 249 | 1204 | 21 | 1 | 3 |
| 251 | 20261377 | 185 | 878 | 15 | 4 | 3 |
| 252 | 20261378 | 432 | 2578 | 35 | 1 | 3 |
| 253 | 20261379 | 234 | 1008 | 17 | 2 | 2 |
| 254 | 20261380 | 322 | 1783 | 21 | 5 | 3 |
| 255 | 20261381 | 306 | 1386 | 19 | 5 | 4 |
| 256 | 20261382 | 243 | 1140 | 21 | 1 | 4 |
| 257 | 20261383 | 276 | 1218 | 27 | 2 | 5 |
| 258 | 20261384 | 268 | 1708 | 29 | 5 | 3 |
| 259 | 20261385 | 263 | 1117 | 17 | 4 | 2 |
| 260 | 20261386 | 264 | 1005 | 24 | 7 | 3 |
| 261 | 20261387 | 253 | 1150 | 33 | 9 | 1 |
| 262 | 20261388 | 206 | 825 | 15 | 4 | 4 |
| 263 | 20261389 | 259 | 1445 | 23 | 4 | 3 |
| 264 | 20261390 | 235 | 961 | 30 | 2 | 3 |
| 265 | 20261391 | 277 | 1261 | 39 | 4 | 5 |
| 266 | 20261392 | 193 | 789 | 14 | 0 | 3 |
| 267 | 20261393 | 308 | 1545 | 41 | 5 | 2 |
| 268 | 20261394 | 262 | 1465 | 26 | 9 | 2 |
| 269 | 20261395 | 200 | 966 | 17 | 6 | 1 |
| 270 | 20261396 | 238 | 1181 | 29 | 0 | 3 |
| 271 | 20261397 | 289 | 1346 | 23 | 4 | 2 |
| 272 | 20261398 | 277 | 1566 | 31 | 1 | 2 |
| 273 | 20261399 | 212 | 1071 | 25 | 7 | 2 |
| 274 | 20261400 | 213 | 683 | 19 | 1 | 3 |
| 275 | 20261401 | 187 | 875 | 21 | 6 | 2 |
| 276 | 20261402 | 259 | 1352 | 23 | 6 | 4 |
| 277 | 20261403 | 227 | 1122 | 24 | 6 | 1 |
| 278 | 20261404 | 254 | 927 | 26 | 5 | 3 |
| 279 | 20261405 | 288 | 983 | 12 | 1 | 4 |
| 280 | 20261406 | 229 | 1208 | 30 | 6 | 1 |
| 281 | 20261407 | 155 | 668 | 8 | 3 | 1 |
| 282 | 20261408 | 252 | 1458 | 25 | 12 | 2 |
| 283 | 20261409 | 231 | 1069 | 19 | 8 | 3 |
| 284 | 20261410 | 273 | 1537 | 21 | 4 | 5 |
| 285 | 20261411 | 233 | 1154 | 25 | 3 | 2 |
| 286 | 20261412 | 268 | 1126 | 20 | 4 | 1 |
| 287 | 20261413 | 207 | 1098 | 16 | 2 | 2 |
| 288 | 20261414 | 264 | 1380 | 23 | 4 | 2 |
| 289 | 20261415 | 258 | 1214 | 33 | 4 | 1 |
| 290 | 20261416 | 275 | 1676 | 18 | 2 | 5 |
| 291 | 20261417 | 180 | 776 | 13 | 3 | 2 |
| 292 | 20261418 | 260 | 1215 | 32 | 5 | 4 |
| 293 | 20261419 | 309 | 1843 | 32 | 6 | 3 |
| 294 | 20261420 | 257 | 1206 | 29 | 7 | 2 |
| 295 | 20261421 | 168 | 868 | 14 | 11 | 0 |
| 296 | 20261422 | 241 | 1055 | 15 | 1 | 4 |
| 297 | 20261423 | 319 | 1819 | 24 | 6 | 6 |
| 298 | 20261424 | 309 | 1446 | 35 | 1 | 4 |
| 299 | 20261425 | 259 | 1188 | 18 | 2 | 2 |
| 300 | 20261426 | 248 | 1133 | 18 | 4 | 3 |
| 301 | 20261427 | 368 | 2154 | 46 | 4 | 2 |
| 302 | 20261428 | 217 | 1018 | 25 | 5 | 1 |
| 303 | 20261429 | 290 | 1509 | 28 | 2 | 1 |
| 304 | 20261430 | 228 | 1076 | 19 | 5 | 2 |
| 305 | 20261431 | 211 | 819 | 21 | 5 | 2 |
| 306 | 20261432 | 248 | 816 | 24 | 3 | 3 |
| 307 | 20261433 | 243 | 1171 | 35 | 3 | 4 |
| 308 | 20261434 | 258 | 1174 | 26 | 2 | 3 |
| 309 | 20261435 | 233 | 1243 | 28 | 2 | 4 |
| 310 | 20261436 | 220 | 1245 | 30 | 4 | 2 |
| 311 | 20261437 | 293 | 1571 | 30 | 4 | 3 |
| 312 | 20261438 | 227 | 1145 | 14 | 3 | 2 |
| 313 | 20261439 | 284 | 1616 | 32 | 5 | 3 |
| 314 | 20261440 | 356 | 1846 | 38 | 7 | 3 |
| 315 | 20261441 | 247 | 1066 | 21 | 1 | 4 |
| 316 | 20261442 | 270 | 1257 | 27 | 4 | 3 |
| 317 | 20261443 | 224 | 1085 | 14 | 2 | 1 |
| 318 | 20261444 | 319 | 1360 | 25 | 6 | 2 |
| 319 | 20261445 | 179 | 847 | 19 | 2 | 3 |
| 320 | 20261446 | 298 | 1936 | 45 | 2 | 2 |
| 321 | 20261447 | 298 | 1639 | 50 | 0 | 5 |
| 322 | 20261448 | 189 | 925 | 14 | 2 | 1 |
| 323 | 20261449 | 200 | 1124 | 16 | 4 | 1 |
| 324 | 20261450 | 222 | 1068 | 24 | 7 | 2 |
| 325 | 20261451 | 222 | 1019 | 16 | 3 | 1 |
| 326 | 20261452 | 240 | 1212 | 22 | 7 | 4 |
| 327 | 20261453 | 261 | 1423 | 32 | 2 | 5 |
| 328 | 20261454 | 233 | 1463 | 23 | 7 | 1 |
| 329 | 20261455 | 213 | 870 | 28 | 8 | 1 |
| 330 | 20261456 | 285 | 1433 | 24 | 1 | 1 |
| 331 | 20261457 | 239 | 1136 | 23 | 3 | 3 |
| 332 | 20261458 | 209 | 1215 | 16 | 5 | 4 |
| 333 | 20261459 | 287 | 1406 | 22 | 4 | 3 |
| 334 | 20261460 | 293 | 1331 | 27 | 5 | 3 |
| 335 | 20261461 | 198 | 1024 | 14 | 4 | 3 |
| 336 | 20261462 | 331 | 1682 | 39 | 8 | 2 |
| 337 | 20261463 | 254 | 1419 | 17 | 3 | 2 |
| 338 | 20261464 | 199 | 981 | 17 | 10 | 2 |
| 339 | 20261465 | 218 | 1171 | 21 | 8 | 3 |
| 340 | 20261466 | 222 | 974 | 18 | 4 | 2 |
| 341 | 20261467 | 351 | 1789 | 47 | 5 | 4 |
| 342 | 20261468 | 194 | 921 | 20 | 5 | 2 |
| 343 | 20261469 | 245 | 1351 | 31 | 7 | 4 |
| 344 | 20261470 | 296 | 1133 | 26 | 2 | 3 |
| 345 | 20261471 | 181 | 844 | 11 | 9 | 1 |
| 346 | 20261472 | 271 | 1363 | 11 | 0 | 3 |
| 347 | 20261473 | 217 | 1176 | 19 | 2 | 1 |
| 348 | 20261474 | 381 | 2323 | 40 | 3 | 1 |
| 349 | 20261475 | 351 | 1762 | 45 | 9 | 3 |
| 350 | 20261476 | 214 | 924 | 11 | 13 | 0 |
| 351 | 20261477 | 313 | 1404 | 32 | 3 | 3 |
| 352 | 20261478 | 223 | 961 | 22 | 6 | 4 |
| 353 | 20261479 | 323 | 1727 | 42 | 2 | 3 |
| 354 | 20261480 | 227 | 872 | 14 | 5 | 1 |
| 355 | 20261481 | 206 | 889 | 13 | 5 | 4 |
| 356 | 20261482 | 214 | 961 | 25 | 6 | 2 |
| 357 | 20261483 | 270 | 1289 | 25 | 5 | 2 |
| 358 | 20261484 | 267 | 1249 | 16 | 5 | 0 |
| 359 | 20261485 | 274 | 1651 | 35 | 2 | 5 |
| 360 | 20261486 | 241 | 937 | 21 | 4 | 3 |
| 361 | 20261487 | 226 | 1075 | 24 | 5 | 4 |
| 362 | 20261488 | 217 | 1248 | 22 | 0 | 2 |
| 363 | 20261489 | 233 | 1091 | 16 | 7 | 2 |
| 364 | 20261490 | 225 | 926 | 16 | 1 | 0 |
| 365 | 20261491 | 263 | 1304 | 28 | 4 | 4 |
| 366 | 20261492 | 225 | 933 | 22 | 7 | 2 |
| 367 | 20261493 | 240 | 1091 | 23 | 3 | 2 |
| 368 | 20261494 | 223 | 1151 | 21 | 3 | 3 |
| 369 | 20261495 | 266 | 1252 | 21 | 11 | 2 |
| 370 | 20261496 | 193 | 889 | 20 | 3 | 3 |
| 371 | 20261497 | 253 | 1325 | 13 | 5 | 3 |
| 372 | 20261498 | 181 | 966 | 27 | 3 | 1 |
| 373 | 20261499 | 228 | 973 | 20 | 2 | 1 |
| 374 | 20261500 | 214 | 823 | 20 | 2 | 3 |
| 375 | 20261501 | 361 | 2050 | 54 | 5 | 2 |
| 376 | 20261502 | 275 | 1410 | 20 | 2 | 2 |
| 377 | 20261503 | 169 | 649 | 12 | 3 | 3 |
| 378 | 20261504 | 179 | 999 | 11 | 5 | 1 |
| 379 | 20261505 | 217 | 1120 | 15 | 7 | 2 |
| 380 | 20261506 | 379 | 2123 | 50 | 9 | 4 |
| 381 | 20261507 | 201 | 960 | 17 | 3 | 1 |
| 382 | 20261508 | 191 | 905 | 18 | 2 | 2 |
| 383 | 20261509 | 221 | 1172 | 18 | 3 | 1 |
| 384 | 20261510 | 165 | 643 | 11 | 4 | 1 |
| 385 | 20261511 | 283 | 1315 | 42 | 10 | 6 |
| 386 | 20261512 | 222 | 1116 | 20 | 5 | 4 |
| 387 | 20261513 | 247 | 1418 | 25 | 3 | 4 |
| 388 | 20261514 | 210 | 1238 | 28 | 3 | 5 |
| 389 | 20261515 | 315 | 2119 | 20 | 10 | 2 |
| 390 | 20261516 | 264 | 1720 | 18 | 7 | 2 |
| 391 | 20261517 | 259 | 1123 | 24 | 3 | 3 |
| 392 | 20261518 | 265 | 1364 | 22 | 5 | 1 |
| 393 | 20261519 | 288 | 1076 | 32 | 6 | 2 |
| 394 | 20261520 | 232 | 878 | 16 | 2 | 4 |
| 395 | 20261521 | 194 | 877 | 20 | 5 | 2 |
| 396 | 20261522 | 288 | 1847 | 29 | 9 | 2 |
| 397 | 20261523 | 221 | 998 | 23 | 6 | 1 |
| 398 | 20261524 | 190 | 728 | 17 | 7 | 2 |
| 399 | 20261525 | 192 | 947 | 18 | 6 | 2 |
| 400 | 20261526 | 256 | 1031 | 16 | 2 | 5 |
| 401 | 20261527 | 222 | 1113 | 21 | 8 | 2 |
| 402 | 20261528 | 238 | 985 | 13 | 1 | 2 |
| 403 | 20261529 | 217 | 1115 | 23 | 5 | 2 |
| 404 | 20261530 | 266 | 1121 | 22 | 3 | 4 |
| 405 | 20261531 | 238 | 1190 | 26 | 1 | 3 |
| 406 | 20261532 | 211 | 908 | 17 | 4 | 1 |
| 407 | 20261533 | 184 | 795 | 10 | 1 | 0 |
| 408 | 20261534 | 237 | 1276 | 29 | 10 | 4 |
| 409 | 20261535 | 245 | 1418 | 30 | 3 | 1 |
| 410 | 20261536 | 234 | 1294 | 30 | 0 | 2 |
| 411 | 20261537 | 188 | 811 | 12 | 3 | 1 |
| 412 | 20261538 | 241 | 1174 | 22 | 6 | 4 |
| 413 | 20261539 | 258 | 1072 | 29 | 2 | 2 |
| 414 | 20261540 | 283 | 1230 | 27 | 2 | 4 |
| 415 | 20261541 | 236 | 1313 | 33 | 2 | 2 |
| 416 | 20261542 | 354 | 1814 | 40 | 1 | 4 |
| 417 | 20261543 | 226 | 902 | 20 | 1 | 3 |
| 418 | 20261544 | 324 | 1778 | 30 | 4 | 3 |
| 419 | 20261545 | 183 | 868 | 5 | 3 | 2 |
| 420 | 20261546 | 258 | 1253 | 35 | 9 | 3 |
| 421 | 20261547 | 227 | 1328 | 25 | 2 | 4 |
| 422 | 20261548 | 227 | 1088 | 23 | 2 | 1 |
| 423 | 20261549 | 241 | 1395 | 23 | 2 | 1 |
| 424 | 20261550 | 236 | 1434 | 23 | 1 | 3 |
| 425 | 20261551 | 218 | 1127 | 19 | 4 | 1 |
| 426 | 20261552 | 363 | 2180 | 39 | 4 | 2 |
| 427 | 20261553 | 235 | 1315 | 31 | 2 | 3 |
| 428 | 20261554 | 264 | 1570 | 28 | 7 | 3 |
| 429 | 20261555 | 266 | 1278 | 25 | 7 | 3 |
| 430 | 20261556 | 226 | 1249 | 11 | 4 | 3 |
| 431 | 20261557 | 216 | 979 | 27 | 5 | 5 |
| 432 | 20261558 | 238 | 1312 | 28 | 4 | 1 |
| 433 | 20261559 | 200 | 1094 | 17 | 4 | 2 |
| 434 | 20261560 | 201 | 1336 | 19 | 0 | 2 |
| 435 | 20261561 | 249 | 1078 | 24 | 6 | 3 |
| 436 | 20261562 | 345 | 1483 | 31 | 4 | 4 |
| 437 | 20261563 | 350 | 2295 | 34 | 8 | 3 |
| 438 | 20261564 | 226 | 1115 | 22 | 3 | 2 |
| 439 | 20261565 | 211 | 1076 | 20 | 6 | 3 |
| 440 | 20261566 | 222 | 1041 | 18 | 5 | 1 |
| 441 | 20261567 | 269 | 1429 | 31 | 6 | 4 |
| 442 | 20261568 | 252 | 1413 | 11 | 10 | 1 |
| 443 | 20261569 | 163 | 606 | 16 | 4 | 3 |
| 444 | 20261570 | 221 | 1266 | 13 | 7 | 1 |
| 445 | 20261571 | 182 | 971 | 7 | 8 | 2 |
| 446 | 20261572 | 258 | 1421 | 34 | 11 | 1 |
| 447 | 20261573 | 251 | 1458 | 27 | 2 | 3 |
| 448 | 20261574 | 320 | 1754 | 30 | 5 | 1 |
| 449 | 20261575 | 245 | 1222 | 31 | 4 | 2 |
| 450 | 20261576 | 276 | 1421 | 27 | 6 | 5 |
| 451 | 20261577 | 215 | 1003 | 22 | 4 | 4 |
| 452 | 20261578 | 426 | 2577 | 34 | 8 | 4 |
| 453 | 20261579 | 239 | 1039 | 35 | 6 | 3 |
| 454 | 20261580 | 310 | 1460 | 38 | 6 | 3 |
| 455 | 20261581 | 217 | 1243 | 28 | 9 | 3 |
| 456 | 20261582 | 182 | 958 | 12 | 9 | 1 |
| 457 | 20261583 | 277 | 1108 | 12 | 7 | 3 |
| 458 | 20261584 | 232 | 1313 | 23 | 10 | 2 |
| 459 | 20261585 | 229 | 1036 | 34 | 2 | 2 |
| 460 | 20261586 | 357 | 2059 | 38 | 1 | 4 |
| 461 | 20261587 | 213 | 1035 | 16 | 3 | 3 |
| 462 | 20261588 | 233 | 927 | 17 | 0 | 2 |
| 463 | 20261589 | 241 | 1137 | 24 | 2 | 3 |
| 464 | 20261590 | 202 | 976 | 14 | 3 | 3 |
| 465 | 20261591 | 243 | 1237 | 23 | 0 | 1 |
| 466 | 20261592 | 314 | 1891 | 19 | 3 | 2 |
| 467 | 20261593 | 249 | 1529 | 19 | 7 | 3 |
| 468 | 20261594 | 234 | 1281 | 20 | 2 | 4 |
| 469 | 20261595 | 321 | 1670 | 30 | 7 | 4 |
| 470 | 20261596 | 214 | 914 | 21 | 4 | 2 |
| 471 | 20261597 | 412 | 2748 | 35 | 1 | 3 |
| 472 | 20261598 | 500 | 2242 | 71 | 7 | 4 |
| 473 | 20261599 | 298 | 1820 | 14 | 0 | 4 |
| 474 | 20261600 | 275 | 1345 | 17 | 1 | 3 |
| 475 | 20261601 | 410 | 2497 | 59 | 9 | 4 |
| 476 | 20261602 | 180 | 773 | 17 | 2 | 3 |
| 477 | 20261603 | 277 | 1524 | 27 | 1 | 2 |
| 478 | 20261604 | 242 | 1172 | 15 | 5 | 5 |
| 479 | 20261605 | 293 | 1800 | 33 | 2 | 3 |
| 480 | 20261606 | 247 | 1321 | 35 | 5 | 1 |
| 481 | 20261607 | 214 | 904 | 19 | 6 | 2 |
| 482 | 20261608 | 209 | 1003 | 22 | 3 | 3 |
| 483 | 20261609 | 218 | 956 | 20 | 3 | 1 |
| 484 | 20261610 | 210 | 961 | 20 | 2 | 2 |
| 485 | 20261611 | 277 | 1302 | 29 | 10 | 5 |
| 486 | 20261612 | 163 | 487 | 6 | 1 | 2 |
| 487 | 20261613 | 257 | 1032 | 18 | 1 | 4 |
| 488 | 20261614 | 329 | 1608 | 30 | 5 | 2 |
| 489 | 20261615 | 248 | 1017 | 15 | 3 | 1 |
| 490 | 20261616 | 331 | 1906 | 47 | 9 | 2 |
| 491 | 20261617 | 252 | 1208 | 16 | 2 | 4 |
| 492 | 20261618 | 340 | 2014 | 25 | 5 | 1 |
| 493 | 20261619 | 232 | 1087 | 31 | 5 | 3 |
| 494 | 20261620 | 268 | 1229 | 37 | 3 | 3 |
| 495 | 20261621 | 198 | 937 | 21 | 2 | 3 |
| 496 | 20261622 | 242 | 1093 | 23 | 4 | 3 |
| 497 | 20261623 | 248 | 1378 | 23 | 3 | 5 |
| 498 | 20261624 | 214 | 932 | 16 | 8 | 2 |
| 499 | 20261625 | 258 | 1449 | 24 | 11 | 1 |
| 500 | 20261626 | 276 | 990 | 28 | 0 | 3 |
| 501 | 20261627 | 290 | 1389 | 22 | 2 | 2 |
| 502 | 20261628 | 240 | 1169 | 18 | 2 | 4 |
| 503 | 20261629 | 250 | 1176 | 33 | 1 | 2 |
| 504 | 20261630 | 282 | 1675 | 25 | 8 | 6 |
| 505 | 20261631 | 205 | 1052 | 15 | 3 | 1 |
| 506 | 20261632 | 206 | 922 | 17 | 1 | 3 |
| 507 | 20261633 | 266 | 1692 | 29 | 8 | 2 |
| 508 | 20261634 | 197 | 1092 | 19 | 5 | 1 |
| 509 | 20261635 | 426 | 1993 | 54 | 5 | 4 |
| 510 | 20261636 | 228 | 1019 | 17 | 6 | 2 |
| 511 | 20261637 | 271 | 1391 | 18 | 2 | 4 |
| 512 | 20261638 | 229 | 1270 | 17 | 0 | 0 |
| 513 | 20261639 | 226 | 910 | 15 | 5 | 0 |
| 514 | 20261640 | 284 | 1603 | 33 | 3 | 3 |
| 515 | 20261641 | 238 | 1149 | 31 | 2 | 5 |
| 516 | 20261642 | 227 | 1304 | 14 | 7 | 2 |
| 517 | 20261643 | 242 | 1224 | 23 | 3 | 4 |
| 518 | 20261644 | 190 | 746 | 6 | 1 | 3 |
| 519 | 20261645 | 399 | 2177 | 39 | 4 | 3 |
| 520 | 20261646 | 335 | 1682 | 26 | 6 | 4 |
| 521 | 20261647 | 212 | 840 | 25 | 5 | 2 |
| 522 | 20261648 | 252 | 1196 | 21 | 9 | 4 |
| 523 | 20261649 | 263 | 1383 | 27 | 11 | 3 |
| 524 | 20261650 | 187 | 941 | 17 | 2 | 0 |
| 525 | 20261651 | 265 | 1337 | 35 | 4 | 3 |
| 526 | 20261652 | 233 | 1399 | 11 | 5 | 4 |
| 527 | 20261653 | 285 | 1512 | 19 | 2 | 6 |
| 528 | 20261654 | 213 | 715 | 13 | 4 | 3 |
| 529 | 20261655 | 327 | 2209 | 47 | 7 | 3 |
| 530 | 20261656 | 293 | 1326 | 15 | 6 | 5 |
| 531 | 20261657 | 279 | 1228 | 18 | 3 | 4 |
| 532 | 20261658 | 182 | 856 | 17 | 3 | 2 |
| 533 | 20261659 | 255 | 1220 | 23 | 1 | 1 |
| 534 | 20261660 | 173 | 623 | 5 | 2 | 1 |
| 535 | 20261661 | 280 | 1359 | 34 | 5 | 2 |
| 536 | 20261662 | 227 | 1169 | 28 | 0 | 1 |
| 537 | 20261663 | 289 | 1553 | 39 | 6 | 4 |
| 538 | 20261664 | 194 | 932 | 18 | 4 | 3 |
| 539 | 20261665 | 296 | 1643 | 40 | 8 | 4 |
| 540 | 20261666 | 278 | 1296 | 34 | 8 | 3 |
| 541 | 20261667 | 230 | 1109 | 34 | 1 | 4 |
| 542 | 20261668 | 285 | 1520 | 29 | 5 | 3 |
| 543 | 20261669 | 169 | 784 | 16 | 4 | 1 |
| 544 | 20261670 | 227 | 957 | 19 | 8 | 4 |
| 545 | 20261671 | 223 | 1097 | 23 | 6 | 3 |
| 546 | 20261672 | 266 | 1015 | 24 | 3 | 3 |
| 547 | 20261673 | 292 | 1831 | 37 | 4 | 5 |
| 548 | 20261674 | 344 | 2095 | 50 | 4 | 3 |
| 549 | 20261675 | 212 | 913 | 15 | 5 | 1 |
| 550 | 20261676 | 207 | 932 | 18 | 3 | 2 |
| 551 | 20261677 | 262 | 922 | 22 | 0 | 2 |
| 552 | 20261678 | 181 | 1093 | 18 | 4 | 2 |
| 553 | 20261679 | 235 | 1120 | 14 | 5 | 4 |
| 554 | 20261680 | 269 | 1302 | 33 | 8 | 2 |
| 555 | 20261681 | 237 | 1114 | 11 | 5 | 4 |
| 556 | 20261682 | 260 | 945 | 20 | 8 | 2 |
| 557 | 20261683 | 193 | 960 | 15 | 1 | 2 |
| 558 | 20261684 | 355 | 2268 | 41 | 9 | 1 |
| 559 | 20261685 | 194 | 841 | 16 | 3 | 4 |
| 560 | 20261686 | 213 | 1203 | 24 | 0 | 3 |
| 561 | 20261687 | 300 | 1584 | 41 | 6 | 2 |
| 562 | 20261688 | 321 | 1676 | 29 | 3 | 2 |
| 563 | 20261689 | 189 | 1002 | 23 | 2 | 1 |
| 564 | 20261690 | 215 | 860 | 17 | 6 | 3 |
| 565 | 20261691 | 279 | 1028 | 29 | 5 | 5 |
| 566 | 20261692 | 228 | 1253 | 26 | 4 | 3 |
| 567 | 20261693 | 227 | 1207 | 29 | 2 | 4 |
| 568 | 20261694 | 169 | 574 | 14 | 2 | 3 |
| 569 | 20261695 | 307 | 1205 | 24 | 2 | 3 |
| 570 | 20261696 | 360 | 1338 | 32 | 0 | 4 |
| 571 | 20261697 | 320 | 1654 | 21 | 4 | 3 |
| 572 | 20261698 | 209 | 932 | 20 | 7 | 2 |
| 573 | 20261699 | 198 | 878 | 25 | 1 | 2 |
| 574 | 20261700 | 184 | 846 | 24 | 5 | 3 |
| 575 | 20261701 | 301 | 1451 | 16 | 2 | 3 |
| 576 | 20261702 | 222 | 1330 | 27 | 1 | 2 |
| 577 | 20261703 | 277 | 1389 | 40 | 8 | 2 |
| 578 | 20261704 | 257 | 1259 | 11 | 6 | 2 |
| 579 | 20261705 | 199 | 970 | 13 | 4 | 2 |
| 580 | 20261706 | 307 | 1732 | 42 | 3 | 3 |
| 581 | 20261707 | 196 | 844 | 25 | 2 | 3 |
| 582 | 20261708 | 218 | 1016 | 16 | 2 | 1 |
| 583 | 20261709 | 205 | 899 | 19 | 1 | 1 |
| 584 | 20261710 | 285 | 1231 | 26 | 9 | 4 |
| 585 | 20261711 | 231 | 1081 | 27 | 7 | 2 |
| 586 | 20261712 | 261 | 1081 | 27 | 3 | 3 |
| 587 | 20261713 | 294 | 1454 | 25 | 2 | 3 |
| 588 | 20261714 | 241 | 1114 | 13 | 2 | 4 |
| 589 | 20261715 | 200 | 755 | 11 | 4 | 2 |
| 590 | 20261716 | 276 | 1583 | 21 | 3 | 3 |
| 591 | 20261717 | 222 | 1178 | 15 | 3 | 3 |
| 592 | 20261718 | 291 | 1547 | 28 | 2 | 3 |
| 593 | 20261719 | 287 | 1718 | 25 | 2 | 1 |
| 594 | 20261720 | 275 | 1415 | 18 | 4 | 1 |
| 595 | 20261721 | 183 | 831 | 15 | 4 | 3 |
| 596 | 20261722 | 265 | 1199 | 33 | 1 | 3 |
| 597 | 20261723 | 227 | 1011 | 36 | 6 | 2 |
| 598 | 20261724 | 296 | 1426 | 37 | 4 | 1 |
| 599 | 20261725 | 214 | 1185 | 19 | 6 | 2 |
| 600 | 20261726 | 266 | 1238 | 41 | 5 | 2 |
| 601 | 20261727 | 234 | 1060 | 25 | 7 | 3 |
| 602 | 20261728 | 270 | 1449 | 25 | 3 | 1 |
| 603 | 20261729 | 218 | 919 | 16 | 3 | 3 |
| 604 | 20261730 | 245 | 1169 | 30 | 3 | 4 |
| 605 | 20261731 | 237 | 1031 | 37 | 5 | 2 |
| 606 | 20261732 | 329 | 1694 | 31 | 3 | 2 |
| 607 | 20261733 | 259 | 1116 | 30 | 2 | 2 |
| 608 | 20261734 | 268 | 1016 | 17 | 2 | 2 |
| 609 | 20261735 | 245 | 1605 | 31 | 7 | 3 |
| 610 | 20261736 | 177 | 486 | 6 | 1 | 2 |
| 611 | 20261737 | 356 | 1947 | 28 | 4 | 4 |
| 612 | 20261738 | 315 | 1827 | 28 | 7 | 2 |
| 613 | 20261739 | 320 | 1739 | 35 | 4 | 2 |
| 614 | 20261740 | 213 | 1180 | 18 | 0 | 3 |
| 615 | 20261741 | 207 | 1025 | 12 | 4 | 2 |
| 616 | 20261742 | 309 | 1535 | 31 | 6 | 2 |
| 617 | 20261743 | 247 | 1326 | 28 | 11 | 3 |
| 618 | 20261744 | 189 | 815 | 16 | 3 | 4 |
| 619 | 20261745 | 342 | 1878 | 18 | 3 | 1 |
| 620 | 20261746 | 288 | 1082 | 21 | 4 | 5 |
| 621 | 20261747 | 241 | 1071 | 16 | 6 | 3 |
| 622 | 20261748 | 247 | 1273 | 30 | 4 | 3 |
| 623 | 20261749 | 290 | 1717 | 31 | 4 | 4 |
| 624 | 20261750 | 203 | 971 | 20 | 2 | 3 |
| 625 | 20261751 | 207 | 1075 | 15 | 3 | 2 |
| 626 | 20261752 | 288 | 1741 | 40 | 9 | 4 |
| 627 | 20261753 | 273 | 1368 | 13 | 7 | 2 |
| 628 | 20261754 | 211 | 879 | 7 | 2 | 4 |
| 629 | 20261755 | 255 | 1617 | 41 | 5 | 4 |
| 630 | 20261756 | 184 | 875 | 13 | 1 | 2 |
| 631 | 20261757 | 268 | 1324 | 24 | 4 | 2 |
| 632 | 20261758 | 236 | 1326 | 23 | 6 | 3 |
| 633 | 20261759 | 266 | 1041 | 16 | 5 | 3 |
| 634 | 20261760 | 232 | 895 | 15 | 0 | 3 |
| 635 | 20261761 | 377 | 1966 | 44 | 1 | 4 |
| 636 | 20261762 | 208 | 660 | 11 | 2 | 3 |
| 637 | 20261763 | 169 | 676 | 11 | 4 | 1 |
| 638 | 20261764 | 269 | 1459 | 31 | 2 | 2 |
| 639 | 20261765 | 176 | 803 | 15 | 6 | 3 |
| 640 | 20261766 | 356 | 1764 | 26 | 1 | 3 |
| 641 | 20261767 | 208 | 1217 | 25 | 7 | 2 |
| 642 | 20261768 | 248 | 1194 | 20 | 6 | 2 |
| 643 | 20261769 | 204 | 1037 | 15 | 1 | 1 |
| 644 | 20261770 | 217 | 906 | 15 | 9 | 3 |
| 645 | 20261771 | 270 | 1107 | 25 | 2 | 1 |
| 646 | 20261772 | 209 | 1162 | 22 | 5 | 0 |
| 647 | 20261773 | 249 | 1198 | 31 | 2 | 1 |
| 648 | 20261774 | 237 | 1422 | 39 | 4 | 2 |
| 649 | 20261775 | 289 | 1517 | 24 | 1 | 5 |
| 650 | 20261776 | 279 | 1590 | 34 | 4 | 4 |
| 651 | 20261777 | 303 | 1777 | 34 | 0 | 1 |
| 652 | 20261778 | 241 | 1100 | 33 | 4 | 3 |
| 653 | 20261779 | 189 | 964 | 17 | 1 | 1 |
| 654 | 20261780 | 368 | 1758 | 33 | 2 | 6 |
| 655 | 20261781 | 229 | 1290 | 30 | 3 | 3 |
| 656 | 20261782 | 253 | 1131 | 30 | 5 | 5 |
| 657 | 20261783 | 286 | 1726 | 34 | 10 | 2 |
| 658 | 20261784 | 179 | 864 | 14 | 3 | 1 |
| 659 | 20261785 | 399 | 1926 | 45 | 2 | 6 |
| 660 | 20261786 | 270 | 1617 | 34 | 6 | 1 |
| 661 | 20261787 | 276 | 1489 | 33 | 3 | 1 |
| 662 | 20261788 | 242 | 1186 | 22 | 1 | 3 |
| 663 | 20261789 | 235 | 1058 | 22 | 5 | 3 |
| 664 | 20261790 | 215 | 1002 | 21 | 2 | 3 |
| 665 | 20261791 | 207 | 963 | 25 | 3 | 2 |
| 666 | 20261792 | 194 | 871 | 14 | 1 | 1 |
| 667 | 20261793 | 218 | 897 | 15 | 4 | 1 |
| 668 | 20261794 | 500 | 2453 | 66 | 9 | 5 |
| 669 | 20261795 | 268 | 1259 | 18 | 7 | 6 |
| 670 | 20261796 | 227 | 918 | 18 | 7 | 1 |
| 671 | 20261797 | 177 | 748 | 8 | 5 | 1 |
| 672 | 20261798 | 217 | 1280 | 21 | 8 | 4 |
| 673 | 20261799 | 183 | 682 | 16 | 3 | 1 |
| 674 | 20261800 | 355 | 1880 | 34 | 6 | 3 |
| 675 | 20261801 | 178 | 725 | 13 | 3 | 2 |
| 676 | 20261802 | 228 | 1424 | 34 | 2 | 3 |
| 677 | 20261803 | 254 | 1386 | 35 | 2 | 4 |
| 678 | 20261804 | 218 | 967 | 18 | 9 | 3 |
| 679 | 20261805 | 229 | 1137 | 14 | 2 | 3 |
| 680 | 20261806 | 208 | 1176 | 16 | 0 | 1 |
| 681 | 20261807 | 286 | 1556 | 39 | 4 | 4 |
| 682 | 20261808 | 318 | 1891 | 20 | 10 | 2 |
| 683 | 20261809 | 249 | 1342 | 28 | 9 | 1 |
| 684 | 20261810 | 217 | 861 | 27 | 3 | 2 |
| 685 | 20261811 | 361 | 1759 | 36 | 4 | 3 |
| 686 | 20261812 | 228 | 829 | 21 | 7 | 2 |
| 687 | 20261813 | 241 | 1119 | 10 | 2 | 3 |
| 688 | 20261814 | 286 | 1338 | 27 | 2 | 3 |
| 689 | 20261815 | 220 | 999 | 19 | 1 | 2 |
| 690 | 20261816 | 267 | 1330 | 28 | 3 | 2 |
| 691 | 20261817 | 372 | 2016 | 23 | 11 | 5 |
| 692 | 20261818 | 301 | 1506 | 36 | 4 | 3 |
| 693 | 20261819 | 219 | 1078 | 21 | 2 | 1 |
| 694 | 20261820 | 226 | 842 | 15 | 5 | 2 |
| 695 | 20261821 | 173 | 534 | 12 | 5 | 3 |
| 696 | 20261822 | 500 | 2542 | 39 | 7 | 3 |
| 697 | 20261823 | 239 | 1151 | 23 | 4 | 3 |
| 698 | 20261824 | 196 | 710 | 15 | 5 | 2 |
| 699 | 20261825 | 255 | 1414 | 28 | 3 | 3 |
| 700 | 20261826 | 246 | 1354 | 32 | 3 | 4 |
| 701 | 20261827 | 344 | 1600 | 29 | 7 | 4 |
| 702 | 20261828 | 365 | 1881 | 36 | 4 | 2 |
| 703 | 20261829 | 319 | 1413 | 19 | 4 | 3 |
| 704 | 20261830 | 248 | 1384 | 21 | 2 | 3 |
| 705 | 20261831 | 206 | 864 | 13 | 3 | 1 |
| 706 | 20261832 | 280 | 1482 | 24 | 2 | 2 |
| 707 | 20261833 | 173 | 905 | 11 | 2 | 2 |
| 708 | 20261834 | 240 | 1099 | 14 | 7 | 0 |
| 709 | 20261835 | 206 | 787 | 10 | 5 | 2 |
| 710 | 20261836 | 284 | 1500 | 25 | 0 | 1 |
| 711 | 20261837 | 185 | 1031 | 22 | 2 | 3 |
| 712 | 20261838 | 188 | 738 | 11 | 8 | 1 |
| 713 | 20261839 | 222 | 1090 | 21 | 1 | 3 |
| 714 | 20261840 | 232 | 982 | 11 | 1 | 4 |
| 715 | 20261841 | 179 | 1003 | 9 | 4 | 1 |
| 716 | 20261842 | 276 | 1507 | 24 | 5 | 2 |
| 717 | 20261843 | 241 | 1243 | 21 | 7 | 4 |
| 718 | 20261844 | 281 | 1529 | 25 | 8 | 2 |
| 719 | 20261845 | 250 | 1364 | 29 | 1 | 1 |
| 720 | 20261846 | 195 | 782 | 14 | 1 | 1 |
| 721 | 20261847 | 189 | 865 | 20 | 2 | 1 |
| 722 | 20261848 | 209 | 692 | 20 | 2 | 4 |
| 723 | 20261849 | 242 | 1232 | 35 | 7 | 1 |
| 724 | 20261850 | 207 | 1194 | 21 | 5 | 1 |
| 725 | 20261851 | 277 | 1549 | 23 | 4 | 3 |
| 726 | 20261852 | 254 | 1385 | 18 | 5 | 2 |
| 727 | 20261853 | 200 | 951 | 13 | 3 | 2 |
| 728 | 20261854 | 220 | 1091 | 23 | 2 | 1 |
| 729 | 20261855 | 198 | 1362 | 23 | 1 | 1 |
| 730 | 20261856 | 272 | 1216 | 25 | 4 | 3 |
| 731 | 20261857 | 195 | 914 | 12 | 3 | 1 |
| 732 | 20261858 | 244 | 1410 | 17 | 1 | 3 |
| 733 | 20261859 | 223 | 1068 | 25 | 2 | 4 |
| 734 | 20261860 | 239 | 1345 | 17 | 6 | 1 |
| 735 | 20261861 | 230 | 1318 | 22 | 3 | 3 |
| 736 | 20261862 | 261 | 1378 | 34 | 7 | 4 |
| 737 | 20261863 | 342 | 1883 | 29 | 2 | 6 |
| 738 | 20261864 | 271 | 1250 | 35 | 2 | 4 |
| 739 | 20261865 | 173 | 820 | 13 | 1 | 2 |
| 740 | 20261866 | 254 | 1248 | 17 | 2 | 0 |
| 741 | 20261867 | 315 | 1905 | 36 | 6 | 3 |
| 742 | 20261868 | 319 | 1699 | 28 | 25 | 1 |
| 743 | 20261869 | 247 | 1084 | 31 | 8 | 3 |
| 744 | 20261870 | 219 | 981 | 22 | 1 | 2 |
| 745 | 20261871 | 433 | 2000 | 34 | 4 | 3 |
| 746 | 20261872 | 240 | 1239 | 20 | 2 | 2 |
| 747 | 20261873 | 187 | 877 | 18 | 4 | 3 |
| 748 | 20261874 | 332 | 2194 | 23 | 5 | 4 |
| 749 | 20261875 | 214 | 1022 | 18 | 3 | 0 |
| 750 | 20261876 | 222 | 1182 | 27 | 2 | 2 |
| 751 | 20261877 | 173 | 735 | 13 | 4 | 3 |
| 752 | 20261878 | 189 | 844 | 12 | 9 | 1 |
| 753 | 20261879 | 290 | 1674 | 27 | 4 | 4 |
| 754 | 20261880 | 222 | 1378 | 16 | 5 | 0 |
| 755 | 20261881 | 179 | 819 | 19 | 4 | 3 |
| 756 | 20261882 | 267 | 1434 | 21 | 0 | 2 |
| 757 | 20261883 | 232 | 1168 | 27 | 4 | 1 |
| 758 | 20261884 | 222 | 1149 | 26 | 5 | 0 |
| 759 | 20261885 | 272 | 1330 | 35 | 4 | 1 |
| 760 | 20261886 | 232 | 1205 | 24 | 3 | 3 |
| 761 | 20261887 | 293 | 1438 | 30 | 4 | 3 |
| 762 | 20261888 | 229 | 1285 | 21 | 7 | 2 |
| 763 | 20261889 | 235 | 1312 | 21 | 2 | 3 |
| 764 | 20261890 | 386 | 2183 | 32 | 6 | 3 |
| 765 | 20261891 | 265 | 1000 | 36 | 7 | 4 |
| 766 | 20261892 | 224 | 886 | 20 | 3 | 1 |
| 767 | 20261893 | 274 | 1421 | 39 | 3 | 3 |
| 768 | 20261894 | 227 | 1096 | 10 | 8 | 3 |
| 769 | 20261895 | 220 | 1001 | 23 | 2 | 2 |
| 770 | 20261896 | 186 | 772 | 15 | 3 | 1 |
| 771 | 20261897 | 226 | 1081 | 22 | 3 | 2 |
| 772 | 20261898 | 198 | 1003 | 22 | 6 | 2 |
| 773 | 20261899 | 314 | 1808 | 24 | 0 | 2 |
| 774 | 20261900 | 288 | 1193 | 20 | 9 | 5 |
| 775 | 20261901 | 281 | 1176 | 37 | 4 | 4 |
| 776 | 20261902 | 228 | 1417 | 18 | 13 | 0 |
| 777 | 20261903 | 260 | 1264 | 29 | 8 | 3 |
| 778 | 20261904 | 238 | 1335 | 28 | 5 | 3 |
| 779 | 20261905 | 207 | 928 | 22 | 2 | 2 |
| 780 | 20261906 | 284 | 1383 | 18 | 4 | 2 |
| 781 | 20261907 | 281 | 1257 | 24 | 10 | 5 |
| 782 | 20261908 | 301 | 1675 | 20 | 7 | 5 |
| 783 | 20261909 | 171 | 867 | 17 | 3 | 1 |
| 784 | 20261910 | 268 | 1604 | 20 | 4 | 2 |
| 785 | 20261911 | 245 | 1211 | 29 | 1 | 4 |
| 786 | 20261912 | 355 | 1588 | 32 | 1 | 4 |
| 787 | 20261913 | 218 | 680 | 11 | 11 | 1 |
| 788 | 20261914 | 256 | 1396 | 17 | 6 | 3 |
| 789 | 20261915 | 277 | 999 | 31 | 7 | 2 |
| 790 | 20261916 | 213 | 977 | 19 | 2 | 1 |
| 791 | 20261917 | 273 | 1482 | 38 | 3 | 2 |
| 792 | 20261918 | 238 | 913 | 15 | 3 | 1 |
| 793 | 20261919 | 221 | 1063 | 33 | 4 | 3 |
| 794 | 20261920 | 395 | 2030 | 39 | 0 | 5 |
| 795 | 20261921 | 238 | 1043 | 18 | 3 | 2 |
| 796 | 20261922 | 239 | 1170 | 26 | 7 | 5 |
| 797 | 20261923 | 284 | 1267 | 26 | 2 | 4 |
| 798 | 20261924 | 208 | 1206 | 24 | 3 | 3 |
| 799 | 20261925 | 259 | 1254 | 32 | 1 | 3 |
| 800 | 20261926 | 294 | 1557 | 25 | 2 | 3 |
| 801 | 20261927 | 205 | 868 | 26 | 3 | 3 |
| 802 | 20261928 | 199 | 705 | 10 | 2 | 2 |
| 803 | 20261929 | 225 | 1027 | 18 | 0 | 2 |
| 804 | 20261930 | 271 | 1274 | 17 | 2 | 4 |
| 805 | 20261931 | 266 | 1190 | 22 | 5 | 4 |
| 806 | 20261932 | 280 | 1636 | 21 | 3 | 6 |
| 807 | 20261933 | 246 | 1336 | 29 | 8 | 2 |
| 808 | 20261934 | 199 | 950 | 8 | 3 | 2 |
| 809 | 20261935 | 273 | 1267 | 27 | 2 | 2 |
| 810 | 20261936 | 351 | 1867 | 45 | 3 | 2 |
| 811 | 20261937 | 239 | 1369 | 20 | 1 | 2 |
| 812 | 20261938 | 383 | 2167 | 40 | 4 | 4 |
| 813 | 20261939 | 206 | 752 | 23 | 2 | 3 |
| 814 | 20261940 | 240 | 1367 | 13 | 1 | 2 |
| 815 | 20261941 | 267 | 1360 | 32 | 5 | 1 |
| 816 | 20261942 | 307 | 1566 | 19 | 6 | 4 |
| 817 | 20261943 | 182 | 922 | 25 | 3 | 3 |
| 818 | 20261944 | 253 | 1397 | 15 | 3 | 2 |
| 819 | 20261945 | 261 | 1417 | 11 | 7 | 2 |
| 820 | 20261946 | 243 | 1227 | 16 | 6 | 3 |
| 821 | 20261947 | 181 | 788 | 9 | 4 | 2 |
| 822 | 20261948 | 181 | 877 | 16 | 7 | 2 |
| 823 | 20261949 | 211 | 1036 | 17 | 10 | 3 |
| 824 | 20261950 | 342 | 1680 | 33 | 1 | 4 |
| 825 | 20261951 | 295 | 1342 | 38 | 6 | 4 |
| 826 | 20261952 | 213 | 832 | 14 | 1 | 2 |
| 827 | 20261953 | 247 | 1215 | 21 | 5 | 2 |
| 828 | 20261954 | 231 | 1215 | 39 | 1 | 3 |
| 829 | 20261955 | 240 | 1301 | 26 | 6 | 3 |
| 830 | 20261956 | 171 | 793 | 17 | 5 | 3 |
| 831 | 20261957 | 233 | 1222 | 24 | 7 | 3 |
| 832 | 20261958 | 277 | 1389 | 27 | 0 | 2 |
| 833 | 20261959 | 176 | 796 | 16 | 3 | 1 |
| 834 | 20261960 | 228 | 1181 | 15 | 3 | 2 |
| 835 | 20261961 | 233 | 989 | 17 | 2 | 2 |
| 836 | 20261962 | 254 | 1101 | 24 | 5 | 3 |
| 837 | 20261963 | 237 | 1098 | 21 | 5 | 3 |
| 838 | 20261964 | 180 | 870 | 12 | 5 | 1 |
| 839 | 20261965 | 203 | 982 | 24 | 6 | 2 |
| 840 | 20261966 | 307 | 1649 | 38 | 3 | 2 |
| 841 | 20261967 | 186 | 663 | 8 | 3 | 1 |
| 842 | 20261968 | 216 | 1154 | 21 | 3 | 1 |
| 843 | 20261969 | 250 | 1605 | 30 | 3 | 1 |
| 844 | 20261970 | 203 | 950 | 21 | 4 | 3 |
| 845 | 20261971 | 213 | 1098 | 16 | 3 | 3 |
| 846 | 20261972 | 394 | 2387 | 37 | 11 | 6 |
| 847 | 20261973 | 222 | 1129 | 22 | 4 | 2 |
| 848 | 20261974 | 344 | 2022 | 32 | 4 | 3 |
| 849 | 20261975 | 207 | 947 | 9 | 7 | 2 |
| 850 | 20261976 | 226 | 1038 | 23 | 3 | 1 |
| 851 | 20261977 | 196 | 1145 | 27 | 3 | 0 |
| 852 | 20261978 | 316 | 1743 | 28 | 13 | 2 |
| 853 | 20261979 | 222 | 852 | 15 | 1 | 3 |
| 854 | 20261980 | 222 | 1028 | 13 | 7 | 1 |
| 855 | 20261981 | 189 | 900 | 17 | 5 | 2 |
| 856 | 20261982 | 205 | 824 | 16 | 2 | 3 |
| 857 | 20261983 | 244 | 1197 | 25 | 10 | 4 |
| 858 | 20261984 | 256 | 1053 | 32 | 0 | 4 |
| 859 | 20261985 | 232 | 1291 | 14 | 5 | 1 |
| 860 | 20261986 | 208 | 1039 | 12 | 6 | 3 |
| 861 | 20261987 | 362 | 2080 | 43 | 6 | 4 |
| 862 | 20261988 | 282 | 1693 | 24 | 4 | 1 |
| 863 | 20261989 | 186 | 949 | 22 | 5 | 2 |
| 864 | 20261990 | 212 | 973 | 13 | 1 | 3 |
| 865 | 20261991 | 254 | 994 | 34 | 2 | 0 |
| 866 | 20261992 | 250 | 1169 | 25 | 0 | 3 |
| 867 | 20261993 | 430 | 2272 | 17 | 7 | 3 |
| 868 | 20261994 | 258 | 1278 | 9 | 3 | 4 |
| 869 | 20261995 | 263 | 1280 | 26 | 1 | 2 |
| 870 | 20261996 | 196 | 1009 | 13 | 4 | 2 |
| 871 | 20261997 | 225 | 1215 | 21 | 5 | 1 |
| 872 | 20261998 | 233 | 1271 | 23 | 2 | 3 |
| 873 | 20261999 | 214 | 967 | 13 | 3 | 2 |
| 874 | 20262000 | 243 | 1307 | 24 | 3 | 0 |
| 875 | 20262001 | 221 | 1067 | 21 | 4 | 1 |
| 876 | 20262002 | 279 | 1055 | 17 | 10 | 3 |
| 877 | 20262003 | 232 | 967 | 16 | 1 | 1 |
| 878 | 20262004 | 210 | 951 | 14 | 3 | 0 |
| 879 | 20262005 | 293 | 1484 | 34 | 6 | 2 |
| 880 | 20262006 | 192 | 784 | 9 | 3 | 3 |
| 881 | 20262007 | 252 | 1282 | 23 | 12 | 2 |
| 882 | 20262008 | 202 | 1120 | 15 | 2 | 3 |
| 883 | 20262009 | 500 | 2328 | 67 | 6 | 6 |
| 884 | 20262010 | 350 | 1642 | 16 | 6 | 3 |
| 885 | 20262011 | 250 | 1142 | 15 | 4 | 1 |
| 886 | 20262012 | 239 | 927 | 28 | 7 | 4 |
| 887 | 20262013 | 277 | 1189 | 25 | 5 | 4 |
| 888 | 20262014 | 167 | 770 | 20 | 3 | 3 |
| 889 | 20262015 | 244 | 1316 | 23 | 2 | 2 |
| 890 | 20262016 | 230 | 1165 | 22 | 1 | 3 |
| 891 | 20262017 | 300 | 1230 | 31 | 4 | 4 |
| 892 | 20262018 | 246 | 914 | 11 | 1 | 3 |
| 893 | 20262019 | 235 | 1336 | 25 | 2 | 3 |
| 894 | 20262020 | 225 | 1071 | 26 | 5 | 4 |
| 895 | 20262021 | 270 | 1589 | 29 | 3 | 3 |
| 896 | 20262022 | 194 | 657 | 14 | 6 | 3 |
| 897 | 20262023 | 219 | 1181 | 30 | 5 | 2 |
| 898 | 20262024 | 434 | 2221 | 38 | 7 | 2 |
| 899 | 20262025 | 253 | 1150 | 7 | 3 | 3 |
| 900 | 20262026 | 232 | 1334 | 23 | 1 | 2 |
| 901 | 20262027 | 282 | 1191 | 16 | 9 | 3 |
| 902 | 20262028 | 209 | 757 | 13 | 3 | 3 |
| 903 | 20262029 | 327 | 1620 | 30 | 2 | 4 |
| 904 | 20262030 | 253 | 1267 | 31 | 6 | 2 |
| 905 | 20262031 | 309 | 1889 | 15 | 4 | 5 |
| 906 | 20262032 | 275 | 1205 | 22 | 7 | 3 |
| 907 | 20262033 | 254 | 1176 | 34 | 4 | 2 |
| 908 | 20262034 | 287 | 1375 | 28 | 4 | 3 |
| 909 | 20262035 | 299 | 1716 | 29 | 4 | 1 |
| 910 | 20262036 | 286 | 1503 | 25 | 2 | 3 |
| 911 | 20262037 | 263 | 1574 | 36 | 5 | 2 |
| 912 | 20262038 | 333 | 1745 | 44 | 2 | 4 |
| 913 | 20262039 | 256 | 1332 | 25 | 3 | 1 |
| 914 | 20262040 | 198 | 866 | 21 | 5 | 3 |
| 915 | 20262041 | 169 | 628 | 11 | 3 | 3 |
| 916 | 20262042 | 260 | 1368 | 35 | 7 | 3 |
| 917 | 20262043 | 241 | 1189 | 22 | 9 | 3 |
| 918 | 20262044 | 339 | 1818 | 35 | 4 | 4 |
| 919 | 20262045 | 421 | 2528 | 61 | 4 | 4 |
| 920 | 20262046 | 220 | 1340 | 26 | 1 | 2 |
| 921 | 20262047 | 240 | 942 | 12 | 4 | 2 |
| 922 | 20262048 | 239 | 1025 | 22 | 3 | 3 |
| 923 | 20262049 | 266 | 1276 | 38 | 3 | 4 |
| 924 | 20262050 | 185 | 910 | 16 | 4 | 1 |
| 925 | 20262051 | 218 | 892 | 22 | 1 | 2 |
| 926 | 20262052 | 251 | 1254 | 35 | 3 | 3 |
| 927 | 20262053 | 233 | 1314 | 7 | 3 | 3 |
| 928 | 20262054 | 188 | 820 | 10 | 2 | 1 |
| 929 | 20262055 | 167 | 821 | 10 | 2 | 2 |
| 930 | 20262056 | 262 | 1132 | 30 | 3 | 4 |
| 931 | 20262057 | 253 | 917 | 22 | 1 | 3 |
| 932 | 20262058 | 273 | 1089 | 33 | 5 | 4 |
| 933 | 20262059 | 175 | 665 | 13 | 4 | 1 |
| 934 | 20262060 | 293 | 1866 | 34 | 5 | 2 |
| 935 | 20262061 | 230 | 955 | 20 | 1 | 4 |
| 936 | 20262062 | 296 | 1653 | 34 | 2 | 5 |
| 937 | 20262063 | 175 | 702 | 14 | 2 | 2 |
| 938 | 20262064 | 493 | 2876 | 51 | 2 | 4 |
| 939 | 20262065 | 272 | 1489 | 24 | 6 | 4 |
| 940 | 20262066 | 211 | 1096 | 26 | 2 | 3 |
| 941 | 20262067 | 288 | 1514 | 22 | 2 | 5 |
| 942 | 20262068 | 297 | 1185 | 42 | 7 | 5 |
| 943 | 20262069 | 272 | 1391 | 31 | 1 | 4 |
| 944 | 20262070 | 233 | 1185 | 25 | 3 | 3 |
| 945 | 20262071 | 267 | 1641 | 30 | 4 | 2 |
| 946 | 20262072 | 161 | 774 | 10 | 5 | 3 |
| 947 | 20262073 | 251 | 1063 | 14 | 4 | 2 |
| 948 | 20262074 | 208 | 1077 | 18 | 0 | 2 |
| 949 | 20262075 | 263 | 1171 | 17 | 10 | 2 |
| 950 | 20262076 | 235 | 1053 | 27 | 4 | 3 |
| 951 | 20262077 | 221 | 1299 | 23 | 7 | 2 |
| 952 | 20262078 | 214 | 746 | 12 | 4 | 4 |
| 953 | 20262079 | 212 | 1011 | 24 | 5 | 2 |
| 954 | 20262080 | 321 | 1711 | 51 | 3 | 2 |
| 955 | 20262081 | 286 | 1567 | 25 | 6 | 4 |
| 956 | 20262082 | 263 | 1166 | 18 | 16 | 3 |
| 957 | 20262083 | 234 | 1057 | 19 | 7 | 3 |
| 958 | 20262084 | 282 | 1508 | 37 | 5 | 2 |
| 959 | 20262085 | 237 | 1237 | 16 | 8 | 4 |
| 960 | 20262086 | 244 | 1082 | 17 | 12 | 3 |
| 961 | 20262087 | 268 | 1238 | 25 | 1 | 3 |
| 962 | 20262088 | 200 | 914 | 14 | 5 | 2 |
| 963 | 20262089 | 221 | 947 | 17 | 3 | 3 |
| 964 | 20262090 | 244 | 1328 | 15 | 2 | 3 |
| 965 | 20262091 | 190 | 1018 | 12 | 2 | 2 |
| 966 | 20262092 | 193 | 940 | 12 | 1 | 2 |
| 967 | 20262093 | 244 | 1255 | 16 | 5 | 3 |
| 968 | 20262094 | 200 | 1161 | 22 | 7 | 2 |
| 969 | 20262095 | 238 | 1122 | 20 | 6 | 2 |
| 970 | 20262096 | 253 | 975 | 29 | 1 | 4 |
| 971 | 20262097 | 275 | 1211 | 20 | 0 | 3 |
| 972 | 20262098 | 260 | 1671 | 25 | 1 | 2 |
| 973 | 20262099 | 233 | 985 | 10 | 1 | 2 |
| 974 | 20262100 | 221 | 1159 | 26 | 7 | 2 |
| 975 | 20262101 | 313 | 1463 | 25 | 9 | 4 |
| 976 | 20262102 | 237 | 1295 | 23 | 4 | 2 |
| 977 | 20262103 | 212 | 1099 | 18 | 3 | 3 |
| 978 | 20262104 | 185 | 1078 | 11 | 3 | 0 |
| 979 | 20262105 | 274 | 1061 | 23 | 1 | 3 |
| 980 | 20262106 | 295 | 1471 | 19 | 6 | 1 |
| 981 | 20262107 | 230 | 1109 | 19 | 7 | 4 |
| 982 | 20262108 | 217 | 797 | 19 | 1 | 3 |
| 983 | 20262109 | 210 | 888 | 16 | 3 | 0 |
| 984 | 20262110 | 257 | 1205 | 20 | 3 | 2 |
| 985 | 20262111 | 197 | 1184 | 18 | 0 | 2 |
| 986 | 20262112 | 235 | 1371 | 13 | 6 | 5 |
| 987 | 20262113 | 191 | 1025 | 15 | 2 | 0 |
| 988 | 20262114 | 211 | 1021 | 19 | 5 | 2 |
| 989 | 20262115 | 227 | 1156 | 18 | 1 | 2 |
| 990 | 20262116 | 476 | 2212 | 37 | 10 | 4 |
| 991 | 20262117 | 368 | 2040 | 39 | 1 | 3 |
| 992 | 20262118 | 308 | 1959 | 31 | 7 | 3 |
| 993 | 20262119 | 315 | 1356 | 13 | 5 | 2 |
| 994 | 20262120 | 228 | 1149 | 31 | 6 | 4 |
| 995 | 20262121 | 249 | 1304 | 20 | 3 | 4 |
| 996 | 20262122 | 175 | 687 | 16 | 3 | 1 |
| 997 | 20262123 | 230 | 1271 | 18 | 5 | 2 |
| 998 | 20262124 | 306 | 1226 | 21 | 2 | 3 |
| 999 | 20262125 | 227 | 1165 | 17 | 8 | 2 |

## Throughput

- Turns simulated: 252352 (first play of each game)
- Aggregate: 107.6 turns/s over 2346.1s wall
- Per game thread: 26.9 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

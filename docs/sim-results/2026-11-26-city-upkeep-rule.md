# Simulation: city-upkeep-rule (2026-11-26)

1000 games, first seed 20261126, brain `scripted`, turn limit off, safety cap 1200, 4 threads, started 2026-11-26T20:22:50Z. Wall time 1967.9s.

Source checkout at run start: `92e568ec41551fd6159e30943af3c4107cdac534`; dirty: no. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.

Command argv: `["/home/user/legatum/target/release/legatum-sim","--games","1000","--seed","20261126","--threads","4","--difficulty","hard","--label","city-upkeep-rule"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 1000 of 1000 |
| Game length, turns (mean / median / min-max) | 250.1 / 243 / 140-500 |
| Win reason mix | VP hold 96.0%, Conquest 0.1%, Capitals 3.4%, Turn limit 0.5% |
| Median end turn | 243 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 3.4% / 96.0% / 0.5% / 0.1% |
| Games reaching the World War era (any empire) | 997 of 1000 (99.7%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1557 / 490 / 964 / 103 |
| Hold break rate | 31.5% of started; 33.7% of resolved (broken / (broken + completed)) |
| Hold wins | 960 |
| First reach to win, turns (mean / median / min-max) | 13.0 / 9 / 9-198 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 156 |
| Wins by a team another team beat to its target | 85 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 960 | 246.3 / 241 / 140-479 |
| Conquest | 1 | 489.0 / 489 / 489-489 |
| Capitals | 34 | 314.4 / 296 / 238-499 |
| Turn limit | 5 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 112 | 11.2% |
| India | 10 | 1.0% |
| Byzantium | 101 | 10.1% |
| Germany | 166 | 16.6% |
| Celts | 183 | 18.3% |
| Mongols | 82 | 8.2% |
| Sparta | 83 | 8.3% |
| Athens | 143 | 14.3% |
| Aztecs | 120 | 12.0% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 960 | 96.0% |
| Conquest | 1 | 0.1% |
| Capitals | 34 | 3.4% |
| Turn limit | 5 | 0.5% |
| Draw | 0 | 0.0% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 899 | 112 | 12.5% |
| India | 895 | 10 | 1.1% |
| Byzantium | 878 | 101 | 11.5% |
| Germany | 880 | 166 | 18.9% |
| Celts | 896 | 183 | 20.4% |
| Mongols | 877 | 82 | 9.4% |
| Sparta | 891 | 83 | 9.3% |
| Athens | 895 | 143 | 16.0% |
| Aztecs | 889 | 120 | 13.5% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 8000 | 1000 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 899 | 112 | 12.5% |
| India | Hard | 895 | 10 | 1.1% |
| Byzantium | Hard | 878 | 101 | 11.5% |
| Germany | Hard | 880 | 166 | 18.9% |
| Celts | Hard | 896 | 183 | 20.4% |
| Mongols | Hard | 877 | 82 | 9.4% |
| Sparta | Hard | 891 | 83 | 9.3% |
| Athens | Hard | 895 | 143 | 16.0% |
| Aztecs | Hard | 889 | 120 | 13.5% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 899 | 148 | 16.5% | 114 | 147 | 180 | 239 | 371 |
| India | 895 | 484 | 54.1% | 58 | 92 | 129 | 213 | 325 |
| Byzantium | 878 | 672 | 76.5% | 68 | 84 | 110 | 155 | 397 |
| Germany | 880 | 338 | 38.4% | 99 | 133 | 172 | 230 | 364 |
| Celts | 896 | 681 | 76.0% | 49 | 184 | 210 | 210 | 297 |
| Mongols | 877 | 167 | 19.0% | 127 | 151 | 205 | 280 | 428 |
| Sparta | 891 | 153 | 17.2% | 118 | 160 | 209 | 289 | 356 |
| Athens | 895 | 399 | 44.6% | 119 | 167 | 255 | 255 | 255 |
| Aztecs | 889 | 371 | 41.7% | 128 | 178 | 225 | 244 | 285 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 140 | 193 | 243 | 311 | 500 |
| Winning team VP | 2 | 10 | 10 | 13 | 19 |
| First Soldier completed (turn, per empire) | 6 | 8 | 27 | 72 | 315 |
| First Settler completed (turn, per empire) | 3 | 14 | 16 | 25 | 218 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 8 | 18 | 23 | 44 | 251 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 7934 | 66 |
| First Settler completed | 7968 | 32 |
| First city founded | 8000 | 0 |
| Second city founded | 7899 | 101 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 1513 |
| 1 | 1334 |
| 2 | 1134 |
| 3 | 811 |
| 4 | 607 |
| 5 | 498 |
| 6 | 365 |
| 7 | 269 |
| 8 | 204 |
| 9 | 181 |
| 10 | 559 |
| 11 | 258 |
| 12 | 139 |
| 13 | 87 |
| 14 | 27 |
| 15 | 7 |
| 16 | 6 |
| 19 | 1 |

## End state

- Average game length: 250.1 turns
- Average cities per empire: 4.34
- Average techs per empire: 33.61
- Average eliminations per game: 2.27

## Strategy diversity: observed behavior (classifier v2)

1000 finished games; 8000 player seats; 1000 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4372** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 3230 | 176 | 5.4% | 40.4% | 17.6% |
| Wide | 2901 | 720 | 24.8% | 36.3% | 72.0% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 14 | 1 | 7.1% | 0.2% | 0.1% |
| Economic | 827 | 103 | 12.5% | 10.3% | 10.3% |
| Generalist | 1028 | 0 | 0.0% | 12.8% | 0.0% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 20.1% / 3.5% (961) | n/a / n/a (0) | 24.0% / 8.3% (12) | 21.4% / 6.6% (562) | 65.5% / 4.6% (675) |
| Wide | 76.4% / 3.5% (961) | — | n/a / n/a (0) | 63.1% / 10.1% (14) | 55.5% / 5.8% (566) | 89.3% / 2.2% (681) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 67.6% / 8.3% (12) | 26.8% / 10.1% (14) | n/a / n/a (0) | — | 42.9% / 0.0% (7) | 88.9% / 11.1% (9) |
| Economic | 72.0% / 6.6% (562) | 38.6% / 5.8% (566) | n/a / n/a (0) | 57.1% / 0.0% (7) | — | 87.7% / 5.0% (381) |
| Generalist | 29.9% / 4.6% (675) | 8.5% / 2.2% (681) | n/a / n/a (0) | 0.0% / 11.1% (9) | 7.3% / 5.0% (381) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 8 / 189 | 94 / 407 | 0 / 0 | 0 / 1 | 10 / 171 | 0 / 131 |
| India | 5 / 472 | 5 / 341 | 0 / 0 | 0 / 0 | 0 / 15 | 0 / 67 |
| Byzantium | 2 / 14 | 86 / 463 | 0 / 0 | 0 / 5 | 13 / 136 | 0 / 260 |
| Germany | 23 / 344 | 133 / 355 | 0 / 0 | 0 / 1 | 10 / 64 | 0 / 116 |
| Celts | 17 / 240 | 154 / 464 | 0 / 0 | 0 / 0 | 12 / 100 | 0 / 92 |
| Mongols | 46 / 722 | 27 / 88 | 0 / 0 | 0 / 0 | 9 / 19 | 0 / 48 |
| Sparta | 48 / 660 | 24 / 115 | 0 / 0 | 1 / 7 | 10 / 72 | 0 / 37 |
| Athens | 10 / 255 | 110 / 353 | 0 / 0 | 0 / 0 | 23 / 129 | 0 / 158 |
| Aztecs | 17 / 334 | 87 / 315 | 0 / 0 | 0 / 0 | 16 / 121 | 0 / 119 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Generalist. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 1000 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 16879/1000 | 16.879 | 1000/1000 (100.0%) |
| Peace offers | 100741/1000 | 100.741 | 1000/1000 (100.0%) |
| Peace agreements | 14427/1000 | 14.427 | 1000/1000 (100.0%) |
| Battles | 1284860/1000 | 1284.860 | 1000/1000 (100.0%) |
| City captures | 23878/1000 | 23.878 | 1000/1000 (100.0%) |
| Settler captures | 5059/1000 | 5.059 | 983/1000 (98.3%) |
| Eliminations | 2268/1000 | 2.268 | 919/1000 (91.9%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 6894/8000 | 6894/8000 | 3124/1000 |
| Faction goals | 5498/8000 | 5498/8000 | 2202/1000 |
| Era objectives | 15082/8000 | 15082/8000 | 4029/1000 |
| Capitals taken | 2638/8000 | 2638/8000 | 1543/1000 |
| Total VP | 30112/8000 | 30112/8000 | 10898/1000 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 43/363 (11.8%) | 0 |
| Tied leaders (winner in set) | 631/637 (99.1%) | 0 |
| Any leader set | 674/1000 (67.4%) | 0 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 7955/8000 | 5 | 21 | 29 | 33 | 84 | -2.3 (1000) |
| Medieval | 7891/8000 | 31 | 58 | 62 | 66 | 123 | -1.9 (1000) |
| Renaissance | 7825/8000 | 61 | 84 | 89 | 98 | 178 | -3.3 (1000) |
| Industrial | 7588/8000 | 84 | 111 | 119 | 133 | 262 | -5.3 (1000) |
| Machine Age | 6979/8000 | 113 | 141 | 151 | 170 | 299 | -7.0 (1000) |
| World War | 5724/8000 | 136 | 165 | 180 | 204 | 320 | -9.3 (994) |
| Modern | 3940/8000 | 160 | 190 | 211 | 242 | 497 | -10.1 (890) |
| Future | 2194/8000 | 184 | 218 | 247 | 267 | 469 | -8.6 (631) |

Research idle turns (techs left, no research set): 1, of which stalled (two or more in a row): 0.

## Tech pace against the calendar

Median resolved turn each technology was learned, the display year of that turn (`calendar::calendar_label`) and the year the technology really arrived (n/a when no single year is established). Empires that never learned it are left out.

| Era | Technology | Learned / seats | Median turn | Median year | Real year |
| --- | --- | --- | --- | --- | --- |
| Ancient | Irrigation | 8000/8000 | 9 | 3173 BC | 6000 BC |
| Ancient | Bronze Working | 7991/8000 | 18 | 2242 BC | 3300 BC |
| Ancient | Masonry | 8000/8000 | 12 | 2863 BC | 3000 BC |
| Ancient | Currency | 7964/8000 | 27 | 1311 BC | 600 BC |
| Ancient | Writing | 7969/8000 | 25 | 1518 BC | 3200 BC |
| Classical | Mathematics | 7913/8000 | 45 | 251 BC | 300 BC |
| Classical | Engineering | 7907/8000 | 51 | 50 AD | n/a |
| Classical | Iron Working | 7932/8000 | 38 | 601 BC | 1200 BC |
| Classical | Code of Laws | 7898/8000 | 59 | 450 AD | 1754 BC |
| Classical | Philosophy | 7891/8000 | 60 | 500 AD | 500 BC |
| Medieval | Feudalism | 7866/8000 | 75 | 950 AD | n/a |
| Medieval | Machinery | 7881/8000 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 6968/8000 | 80 | 1100 AD | n/a |
| Medieval | Education | 7842/8000 | 86 | 1280 AD | 1088 AD |
| Medieval | Crop Rotation | 7828/8000 | 89 | 1370 AD | n/a |
| Renaissance | Gunpowder | 7796/8000 | 96 | 1460 AD | n/a |
| Renaissance | Metallurgy | 7732/8000 | 107 | 1570 AD | n/a |
| Renaissance | Military Engineering | 7756/8000 | 104 | 1540 AD | n/a |
| Renaissance | Banking | 7644/8000 | 116 | 1660 AD | 1397 AD |
| Renaissance | Printing Press | 7602/8000 | 117 | 1670 AD | 1440 AD |
| Industrial | Rifling | 7208/8000 | 141 | 1809 AD | n/a |
| Industrial | Military Science | 7397/8000 | 133 | 1779 AD | n/a |
| Industrial | Ballistics | 7262/8000 | 143 | 1816 AD | n/a |
| Industrial | Industrialization | 7492/8000 | 128 | 1761 AD | 1769 AD |
| Industrial | Railways | 6985/8000 | 151 | 1846 AD | 1825 AD |
| Machine Age | Replaceable Parts | 6417/8000 | 166 | 1886 AD | 1800 AD |
| Machine Age | Combustion | 6696/8000 | 160 | 1880 AD | 1876 AD |
| Machine Age | Steel | 6380/8000 | 164 | 1884 AD | 1856 AD |
| Machine Age | Flight | 6012/8000 | 174 | 1895 AD | 1903 AD |
| Machine Age | Electricity | 5777/8000 | 180 | 1902 AD | 1879 AD |
| World War | Mass Mobilisation | 4823/8000 | 195 | 1921 AD | n/a |
| World War | Rocketry | 4542/8000 | 200 | 1928 AD | 1926 AD |
| World War | Armoured Warfare | 4334/8000 | 205 | 1935 AD | 1916 AD |
| World War | Air Power | 4161/8000 | 209 | 1941 AD | n/a |
| World War | Electronics | 5348/8000 | 187 | 1910 AD | n/a |
| Modern | Mechanised Warfare | 2930/8000 | 227 | 1978 AD | n/a |
| Modern | Composite Armour | 2661/8000 | 233 | 1993 AD | n/a |
| Modern | Guided Missiles | 2552/8000 | 239 | 2007 AD | n/a |
| Modern | Jet Engines | 2516/8000 | 243 | 2014 AD | 1939 AD |
| Modern | Green Revolution | 3416/8000 | 220 | 1962 AD | n/a |
| Future | Autonomous Systems | 1474/8000 | 256 | 2032 AD | n/a |
| Future | Robotics | 1188/8000 | 262 | 2038 AD | n/a |
| Future | Fusion Power | 1193/8000 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 1737/8000 | 256 | 2032 AD | n/a |
| Future | Recursive AI | 1083/8000 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 68.02 |
| 1 | 79.35 | 14.88 |
| 2 | 25.83 | 5.45 |
| 3 | 6.48 | 1.29 |
| 4 | 1.30 | 0.29 |
| 5 | 0.27 | 0.08 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 20261126 | 208 | 1058 | 9 | 4 | 2 |
| 1 | 20261127 | 216 | 886 | 17 | 0 | 2 |
| 2 | 20261128 | 331 | 1982 | 17 | 1 | 2 |
| 3 | 20261129 | 258 | 1299 | 23 | 7 | 1 |
| 4 | 20261130 | 259 | 1283 | 42 | 7 | 4 |
| 5 | 20261131 | 237 | 1154 | 31 | 10 | 3 |
| 6 | 20261132 | 208 | 1019 | 22 | 2 | 0 |
| 7 | 20261133 | 264 | 1223 | 19 | 4 | 3 |
| 8 | 20261134 | 188 | 777 | 17 | 3 | 1 |
| 9 | 20261135 | 185 | 743 | 11 | 6 | 1 |
| 10 | 20261136 | 251 | 1081 | 30 | 3 | 3 |
| 11 | 20261137 | 340 | 1577 | 19 | 11 | 2 |
| 12 | 20261138 | 274 | 1504 | 30 | 5 | 4 |
| 13 | 20261139 | 226 | 988 | 14 | 8 | 1 |
| 14 | 20261140 | 399 | 1849 | 32 | 6 | 3 |
| 15 | 20261141 | 269 | 1489 | 28 | 6 | 4 |
| 16 | 20261142 | 266 | 1231 | 21 | 4 | 2 |
| 17 | 20261143 | 191 | 1001 | 15 | 6 | 1 |
| 18 | 20261144 | 258 | 1270 | 32 | 3 | 2 |
| 19 | 20261145 | 211 | 959 | 13 | 11 | 1 |
| 20 | 20261146 | 239 | 1342 | 32 | 5 | 2 |
| 21 | 20261147 | 457 | 2358 | 46 | 15 | 3 |
| 22 | 20261148 | 201 | 993 | 22 | 3 | 3 |
| 23 | 20261149 | 215 | 1240 | 10 | 3 | 0 |
| 24 | 20261150 | 291 | 1632 | 23 | 4 | 0 |
| 25 | 20261151 | 196 | 852 | 14 | 5 | 2 |
| 26 | 20261152 | 299 | 1692 | 37 | 5 | 5 |
| 27 | 20261153 | 256 | 1232 | 23 | 8 | 1 |
| 28 | 20261154 | 237 | 1212 | 28 | 9 | 1 |
| 29 | 20261155 | 252 | 1315 | 22 | 6 | 3 |
| 30 | 20261156 | 489 | 3074 | 50 | 8 | 7 |
| 31 | 20261157 | 210 | 1046 | 20 | 4 | 1 |
| 32 | 20261158 | 216 | 1080 | 22 | 12 | 2 |
| 33 | 20261159 | 255 | 1483 | 31 | 1 | 2 |
| 34 | 20261160 | 257 | 1449 | 30 | 4 | 3 |
| 35 | 20261161 | 263 | 1322 | 19 | 2 | 3 |
| 36 | 20261162 | 214 | 927 | 21 | 0 | 2 |
| 37 | 20261163 | 149 | 537 | 7 | 8 | 1 |
| 38 | 20261164 | 232 | 1060 | 11 | 5 | 1 |
| 39 | 20261165 | 281 | 1615 | 31 | 2 | 0 |
| 40 | 20261166 | 235 | 1330 | 27 | 9 | 3 |
| 41 | 20261167 | 191 | 954 | 14 | 3 | 2 |
| 42 | 20261168 | 232 | 1204 | 24 | 3 | 2 |
| 43 | 20261169 | 274 | 1395 | 17 | 1 | 1 |
| 44 | 20261170 | 500 | 2223 | 27 | 6 | 5 |
| 45 | 20261171 | 260 | 1293 | 21 | 4 | 4 |
| 46 | 20261172 | 201 | 840 | 11 | 6 | 2 |
| 47 | 20261173 | 320 | 1791 | 32 | 4 | 1 |
| 48 | 20261174 | 211 | 894 | 20 | 0 | 2 |
| 49 | 20261175 | 274 | 1272 | 13 | 7 | 1 |
| 50 | 20261176 | 193 | 976 | 14 | 7 | 1 |
| 51 | 20261177 | 268 | 1310 | 27 | 5 | 3 |
| 52 | 20261178 | 192 | 866 | 12 | 5 | 3 |
| 53 | 20261179 | 269 | 1453 | 34 | 6 | 3 |
| 54 | 20261180 | 277 | 1529 | 23 | 9 | 3 |
| 55 | 20261181 | 258 | 1331 | 39 | 1 | 5 |
| 56 | 20261182 | 220 | 872 | 23 | 5 | 1 |
| 57 | 20261183 | 184 | 790 | 17 | 2 | 1 |
| 58 | 20261184 | 293 | 1412 | 44 | 5 | 5 |
| 59 | 20261185 | 180 | 763 | 13 | 3 | 1 |
| 60 | 20261186 | 289 | 1300 | 31 | 1 | 3 |
| 61 | 20261187 | 263 | 1090 | 20 | 5 | 2 |
| 62 | 20261188 | 252 | 1395 | 30 | 4 | 2 |
| 63 | 20261189 | 500 | 3158 | 36 | 7 | 1 |
| 64 | 20261190 | 227 | 1112 | 21 | 5 | 2 |
| 65 | 20261191 | 294 | 1624 | 26 | 18 | 2 |
| 66 | 20261192 | 260 | 1373 | 29 | 1 | 2 |
| 67 | 20261193 | 287 | 1556 | 28 | 9 | 3 |
| 68 | 20261194 | 251 | 1336 | 26 | 6 | 2 |
| 69 | 20261195 | 219 | 1082 | 12 | 7 | 3 |
| 70 | 20261196 | 305 | 1628 | 33 | 7 | 3 |
| 71 | 20261197 | 243 | 956 | 38 | 3 | 1 |
| 72 | 20261198 | 211 | 1164 | 25 | 1 | 2 |
| 73 | 20261199 | 200 | 1152 | 21 | 7 | 0 |
| 74 | 20261200 | 242 | 1391 | 17 | 3 | 2 |
| 75 | 20261201 | 206 | 958 | 27 | 6 | 2 |
| 76 | 20261202 | 285 | 1628 | 32 | 8 | 1 |
| 77 | 20261203 | 277 | 1408 | 13 | 12 | 2 |
| 78 | 20261204 | 328 | 1513 | 24 | 6 | 6 |
| 79 | 20261205 | 224 | 1139 | 28 | 0 | 1 |
| 80 | 20261206 | 253 | 1132 | 25 | 9 | 3 |
| 81 | 20261207 | 176 | 883 | 10 | 4 | 1 |
| 82 | 20261208 | 219 | 1360 | 17 | 5 | 0 |
| 83 | 20261209 | 312 | 1900 | 34 | 6 | 5 |
| 84 | 20261210 | 266 | 1292 | 26 | 7 | 2 |
| 85 | 20261211 | 180 | 852 | 21 | 3 | 2 |
| 86 | 20261212 | 253 | 1339 | 40 | 2 | 3 |
| 87 | 20261213 | 195 | 760 | 16 | 6 | 3 |
| 88 | 20261214 | 238 | 736 | 5 | 10 | 2 |
| 89 | 20261215 | 236 | 1156 | 27 | 6 | 1 |
| 90 | 20261216 | 258 | 1106 | 34 | 2 | 5 |
| 91 | 20261217 | 241 | 1117 | 18 | 1 | 2 |
| 92 | 20261218 | 240 | 1012 | 26 | 3 | 2 |
| 93 | 20261219 | 264 | 1442 | 28 | 7 | 2 |
| 94 | 20261220 | 195 | 896 | 17 | 2 | 1 |
| 95 | 20261221 | 155 | 589 | 6 | 6 | 0 |
| 96 | 20261222 | 250 | 1348 | 35 | 9 | 2 |
| 97 | 20261223 | 235 | 1251 | 26 | 6 | 1 |
| 98 | 20261224 | 319 | 1933 | 30 | 4 | 3 |
| 99 | 20261225 | 294 | 2091 | 25 | 2 | 2 |
| 100 | 20261226 | 209 | 1088 | 17 | 3 | 2 |
| 101 | 20261227 | 284 | 1255 | 28 | 6 | 3 |
| 102 | 20261228 | 331 | 1452 | 26 | 21 | 2 |
| 103 | 20261229 | 283 | 1420 | 28 | 8 | 2 |
| 104 | 20261230 | 306 | 1569 | 24 | 7 | 3 |
| 105 | 20261231 | 247 | 1045 | 15 | 6 | 3 |
| 106 | 20261232 | 364 | 2057 | 42 | 2 | 3 |
| 107 | 20261233 | 406 | 2393 | 62 | 13 | 2 |
| 108 | 20261234 | 169 | 754 | 10 | 4 | 0 |
| 109 | 20261235 | 202 | 653 | 7 | 9 | 0 |
| 110 | 20261236 | 229 | 1182 | 26 | 4 | 2 |
| 111 | 20261237 | 184 | 706 | 19 | 3 | 3 |
| 112 | 20261238 | 242 | 1017 | 15 | 5 | 1 |
| 113 | 20261239 | 218 | 1222 | 15 | 2 | 1 |
| 114 | 20261240 | 355 | 1723 | 34 | 3 | 3 |
| 115 | 20261241 | 266 | 1084 | 17 | 8 | 4 |
| 116 | 20261242 | 223 | 1144 | 29 | 3 | 1 |
| 117 | 20261243 | 248 | 1483 | 33 | 2 | 0 |
| 118 | 20261244 | 212 | 1119 | 25 | 4 | 1 |
| 119 | 20261245 | 278 | 1623 | 28 | 4 | 1 |
| 120 | 20261246 | 230 | 1343 | 14 | 4 | 2 |
| 121 | 20261247 | 190 | 907 | 17 | 0 | 0 |
| 122 | 20261248 | 230 | 1256 | 28 | 2 | 3 |
| 123 | 20261249 | 235 | 1070 | 23 | 3 | 2 |
| 124 | 20261250 | 225 | 1036 | 25 | 5 | 3 |
| 125 | 20261251 | 208 | 1050 | 16 | 7 | 2 |
| 126 | 20261252 | 185 | 722 | 18 | 5 | 2 |
| 127 | 20261253 | 279 | 1650 | 38 | 7 | 3 |
| 128 | 20261254 | 277 | 1481 | 15 | 7 | 3 |
| 129 | 20261255 | 297 | 1211 | 28 | 10 | 3 |
| 130 | 20261256 | 211 | 1023 | 30 | 3 | 2 |
| 131 | 20261257 | 261 | 1343 | 18 | 0 | 1 |
| 132 | 20261258 | 227 | 1056 | 13 | 5 | 3 |
| 133 | 20261259 | 232 | 1093 | 20 | 7 | 3 |
| 134 | 20261260 | 233 | 1226 | 28 | 4 | 2 |
| 135 | 20261261 | 301 | 1510 | 20 | 15 | 4 |
| 136 | 20261262 | 217 | 973 | 26 | 7 | 1 |
| 137 | 20261263 | 249 | 1295 | 22 | 3 | 3 |
| 138 | 20261264 | 253 | 1223 | 22 | 4 | 1 |
| 139 | 20261265 | 320 | 1460 | 19 | 7 | 3 |
| 140 | 20261266 | 206 | 940 | 21 | 2 | 2 |
| 141 | 20261267 | 216 | 1203 | 14 | 9 | 1 |
| 142 | 20261268 | 249 | 1154 | 20 | 10 | 4 |
| 143 | 20261269 | 337 | 2074 | 35 | 10 | 4 |
| 144 | 20261270 | 241 | 1113 | 26 | 5 | 3 |
| 145 | 20261271 | 190 | 764 | 13 | 3 | 0 |
| 146 | 20261272 | 238 | 1086 | 28 | 3 | 1 |
| 147 | 20261273 | 257 | 1360 | 29 | 7 | 3 |
| 148 | 20261274 | 276 | 1596 | 18 | 3 | 2 |
| 149 | 20261275 | 213 | 1178 | 14 | 5 | 2 |
| 150 | 20261276 | 281 | 1512 | 24 | 9 | 4 |
| 151 | 20261277 | 271 | 1562 | 20 | 5 | 5 |
| 152 | 20261278 | 278 | 1361 | 28 | 3 | 0 |
| 153 | 20261279 | 183 | 1007 | 16 | 7 | 0 |
| 154 | 20261280 | 270 | 1403 | 34 | 1 | 4 |
| 155 | 20261281 | 223 | 1378 | 25 | 1 | 2 |
| 156 | 20261282 | 203 | 1225 | 24 | 4 | 2 |
| 157 | 20261283 | 212 | 1038 | 18 | 2 | 1 |
| 158 | 20261284 | 341 | 1985 | 28 | 9 | 2 |
| 159 | 20261285 | 279 | 1793 | 32 | 14 | 2 |
| 160 | 20261286 | 339 | 2002 | 38 | 8 | 2 |
| 161 | 20261287 | 253 | 1391 | 24 | 2 | 1 |
| 162 | 20261288 | 273 | 1803 | 35 | 6 | 5 |
| 163 | 20261289 | 322 | 1656 | 35 | 2 | 4 |
| 164 | 20261290 | 331 | 1721 | 39 | 6 | 5 |
| 165 | 20261291 | 244 | 1356 | 24 | 4 | 4 |
| 166 | 20261292 | 235 | 1127 | 24 | 12 | 4 |
| 167 | 20261293 | 264 | 1692 | 40 | 12 | 3 |
| 168 | 20261294 | 243 | 1113 | 20 | 2 | 3 |
| 169 | 20261295 | 321 | 1694 | 31 | 3 | 4 |
| 170 | 20261296 | 303 | 1725 | 40 | 2 | 4 |
| 171 | 20261297 | 213 | 952 | 11 | 9 | 2 |
| 172 | 20261298 | 224 | 1189 | 25 | 4 | 3 |
| 173 | 20261299 | 255 | 1453 | 33 | 0 | 2 |
| 174 | 20261300 | 220 | 1189 | 19 | 1 | 1 |
| 175 | 20261301 | 297 | 1453 | 25 | 4 | 2 |
| 176 | 20261302 | 349 | 1893 | 27 | 4 | 4 |
| 177 | 20261303 | 214 | 1129 | 20 | 12 | 4 |
| 178 | 20261304 | 284 | 1257 | 12 | 6 | 2 |
| 179 | 20261305 | 279 | 1678 | 26 | 2 | 1 |
| 180 | 20261306 | 206 | 1080 | 19 | 4 | 1 |
| 181 | 20261307 | 265 | 1516 | 31 | 5 | 1 |
| 182 | 20261308 | 240 | 1386 | 37 | 2 | 2 |
| 183 | 20261309 | 246 | 1177 | 25 | 3 | 2 |
| 184 | 20261310 | 382 | 2448 | 43 | 2 | 3 |
| 185 | 20261311 | 176 | 695 | 12 | 10 | 4 |
| 186 | 20261312 | 260 | 1488 | 17 | 6 | 5 |
| 187 | 20261313 | 246 | 1076 | 14 | 13 | 0 |
| 188 | 20261314 | 229 | 1001 | 16 | 8 | 3 |
| 189 | 20261315 | 298 | 1374 | 30 | 6 | 2 |
| 190 | 20261316 | 283 | 1575 | 29 | 5 | 2 |
| 191 | 20261317 | 219 | 1040 | 32 | 5 | 4 |
| 192 | 20261318 | 228 | 1239 | 28 | 4 | 3 |
| 193 | 20261319 | 188 | 930 | 15 | 2 | 2 |
| 194 | 20261320 | 230 | 1140 | 16 | 8 | 2 |
| 195 | 20261321 | 188 | 716 | 12 | 1 | 1 |
| 196 | 20261322 | 280 | 1115 | 20 | 2 | 4 |
| 197 | 20261323 | 184 | 915 | 13 | 2 | 1 |
| 198 | 20261324 | 288 | 1756 | 35 | 4 | 0 |
| 199 | 20261325 | 267 | 1649 | 27 | 4 | 1 |
| 200 | 20261326 | 287 | 1649 | 39 | 9 | 3 |
| 201 | 20261327 | 161 | 692 | 10 | 0 | 1 |
| 202 | 20261328 | 276 | 1229 | 32 | 4 | 3 |
| 203 | 20261329 | 223 | 1184 | 10 | 9 | 3 |
| 204 | 20261330 | 201 | 997 | 22 | 3 | 3 |
| 205 | 20261331 | 274 | 1524 | 22 | 3 | 4 |
| 206 | 20261332 | 273 | 1522 | 32 | 6 | 2 |
| 207 | 20261333 | 229 | 1357 | 18 | 2 | 2 |
| 208 | 20261334 | 277 | 1558 | 37 | 5 | 5 |
| 209 | 20261335 | 184 | 896 | 17 | 2 | 1 |
| 210 | 20261336 | 174 | 833 | 17 | 1 | 3 |
| 211 | 20261337 | 230 | 1120 | 22 | 5 | 1 |
| 212 | 20261338 | 223 | 1378 | 18 | 14 | 1 |
| 213 | 20261339 | 263 | 1615 | 25 | 8 | 2 |
| 214 | 20261340 | 208 | 993 | 7 | 3 | 2 |
| 215 | 20261341 | 235 | 1276 | 16 | 4 | 2 |
| 216 | 20261342 | 226 | 986 | 20 | 3 | 3 |
| 217 | 20261343 | 204 | 856 | 16 | 5 | 2 |
| 218 | 20261344 | 207 | 985 | 23 | 5 | 1 |
| 219 | 20261345 | 256 | 1068 | 13 | 7 | 2 |
| 220 | 20261346 | 291 | 1657 | 21 | 5 | 1 |
| 221 | 20261347 | 252 | 1481 | 14 | 3 | 2 |
| 222 | 20261348 | 184 | 898 | 9 | 4 | 3 |
| 223 | 20261349 | 238 | 1330 | 18 | 2 | 5 |
| 224 | 20261350 | 255 | 994 | 29 | 5 | 6 |
| 225 | 20261351 | 205 | 1013 | 17 | 4 | 0 |
| 226 | 20261352 | 190 | 904 | 19 | 4 | 3 |
| 227 | 20261353 | 222 | 1187 | 21 | 8 | 1 |
| 228 | 20261354 | 155 | 462 | 12 | 2 | 2 |
| 229 | 20261355 | 273 | 1654 | 17 | 2 | 2 |
| 230 | 20261356 | 219 | 1172 | 13 | 6 | 2 |
| 231 | 20261357 | 179 | 909 | 18 | 5 | 0 |
| 232 | 20261358 | 167 | 768 | 9 | 4 | 2 |
| 233 | 20261359 | 266 | 1538 | 29 | 11 | 3 |
| 234 | 20261360 | 276 | 1245 | 36 | 3 | 4 |
| 235 | 20261361 | 268 | 1284 | 33 | 10 | 3 |
| 236 | 20261362 | 372 | 2264 | 49 | 1 | 3 |
| 237 | 20261363 | 217 | 1213 | 24 | 3 | 3 |
| 238 | 20261364 | 188 | 768 | 22 | 2 | 3 |
| 239 | 20261365 | 258 | 1544 | 23 | 5 | 2 |
| 240 | 20261366 | 268 | 1618 | 28 | 4 | 2 |
| 241 | 20261367 | 223 | 1095 | 20 | 3 | 4 |
| 242 | 20261368 | 235 | 1352 | 29 | 4 | 0 |
| 243 | 20261369 | 226 | 1244 | 24 | 5 | 2 |
| 244 | 20261370 | 199 | 944 | 22 | 4 | 2 |
| 245 | 20261371 | 261 | 1385 | 26 | 3 | 1 |
| 246 | 20261372 | 284 | 1388 | 14 | 10 | 2 |
| 247 | 20261373 | 286 | 1374 | 19 | 8 | 3 |
| 248 | 20261374 | 408 | 2644 | 31 | 5 | 0 |
| 249 | 20261375 | 213 | 1294 | 14 | 3 | 0 |
| 250 | 20261376 | 198 | 754 | 12 | 6 | 2 |
| 251 | 20261377 | 256 | 1383 | 18 | 5 | 3 |
| 252 | 20261378 | 223 | 1098 | 25 | 3 | 3 |
| 253 | 20261379 | 248 | 1321 | 23 | 6 | 4 |
| 254 | 20261380 | 233 | 1091 | 15 | 5 | 2 |
| 255 | 20261381 | 238 | 908 | 22 | 2 | 1 |
| 256 | 20261382 | 256 | 1081 | 12 | 6 | 1 |
| 257 | 20261383 | 191 | 892 | 17 | 0 | 2 |
| 258 | 20261384 | 217 | 970 | 21 | 2 | 2 |
| 259 | 20261385 | 326 | 1789 | 39 | 4 | 2 |
| 260 | 20261386 | 247 | 1178 | 20 | 9 | 1 |
| 261 | 20261387 | 259 | 1145 | 30 | 5 | 2 |
| 262 | 20261388 | 204 | 1011 | 17 | 7 | 1 |
| 263 | 20261389 | 332 | 2221 | 46 | 3 | 3 |
| 264 | 20261390 | 213 | 856 | 22 | 8 | 2 |
| 265 | 20261391 | 234 | 1060 | 25 | 3 | 1 |
| 266 | 20261392 | 182 | 689 | 11 | 2 | 1 |
| 267 | 20261393 | 259 | 1561 | 35 | 9 | 1 |
| 268 | 20261394 | 246 | 1191 | 28 | 4 | 4 |
| 269 | 20261395 | 200 | 966 | 17 | 6 | 1 |
| 270 | 20261396 | 261 | 1360 | 28 | 4 | 4 |
| 271 | 20261397 | 194 | 764 | 8 | 6 | 1 |
| 272 | 20261398 | 228 | 1063 | 26 | 4 | 4 |
| 273 | 20261399 | 200 | 983 | 24 | 10 | 3 |
| 274 | 20261400 | 187 | 616 | 17 | 4 | 4 |
| 275 | 20261401 | 179 | 746 | 17 | 6 | 2 |
| 276 | 20261402 | 159 | 633 | 9 | 5 | 0 |
| 277 | 20261403 | 214 | 996 | 20 | 3 | 3 |
| 278 | 20261404 | 219 | 1300 | 26 | 4 | 2 |
| 279 | 20261405 | 266 | 1356 | 26 | 3 | 1 |
| 280 | 20261406 | 229 | 1208 | 30 | 6 | 1 |
| 281 | 20261407 | 207 | 1108 | 20 | 2 | 3 |
| 282 | 20261408 | 368 | 2401 | 29 | 14 | 3 |
| 283 | 20261409 | 231 | 1069 | 19 | 8 | 3 |
| 284 | 20261410 | 260 | 1622 | 35 | 3 | 2 |
| 285 | 20261411 | 241 | 1304 | 33 | 1 | 5 |
| 286 | 20261412 | 268 | 1126 | 20 | 4 | 1 |
| 287 | 20261413 | 212 | 1021 | 20 | 4 | 2 |
| 288 | 20261414 | 256 | 1226 | 23 | 4 | 1 |
| 289 | 20261415 | 206 | 1164 | 23 | 3 | 0 |
| 290 | 20261416 | 248 | 1324 | 18 | 9 | 4 |
| 291 | 20261417 | 224 | 1038 | 22 | 4 | 2 |
| 292 | 20261418 | 277 | 1532 | 41 | 8 | 3 |
| 293 | 20261419 | 183 | 768 | 16 | 3 | 0 |
| 294 | 20261420 | 308 | 1319 | 31 | 2 | 3 |
| 295 | 20261421 | 168 | 868 | 14 | 11 | 0 |
| 296 | 20261422 | 208 | 1054 | 13 | 12 | 2 |
| 297 | 20261423 | 332 | 1975 | 23 | 5 | 2 |
| 298 | 20261424 | 255 | 1282 | 29 | 8 | 5 |
| 299 | 20261425 | 270 | 1485 | 42 | 5 | 4 |
| 300 | 20261426 | 431 | 2400 | 33 | 4 | 2 |
| 301 | 20261427 | 249 | 1368 | 33 | 1 | 1 |
| 302 | 20261428 | 267 | 1818 | 29 | 3 | 3 |
| 303 | 20261429 | 290 | 1509 | 28 | 2 | 1 |
| 304 | 20261430 | 256 | 1696 | 22 | 6 | 2 |
| 305 | 20261431 | 231 | 890 | 21 | 4 | 2 |
| 306 | 20261432 | 278 | 1311 | 34 | 7 | 2 |
| 307 | 20261433 | 298 | 1715 | 34 | 7 | 3 |
| 308 | 20261434 | 260 | 1205 | 21 | 7 | 2 |
| 309 | 20261435 | 241 | 1012 | 19 | 6 | 4 |
| 310 | 20261436 | 224 | 1410 | 20 | 2 | 4 |
| 311 | 20261437 | 257 | 1165 | 28 | 7 | 2 |
| 312 | 20261438 | 168 | 815 | 14 | 8 | 2 |
| 313 | 20261439 | 199 | 967 | 15 | 1 | 1 |
| 314 | 20261440 | 199 | 823 | 16 | 4 | 1 |
| 315 | 20261441 | 240 | 1447 | 18 | 5 | 3 |
| 316 | 20261442 | 195 | 797 | 20 | 6 | 2 |
| 317 | 20261443 | 235 | 1243 | 13 | 3 | 0 |
| 318 | 20261444 | 278 | 1670 | 23 | 4 | 0 |
| 319 | 20261445 | 236 | 837 | 19 | 16 | 2 |
| 320 | 20261446 | 265 | 1587 | 23 | 5 | 3 |
| 321 | 20261447 | 227 | 1335 | 18 | 5 | 0 |
| 322 | 20261448 | 197 | 949 | 25 | 6 | 1 |
| 323 | 20261449 | 200 | 1124 | 16 | 4 | 1 |
| 324 | 20261450 | 297 | 1525 | 35 | 5 | 4 |
| 325 | 20261451 | 222 | 1019 | 16 | 3 | 1 |
| 326 | 20261452 | 250 | 1267 | 18 | 8 | 5 |
| 327 | 20261453 | 249 | 1300 | 29 | 4 | 2 |
| 328 | 20261454 | 195 | 1023 | 22 | 1 | 2 |
| 329 | 20261455 | 342 | 2091 | 28 | 7 | 4 |
| 330 | 20261456 | 229 | 1262 | 35 | 5 | 3 |
| 331 | 20261457 | 262 | 1191 | 22 | 4 | 4 |
| 332 | 20261458 | 500 | 2481 | 31 | 3 | 4 |
| 333 | 20261459 | 246 | 1581 | 17 | 10 | 1 |
| 334 | 20261460 | 266 | 1523 | 22 | 5 | 4 |
| 335 | 20261461 | 238 | 1265 | 17 | 6 | 2 |
| 336 | 20261462 | 244 | 1230 | 28 | 6 | 3 |
| 337 | 20261463 | 208 | 1191 | 15 | 2 | 1 |
| 338 | 20261464 | 217 | 1117 | 16 | 1 | 0 |
| 339 | 20261465 | 249 | 1356 | 27 | 9 | 3 |
| 340 | 20261466 | 255 | 1249 | 32 | 8 | 2 |
| 341 | 20261467 | 257 | 1527 | 40 | 5 | 3 |
| 342 | 20261468 | 218 | 847 | 15 | 8 | 1 |
| 343 | 20261469 | 330 | 1993 | 23 | 4 | 2 |
| 344 | 20261470 | 228 | 999 | 22 | 1 | 3 |
| 345 | 20261471 | 181 | 844 | 11 | 9 | 1 |
| 346 | 20261472 | 253 | 1147 | 19 | 5 | 2 |
| 347 | 20261473 | 283 | 1480 | 28 | 3 | 3 |
| 348 | 20261474 | 381 | 2323 | 40 | 3 | 1 |
| 349 | 20261475 | 479 | 3459 | 34 | 6 | 2 |
| 350 | 20261476 | 214 | 924 | 11 | 13 | 0 |
| 351 | 20261477 | 234 | 1282 | 25 | 2 | 2 |
| 352 | 20261478 | 255 | 1500 | 22 | 10 | 2 |
| 353 | 20261479 | 233 | 994 | 21 | 7 | 0 |
| 354 | 20261480 | 306 | 1601 | 28 | 4 | 4 |
| 355 | 20261481 | 238 | 1172 | 32 | 8 | 3 |
| 356 | 20261482 | 232 | 1223 | 16 | 5 | 3 |
| 357 | 20261483 | 223 | 1000 | 19 | 4 | 0 |
| 358 | 20261484 | 193 | 794 | 15 | 5 | 1 |
| 359 | 20261485 | 248 | 1230 | 19 | 3 | 2 |
| 360 | 20261486 | 226 | 779 | 20 | 2 | 3 |
| 361 | 20261487 | 247 | 1064 | 23 | 4 | 2 |
| 362 | 20261488 | 235 | 1303 | 30 | 4 | 1 |
| 363 | 20261489 | 239 | 1121 | 25 | 2 | 1 |
| 364 | 20261490 | 245 | 1018 | 17 | 11 | 1 |
| 365 | 20261491 | 288 | 1618 | 35 | 1 | 3 |
| 366 | 20261492 | 225 | 933 | 22 | 7 | 2 |
| 367 | 20261493 | 236 | 857 | 22 | 7 | 1 |
| 368 | 20261494 | 244 | 1273 | 27 | 1 | 3 |
| 369 | 20261495 | 266 | 1252 | 21 | 11 | 2 |
| 370 | 20261496 | 279 | 1327 | 28 | 2 | 3 |
| 371 | 20261497 | 238 | 1287 | 19 | 6 | 5 |
| 372 | 20261498 | 236 | 1082 | 19 | 3 | 1 |
| 373 | 20261499 | 266 | 1453 | 41 | 5 | 3 |
| 374 | 20261500 | 314 | 1654 | 27 | 8 | 3 |
| 375 | 20261501 | 250 | 1442 | 29 | 1 | 3 |
| 376 | 20261502 | 275 | 1410 | 20 | 2 | 2 |
| 377 | 20261503 | 252 | 1313 | 24 | 6 | 2 |
| 378 | 20261504 | 252 | 1300 | 35 | 12 | 3 |
| 379 | 20261505 | 196 | 921 | 18 | 6 | 1 |
| 380 | 20261506 | 263 | 1514 | 40 | 12 | 2 |
| 381 | 20261507 | 201 | 960 | 17 | 3 | 1 |
| 382 | 20261508 | 265 | 1453 | 27 | 4 | 3 |
| 383 | 20261509 | 221 | 1172 | 18 | 3 | 1 |
| 384 | 20261510 | 268 | 1418 | 17 | 8 | 2 |
| 385 | 20261511 | 190 | 853 | 23 | 4 | 3 |
| 386 | 20261512 | 179 | 630 | 11 | 6 | 1 |
| 387 | 20261513 | 229 | 1199 | 27 | 1 | 2 |
| 388 | 20261514 | 212 | 976 | 10 | 4 | 4 |
| 389 | 20261515 | 213 | 1067 | 15 | 3 | 3 |
| 390 | 20261516 | 264 | 1720 | 18 | 7 | 2 |
| 391 | 20261517 | 259 | 1123 | 24 | 3 | 3 |
| 392 | 20261518 | 316 | 1621 | 33 | 9 | 5 |
| 393 | 20261519 | 337 | 1661 | 31 | 5 | 5 |
| 394 | 20261520 | 237 | 1247 | 28 | 6 | 3 |
| 395 | 20261521 | 230 | 1143 | 29 | 5 | 3 |
| 396 | 20261522 | 288 | 1847 | 29 | 9 | 2 |
| 397 | 20261523 | 205 | 895 | 20 | 5 | 0 |
| 398 | 20261524 | 198 | 655 | 24 | 8 | 3 |
| 399 | 20261525 | 213 | 1108 | 28 | 2 | 1 |
| 400 | 20261526 | 231 | 1015 | 12 | 3 | 3 |
| 401 | 20261527 | 194 | 1118 | 22 | 0 | 3 |
| 402 | 20261528 | 300 | 1475 | 22 | 7 | 4 |
| 403 | 20261529 | 217 | 1115 | 23 | 5 | 2 |
| 404 | 20261530 | 260 | 1326 | 19 | 8 | 3 |
| 405 | 20261531 | 303 | 1574 | 25 | 9 | 4 |
| 406 | 20261532 | 211 | 908 | 17 | 4 | 1 |
| 407 | 20261533 | 263 | 1129 | 17 | 4 | 3 |
| 408 | 20261534 | 237 | 1276 | 29 | 10 | 4 |
| 409 | 20261535 | 248 | 1546 | 24 | 9 | 0 |
| 410 | 20261536 | 205 | 961 | 23 | 3 | 2 |
| 411 | 20261537 | 195 | 713 | 17 | 4 | 0 |
| 412 | 20261538 | 199 | 1081 | 18 | 6 | 0 |
| 413 | 20261539 | 224 | 1137 | 15 | 6 | 1 |
| 414 | 20261540 | 246 | 1557 | 19 | 4 | 4 |
| 415 | 20261541 | 348 | 2134 | 37 | 2 | 3 |
| 416 | 20261542 | 253 | 1349 | 30 | 2 | 3 |
| 417 | 20261543 | 245 | 991 | 21 | 1 | 2 |
| 418 | 20261544 | 183 | 1011 | 14 | 9 | 0 |
| 419 | 20261545 | 226 | 1227 | 18 | 2 | 3 |
| 420 | 20261546 | 244 | 1398 | 19 | 7 | 3 |
| 421 | 20261547 | 227 | 1328 | 25 | 2 | 4 |
| 422 | 20261548 | 172 | 706 | 17 | 2 | 1 |
| 423 | 20261549 | 245 | 1494 | 23 | 1 | 1 |
| 424 | 20261550 | 290 | 1495 | 23 | 6 | 2 |
| 425 | 20261551 | 263 | 1465 | 21 | 4 | 2 |
| 426 | 20261552 | 363 | 2180 | 39 | 4 | 2 |
| 427 | 20261553 | 235 | 1315 | 31 | 2 | 3 |
| 428 | 20261554 | 243 | 1240 | 24 | 5 | 2 |
| 429 | 20261555 | 249 | 1049 | 25 | 3 | 3 |
| 430 | 20261556 | 212 | 767 | 26 | 7 | 3 |
| 431 | 20261557 | 243 | 1280 | 23 | 7 | 1 |
| 432 | 20261558 | 269 | 1190 | 31 | 9 | 1 |
| 433 | 20261559 | 233 | 891 | 27 | 8 | 4 |
| 434 | 20261560 | 190 | 965 | 12 | 5 | 0 |
| 435 | 20261561 | 249 | 1078 | 24 | 6 | 3 |
| 436 | 20261562 | 313 | 1641 | 30 | 4 | 4 |
| 437 | 20261563 | 228 | 1030 | 26 | 5 | 2 |
| 438 | 20261564 | 257 | 1280 | 13 | 7 | 2 |
| 439 | 20261565 | 236 | 1178 | 15 | 6 | 2 |
| 440 | 20261566 | 244 | 1286 | 16 | 5 | 3 |
| 441 | 20261567 | 248 | 1193 | 21 | 4 | 3 |
| 442 | 20261568 | 252 | 1413 | 11 | 10 | 1 |
| 443 | 20261569 | 202 | 858 | 16 | 5 | 3 |
| 444 | 20261570 | 221 | 1266 | 13 | 7 | 1 |
| 445 | 20261571 | 311 | 1909 | 27 | 4 | 3 |
| 446 | 20261572 | 286 | 1588 | 33 | 4 | 3 |
| 447 | 20261573 | 263 | 1031 | 30 | 5 | 4 |
| 448 | 20261574 | 320 | 1754 | 30 | 5 | 1 |
| 449 | 20261575 | 306 | 1806 | 32 | 0 | 2 |
| 450 | 20261576 | 294 | 1271 | 29 | 5 | 5 |
| 451 | 20261577 | 269 | 1503 | 25 | 1 | 3 |
| 452 | 20261578 | 207 | 1076 | 11 | 3 | 1 |
| 453 | 20261579 | 279 | 1205 | 11 | 6 | 2 |
| 454 | 20261580 | 232 | 1223 | 19 | 4 | 3 |
| 455 | 20261581 | 330 | 1972 | 22 | 3 | 1 |
| 456 | 20261582 | 340 | 1937 | 29 | 3 | 3 |
| 457 | 20261583 | 265 | 1415 | 13 | 9 | 1 |
| 458 | 20261584 | 303 | 1719 | 42 | 5 | 3 |
| 459 | 20261585 | 223 | 1205 | 24 | 5 | 1 |
| 460 | 20261586 | 236 | 1260 | 18 | 6 | 3 |
| 461 | 20261587 | 213 | 1044 | 18 | 4 | 2 |
| 462 | 20261588 | 255 | 1344 | 14 | 2 | 2 |
| 463 | 20261589 | 194 | 1061 | 22 | 3 | 1 |
| 464 | 20261590 | 232 | 1053 | 17 | 6 | 3 |
| 465 | 20261591 | 209 | 835 | 24 | 1 | 0 |
| 466 | 20261592 | 243 | 1146 | 21 | 1 | 3 |
| 467 | 20261593 | 269 | 1689 | 32 | 3 | 1 |
| 468 | 20261594 | 199 | 1007 | 19 | 7 | 1 |
| 469 | 20261595 | 303 | 1811 | 35 | 8 | 2 |
| 470 | 20261596 | 291 | 1669 | 30 | 5 | 3 |
| 471 | 20261597 | 229 | 1049 | 15 | 8 | 2 |
| 472 | 20261598 | 313 | 1727 | 26 | 8 | 4 |
| 473 | 20261599 | 257 | 1413 | 27 | 9 | 4 |
| 474 | 20261600 | 218 | 1053 | 24 | 5 | 2 |
| 475 | 20261601 | 293 | 1440 | 26 | 2 | 3 |
| 476 | 20261602 | 226 | 1113 | 30 | 7 | 3 |
| 477 | 20261603 | 275 | 1217 | 33 | 6 | 4 |
| 478 | 20261604 | 226 | 1134 | 22 | 6 | 3 |
| 479 | 20261605 | 293 | 1800 | 33 | 2 | 3 |
| 480 | 20261606 | 247 | 1321 | 35 | 5 | 1 |
| 481 | 20261607 | 201 | 591 | 18 | 4 | 1 |
| 482 | 20261608 | 234 | 1239 | 29 | 5 | 1 |
| 483 | 20261609 | 218 | 956 | 20 | 3 | 1 |
| 484 | 20261610 | 178 | 847 | 12 | 6 | 0 |
| 485 | 20261611 | 206 | 931 | 18 | 6 | 1 |
| 486 | 20261612 | 178 | 773 | 12 | 6 | 1 |
| 487 | 20261613 | 282 | 1639 | 34 | 8 | 1 |
| 488 | 20261614 | 271 | 1644 | 11 | 11 | 3 |
| 489 | 20261615 | 288 | 1536 | 22 | 4 | 3 |
| 490 | 20261616 | 264 | 1511 | 37 | 7 | 2 |
| 491 | 20261617 | 240 | 1165 | 20 | 8 | 3 |
| 492 | 20261618 | 195 | 883 | 16 | 11 | 1 |
| 493 | 20261619 | 209 | 1288 | 21 | 6 | 0 |
| 494 | 20261620 | 264 | 1493 | 33 | 3 | 3 |
| 495 | 20261621 | 270 | 1342 | 40 | 2 | 4 |
| 496 | 20261622 | 258 | 1626 | 26 | 5 | 3 |
| 497 | 20261623 | 268 | 1533 | 37 | 1 | 4 |
| 498 | 20261624 | 265 | 1400 | 26 | 6 | 1 |
| 499 | 20261625 | 293 | 1310 | 34 | 7 | 3 |
| 500 | 20261626 | 249 | 1023 | 25 | 3 | 1 |
| 501 | 20261627 | 251 | 1352 | 30 | 5 | 1 |
| 502 | 20261628 | 226 | 1009 | 12 | 1 | 2 |
| 503 | 20261629 | 226 | 1222 | 20 | 4 | 0 |
| 504 | 20261630 | 284 | 1853 | 19 | 2 | 3 |
| 505 | 20261631 | 270 | 1274 | 28 | 3 | 3 |
| 506 | 20261632 | 234 | 1084 | 22 | 4 | 2 |
| 507 | 20261633 | 344 | 1929 | 44 | 6 | 3 |
| 508 | 20261634 | 197 | 1092 | 19 | 5 | 1 |
| 509 | 20261635 | 426 | 1993 | 54 | 5 | 4 |
| 510 | 20261636 | 241 | 1164 | 26 | 5 | 1 |
| 511 | 20261637 | 470 | 2082 | 48 | 10 | 2 |
| 512 | 20261638 | 221 | 1378 | 14 | 3 | 1 |
| 513 | 20261639 | 246 | 1321 | 22 | 9 | 2 |
| 514 | 20261640 | 242 | 1556 | 12 | 5 | 2 |
| 515 | 20261641 | 357 | 2166 | 39 | 5 | 4 |
| 516 | 20261642 | 182 | 1150 | 18 | 1 | 2 |
| 517 | 20261643 | 321 | 1820 | 20 | 6 | 6 |
| 518 | 20261644 | 199 | 874 | 19 | 8 | 2 |
| 519 | 20261645 | 307 | 1422 | 22 | 2 | 3 |
| 520 | 20261646 | 282 | 1503 | 25 | 3 | 2 |
| 521 | 20261647 | 197 | 990 | 17 | 10 | 2 |
| 522 | 20261648 | 228 | 869 | 17 | 3 | 1 |
| 523 | 20261649 | 263 | 1383 | 27 | 11 | 3 |
| 524 | 20261650 | 187 | 941 | 17 | 2 | 0 |
| 525 | 20261651 | 197 | 801 | 19 | 9 | 3 |
| 526 | 20261652 | 268 | 1239 | 34 | 7 | 3 |
| 527 | 20261653 | 276 | 1513 | 19 | 2 | 1 |
| 528 | 20261654 | 268 | 1009 | 19 | 1 | 3 |
| 529 | 20261655 | 285 | 1727 | 43 | 3 | 2 |
| 530 | 20261656 | 258 | 1525 | 26 | 3 | 4 |
| 531 | 20261657 | 349 | 2149 | 23 | 14 | 3 |
| 532 | 20261658 | 235 | 1041 | 29 | 4 | 2 |
| 533 | 20261659 | 209 | 822 | 26 | 8 | 2 |
| 534 | 20261660 | 276 | 1376 | 20 | 3 | 2 |
| 535 | 20261661 | 251 | 781 | 13 | 5 | 0 |
| 536 | 20261662 | 177 | 741 | 25 | 3 | 2 |
| 537 | 20261663 | 298 | 1800 | 30 | 7 | 3 |
| 538 | 20261664 | 371 | 1663 | 44 | 6 | 3 |
| 539 | 20261665 | 200 | 998 | 15 | 4 | 1 |
| 540 | 20261666 | 284 | 1412 | 60 | 4 | 2 |
| 541 | 20261667 | 226 | 1163 | 28 | 11 | 4 |
| 542 | 20261668 | 329 | 1546 | 32 | 5 | 2 |
| 543 | 20261669 | 213 | 1263 | 25 | 2 | 0 |
| 544 | 20261670 | 296 | 1609 | 37 | 9 | 3 |
| 545 | 20261671 | 198 | 1000 | 23 | 6 | 4 |
| 546 | 20261672 | 264 | 1442 | 32 | 4 | 1 |
| 547 | 20261673 | 292 | 1646 | 18 | 4 | 6 |
| 548 | 20261674 | 192 | 931 | 19 | 8 | 2 |
| 549 | 20261675 | 225 | 1299 | 28 | 4 | 3 |
| 550 | 20261676 | 189 | 988 | 6 | 3 | 1 |
| 551 | 20261677 | 189 | 745 | 22 | 5 | 1 |
| 552 | 20261678 | 181 | 1093 | 18 | 4 | 2 |
| 553 | 20261679 | 235 | 1120 | 14 | 5 | 4 |
| 554 | 20261680 | 248 | 1150 | 23 | 5 | 1 |
| 555 | 20261681 | 276 | 1556 | 38 | 6 | 4 |
| 556 | 20261682 | 215 | 1092 | 9 | 4 | 2 |
| 557 | 20261683 | 216 | 1070 | 16 | 5 | 2 |
| 558 | 20261684 | 355 | 2268 | 41 | 9 | 1 |
| 559 | 20261685 | 188 | 1042 | 29 | 1 | 0 |
| 560 | 20261686 | 189 | 792 | 17 | 1 | 1 |
| 561 | 20261687 | 218 | 1291 | 26 | 1 | 0 |
| 562 | 20261688 | 283 | 1644 | 35 | 4 | 5 |
| 563 | 20261689 | 278 | 1762 | 35 | 5 | 2 |
| 564 | 20261690 | 230 | 1343 | 14 | 8 | 3 |
| 565 | 20261691 | 248 | 1177 | 29 | 5 | 5 |
| 566 | 20261692 | 321 | 1965 | 24 | 0 | 4 |
| 567 | 20261693 | 325 | 2078 | 50 | 3 | 4 |
| 568 | 20261694 | 235 | 1098 | 21 | 4 | 0 |
| 569 | 20261695 | 192 | 955 | 19 | 1 | 3 |
| 570 | 20261696 | 150 | 460 | 9 | 4 | 3 |
| 571 | 20261697 | 356 | 1853 | 22 | 3 | 4 |
| 572 | 20261698 | 199 | 1078 | 21 | 3 | 1 |
| 573 | 20261699 | 219 | 824 | 23 | 4 | 1 |
| 574 | 20261700 | 175 | 673 | 15 | 6 | 1 |
| 575 | 20261701 | 194 | 855 | 4 | 3 | 3 |
| 576 | 20261702 | 161 | 735 | 16 | 4 | 2 |
| 577 | 20261703 | 192 | 926 | 22 | 3 | 1 |
| 578 | 20261704 | 328 | 1989 | 39 | 5 | 4 |
| 579 | 20261705 | 187 | 991 | 19 | 1 | 0 |
| 580 | 20261706 | 266 | 1384 | 18 | 3 | 3 |
| 581 | 20261707 | 195 | 802 | 17 | 4 | 2 |
| 582 | 20261708 | 218 | 1016 | 16 | 2 | 1 |
| 583 | 20261709 | 192 | 769 | 15 | 2 | 1 |
| 584 | 20261710 | 253 | 1235 | 30 | 5 | 2 |
| 585 | 20261711 | 231 | 1081 | 27 | 7 | 2 |
| 586 | 20261712 | 270 | 1391 | 28 | 6 | 1 |
| 587 | 20261713 | 334 | 1727 | 25 | 8 | 3 |
| 588 | 20261714 | 228 | 1111 | 11 | 6 | 2 |
| 589 | 20261715 | 223 | 1137 | 9 | 7 | 4 |
| 590 | 20261716 | 180 | 674 | 17 | 3 | 0 |
| 591 | 20261717 | 217 | 1276 | 18 | 2 | 0 |
| 592 | 20261718 | 287 | 1510 | 28 | 6 | 2 |
| 593 | 20261719 | 188 | 835 | 16 | 2 | 1 |
| 594 | 20261720 | 275 | 1415 | 18 | 4 | 1 |
| 595 | 20261721 | 246 | 1113 | 17 | 3 | 2 |
| 596 | 20261722 | 284 | 1439 | 28 | 3 | 1 |
| 597 | 20261723 | 222 | 938 | 27 | 5 | 1 |
| 598 | 20261724 | 257 | 1400 | 22 | 9 | 1 |
| 599 | 20261725 | 214 | 1185 | 19 | 6 | 2 |
| 600 | 20261726 | 230 | 1133 | 29 | 5 | 4 |
| 601 | 20261727 | 213 | 928 | 13 | 4 | 1 |
| 602 | 20261728 | 216 | 903 | 21 | 2 | 1 |
| 603 | 20261729 | 168 | 686 | 18 | 4 | 0 |
| 604 | 20261730 | 297 | 1700 | 38 | 2 | 5 |
| 605 | 20261731 | 242 | 1292 | 20 | 5 | 1 |
| 606 | 20261732 | 266 | 1099 | 21 | 5 | 1 |
| 607 | 20261733 | 413 | 2042 | 44 | 2 | 2 |
| 608 | 20261734 | 257 | 1152 | 21 | 4 | 2 |
| 609 | 20261735 | 256 | 1096 | 14 | 3 | 3 |
| 610 | 20261736 | 218 | 1268 | 9 | 4 | 2 |
| 611 | 20261737 | 307 | 1950 | 32 | 5 | 2 |
| 612 | 20261738 | 315 | 1827 | 28 | 7 | 2 |
| 613 | 20261739 | 292 | 1627 | 16 | 5 | 3 |
| 614 | 20261740 | 274 | 1475 | 32 | 4 | 2 |
| 615 | 20261741 | 255 | 1169 | 28 | 7 | 4 |
| 616 | 20261742 | 355 | 1829 | 26 | 2 | 5 |
| 617 | 20261743 | 263 | 1516 | 24 | 7 | 3 |
| 618 | 20261744 | 177 | 721 | 6 | 4 | 0 |
| 619 | 20261745 | 286 | 1610 | 38 | 3 | 1 |
| 620 | 20261746 | 199 | 1023 | 24 | 7 | 1 |
| 621 | 20261747 | 188 | 760 | 11 | 8 | 2 |
| 622 | 20261748 | 280 | 1445 | 40 | 8 | 3 |
| 623 | 20261749 | 312 | 1924 | 29 | 12 | 3 |
| 624 | 20261750 | 320 | 1787 | 35 | 9 | 5 |
| 625 | 20261751 | 279 | 1351 | 33 | 10 | 3 |
| 626 | 20261752 | 288 | 1741 | 40 | 9 | 4 |
| 627 | 20261753 | 300 | 1562 | 37 | 4 | 4 |
| 628 | 20261754 | 206 | 936 | 18 | 1 | 5 |
| 629 | 20261755 | 192 | 958 | 20 | 4 | 1 |
| 630 | 20261756 | 202 | 729 | 11 | 3 | 1 |
| 631 | 20261757 | 189 | 820 | 15 | 3 | 2 |
| 632 | 20261758 | 262 | 1516 | 27 | 3 | 5 |
| 633 | 20261759 | 262 | 1294 | 21 | 12 | 4 |
| 634 | 20261760 | 269 | 1503 | 37 | 3 | 2 |
| 635 | 20261761 | 231 | 1169 | 30 | 5 | 2 |
| 636 | 20261762 | 209 | 768 | 13 | 6 | 4 |
| 637 | 20261763 | 193 | 746 | 18 | 4 | 3 |
| 638 | 20261764 | 202 | 1026 | 20 | 5 | 1 |
| 639 | 20261765 | 225 | 1322 | 26 | 9 | 2 |
| 640 | 20261766 | 227 | 816 | 20 | 4 | 3 |
| 641 | 20261767 | 236 | 1130 | 27 | 7 | 0 |
| 642 | 20261768 | 314 | 1629 | 33 | 8 | 2 |
| 643 | 20261769 | 238 | 1097 | 29 | 3 | 3 |
| 644 | 20261770 | 289 | 1626 | 19 | 2 | 4 |
| 645 | 20261771 | 286 | 1454 | 36 | 10 | 4 |
| 646 | 20261772 | 209 | 1162 | 22 | 5 | 0 |
| 647 | 20261773 | 248 | 1147 | 22 | 7 | 2 |
| 648 | 20261774 | 273 | 1379 | 42 | 6 | 2 |
| 649 | 20261775 | 300 | 1563 | 31 | 5 | 4 |
| 650 | 20261776 | 287 | 1227 | 24 | 8 | 4 |
| 651 | 20261777 | 318 | 1671 | 33 | 7 | 3 |
| 652 | 20261778 | 294 | 1373 | 36 | 7 | 4 |
| 653 | 20261779 | 211 | 997 | 15 | 8 | 0 |
| 654 | 20261780 | 295 | 1216 | 31 | 5 | 3 |
| 655 | 20261781 | 241 | 1025 | 23 | 7 | 4 |
| 656 | 20261782 | 180 | 760 | 16 | 3 | 1 |
| 657 | 20261783 | 286 | 1726 | 34 | 10 | 2 |
| 658 | 20261784 | 210 | 1052 | 19 | 3 | 1 |
| 659 | 20261785 | 243 | 1037 | 24 | 2 | 2 |
| 660 | 20261786 | 251 | 1421 | 16 | 3 | 2 |
| 661 | 20261787 | 191 | 824 | 15 | 2 | 0 |
| 662 | 20261788 | 221 | 1027 | 28 | 4 | 5 |
| 663 | 20261789 | 230 | 1018 | 25 | 6 | 4 |
| 664 | 20261790 | 178 | 617 | 11 | 3 | 2 |
| 665 | 20261791 | 179 | 642 | 11 | 5 | 0 |
| 666 | 20261792 | 315 | 1571 | 36 | 9 | 5 |
| 667 | 20261793 | 253 | 1215 | 11 | 4 | 2 |
| 668 | 20261794 | 331 | 1708 | 42 | 4 | 2 |
| 669 | 20261795 | 234 | 1073 | 14 | 12 | 4 |
| 670 | 20261796 | 239 | 1257 | 22 | 4 | 0 |
| 671 | 20261797 | 306 | 1778 | 25 | 7 | 3 |
| 672 | 20261798 | 205 | 866 | 24 | 1 | 2 |
| 673 | 20261799 | 210 | 758 | 15 | 11 | 2 |
| 674 | 20261800 | 271 | 1333 | 29 | 6 | 3 |
| 675 | 20261801 | 310 | 1719 | 23 | 1 | 2 |
| 676 | 20261802 | 230 | 1225 | 32 | 7 | 4 |
| 677 | 20261803 | 251 | 1542 | 20 | 6 | 1 |
| 678 | 20261804 | 218 | 967 | 18 | 9 | 3 |
| 679 | 20261805 | 415 | 2314 | 42 | 7 | 5 |
| 680 | 20261806 | 292 | 1609 | 23 | 3 | 3 |
| 681 | 20261807 | 302 | 1380 | 28 | 2 | 0 |
| 682 | 20261808 | 318 | 1891 | 20 | 10 | 2 |
| 683 | 20261809 | 288 | 1652 | 24 | 7 | 3 |
| 684 | 20261810 | 246 | 1193 | 30 | 5 | 4 |
| 685 | 20261811 | 268 | 1487 | 30 | 4 | 2 |
| 686 | 20261812 | 219 | 755 | 13 | 9 | 2 |
| 687 | 20261813 | 239 | 1222 | 22 | 1 | 2 |
| 688 | 20261814 | 263 | 1373 | 24 | 5 | 2 |
| 689 | 20261815 | 194 | 1012 | 14 | 4 | 1 |
| 690 | 20261816 | 348 | 2025 | 50 | 6 | 1 |
| 691 | 20261817 | 169 | 738 | 16 | 2 | 0 |
| 692 | 20261818 | 266 | 1550 | 24 | 8 | 2 |
| 693 | 20261819 | 292 | 1895 | 33 | 6 | 3 |
| 694 | 20261820 | 334 | 1797 | 48 | 6 | 3 |
| 695 | 20261821 | 213 | 1296 | 28 | 5 | 2 |
| 696 | 20261822 | 197 | 950 | 15 | 5 | 0 |
| 697 | 20261823 | 253 | 1539 | 33 | 4 | 3 |
| 698 | 20261824 | 190 | 587 | 14 | 3 | 4 |
| 699 | 20261825 | 255 | 1414 | 28 | 3 | 3 |
| 700 | 20261826 | 171 | 1011 | 13 | 4 | 0 |
| 701 | 20261827 | 203 | 892 | 15 | 1 | 3 |
| 702 | 20261828 | 273 | 1353 | 24 | 7 | 3 |
| 703 | 20261829 | 234 | 740 | 19 | 5 | 3 |
| 704 | 20261830 | 265 | 1271 | 24 | 10 | 3 |
| 705 | 20261831 | 236 | 1104 | 22 | 5 | 4 |
| 706 | 20261832 | 256 | 1307 | 40 | 4 | 4 |
| 707 | 20261833 | 237 | 1465 | 24 | 4 | 2 |
| 708 | 20261834 | 310 | 1579 | 34 | 6 | 3 |
| 709 | 20261835 | 206 | 787 | 10 | 5 | 2 |
| 710 | 20261836 | 270 | 1602 | 28 | 2 | 2 |
| 711 | 20261837 | 236 | 1342 | 25 | 0 | 1 |
| 712 | 20261838 | 246 | 1351 | 28 | 3 | 2 |
| 713 | 20261839 | 199 | 1013 | 22 | 1 | 4 |
| 714 | 20261840 | 286 | 1317 | 14 | 4 | 2 |
| 715 | 20261841 | 274 | 1668 | 25 | 6 | 3 |
| 716 | 20261842 | 269 | 1268 | 22 | 7 | 3 |
| 717 | 20261843 | 241 | 1243 | 21 | 7 | 4 |
| 718 | 20261844 | 222 | 1028 | 27 | 5 | 1 |
| 719 | 20261845 | 194 | 1012 | 14 | 4 | 3 |
| 720 | 20261846 | 229 | 1191 | 22 | 3 | 3 |
| 721 | 20261847 | 220 | 1264 | 24 | 5 | 1 |
| 722 | 20261848 | 206 | 880 | 21 | 1 | 3 |
| 723 | 20261849 | 194 | 1034 | 17 | 1 | 0 |
| 724 | 20261850 | 313 | 1607 | 29 | 5 | 3 |
| 725 | 20261851 | 499 | 3177 | 63 | 12 | 3 |
| 726 | 20261852 | 254 | 1385 | 18 | 5 | 2 |
| 727 | 20261853 | 237 | 1301 | 21 | 10 | 1 |
| 728 | 20261854 | 233 | 1318 | 25 | 5 | 1 |
| 729 | 20261855 | 198 | 1362 | 23 | 1 | 1 |
| 730 | 20261856 | 223 | 1111 | 30 | 7 | 2 |
| 731 | 20261857 | 200 | 1213 | 14 | 6 | 0 |
| 732 | 20261858 | 272 | 1300 | 30 | 3 | 2 |
| 733 | 20261859 | 248 | 1428 | 27 | 2 | 2 |
| 734 | 20261860 | 239 | 1345 | 17 | 6 | 1 |
| 735 | 20261861 | 249 | 1331 | 22 | 5 | 3 |
| 736 | 20261862 | 229 | 1242 | 18 | 6 | 1 |
| 737 | 20261863 | 240 | 1510 | 29 | 5 | 2 |
| 738 | 20261864 | 226 | 1170 | 29 | 4 | 3 |
| 739 | 20261865 | 269 | 1465 | 12 | 2 | 1 |
| 740 | 20261866 | 226 | 1045 | 23 | 8 | 4 |
| 741 | 20261867 | 218 | 1032 | 15 | 6 | 2 |
| 742 | 20261868 | 296 | 1865 | 29 | 3 | 2 |
| 743 | 20261869 | 250 | 1194 | 35 | 4 | 2 |
| 744 | 20261870 | 234 | 1287 | 14 | 6 | 2 |
| 745 | 20261871 | 163 | 609 | 8 | 3 | 0 |
| 746 | 20261872 | 286 | 1703 | 35 | 6 | 3 |
| 747 | 20261873 | 187 | 877 | 18 | 4 | 3 |
| 748 | 20261874 | 277 | 1504 | 31 | 14 | 5 |
| 749 | 20261875 | 242 | 1130 | 29 | 7 | 4 |
| 750 | 20261876 | 306 | 2237 | 31 | 3 | 2 |
| 751 | 20261877 | 219 | 1155 | 21 | 5 | 4 |
| 752 | 20261878 | 215 | 1193 | 22 | 6 | 2 |
| 753 | 20261879 | 242 | 1443 | 26 | 9 | 3 |
| 754 | 20261880 | 222 | 1378 | 16 | 5 | 0 |
| 755 | 20261881 | 218 | 1065 | 21 | 5 | 1 |
| 756 | 20261882 | 234 | 1159 | 21 | 3 | 0 |
| 757 | 20261883 | 261 | 1337 | 28 | 5 | 1 |
| 758 | 20261884 | 220 | 1098 | 30 | 6 | 0 |
| 759 | 20261885 | 272 | 1330 | 35 | 4 | 1 |
| 760 | 20261886 | 230 | 1030 | 29 | 5 | 2 |
| 761 | 20261887 | 245 | 1338 | 19 | 1 | 2 |
| 762 | 20261888 | 241 | 1123 | 27 | 4 | 4 |
| 763 | 20261889 | 263 | 1385 | 25 | 1 | 4 |
| 764 | 20261890 | 288 | 1330 | 23 | 3 | 1 |
| 765 | 20261891 | 193 | 1071 | 23 | 7 | 4 |
| 766 | 20261892 | 225 | 1038 | 16 | 2 | 5 |
| 767 | 20261893 | 252 | 1455 | 20 | 6 | 3 |
| 768 | 20261894 | 211 | 994 | 22 | 7 | 1 |
| 769 | 20261895 | 210 | 818 | 18 | 6 | 2 |
| 770 | 20261896 | 210 | 973 | 20 | 2 | 2 |
| 771 | 20261897 | 229 | 1032 | 27 | 4 | 2 |
| 772 | 20261898 | 286 | 1550 | 42 | 7 | 3 |
| 773 | 20261899 | 232 | 1259 | 26 | 5 | 1 |
| 774 | 20261900 | 375 | 2307 | 39 | 1 | 2 |
| 775 | 20261901 | 254 | 1216 | 20 | 4 | 2 |
| 776 | 20261902 | 228 | 1417 | 18 | 13 | 0 |
| 777 | 20261903 | 265 | 1163 | 30 | 0 | 1 |
| 778 | 20261904 | 297 | 1470 | 22 | 7 | 5 |
| 779 | 20261905 | 300 | 1599 | 36 | 7 | 3 |
| 780 | 20261906 | 210 | 1241 | 17 | 5 | 1 |
| 781 | 20261907 | 219 | 1036 | 29 | 2 | 3 |
| 782 | 20261908 | 301 | 1675 | 20 | 7 | 5 |
| 783 | 20261909 | 171 | 867 | 17 | 3 | 1 |
| 784 | 20261910 | 267 | 1106 | 45 | 5 | 6 |
| 785 | 20261911 | 232 | 1188 | 19 | 9 | 2 |
| 786 | 20261912 | 256 | 1332 | 28 | 9 | 2 |
| 787 | 20261913 | 261 | 1699 | 22 | 3 | 2 |
| 788 | 20261914 | 272 | 1372 | 26 | 9 | 3 |
| 789 | 20261915 | 302 | 1496 | 31 | 5 | 5 |
| 790 | 20261916 | 242 | 1422 | 28 | 6 | 4 |
| 791 | 20261917 | 201 | 911 | 21 | 7 | 4 |
| 792 | 20261918 | 371 | 2007 | 24 | 3 | 2 |
| 793 | 20261919 | 276 | 1258 | 33 | 6 | 2 |
| 794 | 20261920 | 304 | 1679 | 31 | 2 | 4 |
| 795 | 20261921 | 166 | 574 | 17 | 6 | 1 |
| 796 | 20261922 | 196 | 887 | 16 | 3 | 1 |
| 797 | 20261923 | 312 | 1703 | 32 | 9 | 4 |
| 798 | 20261924 | 269 | 1685 | 22 | 2 | 3 |
| 799 | 20261925 | 224 | 1102 | 22 | 2 | 2 |
| 800 | 20261926 | 209 | 980 | 14 | 7 | 1 |
| 801 | 20261927 | 238 | 1394 | 42 | 2 | 2 |
| 802 | 20261928 | 233 | 1056 | 25 | 8 | 1 |
| 803 | 20261929 | 225 | 1118 | 9 | 3 | 2 |
| 804 | 20261930 | 208 | 1035 | 16 | 3 | 2 |
| 805 | 20261931 | 302 | 1465 | 39 | 5 | 4 |
| 806 | 20261932 | 258 | 1186 | 23 | 7 | 2 |
| 807 | 20261933 | 246 | 1336 | 29 | 8 | 2 |
| 808 | 20261934 | 242 | 1137 | 21 | 4 | 2 |
| 809 | 20261935 | 299 | 1582 | 29 | 8 | 2 |
| 810 | 20261936 | 233 | 1151 | 39 | 6 | 4 |
| 811 | 20261937 | 238 | 1084 | 15 | 3 | 1 |
| 812 | 20261938 | 373 | 2823 | 45 | 6 | 2 |
| 813 | 20261939 | 188 | 860 | 15 | 6 | 2 |
| 814 | 20261940 | 259 | 1902 | 17 | 6 | 4 |
| 815 | 20261941 | 215 | 961 | 27 | 5 | 2 |
| 816 | 20261942 | 274 | 1603 | 18 | 3 | 3 |
| 817 | 20261943 | 227 | 1022 | 35 | 3 | 4 |
| 818 | 20261944 | 254 | 1479 | 16 | 2 | 6 |
| 819 | 20261945 | 256 | 1129 | 18 | 4 | 2 |
| 820 | 20261946 | 202 | 964 | 8 | 10 | 3 |
| 821 | 20261947 | 260 | 1211 | 26 | 5 | 2 |
| 822 | 20261948 | 181 | 877 | 16 | 7 | 2 |
| 823 | 20261949 | 269 | 1434 | 26 | 12 | 4 |
| 824 | 20261950 | 236 | 1019 | 29 | 12 | 3 |
| 825 | 20261951 | 278 | 1395 | 37 | 2 | 2 |
| 826 | 20261952 | 237 | 1167 | 22 | 3 | 2 |
| 827 | 20261953 | 385 | 2227 | 50 | 9 | 2 |
| 828 | 20261954 | 212 | 1060 | 22 | 5 | 3 |
| 829 | 20261955 | 279 | 1410 | 18 | 8 | 4 |
| 830 | 20261956 | 237 | 1012 | 22 | 1 | 3 |
| 831 | 20261957 | 220 | 664 | 16 | 2 | 1 |
| 832 | 20261958 | 208 | 908 | 19 | 4 | 1 |
| 833 | 20261959 | 176 | 796 | 16 | 3 | 1 |
| 834 | 20261960 | 239 | 1110 | 25 | 4 | 4 |
| 835 | 20261961 | 188 | 743 | 21 | 2 | 2 |
| 836 | 20261962 | 228 | 1161 | 29 | 12 | 3 |
| 837 | 20261963 | 236 | 1201 | 25 | 3 | 2 |
| 838 | 20261964 | 179 | 872 | 17 | 7 | 2 |
| 839 | 20261965 | 346 | 1912 | 50 | 7 | 3 |
| 840 | 20261966 | 274 | 1507 | 29 | 5 | 2 |
| 841 | 20261967 | 260 | 1549 | 31 | 7 | 3 |
| 842 | 20261968 | 216 | 1186 | 20 | 4 | 2 |
| 843 | 20261969 | 274 | 1215 | 30 | 8 | 3 |
| 844 | 20261970 | 242 | 1174 | 33 | 8 | 4 |
| 845 | 20261971 | 199 | 818 | 10 | 3 | 0 |
| 846 | 20261972 | 394 | 2387 | 37 | 11 | 6 |
| 847 | 20261973 | 279 | 1458 | 33 | 9 | 3 |
| 848 | 20261974 | 222 | 1247 | 26 | 4 | 2 |
| 849 | 20261975 | 207 | 947 | 9 | 7 | 2 |
| 850 | 20261976 | 263 | 1147 | 31 | 7 | 4 |
| 851 | 20261977 | 196 | 1145 | 27 | 3 | 0 |
| 852 | 20261978 | 187 | 826 | 17 | 4 | 1 |
| 853 | 20261979 | 203 | 827 | 20 | 3 | 2 |
| 854 | 20261980 | 222 | 1028 | 13 | 7 | 1 |
| 855 | 20261981 | 183 | 925 | 20 | 3 | 2 |
| 856 | 20261982 | 209 | 868 | 12 | 2 | 1 |
| 857 | 20261983 | 254 | 1454 | 20 | 2 | 2 |
| 858 | 20261984 | 213 | 1082 | 16 | 4 | 2 |
| 859 | 20261985 | 232 | 1291 | 14 | 5 | 1 |
| 860 | 20261986 | 235 | 1422 | 17 | 5 | 4 |
| 861 | 20261987 | 322 | 1462 | 43 | 6 | 4 |
| 862 | 20261988 | 248 | 1494 | 27 | 3 | 2 |
| 863 | 20261989 | 186 | 949 | 22 | 5 | 2 |
| 864 | 20261990 | 188 | 778 | 17 | 2 | 2 |
| 865 | 20261991 | 231 | 1198 | 28 | 11 | 1 |
| 866 | 20261992 | 186 | 799 | 16 | 7 | 3 |
| 867 | 20261993 | 255 | 1165 | 34 | 3 | 0 |
| 868 | 20261994 | 356 | 2023 | 43 | 6 | 1 |
| 869 | 20261995 | 287 | 1810 | 23 | 4 | 2 |
| 870 | 20261996 | 196 | 1009 | 13 | 4 | 2 |
| 871 | 20261997 | 187 | 857 | 20 | 4 | 1 |
| 872 | 20261998 | 233 | 1271 | 23 | 2 | 3 |
| 873 | 20261999 | 271 | 1434 | 25 | 2 | 2 |
| 874 | 20262000 | 263 | 1258 | 39 | 7 | 3 |
| 875 | 20262001 | 251 | 1318 | 24 | 7 | 3 |
| 876 | 20262002 | 192 | 713 | 14 | 3 | 1 |
| 877 | 20262003 | 241 | 960 | 12 | 5 | 2 |
| 878 | 20262004 | 210 | 951 | 14 | 3 | 0 |
| 879 | 20262005 | 284 | 1181 | 40 | 4 | 3 |
| 880 | 20262006 | 258 | 1312 | 21 | 3 | 4 |
| 881 | 20262007 | 253 | 1304 | 18 | 9 | 4 |
| 882 | 20262008 | 215 | 1345 | 21 | 4 | 1 |
| 883 | 20262009 | 242 | 1286 | 35 | 2 | 2 |
| 884 | 20262010 | 350 | 1642 | 16 | 6 | 3 |
| 885 | 20262011 | 292 | 1580 | 22 | 3 | 1 |
| 886 | 20262012 | 238 | 1188 | 26 | 5 | 4 |
| 887 | 20262013 | 232 | 1349 | 25 | 5 | 4 |
| 888 | 20262014 | 198 | 1007 | 14 | 3 | 1 |
| 889 | 20262015 | 363 | 2192 | 34 | 5 | 2 |
| 890 | 20262016 | 249 | 1442 | 16 | 6 | 2 |
| 891 | 20262017 | 313 | 1474 | 18 | 5 | 4 |
| 892 | 20262018 | 203 | 1005 | 18 | 5 | 4 |
| 893 | 20262019 | 220 | 1063 | 17 | 4 | 0 |
| 894 | 20262020 | 276 | 1650 | 25 | 3 | 4 |
| 895 | 20262021 | 380 | 2532 | 27 | 6 | 3 |
| 896 | 20262022 | 295 | 1832 | 27 | 2 | 2 |
| 897 | 20262023 | 269 | 1372 | 24 | 5 | 2 |
| 898 | 20262024 | 178 | 645 | 13 | 4 | 1 |
| 899 | 20262025 | 268 | 1339 | 24 | 7 | 4 |
| 900 | 20262026 | 208 | 1159 | 20 | 0 | 1 |
| 901 | 20262027 | 277 | 1375 | 24 | 6 | 3 |
| 902 | 20262028 | 198 | 884 | 17 | 4 | 2 |
| 903 | 20262029 | 197 | 1016 | 19 | 6 | 0 |
| 904 | 20262030 | 214 | 796 | 17 | 5 | 1 |
| 905 | 20262031 | 254 | 1486 | 15 | 6 | 2 |
| 906 | 20262032 | 357 | 1537 | 21 | 16 | 3 |
| 907 | 20262033 | 269 | 1455 | 30 | 2 | 3 |
| 908 | 20262034 | 253 | 1182 | 19 | 14 | 1 |
| 909 | 20262035 | 365 | 2283 | 49 | 7 | 3 |
| 910 | 20262036 | 234 | 1043 | 12 | 7 | 3 |
| 911 | 20262037 | 306 | 1971 | 22 | 5 | 3 |
| 912 | 20262038 | 193 | 1040 | 21 | 2 | 1 |
| 913 | 20262039 | 256 | 1332 | 25 | 3 | 1 |
| 914 | 20262040 | 198 | 866 | 21 | 5 | 3 |
| 915 | 20262041 | 207 | 971 | 21 | 4 | 1 |
| 916 | 20262042 | 249 | 1491 | 28 | 5 | 3 |
| 917 | 20262043 | 241 | 1189 | 22 | 9 | 3 |
| 918 | 20262044 | 263 | 1057 | 22 | 3 | 2 |
| 919 | 20262045 | 421 | 2528 | 61 | 4 | 4 |
| 920 | 20262046 | 219 | 1233 | 24 | 0 | 1 |
| 921 | 20262047 | 229 | 899 | 10 | 6 | 2 |
| 922 | 20262048 | 184 | 654 | 16 | 5 | 1 |
| 923 | 20262049 | 213 | 969 | 20 | 4 | 0 |
| 924 | 20262050 | 201 | 866 | 14 | 2 | 1 |
| 925 | 20262051 | 500 | 3058 | 48 | 18 | 2 |
| 926 | 20262052 | 227 | 952 | 29 | 8 | 3 |
| 927 | 20262053 | 274 | 1772 | 28 | 3 | 3 |
| 928 | 20262054 | 313 | 1801 | 25 | 5 | 2 |
| 929 | 20262055 | 268 | 1339 | 19 | 4 | 2 |
| 930 | 20262056 | 296 | 1742 | 48 | 5 | 1 |
| 931 | 20262057 | 234 | 1101 | 23 | 6 | 2 |
| 932 | 20262058 | 307 | 1740 | 32 | 7 | 1 |
| 933 | 20262059 | 156 | 629 | 9 | 3 | 0 |
| 934 | 20262060 | 293 | 1866 | 34 | 5 | 2 |
| 935 | 20262061 | 235 | 1234 | 20 | 1 | 5 |
| 936 | 20262062 | 258 | 1520 | 39 | 7 | 2 |
| 937 | 20262063 | 290 | 1779 | 36 | 6 | 4 |
| 938 | 20262064 | 276 | 1494 | 32 | 1 | 3 |
| 939 | 20262065 | 320 | 1838 | 28 | 3 | 4 |
| 940 | 20262066 | 240 | 1106 | 28 | 6 | 1 |
| 941 | 20262067 | 282 | 1424 | 21 | 4 | 6 |
| 942 | 20262068 | 214 | 893 | 11 | 5 | 1 |
| 943 | 20262069 | 264 | 1278 | 35 | 3 | 3 |
| 944 | 20262070 | 230 | 1128 | 20 | 6 | 1 |
| 945 | 20262071 | 258 | 1487 | 25 | 2 | 2 |
| 946 | 20262072 | 330 | 2274 | 34 | 2 | 5 |
| 947 | 20262073 | 313 | 1629 | 21 | 8 | 4 |
| 948 | 20262074 | 316 | 1540 | 42 | 3 | 2 |
| 949 | 20262075 | 303 | 1645 | 37 | 9 | 4 |
| 950 | 20262076 | 251 | 1375 | 29 | 7 | 3 |
| 951 | 20262077 | 249 | 1514 | 28 | 4 | 1 |
| 952 | 20262078 | 261 | 1150 | 20 | 8 | 4 |
| 953 | 20262079 | 238 | 1013 | 19 | 7 | 3 |
| 954 | 20262080 | 226 | 1302 | 29 | 5 | 3 |
| 955 | 20262081 | 227 | 1236 | 22 | 3 | 2 |
| 956 | 20262082 | 240 | 1185 | 10 | 8 | 1 |
| 957 | 20262083 | 223 | 990 | 24 | 8 | 2 |
| 958 | 20262084 | 256 | 1373 | 31 | 6 | 4 |
| 959 | 20262085 | 273 | 1630 | 17 | 2 | 4 |
| 960 | 20262086 | 234 | 951 | 15 | 12 | 1 |
| 961 | 20262087 | 232 | 1188 | 20 | 3 | 3 |
| 962 | 20262088 | 255 | 1432 | 17 | 3 | 1 |
| 963 | 20262089 | 283 | 1552 | 24 | 0 | 3 |
| 964 | 20262090 | 303 | 1896 | 44 | 2 | 3 |
| 965 | 20262091 | 198 | 1088 | 9 | 7 | 1 |
| 966 | 20262092 | 236 | 1370 | 24 | 5 | 3 |
| 967 | 20262093 | 261 | 1061 | 22 | 6 | 3 |
| 968 | 20262094 | 236 | 1125 | 20 | 11 | 1 |
| 969 | 20262095 | 205 | 1059 | 30 | 2 | 2 |
| 970 | 20262096 | 306 | 1173 | 30 | 5 | 5 |
| 971 | 20262097 | 287 | 1470 | 31 | 2 | 4 |
| 972 | 20262098 | 260 | 1671 | 25 | 1 | 2 |
| 973 | 20262099 | 212 | 1014 | 19 | 1 | 3 |
| 974 | 20262100 | 221 | 1159 | 26 | 7 | 2 |
| 975 | 20262101 | 447 | 2664 | 57 | 7 | 5 |
| 976 | 20262102 | 255 | 1016 | 20 | 3 | 3 |
| 977 | 20262103 | 253 | 1102 | 24 | 8 | 4 |
| 978 | 20262104 | 190 | 758 | 23 | 7 | 1 |
| 979 | 20262105 | 272 | 1444 | 38 | 2 | 1 |
| 980 | 20262106 | 295 | 1471 | 19 | 6 | 1 |
| 981 | 20262107 | 213 | 1173 | 19 | 2 | 2 |
| 982 | 20262108 | 225 | 936 | 24 | 4 | 3 |
| 983 | 20262109 | 256 | 1296 | 15 | 5 | 1 |
| 984 | 20262110 | 140 | 527 | 4 | 3 | 0 |
| 985 | 20262111 | 309 | 1753 | 61 | 3 | 5 |
| 986 | 20262112 | 235 | 1371 | 13 | 6 | 5 |
| 987 | 20262113 | 191 | 1025 | 15 | 2 | 0 |
| 988 | 20262114 | 242 | 1199 | 21 | 1 | 2 |
| 989 | 20262115 | 237 | 1036 | 23 | 7 | 2 |
| 990 | 20262116 | 196 | 829 | 11 | 4 | 0 |
| 991 | 20262117 | 262 | 1311 | 20 | 2 | 2 |
| 992 | 20262118 | 226 | 1255 | 24 | 4 | 3 |
| 993 | 20262119 | 380 | 2115 | 21 | 3 | 4 |
| 994 | 20262120 | 248 | 1227 | 29 | 4 | 3 |
| 995 | 20262121 | 189 | 792 | 4 | 5 | 1 |
| 996 | 20262122 | 175 | 687 | 16 | 3 | 1 |
| 997 | 20262123 | 232 | 1302 | 22 | 1 | 2 |
| 998 | 20262124 | 260 | 1397 | 29 | 4 | 3 |
| 999 | 20262125 | 500 | 2292 | 31 | 2 | 6 |

## Throughput

- Turns simulated: 250113 (first play of each game)
- Aggregate: 127.1 turns/s over 1967.9s wall
- Per game thread: 31.9 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

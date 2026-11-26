# Simulation: city-upkeep-off (2026-11-26)

1000 games, first seed 20261126, brain `scripted`, turn limit off, safety cap 1200, 4 threads, started 2026-11-26T20:55:41Z. Wall time 1645.6s.

Source checkout at run start: `95c4ebf5798f6781d284c74c99f8ce68071da87b`; dirty: yes. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.

Command argv: `["/home/user/legatum/target/release/legatum-sim","--games","1000","--seed","20261126","--threads","4","--difficulty","hard","--rules-experiment","no-city-upkeep","--label","city-upkeep-off"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 1000 of 1000 |
| Game length, turns (mean / median / min-max) | 228.9 / 224 / 134-456 |
| Win reason mix | VP hold 98.2%, Capitals 1.8% |
| Median end turn | 224 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 1.8% / 98.2% / 0.0% / 0.0% |
| Games reaching the World War era (any empire) | 993 of 1000 (99.3%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1394 / 328 / 986 / 80 |
| Hold break rate | 23.5% of started; 25.0% of resolved (broken / (broken + completed)) |
| Hold wins | 982 |
| First reach to win, turns (mean / median / min-max) | 10.8 / 9 / 9-124 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 100 |
| Wins by a team another team beat to its target | 71 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 982 | 228.0 / 223 / 134-456 |
| Capitals | 18 | 274.3 / 263 / 220-395 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 95 | 9.5% |
| India | 45 | 4.5% |
| Byzantium | 85 | 8.5% |
| Germany | 157 | 15.7% |
| Celts | 125 | 12.5% |
| Mongols | 112 | 11.2% |
| Sparta | 125 | 12.5% |
| Athens | 146 | 14.6% |
| Aztecs | 110 | 11.0% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 982 | 98.2% |
| Conquest | 0 | 0.0% |
| Capitals | 18 | 1.8% |
| Turn limit | 0 | 0.0% |
| Draw | 0 | 0.0% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 899 | 95 | 10.6% |
| India | 895 | 45 | 5.0% |
| Byzantium | 878 | 85 | 9.7% |
| Germany | 880 | 157 | 17.8% |
| Celts | 896 | 125 | 14.0% |
| Mongols | 877 | 112 | 12.8% |
| Sparta | 891 | 125 | 14.0% |
| Athens | 895 | 146 | 16.3% |
| Aztecs | 889 | 110 | 12.4% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 8000 | 1000 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 899 | 95 | 10.6% |
| India | Hard | 895 | 45 | 5.0% |
| Byzantium | Hard | 878 | 85 | 9.7% |
| Germany | Hard | 880 | 157 | 17.8% |
| Celts | Hard | 896 | 125 | 14.0% |
| Mongols | Hard | 877 | 112 | 12.8% |
| Sparta | Hard | 891 | 125 | 14.0% |
| Athens | Hard | 895 | 146 | 16.3% |
| Aztecs | Hard | 889 | 110 | 12.4% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 899 | 160 | 17.8% | 114 | 144 | 179 | 223 | 310 |
| India | 895 | 453 | 50.6% | 58 | 91 | 126 | 200 | 280 |
| Byzantium | 878 | 669 | 76.2% | 68 | 84 | 109 | 149 | 385 |
| Germany | 880 | 301 | 34.2% | 99 | 129 | 164 | 213 | 287 |
| Celts | 896 | 583 | 65.1% | 49 | 182 | 210 | 210 | 240 |
| Mongols | 877 | 169 | 19.3% | 132 | 152 | 195 | 242 | 343 |
| Sparta | 891 | 134 | 15.0% | 115 | 158 | 199 | 267 | 326 |
| Athens | 895 | 293 | 32.7% | 119 | 157 | 226 | 255 | 255 |
| Aztecs | 889 | 262 | 29.5% | 135 | 180 | 225 | 237 | 284 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 134 | 185 | 224 | 279 | 456 |
| Winning team VP | 2 | 10 | 11 | 13 | 19 |
| First Soldier completed (turn, per empire) | 6 | 8 | 27 | 72 | 299 |
| First Settler completed (turn, per empire) | 3 | 14 | 16 | 25 | 175 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 8 | 18 | 23 | 44 | 286 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 7932 | 68 |
| First Settler completed | 7965 | 35 |
| First city founded | 8000 | 0 |
| Second city founded | 7899 | 101 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 1724 |
| 1 | 1492 |
| 2 | 1111 |
| 3 | 742 |
| 4 | 571 |
| 5 | 454 |
| 6 | 294 |
| 7 | 268 |
| 8 | 146 |
| 9 | 129 |
| 10 | 476 |
| 11 | 265 |
| 12 | 155 |
| 13 | 87 |
| 14 | 46 |
| 15 | 23 |
| 16 | 14 |
| 17 | 1 |
| 18 | 1 |
| 19 | 1 |

## End state

- Average game length: 228.9 turns
- Average cities per empire: 4.52
- Average techs per empire: 32.04
- Average eliminations per game: 2.22

## Strategy diversity: observed behavior (classifier v2)

1000 finished games; 8000 player seats; 1000 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4185** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 3299 | 211 | 6.4% | 41.2% | 21.1% |
| Wide | 2833 | 726 | 25.6% | 35.4% | 72.6% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 13 | 0 | 0.0% | 0.2% | 0.0% |
| Economic | 790 | 59 | 7.5% | 9.9% | 5.9% |
| Generalist | 1065 | 4 | 0.4% | 13.3% | 0.4% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 20.9% / 3.8% (961) | n/a / n/a (0) | 24.4% / 20.5% (13) | 22.7% / 7.8% (540) | 62.5% / 5.3% (682) |
| Wide | 75.4% / 3.8% (961) | — | n/a / n/a (0) | 71.8% / 6.4% (13) | 58.9% / 6.2% (547) | 87.9% / 2.7% (691) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 55.1% / 20.5% (13) | 21.8% / 6.4% (13) | n/a / n/a (0) | — | 44.4% / 18.5% (9) | 81.0% / 4.8% (7) |
| Economic | 69.5% / 7.8% (540) | 34.8% / 6.2% (547) | n/a / n/a (0) | 37.0% / 18.5% (9) | — | 86.6% / 6.2% (369) |
| Generalist | 32.2% / 5.3% (682) | 9.5% / 2.7% (691) | n/a / n/a (0) | 14.3% / 4.8% (7) | 7.2% / 6.2% (369) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 6 / 204 | 87 / 403 | 0 / 0 | 0 / 1 | 2 / 162 | 0 / 129 |
| India | 7 / 490 | 38 / 318 | 0 / 0 | 0 / 0 | 0 / 17 | 0 / 70 |
| Byzantium | 1 / 13 | 79 / 464 | 0 / 0 | 0 / 5 | 5 / 125 | 0 / 271 |
| Germany | 16 / 348 | 130 / 345 | 0 / 0 | 0 / 1 | 10 / 69 | 1 / 117 |
| Celts | 11 / 246 | 111 / 452 | 0 / 0 | 0 / 0 | 3 / 104 | 0 / 94 |
| Mongols | 68 / 719 | 40 / 94 | 0 / 0 | 0 / 0 | 4 / 15 | 0 / 49 |
| Sparta | 68 / 670 | 45 / 112 | 0 / 0 | 0 / 6 | 11 / 54 | 1 / 49 |
| Athens | 20 / 268 | 110 / 333 | 0 / 0 | 0 / 0 | 14 / 119 | 2 / 175 |
| Aztecs | 14 / 341 | 86 / 312 | 0 / 0 | 0 / 0 | 10 / 125 | 0 / 111 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Turtle/Wonder. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 1000 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 15738/1000 | 15.738 | 1000/1000 (100.0%) |
| Peace offers | 87887/1000 | 87.887 | 1000/1000 (100.0%) |
| Peace agreements | 13244/1000 | 13.244 | 1000/1000 (100.0%) |
| Battles | 1176591/1000 | 1176.591 | 1000/1000 (100.0%) |
| City captures | 22315/1000 | 22.315 | 1000/1000 (100.0%) |
| Settler captures | 4883/1000 | 4.883 | 984/1000 (98.4%) |
| Eliminations | 2218/1000 | 2.218 | 925/1000 (92.5%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 6904/8000 | 6904/8000 | 3250/1000 |
| Faction goals | 4868/8000 | 4868/8000 | 2118/1000 |
| Era objectives | 13774/8000 | 13774/8000 | 4071/1000 |
| Capitals taken | 2850/8000 | 2850/8000 | 1731/1000 |
| Total VP | 28396/8000 | 28396/8000 | 11170/1000 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 60/363 (16.5%) | 0 |
| Tied leaders (winner in set) | 631/637 (99.1%) | 0 |
| Any leader set | 691/1000 (69.1%) | 0 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 7955/8000 | 5 | 21 | 29 | 33 | 84 | -2.8 (1000) |
| Medieval | 7891/8000 | 31 | 58 | 62 | 66 | 123 | -2.1 (1000) |
| Renaissance | 7819/8000 | 61 | 84 | 89 | 98 | 182 | -3.5 (1000) |
| Industrial | 7566/8000 | 84 | 111 | 119 | 133 | 256 | -5.8 (1000) |
| Machine Age | 6808/8000 | 113 | 140 | 151 | 169 | 262 | -7.9 (1000) |
| World War | 5326/8000 | 133 | 164 | 179 | 201 | 299 | -9.9 (988) |
| Modern | 3161/8000 | 153 | 188 | 208 | 235 | 330 | -9.6 (858) |
| Future | 1375/8000 | 176 | 212 | 238 | 260 | 396 | -6.4 (514) |

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
| Medieval | Feudalism | 7869/8000 | 75 | 950 AD | n/a |
| Medieval | Machinery | 7881/8000 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 6965/8000 | 80 | 1100 AD | n/a |
| Medieval | Education | 7839/8000 | 86 | 1280 AD | 1088 AD |
| Medieval | Crop Rotation | 7821/8000 | 89 | 1370 AD | n/a |
| Renaissance | Gunpowder | 7796/8000 | 96 | 1460 AD | n/a |
| Renaissance | Metallurgy | 7724/8000 | 107 | 1570 AD | n/a |
| Renaissance | Military Engineering | 7751/8000 | 104 | 1540 AD | n/a |
| Renaissance | Banking | 7623/8000 | 116 | 1660 AD | 1397 AD |
| Renaissance | Printing Press | 7586/8000 | 117 | 1670 AD | 1440 AD |
| Industrial | Rifling | 7072/8000 | 141 | 1809 AD | n/a |
| Industrial | Military Science | 7302/8000 | 132 | 1776 AD | n/a |
| Industrial | Ballistics | 7143/8000 | 143 | 1816 AD | n/a |
| Industrial | Industrialization | 7438/8000 | 128 | 1761 AD | 1769 AD |
| Industrial | Railways | 6814/8000 | 151 | 1846 AD | 1825 AD |
| Machine Age | Replaceable Parts | 6162/8000 | 166 | 1886 AD | 1800 AD |
| Machine Age | Combustion | 6505/8000 | 159 | 1876 AD | 1876 AD |
| Machine Age | Steel | 6156/8000 | 164 | 1884 AD | 1856 AD |
| Machine Age | Flight | 5650/8000 | 174 | 1895 AD | 1903 AD |
| Machine Age | Electricity | 5377/8000 | 179 | 1901 AD | 1879 AD |
| World War | Mass Mobilisation | 4224/8000 | 194 | 1919 AD | n/a |
| World War | Rocketry | 3877/8000 | 198 | 1925 AD | 1926 AD |
| World War | Armoured Warfare | 3682/8000 | 202 | 1931 AD | 1916 AD |
| World War | Air Power | 3436/8000 | 205 | 1935 AD | n/a |
| World War | Electronics | 4883/8000 | 186 | 1909 AD | n/a |
| Modern | Mechanised Warfare | 2068/8000 | 222 | 1966 AD | n/a |
| Modern | Composite Armour | 1790/8000 | 227 | 1978 AD | n/a |
| Modern | Guided Missiles | 1696/8000 | 231 | 1988 AD | n/a |
| Modern | Jet Engines | 1624/8000 | 235 | 1998 AD | 1939 AD |
| Modern | Green Revolution | 2601/8000 | 215 | 1950 AD | n/a |
| Future | Autonomous Systems | 807/8000 | 254 | 2030 AD | n/a |
| Future | Robotics | 630/8000 | 256 | 2032 AD | n/a |
| Future | Fusion Power | 625/8000 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 995/8000 | 248 | 2022 AD | n/a |
| Future | Recursive AI | 572/8000 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 74.60 |
| 1 | 73.68 | 17.96 |
| 2 | 24.26 | 5.60 |
| 3 | 5.89 | 1.76 |
| 4 | 1.09 | 0.36 |
| 5 | 0.20 | 0.08 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 20261126 | 264 | 1538 | 12 | 4 | 2 |
| 1 | 20261127 | 313 | 1658 | 27 | 0 | 2 |
| 2 | 20261128 | 245 | 1157 | 19 | 7 | 2 |
| 3 | 20261129 | 255 | 1259 | 19 | 6 | 2 |
| 4 | 20261130 | 228 | 1107 | 23 | 5 | 4 |
| 5 | 20261131 | 263 | 1287 | 32 | 6 | 2 |
| 6 | 20261132 | 213 | 1118 | 12 | 4 | 0 |
| 7 | 20261133 | 241 | 1236 | 31 | 3 | 2 |
| 8 | 20261134 | 170 | 657 | 12 | 4 | 1 |
| 9 | 20261135 | 191 | 1042 | 19 | 6 | 1 |
| 10 | 20261136 | 174 | 716 | 12 | 3 | 3 |
| 11 | 20261137 | 289 | 1749 | 22 | 5 | 2 |
| 12 | 20261138 | 199 | 942 | 16 | 4 | 1 |
| 13 | 20261139 | 261 | 1398 | 39 | 3 | 1 |
| 14 | 20261140 | 197 | 949 | 23 | 2 | 3 |
| 15 | 20261141 | 249 | 1286 | 28 | 4 | 2 |
| 16 | 20261142 | 169 | 709 | 14 | 4 | 2 |
| 17 | 20261143 | 195 | 953 | 16 | 4 | 2 |
| 18 | 20261144 | 275 | 1428 | 34 | 3 | 3 |
| 19 | 20261145 | 221 | 1159 | 23 | 7 | 2 |
| 20 | 20261146 | 224 | 1100 | 24 | 1 | 3 |
| 21 | 20261147 | 217 | 987 | 19 | 15 | 2 |
| 22 | 20261148 | 196 | 761 | 19 | 3 | 1 |
| 23 | 20261149 | 189 | 1148 | 10 | 2 | 1 |
| 24 | 20261150 | 277 | 1610 | 30 | 5 | 2 |
| 25 | 20261151 | 185 | 830 | 12 | 5 | 2 |
| 26 | 20261152 | 236 | 1351 | 24 | 3 | 1 |
| 27 | 20261153 | 218 | 1015 | 27 | 8 | 1 |
| 28 | 20261154 | 231 | 1150 | 24 | 9 | 2 |
| 29 | 20261155 | 224 | 1226 | 16 | 4 | 1 |
| 30 | 20261156 | 279 | 1440 | 26 | 6 | 3 |
| 31 | 20261157 | 220 | 1315 | 31 | 4 | 2 |
| 32 | 20261158 | 250 | 1382 | 23 | 11 | 3 |
| 33 | 20261159 | 248 | 1426 | 33 | 2 | 2 |
| 34 | 20261160 | 288 | 1441 | 31 | 4 | 2 |
| 35 | 20261161 | 248 | 1259 | 24 | 3 | 4 |
| 36 | 20261162 | 226 | 928 | 21 | 0 | 2 |
| 37 | 20261163 | 152 | 449 | 6 | 7 | 1 |
| 38 | 20261164 | 196 | 756 | 14 | 5 | 1 |
| 39 | 20261165 | 263 | 1561 | 27 | 4 | 0 |
| 40 | 20261166 | 174 | 771 | 9 | 2 | 1 |
| 41 | 20261167 | 199 | 1037 | 13 | 3 | 2 |
| 42 | 20261168 | 211 | 1068 | 20 | 4 | 2 |
| 43 | 20261169 | 190 | 1025 | 13 | 1 | 1 |
| 44 | 20261170 | 341 | 1995 | 15 | 7 | 4 |
| 45 | 20261171 | 194 | 915 | 20 | 3 | 3 |
| 46 | 20261172 | 223 | 1074 | 18 | 6 | 1 |
| 47 | 20261173 | 221 | 1205 | 29 | 6 | 2 |
| 48 | 20261174 | 259 | 1167 | 21 | 0 | 0 |
| 49 | 20261175 | 247 | 1339 | 24 | 7 | 4 |
| 50 | 20261176 | 189 | 1100 | 16 | 8 | 1 |
| 51 | 20261177 | 223 | 1083 | 19 | 7 | 1 |
| 52 | 20261178 | 228 | 1065 | 25 | 4 | 3 |
| 53 | 20261179 | 203 | 1004 | 29 | 4 | 3 |
| 54 | 20261180 | 258 | 1517 | 19 | 7 | 4 |
| 55 | 20261181 | 203 | 1153 | 21 | 1 | 3 |
| 56 | 20261182 | 246 | 992 | 26 | 5 | 1 |
| 57 | 20261183 | 256 | 1213 | 29 | 3 | 2 |
| 58 | 20261184 | 214 | 985 | 22 | 2 | 2 |
| 59 | 20261185 | 263 | 1523 | 24 | 3 | 3 |
| 60 | 20261186 | 162 | 697 | 15 | 2 | 2 |
| 61 | 20261187 | 221 | 1035 | 18 | 2 | 4 |
| 62 | 20261188 | 216 | 1199 | 33 | 4 | 0 |
| 63 | 20261189 | 275 | 1618 | 24 | 9 | 3 |
| 64 | 20261190 | 166 | 773 | 14 | 5 | 0 |
| 65 | 20261191 | 280 | 1600 | 34 | 16 | 1 |
| 66 | 20261192 | 242 | 1531 | 30 | 0 | 3 |
| 67 | 20261193 | 239 | 1230 | 19 | 9 | 5 |
| 68 | 20261194 | 201 | 1117 | 11 | 5 | 2 |
| 69 | 20261195 | 191 | 820 | 8 | 9 | 3 |
| 70 | 20261196 | 197 | 875 | 18 | 4 | 2 |
| 71 | 20261197 | 209 | 1064 | 31 | 2 | 3 |
| 72 | 20261198 | 248 | 1490 | 38 | 1 | 3 |
| 73 | 20261199 | 247 | 1534 | 35 | 9 | 2 |
| 74 | 20261200 | 322 | 1965 | 35 | 3 | 2 |
| 75 | 20261201 | 158 | 623 | 14 | 7 | 1 |
| 76 | 20261202 | 221 | 1326 | 32 | 7 | 2 |
| 77 | 20261203 | 270 | 1342 | 17 | 6 | 3 |
| 78 | 20261204 | 283 | 1315 | 20 | 6 | 6 |
| 79 | 20261205 | 193 | 953 | 24 | 0 | 2 |
| 80 | 20261206 | 210 | 943 | 14 | 4 | 2 |
| 81 | 20261207 | 174 | 712 | 8 | 8 | 0 |
| 82 | 20261208 | 233 | 1431 | 24 | 5 | 1 |
| 83 | 20261209 | 231 | 1303 | 26 | 4 | 5 |
| 84 | 20261210 | 187 | 827 | 19 | 3 | 2 |
| 85 | 20261211 | 212 | 1121 | 30 | 5 | 3 |
| 86 | 20261212 | 225 | 1227 | 27 | 4 | 3 |
| 87 | 20261213 | 195 | 744 | 15 | 5 | 2 |
| 88 | 20261214 | 268 | 1161 | 13 | 10 | 4 |
| 89 | 20261215 | 251 | 1237 | 25 | 5 | 0 |
| 90 | 20261216 | 263 | 1343 | 30 | 4 | 4 |
| 91 | 20261217 | 249 | 1347 | 19 | 2 | 2 |
| 92 | 20261218 | 229 | 1077 | 23 | 2 | 2 |
| 93 | 20261219 | 232 | 1360 | 19 | 6 | 2 |
| 94 | 20261220 | 175 | 903 | 13 | 1 | 1 |
| 95 | 20261221 | 155 | 585 | 6 | 6 | 0 |
| 96 | 20261222 | 237 | 1226 | 23 | 3 | 1 |
| 97 | 20261223 | 251 | 1214 | 25 | 5 | 2 |
| 98 | 20261224 | 274 | 1792 | 28 | 4 | 0 |
| 99 | 20261225 | 209 | 1065 | 17 | 3 | 1 |
| 100 | 20261226 | 214 | 893 | 13 | 6 | 2 |
| 101 | 20261227 | 275 | 1155 | 22 | 2 | 0 |
| 102 | 20261228 | 279 | 1181 | 27 | 16 | 0 |
| 103 | 20261229 | 215 | 1064 | 22 | 9 | 3 |
| 104 | 20261230 | 256 | 1165 | 24 | 5 | 2 |
| 105 | 20261231 | 253 | 1222 | 20 | 6 | 4 |
| 106 | 20261232 | 220 | 1115 | 17 | 2 | 2 |
| 107 | 20261233 | 257 | 1036 | 22 | 7 | 3 |
| 108 | 20261234 | 168 | 767 | 15 | 3 | 1 |
| 109 | 20261235 | 195 | 788 | 8 | 3 | 1 |
| 110 | 20261236 | 226 | 1298 | 32 | 6 | 2 |
| 111 | 20261237 | 215 | 1040 | 21 | 4 | 4 |
| 112 | 20261238 | 245 | 1398 | 31 | 9 | 1 |
| 113 | 20261239 | 238 | 1638 | 13 | 5 | 1 |
| 114 | 20261240 | 236 | 1170 | 37 | 3 | 4 |
| 115 | 20261241 | 240 | 1012 | 14 | 7 | 2 |
| 116 | 20261242 | 195 | 888 | 18 | 3 | 2 |
| 117 | 20261243 | 359 | 2080 | 60 | 5 | 4 |
| 118 | 20261244 | 170 | 898 | 15 | 6 | 1 |
| 119 | 20261245 | 212 | 1086 | 16 | 5 | 1 |
| 120 | 20261246 | 208 | 1143 | 13 | 2 | 2 |
| 121 | 20261247 | 185 | 861 | 13 | 1 | 0 |
| 122 | 20261248 | 208 | 1083 | 25 | 2 | 3 |
| 123 | 20261249 | 220 | 1098 | 19 | 3 | 2 |
| 124 | 20261250 | 246 | 1357 | 27 | 4 | 2 |
| 125 | 20261251 | 186 | 873 | 9 | 5 | 2 |
| 126 | 20261252 | 184 | 711 | 13 | 6 | 2 |
| 127 | 20261253 | 154 | 739 | 11 | 4 | 0 |
| 128 | 20261254 | 194 | 995 | 23 | 7 | 3 |
| 129 | 20261255 | 197 | 813 | 21 | 9 | 3 |
| 130 | 20261256 | 254 | 1522 | 29 | 1 | 1 |
| 131 | 20261257 | 194 | 892 | 12 | 2 | 1 |
| 132 | 20261258 | 244 | 1102 | 17 | 1 | 3 |
| 133 | 20261259 | 232 | 1163 | 28 | 5 | 4 |
| 134 | 20261260 | 263 | 1316 | 29 | 5 | 3 |
| 135 | 20261261 | 241 | 1173 | 29 | 14 | 4 |
| 136 | 20261262 | 202 | 864 | 21 | 6 | 2 |
| 137 | 20261263 | 227 | 1351 | 25 | 3 | 3 |
| 138 | 20261264 | 273 | 943 | 17 | 4 | 1 |
| 139 | 20261265 | 218 | 1037 | 27 | 7 | 4 |
| 140 | 20261266 | 200 | 1085 | 26 | 4 | 2 |
| 141 | 20261267 | 196 | 963 | 14 | 8 | 1 |
| 142 | 20261268 | 220 | 1071 | 16 | 11 | 4 |
| 143 | 20261269 | 357 | 2409 | 33 | 9 | 3 |
| 144 | 20261270 | 309 | 1546 | 27 | 5 | 4 |
| 145 | 20261271 | 192 | 944 | 18 | 3 | 1 |
| 146 | 20261272 | 220 | 1004 | 33 | 4 | 3 |
| 147 | 20261273 | 231 | 1216 | 26 | 4 | 2 |
| 148 | 20261274 | 317 | 1893 | 27 | 3 | 1 |
| 149 | 20261275 | 220 | 1195 | 27 | 6 | 3 |
| 150 | 20261276 | 236 | 1147 | 11 | 9 | 3 |
| 151 | 20261277 | 236 | 1397 | 14 | 6 | 5 |
| 152 | 20261278 | 168 | 628 | 19 | 5 | 2 |
| 153 | 20261279 | 200 | 967 | 19 | 6 | 1 |
| 154 | 20261280 | 222 | 1206 | 31 | 1 | 2 |
| 155 | 20261281 | 231 | 1157 | 31 | 4 | 4 |
| 156 | 20261282 | 177 | 1052 | 27 | 2 | 0 |
| 157 | 20261283 | 228 | 1063 | 18 | 3 | 2 |
| 158 | 20261284 | 243 | 1284 | 27 | 9 | 3 |
| 159 | 20261285 | 225 | 1426 | 22 | 6 | 4 |
| 160 | 20261286 | 213 | 1075 | 22 | 7 | 1 |
| 161 | 20261287 | 280 | 1578 | 40 | 2 | 3 |
| 162 | 20261288 | 205 | 1303 | 15 | 4 | 3 |
| 163 | 20261289 | 231 | 1146 | 28 | 2 | 4 |
| 164 | 20261290 | 217 | 897 | 22 | 3 | 3 |
| 165 | 20261291 | 236 | 1277 | 25 | 6 | 4 |
| 166 | 20261292 | 210 | 1059 | 15 | 8 | 1 |
| 167 | 20261293 | 291 | 2140 | 19 | 14 | 1 |
| 168 | 20261294 | 215 | 1085 | 26 | 2 | 2 |
| 169 | 20261295 | 283 | 1615 | 38 | 5 | 4 |
| 170 | 20261296 | 327 | 2042 | 46 | 3 | 3 |
| 171 | 20261297 | 191 | 825 | 15 | 7 | 0 |
| 172 | 20261298 | 233 | 1431 | 30 | 2 | 2 |
| 173 | 20261299 | 286 | 1643 | 40 | 1 | 2 |
| 174 | 20261300 | 198 | 963 | 18 | 4 | 2 |
| 175 | 20261301 | 205 | 975 | 18 | 4 | 1 |
| 176 | 20261302 | 273 | 1369 | 25 | 4 | 2 |
| 177 | 20261303 | 177 | 824 | 14 | 8 | 3 |
| 178 | 20261304 | 281 | 1467 | 19 | 5 | 4 |
| 179 | 20261305 | 233 | 1406 | 37 | 6 | 1 |
| 180 | 20261306 | 172 | 808 | 13 | 4 | 0 |
| 181 | 20261307 | 318 | 1790 | 41 | 7 | 3 |
| 182 | 20261308 | 239 | 1324 | 35 | 1 | 3 |
| 183 | 20261309 | 230 | 1233 | 25 | 2 | 2 |
| 184 | 20261310 | 245 | 1553 | 43 | 5 | 4 |
| 185 | 20261311 | 202 | 852 | 9 | 11 | 2 |
| 186 | 20261312 | 278 | 1642 | 21 | 6 | 3 |
| 187 | 20261313 | 242 | 1077 | 21 | 3 | 0 |
| 188 | 20261314 | 242 | 1225 | 20 | 4 | 3 |
| 189 | 20261315 | 234 | 1029 | 19 | 6 | 4 |
| 190 | 20261316 | 221 | 1076 | 22 | 5 | 3 |
| 191 | 20261317 | 268 | 1283 | 34 | 5 | 5 |
| 192 | 20261318 | 319 | 1941 | 55 | 3 | 1 |
| 193 | 20261319 | 194 | 880 | 13 | 3 | 3 |
| 194 | 20261320 | 246 | 1293 | 19 | 11 | 1 |
| 195 | 20261321 | 197 | 653 | 10 | 4 | 1 |
| 196 | 20261322 | 259 | 1359 | 33 | 3 | 2 |
| 197 | 20261323 | 262 | 1257 | 27 | 2 | 2 |
| 198 | 20261324 | 173 | 789 | 11 | 4 | 1 |
| 199 | 20261325 | 246 | 1466 | 28 | 4 | 2 |
| 200 | 20261326 | 196 | 1099 | 17 | 7 | 0 |
| 201 | 20261327 | 218 | 1117 | 17 | 0 | 1 |
| 202 | 20261328 | 275 | 1506 | 33 | 6 | 3 |
| 203 | 20261329 | 395 | 2853 | 46 | 8 | 4 |
| 204 | 20261330 | 237 | 1247 | 29 | 1 | 2 |
| 205 | 20261331 | 174 | 739 | 13 | 3 | 2 |
| 206 | 20261332 | 214 | 855 | 9 | 4 | 0 |
| 207 | 20261333 | 209 | 1081 | 18 | 3 | 2 |
| 208 | 20261334 | 263 | 1735 | 30 | 6 | 4 |
| 209 | 20261335 | 259 | 1580 | 29 | 3 | 2 |
| 210 | 20261336 | 186 | 1035 | 16 | 1 | 3 |
| 211 | 20261337 | 173 | 711 | 11 | 5 | 1 |
| 212 | 20261338 | 217 | 1106 | 14 | 15 | 0 |
| 213 | 20261339 | 251 | 1486 | 24 | 8 | 3 |
| 214 | 20261340 | 223 | 1046 | 7 | 3 | 1 |
| 215 | 20261341 | 206 | 1049 | 17 | 2 | 2 |
| 216 | 20261342 | 174 | 713 | 11 | 2 | 2 |
| 217 | 20261343 | 208 | 1018 | 17 | 5 | 1 |
| 218 | 20261344 | 212 | 1163 | 16 | 6 | 0 |
| 219 | 20261345 | 228 | 1163 | 22 | 6 | 3 |
| 220 | 20261346 | 253 | 1274 | 27 | 6 | 3 |
| 221 | 20261347 | 247 | 1608 | 25 | 2 | 1 |
| 222 | 20261348 | 163 | 752 | 7 | 4 | 3 |
| 223 | 20261349 | 253 | 1410 | 21 | 3 | 3 |
| 224 | 20261350 | 228 | 1001 | 20 | 4 | 5 |
| 225 | 20261351 | 186 | 863 | 16 | 3 | 0 |
| 226 | 20261352 | 250 | 1192 | 24 | 3 | 2 |
| 227 | 20261353 | 234 | 1301 | 14 | 9 | 2 |
| 228 | 20261354 | 181 | 697 | 21 | 3 | 3 |
| 229 | 20261355 | 300 | 1943 | 18 | 3 | 4 |
| 230 | 20261356 | 223 | 1193 | 20 | 7 | 3 |
| 231 | 20261357 | 157 | 716 | 12 | 7 | 1 |
| 232 | 20261358 | 158 | 682 | 9 | 2 | 1 |
| 233 | 20261359 | 296 | 1760 | 41 | 2 | 4 |
| 234 | 20261360 | 183 | 612 | 19 | 3 | 3 |
| 235 | 20261361 | 269 | 1284 | 23 | 6 | 3 |
| 236 | 20261362 | 274 | 1661 | 26 | 0 | 2 |
| 237 | 20261363 | 231 | 1493 | 27 | 4 | 4 |
| 238 | 20261364 | 216 | 1135 | 32 | 5 | 1 |
| 239 | 20261365 | 289 | 1836 | 24 | 4 | 3 |
| 240 | 20261366 | 190 | 933 | 8 | 4 | 1 |
| 241 | 20261367 | 223 | 1130 | 14 | 3 | 3 |
| 242 | 20261368 | 249 | 1384 | 27 | 3 | 0 |
| 243 | 20261369 | 182 | 933 | 16 | 10 | 3 |
| 244 | 20261370 | 207 | 1054 | 29 | 6 | 4 |
| 245 | 20261371 | 216 | 1168 | 29 | 1 | 2 |
| 246 | 20261372 | 227 | 1144 | 6 | 5 | 3 |
| 247 | 20261373 | 199 | 952 | 12 | 6 | 0 |
| 248 | 20261374 | 252 | 1406 | 25 | 6 | 0 |
| 249 | 20261375 | 231 | 1357 | 19 | 4 | 2 |
| 250 | 20261376 | 156 | 684 | 8 | 4 | 1 |
| 251 | 20261377 | 216 | 938 | 12 | 6 | 3 |
| 252 | 20261378 | 220 | 1160 | 28 | 3 | 3 |
| 253 | 20261379 | 204 | 934 | 14 | 5 | 2 |
| 254 | 20261380 | 263 | 1461 | 25 | 6 | 3 |
| 255 | 20261381 | 236 | 940 | 23 | 3 | 2 |
| 256 | 20261382 | 236 | 1004 | 18 | 7 | 3 |
| 257 | 20261383 | 274 | 1544 | 32 | 3 | 4 |
| 258 | 20261384 | 227 | 1146 | 29 | 1 | 3 |
| 259 | 20261385 | 187 | 810 | 14 | 2 | 2 |
| 260 | 20261386 | 201 | 946 | 18 | 8 | 3 |
| 261 | 20261387 | 209 | 815 | 23 | 6 | 3 |
| 262 | 20261388 | 222 | 1150 | 18 | 13 | 1 |
| 263 | 20261389 | 208 | 1307 | 21 | 2 | 0 |
| 264 | 20261390 | 189 | 740 | 15 | 12 | 2 |
| 265 | 20261391 | 254 | 1237 | 35 | 7 | 4 |
| 266 | 20261392 | 240 | 995 | 17 | 7 | 5 |
| 267 | 20261393 | 217 | 1014 | 20 | 12 | 2 |
| 268 | 20261394 | 218 | 1052 | 22 | 4 | 4 |
| 269 | 20261395 | 235 | 1252 | 20 | 6 | 3 |
| 270 | 20261396 | 288 | 1648 | 27 | 4 | 3 |
| 271 | 20261397 | 182 | 750 | 9 | 5 | 0 |
| 272 | 20261398 | 224 | 988 | 21 | 3 | 2 |
| 273 | 20261399 | 201 | 996 | 23 | 9 | 2 |
| 274 | 20261400 | 182 | 721 | 15 | 5 | 3 |
| 275 | 20261401 | 190 | 505 | 15 | 2 | 2 |
| 276 | 20261402 | 158 | 604 | 8 | 5 | 0 |
| 277 | 20261403 | 197 | 889 | 21 | 2 | 1 |
| 278 | 20261404 | 210 | 1086 | 23 | 5 | 1 |
| 279 | 20261405 | 232 | 1247 | 26 | 3 | 1 |
| 280 | 20261406 | 219 | 1188 | 25 | 13 | 3 |
| 281 | 20261407 | 244 | 1378 | 23 | 2 | 2 |
| 282 | 20261408 | 281 | 1565 | 42 | 7 | 3 |
| 283 | 20261409 | 187 | 736 | 14 | 8 | 2 |
| 284 | 20261410 | 224 | 1399 | 29 | 2 | 1 |
| 285 | 20261411 | 176 | 785 | 16 | 2 | 0 |
| 286 | 20261412 | 242 | 1625 | 32 | 2 | 2 |
| 287 | 20261413 | 264 | 1421 | 33 | 5 | 3 |
| 288 | 20261414 | 207 | 1067 | 35 | 4 | 2 |
| 289 | 20261415 | 225 | 1084 | 30 | 4 | 3 |
| 290 | 20261416 | 216 | 1198 | 20 | 3 | 1 |
| 291 | 20261417 | 212 | 987 | 18 | 4 | 1 |
| 292 | 20261418 | 239 | 1262 | 24 | 4 | 1 |
| 293 | 20261419 | 232 | 1128 | 26 | 5 | 1 |
| 294 | 20261420 | 212 | 889 | 26 | 4 | 4 |
| 295 | 20261421 | 223 | 1230 | 23 | 15 | 3 |
| 296 | 20261422 | 240 | 1275 | 16 | 4 | 2 |
| 297 | 20261423 | 226 | 1397 | 15 | 4 | 3 |
| 298 | 20261424 | 257 | 1408 | 24 | 6 | 1 |
| 299 | 20261425 | 257 | 1443 | 29 | 4 | 3 |
| 300 | 20261426 | 228 | 1337 | 20 | 4 | 2 |
| 301 | 20261427 | 206 | 1074 | 20 | 1 | 1 |
| 302 | 20261428 | 197 | 1060 | 17 | 3 | 1 |
| 303 | 20261429 | 210 | 959 | 15 | 2 | 2 |
| 304 | 20261430 | 204 | 1179 | 12 | 6 | 1 |
| 305 | 20261431 | 266 | 1182 | 27 | 6 | 4 |
| 306 | 20261432 | 185 | 802 | 16 | 6 | 1 |
| 307 | 20261433 | 276 | 1674 | 32 | 7 | 2 |
| 308 | 20261434 | 187 | 671 | 17 | 6 | 1 |
| 309 | 20261435 | 230 | 1037 | 19 | 9 | 3 |
| 310 | 20261436 | 233 | 1498 | 38 | 3 | 4 |
| 311 | 20261437 | 189 | 774 | 14 | 7 | 2 |
| 312 | 20261438 | 190 | 949 | 15 | 6 | 0 |
| 313 | 20261439 | 195 | 1108 | 21 | 3 | 3 |
| 314 | 20261440 | 187 | 781 | 24 | 5 | 2 |
| 315 | 20261441 | 214 | 1297 | 15 | 4 | 1 |
| 316 | 20261442 | 196 | 824 | 19 | 5 | 4 |
| 317 | 20261443 | 229 | 1348 | 14 | 6 | 3 |
| 318 | 20261444 | 228 | 1251 | 22 | 5 | 2 |
| 319 | 20261445 | 187 | 850 | 19 | 6 | 1 |
| 320 | 20261446 | 231 | 1398 | 27 | 6 | 3 |
| 321 | 20261447 | 204 | 1085 | 13 | 5 | 0 |
| 322 | 20261448 | 252 | 1360 | 29 | 9 | 4 |
| 323 | 20261449 | 180 | 914 | 16 | 5 | 1 |
| 324 | 20261450 | 217 | 1046 | 26 | 5 | 2 |
| 325 | 20261451 | 270 | 1389 | 36 | 9 | 2 |
| 326 | 20261452 | 221 | 1412 | 17 | 6 | 3 |
| 327 | 20261453 | 241 | 1197 | 28 | 3 | 3 |
| 328 | 20261454 | 254 | 1640 | 29 | 4 | 3 |
| 329 | 20261455 | 280 | 1619 | 21 | 7 | 3 |
| 330 | 20261456 | 175 | 857 | 22 | 5 | 0 |
| 331 | 20261457 | 238 | 1056 | 15 | 4 | 2 |
| 332 | 20261458 | 238 | 1416 | 23 | 3 | 3 |
| 333 | 20261459 | 256 | 1706 | 25 | 7 | 2 |
| 334 | 20261460 | 221 | 1378 | 19 | 3 | 2 |
| 335 | 20261461 | 235 | 1363 | 20 | 4 | 3 |
| 336 | 20261462 | 236 | 1275 | 30 | 3 | 2 |
| 337 | 20261463 | 233 | 1286 | 18 | 3 | 2 |
| 338 | 20261464 | 198 | 955 | 15 | 2 | 0 |
| 339 | 20261465 | 176 | 688 | 16 | 7 | 2 |
| 340 | 20261466 | 201 | 1026 | 17 | 7 | 1 |
| 341 | 20261467 | 219 | 1258 | 26 | 5 | 2 |
| 342 | 20261468 | 217 | 941 | 21 | 9 | 3 |
| 343 | 20261469 | 275 | 1615 | 20 | 4 | 3 |
| 344 | 20261470 | 215 | 960 | 30 | 2 | 4 |
| 345 | 20261471 | 165 | 682 | 11 | 9 | 1 |
| 346 | 20261472 | 238 | 1136 | 23 | 3 | 2 |
| 347 | 20261473 | 204 | 1116 | 12 | 2 | 2 |
| 348 | 20261474 | 269 | 1481 | 25 | 7 | 3 |
| 349 | 20261475 | 185 | 1012 | 15 | 3 | 2 |
| 350 | 20261476 | 232 | 1133 | 19 | 13 | 1 |
| 351 | 20261477 | 202 | 897 | 14 | 3 | 1 |
| 352 | 20261478 | 253 | 1466 | 20 | 14 | 3 |
| 353 | 20261479 | 194 | 890 | 14 | 7 | 1 |
| 354 | 20261480 | 269 | 1347 | 30 | 2 | 4 |
| 355 | 20261481 | 191 | 890 | 12 | 4 | 2 |
| 356 | 20261482 | 242 | 1268 | 24 | 4 | 1 |
| 357 | 20261483 | 215 | 971 | 18 | 5 | 0 |
| 358 | 20261484 | 265 | 1177 | 23 | 3 | 1 |
| 359 | 20261485 | 339 | 2010 | 40 | 3 | 4 |
| 360 | 20261486 | 213 | 1000 | 26 | 2 | 4 |
| 361 | 20261487 | 269 | 1420 | 23 | 3 | 4 |
| 362 | 20261488 | 306 | 1713 | 39 | 4 | 5 |
| 363 | 20261489 | 246 | 1064 | 26 | 6 | 1 |
| 364 | 20261490 | 299 | 1339 | 19 | 11 | 3 |
| 365 | 20261491 | 221 | 1034 | 17 | 2 | 3 |
| 366 | 20261492 | 168 | 628 | 15 | 6 | 2 |
| 367 | 20261493 | 213 | 906 | 22 | 7 | 1 |
| 368 | 20261494 | 223 | 1247 | 28 | 8 | 2 |
| 369 | 20261495 | 214 | 936 | 13 | 9 | 1 |
| 370 | 20261496 | 220 | 1036 | 28 | 2 | 2 |
| 371 | 20261497 | 196 | 913 | 10 | 5 | 1 |
| 372 | 20261498 | 238 | 1108 | 23 | 3 | 2 |
| 373 | 20261499 | 291 | 1583 | 36 | 6 | 2 |
| 374 | 20261500 | 224 | 1033 | 18 | 5 | 1 |
| 375 | 20261501 | 207 | 1054 | 19 | 1 | 1 |
| 376 | 20261502 | 208 | 1028 | 18 | 2 | 1 |
| 377 | 20261503 | 222 | 967 | 16 | 6 | 0 |
| 378 | 20261504 | 254 | 1473 | 24 | 13 | 0 |
| 379 | 20261505 | 187 | 800 | 11 | 2 | 0 |
| 380 | 20261506 | 271 | 1609 | 55 | 9 | 5 |
| 381 | 20261507 | 203 | 1002 | 20 | 5 | 3 |
| 382 | 20261508 | 228 | 1223 | 28 | 5 | 2 |
| 383 | 20261509 | 195 | 1062 | 18 | 3 | 1 |
| 384 | 20261510 | 264 | 1349 | 23 | 5 | 1 |
| 385 | 20261511 | 178 | 806 | 21 | 3 | 3 |
| 386 | 20261512 | 217 | 878 | 13 | 6 | 0 |
| 387 | 20261513 | 206 | 1098 | 28 | 1 | 1 |
| 388 | 20261514 | 218 | 1104 | 13 | 2 | 4 |
| 389 | 20261515 | 229 | 1183 | 15 | 4 | 5 |
| 390 | 20261516 | 186 | 1079 | 13 | 6 | 0 |
| 391 | 20261517 | 168 | 613 | 14 | 3 | 3 |
| 392 | 20261518 | 252 | 1410 | 24 | 5 | 2 |
| 393 | 20261519 | 247 | 1166 | 21 | 3 | 3 |
| 394 | 20261520 | 215 | 1014 | 27 | 3 | 2 |
| 395 | 20261521 | 258 | 1420 | 17 | 3 | 3 |
| 396 | 20261522 | 212 | 1237 | 14 | 9 | 0 |
| 397 | 20261523 | 224 | 1025 | 17 | 4 | 1 |
| 398 | 20261524 | 203 | 763 | 25 | 8 | 2 |
| 399 | 20261525 | 201 | 1126 | 26 | 3 | 3 |
| 400 | 20261526 | 271 | 1352 | 20 | 2 | 3 |
| 401 | 20261527 | 194 | 991 | 25 | 0 | 2 |
| 402 | 20261528 | 269 | 1410 | 14 | 5 | 1 |
| 403 | 20261529 | 205 | 854 | 23 | 10 | 2 |
| 404 | 20261530 | 276 | 1485 | 18 | 8 | 3 |
| 405 | 20261531 | 152 | 558 | 11 | 7 | 1 |
| 406 | 20261532 | 201 | 824 | 16 | 5 | 3 |
| 407 | 20261533 | 243 | 1224 | 29 | 4 | 2 |
| 408 | 20261534 | 248 | 1277 | 24 | 14 | 3 |
| 409 | 20261535 | 212 | 1268 | 33 | 3 | 1 |
| 410 | 20261536 | 232 | 1159 | 31 | 3 | 1 |
| 411 | 20261537 | 178 | 710 | 13 | 3 | 0 |
| 412 | 20261538 | 289 | 1780 | 23 | 6 | 3 |
| 413 | 20261539 | 235 | 1053 | 32 | 6 | 3 |
| 414 | 20261540 | 313 | 1955 | 25 | 3 | 3 |
| 415 | 20261541 | 208 | 1401 | 14 | 2 | 0 |
| 416 | 20261542 | 225 | 1253 | 22 | 6 | 3 |
| 417 | 20261543 | 320 | 1626 | 25 | 2 | 3 |
| 418 | 20261544 | 175 | 944 | 13 | 8 | 2 |
| 419 | 20261545 | 205 | 822 | 14 | 2 | 1 |
| 420 | 20261546 | 222 | 1320 | 29 | 8 | 2 |
| 421 | 20261547 | 193 | 1166 | 14 | 2 | 2 |
| 422 | 20261548 | 182 | 707 | 15 | 2 | 1 |
| 423 | 20261549 | 241 | 1510 | 34 | 1 | 2 |
| 424 | 20261550 | 206 | 1092 | 19 | 9 | 3 |
| 425 | 20261551 | 220 | 1048 | 16 | 3 | 3 |
| 426 | 20261552 | 268 | 1734 | 27 | 6 | 1 |
| 427 | 20261553 | 304 | 2117 | 29 | 2 | 3 |
| 428 | 20261554 | 201 | 1044 | 20 | 8 | 1 |
| 429 | 20261555 | 183 | 780 | 12 | 3 | 2 |
| 430 | 20261556 | 232 | 915 | 22 | 6 | 3 |
| 431 | 20261557 | 243 | 1210 | 27 | 8 | 1 |
| 432 | 20261558 | 204 | 993 | 22 | 5 | 2 |
| 433 | 20261559 | 165 | 726 | 19 | 5 | 4 |
| 434 | 20261560 | 205 | 1083 | 15 | 5 | 2 |
| 435 | 20261561 | 230 | 1129 | 28 | 9 | 3 |
| 436 | 20261562 | 300 | 1431 | 18 | 5 | 2 |
| 437 | 20261563 | 232 | 1167 | 35 | 9 | 1 |
| 438 | 20261564 | 217 | 1109 | 25 | 5 | 2 |
| 439 | 20261565 | 232 | 1209 | 17 | 4 | 2 |
| 440 | 20261566 | 194 | 960 | 13 | 3 | 3 |
| 441 | 20261567 | 216 | 1105 | 24 | 6 | 1 |
| 442 | 20261568 | 234 | 1350 | 9 | 8 | 2 |
| 443 | 20261569 | 250 | 1325 | 14 | 8 | 5 |
| 444 | 20261570 | 188 | 867 | 18 | 8 | 3 |
| 445 | 20261571 | 206 | 1006 | 16 | 2 | 1 |
| 446 | 20261572 | 225 | 1110 | 19 | 4 | 1 |
| 447 | 20261573 | 170 | 606 | 12 | 5 | 2 |
| 448 | 20261574 | 174 | 836 | 17 | 4 | 1 |
| 449 | 20261575 | 254 | 1118 | 20 | 0 | 2 |
| 450 | 20261576 | 263 | 1332 | 24 | 5 | 3 |
| 451 | 20261577 | 306 | 1497 | 15 | 4 | 5 |
| 452 | 20261578 | 245 | 1464 | 17 | 2 | 3 |
| 453 | 20261579 | 285 | 1384 | 23 | 8 | 3 |
| 454 | 20261580 | 202 | 1067 | 13 | 5 | 2 |
| 455 | 20261581 | 202 | 1019 | 24 | 6 | 2 |
| 456 | 20261582 | 286 | 2178 | 34 | 3 | 2 |
| 457 | 20261583 | 231 | 1178 | 16 | 4 | 1 |
| 458 | 20261584 | 218 | 1191 | 28 | 4 | 2 |
| 459 | 20261585 | 184 | 992 | 18 | 4 | 2 |
| 460 | 20261586 | 290 | 1576 | 27 | 8 | 4 |
| 461 | 20261587 | 254 | 1138 | 23 | 4 | 1 |
| 462 | 20261588 | 254 | 1392 | 27 | 1 | 5 |
| 463 | 20261589 | 212 | 1201 | 19 | 3 | 3 |
| 464 | 20261590 | 235 | 1037 | 19 | 8 | 1 |
| 465 | 20261591 | 187 | 697 | 21 | 1 | 0 |
| 466 | 20261592 | 199 | 967 | 17 | 1 | 3 |
| 467 | 20261593 | 232 | 1439 | 28 | 4 | 0 |
| 468 | 20261594 | 239 | 1483 | 25 | 8 | 3 |
| 469 | 20261595 | 246 | 1358 | 20 | 4 | 2 |
| 470 | 20261596 | 217 | 1243 | 18 | 5 | 1 |
| 471 | 20261597 | 230 | 1003 | 22 | 9 | 3 |
| 472 | 20261598 | 238 | 1280 | 29 | 9 | 3 |
| 473 | 20261599 | 313 | 1791 | 49 | 7 | 5 |
| 474 | 20261600 | 206 | 993 | 23 | 4 | 2 |
| 475 | 20261601 | 203 | 1097 | 20 | 2 | 4 |
| 476 | 20261602 | 180 | 892 | 19 | 9 | 1 |
| 477 | 20261603 | 224 | 1079 | 25 | 5 | 3 |
| 478 | 20261604 | 186 | 967 | 14 | 5 | 1 |
| 479 | 20261605 | 227 | 1328 | 23 | 2 | 3 |
| 480 | 20261606 | 259 | 1332 | 24 | 10 | 1 |
| 481 | 20261607 | 226 | 815 | 16 | 14 | 0 |
| 482 | 20261608 | 228 | 1201 | 27 | 6 | 1 |
| 483 | 20261609 | 212 | 925 | 16 | 4 | 2 |
| 484 | 20261610 | 171 | 854 | 9 | 6 | 0 |
| 485 | 20261611 | 236 | 1255 | 22 | 7 | 3 |
| 486 | 20261612 | 191 | 805 | 12 | 7 | 1 |
| 487 | 20261613 | 292 | 1581 | 41 | 10 | 3 |
| 488 | 20261614 | 324 | 2351 | 13 | 7 | 1 |
| 489 | 20261615 | 256 | 1313 | 24 | 5 | 3 |
| 490 | 20261616 | 312 | 1800 | 40 | 9 | 2 |
| 491 | 20261617 | 194 | 916 | 12 | 8 | 2 |
| 492 | 20261618 | 251 | 1399 | 15 | 14 | 3 |
| 493 | 20261619 | 286 | 1706 | 45 | 10 | 5 |
| 494 | 20261620 | 228 | 1193 | 31 | 1 | 4 |
| 495 | 20261621 | 213 | 1143 | 21 | 3 | 3 |
| 496 | 20261622 | 276 | 1716 | 44 | 3 | 4 |
| 497 | 20261623 | 224 | 1103 | 21 | 1 | 0 |
| 498 | 20261624 | 249 | 1319 | 28 | 7 | 1 |
| 499 | 20261625 | 251 | 1298 | 25 | 4 | 0 |
| 500 | 20261626 | 192 | 853 | 26 | 2 | 1 |
| 501 | 20261627 | 295 | 1470 | 38 | 6 | 4 |
| 502 | 20261628 | 216 | 939 | 15 | 1 | 4 |
| 503 | 20261629 | 193 | 850 | 24 | 5 | 3 |
| 504 | 20261630 | 263 | 1632 | 22 | 6 | 3 |
| 505 | 20261631 | 193 | 1054 | 24 | 3 | 2 |
| 506 | 20261632 | 239 | 1293 | 35 | 4 | 1 |
| 507 | 20261633 | 299 | 1772 | 36 | 5 | 3 |
| 508 | 20261634 | 240 | 1254 | 20 | 2 | 3 |
| 509 | 20261635 | 216 | 934 | 21 | 5 | 3 |
| 510 | 20261636 | 324 | 1638 | 35 | 6 | 3 |
| 511 | 20261637 | 253 | 1336 | 29 | 5 | 2 |
| 512 | 20261638 | 201 | 1175 | 11 | 6 | 1 |
| 513 | 20261639 | 288 | 1705 | 41 | 14 | 4 |
| 514 | 20261640 | 309 | 1914 | 28 | 8 | 4 |
| 515 | 20261641 | 233 | 1562 | 20 | 3 | 2 |
| 516 | 20261642 | 195 | 1348 | 31 | 1 | 2 |
| 517 | 20261643 | 305 | 1891 | 26 | 6 | 5 |
| 518 | 20261644 | 203 | 1062 | 24 | 9 | 2 |
| 519 | 20261645 | 179 | 852 | 13 | 3 | 1 |
| 520 | 20261646 | 270 | 1500 | 30 | 4 | 3 |
| 521 | 20261647 | 179 | 924 | 18 | 7 | 2 |
| 522 | 20261648 | 221 | 1050 | 23 | 6 | 3 |
| 523 | 20261649 | 221 | 1181 | 13 | 13 | 0 |
| 524 | 20261650 | 179 | 901 | 21 | 2 | 1 |
| 525 | 20261651 | 225 | 1149 | 35 | 9 | 3 |
| 526 | 20261652 | 198 | 1045 | 28 | 3 | 2 |
| 527 | 20261653 | 224 | 1172 | 22 | 4 | 2 |
| 528 | 20261654 | 246 | 1034 | 18 | 2 | 3 |
| 529 | 20261655 | 253 | 1364 | 35 | 4 | 3 |
| 530 | 20261656 | 193 | 977 | 15 | 1 | 3 |
| 531 | 20261657 | 246 | 1197 | 16 | 7 | 1 |
| 532 | 20261658 | 238 | 1035 | 27 | 1 | 2 |
| 533 | 20261659 | 200 | 802 | 24 | 7 | 2 |
| 534 | 20261660 | 257 | 1268 | 19 | 2 | 2 |
| 535 | 20261661 | 205 | 697 | 21 | 4 | 1 |
| 536 | 20261662 | 190 | 757 | 18 | 5 | 0 |
| 537 | 20261663 | 189 | 965 | 20 | 4 | 2 |
| 538 | 20261664 | 250 | 1145 | 18 | 6 | 1 |
| 539 | 20261665 | 256 | 840 | 13 | 5 | 0 |
| 540 | 20261666 | 262 | 1433 | 33 | 5 | 3 |
| 541 | 20261667 | 235 | 995 | 21 | 8 | 3 |
| 542 | 20261668 | 246 | 1328 | 26 | 2 | 1 |
| 543 | 20261669 | 245 | 1555 | 27 | 2 | 3 |
| 544 | 20261670 | 251 | 1383 | 26 | 4 | 3 |
| 545 | 20261671 | 310 | 1912 | 29 | 5 | 4 |
| 546 | 20261672 | 274 | 1454 | 39 | 4 | 2 |
| 547 | 20261673 | 223 | 1339 | 24 | 4 | 3 |
| 548 | 20261674 | 178 | 776 | 19 | 4 | 1 |
| 549 | 20261675 | 227 | 1082 | 22 | 3 | 3 |
| 550 | 20261676 | 252 | 1378 | 14 | 4 | 2 |
| 551 | 20261677 | 186 | 896 | 19 | 4 | 0 |
| 552 | 20261678 | 170 | 893 | 20 | 5 | 2 |
| 553 | 20261679 | 185 | 830 | 12 | 5 | 3 |
| 554 | 20261680 | 211 | 802 | 18 | 5 | 2 |
| 555 | 20261681 | 250 | 1410 | 25 | 13 | 3 |
| 556 | 20261682 | 203 | 957 | 15 | 4 | 2 |
| 557 | 20261683 | 206 | 1091 | 17 | 6 | 1 |
| 558 | 20261684 | 280 | 1676 | 31 | 3 | 1 |
| 559 | 20261685 | 222 | 1365 | 27 | 2 | 1 |
| 560 | 20261686 | 190 | 929 | 21 | 2 | 2 |
| 561 | 20261687 | 181 | 979 | 27 | 2 | 2 |
| 562 | 20261688 | 312 | 1883 | 35 | 4 | 4 |
| 563 | 20261689 | 200 | 1059 | 23 | 2 | 1 |
| 564 | 20261690 | 246 | 1166 | 17 | 10 | 2 |
| 565 | 20261691 | 256 | 1258 | 29 | 7 | 5 |
| 566 | 20261692 | 275 | 1709 | 28 | 1 | 2 |
| 567 | 20261693 | 258 | 1497 | 36 | 5 | 4 |
| 568 | 20261694 | 332 | 1813 | 27 | 8 | 4 |
| 569 | 20261695 | 176 | 781 | 20 | 6 | 3 |
| 570 | 20261696 | 178 | 691 | 11 | 4 | 3 |
| 571 | 20261697 | 242 | 1269 | 17 | 4 | 4 |
| 572 | 20261698 | 197 | 773 | 21 | 3 | 0 |
| 573 | 20261699 | 205 | 992 | 29 | 4 | 2 |
| 574 | 20261700 | 188 | 869 | 26 | 6 | 1 |
| 575 | 20261701 | 261 | 1344 | 16 | 4 | 3 |
| 576 | 20261702 | 185 | 956 | 19 | 5 | 1 |
| 577 | 20261703 | 236 | 1262 | 18 | 1 | 2 |
| 578 | 20261704 | 358 | 1961 | 35 | 6 | 3 |
| 579 | 20261705 | 207 | 1193 | 28 | 2 | 0 |
| 580 | 20261706 | 208 | 891 | 17 | 2 | 4 |
| 581 | 20261707 | 186 | 741 | 18 | 4 | 2 |
| 582 | 20261708 | 209 | 1065 | 15 | 3 | 3 |
| 583 | 20261709 | 181 | 712 | 11 | 2 | 2 |
| 584 | 20261710 | 211 | 998 | 24 | 5 | 1 |
| 585 | 20261711 | 242 | 1327 | 28 | 10 | 4 |
| 586 | 20261712 | 290 | 1679 | 39 | 6 | 2 |
| 587 | 20261713 | 293 | 1730 | 37 | 7 | 1 |
| 588 | 20261714 | 240 | 1285 | 15 | 4 | 3 |
| 589 | 20261715 | 279 | 1333 | 27 | 7 | 4 |
| 590 | 20261716 | 247 | 976 | 31 | 3 | 4 |
| 591 | 20261717 | 207 | 1164 | 16 | 3 | 2 |
| 592 | 20261718 | 221 | 1059 | 24 | 6 | 2 |
| 593 | 20261719 | 207 | 1083 | 25 | 3 | 1 |
| 594 | 20261720 | 285 | 1607 | 33 | 3 | 2 |
| 595 | 20261721 | 214 | 1151 | 12 | 7 | 3 |
| 596 | 20261722 | 226 | 1091 | 28 | 3 | 3 |
| 597 | 20261723 | 221 | 1004 | 30 | 4 | 1 |
| 598 | 20261724 | 199 | 744 | 4 | 10 | 1 |
| 599 | 20261725 | 215 | 1115 | 17 | 9 | 1 |
| 600 | 20261726 | 182 | 907 | 21 | 4 | 0 |
| 601 | 20261727 | 183 | 746 | 13 | 2 | 0 |
| 602 | 20261728 | 216 | 1046 | 19 | 2 | 2 |
| 603 | 20261729 | 152 | 454 | 18 | 4 | 3 |
| 604 | 20261730 | 209 | 1010 | 18 | 1 | 2 |
| 605 | 20261731 | 235 | 1378 | 27 | 7 | 4 |
| 606 | 20261732 | 197 | 991 | 19 | 5 | 4 |
| 607 | 20261733 | 297 | 1722 | 35 | 4 | 2 |
| 608 | 20261734 | 202 | 905 | 17 | 4 | 2 |
| 609 | 20261735 | 220 | 1043 | 11 | 1 | 2 |
| 610 | 20261736 | 244 | 1296 | 18 | 7 | 4 |
| 611 | 20261737 | 211 | 1145 | 19 | 3 | 1 |
| 612 | 20261738 | 221 | 1181 | 27 | 6 | 2 |
| 613 | 20261739 | 316 | 1694 | 33 | 3 | 3 |
| 614 | 20261740 | 264 | 1484 | 30 | 3 | 1 |
| 615 | 20261741 | 223 | 925 | 22 | 4 | 3 |
| 616 | 20261742 | 189 | 820 | 17 | 0 | 3 |
| 617 | 20261743 | 236 | 1227 | 22 | 4 | 2 |
| 618 | 20261744 | 202 | 970 | 15 | 4 | 2 |
| 619 | 20261745 | 236 | 1347 | 31 | 3 | 2 |
| 620 | 20261746 | 179 | 838 | 12 | 5 | 0 |
| 621 | 20261747 | 240 | 939 | 12 | 13 | 3 |
| 622 | 20261748 | 218 | 997 | 30 | 5 | 1 |
| 623 | 20261749 | 273 | 1680 | 21 | 9 | 3 |
| 624 | 20261750 | 296 | 1703 | 34 | 2 | 1 |
| 625 | 20261751 | 220 | 991 | 22 | 6 | 1 |
| 626 | 20261752 | 253 | 1489 | 17 | 5 | 3 |
| 627 | 20261753 | 253 | 1261 | 29 | 6 | 5 |
| 628 | 20261754 | 243 | 921 | 8 | 2 | 4 |
| 629 | 20261755 | 201 | 1129 | 16 | 2 | 0 |
| 630 | 20261756 | 210 | 795 | 14 | 3 | 1 |
| 631 | 20261757 | 224 | 976 | 14 | 3 | 1 |
| 632 | 20261758 | 202 | 1159 | 19 | 6 | 4 |
| 633 | 20261759 | 224 | 1299 | 30 | 5 | 3 |
| 634 | 20261760 | 192 | 971 | 21 | 1 | 1 |
| 635 | 20261761 | 275 | 1591 | 38 | 5 | 3 |
| 636 | 20261762 | 204 | 718 | 16 | 3 | 4 |
| 637 | 20261763 | 232 | 1019 | 26 | 8 | 4 |
| 638 | 20261764 | 287 | 1507 | 24 | 3 | 1 |
| 639 | 20261765 | 274 | 1525 | 38 | 7 | 3 |
| 640 | 20261766 | 204 | 836 | 18 | 4 | 4 |
| 641 | 20261767 | 179 | 895 | 17 | 11 | 2 |
| 642 | 20261768 | 276 | 1841 | 32 | 6 | 4 |
| 643 | 20261769 | 211 | 930 | 16 | 5 | 1 |
| 644 | 20261770 | 226 | 1309 | 25 | 3 | 1 |
| 645 | 20261771 | 296 | 1589 | 24 | 11 | 4 |
| 646 | 20261772 | 188 | 1037 | 17 | 11 | 2 |
| 647 | 20261773 | 190 | 914 | 16 | 6 | 1 |
| 648 | 20261774 | 273 | 1236 | 35 | 7 | 4 |
| 649 | 20261775 | 262 | 1378 | 22 | 9 | 4 |
| 650 | 20261776 | 261 | 1096 | 18 | 6 | 1 |
| 651 | 20261777 | 266 | 1217 | 31 | 9 | 3 |
| 652 | 20261778 | 207 | 952 | 24 | 6 | 2 |
| 653 | 20261779 | 255 | 1250 | 21 | 7 | 1 |
| 654 | 20261780 | 283 | 1321 | 25 | 9 | 3 |
| 655 | 20261781 | 272 | 1360 | 25 | 8 | 4 |
| 656 | 20261782 | 188 | 1009 | 30 | 3 | 2 |
| 657 | 20261783 | 211 | 1195 | 28 | 9 | 2 |
| 658 | 20261784 | 184 | 930 | 26 | 3 | 2 |
| 659 | 20261785 | 188 | 704 | 14 | 2 | 2 |
| 660 | 20261786 | 263 | 1596 | 18 | 2 | 2 |
| 661 | 20261787 | 207 | 1010 | 19 | 2 | 1 |
| 662 | 20261788 | 207 | 1002 | 26 | 5 | 4 |
| 663 | 20261789 | 223 | 1106 | 26 | 11 | 3 |
| 664 | 20261790 | 165 | 521 | 11 | 2 | 0 |
| 665 | 20261791 | 199 | 996 | 25 | 4 | 1 |
| 666 | 20261792 | 218 | 1086 | 28 | 7 | 2 |
| 667 | 20261793 | 353 | 2076 | 27 | 9 | 4 |
| 668 | 20261794 | 337 | 1665 | 35 | 4 | 0 |
| 669 | 20261795 | 238 | 1023 | 18 | 9 | 3 |
| 670 | 20261796 | 198 | 1110 | 21 | 4 | 2 |
| 671 | 20261797 | 272 | 1651 | 18 | 5 | 2 |
| 672 | 20261798 | 191 | 939 | 15 | 1 | 2 |
| 673 | 20261799 | 193 | 796 | 16 | 5 | 0 |
| 674 | 20261800 | 225 | 1137 | 24 | 5 | 1 |
| 675 | 20261801 | 199 | 986 | 17 | 1 | 1 |
| 676 | 20261802 | 260 | 1380 | 43 | 9 | 5 |
| 677 | 20261803 | 238 | 1398 | 17 | 4 | 2 |
| 678 | 20261804 | 239 | 1337 | 29 | 7 | 2 |
| 679 | 20261805 | 318 | 1769 | 38 | 6 | 6 |
| 680 | 20261806 | 243 | 1235 | 30 | 3 | 4 |
| 681 | 20261807 | 343 | 1701 | 27 | 2 | 1 |
| 682 | 20261808 | 283 | 1464 | 25 | 9 | 1 |
| 683 | 20261809 | 287 | 1882 | 21 | 7 | 1 |
| 684 | 20261810 | 229 | 1098 | 24 | 5 | 5 |
| 685 | 20261811 | 191 | 883 | 14 | 3 | 1 |
| 686 | 20261812 | 200 | 765 | 17 | 9 | 3 |
| 687 | 20261813 | 218 | 1106 | 23 | 3 | 3 |
| 688 | 20261814 | 446 | 2067 | 42 | 10 | 3 |
| 689 | 20261815 | 220 | 1114 | 28 | 4 | 2 |
| 690 | 20261816 | 206 | 1029 | 17 | 5 | 0 |
| 691 | 20261817 | 157 | 587 | 6 | 3 | 0 |
| 692 | 20261818 | 294 | 1609 | 27 | 9 | 2 |
| 693 | 20261819 | 215 | 1052 | 24 | 3 | 2 |
| 694 | 20261820 | 192 | 963 | 21 | 6 | 3 |
| 695 | 20261821 | 250 | 1456 | 31 | 7 | 3 |
| 696 | 20261822 | 237 | 1101 | 26 | 5 | 2 |
| 697 | 20261823 | 233 | 1338 | 33 | 5 | 4 |
| 698 | 20261824 | 205 | 484 | 13 | 3 | 2 |
| 699 | 20261825 | 190 | 941 | 21 | 3 | 2 |
| 700 | 20261826 | 199 | 1197 | 24 | 2 | 2 |
| 701 | 20261827 | 220 | 1116 | 21 | 2 | 3 |
| 702 | 20261828 | 264 | 1506 | 24 | 4 | 4 |
| 703 | 20261829 | 216 | 633 | 14 | 6 | 3 |
| 704 | 20261830 | 301 | 1776 | 7 | 7 | 3 |
| 705 | 20261831 | 212 | 853 | 19 | 4 | 4 |
| 706 | 20261832 | 243 | 1155 | 34 | 2 | 1 |
| 707 | 20261833 | 291 | 1791 | 25 | 4 | 3 |
| 708 | 20261834 | 218 | 1107 | 26 | 6 | 2 |
| 709 | 20261835 | 208 | 857 | 19 | 2 | 0 |
| 710 | 20261836 | 242 | 1267 | 31 | 5 | 2 |
| 711 | 20261837 | 228 | 1158 | 25 | 0 | 1 |
| 712 | 20261838 | 292 | 1497 | 37 | 6 | 2 |
| 713 | 20261839 | 176 | 912 | 25 | 1 | 2 |
| 714 | 20261840 | 222 | 954 | 21 | 3 | 1 |
| 715 | 20261841 | 201 | 1107 | 13 | 5 | 1 |
| 716 | 20261842 | 226 | 1034 | 17 | 9 | 1 |
| 717 | 20261843 | 235 | 1207 | 12 | 3 | 2 |
| 718 | 20261844 | 197 | 908 | 22 | 3 | 2 |
| 719 | 20261845 | 247 | 1283 | 24 | 3 | 2 |
| 720 | 20261846 | 250 | 1339 | 28 | 5 | 4 |
| 721 | 20261847 | 221 | 1041 | 17 | 3 | 1 |
| 722 | 20261848 | 183 | 761 | 22 | 1 | 3 |
| 723 | 20261849 | 195 | 941 | 17 | 2 | 1 |
| 724 | 20261850 | 227 | 1215 | 30 | 7 | 2 |
| 725 | 20261851 | 247 | 1459 | 40 | 10 | 4 |
| 726 | 20261852 | 215 | 1127 | 37 | 6 | 2 |
| 727 | 20261853 | 181 | 1033 | 18 | 10 | 2 |
| 728 | 20261854 | 249 | 1246 | 24 | 5 | 0 |
| 729 | 20261855 | 231 | 1487 | 24 | 1 | 1 |
| 730 | 20261856 | 248 | 1288 | 34 | 5 | 3 |
| 731 | 20261857 | 197 | 996 | 12 | 4 | 1 |
| 732 | 20261858 | 200 | 1030 | 13 | 3 | 2 |
| 733 | 20261859 | 188 | 1122 | 13 | 1 | 2 |
| 734 | 20261860 | 134 | 367 | 5 | 9 | 1 |
| 735 | 20261861 | 207 | 1115 | 16 | 5 | 2 |
| 736 | 20261862 | 274 | 1497 | 8 | 7 | 3 |
| 737 | 20261863 | 163 | 865 | 12 | 3 | 2 |
| 738 | 20261864 | 218 | 1086 | 31 | 6 | 4 |
| 739 | 20261865 | 236 | 1173 | 22 | 4 | 2 |
| 740 | 20261866 | 229 | 1024 | 19 | 7 | 2 |
| 741 | 20261867 | 247 | 1241 | 16 | 6 | 1 |
| 742 | 20261868 | 235 | 1177 | 19 | 5 | 2 |
| 743 | 20261869 | 242 | 1147 | 27 | 5 | 2 |
| 744 | 20261870 | 177 | 924 | 14 | 4 | 2 |
| 745 | 20261871 | 172 | 833 | 10 | 3 | 1 |
| 746 | 20261872 | 215 | 1033 | 26 | 10 | 3 |
| 747 | 20261873 | 244 | 1272 | 29 | 4 | 3 |
| 748 | 20261874 | 200 | 1114 | 34 | 10 | 2 |
| 749 | 20261875 | 217 | 1217 | 29 | 6 | 2 |
| 750 | 20261876 | 275 | 1679 | 37 | 3 | 5 |
| 751 | 20261877 | 223 | 1165 | 28 | 3 | 4 |
| 752 | 20261878 | 231 | 1290 | 20 | 7 | 3 |
| 753 | 20261879 | 220 | 1354 | 23 | 6 | 4 |
| 754 | 20261880 | 219 | 1318 | 15 | 8 | 2 |
| 755 | 20261881 | 288 | 1456 | 25 | 6 | 3 |
| 756 | 20261882 | 221 | 1050 | 31 | 3 | 4 |
| 757 | 20261883 | 273 | 1526 | 42 | 6 | 3 |
| 758 | 20261884 | 205 | 956 | 20 | 5 | 2 |
| 759 | 20261885 | 272 | 1255 | 43 | 4 | 2 |
| 760 | 20261886 | 279 | 1316 | 31 | 7 | 3 |
| 761 | 20261887 | 358 | 1903 | 40 | 2 | 6 |
| 762 | 20261888 | 219 | 1042 | 18 | 0 | 2 |
| 763 | 20261889 | 291 | 1607 | 26 | 5 | 5 |
| 764 | 20261890 | 283 | 1176 | 30 | 4 | 2 |
| 765 | 20261891 | 273 | 1575 | 39 | 8 | 5 |
| 766 | 20261892 | 231 | 1023 | 21 | 4 | 4 |
| 767 | 20261893 | 244 | 1302 | 19 | 5 | 3 |
| 768 | 20261894 | 248 | 1232 | 29 | 7 | 1 |
| 769 | 20261895 | 190 | 779 | 14 | 2 | 1 |
| 770 | 20261896 | 217 | 1211 | 30 | 0 | 2 |
| 771 | 20261897 | 193 | 965 | 20 | 3 | 2 |
| 772 | 20261898 | 235 | 1197 | 27 | 3 | 2 |
| 773 | 20261899 | 180 | 838 | 16 | 3 | 2 |
| 774 | 20261900 | 282 | 1646 | 37 | 3 | 2 |
| 775 | 20261901 | 181 | 782 | 8 | 4 | 2 |
| 776 | 20261902 | 248 | 1465 | 20 | 15 | 1 |
| 777 | 20261903 | 233 | 1227 | 27 | 4 | 2 |
| 778 | 20261904 | 310 | 2233 | 33 | 6 | 3 |
| 779 | 20261905 | 244 | 1340 | 26 | 9 | 2 |
| 780 | 20261906 | 230 | 1476 | 22 | 6 | 2 |
| 781 | 20261907 | 241 | 1202 | 28 | 1 | 5 |
| 782 | 20261908 | 203 | 987 | 20 | 7 | 1 |
| 783 | 20261909 | 217 | 1159 | 28 | 3 | 3 |
| 784 | 20261910 | 201 | 1105 | 23 | 8 | 4 |
| 785 | 20261911 | 253 | 1331 | 28 | 5 | 5 |
| 786 | 20261912 | 342 | 1827 | 43 | 8 | 2 |
| 787 | 20261913 | 193 | 987 | 20 | 3 | 3 |
| 788 | 20261914 | 210 | 1056 | 16 | 4 | 1 |
| 789 | 20261915 | 267 | 1415 | 26 | 5 | 2 |
| 790 | 20261916 | 229 | 1355 | 29 | 4 | 3 |
| 791 | 20261917 | 180 | 900 | 16 | 7 | 2 |
| 792 | 20261918 | 202 | 911 | 20 | 3 | 1 |
| 793 | 20261919 | 253 | 1261 | 29 | 7 | 2 |
| 794 | 20261920 | 220 | 1214 | 20 | 2 | 1 |
| 795 | 20261921 | 160 | 571 | 16 | 8 | 1 |
| 796 | 20261922 | 163 | 652 | 15 | 2 | 1 |
| 797 | 20261923 | 242 | 1432 | 24 | 6 | 4 |
| 798 | 20261924 | 303 | 1831 | 40 | 4 | 4 |
| 799 | 20261925 | 196 | 809 | 16 | 3 | 2 |
| 800 | 20261926 | 218 | 1166 | 15 | 7 | 3 |
| 801 | 20261927 | 220 | 1255 | 33 | 1 | 1 |
| 802 | 20261928 | 245 | 1249 | 27 | 9 | 2 |
| 803 | 20261929 | 253 | 1420 | 13 | 3 | 2 |
| 804 | 20261930 | 179 | 751 | 16 | 2 | 1 |
| 805 | 20261931 | 281 | 1395 | 33 | 1 | 2 |
| 806 | 20261932 | 225 | 1115 | 25 | 8 | 2 |
| 807 | 20261933 | 212 | 1130 | 23 | 9 | 2 |
| 808 | 20261934 | 164 | 641 | 16 | 6 | 2 |
| 809 | 20261935 | 232 | 1066 | 17 | 1 | 2 |
| 810 | 20261936 | 214 | 1073 | 33 | 3 | 1 |
| 811 | 20261937 | 259 | 1308 | 18 | 3 | 4 |
| 812 | 20261938 | 210 | 1288 | 20 | 5 | 1 |
| 813 | 20261939 | 192 | 821 | 22 | 6 | 2 |
| 814 | 20261940 | 217 | 978 | 16 | 11 | 3 |
| 815 | 20261941 | 201 | 983 | 23 | 7 | 2 |
| 816 | 20261942 | 285 | 1615 | 30 | 3 | 3 |
| 817 | 20261943 | 206 | 1004 | 24 | 2 | 4 |
| 818 | 20261944 | 228 | 1340 | 17 | 3 | 4 |
| 819 | 20261945 | 278 | 1495 | 24 | 3 | 2 |
| 820 | 20261946 | 196 | 1050 | 12 | 8 | 4 |
| 821 | 20261947 | 243 | 1123 | 31 | 6 | 2 |
| 822 | 20261948 | 248 | 1263 | 12 | 4 | 3 |
| 823 | 20261949 | 230 | 1203 | 30 | 5 | 3 |
| 824 | 20261950 | 226 | 1121 | 17 | 5 | 2 |
| 825 | 20261951 | 205 | 1056 | 26 | 2 | 4 |
| 826 | 20261952 | 240 | 1291 | 27 | 4 | 3 |
| 827 | 20261953 | 220 | 1295 | 27 | 7 | 3 |
| 828 | 20261954 | 166 | 785 | 20 | 5 | 2 |
| 829 | 20261955 | 202 | 1056 | 17 | 8 | 2 |
| 830 | 20261956 | 257 | 1216 | 26 | 3 | 4 |
| 831 | 20261957 | 210 | 983 | 22 | 3 | 1 |
| 832 | 20261958 | 202 | 879 | 17 | 5 | 2 |
| 833 | 20261959 | 217 | 1040 | 25 | 3 | 4 |
| 834 | 20261960 | 226 | 745 | 15 | 4 | 3 |
| 835 | 20261961 | 257 | 1168 | 25 | 2 | 4 |
| 836 | 20261962 | 166 | 713 | 14 | 6 | 1 |
| 837 | 20261963 | 193 | 982 | 18 | 3 | 2 |
| 838 | 20261964 | 225 | 1335 | 32 | 4 | 3 |
| 839 | 20261965 | 195 | 969 | 25 | 4 | 2 |
| 840 | 20261966 | 249 | 1261 | 29 | 4 | 2 |
| 841 | 20261967 | 236 | 1260 | 19 | 7 | 2 |
| 842 | 20261968 | 224 | 1241 | 28 | 6 | 3 |
| 843 | 20261969 | 215 | 958 | 25 | 8 | 2 |
| 844 | 20261970 | 199 | 971 | 27 | 6 | 5 |
| 845 | 20261971 | 202 | 778 | 12 | 10 | 1 |
| 846 | 20261972 | 304 | 1896 | 16 | 9 | 3 |
| 847 | 20261973 | 194 | 967 | 18 | 7 | 1 |
| 848 | 20261974 | 264 | 1699 | 20 | 5 | 1 |
| 849 | 20261975 | 231 | 1120 | 22 | 5 | 2 |
| 850 | 20261976 | 201 | 958 | 30 | 6 | 3 |
| 851 | 20261977 | 209 | 1272 | 28 | 5 | 1 |
| 852 | 20261978 | 226 | 1163 | 20 | 5 | 2 |
| 853 | 20261979 | 207 | 830 | 19 | 3 | 2 |
| 854 | 20261980 | 216 | 891 | 19 | 7 | 2 |
| 855 | 20261981 | 242 | 1505 | 26 | 3 | 3 |
| 856 | 20261982 | 230 | 1080 | 17 | 4 | 1 |
| 857 | 20261983 | 227 | 1309 | 22 | 2 | 1 |
| 858 | 20261984 | 208 | 885 | 18 | 4 | 3 |
| 859 | 20261985 | 221 | 1400 | 22 | 8 | 2 |
| 860 | 20261986 | 272 | 1666 | 19 | 6 | 4 |
| 861 | 20261987 | 243 | 1280 | 31 | 8 | 3 |
| 862 | 20261988 | 227 | 1236 | 30 | 2 | 2 |
| 863 | 20261989 | 228 | 1125 | 29 | 5 | 2 |
| 864 | 20261990 | 147 | 577 | 12 | 2 | 0 |
| 865 | 20261991 | 222 | 1226 | 18 | 3 | 1 |
| 866 | 20261992 | 207 | 892 | 23 | 8 | 3 |
| 867 | 20261993 | 241 | 1188 | 23 | 2 | 2 |
| 868 | 20261994 | 319 | 1590 | 30 | 7 | 5 |
| 869 | 20261995 | 292 | 1759 | 23 | 1 | 1 |
| 870 | 20261996 | 222 | 1195 | 9 | 5 | 1 |
| 871 | 20261997 | 199 | 918 | 17 | 5 | 1 |
| 872 | 20261998 | 249 | 1359 | 17 | 2 | 4 |
| 873 | 20261999 | 199 | 1095 | 15 | 1 | 1 |
| 874 | 20262000 | 181 | 775 | 22 | 5 | 0 |
| 875 | 20262001 | 247 | 1409 | 34 | 10 | 3 |
| 876 | 20262002 | 226 | 822 | 14 | 5 | 2 |
| 877 | 20262003 | 210 | 930 | 7 | 5 | 2 |
| 878 | 20262004 | 456 | 3142 | 28 | 5 | 3 |
| 879 | 20262005 | 216 | 918 | 26 | 6 | 4 |
| 880 | 20262006 | 212 | 1040 | 18 | 3 | 4 |
| 881 | 20262007 | 271 | 1385 | 31 | 6 | 3 |
| 882 | 20262008 | 184 | 1204 | 20 | 5 | 1 |
| 883 | 20262009 | 218 | 1111 | 24 | 1 | 4 |
| 884 | 20262010 | 288 | 1194 | 14 | 6 | 2 |
| 885 | 20262011 | 260 | 1225 | 33 | 4 | 2 |
| 886 | 20262012 | 207 | 1198 | 17 | 5 | 0 |
| 887 | 20262013 | 284 | 1747 | 32 | 5 | 2 |
| 888 | 20262014 | 222 | 1236 | 24 | 3 | 3 |
| 889 | 20262015 | 198 | 1130 | 23 | 5 | 2 |
| 890 | 20262016 | 233 | 1205 | 19 | 6 | 3 |
| 891 | 20262017 | 247 | 1029 | 16 | 4 | 4 |
| 892 | 20262018 | 205 | 904 | 21 | 3 | 4 |
| 893 | 20262019 | 242 | 1200 | 32 | 5 | 4 |
| 894 | 20262020 | 266 | 1415 | 26 | 3 | 5 |
| 895 | 20262021 | 213 | 1272 | 16 | 5 | 1 |
| 896 | 20262022 | 229 | 1268 | 26 | 2 | 3 |
| 897 | 20262023 | 267 | 1240 | 23 | 4 | 3 |
| 898 | 20262024 | 212 | 860 | 18 | 3 | 2 |
| 899 | 20262025 | 218 | 1068 | 25 | 5 | 3 |
| 900 | 20262026 | 213 | 1009 | 15 | 0 | 2 |
| 901 | 20262027 | 223 | 1221 | 19 | 9 | 3 |
| 902 | 20262028 | 199 | 751 | 25 | 3 | 3 |
| 903 | 20262029 | 213 | 1176 | 19 | 6 | 2 |
| 904 | 20262030 | 197 | 757 | 13 | 3 | 2 |
| 905 | 20262031 | 293 | 1829 | 15 | 4 | 3 |
| 906 | 20262032 | 225 | 814 | 19 | 15 | 3 |
| 907 | 20262033 | 239 | 1296 | 28 | 3 | 3 |
| 908 | 20262034 | 260 | 1365 | 28 | 12 | 3 |
| 909 | 20262035 | 252 | 1863 | 30 | 6 | 2 |
| 910 | 20262036 | 211 | 1189 | 35 | 8 | 3 |
| 911 | 20262037 | 236 | 1552 | 23 | 6 | 2 |
| 912 | 20262038 | 202 | 1188 | 26 | 2 | 1 |
| 913 | 20262039 | 296 | 1609 | 33 | 1 | 3 |
| 914 | 20262040 | 179 | 981 | 8 | 7 | 2 |
| 915 | 20262041 | 220 | 1113 | 16 | 2 | 2 |
| 916 | 20262042 | 185 | 850 | 14 | 6 | 2 |
| 917 | 20262043 | 221 | 1121 | 16 | 8 | 2 |
| 918 | 20262044 | 349 | 2159 | 33 | 5 | 5 |
| 919 | 20262045 | 221 | 1265 | 32 | 3 | 3 |
| 920 | 20262046 | 259 | 1537 | 34 | 3 | 4 |
| 921 | 20262047 | 231 | 904 | 9 | 3 | 2 |
| 922 | 20262048 | 159 | 622 | 13 | 4 | 1 |
| 923 | 20262049 | 229 | 1147 | 29 | 8 | 3 |
| 924 | 20262050 | 165 | 630 | 15 | 3 | 3 |
| 925 | 20262051 | 244 | 1292 | 26 | 10 | 2 |
| 926 | 20262052 | 254 | 1255 | 28 | 5 | 2 |
| 927 | 20262053 | 211 | 1333 | 19 | 6 | 1 |
| 928 | 20262054 | 217 | 1044 | 12 | 4 | 2 |
| 929 | 20262055 | 233 | 1236 | 20 | 2 | 3 |
| 930 | 20262056 | 295 | 1602 | 43 | 7 | 5 |
| 931 | 20262057 | 182 | 853 | 16 | 5 | 1 |
| 932 | 20262058 | 217 | 1236 | 23 | 3 | 3 |
| 933 | 20262059 | 212 | 917 | 19 | 4 | 1 |
| 934 | 20262060 | 210 | 1157 | 17 | 2 | 0 |
| 935 | 20262061 | 274 | 1472 | 21 | 3 | 5 |
| 936 | 20262062 | 226 | 1386 | 32 | 9 | 1 |
| 937 | 20262063 | 229 | 1364 | 32 | 4 | 3 |
| 938 | 20262064 | 222 | 1273 | 27 | 3 | 3 |
| 939 | 20262065 | 292 | 1761 | 34 | 0 | 3 |
| 940 | 20262066 | 218 | 1090 | 25 | 5 | 1 |
| 941 | 20262067 | 224 | 1277 | 18 | 3 | 5 |
| 942 | 20262068 | 217 | 992 | 23 | 8 | 2 |
| 943 | 20262069 | 198 | 1017 | 23 | 2 | 2 |
| 944 | 20262070 | 241 | 1183 | 24 | 5 | 2 |
| 945 | 20262071 | 259 | 1542 | 33 | 2 | 2 |
| 946 | 20262072 | 352 | 2042 | 42 | 3 | 3 |
| 947 | 20262073 | 230 | 879 | 17 | 5 | 2 |
| 948 | 20262074 | 244 | 1401 | 13 | 4 | 1 |
| 949 | 20262075 | 192 | 809 | 23 | 1 | 1 |
| 950 | 20262076 | 307 | 1734 | 46 | 7 | 3 |
| 951 | 20262077 | 262 | 1568 | 18 | 8 | 3 |
| 952 | 20262078 | 164 | 455 | 7 | 3 | 0 |
| 953 | 20262079 | 254 | 1071 | 30 | 7 | 2 |
| 954 | 20262080 | 233 | 1490 | 27 | 3 | 2 |
| 955 | 20262081 | 253 | 1163 | 18 | 5 | 1 |
| 956 | 20262082 | 240 | 1158 | 22 | 8 | 1 |
| 957 | 20262083 | 190 | 815 | 14 | 4 | 1 |
| 958 | 20262084 | 227 | 1221 | 18 | 8 | 3 |
| 959 | 20262085 | 230 | 1209 | 16 | 3 | 5 |
| 960 | 20262086 | 264 | 1072 | 21 | 8 | 2 |
| 961 | 20262087 | 193 | 955 | 20 | 4 | 3 |
| 962 | 20262088 | 225 | 1142 | 21 | 4 | 3 |
| 963 | 20262089 | 235 | 1130 | 24 | 0 | 3 |
| 964 | 20262090 | 261 | 1637 | 31 | 3 | 3 |
| 965 | 20262091 | 256 | 1454 | 19 | 6 | 3 |
| 966 | 20262092 | 238 | 1364 | 27 | 3 | 3 |
| 967 | 20262093 | 219 | 1083 | 26 | 6 | 3 |
| 968 | 20262094 | 241 | 1116 | 18 | 10 | 0 |
| 969 | 20262095 | 204 | 1000 | 25 | 3 | 1 |
| 970 | 20262096 | 241 | 854 | 21 | 5 | 2 |
| 971 | 20262097 | 260 | 1434 | 19 | 3 | 2 |
| 972 | 20262098 | 216 | 1243 | 21 | 3 | 3 |
| 973 | 20262099 | 295 | 1416 | 30 | 1 | 4 |
| 974 | 20262100 | 193 | 897 | 16 | 5 | 3 |
| 975 | 20262101 | 245 | 1543 | 30 | 3 | 3 |
| 976 | 20262102 | 261 | 1300 | 22 | 3 | 4 |
| 977 | 20262103 | 278 | 1421 | 32 | 8 | 2 |
| 978 | 20262104 | 197 | 966 | 18 | 8 | 3 |
| 979 | 20262105 | 217 | 1117 | 18 | 3 | 3 |
| 980 | 20262106 | 219 | 1256 | 23 | 7 | 2 |
| 981 | 20262107 | 166 | 742 | 9 | 3 | 0 |
| 982 | 20262108 | 219 | 917 | 17 | 3 | 4 |
| 983 | 20262109 | 226 | 1125 | 27 | 7 | 1 |
| 984 | 20262110 | 145 | 553 | 7 | 3 | 0 |
| 985 | 20262111 | 200 | 1153 | 24 | 3 | 1 |
| 986 | 20262112 | 200 | 937 | 18 | 5 | 3 |
| 987 | 20262113 | 183 | 979 | 13 | 2 | 0 |
| 988 | 20262114 | 205 | 1071 | 13 | 1 | 1 |
| 989 | 20262115 | 197 | 851 | 21 | 9 | 1 |
| 990 | 20262116 | 189 | 647 | 12 | 3 | 0 |
| 991 | 20262117 | 219 | 1203 | 21 | 2 | 1 |
| 992 | 20262118 | 238 | 1512 | 28 | 7 | 3 |
| 993 | 20262119 | 230 | 881 | 11 | 4 | 2 |
| 994 | 20262120 | 238 | 1299 | 31 | 5 | 4 |
| 995 | 20262121 | 202 | 963 | 8 | 5 | 1 |
| 996 | 20262122 | 209 | 1163 | 22 | 9 | 2 |
| 997 | 20262123 | 234 | 1120 | 29 | 1 | 2 |
| 998 | 20262124 | 242 | 1244 | 21 | 4 | 3 |
| 999 | 20262125 | 208 | 1075 | 21 | 3 | 3 |

## Throughput

- Turns simulated: 228865 (first play of each game)
- Aggregate: 139.1 turns/s over 1645.6s wall
- Per game thread: 35.3 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

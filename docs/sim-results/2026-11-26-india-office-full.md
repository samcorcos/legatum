# Simulation: india-office-full (2026-11-26)

1000 games, first seed 20261126, brain `scripted`, turn limit off, safety cap 1200, 6 threads, started 2026-11-26T23:12:44Z. Wall time 879.5s.

Source checkout at run start: `0c5d2e8f05b65f08db2319b210af40dfe36974b9`; dirty: yes. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--games","1000","--seed","20261126","--threads","6","--difficulty","hard","--rules-experiment","india-office-full","--label","india-office-full"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 1000 of 1000 |
| Game length, turns (mean / median / min-max) | 250.8 / 241 / 143-500 |
| Win reason mix | VP hold 95.5%, Conquest 0.1%, Capitals 3.9%, Turn limit 0.4%, Draw 0.1% |
| Median end turn | 241 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 3.9% / 95.5% / 0.4% / 0.2% |
| Games reaching the World War era (any empire) | 994 of 1000 (99.4%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1466 / 425 / 957 / 84 |
| Hold break rate | 29.0% of started; 30.8% of resolved (broken / (broken + completed)) |
| Hold wins | 955 |
| First reach to win, turns (mean / median / min-max) | 13.2 / 9 / 9-281 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 143 |
| Wins by a team another team beat to its target | 86 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 955 | 247.7 / 240 / 143-497 |
| Conquest | 1 | 280.0 / 280 / 280-280 |
| Capitals | 39 | 293.8 / 280 / 207-493 |
| Turn limit | 4 | 500.0 / 500 / 500-500 |
| Draw | 1 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 105 | 10.5% |
| India | 36 | 3.6% |
| Byzantium | 90 | 9.0% |
| Germany | 150 | 15.0% |
| Celts | 169 | 16.9% |
| Mongols | 77 | 7.7% |
| Sparta | 121 | 12.1% |
| Athens | 144 | 14.4% |
| Aztecs | 107 | 10.7% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 955 | 95.5% |
| Conquest | 1 | 0.1% |
| Capitals | 39 | 3.9% |
| Turn limit | 4 | 0.4% |
| Draw | 1 | 0.1% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 899 | 105 | 11.7% |
| India | 895 | 36 | 4.0% |
| Byzantium | 878 | 90 | 10.3% |
| Germany | 880 | 150 | 17.0% |
| Celts | 896 | 169 | 18.9% |
| Mongols | 877 | 77 | 8.8% |
| Sparta | 891 | 121 | 13.6% |
| Athens | 895 | 144 | 16.1% |
| Aztecs | 889 | 107 | 12.0% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 8000 | 999 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 899 | 105 | 11.7% |
| India | Hard | 895 | 36 | 4.0% |
| Byzantium | Hard | 878 | 90 | 10.3% |
| Germany | Hard | 880 | 150 | 17.0% |
| Celts | Hard | 896 | 169 | 18.9% |
| Mongols | Hard | 877 | 77 | 8.8% |
| Sparta | Hard | 891 | 121 | 13.6% |
| Athens | Hard | 895 | 144 | 16.1% |
| Aztecs | Hard | 889 | 107 | 12.0% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 899 | 136 | 15.1% | 113 | 147 | 192 | 261 | 487 |
| India | 895 | 402 | 44.9% | 42 | 90 | 128 | 215 | 377 |
| Byzantium | 878 | 606 | 69.0% | 69 | 85 | 108 | 161 | 324 |
| Germany | 880 | 280 | 31.8% | 96 | 128 | 172 | 227 | 465 |
| Celts | 896 | 645 | 72.0% | 31 | 185 | 210 | 210 | 309 |
| Mongols | 877 | 137 | 15.6% | 119 | 162 | 205 | 256 | 399 |
| Sparta | 891 | 186 | 20.9% | 127 | 169 | 212 | 287 | 377 |
| Athens | 895 | 342 | 38.2% | 114 | 174 | 255 | 255 | 255 |
| Aztecs | 889 | 346 | 38.9% | 125 | 198 | 225 | 240 | 321 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 143 | 195 | 241 | 311 | 500 |
| Winning team VP | 2 | 10 | 11 | 13 | 17 |
| First Soldier completed (turn, per empire) | 6 | 8 | 21 | 64 | 314 |
| First Settler completed (turn, per empire) | 3 | 14 | 16 | 32 | 205 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 8 | 18 | 23 | 46 | 233 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 7675 | 325 |
| First Settler completed | 7655 | 345 |
| First city founded | 8000 | 0 |
| Second city founded | 7567 | 433 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 1970 |
| 1 | 1232 |
| 2 | 970 |
| 3 | 734 |
| 4 | 598 |
| 5 | 445 |
| 6 | 376 |
| 7 | 280 |
| 8 | 192 |
| 9 | 146 |
| 10 | 512 |
| 11 | 286 |
| 12 | 142 |
| 13 | 75 |
| 14 | 26 |
| 15 | 10 |
| 16 | 5 |
| 17 | 1 |

## End state

- Average game length: 250.8 turns
- Average cities per empire: 4.26
- Average techs per empire: 31.21
- Average eliminations per game: 2.70

## Strategy diversity: observed behavior (classifier v2)

1000 finished games; 8000 player seats; 999 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4410** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 3343 | 207 | 6.2% | 41.8% | 20.7% |
| Wide | 2557 | 705 | 27.6% | 32.0% | 70.6% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 12 | 0 | 0.0% | 0.1% | 0.0% |
| Economic | 650 | 86 | 13.2% | 8.1% | 8.6% |
| Generalist | 1438 | 1 | 0.1% | 18.0% | 0.1% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 19.8% / 3.2% (964) | n/a / n/a (0) | 27.1% / 2.8% (12) | 22.6% / 6.1% (472) | 81.7% / 2.0% (864) |
| Wide | 77.0% / 3.2% (964) | — | n/a / n/a (0) | 70.8% / 2.8% (12) | 57.1% / 4.8% (461) | 95.5% / 0.8% (855) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 70.1% / 2.8% (12) | 26.4% / 2.8% (12) | n/a / n/a (0) | — | 25.0% / 0.0% (4) | 95.5% / 4.5% (11) |
| Economic | 71.3% / 6.1% (472) | 38.1% / 4.8% (461) | n/a / n/a (0) | 75.0% / 0.0% (4) | — | 94.2% / 2.3% (392) |
| Generalist | 16.3% / 2.0% (864) | 3.6% / 0.8% (855) | n/a / n/a (0) | 0.0% / 4.5% (11) | 3.6% / 2.3% (392) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 9 / 254 | 86 / 379 | 0 / 0 | 0 / 0 | 10 / 119 | 0 / 147 |
| India | 13 / 494 | 23 / 239 | 0 / 0 | 0 / 1 | 0 / 6 | 0 / 155 |
| Byzantium | 0 / 10 | 82 / 408 | 0 / 0 | 0 / 3 | 8 / 127 | 0 / 330 |
| Germany | 24 / 345 | 120 / 297 | 0 / 0 | 0 / 3 | 6 / 47 | 0 / 188 |
| Celts | 17 / 264 | 141 / 411 | 0 / 0 | 0 / 0 | 11 / 90 | 0 / 131 |
| Mongols | 46 / 677 | 28 / 79 | 0 / 0 | 0 / 0 | 3 / 13 | 0 / 108 |
| Sparta | 61 / 690 | 44 / 151 | 0 / 0 | 0 / 5 | 16 / 33 | 0 / 12 |
| Athens | 17 / 267 | 104 / 304 | 0 / 0 | 0 / 0 | 23 / 126 | 0 / 198 |
| Aztecs | 20 / 342 | 77 / 289 | 0 / 0 | 0 / 0 | 9 / 89 | 1 / 169 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Turtle/Wonder. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 999 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 15445/1000 | 15.445 | 1000/1000 (100.0%) |
| Peace offers | 100603/1000 | 100.603 | 1000/1000 (100.0%) |
| Peace agreements | 12338/1000 | 12.338 | 997/1000 (99.7%) |
| Battles | 1255085/1000 | 1255.085 | 1000/1000 (100.0%) |
| City captures | 23790/1000 | 23.790 | 1000/1000 (100.0%) |
| Settler captures | 4309/1000 | 4.309 | 961/1000 (96.1%) |
| Eliminations | 2698/1000 | 2.698 | 979/1000 (97.9%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 6574/8000 | 6574/8000 | 3132/999 |
| Faction goals | 5040/8000 | 5040/8000 | 2096/999 |
| Era objectives | 14416/8000 | 14416/8000 | 4094/999 |
| Capitals taken | 2583/8000 | 2583/8000 | 1513/999 |
| Total VP | 28613/8000 | 28613/8000 | 10835/999 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 101/723 (14.0%) | 1 |
| Tied leaders (winner in set) | 265/277 (95.7%) | 0 |
| Any leader set | 366/1000 (36.6%) | 1 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 7536/8000 | 5 | 20 | 29 | 33 | 70 | -2.1 (999) |
| Medieval | 7303/8000 | 31 | 58 | 62 | 66 | 130 | -1.9 (999) |
| Renaissance | 7234/8000 | 65 | 84 | 90 | 98 | 169 | -3.5 (999) |
| Industrial | 7015/8000 | 89 | 112 | 121 | 136 | 349 | -5.8 (999) |
| Machine Age | 6398/8000 | 109 | 141 | 153 | 174 | 444 | -8.3 (999) |
| World War | 5245/8000 | 128 | 167 | 182 | 208 | 499 | -10.5 (989) |
| Modern | 3601/8000 | 152 | 191 | 212 | 245 | 457 | -11.1 (899) |
| Future | 1896/8000 | 176 | 216 | 244 | 267 | 427 | -8.9 (613) |

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
| Medieval | Chivalry | 6447/8000 | 81 | 1130 AD | n/a |
| Medieval | Education | 7248/8000 | 86 | 1280 AD | 1088 AD |
| Medieval | Crop Rotation | 7236/8000 | 89 | 1370 AD | n/a |
| Renaissance | Gunpowder | 7217/8000 | 97 | 1470 AD | n/a |
| Renaissance | Metallurgy | 7152/8000 | 108 | 1580 AD | n/a |
| Renaissance | Military Engineering | 7172/8000 | 105 | 1550 AD | n/a |
| Renaissance | Banking | 7064/8000 | 118 | 1680 AD | 1397 AD |
| Renaissance | Printing Press | 7026/8000 | 119 | 1690 AD | 1440 AD |
| Industrial | Rifling | 6663/8000 | 143 | 1816 AD | n/a |
| Industrial | Military Science | 6827/8000 | 134 | 1783 AD | n/a |
| Industrial | Ballistics | 6694/8000 | 145 | 1824 AD | n/a |
| Industrial | Industrialization | 6921/8000 | 130 | 1768 AD | 1769 AD |
| Industrial | Railways | 6401/8000 | 153 | 1854 AD | 1825 AD |
| Machine Age | Replaceable Parts | 5918/8000 | 168 | 1889 AD | 1800 AD |
| Machine Age | Combustion | 6170/8000 | 162 | 1882 AD | 1876 AD |
| Machine Age | Steel | 5877/8000 | 167 | 1887 AD | 1856 AD |
| Machine Age | Flight | 5501/8000 | 176 | 1898 AD | 1903 AD |
| Machine Age | Electricity | 5296/8000 | 182 | 1904 AD | 1879 AD |
| World War | Mass Mobilisation | 4454/8000 | 197 | 1924 AD | n/a |
| World War | Rocketry | 4184/8000 | 202 | 1931 AD | 1926 AD |
| World War | Armoured Warfare | 3980/8000 | 207 | 1938 AD | 1916 AD |
| World War | Air Power | 3789/8000 | 210 | 1942 AD | n/a |
| World War | Electronics | 4914/8000 | 189 | 1912 AD | n/a |
| Modern | Mechanised Warfare | 2659/8000 | 226 | 1976 AD | n/a |
| Modern | Composite Armour | 2386/8000 | 230 | 1986 AD | n/a |
| Modern | Guided Missiles | 2237/8000 | 237 | 2002 AD | n/a |
| Modern | Jet Engines | 2150/8000 | 239 | 2007 AD | 1939 AD |
| Modern | Green Revolution | 3095/8000 | 219 | 1959 AD | n/a |
| Future | Autonomous Systems | 1292/8000 | 256 | 2032 AD | n/a |
| Future | Robotics | 1086/8000 | 260 | 2036 AD | n/a |
| Future | Fusion Power | 1044/8000 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 1516/8000 | 253 | 2029 AD | n/a |
| Future | Recursive AI | 933/8000 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 62.35 |
| 1 | 72.93 | 14.30 |
| 2 | 27.86 | 5.47 |
| 3 | 7.86 | 1.25 |
| 4 | 1.62 | 0.28 |
| 5 | 0.35 | 0.08 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 20261126 | 297 | 1566 | 32 | 7 | 4 |
| 1 | 20261127 | 229 | 1048 | 13 | 3 | 3 |
| 2 | 20261128 | 277 | 1660 | 14 | 8 | 1 |
| 3 | 20261129 | 287 | 1198 | 37 | 1 | 2 |
| 4 | 20261130 | 240 | 1364 | 32 | 3 | 3 |
| 5 | 20261131 | 295 | 1351 | 39 | 5 | 5 |
| 6 | 20261132 | 287 | 1553 | 18 | 2 | 3 |
| 7 | 20261133 | 311 | 1597 | 38 | 2 | 5 |
| 8 | 20261134 | 244 | 1096 | 19 | 6 | 3 |
| 9 | 20261135 | 221 | 1218 | 22 | 6 | 3 |
| 10 | 20261136 | 286 | 1360 | 29 | 6 | 2 |
| 11 | 20261137 | 278 | 1028 | 26 | 2 | 3 |
| 12 | 20261138 | 217 | 1251 | 24 | 2 | 1 |
| 13 | 20261139 | 238 | 1105 | 23 | 2 | 2 |
| 14 | 20261140 | 348 | 1858 | 42 | 2 | 4 |
| 15 | 20261141 | 231 | 1056 | 16 | 5 | 2 |
| 16 | 20261142 | 244 | 1235 | 31 | 4 | 1 |
| 17 | 20261143 | 250 | 1498 | 27 | 10 | 3 |
| 18 | 20261144 | 278 | 1146 | 29 | 3 | 4 |
| 19 | 20261145 | 318 | 1466 | 43 | 3 | 4 |
| 20 | 20261146 | 215 | 1035 | 15 | 1 | 2 |
| 21 | 20261147 | 377 | 2024 | 29 | 10 | 1 |
| 22 | 20261148 | 277 | 1679 | 27 | 5 | 2 |
| 23 | 20261149 | 333 | 2052 | 26 | 1 | 3 |
| 24 | 20261150 | 279 | 1608 | 21 | 4 | 1 |
| 25 | 20261151 | 242 | 1059 | 24 | 0 | 2 |
| 26 | 20261152 | 184 | 982 | 23 | 4 | 4 |
| 27 | 20261153 | 200 | 931 | 10 | 6 | 2 |
| 28 | 20261154 | 398 | 2498 | 39 | 9 | 1 |
| 29 | 20261155 | 204 | 859 | 23 | 7 | 2 |
| 30 | 20261156 | 408 | 2362 | 41 | 3 | 4 |
| 31 | 20261157 | 210 | 1069 | 22 | 2 | 3 |
| 32 | 20261158 | 246 | 1330 | 25 | 5 | 4 |
| 33 | 20261159 | 289 | 1405 | 36 | 3 | 3 |
| 34 | 20261160 | 188 | 750 | 17 | 3 | 3 |
| 35 | 20261161 | 211 | 834 | 15 | 4 | 2 |
| 36 | 20261162 | 424 | 1994 | 44 | 0 | 4 |
| 37 | 20261163 | 252 | 1337 | 32 | 4 | 2 |
| 38 | 20261164 | 199 | 803 | 21 | 4 | 3 |
| 39 | 20261165 | 297 | 1747 | 25 | 2 | 1 |
| 40 | 20261166 | 167 | 925 | 13 | 6 | 2 |
| 41 | 20261167 | 193 | 945 | 24 | 4 | 4 |
| 42 | 20261168 | 204 | 1225 | 23 | 2 | 3 |
| 43 | 20261169 | 285 | 1744 | 35 | 11 | 2 |
| 44 | 20261170 | 257 | 1334 | 15 | 5 | 3 |
| 45 | 20261171 | 220 | 886 | 16 | 6 | 2 |
| 46 | 20261172 | 268 | 1348 | 29 | 3 | 4 |
| 47 | 20261173 | 238 | 1031 | 20 | 3 | 3 |
| 48 | 20261174 | 210 | 1080 | 24 | 2 | 3 |
| 49 | 20261175 | 232 | 1337 | 20 | 1 | 3 |
| 50 | 20261176 | 231 | 1135 | 21 | 3 | 2 |
| 51 | 20261177 | 232 | 1249 | 21 | 4 | 3 |
| 52 | 20261178 | 242 | 1088 | 18 | 0 | 2 |
| 53 | 20261179 | 222 | 1014 | 18 | 3 | 2 |
| 54 | 20261180 | 203 | 789 | 10 | 2 | 3 |
| 55 | 20261181 | 216 | 888 | 16 | 5 | 2 |
| 56 | 20261182 | 257 | 1222 | 30 | 4 | 2 |
| 57 | 20261183 | 233 | 1269 | 34 | 4 | 4 |
| 58 | 20261184 | 234 | 1150 | 26 | 1 | 3 |
| 59 | 20261185 | 250 | 1044 | 21 | 3 | 2 |
| 60 | 20261186 | 300 | 1157 | 28 | 1 | 2 |
| 61 | 20261187 | 271 | 1099 | 30 | 5 | 3 |
| 62 | 20261188 | 339 | 2048 | 39 | 1 | 3 |
| 63 | 20261189 | 235 | 1315 | 20 | 6 | 3 |
| 64 | 20261190 | 272 | 1347 | 34 | 9 | 3 |
| 65 | 20261191 | 500 | 2699 | 25 | 7 | 4 |
| 66 | 20261192 | 271 | 1513 | 37 | 2 | 2 |
| 67 | 20261193 | 276 | 1677 | 20 | 1 | 3 |
| 68 | 20261194 | 324 | 1486 | 31 | 2 | 3 |
| 69 | 20261195 | 198 | 973 | 10 | 5 | 2 |
| 70 | 20261196 | 193 | 836 | 13 | 7 | 2 |
| 71 | 20261197 | 284 | 1564 | 20 | 3 | 5 |
| 72 | 20261198 | 216 | 984 | 11 | 4 | 1 |
| 73 | 20261199 | 323 | 1581 | 33 | 6 | 4 |
| 74 | 20261200 | 228 | 1239 | 33 | 3 | 1 |
| 75 | 20261201 | 240 | 1136 | 24 | 7 | 2 |
| 76 | 20261202 | 262 | 1576 | 32 | 10 | 0 |
| 77 | 20261203 | 280 | 1512 | 33 | 9 | 7 |
| 78 | 20261204 | 365 | 1637 | 22 | 8 | 6 |
| 79 | 20261205 | 268 | 1406 | 26 | 3 | 1 |
| 80 | 20261206 | 222 | 838 | 23 | 12 | 2 |
| 81 | 20261207 | 232 | 811 | 13 | 7 | 2 |
| 82 | 20261208 | 207 | 1015 | 22 | 7 | 0 |
| 83 | 20261209 | 208 | 1127 | 20 | 2 | 1 |
| 84 | 20261210 | 303 | 1486 | 35 | 10 | 2 |
| 85 | 20261211 | 224 | 1296 | 35 | 3 | 3 |
| 86 | 20261212 | 241 | 1080 | 17 | 2 | 3 |
| 87 | 20261213 | 246 | 868 | 20 | 7 | 2 |
| 88 | 20261214 | 270 | 993 | 25 | 6 | 3 |
| 89 | 20261215 | 367 | 1790 | 36 | 6 | 5 |
| 90 | 20261216 | 245 | 1198 | 30 | 7 | 3 |
| 91 | 20261217 | 269 | 1255 | 40 | 4 | 3 |
| 92 | 20261218 | 209 | 1044 | 19 | 4 | 2 |
| 93 | 20261219 | 241 | 1088 | 9 | 3 | 4 |
| 94 | 20261220 | 283 | 1246 | 28 | 3 | 3 |
| 95 | 20261221 | 273 | 1236 | 31 | 8 | 3 |
| 96 | 20261222 | 237 | 1317 | 51 | 5 | 2 |
| 97 | 20261223 | 241 | 956 | 19 | 3 | 2 |
| 98 | 20261224 | 299 | 1476 | 30 | 6 | 2 |
| 99 | 20261225 | 322 | 1978 | 40 | 6 | 2 |
| 100 | 20261226 | 234 | 1208 | 26 | 3 | 3 |
| 101 | 20261227 | 229 | 1227 | 38 | 5 | 3 |
| 102 | 20261228 | 219 | 1051 | 10 | 4 | 1 |
| 103 | 20261229 | 254 | 1157 | 22 | 3 | 3 |
| 104 | 20261230 | 379 | 2031 | 29 | 4 | 3 |
| 105 | 20261231 | 199 | 692 | 12 | 1 | 2 |
| 106 | 20261232 | 215 | 1237 | 23 | 4 | 2 |
| 107 | 20261233 | 272 | 1335 | 27 | 2 | 2 |
| 108 | 20261234 | 207 | 655 | 22 | 9 | 4 |
| 109 | 20261235 | 271 | 1351 | 20 | 3 | 3 |
| 110 | 20261236 | 181 | 746 | 9 | 3 | 2 |
| 111 | 20261237 | 265 | 1382 | 31 | 2 | 4 |
| 112 | 20261238 | 241 | 1084 | 40 | 5 | 4 |
| 113 | 20261239 | 267 | 1580 | 22 | 3 | 2 |
| 114 | 20261240 | 223 | 1135 | 21 | 4 | 2 |
| 115 | 20261241 | 236 | 1020 | 15 | 0 | 4 |
| 116 | 20261242 | 218 | 1102 | 29 | 4 | 3 |
| 117 | 20261243 | 226 | 1131 | 17 | 5 | 2 |
| 118 | 20261244 | 216 | 1224 | 21 | 6 | 2 |
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
| 130 | 20261256 | 209 | 1040 | 23 | 1 | 3 |
| 131 | 20261257 | 237 | 1143 | 22 | 1 | 3 |
| 132 | 20261258 | 272 | 1071 | 13 | 1 | 3 |
| 133 | 20261259 | 176 | 753 | 14 | 5 | 1 |
| 134 | 20261260 | 243 | 1339 | 28 | 3 | 2 |
| 135 | 20261261 | 500 | 2016 | 64 | 5 | 5 |
| 136 | 20261262 | 212 | 988 | 24 | 1 | 1 |
| 137 | 20261263 | 229 | 1226 | 23 | 1 | 3 |
| 138 | 20261264 | 192 | 942 | 20 | 5 | 3 |
| 139 | 20261265 | 224 | 1086 | 20 | 5 | 3 |
| 140 | 20261266 | 234 | 1339 | 23 | 8 | 2 |
| 141 | 20261267 | 280 | 1206 | 32 | 5 | 5 |
| 142 | 20261268 | 226 | 1125 | 24 | 3 | 0 |
| 143 | 20261269 | 247 | 1311 | 27 | 6 | 4 |
| 144 | 20261270 | 234 | 1004 | 22 | 1 | 1 |
| 145 | 20261271 | 277 | 1335 | 21 | 2 | 2 |
| 146 | 20261272 | 264 | 1290 | 29 | 9 | 2 |
| 147 | 20261273 | 243 | 1078 | 23 | 1 | 4 |
| 148 | 20261274 | 243 | 1168 | 22 | 4 | 2 |
| 149 | 20261275 | 212 | 974 | 20 | 3 | 2 |
| 150 | 20261276 | 279 | 1047 | 10 | 8 | 1 |
| 151 | 20261277 | 221 | 1100 | 18 | 6 | 3 |
| 152 | 20261278 | 225 | 1236 | 23 | 6 | 2 |
| 153 | 20261279 | 208 | 1068 | 20 | 14 | 0 |
| 154 | 20261280 | 242 | 1068 | 30 | 5 | 3 |
| 155 | 20261281 | 194 | 959 | 31 | 4 | 2 |
| 156 | 20261282 | 229 | 1102 | 20 | 4 | 3 |
| 157 | 20261283 | 225 | 1069 | 25 | 3 | 1 |
| 158 | 20261284 | 298 | 1430 | 28 | 5 | 4 |
| 159 | 20261285 | 236 | 1250 | 29 | 10 | 3 |
| 160 | 20261286 | 157 | 801 | 19 | 4 | 2 |
| 161 | 20261287 | 270 | 1488 | 23 | 3 | 1 |
| 162 | 20261288 | 228 | 1253 | 31 | 3 | 3 |
| 163 | 20261289 | 228 | 1161 | 17 | 1 | 3 |
| 164 | 20261290 | 298 | 1638 | 35 | 5 | 3 |
| 165 | 20261291 | 270 | 1390 | 27 | 6 | 3 |
| 166 | 20261292 | 251 | 1303 | 32 | 5 | 4 |
| 167 | 20261293 | 225 | 1068 | 24 | 3 | 2 |
| 168 | 20261294 | 229 | 1317 | 15 | 4 | 3 |
| 169 | 20261295 | 354 | 1933 | 27 | 6 | 4 |
| 170 | 20261296 | 306 | 1842 | 21 | 7 | 3 |
| 171 | 20261297 | 220 | 781 | 21 | 1 | 3 |
| 172 | 20261298 | 236 | 1056 | 27 | 3 | 3 |
| 173 | 20261299 | 330 | 1534 | 34 | 5 | 4 |
| 174 | 20261300 | 237 | 1274 | 25 | 8 | 1 |
| 175 | 20261301 | 243 | 1217 | 30 | 0 | 2 |
| 176 | 20261302 | 195 | 969 | 17 | 5 | 3 |
| 177 | 20261303 | 237 | 1083 | 28 | 0 | 3 |
| 178 | 20261304 | 246 | 880 | 7 | 1 | 5 |
| 179 | 20261305 | 299 | 2003 | 26 | 8 | 1 |
| 180 | 20261306 | 309 | 1478 | 30 | 3 | 5 |
| 181 | 20261307 | 214 | 953 | 25 | 2 | 2 |
| 182 | 20261308 | 257 | 1100 | 28 | 4 | 3 |
| 183 | 20261309 | 251 | 1122 | 30 | 4 | 4 |
| 184 | 20261310 | 271 | 1617 | 26 | 9 | 3 |
| 185 | 20261311 | 214 | 1085 | 25 | 5 | 3 |
| 186 | 20261312 | 276 | 1440 | 18 | 5 | 4 |
| 187 | 20261313 | 263 | 1384 | 32 | 7 | 3 |
| 188 | 20261314 | 208 | 1046 | 18 | 7 | 2 |
| 189 | 20261315 | 402 | 2095 | 32 | 10 | 5 |
| 190 | 20261316 | 202 | 870 | 13 | 7 | 4 |
| 191 | 20261317 | 242 | 1243 | 16 | 4 | 3 |
| 192 | 20261318 | 260 | 1285 | 26 | 3 | 2 |
| 193 | 20261319 | 278 | 1137 | 26 | 1 | 3 |
| 194 | 20261320 | 226 | 1076 | 23 | 4 | 1 |
| 195 | 20261321 | 235 | 1170 | 31 | 6 | 3 |
| 196 | 20261322 | 259 | 1034 | 27 | 1 | 2 |
| 197 | 20261323 | 194 | 997 | 10 | 2 | 3 |
| 198 | 20261324 | 219 | 981 | 15 | 5 | 3 |
| 199 | 20261325 | 175 | 794 | 17 | 5 | 2 |
| 200 | 20261326 | 236 | 1214 | 28 | 2 | 3 |
| 201 | 20261327 | 218 | 784 | 21 | 5 | 3 |
| 202 | 20261328 | 203 | 996 | 17 | 2 | 4 |
| 203 | 20261329 | 225 | 1183 | 19 | 2 | 1 |
| 204 | 20261330 | 201 | 862 | 19 | 3 | 1 |
| 205 | 20261331 | 299 | 1330 | 38 | 2 | 5 |
| 206 | 20261332 | 296 | 1403 | 18 | 4 | 3 |
| 207 | 20261333 | 179 | 738 | 10 | 5 | 1 |
| 208 | 20261334 | 232 | 1303 | 20 | 3 | 3 |
| 209 | 20261335 | 447 | 2363 | 53 | 7 | 3 |
| 210 | 20261336 | 251 | 1370 | 16 | 12 | 2 |
| 211 | 20261337 | 249 | 1294 | 22 | 11 | 3 |
| 212 | 20261338 | 249 | 1175 | 12 | 5 | 0 |
| 213 | 20261339 | 208 | 909 | 14 | 6 | 1 |
| 214 | 20261340 | 233 | 764 | 8 | 3 | 2 |
| 215 | 20261341 | 182 | 720 | 18 | 7 | 4 |
| 216 | 20261342 | 238 | 1245 | 29 | 3 | 3 |
| 217 | 20261343 | 220 | 1000 | 22 | 10 | 3 |
| 218 | 20261344 | 233 | 1096 | 17 | 3 | 2 |
| 219 | 20261345 | 227 | 926 | 31 | 2 | 2 |
| 220 | 20261346 | 318 | 1461 | 35 | 1 | 6 |
| 221 | 20261347 | 266 | 1457 | 29 | 3 | 4 |
| 222 | 20261348 | 228 | 1082 | 22 | 5 | 3 |
| 223 | 20261349 | 215 | 892 | 12 | 6 | 3 |
| 224 | 20261350 | 207 | 1095 | 22 | 4 | 5 |
| 225 | 20261351 | 193 | 836 | 24 | 5 | 3 |
| 226 | 20261352 | 186 | 794 | 17 | 2 | 2 |
| 227 | 20261353 | 231 | 1277 | 20 | 5 | 3 |
| 228 | 20261354 | 231 | 928 | 15 | 2 | 2 |
| 229 | 20261355 | 253 | 1416 | 21 | 0 | 3 |
| 230 | 20261356 | 325 | 1387 | 22 | 1 | 6 |
| 231 | 20261357 | 244 | 1220 | 23 | 3 | 2 |
| 232 | 20261358 | 224 | 1094 | 15 | 7 | 4 |
| 233 | 20261359 | 233 | 1180 | 28 | 4 | 3 |
| 234 | 20261360 | 202 | 811 | 15 | 4 | 4 |
| 235 | 20261361 | 237 | 921 | 13 | 4 | 2 |
| 236 | 20261362 | 370 | 1909 | 42 | 4 | 2 |
| 237 | 20261363 | 276 | 1456 | 26 | 7 | 3 |
| 238 | 20261364 | 212 | 862 | 21 | 6 | 4 |
| 239 | 20261365 | 335 | 1677 | 27 | 4 | 5 |
| 240 | 20261366 | 228 | 1267 | 27 | 7 | 3 |
| 241 | 20261367 | 241 | 1208 | 14 | 1 | 4 |
| 242 | 20261368 | 229 | 1224 | 26 | 3 | 3 |
| 243 | 20261369 | 249 | 1091 | 24 | 4 | 2 |
| 244 | 20261370 | 200 | 995 | 23 | 7 | 2 |
| 245 | 20261371 | 210 | 868 | 20 | 1 | 1 |
| 246 | 20261372 | 278 | 1261 | 10 | 2 | 5 |
| 247 | 20261373 | 237 | 1350 | 32 | 1 | 3 |
| 248 | 20261374 | 394 | 2741 | 16 | 3 | 5 |
| 249 | 20261375 | 222 | 959 | 16 | 3 | 2 |
| 250 | 20261376 | 242 | 1414 | 27 | 0 | 2 |
| 251 | 20261377 | 185 | 878 | 15 | 4 | 3 |
| 252 | 20261378 | 309 | 1865 | 23 | 6 | 3 |
| 253 | 20261379 | 234 | 1008 | 17 | 2 | 2 |
| 254 | 20261380 | 235 | 1163 | 24 | 7 | 3 |
| 255 | 20261381 | 306 | 1386 | 19 | 5 | 4 |
| 256 | 20261382 | 221 | 849 | 22 | 1 | 4 |
| 257 | 20261383 | 267 | 1284 | 30 | 4 | 5 |
| 258 | 20261384 | 246 | 1616 | 25 | 4 | 3 |
| 259 | 20261385 | 216 | 927 | 12 | 7 | 4 |
| 260 | 20261386 | 285 | 1292 | 23 | 7 | 2 |
| 261 | 20261387 | 253 | 1150 | 33 | 9 | 1 |
| 262 | 20261388 | 206 | 825 | 15 | 4 | 4 |
| 263 | 20261389 | 259 | 1445 | 23 | 4 | 3 |
| 264 | 20261390 | 220 | 985 | 25 | 2 | 4 |
| 265 | 20261391 | 200 | 966 | 17 | 5 | 2 |
| 266 | 20261392 | 193 | 789 | 14 | 0 | 3 |
| 267 | 20261393 | 258 | 1293 | 22 | 5 | 1 |
| 268 | 20261394 | 415 | 2221 | 55 | 10 | 4 |
| 269 | 20261395 | 324 | 1918 | 32 | 7 | 6 |
| 270 | 20261396 | 236 | 1076 | 33 | 3 | 3 |
| 271 | 20261397 | 219 | 1213 | 22 | 5 | 3 |
| 272 | 20261398 | 324 | 1849 | 29 | 2 | 4 |
| 273 | 20261399 | 276 | 1382 | 35 | 7 | 3 |
| 274 | 20261400 | 213 | 683 | 19 | 1 | 3 |
| 275 | 20261401 | 187 | 883 | 19 | 6 | 2 |
| 276 | 20261402 | 202 | 952 | 20 | 3 | 3 |
| 277 | 20261403 | 311 | 1452 | 32 | 7 | 3 |
| 278 | 20261404 | 250 | 1423 | 24 | 2 | 2 |
| 279 | 20261405 | 276 | 1265 | 21 | 2 | 4 |
| 280 | 20261406 | 277 | 1581 | 26 | 13 | 2 |
| 281 | 20261407 | 240 | 1277 | 22 | 3 | 2 |
| 282 | 20261408 | 246 | 1109 | 31 | 9 | 2 |
| 283 | 20261409 | 283 | 1469 | 19 | 8 | 3 |
| 284 | 20261410 | 273 | 1537 | 21 | 4 | 5 |
| 285 | 20261411 | 296 | 1482 | 32 | 9 | 3 |
| 286 | 20261412 | 208 | 1102 | 14 | 6 | 1 |
| 287 | 20261413 | 301 | 1535 | 25 | 2 | 2 |
| 288 | 20261414 | 264 | 1380 | 23 | 4 | 2 |
| 289 | 20261415 | 281 | 1633 | 30 | 4 | 2 |
| 290 | 20261416 | 275 | 1676 | 18 | 2 | 5 |
| 291 | 20261417 | 180 | 776 | 13 | 3 | 2 |
| 292 | 20261418 | 271 | 1351 | 27 | 1 | 3 |
| 293 | 20261419 | 309 | 1843 | 32 | 6 | 3 |
| 294 | 20261420 | 257 | 1206 | 29 | 7 | 2 |
| 295 | 20261421 | 226 | 1157 | 15 | 11 | 0 |
| 296 | 20261422 | 190 | 759 | 15 | 0 | 1 |
| 297 | 20261423 | 229 | 1072 | 13 | 7 | 2 |
| 298 | 20261424 | 334 | 1413 | 34 | 1 | 5 |
| 299 | 20261425 | 248 | 1614 | 20 | 2 | 3 |
| 300 | 20261426 | 188 | 786 | 20 | 4 | 1 |
| 301 | 20261427 | 341 | 2389 | 46 | 1 | 3 |
| 302 | 20261428 | 221 | 1207 | 28 | 8 | 2 |
| 303 | 20261429 | 237 | 1401 | 13 | 3 | 2 |
| 304 | 20261430 | 192 | 915 | 14 | 7 | 2 |
| 305 | 20261431 | 269 | 1212 | 26 | 6 | 4 |
| 306 | 20261432 | 248 | 816 | 24 | 3 | 3 |
| 307 | 20261433 | 229 | 1157 | 40 | 3 | 3 |
| 308 | 20261434 | 196 | 852 | 20 | 4 | 3 |
| 309 | 20261435 | 208 | 1223 | 18 | 2 | 3 |
| 310 | 20261436 | 217 | 1208 | 30 | 1 | 2 |
| 311 | 20261437 | 348 | 1821 | 37 | 4 | 2 |
| 312 | 20261438 | 226 | 1073 | 13 | 3 | 3 |
| 313 | 20261439 | 172 | 814 | 14 | 3 | 2 |
| 314 | 20261440 | 374 | 1878 | 35 | 7 | 4 |
| 315 | 20261441 | 273 | 1197 | 14 | 3 | 1 |
| 316 | 20261442 | 257 | 1174 | 33 | 4 | 4 |
| 317 | 20261443 | 235 | 1075 | 16 | 3 | 2 |
| 318 | 20261444 | 286 | 1325 | 33 | 6 | 2 |
| 319 | 20261445 | 312 | 1599 | 41 | 4 | 2 |
| 320 | 20261446 | 313 | 1940 | 38 | 2 | 3 |
| 321 | 20261447 | 244 | 1322 | 28 | 1 | 1 |
| 322 | 20261448 | 189 | 925 | 14 | 2 | 1 |
| 323 | 20261449 | 209 | 1130 | 14 | 3 | 0 |
| 324 | 20261450 | 222 | 1068 | 24 | 7 | 2 |
| 325 | 20261451 | 286 | 1523 | 38 | 8 | 3 |
| 326 | 20261452 | 240 | 1212 | 22 | 7 | 4 |
| 327 | 20261453 | 303 | 1550 | 32 | 0 | 4 |
| 328 | 20261454 | 276 | 1770 | 35 | 5 | 1 |
| 329 | 20261455 | 213 | 870 | 28 | 8 | 1 |
| 330 | 20261456 | 210 | 1098 | 20 | 2 | 3 |
| 331 | 20261457 | 248 | 1012 | 30 | 5 | 3 |
| 332 | 20261458 | 253 | 1400 | 34 | 9 | 3 |
| 333 | 20261459 | 227 | 1181 | 32 | 3 | 3 |
| 334 | 20261460 | 293 | 1331 | 27 | 5 | 3 |
| 335 | 20261461 | 178 | 761 | 11 | 4 | 2 |
| 336 | 20261462 | 461 | 2788 | 56 | 3 | 5 |
| 337 | 20261463 | 248 | 1500 | 14 | 4 | 1 |
| 338 | 20261464 | 199 | 981 | 17 | 10 | 2 |
| 339 | 20261465 | 229 | 1183 | 19 | 10 | 1 |
| 340 | 20261466 | 222 | 1144 | 20 | 4 | 3 |
| 341 | 20261467 | 292 | 1533 | 37 | 5 | 5 |
| 342 | 20261468 | 194 | 921 | 20 | 5 | 2 |
| 343 | 20261469 | 315 | 1873 | 36 | 5 | 3 |
| 344 | 20261470 | 296 | 1133 | 26 | 2 | 3 |
| 345 | 20261471 | 227 | 1224 | 23 | 9 | 1 |
| 346 | 20261472 | 234 | 1250 | 18 | 1 | 3 |
| 347 | 20261473 | 263 | 1323 | 29 | 2 | 2 |
| 348 | 20261474 | 270 | 1234 | 29 | 3 | 3 |
| 349 | 20261475 | 395 | 2164 | 65 | 6 | 4 |
| 350 | 20261476 | 283 | 1621 | 22 | 21 | 1 |
| 351 | 20261477 | 233 | 1115 | 27 | 2 | 3 |
| 352 | 20261478 | 223 | 961 | 22 | 6 | 4 |
| 353 | 20261479 | 255 | 1509 | 17 | 4 | 3 |
| 354 | 20261480 | 351 | 1470 | 41 | 7 | 5 |
| 355 | 20261481 | 245 | 1389 | 24 | 2 | 3 |
| 356 | 20261482 | 311 | 1597 | 27 | 9 | 4 |
| 357 | 20261483 | 191 | 899 | 18 | 4 | 3 |
| 358 | 20261484 | 221 | 1032 | 23 | 6 | 1 |
| 359 | 20261485 | 197 | 1368 | 31 | 1 | 1 |
| 360 | 20261486 | 277 | 1072 | 25 | 4 | 2 |
| 361 | 20261487 | 231 | 1147 | 21 | 4 | 2 |
| 362 | 20261488 | 217 | 1248 | 22 | 0 | 2 |
| 363 | 20261489 | 233 | 1091 | 16 | 7 | 2 |
| 364 | 20261490 | 237 | 979 | 23 | 2 | 4 |
| 365 | 20261491 | 263 | 1304 | 28 | 4 | 4 |
| 366 | 20261492 | 228 | 940 | 19 | 7 | 2 |
| 367 | 20261493 | 233 | 1258 | 16 | 1 | 3 |
| 368 | 20261494 | 299 | 1653 | 23 | 19 | 4 |
| 369 | 20261495 | 223 | 1208 | 18 | 11 | 3 |
| 370 | 20261496 | 193 | 889 | 20 | 3 | 3 |
| 371 | 20261497 | 255 | 1299 | 16 | 4 | 4 |
| 372 | 20261498 | 227 | 1201 | 32 | 2 | 4 |
| 373 | 20261499 | 222 | 1191 | 31 | 1 | 2 |
| 374 | 20261500 | 272 | 1357 | 30 | 1 | 4 |
| 375 | 20261501 | 244 | 1137 | 28 | 6 | 1 |
| 376 | 20261502 | 290 | 1624 | 20 | 3 | 5 |
| 377 | 20261503 | 183 | 692 | 14 | 4 | 2 |
| 378 | 20261504 | 234 | 1348 | 25 | 12 | 3 |
| 379 | 20261505 | 223 | 1119 | 19 | 6 | 2 |
| 380 | 20261506 | 379 | 2123 | 50 | 9 | 4 |
| 381 | 20261507 | 241 | 1162 | 20 | 5 | 1 |
| 382 | 20261508 | 257 | 1237 | 19 | 4 | 2 |
| 383 | 20261509 | 188 | 1005 | 16 | 6 | 2 |
| 384 | 20261510 | 234 | 1105 | 27 | 2 | 3 |
| 385 | 20261511 | 216 | 1029 | 35 | 6 | 4 |
| 386 | 20261512 | 189 | 936 | 10 | 6 | 2 |
| 387 | 20261513 | 220 | 1063 | 27 | 3 | 2 |
| 388 | 20261514 | 246 | 1311 | 26 | 2 | 5 |
| 389 | 20261515 | 223 | 1018 | 9 | 6 | 4 |
| 390 | 20261516 | 266 | 1864 | 21 | 11 | 3 |
| 391 | 20261517 | 217 | 1019 | 14 | 6 | 2 |
| 392 | 20261518 | 271 | 1464 | 18 | 5 | 0 |
| 393 | 20261519 | 288 | 1076 | 32 | 6 | 2 |
| 394 | 20261520 | 232 | 878 | 16 | 2 | 4 |
| 395 | 20261521 | 206 | 964 | 23 | 6 | 2 |
| 396 | 20261522 | 206 | 1119 | 26 | 8 | 3 |
| 397 | 20261523 | 217 | 1162 | 25 | 6 | 2 |
| 398 | 20261524 | 214 | 751 | 20 | 8 | 2 |
| 399 | 20261525 | 220 | 986 | 24 | 5 | 4 |
| 400 | 20261526 | 256 | 1031 | 16 | 2 | 5 |
| 401 | 20261527 | 260 | 1464 | 40 | 6 | 0 |
| 402 | 20261528 | 308 | 1111 | 29 | 3 | 1 |
| 403 | 20261529 | 229 | 969 | 22 | 7 | 1 |
| 404 | 20261530 | 201 | 931 | 18 | 4 | 2 |
| 405 | 20261531 | 238 | 1190 | 26 | 1 | 3 |
| 406 | 20261532 | 240 | 1182 | 20 | 6 | 2 |
| 407 | 20261533 | 188 | 880 | 7 | 2 | 1 |
| 408 | 20261534 | 195 | 942 | 18 | 10 | 0 |
| 409 | 20261535 | 249 | 1274 | 29 | 3 | 2 |
| 410 | 20261536 | 231 | 1467 | 20 | 2 | 2 |
| 411 | 20261537 | 233 | 1177 | 26 | 8 | 3 |
| 412 | 20261538 | 252 | 1267 | 24 | 8 | 2 |
| 413 | 20261539 | 257 | 1060 | 32 | 4 | 3 |
| 414 | 20261540 | 283 | 1230 | 27 | 2 | 4 |
| 415 | 20261541 | 221 | 1226 | 21 | 2 | 1 |
| 416 | 20261542 | 232 | 1185 | 29 | 2 | 3 |
| 417 | 20261543 | 232 | 982 | 17 | 2 | 3 |
| 418 | 20261544 | 230 | 1183 | 27 | 2 | 2 |
| 419 | 20261545 | 191 | 924 | 8 | 5 | 3 |
| 420 | 20261546 | 258 | 1253 | 35 | 9 | 3 |
| 421 | 20261547 | 225 | 1289 | 14 | 0 | 1 |
| 422 | 20261548 | 227 | 1088 | 23 | 2 | 1 |
| 423 | 20261549 | 308 | 2256 | 24 | 8 | 1 |
| 424 | 20261550 | 272 | 1627 | 37 | 1 | 5 |
| 425 | 20261551 | 270 | 1330 | 31 | 3 | 2 |
| 426 | 20261552 | 299 | 1877 | 38 | 5 | 3 |
| 427 | 20261553 | 311 | 2173 | 25 | 1 | 3 |
| 428 | 20261554 | 243 | 1340 | 29 | 3 | 3 |
| 429 | 20261555 | 251 | 1233 | 22 | 6 | 4 |
| 430 | 20261556 | 257 | 1119 | 15 | 2 | 4 |
| 431 | 20261557 | 216 | 979 | 27 | 5 | 5 |
| 432 | 20261558 | 271 | 1553 | 24 | 3 | 2 |
| 433 | 20261559 | 247 | 1342 | 24 | 5 | 2 |
| 434 | 20261560 | 201 | 1336 | 19 | 0 | 2 |
| 435 | 20261561 | 363 | 2299 | 39 | 4 | 2 |
| 436 | 20261562 | 345 | 1483 | 31 | 4 | 4 |
| 437 | 20261563 | 202 | 980 | 16 | 7 | 3 |
| 438 | 20261564 | 238 | 1190 | 11 | 3 | 3 |
| 439 | 20261565 | 248 | 1268 | 22 | 6 | 2 |
| 440 | 20261566 | 259 | 1054 | 22 | 9 | 3 |
| 441 | 20261567 | 211 | 915 | 19 | 9 | 3 |
| 442 | 20261568 | 264 | 1413 | 21 | 7 | 0 |
| 443 | 20261569 | 214 | 1061 | 25 | 4 | 5 |
| 444 | 20261570 | 265 | 1367 | 22 | 6 | 3 |
| 445 | 20261571 | 233 | 1249 | 31 | 12 | 2 |
| 446 | 20261572 | 258 | 1421 | 34 | 11 | 1 |
| 447 | 20261573 | 288 | 1750 | 50 | 2 | 3 |
| 448 | 20261574 | 226 | 979 | 18 | 4 | 2 |
| 449 | 20261575 | 325 | 1743 | 26 | 6 | 3 |
| 450 | 20261576 | 276 | 1421 | 27 | 6 | 5 |
| 451 | 20261577 | 215 | 1003 | 22 | 4 | 4 |
| 452 | 20261578 | 426 | 2577 | 34 | 8 | 4 |
| 453 | 20261579 | 207 | 900 | 22 | 4 | 2 |
| 454 | 20261580 | 219 | 894 | 16 | 6 | 3 |
| 455 | 20261581 | 179 | 854 | 17 | 5 | 1 |
| 456 | 20261582 | 202 | 1109 | 10 | 3 | 1 |
| 457 | 20261583 | 272 | 1312 | 15 | 6 | 4 |
| 458 | 20261584 | 210 | 1187 | 26 | 5 | 1 |
| 459 | 20261585 | 206 | 1065 | 29 | 1 | 0 |
| 460 | 20261586 | 179 | 920 | 14 | 2 | 1 |
| 461 | 20261587 | 251 | 1321 | 20 | 2 | 2 |
| 462 | 20261588 | 320 | 1550 | 25 | 0 | 4 |
| 463 | 20261589 | 254 | 1390 | 19 | 3 | 3 |
| 464 | 20261590 | 202 | 976 | 14 | 3 | 3 |
| 465 | 20261591 | 284 | 1676 | 31 | 0 | 3 |
| 466 | 20261592 | 248 | 1581 | 22 | 3 | 3 |
| 467 | 20261593 | 241 | 1583 | 23 | 6 | 2 |
| 468 | 20261594 | 234 | 1281 | 20 | 2 | 4 |
| 469 | 20261595 | 259 | 1200 | 24 | 4 | 3 |
| 470 | 20261596 | 213 | 894 | 23 | 5 | 3 |
| 471 | 20261597 | 412 | 2748 | 35 | 1 | 3 |
| 472 | 20261598 | 259 | 1213 | 28 | 10 | 2 |
| 473 | 20261599 | 292 | 1578 | 27 | 1 | 5 |
| 474 | 20261600 | 242 | 1169 | 22 | 1 | 2 |
| 475 | 20261601 | 500 | 2410 | 52 | 6 | 5 |
| 476 | 20261602 | 180 | 772 | 17 | 2 | 3 |
| 477 | 20261603 | 293 | 1259 | 34 | 2 | 4 |
| 478 | 20261604 | 242 | 1172 | 15 | 5 | 5 |
| 479 | 20261605 | 240 | 1437 | 29 | 4 | 3 |
| 480 | 20261606 | 246 | 1342 | 29 | 8 | 2 |
| 481 | 20261607 | 214 | 904 | 19 | 6 | 2 |
| 482 | 20261608 | 332 | 1640 | 26 | 3 | 3 |
| 483 | 20261609 | 218 | 1011 | 17 | 3 | 1 |
| 484 | 20261610 | 204 | 1023 | 26 | 3 | 1 |
| 485 | 20261611 | 277 | 1302 | 29 | 10 | 5 |
| 486 | 20261612 | 189 | 745 | 15 | 1 | 3 |
| 487 | 20261613 | 258 | 1134 | 19 | 2 | 4 |
| 488 | 20261614 | 329 | 1608 | 30 | 5 | 2 |
| 489 | 20261615 | 206 | 971 | 11 | 9 | 2 |
| 490 | 20261616 | 245 | 1245 | 35 | 5 | 2 |
| 491 | 20261617 | 252 | 1208 | 16 | 2 | 4 |
| 492 | 20261618 | 268 | 1342 | 32 | 7 | 2 |
| 493 | 20261619 | 232 | 1087 | 31 | 5 | 3 |
| 494 | 20261620 | 174 | 900 | 16 | 1 | 0 |
| 495 | 20261621 | 259 | 1364 | 31 | 0 | 1 |
| 496 | 20261622 | 197 | 826 | 13 | 4 | 2 |
| 497 | 20261623 | 252 | 1415 | 25 | 2 | 2 |
| 498 | 20261624 | 214 | 932 | 16 | 8 | 2 |
| 499 | 20261625 | 272 | 1263 | 22 | 12 | 2 |
| 500 | 20261626 | 276 | 990 | 28 | 0 | 3 |
| 501 | 20261627 | 350 | 1804 | 30 | 2 | 4 |
| 502 | 20261628 | 268 | 1134 | 18 | 3 | 4 |
| 503 | 20261629 | 198 | 849 | 18 | 2 | 1 |
| 504 | 20261630 | 276 | 1417 | 16 | 6 | 5 |
| 505 | 20261631 | 252 | 1236 | 18 | 1 | 1 |
| 506 | 20261632 | 216 | 963 | 26 | 3 | 3 |
| 507 | 20261633 | 242 | 1415 | 20 | 7 | 1 |
| 508 | 20261634 | 192 | 1079 | 20 | 3 | 1 |
| 509 | 20261635 | 244 | 1204 | 26 | 6 | 3 |
| 510 | 20261636 | 257 | 1133 | 26 | 4 | 2 |
| 511 | 20261637 | 245 | 1341 | 23 | 2 | 3 |
| 512 | 20261638 | 273 | 1922 | 32 | 1 | 0 |
| 513 | 20261639 | 236 | 1149 | 24 | 5 | 2 |
| 514 | 20261640 | 255 | 1460 | 22 | 1 | 1 |
| 515 | 20261641 | 277 | 1530 | 27 | 2 | 5 |
| 516 | 20261642 | 227 | 1304 | 14 | 7 | 2 |
| 517 | 20261643 | 253 | 1213 | 18 | 3 | 5 |
| 518 | 20261644 | 248 | 867 | 13 | 0 | 2 |
| 519 | 20261645 | 176 | 949 | 16 | 3 | 4 |
| 520 | 20261646 | 270 | 1146 | 29 | 8 | 5 |
| 521 | 20261647 | 212 | 840 | 25 | 5 | 2 |
| 522 | 20261648 | 315 | 1880 | 19 | 8 | 4 |
| 523 | 20261649 | 231 | 1273 | 28 | 9 | 3 |
| 524 | 20261650 | 199 | 1135 | 19 | 5 | 1 |
| 525 | 20261651 | 294 | 1521 | 28 | 5 | 3 |
| 526 | 20261652 | 233 | 1399 | 11 | 5 | 4 |
| 527 | 20261653 | 219 | 1278 | 22 | 0 | 3 |
| 528 | 20261654 | 290 | 1150 | 18 | 5 | 2 |
| 529 | 20261655 | 351 | 2118 | 35 | 8 | 4 |
| 530 | 20261656 | 197 | 888 | 21 | 6 | 2 |
| 531 | 20261657 | 258 | 1058 | 17 | 3 | 3 |
| 532 | 20261658 | 184 | 732 | 10 | 3 | 1 |
| 533 | 20261659 | 255 | 1220 | 23 | 1 | 1 |
| 534 | 20261660 | 210 | 676 | 21 | 6 | 3 |
| 535 | 20261661 | 195 | 861 | 24 | 5 | 2 |
| 536 | 20261662 | 218 | 1031 | 19 | 0 | 2 |
| 537 | 20261663 | 212 | 1063 | 28 | 6 | 2 |
| 538 | 20261664 | 206 | 1078 | 19 | 3 | 3 |
| 539 | 20261665 | 362 | 1918 | 60 | 11 | 4 |
| 540 | 20261666 | 291 | 1358 | 32 | 7 | 3 |
| 541 | 20261667 | 230 | 1109 | 34 | 1 | 4 |
| 542 | 20261668 | 267 | 1277 | 18 | 5 | 2 |
| 543 | 20261669 | 169 | 784 | 16 | 4 | 1 |
| 544 | 20261670 | 227 | 957 | 19 | 8 | 4 |
| 545 | 20261671 | 222 | 1029 | 24 | 7 | 4 |
| 546 | 20261672 | 234 | 880 | 18 | 5 | 4 |
| 547 | 20261673 | 258 | 1628 | 32 | 2 | 1 |
| 548 | 20261674 | 289 | 1624 | 42 | 4 | 5 |
| 549 | 20261675 | 220 | 1302 | 27 | 5 | 2 |
| 550 | 20261676 | 230 | 1239 | 9 | 4 | 4 |
| 551 | 20261677 | 276 | 1067 | 28 | 1 | 2 |
| 552 | 20261678 | 285 | 1861 | 21 | 5 | 1 |
| 553 | 20261679 | 194 | 995 | 12 | 5 | 2 |
| 554 | 20261680 | 212 | 1138 | 29 | 6 | 4 |
| 555 | 20261681 | 317 | 1597 | 27 | 9 | 5 |
| 556 | 20261682 | 260 | 945 | 20 | 8 | 2 |
| 557 | 20261683 | 191 | 977 | 20 | 0 | 4 |
| 558 | 20261684 | 170 | 762 | 14 | 1 | 1 |
| 559 | 20261685 | 195 | 806 | 14 | 4 | 2 |
| 560 | 20261686 | 254 | 1329 | 35 | 3 | 3 |
| 561 | 20261687 | 260 | 1377 | 37 | 3 | 3 |
| 562 | 20261688 | 321 | 1676 | 29 | 3 | 2 |
| 563 | 20261689 | 253 | 1583 | 26 | 3 | 1 |
| 564 | 20261690 | 294 | 1351 | 13 | 6 | 4 |
| 565 | 20261691 | 279 | 1028 | 29 | 5 | 5 |
| 566 | 20261692 | 193 | 1151 | 26 | 3 | 1 |
| 567 | 20261693 | 289 | 1713 | 12 | 3 | 4 |
| 568 | 20261694 | 169 | 574 | 14 | 2 | 3 |
| 569 | 20261695 | 258 | 905 | 22 | 4 | 3 |
| 570 | 20261696 | 366 | 1914 | 49 | 6 | 5 |
| 571 | 20261697 | 321 | 1785 | 15 | 6 | 4 |
| 572 | 20261698 | 209 | 932 | 20 | 7 | 2 |
| 573 | 20261699 | 227 | 1013 | 29 | 1 | 3 |
| 574 | 20261700 | 206 | 898 | 23 | 4 | 4 |
| 575 | 20261701 | 301 | 1451 | 16 | 2 | 3 |
| 576 | 20261702 | 278 | 1489 | 33 | 3 | 4 |
| 577 | 20261703 | 176 | 910 | 19 | 1 | 1 |
| 578 | 20261704 | 230 | 1152 | 28 | 7 | 3 |
| 579 | 20261705 | 187 | 972 | 17 | 4 | 3 |
| 580 | 20261706 | 307 | 1732 | 42 | 3 | 3 |
| 581 | 20261707 | 196 | 844 | 25 | 2 | 3 |
| 582 | 20261708 | 213 | 1025 | 17 | 2 | 2 |
| 583 | 20261709 | 237 | 1277 | 22 | 1 | 3 |
| 584 | 20261710 | 285 | 1231 | 26 | 9 | 4 |
| 585 | 20261711 | 267 | 1507 | 33 | 11 | 4 |
| 586 | 20261712 | 272 | 1393 | 24 | 2 | 2 |
| 587 | 20261713 | 294 | 1454 | 25 | 2 | 3 |
| 588 | 20261714 | 251 | 1415 | 26 | 3 | 4 |
| 589 | 20261715 | 200 | 755 | 11 | 4 | 2 |
| 590 | 20261716 | 365 | 1812 | 23 | 3 | 4 |
| 591 | 20261717 | 222 | 1178 | 15 | 3 | 3 |
| 592 | 20261718 | 224 | 1191 | 23 | 4 | 2 |
| 593 | 20261719 | 317 | 1652 | 42 | 2 | 4 |
| 594 | 20261720 | 236 | 1534 | 15 | 5 | 1 |
| 595 | 20261721 | 183 | 831 | 15 | 4 | 3 |
| 596 | 20261722 | 245 | 1296 | 29 | 1 | 2 |
| 597 | 20261723 | 265 | 1351 | 44 | 6 | 4 |
| 598 | 20261724 | 294 | 1559 | 27 | 2 | 5 |
| 599 | 20261725 | 279 | 1722 | 26 | 4 | 1 |
| 600 | 20261726 | 266 | 1238 | 41 | 5 | 2 |
| 601 | 20261727 | 330 | 1576 | 34 | 8 | 2 |
| 602 | 20261728 | 296 | 1753 | 32 | 2 | 2 |
| 603 | 20261729 | 218 | 919 | 16 | 3 | 3 |
| 604 | 20261730 | 245 | 1169 | 30 | 3 | 4 |
| 605 | 20261731 | 256 | 1356 | 20 | 4 | 3 |
| 606 | 20261732 | 190 | 926 | 15 | 5 | 1 |
| 607 | 20261733 | 278 | 1146 | 30 | 3 | 2 |
| 608 | 20261734 | 233 | 979 | 20 | 3 | 1 |
| 609 | 20261735 | 255 | 1512 | 28 | 11 | 2 |
| 610 | 20261736 | 177 | 486 | 6 | 1 | 2 |
| 611 | 20261737 | 235 | 1222 | 25 | 2 | 4 |
| 612 | 20261738 | 233 | 1261 | 31 | 3 | 1 |
| 613 | 20261739 | 225 | 1152 | 20 | 3 | 2 |
| 614 | 20261740 | 207 | 1101 | 19 | 0 | 4 |
| 615 | 20261741 | 345 | 1875 | 35 | 20 | 5 |
| 616 | 20261742 | 227 | 1134 | 24 | 5 | 3 |
| 617 | 20261743 | 210 | 1118 | 14 | 5 | 2 |
| 618 | 20261744 | 189 | 815 | 16 | 3 | 4 |
| 619 | 20261745 | 221 | 930 | 17 | 2 | 3 |
| 620 | 20261746 | 263 | 1196 | 18 | 4 | 4 |
| 621 | 20261747 | 241 | 1071 | 16 | 6 | 3 |
| 622 | 20261748 | 269 | 1266 | 34 | 4 | 3 |
| 623 | 20261749 | 231 | 1276 | 20 | 7 | 2 |
| 624 | 20261750 | 203 | 971 | 20 | 2 | 3 |
| 625 | 20261751 | 282 | 1546 | 26 | 5 | 5 |
| 626 | 20261752 | 262 | 1395 | 29 | 7 | 3 |
| 627 | 20261753 | 273 | 1368 | 13 | 7 | 2 |
| 628 | 20261754 | 229 | 1021 | 11 | 2 | 5 |
| 629 | 20261755 | 232 | 1315 | 26 | 0 | 1 |
| 630 | 20261756 | 184 | 875 | 13 | 1 | 2 |
| 631 | 20261757 | 242 | 1415 | 24 | 4 | 1 |
| 632 | 20261758 | 316 | 1986 | 29 | 8 | 3 |
| 633 | 20261759 | 275 | 1239 | 16 | 8 | 2 |
| 634 | 20261760 | 232 | 895 | 15 | 0 | 3 |
| 635 | 20261761 | 250 | 1154 | 28 | 1 | 4 |
| 636 | 20261762 | 208 | 660 | 11 | 2 | 3 |
| 637 | 20261763 | 169 | 676 | 11 | 4 | 1 |
| 638 | 20261764 | 269 | 1459 | 31 | 2 | 2 |
| 639 | 20261765 | 293 | 1531 | 33 | 5 | 2 |
| 640 | 20261766 | 356 | 1764 | 26 | 1 | 3 |
| 641 | 20261767 | 251 | 1629 | 41 | 8 | 2 |
| 642 | 20261768 | 248 | 1194 | 20 | 6 | 2 |
| 643 | 20261769 | 281 | 1582 | 22 | 2 | 3 |
| 644 | 20261770 | 208 | 950 | 18 | 11 | 3 |
| 645 | 20261771 | 386 | 1777 | 34 | 5 | 5 |
| 646 | 20261772 | 226 | 1252 | 15 | 10 | 3 |
| 647 | 20261773 | 183 | 741 | 20 | 5 | 2 |
| 648 | 20261774 | 242 | 1124 | 25 | 3 | 2 |
| 649 | 20261775 | 289 | 1517 | 24 | 1 | 5 |
| 650 | 20261776 | 279 | 1590 | 34 | 4 | 4 |
| 651 | 20261777 | 322 | 1526 | 33 | 5 | 1 |
| 652 | 20261778 | 223 | 1071 | 23 | 4 | 3 |
| 653 | 20261779 | 250 | 1245 | 27 | 4 | 3 |
| 654 | 20261780 | 229 | 1037 | 14 | 3 | 2 |
| 655 | 20261781 | 229 | 1290 | 30 | 3 | 3 |
| 656 | 20261782 | 253 | 1131 | 30 | 5 | 5 |
| 657 | 20261783 | 237 | 1358 | 23 | 11 | 0 |
| 658 | 20261784 | 319 | 1603 | 46 | 8 | 2 |
| 659 | 20261785 | 387 | 1948 | 48 | 2 | 5 |
| 660 | 20261786 | 174 | 895 | 18 | 3 | 1 |
| 661 | 20261787 | 276 | 1489 | 33 | 3 | 1 |
| 662 | 20261788 | 281 | 1532 | 30 | 1 | 4 |
| 663 | 20261789 | 235 | 1058 | 22 | 5 | 3 |
| 664 | 20261790 | 172 | 868 | 12 | 3 | 1 |
| 665 | 20261791 | 224 | 1157 | 15 | 3 | 3 |
| 666 | 20261792 | 232 | 1146 | 19 | 2 | 3 |
| 667 | 20261793 | 209 | 791 | 17 | 3 | 1 |
| 668 | 20261794 | 500 | 2453 | 66 | 9 | 5 |
| 669 | 20261795 | 343 | 1718 | 22 | 5 | 5 |
| 670 | 20261796 | 234 | 1073 | 18 | 5 | 3 |
| 671 | 20261797 | 276 | 1647 | 30 | 5 | 3 |
| 672 | 20261798 | 171 | 875 | 18 | 6 | 2 |
| 673 | 20261799 | 180 | 651 | 16 | 3 | 2 |
| 674 | 20261800 | 210 | 796 | 18 | 6 | 1 |
| 675 | 20261801 | 178 | 725 | 13 | 3 | 2 |
| 676 | 20261802 | 201 | 1014 | 17 | 2 | 2 |
| 677 | 20261803 | 261 | 1539 | 29 | 5 | 1 |
| 678 | 20261804 | 254 | 1478 | 32 | 4 | 1 |
| 679 | 20261805 | 229 | 1137 | 14 | 2 | 3 |
| 680 | 20261806 | 208 | 1056 | 23 | 0 | 3 |
| 681 | 20261807 | 241 | 1404 | 25 | 1 | 1 |
| 682 | 20261808 | 238 | 1172 | 22 | 7 | 2 |
| 683 | 20261809 | 233 | 1334 | 22 | 12 | 1 |
| 684 | 20261810 | 217 | 861 | 27 | 3 | 2 |
| 685 | 20261811 | 361 | 1759 | 36 | 4 | 3 |
| 686 | 20261812 | 228 | 829 | 21 | 7 | 2 |
| 687 | 20261813 | 230 | 1017 | 23 | 1 | 3 |
| 688 | 20261814 | 281 | 1331 | 15 | 8 | 4 |
| 689 | 20261815 | 220 | 999 | 19 | 1 | 2 |
| 690 | 20261816 | 291 | 1477 | 36 | 2 | 3 |
| 691 | 20261817 | 357 | 1774 | 22 | 10 | 4 |
| 692 | 20261818 | 287 | 1786 | 30 | 4 | 2 |
| 693 | 20261819 | 301 | 1250 | 28 | 3 | 5 |
| 694 | 20261820 | 226 | 842 | 15 | 5 | 2 |
| 695 | 20261821 | 185 | 623 | 17 | 5 | 3 |
| 696 | 20261822 | 267 | 1276 | 30 | 6 | 3 |
| 697 | 20261823 | 202 | 1003 | 25 | 5 | 3 |
| 698 | 20261824 | 196 | 710 | 15 | 5 | 2 |
| 699 | 20261825 | 192 | 1034 | 19 | 5 | 1 |
| 700 | 20261826 | 246 | 1354 | 32 | 3 | 4 |
| 701 | 20261827 | 236 | 1124 | 26 | 0 | 2 |
| 702 | 20261828 | 365 | 1881 | 36 | 4 | 2 |
| 703 | 20261829 | 319 | 1413 | 19 | 4 | 3 |
| 704 | 20261830 | 248 | 1384 | 21 | 2 | 3 |
| 705 | 20261831 | 306 | 1793 | 33 | 3 | 4 |
| 706 | 20261832 | 280 | 1482 | 24 | 2 | 2 |
| 707 | 20261833 | 173 | 905 | 11 | 2 | 2 |
| 708 | 20261834 | 240 | 1099 | 14 | 7 | 0 |
| 709 | 20261835 | 224 | 968 | 15 | 10 | 3 |
| 710 | 20261836 | 280 | 1282 | 26 | 2 | 1 |
| 711 | 20261837 | 263 | 1637 | 30 | 6 | 4 |
| 712 | 20261838 | 164 | 634 | 8 | 6 | 1 |
| 713 | 20261839 | 253 | 1379 | 22 | 3 | 2 |
| 714 | 20261840 | 232 | 982 | 11 | 1 | 4 |
| 715 | 20261841 | 210 | 1130 | 17 | 4 | 2 |
| 716 | 20261842 | 276 | 1507 | 24 | 5 | 2 |
| 717 | 20261843 | 244 | 1268 | 22 | 7 | 3 |
| 718 | 20261844 | 227 | 1038 | 20 | 7 | 2 |
| 719 | 20261845 | 228 | 1207 | 30 | 3 | 4 |
| 720 | 20261846 | 189 | 777 | 17 | 2 | 2 |
| 721 | 20261847 | 187 | 1024 | 17 | 6 | 2 |
| 722 | 20261848 | 222 | 722 | 22 | 2 | 4 |
| 723 | 20261849 | 242 | 1232 | 35 | 7 | 1 |
| 724 | 20261850 | 247 | 1273 | 24 | 7 | 2 |
| 725 | 20261851 | 268 | 1316 | 33 | 3 | 2 |
| 726 | 20261852 | 226 | 1202 | 23 | 6 | 2 |
| 727 | 20261853 | 219 | 1129 | 24 | 2 | 2 |
| 728 | 20261854 | 279 | 1514 | 36 | 6 | 4 |
| 729 | 20261855 | 249 | 1670 | 40 | 2 | 1 |
| 730 | 20261856 | 258 | 1287 | 36 | 4 | 3 |
| 731 | 20261857 | 195 | 914 | 12 | 3 | 1 |
| 732 | 20261858 | 244 | 1410 | 17 | 1 | 3 |
| 733 | 20261859 | 211 | 1110 | 10 | 0 | 1 |
| 734 | 20261860 | 206 | 1044 | 17 | 7 | 1 |
| 735 | 20261861 | 230 | 1318 | 22 | 3 | 3 |
| 736 | 20261862 | 362 | 2109 | 52 | 7 | 3 |
| 737 | 20261863 | 342 | 1883 | 29 | 2 | 6 |
| 738 | 20261864 | 271 | 1250 | 35 | 2 | 4 |
| 739 | 20261865 | 379 | 1775 | 42 | 2 | 4 |
| 740 | 20261866 | 191 | 798 | 14 | 2 | 1 |
| 741 | 20261867 | 258 | 1275 | 25 | 6 | 3 |
| 742 | 20261868 | 287 | 1718 | 28 | 6 | 1 |
| 743 | 20261869 | 254 | 1299 | 32 | 3 | 3 |
| 744 | 20261870 | 234 | 825 | 18 | 2 | 4 |
| 745 | 20261871 | 207 | 1147 | 14 | 3 | 2 |
| 746 | 20261872 | 240 | 1239 | 20 | 2 | 2 |
| 747 | 20261873 | 196 | 1049 | 26 | 3 | 3 |
| 748 | 20261874 | 332 | 2194 | 23 | 5 | 4 |
| 749 | 20261875 | 277 | 1182 | 26 | 2 | 2 |
| 750 | 20261876 | 294 | 1725 | 48 | 4 | 4 |
| 751 | 20261877 | 159 | 644 | 10 | 1 | 1 |
| 752 | 20261878 | 189 | 844 | 12 | 9 | 1 |
| 753 | 20261879 | 290 | 1674 | 27 | 4 | 4 |
| 754 | 20261880 | 272 | 1675 | 25 | 7 | 4 |
| 755 | 20261881 | 290 | 1253 | 34 | 4 | 4 |
| 756 | 20261882 | 195 | 1177 | 15 | 1 | 1 |
| 757 | 20261883 | 218 | 891 | 23 | 2 | 1 |
| 758 | 20261884 | 314 | 1635 | 43 | 7 | 2 |
| 759 | 20261885 | 275 | 1424 | 31 | 7 | 2 |
| 760 | 20261886 | 224 | 1130 | 31 | 1 | 3 |
| 761 | 20261887 | 234 | 1016 | 20 | 3 | 3 |
| 762 | 20261888 | 217 | 1067 | 26 | 1 | 2 |
| 763 | 20261889 | 247 | 1390 | 32 | 4 | 4 |
| 764 | 20261890 | 287 | 1214 | 20 | 5 | 6 |
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
| 776 | 20261902 | 250 | 1806 | 21 | 15 | 0 |
| 777 | 20261903 | 359 | 1790 | 35 | 8 | 4 |
| 778 | 20261904 | 216 | 1275 | 25 | 2 | 1 |
| 779 | 20261905 | 207 | 928 | 22 | 2 | 2 |
| 780 | 20261906 | 284 | 1383 | 18 | 4 | 2 |
| 781 | 20261907 | 232 | 1100 | 27 | 9 | 3 |
| 782 | 20261908 | 228 | 1122 | 18 | 8 | 3 |
| 783 | 20261909 | 162 | 816 | 15 | 3 | 2 |
| 784 | 20261910 | 248 | 1238 | 33 | 5 | 4 |
| 785 | 20261911 | 245 | 1211 | 29 | 1 | 4 |
| 786 | 20261912 | 355 | 1588 | 32 | 1 | 4 |
| 787 | 20261913 | 218 | 680 | 11 | 11 | 1 |
| 788 | 20261914 | 256 | 1396 | 17 | 6 | 3 |
| 789 | 20261915 | 257 | 1150 | 25 | 3 | 3 |
| 790 | 20261916 | 213 | 977 | 19 | 2 | 1 |
| 791 | 20261917 | 242 | 1268 | 31 | 2 | 2 |
| 792 | 20261918 | 259 | 855 | 18 | 6 | 2 |
| 793 | 20261919 | 235 | 1230 | 17 | 4 | 1 |
| 794 | 20261920 | 262 | 1447 | 22 | 2 | 4 |
| 795 | 20261921 | 238 | 1043 | 18 | 3 | 2 |
| 796 | 20261922 | 239 | 1170 | 26 | 7 | 5 |
| 797 | 20261923 | 249 | 1353 | 23 | 2 | 3 |
| 798 | 20261924 | 260 | 1396 | 34 | 4 | 3 |
| 799 | 20261925 | 259 | 1254 | 32 | 1 | 3 |
| 800 | 20261926 | 294 | 1557 | 25 | 2 | 3 |
| 801 | 20261927 | 249 | 1161 | 25 | 7 | 2 |
| 802 | 20261928 | 241 | 1209 | 19 | 2 | 4 |
| 803 | 20261929 | 225 | 1027 | 18 | 0 | 2 |
| 804 | 20261930 | 271 | 1274 | 17 | 2 | 4 |
| 805 | 20261931 | 266 | 1190 | 22 | 5 | 4 |
| 806 | 20261932 | 263 | 1513 | 22 | 1 | 4 |
| 807 | 20261933 | 220 | 1182 | 31 | 8 | 3 |
| 808 | 20261934 | 225 | 1102 | 12 | 3 | 3 |
| 809 | 20261935 | 287 | 1647 | 23 | 1 | 1 |
| 810 | 20261936 | 242 | 1239 | 26 | 3 | 2 |
| 811 | 20261937 | 244 | 1477 | 12 | 1 | 2 |
| 812 | 20261938 | 241 | 1318 | 29 | 3 | 2 |
| 813 | 20261939 | 206 | 752 | 23 | 2 | 3 |
| 814 | 20261940 | 254 | 1314 | 23 | 3 | 2 |
| 815 | 20261941 | 232 | 1149 | 31 | 6 | 3 |
| 816 | 20261942 | 228 | 1306 | 17 | 5 | 3 |
| 817 | 20261943 | 271 | 1175 | 33 | 2 | 4 |
| 818 | 20261944 | 249 | 1510 | 18 | 3 | 5 |
| 819 | 20261945 | 295 | 1752 | 21 | 5 | 5 |
| 820 | 20261946 | 243 | 1227 | 16 | 6 | 3 |
| 821 | 20261947 | 226 | 994 | 26 | 5 | 4 |
| 822 | 20261948 | 250 | 1302 | 12 | 3 | 2 |
| 823 | 20261949 | 211 | 1133 | 17 | 4 | 1 |
| 824 | 20261950 | 339 | 1696 | 27 | 0 | 4 |
| 825 | 20261951 | 295 | 1342 | 38 | 6 | 4 |
| 826 | 20261952 | 213 | 832 | 14 | 1 | 2 |
| 827 | 20261953 | 222 | 1063 | 17 | 5 | 2 |
| 828 | 20261954 | 254 | 1244 | 35 | 1 | 4 |
| 829 | 20261955 | 228 | 1427 | 21 | 7 | 2 |
| 830 | 20261956 | 199 | 822 | 21 | 4 | 4 |
| 831 | 20261957 | 302 | 1532 | 34 | 0 | 4 |
| 832 | 20261958 | 231 | 1183 | 29 | 1 | 3 |
| 833 | 20261959 | 210 | 1028 | 29 | 4 | 3 |
| 834 | 20261960 | 250 | 1070 | 14 | 3 | 4 |
| 835 | 20261961 | 233 | 989 | 17 | 2 | 2 |
| 836 | 20261962 | 254 | 1101 | 24 | 5 | 3 |
| 837 | 20261963 | 237 | 1098 | 21 | 5 | 3 |
| 838 | 20261964 | 194 | 912 | 20 | 4 | 1 |
| 839 | 20261965 | 242 | 1194 | 23 | 3 | 2 |
| 840 | 20261966 | 314 | 1602 | 26 | 4 | 2 |
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
| 851 | 20261977 | 225 | 1429 | 19 | 5 | 1 |
| 852 | 20261978 | 458 | 3124 | 49 | 20 | 2 |
| 853 | 20261979 | 222 | 852 | 15 | 1 | 3 |
| 854 | 20261980 | 243 | 1271 | 21 | 7 | 2 |
| 855 | 20261981 | 189 | 900 | 17 | 5 | 2 |
| 856 | 20261982 | 269 | 1138 | 20 | 3 | 5 |
| 857 | 20261983 | 189 | 847 | 20 | 5 | 1 |
| 858 | 20261984 | 224 | 827 | 22 | 0 | 3 |
| 859 | 20261985 | 228 | 1456 | 20 | 10 | 3 |
| 860 | 20261986 | 233 | 1101 | 18 | 3 | 4 |
| 861 | 20261987 | 325 | 1981 | 47 | 3 | 4 |
| 862 | 20261988 | 227 | 1434 | 18 | 6 | 3 |
| 863 | 20261989 | 189 | 799 | 17 | 4 | 0 |
| 864 | 20261990 | 203 | 937 | 17 | 1 | 3 |
| 865 | 20261991 | 237 | 989 | 32 | 4 | 2 |
| 866 | 20261992 | 260 | 1166 | 30 | 2 | 3 |
| 867 | 20261993 | 241 | 1085 | 16 | 5 | 3 |
| 868 | 20261994 | 258 | 1278 | 9 | 3 | 4 |
| 869 | 20261995 | 227 | 1069 | 17 | 1 | 2 |
| 870 | 20261996 | 232 | 1163 | 20 | 12 | 1 |
| 871 | 20261997 | 161 | 868 | 7 | 6 | 1 |
| 872 | 20261998 | 233 | 1271 | 23 | 2 | 3 |
| 873 | 20261999 | 214 | 967 | 13 | 3 | 2 |
| 874 | 20262000 | 182 | 976 | 10 | 4 | 0 |
| 875 | 20262001 | 221 | 1067 | 21 | 4 | 1 |
| 876 | 20262002 | 279 | 1055 | 17 | 10 | 3 |
| 877 | 20262003 | 230 | 1150 | 14 | 10 | 2 |
| 878 | 20262004 | 227 | 970 | 20 | 1 | 3 |
| 879 | 20262005 | 277 | 1514 | 33 | 6 | 3 |
| 880 | 20262006 | 192 | 784 | 9 | 3 | 3 |
| 881 | 20262007 | 181 | 709 | 14 | 7 | 2 |
| 882 | 20262008 | 196 | 1197 | 10 | 2 | 2 |
| 883 | 20262009 | 500 | 2328 | 67 | 6 | 6 |
| 884 | 20262010 | 305 | 1460 | 8 | 4 | 3 |
| 885 | 20262011 | 250 | 1142 | 15 | 4 | 1 |
| 886 | 20262012 | 239 | 927 | 28 | 7 | 4 |
| 887 | 20262013 | 277 | 1189 | 25 | 5 | 4 |
| 888 | 20262014 | 143 | 508 | 9 | 2 | 2 |
| 889 | 20262015 | 257 | 1506 | 41 | 7 | 3 |
| 890 | 20262016 | 230 | 1165 | 22 | 1 | 3 |
| 891 | 20262017 | 204 | 1025 | 16 | 6 | 2 |
| 892 | 20262018 | 246 | 914 | 11 | 1 | 3 |
| 893 | 20262019 | 235 | 1336 | 25 | 2 | 3 |
| 894 | 20262020 | 255 | 1211 | 29 | 4 | 3 |
| 895 | 20262021 | 219 | 1126 | 23 | 5 | 2 |
| 896 | 20262022 | 244 | 828 | 18 | 5 | 4 |
| 897 | 20262023 | 263 | 1360 | 31 | 4 | 2 |
| 898 | 20262024 | 434 | 2221 | 38 | 7 | 2 |
| 899 | 20262025 | 253 | 1039 | 18 | 5 | 4 |
| 900 | 20262026 | 263 | 1102 | 29 | 0 | 5 |
| 901 | 20262027 | 343 | 1670 | 26 | 9 | 3 |
| 902 | 20262028 | 209 | 757 | 13 | 3 | 3 |
| 903 | 20262029 | 331 | 2053 | 26 | 6 | 4 |
| 904 | 20262030 | 253 | 1267 | 31 | 6 | 2 |
| 905 | 20262031 | 387 | 2771 | 36 | 4 | 4 |
| 906 | 20262032 | 275 | 1205 | 22 | 7 | 3 |
| 907 | 20262033 | 254 | 1176 | 34 | 4 | 2 |
| 908 | 20262034 | 243 | 1091 | 25 | 5 | 4 |
| 909 | 20262035 | 291 | 1587 | 48 | 1 | 2 |
| 910 | 20262036 | 261 | 1584 | 22 | 4 | 3 |
| 911 | 20262037 | 234 | 1294 | 22 | 5 | 1 |
| 912 | 20262038 | 399 | 1971 | 61 | 4 | 4 |
| 913 | 20262039 | 296 | 1941 | 29 | 1 | 3 |
| 914 | 20262040 | 177 | 945 | 11 | 7 | 2 |
| 915 | 20262041 | 296 | 1589 | 19 | 4 | 4 |
| 916 | 20262042 | 219 | 1087 | 28 | 4 | 2 |
| 917 | 20262043 | 262 | 1326 | 38 | 8 | 2 |
| 918 | 20262044 | 314 | 1812 | 24 | 6 | 6 |
| 919 | 20262045 | 271 | 1516 | 43 | 5 | 5 |
| 920 | 20262046 | 233 | 1420 | 24 | 1 | 2 |
| 921 | 20262047 | 240 | 942 | 12 | 4 | 2 |
| 922 | 20262048 | 239 | 1025 | 22 | 3 | 3 |
| 923 | 20262049 | 269 | 1160 | 26 | 3 | 2 |
| 924 | 20262050 | 208 | 1170 | 20 | 4 | 2 |
| 925 | 20262051 | 218 | 892 | 22 | 1 | 2 |
| 926 | 20262052 | 350 | 2281 | 19 | 2 | 2 |
| 927 | 20262053 | 222 | 1155 | 6 | 2 | 4 |
| 928 | 20262054 | 188 | 820 | 10 | 2 | 1 |
| 929 | 20262055 | 167 | 821 | 10 | 2 | 2 |
| 930 | 20262056 | 257 | 1314 | 31 | 5 | 3 |
| 931 | 20262057 | 253 | 917 | 22 | 1 | 3 |
| 932 | 20262058 | 231 | 1243 | 26 | 1 | 3 |
| 933 | 20262059 | 190 | 722 | 16 | 4 | 3 |
| 934 | 20262060 | 309 | 1863 | 36 | 2 | 1 |
| 935 | 20262061 | 230 | 955 | 20 | 1 | 4 |
| 936 | 20262062 | 276 | 1634 | 32 | 3 | 2 |
| 937 | 20262063 | 261 | 1098 | 33 | 6 | 3 |
| 938 | 20262064 | 493 | 2876 | 51 | 2 | 4 |
| 939 | 20262065 | 296 | 1529 | 28 | 4 | 5 |
| 940 | 20262066 | 289 | 1581 | 37 | 10 | 3 |
| 941 | 20262067 | 241 | 1415 | 36 | 0 | 2 |
| 942 | 20262068 | 306 | 1557 | 40 | 9 | 4 |
| 943 | 20262069 | 250 | 1238 | 19 | 1 | 3 |
| 944 | 20262070 | 224 | 1120 | 26 | 2 | 2 |
| 945 | 20262071 | 226 | 1308 | 25 | 2 | 2 |
| 946 | 20262072 | 161 | 774 | 10 | 5 | 3 |
| 947 | 20262073 | 269 | 1138 | 9 | 2 | 3 |
| 948 | 20262074 | 193 | 1030 | 15 | 0 | 1 |
| 949 | 20262075 | 263 | 1171 | 17 | 10 | 2 |
| 950 | 20262076 | 258 | 1376 | 23 | 6 | 2 |
| 951 | 20262077 | 306 | 1838 | 36 | 5 | 1 |
| 952 | 20262078 | 214 | 746 | 12 | 4 | 4 |
| 953 | 20262079 | 212 | 1011 | 24 | 5 | 2 |
| 954 | 20262080 | 340 | 1993 | 43 | 5 | 3 |
| 955 | 20262081 | 243 | 1415 | 29 | 3 | 4 |
| 956 | 20262082 | 263 | 1166 | 18 | 16 | 3 |
| 957 | 20262083 | 234 | 1057 | 19 | 7 | 3 |
| 958 | 20262084 | 276 | 1671 | 35 | 3 | 4 |
| 959 | 20262085 | 216 | 1214 | 17 | 8 | 4 |
| 960 | 20262086 | 223 | 1119 | 14 | 7 | 2 |
| 961 | 20262087 | 256 | 1391 | 33 | 2 | 3 |
| 962 | 20262088 | 200 | 1037 | 13 | 4 | 2 |
| 963 | 20262089 | 221 | 947 | 17 | 3 | 3 |
| 964 | 20262090 | 244 | 1328 | 15 | 2 | 3 |
| 965 | 20262091 | 190 | 798 | 7 | 5 | 3 |
| 966 | 20262092 | 249 | 1477 | 17 | 1 | 2 |
| 967 | 20262093 | 227 | 1240 | 21 | 2 | 3 |
| 968 | 20262094 | 211 | 1239 | 28 | 7 | 0 |
| 969 | 20262095 | 268 | 1306 | 18 | 6 | 2 |
| 970 | 20262096 | 253 | 975 | 29 | 1 | 4 |
| 971 | 20262097 | 275 | 1211 | 20 | 0 | 3 |
| 972 | 20262098 | 309 | 1946 | 29 | 2 | 2 |
| 973 | 20262099 | 311 | 1830 | 16 | 1 | 1 |
| 974 | 20262100 | 220 | 961 | 12 | 5 | 2 |
| 975 | 20262101 | 308 | 1636 | 31 | 5 | 3 |
| 976 | 20262102 | 215 | 1039 | 9 | 3 | 2 |
| 977 | 20262103 | 193 | 916 | 10 | 6 | 3 |
| 978 | 20262104 | 185 | 1078 | 11 | 3 | 0 |
| 979 | 20262105 | 253 | 1438 | 19 | 2 | 2 |
| 980 | 20262106 | 272 | 1701 | 23 | 9 | 5 |
| 981 | 20262107 | 230 | 1109 | 19 | 7 | 4 |
| 982 | 20262108 | 237 | 1218 | 15 | 2 | 2 |
| 983 | 20262109 | 238 | 1092 | 20 | 4 | 1 |
| 984 | 20262110 | 219 | 1172 | 21 | 7 | 2 |
| 985 | 20262111 | 360 | 1987 | 35 | 0 | 3 |
| 986 | 20262112 | 182 | 796 | 10 | 4 | 1 |
| 987 | 20262113 | 194 | 1058 | 21 | 1 | 3 |
| 988 | 20262114 | 232 | 1107 | 14 | 6 | 3 |
| 989 | 20262115 | 227 | 1156 | 18 | 1 | 2 |
| 990 | 20262116 | 476 | 2212 | 37 | 10 | 4 |
| 991 | 20262117 | 246 | 1386 | 12 | 3 | 3 |
| 992 | 20262118 | 497 | 2524 | 42 | 12 | 4 |
| 993 | 20262119 | 315 | 1356 | 13 | 5 | 2 |
| 994 | 20262120 | 194 | 973 | 22 | 4 | 3 |
| 995 | 20262121 | 233 | 1109 | 17 | 3 | 3 |
| 996 | 20262122 | 192 | 1102 | 21 | 5 | 2 |
| 997 | 20262123 | 230 | 1271 | 18 | 5 | 2 |
| 998 | 20262124 | 306 | 1226 | 21 | 2 | 3 |
| 999 | 20262125 | 292 | 1398 | 21 | 8 | 5 |

## Throughput

- Turns simulated: 250784 (first play of each game)
- Aggregate: 285.1 turns/s over 879.5s wall
- Per game thread: 47.6 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

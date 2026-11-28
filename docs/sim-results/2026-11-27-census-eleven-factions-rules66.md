# Simulation: census-eleven-factions-rules66 (2026-11-27)

2112 games, first seed 9311003, brain `scripted`, turn limit off, safety cap 1200, 8 threads, started 2026-11-27T06:29:34Z. Wall time 2075.4s.

Source checkout at run start: `9bc02912cf08b05bd9b237f7b2c045d2a0c56d23`; dirty: no. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. Games seat the eleven cyclic rosters of eight over all eleven factions (`--all-factions`), one roster per eight games, turned one seat per game, so every faction plays every seat equally often, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--players","8","--difficulty","hard","--all-factions","--games","2112","--seed","9311003","--threads","8","--label","census-eleven-factions-rules66"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 2112 of 2112 |
| Game length, turns (mean / median / min-max) | 252.1 / 245 / 132-500 |
| Win reason mix | VP hold 94.3%, Capitals 5.3%, Turn limit 0.3%, Draw 0.1% |
| Median end turn | 245 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 5.3% / 94.3% / 0.3% / 0.1% |
| Games reaching the World War era (any empire) | 2100 of 2112 (99.4%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 2848 / 727 / 1997 / 124 |
| Hold break rate | 25.5% of started; 26.7% of resolved (broken / (broken + completed)) |
| Hold wins | 1992 |
| First reach to win, turns (mean / median / min-max) | 13.2 / 9 / 9-254 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 277 |
| Wins by a team another team beat to its target | 136 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 1992 | 248.0 / 242 / 132-498 |
| Capitals | 111 | 304.1 / 292 / 187-487 |
| Turn limit | 7 | 500.0 / 500 / 500-500 |
| Draw | 2 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 122 | 5.8% |
| India | 155 | 7.3% |
| Byzantium | 173 | 8.2% |
| Germany | 206 | 9.8% |
| Celts | 244 | 11.6% |
| Mongols | 92 | 4.4% |
| Sparta | 189 | 9.0% |
| Athens | 193 | 9.1% |
| Aztecs | 151 | 7.2% |
| Egypt | 523 | 24.8% |
| Rome | 62 | 2.9% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 1992 | 94.3% |
| Conquest | 0 | 0.0% |
| Capitals | 111 | 5.3% |
| Turn limit | 7 | 0.3% |
| Draw | 2 | 0.1% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 1536 | 122 | 7.9% |
| India | 1536 | 155 | 10.1% |
| Byzantium | 1536 | 173 | 11.3% |
| Germany | 1536 | 206 | 13.4% |
| Celts | 1536 | 244 | 15.9% |
| Mongols | 1536 | 92 | 6.0% |
| Sparta | 1536 | 189 | 12.3% |
| Athens | 1536 | 193 | 12.6% |
| Aztecs | 1536 | 151 | 9.8% |
| Egypt | 1536 | 523 | 34.0% |
| Rome | 1536 | 62 | 4.0% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 16896 | 2110 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 1536 | 122 | 7.9% |
| India | Hard | 1536 | 155 | 10.1% |
| Byzantium | Hard | 1536 | 173 | 11.3% |
| Germany | Hard | 1536 | 206 | 13.4% |
| Celts | Hard | 1536 | 244 | 15.9% |
| Mongols | Hard | 1536 | 92 | 6.0% |
| Sparta | Hard | 1536 | 189 | 12.3% |
| Athens | Hard | 1536 | 193 | 12.6% |
| Aztecs | Hard | 1536 | 151 | 9.8% |
| Egypt | Hard | 1536 | 523 | 34.0% |
| Rome | Hard | 1536 | 62 | 4.0% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 1536 | 127 | 8.3% | 135 | 150 | 185 | 264 | 342 |
| India | 1536 | 642 | 41.8% | 48 | 89 | 124 | 219 | 380 |
| Byzantium | 1536 | 858 | 55.9% | 71 | 86 | 122 | 177 | 334 |
| Germany | 1536 | 417 | 27.1% | 102 | 130 | 172 | 234 | 488 |
| Celts | 1536 | 1042 | 67.8% | 30 | 189 | 210 | 210 | 292 |
| Mongols | 1536 | 144 | 9.4% | 136 | 170 | 216 | 278 | 474 |
| Sparta | 1536 | 239 | 15.6% | 138 | 172 | 223 | 290 | 441 |
| Athens | 1536 | 569 | 37.0% | 128 | 187 | 255 | 255 | 255 |
| Aztecs | 1536 | 497 | 32.4% | 134 | 201 | 225 | 248 | 358 |
| Egypt | 1536 | 585 | 38.1% | 115 | 177 | 209 | 273 | 427 |
| Rome | 1536 | 53 | 3.5% | 131 | 153 | 210 | 295 | 487 |

## Goal cities (Rome's Colonia, Egypt's tomb)

Per seat of the faction: its goal city stood (median first turn), the seat scored it, held it at the end, a rival held it at the end, it changed hands. Rome: seats with a level-3 melee unit while the Colonia was open (median first turn) and the share of those seat-turns with no legal Colonia site on the map. Egypt: seats that started the work (median first start), work turns and pauses per starting seat.

| Faction | Seats | Stood | Turn | Scored | Held at end | Rival holds | Changed hands | Level 3 | Turn | No site | Started | Turn | Work turns | Pauses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Egypt | 1536 | 475 | 214 | 475 | 335 | 96 | 128 | 0 | - | - | 774 | 197 | 11.6 | 0.7 |
| Rome | 1536 | 0 | - | 0 | 0 | 0 | 0 | 0 | - | - | 0 | - | - | - |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 132 | 193 | 245 | 316 | 500 |
| Winning team VP | 3 | 10 | 10 | 13 | 18 |
| First Soldier completed (turn, per empire) | 6 | 8 | 14 | 55 | 253 |
| First Settler completed (turn, per empire) | 2 | 9 | 16 | 28 | 239 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 5 | 15 | 23 | 52 | 273 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 16896 observed empires in 2112 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 16271 | 625 |
| First Settler completed | 14645 | 2251 |
| First city founded | 16896 | 0 |
| Second city founded | 15693 | 1203 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 4682 |
| 1 | 2717 |
| 2 | 1915 |
| 3 | 1455 |
| 4 | 1095 |
| 5 | 858 |
| 6 | 730 |
| 7 | 574 |
| 8 | 416 |
| 9 | 291 |
| 10 | 1093 |
| 11 | 598 |
| 12 | 252 |
| 13 | 126 |
| 14 | 52 |
| 15 | 26 |
| 16 | 13 |
| 17 | 1 |
| 18 | 2 |

## End state

- Average game length: 252.1 turns
- Average cities per empire: 4.24
- Average techs per empire: 29.81
- Average eliminations per game: 3.06

## Strategy diversity: observed behavior (classifier v2)

2112 finished games; 16896 player seats; 2110 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4229** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 8579 | 500 | 5.8% | 50.8% | 23.7% |
| Wide | 4694 | 1489 | 31.7% | 27.8% | 70.6% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 11 | 0 | 0.0% | 0.1% | 0.0% |
| Economic | 999 | 118 | 11.8% | 5.9% | 5.6% |
| Generalist | 2613 | 3 | 0.1% | 15.5% | 0.1% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 17.0% / 2.6% (2010) | n/a / n/a (0) | 37.3% / 3.7% (10) | 22.1% / 5.2% (801) | 83.8% / 2.0% (1677) |
| Wide | 80.4% / 2.6% (2010) | — | n/a / n/a (0) | 75.8% / 3.0% (11) | 62.9% / 3.9% (759) | 96.5% / 0.6% (1599) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 59.0% / 3.7% (10) | 21.2% / 3.0% (11) | n/a / n/a (0) | — | 37.5% / 37.5% (4) | 94.4% / 0.0% (9) |
| Economic | 72.7% / 5.2% (801) | 33.2% / 3.9% (759) | n/a / n/a (0) | 25.0% / 37.5% (4) | — | 93.9% / 1.7% (597) |
| Generalist | 14.3% / 2.0% (1677) | 2.9% / 0.6% (1599) | n/a / n/a (0) | 5.6% / 0.0% (9) | 4.3% / 1.7% (597) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 8 / 454 | 104 / 556 | 0 / 0 | 0 / 0 | 10 / 176 | 0 / 350 |
| India | 49 / 849 | 105 / 358 | 0 / 0 | 0 / 0 | 0 / 13 | 1 / 316 |
| Byzantium | 47 / 643 | 112 / 456 | 0 / 0 | 0 / 1 | 14 / 149 | 0 / 287 |
| Germany | 29 / 642 | 167 / 518 | 0 / 0 | 0 / 1 | 10 / 76 | 0 / 299 |
| Celts | 34 / 537 | 197 / 636 | 0 / 0 | 0 / 0 | 13 / 107 | 0 / 256 |
| Mongols | 44 / 1159 | 45 / 110 | 0 / 0 | 0 / 0 | 3 / 21 | 0 / 246 |
| Sparta | 117 / 1260 | 61 / 211 | 0 / 0 | 0 / 2 | 10 / 45 | 1 / 18 |
| Athens | 18 / 528 | 148 / 499 | 0 / 0 | 0 / 0 | 26 / 151 | 1 / 358 |
| Aztecs | 28 / 671 | 114 / 428 | 0 / 0 | 0 / 2 | 9 / 121 | 0 / 314 |
| Egypt | 80 / 500 | 424 / 880 | 0 / 0 | 0 / 4 | 19 / 94 | 0 / 58 |
| Rome | 46 / 1336 | 12 / 42 | 0 / 0 | 0 / 1 | 4 / 46 | 0 / 111 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Turtle/Wonder. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 2110 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

2112 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 32362/2112 | 15.323 | 2112/2112 (100.0%) |
| Peace offers | 213568/2112 | 101.121 | 2112/2112 (100.0%) |
| Peace agreements | 24800/2112 | 11.742 | 2110/2112 (99.9%) |
| Battles | 2864257/2112 | 1356.182 | 2112/2112 (100.0%) |
| City captures | 53386/2112 | 25.277 | 2112/2112 (100.0%) |
| Settler captures | 7106/2112 | 3.365 | 1977/2112 (93.6%) |
| Eliminations | 6472/2112 | 3.064 | 2088/2112 (98.9%) |

## Terminal VP sources

2112 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 13644/16896 | 13644/16896 | 6876/2110 |
| Faction goals | 8368/16896 | 8368/16896 | 3316/2110 |
| Era objectives | 30052/16896 | 30052/16896 | 9278/2110 |
| Capitals taken | 5412/16896 | 5412/16896 | 3319/2110 |
| Total VP | 57476/16896 | 57476/16896 | 22789/2110 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 191/1434 (13.3%) | 2 |
| Tied leaders (winner in set) | 486/678 (71.7%) | 0 |
| Any leader set | 677/2112 (32.1%) | 2 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 15804/16896 | 5 | 20 | 28 | 33 | 86 | -2.4 (2110) |
| Medieval | 15098/16896 | 35 | 58 | 62 | 67 | 178 | -2.6 (2110) |
| Renaissance | 14875/16896 | 60 | 84 | 91 | 103 | 240 | -5.0 (2110) |
| Industrial | 14195/16896 | 83 | 111 | 122 | 142 | 277 | -8.4 (2110) |
| Machine Age | 12460/16896 | 109 | 140 | 154 | 179 | 323 | -11.0 (2109) |
| World War | 9882/16896 | 120 | 164 | 182 | 212 | 334 | -12.8 (2094) |
| Modern | 6805/16896 | 144 | 187 | 212 | 247 | 404 | -12.9 (1902) |
| Future | 3839/16896 | 164 | 210 | 242 | 272 | 483 | -10.0 (1383) |

Research idle turns (techs left, no research set): 18, of which stalled (two or more in a row): 0.

## Tech pace against the calendar

Median resolved turn each technology was learned, the display year of that turn (`calendar::calendar_label`) and the year the technology really arrived (n/a when no single year is established). Empires that never learned it are left out.

| Era | Technology | Learned / seats | Median turn | Median year | Real year |
| --- | --- | --- | --- | --- | --- |
| Ancient | Irrigation | 16815/16896 | 9 | 3173 BC | 6000 BC |
| Ancient | Bronze Working | 14970/16896 | 18 | 2242 BC | 3300 BC |
| Ancient | Masonry | 16893/16896 | 12 | 2863 BC | 3000 BC |
| Ancient | Currency | 15989/16896 | 27 | 1311 BC | 600 BC |
| Ancient | Writing | 16089/16896 | 24 | 1621 BC | 3200 BC |
| Classical | Mathematics | 15255/16896 | 44 | 301 BC | 300 BC |
| Classical | Engineering | 15143/16896 | 51 | 50 AD | n/a |
| Classical | Iron Working | 15390/16896 | 38 | 601 BC | 1200 BC |
| Classical | Code of Laws | 15117/16896 | 59 | 450 AD | 1754 BC |
| Classical | Philosophy | 15100/16896 | 61 | 530 AD | 500 BC |
| Medieval | Feudalism | 15018/16896 | 76 | 980 AD | n/a |
| Medieval | Machinery | 15050/16896 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 13624/16896 | 81 | 1130 AD | n/a |
| Medieval | Education | 14923/16896 | 87 | 1310 AD | 1088 AD |
| Medieval | Crop Rotation | 14880/16896 | 90 | 1400 AD | n/a |
| Renaissance | Gunpowder | 14805/16896 | 98 | 1480 AD | n/a |
| Renaissance | Metallurgy | 14619/16896 | 109 | 1590 AD | n/a |
| Renaissance | Military Engineering | 14683/16896 | 106 | 1560 AD | n/a |
| Renaissance | Banking | 14349/16896 | 119 | 1690 AD | 1397 AD |
| Renaissance | Printing Press | 14242/16896 | 120 | 1700 AD | 1440 AD |
| Industrial | Rifling | 13063/16896 | 143 | 1816 AD | n/a |
| Industrial | Military Science | 13552/16896 | 135 | 1787 AD | n/a |
| Industrial | Ballistics | 13217/16896 | 147 | 1831 AD | n/a |
| Industrial | Industrialization | 13839/16896 | 132 | 1776 AD | 1769 AD |
| Industrial | Railways | 12467/16896 | 154 | 1857 AD | 1825 AD |
| Machine Age | Replaceable Parts | 11248/16896 | 169 | 1890 AD | 1800 AD |
| Machine Age | Combustion | 11855/16896 | 163 | 1883 AD | 1876 AD |
| Machine Age | Steel | 11320/16896 | 167 | 1887 AD | 1856 AD |
| Machine Age | Flight | 10425/16896 | 177 | 1899 AD | 1903 AD |
| Machine Age | Electricity | 9936/16896 | 182 | 1904 AD | 1879 AD |
| World War | Mass Mobilisation | 8315/16896 | 197 | 1924 AD | n/a |
| World War | Rocketry | 7875/16896 | 201 | 1929 AD | 1926 AD |
| World War | Armoured Warfare | 7409/16896 | 206 | 1937 AD | 1916 AD |
| World War | Air Power | 7111/16896 | 210 | 1942 AD | n/a |
| World War | Electronics | 9150/16896 | 189 | 1912 AD | n/a |
| Modern | Mechanised Warfare | 5244/16896 | 226 | 1976 AD | n/a |
| Modern | Composite Armour | 4737/16896 | 231 | 1988 AD | n/a |
| Modern | Guided Missiles | 4445/16896 | 236 | 2000 AD | n/a |
| Modern | Jet Engines | 4257/16896 | 239 | 2007 AD | 1939 AD |
| Modern | Green Revolution | 5928/16896 | 219 | 1959 AD | n/a |
| Future | Autonomous Systems | 2693/16896 | 256 | 2032 AD | n/a |
| Future | Robotics | 2339/16896 | 258 | 2034 AD | n/a |
| Future | Fusion Power | 2173/16896 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 3166/16896 | 250 | 2026 AD | n/a |
| Future | Recursive AI | 1880/16896 | 256 | 2032 AD | n/a |

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

2112 games with promotion telemetry. Promotions are UnitPromoted events by the level reached; final units are military units alive when the game ended.

| Level | Promotions / game | Final units / game |
| --- | --- | --- |
| 0 | 0.00 | 66.45 |
| 1 | 84.09 | 12.01 |
| 2 | 26.84 | 4.73 |
| 3 | 7.18 | 1.11 |
| 4 | 1.44 | 0.24 |
| 5 | 0.28 | 0.07 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 9311003 | 201 | 905 | 17 | 6 | 2 |
| 1 | 9311004 | 216 | 993 | 10 | 16 | 2 |
| 2 | 9311005 | 315 | 1575 | 37 | 3 | 4 |
| 3 | 9311006 | 230 | 1224 | 15 | 2 | 2 |
| 4 | 9311007 | 279 | 1237 | 32 | 8 | 3 |
| 5 | 9311008 | 231 | 1079 | 20 | 4 | 4 |
| 6 | 9311009 | 203 | 887 | 19 | 2 | 3 |
| 7 | 9311010 | 243 | 1056 | 20 | 3 | 3 |
| 8 | 9311011 | 223 | 976 | 18 | 4 | 3 |
| 9 | 9311012 | 302 | 1534 | 23 | 3 | 2 |
| 10 | 9311013 | 283 | 1498 | 29 | 1 | 4 |
| 11 | 9311014 | 191 | 1002 | 21 | 1 | 2 |
| 12 | 9311015 | 232 | 1264 | 25 | 3 | 3 |
| 13 | 9311016 | 184 | 945 | 9 | 3 | 3 |
| 14 | 9311017 | 207 | 1146 | 14 | 2 | 4 |
| 15 | 9311018 | 205 | 1170 | 16 | 4 | 3 |
| 16 | 9311019 | 211 | 865 | 18 | 1 | 3 |
| 17 | 9311020 | 208 | 1145 | 12 | 3 | 1 |
| 18 | 9311021 | 209 | 1323 | 20 | 8 | 2 |
| 19 | 9311022 | 245 | 1374 | 26 | 3 | 3 |
| 20 | 9311023 | 206 | 1093 | 18 | 3 | 2 |
| 21 | 9311024 | 205 | 1161 | 24 | 8 | 3 |
| 22 | 9311025 | 228 | 1128 | 8 | 4 | 3 |
| 23 | 9311026 | 279 | 1432 | 17 | 6 | 3 |
| 24 | 9311027 | 280 | 1499 | 24 | 4 | 2 |
| 25 | 9311028 | 195 | 862 | 22 | 2 | 4 |
| 26 | 9311029 | 219 | 1126 | 16 | 4 | 1 |
| 27 | 9311030 | 193 | 891 | 11 | 3 | 2 |
| 28 | 9311031 | 303 | 1434 | 28 | 3 | 5 |
| 29 | 9311032 | 301 | 1706 | 27 | 2 | 4 |
| 30 | 9311033 | 251 | 1205 | 22 | 7 | 5 |
| 31 | 9311034 | 272 | 1424 | 20 | 4 | 6 |
| 32 | 9311035 | 267 | 1990 | 34 | 1 | 3 |
| 33 | 9311036 | 187 | 1206 | 21 | 2 | 2 |
| 34 | 9311037 | 278 | 1698 | 24 | 0 | 4 |
| 35 | 9311038 | 238 | 1450 | 14 | 4 | 2 |
| 36 | 9311039 | 190 | 1069 | 18 | 1 | 3 |
| 37 | 9311040 | 256 | 1255 | 20 | 3 | 2 |
| 38 | 9311041 | 291 | 1398 | 28 | 8 | 3 |
| 39 | 9311042 | 272 | 1642 | 21 | 2 | 5 |
| 40 | 9311043 | 293 | 1767 | 33 | 3 | 3 |
| 41 | 9311044 | 401 | 1740 | 41 | 3 | 5 |
| 42 | 9311045 | 194 | 974 | 22 | 5 | 3 |
| 43 | 9311046 | 235 | 1031 | 22 | 2 | 4 |
| 44 | 9311047 | 221 | 936 | 19 | 4 | 2 |
| 45 | 9311048 | 180 | 833 | 13 | 1 | 2 |
| 46 | 9311049 | 244 | 1104 | 27 | 5 | 3 |
| 47 | 9311050 | 218 | 1281 | 24 | 4 | 3 |
| 48 | 9311051 | 236 | 1268 | 28 | 5 | 5 |
| 49 | 9311052 | 273 | 1457 | 39 | 1 | 3 |
| 50 | 9311053 | 218 | 1433 | 25 | 2 | 1 |
| 51 | 9311054 | 277 | 1417 | 25 | 5 | 3 |
| 52 | 9311055 | 285 | 1581 | 39 | 5 | 5 |
| 53 | 9311056 | 253 | 1579 | 22 | 3 | 2 |
| 54 | 9311057 | 211 | 1199 | 24 | 0 | 3 |
| 55 | 9311058 | 231 | 1568 | 31 | 1 | 3 |
| 56 | 9311059 | 209 | 968 | 23 | 1 | 3 |
| 57 | 9311060 | 187 | 1243 | 16 | 5 | 2 |
| 58 | 9311061 | 132 | 484 | 14 | 4 | 0 |
| 59 | 9311062 | 232 | 1210 | 22 | 5 | 2 |
| 60 | 9311063 | 266 | 1248 | 38 | 4 | 5 |
| 61 | 9311064 | 236 | 1210 | 24 | 3 | 4 |
| 62 | 9311065 | 253 | 1489 | 38 | 2 | 3 |
| 63 | 9311066 | 206 | 1075 | 19 | 7 | 1 |
| 64 | 9311067 | 200 | 1423 | 32 | 3 | 1 |
| 65 | 9311068 | 215 | 851 | 19 | 1 | 3 |
| 66 | 9311069 | 233 | 1396 | 29 | 3 | 5 |
| 67 | 9311070 | 291 | 1681 | 39 | 3 | 4 |
| 68 | 9311071 | 339 | 2561 | 26 | 5 | 2 |
| 69 | 9311072 | 268 | 1472 | 32 | 1 | 3 |
| 70 | 9311073 | 192 | 808 | 19 | 2 | 1 |
| 71 | 9311074 | 324 | 2317 | 39 | 6 | 3 |
| 72 | 9311075 | 253 | 1657 | 22 | 4 | 3 |
| 73 | 9311076 | 293 | 1809 | 29 | 3 | 1 |
| 74 | 9311077 | 270 | 1670 | 27 | 4 | 2 |
| 75 | 9311078 | 205 | 1070 | 19 | 3 | 2 |
| 76 | 9311079 | 327 | 1640 | 53 | 0 | 6 |
| 77 | 9311080 | 230 | 1299 | 21 | 4 | 4 |
| 78 | 9311081 | 205 | 1250 | 25 | 2 | 2 |
| 79 | 9311082 | 487 | 3385 | 68 | 4 | 2 |
| 80 | 9311083 | 238 | 1005 | 14 | 1 | 1 |
| 81 | 9311084 | 250 | 1384 | 23 | 4 | 2 |
| 82 | 9311085 | 299 | 1548 | 32 | 3 | 3 |
| 83 | 9311086 | 227 | 1286 | 20 | 1 | 3 |
| 84 | 9311087 | 236 | 1359 | 34 | 3 | 2 |
| 85 | 9311088 | 216 | 1001 | 15 | 1 | 3 |
| 86 | 9311089 | 261 | 1410 | 26 | 4 | 4 |
| 87 | 9311090 | 227 | 1262 | 18 | 1 | 1 |
| 88 | 9311091 | 206 | 993 | 12 | 4 | 3 |
| 89 | 9311092 | 249 | 1145 | 25 | 9 | 4 |
| 90 | 9311093 | 214 | 764 | 17 | 6 | 2 |
| 91 | 9311094 | 260 | 1239 | 23 | 4 | 2 |
| 92 | 9311095 | 276 | 1194 | 22 | 2 | 4 |
| 93 | 9311096 | 310 | 1659 | 49 | 1 | 5 |
| 94 | 9311097 | 269 | 1401 | 26 | 5 | 3 |
| 95 | 9311098 | 282 | 1113 | 24 | 4 | 3 |
| 96 | 9311099 | 252 | 1203 | 17 | 6 | 4 |
| 97 | 9311100 | 268 | 1228 | 28 | 5 | 5 |
| 98 | 9311101 | 245 | 1249 | 23 | 7 | 3 |
| 99 | 9311102 | 279 | 1496 | 26 | 2 | 4 |
| 100 | 9311103 | 247 | 1006 | 18 | 4 | 5 |
| 101 | 9311104 | 192 | 874 | 12 | 6 | 3 |
| 102 | 9311105 | 233 | 1310 | 34 | 2 | 4 |
| 103 | 9311106 | 328 | 1687 | 43 | 2 | 3 |
| 104 | 9311107 | 264 | 1590 | 26 | 1 | 3 |
| 105 | 9311108 | 265 | 1391 | 19 | 2 | 3 |
| 106 | 9311109 | 214 | 1287 | 26 | 6 | 1 |
| 107 | 9311110 | 219 | 1222 | 27 | 7 | 3 |
| 108 | 9311111 | 212 | 654 | 19 | 0 | 3 |
| 109 | 9311112 | 269 | 1511 | 31 | 1 | 2 |
| 110 | 9311113 | 189 | 1179 | 13 | 5 | 3 |
| 111 | 9311114 | 260 | 1126 | 19 | 6 | 4 |
| 112 | 9311115 | 329 | 2061 | 35 | 3 | 1 |
| 113 | 9311116 | 231 | 1234 | 26 | 3 | 4 |
| 114 | 9311117 | 209 | 914 | 4 | 4 | 4 |
| 115 | 9311118 | 394 | 2402 | 25 | 3 | 4 |
| 116 | 9311119 | 189 | 1160 | 17 | 4 | 1 |
| 117 | 9311120 | 220 | 839 | 19 | 0 | 4 |
| 118 | 9311121 | 300 | 1572 | 25 | 3 | 5 |
| 119 | 9311122 | 281 | 1196 | 35 | 0 | 4 |
| 120 | 9311123 | 190 | 870 | 12 | 2 | 4 |
| 121 | 9311124 | 343 | 1641 | 28 | 5 | 3 |
| 122 | 9311125 | 235 | 1100 | 28 | 3 | 3 |
| 123 | 9311126 | 379 | 2272 | 34 | 6 | 4 |
| 124 | 9311127 | 258 | 1376 | 20 | 1 | 2 |
| 125 | 9311128 | 203 | 1230 | 22 | 0 | 3 |
| 126 | 9311129 | 273 | 1110 | 30 | 4 | 3 |
| 127 | 9311130 | 226 | 1223 | 27 | 4 | 3 |
| 128 | 9311131 | 224 | 1261 | 22 | 2 | 4 |
| 129 | 9311132 | 288 | 1114 | 36 | 1 | 6 |
| 130 | 9311133 | 166 | 657 | 20 | 2 | 3 |
| 131 | 9311134 | 231 | 1140 | 15 | 2 | 2 |
| 132 | 9311135 | 216 | 960 | 20 | 3 | 4 |
| 133 | 9311136 | 199 | 1156 | 24 | 4 | 4 |
| 134 | 9311137 | 228 | 1253 | 27 | 2 | 3 |
| 135 | 9311138 | 194 | 931 | 18 | 1 | 1 |
| 136 | 9311139 | 265 | 1431 | 32 | 2 | 3 |
| 137 | 9311140 | 262 | 840 | 16 | 1 | 4 |
| 138 | 9311141 | 185 | 965 | 12 | 2 | 1 |
| 139 | 9311142 | 179 | 1070 | 26 | 1 | 1 |
| 140 | 9311143 | 204 | 1296 | 20 | 1 | 4 |
| 141 | 9311144 | 338 | 1887 | 25 | 2 | 2 |
| 142 | 9311145 | 273 | 1584 | 31 | 10 | 2 |
| 143 | 9311146 | 300 | 1459 | 26 | 5 | 1 |
| 144 | 9311147 | 289 | 2102 | 35 | 3 | 2 |
| 145 | 9311148 | 182 | 1149 | 20 | 5 | 2 |
| 146 | 9311149 | 290 | 1633 | 19 | 4 | 3 |
| 147 | 9311150 | 237 | 1316 | 39 | 3 | 1 |
| 148 | 9311151 | 257 | 1538 | 28 | 10 | 3 |
| 149 | 9311152 | 202 | 1335 | 27 | 1 | 2 |
| 150 | 9311153 | 277 | 1829 | 32 | 4 | 2 |
| 151 | 9311154 | 287 | 1896 | 39 | 3 | 4 |
| 152 | 9311155 | 268 | 1550 | 26 | 6 | 5 |
| 153 | 9311156 | 231 | 1329 | 23 | 1 | 3 |
| 154 | 9311157 | 233 | 1371 | 16 | 3 | 2 |
| 155 | 9311158 | 260 | 1860 | 25 | 2 | 1 |
| 156 | 9311159 | 211 | 1064 | 27 | 4 | 1 |
| 157 | 9311160 | 222 | 1025 | 19 | 7 | 2 |
| 158 | 9311161 | 202 | 1122 | 13 | 3 | 3 |
| 159 | 9311162 | 192 | 1029 | 10 | 1 | 3 |
| 160 | 9311163 | 160 | 658 | 17 | 3 | 1 |
| 161 | 9311164 | 500 | 2586 | 52 | 5 | 6 |
| 162 | 9311165 | 248 | 1591 | 24 | 10 | 3 |
| 163 | 9311166 | 316 | 1977 | 47 | 4 | 4 |
| 164 | 9311167 | 298 | 1921 | 27 | 4 | 4 |
| 165 | 9311168 | 230 | 1192 | 22 | 6 | 4 |
| 166 | 9311169 | 365 | 2674 | 41 | 9 | 2 |
| 167 | 9311170 | 233 | 1320 | 28 | 4 | 2 |
| 168 | 9311171 | 316 | 1312 | 38 | 3 | 5 |
| 169 | 9311172 | 256 | 1069 | 26 | 6 | 5 |
| 170 | 9311173 | 299 | 1737 | 29 | 5 | 3 |
| 171 | 9311174 | 273 | 1392 | 30 | 1 | 4 |
| 172 | 9311175 | 217 | 882 | 25 | 0 | 3 |
| 173 | 9311176 | 263 | 1283 | 23 | 1 | 2 |
| 174 | 9311177 | 310 | 1536 | 31 | 2 | 2 |
| 175 | 9311178 | 205 | 964 | 23 | 2 | 3 |
| 176 | 9311179 | 195 | 826 | 17 | 7 | 1 |
| 177 | 9311180 | 277 | 1389 | 23 | 4 | 3 |
| 178 | 9311181 | 223 | 1099 | 20 | 3 | 3 |
| 179 | 9311182 | 264 | 1526 | 27 | 4 | 3 |
| 180 | 9311183 | 207 | 1125 | 14 | 0 | 1 |
| 181 | 9311184 | 191 | 774 | 14 | 1 | 2 |
| 182 | 9311185 | 248 | 1207 | 34 | 4 | 5 |
| 183 | 9311186 | 212 | 1008 | 19 | 7 | 3 |
| 184 | 9311187 | 310 | 1896 | 27 | 2 | 3 |
| 185 | 9311188 | 217 | 1117 | 17 | 6 | 4 |
| 186 | 9311189 | 288 | 1503 | 37 | 4 | 4 |
| 187 | 9311190 | 186 | 618 | 12 | 4 | 4 |
| 188 | 9311191 | 203 | 1156 | 19 | 6 | 1 |
| 189 | 9311192 | 256 | 1238 | 32 | 1 | 3 |
| 190 | 9311193 | 202 | 1031 | 12 | 4 | 2 |
| 191 | 9311194 | 314 | 1677 | 25 | 0 | 3 |
| 192 | 9311195 | 259 | 1332 | 30 | 5 | 2 |
| 193 | 9311196 | 227 | 1258 | 17 | 7 | 5 |
| 194 | 9311197 | 273 | 1556 | 29 | 4 | 3 |
| 195 | 9311198 | 256 | 1406 | 26 | 2 | 2 |
| 196 | 9311199 | 249 | 1424 | 28 | 8 | 2 |
| 197 | 9311200 | 244 | 1200 | 19 | 7 | 1 |
| 198 | 9311201 | 247 | 1544 | 27 | 15 | 4 |
| 199 | 9311202 | 422 | 1620 | 45 | 2 | 5 |
| 200 | 9311203 | 221 | 1254 | 27 | 5 | 4 |
| 201 | 9311204 | 226 | 1274 | 16 | 0 | 3 |
| 202 | 9311205 | 291 | 1802 | 29 | 5 | 5 |
| 203 | 9311206 | 325 | 2009 | 30 | 5 | 2 |
| 204 | 9311207 | 287 | 1300 | 24 | 3 | 4 |
| 205 | 9311208 | 351 | 1960 | 36 | 4 | 5 |
| 206 | 9311209 | 239 | 1060 | 18 | 1 | 5 |
| 207 | 9311210 | 219 | 1110 | 14 | 2 | 3 |
| 208 | 9311211 | 255 | 1365 | 27 | 1 | 2 |
| 209 | 9311212 | 242 | 1180 | 23 | 1 | 2 |
| 210 | 9311213 | 272 | 1563 | 28 | 1 | 2 |
| 211 | 9311214 | 365 | 2079 | 41 | 1 | 4 |
| 212 | 9311215 | 205 | 949 | 20 | 0 | 3 |
| 213 | 9311216 | 257 | 1592 | 29 | 4 | 3 |
| 214 | 9311217 | 260 | 1595 | 23 | 4 | 4 |
| 215 | 9311218 | 281 | 1749 | 29 | 2 | 3 |
| 216 | 9311219 | 210 | 1328 | 22 | 3 | 3 |
| 217 | 9311220 | 298 | 1719 | 35 | 2 | 4 |
| 218 | 9311221 | 203 | 827 | 16 | 0 | 4 |
| 219 | 9311222 | 273 | 1557 | 28 | 1 | 3 |
| 220 | 9311223 | 299 | 1671 | 27 | 6 | 4 |
| 221 | 9311224 | 167 | 691 | 19 | 1 | 3 |
| 222 | 9311225 | 206 | 1347 | 21 | 2 | 2 |
| 223 | 9311226 | 230 | 1395 | 34 | 0 | 4 |
| 224 | 9311227 | 304 | 1609 | 35 | 4 | 5 |
| 225 | 9311228 | 186 | 791 | 19 | 1 | 3 |
| 226 | 9311229 | 233 | 1375 | 19 | 1 | 2 |
| 227 | 9311230 | 269 | 1523 | 36 | 0 | 3 |
| 228 | 9311231 | 258 | 1513 | 27 | 3 | 2 |
| 229 | 9311232 | 255 | 1527 | 26 | 2 | 3 |
| 230 | 9311233 | 233 | 1120 | 29 | 3 | 3 |
| 231 | 9311234 | 159 | 560 | 11 | 2 | 2 |
| 232 | 9311235 | 332 | 2143 | 30 | 4 | 3 |
| 233 | 9311236 | 272 | 1638 | 50 | 3 | 4 |
| 234 | 9311237 | 405 | 2368 | 50 | 4 | 4 |
| 235 | 9311238 | 246 | 1267 | 31 | 10 | 4 |
| 236 | 9311239 | 190 | 933 | 16 | 1 | 1 |
| 237 | 9311240 | 177 | 757 | 16 | 3 | 2 |
| 238 | 9311241 | 200 | 1093 | 23 | 5 | 4 |
| 239 | 9311242 | 262 | 1138 | 20 | 3 | 1 |
| 240 | 9311243 | 239 | 1475 | 24 | 4 | 5 |
| 241 | 9311244 | 270 | 1893 | 18 | 4 | 4 |
| 242 | 9311245 | 294 | 1974 | 29 | 11 | 5 |
| 243 | 9311246 | 194 | 1111 | 18 | 2 | 2 |
| 244 | 9311247 | 325 | 1848 | 38 | 5 | 3 |
| 245 | 9311248 | 214 | 1135 | 16 | 3 | 2 |
| 246 | 9311249 | 262 | 1609 | 24 | 4 | 5 |
| 247 | 9311250 | 190 | 1028 | 22 | 8 | 3 |
| 248 | 9311251 | 240 | 1328 | 22 | 4 | 4 |
| 249 | 9311252 | 194 | 1243 | 28 | 2 | 3 |
| 250 | 9311253 | 249 | 1487 | 22 | 2 | 4 |
| 251 | 9311254 | 304 | 2070 | 28 | 3 | 2 |
| 252 | 9311255 | 327 | 1697 | 41 | 1 | 5 |
| 253 | 9311256 | 217 | 1359 | 17 | 12 | 2 |
| 254 | 9311257 | 184 | 844 | 12 | 8 | 2 |
| 255 | 9311258 | 223 | 1280 | 26 | 4 | 4 |
| 256 | 9311259 | 218 | 995 | 18 | 0 | 3 |
| 257 | 9311260 | 194 | 801 | 17 | 2 | 3 |
| 258 | 9311261 | 266 | 1189 | 32 | 3 | 4 |
| 259 | 9311262 | 207 | 1041 | 16 | 2 | 3 |
| 260 | 9311263 | 310 | 1607 | 31 | 2 | 4 |
| 261 | 9311264 | 266 | 1282 | 31 | 0 | 4 |
| 262 | 9311265 | 227 | 1210 | 26 | 3 | 3 |
| 263 | 9311266 | 307 | 1291 | 28 | 8 | 5 |
| 264 | 9311267 | 324 | 1307 | 6 | 5 | 6 |
| 265 | 9311268 | 232 | 1097 | 17 | 2 | 3 |
| 266 | 9311269 | 253 | 1005 | 30 | 8 | 3 |
| 267 | 9311270 | 398 | 2714 | 42 | 3 | 4 |
| 268 | 9311271 | 201 | 917 | 27 | 2 | 4 |
| 269 | 9311272 | 229 | 1185 | 17 | 1 | 1 |
| 270 | 9311273 | 227 | 1188 | 26 | 1 | 3 |
| 271 | 9311274 | 188 | 914 | 23 | 2 | 2 |
| 272 | 9311275 | 283 | 1280 | 18 | 4 | 6 |
| 273 | 9311276 | 275 | 1291 | 32 | 5 | 5 |
| 274 | 9311277 | 324 | 1912 | 42 | 3 | 4 |
| 275 | 9311278 | 237 | 1319 | 24 | 8 | 3 |
| 276 | 9311279 | 243 | 1062 | 19 | 2 | 3 |
| 277 | 9311280 | 213 | 949 | 23 | 2 | 3 |
| 278 | 9311281 | 199 | 843 | 21 | 0 | 3 |
| 279 | 9311282 | 286 | 1293 | 13 | 1 | 2 |
| 280 | 9311283 | 223 | 921 | 15 | 3 | 0 |
| 281 | 9311284 | 216 | 862 | 14 | 5 | 3 |
| 282 | 9311285 | 237 | 1518 | 21 | 5 | 4 |
| 283 | 9311286 | 265 | 1614 | 17 | 7 | 1 |
| 284 | 9311287 | 306 | 1117 | 35 | 3 | 6 |
| 285 | 9311288 | 349 | 2141 | 49 | 9 | 4 |
| 286 | 9311289 | 175 | 846 | 11 | 7 | 1 |
| 287 | 9311290 | 267 | 1310 | 25 | 1 | 2 |
| 288 | 9311291 | 321 | 1804 | 38 | 1 | 4 |
| 289 | 9311292 | 242 | 1121 | 24 | 5 | 3 |
| 290 | 9311293 | 172 | 869 | 12 | 3 | 1 |
| 291 | 9311294 | 224 | 879 | 19 | 0 | 3 |
| 292 | 9311295 | 212 | 948 | 13 | 2 | 3 |
| 293 | 9311296 | 229 | 876 | 15 | 6 | 3 |
| 294 | 9311297 | 277 | 1755 | 29 | 3 | 2 |
| 295 | 9311298 | 363 | 1977 | 28 | 3 | 3 |
| 296 | 9311299 | 228 | 1274 | 25 | 2 | 3 |
| 297 | 9311300 | 204 | 845 | 16 | 2 | 3 |
| 298 | 9311301 | 181 | 708 | 6 | 5 | 2 |
| 299 | 9311302 | 259 | 1216 | 26 | 3 | 4 |
| 300 | 9311303 | 265 | 1752 | 18 | 2 | 2 |
| 301 | 9311304 | 281 | 1701 | 29 | 3 | 1 |
| 302 | 9311305 | 279 | 1598 | 13 | 5 | 5 |
| 303 | 9311306 | 248 | 1259 | 13 | 5 | 5 |
| 304 | 9311307 | 280 | 1433 | 24 | 0 | 3 |
| 305 | 9311308 | 315 | 1685 | 28 | 1 | 1 |
| 306 | 9311309 | 163 | 807 | 17 | 1 | 3 |
| 307 | 9311310 | 261 | 1366 | 25 | 2 | 3 |
| 308 | 9311311 | 275 | 1893 | 35 | 1 | 5 |
| 309 | 9311312 | 223 | 822 | 18 | 2 | 3 |
| 310 | 9311313 | 217 | 813 | 20 | 3 | 5 |
| 311 | 9311314 | 186 | 939 | 12 | 2 | 4 |
| 312 | 9311315 | 276 | 1544 | 30 | 3 | 4 |
| 313 | 9311316 | 208 | 1018 | 23 | 3 | 3 |
| 314 | 9311317 | 432 | 2525 | 51 | 3 | 4 |
| 315 | 9311318 | 211 | 1062 | 21 | 3 | 5 |
| 316 | 9311319 | 292 | 1841 | 31 | 2 | 3 |
| 317 | 9311320 | 277 | 1433 | 28 | 4 | 3 |
| 318 | 9311321 | 185 | 858 | 19 | 2 | 4 |
| 319 | 9311322 | 205 | 977 | 15 | 3 | 5 |
| 320 | 9311323 | 276 | 1758 | 24 | 6 | 1 |
| 321 | 9311324 | 254 | 1214 | 29 | 2 | 3 |
| 322 | 9311325 | 271 | 1386 | 21 | 6 | 4 |
| 323 | 9311326 | 230 | 974 | 16 | 1 | 3 |
| 324 | 9311327 | 242 | 1301 | 27 | 4 | 4 |
| 325 | 9311328 | 303 | 1807 | 43 | 7 | 4 |
| 326 | 9311329 | 264 | 1610 | 42 | 10 | 4 |
| 327 | 9311330 | 215 | 1207 | 18 | 1 | 3 |
| 328 | 9311331 | 172 | 773 | 16 | 1 | 2 |
| 329 | 9311332 | 247 | 1732 | 29 | 1 | 3 |
| 330 | 9311333 | 286 | 1879 | 30 | 2 | 4 |
| 331 | 9311334 | 281 | 1956 | 23 | 2 | 1 |
| 332 | 9311335 | 313 | 2289 | 29 | 10 | 3 |
| 333 | 9311336 | 256 | 1536 | 26 | 1 | 3 |
| 334 | 9311337 | 201 | 1128 | 21 | 3 | 1 |
| 335 | 9311338 | 240 | 1356 | 19 | 1 | 3 |
| 336 | 9311339 | 207 | 1008 | 11 | 3 | 2 |
| 337 | 9311340 | 237 | 1401 | 24 | 4 | 2 |
| 338 | 9311341 | 185 | 948 | 11 | 1 | 2 |
| 339 | 9311342 | 234 | 1313 | 13 | 1 | 1 |
| 340 | 9311343 | 204 | 1129 | 15 | 4 | 1 |
| 341 | 9311344 | 229 | 1046 | 15 | 3 | 3 |
| 342 | 9311345 | 254 | 1584 | 44 | 3 | 4 |
| 343 | 9311346 | 251 | 1241 | 31 | 3 | 4 |
| 344 | 9311347 | 200 | 977 | 21 | 5 | 3 |
| 345 | 9311348 | 212 | 1198 | 19 | 4 | 2 |
| 346 | 9311349 | 230 | 1047 | 24 | 3 | 4 |
| 347 | 9311350 | 290 | 1635 | 31 | 3 | 3 |
| 348 | 9311351 | 293 | 1463 | 21 | 0 | 3 |
| 349 | 9311352 | 266 | 1086 | 29 | 2 | 2 |
| 350 | 9311353 | 187 | 786 | 7 | 3 | 4 |
| 351 | 9311354 | 218 | 1256 | 19 | 3 | 1 |
| 352 | 9311355 | 235 | 1003 | 20 | 1 | 4 |
| 353 | 9311356 | 262 | 1240 | 22 | 7 | 5 |
| 354 | 9311357 | 235 | 868 | 12 | 5 | 5 |
| 355 | 9311358 | 427 | 2579 | 52 | 10 | 5 |
| 356 | 9311359 | 218 | 897 | 16 | 5 | 1 |
| 357 | 9311360 | 222 | 1077 | 32 | 4 | 3 |
| 358 | 9311361 | 321 | 1487 | 38 | 0 | 3 |
| 359 | 9311362 | 196 | 874 | 9 | 2 | 2 |
| 360 | 9311363 | 205 | 906 | 14 | 4 | 4 |
| 361 | 9311364 | 228 | 1199 | 16 | 4 | 3 |
| 362 | 9311365 | 203 | 927 | 17 | 4 | 3 |
| 363 | 9311366 | 206 | 889 | 25 | 1 | 3 |
| 364 | 9311367 | 270 | 1587 | 26 | 1 | 1 |
| 365 | 9311368 | 308 | 1394 | 32 | 1 | 4 |
| 366 | 9311369 | 240 | 1332 | 19 | 1 | 4 |
| 367 | 9311370 | 269 | 1296 | 21 | 3 | 3 |
| 368 | 9311371 | 254 | 1354 | 34 | 6 | 4 |
| 369 | 9311372 | 239 | 1187 | 20 | 2 | 3 |
| 370 | 9311373 | 186 | 1045 | 16 | 4 | 3 |
| 371 | 9311374 | 189 | 1032 | 16 | 4 | 0 |
| 372 | 9311375 | 217 | 817 | 15 | 4 | 0 |
| 373 | 9311376 | 202 | 1079 | 23 | 3 | 2 |
| 374 | 9311377 | 245 | 1051 | 22 | 2 | 6 |
| 375 | 9311378 | 222 | 1247 | 16 | 5 | 3 |
| 376 | 9311379 | 241 | 973 | 27 | 1 | 2 |
| 377 | 9311380 | 268 | 1885 | 26 | 1 | 3 |
| 378 | 9311381 | 258 | 1356 | 31 | 8 | 3 |
| 379 | 9311382 | 203 | 1008 | 18 | 2 | 3 |
| 380 | 9311383 | 245 | 1386 | 21 | 4 | 5 |
| 381 | 9311384 | 215 | 1198 | 28 | 3 | 3 |
| 382 | 9311385 | 313 | 1638 | 31 | 4 | 4 |
| 383 | 9311386 | 210 | 1034 | 16 | 4 | 3 |
| 384 | 9311387 | 259 | 1082 | 22 | 5 | 3 |
| 385 | 9311388 | 286 | 1850 | 29 | 4 | 5 |
| 386 | 9311389 | 307 | 1955 | 31 | 2 | 3 |
| 387 | 9311390 | 228 | 967 | 29 | 3 | 4 |
| 388 | 9311391 | 265 | 1449 | 15 | 5 | 4 |
| 389 | 9311392 | 196 | 809 | 14 | 3 | 0 |
| 390 | 9311393 | 224 | 1234 | 11 | 3 | 3 |
| 391 | 9311394 | 253 | 1124 | 30 | 3 | 3 |
| 392 | 9311395 | 226 | 1253 | 25 | 3 | 1 |
| 393 | 9311396 | 225 | 1172 | 21 | 2 | 4 |
| 394 | 9311397 | 379 | 2801 | 43 | 3 | 3 |
| 395 | 9311398 | 270 | 1512 | 32 | 1 | 5 |
| 396 | 9311399 | 222 | 1158 | 19 | 5 | 1 |
| 397 | 9311400 | 201 | 945 | 18 | 1 | 4 |
| 398 | 9311401 | 186 | 1005 | 16 | 1 | 2 |
| 399 | 9311402 | 376 | 1716 | 36 | 0 | 5 |
| 400 | 9311403 | 246 | 1290 | 30 | 2 | 3 |
| 401 | 9311404 | 215 | 1063 | 22 | 2 | 3 |
| 402 | 9311405 | 195 | 647 | 21 | 5 | 5 |
| 403 | 9311406 | 254 | 1270 | 30 | 1 | 3 |
| 404 | 9311407 | 232 | 1222 | 22 | 4 | 1 |
| 405 | 9311408 | 318 | 1735 | 46 | 2 | 4 |
| 406 | 9311409 | 240 | 1562 | 27 | 3 | 3 |
| 407 | 9311410 | 200 | 986 | 25 | 4 | 5 |
| 408 | 9311411 | 253 | 1385 | 25 | 3 | 4 |
| 409 | 9311412 | 257 | 1412 | 35 | 6 | 2 |
| 410 | 9311413 | 176 | 1028 | 13 | 3 | 1 |
| 411 | 9311414 | 212 | 1208 | 22 | 3 | 1 |
| 412 | 9311415 | 215 | 1002 | 20 | 0 | 2 |
| 413 | 9311416 | 305 | 2041 | 22 | 4 | 2 |
| 414 | 9311417 | 216 | 888 | 18 | 5 | 2 |
| 415 | 9311418 | 231 | 1286 | 28 | 5 | 4 |
| 416 | 9311419 | 298 | 1847 | 39 | 4 | 2 |
| 417 | 9311420 | 238 | 1491 | 18 | 1 | 2 |
| 418 | 9311421 | 250 | 1238 | 14 | 2 | 4 |
| 419 | 9311422 | 238 | 1235 | 16 | 3 | 1 |
| 420 | 9311423 | 218 | 1562 | 27 | 6 | 2 |
| 421 | 9311424 | 248 | 1753 | 22 | 6 | 2 |
| 422 | 9311425 | 269 | 1673 | 41 | 8 | 3 |
| 423 | 9311426 | 281 | 1282 | 30 | 2 | 4 |
| 424 | 9311427 | 261 | 1879 | 26 | 4 | 2 |
| 425 | 9311428 | 201 | 1081 | 20 | 2 | 2 |
| 426 | 9311429 | 303 | 1758 | 46 | 2 | 1 |
| 427 | 9311430 | 212 | 1170 | 29 | 2 | 2 |
| 428 | 9311431 | 205 | 1274 | 16 | 6 | 3 |
| 429 | 9311432 | 161 | 721 | 10 | 2 | 0 |
| 430 | 9311433 | 271 | 1543 | 27 | 11 | 3 |
| 431 | 9311434 | 497 | 2830 | 60 | 2 | 4 |
| 432 | 9311435 | 287 | 1323 | 29 | 3 | 4 |
| 433 | 9311436 | 196 | 657 | 20 | 0 | 4 |
| 434 | 9311437 | 253 | 1571 | 31 | 5 | 4 |
| 435 | 9311438 | 184 | 705 | 11 | 3 | 3 |
| 436 | 9311439 | 327 | 2006 | 26 | 6 | 2 |
| 437 | 9311440 | 256 | 1167 | 24 | 4 | 2 |
| 438 | 9311441 | 279 | 1216 | 25 | 4 | 4 |
| 439 | 9311442 | 292 | 1766 | 28 | 4 | 1 |
| 440 | 9311443 | 263 | 1134 | 23 | 9 | 3 |
| 441 | 9311444 | 215 | 887 | 19 | 1 | 2 |
| 442 | 9311445 | 287 | 1389 | 42 | 7 | 4 |
| 443 | 9311446 | 259 | 1814 | 31 | 7 | 3 |
| 444 | 9311447 | 272 | 1603 | 27 | 3 | 3 |
| 445 | 9311448 | 213 | 1117 | 11 | 3 | 5 |
| 446 | 9311449 | 288 | 1637 | 37 | 3 | 2 |
| 447 | 9311450 | 294 | 1546 | 24 | 4 | 4 |
| 448 | 9311451 | 320 | 2168 | 38 | 7 | 4 |
| 449 | 9311452 | 216 | 1224 | 19 | 6 | 2 |
| 450 | 9311453 | 227 | 907 | 16 | 3 | 1 |
| 451 | 9311454 | 193 | 947 | 14 | 2 | 2 |
| 452 | 9311455 | 263 | 1259 | 31 | 2 | 3 |
| 453 | 9311456 | 206 | 785 | 21 | 4 | 4 |
| 454 | 9311457 | 266 | 1323 | 22 | 2 | 4 |
| 455 | 9311458 | 230 | 1117 | 23 | 3 | 4 |
| 456 | 9311459 | 297 | 1585 | 30 | 8 | 3 |
| 457 | 9311460 | 256 | 1641 | 23 | 9 | 2 |
| 458 | 9311461 | 181 | 1022 | 17 | 6 | 2 |
| 459 | 9311462 | 168 | 793 | 13 | 4 | 4 |
| 460 | 9311463 | 251 | 1421 | 29 | 1 | 3 |
| 461 | 9311464 | 279 | 1711 | 34 | 3 | 3 |
| 462 | 9311465 | 231 | 1091 | 17 | 7 | 5 |
| 463 | 9311466 | 263 | 1304 | 29 | 4 | 4 |
| 464 | 9311467 | 240 | 1170 | 22 | 0 | 4 |
| 465 | 9311468 | 193 | 977 | 7 | 4 | 2 |
| 466 | 9311469 | 165 | 867 | 13 | 4 | 3 |
| 467 | 9311470 | 344 | 1950 | 31 | 2 | 5 |
| 468 | 9311471 | 341 | 2050 | 46 | 2 | 4 |
| 469 | 9311472 | 212 | 1168 | 23 | 1 | 3 |
| 470 | 9311473 | 293 | 1484 | 14 | 1 | 4 |
| 471 | 9311474 | 264 | 1441 | 20 | 4 | 2 |
| 472 | 9311475 | 341 | 1527 | 36 | 1 | 2 |
| 473 | 9311476 | 197 | 1172 | 24 | 3 | 2 |
| 474 | 9311477 | 233 | 1164 | 27 | 2 | 4 |
| 475 | 9311478 | 210 | 1273 | 15 | 2 | 2 |
| 476 | 9311479 | 204 | 1304 | 15 | 2 | 1 |
| 477 | 9311480 | 243 | 1493 | 31 | 3 | 3 |
| 478 | 9311481 | 264 | 1251 | 21 | 0 | 4 |
| 479 | 9311482 | 262 | 1790 | 31 | 2 | 2 |
| 480 | 9311483 | 239 | 1283 | 19 | 4 | 2 |
| 481 | 9311484 | 184 | 781 | 19 | 2 | 3 |
| 482 | 9311485 | 228 | 1277 | 31 | 2 | 3 |
| 483 | 9311486 | 286 | 1622 | 33 | 1 | 4 |
| 484 | 9311487 | 295 | 1705 | 42 | 2 | 2 |
| 485 | 9311488 | 281 | 1709 | 26 | 1 | 3 |
| 486 | 9311489 | 258 | 1325 | 17 | 1 | 1 |
| 487 | 9311490 | 239 | 1231 | 24 | 2 | 2 |
| 488 | 9311491 | 243 | 1112 | 20 | 5 | 3 |
| 489 | 9311492 | 232 | 1375 | 30 | 1 | 2 |
| 490 | 9311493 | 175 | 883 | 17 | 2 | 4 |
| 491 | 9311494 | 214 | 1059 | 20 | 2 | 2 |
| 492 | 9311495 | 320 | 2007 | 29 | 3 | 2 |
| 493 | 9311496 | 204 | 1048 | 20 | 4 | 4 |
| 494 | 9311497 | 232 | 1183 | 25 | 0 | 2 |
| 495 | 9311498 | 225 | 1142 | 19 | 3 | 3 |
| 496 | 9311499 | 177 | 969 | 26 | 5 | 3 |
| 497 | 9311500 | 322 | 1903 | 37 | 6 | 2 |
| 498 | 9311501 | 283 | 1566 | 30 | 1 | 3 |
| 499 | 9311502 | 255 | 1450 | 33 | 4 | 4 |
| 500 | 9311503 | 458 | 3257 | 52 | 7 | 6 |
| 501 | 9311504 | 311 | 1617 | 39 | 2 | 3 |
| 502 | 9311505 | 189 | 1081 | 26 | 4 | 1 |
| 503 | 9311506 | 230 | 1228 | 18 | 0 | 3 |
| 504 | 9311507 | 293 | 1589 | 22 | 2 | 4 |
| 505 | 9311508 | 244 | 1558 | 27 | 4 | 0 |
| 506 | 9311509 | 255 | 1815 | 26 | 3 | 2 |
| 507 | 9311510 | 327 | 2333 | 42 | 1 | 2 |
| 508 | 9311511 | 246 | 1200 | 28 | 5 | 4 |
| 509 | 9311512 | 259 | 1243 | 27 | 2 | 3 |
| 510 | 9311513 | 198 | 974 | 11 | 4 | 3 |
| 511 | 9311514 | 250 | 1570 | 30 | 3 | 3 |
| 512 | 9311515 | 227 | 1287 | 27 | 4 | 2 |
| 513 | 9311516 | 199 | 1127 | 28 | 2 | 3 |
| 514 | 9311517 | 273 | 1610 | 29 | 2 | 2 |
| 515 | 9311518 | 194 | 969 | 25 | 1 | 3 |
| 516 | 9311519 | 199 | 1147 | 23 | 6 | 3 |
| 517 | 9311520 | 198 | 1035 | 13 | 5 | 2 |
| 518 | 9311521 | 329 | 1808 | 24 | 1 | 3 |
| 519 | 9311522 | 234 | 1290 | 25 | 3 | 4 |
| 520 | 9311523 | 257 | 1293 | 23 | 1 | 4 |
| 521 | 9311524 | 341 | 1941 | 33 | 1 | 3 |
| 522 | 9311525 | 210 | 1081 | 20 | 3 | 4 |
| 523 | 9311526 | 301 | 1545 | 29 | 1 | 5 |
| 524 | 9311527 | 213 | 1089 | 15 | 3 | 3 |
| 525 | 9311528 | 266 | 1416 | 30 | 3 | 2 |
| 526 | 9311529 | 229 | 1331 | 25 | 0 | 1 |
| 527 | 9311530 | 184 | 948 | 19 | 2 | 3 |
| 528 | 9311531 | 216 | 1044 | 23 | 3 | 5 |
| 529 | 9311532 | 284 | 1327 | 41 | 4 | 6 |
| 530 | 9311533 | 201 | 797 | 19 | 2 | 3 |
| 531 | 9311534 | 388 | 2133 | 55 | 4 | 3 |
| 532 | 9311535 | 246 | 1530 | 22 | 1 | 2 |
| 533 | 9311536 | 246 | 884 | 28 | 1 | 3 |
| 534 | 9311537 | 291 | 1703 | 39 | 8 | 3 |
| 535 | 9311538 | 212 | 1046 | 13 | 3 | 4 |
| 536 | 9311539 | 235 | 1254 | 20 | 1 | 4 |
| 537 | 9311540 | 178 | 765 | 16 | 4 | 3 |
| 538 | 9311541 | 228 | 707 | 16 | 7 | 6 |
| 539 | 9311542 | 240 | 868 | 15 | 6 | 2 |
| 540 | 9311543 | 253 | 1359 | 16 | 5 | 2 |
| 541 | 9311544 | 214 | 954 | 16 | 1 | 3 |
| 542 | 9311545 | 160 | 666 | 8 | 2 | 2 |
| 543 | 9311546 | 305 | 1551 | 41 | 6 | 3 |
| 544 | 9311547 | 227 | 931 | 27 | 3 | 3 |
| 545 | 9311548 | 220 | 1321 | 13 | 4 | 2 |
| 546 | 9311549 | 224 | 1433 | 18 | 1 | 3 |
| 547 | 9311550 | 238 | 1250 | 19 | 3 | 5 |
| 548 | 9311551 | 202 | 612 | 13 | 3 | 4 |
| 549 | 9311552 | 213 | 1099 | 19 | 1 | 3 |
| 550 | 9311553 | 248 | 1312 | 25 | 3 | 1 |
| 551 | 9311554 | 181 | 1018 | 24 | 3 | 4 |
| 552 | 9311555 | 199 | 909 | 19 | 2 | 2 |
| 553 | 9311556 | 224 | 1427 | 18 | 1 | 3 |
| 554 | 9311557 | 218 | 1124 | 26 | 1 | 4 |
| 555 | 9311558 | 219 | 968 | 13 | 6 | 3 |
| 556 | 9311559 | 220 | 1252 | 25 | 2 | 3 |
| 557 | 9311560 | 234 | 1597 | 26 | 4 | 1 |
| 558 | 9311561 | 205 | 894 | 17 | 1 | 2 |
| 559 | 9311562 | 259 | 1600 | 33 | 5 | 3 |
| 560 | 9311563 | 274 | 1753 | 27 | 5 | 3 |
| 561 | 9311564 | 239 | 818 | 23 | 7 | 4 |
| 562 | 9311565 | 260 | 1093 | 15 | 3 | 3 |
| 563 | 9311566 | 386 | 1925 | 32 | 4 | 4 |
| 564 | 9311567 | 259 | 1389 | 25 | 4 | 3 |
| 565 | 9311568 | 281 | 1393 | 26 | 3 | 5 |
| 566 | 9311569 | 252 | 1504 | 27 | 3 | 3 |
| 567 | 9311570 | 238 | 1107 | 17 | 3 | 3 |
| 568 | 9311571 | 226 | 1150 | 22 | 1 | 4 |
| 569 | 9311572 | 204 | 1157 | 27 | 1 | 3 |
| 570 | 9311573 | 223 | 1174 | 24 | 1 | 4 |
| 571 | 9311574 | 218 | 1181 | 20 | 2 | 3 |
| 572 | 9311575 | 276 | 1241 | 35 | 3 | 4 |
| 573 | 9311576 | 321 | 2013 | 37 | 4 | 3 |
| 574 | 9311577 | 206 | 1062 | 16 | 2 | 3 |
| 575 | 9311578 | 273 | 1320 | 21 | 4 | 3 |
| 576 | 9311579 | 264 | 1238 | 30 | 4 | 4 |
| 577 | 9311580 | 238 | 1194 | 22 | 0 | 3 |
| 578 | 9311581 | 245 | 1350 | 34 | 2 | 2 |
| 579 | 9311582 | 179 | 813 | 15 | 3 | 2 |
| 580 | 9311583 | 175 | 911 | 21 | 2 | 2 |
| 581 | 9311584 | 223 | 1393 | 34 | 1 | 4 |
| 582 | 9311585 | 197 | 1266 | 22 | 4 | 2 |
| 583 | 9311586 | 169 | 750 | 9 | 3 | 1 |
| 584 | 9311587 | 280 | 1956 | 41 | 2 | 2 |
| 585 | 9311588 | 215 | 1369 | 31 | 3 | 3 |
| 586 | 9311589 | 261 | 1200 | 22 | 0 | 2 |
| 587 | 9311590 | 185 | 1049 | 17 | 13 | 2 |
| 588 | 9311591 | 195 | 846 | 14 | 2 | 1 |
| 589 | 9311592 | 197 | 866 | 19 | 5 | 4 |
| 590 | 9311593 | 364 | 2458 | 48 | 5 | 5 |
| 591 | 9311594 | 189 | 885 | 23 | 1 | 3 |
| 592 | 9311595 | 272 | 1544 | 23 | 10 | 3 |
| 593 | 9311596 | 238 | 1299 | 17 | 3 | 2 |
| 594 | 9311597 | 252 | 1424 | 28 | 4 | 5 |
| 595 | 9311598 | 282 | 1504 | 36 | 4 | 4 |
| 596 | 9311599 | 268 | 1554 | 30 | 5 | 4 |
| 597 | 9311600 | 203 | 982 | 19 | 4 | 1 |
| 598 | 9311601 | 192 | 1092 | 11 | 2 | 1 |
| 599 | 9311602 | 237 | 1410 | 32 | 2 | 2 |
| 600 | 9311603 | 243 | 1448 | 32 | 0 | 4 |
| 601 | 9311604 | 272 | 1720 | 44 | 3 | 2 |
| 602 | 9311605 | 265 | 1322 | 16 | 5 | 6 |
| 603 | 9311606 | 237 | 1843 | 33 | 4 | 3 |
| 604 | 9311607 | 228 | 1368 | 25 | 5 | 4 |
| 605 | 9311608 | 193 | 695 | 14 | 5 | 4 |
| 606 | 9311609 | 222 | 1338 | 15 | 1 | 5 |
| 607 | 9311610 | 212 | 1334 | 26 | 6 | 2 |
| 608 | 9311611 | 253 | 1282 | 21 | 4 | 1 |
| 609 | 9311612 | 282 | 1759 | 21 | 6 | 3 |
| 610 | 9311613 | 255 | 1295 | 24 | 1 | 5 |
| 611 | 9311614 | 225 | 1010 | 14 | 4 | 3 |
| 612 | 9311615 | 185 | 811 | 19 | 3 | 2 |
| 613 | 9311616 | 212 | 1088 | 26 | 1 | 2 |
| 614 | 9311617 | 364 | 2126 | 33 | 2 | 3 |
| 615 | 9311618 | 291 | 1527 | 35 | 1 | 6 |
| 616 | 9311619 | 254 | 1170 | 13 | 7 | 3 |
| 617 | 9311620 | 199 | 990 | 24 | 12 | 1 |
| 618 | 9311621 | 245 | 1085 | 18 | 0 | 4 |
| 619 | 9311622 | 194 | 926 | 21 | 3 | 2 |
| 620 | 9311623 | 236 | 1100 | 23 | 1 | 3 |
| 621 | 9311624 | 265 | 1353 | 14 | 2 | 3 |
| 622 | 9311625 | 302 | 1786 | 25 | 4 | 3 |
| 623 | 9311626 | 243 | 1297 | 31 | 3 | 3 |
| 624 | 9311627 | 206 | 1247 | 26 | 11 | 3 |
| 625 | 9311628 | 299 | 1173 | 34 | 3 | 4 |
| 626 | 9311629 | 197 | 958 | 12 | 2 | 2 |
| 627 | 9311630 | 224 | 1452 | 30 | 2 | 1 |
| 628 | 9311631 | 204 | 1205 | 26 | 6 | 3 |
| 629 | 9311632 | 301 | 1281 | 21 | 1 | 2 |
| 630 | 9311633 | 287 | 1712 | 32 | 1 | 3 |
| 631 | 9311634 | 237 | 1326 | 25 | 4 | 2 |
| 632 | 9311635 | 288 | 1898 | 22 | 4 | 2 |
| 633 | 9311636 | 184 | 827 | 13 | 1 | 1 |
| 634 | 9311637 | 322 | 1507 | 37 | 6 | 4 |
| 635 | 9311638 | 221 | 870 | 25 | 6 | 3 |
| 636 | 9311639 | 199 | 1233 | 16 | 5 | 3 |
| 637 | 9311640 | 446 | 2842 | 47 | 8 | 4 |
| 638 | 9311641 | 302 | 1709 | 53 | 4 | 3 |
| 639 | 9311642 | 227 | 943 | 26 | 3 | 2 |
| 640 | 9311643 | 333 | 1916 | 27 | 5 | 5 |
| 641 | 9311644 | 300 | 1657 | 36 | 5 | 4 |
| 642 | 9311645 | 198 | 750 | 21 | 7 | 4 |
| 643 | 9311646 | 343 | 1616 | 35 | 2 | 5 |
| 644 | 9311647 | 266 | 1626 | 29 | 1 | 2 |
| 645 | 9311648 | 223 | 1217 | 25 | 2 | 4 |
| 646 | 9311649 | 187 | 893 | 12 | 0 | 2 |
| 647 | 9311650 | 218 | 911 | 23 | 2 | 3 |
| 648 | 9311651 | 285 | 1987 | 27 | 1 | 5 |
| 649 | 9311652 | 243 | 1200 | 25 | 3 | 3 |
| 650 | 9311653 | 254 | 1394 | 23 | 2 | 3 |
| 651 | 9311654 | 210 | 831 | 20 | 2 | 4 |
| 652 | 9311655 | 371 | 2141 | 28 | 4 | 4 |
| 653 | 9311656 | 315 | 1933 | 33 | 3 | 3 |
| 654 | 9311657 | 275 | 1494 | 31 | 2 | 4 |
| 655 | 9311658 | 227 | 1194 | 22 | 4 | 3 |
| 656 | 9311659 | 287 | 1744 | 27 | 4 | 1 |
| 657 | 9311660 | 156 | 711 | 11 | 3 | 1 |
| 658 | 9311661 | 240 | 1410 | 35 | 2 | 4 |
| 659 | 9311662 | 261 | 1626 | 29 | 3 | 3 |
| 660 | 9311663 | 247 | 1684 | 25 | 1 | 4 |
| 661 | 9311664 | 365 | 2690 | 28 | 1 | 2 |
| 662 | 9311665 | 304 | 1831 | 28 | 1 | 4 |
| 663 | 9311666 | 186 | 1028 | 16 | 3 | 2 |
| 664 | 9311667 | 233 | 1389 | 42 | 0 | 4 |
| 665 | 9311668 | 181 | 1282 | 24 | 1 | 1 |
| 666 | 9311669 | 231 | 1165 | 21 | 0 | 3 |
| 667 | 9311670 | 246 | 1191 | 29 | 3 | 3 |
| 668 | 9311671 | 211 | 1112 | 23 | 5 | 4 |
| 669 | 9311672 | 277 | 1536 | 30 | 1 | 2 |
| 670 | 9311673 | 337 | 1274 | 46 | 3 | 5 |
| 671 | 9311674 | 261 | 1671 | 30 | 8 | 3 |
| 672 | 9311675 | 153 | 668 | 11 | 0 | 2 |
| 673 | 9311676 | 316 | 2100 | 32 | 1 | 0 |
| 674 | 9311677 | 186 | 1127 | 14 | 4 | 1 |
| 675 | 9311678 | 175 | 873 | 18 | 4 | 1 |
| 676 | 9311679 | 215 | 1076 | 20 | 7 | 3 |
| 677 | 9311680 | 272 | 1706 | 33 | 3 | 5 |
| 678 | 9311681 | 232 | 1400 | 17 | 2 | 0 |
| 679 | 9311682 | 237 | 1263 | 25 | 3 | 1 |
| 680 | 9311683 | 279 | 1887 | 24 | 5 | 4 |
| 681 | 9311684 | 190 | 1095 | 17 | 1 | 3 |
| 682 | 9311685 | 289 | 1645 | 20 | 1 | 3 |
| 683 | 9311686 | 223 | 1188 | 22 | 2 | 4 |
| 684 | 9311687 | 199 | 1051 | 17 | 5 | 3 |
| 685 | 9311688 | 218 | 1123 | 19 | 4 | 3 |
| 686 | 9311689 | 268 | 1784 | 22 | 0 | 2 |
| 687 | 9311690 | 197 | 1221 | 18 | 2 | 2 |
| 688 | 9311691 | 225 | 1483 | 20 | 3 | 3 |
| 689 | 9311692 | 257 | 1780 | 23 | 9 | 1 |
| 690 | 9311693 | 236 | 1314 | 28 | 3 | 2 |
| 691 | 9311694 | 194 | 1109 | 22 | 3 | 2 |
| 692 | 9311695 | 246 | 1217 | 13 | 2 | 4 |
| 693 | 9311696 | 269 | 1548 | 33 | 3 | 1 |
| 694 | 9311697 | 227 | 1332 | 12 | 5 | 2 |
| 695 | 9311698 | 307 | 1693 | 34 | 3 | 3 |
| 696 | 9311699 | 234 | 860 | 16 | 3 | 5 |
| 697 | 9311700 | 203 | 912 | 20 | 3 | 4 |
| 698 | 9311701 | 270 | 1503 | 32 | 2 | 2 |
| 699 | 9311702 | 325 | 1466 | 33 | 4 | 3 |
| 700 | 9311703 | 304 | 1644 | 18 | 4 | 5 |
| 701 | 9311704 | 224 | 1237 | 31 | 1 | 3 |
| 702 | 9311705 | 450 | 1993 | 36 | 12 | 5 |
| 703 | 9311706 | 245 | 1056 | 25 | 2 | 1 |
| 704 | 9311707 | 281 | 1344 | 27 | 4 | 5 |
| 705 | 9311708 | 266 | 1301 | 30 | 3 | 4 |
| 706 | 9311709 | 219 | 1210 | 18 | 1 | 2 |
| 707 | 9311710 | 272 | 1301 | 29 | 4 | 3 |
| 708 | 9311711 | 231 | 1223 | 33 | 3 | 3 |
| 709 | 9311712 | 171 | 806 | 15 | 2 | 1 |
| 710 | 9311713 | 252 | 1402 | 26 | 4 | 3 |
| 711 | 9311714 | 310 | 1638 | 41 | 4 | 3 |
| 712 | 9311715 | 294 | 1421 | 21 | 5 | 2 |
| 713 | 9311716 | 429 | 2363 | 46 | 8 | 4 |
| 714 | 9311717 | 272 | 1239 | 18 | 1 | 2 |
| 715 | 9311718 | 310 | 1329 | 36 | 2 | 5 |
| 716 | 9311719 | 192 | 945 | 24 | 2 | 2 |
| 717 | 9311720 | 289 | 1351 | 40 | 5 | 3 |
| 718 | 9311721 | 246 | 1320 | 27 | 12 | 3 |
| 719 | 9311722 | 220 | 1438 | 39 | 6 | 2 |
| 720 | 9311723 | 196 | 1136 | 28 | 6 | 2 |
| 721 | 9311724 | 204 | 1306 | 25 | 1 | 2 |
| 722 | 9311725 | 280 | 1354 | 21 | 2 | 4 |
| 723 | 9311726 | 192 | 852 | 17 | 3 | 2 |
| 724 | 9311727 | 281 | 1453 | 16 | 7 | 4 |
| 725 | 9311728 | 247 | 1306 | 21 | 4 | 4 |
| 726 | 9311729 | 248 | 1448 | 22 | 4 | 2 |
| 727 | 9311730 | 220 | 1299 | 18 | 3 | 2 |
| 728 | 9311731 | 228 | 1026 | 20 | 1 | 5 |
| 729 | 9311732 | 237 | 1511 | 25 | 8 | 2 |
| 730 | 9311733 | 303 | 2074 | 26 | 2 | 4 |
| 731 | 9311734 | 248 | 786 | 24 | 8 | 4 |
| 732 | 9311735 | 307 | 1882 | 28 | 3 | 4 |
| 733 | 9311736 | 244 | 1246 | 27 | 4 | 3 |
| 734 | 9311737 | 279 | 1376 | 35 | 3 | 5 |
| 735 | 9311738 | 199 | 920 | 16 | 5 | 3 |
| 736 | 9311739 | 243 | 890 | 27 | 2 | 4 |
| 737 | 9311740 | 236 | 1490 | 22 | 2 | 4 |
| 738 | 9311741 | 220 | 1295 | 17 | 5 | 2 |
| 739 | 9311742 | 351 | 2202 | 53 | 9 | 4 |
| 740 | 9311743 | 217 | 1148 | 25 | 2 | 3 |
| 741 | 9311744 | 210 | 1072 | 15 | 1 | 3 |
| 742 | 9311745 | 365 | 2507 | 37 | 6 | 5 |
| 743 | 9311746 | 286 | 1632 | 21 | 0 | 5 |
| 744 | 9311747 | 277 | 1625 | 30 | 4 | 3 |
| 745 | 9311748 | 411 | 2443 | 58 | 1 | 5 |
| 746 | 9311749 | 264 | 1526 | 22 | 6 | 3 |
| 747 | 9311750 | 229 | 1391 | 25 | 0 | 2 |
| 748 | 9311751 | 304 | 1530 | 36 | 0 | 4 |
| 749 | 9311752 | 180 | 1072 | 14 | 2 | 2 |
| 750 | 9311753 | 168 | 749 | 14 | 2 | 3 |
| 751 | 9311754 | 289 | 1395 | 43 | 3 | 4 |
| 752 | 9311755 | 205 | 1164 | 26 | 0 | 1 |
| 753 | 9311756 | 212 | 1196 | 23 | 1 | 3 |
| 754 | 9311757 | 189 | 1006 | 27 | 4 | 1 |
| 755 | 9311758 | 257 | 1415 | 30 | 2 | 3 |
| 756 | 9311759 | 265 | 1369 | 23 | 4 | 4 |
| 757 | 9311760 | 498 | 3025 | 63 | 3 | 4 |
| 758 | 9311761 | 201 | 934 | 19 | 1 | 4 |
| 759 | 9311762 | 276 | 1321 | 26 | 5 | 2 |
| 760 | 9311763 | 192 | 1170 | 23 | 12 | 1 |
| 761 | 9311764 | 276 | 1323 | 30 | 7 | 4 |
| 762 | 9311765 | 221 | 1536 | 42 | 4 | 4 |
| 763 | 9311766 | 249 | 1159 | 28 | 6 | 2 |
| 764 | 9311767 | 214 | 1645 | 30 | 3 | 2 |
| 765 | 9311768 | 199 | 1082 | 25 | 6 | 5 |
| 766 | 9311769 | 229 | 1532 | 20 | 3 | 3 |
| 767 | 9311770 | 256 | 1103 | 37 | 7 | 3 |
| 768 | 9311771 | 237 | 1647 | 17 | 1 | 2 |
| 769 | 9311772 | 269 | 1457 | 23 | 11 | 3 |
| 770 | 9311773 | 216 | 1138 | 20 | 1 | 4 |
| 771 | 9311774 | 409 | 2301 | 38 | 6 | 6 |
| 772 | 9311775 | 249 | 1279 | 22 | 7 | 4 |
| 773 | 9311776 | 269 | 1281 | 30 | 2 | 4 |
| 774 | 9311777 | 232 | 1389 | 28 | 2 | 4 |
| 775 | 9311778 | 178 | 937 | 11 | 3 | 2 |
| 776 | 9311779 | 354 | 2076 | 44 | 9 | 3 |
| 777 | 9311780 | 180 | 1034 | 19 | 0 | 2 |
| 778 | 9311781 | 230 | 1296 | 15 | 0 | 4 |
| 779 | 9311782 | 277 | 1216 | 14 | 6 | 5 |
| 780 | 9311783 | 286 | 1450 | 27 | 1 | 4 |
| 781 | 9311784 | 234 | 1192 | 25 | 10 | 2 |
| 782 | 9311785 | 165 | 831 | 13 | 5 | 0 |
| 783 | 9311786 | 222 | 1125 | 22 | 3 | 3 |
| 784 | 9311787 | 319 | 1674 | 24 | 4 | 3 |
| 785 | 9311788 | 235 | 1417 | 25 | 2 | 5 |
| 786 | 9311789 | 256 | 1179 | 24 | 6 | 4 |
| 787 | 9311790 | 295 | 1471 | 40 | 7 | 3 |
| 788 | 9311791 | 339 | 1732 | 50 | 1 | 6 |
| 789 | 9311792 | 270 | 1191 | 17 | 6 | 5 |
| 790 | 9311793 | 293 | 1538 | 32 | 6 | 6 |
| 791 | 9311794 | 271 | 1218 | 22 | 1 | 3 |
| 792 | 9311795 | 240 | 1087 | 25 | 2 | 5 |
| 793 | 9311796 | 217 | 798 | 16 | 4 | 3 |
| 794 | 9311797 | 278 | 1219 | 27 | 3 | 3 |
| 795 | 9311798 | 312 | 1376 | 29 | 1 | 3 |
| 796 | 9311799 | 182 | 851 | 10 | 4 | 2 |
| 797 | 9311800 | 225 | 949 | 13 | 4 | 3 |
| 798 | 9311801 | 208 | 877 | 28 | 1 | 4 |
| 799 | 9311802 | 224 | 1165 | 20 | 1 | 2 |
| 800 | 9311803 | 243 | 1306 | 15 | 1 | 2 |
| 801 | 9311804 | 219 | 1222 | 25 | 5 | 3 |
| 802 | 9311805 | 223 | 1132 | 22 | 1 | 4 |
| 803 | 9311806 | 203 | 1032 | 18 | 1 | 3 |
| 804 | 9311807 | 215 | 1259 | 22 | 0 | 1 |
| 805 | 9311808 | 304 | 1853 | 37 | 1 | 3 |
| 806 | 9311809 | 297 | 1297 | 30 | 1 | 2 |
| 807 | 9311810 | 241 | 1131 | 18 | 5 | 3 |
| 808 | 9311811 | 276 | 1638 | 31 | 8 | 4 |
| 809 | 9311812 | 248 | 1466 | 21 | 7 | 3 |
| 810 | 9311813 | 351 | 2224 | 34 | 5 | 2 |
| 811 | 9311814 | 222 | 1356 | 25 | 5 | 2 |
| 812 | 9311815 | 222 | 1325 | 31 | 5 | 5 |
| 813 | 9311816 | 316 | 1718 | 36 | 15 | 3 |
| 814 | 9311817 | 246 | 1488 | 21 | 1 | 1 |
| 815 | 9311818 | 236 | 1303 | 11 | 2 | 3 |
| 816 | 9311819 | 235 | 1503 | 27 | 0 | 3 |
| 817 | 9311820 | 224 | 1116 | 15 | 2 | 3 |
| 818 | 9311821 | 253 | 1603 | 21 | 3 | 3 |
| 819 | 9311822 | 220 | 1016 | 27 | 6 | 3 |
| 820 | 9311823 | 224 | 1116 | 19 | 2 | 4 |
| 821 | 9311824 | 280 | 1296 | 12 | 7 | 3 |
| 822 | 9311825 | 303 | 1850 | 39 | 5 | 3 |
| 823 | 9311826 | 230 | 1390 | 28 | 1 | 2 |
| 824 | 9311827 | 291 | 1820 | 22 | 2 | 5 |
| 825 | 9311828 | 214 | 1225 | 18 | 5 | 2 |
| 826 | 9311829 | 234 | 931 | 22 | 3 | 4 |
| 827 | 9311830 | 271 | 1490 | 38 | 2 | 2 |
| 828 | 9311831 | 225 | 1286 | 24 | 1 | 3 |
| 829 | 9311832 | 288 | 1613 | 33 | 5 | 1 |
| 830 | 9311833 | 205 | 1149 | 21 | 4 | 1 |
| 831 | 9311834 | 432 | 2441 | 65 | 0 | 6 |
| 832 | 9311835 | 219 | 1232 | 29 | 6 | 2 |
| 833 | 9311836 | 268 | 1764 | 38 | 1 | 4 |
| 834 | 9311837 | 222 | 1142 | 34 | 3 | 3 |
| 835 | 9311838 | 278 | 2051 | 36 | 7 | 3 |
| 836 | 9311839 | 215 | 1239 | 37 | 0 | 3 |
| 837 | 9311840 | 212 | 1018 | 20 | 2 | 3 |
| 838 | 9311841 | 165 | 614 | 19 | 0 | 2 |
| 839 | 9311842 | 208 | 1187 | 22 | 4 | 3 |
| 840 | 9311843 | 164 | 881 | 10 | 0 | 3 |
| 841 | 9311844 | 223 | 892 | 14 | 4 | 4 |
| 842 | 9311845 | 259 | 1394 | 28 | 3 | 3 |
| 843 | 9311846 | 256 | 1520 | 29 | 6 | 4 |
| 844 | 9311847 | 281 | 1805 | 37 | 1 | 2 |
| 845 | 9311848 | 245 | 1090 | 34 | 5 | 3 |
| 846 | 9311849 | 250 | 958 | 25 | 1 | 5 |
| 847 | 9311850 | 177 | 1046 | 7 | 6 | 1 |
| 848 | 9311851 | 227 | 1274 | 33 | 2 | 2 |
| 849 | 9311852 | 273 | 1948 | 38 | 7 | 0 |
| 850 | 9311853 | 277 | 1576 | 30 | 3 | 4 |
| 851 | 9311854 | 218 | 1356 | 31 | 4 | 4 |
| 852 | 9311855 | 265 | 1668 | 33 | 2 | 4 |
| 853 | 9311856 | 262 | 1671 | 40 | 2 | 2 |
| 854 | 9311857 | 179 | 897 | 18 | 0 | 2 |
| 855 | 9311858 | 239 | 1121 | 12 | 3 | 0 |
| 856 | 9311859 | 260 | 1334 | 31 | 11 | 4 |
| 857 | 9311860 | 240 | 1559 | 24 | 3 | 2 |
| 858 | 9311861 | 174 | 869 | 12 | 1 | 2 |
| 859 | 9311862 | 206 | 1244 | 30 | 4 | 3 |
| 860 | 9311863 | 232 | 1492 | 27 | 1 | 4 |
| 861 | 9311864 | 206 | 1010 | 23 | 3 | 3 |
| 862 | 9311865 | 229 | 1351 | 14 | 8 | 3 |
| 863 | 9311866 | 223 | 1412 | 35 | 3 | 3 |
| 864 | 9311867 | 212 | 1058 | 24 | 4 | 2 |
| 865 | 9311868 | 279 | 1631 | 34 | 2 | 3 |
| 866 | 9311869 | 221 | 1297 | 26 | 4 | 4 |
| 867 | 9311870 | 180 | 815 | 13 | 1 | 3 |
| 868 | 9311871 | 251 | 1002 | 13 | 6 | 2 |
| 869 | 9311872 | 260 | 1705 | 31 | 5 | 1 |
| 870 | 9311873 | 225 | 1009 | 25 | 7 | 4 |
| 871 | 9311874 | 270 | 1528 | 28 | 6 | 4 |
| 872 | 9311875 | 271 | 1324 | 31 | 0 | 4 |
| 873 | 9311876 | 344 | 1522 | 39 | 6 | 5 |
| 874 | 9311877 | 292 | 1631 | 32 | 3 | 3 |
| 875 | 9311878 | 275 | 1378 | 21 | 7 | 3 |
| 876 | 9311879 | 279 | 1671 | 40 | 2 | 4 |
| 877 | 9311880 | 237 | 1145 | 32 | 3 | 4 |
| 878 | 9311881 | 199 | 883 | 14 | 1 | 4 |
| 879 | 9311882 | 310 | 1278 | 25 | 3 | 6 |
| 880 | 9311883 | 269 | 1308 | 26 | 3 | 4 |
| 881 | 9311884 | 286 | 1705 | 29 | 8 | 2 |
| 882 | 9311885 | 221 | 1080 | 25 | 2 | 4 |
| 883 | 9311886 | 261 | 1306 | 26 | 0 | 4 |
| 884 | 9311887 | 249 | 1150 | 28 | 5 | 4 |
| 885 | 9311888 | 238 | 1242 | 17 | 1 | 4 |
| 886 | 9311889 | 259 | 1574 | 34 | 2 | 4 |
| 887 | 9311890 | 215 | 950 | 12 | 5 | 2 |
| 888 | 9311891 | 341 | 1861 | 48 | 3 | 3 |
| 889 | 9311892 | 294 | 1203 | 24 | 9 | 3 |
| 890 | 9311893 | 290 | 1847 | 16 | 8 | 3 |
| 891 | 9311894 | 222 | 1016 | 17 | 0 | 3 |
| 892 | 9311895 | 203 | 818 | 9 | 1 | 3 |
| 893 | 9311896 | 222 | 1181 | 26 | 3 | 2 |
| 894 | 9311897 | 284 | 1392 | 29 | 5 | 4 |
| 895 | 9311898 | 231 | 1373 | 18 | 5 | 3 |
| 896 | 9311899 | 395 | 2576 | 40 | 6 | 3 |
| 897 | 9311900 | 216 | 1234 | 19 | 5 | 1 |
| 898 | 9311901 | 227 | 1115 | 20 | 1 | 3 |
| 899 | 9311902 | 256 | 1574 | 18 | 4 | 3 |
| 900 | 9311903 | 216 | 1133 | 31 | 5 | 3 |
| 901 | 9311904 | 270 | 1353 | 34 | 4 | 3 |
| 902 | 9311905 | 249 | 1599 | 21 | 8 | 2 |
| 903 | 9311906 | 254 | 1518 | 21 | 5 | 1 |
| 904 | 9311907 | 328 | 1636 | 29 | 7 | 4 |
| 905 | 9311908 | 223 | 1094 | 15 | 3 | 3 |
| 906 | 9311909 | 241 | 1530 | 29 | 6 | 1 |
| 907 | 9311910 | 273 | 1582 | 27 | 1 | 4 |
| 908 | 9311911 | 179 | 877 | 12 | 0 | 1 |
| 909 | 9311912 | 198 | 814 | 16 | 1 | 3 |
| 910 | 9311913 | 231 | 1272 | 24 | 2 | 3 |
| 911 | 9311914 | 191 | 935 | 10 | 3 | 4 |
| 912 | 9311915 | 309 | 2008 | 22 | 4 | 5 |
| 913 | 9311916 | 280 | 2021 | 25 | 6 | 4 |
| 914 | 9311917 | 366 | 2325 | 44 | 8 | 5 |
| 915 | 9311918 | 287 | 1447 | 35 | 3 | 3 |
| 916 | 9311919 | 257 | 1325 | 22 | 3 | 3 |
| 917 | 9311920 | 238 | 1373 | 25 | 2 | 4 |
| 918 | 9311921 | 247 | 1274 | 28 | 1 | 4 |
| 919 | 9311922 | 273 | 1222 | 28 | 1 | 5 |
| 920 | 9311923 | 218 | 1279 | 34 | 0 | 4 |
| 921 | 9311924 | 178 | 842 | 15 | 2 | 3 |
| 922 | 9311925 | 337 | 1947 | 47 | 5 | 3 |
| 923 | 9311926 | 246 | 1115 | 20 | 4 | 3 |
| 924 | 9311927 | 243 | 1020 | 23 | 3 | 4 |
| 925 | 9311928 | 202 | 1029 | 19 | 0 | 4 |
| 926 | 9311929 | 314 | 1960 | 41 | 3 | 4 |
| 927 | 9311930 | 263 | 1082 | 20 | 3 | 3 |
| 928 | 9311931 | 237 | 844 | 17 | 4 | 4 |
| 929 | 9311932 | 300 | 1640 | 26 | 3 | 4 |
| 930 | 9311933 | 317 | 1571 | 26 | 2 | 4 |
| 931 | 9311934 | 254 | 1450 | 30 | 3 | 4 |
| 932 | 9311935 | 191 | 974 | 19 | 2 | 4 |
| 933 | 9311936 | 317 | 1745 | 38 | 5 | 3 |
| 934 | 9311937 | 228 | 1049 | 20 | 3 | 5 |
| 935 | 9311938 | 197 | 993 | 13 | 2 | 2 |
| 936 | 9311939 | 277 | 1401 | 39 | 7 | 3 |
| 937 | 9311940 | 217 | 1399 | 31 | 2 | 2 |
| 938 | 9311941 | 283 | 1884 | 21 | 3 | 1 |
| 939 | 9311942 | 226 | 1067 | 29 | 4 | 3 |
| 940 | 9311943 | 191 | 1022 | 23 | 8 | 4 |
| 941 | 9311944 | 206 | 1128 | 26 | 2 | 0 |
| 942 | 9311945 | 202 | 1108 | 20 | 1 | 3 |
| 943 | 9311946 | 175 | 1264 | 24 | 6 | 1 |
| 944 | 9311947 | 208 | 1135 | 15 | 7 | 4 |
| 945 | 9311948 | 267 | 1375 | 23 | 1 | 3 |
| 946 | 9311949 | 237 | 1324 | 18 | 4 | 2 |
| 947 | 9311950 | 180 | 735 | 16 | 3 | 0 |
| 948 | 9311951 | 320 | 2204 | 26 | 6 | 2 |
| 949 | 9311952 | 230 | 1037 | 25 | 6 | 3 |
| 950 | 9311953 | 223 | 1170 | 16 | 5 | 2 |
| 951 | 9311954 | 227 | 1078 | 16 | 10 | 3 |
| 952 | 9311955 | 276 | 1759 | 39 | 7 | 3 |
| 953 | 9311956 | 208 | 1052 | 21 | 3 | 3 |
| 954 | 9311957 | 349 | 2148 | 41 | 5 | 5 |
| 955 | 9311958 | 209 | 1098 | 20 | 1 | 3 |
| 956 | 9311959 | 216 | 1253 | 25 | 4 | 4 |
| 957 | 9311960 | 292 | 1538 | 46 | 7 | 5 |
| 958 | 9311961 | 179 | 947 | 13 | 2 | 2 |
| 959 | 9311962 | 242 | 1135 | 30 | 2 | 3 |
| 960 | 9311963 | 259 | 1338 | 26 | 5 | 3 |
| 961 | 9311964 | 196 | 765 | 16 | 3 | 4 |
| 962 | 9311965 | 344 | 1736 | 37 | 4 | 6 |
| 963 | 9311966 | 268 | 1272 | 30 | 1 | 3 |
| 964 | 9311967 | 256 | 1336 | 24 | 1 | 5 |
| 965 | 9311968 | 374 | 1924 | 51 | 1 | 6 |
| 966 | 9311969 | 229 | 1126 | 26 | 2 | 4 |
| 967 | 9311970 | 224 | 1191 | 27 | 0 | 2 |
| 968 | 9311971 | 285 | 1368 | 33 | 3 | 5 |
| 969 | 9311972 | 197 | 1134 | 19 | 6 | 2 |
| 970 | 9311973 | 271 | 1588 | 30 | 1 | 2 |
| 971 | 9311974 | 209 | 1219 | 13 | 5 | 3 |
| 972 | 9311975 | 262 | 1172 | 22 | 8 | 3 |
| 973 | 9311976 | 295 | 1641 | 32 | 6 | 4 |
| 974 | 9311977 | 288 | 1249 | 25 | 7 | 5 |
| 975 | 9311978 | 204 | 1164 | 20 | 3 | 2 |
| 976 | 9311979 | 282 | 1577 | 17 | 6 | 4 |
| 977 | 9311980 | 264 | 1571 | 17 | 6 | 3 |
| 978 | 9311981 | 219 | 1161 | 23 | 3 | 2 |
| 979 | 9311982 | 305 | 1824 | 38 | 2 | 2 |
| 980 | 9311983 | 212 | 915 | 14 | 0 | 4 |
| 981 | 9311984 | 279 | 1469 | 35 | 6 | 2 |
| 982 | 9311985 | 210 | 952 | 28 | 1 | 4 |
| 983 | 9311986 | 367 | 1726 | 39 | 4 | 3 |
| 984 | 9311987 | 273 | 1649 | 31 | 5 | 3 |
| 985 | 9311988 | 234 | 1169 | 21 | 3 | 3 |
| 986 | 9311989 | 276 | 1368 | 18 | 4 | 3 |
| 987 | 9311990 | 204 | 983 | 18 | 2 | 3 |
| 988 | 9311991 | 358 | 2200 | 34 | 14 | 4 |
| 989 | 9311992 | 226 | 1226 | 17 | 3 | 3 |
| 990 | 9311993 | 221 | 1106 | 18 | 7 | 4 |
| 991 | 9311994 | 245 | 1113 | 20 | 3 | 4 |
| 992 | 9311995 | 319 | 1865 | 43 | 2 | 5 |
| 993 | 9311996 | 260 | 1480 | 19 | 8 | 3 |
| 994 | 9311997 | 500 | 2374 | 44 | 4 | 4 |
| 995 | 9311998 | 242 | 988 | 26 | 2 | 4 |
| 996 | 9311999 | 244 | 1466 | 19 | 11 | 2 |
| 997 | 9312000 | 311 | 1595 | 19 | 7 | 5 |
| 998 | 9312001 | 226 | 1070 | 26 | 1 | 3 |
| 999 | 9312002 | 193 | 709 | 22 | 2 | 4 |
| 1000 | 9312003 | 260 | 1501 | 34 | 3 | 1 |
| 1001 | 9312004 | 313 | 1352 | 28 | 3 | 2 |
| 1002 | 9312005 | 239 | 1108 | 19 | 1 | 3 |
| 1003 | 9312006 | 206 | 921 | 27 | 2 | 5 |
| 1004 | 9312007 | 264 | 1266 | 17 | 3 | 3 |
| 1005 | 9312008 | 183 | 1145 | 20 | 4 | 3 |
| 1006 | 9312009 | 320 | 1479 | 38 | 0 | 5 |
| 1007 | 9312010 | 270 | 1205 | 32 | 3 | 5 |
| 1008 | 9312011 | 261 | 1135 | 26 | 5 | 3 |
| 1009 | 9312012 | 186 | 997 | 22 | 2 | 2 |
| 1010 | 9312013 | 180 | 986 | 13 | 1 | 4 |
| 1011 | 9312014 | 289 | 1544 | 29 | 0 | 4 |
| 1012 | 9312015 | 225 | 1226 | 27 | 3 | 5 |
| 1013 | 9312016 | 218 | 1105 | 26 | 2 | 4 |
| 1014 | 9312017 | 279 | 1688 | 23 | 3 | 4 |
| 1015 | 9312018 | 288 | 1781 | 35 | 2 | 2 |
| 1016 | 9312019 | 344 | 2054 | 37 | 1 | 3 |
| 1017 | 9312020 | 198 | 1399 | 18 | 1 | 0 |
| 1018 | 9312021 | 195 | 846 | 17 | 0 | 2 |
| 1019 | 9312022 | 258 | 1582 | 30 | 5 | 4 |
| 1020 | 9312023 | 232 | 1037 | 29 | 4 | 5 |
| 1021 | 9312024 | 302 | 1580 | 35 | 1 | 6 |
| 1022 | 9312025 | 211 | 1053 | 16 | 0 | 2 |
| 1023 | 9312026 | 198 | 928 | 20 | 2 | 3 |
| 1024 | 9312027 | 277 | 1742 | 38 | 1 | 3 |
| 1025 | 9312028 | 225 | 1732 | 25 | 4 | 1 |
| 1026 | 9312029 | 321 | 1506 | 34 | 7 | 3 |
| 1027 | 9312030 | 253 | 1328 | 46 | 6 | 5 |
| 1028 | 9312031 | 183 | 974 | 18 | 6 | 2 |
| 1029 | 9312032 | 237 | 1698 | 29 | 4 | 3 |
| 1030 | 9312033 | 279 | 1237 | 25 | 9 | 4 |
| 1031 | 9312034 | 213 | 1180 | 23 | 2 | 2 |
| 1032 | 9312035 | 280 | 1995 | 34 | 5 | 3 |
| 1033 | 9312036 | 239 | 1225 | 26 | 8 | 4 |
| 1034 | 9312037 | 261 | 1377 | 29 | 2 | 4 |
| 1035 | 9312038 | 143 | 665 | 12 | 1 | 1 |
| 1036 | 9312039 | 305 | 1614 | 22 | 1 | 3 |
| 1037 | 9312040 | 277 | 1436 | 15 | 3 | 5 |
| 1038 | 9312041 | 273 | 1507 | 35 | 1 | 5 |
| 1039 | 9312042 | 339 | 2235 | 28 | 2 | 4 |
| 1040 | 9312043 | 182 | 871 | 15 | 4 | 3 |
| 1041 | 9312044 | 244 | 1459 | 18 | 1 | 1 |
| 1042 | 9312045 | 185 | 780 | 15 | 2 | 0 |
| 1043 | 9312046 | 276 | 1785 | 38 | 5 | 4 |
| 1044 | 9312047 | 183 | 1034 | 26 | 5 | 2 |
| 1045 | 9312048 | 288 | 1690 | 42 | 5 | 2 |
| 1046 | 9312049 | 319 | 1633 | 44 | 8 | 4 |
| 1047 | 9312050 | 233 | 1081 | 13 | 8 | 4 |
| 1048 | 9312051 | 264 | 1218 | 39 | 0 | 4 |
| 1049 | 9312052 | 305 | 1174 | 26 | 7 | 4 |
| 1050 | 9312053 | 288 | 1445 | 36 | 0 | 4 |
| 1051 | 9312054 | 378 | 1378 | 21 | 4 | 4 |
| 1052 | 9312055 | 222 | 1027 | 16 | 2 | 4 |
| 1053 | 9312056 | 183 | 789 | 17 | 0 | 1 |
| 1054 | 9312057 | 333 | 1687 | 33 | 6 | 4 |
| 1055 | 9312058 | 249 | 1081 | 21 | 1 | 4 |
| 1056 | 9312059 | 324 | 1964 | 28 | 2 | 2 |
| 1057 | 9312060 | 233 | 1069 | 30 | 3 | 5 |
| 1058 | 9312061 | 248 | 1255 | 16 | 2 | 2 |
| 1059 | 9312062 | 207 | 978 | 19 | 4 | 2 |
| 1060 | 9312063 | 229 | 1122 | 14 | 0 | 3 |
| 1061 | 9312064 | 284 | 1493 | 35 | 3 | 3 |
| 1062 | 9312065 | 245 | 1154 | 12 | 7 | 2 |
| 1063 | 9312066 | 217 | 967 | 14 | 1 | 1 |
| 1064 | 9312067 | 277 | 1460 | 24 | 1 | 4 |
| 1065 | 9312068 | 267 | 1666 | 38 | 4 | 3 |
| 1066 | 9312069 | 204 | 1175 | 11 | 3 | 3 |
| 1067 | 9312070 | 253 | 1310 | 18 | 4 | 3 |
| 1068 | 9312071 | 339 | 1790 | 24 | 3 | 1 |
| 1069 | 9312072 | 284 | 1369 | 28 | 4 | 5 |
| 1070 | 9312073 | 279 | 1575 | 25 | 2 | 5 |
| 1071 | 9312074 | 264 | 1391 | 32 | 2 | 3 |
| 1072 | 9312075 | 233 | 1333 | 26 | 4 | 3 |
| 1073 | 9312076 | 317 | 1514 | 34 | 3 | 4 |
| 1074 | 9312077 | 276 | 1617 | 30 | 10 | 4 |
| 1075 | 9312078 | 253 | 1442 | 21 | 2 | 4 |
| 1076 | 9312079 | 318 | 2008 | 34 | 1 | 4 |
| 1077 | 9312080 | 223 | 1074 | 16 | 6 | 2 |
| 1078 | 9312081 | 297 | 1971 | 34 | 3 | 4 |
| 1079 | 9312082 | 180 | 772 | 12 | 0 | 3 |
| 1080 | 9312083 | 500 | 2473 | 31 | 0 | 5 |
| 1081 | 9312084 | 232 | 1166 | 23 | 6 | 3 |
| 1082 | 9312085 | 236 | 1275 | 33 | 7 | 4 |
| 1083 | 9312086 | 238 | 1334 | 19 | 4 | 2 |
| 1084 | 9312087 | 236 | 1129 | 21 | 5 | 2 |
| 1085 | 9312088 | 385 | 1733 | 44 | 1 | 3 |
| 1086 | 9312089 | 230 | 1408 | 22 | 3 | 2 |
| 1087 | 9312090 | 193 | 1018 | 19 | 3 | 4 |
| 1088 | 9312091 | 182 | 802 | 9 | 5 | 1 |
| 1089 | 9312092 | 268 | 1640 | 24 | 1 | 3 |
| 1090 | 9312093 | 211 | 1180 | 12 | 0 | 2 |
| 1091 | 9312094 | 212 | 1122 | 19 | 0 | 3 |
| 1092 | 9312095 | 203 | 1076 | 16 | 1 | 2 |
| 1093 | 9312096 | 259 | 1576 | 28 | 2 | 4 |
| 1094 | 9312097 | 257 | 1398 | 26 | 4 | 5 |
| 1095 | 9312098 | 185 | 936 | 19 | 1 | 2 |
| 1096 | 9312099 | 231 | 1333 | 26 | 2 | 4 |
| 1097 | 9312100 | 207 | 1217 | 30 | 0 | 4 |
| 1098 | 9312101 | 223 | 835 | 19 | 3 | 5 |
| 1099 | 9312102 | 155 | 431 | 9 | 2 | 1 |
| 1100 | 9312103 | 166 | 802 | 16 | 0 | 2 |
| 1101 | 9312104 | 260 | 1397 | 29 | 2 | 3 |
| 1102 | 9312105 | 416 | 2506 | 28 | 2 | 3 |
| 1103 | 9312106 | 237 | 1758 | 23 | 2 | 2 |
| 1104 | 9312107 | 158 | 766 | 10 | 3 | 1 |
| 1105 | 9312108 | 255 | 1319 | 31 | 4 | 2 |
| 1106 | 9312109 | 289 | 1457 | 30 | 3 | 4 |
| 1107 | 9312110 | 174 | 778 | 17 | 0 | 2 |
| 1108 | 9312111 | 173 | 1012 | 19 | 0 | 2 |
| 1109 | 9312112 | 290 | 1873 | 21 | 1 | 2 |
| 1110 | 9312113 | 230 | 1437 | 29 | 1 | 1 |
| 1111 | 9312114 | 197 | 901 | 19 | 1 | 4 |
| 1112 | 9312115 | 239 | 1325 | 25 | 5 | 4 |
| 1113 | 9312116 | 205 | 816 | 10 | 5 | 2 |
| 1114 | 9312117 | 218 | 1294 | 28 | 2 | 2 |
| 1115 | 9312118 | 182 | 989 | 21 | 2 | 3 |
| 1116 | 9312119 | 250 | 1637 | 27 | 1 | 4 |
| 1117 | 9312120 | 241 | 1118 | 20 | 16 | 2 |
| 1118 | 9312121 | 308 | 1616 | 35 | 8 | 3 |
| 1119 | 9312122 | 219 | 1369 | 14 | 10 | 3 |
| 1120 | 9312123 | 218 | 1558 | 18 | 2 | 1 |
| 1121 | 9312124 | 239 | 1247 | 32 | 3 | 1 |
| 1122 | 9312125 | 208 | 1426 | 13 | 4 | 3 |
| 1123 | 9312126 | 258 | 1534 | 19 | 0 | 1 |
| 1124 | 9312127 | 247 | 1364 | 22 | 6 | 4 |
| 1125 | 9312128 | 193 | 1039 | 26 | 4 | 2 |
| 1126 | 9312129 | 224 | 1225 | 19 | 4 | 2 |
| 1127 | 9312130 | 249 | 1567 | 28 | 3 | 2 |
| 1128 | 9312131 | 254 | 1754 | 32 | 4 | 2 |
| 1129 | 9312132 | 209 | 980 | 28 | 6 | 2 |
| 1130 | 9312133 | 223 | 1559 | 33 | 2 | 2 |
| 1131 | 9312134 | 283 | 1266 | 21 | 2 | 5 |
| 1132 | 9312135 | 265 | 1570 | 36 | 4 | 5 |
| 1133 | 9312136 | 244 | 1617 | 34 | 2 | 1 |
| 1134 | 9312137 | 198 | 1291 | 21 | 2 | 1 |
| 1135 | 9312138 | 219 | 1469 | 16 | 0 | 2 |
| 1136 | 9312139 | 299 | 1268 | 10 | 1 | 6 |
| 1137 | 9312140 | 470 | 2369 | 48 | 3 | 5 |
| 1138 | 9312141 | 245 | 1277 | 33 | 2 | 3 |
| 1139 | 9312142 | 179 | 850 | 17 | 4 | 2 |
| 1140 | 9312143 | 236 | 1209 | 20 | 1 | 2 |
| 1141 | 9312144 | 351 | 1558 | 31 | 5 | 5 |
| 1142 | 9312145 | 219 | 1009 | 20 | 8 | 4 |
| 1143 | 9312146 | 275 | 1540 | 31 | 1 | 4 |
| 1144 | 9312147 | 372 | 1796 | 31 | 2 | 5 |
| 1145 | 9312148 | 289 | 1330 | 24 | 1 | 6 |
| 1146 | 9312149 | 238 | 1242 | 12 | 10 | 4 |
| 1147 | 9312150 | 246 | 1234 | 35 | 3 | 4 |
| 1148 | 9312151 | 190 | 948 | 9 | 1 | 2 |
| 1149 | 9312152 | 176 | 868 | 19 | 1 | 2 |
| 1150 | 9312153 | 235 | 1289 | 18 | 6 | 5 |
| 1151 | 9312154 | 313 | 1639 | 17 | 7 | 3 |
| 1152 | 9312155 | 209 | 1190 | 20 | 3 | 3 |
| 1153 | 9312156 | 253 | 1267 | 35 | 9 | 3 |
| 1154 | 9312157 | 280 | 1508 | 19 | 7 | 4 |
| 1155 | 9312158 | 186 | 892 | 11 | 4 | 1 |
| 1156 | 9312159 | 240 | 1423 | 23 | 3 | 3 |
| 1157 | 9312160 | 260 | 1568 | 24 | 6 | 2 |
| 1158 | 9312161 | 234 | 1150 | 22 | 4 | 2 |
| 1159 | 9312162 | 247 | 1488 | 25 | 3 | 3 |
| 1160 | 9312163 | 194 | 1018 | 10 | 3 | 1 |
| 1161 | 9312164 | 271 | 1341 | 27 | 7 | 2 |
| 1162 | 9312165 | 198 | 1137 | 15 | 5 | 1 |
| 1163 | 9312166 | 269 | 1524 | 23 | 6 | 3 |
| 1164 | 9312167 | 206 | 1318 | 21 | 4 | 6 |
| 1165 | 9312168 | 244 | 1286 | 28 | 4 | 1 |
| 1166 | 9312169 | 253 | 1723 | 23 | 9 | 2 |
| 1167 | 9312170 | 205 | 1086 | 21 | 2 | 3 |
| 1168 | 9312171 | 222 | 689 | 22 | 4 | 3 |
| 1169 | 9312172 | 240 | 1276 | 24 | 2 | 4 |
| 1170 | 9312173 | 187 | 775 | 16 | 1 | 4 |
| 1171 | 9312174 | 204 | 1288 | 25 | 4 | 5 |
| 1172 | 9312175 | 248 | 1184 | 28 | 3 | 4 |
| 1173 | 9312176 | 418 | 2365 | 48 | 7 | 4 |
| 1174 | 9312177 | 209 | 683 | 22 | 2 | 5 |
| 1175 | 9312178 | 278 | 1686 | 16 | 3 | 4 |
| 1176 | 9312179 | 223 | 1425 | 36 | 1 | 2 |
| 1177 | 9312180 | 302 | 1889 | 24 | 6 | 4 |
| 1178 | 9312181 | 228 | 1305 | 19 | 1 | 2 |
| 1179 | 9312182 | 229 | 1264 | 19 | 2 | 2 |
| 1180 | 9312183 | 265 | 1168 | 22 | 0 | 4 |
| 1181 | 9312184 | 243 | 1419 | 21 | 2 | 3 |
| 1182 | 9312185 | 242 | 1150 | 36 | 3 | 3 |
| 1183 | 9312186 | 228 | 1057 | 12 | 0 | 4 |
| 1184 | 9312187 | 272 | 1470 | 30 | 3 | 4 |
| 1185 | 9312188 | 230 | 1376 | 26 | 1 | 4 |
| 1186 | 9312189 | 302 | 1568 | 42 | 4 | 4 |
| 1187 | 9312190 | 292 | 1704 | 37 | 3 | 3 |
| 1188 | 9312191 | 239 | 1340 | 27 | 3 | 3 |
| 1189 | 9312192 | 245 | 1259 | 29 | 7 | 3 |
| 1190 | 9312193 | 315 | 1667 | 33 | 2 | 4 |
| 1191 | 9312194 | 238 | 1575 | 34 | 2 | 3 |
| 1192 | 9312195 | 207 | 1146 | 21 | 2 | 1 |
| 1193 | 9312196 | 258 | 1229 | 17 | 3 | 3 |
| 1194 | 9312197 | 333 | 1661 | 45 | 4 | 4 |
| 1195 | 9312198 | 180 | 716 | 17 | 1 | 3 |
| 1196 | 9312199 | 283 | 1187 | 23 | 5 | 3 |
| 1197 | 9312200 | 229 | 1184 | 24 | 0 | 4 |
| 1198 | 9312201 | 175 | 800 | 18 | 0 | 1 |
| 1199 | 9312202 | 286 | 1422 | 39 | 2 | 5 |
| 1200 | 9312203 | 270 | 1610 | 42 | 8 | 4 |
| 1201 | 9312204 | 242 | 1416 | 34 | 4 | 4 |
| 1202 | 9312205 | 236 | 1246 | 33 | 4 | 3 |
| 1203 | 9312206 | 257 | 1647 | 30 | 10 | 2 |
| 1204 | 9312207 | 217 | 1307 | 18 | 5 | 1 |
| 1205 | 9312208 | 212 | 1069 | 26 | 2 | 2 |
| 1206 | 9312209 | 308 | 1879 | 26 | 2 | 3 |
| 1207 | 9312210 | 249 | 1299 | 24 | 4 | 1 |
| 1208 | 9312211 | 246 | 1557 | 20 | 7 | 3 |
| 1209 | 9312212 | 179 | 867 | 21 | 3 | 3 |
| 1210 | 9312213 | 260 | 1553 | 23 | 5 | 3 |
| 1211 | 9312214 | 204 | 989 | 15 | 6 | 0 |
| 1212 | 9312215 | 285 | 1600 | 20 | 6 | 3 |
| 1213 | 9312216 | 216 | 1053 | 26 | 7 | 3 |
| 1214 | 9312217 | 287 | 2004 | 29 | 1 | 4 |
| 1215 | 9312218 | 323 | 2104 | 22 | 6 | 1 |
| 1216 | 9312219 | 208 | 1151 | 27 | 3 | 2 |
| 1217 | 9312220 | 195 | 1025 | 25 | 2 | 1 |
| 1218 | 9312221 | 228 | 1440 | 20 | 2 | 2 |
| 1219 | 9312222 | 218 | 1150 | 25 | 3 | 4 |
| 1220 | 9312223 | 212 | 1034 | 34 | 3 | 2 |
| 1221 | 9312224 | 226 | 946 | 18 | 2 | 3 |
| 1222 | 9312225 | 280 | 1324 | 18 | 7 | 4 |
| 1223 | 9312226 | 210 | 958 | 16 | 4 | 2 |
| 1224 | 9312227 | 216 | 1419 | 25 | 4 | 3 |
| 1225 | 9312228 | 196 | 944 | 21 | 3 | 4 |
| 1226 | 9312229 | 183 | 653 | 17 | 5 | 5 |
| 1227 | 9312230 | 298 | 1607 | 31 | 1 | 4 |
| 1228 | 9312231 | 172 | 833 | 12 | 3 | 2 |
| 1229 | 9312232 | 188 | 872 | 10 | 0 | 2 |
| 1230 | 9312233 | 293 | 1358 | 26 | 4 | 2 |
| 1231 | 9312234 | 228 | 1090 | 30 | 1 | 5 |
| 1232 | 9312235 | 254 | 1661 | 28 | 4 | 5 |
| 1233 | 9312236 | 180 | 800 | 12 | 1 | 1 |
| 1234 | 9312237 | 223 | 1047 | 13 | 3 | 1 |
| 1235 | 9312238 | 230 | 1138 | 24 | 3 | 2 |
| 1236 | 9312239 | 255 | 1207 | 10 | 5 | 3 |
| 1237 | 9312240 | 287 | 1406 | 26 | 2 | 2 |
| 1238 | 9312241 | 184 | 717 | 13 | 3 | 3 |
| 1239 | 9312242 | 286 | 1563 | 31 | 8 | 5 |
| 1240 | 9312243 | 191 | 1211 | 20 | 1 | 2 |
| 1241 | 9312244 | 162 | 632 | 18 | 1 | 2 |
| 1242 | 9312245 | 204 | 1039 | 20 | 2 | 3 |
| 1243 | 9312246 | 178 | 626 | 13 | 3 | 3 |
| 1244 | 9312247 | 196 | 951 | 18 | 0 | 4 |
| 1245 | 9312248 | 214 | 853 | 25 | 3 | 3 |
| 1246 | 9312249 | 179 | 904 | 9 | 6 | 1 |
| 1247 | 9312250 | 226 | 1260 | 22 | 4 | 2 |
| 1248 | 9312251 | 283 | 1492 | 35 | 4 | 3 |
| 1249 | 9312252 | 313 | 1607 | 18 | 5 | 5 |
| 1250 | 9312253 | 316 | 1808 | 23 | 1 | 1 |
| 1251 | 9312254 | 253 | 1298 | 25 | 4 | 3 |
| 1252 | 9312255 | 349 | 1849 | 27 | 5 | 2 |
| 1253 | 9312256 | 248 | 1115 | 26 | 2 | 3 |
| 1254 | 9312257 | 277 | 1596 | 21 | 2 | 3 |
| 1255 | 9312258 | 329 | 2144 | 47 | 4 | 4 |
| 1256 | 9312259 | 183 | 998 | 16 | 2 | 1 |
| 1257 | 9312260 | 282 | 1280 | 24 | 2 | 3 |
| 1258 | 9312261 | 230 | 1203 | 23 | 9 | 3 |
| 1259 | 9312262 | 241 | 1181 | 26 | 1 | 5 |
| 1260 | 9312263 | 234 | 1073 | 21 | 1 | 4 |
| 1261 | 9312264 | 266 | 1396 | 28 | 0 | 5 |
| 1262 | 9312265 | 178 | 802 | 18 | 3 | 2 |
| 1263 | 9312266 | 291 | 1513 | 20 | 1 | 4 |
| 1264 | 9312267 | 366 | 2220 | 42 | 2 | 5 |
| 1265 | 9312268 | 308 | 1988 | 14 | 4 | 4 |
| 1266 | 9312269 | 211 | 936 | 13 | 2 | 3 |
| 1267 | 9312270 | 137 | 670 | 13 | 2 | 2 |
| 1268 | 9312271 | 216 | 906 | 23 | 1 | 5 |
| 1269 | 9312272 | 276 | 1642 | 18 | 4 | 4 |
| 1270 | 9312273 | 218 | 1079 | 20 | 4 | 2 |
| 1271 | 9312274 | 332 | 1909 | 38 | 1 | 3 |
| 1272 | 9312275 | 233 | 1423 | 30 | 3 | 3 |
| 1273 | 9312276 | 224 | 1303 | 23 | 2 | 3 |
| 1274 | 9312277 | 239 | 1357 | 30 | 3 | 3 |
| 1275 | 9312278 | 217 | 1368 | 25 | 2 | 3 |
| 1276 | 9312279 | 300 | 1523 | 31 | 0 | 2 |
| 1277 | 9312280 | 276 | 1242 | 33 | 1 | 4 |
| 1278 | 9312281 | 286 | 1699 | 31 | 2 | 3 |
| 1279 | 9312282 | 185 | 936 | 20 | 2 | 4 |
| 1280 | 9312283 | 194 | 859 | 20 | 4 | 2 |
| 1281 | 9312284 | 252 | 1045 | 25 | 3 | 2 |
| 1282 | 9312285 | 286 | 1672 | 33 | 4 | 3 |
| 1283 | 9312286 | 158 | 841 | 12 | 2 | 2 |
| 1284 | 9312287 | 314 | 2480 | 28 | 3 | 2 |
| 1285 | 9312288 | 190 | 873 | 18 | 0 | 3 |
| 1286 | 9312289 | 212 | 1233 | 15 | 7 | 2 |
| 1287 | 9312290 | 227 | 1028 | 22 | 1 | 3 |
| 1288 | 9312291 | 267 | 1530 | 23 | 1 | 2 |
| 1289 | 9312292 | 252 | 1441 | 31 | 1 | 3 |
| 1290 | 9312293 | 247 | 1311 | 19 | 1 | 3 |
| 1291 | 9312294 | 237 | 1522 | 34 | 5 | 4 |
| 1292 | 9312295 | 249 | 1226 | 29 | 9 | 4 |
| 1293 | 9312296 | 282 | 1507 | 29 | 9 | 2 |
| 1294 | 9312297 | 192 | 1120 | 22 | 4 | 1 |
| 1295 | 9312298 | 228 | 1408 | 22 | 4 | 3 |
| 1296 | 9312299 | 188 | 768 | 20 | 2 | 1 |
| 1297 | 9312300 | 187 | 974 | 22 | 3 | 2 |
| 1298 | 9312301 | 250 | 1377 | 24 | 1 | 3 |
| 1299 | 9312302 | 197 | 1288 | 16 | 2 | 2 |
| 1300 | 9312303 | 183 | 764 | 12 | 1 | 2 |
| 1301 | 9312304 | 319 | 1795 | 29 | 10 | 5 |
| 1302 | 9312305 | 214 | 1337 | 18 | 3 | 2 |
| 1303 | 9312306 | 235 | 1357 | 19 | 6 | 3 |
| 1304 | 9312307 | 244 | 1530 | 30 | 11 | 2 |
| 1305 | 9312308 | 294 | 2035 | 38 | 2 | 4 |
| 1306 | 9312309 | 342 | 2051 | 37 | 9 | 4 |
| 1307 | 9312310 | 342 | 2157 | 26 | 3 | 5 |
| 1308 | 9312311 | 258 | 1527 | 18 | 2 | 2 |
| 1309 | 9312312 | 315 | 2155 | 39 | 5 | 2 |
| 1310 | 9312313 | 292 | 1744 | 44 | 4 | 4 |
| 1311 | 9312314 | 316 | 2244 | 26 | 6 | 4 |
| 1312 | 9312315 | 181 | 888 | 16 | 3 | 2 |
| 1313 | 9312316 | 422 | 2087 | 46 | 8 | 5 |
| 1314 | 9312317 | 235 | 1165 | 21 | 1 | 5 |
| 1315 | 9312318 | 184 | 776 | 16 | 1 | 4 |
| 1316 | 9312319 | 212 | 1201 | 6 | 3 | 2 |
| 1317 | 9312320 | 244 | 1236 | 22 | 5 | 3 |
| 1318 | 9312321 | 328 | 1953 | 42 | 4 | 3 |
| 1319 | 9312322 | 321 | 2148 | 37 | 6 | 2 |
| 1320 | 9312323 | 263 | 1349 | 15 | 1 | 4 |
| 1321 | 9312324 | 210 | 872 | 17 | 3 | 3 |
| 1322 | 9312325 | 231 | 1164 | 25 | 5 | 3 |
| 1323 | 9312326 | 250 | 1307 | 21 | 7 | 2 |
| 1324 | 9312327 | 201 | 835 | 16 | 4 | 2 |
| 1325 | 9312328 | 185 | 670 | 21 | 6 | 3 |
| 1326 | 9312329 | 238 | 1252 | 8 | 4 | 4 |
| 1327 | 9312330 | 276 | 1329 | 38 | 13 | 5 |
| 1328 | 9312331 | 281 | 1679 | 37 | 5 | 1 |
| 1329 | 9312332 | 274 | 1522 | 15 | 6 | 5 |
| 1330 | 9312333 | 245 | 1332 | 41 | 6 | 5 |
| 1331 | 9312334 | 298 | 1575 | 16 | 1 | 3 |
| 1332 | 9312335 | 251 | 1333 | 30 | 8 | 2 |
| 1333 | 9312336 | 205 | 1149 | 21 | 2 | 1 |
| 1334 | 9312337 | 261 | 1482 | 32 | 1 | 2 |
| 1335 | 9312338 | 350 | 2066 | 51 | 2 | 5 |
| 1336 | 9312339 | 178 | 944 | 10 | 3 | 5 |
| 1337 | 9312340 | 195 | 1196 | 24 | 1 | 2 |
| 1338 | 9312341 | 216 | 1372 | 23 | 7 | 2 |
| 1339 | 9312342 | 309 | 1448 | 28 | 1 | 5 |
| 1340 | 9312343 | 259 | 973 | 28 | 2 | 6 |
| 1341 | 9312344 | 219 | 1012 | 25 | 8 | 3 |
| 1342 | 9312345 | 155 | 795 | 12 | 1 | 1 |
| 1343 | 9312346 | 354 | 1971 | 47 | 4 | 4 |
| 1344 | 9312347 | 220 | 992 | 13 | 3 | 6 |
| 1345 | 9312348 | 287 | 1591 | 31 | 4 | 3 |
| 1346 | 9312349 | 500 | 2496 | 46 | 2 | 5 |
| 1347 | 9312350 | 300 | 1241 | 33 | 5 | 6 |
| 1348 | 9312351 | 239 | 1588 | 24 | 3 | 2 |
| 1349 | 9312352 | 269 | 1610 | 30 | 1 | 2 |
| 1350 | 9312353 | 231 | 1106 | 15 | 3 | 4 |
| 1351 | 9312354 | 255 | 1177 | 7 | 1 | 6 |
| 1352 | 9312355 | 321 | 2157 | 45 | 2 | 4 |
| 1353 | 9312356 | 192 | 796 | 16 | 1 | 4 |
| 1354 | 9312357 | 218 | 1062 | 18 | 1 | 3 |
| 1355 | 9312358 | 220 | 1233 | 25 | 2 | 5 |
| 1356 | 9312359 | 199 | 1080 | 16 | 0 | 3 |
| 1357 | 9312360 | 267 | 1278 | 27 | 6 | 4 |
| 1358 | 9312361 | 179 | 627 | 14 | 1 | 5 |
| 1359 | 9312362 | 167 | 740 | 18 | 3 | 4 |
| 1360 | 9312363 | 273 | 1265 | 20 | 2 | 3 |
| 1361 | 9312364 | 262 | 1668 | 33 | 1 | 4 |
| 1362 | 9312365 | 307 | 1663 | 28 | 1 | 4 |
| 1363 | 9312366 | 355 | 2071 | 51 | 3 | 5 |
| 1364 | 9312367 | 189 | 1041 | 15 | 3 | 2 |
| 1365 | 9312368 | 435 | 2224 | 41 | 1 | 5 |
| 1366 | 9312369 | 254 | 1333 | 24 | 2 | 2 |
| 1367 | 9312370 | 190 | 1062 | 12 | 2 | 3 |
| 1368 | 9312371 | 168 | 937 | 18 | 1 | 3 |
| 1369 | 9312372 | 188 | 1170 | 12 | 0 | 2 |
| 1370 | 9312373 | 262 | 1488 | 24 | 5 | 1 |
| 1371 | 9312374 | 500 | 3591 | 69 | 7 | 2 |
| 1372 | 9312375 | 243 | 1225 | 26 | 2 | 3 |
| 1373 | 9312376 | 214 | 1599 | 21 | 0 | 3 |
| 1374 | 9312377 | 200 | 1005 | 17 | 1 | 3 |
| 1375 | 9312378 | 236 | 1220 | 25 | 1 | 2 |
| 1376 | 9312379 | 232 | 1199 | 22 | 2 | 3 |
| 1377 | 9312380 | 360 | 2514 | 37 | 7 | 1 |
| 1378 | 9312381 | 265 | 1823 | 27 | 3 | 2 |
| 1379 | 9312382 | 206 | 942 | 30 | 2 | 3 |
| 1380 | 9312383 | 211 | 1429 | 28 | 3 | 2 |
| 1381 | 9312384 | 264 | 1449 | 22 | 8 | 1 |
| 1382 | 9312385 | 227 | 1220 | 38 | 8 | 3 |
| 1383 | 9312386 | 253 | 1372 | 34 | 5 | 4 |
| 1384 | 9312387 | 185 | 885 | 16 | 2 | 2 |
| 1385 | 9312388 | 347 | 2092 | 40 | 5 | 4 |
| 1386 | 9312389 | 228 | 1185 | 21 | 1 | 2 |
| 1387 | 9312390 | 245 | 1773 | 16 | 3 | 0 |
| 1388 | 9312391 | 203 | 873 | 11 | 7 | 1 |
| 1389 | 9312392 | 212 | 1345 | 19 | 3 | 2 |
| 1390 | 9312393 | 256 | 1209 | 33 | 6 | 4 |
| 1391 | 9312394 | 277 | 1969 | 20 | 3 | 4 |
| 1392 | 9312395 | 208 | 1099 | 18 | 3 | 4 |
| 1393 | 9312396 | 222 | 1132 | 11 | 4 | 3 |
| 1394 | 9312397 | 193 | 1054 | 16 | 4 | 2 |
| 1395 | 9312398 | 278 | 1641 | 31 | 6 | 4 |
| 1396 | 9312399 | 244 | 1122 | 24 | 2 | 4 |
| 1397 | 9312400 | 248 | 1410 | 39 | 5 | 4 |
| 1398 | 9312401 | 216 | 927 | 18 | 4 | 4 |
| 1399 | 9312402 | 232 | 1361 | 19 | 9 | 3 |
| 1400 | 9312403 | 218 | 1074 | 30 | 2 | 5 |
| 1401 | 9312404 | 289 | 1416 | 25 | 4 | 3 |
| 1402 | 9312405 | 225 | 1061 | 11 | 1 | 3 |
| 1403 | 9312406 | 232 | 1069 | 20 | 1 | 3 |
| 1404 | 9312407 | 334 | 1932 | 34 | 3 | 5 |
| 1405 | 9312408 | 254 | 1142 | 17 | 4 | 4 |
| 1406 | 9312409 | 220 | 1160 | 18 | 2 | 2 |
| 1407 | 9312410 | 238 | 1276 | 25 | 5 | 3 |
| 1408 | 9312411 | 261 | 1374 | 32 | 3 | 4 |
| 1409 | 9312412 | 236 | 1227 | 21 | 4 | 4 |
| 1410 | 9312413 | 264 | 1369 | 31 | 7 | 2 |
| 1411 | 9312414 | 235 | 1158 | 26 | 3 | 4 |
| 1412 | 9312415 | 272 | 1459 | 26 | 1 | 3 |
| 1413 | 9312416 | 258 | 1391 | 25 | 8 | 1 |
| 1414 | 9312417 | 321 | 1556 | 39 | 6 | 5 |
| 1415 | 9312418 | 217 | 1172 | 18 | 1 | 2 |
| 1416 | 9312419 | 218 | 880 | 14 | 3 | 3 |
| 1417 | 9312420 | 241 | 1371 | 16 | 5 | 4 |
| 1418 | 9312421 | 323 | 1910 | 32 | 10 | 5 |
| 1419 | 9312422 | 200 | 1182 | 22 | 4 | 2 |
| 1420 | 9312423 | 194 | 1089 | 23 | 4 | 2 |
| 1421 | 9312424 | 382 | 1661 | 35 | 0 | 5 |
| 1422 | 9312425 | 196 | 975 | 22 | 2 | 4 |
| 1423 | 9312426 | 309 | 1612 | 34 | 1 | 5 |
| 1424 | 9312427 | 196 | 929 | 18 | 8 | 3 |
| 1425 | 9312428 | 243 | 1395 | 11 | 5 | 3 |
| 1426 | 9312429 | 289 | 1221 | 23 | 8 | 2 |
| 1427 | 9312430 | 240 | 1317 | 25 | 5 | 5 |
| 1428 | 9312431 | 230 | 1409 | 7 | 2 | 3 |
| 1429 | 9312432 | 301 | 1200 | 15 | 4 | 1 |
| 1430 | 9312433 | 262 | 1431 | 30 | 3 | 4 |
| 1431 | 9312434 | 208 | 859 | 16 | 9 | 4 |
| 1432 | 9312435 | 358 | 1707 | 45 | 2 | 4 |
| 1433 | 9312436 | 442 | 2862 | 23 | 3 | 6 |
| 1434 | 9312437 | 296 | 1576 | 39 | 3 | 5 |
| 1435 | 9312438 | 263 | 1315 | 16 | 5 | 4 |
| 1436 | 9312439 | 215 | 1079 | 25 | 6 | 3 |
| 1437 | 9312440 | 209 | 1215 | 17 | 3 | 2 |
| 1438 | 9312441 | 320 | 2146 | 36 | 2 | 4 |
| 1439 | 9312442 | 237 | 1393 | 23 | 1 | 5 |
| 1440 | 9312443 | 213 | 1053 | 17 | 6 | 2 |
| 1441 | 9312444 | 204 | 1078 | 19 | 3 | 2 |
| 1442 | 9312445 | 385 | 2228 | 30 | 0 | 6 |
| 1443 | 9312446 | 287 | 1645 | 27 | 1 | 4 |
| 1444 | 9312447 | 255 | 1428 | 21 | 5 | 5 |
| 1445 | 9312448 | 269 | 1857 | 27 | 2 | 3 |
| 1446 | 9312449 | 273 | 1478 | 33 | 2 | 4 |
| 1447 | 9312450 | 456 | 2748 | 30 | 8 | 3 |
| 1448 | 9312451 | 292 | 1684 | 39 | 4 | 4 |
| 1449 | 9312452 | 290 | 1661 | 41 | 3 | 5 |
| 1450 | 9312453 | 216 | 1344 | 22 | 2 | 1 |
| 1451 | 9312454 | 284 | 1635 | 40 | 3 | 2 |
| 1452 | 9312455 | 304 | 1767 | 35 | 1 | 2 |
| 1453 | 9312456 | 262 | 1589 | 29 | 3 | 1 |
| 1454 | 9312457 | 280 | 1772 | 26 | 0 | 3 |
| 1455 | 9312458 | 204 | 1034 | 16 | 1 | 3 |
| 1456 | 9312459 | 327 | 1693 | 31 | 1 | 3 |
| 1457 | 9312460 | 298 | 1676 | 19 | 1 | 3 |
| 1458 | 9312461 | 219 | 1587 | 28 | 3 | 2 |
| 1459 | 9312462 | 183 | 920 | 20 | 3 | 3 |
| 1460 | 9312463 | 304 | 1363 | 42 | 1 | 5 |
| 1461 | 9312464 | 271 | 1575 | 41 | 3 | 1 |
| 1462 | 9312465 | 176 | 754 | 23 | 2 | 3 |
| 1463 | 9312466 | 204 | 1267 | 21 | 3 | 2 |
| 1464 | 9312467 | 346 | 2098 | 52 | 9 | 2 |
| 1465 | 9312468 | 244 | 1130 | 29 | 13 | 4 |
| 1466 | 9312469 | 376 | 1979 | 56 | 4 | 4 |
| 1467 | 9312470 | 291 | 1640 | 40 | 9 | 3 |
| 1468 | 9312471 | 293 | 1777 | 33 | 3 | 2 |
| 1469 | 9312472 | 259 | 1868 | 27 | 2 | 2 |
| 1470 | 9312473 | 500 | 2807 | 49 | 2 | 4 |
| 1471 | 9312474 | 352 | 2204 | 36 | 7 | 2 |
| 1472 | 9312475 | 282 | 1907 | 32 | 2 | 4 |
| 1473 | 9312476 | 357 | 2229 | 42 | 1 | 3 |
| 1474 | 9312477 | 260 | 1405 | 36 | 3 | 3 |
| 1475 | 9312478 | 272 | 1457 | 18 | 6 | 2 |
| 1476 | 9312479 | 248 | 1532 | 18 | 1 | 5 |
| 1477 | 9312480 | 294 | 2176 | 35 | 4 | 2 |
| 1478 | 9312481 | 254 | 1781 | 26 | 5 | 2 |
| 1479 | 9312482 | 206 | 1214 | 20 | 2 | 2 |
| 1480 | 9312483 | 230 | 1409 | 13 | 4 | 2 |
| 1481 | 9312484 | 215 | 1197 | 21 | 7 | 4 |
| 1482 | 9312485 | 175 | 932 | 12 | 4 | 1 |
| 1483 | 9312486 | 218 | 1215 | 32 | 1 | 4 |
| 1484 | 9312487 | 215 | 1326 | 25 | 4 | 4 |
| 1485 | 9312488 | 350 | 2747 | 46 | 1 | 1 |
| 1486 | 9312489 | 269 | 1265 | 16 | 4 | 0 |
| 1487 | 9312490 | 245 | 1332 | 22 | 2 | 1 |
| 1488 | 9312491 | 268 | 1047 | 18 | 6 | 4 |
| 1489 | 9312492 | 196 | 894 | 17 | 2 | 3 |
| 1490 | 9312493 | 317 | 1804 | 29 | 2 | 4 |
| 1491 | 9312494 | 240 | 1246 | 16 | 7 | 3 |
| 1492 | 9312495 | 249 | 1158 | 17 | 7 | 4 |
| 1493 | 9312496 | 290 | 1826 | 19 | 1 | 3 |
| 1494 | 9312497 | 500 | 2734 | 30 | 6 | 4 |
| 1495 | 9312498 | 235 | 1133 | 31 | 2 | 3 |
| 1496 | 9312499 | 259 | 1076 | 29 | 5 | 3 |
| 1497 | 9312500 | 246 | 823 | 21 | 3 | 4 |
| 1498 | 9312501 | 256 | 1468 | 29 | 1 | 2 |
| 1499 | 9312502 | 246 | 1002 | 20 | 2 | 3 |
| 1500 | 9312503 | 318 | 1795 | 37 | 6 | 4 |
| 1501 | 9312504 | 262 | 1250 | 12 | 3 | 4 |
| 1502 | 9312505 | 192 | 951 | 12 | 0 | 2 |
| 1503 | 9312506 | 305 | 1692 | 27 | 2 | 3 |
| 1504 | 9312507 | 186 | 827 | 15 | 3 | 1 |
| 1505 | 9312508 | 273 | 1731 | 25 | 2 | 3 |
| 1506 | 9312509 | 168 | 663 | 15 | 3 | 2 |
| 1507 | 9312510 | 286 | 1631 | 40 | 2 | 4 |
| 1508 | 9312511 | 229 | 1461 | 17 | 4 | 1 |
| 1509 | 9312512 | 327 | 1887 | 16 | 5 | 2 |
| 1510 | 9312513 | 237 | 1521 | 31 | 3 | 2 |
| 1511 | 9312514 | 267 | 1540 | 28 | 6 | 3 |
| 1512 | 9312515 | 261 | 1595 | 13 | 4 | 3 |
| 1513 | 9312516 | 168 | 740 | 20 | 1 | 3 |
| 1514 | 9312517 | 276 | 1538 | 29 | 1 | 3 |
| 1515 | 9312518 | 209 | 919 | 20 | 2 | 3 |
| 1516 | 9312519 | 273 | 1667 | 27 | 2 | 4 |
| 1517 | 9312520 | 160 | 697 | 13 | 4 | 4 |
| 1518 | 9312521 | 363 | 1921 | 29 | 5 | 3 |
| 1519 | 9312522 | 178 | 803 | 18 | 6 | 3 |
| 1520 | 9312523 | 192 | 829 | 20 | 1 | 4 |
| 1521 | 9312524 | 273 | 1398 | 31 | 3 | 2 |
| 1522 | 9312525 | 226 | 1141 | 17 | 5 | 3 |
| 1523 | 9312526 | 280 | 1875 | 18 | 7 | 3 |
| 1524 | 9312527 | 255 | 1165 | 26 | 2 | 4 |
| 1525 | 9312528 | 249 | 1457 | 26 | 1 | 4 |
| 1526 | 9312529 | 245 | 1339 | 19 | 5 | 4 |
| 1527 | 9312530 | 299 | 1066 | 26 | 4 | 5 |
| 1528 | 9312531 | 237 | 1391 | 20 | 2 | 1 |
| 1529 | 9312532 | 265 | 1829 | 38 | 5 | 4 |
| 1530 | 9312533 | 337 | 1852 | 25 | 2 | 3 |
| 1531 | 9312534 | 195 | 906 | 15 | 0 | 4 |
| 1532 | 9312535 | 183 | 799 | 17 | 2 | 5 |
| 1533 | 9312536 | 255 | 1462 | 29 | 7 | 4 |
| 1534 | 9312537 | 267 | 1286 | 24 | 1 | 5 |
| 1535 | 9312538 | 261 | 1137 | 27 | 3 | 4 |
| 1536 | 9312539 | 251 | 1227 | 29 | 3 | 2 |
| 1537 | 9312540 | 260 | 1277 | 33 | 1 | 3 |
| 1538 | 9312541 | 262 | 1600 | 31 | 1 | 3 |
| 1539 | 9312542 | 274 | 1032 | 21 | 0 | 5 |
| 1540 | 9312543 | 222 | 1175 | 25 | 1 | 2 |
| 1541 | 9312544 | 340 | 1854 | 33 | 3 | 2 |
| 1542 | 9312545 | 237 | 1323 | 26 | 0 | 2 |
| 1543 | 9312546 | 297 | 1679 | 22 | 1 | 4 |
| 1544 | 9312547 | 282 | 1612 | 34 | 3 | 2 |
| 1545 | 9312548 | 213 | 1078 | 22 | 4 | 3 |
| 1546 | 9312549 | 287 | 1483 | 40 | 1 | 5 |
| 1547 | 9312550 | 227 | 1063 | 17 | 1 | 3 |
| 1548 | 9312551 | 271 | 1760 | 30 | 2 | 3 |
| 1549 | 9312552 | 245 | 1246 | 19 | 0 | 1 |
| 1550 | 9312553 | 323 | 2185 | 36 | 3 | 3 |
| 1551 | 9312554 | 185 | 1149 | 23 | 3 | 3 |
| 1552 | 9312555 | 373 | 2644 | 53 | 2 | 4 |
| 1553 | 9312556 | 249 | 1273 | 35 | 6 | 2 |
| 1554 | 9312557 | 230 | 1419 | 21 | 2 | 1 |
| 1555 | 9312558 | 311 | 2440 | 49 | 2 | 2 |
| 1556 | 9312559 | 192 | 1013 | 26 | 2 | 3 |
| 1557 | 9312560 | 224 | 903 | 17 | 3 | 0 |
| 1558 | 9312561 | 219 | 1048 | 13 | 4 | 0 |
| 1559 | 9312562 | 200 | 1029 | 18 | 2 | 3 |
| 1560 | 9312563 | 217 | 751 | 16 | 5 | 4 |
| 1561 | 9312564 | 185 | 781 | 18 | 2 | 2 |
| 1562 | 9312565 | 254 | 1324 | 19 | 8 | 4 |
| 1563 | 9312566 | 331 | 2227 | 25 | 8 | 2 |
| 1564 | 9312567 | 235 | 1246 | 15 | 5 | 2 |
| 1565 | 9312568 | 257 | 1702 | 23 | 3 | 4 |
| 1566 | 9312569 | 179 | 811 | 15 | 4 | 1 |
| 1567 | 9312570 | 184 | 836 | 18 | 3 | 3 |
| 1568 | 9312571 | 176 | 888 | 13 | 2 | 1 |
| 1569 | 9312572 | 262 | 1548 | 19 | 5 | 3 |
| 1570 | 9312573 | 230 | 1343 | 19 | 3 | 4 |
| 1571 | 9312574 | 382 | 2445 | 39 | 8 | 3 |
| 1572 | 9312575 | 275 | 1337 | 33 | 6 | 3 |
| 1573 | 9312576 | 212 | 958 | 18 | 6 | 3 |
| 1574 | 9312577 | 267 | 1552 | 28 | 2 | 3 |
| 1575 | 9312578 | 263 | 1302 | 21 | 1 | 4 |
| 1576 | 9312579 | 328 | 1344 | 19 | 4 | 5 |
| 1577 | 9312580 | 286 | 1193 | 30 | 1 | 4 |
| 1578 | 9312581 | 256 | 1266 | 31 | 4 | 2 |
| 1579 | 9312582 | 312 | 1400 | 27 | 1 | 4 |
| 1580 | 9312583 | 310 | 1452 | 30 | 4 | 6 |
| 1581 | 9312584 | 279 | 1348 | 29 | 2 | 4 |
| 1582 | 9312585 | 238 | 1190 | 26 | 1 | 3 |
| 1583 | 9312586 | 291 | 1702 | 25 | 1 | 3 |
| 1584 | 9312587 | 325 | 1672 | 22 | 3 | 4 |
| 1585 | 9312588 | 203 | 785 | 17 | 4 | 4 |
| 1586 | 9312589 | 219 | 1087 | 29 | 4 | 4 |
| 1587 | 9312590 | 264 | 1382 | 25 | 6 | 2 |
| 1588 | 9312591 | 235 | 1215 | 26 | 6 | 2 |
| 1589 | 9312592 | 281 | 1599 | 27 | 2 | 4 |
| 1590 | 9312593 | 220 | 936 | 25 | 1 | 3 |
| 1591 | 9312594 | 238 | 1159 | 21 | 4 | 5 |
| 1592 | 9312595 | 319 | 1662 | 30 | 5 | 4 |
| 1593 | 9312596 | 252 | 1325 | 24 | 1 | 3 |
| 1594 | 9312597 | 235 | 1268 | 22 | 4 | 4 |
| 1595 | 9312598 | 264 | 1215 | 26 | 1 | 3 |
| 1596 | 9312599 | 291 | 1505 | 18 | 3 | 5 |
| 1597 | 9312600 | 299 | 1924 | 44 | 3 | 4 |
| 1598 | 9312601 | 246 | 1547 | 23 | 3 | 1 |
| 1599 | 9312602 | 187 | 913 | 20 | 2 | 3 |
| 1600 | 9312603 | 293 | 1473 | 23 | 2 | 4 |
| 1601 | 9312604 | 212 | 1159 | 29 | 3 | 4 |
| 1602 | 9312605 | 375 | 2567 | 42 | 6 | 2 |
| 1603 | 9312606 | 218 | 1390 | 26 | 5 | 1 |
| 1604 | 9312607 | 251 | 833 | 14 | 1 | 3 |
| 1605 | 9312608 | 249 | 1526 | 29 | 0 | 5 |
| 1606 | 9312609 | 242 | 1205 | 33 | 2 | 3 |
| 1607 | 9312610 | 240 | 1348 | 28 | 2 | 3 |
| 1608 | 9312611 | 211 | 1231 | 22 | 1 | 3 |
| 1609 | 9312612 | 304 | 1606 | 35 | 1 | 5 |
| 1610 | 9312613 | 225 | 893 | 23 | 2 | 3 |
| 1611 | 9312614 | 277 | 1317 | 19 | 4 | 4 |
| 1612 | 9312615 | 234 | 1600 | 25 | 3 | 2 |
| 1613 | 9312616 | 177 | 875 | 21 | 1 | 4 |
| 1614 | 9312617 | 284 | 1400 | 18 | 3 | 5 |
| 1615 | 9312618 | 284 | 1752 | 30 | 2 | 3 |
| 1616 | 9312619 | 264 | 1420 | 23 | 2 | 5 |
| 1617 | 9312620 | 235 | 1605 | 21 | 1 | 1 |
| 1618 | 9312621 | 229 | 1210 | 23 | 3 | 2 |
| 1619 | 9312622 | 338 | 1751 | 36 | 3 | 5 |
| 1620 | 9312623 | 297 | 1590 | 42 | 3 | 5 |
| 1621 | 9312624 | 237 | 1033 | 22 | 5 | 3 |
| 1622 | 9312625 | 198 | 851 | 24 | 0 | 1 |
| 1623 | 9312626 | 260 | 1272 | 27 | 4 | 3 |
| 1624 | 9312627 | 253 | 1235 | 30 | 2 | 4 |
| 1625 | 9312628 | 224 | 989 | 28 | 1 | 4 |
| 1626 | 9312629 | 208 | 1441 | 27 | 5 | 2 |
| 1627 | 9312630 | 239 | 1079 | 28 | 1 | 3 |
| 1628 | 9312631 | 376 | 1924 | 49 | 0 | 5 |
| 1629 | 9312632 | 332 | 2086 | 38 | 1 | 4 |
| 1630 | 9312633 | 357 | 2274 | 58 | 3 | 4 |
| 1631 | 9312634 | 176 | 831 | 15 | 2 | 3 |
| 1632 | 9312635 | 237 | 1284 | 29 | 7 | 2 |
| 1633 | 9312636 | 266 | 1771 | 45 | 1 | 4 |
| 1634 | 9312637 | 224 | 1011 | 23 | 2 | 4 |
| 1635 | 9312638 | 286 | 2058 | 36 | 2 | 1 |
| 1636 | 9312639 | 213 | 1019 | 24 | 1 | 3 |
| 1637 | 9312640 | 196 | 653 | 13 | 0 | 3 |
| 1638 | 9312641 | 252 | 1523 | 20 | 0 | 3 |
| 1639 | 9312642 | 210 | 1065 | 27 | 5 | 5 |
| 1640 | 9312643 | 253 | 1147 | 31 | 2 | 3 |
| 1641 | 9312644 | 267 | 1282 | 21 | 5 | 2 |
| 1642 | 9312645 | 167 | 888 | 17 | 0 | 1 |
| 1643 | 9312646 | 251 | 1312 | 27 | 3 | 3 |
| 1644 | 9312647 | 301 | 2211 | 38 | 3 | 2 |
| 1645 | 9312648 | 198 | 1147 | 23 | 3 | 3 |
| 1646 | 9312649 | 179 | 958 | 21 | 3 | 2 |
| 1647 | 9312650 | 199 | 899 | 19 | 2 | 2 |
| 1648 | 9312651 | 500 | 2567 | 49 | 5 | 3 |
| 1649 | 9312652 | 212 | 1470 | 30 | 5 | 4 |
| 1650 | 9312653 | 193 | 1184 | 18 | 6 | 2 |
| 1651 | 9312654 | 234 | 1176 | 18 | 5 | 2 |
| 1652 | 9312655 | 234 | 1277 | 29 | 4 | 3 |
| 1653 | 9312656 | 262 | 1625 | 26 | 6 | 3 |
| 1654 | 9312657 | 241 | 1382 | 30 | 2 | 2 |
| 1655 | 9312658 | 224 | 1250 | 22 | 1 | 3 |
| 1656 | 9312659 | 212 | 1292 | 19 | 7 | 2 |
| 1657 | 9312660 | 206 | 1250 | 12 | 6 | 4 |
| 1658 | 9312661 | 207 | 963 | 22 | 3 | 2 |
| 1659 | 9312662 | 212 | 1204 | 10 | 0 | 1 |
| 1660 | 9312663 | 210 | 1218 | 22 | 3 | 2 |
| 1661 | 9312664 | 290 | 1891 | 33 | 8 | 2 |
| 1662 | 9312665 | 373 | 2252 | 57 | 8 | 5 |
| 1663 | 9312666 | 191 | 850 | 23 | 5 | 4 |
| 1664 | 9312667 | 173 | 791 | 10 | 2 | 3 |
| 1665 | 9312668 | 241 | 1266 | 17 | 0 | 3 |
| 1666 | 9312669 | 227 | 1280 | 7 | 3 | 5 |
| 1667 | 9312670 | 264 | 1569 | 34 | 4 | 2 |
| 1668 | 9312671 | 264 | 1071 | 16 | 3 | 4 |
| 1669 | 9312672 | 303 | 1483 | 29 | 3 | 3 |
| 1670 | 9312673 | 240 | 1405 | 22 | 2 | 4 |
| 1671 | 9312674 | 241 | 1118 | 18 | 3 | 3 |
| 1672 | 9312675 | 284 | 1502 | 18 | 4 | 3 |
| 1673 | 9312676 | 270 | 1381 | 24 | 6 | 2 |
| 1674 | 9312677 | 227 | 917 | 23 | 3 | 4 |
| 1675 | 9312678 | 277 | 1701 | 27 | 9 | 3 |
| 1676 | 9312679 | 262 | 1499 | 28 | 5 | 4 |
| 1677 | 9312680 | 199 | 938 | 15 | 3 | 3 |
| 1678 | 9312681 | 219 | 1105 | 16 | 6 | 2 |
| 1679 | 9312682 | 270 | 1546 | 26 | 4 | 4 |
| 1680 | 9312683 | 186 | 1034 | 18 | 4 | 2 |
| 1681 | 9312684 | 212 | 1001 | 16 | 2 | 3 |
| 1682 | 9312685 | 279 | 1780 | 28 | 8 | 3 |
| 1683 | 9312686 | 359 | 1900 | 39 | 3 | 5 |
| 1684 | 9312687 | 262 | 1581 | 27 | 2 | 5 |
| 1685 | 9312688 | 177 | 594 | 11 | 2 | 1 |
| 1686 | 9312689 | 302 | 1695 | 39 | 2 | 1 |
| 1687 | 9312690 | 311 | 1481 | 36 | 5 | 6 |
| 1688 | 9312691 | 323 | 1591 | 40 | 6 | 5 |
| 1689 | 9312692 | 264 | 1752 | 36 | 9 | 3 |
| 1690 | 9312693 | 275 | 1759 | 29 | 5 | 4 |
| 1691 | 9312694 | 215 | 1177 | 23 | 4 | 2 |
| 1692 | 9312695 | 267 | 1556 | 18 | 3 | 2 |
| 1693 | 9312696 | 195 | 758 | 12 | 2 | 1 |
| 1694 | 9312697 | 208 | 990 | 17 | 3 | 2 |
| 1695 | 9312698 | 238 | 1035 | 26 | 2 | 4 |
| 1696 | 9312699 | 178 | 1038 | 14 | 0 | 2 |
| 1697 | 9312700 | 203 | 1144 | 20 | 0 | 5 |
| 1698 | 9312701 | 195 | 1262 | 31 | 2 | 2 |
| 1699 | 9312702 | 197 | 968 | 19 | 0 | 1 |
| 1700 | 9312703 | 250 | 1617 | 21 | 3 | 4 |
| 1701 | 9312704 | 187 | 801 | 19 | 4 | 3 |
| 1702 | 9312705 | 218 | 952 | 17 | 6 | 1 |
| 1703 | 9312706 | 188 | 984 | 11 | 3 | 3 |
| 1704 | 9312707 | 261 | 1138 | 34 | 3 | 5 |
| 1705 | 9312708 | 301 | 1750 | 31 | 3 | 1 |
| 1706 | 9312709 | 179 | 800 | 13 | 0 | 4 |
| 1707 | 9312710 | 213 | 1035 | 25 | 7 | 4 |
| 1708 | 9312711 | 251 | 1290 | 26 | 3 | 6 |
| 1709 | 9312712 | 361 | 2387 | 53 | 3 | 5 |
| 1710 | 9312713 | 291 | 1470 | 33 | 1 | 4 |
| 1711 | 9312714 | 363 | 1788 | 24 | 5 | 6 |
| 1712 | 9312715 | 288 | 1309 | 28 | 2 | 5 |
| 1713 | 9312716 | 334 | 1656 | 38 | 1 | 4 |
| 1714 | 9312717 | 230 | 1318 | 19 | 1 | 3 |
| 1715 | 9312718 | 202 | 1164 | 31 | 2 | 2 |
| 1716 | 9312719 | 298 | 1354 | 33 | 1 | 6 |
| 1717 | 9312720 | 255 | 1174 | 28 | 4 | 3 |
| 1718 | 9312721 | 285 | 1279 | 19 | 2 | 5 |
| 1719 | 9312722 | 343 | 1730 | 37 | 3 | 4 |
| 1720 | 9312723 | 306 | 1566 | 42 | 1 | 5 |
| 1721 | 9312724 | 298 | 1353 | 33 | 4 | 6 |
| 1722 | 9312725 | 240 | 998 | 24 | 2 | 4 |
| 1723 | 9312726 | 266 | 1270 | 30 | 0 | 5 |
| 1724 | 9312727 | 260 | 1315 | 30 | 4 | 3 |
| 1725 | 9312728 | 306 | 1852 | 43 | 0 | 5 |
| 1726 | 9312729 | 282 | 1752 | 38 | 0 | 2 |
| 1727 | 9312730 | 285 | 1381 | 33 | 3 | 3 |
| 1728 | 9312731 | 256 | 1580 | 32 | 3 | 2 |
| 1729 | 9312732 | 281 | 1516 | 34 | 4 | 2 |
| 1730 | 9312733 | 220 | 1024 | 35 | 1 | 4 |
| 1731 | 9312734 | 282 | 1640 | 30 | 4 | 4 |
| 1732 | 9312735 | 302 | 1601 | 33 | 1 | 3 |
| 1733 | 9312736 | 195 | 952 | 18 | 6 | 2 |
| 1734 | 9312737 | 223 | 1475 | 36 | 1 | 1 |
| 1735 | 9312738 | 259 | 1393 | 43 | 3 | 4 |
| 1736 | 9312739 | 342 | 1970 | 18 | 3 | 3 |
| 1737 | 9312740 | 307 | 1821 | 43 | 1 | 4 |
| 1738 | 9312741 | 214 | 1377 | 26 | 2 | 2 |
| 1739 | 9312742 | 290 | 1519 | 32 | 5 | 4 |
| 1740 | 9312743 | 190 | 1198 | 21 | 4 | 2 |
| 1741 | 9312744 | 201 | 974 | 18 | 1 | 2 |
| 1742 | 9312745 | 170 | 886 | 18 | 3 | 4 |
| 1743 | 9312746 | 235 | 1258 | 27 | 3 | 3 |
| 1744 | 9312747 | 275 | 1346 | 20 | 3 | 2 |
| 1745 | 9312748 | 263 | 1501 | 29 | 6 | 3 |
| 1746 | 9312749 | 458 | 2211 | 44 | 3 | 4 |
| 1747 | 9312750 | 281 | 1662 | 35 | 5 | 4 |
| 1748 | 9312751 | 209 | 1395 | 19 | 4 | 1 |
| 1749 | 9312752 | 210 | 820 | 18 | 4 | 4 |
| 1750 | 9312753 | 220 | 1158 | 19 | 4 | 2 |
| 1751 | 9312754 | 287 | 1695 | 35 | 3 | 2 |
| 1752 | 9312755 | 289 | 1227 | 20 | 1 | 5 |
| 1753 | 9312756 | 231 | 1087 | 14 | 3 | 4 |
| 1754 | 9312757 | 322 | 1320 | 53 | 4 | 4 |
| 1755 | 9312758 | 233 | 1231 | 38 | 1 | 4 |
| 1756 | 9312759 | 195 | 667 | 5 | 1 | 2 |
| 1757 | 9312760 | 253 | 1324 | 13 | 3 | 2 |
| 1758 | 9312761 | 294 | 1247 | 25 | 0 | 3 |
| 1759 | 9312762 | 223 | 1079 | 19 | 2 | 4 |
| 1760 | 9312763 | 335 | 1546 | 31 | 3 | 4 |
| 1761 | 9312764 | 232 | 1013 | 12 | 8 | 3 |
| 1762 | 9312765 | 223 | 992 | 21 | 1 | 6 |
| 1763 | 9312766 | 249 | 1276 | 30 | 4 | 2 |
| 1764 | 9312767 | 183 | 712 | 21 | 7 | 3 |
| 1765 | 9312768 | 216 | 1021 | 21 | 7 | 3 |
| 1766 | 9312769 | 242 | 1353 | 23 | 4 | 3 |
| 1767 | 9312770 | 269 | 1517 | 26 | 3 | 3 |
| 1768 | 9312771 | 217 | 1206 | 21 | 1 | 3 |
| 1769 | 9312772 | 250 | 1358 | 26 | 1 | 3 |
| 1770 | 9312773 | 208 | 971 | 27 | 1 | 5 |
| 1771 | 9312774 | 242 | 1220 | 30 | 7 | 2 |
| 1772 | 9312775 | 270 | 1280 | 27 | 3 | 4 |
| 1773 | 9312776 | 242 | 1021 | 20 | 4 | 1 |
| 1774 | 9312777 | 242 | 1153 | 25 | 4 | 2 |
| 1775 | 9312778 | 244 | 1403 | 36 | 5 | 4 |
| 1776 | 9312779 | 242 | 1393 | 28 | 4 | 2 |
| 1777 | 9312780 | 269 | 1288 | 26 | 3 | 3 |
| 1778 | 9312781 | 225 | 1256 | 23 | 3 | 0 |
| 1779 | 9312782 | 222 | 1353 | 17 | 6 | 1 |
| 1780 | 9312783 | 363 | 2267 | 41 | 6 | 3 |
| 1781 | 9312784 | 239 | 1251 | 23 | 2 | 3 |
| 1782 | 9312785 | 241 | 1358 | 24 | 3 | 4 |
| 1783 | 9312786 | 254 | 1266 | 37 | 0 | 4 |
| 1784 | 9312787 | 291 | 1662 | 22 | 3 | 3 |
| 1785 | 9312788 | 192 | 832 | 22 | 2 | 4 |
| 1786 | 9312789 | 205 | 927 | 13 | 2 | 4 |
| 1787 | 9312790 | 251 | 1098 | 33 | 0 | 3 |
| 1788 | 9312791 | 238 | 1264 | 16 | 1 | 4 |
| 1789 | 9312792 | 240 | 1488 | 30 | 2 | 3 |
| 1790 | 9312793 | 218 | 1112 | 22 | 2 | 3 |
| 1791 | 9312794 | 234 | 1340 | 21 | 1 | 2 |
| 1792 | 9312795 | 304 | 1753 | 35 | 7 | 4 |
| 1793 | 9312796 | 200 | 844 | 18 | 6 | 4 |
| 1794 | 9312797 | 223 | 1026 | 23 | 3 | 3 |
| 1795 | 9312798 | 221 | 1523 | 20 | 4 | 3 |
| 1796 | 9312799 | 331 | 2054 | 33 | 6 | 5 |
| 1797 | 9312800 | 232 | 1174 | 22 | 2 | 3 |
| 1798 | 9312801 | 377 | 2909 | 30 | 2 | 4 |
| 1799 | 9312802 | 221 | 1144 | 22 | 4 | 2 |
| 1800 | 9312803 | 399 | 2118 | 39 | 0 | 4 |
| 1801 | 9312804 | 481 | 2882 | 55 | 2 | 4 |
| 1802 | 9312805 | 215 | 1086 | 23 | 4 | 4 |
| 1803 | 9312806 | 197 | 794 | 13 | 2 | 1 |
| 1804 | 9312807 | 401 | 2576 | 37 | 1 | 3 |
| 1805 | 9312808 | 250 | 1032 | 25 | 5 | 5 |
| 1806 | 9312809 | 394 | 2479 | 55 | 3 | 4 |
| 1807 | 9312810 | 274 | 1252 | 22 | 0 | 5 |
| 1808 | 9312811 | 263 | 1040 | 29 | 2 | 6 |
| 1809 | 9312812 | 256 | 1243 | 31 | 2 | 5 |
| 1810 | 9312813 | 314 | 1748 | 24 | 0 | 3 |
| 1811 | 9312814 | 316 | 1710 | 36 | 5 | 5 |
| 1812 | 9312815 | 210 | 1201 | 18 | 5 | 3 |
| 1813 | 9312816 | 287 | 1465 | 32 | 2 | 4 |
| 1814 | 9312817 | 279 | 1528 | 35 | 14 | 5 |
| 1815 | 9312818 | 177 | 906 | 13 | 4 | 2 |
| 1816 | 9312819 | 269 | 1416 | 29 | 6 | 2 |
| 1817 | 9312820 | 273 | 1365 | 26 | 2 | 2 |
| 1818 | 9312821 | 272 | 1594 | 27 | 3 | 3 |
| 1819 | 9312822 | 200 | 855 | 16 | 3 | 3 |
| 1820 | 9312823 | 228 | 1028 | 32 | 3 | 5 |
| 1821 | 9312824 | 300 | 1887 | 34 | 5 | 2 |
| 1822 | 9312825 | 213 | 1205 | 25 | 2 | 2 |
| 1823 | 9312826 | 241 | 1340 | 28 | 4 | 2 |
| 1824 | 9312827 | 326 | 2336 | 26 | 5 | 3 |
| 1825 | 9312828 | 247 | 1576 | 32 | 5 | 2 |
| 1826 | 9312829 | 426 | 3184 | 53 | 7 | 1 |
| 1827 | 9312830 | 212 | 1270 | 20 | 5 | 1 |
| 1828 | 9312831 | 221 | 1219 | 23 | 0 | 3 |
| 1829 | 9312832 | 500 | 2343 | 23 | 6 | 5 |
| 1830 | 9312833 | 276 | 1833 | 41 | 2 | 1 |
| 1831 | 9312834 | 274 | 1707 | 28 | 3 | 2 |
| 1832 | 9312835 | 242 | 1333 | 18 | 6 | 4 |
| 1833 | 9312836 | 275 | 1635 | 46 | 3 | 4 |
| 1834 | 9312837 | 228 | 1247 | 26 | 0 | 2 |
| 1835 | 9312838 | 222 | 1144 | 22 | 4 | 4 |
| 1836 | 9312839 | 235 | 1116 | 27 | 5 | 2 |
| 1837 | 9312840 | 206 | 1002 | 29 | 3 | 4 |
| 1838 | 9312841 | 265 | 1623 | 23 | 8 | 4 |
| 1839 | 9312842 | 243 | 1253 | 22 | 2 | 3 |
| 1840 | 9312843 | 336 | 1606 | 35 | 2 | 4 |
| 1841 | 9312844 | 275 | 1435 | 20 | 5 | 3 |
| 1842 | 9312845 | 263 | 1530 | 26 | 5 | 3 |
| 1843 | 9312846 | 269 | 1280 | 21 | 0 | 5 |
| 1844 | 9312847 | 290 | 1465 | 25 | 0 | 3 |
| 1845 | 9312848 | 192 | 980 | 20 | 3 | 3 |
| 1846 | 9312849 | 249 | 1321 | 30 | 0 | 3 |
| 1847 | 9312850 | 298 | 1598 | 32 | 0 | 2 |
| 1848 | 9312851 | 201 | 1043 | 24 | 5 | 2 |
| 1849 | 9312852 | 185 | 1020 | 13 | 1 | 2 |
| 1850 | 9312853 | 241 | 905 | 17 | 5 | 5 |
| 1851 | 9312854 | 221 | 890 | 23 | 2 | 3 |
| 1852 | 9312855 | 262 | 1484 | 33 | 2 | 4 |
| 1853 | 9312856 | 310 | 1481 | 15 | 2 | 3 |
| 1854 | 9312857 | 255 | 1438 | 19 | 5 | 1 |
| 1855 | 9312858 | 208 | 1026 | 16 | 9 | 2 |
| 1856 | 9312859 | 293 | 1540 | 30 | 5 | 2 |
| 1857 | 9312860 | 230 | 916 | 8 | 10 | 2 |
| 1858 | 9312861 | 229 | 1297 | 27 | 2 | 1 |
| 1859 | 9312862 | 221 | 1084 | 30 | 2 | 4 |
| 1860 | 9312863 | 219 | 1215 | 18 | 4 | 3 |
| 1861 | 9312864 | 227 | 1273 | 27 | 5 | 3 |
| 1862 | 9312865 | 194 | 992 | 6 | 2 | 1 |
| 1863 | 9312866 | 314 | 1146 | 27 | 0 | 5 |
| 1864 | 9312867 | 318 | 1615 | 22 | 2 | 5 |
| 1865 | 9312868 | 297 | 1620 | 30 | 2 | 4 |
| 1866 | 9312869 | 259 | 1366 | 34 | 3 | 2 |
| 1867 | 9312870 | 251 | 1183 | 21 | 2 | 2 |
| 1868 | 9312871 | 261 | 1567 | 28 | 2 | 5 |
| 1869 | 9312872 | 225 | 1105 | 22 | 4 | 3 |
| 1870 | 9312873 | 273 | 1723 | 29 | 2 | 2 |
| 1871 | 9312874 | 154 | 802 | 13 | 6 | 2 |
| 1872 | 9312875 | 206 | 1127 | 33 | 2 | 4 |
| 1873 | 9312876 | 347 | 1978 | 48 | 4 | 3 |
| 1874 | 9312877 | 255 | 1724 | 27 | 1 | 4 |
| 1875 | 9312878 | 276 | 1472 | 41 | 1 | 4 |
| 1876 | 9312879 | 244 | 1450 | 22 | 4 | 2 |
| 1877 | 9312880 | 304 | 1726 | 24 | 4 | 4 |
| 1878 | 9312881 | 231 | 1259 | 22 | 2 | 4 |
| 1879 | 9312882 | 236 | 1048 | 17 | 1 | 3 |
| 1880 | 9312883 | 209 | 949 | 16 | 4 | 2 |
| 1881 | 9312884 | 412 | 2477 | 30 | 5 | 4 |
| 1882 | 9312885 | 427 | 2595 | 44 | 4 | 4 |
| 1883 | 9312886 | 302 | 1634 | 20 | 1 | 2 |
| 1884 | 9312887 | 298 | 1910 | 21 | 2 | 5 |
| 1885 | 9312888 | 220 | 1178 | 13 | 0 | 2 |
| 1886 | 9312889 | 295 | 1339 | 41 | 0 | 5 |
| 1887 | 9312890 | 319 | 1667 | 32 | 7 | 3 |
| 1888 | 9312891 | 305 | 1347 | 29 | 3 | 5 |
| 1889 | 9312892 | 234 | 1179 | 28 | 1 | 4 |
| 1890 | 9312893 | 321 | 2024 | 54 | 1 | 5 |
| 1891 | 9312894 | 257 | 1343 | 46 | 2 | 4 |
| 1892 | 9312895 | 264 | 1365 | 19 | 5 | 3 |
| 1893 | 9312896 | 316 | 2092 | 38 | 3 | 3 |
| 1894 | 9312897 | 206 | 1232 | 22 | 3 | 2 |
| 1895 | 9312898 | 332 | 1545 | 30 | 1 | 4 |
| 1896 | 9312899 | 212 | 1013 | 17 | 3 | 2 |
| 1897 | 9312900 | 184 | 897 | 11 | 3 | 0 |
| 1898 | 9312901 | 256 | 1651 | 32 | 2 | 4 |
| 1899 | 9312902 | 171 | 658 | 18 | 2 | 2 |
| 1900 | 9312903 | 211 | 1151 | 19 | 3 | 4 |
| 1901 | 9312904 | 227 | 1329 | 39 | 7 | 2 |
| 1902 | 9312905 | 313 | 1437 | 27 | 5 | 4 |
| 1903 | 9312906 | 205 | 1231 | 16 | 2 | 2 |
| 1904 | 9312907 | 198 | 1032 | 15 | 4 | 3 |
| 1905 | 9312908 | 243 | 1334 | 30 | 3 | 2 |
| 1906 | 9312909 | 258 | 1651 | 33 | 4 | 4 |
| 1907 | 9312910 | 263 | 1516 | 38 | 3 | 2 |
| 1908 | 9312911 | 229 | 1367 | 30 | 4 | 2 |
| 1909 | 9312912 | 238 | 1505 | 23 | 2 | 3 |
| 1910 | 9312913 | 194 | 1161 | 15 | 2 | 1 |
| 1911 | 9312914 | 248 | 1471 | 32 | 4 | 4 |
| 1912 | 9312915 | 260 | 1338 | 25 | 3 | 2 |
| 1913 | 9312916 | 258 | 1784 | 28 | 4 | 2 |
| 1914 | 9312917 | 356 | 1975 | 49 | 4 | 3 |
| 1915 | 9312918 | 281 | 1705 | 36 | 5 | 2 |
| 1916 | 9312919 | 228 | 1185 | 20 | 1 | 3 |
| 1917 | 9312920 | 250 | 1612 | 23 | 7 | 1 |
| 1918 | 9312921 | 203 | 1064 | 20 | 2 | 3 |
| 1919 | 9312922 | 253 | 1665 | 21 | 4 | 2 |
| 1920 | 9312923 | 313 | 1968 | 23 | 8 | 3 |
| 1921 | 9312924 | 270 | 1856 | 30 | 2 | 4 |
| 1922 | 9312925 | 253 | 1608 | 28 | 12 | 3 |
| 1923 | 9312926 | 290 | 2177 | 33 | 6 | 3 |
| 1924 | 9312927 | 222 | 1388 | 26 | 9 | 3 |
| 1925 | 9312928 | 250 | 1604 | 24 | 5 | 4 |
| 1926 | 9312929 | 267 | 1620 | 30 | 4 | 3 |
| 1927 | 9312930 | 222 | 1240 | 19 | 6 | 2 |
| 1928 | 9312931 | 274 | 1602 | 20 | 1 | 3 |
| 1929 | 9312932 | 305 | 1672 | 32 | 4 | 3 |
| 1930 | 9312933 | 312 | 1982 | 32 | 2 | 2 |
| 1931 | 9312934 | 254 | 1422 | 20 | 2 | 4 |
| 1932 | 9312935 | 191 | 862 | 6 | 7 | 2 |
| 1933 | 9312936 | 242 | 1395 | 26 | 1 | 3 |
| 1934 | 9312937 | 217 | 875 | 19 | 1 | 3 |
| 1935 | 9312938 | 283 | 1808 | 21 | 1 | 1 |
| 1936 | 9312939 | 273 | 1049 | 33 | 3 | 3 |
| 1937 | 9312940 | 260 | 1214 | 19 | 3 | 5 |
| 1938 | 9312941 | 256 | 1154 | 24 | 7 | 4 |
| 1939 | 9312942 | 266 | 1556 | 33 | 2 | 2 |
| 1940 | 9312943 | 222 | 1127 | 28 | 4 | 3 |
| 1941 | 9312944 | 417 | 2002 | 46 | 12 | 5 |
| 1942 | 9312945 | 231 | 1320 | 20 | 2 | 0 |
| 1943 | 9312946 | 284 | 1359 | 19 | 8 | 5 |
| 1944 | 9312947 | 265 | 1299 | 12 | 3 | 4 |
| 1945 | 9312948 | 246 | 1032 | 23 | 7 | 3 |
| 1946 | 9312949 | 194 | 858 | 22 | 1 | 2 |
| 1947 | 9312950 | 424 | 2779 | 41 | 9 | 3 |
| 1948 | 9312951 | 179 | 885 | 17 | 2 | 1 |
| 1949 | 9312952 | 196 | 871 | 15 | 2 | 2 |
| 1950 | 9312953 | 225 | 1143 | 25 | 1 | 4 |
| 1951 | 9312954 | 244 | 1215 | 15 | 3 | 4 |
| 1952 | 9312955 | 313 | 1727 | 45 | 4 | 1 |
| 1953 | 9312956 | 219 | 1191 | 26 | 1 | 2 |
| 1954 | 9312957 | 203 | 806 | 15 | 3 | 2 |
| 1955 | 9312958 | 271 | 1158 | 25 | 1 | 3 |
| 1956 | 9312959 | 289 | 1800 | 38 | 2 | 4 |
| 1957 | 9312960 | 260 | 1204 | 33 | 3 | 5 |
| 1958 | 9312961 | 360 | 2030 | 38 | 4 | 1 |
| 1959 | 9312962 | 251 | 1558 | 30 | 5 | 3 |
| 1960 | 9312963 | 231 | 926 | 27 | 1 | 4 |
| 1961 | 9312964 | 260 | 1523 | 37 | 0 | 3 |
| 1962 | 9312965 | 215 | 1047 | 16 | 10 | 1 |
| 1963 | 9312966 | 247 | 944 | 19 | 3 | 4 |
| 1964 | 9312967 | 211 | 1098 | 34 | 0 | 2 |
| 1965 | 9312968 | 186 | 950 | 13 | 1 | 2 |
| 1966 | 9312969 | 191 | 921 | 21 | 4 | 3 |
| 1967 | 9312970 | 276 | 1492 | 39 | 1 | 1 |
| 1968 | 9312971 | 205 | 976 | 14 | 3 | 3 |
| 1969 | 9312972 | 299 | 1646 | 30 | 4 | 5 |
| 1970 | 9312973 | 289 | 1745 | 22 | 0 | 2 |
| 1971 | 9312974 | 465 | 3180 | 51 | 1 | 3 |
| 1972 | 9312975 | 185 | 874 | 26 | 3 | 2 |
| 1973 | 9312976 | 299 | 1938 | 33 | 1 | 4 |
| 1974 | 9312977 | 228 | 1320 | 13 | 2 | 3 |
| 1975 | 9312978 | 263 | 1290 | 27 | 2 | 3 |
| 1976 | 9312979 | 231 | 1515 | 32 | 3 | 4 |
| 1977 | 9312980 | 399 | 2093 | 51 | 1 | 3 |
| 1978 | 9312981 | 234 | 1183 | 30 | 3 | 4 |
| 1979 | 9312982 | 207 | 785 | 23 | 3 | 6 |
| 1980 | 9312983 | 274 | 1699 | 38 | 1 | 4 |
| 1981 | 9312984 | 229 | 1094 | 22 | 5 | 5 |
| 1982 | 9312985 | 251 | 1390 | 23 | 1 | 3 |
| 1983 | 9312986 | 292 | 1353 | 23 | 2 | 2 |
| 1984 | 9312987 | 216 | 914 | 17 | 3 | 2 |
| 1985 | 9312988 | 338 | 1838 | 47 | 2 | 3 |
| 1986 | 9312989 | 228 | 940 | 23 | 2 | 3 |
| 1987 | 9312990 | 259 | 1191 | 33 | 2 | 3 |
| 1988 | 9312991 | 230 | 1354 | 27 | 6 | 5 |
| 1989 | 9312992 | 233 | 1259 | 22 | 1 | 4 |
| 1990 | 9312993 | 300 | 1655 | 26 | 4 | 3 |
| 1991 | 9312994 | 219 | 1099 | 19 | 0 | 1 |
| 1992 | 9312995 | 255 | 1419 | 30 | 6 | 3 |
| 1993 | 9312996 | 187 | 953 | 23 | 3 | 4 |
| 1994 | 9312997 | 195 | 721 | 13 | 1 | 2 |
| 1995 | 9312998 | 207 | 1111 | 18 | 1 | 1 |
| 1996 | 9312999 | 237 | 1216 | 30 | 5 | 3 |
| 1997 | 9313000 | 255 | 1379 | 17 | 1 | 2 |
| 1998 | 9313001 | 206 | 1143 | 25 | 6 | 3 |
| 1999 | 9313002 | 244 | 1764 | 31 | 3 | 3 |
| 2000 | 9313003 | 259 | 1251 | 32 | 3 | 3 |
| 2001 | 9313004 | 243 | 1598 | 21 | 5 | 3 |
| 2002 | 9313005 | 274 | 1667 | 28 | 4 | 2 |
| 2003 | 9313006 | 257 | 1640 | 35 | 2 | 2 |
| 2004 | 9313007 | 277 | 1577 | 28 | 7 | 3 |
| 2005 | 9313008 | 195 | 784 | 14 | 1 | 1 |
| 2006 | 9313009 | 180 | 1003 | 17 | 4 | 1 |
| 2007 | 9313010 | 231 | 1395 | 28 | 10 | 3 |
| 2008 | 9313011 | 233 | 904 | 24 | 1 | 2 |
| 2009 | 9313012 | 209 | 1287 | 17 | 2 | 3 |
| 2010 | 9313013 | 284 | 2074 | 36 | 18 | 2 |
| 2011 | 9313014 | 175 | 825 | 12 | 2 | 1 |
| 2012 | 9313015 | 290 | 1490 | 34 | 4 | 5 |
| 2013 | 9313016 | 363 | 2171 | 31 | 5 | 3 |
| 2014 | 9313017 | 179 | 817 | 14 | 5 | 1 |
| 2015 | 9313018 | 245 | 1643 | 37 | 4 | 2 |
| 2016 | 9313019 | 210 | 969 | 17 | 6 | 2 |
| 2017 | 9313020 | 266 | 1328 | 24 | 7 | 5 |
| 2018 | 9313021 | 234 | 1386 | 21 | 3 | 4 |
| 2019 | 9313022 | 193 | 905 | 18 | 3 | 4 |
| 2020 | 9313023 | 279 | 1222 | 25 | 2 | 5 |
| 2021 | 9313024 | 286 | 1486 | 20 | 3 | 6 |
| 2022 | 9313025 | 236 | 1432 | 14 | 6 | 2 |
| 2023 | 9313026 | 215 | 1188 | 23 | 4 | 2 |
| 2024 | 9313027 | 225 | 1251 | 27 | 2 | 2 |
| 2025 | 9313028 | 280 | 1732 | 18 | 4 | 4 |
| 2026 | 9313029 | 206 | 1127 | 20 | 0 | 1 |
| 2027 | 9313030 | 277 | 1417 | 23 | 3 | 2 |
| 2028 | 9313031 | 319 | 1986 | 46 | 5 | 4 |
| 2029 | 9313032 | 274 | 1080 | 20 | 4 | 3 |
| 2030 | 9313033 | 262 | 1324 | 19 | 2 | 3 |
| 2031 | 9313034 | 405 | 1871 | 31 | 2 | 5 |
| 2032 | 9313035 | 197 | 947 | 21 | 3 | 2 |
| 2033 | 9313036 | 274 | 1340 | 42 | 3 | 3 |
| 2034 | 9313037 | 297 | 1639 | 38 | 4 | 4 |
| 2035 | 9313038 | 210 | 1090 | 18 | 6 | 4 |
| 2036 | 9313039 | 364 | 2382 | 41 | 8 | 3 |
| 2037 | 9313040 | 273 | 1060 | 25 | 3 | 5 |
| 2038 | 9313041 | 353 | 2171 | 46 | 12 | 4 |
| 2039 | 9313042 | 313 | 1726 | 32 | 4 | 3 |
| 2040 | 9313043 | 196 | 1019 | 16 | 4 | 1 |
| 2041 | 9313044 | 179 | 871 | 22 | 3 | 4 |
| 2042 | 9313045 | 281 | 1654 | 30 | 4 | 3 |
| 2043 | 9313046 | 279 | 1510 | 43 | 3 | 3 |
| 2044 | 9313047 | 239 | 1320 | 18 | 3 | 2 |
| 2045 | 9313048 | 198 | 1131 | 20 | 3 | 2 |
| 2046 | 9313049 | 268 | 1178 | 25 | 8 | 4 |
| 2047 | 9313050 | 257 | 1446 | 27 | 5 | 3 |
| 2048 | 9313051 | 392 | 2013 | 24 | 5 | 4 |
| 2049 | 9313052 | 295 | 1574 | 24 | 2 | 4 |
| 2050 | 9313053 | 231 | 1357 | 25 | 5 | 4 |
| 2051 | 9313054 | 250 | 1469 | 22 | 3 | 3 |
| 2052 | 9313055 | 340 | 2130 | 31 | 4 | 2 |
| 2053 | 9313056 | 214 | 993 | 17 | 9 | 4 |
| 2054 | 9313057 | 296 | 1800 | 25 | 3 | 4 |
| 2055 | 9313058 | 281 | 1963 | 32 | 3 | 3 |
| 2056 | 9313059 | 231 | 844 | 18 | 2 | 3 |
| 2057 | 9313060 | 384 | 2378 | 41 | 6 | 3 |
| 2058 | 9313061 | 212 | 1074 | 22 | 3 | 2 |
| 2059 | 9313062 | 187 | 704 | 20 | 1 | 4 |
| 2060 | 9313063 | 245 | 881 | 16 | 1 | 3 |
| 2061 | 9313064 | 262 | 1378 | 15 | 1 | 4 |
| 2062 | 9313065 | 194 | 873 | 13 | 2 | 5 |
| 2063 | 9313066 | 228 | 1241 | 25 | 1 | 1 |
| 2064 | 9313067 | 256 | 1144 | 39 | 3 | 1 |
| 2065 | 9313068 | 233 | 998 | 28 | 3 | 4 |
| 2066 | 9313069 | 257 | 1323 | 29 | 2 | 5 |
| 2067 | 9313070 | 298 | 1550 | 40 | 1 | 4 |
| 2068 | 9313071 | 205 | 1024 | 28 | 6 | 4 |
| 2069 | 9313072 | 266 | 1842 | 34 | 5 | 3 |
| 2070 | 9313073 | 272 | 1496 | 37 | 2 | 4 |
| 2071 | 9313074 | 269 | 1530 | 45 | 2 | 1 |
| 2072 | 9313075 | 231 | 1118 | 21 | 5 | 4 |
| 2073 | 9313076 | 244 | 1657 | 24 | 4 | 2 |
| 2074 | 9313077 | 218 | 1110 | 17 | 2 | 3 |
| 2075 | 9313078 | 165 | 881 | 14 | 5 | 1 |
| 2076 | 9313079 | 189 | 1170 | 23 | 1 | 2 |
| 2077 | 9313080 | 259 | 1628 | 23 | 3 | 3 |
| 2078 | 9313081 | 289 | 1566 | 29 | 2 | 3 |
| 2079 | 9313082 | 196 | 945 | 20 | 2 | 2 |
| 2080 | 9313083 | 215 | 1108 | 24 | 3 | 3 |
| 2081 | 9313084 | 226 | 1270 | 24 | 4 | 1 |
| 2082 | 9313085 | 212 | 1274 | 12 | 3 | 2 |
| 2083 | 9313086 | 275 | 1874 | 30 | 10 | 3 |
| 2084 | 9313087 | 175 | 888 | 12 | 5 | 2 |
| 2085 | 9313088 | 204 | 1154 | 17 | 2 | 2 |
| 2086 | 9313089 | 231 | 1270 | 33 | 6 | 3 |
| 2087 | 9313090 | 253 | 1175 | 29 | 6 | 3 |
| 2088 | 9313091 | 301 | 1624 | 23 | 3 | 5 |
| 2089 | 9313092 | 286 | 1648 | 23 | 6 | 4 |
| 2090 | 9313093 | 295 | 1845 | 38 | 3 | 4 |
| 2091 | 9313094 | 229 | 1288 | 27 | 11 | 1 |
| 2092 | 9313095 | 280 | 1717 | 38 | 3 | 3 |
| 2093 | 9313096 | 215 | 1212 | 17 | 6 | 2 |
| 2094 | 9313097 | 293 | 1712 | 28 | 1 | 4 |
| 2095 | 9313098 | 226 | 1386 | 23 | 2 | 1 |
| 2096 | 9313099 | 321 | 2368 | 44 | 3 | 4 |
| 2097 | 9313100 | 272 | 1403 | 32 | 3 | 4 |
| 2098 | 9313101 | 236 | 1233 | 36 | 2 | 3 |
| 2099 | 9313102 | 361 | 2200 | 42 | 5 | 2 |
| 2100 | 9313103 | 261 | 1557 | 22 | 6 | 3 |
| 2101 | 9313104 | 258 | 1316 | 26 | 2 | 2 |
| 2102 | 9313105 | 265 | 1605 | 26 | 1 | 2 |
| 2103 | 9313106 | 221 | 879 | 25 | 0 | 3 |
| 2104 | 9313107 | 253 | 1236 | 34 | 0 | 3 |
| 2105 | 9313108 | 231 | 1321 | 41 | 1 | 5 |
| 2106 | 9313109 | 194 | 855 | 15 | 6 | 4 |
| 2107 | 9313110 | 300 | 1683 | 27 | 5 | 3 |
| 2108 | 9313111 | 254 | 1021 | 34 | 4 | 4 |
| 2109 | 9313112 | 274 | 1492 | 30 | 4 | 5 |
| 2110 | 9313113 | 254 | 1572 | 26 | 4 | 3 |
| 2111 | 9313114 | 298 | 1571 | 19 | 3 | 4 |

## Throughput

- Turns simulated: 532354 (first play of each game)
- Aggregate: 256.5 turns/s over 2075.4s wall
- Per game thread: 32.8 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

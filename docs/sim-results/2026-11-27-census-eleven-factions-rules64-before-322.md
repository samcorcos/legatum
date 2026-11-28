# Simulation: census-eleven-factions-rules64-before-322 (2026-11-27)

2112 games, first seed 9311003, brain `scripted`, turn limit off, safety cap 1200, 8 threads, started 2026-11-27T00:47:21Z. Wall time 1827.1s.

Source checkout at run start: `d46062d3e7f46b84a7fbbfdfb814208c6a4eab0d`; dirty: no. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. Games seat the eleven cyclic rosters of eight over all eleven factions (`--all-factions`), one roster per eight games, turned one seat per game, so every faction plays every seat equally often, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--players","8","--difficulty","hard","--all-factions","--games","2112","--seed","9311003","--threads","8","--label","census-eleven-factions-rules64-before-322","--out-dir","/home/ubuntu/census-before-out"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 2112 of 2112 |
| Game length, turns (mean / median / min-max) | 255.9 / 247 / 142-500 |
| Win reason mix | VP hold 95.0%, Capitals 4.4%, Turn limit 0.5%, Draw 0.1% |
| Median end turn | 247 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 4.4% / 95.0% / 0.5% / 0.1% |
| Games reaching the World War era (any empire) | 2104 of 2112 (99.6%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 3014 / 859 / 2013 / 142 |
| Hold break rate | 28.5% of started; 29.9% of resolved (broken / (broken + completed)) |
| Hold wins | 2007 |
| First reach to win, turns (mean / median / min-max) | 13.3 / 9 / 9-203 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 313 |
| Wins by a team another team beat to its target | 169 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 2007 | 252.5 / 245 / 142-487 |
| Capitals | 92 | 295.7 / 282 / 204-484 |
| Turn limit | 11 | 500.0 / 500 / 500-500 |
| Draw | 2 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 205 | 9.7% |
| India | 33 | 1.6% |
| Byzantium | 216 | 10.2% |
| Germany | 246 | 11.7% |
| Celts | 290 | 13.7% |
| Mongols | 105 | 5.0% |
| Sparta | 238 | 11.3% |
| Athens | 270 | 12.8% |
| Aztecs | 194 | 9.2% |
| Egypt | 224 | 10.6% |
| Rome | 89 | 4.2% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 2007 | 95.0% |
| Conquest | 0 | 0.0% |
| Capitals | 92 | 4.4% |
| Turn limit | 11 | 0.5% |
| Draw | 2 | 0.1% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 1536 | 205 | 13.3% |
| India | 1536 | 33 | 2.1% |
| Byzantium | 1536 | 216 | 14.1% |
| Germany | 1536 | 246 | 16.0% |
| Celts | 1536 | 290 | 18.9% |
| Mongols | 1536 | 105 | 6.8% |
| Sparta | 1536 | 238 | 15.5% |
| Athens | 1536 | 270 | 17.6% |
| Aztecs | 1536 | 194 | 12.6% |
| Egypt | 1536 | 224 | 14.6% |
| Rome | 1536 | 89 | 5.8% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 16896 | 2110 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 1536 | 205 | 13.3% |
| India | Hard | 1536 | 33 | 2.1% |
| Byzantium | Hard | 1536 | 216 | 14.1% |
| Germany | Hard | 1536 | 246 | 16.0% |
| Celts | Hard | 1536 | 290 | 18.9% |
| Mongols | Hard | 1536 | 105 | 6.8% |
| Sparta | Hard | 1536 | 238 | 15.5% |
| Athens | Hard | 1536 | 270 | 17.6% |
| Aztecs | Hard | 1536 | 194 | 12.6% |
| Egypt | Hard | 1536 | 224 | 14.6% |
| Rome | Hard | 1536 | 89 | 5.8% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 1536 | 210 | 13.7% | 118 | 155 | 197 | 288 | 441 |
| India | 1536 | 738 | 48.0% | 43 | 93 | 134 | 215 | 368 |
| Byzantium | 1536 | 1040 | 67.7% | 66 | 85 | 112 | 166 | 393 |
| Germany | 1536 | 480 | 31.2% | 95 | 133 | 174 | 235 | 441 |
| Celts | 1536 | 1099 | 71.5% | 26 | 191 | 210 | 210 | 291 |
| Mongols | 1536 | 221 | 14.4% | 99 | 168 | 212 | 279 | 420 |
| Sparta | 1536 | 330 | 21.5% | 141 | 174 | 221 | 287 | 410 |
| Athens | 1536 | 663 | 43.2% | 106 | 181 | 255 | 255 | 255 |
| Aztecs | 1536 | 546 | 35.5% | 131 | 193 | 225 | 244 | 333 |
| Egypt | 1536 | 733 | 47.7% | 118 | 194 | 206 | 264 | 415 |
| Rome | 1536 | 74 | 4.8% | 136 | 168 | 216 | 297 | 426 |

## Goal cities (Rome's Colonia, Egypt's tomb)

Per seat of the faction: its goal city stood (median first turn), the seat scored it, held it at the end, a rival held it at the end, it changed hands. Rome: seats with a level-3 melee unit while the Colonia was open (median first turn) and the share of those seat-turns with no legal Colonia site on the map. Egypt: seats that started the work (median first start), work turns and pauses per starting seat.

| Faction | Seats | Stood | Turn | Scored | Held at end | Rival holds | Changed hands | Level 3 | Turn | No site | Started | Turn | Work turns | Pauses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Egypt | 1536 | 666 | 208 | 666 | 429 | 151 | 219 | 0 | - | - | 957 | 191 | 12.8 | 0.6 |
| Rome | 1536 | 0 | - | 0 | 0 | 0 | 0 | 1 | 272 | 0% | 0 | - | - | - |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 142 | 199 | 247 | 322 | 500 |
| Winning team VP | 3 | 10 | 10 | 13 | 18 |
| First Soldier completed (turn, per empire) | 5 | 8 | 12 | 54 | 286 |
| First Settler completed (turn, per empire) | 2 | 14 | 16 | 41 | 269 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 5 | 19 | 25 | 56 | 331 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 16896 observed empires in 2112 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 16273 | 623 |
| First Settler completed | 14676 | 2220 |
| First city founded | 16896 | 0 |
| Second city founded | 15736 | 1160 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 4526 |
| 1 | 2504 |
| 2 | 1982 |
| 3 | 1509 |
| 4 | 1178 |
| 5 | 964 |
| 6 | 751 |
| 7 | 573 |
| 8 | 414 |
| 9 | 305 |
| 10 | 1094 |
| 11 | 592 |
| 12 | 256 |
| 13 | 140 |
| 14 | 64 |
| 15 | 23 |
| 16 | 15 |
| 17 | 5 |
| 18 | 1 |

## End state

- Average game length: 255.9 turns
- Average cities per empire: 4.21
- Average techs per empire: 30.23
- Average eliminations per game: 2.85

## Strategy diversity: observed behavior (classifier v2)

2112 finished games; 16896 player seats; 2110 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4582** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 8234 | 523 | 6.4% | 48.7% | 24.8% |
| Wide | 4609 | 1420 | 30.8% | 27.3% | 67.3% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 16 | 1 | 6.2% | 0.1% | 0.0% |
| Economic | 1184 | 164 | 13.9% | 7.0% | 7.8% |
| Generalist | 2853 | 2 | 0.1% | 16.9% | 0.1% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 17.9% / 2.9% (1977) | n/a / n/a (0) | 30.9% / 2.1% (16) | 23.0% / 6.6% (914) | 82.6% / 2.2% (1796) |
| Wide | 79.2% / 2.9% (1977) | — | n/a / n/a (0) | 70.8% / 1.6% (16) | 59.9% / 4.8% (851) | 96.4% / 0.6% (1686) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 67.0% / 2.1% (16) | 27.6% / 1.6% (16) | n/a / n/a (0) | — | 35.7% / 0.0% (7) | 96.2% / 0.0% (13) |
| Economic | 70.4% / 6.6% (914) | 35.3% / 4.8% (851) | n/a / n/a (0) | 64.3% / 0.0% (7) | — | 96.2% / 1.5% (734) |
| Generalist | 15.2% / 2.2% (1796) | 2.9% / 0.6% (1686) | n/a / n/a (0) | 3.8% / 0.0% (13) | 2.3% / 1.5% (734) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 15 / 431 | 167 / 572 | 0 / 0 | 0 / 0 | 23 / 208 | 0 / 325 |
| India | 9 / 766 | 24 / 416 | 0 / 0 | 0 / 0 | 0 / 21 | 0 / 333 |
| Byzantium | 0 / 20 | 192 / 707 | 0 / 0 | 0 / 7 | 24 / 239 | 0 / 563 |
| Germany | 41 / 626 | 193 / 525 | 0 / 0 | 0 / 5 | 11 / 73 | 1 / 307 |
| Celts | 33 / 481 | 243 / 682 | 0 / 0 | 0 / 0 | 14 / 131 | 0 / 242 |
| Mongols | 61 / 1174 | 40 / 111 | 0 / 0 | 0 / 0 | 4 / 22 | 0 / 229 |
| Sparta | 151 / 1237 | 65 / 215 | 0 / 0 | 1 / 4 | 21 / 56 | 0 / 24 |
| Athens | 29 / 463 | 212 / 544 | 0 / 0 | 0 / 0 | 28 / 160 | 1 / 369 |
| Aztecs | 31 / 652 | 148 / 451 | 0 / 0 | 0 / 0 | 15 / 131 | 0 / 302 |
| Egypt | 91 / 1054 | 120 / 343 | 0 / 0 | 0 / 0 | 13 / 91 | 0 / 48 |
| Rome | 62 / 1330 | 16 / 43 | 0 / 0 | 0 / 0 | 11 / 52 | 0 / 111 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: none. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 2110 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

2112 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 33345/2112 | 15.788 | 2112/2112 (100.0%) |
| Peace offers | 222082/2112 | 105.152 | 2112/2112 (100.0%) |
| Peace agreements | 26097/2112 | 12.357 | 2112/2112 (100.0%) |
| Battles | 2799692/2112 | 1325.612 | 2112/2112 (100.0%) |
| City captures | 52057/2112 | 24.648 | 2112/2112 (100.0%) |
| Settler captures | 7176/2112 | 3.398 | 1936/2112 (91.7%) |
| Eliminations | 6021/2112 | 2.851 | 2064/2112 (97.7%) |

## Terminal VP sources

2112 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 14000/16896 | 14000/16896 | 6628/2110 |
| Faction goals | 9970/16896 | 9970/16896 | 4138/2110 |
| Era objectives | 29696/16896 | 29696/16896 | 8909/2110 |
| Capitals taken | 5363/16896 | 5363/16896 | 3260/2110 |
| Total VP | 59029/16896 | 59029/16896 | 22935/2110 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 200/1391 (14.4%) | 1 |
| Tied leaders (winner in set) | 505/721 (70.0%) | 1 |
| Any leader set | 705/2112 (33.4%) | 2 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 15811/16896 | 5 | 21 | 29 | 33 | 256 | -1.9 (2110) |
| Medieval | 15089/16896 | 34 | 59 | 64 | 70 | 168 | -2.1 (2110) |
| Renaissance | 14904/16896 | 66 | 86 | 93 | 107 | 253 | -4.7 (2110) |
| Industrial | 14329/16896 | 89 | 114 | 125 | 147 | 256 | -8.1 (2110) |
| Machine Age | 12893/16896 | 115 | 143 | 157 | 184 | 340 | -10.7 (2110) |
| World War | 10347/16896 | 135 | 168 | 186 | 216 | 394 | -12.2 (2097) |
| Modern | 7074/16896 | 156 | 192 | 215 | 253 | 405 | -12.0 (1892) |
| Future | 3937/16896 | 177 | 218 | 247 | 276 | 451 | -9.3 (1332) |

Research idle turns (techs left, no research set): 9, of which stalled (two or more in a row): 0.

## Tech pace against the calendar

Median resolved turn each technology was learned, the display year of that turn (`calendar::calendar_label`) and the year the technology really arrived (n/a when no single year is established). Empires that never learned it are left out.

| Era | Technology | Learned / seats | Median turn | Median year | Real year |
| --- | --- | --- | --- | --- | --- |
| Ancient | Irrigation | 16817/16896 | 9 | 3173 BC | 6000 BC |
| Ancient | Bronze Working | 14964/16896 | 18 | 2242 BC | 3300 BC |
| Ancient | Masonry | 16892/16896 | 12 | 2863 BC | 3000 BC |
| Ancient | Currency | 16004/16896 | 27 | 1311 BC | 600 BC |
| Ancient | Writing | 16102/16896 | 25 | 1518 BC | 3200 BC |
| Classical | Mathematics | 15237/16896 | 45 | 251 BC | 300 BC |
| Classical | Engineering | 15154/16896 | 52 | 100 AD | n/a |
| Classical | Iron Working | 15363/16896 | 39 | 551 BC | 1200 BC |
| Classical | Code of Laws | 15118/16896 | 60 | 500 AD | 1754 BC |
| Classical | Philosophy | 15093/16896 | 62 | 560 AD | 500 BC |
| Medieval | Feudalism | 15031/16896 | 78 | 1040 AD | n/a |
| Medieval | Machinery | 15059/16896 | 72 | 860 AD | n/a |
| Medieval | Chivalry | 13643/16896 | 83 | 1190 AD | n/a |
| Medieval | Education | 14939/16896 | 90 | 1400 AD | 1088 AD |
| Medieval | Crop Rotation | 14915/16896 | 92 | 1420 AD | n/a |
| Renaissance | Gunpowder | 14848/16896 | 100 | 1500 AD | n/a |
| Renaissance | Metallurgy | 14665/16896 | 112 | 1620 AD | n/a |
| Renaissance | Military Engineering | 14715/16896 | 109 | 1590 AD | n/a |
| Renaissance | Banking | 14460/16896 | 121 | 1710 AD | 1397 AD |
| Renaissance | Printing Press | 14369/16896 | 123 | 1730 AD | 1440 AD |
| Industrial | Rifling | 13400/16896 | 147 | 1831 AD | n/a |
| Industrial | Military Science | 13834/16896 | 138 | 1798 AD | n/a |
| Industrial | Ballistics | 13552/16896 | 150 | 1842 AD | n/a |
| Industrial | Industrialization | 14045/16896 | 135 | 1787 AD | 1769 AD |
| Industrial | Railways | 12908/16896 | 157 | 1868 AD | 1825 AD |
| Machine Age | Replaceable Parts | 11745/16896 | 172 | 1893 AD | 1800 AD |
| Machine Age | Combustion | 12300/16896 | 166 | 1886 AD | 1876 AD |
| Machine Age | Steel | 11638/16896 | 171 | 1892 AD | 1856 AD |
| Machine Age | Flight | 10869/16896 | 180 | 1902 AD | 1903 AD |
| Machine Age | Electricity | 10463/16896 | 186 | 1909 AD | 1879 AD |
| World War | Mass Mobilisation | 8662/16896 | 200 | 1928 AD | n/a |
| World War | Rocketry | 8252/16896 | 204 | 1934 AD | 1926 AD |
| World War | Armoured Warfare | 7767/16896 | 209 | 1941 AD | 1916 AD |
| World War | Air Power | 7415/16896 | 213 | 1947 AD | n/a |
| World War | Electronics | 9635/16896 | 193 | 1918 AD | n/a |
| Modern | Mechanised Warfare | 5294/16896 | 229 | 1983 AD | n/a |
| Modern | Composite Armour | 4852/16896 | 235 | 1998 AD | n/a |
| Modern | Guided Missiles | 4562/16896 | 240 | 2010 AD | n/a |
| Modern | Jet Engines | 4409/16896 | 244 | 2016 AD | 1939 AD |
| Modern | Green Revolution | 6113/16896 | 223 | 1969 AD | n/a |
| Future | Autonomous Systems | 2751/16896 | 256 | 2032 AD | n/a |
| Future | Robotics | 2346/16896 | 264 | 2040 AD | n/a |
| Future | Fusion Power | 2241/16896 | 257 | 2033 AD | n/a |
| Future | Stealth Technology | 3206/16896 | 256 | 2032 AD | n/a |
| Future | Recursive AI | 1977/16896 | 257 | 2033 AD | n/a |

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
| 0 | 0.00 | 64.39 |
| 1 | 80.74 | 12.34 |
| 2 | 26.31 | 4.89 |
| 3 | 7.35 | 1.21 |
| 4 | 1.50 | 0.29 |
| 5 | 0.33 | 0.09 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 9311003 | 337 | 2009 | 39 | 3 | 3 |
| 1 | 9311004 | 233 | 997 | 14 | 11 | 0 |
| 2 | 9311005 | 184 | 862 | 17 | 6 | 3 |
| 3 | 9311006 | 206 | 873 | 19 | 2 | 3 |
| 4 | 9311007 | 270 | 980 | 18 | 4 | 3 |
| 5 | 9311008 | 223 | 1019 | 18 | 5 | 3 |
| 6 | 9311009 | 293 | 1548 | 28 | 0 | 1 |
| 7 | 9311010 | 302 | 1608 | 34 | 7 | 5 |
| 8 | 9311011 | 212 | 720 | 14 | 2 | 2 |
| 9 | 9311012 | 242 | 1145 | 25 | 2 | 2 |
| 10 | 9311013 | 249 | 1321 | 16 | 2 | 4 |
| 11 | 9311014 | 253 | 1128 | 21 | 2 | 4 |
| 12 | 9311015 | 210 | 1078 | 15 | 4 | 3 |
| 13 | 9311016 | 240 | 1246 | 12 | 4 | 5 |
| 14 | 9311017 | 226 | 974 | 9 | 3 | 4 |
| 15 | 9311018 | 222 | 1090 | 23 | 2 | 2 |
| 16 | 9311019 | 208 | 1064 | 16 | 2 | 2 |
| 17 | 9311020 | 413 | 2178 | 37 | 11 | 4 |
| 18 | 9311021 | 215 | 1118 | 35 | 1 | 1 |
| 19 | 9311022 | 223 | 1113 | 20 | 13 | 4 |
| 20 | 9311023 | 314 | 1416 | 18 | 3 | 2 |
| 21 | 9311024 | 266 | 1354 | 25 | 0 | 3 |
| 22 | 9311025 | 235 | 1172 | 24 | 2 | 3 |
| 23 | 9311026 | 189 | 803 | 12 | 1 | 2 |
| 24 | 9311027 | 221 | 1042 | 17 | 4 | 4 |
| 25 | 9311028 | 195 | 1013 | 15 | 0 | 2 |
| 26 | 9311029 | 293 | 1412 | 22 | 15 | 4 |
| 27 | 9311030 | 221 | 970 | 8 | 2 | 0 |
| 28 | 9311031 | 304 | 1760 | 29 | 2 | 3 |
| 29 | 9311032 | 304 | 1385 | 23 | 3 | 5 |
| 30 | 9311033 | 334 | 1929 | 25 | 1 | 4 |
| 31 | 9311034 | 277 | 1376 | 22 | 1 | 4 |
| 32 | 9311035 | 246 | 1061 | 26 | 1 | 3 |
| 33 | 9311036 | 239 | 1425 | 26 | 0 | 2 |
| 34 | 9311037 | 247 | 1445 | 13 | 1 | 3 |
| 35 | 9311038 | 254 | 1366 | 19 | 6 | 4 |
| 36 | 9311039 | 274 | 1459 | 32 | 2 | 2 |
| 37 | 9311040 | 199 | 656 | 13 | 3 | 4 |
| 38 | 9311041 | 261 | 1343 | 26 | 5 | 4 |
| 39 | 9311042 | 335 | 1770 | 23 | 0 | 2 |
| 40 | 9311043 | 333 | 1763 | 38 | 4 | 5 |
| 41 | 9311044 | 378 | 1813 | 32 | 5 | 3 |
| 42 | 9311045 | 210 | 1012 | 22 | 0 | 4 |
| 43 | 9311046 | 236 | 1325 | 20 | 1 | 4 |
| 44 | 9311047 | 329 | 1716 | 34 | 5 | 4 |
| 45 | 9311048 | 224 | 1381 | 18 | 6 | 2 |
| 46 | 9311049 | 214 | 1015 | 12 | 5 | 1 |
| 47 | 9311050 | 311 | 1813 | 38 | 3 | 4 |
| 48 | 9311051 | 199 | 809 | 17 | 3 | 2 |
| 49 | 9311052 | 500 | 2266 | 33 | 2 | 2 |
| 50 | 9311053 | 196 | 1159 | 24 | 3 | 2 |
| 51 | 9311054 | 195 | 902 | 24 | 3 | 3 |
| 52 | 9311055 | 258 | 1371 | 43 | 1 | 4 |
| 53 | 9311056 | 272 | 1385 | 26 | 5 | 2 |
| 54 | 9311057 | 249 | 1089 | 34 | 2 | 5 |
| 55 | 9311058 | 259 | 1530 | 25 | 5 | 2 |
| 56 | 9311059 | 217 | 776 | 12 | 5 | 1 |
| 57 | 9311060 | 247 | 1579 | 23 | 4 | 3 |
| 58 | 9311061 | 199 | 917 | 17 | 6 | 2 |
| 59 | 9311062 | 239 | 1579 | 22 | 6 | 3 |
| 60 | 9311063 | 175 | 547 | 10 | 4 | 2 |
| 61 | 9311064 | 233 | 1216 | 21 | 0 | 1 |
| 62 | 9311065 | 207 | 951 | 18 | 4 | 2 |
| 63 | 9311066 | 208 | 1065 | 28 | 2 | 2 |
| 64 | 9311067 | 236 | 1355 | 20 | 0 | 3 |
| 65 | 9311068 | 235 | 1075 | 18 | 3 | 1 |
| 66 | 9311069 | 204 | 864 | 20 | 2 | 2 |
| 67 | 9311070 | 204 | 1141 | 14 | 11 | 2 |
| 68 | 9311071 | 247 | 1565 | 22 | 3 | 1 |
| 69 | 9311072 | 403 | 2670 | 39 | 5 | 3 |
| 70 | 9311073 | 276 | 1525 | 30 | 3 | 2 |
| 71 | 9311074 | 247 | 1471 | 16 | 6 | 4 |
| 72 | 9311075 | 233 | 1323 | 32 | 3 | 2 |
| 73 | 9311076 | 363 | 2185 | 43 | 14 | 3 |
| 74 | 9311077 | 206 | 1017 | 26 | 1 | 2 |
| 75 | 9311078 | 257 | 1368 | 17 | 3 | 3 |
| 76 | 9311079 | 321 | 1752 | 28 | 7 | 2 |
| 77 | 9311080 | 260 | 1198 | 28 | 3 | 2 |
| 78 | 9311081 | 257 | 1453 | 36 | 2 | 3 |
| 79 | 9311082 | 224 | 1250 | 19 | 5 | 4 |
| 80 | 9311083 | 193 | 943 | 12 | 1 | 3 |
| 81 | 9311084 | 306 | 1332 | 32 | 9 | 3 |
| 82 | 9311085 | 332 | 1796 | 30 | 3 | 3 |
| 83 | 9311086 | 250 | 1072 | 17 | 4 | 5 |
| 84 | 9311087 | 311 | 1549 | 31 | 3 | 3 |
| 85 | 9311088 | 273 | 1188 | 22 | 3 | 3 |
| 86 | 9311089 | 272 | 1580 | 13 | 11 | 5 |
| 87 | 9311090 | 237 | 1298 | 28 | 2 | 5 |
| 88 | 9311091 | 244 | 1173 | 21 | 7 | 3 |
| 89 | 9311092 | 248 | 1248 | 16 | 2 | 1 |
| 90 | 9311093 | 246 | 989 | 19 | 4 | 1 |
| 91 | 9311094 | 333 | 1863 | 39 | 2 | 4 |
| 92 | 9311095 | 316 | 1518 | 18 | 3 | 5 |
| 93 | 9311096 | 202 | 882 | 23 | 2 | 2 |
| 94 | 9311097 | 236 | 941 | 25 | 8 | 2 |
| 95 | 9311098 | 256 | 1166 | 29 | 5 | 3 |
| 96 | 9311099 | 233 | 1160 | 23 | 4 | 2 |
| 97 | 9311100 | 204 | 871 | 21 | 2 | 3 |
| 98 | 9311101 | 277 | 1241 | 14 | 5 | 6 |
| 99 | 9311102 | 209 | 1038 | 19 | 0 | 1 |
| 100 | 9311103 | 254 | 1225 | 14 | 3 | 4 |
| 101 | 9311104 | 242 | 1096 | 13 | 2 | 4 |
| 102 | 9311105 | 226 | 1012 | 26 | 3 | 2 |
| 103 | 9311106 | 280 | 1292 | 31 | 0 | 4 |
| 104 | 9311107 | 229 | 955 | 17 | 5 | 2 |
| 105 | 9311108 | 204 | 1002 | 20 | 7 | 1 |
| 106 | 9311109 | 209 | 1088 | 22 | 5 | 3 |
| 107 | 9311110 | 270 | 1195 | 24 | 3 | 4 |
| 108 | 9311111 | 284 | 1403 | 21 | 2 | 4 |
| 109 | 9311112 | 256 | 1266 | 17 | 13 | 1 |
| 110 | 9311113 | 177 | 867 | 13 | 1 | 1 |
| 111 | 9311114 | 215 | 841 | 25 | 3 | 5 |
| 112 | 9311115 | 210 | 824 | 22 | 3 | 2 |
| 113 | 9311116 | 363 | 2357 | 41 | 5 | 5 |
| 114 | 9311117 | 199 | 708 | 6 | 7 | 3 |
| 115 | 9311118 | 233 | 1320 | 19 | 4 | 2 |
| 116 | 9311119 | 334 | 2065 | 29 | 3 | 2 |
| 117 | 9311120 | 262 | 1855 | 21 | 2 | 3 |
| 118 | 9311121 | 249 | 1459 | 21 | 6 | 3 |
| 119 | 9311122 | 256 | 1098 | 20 | 0 | 2 |
| 120 | 9311123 | 251 | 1509 | 27 | 0 | 3 |
| 121 | 9311124 | 269 | 1400 | 19 | 4 | 3 |
| 122 | 9311125 | 282 | 1087 | 27 | 3 | 3 |
| 123 | 9311126 | 243 | 1595 | 23 | 1 | 3 |
| 124 | 9311127 | 355 | 2123 | 34 | 0 | 3 |
| 125 | 9311128 | 211 | 901 | 16 | 2 | 3 |
| 126 | 9311129 | 500 | 1983 | 57 | 1 | 4 |
| 127 | 9311130 | 199 | 1070 | 11 | 2 | 1 |
| 128 | 9311131 | 244 | 1353 | 19 | 2 | 4 |
| 129 | 9311132 | 332 | 2042 | 26 | 4 | 4 |
| 130 | 9311133 | 203 | 862 | 21 | 1 | 3 |
| 131 | 9311134 | 199 | 778 | 24 | 2 | 2 |
| 132 | 9311135 | 209 | 1390 | 27 | 0 | 2 |
| 133 | 9311136 | 298 | 1474 | 29 | 2 | 4 |
| 134 | 9311137 | 221 | 1134 | 21 | 1 | 2 |
| 135 | 9311138 | 273 | 1543 | 37 | 2 | 3 |
| 136 | 9311139 | 269 | 1355 | 19 | 3 | 2 |
| 137 | 9311140 | 176 | 696 | 13 | 0 | 2 |
| 138 | 9311141 | 225 | 1059 | 26 | 3 | 3 |
| 139 | 9311142 | 219 | 1360 | 21 | 2 | 2 |
| 140 | 9311143 | 291 | 1430 | 30 | 5 | 3 |
| 141 | 9311144 | 232 | 1334 | 26 | 3 | 4 |
| 142 | 9311145 | 267 | 1438 | 24 | 2 | 3 |
| 143 | 9311146 | 271 | 1614 | 22 | 3 | 0 |
| 144 | 9311147 | 231 | 1137 | 31 | 0 | 1 |
| 145 | 9311148 | 207 | 1100 | 25 | 4 | 4 |
| 146 | 9311149 | 204 | 1449 | 23 | 1 | 3 |
| 147 | 9311150 | 173 | 885 | 18 | 3 | 2 |
| 148 | 9311151 | 201 | 978 | 18 | 3 | 1 |
| 149 | 9311152 | 218 | 779 | 15 | 5 | 1 |
| 150 | 9311153 | 294 | 1708 | 43 | 3 | 2 |
| 151 | 9311154 | 222 | 1243 | 28 | 2 | 3 |
| 152 | 9311155 | 245 | 1224 | 22 | 1 | 3 |
| 153 | 9311156 | 184 | 809 | 16 | 0 | 2 |
| 154 | 9311157 | 324 | 2260 | 36 | 1 | 4 |
| 155 | 9311158 | 291 | 1750 | 48 | 6 | 3 |
| 156 | 9311159 | 274 | 1628 | 23 | 4 | 2 |
| 157 | 9311160 | 259 | 1622 | 21 | 3 | 1 |
| 158 | 9311161 | 232 | 1068 | 18 | 2 | 1 |
| 159 | 9311162 | 274 | 1490 | 24 | 1 | 4 |
| 160 | 9311163 | 191 | 975 | 15 | 4 | 1 |
| 161 | 9311164 | 231 | 1262 | 25 | 5 | 2 |
| 162 | 9311165 | 254 | 1337 | 28 | 5 | 5 |
| 163 | 9311166 | 273 | 1537 | 40 | 1 | 4 |
| 164 | 9311167 | 242 | 1273 | 16 | 4 | 4 |
| 165 | 9311168 | 224 | 1362 | 23 | 5 | 3 |
| 166 | 9311169 | 276 | 1673 | 20 | 3 | 5 |
| 167 | 9311170 | 312 | 1735 | 31 | 7 | 2 |
| 168 | 9311171 | 289 | 1325 | 38 | 4 | 5 |
| 169 | 9311172 | 252 | 1146 | 23 | 6 | 5 |
| 170 | 9311173 | 233 | 1071 | 30 | 5 | 1 |
| 171 | 9311174 | 354 | 1804 | 28 | 1 | 4 |
| 172 | 9311175 | 170 | 576 | 20 | 0 | 2 |
| 173 | 9311176 | 252 | 1560 | 23 | 0 | 2 |
| 174 | 9311177 | 234 | 1100 | 32 | 1 | 3 |
| 175 | 9311178 | 238 | 1126 | 27 | 2 | 3 |
| 176 | 9311179 | 356 | 1495 | 23 | 6 | 1 |
| 177 | 9311180 | 319 | 1423 | 43 | 7 | 4 |
| 178 | 9311181 | 372 | 2335 | 47 | 8 | 2 |
| 179 | 9311182 | 361 | 2023 | 33 | 7 | 2 |
| 180 | 9311183 | 196 | 1034 | 27 | 0 | 2 |
| 181 | 9311184 | 281 | 1590 | 33 | 6 | 2 |
| 182 | 9311185 | 261 | 993 | 35 | 4 | 5 |
| 183 | 9311186 | 249 | 1210 | 22 | 4 | 3 |
| 184 | 9311187 | 317 | 1651 | 27 | 2 | 4 |
| 185 | 9311188 | 230 | 1019 | 22 | 3 | 4 |
| 186 | 9311189 | 222 | 1020 | 22 | 3 | 2 |
| 187 | 9311190 | 219 | 962 | 12 | 2 | 3 |
| 188 | 9311191 | 266 | 1506 | 29 | 6 | 0 |
| 189 | 9311192 | 210 | 950 | 10 | 2 | 1 |
| 190 | 9311193 | 265 | 988 | 16 | 1 | 2 |
| 191 | 9311194 | 174 | 596 | 12 | 2 | 3 |
| 192 | 9311195 | 250 | 1316 | 16 | 1 | 3 |
| 193 | 9311196 | 254 | 1311 | 28 | 3 | 1 |
| 194 | 9311197 | 299 | 1651 | 38 | 9 | 4 |
| 195 | 9311198 | 290 | 1766 | 36 | 4 | 2 |
| 196 | 9311199 | 262 | 1336 | 40 | 4 | 4 |
| 197 | 9311200 | 241 | 1112 | 18 | 5 | 1 |
| 198 | 9311201 | 309 | 1746 | 36 | 8 | 4 |
| 199 | 9311202 | 197 | 1201 | 18 | 5 | 0 |
| 200 | 9311203 | 233 | 1459 | 28 | 2 | 3 |
| 201 | 9311204 | 216 | 1212 | 10 | 4 | 3 |
| 202 | 9311205 | 394 | 2502 | 54 | 6 | 2 |
| 203 | 9311206 | 275 | 1624 | 16 | 2 | 4 |
| 204 | 9311207 | 270 | 1363 | 21 | 7 | 4 |
| 205 | 9311208 | 223 | 1155 | 11 | 4 | 4 |
| 206 | 9311209 | 280 | 1577 | 37 | 3 | 5 |
| 207 | 9311210 | 212 | 911 | 9 | 6 | 3 |
| 208 | 9311211 | 290 | 1947 | 34 | 3 | 4 |
| 209 | 9311212 | 310 | 1697 | 39 | 2 | 2 |
| 210 | 9311213 | 280 | 1763 | 27 | 3 | 2 |
| 211 | 9311214 | 218 | 1267 | 24 | 0 | 4 |
| 212 | 9311215 | 335 | 1804 | 30 | 2 | 5 |
| 213 | 9311216 | 331 | 1983 | 24 | 4 | 4 |
| 214 | 9311217 | 216 | 969 | 23 | 0 | 4 |
| 215 | 9311218 | 338 | 1441 | 34 | 5 | 1 |
| 216 | 9311219 | 218 | 873 | 30 | 1 | 3 |
| 217 | 9311220 | 275 | 1315 | 38 | 4 | 4 |
| 218 | 9311221 | 234 | 869 | 26 | 4 | 5 |
| 219 | 9311222 | 308 | 1854 | 37 | 4 | 5 |
| 220 | 9311223 | 224 | 1267 | 28 | 5 | 3 |
| 221 | 9311224 | 244 | 1391 | 21 | 4 | 3 |
| 222 | 9311225 | 334 | 1778 | 38 | 1 | 4 |
| 223 | 9311226 | 282 | 1543 | 22 | 2 | 1 |
| 224 | 9311227 | 227 | 1141 | 17 | 2 | 2 |
| 225 | 9311228 | 219 | 1157 | 24 | 3 | 3 |
| 226 | 9311229 | 217 | 1008 | 15 | 3 | 2 |
| 227 | 9311230 | 269 | 1123 | 25 | 9 | 2 |
| 228 | 9311231 | 243 | 1297 | 24 | 0 | 2 |
| 229 | 9311232 | 260 | 1442 | 28 | 2 | 4 |
| 230 | 9311233 | 207 | 1029 | 25 | 4 | 2 |
| 231 | 9311234 | 199 | 868 | 13 | 5 | 2 |
| 232 | 9311235 | 208 | 930 | 17 | 4 | 1 |
| 233 | 9311236 | 348 | 2240 | 59 | 6 | 2 |
| 234 | 9311237 | 207 | 1013 | 25 | 1 | 4 |
| 235 | 9311238 | 233 | 853 | 25 | 1 | 3 |
| 236 | 9311239 | 235 | 989 | 16 | 3 | 0 |
| 237 | 9311240 | 213 | 1136 | 29 | 3 | 2 |
| 238 | 9311241 | 219 | 1096 | 26 | 3 | 3 |
| 239 | 9311242 | 257 | 1298 | 20 | 3 | 3 |
| 240 | 9311243 | 233 | 1230 | 28 | 4 | 3 |
| 241 | 9311244 | 206 | 1186 | 13 | 8 | 3 |
| 242 | 9311245 | 282 | 1485 | 22 | 1 | 4 |
| 243 | 9311246 | 258 | 1486 | 34 | 6 | 3 |
| 244 | 9311247 | 264 | 1556 | 32 | 0 | 3 |
| 245 | 9311248 | 205 | 1099 | 19 | 4 | 3 |
| 246 | 9311249 | 200 | 823 | 22 | 3 | 2 |
| 247 | 9311250 | 228 | 1226 | 29 | 1 | 4 |
| 248 | 9311251 | 298 | 1583 | 35 | 3 | 5 |
| 249 | 9311252 | 317 | 1563 | 46 | 7 | 3 |
| 250 | 9311253 | 250 | 1252 | 20 | 5 | 3 |
| 251 | 9311254 | 279 | 1615 | 21 | 0 | 4 |
| 252 | 9311255 | 245 | 1278 | 24 | 10 | 3 |
| 253 | 9311256 | 244 | 1190 | 32 | 10 | 2 |
| 254 | 9311257 | 248 | 1310 | 33 | 6 | 3 |
| 255 | 9311258 | 224 | 1163 | 20 | 3 | 3 |
| 256 | 9311259 | 252 | 1036 | 15 | 0 | 2 |
| 257 | 9311260 | 247 | 1171 | 19 | 4 | 3 |
| 258 | 9311261 | 219 | 1071 | 11 | 3 | 2 |
| 259 | 9311262 | 370 | 1927 | 29 | 8 | 3 |
| 260 | 9311263 | 370 | 1751 | 31 | 3 | 3 |
| 261 | 9311264 | 310 | 1423 | 25 | 1 | 4 |
| 262 | 9311265 | 252 | 1123 | 17 | 3 | 4 |
| 263 | 9311266 | 393 | 1896 | 40 | 2 | 5 |
| 264 | 9311267 | 330 | 1282 | 26 | 4 | 5 |
| 265 | 9311268 | 226 | 1041 | 22 | 1 | 2 |
| 266 | 9311269 | 346 | 1983 | 55 | 3 | 1 |
| 267 | 9311270 | 261 | 1267 | 17 | 3 | 2 |
| 268 | 9311271 | 297 | 1397 | 34 | 3 | 2 |
| 269 | 9311272 | 230 | 935 | 25 | 3 | 4 |
| 270 | 9311273 | 276 | 1470 | 32 | 2 | 5 |
| 271 | 9311274 | 232 | 988 | 28 | 3 | 3 |
| 272 | 9311275 | 242 | 1171 | 13 | 0 | 4 |
| 273 | 9311276 | 183 | 655 | 13 | 3 | 1 |
| 274 | 9311277 | 167 | 759 | 16 | 5 | 0 |
| 275 | 9311278 | 288 | 1432 | 34 | 5 | 2 |
| 276 | 9311279 | 267 | 997 | 24 | 1 | 4 |
| 277 | 9311280 | 219 | 1065 | 20 | 4 | 2 |
| 278 | 9311281 | 244 | 1089 | 26 | 8 | 2 |
| 279 | 9311282 | 290 | 1394 | 30 | 4 | 1 |
| 280 | 9311283 | 223 | 1553 | 31 | 2 | 4 |
| 281 | 9311284 | 179 | 759 | 20 | 7 | 1 |
| 282 | 9311285 | 321 | 1677 | 19 | 11 | 4 |
| 283 | 9311286 | 257 | 1016 | 14 | 7 | 3 |
| 284 | 9311287 | 254 | 1330 | 20 | 7 | 4 |
| 285 | 9311288 | 372 | 2343 | 34 | 5 | 4 |
| 286 | 9311289 | 284 | 1443 | 32 | 4 | 2 |
| 287 | 9311290 | 217 | 1245 | 17 | 5 | 3 |
| 288 | 9311291 | 266 | 1254 | 40 | 1 | 5 |
| 289 | 9311292 | 295 | 1417 | 21 | 0 | 5 |
| 290 | 9311293 | 357 | 2353 | 33 | 1 | 3 |
| 291 | 9311294 | 192 | 985 | 12 | 2 | 2 |
| 292 | 9311295 | 224 | 1030 | 6 | 2 | 1 |
| 293 | 9311296 | 322 | 1972 | 27 | 5 | 4 |
| 294 | 9311297 | 172 | 801 | 18 | 3 | 2 |
| 295 | 9311298 | 203 | 951 | 20 | 2 | 4 |
| 296 | 9311299 | 265 | 1435 | 19 | 3 | 1 |
| 297 | 9311300 | 269 | 1259 | 20 | 0 | 2 |
| 298 | 9311301 | 279 | 1494 | 15 | 6 | 4 |
| 299 | 9311302 | 475 | 2277 | 38 | 4 | 5 |
| 300 | 9311303 | 444 | 2494 | 43 | 1 | 3 |
| 301 | 9311304 | 228 | 1365 | 19 | 0 | 3 |
| 302 | 9311305 | 410 | 2300 | 36 | 0 | 5 |
| 303 | 9311306 | 270 | 1413 | 21 | 7 | 4 |
| 304 | 9311307 | 204 | 785 | 16 | 1 | 3 |
| 305 | 9311308 | 399 | 2026 | 32 | 0 | 3 |
| 306 | 9311309 | 204 | 879 | 15 | 4 | 2 |
| 307 | 9311310 | 338 | 2171 | 42 | 3 | 4 |
| 308 | 9311311 | 179 | 1007 | 11 | 0 | 4 |
| 309 | 9311312 | 298 | 1653 | 37 | 8 | 5 |
| 310 | 9311313 | 244 | 1265 | 17 | 2 | 3 |
| 311 | 9311314 | 312 | 1354 | 25 | 1 | 2 |
| 312 | 9311315 | 202 | 1183 | 24 | 1 | 3 |
| 313 | 9311316 | 235 | 1032 | 28 | 4 | 3 |
| 314 | 9311317 | 246 | 1165 | 19 | 2 | 2 |
| 315 | 9311318 | 266 | 1189 | 27 | 0 | 3 |
| 316 | 9311319 | 214 | 1033 | 22 | 0 | 3 |
| 317 | 9311320 | 288 | 1513 | 28 | 3 | 4 |
| 318 | 9311321 | 288 | 1578 | 33 | 5 | 3 |
| 319 | 9311322 | 298 | 1778 | 36 | 3 | 2 |
| 320 | 9311323 | 234 | 1356 | 29 | 5 | 1 |
| 321 | 9311324 | 258 | 1172 | 22 | 0 | 2 |
| 322 | 9311325 | 233 | 1163 | 20 | 2 | 3 |
| 323 | 9311326 | 211 | 983 | 16 | 1 | 3 |
| 324 | 9311327 | 175 | 714 | 16 | 2 | 2 |
| 325 | 9311328 | 273 | 1501 | 20 | 6 | 0 |
| 326 | 9311329 | 182 | 769 | 19 | 4 | 2 |
| 327 | 9311330 | 298 | 1493 | 34 | 4 | 3 |
| 328 | 9311331 | 225 | 1215 | 17 | 3 | 2 |
| 329 | 9311332 | 228 | 1073 | 21 | 3 | 3 |
| 330 | 9311333 | 185 | 808 | 16 | 4 | 1 |
| 331 | 9311334 | 243 | 1447 | 25 | 8 | 4 |
| 332 | 9311335 | 201 | 1083 | 17 | 6 | 1 |
| 333 | 9311336 | 358 | 1706 | 30 | 15 | 4 |
| 334 | 9311337 | 175 | 936 | 20 | 1 | 2 |
| 335 | 9311338 | 225 | 1144 | 23 | 3 | 3 |
| 336 | 9311339 | 212 | 1407 | 14 | 4 | 2 |
| 337 | 9311340 | 246 | 1490 | 21 | 5 | 3 |
| 338 | 9311341 | 209 | 1085 | 15 | 4 | 2 |
| 339 | 9311342 | 282 | 1352 | 27 | 6 | 3 |
| 340 | 9311343 | 209 | 1078 | 25 | 1 | 5 |
| 341 | 9311344 | 200 | 1035 | 18 | 2 | 3 |
| 342 | 9311345 | 222 | 1170 | 30 | 5 | 2 |
| 343 | 9311346 | 192 | 911 | 16 | 2 | 2 |
| 344 | 9311347 | 226 | 1056 | 27 | 2 | 3 |
| 345 | 9311348 | 337 | 1524 | 48 | 5 | 5 |
| 346 | 9311349 | 222 | 942 | 18 | 2 | 4 |
| 347 | 9311350 | 233 | 1009 | 19 | 3 | 1 |
| 348 | 9311351 | 238 | 1174 | 20 | 1 | 2 |
| 349 | 9311352 | 257 | 1139 | 21 | 1 | 2 |
| 350 | 9311353 | 292 | 1492 | 19 | 0 | 2 |
| 351 | 9311354 | 229 | 1075 | 26 | 0 | 3 |
| 352 | 9311355 | 237 | 1134 | 22 | 1 | 4 |
| 353 | 9311356 | 332 | 1543 | 30 | 8 | 6 |
| 354 | 9311357 | 303 | 1340 | 23 | 4 | 5 |
| 355 | 9311358 | 232 | 1108 | 17 | 4 | 1 |
| 356 | 9311359 | 350 | 1900 | 34 | 8 | 2 |
| 357 | 9311360 | 266 | 1171 | 30 | 3 | 3 |
| 358 | 9311361 | 202 | 917 | 12 | 1 | 2 |
| 359 | 9311362 | 368 | 1640 | 22 | 1 | 3 |
| 360 | 9311363 | 184 | 795 | 10 | 4 | 1 |
| 361 | 9311364 | 231 | 947 | 14 | 5 | 1 |
| 362 | 9311365 | 279 | 1475 | 19 | 4 | 5 |
| 363 | 9311366 | 142 | 476 | 13 | 2 | 1 |
| 364 | 9311367 | 259 | 1017 | 25 | 2 | 3 |
| 365 | 9311368 | 239 | 898 | 25 | 1 | 4 |
| 366 | 9311369 | 220 | 1232 | 20 | 2 | 1 |
| 367 | 9311370 | 280 | 1372 | 24 | 2 | 4 |
| 368 | 9311371 | 166 | 750 | 9 | 7 | 2 |
| 369 | 9311372 | 262 | 1489 | 21 | 7 | 3 |
| 370 | 9311373 | 195 | 977 | 12 | 0 | 2 |
| 371 | 9311374 | 242 | 1320 | 31 | 3 | 3 |
| 372 | 9311375 | 350 | 2261 | 42 | 4 | 4 |
| 373 | 9311376 | 384 | 1832 | 42 | 4 | 3 |
| 374 | 9311377 | 303 | 1431 | 20 | 4 | 2 |
| 375 | 9311378 | 218 | 1072 | 21 | 6 | 3 |
| 376 | 9311379 | 226 | 1179 | 23 | 1 | 4 |
| 377 | 9311380 | 252 | 1525 | 9 | 1 | 3 |
| 378 | 9311381 | 261 | 1298 | 32 | 3 | 2 |
| 379 | 9311382 | 305 | 1535 | 45 | 0 | 4 |
| 380 | 9311383 | 219 | 1091 | 18 | 5 | 5 |
| 381 | 9311384 | 298 | 2204 | 35 | 0 | 3 |
| 382 | 9311385 | 188 | 831 | 11 | 3 | 3 |
| 383 | 9311386 | 279 | 1619 | 17 | 1 | 6 |
| 384 | 9311387 | 263 | 1049 | 32 | 5 | 4 |
| 385 | 9311388 | 225 | 1317 | 19 | 5 | 3 |
| 386 | 9311389 | 243 | 910 | 17 | 1 | 5 |
| 387 | 9311390 | 500 | 2933 | 39 | 4 | 5 |
| 388 | 9311391 | 289 | 1371 | 18 | 3 | 3 |
| 389 | 9311392 | 216 | 1219 | 23 | 1 | 1 |
| 390 | 9311393 | 313 | 2010 | 32 | 4 | 3 |
| 391 | 9311394 | 337 | 1789 | 32 | 4 | 5 |
| 392 | 9311395 | 234 | 1347 | 27 | 1 | 3 |
| 393 | 9311396 | 219 | 1134 | 30 | 0 | 3 |
| 394 | 9311397 | 303 | 1980 | 39 | 2 | 2 |
| 395 | 9311398 | 294 | 1819 | 39 | 3 | 4 |
| 396 | 9311399 | 226 | 972 | 16 | 3 | 2 |
| 397 | 9311400 | 195 | 885 | 12 | 1 | 3 |
| 398 | 9311401 | 278 | 1294 | 33 | 0 | 4 |
| 399 | 9311402 | 417 | 2076 | 56 | 0 | 5 |
| 400 | 9311403 | 259 | 1215 | 28 | 2 | 2 |
| 401 | 9311404 | 193 | 943 | 15 | 2 | 1 |
| 402 | 9311405 | 215 | 752 | 16 | 1 | 3 |
| 403 | 9311406 | 235 | 1389 | 30 | 1 | 2 |
| 404 | 9311407 | 281 | 1533 | 26 | 6 | 2 |
| 405 | 9311408 | 225 | 1113 | 25 | 3 | 3 |
| 406 | 9311409 | 421 | 2535 | 29 | 0 | 2 |
| 407 | 9311410 | 232 | 1067 | 30 | 2 | 3 |
| 408 | 9311411 | 222 | 1230 | 29 | 2 | 3 |
| 409 | 9311412 | 249 | 1523 | 28 | 3 | 2 |
| 410 | 9311413 | 335 | 1570 | 38 | 0 | 5 |
| 411 | 9311414 | 215 | 1001 | 21 | 7 | 1 |
| 412 | 9311415 | 214 | 889 | 21 | 1 | 1 |
| 413 | 9311416 | 205 | 1204 | 21 | 2 | 1 |
| 414 | 9311417 | 276 | 1350 | 38 | 2 | 2 |
| 415 | 9311418 | 200 | 948 | 28 | 2 | 1 |
| 416 | 9311419 | 234 | 1020 | 10 | 4 | 1 |
| 417 | 9311420 | 292 | 1730 | 29 | 3 | 2 |
| 418 | 9311421 | 308 | 1848 | 26 | 2 | 4 |
| 419 | 9311422 | 245 | 1264 | 20 | 1 | 1 |
| 420 | 9311423 | 191 | 1088 | 19 | 4 | 2 |
| 421 | 9311424 | 245 | 1298 | 22 | 10 | 1 |
| 422 | 9311425 | 431 | 2915 | 75 | 3 | 5 |
| 423 | 9311426 | 317 | 2037 | 34 | 4 | 2 |
| 424 | 9311427 | 252 | 1601 | 20 | 2 | 1 |
| 425 | 9311428 | 262 | 1441 | 31 | 4 | 2 |
| 426 | 9311429 | 232 | 1254 | 19 | 3 | 4 |
| 427 | 9311430 | 268 | 1436 | 32 | 1 | 3 |
| 428 | 9311431 | 252 | 1440 | 19 | 5 | 2 |
| 429 | 9311432 | 500 | 2449 | 54 | 6 | 3 |
| 430 | 9311433 | 325 | 2158 | 23 | 10 | 5 |
| 431 | 9311434 | 234 | 1449 | 36 | 7 | 4 |
| 432 | 9311435 | 235 | 968 | 16 | 3 | 5 |
| 433 | 9311436 | 243 | 741 | 16 | 1 | 3 |
| 434 | 9311437 | 225 | 1214 | 21 | 3 | 2 |
| 435 | 9311438 | 292 | 1135 | 21 | 3 | 4 |
| 436 | 9311439 | 225 | 1266 | 19 | 2 | 4 |
| 437 | 9311440 | 289 | 1321 | 21 | 3 | 2 |
| 438 | 9311441 | 285 | 1274 | 21 | 1 | 3 |
| 439 | 9311442 | 250 | 1134 | 23 | 2 | 1 |
| 440 | 9311443 | 272 | 1208 | 24 | 5 | 4 |
| 441 | 9311444 | 190 | 709 | 12 | 5 | 1 |
| 442 | 9311445 | 316 | 1527 | 44 | 10 | 3 |
| 443 | 9311446 | 319 | 1609 | 39 | 4 | 3 |
| 444 | 9311447 | 195 | 920 | 17 | 8 | 2 |
| 445 | 9311448 | 261 | 1265 | 19 | 6 | 4 |
| 446 | 9311449 | 295 | 1445 | 26 | 4 | 2 |
| 447 | 9311450 | 232 | 1026 | 24 | 6 | 2 |
| 448 | 9311451 | 220 | 949 | 21 | 12 | 4 |
| 449 | 9311452 | 305 | 1511 | 25 | 6 | 6 |
| 450 | 9311453 | 267 | 1208 | 30 | 3 | 2 |
| 451 | 9311454 | 194 | 1062 | 17 | 3 | 2 |
| 452 | 9311455 | 293 | 1282 | 27 | 5 | 5 |
| 453 | 9311456 | 199 | 865 | 18 | 3 | 2 |
| 454 | 9311457 | 225 | 1161 | 20 | 1 | 4 |
| 455 | 9311458 | 149 | 573 | 11 | 1 | 2 |
| 456 | 9311459 | 233 | 1302 | 19 | 1 | 2 |
| 457 | 9311460 | 200 | 1109 | 15 | 0 | 4 |
| 458 | 9311461 | 225 | 825 | 9 | 5 | 4 |
| 459 | 9311462 | 155 | 563 | 11 | 5 | 3 |
| 460 | 9311463 | 241 | 1084 | 22 | 4 | 2 |
| 461 | 9311464 | 276 | 1467 | 26 | 6 | 2 |
| 462 | 9311465 | 234 | 1402 | 18 | 4 | 3 |
| 463 | 9311466 | 252 | 1133 | 23 | 8 | 2 |
| 464 | 9311467 | 292 | 1638 | 42 | 7 | 5 |
| 465 | 9311468 | 194 | 824 | 15 | 0 | 3 |
| 466 | 9311469 | 255 | 1273 | 24 | 3 | 3 |
| 467 | 9311470 | 264 | 1641 | 26 | 4 | 5 |
| 468 | 9311471 | 259 | 1467 | 20 | 9 | 5 |
| 469 | 9311472 | 291 | 1241 | 22 | 2 | 4 |
| 470 | 9311473 | 283 | 1827 | 17 | 4 | 3 |
| 471 | 9311474 | 228 | 954 | 13 | 0 | 6 |
| 472 | 9311475 | 245 | 1181 | 13 | 1 | 2 |
| 473 | 9311476 | 205 | 899 | 14 | 2 | 1 |
| 474 | 9311477 | 183 | 920 | 21 | 3 | 3 |
| 475 | 9311478 | 207 | 977 | 5 | 2 | 1 |
| 476 | 9311479 | 290 | 1591 | 39 | 3 | 4 |
| 477 | 9311480 | 239 | 1372 | 25 | 0 | 3 |
| 478 | 9311481 | 279 | 1213 | 29 | 4 | 5 |
| 479 | 9311482 | 280 | 1601 | 12 | 4 | 2 |
| 480 | 9311483 | 268 | 1263 | 24 | 7 | 2 |
| 481 | 9311484 | 274 | 1367 | 22 | 2 | 2 |
| 482 | 9311485 | 217 | 1161 | 20 | 0 | 3 |
| 483 | 9311486 | 278 | 1297 | 31 | 0 | 5 |
| 484 | 9311487 | 307 | 2093 | 37 | 1 | 5 |
| 485 | 9311488 | 232 | 1334 | 18 | 1 | 4 |
| 486 | 9311489 | 500 | 3256 | 42 | 3 | 4 |
| 487 | 9311490 | 392 | 2328 | 49 | 3 | 3 |
| 488 | 9311491 | 288 | 1606 | 27 | 1 | 3 |
| 489 | 9311492 | 213 | 872 | 20 | 1 | 1 |
| 490 | 9311493 | 248 | 1435 | 22 | 5 | 3 |
| 491 | 9311494 | 290 | 1165 | 24 | 1 | 5 |
| 492 | 9311495 | 312 | 1491 | 29 | 3 | 3 |
| 493 | 9311496 | 209 | 950 | 19 | 2 | 3 |
| 494 | 9311497 | 322 | 1658 | 39 | 0 | 3 |
| 495 | 9311498 | 257 | 1716 | 34 | 2 | 4 |
| 496 | 9311499 | 175 | 867 | 15 | 2 | 3 |
| 497 | 9311500 | 205 | 1219 | 30 | 1 | 2 |
| 498 | 9311501 | 367 | 2123 | 30 | 3 | 2 |
| 499 | 9311502 | 181 | 770 | 15 | 2 | 3 |
| 500 | 9311503 | 208 | 1237 | 30 | 4 | 1 |
| 501 | 9311504 | 162 | 758 | 16 | 1 | 2 |
| 502 | 9311505 | 294 | 1565 | 44 | 3 | 3 |
| 503 | 9311506 | 167 | 845 | 15 | 5 | 1 |
| 504 | 9311507 | 287 | 1007 | 17 | 3 | 1 |
| 505 | 9311508 | 242 | 1312 | 28 | 3 | 3 |
| 506 | 9311509 | 254 | 1599 | 28 | 3 | 4 |
| 507 | 9311510 | 330 | 2259 | 38 | 1 | 3 |
| 508 | 9311511 | 384 | 1972 | 53 | 5 | 6 |
| 509 | 9311512 | 226 | 1297 | 26 | 1 | 1 |
| 510 | 9311513 | 271 | 1328 | 18 | 1 | 4 |
| 511 | 9311514 | 218 | 1209 | 18 | 4 | 3 |
| 512 | 9311515 | 244 | 1421 | 23 | 10 | 2 |
| 513 | 9311516 | 273 | 1442 | 35 | 4 | 3 |
| 514 | 9311517 | 318 | 1989 | 32 | 1 | 3 |
| 515 | 9311518 | 245 | 1097 | 15 | 6 | 2 |
| 516 | 9311519 | 237 | 1356 | 24 | 5 | 3 |
| 517 | 9311520 | 220 | 1046 | 18 | 2 | 2 |
| 518 | 9311521 | 255 | 1200 | 20 | 1 | 2 |
| 519 | 9311522 | 306 | 1809 | 34 | 6 | 3 |
| 520 | 9311523 | 280 | 1239 | 30 | 2 | 6 |
| 521 | 9311524 | 284 | 1540 | 25 | 3 | 4 |
| 522 | 9311525 | 173 | 828 | 12 | 2 | 2 |
| 523 | 9311526 | 245 | 1179 | 26 | 1 | 2 |
| 524 | 9311527 | 367 | 2040 | 28 | 2 | 4 |
| 525 | 9311528 | 272 | 1414 | 34 | 3 | 5 |
| 526 | 9311529 | 271 | 1436 | 22 | 8 | 5 |
| 527 | 9311530 | 200 | 968 | 20 | 4 | 1 |
| 528 | 9311531 | 204 | 802 | 20 | 1 | 4 |
| 529 | 9311532 | 182 | 807 | 29 | 4 | 5 |
| 530 | 9311533 | 230 | 809 | 20 | 1 | 3 |
| 531 | 9311534 | 201 | 890 | 17 | 3 | 4 |
| 532 | 9311535 | 309 | 1711 | 39 | 1 | 3 |
| 533 | 9311536 | 220 | 1143 | 21 | 6 | 3 |
| 534 | 9311537 | 182 | 780 | 14 | 4 | 0 |
| 535 | 9311538 | 440 | 2338 | 47 | 4 | 5 |
| 536 | 9311539 | 239 | 1248 | 25 | 0 | 2 |
| 537 | 9311540 | 294 | 1440 | 30 | 5 | 3 |
| 538 | 9311541 | 232 | 1159 | 16 | 3 | 6 |
| 539 | 9311542 | 242 | 869 | 14 | 3 | 2 |
| 540 | 9311543 | 229 | 1233 | 12 | 9 | 2 |
| 541 | 9311544 | 281 | 1750 | 40 | 7 | 3 |
| 542 | 9311545 | 198 | 915 | 21 | 3 | 2 |
| 543 | 9311546 | 216 | 840 | 23 | 6 | 1 |
| 544 | 9311547 | 268 | 1580 | 15 | 1 | 4 |
| 545 | 9311548 | 227 | 1141 | 24 | 2 | 3 |
| 546 | 9311549 | 244 | 1352 | 24 | 5 | 2 |
| 547 | 9311550 | 241 | 1424 | 18 | 8 | 4 |
| 548 | 9311551 | 230 | 1027 | 14 | 3 | 4 |
| 549 | 9311552 | 261 | 1321 | 19 | 1 | 2 |
| 550 | 9311553 | 339 | 1775 | 30 | 0 | 3 |
| 551 | 9311554 | 232 | 1227 | 20 | 2 | 2 |
| 552 | 9311555 | 283 | 1450 | 32 | 6 | 4 |
| 553 | 9311556 | 204 | 1299 | 19 | 1 | 2 |
| 554 | 9311557 | 227 | 1055 | 20 | 1 | 3 |
| 555 | 9311558 | 199 | 1007 | 17 | 3 | 4 |
| 556 | 9311559 | 277 | 1254 | 26 | 2 | 4 |
| 557 | 9311560 | 258 | 1445 | 33 | 3 | 4 |
| 558 | 9311561 | 183 | 904 | 19 | 0 | 2 |
| 559 | 9311562 | 283 | 1623 | 25 | 1 | 4 |
| 560 | 9311563 | 262 | 1652 | 22 | 6 | 4 |
| 561 | 9311564 | 199 | 866 | 15 | 1 | 5 |
| 562 | 9311565 | 282 | 1380 | 17 | 1 | 4 |
| 563 | 9311566 | 439 | 2406 | 18 | 0 | 2 |
| 564 | 9311567 | 253 | 1139 | 11 | 2 | 4 |
| 565 | 9311568 | 283 | 1591 | 28 | 2 | 4 |
| 566 | 9311569 | 308 | 1822 | 25 | 7 | 6 |
| 567 | 9311570 | 270 | 1392 | 32 | 3 | 2 |
| 568 | 9311571 | 282 | 1520 | 33 | 1 | 3 |
| 569 | 9311572 | 244 | 1173 | 19 | 0 | 3 |
| 570 | 9311573 | 200 | 1123 | 22 | 1 | 5 |
| 571 | 9311574 | 336 | 2055 | 49 | 4 | 4 |
| 572 | 9311575 | 202 | 1065 | 20 | 1 | 5 |
| 573 | 9311576 | 237 | 1277 | 29 | 4 | 2 |
| 574 | 9311577 | 210 | 1194 | 24 | 0 | 3 |
| 575 | 9311578 | 297 | 1448 | 26 | 3 | 2 |
| 576 | 9311579 | 276 | 1137 | 22 | 4 | 4 |
| 577 | 9311580 | 330 | 1135 | 33 | 2 | 4 |
| 578 | 9311581 | 328 | 2041 | 39 | 2 | 4 |
| 579 | 9311582 | 269 | 1088 | 28 | 2 | 3 |
| 580 | 9311583 | 263 | 1800 | 29 | 3 | 5 |
| 581 | 9311584 | 238 | 1188 | 26 | 2 | 2 |
| 582 | 9311585 | 208 | 1340 | 29 | 4 | 1 |
| 583 | 9311586 | 201 | 923 | 17 | 2 | 2 |
| 584 | 9311587 | 239 | 1302 | 21 | 5 | 2 |
| 585 | 9311588 | 255 | 1556 | 34 | 4 | 3 |
| 586 | 9311589 | 262 | 1723 | 28 | 3 | 2 |
| 587 | 9311590 | 207 | 845 | 15 | 5 | 1 |
| 588 | 9311591 | 205 | 990 | 17 | 2 | 2 |
| 589 | 9311592 | 303 | 1844 | 38 | 2 | 3 |
| 590 | 9311593 | 209 | 1123 | 20 | 0 | 1 |
| 591 | 9311594 | 318 | 1728 | 32 | 4 | 3 |
| 592 | 9311595 | 242 | 1418 | 22 | 3 | 1 |
| 593 | 9311596 | 500 | 3070 | 26 | 5 | 1 |
| 594 | 9311597 | 275 | 1654 | 28 | 6 | 2 |
| 595 | 9311598 | 214 | 1172 | 9 | 5 | 1 |
| 596 | 9311599 | 204 | 1309 | 22 | 6 | 1 |
| 597 | 9311600 | 335 | 1824 | 33 | 18 | 3 |
| 598 | 9311601 | 295 | 1732 | 14 | 7 | 1 |
| 599 | 9311602 | 309 | 1820 | 43 | 2 | 4 |
| 600 | 9311603 | 248 | 1041 | 33 | 2 | 3 |
| 601 | 9311604 | 269 | 1189 | 24 | 1 | 3 |
| 602 | 9311605 | 183 | 806 | 16 | 1 | 2 |
| 603 | 9311606 | 304 | 1813 | 31 | 1 | 3 |
| 604 | 9311607 | 323 | 1414 | 29 | 9 | 4 |
| 605 | 9311608 | 223 | 999 | 33 | 4 | 3 |
| 606 | 9311609 | 277 | 1549 | 22 | 2 | 2 |
| 607 | 9311610 | 279 | 1523 | 34 | 1 | 5 |
| 608 | 9311611 | 280 | 1552 | 17 | 17 | 4 |
| 609 | 9311612 | 266 | 1203 | 12 | 3 | 3 |
| 610 | 9311613 | 306 | 1284 | 27 | 0 | 3 |
| 611 | 9311614 | 257 | 1020 | 16 | 9 | 5 |
| 612 | 9311615 | 201 | 1039 | 20 | 6 | 2 |
| 613 | 9311616 | 242 | 1215 | 25 | 3 | 3 |
| 614 | 9311617 | 375 | 2373 | 32 | 2 | 3 |
| 615 | 9311618 | 188 | 985 | 25 | 5 | 3 |
| 616 | 9311619 | 231 | 1030 | 18 | 1 | 2 |
| 617 | 9311620 | 389 | 2208 | 37 | 16 | 3 |
| 618 | 9311621 | 255 | 1355 | 23 | 0 | 2 |
| 619 | 9311622 | 234 | 1053 | 27 | 2 | 3 |
| 620 | 9311623 | 264 | 981 | 28 | 2 | 3 |
| 621 | 9311624 | 284 | 1198 | 21 | 1 | 2 |
| 622 | 9311625 | 245 | 1374 | 26 | 4 | 2 |
| 623 | 9311626 | 193 | 1058 | 16 | 2 | 1 |
| 624 | 9311627 | 195 | 998 | 18 | 7 | 4 |
| 625 | 9311628 | 220 | 1091 | 20 | 4 | 2 |
| 626 | 9311629 | 252 | 1158 | 20 | 4 | 2 |
| 627 | 9311630 | 303 | 1909 | 25 | 6 | 1 |
| 628 | 9311631 | 218 | 1327 | 36 | 3 | 1 |
| 629 | 9311632 | 212 | 1117 | 13 | 4 | 3 |
| 630 | 9311633 | 188 | 936 | 12 | 2 | 2 |
| 631 | 9311634 | 240 | 1199 | 26 | 4 | 3 |
| 632 | 9311635 | 311 | 1872 | 31 | 7 | 3 |
| 633 | 9311636 | 236 | 1111 | 14 | 1 | 3 |
| 634 | 9311637 | 265 | 1371 | 36 | 1 | 4 |
| 635 | 9311638 | 272 | 1175 | 29 | 8 | 3 |
| 636 | 9311639 | 230 | 1180 | 24 | 4 | 3 |
| 637 | 9311640 | 202 | 1115 | 24 | 4 | 4 |
| 638 | 9311641 | 208 | 1058 | 27 | 1 | 3 |
| 639 | 9311642 | 292 | 1426 | 36 | 2 | 3 |
| 640 | 9311643 | 367 | 1903 | 22 | 1 | 4 |
| 641 | 9311644 | 219 | 1178 | 21 | 3 | 2 |
| 642 | 9311645 | 248 | 829 | 15 | 1 | 4 |
| 643 | 9311646 | 321 | 1458 | 24 | 12 | 3 |
| 644 | 9311647 | 284 | 1605 | 16 | 1 | 3 |
| 645 | 9311648 | 213 | 1026 | 18 | 2 | 3 |
| 646 | 9311649 | 189 | 776 | 11 | 3 | 2 |
| 647 | 9311650 | 282 | 1439 | 27 | 1 | 2 |
| 648 | 9311651 | 228 | 1355 | 16 | 1 | 5 |
| 649 | 9311652 | 346 | 1692 | 28 | 1 | 5 |
| 650 | 9311653 | 263 | 1463 | 26 | 2 | 2 |
| 651 | 9311654 | 236 | 1201 | 18 | 1 | 4 |
| 652 | 9311655 | 451 | 1962 | 34 | 5 | 5 |
| 653 | 9311656 | 326 | 1784 | 26 | 0 | 5 |
| 654 | 9311657 | 261 | 1448 | 27 | 4 | 4 |
| 655 | 9311658 | 216 | 1196 | 22 | 2 | 3 |
| 656 | 9311659 | 257 | 1089 | 25 | 1 | 3 |
| 657 | 9311660 | 223 | 1269 | 28 | 3 | 0 |
| 658 | 9311661 | 265 | 1386 | 35 | 3 | 4 |
| 659 | 9311662 | 293 | 1615 | 42 | 4 | 4 |
| 660 | 9311663 | 214 | 1187 | 18 | 2 | 2 |
| 661 | 9311664 | 276 | 1854 | 28 | 4 | 3 |
| 662 | 9311665 | 172 | 701 | 11 | 0 | 2 |
| 663 | 9311666 | 219 | 973 | 22 | 2 | 5 |
| 664 | 9311667 | 289 | 1426 | 38 | 0 | 3 |
| 665 | 9311668 | 288 | 1634 | 43 | 0 | 4 |
| 666 | 9311669 | 225 | 843 | 11 | 4 | 2 |
| 667 | 9311670 | 245 | 1210 | 20 | 0 | 1 |
| 668 | 9311671 | 242 | 1343 | 29 | 1 | 3 |
| 669 | 9311672 | 258 | 1226 | 31 | 0 | 3 |
| 670 | 9311673 | 192 | 735 | 20 | 2 | 2 |
| 671 | 9311674 | 206 | 1101 | 23 | 4 | 2 |
| 672 | 9311675 | 260 | 1644 | 26 | 1 | 1 |
| 673 | 9311676 | 192 | 969 | 24 | 1 | 0 |
| 674 | 9311677 | 191 | 1023 | 26 | 4 | 1 |
| 675 | 9311678 | 277 | 1628 | 25 | 5 | 3 |
| 676 | 9311679 | 207 | 1151 | 22 | 4 | 4 |
| 677 | 9311680 | 328 | 1972 | 45 | 7 | 2 |
| 678 | 9311681 | 206 | 1091 | 23 | 2 | 2 |
| 679 | 9311682 | 238 | 1154 | 23 | 4 | 0 |
| 680 | 9311683 | 249 | 1661 | 24 | 12 | 2 |
| 681 | 9311684 | 210 | 1058 | 13 | 3 | 2 |
| 682 | 9311685 | 225 | 1231 | 27 | 4 | 3 |
| 683 | 9311686 | 226 | 1061 | 15 | 4 | 4 |
| 684 | 9311687 | 237 | 1245 | 27 | 6 | 1 |
| 685 | 9311688 | 187 | 589 | 10 | 4 | 3 |
| 686 | 9311689 | 279 | 1557 | 30 | 1 | 2 |
| 687 | 9311690 | 191 | 1098 | 18 | 5 | 1 |
| 688 | 9311691 | 221 | 1083 | 17 | 5 | 1 |
| 689 | 9311692 | 269 | 1469 | 27 | 4 | 3 |
| 690 | 9311693 | 261 | 1485 | 37 | 4 | 4 |
| 691 | 9311694 | 200 | 930 | 5 | 1 | 1 |
| 692 | 9311695 | 262 | 1559 | 14 | 3 | 4 |
| 693 | 9311696 | 245 | 1162 | 31 | 6 | 5 |
| 694 | 9311697 | 211 | 1125 | 26 | 5 | 3 |
| 695 | 9311698 | 214 | 1171 | 15 | 3 | 3 |
| 696 | 9311699 | 207 | 1033 | 12 | 1 | 4 |
| 697 | 9311700 | 198 | 791 | 18 | 0 | 3 |
| 698 | 9311701 | 199 | 1007 | 24 | 2 | 2 |
| 699 | 9311702 | 348 | 1727 | 25 | 2 | 5 |
| 700 | 9311703 | 322 | 1483 | 23 | 4 | 3 |
| 701 | 9311704 | 304 | 1688 | 39 | 3 | 4 |
| 702 | 9311705 | 336 | 1504 | 31 | 0 | 5 |
| 703 | 9311706 | 280 | 934 | 16 | 1 | 3 |
| 704 | 9311707 | 213 | 874 | 19 | 5 | 3 |
| 705 | 9311708 | 274 | 1195 | 33 | 5 | 4 |
| 706 | 9311709 | 254 | 1071 | 28 | 5 | 2 |
| 707 | 9311710 | 218 | 922 | 22 | 4 | 2 |
| 708 | 9311711 | 244 | 1137 | 21 | 3 | 4 |
| 709 | 9311712 | 193 | 897 | 24 | 4 | 1 |
| 710 | 9311713 | 307 | 1462 | 14 | 5 | 2 |
| 711 | 9311714 | 184 | 833 | 18 | 5 | 0 |
| 712 | 9311715 | 224 | 989 | 19 | 5 | 3 |
| 713 | 9311716 | 224 | 1088 | 30 | 1 | 4 |
| 714 | 9311717 | 272 | 1239 | 18 | 1 | 2 |
| 715 | 9311718 | 264 | 1389 | 36 | 4 | 4 |
| 716 | 9311719 | 332 | 1444 | 28 | 2 | 4 |
| 717 | 9311720 | 356 | 1673 | 47 | 4 | 3 |
| 718 | 9311721 | 246 | 1205 | 26 | 9 | 4 |
| 719 | 9311722 | 302 | 1375 | 42 | 7 | 4 |
| 720 | 9311723 | 271 | 1393 | 30 | 3 | 3 |
| 721 | 9311724 | 200 | 948 | 11 | 6 | 2 |
| 722 | 9311725 | 252 | 1356 | 13 | 3 | 3 |
| 723 | 9311726 | 306 | 1853 | 26 | 3 | 3 |
| 724 | 9311727 | 283 | 1389 | 30 | 3 | 3 |
| 725 | 9311728 | 230 | 1422 | 21 | 2 | 3 |
| 726 | 9311729 | 199 | 1032 | 11 | 4 | 2 |
| 727 | 9311730 | 220 | 1138 | 19 | 8 | 2 |
| 728 | 9311731 | 240 | 1212 | 22 | 3 | 4 |
| 729 | 9311732 | 182 | 778 | 12 | 4 | 1 |
| 730 | 9311733 | 218 | 1279 | 17 | 0 | 3 |
| 731 | 9311734 | 222 | 1063 | 14 | 7 | 4 |
| 732 | 9311735 | 360 | 2142 | 22 | 1 | 1 |
| 733 | 9311736 | 258 | 1883 | 22 | 7 | 4 |
| 734 | 9311737 | 277 | 1576 | 21 | 1 | 2 |
| 735 | 9311738 | 229 | 1219 | 22 | 3 | 5 |
| 736 | 9311739 | 277 | 997 | 15 | 0 | 3 |
| 737 | 9311740 | 262 | 1832 | 26 | 0 | 2 |
| 738 | 9311741 | 306 | 1905 | 33 | 6 | 4 |
| 739 | 9311742 | 320 | 1960 | 40 | 4 | 3 |
| 740 | 9311743 | 360 | 1726 | 28 | 4 | 4 |
| 741 | 9311744 | 298 | 1659 | 35 | 2 | 5 |
| 742 | 9311745 | 202 | 1073 | 16 | 5 | 1 |
| 743 | 9311746 | 220 | 1032 | 22 | 0 | 3 |
| 744 | 9311747 | 243 | 1227 | 31 | 1 | 4 |
| 745 | 9311748 | 354 | 1946 | 29 | 4 | 4 |
| 746 | 9311749 | 258 | 1490 | 33 | 3 | 4 |
| 747 | 9311750 | 257 | 1520 | 40 | 1 | 2 |
| 748 | 9311751 | 283 | 1288 | 30 | 3 | 5 |
| 749 | 9311752 | 241 | 1273 | 27 | 1 | 3 |
| 750 | 9311753 | 365 | 2159 | 51 | 2 | 4 |
| 751 | 9311754 | 333 | 1552 | 37 | 1 | 5 |
| 752 | 9311755 | 348 | 1764 | 39 | 3 | 5 |
| 753 | 9311756 | 190 | 731 | 16 | 3 | 3 |
| 754 | 9311757 | 207 | 1107 | 12 | 2 | 0 |
| 755 | 9311758 | 338 | 1696 | 47 | 0 | 4 |
| 756 | 9311759 | 270 | 1063 | 20 | 3 | 4 |
| 757 | 9311760 | 290 | 1730 | 35 | 2 | 3 |
| 758 | 9311761 | 235 | 1219 | 27 | 2 | 3 |
| 759 | 9311762 | 221 | 1172 | 16 | 0 | 4 |
| 760 | 9311763 | 191 | 827 | 14 | 2 | 1 |
| 761 | 9311764 | 331 | 1904 | 27 | 6 | 2 |
| 762 | 9311765 | 263 | 1501 | 31 | 9 | 3 |
| 763 | 9311766 | 258 | 1115 | 25 | 5 | 3 |
| 764 | 9311767 | 242 | 1436 | 34 | 1 | 2 |
| 765 | 9311768 | 332 | 2160 | 42 | 4 | 3 |
| 766 | 9311769 | 254 | 1402 | 41 | 2 | 2 |
| 767 | 9311770 | 196 | 1073 | 21 | 1 | 2 |
| 768 | 9311771 | 319 | 1726 | 32 | 8 | 4 |
| 769 | 9311772 | 307 | 1987 | 31 | 12 | 4 |
| 770 | 9311773 | 216 | 1069 | 8 | 6 | 4 |
| 771 | 9311774 | 174 | 701 | 10 | 5 | 3 |
| 772 | 9311775 | 210 | 1040 | 14 | 5 | 3 |
| 773 | 9311776 | 241 | 1433 | 22 | 3 | 2 |
| 774 | 9311777 | 211 | 1071 | 16 | 2 | 1 |
| 775 | 9311778 | 264 | 1542 | 32 | 4 | 1 |
| 776 | 9311779 | 238 | 1267 | 25 | 1 | 1 |
| 777 | 9311780 | 232 | 1385 | 23 | 5 | 2 |
| 778 | 9311781 | 219 | 1237 | 20 | 5 | 4 |
| 779 | 9311782 | 221 | 1014 | 19 | 4 | 5 |
| 780 | 9311783 | 242 | 1300 | 22 | 2 | 2 |
| 781 | 9311784 | 246 | 1541 | 30 | 6 | 5 |
| 782 | 9311785 | 223 | 1130 | 19 | 3 | 2 |
| 783 | 9311786 | 353 | 1855 | 49 | 9 | 5 |
| 784 | 9311787 | 243 | 1135 | 21 | 6 | 1 |
| 785 | 9311788 | 248 | 1423 | 30 | 2 | 2 |
| 786 | 9311789 | 212 | 887 | 24 | 6 | 5 |
| 787 | 9311790 | 198 | 921 | 20 | 4 | 2 |
| 788 | 9311791 | 239 | 876 | 23 | 3 | 3 |
| 789 | 9311792 | 262 | 1169 | 26 | 4 | 5 |
| 790 | 9311793 | 309 | 1660 | 35 | 3 | 6 |
| 791 | 9311794 | 420 | 1953 | 37 | 3 | 3 |
| 792 | 9311795 | 228 | 1065 | 25 | 4 | 2 |
| 793 | 9311796 | 264 | 1092 | 26 | 7 | 3 |
| 794 | 9311797 | 218 | 861 | 20 | 2 | 3 |
| 795 | 9311798 | 245 | 1153 | 23 | 8 | 5 |
| 796 | 9311799 | 263 | 1465 | 19 | 4 | 2 |
| 797 | 9311800 | 254 | 1217 | 23 | 0 | 2 |
| 798 | 9311801 | 243 | 1373 | 29 | 4 | 4 |
| 799 | 9311802 | 201 | 1055 | 23 | 1 | 2 |
| 800 | 9311803 | 296 | 1561 | 18 | 5 | 5 |
| 801 | 9311804 | 219 | 1005 | 24 | 4 | 4 |
| 802 | 9311805 | 191 | 851 | 12 | 0 | 2 |
| 803 | 9311806 | 204 | 788 | 20 | 3 | 2 |
| 804 | 9311807 | 315 | 1573 | 46 | 2 | 2 |
| 805 | 9311808 | 280 | 1162 | 32 | 3 | 3 |
| 806 | 9311809 | 164 | 679 | 14 | 2 | 1 |
| 807 | 9311810 | 230 | 1142 | 22 | 3 | 2 |
| 808 | 9311811 | 254 | 1467 | 19 | 10 | 2 |
| 809 | 9311812 | 218 | 1226 | 23 | 3 | 2 |
| 810 | 9311813 | 276 | 1318 | 18 | 2 | 3 |
| 811 | 9311814 | 296 | 1484 | 32 | 5 | 4 |
| 812 | 9311815 | 196 | 922 | 26 | 4 | 2 |
| 813 | 9311816 | 176 | 770 | 15 | 6 | 3 |
| 814 | 9311817 | 216 | 1009 | 19 | 1 | 1 |
| 815 | 9311818 | 222 | 1017 | 24 | 2 | 1 |
| 816 | 9311819 | 244 | 1482 | 18 | 5 | 3 |
| 817 | 9311820 | 287 | 1295 | 27 | 4 | 2 |
| 818 | 9311821 | 233 | 1624 | 20 | 2 | 3 |
| 819 | 9311822 | 304 | 1547 | 17 | 3 | 6 |
| 820 | 9311823 | 255 | 1711 | 23 | 1 | 3 |
| 821 | 9311824 | 209 | 1046 | 19 | 4 | 4 |
| 822 | 9311825 | 342 | 1915 | 21 | 6 | 6 |
| 823 | 9311826 | 276 | 1622 | 31 | 4 | 4 |
| 824 | 9311827 | 238 | 1126 | 12 | 1 | 3 |
| 825 | 9311828 | 319 | 1621 | 17 | 0 | 2 |
| 826 | 9311829 | 298 | 1677 | 34 | 5 | 4 |
| 827 | 9311830 | 199 | 977 | 19 | 1 | 1 |
| 828 | 9311831 | 333 | 1868 | 36 | 1 | 4 |
| 829 | 9311832 | 250 | 1291 | 21 | 1 | 5 |
| 830 | 9311833 | 219 | 1031 | 23 | 1 | 4 |
| 831 | 9311834 | 288 | 1301 | 24 | 2 | 2 |
| 832 | 9311835 | 223 | 913 | 18 | 2 | 2 |
| 833 | 9311836 | 230 | 1221 | 15 | 7 | 2 |
| 834 | 9311837 | 207 | 1235 | 23 | 3 | 3 |
| 835 | 9311838 | 247 | 1440 | 24 | 4 | 2 |
| 836 | 9311839 | 174 | 832 | 18 | 1 | 2 |
| 837 | 9311840 | 241 | 1324 | 24 | 1 | 3 |
| 838 | 9311841 | 307 | 1287 | 30 | 3 | 2 |
| 839 | 9311842 | 191 | 996 | 24 | 3 | 1 |
| 840 | 9311843 | 291 | 1462 | 25 | 1 | 4 |
| 841 | 9311844 | 266 | 1089 | 26 | 6 | 2 |
| 842 | 9311845 | 229 | 1103 | 18 | 2 | 2 |
| 843 | 9311846 | 192 | 1232 | 14 | 2 | 2 |
| 844 | 9311847 | 206 | 1090 | 12 | 6 | 1 |
| 845 | 9311848 | 294 | 1751 | 25 | 1 | 3 |
| 846 | 9311849 | 236 | 1030 | 29 | 3 | 4 |
| 847 | 9311850 | 273 | 1636 | 27 | 8 | 2 |
| 848 | 9311851 | 282 | 1823 | 32 | 8 | 0 |
| 849 | 9311852 | 225 | 1084 | 26 | 4 | 3 |
| 850 | 9311853 | 210 | 1226 | 10 | 6 | 1 |
| 851 | 9311854 | 254 | 1397 | 20 | 2 | 2 |
| 852 | 9311855 | 221 | 1235 | 24 | 5 | 2 |
| 853 | 9311856 | 299 | 1893 | 31 | 4 | 2 |
| 854 | 9311857 | 169 | 760 | 19 | 1 | 2 |
| 855 | 9311858 | 191 | 922 | 18 | 1 | 2 |
| 856 | 9311859 | 187 | 770 | 15 | 3 | 2 |
| 857 | 9311860 | 198 | 1033 | 13 | 1 | 1 |
| 858 | 9311861 | 207 | 901 | 23 | 6 | 2 |
| 859 | 9311862 | 250 | 1744 | 24 | 5 | 1 |
| 860 | 9311863 | 228 | 1147 | 13 | 3 | 2 |
| 861 | 9311864 | 265 | 1407 | 29 | 5 | 3 |
| 862 | 9311865 | 198 | 966 | 12 | 3 | 0 |
| 863 | 9311866 | 307 | 1698 | 41 | 2 | 3 |
| 864 | 9311867 | 348 | 1699 | 25 | 12 | 3 |
| 865 | 9311868 | 248 | 1384 | 12 | 6 | 4 |
| 866 | 9311869 | 206 | 1065 | 23 | 3 | 3 |
| 867 | 9311870 | 208 | 841 | 23 | 3 | 3 |
| 868 | 9311871 | 222 | 1118 | 24 | 7 | 3 |
| 869 | 9311872 | 271 | 1776 | 23 | 15 | 4 |
| 870 | 9311873 | 398 | 1831 | 24 | 9 | 4 |
| 871 | 9311874 | 191 | 1200 | 16 | 3 | 1 |
| 872 | 9311875 | 223 | 935 | 25 | 1 | 4 |
| 873 | 9311876 | 287 | 1345 | 30 | 2 | 5 |
| 874 | 9311877 | 299 | 1679 | 31 | 1 | 5 |
| 875 | 9311878 | 268 | 1351 | 28 | 4 | 3 |
| 876 | 9311879 | 196 | 1128 | 17 | 1 | 1 |
| 877 | 9311880 | 448 | 2314 | 34 | 0 | 2 |
| 878 | 9311881 | 247 | 1010 | 23 | 2 | 4 |
| 879 | 9311882 | 339 | 1728 | 27 | 1 | 4 |
| 880 | 9311883 | 237 | 1160 | 18 | 5 | 3 |
| 881 | 9311884 | 251 | 1011 | 20 | 2 | 1 |
| 882 | 9311885 | 247 | 728 | 14 | 4 | 3 |
| 883 | 9311886 | 221 | 959 | 25 | 3 | 3 |
| 884 | 9311887 | 229 | 1027 | 23 | 7 | 4 |
| 885 | 9311888 | 204 | 985 | 20 | 0 | 4 |
| 886 | 9311889 | 262 | 1227 | 27 | 2 | 2 |
| 887 | 9311890 | 500 | 1816 | 19 | 2 | 4 |
| 888 | 9311891 | 224 | 1146 | 18 | 2 | 1 |
| 889 | 9311892 | 247 | 944 | 22 | 5 | 3 |
| 890 | 9311893 | 248 | 1492 | 28 | 7 | 2 |
| 891 | 9311894 | 206 | 1064 | 25 | 1 | 3 |
| 892 | 9311895 | 285 | 1453 | 18 | 5 | 3 |
| 893 | 9311896 | 212 | 1049 | 13 | 2 | 0 |
| 894 | 9311897 | 290 | 1430 | 30 | 6 | 2 |
| 895 | 9311898 | 261 | 1560 | 15 | 5 | 4 |
| 896 | 9311899 | 251 | 1281 | 29 | 5 | 1 |
| 897 | 9311900 | 349 | 1880 | 36 | 1 | 4 |
| 898 | 9311901 | 217 | 1047 | 20 | 0 | 4 |
| 899 | 9311902 | 176 | 791 | 9 | 2 | 2 |
| 900 | 9311903 | 199 | 1252 | 25 | 3 | 3 |
| 901 | 9311904 | 202 | 822 | 14 | 2 | 3 |
| 902 | 9311905 | 207 | 985 | 26 | 2 | 2 |
| 903 | 9311906 | 239 | 1287 | 19 | 5 | 4 |
| 904 | 9311907 | 252 | 1425 | 27 | 3 | 3 |
| 905 | 9311908 | 272 | 1541 | 21 | 3 | 2 |
| 906 | 9311909 | 311 | 1959 | 24 | 5 | 3 |
| 907 | 9311910 | 232 | 1307 | 18 | 1 | 2 |
| 908 | 9311911 | 337 | 1856 | 39 | 2 | 5 |
| 909 | 9311912 | 224 | 1335 | 27 | 0 | 2 |
| 910 | 9311913 | 213 | 1089 | 17 | 2 | 3 |
| 911 | 9311914 | 192 | 1087 | 16 | 6 | 2 |
| 912 | 9311915 | 290 | 1259 | 25 | 4 | 3 |
| 913 | 9311916 | 254 | 1328 | 19 | 3 | 4 |
| 914 | 9311917 | 308 | 1914 | 31 | 0 | 4 |
| 915 | 9311918 | 189 | 881 | 25 | 2 | 2 |
| 916 | 9311919 | 331 | 1246 | 26 | 2 | 5 |
| 917 | 9311920 | 288 | 1288 | 34 | 4 | 5 |
| 918 | 9311921 | 247 | 1302 | 13 | 2 | 3 |
| 919 | 9311922 | 415 | 2501 | 46 | 1 | 6 |
| 920 | 9311923 | 186 | 1042 | 27 | 1 | 2 |
| 921 | 9311924 | 308 | 1492 | 25 | 2 | 2 |
| 922 | 9311925 | 315 | 1665 | 46 | 1 | 4 |
| 923 | 9311926 | 182 | 1001 | 16 | 2 | 2 |
| 924 | 9311927 | 260 | 1425 | 27 | 5 | 3 |
| 925 | 9311928 | 311 | 2209 | 33 | 0 | 2 |
| 926 | 9311929 | 209 | 979 | 15 | 3 | 2 |
| 927 | 9311930 | 247 | 1331 | 21 | 0 | 3 |
| 928 | 9311931 | 208 | 1127 | 21 | 3 | 2 |
| 929 | 9311932 | 361 | 1727 | 40 | 6 | 4 |
| 930 | 9311933 | 237 | 1043 | 22 | 0 | 2 |
| 931 | 9311934 | 196 | 1146 | 17 | 0 | 4 |
| 932 | 9311935 | 251 | 1336 | 25 | 3 | 2 |
| 933 | 9311936 | 221 | 1175 | 22 | 0 | 2 |
| 934 | 9311937 | 284 | 1105 | 26 | 7 | 4 |
| 935 | 9311938 | 197 | 1071 | 22 | 1 | 4 |
| 936 | 9311939 | 231 | 1183 | 21 | 2 | 1 |
| 937 | 9311940 | 253 | 1060 | 22 | 3 | 3 |
| 938 | 9311941 | 209 | 1173 | 14 | 2 | 2 |
| 939 | 9311942 | 225 | 937 | 27 | 6 | 1 |
| 940 | 9311943 | 249 | 1070 | 17 | 10 | 0 |
| 941 | 9311944 | 197 | 1094 | 24 | 3 | 0 |
| 942 | 9311945 | 234 | 1093 | 23 | 10 | 2 |
| 943 | 9311946 | 202 | 1043 | 26 | 6 | 0 |
| 944 | 9311947 | 302 | 1909 | 27 | 8 | 2 |
| 945 | 9311948 | 191 | 819 | 11 | 3 | 2 |
| 946 | 9311949 | 329 | 1997 | 29 | 3 | 3 |
| 947 | 9311950 | 252 | 1229 | 28 | 6 | 3 |
| 948 | 9311951 | 273 | 1632 | 31 | 3 | 4 |
| 949 | 9311952 | 242 | 1347 | 20 | 4 | 2 |
| 950 | 9311953 | 205 | 778 | 9 | 1 | 1 |
| 951 | 9311954 | 270 | 1311 | 16 | 3 | 3 |
| 952 | 9311955 | 345 | 2267 | 30 | 1 | 2 |
| 953 | 9311956 | 219 | 1047 | 25 | 8 | 2 |
| 954 | 9311957 | 216 | 999 | 22 | 2 | 2 |
| 955 | 9311958 | 374 | 2448 | 61 | 5 | 3 |
| 956 | 9311959 | 248 | 1205 | 34 | 3 | 4 |
| 957 | 9311960 | 343 | 1917 | 33 | 10 | 3 |
| 958 | 9311961 | 212 | 1145 | 11 | 3 | 2 |
| 959 | 9311962 | 229 | 1360 | 10 | 4 | 1 |
| 960 | 9311963 | 323 | 1977 | 41 | 9 | 3 |
| 961 | 9311964 | 265 | 1109 | 18 | 3 | 4 |
| 962 | 9311965 | 295 | 1392 | 38 | 3 | 4 |
| 963 | 9311966 | 433 | 2383 | 61 | 2 | 5 |
| 964 | 9311967 | 226 | 989 | 18 | 1 | 3 |
| 965 | 9311968 | 175 | 776 | 12 | 1 | 3 |
| 966 | 9311969 | 278 | 1247 | 39 | 1 | 3 |
| 967 | 9311970 | 193 | 851 | 18 | 4 | 4 |
| 968 | 9311971 | 240 | 1135 | 23 | 2 | 2 |
| 969 | 9311972 | 316 | 1597 | 37 | 7 | 2 |
| 970 | 9311973 | 231 | 1188 | 22 | 3 | 1 |
| 971 | 9311974 | 185 | 805 | 16 | 3 | 4 |
| 972 | 9311975 | 219 | 1009 | 18 | 6 | 4 |
| 973 | 9311976 | 261 | 1424 | 37 | 4 | 1 |
| 974 | 9311977 | 304 | 1437 | 40 | 7 | 4 |
| 975 | 9311978 | 221 | 1224 | 16 | 7 | 2 |
| 976 | 9311979 | 217 | 1257 | 14 | 1 | 3 |
| 977 | 9311980 | 189 | 939 | 9 | 4 | 1 |
| 978 | 9311981 | 275 | 1352 | 18 | 8 | 2 |
| 979 | 9311982 | 258 | 1376 | 37 | 7 | 4 |
| 980 | 9311983 | 277 | 1074 | 18 | 1 | 4 |
| 981 | 9311984 | 273 | 1351 | 20 | 7 | 2 |
| 982 | 9311985 | 252 | 1217 | 33 | 4 | 3 |
| 983 | 9311986 | 358 | 1753 | 33 | 7 | 4 |
| 984 | 9311987 | 263 | 1382 | 25 | 6 | 3 |
| 985 | 9311988 | 211 | 860 | 24 | 2 | 4 |
| 986 | 9311989 | 273 | 1470 | 29 | 1 | 1 |
| 987 | 9311990 | 187 | 1073 | 20 | 1 | 1 |
| 988 | 9311991 | 212 | 979 | 7 | 4 | 2 |
| 989 | 9311992 | 218 | 1195 | 16 | 3 | 1 |
| 990 | 9311993 | 214 | 1094 | 18 | 3 | 3 |
| 991 | 9311994 | 343 | 1704 | 38 | 9 | 5 |
| 992 | 9311995 | 258 | 1006 | 29 | 6 | 5 |
| 993 | 9311996 | 302 | 1741 | 22 | 3 | 6 |
| 994 | 9311997 | 333 | 1744 | 29 | 2 | 5 |
| 995 | 9311998 | 235 | 1138 | 29 | 3 | 4 |
| 996 | 9311999 | 311 | 1986 | 17 | 11 | 4 |
| 997 | 9312000 | 256 | 1601 | 23 | 5 | 2 |
| 998 | 9312001 | 235 | 1009 | 17 | 0 | 3 |
| 999 | 9312002 | 296 | 1676 | 34 | 4 | 6 |
| 1000 | 9312003 | 500 | 1872 | 30 | 7 | 5 |
| 1001 | 9312004 | 247 | 1287 | 24 | 3 | 4 |
| 1002 | 9312005 | 266 | 1272 | 19 | 1 | 6 |
| 1003 | 9312006 | 246 | 1041 | 16 | 2 | 3 |
| 1004 | 9312007 | 357 | 1763 | 50 | 2 | 6 |
| 1005 | 9312008 | 254 | 1733 | 28 | 2 | 2 |
| 1006 | 9312009 | 196 | 1147 | 15 | 1 | 3 |
| 1007 | 9312010 | 375 | 2203 | 39 | 1 | 2 |
| 1008 | 9312011 | 205 | 1064 | 18 | 1 | 1 |
| 1009 | 9312012 | 181 | 1023 | 20 | 1 | 3 |
| 1010 | 9312013 | 343 | 1846 | 42 | 1 | 4 |
| 1011 | 9312014 | 285 | 1301 | 30 | 1 | 6 |
| 1012 | 9312015 | 226 | 1138 | 32 | 0 | 5 |
| 1013 | 9312016 | 208 | 1287 | 26 | 4 | 1 |
| 1014 | 9312017 | 260 | 1585 | 36 | 2 | 3 |
| 1015 | 9312018 | 297 | 1774 | 35 | 3 | 3 |
| 1016 | 9312019 | 203 | 954 | 20 | 3 | 4 |
| 1017 | 9312020 | 299 | 2103 | 28 | 3 | 1 |
| 1018 | 9312021 | 250 | 1401 | 23 | 0 | 1 |
| 1019 | 9312022 | 257 | 1281 | 31 | 3 | 1 |
| 1020 | 9312023 | 165 | 535 | 10 | 0 | 1 |
| 1021 | 9312024 | 257 | 1265 | 29 | 2 | 4 |
| 1022 | 9312025 | 278 | 999 | 29 | 0 | 4 |
| 1023 | 9312026 | 223 | 1226 | 20 | 2 | 1 |
| 1024 | 9312027 | 210 | 1201 | 19 | 3 | 3 |
| 1025 | 9312028 | 209 | 1088 | 16 | 3 | 0 |
| 1026 | 9312029 | 247 | 1358 | 37 | 3 | 2 |
| 1027 | 9312030 | 213 | 1178 | 26 | 3 | 2 |
| 1028 | 9312031 | 219 | 1071 | 17 | 8 | 0 |
| 1029 | 9312032 | 262 | 1332 | 16 | 4 | 1 |
| 1030 | 9312033 | 239 | 1249 | 27 | 4 | 1 |
| 1031 | 9312034 | 242 | 1097 | 24 | 6 | 2 |
| 1032 | 9312035 | 240 | 1532 | 25 | 1 | 2 |
| 1033 | 9312036 | 267 | 1493 | 17 | 8 | 3 |
| 1034 | 9312037 | 218 | 1270 | 21 | 1 | 2 |
| 1035 | 9312038 | 256 | 1525 | 27 | 9 | 1 |
| 1036 | 9312039 | 250 | 1454 | 32 | 7 | 3 |
| 1037 | 9312040 | 256 | 1531 | 13 | 7 | 4 |
| 1038 | 9312041 | 218 | 1001 | 22 | 4 | 4 |
| 1039 | 9312042 | 239 | 1356 | 15 | 7 | 3 |
| 1040 | 9312043 | 180 | 731 | 15 | 7 | 2 |
| 1041 | 9312044 | 229 | 1391 | 21 | 4 | 1 |
| 1042 | 9312045 | 204 | 780 | 10 | 4 | 2 |
| 1043 | 9312046 | 500 | 2805 | 55 | 2 | 5 |
| 1044 | 9312047 | 248 | 1377 | 33 | 3 | 4 |
| 1045 | 9312048 | 320 | 1758 | 44 | 1 | 4 |
| 1046 | 9312049 | 307 | 1610 | 33 | 5 | 1 |
| 1047 | 9312050 | 293 | 1613 | 29 | 5 | 3 |
| 1048 | 9312051 | 187 | 709 | 24 | 0 | 2 |
| 1049 | 9312052 | 207 | 936 | 12 | 2 | 3 |
| 1050 | 9312053 | 291 | 1407 | 17 | 2 | 5 |
| 1051 | 9312054 | 211 | 879 | 11 | 1 | 4 |
| 1052 | 9312055 | 196 | 972 | 22 | 2 | 4 |
| 1053 | 9312056 | 189 | 713 | 21 | 1 | 3 |
| 1054 | 9312057 | 241 | 1295 | 19 | 10 | 2 |
| 1055 | 9312058 | 275 | 1219 | 20 | 1 | 3 |
| 1056 | 9312059 | 279 | 1407 | 43 | 5 | 4 |
| 1057 | 9312060 | 245 | 1176 | 30 | 4 | 4 |
| 1058 | 9312061 | 238 | 1239 | 11 | 9 | 2 |
| 1059 | 9312062 | 182 | 763 | 18 | 2 | 2 |
| 1060 | 9312063 | 256 | 1102 | 18 | 4 | 3 |
| 1061 | 9312064 | 313 | 1524 | 37 | 8 | 3 |
| 1062 | 9312065 | 225 | 778 | 14 | 5 | 4 |
| 1063 | 9312066 | 209 | 918 | 15 | 1 | 1 |
| 1064 | 9312067 | 333 | 1871 | 22 | 3 | 4 |
| 1065 | 9312068 | 193 | 960 | 17 | 6 | 0 |
| 1066 | 9312069 | 240 | 1488 | 14 | 8 | 4 |
| 1067 | 9312070 | 223 | 986 | 15 | 4 | 1 |
| 1068 | 9312071 | 272 | 1279 | 32 | 1 | 4 |
| 1069 | 9312072 | 237 | 876 | 27 | 2 | 4 |
| 1070 | 9312073 | 226 | 906 | 10 | 3 | 2 |
| 1071 | 9312074 | 220 | 985 | 19 | 8 | 2 |
| 1072 | 9312075 | 228 | 983 | 18 | 2 | 2 |
| 1073 | 9312076 | 244 | 1378 | 30 | 7 | 4 |
| 1074 | 9312077 | 368 | 2207 | 32 | 5 | 3 |
| 1075 | 9312078 | 229 | 1254 | 19 | 3 | 2 |
| 1076 | 9312079 | 201 | 819 | 21 | 2 | 2 |
| 1077 | 9312080 | 229 | 1117 | 18 | 4 | 2 |
| 1078 | 9312081 | 209 | 920 | 14 | 1 | 2 |
| 1079 | 9312082 | 307 | 1641 | 19 | 4 | 2 |
| 1080 | 9312083 | 229 | 1435 | 20 | 3 | 1 |
| 1081 | 9312084 | 303 | 1603 | 35 | 8 | 3 |
| 1082 | 9312085 | 290 | 1637 | 28 | 6 | 3 |
| 1083 | 9312086 | 210 | 1053 | 16 | 2 | 2 |
| 1084 | 9312087 | 267 | 1364 | 22 | 1 | 2 |
| 1085 | 9312088 | 215 | 824 | 18 | 6 | 2 |
| 1086 | 9312089 | 233 | 1342 | 26 | 2 | 3 |
| 1087 | 9312090 | 210 | 1408 | 19 | 5 | 3 |
| 1088 | 9312091 | 333 | 1527 | 28 | 2 | 2 |
| 1089 | 9312092 | 210 | 1259 | 28 | 2 | 3 |
| 1090 | 9312093 | 247 | 1393 | 30 | 2 | 3 |
| 1091 | 9312094 | 362 | 2481 | 35 | 4 | 3 |
| 1092 | 9312095 | 414 | 2153 | 29 | 4 | 5 |
| 1093 | 9312096 | 265 | 1459 | 14 | 0 | 3 |
| 1094 | 9312097 | 209 | 1217 | 23 | 2 | 1 |
| 1095 | 9312098 | 304 | 1701 | 35 | 4 | 4 |
| 1096 | 9312099 | 246 | 1252 | 25 | 6 | 2 |
| 1097 | 9312100 | 278 | 1637 | 29 | 0 | 2 |
| 1098 | 9312101 | 278 | 1707 | 28 | 1 | 1 |
| 1099 | 9312102 | 210 | 1253 | 27 | 1 | 3 |
| 1100 | 9312103 | 166 | 864 | 17 | 2 | 2 |
| 1101 | 9312104 | 403 | 2041 | 54 | 3 | 3 |
| 1102 | 9312105 | 321 | 1808 | 38 | 3 | 5 |
| 1103 | 9312106 | 271 | 1684 | 27 | 9 | 2 |
| 1104 | 9312107 | 255 | 1436 | 47 | 2 | 2 |
| 1105 | 9312108 | 219 | 1197 | 24 | 0 | 2 |
| 1106 | 9312109 | 215 | 747 | 11 | 3 | 2 |
| 1107 | 9312110 | 285 | 1536 | 15 | 1 | 2 |
| 1108 | 9312111 | 299 | 1369 | 27 | 0 | 4 |
| 1109 | 9312112 | 238 | 768 | 10 | 4 | 2 |
| 1110 | 9312113 | 234 | 1147 | 27 | 0 | 1 |
| 1111 | 9312114 | 288 | 1257 | 38 | 3 | 4 |
| 1112 | 9312115 | 215 | 800 | 14 | 3 | 2 |
| 1113 | 9312116 | 158 | 776 | 16 | 2 | 2 |
| 1114 | 9312117 | 213 | 1206 | 25 | 5 | 2 |
| 1115 | 9312118 | 207 | 1328 | 35 | 1 | 3 |
| 1116 | 9312119 | 240 | 1322 | 40 | 2 | 1 |
| 1117 | 9312120 | 233 | 1062 | 20 | 3 | 1 |
| 1118 | 9312121 | 261 | 1485 | 34 | 6 | 1 |
| 1119 | 9312122 | 281 | 1367 | 32 | 10 | 3 |
| 1120 | 9312123 | 226 | 1285 | 14 | 3 | 2 |
| 1121 | 9312124 | 282 | 1469 | 42 | 3 | 4 |
| 1122 | 9312125 | 280 | 1578 | 25 | 5 | 2 |
| 1123 | 9312126 | 261 | 1631 | 31 | 2 | 1 |
| 1124 | 9312127 | 216 | 1330 | 20 | 0 | 1 |
| 1125 | 9312128 | 282 | 1825 | 35 | 4 | 4 |
| 1126 | 9312129 | 224 | 1239 | 31 | 2 | 4 |
| 1127 | 9312130 | 244 | 1728 | 36 | 9 | 3 |
| 1128 | 9312131 | 185 | 868 | 13 | 3 | 2 |
| 1129 | 9312132 | 255 | 1000 | 29 | 13 | 4 |
| 1130 | 9312133 | 272 | 1129 | 22 | 1 | 2 |
| 1131 | 9312134 | 264 | 1396 | 36 | 3 | 2 |
| 1132 | 9312135 | 276 | 1580 | 27 | 4 | 2 |
| 1133 | 9312136 | 264 | 1642 | 34 | 2 | 2 |
| 1134 | 9312137 | 318 | 1872 | 38 | 8 | 3 |
| 1135 | 9312138 | 231 | 1484 | 23 | 0 | 1 |
| 1136 | 9312139 | 274 | 1192 | 26 | 2 | 4 |
| 1137 | 9312140 | 285 | 1672 | 26 | 1 | 3 |
| 1138 | 9312141 | 244 | 1107 | 28 | 3 | 4 |
| 1139 | 9312142 | 334 | 1573 | 33 | 5 | 6 |
| 1140 | 9312143 | 343 | 1640 | 24 | 6 | 3 |
| 1141 | 9312144 | 277 | 1305 | 20 | 2 | 4 |
| 1142 | 9312145 | 198 | 880 | 9 | 4 | 3 |
| 1143 | 9312146 | 348 | 1190 | 24 | 3 | 3 |
| 1144 | 9312147 | 231 | 1136 | 22 | 4 | 2 |
| 1145 | 9312148 | 230 | 1055 | 16 | 2 | 3 |
| 1146 | 9312149 | 248 | 1298 | 11 | 4 | 5 |
| 1147 | 9312150 | 218 | 1108 | 21 | 2 | 2 |
| 1148 | 9312151 | 270 | 1325 | 14 | 5 | 1 |
| 1149 | 9312152 | 215 | 1124 | 18 | 3 | 3 |
| 1150 | 9312153 | 223 | 1136 | 16 | 1 | 3 |
| 1151 | 9312154 | 258 | 1200 | 23 | 7 | 2 |
| 1152 | 9312155 | 204 | 1106 | 22 | 4 | 1 |
| 1153 | 9312156 | 229 | 1018 | 22 | 8 | 2 |
| 1154 | 9312157 | 262 | 1228 | 12 | 13 | 4 |
| 1155 | 9312158 | 217 | 1029 | 17 | 3 | 1 |
| 1156 | 9312159 | 268 | 1457 | 26 | 6 | 2 |
| 1157 | 9312160 | 231 | 1076 | 20 | 4 | 5 |
| 1158 | 9312161 | 248 | 1206 | 29 | 6 | 2 |
| 1159 | 9312162 | 253 | 1521 | 30 | 2 | 2 |
| 1160 | 9312163 | 169 | 751 | 7 | 4 | 0 |
| 1161 | 9312164 | 224 | 1216 | 25 | 2 | 3 |
| 1162 | 9312165 | 172 | 643 | 14 | 3 | 2 |
| 1163 | 9312166 | 183 | 687 | 17 | 2 | 3 |
| 1164 | 9312167 | 266 | 1403 | 32 | 5 | 4 |
| 1165 | 9312168 | 238 | 735 | 15 | 3 | 3 |
| 1166 | 9312169 | 275 | 1466 | 36 | 4 | 2 |
| 1167 | 9312170 | 203 | 869 | 10 | 4 | 2 |
| 1168 | 9312171 | 435 | 2672 | 38 | 5 | 4 |
| 1169 | 9312172 | 227 | 1150 | 27 | 8 | 3 |
| 1170 | 9312173 | 290 | 1453 | 33 | 1 | 4 |
| 1171 | 9312174 | 347 | 2192 | 51 | 6 | 3 |
| 1172 | 9312175 | 212 | 1117 | 24 | 0 | 2 |
| 1173 | 9312176 | 366 | 1999 | 30 | 2 | 4 |
| 1174 | 9312177 | 311 | 1435 | 39 | 7 | 5 |
| 1175 | 9312178 | 187 | 1135 | 19 | 1 | 2 |
| 1176 | 9312179 | 224 | 1146 | 28 | 6 | 2 |
| 1177 | 9312180 | 270 | 1468 | 11 | 4 | 3 |
| 1178 | 9312181 | 191 | 886 | 20 | 6 | 1 |
| 1179 | 9312182 | 231 | 1299 | 31 | 3 | 3 |
| 1180 | 9312183 | 353 | 2088 | 30 | 4 | 2 |
| 1181 | 9312184 | 227 | 1002 | 23 | 3 | 3 |
| 1182 | 9312185 | 206 | 1173 | 22 | 5 | 4 |
| 1183 | 9312186 | 198 | 972 | 9 | 1 | 2 |
| 1184 | 9312187 | 339 | 1566 | 27 | 1 | 3 |
| 1185 | 9312188 | 256 | 1362 | 27 | 1 | 3 |
| 1186 | 9312189 | 248 | 1292 | 25 | 5 | 1 |
| 1187 | 9312190 | 312 | 1554 | 28 | 3 | 1 |
| 1188 | 9312191 | 292 | 1839 | 26 | 1 | 2 |
| 1189 | 9312192 | 233 | 1191 | 31 | 4 | 5 |
| 1190 | 9312193 | 210 | 747 | 21 | 1 | 4 |
| 1191 | 9312194 | 307 | 1406 | 23 | 2 | 2 |
| 1192 | 9312195 | 219 | 1253 | 24 | 3 | 3 |
| 1193 | 9312196 | 267 | 1813 | 20 | 3 | 3 |
| 1194 | 9312197 | 262 | 1434 | 20 | 2 | 1 |
| 1195 | 9312198 | 188 | 851 | 23 | 3 | 2 |
| 1196 | 9312199 | 237 | 1153 | 27 | 3 | 4 |
| 1197 | 9312200 | 156 | 750 | 10 | 0 | 2 |
| 1198 | 9312201 | 281 | 1652 | 34 | 1 | 5 |
| 1199 | 9312202 | 263 | 1543 | 37 | 1 | 1 |
| 1200 | 9312203 | 224 | 1055 | 31 | 9 | 3 |
| 1201 | 9312204 | 228 | 970 | 30 | 6 | 4 |
| 1202 | 9312205 | 174 | 749 | 18 | 3 | 2 |
| 1203 | 9312206 | 183 | 882 | 21 | 1 | 1 |
| 1204 | 9312207 | 319 | 2457 | 27 | 2 | 2 |
| 1205 | 9312208 | 242 | 1167 | 29 | 2 | 2 |
| 1206 | 9312209 | 191 | 909 | 14 | 1 | 1 |
| 1207 | 9312210 | 216 | 996 | 33 | 3 | 3 |
| 1208 | 9312211 | 232 | 1481 | 21 | 3 | 2 |
| 1209 | 9312212 | 395 | 2490 | 60 | 6 | 3 |
| 1210 | 9312213 | 322 | 1856 | 41 | 1 | 3 |
| 1211 | 9312214 | 166 | 880 | 19 | 2 | 3 |
| 1212 | 9312215 | 232 | 1348 | 24 | 3 | 0 |
| 1213 | 9312216 | 221 | 1354 | 19 | 2 | 3 |
| 1214 | 9312217 | 248 | 1260 | 22 | 4 | 3 |
| 1215 | 9312218 | 226 | 1000 | 15 | 2 | 1 |
| 1216 | 9312219 | 219 | 1168 | 23 | 3 | 3 |
| 1217 | 9312220 | 271 | 1538 | 35 | 1 | 3 |
| 1218 | 9312221 | 296 | 1677 | 30 | 2 | 2 |
| 1219 | 9312222 | 217 | 1061 | 22 | 3 | 2 |
| 1220 | 9312223 | 201 | 640 | 14 | 1 | 3 |
| 1221 | 9312224 | 201 | 684 | 19 | 3 | 0 |
| 1222 | 9312225 | 252 | 1115 | 26 | 2 | 4 |
| 1223 | 9312226 | 236 | 1149 | 20 | 5 | 5 |
| 1224 | 9312227 | 240 | 1262 | 24 | 8 | 4 |
| 1225 | 9312228 | 291 | 935 | 24 | 1 | 5 |
| 1226 | 9312229 | 190 | 875 | 13 | 1 | 2 |
| 1227 | 9312230 | 403 | 1865 | 51 | 8 | 4 |
| 1228 | 9312231 | 299 | 1338 | 29 | 4 | 5 |
| 1229 | 9312232 | 250 | 926 | 15 | 7 | 3 |
| 1230 | 9312233 | 251 | 1206 | 23 | 1 | 3 |
| 1231 | 9312234 | 240 | 982 | 21 | 1 | 5 |
| 1232 | 9312235 | 398 | 2010 | 46 | 11 | 4 |
| 1233 | 9312236 | 210 | 777 | 13 | 4 | 1 |
| 1234 | 9312237 | 261 | 1330 | 22 | 2 | 4 |
| 1235 | 9312238 | 237 | 1049 | 32 | 2 | 4 |
| 1236 | 9312239 | 229 | 1255 | 7 | 5 | 3 |
| 1237 | 9312240 | 331 | 1872 | 20 | 1 | 5 |
| 1238 | 9312241 | 193 | 775 | 17 | 2 | 4 |
| 1239 | 9312242 | 425 | 2752 | 52 | 4 | 5 |
| 1240 | 9312243 | 290 | 1335 | 26 | 4 | 4 |
| 1241 | 9312244 | 170 | 619 | 12 | 3 | 2 |
| 1242 | 9312245 | 263 | 1420 | 36 | 5 | 4 |
| 1243 | 9312246 | 178 | 803 | 16 | 3 | 1 |
| 1244 | 9312247 | 219 | 979 | 17 | 2 | 3 |
| 1245 | 9312248 | 294 | 1347 | 23 | 1 | 4 |
| 1246 | 9312249 | 218 | 1154 | 19 | 5 | 4 |
| 1247 | 9312250 | 252 | 1329 | 28 | 7 | 2 |
| 1248 | 9312251 | 158 | 543 | 8 | 8 | 2 |
| 1249 | 9312252 | 283 | 1909 | 20 | 1 | 2 |
| 1250 | 9312253 | 282 | 1425 | 24 | 3 | 2 |
| 1251 | 9312254 | 280 | 1628 | 29 | 5 | 3 |
| 1252 | 9312255 | 183 | 753 | 15 | 2 | 2 |
| 1253 | 9312256 | 235 | 1123 | 20 | 0 | 2 |
| 1254 | 9312257 | 223 | 1291 | 15 | 1 | 2 |
| 1255 | 9312258 | 258 | 1391 | 15 | 4 | 3 |
| 1256 | 9312259 | 233 | 1565 | 20 | 3 | 2 |
| 1257 | 9312260 | 224 | 584 | 18 | 2 | 5 |
| 1258 | 9312261 | 251 | 1461 | 31 | 1 | 4 |
| 1259 | 9312262 | 221 | 1281 | 17 | 4 | 4 |
| 1260 | 9312263 | 281 | 1409 | 31 | 6 | 3 |
| 1261 | 9312264 | 229 | 1283 | 25 | 2 | 3 |
| 1262 | 9312265 | 294 | 1461 | 22 | 3 | 3 |
| 1263 | 9312266 | 240 | 1405 | 18 | 3 | 3 |
| 1264 | 9312267 | 216 | 1298 | 20 | 3 | 1 |
| 1265 | 9312268 | 484 | 2440 | 62 | 2 | 4 |
| 1266 | 9312269 | 310 | 1649 | 26 | 5 | 4 |
| 1267 | 9312270 | 264 | 1167 | 27 | 2 | 4 |
| 1268 | 9312271 | 275 | 1722 | 25 | 1 | 2 |
| 1269 | 9312272 | 209 | 1152 | 23 | 0 | 3 |
| 1270 | 9312273 | 248 | 1313 | 18 | 5 | 4 |
| 1271 | 9312274 | 203 | 1229 | 18 | 0 | 3 |
| 1272 | 9312275 | 217 | 1310 | 21 | 2 | 2 |
| 1273 | 9312276 | 251 | 1363 | 24 | 5 | 2 |
| 1274 | 9312277 | 388 | 2092 | 45 | 3 | 4 |
| 1275 | 9312278 | 211 | 1180 | 22 | 1 | 1 |
| 1276 | 9312279 | 289 | 1784 | 34 | 2 | 2 |
| 1277 | 9312280 | 252 | 1417 | 27 | 1 | 3 |
| 1278 | 9312281 | 238 | 1374 | 26 | 4 | 3 |
| 1279 | 9312282 | 223 | 1133 | 27 | 1 | 4 |
| 1280 | 9312283 | 210 | 917 | 16 | 0 | 2 |
| 1281 | 9312284 | 232 | 1314 | 37 | 7 | 2 |
| 1282 | 9312285 | 349 | 1954 | 29 | 12 | 4 |
| 1283 | 9312286 | 229 | 1165 | 29 | 4 | 2 |
| 1284 | 9312287 | 238 | 1342 | 29 | 2 | 1 |
| 1285 | 9312288 | 230 | 1177 | 23 | 3 | 2 |
| 1286 | 9312289 | 206 | 831 | 16 | 0 | 3 |
| 1287 | 9312290 | 268 | 1264 | 28 | 2 | 3 |
| 1288 | 9312291 | 183 | 872 | 11 | 5 | 1 |
| 1289 | 9312292 | 204 | 1156 | 21 | 3 | 1 |
| 1290 | 9312293 | 264 | 1451 | 30 | 4 | 4 |
| 1291 | 9312294 | 299 | 1671 | 38 | 4 | 3 |
| 1292 | 9312295 | 261 | 1492 | 23 | 8 | 4 |
| 1293 | 9312296 | 192 | 966 | 14 | 4 | 0 |
| 1294 | 9312297 | 270 | 1422 | 25 | 4 | 0 |
| 1295 | 9312298 | 266 | 1512 | 27 | 3 | 2 |
| 1296 | 9312299 | 245 | 1115 | 14 | 2 | 4 |
| 1297 | 9312300 | 208 | 1021 | 28 | 3 | 2 |
| 1298 | 9312301 | 254 | 1454 | 27 | 5 | 1 |
| 1299 | 9312302 | 186 | 1100 | 17 | 5 | 1 |
| 1300 | 9312303 | 200 | 971 | 10 | 4 | 3 |
| 1301 | 9312304 | 216 | 1037 | 29 | 4 | 3 |
| 1302 | 9312305 | 218 | 1285 | 27 | 3 | 2 |
| 1303 | 9312306 | 227 | 1370 | 25 | 3 | 2 |
| 1304 | 9312307 | 202 | 1006 | 21 | 4 | 1 |
| 1305 | 9312308 | 201 | 1116 | 20 | 1 | 3 |
| 1306 | 9312309 | 264 | 1242 | 25 | 1 | 4 |
| 1307 | 9312310 | 255 | 1390 | 32 | 8 | 4 |
| 1308 | 9312311 | 500 | 2192 | 48 | 9 | 4 |
| 1309 | 9312312 | 283 | 1954 | 38 | 3 | 3 |
| 1310 | 9312313 | 256 | 1377 | 18 | 1 | 3 |
| 1311 | 9312314 | 330 | 1861 | 42 | 2 | 4 |
| 1312 | 9312315 | 267 | 1166 | 26 | 6 | 3 |
| 1313 | 9312316 | 203 | 972 | 13 | 4 | 3 |
| 1314 | 9312317 | 230 | 1124 | 17 | 2 | 5 |
| 1315 | 9312318 | 218 | 978 | 20 | 3 | 5 |
| 1316 | 9312319 | 291 | 1330 | 24 | 1 | 5 |
| 1317 | 9312320 | 215 | 1065 | 11 | 5 | 4 |
| 1318 | 9312321 | 227 | 1370 | 28 | 1 | 3 |
| 1319 | 9312322 | 206 | 1159 | 21 | 7 | 3 |
| 1320 | 9312323 | 220 | 1015 | 9 | 2 | 2 |
| 1321 | 9312324 | 193 | 942 | 19 | 6 | 3 |
| 1322 | 9312325 | 271 | 1315 | 24 | 5 | 5 |
| 1323 | 9312326 | 208 | 1102 | 23 | 5 | 2 |
| 1324 | 9312327 | 216 | 683 | 20 | 3 | 3 |
| 1325 | 9312328 | 223 | 1030 | 27 | 4 | 3 |
| 1326 | 9312329 | 250 | 1039 | 14 | 4 | 6 |
| 1327 | 9312330 | 246 | 1186 | 19 | 7 | 2 |
| 1328 | 9312331 | 234 | 1172 | 25 | 7 | 2 |
| 1329 | 9312332 | 217 | 880 | 19 | 4 | 3 |
| 1330 | 9312333 | 256 | 1391 | 33 | 5 | 3 |
| 1331 | 9312334 | 269 | 1249 | 18 | 3 | 5 |
| 1332 | 9312335 | 247 | 1169 | 40 | 5 | 1 |
| 1333 | 9312336 | 226 | 1050 | 29 | 2 | 4 |
| 1334 | 9312337 | 255 | 1290 | 11 | 6 | 4 |
| 1335 | 9312338 | 248 | 1484 | 33 | 2 | 2 |
| 1336 | 9312339 | 218 | 1049 | 14 | 3 | 4 |
| 1337 | 9312340 | 247 | 1426 | 24 | 1 | 2 |
| 1338 | 9312341 | 258 | 1650 | 21 | 4 | 3 |
| 1339 | 9312342 | 206 | 929 | 14 | 7 | 4 |
| 1340 | 9312343 | 487 | 2806 | 49 | 2 | 5 |
| 1341 | 9312344 | 335 | 1403 | 42 | 5 | 6 |
| 1342 | 9312345 | 222 | 929 | 18 | 3 | 4 |
| 1343 | 9312346 | 229 | 1064 | 23 | 7 | 4 |
| 1344 | 9312347 | 180 | 578 | 9 | 3 | 4 |
| 1345 | 9312348 | 195 | 1112 | 16 | 2 | 2 |
| 1346 | 9312349 | 281 | 1442 | 16 | 3 | 5 |
| 1347 | 9312350 | 190 | 1060 | 18 | 1 | 4 |
| 1348 | 9312351 | 278 | 1687 | 30 | 3 | 4 |
| 1349 | 9312352 | 284 | 1844 | 37 | 5 | 2 |
| 1350 | 9312353 | 297 | 1451 | 19 | 1 | 5 |
| 1351 | 9312354 | 310 | 1701 | 18 | 2 | 3 |
| 1352 | 9312355 | 231 | 1317 | 20 | 6 | 3 |
| 1353 | 9312356 | 255 | 1435 | 13 | 4 | 2 |
| 1354 | 9312357 | 252 | 1897 | 19 | 5 | 4 |
| 1355 | 9312358 | 237 | 1185 | 20 | 4 | 5 |
| 1356 | 9312359 | 303 | 1305 | 16 | 1 | 2 |
| 1357 | 9312360 | 279 | 1604 | 25 | 2 | 3 |
| 1358 | 9312361 | 213 | 1012 | 23 | 0 | 5 |
| 1359 | 9312362 | 300 | 1678 | 28 | 2 | 6 |
| 1360 | 9312363 | 287 | 1726 | 47 | 5 | 4 |
| 1361 | 9312364 | 288 | 1081 | 15 | 1 | 2 |
| 1362 | 9312365 | 214 | 1181 | 20 | 4 | 4 |
| 1363 | 9312366 | 264 | 1225 | 21 | 3 | 2 |
| 1364 | 9312367 | 266 | 1455 | 33 | 5 | 4 |
| 1365 | 9312368 | 240 | 1278 | 29 | 3 | 3 |
| 1366 | 9312369 | 233 | 1064 | 33 | 0 | 2 |
| 1367 | 9312370 | 281 | 1518 | 22 | 5 | 5 |
| 1368 | 9312371 | 240 | 1170 | 29 | 3 | 4 |
| 1369 | 9312372 | 273 | 1546 | 35 | 2 | 2 |
| 1370 | 9312373 | 338 | 2074 | 39 | 6 | 4 |
| 1371 | 9312374 | 230 | 1107 | 22 | 3 | 2 |
| 1372 | 9312375 | 240 | 1055 | 29 | 1 | 2 |
| 1373 | 9312376 | 238 | 1229 | 17 | 4 | 1 |
| 1374 | 9312377 | 329 | 1798 | 36 | 2 | 5 |
| 1375 | 9312378 | 347 | 1686 | 37 | 4 | 4 |
| 1376 | 9312379 | 276 | 1504 | 28 | 4 | 5 |
| 1377 | 9312380 | 273 | 1349 | 28 | 3 | 2 |
| 1378 | 9312381 | 221 | 1185 | 23 | 3 | 2 |
| 1379 | 9312382 | 242 | 1058 | 26 | 5 | 5 |
| 1380 | 9312383 | 262 | 1518 | 27 | 9 | 0 |
| 1381 | 9312384 | 229 | 1086 | 26 | 3 | 0 |
| 1382 | 9312385 | 252 | 1196 | 32 | 4 | 3 |
| 1383 | 9312386 | 259 | 1432 | 24 | 5 | 2 |
| 1384 | 9312387 | 173 | 735 | 17 | 3 | 1 |
| 1385 | 9312388 | 217 | 1171 | 22 | 5 | 2 |
| 1386 | 9312389 | 343 | 2172 | 38 | 3 | 3 |
| 1387 | 9312390 | 220 | 1320 | 12 | 5 | 1 |
| 1388 | 9312391 | 272 | 1599 | 16 | 1 | 2 |
| 1389 | 9312392 | 179 | 888 | 13 | 3 | 3 |
| 1390 | 9312393 | 270 | 1265 | 36 | 1 | 3 |
| 1391 | 9312394 | 242 | 1495 | 18 | 5 | 5 |
| 1392 | 9312395 | 256 | 1342 | 32 | 4 | 4 |
| 1393 | 9312396 | 228 | 1154 | 25 | 5 | 1 |
| 1394 | 9312397 | 357 | 2190 | 31 | 3 | 2 |
| 1395 | 9312398 | 249 | 1151 | 29 | 6 | 4 |
| 1396 | 9312399 | 304 | 1384 | 42 | 7 | 4 |
| 1397 | 9312400 | 307 | 1595 | 40 | 2 | 2 |
| 1398 | 9312401 | 346 | 1769 | 25 | 4 | 1 |
| 1399 | 9312402 | 240 | 1326 | 27 | 4 | 4 |
| 1400 | 9312403 | 450 | 2092 | 48 | 2 | 4 |
| 1401 | 9312404 | 258 | 1058 | 20 | 3 | 2 |
| 1402 | 9312405 | 203 | 917 | 18 | 1 | 2 |
| 1403 | 9312406 | 223 | 997 | 23 | 1 | 2 |
| 1404 | 9312407 | 267 | 1618 | 38 | 3 | 3 |
| 1405 | 9312408 | 252 | 1022 | 19 | 7 | 3 |
| 1406 | 9312409 | 254 | 1032 | 28 | 0 | 4 |
| 1407 | 9312410 | 228 | 1172 | 25 | 3 | 2 |
| 1408 | 9312411 | 251 | 1135 | 20 | 5 | 1 |
| 1409 | 9312412 | 299 | 1737 | 26 | 4 | 2 |
| 1410 | 9312413 | 255 | 1254 | 14 | 8 | 2 |
| 1411 | 9312414 | 281 | 1301 | 22 | 11 | 5 |
| 1412 | 9312415 | 237 | 1135 | 24 | 3 | 5 |
| 1413 | 9312416 | 281 | 1514 | 27 | 8 | 0 |
| 1414 | 9312417 | 217 | 863 | 16 | 1 | 2 |
| 1415 | 9312418 | 270 | 1237 | 38 | 3 | 4 |
| 1416 | 9312419 | 267 | 1186 | 30 | 8 | 2 |
| 1417 | 9312420 | 239 | 1024 | 26 | 11 | 3 |
| 1418 | 9312421 | 282 | 1758 | 24 | 11 | 4 |
| 1419 | 9312422 | 262 | 1335 | 23 | 9 | 4 |
| 1420 | 9312423 | 184 | 1043 | 26 | 3 | 2 |
| 1421 | 9312424 | 307 | 1535 | 41 | 1 | 3 |
| 1422 | 9312425 | 233 | 1134 | 19 | 3 | 3 |
| 1423 | 9312426 | 304 | 1428 | 26 | 1 | 5 |
| 1424 | 9312427 | 280 | 1650 | 33 | 1 | 4 |
| 1425 | 9312428 | 292 | 1463 | 20 | 3 | 0 |
| 1426 | 9312429 | 318 | 1340 | 17 | 4 | 4 |
| 1427 | 9312430 | 222 | 889 | 26 | 2 | 3 |
| 1428 | 9312431 | 227 | 1547 | 13 | 6 | 4 |
| 1429 | 9312432 | 204 | 849 | 19 | 5 | 4 |
| 1430 | 9312433 | 281 | 1202 | 26 | 4 | 5 |
| 1431 | 9312434 | 262 | 1271 | 30 | 10 | 3 |
| 1432 | 9312435 | 253 | 1084 | 19 | 1 | 3 |
| 1433 | 9312436 | 251 | 1788 | 26 | 7 | 2 |
| 1434 | 9312437 | 281 | 1854 | 35 | 1 | 2 |
| 1435 | 9312438 | 396 | 2384 | 41 | 6 | 4 |
| 1436 | 9312439 | 277 | 1403 | 24 | 3 | 5 |
| 1437 | 9312440 | 195 | 1024 | 25 | 6 | 2 |
| 1438 | 9312441 | 293 | 1482 | 35 | 1 | 3 |
| 1439 | 9312442 | 218 | 1331 | 16 | 1 | 3 |
| 1440 | 9312443 | 217 | 1294 | 17 | 2 | 1 |
| 1441 | 9312444 | 250 | 1044 | 13 | 2 | 2 |
| 1442 | 9312445 | 257 | 1623 | 30 | 1 | 3 |
| 1443 | 9312446 | 295 | 1612 | 24 | 4 | 4 |
| 1444 | 9312447 | 263 | 1473 | 16 | 5 | 4 |
| 1445 | 9312448 | 274 | 1238 | 32 | 4 | 6 |
| 1446 | 9312449 | 364 | 1973 | 32 | 4 | 2 |
| 1447 | 9312450 | 182 | 800 | 20 | 4 | 2 |
| 1448 | 9312451 | 306 | 1618 | 35 | 1 | 4 |
| 1449 | 9312452 | 202 | 1086 | 26 | 5 | 2 |
| 1450 | 9312453 | 253 | 1441 | 32 | 0 | 4 |
| 1451 | 9312454 | 248 | 1261 | 36 | 1 | 2 |
| 1452 | 9312455 | 223 | 1311 | 23 | 0 | 3 |
| 1453 | 9312456 | 304 | 2089 | 41 | 5 | 3 |
| 1454 | 9312457 | 298 | 1612 | 41 | 3 | 4 |
| 1455 | 9312458 | 342 | 1520 | 38 | 2 | 3 |
| 1456 | 9312459 | 316 | 1517 | 34 | 3 | 3 |
| 1457 | 9312460 | 250 | 1588 | 22 | 3 | 4 |
| 1458 | 9312461 | 219 | 1203 | 20 | 4 | 1 |
| 1459 | 9312462 | 168 | 710 | 11 | 2 | 2 |
| 1460 | 9312463 | 200 | 939 | 18 | 3 | 1 |
| 1461 | 9312464 | 257 | 1525 | 33 | 1 | 2 |
| 1462 | 9312465 | 287 | 1902 | 33 | 1 | 2 |
| 1463 | 9312466 | 238 | 1087 | 33 | 1 | 1 |
| 1464 | 9312467 | 500 | 2775 | 61 | 2 | 4 |
| 1465 | 9312468 | 266 | 1453 | 29 | 11 | 3 |
| 1466 | 9312469 | 222 | 1294 | 26 | 3 | 1 |
| 1467 | 9312470 | 249 | 1407 | 23 | 4 | 3 |
| 1468 | 9312471 | 335 | 2115 | 37 | 7 | 2 |
| 1469 | 9312472 | 266 | 1370 | 34 | 2 | 5 |
| 1470 | 9312473 | 253 | 1388 | 34 | 4 | 4 |
| 1471 | 9312474 | 295 | 1681 | 28 | 4 | 2 |
| 1472 | 9312475 | 272 | 1615 | 27 | 4 | 3 |
| 1473 | 9312476 | 207 | 1030 | 16 | 4 | 3 |
| 1474 | 9312477 | 225 | 1355 | 31 | 1 | 2 |
| 1475 | 9312478 | 283 | 1814 | 24 | 8 | 2 |
| 1476 | 9312479 | 281 | 1876 | 33 | 2 | 4 |
| 1477 | 9312480 | 271 | 1743 | 30 | 1 | 1 |
| 1478 | 9312481 | 232 | 1564 | 18 | 1 | 2 |
| 1479 | 9312482 | 281 | 1731 | 24 | 5 | 4 |
| 1480 | 9312483 | 231 | 1266 | 14 | 4 | 1 |
| 1481 | 9312484 | 210 | 1201 | 16 | 8 | 2 |
| 1482 | 9312485 | 194 | 955 | 19 | 3 | 1 |
| 1483 | 9312486 | 264 | 1442 | 15 | 2 | 4 |
| 1484 | 9312487 | 331 | 1882 | 25 | 3 | 3 |
| 1485 | 9312488 | 295 | 1983 | 27 | 6 | 2 |
| 1486 | 9312489 | 310 | 1791 | 31 | 3 | 1 |
| 1487 | 9312490 | 295 | 1928 | 27 | 0 | 2 |
| 1488 | 9312491 | 301 | 1359 | 18 | 5 | 5 |
| 1489 | 9312492 | 208 | 1082 | 20 | 4 | 2 |
| 1490 | 9312493 | 210 | 1261 | 17 | 4 | 5 |
| 1491 | 9312494 | 245 | 1318 | 12 | 4 | 4 |
| 1492 | 9312495 | 252 | 1095 | 7 | 5 | 5 |
| 1493 | 9312496 | 263 | 1202 | 16 | 4 | 3 |
| 1494 | 9312497 | 262 | 1257 | 25 | 7 | 4 |
| 1495 | 9312498 | 287 | 1385 | 28 | 2 | 3 |
| 1496 | 9312499 | 237 | 958 | 21 | 5 | 2 |
| 1497 | 9312500 | 285 | 1686 | 30 | 1 | 4 |
| 1498 | 9312501 | 325 | 1686 | 39 | 4 | 4 |
| 1499 | 9312502 | 230 | 1057 | 19 | 3 | 4 |
| 1500 | 9312503 | 264 | 1507 | 24 | 3 | 4 |
| 1501 | 9312504 | 183 | 735 | 14 | 2 | 2 |
| 1502 | 9312505 | 280 | 1316 | 30 | 6 | 3 |
| 1503 | 9312506 | 239 | 1179 | 37 | 2 | 2 |
| 1504 | 9312507 | 195 | 905 | 16 | 3 | 2 |
| 1505 | 9312508 | 211 | 1103 | 10 | 3 | 2 |
| 1506 | 9312509 | 235 | 1068 | 23 | 4 | 3 |
| 1507 | 9312510 | 216 | 1009 | 14 | 6 | 2 |
| 1508 | 9312511 | 263 | 1513 | 25 | 6 | 1 |
| 1509 | 9312512 | 235 | 1232 | 18 | 2 | 1 |
| 1510 | 9312513 | 275 | 1204 | 36 | 5 | 5 |
| 1511 | 9312514 | 288 | 1735 | 25 | 7 | 4 |
| 1512 | 9312515 | 216 | 989 | 19 | 3 | 2 |
| 1513 | 9312516 | 272 | 1713 | 32 | 1 | 4 |
| 1514 | 9312517 | 223 | 1222 | 19 | 9 | 2 |
| 1515 | 9312518 | 224 | 1206 | 27 | 0 | 3 |
| 1516 | 9312519 | 263 | 1588 | 18 | 5 | 3 |
| 1517 | 9312520 | 194 | 967 | 17 | 7 | 3 |
| 1518 | 9312521 | 313 | 1439 | 28 | 8 | 4 |
| 1519 | 9312522 | 206 | 1020 | 17 | 2 | 3 |
| 1520 | 9312523 | 302 | 1828 | 27 | 3 | 3 |
| 1521 | 9312524 | 189 | 933 | 15 | 1 | 2 |
| 1522 | 9312525 | 293 | 1304 | 24 | 3 | 4 |
| 1523 | 9312526 | 256 | 1633 | 26 | 0 | 1 |
| 1524 | 9312527 | 304 | 1611 | 22 | 2 | 1 |
| 1525 | 9312528 | 222 | 1122 | 19 | 1 | 5 |
| 1526 | 9312529 | 241 | 1492 | 9 | 4 | 4 |
| 1527 | 9312530 | 383 | 2635 | 31 | 3 | 2 |
| 1528 | 9312531 | 228 | 939 | 27 | 3 | 2 |
| 1529 | 9312532 | 381 | 2696 | 26 | 5 | 4 |
| 1530 | 9312533 | 269 | 1308 | 15 | 4 | 4 |
| 1531 | 9312534 | 249 | 1331 | 28 | 3 | 3 |
| 1532 | 9312535 | 214 | 969 | 23 | 2 | 4 |
| 1533 | 9312536 | 346 | 1945 | 26 | 2 | 5 |
| 1534 | 9312537 | 182 | 816 | 6 | 0 | 2 |
| 1535 | 9312538 | 283 | 1182 | 19 | 4 | 5 |
| 1536 | 9312539 | 278 | 1351 | 28 | 1 | 3 |
| 1537 | 9312540 | 318 | 1277 | 20 | 2 | 3 |
| 1538 | 9312541 | 282 | 1393 | 23 | 0 | 3 |
| 1539 | 9312542 | 253 | 978 | 15 | 0 | 4 |
| 1540 | 9312543 | 270 | 1384 | 24 | 1 | 2 |
| 1541 | 9312544 | 500 | 3007 | 47 | 5 | 4 |
| 1542 | 9312545 | 342 | 1747 | 24 | 1 | 4 |
| 1543 | 9312546 | 266 | 1453 | 31 | 3 | 4 |
| 1544 | 9312547 | 217 | 1215 | 23 | 4 | 4 |
| 1545 | 9312548 | 207 | 1041 | 20 | 3 | 3 |
| 1546 | 9312549 | 334 | 1707 | 41 | 1 | 5 |
| 1547 | 9312550 | 304 | 1730 | 28 | 2 | 4 |
| 1548 | 9312551 | 239 | 1175 | 30 | 3 | 3 |
| 1549 | 9312552 | 227 | 1094 | 26 | 0 | 3 |
| 1550 | 9312553 | 223 | 967 | 20 | 3 | 2 |
| 1551 | 9312554 | 244 | 1317 | 28 | 1 | 2 |
| 1552 | 9312555 | 373 | 2333 | 31 | 6 | 3 |
| 1553 | 9312556 | 245 | 1607 | 28 | 5 | 3 |
| 1554 | 9312557 | 443 | 2484 | 60 | 3 | 5 |
| 1555 | 9312558 | 265 | 1533 | 36 | 2 | 3 |
| 1556 | 9312559 | 217 | 1225 | 18 | 2 | 1 |
| 1557 | 9312560 | 230 | 1094 | 30 | 3 | 0 |
| 1558 | 9312561 | 190 | 940 | 19 | 3 | 2 |
| 1559 | 9312562 | 232 | 1173 | 33 | 3 | 1 |
| 1560 | 9312563 | 225 | 1095 | 27 | 2 | 2 |
| 1561 | 9312564 | 199 | 1186 | 21 | 2 | 1 |
| 1562 | 9312565 | 249 | 1501 | 26 | 3 | 3 |
| 1563 | 9312566 | 354 | 2364 | 35 | 6 | 2 |
| 1564 | 9312567 | 240 | 1271 | 19 | 5 | 4 |
| 1565 | 9312568 | 217 | 1033 | 17 | 1 | 2 |
| 1566 | 9312569 | 180 | 744 | 12 | 5 | 1 |
| 1567 | 9312570 | 238 | 1446 | 19 | 4 | 2 |
| 1568 | 9312571 | 253 | 1561 | 31 | 2 | 2 |
| 1569 | 9312572 | 225 | 1033 | 11 | 5 | 2 |
| 1570 | 9312573 | 251 | 1370 | 11 | 5 | 3 |
| 1571 | 9312574 | 226 | 1306 | 21 | 3 | 2 |
| 1572 | 9312575 | 178 | 906 | 13 | 6 | 0 |
| 1573 | 9312576 | 279 | 1504 | 24 | 3 | 2 |
| 1574 | 9312577 | 216 | 1135 | 28 | 5 | 2 |
| 1575 | 9312578 | 265 | 1528 | 18 | 9 | 3 |
| 1576 | 9312579 | 265 | 1328 | 17 | 2 | 4 |
| 1577 | 9312580 | 182 | 757 | 18 | 2 | 2 |
| 1578 | 9312581 | 208 | 978 | 21 | 4 | 2 |
| 1579 | 9312582 | 231 | 819 | 13 | 4 | 3 |
| 1580 | 9312583 | 435 | 2375 | 54 | 9 | 4 |
| 1581 | 9312584 | 299 | 1422 | 22 | 0 | 5 |
| 1582 | 9312585 | 267 | 1504 | 32 | 0 | 2 |
| 1583 | 9312586 | 282 | 1541 | 32 | 2 | 2 |
| 1584 | 9312587 | 212 | 1041 | 14 | 3 | 1 |
| 1585 | 9312588 | 209 | 780 | 27 | 3 | 3 |
| 1586 | 9312589 | 220 | 1244 | 16 | 5 | 1 |
| 1587 | 9312590 | 214 | 939 | 19 | 2 | 3 |
| 1588 | 9312591 | 303 | 1457 | 38 | 6 | 4 |
| 1589 | 9312592 | 197 | 1007 | 11 | 3 | 3 |
| 1590 | 9312593 | 290 | 1617 | 20 | 3 | 4 |
| 1591 | 9312594 | 251 | 1334 | 22 | 5 | 2 |
| 1592 | 9312595 | 207 | 1094 | 18 | 4 | 1 |
| 1593 | 9312596 | 230 | 1066 | 22 | 4 | 2 |
| 1594 | 9312597 | 205 | 1117 | 17 | 4 | 4 |
| 1595 | 9312598 | 269 | 1008 | 27 | 3 | 4 |
| 1596 | 9312599 | 214 | 1348 | 18 | 4 | 3 |
| 1597 | 9312600 | 246 | 1209 | 22 | 6 | 2 |
| 1598 | 9312601 | 275 | 1384 | 28 | 1 | 4 |
| 1599 | 9312602 | 192 | 950 | 20 | 2 | 2 |
| 1600 | 9312603 | 274 | 1604 | 23 | 3 | 4 |
| 1601 | 9312604 | 227 | 1183 | 19 | 3 | 4 |
| 1602 | 9312605 | 243 | 1454 | 29 | 4 | 3 |
| 1603 | 9312606 | 250 | 1084 | 23 | 6 | 3 |
| 1604 | 9312607 | 273 | 1260 | 14 | 6 | 3 |
| 1605 | 9312608 | 293 | 1813 | 23 | 10 | 6 |
| 1606 | 9312609 | 229 | 1262 | 27 | 6 | 3 |
| 1607 | 9312610 | 236 | 1198 | 18 | 2 | 4 |
| 1608 | 9312611 | 302 | 1702 | 38 | 2 | 5 |
| 1609 | 9312612 | 246 | 1509 | 15 | 1 | 3 |
| 1610 | 9312613 | 236 | 1159 | 17 | 2 | 2 |
| 1611 | 9312614 | 285 | 1252 | 18 | 2 | 2 |
| 1612 | 9312615 | 210 | 951 | 19 | 2 | 2 |
| 1613 | 9312616 | 265 | 1350 | 28 | 3 | 3 |
| 1614 | 9312617 | 324 | 1832 | 42 | 4 | 4 |
| 1615 | 9312618 | 412 | 2334 | 34 | 3 | 4 |
| 1616 | 9312619 | 254 | 1301 | 17 | 4 | 4 |
| 1617 | 9312620 | 221 | 1431 | 26 | 2 | 2 |
| 1618 | 9312621 | 215 | 999 | 22 | 0 | 4 |
| 1619 | 9312622 | 298 | 1932 | 35 | 8 | 4 |
| 1620 | 9312623 | 212 | 1083 | 20 | 3 | 2 |
| 1621 | 9312624 | 211 | 1183 | 25 | 4 | 2 |
| 1622 | 9312625 | 242 | 1738 | 21 | 2 | 3 |
| 1623 | 9312626 | 288 | 1417 | 31 | 4 | 4 |
| 1624 | 9312627 | 220 | 1394 | 30 | 4 | 3 |
| 1625 | 9312628 | 225 | 1056 | 22 | 5 | 4 |
| 1626 | 9312629 | 246 | 1251 | 20 | 1 | 2 |
| 1627 | 9312630 | 235 | 1298 | 25 | 8 | 3 |
| 1628 | 9312631 | 267 | 1133 | 29 | 0 | 5 |
| 1629 | 9312632 | 319 | 1678 | 40 | 2 | 4 |
| 1630 | 9312633 | 321 | 1865 | 35 | 1 | 5 |
| 1631 | 9312634 | 213 | 1052 | 20 | 5 | 2 |
| 1632 | 9312635 | 272 | 1270 | 32 | 4 | 2 |
| 1633 | 9312636 | 193 | 1051 | 17 | 1 | 3 |
| 1634 | 9312637 | 341 | 1466 | 35 | 0 | 3 |
| 1635 | 9312638 | 243 | 1023 | 26 | 6 | 2 |
| 1636 | 9312639 | 226 | 1218 | 29 | 2 | 4 |
| 1637 | 9312640 | 213 | 962 | 23 | 4 | 4 |
| 1638 | 9312641 | 158 | 614 | 13 | 2 | 2 |
| 1639 | 9312642 | 243 | 1236 | 29 | 3 | 2 |
| 1640 | 9312643 | 280 | 1557 | 45 | 2 | 2 |
| 1641 | 9312644 | 250 | 1358 | 14 | 3 | 1 |
| 1642 | 9312645 | 316 | 2050 | 37 | 3 | 3 |
| 1643 | 9312646 | 270 | 1316 | 38 | 6 | 2 |
| 1644 | 9312647 | 228 | 1548 | 29 | 2 | 2 |
| 1645 | 9312648 | 212 | 1163 | 15 | 5 | 0 |
| 1646 | 9312649 | 215 | 1347 | 27 | 1 | 1 |
| 1647 | 9312650 | 223 | 1457 | 21 | 0 | 3 |
| 1648 | 9312651 | 281 | 1578 | 22 | 2 | 1 |
| 1649 | 9312652 | 260 | 1715 | 35 | 4 | 4 |
| 1650 | 9312653 | 226 | 1480 | 12 | 7 | 4 |
| 1651 | 9312654 | 287 | 1891 | 28 | 7 | 3 |
| 1652 | 9312655 | 212 | 1113 | 12 | 9 | 2 |
| 1653 | 9312656 | 201 | 1222 | 17 | 1 | 3 |
| 1654 | 9312657 | 227 | 1175 | 20 | 4 | 0 |
| 1655 | 9312658 | 239 | 1297 | 31 | 3 | 3 |
| 1656 | 9312659 | 321 | 1993 | 34 | 3 | 2 |
| 1657 | 9312660 | 189 | 1187 | 10 | 6 | 4 |
| 1658 | 9312661 | 383 | 1961 | 38 | 1 | 3 |
| 1659 | 9312662 | 217 | 1125 | 17 | 1 | 2 |
| 1660 | 9312663 | 228 | 1323 | 23 | 2 | 1 |
| 1661 | 9312664 | 234 | 1323 | 32 | 7 | 3 |
| 1662 | 9312665 | 221 | 1259 | 16 | 11 | 6 |
| 1663 | 9312666 | 352 | 1660 | 37 | 17 | 4 |
| 1664 | 9312667 | 215 | 1144 | 10 | 5 | 3 |
| 1665 | 9312668 | 251 | 1205 | 32 | 2 | 2 |
| 1666 | 9312669 | 314 | 1673 | 23 | 2 | 3 |
| 1667 | 9312670 | 224 | 1345 | 26 | 3 | 1 |
| 1668 | 9312671 | 278 | 1357 | 26 | 2 | 4 |
| 1669 | 9312672 | 355 | 1796 | 37 | 7 | 2 |
| 1670 | 9312673 | 262 | 1324 | 32 | 5 | 4 |
| 1671 | 9312674 | 248 | 1069 | 20 | 2 | 3 |
| 1672 | 9312675 | 232 | 1128 | 27 | 3 | 2 |
| 1673 | 9312676 | 270 | 1227 | 40 | 2 | 5 |
| 1674 | 9312677 | 500 | 2460 | 49 | 1 | 3 |
| 1675 | 9312678 | 227 | 1356 | 12 | 8 | 3 |
| 1676 | 9312679 | 239 | 965 | 13 | 3 | 4 |
| 1677 | 9312680 | 209 | 744 | 15 | 4 | 2 |
| 1678 | 9312681 | 271 | 1421 | 28 | 3 | 2 |
| 1679 | 9312682 | 208 | 1175 | 24 | 2 | 2 |
| 1680 | 9312683 | 217 | 1106 | 24 | 7 | 3 |
| 1681 | 9312684 | 192 | 960 | 15 | 2 | 1 |
| 1682 | 9312685 | 182 | 913 | 15 | 7 | 2 |
| 1683 | 9312686 | 247 | 1372 | 27 | 2 | 3 |
| 1684 | 9312687 | 259 | 1482 | 23 | 6 | 4 |
| 1685 | 9312688 | 196 | 775 | 21 | 2 | 1 |
| 1686 | 9312689 | 241 | 1187 | 32 | 2 | 3 |
| 1687 | 9312690 | 237 | 1175 | 24 | 4 | 3 |
| 1688 | 9312691 | 201 | 1119 | 15 | 4 | 3 |
| 1689 | 9312692 | 313 | 1536 | 20 | 0 | 5 |
| 1690 | 9312693 | 292 | 1236 | 28 | 2 | 2 |
| 1691 | 9312694 | 209 | 1058 | 25 | 1 | 1 |
| 1692 | 9312695 | 216 | 1233 | 20 | 3 | 3 |
| 1693 | 9312696 | 295 | 1889 | 17 | 6 | 5 |
| 1694 | 9312697 | 250 | 1193 | 15 | 1 | 0 |
| 1695 | 9312698 | 257 | 933 | 28 | 8 | 5 |
| 1696 | 9312699 | 239 | 1356 | 22 | 2 | 4 |
| 1697 | 9312700 | 221 | 926 | 16 | 2 | 5 |
| 1698 | 9312701 | 242 | 1638 | 24 | 2 | 3 |
| 1699 | 9312702 | 274 | 1502 | 21 | 5 | 2 |
| 1700 | 9312703 | 223 | 1507 | 22 | 4 | 3 |
| 1701 | 9312704 | 232 | 1144 | 14 | 1 | 2 |
| 1702 | 9312705 | 294 | 1796 | 53 | 0 | 4 |
| 1703 | 9312706 | 278 | 1587 | 32 | 0 | 5 |
| 1704 | 9312707 | 233 | 991 | 21 | 0 | 6 |
| 1705 | 9312708 | 216 | 1308 | 27 | 6 | 3 |
| 1706 | 9312709 | 242 | 1018 | 13 | 2 | 4 |
| 1707 | 9312710 | 218 | 1116 | 14 | 3 | 3 |
| 1708 | 9312711 | 217 | 1037 | 17 | 0 | 3 |
| 1709 | 9312712 | 215 | 903 | 30 | 2 | 5 |
| 1710 | 9312713 | 266 | 1188 | 20 | 3 | 4 |
| 1711 | 9312714 | 241 | 1022 | 16 | 3 | 5 |
| 1712 | 9312715 | 282 | 1907 | 30 | 2 | 4 |
| 1713 | 9312716 | 324 | 1663 | 42 | 3 | 5 |
| 1714 | 9312717 | 309 | 1787 | 23 | 5 | 3 |
| 1715 | 9312718 | 273 | 1431 | 45 | 0 | 3 |
| 1716 | 9312719 | 281 | 1605 | 26 | 3 | 4 |
| 1717 | 9312720 | 292 | 1729 | 35 | 6 | 4 |
| 1718 | 9312721 | 221 | 915 | 18 | 2 | 3 |
| 1719 | 9312722 | 472 | 3061 | 50 | 4 | 4 |
| 1720 | 9312723 | 267 | 1135 | 31 | 2 | 3 |
| 1721 | 9312724 | 287 | 1404 | 28 | 0 | 3 |
| 1722 | 9312725 | 220 | 1053 | 22 | 4 | 4 |
| 1723 | 9312726 | 302 | 1752 | 39 | 1 | 3 |
| 1724 | 9312727 | 237 | 1434 | 29 | 3 | 2 |
| 1725 | 9312728 | 199 | 1048 | 27 | 6 | 4 |
| 1726 | 9312729 | 179 | 987 | 16 | 0 | 3 |
| 1727 | 9312730 | 297 | 1422 | 41 | 2 | 4 |
| 1728 | 9312731 | 194 | 1072 | 23 | 2 | 2 |
| 1729 | 9312732 | 182 | 763 | 12 | 2 | 1 |
| 1730 | 9312733 | 212 | 882 | 19 | 0 | 2 |
| 1731 | 9312734 | 269 | 1356 | 27 | 4 | 1 |
| 1732 | 9312735 | 235 | 1455 | 32 | 3 | 3 |
| 1733 | 9312736 | 189 | 865 | 23 | 1 | 3 |
| 1734 | 9312737 | 353 | 1916 | 45 | 5 | 4 |
| 1735 | 9312738 | 254 | 1284 | 26 | 5 | 2 |
| 1736 | 9312739 | 255 | 1583 | 12 | 1 | 4 |
| 1737 | 9312740 | 226 | 1168 | 16 | 4 | 2 |
| 1738 | 9312741 | 290 | 1686 | 44 | 6 | 3 |
| 1739 | 9312742 | 210 | 1018 | 18 | 5 | 2 |
| 1740 | 9312743 | 216 | 1220 | 21 | 2 | 2 |
| 1741 | 9312744 | 324 | 1753 | 22 | 1 | 3 |
| 1742 | 9312745 | 236 | 1208 | 38 | 1 | 3 |
| 1743 | 9312746 | 317 | 2269 | 33 | 6 | 4 |
| 1744 | 9312747 | 304 | 1729 | 31 | 2 | 2 |
| 1745 | 9312748 | 248 | 1420 | 32 | 0 | 3 |
| 1746 | 9312749 | 275 | 1231 | 37 | 8 | 5 |
| 1747 | 9312750 | 300 | 1650 | 27 | 2 | 4 |
| 1748 | 9312751 | 252 | 1415 | 31 | 5 | 3 |
| 1749 | 9312752 | 206 | 1129 | 19 | 0 | 4 |
| 1750 | 9312753 | 193 | 974 | 13 | 7 | 1 |
| 1751 | 9312754 | 246 | 1415 | 17 | 1 | 0 |
| 1752 | 9312755 | 246 | 1061 | 20 | 2 | 4 |
| 1753 | 9312756 | 204 | 835 | 21 | 2 | 3 |
| 1754 | 9312757 | 308 | 1173 | 39 | 1 | 5 |
| 1755 | 9312758 | 235 | 879 | 27 | 3 | 3 |
| 1756 | 9312759 | 191 | 720 | 9 | 1 | 4 |
| 1757 | 9312760 | 168 | 620 | 11 | 2 | 1 |
| 1758 | 9312761 | 398 | 2143 | 38 | 2 | 2 |
| 1759 | 9312762 | 275 | 1207 | 24 | 1 | 4 |
| 1760 | 9312763 | 221 | 1024 | 15 | 4 | 3 |
| 1761 | 9312764 | 243 | 1178 | 27 | 5 | 1 |
| 1762 | 9312765 | 242 | 1295 | 13 | 1 | 3 |
| 1763 | 9312766 | 219 | 991 | 18 | 2 | 3 |
| 1764 | 9312767 | 200 | 825 | 17 | 5 | 0 |
| 1765 | 9312768 | 225 | 849 | 17 | 4 | 1 |
| 1766 | 9312769 | 392 | 1760 | 40 | 2 | 4 |
| 1767 | 9312770 | 216 | 989 | 16 | 4 | 3 |
| 1768 | 9312771 | 195 | 690 | 14 | 2 | 2 |
| 1769 | 9312772 | 277 | 1417 | 32 | 2 | 4 |
| 1770 | 9312773 | 187 | 687 | 10 | 2 | 2 |
| 1771 | 9312774 | 295 | 1657 | 22 | 9 | 2 |
| 1772 | 9312775 | 266 | 1343 | 24 | 2 | 3 |
| 1773 | 9312776 | 228 | 852 | 21 | 5 | 4 |
| 1774 | 9312777 | 220 | 850 | 21 | 5 | 3 |
| 1775 | 9312778 | 233 | 1034 | 22 | 2 | 3 |
| 1776 | 9312779 | 210 | 923 | 17 | 2 | 0 |
| 1777 | 9312780 | 189 | 783 | 20 | 0 | 2 |
| 1778 | 9312781 | 205 | 938 | 19 | 3 | 2 |
| 1779 | 9312782 | 400 | 2242 | 41 | 8 | 5 |
| 1780 | 9312783 | 268 | 1578 | 20 | 9 | 3 |
| 1781 | 9312784 | 235 | 1152 | 24 | 4 | 3 |
| 1782 | 9312785 | 226 | 944 | 17 | 4 | 3 |
| 1783 | 9312786 | 235 | 1235 | 23 | 1 | 1 |
| 1784 | 9312787 | 263 | 1447 | 25 | 5 | 3 |
| 1785 | 9312788 | 304 | 1664 | 31 | 1 | 4 |
| 1786 | 9312789 | 242 | 1288 | 22 | 4 | 5 |
| 1787 | 9312790 | 228 | 1241 | 18 | 3 | 5 |
| 1788 | 9312791 | 270 | 1335 | 19 | 1 | 1 |
| 1789 | 9312792 | 248 | 1272 | 23 | 0 | 2 |
| 1790 | 9312793 | 195 | 1148 | 16 | 1 | 3 |
| 1791 | 9312794 | 311 | 1848 | 29 | 4 | 5 |
| 1792 | 9312795 | 213 | 1148 | 21 | 1 | 2 |
| 1793 | 9312796 | 280 | 1128 | 24 | 2 | 3 |
| 1794 | 9312797 | 351 | 2355 | 29 | 7 | 2 |
| 1795 | 9312798 | 202 | 962 | 21 | 2 | 4 |
| 1796 | 9312799 | 238 | 1284 | 17 | 7 | 3 |
| 1797 | 9312800 | 272 | 1341 | 27 | 7 | 5 |
| 1798 | 9312801 | 289 | 1791 | 29 | 8 | 3 |
| 1799 | 9312802 | 231 | 1476 | 14 | 3 | 5 |
| 1800 | 9312803 | 346 | 1794 | 37 | 3 | 5 |
| 1801 | 9312804 | 299 | 1599 | 31 | 0 | 2 |
| 1802 | 9312805 | 308 | 1765 | 42 | 2 | 3 |
| 1803 | 9312806 | 245 | 919 | 15 | 3 | 2 |
| 1804 | 9312807 | 298 | 2161 | 24 | 3 | 2 |
| 1805 | 9312808 | 221 | 1324 | 20 | 2 | 2 |
| 1806 | 9312809 | 360 | 2456 | 40 | 1 | 3 |
| 1807 | 9312810 | 309 | 1395 | 28 | 5 | 5 |
| 1808 | 9312811 | 250 | 1342 | 23 | 0 | 4 |
| 1809 | 9312812 | 236 | 1480 | 20 | 0 | 3 |
| 1810 | 9312813 | 253 | 1270 | 26 | 1 | 2 |
| 1811 | 9312814 | 217 | 1048 | 27 | 3 | 1 |
| 1812 | 9312815 | 289 | 1401 | 25 | 4 | 4 |
| 1813 | 9312816 | 216 | 884 | 16 | 1 | 2 |
| 1814 | 9312817 | 219 | 1049 | 15 | 4 | 1 |
| 1815 | 9312818 | 198 | 1053 | 17 | 4 | 1 |
| 1816 | 9312819 | 213 | 1079 | 16 | 4 | 1 |
| 1817 | 9312820 | 301 | 1717 | 37 | 7 | 3 |
| 1818 | 9312821 | 219 | 1045 | 20 | 1 | 2 |
| 1819 | 9312822 | 222 | 964 | 24 | 2 | 3 |
| 1820 | 9312823 | 259 | 1168 | 31 | 1 | 4 |
| 1821 | 9312824 | 223 | 1317 | 24 | 0 | 2 |
| 1822 | 9312825 | 322 | 1845 | 39 | 4 | 3 |
| 1823 | 9312826 | 235 | 1196 | 30 | 2 | 2 |
| 1824 | 9312827 | 270 | 1363 | 36 | 8 | 2 |
| 1825 | 9312828 | 339 | 2225 | 34 | 5 | 1 |
| 1826 | 9312829 | 207 | 1127 | 25 | 1 | 1 |
| 1827 | 9312830 | 199 | 831 | 15 | 3 | 2 |
| 1828 | 9312831 | 239 | 1106 | 25 | 2 | 1 |
| 1829 | 9312832 | 355 | 2622 | 28 | 8 | 2 |
| 1830 | 9312833 | 196 | 1000 | 15 | 3 | 1 |
| 1831 | 9312834 | 291 | 1528 | 28 | 5 | 2 |
| 1832 | 9312835 | 261 | 1411 | 17 | 7 | 4 |
| 1833 | 9312836 | 206 | 1224 | 19 | 4 | 0 |
| 1834 | 9312837 | 274 | 1526 | 33 | 0 | 2 |
| 1835 | 9312838 | 221 | 1100 | 26 | 4 | 2 |
| 1836 | 9312839 | 298 | 1578 | 36 | 3 | 2 |
| 1837 | 9312840 | 229 | 1213 | 31 | 5 | 5 |
| 1838 | 9312841 | 485 | 2926 | 34 | 5 | 4 |
| 1839 | 9312842 | 280 | 1616 | 31 | 6 | 3 |
| 1840 | 9312843 | 258 | 1299 | 23 | 1 | 5 |
| 1841 | 9312844 | 229 | 919 | 13 | 5 | 1 |
| 1842 | 9312845 | 242 | 1288 | 23 | 1 | 3 |
| 1843 | 9312846 | 187 | 700 | 12 | 0 | 3 |
| 1844 | 9312847 | 240 | 1242 | 30 | 0 | 3 |
| 1845 | 9312848 | 268 | 1096 | 20 | 3 | 2 |
| 1846 | 9312849 | 272 | 1130 | 21 | 0 | 3 |
| 1847 | 9312850 | 211 | 1110 | 25 | 0 | 2 |
| 1848 | 9312851 | 240 | 1168 | 33 | 14 | 0 |
| 1849 | 9312852 | 279 | 1395 | 22 | 3 | 3 |
| 1850 | 9312853 | 221 | 895 | 21 | 5 | 2 |
| 1851 | 9312854 | 301 | 1283 | 30 | 8 | 3 |
| 1852 | 9312855 | 287 | 1537 | 29 | 7 | 4 |
| 1853 | 9312856 | 246 | 1002 | 10 | 4 | 2 |
| 1854 | 9312857 | 267 | 1339 | 20 | 3 | 3 |
| 1855 | 9312858 | 266 | 1536 | 21 | 10 | 2 |
| 1856 | 9312859 | 204 | 1006 | 21 | 7 | 1 |
| 1857 | 9312860 | 271 | 1261 | 19 | 5 | 1 |
| 1858 | 9312861 | 264 | 1664 | 23 | 2 | 4 |
| 1859 | 9312862 | 276 | 1123 | 43 | 5 | 4 |
| 1860 | 9312863 | 213 | 756 | 22 | 3 | 1 |
| 1861 | 9312864 | 256 | 1300 | 36 | 8 | 3 |
| 1862 | 9312865 | 257 | 1077 | 21 | 0 | 2 |
| 1863 | 9312866 | 219 | 714 | 15 | 2 | 4 |
| 1864 | 9312867 | 228 | 1240 | 24 | 4 | 5 |
| 1865 | 9312868 | 294 | 1565 | 25 | 6 | 2 |
| 1866 | 9312869 | 301 | 1157 | 35 | 2 | 3 |
| 1867 | 9312870 | 295 | 1505 | 31 | 3 | 3 |
| 1868 | 9312871 | 258 | 1491 | 22 | 3 | 3 |
| 1869 | 9312872 | 247 | 1324 | 24 | 0 | 3 |
| 1870 | 9312873 | 208 | 877 | 23 | 3 | 2 |
| 1871 | 9312874 | 232 | 1076 | 22 | 4 | 3 |
| 1872 | 9312875 | 290 | 1696 | 40 | 3 | 5 |
| 1873 | 9312876 | 222 | 1351 | 34 | 4 | 3 |
| 1874 | 9312877 | 217 | 1083 | 16 | 0 | 4 |
| 1875 | 9312878 | 222 | 1145 | 20 | 2 | 3 |
| 1876 | 9312879 | 217 | 1164 | 16 | 2 | 3 |
| 1877 | 9312880 | 296 | 2045 | 23 | 8 | 1 |
| 1878 | 9312881 | 299 | 1666 | 19 | 0 | 3 |
| 1879 | 9312882 | 242 | 1331 | 21 | 2 | 4 |
| 1880 | 9312883 | 190 | 968 | 11 | 4 | 3 |
| 1881 | 9312884 | 411 | 2751 | 41 | 2 | 1 |
| 1882 | 9312885 | 297 | 1782 | 31 | 4 | 3 |
| 1883 | 9312886 | 276 | 1400 | 13 | 7 | 5 |
| 1884 | 9312887 | 229 | 1336 | 27 | 2 | 3 |
| 1885 | 9312888 | 192 | 801 | 17 | 0 | 3 |
| 1886 | 9312889 | 475 | 2940 | 42 | 2 | 3 |
| 1887 | 9312890 | 282 | 1701 | 38 | 4 | 4 |
| 1888 | 9312891 | 246 | 1135 | 24 | 2 | 2 |
| 1889 | 9312892 | 173 | 787 | 10 | 1 | 1 |
| 1890 | 9312893 | 306 | 1870 | 28 | 2 | 3 |
| 1891 | 9312894 | 340 | 1984 | 53 | 3 | 4 |
| 1892 | 9312895 | 336 | 1974 | 35 | 3 | 3 |
| 1893 | 9312896 | 259 | 1110 | 16 | 1 | 4 |
| 1894 | 9312897 | 295 | 1577 | 31 | 4 | 4 |
| 1895 | 9312898 | 277 | 1095 | 27 | 2 | 4 |
| 1896 | 9312899 | 249 | 1302 | 23 | 3 | 3 |
| 1897 | 9312900 | 227 | 901 | 28 | 4 | 3 |
| 1898 | 9312901 | 263 | 1866 | 23 | 2 | 1 |
| 1899 | 9312902 | 236 | 1069 | 26 | 2 | 2 |
| 1900 | 9312903 | 222 | 1277 | 22 | 3 | 2 |
| 1901 | 9312904 | 252 | 1174 | 25 | 0 | 3 |
| 1902 | 9312905 | 242 | 1556 | 25 | 2 | 2 |
| 1903 | 9312906 | 315 | 1791 | 39 | 3 | 4 |
| 1904 | 9312907 | 229 | 1154 | 24 | 3 | 2 |
| 1905 | 9312908 | 300 | 1595 | 37 | 3 | 3 |
| 1906 | 9312909 | 283 | 1877 | 37 | 5 | 4 |
| 1907 | 9312910 | 181 | 991 | 17 | 6 | 1 |
| 1908 | 9312911 | 265 | 1492 | 31 | 1 | 2 |
| 1909 | 9312912 | 178 | 1076 | 20 | 0 | 1 |
| 1910 | 9312913 | 286 | 1842 | 33 | 3 | 1 |
| 1911 | 9312914 | 214 | 1116 | 22 | 3 | 3 |
| 1912 | 9312915 | 194 | 1169 | 18 | 4 | 3 |
| 1913 | 9312916 | 300 | 1975 | 35 | 5 | 1 |
| 1914 | 9312917 | 266 | 1493 | 39 | 5 | 3 |
| 1915 | 9312918 | 234 | 1102 | 18 | 5 | 0 |
| 1916 | 9312919 | 269 | 1716 | 25 | 4 | 3 |
| 1917 | 9312920 | 292 | 1698 | 35 | 4 | 2 |
| 1918 | 9312921 | 385 | 2147 | 41 | 6 | 3 |
| 1919 | 9312922 | 273 | 1550 | 30 | 5 | 5 |
| 1920 | 9312923 | 239 | 1184 | 34 | 3 | 1 |
| 1921 | 9312924 | 203 | 1192 | 18 | 5 | 2 |
| 1922 | 9312925 | 198 | 1063 | 8 | 7 | 1 |
| 1923 | 9312926 | 304 | 1914 | 46 | 9 | 3 |
| 1924 | 9312927 | 208 | 1182 | 21 | 4 | 3 |
| 1925 | 9312928 | 326 | 1985 | 45 | 8 | 3 |
| 1926 | 9312929 | 250 | 1604 | 22 | 4 | 1 |
| 1927 | 9312930 | 286 | 1882 | 33 | 1 | 3 |
| 1928 | 9312931 | 261 | 1155 | 33 | 1 | 3 |
| 1929 | 9312932 | 232 | 1209 | 25 | 2 | 1 |
| 1930 | 9312933 | 265 | 1468 | 29 | 1 | 3 |
| 1931 | 9312934 | 217 | 983 | 18 | 0 | 4 |
| 1932 | 9312935 | 242 | 1174 | 18 | 1 | 5 |
| 1933 | 9312936 | 275 | 1522 | 31 | 0 | 2 |
| 1934 | 9312937 | 328 | 1743 | 22 | 4 | 4 |
| 1935 | 9312938 | 317 | 1555 | 42 | 2 | 2 |
| 1936 | 9312939 | 276 | 1108 | 25 | 2 | 3 |
| 1937 | 9312940 | 180 | 807 | 12 | 3 | 3 |
| 1938 | 9312941 | 270 | 1281 | 28 | 5 | 5 |
| 1939 | 9312942 | 226 | 1069 | 27 | 3 | 2 |
| 1940 | 9312943 | 177 | 655 | 10 | 1 | 1 |
| 1941 | 9312944 | 342 | 1731 | 36 | 8 | 2 |
| 1942 | 9312945 | 261 | 1326 | 27 | 0 | 1 |
| 1943 | 9312946 | 317 | 1504 | 23 | 3 | 5 |
| 1944 | 9312947 | 253 | 1263 | 28 | 9 | 5 |
| 1945 | 9312948 | 283 | 1030 | 25 | 4 | 4 |
| 1946 | 9312949 | 192 | 764 | 21 | 1 | 2 |
| 1947 | 9312950 | 297 | 1430 | 28 | 13 | 2 |
| 1948 | 9312951 | 199 | 920 | 23 | 3 | 2 |
| 1949 | 9312952 | 237 | 1009 | 25 | 4 | 5 |
| 1950 | 9312953 | 244 | 1180 | 25 | 4 | 2 |
| 1951 | 9312954 | 195 | 869 | 7 | 3 | 2 |
| 1952 | 9312955 | 211 | 1072 | 24 | 8 | 4 |
| 1953 | 9312956 | 270 | 1832 | 25 | 4 | 2 |
| 1954 | 9312957 | 283 | 1506 | 25 | 0 | 2 |
| 1955 | 9312958 | 261 | 1465 | 33 | 1 | 4 |
| 1956 | 9312959 | 278 | 1328 | 28 | 1 | 2 |
| 1957 | 9312960 | 309 | 1824 | 26 | 8 | 6 |
| 1958 | 9312961 | 221 | 1089 | 13 | 2 | 4 |
| 1959 | 9312962 | 275 | 1486 | 39 | 2 | 3 |
| 1960 | 9312963 | 226 | 1194 | 26 | 0 | 2 |
| 1961 | 9312964 | 352 | 1998 | 48 | 2 | 5 |
| 1962 | 9312965 | 214 | 939 | 15 | 3 | 3 |
| 1963 | 9312966 | 256 | 1002 | 31 | 2 | 5 |
| 1964 | 9312967 | 346 | 1881 | 25 | 3 | 5 |
| 1965 | 9312968 | 233 | 1157 | 13 | 2 | 3 |
| 1966 | 9312969 | 328 | 1565 | 40 | 0 | 6 |
| 1967 | 9312970 | 278 | 1744 | 21 | 4 | 1 |
| 1968 | 9312971 | 208 | 1114 | 11 | 4 | 0 |
| 1969 | 9312972 | 376 | 2291 | 31 | 7 | 2 |
| 1970 | 9312973 | 329 | 1907 | 33 | 1 | 3 |
| 1971 | 9312974 | 320 | 1765 | 30 | 0 | 4 |
| 1972 | 9312975 | 219 | 1109 | 20 | 3 | 2 |
| 1973 | 9312976 | 262 | 1479 | 26 | 6 | 1 |
| 1974 | 9312977 | 284 | 1556 | 21 | 5 | 5 |
| 1975 | 9312978 | 313 | 1679 | 37 | 2 | 3 |
| 1976 | 9312979 | 250 | 1628 | 24 | 2 | 2 |
| 1977 | 9312980 | 193 | 981 | 16 | 1 | 3 |
| 1978 | 9312981 | 253 | 1172 | 24 | 6 | 3 |
| 1979 | 9312982 | 262 | 1366 | 29 | 0 | 4 |
| 1980 | 9312983 | 271 | 1687 | 46 | 1 | 2 |
| 1981 | 9312984 | 264 | 1287 | 29 | 3 | 4 |
| 1982 | 9312985 | 262 | 1448 | 39 | 2 | 4 |
| 1983 | 9312986 | 249 | 1073 | 21 | 1 | 2 |
| 1984 | 9312987 | 202 | 965 | 11 | 0 | 2 |
| 1985 | 9312988 | 250 | 1405 | 31 | 1 | 3 |
| 1986 | 9312989 | 422 | 2001 | 37 | 0 | 4 |
| 1987 | 9312990 | 222 | 899 | 12 | 5 | 1 |
| 1988 | 9312991 | 262 | 1610 | 24 | 4 | 1 |
| 1989 | 9312992 | 251 | 1048 | 29 | 1 | 5 |
| 1990 | 9312993 | 325 | 1520 | 31 | 2 | 4 |
| 1991 | 9312994 | 226 | 994 | 29 | 2 | 2 |
| 1992 | 9312995 | 288 | 1561 | 35 | 5 | 2 |
| 1993 | 9312996 | 197 | 1113 | 27 | 4 | 3 |
| 1994 | 9312997 | 293 | 1613 | 27 | 4 | 2 |
| 1995 | 9312998 | 241 | 1304 | 18 | 1 | 0 |
| 1996 | 9312999 | 280 | 1307 | 31 | 0 | 3 |
| 1997 | 9313000 | 240 | 1320 | 24 | 3 | 4 |
| 1998 | 9313001 | 283 | 1281 | 29 | 9 | 2 |
| 1999 | 9313002 | 220 | 977 | 20 | 5 | 3 |
| 2000 | 9313003 | 280 | 1564 | 29 | 6 | 3 |
| 2001 | 9313004 | 195 | 863 | 17 | 11 | 3 |
| 2002 | 9313005 | 199 | 917 | 13 | 4 | 0 |
| 2003 | 9313006 | 245 | 1243 | 26 | 3 | 2 |
| 2004 | 9313007 | 188 | 912 | 20 | 2 | 4 |
| 2005 | 9313008 | 191 | 1060 | 16 | 1 | 2 |
| 2006 | 9313009 | 294 | 1577 | 27 | 6 | 1 |
| 2007 | 9313010 | 195 | 1122 | 19 | 6 | 4 |
| 2008 | 9313011 | 233 | 1108 | 23 | 4 | 2 |
| 2009 | 9313012 | 243 | 1344 | 22 | 6 | 2 |
| 2010 | 9313013 | 240 | 1302 | 25 | 9 | 2 |
| 2011 | 9313014 | 264 | 1506 | 21 | 7 | 5 |
| 2012 | 9313015 | 269 | 1342 | 36 | 5 | 3 |
| 2013 | 9313016 | 182 | 925 | 14 | 3 | 2 |
| 2014 | 9313017 | 323 | 1912 | 43 | 4 | 3 |
| 2015 | 9313018 | 191 | 975 | 21 | 11 | 1 |
| 2016 | 9313019 | 259 | 1204 | 20 | 10 | 5 |
| 2017 | 9313020 | 234 | 1050 | 16 | 2 | 2 |
| 2018 | 9313021 | 272 | 1535 | 29 | 2 | 3 |
| 2019 | 9313022 | 211 | 977 | 18 | 1 | 3 |
| 2020 | 9313023 | 236 | 1004 | 30 | 4 | 3 |
| 2021 | 9313024 | 233 | 1151 | 14 | 3 | 3 |
| 2022 | 9313025 | 192 | 1057 | 20 | 5 | 3 |
| 2023 | 9313026 | 353 | 1880 | 38 | 2 | 4 |
| 2024 | 9313027 | 254 | 1375 | 27 | 2 | 3 |
| 2025 | 9313028 | 213 | 1153 | 19 | 4 | 2 |
| 2026 | 9313029 | 229 | 868 | 16 | 1 | 3 |
| 2027 | 9313030 | 207 | 787 | 11 | 8 | 3 |
| 2028 | 9313031 | 272 | 1725 | 27 | 7 | 2 |
| 2029 | 9313032 | 275 | 1095 | 25 | 3 | 4 |
| 2030 | 9313033 | 289 | 1234 | 21 | 2 | 2 |
| 2031 | 9313034 | 294 | 1425 | 29 | 2 | 4 |
| 2032 | 9313035 | 193 | 865 | 24 | 3 | 3 |
| 2033 | 9313036 | 288 | 1257 | 33 | 3 | 2 |
| 2034 | 9313037 | 278 | 1585 | 43 | 5 | 3 |
| 2035 | 9313038 | 225 | 943 | 21 | 8 | 2 |
| 2036 | 9313039 | 210 | 842 | 19 | 7 | 3 |
| 2037 | 9313040 | 184 | 837 | 15 | 2 | 2 |
| 2038 | 9313041 | 250 | 1499 | 32 | 1 | 1 |
| 2039 | 9313042 | 314 | 1654 | 29 | 5 | 4 |
| 2040 | 9313043 | 296 | 1590 | 31 | 5 | 2 |
| 2041 | 9313044 | 258 | 1361 | 25 | 4 | 4 |
| 2042 | 9313045 | 274 | 1318 | 24 | 2 | 4 |
| 2043 | 9313046 | 228 | 1300 | 18 | 6 | 2 |
| 2044 | 9313047 | 243 | 1797 | 22 | 6 | 3 |
| 2045 | 9313048 | 401 | 2238 | 31 | 10 | 3 |
| 2046 | 9313049 | 254 | 1198 | 27 | 11 | 4 |
| 2047 | 9313050 | 204 | 1090 | 22 | 3 | 2 |
| 2048 | 9313051 | 216 | 992 | 11 | 3 | 4 |
| 2049 | 9313052 | 205 | 1255 | 17 | 3 | 3 |
| 2050 | 9313053 | 226 | 1275 | 21 | 4 | 3 |
| 2051 | 9313054 | 233 | 1253 | 23 | 3 | 2 |
| 2052 | 9313055 | 210 | 932 | 13 | 3 | 3 |
| 2053 | 9313056 | 230 | 1258 | 24 | 2 | 3 |
| 2054 | 9313057 | 232 | 1219 | 26 | 2 | 3 |
| 2055 | 9313058 | 266 | 1361 | 31 | 1 | 4 |
| 2056 | 9313059 | 296 | 1217 | 18 | 0 | 3 |
| 2057 | 9313060 | 286 | 1360 | 22 | 8 | 3 |
| 2058 | 9313061 | 195 | 1217 | 15 | 3 | 0 |
| 2059 | 9313062 | 240 | 1012 | 25 | 3 | 3 |
| 2060 | 9313063 | 302 | 1520 | 16 | 1 | 5 |
| 2061 | 9313064 | 279 | 1588 | 34 | 6 | 4 |
| 2062 | 9313065 | 250 | 1170 | 16 | 3 | 2 |
| 2063 | 9313066 | 209 | 801 | 18 | 2 | 2 |
| 2064 | 9313067 | 223 | 1373 | 18 | 1 | 3 |
| 2065 | 9313068 | 249 | 1140 | 27 | 4 | 4 |
| 2066 | 9313069 | 387 | 2292 | 48 | 2 | 3 |
| 2067 | 9313070 | 177 | 703 | 16 | 0 | 4 |
| 2068 | 9313071 | 219 | 1084 | 20 | 2 | 2 |
| 2069 | 9313072 | 228 | 1251 | 20 | 2 | 3 |
| 2070 | 9313073 | 258 | 1351 | 37 | 6 | 4 |
| 2071 | 9313074 | 247 | 899 | 29 | 1 | 4 |
| 2072 | 9313075 | 250 | 896 | 25 | 1 | 2 |
| 2073 | 9313076 | 291 | 1668 | 29 | 7 | 2 |
| 2074 | 9313077 | 216 | 1345 | 34 | 1 | 3 |
| 2075 | 9313078 | 247 | 1617 | 26 | 4 | 3 |
| 2076 | 9313079 | 178 | 1028 | 11 | 0 | 0 |
| 2077 | 9313080 | 269 | 1270 | 21 | 2 | 2 |
| 2078 | 9313081 | 190 | 1121 | 19 | 3 | 2 |
| 2079 | 9313082 | 229 | 1021 | 24 | 2 | 4 |
| 2080 | 9313083 | 180 | 742 | 10 | 1 | 2 |
| 2081 | 9313084 | 244 | 1129 | 20 | 5 | 1 |
| 2082 | 9313085 | 217 | 1102 | 28 | 4 | 2 |
| 2083 | 9313086 | 268 | 1452 | 28 | 0 | 3 |
| 2084 | 9313087 | 194 | 739 | 17 | 6 | 2 |
| 2085 | 9313088 | 277 | 1372 | 41 | 3 | 3 |
| 2086 | 9313089 | 223 | 1177 | 23 | 6 | 3 |
| 2087 | 9313090 | 219 | 1519 | 24 | 2 | 2 |
| 2088 | 9313091 | 332 | 1482 | 42 | 5 | 6 |
| 2089 | 9313092 | 234 | 1397 | 28 | 2 | 2 |
| 2090 | 9313093 | 266 | 1648 | 17 | 3 | 4 |
| 2091 | 9313094 | 272 | 1651 | 35 | 1 | 4 |
| 2092 | 9313095 | 245 | 1639 | 23 | 0 | 4 |
| 2093 | 9313096 | 279 | 1768 | 37 | 2 | 2 |
| 2094 | 9313097 | 196 | 836 | 25 | 6 | 5 |
| 2095 | 9313098 | 221 | 1201 | 18 | 4 | 4 |
| 2096 | 9313099 | 281 | 1530 | 35 | 4 | 1 |
| 2097 | 9313100 | 231 | 1035 | 19 | 3 | 4 |
| 2098 | 9313101 | 207 | 1177 | 20 | 5 | 3 |
| 2099 | 9313102 | 281 | 1570 | 28 | 7 | 1 |
| 2100 | 9313103 | 183 | 899 | 6 | 0 | 1 |
| 2101 | 9313104 | 225 | 1174 | 22 | 2 | 1 |
| 2102 | 9313105 | 273 | 1484 | 30 | 0 | 2 |
| 2103 | 9313106 | 326 | 1865 | 43 | 0 | 3 |
| 2104 | 9313107 | 234 | 971 | 22 | 1 | 4 |
| 2105 | 9313108 | 252 | 1314 | 30 | 1 | 2 |
| 2106 | 9313109 | 251 | 947 | 30 | 6 | 3 |
| 2107 | 9313110 | 221 | 1440 | 28 | 6 | 3 |
| 2108 | 9313111 | 292 | 1049 | 32 | 9 | 4 |
| 2109 | 9313112 | 216 | 1015 | 19 | 0 | 4 |
| 2110 | 9313113 | 316 | 1863 | 35 | 6 | 4 |
| 2111 | 9313114 | 212 | 966 | 21 | 4 | 2 |

## Throughput

- Turns simulated: 540374 (first play of each game)
- Aggregate: 295.8 turns/s over 1827.1s wall
- Per game thread: 37.2 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

# Simulation: census-eleven-factions-rules67 (2026-11-28)

2112 games, first seed 9311003, brain `scripted`, turn limit off, safety cap 1200, 8 threads, started 2026-11-28T06:52:09Z. Wall time 1692.2s.

Source checkout at run start: `fed7207131c82dfd4c1a409439e3bfa993080943`; dirty: no. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. Games seat the eleven cyclic rosters of eight over all eleven factions (`--all-factions`), one roster per eight games, turned one seat per game, so every faction plays every seat equally often, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--players","8","--difficulty","hard","--all-factions","--games","2112","--seed","9311003","--threads","8","--label","census-eleven-factions-rules67"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 2112 of 2112 |
| Game length, turns (mean / median / min-max) | 252.0 / 244 / 140-500 |
| Win reason mix | VP hold 91.5%, Capitals 8.0%, Turn limit 0.5% |
| Median end turn | 244 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 8.0% / 91.5% / 0.5% / 0.0% |
| Games reaching the World War era (any empire) | 2097 of 2112 (99.3%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 2893 / 810 / 1940 / 143 |
| Hold break rate | 28.0% of started; 29.5% of resolved (broken / (broken + completed)) |
| Hold wins | 1933 |
| First reach to win, turns (mean / median / min-max) | 13.6 / 9 / 9-171 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 300 |
| Wins by a team another team beat to its target | 144 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 1933 | 246.9 / 241 / 140-477 |
| Capitals | 169 | 294.6 / 278 / 175-500 |
| Turn limit | 10 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 172 | 8.1% |
| India | 137 | 6.5% |
| Byzantium | 160 | 7.6% |
| Germany | 199 | 9.4% |
| Celts | 252 | 11.9% |
| Mongols | 79 | 3.7% |
| Sparta | 174 | 8.2% |
| Athens | 202 | 9.6% |
| Aztecs | 142 | 6.7% |
| Egypt | 546 | 25.9% |
| Rome | 49 | 2.3% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 1933 | 91.5% |
| Conquest | 0 | 0.0% |
| Capitals | 169 | 8.0% |
| Turn limit | 10 | 0.5% |
| Draw | 0 | 0.0% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 1536 | 172 | 11.2% |
| India | 1536 | 137 | 8.9% |
| Byzantium | 1536 | 160 | 10.4% |
| Germany | 1536 | 199 | 13.0% |
| Celts | 1536 | 252 | 16.4% |
| Mongols | 1536 | 79 | 5.1% |
| Sparta | 1536 | 174 | 11.3% |
| Athens | 1536 | 202 | 13.2% |
| Aztecs | 1536 | 142 | 9.2% |
| Egypt | 1536 | 546 | 35.5% |
| Rome | 1536 | 49 | 3.2% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 16896 | 2112 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 1536 | 172 | 11.2% |
| India | Hard | 1536 | 137 | 8.9% |
| Byzantium | Hard | 1536 | 160 | 10.4% |
| Germany | Hard | 1536 | 199 | 13.0% |
| Celts | Hard | 1536 | 252 | 16.4% |
| Mongols | Hard | 1536 | 79 | 5.1% |
| Sparta | Hard | 1536 | 174 | 11.3% |
| Athens | Hard | 1536 | 202 | 13.2% |
| Aztecs | Hard | 1536 | 142 | 9.2% |
| Egypt | Hard | 1536 | 546 | 35.5% |
| Rome | Hard | 1536 | 49 | 3.2% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 1536 | 148 | 9.6% | 137 | 170 | 208 | 286 | 388 |
| India | 1536 | 591 | 38.5% | 48 | 89 | 128 | 222 | 435 |
| Byzantium | 1536 | 860 | 56.0% | 71 | 86 | 120 | 170 | 334 |
| Germany | 1536 | 402 | 26.2% | 91 | 134 | 174 | 237 | 333 |
| Celts | 1536 | 1023 | 66.6% | 30 | 192 | 210 | 210 | 287 |
| Mongols | 1536 | 120 | 7.8% | 91 | 171 | 218 | 294 | 371 |
| Sparta | 1536 | 212 | 13.8% | 134 | 168 | 225 | 314 | 430 |
| Athens | 1536 | 565 | 36.8% | 128 | 188 | 255 | 255 | 255 |
| Aztecs | 1536 | 497 | 32.4% | 131 | 198 | 225 | 237 | 340 |
| Egypt | 1536 | 603 | 39.3% | 115 | 170 | 210 | 280 | 468 |
| Rome | 1536 | 38 | 2.5% | 119 | 147 | 207 | 322 | 406 |

## Goal cities (Rome's Colonia, Egypt's tomb)

Per seat of the faction: its goal city stood (median first turn), the seat scored it, held it at the end, a rival held it at the end, it changed hands. Rome: seats with a level-3 melee unit while the Colonia was open (median first turn) and the share of those seat-turns with no legal Colonia site on the map. Egypt: seats that started the work (median first start), work turns and pauses per starting seat.

| Faction | Seats | Stood | Turn | Scored | Held at end | Rival holds | Changed hands | Level 3 | Turn | No site | Started | Turn | Work turns | Pauses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Egypt | 1536 | 490 | 218 | 490 | 346 | 107 | 137 | 0 | - | - | 837 | 200 | 11.5 | 0.8 |
| Rome | 1536 | 0 | - | 0 | 0 | 0 | 0 | 1 | 230 | 0% | 0 | - | - | - |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 140 | 194 | 244 | 318 | 500 |
| Winning team VP | 0 | 10 | 10 | 12 | 17 |
| First Soldier completed (turn, per empire) | 6 | 8 | 14 | 53 | 272 |
| First Settler completed (turn, per empire) | 2 | 9 | 16 | 28 | 238 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 5 | 15 | 23 | 52 | 286 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 16896 observed empires in 2112 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 16208 | 688 |
| First Settler completed | 14652 | 2244 |
| First city founded | 16896 | 0 |
| Second city founded | 15661 | 1235 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 5264 |
| 1 | 2716 |
| 2 | 1861 |
| 3 | 1294 |
| 4 | 1017 |
| 5 | 796 |
| 6 | 643 |
| 7 | 525 |
| 8 | 383 |
| 9 | 258 |
| 10 | 1066 |
| 11 | 545 |
| 12 | 317 |
| 13 | 119 |
| 14 | 60 |
| 15 | 20 |
| 16 | 6 |
| 17 | 6 |

## End state

- Average game length: 252.0 turns
- Average cities per empire: 4.19
- Average techs per empire: 29.29
- Average eliminations per game: 3.23

## Strategy diversity: observed behavior (classifier v2)

2112 finished games; 16896 player seats; 2112 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4247** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 9252 | 543 | 5.9% | 54.8% | 25.7% |
| Wide | 4380 | 1461 | 33.4% | 25.9% | 69.2% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 14 | 0 | 0.0% | 0.1% | 0.0% |
| Economic | 716 | 106 | 14.8% | 4.2% | 5.0% |
| Generalist | 2534 | 2 | 0.1% | 15.0% | 0.1% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 15.9% / 2.4% (1979) | n/a / n/a (0) | 36.7% / 6.4% (14) | 21.8% / 4.9% (618) | 83.5% / 1.7% (1687) |
| Wide | 81.7% / 2.4% (1979) | — | n/a / n/a (0) | 86.9% / 5.4% (14) | 60.2% / 3.9% (570) | 96.7% / 0.5% (1575) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 56.9% / 6.4% (14) | 7.7% / 5.4% (14) | n/a / n/a (0) | — | 25.0% / 12.5% (4) | 80.0% / 0.0% (10) |
| Economic | 73.3% / 4.9% (618) | 35.9% / 3.9% (570) | n/a / n/a (0) | 62.5% / 12.5% (4) | — | 93.6% / 1.0% (464) |
| Generalist | 14.9% / 1.7% (1687) | 2.8% / 0.5% (1575) | n/a / n/a (0) | 20.0% / 0.0% (10) | 5.4% / 1.0% (464) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 20 / 565 | 130 / 499 | 0 / 0 | 0 / 0 | 22 / 154 | 0 / 318 |
| India | 57 / 914 | 80 / 322 | 0 / 0 | 0 / 0 | 0 / 6 | 0 / 294 |
| Byzantium | 43 / 672 | 108 / 452 | 0 / 0 | 0 / 3 | 9 / 100 | 0 / 309 |
| Germany | 36 / 672 | 154 / 515 | 0 / 0 | 0 / 2 | 9 / 63 | 0 / 284 |
| Celts | 35 / 601 | 208 / 606 | 0 / 0 | 0 / 0 | 9 / 80 | 0 / 249 |
| Mongols | 44 / 1197 | 33 / 80 | 0 / 0 | 0 / 0 | 2 / 15 | 0 / 244 |
| Sparta | 100 / 1285 | 60 / 195 | 0 / 0 | 0 / 5 | 14 / 29 | 0 / 22 |
| Athens | 40 / 592 | 147 / 469 | 0 / 0 | 0 / 0 | 14 / 103 | 1 / 372 |
| Aztecs | 37 / 744 | 98 / 410 | 0 / 0 | 0 / 1 | 6 / 79 | 1 / 302 |
| Egypt | 93 / 584 | 437 / 816 | 0 / 0 | 0 / 3 | 16 / 73 | 0 / 60 |
| Rome | 38 / 1426 | 6 / 16 | 0 / 0 | 0 / 0 | 5 / 14 | 0 / 80 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Turtle/Wonder. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 2112 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

2112 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 32975/2112 | 15.613 | 2112/2112 (100.0%) |
| Peace offers | 213879/2112 | 101.268 | 2112/2112 (100.0%) |
| Peace agreements | 26501/2112 | 12.548 | 2111/2112 (100.0%) |
| Battles | 2293707/2112 | 1086.036 | 2112/2112 (100.0%) |
| City captures | 54951/2112 | 26.018 | 2112/2112 (100.0%) |
| Settler captures | 7140/2112 | 3.381 | 1952/2112 (92.4%) |
| Eliminations | 6828/2112 | 3.233 | 2091/2112 (99.0%) |

## Terminal VP sources

2112 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 12552/16896 | 12552/16896 | 6752/2112 |
| Faction goals | 8148/16896 | 8148/16896 | 3370/2112 |
| Era objectives | 28704/16896 | 28704/16896 | 9356/2112 |
| Capitals taken | 5227/16896 | 5227/16896 | 3276/2112 |
| Total VP | 54631/16896 | 54631/16896 | 22754/2112 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 173/1409 (12.3%) | 0 |
| Tied leaders (winner in set) | 499/703 (71.0%) | 0 |
| Any leader set | 672/2112 (31.8%) | 0 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 15802/16896 | 5 | 20 | 28 | 33 | 305 | -2.6 (2112) |
| Medieval | 15062/16896 | 35 | 58 | 62 | 67 | 291 | -2.6 (2112) |
| Renaissance | 14851/16896 | 61 | 84 | 91 | 103 | 335 | -5.2 (2112) |
| Industrial | 14003/16896 | 85 | 111 | 122 | 143 | 256 | -8.5 (2112) |
| Machine Age | 12133/16896 | 106 | 140 | 155 | 180 | 297 | -11.4 (2111) |
| World War | 9427/16896 | 127 | 163 | 183 | 213 | 401 | -13.2 (2090) |
| Modern | 6352/16896 | 146 | 185 | 212 | 249 | 456 | -12.3 (1894) |
| Future | 3545/16896 | 163 | 208 | 241 | 273 | 440 | -9.9 (1367) |

Research idle turns (techs left, no research set): 18, of which stalled (two or more in a row): 0.

## Tech pace against the calendar

Median resolved turn each technology was learned, the display year of that turn (`calendar::calendar_label`) and the year the technology really arrived (n/a when no single year is established). Empires that never learned it are left out.

| Era | Technology | Learned / seats | Median turn | Median year | Real year |
| --- | --- | --- | --- | --- | --- |
| Ancient | Irrigation | 16815/16896 | 9 | 3173 BC | 6000 BC |
| Ancient | Bronze Working | 14970/16896 | 18 | 2242 BC | 3300 BC |
| Ancient | Masonry | 16893/16896 | 12 | 2863 BC | 3000 BC |
| Ancient | Currency | 15993/16896 | 26 | 1414 BC | 600 BC |
| Ancient | Writing | 16085/16896 | 24 | 1621 BC | 3200 BC |
| Classical | Mathematics | 15240/16896 | 44 | 301 BC | 300 BC |
| Classical | Engineering | 15133/16896 | 51 | 50 AD | n/a |
| Classical | Iron Working | 15378/16896 | 38 | 601 BC | 1200 BC |
| Classical | Code of Laws | 15083/16896 | 59 | 450 AD | 1754 BC |
| Classical | Philosophy | 15069/16896 | 61 | 530 AD | 500 BC |
| Medieval | Feudalism | 14986/16896 | 76 | 980 AD | n/a |
| Medieval | Machinery | 15019/16896 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 13594/16896 | 81 | 1130 AD | n/a |
| Medieval | Education | 14903/16896 | 88 | 1340 AD | 1088 AD |
| Medieval | Crop Rotation | 14857/16896 | 90 | 1400 AD | n/a |
| Renaissance | Gunpowder | 14769/16896 | 98 | 1480 AD | n/a |
| Renaissance | Metallurgy | 14510/16896 | 109 | 1590 AD | n/a |
| Renaissance | Military Engineering | 14569/16896 | 106 | 1560 AD | n/a |
| Renaissance | Banking | 14180/16896 | 119 | 1690 AD | 1397 AD |
| Renaissance | Printing Press | 14070/16896 | 120 | 1700 AD | 1440 AD |
| Industrial | Rifling | 12798/16896 | 144 | 1820 AD | n/a |
| Industrial | Military Science | 13322/16896 | 136 | 1790 AD | n/a |
| Industrial | Ballistics | 12988/16896 | 147 | 1831 AD | n/a |
| Industrial | Industrialization | 13690/16896 | 131 | 1772 AD | 1769 AD |
| Industrial | Railways | 12136/16896 | 155 | 1861 AD | 1825 AD |
| Machine Age | Replaceable Parts | 10878/16896 | 169 | 1890 AD | 1800 AD |
| Machine Age | Combustion | 11498/16896 | 164 | 1884 AD | 1876 AD |
| Machine Age | Steel | 10917/16896 | 168 | 1889 AD | 1856 AD |
| Machine Age | Flight | 9969/16896 | 178 | 1900 AD | 1903 AD |
| Machine Age | Electricity | 9512/16896 | 183 | 1906 AD | 1879 AD |
| World War | Mass Mobilisation | 7879/16896 | 198 | 1925 AD | n/a |
| World War | Rocketry | 7406/16896 | 201 | 1929 AD | 1926 AD |
| World War | Armoured Warfare | 7004/16896 | 206 | 1937 AD | 1916 AD |
| World War | Air Power | 6673/16896 | 210 | 1942 AD | n/a |
| World War | Electronics | 8770/16896 | 190 | 1914 AD | n/a |
| Modern | Mechanised Warfare | 4822/16896 | 226 | 1976 AD | n/a |
| Modern | Composite Armour | 4410/16896 | 231 | 1988 AD | n/a |
| Modern | Guided Missiles | 4119/16896 | 235 | 1998 AD | n/a |
| Modern | Jet Engines | 3979/16896 | 239 | 2007 AD | 1939 AD |
| Modern | Green Revolution | 5575/16896 | 219 | 1959 AD | n/a |
| Future | Autonomous Systems | 2531/16896 | 255 | 2031 AD | n/a |
| Future | Robotics | 2127/16896 | 258 | 2034 AD | n/a |
| Future | Fusion Power | 2049/16896 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 2876/16896 | 250 | 2026 AD | n/a |
| Future | Recursive AI | 1793/16896 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 37.72 |
| 1 | 69.07 | 8.39 |
| 2 | 20.13 | 3.00 |
| 3 | 5.13 | 0.76 |
| 4 | 0.97 | 0.17 |
| 5 | 0.17 | 0.04 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 9311003 | 312 | 1334 | 38 | 3 | 5 |
| 1 | 9311004 | 220 | 924 | 15 | 5 | 4 |
| 2 | 9311005 | 228 | 877 | 14 | 4 | 2 |
| 3 | 9311006 | 203 | 901 | 20 | 1 | 2 |
| 4 | 9311007 | 300 | 1114 | 27 | 5 | 4 |
| 5 | 9311008 | 211 | 846 | 21 | 6 | 3 |
| 6 | 9311009 | 186 | 868 | 17 | 2 | 1 |
| 7 | 9311010 | 212 | 738 | 20 | 2 | 2 |
| 8 | 9311011 | 257 | 888 | 25 | 5 | 3 |
| 9 | 9311012 | 285 | 831 | 24 | 3 | 3 |
| 10 | 9311013 | 324 | 1567 | 33 | 2 | 2 |
| 11 | 9311014 | 238 | 882 | 16 | 9 | 1 |
| 12 | 9311015 | 226 | 936 | 26 | 5 | 5 |
| 13 | 9311016 | 193 | 834 | 20 | 1 | 3 |
| 14 | 9311017 | 252 | 976 | 15 | 3 | 4 |
| 15 | 9311018 | 204 | 848 | 18 | 4 | 3 |
| 16 | 9311019 | 190 | 762 | 17 | 3 | 3 |
| 17 | 9311020 | 236 | 1107 | 23 | 6 | 5 |
| 18 | 9311021 | 238 | 991 | 32 | 6 | 4 |
| 19 | 9311022 | 252 | 1071 | 31 | 14 | 3 |
| 20 | 9311023 | 293 | 1380 | 26 | 7 | 4 |
| 21 | 9311024 | 289 | 1260 | 31 | 3 | 3 |
| 22 | 9311025 | 214 | 819 | 10 | 4 | 4 |
| 23 | 9311026 | 305 | 1234 | 36 | 7 | 4 |
| 24 | 9311027 | 260 | 1078 | 12 | 2 | 5 |
| 25 | 9311028 | 230 | 566 | 17 | 6 | 4 |
| 26 | 9311029 | 259 | 1144 | 12 | 14 | 2 |
| 27 | 9311030 | 213 | 1034 | 22 | 4 | 3 |
| 28 | 9311031 | 246 | 919 | 26 | 2 | 2 |
| 29 | 9311032 | 249 | 1126 | 34 | 2 | 3 |
| 30 | 9311033 | 248 | 847 | 23 | 4 | 5 |
| 31 | 9311034 | 246 | 1128 | 20 | 3 | 4 |
| 32 | 9311035 | 241 | 1349 | 35 | 1 | 1 |
| 33 | 9311036 | 306 | 1546 | 38 | 2 | 3 |
| 34 | 9311037 | 305 | 1324 | 28 | 3 | 6 |
| 35 | 9311038 | 260 | 1186 | 15 | 4 | 0 |
| 36 | 9311039 | 226 | 1023 | 22 | 1 | 3 |
| 37 | 9311040 | 228 | 986 | 19 | 1 | 2 |
| 38 | 9311041 | 318 | 1351 | 37 | 1 | 3 |
| 39 | 9311042 | 333 | 1299 | 33 | 4 | 5 |
| 40 | 9311043 | 314 | 1337 | 45 | 2 | 2 |
| 41 | 9311044 | 229 | 908 | 24 | 2 | 5 |
| 42 | 9311045 | 264 | 1137 | 31 | 6 | 3 |
| 43 | 9311046 | 209 | 794 | 22 | 6 | 4 |
| 44 | 9311047 | 242 | 879 | 18 | 1 | 4 |
| 45 | 9311048 | 188 | 709 | 11 | 0 | 1 |
| 46 | 9311049 | 376 | 1301 | 41 | 2 | 5 |
| 47 | 9311050 | 290 | 1133 | 27 | 3 | 2 |
| 48 | 9311051 | 270 | 1415 | 45 | 1 | 3 |
| 49 | 9311052 | 241 | 1035 | 29 | 2 | 4 |
| 50 | 9311053 | 321 | 1380 | 36 | 8 | 2 |
| 51 | 9311054 | 189 | 644 | 12 | 1 | 3 |
| 52 | 9311055 | 172 | 758 | 14 | 2 | 2 |
| 53 | 9311056 | 191 | 802 | 15 | 2 | 1 |
| 54 | 9311057 | 221 | 973 | 24 | 1 | 3 |
| 55 | 9311058 | 238 | 1163 | 24 | 3 | 3 |
| 56 | 9311059 | 331 | 1624 | 29 | 11 | 3 |
| 57 | 9311060 | 313 | 1645 | 33 | 9 | 4 |
| 58 | 9311061 | 212 | 695 | 25 | 5 | 3 |
| 59 | 9311062 | 153 | 653 | 13 | 2 | 1 |
| 60 | 9311063 | 189 | 827 | 26 | 6 | 1 |
| 61 | 9311064 | 342 | 1049 | 38 | 1 | 6 |
| 62 | 9311065 | 194 | 917 | 19 | 3 | 1 |
| 63 | 9311066 | 174 | 640 | 12 | 7 | 4 |
| 64 | 9311067 | 305 | 1499 | 39 | 3 | 4 |
| 65 | 9311068 | 234 | 963 | 25 | 4 | 3 |
| 66 | 9311069 | 210 | 1013 | 19 | 6 | 3 |
| 67 | 9311070 | 270 | 1410 | 38 | 8 | 1 |
| 68 | 9311071 | 223 | 1184 | 19 | 4 | 4 |
| 69 | 9311072 | 405 | 2134 | 50 | 5 | 5 |
| 70 | 9311073 | 178 | 654 | 17 | 1 | 1 |
| 71 | 9311074 | 500 | 2604 | 50 | 7 | 1 |
| 72 | 9311075 | 205 | 1108 | 28 | 1 | 4 |
| 73 | 9311076 | 216 | 1052 | 17 | 3 | 1 |
| 74 | 9311077 | 273 | 1359 | 35 | 2 | 3 |
| 75 | 9311078 | 184 | 720 | 16 | 3 | 2 |
| 76 | 9311079 | 336 | 1429 | 44 | 3 | 4 |
| 77 | 9311080 | 381 | 1752 | 48 | 4 | 4 |
| 78 | 9311081 | 252 | 1041 | 34 | 4 | 2 |
| 79 | 9311082 | 264 | 1241 | 34 | 3 | 5 |
| 80 | 9311083 | 334 | 1353 | 36 | 0 | 3 |
| 81 | 9311084 | 220 | 842 | 17 | 5 | 2 |
| 82 | 9311085 | 213 | 972 | 19 | 3 | 3 |
| 83 | 9311086 | 374 | 2000 | 48 | 11 | 5 |
| 84 | 9311087 | 164 | 691 | 17 | 3 | 1 |
| 85 | 9311088 | 199 | 796 | 15 | 1 | 5 |
| 86 | 9311089 | 267 | 1271 | 27 | 5 | 5 |
| 87 | 9311090 | 291 | 1267 | 39 | 1 | 4 |
| 88 | 9311091 | 222 | 1031 | 6 | 2 | 3 |
| 89 | 9311092 | 274 | 888 | 30 | 5 | 4 |
| 90 | 9311093 | 198 | 638 | 20 | 3 | 2 |
| 91 | 9311094 | 262 | 901 | 16 | 2 | 2 |
| 92 | 9311095 | 272 | 1026 | 29 | 4 | 5 |
| 93 | 9311096 | 280 | 978 | 49 | 3 | 4 |
| 94 | 9311097 | 305 | 1483 | 31 | 3 | 0 |
| 95 | 9311098 | 294 | 1201 | 30 | 4 | 2 |
| 96 | 9311099 | 288 | 1185 | 24 | 6 | 5 |
| 97 | 9311100 | 231 | 885 | 24 | 4 | 4 |
| 98 | 9311101 | 259 | 852 | 22 | 3 | 5 |
| 99 | 9311102 | 274 | 1234 | 22 | 1 | 2 |
| 100 | 9311103 | 227 | 892 | 15 | 3 | 4 |
| 101 | 9311104 | 326 | 1533 | 31 | 5 | 3 |
| 102 | 9311105 | 264 | 1184 | 41 | 6 | 3 |
| 103 | 9311106 | 254 | 1231 | 30 | 1 | 3 |
| 104 | 9311107 | 185 | 698 | 18 | 3 | 2 |
| 105 | 9311108 | 216 | 973 | 15 | 2 | 3 |
| 106 | 9311109 | 180 | 783 | 15 | 0 | 1 |
| 107 | 9311110 | 326 | 1222 | 43 | 5 | 4 |
| 108 | 9311111 | 283 | 1019 | 29 | 3 | 5 |
| 109 | 9311112 | 244 | 867 | 23 | 1 | 4 |
| 110 | 9311113 | 311 | 1733 | 27 | 7 | 3 |
| 111 | 9311114 | 267 | 891 | 34 | 5 | 5 |
| 112 | 9311115 | 271 | 1121 | 26 | 0 | 2 |
| 113 | 9311116 | 241 | 1079 | 13 | 4 | 3 |
| 114 | 9311117 | 220 | 905 | 9 | 3 | 3 |
| 115 | 9311118 | 301 | 1527 | 29 | 8 | 4 |
| 116 | 9311119 | 262 | 1208 | 20 | 4 | 5 |
| 117 | 9311120 | 417 | 1837 | 39 | 6 | 6 |
| 118 | 9311121 | 299 | 1048 | 20 | 1 | 4 |
| 119 | 9311122 | 284 | 1216 | 33 | 2 | 4 |
| 120 | 9311123 | 256 | 1067 | 23 | 1 | 3 |
| 121 | 9311124 | 303 | 1103 | 28 | 7 | 5 |
| 122 | 9311125 | 432 | 1117 | 32 | 2 | 5 |
| 123 | 9311126 | 321 | 1555 | 36 | 4 | 5 |
| 124 | 9311127 | 380 | 1965 | 36 | 2 | 3 |
| 125 | 9311128 | 274 | 1126 | 23 | 2 | 6 |
| 126 | 9311129 | 500 | 1826 | 38 | 0 | 4 |
| 127 | 9311130 | 254 | 1052 | 17 | 2 | 3 |
| 128 | 9311131 | 206 | 958 | 28 | 1 | 3 |
| 129 | 9311132 | 307 | 1064 | 29 | 2 | 3 |
| 130 | 9311133 | 271 | 1208 | 27 | 1 | 4 |
| 131 | 9311134 | 245 | 910 | 28 | 3 | 5 |
| 132 | 9311135 | 254 | 961 | 31 | 3 | 3 |
| 133 | 9311136 | 200 | 798 | 20 | 2 | 4 |
| 134 | 9311137 | 193 | 1012 | 29 | 5 | 1 |
| 135 | 9311138 | 267 | 1267 | 21 | 3 | 3 |
| 136 | 9311139 | 272 | 1107 | 35 | 6 | 2 |
| 137 | 9311140 | 270 | 858 | 28 | 1 | 4 |
| 138 | 9311141 | 170 | 660 | 16 | 0 | 2 |
| 139 | 9311142 | 219 | 1046 | 32 | 2 | 2 |
| 140 | 9311143 | 236 | 1018 | 23 | 1 | 3 |
| 141 | 9311144 | 273 | 1153 | 30 | 5 | 4 |
| 142 | 9311145 | 232 | 1027 | 29 | 5 | 3 |
| 143 | 9311146 | 231 | 917 | 21 | 4 | 1 |
| 144 | 9311147 | 214 | 987 | 25 | 4 | 3 |
| 145 | 9311148 | 258 | 977 | 30 | 4 | 3 |
| 146 | 9311149 | 259 | 1280 | 33 | 4 | 2 |
| 147 | 9311150 | 272 | 1293 | 39 | 4 | 4 |
| 148 | 9311151 | 248 | 1156 | 30 | 8 | 1 |
| 149 | 9311152 | 178 | 903 | 23 | 1 | 0 |
| 150 | 9311153 | 179 | 815 | 19 | 2 | 2 |
| 151 | 9311154 | 212 | 982 | 28 | 3 | 3 |
| 152 | 9311155 | 245 | 1195 | 31 | 1 | 4 |
| 153 | 9311156 | 246 | 1147 | 30 | 1 | 4 |
| 154 | 9311157 | 301 | 1473 | 35 | 1 | 3 |
| 155 | 9311158 | 258 | 1267 | 32 | 1 | 4 |
| 156 | 9311159 | 242 | 1176 | 35 | 5 | 3 |
| 157 | 9311160 | 238 | 1172 | 17 | 2 | 2 |
| 158 | 9311161 | 281 | 1366 | 35 | 2 | 4 |
| 159 | 9311162 | 264 | 1036 | 17 | 0 | 3 |
| 160 | 9311163 | 239 | 1196 | 30 | 3 | 1 |
| 161 | 9311164 | 239 | 1072 | 28 | 7 | 3 |
| 162 | 9311165 | 214 | 1103 | 24 | 8 | 2 |
| 163 | 9311166 | 323 | 1575 | 42 | 4 | 4 |
| 164 | 9311167 | 216 | 884 | 17 | 4 | 4 |
| 165 | 9311168 | 275 | 1274 | 35 | 5 | 4 |
| 166 | 9311169 | 219 | 1141 | 19 | 8 | 4 |
| 167 | 9311170 | 202 | 770 | 15 | 5 | 2 |
| 168 | 9311171 | 294 | 1125 | 38 | 3 | 4 |
| 169 | 9311172 | 242 | 812 | 28 | 7 | 5 |
| 170 | 9311173 | 246 | 1113 | 31 | 2 | 4 |
| 171 | 9311174 | 259 | 976 | 21 | 6 | 4 |
| 172 | 9311175 | 253 | 1231 | 37 | 3 | 3 |
| 173 | 9311176 | 236 | 1047 | 20 | 1 | 1 |
| 174 | 9311177 | 281 | 1233 | 27 | 1 | 4 |
| 175 | 9311178 | 227 | 861 | 30 | 3 | 1 |
| 176 | 9311179 | 240 | 856 | 19 | 7 | 3 |
| 177 | 9311180 | 234 | 876 | 27 | 5 | 5 |
| 178 | 9311181 | 259 | 1116 | 25 | 8 | 3 |
| 179 | 9311182 | 227 | 1004 | 29 | 5 | 3 |
| 180 | 9311183 | 211 | 942 | 27 | 1 | 2 |
| 181 | 9311184 | 223 | 980 | 21 | 1 | 2 |
| 182 | 9311185 | 230 | 873 | 14 | 4 | 5 |
| 183 | 9311186 | 279 | 1004 | 32 | 10 | 4 |
| 184 | 9311187 | 316 | 1215 | 37 | 9 | 4 |
| 185 | 9311188 | 178 | 671 | 22 | 3 | 4 |
| 186 | 9311189 | 205 | 901 | 19 | 3 | 2 |
| 187 | 9311190 | 145 | 408 | 9 | 1 | 1 |
| 188 | 9311191 | 331 | 1601 | 28 | 5 | 3 |
| 189 | 9311192 | 216 | 914 | 20 | 1 | 1 |
| 190 | 9311193 | 320 | 1273 | 27 | 4 | 3 |
| 191 | 9311194 | 292 | 990 | 28 | 4 | 3 |
| 192 | 9311195 | 280 | 1160 | 34 | 10 | 2 |
| 193 | 9311196 | 264 | 1329 | 20 | 6 | 2 |
| 194 | 9311197 | 339 | 1417 | 36 | 3 | 3 |
| 195 | 9311198 | 249 | 1109 | 28 | 2 | 4 |
| 196 | 9311199 | 211 | 847 | 17 | 9 | 3 |
| 197 | 9311200 | 270 | 1280 | 24 | 8 | 2 |
| 198 | 9311201 | 207 | 1074 | 28 | 4 | 1 |
| 199 | 9311202 | 366 | 1260 | 45 | 5 | 6 |
| 200 | 9311203 | 267 | 1173 | 27 | 3 | 4 |
| 201 | 9311204 | 297 | 1435 | 30 | 2 | 2 |
| 202 | 9311205 | 243 | 1115 | 26 | 4 | 5 |
| 203 | 9311206 | 244 | 1212 | 27 | 6 | 5 |
| 204 | 9311207 | 326 | 1287 | 21 | 6 | 6 |
| 205 | 9311208 | 248 | 1319 | 23 | 3 | 6 |
| 206 | 9311209 | 285 | 937 | 28 | 0 | 5 |
| 207 | 9311210 | 343 | 1691 | 27 | 4 | 4 |
| 208 | 9311211 | 304 | 1195 | 37 | 1 | 5 |
| 209 | 9311212 | 259 | 1069 | 31 | 2 | 3 |
| 210 | 9311213 | 202 | 754 | 16 | 5 | 3 |
| 211 | 9311214 | 255 | 1124 | 24 | 1 | 3 |
| 212 | 9311215 | 272 | 1136 | 21 | 1 | 2 |
| 213 | 9311216 | 257 | 1115 | 25 | 5 | 3 |
| 214 | 9311217 | 241 | 1116 | 15 | 5 | 4 |
| 215 | 9311218 | 214 | 1011 | 14 | 5 | 0 |
| 216 | 9311219 | 278 | 1400 | 44 | 4 | 4 |
| 217 | 9311220 | 227 | 1027 | 30 | 2 | 5 |
| 218 | 9311221 | 175 | 569 | 14 | 0 | 3 |
| 219 | 9311222 | 191 | 861 | 19 | 2 | 2 |
| 220 | 9311223 | 264 | 958 | 27 | 7 | 4 |
| 221 | 9311224 | 192 | 941 | 29 | 1 | 3 |
| 222 | 9311225 | 182 | 817 | 13 | 4 | 3 |
| 223 | 9311226 | 288 | 1432 | 33 | 0 | 5 |
| 224 | 9311227 | 252 | 1294 | 28 | 4 | 3 |
| 225 | 9311228 | 251 | 934 | 36 | 1 | 5 |
| 226 | 9311229 | 174 | 764 | 11 | 4 | 1 |
| 227 | 9311230 | 231 | 828 | 27 | 2 | 4 |
| 228 | 9311231 | 189 | 765 | 15 | 7 | 2 |
| 229 | 9311232 | 271 | 1267 | 40 | 2 | 3 |
| 230 | 9311233 | 260 | 1200 | 34 | 6 | 2 |
| 231 | 9311234 | 306 | 1264 | 25 | 1 | 3 |
| 232 | 9311235 | 184 | 821 | 19 | 1 | 3 |
| 233 | 9311236 | 176 | 714 | 25 | 4 | 1 |
| 234 | 9311237 | 243 | 1298 | 36 | 3 | 3 |
| 235 | 9311238 | 272 | 1227 | 22 | 5 | 2 |
| 236 | 9311239 | 188 | 842 | 16 | 2 | 1 |
| 237 | 9311240 | 253 | 1019 | 27 | 6 | 4 |
| 238 | 9311241 | 281 | 1251 | 31 | 4 | 2 |
| 239 | 9311242 | 275 | 1037 | 29 | 5 | 2 |
| 240 | 9311243 | 182 | 735 | 16 | 2 | 3 |
| 241 | 9311244 | 254 | 1146 | 30 | 7 | 4 |
| 242 | 9311245 | 261 | 1298 | 23 | 11 | 3 |
| 243 | 9311246 | 231 | 1023 | 25 | 4 | 3 |
| 244 | 9311247 | 473 | 2826 | 46 | 5 | 3 |
| 245 | 9311248 | 218 | 955 | 31 | 4 | 2 |
| 246 | 9311249 | 316 | 1707 | 22 | 8 | 4 |
| 247 | 9311250 | 270 | 1220 | 26 | 5 | 4 |
| 248 | 9311251 | 274 | 1070 | 46 | 2 | 6 |
| 249 | 9311252 | 263 | 1281 | 34 | 2 | 6 |
| 250 | 9311253 | 273 | 1334 | 23 | 3 | 3 |
| 251 | 9311254 | 276 | 1747 | 29 | 2 | 2 |
| 252 | 9311255 | 247 | 1014 | 22 | 2 | 4 |
| 253 | 9311256 | 224 | 1063 | 27 | 5 | 3 |
| 254 | 9311257 | 167 | 584 | 13 | 4 | 3 |
| 255 | 9311258 | 238 | 1069 | 28 | 5 | 3 |
| 256 | 9311259 | 230 | 881 | 17 | 1 | 4 |
| 257 | 9311260 | 201 | 718 | 15 | 3 | 3 |
| 258 | 9311261 | 266 | 1096 | 14 | 1 | 4 |
| 259 | 9311262 | 243 | 838 | 22 | 3 | 3 |
| 260 | 9311263 | 404 | 2005 | 31 | 5 | 3 |
| 261 | 9311264 | 215 | 609 | 21 | 1 | 4 |
| 262 | 9311265 | 268 | 821 | 20 | 6 | 4 |
| 263 | 9311266 | 259 | 979 | 14 | 0 | 3 |
| 264 | 9311267 | 227 | 1016 | 17 | 3 | 2 |
| 265 | 9311268 | 237 | 1026 | 26 | 3 | 5 |
| 266 | 9311269 | 251 | 998 | 35 | 7 | 4 |
| 267 | 9311270 | 246 | 1275 | 25 | 4 | 4 |
| 268 | 9311271 | 182 | 769 | 15 | 2 | 3 |
| 269 | 9311272 | 208 | 998 | 23 | 2 | 3 |
| 270 | 9311273 | 279 | 1233 | 33 | 6 | 4 |
| 271 | 9311274 | 214 | 835 | 26 | 3 | 1 |
| 272 | 9311275 | 314 | 1078 | 21 | 1 | 6 |
| 273 | 9311276 | 204 | 800 | 19 | 6 | 4 |
| 274 | 9311277 | 245 | 845 | 29 | 3 | 4 |
| 275 | 9311278 | 211 | 942 | 17 | 7 | 3 |
| 276 | 9311279 | 174 | 680 | 14 | 8 | 1 |
| 277 | 9311280 | 252 | 1139 | 26 | 1 | 5 |
| 278 | 9311281 | 311 | 1356 | 40 | 4 | 5 |
| 279 | 9311282 | 373 | 1483 | 41 | 9 | 3 |
| 280 | 9311283 | 236 | 953 | 26 | 5 | 3 |
| 281 | 9311284 | 264 | 1134 | 17 | 4 | 3 |
| 282 | 9311285 | 234 | 1071 | 22 | 1 | 4 |
| 283 | 9311286 | 262 | 1148 | 22 | 3 | 2 |
| 284 | 9311287 | 226 | 721 | 21 | 4 | 4 |
| 285 | 9311288 | 259 | 1239 | 33 | 7 | 3 |
| 286 | 9311289 | 206 | 813 | 24 | 7 | 2 |
| 287 | 9311290 | 246 | 1106 | 19 | 2 | 4 |
| 288 | 9311291 | 249 | 1143 | 26 | 2 | 4 |
| 289 | 9311292 | 321 | 1322 | 31 | 2 | 3 |
| 290 | 9311293 | 164 | 703 | 10 | 5 | 0 |
| 291 | 9311294 | 236 | 931 | 23 | 0 | 3 |
| 292 | 9311295 | 242 | 1024 | 16 | 2 | 4 |
| 293 | 9311296 | 268 | 1191 | 28 | 3 | 3 |
| 294 | 9311297 | 233 | 953 | 38 | 7 | 5 |
| 295 | 9311298 | 238 | 968 | 30 | 4 | 5 |
| 296 | 9311299 | 244 | 1212 | 30 | 1 | 2 |
| 297 | 9311300 | 180 | 474 | 16 | 3 | 3 |
| 298 | 9311301 | 213 | 837 | 7 | 2 | 4 |
| 299 | 9311302 | 246 | 913 | 14 | 5 | 5 |
| 300 | 9311303 | 185 | 681 | 13 | 6 | 4 |
| 301 | 9311304 | 217 | 897 | 22 | 3 | 3 |
| 302 | 9311305 | 307 | 1347 | 25 | 5 | 2 |
| 303 | 9311306 | 290 | 1223 | 28 | 3 | 4 |
| 304 | 9311307 | 418 | 1326 | 29 | 6 | 3 |
| 305 | 9311308 | 289 | 1323 | 28 | 2 | 4 |
| 306 | 9311309 | 261 | 1027 | 27 | 5 | 5 |
| 307 | 9311310 | 257 | 987 | 34 | 2 | 6 |
| 308 | 9311311 | 203 | 1051 | 24 | 2 | 4 |
| 309 | 9311312 | 257 | 1164 | 24 | 4 | 4 |
| 310 | 9311313 | 272 | 918 | 27 | 2 | 6 |
| 311 | 9311314 | 152 | 535 | 16 | 0 | 4 |
| 312 | 9311315 | 241 | 1198 | 32 | 2 | 2 |
| 313 | 9311316 | 186 | 738 | 22 | 3 | 3 |
| 314 | 9311317 | 207 | 877 | 25 | 2 | 1 |
| 315 | 9311318 | 200 | 908 | 20 | 1 | 4 |
| 316 | 9311319 | 255 | 1045 | 31 | 2 | 2 |
| 317 | 9311320 | 287 | 1395 | 42 | 2 | 3 |
| 318 | 9311321 | 169 | 598 | 20 | 3 | 3 |
| 319 | 9311322 | 275 | 1234 | 27 | 1 | 3 |
| 320 | 9311323 | 230 | 1092 | 32 | 2 | 2 |
| 321 | 9311324 | 364 | 1731 | 61 | 4 | 4 |
| 322 | 9311325 | 218 | 1015 | 27 | 2 | 4 |
| 323 | 9311326 | 262 | 1176 | 24 | 0 | 3 |
| 324 | 9311327 | 211 | 659 | 13 | 2 | 0 |
| 325 | 9311328 | 211 | 1150 | 18 | 3 | 3 |
| 326 | 9311329 | 204 | 1088 | 37 | 5 | 3 |
| 327 | 9311330 | 214 | 908 | 33 | 5 | 3 |
| 328 | 9311331 | 216 | 863 | 19 | 4 | 3 |
| 329 | 9311332 | 220 | 1224 | 33 | 3 | 4 |
| 330 | 9311333 | 255 | 1436 | 12 | 2 | 2 |
| 331 | 9311334 | 230 | 974 | 22 | 2 | 2 |
| 332 | 9311335 | 224 | 883 | 20 | 3 | 5 |
| 333 | 9311336 | 307 | 1693 | 28 | 1 | 6 |
| 334 | 9311337 | 203 | 817 | 20 | 6 | 2 |
| 335 | 9311338 | 500 | 1985 | 41 | 3 | 3 |
| 336 | 9311339 | 341 | 1539 | 22 | 5 | 2 |
| 337 | 9311340 | 169 | 774 | 13 | 2 | 1 |
| 338 | 9311341 | 221 | 1036 | 15 | 5 | 5 |
| 339 | 9311342 | 236 | 1289 | 22 | 4 | 3 |
| 340 | 9311343 | 234 | 1089 | 24 | 2 | 4 |
| 341 | 9311344 | 237 | 996 | 27 | 4 | 3 |
| 342 | 9311345 | 307 | 1285 | 38 | 3 | 5 |
| 343 | 9311346 | 213 | 932 | 27 | 4 | 1 |
| 344 | 9311347 | 173 | 675 | 15 | 3 | 1 |
| 345 | 9311348 | 332 | 1413 | 43 | 2 | 6 |
| 346 | 9311349 | 259 | 1027 | 26 | 3 | 3 |
| 347 | 9311350 | 217 | 859 | 15 | 6 | 2 |
| 348 | 9311351 | 233 | 991 | 24 | 0 | 3 |
| 349 | 9311352 | 225 | 580 | 20 | 2 | 1 |
| 350 | 9311353 | 234 | 924 | 12 | 3 | 4 |
| 351 | 9311354 | 298 | 1175 | 44 | 4 | 4 |
| 352 | 9311355 | 241 | 850 | 21 | 1 | 2 |
| 353 | 9311356 | 227 | 841 | 6 | 4 | 4 |
| 354 | 9311357 | 261 | 943 | 13 | 2 | 4 |
| 355 | 9311358 | 310 | 1251 | 26 | 5 | 3 |
| 356 | 9311359 | 276 | 1013 | 34 | 9 | 3 |
| 357 | 9311360 | 274 | 944 | 26 | 3 | 3 |
| 358 | 9311361 | 266 | 1130 | 28 | 2 | 3 |
| 359 | 9311362 | 250 | 1103 | 23 | 2 | 5 |
| 360 | 9311363 | 282 | 1308 | 35 | 5 | 4 |
| 361 | 9311364 | 237 | 973 | 17 | 5 | 2 |
| 362 | 9311365 | 251 | 1197 | 29 | 3 | 4 |
| 363 | 9311366 | 234 | 710 | 27 | 1 | 2 |
| 364 | 9311367 | 360 | 1677 | 46 | 1 | 5 |
| 365 | 9311368 | 182 | 657 | 16 | 6 | 2 |
| 366 | 9311369 | 205 | 886 | 13 | 0 | 3 |
| 367 | 9311370 | 346 | 1413 | 43 | 1 | 6 |
| 368 | 9311371 | 212 | 882 | 20 | 5 | 3 |
| 369 | 9311372 | 171 | 593 | 12 | 2 | 2 |
| 370 | 9311373 | 255 | 944 | 27 | 2 | 3 |
| 371 | 9311374 | 280 | 1229 | 37 | 5 | 4 |
| 372 | 9311375 | 219 | 1125 | 24 | 5 | 3 |
| 373 | 9311376 | 257 | 918 | 26 | 4 | 4 |
| 374 | 9311377 | 251 | 1049 | 23 | 2 | 5 |
| 375 | 9311378 | 210 | 1097 | 26 | 6 | 3 |
| 376 | 9311379 | 209 | 842 | 19 | 1 | 4 |
| 377 | 9311380 | 237 | 1313 | 20 | 3 | 2 |
| 378 | 9311381 | 238 | 826 | 27 | 14 | 4 |
| 379 | 9311382 | 315 | 1252 | 36 | 5 | 4 |
| 380 | 9311383 | 221 | 797 | 22 | 4 | 3 |
| 381 | 9311384 | 226 | 1235 | 26 | 4 | 2 |
| 382 | 9311385 | 209 | 830 | 15 | 2 | 2 |
| 383 | 9311386 | 261 | 1196 | 19 | 1 | 3 |
| 384 | 9311387 | 256 | 966 | 26 | 1 | 6 |
| 385 | 9311388 | 270 | 1455 | 26 | 6 | 4 |
| 386 | 9311389 | 278 | 1376 | 25 | 1 | 2 |
| 387 | 9311390 | 219 | 1041 | 31 | 5 | 4 |
| 388 | 9311391 | 267 | 1254 | 18 | 2 | 3 |
| 389 | 9311392 | 236 | 852 | 21 | 4 | 3 |
| 390 | 9311393 | 218 | 754 | 18 | 1 | 4 |
| 391 | 9311394 | 248 | 935 | 29 | 5 | 5 |
| 392 | 9311395 | 214 | 950 | 25 | 4 | 0 |
| 393 | 9311396 | 180 | 791 | 17 | 4 | 3 |
| 394 | 9311397 | 402 | 2220 | 56 | 1 | 2 |
| 395 | 9311398 | 244 | 1010 | 21 | 0 | 3 |
| 396 | 9311399 | 243 | 961 | 26 | 7 | 3 |
| 397 | 9311400 | 232 | 978 | 24 | 1 | 4 |
| 398 | 9311401 | 259 | 988 | 24 | 2 | 5 |
| 399 | 9311402 | 328 | 1455 | 31 | 1 | 4 |
| 400 | 9311403 | 249 | 879 | 22 | 3 | 2 |
| 401 | 9311404 | 239 | 1020 | 23 | 2 | 1 |
| 402 | 9311405 | 296 | 939 | 26 | 0 | 5 |
| 403 | 9311406 | 218 | 1001 | 25 | 2 | 3 |
| 404 | 9311407 | 261 | 1286 | 28 | 7 | 3 |
| 405 | 9311408 | 273 | 1204 | 45 | 2 | 4 |
| 406 | 9311409 | 313 | 1340 | 46 | 3 | 4 |
| 407 | 9311410 | 201 | 885 | 17 | 5 | 3 |
| 408 | 9311411 | 264 | 885 | 27 | 2 | 5 |
| 409 | 9311412 | 225 | 1015 | 19 | 1 | 1 |
| 410 | 9311413 | 189 | 924 | 19 | 3 | 1 |
| 411 | 9311414 | 229 | 974 | 22 | 3 | 1 |
| 412 | 9311415 | 266 | 1047 | 32 | 1 | 3 |
| 413 | 9311416 | 241 | 1189 | 25 | 3 | 4 |
| 414 | 9311417 | 257 | 1005 | 28 | 4 | 4 |
| 415 | 9311418 | 228 | 985 | 29 | 1 | 3 |
| 416 | 9311419 | 237 | 1206 | 13 | 10 | 4 |
| 417 | 9311420 | 314 | 1663 | 48 | 5 | 3 |
| 418 | 9311421 | 214 | 913 | 9 | 3 | 5 |
| 419 | 9311422 | 277 | 1267 | 32 | 4 | 4 |
| 420 | 9311423 | 167 | 630 | 11 | 4 | 1 |
| 421 | 9311424 | 215 | 1026 | 20 | 5 | 2 |
| 422 | 9311425 | 243 | 1175 | 32 | 4 | 3 |
| 423 | 9311426 | 277 | 1282 | 36 | 0 | 5 |
| 424 | 9311427 | 252 | 1337 | 26 | 3 | 2 |
| 425 | 9311428 | 214 | 1053 | 31 | 2 | 4 |
| 426 | 9311429 | 209 | 979 | 24 | 6 | 2 |
| 427 | 9311430 | 332 | 1365 | 34 | 5 | 5 |
| 428 | 9311431 | 248 | 1198 | 30 | 6 | 3 |
| 429 | 9311432 | 243 | 1111 | 23 | 8 | 3 |
| 430 | 9311433 | 305 | 1460 | 31 | 13 | 4 |
| 431 | 9311434 | 212 | 1038 | 27 | 6 | 3 |
| 432 | 9311435 | 183 | 655 | 18 | 0 | 3 |
| 433 | 9311436 | 212 | 765 | 21 | 1 | 3 |
| 434 | 9311437 | 211 | 852 | 20 | 4 | 3 |
| 435 | 9311438 | 218 | 838 | 21 | 8 | 4 |
| 436 | 9311439 | 257 | 1194 | 18 | 5 | 2 |
| 437 | 9311440 | 210 | 881 | 13 | 2 | 2 |
| 438 | 9311441 | 272 | 833 | 24 | 0 | 4 |
| 439 | 9311442 | 274 | 1312 | 16 | 4 | 3 |
| 440 | 9311443 | 262 | 1098 | 29 | 4 | 5 |
| 441 | 9311444 | 223 | 847 | 25 | 4 | 3 |
| 442 | 9311445 | 207 | 1053 | 34 | 5 | 1 |
| 443 | 9311446 | 381 | 1988 | 38 | 12 | 2 |
| 444 | 9311447 | 270 | 1144 | 36 | 1 | 4 |
| 445 | 9311448 | 207 | 892 | 8 | 1 | 2 |
| 446 | 9311449 | 234 | 1116 | 21 | 5 | 3 |
| 447 | 9311450 | 239 | 1096 | 17 | 3 | 3 |
| 448 | 9311451 | 275 | 1133 | 25 | 6 | 4 |
| 449 | 9311452 | 190 | 857 | 17 | 4 | 2 |
| 450 | 9311453 | 252 | 960 | 26 | 2 | 4 |
| 451 | 9311454 | 234 | 879 | 18 | 3 | 4 |
| 452 | 9311455 | 219 | 934 | 22 | 3 | 2 |
| 453 | 9311456 | 261 | 1211 | 27 | 4 | 3 |
| 454 | 9311457 | 194 | 823 | 19 | 0 | 5 |
| 455 | 9311458 | 210 | 772 | 18 | 6 | 3 |
| 456 | 9311459 | 268 | 887 | 23 | 3 | 3 |
| 457 | 9311460 | 232 | 1312 | 12 | 3 | 2 |
| 458 | 9311461 | 216 | 733 | 14 | 8 | 3 |
| 459 | 9311462 | 199 | 736 | 27 | 1 | 5 |
| 460 | 9311463 | 194 | 837 | 16 | 4 | 2 |
| 461 | 9311464 | 183 | 902 | 14 | 3 | 1 |
| 462 | 9311465 | 278 | 1080 | 29 | 1 | 4 |
| 463 | 9311466 | 247 | 979 | 34 | 4 | 4 |
| 464 | 9311467 | 237 | 949 | 20 | 0 | 4 |
| 465 | 9311468 | 212 | 742 | 16 | 3 | 3 |
| 466 | 9311469 | 262 | 821 | 11 | 1 | 6 |
| 467 | 9311470 | 250 | 1202 | 16 | 2 | 4 |
| 468 | 9311471 | 317 | 1663 | 18 | 6 | 4 |
| 469 | 9311472 | 244 | 1035 | 21 | 2 | 4 |
| 470 | 9311473 | 257 | 892 | 19 | 1 | 3 |
| 471 | 9311474 | 308 | 1467 | 35 | 0 | 3 |
| 472 | 9311475 | 441 | 1239 | 42 | 1 | 3 |
| 473 | 9311476 | 266 | 1034 | 25 | 4 | 4 |
| 474 | 9311477 | 309 | 1631 | 39 | 3 | 3 |
| 475 | 9311478 | 209 | 1077 | 21 | 2 | 3 |
| 476 | 9311479 | 186 | 880 | 19 | 3 | 1 |
| 477 | 9311480 | 275 | 1320 | 38 | 2 | 3 |
| 478 | 9311481 | 253 | 910 | 24 | 1 | 3 |
| 479 | 9311482 | 255 | 1172 | 25 | 1 | 3 |
| 480 | 9311483 | 204 | 742 | 24 | 4 | 4 |
| 481 | 9311484 | 201 | 691 | 26 | 4 | 3 |
| 482 | 9311485 | 336 | 1552 | 32 | 2 | 6 |
| 483 | 9311486 | 354 | 1715 | 41 | 2 | 4 |
| 484 | 9311487 | 265 | 1133 | 32 | 2 | 3 |
| 485 | 9311488 | 264 | 1192 | 25 | 2 | 3 |
| 486 | 9311489 | 438 | 2014 | 63 | 4 | 4 |
| 487 | 9311490 | 390 | 1504 | 48 | 1 | 3 |
| 488 | 9311491 | 242 | 812 | 31 | 3 | 3 |
| 489 | 9311492 | 281 | 1337 | 29 | 5 | 4 |
| 490 | 9311493 | 164 | 497 | 21 | 3 | 4 |
| 491 | 9311494 | 249 | 1128 | 29 | 0 | 2 |
| 492 | 9311495 | 278 | 1281 | 29 | 6 | 2 |
| 493 | 9311496 | 202 | 813 | 16 | 4 | 3 |
| 494 | 9311497 | 250 | 950 | 29 | 3 | 4 |
| 495 | 9311498 | 274 | 1606 | 36 | 4 | 4 |
| 496 | 9311499 | 257 | 1248 | 37 | 6 | 1 |
| 497 | 9311500 | 258 | 1134 | 41 | 7 | 3 |
| 498 | 9311501 | 236 | 965 | 25 | 7 | 1 |
| 499 | 9311502 | 212 | 954 | 24 | 3 | 3 |
| 500 | 9311503 | 271 | 1426 | 34 | 10 | 4 |
| 501 | 9311504 | 306 | 1363 | 25 | 3 | 3 |
| 502 | 9311505 | 278 | 1340 | 40 | 6 | 3 |
| 503 | 9311506 | 186 | 644 | 17 | 1 | 2 |
| 504 | 9311507 | 253 | 1178 | 25 | 2 | 3 |
| 505 | 9311508 | 177 | 778 | 13 | 3 | 0 |
| 506 | 9311509 | 187 | 948 | 22 | 3 | 2 |
| 507 | 9311510 | 317 | 1373 | 43 | 2 | 4 |
| 508 | 9311511 | 268 | 1079 | 39 | 3 | 2 |
| 509 | 9311512 | 327 | 1490 | 38 | 3 | 3 |
| 510 | 9311513 | 205 | 798 | 17 | 3 | 4 |
| 511 | 9311514 | 237 | 960 | 27 | 3 | 2 |
| 512 | 9311515 | 241 | 1145 | 22 | 8 | 2 |
| 513 | 9311516 | 315 | 1469 | 35 | 1 | 3 |
| 514 | 9311517 | 214 | 996 | 21 | 0 | 3 |
| 515 | 9311518 | 232 | 1039 | 23 | 0 | 3 |
| 516 | 9311519 | 298 | 1356 | 40 | 3 | 4 |
| 517 | 9311520 | 291 | 1288 | 23 | 7 | 5 |
| 518 | 9311521 | 226 | 854 | 20 | 1 | 2 |
| 519 | 9311522 | 229 | 1085 | 31 | 1 | 3 |
| 520 | 9311523 | 212 | 878 | 24 | 0 | 3 |
| 521 | 9311524 | 308 | 1597 | 38 | 3 | 5 |
| 522 | 9311525 | 196 | 743 | 17 | 2 | 1 |
| 523 | 9311526 | 203 | 871 | 12 | 1 | 3 |
| 524 | 9311527 | 260 | 1058 | 32 | 0 | 4 |
| 525 | 9311528 | 197 | 899 | 15 | 3 | 3 |
| 526 | 9311529 | 310 | 1630 | 42 | 0 | 1 |
| 527 | 9311530 | 356 | 1182 | 42 | 6 | 5 |
| 528 | 9311531 | 298 | 1156 | 35 | 4 | 5 |
| 529 | 9311532 | 177 | 646 | 17 | 1 | 1 |
| 530 | 9311533 | 193 | 672 | 19 | 4 | 4 |
| 531 | 9311534 | 240 | 851 | 18 | 4 | 3 |
| 532 | 9311535 | 217 | 965 | 22 | 3 | 1 |
| 533 | 9311536 | 357 | 1398 | 34 | 1 | 3 |
| 534 | 9311537 | 242 | 967 | 19 | 2 | 4 |
| 535 | 9311538 | 282 | 1138 | 21 | 4 | 5 |
| 536 | 9311539 | 198 | 872 | 16 | 2 | 3 |
| 537 | 9311540 | 214 | 997 | 27 | 3 | 4 |
| 538 | 9311541 | 197 | 577 | 16 | 2 | 3 |
| 539 | 9311542 | 249 | 894 | 23 | 4 | 3 |
| 540 | 9311543 | 176 | 673 | 9 | 4 | 2 |
| 541 | 9311544 | 176 | 668 | 12 | 0 | 3 |
| 542 | 9311545 | 188 | 731 | 15 | 3 | 3 |
| 543 | 9311546 | 236 | 1091 | 33 | 6 | 2 |
| 544 | 9311547 | 223 | 819 | 28 | 6 | 5 |
| 545 | 9311548 | 303 | 1593 | 34 | 11 | 2 |
| 546 | 9311549 | 163 | 788 | 8 | 9 | 1 |
| 547 | 9311550 | 182 | 673 | 19 | 4 | 3 |
| 548 | 9311551 | 262 | 868 | 31 | 3 | 5 |
| 549 | 9311552 | 266 | 1076 | 42 | 1 | 3 |
| 550 | 9311553 | 197 | 931 | 20 | 5 | 3 |
| 551 | 9311554 | 224 | 926 | 23 | 0 | 2 |
| 552 | 9311555 | 219 | 711 | 22 | 2 | 3 |
| 553 | 9311556 | 189 | 886 | 25 | 2 | 5 |
| 554 | 9311557 | 256 | 882 | 30 | 12 | 4 |
| 555 | 9311558 | 218 | 1128 | 19 | 2 | 3 |
| 556 | 9311559 | 220 | 1033 | 24 | 2 | 4 |
| 557 | 9311560 | 200 | 847 | 19 | 2 | 2 |
| 558 | 9311561 | 235 | 797 | 20 | 3 | 4 |
| 559 | 9311562 | 282 | 1296 | 27 | 2 | 2 |
| 560 | 9311563 | 214 | 1213 | 30 | 4 | 2 |
| 561 | 9311564 | 297 | 1107 | 29 | 2 | 3 |
| 562 | 9311565 | 260 | 1059 | 18 | 0 | 2 |
| 563 | 9311566 | 241 | 966 | 22 | 1 | 5 |
| 564 | 9311567 | 328 | 1435 | 19 | 3 | 2 |
| 565 | 9311568 | 260 | 1092 | 33 | 3 | 3 |
| 566 | 9311569 | 227 | 859 | 27 | 1 | 3 |
| 567 | 9311570 | 267 | 1194 | 24 | 4 | 3 |
| 568 | 9311571 | 197 | 995 | 18 | 2 | 3 |
| 569 | 9311572 | 250 | 1176 | 35 | 0 | 5 |
| 570 | 9311573 | 240 | 1242 | 37 | 0 | 5 |
| 571 | 9311574 | 323 | 1163 | 24 | 2 | 4 |
| 572 | 9311575 | 229 | 1033 | 24 | 3 | 3 |
| 573 | 9311576 | 229 | 1013 | 30 | 8 | 2 |
| 574 | 9311577 | 204 | 1050 | 19 | 1 | 2 |
| 575 | 9311578 | 198 | 799 | 22 | 5 | 3 |
| 576 | 9311579 | 234 | 787 | 22 | 3 | 3 |
| 577 | 9311580 | 256 | 1089 | 36 | 6 | 4 |
| 578 | 9311581 | 195 | 875 | 16 | 4 | 2 |
| 579 | 9311582 | 179 | 674 | 14 | 2 | 2 |
| 580 | 9311583 | 313 | 1267 | 36 | 3 | 3 |
| 581 | 9311584 | 284 | 1311 | 25 | 4 | 2 |
| 582 | 9311585 | 403 | 1716 | 45 | 8 | 5 |
| 583 | 9311586 | 215 | 735 | 18 | 0 | 2 |
| 584 | 9311587 | 230 | 1103 | 30 | 1 | 3 |
| 585 | 9311588 | 238 | 1126 | 34 | 4 | 3 |
| 586 | 9311589 | 238 | 899 | 21 | 4 | 1 |
| 587 | 9311590 | 245 | 710 | 16 | 8 | 2 |
| 588 | 9311591 | 215 | 1001 | 21 | 2 | 3 |
| 589 | 9311592 | 203 | 847 | 22 | 5 | 5 |
| 590 | 9311593 | 241 | 1276 | 31 | 4 | 2 |
| 591 | 9311594 | 247 | 1277 | 38 | 3 | 2 |
| 592 | 9311595 | 218 | 1243 | 22 | 9 | 4 |
| 593 | 9311596 | 253 | 1195 | 9 | 3 | 2 |
| 594 | 9311597 | 254 | 1103 | 27 | 3 | 3 |
| 595 | 9311598 | 271 | 1136 | 33 | 10 | 4 |
| 596 | 9311599 | 272 | 1305 | 32 | 3 | 1 |
| 597 | 9311600 | 244 | 1093 | 21 | 3 | 3 |
| 598 | 9311601 | 251 | 1084 | 25 | 2 | 2 |
| 599 | 9311602 | 230 | 1125 | 27 | 4 | 4 |
| 600 | 9311603 | 221 | 885 | 24 | 5 | 4 |
| 601 | 9311604 | 190 | 832 | 20 | 4 | 4 |
| 602 | 9311605 | 256 | 1166 | 21 | 3 | 3 |
| 603 | 9311606 | 198 | 932 | 21 | 4 | 2 |
| 604 | 9311607 | 293 | 1515 | 18 | 1 | 3 |
| 605 | 9311608 | 254 | 1243 | 37 | 3 | 4 |
| 606 | 9311609 | 140 | 565 | 9 | 3 | 2 |
| 607 | 9311610 | 229 | 1290 | 35 | 13 | 4 |
| 608 | 9311611 | 225 | 798 | 25 | 2 | 3 |
| 609 | 9311612 | 272 | 1348 | 22 | 1 | 3 |
| 610 | 9311613 | 184 | 660 | 20 | 3 | 3 |
| 611 | 9311614 | 212 | 921 | 19 | 2 | 3 |
| 612 | 9311615 | 443 | 1684 | 38 | 13 | 4 |
| 613 | 9311616 | 229 | 1070 | 28 | 3 | 3 |
| 614 | 9311617 | 295 | 1264 | 31 | 1 | 4 |
| 615 | 9311618 | 321 | 1428 | 37 | 4 | 5 |
| 616 | 9311619 | 232 | 896 | 12 | 4 | 2 |
| 617 | 9311620 | 403 | 1967 | 41 | 17 | 2 |
| 618 | 9311621 | 312 | 1308 | 38 | 4 | 4 |
| 619 | 9311622 | 182 | 839 | 23 | 3 | 1 |
| 620 | 9311623 | 262 | 1021 | 33 | 5 | 3 |
| 621 | 9311624 | 255 | 974 | 19 | 4 | 2 |
| 622 | 9311625 | 225 | 977 | 32 | 3 | 2 |
| 623 | 9311626 | 198 | 639 | 14 | 4 | 2 |
| 624 | 9311627 | 187 | 763 | 17 | 6 | 2 |
| 625 | 9311628 | 229 | 756 | 24 | 5 | 4 |
| 626 | 9311629 | 233 | 883 | 13 | 7 | 3 |
| 627 | 9311630 | 329 | 1681 | 43 | 5 | 4 |
| 628 | 9311631 | 281 | 1387 | 42 | 8 | 4 |
| 629 | 9311632 | 228 | 842 | 23 | 3 | 4 |
| 630 | 9311633 | 210 | 989 | 26 | 0 | 3 |
| 631 | 9311634 | 239 | 1285 | 29 | 2 | 3 |
| 632 | 9311635 | 237 | 1091 | 22 | 1 | 3 |
| 633 | 9311636 | 225 | 840 | 18 | 1 | 4 |
| 634 | 9311637 | 233 | 672 | 20 | 8 | 6 |
| 635 | 9311638 | 226 | 801 | 24 | 3 | 3 |
| 636 | 9311639 | 263 | 970 | 25 | 5 | 1 |
| 637 | 9311640 | 290 | 1210 | 38 | 5 | 5 |
| 638 | 9311641 | 267 | 944 | 31 | 3 | 3 |
| 639 | 9311642 | 255 | 945 | 27 | 4 | 2 |
| 640 | 9311643 | 233 | 924 | 12 | 3 | 5 |
| 641 | 9311644 | 188 | 937 | 25 | 1 | 2 |
| 642 | 9311645 | 176 | 609 | 13 | 1 | 5 |
| 643 | 9311646 | 254 | 960 | 13 | 8 | 3 |
| 644 | 9311647 | 251 | 870 | 20 | 3 | 5 |
| 645 | 9311648 | 224 | 1299 | 31 | 4 | 3 |
| 646 | 9311649 | 201 | 742 | 15 | 0 | 2 |
| 647 | 9311650 | 225 | 1031 | 31 | 0 | 4 |
| 648 | 9311651 | 241 | 1311 | 19 | 1 | 4 |
| 649 | 9311652 | 267 | 1197 | 31 | 6 | 4 |
| 650 | 9311653 | 204 | 912 | 21 | 6 | 3 |
| 651 | 9311654 | 226 | 732 | 25 | 3 | 4 |
| 652 | 9311655 | 323 | 1205 | 21 | 8 | 6 |
| 653 | 9311656 | 314 | 1448 | 26 | 3 | 6 |
| 654 | 9311657 | 303 | 1501 | 41 | 3 | 3 |
| 655 | 9311658 | 198 | 853 | 24 | 3 | 3 |
| 656 | 9311659 | 220 | 1111 | 23 | 2 | 3 |
| 657 | 9311660 | 181 | 570 | 14 | 3 | 3 |
| 658 | 9311661 | 268 | 1052 | 35 | 4 | 3 |
| 659 | 9311662 | 208 | 1077 | 29 | 3 | 4 |
| 660 | 9311663 | 269 | 1051 | 28 | 1 | 5 |
| 661 | 9311664 | 264 | 940 | 20 | 0 | 2 |
| 662 | 9311665 | 247 | 1126 | 21 | 2 | 2 |
| 663 | 9311666 | 222 | 762 | 21 | 4 | 4 |
| 664 | 9311667 | 193 | 857 | 28 | 0 | 3 |
| 665 | 9311668 | 220 | 1036 | 29 | 1 | 2 |
| 666 | 9311669 | 191 | 763 | 26 | 0 | 3 |
| 667 | 9311670 | 202 | 890 | 13 | 1 | 3 |
| 668 | 9311671 | 320 | 1254 | 46 | 5 | 3 |
| 669 | 9311672 | 212 | 843 | 17 | 1 | 3 |
| 670 | 9311673 | 212 | 618 | 13 | 4 | 2 |
| 671 | 9311674 | 416 | 1989 | 57 | 2 | 3 |
| 672 | 9311675 | 189 | 788 | 21 | 0 | 3 |
| 673 | 9311676 | 220 | 1255 | 19 | 3 | 1 |
| 674 | 9311677 | 173 | 554 | 14 | 4 | 1 |
| 675 | 9311678 | 218 | 1075 | 31 | 2 | 4 |
| 676 | 9311679 | 222 | 1016 | 22 | 8 | 4 |
| 677 | 9311680 | 286 | 1385 | 31 | 7 | 2 |
| 678 | 9311681 | 213 | 954 | 24 | 5 | 2 |
| 679 | 9311682 | 205 | 695 | 19 | 6 | 2 |
| 680 | 9311683 | 283 | 1299 | 26 | 7 | 3 |
| 681 | 9311684 | 240 | 1157 | 21 | 4 | 3 |
| 682 | 9311685 | 255 | 998 | 25 | 3 | 4 |
| 683 | 9311686 | 228 | 875 | 25 | 7 | 4 |
| 684 | 9311687 | 209 | 726 | 15 | 6 | 2 |
| 685 | 9311688 | 257 | 1141 | 24 | 2 | 5 |
| 686 | 9311689 | 225 | 991 | 21 | 0 | 2 |
| 687 | 9311690 | 278 | 1118 | 26 | 5 | 5 |
| 688 | 9311691 | 252 | 1160 | 23 | 3 | 3 |
| 689 | 9311692 | 249 | 1366 | 22 | 8 | 2 |
| 690 | 9311693 | 234 | 797 | 21 | 4 | 2 |
| 691 | 9311694 | 198 | 1006 | 20 | 5 | 3 |
| 692 | 9311695 | 229 | 1050 | 14 | 7 | 2 |
| 693 | 9311696 | 301 | 1533 | 30 | 2 | 4 |
| 694 | 9311697 | 225 | 1084 | 13 | 4 | 4 |
| 695 | 9311698 | 211 | 852 | 17 | 3 | 2 |
| 696 | 9311699 | 251 | 788 | 11 | 1 | 5 |
| 697 | 9311700 | 197 | 893 | 26 | 0 | 1 |
| 698 | 9311701 | 205 | 851 | 13 | 4 | 2 |
| 699 | 9311702 | 223 | 1184 | 12 | 2 | 3 |
| 700 | 9311703 | 365 | 1576 | 21 | 2 | 6 |
| 701 | 9311704 | 203 | 883 | 18 | 2 | 1 |
| 702 | 9311705 | 311 | 1173 | 32 | 1 | 5 |
| 703 | 9311706 | 234 | 848 | 23 | 4 | 5 |
| 704 | 9311707 | 243 | 918 | 18 | 4 | 5 |
| 705 | 9311708 | 230 | 901 | 25 | 4 | 2 |
| 706 | 9311709 | 204 | 956 | 14 | 3 | 2 |
| 707 | 9311710 | 235 | 990 | 26 | 3 | 3 |
| 708 | 9311711 | 281 | 974 | 32 | 2 | 4 |
| 709 | 9311712 | 181 | 791 | 18 | 1 | 1 |
| 710 | 9311713 | 214 | 766 | 18 | 4 | 2 |
| 711 | 9311714 | 297 | 1586 | 39 | 4 | 3 |
| 712 | 9311715 | 270 | 1005 | 28 | 3 | 5 |
| 713 | 9311716 | 439 | 1607 | 38 | 4 | 6 |
| 714 | 9311717 | 215 | 561 | 16 | 1 | 2 |
| 715 | 9311718 | 273 | 1129 | 41 | 2 | 4 |
| 716 | 9311719 | 220 | 982 | 24 | 2 | 3 |
| 717 | 9311720 | 368 | 1422 | 41 | 9 | 5 |
| 718 | 9311721 | 188 | 809 | 22 | 6 | 2 |
| 719 | 9311722 | 235 | 1172 | 28 | 3 | 3 |
| 720 | 9311723 | 426 | 1786 | 59 | 1 | 5 |
| 721 | 9311724 | 225 | 1166 | 20 | 2 | 5 |
| 722 | 9311725 | 300 | 1262 | 24 | 1 | 5 |
| 723 | 9311726 | 332 | 1528 | 27 | 5 | 4 |
| 724 | 9311727 | 249 | 1212 | 21 | 4 | 3 |
| 725 | 9311728 | 211 | 854 | 20 | 4 | 3 |
| 726 | 9311729 | 215 | 1169 | 27 | 8 | 2 |
| 727 | 9311730 | 213 | 930 | 20 | 5 | 3 |
| 728 | 9311731 | 209 | 741 | 12 | 2 | 3 |
| 729 | 9311732 | 197 | 807 | 17 | 5 | 2 |
| 730 | 9311733 | 248 | 1247 | 30 | 0 | 4 |
| 731 | 9311734 | 250 | 1185 | 22 | 7 | 2 |
| 732 | 9311735 | 231 | 989 | 33 | 0 | 5 |
| 733 | 9311736 | 230 | 973 | 30 | 2 | 3 |
| 734 | 9311737 | 243 | 1313 | 20 | 4 | 2 |
| 735 | 9311738 | 249 | 959 | 23 | 5 | 3 |
| 736 | 9311739 | 211 | 595 | 18 | 3 | 3 |
| 737 | 9311740 | 245 | 992 | 28 | 4 | 1 |
| 738 | 9311741 | 203 | 915 | 21 | 1 | 3 |
| 739 | 9311742 | 270 | 1325 | 37 | 2 | 4 |
| 740 | 9311743 | 218 | 1004 | 18 | 2 | 4 |
| 741 | 9311744 | 252 | 969 | 23 | 0 | 4 |
| 742 | 9311745 | 279 | 1604 | 46 | 4 | 3 |
| 743 | 9311746 | 400 | 1749 | 40 | 1 | 6 |
| 744 | 9311747 | 252 | 1091 | 25 | 5 | 4 |
| 745 | 9311748 | 286 | 1463 | 41 | 1 | 3 |
| 746 | 9311749 | 367 | 1870 | 33 | 3 | 4 |
| 747 | 9311750 | 318 | 1393 | 50 | 0 | 6 |
| 748 | 9311751 | 244 | 1328 | 26 | 2 | 4 |
| 749 | 9311752 | 289 | 1325 | 39 | 3 | 3 |
| 750 | 9311753 | 161 | 710 | 16 | 0 | 2 |
| 751 | 9311754 | 219 | 971 | 26 | 5 | 3 |
| 752 | 9311755 | 240 | 974 | 30 | 2 | 4 |
| 753 | 9311756 | 269 | 1181 | 24 | 2 | 3 |
| 754 | 9311757 | 296 | 1556 | 52 | 3 | 6 |
| 755 | 9311758 | 311 | 1141 | 35 | 2 | 5 |
| 756 | 9311759 | 230 | 883 | 22 | 3 | 4 |
| 757 | 9311760 | 399 | 1435 | 34 | 2 | 5 |
| 758 | 9311761 | 290 | 1145 | 24 | 1 | 1 |
| 759 | 9311762 | 500 | 1759 | 42 | 1 | 5 |
| 760 | 9311763 | 210 | 1142 | 24 | 6 | 2 |
| 761 | 9311764 | 266 | 1334 | 31 | 3 | 1 |
| 762 | 9311765 | 191 | 780 | 13 | 3 | 0 |
| 763 | 9311766 | 238 | 1004 | 22 | 6 | 3 |
| 764 | 9311767 | 262 | 1615 | 36 | 3 | 0 |
| 765 | 9311768 | 194 | 1054 | 25 | 1 | 1 |
| 766 | 9311769 | 232 | 1195 | 23 | 5 | 4 |
| 767 | 9311770 | 209 | 888 | 24 | 6 | 3 |
| 768 | 9311771 | 288 | 1544 | 27 | 1 | 3 |
| 769 | 9311772 | 202 | 788 | 18 | 9 | 3 |
| 770 | 9311773 | 208 | 852 | 24 | 1 | 3 |
| 771 | 9311774 | 245 | 1081 | 26 | 5 | 5 |
| 772 | 9311775 | 209 | 603 | 9 | 4 | 1 |
| 773 | 9311776 | 239 | 889 | 28 | 2 | 3 |
| 774 | 9311777 | 205 | 1212 | 20 | 3 | 1 |
| 775 | 9311778 | 200 | 785 | 23 | 5 | 4 |
| 776 | 9311779 | 321 | 1755 | 23 | 15 | 2 |
| 777 | 9311780 | 205 | 982 | 25 | 1 | 4 |
| 778 | 9311781 | 244 | 1055 | 18 | 2 | 4 |
| 779 | 9311782 | 230 | 1077 | 14 | 1 | 4 |
| 780 | 9311783 | 178 | 808 | 21 | 1 | 2 |
| 781 | 9311784 | 277 | 1101 | 42 | 5 | 5 |
| 782 | 9311785 | 193 | 808 | 27 | 9 | 3 |
| 783 | 9311786 | 421 | 2045 | 48 | 5 | 5 |
| 784 | 9311787 | 371 | 1158 | 29 | 2 | 3 |
| 785 | 9311788 | 352 | 1868 | 35 | 4 | 3 |
| 786 | 9311789 | 198 | 763 | 19 | 2 | 4 |
| 787 | 9311790 | 349 | 1426 | 51 | 5 | 3 |
| 788 | 9311791 | 293 | 1298 | 33 | 2 | 3 |
| 789 | 9311792 | 263 | 1159 | 12 | 6 | 5 |
| 790 | 9311793 | 200 | 1038 | 16 | 7 | 5 |
| 791 | 9311794 | 228 | 903 | 24 | 3 | 3 |
| 792 | 9311795 | 205 | 870 | 18 | 2 | 2 |
| 793 | 9311796 | 231 | 766 | 20 | 3 | 5 |
| 794 | 9311797 | 258 | 1137 | 29 | 4 | 3 |
| 795 | 9311798 | 200 | 713 | 21 | 2 | 3 |
| 796 | 9311799 | 187 | 802 | 12 | 10 | 2 |
| 797 | 9311800 | 214 | 870 | 15 | 1 | 2 |
| 798 | 9311801 | 220 | 793 | 25 | 3 | 1 |
| 799 | 9311802 | 347 | 1599 | 47 | 2 | 3 |
| 800 | 9311803 | 237 | 1015 | 18 | 2 | 2 |
| 801 | 9311804 | 197 | 849 | 20 | 3 | 2 |
| 802 | 9311805 | 262 | 1318 | 22 | 1 | 2 |
| 803 | 9311806 | 189 | 802 | 16 | 1 | 3 |
| 804 | 9311807 | 193 | 960 | 15 | 0 | 1 |
| 805 | 9311808 | 433 | 1948 | 35 | 2 | 3 |
| 806 | 9311809 | 194 | 886 | 26 | 1 | 2 |
| 807 | 9311810 | 230 | 863 | 18 | 10 | 4 |
| 808 | 9311811 | 195 | 747 | 18 | 5 | 2 |
| 809 | 9311812 | 243 | 1191 | 23 | 2 | 4 |
| 810 | 9311813 | 248 | 971 | 16 | 6 | 3 |
| 811 | 9311814 | 238 | 1272 | 23 | 2 | 4 |
| 812 | 9311815 | 234 | 1125 | 27 | 6 | 3 |
| 813 | 9311816 | 260 | 1085 | 20 | 8 | 1 |
| 814 | 9311817 | 216 | 1158 | 27 | 4 | 2 |
| 815 | 9311818 | 218 | 606 | 14 | 0 | 3 |
| 816 | 9311819 | 282 | 1497 | 25 | 0 | 1 |
| 817 | 9311820 | 255 | 849 | 22 | 1 | 3 |
| 818 | 9311821 | 301 | 1544 | 29 | 2 | 3 |
| 819 | 9311822 | 190 | 737 | 19 | 4 | 3 |
| 820 | 9311823 | 186 | 719 | 18 | 3 | 2 |
| 821 | 9311824 | 262 | 1091 | 16 | 9 | 5 |
| 822 | 9311825 | 416 | 1830 | 32 | 5 | 5 |
| 823 | 9311826 | 283 | 1161 | 25 | 5 | 2 |
| 824 | 9311827 | 198 | 890 | 24 | 1 | 2 |
| 825 | 9311828 | 223 | 913 | 13 | 1 | 2 |
| 826 | 9311829 | 282 | 1030 | 27 | 2 | 3 |
| 827 | 9311830 | 278 | 1342 | 34 | 6 | 3 |
| 828 | 9311831 | 200 | 842 | 19 | 1 | 2 |
| 829 | 9311832 | 262 | 1413 | 35 | 3 | 3 |
| 830 | 9311833 | 208 | 899 | 24 | 4 | 2 |
| 831 | 9311834 | 256 | 1186 | 25 | 0 | 2 |
| 832 | 9311835 | 242 | 1080 | 27 | 2 | 3 |
| 833 | 9311836 | 340 | 1431 | 36 | 2 | 4 |
| 834 | 9311837 | 224 | 982 | 25 | 3 | 2 |
| 835 | 9311838 | 261 | 1534 | 39 | 4 | 3 |
| 836 | 9311839 | 248 | 1124 | 29 | 3 | 1 |
| 837 | 9311840 | 190 | 850 | 19 | 0 | 3 |
| 838 | 9311841 | 149 | 442 | 12 | 2 | 1 |
| 839 | 9311842 | 247 | 1163 | 39 | 2 | 2 |
| 840 | 9311843 | 169 | 800 | 15 | 0 | 3 |
| 841 | 9311844 | 221 | 732 | 20 | 2 | 3 |
| 842 | 9311845 | 233 | 1071 | 28 | 4 | 2 |
| 843 | 9311846 | 204 | 804 | 18 | 4 | 1 |
| 844 | 9311847 | 264 | 1191 | 23 | 3 | 2 |
| 845 | 9311848 | 268 | 1158 | 48 | 4 | 5 |
| 846 | 9311849 | 178 | 650 | 19 | 1 | 3 |
| 847 | 9311850 | 322 | 1615 | 42 | 3 | 5 |
| 848 | 9311851 | 255 | 1101 | 39 | 5 | 2 |
| 849 | 9311852 | 341 | 1790 | 38 | 8 | 3 |
| 850 | 9311853 | 277 | 1068 | 27 | 3 | 1 |
| 851 | 9311854 | 258 | 1001 | 27 | 4 | 3 |
| 852 | 9311855 | 367 | 2117 | 69 | 4 | 4 |
| 853 | 9311856 | 302 | 1369 | 40 | 6 | 4 |
| 854 | 9311857 | 164 | 632 | 12 | 5 | 1 |
| 855 | 9311858 | 306 | 1669 | 38 | 1 | 3 |
| 856 | 9311859 | 200 | 867 | 28 | 4 | 3 |
| 857 | 9311860 | 222 | 1267 | 36 | 4 | 3 |
| 858 | 9311861 | 308 | 1668 | 42 | 1 | 4 |
| 859 | 9311862 | 281 | 1355 | 51 | 3 | 5 |
| 860 | 9311863 | 238 | 1196 | 23 | 3 | 4 |
| 861 | 9311864 | 181 | 863 | 20 | 9 | 2 |
| 862 | 9311865 | 268 | 1352 | 23 | 8 | 4 |
| 863 | 9311866 | 270 | 1523 | 43 | 6 | 4 |
| 864 | 9311867 | 320 | 1511 | 21 | 8 | 5 |
| 865 | 9311868 | 265 | 1427 | 32 | 5 | 1 |
| 866 | 9311869 | 225 | 1197 | 16 | 6 | 3 |
| 867 | 9311870 | 211 | 651 | 18 | 3 | 2 |
| 868 | 9311871 | 281 | 1238 | 22 | 5 | 3 |
| 869 | 9311872 | 303 | 1516 | 27 | 6 | 3 |
| 870 | 9311873 | 287 | 1464 | 22 | 6 | 5 |
| 871 | 9311874 | 174 | 798 | 9 | 8 | 2 |
| 872 | 9311875 | 195 | 588 | 21 | 0 | 4 |
| 873 | 9311876 | 301 | 1090 | 28 | 4 | 5 |
| 874 | 9311877 | 226 | 975 | 14 | 4 | 3 |
| 875 | 9311878 | 253 | 1118 | 32 | 2 | 3 |
| 876 | 9311879 | 202 | 1004 | 20 | 1 | 1 |
| 877 | 9311880 | 261 | 766 | 28 | 4 | 5 |
| 878 | 9311881 | 293 | 975 | 34 | 0 | 5 |
| 879 | 9311882 | 338 | 1212 | 32 | 1 | 5 |
| 880 | 9311883 | 213 | 810 | 27 | 2 | 4 |
| 881 | 9311884 | 334 | 1528 | 32 | 3 | 3 |
| 882 | 9311885 | 232 | 982 | 30 | 6 | 4 |
| 883 | 9311886 | 233 | 686 | 20 | 2 | 3 |
| 884 | 9311887 | 346 | 1460 | 38 | 7 | 4 |
| 885 | 9311888 | 195 | 789 | 20 | 2 | 3 |
| 886 | 9311889 | 250 | 1267 | 28 | 2 | 3 |
| 887 | 9311890 | 274 | 890 | 8 | 5 | 5 |
| 888 | 9311891 | 234 | 991 | 24 | 1 | 1 |
| 889 | 9311892 | 251 | 891 | 19 | 4 | 4 |
| 890 | 9311893 | 185 | 795 | 11 | 7 | 0 |
| 891 | 9311894 | 229 | 886 | 20 | 4 | 4 |
| 892 | 9311895 | 240 | 1094 | 28 | 4 | 5 |
| 893 | 9311896 | 196 | 886 | 12 | 2 | 2 |
| 894 | 9311897 | 215 | 643 | 23 | 5 | 2 |
| 895 | 9311898 | 244 | 1211 | 28 | 11 | 3 |
| 896 | 9311899 | 209 | 957 | 13 | 3 | 3 |
| 897 | 9311900 | 167 | 838 | 16 | 4 | 0 |
| 898 | 9311901 | 187 | 758 | 16 | 1 | 2 |
| 899 | 9311902 | 285 | 1397 | 36 | 10 | 4 |
| 900 | 9311903 | 206 | 1099 | 20 | 3 | 1 |
| 901 | 9311904 | 291 | 1186 | 36 | 3 | 2 |
| 902 | 9311905 | 334 | 1472 | 19 | 10 | 1 |
| 903 | 9311906 | 214 | 945 | 20 | 3 | 2 |
| 904 | 9311907 | 209 | 818 | 23 | 5 | 3 |
| 905 | 9311908 | 243 | 1126 | 15 | 5 | 3 |
| 906 | 9311909 | 297 | 1447 | 30 | 6 | 5 |
| 907 | 9311910 | 296 | 1367 | 14 | 1 | 5 |
| 908 | 9311911 | 206 | 813 | 20 | 5 | 3 |
| 909 | 9311912 | 202 | 1016 | 17 | 1 | 3 |
| 910 | 9311913 | 263 | 897 | 28 | 1 | 3 |
| 911 | 9311914 | 209 | 902 | 19 | 3 | 3 |
| 912 | 9311915 | 271 | 1486 | 18 | 7 | 3 |
| 913 | 9311916 | 339 | 1786 | 32 | 3 | 5 |
| 914 | 9311917 | 262 | 1202 | 35 | 4 | 3 |
| 915 | 9311918 | 254 | 978 | 30 | 2 | 5 |
| 916 | 9311919 | 201 | 781 | 21 | 4 | 3 |
| 917 | 9311920 | 289 | 1190 | 31 | 2 | 5 |
| 918 | 9311921 | 226 | 756 | 22 | 0 | 3 |
| 919 | 9311922 | 250 | 951 | 27 | 1 | 5 |
| 920 | 9311923 | 249 | 1098 | 30 | 5 | 1 |
| 921 | 9311924 | 337 | 1308 | 61 | 4 | 5 |
| 922 | 9311925 | 444 | 1866 | 37 | 3 | 4 |
| 923 | 9311926 | 444 | 1536 | 49 | 1 | 5 |
| 924 | 9311927 | 217 | 825 | 22 | 4 | 4 |
| 925 | 9311928 | 242 | 1152 | 23 | 1 | 3 |
| 926 | 9311929 | 299 | 1483 | 37 | 1 | 4 |
| 927 | 9311930 | 262 | 1217 | 30 | 1 | 4 |
| 928 | 9311931 | 207 | 699 | 22 | 3 | 4 |
| 929 | 9311932 | 347 | 1293 | 42 | 2 | 5 |
| 930 | 9311933 | 244 | 905 | 34 | 0 | 4 |
| 931 | 9311934 | 293 | 1453 | 41 | 1 | 5 |
| 932 | 9311935 | 192 | 680 | 22 | 2 | 3 |
| 933 | 9311936 | 341 | 1457 | 37 | 1 | 5 |
| 934 | 9311937 | 249 | 813 | 22 | 5 | 4 |
| 935 | 9311938 | 234 | 814 | 30 | 0 | 2 |
| 936 | 9311939 | 427 | 1660 | 39 | 5 | 3 |
| 937 | 9311940 | 366 | 1674 | 35 | 4 | 1 |
| 938 | 9311941 | 251 | 1489 | 34 | 4 | 2 |
| 939 | 9311942 | 354 | 1513 | 42 | 4 | 4 |
| 940 | 9311943 | 268 | 1116 | 29 | 6 | 4 |
| 941 | 9311944 | 208 | 1036 | 26 | 4 | 1 |
| 942 | 9311945 | 228 | 1095 | 31 | 1 | 4 |
| 943 | 9311946 | 217 | 1091 | 44 | 5 | 5 |
| 944 | 9311947 | 231 | 951 | 28 | 1 | 2 |
| 945 | 9311948 | 205 | 710 | 14 | 3 | 1 |
| 946 | 9311949 | 265 | 1210 | 24 | 9 | 3 |
| 947 | 9311950 | 198 | 666 | 16 | 4 | 2 |
| 948 | 9311951 | 276 | 1212 | 35 | 7 | 5 |
| 949 | 9311952 | 216 | 848 | 16 | 2 | 2 |
| 950 | 9311953 | 282 | 1439 | 33 | 4 | 3 |
| 951 | 9311954 | 240 | 951 | 8 | 6 | 3 |
| 952 | 9311955 | 337 | 1480 | 50 | 5 | 3 |
| 953 | 9311956 | 314 | 1677 | 35 | 4 | 1 |
| 954 | 9311957 | 258 | 1341 | 34 | 0 | 3 |
| 955 | 9311958 | 239 | 1359 | 27 | 2 | 5 |
| 956 | 9311959 | 217 | 1140 | 22 | 4 | 3 |
| 957 | 9311960 | 252 | 1232 | 27 | 3 | 2 |
| 958 | 9311961 | 365 | 1733 | 44 | 3 | 5 |
| 959 | 9311962 | 232 | 1006 | 26 | 3 | 2 |
| 960 | 9311963 | 277 | 1299 | 32 | 4 | 2 |
| 961 | 9311964 | 200 | 794 | 17 | 1 | 4 |
| 962 | 9311965 | 320 | 1488 | 25 | 1 | 4 |
| 963 | 9311966 | 227 | 686 | 21 | 1 | 5 |
| 964 | 9311967 | 260 | 1234 | 29 | 4 | 5 |
| 965 | 9311968 | 253 | 1262 | 31 | 2 | 3 |
| 966 | 9311969 | 230 | 1010 | 25 | 1 | 4 |
| 967 | 9311970 | 276 | 1240 | 22 | 0 | 3 |
| 968 | 9311971 | 241 | 765 | 23 | 4 | 4 |
| 969 | 9311972 | 254 | 1138 | 33 | 8 | 4 |
| 970 | 9311973 | 245 | 1165 | 28 | 1 | 3 |
| 971 | 9311974 | 206 | 1000 | 12 | 0 | 3 |
| 972 | 9311975 | 264 | 959 | 20 | 8 | 3 |
| 973 | 9311976 | 265 | 1204 | 35 | 5 | 3 |
| 974 | 9311977 | 257 | 913 | 15 | 10 | 3 |
| 975 | 9311978 | 244 | 1093 | 31 | 2 | 2 |
| 976 | 9311979 | 250 | 1033 | 18 | 1 | 4 |
| 977 | 9311980 | 201 | 776 | 14 | 5 | 2 |
| 978 | 9311981 | 254 | 905 | 31 | 3 | 3 |
| 979 | 9311982 | 181 | 701 | 17 | 3 | 1 |
| 980 | 9311983 | 247 | 850 | 20 | 0 | 5 |
| 981 | 9311984 | 300 | 1329 | 22 | 7 | 3 |
| 982 | 9311985 | 201 | 820 | 21 | 1 | 4 |
| 983 | 9311986 | 334 | 1090 | 32 | 4 | 5 |
| 984 | 9311987 | 236 | 943 | 26 | 8 | 2 |
| 985 | 9311988 | 245 | 904 | 39 | 6 | 4 |
| 986 | 9311989 | 258 | 993 | 29 | 0 | 3 |
| 987 | 9311990 | 352 | 1440 | 31 | 3 | 5 |
| 988 | 9311991 | 219 | 966 | 19 | 1 | 1 |
| 989 | 9311992 | 285 | 1292 | 30 | 3 | 3 |
| 990 | 9311993 | 235 | 1196 | 27 | 2 | 4 |
| 991 | 9311994 | 269 | 1156 | 21 | 8 | 3 |
| 992 | 9311995 | 174 | 790 | 20 | 0 | 3 |
| 993 | 9311996 | 210 | 1093 | 14 | 3 | 5 |
| 994 | 9311997 | 219 | 927 | 15 | 1 | 4 |
| 995 | 9311998 | 156 | 589 | 15 | 4 | 2 |
| 996 | 9311999 | 308 | 1383 | 31 | 1 | 4 |
| 997 | 9312000 | 225 | 788 | 13 | 2 | 3 |
| 998 | 9312001 | 167 | 711 | 14 | 1 | 3 |
| 999 | 9312002 | 259 | 1328 | 24 | 0 | 5 |
| 1000 | 9312003 | 253 | 823 | 21 | 10 | 3 |
| 1001 | 9312004 | 230 | 995 | 19 | 4 | 4 |
| 1002 | 9312005 | 234 | 1090 | 22 | 0 | 3 |
| 1003 | 9312006 | 164 | 495 | 18 | 2 | 4 |
| 1004 | 9312007 | 279 | 1247 | 26 | 1 | 3 |
| 1005 | 9312008 | 261 | 1239 | 21 | 2 | 4 |
| 1006 | 9312009 | 327 | 1157 | 46 | 4 | 5 |
| 1007 | 9312010 | 261 | 1264 | 29 | 2 | 4 |
| 1008 | 9312011 | 210 | 830 | 25 | 4 | 4 |
| 1009 | 9312012 | 234 | 1003 | 33 | 3 | 4 |
| 1010 | 9312013 | 345 | 1409 | 34 | 2 | 4 |
| 1011 | 9312014 | 322 | 1423 | 46 | 3 | 3 |
| 1012 | 9312015 | 295 | 1201 | 42 | 1 | 3 |
| 1013 | 9312016 | 276 | 1069 | 37 | 4 | 4 |
| 1014 | 9312017 | 406 | 1366 | 35 | 5 | 4 |
| 1015 | 9312018 | 165 | 582 | 20 | 2 | 2 |
| 1016 | 9312019 | 148 | 498 | 8 | 2 | 4 |
| 1017 | 9312020 | 231 | 1153 | 31 | 1 | 3 |
| 1018 | 9312021 | 267 | 1087 | 32 | 1 | 4 |
| 1019 | 9312022 | 282 | 1316 | 31 | 7 | 3 |
| 1020 | 9312023 | 195 | 633 | 17 | 2 | 3 |
| 1021 | 9312024 | 339 | 1473 | 44 | 3 | 5 |
| 1022 | 9312025 | 204 | 989 | 24 | 3 | 2 |
| 1023 | 9312026 | 280 | 1028 | 32 | 4 | 3 |
| 1024 | 9312027 | 166 | 677 | 22 | 3 | 2 |
| 1025 | 9312028 | 184 | 726 | 15 | 5 | 0 |
| 1026 | 9312029 | 267 | 1289 | 39 | 10 | 3 |
| 1027 | 9312030 | 250 | 1165 | 30 | 7 | 3 |
| 1028 | 9312031 | 201 | 754 | 25 | 4 | 4 |
| 1029 | 9312032 | 233 | 973 | 29 | 6 | 2 |
| 1030 | 9312033 | 247 | 1015 | 24 | 4 | 3 |
| 1031 | 9312034 | 200 | 999 | 15 | 3 | 2 |
| 1032 | 9312035 | 239 | 1251 | 27 | 5 | 4 |
| 1033 | 9312036 | 246 | 1027 | 29 | 7 | 4 |
| 1034 | 9312037 | 222 | 1072 | 24 | 4 | 5 |
| 1035 | 9312038 | 202 | 593 | 18 | 4 | 3 |
| 1036 | 9312039 | 207 | 996 | 28 | 3 | 2 |
| 1037 | 9312040 | 218 | 867 | 20 | 2 | 2 |
| 1038 | 9312041 | 190 | 798 | 23 | 4 | 2 |
| 1039 | 9312042 | 214 | 853 | 17 | 3 | 1 |
| 1040 | 9312043 | 184 | 691 | 23 | 5 | 2 |
| 1041 | 9312044 | 321 | 1520 | 35 | 3 | 4 |
| 1042 | 9312045 | 212 | 845 | 19 | 4 | 2 |
| 1043 | 9312046 | 231 | 1171 | 36 | 3 | 5 |
| 1044 | 9312047 | 273 | 1353 | 38 | 4 | 3 |
| 1045 | 9312048 | 264 | 1296 | 40 | 4 | 4 |
| 1046 | 9312049 | 279 | 1306 | 37 | 4 | 3 |
| 1047 | 9312050 | 364 | 1531 | 46 | 6 | 3 |
| 1048 | 9312051 | 323 | 1155 | 40 | 0 | 4 |
| 1049 | 9312052 | 236 | 984 | 18 | 2 | 3 |
| 1050 | 9312053 | 260 | 927 | 21 | 0 | 4 |
| 1051 | 9312054 | 277 | 893 | 13 | 1 | 4 |
| 1052 | 9312055 | 213 | 866 | 17 | 6 | 4 |
| 1053 | 9312056 | 296 | 1060 | 32 | 0 | 4 |
| 1054 | 9312057 | 242 | 1054 | 24 | 4 | 4 |
| 1055 | 9312058 | 196 | 603 | 15 | 1 | 4 |
| 1056 | 9312059 | 467 | 2246 | 47 | 11 | 5 |
| 1057 | 9312060 | 315 | 1247 | 27 | 2 | 4 |
| 1058 | 9312061 | 221 | 873 | 16 | 2 | 4 |
| 1059 | 9312062 | 256 | 1026 | 19 | 10 | 1 |
| 1060 | 9312063 | 232 | 919 | 19 | 5 | 5 |
| 1061 | 9312064 | 201 | 801 | 18 | 2 | 3 |
| 1062 | 9312065 | 277 | 1294 | 15 | 8 | 4 |
| 1063 | 9312066 | 254 | 1232 | 16 | 3 | 2 |
| 1064 | 9312067 | 190 | 710 | 17 | 0 | 1 |
| 1065 | 9312068 | 172 | 722 | 10 | 2 | 1 |
| 1066 | 9312069 | 254 | 969 | 14 | 5 | 6 |
| 1067 | 9312070 | 272 | 1076 | 21 | 3 | 3 |
| 1068 | 9312071 | 202 | 710 | 25 | 1 | 4 |
| 1069 | 9312072 | 330 | 1271 | 44 | 3 | 5 |
| 1070 | 9312073 | 282 | 1228 | 25 | 1 | 4 |
| 1071 | 9312074 | 254 | 963 | 19 | 0 | 2 |
| 1072 | 9312075 | 263 | 1287 | 29 | 2 | 3 |
| 1073 | 9312076 | 233 | 916 | 22 | 3 | 4 |
| 1074 | 9312077 | 238 | 1146 | 14 | 3 | 3 |
| 1075 | 9312078 | 243 | 1202 | 23 | 4 | 4 |
| 1076 | 9312079 | 289 | 1250 | 34 | 1 | 3 |
| 1077 | 9312080 | 202 | 628 | 20 | 6 | 2 |
| 1078 | 9312081 | 300 | 1386 | 23 | 3 | 4 |
| 1079 | 9312082 | 225 | 881 | 23 | 2 | 5 |
| 1080 | 9312083 | 257 | 1169 | 35 | 0 | 4 |
| 1081 | 9312084 | 171 | 747 | 18 | 6 | 2 |
| 1082 | 9312085 | 204 | 815 | 14 | 4 | 2 |
| 1083 | 9312086 | 313 | 1421 | 29 | 4 | 3 |
| 1084 | 9312087 | 220 | 933 | 21 | 5 | 3 |
| 1085 | 9312088 | 241 | 899 | 26 | 2 | 3 |
| 1086 | 9312089 | 225 | 1079 | 24 | 4 | 4 |
| 1087 | 9312090 | 230 | 841 | 19 | 1 | 3 |
| 1088 | 9312091 | 347 | 1358 | 36 | 6 | 3 |
| 1089 | 9312092 | 262 | 1115 | 42 | 2 | 4 |
| 1090 | 9312093 | 226 | 1047 | 19 | 1 | 3 |
| 1091 | 9312094 | 291 | 1336 | 22 | 0 | 4 |
| 1092 | 9312095 | 305 | 1438 | 29 | 2 | 3 |
| 1093 | 9312096 | 239 | 1358 | 21 | 1 | 2 |
| 1094 | 9312097 | 205 | 1034 | 16 | 5 | 2 |
| 1095 | 9312098 | 184 | 769 | 23 | 1 | 4 |
| 1096 | 9312099 | 252 | 1108 | 27 | 1 | 4 |
| 1097 | 9312100 | 323 | 1634 | 42 | 0 | 4 |
| 1098 | 9312101 | 185 | 608 | 20 | 1 | 3 |
| 1099 | 9312102 | 240 | 849 | 19 | 2 | 4 |
| 1100 | 9312103 | 172 | 701 | 16 | 1 | 2 |
| 1101 | 9312104 | 261 | 1137 | 34 | 3 | 4 |
| 1102 | 9312105 | 313 | 1520 | 28 | 0 | 3 |
| 1103 | 9312106 | 295 | 1316 | 36 | 3 | 4 |
| 1104 | 9312107 | 240 | 964 | 29 | 3 | 2 |
| 1105 | 9312108 | 196 | 782 | 18 | 3 | 2 |
| 1106 | 9312109 | 260 | 1067 | 18 | 1 | 2 |
| 1107 | 9312110 | 190 | 616 | 24 | 1 | 4 |
| 1108 | 9312111 | 216 | 1087 | 29 | 0 | 3 |
| 1109 | 9312112 | 156 | 664 | 8 | 2 | 1 |
| 1110 | 9312113 | 245 | 1105 | 39 | 3 | 3 |
| 1111 | 9312114 | 311 | 1131 | 38 | 2 | 4 |
| 1112 | 9312115 | 306 | 1271 | 25 | 6 | 5 |
| 1113 | 9312116 | 273 | 1151 | 19 | 5 | 3 |
| 1114 | 9312117 | 313 | 1371 | 46 | 4 | 4 |
| 1115 | 9312118 | 236 | 1040 | 41 | 0 | 5 |
| 1116 | 9312119 | 186 | 874 | 19 | 1 | 2 |
| 1117 | 9312120 | 206 | 877 | 17 | 5 | 2 |
| 1118 | 9312121 | 282 | 1015 | 29 | 11 | 3 |
| 1119 | 9312122 | 244 | 1290 | 27 | 7 | 3 |
| 1120 | 9312123 | 186 | 896 | 19 | 2 | 1 |
| 1121 | 9312124 | 388 | 1419 | 43 | 7 | 4 |
| 1122 | 9312125 | 224 | 951 | 13 | 3 | 3 |
| 1123 | 9312126 | 243 | 1008 | 27 | 4 | 2 |
| 1124 | 9312127 | 234 | 1437 | 30 | 5 | 4 |
| 1125 | 9312128 | 227 | 1067 | 30 | 4 | 3 |
| 1126 | 9312129 | 270 | 1134 | 41 | 5 | 5 |
| 1127 | 9312130 | 261 | 1188 | 27 | 3 | 2 |
| 1128 | 9312131 | 319 | 1823 | 35 | 2 | 3 |
| 1129 | 9312132 | 212 | 849 | 28 | 6 | 2 |
| 1130 | 9312133 | 242 | 1193 | 29 | 6 | 3 |
| 1131 | 9312134 | 273 | 1127 | 23 | 4 | 3 |
| 1132 | 9312135 | 206 | 1047 | 24 | 6 | 3 |
| 1133 | 9312136 | 204 | 852 | 23 | 11 | 3 |
| 1134 | 9312137 | 252 | 1103 | 34 | 3 | 2 |
| 1135 | 9312138 | 302 | 1356 | 37 | 1 | 5 |
| 1136 | 9312139 | 321 | 1332 | 31 | 2 | 1 |
| 1137 | 9312140 | 283 | 1176 | 26 | 3 | 4 |
| 1138 | 9312141 | 280 | 1271 | 40 | 1 | 3 |
| 1139 | 9312142 | 222 | 739 | 19 | 6 | 1 |
| 1140 | 9312143 | 300 | 1387 | 20 | 4 | 6 |
| 1141 | 9312144 | 252 | 1048 | 27 | 3 | 3 |
| 1142 | 9312145 | 236 | 834 | 16 | 7 | 2 |
| 1143 | 9312146 | 238 | 863 | 26 | 1 | 2 |
| 1144 | 9312147 | 226 | 996 | 23 | 3 | 2 |
| 1145 | 9312148 | 200 | 736 | 11 | 2 | 5 |
| 1146 | 9312149 | 286 | 1158 | 16 | 4 | 4 |
| 1147 | 9312150 | 227 | 1085 | 26 | 3 | 2 |
| 1148 | 9312151 | 242 | 1001 | 26 | 2 | 4 |
| 1149 | 9312152 | 228 | 944 | 27 | 0 | 5 |
| 1150 | 9312153 | 215 | 940 | 23 | 0 | 1 |
| 1151 | 9312154 | 223 | 778 | 11 | 2 | 3 |
| 1152 | 9312155 | 275 | 1100 | 31 | 3 | 5 |
| 1153 | 9312156 | 282 | 1315 | 24 | 3 | 3 |
| 1154 | 9312157 | 239 | 1010 | 18 | 11 | 2 |
| 1155 | 9312158 | 192 | 714 | 12 | 5 | 1 |
| 1156 | 9312159 | 293 | 1496 | 15 | 3 | 4 |
| 1157 | 9312160 | 240 | 1190 | 23 | 1 | 2 |
| 1158 | 9312161 | 276 | 998 | 34 | 3 | 4 |
| 1159 | 9312162 | 240 | 989 | 38 | 6 | 3 |
| 1160 | 9312163 | 240 | 939 | 26 | 3 | 2 |
| 1161 | 9312164 | 299 | 1380 | 36 | 10 | 3 |
| 1162 | 9312165 | 214 | 1161 | 22 | 2 | 3 |
| 1163 | 9312166 | 302 | 1322 | 45 | 3 | 3 |
| 1164 | 9312167 | 194 | 932 | 15 | 3 | 4 |
| 1165 | 9312168 | 309 | 1170 | 35 | 7 | 2 |
| 1166 | 9312169 | 245 | 1088 | 28 | 7 | 4 |
| 1167 | 9312170 | 195 | 911 | 18 | 5 | 4 |
| 1168 | 9312171 | 385 | 1408 | 45 | 4 | 6 |
| 1169 | 9312172 | 280 | 1417 | 32 | 3 | 4 |
| 1170 | 9312173 | 240 | 886 | 34 | 0 | 4 |
| 1171 | 9312174 | 193 | 940 | 23 | 3 | 4 |
| 1172 | 9312175 | 219 | 981 | 28 | 2 | 3 |
| 1173 | 9312176 | 271 | 1369 | 34 | 6 | 3 |
| 1174 | 9312177 | 180 | 528 | 20 | 4 | 1 |
| 1175 | 9312178 | 394 | 2042 | 42 | 4 | 3 |
| 1176 | 9312179 | 216 | 1017 | 32 | 0 | 5 |
| 1177 | 9312180 | 260 | 1103 | 18 | 2 | 3 |
| 1178 | 9312181 | 201 | 861 | 16 | 1 | 2 |
| 1179 | 9312182 | 269 | 1182 | 28 | 1 | 4 |
| 1180 | 9312183 | 260 | 1292 | 22 | 2 | 4 |
| 1181 | 9312184 | 249 | 1070 | 27 | 1 | 4 |
| 1182 | 9312185 | 220 | 917 | 28 | 2 | 3 |
| 1183 | 9312186 | 196 | 741 | 17 | 2 | 3 |
| 1184 | 9312187 | 335 | 1465 | 32 | 2 | 2 |
| 1185 | 9312188 | 228 | 1015 | 28 | 4 | 4 |
| 1186 | 9312189 | 223 | 939 | 26 | 1 | 2 |
| 1187 | 9312190 | 213 | 844 | 28 | 1 | 3 |
| 1188 | 9312191 | 232 | 1057 | 31 | 0 | 4 |
| 1189 | 9312192 | 238 | 1045 | 38 | 1 | 3 |
| 1190 | 9312193 | 246 | 1000 | 26 | 1 | 3 |
| 1191 | 9312194 | 202 | 825 | 18 | 2 | 4 |
| 1192 | 9312195 | 265 | 1030 | 33 | 0 | 6 |
| 1193 | 9312196 | 223 | 964 | 24 | 3 | 4 |
| 1194 | 9312197 | 268 | 1149 | 29 | 4 | 5 |
| 1195 | 9312198 | 500 | 2105 | 42 | 5 | 5 |
| 1196 | 9312199 | 218 | 732 | 22 | 4 | 4 |
| 1197 | 9312200 | 407 | 1855 | 46 | 0 | 4 |
| 1198 | 9312201 | 269 | 1157 | 35 | 3 | 4 |
| 1199 | 9312202 | 414 | 1568 | 44 | 3 | 6 |
| 1200 | 9312203 | 242 | 1217 | 29 | 4 | 2 |
| 1201 | 9312204 | 219 | 977 | 32 | 5 | 4 |
| 1202 | 9312205 | 194 | 1033 | 25 | 2 | 4 |
| 1203 | 9312206 | 259 | 1553 | 37 | 7 | 3 |
| 1204 | 9312207 | 289 | 1254 | 23 | 5 | 3 |
| 1205 | 9312208 | 224 | 820 | 27 | 1 | 3 |
| 1206 | 9312209 | 307 | 1445 | 36 | 2 | 3 |
| 1207 | 9312210 | 500 | 2146 | 66 | 5 | 4 |
| 1208 | 9312211 | 258 | 1344 | 33 | 3 | 4 |
| 1209 | 9312212 | 273 | 1396 | 36 | 4 | 5 |
| 1210 | 9312213 | 216 | 1066 | 25 | 8 | 3 |
| 1211 | 9312214 | 240 | 1275 | 40 | 4 | 3 |
| 1212 | 9312215 | 292 | 1545 | 43 | 8 | 4 |
| 1213 | 9312216 | 181 | 666 | 14 | 5 | 1 |
| 1214 | 9312217 | 245 | 1359 | 36 | 5 | 6 |
| 1215 | 9312218 | 253 | 1229 | 26 | 8 | 3 |
| 1216 | 9312219 | 310 | 1311 | 37 | 4 | 2 |
| 1217 | 9312220 | 227 | 1074 | 20 | 5 | 3 |
| 1218 | 9312221 | 333 | 1543 | 28 | 3 | 5 |
| 1219 | 9312222 | 241 | 1172 | 33 | 3 | 5 |
| 1220 | 9312223 | 165 | 685 | 19 | 4 | 4 |
| 1221 | 9312224 | 224 | 949 | 14 | 2 | 3 |
| 1222 | 9312225 | 206 | 873 | 15 | 6 | 4 |
| 1223 | 9312226 | 224 | 978 | 17 | 2 | 3 |
| 1224 | 9312227 | 207 | 958 | 23 | 4 | 4 |
| 1225 | 9312228 | 183 | 725 | 13 | 4 | 2 |
| 1226 | 9312229 | 294 | 1124 | 29 | 2 | 3 |
| 1227 | 9312230 | 201 | 770 | 21 | 7 | 3 |
| 1228 | 9312231 | 341 | 1442 | 26 | 3 | 5 |
| 1229 | 9312232 | 500 | 2115 | 12 | 2 | 5 |
| 1230 | 9312233 | 214 | 761 | 20 | 7 | 3 |
| 1231 | 9312234 | 318 | 1120 | 31 | 3 | 4 |
| 1232 | 9312235 | 280 | 1204 | 34 | 5 | 5 |
| 1233 | 9312236 | 244 | 962 | 25 | 2 | 4 |
| 1234 | 9312237 | 259 | 1073 | 18 | 1 | 2 |
| 1235 | 9312238 | 228 | 736 | 22 | 1 | 3 |
| 1236 | 9312239 | 246 | 974 | 7 | 6 | 1 |
| 1237 | 9312240 | 260 | 1177 | 27 | 1 | 4 |
| 1238 | 9312241 | 249 | 973 | 15 | 4 | 3 |
| 1239 | 9312242 | 263 | 1255 | 30 | 1 | 5 |
| 1240 | 9312243 | 214 | 1150 | 29 | 2 | 3 |
| 1241 | 9312244 | 189 | 604 | 22 | 2 | 3 |
| 1242 | 9312245 | 230 | 931 | 26 | 1 | 5 |
| 1243 | 9312246 | 192 | 723 | 18 | 3 | 3 |
| 1244 | 9312247 | 213 | 820 | 21 | 2 | 3 |
| 1245 | 9312248 | 244 | 1014 | 26 | 2 | 3 |
| 1246 | 9312249 | 246 | 963 | 24 | 6 | 3 |
| 1247 | 9312250 | 241 | 1026 | 16 | 2 | 3 |
| 1248 | 9312251 | 274 | 1123 | 33 | 9 | 3 |
| 1249 | 9312252 | 200 | 916 | 13 | 3 | 2 |
| 1250 | 9312253 | 227 | 758 | 20 | 1 | 1 |
| 1251 | 9312254 | 227 | 886 | 24 | 4 | 2 |
| 1252 | 9312255 | 285 | 1160 | 22 | 7 | 5 |
| 1253 | 9312256 | 272 | 1273 | 21 | 2 | 5 |
| 1254 | 9312257 | 247 | 873 | 14 | 7 | 3 |
| 1255 | 9312258 | 218 | 981 | 18 | 3 | 0 |
| 1256 | 9312259 | 226 | 913 | 30 | 4 | 4 |
| 1257 | 9312260 | 260 | 899 | 35 | 3 | 4 |
| 1258 | 9312261 | 238 | 998 | 22 | 1 | 3 |
| 1259 | 9312262 | 182 | 699 | 21 | 0 | 3 |
| 1260 | 9312263 | 273 | 1024 | 32 | 2 | 5 |
| 1261 | 9312264 | 332 | 1456 | 35 | 4 | 5 |
| 1262 | 9312265 | 276 | 1224 | 47 | 3 | 2 |
| 1263 | 9312266 | 204 | 859 | 17 | 1 | 4 |
| 1264 | 9312267 | 255 | 1120 | 27 | 2 | 2 |
| 1265 | 9312268 | 277 | 1385 | 23 | 2 | 4 |
| 1266 | 9312269 | 197 | 684 | 20 | 4 | 3 |
| 1267 | 9312270 | 162 | 590 | 17 | 1 | 4 |
| 1268 | 9312271 | 297 | 1210 | 29 | 0 | 2 |
| 1269 | 9312272 | 265 | 1386 | 25 | 5 | 3 |
| 1270 | 9312273 | 261 | 1149 | 26 | 3 | 3 |
| 1271 | 9312274 | 211 | 962 | 27 | 2 | 2 |
| 1272 | 9312275 | 235 | 1167 | 32 | 0 | 3 |
| 1273 | 9312276 | 351 | 1646 | 43 | 4 | 4 |
| 1274 | 9312277 | 228 | 1229 | 23 | 3 | 1 |
| 1275 | 9312278 | 191 | 841 | 20 | 3 | 2 |
| 1276 | 9312279 | 201 | 755 | 19 | 0 | 2 |
| 1277 | 9312280 | 195 | 854 | 21 | 2 | 3 |
| 1278 | 9312281 | 414 | 1893 | 45 | 4 | 5 |
| 1279 | 9312282 | 345 | 1518 | 39 | 1 | 6 |
| 1280 | 9312283 | 262 | 1096 | 26 | 0 | 4 |
| 1281 | 9312284 | 241 | 899 | 21 | 3 | 3 |
| 1282 | 9312285 | 318 | 1360 | 41 | 2 | 4 |
| 1283 | 9312286 | 158 | 782 | 14 | 1 | 1 |
| 1284 | 9312287 | 328 | 1680 | 32 | 3 | 3 |
| 1285 | 9312288 | 291 | 1348 | 30 | 2 | 2 |
| 1286 | 9312289 | 200 | 968 | 18 | 3 | 1 |
| 1287 | 9312290 | 260 | 1124 | 33 | 1 | 4 |
| 1288 | 9312291 | 304 | 1264 | 33 | 3 | 3 |
| 1289 | 9312292 | 189 | 991 | 22 | 1 | 3 |
| 1290 | 9312293 | 343 | 1408 | 44 | 0 | 4 |
| 1291 | 9312294 | 225 | 952 | 16 | 7 | 4 |
| 1292 | 9312295 | 420 | 1720 | 52 | 11 | 5 |
| 1293 | 9312296 | 224 | 1057 | 21 | 5 | 1 |
| 1294 | 9312297 | 273 | 1452 | 35 | 3 | 3 |
| 1295 | 9312298 | 342 | 1749 | 30 | 9 | 3 |
| 1296 | 9312299 | 213 | 1001 | 28 | 4 | 4 |
| 1297 | 9312300 | 284 | 1421 | 46 | 5 | 3 |
| 1298 | 9312301 | 244 | 948 | 22 | 1 | 1 |
| 1299 | 9312302 | 213 | 1026 | 22 | 2 | 4 |
| 1300 | 9312303 | 241 | 1009 | 21 | 3 | 4 |
| 1301 | 9312304 | 284 | 1284 | 43 | 7 | 4 |
| 1302 | 9312305 | 181 | 882 | 20 | 3 | 1 |
| 1303 | 9312306 | 226 | 758 | 20 | 4 | 4 |
| 1304 | 9312307 | 255 | 1285 | 32 | 3 | 3 |
| 1305 | 9312308 | 250 | 1704 | 33 | 2 | 3 |
| 1306 | 9312309 | 218 | 791 | 18 | 4 | 3 |
| 1307 | 9312310 | 256 | 1492 | 20 | 4 | 5 |
| 1308 | 9312311 | 295 | 1391 | 26 | 1 | 4 |
| 1309 | 9312312 | 255 | 1219 | 46 | 6 | 3 |
| 1310 | 9312313 | 229 | 864 | 32 | 1 | 0 |
| 1311 | 9312314 | 268 | 1323 | 20 | 1 | 3 |
| 1312 | 9312315 | 304 | 964 | 26 | 3 | 3 |
| 1313 | 9312316 | 234 | 983 | 24 | 2 | 3 |
| 1314 | 9312317 | 230 | 952 | 18 | 3 | 5 |
| 1315 | 9312318 | 204 | 578 | 18 | 1 | 4 |
| 1316 | 9312319 | 238 | 1048 | 23 | 1 | 4 |
| 1317 | 9312320 | 247 | 1067 | 23 | 4 | 2 |
| 1318 | 9312321 | 179 | 774 | 18 | 0 | 0 |
| 1319 | 9312322 | 292 | 1179 | 28 | 2 | 3 |
| 1320 | 9312323 | 200 | 875 | 14 | 0 | 3 |
| 1321 | 9312324 | 208 | 799 | 24 | 7 | 3 |
| 1322 | 9312325 | 263 | 1162 | 24 | 5 | 4 |
| 1323 | 9312326 | 190 | 717 | 20 | 3 | 2 |
| 1324 | 9312327 | 214 | 746 | 14 | 2 | 1 |
| 1325 | 9312328 | 198 | 809 | 25 | 4 | 2 |
| 1326 | 9312329 | 284 | 1067 | 21 | 6 | 6 |
| 1327 | 9312330 | 222 | 895 | 22 | 7 | 3 |
| 1328 | 9312331 | 217 | 895 | 26 | 3 | 2 |
| 1329 | 9312332 | 272 | 1217 | 24 | 5 | 5 |
| 1330 | 9312333 | 353 | 1430 | 45 | 5 | 4 |
| 1331 | 9312334 | 312 | 1209 | 23 | 1 | 4 |
| 1332 | 9312335 | 260 | 1069 | 38 | 5 | 4 |
| 1333 | 9312336 | 233 | 1025 | 38 | 2 | 5 |
| 1334 | 9312337 | 215 | 1103 | 20 | 1 | 3 |
| 1335 | 9312338 | 327 | 1599 | 40 | 2 | 2 |
| 1336 | 9312339 | 224 | 889 | 19 | 5 | 3 |
| 1337 | 9312340 | 274 | 1499 | 30 | 1 | 3 |
| 1338 | 9312341 | 215 | 1124 | 26 | 8 | 3 |
| 1339 | 9312342 | 253 | 992 | 32 | 2 | 5 |
| 1340 | 9312343 | 257 | 786 | 15 | 3 | 4 |
| 1341 | 9312344 | 337 | 1292 | 34 | 6 | 4 |
| 1342 | 9312345 | 204 | 793 | 19 | 3 | 3 |
| 1343 | 9312346 | 279 | 1241 | 22 | 5 | 5 |
| 1344 | 9312347 | 191 | 693 | 14 | 5 | 2 |
| 1345 | 9312348 | 207 | 1065 | 17 | 2 | 3 |
| 1346 | 9312349 | 255 | 1026 | 15 | 2 | 5 |
| 1347 | 9312350 | 206 | 742 | 18 | 4 | 4 |
| 1348 | 9312351 | 299 | 1542 | 25 | 5 | 3 |
| 1349 | 9312352 | 239 | 917 | 24 | 1 | 4 |
| 1350 | 9312353 | 207 | 669 | 11 | 2 | 4 |
| 1351 | 9312354 | 211 | 743 | 15 | 3 | 3 |
| 1352 | 9312355 | 275 | 1238 | 30 | 0 | 3 |
| 1353 | 9312356 | 272 | 1348 | 40 | 5 | 4 |
| 1354 | 9312357 | 259 | 1137 | 26 | 5 | 4 |
| 1355 | 9312358 | 252 | 996 | 16 | 1 | 5 |
| 1356 | 9312359 | 210 | 918 | 15 | 0 | 3 |
| 1357 | 9312360 | 230 | 988 | 28 | 1 | 4 |
| 1358 | 9312361 | 207 | 588 | 19 | 1 | 5 |
| 1359 | 9312362 | 218 | 606 | 26 | 4 | 4 |
| 1360 | 9312363 | 272 | 949 | 26 | 3 | 5 |
| 1361 | 9312364 | 290 | 1239 | 32 | 4 | 4 |
| 1362 | 9312365 | 203 | 1036 | 20 | 0 | 4 |
| 1363 | 9312366 | 333 | 1377 | 44 | 6 | 3 |
| 1364 | 9312367 | 302 | 1478 | 36 | 1 | 4 |
| 1365 | 9312368 | 241 | 1139 | 34 | 2 | 5 |
| 1366 | 9312369 | 230 | 1147 | 31 | 2 | 2 |
| 1367 | 9312370 | 298 | 1156 | 27 | 1 | 5 |
| 1368 | 9312371 | 228 | 847 | 34 | 2 | 3 |
| 1369 | 9312372 | 184 | 718 | 14 | 1 | 2 |
| 1370 | 9312373 | 227 | 820 | 23 | 3 | 1 |
| 1371 | 9312374 | 222 | 1016 | 27 | 2 | 1 |
| 1372 | 9312375 | 290 | 1152 | 21 | 0 | 2 |
| 1373 | 9312376 | 297 | 1464 | 38 | 2 | 5 |
| 1374 | 9312377 | 174 | 632 | 17 | 0 | 4 |
| 1375 | 9312378 | 272 | 1265 | 31 | 5 | 3 |
| 1376 | 9312379 | 279 | 1230 | 36 | 5 | 2 |
| 1377 | 9312380 | 217 | 1141 | 27 | 3 | 2 |
| 1378 | 9312381 | 220 | 1077 | 26 | 2 | 2 |
| 1379 | 9312382 | 256 | 1079 | 30 | 2 | 3 |
| 1380 | 9312383 | 234 | 1216 | 29 | 6 | 2 |
| 1381 | 9312384 | 197 | 804 | 27 | 7 | 2 |
| 1382 | 9312385 | 280 | 1205 | 27 | 5 | 4 |
| 1383 | 9312386 | 227 | 1021 | 33 | 4 | 4 |
| 1384 | 9312387 | 273 | 1402 | 27 | 3 | 2 |
| 1385 | 9312388 | 212 | 1061 | 22 | 5 | 2 |
| 1386 | 9312389 | 232 | 1124 | 28 | 4 | 2 |
| 1387 | 9312390 | 249 | 1151 | 38 | 3 | 3 |
| 1388 | 9312391 | 257 | 1098 | 30 | 11 | 2 |
| 1389 | 9312392 | 270 | 1314 | 31 | 6 | 3 |
| 1390 | 9312393 | 235 | 936 | 28 | 1 | 4 |
| 1391 | 9312394 | 233 | 1305 | 18 | 5 | 4 |
| 1392 | 9312395 | 341 | 1574 | 22 | 4 | 5 |
| 1393 | 9312396 | 210 | 830 | 24 | 5 | 3 |
| 1394 | 9312397 | 252 | 1261 | 38 | 2 | 2 |
| 1395 | 9312398 | 208 | 757 | 17 | 0 | 2 |
| 1396 | 9312399 | 208 | 1102 | 20 | 2 | 3 |
| 1397 | 9312400 | 239 | 1205 | 33 | 6 | 4 |
| 1398 | 9312401 | 188 | 739 | 13 | 4 | 4 |
| 1399 | 9312402 | 301 | 1488 | 19 | 8 | 3 |
| 1400 | 9312403 | 181 | 752 | 26 | 3 | 2 |
| 1401 | 9312404 | 214 | 787 | 13 | 2 | 3 |
| 1402 | 9312405 | 224 | 897 | 22 | 5 | 3 |
| 1403 | 9312406 | 266 | 932 | 27 | 1 | 5 |
| 1404 | 9312407 | 378 | 1599 | 46 | 4 | 6 |
| 1405 | 9312408 | 277 | 1186 | 34 | 6 | 4 |
| 1406 | 9312409 | 195 | 679 | 15 | 3 | 3 |
| 1407 | 9312410 | 300 | 1287 | 42 | 3 | 3 |
| 1408 | 9312411 | 277 | 958 | 29 | 4 | 5 |
| 1409 | 9312412 | 254 | 1066 | 27 | 2 | 5 |
| 1410 | 9312413 | 196 | 647 | 11 | 7 | 2 |
| 1411 | 9312414 | 268 | 1131 | 33 | 6 | 2 |
| 1412 | 9312415 | 251 | 1118 | 29 | 2 | 5 |
| 1413 | 9312416 | 233 | 1119 | 25 | 2 | 0 |
| 1414 | 9312417 | 226 | 735 | 17 | 3 | 3 |
| 1415 | 9312418 | 226 | 844 | 21 | 3 | 4 |
| 1416 | 9312419 | 239 | 1032 | 28 | 4 | 4 |
| 1417 | 9312420 | 224 | 987 | 17 | 3 | 5 |
| 1418 | 9312421 | 244 | 1188 | 21 | 8 | 2 |
| 1419 | 9312422 | 349 | 1479 | 46 | 6 | 3 |
| 1420 | 9312423 | 253 | 1362 | 29 | 3 | 2 |
| 1421 | 9312424 | 226 | 904 | 22 | 2 | 3 |
| 1422 | 9312425 | 215 | 863 | 26 | 2 | 4 |
| 1423 | 9312426 | 258 | 1197 | 24 | 3 | 5 |
| 1424 | 9312427 | 224 | 936 | 24 | 3 | 6 |
| 1425 | 9312428 | 283 | 1296 | 27 | 4 | 4 |
| 1426 | 9312429 | 294 | 998 | 31 | 3 | 3 |
| 1427 | 9312430 | 220 | 990 | 24 | 4 | 2 |
| 1428 | 9312431 | 244 | 1187 | 10 | 3 | 4 |
| 1429 | 9312432 | 250 | 631 | 22 | 5 | 2 |
| 1430 | 9312433 | 239 | 1007 | 25 | 5 | 4 |
| 1431 | 9312434 | 224 | 809 | 8 | 8 | 2 |
| 1432 | 9312435 | 231 | 1085 | 21 | 1 | 4 |
| 1433 | 9312436 | 267 | 1219 | 30 | 5 | 4 |
| 1434 | 9312437 | 275 | 1184 | 34 | 6 | 4 |
| 1435 | 9312438 | 477 | 1814 | 50 | 3 | 4 |
| 1436 | 9312439 | 255 | 953 | 33 | 2 | 5 |
| 1437 | 9312440 | 211 | 826 | 23 | 3 | 2 |
| 1438 | 9312441 | 270 | 1254 | 33 | 2 | 5 |
| 1439 | 9312442 | 259 | 1250 | 27 | 0 | 4 |
| 1440 | 9312443 | 269 | 731 | 14 | 3 | 2 |
| 1441 | 9312444 | 343 | 1733 | 42 | 2 | 2 |
| 1442 | 9312445 | 500 | 2318 | 44 | 3 | 3 |
| 1443 | 9312446 | 222 | 1102 | 30 | 1 | 4 |
| 1444 | 9312447 | 274 | 1179 | 23 | 4 | 5 |
| 1445 | 9312448 | 188 | 854 | 19 | 3 | 3 |
| 1446 | 9312449 | 226 | 1072 | 15 | 4 | 5 |
| 1447 | 9312450 | 175 | 561 | 8 | 3 | 1 |
| 1448 | 9312451 | 244 | 1282 | 25 | 2 | 2 |
| 1449 | 9312452 | 315 | 1230 | 44 | 5 | 4 |
| 1450 | 9312453 | 261 | 1061 | 25 | 2 | 3 |
| 1451 | 9312454 | 369 | 1686 | 48 | 2 | 4 |
| 1452 | 9312455 | 329 | 1305 | 36 | 2 | 5 |
| 1453 | 9312456 | 260 | 1097 | 36 | 5 | 2 |
| 1454 | 9312457 | 185 | 810 | 14 | 1 | 3 |
| 1455 | 9312458 | 254 | 972 | 29 | 1 | 4 |
| 1456 | 9312459 | 275 | 1137 | 32 | 1 | 4 |
| 1457 | 9312460 | 424 | 1537 | 40 | 2 | 4 |
| 1458 | 9312461 | 302 | 1474 | 42 | 2 | 5 |
| 1459 | 9312462 | 187 | 867 | 19 | 2 | 2 |
| 1460 | 9312463 | 231 | 1063 | 29 | 4 | 4 |
| 1461 | 9312464 | 188 | 884 | 28 | 3 | 4 |
| 1462 | 9312465 | 264 | 992 | 31 | 2 | 4 |
| 1463 | 9312466 | 164 | 592 | 12 | 2 | 3 |
| 1464 | 9312467 | 330 | 1301 | 39 | 4 | 4 |
| 1465 | 9312468 | 257 | 831 | 35 | 9 | 5 |
| 1466 | 9312469 | 259 | 1119 | 34 | 1 | 3 |
| 1467 | 9312470 | 228 | 970 | 36 | 8 | 3 |
| 1468 | 9312471 | 260 | 1445 | 36 | 5 | 3 |
| 1469 | 9312472 | 246 | 1280 | 27 | 4 | 2 |
| 1470 | 9312473 | 213 | 884 | 22 | 4 | 2 |
| 1471 | 9312474 | 392 | 1874 | 41 | 1 | 4 |
| 1472 | 9312475 | 234 | 1395 | 19 | 3 | 4 |
| 1473 | 9312476 | 258 | 1299 | 40 | 2 | 3 |
| 1474 | 9312477 | 232 | 1042 | 28 | 7 | 3 |
| 1475 | 9312478 | 191 | 770 | 17 | 3 | 3 |
| 1476 | 9312479 | 244 | 1302 | 31 | 6 | 3 |
| 1477 | 9312480 | 285 | 1228 | 31 | 2 | 4 |
| 1478 | 9312481 | 176 | 759 | 17 | 1 | 1 |
| 1479 | 9312482 | 256 | 1364 | 25 | 3 | 2 |
| 1480 | 9312483 | 204 | 1106 | 22 | 6 | 4 |
| 1481 | 9312484 | 238 | 1252 | 15 | 4 | 3 |
| 1482 | 9312485 | 177 | 732 | 16 | 3 | 2 |
| 1483 | 9312486 | 236 | 1077 | 21 | 3 | 2 |
| 1484 | 9312487 | 245 | 1198 | 13 | 4 | 6 |
| 1485 | 9312488 | 231 | 1143 | 27 | 1 | 2 |
| 1486 | 9312489 | 250 | 1100 | 24 | 5 | 5 |
| 1487 | 9312490 | 316 | 1714 | 29 | 4 | 3 |
| 1488 | 9312491 | 234 | 712 | 20 | 4 | 4 |
| 1489 | 9312492 | 245 | 1093 | 29 | 6 | 3 |
| 1490 | 9312493 | 281 | 1307 | 14 | 5 | 4 |
| 1491 | 9312494 | 302 | 1119 | 13 | 0 | 3 |
| 1492 | 9312495 | 220 | 955 | 12 | 6 | 3 |
| 1493 | 9312496 | 351 | 1644 | 46 | 4 | 5 |
| 1494 | 9312497 | 221 | 822 | 21 | 7 | 3 |
| 1495 | 9312498 | 201 | 860 | 18 | 1 | 3 |
| 1496 | 9312499 | 266 | 997 | 27 | 1 | 2 |
| 1497 | 9312500 | 242 | 808 | 25 | 14 | 3 |
| 1498 | 9312501 | 260 | 1144 | 22 | 2 | 3 |
| 1499 | 9312502 | 262 | 945 | 28 | 3 | 3 |
| 1500 | 9312503 | 229 | 1118 | 25 | 5 | 4 |
| 1501 | 9312504 | 231 | 841 | 23 | 3 | 4 |
| 1502 | 9312505 | 271 | 1186 | 12 | 0 | 5 |
| 1503 | 9312506 | 331 | 1391 | 38 | 6 | 3 |
| 1504 | 9312507 | 190 | 710 | 12 | 5 | 4 |
| 1505 | 9312508 | 181 | 831 | 13 | 1 | 1 |
| 1506 | 9312509 | 232 | 737 | 23 | 5 | 4 |
| 1507 | 9312510 | 265 | 1284 | 22 | 2 | 3 |
| 1508 | 9312511 | 257 | 1266 | 19 | 5 | 1 |
| 1509 | 9312512 | 242 | 977 | 16 | 1 | 3 |
| 1510 | 9312513 | 241 | 973 | 31 | 8 | 3 |
| 1511 | 9312514 | 216 | 1104 | 25 | 0 | 4 |
| 1512 | 9312515 | 263 | 1224 | 30 | 7 | 3 |
| 1513 | 9312516 | 273 | 1075 | 34 | 4 | 4 |
| 1514 | 9312517 | 198 | 904 | 19 | 2 | 3 |
| 1515 | 9312518 | 261 | 834 | 30 | 2 | 5 |
| 1516 | 9312519 | 245 | 1196 | 32 | 2 | 5 |
| 1517 | 9312520 | 215 | 827 | 24 | 7 | 3 |
| 1518 | 9312521 | 244 | 893 | 32 | 6 | 4 |
| 1519 | 9312522 | 268 | 1076 | 31 | 3 | 5 |
| 1520 | 9312523 | 260 | 1058 | 32 | 1 | 3 |
| 1521 | 9312524 | 248 | 957 | 24 | 1 | 3 |
| 1522 | 9312525 | 199 | 934 | 14 | 6 | 4 |
| 1523 | 9312526 | 182 | 882 | 5 | 3 | 2 |
| 1524 | 9312527 | 299 | 1104 | 17 | 0 | 6 |
| 1525 | 9312528 | 317 | 1358 | 28 | 2 | 3 |
| 1526 | 9312529 | 274 | 1413 | 22 | 6 | 4 |
| 1527 | 9312530 | 314 | 1294 | 31 | 3 | 3 |
| 1528 | 9312531 | 236 | 920 | 20 | 4 | 3 |
| 1529 | 9312532 | 244 | 1123 | 29 | 0 | 4 |
| 1530 | 9312533 | 351 | 1377 | 40 | 5 | 4 |
| 1531 | 9312534 | 226 | 850 | 15 | 0 | 2 |
| 1532 | 9312535 | 243 | 816 | 27 | 0 | 5 |
| 1533 | 9312536 | 375 | 1362 | 39 | 4 | 5 |
| 1534 | 9312537 | 204 | 651 | 10 | 5 | 5 |
| 1535 | 9312538 | 237 | 1001 | 28 | 1 | 4 |
| 1536 | 9312539 | 228 | 861 | 28 | 4 | 4 |
| 1537 | 9312540 | 245 | 744 | 25 | 3 | 3 |
| 1538 | 9312541 | 264 | 1261 | 38 | 0 | 4 |
| 1539 | 9312542 | 259 | 610 | 21 | 0 | 5 |
| 1540 | 9312543 | 297 | 1479 | 38 | 2 | 4 |
| 1541 | 9312544 | 296 | 1305 | 19 | 1 | 3 |
| 1542 | 9312545 | 232 | 872 | 19 | 0 | 3 |
| 1543 | 9312546 | 296 | 1680 | 32 | 0 | 4 |
| 1544 | 9312547 | 203 | 947 | 20 | 2 | 3 |
| 1545 | 9312548 | 174 | 688 | 14 | 3 | 3 |
| 1546 | 9312549 | 256 | 1056 | 28 | 1 | 4 |
| 1547 | 9312550 | 256 | 949 | 20 | 0 | 3 |
| 1548 | 9312551 | 245 | 1100 | 31 | 3 | 3 |
| 1549 | 9312552 | 251 | 1047 | 26 | 1 | 3 |
| 1550 | 9312553 | 216 | 1133 | 20 | 1 | 2 |
| 1551 | 9312554 | 273 | 1200 | 40 | 6 | 0 |
| 1552 | 9312555 | 260 | 1261 | 27 | 4 | 3 |
| 1553 | 9312556 | 346 | 1782 | 40 | 7 | 3 |
| 1554 | 9312557 | 362 | 1785 | 46 | 3 | 4 |
| 1555 | 9312558 | 324 | 1600 | 45 | 2 | 2 |
| 1556 | 9312559 | 202 | 858 | 30 | 2 | 5 |
| 1557 | 9312560 | 167 | 594 | 10 | 3 | 3 |
| 1558 | 9312561 | 260 | 1225 | 30 | 6 | 3 |
| 1559 | 9312562 | 215 | 874 | 32 | 3 | 2 |
| 1560 | 9312563 | 225 | 1105 | 21 | 2 | 2 |
| 1561 | 9312564 | 191 | 756 | 18 | 4 | 3 |
| 1562 | 9312565 | 273 | 1150 | 22 | 4 | 3 |
| 1563 | 9312566 | 281 | 1520 | 23 | 9 | 3 |
| 1564 | 9312567 | 249 | 1236 | 20 | 7 | 5 |
| 1565 | 9312568 | 233 | 1288 | 22 | 0 | 4 |
| 1566 | 9312569 | 208 | 719 | 17 | 1 | 3 |
| 1567 | 9312570 | 264 | 1203 | 28 | 0 | 5 |
| 1568 | 9312571 | 249 | 1325 | 28 | 1 | 3 |
| 1569 | 9312572 | 299 | 1539 | 17 | 3 | 4 |
| 1570 | 9312573 | 218 | 1142 | 20 | 4 | 5 |
| 1571 | 9312574 | 264 | 1241 | 20 | 6 | 2 |
| 1572 | 9312575 | 190 | 859 | 27 | 4 | 4 |
| 1573 | 9312576 | 241 | 1114 | 30 | 4 | 4 |
| 1574 | 9312577 | 206 | 898 | 26 | 0 | 3 |
| 1575 | 9312578 | 298 | 1355 | 18 | 3 | 4 |
| 1576 | 9312579 | 266 | 1056 | 14 | 4 | 5 |
| 1577 | 9312580 | 218 | 818 | 23 | 2 | 2 |
| 1578 | 9312581 | 214 | 738 | 16 | 8 | 4 |
| 1579 | 9312582 | 265 | 802 | 25 | 3 | 3 |
| 1580 | 9312583 | 171 | 733 | 15 | 5 | 4 |
| 1581 | 9312584 | 300 | 1274 | 32 | 3 | 4 |
| 1582 | 9312585 | 249 | 1181 | 34 | 4 | 3 |
| 1583 | 9312586 | 270 | 1132 | 42 | 5 | 4 |
| 1584 | 9312587 | 263 | 1037 | 19 | 4 | 2 |
| 1585 | 9312588 | 203 | 748 | 17 | 3 | 1 |
| 1586 | 9312589 | 244 | 1061 | 24 | 6 | 3 |
| 1587 | 9312590 | 284 | 1247 | 33 | 5 | 5 |
| 1588 | 9312591 | 204 | 901 | 15 | 2 | 1 |
| 1589 | 9312592 | 332 | 1175 | 33 | 3 | 5 |
| 1590 | 9312593 | 199 | 690 | 24 | 1 | 5 |
| 1591 | 9312594 | 357 | 1316 | 41 | 5 | 3 |
| 1592 | 9312595 | 218 | 963 | 14 | 3 | 1 |
| 1593 | 9312596 | 221 | 1134 | 14 | 3 | 4 |
| 1594 | 9312597 | 231 | 1064 | 23 | 2 | 5 |
| 1595 | 9312598 | 226 | 811 | 28 | 4 | 2 |
| 1596 | 9312599 | 271 | 1018 | 34 | 7 | 4 |
| 1597 | 9312600 | 249 | 1235 | 24 | 4 | 2 |
| 1598 | 9312601 | 263 | 1211 | 34 | 6 | 4 |
| 1599 | 9312602 | 162 | 484 | 14 | 2 | 3 |
| 1600 | 9312603 | 259 | 1096 | 21 | 3 | 3 |
| 1601 | 9312604 | 285 | 1206 | 41 | 5 | 6 |
| 1602 | 9312605 | 259 | 1458 | 27 | 3 | 2 |
| 1603 | 9312606 | 419 | 1906 | 41 | 10 | 4 |
| 1604 | 9312607 | 319 | 892 | 20 | 2 | 3 |
| 1605 | 9312608 | 335 | 2008 | 30 | 0 | 4 |
| 1606 | 9312609 | 228 | 911 | 28 | 3 | 2 |
| 1607 | 9312610 | 231 | 1093 | 24 | 5 | 2 |
| 1608 | 9312611 | 248 | 909 | 26 | 5 | 6 |
| 1609 | 9312612 | 283 | 1273 | 33 | 1 | 4 |
| 1610 | 9312613 | 316 | 1204 | 23 | 8 | 5 |
| 1611 | 9312614 | 254 | 1105 | 20 | 8 | 4 |
| 1612 | 9312615 | 239 | 1235 | 29 | 1 | 2 |
| 1613 | 9312616 | 206 | 762 | 19 | 3 | 2 |
| 1614 | 9312617 | 268 | 1234 | 31 | 2 | 5 |
| 1615 | 9312618 | 235 | 1112 | 23 | 3 | 3 |
| 1616 | 9312619 | 270 | 1352 | 34 | 2 | 3 |
| 1617 | 9312620 | 159 | 724 | 13 | 2 | 2 |
| 1618 | 9312621 | 249 | 1310 | 30 | 1 | 4 |
| 1619 | 9312622 | 352 | 1689 | 40 | 1 | 4 |
| 1620 | 9312623 | 180 | 713 | 21 | 3 | 2 |
| 1621 | 9312624 | 217 | 1094 | 32 | 4 | 5 |
| 1622 | 9312625 | 235 | 1097 | 30 | 1 | 2 |
| 1623 | 9312626 | 251 | 1184 | 41 | 1 | 2 |
| 1624 | 9312627 | 208 | 864 | 22 | 4 | 3 |
| 1625 | 9312628 | 194 | 858 | 22 | 1 | 4 |
| 1626 | 9312629 | 178 | 898 | 20 | 2 | 2 |
| 1627 | 9312630 | 258 | 1099 | 27 | 5 | 4 |
| 1628 | 9312631 | 315 | 1257 | 33 | 2 | 5 |
| 1629 | 9312632 | 230 | 1178 | 23 | 1 | 3 |
| 1630 | 9312633 | 295 | 1539 | 47 | 1 | 1 |
| 1631 | 9312634 | 302 | 957 | 24 | 2 | 4 |
| 1632 | 9312635 | 316 | 1292 | 40 | 5 | 4 |
| 1633 | 9312636 | 203 | 1015 | 28 | 0 | 4 |
| 1634 | 9312637 | 228 | 767 | 20 | 2 | 2 |
| 1635 | 9312638 | 273 | 1348 | 35 | 0 | 3 |
| 1636 | 9312639 | 189 | 762 | 19 | 2 | 2 |
| 1637 | 9312640 | 206 | 640 | 16 | 2 | 3 |
| 1638 | 9312641 | 275 | 1297 | 27 | 1 | 4 |
| 1639 | 9312642 | 213 | 1018 | 26 | 2 | 2 |
| 1640 | 9312643 | 314 | 1289 | 31 | 2 | 4 |
| 1641 | 9312644 | 234 | 913 | 20 | 2 | 3 |
| 1642 | 9312645 | 197 | 914 | 23 | 0 | 3 |
| 1643 | 9312646 | 415 | 1835 | 58 | 9 | 3 |
| 1644 | 9312647 | 308 | 1719 | 40 | 3 | 3 |
| 1645 | 9312648 | 184 | 809 | 23 | 0 | 3 |
| 1646 | 9312649 | 457 | 2158 | 51 | 5 | 3 |
| 1647 | 9312650 | 291 | 1076 | 31 | 3 | 3 |
| 1648 | 9312651 | 309 | 1541 | 49 | 0 | 3 |
| 1649 | 9312652 | 206 | 1020 | 20 | 5 | 3 |
| 1650 | 9312653 | 190 | 748 | 13 | 0 | 2 |
| 1651 | 9312654 | 208 | 862 | 17 | 2 | 1 |
| 1652 | 9312655 | 205 | 849 | 20 | 6 | 1 |
| 1653 | 9312656 | 231 | 937 | 16 | 4 | 4 |
| 1654 | 9312657 | 215 | 1147 | 26 | 4 | 3 |
| 1655 | 9312658 | 219 | 922 | 21 | 5 | 4 |
| 1656 | 9312659 | 240 | 1315 | 23 | 5 | 2 |
| 1657 | 9312660 | 187 | 903 | 12 | 4 | 4 |
| 1658 | 9312661 | 245 | 1020 | 20 | 5 | 2 |
| 1659 | 9312662 | 217 | 742 | 18 | 0 | 2 |
| 1660 | 9312663 | 271 | 1336 | 28 | 2 | 5 |
| 1661 | 9312664 | 213 | 999 | 24 | 8 | 2 |
| 1662 | 9312665 | 202 | 967 | 26 | 7 | 3 |
| 1663 | 9312666 | 381 | 1506 | 42 | 7 | 6 |
| 1664 | 9312667 | 199 | 907 | 14 | 3 | 3 |
| 1665 | 9312668 | 269 | 961 | 27 | 4 | 4 |
| 1666 | 9312669 | 233 | 851 | 16 | 3 | 5 |
| 1667 | 9312670 | 288 | 1429 | 43 | 4 | 2 |
| 1668 | 9312671 | 248 | 629 | 15 | 8 | 4 |
| 1669 | 9312672 | 352 | 1526 | 27 | 5 | 3 |
| 1670 | 9312673 | 285 | 1192 | 34 | 3 | 4 |
| 1671 | 9312674 | 209 | 666 | 12 | 2 | 5 |
| 1672 | 9312675 | 193 | 865 | 15 | 5 | 2 |
| 1673 | 9312676 | 289 | 1217 | 29 | 2 | 4 |
| 1674 | 9312677 | 226 | 777 | 32 | 1 | 5 |
| 1675 | 9312678 | 240 | 1051 | 23 | 5 | 1 |
| 1676 | 9312679 | 275 | 1114 | 11 | 3 | 3 |
| 1677 | 9312680 | 188 | 786 | 24 | 4 | 3 |
| 1678 | 9312681 | 338 | 1502 | 36 | 1 | 3 |
| 1679 | 9312682 | 298 | 1246 | 31 | 2 | 5 |
| 1680 | 9312683 | 255 | 1172 | 24 | 6 | 1 |
| 1681 | 9312684 | 257 | 951 | 22 | 0 | 3 |
| 1682 | 9312685 | 251 | 1251 | 10 | 5 | 1 |
| 1683 | 9312686 | 327 | 1589 | 26 | 5 | 3 |
| 1684 | 9312687 | 327 | 1416 | 39 | 3 | 5 |
| 1685 | 9312688 | 288 | 1100 | 30 | 3 | 4 |
| 1686 | 9312689 | 220 | 910 | 29 | 1 | 3 |
| 1687 | 9312690 | 307 | 1302 | 24 | 5 | 4 |
| 1688 | 9312691 | 413 | 1328 | 28 | 7 | 5 |
| 1689 | 9312692 | 278 | 1088 | 31 | 7 | 3 |
| 1690 | 9312693 | 256 | 1205 | 22 | 10 | 3 |
| 1691 | 9312694 | 241 | 1172 | 26 | 3 | 4 |
| 1692 | 9312695 | 204 | 919 | 13 | 6 | 3 |
| 1693 | 9312696 | 251 | 980 | 26 | 2 | 4 |
| 1694 | 9312697 | 176 | 601 | 14 | 3 | 2 |
| 1695 | 9312698 | 249 | 1012 | 32 | 7 | 3 |
| 1696 | 9312699 | 273 | 1127 | 35 | 1 | 5 |
| 1697 | 9312700 | 184 | 659 | 14 | 1 | 4 |
| 1698 | 9312701 | 279 | 1457 | 26 | 4 | 4 |
| 1699 | 9312702 | 231 | 819 | 17 | 3 | 4 |
| 1700 | 9312703 | 231 | 1034 | 22 | 9 | 4 |
| 1701 | 9312704 | 232 | 1166 | 24 | 5 | 3 |
| 1702 | 9312705 | 331 | 1481 | 41 | 4 | 3 |
| 1703 | 9312706 | 228 | 896 | 23 | 5 | 4 |
| 1704 | 9312707 | 305 | 1194 | 35 | 1 | 4 |
| 1705 | 9312708 | 332 | 1446 | 39 | 6 | 2 |
| 1706 | 9312709 | 233 | 933 | 23 | 4 | 3 |
| 1707 | 9312710 | 215 | 768 | 16 | 9 | 3 |
| 1708 | 9312711 | 300 | 1055 | 26 | 4 | 5 |
| 1709 | 9312712 | 290 | 1246 | 38 | 1 | 2 |
| 1710 | 9312713 | 382 | 1405 | 28 | 2 | 4 |
| 1711 | 9312714 | 233 | 886 | 17 | 0 | 4 |
| 1712 | 9312715 | 306 | 1405 | 41 | 0 | 5 |
| 1713 | 9312716 | 250 | 1263 | 26 | 1 | 5 |
| 1714 | 9312717 | 247 | 1019 | 22 | 2 | 5 |
| 1715 | 9312718 | 234 | 1271 | 28 | 1 | 2 |
| 1716 | 9312719 | 214 | 756 | 22 | 3 | 4 |
| 1717 | 9312720 | 181 | 609 | 19 | 4 | 3 |
| 1718 | 9312721 | 317 | 1102 | 31 | 3 | 6 |
| 1719 | 9312722 | 237 | 1067 | 29 | 2 | 4 |
| 1720 | 9312723 | 434 | 1851 | 56 | 0 | 6 |
| 1721 | 9312724 | 240 | 801 | 22 | 2 | 5 |
| 1722 | 9312725 | 196 | 754 | 21 | 3 | 3 |
| 1723 | 9312726 | 260 | 1154 | 31 | 0 | 2 |
| 1724 | 9312727 | 229 | 692 | 19 | 6 | 3 |
| 1725 | 9312728 | 222 | 1132 | 39 | 1 | 4 |
| 1726 | 9312729 | 281 | 1321 | 34 | 0 | 5 |
| 1727 | 9312730 | 207 | 1013 | 29 | 2 | 5 |
| 1728 | 9312731 | 213 | 949 | 20 | 1 | 2 |
| 1729 | 9312732 | 233 | 894 | 17 | 5 | 1 |
| 1730 | 9312733 | 201 | 805 | 25 | 1 | 0 |
| 1731 | 9312734 | 284 | 1707 | 32 | 6 | 2 |
| 1732 | 9312735 | 321 | 1569 | 37 | 2 | 2 |
| 1733 | 9312736 | 170 | 575 | 16 | 6 | 2 |
| 1734 | 9312737 | 244 | 1137 | 33 | 2 | 6 |
| 1735 | 9312738 | 383 | 1753 | 58 | 4 | 3 |
| 1736 | 9312739 | 183 | 770 | 16 | 3 | 2 |
| 1737 | 9312740 | 203 | 1046 | 20 | 2 | 4 |
| 1738 | 9312741 | 246 | 1466 | 35 | 0 | 3 |
| 1739 | 9312742 | 165 | 658 | 12 | 7 | 1 |
| 1740 | 9312743 | 260 | 1381 | 41 | 1 | 5 |
| 1741 | 9312744 | 323 | 1421 | 39 | 4 | 4 |
| 1742 | 9312745 | 181 | 700 | 24 | 3 | 2 |
| 1743 | 9312746 | 189 | 620 | 11 | 3 | 1 |
| 1744 | 9312747 | 255 | 1091 | 23 | 6 | 3 |
| 1745 | 9312748 | 304 | 1339 | 26 | 2 | 1 |
| 1746 | 9312749 | 344 | 1845 | 42 | 12 | 2 |
| 1747 | 9312750 | 266 | 1450 | 33 | 6 | 3 |
| 1748 | 9312751 | 264 | 1473 | 34 | 5 | 2 |
| 1749 | 9312752 | 231 | 710 | 17 | 4 | 3 |
| 1750 | 9312753 | 278 | 1320 | 33 | 8 | 1 |
| 1751 | 9312754 | 225 | 948 | 24 | 2 | 1 |
| 1752 | 9312755 | 220 | 753 | 17 | 0 | 2 |
| 1753 | 9312756 | 223 | 880 | 22 | 3 | 4 |
| 1754 | 9312757 | 218 | 657 | 24 | 3 | 3 |
| 1755 | 9312758 | 231 | 951 | 34 | 1 | 4 |
| 1756 | 9312759 | 203 | 533 | 6 | 0 | 2 |
| 1757 | 9312760 | 249 | 1078 | 17 | 2 | 5 |
| 1758 | 9312761 | 253 | 1022 | 20 | 0 | 3 |
| 1759 | 9312762 | 377 | 1325 | 35 | 1 | 5 |
| 1760 | 9312763 | 216 | 843 | 17 | 9 | 4 |
| 1761 | 9312764 | 237 | 975 | 15 | 11 | 5 |
| 1762 | 9312765 | 221 | 677 | 19 | 3 | 5 |
| 1763 | 9312766 | 328 | 1489 | 39 | 5 | 3 |
| 1764 | 9312767 | 164 | 575 | 16 | 1 | 2 |
| 1765 | 9312768 | 182 | 684 | 14 | 4 | 2 |
| 1766 | 9312769 | 238 | 1120 | 11 | 1 | 4 |
| 1767 | 9312770 | 232 | 1094 | 20 | 5 | 3 |
| 1768 | 9312771 | 227 | 969 | 27 | 7 | 2 |
| 1769 | 9312772 | 276 | 1374 | 32 | 2 | 5 |
| 1770 | 9312773 | 247 | 942 | 29 | 0 | 5 |
| 1771 | 9312774 | 269 | 1049 | 29 | 7 | 4 |
| 1772 | 9312775 | 220 | 949 | 11 | 6 | 3 |
| 1773 | 9312776 | 261 | 976 | 28 | 3 | 3 |
| 1774 | 9312777 | 354 | 1403 | 32 | 3 | 5 |
| 1775 | 9312778 | 262 | 1215 | 32 | 6 | 5 |
| 1776 | 9312779 | 289 | 1105 | 25 | 3 | 2 |
| 1777 | 9312780 | 328 | 1319 | 35 | 4 | 4 |
| 1778 | 9312781 | 307 | 1075 | 49 | 6 | 5 |
| 1779 | 9312782 | 275 | 1116 | 28 | 7 | 3 |
| 1780 | 9312783 | 254 | 1305 | 22 | 6 | 1 |
| 1781 | 9312784 | 364 | 1189 | 25 | 6 | 5 |
| 1782 | 9312785 | 226 | 1002 | 28 | 3 | 4 |
| 1783 | 9312786 | 204 | 951 | 23 | 1 | 1 |
| 1784 | 9312787 | 251 | 1337 | 38 | 2 | 1 |
| 1785 | 9312788 | 244 | 926 | 25 | 3 | 6 |
| 1786 | 9312789 | 233 | 855 | 22 | 1 | 3 |
| 1787 | 9312790 | 303 | 1291 | 30 | 3 | 4 |
| 1788 | 9312791 | 175 | 637 | 23 | 3 | 2 |
| 1789 | 9312792 | 259 | 1227 | 29 | 1 | 4 |
| 1790 | 9312793 | 267 | 1207 | 19 | 2 | 2 |
| 1791 | 9312794 | 249 | 976 | 21 | 4 | 5 |
| 1792 | 9312795 | 276 | 1406 | 37 | 5 | 2 |
| 1793 | 9312796 | 248 | 865 | 20 | 4 | 4 |
| 1794 | 9312797 | 232 | 1066 | 23 | 6 | 2 |
| 1795 | 9312798 | 256 | 1260 | 28 | 2 | 4 |
| 1796 | 9312799 | 278 | 1057 | 31 | 2 | 3 |
| 1797 | 9312800 | 240 | 716 | 24 | 1 | 5 |
| 1798 | 9312801 | 254 | 1101 | 16 | 4 | 2 |
| 1799 | 9312802 | 213 | 922 | 14 | 6 | 4 |
| 1800 | 9312803 | 178 | 560 | 14 | 0 | 1 |
| 1801 | 9312804 | 408 | 1705 | 37 | 2 | 4 |
| 1802 | 9312805 | 255 | 1042 | 27 | 4 | 3 |
| 1803 | 9312806 | 323 | 1271 | 28 | 0 | 3 |
| 1804 | 9312807 | 193 | 686 | 16 | 3 | 2 |
| 1805 | 9312808 | 354 | 1220 | 38 | 3 | 5 |
| 1806 | 9312809 | 429 | 2093 | 65 | 4 | 4 |
| 1807 | 9312810 | 472 | 1534 | 42 | 0 | 6 |
| 1808 | 9312811 | 204 | 644 | 22 | 3 | 5 |
| 1809 | 9312812 | 199 | 850 | 18 | 3 | 2 |
| 1810 | 9312813 | 227 | 979 | 21 | 2 | 3 |
| 1811 | 9312814 | 248 | 920 | 21 | 3 | 4 |
| 1812 | 9312815 | 500 | 2080 | 42 | 6 | 4 |
| 1813 | 9312816 | 259 | 1014 | 33 | 2 | 4 |
| 1814 | 9312817 | 211 | 666 | 24 | 4 | 1 |
| 1815 | 9312818 | 178 | 812 | 10 | 4 | 1 |
| 1816 | 9312819 | 204 | 964 | 20 | 6 | 3 |
| 1817 | 9312820 | 264 | 1244 | 39 | 2 | 4 |
| 1818 | 9312821 | 190 | 780 | 23 | 3 | 4 |
| 1819 | 9312822 | 164 | 584 | 14 | 2 | 3 |
| 1820 | 9312823 | 228 | 1073 | 31 | 2 | 3 |
| 1821 | 9312824 | 233 | 1139 | 33 | 4 | 3 |
| 1822 | 9312825 | 256 | 1160 | 34 | 9 | 4 |
| 1823 | 9312826 | 186 | 753 | 22 | 6 | 3 |
| 1824 | 9312827 | 241 | 1288 | 23 | 9 | 3 |
| 1825 | 9312828 | 305 | 1298 | 32 | 8 | 1 |
| 1826 | 9312829 | 230 | 1204 | 26 | 2 | 2 |
| 1827 | 9312830 | 218 | 844 | 26 | 2 | 3 |
| 1828 | 9312831 | 268 | 1379 | 40 | 2 | 4 |
| 1829 | 9312832 | 299 | 1151 | 29 | 2 | 4 |
| 1830 | 9312833 | 202 | 848 | 20 | 2 | 1 |
| 1831 | 9312834 | 306 | 1453 | 40 | 4 | 4 |
| 1832 | 9312835 | 232 | 962 | 22 | 3 | 1 |
| 1833 | 9312836 | 358 | 1612 | 36 | 1 | 4 |
| 1834 | 9312837 | 190 | 964 | 24 | 2 | 2 |
| 1835 | 9312838 | 253 | 1140 | 31 | 1 | 2 |
| 1836 | 9312839 | 278 | 1181 | 32 | 3 | 3 |
| 1837 | 9312840 | 247 | 959 | 29 | 3 | 3 |
| 1838 | 9312841 | 191 | 913 | 11 | 7 | 2 |
| 1839 | 9312842 | 240 | 927 | 25 | 2 | 3 |
| 1840 | 9312843 | 263 | 964 | 23 | 2 | 4 |
| 1841 | 9312844 | 260 | 993 | 25 | 6 | 4 |
| 1842 | 9312845 | 387 | 1553 | 31 | 7 | 4 |
| 1843 | 9312846 | 244 | 992 | 28 | 0 | 4 |
| 1844 | 9312847 | 285 | 1346 | 32 | 2 | 3 |
| 1845 | 9312848 | 231 | 846 | 27 | 7 | 4 |
| 1846 | 9312849 | 256 | 865 | 25 | 1 | 5 |
| 1847 | 9312850 | 337 | 1337 | 32 | 1 | 5 |
| 1848 | 9312851 | 230 | 938 | 27 | 3 | 3 |
| 1849 | 9312852 | 278 | 1230 | 31 | 1 | 4 |
| 1850 | 9312853 | 274 | 950 | 21 | 7 | 3 |
| 1851 | 9312854 | 244 | 1029 | 36 | 5 | 5 |
| 1852 | 9312855 | 210 | 837 | 26 | 3 | 1 |
| 1853 | 9312856 | 254 | 963 | 16 | 3 | 3 |
| 1854 | 9312857 | 276 | 1352 | 33 | 2 | 1 |
| 1855 | 9312858 | 232 | 887 | 18 | 3 | 2 |
| 1856 | 9312859 | 289 | 1213 | 34 | 6 | 4 |
| 1857 | 9312860 | 211 | 826 | 19 | 10 | 3 |
| 1858 | 9312861 | 197 | 889 | 15 | 7 | 2 |
| 1859 | 9312862 | 231 | 919 | 32 | 1 | 4 |
| 1860 | 9312863 | 230 | 879 | 18 | 2 | 2 |
| 1861 | 9312864 | 204 | 762 | 23 | 4 | 1 |
| 1862 | 9312865 | 176 | 665 | 11 | 2 | 2 |
| 1863 | 9312866 | 199 | 651 | 21 | 1 | 4 |
| 1864 | 9312867 | 341 | 1726 | 27 | 0 | 4 |
| 1865 | 9312868 | 237 | 1023 | 31 | 7 | 2 |
| 1866 | 9312869 | 275 | 1037 | 37 | 5 | 4 |
| 1867 | 9312870 | 240 | 878 | 36 | 3 | 3 |
| 1868 | 9312871 | 220 | 998 | 15 | 5 | 3 |
| 1869 | 9312872 | 263 | 1058 | 36 | 0 | 4 |
| 1870 | 9312873 | 307 | 1238 | 33 | 4 | 3 |
| 1871 | 9312874 | 207 | 820 | 17 | 6 | 2 |
| 1872 | 9312875 | 285 | 1477 | 39 | 3 | 3 |
| 1873 | 9312876 | 274 | 1282 | 33 | 8 | 5 |
| 1874 | 9312877 | 238 | 914 | 25 | 3 | 3 |
| 1875 | 9312878 | 201 | 709 | 17 | 1 | 2 |
| 1876 | 9312879 | 174 | 637 | 9 | 1 | 2 |
| 1877 | 9312880 | 296 | 1505 | 28 | 10 | 3 |
| 1878 | 9312881 | 274 | 1277 | 36 | 1 | 3 |
| 1879 | 9312882 | 249 | 1111 | 21 | 8 | 2 |
| 1880 | 9312883 | 250 | 1127 | 14 | 6 | 4 |
| 1881 | 9312884 | 262 | 1011 | 24 | 3 | 4 |
| 1882 | 9312885 | 404 | 1976 | 51 | 4 | 4 |
| 1883 | 9312886 | 393 | 1716 | 27 | 5 | 3 |
| 1884 | 9312887 | 259 | 1310 | 18 | 3 | 5 |
| 1885 | 9312888 | 237 | 1169 | 27 | 0 | 2 |
| 1886 | 9312889 | 411 | 1614 | 44 | 2 | 5 |
| 1887 | 9312890 | 236 | 1067 | 21 | 5 | 3 |
| 1888 | 9312891 | 233 | 1106 | 23 | 4 | 2 |
| 1889 | 9312892 | 228 | 1037 | 23 | 4 | 4 |
| 1890 | 9312893 | 243 | 1037 | 26 | 0 | 4 |
| 1891 | 9312894 | 235 | 1012 | 30 | 2 | 3 |
| 1892 | 9312895 | 300 | 1214 | 25 | 3 | 3 |
| 1893 | 9312896 | 222 | 1033 | 20 | 5 | 2 |
| 1894 | 9312897 | 282 | 1279 | 32 | 2 | 4 |
| 1895 | 9312898 | 224 | 792 | 16 | 4 | 3 |
| 1896 | 9312899 | 275 | 1287 | 46 | 4 | 1 |
| 1897 | 9312900 | 262 | 1320 | 32 | 4 | 1 |
| 1898 | 9312901 | 186 | 932 | 21 | 2 | 1 |
| 1899 | 9312902 | 202 | 725 | 20 | 3 | 3 |
| 1900 | 9312903 | 223 | 993 | 30 | 5 | 4 |
| 1901 | 9312904 | 256 | 1141 | 27 | 3 | 2 |
| 1902 | 9312905 | 233 | 1036 | 30 | 4 | 3 |
| 1903 | 9312906 | 212 | 999 | 25 | 2 | 1 |
| 1904 | 9312907 | 235 | 882 | 23 | 2 | 3 |
| 1905 | 9312908 | 213 | 1172 | 32 | 1 | 3 |
| 1906 | 9312909 | 312 | 1838 | 29 | 4 | 2 |
| 1907 | 9312910 | 215 | 843 | 20 | 8 | 2 |
| 1908 | 9312911 | 208 | 1002 | 28 | 2 | 3 |
| 1909 | 9312912 | 200 | 862 | 28 | 3 | 2 |
| 1910 | 9312913 | 251 | 1269 | 28 | 2 | 4 |
| 1911 | 9312914 | 177 | 766 | 22 | 3 | 3 |
| 1912 | 9312915 | 257 | 1197 | 26 | 1 | 3 |
| 1913 | 9312916 | 224 | 1120 | 11 | 3 | 2 |
| 1914 | 9312917 | 193 | 744 | 23 | 3 | 1 |
| 1915 | 9312918 | 241 | 1135 | 21 | 3 | 0 |
| 1916 | 9312919 | 248 | 1238 | 26 | 2 | 2 |
| 1917 | 9312920 | 229 | 1040 | 30 | 5 | 3 |
| 1918 | 9312921 | 273 | 1839 | 27 | 2 | 2 |
| 1919 | 9312922 | 283 | 1409 | 33 | 3 | 1 |
| 1920 | 9312923 | 214 | 1056 | 13 | 5 | 1 |
| 1921 | 9312924 | 258 | 1412 | 22 | 5 | 3 |
| 1922 | 9312925 | 195 | 1003 | 17 | 4 | 2 |
| 1923 | 9312926 | 200 | 781 | 16 | 5 | 4 |
| 1924 | 9312927 | 294 | 1312 | 26 | 4 | 0 |
| 1925 | 9312928 | 186 | 656 | 13 | 2 | 3 |
| 1926 | 9312929 | 297 | 1527 | 42 | 4 | 5 |
| 1927 | 9312930 | 372 | 1536 | 40 | 5 | 4 |
| 1928 | 9312931 | 227 | 934 | 32 | 1 | 4 |
| 1929 | 9312932 | 350 | 1375 | 53 | 3 | 4 |
| 1930 | 9312933 | 270 | 1255 | 27 | 1 | 3 |
| 1931 | 9312934 | 213 | 768 | 15 | 5 | 4 |
| 1932 | 9312935 | 220 | 878 | 13 | 5 | 2 |
| 1933 | 9312936 | 253 | 1118 | 27 | 2 | 4 |
| 1934 | 9312937 | 290 | 962 | 41 | 1 | 2 |
| 1935 | 9312938 | 220 | 1137 | 25 | 4 | 5 |
| 1936 | 9312939 | 229 | 629 | 21 | 2 | 2 |
| 1937 | 9312940 | 248 | 1099 | 21 | 7 | 5 |
| 1938 | 9312941 | 236 | 889 | 29 | 4 | 2 |
| 1939 | 9312942 | 246 | 1027 | 35 | 2 | 2 |
| 1940 | 9312943 | 303 | 1255 | 32 | 6 | 4 |
| 1941 | 9312944 | 401 | 1659 | 31 | 10 | 3 |
| 1942 | 9312945 | 219 | 1023 | 26 | 3 | 3 |
| 1943 | 9312946 | 286 | 1059 | 23 | 4 | 4 |
| 1944 | 9312947 | 266 | 1097 | 12 | 4 | 4 |
| 1945 | 9312948 | 212 | 630 | 17 | 4 | 4 |
| 1946 | 9312949 | 226 | 1010 | 15 | 0 | 4 |
| 1947 | 9312950 | 218 | 831 | 15 | 6 | 3 |
| 1948 | 9312951 | 220 | 967 | 25 | 2 | 2 |
| 1949 | 9312952 | 250 | 891 | 40 | 2 | 4 |
| 1950 | 9312953 | 328 | 1343 | 50 | 5 | 4 |
| 1951 | 9312954 | 312 | 1389 | 35 | 5 | 4 |
| 1952 | 9312955 | 169 | 716 | 16 | 2 | 3 |
| 1953 | 9312956 | 247 | 979 | 26 | 4 | 3 |
| 1954 | 9312957 | 194 | 748 | 21 | 4 | 2 |
| 1955 | 9312958 | 166 | 546 | 16 | 0 | 3 |
| 1956 | 9312959 | 324 | 1444 | 32 | 4 | 2 |
| 1957 | 9312960 | 212 | 844 | 27 | 3 | 3 |
| 1958 | 9312961 | 205 | 538 | 11 | 2 | 3 |
| 1959 | 9312962 | 306 | 1411 | 43 | 2 | 4 |
| 1960 | 9312963 | 184 | 719 | 18 | 5 | 3 |
| 1961 | 9312964 | 259 | 1285 | 31 | 2 | 3 |
| 1962 | 9312965 | 243 | 833 | 25 | 8 | 3 |
| 1963 | 9312966 | 236 | 739 | 28 | 1 | 4 |
| 1964 | 9312967 | 238 | 1106 | 36 | 0 | 4 |
| 1965 | 9312968 | 222 | 613 | 15 | 0 | 5 |
| 1966 | 9312969 | 285 | 1378 | 35 | 2 | 5 |
| 1967 | 9312970 | 274 | 1172 | 40 | 4 | 4 |
| 1968 | 9312971 | 240 | 990 | 14 | 1 | 4 |
| 1969 | 9312972 | 267 | 1252 | 19 | 1 | 4 |
| 1970 | 9312973 | 230 | 897 | 23 | 0 | 2 |
| 1971 | 9312974 | 341 | 1803 | 40 | 3 | 4 |
| 1972 | 9312975 | 192 | 761 | 17 | 6 | 2 |
| 1973 | 9312976 | 298 | 1645 | 30 | 1 | 2 |
| 1974 | 9312977 | 231 | 1194 | 10 | 4 | 2 |
| 1975 | 9312978 | 230 | 887 | 24 | 2 | 3 |
| 1976 | 9312979 | 264 | 1341 | 35 | 0 | 3 |
| 1977 | 9312980 | 333 | 1524 | 45 | 4 | 4 |
| 1978 | 9312981 | 237 | 1053 | 26 | 2 | 4 |
| 1979 | 9312982 | 253 | 949 | 35 | 1 | 4 |
| 1980 | 9312983 | 193 | 997 | 19 | 2 | 2 |
| 1981 | 9312984 | 269 | 1240 | 38 | 2 | 4 |
| 1982 | 9312985 | 297 | 1509 | 34 | 1 | 2 |
| 1983 | 9312986 | 230 | 778 | 21 | 1 | 4 |
| 1984 | 9312987 | 210 | 698 | 16 | 3 | 3 |
| 1985 | 9312988 | 283 | 1209 | 38 | 2 | 5 |
| 1986 | 9312989 | 267 | 737 | 21 | 3 | 4 |
| 1987 | 9312990 | 228 | 868 | 33 | 1 | 4 |
| 1988 | 9312991 | 258 | 1007 | 26 | 3 | 2 |
| 1989 | 9312992 | 272 | 1114 | 31 | 0 | 5 |
| 1990 | 9312993 | 230 | 1052 | 31 | 9 | 2 |
| 1991 | 9312994 | 255 | 1143 | 29 | 1 | 4 |
| 1992 | 9312995 | 218 | 1248 | 26 | 1 | 2 |
| 1993 | 9312996 | 172 | 642 | 12 | 4 | 1 |
| 1994 | 9312997 | 230 | 919 | 32 | 1 | 3 |
| 1995 | 9312998 | 255 | 1298 | 36 | 1 | 2 |
| 1996 | 9312999 | 281 | 1274 | 46 | 5 | 5 |
| 1997 | 9313000 | 269 | 1421 | 37 | 4 | 3 |
| 1998 | 9313001 | 184 | 866 | 17 | 4 | 1 |
| 1999 | 9313002 | 253 | 1342 | 29 | 5 | 2 |
| 2000 | 9313003 | 188 | 800 | 17 | 0 | 2 |
| 2001 | 9313004 | 271 | 1516 | 35 | 3 | 2 |
| 2002 | 9313005 | 270 | 1077 | 24 | 3 | 4 |
| 2003 | 9313006 | 190 | 955 | 24 | 2 | 4 |
| 2004 | 9313007 | 254 | 1137 | 22 | 6 | 3 |
| 2005 | 9313008 | 189 | 741 | 20 | 6 | 2 |
| 2006 | 9313009 | 173 | 786 | 13 | 3 | 1 |
| 2007 | 9313010 | 203 | 951 | 24 | 5 | 3 |
| 2008 | 9313011 | 292 | 1161 | 36 | 0 | 4 |
| 2009 | 9313012 | 287 | 1585 | 28 | 3 | 4 |
| 2010 | 9313013 | 286 | 1337 | 40 | 10 | 5 |
| 2011 | 9313014 | 500 | 2133 | 53 | 4 | 3 |
| 2012 | 9313015 | 267 | 1087 | 22 | 2 | 3 |
| 2013 | 9313016 | 298 | 1353 | 23 | 3 | 4 |
| 2014 | 9313017 | 249 | 1045 | 19 | 6 | 5 |
| 2015 | 9313018 | 290 | 1633 | 32 | 7 | 3 |
| 2016 | 9313019 | 323 | 1216 | 28 | 7 | 4 |
| 2017 | 9313020 | 288 | 1351 | 26 | 3 | 3 |
| 2018 | 9313021 | 277 | 1114 | 25 | 2 | 4 |
| 2019 | 9313022 | 207 | 605 | 15 | 1 | 4 |
| 2020 | 9313023 | 226 | 884 | 22 | 1 | 3 |
| 2021 | 9313024 | 286 | 1355 | 25 | 2 | 4 |
| 2022 | 9313025 | 218 | 1083 | 23 | 1 | 4 |
| 2023 | 9313026 | 249 | 927 | 39 | 0 | 4 |
| 2024 | 9313027 | 273 | 1113 | 37 | 5 | 3 |
| 2025 | 9313028 | 243 | 1334 | 28 | 2 | 3 |
| 2026 | 9313029 | 232 | 1016 | 24 | 2 | 4 |
| 2027 | 9313030 | 269 | 1116 | 36 | 4 | 3 |
| 2028 | 9313031 | 313 | 1497 | 35 | 4 | 4 |
| 2029 | 9313032 | 240 | 884 | 18 | 1 | 4 |
| 2030 | 9313033 | 271 | 1111 | 14 | 3 | 4 |
| 2031 | 9313034 | 500 | 1800 | 23 | 2 | 5 |
| 2032 | 9313035 | 233 | 1084 | 35 | 1 | 2 |
| 2033 | 9313036 | 273 | 1223 | 39 | 4 | 4 |
| 2034 | 9313037 | 245 | 1389 | 38 | 3 | 2 |
| 2035 | 9313038 | 278 | 1133 | 30 | 6 | 3 |
| 2036 | 9313039 | 301 | 1369 | 33 | 5 | 3 |
| 2037 | 9313040 | 280 | 1358 | 38 | 3 | 2 |
| 2038 | 9313041 | 239 | 1269 | 19 | 4 | 4 |
| 2039 | 9313042 | 219 | 981 | 24 | 1 | 2 |
| 2040 | 9313043 | 206 | 859 | 24 | 2 | 3 |
| 2041 | 9313044 | 177 | 782 | 19 | 1 | 5 |
| 2042 | 9313045 | 271 | 1154 | 25 | 3 | 5 |
| 2043 | 9313046 | 282 | 1085 | 30 | 4 | 3 |
| 2044 | 9313047 | 153 | 683 | 14 | 2 | 3 |
| 2045 | 9313048 | 227 | 880 | 14 | 4 | 4 |
| 2046 | 9313049 | 229 | 705 | 16 | 8 | 5 |
| 2047 | 9313050 | 262 | 1087 | 23 | 4 | 3 |
| 2048 | 9313051 | 286 | 1154 | 24 | 5 | 6 |
| 2049 | 9313052 | 247 | 1170 | 23 | 2 | 4 |
| 2050 | 9313053 | 296 | 1629 | 19 | 6 | 5 |
| 2051 | 9313054 | 202 | 848 | 24 | 2 | 1 |
| 2052 | 9313055 | 290 | 1235 | 12 | 8 | 5 |
| 2053 | 9313056 | 208 | 795 | 23 | 4 | 3 |
| 2054 | 9313057 | 295 | 1238 | 28 | 3 | 5 |
| 2055 | 9313058 | 171 | 827 | 19 | 0 | 2 |
| 2056 | 9313059 | 207 | 833 | 20 | 0 | 3 |
| 2057 | 9313060 | 187 | 865 | 15 | 3 | 3 |
| 2058 | 9313061 | 214 | 950 | 18 | 3 | 4 |
| 2059 | 9313062 | 238 | 1047 | 26 | 2 | 3 |
| 2060 | 9313063 | 234 | 621 | 20 | 0 | 6 |
| 2061 | 9313064 | 172 | 627 | 13 | 2 | 3 |
| 2062 | 9313065 | 269 | 1068 | 17 | 2 | 5 |
| 2063 | 9313066 | 210 | 796 | 21 | 1 | 3 |
| 2064 | 9313067 | 259 | 1128 | 24 | 6 | 1 |
| 2065 | 9313068 | 246 | 786 | 27 | 4 | 5 |
| 2066 | 9313069 | 197 | 924 | 22 | 0 | 4 |
| 2067 | 9313070 | 253 | 793 | 27 | 4 | 4 |
| 2068 | 9313071 | 219 | 1005 | 31 | 2 | 5 |
| 2069 | 9313072 | 243 | 1200 | 19 | 3 | 2 |
| 2070 | 9313073 | 273 | 1002 | 26 | 2 | 4 |
| 2071 | 9313074 | 240 | 1075 | 30 | 6 | 3 |
| 2072 | 9313075 | 277 | 1105 | 34 | 1 | 5 |
| 2073 | 9313076 | 200 | 797 | 18 | 4 | 3 |
| 2074 | 9313077 | 194 | 889 | 20 | 3 | 2 |
| 2075 | 9313078 | 241 | 1259 | 28 | 3 | 6 |
| 2076 | 9313079 | 221 | 1055 | 24 | 2 | 4 |
| 2077 | 9313080 | 307 | 1228 | 27 | 7 | 5 |
| 2078 | 9313081 | 307 | 1486 | 49 | 0 | 3 |
| 2079 | 9313082 | 202 | 901 | 24 | 2 | 3 |
| 2080 | 9313083 | 255 | 1374 | 22 | 11 | 2 |
| 2081 | 9313084 | 391 | 1936 | 49 | 1 | 4 |
| 2082 | 9313085 | 202 | 839 | 17 | 5 | 3 |
| 2083 | 9313086 | 221 | 1000 | 24 | 7 | 4 |
| 2084 | 9313087 | 212 | 691 | 15 | 3 | 3 |
| 2085 | 9313088 | 233 | 1069 | 30 | 4 | 4 |
| 2086 | 9313089 | 207 | 762 | 27 | 7 | 5 |
| 2087 | 9313090 | 260 | 1514 | 23 | 5 | 2 |
| 2088 | 9313091 | 252 | 1105 | 28 | 3 | 6 |
| 2089 | 9313092 | 247 | 1246 | 22 | 6 | 5 |
| 2090 | 9313093 | 338 | 1350 | 33 | 1 | 5 |
| 2091 | 9313094 | 200 | 1072 | 22 | 6 | 2 |
| 2092 | 9313095 | 220 | 1154 | 28 | 0 | 2 |
| 2093 | 9313096 | 234 | 1211 | 20 | 4 | 3 |
| 2094 | 9313097 | 241 | 1118 | 27 | 1 | 2 |
| 2095 | 9313098 | 342 | 1438 | 29 | 6 | 5 |
| 2096 | 9313099 | 282 | 1393 | 39 | 6 | 3 |
| 2097 | 9313100 | 214 | 891 | 25 | 2 | 3 |
| 2098 | 9313101 | 283 | 1260 | 35 | 2 | 4 |
| 2099 | 9313102 | 239 | 1113 | 29 | 4 | 2 |
| 2100 | 9313103 | 205 | 939 | 18 | 7 | 4 |
| 2101 | 9313104 | 188 | 573 | 19 | 1 | 3 |
| 2102 | 9313105 | 205 | 967 | 18 | 2 | 2 |
| 2103 | 9313106 | 241 | 882 | 22 | 1 | 3 |
| 2104 | 9313107 | 264 | 1126 | 22 | 3 | 3 |
| 2105 | 9313108 | 214 | 812 | 13 | 2 | 2 |
| 2106 | 9313109 | 203 | 805 | 20 | 8 | 4 |
| 2107 | 9313110 | 312 | 1447 | 35 | 4 | 4 |
| 2108 | 9313111 | 369 | 1310 | 35 | 2 | 5 |
| 2109 | 9313112 | 388 | 1943 | 32 | 2 | 4 |
| 2110 | 9313113 | 210 | 928 | 24 | 4 | 3 |
| 2111 | 9313114 | 224 | 970 | 22 | 1 | 4 |

## Throughput

- Turns simulated: 532121 (first play of each game)
- Aggregate: 314.5 turns/s over 1692.2s wall
- Per game thread: 39.5 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

# Simulation: census-eleven-factions-rules64 (2026-11-27)

2112 games, first seed 9311003, brain `scripted`, turn limit off, safety cap 1200, 8 threads, started 2026-11-27T00:13:06Z. Wall time 1885.7s.

Source checkout at run start: `bcb5825369c29c4f223ce6de4f1e921daa7cfcb6`; dirty: no. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. Games seat the eleven cyclic rosters of eight over all eleven factions (`--all-factions`), one roster per eight games, turned one seat per game, so every faction plays every seat equally often, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--players","8","--difficulty","hard","--all-factions","--games","2112","--seed","9311003","--threads","8","--label","census-eleven-factions-rules64"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 2112 of 2112 |
| Game length, turns (mean / median / min-max) | 250.3 / 243 / 137-500 |
| Win reason mix | VP hold 95.1%, Capitals 4.5%, Turn limit 0.4% |
| Median end turn | 243 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 4.5% / 95.1% / 0.4% / 0.0% |
| Games reaching the World War era (any empire) | 2103 of 2112 (99.6%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 2859 / 723 / 2013 / 123 |
| Hold break rate | 25.3% of started; 26.4% of resolved (broken / (broken + completed)) |
| Hold wins | 2009 |
| First reach to win, turns (mean / median / min-max) | 13.3 / 9 / 9-218 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 283 |
| Wins by a team another team beat to its target | 131 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 2009 | 247.1 / 240 / 137-496 |
| Capitals | 94 | 293.9 / 285 / 186-442 |
| Turn limit | 9 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 144 | 6.8% |
| India | 29 | 1.4% |
| Byzantium | 193 | 9.1% |
| Germany | 208 | 9.8% |
| Celts | 257 | 12.2% |
| Mongols | 99 | 4.7% |
| Sparta | 189 | 8.9% |
| Athens | 194 | 9.2% |
| Aztecs | 159 | 7.5% |
| Egypt | 568 | 26.9% |
| Rome | 72 | 3.4% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 2009 | 95.1% |
| Conquest | 0 | 0.0% |
| Capitals | 94 | 4.5% |
| Turn limit | 9 | 0.4% |
| Draw | 0 | 0.0% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 1536 | 144 | 9.4% |
| India | 1536 | 29 | 1.9% |
| Byzantium | 1536 | 193 | 12.6% |
| Germany | 1536 | 208 | 13.5% |
| Celts | 1536 | 257 | 16.7% |
| Mongols | 1536 | 99 | 6.4% |
| Sparta | 1536 | 189 | 12.3% |
| Athens | 1536 | 194 | 12.6% |
| Aztecs | 1536 | 159 | 10.4% |
| Egypt | 1536 | 568 | 37.0% |
| Rome | 1536 | 72 | 4.7% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 16896 | 2112 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 1536 | 144 | 9.4% |
| India | Hard | 1536 | 29 | 1.9% |
| Byzantium | Hard | 1536 | 193 | 12.6% |
| Germany | Hard | 1536 | 208 | 13.5% |
| Celts | Hard | 1536 | 257 | 16.7% |
| Mongols | Hard | 1536 | 99 | 6.4% |
| Sparta | Hard | 1536 | 189 | 12.3% |
| Athens | Hard | 1536 | 194 | 12.6% |
| Aztecs | Hard | 1536 | 159 | 10.4% |
| Egypt | Hard | 1536 | 568 | 37.0% |
| Rome | Hard | 1536 | 72 | 4.7% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 1536 | 167 | 10.9% | 117 | 155 | 192 | 270 | 443 |
| India | 1536 | 717 | 46.7% | 43 | 91 | 131 | 214 | 389 |
| Byzantium | 1536 | 1047 | 68.2% | 70 | 84 | 110 | 165 | 365 |
| Germany | 1536 | 437 | 28.5% | 95 | 134 | 172 | 236 | 398 |
| Celts | 1536 | 1060 | 69.0% | 30 | 189 | 210 | 210 | 251 |
| Mongols | 1536 | 179 | 11.7% | 120 | 167 | 209 | 279 | 420 |
| Sparta | 1536 | 258 | 16.8% | 138 | 172 | 219 | 287 | 487 |
| Athens | 1536 | 578 | 37.6% | 129 | 183 | 255 | 255 | 255 |
| Aztecs | 1536 | 494 | 32.2% | 131 | 190 | 225 | 244 | 308 |
| Egypt | 1536 | 604 | 39.3% | 109 | 169 | 210 | 267 | 497 |
| Rome | 1536 | 55 | 3.6% | 137 | 167 | 208 | 294 | 426 |

## Goal cities (Rome's Colonia, Egypt's tomb)

Per seat of the faction: its goal city stood (median first turn), the seat scored it, held it at the end, a rival held it at the end, it changed hands. Rome: seats with a level-3 melee unit while the Colonia was open (median first turn) and the share of those seat-turns with no legal Colonia site on the map. Egypt: seats that started the work (median first start), work turns and pauses per starting seat.

| Faction | Seats | Stood | Turn | Scored | Held at end | Rival holds | Changed hands | Level 3 | Turn | No site | Started | Turn | Work turns | Pauses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Egypt | 1536 | 417 | 218 | 417 | 307 | 82 | 93 | 0 | - | - | 734 | 199 | 11.0 | 0.7 |
| Rome | 1536 | 0 | - | 0 | 0 | 0 | 0 | 0 | - | - | 0 | - | - | - |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 137 | 192 | 243 | 313 | 500 |
| Winning team VP | 3 | 10 | 11 | 13 | 18 |
| First Soldier completed (turn, per empire) | 6 | 8 | 14 | 56 | 303 |
| First Settler completed (turn, per empire) | 2 | 9 | 16 | 28 | 239 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 5 | 15 | 23 | 51 | 300 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 16896 observed empires in 2112 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 16305 | 591 |
| First Settler completed | 14693 | 2203 |
| First city founded | 16896 | 0 |
| Second city founded | 15770 | 1126 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 4527 |
| 1 | 2672 |
| 2 | 2053 |
| 3 | 1461 |
| 4 | 1150 |
| 5 | 956 |
| 6 | 687 |
| 7 | 531 |
| 8 | 409 |
| 9 | 279 |
| 10 | 1031 |
| 11 | 594 |
| 12 | 304 |
| 13 | 142 |
| 14 | 56 |
| 15 | 23 |
| 16 | 15 |
| 17 | 4 |
| 18 | 2 |

## End state

- Average game length: 250.3 turns
- Average cities per empire: 4.26
- Average techs per empire: 30.18
- Average eliminations per game: 2.88

## Strategy diversity: observed behavior (classifier v2)

2112 finished games; 16896 player seats; 2112 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4156** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 7930 | 442 | 5.6% | 46.9% | 20.9% |
| Wide | 5001 | 1535 | 30.7% | 29.6% | 72.7% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 21 | 1 | 4.8% | 0.1% | 0.0% |
| Economic | 1175 | 131 | 11.1% | 7.0% | 6.2% |
| Generalist | 2769 | 3 | 0.1% | 16.4% | 0.1% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 17.4% / 3.2% (2029) | n/a / n/a (0) | 32.7% / 7.5% (20) | 22.2% / 6.2% (910) | 81.4% / 2.4% (1747) |
| Wide | 79.5% / 3.2% (2029) | — | n/a / n/a (0) | 62.7% / 4.0% (21) | 60.3% / 5.4% (868) | 96.0% / 0.9% (1673) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 59.8% / 7.5% (20) | 33.3% / 4.0% (21) | n/a / n/a (0) | — | 40.9% / 18.2% (11) | 96.9% / 0.0% (16) |
| Economic | 71.6% / 6.2% (910) | 34.4% / 5.4% (868) | n/a / n/a (0) | 40.9% / 18.2% (11) | — | 95.1% / 2.6% (706) |
| Generalist | 16.3% / 2.4% (1747) | 3.1% / 0.9% (1673) | n/a / n/a (0) | 3.1% / 0.0% (16) | 2.2% / 2.6% (706) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 8 / 460 | 120 / 562 | 0 / 0 | 0 / 1 | 16 / 197 | 0 / 316 |
| India | 5 / 785 | 24 / 414 | 0 / 0 | 0 / 0 | 0 / 21 | 0 / 316 |
| Byzantium | 2 / 24 | 167 / 712 | 0 / 0 | 0 / 9 | 24 / 244 | 0 / 547 |
| Germany | 31 / 663 | 163 / 506 | 0 / 0 | 0 / 3 | 13 / 80 | 1 / 284 |
| Celts | 31 / 521 | 213 / 652 | 0 / 0 | 0 / 0 | 13 / 129 | 0 / 234 |
| Mongols | 57 / 1179 | 37 / 104 | 0 / 0 | 0 / 0 | 5 / 27 | 0 / 226 |
| Sparta | 126 / 1278 | 51 / 192 | 0 / 0 | 1 / 4 | 11 / 44 | 0 / 18 |
| Athens | 15 / 517 | 165 / 491 | 0 / 0 | 0 / 0 | 13 / 146 | 1 / 382 |
| Aztecs | 34 / 675 | 108 / 423 | 0 / 0 | 0 / 0 | 16 / 146 | 1 / 292 |
| Egypt | 84 / 481 | 472 / 901 | 0 / 0 | 0 / 4 | 12 / 93 | 0 / 57 |
| Rome | 49 / 1347 | 15 / 44 | 0 / 0 | 0 / 0 | 8 / 48 | 0 / 97 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: none. Unobserved: Science. Under-sampled pairs: 7.

- insufficient evidence: 7 under-sampled matchups; 2112 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

2112 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 32349/2112 | 15.317 | 2112/2112 (100.0%) |
| Peace offers | 214064/2112 | 101.356 | 2112/2112 (100.0%) |
| Peace agreements | 24958/2112 | 11.817 | 2112/2112 (100.0%) |
| Battles | 2734869/2112 | 1294.919 | 2112/2112 (100.0%) |
| City captures | 52122/2112 | 24.679 | 2112/2112 (100.0%) |
| Settler captures | 7509/2112 | 3.555 | 1984/2112 (93.9%) |
| Eliminations | 6084/2112 | 2.881 | 2066/2112 (97.8%) |

## Terminal VP sources

2112 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 13996/16896 | 13996/16896 | 6932/2112 |
| Faction goals | 9080/16896 | 9080/16896 | 3674/2112 |
| Era objectives | 29585/16896 | 29585/16896 | 9117/2112 |
| Capitals taken | 5313/16896 | 5313/16896 | 3298/2112 |
| Total VP | 57974/16896 | 57974/16896 | 23021/2112 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 187/1432 (13.1%) | 0 |
| Tied leaders (winner in set) | 470/680 (69.1%) | 0 |
| Any leader set | 657/2112 (31.1%) | 0 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 15873/16896 | 5 | 20 | 28 | 33 | 83 | -2.4 (2112) |
| Medieval | 15232/16896 | 35 | 58 | 62 | 67 | 168 | -2.5 (2112) |
| Renaissance | 15034/16896 | 60 | 84 | 90 | 102 | 256 | -5.0 (2112) |
| Industrial | 14339/16896 | 84 | 111 | 121 | 141 | 256 | -8.2 (2112) |
| Machine Age | 12736/16896 | 106 | 140 | 154 | 178 | 345 | -11.1 (2112) |
| World War | 10166/16896 | 120 | 164 | 182 | 211 | 355 | -13.1 (2095) |
| Modern | 6939/16896 | 142 | 187 | 211 | 246 | 405 | -13.1 (1891) |
| Future | 3868/16896 | 161 | 209 | 242 | 270 | 442 | -10.7 (1376) |

Research idle turns (techs left, no research set): 18, of which stalled (two or more in a row): 0.

## Tech pace against the calendar

Median resolved turn each technology was learned, the display year of that turn (`calendar::calendar_label`) and the year the technology really arrived (n/a when no single year is established). Empires that never learned it are left out.

| Era | Technology | Learned / seats | Median turn | Median year | Real year |
| --- | --- | --- | --- | --- | --- |
| Ancient | Irrigation | 16818/16896 | 9 | 3173 BC | 6000 BC |
| Ancient | Bronze Working | 14990/16896 | 18 | 2242 BC | 3300 BC |
| Ancient | Masonry | 16893/16896 | 12 | 2863 BC | 3000 BC |
| Ancient | Currency | 16041/16896 | 27 | 1311 BC | 600 BC |
| Ancient | Writing | 16141/16896 | 24 | 1621 BC | 3200 BC |
| Classical | Mathematics | 15381/16896 | 44 | 301 BC | 300 BC |
| Classical | Engineering | 15290/16896 | 51 | 50 AD | n/a |
| Classical | Iron Working | 15501/16896 | 38 | 601 BC | 1200 BC |
| Classical | Code of Laws | 15254/16896 | 59 | 450 AD | 1754 BC |
| Classical | Philosophy | 15236/16896 | 61 | 530 AD | 500 BC |
| Medieval | Feudalism | 15157/16896 | 75 | 950 AD | n/a |
| Medieval | Machinery | 15190/16896 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 13754/16896 | 81 | 1130 AD | n/a |
| Medieval | Education | 15071/16896 | 87 | 1310 AD | 1088 AD |
| Medieval | Crop Rotation | 15043/16896 | 90 | 1400 AD | n/a |
| Renaissance | Gunpowder | 14951/16896 | 97 | 1470 AD | n/a |
| Renaissance | Metallurgy | 14750/16896 | 109 | 1590 AD | n/a |
| Renaissance | Military Engineering | 14820/16896 | 106 | 1560 AD | n/a |
| Renaissance | Banking | 14491/16896 | 118 | 1680 AD | 1397 AD |
| Renaissance | Printing Press | 14387/16896 | 120 | 1700 AD | 1440 AD |
| Industrial | Rifling | 13298/16896 | 143 | 1816 AD | n/a |
| Industrial | Military Science | 13795/16896 | 134 | 1783 AD | n/a |
| Industrial | Ballistics | 13472/16896 | 146 | 1828 AD | n/a |
| Industrial | Industrialization | 14034/16896 | 131 | 1772 AD | 1769 AD |
| Industrial | Railways | 12741/16896 | 154 | 1857 AD | 1825 AD |
| Machine Age | Replaceable Parts | 11539/16896 | 168 | 1889 AD | 1800 AD |
| Machine Age | Combustion | 12158/16896 | 162 | 1882 AD | 1876 AD |
| Machine Age | Steel | 11546/16896 | 167 | 1887 AD | 1856 AD |
| Machine Age | Flight | 10670/16896 | 177 | 1899 AD | 1903 AD |
| Machine Age | Electricity | 10244/16896 | 182 | 1904 AD | 1879 AD |
| World War | Mass Mobilisation | 8557/16896 | 197 | 1924 AD | n/a |
| World War | Rocketry | 8117/16896 | 201 | 1929 AD | 1926 AD |
| World War | Armoured Warfare | 7633/16896 | 206 | 1937 AD | 1916 AD |
| World War | Air Power | 7237/16896 | 210 | 1942 AD | n/a |
| World War | Electronics | 9466/16896 | 189 | 1912 AD | n/a |
| Modern | Mechanised Warfare | 5268/16896 | 225 | 1974 AD | n/a |
| Modern | Composite Armour | 4772/16896 | 230 | 1986 AD | n/a |
| Modern | Guided Missiles | 4530/16896 | 236 | 2000 AD | n/a |
| Modern | Jet Engines | 4309/16896 | 240 | 2010 AD | 1939 AD |
| Modern | Green Revolution | 6019/16896 | 219 | 1959 AD | n/a |
| Future | Autonomous Systems | 2735/16896 | 256 | 2032 AD | n/a |
| Future | Robotics | 2330/16896 | 258 | 2034 AD | n/a |
| Future | Fusion Power | 2195/16896 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 3146/16896 | 250 | 2026 AD | n/a |
| Future | Recursive AI | 1961/16896 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 62.73 |
| 1 | 79.25 | 12.14 |
| 2 | 25.73 | 4.77 |
| 3 | 6.75 | 1.07 |
| 4 | 1.29 | 0.24 |
| 5 | 0.27 | 0.06 |

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
| 16 | 9311019 | 258 | 1366 | 27 | 2 | 5 |
| 17 | 9311020 | 177 | 980 | 17 | 5 | 3 |
| 18 | 9311021 | 184 | 874 | 20 | 6 | 2 |
| 19 | 9311022 | 228 | 1256 | 27 | 4 | 3 |
| 20 | 9311023 | 315 | 1660 | 36 | 5 | 5 |
| 21 | 9311024 | 232 | 919 | 20 | 6 | 4 |
| 22 | 9311025 | 221 | 856 | 13 | 4 | 3 |
| 23 | 9311026 | 279 | 1432 | 17 | 6 | 3 |
| 24 | 9311027 | 280 | 1499 | 24 | 4 | 2 |
| 25 | 9311028 | 202 | 728 | 20 | 3 | 3 |
| 26 | 9311029 | 219 | 1126 | 16 | 4 | 1 |
| 27 | 9311030 | 193 | 891 | 11 | 3 | 2 |
| 28 | 9311031 | 303 | 1434 | 28 | 3 | 5 |
| 29 | 9311032 | 301 | 1706 | 27 | 2 | 4 |
| 30 | 9311033 | 238 | 930 | 22 | 0 | 4 |
| 31 | 9311034 | 285 | 1624 | 31 | 5 | 3 |
| 32 | 9311035 | 267 | 1990 | 34 | 1 | 3 |
| 33 | 9311036 | 187 | 1206 | 21 | 2 | 2 |
| 34 | 9311037 | 314 | 1921 | 28 | 1 | 4 |
| 35 | 9311038 | 238 | 1450 | 14 | 4 | 2 |
| 36 | 9311039 | 259 | 1682 | 30 | 2 | 3 |
| 37 | 9311040 | 200 | 1005 | 20 | 1 | 4 |
| 38 | 9311041 | 238 | 1214 | 28 | 2 | 2 |
| 39 | 9311042 | 272 | 1642 | 21 | 2 | 5 |
| 40 | 9311043 | 295 | 1717 | 25 | 3 | 3 |
| 41 | 9311044 | 401 | 1740 | 41 | 3 | 5 |
| 42 | 9311045 | 200 | 917 | 23 | 11 | 3 |
| 43 | 9311046 | 177 | 817 | 20 | 6 | 2 |
| 44 | 9311047 | 221 | 936 | 19 | 4 | 2 |
| 45 | 9311048 | 180 | 833 | 13 | 1 | 2 |
| 46 | 9311049 | 400 | 1680 | 50 | 3 | 6 |
| 47 | 9311050 | 202 | 1287 | 28 | 3 | 1 |
| 48 | 9311051 | 277 | 1509 | 36 | 2 | 3 |
| 49 | 9311052 | 235 | 1039 | 27 | 2 | 2 |
| 50 | 9311053 | 223 | 1149 | 23 | 3 | 2 |
| 51 | 9311054 | 248 | 1324 | 26 | 6 | 3 |
| 52 | 9311055 | 184 | 865 | 22 | 6 | 3 |
| 53 | 9311056 | 260 | 1300 | 29 | 4 | 2 |
| 54 | 9311057 | 161 | 791 | 14 | 0 | 2 |
| 55 | 9311058 | 222 | 1273 | 20 | 1 | 2 |
| 56 | 9311059 | 208 | 875 | 23 | 2 | 3 |
| 57 | 9311060 | 195 | 1076 | 27 | 5 | 1 |
| 58 | 9311061 | 172 | 835 | 19 | 5 | 2 |
| 59 | 9311062 | 178 | 742 | 17 | 2 | 1 |
| 60 | 9311063 | 163 | 772 | 15 | 6 | 1 |
| 61 | 9311064 | 168 | 740 | 13 | 5 | 1 |
| 62 | 9311065 | 230 | 1421 | 22 | 2 | 2 |
| 63 | 9311066 | 287 | 1445 | 33 | 10 | 4 |
| 64 | 9311067 | 207 | 1150 | 23 | 5 | 2 |
| 65 | 9311068 | 306 | 1759 | 35 | 2 | 1 |
| 66 | 9311069 | 261 | 1013 | 19 | 3 | 3 |
| 67 | 9311070 | 231 | 1535 | 25 | 5 | 4 |
| 68 | 9311071 | 216 | 1137 | 20 | 6 | 2 |
| 69 | 9311072 | 311 | 2008 | 26 | 5 | 3 |
| 70 | 9311073 | 206 | 1076 | 17 | 0 | 2 |
| 71 | 9311074 | 256 | 1520 | 24 | 4 | 2 |
| 72 | 9311075 | 296 | 1710 | 30 | 4 | 4 |
| 73 | 9311076 | 241 | 1423 | 22 | 7 | 1 |
| 74 | 9311077 | 297 | 1807 | 46 | 0 | 3 |
| 75 | 9311078 | 195 | 1019 | 19 | 3 | 2 |
| 76 | 9311079 | 250 | 1054 | 34 | 0 | 2 |
| 77 | 9311080 | 274 | 1825 | 33 | 2 | 2 |
| 78 | 9311081 | 251 | 1375 | 41 | 4 | 2 |
| 79 | 9311082 | 208 | 1355 | 24 | 1 | 3 |
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
| 104 | 9311107 | 248 | 1032 | 25 | 8 | 4 |
| 105 | 9311108 | 213 | 1114 | 17 | 2 | 2 |
| 106 | 9311109 | 189 | 1137 | 18 | 2 | 1 |
| 107 | 9311110 | 207 | 1021 | 16 | 7 | 1 |
| 108 | 9311111 | 249 | 1061 | 30 | 6 | 2 |
| 109 | 9311112 | 291 | 1536 | 33 | 3 | 2 |
| 110 | 9311113 | 236 | 1412 | 21 | 5 | 3 |
| 111 | 9311114 | 251 | 980 | 20 | 6 | 4 |
| 112 | 9311115 | 329 | 2061 | 35 | 3 | 1 |
| 113 | 9311116 | 231 | 1234 | 26 | 3 | 4 |
| 114 | 9311117 | 209 | 914 | 4 | 4 | 4 |
| 115 | 9311118 | 279 | 1532 | 24 | 4 | 5 |
| 116 | 9311119 | 189 | 1160 | 17 | 4 | 1 |
| 117 | 9311120 | 273 | 1429 | 23 | 1 | 4 |
| 118 | 9311121 | 262 | 1230 | 17 | 4 | 3 |
| 119 | 9311122 | 266 | 1395 | 30 | 2 | 3 |
| 120 | 9311123 | 190 | 870 | 12 | 2 | 4 |
| 121 | 9311124 | 343 | 1641 | 28 | 5 | 3 |
| 122 | 9311125 | 319 | 1174 | 39 | 3 | 5 |
| 123 | 9311126 | 379 | 2272 | 34 | 6 | 4 |
| 124 | 9311127 | 258 | 1376 | 20 | 1 | 2 |
| 125 | 9311128 | 203 | 1230 | 22 | 0 | 3 |
| 126 | 9311129 | 370 | 2067 | 40 | 3 | 5 |
| 127 | 9311130 | 226 | 1223 | 27 | 4 | 3 |
| 128 | 9311131 | 244 | 1389 | 22 | 4 | 4 |
| 129 | 9311132 | 255 | 1276 | 27 | 6 | 3 |
| 130 | 9311133 | 197 | 688 | 13 | 2 | 2 |
| 131 | 9311134 | 293 | 1555 | 25 | 2 | 3 |
| 132 | 9311135 | 196 | 1142 | 24 | 3 | 4 |
| 133 | 9311136 | 199 | 1156 | 24 | 4 | 4 |
| 134 | 9311137 | 197 | 1306 | 26 | 3 | 1 |
| 135 | 9311138 | 269 | 1600 | 22 | 1 | 1 |
| 136 | 9311139 | 268 | 1432 | 38 | 3 | 2 |
| 137 | 9311140 | 200 | 797 | 15 | 3 | 3 |
| 138 | 9311141 | 259 | 1303 | 22 | 1 | 2 |
| 139 | 9311142 | 200 | 1026 | 24 | 3 | 2 |
| 140 | 9311143 | 235 | 1164 | 22 | 1 | 2 |
| 141 | 9311144 | 299 | 1307 | 28 | 4 | 2 |
| 142 | 9311145 | 216 | 1107 | 21 | 4 | 3 |
| 143 | 9311146 | 248 | 1240 | 32 | 6 | 2 |
| 144 | 9311147 | 202 | 990 | 15 | 4 | 2 |
| 145 | 9311148 | 223 | 1187 | 24 | 6 | 4 |
| 146 | 9311149 | 324 | 2012 | 31 | 5 | 2 |
| 147 | 9311150 | 268 | 1295 | 14 | 2 | 1 |
| 148 | 9311151 | 201 | 1066 | 13 | 9 | 1 |
| 149 | 9311152 | 162 | 858 | 12 | 2 | 1 |
| 150 | 9311153 | 304 | 1671 | 41 | 7 | 5 |
| 151 | 9311154 | 173 | 873 | 21 | 4 | 2 |
| 152 | 9311155 | 274 | 1862 | 32 | 3 | 2 |
| 153 | 9311156 | 192 | 1125 | 27 | 2 | 3 |
| 154 | 9311157 | 231 | 1476 | 18 | 5 | 1 |
| 155 | 9311158 | 199 | 1026 | 10 | 8 | 2 |
| 156 | 9311159 | 204 | 930 | 13 | 4 | 2 |
| 157 | 9311160 | 204 | 883 | 17 | 3 | 3 |
| 158 | 9311161 | 220 | 1392 | 17 | 4 | 3 |
| 159 | 9311162 | 212 | 1094 | 17 | 1 | 2 |
| 160 | 9311163 | 163 | 829 | 15 | 5 | 1 |
| 161 | 9311164 | 457 | 2481 | 64 | 6 | 5 |
| 162 | 9311165 | 307 | 1728 | 44 | 9 | 4 |
| 163 | 9311166 | 191 | 1129 | 17 | 3 | 3 |
| 164 | 9311167 | 282 | 1622 | 22 | 5 | 3 |
| 165 | 9311168 | 245 | 1542 | 22 | 3 | 4 |
| 166 | 9311169 | 301 | 1868 | 42 | 13 | 2 |
| 167 | 9311170 | 227 | 986 | 19 | 10 | 3 |
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
| 192 | 9311195 | 261 | 1480 | 19 | 4 | 2 |
| 193 | 9311196 | 244 | 1599 | 10 | 9 | 2 |
| 194 | 9311197 | 265 | 1477 | 29 | 3 | 3 |
| 195 | 9311198 | 210 | 1029 | 23 | 2 | 3 |
| 196 | 9311199 | 209 | 1337 | 23 | 8 | 4 |
| 197 | 9311200 | 231 | 1237 | 18 | 8 | 1 |
| 198 | 9311201 | 352 | 2078 | 34 | 6 | 3 |
| 199 | 9311202 | 244 | 1000 | 31 | 2 | 3 |
| 200 | 9311203 | 221 | 1254 | 27 | 5 | 4 |
| 201 | 9311204 | 226 | 1274 | 16 | 0 | 3 |
| 202 | 9311205 | 291 | 1802 | 29 | 5 | 5 |
| 203 | 9311206 | 325 | 2009 | 30 | 5 | 2 |
| 204 | 9311207 | 317 | 1708 | 27 | 6 | 5 |
| 205 | 9311208 | 351 | 1960 | 36 | 4 | 5 |
| 206 | 9311209 | 239 | 1060 | 18 | 1 | 5 |
| 207 | 9311210 | 219 | 1110 | 14 | 2 | 3 |
| 208 | 9311211 | 255 | 1365 | 27 | 1 | 2 |
| 209 | 9311212 | 264 | 1522 | 32 | 7 | 2 |
| 210 | 9311213 | 272 | 1563 | 28 | 1 | 2 |
| 211 | 9311214 | 365 | 2079 | 41 | 1 | 4 |
| 212 | 9311215 | 205 | 949 | 20 | 0 | 3 |
| 213 | 9311216 | 257 | 1592 | 29 | 4 | 3 |
| 214 | 9311217 | 260 | 1595 | 23 | 4 | 4 |
| 215 | 9311218 | 281 | 1749 | 29 | 2 | 3 |
| 216 | 9311219 | 229 | 1309 | 35 | 5 | 2 |
| 217 | 9311220 | 265 | 1484 | 31 | 2 | 4 |
| 218 | 9311221 | 228 | 1049 | 24 | 1 | 4 |
| 219 | 9311222 | 257 | 1724 | 28 | 2 | 2 |
| 220 | 9311223 | 299 | 1671 | 27 | 6 | 4 |
| 221 | 9311224 | 226 | 888 | 21 | 1 | 1 |
| 222 | 9311225 | 195 | 1038 | 19 | 2 | 4 |
| 223 | 9311226 | 234 | 1134 | 24 | 0 | 3 |
| 224 | 9311227 | 323 | 1603 | 31 | 5 | 3 |
| 225 | 9311228 | 403 | 1882 | 53 | 3 | 5 |
| 226 | 9311229 | 299 | 1751 | 24 | 1 | 5 |
| 227 | 9311230 | 226 | 1135 | 20 | 0 | 4 |
| 228 | 9311231 | 206 | 1105 | 18 | 5 | 2 |
| 229 | 9311232 | 237 | 1344 | 25 | 3 | 2 |
| 230 | 9311233 | 187 | 967 | 23 | 2 | 2 |
| 231 | 9311234 | 251 | 1300 | 17 | 1 | 3 |
| 232 | 9311235 | 202 | 1041 | 19 | 2 | 2 |
| 233 | 9311236 | 287 | 1565 | 34 | 5 | 3 |
| 234 | 9311237 | 297 | 1616 | 40 | 9 | 4 |
| 235 | 9311238 | 264 | 1303 | 39 | 4 | 4 |
| 236 | 9311239 | 176 | 755 | 15 | 6 | 0 |
| 237 | 9311240 | 198 | 800 | 16 | 5 | 4 |
| 238 | 9311241 | 276 | 1504 | 29 | 3 | 2 |
| 239 | 9311242 | 215 | 1042 | 24 | 4 | 2 |
| 240 | 9311243 | 194 | 1162 | 16 | 0 | 2 |
| 241 | 9311244 | 209 | 1065 | 16 | 3 | 3 |
| 242 | 9311245 | 276 | 1524 | 32 | 7 | 5 |
| 243 | 9311246 | 256 | 1483 | 30 | 5 | 2 |
| 244 | 9311247 | 211 | 1080 | 12 | 4 | 0 |
| 245 | 9311248 | 193 | 1060 | 11 | 3 | 4 |
| 246 | 9311249 | 228 | 1171 | 29 | 6 | 2 |
| 247 | 9311250 | 170 | 842 | 15 | 1 | 3 |
| 248 | 9311251 | 227 | 1197 | 18 | 5 | 1 |
| 249 | 9311252 | 197 | 942 | 21 | 5 | 3 |
| 250 | 9311253 | 333 | 1637 | 41 | 3 | 5 |
| 251 | 9311254 | 294 | 1795 | 22 | 3 | 2 |
| 252 | 9311255 | 248 | 1458 | 29 | 4 | 3 |
| 253 | 9311256 | 217 | 1239 | 31 | 2 | 3 |
| 254 | 9311257 | 200 | 1166 | 20 | 4 | 3 |
| 255 | 9311258 | 195 | 1087 | 22 | 5 | 2 |
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
| 280 | 9311283 | 178 | 927 | 19 | 2 | 3 |
| 281 | 9311284 | 206 | 746 | 19 | 10 | 3 |
| 282 | 9311285 | 204 | 1312 | 28 | 1 | 3 |
| 283 | 9311286 | 271 | 1529 | 29 | 3 | 2 |
| 284 | 9311287 | 262 | 979 | 28 | 6 | 5 |
| 285 | 9311288 | 362 | 2305 | 34 | 6 | 3 |
| 286 | 9311289 | 333 | 2199 | 34 | 18 | 5 |
| 287 | 9311290 | 175 | 855 | 9 | 2 | 2 |
| 288 | 9311291 | 232 | 1402 | 27 | 8 | 3 |
| 289 | 9311292 | 242 | 1121 | 24 | 5 | 3 |
| 290 | 9311293 | 172 | 869 | 12 | 3 | 1 |
| 291 | 9311294 | 224 | 879 | 19 | 0 | 3 |
| 292 | 9311295 | 212 | 948 | 13 | 2 | 3 |
| 293 | 9311296 | 283 | 1280 | 20 | 5 | 3 |
| 294 | 9311297 | 277 | 1755 | 29 | 3 | 2 |
| 295 | 9311298 | 363 | 1977 | 28 | 3 | 3 |
| 296 | 9311299 | 228 | 1274 | 25 | 2 | 3 |
| 297 | 9311300 | 244 | 1021 | 23 | 2 | 3 |
| 298 | 9311301 | 208 | 922 | 8 | 6 | 4 |
| 299 | 9311302 | 259 | 1216 | 26 | 3 | 4 |
| 300 | 9311303 | 265 | 1752 | 18 | 2 | 2 |
| 301 | 9311304 | 281 | 1701 | 29 | 3 | 1 |
| 302 | 9311305 | 279 | 1598 | 13 | 5 | 5 |
| 303 | 9311306 | 248 | 1259 | 13 | 5 | 5 |
| 304 | 9311307 | 192 | 926 | 16 | 3 | 2 |
| 305 | 9311308 | 187 | 848 | 15 | 2 | 1 |
| 306 | 9311309 | 241 | 1078 | 20 | 2 | 4 |
| 307 | 9311310 | 261 | 1366 | 25 | 2 | 3 |
| 308 | 9311311 | 275 | 1893 | 35 | 1 | 5 |
| 309 | 9311312 | 223 | 822 | 18 | 2 | 3 |
| 310 | 9311313 | 201 | 639 | 18 | 2 | 5 |
| 311 | 9311314 | 186 | 939 | 12 | 2 | 4 |
| 312 | 9311315 | 227 | 1217 | 33 | 2 | 3 |
| 313 | 9311316 | 270 | 1555 | 23 | 11 | 2 |
| 314 | 9311317 | 296 | 1498 | 34 | 2 | 3 |
| 315 | 9311318 | 187 | 1208 | 23 | 1 | 3 |
| 316 | 9311319 | 384 | 2262 | 54 | 2 | 3 |
| 317 | 9311320 | 290 | 1344 | 19 | 3 | 1 |
| 318 | 9311321 | 352 | 1861 | 47 | 5 | 4 |
| 319 | 9311322 | 202 | 1089 | 22 | 6 | 2 |
| 320 | 9311323 | 266 | 1455 | 23 | 3 | 2 |
| 321 | 9311324 | 359 | 1771 | 50 | 4 | 3 |
| 322 | 9311325 | 248 | 1237 | 32 | 2 | 4 |
| 323 | 9311326 | 252 | 1125 | 36 | 6 | 2 |
| 324 | 9311327 | 252 | 947 | 26 | 5 | 2 |
| 325 | 9311328 | 262 | 1551 | 22 | 6 | 3 |
| 326 | 9311329 | 362 | 1828 | 57 | 7 | 1 |
| 327 | 9311330 | 203 | 976 | 17 | 3 | 2 |
| 328 | 9311331 | 196 | 962 | 18 | 3 | 1 |
| 329 | 9311332 | 187 | 1032 | 23 | 1 | 4 |
| 330 | 9311333 | 238 | 1347 | 9 | 1 | 3 |
| 331 | 9311334 | 308 | 1895 | 24 | 3 | 2 |
| 332 | 9311335 | 192 | 792 | 13 | 3 | 1 |
| 333 | 9311336 | 277 | 1458 | 18 | 2 | 5 |
| 334 | 9311337 | 256 | 1485 | 28 | 2 | 2 |
| 335 | 9311338 | 231 | 1454 | 27 | 2 | 2 |
| 336 | 9311339 | 196 | 952 | 12 | 1 | 2 |
| 337 | 9311340 | 310 | 1827 | 29 | 4 | 3 |
| 338 | 9311341 | 194 | 1107 | 8 | 6 | 1 |
| 339 | 9311342 | 231 | 1390 | 28 | 3 | 2 |
| 340 | 9311343 | 250 | 1120 | 33 | 6 | 3 |
| 341 | 9311344 | 231 | 1030 | 23 | 2 | 3 |
| 342 | 9311345 | 314 | 1805 | 54 | 5 | 3 |
| 343 | 9311346 | 164 | 615 | 14 | 6 | 1 |
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
| 368 | 9311371 | 188 | 978 | 17 | 5 | 4 |
| 369 | 9311372 | 216 | 1310 | 15 | 5 | 5 |
| 370 | 9311373 | 243 | 1108 | 29 | 11 | 2 |
| 371 | 9311374 | 328 | 1830 | 34 | 7 | 4 |
| 372 | 9311375 | 192 | 766 | 16 | 3 | 2 |
| 373 | 9311376 | 272 | 1223 | 36 | 2 | 4 |
| 374 | 9311377 | 251 | 1290 | 19 | 0 | 5 |
| 375 | 9311378 | 224 | 1160 | 27 | 4 | 3 |
| 376 | 9311379 | 214 | 1034 | 16 | 1 | 3 |
| 377 | 9311380 | 268 | 1885 | 26 | 1 | 3 |
| 378 | 9311381 | 258 | 1356 | 31 | 8 | 3 |
| 379 | 9311382 | 203 | 1008 | 18 | 2 | 3 |
| 380 | 9311383 | 245 | 1386 | 21 | 4 | 5 |
| 381 | 9311384 | 218 | 1255 | 27 | 3 | 1 |
| 382 | 9311385 | 313 | 1638 | 31 | 4 | 4 |
| 383 | 9311386 | 223 | 1236 | 13 | 1 | 5 |
| 384 | 9311387 | 259 | 1082 | 22 | 5 | 3 |
| 385 | 9311388 | 286 | 1850 | 29 | 4 | 5 |
| 386 | 9311389 | 307 | 1955 | 31 | 2 | 3 |
| 387 | 9311390 | 228 | 967 | 29 | 3 | 4 |
| 388 | 9311391 | 265 | 1449 | 15 | 5 | 4 |
| 389 | 9311392 | 196 | 809 | 14 | 3 | 0 |
| 390 | 9311393 | 328 | 1822 | 18 | 1 | 4 |
| 391 | 9311394 | 253 | 1124 | 30 | 3 | 3 |
| 392 | 9311395 | 284 | 1571 | 39 | 6 | 0 |
| 393 | 9311396 | 233 | 1021 | 18 | 1 | 4 |
| 394 | 9311397 | 268 | 1604 | 33 | 1 | 4 |
| 395 | 9311398 | 270 | 1512 | 32 | 1 | 5 |
| 396 | 9311399 | 240 | 1267 | 28 | 6 | 2 |
| 397 | 9311400 | 193 | 852 | 16 | 3 | 2 |
| 398 | 9311401 | 186 | 1005 | 16 | 1 | 2 |
| 399 | 9311402 | 256 | 1304 | 24 | 0 | 4 |
| 400 | 9311403 | 160 | 690 | 15 | 1 | 1 |
| 401 | 9311404 | 202 | 826 | 23 | 1 | 4 |
| 402 | 9311405 | 247 | 1211 | 25 | 3 | 3 |
| 403 | 9311406 | 243 | 1498 | 37 | 1 | 3 |
| 404 | 9311407 | 257 | 1326 | 22 | 3 | 1 |
| 405 | 9311408 | 174 | 778 | 20 | 4 | 3 |
| 406 | 9311409 | 252 | 1512 | 39 | 3 | 3 |
| 407 | 9311410 | 219 | 977 | 21 | 2 | 3 |
| 408 | 9311411 | 269 | 1245 | 33 | 3 | 5 |
| 409 | 9311412 | 283 | 1487 | 28 | 2 | 2 |
| 410 | 9311413 | 211 | 992 | 21 | 5 | 2 |
| 411 | 9311414 | 248 | 1642 | 25 | 3 | 3 |
| 412 | 9311415 | 257 | 1200 | 30 | 1 | 2 |
| 413 | 9311416 | 420 | 2998 | 39 | 11 | 3 |
| 414 | 9311417 | 298 | 1515 | 28 | 4 | 3 |
| 415 | 9311418 | 231 | 981 | 25 | 0 | 0 |
| 416 | 9311419 | 239 | 1456 | 28 | 3 | 3 |
| 417 | 9311420 | 218 | 1245 | 27 | 3 | 3 |
| 418 | 9311421 | 175 | 714 | 15 | 3 | 1 |
| 419 | 9311422 | 211 | 1201 | 20 | 2 | 1 |
| 420 | 9311423 | 213 | 1440 | 27 | 6 | 3 |
| 421 | 9311424 | 211 | 1244 | 21 | 1 | 1 |
| 422 | 9311425 | 242 | 1488 | 26 | 3 | 2 |
| 423 | 9311426 | 286 | 1581 | 27 | 0 | 4 |
| 424 | 9311427 | 214 | 1395 | 18 | 3 | 4 |
| 425 | 9311428 | 238 | 1170 | 23 | 2 | 3 |
| 426 | 9311429 | 244 | 1213 | 20 | 1 | 3 |
| 427 | 9311430 | 423 | 1829 | 45 | 2 | 2 |
| 428 | 9311431 | 208 | 1356 | 20 | 6 | 2 |
| 429 | 9311432 | 205 | 1103 | 15 | 3 | 3 |
| 430 | 9311433 | 233 | 1552 | 33 | 6 | 2 |
| 431 | 9311434 | 259 | 1381 | 34 | 6 | 4 |
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
| 456 | 9311459 | 235 | 939 | 21 | 6 | 2 |
| 457 | 9311460 | 166 | 898 | 10 | 4 | 0 |
| 458 | 9311461 | 210 | 918 | 11 | 2 | 3 |
| 459 | 9311462 | 180 | 905 | 10 | 4 | 3 |
| 460 | 9311463 | 297 | 1680 | 10 | 6 | 1 |
| 461 | 9311464 | 303 | 1602 | 31 | 2 | 4 |
| 462 | 9311465 | 259 | 1303 | 23 | 3 | 3 |
| 463 | 9311466 | 160 | 536 | 9 | 3 | 2 |
| 464 | 9311467 | 267 | 1548 | 30 | 0 | 5 |
| 465 | 9311468 | 193 | 977 | 7 | 4 | 2 |
| 466 | 9311469 | 165 | 867 | 13 | 4 | 3 |
| 467 | 9311470 | 344 | 1950 | 31 | 2 | 5 |
| 468 | 9311471 | 341 | 2050 | 46 | 2 | 4 |
| 469 | 9311472 | 212 | 1168 | 23 | 1 | 3 |
| 470 | 9311473 | 293 | 1484 | 14 | 1 | 4 |
| 471 | 9311474 | 268 | 1508 | 24 | 0 | 4 |
| 472 | 9311475 | 341 | 1527 | 36 | 1 | 2 |
| 473 | 9311476 | 197 | 1172 | 24 | 3 | 2 |
| 474 | 9311477 | 233 | 1164 | 27 | 2 | 4 |
| 475 | 9311478 | 210 | 1273 | 15 | 2 | 2 |
| 476 | 9311479 | 204 | 1304 | 15 | 2 | 1 |
| 477 | 9311480 | 230 | 1661 | 27 | 1 | 1 |
| 478 | 9311481 | 264 | 1251 | 21 | 0 | 4 |
| 479 | 9311482 | 269 | 1348 | 20 | 1 | 3 |
| 480 | 9311483 | 244 | 1364 | 26 | 4 | 2 |
| 481 | 9311484 | 272 | 1085 | 22 | 2 | 4 |
| 482 | 9311485 | 221 | 1295 | 31 | 2 | 3 |
| 483 | 9311486 | 354 | 1758 | 37 | 1 | 4 |
| 484 | 9311487 | 295 | 1705 | 42 | 2 | 2 |
| 485 | 9311488 | 281 | 1709 | 26 | 1 | 3 |
| 486 | 9311489 | 270 | 1488 | 25 | 1 | 4 |
| 487 | 9311490 | 289 | 1526 | 34 | 5 | 3 |
| 488 | 9311491 | 260 | 1098 | 32 | 1 | 4 |
| 489 | 9311492 | 231 | 1125 | 29 | 0 | 4 |
| 490 | 9311493 | 201 | 1000 | 15 | 3 | 2 |
| 491 | 9311494 | 273 | 1475 | 36 | 3 | 4 |
| 492 | 9311495 | 301 | 1767 | 19 | 2 | 2 |
| 493 | 9311496 | 255 | 1241 | 15 | 4 | 3 |
| 494 | 9311497 | 202 | 954 | 24 | 2 | 3 |
| 495 | 9311498 | 238 | 1695 | 26 | 5 | 2 |
| 496 | 9311499 | 201 | 1232 | 14 | 2 | 2 |
| 497 | 9311500 | 500 | 2374 | 65 | 4 | 4 |
| 498 | 9311501 | 265 | 1275 | 22 | 5 | 2 |
| 499 | 9311502 | 266 | 1242 | 24 | 5 | 2 |
| 500 | 9311503 | 272 | 1553 | 30 | 3 | 2 |
| 501 | 9311504 | 246 | 1559 | 27 | 5 | 3 |
| 502 | 9311505 | 243 | 1375 | 39 | 1 | 2 |
| 503 | 9311506 | 154 | 693 | 12 | 0 | 2 |
| 504 | 9311507 | 295 | 1610 | 26 | 4 | 4 |
| 505 | 9311508 | 184 | 883 | 10 | 4 | 0 |
| 506 | 9311509 | 241 | 1382 | 16 | 6 | 2 |
| 507 | 9311510 | 234 | 1359 | 20 | 0 | 2 |
| 508 | 9311511 | 363 | 1823 | 19 | 9 | 3 |
| 509 | 9311512 | 223 | 940 | 29 | 2 | 0 |
| 510 | 9311513 | 180 | 705 | 16 | 3 | 2 |
| 511 | 9311514 | 242 | 1300 | 29 | 5 | 3 |
| 512 | 9311515 | 245 | 1549 | 30 | 4 | 3 |
| 513 | 9311516 | 259 | 1726 | 25 | 7 | 1 |
| 514 | 9311517 | 193 | 975 | 13 | 1 | 1 |
| 515 | 9311518 | 206 | 758 | 22 | 0 | 2 |
| 516 | 9311519 | 342 | 1975 | 35 | 8 | 2 |
| 517 | 9311520 | 352 | 1951 | 31 | 11 | 2 |
| 518 | 9311521 | 282 | 1356 | 28 | 2 | 3 |
| 519 | 9311522 | 192 | 998 | 21 | 4 | 1 |
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
| 544 | 9311547 | 227 | 931 | 27 | 3 | 3 |
| 545 | 9311548 | 267 | 1407 | 18 | 6 | 4 |
| 546 | 9311549 | 188 | 1020 | 10 | 5 | 2 |
| 547 | 9311550 | 206 | 1142 | 14 | 3 | 3 |
| 548 | 9311551 | 185 | 649 | 16 | 4 | 4 |
| 549 | 9311552 | 249 | 1459 | 19 | 2 | 2 |
| 550 | 9311553 | 192 | 1011 | 14 | 6 | 1 |
| 551 | 9311554 | 225 | 1418 | 27 | 4 | 4 |
| 552 | 9311555 | 199 | 909 | 19 | 2 | 2 |
| 553 | 9311556 | 224 | 1427 | 18 | 1 | 3 |
| 554 | 9311557 | 317 | 1809 | 40 | 0 | 4 |
| 555 | 9311558 | 217 | 1223 | 19 | 3 | 4 |
| 556 | 9311559 | 416 | 2045 | 42 | 5 | 6 |
| 557 | 9311560 | 234 | 1597 | 26 | 4 | 1 |
| 558 | 9311561 | 166 | 717 | 11 | 1 | 3 |
| 559 | 9311562 | 259 | 1600 | 33 | 5 | 3 |
| 560 | 9311563 | 274 | 1753 | 27 | 5 | 3 |
| 561 | 9311564 | 248 | 1326 | 18 | 2 | 3 |
| 562 | 9311565 | 297 | 1267 | 27 | 4 | 4 |
| 563 | 9311566 | 386 | 1925 | 32 | 4 | 4 |
| 564 | 9311567 | 238 | 1147 | 21 | 2 | 2 |
| 565 | 9311568 | 281 | 1393 | 26 | 3 | 5 |
| 566 | 9311569 | 252 | 1498 | 28 | 3 | 3 |
| 567 | 9311570 | 238 | 1107 | 17 | 3 | 3 |
| 568 | 9311571 | 226 | 1150 | 22 | 1 | 4 |
| 569 | 9311572 | 232 | 1286 | 18 | 0 | 3 |
| 570 | 9311573 | 200 | 1072 | 16 | 1 | 2 |
| 571 | 9311574 | 244 | 1104 | 23 | 3 | 3 |
| 572 | 9311575 | 283 | 1289 | 38 | 2 | 3 |
| 573 | 9311576 | 306 | 1981 | 34 | 4 | 2 |
| 574 | 9311577 | 217 | 1389 | 24 | 2 | 4 |
| 575 | 9311578 | 193 | 1066 | 22 | 3 | 3 |
| 576 | 9311579 | 234 | 1142 | 20 | 3 | 4 |
| 577 | 9311580 | 269 | 1116 | 23 | 4 | 3 |
| 578 | 9311581 | 228 | 1034 | 20 | 3 | 4 |
| 579 | 9311582 | 233 | 1333 | 35 | 3 | 3 |
| 580 | 9311583 | 296 | 1846 | 36 | 2 | 4 |
| 581 | 9311584 | 266 | 1308 | 37 | 6 | 3 |
| 582 | 9311585 | 207 | 940 | 24 | 3 | 2 |
| 583 | 9311586 | 285 | 1219 | 34 | 3 | 4 |
| 584 | 9311587 | 352 | 1722 | 49 | 5 | 4 |
| 585 | 9311588 | 274 | 1472 | 39 | 4 | 4 |
| 586 | 9311589 | 146 | 746 | 13 | 1 | 1 |
| 587 | 9311590 | 182 | 631 | 15 | 7 | 2 |
| 588 | 9311591 | 244 | 1247 | 27 | 1 | 3 |
| 589 | 9311592 | 181 | 809 | 18 | 5 | 1 |
| 590 | 9311593 | 194 | 994 | 17 | 4 | 1 |
| 591 | 9311594 | 184 | 1055 | 16 | 3 | 1 |
| 592 | 9311595 | 215 | 967 | 21 | 9 | 3 |
| 593 | 9311596 | 221 | 1132 | 20 | 3 | 2 |
| 594 | 9311597 | 260 | 1415 | 21 | 1 | 4 |
| 595 | 9311598 | 223 | 961 | 14 | 3 | 3 |
| 596 | 9311599 | 276 | 1830 | 21 | 9 | 3 |
| 597 | 9311600 | 233 | 1503 | 28 | 4 | 2 |
| 598 | 9311601 | 180 | 838 | 21 | 5 | 1 |
| 599 | 9311602 | 219 | 1006 | 26 | 2 | 2 |
| 600 | 9311603 | 218 | 1095 | 16 | 0 | 2 |
| 601 | 9311604 | 196 | 931 | 17 | 2 | 1 |
| 602 | 9311605 | 237 | 1388 | 23 | 1 | 4 |
| 603 | 9311606 | 206 | 1012 | 18 | 5 | 2 |
| 604 | 9311607 | 257 | 1419 | 23 | 5 | 4 |
| 605 | 9311608 | 198 | 819 | 18 | 3 | 3 |
| 606 | 9311609 | 213 | 1168 | 19 | 2 | 4 |
| 607 | 9311610 | 181 | 1095 | 24 | 5 | 1 |
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
| 632 | 9311635 | 288 | 1898 | 22 | 4 | 2 |
| 633 | 9311636 | 224 | 1258 | 20 | 6 | 2 |
| 634 | 9311637 | 263 | 1242 | 32 | 1 | 5 |
| 635 | 9311638 | 230 | 917 | 24 | 3 | 3 |
| 636 | 9311639 | 303 | 1755 | 33 | 6 | 3 |
| 637 | 9311640 | 283 | 1761 | 22 | 8 | 0 |
| 638 | 9311641 | 243 | 1013 | 23 | 2 | 4 |
| 639 | 9311642 | 231 | 1079 | 30 | 5 | 5 |
| 640 | 9311643 | 333 | 1916 | 27 | 5 | 5 |
| 641 | 9311644 | 300 | 1657 | 36 | 5 | 4 |
| 642 | 9311645 | 221 | 972 | 20 | 3 | 4 |
| 643 | 9311646 | 311 | 1351 | 21 | 5 | 5 |
| 644 | 9311647 | 415 | 2088 | 45 | 2 | 5 |
| 645 | 9311648 | 223 | 1217 | 25 | 2 | 4 |
| 646 | 9311649 | 187 | 893 | 12 | 0 | 2 |
| 647 | 9311650 | 177 | 901 | 22 | 4 | 3 |
| 648 | 9311651 | 285 | 1987 | 27 | 1 | 5 |
| 649 | 9311652 | 243 | 1200 | 25 | 3 | 3 |
| 650 | 9311653 | 253 | 1406 | 22 | 1 | 3 |
| 651 | 9311654 | 210 | 831 | 20 | 2 | 4 |
| 652 | 9311655 | 371 | 2141 | 28 | 4 | 4 |
| 653 | 9311656 | 293 | 1716 | 20 | 4 | 4 |
| 654 | 9311657 | 271 | 1495 | 27 | 2 | 3 |
| 655 | 9311658 | 318 | 2239 | 33 | 12 | 5 |
| 656 | 9311659 | 198 | 835 | 15 | 0 | 2 |
| 657 | 9311660 | 201 | 889 | 14 | 2 | 2 |
| 658 | 9311661 | 194 | 823 | 21 | 0 | 2 |
| 659 | 9311662 | 308 | 1450 | 29 | 2 | 4 |
| 660 | 9311663 | 247 | 1684 | 25 | 1 | 4 |
| 661 | 9311664 | 255 | 1403 | 28 | 0 | 4 |
| 662 | 9311665 | 203 | 1257 | 15 | 2 | 2 |
| 663 | 9311666 | 197 | 777 | 16 | 1 | 2 |
| 664 | 9311667 | 321 | 1636 | 43 | 0 | 3 |
| 665 | 9311668 | 223 | 1253 | 24 | 2 | 3 |
| 666 | 9311669 | 262 | 1461 | 18 | 1 | 2 |
| 667 | 9311670 | 274 | 1590 | 19 | 5 | 2 |
| 668 | 9311671 | 217 | 1211 | 26 | 5 | 4 |
| 669 | 9311672 | 325 | 1393 | 42 | 2 | 6 |
| 670 | 9311673 | 248 | 770 | 19 | 2 | 4 |
| 671 | 9311674 | 248 | 1388 | 33 | 3 | 3 |
| 672 | 9311675 | 190 | 855 | 24 | 1 | 3 |
| 673 | 9311676 | 206 | 1183 | 34 | 3 | 1 |
| 674 | 9311677 | 364 | 1897 | 40 | 3 | 1 |
| 675 | 9311678 | 232 | 1223 | 25 | 2 | 2 |
| 676 | 9311679 | 159 | 788 | 13 | 5 | 2 |
| 677 | 9311680 | 394 | 2162 | 46 | 6 | 2 |
| 678 | 9311681 | 182 | 816 | 23 | 1 | 1 |
| 679 | 9311682 | 213 | 887 | 16 | 2 | 1 |
| 680 | 9311683 | 201 | 1098 | 24 | 1 | 2 |
| 681 | 9311684 | 173 | 672 | 14 | 2 | 2 |
| 682 | 9311685 | 312 | 1661 | 24 | 4 | 2 |
| 683 | 9311686 | 255 | 1178 | 33 | 4 | 3 |
| 684 | 9311687 | 186 | 955 | 13 | 3 | 1 |
| 685 | 9311688 | 210 | 1042 | 12 | 5 | 3 |
| 686 | 9311689 | 225 | 1343 | 22 | 5 | 4 |
| 687 | 9311690 | 219 | 1274 | 15 | 1 | 1 |
| 688 | 9311691 | 201 | 863 | 23 | 3 | 0 |
| 689 | 9311692 | 445 | 3319 | 43 | 7 | 1 |
| 690 | 9311693 | 256 | 1434 | 31 | 6 | 2 |
| 691 | 9311694 | 200 | 1201 | 26 | 3 | 2 |
| 692 | 9311695 | 339 | 2169 | 30 | 4 | 3 |
| 693 | 9311696 | 268 | 1286 | 22 | 7 | 2 |
| 694 | 9311697 | 200 | 1085 | 12 | 6 | 2 |
| 695 | 9311698 | 254 | 1182 | 12 | 3 | 4 |
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
| 720 | 9311723 | 216 | 1282 | 25 | 5 | 3 |
| 721 | 9311724 | 254 | 1696 | 37 | 8 | 2 |
| 722 | 9311725 | 193 | 1047 | 11 | 0 | 3 |
| 723 | 9311726 | 355 | 1217 | 44 | 5 | 6 |
| 724 | 9311727 | 234 | 1314 | 23 | 1 | 3 |
| 725 | 9311728 | 186 | 980 | 18 | 4 | 1 |
| 726 | 9311729 | 182 | 931 | 18 | 5 | 3 |
| 727 | 9311730 | 292 | 1668 | 35 | 10 | 3 |
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
| 742 | 9311745 | 359 | 2396 | 33 | 6 | 4 |
| 743 | 9311746 | 249 | 1269 | 23 | 2 | 5 |
| 744 | 9311747 | 281 | 1862 | 40 | 1 | 3 |
| 745 | 9311748 | 235 | 1315 | 24 | 4 | 3 |
| 746 | 9311749 | 274 | 1713 | 26 | 5 | 3 |
| 747 | 9311750 | 238 | 1380 | 28 | 0 | 2 |
| 748 | 9311751 | 261 | 1389 | 31 | 0 | 4 |
| 749 | 9311752 | 314 | 1435 | 38 | 2 | 6 |
| 750 | 9311753 | 185 | 772 | 15 | 2 | 4 |
| 751 | 9311754 | 241 | 729 | 22 | 4 | 5 |
| 752 | 9311755 | 268 | 1697 | 40 | 4 | 3 |
| 753 | 9311756 | 381 | 2270 | 24 | 2 | 3 |
| 754 | 9311757 | 228 | 1135 | 24 | 1 | 3 |
| 755 | 9311758 | 249 | 1406 | 21 | 3 | 1 |
| 756 | 9311759 | 226 | 1100 | 21 | 6 | 3 |
| 757 | 9311760 | 244 | 1027 | 24 | 0 | 4 |
| 758 | 9311761 | 279 | 1243 | 46 | 1 | 4 |
| 759 | 9311762 | 500 | 2193 | 38 | 1 | 5 |
| 760 | 9311763 | 165 | 689 | 12 | 9 | 1 |
| 761 | 9311764 | 226 | 1021 | 27 | 5 | 2 |
| 762 | 9311765 | 214 | 1144 | 32 | 3 | 2 |
| 763 | 9311766 | 283 | 1349 | 27 | 4 | 4 |
| 764 | 9311767 | 246 | 1369 | 23 | 2 | 2 |
| 765 | 9311768 | 223 | 1379 | 32 | 4 | 2 |
| 766 | 9311769 | 232 | 1329 | 31 | 10 | 3 |
| 767 | 9311770 | 215 | 885 | 22 | 12 | 4 |
| 768 | 9311771 | 244 | 1268 | 29 | 2 | 1 |
| 769 | 9311772 | 216 | 1176 | 22 | 4 | 2 |
| 770 | 9311773 | 253 | 1387 | 29 | 3 | 4 |
| 771 | 9311774 | 269 | 1654 | 30 | 4 | 4 |
| 772 | 9311775 | 245 | 1352 | 18 | 6 | 3 |
| 773 | 9311776 | 238 | 989 | 19 | 3 | 0 |
| 774 | 9311777 | 188 | 915 | 12 | 3 | 1 |
| 775 | 9311778 | 208 | 1006 | 16 | 6 | 5 |
| 776 | 9311779 | 296 | 1612 | 41 | 4 | 2 |
| 777 | 9311780 | 230 | 1253 | 20 | 5 | 3 |
| 778 | 9311781 | 228 | 1364 | 16 | 0 | 3 |
| 779 | 9311782 | 189 | 902 | 8 | 5 | 3 |
| 780 | 9311783 | 247 | 1354 | 24 | 1 | 2 |
| 781 | 9311784 | 282 | 1609 | 31 | 6 | 2 |
| 782 | 9311785 | 214 | 1051 | 27 | 5 | 3 |
| 783 | 9311786 | 279 | 1596 | 35 | 3 | 4 |
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
| 808 | 9311811 | 294 | 1374 | 26 | 7 | 2 |
| 809 | 9311812 | 224 | 1237 | 17 | 4 | 3 |
| 810 | 9311813 | 288 | 1588 | 25 | 3 | 2 |
| 811 | 9311814 | 333 | 2055 | 42 | 3 | 3 |
| 812 | 9311815 | 273 | 1566 | 37 | 9 | 2 |
| 813 | 9311816 | 252 | 1363 | 28 | 7 | 3 |
| 814 | 9311817 | 275 | 1345 | 27 | 7 | 2 |
| 815 | 9311818 | 246 | 1281 | 17 | 7 | 3 |
| 816 | 9311819 | 207 | 1122 | 15 | 4 | 2 |
| 817 | 9311820 | 319 | 1553 | 29 | 9 | 3 |
| 818 | 9311821 | 212 | 1408 | 21 | 3 | 3 |
| 819 | 9311822 | 220 | 1016 | 27 | 6 | 3 |
| 820 | 9311823 | 224 | 1116 | 19 | 2 | 4 |
| 821 | 9311824 | 230 | 1041 | 10 | 3 | 4 |
| 822 | 9311825 | 401 | 2413 | 27 | 11 | 4 |
| 823 | 9311826 | 230 | 1390 | 28 | 1 | 2 |
| 824 | 9311827 | 291 | 1820 | 22 | 2 | 5 |
| 825 | 9311828 | 214 | 1225 | 18 | 5 | 2 |
| 826 | 9311829 | 234 | 931 | 22 | 3 | 4 |
| 827 | 9311830 | 271 | 1490 | 38 | 2 | 2 |
| 828 | 9311831 | 225 | 1286 | 24 | 1 | 3 |
| 829 | 9311832 | 288 | 1613 | 33 | 5 | 1 |
| 830 | 9311833 | 205 | 1149 | 21 | 4 | 1 |
| 831 | 9311834 | 432 | 2441 | 65 | 0 | 6 |
| 832 | 9311835 | 248 | 1319 | 32 | 5 | 5 |
| 833 | 9311836 | 229 | 1582 | 36 | 2 | 3 |
| 834 | 9311837 | 180 | 998 | 15 | 2 | 2 |
| 835 | 9311838 | 240 | 1635 | 40 | 4 | 5 |
| 836 | 9311839 | 244 | 1149 | 19 | 6 | 3 |
| 837 | 9311840 | 268 | 1267 | 31 | 1 | 4 |
| 838 | 9311841 | 274 | 1494 | 26 | 2 | 3 |
| 839 | 9311842 | 211 | 1232 | 24 | 1 | 3 |
| 840 | 9311843 | 218 | 1032 | 17 | 0 | 3 |
| 841 | 9311844 | 188 | 631 | 13 | 4 | 3 |
| 842 | 9311845 | 282 | 1239 | 27 | 3 | 2 |
| 843 | 9311846 | 228 | 1328 | 22 | 4 | 1 |
| 844 | 9311847 | 215 | 1078 | 25 | 1 | 1 |
| 845 | 9311848 | 222 | 1444 | 37 | 3 | 3 |
| 846 | 9311849 | 261 | 1540 | 29 | 2 | 4 |
| 847 | 9311850 | 222 | 1158 | 24 | 3 | 2 |
| 848 | 9311851 | 246 | 1339 | 29 | 9 | 0 |
| 849 | 9311852 | 327 | 1713 | 35 | 7 | 4 |
| 850 | 9311853 | 276 | 1376 | 24 | 4 | 2 |
| 851 | 9311854 | 222 | 953 | 26 | 4 | 4 |
| 852 | 9311855 | 283 | 1888 | 37 | 7 | 3 |
| 853 | 9311856 | 253 | 1570 | 26 | 6 | 1 |
| 854 | 9311857 | 186 | 928 | 15 | 2 | 1 |
| 855 | 9311858 | 168 | 721 | 8 | 4 | 0 |
| 856 | 9311859 | 284 | 1382 | 33 | 10 | 3 |
| 857 | 9311860 | 216 | 1483 | 27 | 2 | 2 |
| 858 | 9311861 | 252 | 1251 | 20 | 2 | 3 |
| 859 | 9311862 | 212 | 1391 | 9 | 2 | 1 |
| 860 | 9311863 | 186 | 1012 | 16 | 2 | 1 |
| 861 | 9311864 | 201 | 1094 | 24 | 2 | 1 |
| 862 | 9311865 | 252 | 1505 | 21 | 7 | 1 |
| 863 | 9311866 | 263 | 1755 | 35 | 5 | 1 |
| 864 | 9311867 | 254 | 1099 | 25 | 4 | 2 |
| 865 | 9311868 | 243 | 1366 | 36 | 4 | 2 |
| 866 | 9311869 | 337 | 1857 | 35 | 9 | 3 |
| 867 | 9311870 | 384 | 1977 | 33 | 9 | 5 |
| 868 | 9311871 | 180 | 855 | 7 | 5 | 3 |
| 869 | 9311872 | 267 | 1387 | 38 | 6 | 5 |
| 870 | 9311873 | 274 | 1468 | 31 | 7 | 4 |
| 871 | 9311874 | 208 | 982 | 19 | 1 | 1 |
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
| 896 | 9311899 | 220 | 1092 | 20 | 7 | 2 |
| 897 | 9311900 | 227 | 1177 | 24 | 5 | 3 |
| 898 | 9311901 | 151 | 498 | 13 | 2 | 3 |
| 899 | 9311902 | 241 | 1392 | 19 | 4 | 3 |
| 900 | 9311903 | 184 | 1035 | 19 | 5 | 4 |
| 901 | 9311904 | 196 | 998 | 19 | 5 | 3 |
| 902 | 9311905 | 214 | 1067 | 18 | 6 | 0 |
| 903 | 9311906 | 138 | 592 | 9 | 4 | 1 |
| 904 | 9311907 | 328 | 1636 | 29 | 7 | 4 |
| 905 | 9311908 | 293 | 1558 | 26 | 1 | 5 |
| 906 | 9311909 | 302 | 1976 | 27 | 3 | 3 |
| 907 | 9311910 | 273 | 1582 | 27 | 1 | 4 |
| 908 | 9311911 | 179 | 877 | 12 | 0 | 1 |
| 909 | 9311912 | 279 | 1237 | 43 | 1 | 5 |
| 910 | 9311913 | 231 | 1272 | 24 | 2 | 3 |
| 911 | 9311914 | 191 | 935 | 10 | 3 | 4 |
| 912 | 9311915 | 309 | 2008 | 22 | 4 | 5 |
| 913 | 9311916 | 280 | 2021 | 25 | 6 | 4 |
| 914 | 9311917 | 366 | 2325 | 44 | 8 | 5 |
| 915 | 9311918 | 287 | 1447 | 35 | 3 | 3 |
| 916 | 9311919 | 257 | 1325 | 22 | 3 | 3 |
| 917 | 9311920 | 253 | 1463 | 27 | 1 | 3 |
| 918 | 9311921 | 247 | 1274 | 28 | 1 | 4 |
| 919 | 9311922 | 214 | 867 | 15 | 1 | 4 |
| 920 | 9311923 | 233 | 1489 | 21 | 1 | 2 |
| 921 | 9311924 | 189 | 1066 | 20 | 3 | 4 |
| 922 | 9311925 | 253 | 1152 | 31 | 5 | 3 |
| 923 | 9311926 | 300 | 1519 | 33 | 3 | 4 |
| 924 | 9311927 | 254 | 1406 | 29 | 3 | 4 |
| 925 | 9311928 | 202 | 1029 | 19 | 0 | 4 |
| 926 | 9311929 | 261 | 1490 | 28 | 3 | 4 |
| 927 | 9311930 | 268 | 1274 | 19 | 2 | 2 |
| 928 | 9311931 | 182 | 803 | 14 | 1 | 3 |
| 929 | 9311932 | 284 | 1321 | 22 | 1 | 3 |
| 930 | 9311933 | 244 | 1118 | 27 | 1 | 5 |
| 931 | 9311934 | 244 | 1224 | 27 | 2 | 4 |
| 932 | 9311935 | 186 | 999 | 21 | 6 | 4 |
| 933 | 9311936 | 241 | 1618 | 29 | 3 | 2 |
| 934 | 9311937 | 199 | 984 | 17 | 4 | 4 |
| 935 | 9311938 | 238 | 1209 | 19 | 6 | 4 |
| 936 | 9311939 | 221 | 1093 | 25 | 3 | 1 |
| 937 | 9311940 | 285 | 1713 | 43 | 3 | 4 |
| 938 | 9311941 | 206 | 1304 | 24 | 4 | 2 |
| 939 | 9311942 | 259 | 1405 | 23 | 5 | 2 |
| 940 | 9311943 | 259 | 1431 | 34 | 7 | 2 |
| 941 | 9311944 | 228 | 1361 | 34 | 6 | 1 |
| 942 | 9311945 | 221 | 1237 | 30 | 1 | 3 |
| 943 | 9311946 | 237 | 1397 | 31 | 4 | 2 |
| 944 | 9311947 | 234 | 1221 | 20 | 1 | 3 |
| 945 | 9311948 | 258 | 1225 | 19 | 1 | 3 |
| 946 | 9311949 | 243 | 1420 | 30 | 7 | 2 |
| 947 | 9311950 | 211 | 756 | 14 | 5 | 1 |
| 948 | 9311951 | 300 | 1966 | 28 | 5 | 2 |
| 949 | 9311952 | 186 | 711 | 12 | 2 | 2 |
| 950 | 9311953 | 288 | 1463 | 40 | 2 | 2 |
| 951 | 9311954 | 274 | 1319 | 13 | 6 | 3 |
| 952 | 9311955 | 217 | 1219 | 32 | 5 | 2 |
| 953 | 9311956 | 287 | 1390 | 25 | 8 | 4 |
| 954 | 9311957 | 252 | 1468 | 19 | 2 | 1 |
| 955 | 9311958 | 356 | 2116 | 46 | 5 | 4 |
| 956 | 9311959 | 236 | 1564 | 20 | 1 | 2 |
| 957 | 9311960 | 216 | 1050 | 25 | 11 | 3 |
| 958 | 9311961 | 285 | 1676 | 30 | 2 | 3 |
| 959 | 9311962 | 222 | 981 | 22 | 3 | 3 |
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
| 984 | 9311987 | 274 | 1460 | 36 | 6 | 2 |
| 985 | 9311988 | 283 | 1393 | 29 | 6 | 3 |
| 986 | 9311989 | 297 | 1823 | 34 | 8 | 2 |
| 987 | 9311990 | 267 | 1482 | 27 | 3 | 3 |
| 988 | 9311991 | 180 | 995 | 6 | 7 | 1 |
| 989 | 9311992 | 209 | 1088 | 16 | 7 | 5 |
| 990 | 9311993 | 293 | 1346 | 22 | 2 | 4 |
| 991 | 9311994 | 219 | 1396 | 19 | 4 | 3 |
| 992 | 9311995 | 221 | 1191 | 26 | 0 | 4 |
| 993 | 9311996 | 212 | 995 | 18 | 4 | 3 |
| 994 | 9311997 | 500 | 2374 | 44 | 4 | 4 |
| 995 | 9311998 | 496 | 1950 | 62 | 9 | 6 |
| 996 | 9311999 | 244 | 1466 | 19 | 11 | 2 |
| 997 | 9312000 | 311 | 1595 | 19 | 7 | 5 |
| 998 | 9312001 | 226 | 1070 | 26 | 1 | 3 |
| 999 | 9312002 | 193 | 709 | 22 | 2 | 4 |
| 1000 | 9312003 | 260 | 1501 | 34 | 3 | 1 |
| 1001 | 9312004 | 237 | 1119 | 20 | 3 | 2 |
| 1002 | 9312005 | 201 | 717 | 7 | 1 | 3 |
| 1003 | 9312006 | 215 | 895 | 24 | 2 | 5 |
| 1004 | 9312007 | 264 | 1266 | 17 | 3 | 3 |
| 1005 | 9312008 | 183 | 1145 | 20 | 4 | 3 |
| 1006 | 9312009 | 320 | 1479 | 38 | 0 | 5 |
| 1007 | 9312010 | 270 | 1205 | 32 | 3 | 5 |
| 1008 | 9312011 | 237 | 1036 | 28 | 6 | 3 |
| 1009 | 9312012 | 199 | 952 | 24 | 4 | 3 |
| 1010 | 9312013 | 300 | 1427 | 33 | 2 | 4 |
| 1011 | 9312014 | 214 | 1211 | 29 | 4 | 2 |
| 1012 | 9312015 | 181 | 903 | 22 | 2 | 3 |
| 1013 | 9312016 | 358 | 2062 | 44 | 3 | 3 |
| 1014 | 9312017 | 285 | 1678 | 20 | 2 | 3 |
| 1015 | 9312018 | 268 | 1285 | 37 | 5 | 3 |
| 1016 | 9312019 | 193 | 746 | 13 | 4 | 2 |
| 1017 | 9312020 | 197 | 1034 | 16 | 1 | 1 |
| 1018 | 9312021 | 166 | 630 | 16 | 1 | 3 |
| 1019 | 9312022 | 254 | 1585 | 36 | 0 | 3 |
| 1020 | 9312023 | 198 | 1047 | 18 | 3 | 2 |
| 1021 | 9312024 | 262 | 1402 | 27 | 1 | 3 |
| 1022 | 9312025 | 227 | 1126 | 15 | 1 | 3 |
| 1023 | 9312026 | 292 | 1608 | 26 | 3 | 0 |
| 1024 | 9312027 | 260 | 1592 | 30 | 9 | 1 |
| 1025 | 9312028 | 233 | 1357 | 32 | 4 | 2 |
| 1026 | 9312029 | 274 | 1319 | 30 | 5 | 3 |
| 1027 | 9312030 | 197 | 953 | 27 | 4 | 3 |
| 1028 | 9312031 | 199 | 1102 | 28 | 7 | 3 |
| 1029 | 9312032 | 256 | 1435 | 17 | 6 | 2 |
| 1030 | 9312033 | 396 | 1441 | 23 | 4 | 2 |
| 1031 | 9312034 | 204 | 903 | 21 | 0 | 4 |
| 1032 | 9312035 | 244 | 1321 | 37 | 8 | 3 |
| 1033 | 9312036 | 211 | 827 | 22 | 5 | 3 |
| 1034 | 9312037 | 156 | 639 | 17 | 2 | 2 |
| 1035 | 9312038 | 236 | 1189 | 22 | 7 | 1 |
| 1036 | 9312039 | 217 | 1312 | 28 | 4 | 1 |
| 1037 | 9312040 | 262 | 1259 | 23 | 7 | 5 |
| 1038 | 9312041 | 230 | 1350 | 23 | 4 | 5 |
| 1039 | 9312042 | 284 | 1749 | 24 | 8 | 4 |
| 1040 | 9312043 | 164 | 621 | 15 | 2 | 1 |
| 1041 | 9312044 | 258 | 1278 | 23 | 0 | 3 |
| 1042 | 9312045 | 229 | 942 | 23 | 3 | 2 |
| 1043 | 9312046 | 226 | 1363 | 25 | 4 | 2 |
| 1044 | 9312047 | 315 | 1698 | 38 | 7 | 1 |
| 1045 | 9312048 | 252 | 1288 | 26 | 2 | 1 |
| 1046 | 9312049 | 248 | 1017 | 30 | 5 | 2 |
| 1047 | 9312050 | 280 | 1234 | 24 | 5 | 4 |
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
| 1072 | 9312075 | 355 | 1751 | 44 | 1 | 4 |
| 1073 | 9312076 | 278 | 1494 | 23 | 6 | 2 |
| 1074 | 9312077 | 293 | 2013 | 31 | 6 | 3 |
| 1075 | 9312078 | 210 | 1196 | 16 | 2 | 4 |
| 1076 | 9312079 | 250 | 1294 | 18 | 0 | 3 |
| 1077 | 9312080 | 274 | 1418 | 27 | 5 | 3 |
| 1078 | 9312081 | 301 | 1910 | 39 | 4 | 3 |
| 1079 | 9312082 | 235 | 906 | 21 | 1 | 3 |
| 1080 | 9312083 | 500 | 2794 | 48 | 4 | 4 |
| 1081 | 9312084 | 232 | 1166 | 23 | 6 | 3 |
| 1082 | 9312085 | 275 | 1420 | 20 | 8 | 3 |
| 1083 | 9312086 | 284 | 1385 | 30 | 7 | 5 |
| 1084 | 9312087 | 329 | 1407 | 30 | 5 | 4 |
| 1085 | 9312088 | 259 | 1146 | 22 | 4 | 5 |
| 1086 | 9312089 | 227 | 1388 | 22 | 3 | 2 |
| 1087 | 9312090 | 193 | 1018 | 19 | 3 | 4 |
| 1088 | 9312091 | 182 | 802 | 9 | 5 | 1 |
| 1089 | 9312092 | 268 | 1640 | 24 | 1 | 3 |
| 1090 | 9312093 | 211 | 1180 | 12 | 0 | 2 |
| 1091 | 9312094 | 212 | 1122 | 19 | 0 | 3 |
| 1092 | 9312095 | 203 | 1076 | 16 | 1 | 2 |
| 1093 | 9312096 | 259 | 1576 | 28 | 2 | 4 |
| 1094 | 9312097 | 310 | 1555 | 33 | 2 | 5 |
| 1095 | 9312098 | 185 | 936 | 19 | 1 | 2 |
| 1096 | 9312099 | 258 | 1517 | 21 | 1 | 3 |
| 1097 | 9312100 | 267 | 1465 | 33 | 1 | 3 |
| 1098 | 9312101 | 165 | 714 | 16 | 1 | 4 |
| 1099 | 9312102 | 162 | 438 | 11 | 2 | 2 |
| 1100 | 9312103 | 213 | 1176 | 19 | 2 | 2 |
| 1101 | 9312104 | 225 | 1172 | 20 | 2 | 4 |
| 1102 | 9312105 | 273 | 1691 | 30 | 2 | 3 |
| 1103 | 9312106 | 244 | 1443 | 21 | 0 | 3 |
| 1104 | 9312107 | 220 | 928 | 19 | 3 | 3 |
| 1105 | 9312108 | 251 | 912 | 29 | 3 | 4 |
| 1106 | 9312109 | 183 | 900 | 14 | 2 | 2 |
| 1107 | 9312110 | 256 | 1204 | 26 | 1 | 4 |
| 1108 | 9312111 | 237 | 1111 | 33 | 2 | 3 |
| 1109 | 9312112 | 229 | 1225 | 18 | 2 | 2 |
| 1110 | 9312113 | 187 | 949 | 10 | 1 | 3 |
| 1111 | 9312114 | 309 | 1345 | 40 | 0 | 4 |
| 1112 | 9312115 | 249 | 1525 | 29 | 4 | 2 |
| 1113 | 9312116 | 177 | 764 | 13 | 4 | 3 |
| 1114 | 9312117 | 218 | 1090 | 21 | 4 | 2 |
| 1115 | 9312118 | 220 | 1150 | 31 | 0 | 2 |
| 1116 | 9312119 | 166 | 1004 | 18 | 1 | 1 |
| 1117 | 9312120 | 272 | 1517 | 33 | 12 | 4 |
| 1118 | 9312121 | 264 | 1279 | 20 | 5 | 2 |
| 1119 | 9312122 | 192 | 1046 | 18 | 6 | 0 |
| 1120 | 9312123 | 195 | 933 | 19 | 3 | 3 |
| 1121 | 9312124 | 275 | 1627 | 20 | 2 | 4 |
| 1122 | 9312125 | 299 | 1870 | 23 | 4 | 1 |
| 1123 | 9312126 | 256 | 1528 | 35 | 0 | 3 |
| 1124 | 9312127 | 199 | 1102 | 16 | 4 | 2 |
| 1125 | 9312128 | 233 | 1174 | 39 | 5 | 5 |
| 1126 | 9312129 | 289 | 1405 | 40 | 5 | 4 |
| 1127 | 9312130 | 294 | 2164 | 30 | 4 | 2 |
| 1128 | 9312131 | 271 | 1530 | 32 | 4 | 3 |
| 1129 | 9312132 | 207 | 853 | 28 | 4 | 2 |
| 1130 | 9312133 | 224 | 1279 | 22 | 7 | 1 |
| 1131 | 9312134 | 242 | 1279 | 35 | 4 | 5 |
| 1132 | 9312135 | 252 | 1736 | 23 | 5 | 4 |
| 1133 | 9312136 | 216 | 1171 | 18 | 4 | 3 |
| 1134 | 9312137 | 205 | 988 | 20 | 5 | 1 |
| 1135 | 9312138 | 275 | 1772 | 30 | 4 | 2 |
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
| 1160 | 9312163 | 168 | 640 | 14 | 3 | 1 |
| 1161 | 9312164 | 246 | 1150 | 20 | 4 | 2 |
| 1162 | 9312165 | 277 | 1567 | 31 | 3 | 4 |
| 1163 | 9312166 | 200 | 1097 | 20 | 3 | 1 |
| 1164 | 9312167 | 294 | 1912 | 31 | 2 | 4 |
| 1165 | 9312168 | 249 | 1292 | 26 | 6 | 1 |
| 1166 | 9312169 | 244 | 1465 | 23 | 10 | 0 |
| 1167 | 9312170 | 202 | 931 | 13 | 3 | 3 |
| 1168 | 9312171 | 225 | 1153 | 21 | 4 | 2 |
| 1169 | 9312172 | 240 | 1276 | 24 | 2 | 4 |
| 1170 | 9312173 | 218 | 1046 | 21 | 3 | 4 |
| 1171 | 9312174 | 204 | 1288 | 25 | 4 | 5 |
| 1172 | 9312175 | 262 | 1196 | 29 | 4 | 4 |
| 1173 | 9312176 | 418 | 2365 | 48 | 7 | 4 |
| 1174 | 9312177 | 228 | 1154 | 19 | 1 | 5 |
| 1175 | 9312178 | 278 | 1686 | 16 | 3 | 4 |
| 1176 | 9312179 | 223 | 1425 | 36 | 1 | 2 |
| 1177 | 9312180 | 302 | 1889 | 24 | 6 | 4 |
| 1178 | 9312181 | 228 | 1305 | 19 | 1 | 2 |
| 1179 | 9312182 | 303 | 1257 | 26 | 1 | 4 |
| 1180 | 9312183 | 245 | 1292 | 25 | 2 | 3 |
| 1181 | 9312184 | 243 | 1419 | 21 | 2 | 3 |
| 1182 | 9312185 | 242 | 1150 | 36 | 3 | 3 |
| 1183 | 9312186 | 228 | 1057 | 12 | 0 | 4 |
| 1184 | 9312187 | 342 | 1837 | 42 | 4 | 3 |
| 1185 | 9312188 | 231 | 1475 | 31 | 2 | 3 |
| 1186 | 9312189 | 241 | 1105 | 28 | 3 | 3 |
| 1187 | 9312190 | 219 | 971 | 24 | 2 | 2 |
| 1188 | 9312191 | 220 | 1367 | 17 | 3 | 3 |
| 1189 | 9312192 | 245 | 1259 | 29 | 7 | 3 |
| 1190 | 9312193 | 205 | 736 | 15 | 2 | 4 |
| 1191 | 9312194 | 217 | 1234 | 20 | 3 | 3 |
| 1192 | 9312195 | 272 | 1431 | 32 | 3 | 4 |
| 1193 | 9312196 | 241 | 987 | 17 | 3 | 2 |
| 1194 | 9312197 | 286 | 1589 | 43 | 7 | 4 |
| 1195 | 9312198 | 300 | 1556 | 31 | 6 | 5 |
| 1196 | 9312199 | 189 | 758 | 19 | 3 | 3 |
| 1197 | 9312200 | 181 | 826 | 15 | 1 | 3 |
| 1198 | 9312201 | 263 | 1254 | 35 | 0 | 4 |
| 1199 | 9312202 | 223 | 1417 | 28 | 3 | 0 |
| 1200 | 9312203 | 295 | 1683 | 42 | 3 | 3 |
| 1201 | 9312204 | 259 | 1418 | 33 | 6 | 5 |
| 1202 | 9312205 | 213 | 1100 | 25 | 3 | 4 |
| 1203 | 9312206 | 231 | 1288 | 29 | 2 | 3 |
| 1204 | 9312207 | 266 | 1565 | 33 | 10 | 1 |
| 1205 | 9312208 | 247 | 1011 | 21 | 6 | 4 |
| 1206 | 9312209 | 247 | 954 | 24 | 8 | 3 |
| 1207 | 9312210 | 300 | 1625 | 33 | 6 | 2 |
| 1208 | 9312211 | 228 | 1466 | 21 | 6 | 3 |
| 1209 | 9312212 | 500 | 2650 | 48 | 7 | 4 |
| 1210 | 9312213 | 450 | 1986 | 28 | 5 | 3 |
| 1211 | 9312214 | 195 | 848 | 12 | 3 | 0 |
| 1212 | 9312215 | 226 | 1376 | 20 | 5 | 1 |
| 1213 | 9312216 | 244 | 1173 | 21 | 4 | 3 |
| 1214 | 9312217 | 232 | 1313 | 31 | 7 | 2 |
| 1215 | 9312218 | 249 | 1355 | 22 | 3 | 2 |
| 1216 | 9312219 | 199 | 1178 | 13 | 4 | 2 |
| 1217 | 9312220 | 226 | 1291 | 32 | 3 | 1 |
| 1218 | 9312221 | 259 | 1330 | 29 | 1 | 4 |
| 1219 | 9312222 | 210 | 996 | 21 | 4 | 4 |
| 1220 | 9312223 | 206 | 1039 | 19 | 2 | 3 |
| 1221 | 9312224 | 191 | 1016 | 16 | 1 | 0 |
| 1222 | 9312225 | 234 | 1194 | 12 | 6 | 3 |
| 1223 | 9312226 | 379 | 2143 | 32 | 9 | 1 |
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
| 1248 | 9312251 | 285 | 1366 | 28 | 4 | 3 |
| 1249 | 9312252 | 296 | 1612 | 19 | 5 | 4 |
| 1250 | 9312253 | 231 | 1199 | 26 | 2 | 3 |
| 1251 | 9312254 | 229 | 1088 | 23 | 5 | 3 |
| 1252 | 9312255 | 252 | 1417 | 15 | 1 | 4 |
| 1253 | 9312256 | 310 | 1856 | 26 | 1 | 2 |
| 1254 | 9312257 | 289 | 1643 | 27 | 0 | 3 |
| 1255 | 9312258 | 279 | 1669 | 26 | 5 | 5 |
| 1256 | 9312259 | 208 | 1157 | 21 | 2 | 1 |
| 1257 | 9312260 | 199 | 988 | 16 | 4 | 3 |
| 1258 | 9312261 | 230 | 1203 | 23 | 9 | 3 |
| 1259 | 9312262 | 245 | 1230 | 26 | 2 | 4 |
| 1260 | 9312263 | 232 | 1518 | 27 | 0 | 2 |
| 1261 | 9312264 | 273 | 1396 | 25 | 2 | 3 |
| 1262 | 9312265 | 269 | 1304 | 22 | 5 | 2 |
| 1263 | 9312266 | 235 | 1115 | 17 | 1 | 4 |
| 1264 | 9312267 | 366 | 2220 | 42 | 2 | 5 |
| 1265 | 9312268 | 316 | 1907 | 43 | 3 | 5 |
| 1266 | 9312269 | 211 | 936 | 13 | 2 | 3 |
| 1267 | 9312270 | 137 | 670 | 13 | 2 | 2 |
| 1268 | 9312271 | 212 | 1090 | 18 | 7 | 4 |
| 1269 | 9312272 | 276 | 1642 | 18 | 4 | 4 |
| 1270 | 9312273 | 218 | 1079 | 20 | 4 | 2 |
| 1271 | 9312274 | 332 | 1909 | 38 | 1 | 3 |
| 1272 | 9312275 | 181 | 958 | 13 | 2 | 2 |
| 1273 | 9312276 | 254 | 1447 | 24 | 2 | 2 |
| 1274 | 9312277 | 226 | 1156 | 30 | 1 | 4 |
| 1275 | 9312278 | 212 | 1155 | 24 | 7 | 4 |
| 1276 | 9312279 | 285 | 1219 | 29 | 0 | 3 |
| 1277 | 9312280 | 184 | 870 | 19 | 1 | 2 |
| 1278 | 9312281 | 382 | 1782 | 47 | 3 | 5 |
| 1279 | 9312282 | 185 | 936 | 20 | 2 | 4 |
| 1280 | 9312283 | 369 | 1539 | 35 | 2 | 3 |
| 1281 | 9312284 | 224 | 907 | 24 | 2 | 2 |
| 1282 | 9312285 | 187 | 1097 | 20 | 3 | 2 |
| 1283 | 9312286 | 189 | 1203 | 15 | 2 | 3 |
| 1284 | 9312287 | 314 | 2008 | 21 | 3 | 2 |
| 1285 | 9312288 | 315 | 1423 | 22 | 4 | 3 |
| 1286 | 9312289 | 201 | 1312 | 20 | 6 | 2 |
| 1287 | 9312290 | 270 | 1226 | 26 | 2 | 3 |
| 1288 | 9312291 | 243 | 1289 | 27 | 4 | 2 |
| 1289 | 9312292 | 224 | 1014 | 15 | 2 | 2 |
| 1290 | 9312293 | 180 | 1020 | 16 | 2 | 4 |
| 1291 | 9312294 | 219 | 1398 | 24 | 4 | 3 |
| 1292 | 9312295 | 280 | 1571 | 36 | 8 | 5 |
| 1293 | 9312296 | 227 | 1107 | 22 | 6 | 2 |
| 1294 | 9312297 | 187 | 1096 | 26 | 2 | 1 |
| 1295 | 9312298 | 259 | 1585 | 35 | 5 | 4 |
| 1296 | 9312299 | 207 | 989 | 24 | 5 | 2 |
| 1297 | 9312300 | 188 | 1084 | 26 | 3 | 2 |
| 1298 | 9312301 | 353 | 1683 | 27 | 3 | 1 |
| 1299 | 9312302 | 193 | 1214 | 11 | 2 | 2 |
| 1300 | 9312303 | 193 | 876 | 12 | 3 | 1 |
| 1301 | 9312304 | 311 | 1793 | 29 | 13 | 6 |
| 1302 | 9312305 | 271 | 1770 | 27 | 6 | 2 |
| 1303 | 9312306 | 285 | 1417 | 18 | 2 | 3 |
| 1304 | 9312307 | 284 | 1767 | 33 | 3 | 2 |
| 1305 | 9312308 | 328 | 1955 | 41 | 3 | 3 |
| 1306 | 9312309 | 267 | 1341 | 32 | 2 | 4 |
| 1307 | 9312310 | 308 | 2103 | 26 | 4 | 4 |
| 1308 | 9312311 | 312 | 1566 | 21 | 4 | 3 |
| 1309 | 9312312 | 214 | 1195 | 29 | 3 | 1 |
| 1310 | 9312313 | 275 | 1374 | 29 | 3 | 3 |
| 1311 | 9312314 | 386 | 2191 | 27 | 2 | 5 |
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
| 1336 | 9312339 | 184 | 877 | 11 | 4 | 2 |
| 1337 | 9312340 | 194 | 1083 | 22 | 2 | 3 |
| 1338 | 9312341 | 339 | 2447 | 37 | 8 | 4 |
| 1339 | 9312342 | 273 | 1375 | 25 | 4 | 5 |
| 1340 | 9312343 | 232 | 1243 | 24 | 6 | 4 |
| 1341 | 9312344 | 310 | 1415 | 34 | 10 | 6 |
| 1342 | 9312345 | 187 | 826 | 17 | 4 | 1 |
| 1343 | 9312346 | 278 | 1634 | 36 | 4 | 2 |
| 1344 | 9312347 | 188 | 916 | 10 | 3 | 5 |
| 1345 | 9312348 | 344 | 1992 | 42 | 7 | 5 |
| 1346 | 9312349 | 276 | 1496 | 32 | 5 | 4 |
| 1347 | 9312350 | 300 | 1241 | 33 | 5 | 6 |
| 1348 | 9312351 | 239 | 1588 | 24 | 3 | 2 |
| 1349 | 9312352 | 269 | 1610 | 30 | 1 | 2 |
| 1350 | 9312353 | 231 | 1106 | 15 | 3 | 4 |
| 1351 | 9312354 | 304 | 1430 | 17 | 1 | 5 |
| 1352 | 9312355 | 321 | 2157 | 45 | 2 | 4 |
| 1353 | 9312356 | 192 | 796 | 16 | 1 | 4 |
| 1354 | 9312357 | 198 | 1111 | 14 | 4 | 2 |
| 1355 | 9312358 | 207 | 1030 | 22 | 0 | 2 |
| 1356 | 9312359 | 263 | 1463 | 27 | 2 | 5 |
| 1357 | 9312360 | 267 | 1278 | 27 | 6 | 4 |
| 1358 | 9312361 | 181 | 595 | 16 | 1 | 5 |
| 1359 | 9312362 | 208 | 815 | 16 | 5 | 2 |
| 1360 | 9312363 | 257 | 1417 | 28 | 6 | 5 |
| 1361 | 9312364 | 276 | 1182 | 37 | 0 | 3 |
| 1362 | 9312365 | 253 | 1557 | 24 | 0 | 2 |
| 1363 | 9312366 | 298 | 1633 | 42 | 4 | 4 |
| 1364 | 9312367 | 330 | 2087 | 43 | 3 | 4 |
| 1365 | 9312368 | 315 | 1861 | 34 | 0 | 3 |
| 1366 | 9312369 | 231 | 1206 | 21 | 2 | 2 |
| 1367 | 9312370 | 220 | 1266 | 21 | 2 | 4 |
| 1368 | 9312371 | 238 | 1381 | 22 | 3 | 1 |
| 1369 | 9312372 | 220 | 931 | 19 | 0 | 1 |
| 1370 | 9312373 | 257 | 1101 | 16 | 4 | 2 |
| 1371 | 9312374 | 210 | 1191 | 24 | 6 | 1 |
| 1372 | 9312375 | 352 | 1656 | 34 | 3 | 5 |
| 1373 | 9312376 | 229 | 1451 | 27 | 1 | 4 |
| 1374 | 9312377 | 252 | 1231 | 30 | 4 | 4 |
| 1375 | 9312378 | 236 | 1186 | 25 | 4 | 2 |
| 1376 | 9312379 | 247 | 1086 | 28 | 4 | 4 |
| 1377 | 9312380 | 230 | 1299 | 24 | 1 | 1 |
| 1378 | 9312381 | 255 | 1518 | 28 | 6 | 2 |
| 1379 | 9312382 | 193 | 1189 | 19 | 7 | 2 |
| 1380 | 9312383 | 238 | 1530 | 28 | 2 | 1 |
| 1381 | 9312384 | 406 | 2034 | 39 | 4 | 1 |
| 1382 | 9312385 | 272 | 1482 | 34 | 4 | 2 |
| 1383 | 9312386 | 219 | 1036 | 20 | 4 | 3 |
| 1384 | 9312387 | 322 | 1835 | 44 | 3 | 4 |
| 1385 | 9312388 | 309 | 1727 | 29 | 8 | 3 |
| 1386 | 9312389 | 204 | 1088 | 17 | 3 | 0 |
| 1387 | 9312390 | 292 | 1667 | 24 | 8 | 2 |
| 1388 | 9312391 | 205 | 1137 | 14 | 8 | 2 |
| 1389 | 9312392 | 157 | 827 | 19 | 4 | 1 |
| 1390 | 9312393 | 258 | 1141 | 44 | 4 | 4 |
| 1391 | 9312394 | 215 | 1216 | 13 | 2 | 4 |
| 1392 | 9312395 | 229 | 1209 | 15 | 4 | 4 |
| 1393 | 9312396 | 300 | 1769 | 41 | 2 | 4 |
| 1394 | 9312397 | 254 | 1280 | 36 | 4 | 2 |
| 1395 | 9312398 | 262 | 1462 | 36 | 4 | 4 |
| 1396 | 9312399 | 233 | 1242 | 15 | 3 | 1 |
| 1397 | 9312400 | 282 | 1538 | 36 | 4 | 0 |
| 1398 | 9312401 | 177 | 688 | 10 | 1 | 2 |
| 1399 | 9312402 | 260 | 1609 | 27 | 10 | 6 |
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
| 1424 | 9312427 | 218 | 1318 | 30 | 1 | 5 |
| 1425 | 9312428 | 198 | 924 | 9 | 2 | 2 |
| 1426 | 9312429 | 225 | 1058 | 17 | 1 | 2 |
| 1427 | 9312430 | 246 | 1204 | 31 | 5 | 3 |
| 1428 | 9312431 | 232 | 1292 | 12 | 11 | 3 |
| 1429 | 9312432 | 366 | 1601 | 23 | 8 | 3 |
| 1430 | 9312433 | 202 | 1077 | 16 | 3 | 4 |
| 1431 | 9312434 | 262 | 1259 | 18 | 7 | 3 |
| 1432 | 9312435 | 256 | 1674 | 29 | 3 | 4 |
| 1433 | 9312436 | 442 | 2862 | 23 | 3 | 6 |
| 1434 | 9312437 | 296 | 1576 | 39 | 3 | 5 |
| 1435 | 9312438 | 371 | 2244 | 23 | 7 | 4 |
| 1436 | 9312439 | 233 | 1159 | 26 | 4 | 4 |
| 1437 | 9312440 | 209 | 1215 | 17 | 3 | 2 |
| 1438 | 9312441 | 320 | 2146 | 36 | 2 | 4 |
| 1439 | 9312442 | 237 | 1393 | 23 | 1 | 5 |
| 1440 | 9312443 | 213 | 1053 | 17 | 6 | 2 |
| 1441 | 9312444 | 204 | 1078 | 19 | 3 | 2 |
| 1442 | 9312445 | 500 | 3006 | 38 | 1 | 5 |
| 1443 | 9312446 | 287 | 1645 | 27 | 1 | 4 |
| 1444 | 9312447 | 249 | 1371 | 21 | 5 | 5 |
| 1445 | 9312448 | 197 | 1099 | 16 | 2 | 4 |
| 1446 | 9312449 | 230 | 1308 | 29 | 1 | 4 |
| 1447 | 9312450 | 456 | 2748 | 30 | 8 | 3 |
| 1448 | 9312451 | 349 | 1751 | 44 | 1 | 5 |
| 1449 | 9312452 | 283 | 1552 | 35 | 3 | 3 |
| 1450 | 9312453 | 201 | 887 | 21 | 2 | 1 |
| 1451 | 9312454 | 273 | 1888 | 35 | 3 | 2 |
| 1452 | 9312455 | 284 | 1655 | 34 | 5 | 5 |
| 1453 | 9312456 | 271 | 1546 | 29 | 3 | 2 |
| 1454 | 9312457 | 281 | 1386 | 22 | 0 | 4 |
| 1455 | 9312458 | 200 | 861 | 18 | 3 | 4 |
| 1456 | 9312459 | 248 | 1300 | 28 | 2 | 4 |
| 1457 | 9312460 | 298 | 1676 | 19 | 1 | 3 |
| 1458 | 9312461 | 208 | 1198 | 24 | 3 | 2 |
| 1459 | 9312462 | 177 | 889 | 11 | 1 | 3 |
| 1460 | 9312463 | 275 | 1464 | 44 | 1 | 3 |
| 1461 | 9312464 | 378 | 2052 | 46 | 6 | 4 |
| 1462 | 9312465 | 224 | 1059 | 23 | 4 | 2 |
| 1463 | 9312466 | 291 | 1526 | 39 | 4 | 3 |
| 1464 | 9312467 | 356 | 2354 | 42 | 6 | 4 |
| 1465 | 9312468 | 191 | 797 | 14 | 3 | 3 |
| 1466 | 9312469 | 183 | 911 | 22 | 5 | 3 |
| 1467 | 9312470 | 247 | 1067 | 29 | 8 | 2 |
| 1468 | 9312471 | 206 | 1244 | 23 | 2 | 1 |
| 1469 | 9312472 | 250 | 1337 | 23 | 5 | 1 |
| 1470 | 9312473 | 219 | 1030 | 30 | 3 | 3 |
| 1471 | 9312474 | 438 | 2687 | 48 | 6 | 3 |
| 1472 | 9312475 | 234 | 1469 | 13 | 10 | 4 |
| 1473 | 9312476 | 253 | 1335 | 31 | 1 | 2 |
| 1474 | 9312477 | 303 | 1684 | 42 | 8 | 4 |
| 1475 | 9312478 | 228 | 1212 | 21 | 7 | 3 |
| 1476 | 9312479 | 225 | 1475 | 17 | 2 | 1 |
| 1477 | 9312480 | 245 | 1450 | 23 | 3 | 4 |
| 1478 | 9312481 | 205 | 1318 | 21 | 2 | 1 |
| 1479 | 9312482 | 213 | 1293 | 13 | 4 | 2 |
| 1480 | 9312483 | 194 | 1034 | 15 | 4 | 2 |
| 1481 | 9312484 | 208 | 710 | 13 | 4 | 1 |
| 1482 | 9312485 | 281 | 1565 | 19 | 10 | 3 |
| 1483 | 9312486 | 174 | 869 | 10 | 2 | 2 |
| 1484 | 9312487 | 297 | 1735 | 26 | 5 | 4 |
| 1485 | 9312488 | 272 | 1889 | 32 | 1 | 4 |
| 1486 | 9312489 | 289 | 1396 | 22 | 6 | 1 |
| 1487 | 9312490 | 232 | 1262 | 18 | 2 | 2 |
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
| 1512 | 9312515 | 241 | 1479 | 17 | 5 | 2 |
| 1513 | 9312516 | 223 | 1103 | 24 | 1 | 3 |
| 1514 | 9312517 | 269 | 1430 | 23 | 4 | 2 |
| 1515 | 9312518 | 187 | 960 | 20 | 2 | 2 |
| 1516 | 9312519 | 308 | 1472 | 36 | 2 | 6 |
| 1517 | 9312520 | 173 | 732 | 16 | 6 | 3 |
| 1518 | 9312521 | 258 | 1346 | 29 | 9 | 5 |
| 1519 | 9312522 | 241 | 1031 | 26 | 2 | 3 |
| 1520 | 9312523 | 320 | 1690 | 43 | 7 | 4 |
| 1521 | 9312524 | 250 | 1045 | 23 | 1 | 4 |
| 1522 | 9312525 | 226 | 1141 | 17 | 5 | 3 |
| 1523 | 9312526 | 280 | 1875 | 18 | 7 | 3 |
| 1524 | 9312527 | 226 | 1285 | 24 | 0 | 3 |
| 1525 | 9312528 | 268 | 1355 | 26 | 3 | 4 |
| 1526 | 9312529 | 245 | 1339 | 19 | 5 | 4 |
| 1527 | 9312530 | 299 | 1066 | 26 | 4 | 5 |
| 1528 | 9312531 | 270 | 1621 | 22 | 1 | 3 |
| 1529 | 9312532 | 265 | 1829 | 38 | 5 | 4 |
| 1530 | 9312533 | 186 | 778 | 19 | 4 | 4 |
| 1531 | 9312534 | 246 | 1097 | 16 | 4 | 5 |
| 1532 | 9312535 | 183 | 799 | 17 | 2 | 5 |
| 1533 | 9312536 | 255 | 1462 | 29 | 7 | 4 |
| 1534 | 9312537 | 228 | 1194 | 26 | 2 | 5 |
| 1535 | 9312538 | 261 | 1137 | 27 | 3 | 4 |
| 1536 | 9312539 | 309 | 1636 | 45 | 6 | 3 |
| 1537 | 9312540 | 220 | 796 | 21 | 1 | 2 |
| 1538 | 9312541 | 206 | 1243 | 22 | 0 | 2 |
| 1539 | 9312542 | 216 | 1007 | 22 | 4 | 5 |
| 1540 | 9312543 | 443 | 2833 | 39 | 1 | 2 |
| 1541 | 9312544 | 340 | 1854 | 33 | 3 | 2 |
| 1542 | 9312545 | 237 | 1323 | 26 | 0 | 2 |
| 1543 | 9312546 | 297 | 1679 | 22 | 1 | 4 |
| 1544 | 9312547 | 279 | 1557 | 30 | 4 | 3 |
| 1545 | 9312548 | 222 | 1201 | 21 | 4 | 3 |
| 1546 | 9312549 | 229 | 1140 | 22 | 1 | 3 |
| 1547 | 9312550 | 225 | 1146 | 27 | 1 | 2 |
| 1548 | 9312551 | 197 | 1214 | 24 | 5 | 1 |
| 1549 | 9312552 | 209 | 1052 | 26 | 2 | 3 |
| 1550 | 9312553 | 244 | 1429 | 19 | 1 | 0 |
| 1551 | 9312554 | 226 | 1054 | 24 | 3 | 1 |
| 1552 | 9312555 | 247 | 1810 | 26 | 2 | 2 |
| 1553 | 9312556 | 284 | 1765 | 40 | 3 | 3 |
| 1554 | 9312557 | 277 | 1496 | 21 | 1 | 2 |
| 1555 | 9312558 | 263 | 1623 | 36 | 4 | 0 |
| 1556 | 9312559 | 184 | 633 | 13 | 2 | 2 |
| 1557 | 9312560 | 237 | 800 | 14 | 3 | 3 |
| 1558 | 9312561 | 214 | 1076 | 21 | 2 | 2 |
| 1559 | 9312562 | 190 | 704 | 18 | 2 | 2 |
| 1560 | 9312563 | 189 | 789 | 14 | 5 | 3 |
| 1561 | 9312564 | 266 | 1303 | 32 | 3 | 2 |
| 1562 | 9312565 | 185 | 899 | 14 | 3 | 0 |
| 1563 | 9312566 | 274 | 1765 | 20 | 12 | 3 |
| 1564 | 9312567 | 223 | 1187 | 17 | 4 | 2 |
| 1565 | 9312568 | 231 | 1530 | 19 | 3 | 3 |
| 1566 | 9312569 | 238 | 1148 | 22 | 4 | 4 |
| 1567 | 9312570 | 181 | 1132 | 12 | 6 | 2 |
| 1568 | 9312571 | 240 | 1458 | 22 | 5 | 1 |
| 1569 | 9312572 | 199 | 992 | 18 | 5 | 2 |
| 1570 | 9312573 | 187 | 1065 | 11 | 3 | 4 |
| 1571 | 9312574 | 274 | 1931 | 25 | 4 | 1 |
| 1572 | 9312575 | 250 | 1256 | 25 | 8 | 1 |
| 1573 | 9312576 | 213 | 1108 | 20 | 3 | 2 |
| 1574 | 9312577 | 180 | 1112 | 19 | 2 | 1 |
| 1575 | 9312578 | 191 | 690 | 10 | 5 | 1 |
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
| 1600 | 9312603 | 252 | 1351 | 33 | 3 | 4 |
| 1601 | 9312604 | 238 | 1189 | 25 | 4 | 4 |
| 1602 | 9312605 | 272 | 1817 | 26 | 7 | 2 |
| 1603 | 9312606 | 287 | 1822 | 35 | 5 | 3 |
| 1604 | 9312607 | 230 | 1174 | 14 | 3 | 2 |
| 1605 | 9312608 | 202 | 971 | 26 | 3 | 4 |
| 1606 | 9312609 | 189 | 704 | 20 | 1 | 4 |
| 1607 | 9312610 | 253 | 1573 | 28 | 3 | 2 |
| 1608 | 9312611 | 211 | 1231 | 22 | 1 | 3 |
| 1609 | 9312612 | 304 | 1606 | 35 | 1 | 5 |
| 1610 | 9312613 | 279 | 1187 | 20 | 7 | 4 |
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
| 1623 | 9312626 | 272 | 1580 | 32 | 3 | 4 |
| 1624 | 9312627 | 251 | 1250 | 26 | 2 | 2 |
| 1625 | 9312628 | 201 | 970 | 22 | 1 | 4 |
| 1626 | 9312629 | 258 | 1603 | 43 | 3 | 2 |
| 1627 | 9312630 | 239 | 1079 | 28 | 1 | 3 |
| 1628 | 9312631 | 431 | 2263 | 42 | 0 | 5 |
| 1629 | 9312632 | 268 | 1505 | 29 | 2 | 4 |
| 1630 | 9312633 | 244 | 1559 | 32 | 1 | 1 |
| 1631 | 9312634 | 176 | 831 | 15 | 2 | 3 |
| 1632 | 9312635 | 285 | 1155 | 33 | 2 | 3 |
| 1633 | 9312636 | 187 | 1062 | 17 | 0 | 2 |
| 1634 | 9312637 | 211 | 951 | 14 | 3 | 4 |
| 1635 | 9312638 | 227 | 1180 | 29 | 1 | 2 |
| 1636 | 9312639 | 276 | 1453 | 24 | 5 | 1 |
| 1637 | 9312640 | 215 | 904 | 17 | 2 | 3 |
| 1638 | 9312641 | 216 | 1415 | 25 | 1 | 2 |
| 1639 | 9312642 | 232 | 887 | 25 | 0 | 3 |
| 1640 | 9312643 | 282 | 1356 | 27 | 3 | 3 |
| 1641 | 9312644 | 180 | 835 | 12 | 4 | 1 |
| 1642 | 9312645 | 198 | 701 | 20 | 4 | 2 |
| 1643 | 9312646 | 288 | 1578 | 33 | 4 | 2 |
| 1644 | 9312647 | 209 | 1229 | 15 | 1 | 1 |
| 1645 | 9312648 | 212 | 1075 | 27 | 3 | 2 |
| 1646 | 9312649 | 223 | 1358 | 23 | 7 | 1 |
| 1647 | 9312650 | 225 | 1203 | 18 | 0 | 2 |
| 1648 | 9312651 | 283 | 1772 | 33 | 2 | 2 |
| 1649 | 9312652 | 213 | 1409 | 15 | 3 | 2 |
| 1650 | 9312653 | 238 | 1280 | 25 | 3 | 3 |
| 1651 | 9312654 | 302 | 1910 | 22 | 3 | 0 |
| 1652 | 9312655 | 212 | 1142 | 28 | 6 | 3 |
| 1653 | 9312656 | 228 | 1292 | 23 | 2 | 2 |
| 1654 | 9312657 | 204 | 1123 | 31 | 2 | 2 |
| 1655 | 9312658 | 188 | 863 | 13 | 3 | 2 |
| 1656 | 9312659 | 411 | 2643 | 57 | 3 | 5 |
| 1657 | 9312660 | 235 | 1628 | 20 | 4 | 5 |
| 1658 | 9312661 | 228 | 944 | 16 | 4 | 1 |
| 1659 | 9312662 | 325 | 2182 | 43 | 3 | 4 |
| 1660 | 9312663 | 226 | 1081 | 21 | 7 | 3 |
| 1661 | 9312664 | 294 | 1966 | 28 | 5 | 4 |
| 1662 | 9312665 | 263 | 1270 | 37 | 3 | 2 |
| 1663 | 9312666 | 448 | 2029 | 44 | 3 | 5 |
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
| 1688 | 9312691 | 231 | 1141 | 14 | 7 | 2 |
| 1689 | 9312692 | 263 | 1376 | 25 | 6 | 2 |
| 1690 | 9312693 | 320 | 1809 | 36 | 7 | 3 |
| 1691 | 9312694 | 154 | 657 | 10 | 3 | 1 |
| 1692 | 9312695 | 249 | 1488 | 27 | 3 | 1 |
| 1693 | 9312696 | 256 | 1047 | 23 | 1 | 2 |
| 1694 | 9312697 | 195 | 889 | 23 | 4 | 2 |
| 1695 | 9312698 | 265 | 959 | 32 | 3 | 3 |
| 1696 | 9312699 | 178 | 1038 | 14 | 0 | 2 |
| 1697 | 9312700 | 227 | 1009 | 23 | 1 | 5 |
| 1698 | 9312701 | 195 | 1262 | 31 | 2 | 2 |
| 1699 | 9312702 | 197 | 968 | 19 | 0 | 1 |
| 1700 | 9312703 | 250 | 1617 | 21 | 3 | 4 |
| 1701 | 9312704 | 313 | 1759 | 35 | 7 | 4 |
| 1702 | 9312705 | 337 | 1824 | 43 | 3 | 2 |
| 1703 | 9312706 | 183 | 845 | 18 | 2 | 4 |
| 1704 | 9312707 | 187 | 861 | 21 | 2 | 3 |
| 1705 | 9312708 | 301 | 1750 | 31 | 3 | 1 |
| 1706 | 9312709 | 179 | 800 | 13 | 0 | 4 |
| 1707 | 9312710 | 213 | 1035 | 25 | 7 | 4 |
| 1708 | 9312711 | 243 | 1277 | 21 | 6 | 5 |
| 1709 | 9312712 | 310 | 1833 | 41 | 5 | 4 |
| 1710 | 9312713 | 291 | 1470 | 33 | 1 | 4 |
| 1711 | 9312714 | 363 | 1788 | 24 | 5 | 6 |
| 1712 | 9312715 | 292 | 1485 | 25 | 0 | 5 |
| 1713 | 9312716 | 266 | 1387 | 18 | 1 | 3 |
| 1714 | 9312717 | 304 | 1684 | 28 | 2 | 2 |
| 1715 | 9312718 | 374 | 2071 | 50 | 3 | 4 |
| 1716 | 9312719 | 308 | 1950 | 31 | 1 | 3 |
| 1717 | 9312720 | 298 | 1408 | 28 | 3 | 3 |
| 1718 | 9312721 | 254 | 1317 | 35 | 3 | 4 |
| 1719 | 9312722 | 283 | 1678 | 29 | 3 | 1 |
| 1720 | 9312723 | 264 | 1120 | 27 | 3 | 4 |
| 1721 | 9312724 | 259 | 1368 | 27 | 1 | 4 |
| 1722 | 9312725 | 226 | 1184 | 28 | 1 | 3 |
| 1723 | 9312726 | 269 | 1229 | 29 | 2 | 4 |
| 1724 | 9312727 | 196 | 808 | 21 | 1 | 3 |
| 1725 | 9312728 | 248 | 1329 | 37 | 0 | 4 |
| 1726 | 9312729 | 500 | 3565 | 77 | 2 | 4 |
| 1727 | 9312730 | 232 | 1313 | 43 | 1 | 3 |
| 1728 | 9312731 | 204 | 1067 | 26 | 7 | 2 |
| 1729 | 9312732 | 298 | 1528 | 23 | 5 | 1 |
| 1730 | 9312733 | 202 | 744 | 20 | 4 | 1 |
| 1731 | 9312734 | 349 | 2048 | 41 | 4 | 3 |
| 1732 | 9312735 | 189 | 907 | 19 | 2 | 3 |
| 1733 | 9312736 | 170 | 611 | 12 | 6 | 2 |
| 1734 | 9312737 | 187 | 990 | 16 | 2 | 1 |
| 1735 | 9312738 | 265 | 1525 | 39 | 4 | 2 |
| 1736 | 9312739 | 305 | 1811 | 37 | 3 | 2 |
| 1737 | 9312740 | 250 | 1654 | 34 | 1 | 2 |
| 1738 | 9312741 | 264 | 1416 | 38 | 0 | 2 |
| 1739 | 9312742 | 297 | 1437 | 47 | 6 | 4 |
| 1740 | 9312743 | 267 | 1610 | 31 | 2 | 2 |
| 1741 | 9312744 | 243 | 1276 | 26 | 4 | 3 |
| 1742 | 9312745 | 251 | 1257 | 31 | 4 | 4 |
| 1743 | 9312746 | 246 | 1295 | 25 | 7 | 4 |
| 1744 | 9312747 | 340 | 2114 | 44 | 4 | 2 |
| 1745 | 9312748 | 227 | 1103 | 23 | 2 | 3 |
| 1746 | 9312749 | 146 | 582 | 9 | 3 | 1 |
| 1747 | 9312750 | 288 | 1757 | 25 | 6 | 2 |
| 1748 | 9312751 | 412 | 2230 | 53 | 5 | 5 |
| 1749 | 9312752 | 218 | 826 | 21 | 3 | 5 |
| 1750 | 9312753 | 208 | 922 | 17 | 3 | 0 |
| 1751 | 9312754 | 169 | 750 | 22 | 2 | 2 |
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
| 1776 | 9312779 | 197 | 941 | 28 | 3 | 3 |
| 1777 | 9312780 | 302 | 1622 | 36 | 2 | 3 |
| 1778 | 9312781 | 296 | 1675 | 22 | 8 | 2 |
| 1779 | 9312782 | 373 | 2274 | 42 | 8 | 3 |
| 1780 | 9312783 | 256 | 1357 | 35 | 10 | 4 |
| 1781 | 9312784 | 320 | 1883 | 49 | 5 | 4 |
| 1782 | 9312785 | 356 | 2145 | 35 | 3 | 2 |
| 1783 | 9312786 | 244 | 1276 | 25 | 3 | 3 |
| 1784 | 9312787 | 291 | 1662 | 22 | 3 | 3 |
| 1785 | 9312788 | 168 | 886 | 12 | 2 | 1 |
| 1786 | 9312789 | 205 | 927 | 13 | 2 | 4 |
| 1787 | 9312790 | 251 | 1098 | 33 | 0 | 3 |
| 1788 | 9312791 | 238 | 1264 | 16 | 1 | 4 |
| 1789 | 9312792 | 240 | 1488 | 30 | 2 | 3 |
| 1790 | 9312793 | 178 | 926 | 13 | 2 | 2 |
| 1791 | 9312794 | 224 | 1075 | 19 | 3 | 5 |
| 1792 | 9312795 | 304 | 1753 | 35 | 7 | 4 |
| 1793 | 9312796 | 220 | 1124 | 20 | 5 | 4 |
| 1794 | 9312797 | 223 | 1026 | 23 | 3 | 3 |
| 1795 | 9312798 | 294 | 1941 | 26 | 1 | 4 |
| 1796 | 9312799 | 319 | 2237 | 30 | 0 | 3 |
| 1797 | 9312800 | 232 | 1174 | 22 | 2 | 3 |
| 1798 | 9312801 | 377 | 2909 | 30 | 2 | 4 |
| 1799 | 9312802 | 221 | 1144 | 22 | 4 | 2 |
| 1800 | 9312803 | 202 | 882 | 14 | 0 | 3 |
| 1801 | 9312804 | 374 | 2311 | 24 | 0 | 1 |
| 1802 | 9312805 | 215 | 1086 | 23 | 4 | 4 |
| 1803 | 9312806 | 247 | 1082 | 20 | 2 | 2 |
| 1804 | 9312807 | 285 | 1494 | 28 | 5 | 3 |
| 1805 | 9312808 | 250 | 1032 | 25 | 5 | 5 |
| 1806 | 9312809 | 253 | 1541 | 30 | 3 | 3 |
| 1807 | 9312810 | 274 | 1228 | 19 | 3 | 5 |
| 1808 | 9312811 | 326 | 1199 | 40 | 2 | 4 |
| 1809 | 9312812 | 283 | 1378 | 20 | 2 | 2 |
| 1810 | 9312813 | 205 | 1061 | 15 | 0 | 2 |
| 1811 | 9312814 | 261 | 1256 | 25 | 6 | 3 |
| 1812 | 9312815 | 205 | 1060 | 21 | 6 | 2 |
| 1813 | 9312816 | 294 | 1430 | 35 | 3 | 4 |
| 1814 | 9312817 | 204 | 960 | 24 | 4 | 2 |
| 1815 | 9312818 | 224 | 983 | 19 | 4 | 1 |
| 1816 | 9312819 | 206 | 1144 | 19 | 14 | 1 |
| 1817 | 9312820 | 255 | 1309 | 27 | 6 | 2 |
| 1818 | 9312821 | 230 | 1125 | 35 | 4 | 2 |
| 1819 | 9312822 | 190 | 702 | 7 | 3 | 2 |
| 1820 | 9312823 | 204 | 1008 | 31 | 7 | 2 |
| 1821 | 9312824 | 287 | 1925 | 33 | 8 | 3 |
| 1822 | 9312825 | 256 | 1496 | 34 | 5 | 4 |
| 1823 | 9312826 | 233 | 1086 | 37 | 7 | 3 |
| 1824 | 9312827 | 230 | 1194 | 20 | 11 | 1 |
| 1825 | 9312828 | 234 | 1427 | 28 | 3 | 0 |
| 1826 | 9312829 | 304 | 2032 | 30 | 4 | 2 |
| 1827 | 9312830 | 206 | 860 | 17 | 4 | 1 |
| 1828 | 9312831 | 220 | 1239 | 21 | 0 | 3 |
| 1829 | 9312832 | 244 | 1056 | 22 | 5 | 5 |
| 1830 | 9312833 | 182 | 989 | 24 | 2 | 1 |
| 1831 | 9312834 | 206 | 1047 | 20 | 4 | 2 |
| 1832 | 9312835 | 251 | 1442 | 20 | 3 | 2 |
| 1833 | 9312836 | 271 | 1516 | 34 | 2 | 2 |
| 1834 | 9312837 | 224 | 1292 | 27 | 1 | 1 |
| 1835 | 9312838 | 232 | 1047 | 19 | 1 | 3 |
| 1836 | 9312839 | 239 | 1176 | 26 | 4 | 3 |
| 1837 | 9312840 | 244 | 1336 | 34 | 2 | 2 |
| 1838 | 9312841 | 237 | 1178 | 31 | 5 | 3 |
| 1839 | 9312842 | 196 | 857 | 18 | 3 | 1 |
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
| 1864 | 9312867 | 271 | 1342 | 17 | 2 | 2 |
| 1865 | 9312868 | 255 | 1665 | 27 | 4 | 1 |
| 1866 | 9312869 | 264 | 1248 | 30 | 7 | 2 |
| 1867 | 9312870 | 265 | 1193 | 23 | 3 | 3 |
| 1868 | 9312871 | 217 | 1027 | 20 | 4 | 3 |
| 1869 | 9312872 | 278 | 1471 | 42 | 6 | 3 |
| 1870 | 9312873 | 254 | 1389 | 41 | 3 | 3 |
| 1871 | 9312874 | 154 | 802 | 13 | 6 | 2 |
| 1872 | 9312875 | 206 | 1127 | 33 | 2 | 4 |
| 1873 | 9312876 | 347 | 1978 | 48 | 4 | 3 |
| 1874 | 9312877 | 255 | 1724 | 27 | 1 | 4 |
| 1875 | 9312878 | 221 | 1146 | 21 | 3 | 4 |
| 1876 | 9312879 | 244 | 1450 | 22 | 4 | 2 |
| 1877 | 9312880 | 249 | 1381 | 29 | 3 | 3 |
| 1878 | 9312881 | 232 | 1302 | 19 | 0 | 3 |
| 1879 | 9312882 | 236 | 1048 | 17 | 1 | 3 |
| 1880 | 9312883 | 241 | 1235 | 24 | 1 | 4 |
| 1881 | 9312884 | 412 | 2477 | 30 | 5 | 4 |
| 1882 | 9312885 | 293 | 1746 | 30 | 4 | 3 |
| 1883 | 9312886 | 253 | 1273 | 26 | 1 | 2 |
| 1884 | 9312887 | 298 | 1910 | 21 | 2 | 5 |
| 1885 | 9312888 | 220 | 1178 | 13 | 0 | 2 |
| 1886 | 9312889 | 295 | 1339 | 41 | 0 | 5 |
| 1887 | 9312890 | 319 | 1667 | 32 | 7 | 3 |
| 1888 | 9312891 | 229 | 1057 | 25 | 4 | 4 |
| 1889 | 9312892 | 217 | 1257 | 20 | 0 | 3 |
| 1890 | 9312893 | 304 | 1654 | 42 | 0 | 5 |
| 1891 | 9312894 | 282 | 1562 | 34 | 2 | 4 |
| 1892 | 9312895 | 268 | 1495 | 20 | 5 | 3 |
| 1893 | 9312896 | 266 | 1700 | 19 | 1 | 1 |
| 1894 | 9312897 | 206 | 1232 | 22 | 3 | 2 |
| 1895 | 9312898 | 230 | 1254 | 22 | 3 | 5 |
| 1896 | 9312899 | 269 | 1358 | 30 | 3 | 2 |
| 1897 | 9312900 | 273 | 1347 | 29 | 6 | 2 |
| 1898 | 9312901 | 237 | 1275 | 27 | 3 | 4 |
| 1899 | 9312902 | 190 | 669 | 18 | 3 | 2 |
| 1900 | 9312903 | 222 | 1359 | 14 | 2 | 3 |
| 1901 | 9312904 | 316 | 1751 | 43 | 7 | 2 |
| 1902 | 9312905 | 298 | 1585 | 37 | 3 | 4 |
| 1903 | 9312906 | 228 | 1357 | 20 | 4 | 0 |
| 1904 | 9312907 | 224 | 1197 | 20 | 4 | 4 |
| 1905 | 9312908 | 267 | 1504 | 38 | 2 | 3 |
| 1906 | 9312909 | 246 | 1385 | 9 | 5 | 2 |
| 1907 | 9312910 | 229 | 1309 | 29 | 3 | 2 |
| 1908 | 9312911 | 299 | 2039 | 38 | 2 | 3 |
| 1909 | 9312912 | 276 | 1643 | 33 | 1 | 3 |
| 1910 | 9312913 | 207 | 1216 | 27 | 4 | 1 |
| 1911 | 9312914 | 214 | 986 | 25 | 8 | 4 |
| 1912 | 9312915 | 210 | 894 | 28 | 3 | 4 |
| 1913 | 9312916 | 229 | 1336 | 9 | 5 | 2 |
| 1914 | 9312917 | 270 | 1400 | 36 | 5 | 5 |
| 1915 | 9312918 | 265 | 1555 | 32 | 5 | 2 |
| 1916 | 9312919 | 261 | 1371 | 34 | 6 | 4 |
| 1917 | 9312920 | 274 | 1601 | 33 | 11 | 3 |
| 1918 | 9312921 | 279 | 1661 | 20 | 2 | 2 |
| 1919 | 9312922 | 171 | 850 | 17 | 5 | 3 |
| 1920 | 9312923 | 243 | 1501 | 17 | 6 | 0 |
| 1921 | 9312924 | 275 | 1619 | 28 | 1 | 3 |
| 1922 | 9312925 | 295 | 1743 | 23 | 9 | 1 |
| 1923 | 9312926 | 252 | 1557 | 29 | 5 | 4 |
| 1924 | 9312927 | 236 | 1464 | 31 | 5 | 3 |
| 1925 | 9312928 | 253 | 1294 | 21 | 2 | 2 |
| 1926 | 9312929 | 219 | 1508 | 22 | 9 | 2 |
| 1927 | 9312930 | 178 | 850 | 12 | 7 | 0 |
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
| 1952 | 9312955 | 219 | 1187 | 26 | 7 | 2 |
| 1953 | 9312956 | 236 | 1366 | 27 | 1 | 2 |
| 1954 | 9312957 | 189 | 651 | 12 | 1 | 2 |
| 1955 | 9312958 | 223 | 962 | 24 | 0 | 2 |
| 1956 | 9312959 | 232 | 1095 | 23 | 0 | 2 |
| 1957 | 9312960 | 219 | 1142 | 22 | 4 | 3 |
| 1958 | 9312961 | 284 | 1580 | 24 | 4 | 2 |
| 1959 | 9312962 | 317 | 2116 | 41 | 4 | 4 |
| 1960 | 9312963 | 231 | 926 | 27 | 1 | 4 |
| 1961 | 9312964 | 260 | 1523 | 37 | 0 | 3 |
| 1962 | 9312965 | 215 | 1047 | 16 | 10 | 1 |
| 1963 | 9312966 | 251 | 970 | 25 | 1 | 4 |
| 1964 | 9312967 | 211 | 1098 | 34 | 0 | 2 |
| 1965 | 9312968 | 186 | 950 | 13 | 1 | 2 |
| 1966 | 9312969 | 191 | 921 | 21 | 4 | 3 |
| 1967 | 9312970 | 276 | 1492 | 39 | 1 | 1 |
| 1968 | 9312971 | 205 | 976 | 14 | 3 | 3 |
| 1969 | 9312972 | 299 | 1646 | 30 | 4 | 5 |
| 1970 | 9312973 | 260 | 1364 | 24 | 1 | 3 |
| 1971 | 9312974 | 465 | 3180 | 51 | 1 | 3 |
| 1972 | 9312975 | 185 | 874 | 26 | 3 | 2 |
| 1973 | 9312976 | 299 | 1938 | 33 | 1 | 4 |
| 1974 | 9312977 | 228 | 1320 | 13 | 2 | 3 |
| 1975 | 9312978 | 267 | 1326 | 30 | 5 | 6 |
| 1976 | 9312979 | 203 | 1089 | 20 | 1 | 3 |
| 1977 | 9312980 | 399 | 2093 | 51 | 1 | 3 |
| 1978 | 9312981 | 244 | 1386 | 24 | 2 | 3 |
| 1979 | 9312982 | 227 | 1238 | 24 | 4 | 4 |
| 1980 | 9312983 | 333 | 1931 | 47 | 4 | 5 |
| 1981 | 9312984 | 278 | 1362 | 39 | 1 | 3 |
| 1982 | 9312985 | 279 | 1720 | 26 | 1 | 4 |
| 1983 | 9312986 | 223 | 1262 | 32 | 4 | 4 |
| 1984 | 9312987 | 204 | 729 | 17 | 7 | 4 |
| 1985 | 9312988 | 220 | 1150 | 27 | 3 | 4 |
| 1986 | 9312989 | 229 | 835 | 19 | 0 | 4 |
| 1987 | 9312990 | 191 | 811 | 18 | 3 | 1 |
| 1988 | 9312991 | 194 | 967 | 17 | 5 | 2 |
| 1989 | 9312992 | 297 | 1691 | 36 | 3 | 5 |
| 1990 | 9312993 | 262 | 1170 | 28 | 6 | 4 |
| 1991 | 9312994 | 269 | 1474 | 16 | 2 | 0 |
| 1992 | 9312995 | 282 | 1633 | 39 | 4 | 4 |
| 1993 | 9312996 | 196 | 857 | 26 | 9 | 2 |
| 1994 | 9312997 | 250 | 915 | 26 | 5 | 2 |
| 1995 | 9312998 | 271 | 1419 | 35 | 2 | 1 |
| 1996 | 9312999 | 207 | 1015 | 21 | 4 | 3 |
| 1997 | 9313000 | 233 | 1445 | 21 | 2 | 3 |
| 1998 | 9313001 | 265 | 1244 | 32 | 2 | 3 |
| 1999 | 9313002 | 252 | 1511 | 28 | 6 | 2 |
| 2000 | 9313003 | 280 | 1257 | 34 | 2 | 4 |
| 2001 | 9313004 | 190 | 859 | 15 | 2 | 2 |
| 2002 | 9313005 | 242 | 1062 | 15 | 7 | 2 |
| 2003 | 9313006 | 225 | 1325 | 19 | 5 | 1 |
| 2004 | 9313007 | 212 | 1094 | 15 | 11 | 3 |
| 2005 | 9313008 | 173 | 677 | 15 | 1 | 2 |
| 2006 | 9313009 | 176 | 880 | 19 | 1 | 0 |
| 2007 | 9313010 | 213 | 1010 | 16 | 9 | 3 |
| 2008 | 9313011 | 264 | 1488 | 21 | 6 | 3 |
| 2009 | 9313012 | 201 | 1359 | 19 | 1 | 2 |
| 2010 | 9313013 | 187 | 1100 | 15 | 7 | 0 |
| 2011 | 9313014 | 371 | 1918 | 30 | 2 | 4 |
| 2012 | 9313015 | 257 | 1263 | 36 | 3 | 3 |
| 2013 | 9313016 | 211 | 1288 | 16 | 1 | 4 |
| 2014 | 9313017 | 251 | 1315 | 25 | 3 | 3 |
| 2015 | 9313018 | 244 | 1587 | 30 | 4 | 0 |
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
| 2040 | 9313043 | 241 | 1310 | 25 | 2 | 3 |
| 2041 | 9313044 | 221 | 1142 | 22 | 3 | 3 |
| 2042 | 9313045 | 218 | 896 | 15 | 4 | 2 |
| 2043 | 9313046 | 238 | 1260 | 23 | 5 | 3 |
| 2044 | 9313047 | 304 | 1811 | 30 | 3 | 4 |
| 2045 | 9313048 | 199 | 924 | 16 | 6 | 3 |
| 2046 | 9313049 | 190 | 743 | 16 | 6 | 2 |
| 2047 | 9313050 | 186 | 1026 | 14 | 1 | 2 |
| 2048 | 9313051 | 292 | 1536 | 28 | 6 | 3 |
| 2049 | 9313052 | 239 | 1320 | 21 | 3 | 4 |
| 2050 | 9313053 | 291 | 1808 | 24 | 10 | 2 |
| 2051 | 9313054 | 250 | 1469 | 22 | 3 | 3 |
| 2052 | 9313055 | 270 | 1394 | 31 | 3 | 4 |
| 2053 | 9313056 | 366 | 1688 | 34 | 2 | 3 |
| 2054 | 9313057 | 296 | 1800 | 25 | 3 | 4 |
| 2055 | 9313058 | 281 | 1963 | 32 | 3 | 3 |
| 2056 | 9313059 | 195 | 1017 | 20 | 0 | 4 |
| 2057 | 9313060 | 384 | 2378 | 41 | 6 | 3 |
| 2058 | 9313061 | 212 | 1074 | 22 | 3 | 2 |
| 2059 | 9313062 | 215 | 1228 | 24 | 1 | 4 |
| 2060 | 9313063 | 256 | 1136 | 31 | 1 | 4 |
| 2061 | 9313064 | 200 | 959 | 11 | 2 | 3 |
| 2062 | 9313065 | 225 | 988 | 13 | 1 | 3 |
| 2063 | 9313066 | 228 | 1241 | 25 | 1 | 1 |
| 2064 | 9313067 | 284 | 1327 | 39 | 3 | 2 |
| 2065 | 9313068 | 202 | 639 | 22 | 4 | 5 |
| 2066 | 9313069 | 259 | 1281 | 38 | 0 | 4 |
| 2067 | 9313070 | 227 | 1187 | 25 | 2 | 4 |
| 2068 | 9313071 | 210 | 1064 | 20 | 2 | 2 |
| 2069 | 9313072 | 239 | 1381 | 32 | 4 | 2 |
| 2070 | 9313073 | 245 | 1416 | 31 | 1 | 4 |
| 2071 | 9313074 | 234 | 873 | 24 | 2 | 3 |
| 2072 | 9313075 | 224 | 1151 | 15 | 1 | 2 |
| 2073 | 9313076 | 200 | 1144 | 20 | 8 | 3 |
| 2074 | 9313077 | 259 | 1300 | 32 | 3 | 4 |
| 2075 | 9313078 | 219 | 1347 | 20 | 6 | 3 |
| 2076 | 9313079 | 275 | 1410 | 35 | 3 | 4 |
| 2077 | 9313080 | 268 | 1427 | 34 | 4 | 3 |
| 2078 | 9313081 | 233 | 1295 | 31 | 2 | 5 |
| 2079 | 9313082 | 186 | 983 | 22 | 6 | 4 |
| 2080 | 9313083 | 267 | 1508 | 47 | 8 | 2 |
| 2081 | 9313084 | 221 | 1079 | 19 | 2 | 1 |
| 2082 | 9313085 | 171 | 927 | 17 | 6 | 1 |
| 2083 | 9313086 | 210 | 1227 | 26 | 7 | 2 |
| 2084 | 9313087 | 183 | 845 | 16 | 5 | 3 |
| 2085 | 9313088 | 196 | 1006 | 25 | 3 | 3 |
| 2086 | 9313089 | 195 | 1123 | 18 | 5 | 3 |
| 2087 | 9313090 | 191 | 688 | 23 | 6 | 1 |
| 2088 | 9313091 | 232 | 1356 | 28 | 2 | 3 |
| 2089 | 9313092 | 343 | 1848 | 26 | 2 | 2 |
| 2090 | 9313093 | 240 | 1651 | 20 | 4 | 3 |
| 2091 | 9313094 | 236 | 1393 | 24 | 2 | 2 |
| 2092 | 9313095 | 204 | 1093 | 15 | 1 | 3 |
| 2093 | 9313096 | 207 | 1183 | 12 | 5 | 3 |
| 2094 | 9313097 | 219 | 935 | 13 | 3 | 3 |
| 2095 | 9313098 | 235 | 1276 | 23 | 1 | 2 |
| 2096 | 9313099 | 283 | 1853 | 36 | 5 | 4 |
| 2097 | 9313100 | 200 | 1056 | 15 | 5 | 0 |
| 2098 | 9313101 | 226 | 1253 | 31 | 1 | 2 |
| 2099 | 9313102 | 199 | 1252 | 18 | 4 | 1 |
| 2100 | 9313103 | 275 | 1663 | 32 | 5 | 2 |
| 2101 | 9313104 | 226 | 1109 | 21 | 1 | 2 |
| 2102 | 9313105 | 389 | 2236 | 34 | 4 | 2 |
| 2103 | 9313106 | 283 | 1197 | 28 | 0 | 3 |
| 2104 | 9313107 | 234 | 971 | 22 | 1 | 4 |
| 2105 | 9313108 | 252 | 1314 | 30 | 1 | 2 |
| 2106 | 9313109 | 251 | 947 | 30 | 6 | 3 |
| 2107 | 9313110 | 221 | 1440 | 28 | 6 | 3 |
| 2108 | 9313111 | 292 | 1049 | 32 | 9 | 4 |
| 2109 | 9313112 | 216 | 1015 | 19 | 0 | 4 |
| 2110 | 9313113 | 316 | 1863 | 35 | 6 | 4 |
| 2111 | 9313114 | 212 | 966 | 21 | 4 | 2 |

## Throughput

- Turns simulated: 528592 (first play of each game)
- Aggregate: 280.3 turns/s over 1885.7s wall
- Per game thread: 35.2 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

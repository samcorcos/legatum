# Simulation: india-remit-off-0c5d2e8 (2026-11-26)

1000 games, first seed 20261126, brain `scripted`, turn limit off, safety cap 1200, 4 threads, started 2026-11-26T23:09:33Z. Wall time 2076.7s.

Source checkout at run start: `54a884b8c0281c8ccef5739dba3d3034d542ddb2`; dirty: yes. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--games","1000","--seed","20261126","--threads","4","--difficulty","hard","--rule-off","india_remittance","--label","india-remit-off-0c5d2e8"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 1000 of 1000 |
| Game length, turns (mean / median / min-max) | 251.9 / 243 / 160-500 |
| Win reason mix | VP hold 95.5%, Capitals 4.1%, Turn limit 0.4% |
| Median end turn | 243 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 4.1% / 95.5% / 0.4% / 0.0% |
| Games reaching the World War era (any empire) | 996 of 1000 (99.6%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1493 / 466 / 958 / 69 |
| Hold break rate | 31.2% of started; 32.7% of resolved (broken / (broken + completed)) |
| Hold wins | 955 |
| First reach to win, turns (mean / median / min-max) | 13.2 / 9 / 9-281 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 151 |
| Wins by a team another team beat to its target | 74 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 955 | 247.8 / 242 / 160-476 |
| Capitals | 41 | 323.1 / 305 / 210-500 |
| Turn limit | 4 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 111 | 11.1% |
| India | 43 | 4.3% |
| Byzantium | 92 | 9.2% |
| Germany | 138 | 13.8% |
| Celts | 156 | 15.6% |
| Mongols | 82 | 8.2% |
| Sparta | 132 | 13.2% |
| Athens | 136 | 13.6% |
| Aztecs | 110 | 11.0% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 955 | 95.5% |
| Conquest | 0 | 0.0% |
| Capitals | 41 | 4.1% |
| Turn limit | 4 | 0.4% |
| Draw | 0 | 0.0% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 899 | 111 | 12.3% |
| India | 895 | 43 | 4.8% |
| Byzantium | 878 | 92 | 10.5% |
| Germany | 880 | 138 | 15.7% |
| Celts | 896 | 156 | 17.4% |
| Mongols | 877 | 82 | 9.4% |
| Sparta | 891 | 132 | 14.8% |
| Athens | 895 | 136 | 15.2% |
| Aztecs | 889 | 110 | 12.4% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 8000 | 1000 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 899 | 111 | 12.3% |
| India | Hard | 895 | 43 | 4.8% |
| Byzantium | Hard | 878 | 92 | 10.5% |
| Germany | Hard | 880 | 138 | 15.7% |
| Celts | Hard | 896 | 156 | 17.4% |
| Mongols | Hard | 877 | 82 | 9.4% |
| Sparta | Hard | 891 | 132 | 14.8% |
| Athens | Hard | 895 | 136 | 15.2% |
| Aztecs | Hard | 889 | 110 | 12.4% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 899 | 135 | 15.0% | 121 | 148 | 194 | 278 | 487 |
| India | 895 | 395 | 44.1% | 42 | 93 | 132 | 224 | 402 |
| Byzantium | 878 | 614 | 69.9% | 65 | 85 | 111 | 169 | 360 |
| Germany | 880 | 291 | 33.1% | 97 | 129 | 167 | 243 | 465 |
| Celts | 896 | 642 | 71.7% | 31 | 184 | 210 | 210 | 289 |
| Mongols | 877 | 130 | 14.8% | 119 | 153 | 203 | 256 | 310 |
| Sparta | 891 | 187 | 21.0% | 131 | 164 | 216 | 287 | 382 |
| Athens | 895 | 353 | 39.4% | 129 | 177 | 255 | 255 | 255 |
| Aztecs | 889 | 348 | 39.1% | 133 | 189 | 225 | 240 | 314 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 160 | 193 | 243 | 313 | 500 |
| Winning team VP | 2 | 10 | 10 | 12 | 17 |
| First Soldier completed (turn, per empire) | 6 | 8 | 21 | 64 | 272 |
| First Settler completed (turn, per empire) | 3 | 14 | 16 | 32 | 208 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 8 | 18 | 23 | 46 | 257 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 7683 | 317 |
| First Settler completed | 7656 | 344 |
| First city founded | 8000 | 0 |
| Second city founded | 7560 | 440 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 1991 |
| 1 | 1160 |
| 2 | 971 |
| 3 | 757 |
| 4 | 623 |
| 5 | 464 |
| 6 | 372 |
| 7 | 271 |
| 8 | 189 |
| 9 | 159 |
| 10 | 521 |
| 11 | 289 |
| 12 | 140 |
| 13 | 62 |
| 14 | 19 |
| 15 | 4 |
| 16 | 7 |
| 17 | 1 |

## End state

- Average game length: 251.9 turns
- Average cities per empire: 4.29
- Average techs per empire: 31.28
- Average eliminations per game: 2.61

## Strategy diversity: observed behavior (classifier v2)

1000 finished games; 8000 player seats; 1000 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4607** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 3355 | 215 | 6.4% | 41.9% | 21.5% |
| Wide | 2534 | 685 | 27.0% | 31.7% | 68.5% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 9 | 0 | 0.0% | 0.1% | 0.0% |
| Economic | 691 | 99 | 14.3% | 8.6% | 9.9% |
| Generalist | 1411 | 1 | 0.1% | 17.6% | 0.1% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 20.6% / 3.1% (954) | n/a / n/a (0) | 62.5% / 4.2% (8) | 23.3% / 5.5% (503) | 81.7% / 2.4% (855) |
| Wide | 76.3% / 3.1% (954) | — | n/a / n/a (0) | 79.3% / 3.7% (9) | 56.8% / 4.5% (494) | 95.4% / 0.9% (842) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 33.3% / 4.2% (8) | 17.0% / 3.7% (9) | n/a / n/a (0) | — | 0.0% / 0.0% (6) | 83.3% / 5.6% (9) |
| Economic | 71.2% / 5.5% (503) | 38.7% / 4.5% (494) | n/a / n/a (0) | 100.0% / 0.0% (6) | — | 94.5% / 1.8% (425) |
| Generalist | 15.9% / 2.4% (855) | 3.7% / 0.9% (842) | n/a / n/a (0) | 11.1% / 5.6% (9) | 3.7% / 1.8% (425) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 14 / 247 | 81 / 365 | 0 / 0 | 0 / 0 | 16 / 133 | 0 / 154 |
| India | 13 / 507 | 28 / 223 | 0 / 0 | 0 / 0 | 1 / 7 | 1 / 158 |
| Byzantium | 0 / 9 | 79 / 407 | 0 / 0 | 0 / 4 | 13 / 153 | 0 / 305 |
| Germany | 22 / 346 | 109 / 307 | 0 / 0 | 0 / 4 | 7 / 49 | 0 / 174 |
| Celts | 18 / 266 | 128 / 416 | 0 / 0 | 0 / 0 | 10 / 79 | 0 / 135 |
| Mongols | 49 / 667 | 29 / 77 | 0 / 0 | 0 / 0 | 4 / 20 | 0 / 113 |
| Sparta | 61 / 693 | 53 / 142 | 0 / 0 | 0 / 1 | 18 / 43 | 0 / 12 |
| Athens | 17 / 268 | 101 / 307 | 0 / 0 | 0 / 0 | 18 / 112 | 0 / 208 |
| Aztecs | 21 / 352 | 77 / 290 | 0 / 0 | 0 / 0 | 12 / 95 | 0 / 152 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Turtle/Wonder. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 1000 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 15375/1000 | 15.375 | 1000/1000 (100.0%) |
| Peace offers | 101274/1000 | 101.274 | 1000/1000 (100.0%) |
| Peace agreements | 12485/1000 | 12.485 | 998/1000 (99.8%) |
| Battles | 1271062/1000 | 1271.062 | 1000/1000 (100.0%) |
| City captures | 24077/1000 | 24.077 | 1000/1000 (100.0%) |
| Settler captures | 4292/1000 | 4.292 | 954/1000 (95.4%) |
| Eliminations | 2606/1000 | 2.606 | 971/1000 (97.1%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 6622/8000 | 6622/8000 | 3150/1000 |
| Faction goals | 5018/8000 | 5018/8000 | 2004/1000 |
| Era objectives | 14370/8000 | 14370/8000 | 4116/1000 |
| Capitals taken | 2577/8000 | 2577/8000 | 1519/1000 |
| Total VP | 28587/8000 | 28587/8000 | 10789/1000 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 108/720 (15.0%) | 0 |
| Tied leaders (winner in set) | 264/280 (94.3%) | 0 |
| Any leader set | 372/1000 (37.2%) | 0 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 7537/8000 | 5 | 20 | 29 | 33 | 70 | -2.3 (1000) |
| Medieval | 7301/8000 | 31 | 58 | 62 | 66 | 117 | -2.0 (1000) |
| Renaissance | 7233/8000 | 66 | 85 | 90 | 98 | 163 | -3.4 (1000) |
| Industrial | 7039/8000 | 89 | 112 | 121 | 136 | 349 | -5.9 (1000) |
| Machine Age | 6409/8000 | 112 | 142 | 153 | 173 | 444 | -8.2 (1000) |
| World War | 5251/8000 | 131 | 167 | 182 | 208 | 499 | -10.4 (992) |
| Modern | 3576/8000 | 153 | 191 | 213 | 244 | 457 | -11.3 (879) |
| Future | 1967/8000 | 178 | 218 | 246 | 269 | 427 | -9.0 (612) |

Research idle turns (techs left, no research set): 2, of which stalled (two or more in a row): 0.

## Tech pace against the calendar

Median resolved turn each technology was learned, the display year of that turn (`calendar::calendar_label`) and the year the technology really arrived (n/a when no single year is established). Empires that never learned it are left out.

| Era | Technology | Learned / seats | Median turn | Median year | Real year |
| --- | --- | --- | --- | --- | --- |
| Ancient | Irrigation | 7961/8000 | 9 | 3173 BC | 6000 BC |
| Ancient | Bronze Working | 7801/8000 | 18 | 2242 BC | 3300 BC |
| Ancient | Masonry | 7997/8000 | 12 | 2863 BC | 3000 BC |
| Ancient | Currency | 7602/8000 | 27 | 1311 BC | 600 BC |
| Ancient | Writing | 7633/8000 | 25 | 1518 BC | 3200 BC |
| Classical | Mathematics | 7337/8000 | 45 | 251 BC | 300 BC |
| Classical | Engineering | 7318/8000 | 51 | 50 AD | n/a |
| Classical | Iron Working | 7378/8000 | 38 | 601 BC | 1200 BC |
| Classical | Code of Laws | 7312/8000 | 59 | 450 AD | 1754 BC |
| Classical | Philosophy | 7301/8000 | 61 | 530 AD | 500 BC |
| Medieval | Feudalism | 7270/8000 | 75 | 950 AD | n/a |
| Medieval | Machinery | 7284/8000 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 6440/8000 | 81 | 1130 AD | n/a |
| Medieval | Education | 7248/8000 | 86 | 1280 AD | 1088 AD |
| Medieval | Crop Rotation | 7233/8000 | 90 | 1400 AD | n/a |
| Renaissance | Gunpowder | 7222/8000 | 97 | 1470 AD | n/a |
| Renaissance | Metallurgy | 7155/8000 | 108 | 1580 AD | n/a |
| Renaissance | Military Engineering | 7183/8000 | 105 | 1550 AD | n/a |
| Renaissance | Banking | 7092/8000 | 118 | 1680 AD | 1397 AD |
| Renaissance | Printing Press | 7050/8000 | 119 | 1690 AD | 1440 AD |
| Industrial | Rifling | 6645/8000 | 143 | 1816 AD | n/a |
| Industrial | Military Science | 6833/8000 | 134 | 1783 AD | n/a |
| Industrial | Ballistics | 6713/8000 | 146 | 1828 AD | n/a |
| Industrial | Industrialization | 6933/8000 | 130 | 1768 AD | 1769 AD |
| Industrial | Railways | 6413/8000 | 153 | 1854 AD | 1825 AD |
| Machine Age | Replaceable Parts | 5928/8000 | 168 | 1889 AD | 1800 AD |
| Machine Age | Combustion | 6177/8000 | 162 | 1882 AD | 1876 AD |
| Machine Age | Steel | 5907/8000 | 167 | 1887 AD | 1856 AD |
| Machine Age | Flight | 5526/8000 | 177 | 1899 AD | 1903 AD |
| Machine Age | Electricity | 5289/8000 | 182 | 1904 AD | 1879 AD |
| World War | Mass Mobilisation | 4403/8000 | 197 | 1924 AD | n/a |
| World War | Rocketry | 4183/8000 | 202 | 1931 AD | 1926 AD |
| World War | Armoured Warfare | 3966/8000 | 207 | 1938 AD | 1916 AD |
| World War | Air Power | 3796/8000 | 211 | 1944 AD | n/a |
| World War | Electronics | 4897/8000 | 189 | 1912 AD | n/a |
| Modern | Mechanised Warfare | 2675/8000 | 228 | 1981 AD | n/a |
| Modern | Composite Armour | 2422/8000 | 233 | 1993 AD | n/a |
| Modern | Guided Missiles | 2323/8000 | 238 | 2005 AD | n/a |
| Modern | Jet Engines | 2221/8000 | 242 | 2013 AD | 1939 AD |
| Modern | Green Revolution | 3116/8000 | 221 | 1964 AD | n/a |
| Future | Autonomous Systems | 1367/8000 | 256 | 2032 AD | n/a |
| Future | Robotics | 1116/8000 | 262 | 2038 AD | n/a |
| Future | Fusion Power | 1093/8000 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 1586/8000 | 255 | 2031 AD | n/a |
| Future | Recursive AI | 1000/8000 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 64.29 |
| 1 | 73.78 | 14.15 |
| 2 | 28.20 | 5.45 |
| 3 | 8.01 | 1.29 |
| 4 | 1.67 | 0.30 |
| 5 | 0.36 | 0.08 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 20261126 | 257 | 1350 | 16 | 1 | 3 |
| 1 | 20261127 | 252 | 1084 | 21 | 6 | 4 |
| 2 | 20261128 | 277 | 1660 | 14 | 8 | 1 |
| 3 | 20261129 | 287 | 1198 | 37 | 1 | 2 |
| 4 | 20261130 | 182 | 700 | 15 | 3 | 1 |
| 5 | 20261131 | 243 | 989 | 28 | 10 | 2 |
| 6 | 20261132 | 268 | 1646 | 27 | 1 | 3 |
| 7 | 20261133 | 335 | 1880 | 25 | 0 | 5 |
| 8 | 20261134 | 185 | 760 | 14 | 3 | 2 |
| 9 | 20261135 | 360 | 1945 | 34 | 5 | 4 |
| 10 | 20261136 | 286 | 1360 | 29 | 6 | 2 |
| 11 | 20261137 | 271 | 987 | 17 | 6 | 3 |
| 12 | 20261138 | 196 | 1333 | 18 | 3 | 1 |
| 13 | 20261139 | 288 | 1422 | 16 | 7 | 2 |
| 14 | 20261140 | 411 | 2248 | 49 | 6 | 4 |
| 15 | 20261141 | 225 | 1278 | 17 | 6 | 4 |
| 16 | 20261142 | 220 | 987 | 20 | 5 | 1 |
| 17 | 20261143 | 201 | 1053 | 28 | 4 | 2 |
| 18 | 20261144 | 278 | 1146 | 29 | 3 | 4 |
| 19 | 20261145 | 318 | 1466 | 43 | 3 | 4 |
| 20 | 20261146 | 332 | 1673 | 42 | 3 | 3 |
| 21 | 20261147 | 309 | 1544 | 32 | 12 | 3 |
| 22 | 20261148 | 197 | 1001 | 22 | 5 | 1 |
| 23 | 20261149 | 203 | 1149 | 13 | 1 | 3 |
| 24 | 20261150 | 253 | 1036 | 28 | 4 | 2 |
| 25 | 20261151 | 242 | 1059 | 24 | 0 | 2 |
| 26 | 20261152 | 190 | 1105 | 16 | 6 | 2 |
| 27 | 20261153 | 212 | 1298 | 26 | 6 | 1 |
| 28 | 20261154 | 241 | 1103 | 27 | 11 | 2 |
| 29 | 20261155 | 182 | 775 | 16 | 5 | 1 |
| 30 | 20261156 | 279 | 1628 | 17 | 0 | 3 |
| 31 | 20261157 | 286 | 1625 | 29 | 3 | 2 |
| 32 | 20261158 | 237 | 1194 | 26 | 5 | 3 |
| 33 | 20261159 | 181 | 743 | 12 | 1 | 1 |
| 34 | 20261160 | 257 | 1331 | 28 | 2 | 3 |
| 35 | 20261161 | 211 | 834 | 15 | 4 | 2 |
| 36 | 20261162 | 254 | 1060 | 18 | 0 | 4 |
| 37 | 20261163 | 252 | 1337 | 32 | 4 | 2 |
| 38 | 20261164 | 217 | 1063 | 14 | 4 | 3 |
| 39 | 20261165 | 279 | 1957 | 15 | 3 | 3 |
| 40 | 20261166 | 167 | 925 | 13 | 6 | 2 |
| 41 | 20261167 | 193 | 945 | 24 | 4 | 4 |
| 42 | 20261168 | 251 | 1562 | 25 | 5 | 2 |
| 43 | 20261169 | 204 | 1237 | 19 | 5 | 2 |
| 44 | 20261170 | 500 | 2423 | 32 | 4 | 5 |
| 45 | 20261171 | 245 | 1054 | 19 | 6 | 2 |
| 46 | 20261172 | 279 | 1593 | 36 | 2 | 4 |
| 47 | 20261173 | 242 | 1228 | 22 | 2 | 3 |
| 48 | 20261174 | 267 | 1363 | 34 | 4 | 4 |
| 49 | 20261175 | 275 | 1490 | 19 | 4 | 2 |
| 50 | 20261176 | 203 | 963 | 20 | 2 | 2 |
| 51 | 20261177 | 267 | 1463 | 28 | 7 | 2 |
| 52 | 20261178 | 242 | 1088 | 18 | 0 | 2 |
| 53 | 20261179 | 243 | 1298 | 29 | 9 | 3 |
| 54 | 20261180 | 203 | 789 | 10 | 2 | 3 |
| 55 | 20261181 | 200 | 872 | 23 | 6 | 3 |
| 56 | 20261182 | 238 | 1282 | 31 | 2 | 1 |
| 57 | 20261183 | 316 | 1843 | 43 | 3 | 2 |
| 58 | 20261184 | 234 | 1150 | 26 | 1 | 3 |
| 59 | 20261185 | 220 | 828 | 19 | 6 | 2 |
| 60 | 20261186 | 254 | 1261 | 29 | 3 | 2 |
| 61 | 20261187 | 271 | 1099 | 30 | 5 | 3 |
| 62 | 20261188 | 339 | 2048 | 39 | 1 | 3 |
| 63 | 20261189 | 242 | 1285 | 26 | 9 | 4 |
| 64 | 20261190 | 262 | 1339 | 37 | 7 | 3 |
| 65 | 20261191 | 227 | 1189 | 18 | 6 | 3 |
| 66 | 20261192 | 271 | 1513 | 37 | 2 | 2 |
| 67 | 20261193 | 267 | 1605 | 15 | 4 | 3 |
| 68 | 20261194 | 296 | 1586 | 22 | 4 | 3 |
| 69 | 20261195 | 189 | 935 | 16 | 6 | 3 |
| 70 | 20261196 | 193 | 836 | 13 | 7 | 2 |
| 71 | 20261197 | 254 | 1429 | 24 | 2 | 5 |
| 72 | 20261198 | 302 | 1653 | 22 | 4 | 2 |
| 73 | 20261199 | 323 | 1581 | 33 | 6 | 4 |
| 74 | 20261200 | 211 | 1024 | 30 | 3 | 3 |
| 75 | 20261201 | 220 | 972 | 26 | 7 | 5 |
| 76 | 20261202 | 228 | 1314 | 27 | 5 | 3 |
| 77 | 20261203 | 305 | 1513 | 22 | 8 | 5 |
| 78 | 20261204 | 365 | 1637 | 22 | 8 | 6 |
| 79 | 20261205 | 234 | 1103 | 24 | 4 | 1 |
| 80 | 20261206 | 241 | 1016 | 24 | 8 | 2 |
| 81 | 20261207 | 208 | 701 | 10 | 3 | 3 |
| 82 | 20261208 | 244 | 1388 | 24 | 3 | 1 |
| 83 | 20261209 | 263 | 1558 | 27 | 5 | 2 |
| 84 | 20261210 | 333 | 1545 | 32 | 5 | 2 |
| 85 | 20261211 | 271 | 1647 | 35 | 1 | 2 |
| 86 | 20261212 | 241 | 1080 | 17 | 2 | 3 |
| 87 | 20261213 | 163 | 489 | 12 | 7 | 3 |
| 88 | 20261214 | 270 | 993 | 25 | 6 | 3 |
| 89 | 20261215 | 176 | 750 | 9 | 2 | 1 |
| 90 | 20261216 | 264 | 1249 | 33 | 9 | 2 |
| 91 | 20261217 | 244 | 1203 | 25 | 5 | 2 |
| 92 | 20261218 | 319 | 1495 | 28 | 2 | 6 |
| 93 | 20261219 | 241 | 1088 | 9 | 3 | 4 |
| 94 | 20261220 | 279 | 1305 | 35 | 9 | 3 |
| 95 | 20261221 | 273 | 1236 | 31 | 8 | 3 |
| 96 | 20261222 | 222 | 1149 | 26 | 3 | 1 |
| 97 | 20261223 | 196 | 689 | 12 | 3 | 1 |
| 98 | 20261224 | 288 | 1602 | 15 | 11 | 4 |
| 99 | 20261225 | 387 | 2563 | 48 | 8 | 2 |
| 100 | 20261226 | 195 | 831 | 18 | 3 | 2 |
| 101 | 20261227 | 234 | 1207 | 28 | 2 | 2 |
| 102 | 20261228 | 219 | 1051 | 10 | 4 | 1 |
| 103 | 20261229 | 254 | 1157 | 22 | 3 | 3 |
| 104 | 20261230 | 357 | 2013 | 32 | 9 | 2 |
| 105 | 20261231 | 199 | 692 | 12 | 1 | 2 |
| 106 | 20261232 | 230 | 1186 | 21 | 9 | 2 |
| 107 | 20261233 | 275 | 1295 | 39 | 4 | 4 |
| 108 | 20261234 | 207 | 655 | 22 | 9 | 4 |
| 109 | 20261235 | 248 | 999 | 26 | 2 | 3 |
| 110 | 20261236 | 181 | 746 | 9 | 3 | 2 |
| 111 | 20261237 | 265 | 1382 | 31 | 2 | 4 |
| 112 | 20261238 | 243 | 1274 | 25 | 4 | 3 |
| 113 | 20261239 | 366 | 2684 | 25 | 3 | 3 |
| 114 | 20261240 | 306 | 1811 | 28 | 4 | 4 |
| 115 | 20261241 | 236 | 1020 | 15 | 0 | 4 |
| 116 | 20261242 | 252 | 1336 | 37 | 1 | 3 |
| 117 | 20261243 | 226 | 1131 | 17 | 5 | 2 |
| 118 | 20261244 | 241 | 1335 | 30 | 4 | 2 |
| 119 | 20261245 | 167 | 700 | 15 | 2 | 2 |
| 120 | 20261246 | 285 | 1699 | 18 | 3 | 2 |
| 121 | 20261247 | 179 | 719 | 8 | 3 | 3 |
| 122 | 20261248 | 417 | 1923 | 52 | 9 | 4 |
| 123 | 20261249 | 269 | 1170 | 34 | 2 | 3 |
| 124 | 20261250 | 260 | 1384 | 36 | 6 | 4 |
| 125 | 20261251 | 208 | 954 | 10 | 1 | 3 |
| 126 | 20261252 | 246 | 1008 | 28 | 4 | 2 |
| 127 | 20261253 | 224 | 926 | 16 | 2 | 2 |
| 128 | 20261254 | 227 | 1340 | 19 | 4 | 2 |
| 129 | 20261255 | 216 | 1087 | 23 | 2 | 3 |
| 130 | 20261256 | 193 | 975 | 27 | 1 | 3 |
| 131 | 20261257 | 229 | 1098 | 6 | 3 | 4 |
| 132 | 20261258 | 272 | 1071 | 13 | 1 | 3 |
| 133 | 20261259 | 252 | 1367 | 27 | 7 | 2 |
| 134 | 20261260 | 273 | 1723 | 29 | 4 | 2 |
| 135 | 20261261 | 500 | 2016 | 64 | 5 | 5 |
| 136 | 20261262 | 186 | 750 | 12 | 2 | 1 |
| 137 | 20261263 | 215 | 1088 | 27 | 2 | 2 |
| 138 | 20261264 | 260 | 1195 | 32 | 3 | 5 |
| 139 | 20261265 | 224 | 1086 | 20 | 5 | 3 |
| 140 | 20261266 | 272 | 1573 | 39 | 3 | 4 |
| 141 | 20261267 | 280 | 1206 | 32 | 5 | 5 |
| 142 | 20261268 | 226 | 1125 | 24 | 3 | 0 |
| 143 | 20261269 | 226 | 1098 | 17 | 7 | 2 |
| 144 | 20261270 | 234 | 1004 | 22 | 1 | 1 |
| 145 | 20261271 | 213 | 900 | 23 | 0 | 2 |
| 146 | 20261272 | 267 | 1450 | 20 | 5 | 2 |
| 147 | 20261273 | 217 | 978 | 22 | 4 | 2 |
| 148 | 20261274 | 241 | 1257 | 23 | 2 | 2 |
| 149 | 20261275 | 256 | 1216 | 32 | 7 | 5 |
| 150 | 20261276 | 236 | 950 | 15 | 11 | 1 |
| 151 | 20261277 | 219 | 982 | 13 | 4 | 2 |
| 152 | 20261278 | 235 | 1336 | 27 | 9 | 3 |
| 153 | 20261279 | 247 | 1239 | 19 | 7 | 1 |
| 154 | 20261280 | 220 | 982 | 23 | 3 | 3 |
| 155 | 20261281 | 292 | 1783 | 36 | 6 | 3 |
| 156 | 20261282 | 273 | 1358 | 19 | 4 | 2 |
| 157 | 20261283 | 215 | 877 | 19 | 2 | 1 |
| 158 | 20261284 | 258 | 1043 | 15 | 8 | 4 |
| 159 | 20261285 | 236 | 1250 | 29 | 10 | 3 |
| 160 | 20261286 | 214 | 1190 | 26 | 6 | 3 |
| 161 | 20261287 | 338 | 1805 | 33 | 4 | 2 |
| 162 | 20261288 | 231 | 1130 | 16 | 6 | 6 |
| 163 | 20261289 | 228 | 1161 | 17 | 1 | 3 |
| 164 | 20261290 | 277 | 1284 | 31 | 4 | 3 |
| 165 | 20261291 | 231 | 1205 | 21 | 5 | 3 |
| 166 | 20261292 | 251 | 1303 | 32 | 5 | 4 |
| 167 | 20261293 | 245 | 1244 | 25 | 4 | 1 |
| 168 | 20261294 | 203 | 953 | 20 | 3 | 1 |
| 169 | 20261295 | 271 | 1398 | 31 | 2 | 2 |
| 170 | 20261296 | 230 | 1110 | 15 | 0 | 2 |
| 171 | 20261297 | 220 | 781 | 21 | 1 | 3 |
| 172 | 20261298 | 328 | 1990 | 45 | 3 | 4 |
| 173 | 20261299 | 330 | 1534 | 34 | 5 | 4 |
| 174 | 20261300 | 239 | 1258 | 24 | 7 | 2 |
| 175 | 20261301 | 255 | 1284 | 20 | 0 | 4 |
| 176 | 20261302 | 191 | 892 | 18 | 7 | 2 |
| 177 | 20261303 | 206 | 965 | 22 | 0 | 1 |
| 178 | 20261304 | 246 | 880 | 7 | 1 | 5 |
| 179 | 20261305 | 299 | 2003 | 26 | 8 | 1 |
| 180 | 20261306 | 250 | 1375 | 12 | 2 | 3 |
| 181 | 20261307 | 214 | 953 | 25 | 2 | 2 |
| 182 | 20261308 | 198 | 977 | 16 | 5 | 1 |
| 183 | 20261309 | 234 | 1039 | 26 | 7 | 3 |
| 184 | 20261310 | 351 | 2136 | 47 | 5 | 3 |
| 185 | 20261311 | 229 | 1211 | 19 | 3 | 3 |
| 186 | 20261312 | 280 | 1680 | 24 | 6 | 4 |
| 187 | 20261313 | 263 | 1384 | 32 | 7 | 3 |
| 188 | 20261314 | 183 | 819 | 11 | 4 | 1 |
| 189 | 20261315 | 402 | 2095 | 32 | 10 | 5 |
| 190 | 20261316 | 395 | 2531 | 40 | 6 | 3 |
| 191 | 20261317 | 277 | 1242 | 32 | 4 | 5 |
| 192 | 20261318 | 218 | 1035 | 14 | 2 | 4 |
| 193 | 20261319 | 273 | 1358 | 17 | 7 | 3 |
| 194 | 20261320 | 226 | 1076 | 23 | 4 | 1 |
| 195 | 20261321 | 235 | 1170 | 31 | 6 | 3 |
| 196 | 20261322 | 264 | 1271 | 13 | 3 | 4 |
| 197 | 20261323 | 333 | 1825 | 21 | 3 | 3 |
| 198 | 20261324 | 203 | 749 | 17 | 3 | 1 |
| 199 | 20261325 | 307 | 1472 | 21 | 7 | 1 |
| 200 | 20261326 | 312 | 1860 | 32 | 3 | 3 |
| 201 | 20261327 | 500 | 2881 | 42 | 2 | 3 |
| 202 | 20261328 | 285 | 1609 | 39 | 1 | 2 |
| 203 | 20261329 | 184 | 928 | 6 | 4 | 2 |
| 204 | 20261330 | 201 | 862 | 19 | 3 | 1 |
| 205 | 20261331 | 303 | 1412 | 45 | 0 | 5 |
| 206 | 20261332 | 296 | 1403 | 18 | 4 | 3 |
| 207 | 20261333 | 264 | 1164 | 27 | 5 | 4 |
| 208 | 20261334 | 225 | 1333 | 24 | 10 | 3 |
| 209 | 20261335 | 247 | 1506 | 34 | 6 | 3 |
| 210 | 20261336 | 179 | 983 | 13 | 9 | 2 |
| 211 | 20261337 | 210 | 1127 | 16 | 12 | 2 |
| 212 | 20261338 | 233 | 1091 | 23 | 6 | 1 |
| 213 | 20261339 | 208 | 909 | 14 | 6 | 1 |
| 214 | 20261340 | 233 | 764 | 8 | 3 | 2 |
| 215 | 20261341 | 204 | 858 | 10 | 1 | 3 |
| 216 | 20261342 | 265 | 1593 | 38 | 3 | 4 |
| 217 | 20261343 | 196 | 977 | 18 | 3 | 1 |
| 218 | 20261344 | 235 | 994 | 21 | 1 | 3 |
| 219 | 20261345 | 243 | 1335 | 26 | 3 | 2 |
| 220 | 20261346 | 275 | 1304 | 25 | 2 | 4 |
| 221 | 20261347 | 341 | 2242 | 22 | 6 | 0 |
| 222 | 20261348 | 228 | 1082 | 22 | 5 | 3 |
| 223 | 20261349 | 238 | 907 | 16 | 5 | 2 |
| 224 | 20261350 | 202 | 939 | 32 | 6 | 3 |
| 225 | 20261351 | 211 | 881 | 22 | 5 | 1 |
| 226 | 20261352 | 204 | 1017 | 21 | 1 | 2 |
| 227 | 20261353 | 304 | 1816 | 17 | 5 | 2 |
| 228 | 20261354 | 278 | 1297 | 31 | 4 | 3 |
| 229 | 20261355 | 229 | 1254 | 29 | 3 | 3 |
| 230 | 20261356 | 222 | 1141 | 26 | 0 | 4 |
| 231 | 20261357 | 160 | 778 | 13 | 3 | 1 |
| 232 | 20261358 | 224 | 1094 | 15 | 7 | 4 |
| 233 | 20261359 | 233 | 1180 | 28 | 4 | 3 |
| 234 | 20261360 | 202 | 811 | 15 | 4 | 4 |
| 235 | 20261361 | 273 | 1219 | 10 | 1 | 3 |
| 236 | 20261362 | 272 | 1288 | 29 | 3 | 2 |
| 237 | 20261363 | 263 | 1324 | 28 | 3 | 4 |
| 238 | 20261364 | 212 | 862 | 21 | 6 | 4 |
| 239 | 20261365 | 315 | 1516 | 33 | 7 | 4 |
| 240 | 20261366 | 244 | 1363 | 25 | 9 | 2 |
| 241 | 20261367 | 312 | 1608 | 35 | 1 | 3 |
| 242 | 20261368 | 222 | 1145 | 27 | 2 | 2 |
| 243 | 20261369 | 183 | 863 | 13 | 4 | 2 |
| 244 | 20261370 | 262 | 1366 | 21 | 7 | 2 |
| 245 | 20261371 | 267 | 1372 | 39 | 4 | 4 |
| 246 | 20261372 | 187 | 905 | 8 | 5 | 1 |
| 247 | 20261373 | 231 | 1111 | 22 | 1 | 0 |
| 248 | 20261374 | 394 | 2741 | 16 | 3 | 5 |
| 249 | 20261375 | 222 | 959 | 16 | 3 | 2 |
| 250 | 20261376 | 263 | 1567 | 29 | 0 | 3 |
| 251 | 20261377 | 185 | 878 | 15 | 4 | 3 |
| 252 | 20261378 | 350 | 2106 | 42 | 8 | 4 |
| 253 | 20261379 | 234 | 1008 | 17 | 2 | 2 |
| 254 | 20261380 | 286 | 1419 | 26 | 5 | 3 |
| 255 | 20261381 | 306 | 1386 | 19 | 5 | 4 |
| 256 | 20261382 | 287 | 1565 | 21 | 3 | 3 |
| 257 | 20261383 | 248 | 1270 | 27 | 0 | 4 |
| 258 | 20261384 | 241 | 1466 | 21 | 5 | 3 |
| 259 | 20261385 | 238 | 933 | 16 | 8 | 2 |
| 260 | 20261386 | 381 | 1754 | 30 | 7 | 4 |
| 261 | 20261387 | 253 | 1150 | 33 | 9 | 1 |
| 262 | 20261388 | 206 | 825 | 15 | 4 | 4 |
| 263 | 20261389 | 259 | 1445 | 23 | 4 | 3 |
| 264 | 20261390 | 284 | 1226 | 33 | 6 | 2 |
| 265 | 20261391 | 223 | 1015 | 17 | 4 | 3 |
| 266 | 20261392 | 193 | 789 | 14 | 0 | 3 |
| 267 | 20261393 | 282 | 1303 | 32 | 7 | 3 |
| 268 | 20261394 | 242 | 1431 | 27 | 6 | 1 |
| 269 | 20261395 | 267 | 1440 | 18 | 8 | 3 |
| 270 | 20261396 | 221 | 1163 | 24 | 2 | 1 |
| 271 | 20261397 | 246 | 1381 | 25 | 1 | 2 |
| 272 | 20261398 | 280 | 1686 | 25 | 5 | 2 |
| 273 | 20261399 | 222 | 1042 | 18 | 6 | 1 |
| 274 | 20261400 | 213 | 683 | 19 | 1 | 3 |
| 275 | 20261401 | 257 | 1186 | 25 | 2 | 2 |
| 276 | 20261402 | 204 | 1037 | 15 | 2 | 3 |
| 277 | 20261403 | 216 | 1019 | 20 | 5 | 2 |
| 278 | 20261404 | 259 | 1288 | 30 | 5 | 2 |
| 279 | 20261405 | 217 | 887 | 9 | 4 | 3 |
| 280 | 20261406 | 175 | 868 | 10 | 12 | 0 |
| 281 | 20261407 | 209 | 1028 | 19 | 4 | 2 |
| 282 | 20261408 | 208 | 1258 | 18 | 10 | 2 |
| 283 | 20261409 | 250 | 1118 | 14 | 8 | 3 |
| 284 | 20261410 | 273 | 1537 | 21 | 4 | 5 |
| 285 | 20261411 | 205 | 1050 | 19 | 4 | 1 |
| 286 | 20261412 | 251 | 1413 | 31 | 3 | 2 |
| 287 | 20261413 | 292 | 1436 | 25 | 3 | 2 |
| 288 | 20261414 | 264 | 1380 | 23 | 4 | 2 |
| 289 | 20261415 | 288 | 1719 | 34 | 2 | 2 |
| 290 | 20261416 | 275 | 1676 | 18 | 2 | 5 |
| 291 | 20261417 | 180 | 776 | 13 | 3 | 2 |
| 292 | 20261418 | 212 | 1196 | 22 | 1 | 2 |
| 293 | 20261419 | 309 | 1843 | 32 | 6 | 3 |
| 294 | 20261420 | 257 | 1206 | 29 | 7 | 2 |
| 295 | 20261421 | 207 | 934 | 11 | 10 | 0 |
| 296 | 20261422 | 220 | 1118 | 7 | 1 | 3 |
| 297 | 20261423 | 297 | 1430 | 15 | 2 | 2 |
| 298 | 20261424 | 265 | 1250 | 28 | 1 | 5 |
| 299 | 20261425 | 248 | 1274 | 30 | 1 | 2 |
| 300 | 20261426 | 212 | 1007 | 20 | 3 | 3 |
| 301 | 20261427 | 295 | 2042 | 39 | 1 | 2 |
| 302 | 20261428 | 248 | 1229 | 36 | 7 | 2 |
| 303 | 20261429 | 192 | 880 | 15 | 2 | 3 |
| 304 | 20261430 | 305 | 1801 | 35 | 11 | 4 |
| 305 | 20261431 | 301 | 1361 | 26 | 6 | 3 |
| 306 | 20261432 | 248 | 816 | 24 | 3 | 3 |
| 307 | 20261433 | 206 | 1032 | 22 | 0 | 2 |
| 308 | 20261434 | 208 | 831 | 22 | 7 | 3 |
| 309 | 20261435 | 193 | 896 | 14 | 6 | 2 |
| 310 | 20261436 | 227 | 1316 | 29 | 4 | 2 |
| 311 | 20261437 | 288 | 1771 | 33 | 2 | 3 |
| 312 | 20261438 | 219 | 1135 | 19 | 1 | 2 |
| 313 | 20261439 | 441 | 2400 | 60 | 8 | 3 |
| 314 | 20261440 | 302 | 1542 | 26 | 6 | 2 |
| 315 | 20261441 | 194 | 1034 | 17 | 1 | 2 |
| 316 | 20261442 | 233 | 937 | 24 | 6 | 2 |
| 317 | 20261443 | 193 | 931 | 18 | 1 | 2 |
| 318 | 20261444 | 300 | 1496 | 40 | 9 | 3 |
| 319 | 20261445 | 290 | 1591 | 37 | 3 | 3 |
| 320 | 20261446 | 370 | 2069 | 43 | 7 | 5 |
| 321 | 20261447 | 168 | 818 | 17 | 1 | 1 |
| 322 | 20261448 | 189 | 925 | 14 | 2 | 1 |
| 323 | 20261449 | 223 | 1178 | 23 | 5 | 0 |
| 324 | 20261450 | 222 | 1068 | 24 | 7 | 2 |
| 325 | 20261451 | 319 | 1794 | 32 | 11 | 1 |
| 326 | 20261452 | 240 | 1212 | 22 | 7 | 4 |
| 327 | 20261453 | 171 | 662 | 16 | 1 | 3 |
| 328 | 20261454 | 258 | 1324 | 39 | 3 | 3 |
| 329 | 20261455 | 213 | 870 | 28 | 8 | 1 |
| 330 | 20261456 | 230 | 1244 | 22 | 0 | 2 |
| 331 | 20261457 | 289 | 1523 | 25 | 5 | 3 |
| 332 | 20261458 | 287 | 1596 | 27 | 7 | 4 |
| 333 | 20261459 | 290 | 1456 | 21 | 2 | 3 |
| 334 | 20261460 | 293 | 1331 | 27 | 5 | 3 |
| 335 | 20261461 | 225 | 985 | 19 | 4 | 3 |
| 336 | 20261462 | 228 | 1216 | 36 | 3 | 2 |
| 337 | 20261463 | 192 | 1067 | 18 | 3 | 2 |
| 338 | 20261464 | 199 | 981 | 17 | 10 | 2 |
| 339 | 20261465 | 228 | 1072 | 22 | 7 | 4 |
| 340 | 20261466 | 265 | 1538 | 28 | 5 | 2 |
| 341 | 20261467 | 295 | 1484 | 32 | 2 | 2 |
| 342 | 20261468 | 194 | 921 | 20 | 5 | 2 |
| 343 | 20261469 | 284 | 1248 | 15 | 5 | 4 |
| 344 | 20261470 | 185 | 816 | 13 | 2 | 1 |
| 345 | 20261471 | 250 | 1299 | 22 | 9 | 2 |
| 346 | 20261472 | 308 | 1713 | 19 | 3 | 3 |
| 347 | 20261473 | 313 | 1988 | 31 | 4 | 5 |
| 348 | 20261474 | 338 | 2077 | 46 | 9 | 4 |
| 349 | 20261475 | 273 | 1544 | 36 | 0 | 3 |
| 350 | 20261476 | 242 | 1265 | 24 | 13 | 0 |
| 351 | 20261477 | 254 | 1210 | 29 | 0 | 2 |
| 352 | 20261478 | 223 | 961 | 22 | 6 | 4 |
| 353 | 20261479 | 213 | 1280 | 20 | 1 | 4 |
| 354 | 20261480 | 234 | 953 | 30 | 4 | 1 |
| 355 | 20261481 | 287 | 1497 | 17 | 3 | 3 |
| 356 | 20261482 | 211 | 942 | 16 | 7 | 3 |
| 357 | 20261483 | 226 | 1128 | 30 | 6 | 2 |
| 358 | 20261484 | 223 | 1009 | 17 | 5 | 1 |
| 359 | 20261485 | 298 | 1711 | 27 | 4 | 4 |
| 360 | 20261486 | 299 | 1298 | 35 | 7 | 4 |
| 361 | 20261487 | 246 | 1351 | 27 | 4 | 3 |
| 362 | 20261488 | 376 | 2353 | 33 | 4 | 2 |
| 363 | 20261489 | 233 | 1091 | 16 | 7 | 2 |
| 364 | 20261490 | 208 | 1124 | 28 | 2 | 2 |
| 365 | 20261491 | 263 | 1304 | 28 | 4 | 4 |
| 366 | 20261492 | 198 | 895 | 17 | 6 | 0 |
| 367 | 20261493 | 255 | 1250 | 30 | 1 | 5 |
| 368 | 20261494 | 400 | 2138 | 53 | 12 | 5 |
| 369 | 20261495 | 213 | 1104 | 15 | 6 | 0 |
| 370 | 20261496 | 190 | 923 | 20 | 2 | 2 |
| 371 | 20261497 | 245 | 1262 | 20 | 5 | 2 |
| 372 | 20261498 | 238 | 1168 | 18 | 2 | 1 |
| 373 | 20261499 | 301 | 1421 | 28 | 7 | 3 |
| 374 | 20261500 | 201 | 815 | 20 | 2 | 2 |
| 375 | 20261501 | 253 | 1568 | 37 | 3 | 1 |
| 376 | 20261502 | 289 | 1657 | 27 | 5 | 4 |
| 377 | 20261503 | 220 | 1225 | 20 | 5 | 3 |
| 378 | 20261504 | 205 | 1048 | 24 | 7 | 1 |
| 379 | 20261505 | 276 | 1262 | 37 | 2 | 2 |
| 380 | 20261506 | 379 | 2123 | 50 | 9 | 4 |
| 381 | 20261507 | 242 | 1225 | 20 | 7 | 0 |
| 382 | 20261508 | 224 | 1131 | 26 | 4 | 1 |
| 383 | 20261509 | 221 | 1426 | 21 | 6 | 2 |
| 384 | 20261510 | 273 | 1278 | 30 | 1 | 4 |
| 385 | 20261511 | 239 | 1252 | 35 | 4 | 4 |
| 386 | 20261512 | 242 | 1403 | 17 | 4 | 2 |
| 387 | 20261513 | 232 | 1359 | 22 | 1 | 0 |
| 388 | 20261514 | 291 | 1325 | 30 | 4 | 5 |
| 389 | 20261515 | 263 | 1708 | 9 | 7 | 3 |
| 390 | 20261516 | 245 | 1518 | 30 | 9 | 1 |
| 391 | 20261517 | 276 | 1513 | 33 | 4 | 4 |
| 392 | 20261518 | 310 | 1409 | 25 | 3 | 1 |
| 393 | 20261519 | 325 | 1624 | 33 | 18 | 3 |
| 394 | 20261520 | 232 | 878 | 16 | 2 | 4 |
| 395 | 20261521 | 196 | 1027 | 23 | 5 | 3 |
| 396 | 20261522 | 203 | 991 | 21 | 5 | 1 |
| 397 | 20261523 | 225 | 1136 | 24 | 12 | 1 |
| 398 | 20261524 | 253 | 1322 | 28 | 12 | 3 |
| 399 | 20261525 | 272 | 1250 | 29 | 6 | 4 |
| 400 | 20261526 | 256 | 1031 | 16 | 2 | 5 |
| 401 | 20261527 | 193 | 909 | 19 | 8 | 0 |
| 402 | 20261528 | 360 | 1597 | 21 | 5 | 3 |
| 403 | 20261529 | 221 | 1058 | 24 | 4 | 2 |
| 404 | 20261530 | 234 | 1106 | 25 | 2 | 4 |
| 405 | 20261531 | 238 | 1190 | 26 | 1 | 3 |
| 406 | 20261532 | 309 | 1343 | 28 | 4 | 2 |
| 407 | 20261533 | 186 | 765 | 20 | 5 | 1 |
| 408 | 20261534 | 281 | 1749 | 33 | 13 | 3 |
| 409 | 20261535 | 213 | 1184 | 22 | 4 | 2 |
| 410 | 20261536 | 215 | 1168 | 19 | 1 | 2 |
| 411 | 20261537 | 236 | 1126 | 20 | 4 | 2 |
| 412 | 20261538 | 238 | 1242 | 34 | 6 | 3 |
| 413 | 20261539 | 214 | 954 | 31 | 3 | 2 |
| 414 | 20261540 | 181 | 750 | 14 | 0 | 2 |
| 415 | 20261541 | 336 | 1543 | 40 | 3 | 4 |
| 416 | 20261542 | 281 | 1565 | 27 | 7 | 3 |
| 417 | 20261543 | 287 | 1088 | 29 | 3 | 4 |
| 418 | 20261544 | 242 | 1280 | 14 | 5 | 2 |
| 419 | 20261545 | 202 | 944 | 8 | 1 | 2 |
| 420 | 20261546 | 258 | 1253 | 35 | 9 | 3 |
| 421 | 20261547 | 233 | 1356 | 22 | 0 | 3 |
| 422 | 20261548 | 227 | 1088 | 23 | 2 | 1 |
| 423 | 20261549 | 193 | 1108 | 16 | 4 | 2 |
| 424 | 20261550 | 291 | 1888 | 26 | 1 | 1 |
| 425 | 20261551 | 313 | 1530 | 30 | 10 | 1 |
| 426 | 20261552 | 232 | 1376 | 32 | 2 | 3 |
| 427 | 20261553 | 301 | 1864 | 36 | 1 | 2 |
| 428 | 20261554 | 240 | 1227 | 24 | 3 | 4 |
| 429 | 20261555 | 330 | 1648 | 39 | 0 | 3 |
| 430 | 20261556 | 252 | 1206 | 11 | 2 | 3 |
| 431 | 20261557 | 216 | 979 | 27 | 5 | 5 |
| 432 | 20261558 | 293 | 1658 | 38 | 5 | 3 |
| 433 | 20261559 | 285 | 1151 | 14 | 4 | 3 |
| 434 | 20261560 | 201 | 1336 | 19 | 0 | 2 |
| 435 | 20261561 | 308 | 1488 | 35 | 10 | 3 |
| 436 | 20261562 | 345 | 1483 | 31 | 4 | 4 |
| 437 | 20261563 | 247 | 1336 | 34 | 11 | 4 |
| 438 | 20261564 | 284 | 1468 | 27 | 2 | 1 |
| 439 | 20261565 | 210 | 1083 | 20 | 8 | 2 |
| 440 | 20261566 | 343 | 1657 | 26 | 8 | 2 |
| 441 | 20261567 | 280 | 1535 | 24 | 9 | 4 |
| 442 | 20261568 | 211 | 936 | 17 | 5 | 1 |
| 443 | 20261569 | 226 | 1174 | 8 | 4 | 3 |
| 444 | 20261570 | 232 | 1005 | 17 | 10 | 3 |
| 445 | 20261571 | 275 | 1319 | 28 | 5 | 2 |
| 446 | 20261572 | 229 | 1126 | 14 | 4 | 1 |
| 447 | 20261573 | 205 | 1027 | 29 | 5 | 3 |
| 448 | 20261574 | 216 | 1145 | 25 | 4 | 2 |
| 449 | 20261575 | 294 | 1654 | 27 | 6 | 3 |
| 450 | 20261576 | 276 | 1421 | 27 | 6 | 5 |
| 451 | 20261577 | 215 | 1003 | 22 | 4 | 4 |
| 452 | 20261578 | 426 | 2577 | 34 | 8 | 4 |
| 453 | 20261579 | 247 | 1184 | 22 | 4 | 4 |
| 454 | 20261580 | 424 | 2228 | 53 | 6 | 6 |
| 455 | 20261581 | 214 | 1329 | 16 | 6 | 2 |
| 456 | 20261582 | 203 | 1206 | 11 | 4 | 1 |
| 457 | 20261583 | 206 | 944 | 11 | 6 | 0 |
| 458 | 20261584 | 235 | 1264 | 20 | 8 | 2 |
| 459 | 20261585 | 218 | 969 | 28 | 4 | 1 |
| 460 | 20261586 | 495 | 2553 | 27 | 3 | 5 |
| 461 | 20261587 | 246 | 1320 | 41 | 3 | 1 |
| 462 | 20261588 | 218 | 978 | 18 | 1 | 2 |
| 463 | 20261589 | 292 | 1690 | 22 | 3 | 3 |
| 464 | 20261590 | 202 | 976 | 14 | 3 | 3 |
| 465 | 20261591 | 228 | 1160 | 28 | 1 | 2 |
| 466 | 20261592 | 249 | 1565 | 33 | 4 | 4 |
| 467 | 20261593 | 276 | 1459 | 30 | 6 | 5 |
| 468 | 20261594 | 234 | 1281 | 20 | 2 | 4 |
| 469 | 20261595 | 257 | 1460 | 28 | 4 | 2 |
| 470 | 20261596 | 337 | 1543 | 39 | 5 | 5 |
| 471 | 20261597 | 412 | 2748 | 35 | 1 | 3 |
| 472 | 20261598 | 455 | 2171 | 42 | 8 | 5 |
| 473 | 20261599 | 232 | 1144 | 25 | 1 | 4 |
| 474 | 20261600 | 269 | 1255 | 16 | 4 | 3 |
| 475 | 20261601 | 312 | 1341 | 22 | 6 | 2 |
| 476 | 20261602 | 191 | 772 | 16 | 1 | 0 |
| 477 | 20261603 | 337 | 1774 | 41 | 1 | 3 |
| 478 | 20261604 | 242 | 1172 | 15 | 5 | 5 |
| 479 | 20261605 | 297 | 1939 | 31 | 3 | 1 |
| 480 | 20261606 | 310 | 1699 | 33 | 7 | 4 |
| 481 | 20261607 | 214 | 904 | 19 | 6 | 2 |
| 482 | 20261608 | 254 | 1366 | 28 | 3 | 4 |
| 483 | 20261609 | 273 | 1148 | 26 | 4 | 1 |
| 484 | 20261610 | 211 | 1048 | 25 | 2 | 1 |
| 485 | 20261611 | 277 | 1302 | 29 | 10 | 5 |
| 486 | 20261612 | 183 | 753 | 15 | 2 | 2 |
| 487 | 20261613 | 379 | 1666 | 40 | 3 | 3 |
| 488 | 20261614 | 329 | 1608 | 30 | 5 | 2 |
| 489 | 20261615 | 199 | 724 | 12 | 7 | 1 |
| 490 | 20261616 | 240 | 1037 | 18 | 5 | 1 |
| 491 | 20261617 | 252 | 1208 | 16 | 2 | 4 |
| 492 | 20261618 | 261 | 1252 | 19 | 5 | 2 |
| 493 | 20261619 | 232 | 1087 | 31 | 5 | 3 |
| 494 | 20261620 | 243 | 1314 | 36 | 1 | 5 |
| 495 | 20261621 | 223 | 1247 | 12 | 0 | 1 |
| 496 | 20261622 | 179 | 857 | 15 | 2 | 2 |
| 497 | 20261623 | 233 | 1243 | 16 | 3 | 3 |
| 498 | 20261624 | 214 | 932 | 16 | 8 | 2 |
| 499 | 20261625 | 211 | 1096 | 27 | 4 | 2 |
| 500 | 20261626 | 276 | 990 | 28 | 0 | 3 |
| 501 | 20261627 | 255 | 1431 | 26 | 3 | 2 |
| 502 | 20261628 | 256 | 1169 | 22 | 0 | 1 |
| 503 | 20261629 | 267 | 1266 | 32 | 2 | 2 |
| 504 | 20261630 | 298 | 1659 | 34 | 7 | 3 |
| 505 | 20261631 | 226 | 1137 | 16 | 0 | 2 |
| 506 | 20261632 | 263 | 1092 | 26 | 1 | 5 |
| 507 | 20261633 | 283 | 1529 | 26 | 2 | 3 |
| 508 | 20261634 | 183 | 1042 | 19 | 2 | 1 |
| 509 | 20261635 | 301 | 1623 | 40 | 5 | 4 |
| 510 | 20261636 | 318 | 1598 | 38 | 2 | 3 |
| 511 | 20261637 | 243 | 1368 | 30 | 2 | 1 |
| 512 | 20261638 | 180 | 974 | 21 | 0 | 1 |
| 513 | 20261639 | 227 | 1098 | 21 | 5 | 3 |
| 514 | 20261640 | 200 | 1106 | 16 | 6 | 3 |
| 515 | 20261641 | 279 | 1818 | 22 | 2 | 4 |
| 516 | 20261642 | 251 | 1509 | 15 | 4 | 2 |
| 517 | 20261643 | 264 | 1418 | 13 | 6 | 5 |
| 518 | 20261644 | 217 | 834 | 7 | 2 | 3 |
| 519 | 20261645 | 262 | 1214 | 16 | 6 | 5 |
| 520 | 20261646 | 260 | 1235 | 24 | 10 | 4 |
| 521 | 20261647 | 212 | 840 | 25 | 5 | 2 |
| 522 | 20261648 | 249 | 1375 | 25 | 8 | 3 |
| 523 | 20261649 | 260 | 1253 | 23 | 11 | 0 |
| 524 | 20261650 | 207 | 969 | 22 | 3 | 2 |
| 525 | 20261651 | 260 | 1323 | 30 | 6 | 5 |
| 526 | 20261652 | 233 | 1399 | 11 | 5 | 4 |
| 527 | 20261653 | 205 | 1011 | 15 | 0 | 2 |
| 528 | 20261654 | 297 | 1443 | 18 | 4 | 2 |
| 529 | 20261655 | 297 | 1609 | 48 | 4 | 5 |
| 530 | 20261656 | 289 | 1355 | 19 | 12 | 4 |
| 531 | 20261657 | 406 | 2244 | 34 | 3 | 4 |
| 532 | 20261658 | 247 | 1248 | 32 | 11 | 3 |
| 533 | 20261659 | 255 | 1220 | 23 | 1 | 1 |
| 534 | 20261660 | 271 | 1360 | 16 | 10 | 1 |
| 535 | 20261661 | 198 | 834 | 16 | 4 | 1 |
| 536 | 20261662 | 211 | 1029 | 25 | 2 | 2 |
| 537 | 20261663 | 201 | 1096 | 23 | 6 | 2 |
| 538 | 20261664 | 180 | 850 | 13 | 4 | 2 |
| 539 | 20261665 | 242 | 1177 | 24 | 7 | 3 |
| 540 | 20261666 | 210 | 966 | 17 | 5 | 1 |
| 541 | 20261667 | 230 | 1109 | 34 | 1 | 4 |
| 542 | 20261668 | 214 | 688 | 12 | 3 | 1 |
| 543 | 20261669 | 169 | 784 | 16 | 4 | 1 |
| 544 | 20261670 | 227 | 957 | 19 | 8 | 4 |
| 545 | 20261671 | 212 | 1156 | 19 | 5 | 2 |
| 546 | 20261672 | 253 | 911 | 27 | 4 | 2 |
| 547 | 20261673 | 278 | 1642 | 35 | 6 | 4 |
| 548 | 20261674 | 270 | 1406 | 34 | 4 | 2 |
| 549 | 20261675 | 208 | 1041 | 10 | 3 | 3 |
| 550 | 20261676 | 240 | 1470 | 14 | 5 | 3 |
| 551 | 20261677 | 278 | 1261 | 27 | 0 | 3 |
| 552 | 20261678 | 222 | 1094 | 17 | 5 | 1 |
| 553 | 20261679 | 261 | 1111 | 19 | 6 | 2 |
| 554 | 20261680 | 261 | 1492 | 32 | 5 | 1 |
| 555 | 20261681 | 199 | 824 | 20 | 4 | 1 |
| 556 | 20261682 | 260 | 945 | 20 | 8 | 2 |
| 557 | 20261683 | 220 | 1219 | 17 | 5 | 3 |
| 558 | 20261684 | 390 | 2566 | 57 | 4 | 2 |
| 559 | 20261685 | 329 | 2000 | 23 | 5 | 6 |
| 560 | 20261686 | 198 | 1118 | 29 | 1 | 2 |
| 561 | 20261687 | 258 | 1457 | 30 | 3 | 2 |
| 562 | 20261688 | 321 | 1676 | 29 | 3 | 2 |
| 563 | 20261689 | 215 | 1384 | 24 | 2 | 3 |
| 564 | 20261690 | 180 | 681 | 13 | 5 | 1 |
| 565 | 20261691 | 279 | 1028 | 29 | 5 | 5 |
| 566 | 20261692 | 297 | 1673 | 48 | 9 | 4 |
| 567 | 20261693 | 204 | 1015 | 21 | 2 | 3 |
| 568 | 20261694 | 169 | 574 | 14 | 2 | 3 |
| 569 | 20261695 | 271 | 994 | 24 | 4 | 4 |
| 570 | 20261696 | 273 | 1464 | 26 | 4 | 4 |
| 571 | 20261697 | 264 | 1610 | 28 | 2 | 4 |
| 572 | 20261698 | 209 | 932 | 20 | 7 | 2 |
| 573 | 20261699 | 179 | 714 | 19 | 2 | 3 |
| 574 | 20261700 | 180 | 897 | 14 | 2 | 1 |
| 575 | 20261701 | 301 | 1451 | 16 | 2 | 3 |
| 576 | 20261702 | 346 | 1819 | 37 | 1 | 2 |
| 577 | 20261703 | 194 | 1033 | 17 | 1 | 1 |
| 578 | 20261704 | 234 | 1085 | 17 | 3 | 4 |
| 579 | 20261705 | 234 | 1196 | 25 | 3 | 2 |
| 580 | 20261706 | 307 | 1732 | 42 | 3 | 3 |
| 581 | 20261707 | 196 | 844 | 25 | 2 | 3 |
| 582 | 20261708 | 260 | 1396 | 19 | 4 | 3 |
| 583 | 20261709 | 283 | 1642 | 39 | 2 | 2 |
| 584 | 20261710 | 285 | 1231 | 26 | 9 | 4 |
| 585 | 20261711 | 266 | 1639 | 32 | 4 | 2 |
| 586 | 20261712 | 184 | 824 | 18 | 5 | 2 |
| 587 | 20261713 | 294 | 1454 | 25 | 2 | 3 |
| 588 | 20261714 | 186 | 1018 | 22 | 1 | 4 |
| 589 | 20261715 | 200 | 755 | 11 | 4 | 2 |
| 590 | 20261716 | 293 | 1523 | 16 | 4 | 4 |
| 591 | 20261717 | 222 | 1178 | 15 | 3 | 3 |
| 592 | 20261718 | 225 | 1114 | 21 | 5 | 2 |
| 593 | 20261719 | 254 | 1569 | 26 | 3 | 2 |
| 594 | 20261720 | 249 | 1417 | 27 | 7 | 3 |
| 595 | 20261721 | 183 | 831 | 15 | 4 | 3 |
| 596 | 20261722 | 258 | 1109 | 31 | 5 | 2 |
| 597 | 20261723 | 259 | 1237 | 28 | 5 | 2 |
| 598 | 20261724 | 329 | 2351 | 25 | 4 | 5 |
| 599 | 20261725 | 306 | 1772 | 30 | 6 | 2 |
| 600 | 20261726 | 266 | 1238 | 41 | 5 | 2 |
| 601 | 20261727 | 222 | 869 | 19 | 7 | 3 |
| 602 | 20261728 | 221 | 1066 | 18 | 4 | 2 |
| 603 | 20261729 | 224 | 1101 | 24 | 2 | 3 |
| 604 | 20261730 | 245 | 1169 | 30 | 3 | 4 |
| 605 | 20261731 | 212 | 1193 | 21 | 5 | 2 |
| 606 | 20261732 | 301 | 1656 | 33 | 3 | 2 |
| 607 | 20261733 | 274 | 1464 | 32 | 2 | 1 |
| 608 | 20261734 | 209 | 716 | 17 | 4 | 1 |
| 609 | 20261735 | 233 | 1158 | 23 | 6 | 2 |
| 610 | 20261736 | 177 | 486 | 6 | 1 | 2 |
| 611 | 20261737 | 258 | 1287 | 22 | 2 | 4 |
| 612 | 20261738 | 309 | 1943 | 34 | 3 | 2 |
| 613 | 20261739 | 250 | 1400 | 23 | 7 | 1 |
| 614 | 20261740 | 232 | 1232 | 21 | 0 | 3 |
| 615 | 20261741 | 305 | 1615 | 29 | 9 | 2 |
| 616 | 20261742 | 256 | 1378 | 23 | 1 | 2 |
| 617 | 20261743 | 317 | 2014 | 30 | 4 | 2 |
| 618 | 20261744 | 189 | 815 | 16 | 3 | 4 |
| 619 | 20261745 | 246 | 1288 | 22 | 2 | 2 |
| 620 | 20261746 | 249 | 1097 | 13 | 3 | 2 |
| 621 | 20261747 | 241 | 1071 | 16 | 6 | 3 |
| 622 | 20261748 | 236 | 1051 | 33 | 10 | 2 |
| 623 | 20261749 | 415 | 2781 | 29 | 9 | 3 |
| 624 | 20261750 | 203 | 971 | 20 | 2 | 3 |
| 625 | 20261751 | 272 | 1350 | 24 | 3 | 3 |
| 626 | 20261752 | 245 | 1422 | 22 | 2 | 2 |
| 627 | 20261753 | 273 | 1368 | 13 | 7 | 2 |
| 628 | 20261754 | 230 | 1053 | 10 | 3 | 5 |
| 629 | 20261755 | 212 | 1240 | 23 | 2 | 1 |
| 630 | 20261756 | 184 | 875 | 13 | 1 | 2 |
| 631 | 20261757 | 254 | 1339 | 26 | 5 | 2 |
| 632 | 20261758 | 239 | 1046 | 18 | 6 | 0 |
| 633 | 20261759 | 257 | 1054 | 18 | 7 | 4 |
| 634 | 20261760 | 232 | 895 | 15 | 0 | 3 |
| 635 | 20261761 | 228 | 1186 | 32 | 1 | 3 |
| 636 | 20261762 | 208 | 660 | 11 | 2 | 3 |
| 637 | 20261763 | 162 | 630 | 15 | 2 | 3 |
| 638 | 20261764 | 269 | 1459 | 31 | 2 | 2 |
| 639 | 20261765 | 291 | 1188 | 29 | 11 | 2 |
| 640 | 20261766 | 356 | 1764 | 26 | 1 | 3 |
| 641 | 20261767 | 273 | 1749 | 34 | 3 | 2 |
| 642 | 20261768 | 248 | 1194 | 20 | 6 | 2 |
| 643 | 20261769 | 230 | 1329 | 28 | 4 | 2 |
| 644 | 20261770 | 286 | 1442 | 24 | 7 | 4 |
| 645 | 20261771 | 233 | 1006 | 19 | 3 | 4 |
| 646 | 20261772 | 243 | 1550 | 21 | 8 | 2 |
| 647 | 20261773 | 226 | 1204 | 24 | 3 | 1 |
| 648 | 20261774 | 222 | 1311 | 29 | 2 | 3 |
| 649 | 20261775 | 241 | 964 | 14 | 2 | 6 |
| 650 | 20261776 | 279 | 1590 | 34 | 4 | 4 |
| 651 | 20261777 | 245 | 1014 | 30 | 3 | 1 |
| 652 | 20261778 | 181 | 705 | 17 | 2 | 2 |
| 653 | 20261779 | 185 | 985 | 24 | 2 | 1 |
| 654 | 20261780 | 277 | 1489 | 23 | 6 | 4 |
| 655 | 20261781 | 229 | 1290 | 30 | 3 | 3 |
| 656 | 20261782 | 253 | 1131 | 30 | 5 | 5 |
| 657 | 20261783 | 280 | 1757 | 28 | 14 | 2 |
| 658 | 20261784 | 244 | 1315 | 21 | 5 | 1 |
| 659 | 20261785 | 269 | 1260 | 31 | 8 | 5 |
| 660 | 20261786 | 187 | 916 | 19 | 3 | 2 |
| 661 | 20261787 | 276 | 1489 | 33 | 3 | 1 |
| 662 | 20261788 | 218 | 1188 | 28 | 3 | 2 |
| 663 | 20261789 | 235 | 1058 | 22 | 5 | 3 |
| 664 | 20261790 | 302 | 1414 | 35 | 3 | 4 |
| 665 | 20261791 | 239 | 1194 | 18 | 3 | 3 |
| 666 | 20261792 | 278 | 1201 | 23 | 4 | 4 |
| 667 | 20261793 | 292 | 1343 | 27 | 3 | 1 |
| 668 | 20261794 | 500 | 2453 | 66 | 9 | 5 |
| 669 | 20261795 | 224 | 1172 | 12 | 6 | 5 |
| 670 | 20261796 | 274 | 1518 | 22 | 11 | 4 |
| 671 | 20261797 | 226 | 1252 | 14 | 3 | 1 |
| 672 | 20261798 | 245 | 1221 | 21 | 4 | 2 |
| 673 | 20261799 | 186 | 669 | 17 | 4 | 1 |
| 674 | 20261800 | 225 | 1054 | 8 | 5 | 3 |
| 675 | 20261801 | 178 | 725 | 13 | 3 | 2 |
| 676 | 20261802 | 247 | 1391 | 37 | 3 | 2 |
| 677 | 20261803 | 230 | 1360 | 18 | 2 | 2 |
| 678 | 20261804 | 193 | 783 | 14 | 4 | 0 |
| 679 | 20261805 | 229 | 1137 | 14 | 2 | 3 |
| 680 | 20261806 | 176 | 850 | 11 | 0 | 3 |
| 681 | 20261807 | 351 | 2014 | 49 | 6 | 5 |
| 682 | 20261808 | 259 | 1652 | 30 | 7 | 0 |
| 683 | 20261809 | 367 | 2114 | 45 | 6 | 3 |
| 684 | 20261810 | 217 | 861 | 27 | 3 | 2 |
| 685 | 20261811 | 361 | 1759 | 36 | 4 | 3 |
| 686 | 20261812 | 228 | 829 | 21 | 7 | 2 |
| 687 | 20261813 | 263 | 1467 | 33 | 3 | 4 |
| 688 | 20261814 | 266 | 1218 | 32 | 2 | 4 |
| 689 | 20261815 | 220 | 999 | 19 | 1 | 2 |
| 690 | 20261816 | 254 | 1367 | 32 | 3 | 3 |
| 691 | 20261817 | 284 | 1592 | 33 | 1 | 3 |
| 692 | 20261818 | 277 | 1559 | 15 | 4 | 2 |
| 693 | 20261819 | 311 | 1712 | 29 | 2 | 4 |
| 694 | 20261820 | 226 | 842 | 15 | 5 | 2 |
| 695 | 20261821 | 162 | 614 | 13 | 4 | 2 |
| 696 | 20261822 | 348 | 1712 | 37 | 4 | 5 |
| 697 | 20261823 | 256 | 1006 | 38 | 8 | 5 |
| 698 | 20261824 | 196 | 710 | 15 | 5 | 2 |
| 699 | 20261825 | 258 | 1339 | 26 | 4 | 4 |
| 700 | 20261826 | 246 | 1354 | 32 | 3 | 4 |
| 701 | 20261827 | 281 | 1228 | 33 | 2 | 4 |
| 702 | 20261828 | 365 | 1881 | 36 | 4 | 2 |
| 703 | 20261829 | 319 | 1413 | 19 | 4 | 3 |
| 704 | 20261830 | 244 | 1303 | 28 | 7 | 1 |
| 705 | 20261831 | 263 | 1265 | 32 | 6 | 3 |
| 706 | 20261832 | 280 | 1482 | 24 | 2 | 2 |
| 707 | 20261833 | 173 | 905 | 11 | 2 | 2 |
| 708 | 20261834 | 240 | 1099 | 14 | 7 | 0 |
| 709 | 20261835 | 221 | 914 | 12 | 5 | 4 |
| 710 | 20261836 | 251 | 1084 | 24 | 0 | 1 |
| 711 | 20261837 | 210 | 1174 | 25 | 1 | 2 |
| 712 | 20261838 | 174 | 600 | 10 | 3 | 1 |
| 713 | 20261839 | 275 | 1468 | 38 | 2 | 4 |
| 714 | 20261840 | 232 | 982 | 11 | 1 | 4 |
| 715 | 20261841 | 209 | 1195 | 17 | 3 | 2 |
| 716 | 20261842 | 276 | 1507 | 24 | 5 | 2 |
| 717 | 20261843 | 257 | 1257 | 20 | 13 | 3 |
| 718 | 20261844 | 277 | 1538 | 28 | 3 | 2 |
| 719 | 20261845 | 310 | 1685 | 35 | 4 | 3 |
| 720 | 20261846 | 190 | 748 | 15 | 2 | 2 |
| 721 | 20261847 | 189 | 989 | 17 | 2 | 2 |
| 722 | 20261848 | 190 | 842 | 17 | 2 | 3 |
| 723 | 20261849 | 242 | 1232 | 35 | 7 | 1 |
| 724 | 20261850 | 244 | 1383 | 27 | 6 | 3 |
| 725 | 20261851 | 233 | 1301 | 34 | 5 | 2 |
| 726 | 20261852 | 251 | 1179 | 23 | 6 | 2 |
| 727 | 20261853 | 347 | 1491 | 31 | 2 | 3 |
| 728 | 20261854 | 315 | 2042 | 41 | 4 | 2 |
| 729 | 20261855 | 269 | 1938 | 40 | 6 | 0 |
| 730 | 20261856 | 202 | 905 | 19 | 4 | 1 |
| 731 | 20261857 | 195 | 914 | 12 | 3 | 1 |
| 732 | 20261858 | 244 | 1410 | 17 | 1 | 3 |
| 733 | 20261859 | 215 | 1126 | 12 | 2 | 2 |
| 734 | 20261860 | 203 | 1000 | 23 | 6 | 2 |
| 735 | 20261861 | 230 | 1318 | 22 | 3 | 3 |
| 736 | 20261862 | 332 | 1871 | 39 | 11 | 4 |
| 737 | 20261863 | 342 | 1883 | 29 | 2 | 6 |
| 738 | 20261864 | 267 | 1319 | 27 | 7 | 4 |
| 739 | 20261865 | 274 | 1502 | 38 | 3 | 3 |
| 740 | 20261866 | 250 | 1200 | 23 | 3 | 1 |
| 741 | 20261867 | 260 | 1440 | 25 | 5 | 2 |
| 742 | 20261868 | 175 | 712 | 12 | 5 | 0 |
| 743 | 20261869 | 311 | 1512 | 36 | 3 | 3 |
| 744 | 20261870 | 353 | 1516 | 39 | 1 | 3 |
| 745 | 20261871 | 261 | 1606 | 27 | 4 | 3 |
| 746 | 20261872 | 240 | 1239 | 20 | 2 | 2 |
| 747 | 20261873 | 214 | 1036 | 23 | 3 | 1 |
| 748 | 20261874 | 332 | 2194 | 23 | 5 | 4 |
| 749 | 20261875 | 326 | 1635 | 38 | 5 | 2 |
| 750 | 20261876 | 301 | 1540 | 41 | 1 | 3 |
| 751 | 20261877 | 274 | 1548 | 31 | 6 | 4 |
| 752 | 20261878 | 189 | 844 | 12 | 9 | 1 |
| 753 | 20261879 | 290 | 1674 | 27 | 4 | 4 |
| 754 | 20261880 | 229 | 1362 | 22 | 3 | 1 |
| 755 | 20261881 | 196 | 1030 | 19 | 4 | 3 |
| 756 | 20261882 | 189 | 805 | 11 | 0 | 1 |
| 757 | 20261883 | 215 | 1114 | 27 | 1 | 3 |
| 758 | 20261884 | 199 | 925 | 21 | 6 | 2 |
| 759 | 20261885 | 223 | 1357 | 26 | 8 | 2 |
| 760 | 20261886 | 219 | 1182 | 27 | 5 | 2 |
| 761 | 20261887 | 239 | 1100 | 17 | 3 | 2 |
| 762 | 20261888 | 226 | 1060 | 22 | 2 | 2 |
| 763 | 20261889 | 195 | 1089 | 18 | 2 | 1 |
| 764 | 20261890 | 318 | 1544 | 21 | 5 | 2 |
| 765 | 20261891 | 276 | 1440 | 36 | 6 | 1 |
| 766 | 20261892 | 224 | 886 | 20 | 3 | 1 |
| 767 | 20261893 | 200 | 1110 | 29 | 4 | 2 |
| 768 | 20261894 | 227 | 1096 | 10 | 8 | 3 |
| 769 | 20261895 | 248 | 1356 | 24 | 3 | 2 |
| 770 | 20261896 | 186 | 772 | 15 | 3 | 1 |
| 771 | 20261897 | 226 | 1081 | 22 | 3 | 2 |
| 772 | 20261898 | 339 | 1807 | 22 | 4 | 1 |
| 773 | 20261899 | 314 | 1808 | 24 | 0 | 2 |
| 774 | 20261900 | 288 | 1193 | 20 | 9 | 5 |
| 775 | 20261901 | 281 | 1176 | 37 | 4 | 4 |
| 776 | 20261902 | 230 | 1279 | 12 | 14 | 1 |
| 777 | 20261903 | 333 | 1678 | 40 | 4 | 4 |
| 778 | 20261904 | 378 | 2290 | 68 | 6 | 4 |
| 779 | 20261905 | 207 | 928 | 22 | 2 | 2 |
| 780 | 20261906 | 227 | 1123 | 16 | 5 | 4 |
| 781 | 20261907 | 184 | 844 | 17 | 8 | 3 |
| 782 | 20261908 | 279 | 1277 | 7 | 11 | 1 |
| 783 | 20261909 | 215 | 1177 | 15 | 2 | 0 |
| 784 | 20261910 | 277 | 1810 | 27 | 2 | 3 |
| 785 | 20261911 | 245 | 1211 | 29 | 1 | 4 |
| 786 | 20261912 | 235 | 1116 | 20 | 1 | 2 |
| 787 | 20261913 | 218 | 680 | 11 | 11 | 1 |
| 788 | 20261914 | 205 | 990 | 16 | 2 | 2 |
| 789 | 20261915 | 217 | 866 | 22 | 8 | 3 |
| 790 | 20261916 | 213 | 977 | 19 | 2 | 1 |
| 791 | 20261917 | 272 | 1394 | 24 | 3 | 3 |
| 792 | 20261918 | 224 | 1022 | 15 | 3 | 1 |
| 793 | 20261919 | 244 | 1246 | 35 | 8 | 1 |
| 794 | 20261920 | 251 | 1409 | 29 | 0 | 4 |
| 795 | 20261921 | 238 | 1043 | 18 | 3 | 2 |
| 796 | 20261922 | 161 | 838 | 13 | 5 | 1 |
| 797 | 20261923 | 242 | 1402 | 14 | 3 | 4 |
| 798 | 20261924 | 190 | 1168 | 17 | 4 | 0 |
| 799 | 20261925 | 259 | 1254 | 32 | 1 | 3 |
| 800 | 20261926 | 294 | 1557 | 25 | 2 | 3 |
| 801 | 20261927 | 249 | 1083 | 31 | 4 | 4 |
| 802 | 20261928 | 212 | 889 | 15 | 2 | 2 |
| 803 | 20261929 | 225 | 1027 | 18 | 0 | 2 |
| 804 | 20261930 | 271 | 1274 | 17 | 2 | 4 |
| 805 | 20261931 | 266 | 1190 | 22 | 5 | 4 |
| 806 | 20261932 | 286 | 1540 | 20 | 2 | 5 |
| 807 | 20261933 | 209 | 1171 | 28 | 11 | 2 |
| 808 | 20261934 | 242 | 1039 | 13 | 4 | 4 |
| 809 | 20261935 | 256 | 1256 | 32 | 4 | 3 |
| 810 | 20261936 | 303 | 1580 | 43 | 6 | 4 |
| 811 | 20261937 | 333 | 1875 | 44 | 1 | 3 |
| 812 | 20261938 | 241 | 1387 | 20 | 2 | 3 |
| 813 | 20261939 | 206 | 752 | 23 | 2 | 3 |
| 814 | 20261940 | 278 | 1186 | 17 | 3 | 1 |
| 815 | 20261941 | 237 | 1225 | 28 | 7 | 2 |
| 816 | 20261942 | 296 | 1692 | 36 | 7 | 4 |
| 817 | 20261943 | 206 | 922 | 27 | 1 | 4 |
| 818 | 20261944 | 244 | 1381 | 18 | 2 | 3 |
| 819 | 20261945 | 234 | 1114 | 16 | 4 | 4 |
| 820 | 20261946 | 249 | 1180 | 24 | 7 | 3 |
| 821 | 20261947 | 172 | 607 | 6 | 3 | 2 |
| 822 | 20261948 | 228 | 1393 | 28 | 3 | 3 |
| 823 | 20261949 | 210 | 1196 | 21 | 6 | 3 |
| 824 | 20261950 | 237 | 1119 | 28 | 2 | 2 |
| 825 | 20261951 | 295 | 1342 | 38 | 6 | 4 |
| 826 | 20261952 | 213 | 832 | 14 | 1 | 2 |
| 827 | 20261953 | 211 | 1085 | 20 | 8 | 3 |
| 828 | 20261954 | 190 | 1042 | 17 | 3 | 3 |
| 829 | 20261955 | 283 | 1555 | 36 | 6 | 3 |
| 830 | 20261956 | 224 | 1043 | 25 | 4 | 5 |
| 831 | 20261957 | 179 | 895 | 18 | 1 | 3 |
| 832 | 20261958 | 222 | 1140 | 25 | 0 | 3 |
| 833 | 20261959 | 190 | 909 | 20 | 5 | 2 |
| 834 | 20261960 | 262 | 1177 | 16 | 2 | 2 |
| 835 | 20261961 | 366 | 1864 | 27 | 3 | 3 |
| 836 | 20261962 | 254 | 1101 | 24 | 5 | 3 |
| 837 | 20261963 | 237 | 1098 | 21 | 5 | 3 |
| 838 | 20261964 | 230 | 1461 | 39 | 5 | 1 |
| 839 | 20261965 | 228 | 1009 | 21 | 15 | 3 |
| 840 | 20261966 | 224 | 1197 | 18 | 2 | 0 |
| 841 | 20261967 | 186 | 663 | 8 | 3 | 1 |
| 842 | 20261968 | 230 | 1389 | 28 | 8 | 3 |
| 843 | 20261969 | 250 | 1605 | 30 | 3 | 1 |
| 844 | 20261970 | 200 | 1134 | 20 | 4 | 3 |
| 845 | 20261971 | 213 | 1098 | 16 | 3 | 3 |
| 846 | 20261972 | 269 | 1837 | 26 | 6 | 1 |
| 847 | 20261973 | 263 | 1348 | 22 | 5 | 4 |
| 848 | 20261974 | 344 | 2022 | 32 | 4 | 3 |
| 849 | 20261975 | 252 | 1260 | 21 | 6 | 2 |
| 850 | 20261976 | 175 | 637 | 14 | 5 | 1 |
| 851 | 20261977 | 288 | 1787 | 28 | 4 | 2 |
| 852 | 20261978 | 410 | 2892 | 53 | 20 | 2 |
| 853 | 20261979 | 222 | 852 | 15 | 1 | 3 |
| 854 | 20261980 | 193 | 800 | 14 | 8 | 0 |
| 855 | 20261981 | 189 | 900 | 17 | 5 | 2 |
| 856 | 20261982 | 184 | 613 | 17 | 5 | 1 |
| 857 | 20261983 | 192 | 871 | 19 | 7 | 2 |
| 858 | 20261984 | 175 | 628 | 14 | 0 | 2 |
| 859 | 20261985 | 198 | 851 | 15 | 7 | 0 |
| 860 | 20261986 | 340 | 1684 | 14 | 8 | 5 |
| 861 | 20261987 | 231 | 1145 | 21 | 6 | 3 |
| 862 | 20261988 | 262 | 1659 | 31 | 4 | 2 |
| 863 | 20261989 | 242 | 1172 | 30 | 6 | 2 |
| 864 | 20261990 | 212 | 947 | 21 | 2 | 4 |
| 865 | 20261991 | 292 | 1577 | 29 | 2 | 3 |
| 866 | 20261992 | 244 | 1337 | 35 | 0 | 4 |
| 867 | 20261993 | 202 | 747 | 21 | 5 | 2 |
| 868 | 20261994 | 258 | 1278 | 9 | 3 | 4 |
| 869 | 20261995 | 281 | 1395 | 13 | 0 | 1 |
| 870 | 20261996 | 207 | 1063 | 16 | 5 | 2 |
| 871 | 20261997 | 213 | 1211 | 20 | 8 | 0 |
| 872 | 20261998 | 232 | 1195 | 23 | 5 | 4 |
| 873 | 20261999 | 214 | 967 | 13 | 3 | 2 |
| 874 | 20262000 | 191 | 1116 | 22 | 1 | 3 |
| 875 | 20262001 | 221 | 1067 | 21 | 4 | 1 |
| 876 | 20262002 | 279 | 1055 | 17 | 10 | 3 |
| 877 | 20262003 | 202 | 962 | 16 | 3 | 1 |
| 878 | 20262004 | 236 | 981 | 15 | 12 | 1 |
| 879 | 20262005 | 366 | 2331 | 36 | 4 | 2 |
| 880 | 20262006 | 192 | 784 | 9 | 3 | 3 |
| 881 | 20262007 | 230 | 1103 | 37 | 9 | 1 |
| 882 | 20262008 | 188 | 994 | 14 | 1 | 2 |
| 883 | 20262009 | 500 | 2328 | 67 | 6 | 6 |
| 884 | 20262010 | 287 | 1388 | 12 | 9 | 3 |
| 885 | 20262011 | 250 | 1142 | 15 | 4 | 1 |
| 886 | 20262012 | 244 | 911 | 23 | 5 | 3 |
| 887 | 20262013 | 277 | 1189 | 25 | 5 | 4 |
| 888 | 20262014 | 226 | 1231 | 29 | 3 | 2 |
| 889 | 20262015 | 343 | 1834 | 37 | 2 | 2 |
| 890 | 20262016 | 230 | 1165 | 22 | 1 | 3 |
| 891 | 20262017 | 359 | 1861 | 28 | 5 | 4 |
| 892 | 20262018 | 262 | 1374 | 18 | 1 | 4 |
| 893 | 20262019 | 235 | 1336 | 25 | 2 | 3 |
| 894 | 20262020 | 182 | 805 | 12 | 8 | 3 |
| 895 | 20262021 | 250 | 1419 | 24 | 7 | 2 |
| 896 | 20262022 | 225 | 969 | 27 | 4 | 3 |
| 897 | 20262023 | 196 | 990 | 27 | 5 | 1 |
| 898 | 20262024 | 434 | 2221 | 38 | 7 | 2 |
| 899 | 20262025 | 346 | 1521 | 28 | 6 | 5 |
| 900 | 20262026 | 219 | 834 | 20 | 0 | 3 |
| 901 | 20262027 | 343 | 1670 | 26 | 9 | 3 |
| 902 | 20262028 | 209 | 757 | 13 | 3 | 3 |
| 903 | 20262029 | 310 | 1498 | 27 | 3 | 3 |
| 904 | 20262030 | 253 | 1267 | 31 | 6 | 2 |
| 905 | 20262031 | 303 | 1940 | 25 | 2 | 6 |
| 906 | 20262032 | 275 | 1205 | 22 | 7 | 3 |
| 907 | 20262033 | 254 | 1176 | 34 | 4 | 2 |
| 908 | 20262034 | 242 | 1211 | 25 | 8 | 3 |
| 909 | 20262035 | 325 | 1995 | 29 | 0 | 2 |
| 910 | 20262036 | 226 | 1161 | 29 | 5 | 3 |
| 911 | 20262037 | 237 | 1375 | 26 | 4 | 3 |
| 912 | 20262038 | 232 | 1241 | 34 | 1 | 1 |
| 913 | 20262039 | 246 | 1242 | 25 | 1 | 1 |
| 914 | 20262040 | 277 | 1594 | 21 | 10 | 4 |
| 915 | 20262041 | 171 | 805 | 23 | 3 | 2 |
| 916 | 20262042 | 262 | 1521 | 27 | 7 | 4 |
| 917 | 20262043 | 290 | 1643 | 46 | 1 | 2 |
| 918 | 20262044 | 238 | 1156 | 24 | 2 | 5 |
| 919 | 20262045 | 266 | 1353 | 38 | 4 | 5 |
| 920 | 20262046 | 351 | 2122 | 35 | 4 | 2 |
| 921 | 20262047 | 240 | 942 | 12 | 4 | 2 |
| 922 | 20262048 | 227 | 825 | 22 | 2 | 2 |
| 923 | 20262049 | 283 | 1127 | 25 | 7 | 2 |
| 924 | 20262050 | 193 | 931 | 19 | 4 | 2 |
| 925 | 20262051 | 218 | 892 | 22 | 1 | 2 |
| 926 | 20262052 | 239 | 1261 | 22 | 3 | 2 |
| 927 | 20262053 | 191 | 1090 | 12 | 2 | 3 |
| 928 | 20262054 | 188 | 820 | 10 | 2 | 1 |
| 929 | 20262055 | 167 | 821 | 10 | 2 | 2 |
| 930 | 20262056 | 340 | 1590 | 40 | 5 | 3 |
| 931 | 20262057 | 253 | 917 | 22 | 1 | 3 |
| 932 | 20262058 | 237 | 1283 | 37 | 2 | 1 |
| 933 | 20262059 | 202 | 936 | 23 | 2 | 2 |
| 934 | 20262060 | 290 | 1688 | 36 | 2 | 3 |
| 935 | 20262061 | 230 | 955 | 20 | 1 | 4 |
| 936 | 20262062 | 234 | 1101 | 17 | 4 | 3 |
| 937 | 20262063 | 233 | 1154 | 33 | 2 | 4 |
| 938 | 20262064 | 493 | 2876 | 51 | 2 | 4 |
| 939 | 20262065 | 240 | 1210 | 31 | 4 | 3 |
| 940 | 20262066 | 254 | 1477 | 27 | 2 | 3 |
| 941 | 20262067 | 294 | 2259 | 35 | 0 | 2 |
| 942 | 20262068 | 278 | 1246 | 26 | 9 | 4 |
| 943 | 20262069 | 219 | 924 | 22 | 5 | 2 |
| 944 | 20262070 | 255 | 1367 | 26 | 3 | 1 |
| 945 | 20262071 | 219 | 1280 | 18 | 1 | 2 |
| 946 | 20262072 | 189 | 921 | 19 | 5 | 4 |
| 947 | 20262073 | 355 | 2285 | 30 | 3 | 2 |
| 948 | 20262074 | 248 | 1367 | 27 | 1 | 3 |
| 949 | 20262075 | 263 | 1171 | 17 | 10 | 2 |
| 950 | 20262076 | 288 | 1483 | 35 | 9 | 5 |
| 951 | 20262077 | 226 | 1317 | 24 | 4 | 2 |
| 952 | 20262078 | 214 | 746 | 12 | 4 | 4 |
| 953 | 20262079 | 212 | 1011 | 24 | 5 | 2 |
| 954 | 20262080 | 292 | 1795 | 33 | 2 | 4 |
| 955 | 20262081 | 181 | 865 | 10 | 5 | 3 |
| 956 | 20262082 | 263 | 1166 | 18 | 16 | 3 |
| 957 | 20262083 | 234 | 1057 | 19 | 7 | 3 |
| 958 | 20262084 | 235 | 1291 | 20 | 5 | 3 |
| 959 | 20262085 | 262 | 1347 | 15 | 13 | 4 |
| 960 | 20262086 | 309 | 1765 | 26 | 10 | 2 |
| 961 | 20262087 | 185 | 921 | 21 | 1 | 3 |
| 962 | 20262088 | 191 | 973 | 21 | 3 | 3 |
| 963 | 20262089 | 253 | 1114 | 14 | 3 | 3 |
| 964 | 20262090 | 244 | 1328 | 15 | 2 | 3 |
| 965 | 20262091 | 275 | 1296 | 12 | 3 | 3 |
| 966 | 20262092 | 265 | 1499 | 32 | 3 | 3 |
| 967 | 20262093 | 252 | 1371 | 21 | 1 | 2 |
| 968 | 20262094 | 219 | 1082 | 27 | 8 | 3 |
| 969 | 20262095 | 288 | 1332 | 36 | 7 | 5 |
| 970 | 20262096 | 253 | 975 | 29 | 1 | 4 |
| 971 | 20262097 | 275 | 1211 | 20 | 0 | 3 |
| 972 | 20262098 | 305 | 1944 | 31 | 5 | 4 |
| 973 | 20262099 | 330 | 1512 | 29 | 1 | 3 |
| 974 | 20262100 | 217 | 1161 | 33 | 3 | 2 |
| 975 | 20262101 | 297 | 1140 | 22 | 2 | 2 |
| 976 | 20262102 | 257 | 1299 | 32 | 6 | 3 |
| 977 | 20262103 | 186 | 869 | 15 | 5 | 1 |
| 978 | 20262104 | 185 | 1078 | 11 | 3 | 0 |
| 979 | 20262105 | 211 | 1059 | 23 | 2 | 2 |
| 980 | 20262106 | 243 | 1236 | 15 | 6 | 1 |
| 981 | 20262107 | 230 | 1109 | 19 | 7 | 4 |
| 982 | 20262108 | 198 | 816 | 18 | 1 | 3 |
| 983 | 20262109 | 229 | 1020 | 21 | 5 | 0 |
| 984 | 20262110 | 170 | 926 | 13 | 3 | 1 |
| 985 | 20262111 | 292 | 1515 | 38 | 4 | 4 |
| 986 | 20262112 | 231 | 1163 | 22 | 6 | 4 |
| 987 | 20262113 | 235 | 1287 | 18 | 9 | 1 |
| 988 | 20262114 | 306 | 1794 | 21 | 6 | 1 |
| 989 | 20262115 | 227 | 1156 | 18 | 1 | 2 |
| 990 | 20262116 | 476 | 2212 | 37 | 10 | 4 |
| 991 | 20262117 | 281 | 1360 | 30 | 2 | 4 |
| 992 | 20262118 | 278 | 1442 | 33 | 7 | 2 |
| 993 | 20262119 | 212 | 819 | 8 | 5 | 2 |
| 994 | 20262120 | 191 | 920 | 14 | 2 | 1 |
| 995 | 20262121 | 203 | 963 | 22 | 2 | 3 |
| 996 | 20262122 | 240 | 1313 | 38 | 3 | 2 |
| 997 | 20262123 | 230 | 1271 | 18 | 5 | 2 |
| 998 | 20262124 | 306 | 1226 | 21 | 2 | 3 |
| 999 | 20262125 | 309 | 1657 | 21 | 6 | 4 |

## Throughput

- Turns simulated: 251886 (first play of each game)
- Aggregate: 121.3 turns/s over 2076.7s wall
- Per game thread: 30.4 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

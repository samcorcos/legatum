# Simulation: india-office-upkeep (2026-11-26)

1000 games, first seed 20261126, brain `scripted`, turn limit off, safety cap 1200, 6 threads, started 2026-11-26T23:00:54Z. Wall time 708.4s.

Source checkout at run start: `0c5d2e8f05b65f08db2319b210af40dfe36974b9`; dirty: yes. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--games","1000","--seed","20261126","--threads","6","--difficulty","hard","--rules-experiment","india-office-upkeep","--label","india-office-upkeep"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 1000 of 1000 |
| Game length, turns (mean / median / min-max) | 250.2 / 240 / 153-500 |
| Win reason mix | VP hold 96.9%, Capitals 2.5%, Turn limit 0.5%, Draw 0.1% |
| Median end turn | 240 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 2.5% / 96.9% / 0.5% / 0.1% |
| Games reaching the World War era (any empire) | 993 of 1000 (99.3%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1504 / 444 / 973 / 87 |
| Hold break rate | 29.5% of started; 31.3% of resolved (broken / (broken + completed)) |
| Hold wins | 969 |
| First reach to win, turns (mean / median / min-max) | 13.2 / 9 / 9-281 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 150 |
| Wins by a team another team beat to its target | 92 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 969 | 247.2 / 239 / 153-476 |
| Capitals | 25 | 305.3 / 307 / 219-493 |
| Turn limit | 5 | 500.0 / 500 / 500-500 |
| Draw | 1 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 100 | 10.0% |
| India | 45 | 4.5% |
| Byzantium | 96 | 9.6% |
| Germany | 149 | 14.9% |
| Celts | 170 | 17.0% |
| Mongols | 72 | 7.2% |
| Sparta | 120 | 12.0% |
| Athens | 136 | 13.6% |
| Aztecs | 111 | 11.1% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 969 | 96.9% |
| Conquest | 0 | 0.0% |
| Capitals | 25 | 2.5% |
| Turn limit | 5 | 0.5% |
| Draw | 1 | 0.1% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 899 | 100 | 11.1% |
| India | 895 | 45 | 5.0% |
| Byzantium | 878 | 96 | 10.9% |
| Germany | 880 | 149 | 16.9% |
| Celts | 896 | 170 | 19.0% |
| Mongols | 877 | 72 | 8.2% |
| Sparta | 891 | 120 | 13.5% |
| Athens | 895 | 136 | 15.2% |
| Aztecs | 889 | 111 | 12.5% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 8000 | 999 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 899 | 100 | 11.1% |
| India | Hard | 895 | 45 | 5.0% |
| Byzantium | Hard | 878 | 96 | 10.9% |
| Germany | Hard | 880 | 149 | 16.9% |
| Celts | Hard | 896 | 170 | 19.0% |
| Mongols | Hard | 877 | 72 | 8.2% |
| Sparta | Hard | 891 | 120 | 13.5% |
| Athens | Hard | 895 | 136 | 15.2% |
| Aztecs | Hard | 889 | 111 | 12.5% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 899 | 127 | 14.1% | 114 | 151 | 192 | 268 | 487 |
| India | 895 | 443 | 49.5% | 42 | 90 | 129 | 212 | 346 |
| Byzantium | 878 | 614 | 69.9% | 69 | 85 | 110 | 164 | 269 |
| Germany | 880 | 292 | 33.2% | 97 | 127 | 173 | 250 | 465 |
| Celts | 896 | 642 | 71.7% | 31 | 185 | 210 | 210 | 247 |
| Mongols | 877 | 136 | 15.5% | 95 | 160 | 203 | 270 | 380 |
| Sparta | 891 | 203 | 22.8% | 131 | 163 | 209 | 284 | 430 |
| Athens | 895 | 347 | 38.8% | 114 | 173 | 255 | 255 | 255 |
| Aztecs | 889 | 348 | 39.1% | 124 | 189 | 225 | 242 | 314 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 153 | 195 | 240 | 310 | 500 |
| Winning team VP | 3 | 10 | 11 | 13 | 17 |
| First Soldier completed (turn, per empire) | 6 | 8 | 21 | 64 | 315 |
| First Settler completed (turn, per empire) | 3 | 14 | 16 | 32 | 205 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 8 | 18 | 23 | 46 | 261 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 7680 | 320 |
| First Settler completed | 7651 | 349 |
| First city founded | 8000 | 0 |
| Second city founded | 7562 | 438 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 1982 |
| 1 | 1196 |
| 2 | 995 |
| 3 | 720 |
| 4 | 622 |
| 5 | 451 |
| 6 | 371 |
| 7 | 259 |
| 8 | 182 |
| 9 | 154 |
| 10 | 532 |
| 11 | 285 |
| 12 | 144 |
| 13 | 67 |
| 14 | 23 |
| 15 | 10 |
| 16 | 4 |
| 17 | 3 |

## End state

- Average game length: 250.2 turns
- Average cities per empire: 4.27
- Average techs per empire: 31.25
- Average eliminations per game: 2.65

## Strategy diversity: observed behavior (classifier v2)

1000 finished games; 8000 player seats; 999 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4309** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 3313 | 198 | 6.0% | 41.4% | 19.8% |
| Wide | 2595 | 719 | 27.7% | 32.4% | 72.0% |
| Science | 1 | 0 | 0.0% | 0.0% | 0.0% |
| Turtle/Wonder | 10 | 0 | 0.0% | 0.1% | 0.0% |
| Economic | 657 | 80 | 12.2% | 8.2% | 8.0% |
| Generalist | 1424 | 2 | 0.1% | 17.8% | 0.2% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 21.0% / 3.3% (966) | 100.0% / 0.0% (1) | 38.3% / 3.3% (10) | 25.4% / 6.0% (485) | 82.0% / 2.6% (869) |
| Wide | 75.7% / 3.3% (966) | — | 100.0% / 0.0% (1) | 65.0% / 0.0% (10) | 56.3% / 4.4% (480) | 94.9% / 1.1% (860) |
| Science | 0.0% / 0.0% (1) | 0.0% / 0.0% (1) | — | n/a / n/a (0) | 0.0% / 0.0% (1) | 100.0% / 0.0% (1) |
| Turtle/Wonder | 58.3% / 3.3% (10) | 35.0% / 0.0% (10) | n/a / n/a (0) | — | 0.0% / 0.0% (4) | 87.5% / 6.2% (8) |
| Economic | 68.6% / 6.0% (485) | 39.2% / 4.4% (480) | 100.0% / 0.0% (1) | 100.0% / 0.0% (4) | — | 94.5% / 2.1% (409) |
| Generalist | 15.4% / 2.6% (869) | 4.0% / 1.1% (860) | 0.0% / 0.0% (1) | 6.2% / 6.2% (8) | 3.3% / 2.1% (409) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 8 / 249 | 83 / 379 | 0 / 0 | 0 / 0 | 9 / 109 | 0 / 162 |
| India | 11 / 462 | 34 / 260 | 0 / 0 | 0 / 0 | 0 / 12 | 0 / 161 |
| Byzantium | 1 / 14 | 87 / 408 | 0 / 1 | 0 / 4 | 8 / 131 | 0 / 320 |
| Germany | 23 / 337 | 118 / 310 | 0 / 0 | 0 / 3 | 7 / 44 | 1 / 186 |
| Celts | 16 / 270 | 145 / 403 | 0 / 0 | 0 / 0 | 9 / 94 | 0 / 129 |
| Mongols | 44 / 677 | 24 / 80 | 0 / 0 | 0 / 0 | 4 / 14 | 0 / 106 |
| Sparta | 68 / 700 | 36 / 142 | 0 / 0 | 0 / 2 | 16 / 41 | 0 / 6 |
| Athens | 9 / 262 | 111 / 315 | 0 / 0 | 0 / 0 | 15 / 121 | 1 / 197 |
| Aztecs | 18 / 342 | 81 / 298 | 0 / 0 | 0 / 1 | 12 / 91 | 0 / 157 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Science, Turtle/Wonder. Unobserved: none. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 999 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 15427/1000 | 15.427 | 1000/1000 (100.0%) |
| Peace offers | 100030/1000 | 100.030 | 1000/1000 (100.0%) |
| Peace agreements | 12390/1000 | 12.390 | 998/1000 (99.8%) |
| Battles | 1248474/1000 | 1248.474 | 1000/1000 (100.0%) |
| City captures | 23673/1000 | 23.673 | 1000/1000 (100.0%) |
| Settler captures | 4211/1000 | 4.211 | 963/1000 (96.3%) |
| Eliminations | 2645/1000 | 2.645 | 973/1000 (97.3%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 6602/8000 | 6602/8000 | 3188/999 |
| Faction goals | 5096/8000 | 5096/8000 | 2080/999 |
| Era objectives | 14282/8000 | 14282/8000 | 4075/999 |
| Capitals taken | 2631/8000 | 2631/8000 | 1517/999 |
| Total VP | 28611/8000 | 28611/8000 | 10860/999 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 101/725 (13.9%) | 1 |
| Tied leaders (winner in set) | 262/275 (95.3%) | 0 |
| Any leader set | 363/1000 (36.3%) | 1 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 7536/8000 | 5 | 20 | 29 | 33 | 70 | -2.1 (999) |
| Medieval | 7304/8000 | 31 | 58 | 62 | 66 | 130 | -1.9 (999) |
| Renaissance | 7233/8000 | 65 | 84 | 90 | 98 | 163 | -3.5 (999) |
| Industrial | 7032/8000 | 89 | 112 | 121 | 135 | 349 | -5.9 (999) |
| Machine Age | 6448/8000 | 109 | 142 | 153 | 173 | 444 | -8.4 (999) |
| World War | 5260/8000 | 128 | 167 | 182 | 208 | 499 | -10.5 (989) |
| Modern | 3550/8000 | 152 | 191 | 212 | 243 | 457 | -10.6 (892) |
| Future | 1946/8000 | 176 | 217 | 246 | 267 | 494 | -8.0 (618) |

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
| Medieval | Education | 7252/8000 | 86 | 1280 AD | 1088 AD |
| Medieval | Crop Rotation | 7234/8000 | 89 | 1370 AD | n/a |
| Renaissance | Gunpowder | 7212/8000 | 97 | 1470 AD | n/a |
| Renaissance | Metallurgy | 7143/8000 | 108 | 1580 AD | n/a |
| Renaissance | Military Engineering | 7174/8000 | 105 | 1550 AD | n/a |
| Renaissance | Banking | 7079/8000 | 117 | 1670 AD | 1397 AD |
| Renaissance | Printing Press | 7039/8000 | 119 | 1690 AD | 1440 AD |
| Industrial | Rifling | 6690/8000 | 143 | 1816 AD | n/a |
| Industrial | Military Science | 6846/8000 | 134 | 1783 AD | n/a |
| Industrial | Ballistics | 6722/8000 | 145 | 1824 AD | n/a |
| Industrial | Industrialization | 6930/8000 | 130 | 1768 AD | 1769 AD |
| Industrial | Railways | 6452/8000 | 153 | 1854 AD | 1825 AD |
| Machine Age | Replaceable Parts | 5939/8000 | 168 | 1889 AD | 1800 AD |
| Machine Age | Combustion | 6202/8000 | 162 | 1882 AD | 1876 AD |
| Machine Age | Steel | 5872/8000 | 166 | 1886 AD | 1856 AD |
| Machine Age | Flight | 5515/8000 | 177 | 1899 AD | 1903 AD |
| Machine Age | Electricity | 5307/8000 | 182 | 1904 AD | 1879 AD |
| World War | Mass Mobilisation | 4430/8000 | 197 | 1924 AD | n/a |
| World War | Rocketry | 4193/8000 | 202 | 1931 AD | 1926 AD |
| World War | Armoured Warfare | 3948/8000 | 207 | 1938 AD | 1916 AD |
| World War | Air Power | 3772/8000 | 210 | 1942 AD | n/a |
| World War | Electronics | 4904/8000 | 189 | 1912 AD | n/a |
| Modern | Mechanised Warfare | 2640/8000 | 227 | 1978 AD | n/a |
| Modern | Composite Armour | 2403/8000 | 233 | 1993 AD | n/a |
| Modern | Guided Missiles | 2251/8000 | 239 | 2007 AD | n/a |
| Modern | Jet Engines | 2191/8000 | 242 | 2013 AD | 1939 AD |
| Modern | Green Revolution | 3084/8000 | 221 | 1964 AD | n/a |
| Future | Autonomous Systems | 1321/8000 | 256 | 2032 AD | n/a |
| Future | Robotics | 1114/8000 | 262 | 2038 AD | n/a |
| Future | Fusion Power | 1066/8000 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 1555/8000 | 255 | 2031 AD | n/a |
| Future | Recursive AI | 946/8000 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 64.08 |
| 1 | 71.56 | 14.39 |
| 2 | 28.04 | 5.55 |
| 3 | 8.06 | 1.33 |
| 4 | 1.68 | 0.32 |
| 5 | 0.36 | 0.08 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 20261126 | 214 | 1257 | 11 | 2 | 2 |
| 1 | 20261127 | 233 | 1018 | 15 | 7 | 3 |
| 2 | 20261128 | 277 | 1660 | 14 | 8 | 1 |
| 3 | 20261129 | 287 | 1198 | 37 | 1 | 2 |
| 4 | 20261130 | 223 | 1324 | 30 | 4 | 4 |
| 5 | 20261131 | 194 | 797 | 15 | 4 | 1 |
| 6 | 20261132 | 208 | 1019 | 22 | 2 | 0 |
| 7 | 20261133 | 331 | 1809 | 40 | 2 | 4 |
| 8 | 20261134 | 244 | 1096 | 19 | 6 | 3 |
| 9 | 20261135 | 186 | 1019 | 10 | 4 | 1 |
| 10 | 20261136 | 286 | 1360 | 29 | 6 | 2 |
| 11 | 20261137 | 281 | 1187 | 21 | 2 | 2 |
| 12 | 20261138 | 220 | 1158 | 20 | 3 | 2 |
| 13 | 20261139 | 226 | 988 | 14 | 8 | 1 |
| 14 | 20261140 | 258 | 1527 | 44 | 6 | 4 |
| 15 | 20261141 | 208 | 894 | 8 | 5 | 1 |
| 16 | 20261142 | 226 | 1173 | 24 | 7 | 2 |
| 17 | 20261143 | 191 | 1001 | 15 | 6 | 1 |
| 18 | 20261144 | 278 | 1146 | 29 | 3 | 4 |
| 19 | 20261145 | 318 | 1466 | 43 | 3 | 4 |
| 20 | 20261146 | 234 | 1070 | 14 | 2 | 1 |
| 21 | 20261147 | 293 | 1517 | 37 | 5 | 2 |
| 22 | 20261148 | 250 | 1223 | 26 | 3 | 3 |
| 23 | 20261149 | 334 | 2087 | 22 | 3 | 2 |
| 24 | 20261150 | 294 | 1666 | 24 | 4 | 0 |
| 25 | 20261151 | 242 | 1059 | 24 | 0 | 2 |
| 26 | 20261152 | 212 | 1309 | 19 | 7 | 4 |
| 27 | 20261153 | 241 | 1125 | 16 | 4 | 3 |
| 28 | 20261154 | 270 | 1444 | 41 | 13 | 2 |
| 29 | 20261155 | 209 | 834 | 12 | 7 | 1 |
| 30 | 20261156 | 470 | 3189 | 42 | 0 | 3 |
| 31 | 20261157 | 215 | 1186 | 29 | 2 | 2 |
| 32 | 20261158 | 274 | 1423 | 27 | 5 | 3 |
| 33 | 20261159 | 204 | 835 | 23 | 4 | 1 |
| 34 | 20261160 | 221 | 1178 | 17 | 3 | 2 |
| 35 | 20261161 | 211 | 834 | 15 | 4 | 2 |
| 36 | 20261162 | 286 | 1150 | 26 | 0 | 4 |
| 37 | 20261163 | 252 | 1337 | 32 | 4 | 2 |
| 38 | 20261164 | 220 | 901 | 26 | 3 | 3 |
| 39 | 20261165 | 269 | 1609 | 28 | 3 | 2 |
| 40 | 20261166 | 167 | 925 | 13 | 6 | 2 |
| 41 | 20261167 | 193 | 945 | 24 | 4 | 4 |
| 42 | 20261168 | 258 | 1492 | 33 | 4 | 4 |
| 43 | 20261169 | 310 | 2082 | 25 | 10 | 2 |
| 44 | 20261170 | 364 | 1791 | 20 | 5 | 4 |
| 45 | 20261171 | 197 | 710 | 14 | 8 | 3 |
| 46 | 20261172 | 305 | 1774 | 30 | 4 | 3 |
| 47 | 20261173 | 249 | 1022 | 21 | 3 | 4 |
| 48 | 20261174 | 227 | 952 | 26 | 7 | 6 |
| 49 | 20261175 | 260 | 1366 | 22 | 1 | 5 |
| 50 | 20261176 | 193 | 1031 | 18 | 2 | 2 |
| 51 | 20261177 | 287 | 1493 | 28 | 11 | 3 |
| 52 | 20261178 | 242 | 1088 | 18 | 0 | 2 |
| 53 | 20261179 | 269 | 1453 | 34 | 6 | 3 |
| 54 | 20261180 | 203 | 789 | 10 | 2 | 3 |
| 55 | 20261181 | 213 | 1065 | 25 | 5 | 3 |
| 56 | 20261182 | 228 | 1081 | 24 | 7 | 1 |
| 57 | 20261183 | 284 | 1557 | 35 | 2 | 3 |
| 58 | 20261184 | 234 | 1150 | 26 | 1 | 3 |
| 59 | 20261185 | 250 | 1044 | 21 | 3 | 2 |
| 60 | 20261186 | 200 | 1030 | 27 | 4 | 3 |
| 61 | 20261187 | 271 | 1099 | 30 | 5 | 3 |
| 62 | 20261188 | 339 | 2048 | 39 | 1 | 3 |
| 63 | 20261189 | 284 | 1697 | 25 | 4 | 3 |
| 64 | 20261190 | 272 | 1347 | 34 | 9 | 3 |
| 65 | 20261191 | 345 | 2054 | 31 | 7 | 5 |
| 66 | 20261192 | 271 | 1513 | 37 | 2 | 2 |
| 67 | 20261193 | 220 | 1341 | 13 | 0 | 4 |
| 68 | 20261194 | 227 | 926 | 12 | 3 | 2 |
| 69 | 20261195 | 235 | 1230 | 20 | 8 | 1 |
| 70 | 20261196 | 193 | 836 | 13 | 7 | 2 |
| 71 | 20261197 | 281 | 1520 | 18 | 5 | 4 |
| 72 | 20261198 | 252 | 1174 | 17 | 4 | 2 |
| 73 | 20261199 | 323 | 1581 | 33 | 6 | 4 |
| 74 | 20261200 | 273 | 1500 | 29 | 3 | 2 |
| 75 | 20261201 | 198 | 827 | 17 | 8 | 4 |
| 76 | 20261202 | 234 | 1295 | 28 | 7 | 0 |
| 77 | 20261203 | 276 | 1284 | 17 | 13 | 1 |
| 78 | 20261204 | 365 | 1637 | 22 | 8 | 6 |
| 79 | 20261205 | 242 | 1215 | 26 | 11 | 4 |
| 80 | 20261206 | 271 | 1127 | 31 | 7 | 2 |
| 81 | 20261207 | 227 | 764 | 8 | 3 | 2 |
| 82 | 20261208 | 259 | 1396 | 23 | 6 | 3 |
| 83 | 20261209 | 239 | 1194 | 27 | 1 | 1 |
| 84 | 20261210 | 405 | 1944 | 42 | 8 | 2 |
| 85 | 20261211 | 210 | 1111 | 30 | 0 | 1 |
| 86 | 20261212 | 241 | 1080 | 17 | 2 | 3 |
| 87 | 20261213 | 246 | 868 | 20 | 7 | 2 |
| 88 | 20261214 | 270 | 993 | 25 | 6 | 3 |
| 89 | 20261215 | 218 | 1159 | 30 | 4 | 2 |
| 90 | 20261216 | 295 | 1561 | 36 | 5 | 1 |
| 91 | 20261217 | 325 | 1794 | 33 | 4 | 5 |
| 92 | 20261218 | 202 | 890 | 15 | 4 | 3 |
| 93 | 20261219 | 241 | 1088 | 9 | 3 | 4 |
| 94 | 20261220 | 228 | 1150 | 29 | 2 | 4 |
| 95 | 20261221 | 273 | 1236 | 31 | 8 | 3 |
| 96 | 20261222 | 253 | 1499 | 27 | 6 | 3 |
| 97 | 20261223 | 203 | 788 | 18 | 3 | 3 |
| 98 | 20261224 | 270 | 1273 | 18 | 7 | 3 |
| 99 | 20261225 | 220 | 1183 | 27 | 5 | 1 |
| 100 | 20261226 | 186 | 808 | 16 | 3 | 3 |
| 101 | 20261227 | 301 | 1392 | 51 | 6 | 4 |
| 102 | 20261228 | 219 | 1051 | 10 | 4 | 1 |
| 103 | 20261229 | 254 | 1157 | 22 | 3 | 3 |
| 104 | 20261230 | 383 | 2077 | 50 | 4 | 2 |
| 105 | 20261231 | 199 | 692 | 12 | 1 | 2 |
| 106 | 20261232 | 180 | 848 | 16 | 4 | 2 |
| 107 | 20261233 | 272 | 1335 | 27 | 2 | 2 |
| 108 | 20261234 | 207 | 655 | 22 | 9 | 4 |
| 109 | 20261235 | 221 | 1145 | 14 | 7 | 3 |
| 110 | 20261236 | 181 | 746 | 9 | 3 | 2 |
| 111 | 20261237 | 265 | 1382 | 31 | 2 | 4 |
| 112 | 20261238 | 244 | 1253 | 33 | 2 | 3 |
| 113 | 20261239 | 203 | 964 | 19 | 3 | 2 |
| 114 | 20261240 | 312 | 2085 | 35 | 6 | 1 |
| 115 | 20261241 | 236 | 1020 | 15 | 0 | 4 |
| 116 | 20261242 | 244 | 1268 | 25 | 2 | 4 |
| 117 | 20261243 | 226 | 1131 | 17 | 5 | 2 |
| 118 | 20261244 | 204 | 1065 | 28 | 3 | 1 |
| 119 | 20261245 | 218 | 1053 | 16 | 2 | 3 |
| 120 | 20261246 | 234 | 1166 | 15 | 4 | 3 |
| 121 | 20261247 | 219 | 819 | 16 | 3 | 2 |
| 122 | 20261248 | 313 | 2126 | 32 | 6 | 2 |
| 123 | 20261249 | 269 | 1187 | 28 | 2 | 2 |
| 124 | 20261250 | 225 | 1036 | 25 | 5 | 3 |
| 125 | 20261251 | 282 | 1516 | 18 | 1 | 1 |
| 126 | 20261252 | 222 | 1002 | 27 | 6 | 3 |
| 127 | 20261253 | 224 | 926 | 16 | 2 | 2 |
| 128 | 20261254 | 261 | 1370 | 16 | 6 | 6 |
| 129 | 20261255 | 216 | 1087 | 23 | 2 | 3 |
| 130 | 20261256 | 261 | 1410 | 33 | 0 | 3 |
| 131 | 20261257 | 277 | 1360 | 17 | 1 | 2 |
| 132 | 20261258 | 272 | 1071 | 13 | 1 | 3 |
| 133 | 20261259 | 290 | 1405 | 28 | 8 | 2 |
| 134 | 20261260 | 233 | 1226 | 28 | 4 | 2 |
| 135 | 20261261 | 500 | 2016 | 64 | 5 | 5 |
| 136 | 20261262 | 208 | 882 | 20 | 2 | 2 |
| 137 | 20261263 | 189 | 762 | 13 | 1 | 1 |
| 138 | 20261264 | 194 | 964 | 23 | 4 | 3 |
| 139 | 20261265 | 224 | 1086 | 20 | 5 | 3 |
| 140 | 20261266 | 250 | 1307 | 32 | 8 | 4 |
| 141 | 20261267 | 280 | 1206 | 32 | 5 | 5 |
| 142 | 20261268 | 226 | 1125 | 24 | 3 | 0 |
| 143 | 20261269 | 321 | 1605 | 28 | 8 | 5 |
| 144 | 20261270 | 234 | 1004 | 22 | 1 | 1 |
| 145 | 20261271 | 277 | 1335 | 21 | 2 | 2 |
| 146 | 20261272 | 240 | 1057 | 22 | 9 | 3 |
| 147 | 20261273 | 262 | 1109 | 33 | 2 | 4 |
| 148 | 20261274 | 229 | 1306 | 22 | 3 | 2 |
| 149 | 20261275 | 220 | 1112 | 17 | 4 | 1 |
| 150 | 20261276 | 256 | 1093 | 17 | 7 | 2 |
| 151 | 20261277 | 240 | 1052 | 22 | 6 | 4 |
| 152 | 20261278 | 185 | 932 | 20 | 12 | 2 |
| 153 | 20261279 | 249 | 1449 | 13 | 7 | 1 |
| 154 | 20261280 | 262 | 1167 | 34 | 5 | 4 |
| 155 | 20261281 | 194 | 992 | 12 | 2 | 2 |
| 156 | 20261282 | 236 | 1260 | 23 | 6 | 3 |
| 157 | 20261283 | 215 | 934 | 18 | 3 | 2 |
| 158 | 20261284 | 285 | 1240 | 14 | 7 | 3 |
| 159 | 20261285 | 236 | 1250 | 29 | 10 | 3 |
| 160 | 20261286 | 182 | 786 | 18 | 3 | 2 |
| 161 | 20261287 | 256 | 1299 | 25 | 5 | 2 |
| 162 | 20261288 | 190 | 1162 | 17 | 3 | 0 |
| 163 | 20261289 | 228 | 1161 | 17 | 1 | 3 |
| 164 | 20261290 | 335 | 1804 | 44 | 6 | 4 |
| 165 | 20261291 | 251 | 1270 | 22 | 8 | 2 |
| 166 | 20261292 | 251 | 1303 | 32 | 5 | 4 |
| 167 | 20261293 | 226 | 1353 | 25 | 3 | 4 |
| 168 | 20261294 | 243 | 1113 | 20 | 2 | 3 |
| 169 | 20261295 | 392 | 1673 | 32 | 3 | 4 |
| 170 | 20261296 | 293 | 1685 | 20 | 3 | 1 |
| 171 | 20261297 | 220 | 781 | 21 | 1 | 3 |
| 172 | 20261298 | 292 | 1692 | 23 | 2 | 5 |
| 173 | 20261299 | 330 | 1534 | 34 | 5 | 4 |
| 174 | 20261300 | 226 | 1416 | 26 | 2 | 3 |
| 175 | 20261301 | 292 | 1568 | 27 | 2 | 5 |
| 176 | 20261302 | 250 | 1154 | 26 | 7 | 4 |
| 177 | 20261303 | 281 | 1310 | 25 | 1 | 5 |
| 178 | 20261304 | 246 | 880 | 7 | 1 | 5 |
| 179 | 20261305 | 299 | 2003 | 26 | 8 | 1 |
| 180 | 20261306 | 264 | 1154 | 29 | 3 | 2 |
| 181 | 20261307 | 214 | 953 | 25 | 2 | 2 |
| 182 | 20261308 | 262 | 1282 | 34 | 4 | 2 |
| 183 | 20261309 | 256 | 1167 | 41 | 6 | 5 |
| 184 | 20261310 | 297 | 1960 | 42 | 3 | 2 |
| 185 | 20261311 | 214 | 1085 | 25 | 5 | 3 |
| 186 | 20261312 | 260 | 1488 | 17 | 6 | 5 |
| 187 | 20261313 | 263 | 1384 | 32 | 7 | 3 |
| 188 | 20261314 | 194 | 1014 | 18 | 4 | 1 |
| 189 | 20261315 | 402 | 2095 | 32 | 10 | 5 |
| 190 | 20261316 | 224 | 1012 | 20 | 6 | 4 |
| 191 | 20261317 | 219 | 1040 | 32 | 5 | 4 |
| 192 | 20261318 | 306 | 1486 | 12 | 10 | 5 |
| 193 | 20261319 | 252 | 842 | 13 | 2 | 1 |
| 194 | 20261320 | 226 | 1076 | 23 | 4 | 1 |
| 195 | 20261321 | 235 | 1170 | 31 | 6 | 3 |
| 196 | 20261322 | 411 | 2411 | 35 | 8 | 3 |
| 197 | 20261323 | 288 | 1345 | 15 | 3 | 5 |
| 198 | 20261324 | 209 | 954 | 9 | 5 | 1 |
| 199 | 20261325 | 219 | 1102 | 24 | 5 | 3 |
| 200 | 20261326 | 300 | 1687 | 36 | 6 | 4 |
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
| 215 | 20261341 | 193 | 737 | 18 | 9 | 5 |
| 216 | 20261342 | 213 | 1098 | 21 | 2 | 2 |
| 217 | 20261343 | 209 | 980 | 21 | 4 | 1 |
| 218 | 20261344 | 242 | 1111 | 25 | 4 | 4 |
| 219 | 20261345 | 216 | 763 | 10 | 2 | 1 |
| 220 | 20261346 | 294 | 1431 | 28 | 2 | 4 |
| 221 | 20261347 | 255 | 1708 | 25 | 3 | 1 |
| 222 | 20261348 | 228 | 1082 | 22 | 5 | 3 |
| 223 | 20261349 | 232 | 948 | 18 | 5 | 3 |
| 224 | 20261350 | 224 | 980 | 15 | 5 | 4 |
| 225 | 20261351 | 220 | 1120 | 17 | 6 | 2 |
| 226 | 20261352 | 207 | 856 | 14 | 3 | 1 |
| 227 | 20261353 | 392 | 2225 | 38 | 7 | 3 |
| 228 | 20261354 | 310 | 1436 | 38 | 5 | 5 |
| 229 | 20261355 | 201 | 970 | 19 | 0 | 2 |
| 230 | 20261356 | 238 | 1211 | 27 | 1 | 3 |
| 231 | 20261357 | 153 | 705 | 12 | 3 | 1 |
| 232 | 20261358 | 224 | 1094 | 15 | 7 | 4 |
| 233 | 20261359 | 233 | 1180 | 28 | 4 | 3 |
| 234 | 20261360 | 202 | 811 | 15 | 4 | 4 |
| 235 | 20261361 | 404 | 1803 | 26 | 5 | 5 |
| 236 | 20261362 | 342 | 2283 | 37 | 3 | 2 |
| 237 | 20261363 | 264 | 1496 | 22 | 7 | 3 |
| 238 | 20261364 | 212 | 862 | 21 | 6 | 4 |
| 239 | 20261365 | 317 | 1539 | 23 | 6 | 5 |
| 240 | 20261366 | 268 | 1618 | 28 | 4 | 2 |
| 241 | 20261367 | 270 | 1494 | 28 | 2 | 6 |
| 242 | 20261368 | 214 | 995 | 17 | 1 | 1 |
| 243 | 20261369 | 206 | 886 | 18 | 4 | 2 |
| 244 | 20261370 | 310 | 1638 | 40 | 7 | 2 |
| 245 | 20261371 | 199 | 710 | 15 | 0 | 2 |
| 246 | 20261372 | 311 | 1499 | 15 | 5 | 4 |
| 247 | 20261373 | 392 | 2046 | 41 | 2 | 3 |
| 248 | 20261374 | 394 | 2741 | 16 | 3 | 5 |
| 249 | 20261375 | 222 | 959 | 16 | 3 | 2 |
| 250 | 20261376 | 249 | 1204 | 21 | 1 | 3 |
| 251 | 20261377 | 185 | 878 | 15 | 4 | 3 |
| 252 | 20261378 | 249 | 1439 | 26 | 1 | 2 |
| 253 | 20261379 | 234 | 1008 | 17 | 2 | 2 |
| 254 | 20261380 | 237 | 1170 | 17 | 4 | 2 |
| 255 | 20261381 | 306 | 1386 | 19 | 5 | 4 |
| 256 | 20261382 | 243 | 1140 | 21 | 1 | 4 |
| 257 | 20261383 | 276 | 1218 | 27 | 2 | 5 |
| 258 | 20261384 | 268 | 1708 | 29 | 5 | 3 |
| 259 | 20261385 | 263 | 1117 | 17 | 4 | 2 |
| 260 | 20261386 | 264 | 1005 | 24 | 7 | 3 |
| 261 | 20261387 | 253 | 1150 | 33 | 9 | 1 |
| 262 | 20261388 | 206 | 825 | 15 | 4 | 4 |
| 263 | 20261389 | 259 | 1445 | 23 | 4 | 3 |
| 264 | 20261390 | 253 | 1207 | 36 | 2 | 4 |
| 265 | 20261391 | 277 | 1261 | 39 | 4 | 5 |
| 266 | 20261392 | 193 | 789 | 14 | 0 | 3 |
| 267 | 20261393 | 282 | 1280 | 35 | 3 | 3 |
| 268 | 20261394 | 280 | 1472 | 34 | 12 | 3 |
| 269 | 20261395 | 200 | 966 | 17 | 6 | 1 |
| 270 | 20261396 | 238 | 1181 | 29 | 0 | 3 |
| 271 | 20261397 | 289 | 1346 | 23 | 4 | 2 |
| 272 | 20261398 | 277 | 1566 | 31 | 1 | 2 |
| 273 | 20261399 | 212 | 1071 | 25 | 7 | 2 |
| 274 | 20261400 | 213 | 683 | 19 | 1 | 3 |
| 275 | 20261401 | 187 | 875 | 21 | 6 | 2 |
| 276 | 20261402 | 259 | 1352 | 23 | 6 | 4 |
| 277 | 20261403 | 227 | 1122 | 24 | 6 | 1 |
| 278 | 20261404 | 218 | 1063 | 21 | 3 | 4 |
| 279 | 20261405 | 258 | 1196 | 25 | 1 | 4 |
| 280 | 20261406 | 255 | 1310 | 29 | 7 | 1 |
| 281 | 20261407 | 155 | 668 | 8 | 3 | 1 |
| 282 | 20261408 | 254 | 1550 | 30 | 10 | 2 |
| 283 | 20261409 | 231 | 1069 | 19 | 8 | 3 |
| 284 | 20261410 | 273 | 1537 | 21 | 4 | 5 |
| 285 | 20261411 | 239 | 1291 | 38 | 5 | 4 |
| 286 | 20261412 | 246 | 1281 | 17 | 5 | 0 |
| 287 | 20261413 | 207 | 1098 | 16 | 2 | 2 |
| 288 | 20261414 | 264 | 1380 | 23 | 4 | 2 |
| 289 | 20261415 | 297 | 1514 | 38 | 3 | 3 |
| 290 | 20261416 | 275 | 1676 | 18 | 2 | 5 |
| 291 | 20261417 | 180 | 776 | 13 | 3 | 2 |
| 292 | 20261418 | 260 | 1215 | 32 | 5 | 4 |
| 293 | 20261419 | 309 | 1843 | 32 | 6 | 3 |
| 294 | 20261420 | 257 | 1206 | 29 | 7 | 2 |
| 295 | 20261421 | 223 | 1171 | 26 | 13 | 1 |
| 296 | 20261422 | 241 | 1055 | 15 | 1 | 4 |
| 297 | 20261423 | 319 | 1819 | 24 | 6 | 6 |
| 298 | 20261424 | 309 | 1446 | 35 | 1 | 4 |
| 299 | 20261425 | 281 | 1702 | 33 | 2 | 3 |
| 300 | 20261426 | 284 | 1367 | 27 | 5 | 3 |
| 301 | 20261427 | 291 | 1888 | 42 | 1 | 2 |
| 302 | 20261428 | 224 | 1217 | 15 | 6 | 1 |
| 303 | 20261429 | 290 | 1509 | 28 | 2 | 1 |
| 304 | 20261430 | 228 | 1076 | 19 | 5 | 2 |
| 305 | 20261431 | 211 | 819 | 21 | 5 | 2 |
| 306 | 20261432 | 248 | 816 | 24 | 3 | 3 |
| 307 | 20261433 | 243 | 1171 | 35 | 3 | 4 |
| 308 | 20261434 | 258 | 1174 | 26 | 2 | 3 |
| 309 | 20261435 | 233 | 1243 | 28 | 2 | 4 |
| 310 | 20261436 | 220 | 1245 | 30 | 4 | 2 |
| 311 | 20261437 | 277 | 1500 | 26 | 5 | 4 |
| 312 | 20261438 | 227 | 1145 | 14 | 3 | 2 |
| 313 | 20261439 | 284 | 1616 | 32 | 5 | 3 |
| 314 | 20261440 | 356 | 1846 | 38 | 7 | 3 |
| 315 | 20261441 | 247 | 1066 | 21 | 1 | 4 |
| 316 | 20261442 | 264 | 1292 | 17 | 4 | 3 |
| 317 | 20261443 | 224 | 1085 | 14 | 2 | 1 |
| 318 | 20261444 | 319 | 1499 | 29 | 3 | 4 |
| 319 | 20261445 | 290 | 1427 | 37 | 2 | 2 |
| 320 | 20261446 | 298 | 1830 | 34 | 2 | 2 |
| 321 | 20261447 | 231 | 1064 | 35 | 1 | 3 |
| 322 | 20261448 | 189 | 925 | 14 | 2 | 1 |
| 323 | 20261449 | 200 | 1124 | 16 | 4 | 1 |
| 324 | 20261450 | 222 | 1068 | 24 | 7 | 2 |
| 325 | 20261451 | 222 | 1019 | 16 | 3 | 1 |
| 326 | 20261452 | 240 | 1212 | 22 | 7 | 4 |
| 327 | 20261453 | 261 | 1423 | 32 | 2 | 5 |
| 328 | 20261454 | 228 | 1354 | 17 | 7 | 1 |
| 329 | 20261455 | 213 | 870 | 28 | 8 | 1 |
| 330 | 20261456 | 285 | 1433 | 24 | 1 | 1 |
| 331 | 20261457 | 227 | 1134 | 27 | 3 | 4 |
| 332 | 20261458 | 209 | 1215 | 16 | 5 | 4 |
| 333 | 20261459 | 214 | 981 | 20 | 3 | 3 |
| 334 | 20261460 | 293 | 1331 | 27 | 5 | 3 |
| 335 | 20261461 | 198 | 1024 | 14 | 4 | 3 |
| 336 | 20261462 | 224 | 1026 | 33 | 0 | 3 |
| 337 | 20261463 | 264 | 1525 | 22 | 3 | 1 |
| 338 | 20261464 | 199 | 981 | 17 | 10 | 2 |
| 339 | 20261465 | 218 | 1171 | 21 | 8 | 3 |
| 340 | 20261466 | 178 | 697 | 16 | 3 | 1 |
| 341 | 20261467 | 351 | 1789 | 47 | 5 | 4 |
| 342 | 20261468 | 194 | 921 | 20 | 5 | 2 |
| 343 | 20261469 | 245 | 1351 | 31 | 7 | 4 |
| 344 | 20261470 | 296 | 1133 | 26 | 2 | 3 |
| 345 | 20261471 | 195 | 1005 | 16 | 9 | 1 |
| 346 | 20261472 | 271 | 1363 | 11 | 0 | 3 |
| 347 | 20261473 | 213 | 1094 | 14 | 2 | 1 |
| 348 | 20261474 | 286 | 1557 | 27 | 3 | 2 |
| 349 | 20261475 | 351 | 1762 | 45 | 9 | 3 |
| 350 | 20261476 | 311 | 1825 | 28 | 15 | 0 |
| 351 | 20261477 | 217 | 920 | 22 | 1 | 2 |
| 352 | 20261478 | 223 | 961 | 22 | 6 | 4 |
| 353 | 20261479 | 298 | 1648 | 36 | 3 | 4 |
| 354 | 20261480 | 206 | 710 | 18 | 4 | 1 |
| 355 | 20261481 | 206 | 889 | 13 | 5 | 4 |
| 356 | 20261482 | 214 | 961 | 25 | 6 | 2 |
| 357 | 20261483 | 270 | 1289 | 25 | 5 | 2 |
| 358 | 20261484 | 267 | 1249 | 16 | 5 | 0 |
| 359 | 20261485 | 218 | 1223 | 20 | 4 | 4 |
| 360 | 20261486 | 241 | 937 | 21 | 4 | 3 |
| 361 | 20261487 | 226 | 1075 | 24 | 5 | 4 |
| 362 | 20261488 | 217 | 1248 | 22 | 0 | 2 |
| 363 | 20261489 | 233 | 1091 | 16 | 7 | 2 |
| 364 | 20261490 | 225 | 926 | 16 | 1 | 0 |
| 365 | 20261491 | 263 | 1304 | 28 | 4 | 4 |
| 366 | 20261492 | 206 | 819 | 18 | 7 | 1 |
| 367 | 20261493 | 240 | 1091 | 23 | 3 | 2 |
| 368 | 20261494 | 249 | 1249 | 21 | 4 | 2 |
| 369 | 20261495 | 239 | 1188 | 22 | 10 | 3 |
| 370 | 20261496 | 193 | 889 | 20 | 3 | 3 |
| 371 | 20261497 | 253 | 1325 | 13 | 5 | 3 |
| 372 | 20261498 | 191 | 925 | 17 | 2 | 2 |
| 373 | 20261499 | 250 | 1294 | 25 | 2 | 2 |
| 374 | 20261500 | 214 | 823 | 20 | 2 | 3 |
| 375 | 20261501 | 221 | 1189 | 18 | 4 | 1 |
| 376 | 20261502 | 275 | 1410 | 20 | 2 | 2 |
| 377 | 20261503 | 210 | 906 | 13 | 4 | 4 |
| 378 | 20261504 | 233 | 1227 | 26 | 6 | 4 |
| 379 | 20261505 | 196 | 984 | 17 | 7 | 3 |
| 380 | 20261506 | 379 | 2123 | 50 | 9 | 4 |
| 381 | 20261507 | 224 | 1032 | 15 | 4 | 1 |
| 382 | 20261508 | 195 | 950 | 22 | 3 | 2 |
| 383 | 20261509 | 223 | 1198 | 18 | 3 | 2 |
| 384 | 20261510 | 165 | 643 | 11 | 4 | 1 |
| 385 | 20261511 | 283 | 1315 | 42 | 10 | 6 |
| 386 | 20261512 | 222 | 1116 | 20 | 5 | 4 |
| 387 | 20261513 | 247 | 1418 | 25 | 3 | 4 |
| 388 | 20261514 | 229 | 1275 | 27 | 3 | 4 |
| 389 | 20261515 | 320 | 1979 | 20 | 8 | 3 |
| 390 | 20261516 | 182 | 1050 | 10 | 6 | 0 |
| 391 | 20261517 | 232 | 1170 | 27 | 3 | 4 |
| 392 | 20261518 | 265 | 1364 | 22 | 5 | 1 |
| 393 | 20261519 | 288 | 1076 | 32 | 6 | 2 |
| 394 | 20261520 | 232 | 878 | 16 | 2 | 4 |
| 395 | 20261521 | 194 | 877 | 20 | 5 | 2 |
| 396 | 20261522 | 288 | 1847 | 29 | 9 | 2 |
| 397 | 20261523 | 221 | 998 | 23 | 6 | 1 |
| 398 | 20261524 | 190 | 863 | 22 | 7 | 3 |
| 399 | 20261525 | 233 | 1128 | 24 | 4 | 5 |
| 400 | 20261526 | 256 | 1031 | 16 | 2 | 5 |
| 401 | 20261527 | 239 | 1138 | 21 | 8 | 2 |
| 402 | 20261528 | 209 | 689 | 17 | 3 | 1 |
| 403 | 20261529 | 234 | 1294 | 25 | 7 | 1 |
| 404 | 20261530 | 266 | 1121 | 22 | 3 | 4 |
| 405 | 20261531 | 238 | 1190 | 26 | 1 | 3 |
| 406 | 20261532 | 211 | 908 | 17 | 4 | 1 |
| 407 | 20261533 | 176 | 767 | 12 | 1 | 0 |
| 408 | 20261534 | 237 | 1276 | 29 | 10 | 4 |
| 409 | 20261535 | 245 | 1418 | 30 | 3 | 1 |
| 410 | 20261536 | 182 | 949 | 20 | 2 | 1 |
| 411 | 20261537 | 188 | 811 | 12 | 3 | 1 |
| 412 | 20261538 | 254 | 1272 | 25 | 6 | 3 |
| 413 | 20261539 | 258 | 1072 | 29 | 2 | 2 |
| 414 | 20261540 | 283 | 1230 | 27 | 2 | 4 |
| 415 | 20261541 | 294 | 1808 | 27 | 2 | 1 |
| 416 | 20261542 | 354 | 1814 | 40 | 1 | 4 |
| 417 | 20261543 | 311 | 1372 | 28 | 1 | 4 |
| 418 | 20261544 | 324 | 1778 | 30 | 4 | 3 |
| 419 | 20261545 | 191 | 923 | 9 | 3 | 4 |
| 420 | 20261546 | 258 | 1253 | 35 | 9 | 3 |
| 421 | 20261547 | 238 | 1781 | 22 | 3 | 1 |
| 422 | 20261548 | 227 | 1088 | 23 | 2 | 1 |
| 423 | 20261549 | 241 | 1395 | 23 | 2 | 1 |
| 424 | 20261550 | 229 | 1144 | 21 | 1 | 2 |
| 425 | 20261551 | 322 | 1893 | 26 | 8 | 1 |
| 426 | 20261552 | 223 | 1226 | 23 | 8 | 2 |
| 427 | 20261553 | 235 | 1315 | 31 | 2 | 3 |
| 428 | 20261554 | 304 | 1775 | 26 | 5 | 3 |
| 429 | 20261555 | 266 | 1278 | 25 | 7 | 3 |
| 430 | 20261556 | 226 | 1249 | 11 | 4 | 3 |
| 431 | 20261557 | 216 | 979 | 27 | 5 | 5 |
| 432 | 20261558 | 254 | 1415 | 27 | 5 | 2 |
| 433 | 20261559 | 200 | 1094 | 17 | 4 | 2 |
| 434 | 20261560 | 201 | 1336 | 19 | 0 | 2 |
| 435 | 20261561 | 260 | 1474 | 28 | 5 | 1 |
| 436 | 20261562 | 345 | 1483 | 31 | 4 | 4 |
| 437 | 20261563 | 350 | 2295 | 34 | 8 | 3 |
| 438 | 20261564 | 243 | 1307 | 21 | 3 | 3 |
| 439 | 20261565 | 211 | 1076 | 20 | 6 | 3 |
| 440 | 20261566 | 222 | 1041 | 18 | 5 | 1 |
| 441 | 20261567 | 269 | 1429 | 31 | 6 | 4 |
| 442 | 20261568 | 252 | 1413 | 11 | 10 | 1 |
| 443 | 20261569 | 163 | 606 | 16 | 4 | 3 |
| 444 | 20261570 | 221 | 1266 | 13 | 7 | 1 |
| 445 | 20261571 | 268 | 1376 | 22 | 8 | 4 |
| 446 | 20261572 | 258 | 1421 | 34 | 11 | 1 |
| 447 | 20261573 | 251 | 1458 | 27 | 2 | 3 |
| 448 | 20261574 | 194 | 1091 | 32 | 4 | 1 |
| 449 | 20261575 | 237 | 1126 | 28 | 4 | 3 |
| 450 | 20261576 | 276 | 1421 | 27 | 6 | 5 |
| 451 | 20261577 | 215 | 1003 | 22 | 4 | 4 |
| 452 | 20261578 | 426 | 2577 | 34 | 8 | 4 |
| 453 | 20261579 | 300 | 1283 | 42 | 4 | 4 |
| 454 | 20261580 | 220 | 1152 | 20 | 4 | 2 |
| 455 | 20261581 | 217 | 1243 | 28 | 9 | 3 |
| 456 | 20261582 | 182 | 958 | 12 | 9 | 1 |
| 457 | 20261583 | 290 | 1419 | 24 | 9 | 4 |
| 458 | 20261584 | 230 | 1257 | 22 | 10 | 2 |
| 459 | 20261585 | 229 | 1036 | 34 | 2 | 2 |
| 460 | 20261586 | 357 | 2059 | 38 | 1 | 4 |
| 461 | 20261587 | 213 | 1035 | 16 | 3 | 3 |
| 462 | 20261588 | 251 | 1119 | 20 | 0 | 3 |
| 463 | 20261589 | 241 | 1137 | 24 | 2 | 3 |
| 464 | 20261590 | 202 | 976 | 14 | 3 | 3 |
| 465 | 20261591 | 243 | 1237 | 23 | 0 | 1 |
| 466 | 20261592 | 246 | 1410 | 24 | 3 | 2 |
| 467 | 20261593 | 249 | 1529 | 19 | 7 | 3 |
| 468 | 20261594 | 234 | 1281 | 20 | 2 | 4 |
| 469 | 20261595 | 195 | 1025 | 21 | 5 | 1 |
| 470 | 20261596 | 214 | 914 | 21 | 4 | 2 |
| 471 | 20261597 | 412 | 2748 | 35 | 1 | 3 |
| 472 | 20261598 | 500 | 2242 | 71 | 7 | 4 |
| 473 | 20261599 | 500 | 2658 | 46 | 4 | 4 |
| 474 | 20261600 | 275 | 1345 | 17 | 1 | 3 |
| 475 | 20261601 | 260 | 1202 | 24 | 9 | 2 |
| 476 | 20261602 | 180 | 773 | 17 | 2 | 3 |
| 477 | 20261603 | 277 | 1524 | 27 | 1 | 2 |
| 478 | 20261604 | 242 | 1172 | 15 | 5 | 5 |
| 479 | 20261605 | 293 | 1800 | 33 | 2 | 3 |
| 480 | 20261606 | 244 | 1323 | 33 | 6 | 1 |
| 481 | 20261607 | 214 | 904 | 19 | 6 | 2 |
| 482 | 20261608 | 209 | 1003 | 22 | 3 | 3 |
| 483 | 20261609 | 218 | 956 | 20 | 3 | 1 |
| 484 | 20261610 | 215 | 997 | 24 | 2 | 2 |
| 485 | 20261611 | 277 | 1302 | 29 | 10 | 5 |
| 486 | 20261612 | 171 | 589 | 9 | 1 | 2 |
| 487 | 20261613 | 257 | 1032 | 18 | 1 | 4 |
| 488 | 20261614 | 329 | 1608 | 30 | 5 | 2 |
| 489 | 20261615 | 248 | 1017 | 15 | 3 | 1 |
| 490 | 20261616 | 226 | 1110 | 16 | 7 | 1 |
| 491 | 20261617 | 252 | 1208 | 16 | 2 | 4 |
| 492 | 20261618 | 286 | 1628 | 22 | 1 | 3 |
| 493 | 20261619 | 232 | 1087 | 31 | 5 | 3 |
| 494 | 20261620 | 283 | 1279 | 42 | 4 | 4 |
| 495 | 20261621 | 198 | 937 | 21 | 2 | 3 |
| 496 | 20261622 | 190 | 800 | 16 | 4 | 3 |
| 497 | 20261623 | 298 | 1786 | 24 | 2 | 3 |
| 498 | 20261624 | 214 | 932 | 16 | 8 | 2 |
| 499 | 20261625 | 268 | 1526 | 26 | 11 | 2 |
| 500 | 20261626 | 276 | 990 | 28 | 0 | 3 |
| 501 | 20261627 | 252 | 1251 | 25 | 3 | 1 |
| 502 | 20261628 | 240 | 1169 | 18 | 2 | 4 |
| 503 | 20261629 | 250 | 1176 | 33 | 1 | 2 |
| 504 | 20261630 | 282 | 1675 | 25 | 8 | 6 |
| 505 | 20261631 | 260 | 1433 | 20 | 2 | 3 |
| 506 | 20261632 | 182 | 876 | 22 | 1 | 2 |
| 507 | 20261633 | 266 | 1692 | 29 | 8 | 2 |
| 508 | 20261634 | 197 | 1092 | 19 | 5 | 1 |
| 509 | 20261635 | 426 | 1993 | 54 | 5 | 4 |
| 510 | 20261636 | 228 | 1019 | 17 | 6 | 2 |
| 511 | 20261637 | 261 | 1322 | 19 | 2 | 4 |
| 512 | 20261638 | 229 | 1270 | 17 | 0 | 0 |
| 513 | 20261639 | 226 | 910 | 15 | 5 | 0 |
| 514 | 20261640 | 284 | 1603 | 33 | 3 | 3 |
| 515 | 20261641 | 238 | 1149 | 31 | 2 | 5 |
| 516 | 20261642 | 227 | 1304 | 14 | 7 | 2 |
| 517 | 20261643 | 256 | 1229 | 15 | 3 | 5 |
| 518 | 20261644 | 190 | 746 | 6 | 1 | 3 |
| 519 | 20261645 | 303 | 1471 | 26 | 3 | 4 |
| 520 | 20261646 | 335 | 1682 | 26 | 6 | 4 |
| 521 | 20261647 | 212 | 840 | 25 | 5 | 2 |
| 522 | 20261648 | 252 | 1196 | 21 | 9 | 4 |
| 523 | 20261649 | 263 | 1383 | 27 | 11 | 3 |
| 524 | 20261650 | 187 | 941 | 17 | 2 | 0 |
| 525 | 20261651 | 265 | 1337 | 35 | 4 | 3 |
| 526 | 20261652 | 233 | 1399 | 11 | 5 | 4 |
| 527 | 20261653 | 277 | 1489 | 28 | 0 | 3 |
| 528 | 20261654 | 213 | 715 | 13 | 4 | 3 |
| 529 | 20261655 | 380 | 2394 | 45 | 6 | 3 |
| 530 | 20261656 | 293 | 1326 | 15 | 6 | 5 |
| 531 | 20261657 | 279 | 1228 | 18 | 3 | 4 |
| 532 | 20261658 | 182 | 856 | 17 | 3 | 2 |
| 533 | 20261659 | 255 | 1220 | 23 | 1 | 1 |
| 534 | 20261660 | 173 | 623 | 5 | 2 | 1 |
| 535 | 20261661 | 244 | 1109 | 28 | 4 | 3 |
| 536 | 20261662 | 251 | 1244 | 26 | 1 | 1 |
| 537 | 20261663 | 289 | 1553 | 39 | 6 | 4 |
| 538 | 20261664 | 287 | 1552 | 27 | 3 | 2 |
| 539 | 20261665 | 296 | 1643 | 40 | 8 | 4 |
| 540 | 20261666 | 251 | 1013 | 30 | 9 | 3 |
| 541 | 20261667 | 230 | 1109 | 34 | 1 | 4 |
| 542 | 20261668 | 285 | 1520 | 29 | 5 | 3 |
| 543 | 20261669 | 169 | 784 | 16 | 4 | 1 |
| 544 | 20261670 | 227 | 957 | 19 | 8 | 4 |
| 545 | 20261671 | 289 | 1387 | 34 | 7 | 4 |
| 546 | 20261672 | 266 | 1147 | 27 | 4 | 3 |
| 547 | 20261673 | 287 | 1832 | 40 | 4 | 2 |
| 548 | 20261674 | 325 | 1864 | 46 | 2 | 5 |
| 549 | 20261675 | 206 | 950 | 21 | 9 | 2 |
| 550 | 20261676 | 207 | 932 | 18 | 3 | 2 |
| 551 | 20261677 | 271 | 1123 | 31 | 1 | 6 |
| 552 | 20261678 | 181 | 1093 | 18 | 4 | 2 |
| 553 | 20261679 | 218 | 981 | 14 | 6 | 2 |
| 554 | 20261680 | 227 | 1428 | 27 | 7 | 2 |
| 555 | 20261681 | 195 | 903 | 7 | 5 | 1 |
| 556 | 20261682 | 260 | 945 | 20 | 8 | 2 |
| 557 | 20261683 | 193 | 960 | 15 | 1 | 2 |
| 558 | 20261684 | 239 | 1284 | 20 | 1 | 0 |
| 559 | 20261685 | 180 | 707 | 15 | 4 | 4 |
| 560 | 20261686 | 258 | 1467 | 36 | 1 | 4 |
| 561 | 20261687 | 300 | 1584 | 41 | 6 | 2 |
| 562 | 20261688 | 321 | 1676 | 29 | 3 | 2 |
| 563 | 20261689 | 204 | 1074 | 20 | 2 | 1 |
| 564 | 20261690 | 215 | 860 | 17 | 6 | 3 |
| 565 | 20261691 | 279 | 1028 | 29 | 5 | 5 |
| 566 | 20261692 | 228 | 1253 | 26 | 4 | 3 |
| 567 | 20261693 | 227 | 1207 | 29 | 2 | 4 |
| 568 | 20261694 | 169 | 574 | 14 | 2 | 3 |
| 569 | 20261695 | 271 | 916 | 30 | 2 | 4 |
| 570 | 20261696 | 293 | 1556 | 43 | 1 | 3 |
| 571 | 20261697 | 320 | 1654 | 21 | 4 | 3 |
| 572 | 20261698 | 209 | 932 | 20 | 7 | 2 |
| 573 | 20261699 | 198 | 878 | 25 | 1 | 2 |
| 574 | 20261700 | 184 | 846 | 24 | 5 | 3 |
| 575 | 20261701 | 301 | 1451 | 16 | 2 | 3 |
| 576 | 20261702 | 228 | 1371 | 30 | 2 | 3 |
| 577 | 20261703 | 183 | 977 | 19 | 7 | 1 |
| 578 | 20261704 | 257 | 1259 | 11 | 6 | 2 |
| 579 | 20261705 | 281 | 1585 | 28 | 9 | 2 |
| 580 | 20261706 | 307 | 1732 | 42 | 3 | 3 |
| 581 | 20261707 | 196 | 844 | 25 | 2 | 3 |
| 582 | 20261708 | 240 | 1262 | 25 | 2 | 2 |
| 583 | 20261709 | 205 | 899 | 19 | 1 | 1 |
| 584 | 20261710 | 285 | 1231 | 26 | 9 | 4 |
| 585 | 20261711 | 231 | 1081 | 27 | 7 | 2 |
| 586 | 20261712 | 175 | 782 | 17 | 1 | 1 |
| 587 | 20261713 | 294 | 1454 | 25 | 2 | 3 |
| 588 | 20261714 | 182 | 1003 | 8 | 2 | 3 |
| 589 | 20261715 | 200 | 755 | 11 | 4 | 2 |
| 590 | 20261716 | 276 | 1583 | 21 | 3 | 3 |
| 591 | 20261717 | 222 | 1178 | 15 | 3 | 3 |
| 592 | 20261718 | 291 | 1547 | 28 | 2 | 3 |
| 593 | 20261719 | 227 | 1369 | 25 | 2 | 1 |
| 594 | 20261720 | 275 | 1415 | 18 | 4 | 1 |
| 595 | 20261721 | 183 | 831 | 15 | 4 | 3 |
| 596 | 20261722 | 253 | 1268 | 28 | 0 | 4 |
| 597 | 20261723 | 261 | 1351 | 46 | 6 | 2 |
| 598 | 20261724 | 296 | 1426 | 37 | 4 | 1 |
| 599 | 20261725 | 214 | 1185 | 19 | 6 | 2 |
| 600 | 20261726 | 266 | 1238 | 41 | 5 | 2 |
| 601 | 20261727 | 204 | 883 | 18 | 7 | 2 |
| 602 | 20261728 | 249 | 1264 | 30 | 3 | 3 |
| 603 | 20261729 | 218 | 919 | 16 | 3 | 3 |
| 604 | 20261730 | 245 | 1169 | 30 | 3 | 4 |
| 605 | 20261731 | 237 | 1031 | 37 | 5 | 2 |
| 606 | 20261732 | 229 | 1203 | 21 | 4 | 2 |
| 607 | 20261733 | 259 | 1116 | 30 | 2 | 2 |
| 608 | 20261734 | 252 | 1121 | 12 | 2 | 1 |
| 609 | 20261735 | 368 | 2739 | 43 | 7 | 4 |
| 610 | 20261736 | 177 | 486 | 6 | 1 | 2 |
| 611 | 20261737 | 311 | 1627 | 33 | 2 | 4 |
| 612 | 20261738 | 240 | 1404 | 31 | 5 | 3 |
| 613 | 20261739 | 305 | 1527 | 40 | 3 | 4 |
| 614 | 20261740 | 213 | 1180 | 18 | 0 | 3 |
| 615 | 20261741 | 197 | 931 | 16 | 5 | 1 |
| 616 | 20261742 | 242 | 1167 | 27 | 3 | 2 |
| 617 | 20261743 | 280 | 1718 | 27 | 11 | 2 |
| 618 | 20261744 | 189 | 815 | 16 | 3 | 4 |
| 619 | 20261745 | 260 | 1250 | 20 | 2 | 4 |
| 620 | 20261746 | 288 | 1082 | 21 | 4 | 5 |
| 621 | 20261747 | 241 | 1071 | 16 | 6 | 3 |
| 622 | 20261748 | 247 | 1273 | 30 | 4 | 3 |
| 623 | 20261749 | 290 | 1717 | 31 | 4 | 4 |
| 624 | 20261750 | 203 | 971 | 20 | 2 | 3 |
| 625 | 20261751 | 207 | 1075 | 15 | 3 | 2 |
| 626 | 20261752 | 258 | 1524 | 21 | 4 | 2 |
| 627 | 20261753 | 273 | 1368 | 13 | 7 | 2 |
| 628 | 20261754 | 238 | 1142 | 16 | 2 | 6 |
| 629 | 20261755 | 198 | 1110 | 20 | 0 | 1 |
| 630 | 20261756 | 184 | 875 | 13 | 1 | 2 |
| 631 | 20261757 | 255 | 1224 | 15 | 4 | 1 |
| 632 | 20261758 | 231 | 1337 | 17 | 5 | 1 |
| 633 | 20261759 | 333 | 1368 | 26 | 7 | 3 |
| 634 | 20261760 | 232 | 895 | 15 | 0 | 3 |
| 635 | 20261761 | 381 | 1869 | 55 | 1 | 4 |
| 636 | 20261762 | 208 | 660 | 11 | 2 | 3 |
| 637 | 20261763 | 169 | 676 | 11 | 4 | 1 |
| 638 | 20261764 | 269 | 1459 | 31 | 2 | 2 |
| 639 | 20261765 | 176 | 803 | 15 | 6 | 3 |
| 640 | 20261766 | 356 | 1764 | 26 | 1 | 3 |
| 641 | 20261767 | 208 | 1217 | 25 | 7 | 2 |
| 642 | 20261768 | 248 | 1194 | 20 | 6 | 2 |
| 643 | 20261769 | 204 | 1037 | 15 | 1 | 1 |
| 644 | 20261770 | 217 | 906 | 15 | 9 | 3 |
| 645 | 20261771 | 263 | 1017 | 25 | 2 | 3 |
| 646 | 20261772 | 233 | 1392 | 27 | 3 | 2 |
| 647 | 20261773 | 287 | 1646 | 43 | 6 | 2 |
| 648 | 20261774 | 211 | 1298 | 30 | 4 | 2 |
| 649 | 20261775 | 289 | 1517 | 24 | 1 | 5 |
| 650 | 20261776 | 279 | 1590 | 34 | 4 | 4 |
| 651 | 20261777 | 212 | 796 | 14 | 3 | 0 |
| 652 | 20261778 | 241 | 1100 | 33 | 4 | 3 |
| 653 | 20261779 | 284 | 1614 | 39 | 2 | 1 |
| 654 | 20261780 | 368 | 1758 | 33 | 2 | 6 |
| 655 | 20261781 | 229 | 1290 | 30 | 3 | 3 |
| 656 | 20261782 | 253 | 1131 | 30 | 5 | 5 |
| 657 | 20261783 | 286 | 1726 | 34 | 10 | 2 |
| 658 | 20261784 | 222 | 1149 | 24 | 3 | 2 |
| 659 | 20261785 | 399 | 1926 | 45 | 2 | 6 |
| 660 | 20261786 | 286 | 1776 | 45 | 5 | 1 |
| 661 | 20261787 | 276 | 1489 | 33 | 3 | 1 |
| 662 | 20261788 | 242 | 1186 | 22 | 1 | 3 |
| 663 | 20261789 | 235 | 1058 | 22 | 5 | 3 |
| 664 | 20261790 | 156 | 657 | 12 | 2 | 1 |
| 665 | 20261791 | 207 | 963 | 25 | 3 | 2 |
| 666 | 20261792 | 194 | 871 | 14 | 1 | 1 |
| 667 | 20261793 | 218 | 897 | 15 | 4 | 1 |
| 668 | 20261794 | 500 | 2453 | 66 | 9 | 5 |
| 669 | 20261795 | 268 | 1259 | 18 | 7 | 6 |
| 670 | 20261796 | 227 | 918 | 18 | 7 | 1 |
| 671 | 20261797 | 195 | 792 | 15 | 5 | 0 |
| 672 | 20261798 | 217 | 1280 | 21 | 8 | 4 |
| 673 | 20261799 | 183 | 682 | 16 | 3 | 1 |
| 674 | 20261800 | 266 | 1378 | 11 | 9 | 4 |
| 675 | 20261801 | 178 | 725 | 13 | 3 | 2 |
| 676 | 20261802 | 228 | 1424 | 34 | 2 | 3 |
| 677 | 20261803 | 272 | 1512 | 33 | 2 | 1 |
| 678 | 20261804 | 218 | 967 | 18 | 9 | 3 |
| 679 | 20261805 | 229 | 1137 | 14 | 2 | 3 |
| 680 | 20261806 | 192 | 885 | 20 | 0 | 2 |
| 681 | 20261807 | 286 | 1556 | 39 | 4 | 4 |
| 682 | 20261808 | 261 | 1521 | 26 | 9 | 2 |
| 683 | 20261809 | 249 | 1342 | 28 | 9 | 1 |
| 684 | 20261810 | 217 | 861 | 27 | 3 | 2 |
| 685 | 20261811 | 361 | 1759 | 36 | 4 | 3 |
| 686 | 20261812 | 228 | 829 | 21 | 7 | 2 |
| 687 | 20261813 | 241 | 1119 | 10 | 2 | 3 |
| 688 | 20261814 | 177 | 727 | 14 | 2 | 1 |
| 689 | 20261815 | 220 | 999 | 19 | 1 | 2 |
| 690 | 20261816 | 288 | 1566 | 32 | 3 | 3 |
| 691 | 20261817 | 266 | 1443 | 30 | 8 | 3 |
| 692 | 20261818 | 352 | 1840 | 49 | 4 | 5 |
| 693 | 20261819 | 219 | 1078 | 21 | 2 | 1 |
| 694 | 20261820 | 226 | 842 | 15 | 5 | 2 |
| 695 | 20261821 | 173 | 534 | 12 | 5 | 3 |
| 696 | 20261822 | 219 | 1142 | 28 | 5 | 2 |
| 697 | 20261823 | 239 | 1151 | 23 | 4 | 3 |
| 698 | 20261824 | 196 | 710 | 15 | 5 | 2 |
| 699 | 20261825 | 310 | 1776 | 31 | 4 | 2 |
| 700 | 20261826 | 246 | 1354 | 32 | 3 | 4 |
| 701 | 20261827 | 344 | 1600 | 29 | 7 | 4 |
| 702 | 20261828 | 365 | 1881 | 36 | 4 | 2 |
| 703 | 20261829 | 319 | 1413 | 19 | 4 | 3 |
| 704 | 20261830 | 248 | 1384 | 21 | 2 | 3 |
| 705 | 20261831 | 261 | 1237 | 21 | 2 | 1 |
| 706 | 20261832 | 280 | 1482 | 24 | 2 | 2 |
| 707 | 20261833 | 173 | 905 | 11 | 2 | 2 |
| 708 | 20261834 | 240 | 1099 | 14 | 7 | 0 |
| 709 | 20261835 | 206 | 758 | 11 | 5 | 1 |
| 710 | 20261836 | 284 | 1500 | 25 | 0 | 1 |
| 711 | 20261837 | 185 | 1031 | 22 | 2 | 3 |
| 712 | 20261838 | 188 | 721 | 10 | 8 | 1 |
| 713 | 20261839 | 222 | 1090 | 21 | 1 | 3 |
| 714 | 20261840 | 232 | 982 | 11 | 1 | 4 |
| 715 | 20261841 | 179 | 1003 | 9 | 4 | 1 |
| 716 | 20261842 | 276 | 1507 | 24 | 5 | 2 |
| 717 | 20261843 | 241 | 1243 | 21 | 7 | 4 |
| 718 | 20261844 | 281 | 1529 | 25 | 8 | 2 |
| 719 | 20261845 | 236 | 1330 | 38 | 1 | 0 |
| 720 | 20261846 | 224 | 1045 | 25 | 2 | 3 |
| 721 | 20261847 | 189 | 865 | 20 | 2 | 1 |
| 722 | 20261848 | 209 | 692 | 20 | 2 | 4 |
| 723 | 20261849 | 242 | 1232 | 35 | 7 | 1 |
| 724 | 20261850 | 251 | 1261 | 25 | 4 | 2 |
| 725 | 20261851 | 345 | 2123 | 41 | 4 | 4 |
| 726 | 20261852 | 254 | 1385 | 18 | 5 | 2 |
| 727 | 20261853 | 195 | 1081 | 16 | 2 | 1 |
| 728 | 20261854 | 220 | 1091 | 23 | 2 | 1 |
| 729 | 20261855 | 199 | 1205 | 18 | 1 | 2 |
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
| 740 | 20261866 | 238 | 1200 | 19 | 3 | 1 |
| 741 | 20261867 | 204 | 1193 | 26 | 5 | 2 |
| 742 | 20261868 | 310 | 1593 | 31 | 19 | 1 |
| 743 | 20261869 | 255 | 1229 | 34 | 2 | 3 |
| 744 | 20261870 | 262 | 1009 | 24 | 3 | 5 |
| 745 | 20261871 | 284 | 1461 | 22 | 4 | 5 |
| 746 | 20261872 | 240 | 1239 | 20 | 2 | 2 |
| 747 | 20261873 | 187 | 877 | 18 | 4 | 3 |
| 748 | 20261874 | 332 | 2194 | 23 | 5 | 4 |
| 749 | 20261875 | 214 | 1022 | 18 | 3 | 0 |
| 750 | 20261876 | 222 | 1182 | 27 | 2 | 2 |
| 751 | 20261877 | 173 | 735 | 13 | 4 | 3 |
| 752 | 20261878 | 189 | 844 | 12 | 9 | 1 |
| 753 | 20261879 | 290 | 1674 | 27 | 4 | 4 |
| 754 | 20261880 | 228 | 1248 | 23 | 7 | 2 |
| 755 | 20261881 | 179 | 819 | 19 | 4 | 3 |
| 756 | 20261882 | 228 | 1282 | 14 | 0 | 3 |
| 757 | 20261883 | 233 | 1101 | 23 | 4 | 1 |
| 758 | 20261884 | 253 | 1469 | 32 | 6 | 2 |
| 759 | 20261885 | 272 | 1330 | 35 | 4 | 1 |
| 760 | 20261886 | 232 | 1205 | 24 | 3 | 3 |
| 761 | 20261887 | 283 | 1316 | 33 | 6 | 1 |
| 762 | 20261888 | 239 | 1212 | 28 | 8 | 3 |
| 763 | 20261889 | 200 | 1250 | 23 | 2 | 3 |
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
| 776 | 20261902 | 270 | 1774 | 18 | 17 | 2 |
| 777 | 20261903 | 260 | 1264 | 29 | 8 | 3 |
| 778 | 20261904 | 254 | 1456 | 34 | 3 | 2 |
| 779 | 20261905 | 207 | 928 | 22 | 2 | 2 |
| 780 | 20261906 | 284 | 1383 | 18 | 4 | 2 |
| 781 | 20261907 | 281 | 1257 | 24 | 10 | 5 |
| 782 | 20261908 | 301 | 1675 | 20 | 7 | 5 |
| 783 | 20261909 | 214 | 1187 | 20 | 5 | 1 |
| 784 | 20261910 | 308 | 1932 | 21 | 4 | 2 |
| 785 | 20261911 | 245 | 1211 | 29 | 1 | 4 |
| 786 | 20261912 | 355 | 1588 | 32 | 1 | 4 |
| 787 | 20261913 | 218 | 680 | 11 | 11 | 1 |
| 788 | 20261914 | 256 | 1396 | 17 | 6 | 3 |
| 789 | 20261915 | 232 | 1034 | 29 | 14 | 3 |
| 790 | 20261916 | 213 | 977 | 19 | 2 | 1 |
| 791 | 20261917 | 373 | 2303 | 36 | 4 | 2 |
| 792 | 20261918 | 238 | 913 | 15 | 3 | 1 |
| 793 | 20261919 | 225 | 1061 | 29 | 4 | 3 |
| 794 | 20261920 | 235 | 1140 | 22 | 1 | 4 |
| 795 | 20261921 | 238 | 1043 | 18 | 3 | 2 |
| 796 | 20261922 | 239 | 1170 | 26 | 7 | 5 |
| 797 | 20261923 | 279 | 1222 | 25 | 3 | 4 |
| 798 | 20261924 | 204 | 1031 | 23 | 3 | 2 |
| 799 | 20261925 | 259 | 1254 | 32 | 1 | 3 |
| 800 | 20261926 | 294 | 1557 | 25 | 2 | 3 |
| 801 | 20261927 | 225 | 1032 | 30 | 3 | 3 |
| 802 | 20261928 | 247 | 1079 | 32 | 2 | 4 |
| 803 | 20261929 | 225 | 1027 | 18 | 0 | 2 |
| 804 | 20261930 | 271 | 1274 | 17 | 2 | 4 |
| 805 | 20261931 | 266 | 1190 | 22 | 5 | 4 |
| 806 | 20261932 | 280 | 1636 | 21 | 3 | 6 |
| 807 | 20261933 | 298 | 2021 | 31 | 8 | 1 |
| 808 | 20261934 | 199 | 950 | 8 | 3 | 2 |
| 809 | 20261935 | 273 | 1267 | 27 | 2 | 2 |
| 810 | 20261936 | 351 | 1867 | 45 | 3 | 2 |
| 811 | 20261937 | 228 | 1233 | 15 | 1 | 2 |
| 812 | 20261938 | 383 | 2167 | 40 | 4 | 4 |
| 813 | 20261939 | 206 | 752 | 23 | 2 | 3 |
| 814 | 20261940 | 240 | 1367 | 13 | 1 | 2 |
| 815 | 20261941 | 267 | 1360 | 32 | 5 | 1 |
| 816 | 20261942 | 307 | 1566 | 19 | 6 | 4 |
| 817 | 20261943 | 275 | 1380 | 42 | 3 | 5 |
| 818 | 20261944 | 253 | 1397 | 15 | 3 | 2 |
| 819 | 20261945 | 261 | 1417 | 11 | 7 | 2 |
| 820 | 20261946 | 243 | 1227 | 16 | 6 | 3 |
| 821 | 20261947 | 181 | 788 | 9 | 4 | 2 |
| 822 | 20261948 | 240 | 1316 | 26 | 9 | 2 |
| 823 | 20261949 | 211 | 1036 | 17 | 10 | 3 |
| 824 | 20261950 | 342 | 1680 | 33 | 1 | 4 |
| 825 | 20261951 | 295 | 1342 | 38 | 6 | 4 |
| 826 | 20261952 | 213 | 832 | 14 | 1 | 2 |
| 827 | 20261953 | 247 | 1215 | 21 | 5 | 2 |
| 828 | 20261954 | 231 | 1215 | 39 | 1 | 3 |
| 829 | 20261955 | 225 | 1285 | 23 | 6 | 2 |
| 830 | 20261956 | 171 | 793 | 17 | 5 | 3 |
| 831 | 20261957 | 258 | 1289 | 19 | 2 | 4 |
| 832 | 20261958 | 277 | 1341 | 33 | 0 | 4 |
| 833 | 20261959 | 242 | 1198 | 25 | 5 | 3 |
| 834 | 20261960 | 228 | 1181 | 15 | 3 | 2 |
| 835 | 20261961 | 233 | 989 | 17 | 2 | 2 |
| 836 | 20261962 | 254 | 1101 | 24 | 5 | 3 |
| 837 | 20261963 | 237 | 1098 | 21 | 5 | 3 |
| 838 | 20261964 | 294 | 1839 | 29 | 7 | 1 |
| 839 | 20261965 | 224 | 1042 | 23 | 3 | 3 |
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
| 851 | 20261977 | 226 | 1309 | 32 | 3 | 2 |
| 852 | 20261978 | 314 | 1452 | 24 | 7 | 2 |
| 853 | 20261979 | 222 | 852 | 15 | 1 | 3 |
| 854 | 20261980 | 222 | 1028 | 13 | 7 | 1 |
| 855 | 20261981 | 189 | 900 | 17 | 5 | 2 |
| 856 | 20261982 | 204 | 835 | 16 | 3 | 3 |
| 857 | 20261983 | 244 | 1197 | 25 | 10 | 4 |
| 858 | 20261984 | 256 | 1053 | 32 | 0 | 4 |
| 859 | 20261985 | 232 | 1291 | 14 | 5 | 1 |
| 860 | 20261986 | 210 | 1011 | 11 | 6 | 3 |
| 861 | 20261987 | 300 | 2084 | 41 | 5 | 4 |
| 862 | 20261988 | 282 | 1693 | 24 | 4 | 1 |
| 863 | 20261989 | 186 | 949 | 22 | 5 | 2 |
| 864 | 20261990 | 212 | 973 | 13 | 1 | 3 |
| 865 | 20261991 | 220 | 951 | 34 | 2 | 1 |
| 866 | 20261992 | 207 | 1047 | 22 | 1 | 2 |
| 867 | 20261993 | 277 | 1252 | 18 | 5 | 1 |
| 868 | 20261994 | 258 | 1278 | 9 | 3 | 4 |
| 869 | 20261995 | 277 | 1624 | 22 | 1 | 4 |
| 870 | 20261996 | 196 | 1009 | 13 | 4 | 2 |
| 871 | 20261997 | 300 | 1856 | 25 | 3 | 1 |
| 872 | 20261998 | 233 | 1271 | 23 | 2 | 3 |
| 873 | 20261999 | 214 | 967 | 13 | 3 | 2 |
| 874 | 20262000 | 212 | 1120 | 23 | 4 | 0 |
| 875 | 20262001 | 221 | 1067 | 21 | 4 | 1 |
| 876 | 20262002 | 279 | 1055 | 17 | 10 | 3 |
| 877 | 20262003 | 196 | 578 | 14 | 1 | 0 |
| 878 | 20262004 | 210 | 951 | 14 | 3 | 0 |
| 879 | 20262005 | 417 | 2614 | 72 | 5 | 4 |
| 880 | 20262006 | 192 | 784 | 9 | 3 | 3 |
| 881 | 20262007 | 215 | 1180 | 20 | 12 | 2 |
| 882 | 20262008 | 202 | 1120 | 15 | 2 | 3 |
| 883 | 20262009 | 500 | 2328 | 67 | 6 | 6 |
| 884 | 20262010 | 253 | 913 | 13 | 4 | 1 |
| 885 | 20262011 | 250 | 1142 | 15 | 4 | 1 |
| 886 | 20262012 | 239 | 927 | 28 | 7 | 4 |
| 887 | 20262013 | 277 | 1189 | 25 | 5 | 4 |
| 888 | 20262014 | 167 | 770 | 20 | 3 | 3 |
| 889 | 20262015 | 234 | 1260 | 20 | 2 | 2 |
| 890 | 20262016 | 230 | 1165 | 22 | 1 | 3 |
| 891 | 20262017 | 299 | 1361 | 28 | 3 | 3 |
| 892 | 20262018 | 246 | 914 | 11 | 1 | 3 |
| 893 | 20262019 | 235 | 1336 | 25 | 2 | 3 |
| 894 | 20262020 | 225 | 1071 | 26 | 5 | 4 |
| 895 | 20262021 | 225 | 1289 | 17 | 3 | 2 |
| 896 | 20262022 | 194 | 657 | 14 | 6 | 3 |
| 897 | 20262023 | 234 | 1197 | 27 | 6 | 3 |
| 898 | 20262024 | 434 | 2221 | 38 | 7 | 2 |
| 899 | 20262025 | 253 | 1150 | 7 | 3 | 3 |
| 900 | 20262026 | 224 | 1083 | 23 | 0 | 3 |
| 901 | 20262027 | 282 | 1191 | 16 | 9 | 3 |
| 902 | 20262028 | 209 | 757 | 13 | 3 | 3 |
| 903 | 20262029 | 224 | 1328 | 32 | 1 | 2 |
| 904 | 20262030 | 253 | 1267 | 31 | 6 | 2 |
| 905 | 20262031 | 321 | 1842 | 20 | 3 | 3 |
| 906 | 20262032 | 275 | 1205 | 22 | 7 | 3 |
| 907 | 20262033 | 254 | 1176 | 34 | 4 | 2 |
| 908 | 20262034 | 287 | 1375 | 28 | 4 | 3 |
| 909 | 20262035 | 328 | 2061 | 30 | 4 | 1 |
| 910 | 20262036 | 286 | 1503 | 25 | 2 | 3 |
| 911 | 20262037 | 500 | 2515 | 58 | 4 | 5 |
| 912 | 20262038 | 333 | 1745 | 44 | 2 | 4 |
| 913 | 20262039 | 242 | 1382 | 32 | 1 | 2 |
| 914 | 20262040 | 192 | 974 | 17 | 8 | 3 |
| 915 | 20262041 | 169 | 628 | 11 | 3 | 3 |
| 916 | 20262042 | 260 | 1368 | 35 | 7 | 3 |
| 917 | 20262043 | 237 | 1071 | 18 | 7 | 2 |
| 918 | 20262044 | 314 | 1820 | 34 | 6 | 5 |
| 919 | 20262045 | 218 | 833 | 23 | 5 | 2 |
| 920 | 20262046 | 220 | 1340 | 26 | 1 | 2 |
| 921 | 20262047 | 240 | 942 | 12 | 4 | 2 |
| 922 | 20262048 | 239 | 1025 | 22 | 3 | 3 |
| 923 | 20262049 | 266 | 1276 | 38 | 3 | 4 |
| 924 | 20262050 | 185 | 910 | 16 | 4 | 1 |
| 925 | 20262051 | 218 | 892 | 22 | 1 | 2 |
| 926 | 20262052 | 213 | 1073 | 21 | 2 | 1 |
| 927 | 20262053 | 233 | 1314 | 7 | 3 | 3 |
| 928 | 20262054 | 188 | 820 | 10 | 2 | 1 |
| 929 | 20262055 | 167 | 821 | 10 | 2 | 2 |
| 930 | 20262056 | 202 | 943 | 24 | 3 | 3 |
| 931 | 20262057 | 253 | 917 | 22 | 1 | 3 |
| 932 | 20262058 | 273 | 1089 | 33 | 5 | 4 |
| 933 | 20262059 | 175 | 665 | 13 | 4 | 1 |
| 934 | 20262060 | 298 | 1729 | 40 | 5 | 2 |
| 935 | 20262061 | 230 | 955 | 20 | 1 | 4 |
| 936 | 20262062 | 223 | 1022 | 18 | 5 | 2 |
| 937 | 20262063 | 211 | 953 | 21 | 2 | 2 |
| 938 | 20262064 | 493 | 2876 | 51 | 2 | 4 |
| 939 | 20262065 | 272 | 1489 | 24 | 6 | 4 |
| 940 | 20262066 | 211 | 1096 | 26 | 2 | 3 |
| 941 | 20262067 | 288 | 1514 | 22 | 2 | 5 |
| 942 | 20262068 | 223 | 1068 | 23 | 6 | 3 |
| 943 | 20262069 | 218 | 1096 | 20 | 0 | 4 |
| 944 | 20262070 | 233 | 1185 | 25 | 3 | 3 |
| 945 | 20262071 | 239 | 1298 | 17 | 5 | 4 |
| 946 | 20262072 | 161 | 774 | 10 | 5 | 3 |
| 947 | 20262073 | 275 | 1380 | 17 | 3 | 2 |
| 948 | 20262074 | 218 | 1176 | 19 | 0 | 2 |
| 949 | 20262075 | 263 | 1171 | 17 | 10 | 2 |
| 950 | 20262076 | 235 | 1053 | 27 | 4 | 3 |
| 951 | 20262077 | 306 | 2030 | 39 | 5 | 2 |
| 952 | 20262078 | 214 | 746 | 12 | 4 | 4 |
| 953 | 20262079 | 212 | 1011 | 24 | 5 | 2 |
| 954 | 20262080 | 248 | 1117 | 25 | 3 | 3 |
| 955 | 20262081 | 286 | 1567 | 25 | 6 | 4 |
| 956 | 20262082 | 263 | 1166 | 18 | 16 | 3 |
| 957 | 20262083 | 234 | 1057 | 19 | 7 | 3 |
| 958 | 20262084 | 282 | 1508 | 37 | 5 | 2 |
| 959 | 20262085 | 237 | 1237 | 16 | 8 | 4 |
| 960 | 20262086 | 239 | 1046 | 12 | 12 | 3 |
| 961 | 20262087 | 258 | 1138 | 23 | 1 | 3 |
| 962 | 20262088 | 233 | 1246 | 24 | 5 | 2 |
| 963 | 20262089 | 221 | 947 | 17 | 3 | 3 |
| 964 | 20262090 | 244 | 1328 | 15 | 2 | 3 |
| 965 | 20262091 | 220 | 1111 | 18 | 2 | 3 |
| 966 | 20262092 | 257 | 1641 | 19 | 3 | 3 |
| 967 | 20262093 | 244 | 1255 | 16 | 5 | 3 |
| 968 | 20262094 | 200 | 1161 | 22 | 7 | 2 |
| 969 | 20262095 | 333 | 1922 | 29 | 7 | 2 |
| 970 | 20262096 | 253 | 975 | 29 | 1 | 4 |
| 971 | 20262097 | 275 | 1211 | 20 | 0 | 3 |
| 972 | 20262098 | 260 | 1671 | 25 | 1 | 2 |
| 973 | 20262099 | 233 | 985 | 10 | 1 | 2 |
| 974 | 20262100 | 221 | 1159 | 26 | 7 | 2 |
| 975 | 20262101 | 356 | 1909 | 30 | 3 | 4 |
| 976 | 20262102 | 237 | 1295 | 23 | 4 | 2 |
| 977 | 20262103 | 212 | 1099 | 18 | 3 | 3 |
| 978 | 20262104 | 185 | 1078 | 11 | 3 | 0 |
| 979 | 20262105 | 234 | 1264 | 24 | 1 | 1 |
| 980 | 20262106 | 210 | 1015 | 16 | 7 | 3 |
| 981 | 20262107 | 230 | 1109 | 19 | 7 | 4 |
| 982 | 20262108 | 186 | 706 | 13 | 1 | 3 |
| 983 | 20262109 | 210 | 888 | 16 | 3 | 0 |
| 984 | 20262110 | 184 | 857 | 13 | 6 | 1 |
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
| 995 | 20262121 | 265 | 1206 | 18 | 3 | 6 |
| 996 | 20262122 | 214 | 1236 | 25 | 6 | 2 |
| 997 | 20262123 | 230 | 1271 | 18 | 5 | 2 |
| 998 | 20262124 | 306 | 1226 | 21 | 2 | 3 |
| 999 | 20262125 | 226 | 1175 | 20 | 8 | 3 |

## Throughput

- Turns simulated: 250198 (first play of each game)
- Aggregate: 353.2 turns/s over 708.4s wall
- Per game thread: 59.0 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

# Simulation: india-office-rule (2026-11-27)

1000 games, first seed 20261126, brain `scripted`, turn limit off, safety cap 1200, 6 threads, started 2026-11-27T00:42:23Z. Wall time 817.8s.

Source checkout at run start: `6c0a00e99b5ba04e2f12c283d2f8fbc609769aff`; dirty: yes. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.

Command argv: `["./target/release/legatum-sim","--games","1000","--seed","20261126","--threads","6","--difficulty","hard","--label","india-office-rule"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 1000 of 1000 |
| Game length, turns (mean / median / min-max) | 250.4 / 243 / 151-500 |
| Win reason mix | VP hold 95.7%, Capitals 3.9%, Turn limit 0.4% |
| Median end turn | 243 |
| Ending shares of played games (capitals / VP hold / turn limit / other) | 3.9% / 95.7% / 0.4% / 0.0% |
| Games reaching the World War era (any empire) | 992 of 1000 (99.2%) |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1418 / 396 / 959 / 63 |
| Hold break rate | 27.9% of started; 29.2% of resolved (broken / (broken + completed)) |
| Hold wins | 957 |
| First reach to win, turns (mean / median / min-max) | 13.2 / 9 / 9-160 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 154 |
| Wins by a team another team beat to its target | 75 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 957 | 247.5 / 241 / 151-499 |
| Capitals | 39 | 295.0 / 286 / 203-496 |
| Turn limit | 4 | 500.0 / 500 / 500-500 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 100 | 10.0% |
| India | 99 | 9.9% |
| Byzantium | 93 | 9.3% |
| Germany | 124 | 12.4% |
| Celts | 154 | 15.4% |
| Mongols | 71 | 7.1% |
| Sparta | 117 | 11.7% |
| Athens | 138 | 13.8% |
| Aztecs | 104 | 10.4% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 957 | 95.7% |
| Conquest | 0 | 0.0% |
| Capitals | 39 | 3.9% |
| Turn limit | 4 | 0.4% |
| Draw | 0 | 0.0% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 899 | 100 | 11.1% |
| India | 895 | 99 | 11.1% |
| Byzantium | 878 | 93 | 10.6% |
| Germany | 880 | 124 | 14.1% |
| Celts | 896 | 154 | 17.2% |
| Mongols | 877 | 71 | 8.1% |
| Sparta | 891 | 117 | 13.1% |
| Athens | 895 | 138 | 15.4% |
| Aztecs | 889 | 104 | 11.7% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 8000 | 1000 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 899 | 100 | 11.1% |
| India | Hard | 895 | 99 | 11.1% |
| Byzantium | Hard | 878 | 93 | 10.6% |
| Germany | Hard | 880 | 124 | 14.1% |
| Celts | Hard | 896 | 154 | 17.2% |
| Mongols | Hard | 877 | 71 | 8.1% |
| Sparta | Hard | 891 | 117 | 13.1% |
| Athens | Hard | 895 | 138 | 15.4% |
| Aztecs | Hard | 889 | 104 | 11.7% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 899 | 109 | 12.1% | 99 | 146 | 187 | 265 | 452 |
| India | 895 | 419 | 46.8% | 42 | 90 | 127 | 201 | 410 |
| Byzantium | 878 | 607 | 69.1% | 69 | 85 | 109 | 161 | 398 |
| Germany | 880 | 265 | 30.1% | 97 | 128 | 171 | 224 | 385 |
| Celts | 896 | 649 | 72.4% | 31 | 189 | 210 | 210 | 263 |
| Mongols | 877 | 131 | 14.9% | 118 | 156 | 197 | 262 | 344 |
| Sparta | 891 | 171 | 19.2% | 142 | 164 | 208 | 285 | 349 |
| Athens | 895 | 344 | 38.4% | 113 | 174 | 255 | 255 | 255 |
| Aztecs | 889 | 348 | 39.1% | 125 | 195 | 225 | 244 | 318 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 151 | 194 | 243 | 311 | 500 |
| Winning team VP | 2 | 10 | 10 | 13 | 18 |
| First Soldier completed (turn, per empire) | 6 | 8 | 21 | 64 | 269 |
| First Settler completed (turn, per empire) | 3 | 14 | 16 | 32 | 252 |
| First city founded (turn, per empire) | 1 | 1 | 1 | 1 | 1 |
| Second city founded (turn, per empire) | 8 | 18 | 23 | 46 | 311 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 7679 | 321 |
| First Settler completed | 7654 | 346 |
| First city founded | 8000 | 0 |
| Second city founded | 7565 | 435 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 2060 |
| 1 | 1207 |
| 2 | 922 |
| 3 | 744 |
| 4 | 572 |
| 5 | 463 |
| 6 | 354 |
| 7 | 278 |
| 8 | 202 |
| 9 | 166 |
| 10 | 527 |
| 11 | 277 |
| 12 | 121 |
| 13 | 68 |
| 14 | 25 |
| 15 | 10 |
| 16 | 3 |
| 18 | 1 |

## End state

- Average game length: 250.4 turns
- Average cities per empire: 4.28
- Average techs per empire: 31.17
- Average eliminations per game: 2.68

## Strategy diversity: observed behavior (classifier v2)

1000 finished games; 8000 player seats; 1000 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.4373** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 3422 | 210 | 6.1% | 42.8% | 21.0% |
| Wide | 2497 | 711 | 28.5% | 31.2% | 71.1% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 12 | 0 | 0.0% | 0.1% | 0.0% |
| Economic | 621 | 76 | 12.2% | 7.8% | 7.6% |
| Generalist | 1448 | 3 | 0.2% | 18.1% | 0.3% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 19.4% / 3.4% (963) | n/a / n/a (0) | 38.9% / 4.9% (12) | 22.8% / 6.9% (462) | 81.8% / 2.2% (894) |
| Wide | 77.3% / 3.4% (963) | — | n/a / n/a (0) | 59.8% / 9.8% (11) | 57.2% / 5.2% (449) | 95.0% / 1.1% (878) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 56.2% / 4.9% (12) | 30.3% / 9.8% (11) | n/a / n/a (0) | — | 50.0% / 25.0% (4) | 90.9% / 0.0% (11) |
| Economic | 70.4% / 6.9% (462) | 37.6% / 5.2% (449) | n/a / n/a (0) | 25.0% / 25.0% (4) | — | 93.1% / 3.6% (412) |
| Generalist | 16.0% / 2.2% (894) | 3.9% / 1.1% (878) | n/a / n/a (0) | 9.1% / 0.0% (11) | 3.3% / 3.6% (412) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 6 / 264 | 87 / 375 | 0 / 0 | 0 / 0 | 7 / 104 | 0 / 156 |
| India | 38 / 520 | 61 / 213 | 0 / 0 | 0 / 0 | 0 / 9 | 0 / 153 |
| Byzantium | 0 / 15 | 81 / 397 | 0 / 0 | 0 / 5 | 12 / 135 | 0 / 326 |
| Germany | 21 / 345 | 100 / 302 | 0 / 0 | 0 / 3 | 3 / 44 | 0 / 186 |
| Celts | 19 / 268 | 127 / 405 | 0 / 0 | 0 / 0 | 8 / 87 | 0 / 136 |
| Mongols | 41 / 683 | 26 / 74 | 0 / 0 | 0 / 0 | 4 / 12 | 0 / 108 |
| Sparta | 62 / 693 | 44 / 145 | 0 / 0 | 0 / 4 | 10 / 38 | 1 / 11 |
| Athens | 8 / 272 | 113 / 309 | 0 / 0 | 0 / 0 | 16 / 105 | 1 / 209 |
| Aztecs | 15 / 362 | 72 / 277 | 0 / 0 | 0 / 0 | 16 / 87 | 1 / 163 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Turtle/Wonder. Unobserved: Science. Under-sampled pairs: 9.

- insufficient evidence: 9 under-sampled matchups; 1000 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 15233/1000 | 15.233 | 1000/1000 (100.0%) |
| Peace offers | 100959/1000 | 100.959 | 1000/1000 (100.0%) |
| Peace agreements | 12265/1000 | 12.265 | 999/1000 (99.9%) |
| Battles | 1266311/1000 | 1266.311 | 1000/1000 (100.0%) |
| City captures | 23675/1000 | 23.675 | 1000/1000 (100.0%) |
| Settler captures | 4180/1000 | 4.180 | 955/1000 (95.5%) |
| Eliminations | 2679/1000 | 2.679 | 984/1000 (98.4%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Held Great Works | 6544/8000 | 6544/8000 | 3160/1000 |
| Faction goals | 4984/8000 | 4984/8000 | 1990/1000 |
| Era objectives | 14177/8000 | 14177/8000 | 4136/1000 |
| Capitals taken | 2580/8000 | 2580/8000 | 1520/1000 |
| Total VP | 28285/8000 | 28285/8000 | 10806/1000 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 99/723 (13.7%) | 0 |
| Tied leaders (winner in set) | 263/277 (94.9%) | 0 |
| Any leader set | 362/1000 (36.2%) | 0 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 7536/8000 | 5 | 20 | 29 | 33 | 70 | -2.2 (1000) |
| Medieval | 7303/8000 | 31 | 58 | 62 | 66 | 129 | -1.9 (1000) |
| Renaissance | 7229/8000 | 65 | 85 | 90 | 98 | 167 | -3.5 (1000) |
| Industrial | 7022/8000 | 89 | 112 | 121 | 136 | 243 | -6.0 (1000) |
| Machine Age | 6430/8000 | 109 | 142 | 153 | 173 | 309 | -8.6 (1000) |
| World War | 5175/8000 | 128 | 166 | 182 | 207 | 345 | -10.6 (988) |
| Modern | 3558/8000 | 151 | 192 | 213 | 243 | 384 | -11.5 (886) |
| Future | 1918/8000 | 177 | 218 | 246 | 266 | 431 | -8.3 (625) |

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
| Classical | Code of Laws | 7311/8000 | 59 | 450 AD | 1754 BC |
| Classical | Philosophy | 7304/8000 | 61 | 530 AD | 500 BC |
| Medieval | Feudalism | 7278/8000 | 75 | 950 AD | n/a |
| Medieval | Machinery | 7289/8000 | 70 | 800 AD | n/a |
| Medieval | Chivalry | 6445/8000 | 81 | 1130 AD | n/a |
| Medieval | Education | 7247/8000 | 86 | 1280 AD | 1088 AD |
| Medieval | Crop Rotation | 7232/8000 | 90 | 1400 AD | n/a |
| Renaissance | Gunpowder | 7215/8000 | 97 | 1470 AD | n/a |
| Renaissance | Metallurgy | 7146/8000 | 108 | 1580 AD | n/a |
| Renaissance | Military Engineering | 7175/8000 | 105 | 1550 AD | n/a |
| Renaissance | Banking | 7074/8000 | 118 | 1680 AD | 1397 AD |
| Renaissance | Printing Press | 7035/8000 | 119 | 1690 AD | 1440 AD |
| Industrial | Rifling | 6637/8000 | 143 | 1816 AD | n/a |
| Industrial | Military Science | 6843/8000 | 134 | 1783 AD | n/a |
| Industrial | Ballistics | 6710/8000 | 146 | 1828 AD | n/a |
| Industrial | Industrialization | 6934/8000 | 130 | 1768 AD | 1769 AD |
| Industrial | Railways | 6437/8000 | 153 | 1854 AD | 1825 AD |
| Machine Age | Replaceable Parts | 5886/8000 | 168 | 1889 AD | 1800 AD |
| Machine Age | Combustion | 6148/8000 | 162 | 1882 AD | 1876 AD |
| Machine Age | Steel | 5863/8000 | 167 | 1887 AD | 1856 AD |
| Machine Age | Flight | 5452/8000 | 176 | 1898 AD | 1903 AD |
| Machine Age | Electricity | 5222/8000 | 182 | 1904 AD | 1879 AD |
| World War | Mass Mobilisation | 4413/8000 | 197 | 1924 AD | n/a |
| World War | Rocketry | 4150/8000 | 202 | 1931 AD | 1926 AD |
| World War | Armoured Warfare | 3961/8000 | 207 | 1938 AD | 1916 AD |
| World War | Air Power | 3750/8000 | 210 | 1942 AD | n/a |
| World War | Electronics | 4861/8000 | 189 | 1912 AD | n/a |
| Modern | Mechanised Warfare | 2648/8000 | 228 | 1981 AD | n/a |
| Modern | Composite Armour | 2368/8000 | 233 | 1993 AD | n/a |
| Modern | Guided Missiles | 2239/8000 | 239 | 2007 AD | n/a |
| Modern | Jet Engines | 2170/8000 | 242 | 2013 AD | 1939 AD |
| Modern | Green Revolution | 3064/8000 | 221 | 1964 AD | n/a |
| Future | Autonomous Systems | 1285/8000 | 256 | 2032 AD | n/a |
| Future | Robotics | 1099/8000 | 263 | 2039 AD | n/a |
| Future | Fusion Power | 1059/8000 | 256 | 2032 AD | n/a |
| Future | Stealth Technology | 1541/8000 | 255 | 2031 AD | n/a |
| Future | Recursive AI | 963/8000 | 256 | 2032 AD | n/a |

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
| 0 | 0.00 | 63.33 |
| 1 | 72.61 | 14.26 |
| 2 | 28.22 | 5.56 |
| 3 | 8.04 | 1.34 |
| 4 | 1.67 | 0.35 |
| 5 | 0.32 | 0.08 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 20261126 | 241 | 1283 | 22 | 3 | 3 |
| 1 | 20261127 | 228 | 997 | 20 | 4 | 3 |
| 2 | 20261128 | 320 | 1925 | 17 | 5 | 3 |
| 3 | 20261129 | 312 | 1425 | 33 | 2 | 3 |
| 4 | 20261130 | 199 | 972 | 12 | 7 | 2 |
| 5 | 20261131 | 243 | 1091 | 29 | 4 | 3 |
| 6 | 20261132 | 278 | 1699 | 16 | 3 | 3 |
| 7 | 20261133 | 273 | 1336 | 23 | 3 | 4 |
| 8 | 20261134 | 272 | 1317 | 25 | 5 | 3 |
| 9 | 20261135 | 175 | 1009 | 13 | 3 | 1 |
| 10 | 20261136 | 282 | 1416 | 37 | 5 | 3 |
| 11 | 20261137 | 356 | 2066 | 27 | 3 | 3 |
| 12 | 20261138 | 227 | 1263 | 28 | 2 | 1 |
| 13 | 20261139 | 280 | 1309 | 38 | 2 | 2 |
| 14 | 20261140 | 209 | 1066 | 22 | 4 | 4 |
| 15 | 20261141 | 218 | 1199 | 22 | 7 | 2 |
| 16 | 20261142 | 212 | 1052 | 25 | 4 | 2 |
| 17 | 20261143 | 305 | 1819 | 19 | 2 | 0 |
| 18 | 20261144 | 219 | 984 | 23 | 3 | 4 |
| 19 | 20261145 | 288 | 1292 | 38 | 5 | 2 |
| 20 | 20261146 | 320 | 1770 | 33 | 2 | 3 |
| 21 | 20261147 | 286 | 1429 | 23 | 12 | 3 |
| 22 | 20261148 | 234 | 1175 | 27 | 2 | 3 |
| 23 | 20261149 | 224 | 1372 | 23 | 1 | 2 |
| 24 | 20261150 | 271 | 1573 | 29 | 7 | 3 |
| 25 | 20261151 | 362 | 1626 | 45 | 0 | 5 |
| 26 | 20261152 | 314 | 1716 | 33 | 6 | 4 |
| 27 | 20261153 | 255 | 1422 | 18 | 6 | 4 |
| 28 | 20261154 | 244 | 1395 | 27 | 9 | 2 |
| 29 | 20261155 | 292 | 1491 | 24 | 4 | 4 |
| 30 | 20261156 | 301 | 1838 | 42 | 0 | 4 |
| 31 | 20261157 | 240 | 1447 | 26 | 2 | 2 |
| 32 | 20261158 | 260 | 1298 | 34 | 6 | 5 |
| 33 | 20261159 | 256 | 1426 | 32 | 2 | 5 |
| 34 | 20261160 | 278 | 1462 | 25 | 4 | 3 |
| 35 | 20261161 | 246 | 883 | 17 | 3 | 2 |
| 36 | 20261162 | 260 | 1046 | 20 | 1 | 4 |
| 37 | 20261163 | 227 | 1244 | 24 | 7 | 4 |
| 38 | 20261164 | 177 | 696 | 13 | 2 | 3 |
| 39 | 20261165 | 230 | 1374 | 29 | 2 | 2 |
| 40 | 20261166 | 175 | 904 | 15 | 5 | 2 |
| 41 | 20261167 | 266 | 1384 | 30 | 5 | 4 |
| 42 | 20261168 | 204 | 1252 | 24 | 4 | 2 |
| 43 | 20261169 | 330 | 1967 | 20 | 7 | 2 |
| 44 | 20261170 | 500 | 2828 | 25 | 4 | 6 |
| 45 | 20261171 | 190 | 704 | 14 | 3 | 3 |
| 46 | 20261172 | 304 | 1657 | 35 | 1 | 3 |
| 47 | 20261173 | 253 | 1076 | 21 | 3 | 3 |
| 48 | 20261174 | 219 | 938 | 19 | 4 | 4 |
| 49 | 20261175 | 257 | 1209 | 26 | 1 | 3 |
| 50 | 20261176 | 296 | 1336 | 23 | 3 | 2 |
| 51 | 20261177 | 265 | 1399 | 35 | 6 | 4 |
| 52 | 20261178 | 180 | 644 | 9 | 0 | 1 |
| 53 | 20261179 | 275 | 1361 | 28 | 5 | 3 |
| 54 | 20261180 | 250 | 1218 | 13 | 2 | 3 |
| 55 | 20261181 | 193 | 823 | 18 | 5 | 2 |
| 56 | 20261182 | 271 | 1242 | 36 | 7 | 4 |
| 57 | 20261183 | 301 | 1726 | 35 | 2 | 3 |
| 58 | 20261184 | 266 | 1288 | 31 | 3 | 6 |
| 59 | 20261185 | 238 | 1015 | 27 | 3 | 3 |
| 60 | 20261186 | 203 | 860 | 26 | 1 | 3 |
| 61 | 20261187 | 347 | 1267 | 36 | 10 | 3 |
| 62 | 20261188 | 500 | 2985 | 63 | 2 | 5 |
| 63 | 20261189 | 330 | 2168 | 25 | 3 | 2 |
| 64 | 20261190 | 210 | 1045 | 27 | 7 | 4 |
| 65 | 20261191 | 231 | 1233 | 18 | 9 | 3 |
| 66 | 20261192 | 309 | 1774 | 35 | 3 | 3 |
| 67 | 20261193 | 227 | 1301 | 14 | 3 | 4 |
| 68 | 20261194 | 264 | 1276 | 23 | 4 | 2 |
| 69 | 20261195 | 201 | 1075 | 14 | 9 | 3 |
| 70 | 20261196 | 192 | 860 | 10 | 6 | 1 |
| 71 | 20261197 | 234 | 1238 | 26 | 2 | 4 |
| 72 | 20261198 | 264 | 1543 | 25 | 6 | 2 |
| 73 | 20261199 | 225 | 1035 | 21 | 9 | 3 |
| 74 | 20261200 | 273 | 1638 | 30 | 2 | 3 |
| 75 | 20261201 | 275 | 1148 | 37 | 8 | 4 |
| 76 | 20261202 | 233 | 1411 | 29 | 17 | 2 |
| 77 | 20261203 | 285 | 1556 | 16 | 6 | 4 |
| 78 | 20261204 | 261 | 1127 | 15 | 12 | 2 |
| 79 | 20261205 | 195 | 923 | 24 | 4 | 2 |
| 80 | 20261206 | 188 | 604 | 14 | 7 | 3 |
| 81 | 20261207 | 213 | 921 | 12 | 7 | 4 |
| 82 | 20261208 | 251 | 1410 | 17 | 6 | 4 |
| 83 | 20261209 | 262 | 1628 | 24 | 2 | 1 |
| 84 | 20261210 | 293 | 1220 | 30 | 4 | 5 |
| 85 | 20261211 | 201 | 1069 | 35 | 0 | 1 |
| 86 | 20261212 | 264 | 1327 | 22 | 2 | 4 |
| 87 | 20261213 | 162 | 394 | 11 | 8 | 2 |
| 88 | 20261214 | 289 | 1118 | 26 | 6 | 3 |
| 89 | 20261215 | 247 | 1306 | 20 | 4 | 2 |
| 90 | 20261216 | 219 | 1113 | 24 | 5 | 2 |
| 91 | 20261217 | 243 | 1236 | 27 | 3 | 3 |
| 92 | 20261218 | 260 | 1123 | 27 | 3 | 3 |
| 93 | 20261219 | 260 | 1096 | 9 | 3 | 3 |
| 94 | 20261220 | 210 | 993 | 19 | 4 | 3 |
| 95 | 20261221 | 286 | 1386 | 37 | 2 | 4 |
| 96 | 20261222 | 305 | 2010 | 51 | 6 | 3 |
| 97 | 20261223 | 214 | 920 | 17 | 3 | 2 |
| 98 | 20261224 | 292 | 1531 | 35 | 6 | 1 |
| 99 | 20261225 | 257 | 1542 | 32 | 4 | 2 |
| 100 | 20261226 | 231 | 1212 | 27 | 4 | 5 |
| 101 | 20261227 | 268 | 1186 | 27 | 6 | 4 |
| 102 | 20261228 | 369 | 1832 | 31 | 5 | 4 |
| 103 | 20261229 | 208 | 912 | 26 | 6 | 3 |
| 104 | 20261230 | 326 | 1641 | 19 | 4 | 3 |
| 105 | 20261231 | 221 | 964 | 19 | 1 | 4 |
| 106 | 20261232 | 202 | 1022 | 17 | 4 | 1 |
| 107 | 20261233 | 322 | 1549 | 28 | 2 | 1 |
| 108 | 20261234 | 206 | 597 | 21 | 9 | 3 |
| 109 | 20261235 | 197 | 690 | 11 | 2 | 3 |
| 110 | 20261236 | 212 | 878 | 15 | 4 | 2 |
| 111 | 20261237 | 228 | 1193 | 21 | 3 | 3 |
| 112 | 20261238 | 201 | 954 | 23 | 2 | 3 |
| 113 | 20261239 | 305 | 1813 | 20 | 3 | 2 |
| 114 | 20261240 | 264 | 1434 | 26 | 4 | 2 |
| 115 | 20261241 | 259 | 1070 | 15 | 4 | 3 |
| 116 | 20261242 | 264 | 1552 | 30 | 2 | 3 |
| 117 | 20261243 | 236 | 1333 | 26 | 5 | 3 |
| 118 | 20261244 | 170 | 833 | 8 | 4 | 1 |
| 119 | 20261245 | 253 | 1251 | 27 | 2 | 3 |
| 120 | 20261246 | 235 | 1086 | 21 | 5 | 0 |
| 121 | 20261247 | 179 | 718 | 11 | 3 | 4 |
| 122 | 20261248 | 265 | 1364 | 27 | 4 | 4 |
| 123 | 20261249 | 203 | 1070 | 22 | 1 | 1 |
| 124 | 20261250 | 183 | 806 | 16 | 5 | 1 |
| 125 | 20261251 | 214 | 972 | 15 | 3 | 4 |
| 126 | 20261252 | 288 | 1120 | 34 | 4 | 3 |
| 127 | 20261253 | 184 | 636 | 12 | 5 | 2 |
| 128 | 20261254 | 234 | 1402 | 25 | 6 | 3 |
| 129 | 20261255 | 232 | 1133 | 23 | 2 | 3 |
| 130 | 20261256 | 231 | 1249 | 29 | 2 | 3 |
| 131 | 20261257 | 280 | 1318 | 16 | 1 | 2 |
| 132 | 20261258 | 212 | 848 | 10 | 2 | 4 |
| 133 | 20261259 | 190 | 841 | 17 | 5 | 3 |
| 134 | 20261260 | 259 | 1415 | 17 | 2 | 2 |
| 135 | 20261261 | 222 | 936 | 25 | 2 | 2 |
| 136 | 20261262 | 232 | 882 | 27 | 1 | 4 |
| 137 | 20261263 | 253 | 1496 | 19 | 0 | 2 |
| 138 | 20261264 | 235 | 1269 | 29 | 5 | 4 |
| 139 | 20261265 | 384 | 2192 | 35 | 4 | 3 |
| 140 | 20261266 | 233 | 1425 | 29 | 7 | 2 |
| 141 | 20261267 | 230 | 1088 | 28 | 5 | 4 |
| 142 | 20261268 | 238 | 1233 | 22 | 2 | 0 |
| 143 | 20261269 | 187 | 874 | 18 | 8 | 2 |
| 144 | 20261270 | 252 | 956 | 23 | 0 | 3 |
| 145 | 20261271 | 223 | 1128 | 21 | 1 | 3 |
| 146 | 20261272 | 259 | 1366 | 11 | 8 | 2 |
| 147 | 20261273 | 215 | 861 | 17 | 10 | 1 |
| 148 | 20261274 | 246 | 1249 | 29 | 1 | 2 |
| 149 | 20261275 | 248 | 1158 | 19 | 6 | 2 |
| 150 | 20261276 | 260 | 1259 | 12 | 6 | 1 |
| 151 | 20261277 | 303 | 1518 | 29 | 8 | 5 |
| 152 | 20261278 | 282 | 1289 | 26 | 10 | 4 |
| 153 | 20261279 | 270 | 1423 | 21 | 12 | 4 |
| 154 | 20261280 | 225 | 1177 | 33 | 3 | 2 |
| 155 | 20261281 | 249 | 1216 | 23 | 6 | 3 |
| 156 | 20261282 | 225 | 1208 | 21 | 4 | 3 |
| 157 | 20261283 | 238 | 1122 | 20 | 2 | 3 |
| 158 | 20261284 | 266 | 1155 | 16 | 8 | 3 |
| 159 | 20261285 | 235 | 1388 | 26 | 6 | 3 |
| 160 | 20261286 | 188 | 982 | 21 | 3 | 1 |
| 161 | 20261287 | 251 | 1218 | 23 | 5 | 2 |
| 162 | 20261288 | 186 | 1085 | 8 | 4 | 2 |
| 163 | 20261289 | 225 | 1049 | 15 | 2 | 2 |
| 164 | 20261290 | 238 | 1032 | 23 | 6 | 1 |
| 165 | 20261291 | 312 | 1703 | 41 | 4 | 3 |
| 166 | 20261292 | 293 | 1654 | 26 | 5 | 2 |
| 167 | 20261293 | 251 | 1492 | 25 | 12 | 4 |
| 168 | 20261294 | 232 | 1238 | 23 | 3 | 4 |
| 169 | 20261295 | 240 | 1262 | 17 | 6 | 4 |
| 170 | 20261296 | 235 | 981 | 18 | 3 | 2 |
| 171 | 20261297 | 215 | 659 | 18 | 1 | 3 |
| 172 | 20261298 | 274 | 1423 | 32 | 2 | 4 |
| 173 | 20261299 | 329 | 1594 | 38 | 3 | 4 |
| 174 | 20261300 | 261 | 1525 | 23 | 6 | 1 |
| 175 | 20261301 | 239 | 1199 | 22 | 2 | 3 |
| 176 | 20261302 | 272 | 1462 | 29 | 5 | 3 |
| 177 | 20261303 | 217 | 1111 | 26 | 1 | 5 |
| 178 | 20261304 | 189 | 805 | 9 | 0 | 5 |
| 179 | 20261305 | 214 | 1185 | 16 | 5 | 2 |
| 180 | 20261306 | 277 | 1042 | 23 | 0 | 3 |
| 181 | 20261307 | 267 | 1432 | 36 | 2 | 1 |
| 182 | 20261308 | 219 | 1089 | 31 | 3 | 2 |
| 183 | 20261309 | 286 | 1439 | 35 | 4 | 4 |
| 184 | 20261310 | 294 | 1816 | 20 | 3 | 3 |
| 185 | 20261311 | 205 | 815 | 17 | 3 | 3 |
| 186 | 20261312 | 266 | 1453 | 15 | 9 | 6 |
| 187 | 20261313 | 288 | 1281 | 27 | 5 | 4 |
| 188 | 20261314 | 217 | 1228 | 19 | 5 | 1 |
| 189 | 20261315 | 261 | 1256 | 20 | 9 | 1 |
| 190 | 20261316 | 223 | 978 | 17 | 5 | 3 |
| 191 | 20261317 | 461 | 1972 | 53 | 4 | 4 |
| 192 | 20261318 | 294 | 1293 | 27 | 2 | 3 |
| 193 | 20261319 | 293 | 1291 | 26 | 2 | 2 |
| 194 | 20261320 | 238 | 1116 | 21 | 2 | 1 |
| 195 | 20261321 | 280 | 1402 | 38 | 4 | 3 |
| 196 | 20261322 | 204 | 1021 | 12 | 4 | 2 |
| 197 | 20261323 | 259 | 1612 | 25 | 4 | 3 |
| 198 | 20261324 | 186 | 838 | 13 | 2 | 2 |
| 199 | 20261325 | 177 | 898 | 17 | 6 | 3 |
| 200 | 20261326 | 277 | 1794 | 37 | 2 | 3 |
| 201 | 20261327 | 221 | 981 | 15 | 3 | 2 |
| 202 | 20261328 | 262 | 1406 | 33 | 0 | 2 |
| 203 | 20261329 | 215 | 1024 | 13 | 2 | 3 |
| 204 | 20261330 | 288 | 1216 | 38 | 4 | 4 |
| 205 | 20261331 | 318 | 1324 | 27 | 5 | 5 |
| 206 | 20261332 | 302 | 1446 | 16 | 1 | 4 |
| 207 | 20261333 | 230 | 1149 | 25 | 7 | 1 |
| 208 | 20261334 | 272 | 1727 | 23 | 11 | 2 |
| 209 | 20261335 | 272 | 1587 | 40 | 4 | 3 |
| 210 | 20261336 | 223 | 1148 | 14 | 16 | 4 |
| 211 | 20261337 | 250 | 1294 | 24 | 12 | 5 |
| 212 | 20261338 | 224 | 909 | 13 | 6 | 0 |
| 213 | 20261339 | 248 | 1298 | 30 | 9 | 3 |
| 214 | 20261340 | 223 | 814 | 12 | 2 | 3 |
| 215 | 20261341 | 184 | 774 | 16 | 6 | 2 |
| 216 | 20261342 | 270 | 1299 | 21 | 3 | 2 |
| 217 | 20261343 | 267 | 1484 | 24 | 3 | 2 |
| 218 | 20261344 | 239 | 1088 | 22 | 3 | 2 |
| 219 | 20261345 | 220 | 941 | 25 | 1 | 4 |
| 220 | 20261346 | 270 | 1452 | 26 | 2 | 3 |
| 221 | 20261347 | 262 | 1409 | 25 | 2 | 2 |
| 222 | 20261348 | 330 | 1739 | 21 | 8 | 4 |
| 223 | 20261349 | 202 | 869 | 15 | 5 | 4 |
| 224 | 20261350 | 176 | 930 | 16 | 7 | 4 |
| 225 | 20261351 | 353 | 1582 | 37 | 5 | 3 |
| 226 | 20261352 | 250 | 1308 | 32 | 1 | 2 |
| 227 | 20261353 | 186 | 957 | 18 | 4 | 2 |
| 228 | 20261354 | 212 | 798 | 22 | 3 | 2 |
| 229 | 20261355 | 388 | 2460 | 34 | 1 | 4 |
| 230 | 20261356 | 254 | 1205 | 27 | 0 | 3 |
| 231 | 20261357 | 160 | 779 | 13 | 4 | 1 |
| 232 | 20261358 | 194 | 964 | 10 | 5 | 3 |
| 233 | 20261359 | 182 | 767 | 11 | 3 | 1 |
| 234 | 20261360 | 239 | 898 | 16 | 4 | 4 |
| 235 | 20261361 | 274 | 1111 | 23 | 3 | 4 |
| 236 | 20261362 | 347 | 2453 | 38 | 5 | 4 |
| 237 | 20261363 | 302 | 1495 | 29 | 6 | 3 |
| 238 | 20261364 | 201 | 803 | 18 | 5 | 3 |
| 239 | 20261365 | 288 | 1491 | 28 | 4 | 3 |
| 240 | 20261366 | 346 | 1683 | 45 | 6 | 4 |
| 241 | 20261367 | 236 | 1215 | 19 | 1 | 4 |
| 242 | 20261368 | 290 | 1411 | 28 | 5 | 2 |
| 243 | 20261369 | 201 | 878 | 12 | 7 | 1 |
| 244 | 20261370 | 214 | 958 | 26 | 7 | 2 |
| 245 | 20261371 | 264 | 1059 | 29 | 4 | 4 |
| 246 | 20261372 | 265 | 1238 | 23 | 5 | 3 |
| 247 | 20261373 | 253 | 1200 | 24 | 4 | 3 |
| 248 | 20261374 | 246 | 1490 | 11 | 2 | 4 |
| 249 | 20261375 | 224 | 1173 | 24 | 1 | 3 |
| 250 | 20261376 | 229 | 1380 | 22 | 0 | 2 |
| 251 | 20261377 | 204 | 965 | 14 | 4 | 3 |
| 252 | 20261378 | 323 | 1667 | 34 | 1 | 3 |
| 253 | 20261379 | 324 | 1563 | 23 | 5 | 4 |
| 254 | 20261380 | 349 | 2126 | 39 | 3 | 4 |
| 255 | 20261381 | 239 | 1054 | 18 | 5 | 2 |
| 256 | 20261382 | 201 | 838 | 19 | 1 | 4 |
| 257 | 20261383 | 305 | 1419 | 42 | 6 | 6 |
| 258 | 20261384 | 269 | 1482 | 22 | 3 | 3 |
| 259 | 20261385 | 251 | 1058 | 20 | 7 | 3 |
| 260 | 20261386 | 278 | 1265 | 34 | 8 | 2 |
| 261 | 20261387 | 255 | 1174 | 26 | 7 | 2 |
| 262 | 20261388 | 244 | 980 | 25 | 3 | 4 |
| 263 | 20261389 | 278 | 1748 | 24 | 3 | 2 |
| 264 | 20261390 | 244 | 1140 | 35 | 4 | 4 |
| 265 | 20261391 | 220 | 1012 | 15 | 5 | 1 |
| 266 | 20261392 | 201 | 911 | 20 | 0 | 2 |
| 267 | 20261393 | 431 | 2976 | 42 | 2 | 1 |
| 268 | 20261394 | 267 | 1497 | 29 | 8 | 2 |
| 269 | 20261395 | 221 | 1321 | 20 | 7 | 3 |
| 270 | 20261396 | 283 | 1518 | 42 | 0 | 3 |
| 271 | 20261397 | 273 | 1603 | 17 | 4 | 4 |
| 272 | 20261398 | 230 | 1128 | 23 | 1 | 2 |
| 273 | 20261399 | 270 | 1434 | 33 | 5 | 4 |
| 274 | 20261400 | 210 | 621 | 18 | 1 | 3 |
| 275 | 20261401 | 182 | 940 | 19 | 2 | 1 |
| 276 | 20261402 | 230 | 1138 | 18 | 4 | 2 |
| 277 | 20261403 | 258 | 1559 | 40 | 7 | 3 |
| 278 | 20261404 | 222 | 1131 | 20 | 2 | 3 |
| 279 | 20261405 | 254 | 1193 | 23 | 2 | 3 |
| 280 | 20261406 | 215 | 1126 | 22 | 6 | 0 |
| 281 | 20261407 | 276 | 1412 | 28 | 5 | 5 |
| 282 | 20261408 | 208 | 1147 | 19 | 9 | 1 |
| 283 | 20261409 | 322 | 1861 | 28 | 7 | 4 |
| 284 | 20261410 | 315 | 1864 | 31 | 5 | 5 |
| 285 | 20261411 | 257 | 1511 | 34 | 6 | 1 |
| 286 | 20261412 | 243 | 1154 | 19 | 0 | 0 |
| 287 | 20261413 | 189 | 923 | 17 | 4 | 1 |
| 288 | 20261414 | 251 | 1323 | 16 | 3 | 2 |
| 289 | 20261415 | 265 | 1499 | 39 | 10 | 1 |
| 290 | 20261416 | 229 | 1068 | 12 | 2 | 4 |
| 291 | 20261417 | 269 | 1410 | 18 | 2 | 2 |
| 292 | 20261418 | 294 | 1566 | 25 | 1 | 4 |
| 293 | 20261419 | 272 | 1474 | 26 | 9 | 2 |
| 294 | 20261420 | 258 | 1155 | 28 | 4 | 3 |
| 295 | 20261421 | 209 | 1032 | 21 | 9 | 1 |
| 296 | 20261422 | 219 | 1270 | 16 | 2 | 3 |
| 297 | 20261423 | 327 | 2056 | 41 | 9 | 3 |
| 298 | 20261424 | 338 | 1649 | 36 | 2 | 3 |
| 299 | 20261425 | 257 | 1484 | 18 | 1 | 2 |
| 300 | 20261426 | 306 | 1179 | 12 | 3 | 2 |
| 301 | 20261427 | 243 | 1320 | 37 | 1 | 2 |
| 302 | 20261428 | 216 | 1008 | 18 | 6 | 2 |
| 303 | 20261429 | 273 | 1498 | 17 | 3 | 2 |
| 304 | 20261430 | 193 | 896 | 15 | 4 | 1 |
| 305 | 20261431 | 264 | 1108 | 22 | 6 | 2 |
| 306 | 20261432 | 255 | 905 | 29 | 4 | 4 |
| 307 | 20261433 | 260 | 1238 | 31 | 2 | 2 |
| 308 | 20261434 | 201 | 870 | 27 | 5 | 2 |
| 309 | 20261435 | 307 | 1796 | 31 | 2 | 5 |
| 310 | 20261436 | 218 | 1176 | 26 | 1 | 3 |
| 311 | 20261437 | 203 | 887 | 16 | 2 | 2 |
| 312 | 20261438 | 269 | 1412 | 26 | 5 | 4 |
| 313 | 20261439 | 235 | 1102 | 25 | 1 | 2 |
| 314 | 20261440 | 173 | 533 | 9 | 5 | 1 |
| 315 | 20261441 | 187 | 835 | 13 | 2 | 1 |
| 316 | 20261442 | 268 | 1139 | 33 | 8 | 2 |
| 317 | 20261443 | 268 | 1067 | 18 | 2 | 2 |
| 318 | 20261444 | 274 | 1394 | 35 | 6 | 3 |
| 319 | 20261445 | 271 | 1393 | 37 | 0 | 3 |
| 320 | 20261446 | 399 | 2879 | 49 | 6 | 5 |
| 321 | 20261447 | 255 | 1443 | 31 | 0 | 2 |
| 322 | 20261448 | 203 | 994 | 17 | 2 | 1 |
| 323 | 20261449 | 202 | 1152 | 17 | 4 | 1 |
| 324 | 20261450 | 224 | 966 | 21 | 6 | 2 |
| 325 | 20261451 | 258 | 1315 | 28 | 4 | 2 |
| 326 | 20261452 | 249 | 1212 | 15 | 7 | 5 |
| 327 | 20261453 | 261 | 1215 | 23 | 0 | 4 |
| 328 | 20261454 | 236 | 1215 | 19 | 3 | 1 |
| 329 | 20261455 | 196 | 859 | 18 | 8 | 3 |
| 330 | 20261456 | 208 | 1121 | 22 | 0 | 2 |
| 331 | 20261457 | 265 | 1350 | 17 | 5 | 1 |
| 332 | 20261458 | 282 | 1754 | 33 | 7 | 5 |
| 333 | 20261459 | 317 | 1561 | 41 | 6 | 3 |
| 334 | 20261460 | 281 | 1253 | 19 | 8 | 4 |
| 335 | 20261461 | 177 | 716 | 11 | 6 | 2 |
| 336 | 20261462 | 220 | 1119 | 32 | 2 | 1 |
| 337 | 20261463 | 177 | 959 | 15 | 2 | 1 |
| 338 | 20261464 | 233 | 1147 | 25 | 9 | 3 |
| 339 | 20261465 | 236 | 1048 | 22 | 7 | 3 |
| 340 | 20261466 | 235 | 1226 | 21 | 3 | 2 |
| 341 | 20261467 | 344 | 1719 | 33 | 2 | 4 |
| 342 | 20261468 | 216 | 1155 | 19 | 5 | 1 |
| 343 | 20261469 | 276 | 1460 | 30 | 8 | 5 |
| 344 | 20261470 | 294 | 1240 | 26 | 2 | 2 |
| 345 | 20261471 | 233 | 1225 | 17 | 9 | 3 |
| 346 | 20261472 | 322 | 1949 | 19 | 1 | 5 |
| 347 | 20261473 | 262 | 1392 | 25 | 2 | 2 |
| 348 | 20261474 | 236 | 1165 | 20 | 0 | 1 |
| 349 | 20261475 | 210 | 1120 | 22 | 2 | 2 |
| 350 | 20261476 | 214 | 1281 | 25 | 13 | 1 |
| 351 | 20261477 | 313 | 1532 | 34 | 3 | 4 |
| 352 | 20261478 | 223 | 907 | 19 | 5 | 2 |
| 353 | 20261479 | 228 | 1167 | 16 | 3 | 3 |
| 354 | 20261480 | 265 | 1202 | 26 | 3 | 4 |
| 355 | 20261481 | 261 | 1390 | 26 | 1 | 3 |
| 356 | 20261482 | 234 | 1253 | 27 | 4 | 3 |
| 357 | 20261483 | 232 | 912 | 27 | 6 | 1 |
| 358 | 20261484 | 253 | 1269 | 17 | 5 | 2 |
| 359 | 20261485 | 271 | 1799 | 41 | 2 | 5 |
| 360 | 20261486 | 247 | 836 | 21 | 6 | 3 |
| 361 | 20261487 | 215 | 1015 | 25 | 5 | 4 |
| 362 | 20261488 | 415 | 2736 | 43 | 2 | 3 |
| 363 | 20261489 | 248 | 1176 | 15 | 3 | 1 |
| 364 | 20261490 | 288 | 1427 | 25 | 6 | 3 |
| 365 | 20261491 | 184 | 647 | 13 | 0 | 3 |
| 366 | 20261492 | 254 | 1200 | 28 | 6 | 2 |
| 367 | 20261493 | 238 | 1207 | 23 | 3 | 1 |
| 368 | 20261494 | 496 | 2639 | 43 | 17 | 5 |
| 369 | 20261495 | 228 | 1274 | 18 | 9 | 1 |
| 370 | 20261496 | 209 | 956 | 27 | 4 | 2 |
| 371 | 20261497 | 274 | 1698 | 24 | 5 | 3 |
| 372 | 20261498 | 315 | 1585 | 34 | 2 | 3 |
| 373 | 20261499 | 293 | 1658 | 34 | 3 | 1 |
| 374 | 20261500 | 216 | 1098 | 11 | 1 | 2 |
| 375 | 20261501 | 168 | 735 | 13 | 5 | 1 |
| 376 | 20261502 | 217 | 1039 | 20 | 7 | 2 |
| 377 | 20261503 | 221 | 1013 | 23 | 1 | 4 |
| 378 | 20261504 | 170 | 896 | 15 | 5 | 2 |
| 379 | 20261505 | 181 | 1020 | 12 | 1 | 1 |
| 380 | 20261506 | 222 | 1094 | 28 | 8 | 2 |
| 381 | 20261507 | 229 | 1167 | 21 | 4 | 2 |
| 382 | 20261508 | 236 | 996 | 27 | 3 | 3 |
| 383 | 20261509 | 224 | 1274 | 25 | 5 | 3 |
| 384 | 20261510 | 201 | 798 | 17 | 2 | 2 |
| 385 | 20261511 | 214 | 1049 | 23 | 4 | 3 |
| 386 | 20261512 | 195 | 1009 | 21 | 7 | 2 |
| 387 | 20261513 | 209 | 1152 | 20 | 3 | 2 |
| 388 | 20261514 | 182 | 1023 | 18 | 2 | 3 |
| 389 | 20261515 | 221 | 1230 | 6 | 6 | 3 |
| 390 | 20261516 | 218 | 1106 | 14 | 9 | 3 |
| 391 | 20261517 | 242 | 1186 | 21 | 4 | 4 |
| 392 | 20261518 | 258 | 1114 | 17 | 5 | 0 |
| 393 | 20261519 | 314 | 1107 | 29 | 11 | 3 |
| 394 | 20261520 | 254 | 1144 | 19 | 3 | 4 |
| 395 | 20261521 | 160 | 733 | 15 | 4 | 1 |
| 396 | 20261522 | 232 | 1253 | 16 | 8 | 3 |
| 397 | 20261523 | 172 | 819 | 12 | 7 | 1 |
| 398 | 20261524 | 253 | 1160 | 26 | 11 | 3 |
| 399 | 20261525 | 239 | 1264 | 21 | 6 | 2 |
| 400 | 20261526 | 219 | 994 | 18 | 2 | 5 |
| 401 | 20261527 | 243 | 1124 | 22 | 6 | 2 |
| 402 | 20261528 | 220 | 719 | 15 | 2 | 1 |
| 403 | 20261529 | 207 | 1038 | 25 | 8 | 3 |
| 404 | 20261530 | 183 | 599 | 17 | 3 | 2 |
| 405 | 20261531 | 242 | 1182 | 14 | 0 | 3 |
| 406 | 20261532 | 313 | 1912 | 23 | 7 | 2 |
| 407 | 20261533 | 285 | 1450 | 32 | 4 | 2 |
| 408 | 20261534 | 217 | 1246 | 22 | 10 | 2 |
| 409 | 20261535 | 264 | 1588 | 29 | 5 | 3 |
| 410 | 20261536 | 200 | 1022 | 25 | 0 | 1 |
| 411 | 20261537 | 205 | 987 | 15 | 8 | 2 |
| 412 | 20261538 | 397 | 2343 | 45 | 10 | 4 |
| 413 | 20261539 | 190 | 771 | 24 | 3 | 3 |
| 414 | 20261540 | 290 | 1367 | 23 | 1 | 2 |
| 415 | 20261541 | 187 | 1032 | 23 | 2 | 2 |
| 416 | 20261542 | 286 | 1525 | 32 | 1 | 2 |
| 417 | 20261543 | 297 | 1319 | 42 | 4 | 4 |
| 418 | 20261544 | 238 | 1229 | 27 | 2 | 2 |
| 419 | 20261545 | 265 | 1099 | 21 | 5 | 5 |
| 420 | 20261546 | 279 | 1348 | 35 | 8 | 1 |
| 421 | 20261547 | 189 | 1003 | 22 | 3 | 3 |
| 422 | 20261548 | 250 | 1301 | 30 | 4 | 3 |
| 423 | 20261549 | 197 | 1058 | 8 | 4 | 3 |
| 424 | 20261550 | 270 | 1771 | 29 | 2 | 1 |
| 425 | 20261551 | 268 | 1527 | 34 | 3 | 2 |
| 426 | 20261552 | 245 | 1534 | 38 | 4 | 1 |
| 427 | 20261553 | 217 | 1347 | 19 | 1 | 0 |
| 428 | 20261554 | 184 | 813 | 17 | 5 | 2 |
| 429 | 20261555 | 392 | 2145 | 35 | 6 | 4 |
| 430 | 20261556 | 227 | 1160 | 14 | 2 | 5 |
| 431 | 20261557 | 224 | 1198 | 24 | 5 | 3 |
| 432 | 20261558 | 238 | 1389 | 17 | 3 | 1 |
| 433 | 20261559 | 194 | 1070 | 13 | 4 | 2 |
| 434 | 20261560 | 297 | 1865 | 39 | 0 | 3 |
| 435 | 20261561 | 249 | 1362 | 28 | 3 | 1 |
| 436 | 20261562 | 280 | 1250 | 20 | 5 | 2 |
| 437 | 20261563 | 237 | 1485 | 26 | 3 | 2 |
| 438 | 20261564 | 254 | 1292 | 23 | 5 | 2 |
| 439 | 20261565 | 197 | 1080 | 23 | 7 | 2 |
| 440 | 20261566 | 254 | 1386 | 27 | 7 | 2 |
| 441 | 20261567 | 218 | 1159 | 20 | 7 | 1 |
| 442 | 20261568 | 234 | 1213 | 20 | 10 | 2 |
| 443 | 20261569 | 200 | 1081 | 18 | 4 | 3 |
| 444 | 20261570 | 215 | 1112 | 17 | 6 | 3 |
| 445 | 20261571 | 254 | 1364 | 32 | 3 | 1 |
| 446 | 20261572 | 258 | 1238 | 19 | 11 | 2 |
| 447 | 20261573 | 228 | 1329 | 25 | 2 | 2 |
| 448 | 20261574 | 209 | 873 | 26 | 4 | 1 |
| 449 | 20261575 | 264 | 1377 | 22 | 4 | 3 |
| 450 | 20261576 | 290 | 1283 | 34 | 6 | 3 |
| 451 | 20261577 | 221 | 1032 | 23 | 4 | 4 |
| 452 | 20261578 | 230 | 1119 | 19 | 8 | 3 |
| 453 | 20261579 | 334 | 1536 | 26 | 4 | 2 |
| 454 | 20261580 | 362 | 2324 | 43 | 9 | 3 |
| 455 | 20261581 | 185 | 961 | 16 | 5 | 2 |
| 456 | 20261582 | 262 | 1256 | 17 | 5 | 3 |
| 457 | 20261583 | 500 | 2873 | 41 | 12 | 4 |
| 458 | 20261584 | 273 | 1672 | 32 | 6 | 4 |
| 459 | 20261585 | 252 | 1292 | 41 | 4 | 2 |
| 460 | 20261586 | 388 | 2238 | 54 | 6 | 4 |
| 461 | 20261587 | 218 | 1136 | 18 | 7 | 3 |
| 462 | 20261588 | 233 | 1143 | 14 | 3 | 3 |
| 463 | 20261589 | 320 | 1776 | 27 | 4 | 2 |
| 464 | 20261590 | 279 | 1421 | 23 | 4 | 3 |
| 465 | 20261591 | 242 | 1380 | 22 | 0 | 3 |
| 466 | 20261592 | 208 | 1092 | 18 | 6 | 2 |
| 467 | 20261593 | 273 | 1616 | 23 | 5 | 2 |
| 468 | 20261594 | 259 | 1420 | 29 | 1 | 3 |
| 469 | 20261595 | 410 | 2102 | 40 | 5 | 5 |
| 470 | 20261596 | 182 | 874 | 12 | 3 | 1 |
| 471 | 20261597 | 361 | 2103 | 23 | 4 | 2 |
| 472 | 20261598 | 220 | 1074 | 18 | 4 | 1 |
| 473 | 20261599 | 207 | 1331 | 26 | 1 | 1 |
| 474 | 20261600 | 206 | 955 | 21 | 5 | 2 |
| 475 | 20261601 | 360 | 1845 | 38 | 4 | 3 |
| 476 | 20261602 | 180 | 650 | 13 | 2 | 1 |
| 477 | 20261603 | 170 | 716 | 20 | 1 | 2 |
| 478 | 20261604 | 218 | 1062 | 16 | 4 | 3 |
| 479 | 20261605 | 274 | 1826 | 36 | 2 | 2 |
| 480 | 20261606 | 208 | 1052 | 17 | 6 | 2 |
| 481 | 20261607 | 182 | 821 | 17 | 4 | 2 |
| 482 | 20261608 | 354 | 1815 | 47 | 6 | 3 |
| 483 | 20261609 | 426 | 2694 | 31 | 5 | 2 |
| 484 | 20261610 | 238 | 1226 | 24 | 1 | 1 |
| 485 | 20261611 | 252 | 1223 | 23 | 4 | 2 |
| 486 | 20261612 | 188 | 811 | 17 | 1 | 2 |
| 487 | 20261613 | 240 | 1171 | 15 | 1 | 3 |
| 488 | 20261614 | 271 | 1426 | 37 | 3 | 5 |
| 489 | 20261615 | 222 | 1001 | 18 | 5 | 1 |
| 490 | 20261616 | 251 | 1306 | 32 | 7 | 2 |
| 491 | 20261617 | 238 | 1250 | 20 | 2 | 3 |
| 492 | 20261618 | 284 | 1593 | 25 | 5 | 2 |
| 493 | 20261619 | 207 | 938 | 26 | 5 | 4 |
| 494 | 20261620 | 212 | 1158 | 25 | 2 | 3 |
| 495 | 20261621 | 216 | 1252 | 29 | 0 | 1 |
| 496 | 20261622 | 200 | 883 | 22 | 4 | 3 |
| 497 | 20261623 | 267 | 1512 | 24 | 3 | 5 |
| 498 | 20261624 | 254 | 1250 | 20 | 6 | 2 |
| 499 | 20261625 | 175 | 878 | 9 | 8 | 0 |
| 500 | 20261626 | 251 | 790 | 17 | 0 | 2 |
| 501 | 20261627 | 266 | 1527 | 28 | 1 | 3 |
| 502 | 20261628 | 237 | 999 | 22 | 1 | 4 |
| 503 | 20261629 | 235 | 1011 | 20 | 2 | 2 |
| 504 | 20261630 | 270 | 1574 | 33 | 6 | 3 |
| 505 | 20261631 | 267 | 1413 | 15 | 5 | 3 |
| 506 | 20261632 | 198 | 591 | 10 | 1 | 1 |
| 507 | 20261633 | 293 | 1417 | 20 | 6 | 2 |
| 508 | 20261634 | 199 | 1147 | 19 | 3 | 1 |
| 509 | 20261635 | 279 | 1422 | 26 | 10 | 5 |
| 510 | 20261636 | 217 | 913 | 19 | 6 | 1 |
| 511 | 20261637 | 280 | 1455 | 35 | 2 | 5 |
| 512 | 20261638 | 276 | 1543 | 37 | 1 | 5 |
| 513 | 20261639 | 323 | 1701 | 38 | 8 | 3 |
| 514 | 20261640 | 218 | 1464 | 21 | 1 | 2 |
| 515 | 20261641 | 369 | 2089 | 42 | 5 | 5 |
| 516 | 20261642 | 261 | 1559 | 21 | 7 | 3 |
| 517 | 20261643 | 280 | 1368 | 14 | 3 | 5 |
| 518 | 20261644 | 216 | 779 | 8 | 2 | 5 |
| 519 | 20261645 | 211 | 903 | 10 | 2 | 3 |
| 520 | 20261646 | 263 | 1389 | 31 | 6 | 3 |
| 521 | 20261647 | 223 | 957 | 30 | 6 | 3 |
| 522 | 20261648 | 251 | 1245 | 23 | 5 | 3 |
| 523 | 20261649 | 166 | 717 | 7 | 7 | 1 |
| 524 | 20261650 | 204 | 848 | 21 | 3 | 3 |
| 525 | 20261651 | 251 | 1231 | 34 | 7 | 3 |
| 526 | 20261652 | 212 | 959 | 17 | 5 | 2 |
| 527 | 20261653 | 218 | 1096 | 16 | 3 | 2 |
| 528 | 20261654 | 246 | 1104 | 19 | 5 | 3 |
| 529 | 20261655 | 252 | 1653 | 26 | 7 | 1 |
| 530 | 20261656 | 232 | 1032 | 17 | 6 | 6 |
| 531 | 20261657 | 255 | 1032 | 14 | 2 | 4 |
| 532 | 20261658 | 246 | 1175 | 29 | 11 | 3 |
| 533 | 20261659 | 321 | 1650 | 29 | 2 | 3 |
| 534 | 20261660 | 209 | 741 | 16 | 3 | 3 |
| 535 | 20261661 | 239 | 1158 | 23 | 4 | 1 |
| 536 | 20261662 | 199 | 947 | 15 | 0 | 2 |
| 537 | 20261663 | 189 | 953 | 21 | 7 | 1 |
| 538 | 20261664 | 248 | 1259 | 30 | 3 | 2 |
| 539 | 20261665 | 274 | 1444 | 31 | 11 | 3 |
| 540 | 20261666 | 319 | 1525 | 36 | 6 | 4 |
| 541 | 20261667 | 289 | 1401 | 35 | 2 | 5 |
| 542 | 20261668 | 226 | 1002 | 15 | 6 | 3 |
| 543 | 20261669 | 243 | 1231 | 31 | 4 | 3 |
| 544 | 20261670 | 261 | 1025 | 17 | 4 | 1 |
| 545 | 20261671 | 286 | 1472 | 39 | 2 | 5 |
| 546 | 20261672 | 240 | 957 | 30 | 6 | 3 |
| 547 | 20261673 | 321 | 1846 | 28 | 5 | 3 |
| 548 | 20261674 | 261 | 1437 | 40 | 6 | 4 |
| 549 | 20261675 | 232 | 1275 | 27 | 7 | 1 |
| 550 | 20261676 | 274 | 1469 | 24 | 5 | 3 |
| 551 | 20261677 | 233 | 1166 | 26 | 0 | 2 |
| 552 | 20261678 | 251 | 1500 | 24 | 6 | 2 |
| 553 | 20261679 | 269 | 1688 | 21 | 7 | 3 |
| 554 | 20261680 | 200 | 1208 | 18 | 5 | 2 |
| 555 | 20261681 | 255 | 1222 | 21 | 7 | 3 |
| 556 | 20261682 | 215 | 793 | 8 | 8 | 4 |
| 557 | 20261683 | 266 | 1541 | 29 | 3 | 4 |
| 558 | 20261684 | 190 | 1058 | 14 | 1 | 0 |
| 559 | 20261685 | 176 | 770 | 12 | 5 | 3 |
| 560 | 20261686 | 240 | 1512 | 32 | 2 | 3 |
| 561 | 20261687 | 296 | 1660 | 26 | 5 | 2 |
| 562 | 20261688 | 283 | 1585 | 19 | 3 | 3 |
| 563 | 20261689 | 240 | 1331 | 33 | 3 | 2 |
| 564 | 20261690 | 171 | 636 | 11 | 5 | 2 |
| 565 | 20261691 | 277 | 1133 | 23 | 7 | 5 |
| 566 | 20261692 | 225 | 1437 | 44 | 4 | 2 |
| 567 | 20261693 | 319 | 1608 | 29 | 2 | 3 |
| 568 | 20261694 | 157 | 515 | 8 | 2 | 2 |
| 569 | 20261695 | 225 | 1069 | 19 | 3 | 2 |
| 570 | 20261696 | 306 | 1868 | 31 | 4 | 4 |
| 571 | 20261697 | 221 | 1210 | 19 | 2 | 2 |
| 572 | 20261698 | 247 | 1274 | 27 | 5 | 3 |
| 573 | 20261699 | 255 | 1142 | 22 | 2 | 4 |
| 574 | 20261700 | 186 | 756 | 16 | 4 | 1 |
| 575 | 20261701 | 277 | 1422 | 16 | 2 | 1 |
| 576 | 20261702 | 267 | 1410 | 44 | 3 | 3 |
| 577 | 20261703 | 151 | 756 | 9 | 2 | 0 |
| 578 | 20261704 | 251 | 1272 | 18 | 5 | 1 |
| 579 | 20261705 | 214 | 1090 | 26 | 4 | 3 |
| 580 | 20261706 | 257 | 1562 | 24 | 2 | 2 |
| 581 | 20261707 | 205 | 841 | 22 | 1 | 2 |
| 582 | 20261708 | 214 | 1008 | 20 | 2 | 1 |
| 583 | 20261709 | 206 | 918 | 22 | 1 | 2 |
| 584 | 20261710 | 282 | 1279 | 25 | 8 | 4 |
| 585 | 20261711 | 264 | 1580 | 27 | 5 | 2 |
| 586 | 20261712 | 431 | 2410 | 27 | 5 | 3 |
| 587 | 20261713 | 294 | 1329 | 46 | 2 | 4 |
| 588 | 20261714 | 255 | 1352 | 24 | 4 | 3 |
| 589 | 20261715 | 300 | 1287 | 25 | 6 | 3 |
| 590 | 20261716 | 242 | 1194 | 20 | 3 | 5 |
| 591 | 20261717 | 263 | 1388 | 27 | 3 | 1 |
| 592 | 20261718 | 232 | 1221 | 25 | 4 | 1 |
| 593 | 20261719 | 297 | 1703 | 28 | 2 | 2 |
| 594 | 20261720 | 199 | 1001 | 19 | 4 | 1 |
| 595 | 20261721 | 200 | 923 | 19 | 4 | 2 |
| 596 | 20261722 | 244 | 1150 | 21 | 1 | 2 |
| 597 | 20261723 | 217 | 1128 | 33 | 5 | 3 |
| 598 | 20261724 | 264 | 1516 | 37 | 5 | 3 |
| 599 | 20261725 | 217 | 922 | 17 | 7 | 1 |
| 600 | 20261726 | 245 | 1054 | 36 | 6 | 3 |
| 601 | 20261727 | 254 | 1321 | 29 | 10 | 2 |
| 602 | 20261728 | 291 | 1691 | 39 | 3 | 1 |
| 603 | 20261729 | 266 | 1501 | 23 | 1 | 3 |
| 604 | 20261730 | 271 | 1191 | 24 | 1 | 3 |
| 605 | 20261731 | 320 | 1603 | 35 | 6 | 5 |
| 606 | 20261732 | 377 | 1858 | 39 | 4 | 4 |
| 607 | 20261733 | 217 | 939 | 25 | 4 | 3 |
| 608 | 20261734 | 253 | 1025 | 22 | 3 | 4 |
| 609 | 20261735 | 225 | 1192 | 17 | 9 | 3 |
| 610 | 20261736 | 181 | 597 | 9 | 1 | 2 |
| 611 | 20261737 | 343 | 1723 | 30 | 3 | 4 |
| 612 | 20261738 | 228 | 1324 | 24 | 4 | 2 |
| 613 | 20261739 | 278 | 1623 | 20 | 1 | 1 |
| 614 | 20261740 | 304 | 1884 | 37 | 1 | 5 |
| 615 | 20261741 | 276 | 1474 | 23 | 15 | 2 |
| 616 | 20261742 | 286 | 1518 | 35 | 8 | 2 |
| 617 | 20261743 | 300 | 1402 | 39 | 8 | 4 |
| 618 | 20261744 | 197 | 843 | 23 | 3 | 6 |
| 619 | 20261745 | 185 | 973 | 9 | 3 | 2 |
| 620 | 20261746 | 191 | 783 | 11 | 2 | 1 |
| 621 | 20261747 | 230 | 975 | 15 | 4 | 2 |
| 622 | 20261748 | 209 | 926 | 23 | 5 | 3 |
| 623 | 20261749 | 240 | 1518 | 20 | 4 | 1 |
| 624 | 20261750 | 258 | 1058 | 15 | 1 | 2 |
| 625 | 20261751 | 320 | 1673 | 32 | 4 | 2 |
| 626 | 20261752 | 289 | 1563 | 36 | 3 | 3 |
| 627 | 20261753 | 271 | 1280 | 18 | 4 | 4 |
| 628 | 20261754 | 285 | 1625 | 28 | 2 | 5 |
| 629 | 20261755 | 200 | 1088 | 22 | 0 | 2 |
| 630 | 20261756 | 255 | 1213 | 31 | 1 | 2 |
| 631 | 20261757 | 333 | 1857 | 32 | 7 | 4 |
| 632 | 20261758 | 278 | 1519 | 27 | 7 | 3 |
| 633 | 20261759 | 274 | 1200 | 17 | 5 | 5 |
| 634 | 20261760 | 220 | 912 | 13 | 3 | 5 |
| 635 | 20261761 | 258 | 1231 | 30 | 1 | 3 |
| 636 | 20261762 | 234 | 856 | 13 | 3 | 2 |
| 637 | 20261763 | 169 | 601 | 15 | 2 | 1 |
| 638 | 20261764 | 311 | 1717 | 35 | 6 | 4 |
| 639 | 20261765 | 235 | 1214 | 21 | 6 | 1 |
| 640 | 20261766 | 358 | 1863 | 21 | 1 | 2 |
| 641 | 20261767 | 323 | 1984 | 28 | 2 | 3 |
| 642 | 20261768 | 270 | 1332 | 25 | 7 | 1 |
| 643 | 20261769 | 230 | 1180 | 22 | 2 | 2 |
| 644 | 20261770 | 237 | 1072 | 20 | 11 | 4 |
| 645 | 20261771 | 178 | 724 | 19 | 3 | 3 |
| 646 | 20261772 | 281 | 1834 | 23 | 5 | 2 |
| 647 | 20261773 | 194 | 1038 | 23 | 2 | 3 |
| 648 | 20261774 | 279 | 1651 | 35 | 1 | 2 |
| 649 | 20261775 | 241 | 1191 | 15 | 2 | 4 |
| 650 | 20261776 | 286 | 1613 | 37 | 3 | 3 |
| 651 | 20261777 | 236 | 804 | 15 | 2 | 2 |
| 652 | 20261778 | 208 | 1007 | 28 | 4 | 3 |
| 653 | 20261779 | 256 | 1279 | 27 | 1 | 1 |
| 654 | 20261780 | 225 | 1062 | 17 | 3 | 5 |
| 655 | 20261781 | 266 | 1421 | 35 | 5 | 4 |
| 656 | 20261782 | 187 | 898 | 17 | 3 | 2 |
| 657 | 20261783 | 227 | 1133 | 27 | 9 | 1 |
| 658 | 20261784 | 198 | 904 | 22 | 6 | 2 |
| 659 | 20261785 | 235 | 1055 | 21 | 2 | 3 |
| 660 | 20261786 | 197 | 996 | 14 | 3 | 1 |
| 661 | 20261787 | 201 | 792 | 18 | 1 | 2 |
| 662 | 20261788 | 191 | 950 | 19 | 3 | 2 |
| 663 | 20261789 | 210 | 870 | 15 | 4 | 1 |
| 664 | 20261790 | 174 | 777 | 20 | 2 | 2 |
| 665 | 20261791 | 215 | 1117 | 13 | 5 | 2 |
| 666 | 20261792 | 226 | 1150 | 13 | 4 | 2 |
| 667 | 20261793 | 248 | 1121 | 25 | 4 | 1 |
| 668 | 20261794 | 241 | 951 | 22 | 3 | 4 |
| 669 | 20261795 | 263 | 1320 | 15 | 7 | 4 |
| 670 | 20261796 | 264 | 1366 | 27 | 4 | 3 |
| 671 | 20261797 | 345 | 2036 | 33 | 3 | 3 |
| 672 | 20261798 | 189 | 748 | 20 | 5 | 2 |
| 673 | 20261799 | 199 | 653 | 19 | 4 | 4 |
| 674 | 20261800 | 272 | 1433 | 30 | 7 | 4 |
| 675 | 20261801 | 173 | 679 | 11 | 4 | 3 |
| 676 | 20261802 | 239 | 1216 | 32 | 5 | 2 |
| 677 | 20261803 | 235 | 1315 | 19 | 3 | 3 |
| 678 | 20261804 | 241 | 1243 | 19 | 8 | 4 |
| 679 | 20261805 | 308 | 1372 | 17 | 3 | 6 |
| 680 | 20261806 | 228 | 1245 | 17 | 0 | 2 |
| 681 | 20261807 | 272 | 1513 | 26 | 2 | 2 |
| 682 | 20261808 | 272 | 1689 | 35 | 10 | 5 |
| 683 | 20261809 | 289 | 1689 | 24 | 7 | 3 |
| 684 | 20261810 | 239 | 1152 | 16 | 4 | 3 |
| 685 | 20261811 | 261 | 1202 | 31 | 0 | 2 |
| 686 | 20261812 | 216 | 940 | 26 | 5 | 3 |
| 687 | 20261813 | 210 | 994 | 11 | 2 | 2 |
| 688 | 20261814 | 240 | 1316 | 10 | 5 | 1 |
| 689 | 20261815 | 190 | 857 | 23 | 2 | 1 |
| 690 | 20261816 | 251 | 1074 | 23 | 6 | 5 |
| 691 | 20261817 | 283 | 1586 | 35 | 6 | 2 |
| 692 | 20261818 | 322 | 2020 | 28 | 3 | 2 |
| 693 | 20261819 | 271 | 1130 | 32 | 4 | 4 |
| 694 | 20261820 | 276 | 1393 | 39 | 3 | 3 |
| 695 | 20261821 | 179 | 655 | 15 | 5 | 2 |
| 696 | 20261822 | 212 | 1173 | 22 | 3 | 3 |
| 697 | 20261823 | 188 | 824 | 18 | 6 | 2 |
| 698 | 20261824 | 199 | 680 | 10 | 6 | 3 |
| 699 | 20261825 | 261 | 1457 | 25 | 3 | 2 |
| 700 | 20261826 | 182 | 907 | 18 | 3 | 4 |
| 701 | 20261827 | 248 | 1106 | 29 | 1 | 4 |
| 702 | 20261828 | 282 | 1483 | 42 | 4 | 3 |
| 703 | 20261829 | 339 | 1528 | 18 | 4 | 4 |
| 704 | 20261830 | 271 | 1496 | 18 | 0 | 3 |
| 705 | 20261831 | 240 | 1336 | 27 | 5 | 2 |
| 706 | 20261832 | 324 | 1560 | 26 | 2 | 4 |
| 707 | 20261833 | 180 | 667 | 5 | 2 | 2 |
| 708 | 20261834 | 238 | 1140 | 23 | 4 | 1 |
| 709 | 20261835 | 211 | 862 | 10 | 6 | 0 |
| 710 | 20261836 | 375 | 1918 | 55 | 1 | 4 |
| 711 | 20261837 | 252 | 1457 | 31 | 3 | 3 |
| 712 | 20261838 | 184 | 690 | 12 | 3 | 1 |
| 713 | 20261839 | 199 | 921 | 17 | 3 | 2 |
| 714 | 20261840 | 216 | 956 | 7 | 0 | 3 |
| 715 | 20261841 | 190 | 887 | 13 | 3 | 1 |
| 716 | 20261842 | 298 | 1193 | 21 | 4 | 1 |
| 717 | 20261843 | 228 | 1065 | 12 | 5 | 3 |
| 718 | 20261844 | 331 | 1670 | 25 | 10 | 3 |
| 719 | 20261845 | 203 | 1068 | 27 | 3 | 1 |
| 720 | 20261846 | 240 | 1095 | 27 | 3 | 2 |
| 721 | 20261847 | 206 | 1042 | 22 | 2 | 2 |
| 722 | 20261848 | 265 | 946 | 21 | 2 | 4 |
| 723 | 20261849 | 189 | 959 | 15 | 6 | 2 |
| 724 | 20261850 | 244 | 1245 | 25 | 4 | 4 |
| 725 | 20261851 | 277 | 1665 | 35 | 4 | 5 |
| 726 | 20261852 | 224 | 1305 | 19 | 4 | 4 |
| 727 | 20261853 | 179 | 949 | 16 | 5 | 2 |
| 728 | 20261854 | 258 | 1541 | 34 | 5 | 3 |
| 729 | 20261855 | 241 | 1669 | 24 | 1 | 1 |
| 730 | 20261856 | 300 | 1424 | 39 | 8 | 3 |
| 731 | 20261857 | 253 | 1292 | 21 | 4 | 2 |
| 732 | 20261858 | 212 | 1121 | 12 | 0 | 3 |
| 733 | 20261859 | 211 | 1003 | 18 | 0 | 2 |
| 734 | 20261860 | 219 | 1193 | 14 | 6 | 3 |
| 735 | 20261861 | 232 | 1178 | 16 | 1 | 3 |
| 736 | 20261862 | 396 | 2619 | 39 | 6 | 2 |
| 737 | 20261863 | 257 | 1347 | 25 | 2 | 3 |
| 738 | 20261864 | 221 | 1073 | 22 | 2 | 2 |
| 739 | 20261865 | 278 | 1359 | 32 | 3 | 3 |
| 740 | 20261866 | 190 | 994 | 9 | 4 | 2 |
| 741 | 20261867 | 239 | 1508 | 29 | 3 | 3 |
| 742 | 20261868 | 314 | 1965 | 25 | 6 | 2 |
| 743 | 20261869 | 198 | 886 | 15 | 3 | 2 |
| 744 | 20261870 | 282 | 1230 | 24 | 1 | 3 |
| 745 | 20261871 | 241 | 1513 | 12 | 2 | 1 |
| 746 | 20261872 | 311 | 1578 | 29 | 6 | 3 |
| 747 | 20261873 | 243 | 1329 | 29 | 4 | 5 |
| 748 | 20261874 | 272 | 1542 | 22 | 4 | 3 |
| 749 | 20261875 | 197 | 1007 | 16 | 2 | 0 |
| 750 | 20261876 | 220 | 1140 | 19 | 3 | 1 |
| 751 | 20261877 | 204 | 1051 | 19 | 2 | 4 |
| 752 | 20261878 | 249 | 1445 | 23 | 9 | 2 |
| 753 | 20261879 | 241 | 1077 | 28 | 4 | 3 |
| 754 | 20261880 | 499 | 3105 | 27 | 6 | 2 |
| 755 | 20261881 | 191 | 915 | 13 | 4 | 3 |
| 756 | 20261882 | 217 | 1296 | 24 | 0 | 3 |
| 757 | 20261883 | 265 | 1282 | 30 | 3 | 2 |
| 758 | 20261884 | 241 | 1312 | 24 | 10 | 2 |
| 759 | 20261885 | 221 | 1114 | 6 | 6 | 1 |
| 760 | 20261886 | 205 | 1204 | 29 | 1 | 3 |
| 761 | 20261887 | 253 | 1172 | 27 | 3 | 3 |
| 762 | 20261888 | 249 | 1326 | 26 | 1 | 2 |
| 763 | 20261889 | 218 | 1320 | 24 | 2 | 2 |
| 764 | 20261890 | 297 | 1362 | 20 | 5 | 2 |
| 765 | 20261891 | 234 | 1120 | 30 | 8 | 3 |
| 766 | 20261892 | 206 | 881 | 17 | 3 | 1 |
| 767 | 20261893 | 267 | 1237 | 34 | 3 | 4 |
| 768 | 20261894 | 207 | 961 | 12 | 3 | 3 |
| 769 | 20261895 | 215 | 1017 | 18 | 2 | 2 |
| 770 | 20261896 | 231 | 1237 | 21 | 2 | 1 |
| 771 | 20261897 | 307 | 1397 | 38 | 1 | 4 |
| 772 | 20261898 | 254 | 1415 | 24 | 5 | 0 |
| 773 | 20261899 | 237 | 1254 | 21 | 0 | 2 |
| 774 | 20261900 | 394 | 1912 | 33 | 4 | 3 |
| 775 | 20261901 | 198 | 794 | 15 | 4 | 2 |
| 776 | 20261902 | 248 | 1263 | 11 | 13 | 2 |
| 777 | 20261903 | 263 | 1323 | 27 | 7 | 4 |
| 778 | 20261904 | 235 | 1370 | 28 | 3 | 2 |
| 779 | 20261905 | 290 | 1694 | 33 | 3 | 2 |
| 780 | 20261906 | 202 | 848 | 17 | 7 | 4 |
| 781 | 20261907 | 310 | 1389 | 31 | 10 | 5 |
| 782 | 20261908 | 255 | 1515 | 31 | 10 | 3 |
| 783 | 20261909 | 219 | 1012 | 15 | 3 | 2 |
| 784 | 20261910 | 354 | 1923 | 49 | 3 | 6 |
| 785 | 20261911 | 238 | 1248 | 18 | 2 | 3 |
| 786 | 20261912 | 235 | 919 | 23 | 2 | 2 |
| 787 | 20261913 | 209 | 864 | 15 | 10 | 2 |
| 788 | 20261914 | 310 | 1569 | 27 | 4 | 2 |
| 789 | 20261915 | 323 | 1870 | 36 | 5 | 3 |
| 790 | 20261916 | 205 | 965 | 20 | 4 | 2 |
| 791 | 20261917 | 263 | 1387 | 41 | 3 | 3 |
| 792 | 20261918 | 220 | 919 | 16 | 7 | 1 |
| 793 | 20261919 | 167 | 771 | 9 | 3 | 1 |
| 794 | 20261920 | 291 | 1792 | 30 | 0 | 4 |
| 795 | 20261921 | 260 | 1208 | 21 | 8 | 2 |
| 796 | 20261922 | 225 | 1138 | 24 | 3 | 3 |
| 797 | 20261923 | 298 | 1436 | 33 | 3 | 5 |
| 798 | 20261924 | 233 | 1457 | 39 | 4 | 4 |
| 799 | 20261925 | 246 | 975 | 14 | 2 | 3 |
| 800 | 20261926 | 238 | 1248 | 17 | 2 | 2 |
| 801 | 20261927 | 264 | 1135 | 28 | 5 | 3 |
| 802 | 20261928 | 269 | 1142 | 22 | 3 | 4 |
| 803 | 20261929 | 283 | 1300 | 29 | 4 | 3 |
| 804 | 20261930 | 235 | 1209 | 16 | 2 | 4 |
| 805 | 20261931 | 247 | 1155 | 15 | 5 | 3 |
| 806 | 20261932 | 247 | 1211 | 15 | 1 | 3 |
| 807 | 20261933 | 255 | 1557 | 29 | 7 | 2 |
| 808 | 20261934 | 288 | 1565 | 18 | 2 | 2 |
| 809 | 20261935 | 223 | 1238 | 21 | 7 | 2 |
| 810 | 20261936 | 291 | 1530 | 40 | 3 | 3 |
| 811 | 20261937 | 217 | 1274 | 25 | 4 | 3 |
| 812 | 20261938 | 343 | 2085 | 47 | 7 | 3 |
| 813 | 20261939 | 215 | 745 | 21 | 2 | 3 |
| 814 | 20261940 | 200 | 1021 | 16 | 0 | 2 |
| 815 | 20261941 | 197 | 955 | 27 | 5 | 1 |
| 816 | 20261942 | 291 | 1673 | 30 | 4 | 2 |
| 817 | 20261943 | 204 | 970 | 35 | 2 | 3 |
| 818 | 20261944 | 233 | 1366 | 18 | 2 | 3 |
| 819 | 20261945 | 250 | 1490 | 14 | 2 | 4 |
| 820 | 20261946 | 271 | 1196 | 8 | 7 | 2 |
| 821 | 20261947 | 220 | 852 | 23 | 4 | 4 |
| 822 | 20261948 | 207 | 991 | 16 | 6 | 1 |
| 823 | 20261949 | 202 | 1139 | 18 | 8 | 2 |
| 824 | 20261950 | 235 | 1205 | 14 | 3 | 2 |
| 825 | 20261951 | 236 | 1003 | 15 | 4 | 2 |
| 826 | 20261952 | 298 | 1304 | 20 | 0 | 2 |
| 827 | 20261953 | 282 | 1149 | 34 | 11 | 4 |
| 828 | 20261954 | 235 | 1247 | 27 | 1 | 3 |
| 829 | 20261955 | 249 | 1554 | 26 | 6 | 3 |
| 830 | 20261956 | 193 | 923 | 23 | 7 | 4 |
| 831 | 20261957 | 281 | 1509 | 26 | 0 | 5 |
| 832 | 20261958 | 237 | 1289 | 22 | 5 | 2 |
| 833 | 20261959 | 174 | 796 | 16 | 3 | 2 |
| 834 | 20261960 | 236 | 1157 | 18 | 2 | 1 |
| 835 | 20261961 | 235 | 797 | 19 | 3 | 4 |
| 836 | 20261962 | 283 | 1262 | 32 | 2 | 3 |
| 837 | 20261963 | 292 | 1418 | 18 | 4 | 4 |
| 838 | 20261964 | 340 | 1750 | 40 | 6 | 4 |
| 839 | 20261965 | 302 | 1511 | 32 | 6 | 3 |
| 840 | 20261966 | 244 | 1159 | 30 | 4 | 4 |
| 841 | 20261967 | 168 | 548 | 12 | 2 | 1 |
| 842 | 20261968 | 248 | 1476 | 22 | 5 | 3 |
| 843 | 20261969 | 220 | 1141 | 25 | 1 | 2 |
| 844 | 20261970 | 195 | 911 | 16 | 1 | 3 |
| 845 | 20261971 | 214 | 1018 | 12 | 2 | 2 |
| 846 | 20261972 | 323 | 2226 | 21 | 9 | 2 |
| 847 | 20261973 | 302 | 1494 | 32 | 2 | 4 |
| 848 | 20261974 | 396 | 2259 | 42 | 2 | 4 |
| 849 | 20261975 | 246 | 1042 | 12 | 5 | 2 |
| 850 | 20261976 | 199 | 818 | 17 | 4 | 3 |
| 851 | 20261977 | 279 | 1690 | 33 | 6 | 2 |
| 852 | 20261978 | 266 | 1503 | 26 | 9 | 1 |
| 853 | 20261979 | 250 | 861 | 16 | 1 | 2 |
| 854 | 20261980 | 225 | 1116 | 15 | 7 | 1 |
| 855 | 20261981 | 212 | 1012 | 21 | 5 | 2 |
| 856 | 20261982 | 185 | 787 | 12 | 3 | 2 |
| 857 | 20261983 | 216 | 1031 | 17 | 7 | 3 |
| 858 | 20261984 | 204 | 840 | 22 | 0 | 3 |
| 859 | 20261985 | 228 | 1271 | 14 | 7 | 2 |
| 860 | 20261986 | 191 | 981 | 7 | 6 | 3 |
| 861 | 20261987 | 292 | 1733 | 38 | 5 | 4 |
| 862 | 20261988 | 373 | 2225 | 34 | 4 | 3 |
| 863 | 20261989 | 223 | 942 | 21 | 2 | 1 |
| 864 | 20261990 | 217 | 996 | 16 | 1 | 3 |
| 865 | 20261991 | 271 | 1126 | 34 | 4 | 2 |
| 866 | 20261992 | 500 | 2734 | 65 | 0 | 5 |
| 867 | 20261993 | 282 | 1437 | 28 | 6 | 3 |
| 868 | 20261994 | 261 | 1346 | 8 | 3 | 4 |
| 869 | 20261995 | 191 | 952 | 19 | 1 | 2 |
| 870 | 20261996 | 309 | 1942 | 26 | 11 | 1 |
| 871 | 20261997 | 240 | 1295 | 23 | 15 | 3 |
| 872 | 20261998 | 276 | 1419 | 34 | 4 | 3 |
| 873 | 20261999 | 204 | 877 | 11 | 3 | 2 |
| 874 | 20262000 | 296 | 1688 | 34 | 4 | 2 |
| 875 | 20262001 | 302 | 1508 | 41 | 4 | 3 |
| 876 | 20262002 | 241 | 1025 | 12 | 8 | 3 |
| 877 | 20262003 | 268 | 1482 | 18 | 1 | 3 |
| 878 | 20262004 | 285 | 1253 | 19 | 2 | 3 |
| 879 | 20262005 | 204 | 990 | 19 | 5 | 3 |
| 880 | 20262006 | 180 | 685 | 6 | 2 | 3 |
| 881 | 20262007 | 233 | 1165 | 30 | 7 | 2 |
| 882 | 20262008 | 188 | 1068 | 20 | 5 | 4 |
| 883 | 20262009 | 240 | 1128 | 29 | 6 | 3 |
| 884 | 20262010 | 291 | 1331 | 14 | 14 | 3 |
| 885 | 20262011 | 335 | 1456 | 29 | 4 | 2 |
| 886 | 20262012 | 243 | 966 | 23 | 5 | 3 |
| 887 | 20262013 | 299 | 1338 | 23 | 8 | 4 |
| 888 | 20262014 | 210 | 955 | 30 | 3 | 3 |
| 889 | 20262015 | 275 | 1341 | 34 | 7 | 3 |
| 890 | 20262016 | 275 | 1405 | 27 | 1 | 4 |
| 891 | 20262017 | 204 | 1037 | 16 | 2 | 2 |
| 892 | 20262018 | 234 | 894 | 16 | 0 | 2 |
| 893 | 20262019 | 195 | 909 | 15 | 1 | 1 |
| 894 | 20262020 | 267 | 1294 | 13 | 7 | 4 |
| 895 | 20262021 | 269 | 1362 | 29 | 7 | 3 |
| 896 | 20262022 | 215 | 807 | 12 | 5 | 2 |
| 897 | 20262023 | 308 | 1701 | 36 | 3 | 3 |
| 898 | 20262024 | 213 | 888 | 19 | 4 | 2 |
| 899 | 20262025 | 227 | 1142 | 15 | 5 | 2 |
| 900 | 20262026 | 184 | 1000 | 18 | 1 | 2 |
| 901 | 20262027 | 415 | 2512 | 32 | 13 | 4 |
| 902 | 20262028 | 215 | 837 | 17 | 3 | 2 |
| 903 | 20262029 | 234 | 1432 | 15 | 3 | 2 |
| 904 | 20262030 | 411 | 2284 | 38 | 9 | 3 |
| 905 | 20262031 | 278 | 1705 | 36 | 1 | 4 |
| 906 | 20262032 | 253 | 1058 | 14 | 7 | 3 |
| 907 | 20262033 | 241 | 1100 | 30 | 4 | 2 |
| 908 | 20262034 | 252 | 1128 | 20 | 5 | 5 |
| 909 | 20262035 | 331 | 1918 | 33 | 1 | 2 |
| 910 | 20262036 | 243 | 1217 | 23 | 2 | 4 |
| 911 | 20262037 | 283 | 1788 | 26 | 2 | 1 |
| 912 | 20262038 | 236 | 1390 | 37 | 1 | 1 |
| 913 | 20262039 | 234 | 1423 | 19 | 3 | 1 |
| 914 | 20262040 | 229 | 1272 | 13 | 7 | 2 |
| 915 | 20262041 | 341 | 1500 | 30 | 6 | 5 |
| 916 | 20262042 | 235 | 1222 | 29 | 4 | 1 |
| 917 | 20262043 | 256 | 1226 | 21 | 1 | 3 |
| 918 | 20262044 | 256 | 1446 | 23 | 6 | 3 |
| 919 | 20262045 | 231 | 1383 | 30 | 5 | 4 |
| 920 | 20262046 | 384 | 2112 | 40 | 1 | 4 |
| 921 | 20262047 | 238 | 994 | 12 | 2 | 2 |
| 922 | 20262048 | 199 | 744 | 19 | 1 | 3 |
| 923 | 20262049 | 247 | 1127 | 26 | 8 | 4 |
| 924 | 20262050 | 235 | 1372 | 34 | 5 | 3 |
| 925 | 20262051 | 293 | 1238 | 34 | 4 | 3 |
| 926 | 20262052 | 232 | 1376 | 33 | 2 | 4 |
| 927 | 20262053 | 190 | 1125 | 8 | 2 | 3 |
| 928 | 20262054 | 187 | 798 | 7 | 5 | 2 |
| 929 | 20262055 | 178 | 804 | 10 | 2 | 2 |
| 930 | 20262056 | 191 | 845 | 19 | 2 | 2 |
| 931 | 20262057 | 239 | 810 | 19 | 2 | 4 |
| 932 | 20262058 | 251 | 1045 | 30 | 9 | 5 |
| 933 | 20262059 | 193 | 780 | 13 | 3 | 1 |
| 934 | 20262060 | 297 | 1753 | 30 | 6 | 1 |
| 935 | 20262061 | 291 | 1285 | 16 | 2 | 6 |
| 936 | 20262062 | 228 | 1227 | 24 | 5 | 4 |
| 937 | 20262063 | 268 | 1116 | 30 | 3 | 4 |
| 938 | 20262064 | 293 | 1811 | 37 | 0 | 3 |
| 939 | 20262065 | 164 | 617 | 8 | 2 | 1 |
| 940 | 20262066 | 236 | 1165 | 21 | 4 | 2 |
| 941 | 20262067 | 238 | 1682 | 23 | 1 | 3 |
| 942 | 20262068 | 244 | 1375 | 26 | 9 | 3 |
| 943 | 20262069 | 273 | 1494 | 21 | 1 | 3 |
| 944 | 20262070 | 357 | 2046 | 35 | 3 | 3 |
| 945 | 20262071 | 261 | 1689 | 23 | 2 | 2 |
| 946 | 20262072 | 244 | 1173 | 22 | 4 | 3 |
| 947 | 20262073 | 311 | 1472 | 17 | 3 | 4 |
| 948 | 20262074 | 215 | 1231 | 20 | 1 | 2 |
| 949 | 20262075 | 181 | 898 | 15 | 4 | 2 |
| 950 | 20262076 | 262 | 1401 | 29 | 5 | 2 |
| 951 | 20262077 | 288 | 1523 | 32 | 7 | 2 |
| 952 | 20262078 | 214 | 790 | 16 | 6 | 3 |
| 953 | 20262079 | 214 | 1078 | 20 | 6 | 4 |
| 954 | 20262080 | 232 | 1025 | 34 | 5 | 3 |
| 955 | 20262081 | 231 | 1366 | 21 | 4 | 4 |
| 956 | 20262082 | 234 | 1024 | 17 | 9 | 2 |
| 957 | 20262083 | 173 | 659 | 17 | 8 | 2 |
| 958 | 20262084 | 273 | 1674 | 26 | 5 | 2 |
| 959 | 20262085 | 205 | 1190 | 17 | 7 | 3 |
| 960 | 20262086 | 226 | 1034 | 13 | 10 | 4 |
| 961 | 20262087 | 299 | 1445 | 34 | 2 | 4 |
| 962 | 20262088 | 206 | 1197 | 24 | 6 | 3 |
| 963 | 20262089 | 246 | 1053 | 24 | 5 | 2 |
| 964 | 20262090 | 216 | 1187 | 14 | 1 | 2 |
| 965 | 20262091 | 231 | 1085 | 23 | 2 | 4 |
| 966 | 20262092 | 218 | 1228 | 21 | 2 | 2 |
| 967 | 20262093 | 274 | 1495 | 22 | 1 | 4 |
| 968 | 20262094 | 284 | 1590 | 37 | 8 | 2 |
| 969 | 20262095 | 298 | 1467 | 40 | 4 | 4 |
| 970 | 20262096 | 322 | 1345 | 31 | 1 | 2 |
| 971 | 20262097 | 259 | 1123 | 14 | 1 | 3 |
| 972 | 20262098 | 286 | 1824 | 39 | 3 | 4 |
| 973 | 20262099 | 197 | 754 | 11 | 1 | 1 |
| 974 | 20262100 | 217 | 1075 | 19 | 7 | 2 |
| 975 | 20262101 | 283 | 1481 | 22 | 10 | 4 |
| 976 | 20262102 | 214 | 1149 | 18 | 7 | 0 |
| 977 | 20262103 | 217 | 1075 | 10 | 5 | 3 |
| 978 | 20262104 | 218 | 1261 | 25 | 5 | 1 |
| 979 | 20262105 | 207 | 1109 | 19 | 2 | 3 |
| 980 | 20262106 | 238 | 1473 | 26 | 5 | 3 |
| 981 | 20262107 | 225 | 1118 | 23 | 6 | 3 |
| 982 | 20262108 | 206 | 942 | 21 | 1 | 3 |
| 983 | 20262109 | 297 | 1505 | 18 | 8 | 2 |
| 984 | 20262110 | 309 | 1643 | 36 | 12 | 2 |
| 985 | 20262111 | 253 | 1494 | 20 | 1 | 3 |
| 986 | 20262112 | 152 | 638 | 8 | 7 | 3 |
| 987 | 20262113 | 232 | 1413 | 25 | 3 | 1 |
| 988 | 20262114 | 310 | 1948 | 25 | 4 | 4 |
| 989 | 20262115 | 302 | 1424 | 31 | 2 | 3 |
| 990 | 20262116 | 260 | 1342 | 20 | 7 | 2 |
| 991 | 20262117 | 314 | 1704 | 33 | 2 | 4 |
| 992 | 20262118 | 325 | 2011 | 50 | 4 | 0 |
| 993 | 20262119 | 327 | 1178 | 15 | 5 | 2 |
| 994 | 20262120 | 206 | 977 | 19 | 4 | 2 |
| 995 | 20262121 | 214 | 1021 | 19 | 2 | 2 |
| 996 | 20262122 | 189 | 936 | 23 | 4 | 1 |
| 997 | 20262123 | 223 | 1208 | 17 | 6 | 3 |
| 998 | 20262124 | 223 | 959 | 17 | 6 | 2 |
| 999 | 20262125 | 340 | 1913 | 33 | 4 | 4 |

## Throughput

- Turns simulated: 250353 (first play of each game)
- Aggregate: 306.1 turns/s over 817.8s wall
- Per game thread: 51.7 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

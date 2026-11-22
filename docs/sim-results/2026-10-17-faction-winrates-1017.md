# Simulation: faction-winrates-1017 (2026-10-17)

512 games, first seed 9170001, brain `scripted`, speed `Standard`, turn limit off, safety cap 1200, 64 threads, started 2026-10-17T20:20:27Z. Wall time 27.0s.

Source checkout at run start: `dcfc8becbf0becc86d95851d8197fa24f5453b6f`; dirty: yes. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i (wrapping); the strategy tournament schedule is specified below. All games use a deterministic shuffle of all eight factions (one seat each), with any Proxy rotation described below.

Command argv: `["./target/sim/legatum-sim","--games","512","--seed","9170001","--players","8","--difficulty","hard","--label","faction-winrates-1017","--out","faction-winrates.json"]`

## Overview

| Metric | Value |
| --- | --- |
| Games (non-panic) | 512 of 512 |
| Game length, turns (mean / median / min-max) | 261.6 / 256 / 151-718 |
| Win reason mix | VP hold 100.0% |
| VP hold length | 10 turn ends (the reaching turn counts) |
| Holds started / broken / completed / unfinished | 1358 / 746 / 516 / 96 |
| Hold break rate | 54.9% of started; 59.1% of resolved (broken / (broken + completed)) |
| Hold wins | 512 |
| First reach to win, turns (mean / median / min-max) | 16.5 / 9 / 9-303 |
| Final hold start to win, turns | 9.0 / 9 / 9-9 |
| Comebacks (winner broke an earlier hold of its own) | 158 |
| Wins by a team another team beat to its target | 114 |

Game length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):

| Ended by | Games | Length mean / median / min-max |
| --- | --- | --- |
| VP hold | 512 | 261.6 / 256 / 151-718 |

Winner share by faction (wins of that faction's seats / all winning seats):

| Faction | Wins | Share of wins |
| --- | --- | --- |
| Switzerland | 57 | 11.1% |
| Japan | 15 | 2.9% |
| Byzantium | 101 | 19.7% |
| Germany | 59 | 11.5% |
| Celts | 56 | 10.9% |
| Mongols | 39 | 7.6% |
| Sparta | 90 | 17.6% |
| Athens | 95 | 18.6% |

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP hold | 512 | 100.0% |
| Conquest | 0 | 0.0% |
| Turn limit | 0 | 0.0% |
| Draw | 0 | 0.0% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 512 | 57 | 11.1% |
| Japan | 512 | 15 | 2.9% |
| Byzantium | 512 | 101 | 19.7% |
| Germany | 512 | 59 | 11.5% |
| Celts | 512 | 56 | 10.9% |
| Mongols | 512 | 39 | 7.6% |
| Sparta | 512 | 90 | 17.6% |
| Athens | 512 | 95 | 18.6% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Hard | 4096 | 512 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Hard | 512 | 57 | 11.1% |
| Japan | Hard | 512 | 15 | 2.9% |
| Byzantium | Hard | 512 | 101 | 19.7% |
| Germany | Hard | 512 | 59 | 11.5% |
| Celts | Hard | 512 | 56 | 10.9% |
| Mongols | Hard | 512 | 39 | 7.6% |
| Sparta | Hard | 512 | 90 | 17.6% |
| Athens | Hard | 512 | 95 | 18.6% |

## Faction goal progress

Per seat: how often the faction's permanent goal was reached before the game ended, and the turn its VP was first awarded (over the seats that reached it). Seats are all initial empires of non-panicked games with telemetry, including eliminated empires and cap stops.

| Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Switzerland | 512 | 280 | 54.7% | 75 | 113 | 184 | 254 | 342 |
| Japan | 512 | 202 | 39.5% | 154 | 194 | 201 | 231 | 305 |
| Byzantium | 512 | 333 | 65.0% | 66 | 103 | 149 | 226 | 304 |
| Germany | 512 | 142 | 27.7% | 64 | 137 | 192 | 264 | 394 |
| Celts | 512 | 473 | 92.4% | 165 | 210 | 210 | 210 | 231 |
| Mongols | 512 | 68 | 13.3% | 189 | 206 | 227 | 278 | 302 |
| Sparta | 512 | 10 | 2.0% | 157 | 200 | 239 | 295 | 306 |
| Athens | 512 | 300 | 58.6% | 142 | 201 | 255 | 255 | 255 |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 151 | 213 | 256 | 317 | 718 |
| Winning team VP | 10 | 10 | 10 | 11 | 12 |
| First Soldier completed (turn, per empire) | 3 | 6 | 29 | 157 | 406 |
| First Settler completed (turn, per empire) | 6 | 15 | 19 | 41 | 257 |
| Second city founded (turn, per empire) | 9 | 19 | 26 | 64 | 320 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 4096 observed empires in 512 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 3683 | 413 |
| First Settler completed | 3965 | 131 |
| Second city founded | 3791 | 305 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 362 |
| 1 | 100 |
| 2 | 470 |
| 3 | 330 |
| 4 | 461 |
| 5 | 367 |
| 6 | 416 |
| 7 | 345 |
| 8 | 379 |
| 9 | 254 |
| 10 | 451 |
| 11 | 148 |
| 12 | 13 |

## End state

- Average game length: 261.6 turns
- Average cities per empire: 4.44
- Average techs per empire: 30.13
- Average eliminations per game: 0.75

## Strategy diversity: observed behavior (classifier v2)

512 finished games; 4096 player seats; 512 winning seats. Excluded 0 unfinished/panicked games and 0 games with missing labels. Winner entropy: **0.5229** (normalized over 6 categories, including zero-win categories).

Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).

| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rush | 1244 | 114 | 9.2% | 30.4% | 22.3% |
| Wide | 1389 | 323 | 23.3% | 33.9% | 63.1% |
| Science | 0 | 0 | n/a | 0.0% | 0.0% |
| Turtle/Wonder | 933 | 71 | 7.6% | 22.8% | 13.9% |
| Economic | 28 | 0 | 0.0% | 0.7% | 0.0% |
| Generalist | 502 | 4 | 0.8% | 12.3% | 0.8% |

### Matchup matrix

Cell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.

| Row vs column | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rush | — | 24.5% / 6.0% (484) | n/a / n/a (0) | 44.2% / 8.7% (440) | 65.6% / 7.6% (24) | 76.3% / 4.5% (331) |
| Wide | 69.5% / 6.0% (484) | — | n/a / n/a (0) | 68.3% / 7.8% (454) | 74.1% / 12.5% (27) | 93.5% / 2.5% (347) |
| Science | n/a / n/a (0) | n/a / n/a (0) | — | n/a / n/a (0) | n/a / n/a (0) | n/a / n/a (0) |
| Turtle/Wonder | 47.1% / 8.7% (440) | 23.9% / 7.8% (454) | n/a / n/a (0) | — | 53.2% / 14.7% (26) | 82.8% / 6.7% (302) |
| Economic | 26.7% / 7.6% (24) | 13.4% / 12.5% (27) | n/a / n/a (0) | 32.1% / 14.7% (26) | — | 73.7% / 5.3% (19) |
| Generalist | 19.2% / 4.5% (331) | 4.1% / 2.5% (347) | n/a / n/a (0) | 10.6% / 6.7% (302) | 21.1% / 5.3% (19) | — |

### Faction wins by archetype

Cell = wins / seats; win rate in JSON.

| Faction | Rush | Wide | Science | Turtle/Wonder | Economic | Generalist |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 4 / 94 | 42 / 207 | 0 / 0 | 11 / 150 | 0 / 9 | 0 / 52 |
| Japan | 3 / 140 | 10 / 151 | 0 / 0 | 2 / 107 | 0 / 1 | 0 / 113 |
| Byzantium | 3 / 18 | 83 / 265 | 0 / 0 | 15 / 163 | 0 / 0 | 0 / 66 |
| Germany | 1 / 36 | 42 / 191 | 0 / 0 | 16 / 190 | 0 / 1 | 0 / 94 |
| Celts | 0 / 20 | 52 / 279 | 0 / 0 | 4 / 172 | 0 / 2 | 0 / 39 |
| Mongols | 23 / 403 | 15 / 61 | 0 / 0 | 0 / 2 | 0 / 15 | 1 / 31 |
| Sparta | 60 / 384 | 22 / 78 | 0 / 0 | 5 / 32 | 0 / 0 | 3 / 18 |
| Athens | 20 / 149 | 57 / 157 | 0 / 0 | 18 / 117 | 0 / 0 | 0 / 89 |

Guardrail: **Fail**. Entropy minimum 0.80; at least 20 games per strategy pair. Dominant by observed score: none. Without an observed counter (all pairs covered): none. Sampled but never won: Economic. Unobserved: Science. Under-sampled pairs: 6.

- insufficient evidence: 6 under-sampled matchups; 512 winning seats

- winner entropy below 0.80

## Battle and capture telemetry

512 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 22128/512 | 43.219 | 512/512 (100.0%) |
| Peace offers | 79108/512 | 154.508 | 512/512 (100.0%) |
| Peace agreements | 18431/512 | 35.998 | 512/512 (100.0%) |
| Battles | 549676/512 | 1073.586 | 512/512 (100.0%) |
| City captures | 7966/512 | 15.559 | 511/512 (99.8%) |
| Settler captures | 4034/512 | 7.879 | 506/512 (98.8%) |
| Eliminations | 383/512 | 0.748 | 283/512 (55.3%) |

## Terminal VP sources

512 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Live title: LargestEconomy | 512/4096 | 512/4096 | 297/512 |
| Live title: MostTerritory | 0/4096 | 0/4096 | 0/512 |
| Live title: StrongestArmy | 512/4096 | 512/4096 | 192/512 |
| Live title: MostAdvanced | 478/4096 | 478/4096 | 271/512 |
| Live title: MostPopulous | 0/4096 | 0/4096 | 0/512 |
| Live titles subtotal | 1502/4096 | 1502/4096 | 760/512 |
| Milestone: FourCities | 0/4096 | 0/4096 | 0/512 |
| Milestone: SixTechs | 0/4096 | 0/4096 | 0/512 |
| Milestone: TwentyPopulation | 0/4096 | 0/4096 | 0/512 |
| Milestone: RelayUplink | 6042/4096 | 6042/4096 | 1012/512 |
| Milestones subtotal | 6042/4096 | 6042/4096 | 1012/512 |
| Faction goals | 2826/4096 | 2826/4096 | 744/512 |
| Total VP | 22232/4096 | 22232/4096 | 5277/512 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 67/269 (24.9%) | 0 |
| Tied leaders (winner in set) | 129/243 (53.1%) | 0 |
| Any leader set | 196/512 (38.3%) | 0 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Era pacing

Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).

| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Classical | 4015/4096 | 6 | 21 | 28 | 31 | 84 | -1.6 (512) |
| Medieval | 3932/4096 | 45 | 60 | 64 | 69 | 256 | -1.2 (512) |
| Renaissance | 3911/4096 | 81 | 101 | 109 | 118 | 187 | -2.7 (512) |
| Industrial | 3861/4096 | 118 | 137 | 150 | 165 | 296 | -4.4 (512) |
| Modern | 3653/4096 | 148 | 176 | 191 | 213 | 286 | -4.6 (508) |
| Future | 2894/4096 | 184 | 208 | 229 | 256 | 331 | -5.0 (456) |

Research idle turns (techs left, no research set): 26, of which stalled (two or more in a row): 0.

## Metric definitions

One Event::Battle, including Scout encounters, ranged shots and meeting engagements; not a war, siege or submitted attack order.

City captures use the existing GameRecord.city_captures count of Event::CityCaptured (recaptures count again). Settler captures count Event::SettlerCaptured separately; neither is inferred from ownership or unit losses.

Count the existing GameRecord.eliminations entries, one per Event::PlayerEliminated; telemetry.elimination_count is their length. Never infer from final alive flags.

Final-state VP: each currently held title is 1 VP; each permanent milestone uses core milestone_vp; faction goals sum the core VpSource::FactionGoal entries (a held goal counts for its holder). Title transfers are not accumulated. Player sources reconcile to core vp_total; team sources sum each member once and reconcile to core team_vp. Player, team and winning-team populations are separate views, never added together. Eliminated members retain banked VP, as in core scoring.

After resolution AND scoring of turn 50, when GameState.turn is 51, before turn-51 orders. Rank all initial TEAMS by core team_vp (sum every member once), matching the probe's all-seat population. Equal VP preserves the entire leader set: no population or ID tie-break. Games ending on turn 50 are eligible; games ending earlier are excluded. Turn-75 replay comeback data is independent.

Per initial empire, record the resolved turn (turn 1 is the first turn) of its first Event::ProductionCompleted for Soldier and Settler, and its second Event::CityFounded. Starting units and captured units do not count as production; purchases count only when production actually completes. City captures do not count as founding, and losing a city does not reset the founding count. Null means never reached by the end of observed play. Distributions pool reached empires only, using the report's existing percentile convention; reached and never counts include eliminated empires and safety-cap stops. Cap stops are censored, not proof a milestone could never happen. Only first plays count; panics and legacy records without opening telemetry are excluded, not never observations.

A team wins by VP only after standing at its VP target at data::VP_HOLD_TURNS consecutive turn ends (10 Standard); the turn it reaches the target is the first held turn. Started = Event::VictoryHoldStarted; broken = Event::VictoryHoldBroken (dropped below the target or was eliminated); completed = the hold reached the full count (a VP win, or a draw when several teams complete with identical standings); unfinished = still running when the game ended otherwise (conquest, turn limit, safety cap). started = broken + completed + unfinished. Break rate = broken / started, and broken / (broken + completed) over resolved holds. First reach to win = win turn minus the first turn the winner ever stood at its target (includes broken holds); final hold start to win is always turns_needed - 1. A comeback is a hold win by a team that broke an earlier hold of its own; wins not first to reach are hold wins where another team first stood at its target strictly earlier. Only games with games[].telemetry.victory_holds count; panics and older reports are excluded, not zero. Outcome `vp` means a completed hold: a winner at or above its target at the turn limit without a completed hold is `turn_limit`.

Event totals and incidence use first plays with telemetry, including runner-stopped games; panics and legacy records without telemetry are excluded, not zeros. Terminal score populations use finished games only. Snowball cohorts use finished games with a turn-50 snapshot and at least one team; unique and tied leaders are separate. Draws stay in each eligible denominator and count as no leader win. Unfinished games have no eventual result and are excluded. Empty denominators are reported as 0/0 (no rate).

Definitions reuse [balance-probe-1](../analysis/balance-probe-1.md) and [the probe's definitions](../../tools/analysis/README.md); this report is a fresh simulation sample, not a rerun of those campaigns.

## Promotions

512 games with promotion telemetry. Promotions are UnitPromoted events by the level reached; final units are military units alive when the game ended.

| Level | Promotions / game | Final units / game |
| --- | --- | --- |
| 0 | 0.00 | 64.66 |
| 1 | 53.96 | 25.11 |
| 2 | 19.14 | 4.08 |
| 3 | 4.95 | 1.08 |
| 4 | 1.31 | 0.42 |
| 5 | 0.34 | 0.13 |

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 9170001 | 207 | 659 | 10 | 8 | 1 |
| 1 | 9170002 | 184 | 561 | 5 | 6 | 1 |
| 2 | 9170003 | 279 | 1409 | 16 | 10 | 1 |
| 3 | 9170004 | 270 | 1127 | 28 | 5 | 0 |
| 4 | 9170005 | 189 | 417 | 6 | 0 | 1 |
| 5 | 9170006 | 321 | 1848 | 7 | 16 | 1 |
| 6 | 9170007 | 272 | 1202 | 26 | 12 | 2 |
| 7 | 9170008 | 309 | 1566 | 10 | 16 | 0 |
| 8 | 9170009 | 212 | 711 | 8 | 9 | 0 |
| 9 | 9170010 | 256 | 1128 | 23 | 10 | 0 |
| 10 | 9170011 | 286 | 1709 | 29 | 3 | 1 |
| 11 | 9170012 | 243 | 1288 | 21 | 6 | 1 |
| 12 | 9170013 | 240 | 1055 | 27 | 8 | 1 |
| 13 | 9170014 | 270 | 1007 | 24 | 5 | 1 |
| 14 | 9170015 | 215 | 681 | 5 | 10 | 0 |
| 15 | 9170016 | 252 | 783 | 16 | 13 | 1 |
| 16 | 9170017 | 250 | 1132 | 25 | 5 | 1 |
| 17 | 9170018 | 209 | 978 | 30 | 8 | 1 |
| 18 | 9170019 | 203 | 396 | 4 | 3 | 1 |
| 19 | 9170020 | 253 | 904 | 12 | 11 | 1 |
| 20 | 9170021 | 228 | 905 | 15 | 6 | 0 |
| 21 | 9170022 | 246 | 1108 | 24 | 6 | 2 |
| 22 | 9170023 | 285 | 1311 | 25 | 7 | 0 |
| 23 | 9170024 | 346 | 1355 | 24 | 7 | 0 |
| 24 | 9170025 | 174 | 474 | 2 | 10 | 2 |
| 25 | 9170026 | 246 | 833 | 24 | 14 | 1 |
| 26 | 9170027 | 264 | 1106 | 20 | 7 | 0 |
| 27 | 9170028 | 218 | 802 | 9 | 6 | 0 |
| 28 | 9170029 | 274 | 755 | 12 | 4 | 1 |
| 29 | 9170030 | 311 | 1392 | 33 | 10 | 2 |
| 30 | 9170031 | 286 | 1425 | 7 | 7 | 0 |
| 31 | 9170032 | 237 | 746 | 11 | 7 | 2 |
| 32 | 9170033 | 274 | 908 | 13 | 9 | 0 |
| 33 | 9170034 | 349 | 1482 | 31 | 15 | 1 |
| 34 | 9170035 | 215 | 736 | 11 | 14 | 1 |
| 35 | 9170036 | 260 | 1106 | 17 | 4 | 4 |
| 36 | 9170037 | 227 | 769 | 14 | 7 | 1 |
| 37 | 9170038 | 718 | 5481 | 52 | 15 | 1 |
| 38 | 9170039 | 239 | 959 | 20 | 11 | 1 |
| 39 | 9170040 | 265 | 1362 | 21 | 2 | 0 |
| 40 | 9170041 | 241 | 1167 | 25 | 11 | 1 |
| 41 | 9170042 | 249 | 1063 | 12 | 11 | 0 |
| 42 | 9170043 | 151 | 425 | 7 | 7 | 0 |
| 43 | 9170044 | 269 | 965 | 10 | 11 | 0 |
| 44 | 9170045 | 353 | 2136 | 42 | 8 | 0 |
| 45 | 9170046 | 322 | 1835 | 19 | 10 | 2 |
| 46 | 9170047 | 288 | 1258 | 16 | 7 | 1 |
| 47 | 9170048 | 310 | 1266 | 15 | 17 | 1 |
| 48 | 9170049 | 291 | 1383 | 19 | 7 | 1 |
| 49 | 9170050 | 260 | 1000 | 23 | 8 | 2 |
| 50 | 9170051 | 288 | 1377 | 22 | 13 | 1 |
| 51 | 9170052 | 308 | 1266 | 14 | 15 | 0 |
| 52 | 9170053 | 216 | 771 | 12 | 3 | 0 |
| 53 | 9170054 | 294 | 1165 | 14 | 19 | 0 |
| 54 | 9170055 | 261 | 1269 | 12 | 4 | 0 |
| 55 | 9170056 | 270 | 1003 | 18 | 3 | 1 |
| 56 | 9170057 | 199 | 703 | 12 | 8 | 0 |
| 57 | 9170058 | 285 | 1398 | 20 | 16 | 3 |
| 58 | 9170059 | 333 | 1146 | 19 | 7 | 0 |
| 59 | 9170060 | 227 | 760 | 7 | 17 | 0 |
| 60 | 9170061 | 251 | 908 | 16 | 5 | 0 |
| 61 | 9170062 | 213 | 712 | 8 | 8 | 0 |
| 62 | 9170063 | 289 | 1643 | 37 | 8 | 0 |
| 63 | 9170064 | 323 | 1670 | 20 | 6 | 1 |
| 64 | 9170065 | 317 | 1325 | 22 | 14 | 0 |
| 65 | 9170066 | 298 | 1377 | 19 | 9 | 1 |
| 66 | 9170067 | 291 | 1712 | 23 | 8 | 2 |
| 67 | 9170068 | 234 | 1060 | 16 | 13 | 0 |
| 68 | 9170069 | 178 | 521 | 7 | 4 | 1 |
| 69 | 9170070 | 225 | 682 | 6 | 8 | 2 |
| 70 | 9170071 | 212 | 601 | 17 | 0 | 0 |
| 71 | 9170072 | 195 | 744 | 13 | 8 | 0 |
| 72 | 9170073 | 332 | 1479 | 32 | 11 | 2 |
| 73 | 9170074 | 272 | 700 | 6 | 13 | 0 |
| 74 | 9170075 | 220 | 943 | 5 | 11 | 1 |
| 75 | 9170076 | 276 | 835 | 8 | 2 | 0 |
| 76 | 9170077 | 222 | 848 | 12 | 6 | 2 |
| 77 | 9170078 | 284 | 1157 | 16 | 12 | 0 |
| 78 | 9170079 | 217 | 668 | 12 | 6 | 1 |
| 79 | 9170080 | 287 | 1180 | 15 | 11 | 1 |
| 80 | 9170081 | 264 | 898 | 18 | 11 | 0 |
| 81 | 9170082 | 265 | 1377 | 29 | 7 | 0 |
| 82 | 9170083 | 405 | 3332 | 28 | 7 | 0 |
| 83 | 9170084 | 235 | 734 | 17 | 9 | 2 |
| 84 | 9170085 | 276 | 1260 | 16 | 4 | 0 |
| 85 | 9170086 | 246 | 935 | 25 | 5 | 2 |
| 86 | 9170087 | 204 | 693 | 12 | 4 | 2 |
| 87 | 9170088 | 273 | 1497 | 25 | 9 | 0 |
| 88 | 9170089 | 259 | 1018 | 26 | 13 | 1 |
| 89 | 9170090 | 254 | 796 | 13 | 16 | 2 |
| 90 | 9170091 | 320 | 1579 | 21 | 13 | 1 |
| 91 | 9170092 | 235 | 886 | 13 | 4 | 0 |
| 92 | 9170093 | 393 | 2389 | 44 | 9 | 1 |
| 93 | 9170094 | 227 | 990 | 30 | 3 | 0 |
| 94 | 9170095 | 238 | 819 | 11 | 19 | 0 |
| 95 | 9170096 | 227 | 767 | 9 | 5 | 1 |
| 96 | 9170097 | 227 | 687 | 14 | 5 | 1 |
| 97 | 9170098 | 271 | 1261 | 8 | 6 | 1 |
| 98 | 9170099 | 247 | 992 | 13 | 10 | 0 |
| 99 | 9170100 | 317 | 1562 | 18 | 15 | 0 |
| 100 | 9170101 | 236 | 908 | 16 | 9 | 0 |
| 101 | 9170102 | 296 | 1121 | 26 | 13 | 0 |
| 102 | 9170103 | 189 | 748 | 22 | 9 | 0 |
| 103 | 9170104 | 270 | 1121 | 17 | 8 | 0 |
| 104 | 9170105 | 309 | 1125 | 18 | 4 | 2 |
| 105 | 9170106 | 231 | 1030 | 24 | 7 | 1 |
| 106 | 9170107 | 236 | 790 | 15 | 9 | 2 |
| 107 | 9170108 | 291 | 1204 | 12 | 12 | 0 |
| 108 | 9170109 | 250 | 1137 | 10 | 11 | 0 |
| 109 | 9170110 | 334 | 1485 | 13 | 10 | 0 |
| 110 | 9170111 | 222 | 704 | 6 | 14 | 1 |
| 111 | 9170112 | 272 | 1213 | 21 | 12 | 3 |
| 112 | 9170113 | 288 | 1091 | 10 | 7 | 2 |
| 113 | 9170114 | 258 | 1103 | 17 | 11 | 0 |
| 114 | 9170115 | 311 | 1055 | 18 | 0 | 1 |
| 115 | 9170116 | 227 | 613 | 14 | 4 | 0 |
| 116 | 9170117 | 229 | 927 | 13 | 8 | 0 |
| 117 | 9170118 | 192 | 714 | 7 | 1 | 1 |
| 118 | 9170119 | 288 | 1296 | 5 | 16 | 1 |
| 119 | 9170120 | 216 | 775 | 9 | 1 | 2 |
| 120 | 9170121 | 223 | 671 | 6 | 7 | 0 |
| 121 | 9170122 | 286 | 1249 | 20 | 7 | 0 |
| 122 | 9170123 | 224 | 803 | 11 | 4 | 0 |
| 123 | 9170124 | 286 | 1539 | 5 | 4 | 1 |
| 124 | 9170125 | 301 | 1206 | 9 | 8 | 0 |
| 125 | 9170126 | 203 | 448 | 10 | 8 | 0 |
| 126 | 9170127 | 273 | 1233 | 14 | 12 | 1 |
| 127 | 9170128 | 216 | 789 | 17 | 6 | 1 |
| 128 | 9170129 | 249 | 1040 | 8 | 13 | 1 |
| 129 | 9170130 | 212 | 562 | 11 | 12 | 0 |
| 130 | 9170131 | 218 | 804 | 4 | 5 | 0 |
| 131 | 9170132 | 279 | 1235 | 26 | 8 | 0 |
| 132 | 9170133 | 237 | 887 | 21 | 6 | 1 |
| 133 | 9170134 | 239 | 968 | 12 | 7 | 0 |
| 134 | 9170135 | 242 | 1055 | 19 | 10 | 0 |
| 135 | 9170136 | 251 | 1153 | 22 | 11 | 1 |
| 136 | 9170137 | 310 | 1241 | 11 | 11 | 0 |
| 137 | 9170138 | 204 | 389 | 7 | 8 | 1 |
| 138 | 9170139 | 248 | 947 | 9 | 6 | 0 |
| 139 | 9170140 | 321 | 1470 | 20 | 2 | 1 |
| 140 | 9170141 | 221 | 738 | 15 | 6 | 1 |
| 141 | 9170142 | 313 | 1383 | 25 | 9 | 1 |
| 142 | 9170143 | 285 | 1140 | 12 | 4 | 0 |
| 143 | 9170144 | 219 | 675 | 16 | 10 | 0 |
| 144 | 9170145 | 221 | 664 | 10 | 6 | 2 |
| 145 | 9170146 | 229 | 737 | 0 | 10 | 0 |
| 146 | 9170147 | 301 | 1259 | 14 | 7 | 0 |
| 147 | 9170148 | 271 | 1011 | 24 | 4 | 2 |
| 148 | 9170149 | 231 | 856 | 14 | 8 | 0 |
| 149 | 9170150 | 275 | 1121 | 32 | 4 | 1 |
| 150 | 9170151 | 283 | 1077 | 10 | 8 | 0 |
| 151 | 9170152 | 289 | 987 | 5 | 9 | 0 |
| 152 | 9170153 | 229 | 838 | 9 | 10 | 1 |
| 153 | 9170154 | 259 | 1003 | 17 | 13 | 1 |
| 154 | 9170155 | 245 | 1018 | 10 | 15 | 1 |
| 155 | 9170156 | 264 | 1104 | 19 | 5 | 4 |
| 156 | 9170157 | 286 | 1523 | 8 | 14 | 0 |
| 157 | 9170158 | 253 | 994 | 11 | 11 | 1 |
| 158 | 9170159 | 216 | 788 | 16 | 6 | 2 |
| 159 | 9170160 | 200 | 558 | 9 | 5 | 1 |
| 160 | 9170161 | 308 | 1322 | 12 | 11 | 1 |
| 161 | 9170162 | 249 | 721 | 10 | 4 | 1 |
| 162 | 9170163 | 279 | 1189 | 12 | 6 | 0 |
| 163 | 9170164 | 267 | 1238 | 14 | 3 | 1 |
| 164 | 9170165 | 303 | 1302 | 21 | 6 | 2 |
| 165 | 9170166 | 226 | 856 | 25 | 8 | 1 |
| 166 | 9170167 | 276 | 1257 | 29 | 4 | 0 |
| 167 | 9170168 | 239 | 1025 | 21 | 3 | 1 |
| 168 | 9170169 | 252 | 841 | 6 | 5 | 1 |
| 169 | 9170170 | 338 | 1836 | 27 | 10 | 1 |
| 170 | 9170171 | 235 | 780 | 16 | 11 | 0 |
| 171 | 9170172 | 255 | 1065 | 19 | 12 | 2 |
| 172 | 9170173 | 293 | 1343 | 22 | 11 | 2 |
| 173 | 9170174 | 309 | 1609 | 30 | 5 | 1 |
| 174 | 9170175 | 284 | 1200 | 18 | 10 | 1 |
| 175 | 9170176 | 246 | 763 | 10 | 3 | 1 |
| 176 | 9170177 | 252 | 974 | 15 | 13 | 1 |
| 177 | 9170178 | 227 | 1200 | 31 | 5 | 2 |
| 178 | 9170179 | 263 | 1368 | 24 | 5 | 3 |
| 179 | 9170180 | 193 | 598 | 15 | 7 | 1 |
| 180 | 9170181 | 269 | 1004 | 13 | 10 | 1 |
| 181 | 9170182 | 241 | 717 | 5 | 5 | 1 |
| 182 | 9170183 | 290 | 1638 | 7 | 10 | 2 |
| 183 | 9170184 | 352 | 1694 | 22 | 10 | 1 |
| 184 | 9170185 | 225 | 712 | 13 | 5 | 0 |
| 185 | 9170186 | 210 | 468 | 11 | 1 | 2 |
| 186 | 9170187 | 271 | 733 | 7 | 6 | 1 |
| 187 | 9170188 | 245 | 1049 | 4 | 9 | 0 |
| 188 | 9170189 | 321 | 1264 | 19 | 29 | 1 |
| 189 | 9170190 | 224 | 945 | 15 | 7 | 1 |
| 190 | 9170191 | 263 | 1236 | 29 | 7 | 0 |
| 191 | 9170192 | 242 | 832 | 10 | 2 | 0 |
| 192 | 9170193 | 243 | 808 | 11 | 0 | 1 |
| 193 | 9170194 | 229 | 712 | 19 | 12 | 0 |
| 194 | 9170195 | 289 | 808 | 14 | 1 | 1 |
| 195 | 9170196 | 271 | 813 | 17 | 8 | 0 |
| 196 | 9170197 | 261 | 953 | 9 | 9 | 0 |
| 197 | 9170198 | 312 | 1344 | 13 | 3 | 0 |
| 198 | 9170199 | 287 | 1392 | 25 | 14 | 0 |
| 199 | 9170200 | 177 | 623 | 14 | 4 | 0 |
| 200 | 9170201 | 295 | 1407 | 36 | 6 | 2 |
| 201 | 9170202 | 238 | 896 | 10 | 7 | 0 |
| 202 | 9170203 | 320 | 1293 | 22 | 6 | 1 |
| 203 | 9170204 | 259 | 992 | 9 | 9 | 2 |
| 204 | 9170205 | 298 | 1512 | 20 | 11 | 3 |
| 205 | 9170206 | 236 | 881 | 13 | 9 | 1 |
| 206 | 9170207 | 310 | 1067 | 15 | 9 | 0 |
| 207 | 9170208 | 258 | 824 | 14 | 5 | 0 |
| 208 | 9170209 | 325 | 1367 | 12 | 13 | 0 |
| 209 | 9170210 | 285 | 1014 | 10 | 9 | 0 |
| 210 | 9170211 | 285 | 1358 | 21 | 6 | 0 |
| 211 | 9170212 | 252 | 1148 | 24 | 3 | 1 |
| 212 | 9170213 | 205 | 582 | 8 | 10 | 1 |
| 213 | 9170214 | 232 | 890 | 11 | 3 | 0 |
| 214 | 9170215 | 258 | 801 | 18 | 6 | 2 |
| 215 | 9170216 | 228 | 684 | 9 | 6 | 1 |
| 216 | 9170217 | 263 | 946 | 19 | 5 | 0 |
| 217 | 9170218 | 284 | 1102 | 27 | 11 | 1 |
| 218 | 9170219 | 285 | 945 | 11 | 8 | 2 |
| 219 | 9170220 | 340 | 2136 | 26 | 9 | 0 |
| 220 | 9170221 | 323 | 1329 | 6 | 12 | 0 |
| 221 | 9170222 | 265 | 1199 | 32 | 8 | 2 |
| 222 | 9170223 | 265 | 1305 | 31 | 6 | 1 |
| 223 | 9170224 | 228 | 949 | 13 | 9 | 2 |
| 224 | 9170225 | 279 | 1410 | 29 | 11 | 2 |
| 225 | 9170226 | 228 | 758 | 10 | 2 | 1 |
| 226 | 9170227 | 313 | 1246 | 23 | 12 | 3 |
| 227 | 9170228 | 314 | 1206 | 20 | 9 | 2 |
| 228 | 9170229 | 241 | 674 | 8 | 8 | 0 |
| 229 | 9170230 | 261 | 1004 | 10 | 6 | 1 |
| 230 | 9170231 | 264 | 1221 | 11 | 11 | 0 |
| 231 | 9170232 | 231 | 890 | 13 | 17 | 1 |
| 232 | 9170233 | 302 | 1378 | 13 | 9 | 1 |
| 233 | 9170234 | 234 | 943 | 12 | 5 | 0 |
| 234 | 9170235 | 225 | 646 | 8 | 7 | 0 |
| 235 | 9170236 | 445 | 2061 | 44 | 6 | 1 |
| 236 | 9170237 | 358 | 2330 | 12 | 13 | 1 |
| 237 | 9170238 | 257 | 825 | 6 | 4 | 1 |
| 238 | 9170239 | 255 | 908 | 18 | 14 | 1 |
| 239 | 9170240 | 314 | 939 | 8 | 4 | 0 |
| 240 | 9170241 | 267 | 934 | 12 | 11 | 0 |
| 241 | 9170242 | 211 | 892 | 13 | 11 | 2 |
| 242 | 9170243 | 238 | 970 | 17 | 7 | 0 |
| 243 | 9170244 | 327 | 1355 | 27 | 6 | 2 |
| 244 | 9170245 | 291 | 1166 | 18 | 6 | 1 |
| 245 | 9170246 | 214 | 760 | 18 | 5 | 2 |
| 246 | 9170247 | 228 | 828 | 21 | 3 | 1 |
| 247 | 9170248 | 195 | 575 | 10 | 3 | 0 |
| 248 | 9170249 | 270 | 1226 | 16 | 6 | 0 |
| 249 | 9170250 | 267 | 1249 | 25 | 2 | 0 |
| 250 | 9170251 | 236 | 795 | 5 | 2 | 2 |
| 251 | 9170252 | 341 | 1559 | 22 | 7 | 2 |
| 252 | 9170253 | 246 | 975 | 12 | 7 | 2 |
| 253 | 9170254 | 204 | 595 | 10 | 5 | 0 |
| 254 | 9170255 | 253 | 765 | 5 | 12 | 0 |
| 255 | 9170256 | 229 | 662 | 10 | 9 | 0 |
| 256 | 9170257 | 292 | 1359 | 16 | 3 | 1 |
| 257 | 9170258 | 276 | 1140 | 14 | 10 | 1 |
| 258 | 9170259 | 215 | 580 | 3 | 8 | 1 |
| 259 | 9170260 | 309 | 1458 | 24 | 7 | 3 |
| 260 | 9170261 | 265 | 825 | 9 | 3 | 0 |
| 261 | 9170262 | 292 | 1420 | 15 | 7 | 0 |
| 262 | 9170263 | 248 | 818 | 24 | 5 | 0 |
| 263 | 9170264 | 248 | 1129 | 12 | 6 | 1 |
| 264 | 9170265 | 243 | 906 | 8 | 7 | 1 |
| 265 | 9170266 | 256 | 926 | 13 | 19 | 0 |
| 266 | 9170267 | 373 | 1603 | 23 | 16 | 1 |
| 267 | 9170268 | 283 | 1415 | 11 | 9 | 0 |
| 268 | 9170269 | 238 | 909 | 13 | 6 | 1 |
| 269 | 9170270 | 225 | 804 | 19 | 7 | 1 |
| 270 | 9170271 | 238 | 879 | 13 | 5 | 1 |
| 271 | 9170272 | 219 | 717 | 12 | 11 | 2 |
| 272 | 9170273 | 255 | 1461 | 38 | 9 | 0 |
| 273 | 9170274 | 262 | 1336 | 18 | 7 | 3 |
| 274 | 9170275 | 249 | 803 | 13 | 11 | 1 |
| 275 | 9170276 | 285 | 1273 | 26 | 4 | 1 |
| 276 | 9170277 | 294 | 1197 | 14 | 3 | 0 |
| 277 | 9170278 | 297 | 1031 | 19 | 6 | 1 |
| 278 | 9170279 | 381 | 1856 | 32 | 12 | 1 |
| 279 | 9170280 | 227 | 807 | 12 | 6 | 2 |
| 280 | 9170281 | 310 | 1795 | 16 | 16 | 0 |
| 281 | 9170282 | 282 | 1078 | 24 | 13 | 1 |
| 282 | 9170283 | 238 | 961 | 12 | 10 | 0 |
| 283 | 9170284 | 252 | 1494 | 14 | 12 | 2 |
| 284 | 9170285 | 211 | 814 | 14 | 5 | 1 |
| 285 | 9170286 | 259 | 1563 | 26 | 10 | 2 |
| 286 | 9170287 | 203 | 581 | 4 | 2 | 1 |
| 287 | 9170288 | 251 | 1070 | 21 | 5 | 1 |
| 288 | 9170289 | 288 | 1805 | 41 | 8 | 0 |
| 289 | 9170290 | 203 | 719 | 5 | 5 | 0 |
| 290 | 9170291 | 208 | 740 | 16 | 6 | 1 |
| 291 | 9170292 | 208 | 711 | 9 | 10 | 1 |
| 292 | 9170293 | 290 | 1338 | 21 | 11 | 0 |
| 293 | 9170294 | 260 | 1053 | 22 | 15 | 0 |
| 294 | 9170295 | 305 | 1299 | 25 | 7 | 0 |
| 295 | 9170296 | 326 | 1254 | 18 | 9 | 0 |
| 296 | 9170297 | 229 | 682 | 9 | 6 | 1 |
| 297 | 9170298 | 282 | 1146 | 7 | 15 | 0 |
| 298 | 9170299 | 243 | 1202 | 31 | 4 | 0 |
| 299 | 9170300 | 216 | 592 | 3 | 6 | 0 |
| 300 | 9170301 | 245 | 877 | 10 | 5 | 1 |
| 301 | 9170302 | 187 | 439 | 4 | 8 | 1 |
| 302 | 9170303 | 222 | 915 | 16 | 4 | 0 |
| 303 | 9170304 | 413 | 1981 | 21 | 8 | 2 |
| 304 | 9170305 | 253 | 990 | 14 | 14 | 1 |
| 305 | 9170306 | 177 | 502 | 6 | 8 | 0 |
| 306 | 9170307 | 307 | 1465 | 6 | 10 | 1 |
| 307 | 9170308 | 314 | 1571 | 16 | 10 | 2 |
| 308 | 9170309 | 251 | 1175 | 35 | 6 | 1 |
| 309 | 9170310 | 299 | 1222 | 6 | 6 | 0 |
| 310 | 9170311 | 224 | 1006 | 20 | 2 | 0 |
| 311 | 9170312 | 230 | 641 | 6 | 11 | 1 |
| 312 | 9170313 | 258 | 1314 | 24 | 11 | 1 |
| 313 | 9170314 | 270 | 1141 | 16 | 13 | 1 |
| 314 | 9170315 | 342 | 2303 | 31 | 2 | 1 |
| 315 | 9170316 | 263 | 1299 | 13 | 6 | 1 |
| 316 | 9170317 | 226 | 745 | 13 | 6 | 0 |
| 317 | 9170318 | 313 | 1313 | 19 | 6 | 1 |
| 318 | 9170319 | 252 | 1280 | 11 | 12 | 0 |
| 319 | 9170320 | 273 | 1374 | 22 | 7 | 0 |
| 320 | 9170321 | 341 | 1193 | 24 | 6 | 1 |
| 321 | 9170322 | 229 | 854 | 20 | 6 | 0 |
| 322 | 9170323 | 248 | 800 | 10 | 6 | 2 |
| 323 | 9170324 | 261 | 842 | 5 | 7 | 0 |
| 324 | 9170325 | 232 | 724 | 6 | 4 | 1 |
| 325 | 9170326 | 219 | 1032 | 9 | 9 | 0 |
| 326 | 9170327 | 260 | 1314 | 18 | 7 | 1 |
| 327 | 9170328 | 240 | 876 | 7 | 4 | 0 |
| 328 | 9170329 | 328 | 1696 | 14 | 9 | 1 |
| 329 | 9170330 | 244 | 904 | 35 | 2 | 1 |
| 330 | 9170331 | 209 | 819 | 17 | 3 | 2 |
| 331 | 9170332 | 208 | 685 | 8 | 3 | 1 |
| 332 | 9170333 | 247 | 1213 | 10 | 6 | 0 |
| 333 | 9170334 | 216 | 654 | 8 | 9 | 1 |
| 334 | 9170335 | 219 | 679 | 11 | 4 | 0 |
| 335 | 9170336 | 238 | 983 | 12 | 7 | 0 |
| 336 | 9170337 | 336 | 2418 | 53 | 7 | 0 |
| 337 | 9170338 | 247 | 1086 | 12 | 6 | 1 |
| 338 | 9170339 | 247 | 880 | 7 | 7 | 1 |
| 339 | 9170340 | 335 | 1592 | 26 | 7 | 1 |
| 340 | 9170341 | 317 | 1329 | 7 | 15 | 0 |
| 341 | 9170342 | 187 | 646 | 14 | 7 | 1 |
| 342 | 9170343 | 222 | 740 | 10 | 5 | 0 |
| 343 | 9170344 | 243 | 1095 | 12 | 6 | 0 |
| 344 | 9170345 | 251 | 999 | 28 | 7 | 1 |
| 345 | 9170346 | 323 | 1269 | 16 | 13 | 0 |
| 346 | 9170347 | 292 | 1218 | 10 | 18 | 0 |
| 347 | 9170348 | 258 | 870 | 12 | 4 | 0 |
| 348 | 9170349 | 301 | 1651 | 39 | 8 | 0 |
| 349 | 9170350 | 290 | 1351 | 21 | 5 | 1 |
| 350 | 9170351 | 275 | 1256 | 11 | 9 | 0 |
| 351 | 9170352 | 261 | 1101 | 13 | 8 | 0 |
| 352 | 9170353 | 264 | 746 | 6 | 8 | 0 |
| 353 | 9170354 | 240 | 891 | 11 | 8 | 1 |
| 354 | 9170355 | 285 | 1422 | 31 | 14 | 1 |
| 355 | 9170356 | 235 | 595 | 3 | 5 | 0 |
| 356 | 9170357 | 214 | 819 | 17 | 9 | 0 |
| 357 | 9170358 | 340 | 1299 | 10 | 8 | 0 |
| 358 | 9170359 | 375 | 1642 | 20 | 8 | 0 |
| 359 | 9170360 | 188 | 605 | 2 | 13 | 0 |
| 360 | 9170361 | 253 | 910 | 12 | 11 | 2 |
| 361 | 9170362 | 268 | 994 | 9 | 2 | 1 |
| 362 | 9170363 | 302 | 1541 | 30 | 6 | 1 |
| 363 | 9170364 | 292 | 898 | 4 | 3 | 0 |
| 364 | 9170365 | 210 | 553 | 24 | 0 | 2 |
| 365 | 9170366 | 243 | 825 | 12 | 3 | 0 |
| 366 | 9170367 | 257 | 1174 | 11 | 9 | 0 |
| 367 | 9170368 | 234 | 769 | 9 | 10 | 2 |
| 368 | 9170369 | 299 | 1795 | 20 | 19 | 0 |
| 369 | 9170370 | 302 | 1216 | 14 | 12 | 0 |
| 370 | 9170371 | 214 | 822 | 16 | 8 | 0 |
| 371 | 9170372 | 248 | 855 | 8 | 15 | 1 |
| 372 | 9170373 | 220 | 773 | 15 | 6 | 0 |
| 373 | 9170374 | 280 | 1548 | 8 | 14 | 0 |
| 374 | 9170375 | 218 | 606 | 9 | 3 | 2 |
| 375 | 9170376 | 323 | 1193 | 9 | 7 | 0 |
| 376 | 9170377 | 257 | 1133 | 7 | 9 | 0 |
| 377 | 9170378 | 263 | 1003 | 20 | 2 | 1 |
| 378 | 9170379 | 297 | 1508 | 20 | 7 | 0 |
| 379 | 9170380 | 217 | 655 | 17 | 1 | 1 |
| 380 | 9170381 | 225 | 823 | 13 | 6 | 0 |
| 381 | 9170382 | 274 | 1063 | 11 | 7 | 2 |
| 382 | 9170383 | 456 | 2029 | 21 | 3 | 0 |
| 383 | 9170384 | 212 | 706 | 6 | 6 | 0 |
| 384 | 9170385 | 283 | 1487 | 15 | 9 | 1 |
| 385 | 9170386 | 257 | 733 | 5 | 3 | 0 |
| 386 | 9170387 | 242 | 1132 | 28 | 4 | 3 |
| 387 | 9170388 | 298 | 1121 | 14 | 13 | 1 |
| 388 | 9170389 | 316 | 859 | 16 | 7 | 2 |
| 389 | 9170390 | 308 | 1265 | 22 | 18 | 1 |
| 390 | 9170391 | 264 | 922 | 2 | 9 | 0 |
| 391 | 9170392 | 249 | 968 | 21 | 6 | 0 |
| 392 | 9170393 | 248 | 985 | 9 | 15 | 1 |
| 393 | 9170394 | 238 | 744 | 5 | 12 | 1 |
| 394 | 9170395 | 229 | 836 | 9 | 10 | 0 |
| 395 | 9170396 | 286 | 1145 | 22 | 12 | 3 |
| 396 | 9170397 | 233 | 863 | 15 | 8 | 0 |
| 397 | 9170398 | 267 | 887 | 10 | 4 | 1 |
| 398 | 9170399 | 249 | 833 | 16 | 17 | 0 |
| 399 | 9170400 | 294 | 1068 | 14 | 10 | 1 |
| 400 | 9170401 | 236 | 1087 | 7 | 6 | 0 |
| 401 | 9170402 | 176 | 415 | 13 | 3 | 2 |
| 402 | 9170403 | 342 | 1965 | 17 | 12 | 1 |
| 403 | 9170404 | 324 | 1887 | 35 | 6 | 0 |
| 404 | 9170405 | 217 | 931 | 21 | 0 | 1 |
| 405 | 9170406 | 215 | 860 | 14 | 9 | 1 |
| 406 | 9170407 | 269 | 1275 | 23 | 15 | 2 |
| 407 | 9170408 | 267 | 1070 | 21 | 8 | 1 |
| 408 | 9170409 | 207 | 604 | 3 | 6 | 1 |
| 409 | 9170410 | 249 | 1099 | 13 | 10 | 2 |
| 410 | 9170411 | 235 | 760 | 11 | 6 | 0 |
| 411 | 9170412 | 212 | 462 | 5 | 7 | 1 |
| 412 | 9170413 | 258 | 1019 | 24 | 7 | 2 |
| 413 | 9170414 | 220 | 651 | 10 | 3 | 0 |
| 414 | 9170415 | 220 | 698 | 7 | 10 | 0 |
| 415 | 9170416 | 215 | 486 | 5 | 8 | 0 |
| 416 | 9170417 | 196 | 713 | 17 | 9 | 0 |
| 417 | 9170418 | 251 | 653 | 17 | 13 | 0 |
| 418 | 9170419 | 275 | 1236 | 19 | 13 | 0 |
| 419 | 9170420 | 270 | 1289 | 15 | 3 | 0 |
| 420 | 9170421 | 239 | 1038 | 13 | 6 | 1 |
| 421 | 9170422 | 249 | 621 | 5 | 10 | 2 |
| 422 | 9170423 | 265 | 777 | 11 | 15 | 0 |
| 423 | 9170424 | 244 | 840 | 14 | 4 | 0 |
| 424 | 9170425 | 236 | 891 | 6 | 6 | 0 |
| 425 | 9170426 | 290 | 1277 | 18 | 9 | 1 |
| 426 | 9170427 | 337 | 2454 | 27 | 10 | 2 |
| 427 | 9170428 | 286 | 1167 | 21 | 8 | 2 |
| 428 | 9170429 | 205 | 685 | 21 | 5 | 0 |
| 429 | 9170430 | 197 | 421 | 1 | 2 | 1 |
| 430 | 9170431 | 239 | 982 | 13 | 10 | 1 |
| 431 | 9170432 | 223 | 681 | 4 | 10 | 0 |
| 432 | 9170433 | 196 | 430 | 2 | 6 | 1 |
| 433 | 9170434 | 239 | 1038 | 33 | 6 | 1 |
| 434 | 9170435 | 233 | 1035 | 11 | 8 | 0 |
| 435 | 9170436 | 238 | 904 | 9 | 8 | 0 |
| 436 | 9170437 | 215 | 723 | 5 | 5 | 1 |
| 437 | 9170438 | 208 | 757 | 9 | 3 | 2 |
| 438 | 9170439 | 248 | 1187 | 11 | 3 | 0 |
| 439 | 9170440 | 264 | 924 | 14 | 7 | 1 |
| 440 | 9170441 | 226 | 1018 | 14 | 9 | 2 |
| 441 | 9170442 | 289 | 1113 | 22 | 10 | 2 |
| 442 | 9170443 | 247 | 861 | 6 | 8 | 0 |
| 443 | 9170444 | 249 | 873 | 10 | 8 | 0 |
| 444 | 9170445 | 304 | 1426 | 6 | 12 | 0 |
| 445 | 9170446 | 245 | 820 | 13 | 5 | 1 |
| 446 | 9170447 | 312 | 1170 | 16 | 13 | 0 |
| 447 | 9170448 | 263 | 837 | 6 | 8 | 1 |
| 448 | 9170449 | 222 | 765 | 15 | 4 | 1 |
| 449 | 9170450 | 267 | 1060 | 17 | 3 | 1 |
| 450 | 9170451 | 322 | 1436 | 21 | 11 | 1 |
| 451 | 9170452 | 222 | 1053 | 15 | 6 | 2 |
| 452 | 9170453 | 272 | 1424 | 6 | 5 | 0 |
| 453 | 9170454 | 259 | 942 | 22 | 8 | 2 |
| 454 | 9170455 | 280 | 1004 | 16 | 6 | 1 |
| 455 | 9170456 | 249 | 1046 | 15 | 10 | 1 |
| 456 | 9170457 | 213 | 677 | 15 | 13 | 0 |
| 457 | 9170458 | 249 | 1054 | 14 | 4 | 0 |
| 458 | 9170459 | 225 | 696 | 16 | 1 | 2 |
| 459 | 9170460 | 278 | 1283 | 6 | 16 | 0 |
| 460 | 9170461 | 255 | 1154 | 13 | 7 | 1 |
| 461 | 9170462 | 227 | 860 | 17 | 11 | 0 |
| 462 | 9170463 | 368 | 1721 | 26 | 7 | 1 |
| 463 | 9170464 | 218 | 763 | 16 | 7 | 0 |
| 464 | 9170465 | 268 | 1421 | 16 | 7 | 2 |
| 465 | 9170466 | 296 | 1755 | 24 | 8 | 0 |
| 466 | 9170467 | 325 | 1968 | 38 | 7 | 1 |
| 467 | 9170468 | 317 | 1484 | 10 | 9 | 0 |
| 468 | 9170469 | 238 | 793 | 14 | 9 | 1 |
| 469 | 9170470 | 236 | 818 | 3 | 3 | 0 |
| 470 | 9170471 | 237 | 1236 | 29 | 5 | 1 |
| 471 | 9170472 | 295 | 984 | 9 | 2 | 0 |
| 472 | 9170473 | 229 | 824 | 14 | 7 | 1 |
| 473 | 9170474 | 309 | 1660 | 13 | 7 | 2 |
| 474 | 9170475 | 292 | 1341 | 11 | 8 | 1 |
| 475 | 9170476 | 244 | 961 | 15 | 9 | 0 |
| 476 | 9170477 | 298 | 1092 | 20 | 7 | 1 |
| 477 | 9170478 | 256 | 1049 | 15 | 9 | 0 |
| 478 | 9170479 | 227 | 834 | 22 | 4 | 2 |
| 479 | 9170480 | 294 | 1280 | 25 | 5 | 1 |
| 480 | 9170481 | 276 | 1113 | 11 | 7 | 0 |
| 481 | 9170482 | 244 | 1016 | 16 | 8 | 1 |
| 482 | 9170483 | 230 | 726 | 12 | 7 | 0 |
| 483 | 9170484 | 304 | 1701 | 15 | 8 | 0 |
| 484 | 9170485 | 217 | 860 | 20 | 3 | 0 |
| 485 | 9170486 | 294 | 1110 | 13 | 9 | 2 |
| 486 | 9170487 | 282 | 1314 | 13 | 11 | 0 |
| 487 | 9170488 | 171 | 294 | 6 | 3 | 1 |
| 488 | 9170489 | 227 | 718 | 13 | 10 | 0 |
| 489 | 9170490 | 286 | 1326 | 7 | 5 | 0 |
| 490 | 9170491 | 315 | 1303 | 23 | 7 | 1 |
| 491 | 9170492 | 232 | 1012 | 24 | 4 | 1 |
| 492 | 9170493 | 314 | 1822 | 12 | 7 | 1 |
| 493 | 9170494 | 217 | 561 | 9 | 4 | 1 |
| 494 | 9170495 | 287 | 827 | 2 | 8 | 0 |
| 495 | 9170496 | 270 | 1211 | 22 | 7 | 0 |
| 496 | 9170497 | 222 | 750 | 5 | 4 | 1 |
| 497 | 9170498 | 328 | 1753 | 34 | 5 | 3 |
| 498 | 9170499 | 259 | 893 | 12 | 16 | 0 |
| 499 | 9170500 | 189 | 762 | 10 | 3 | 2 |
| 500 | 9170501 | 209 | 641 | 9 | 10 | 0 |
| 501 | 9170502 | 220 | 833 | 11 | 3 | 1 |
| 502 | 9170503 | 335 | 1560 | 18 | 8 | 0 |
| 503 | 9170504 | 333 | 2566 | 23 | 10 | 1 |
| 504 | 9170505 | 191 | 677 | 10 | 10 | 0 |
| 505 | 9170506 | 253 | 984 | 10 | 3 | 1 |
| 506 | 9170507 | 304 | 958 | 8 | 8 | 0 |
| 507 | 9170508 | 221 | 922 | 9 | 5 | 0 |
| 508 | 9170509 | 257 | 1364 | 29 | 11 | 0 |
| 509 | 9170510 | 241 | 952 | 19 | 6 | 1 |
| 510 | 9170511 | 314 | 1512 | 21 | 6 | 1 |
| 511 | 9170512 | 264 | 861 | 10 | 9 | 0 |

## Throughput

- Turns simulated: 133940 (first play of each game)
- Aggregate: 4957.6 turns/s over 27.0s wall
- Per game thread: 90.1 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.

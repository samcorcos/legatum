# Simulation: diplomacy-slice-2 (2026-09-27)

This slice completes the approved Skateboard pact, shared Influence, trust and inbox rules, without trade or alliances. The required **1,000 Standard games** (seeds **700001–701000**, Normal scripted AI, eight solo teams) all reached the VP target. Determinism passed **1,000/1,000** complete second plays. **Faction balance fails pillar 8: 19.5% / 4.7% = 4.1489×**, above the ≤1.5× target.

| Requested measure | Result |
| --- | ---: |
| Mean / median length | 301.63 / 298 turns |
| P10 / P90; range | 250 / 354; 151–652 turns |
| VP share | 1,000/1,000 (100%) |
| Stalls / panics / determinism mismatches | 0 / 0 / 0 |
| City captures | 2,308 (2.308/game) |
| Settler captures | 3,996 (3.996/game) |
| Declarations / pairwise war starts | 21,744 / 21,740 |
| Pacts signed | 58,222 (58.222/game) |
| Pacts kept through minimum | 54,376 (93.39% of signed terms) |
| Pacts broken | 111 (in 93/1,000 games) |
| Peace offers / agreements | 56,193 / 19,322 |
| Eligible leader turns at war | 66,725/90,588 (73.66%) |
| Eligible leader turns involving battle | 33,987/90,588 (37.52%) |
| Games with an eligible leader in battle | 955/1,000 (95.5%) |
| Eligible leader turns losing a city | 180/90,588 (0.20%); 136 games |

A pact is "kept" when its once-per-term honor event fires at the eight-turn minimum, even if it later ends or is breached. "Broken" counts distinct treaty IDs, so mutual breaking declarations do not count the same pact twice. Kept and broken are not mutually exclusive. Unfinished terms remain in the signed denominator.

Leader observations use the post-resolution public VP progress table, with lowest TeamId breaking ties and exactly the spec's coalition thresholds (80%, or 60% with a 15-point lead). "At war" includes declarations just activated for the next planning turn. "In battle" means that turn contained any battle involving a member of that leader's team, including battles the leader initiated and Scout encounters; it is a transparent contestation proxy, not proof of a successful intervention. Captured-city turns count a loss by that post-resolution leader. This does not measure every title challenge or claim causal coalition success.

Raw per-game evidence, hashes and measurement metadata: compressed JSON (outside git: `~/LegatumData/sim-artifacts/2026-09-27-diplomacy/2026-09-27-diplomacy-slice-2.json.gz`). SHA-256 of the uncompressed JSON: `79296f203bbbb8fb95df49059e7ab83b064efaff0811f00611f2cf4cd261978d`. The run used the final gameplay rules and AI scores; subsequent changes were presentation/preview corrections, mock parity, tests and compatibility/check fixes, not native game outcomes. The dirty revision shown below is the required dependency merge; this report is committed together with the implementation.

These are development seeds, one speed and solo teams. They do not satisfy the broader pillar-8 balance campaign (about 2,000 games per speed plus held-out seeds). Teammate behavior is covered by boundary, LAN, UI and replay tests. No faction stats, prices or AI formula bonuses were retuned to hide the failed balance result.

The approved war formula caps the non-leader strength advantage bonus at +25. With default aggression 25–45, several economic personalities cannot reach the 70 declaration threshold without the coalition bonus; this differs intentionally from the merged experiment's additional expansion/title bonuses. The inherited fog-safe campaign planner and title prioritization remain. This is a concrete tuning question for a later approved balance change.

Final verification:

| Check | Result |
| --- | --- |
| `cargo fmt --all` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace` | 547 passed, 0 failed, 4 existing ignored tests |
| `scripts/build-wasm.sh` | Pass |
| `cargo test --release -p legatum-sim --test determinism` | 8 passed |
| `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures` | 5 tapes, 147 turns; initial, every-turn and save/load hashes match |
| `cd app && npm test` | 562 passed across 51 files, including real-WASM UI and mock parity |
| `cd app && npm run build` | Pass |

Commands used `PATH=$HOME/.cargo/bin:/opt/homebrew/bin:$PATH`. WASM builds used `WASM_PACK_CACHE=/tmp/legatum-diplomacy-wasm-cache` because the default cache is outside this workspace's write permissions. The four ignored Rust tests are the existing midgame timing benchmark, paired AI evaluation, many-game summary, and explicit cross-build visibility corpus check. The separately requested 1,000-game determinism campaign above was completed.

Border dates are forecasts over at most 512 turns at current output and known territory, with no further spending or competing claims; this assumption appears in the UI. The mock uses the same negotiation formulas and rule boundaries but retains its existing simpler military planner. Optional browser visual review was not completed because localhost access was declined; no screenshot evidence is claimed.

1000 games, seeds 700001.. (+i), brain `scripted`, speed `Standard`, turn limit off, safety cap 1200, 8 threads, started 2026-09-27T15:18:13Z. Wall time 1179.0s.

Source checkout at run start: `c03e3f88ff3af3c9fde7ae50a252a5d7e7a58379`; dirty: yes. Map 48×32, 8 initial seats. `first_setup` and exact binary argv are recorded in JSON; later seeds use wrapping addition and a deterministic random shuffle of all eight factions (one seat each).

Command argv: `["./target/release/legatum-sim","--games","1000","--seed","700001","--speed","standard","--determinism","--threads","8","--label","diplomacy-slice-2","--status-file","$HOME/code/legatum/.progress/diplomacy-slice-2.json"]`

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP target | 1000 | 100.0% |
| Conquest | 0 | 0.0% |
| Turn limit | 0 | 0.0% |
| Draw | 0 | 0.0% |
| Stalled (safety cap) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 1000 | 195 | 19.5% |
| Japan | 1000 | 168 | 16.8% |
| Byzantium | 1000 | 78 | 7.8% |
| Germany | 1000 | 65 | 6.5% |
| Celts | 1000 | 139 | 13.9% |
| Mongols | 1000 | 181 | 18.1% |
| Sparta | 1000 | 127 | 12.7% |
| Athens | 1000 | 47 | 4.7% |

## Win rate by difficulty

All initial seats in non-panicked games, including eliminated empires and cap stops. Determinism replays are not additional samples.

| Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Normal | 8000 | 1000 | 12.5% |

## Win rate by faction and difficulty

| Faction | Difficulty | Seats | Wins | Win rate |
| --- | --- | --- | --- | --- |
| Switzerland | Normal | 1000 | 195 | 19.5% |
| Japan | Normal | 1000 | 168 | 16.8% |
| Byzantium | Normal | 1000 | 78 | 7.8% |
| Germany | Normal | 1000 | 65 | 6.5% |
| Celts | Normal | 1000 | 139 | 13.9% |
| Mongols | Normal | 1000 | 181 | 18.1% |
| Sparta | Normal | 1000 | 127 | 12.7% |
| Athens | Normal | 1000 | 47 | 4.7% |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Observed game length (turns, including cap stops) | 151 | 250 | 298 | 354 | 652 |
| Winning team VP | 10 | 10 | 10 | 11 | 11 |
| First Soldier completed (turn, per empire) | 4 | 19 | 24 | 49 | 95 |
| First Settler completed (turn, per empire) | 10 | 36 | 56 | 74 | 114 |
| Second city founded (turn, per empire) | 13 | 41 | 61 | 82 | 291 |

Safety-cap stops are censored observations, not completed game lengths.

Openings: 8000 observed empires in 1000 first plays; 0 games without opening telemetry excluded. Turn distributions include only empires that reached the milestone. Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).

| Opening milestone | Reached empires | Never empires |
| --- | --- | --- |
| First Soldier completed | 8000 | 0 |
| First Settler completed | 8000 | 0 |
| Second city founded | 7900 | 100 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 37 |
| 1 | 689 |
| 2 | 546 |
| 3 | 915 |
| 4 | 512 |
| 5 | 1362 |
| 6 | 531 |
| 7 | 1552 |
| 8 | 775 |
| 9 | 81 |
| 10 | 848 |
| 11 | 152 |

## End state

- Average game length: 301.6 turns
- Average cities per empire: 4.24
- Average techs per empire: 18.64
- Average eliminations per game: 0.12

## Battle and capture telemetry

1000 observed first plays; 0 records without telemetry excluded.

| Event | Total / observed games | Mean per game | Games with event / observed games |
| --- | --- | --- | --- |
| War declarations | 21744/1000 | 21.744 | 999/1000 (99.9%) |
| Peace offers | 56193/1000 | 56.193 | 999/1000 (99.9%) |
| Peace agreements | 19322/1000 | 19.322 | 990/1000 (99.0%) |
| Battles | 216905/1000 | 216.905 | 998/1000 (99.8%) |
| City captures | 2308/1000 | 2.308 | 830/1000 (83.0%) |
| Settler captures | 3996/1000 | 3.996 | 879/1000 (87.9%) |
| Eliminations | 122/1000 | 0.122 | 120/1000 (12.0%) |

## Terminal VP sources

1000 finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.

| Source | All players | All teams | Winning teams |
| --- | --- | --- | --- |
| Live title: LargestEconomy | 1000/8000 | 1000/8000 | 742/1000 |
| Live title: MostTerritory | 1000/8000 | 1000/8000 | 567/1000 |
| Live title: StrongestArmy | 1000/8000 | 1000/8000 | 835/1000 |
| Live title: MostAdvanced | 1000/8000 | 1000/8000 | 392/1000 |
| Live title: MostPopulous | 1000/8000 | 1000/8000 | 795/1000 |
| Live titles subtotal | 5000/8000 | 5000/8000 | 3331/1000 |
| Milestone: FourCities | 6578/8000 | 6578/8000 | 1000/1000 |
| Milestone: SixTechs | 7957/8000 | 7957/8000 | 1000/1000 |
| Milestone: TwentyPopulation | 5634/8000 | 5634/8000 | 999/1000 |
| Milestone: RelayUplink | 8096/8000 | 8096/8000 | 1840/1000 |
| Milestones subtotal | 28265/8000 | 28265/8000 | 4839/1000 |
| Faction goals | 11250/8000 | 11250/8000 | 1982/1000 |
| Total VP | 44515/8000 | 44515/8000 | 10152/1000 |

Reconciliation failures: 0 player observations, 0 team observations.

## Turn-50 team leaders

| Cohort | Eventual leader-set wins / eligible games | Draws (included) |
| --- | --- | --- |
| Unique leader | 177/641 (27.6%) | 0 |
| Tied leaders (winner in set) | 164/359 (45.7%) | 0 |
| Any leader set | 341/1000 (34.1%) | 0 |

Excluded: 0 games ended before turn 50, 0 unfinished games, 0 snapshots without teams, 0 records without telemetry.

## Metric definitions

One Event::Battle, including Scout encounters, ranged shots and meeting engagements; not a war, siege or submitted attack order.

City captures use the existing GameRecord.city_captures count of Event::CityCaptured (recaptures count again). Settler captures count Event::SettlerCaptured separately; neither is inferred from ownership or unit losses.

Count the existing GameRecord.eliminations entries, one per Event::PlayerEliminated; telemetry.elimination_count is their length. Never infer from final alive flags.

Final-state VP: each currently held title is 1 VP; each permanent milestone uses core milestone_vp; completed faction goals use core FACTION_GOAL_VP. Title transfers are not accumulated. Player sources reconcile to core vp_total; team sources sum each member once and reconcile to core team_vp. Player, team and winning-team populations are separate views, never added together. Eliminated members retain banked VP, as in core scoring.

After resolution AND scoring of turn 50, when GameState.turn is 51, before turn-51 orders. Rank all initial TEAMS by core team_vp (sum every member once), matching the probe's all-seat population. Equal VP preserves the entire leader set: no population or ID tie-break. Games ending on turn 50 are eligible; games ending earlier are excluded. Turn-75 replay comeback data is independent.

Per initial empire, record the resolved turn (turn 1 is the first turn) of its first Event::ProductionCompleted for Soldier and Settler, and its second Event::CityFounded. Starting units and captured units do not count as production; purchases count only when production actually completes. City captures do not count as founding, and losing a city does not reset the founding count. Null means never reached by the end of observed play. Distributions pool reached empires only, using the report's existing percentile convention; reached and never counts include eliminated empires and safety-cap stops. Cap stops are censored, not proof a milestone could never happen. Only first plays count; panics and legacy records without opening telemetry are excluded, not never observations.

Event totals and incidence use first plays with telemetry, including runner-stopped games; panics and legacy records without telemetry are excluded, not zeros. Terminal score populations use finished games only. Snowball cohorts use finished games with a turn-50 snapshot and at least one team; unique and tied leaders are separate. Draws stay in each eligible denominator and count as no leader win. Unfinished games have no eventual result and are excluded. Empty denominators are reported as 0/0 (no rate).

Definitions reuse [balance-probe-1](../analysis/balance-probe-1.md) and [the probe's definitions](../../tools/analysis/README.md); this report is a fresh simulation sample, not a rerun of those campaigns.

## Per-game events

Each row is one observed first play; unavailable telemetry is shown as n/a. Full per-player/team score sources and turn-50 standings are in JSON.

| Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 700001 | 285 | 52 | 0 | 0 | 0 |
| 1 | 700002 | 288 | 250 | 2 | 3 | 0 |
| 2 | 700003 | 294 | 261 | 6 | 3 | 0 |
| 3 | 700004 | 309 | 373 | 5 | 14 | 1 |
| 4 | 700005 | 320 | 247 | 2 | 2 | 0 |
| 5 | 700006 | 312 | 177 | 2 | 3 | 0 |
| 6 | 700007 | 236 | 41 | 0 | 0 | 0 |
| 7 | 700008 | 307 | 109 | 1 | 0 | 0 |
| 8 | 700009 | 281 | 122 | 3 | 1 | 1 |
| 9 | 700010 | 349 | 214 | 2 | 4 | 1 |
| 10 | 700011 | 295 | 248 | 3 | 2 | 1 |
| 11 | 700012 | 316 | 308 | 3 | 3 | 0 |
| 12 | 700013 | 362 | 191 | 5 | 2 | 1 |
| 13 | 700014 | 311 | 404 | 9 | 11 | 1 |
| 14 | 700015 | 267 | 163 | 2 | 3 | 0 |
| 15 | 700016 | 269 | 147 | 2 | 1 | 0 |
| 16 | 700017 | 308 | 212 | 3 | 4 | 0 |
| 17 | 700018 | 285 | 224 | 3 | 5 | 0 |
| 18 | 700019 | 286 | 168 | 3 | 1 | 0 |
| 19 | 700020 | 324 | 140 | 1 | 0 | 0 |
| 20 | 700021 | 344 | 413 | 3 | 4 | 0 |
| 21 | 700022 | 307 | 196 | 0 | 0 | 0 |
| 22 | 700023 | 247 | 75 | 0 | 5 | 0 |
| 23 | 700024 | 321 | 332 | 0 | 7 | 0 |
| 24 | 700025 | 295 | 114 | 2 | 7 | 0 |
| 25 | 700026 | 321 | 241 | 3 | 4 | 0 |
| 26 | 700027 | 300 | 329 | 3 | 6 | 1 |
| 27 | 700028 | 279 | 131 | 2 | 5 | 0 |
| 28 | 700029 | 368 | 417 | 3 | 4 | 0 |
| 29 | 700030 | 337 | 278 | 5 | 14 | 0 |
| 30 | 700031 | 279 | 133 | 0 | 1 | 0 |
| 31 | 700032 | 412 | 350 | 5 | 2 | 0 |
| 32 | 700033 | 284 | 168 | 2 | 4 | 0 |
| 33 | 700034 | 278 | 107 | 2 | 2 | 0 |
| 34 | 700035 | 292 | 94 | 2 | 2 | 0 |
| 35 | 700036 | 281 | 60 | 1 | 0 | 0 |
| 36 | 700037 | 205 | 81 | 1 | 3 | 0 |
| 37 | 700038 | 420 | 403 | 4 | 1 | 0 |
| 38 | 700039 | 270 | 230 | 1 | 1 | 0 |
| 39 | 700040 | 450 | 122 | 0 | 2 | 0 |
| 40 | 700041 | 295 | 267 | 2 | 3 | 0 |
| 41 | 700042 | 290 | 339 | 2 | 4 | 0 |
| 42 | 700043 | 325 | 461 | 8 | 4 | 1 |
| 43 | 700044 | 269 | 107 | 2 | 3 | 0 |
| 44 | 700045 | 292 | 120 | 0 | 5 | 0 |
| 45 | 700046 | 297 | 187 | 1 | 4 | 0 |
| 46 | 700047 | 269 | 164 | 1 | 8 | 0 |
| 47 | 700048 | 278 | 158 | 0 | 4 | 0 |
| 48 | 700049 | 331 | 346 | 7 | 3 | 0 |
| 49 | 700050 | 224 | 72 | 1 | 0 | 0 |
| 50 | 700051 | 319 | 179 | 0 | 0 | 0 |
| 51 | 700052 | 279 | 177 | 3 | 4 | 0 |
| 52 | 700053 | 262 | 48 | 0 | 2 | 0 |
| 53 | 700054 | 226 | 41 | 1 | 4 | 0 |
| 54 | 700055 | 293 | 146 | 2 | 3 | 0 |
| 55 | 700056 | 249 | 89 | 1 | 1 | 0 |
| 56 | 700057 | 348 | 435 | 3 | 11 | 1 |
| 57 | 700058 | 297 | 176 | 1 | 3 | 0 |
| 58 | 700059 | 344 | 338 | 5 | 8 | 0 |
| 59 | 700060 | 351 | 350 | 4 | 6 | 0 |
| 60 | 700061 | 230 | 49 | 1 | 1 | 0 |
| 61 | 700062 | 398 | 593 | 9 | 21 | 0 |
| 62 | 700063 | 301 | 206 | 1 | 2 | 0 |
| 63 | 700064 | 296 | 171 | 0 | 2 | 0 |
| 64 | 700065 | 309 | 357 | 3 | 4 | 0 |
| 65 | 700066 | 405 | 364 | 3 | 5 | 0 |
| 66 | 700067 | 267 | 157 | 2 | 0 | 0 |
| 67 | 700068 | 304 | 194 | 1 | 6 | 0 |
| 68 | 700069 | 281 | 131 | 0 | 1 | 0 |
| 69 | 700070 | 284 | 275 | 1 | 0 | 0 |
| 70 | 700071 | 305 | 266 | 2 | 2 | 0 |
| 71 | 700072 | 337 | 242 | 2 | 1 | 0 |
| 72 | 700073 | 313 | 181 | 0 | 1 | 0 |
| 73 | 700074 | 308 | 150 | 4 | 2 | 0 |
| 74 | 700075 | 294 | 211 | 0 | 4 | 0 |
| 75 | 700076 | 339 | 344 | 5 | 17 | 0 |
| 76 | 700077 | 307 | 186 | 2 | 2 | 0 |
| 77 | 700078 | 307 | 198 | 1 | 13 | 0 |
| 78 | 700079 | 287 | 124 | 1 | 0 | 0 |
| 79 | 700080 | 283 | 141 | 0 | 2 | 0 |
| 80 | 700081 | 314 | 412 | 2 | 1 | 0 |
| 81 | 700082 | 310 | 272 | 6 | 5 | 0 |
| 82 | 700083 | 268 | 133 | 1 | 2 | 0 |
| 83 | 700084 | 338 | 309 | 6 | 6 | 1 |
| 84 | 700085 | 349 | 242 | 5 | 3 | 0 |
| 85 | 700086 | 326 | 293 | 4 | 8 | 2 |
| 86 | 700087 | 305 | 189 | 4 | 5 | 1 |
| 87 | 700088 | 295 | 250 | 0 | 6 | 0 |
| 88 | 700089 | 291 | 244 | 4 | 5 | 1 |
| 89 | 700090 | 251 | 87 | 1 | 7 | 0 |
| 90 | 700091 | 255 | 108 | 2 | 6 | 0 |
| 91 | 700092 | 260 | 64 | 0 | 1 | 0 |
| 92 | 700093 | 299 | 162 | 2 | 6 | 0 |
| 93 | 700094 | 276 | 132 | 1 | 1 | 0 |
| 94 | 700095 | 293 | 233 | 2 | 2 | 0 |
| 95 | 700096 | 275 | 134 | 1 | 3 | 0 |
| 96 | 700097 | 301 | 111 | 1 | 1 | 0 |
| 97 | 700098 | 308 | 226 | 6 | 2 | 1 |
| 98 | 700099 | 289 | 185 | 2 | 1 | 0 |
| 99 | 700100 | 294 | 163 | 2 | 2 | 0 |
| 100 | 700101 | 288 | 201 | 5 | 8 | 1 |
| 101 | 700102 | 343 | 317 | 0 | 3 | 0 |
| 102 | 700103 | 307 | 165 | 4 | 5 | 0 |
| 103 | 700104 | 347 | 342 | 2 | 2 | 0 |
| 104 | 700105 | 250 | 20 | 0 | 1 | 0 |
| 105 | 700106 | 289 | 73 | 0 | 2 | 0 |
| 106 | 700107 | 272 | 179 | 1 | 5 | 0 |
| 107 | 700108 | 271 | 124 | 2 | 4 | 0 |
| 108 | 700109 | 447 | 663 | 6 | 4 | 1 |
| 109 | 700110 | 320 | 329 | 2 | 3 | 0 |
| 110 | 700111 | 275 | 109 | 2 | 1 | 0 |
| 111 | 700112 | 293 | 375 | 4 | 2 | 0 |
| 112 | 700113 | 377 | 562 | 5 | 13 | 0 |
| 113 | 700114 | 302 | 162 | 1 | 1 | 0 |
| 114 | 700115 | 307 | 123 | 3 | 3 | 0 |
| 115 | 700116 | 291 | 149 | 4 | 8 | 1 |
| 116 | 700117 | 283 | 263 | 3 | 2 | 0 |
| 117 | 700118 | 278 | 142 | 1 | 0 | 0 |
| 118 | 700119 | 278 | 33 | 0 | 0 | 0 |
| 119 | 700120 | 292 | 92 | 1 | 0 | 0 |
| 120 | 700121 | 293 | 160 | 1 | 1 | 0 |
| 121 | 700122 | 326 | 247 | 2 | 4 | 0 |
| 122 | 700123 | 359 | 522 | 3 | 28 | 0 |
| 123 | 700124 | 295 | 243 | 0 | 3 | 0 |
| 124 | 700125 | 300 | 180 | 7 | 5 | 0 |
| 125 | 700126 | 326 | 213 | 2 | 6 | 0 |
| 126 | 700127 | 312 | 381 | 2 | 2 | 0 |
| 127 | 700128 | 365 | 330 | 2 | 4 | 0 |
| 128 | 700129 | 328 | 143 | 2 | 7 | 0 |
| 129 | 700130 | 252 | 129 | 1 | 5 | 0 |
| 130 | 700131 | 316 | 251 | 3 | 8 | 0 |
| 131 | 700132 | 354 | 479 | 4 | 8 | 0 |
| 132 | 700133 | 365 | 403 | 4 | 13 | 0 |
| 133 | 700134 | 330 | 306 | 2 | 4 | 0 |
| 134 | 700135 | 266 | 146 | 3 | 5 | 0 |
| 135 | 700136 | 350 | 347 | 6 | 3 | 1 |
| 136 | 700137 | 445 | 608 | 4 | 4 | 0 |
| 137 | 700138 | 246 | 95 | 0 | 0 | 0 |
| 138 | 700139 | 444 | 672 | 2 | 2 | 0 |
| 139 | 700140 | 310 | 322 | 3 | 6 | 1 |
| 140 | 700141 | 346 | 298 | 2 | 5 | 0 |
| 141 | 700142 | 321 | 243 | 2 | 6 | 0 |
| 142 | 700143 | 216 | 77 | 2 | 2 | 0 |
| 143 | 700144 | 299 | 236 | 2 | 9 | 0 |
| 144 | 700145 | 375 | 245 | 2 | 7 | 0 |
| 145 | 700146 | 351 | 345 | 7 | 3 | 1 |
| 146 | 700147 | 287 | 23 | 0 | 0 | 0 |
| 147 | 700148 | 299 | 246 | 2 | 2 | 0 |
| 148 | 700149 | 299 | 87 | 0 | 0 | 0 |
| 149 | 700150 | 273 | 109 | 0 | 2 | 0 |
| 150 | 700151 | 276 | 255 | 2 | 7 | 0 |
| 151 | 700152 | 357 | 455 | 5 | 4 | 1 |
| 152 | 700153 | 222 | 46 | 0 | 0 | 0 |
| 153 | 700154 | 244 | 62 | 0 | 3 | 0 |
| 154 | 700155 | 293 | 175 | 1 | 4 | 0 |
| 155 | 700156 | 340 | 639 | 6 | 15 | 0 |
| 156 | 700157 | 271 | 124 | 1 | 2 | 0 |
| 157 | 700158 | 286 | 248 | 1 | 4 | 0 |
| 158 | 700159 | 306 | 467 | 3 | 4 | 0 |
| 159 | 700160 | 289 | 193 | 3 | 2 | 0 |
| 160 | 700161 | 218 | 66 | 2 | 1 | 0 |
| 161 | 700162 | 479 | 369 | 1 | 6 | 0 |
| 162 | 700163 | 326 | 322 | 3 | 7 | 0 |
| 163 | 700164 | 151 | 18 | 1 | 1 | 0 |
| 164 | 700165 | 285 | 124 | 2 | 2 | 0 |
| 165 | 700166 | 317 | 273 | 3 | 2 | 0 |
| 166 | 700167 | 297 | 190 | 2 | 9 | 0 |
| 167 | 700168 | 329 | 251 | 4 | 1 | 0 |
| 168 | 700169 | 317 | 257 | 3 | 7 | 0 |
| 169 | 700170 | 293 | 257 | 2 | 6 | 0 |
| 170 | 700171 | 311 | 237 | 2 | 5 | 1 |
| 171 | 700172 | 295 | 203 | 1 | 2 | 0 |
| 172 | 700173 | 325 | 208 | 2 | 5 | 0 |
| 173 | 700174 | 294 | 139 | 3 | 3 | 0 |
| 174 | 700175 | 334 | 457 | 5 | 5 | 0 |
| 175 | 700176 | 238 | 98 | 2 | 1 | 0 |
| 176 | 700177 | 284 | 278 | 4 | 9 | 0 |
| 177 | 700178 | 287 | 98 | 2 | 2 | 0 |
| 178 | 700179 | 330 | 310 | 1 | 2 | 0 |
| 179 | 700180 | 281 | 124 | 0 | 2 | 0 |
| 180 | 700181 | 334 | 497 | 5 | 4 | 1 |
| 181 | 700182 | 396 | 475 | 2 | 5 | 0 |
| 182 | 700183 | 341 | 212 | 0 | 3 | 0 |
| 183 | 700184 | 270 | 44 | 0 | 0 | 0 |
| 184 | 700185 | 320 | 158 | 1 | 5 | 0 |
| 185 | 700186 | 269 | 50 | 0 | 0 | 0 |
| 186 | 700187 | 278 | 165 | 1 | 4 | 0 |
| 187 | 700188 | 392 | 368 | 4 | 0 | 0 |
| 188 | 700189 | 221 | 61 | 2 | 0 | 0 |
| 189 | 700190 | 287 | 190 | 2 | 3 | 0 |
| 190 | 700191 | 254 | 39 | 1 | 9 | 0 |
| 191 | 700192 | 299 | 120 | 2 | 7 | 0 |
| 192 | 700193 | 296 | 179 | 0 | 2 | 0 |
| 193 | 700194 | 316 | 243 | 3 | 2 | 1 |
| 194 | 700195 | 334 | 321 | 5 | 7 | 0 |
| 195 | 700196 | 309 | 152 | 1 | 7 | 0 |
| 196 | 700197 | 295 | 179 | 1 | 3 | 0 |
| 197 | 700198 | 311 | 211 | 4 | 9 | 0 |
| 198 | 700199 | 276 | 129 | 2 | 0 | 0 |
| 199 | 700200 | 321 | 113 | 4 | 4 | 0 |
| 200 | 700201 | 270 | 140 | 0 | 6 | 0 |
| 201 | 700202 | 284 | 158 | 3 | 1 | 0 |
| 202 | 700203 | 281 | 120 | 1 | 1 | 0 |
| 203 | 700204 | 266 | 152 | 4 | 5 | 1 |
| 204 | 700205 | 342 | 471 | 5 | 3 | 0 |
| 205 | 700206 | 359 | 615 | 2 | 30 | 0 |
| 206 | 700207 | 311 | 87 | 1 | 3 | 0 |
| 207 | 700208 | 279 | 157 | 0 | 3 | 0 |
| 208 | 700209 | 299 | 291 | 6 | 4 | 1 |
| 209 | 700210 | 376 | 346 | 8 | 5 | 1 |
| 210 | 700211 | 282 | 279 | 2 | 2 | 0 |
| 211 | 700212 | 291 | 134 | 1 | 2 | 0 |
| 212 | 700213 | 266 | 190 | 1 | 6 | 0 |
| 213 | 700214 | 313 | 136 | 1 | 2 | 0 |
| 214 | 700215 | 284 | 180 | 2 | 1 | 0 |
| 215 | 700216 | 252 | 89 | 0 | 0 | 0 |
| 216 | 700217 | 319 | 295 | 3 | 5 | 0 |
| 217 | 700218 | 248 | 129 | 0 | 4 | 0 |
| 218 | 700219 | 212 | 5 | 0 | 1 | 0 |
| 219 | 700220 | 350 | 392 | 4 | 4 | 0 |
| 220 | 700221 | 242 | 210 | 1 | 0 | 0 |
| 221 | 700222 | 314 | 204 | 3 | 1 | 0 |
| 222 | 700223 | 283 | 120 | 2 | 1 | 0 |
| 223 | 700224 | 301 | 222 | 3 | 3 | 1 |
| 224 | 700225 | 300 | 216 | 1 | 4 | 1 |
| 225 | 700226 | 252 | 60 | 0 | 2 | 0 |
| 226 | 700227 | 343 | 224 | 1 | 1 | 0 |
| 227 | 700228 | 318 | 194 | 2 | 0 | 0 |
| 228 | 700229 | 233 | 68 | 2 | 1 | 0 |
| 229 | 700230 | 330 | 390 | 5 | 18 | 1 |
| 230 | 700231 | 242 | 174 | 1 | 1 | 0 |
| 231 | 700232 | 295 | 138 | 0 | 2 | 0 |
| 232 | 700233 | 297 | 138 | 3 | 2 | 0 |
| 233 | 700234 | 319 | 314 | 3 | 1 | 0 |
| 234 | 700235 | 313 | 197 | 3 | 2 | 0 |
| 235 | 700236 | 315 | 195 | 2 | 4 | 0 |
| 236 | 700237 | 310 | 219 | 0 | 1 | 0 |
| 237 | 700238 | 306 | 252 | 6 | 7 | 1 |
| 238 | 700239 | 300 | 251 | 1 | 11 | 0 |
| 239 | 700240 | 247 | 58 | 1 | 1 | 0 |
| 240 | 700241 | 303 | 241 | 3 | 10 | 0 |
| 241 | 700242 | 295 | 79 | 0 | 0 | 0 |
| 242 | 700243 | 341 | 257 | 2 | 7 | 0 |
| 243 | 700244 | 302 | 93 | 1 | 4 | 0 |
| 244 | 700245 | 237 | 91 | 0 | 1 | 0 |
| 245 | 700246 | 321 | 246 | 2 | 7 | 0 |
| 246 | 700247 | 292 | 100 | 0 | 0 | 0 |
| 247 | 700248 | 320 | 296 | 4 | 9 | 0 |
| 248 | 700249 | 294 | 216 | 2 | 5 | 0 |
| 249 | 700250 | 312 | 207 | 1 | 9 | 0 |
| 250 | 700251 | 320 | 275 | 4 | 14 | 0 |
| 251 | 700252 | 282 | 213 | 1 | 0 | 0 |
| 252 | 700253 | 331 | 161 | 1 | 3 | 0 |
| 253 | 700254 | 276 | 274 | 0 | 2 | 0 |
| 254 | 700255 | 319 | 308 | 4 | 3 | 0 |
| 255 | 700256 | 283 | 206 | 3 | 8 | 0 |
| 256 | 700257 | 261 | 136 | 3 | 0 | 0 |
| 257 | 700258 | 299 | 240 | 3 | 2 | 0 |
| 258 | 700259 | 307 | 166 | 1 | 1 | 0 |
| 259 | 700260 | 290 | 69 | 2 | 3 | 0 |
| 260 | 700261 | 407 | 699 | 5 | 15 | 0 |
| 261 | 700262 | 308 | 215 | 1 | 4 | 0 |
| 262 | 700263 | 372 | 328 | 1 | 8 | 0 |
| 263 | 700264 | 301 | 181 | 1 | 2 | 0 |
| 264 | 700265 | 244 | 126 | 3 | 4 | 0 |
| 265 | 700266 | 312 | 207 | 2 | 11 | 0 |
| 266 | 700267 | 317 | 279 | 2 | 4 | 0 |
| 267 | 700268 | 346 | 285 | 6 | 5 | 1 |
| 268 | 700269 | 323 | 275 | 4 | 2 | 0 |
| 269 | 700270 | 297 | 230 | 3 | 10 | 0 |
| 270 | 700271 | 375 | 469 | 10 | 6 | 0 |
| 271 | 700272 | 326 | 288 | 3 | 4 | 0 |
| 272 | 700273 | 326 | 366 | 2 | 3 | 0 |
| 273 | 700274 | 318 | 486 | 2 | 3 | 0 |
| 274 | 700275 | 251 | 117 | 2 | 1 | 0 |
| 275 | 700276 | 223 | 36 | 0 | 1 | 0 |
| 276 | 700277 | 251 | 82 | 1 | 3 | 0 |
| 277 | 700278 | 194 | 37 | 1 | 0 | 0 |
| 278 | 700279 | 287 | 279 | 8 | 5 | 1 |
| 279 | 700280 | 311 | 88 | 2 | 5 | 0 |
| 280 | 700281 | 349 | 317 | 3 | 1 | 0 |
| 281 | 700282 | 362 | 225 | 4 | 5 | 0 |
| 282 | 700283 | 316 | 304 | 7 | 7 | 1 |
| 283 | 700284 | 344 | 353 | 5 | 5 | 0 |
| 284 | 700285 | 301 | 150 | 1 | 1 | 0 |
| 285 | 700286 | 421 | 605 | 7 | 15 | 1 |
| 286 | 700287 | 328 | 337 | 5 | 6 | 0 |
| 287 | 700288 | 321 | 361 | 3 | 23 | 0 |
| 288 | 700289 | 344 | 346 | 5 | 9 | 0 |
| 289 | 700290 | 200 | 71 | 1 | 0 | 0 |
| 290 | 700291 | 252 | 53 | 1 | 0 | 0 |
| 291 | 700292 | 304 | 105 | 1 | 3 | 0 |
| 292 | 700293 | 324 | 368 | 3 | 3 | 0 |
| 293 | 700294 | 281 | 141 | 0 | 2 | 0 |
| 294 | 700295 | 284 | 157 | 1 | 4 | 0 |
| 295 | 700296 | 282 | 144 | 3 | 4 | 1 |
| 296 | 700297 | 294 | 88 | 0 | 3 | 0 |
| 297 | 700298 | 276 | 179 | 0 | 6 | 0 |
| 298 | 700299 | 329 | 235 | 3 | 3 | 0 |
| 299 | 700300 | 308 | 226 | 4 | 2 | 0 |
| 300 | 700301 | 213 | 9 | 0 | 1 | 0 |
| 301 | 700302 | 299 | 122 | 2 | 1 | 0 |
| 302 | 700303 | 203 | 47 | 1 | 3 | 0 |
| 303 | 700304 | 278 | 151 | 1 | 2 | 0 |
| 304 | 700305 | 307 | 196 | 6 | 0 | 1 |
| 305 | 700306 | 388 | 440 | 3 | 3 | 0 |
| 306 | 700307 | 345 | 182 | 3 | 0 | 0 |
| 307 | 700308 | 298 | 142 | 4 | 2 | 0 |
| 308 | 700309 | 333 | 333 | 5 | 6 | 0 |
| 309 | 700310 | 482 | 673 | 8 | 3 | 1 |
| 310 | 700311 | 335 | 327 | 2 | 7 | 0 |
| 311 | 700312 | 280 | 166 | 2 | 1 | 0 |
| 312 | 700313 | 199 | 44 | 0 | 3 | 0 |
| 313 | 700314 | 274 | 143 | 3 | 3 | 0 |
| 314 | 700315 | 271 | 88 | 1 | 6 | 0 |
| 315 | 700316 | 330 | 304 | 4 | 8 | 0 |
| 316 | 700317 | 277 | 146 | 2 | 1 | 0 |
| 317 | 700318 | 204 | 104 | 1 | 0 | 0 |
| 318 | 700319 | 301 | 149 | 1 | 1 | 0 |
| 319 | 700320 | 288 | 71 | 1 | 2 | 0 |
| 320 | 700321 | 338 | 363 | 7 | 5 | 0 |
| 321 | 700322 | 294 | 201 | 3 | 1 | 0 |
| 322 | 700323 | 276 | 129 | 2 | 2 | 0 |
| 323 | 700324 | 270 | 153 | 2 | 1 | 0 |
| 324 | 700325 | 325 | 298 | 5 | 1 | 0 |
| 325 | 700326 | 288 | 101 | 1 | 3 | 0 |
| 326 | 700327 | 274 | 130 | 0 | 2 | 0 |
| 327 | 700328 | 248 | 130 | 1 | 0 | 0 |
| 328 | 700329 | 369 | 239 | 1 | 4 | 0 |
| 329 | 700330 | 298 | 195 | 5 | 5 | 1 |
| 330 | 700331 | 317 | 125 | 1 | 8 | 0 |
| 331 | 700332 | 276 | 294 | 2 | 1 | 0 |
| 332 | 700333 | 238 | 82 | 1 | 2 | 0 |
| 333 | 700334 | 400 | 569 | 8 | 10 | 0 |
| 334 | 700335 | 307 | 222 | 3 | 5 | 1 |
| 335 | 700336 | 348 | 363 | 3 | 3 | 0 |
| 336 | 700337 | 318 | 136 | 1 | 5 | 0 |
| 337 | 700338 | 237 | 125 | 1 | 4 | 0 |
| 338 | 700339 | 387 | 580 | 13 | 14 | 0 |
| 339 | 700340 | 243 | 31 | 0 | 2 | 0 |
| 340 | 700341 | 361 | 244 | 2 | 3 | 0 |
| 341 | 700342 | 311 | 131 | 1 | 1 | 0 |
| 342 | 700343 | 233 | 21 | 0 | 0 | 0 |
| 343 | 700344 | 246 | 130 | 2 | 2 | 0 |
| 344 | 700345 | 294 | 140 | 4 | 9 | 0 |
| 345 | 700346 | 396 | 284 | 5 | 5 | 0 |
| 346 | 700347 | 212 | 57 | 0 | 3 | 0 |
| 347 | 700348 | 300 | 223 | 4 | 5 | 1 |
| 348 | 700349 | 201 | 77 | 1 | 1 | 0 |
| 349 | 700350 | 273 | 164 | 3 | 11 | 0 |
| 350 | 700351 | 241 | 59 | 0 | 1 | 0 |
| 351 | 700352 | 261 | 125 | 1 | 6 | 0 |
| 352 | 700353 | 283 | 170 | 0 | 4 | 0 |
| 353 | 700354 | 220 | 37 | 1 | 2 | 0 |
| 354 | 700355 | 283 | 158 | 1 | 2 | 0 |
| 355 | 700356 | 349 | 281 | 2 | 0 | 0 |
| 356 | 700357 | 300 | 231 | 4 | 7 | 1 |
| 357 | 700358 | 258 | 51 | 0 | 3 | 0 |
| 358 | 700359 | 357 | 375 | 4 | 8 | 0 |
| 359 | 700360 | 275 | 169 | 3 | 4 | 1 |
| 360 | 700361 | 312 | 252 | 4 | 3 | 0 |
| 361 | 700362 | 290 | 159 | 5 | 6 | 0 |
| 362 | 700363 | 261 | 152 | 2 | 2 | 1 |
| 363 | 700364 | 272 | 69 | 1 | 0 | 0 |
| 364 | 700365 | 307 | 147 | 0 | 5 | 0 |
| 365 | 700366 | 318 | 237 | 1 | 2 | 0 |
| 366 | 700367 | 335 | 369 | 6 | 6 | 0 |
| 367 | 700368 | 307 | 180 | 1 | 4 | 0 |
| 368 | 700369 | 357 | 450 | 4 | 7 | 0 |
| 369 | 700370 | 267 | 64 | 1 | 7 | 0 |
| 370 | 700371 | 303 | 272 | 2 | 9 | 0 |
| 371 | 700372 | 295 | 128 | 2 | 2 | 0 |
| 372 | 700373 | 329 | 280 | 5 | 3 | 1 |
| 373 | 700374 | 252 | 142 | 1 | 2 | 0 |
| 374 | 700375 | 373 | 581 | 7 | 11 | 0 |
| 375 | 700376 | 311 | 124 | 2 | 3 | 0 |
| 376 | 700377 | 317 | 186 | 3 | 1 | 0 |
| 377 | 700378 | 268 | 146 | 0 | 4 | 0 |
| 378 | 700379 | 357 | 442 | 1 | 2 | 0 |
| 379 | 700380 | 385 | 363 | 4 | 2 | 0 |
| 380 | 700381 | 250 | 111 | 2 | 2 | 0 |
| 381 | 700382 | 278 | 34 | 0 | 1 | 0 |
| 382 | 700383 | 260 | 66 | 0 | 0 | 0 |
| 383 | 700384 | 310 | 74 | 1 | 1 | 0 |
| 384 | 700385 | 258 | 185 | 2 | 6 | 0 |
| 385 | 700386 | 288 | 97 | 1 | 2 | 0 |
| 386 | 700387 | 290 | 133 | 2 | 1 | 0 |
| 387 | 700388 | 287 | 229 | 0 | 1 | 0 |
| 388 | 700389 | 318 | 252 | 0 | 6 | 0 |
| 389 | 700390 | 215 | 22 | 0 | 0 | 0 |
| 390 | 700391 | 263 | 58 | 1 | 8 | 0 |
| 391 | 700392 | 329 | 401 | 4 | 3 | 0 |
| 392 | 700393 | 316 | 232 | 1 | 16 | 0 |
| 393 | 700394 | 232 | 86 | 1 | 0 | 0 |
| 394 | 700395 | 267 | 121 | 2 | 2 | 1 |
| 395 | 700396 | 299 | 207 | 2 | 8 | 1 |
| 396 | 700397 | 263 | 128 | 0 | 6 | 0 |
| 397 | 700398 | 156 | 17 | 1 | 0 | 0 |
| 398 | 700399 | 280 | 165 | 2 | 3 | 0 |
| 399 | 700400 | 331 | 309 | 3 | 1 | 0 |
| 400 | 700401 | 266 | 106 | 0 | 2 | 0 |
| 401 | 700402 | 211 | 72 | 0 | 2 | 0 |
| 402 | 700403 | 279 | 81 | 0 | 0 | 0 |
| 403 | 700404 | 242 | 118 | 0 | 0 | 0 |
| 404 | 700405 | 287 | 58 | 0 | 6 | 0 |
| 405 | 700406 | 359 | 495 | 4 | 10 | 0 |
| 406 | 700407 | 327 | 288 | 4 | 13 | 0 |
| 407 | 700408 | 307 | 232 | 2 | 14 | 0 |
| 408 | 700409 | 315 | 248 | 5 | 5 | 1 |
| 409 | 700410 | 304 | 194 | 2 | 2 | 0 |
| 410 | 700411 | 256 | 168 | 4 | 7 | 0 |
| 411 | 700412 | 320 | 249 | 4 | 8 | 0 |
| 412 | 700413 | 179 | 35 | 1 | 1 | 0 |
| 413 | 700414 | 301 | 263 | 5 | 7 | 0 |
| 414 | 700415 | 299 | 151 | 2 | 3 | 0 |
| 415 | 700416 | 292 | 221 | 3 | 5 | 0 |
| 416 | 700417 | 279 | 128 | 2 | 3 | 0 |
| 417 | 700418 | 275 | 89 | 1 | 1 | 0 |
| 418 | 700419 | 247 | 160 | 2 | 10 | 0 |
| 419 | 700420 | 312 | 191 | 4 | 9 | 0 |
| 420 | 700421 | 278 | 218 | 1 | 3 | 0 |
| 421 | 700422 | 287 | 224 | 3 | 0 | 0 |
| 422 | 700423 | 363 | 316 | 3 | 2 | 0 |
| 423 | 700424 | 165 | 27 | 1 | 1 | 0 |
| 424 | 700425 | 241 | 14 | 0 | 1 | 0 |
| 425 | 700426 | 349 | 253 | 5 | 4 | 0 |
| 426 | 700427 | 308 | 156 | 4 | 10 | 1 |
| 427 | 700428 | 287 | 179 | 3 | 2 | 0 |
| 428 | 700429 | 290 | 100 | 4 | 1 | 0 |
| 429 | 700430 | 305 | 271 | 2 | 1 | 0 |
| 430 | 700431 | 287 | 146 | 3 | 2 | 0 |
| 431 | 700432 | 270 | 110 | 2 | 4 | 0 |
| 432 | 700433 | 254 | 139 | 0 | 2 | 0 |
| 433 | 700434 | 366 | 358 | 4 | 1 | 0 |
| 434 | 700435 | 266 | 79 | 1 | 4 | 0 |
| 435 | 700436 | 314 | 187 | 0 | 4 | 0 |
| 436 | 700437 | 280 | 96 | 0 | 2 | 0 |
| 437 | 700438 | 340 | 300 | 1 | 5 | 0 |
| 438 | 700439 | 328 | 355 | 5 | 10 | 0 |
| 439 | 700440 | 386 | 872 | 6 | 19 | 0 |
| 440 | 700441 | 270 | 0 | 0 | 0 | 0 |
| 441 | 700442 | 287 | 179 | 2 | 3 | 0 |
| 442 | 700443 | 277 | 107 | 0 | 6 | 0 |
| 443 | 700444 | 298 | 217 | 0 | 8 | 0 |
| 444 | 700445 | 325 | 270 | 3 | 8 | 0 |
| 445 | 700446 | 296 | 175 | 2 | 10 | 0 |
| 446 | 700447 | 304 | 289 | 7 | 8 | 0 |
| 447 | 700448 | 395 | 570 | 3 | 19 | 0 |
| 448 | 700449 | 316 | 158 | 1 | 4 | 0 |
| 449 | 700450 | 288 | 211 | 0 | 1 | 0 |
| 450 | 700451 | 213 | 24 | 0 | 0 | 0 |
| 451 | 700452 | 253 | 123 | 3 | 1 | 0 |
| 452 | 700453 | 353 | 280 | 6 | 10 | 1 |
| 453 | 700454 | 339 | 224 | 6 | 7 | 0 |
| 454 | 700455 | 332 | 287 | 2 | 3 | 0 |
| 455 | 700456 | 291 | 222 | 3 | 12 | 0 |
| 456 | 700457 | 306 | 168 | 1 | 3 | 0 |
| 457 | 700458 | 302 | 143 | 3 | 1 | 0 |
| 458 | 700459 | 327 | 210 | 2 | 6 | 0 |
| 459 | 700460 | 282 | 159 | 2 | 2 | 0 |
| 460 | 700461 | 286 | 182 | 0 | 9 | 0 |
| 461 | 700462 | 289 | 127 | 0 | 1 | 0 |
| 462 | 700463 | 285 | 205 | 2 | 0 | 0 |
| 463 | 700464 | 259 | 116 | 1 | 6 | 0 |
| 464 | 700465 | 306 | 206 | 0 | 1 | 0 |
| 465 | 700466 | 287 | 161 | 1 | 2 | 0 |
| 466 | 700467 | 299 | 154 | 3 | 13 | 0 |
| 467 | 700468 | 341 | 223 | 2 | 3 | 0 |
| 468 | 700469 | 272 | 159 | 2 | 4 | 1 |
| 469 | 700470 | 243 | 134 | 2 | 4 | 0 |
| 470 | 700471 | 317 | 96 | 3 | 7 | 0 |
| 471 | 700472 | 253 | 14 | 0 | 0 | 0 |
| 472 | 700473 | 399 | 544 | 4 | 6 | 1 |
| 473 | 700474 | 225 | 38 | 0 | 0 | 0 |
| 474 | 700475 | 329 | 60 | 1 | 0 | 0 |
| 475 | 700476 | 334 | 333 | 6 | 2 | 0 |
| 476 | 700477 | 230 | 126 | 4 | 0 | 0 |
| 477 | 700478 | 286 | 234 | 3 | 0 | 0 |
| 478 | 700479 | 372 | 356 | 4 | 6 | 0 |
| 479 | 700480 | 296 | 219 | 1 | 8 | 1 |
| 480 | 700481 | 322 | 235 | 3 | 7 | 0 |
| 481 | 700482 | 251 | 52 | 2 | 5 | 0 |
| 482 | 700483 | 294 | 285 | 0 | 1 | 0 |
| 483 | 700484 | 265 | 95 | 1 | 3 | 0 |
| 484 | 700485 | 290 | 24 | 1 | 1 | 0 |
| 485 | 700486 | 292 | 139 | 4 | 1 | 0 |
| 486 | 700487 | 342 | 280 | 4 | 7 | 0 |
| 487 | 700488 | 300 | 196 | 3 | 5 | 1 |
| 488 | 700489 | 295 | 218 | 1 | 11 | 0 |
| 489 | 700490 | 255 | 84 | 0 | 1 | 0 |
| 490 | 700491 | 316 | 262 | 3 | 2 | 0 |
| 491 | 700492 | 303 | 163 | 0 | 0 | 0 |
| 492 | 700493 | 307 | 234 | 3 | 1 | 0 |
| 493 | 700494 | 284 | 325 | 2 | 5 | 1 |
| 494 | 700495 | 270 | 203 | 1 | 0 | 0 |
| 495 | 700496 | 352 | 395 | 2 | 1 | 0 |
| 496 | 700497 | 276 | 134 | 1 | 5 | 0 |
| 497 | 700498 | 329 | 362 | 5 | 4 | 1 |
| 498 | 700499 | 289 | 210 | 0 | 3 | 0 |
| 499 | 700500 | 273 | 177 | 2 | 0 | 0 |
| 500 | 700501 | 296 | 124 | 0 | 0 | 0 |
| 501 | 700502 | 486 | 866 | 5 | 4 | 0 |
| 502 | 700503 | 214 | 115 | 2 | 4 | 0 |
| 503 | 700504 | 288 | 63 | 0 | 2 | 0 |
| 504 | 700505 | 338 | 294 | 2 | 6 | 0 |
| 505 | 700506 | 338 | 292 | 1 | 0 | 0 |
| 506 | 700507 | 292 | 320 | 3 | 11 | 0 |
| 507 | 700508 | 288 | 12 | 0 | 0 | 0 |
| 508 | 700509 | 335 | 242 | 5 | 5 | 0 |
| 509 | 700510 | 314 | 271 | 4 | 6 | 1 |
| 510 | 700511 | 316 | 324 | 2 | 6 | 0 |
| 511 | 700512 | 253 | 101 | 1 | 5 | 0 |
| 512 | 700513 | 332 | 287 | 3 | 2 | 0 |
| 513 | 700514 | 339 | 279 | 3 | 2 | 0 |
| 514 | 700515 | 168 | 44 | 1 | 4 | 0 |
| 515 | 700516 | 208 | 49 | 2 | 0 | 0 |
| 516 | 700517 | 262 | 63 | 1 | 2 | 0 |
| 517 | 700518 | 312 | 193 | 4 | 7 | 1 |
| 518 | 700519 | 331 | 295 | 1 | 7 | 0 |
| 519 | 700520 | 279 | 123 | 1 | 3 | 0 |
| 520 | 700521 | 294 | 105 | 1 | 3 | 0 |
| 521 | 700522 | 325 | 279 | 5 | 6 | 1 |
| 522 | 700523 | 457 | 868 | 6 | 9 | 0 |
| 523 | 700524 | 317 | 293 | 1 | 4 | 0 |
| 524 | 700525 | 319 | 192 | 4 | 1 | 0 |
| 525 | 700526 | 292 | 318 | 5 | 4 | 0 |
| 526 | 700527 | 324 | 259 | 5 | 11 | 1 |
| 527 | 700528 | 282 | 140 | 2 | 0 | 0 |
| 528 | 700529 | 325 | 259 | 2 | 3 | 0 |
| 529 | 700530 | 297 | 164 | 2 | 4 | 0 |
| 530 | 700531 | 340 | 223 | 3 | 0 | 0 |
| 531 | 700532 | 394 | 617 | 5 | 10 | 1 |
| 532 | 700533 | 205 | 119 | 1 | 1 | 0 |
| 533 | 700534 | 349 | 442 | 2 | 7 | 0 |
| 534 | 700535 | 292 | 127 | 0 | 4 | 0 |
| 535 | 700536 | 243 | 107 | 0 | 1 | 0 |
| 536 | 700537 | 259 | 113 | 2 | 1 | 0 |
| 537 | 700538 | 285 | 151 | 1 | 3 | 0 |
| 538 | 700539 | 279 | 127 | 2 | 3 | 0 |
| 539 | 700540 | 307 | 205 | 2 | 1 | 0 |
| 540 | 700541 | 320 | 208 | 0 | 5 | 0 |
| 541 | 700542 | 263 | 46 | 1 | 3 | 0 |
| 542 | 700543 | 310 | 83 | 2 | 11 | 0 |
| 543 | 700544 | 274 | 86 | 2 | 4 | 0 |
| 544 | 700545 | 230 | 63 | 0 | 3 | 0 |
| 545 | 700546 | 257 | 104 | 1 | 2 | 0 |
| 546 | 700547 | 443 | 825 | 13 | 4 | 1 |
| 547 | 700548 | 301 | 347 | 3 | 13 | 0 |
| 548 | 700549 | 264 | 125 | 2 | 1 | 0 |
| 549 | 700550 | 344 | 239 | 0 | 1 | 0 |
| 550 | 700551 | 277 | 156 | 0 | 1 | 0 |
| 551 | 700552 | 285 | 140 | 2 | 1 | 0 |
| 552 | 700553 | 326 | 370 | 3 | 0 | 0 |
| 553 | 700554 | 413 | 589 | 8 | 2 | 0 |
| 554 | 700555 | 285 | 149 | 2 | 4 | 1 |
| 555 | 700556 | 254 | 131 | 1 | 1 | 0 |
| 556 | 700557 | 381 | 411 | 4 | 8 | 1 |
| 557 | 700558 | 322 | 194 | 4 | 7 | 0 |
| 558 | 700559 | 281 | 118 | 0 | 5 | 0 |
| 559 | 700560 | 189 | 34 | 2 | 0 | 0 |
| 560 | 700561 | 290 | 155 | 5 | 3 | 1 |
| 561 | 700562 | 358 | 357 | 4 | 10 | 0 |
| 562 | 700563 | 327 | 351 | 2 | 1 | 0 |
| 563 | 700564 | 198 | 22 | 0 | 0 | 0 |
| 564 | 700565 | 322 | 301 | 4 | 5 | 0 |
| 565 | 700566 | 242 | 66 | 3 | 3 | 1 |
| 566 | 700567 | 332 | 355 | 0 | 2 | 0 |
| 567 | 700568 | 278 | 255 | 4 | 3 | 0 |
| 568 | 700569 | 346 | 377 | 2 | 5 | 0 |
| 569 | 700570 | 259 | 96 | 0 | 0 | 0 |
| 570 | 700571 | 303 | 374 | 4 | 10 | 0 |
| 571 | 700572 | 294 | 215 | 4 | 5 | 0 |
| 572 | 700573 | 297 | 290 | 3 | 3 | 0 |
| 573 | 700574 | 293 | 189 | 2 | 2 | 0 |
| 574 | 700575 | 326 | 328 | 5 | 9 | 0 |
| 575 | 700576 | 427 | 517 | 6 | 3 | 0 |
| 576 | 700577 | 429 | 231 | 0 | 0 | 0 |
| 577 | 700578 | 280 | 46 | 1 | 2 | 0 |
| 578 | 700579 | 284 | 121 | 0 | 2 | 0 |
| 579 | 700580 | 290 | 100 | 2 | 0 | 0 |
| 580 | 700581 | 313 | 359 | 2 | 0 | 0 |
| 581 | 700582 | 261 | 33 | 0 | 0 | 0 |
| 582 | 700583 | 311 | 68 | 1 | 2 | 0 |
| 583 | 700584 | 302 | 223 | 3 | 5 | 0 |
| 584 | 700585 | 330 | 364 | 2 | 3 | 0 |
| 585 | 700586 | 291 | 211 | 1 | 0 | 0 |
| 586 | 700587 | 316 | 259 | 5 | 8 | 0 |
| 587 | 700588 | 340 | 428 | 4 | 7 | 1 |
| 588 | 700589 | 367 | 458 | 5 | 9 | 0 |
| 589 | 700590 | 287 | 91 | 0 | 4 | 0 |
| 590 | 700591 | 329 | 169 | 2 | 4 | 0 |
| 591 | 700592 | 298 | 190 | 3 | 5 | 0 |
| 592 | 700593 | 327 | 207 | 2 | 3 | 0 |
| 593 | 700594 | 449 | 708 | 4 | 6 | 0 |
| 594 | 700595 | 419 | 491 | 1 | 12 | 0 |
| 595 | 700596 | 384 | 278 | 3 | 7 | 0 |
| 596 | 700597 | 307 | 260 | 4 | 6 | 0 |
| 597 | 700598 | 349 | 389 | 5 | 4 | 0 |
| 598 | 700599 | 322 | 257 | 2 | 4 | 0 |
| 599 | 700600 | 347 | 313 | 0 | 4 | 0 |
| 600 | 700601 | 305 | 203 | 4 | 0 | 0 |
| 601 | 700602 | 297 | 337 | 4 | 9 | 2 |
| 602 | 700603 | 357 | 378 | 1 | 4 | 0 |
| 603 | 700604 | 326 | 320 | 4 | 4 | 1 |
| 604 | 700605 | 284 | 180 | 2 | 9 | 0 |
| 605 | 700606 | 403 | 614 | 3 | 4 | 0 |
| 606 | 700607 | 311 | 206 | 2 | 3 | 0 |
| 607 | 700608 | 305 | 46 | 0 | 0 | 0 |
| 608 | 700609 | 302 | 223 | 5 | 3 | 0 |
| 609 | 700610 | 285 | 131 | 3 | 9 | 0 |
| 610 | 700611 | 351 | 366 | 3 | 12 | 1 |
| 611 | 700612 | 322 | 308 | 0 | 9 | 0 |
| 612 | 700613 | 291 | 227 | 2 | 4 | 0 |
| 613 | 700614 | 271 | 58 | 0 | 0 | 0 |
| 614 | 700615 | 312 | 188 | 5 | 11 | 0 |
| 615 | 700616 | 305 | 318 | 1 | 13 | 0 |
| 616 | 700617 | 437 | 784 | 7 | 10 | 1 |
| 617 | 700618 | 293 | 242 | 1 | 3 | 0 |
| 618 | 700619 | 322 | 357 | 4 | 6 | 0 |
| 619 | 700620 | 365 | 248 | 0 | 0 | 0 |
| 620 | 700621 | 404 | 530 | 3 | 2 | 0 |
| 621 | 700622 | 277 | 145 | 3 | 1 | 0 |
| 622 | 700623 | 309 | 298 | 2 | 5 | 0 |
| 623 | 700624 | 212 | 58 | 0 | 0 | 0 |
| 624 | 700625 | 316 | 142 | 1 | 2 | 0 |
| 625 | 700626 | 285 | 215 | 3 | 4 | 0 |
| 626 | 700627 | 285 | 294 | 3 | 1 | 0 |
| 627 | 700628 | 249 | 32 | 0 | 0 | 0 |
| 628 | 700629 | 327 | 244 | 1 | 8 | 0 |
| 629 | 700630 | 297 | 148 | 3 | 12 | 0 |
| 630 | 700631 | 246 | 114 | 2 | 5 | 0 |
| 631 | 700632 | 345 | 322 | 3 | 1 | 0 |
| 632 | 700633 | 243 | 53 | 1 | 1 | 0 |
| 633 | 700634 | 305 | 166 | 4 | 8 | 1 |
| 634 | 700635 | 287 | 220 | 2 | 18 | 0 |
| 635 | 700636 | 300 | 180 | 1 | 5 | 0 |
| 636 | 700637 | 298 | 206 | 3 | 8 | 0 |
| 637 | 700638 | 285 | 318 | 1 | 6 | 0 |
| 638 | 700639 | 319 | 312 | 5 | 5 | 1 |
| 639 | 700640 | 341 | 209 | 4 | 7 | 0 |
| 640 | 700641 | 292 | 183 | 3 | 8 | 0 |
| 641 | 700642 | 295 | 137 | 5 | 1 | 0 |
| 642 | 700643 | 360 | 505 | 7 | 7 | 1 |
| 643 | 700644 | 255 | 63 | 2 | 6 | 0 |
| 644 | 700645 | 292 | 182 | 1 | 7 | 0 |
| 645 | 700646 | 300 | 177 | 3 | 3 | 0 |
| 646 | 700647 | 378 | 473 | 4 | 4 | 0 |
| 647 | 700648 | 313 | 219 | 3 | 6 | 0 |
| 648 | 700649 | 299 | 86 | 2 | 1 | 0 |
| 649 | 700650 | 206 | 43 | 1 | 2 | 0 |
| 650 | 700651 | 298 | 100 | 1 | 5 | 0 |
| 651 | 700652 | 296 | 244 | 2 | 0 | 0 |
| 652 | 700653 | 289 | 175 | 3 | 3 | 1 |
| 653 | 700654 | 318 | 167 | 2 | 4 | 0 |
| 654 | 700655 | 287 | 150 | 3 | 7 | 0 |
| 655 | 700656 | 315 | 226 | 2 | 6 | 0 |
| 656 | 700657 | 300 | 291 | 2 | 1 | 0 |
| 657 | 700658 | 282 | 48 | 1 | 7 | 0 |
| 658 | 700659 | 288 | 231 | 3 | 2 | 0 |
| 659 | 700660 | 329 | 287 | 6 | 2 | 0 |
| 660 | 700661 | 323 | 394 | 3 | 1 | 0 |
| 661 | 700662 | 333 | 406 | 6 | 5 | 0 |
| 662 | 700663 | 328 | 324 | 3 | 5 | 0 |
| 663 | 700664 | 342 | 415 | 2 | 13 | 1 |
| 664 | 700665 | 316 | 376 | 1 | 13 | 0 |
| 665 | 700666 | 300 | 232 | 4 | 1 | 0 |
| 666 | 700667 | 280 | 29 | 1 | 1 | 0 |
| 667 | 700668 | 289 | 222 | 3 | 3 | 0 |
| 668 | 700669 | 280 | 224 | 6 | 6 | 1 |
| 669 | 700670 | 260 | 65 | 0 | 1 | 0 |
| 670 | 700671 | 294 | 219 | 2 | 4 | 0 |
| 671 | 700672 | 270 | 191 | 1 | 1 | 0 |
| 672 | 700673 | 437 | 387 | 0 | 7 | 0 |
| 673 | 700674 | 372 | 449 | 5 | 8 | 0 |
| 674 | 700675 | 310 | 228 | 1 | 2 | 0 |
| 675 | 700676 | 315 | 251 | 6 | 0 | 1 |
| 676 | 700677 | 252 | 119 | 3 | 8 | 0 |
| 677 | 700678 | 292 | 140 | 4 | 1 | 0 |
| 678 | 700679 | 310 | 224 | 2 | 6 | 1 |
| 679 | 700680 | 277 | 129 | 3 | 5 | 1 |
| 680 | 700681 | 283 | 161 | 2 | 10 | 0 |
| 681 | 700682 | 314 | 166 | 0 | 1 | 0 |
| 682 | 700683 | 283 | 56 | 0 | 6 | 0 |
| 683 | 700684 | 354 | 156 | 1 | 10 | 0 |
| 684 | 700685 | 281 | 187 | 2 | 1 | 0 |
| 685 | 700686 | 344 | 299 | 5 | 18 | 1 |
| 686 | 700687 | 221 | 18 | 0 | 0 | 0 |
| 687 | 700688 | 337 | 252 | 2 | 7 | 1 |
| 688 | 700689 | 334 | 248 | 0 | 1 | 0 |
| 689 | 700690 | 408 | 629 | 4 | 3 | 0 |
| 690 | 700691 | 293 | 189 | 2 | 5 | 0 |
| 691 | 700692 | 267 | 84 | 0 | 3 | 0 |
| 692 | 700693 | 307 | 283 | 3 | 3 | 0 |
| 693 | 700694 | 322 | 231 | 5 | 1 | 1 |
| 694 | 700695 | 361 | 273 | 2 | 10 | 1 |
| 695 | 700696 | 281 | 161 | 3 | 3 | 0 |
| 696 | 700697 | 279 | 115 | 3 | 3 | 0 |
| 697 | 700698 | 350 | 275 | 3 | 10 | 0 |
| 698 | 700699 | 293 | 262 | 3 | 6 | 1 |
| 699 | 700700 | 286 | 84 | 3 | 3 | 0 |
| 700 | 700701 | 306 | 242 | 1 | 1 | 0 |
| 701 | 700702 | 264 | 100 | 0 | 3 | 0 |
| 702 | 700703 | 303 | 240 | 2 | 3 | 1 |
| 703 | 700704 | 294 | 251 | 4 | 4 | 0 |
| 704 | 700705 | 492 | 544 | 5 | 5 | 0 |
| 705 | 700706 | 334 | 308 | 3 | 4 | 0 |
| 706 | 700707 | 310 | 124 | 4 | 3 | 1 |
| 707 | 700708 | 298 | 146 | 1 | 1 | 0 |
| 708 | 700709 | 276 | 263 | 1 | 2 | 0 |
| 709 | 700710 | 360 | 491 | 6 | 2 | 0 |
| 710 | 700711 | 309 | 202 | 4 | 5 | 0 |
| 711 | 700712 | 301 | 309 | 4 | 6 | 0 |
| 712 | 700713 | 267 | 159 | 1 | 9 | 1 |
| 713 | 700714 | 331 | 256 | 1 | 11 | 0 |
| 714 | 700715 | 281 | 154 | 1 | 6 | 0 |
| 715 | 700716 | 387 | 558 | 2 | 3 | 0 |
| 716 | 700717 | 330 | 217 | 1 | 4 | 0 |
| 717 | 700718 | 294 | 135 | 4 | 0 | 1 |
| 718 | 700719 | 285 | 155 | 2 | 7 | 0 |
| 719 | 700720 | 281 | 187 | 1 | 4 | 0 |
| 720 | 700721 | 276 | 171 | 1 | 7 | 0 |
| 721 | 700722 | 290 | 281 | 2 | 3 | 1 |
| 722 | 700723 | 290 | 360 | 2 | 27 | 1 |
| 723 | 700724 | 287 | 143 | 2 | 2 | 0 |
| 724 | 700725 | 286 | 155 | 1 | 4 | 0 |
| 725 | 700726 | 352 | 398 | 2 | 5 | 0 |
| 726 | 700727 | 244 | 37 | 0 | 2 | 0 |
| 727 | 700728 | 334 | 239 | 5 | 6 | 1 |
| 728 | 700729 | 327 | 204 | 2 | 5 | 0 |
| 729 | 700730 | 389 | 503 | 5 | 8 | 0 |
| 730 | 700731 | 317 | 340 | 3 | 3 | 0 |
| 731 | 700732 | 304 | 246 | 4 | 4 | 0 |
| 732 | 700733 | 346 | 518 | 4 | 12 | 1 |
| 733 | 700734 | 300 | 218 | 1 | 1 | 0 |
| 734 | 700735 | 215 | 52 | 1 | 3 | 0 |
| 735 | 700736 | 260 | 139 | 4 | 3 | 1 |
| 736 | 700737 | 257 | 127 | 2 | 0 | 0 |
| 737 | 700738 | 248 | 156 | 0 | 4 | 0 |
| 738 | 700739 | 241 | 63 | 1 | 0 | 0 |
| 739 | 700740 | 257 | 201 | 2 | 5 | 0 |
| 740 | 700741 | 297 | 114 | 2 | 1 | 0 |
| 741 | 700742 | 293 | 185 | 5 | 2 | 0 |
| 742 | 700743 | 313 | 47 | 2 | 2 | 0 |
| 743 | 700744 | 266 | 66 | 1 | 0 | 0 |
| 744 | 700745 | 333 | 402 | 3 | 9 | 0 |
| 745 | 700746 | 267 | 6 | 0 | 1 | 0 |
| 746 | 700747 | 290 | 136 | 1 | 8 | 0 |
| 747 | 700748 | 362 | 551 | 5 | 10 | 1 |
| 748 | 700749 | 211 | 98 | 1 | 4 | 0 |
| 749 | 700750 | 346 | 661 | 6 | 8 | 0 |
| 750 | 700751 | 334 | 386 | 2 | 1 | 0 |
| 751 | 700752 | 281 | 185 | 2 | 1 | 0 |
| 752 | 700753 | 282 | 97 | 0 | 0 | 0 |
| 753 | 700754 | 296 | 140 | 1 | 0 | 0 |
| 754 | 700755 | 313 | 262 | 2 | 3 | 0 |
| 755 | 700756 | 297 | 105 | 2 | 3 | 0 |
| 756 | 700757 | 315 | 62 | 1 | 0 | 0 |
| 757 | 700758 | 286 | 116 | 2 | 1 | 0 |
| 758 | 700759 | 359 | 181 | 2 | 6 | 0 |
| 759 | 700760 | 274 | 228 | 2 | 3 | 1 |
| 760 | 700761 | 321 | 191 | 0 | 2 | 0 |
| 761 | 700762 | 415 | 621 | 6 | 4 | 0 |
| 762 | 700763 | 310 | 221 | 2 | 9 | 0 |
| 763 | 700764 | 268 | 107 | 0 | 1 | 0 |
| 764 | 700765 | 296 | 78 | 1 | 3 | 0 |
| 765 | 700766 | 327 | 197 | 0 | 5 | 0 |
| 766 | 700767 | 265 | 109 | 0 | 6 | 0 |
| 767 | 700768 | 264 | 103 | 1 | 0 | 0 |
| 768 | 700769 | 459 | 908 | 9 | 2 | 0 |
| 769 | 700770 | 218 | 42 | 1 | 0 | 0 |
| 770 | 700771 | 296 | 171 | 1 | 7 | 0 |
| 771 | 700772 | 203 | 72 | 2 | 2 | 0 |
| 772 | 700773 | 268 | 116 | 2 | 0 | 0 |
| 773 | 700774 | 396 | 549 | 10 | 7 | 1 |
| 774 | 700775 | 288 | 185 | 1 | 2 | 0 |
| 775 | 700776 | 403 | 547 | 5 | 2 | 0 |
| 776 | 700777 | 333 | 304 | 5 | 4 | 1 |
| 777 | 700778 | 302 | 174 | 1 | 7 | 0 |
| 778 | 700779 | 307 | 177 | 2 | 11 | 0 |
| 779 | 700780 | 205 | 10 | 0 | 0 | 0 |
| 780 | 700781 | 505 | 777 | 0 | 4 | 0 |
| 781 | 700782 | 263 | 25 | 0 | 3 | 0 |
| 782 | 700783 | 276 | 84 | 0 | 6 | 0 |
| 783 | 700784 | 292 | 278 | 4 | 2 | 0 |
| 784 | 700785 | 652 | 764 | 6 | 6 | 0 |
| 785 | 700786 | 295 | 226 | 1 | 3 | 0 |
| 786 | 700787 | 361 | 492 | 5 | 14 | 1 |
| 787 | 700788 | 312 | 172 | 1 | 2 | 0 |
| 788 | 700789 | 280 | 107 | 5 | 2 | 0 |
| 789 | 700790 | 304 | 148 | 1 | 5 | 0 |
| 790 | 700791 | 234 | 86 | 1 | 1 | 0 |
| 791 | 700792 | 260 | 150 | 1 | 6 | 0 |
| 792 | 700793 | 280 | 258 | 2 | 5 | 0 |
| 793 | 700794 | 281 | 186 | 1 | 3 | 0 |
| 794 | 700795 | 313 | 266 | 5 | 4 | 1 |
| 795 | 700796 | 276 | 197 | 2 | 4 | 0 |
| 796 | 700797 | 301 | 201 | 3 | 1 | 0 |
| 797 | 700798 | 315 | 213 | 6 | 4 | 0 |
| 798 | 700799 | 258 | 92 | 1 | 3 | 0 |
| 799 | 700800 | 219 | 66 | 2 | 2 | 0 |
| 800 | 700801 | 320 | 118 | 1 | 4 | 0 |
| 801 | 700802 | 307 | 173 | 1 | 1 | 0 |
| 802 | 700803 | 392 | 623 | 1 | 2 | 0 |
| 803 | 700804 | 324 | 207 | 2 | 1 | 0 |
| 804 | 700805 | 307 | 252 | 4 | 3 | 0 |
| 805 | 700806 | 259 | 90 | 1 | 2 | 0 |
| 806 | 700807 | 304 | 320 | 4 | 2 | 0 |
| 807 | 700808 | 326 | 145 | 4 | 5 | 1 |
| 808 | 700809 | 259 | 51 | 1 | 1 | 0 |
| 809 | 700810 | 248 | 86 | 0 | 1 | 0 |
| 810 | 700811 | 289 | 221 | 2 | 1 | 0 |
| 811 | 700812 | 284 | 222 | 2 | 3 | 0 |
| 812 | 700813 | 308 | 130 | 3 | 2 | 0 |
| 813 | 700814 | 288 | 94 | 1 | 0 | 0 |
| 814 | 700815 | 263 | 217 | 3 | 7 | 1 |
| 815 | 700816 | 265 | 75 | 0 | 2 | 0 |
| 816 | 700817 | 209 | 14 | 0 | 0 | 0 |
| 817 | 700818 | 356 | 345 | 1 | 2 | 0 |
| 818 | 700819 | 320 | 238 | 3 | 4 | 0 |
| 819 | 700820 | 266 | 142 | 1 | 2 | 0 |
| 820 | 700821 | 237 | 82 | 1 | 3 | 0 |
| 821 | 700822 | 282 | 48 | 0 | 1 | 0 |
| 822 | 700823 | 299 | 166 | 0 | 1 | 0 |
| 823 | 700824 | 222 | 106 | 2 | 1 | 0 |
| 824 | 700825 | 313 | 242 | 2 | 2 | 0 |
| 825 | 700826 | 316 | 182 | 2 | 3 | 0 |
| 826 | 700827 | 407 | 443 | 7 | 6 | 0 |
| 827 | 700828 | 194 | 53 | 0 | 0 | 0 |
| 828 | 700829 | 298 | 229 | 2 | 2 | 0 |
| 829 | 700830 | 337 | 270 | 3 | 7 | 0 |
| 830 | 700831 | 288 | 188 | 4 | 5 | 0 |
| 831 | 700832 | 302 | 302 | 3 | 2 | 1 |
| 832 | 700833 | 301 | 309 | 2 | 1 | 0 |
| 833 | 700834 | 510 | 741 | 3 | 7 | 0 |
| 834 | 700835 | 294 | 122 | 0 | 1 | 0 |
| 835 | 700836 | 298 | 107 | 1 | 5 | 0 |
| 836 | 700837 | 276 | 114 | 0 | 3 | 0 |
| 837 | 700838 | 236 | 69 | 0 | 0 | 0 |
| 838 | 700839 | 284 | 216 | 3 | 1 | 1 |
| 839 | 700840 | 286 | 156 | 3 | 8 | 0 |
| 840 | 700841 | 291 | 179 | 1 | 3 | 0 |
| 841 | 700842 | 299 | 230 | 3 | 10 | 0 |
| 842 | 700843 | 242 | 142 | 3 | 2 | 0 |
| 843 | 700844 | 284 | 149 | 3 | 6 | 1 |
| 844 | 700845 | 323 | 251 | 5 | 6 | 1 |
| 845 | 700846 | 283 | 191 | 4 | 3 | 0 |
| 846 | 700847 | 270 | 73 | 1 | 1 | 0 |
| 847 | 700848 | 325 | 432 | 2 | 10 | 0 |
| 848 | 700849 | 267 | 92 | 0 | 3 | 0 |
| 849 | 700850 | 262 | 95 | 0 | 2 | 0 |
| 850 | 700851 | 281 | 104 | 0 | 3 | 0 |
| 851 | 700852 | 263 | 105 | 2 | 7 | 0 |
| 852 | 700853 | 276 | 128 | 3 | 3 | 0 |
| 853 | 700854 | 316 | 213 | 5 | 3 | 0 |
| 854 | 700855 | 322 | 241 | 4 | 5 | 0 |
| 855 | 700856 | 360 | 217 | 2 | 4 | 0 |
| 856 | 700857 | 289 | 169 | 3 | 5 | 0 |
| 857 | 700858 | 338 | 133 | 1 | 1 | 0 |
| 858 | 700859 | 327 | 366 | 7 | 4 | 0 |
| 859 | 700860 | 279 | 145 | 3 | 2 | 0 |
| 860 | 700861 | 306 | 203 | 3 | 5 | 0 |
| 861 | 700862 | 304 | 127 | 0 | 0 | 0 |
| 862 | 700863 | 362 | 262 | 2 | 5 | 0 |
| 863 | 700864 | 354 | 723 | 9 | 10 | 0 |
| 864 | 700865 | 320 | 252 | 3 | 4 | 0 |
| 865 | 700866 | 308 | 250 | 3 | 8 | 1 |
| 866 | 700867 | 325 | 249 | 6 | 7 | 0 |
| 867 | 700868 | 308 | 290 | 4 | 5 | 1 |
| 868 | 700869 | 282 | 97 | 1 | 5 | 0 |
| 869 | 700870 | 318 | 208 | 1 | 3 | 0 |
| 870 | 700871 | 314 | 238 | 6 | 1 | 1 |
| 871 | 700872 | 289 | 148 | 1 | 2 | 0 |
| 872 | 700873 | 393 | 431 | 4 | 1 | 1 |
| 873 | 700874 | 156 | 0 | 0 | 0 | 0 |
| 874 | 700875 | 284 | 184 | 3 | 1 | 0 |
| 875 | 700876 | 326 | 390 | 4 | 9 | 0 |
| 876 | 700877 | 256 | 125 | 2 | 0 | 0 |
| 877 | 700878 | 297 | 286 | 4 | 0 | 0 |
| 878 | 700879 | 308 | 238 | 3 | 1 | 0 |
| 879 | 700880 | 286 | 136 | 2 | 3 | 0 |
| 880 | 700881 | 296 | 131 | 4 | 1 | 0 |
| 881 | 700882 | 290 | 209 | 5 | 2 | 0 |
| 882 | 700883 | 298 | 264 | 3 | 5 | 0 |
| 883 | 700884 | 328 | 257 | 1 | 7 | 0 |
| 884 | 700885 | 328 | 194 | 2 | 2 | 0 |
| 885 | 700886 | 282 | 96 | 4 | 5 | 1 |
| 886 | 700887 | 272 | 164 | 1 | 3 | 0 |
| 887 | 700888 | 305 | 255 | 1 | 2 | 0 |
| 888 | 700889 | 283 | 188 | 4 | 0 | 0 |
| 889 | 700890 | 316 | 19 | 1 | 0 | 0 |
| 890 | 700891 | 250 | 67 | 1 | 7 | 0 |
| 891 | 700892 | 312 | 204 | 1 | 1 | 0 |
| 892 | 700893 | 286 | 130 | 2 | 1 | 0 |
| 893 | 700894 | 287 | 248 | 1 | 2 | 0 |
| 894 | 700895 | 318 | 233 | 3 | 4 | 0 |
| 895 | 700896 | 274 | 151 | 2 | 13 | 0 |
| 896 | 700897 | 171 | 10 | 1 | 0 | 0 |
| 897 | 700898 | 318 | 247 | 6 | 3 | 0 |
| 898 | 700899 | 278 | 229 | 2 | 6 | 0 |
| 899 | 700900 | 166 | 20 | 0 | 0 | 0 |
| 900 | 700901 | 407 | 220 | 1 | 3 | 0 |
| 901 | 700902 | 285 | 209 | 1 | 4 | 0 |
| 902 | 700903 | 224 | 76 | 0 | 1 | 0 |
| 903 | 700904 | 571 | 1307 | 11 | 11 | 1 |
| 904 | 700905 | 324 | 167 | 3 | 4 | 0 |
| 905 | 700906 | 316 | 239 | 0 | 0 | 0 |
| 906 | 700907 | 306 | 55 | 2 | 1 | 0 |
| 907 | 700908 | 294 | 140 | 3 | 0 | 0 |
| 908 | 700909 | 276 | 120 | 1 | 1 | 0 |
| 909 | 700910 | 334 | 409 | 2 | 4 | 0 |
| 910 | 700911 | 233 | 40 | 1 | 2 | 0 |
| 911 | 700912 | 238 | 54 | 0 | 0 | 0 |
| 912 | 700913 | 412 | 606 | 7 | 12 | 0 |
| 913 | 700914 | 274 | 73 | 1 | 0 | 0 |
| 914 | 700915 | 329 | 391 | 3 | 7 | 0 |
| 915 | 700916 | 309 | 149 | 3 | 3 | 0 |
| 916 | 700917 | 398 | 828 | 7 | 11 | 1 |
| 917 | 700918 | 279 | 223 | 0 | 5 | 0 |
| 918 | 700919 | 291 | 188 | 0 | 10 | 0 |
| 919 | 700920 | 300 | 2 | 0 | 0 | 0 |
| 920 | 700921 | 191 | 47 | 1 | 3 | 0 |
| 921 | 700922 | 259 | 163 | 0 | 7 | 0 |
| 922 | 700923 | 288 | 223 | 2 | 11 | 0 |
| 923 | 700924 | 334 | 182 | 5 | 6 | 0 |
| 924 | 700925 | 343 | 288 | 3 | 4 | 0 |
| 925 | 700926 | 303 | 224 | 5 | 2 | 0 |
| 926 | 700927 | 342 | 290 | 4 | 9 | 1 |
| 927 | 700928 | 291 | 271 | 2 | 4 | 0 |
| 928 | 700929 | 293 | 170 | 4 | 5 | 0 |
| 929 | 700930 | 316 | 297 | 1 | 2 | 0 |
| 930 | 700931 | 301 | 184 | 0 | 8 | 0 |
| 931 | 700932 | 355 | 304 | 3 | 3 | 0 |
| 932 | 700933 | 275 | 111 | 2 | 6 | 0 |
| 933 | 700934 | 236 | 28 | 0 | 2 | 0 |
| 934 | 700935 | 334 | 346 | 2 | 0 | 0 |
| 935 | 700936 | 308 | 255 | 2 | 3 | 0 |
| 936 | 700937 | 402 | 605 | 7 | 14 | 0 |
| 937 | 700938 | 333 | 217 | 2 | 3 | 0 |
| 938 | 700939 | 347 | 366 | 4 | 11 | 0 |
| 939 | 700940 | 285 | 207 | 0 | 5 | 0 |
| 940 | 700941 | 372 | 291 | 3 | 0 | 0 |
| 941 | 700942 | 295 | 109 | 2 | 4 | 1 |
| 942 | 700943 | 349 | 408 | 4 | 3 | 0 |
| 943 | 700944 | 265 | 19 | 0 | 0 | 0 |
| 944 | 700945 | 309 | 256 | 4 | 3 | 0 |
| 945 | 700946 | 288 | 175 | 1 | 3 | 0 |
| 946 | 700947 | 305 | 281 | 4 | 3 | 0 |
| 947 | 700948 | 213 | 42 | 0 | 0 | 0 |
| 948 | 700949 | 261 | 158 | 0 | 4 | 0 |
| 949 | 700950 | 285 | 148 | 0 | 5 | 0 |
| 950 | 700951 | 312 | 201 | 4 | 7 | 1 |
| 951 | 700952 | 206 | 41 | 0 | 2 | 0 |
| 952 | 700953 | 318 | 341 | 5 | 3 | 1 |
| 953 | 700954 | 291 | 223 | 2 | 3 | 0 |
| 954 | 700955 | 310 | 276 | 2 | 0 | 0 |
| 955 | 700956 | 286 | 181 | 4 | 1 | 0 |
| 956 | 700957 | 323 | 306 | 4 | 10 | 0 |
| 957 | 700958 | 324 | 260 | 2 | 4 | 0 |
| 958 | 700959 | 300 | 241 | 1 | 1 | 0 |
| 959 | 700960 | 284 | 180 | 3 | 4 | 0 |
| 960 | 700961 | 269 | 113 | 1 | 9 | 0 |
| 961 | 700962 | 295 | 201 | 2 | 4 | 0 |
| 962 | 700963 | 286 | 268 | 5 | 4 | 0 |
| 963 | 700964 | 365 | 366 | 4 | 6 | 0 |
| 964 | 700965 | 175 | 5 | 0 | 0 | 0 |
| 965 | 700966 | 320 | 203 | 4 | 10 | 0 |
| 966 | 700967 | 364 | 391 | 7 | 5 | 0 |
| 967 | 700968 | 312 | 42 | 1 | 1 | 0 |
| 968 | 700969 | 280 | 167 | 1 | 7 | 0 |
| 969 | 700970 | 309 | 365 | 2 | 3 | 0 |
| 970 | 700971 | 269 | 98 | 1 | 2 | 0 |
| 971 | 700972 | 289 | 68 | 1 | 2 | 0 |
| 972 | 700973 | 340 | 170 | 3 | 2 | 0 |
| 973 | 700974 | 271 | 248 | 3 | 1 | 0 |
| 974 | 700975 | 289 | 148 | 2 | 4 | 0 |
| 975 | 700976 | 454 | 77 | 2 | 10 | 1 |
| 976 | 700977 | 308 | 307 | 0 | 0 | 0 |
| 977 | 700978 | 297 | 111 | 1 | 1 | 0 |
| 978 | 700979 | 228 | 19 | 0 | 0 | 0 |
| 979 | 700980 | 265 | 125 | 3 | 1 | 0 |
| 980 | 700981 | 237 | 116 | 0 | 0 | 0 |
| 981 | 700982 | 266 | 79 | 2 | 1 | 0 |
| 982 | 700983 | 325 | 286 | 1 | 3 | 0 |
| 983 | 700984 | 305 | 295 | 2 | 6 | 0 |
| 984 | 700985 | 277 | 112 | 2 | 3 | 0 |
| 985 | 700986 | 289 | 198 | 2 | 1 | 0 |
| 986 | 700987 | 254 | 108 | 2 | 0 | 0 |
| 987 | 700988 | 264 | 140 | 3 | 3 | 1 |
| 988 | 700989 | 318 | 254 | 3 | 6 | 0 |
| 989 | 700990 | 330 | 337 | 1 | 2 | 0 |
| 990 | 700991 | 206 | 19 | 0 | 2 | 0 |
| 991 | 700992 | 300 | 102 | 1 | 4 | 0 |
| 992 | 700993 | 286 | 131 | 3 | 2 | 0 |
| 993 | 700994 | 271 | 147 | 4 | 4 | 1 |
| 994 | 700995 | 302 | 322 | 4 | 8 | 0 |
| 995 | 700996 | 295 | 220 | 3 | 1 | 0 |
| 996 | 700997 | 192 | 5 | 0 | 0 | 0 |
| 997 | 700998 | 288 | 75 | 1 | 4 | 0 |
| 998 | 700999 | 283 | 227 | 1 | 3 | 0 |
| 999 | 701000 | 302 | 241 | 3 | 2 | 0 |

## Throughput

- Turns simulated: 301630 (first play of each game)
- Aggregate: 255.8 turns/s over 1179.0s wall
- Per game thread: 64.1 turns/s

## Determinism

PASS: 1000 games played twice with identical per-turn hashes (and save/load keeps the final hash).

## Panics

None.

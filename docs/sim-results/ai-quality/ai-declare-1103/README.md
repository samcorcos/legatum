# Why Hard would not declare on unwalled Lugano (2026-11-03, branch `claude/ai-declare`)

From the coach's review of the designer's 8.9.24 Swiss win (`docs/playtests/reviews/2026-11-02-8.9.24-swiss.md`). In the Scenario Lab cut `swiss-strongbox` (the Mongols at T50: six military units against two, unwalled Lugano seven tiles away), Hard declared War in only 29 of 200 runs and took Lugano in 6%. In his game the Mongols attacked Lugano at T58 with too little.

## Diagnosis (the Mongols' own view, `war_trace` with `WAR_TRACE_DIAG=3`)

`war_diagnosis` now also prints, at Peace, each rival a war could open on: its nearest cities, the units in the two-turn approach ("reach") and in the six-turn march, each force's siege forecast, `campaign_forecast` and `campaign_outlook`.

- The war value was never asked. From T53 (the Peace signed at T43 lapses) to T75 the plan layer had **no Campaign candidate at all** (`alternatives []`): `campaign_forecast` returned `None` for Lugano on every turn.
- The cause is one early return in `plan.rs::campaign_forecast_with`: when any unit is in reach, the forecast is the in-reach force's alone, and a reach force with no melee unit ends it. The Rangers walked ahead of the Soldiers, so on 10 to 15 of the 25 turns in each run (6 of 6 runs traced) three Rangers stood in reach with no capturer while the march force (a Soldier or Phalangite with them) forecast taking Lugano in 4 rounds (expected losses 58 Production). The whole army placed beside Lugano takes it in 3 rounds.
- With that fixed alone, a Campaign appears at T58 (value +93: capture +455 at 7% odds, failed siege -95, peace credibility -150, broken peace -44, aggressor risk -35, era deficit -40), wins the plan at T59 and declares at once, four turns' march out. That is the raid of his game: 200 of 200 runs declare, 1% take Lugano.
- The fix is the forecast plus arriving together: the existing Sparta staging (`planned_stage`: the field force walks to the target at Peace) and muster (`early_campaign_muster`: an early Campaign on an open city declares only when the force in reach takes it). Staging and muster without the forecast fix: 1% (no candidate to stage for).

## Gene

| Gene | Value | What it does |
|---|---|---|
| `march_past_reach` | 2 (new, default) | A campaign forecast with no committed force whose in-reach units cannot take the city alone is judged by the march window's field army (1: only for a new war; 2: also a city already at war); a Campaign plan then stages its field force toward the city (as `planned_stage`: at Peace, and at war every free unit marches, even alone) and, in the first third of the game against an open city, declares only once the force in reach takes it (as `early_campaign_muster`). `plan.rs::campaign_forecast_with`, `stages_campaigns`, `musters_campaigns`. |

## Scenario Lab (200 runs, Hard, all AI seats on the given parameters)

| Arm | `swiss-strongbox` taken | Seat declares (median turn) | Median score | `german-second-front` |
|---|---:|---:|---:|---:|
| control | 6% (12) | 29 (T65) | 380 | 0% |
| forecast fix only | 1% (2) | 200 (T59) | 340 | |
| staging + muster only | 1% (2) | 19 (T66) | | |
| forecast + muster | 0.5% (1) | 18 | 380 | |
| forecast + staging | 0% | 200 (T59) | | |
| `march_past_reach` 1 or 2 (all three) | **85% (170)** | 200 (T61) | 890 | 0% |

`german-second-front` is a different failure: Germany starts at T110 knowing no Swiss city (`campaign_against`: "no city seen yet"), then only Winterthur, 11 hexes from its nearest unit; Lugano is never seen. That is the coach's recommendation 3 (keep the leader's capital in view), not touched here.

## His game, replayed (`coach_branch`, his tape for the Swiss seat including his T58 and T59 purchases, Hard rivals, 32 reseeded runs to T75)

| Branch | Rivals | Mongols declare | Mongol units within 3 of Lugano at the declaration | Lugano falls | Lowest Lugano HP (median) |
|---|---|---|---:|---:|---:|
| T50 | `march_past_reach` 0 | T54 in 32 | 1 | 0 | 27.5 |
| T50 | `march_past_reach` 2 | T58 in 32 | 4 | 3 (T63) | 176 |
| T58 (at war) | `march_past_reach` 0 | | | 0 | 28 |
| T58 (at war) | `march_past_reach` 2 | | | **31 (T62-T63)** | 0 |
| T50 | `march_past_reach` 1 | T58 in 32 | | 8 | |
| T58 (at war) | `march_past_reach` 1 | | | 31 | |
| T58 (at war) | 1, staging only at Peace (experiment) | | | 0 | |

From T58, the decision the coach named, the staging marches every free unit at war, so the melee units keep coming and the raid becomes a capture (staging only at Peace: 0 of 32). From T50 the Mongols now stage and declare with four units at the walls instead of one, but his two bought Composite Archers turn the campaign onto Thun (T60) and back; the tape seat diverges more (median 17 failed orders against 9), so read the T50 row as direction only.

## Human-seat gate

`hard_vs_human`, seed 310001, 256 reseeded salts x 8 maps (2,048 games per arm), plain Hard rivals, one spot server, batch `declare-gate1-1103-012135` (0.14 server-hours). Control `ctl`: `march_past_reach` 0 in every seat. Gate 1 ran the code that is now value 2; gate 2 (`declare-gate2-1103-021248`, [arms-new-wars.json](arms-new-wars.json), [comparison-new-wars.json](comparison-new-wars.json)) value 1, same control games. The rivals-only and seat-only arms set the other seats to the control through `other_params` ([arms.json](arms.json), [comparison.json](comparison.json), [compare.py](compare.py)). Delta: change in the designer's-seat win rate; on a rivals-only arm negative means stronger rivals.

| Arm | Seat wins | Delta pp [95%] | Rival cities T50 / T100 | Rival Settlers lost a game |
|---|---:|---:|---:|---:|
| ctl | 456 | | 3.10 / 5.32 | 0.537 |
| `march_past_reach` 2, rivals only | 404 | **-2.54 [-4.35, -0.68]** | 3.10 / 5.33 | 0.514 |
| `march_past_reach` 2, seat only | 486 | +1.46 [-0.05, +3.08] | 3.10 / 5.31 | 0.523 |
| `march_past_reach` 1, rivals only | 416 | **-1.95 [-3.66, -0.24]** | 3.10 / 5.32 | 0.523 |
| `march_past_reach` 1, seat only | 466 | +0.49 [-0.98, +1.95] | 3.10 / 5.31 | 0.527 |

Per map, rivals only: his 8.3 Switzerland map 104 to 56 seat wins, 8.6 Germany loss 95 to 79, 8.5 Switzerland 21 to 10; 8.4 Japan, Celts and Sparta a few wins up (60 to 66, 31 to 37, 27 to 32).

## War-scenario suite (`tools/sim/war-scenarios.sh`, 64 runs, paired against the gene off in every seat)

| Arm | Suite mean | Moved beyond noise |
|---|---:|---|
| on in every seat | 37% (off: 38%) | `war-japan-sparta-t142` 28% to 4% (Japan's cities against a Sparta that now brings its army; not there seat-only), `war-mongols-solothurn-t158` 14% to 0% |
| on in the seat only (1 and 2 identical) | 37% | `war-mongols-solothurn-t158` 14% to 0% [-23.4, -6.2], `war-mongols-chur-t137` 32% to 23% [-18.8, 0.0] |

The two late Mongol attacks lose: at Solothurn (T158, at war with Japan, Peace with the Swiss) the Mongols declare on Solothurn as before, then re-aim at walled St. Gallen (T161, value 5,724) once the marching units put a melee unit in its reach, and take nothing. Values 1 and 2 read the same, so the cost is the staging at war, the same marching that wins his T58. Not fixed here: the next lever is keeping a siege on the city it started (re-aim hysteresis), not less marching.

## Decision

Ship `march_past_reach` 2: rivals stronger (the interval excludes zero), the seat stronger, the scenario goes from 6% to 85%, and from his T58 position Lugano falls in 31 of 32 runs. Value 1 is weaker on both gate arms and reads the same on the war suite. Known cost: the late Mongol attack scenarios Solothurn and Chur (above). The archived Germany T185 air decision test pins the gene to 0 (one unit stages instead of skipping).

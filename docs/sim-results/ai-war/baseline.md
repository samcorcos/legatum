# AI war plan: baseline (increment 0)

Other sections (fixtures, the gate, the ranked fixes) are added as `plans/ai-war.md` story W0.1 to W0.5 land.

## W2.4: execute a war on the hollow leader (2026-10-30, gene `hollow_march`, off)

Branch `claude/hollow-attack`. Chain and design: `docs/learnings.md` 2026-10-30; rules: `.claude/skills/legatum-rules/references/game-rules.md`. New fixtures from the designer's 8.9.17 Mongols save (`~/LegatumRuns/bug-saves/mongols-hollow-8917-20261030.legatum`; 0 hash mismatches, 380/380 AI order lists, memory equal to the save's): `war-hollow-swiss-t76` (his recorded orders T76-95), `war-hollow-byzantium-t86` (his orders T86-95, then Hard), `war-hollow-swiss-t96` (Hard in his seat).

### War suite (33 cases x 64 runs; arms: `w24/params/`, runner `w24/run.sh`)

| Arm (`hollow_march`) | Seat mean / parity | All seats | Cases that move (control -> arm, seat) |
| --- | --- | --- | --- |
| control | 43% / 13 | - | - |
| 3 (spare garrisons, capital too) | 44% / 14 | 44% / 14 | Chur T137 34 -> 59; Mongol bombers T150 17 -> 26; Golden Horde thin 0 -> 4 |
| 11 (3 + buys), 15 (all four parts) | 44% / 14 | 44% / 14 | same as 3 |
| 31 (with a long-siege forecast, removed) | 43% / 13 | 43% / 13 | Chur 37; Golden Horde hold 100 -> 95 (all) |

The three new fixtures read 0% in every arm (Swiss T96 enemy units lost 0.01 -> 1.71 with buys). Scenario Lab `mongols-rich-and-empty`: 0/50 in every arm, median 1,230-1,240.

### Census (256 all-AI 8-empire Hard games, seed 7300001, arm on every seat; `tools/sim/flip-metrics.sh`)

| Arm | Captures / game | Wars declared / game | Re-captures within 20 turns |
| --- | --- | --- | --- |
| control | 17.6 | 25.4 | 3.57 |
| 3 | 16.9 | 24.8 | 3.16 |
| 15 | 17.4 | 25.5 | 3.53 |

### Decision cost (seat planning ms per turn, `war_trace`, 3 runs each, native)

Arm 3 within noise of the control (Swiss T96 2.3 -> 2.4, Chur 4.8 -> 4.7, Golden Horde thin 2.8 -> 2.8); arm 15 +0.3 ms on the Swiss T96 (the forecast in the purchase check).

### Attack check bisect

`war-golden-horde` / `-thin` / `war-walls-rise`: 96 / 87 / 81% at 38eef332d, 0 / 0 / 0% at a590efad6 (door defense on). On today's master with the defenders' `door_defense` 0: 98 / 89 / 81%.

## W2.3: read the hollow leader (2026-10-29, genes `hollow_read` and `persist_hollow`, both off)

Branch `claude/hollow-leader` (on `claude/war-persistence`). From the 8.9.15 Swiss review rec 2: an empire whose public army per city (Strongest Army title values) is at most half the median empire's, and that is rich in public (a crown, a live goal, a Power lead, an era lead or a visible Swiss bounty), counts as a runaway without a strict VP lead, is containable, and its cities rank higher as targets. Once read it stays hollow down to 25 (one bought defender took the Swiss from 66 to 35 the turn after the Mongols declared). `persist_hollow` limits W2.2's `war_reaim` and `peace_own_danger` to such an enemy. Rules: `.claude/skills/legatum-rules/references/game-rules.md`.

**The hollow read is neutral in full games and helps the war it was cut from; with it, W2.2's persistence no longer costs the AIs in full games.**

### War suite (30 cases x 64 runs; `w23/war-suite.json`)

Suite mean 47% in every arm (control 47%, parity 13). Only these cases move (control -> arm):

| Case | Control | `hollow_read` seat / all | hollow + persistence seat / all |
| --- | --- | --- | --- |
| `war-mongols-chur-t137` | 3% | 16% / 16% | **34% / 34%** |
| `war-japan-sparta-t142` | 89% | 89% / 89% | 88% / **83%** |
| `war-mongol-bombers-t150` | 16% | 16% / 17% | 14% / 17% |
| `war-mongol-walls` | 98% | 98% / 98% | 98% / 97% |

Persistence gated on hollowness alone (no riches, no hysteresis) cost Japan T142 89 -> 72 (all seats): the designer's Sparta of four veterans reads 55-74 hollow, and Japan's Peace offers were its defence. Gating on the rich-and-hollow bonus fixed that (88). Without hysteresis persistence never fired in Chur (3%), because the Swiss stopped reading hollow on T139; with it, 34%. In the Chur trace the Mongols name the Swiss their top threat on T137 instead of T145.

### Scenario Lab `mongols-rich-and-empty` (50 runs, every seat on the arm)

0/50 in every arm, median 1,240-1,250 (control 1,250): the walled Lausanne and St. Gallen in 20 turns stay out of reach, and every seat (the Swiss too) is Hard here, so the Swiss buy defenders. Peace offers to Switzerland: control 8 runs, `hollow_read` 7, hollow + persistence 48 (as W2.2's re-aim alone: a re-aimed war that loses its target makes the plan stand down).

### Human-seat gate (AWS, 2,048 games per arm, seeds 270001+, dice reseeded; `w23/full/`)

Control 329/2,048 (same as W2.1, W2.2 and the Siege gate). Positive = the seat wins more.

| Arm | Delta pp | 95% desc | Decision |
| --- | --- | --- | --- |
| `hollow_read`, rivals only | +0.34 | [-1.39, +2.07] | ship-monitor |
| `hollow_read`, seat only | +1.07 | [-0.18, +2.33] | noninferior |
| hollow + `persist_hollow` + `war_reaim` 1 + `peace_own_danger` 2, rivals only | +0.34 | [-1.38, +2.07] | ship-monitor |
| same, all seats | +0.88 | [-0.97, +2.73] | ship-monitor |

An earlier build without the hysteresis read `hollow_read` rivals-only at -0.73 [-2.41, +0.95]. W2.2's blanket persistence was +2.10 [+0.38, +3.81] on the same seeds.

### Recommendation

Turn `hollow_read` on: neutral in full games, Chur 3% -> 16%, the leader named 8 turns earlier, nothing else moves. The persistence bundle (`persist_hollow` 1, `war_reaim` 1, `peace_own_danger` 2 with `hollow_read`) meets the bar too (full games neutral, Chur 34%) but costs Japan T142 6 points when every seat has it; turn it on with the human-seat benchmark watching Japan-style defences, or first make `peace_own_danger` count the exchange (W2.2's note).

Cost: 0.32 server-hours on AWS (two gates, about $0.21) and about 35 minutes of box time.

## W2.2: a war on the leader survives one bought defender (2026-10-29, genes `war_reaim` and `peace_own_danger`, both off)

Branch `claude/war-persistence`. From the 8.9.15 Swiss review (`docs/playtests/2026-10-28-swiss-review.md`, recs 1 and 3): the Mongol wars on the designer ended two turns after one bought defender and fell back to Peace, and the Mongols offered Peace mid-campaign because Japanese units pressed their own towns.

**Re-aim, don't quit, helps the AI in the war it was cut from and costs nothing elsewhere; Peace that follows the plan helps the attacker only in its stricter form (level 2), which costs the Japan T142 defence 9 points. In full games both are neutral within ±4 pp. Both stay off; the AWS gate decides.**

### What the genes do

- `war_reaim` 1 (`plan.rs::reaim_candidates`): when the plan's campaign on an enemy ends (insufficient army, Walls before arrival, negative value, target gone) or its city drops off the candidates, that enemy's other cities within the march window are ranked by worth over defence (with one defender the owner could buy; treasuries are hidden, so every owner is assumed able to buy one unit a city a turn) and march time, and the best three are priced as campaigns with the war already open (no second-front veto; every march window tried, the best capture odds kept). Peace wins only if none reaches `min_seat_war_odds`. Level 2 also forecasts an empty target as holding that bought defender.
- `peace_own_danger` (`diplomacy.rs::wants_peace`): toward the team the plan fights, `endangered()` (any enemy's pressure) becomes that team's danger or a collapse (a city at half HP with any enemy beside it). Level 1: the usual local test, the team's units within 5 against our units within 2. Level 2: against all our units within 5, or the city at half HP.

### Fixtures (new, from the 8.9.15 save, replayed with 0 hash mismatches and 531/531 AI order lists; the designer's seat replays their recorded orders, every AI carries its memory)

| Fixture | Goal | Control | `war_reaim` 1 | `war_reaim` 2 | level 1 Peace | level 2 Peace | reaim + L1 | reaim + L2 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `war-mongols-chur-t137` | take a Swiss city in 20 turns | 3% (2/64) | 17% | 3% | 3% | 3% | 17% | **23%** |
| `war-mongols-solothurn-t158` | take a Swiss city in 14 turns | 43% | 43% | 43% | **0%** | 43% | 0% | 43% |

Read the Solothurn row with care. In the control the Mongol Peace offer comes on T159, the designer's recorded acceptance on T162 names a proposal that no longer exists, the war stays on and the Mongols take Chur around T171. With level 1 the offer moves to T160 (one Swiss Field Cannon four hexes from an empty Mongol colony is "danger from the Swiss"), the recorded acceptance matches it, and the war ends. That is the fixture's proposal-id fidelity, but the offer itself is the behaviour rec 3 wanted gone, and level 2 removes it.

On T139 in the Chur fixture the re-aimed plan switched from Chur to Schaffhausen (value +427) instead of to Peace (−300); without the second-front veto and with the march judged window by window, Lausanne and Schaffhausen forecast at 15–19% capture odds where the full six-turn march read 0–7%. Level 2 of `war_reaim` priced every target under 15% and changed nothing.

### War suite (30 cases × 64 runs, mean goal success; control 41%, 11 of 30 at human parity)

| Arm | Seat | Opponents | All | What moves (control → arm) |
| --- | --- | --- | --- | --- |
| `war_reaim` 1 | 42% | 41% | 42% | Chur 3 → 17 (seat, all); nothing else outside ±3 |
| `peace_own_danger` 1 | 40% | 41% | 40% | Solothurn 43 → 0 (seat, all); German stand 95 → 93 (opp, all) |
| `peace_own_danger` 2 | 41% | 41% | 41% | **Japan T142 82 → 73 (seat), 75 (opp), 70 (all)**; Mongol bombers 15 → 17 |
| reaim 1 + level 1 | 40% | 41% | 40% | Chur 17, Solothurn 0 |
| reaim 1 + level 2 | 42% | 41% | 41% | Chur 23; Japan T142 73 (seat), 76 (opp), 70 (all) |

Japan T142 under level 2: the Japanese AI's thirteen units outweigh the four Spartan veterans by the strength estimate, so it no longer offers Sparta Peace, but it loses every fight (30 units for none in the designer's game) and those offers were its defence. Golden Horde (thin) and every other attack check are unchanged. Per-case numbers: `w22/war-suite.json`.

### What-if: Hard in the designer's seat from T137 and T158, genes on for every AI (64 salts, dice reseeded; `coach8915_whatif`)

| Branch | Hard seat wins | Mongols win |
| --- | --- | --- |
| T137 control | 28/64 (44%, the coach's 44%) | 14 |
| T137 `war_reaim` 1 | 20/64 (31%) | 22 |
| T137 `peace_own_danger` 1 | 32/64 (50%) | 13 |
| T137 `peace_own_danger` 2 | 18/64 (28%) | 21 |
| T137 reaim + level 1 | 25/64 (39%) | 20 |
| T137 reaim + level 2 | **15/64 (23%)**; the coach's "the Mongols never make Peace" was 25% | 20 |
| T158 control / reaim / L1 / L2 / reaim + L1 / reaim + L2 | 56 / 57 / 58 / 54 / 60 / 56 of 64 | |

The designer's recorded orders from T137 win 2/64 on this build against the coach's 49/64 at the 8.9.15 tag (the 12 commits since change enough of the Mongol and Sparta theatre to break the recorded line), and 0/64 with reaim + L1 or L2, so the tape column says nothing here. Raw counts: `w22/whatif.json`.

### Scenario Lab `mongols-rich-and-empty` (50 runs, every seat Hard with the genes)

0/50 in every arm (control, `war_reaim` 1, reaim + L1, reaim + L2), median score 1,250. The goal (Lausanne and St. Gallen, walled, in 20 turns) is beyond this army either way. Peace offers to Switzerland in the recorded decision turns: control 8 of 50 runs, `war_reaim` 1 48, reaim + L1 34, reaim + L2 3. Re-aiming alone keeps a war plan on the Swiss alive and so lets the old any-enemy danger test offer Peace more often: the two genes belong together.

### Human-seat gate (local W2.0 method on the box: 8 maps × 64 salts from 270001 = 512 games per arm, dice reseeded, plain Hard rivals; `w04_gate.py`, Bonferroni over 6)

Control 103/512 (20.1%).

| Arm | Seat Δ pp [95%] {adjusted} |
| --- | --- |
| reaim 1 + level 1, seat only | −1.4 [−3.4, +0.6] {−4.1, +1.3} |
| reaim 1 + level 1, rivals only | +0.0 [−3.1, +3.1] {−4.1, +4.1} |
| reaim 1 + level 1, all seats | −1.6 [−5.0, +1.8] {−6.1, +3.0} |
| reaim 1 + level 2, seat only | −1.2 [−3.7, +1.3] {−4.5, +2.2} |
| reaim 1 + level 2, rivals only | +0.6 [−3.2, +4.4] {−4.5, +5.7} |
| reaim 1 + level 2, all seats | −3.3 [−7.1, +0.4] {−8.4, +1.7} |

All inside noise. Rivals with the genes do not beat the seat more in full games, as with door defense: the full benchmark is dominated by economy and expansion. The level-2 control arm is the level-1 batch's (same salts, genes off; separate binary). Arms: `w22/arms.json`; summary: `w22/gate-summary.json`.

### Recommendation

Keep both off. The case for turning them on is the war they were cut from: reaim + level 2 takes the T137 Hard seat from 44% to 23% and the Chur fixture from 3% to 23%. The case against: level 2 costs the Japan T142 defence 9 points, the full-game gate is neutral (seat −1.2, rivals +0.6), and a 512-game gate cannot see a ±2 pp effect. The AWS gate (2,048 games) should run `war_reaim` 1 alone and with `peace_own_danger` 2, seat-only and rivals-only. A level 2 that counts the exchange (our units lost to that enemy against theirs) rather than raw strength should keep Japan's Peace offers and is the obvious next round.

## W2.1: "defend where the enemy army is" turned on (2026-10-28, `door_defense` 100, `door_parts` 29)

Full-size human-seat gate (batch `w21-door-1028-153237`, 4 spot servers, 0.15 server-hours, about $0.10; plain Hard rivals; control 331 of 2,048, equal to W2.0's): seat-only -1.1 pp [-2.8, +0.6], rivals-only -0.3 [-2.3, +1.6], all seats -0.8 [-2.8, +1.2]; all inside noise, no map drops more than 9 wins of 256. Japan T142 recheck, 128 runs, every seat: 79% off, 78% on, Japan's units lost 8.5 to 7.8, Sparta's 9.7 to 10.4. With the war suite (+6.5 points) and adaptive Sparta (19 to 10 wins of 32) from round 2, it meets both ship conditions (rivals-only at or below 0, Japan T142 holds) and is on by default from 8.9.15. Its gains show in wars, not yet in full-game win rates. Per-arm JSON: `w21/full/`.

## W2.0: gate rivals use aircraft (2026-10-28, branch `claude/gate-rivals-bomb`)

**The human-seat benchmark's rivals now play plain Hard, like the app's default lobby, so they bomb. The control seat wins 331 of 2048 (16.2%), not 39.8%. On that fixed benchmark Fighters (`air_defense=100`) gain +3.2 pp for the seat, and 8.9.14's modernization gain still holds.**

### The choice: plain Hard rivals (option b), not `search_air_orders=1` (option a)

- **What the designer plays against:** the lobby's default level is Hard (`app/src/lobby.ts`, `levelFields(preference?.level ?? 'Hard')`), which sets no `search` flag, and the wasm session only wraps a seat in Search when `search` is true (`crates/wasm/src/session.rs`). Hard+Search is the lobby's "(experimental)" level. The benchmark maps carry `search: true` on every AI seat, so the old gate measured the experimental level, which also drops air strikes.
- **Option (a) is not a small change:** `search_air_orders` exists only on `astra/search-air-execution`, which is 229 files and about 145,000 lines (strategy lab, gates, reports) against master, and `hard_vs_human` on master rejects the gene name (a first batch of this work died on `unknown field search_air_orders`). Landing it only to default a flag to 1 would also leave the rivals as a Search approximation of plain Hard: W0.4 measured `search_air_orders` at -22.4 pp for the seat, and the plain-Hard control here is 16.2% against 36.4% for the old Search rivals, a gap of 20.2 pp, about the same size, so plain Hard rivals and Search rivals with air strikes on are close to the same opponent.
- **Cost and speed:** plain Hard is 31 ms p95 for a whole 8-AI lobby at T250 against 2.3 to 5.0 s with Search (W0.3). This batch (5 arms by 2,048 games) used 0.74 server-hours (about $0.48). The W0.4 batch of 14 arms with Search rivals used 7.78 server-hours.
- **What changed in the code:** `hard_vs_human` plays rivals as plain `ScriptedBrain` unless `--rival-search` (single runs, and `tools/sim/hard-vs-human.sh --rival-search`) or an arm's `"rival_search": true` is given. Hard+Search stays available for the rivals that way. The seat is unchanged (`--brain hard+search` still searches). Nothing in the game or the AI changed, so no AI default or rule changed.
- **Ledger and old reports:** every figure in `docs/sim-results/hard-vs-human/ledger.jsonl` and W0.4 above was measured against Search rivals. Compare new numbers only with new numbers. The `ctl_search` arm below reproduces the old benchmark on 8.9.14.

### Method

- Binary: `claude/gate-rivals-bomb` `69171c6cd` (master `1e481f8c3` = playtest 8.9.14 plus the rivals change). Gate: the same frozen-control method as W0.4: the eight maps, salts 270001 to 270256 (2,048 games per arm), dice reseeded per salt (`LEGATUM_HVH_RESEED_DICE=1`), every arm paired with the `ctl` arm of the same batch through `tools/ai-quality/w04_gate.py` (equal-weight map strata, the larger of the stratified and salt-cluster standard errors). Simultaneous intervals are Bonferroni over 4 comparisons. Arms: `docs/sim-results/ai-war/w20/arms.json`; results: `w20/gate-summary.json`, `w20/per-map-wins.json`. The control was not repeated this time (W0.4 showed 2,048 of 2,048 hashes reproduce).
- Modernization "off" means `modernization_upgrade=0` and `modernization_return=0` (what 8.9.14 turned on). `modernization_value` stays at 100. Positive delta is a higher seat win rate.
- The arm `a_opp` (Search rivals plus `search_air_orders=1`) was dropped because the gene is not on master.

### New frozen control and per-map wins (of 256 games per map)

| Map | ctl (plain Hard rivals) | Fighters, seat only | Modernization off, seat only | Modernization off, rivals only | Old benchmark (Search rivals) |
| --- | --- | --- | --- | --- | --- |
| 8.3-switzerland-win | 11 (4.3%) | 17 (6.6%) | 9 (3.5%) | 24 (9.4%) | 51 (19.9%) |
| 8.4-sparta-win | 48 (18.8%) | 69 (27.0%) | 35 (13.7%) | 54 (21.1%) | 101 (39.5%) |
| 8.4-japan-win | 66 (25.8%) | 71 (27.7%) | 57 (22.3%) | 83 (32.4%) | 151 (59.0%) |
| 8.4-celts-win | 11 (4.3%) | 7 (2.7%) | 5 (2.0%) | 8 (3.1%) | 21 (8.2%) |
| 8.5-mongols-win | 11 (4.3%) | 13 (5.1%) | 4 (1.6%) | 11 (4.3%) | 20 (7.8%) |
| 8.5-switzerland-win | 39 (15.2%) | 46 (18.0%) | 34 (13.3%) | 44 (17.2%) | 104 (40.6%) |
| 8.6-germany-loss | 35 (13.7%) | 42 (16.4%) | 23 (9.0%) | 40 (15.6%) | 121 (47.3%) |
| 8.6-sparta-win | 110 (43.0%) | 131 (51.2%) | 89 (34.8%) | 136 (53.1%) | 176 (68.8%) |
| **All maps** | **331** (16.2%) | **396** (19.3%) | **256** (12.5%) | **400** (19.5%) | **745** (36.4%) |

Germany (`8.6-germany-loss`) is 35 of 256 against bombing rivals; with the old Search rivals it was 121 of 256 on this same 8.9.14 binary (144 on 8.9.13 in W0.4). The Germany map is where bombing rivals cost the seat most (-33.6 pp), then Switzerland 8.5 (-25.4) and Sparta 8.4 (-20.7). Celts and Mongols are near the floor (4 to 8%), so single-map deltas there are noise.

### Arms against the new control (paired, seat's win change in pp, [95%] {adjusted})

| Arm | Seat Δ pp | Reading |
| --- | --- | --- |
| Fighters: `air_defense=100`, seat only | **+3.2 [+1.6, +4.8] {+1.1, +5.2}** | Was -4.7 [-6.7, -2.8] against rivals that never bombed. Against bombing rivals Fighters help: wins rise on 7 of 8 maps (Sparta 8.4 +21 wins, Sparta 8.6 +21, Germany +7; Celts -4). The old reading was the benchmark bias. `air_capture_bundle` stays off (not measured here). |
| Modernization off, seat only | **-3.7 [-5.5, -1.8] {-6.0, -1.3}** | 8.9.14's seat gain holds against bombing rivals (W0.4: +2.4 [+0.1, +4.6]); it reads +3.7 now, inside the same interval. |
| Modernization off, rivals only | +3.4 [+1.4, +5.4] {+0.8, +5.9} | Rivals without refits are easier for the seat. The modernized rivals cost the seat 3.4 pp (W0.4: 6.5). |
| Old benchmark (`rival_search`), all seats | +20.2 [+17.9, +22.5] {+17.3, +23.2} | The size of the old bias: the seat won 36.4% against rivals that never bombed. |

### Second batch: Fighters in every role, the campaign planner, and the war suite (2026-10-28)

Same binary, salts, dice reseeding and pairing as above; the control was run again in each of the three batches and **all 2,048 games matched game for game each time** (8 repeats of 512). Adjusted intervals are Bonferroni over 6 comparisons (the four arms above are recomputed in `w20/gate-summary.json`; their figures move by about 0.1 pp). Cost: 0.13 server-hours for the air batch (about $0.08), 0.18 for the planner batch (about $0.12), and 0.01 each for the two war-suite runs.

| Arm | Wins of 2,048 | Seat Δ pp [95%] {adjusted} | Germany 8.6 wins of 256 |
| --- | --- | --- | --- |
| Control | 331 (16.2%) | - | 35 (13.7%) |
| Fighters, seat only | 396 (19.3%) | +3.2 [+1.6, +4.8] {+1.0, +5.3} | 42 (16.4%) |
| Fighters, rivals only | 305 (14.9%) | -1.3 [-2.9, +0.3] {-3.4, +0.9} | 15 (5.9%) |
| Fighters, all seats | 351 (17.1%) | +1.0 [-0.6, +2.6] {-1.2, +3.2} | 21 (8.2%) |
| Campaign planner, seat only | 355 (17.3%) | +1.2 [-0.5, +2.9] {-1.1, +3.4} | 39 (15.2%) |
| Campaign planner, rivals only | 351 (17.1%) | +1.0 [-1.0, +3.0] {-1.7, +3.6} | 35 (13.7%) |

Seat-only reads the AI's skill in the designer's seat. Rivals-only is the seat's win change when every rival uses the gene, so a negative number means the rivals became harder, which is good for the AI.

- **Fighters (`air_defense=100`) are positive in the seat and directionally positive as rivals.** In the seat they gain +3.2 pp (significant). As rivals they cost the seat 1.3 pp, in the AI's favour, but that interval includes zero [-2.9, +0.3]; all seats together read +1.0 [-0.6, +2.6]. Per map the rivals-only effect is mixed: Germany 35 to 15 and Japan 8.4 66 to 52 (rivals with Fighters beat the seat more), Sparta 8.4 and 8.6 the other way (48 to 57, 110 to 125).
- **The campaign planner (`operation_planner=1`, built from `claude/ai-operation-planner` merged onto master) is neutral in full games:** seat-only +1.2 [-0.5, +2.9], rivals-only +1.0 [-1.0, +3.0] (the positive rivals-only figure means the planner makes rivals slightly weaker, not stronger). W0.4 read -1.0 and 0.0 against Search rivals. Its value stays in the war suite (Swiss T220 23 to 65%), not in the benchmark. The planner binary is a scratch merge, `claude/planner-bench` (not for landing): the AI crate merged cleanly, the war tooling (`war_script`, `war_suite`, `war_trace`, `war_build`, the war-scenarios README) conflicted and was kept at master's version, and the branch's Swiss T180 replay fixture was dropped. Its control arm reproduced master's 2,048 of 2,048 games, so genes off equals master.
- **`air_capture_bundle` was skipped:** neither it nor `search_air_orders` is on master. The gene lives only on `astra/search-air-execution` (inside the W0.4 bench branch `claude/w04-bench`), and the strategy lab that carries it is 229 files. Testing it needs that branch landed or cherry-picked first. The seat-only, rivals-only and combined `air_defense + air_capture_bundle` arms are therefore not run.
- **War suite, seat on `air_defense=100` against off, 64 runs per scenario, 21 scenarios:** every scenario is identical (49% mean goal success, 9 of 21 at human parity, both runs). Golden Horde thin 87% (56 of 64; floor 40), Golden Horde 96%, Sparta First Blood 100%, German air T214 100%, T219 0%, early 23%. The gene changed no outcome in any scenario (not even the German air cases), so the suite shows only that it costs nothing there; I did not trace why (likely too short to buy and base a Fighter). The suite binary was built from the planner scratch merge with every gene off (the same code as master for these runs; the report records commit `5b1598c2`). Reports: `w20/war-ctl.json` and `w20/war-airdef.json`.

Germany 8.6 and the other maps, wins of 256 per arm:

| Map | Control | Fighters, seat only | Fighters, rivals only | Fighters, all seats | Campaign planner, seat only | Campaign planner, rivals only |
| --- | --- | --- | --- | --- | --- | --- |
| 8.3-switzerland-win | 11 | 17 | 6 | 16 | 15 | 5 |
| 8.4-sparta-win | 48 | 69 | 57 | 78 | 50 | 49 |
| 8.4-japan-win | 66 | 71 | 52 | 56 | 54 | 75 |
| 8.4-celts-win | 11 | 7 | 7 | 5 | 12 | 14 |
| 8.5-mongols-win | 11 | 13 | 12 | 11 | 9 | 3 |
| 8.5-switzerland-win | 39 | 46 | 31 | 27 | 49 | 43 |
| 8.6-germany-loss | 35 | 42 | 15 | 21 | 39 | 35 |
| 8.6-sparta-win | 110 | 131 | 125 | 137 | 127 | 127 |

### What this changes for the plan

- **The finish line's first measure needs re-reading.** "Hard playing the designer's seat wins 50% or more" was set against 32 to 39% measured with non-bombing rivals. Against the app's actual Hard the seat wins 16.2% on the same maps. Seven of the eight maps are the designer's wins (Germany is his loss), so the gap to the target is bigger than the plan said, and the Switzerland 8.3, Celts and Mongols maps are near unwinnable for the AI seat today (4% each).
- **Air genes are now judged on a benchmark where they matter.** `air_defense` is worth re-testing in the war suite and with rivals-only and combined arms.

## W0.4: candidates ranked (2026-10-27, bench `claude/w04-bench`)

*Measured with the old Search rivals (see W2.0 above): the control seat wins 39.8% here and about 16% against the app's plain Hard rivals, and the air defense reading (-4.7) flips to +3.2.*

**Only army modernization helps in both roles. Search air execution makes the benchmark's opponents far stronger, but that's a benchmark repair, not a better app AI. Everything else is neutral or worse in the gate.**

![Seat-only change per arm](w04-chart.png)

### Method

- **One binary, gene groups as arms.** `claude/w04-bench` is master `2e8594ece` plus the six branches below, all genes off. The binary with no parameters reproduces master: 128 of 128 games identical on every field master records (the bench only adds air telemetry fields). `cargo test -p legatum-ai -p legatum-core` passed on the box, 1,803 of 1,803 tests.
- **Branches merged:**
  - `claude/ai-operation-planner` (6b33c8896);
  - `astra/fix-staging-ring-r2`;
  - `astra/ai-local-opportunism`;
  - `astra/search-air-execution`, which already contains `astra/ai-air-defense-r2`.

  `astra/gold-usage-diagnosis` was already on master, so it needed no merge. None was dropped. The conflicts were in `military.rs`, `scripted.rs`, `plan.rs`, `diplomacy.rs`, `production.rs` and the rules doc, and each was resolved by keeping both sides. Where two genes act at the same spot, the operation planner goes first, because it claims its units before the other passes.
- **The gate:** the `astra/gate-noise-audit` frozen-control method.
  - **Games:** the eight human-seat maps, fresh salts 270001 to 270256 (2,048 games per arm), dice reseeded per salt.
  - **Pairing:** every arm is paired with the `ctl` arm of the same binary, through `tools/sim/quality_gate.py::compare` (equal-weight map strata, the larger of the stratified and salt-cluster standard errors).
  - **Repeat:** a second batch repeated the control, and all 2,048 game hashes matched.
  - **Intervals:** descriptive 95%, and simultaneous intervals Bonferroni-adjusted over the 16 comparisons (z = 2.95).
  - **Scripts:** `tools/ai-quality/w04_gate.py` and `w04_war.sh`. Arms are in `w04/arms.json` and `w04/arms2.json`, results in `w04/gate-summary.json`, and per-arm decision JSON in `~/LegatumRuns/ai-war-w0/w04-gate/`.
- **Signs:**
  - **Seat-only:** the candidate plays the designer's seat. Positive Δ means the AI is better.
  - **Opponents-only:** the candidate plays every rival, and Δ is still the seat's win change. Negative Δ means the rivals got harder, which means the AI is better.
- **Arms (intended "on" values from each branch's report):**

  | Arm | Genes |
  | --- | --- |
  | planner | `operation_planner=1` |
  | staging | `staging_redecide=1` |
  | local opportunism | `local_war_force=1`, `limited_war=100`, `exposed_flank=100` |
  | air defense r2 | `air_defense=100`, `air_capture_bundle=1` |
  | Search air | `search_air_orders=1` |
  | Gold | `purchase_savings_turns=6` |
  | modernization | `modernization_upgrade=100`, `modernization_return=100` |

- **Combined arm:** every group whose own arm was non-negative for the AI in both roles. That is modernization plus Search air. Search air is inert for the seat (the seat plays plain `ScriptedBrain`; the gene only acts in Search wrappers), so the combined seat-only arm would replay `modern_seat` exactly. Only its opponents-only arm was run.
- **War suite:** 22 cases x 64 runs, candidate on the seat. Search air is inert there too (plain Hard on both sides), so the combined war-suite row equals modernization.

### Ranking

| Rank | Arm | Seat-only Δ, pp [95%] {adjusted} | Opponents-only Δ, pp [95%] | War suite mean (parity) | Key war cases (control → arm) | Germany map, seat-only / opponents-only (control 144/256) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | **Army modernization** | **+2.4 [+0.1, +4.6]** {−1.0, +5.8} | **−6.5 [−9.0, −4.1]** | **47% (9)** | German air T214 0 → 100, German air early 4 → 23 | 150 / 115 |
| 2 | Search air execution | inert | **−22.4 [−24.4, −20.4]** | inert | none | n/a / 39 |
| — | Combined (modernization + Search air) | = modernization | **−25.6 [−27.9, −23.3]** | = modernization | as modernization | = 150 / 43 |
| 3 | Gold purchase savings | −0.1 [−2.3, +2.0] | −0.2 [−2.6, +2.2] | 41% (8) | no change | 145 / 153 |
| 4 | Operation planner (W1) | −1.0 [−3.0, +1.1] | 0.0 [−2.4, +2.4] | 45% (9) | Swiss T220 23 → 65, Walls Rise 81 → 100, T220-fresh 51 → 57 | 144 / 138 |
| 5 | Local opportunism | −1.3 [−3.7, +1.0] | −2.5 [−5.0, −0.1] | 33% (6) | **Golden Horde 100 → 0, thin 90 → 0**; Japan T142 78 → 92 | 122 / 135 |
| 6 | Staging ring r2 | −1.7 [−3.9, +0.6] | +0.3 [−2.1, +2.7] | 42% (8) | Japan T142 78 → 90 | 139 / 142 |
| 7 | Air defense r2 | **−4.7 [−6.7, −2.8]** {−7.6, −1.9} | −24.2 [−26.2, −22.1] | 46% (9) | German air T214 0 → 100 | 125 / 28 |

Control: 815 of 2,048 wins (39.8%). War-suite control: 41% mean goal success, 8 of 22 cases at human parity.

What the numbers mean:

- **Modernization (refits and return routing) is the one clear winner.** Each role helps the AI: in the seat it gains +2.4 pp (noninferior at the adjusted bound), and as rivals it costs the seat 6.5 pp. It also lifts the war suite most (41 → 47%), because Germany's modernized army holds against his bombers at T214.
- **Search air execution's −22 pp is mostly a benchmark repair.** The gate's rivals are Hard+Search, and Search never submitted air strikes. With the gene on, they bomb as plain Hard already does in the app. The gate today therefore measures every air idea against rivals that never strike. Air defense r2's seat loss (−4.7, Fighters bought against nobody) is that bias, and its −24 opponents-only figure comes from `air_capture_bundle` turning on the same Search air pass.
- **The planner is neutral in full games (−1.0 / 0.0)** and misses W1's seat mark of +3 pp, but it carries the war-suite gains (Swiss T220 23 → 65%).
- **Local opportunism and staging ring r2 are slightly negative in the seat.** Opportunism still breaks the Golden Horde attack check (0 of 64; the floor is 30%).
- **Gold savings is flat.** That makes three readings near zero or below (−0.6, −2.8 and now −0.1).

### Germany map (8.6-germany-loss)

- **Arms:** none of the seven arms recovers the 8.9.13 drop (360 → 281 of 512, about −15 pp). The best seat-only arm is modernization at 150 of 256 against the control's 144 (+2 pp).
- **Diagnostic arm `value0_all`** (`modernization_value=0` on every seat, the pre-8.9.13 valuation; 2,048 games) recovers it fully: Germany 144 → 186 of 256 (+16 pp), and overall +1.8 [−0.9, +4.4]. The same arm costs Switzerland 8.3 87 → 52. So the drop comes from 8.9.13's valuation gene on that one map, not from engine drift.
- **Unknown:** this arm changes every seat at once, so it does not say whether the drop comes from the seat or the rivals. A seat-only and an opponents-only `value0` arm would split it (about $1).

### Recommendation (nothing deleted yet)

| Branch | Keep or delete | Why |
| --- | --- | --- |
| `claude/ai-operation-planner` | Keep | W2's starting point. It carries the war-suite gains and is neutral in the gate. |
| `astra/search-air-execution` (contains air-defense-r2 and the strategy lab) | Keep | Land `search_air_orders=1` as a benchmark fix first, then re-freeze the control. Every air gene needs re-measuring against rivals that strike. `air_defense` and `air_capture_bundle` stay off. |
| `astra/ai-air-defense-r2` | Delete the ref | Fully contained in search-air-execution. |
| `astra/fix-staging-ring-r2` | Delete | Second round failed its gate (stop rule), and it reads −1.7 here. Keep the learnings. |
| `astra/ai-local-opportunism` | Delete | Seat −1.3, and the attack check fails (Golden Horde 0/64). The defensive reservations are the known cause. |
| `astra/gold-usage-diagnosis` | Delete the ref | Already on master. Leave `purchase_savings_turns=0`; three readings are flat. |

**Modernization genes (on master, off by default):** turning on `modernization_upgrade=100` and `modernization_return=100` is the one candidate that clears the gate in both roles. Run the release check with them on, then land them.

**W2 starts from:** master + modernization upgrade/return on + `search_air_orders=1` (as the new frozen control), with the planner branch on top.

## W0.5: baseline (2026-10-27, master `2e8594ece`)

- **Human-seat benchmark, today's Hard (against Search rivals that never bombed; superseded by W2.0, which measures 16.2% against the app's plain Hard rivals):** 815 of 2,048 games won (39.8%) on the eight maps, salts 270001 to 270256, dice reseeded per salt. The plan's finish line is 50%. The table below gives the per-map figures.

  | Map | Wins / 256 |
  | --- | --- |
  | Sparta 8.6 | 170 (66%) |
  | Germany 8.6 | 144 (56%) |
  | Japan 8.4 | 133 (52%) |
  | Sparta 8.4 | 122 (48%) |
  | Switzerland 8.5 | 106 (41%) |
  | Switzerland 8.3 | 87 (34%) |
  | Mongols 8.5 | 32 (13%) |
  | Celts 8.4 | 21 (8%) |

- **War suite, today's Hard** (22 cases x 64 runs, `~/LegatumRuns/ai-war-w0/w04-war/war-out/ctl.json`): 41% mean goal success, 8 of 22 at human parity.

  | Result | Cases |
  | --- | --- |
  | 100% | First Blood, Nicaea, Mongol Walls, Golden Horde, Swiss T210 |
  | Partial | German stand 93%, Golden Horde thin 90%, Walls Rise 81%, Japan v Sparta T142 78%, Swiss T220-fresh 51%, Swiss T220 23%, German air early 4% |
  | 0% | German air T214 and T219, Golden Horde hold, Chur ring, Mongol rush, Sparta all-out, Sparta walls, Swiss poachers, Swiss T180 and its replay |

- **Speed:** the speed section below (W0.3) is unchanged.
- **Cost of W0.4:**
  - **Spot:** 9.44 server-hours, about $6.14 at launch prices.
  - **Box:** about 45 slot-minutes on the shared box (tests, determinism samples, build, war suite).
  - **Wall time:** about 3 hours, 2 h 5 min of it the main batch.

## Speed budget (story W0.3, 2026-10-27, master a58e80856 = playtest 8.9.13)

The plan's budget: **whole-lobby turn p95 under 1 s on the designer's laptop.** This section says what 8.9.13 spends and how much is left for operation search and engagement search.

### What was measured

- **Game:** 8 empires on the Standard map (48x32), barbarians on, **every seat AI**, Hard difficulty (`ScriptedBrain`, what the app uses), or Hard+Search on every seat (`SearchConfig::in_app(8)`, the app's lobby option). Seeds 7000001 to 7000006, played from turn 1 in one process (so the WASM JIT is warm by T50) to turn 260. Four of the six seeds reach T250; the other two end earlier by victory (T164 to T193), so T250 is the long games only (64 turns Hard, 94 turns Search).
- **Native:** `--release`, one thread (`RAYON_NUM_THREADS=1`), because the app's WASM engine is single-threaded. **WASM:** the build the app ships (`wasm-release` profile: fat LTO, one codegen unit, `wasm-opt -O3`), run headless in Node 23.6 (V8). The two engines play the same game: the state hash matched on every one of the 2,858 turns compared.
- **Machine:** Apple M3 Max, 16 cores. Load average was 11 to 13 from other agents while this ran, with 2 of my games at a time, so wall times carry some noise. The smoke run of seed 7000001 (Hard, native, one game at a time) and the pooled run agree to within a few ms for plain Hard.
- **Windows:** turn +/- 10, so T50 is turns 40 to 60, and so on. Medians and p95 pool every seat-turn (or turn) of every seed in the window.
- **Columns:** *seat* is one AI seat's planning time for one turn (its Decisions view plus the brain), all 8 seats pooled. *Search seat* is only the seat-turns on which a search ran. *Lobby* is the whole End Turn (`Game::end_turn_json`: every seat's planning, rules resolution, history), with no human view. The human views are added below.

### Table (ms, median / p95)

| Config | Window | Seat (all seat-turns) | Search seat (turns it searched) | Whole lobby |
| --- | --- | --- | --- | --- |
| **Native, Hard** | T50 | 0.6 / 1.0 | n/a | 6.3 / 7.3 |
| | T150 | 1.6 / 3.5 | n/a | 16.5 / 19.9 |
| | T250 | 2.2 / 4.1 | n/a | 22.8 / 27.0 |
| **WASM, Hard** | T50 | 0.7 / 1.3 | n/a | 8.1 / 9.3 |
| | T150 | 1.8 / 4.0 | n/a | 19.1 / 23.8 |
| | T250 | 2.5 / 4.8 | n/a | 25.8 / 31.1 |
| **Native, Hard+Search** | T50 | 0.6 / 211 | 214 / 396 | 178 / 576 |
| | T150 | 1.7 / 659 | 723 / 1,153 | 19 / 1,684 |
| | T250 | 2.5 / 1,470 | 1,464 / 3,093 | 978 / 3,768 |
| **WASM, Hard+Search** | T50 | 0.7 / 275 | 291 / 524 | 225 / 763 |
| | T150 | 2.0 / 796 | 877 / 1,378 | 22 / 1,978 |
| | T250 | 2.7 / 1,765 | 1,711 / 3,320 | 1,161 / 4,916 |

Other facts from the same runs:

- **Plain Hard is far inside the budget.** A whole 8-AI lobby turn at T250 is 31 ms p95 in WASM (27 native), 3% of the second. WASM runs 1.1x to 1.3x native for plain Hard and 1.15x to 1.2x for search.
- **Hard+Search is far outside it, and gets worse as the game grows.** With every seat searching, a seat searches on about 10% of its turns (9.7% at T50, 8.1% at T150, 10.5% at T250), and one search costs 0.2 s (T50), 0.7 s (T150) to 1.5 s (T250) of native time, 0.3 / 0.9 / 1.7 s in WASM. Up to 3 seats search on one turn at T50 and T250, 4 at T150. The whole-lobby turn is over 1 s on 4% / 24% / 52% of WASM turns at T50 / T150 / T250. The largest single seat-turn in the run was 16.1 s native and 7.3 s WASM (all windows).
- **The `SearchConfig::in_app` comment ("keeps a search turn near a second", measured max 690 ms for 8 players) does not hold late.** It was measured on the first turns of a game (`app_search_time`, default 60 turns); search cost scales with game size.
- **Seed spread is wide for Search:** per-seed lobby p95 at T250 runs from 2.3 s to 5.0 s in WASM (2.8 s to 4.1 s native), and at T50 from 0.25 s to 1.3 s.
- **A designer's lobby has 6 AIs, not 8.** Taking the lobby turn minus two seats' planning (a proxy for the two human seats) changes plain Hard by about 5 ms and Hard+Search p95 by up to 30% (WASM T150: 1,392 against 1,978 ms; no change at T250), still far over.

### The two human seats and the follow-up work

The tables leave out what a human seat adds. In these all-AI runs the human-view cost shows only as `view(0)`, the full view JSON of seat 0 (mean 2.4 ms native, 3.0 ms WASM at T250 in Hard games; 3.5 and 4.4 ms in Search games). `turn-latency.md` measured a human in seat 0 and found the End Turn work around it (a view and event visibility before and after the turn, hashing and JSON) was about 8 ms in a late native turn before the 10-24 perf work, and the UI parses about 660 KB of JSON (about 5 ms). I reserve **20 ms for two humans** in the arithmetic below. This is an assumption, not a measurement of a two-human lobby.

### Scaling to the designer's laptop

This machine is an M3 Max, and its P-cores are among the fastest M-series cores. For a base M1 or M2 laptop, multiply the times by about 1.15x (M2) to 1.3x (M1); the app also runs the engine in WKWebView (JavaScriptCore), not Node's V8, whose WASM speed was not measured here (the plan forbids Playwright WebKit on this Mac). A thermally limited fanless Air can be slower still under a long search. I use **1.3x** as the planning factor, and the headroom below is given both as measured and as M1-equivalent.

### Headroom for operation search and engagement search

The budget is 1,000 ms of whole-lobby turn at p95. Today's plain Hard play, with the two humans reserved, costs:

| | T150 | T250 |
| --- | --- | --- |
| WASM: 8 Hard AI lobby p95 + 20 ms humans | 24 + 20 = 44 ms | 31 + 20 = 51 ms |
| Native: the same | 20 + 20 = 40 ms | 27 + 20 = 47 ms |
| **Left for extra search in one turn, WASM** | **about 955 ms** | **about 950 ms** |
| **Left, native** | **about 960 ms** | **about 950 ms** |
| Left, WASM scaled to an M1-class laptop (divide by 1.3) | about 735 ms | about 730 ms |

How that splits per seat, at T250 in WASM:

- **If all 6 AI seats run the new search on the same turn:** about 950 / 6 = **155 ms per seat per turn** (M1-class: about 120 ms). The same in native: about 160 ms.
- **If searches are staggered so that one seat runs the heavy part on a turn** (the `stagger` option already spreads first reviews): up to **about 900 ms for that one seat**, which means a seat can afford one big search every few turns, but nobody else may search heavily on the same turn. Because the p95 allows 5% of turns over, the real cap on rare spikes is looser, but a designer who clicks End Turn and waits 3 s on one turn in ten is the experience being budgeted against.
- **Per-seat average over many turns:** about 950 ms / 6 seats = 155 ms of extra search per seat-turn, shared however the stagger allows. Hard+Search today averages **203 ms per seat per turn at T250 in WASM** (170 native), so even its average is above this, and its p95 lobby is 5x the budget.

What this means for the plan:

1. **The current Hard+Search config does not fit the 1 s budget** at T150 (2.0 s p95 WASM) or T250 (4.9 s), on either engine. Its search has to be at least about 5x cheaper at T250 and 2x at T150, or be bounded by a per-turn time budget (stop a search at its share of the ~950 ms).
2. **Operation search plus engagement search must share the same ~950 ms**, not each get it. A split such as 600 ms for the campaign-level (operation) search, run for at most one or two seats per turn, and 300 ms for engagement search across all seats' battles on that turn, fits with margin. Engagement (battle) search only runs when armies are adjacent, so it can be triggered rather than every turn.
3. **The cheap path has plenty of room:** plain Hard's total is 5% of the budget, so deterministic improvements to the scripted planner (the campaign planner, increment 1) are free in speed terms up to about 30x their current planning cost.
4. **A search that can't fit should move off the turn's critical path:** run it for AI seats during the human's thinking time (a Web Worker in the app), where the WASM engine has seconds to spare. That is not done today and would need engine work (a state clone sent to a worker, with the result applied next turn).

### Reproduce

```
# build (about 1 min native, 4 min WASM with wasm-opt on this Mac), run 6 seeds x hard,search x native,wasm
# (2 games at a time under nice, about 20 minutes), summarize, and check native = WASM hashes:
tools/perf/ai-budget.sh
# or part of it:
SEEDS="7000001 7000002" TURNS=100 MODES=hard SKIP_BUILD=1 tools/perf/ai-budget.sh
# one game by hand:
cargo build --release -p legatum-wasm --features seat-timing --example ai_budget
RAYON_NUM_THREADS=1 target/release/examples/ai_budget 7000001 260 search > native.csv
wasm-pack build crates/wasm --target nodejs --profile wasm-release --out-dir ../../target/wasm-budget -- --features seat-timing
node scripts/perf-wasm-budget.mjs target/wasm-budget 7000001 260 search > wasm.csv
tools/perf/budget_summary.py native.csv
tools/perf/budget_summary.py --same-hashes native.csv wasm.csv
```

The raw per-turn CSVs (one row per turn: whole-lobby ms, human-view ms, state hash, each seat's planning ms, a bitmask of the seats that searched) are in `~/LegatumRuns/ai-war-w0/budget/`. The `seat-timing` Cargo feature of `legatum-wasm` only records each seat's planning time; a build without it plays and performs identically.

## Siege built in from the Industrial era (designer, 2026-10-28; merged 2026-10-29)

Rule change: Industrial and later land units ignore every wall tier and can't buy Siege (`data::built_in_siege`, `data::siege_cover`).

- **War suite** (Hard, 64 runs, 28 scenarios): goal success 43% → 46%, human parity 11 → 12.
  - Swiss T220: 23% → 100%. Swiss T220 fresh: 53% → 100%. Japan vs Sparta T142: 82% → 89%.
  - German air early (a defence): 28% → 0%, because Germany's walls no longer slow Japan's Modern army.
  - Pre-Industrial scenarios are unchanged.
- **Human-seat gate** (AWS, 2,048 games, seeds 270001+):
  - Pairing: each game against the same seed in the W2.1 gate's `door_all` arm (today's default), so the only difference is the rule.
  - Result: Hard in the designer's seat 315 → 329 wins, +0.68 pp [−1.07, +2.43], `ship-monitor`. Neutral in full games.
  - Files: `docs/sim-results/ai-war/siege-industrial/`.
  - Cost: 0.05 server-hours, about $0.03.

### W2.2 full human-seat gate (AWS, 2026-10-29, on master with Siege built in)

2,048 games per arm, seeds 270001+, `docs/sim-results/ai-war/w22/`. Control: the Hard seat wins 329. That matches the Siege gate's 329 on the same seeds, so the binary is consistent. The delta is the seat's win rate, so negative means the AIs got stronger.

| Arm | Δ seat win (pp) | 95% desc | Decision |
|---|---|---|---|
| `war_reaim` 1, rivals only | +0.34 | [−0.74, +1.42] | noninferior |
| `war_reaim` 1 + `peace_own_danger` 2, rivals only | **+2.10** | [+0.38, +3.81] | noninferior |
| Same, all seats | +1.71 | [−0.15, +3.57] | noninferior |

**Verdict: keep both genes off.**
- The fixtures improved (Chur T137 3% → 23%; Hard-in-seat from T137 44% → 23%), but in full games the persistent rivals did slightly *worse* against the seat (+2.1 pp, descriptive interval excludes 0).
- Likely cause: wars that no longer end cost the AIs more than they take, and Peace refusals against the seat's counter-pressure hurt them.
- Next round, per the agent's note: judge "losing this war" by units lost to that enemy, and only persist when the leader is actually hollow (rec 2, the leader read).

Cost: 0.14 server-hours, about $0.09.

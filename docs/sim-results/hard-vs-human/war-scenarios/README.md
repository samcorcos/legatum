# War scenarios: campaigns from the user's playtests

The user's idea (2026-10-11): "the goal is to take these two cities in the amount of time given that you know that it is achievable because I did it as a human." Each scenario starts at the moment a real campaign began in one of his games, with a goal, a turn limit and his own result. The AI plays his seat against the same opponents (Hard, or Hard + Search where his game had it), many seeded runs per scenario. Runs are 8 to 27 turns, so the whole suite costs about a minute of cloud compute.

Two scenarios (added 2026-10-17) work the other way round: the AI plays the **attacker** in a game the user was defending, and the "result to beat" is what the AI itself did to him. They are the **attack regression check** for the AI's best offensive so far (below).

**2026-11-04 audit:** The Golden Horde thin check is 0/64 on master snapshot `87e5f43e` against its 60% floor. [The bisect and plan trace](../../ai-war/golden-regression-1104/README.md) show a stronger Swiss defender and an archived Mongol army that becomes wholly unsupplied on turn two under the newer rules. The old floor is uncalibrated for this saved state; the audit proposes a new current-rules fixture before setting a positive floor.

**Renamed 2026-11-08:** `war-japan-chur-ring` is now `war-india-chur-ring` and `war-japan-sparta-t142` is now `war-india-sparta-t142` (Japan is India). The dated results below keep the old ids as they were run; the files in `scenarios/war/` use the new ones.

## The scenarios

| Scenario | From | Start | Goal (turns) | The user's result |
|---|---|---|---|---|
| The Mongol Storm (`war-mongol-rush`) | Playtest 6, Mongols, 8 empires | T25, 1 city, 2 Archers + 1 Spearman, at Peace | Byzantium and Athens hold no cities (27) | Met in 26 turns (both gone by T51), 1 unit lost |
| Sparta's First Blood (`war-sparta-first-blood`) | Playtest 8.6, Sparta, 4 empires | T29, 1 city, 5 Spearmen, at Peace | Germany holds no cities (12) | Met in 8 turns, no losses |
| Sparta Against All (`war-sparta-all-out`) | Playtest 8.6, Sparta | T65, 7 cities, 7 Spearmen + Archer | Japan and Byzantium hold no cities (20) | Met in 20 turns (the T79 attack took Byzantium's last city on T84), 2 units lost |
| Walls of Keystone (`war-sparta-walls`) | Playtest 8.4, Sparta | T51, 5 cities, 7 Spearmen, 408 Gold | Capture Anvil and Keystone (12) | Met in 11 turns; Keystone's Walls rose mid-siege, 1 Spearman lost |
| The Swiss Poachers (`war-swiss-poachers`) | Playtest 8.5, Switzerland | T46, 3 cities, 1 Spearman, 3 Scouts, 446 Gold | Own 7 cities (14) | Met in 10 turns: two Settlers stolen in the turns war was declared |
| Behind the Walls (`war-mongol-walls`) | Playtest 8.5, Mongols | T54, 5 cities (4 walled), 1 Spearman, Sparta at war with 7 units next door | Hold every city (20) | Held all, no losses |
| Germany's Last Stand (`war-german-stand`) | Playtest 8.6, Germany | T100, 5 walled cities, 1 Spearman, the Mongols at war with 11 units | Hold every city (24) | Held all 24 turns (lost Kiln on the 25th) |
| Sparta's Nicaea Strike (`war-sparta-nicaea`) | Playtest 8.9.2, Sparta, 8 empires | T18, 1 city, 4 Hoplites one to three hexes from Nicaea, at Peace | Take Nicaea, Byzantium's only city (8) | Met in 4 turns (T22), no losses |
| Germany before the Bombers land (`war-german-air-early`, 2026-10-27) | Playtest 8.9.11, the German AI (Hard) against his recorded orders | T210, 6 walled cities, 8 land units, 189 Gold, at war with Japan since T172 (4 Bombers, struck German units since T173); no Fighter | Keep at least 4 of the 6 cities (12) | All six lost (T211, 214, 216, 218, 219, 221), the third on T216 (turn 7); Germany eliminated, no Fighter ever built |
| Germany under the Bombers (`war-german-air-t214`, 2026-10-27) | Playtest 8.9.11, **the German AI** (Hard) against **his recorded orders**, 5 empires | T214, 5 walled cities, 5 weak land units and 1 Bomber, 134 Gold, at war with Japan (4 Bombers, 2 Autonomous Armour) since T172 | Keep at least 3 of the 5 cities (8) | **The AI's result in his game:** all five lost (T214, 216, 218, 219, 221), Germany eliminated, no Fighter ever built |
| Germany's last two cities (`war-german-air-t219`, 2026-10-27) | the same game and script, the German AI | T219, 2 walled cities (Hamburg, Essen), no army, 130 Gold, no aircraft | Keep at least 1 of the 2 cities (3) | Hamburg lost on T219 and Essen on T221, Germany eliminated |
| Japan against the veteran (`war-japan-sparta-t142`, 2026-10-27) | Playtest 8.9.10, **the Japanese AI** (Hard) against Hard Sparta, 4 empires | T142, 10 cities (Kyoto 20 HP, no Walls), 13 combat units, 27 Gold, at war with Sparta since T42 (12 cities, 4 combat units, 2,457 Gold) | Keep at least 9 of the 10 cities (30), all 30 turns always played | Kyoto lost on T158 (the 17th turn), Edo on T169; **30 Japanese combat units dead for none of Sparta's** in the 30 turns, 40 for 2 by T184 |
| The Golden Horde comes (`war-golden-horde`) | Playtest 8.9.2, **the Mongol AI** (Hard), 8 empires | T136, 5 cities, 16 soldiers, at war with Switzerland since T78 | Take Aarau, the walled Swiss city six hexes from Khanbaliq (14) | **The AI's result in his game:** taken on T147, the 12th turn, 7 combat units lost |
| The Golden Horde comes, thin treasury (`war-golden-horde-thin`, 2026-10-18) | the same start, the Mongol AI (Hard) | T136, 5 cities, 16 soldiers, at war with Switzerland since T78; **Swiss treasury 500 Gold** | Take Aarau (14) | The attack regression check (below): the user's result is the same game |
| The Tepid Swiss, turn 180 (`war-swiss-tepid-t180`, 2026-10-27) | Playtest 8.9.10, **the Swiss AI** (Hard), 8 empires | T180, 3 cities, 16 land units 2 to 6 hexes from Athens, at war with Athens since T177 | Take Athens, the walled border city four hexes from Winterthur, empty at T180 (14) | **The AI's result in his game:** no city taken in 47 turns of war, 8 combat units lost |
| The Tepid Swiss, turn 210 (`war-swiss-tepid-t210`) | the same game, the Swiss AI (Hard) | T210, 10 land units 3 to 5 hexes out, 2 Bombers; Athens at 0 HP, one Spearman | Take Athens (8) | Not met in 14 turns: the city healed to 200 HP by T220, 1 unit lost |
| The Tepid Swiss, turn 220 (`war-swiss-tepid-t220`) | the same game, the Swiss AI (Hard) | T220, 9 land units 3 to 5 hexes out, 5 Bombers; Athens at 200 HP, one Spearman | Take Athens before Athens wins on VP at the end of T223 (4) | Not met: the Bombers cleared the city on T222, nobody walked in |
| The Ring around Chur (`war-japan-chur-ring`) | Playtest 8.9.12, **the Japanese AI**, 6 empires (that game ran every AI at Normal by mistake; here Hard) | T143, 9 cities, 20 soldiers (13 within 8 hexes of Chur), at war with Switzerland since T95 | Take Chur, the walled Swiss city behind the mountain (20; no "hold every city" condition) | **The AI's result in his game:** 21 turns ringed around Chur, no attack, Chur at 200 HP throughout |
| The Golden Horde comes, hold (`war-golden-horde-hold`) | the same start, **Switzerland** (the user's seat) | T136, 5 cities (4 walled), 1 Crossbowman and 2 Scouts, 2,235 Gold | Hold all five cities (14) | Lost Aarau on the 12th turn |
| Sparta next door (`war-sparta-next-door-t101`, 2026-10-27) | Playtest 8.9.13, **the Spartan AI** (Hard) against **his recorded orders** (Athens), 5 empires | T101, 6 walled cities and 9 soldiers; Athens 6 cities, 2 military units, 318 Gold, at Peace | Capture an Athenian city (20) | **The AI's result in his game:** declared on T101, no attack on an Athenian city in 29 turns at war, Peace T130 |
| The Mongols against the Bomber (`war-mongol-bombers-t150`) | the same game, **the Mongol AI** (Hard) against his orders | T150, 6 cities (5 walled), 9 Medieval and Renaissance soldiers, 17 Gold, at war with Athens since T133 | Keep at least 4 of the 6 cities (18) | All six lost (T153, 159, 160, 163, 165, 167), eliminated; the third on the 11th turn |
| Break the hold (`war-break-the-hold-t168`) | the same game, **the Byzantine AI** (Hard) against his orders | T168, 9 cities, 13 soldiers, at war with Athens since T166; Athens 16 cities, 3 military units, 10 VP | Capture an Athenian city (10) | **The AI's result in his game:** no attack, Peace T172; he won on T177 |
| Break the warband hold (`war-celts-wild-hold-t150`, 2026-11-03) | Playtest 8.9.25, **the Byzantine AI** (Hard) against **his recorded orders** (the Celts), 6 empires | T150, 8 cities, 11 military units, at war with the Celts; the Celts on exactly 10 VP, 2 of them Raise the Warbands (one camp per city, most unguarded) | Stop the Celts winning (`StopWin`, keep and win, 10) | **The AI's result in his game:** at war throughout, 2 to 7 units within 3 tiles of his camps, no camp cleared; he won on T159. Gene `contest_hold` off 20%, on in the seat 82%, in every AI seat 100% of 64 runs (`docs/sim-results/ai-quality/contest-hold-1103/`) |
| Japan before the Hoplites (`war-japan-hoplites-t43`) | Playtest 8.9.13, **the Japanese AI** (Hard) against his orders (Sparta) | T43, the first turn of the war: 3 cities, 2 Spearmen, 42 Gold; 7 Hoplites on the border | Keep at least 1 of the 3 cities (14) | All three lost (T44, 49, 56), eliminated |
| Germany's Berlin (`war-germany-berlin-t65`) | the same game, **the German AI** (Hard) against his orders | T65, 3 walled cities, 3 Spearmen, an Archer, 149 Gold; 4 Hoplites at Berlin | Keep at least 2 of 3 cities (28) | All three lost (Berlin T67, Essen T86, Cologne T92), eliminated |
| Athens against two Pikemen (`war-athens-two-pikemen-t97`) | the same game, **the Athenian AI** (Hard) against his orders | T97, 8 cities, 6 soldiers, 111 Gold; his army is 2 Siege Pikemen and an Arquebusier | Keep at least 5 of 8 cities and kill both Pikemen (29) | Five starting cities lost (T99 to T125), a sixth founded and lost; both Pikemen alive |
| Athens against two Pikemen, late (`war-athens-two-pikemen-t112`) | the same | T112, 6 cities, 3 soldiers, 64 Gold, after three cities have fallen | Keep at least 4 of 6 cities and kill both Pikemen (14) | Three more lost (T118, 121, 125) |
| The Mongols at Chur (`war-mongols-chur-t137`, 2026-10-29) | Playtest 8.9.15, **the Mongol AI** (Hard, carried memory) against **the designer's recorded orders** (Switzerland), 4 empires | T137, 11 cities, 12 military units, 145 Gold, at war with Japan; Switzerland 8 cities, 1 Arquebusier, 3,592 Gold | Capture a Swiss city (20) | **The AI's result in his game:** declared on T137, aimed at Chur, ended the campaign as an insufficient army on T139 after the designer bought one Arquebusier there, Peace T141 |
| The Mongols at Solothurn (`war-mongols-solothurn-t158`) | the same game | T158, 11 cities, 15 military units with a Field Cannon, at war with Japan; Switzerland 4,797 Gold | Capture a Swiss city (14) | Declared on T158, the campaign ended on T160 after the designer bought a Field Cannon in Solothurn, Peace proposed T161 mid-campaign and accepted T162 |

## Baseline, 2026-10-11 (commit f84d79bc, 64 seeded runs per scenario, one spot server, about $0.02)

![War scenarios chart](chart.png)

| Scenario | User | Hard: met | Hard + Search: met | Predicted (written first) | What the AI did |
|---|---|---|---|---|---|
| The Mongol Storm | 26 turns | 0% | 0% | 8% | Hard declares in every run (median 3 turns in) but takes a city in 1 of 64 runs and sits at war 12 turns without a battle; Search declares in 52 of 64 and takes none. It steals a Settler about once a run. |
| Sparta's First Blood | 8 turns | **0%** | **100%** (median 9 turns) | 40% | Hard never declares: it marches all five Spearmen home to Haven and fortifies. Search declares on the first or second turn and takes both cities every time, one turn slower than the user. |
| Sparta Against All | 20 turns | 0% | 0% | 15% | Both take about one city a run (Coda, or Byzantium's new one); Tessitura (walled, four Archers) never falls. Hard never declares; Japan declares on it instead, and in all 8 traced runs Hard proposed Peace to Japan the next turn with seven Spearmen against four Archers. Search declares at once. |
| Walls of Keystone | 11 turns | 0% | 0% | 35% | Hard declares at once and takes Anvil in 45 of 64 runs, never Keystone. Search declines the war in 62 of 64 runs. |
| The Swiss Poachers | 10 turns | 1% | 3% | 15% | Declares at once and steals 1.2 to 1.3 Settlers a run (the user stole 2 by the same point), but never gets to seven cities: it does not found fast enough. |
| Behind the Walls | held 20 | 95% | 95% | 70% | Sparta's AI rarely presses the attack, as in the user's game. |
| Germany's Last Stand | held 24 | 98% | 98% | 55% | The Mongol AI never takes a walled German city. |

Suite score: **Hard 27%** mean goal success, **Hard + Search 42%**; 2 of 7 at human parity for each (only the two defences). The human met all seven.

What this says:
- **The gap is offence.** On every attack scenario but one, both brains fail a goal the user met, usually because they never commit: Hard won't start the first Spartan war with an army that wins it every time once declared, Search turns down the Keystone war that Hard at least half-wins, and both make peace or idle at war once a war is on. This is the user's "the AIs are at war with me, but they aren't really attacking me" in measurable form.
- **Search fixes the clearest case** (First Blood 0% to 100%) and breaks another (Keystone declarations 64 to 2 of 64): its rollouts value a short, sure war and undervalue a costly one. That is a lead for the AI work, not a verdict.
- **Defences are near ceiling.** They are regression checks, not hill-climb targets: the attacking AIs rarely press, as in the user's games. A harder defence needs an attacker that commits, which is the same fix.
- Predictions were too optimistic on attack (written expectation 8 to 40%) and too pessimistic on defence (55 to 70%).

No impossible states or panics were found in 896 runs. The first bug-hunt pass flagged units "standing in foreign land at Peace"; those are legal (land closes under them when borders grow or Peace is signed, and a stranded unit may walk out), so the check now reports only a unit stepping from open land into land closed to it at both ends of the turn.

## The Golden Horde comes: the first AI offensive that beat the user (2026-10-17)

The user, playtest 8.9.2 (Switzerland, Hard, T149): "The Mongols had an actually competent attack... I was unprepared for such a large assault and they took one of my cities... I actually lost a city that I tried to defend." (The save, `~/LegatumRuns/bug-saves/p892-mongol-assault-t149-1416.legatum`, has the user as Switzerland/Switzerland; Aarau is the city that fell. The replay on today's engine matches all 150 recorded state hashes.) Tools: `crates/sim/examples/p881_trace.rs` (every AI's plans turn by turn) and `p892_assault.rs` (the armies, the city and every battle between two players); `war_trace.rs` plays one run of any scenario the same way.

**What the Mongol AI did, in order**

| Turn | What happened |
|---|---|
| 77 to 78 | The user founded Aarau six hexes from the Mongol city of Khanbaliq, the Mongols' nearest. The Mongol AI declared war the next turn (Campaign plan worth 2,878, "capture at 37%") with 10 soldiers. |
| 79 to 85 | The first strike failed: one Phalangite hit the city alone while the shooters trickled in, the Swiss bought Walls in a turn, and the city's own fire killed a unit a turn. 4 units lost, campaign dropped on T80. |
| 80 to 138 | **No battle for 58 turns, and no peace.** The plan layer said Peace (value 0) because no siege could be forecast. The army never disbanded or went home: it stayed around Khanbaliq and the road to Aarau, 4 to 8 hexes away (inside the six-turn march window), and grew from 10 to 17 units. The shooters were modernised: Composite Archers gave way to Crossbowmen (the first by T90, four by T138) and an Arquebusier appeared on T132. |
| 101 to 143 | Aarau had **no garrison at all** (the Swiss army was one or two units, and the Swiss held 2,235 Gold unspent). |
| 139 to 143 | A Campaign candidate appeared (T139, "capture at 22%") and flickered off and on until T143 ("capture at 44%"): the forecast needed four or five gunpowder-era shooters near the wall. |
| 141 to 146 | The siege: three or four shooters (Crossbowmen and the Arquebusier, about 13 damage a shot) fired on Aarau from two hexes every turn, from T141, 200 HP to 0 in six turns, while the melee stayed out of the way. Walls then a Castle then a Star Fort rose mid-siege and the city's fire killed a shooter a turn (60 to 70 damage each); the AI replaced them and kept firing. Reinforcements: an Arquebusier on T144, an Assault Trooper on T145. |
| 146 to 147 | At 0 HP two Phalangites struck the garrison (Arquebusier 100 to 52); next turn three shooters killed it and a Phalangite walked in. Seven combat units lost in all. |

**The AI features responsible** (all generic Hard, none Mongol-specific; Mongols's courier mobility plays no part at war):

1. **The city-assault coordinator** (`city_assault.rs`, from 2026-10-01). It acts on any visible enemy city with a melee unit of ours within four hexes, whatever the plan says: shooters take the ring at their range (best exposure, hills first), melee holds at ring 2, and the wing goes in only when a four-turn projection (`city_assault_succeeds`) says the city falls. That is why the shooting began on T141 while the plan flickered back to Peace.
2. **The goal-driven plan layer** (`plan.rs`, 2026-10-04): a Campaign candidate exists only when the six-turn strike force forecasts a siege, priced by calibrated capture odds. It kept the war open and the army home for 58 turns and flipped on when the shooters upgraded.
3. **The standing army kept at the front** (it never disbanded) and modernised; once a Campaign is priced the units within four hexes of the target are committed (`campaign_force`) and the rest march (`strike_force_within`). The plan flickering on and off is the gap between the six-turn forecast and the four-hex commit.
4. **Capturing with melee only after the shooters have killed the garrison** (a shooter never takes an empty city under Hard).
5. **Not responsible, tested:** `troop_runway`, `restless_far_war` (Sparta-only), `capture_economy_percent`, `price_bought_walls`, `staging_damage_percent` (50 and 200) and `war_commit_ratio` (250) all give run-for-run identical results on this scenario (128 runs). The one personality knob that matters is Mongols's `aggression`: 0 halves the success rate (25%), 100 does not raise it (50%).

**Measured (2026-10-17, plain Hard, 64 seeded runs, one spot server, about $0.02):**

| Scenario | Hard | Hard + Search | The user's game |
|---|---|---|---|
| The Golden Horde comes: take Aarau in 14 turns | **43%**, median 8 turns (46% on 06d93758, 50% on 128 runs) | **1%** (06d93758) | taken on turn 12 |
| The Golden Horde comes, hold all five cities | 56% (53% on 06d93758) | 54% (06d93758) | lost Aarau on turn 12 |

- **The AI attacker succeeds about half the time** against a Hard Swiss defender, and faster than in the user's game (median 8 turns against 12): a fresh brain starts the siege on turn 2 where the user's Mongols took until T141. The Swiss AI spends its 2,235 Gold, but mostly on buildings (an Archive, Walls, a Workshop) and one Crossbowman at a time, so it holds only about half the time too. The two scenarios are the same games seen from either seat, so 43% attack and 56% hold agree within noise (they differ because "hold" fails on any lost city and "take Aarau" requires the Mongols to keep their own).
- **Hard + Search never gets there (1%, measured one merge earlier: the spot quota was full when the merged build was due).** Its rollouts turn the war down or sue for peace: 6 of 64 runs never declare and the rest do not commit. The same weakness as Keystone and the Mongol Storm, on the position where plain Hard does best.
- **Attack regression check:** `tools/sim/war-scenarios.sh --check` (Hard, 64 runs, only this scenario) exits 1 below 30%, more than two standard errors under the baseline. Compare a candidate paired with `war-scenarios-report.py --compare`; run `--runs 128` for a tighter reading.
- **Predictions (written before measuring):** attack 30%, hold 70%. The attacker did better than predicted and the defender worse.

**Does the pattern transfer to the other AIs?** Not in this position. The same T136 start, with each other seat as the attacker against its current enemy (64 Hard runs each, took at least one city in 14 turns; a throwaway variant per seat, not committed): Japan, Germany, Celts and the other Switzerland AI against Sparta 0% each, the Swiss AI against the Mongols 0%, Sparta against any of its four enemies 0% and the Mongols against Switzerland 46%. Traced (`war_trace`, run 0): the Mongols' target was a Campaign priced 1,500 to 5,000 ("capture at 30 to 58%"); the AIs at war with Sparta (the VP leader, eight cities, seven walled, 13 soldiers) all sat in a Contain plan worth about 100 (92 to 133) with no siege forecast, so nothing marched; Sparta itself ran a "forced expedition" Campaign valued at 0 on Germany's Berlin, then a 22% Campaign on Celts's Alesia, and lost three soldiers (13 to 10) without taking anything, though its genes took four weaker cities earlier in the game (T47, T57, T105, T125). Three ingredients were present together only for the Mongols: a city six hexes from one of their own with the army already around it, nothing inside it, and shooters a tier ahead of its walls. See `docs/sim-results/ai-quality/ideas.md` (#15) for what to try.

## How the start states were made (faithful replays, then today's rules)

Every state is the real game state, not a reconstruction: `tools/scenarios/extract-war-states.sh` replays each save with the engine build it was recorded on (a worktree at the save's engine commit), checks every turn against the hash recorded in the save (0 mismatches in all six games), and writes the state before the campaign's first turn. `legatum-sim war-build` then loads it into the current engine:

- old technologies map to the seven-era tree through the save migrations, and old units keep their ids, which are now the A2 units (a Soldier is a Spearman, a Ranger an Archer, a Trooper a Pikeman);
- today's rule switches (VP v2, crowns, overreach, decadence) apply;
- **decadence is a reconstruction**: the old engines kept no Power history, so each empire's 30-turn window is filled with its current yields. A leader therefore starts with today's decadence (Sparta in the all-out war: 37% off every yield), which the user never faced;
- playtest 6 predates the 2026-10-03 diplomacy simplification: its seven Pacts became Peace and pending Pact offers were dropped;
- the user's units lose the orders he gave them. Byzantium is now the bought-peace faction, so it can buy 10 turns of peace where the user's Byzantium could not.

The two Golden Horde scenarios come from a game recorded on today's rules (playtest 8.9.2), so nothing is migrated: the state is the replay at turn 136 as it stood (hash-checked on all 150 turns), the Power history and decadence are the real ones, `legatum-sim war-build` only clears the seat's orders and makes every seat a Hard AI. Every AI's memory (threat model, plans, paused campaigns) starts fresh at turn 136, as in every scenario, which is why the siege starts earlier here than in the user's game. The user's seat (Switzerland) is a Hard AI with his 2,235 Gold, not a player who leaves a city empty.

So the human result in the older scenarios is a target set under older rules: unit strengths, Walls and decadence all differ. It shows the goal is achievable from the position, not that it is exactly as easy now.

## Air-stage and exchange fixtures (2026-10-27)

Three scenarios from the designer's saves `japan-win-bombers-20261026` (engine d1233d48, 234 turns replayed with 0 hash mismatches) and `sparta-crushing-japan-20261025-0841` (engine 8e9433e5, 184 turns, 0 mismatches), built by `tools/scenarios/extract-war-states.sh` (entries `jp` and `sj`) and `legatum-sim war-build`. Every seat is a Hard AI, **including the designer's**, except where a script replays his orders (below). Goals and deadlines:

| Scenario | Goal | Deadline and why |
|---|---|---|
| `war-german-air-t214` | Keep at least 3 of 5 cities (he took all 5) | 8 turns, T214 to T221: the turns he needed to take all five and eliminate Germany |
| `war-german-air-t219` | Keep at least 1 of 2 cities (he took both) | 3 turns, T219 to T221: Hamburg fell on turn 1 and Essen on turn 3 |
| `war-japan-sparta-t142` | Keep at least 9 of 10 cities; always play all 30 turns | 30 turns, T142 to T171: the window in which he took Kyoto (turn 17) and Edo (turn 28). Kyoto, with 20 HP and no Walls, falls on turn 3 in every run, so "hold all ten" was dropped (round 1). With no early stop, both sides' losses are counted over the same 30 turns |

### Round 1: all seats Hard AI (commit 7d8a47d7, 64 runs each). The predictions were written first.

| Scenario | Goal then | Predicted | Measured | Why it was not a usable fixture |
|---|---|---|---|---|
| `war-german-air-t214` | keep 3 of 5 | 70% (range 50 to 90%): Japan's Hard AI strikes with every Bomber but plans no bomb-then-capture, and is also at war with Sparta and Athens; 30% lose no city | **100%** (no city lost in 64 of 64) | Japan's AI bombs (run 0: two Bombers hit Cologne on T214 for 102 and 87, two of his four first strikes) but never walks a unit into the stripped city |
| `war-german-air-t219` | keep 1 of 2 | 90% (range 70 to 100%), 60% keep both | **100%** (no city lost) | The same |
| `war-japan-sparta-t142` | hold all 10 | 70% (range 50 to 85%): AI attackers rarely commit, AI defences here hold 95 to 100%; Japan loses 2 to 4 units per Spartan one | **0%** | Kyoto falls on turn 3 in all 64 runs (Japan loses 2 combat units, Sparta none, the lopsided pattern in miniature) and that ends the run |

### Round 2: his orders replayed for the German cases, the Japan window played in full (predictions written before this run)

- **German scenarios:** Japan's seat replays the designer's recorded orders for the window (`scenarios/war/scripts/<id>.orders.json`: his air strikes, rebases and Armour moves, recorded from the tape by `extract-war-states.sh`, written by `war-build`, loaded by `crates/sim/examples/war_script` in `war_suite` and `war_trace`). The strikes name hexes and the Armour moves name tiles, so they replay on a position that differs from the real one; the engine skips an order that no longer fits. Everything else, Germany above all, is the Hard AI. A scenario carries a script only where the AI cannot play the attacker's side.
- **Japan scenario:** Sparta is a Hard AI with his army and 2,457 Gold, not him.

| Scenario | Predicted | Measured (64 runs, commit of this README) |
|---|---|---|
| `war-german-air-t214` | **15%** (range 5 to 35%); no city lost in under 5% | **0%** (0 of 64); exactly 3 cities lost in every run, the third on turn 5 (T218) in 61 runs and turn 6 in 3. The user's game: the third city, Berlin, fell on T218, turn 5 |
| `war-german-air-t219` | **25%** (range 5 to 50%); both kept in under 5% | **0%** (0 of 64); Germany eliminated on turn 3 (T221) in every run, as in his game |
| `war-japan-sparta-t142` | **40%** (range 20 to 60%); Japan loses about 8 combat units (range 4 to 15) to Sparta's 2 (range 0 to 4), about 0.25 Spartan losses per Japanese one | **78%** (50 of 64) kept at least 9 cities (lost 1 city in 33 runs, 2 in 26, 3 in 5; none kept all ten). **Exchange: Japan lost 7.5 combat units a run (median 7); Sparta lost 7.9 and Athens 1.3, 1.23 enemy losses per Japanese one** (1.06 counting Sparta alone), against the user's 0 for 30 |

Reading them: both German fixtures reproduce the real defeat to the damage roll (run 0 of T214: the four Bombers hit Cologne for 102, 87, 124 and 89 and the Armour walks in on T214, exactly as in his game; Munich T216, Berlin T218, Hamburg T219, Essen T221), so the baseline to beat is 0% with a result identical to what the German AI did to itself. The Japan fixture does not: with Sparta played by an AI the exchange is about even, where his Sparta killed 30 for none. It measures how the AI defends and fights against another AI, not against him, and it is the baseline an exchange-ratio change (increment 2) must improve.

**What the AI did (traces, `war_trace` runs and a scan of all 64 runs)**

- **Germany (T214, T219), 128 runs:** it never ordered a Fighter, in production or by purchase. It put a Bomber in the queue in 195 production orders (Munich and Essen from the first turn; run 0 completes a second Bomber, 229, on T221), and the `air_rebase` default works: its one Bomber (221) rebases out of Cologne to Munich on T214, strikes on T215, T217, T218 and T221 (131 `Rebase` orders over the 64 T214 runs), but the strikes go at ground targets: on T221 two Bombers hit the Armour for 7 and 10 and leave it at 22 HP, and nothing stops the four Japanese planes. Its Gold (134 to 257 a turn) goes to Siege upgrades and one Soldier (T221, 160). It accepts Athens's Peace on T215, offers Japan Peace on T216 (refused) and loses its third city on T218 in 61 of 64 runs. It sees no air threat, as in the real game.
- **Japan (T142), run 0 and a peace scan of all 64 runs:** Sparta's veteran Arquebusier shoots Kyoto to 0 HP on turn 1 (230 damage) and Sparta walks in on T144 (turn 3) after the Arquebusier kills the Trooper garrison (156); Japan's counterattack kills two Spartan Knights (T145, T146) and a Peace follows on T146 (effective T147). **Sparta and Japan sign Peace in 45 of 64 runs (median T153, 15 of them on T146) and never in 19**, where the user refused every Japanese offer for 150 turns, so the exchange is measured over the fighting turns. In run 0 Sparta declares again on T161, the fight is around Edo, and Peace follows on T170.

### Early German fixture (`war-german-air-early`, T210)

Added so that Germany has time to react. From the tape: Japan bought its first Bomber on T172 (a private hangar), its first strike on a German unit was T173 and on a German city T183 (Leipzig), so the threat was visible from T173, and it had four Bombers from T204. The goal is more than half of the six cities (4 of 6), which his game fails when the third city (Munich) falls on T216.

- **First attempt, T205 (6 cities, 9 land units, 301 Gold, 17 turns). Predicted 5% (range 0 to 25%), written before the run. Measured 100%** (64 of 64; Germany lost exactly one city, Leipzig). Not usable: the replay drifted. From T207 on most recorded orders no longer fit (`WAR_TRACE_FAILED=1 war_trace` lists them): strikes at hexes where his target units stood in the real game ("no enemy unit or city to attack there"), the Armour's route ("must not pass through a known enemy"), then Cologne never visible to Japan, so four city strikes were refused on T214, and the planes out of range later. A script that long is no longer his offensive.
- **Start search (fidelity, not AI strength).** The same script from each start, 64 runs: T207, T208 and T209 read 100%; **T210 reads 4%** and T212 4%. T210 is the earliest start at which the replay still reproduces his game, so that is the fixture. This choice was made after seeing the results; no prediction was written for T210.
- **T210 as shipped** (6 cities, 8 land units, 189 Gold, 12 turns, 4 of 6): **4%** (3 of 64 kept four cities; 61 of 64 fall below four on turn 7, T216, as in his game). Germany loses 2.95 cities a run on average, 6.6 land units for 1.1 Japanese ones. It gets five turns before Cologne falls and seven before the third city, which is the room a better AI has to buy Fighters. In its 64 runs the German AI ordered 570 Bomber strikes and 128 rebases and queued Bombers, and **never a Fighter**.
- Replay length is the limit of this method: it holds for about 12 turns from a state where the position still matches his, not for the 49 turns since his first strike.

### Rerun

```sh
TMPDIR=/some/scratch tools/sim/war-scenarios.sh --local --no-ledger --runs 64 \
  --ids war-german-air-t214,war-german-air-t219,war-japan-sparta-t142
python3 tools/sim/war-scenarios-report.py --compare BASE.json CANDIDATE.json   # paired, run by run
```

`--local` matters: the wrapper's default is the cloud box. A full run takes about 8 seconds on the Mac. The report table has an *Enemy units lost* column (mean combat units destroyed that belonged to a player at war with the seat, with the ratio to the seat's own losses in brackets); the JSON also has `enemy_units_lost_by` per run (owner id to count). Round 1 is `runs/20261027T164616Z-hard-air-fixtures-round1.json`, round 2 `runs/20261027T165403Z-hard-air-fixtures-round2.json`. Trace one run: `target/release/examples/war_trace scenarios/war/war-german-air-t214.json 0` (it prints the seat's Fighter, Bomber, strike and rebase orders as well).

## Measuring an AI change

```sh
tools/sim/war-scenarios.sh --check                                         # attack regression check (Golden Horde, thin treasury, Hard, 64 runs; exit 1 under 30%)
tools/sim/war-scenarios.sh --brain hard --runs 64                          # baseline
tools/sim/war-scenarios.sh --brain hard+search --runs 64
tools/sim/war-scenarios.sh --brain hard --runs 64 --ai-params CAND.json --params-seats seat
python3 tools/sim/war-scenarios-report.py --compare runs/BASE.json runs/CAND.json
```

The wrapper builds the `war_suite` example on the cloud box, runs it on one spot server, writes `runs/<stamp>-<tag>.json`, appends `ledger.jsonl`, writes `latest-<tag>.md` and redraws `chart-<tag>.png` (portrait, 1080 px). Run `r` of every scenario uses the same seed in every report (run 0 keeps the game's own seed), so `--compare` pairs outcomes run by run and prints a bootstrap interval per scenario. Like the full-game benchmark, judge a candidate seat-only (`--params-seats seat`, the default) and report opponent-only as context.

Each run also hunts for bugs: the position is checked after every turn (`scenario::validate_state`, plus any unit that moved into land closed to it), and panics are caught; they appear in the report's `bugs` lists. Units left in land that closed under them (borders grew, Peace was made) are legal and stranded, so they are not reported.

## Sparta's Nicaea Strike (2026-10-17)

From the coach's 8.9.2 Sparta review: the user's T18 state, four Hoplites at Nicaea's border, take it by T26 (he took it on T22 with no losses). The state is the replay of his save at engine 28be168b (hash-checked every turn), current rules. Prediction written first: plain Hard 30%, with the phase 1 attacker genes 55%. Measured (64 runs): **plain Hard 0%** (it waits at the border for a siege it can forecast while Byzantium builds), **100% with `storm_forecast` plus `value_first_target` or `campaign_commit`** (median 5 turns, no losses), now the default. First Blood stays at 100% (9 to 8 turns).

## After the coalition and runaway package (2026-10-18)

With the phase 1 and phase 2 genes on by default (every seat plays them, 64 runs each): Golden Horde **98%** (was 43%; the `--check` floor stays 30%), Golden Horde hold **1%** (was 56%), Behind the Walls **7%** (was 90 to 100%), Sparta's Nicaea Strike 100% (was 0%), First Blood 100% (8 turns), Germany's Last Stand 98%, the other four unchanged at 0%. Suite 40%. The defences fall because the attacking AIs now go at the leader: `runaway_read` alone moves all three walled-city scenarios. The 315 "bugs" in Germany's Last Stand are one validator flag repeated (an allied Scout standing in an allied city, legal by the movement rules; see the 2026-10-17 journal entry).

## The Golden Horde check broke when the defender could see (2026-10-18)

`--check` read **0%** on master (`6d1cbd7b9`, floor 30%, 100% at the phase 2 landing `ac975214c`). It was not the Mongol AI. A bisect on the box (one job, build plus 64 runs a probe) put the drop at `dbf1395e7`, "Make owned territory and border ring always visible": every team now sees its own land plus one ring around it, and an ordinary unit that attacked a team stays visible to it through its next planning turn. With both rules switched off in a throwaway build, master reads 100% (median 7 turns); with only one off it reads 0 to 1%, so each is enough. The Hard Swiss defender (the scenario's P0) used to be blind to the army walking up to Aarau and sat on its 2,235 Gold; now it sees it on turn 137, buys a Soldier and then a Marksman a turn in Aarau and a second city, the Mongol Campaign ("capture at 30%") ends on turn 139 for lack of an army that can take the bought garrison, and the Mongols propose Peace.

| Swiss treasury at T136 | Mongol attack (take Aarau) | Swiss hold |
|---|---|---|
| 0 Gold | 100% | 0% |
| 300 | 100% | 0% |
| 700 | 100% | 0% |
| 1,200 | 84% | 15% |
| 2,235 (the user's game) | **0%** | **100%** |

So `war-golden-horde` and `war-golden-horde-hold` are now "an informed Hard defender with the user's full treasury" and stay in the suite as they are (0% attack, 100% hold: the AI attack does not beat a defender that spends 2,235 Gold, and a human who did the same would hold too). The regression check moved to `war-golden-horde-thin`, the same start with the Swiss treasury cut to 500 Gold (`war-build` recipe with a treasury edit), where the attack reads 100% and the floor stays 30%. The Mongol march and siege were not touched. Open AI question, for the backlog: a Campaign that outlasts a defender buying garrison turn by turn instead of ending on the first forecast miss.

Suite on master `912d3c0fb` plus the thin scenario (Hard, 64 runs; the thin check 128 runs): First Blood 100% (8 turns), Nicaea 100% (4), Germany's Last Stand 100%, Golden Horde hold 100%, **thin 100% (median 7 turns, 128 of 128)**, Golden Horde 0%, Behind the Walls 1%, and the other five at 0%.

## Sieges and follow-through (2026-10-18)
`war_siege` 100 and `campaign_persist` 100 on for Hard (`docs/sim-results/ai-quality/eu4-wars.md`). Hard, 64 runs each, genes in the seat: The Golden Horde comes (original, informed defender with 2,235 Gold) **0% to 71%** (median 12 turns), the thin check 100% to 96% (the floor is 30%), First Blood and Nicaea Strike unchanged at 100%, suite mean 45.5% to 51.6%. Genes in the rivals: the Golden Horde hold 100% to 26%, Behind the Walls 1% to 0%, suite mean 38.7%. The original scenario failed because the campaign counted only the two units it began with and ended at the first forecast miss, the Mongol wall gate then dropped the city as a target, and the retreat from city fire pulled the shooters back each turn. With the genes in every seat (the new default, landing check on the merged master): Golden Horde 64%, its hold 34%, thin 96% (the `--check` floor is 30%), Behind the Walls 100% (the Mongol defender with the genes now holds against Sparta, which doesn't besiege), First Blood and Nicaea 100%, suite mean 54.0%.

## 2026-10-23: free Alliances and the Golden Horde scenarios

Free Alliances (`72fc33073`) took `war-golden-horde-thin` from 89% to 0% on 64 runs: the Mongol AI joined a six-team bloc that dragged it into a second war and its siege on Aarau stopped (the entry of 2026-10-23 in `docs/learnings.md` has the bisect and the plan trace). With the Alliance value priced for inherited wars, an ally's reach and the size of the bloc it creates (`BLOC_SIZE_PENALTY`, `crates/ai/src/diplomacy.rs`), the two scenarios that start from the user's game read 78 to 84% (`war-golden-horde`, the attack with the Swiss on 2,235 Gold) and 15 to 21% (`war-golden-horde-hold`; the lower figure before merging master's stalls work, the higher after), nearer his result, and the thin check reads 68% (floor 30%).

**2026-10-22, era resources (`claude/era-resources`):** every scenario state here was saved before land units from the Classical era on needed their era's resource; they are re-encoded so their units and buildings load grandfathered (they need no supply until upgraded). The check's 0% at the time came from free Alliances on master, not from this (`docs/learnings.md`, 2026-10-23).

## Increment 0 fixtures: the tepid Swiss and the Chur ring (2026-10-27, story W0.2 of `plans/ai-war.md`)

Four attacker scenarios from the designer's two latest sessions, built with `tools/scenarios/extract-war-states.sh` (saves `a1025`, `r1026`; every turn of both replays hash-checked, 0 mismatches in 223 and 142 turns) and `legatum-sim war-build`. All four are on today's rules (recorded on 8.9.10 and 8.9.12), so nothing is migrated. In each the AI plays the attacker and every other seat, including the seat the designer played, is a Hard AI with fresh memory. That is the main way they differ from his games: the defender now sees the army (own land and the border ring are always visible), has the designer's treasury and spends it. The goal is a single city, with a deadline.

**Deadlines.** `turns` counts orders turns, so the last turn to take the city is the start turn plus `turns` minus 1.
- **T180, 14 turns (to T193).** Athens (city 14) is empty and stands 200 HP behind Walls with the designer's 3,152 Gold unspent, 16 Swiss land units two to six hexes away, two of them Musketmen. Fourteen turns is the Golden Horde's window (which a Hard AI took in a median of 8 turns): enough to shoot a walled 200 HP city down and walk in before the Athenian treasury turns into garrisons.
- **T210, 8 turns (to T217).** The Bombers have put the city at 0 HP, one Spearman holds it, 10 land units are three to five hexes out and the Athenians have 297 Gold. A human needs about three turns (kill the Spearman, walk in). Eight is more than twice that, and it stops before the city can heal to full.
- **T220, 4 turns (to T223).** The real clock: Athens held 10 VP for six turns at T220 and wins at the end of T223 (the game ends and the scenario with it). Five Bombers and a lone Spearman against 200 HP: two Bombers did the whole job on T222 in his game, so the window is enough.
- **Chur, 20 turns (to T162), no "hold every city" condition.** (The first baseline run had one: 62 of 64 runs failed it, in run 0 when Byzantium took Sendai, a population-1 city at the other end of Japan, which says nothing about Chur, so it was dropped and the scenario is revision 2.) One Arquebusier (100 HP) in a population-5 walled city, 13 Japanese soldiers within eight hexes, 20 in all; the designer's ring was 21 turns long, so 20 gives the army a full ring's worth of time. **Caveat: no human has taken Chur from here.** The analysis on `astra/fix-staging-ring-r2` (an oracle probe with perfect sight) projected the saved formation's best assault as losing all six of its legal arrivals and leaving Chur at 159/200 HP. So this one has not been shown to be achievable, only to be where the AI stood still; treat it as a "does the ring do anything" measure first.

**Predictions (written before the first run), plain Hard, 64 runs, goal met:**

| Scenario | Predicted | Why |
|---|---|---|
| `war-swiss-tepid-t180` | 5% | The Hard Athenian AI sees the Swiss army and spends its 3,152 Gold on garrisons, as the Swiss did against the Mongols at Aarau (0% with the same treasury), and the Swiss siege pass wants a survivable firing tile, which the old units did not have at T180 (two campaigns ended `insufficient_army`). The one modern gun may still be reserved at home by the Mongol war. |
| `war-swiss-tepid-t210` | 25% | The city is at 0 HP and the Swiss AI now starts with fresh memory, so it sees 0 HP instead of remembering 21, and the city-assault coordinator acts on any visible city with a melee unit within four hexes. Against that: the Athenian AI heals and buys a second defender with 297 Gold. |
| `war-swiss-tepid-t220` | 5% | Four turns, and the Bombers and the ground wing have to line up in the same two turns, which they did not in his game. |
| `war-japan-chur-ring` | 5% | The Hard siege pass runs, but Chur is invisible from the ring, the defender sees the army and has 6,481 Gold, and the oracle assault forecast loses everything. |

**Measured baseline (2026-10-27, master `a58e80856` = 8.9.13 plus the fixtures, plain Hard in the seat, every other seat Hard, 64 seeded runs each, run locally on the Mac in 15 s; report `runs/20261027T164825Z-hard-w02-fixtures.json`):**

| Scenario | Predicted | Met (of 64) | Median turns | Combat units lost | What the AI did (traces `war_trace`, runs 0 to 7) |
|---|---|---|---|---|---|
| `war-swiss-tepid-t180` | 5% | **0%** | - | 6.6 | The Swiss never fire on Athens (0 shots at the city in 8 of 8 traced runs, 2 to 4 at units). The Hard Athenian AI buys 4 to 6 units over the window, the first four in turns 180 and 181 in run 0 (two Infantry at 800 Gold each, an Artillery at 880, an Arquebusier at 544), and its city and Artillery kill several Swiss units in the first turns. The Swiss plan is Contain Athens at about 320; no Campaign on Athens is ever made (0 in 8 runs) and the siege count reads 0 in every traced turn. Same stall as the designer's game. |
| `war-swiss-tepid-t210` | 25% | **100%** | 4 | 3.0 | Not the designer's game: with fresh memory the Swiss see the city at 0 HP and start a Campaign on it on turn 210 ("capture at 15%"), the Artillery fires on the city on turn 211 (and again on 213) and the city is taken on turn 213 (3 turns in, in 63 of 64 runs; 2 in the other). Athens buys one Ranger (264 Gold), which dies on turn 211; its Artillery kills two Spearmen on turn 212, which is most of the 3.0 units lost. |
| `war-swiss-tepid-t220` | 5% | **51%** (33) | 4 (the last turn) | 0.5 | Bombers hit the city on turn 221 (253 damage in run 0), the Swiss plan flips to Campaign on turn 222, and about half the runs put a unit into the emptied city on the last turn (turn 223); in the rest nothing gets in by the deadline. Athens buys one Artillery (880 Gold) on turn 220 and a Spearman on 223. |
| `war-japan-chur-ring` | 5% | **0%** | - | 9.2 (and 1.1 of its own cities per run, to other empires) | The Hard Japanese AI never fires on Chur (0 shots in 8 of 8 traced runs, "siege 0" every turn). Plan on turn 143: Contain Switzerland (value 324). The Hard Swiss AI buys 7 to 9 units over the run (in run 0 three on turn 143: two Riflemen at 507 Gold each and an Assault Trooper; Cavalry and a Field Cannon later), and Chur and the new Riflemen and Field Cannon shoot the ring from the walls (5 to 6 city shots in each run, a ring unit killed most turns). The Swiss propose Peace on turn 148 and Japan accepts it (8 of 8 runs), which ends the war six turns in. |

- **Predictions:** two wrong in opposite directions. The T180 and Chur calls (5%, 5%) were right for the right reason (the informed defender spends the treasury, as Aarau's Swiss did). The T210 call (25%) was far too low and T220 (5%) too low: I expected the designer's stall, and these two scenarios do not carry it.
- **Why T210 and T220 did not reproduce the stall in this first baseline:** the scenarios started every AI with fresh memory (`war-build` loaded the state only). That turned out not to be the cause; see the next section, which carries the memory.
- **The two failures that do reproduce** are T180 (a big army that never opens fire on a city the defender is reinforcing) and Chur (a ring that never fires and a war that ends with peace). Chur is at Hard here; the designer's game ran Normal, and Hard does not change the outcome: the ring still never attacks.
- **Rerun:** `tools/sim/war-scenarios.sh --local --no-ledger --runs 64 --ids war-swiss-tepid-t180,war-swiss-tepid-t210,war-swiss-tepid-t220,war-japan-chur-ring` (it takes seconds; drop `--local` for the cloud box, which is at its monthly cap today). `--no-ledger` keeps the new rows out of the suite ledger, which still compares like with like.

### Carrying the AIs' memory (2026-10-27, later the same day)

Scenarios can now carry each AI's saved memory. `ScenarioFile` has an optional `ai_memory` (gzip-base64 JSON of every AI seat's `OpponentMemory`); `war_suite` and `war_trace` restore it into the brains before turn one. Scenarios without the field are unchanged: the 12 older war scenarios, 64 runs each, give **identical per-run results before and after** the change (win, turns, units lost, score and seed, all 768 runs; also after the final build). Against the ledger's last row (`era-p30`, commit `4d7810c5`, 2026-10-23) they differ only where master's AI changed since (Golden Horde 76 to 100%, its hold 23 to 0%, thin 100 to 90%, German stand 100 to 93%), which is not this change.

**What is persisted.** `ScriptedBrain` carries exactly one thing between turns, `OpponentMemory` (opponent assessments, adaptation, strategy switches, the threat log and the campaign memory with its plan and events); everything else (knobs, seed, difficulty) is rebuilt from the setup. That is also all the app writes into a save (`ai_opponent_memory`).

**How the memory is made.** `tools/scenarios/war_memory.rs` runs inside the recording engine's tree (`extract-war-states.sh` with the `memory` field): a `Session` with the original brains replans every AI seat each turn while the recorded orders advance the game. Checks on both games: every state hash matches, **every recorded AI order list is reproduced (1,561 of 1,561 for Athens; 593 of 593 for Chur) and the replayed final memory equals the one in the save**. The designer's own seat has no brain and starts fresh. In the Chur game the AIs were Normal, so Japan's memory is a Normal brain's (a Hard Japan restores its campaign memory but has no plan state to restore).

`war-swiss-tepid-t180`, `-t210`, `-t220` and `war-japan-chur-ring` now carry the memory (revisions 3, 3, 3, 4). `war-swiss-tepid-t220-fresh` is the fresh-memory twin of T220 (an informed control: it differs from the carried one); a T210 twin was dropped because it reads exactly the same (100%, median 4). The attacker also keeps the paths and standing orders its own brain had set (the first version cleared them as it does for the designer's seat; this changed no result).

**Baseline with the memory carried (plain Hard in the seat, master `a58e80856` plus this branch, 64 runs, goal met):**

| Scenario | Fresh memory (first baseline) | **Memory carried (default)** | Memory carried, `local_mongol_guard` 0 (8.9.10 behaviour) |
|---|---|---|---|
| `war-swiss-tepid-t180` | 0% | **0%** (8.1 units lost) | 0% |
| `war-swiss-tepid-t210` | 100% (median 4) | **100%** (median 4; 4.0 lost) | 100% (median 5; 1.0 lost) |
| `war-swiss-tepid-t220` | 51% | **23%** (median 4) | **0%** |
| `war-swiss-tepid-t220-fresh` | - | 51% | 0% |
| `war-japan-chur-ring` | 0% | **0%** (8.9 lost) | 0% |

Reports: `runs/20261027T170805Z-hard-w02-memory.json` and `runs/20261027T171044Z-hard-guard0-seat-w02-memory.json`.

**Does the stall reproduce?**
- **T220: yes, but the cause is the garrison reservation, not memory.** With the 8.9.10 behaviour (the gene `local_mongol_guard` switched back to 0 for the seat) T220 reads **0 of 64 with or without the memory**, the stall the designer saw. With today's default (the gene is on since `a3c046bd7`, the Athens-audit fix that stops an unrelated Mongol war from pinning the Swiss Artillery at home) it is 23% with the memory and 51% without. So 8.9.13 already repairs part of this stall, and carrying the memory lowers it by 28 points (not looked into).
- **T210: no.** At T210 the scenario brain's first orders are the recorded ones (with the gene off they match the tape's Swiss orders exactly except for a Peace proposal and a Bomber order), but the run then diverges from the designer's game, where the Hard Athenian AI replaces him: the Swiss take the city in 4 to 5 turns (100% with the gene on or off, memory or not; with the default the Artillery fires from turn 211 and the city falls on turn 213 in run 0). I did not isolate which Athenian difference opens it. T210 stays in the suite as a "finish a city the Bombers opened" control, not as a reproduction.
- **T180: yes**, unchanged: no capture in either setting (the Swiss never fire on the city while Hard Athens buys 4 to 6 units, now 8.1 Swiss units lost per run).
- **Chur: unchanged, 0%.** The carried Normal memory changes nothing visible: the Hard Japan plans Contain Switzerland on turn 143 as before, fires 0 shots at Chur in 8 of 8 traced runs, the Swiss buy 6 to 10 units, and in 7 of 8 runs Japan accepts the Swiss Peace on turn 149 (the eighth never signs peace and still never attacks).
- **One bug report** in `war-japan-chur-ring` (run 41, turn 156: a Ranger standing in the foreign city Hiroshima); it is one run of 64 and looks like the known legal-but-flagged allied-unit case, not looked into.

**Rerun the carried-memory set:** `tools/sim/war-scenarios.sh --local --no-ledger --runs 64 --ids war-swiss-tepid-t180,war-swiss-tepid-t210,war-swiss-tepid-t220,war-swiss-tepid-t220-fresh,war-japan-chur-ring` (the 8.9.10 arm: add `--ai-params guard0.json` with `{"schema":1,"name":"guard0","decisions":{"local_mongol_guard":0}}`). Extract with memory: `tools/scenarios/extract-war-states.sh OUT a1025 r1026`, then `legatum-sim war-build --states OUT <ids>`.

## Fixtures from the designer's two 10-27 games (2026-10-27, W2 queue)

Seven cases (six specified by the reviews, one added) from the 8.9.13 Athens bomber win (`athens-win-bombers-20261027`, seat 0 = Athens, 177 turns) and the 8.9.13 Sparta conquest (`sparta-win-20261027`, seat 0 = Sparta, stopped at T126), specified by the coach reviews `docs/playtests/2026-10-27-athens-bombers-review.md` and `2026-10-27-sparta-review.md`. Built with `tools/scenarios/extract-war-states.sh` (entries `ab` and `sw`, engine `a58e80856` = `playtest-8.9.13`; both replays hash-checked every turn, 0 mismatches in 177 and 125 turns; every recorded AI order list reproduced by the replayed brains, 698 of 698 and 309 of 309, and the replayed final AI memory equals the save's) and `legatum-sim war-build`. In each, **the designer's seat (0) replays his recorded orders** for the window (`scenarios/war/scripts/<id>.orders.json`) and **every AI seat restores the memory its brain had** in his game. The tested seat is a Hard AI.

What is new in the tooling (all additive):
- `extract-war-states.sh` takes `memory,0:101-177` in the fifth field (memory and recorded orders together; before, one or the other).
- `core::scenario::Condition` gains three variants, each with a unit test in `crates/core/src/scenario/tests.rs`: `CapturedFrom { rival, min }` (the seat owns at least `min` cities that `rival` founded: a capture from that rival, including a colony it founds after the start), `MinStartingCities(n)` (the seat still owns at least `n` of the cities it started with) and `UnitsGone([ids])` (those units no longer exist). Two goal mistakes in the first builds, both fixed before the baseline below, are worth keeping: (1) the attacker goal was an any-city capture, and it read 14% on `war-sparta-next-door-t101` when the answer was 0%, because in 9 of 64 runs the Spartan army took a **Mongol** city (the Mongols are at war with it, and Byzantium takes the same city in his game on T108): a capture goal must name the victim; (2) the defence goals were `MinCities`, and `war-japan-hoplites-t43` read 71% when the answer was 28%, because in 28 of 64 runs the Japanese AI had lost all three cities and founded a new one elsewhere: a defence goal must count the starting cities.
- `war_build::modernize` takes an `Option` seat. In these seven nobody is cleared: the AI keeps the paths and standing orders its own brain set, and the designer's units keep the paths he had given them, because his recorded orders are issued on top of those (clearing them stops the units and is the main reason a replay drifts).

### Fidelity: the replay reproduces his game in run 0, and in other seeds only for as long as the position stays close to his

Run 0 keeps the game's own seed. On the 8.9.13 build (the binaries below) the recorded captures come back on the same turns:

| Fixture | Captures in run 0 (turn, city) | His game |
|---|---|---|
| `war-mongol-bombers-t150` | 153, 159, 160, 163, 165, 167 | the same six |
| `war-germany-berlin-t65` | 67 Berlin, 86 Essen, 92 Cologne | the same |
| `war-athens-two-pikemen-t97` | 99, 106, 111, 118, 121, 125 | the same |
| `war-sparta-next-door-t101` | Byzantium takes the Mongol city on 108; Sparta declares on 101, aims at Phaleron, moves to the Mongol city on 104, proposes Peace on 105 and never fights an Athenian unit but with city fire | the same |
| `war-break-the-hold-t168` | Athens takes Bibracte on 171, no attack by Byzantium | the same |
| `war-japan-hoplites-t43` | 44, 49, 56 | the same |

(Some recorded orders fail in run 0 as well, 9 in the Mongol case, the first on T151: a move for a unit that no longer exists, a proposal already answered. The captures still match. `WAR_TRACE_FAILED=1 war_trace` lists them.)

**The turn-42 start did not replay.** `war-japan-hoplites-t42` as the review specified it: the state at T42 has one Hoplite standing in Japanese land at Peace (the declaration turn), which today's rules forbid; `war-build` moves it one tile and his T42 orders for it no longer fit, so even run 0 took only Matsuyama (T49). T43, the first turn at war, replays exactly. The scenario is `war-japan-hoplites-t43` (14 turns, deadline T56 unchanged). The review's prediction (5 to 15%) was for T42; **T43 has no prior prediction.**

**Other seeds are not his game, for the three Sparta-game defences.** Seeds 1 and up change the dice and the AI's own tie-breaks, and his orders (moves by exact path, strikes by hex) stop fitting as soon as a unit dies or stands a tile elsewhere. Germany's Berlin hangs on one roll: in run 1 the Hoplite that fires at T66 is killed (the city's shot leaves it at 0 HP where run 0 leaves it at 6) and Berlin stands at 1 HP for a turn in which the script has no unit left to use. A start-turn scan (throwaway build on the 8.9.13 tree, 32 runs per start, share of runs in which the AI seat lost at least as many of its cities as he took in the window; the run stops at the first goal failure, so the count is capped by the base goal) shows where the replay still reproduces his result:

| Fixture | Start | Share of runs with his losses or more |
|---|---|---|
| Athens (Pikemen) | T97 (shipped) | 3% (2 of 64 runs; he took 5 starting cities) |
| | T99 / T102 / T106 / T110 | 3% / 0% / 0% / 9% |
| | T112 (shipped as the late twin) | 72% |
| | T114 and later | 100% (he took 2 more, both already under attack) |
| Germany (Berlin) | T65, T66, T67 | 0% at every start from T66 to T85 (he took 3, then 2) |
| Japan (Hoplites) | T43 (shipped) | 72% |
| | T44 / T45 / T46 / T47 / T49 / T51 | 78% / 94% / 91% / 81% / 75% / 100% |

So: Germany has no start at which the verbatim script reproduces more than run 0 (the real result is a roll the script cannot repeat), and the Athens case reproduces from T112. The T97 start stays as specified by the review (it is his game's real start and run 0 is exact) and **`war-athens-two-pikemen-t112` is added as the late twin** (same units 25 and 33, same script, goal scaled to the six cities it starts with: keep 4, kill both Pikemen). Both Sparta-game cases at their specified goals read 95 to 100% for the AI and are **regression guards, not hill-climb targets**; the Mongol, Japan T43 and Athens T112 cases have room (12%, 28%, 14%).

### Baseline (64 runs each, plain Hard in the tested seat, every other seat Hard with its carried memory, commit `01325f49d` = master `1e481f8c3` plus the fixtures)

"8.9.13" is the same code with both modernization genes at 0 for every seat (`--ai-params modern0.json --params-seats all`, `{"schema":1,"name":"modern0","decisions":{"modernization_upgrade":0,"modernization_return":0}}`). I also built a tree with the other 8.9.14 changes (the stranded-unit egress fix) reverted to the `a58e80856` files: **its 448 runs equal the genes-off runs run for run**, so the egress fix changes nothing here and the whole 8.9.13 to 8.9.14 difference on these seven is the two genes. (The 8.9.13 numbers below come from that reverted tree.)

| Fixture | Predicted (coach, written before any run) | 8.9.13 (genes off) | **8.9.14 (genes on)** | What the AI did (traces `war_trace`, runs 0 to 2, and the 64-run reports) |
|---|---|---|---|---|
| `war-sparta-next-door-t101` | plain Hard 5 to 15%; planner on 20 to 40% | **0%** (0 of 64) | **0%** (0 of 64) | A little under the predicted range. Sparta declares on turn 1 in 64 of 64 runs, as in his game, with the same plan (Campaign on Phaleron at value 0, "forced expedition before local siege forecast"), moves to the Mongol city on turn 4 (T104, "capture at 12%") and proposes Peace on T105. The only fire on an Athenian unit is Sellasia's (one Athenian Marksman killed a run on average). It takes a Mongol city in 9 runs on 8.9.14 (4 on 8.9.13), never an Athenian one. 63 of 64 runs differ between the two builds, none in the goal. |
| `war-mongol-bombers-t150` | plain 0 to 5%; with the upgrade gene 10 to 30% | **17%** (11) | **12%** (8) | Fails the goal in 56 of 64 runs on 8.9.14 (53 on 8.9.13), the third city lost on the 11th turn (T160) as in his game; 2.7 starting cities lost a run, 6.0 combat units. The Mongols fight Byzantium and the Celts (5.8 and 1.0 enemy units killed a run) and 0.2 Athenian units. The plain-Hard prediction was too low (17% against 0 to 5%); the genes moved the goal result in 3 runs of 64 (−5 pp, inside the noise, and the wrong way for the 10 to 30% the upgrade-gene call implied). |
| `war-break-the-hold-t168` | 0 to 10%; planner on 10 to 25% | **0%** | **0%** | Byzantium never takes an Athenian city (0 captures in 128 runs). It fights Sparta instead (2.5 Spartan units killed a run, 6.4 of its own lost). In range. |
| `war-japan-hoplites-t43` | none for T43 (T42: 5 to 15%; with the gene 10 to 20%) | **28%** (18) | **28%** (18) | Identical in all 64 runs on the two builds (Japan has 42 Gold and nothing to upgrade). It keeps a starting city to T56 in 18 runs; in 16 it is eliminated and in 30 more it loses the last starting city before the end (28 of those 30 had founded a city elsewhere, which the first version of the goal counted as keeping one); the failed runs end after a median of 13 of 14 turns. Japan loses 2.7 combat units a run and kills 1.9 of his. Above the T42 call (5 to 15%), not far. |
| `war-germany-berlin-t65` | plain 10 to 25%; with the gene 20 to 35% | **95%** (61) | **95%** (61) | Far above the call. Berlin holds in all but 3 runs (3 of 64 end with fewer than two of the starting cities); Germany loses 3.1 combat units and kills 3.9 Hoplites a run and in 18 runs captures a city itself (median first capture on turn 19). 35 runs differ between the builds, none in the goal. A regression guard. |
| `war-athens-two-pikemen-t97` | plain 10 to 20%; with the gene 15 to 30% | **96%** (62) | **100%** (64) | Far above the call, for the reason in the fidelity section: only run 0 reproduces his Pikemen. In the other runs the Pikemen walk into the fire of walled cities and die (in all 64 runs on 8.9.14 with 2.6 Spartan units killed a run); Athens loses Phaleron (cities lost 1.0) and nothing else. |
| `war-athens-two-pikemen-t112` | none (added after the scan) | **17%** (11) | **14%** (9) | Fails in 55 of 64: in 30 the cities hold (at least four of the six) but a Pikeman survives, in 25 they are not kept. Athens kills 1.1 Spartan units a run, loses 2.3 of its own and 2.1 cities. This one has room: 9 to 11 successes of 64. |

Every run in every report had 0 bugs and 0 panics.

### The 21 earlier cases are unchanged

The whole suite was rerun with the new code (`tools/sim/war-scenarios.sh --local --no-ledger --runs 64`, 28 scenarios, 138 s with other work running, 45% mean goal success, 0 bugs): the 21 earlier cases give **identical per-run results** (every field of every run: win, turns, units lost, score, seed; 1,344 of 1,344 runs) to the same 21 run on master `1e481f8c3` (the baseline taken with the unmodified suite before the fixtures were added). Reports: `runs/20261028T032552Z-hard-w03-fullsuite.json`, `runs/20261028T0334Z-hard-w03-new7-8914.json` and `runs/20261028T0334Z-hard-w03-new7-8913-tree.json` (the 8.9.13 numbers; its 448 runs equal the genes-off arm).

### Rerun

```sh
TMPDIR=/some/scratch tools/sim/war-scenarios.sh --local --no-ledger --runs 64 \
  --ids war-sparta-next-door-t101,war-mongol-bombers-t150,war-break-the-hold-t168,war-japan-hoplites-t43,war-germany-berlin-t65,war-athens-two-pikemen-t97,war-athens-two-pikemen-t112
# the 8.9.13 arm on a current build: add  --ai-params modern0.json --params-seats all   (JSON above)
# rebuild the states: tools/scenarios/extract-war-states.sh OUT ab sw; legatum-sim war-build --states OUT <ids>
# trace one run: WAR_TRACE_FAILED=1 target/release/examples/war_trace scenarios/war/war-mongol-bombers-t150.json 0
```

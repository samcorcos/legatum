# Sparta 8.9.10: where the AI lost its wars (2026-10-25)

**Operations, coupled to tactical survival, is the largest demonstrated failure.** Japan lost **53 combat units to Sparta's 3** in the long war, despite completing **9,488 Production** of military construction and assigning about **58%** of its wartime Production to military queues. One Spartan ranged unit, **40**, contributed to **37/53 deaths (69.8%)**; it was **never attacked in melee**. Nearby Japanese troops were counted as a force before they could arrive together, then fed into its firing arc. Giving Japan more production does not address that failure.

The next large, concrete mistake was strategic placement: Japan founded Kyoto under a visible veteran's approach, before Walls could finish. That city became both a repeated capture objective and an XP source. Two opt-in, value-scored fixes are implemented: `raid_arrival_weight=100` and `founding_survival_weight=200`. **Both default to 0**, preserving today's behavior until the lead's benchmark gate. No faction balance or combat rule changes are made here.

The 69.8% is **casualty concentration, not an estimate that the fix reverses 69.8% of the outcome**. Strategy, movement and battle choice overlap. One recorded game cannot identify additive causal percentages of winning; the local intervention below isolates one avoidable death. Long-run win-rate improvement remains unmeasured.

## Evidence and definitions

Both supplied copies were read; no live save was opened for writing. The mid-war save's first **184 turns are an exact tape prefix** of the final save's **207 resolved turns**. The original engine reconstructed every turn with **207/207 matching hashes**. Every serialized state field in both saved snapshots also equals its rebuilt T185/T208 state. Its persistent AI brains were also replayed from the initial setup: **421/421 complete order lists match** (Celts 22, Japan 192, Athens 207). This is the AI's actual planning, using its fogged view, not a new seed or omniscient approximation.

- Original engine: `8e9433e5cf53ea4ffe4e8729d5904c6320cb519a`; setup seed `7732777812994077`; 40×26; human Sparta and three Hard AIs.
- The final save is the start of T208, after Sparta wins 10 VP on T207; the mid-war save is the start of T185. Japan is eliminated T192; Athens remains alive.
- [Compressed ledger](sparta-1025/ledger.json.gz): every player at every pre-turn, full army rosters (kind, level, XP, HP, base melee/ranged, tile), research counts, supply, upgrade opportunities, economy, all battles, casualties, captures, and war windows. [Original planning trace](sparta-1025/original-plans.jsonl.gz): orders, pre-plan memory, plan scores and campaign events for every AI turn. [Position comparisons](sparta-1025/positions.json.gz): current code on eight historical views, five arms each.
- A combat unit excludes Scout, Settler and Heir. Scouts are listed separately. Four garrison deaths come from ranged damage overflowing a city; the ledger labels these `city-overflow`, not direct duels. A contributing shooter is not necessarily the sole causal attacker.
- **P_HP** below is a reproducible descriptive index: sum of `floor(max(base melee, base ranged) × (1 + .1 × level) × HP/100)` per unit. It includes promotions and injury but not terrain, triangle, fortification, timing, faction combat effects or combined fire. It is **not** the engine forecast or a probability of winning. Actual forecasts drive the implemented fixes.
- Gold spent is actual event debits. Production investment is the quoted cost of completed construction, excluding Gold purchases (which also emit completion). It can include work started before a war. Queue allocation is pre-resolution city Production directed to its selected queue; focus changes and captures can change realized output. It is a **planning-share proxy**, not an exact expenditure ledger. Overlapping war windows contain the same empire spending and must not be summed.
- Timestamps in tables are game turns. Opening armies are before declaration resolves; all turn-by-turn changes and exact per-unit promotion/HP values are in the ledger.

Source SHA-256:

- `sparta-win-final-20261025-0902.legatum`: `e4e8724d38281140f45b428fc62b7a36f6abfbf023a2e1c8f8cde3d3c282d450`
- `sparta-crushing-japan-20261025-0841.legatum`: `a64e84c3a4513b47f5ed90d87fc4a8b714f4813374528995f6a737dfe0f8337a`

## Every war

There are seven human war windows and one AI–AI window. No other wars are present in the chronicle.

| Turns | Declarer → opponent | Combat losses, first side : second side | Cities changing hands | End |
| --- | --- | --- | --- | --- |
| 19–22 | Sparta → Celts | Sparta 0 : Celts 2 | Bibracte T22 → Sparta | Eliminated |
| 25–31 | Japan → Sparta | Sparta 0 : Japan 1 | None | Peace |
| 38–80 | Sparta → Athens | Sparta 3 : Athens 2 | Marathon T41 → Sparta | Peace |
| 42–192 | Japan → Sparta | Sparta 3 : Japan 53 | Kyoto T135 → Sparta; Kyoto T136 → Japan; Kyoto T158 → Sparta; Edo T169 → Sparta; Shizuoka T175 → Sparta; Takamatsu T179 → Sparta; Sapporo T180 → Sparta; Fukuoka T183 → Sparta; Okayama T187 → Sparta; Osaka T188 → Sparta; Matsuyama T189 → Sparta; Matsue T190 → Sparta; Wakayama T192 → Sparta | Eliminated |
| 71–192 | Athens → Japan | Japan 5 : Athens 2 | None | Eliminated |
| 193–195 | Athens → Sparta | Sparta 0 : Athens 1 | None | Peace |
| 199–201 | Athens → Sparta | Sparta 0 : Athens 2 | None | Peace |
| 204–207 | Athens → Sparta | Sparta 0 : Athens 0 | None | Won |

### Armies at declaration

`L0:2 L1:3` means two level-zero and three level-one units. The ledger gives these same fields for **every turn**, including all arrivals, upgrades, promotions and surviving armies.

| War starts | Side | Count and composition | Total HP | Levels | P_HP |
| --- | --- | --- | --- | --- | --- |
| 19 | Sparta | 3: 3 Spearman | 300 | L1:3 | 66 |
| 19 | Celts | 1: 1 Spearman | 100 | L0:1 | 18 |
| 25 | Sparta | 4: 4 Spearman | 400 | L1:3 L2:1 | 90 |
| 25 | Japan | 2: 1 Spearman, 1 Archer | 200 | L0:2 | 35 |
| 38 | Sparta | 6: 6 Spearman | 516 | L1:5 L2:1 | 114 |
| 38 | Athens | 2: 1 Spearman, 1 Archer | 200 | L0:2 | 35 |
| 42 | Sparta | 4: 4 Spearman | 400 | L1:1 L2:3 | 94 |
| 42 | Japan | 3: 2 Spearman, 1 Archer | 300 | L0:3 | 55 |
| 71 | Japan | 7: 5 Spearman, 1 Archer, 1 CompositeArcher | 630 | L0:7 | 122 |
| 71 | Athens | 4: 2 Phalangite, 2 Archer | 400 | L0:4 | 80 |
| 193 | Sparta | 7: 1 Pikeman, 3 Rifleman, 3 Artillery | 628 | L1:2 L2:2 L3:2 L6:1 | 594 |
| 193 | Athens | 9: 2 Infantry, 1 Archer, 2 Rifleman, 1 FieldCannon, 1 Crossbowman, 2 Spearman | 900 | L0:8 L1:1 | 452 |
| 199 | Sparta | 11: 1 Pikeman, 3 Infantry, 3 Artillery, 4 Tank | 1035 | L1:6 L2:2 L3:1 L4:1 L6:1 | 1094 |
| 199 | Athens | 10: 3 Infantry, 1 Archer, 2 Rifleman, 1 FieldCannon, 1 Crossbowman, 2 Spearman | 1000 | L0:8 L1:2 | 547 |
| 204 | Sparta | 11: 1 Pikeman, 3 Infantry, 3 Artillery, 4 Tank | 1046 | L1:6 L2:2 L3:1 L4:1 L6:1 | 1105 |
| 204 | Athens | 10: 3 Infantry, 2 Rifleman, 1 FieldCannon, 4 Spearman | 1000 | L0:9 L1:1 | 540 |

### Military economy by war window

Gold columns are military purchases / upgrades / siege purchases. Walls are Production-only under this engine; “unspent Gold that could buy Walls” is not a legal alternative. `M / total` is military queue allocation; Walls are additional defense spending. The human's captured-city economy and Gold purchases matter: zero military construction does not mean zero military investment.

| Turns | Side | Completed military P | Completed Walls-line P | Gold: units / upgrades / siege | Military queue P / all P | Peak treasury |
| --- | --- | --- | --- | --- | --- | --- |
| 19–22 | Sparta | 80 | 0 | 0 / 0 / 0 | 60 / 60 | 151 |
| 19–22 | Celts | 152 | 0 | 0 / 0 / 0 | 20 / 40 | 81 |
| 25–31 | Sparta | 160 | 0 | 0 / 0 / 0 | 115 / 147 | 312 |
| 25–31 | Japan | 80 | 60 | 0 / 0 / 0 | 95 / 180 | 34 |
| 38–80 | Sparta | 0 | 300 | 264 / 0 / 480 | 0 / 2,016 | 581 |
| 38–80 | Athens | 344 | 120 | 0 / 288 / 0 | 289 / 1,459 | 131 |
| 42–192 | Sparta | 0 | 780 | 3,860 / 1,208 / 1,109 | 0 / 15,637 | 3,816 |
| 42–192 | Japan | 9,488 | 1,080 | 0 / 1,312 / 520 | 10,586 / 18,234 | 114 |
| 71–192 | Japan | 8,948 | 900 | 0 / 1,312 / 520 | 9,934 / 16,387 | 114 |
| 71–192 | Athens | 1,008 | 2,040 | 160 / 1,348 / 0 | 1,274 / 8,722 | 313 |
| 193–195 | Sparta | 0 | 0 | 0 / 110 / 0 | 0 / 610 | 4,410 |
| 193–195 | Athens | 480 | 0 | 0 / 0 / 0 | 153 / 273 | 143 |
| 199–201 | Sparta | 0 | 0 | 0 / 0 / 0 | 0 / 736 | 810 |
| 199–201 | Athens | 0 | 0 | 320 / 0 / 0 | 72 / 273 | 287 |
| 204–207 | Sparta | 0 | 0 | 0 / 0 / 0 | 13 / 958 | 1,564 |
| 204–207 | Athens | 0 | 0 | 0 / 0 / 0 | 201 / 364 | 260 |

### How the fronts developed

**Celts, T19–22.** Sparta entered with three level-one Spearmen against one level-zero Spearman (P_HP 66:18). Celts saw one Spartan military unit on T18 and all three on T19, before declaration resolution. Bibracte had no Walls: 200 HP before T19, 113 before T20, 9 before T21. Its nearly completed Archer finished T20 and fired T21, then died. Walls were selected only T21. Bibracte fell T22; the last defender and empire disappeared. Two combat deaths, no Spartan losses. This was an opening preparation/arrival failure, not a late-game resource shortage; Copper supply was available, but treasury 72–81 could not buy an emergency army.

**First Japan war, T25–31.** Japan declared with two units against four. Its actual Contain score was +142: VP denial +25, leader race +311, war footing −98, attrition −96. One Archer and one Scout died; no city changed hands. Japan proposed Peace T30, effective T31. This was a weak opening containment attempt, not refusal to exit a losing war.

**First Athens war, T38–80.** Sparta had six promoted Spearmen against one Spearman and one Archer (114:35 P_HP). Athens saw two Spartan combat units on T37 and five on T38. Marathon was already building Walls, but the attack arrived first; it fell T41. Sparta lost two Spearmen during the assault (T39/T40) and another T79; Athens lost two combat units and a Scout. Athens retained its walled core and peace ended the window T80. Its military queue share was 19.8%, with additional Walls spending. The defense inflicted real losses; this is less severe than Japan's later exchanges.

**Long Japan war, T42–192.** Japan declared again while Sparta was fighting Athens. Its Contain score +614 included leader race +1,082, war footing −164, attrition −185 and peace credibility −200. Japan later tried diplomacy: Alliance with Athens T48; Peace with Sparta T130, 138, 146, 154, 162, 170, 178 and 186; repeated Peace proposals to Athens too. None ended these wars. “It should simply have made peace/allied” is not unilateral control it possessed.

Japan's campaigns repeatedly started and failed to assemble: a target on T90 ended `insufficient_army` T92; its successful T136 recapture of Kyoto ended `captured` T137; T154's new target ended `insufficient_army` T155; T162's ended T163; T164's ended `walls_before_arrival` T165. This is recorded planner memory, not inferred intent. It continued to fund an army throughout.

| Long-war phase | Japan combat deaths to Sparta | Sparta combat deaths to Japan | What happened |
| --- | ---: | ---: | --- |
| 42–130 | 13 | 0 | Attrition without a Japanese city loss; veteran 40 survives and upgrades. |
| 131–157 | 11 | 1 | Exposed Kyoto, capture/recapture, repeated city shots and isolated replacements. |
| 158–174 | 20 | 1 | Permanent Kyoto capture T158, Edo T169; largest burst of Japanese losses. |
| 175–192 | 9 | 1 | Nine further Japanese cities fall; the empire loses its resource base and is eliminated. |

Sparta's three deaths were a Knight at Kyoto T136 and Field Cannons T174 and T179. Scout 2 also died T113 (excluded from combat totals). Unit 40 survives to the final save. Its T158 Field Cannon shot dealt 224 damage to full-HP Pikeman 107; Japanese Crossbowman 90's return shot dealt 3. That matchup cannot be assessed by empire-wide unit count.

**Athens–Japan, T71–192.** Athens declared to contain Japan (+168 plan value, of which leader race +175). Opening forces were 4:7 units and 80:122 P_HP. Neither took a city from the other. Japan killed two Athenian combat units and four Scouts; Athens killed five Japanese combat units. Athens's military queue share was 14.6%; its 2,040 completed Walls-line Production exceeded 1,008 on units. It never converted its second front into a capture before Sparta eliminated Japan. This was an ineffective offensive operation, but not an Athenian territorial defeat.

**Late Athens–Sparta, T193–195, 199–201, 204–207.** Athens switched targets immediately after Japan's elimination. Its relative army index fell from 452:594 to 547:1,094 and 540:1,105. It lost one, then two, then zero combat units (plus a Scout in the middle war), taking no city. Peace ended the first two windows; Sparta's VP win ended the last. Athens was trying to stop a victory countdown: T199 and T204 Contain scores explicitly included `contest the hold` +590/+348. Declaring while weaker is not by itself irrational when inaction loses the game. The ineffective force and repeated peace intervals are the failures to investigate. The recorded engine's short truce applies here; later truce changes must not be retroactively attributed to this game.

### Midwar force snapshots

| Turn | Side | Army count | HP | Levels | P_HP | Composition |
| --- | --- | --- | --- | --- | --- | --- |
| 100 | Sparta | 4 | 388 | L1:1 L2:3 | 103 | 3 Spearman, 1 Crossbowman |
| 100 | Japan | 9 | 774 | L0:9 | 170 | 4 Phalangite, 2 Spearman, 2 CompositeArcher, 1 Archer |
| 129 | Sparta | 4 | 400 | L1:1 L2:2 L3:1 | 128 | 3 Spearman, 1 Arquebusier |
| 129 | Japan | 7 | 622 | L0:7 | 174 | 3 Pikeman, 1 Spearman, 1 Musketman, 1 CompositeArcher, 1 Archer |
| 142 | Sparta | 4 | 400 | L1:1 L2:2 L5:1 | 137 | 3 Spearman, 1 Arquebusier |
| 142 | Japan | 13 | 1114 | L0:13 | 355 | 3 Pikeman, 5 Musketman, 1 Crossbowman, 1 Archer, 2 Phalangite, 1 Spearman |
| 158 | Sparta | 5 | 500 | L1:2 L2:2 L6:1 | 234 | 2 Phalangite, 1 Spearman, 1 FieldCannon, 1 Musketman |
| 158 | Japan | 15 | 1492 | L0:15 | 545 | 3 Musketman, 1 Crossbowman, 5 Pikeman, 1 Phalangite, 1 Archer, 2 Rifleman, 1 Arquebusier, 1 Spearman |
| 165 | Sparta | 5 | 451 | L1:2 L2:2 L6:1 | 284 | 2 Pikeman, 2 Rifleman, 1 FieldCannon |
| 165 | Japan | 15 | 1500 | L0:15 | 660 | 2 Musketman, 3 Pikeman, 6 Rifleman, 1 Arquebusier, 1 Phalangite, 2 Spearman |
| 175 | Sparta | 8 | 771 | L1:2 L2:5 L6:1 | 563 | 1 Pikeman, 3 Rifleman, 4 FieldCannon |
| 175 | Japan | 6 | 600 | L0:6 | 307 | 2 Musketman, 2 Rifleman, 1 Pikeman, 1 FieldCannon |
| 185 | Sparta | 7 | 634 | L1:2 L2:3 L3:1 L6:1 | 596 | 1 Pikeman, 3 Rifleman, 3 Artillery |
| 185 | Japan | 2 | 200 | L0:2 | 110 | 1 FieldCannon, 1 Cuirassier |
| 192 | Sparta | 7 | 626 | L1:2 L2:2 L3:2 L6:1 | 592 | 1 Pikeman, 3 Rifleman, 3 Artillery |
| 192 | Japan | 0 | 0 | — | 0 | — |

## Layer attribution

Scores are an evidence-based severity judgment, **0 no demonstrated failure, 1 minor, 2 substantial, 3 major, 4 dominant**, not a fitted decomposition of win probability. A `—` means there is too little combat to assess. The cost column is measured; attribution is explicitly interpretation.

| AI side / window | Strategy | Operations | Tactics | Economy | Measured cost and explanatory scope |
| --- | ---: | ---: | ---: | ---: | --- |
| Celts, T19–22 | 4 | 2 | 1 | 2 | Entire empire, 2 units, 1 city. The 3.7:1 opening index and Walls arriving after collapse explain the defense's starting disadvantage; no evidence that a different duel alone saves it. |
| Japan, T25–31 | 2 | 2 | 2 | 1 | 1 combat unit and 1 Scout, no city. Understrength containment; Peace is actually proposed and signed. |
| Athens, T38–80 | 3 | 2 | 1 | 2 | Marathon and 2 combat units; inflicts 3 combat kills. Preparation matters more than a demonstrated tactical collapse. |
| Japan, T42–192 | 3 | **4** | **4** | 2 | 53:3 exchange and eventual elimination. 37 deaths involve one veteran; four are city-overflow garrison deaths (7.5%, overlapping). Production quantity is not the primary bottleneck. |
| Japan vs Athens, T71–192 | 2 | 2 | 1 | 1 | 5:2 combat loss exchange, no city changes on this front. The second war divides attention; this tape does not establish how many Spartan-front deaths that division caused. |
| Athens vs Japan, T71–192 | 2 | 3 | 1 | 2 | No captures in 122 turns; 2 combat units and 4 Scouts lost. Its offensive campaign repeatedly dissolves; low military allocation coexists with heavy fortification. |
| Athens, T193–195 / 199–201 / 204–207 | 2 | 3 | 2 / 2 / — | 2 | 3 combat units and 1 Scout lost, no captures, then human VP victory. The economic/army deficit is already large; a declaration can be rational denial despite poor odds. |

**What is and is not explained.** Japan's 37/53 concentration is much larger than the four directly observed overflow deaths, the one isolated casualty prevented in the two-turn fixture, or the three late Athenian combat losses. It identifies where to spend engineering effort, not 37 guaranteed saved units. There is no defensible additive “strategy 20%, tactics 50%” estimate without sustained, adaptive counterfactual games. Economy and promotion effects amplify operations: the human retains expensive veteran strength while Japan repeatedly buys the same strength back through Production.

### Ranked failures, interventions, tests and genes

1. **A nearby force is treated as an arriving force (implemented).** `crates/ai/src/siege.rs::raid_field_army` used all troops within `raid_reach=6` to approve a field raid, then independently chose each unit's destination. It checked city damage but did not price the target's reply or whether the supporting units could arrive that turn. T142: Phalangite 97 at (12,12), 100 HP, is sent to (14,10), beside veteran 40 at (15,10). T143 it skips and dies to 135 damage. Musketman 108 and Phalangites 109/115 then die on T144/T145/T146. **Four units lost in four consecutive turns to that shooter, no corresponding Spartan unit loss.** Not all four are claimed to be prevented by the fixture.

   **Value fix:** `raid_arrival_weight` (0 default, 0–300): positive values count only a group of at least two units that can reach distinct firing/screen tiles this turn on known routes. Compare its attack force, less weighted forecast reply losses, with the target's defense. At 100, only the force surviving the visible enemy reply counts. Reserve the group for final traffic reconciliation; a blocked member cancels the group. A rejected raid leaves units available for safer staging. Weight 0 executes the original branch unchanged.

   **Executed fixtures:** `sparta_142_raid_counts_arrivals_before_sending_the_first_phalangite` restores the original fogged view **and persistent memory**, reproduces the old move at 0, validates the new full order list against the rebuilt raw state, and resolves T142–143 with the recorded human/other-AI orders. At 100, unit 97 reaches (12,11) with 100 HP and survives T143; at 0 it dies. The paired test `raid_weights_never_credit_troops_that_cannot_arrive_but_allow_a_ready_force` covers weights 1/50/100, rejects a lone arrival supported only on paper, and still allows a strong ready force to raid. These short resolver checks run on current code; the historical full-tape hash/brain reconstruction separately runs on the original engine. Human adaptation is not modeled.

2. **A safe Settler trip can end in an undefendable city (implemented).** `civilians.rs::choose_city_site` valued site quality and travel/interception risk but did not subtract the imminent loss of the newly founded city. Japan founds Kyoto T129 at (16,11). Its own view contains level-three Arquebusier 40 at (13,9), five hexes away. The human advances to (14,10), then (15,10), and shoots T131 for 165 damage. Kyoto is at zero HP by T133. T130 Kyoto produces 4 P/turn, shows Walls 14 turns away and an Archer 32 turns away; that Archer costs 264 Gold against Japan's 66. It was not a city that an emergency purchase could save.

   **Measured amplification:** between pre-T129 and pre-T154, unit 40 gains 188 XP; **110 XP (58.5%) comes from eight Kyoto city shots**, including repeated zero/low-HP city attacks. It reaches level 4 T133, level 5 T140, level 6 T153, then upgrades to Field Cannon T154. A defensible placement does not erase its earlier kills, but removes this particular city/XP opportunity. Kyoto is captured T135, recaptured T136, permanently lost T158. The capture chain and the 37-kill veteran are overlapping evidence, not separate percentages to sum.

   **Value fix:** `founding_survival_weight` (0 default, 0–400) subtracts site value × forecast fraction of city HP lost before Walls × weight. It uses only visible wartime shooters, known passable routes, border permission, terrain-aware line of sight, promotion/HP-dependent ranged strength and current-era city defense. It considers a bounded near-term approach and at most two early shots per shooter, not hidden troops or a future enemy tech. Keeping the Settler is a zero-value alternative. The Walls estimate is deliberately optimistic; this is a bounded heuristic, not a complete settlement combat simulation.

   **Executed fixture:** `exposed_kyoto_is_deferred_for_a_walled_home_city` proves weight 0 founds, 200 defers and routes unit 89 through (15,12), (14,13), (13,14) to an existing walled Japanese city. Every path step is known/passable/legal in the view. Merely shifting the proposed city to (17,13) still forecasts full HP loss. Remove only visible shooter 40 and founding wins again. The full-memory position replay independently produces the same deferral. Unlike the raid fixture, this intervention has not been resolved against adaptive enemy movement or a new full game.

3. **Holding a damaged city can consume a doomed garrison (follow-up, not implemented).** `military.rs::hold_captures` acts before later recovery; `tactics.rs::garrison_doomed` emphasizes melee capture rather than all ranged overflow. Kyoto's Musketman 94 remains at 65 HP before T141 and 4 HP before T142, then dies to city overflow. There are four such Japanese deaths, units 81/94/106/152 on T135/142/180/190, **4/53 = 7.5%** of losses to Sparta; 40 contributes to three of them.

   **Proposed value fix/gene:** `garrison_survival_weight`, default 0: compare expected retained unit value and healing after retreat against the city's probability-weighted survival/capture denial from holding. A garrison that materially prevents capture may still be worth sacrificing. **Proposed fixture:** rebuilt T141 Japan view/memory, unit 94, hold versus a legal retreat; resolve city and garrison damage and check that later fire-from-hold cannot overwrite the selected action. This is a narrower next experiment, not a promised four saved units.

4. **Tech and resource spending do not consistently produce the counterforce (follow-up).** `production.rs`, `military.rs::hold_or_upgrade`, `supply_value.rs`. Japan spends 1,312 Gold upgrading in the long war; there is no evidence of a large bank refusing all upgrades. Across its 192 turns, the ledger has **150 individually legal upgrade unit-turns, 123 not selected**, hence 27 selected. Those alternatives share a treasury/resource budget and cannot all be purchased. T150 has only 40 Gold: Archer 118 can upgrade for 36 but moves instead, and is still an Archer when it dies T159. At T158 it again has a legal 36-Gold upgrade but fires. Japan has Rifling and six Riflemen by T165, yet Coal is fully committed (six Riflemen plus a held Field Cannon), while the Spartan level-six Field Cannon remains untouchable. Iron/Niter are tight at T142, Coal at T158–165; Oil is available at T165. These are unit mix/availability tradeoffs, not total research paralysis.

   **Proposed value fix/gene:** `front_upgrade_value`, default 0: value the forecast damage/survival change from upgrading at the front against the current shot, movement and the best competing purchase; price the resource reservation and any delay. **Proposed fixture:** T150/T158 Japan Archer 118, with the actual 40/53 Gold budgets, compare upgrade versus marginal fire and require no double-spending. Causal share of losses from skipped upgrades is not yet identified; merely counting every unselected legal upgrade would overstate the problem.

5. **Campaign supply value can overwhelm the win model (follow-up).** `plan.rs::price_campaign` and `supply_value.rs::capture_value`. Athens T193 scores taking Wakayama at **+23,777**, including **+22,032 supply**, +1,785 break-hold and −40 era deficit; ordinary capture win-chance delta is zero. The actual calculation is 36 estimated Production/turn of supply gain/denial × `supply_term=400` × shortage multiplier 3 × capture probability .51. This one additive term exceeds the model's whole 0–10,000 win-chance scale. The deposits are real, but future, not-yet-built users are valued and the projected economy already includes supply effects. The trace does not establish that removing it would avert Sparta's win; Athens still needs to contest the countdown.

   **Proposed value fix/gene:** `supply_campaign_discount`, default 0: discount the extra resource term by actual build readiness, time before victory and overlap with the projected economy benefit; keep break-hold value separate. **Proposed fixture:** T193 Athens view/memory, Wakayama target; compare `supply_term=0` with the old score and the discounted score, preserving an independently justified hold-breaking attack. No claim of a measured outcome share yet.

### Tactical checks that do not support a blanket “combat math is broken” conclusion

The human's veteran range, promotions, terrain and timing make the local exchange radically different from the army-count ratio. Japan does shoot back and sometimes kills expensive human units; Athens kills two assaulting Spearmen at Marathon. Immediate counterattack logic uses forecasts, but the field-raid path bypassed that protection by **moving into** the next shot. Cheap ranged return fire dealing 3–8 damage does not neutralize a unit killing one replacement per turn. The new arrival valuation uses the engine's ranged/melee forecasts and the existing one-attack-per-visible-enemy staging model; no new triangle math is hard-coded.

**Outrider is not in this recorded engine or the audited master baseline.** The separate era/Outrider rule branch failed `gate-audit2` and was not integrated. Its rules cannot explain this save. The premodern triangle still matters for real forecasts; no evidence here isolates a triangle implementation error. Focus-fire, flanking, damaged-target selection and chokepoint occupation remain visible in the complete battle/position ledger, but this analysis has not established a separate preventable-loss percentage for each. Existing ranged-survival and melee-response fixes do not cover every movement pass.

## Did the off defaults hurt?

Cross-check: [threat static audit](static-audit-2026-10-24.md), [tactics static audit](static-audit-2026-10-24-astra.md), [gate-audit2](gate-audit2/README.md). The static claims were hypotheses; the gate's measured results take precedence over enabling a whole group because one symptom sounds similar. Border threat failed by −12.2 pp on the seat; city defense by −6.8 pp. The recorded game predates that mixed-default integration.

I replayed the current Hard planner on the original **pre-plan memories and fogged views**, changing only each group's decision fields. Border on = `peace_exposure_percent=100, threat_horizon_turns=2, assessment_interval=1, army_threat_weight=100, field_strike=100`; city on = `scout_threat=100, walled_defense_percent=100, counter_composition=100`. Both together are also recorded. This is a one-step sensitivity check, not a new whole-game outcome.

| Position | Border on vs current defaults | City defense on vs current defaults | What this establishes |
| --- | --- | --- | --- |
| T18–19 Celts | Redirects Scout 5 (T18 toward (10,5), T19 (11,9)); no new Walls/army queue | No order changes | Peace-time threat blindness exists, but these toggles do not buy a timely Bibracte defense here. |
| T37–38 Athens | Moves Archer 12 toward (26,14); T37 also redirects Scout/research | No order changes | There is a real prewar positioning response. Whether it saves Marathon needs a longer intervention. |
| T129 Japan | Changes Sapporo's queue from Spearman to Laboratory, reroutes several units, upgrades 75; still founds Kyoto | No order changes | Neither group prices the new city's survival. Turning on “threat” is not monotonically more military. |
| T142 Japan | No order changes | No order changes | **Both off defaults leave the observed raid bug untouched; both on also leave it untouched.** New arrival gene changes 97/108 instead. |
| T164 Japan | Market → Machine Shop in Takamatsu; same unit orders | No order changes | Neither prevents this wave's field deployment. New arrival gene changes 112/119/131/132. |
| T193 Athens | No order changes | Adds Cavalry queues in Phaleron and Laurion | Counter-composition does respond to visible artillery. No measured proof those units arrive in time to stop the win. |

Current defaults match the entire recorded order list at six of eight sampled positions. T37 Athens and T193 Athens differ because current code is newer; **421/421 matching plans refers only to the original-engine replay**, not to these comparisons. The off groups plausibly weaken some local preparation/counter-composition, but the strongest demonstrated Japanese failures persist with both groups enabled. This game is not evidence to override their failed benchmark defaults.

## Japan's snowball (data only)

Each cell is **cities / population / tiles; Food / Production / Gold / Science per turn**, from the player's own pre-turn city yield breakdowns (gross city yields, not net treasury income).

| Turn | Japan | Athens | Sparta |
| --- | --- | --- | --- |
| 19 | 1 / 4 / 10; 17 / 15 / 1 / 4 | 1 / 4 / 10; 18 / 14 / 3 / 4 | 1 / 4 / 10; 17 / 15 / 1 / 4 |
| 71 | 6 / 25 / 59; 117 / 87 / 8 / 12 | 2 / 12 / 24; 38 / 35 / 9 / 14 | 5 / 19 / 50; 69 / 61 / 20 / 10 |
| 100 | 7 / 40 / 80; 142 / 115 / 15 / 25 | 3 / 17 / 36; 55 / 60 / 12 / 15 | 8 / 37 / 89; 136 / 95 / 41 / 21 |
| 150 | 10 / 71 / 127; 233 / 169 / 28 / 48 | 4 / 30 / 56; 88 / 81 / 27 / 37 | 12 / 73 / 153; 250 / 131 / 73 / 45 |
| 165 | 10 / 77 / 130; 222 / 173 / 30 / 50 | 4 / 33 / 59; 91 / 84 / 33 / 45 | 14 / 89 / 181; 297 / 141 / 64 / 48 |
| 185 | 5 / 33 / 61; 83 / 100 / 27 / 26 | 4 / 36 / 62; 95 / 91 / 43 / 51 | 19 / 149 / 265; 378 / 167 / 75 / 75 |

Celts at T19 had 1 city, 3 population, 9 tiles, and city yields 14/10/3/3; they are eliminated T22.

The mechanism is visible in the yield sources: Japan can space cities three hexes apart; each other Japanese city within five hexes adds 7% Food and Production, plus the flat +1 Food/city and its Food building. Population growth claims another tile, and cities work their owned territory, feeding back into yields. At T150 the local-neighbor percentage lines add **65 Food and 57 Production**; flat Japan Food adds 10; capital-distance penalties subtract 10 Food, 9 Production, 3 Gold and 3 Science. Net Japan-labeled contributions are therefore +65 Food/+48 Production/−3 Gold/−3 Science. Those are breakdown components on the actual population, not a simulation of Japan without its trait.

Japan is far ahead of the surviving AI in population/Production (T150: 71 vs 30 population; 169 vs 81 P), but **not ahead of the human in everything**: Sparta already has more tiles and much more Gold output. Japan's Food/network advantage supports replacing troops; it does not make up for the observed exchange rate. No faction change belongs in this branch; `india-faction-proposal` owns that design work.

## Reproduction, fixture provenance and handoff

The extraction extends the existing archive tooling documented in `docs/game-archive.md` and the tape/media workflow (`claude/trailer:docs/media/v2/HANDOFF.md`). `extract-state.sh` checks out the save's engine temporarily and builds/runs on the cloud box. `--audit-plans` restores continuous original-brain history while the recorded orders, not newly chosen ones, advance the tape. Full states are diagnostic inputs only; every planner receives its own `PlayerView`.

```sh
tools/scenarios/extract-state.sh ~/LegatumRuns/bug-saves/sparta-win-final-20261025-0902.legatum   --out ~/LegatumRuns/sparta-audit-1025/states   --every 1 --from 1 --to 208 --audit-plans
python3 tools/ai-quality/sparta-1025/analyze.py   ~/LegatumRuns/bug-saves/sparta-win-final-20261025-0902.legatum   ~/LegatumRuns/bug-saves/sparta-crushing-japan-20261025-0841.legatum   ~/LegatumRuns/sparta-audit-1025/states /tmp/sparta-ledger.json
```

`replay_positions.py prepare STATES PLANS INPUTS` packages eight views plus original memories; `run INPUTS BINARY OUTPUT` runs five arms with `war_position`. The example derives the seat's brain seed from the original setup seed, preserves faction defaults while applying decision overrides, restores memory, and emits orders/production diagnostics/plan scores. Run its build and the batch on `tools/cloud/remote.sh`, with Cargo wrapped in `tools/sim/build-slot.sh --`; this is forty short planning calls, not a benchmark. To reproduce the two new genes alone, pass `{"raid_arrival_weight":100,"founding_survival_weight":200}` as the decision overrides.

Fixtures in `crates/ai/src/fixtures/sparta-1025/` are gzip copies of the original extracted view/state/memory, plus T142/T143 recorded orders. Raw-state fixture loading converts the JSON string seed to u64, as the WASM parser does; it does not hand-edit combat fields. The report ledger covers all turns; the fixture subset is intentionally limited to executed regressions. No `.legatum` file is committed.

Validation and limits:

- Original-engine reconstruction: **207/207 state hashes, 421/421 AI order lists**. Mid/final tape prefix: **184/184 turns**.
- Focused founding fixture: **1 passed**. Focused siege module, including historical two-turn resolution and low-weight/positive raid cases: **6 passed**.
- Final `cargo fmt --all`: **passed**. `tools/sim/build-slot.sh -- cargo clippy --workspace --all-targets -- -D warnings`: **passed**, exit 0. `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked`: **passed**, exit 0, **1,948 passed / 0 failed / 10 ignored**, including both pinned tapes and all 12 determinism tests. The wrapper used its documented niced local fallback when the box was temporarily unavailable; focused tests and forty final position comparisons ran on the box.
- Master integration: merged documentation-only master `1314ff164` after the full checks; no code conflicts or pin regeneration were needed. Post-merge `cargo test -p legatum-sim --profile release-checked --test determinism`: **12 passed**; `cargo test -p legatum-sim --profile release-checked --bin legatum-sim pinned`: **2 passed**, both exit 0, both wrapped in `tools/sim/build-slot.sh --`. The first pin selector used `--lib` and selected zero tests; the corrected binary selector above ran both actual pins.
- Post-merge receipts: `~/LegatumRuns/sparta-audit-1025-merged-{determinism,pins-binary}.log`; pin box job `20261025T163902Z-d54b9a`.
- Receipts: `~/LegatumRuns/sparta-audit-1025-{clippy,workspace-tests,siege-tests,positions-final}.log`. Final position comparison box job `20261025T163031Z-48d173`: **40/40 completed**, no order changes from the earlier diagnostic batch. Ledger regeneration reproduces the recorded data and verifies both saved snapshots exactly.
- Lead-owned human-seat/war benchmark gate: **not run here**. [Ready-to-run arms](sparta-1025/gate-arms.json) compare control, each gene and both, separately on the seat and opponents. Defaults remain 0. Safe staging can be too passive against adaptive opponents; the founding horizon is deliberately bounded; the gate must evaluate those tradeoffs.
- No claim that the new genes change the designer's final victory. Longer counterfactual wars would diverge in human choices, hidden scouting and diplomacy; holding the original human orders fixed for 151 turns would not be an honest estimate of challenge.

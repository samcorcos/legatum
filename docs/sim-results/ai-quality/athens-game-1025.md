# Athens win, 2026-10-25 — war and victory-race audit

**Switzerland declared a real containment war, but its planners did not assemble a usable attacking force.** An unrelated war with the Mongols reserved all three city garrisons *before* siege planning, even with no Mongol soldiers visible. This trapped Switzerland's best ranged unit at home. Most of the impressive-looking border army was still Ancient Spearmen and Archers; their refusal to walk into lethal city fire was often correct. The failure was sending those troops to stage indefinitely while keeping the modern artillery out of the operation. Later, Swiss bombers actually flattened the city, but the ground army never converted that opening into a capture.

**The AIs did not simply ignore the victory track.** Germany broke Athens's first countdown and needed only two more turn ends to win its own. Switzerland, Germany, Celts, Japan and eventually Sparta fought Athens; a Japan–Celts coalition formed. However, recognizing the leader did not reliably produce an executable way to stop it. Germany's late city-capture valuation included a fictitious loss of Athens's science crown and omitted the loss of Germany's own live Engine goal. Taking a city could therefore look like a decisive victory move while leaving Athens's countdown running and reducing Germany's VP.

These are AI changes, not weaker Walls, combat buffs, resource grants or extra vision. New behavior is opt-in: each new gene defaults to **0**, preserving today's policy until a separate quality gate measures it.

## Evidence and reproducibility

Source: a private copy of `athens-win-swiss-tepid-20261025.legatum`, playtest 8.9.10; engine `8e9433e5cf53ea4ffe4e8729d5904c6320cb519a`, setup seed `7970005121204451`. The source save was read and copied, never modified; both still hash to SHA-256 `4fae4d7b3e1e2e18a5e595445e10d09b3d7b529af583132ebe986c1e5561127e`. Athens is player/team 0; Germany 2; Switzerland 5. All eight seats are solo, seven AIs Hard. The tape has 223 resolved turns, ending in state T224.

- **223/223 original-engine state hashes match.** The extractor advances with recorded orders, not counterfactual AI choices.
- **1,561/1,561 original-brain full order lists match**, with continuous persistent memory and each seat's own fogged view.
- **196/196 instrumented position replays match the complete recorded order list**: Swiss T67–223 and German T185–223. Each replay restores the original *pre-plan* memory. The instrumentation records orders after each military pass, siege ratios and every candidate siege tile's rejection numbers; it changes no decisions.
- **28/28 additional strategic replays match**, covering every AI at T194, T201, T215 and T221. [Candidate scores and campaign blockers](athens-1025/strategic-trace.json.gz) include the alternatives rejected by the planner.
- [Complete evidence ledger](athens-1025/ledger.json.gz): every turn's orders, events, Swiss units and aircraft, economy/supply, public standings, known cities, raw cities/units and all AI plans. Raw state is labeled separately; it is not given to a planner.
- [Every Swiss war turn](athens-1025/swiss-turns.md), [every unit's order and assigning pass](athens-1025/swiss-unit-turns.csv.gz), and [original phase snapshots](athens-1025/original-trace.json.gz). The table covers all **77 war turns**: T67–96 and T177–223. The unit ledger also covers the intervening peace, making refits and standing garrisons inspectable.

Turn numbers in the tables mean the state *before* orders for that turn; a capture or crown change in events happens at that turn's resolution. Unit names in machine files are stable Rust identifiers: `Soldier` = Spearman, `Ranger` = Archer, `Trooper` = Pikeman, `Marksman` = Crossbowman, `AssaultTrooper` = Musketman. Coordinates are axial.

The requested `.reviews/astra-analyze-sparta-game-1025.md` is not tracked on the named Sparta branch. Its tracked `docs/sim-results/ai-quality/sparta-game-1025.md` and extraction/example tooling on `astra/analyze-sparta-game-1025` were used; this branch imports the tooling without importing Sparta's behavioral changes.

## Switzerland: what actually stopped the army

### Declaration, commitment and outcomes

T67: `plan.rs::contain_candidates` wins with **+185**: VP denied +53, leader's race +183, war footing −39, attrition −12. This is an explicit response to Athens, not a random war. T68 switches to Peace-with-Mongols at value 0; T77 resumes Contain Athens at +131. A campaign on Athens city 14 starts T80, ends on a plan change T90, restarts T91, fails value T92, restarts T93, and ends when Peace takes effect T97. Across this first war Switzerland makes **zero attacks on Athens**; there is one Athens–Swiss battle, with Athens attacking.

T177: Switzerland declares again, selecting Contain Athens at **+101** (23 VP denied +92 leader's race −11 footing −3 attrition). It keeps fighting Athens through the end. Campaigns on city 14 start T179 and T182 and fail `insufficient_army` T180 and T184; the next begins T210, is replaced alongside the plan switch T215, and fails T222. This is not a campaign waiting to declare, nor late peace valuation quietly ending the war.

Across T177–223 there are **26 Athens–Swiss battle events**: Switzerland initiates four land ranged shots and seven bomber strikes. It loses eight military land units plus a Scout to Athens and captures **no city**. Its land shots at the human Artillery do only 3–4 damage each. The bomber attacks are materially effective: T208 removes 179 city HP; T209 two strikes remove the remaining city HP; T213 they kill an exposed human Artillery; T222 they again reduce the city to zero and clear its garrison. The problem is conversion and coordination, not complete refusal to use weapons.

### The force was large in bodies, old in combat power

All military land units are at 100 HP at T177 and T180. Only Spearman **123** has a promotion, level 1 (16 XP); the other listed Swiss units are level 0. The detailed unit ledger retains HP, XP, level, fortification and both strengths separately, rather than treating unit count as combat power.

| Pre-turn | Swiss military land army (Scouts excluded) | Relevant target and readiness |
| --- | --- | --- |
| 67 | 7: 5 Spearmen (strength 20), 2 Archers (15 ranged / 8 defense) | All level 0; unit 18 has 53 HP / 8 XP, others 100 HP / 0 XP. Four within six hexes. Known city 14: 200 HP, population 1, Walls, strength 22; human Spearman 3 garrisons it at 100 HP / 7 XP / fortify 6. |
| 80 | 7: 5 Spearmen, 2 Archers | All 100 HP / level 0; only three are free for the siege (50, 84, 103): weight **55**, defense **66**, **83% < 170%**. Six within six hexes. Remembered city still 200 HP / strength 22, same actual garrison. |
| 177 | 15: 9 Spearmen, 3 Archers, 1 Crossbowman, 2 Musketmen | Athens city 14 at (22,15): 200 HP, population 3, Walls only, strength 47; actually empty. Only eight military units are within six hexes at declaration. |
| 180 | 16: 10 Spearmen, 3 Archers, 1 Crossbowman, 2 Musketmen | Same city, 200 HP/47 strength. Twelve free nearby units contribute **273** attack weight; defense weight **94**; **290%** exceeds the Swiss **170%** siege threshold. Most still cannot survive a firing/screen tile. |
| 185 | 14: 8 Spearmen, 3 Archers, 1 Crossbowman, 2 Musketmen | City now strength **62**, still empty; human Artillery 102 is at (22,17), outside Swiss live sight. Retreats remove both Musketmen before late siege; nine free units contribute **167**, defense **124**, margin **134% < 170%**. |
| 193 | 11: 6 Spearmen, 2 Archers, 1 Pikeman, 1 Musketman, 1 Arquebusier | Human Artillery 102 now actually garrisons the city (89 HP, 55 XP), unseen by Switzerland. Fogged defense weight 186; free force 176; **94% < 170%**. |
| 210 | 10: 5 Spearmen, 2 Archers, 1 Phalangite, 1 Musketman, 1 Artillery; plus 2 Bombers | Bombers had flattened city 14; actual city HP **0**, human Spearman 3 garrison **100 HP / 7 XP**. Switzerland remembers **21 HP**, strength 59, with no live garrison observation. Seven free units weigh **132**, remembered defense **18**, **733% > 170%**—still no survivable siege tile for any of them. |
| 220 | 9: 4 Spearmen, 1 Archer, 1 Pikeman, 1 Musketman, 1 Rifleman, 1 Artillery; plus 2 Bombers | City has healed back to **200 HP**, Spearman 3 still inside; Swiss memory still says 21 HP. Free force **129**, defense allowance **67**, **192% > 170%**; every free unit fails siege placement. |

The city and raw garrison column is hindsight for auditing, not information granted to the AI. City 14 is a small noncapital city named Athens; the human capital is Phaleron. Reaching this nearby city was itself failing, before any question of marching on the distant capital.

### Exact decisions and numbers

1. **The modern unit was reserved for the wrong war.** `military.rs::plan_military` calls `garrison_cities` before coordinated assaults and early siege whenever at war with the Mongols. Only Sparta previously applied `mongols_guard_radius`; Switzerland did not. Swiss–Mongol war persisted from T90, but there are **no visible Mongol units** in the late positions. The original trace shows `garrison_cities` assigning 49, 62 and 90 before `siege_war`. Unit **62 stays at Winterthur (18,16), four hexes from Athens**; it upgrades there from Crossbowman to Arquebusier and eventually Artillery (T210: 100 HP, 77 melee defense, 98 ranged, level 0). The global faction-war flag consumes precisely the unit that could survive and bombard. Routine home defense and later garrison assignment still exist after this gate.

2. **The strength ratio is not the final attack gate.** `siege.rs::siege_war` checks `attack_force(near) * 100 / city_defence_weight`, then `siege_city` separately requires a free reachable tile, line of sight, acceptable enemy exposure, and survival after city fire. At T180 Archer 242 has eligible tiles with zero enemy-unit exposure, but **maximum city strike 171 vs 100 HP**; Spearman 103 faces **105 vs 100**. Both get no spot. Musketmen 126/192 face **40**, receive approach orders, then are removed by recovery/exposure passes on later turns. At T210 the free Archer faces **266**, Spearmen **171**. At T220 the newly upgraded Pikeman 103 still faces **105**. The refusal is the city-strike survival predicate, not an overweighted human garrison or an unmet 170% ratio on those turns. Charging these units alone would not repair the army.

3. **Safe staging becomes a permanent destination.** After siege fails, `military.rs::campaign` holds a unit already within the staging radius on a `safe_campaign_tile`, otherwise picks a safe ring through `campaign_staging_spot`. On T210, all seven free units get `SkipUnit` from this pass. This is observable order provenance, not a guess based on where sprites stood. On T214/T220 the same pass upgrades unit 103; the rest continue skipping. The planner has no operational requirement that a failed siege must release the capable garrison, refit the obsolete army, or withdraw to a different attainable objective.

4. **Air successes are not handed off to ground operations.** Air orders are appended after ground planning (`scripted.rs::orders`, `air.rs::air_orders` in the recorded engine). The two Bombers open a capture window T209, but no coordinated melee follow-through takes city 14. Once the live sighting expires, planes have no visible target to strike and ground staging does not refresh it. The city heals from 0 to 200 HP. The final T222 strike opens another window with only one recorded turn left. This deserves an explicit air-plus-ground capture scenario; the small garrison fix below does not claim to solve all such coordination.

5. **Refitting and supply matter.** At T177 Switzerland has 207 Gold, +7 net Gold/turn, 39 military upkeep; Musketman 126 cannot upgrade to Rifleman because there is no Coal. It also cannot buy Siege because all three Iron are committed. These are real resource constraints. They do not explain leaving its existing modern ranged unit at home. The ledger includes each turn's upgrade costs/blockers and supply; it should not be read as if every old unit could be upgraded simultaneously with the same treasury.

## VP race

The solo target is 10 VP held for 10 consecutive turn ends (`core/src/experiments.rs::solo_vp_target`, `core/src/scoring.rs`; rule reference section 6). The reaching turn counts.

| After turn | Athens | Germany | Decisive change |
| --- | ---: | ---: | --- |
| 193 | 10, hold 1/10 | 7 | Athens completed its +2 faction goal and began its first hold. |
| 201 | 9, hold broken after 8/10 | 9 | Economy crown moved Athens→Germany, from a 206–201 lead before scoring to 206–207 after scoring (`T201`/`T202-p2.view.json`; chronicle `HoldBroken`). |
| 206 | 9 | 10, hold 1/10 | Germany scored era 7's Factory objective. Sources: 4 era VP (2, 5, 6, 7), +2 Great Work, +2 live Engine goal, Economy and Army crowns. |
| 213 | 9 | 10, hold 8/10 | Athens was one attack turn from disrupting Germany's near win. `T214-p2.view.json` publicly showed Germany's 10/10, 8 held, and its endangered Germany Colony 8. |
| 214 | 11, new hold 1/10 | 9, hold broken | Athens took Germany Colony 8, flipped Economy crown (202 versus Germany 186 next view), and scored its overdue era 5 "take a city from the VP leader" objective. Germany needed the Economy crown and had zero VP margin. |
| 219 | 10, hold 6/10 | 8 | Germany recaptured Colony 8 on T216, then captured Athens Colony 13 on T219. That capture moved Economy back to Germany but added an Engine-less city, losing Germany's live +2 goal; Athens retained Most Advanced and a 10 VP hold. |
| 223 | 10, hold 10/10; won | 8 | Athens completed the hold. |

Athens' second hold began at **11 VP**, so the Economy crown flip on T219 took it to 10 without breaking the countdown. Its final 10: five era objectives (2, 3, 5, 6, 7), +2 goal, +2 Great Work, Most Advanced. Germany's final 8: four era objectives (2, 5, 6, 7), +2 Great Work, Economy and Army; its goal was absent because 8 of its 9 cities had Engines (`T224.json`). Sources: `history.chronicle` in final save, `T194`, `T202`, `T207`, `T214`–`T223` player views, and `core/src/goals.rs::reached`/`engine_cities`.

## What the AIs did to Athens

Athens was publicly at 6 VP before T177, 7 before T183, 8 before T193, and 10 with a running hold before T194. The response was already visible before the countdown: Germany declared T162 and Switzerland T177. Earlier Swiss containment at T67 also explicitly priced Athens's lead. The relevant question is why those responses did not remove enough live VP, rather than whether the AIs could read the track.

| AI | Response while Athens led / held | Effect by T223 |
| --- | --- | --- |
| Byzantium (P1) | Stayed at Peace with Athens. Its Hard plan was Peace with Sparta at T194–207, then campaigned against Sparta. | No action against Athens. Captured Sparta's Corinth on T219, scoring its own era 6 objective. |
| Germany (P2) | Declared on Athens T162 and planned Contain until T202. A campaign on Athens city 39 ran T200–202, then ended `negative_value` after Walls; plan switched to Peace with Athens at T202 even while the war persisted. Held its own 10 VP T206–213. | Recovered Colony 8 on T216 and captured Athens Colony 13 T219. Its first hold was broken by the earlier Colony 8 loss; later capture hurt its own VP. |
| Mongols (P3) | Stayed at Peace with Athens; planned Peace or Campaign against Sparta. | No Athens city capture, attack war, or hold break. Captured Sparta's Matsue T223. |
| Japan (P4) | Planned Peace with Sparta through T214. Declared on Athens T215 and later planned Contain Athens; allied with Celts as an anti-Athens coalition on T202–211, broke it, then renewed T218. | No Athens city capture or VP denial before the win. |
| Switzerland (P5) | Declared on Athens T177; planned Contain Athens through the late game, with a campaign targeting Athens city 14 by T214. Later allied with Sparta against coalition target Athens on T221. | No Athens city capture or title/goal denial. See separate Swiss military audit for why its siege did not advance. |
| Celts (P6) | Declared on Athens T194, immediately after the first hold started. Razed Athens Colony 15 T196; maintained Contain/Campaign against Athens and allied with Japan (T202–211, T218 onward). | The raze did **not** break Athens' 10 VP hold. Athens lost its hold only to the Economy crown flip on T201. No later VP denial. |
| Sparta (P7) | Its Hard plan remained Contain Mongols, including at T221–223. Declared on Athens T221 while already fighting other fronts; allied with Switzerland against Athens on T221. | No Athens VP denial. |

The anti-leader diplomatic response widened late: Athens faced Germany and Switzerland at War when its first hold began, Celts joined T194, Japan T215, Sparta T221. Byzantium and Mongols never fought Athens. Two anti-Athens Alliances existed at the finish (Japan–Celts, Switzerland–Sparta), but no member other than Germany or Celts took or razed an Athens city in this period (`T193`–`T223.json` diplomacy relations; final `history.chronicle`). "War" and "Contain" therefore overstate effective blockade here.

### Where leader-stopping logic fires and where it stops

Original-engine `plan.rs::contain_candidates` offers every rival with a live hold, as well as `contain_leader()` and the diplomatic target. It discounts containment by geographic reach: `distance = min(nearest_known_city, nearest_army_to_known_city)`, missing distances become **99**, and `reach = clamp(15 − distance, 0, 10)`. Its hold pressure also uses `front = clamp(2 × turns_left − distance, 0, 10)`. Public victory knowledge therefore does not supply hidden city coordinates.

- **Byzantium and Mongols:** at T194, T215 and T221, `campaign_against(Athens)` returns **“no city seen yet”**. Each has a Contain Athens candidate at **0**, including **0** leader-race and **0** hold-contest terms, tied with Peace at 0. The leader/hold logic fires; reach and the missing target make it inert. There is no demonstrated fallback that discovers an Athens objective in time.
- **Japan:** T194 has the same no-city/zero-value problem. At T215, Contain Athens rises to **+1,000**, entirely `contest the hold`, beating Peace **0** and the Sparta campaign **−80**. It declares, but `campaign_against(Athens)` returns **“already at war with another empire, no second front”**. That is the function's diagnostic after finding no usable forecast, not proof that a second-front flag alone caused the failure. The important verified mismatch is a positive containment value with no funded capture outlook. At T221 the containment value is back to **0**; the countdown is nearly over.
- **Sparta:** before its late alliance war, the faction-specific `contain_commit` filter requires a funded campaign with at least `contain_min_odds`. With no known Athens city, its Athens candidate is omitted altogether; it keeps Contain Mongols at **0**. The T221 alliance brings it into the war without replacing that operational target.
- **Germany, Switzerland and Celts:** the response produces real wars and, for Germany/Celts, city captures. Their failures are respectively incorrect VP accounting and a lost home countdown, an unusable siege force, and a raze that does not remove enough VP. Targeting any leader-owned city is not equivalent to targeting the VP that keeps its hold alive.

## Germany's win line and what it saw

**Known and near-certain rule line:** keep at least 10 VP through T215. At the start of T214 Germany had held 10 for eight ends, requiring the ends of T214 and T215. Its 10 included two live crowns and the live Engine goal; any one-point loss broke the hold. The T214 German view exposed Germany Colony 8 at **72/200 HP**, with an Artillery garrison at 100 HP, Athens Artillery at (27,24) 4 HP and (27,22) 100 HP, and an Athens Tank at (27,23) 82 HP. The city was a material Economy contributor; Germany led the public crown 215–202 before the attack (`T214-p2.view.json`). Keeping this city/crown for two more turns, or earning one extra permanent VP buffer, would have completed Germany's hold first. Its unscored era 3 and 4 objectives were far from threshold at T214 (Build 0/4; expansion 1/42 or 1/34; Fight 0/38), so a quick permanent buffer was not an evident practical line. This is an inference about feasible alternatives, not a what-if replay.

The original planner **saw the countdown and visible assault**. It switched from Contain to Peace with Athens at T202; that Peace plan remained active at T214 despite the continuing War. The defensive passes did run. T211 `defend_cities` upgraded garrison unit 245 from Field Cannon to Artillery and moved units 177, 257, 282 toward Colony 8. At T214 `hold_captures` initially skipped the garrison, `defend_cities` skipped all three nearby units, and `fire_from_holds` changed garrison 245 to shoot Athens' 4-HP Artillery at (27,24). Athens' other Artillery shot the city and its Tank moved in and captured it that same turn. This establishes a failed defense, not a complete failure to detect the attack (`athens-1025/original-trace.json.gz`, P2 T211–214; `source.legatum` tape orders; `T211`–`T215-p2.view.json`). Whether a different legal T214 tactical order would save the city needs a replay; this audit does not claim it.

After T214, simply regaining Germany's Economy crown was insufficient to win before Athens: Germany's prior hold was gone, and Athens had an 11 VP hold already running. Germany needed to break that hold and restore its own 10 VP, then sustain a new 10-turn hold. Its T219 plan instead projected that taking Athens Colony 13 would break Athens' hold. The original T219 plan recorded a **+9,546** campaign value, including `capture at 58%` +4,309, `supply` +3,248 and `break the hold` +2,030. The actual capture flipped only Economy. Athens stayed at 10 VP, while Germany's newly owned Colony 13 had no Engine and its live goal fell from 2 VP to 0 (`T219`/`T220-p2.view.json`, P2 memory plan event, `T224.json`).

Two original-code model errors explain the T219 misread. `crates/ai/src/hold.rs::science_loss_on_capture` assumes Most Advanced is rolling Science from cities, but VP v2's `crates/core/src/scoring.rs::title_metric` awards it by furthest era and known technologies: capturing a city cannot undo either. `crates/ai/src/plan.rs::price_campaign` added captured city output and rival losses but never subtracted the German captor's own live 2 VP Engine goal. The current branch still had both defects before this audit. A default-off `capture_vp_accounting` gene now makes the science-title projection obey `view.rule_experiments.vp_v2()` and charges a held German Engine goal when capturing a city not known to have an Engine. Enemy building detail is private, so the latter is a conservative assumption; a recaptured former German city could already have one. On the actual T219 German fogged fixture, current candidate value changed **10,070 → 3,374**, its false `break the hold` term disappeared, and `capture at 58%` changed **4,838 → 172**. These are corrected current-branch plan scores, not a replayed game outcome.


## Comparison with Sparta

The Sparta analysis on `astra/analyze-sparta-game-1025` found an arriving-force illusion, isolated troops sent into a veteran shooter's next attack, vulnerable city founding, and campaign supply values exceeding the model's win scale. Athens reproduces the **operational fragmentation**: a nominal army is split between ceremonial home defense, safety retreats and static staging; a few units enter combat without a useful supporting force. Human veterans again punish poor exchanges, and basic units remain in service far past their useful frontline era.

The sharp new findings are **global anti-Mongol reservation stranding a nearby modern gun**, **successful bombing with no capture handoff**, and **capture VP accounting that disagrees with the actual victory rules**. The Swiss city-strike veto itself is frequently sensible. Removing safety checks wholesale would reproduce Sparta's piecemeal losses.

The common cause is disagreement between layers: strategy prices a war or capture using one force and one projected reward; operations reserve a different force; tactics independently veto its arrivals; movement can cancel orders again. No shared, verified commitment says “these units can arrive, survive and take this objective before the victory deadline,” nor requires a new plan when that commitment becomes impossible.

## Implemented fixes and bounded fixtures

| Gene | Default / candidate | Behavior and evidence |
| --- | --- | --- |
| `local_mongol_guard` | 0 / 1 | Apply the existing five-hex Mongol-threat locality check to every faction before early garrison reservation. Genuine nearby Mongol threats retain their home gun. Actual T210 Swiss view/memory: artillery 62 advances to (20,15), then resolves attacks against city 14 on T211 and T212; Switzerland owns the city by the end of T215. At 0 it stays home throughout T210–215 and the city remains Athens-owned. All Swiss orders are validated against the raw state before resolving. |
| `capture_vp_accounting` | 0 / 1 | Price captures with the actual VP-v2 science metric and Germany's own live Engine goal. T219 German fixture removes the spurious +2,030 hold-breaking term; candidate value falls 10,070 → 3,374. Separate tests cover the Engine loss and preservation of legacy VP science behavior. |

The Swiss fixture replaces only Switzerland's orders for six turns, restoring the original starting memory and then maintaining the brain normally; other seats keep their recorded replies. It runs on the current engine, so it is a bounded causal test of deploying the gun and taking the city, **not** an old-engine counterfactual win-rate estimate. The original-engine tape/brain match is a separate check. Neither gene claims Germany wins or Athens loses a full adaptive replay. The old army, campaign target selection, air/ground handoff and ability to defend Germany's countdown still need work.

Candidate [gate arms](athens-1025/gate-arms.json) separate each gene and both on the human seat and on opponents. Defaults stay 0: freeing garrisons can weaken an insufficiently scouted home, and Germany's missing-enemy-Engine assumption is intentionally conservative. These fixtures establish the mechanism, not an across-game quality gain.

## Structural proposal: a small outcome gate before a new war layer

Extend `scenarios/war/`, `tools/sim/war-scenarios.sh` and the real-resolver `combat_lab`; use the game's resolver and fogged views throughout.

1. **Six save-derived cases:** Swiss T180 (refit or commit), T210 (exploit bombing), T220 (urgent VP denial); Germany T214 (defend its countdown), T219 (price a real hold break); Sparta-game Japan T142 (avoid an unsupported first arrival). Keep two additional positions per failure family held out. Score city capture/retention by a deadline, actual VP and countdown changes, Production lost, healthy idle turns, committed units that arrive, and home losses after releasing guards. A predicted hold break that leaves the real hold running is a hard semantic failure.
2. **Costed gate:** start with deterministic assertions plus 16 paired short rollouts per case, baseline/candidate: **192 rollouts capped at 12 turns** (eight salts with recorded replies and eight with adaptive opponents). Budget **five spot-compute minutes and $1 per batch**, to be measured from receipts; these are targets, not observed costs. Estimate 1–2 engineer-days for fixtures/CI, another 2–3 for adaptive metrics and held-out splits. Run 64-salt and full-game seat/opponent gates weekly. No long local runs.
3. **Then change operations:** each campaign owns a unit roster, reserve, route, arrival times and capture/loss forecast. Each turn it must commit, reinforce/refit, or withdraw. Tactical rejections return explicit blockers; they cannot silently leave an impossible campaign staging forever. Include home-defense cost and the actual VP deadline. Test this behind a gene against the scenario gate and full games.
4. **Offline search:** use bounded resolver search over attack timing, allocation and refit choices. Distil successful choices into cheap live features; measure predicted versus actual capture, losses and VP denial on held-out cases. Stop scaling search when gains fail to transfer.

## Reproduce and validate

```sh
# Work from a copy of the supplied bug-save; never pass a live save as an output.
tools/scenarios/extract-state.sh "$SAVE_COPY" --out "$STATES" --every 1 --from 1 --to 223 --audit-plans
python3 tools/ai-quality/athens-1025/analyze.py "$SAVE_COPY" "$STATES" ledger.json.gz
tools/ai-quality/athens-1025/trace.sh "$STATES" "$WAR_TRACE"
tools/ai-quality/athens-1025/trace.sh "$STATES" "$STRATEGIC_TRACE" 8e9433e5cf53ea4ffe4e8729d5904c6320cb519a strategic
python3 tools/ai-quality/athens-1025/summarize_trace.py ledger.json.gz "$WAR_TRACE/trace.json.gz" "$TABLES"
```

Validation of the implementation (Rust code snapshot committed as `c2cb724e4`):

- `cargo fmt --all`: **pass**.
- `tools/sim/build-slot.sh -- cargo clippy --workspace --all-targets -- -D warnings`: **pass**, cloud job `20261025T235230Z-9b93d5`, exit 0.
- `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked -- --quiet`: **1950 passed, 0 failed, 10 ignored**, cloud job `20261025T235513Z-8e67b6`, exit 0; doctests also pass. Includes the Swiss capture, visible-Mongol guard, German capture and legacy-science regressions.
- The first full-test invocation stopped before running unit tests: the cloud Cargo shim translated Cargo's `--profile release-checked` into nextest's unrelated profile flag (job `20261025T235229Z-918a87`, exit 96). Adding the standard test-harness `-- --quiet` argument keeps plain Cargo on the same cloud box; the corrected invocation above completed. No local fallback or relaxed checks were used.
- Shell/Python syntax, compressed JSON decoding and `git diff --check`: **pass**. The supplied bug-save and private copy still have identical SHA-256 hashes.

Merged master `1178270dc` into this branch as `dd0e0fac6`; only design documentation changed, with no conflicts or pin regeneration needed. Post-merge checks:

- `tools/sim/build-slot.sh -- cargo test -p legatum-sim --profile release-checked --test determinism -- --quiet`: **12 passed, 0 failed**, cloud job `20261026T000138Z-2d42db`, exit 0.
- `tools/sim/build-slot.sh -- cargo test -p legatum-sim --profile release-checked --bin legatum-sim pinned -- --quiet`: **2 passed, 0 failed**, cloud job `20261026T000138Z-073ed7`, exit 0.

 The counterfactual fixtures use checked-overflow optimized Rust; the historical trace uses the recorded engine. This audit does not promote either gene or claim a full-game win-rate improvement.

Co-Authored-By: GPT-6 Astra (Codex) <noreply@openai.com>

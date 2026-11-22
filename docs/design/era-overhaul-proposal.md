# Era and research overhaul: proposal (draft, 2026-10-10)

> **The user's decisions on 2026-10-10 supersede parts of this draft. The live doc is (a private document, not published) Seven eras ending in a living Future era (drones, humanoid robots, fusion, quantum computing, recursive AI). Era length is set from the measured war cycle at both speeds. The team target is 20. No navy. Crowns, era objectives and escalating overreach are approved.

Draft from the lead for the user's review. Nothing here is decided until the user picks it. Decisions then go to the design doc's decisions log and the `legatum-design` skill. Standard speed unless marked; Quick is half. Every number marked "start" is a tuning knob, not a measured value.

**The ask, in the user's words:** "I'm at turn 170... I'm bored... all the science at this point is useless. I just pick whatever is closest to click... The eras should reflect human progress... when someone gets ahead it's a big deal. This whole game is basically 1 era."

**The recommendation in one paragraph.** Six historical eras, Ancient to Modern, each long enough for its armies to fight. Each empire enters the next era by finishing all 5 techs of its current one (the user's decision), so research is about order and timing. Entering an era is a celebrated moment: a full-screen takeover, and your cities and units redraw in the new era's look. Cities also grow visibly with population. Every tech unlocks something you can see, and the three land classes get a new generation across the eras, ending with tanks and aircraft. Victory points move from five size-linked titles to three crowns plus one public era objective per era that anyone can score. Each crown you hold costs Gold every turn, and the cost rises with your era. A leader can stay on top by banking permanent points and holding only the crowns it needs. Fifth place always has a cheap crown and open era slots to chase.

## 1. Diagnosis: why turn 170 is flat

### Predictable winners
Share of games where the eventual winner already leads on VP (late-game study, 256 games per lobby, Hard + Search, Standard):

| Turn | 4 players (fair 25%) | 8 players (fair 12.5%) |
|---|---:|---:|
| T50 | 61% | 51% |
| T100 | 73% | 65% |
| T125 | 81% | 69% |

The last change of VP leader comes at a median T95 in 4p games (about half of a 191-turn median game). The second half is a countdown.

### Titles cluster on the leader
- The top team holds 3.5 of the 5 titles on average (4p) and 3.1 (8p). One team holds 4 or more titles in 47% of 4p game-turns.
- Most Populous and Most Territory are nearly the same title (r = 0.80). Both correlate with Largest Economy (r = 0.39 to 0.49).
- Most Advanced is a five-turn Science total, so it also follows size.
- In all eight of the user's saves one team held 4 or 5 titles; six showed all five.
- The three milestones (Four Cities, Six Techs, Twenty Population) are thresholds everyone reaches, so they never separate teams.

### Research is a menu of small numbers
- The user's saves end with 5 to 14 of 32 techs, mostly eras 1 and 2.
- At T100 in sims, an average empire knows 6.0 / 3.6 / 0.04 / 0.00 techs of eras 1 to 4.
- Sim winners enter era 3 at T136 and era 4 at T166 (8p): the last two eras are squeezed into the final 40% of the game.
- 10 of 32 techs unlock no item at all, only a yield or a rule tweak. About 7 options are open at once and most read as "+3 of something". So "pick the closest" is a rational answer.
- Making research cheaper doesn't fix it. The pacing study's best setting (40% cost) missed both era-timing targets in every cohort (mean error 16 points), and it only compresses the same single-era content.

### Content and war
- 6 combat units in 2 classes (melee, ranged): no mounted units, no aircraft. The late units (Trooper, Assault Trooper) are bigger Soldiers.
- Unused content: Siege Engine was built in 1.6% of games, Planetary Foundry in 25%, Assault Trooper in 40%.
- Ranged dominates real wars. In 4p campaigns melee units killed 11,712 and lost 18,735; ranged killed 21,300 and lost 12,129. 6,643 melee units died before reaching a target. Yet in equal-cost stand-up fights a Soldier beats a Ranger 92% of the time. The cause is range plus fire before movement, which is kiting, not raw stats.

### The AI stops pressing
At T200, cities are captured at most 0.02 times per living team-turn and wars declared 0.01 to 0.02 times, while battles run at 1.2 to 1.5 per team-turn. That's skirmishing without conquest: "at war but not really attacking". With every army the same generation, defence is cheap and attacking rarely pays.

**Root causes:** one era of content, research effects that are numbers rather than new things, VP that rewards size five times over, and no reason for anyone to attack the leader late.

## 2. Design goals

| Goal | Pillar |
|---|---|
| Every research pick is a strategic choice, in every era | 1, 10 |
| Eras feel like real human history: new units, buildings and decisions | 2, setting |
| Getting ahead an era is public and a big deal | 4 |
| Entering an era feels like an achievement and changes the board | 4, presentation |
| Each era's units have time to fight wars | 10 |
| Nobody knows the winner at T100; fifth place has a path back | 11 |
| A well-playing leader can stay on top | 11 ("deserved lead"), user |
| Turns don't grow; every cost has a breakdown | 3, 5 |
| Integer maths, simultaneous turns, one order per unit | 7, engine |

## 3. Era structure

### Six eras

Each era asks a new question, not just bigger numbers. Entry is for the median winner in a ~210-turn Standard game.

| Era | The new question | Entry |
|---|---|---|
| Ancient | Rush, expand or wall up? Spears, bows, horses | Start |
| Classical | Claim land before borders close, or start a Great Work? Roads, catapults | 15% (~T30) |
| Medieval | Castles make cities hard: which do you fortify? Knights finally answer archers | 30% (~T65) |
| Renaissance | Bombards end the castle age: the first gunpowder empire has a siege window | 47% (~T100) |
| Industrial | Factories that pay back slowly, or strike now with rifles and cavalry? | 63% (~T135) |
| Modern | Tanks and bombers break any line: the fight over the final hold | 80% (~T170) |

**Why six.**
- A 3 to 6 hour game is 190 to 220 turns at Standard, so each era lasts about 35 turns: long enough to feel, short enough that the next is always in view. Four eras can't fit spears, knights, gunpowder and tanks; eight (Civ 6) blur together. Humankind uses the same six.
- The game stops around 1940: nuclear weapons and computers would add a seventh era and a different kind of war.

### Time to breathe (user, 2026-10-10)
- Every era lasts at least 30 turns for the median winner (15 at Quick); the sim guard is "no winner's era shorter than 25 turns".
- With 5 techs per era, a tech lands about every 7 turns. Research a unit tech first and its unit fights for 25+ turns before the next era opens, and keeps fighting until it is upgraded.
- Per-empire eras: each empire enters an era by its own research, Civ 6 style. No global cutover, reset or civ swap.

### The era-up moment (user, 2026-10-10)
Finishing the last tech of an era is the game's big recurring reward.
1. **Takeover.** Your next turn opens on a full-screen era card: the era's name, painted key art of your people in that era (Mongol horsemen on the steppe for Medieval), and a music sting that hands over to the era's own music. Three lines say what changed: new units and buildings to research, the era's objectives are open, crowns that moved. It's held until you act, like the tech unlock card, and skippable. It plays on your screen only, so nobody waits (pillar 7). It uses the celebration layer at its top tier, like Victory.
2. **The board changes.** Every one of your cities redraws in the new era's architecture, and your units' figures change to the era's dress. Looks only: stats change through the upgrade order.
3. **Rivals see it.** A public banner ("Japan has entered the Industrial era"; larger for the first empire into an era), the era badge on the VP panel flips, and your cities show the new architecture on every map that has explored them. You can tell who is ahead by looking at their cities.
- No extra gameplay bonus on entry: the unlocks, the open objectives and the Most Advanced crown are the reward, which avoids feeding a snowball.

### Cities that grow (user, 2026-10-10)
- Today cities are abstract cubes. Each city becomes an engraved town vignette chosen by **era (6) × size tier (4)**: hamlet (population 1–3), town (4–7), city (8–12), metropolis (13+). That's 24 vignettes in the atlas ink style.
- Crossing a size tier redraws the city with a small flourish; walls, castles and star forts draw as its wall ring.
- Regional styles per people (Mediterranean, Northern European, East Asian, Steppe) come second, as an art-only step.
- Three art directions are being explored now (Luna task `city-art-directions`: painted miniature, ink-and-wash town, illuminated emblem). The user picks the look before the full set of 24 is made.

## 4. Research model

### The gate: finish the era (decided by the user, 2026-10-10)
**Rule:** each era has **5 techs**. You can research only techs of your current era, and you enter the next era the turn you finish the last of them. That's 30 techs in all. A typical winner finishes 25 by about 80% of the game, which puts it in Modern, and the last 5 by the end.

**The honest trade-off.**
- *Gains:* eras are real, because everyone in an era shares its whole toolset. Nobody beelines gunpowder on a thin economy. No tech goes unresearched. The tree is simple to read.
- *Costs:* research is no longer about *what* you get, only *when*. Everyone gets everything in an era, so the choice is the order of five techs. A trailing empire can't specialise around a gap; it has to finish the era (diffusion helps, below).

### Keeping research strategic under the gate
1. **Order is timing, and the swing is big.** In a 35-turn era the first tech lands about 7 turns in and the fifth about 35 in. Knights first or the University first moves a knight window or a Science lead by up to 28 turns. The research screen forecasts the arrival turn of each item in your queued order (pillar 5).
2. **Each era mixes war and economy techs**, so order answers the user's own playtest question: army first, or economy first?
3. **No prerequisites inside an era**, so any of the five can come first.
4. **Boosts (Civ 6 Eurekas).** Each tech has one boost from play that pays 40% of its cost at once (rounded down). Examples: Masonry from building a Quarry, Chivalry from winning a battle with a Horseman, Banking from owning 3 Markets. Boosts pull the order toward the plan you're already playing.
5. **Faction items hang on specific techs**, so each faction has a different best order. They're designed through the faction method, not here.
6. **No dead techs.** Every tech unlocks an item or rule every faction can use. Where a faction can't use an item, it gets a replacement (the existing "no dead buildings" rule). Rules that grant techs, such as the Athens' broadcast, grant techs of the receiver's current era.

### Catch-up and diffusion
- Keep diffusion: −10% per other empire that knows the tech. **Raise the cap from −30% to −50%** (start). Under the gate this matters more: a trailing empire's whole era is already known by others, so it finishes the era up to twice as fast. The leader never gets it on new techs.
- The discount is a line on the tech card ("−40%: known by 4 empires"). Unit upgrades for Gold stay, so a trailing empire can leap a generation with one big spend.

### Pacing targets (tuned by sim)
- **Calibrate to human games, not only AI games.** All-AI games run longer (median 191 turns at 4 players) than the user's recent wins (T84 to T163). Era entry turns are fractions of game length, and they're checked on human-length games too: the human-seat benchmark and playtests.
- The median winner enters each era at the turn in section 3, ±5% of game length. Every era lasts at least 30 turns.
- At least 60% of winners reach Modern. The median non-winner ends at most one era behind.
- Reach: the user's saves end with 5 to 14 techs, and the gate needs about 25 by 80% of the game. Sim winners already reach era 4 at T166, so costs come from a sweep starting near the 40–50% setting of the existing `tech_cost_percent` knob, together with the diffusion cap.

### Tech list (30 techs, real history)
Most items are existing content under historical names. New items are **bold**. Spearman, Archer, **Horseman**, Scout and Settler are available from the start, so the triangle exists from turn 1.

| Era | Tech | Unlocks |
|---|---|---|
| Ancient | Irrigation | +1 Food in every city |
| | Bronze Working | Workshop; Copper visible |
| | Masonry | Walls |
| | Currency | Market |
| | Writing | Library (today's Laboratory) |
| Classical | Mathematics | **Catapult** (siege) |
| | Engineering | Roadworks (roads) |
| | Iron Working | Forge (today's Machine Shop); Mine/Quarry upgrades |
| | Code of Laws | Tiles 25% cheaper; new cities claim a ring-2 tile |
| | Philosophy | Great Library (Great Work; requires a Library) |
| Medieval | Feudalism | **Pikeman**; **Castle** (Walls tier 2) |
| | Machinery | **Crossbowman** |
| | Chivalry | **Knight** |
| | Education | University (today's College) |
| | Crop Rotation | Mill (today's Hydroponics); Farm/Pasture upgrades |
| Renaissance | Gunpowder | **Musketman** |
| | Metallurgy | **Bombard** (siege tier 2) |
| | Military Engineering | **Star Fort** (Walls tier 3); upgrades cost half |
| | Banking | Bank (today's Clearinghouse); Grand Bazaar (Great Work) |
| | Printing Press | +1 Science per Library-family building |
| Industrial | Rifling | **Rifleman** |
| | Military Science | **Cavalry** |
| | Ballistics | **Field Cannon** |
| | Industrialization | Factory (today's Autoworks); Royal Arsenal (Great Work) |
| | Railways | **Your units starting on a road get +2 moves, not +1** |
| Modern | Replaceable Parts | **Infantry** |
| | Combustion | **Tank** |
| | Steel | **Artillery** |
| | Flight | **Fighter, Bomber** (air layer) |
| | Electricity | Power Plant (today's Reactor); Research Lab (today's Inference Core) |

Cut or folded: Salvage Protocols, Nutrient Synthesis, Precision Industry, Planetary Accord, Legacy Integration (number-only techs); Diplomatic Protocols and Strategic Logistics (folded in above); Arcology. Faction items keep their gates under new names.

## 5. Units across eras: three classes

**The triangle (user, 2026-10-10):** the melee line beats mounted, ranged beats melee, mounted beats ranged. Siege supports. Aircraft are a new layer in the last era, not a fourth land class.

| Era | Melee line | Ranged | Mounted | Siege / air |
|---|---|---|---|---|
| Ancient | Spearman | Archer | Horseman | — |
| Classical | (Spearman) | (Archer) | (Horseman) | Catapult |
| Medieval | Pikeman | Crossbowman | Knight | (Catapult) |
| Renaissance | Musketman | (Crossbowman) | (Knight) | Bombard |
| Industrial | Rifleman | Field Cannon | Cavalry | (Bombard) |
| Modern | Infantry | Artillery | Tank | Fighter, Bomber |

Brackets mean the previous unit carries on. Not every class upgrades every era, so who leads in which class keeps shifting.

**The numbers that make it hold (start values for the scenario grid):**
- Each era tier adds +10 strength (today: Soldier 20, Trooper 32, Assault Trooper 44, so a similar step).
- The class bonus is +10 against the class it beats. Under the current formula +10 means about 45 damage dealt against 20 taken, roughly a 2.2× exchange.
- So within an era the triangle decides fights. A unit one era ahead fights its counter-class evenly, and a two-era gap beats the triangle: tanks beat pikemen.
- Mounted units get 4 moves and the +10 also applies against ranged fire. They are the answer to kiting: they catch a ranged unit that steps back two hexes.
- Ranged gets no class bonus; its range is the bonus. Siege keeps its bonus against cities.
- The cavalry experiment (`astra/cavalry-experiment-v2`) and the combat scenario grid (`astra/combat-scenarios-v2`) set the final values. The red flag is one composition winning everywhere.

**The air layer (Modern only), checked against the rules:**
- Aircraft are based in a city and never stand on map tiles (up to 2 per city, start). They never capture; if the base city falls, the plane is lost.
- A Bomber gets one order a turn: strike a tile within 5 hexes, resolved in the existing ranged phase with no retaliation, or rebase to another of your cities.
- A Fighter intercepts the first enemy strike each turn within 4 hexes of its city, in unit-id order (deterministic). That strike doesn't land, and the two planes trade damage. Existing orders, ranged phase and integer damage; one order per unit.

**Turn size:** new generations replace units through upgrades rather than adding more. Air adds a handful of units in the last 20% of the game. Railways cut movement clicks.

## 6. Victory points

### New VP sources
| Source | VP | Kind |
|---|---:|---|
| Faction goal | 2 | Unchanged |
| Great Work | 2 | Unchanged; three historical routes |
| Era objectives, Classical to Modern | 1 per era, max 5 | Permanent, public, anyone can score |
| Crowns: Strongest Army, Largest Economy, Most Advanced | 1 each, max 3 | Contested, live, cost upkeep |

- Permanent ceiling per empire: 9. Maximum total: 12.
- Win: 10 VP held for 10 turns (5 at Quick), unchanged. Because the permanent ceiling is 9, a solo winner always needs at least one crown.
- Team target: 9 × teammates + 1, so 10 solo and **19 for the two-human team** (today 17). Check by sim.

### Era objectives (TI4 style)
- Each era shows 3 public objectives, one from each family: Build, Expand, Fight.
- **Meeting any one of them earns that era's 1 VP.** The choice is which one to go for.
- An era's objectives are drawn from a pool of about 8 at game start, so games differ. Next era's three are always visible on the VP panel.
- You can score an era once you've entered it, and an unscored era stays open forever. A trailing empire can still collect its Classical point at T150.
- Examples (start):
  - Classical: own 4 cities, or reach 20 population, or capture a city.
  - Medieval: walls or a castle in 4 cities, or 3 Universities, or capture a city from an empire ahead of you on VP.
  - Renaissance: Star Forts in 3 cities, or 3 Banks, or destroy 6 units of an empire in a later era.
  - Industrial: 3 Factories, or 10 cities, or take a city from the VP leader.
  - Modern: hold a Power Plant and a Research Lab, or 60 population, or end the VP leader's hold.
- The old milestones become Classical objectives. Six Technologies is replaced by eras.

### Crowns (three titles instead of five)
- **Drop Most Populous and Most Territory.** They are one title counted twice (r = 0.80), and expansion still pays through every other source.
- **Keep Strongest Army and Largest Economy.** They correlate with each other at only r = 0.33.
- **Most Advanced becomes the furthest era, ties broken by techs known; the incumbent keeps ties.** Entering an era first takes a crown, so getting ahead is a big deal.

**Why one empire can't own everything:** 3 crowns instead of 5, and the least correlated ones; era objectives that a size lead can't take from anyone; and each extra crown costs more Gold (section 7).

### The path back for fifth place
1. **Open era slots.** Any unscored era is still worth 1 VP, and its objectives don't expire.
2. **A cheap crown.** Your first crown costs only your era number in Gold. One push with a fresh generation (a Cavalry window, a Tank window) takes Strongest Army.
3. **Fight objectives aimed at the leader:** capture a city from the VP leader, or end their hold.
4. **Diffusion up to −50%,** so the laggard catches up eras fast and can enter the next era's objectives.

## 7. Imperial overreach (decided; the numbers are new)

**Rule:** each crown you hold costs Gold every turn. Your first crown costs your era number (Ancient 1 to Modern 6). Your second costs twice that, your third three times. It's paid with upkeep, shown as one "Overreach" line in the Gold breakdown, and an unpaid deficit uses the existing bankruptcy rules.

| Your era | 1 crown | 2 crowns | 3 crowns |
|---|---:|---:|---:|
| Medieval (3) | 3 | 9 | 18 |
| Renaissance (4) | 4 | 12 | 24 |
| Industrial (5) | 5 | 15 | 30 |
| Modern (6) | 6 | 18 | 36 |

Gold per turn, not speed-scaled (per-turn yields aren't). Leaders in long sim games earn about 40 to 55 Gold a turn: three Industrial crowns take most of that, one is a rounding error.

**Interaction with the new VP:**
- Only crowns cost overreach. Faction goals already carry their own cost by design, and era objectives are permanent.
- So overreach pushes a leader to win on permanent points, which every rival can also earn, and to hold just enough crowns.
- It's self-scaling: being an era ahead costs more per crown. A laggard holding one crown pays little.

**A well-playing leader stays on top.** Worked example, Industrial era:
- The leader has 8 permanent VP plus Army and Economy: 10 VP for 15 Gold a turn out of about 50. It holds easily, unless a rival takes a crown.
- Taking Most Advanced as well, for insurance, costs 30 a turn. That's a real choice, not free.
- Entering Modern first earns a 9th permanent point. From then on the leader needs one crown, for 6 Gold a turn.

**Tuning knobs:** Gold per crown per era (start 1); escalation for the 2nd and 3rd crowns (start ×2, ×3); which era counts (your own, start, or the world's furthest); a surcharge during your own hold (start off); the number of crowns (3).

## 8. The late game: turn 170 under this design

**What the player does at T170.**
- The leader is entering Modern. The first Tank and Bomber banners go out, and the Modern objectives are live.
- Research is a timing call: Combustion and Flight first for the decisive push, or Electricity first for the economy and the Modern objective.
- If you're leading, you're paying 15 to 30 Gold a turn in overreach. Every turn you ask which crown is worth keeping.
- If you're fifth (the user's 7 VP case), three concrete plans are open, not "press return and wait": the Industrial slot if it's unscored ("take a city from the VP leader"); a cheap crown with a fresh Cavalry or Field Cannon army against the leader's older units; and discounted techs to reach Modern.

**What the AI must learn:**
- **Research order under the gate:** sequence each era's five techs by the committed strategy bundle and the available boosts.
- **Era objectives:** pick the reachable one per era and value open older slots.
- **Crown economics:** hold a crown only when its VP is worth the overreach. Drop one deliberately when it isn't.
- **Era windows:** attack when one era ahead in a class, and fear a neighbour who is. This is the main fix for "at war but not attacking".
- **Composition:** scout enemy armies and counter with the triangle. Hunt ranged units with mounted ones.
- **Breaking holds:** value the crown or city that drops the holder below target. The Contain plan already prices VP denial.
- **Air:** strike targeting reuses ranged targeting. Fighters guard cities that are threatened.

**Where current AI work fits:**
- The search AI rolls plans forward on the real engine, so once the base policy can play a mechanic, search prices it.
- Each increment ships with base-policy support and telemetry: objectives scored per era, crown flips, overreach paid, era entry turns, units per class.
- The combat scenario grid validates the triangle before campaigns do.
- The hill-climb ledger and the 1v1 human-playbook proxy remain the AI-quality bar. Deep offline search can then find good routes through each era per faction, to distil into the live AI.

## 9. Competitiveness tuning plan

**Metrics** (per lobby size, Standard and Quick):

| Metric | Today 4p / 8p | Start target |
|---|---|---|
| P(T50 leader wins) | 61% / 51% | ≤ 45% / ≤ 35% |
| P(T100 leader wins) | 73% / 65% | ≤ 55% / ≤ 45% |
| VP lead changes per game | to measure | ≥ 3 |
| Settled turn (last leader change) | T95 / T181 | ≥ 75% of game length |

**Method:**
1. Sweep three knobs on cloud cohorts of 256 games per lobby (about $0.16 per 512 games): overreach Gold per era (0.5, 1, 2), escalation (flat, ×2/×3) and the diffusion cap (−30%, −50%). That's 18 settings for a few dollars.
2. Plot each setting on the four metrics. Pick three: **Gentle, Medium, Strong**.
3. The user plays two or three of them and picks by feel. Sims can't say how much lead-swapping is fun.
4. Sims here check mechanics and lead dynamics, not faction balance, which still waits for a competent AI. Report on held-out seeds.

## 10. Rollout as playable increments

| Step | Delivers (each playable) | Size |
|---|---|---|
| 1. Skateboard | 3 crowns, overreach Gold, era objectives keyed to today's 4 eras, AI valuation, VP panel | M |
| 2. Scooter | Six historical eras (30 techs, mostly renamed items plus Horseman and Catapult), the finish-the-era gate, era banners, diffusion cap, pacing by sim; the era-up takeover with key art and music | M–L |
| Art lane, from step 1 | City art by era × size (24), in the style the user picks from `city-art-directions`; must land with the scooter so the era-up redraw works; regional styles later | L |
| 3. Bicycle | Every unit in section 5 with its era look, era step and class bonus, castles and star forts; scenario grid first | L |
| 4. Motorcycle | Boosts, Railways, Fighter and Bomber | M–L |
| 5. Car | Hold-breaking and era-window AI, Gentle/Medium/Strong playtests, era art and music | L |

The skateboard is the smallest change that helps turn 170: it removes title clustering, gives a 7-VP player concrete targets and puts a price on the leader. Measure the section 9 metrics against today. Every step ships with AI support and goes to the user as a playtest build.

## 11. Open questions for the user

1. **Six eras, ending around 1940** (tanks and planes; no nuclear weapons or computers)? *Recommended: yes.*
2. **City styles:** one shared engraved style per era first, then regional styles for each people (Mediterranean, Northern European, East Asian, Steppe)? *Recommended: yes, regional second.*
3. **Three crowns** (Army, Economy, Most Advanced as furthest era), dropping Most Populous and Most Territory? *Recommended: yes.*
4. **Overreach as escalating Gold** (1st crown = your era in Gold a turn, 2nd twice that, 3rd three times)? *Recommended: yes, starting from the Medium setting.*
5. **Two-human team target 19** (9 per teammate + 1), so a team always needs a crown? *Recommended: yes, confirmed by sim.*
6. **Navy stays out** for this overhaul (the map plays as land; ships are a separate project)? *Recommended: yes.*

## 12. Rejected options

| Option | Why not |
|---|---|
| Civ 7 ages (global reset, civ swap) | The user rejected hard cutovers. It's Civ 7's most criticised feature (48% Steam recommendation), and Firaxis is walking it back. |
| A 4-of-6 fractional gate (the lead's first draft) | The user chose to finish every era, so every empire passes through all of its content and no tech is wasted. The cost, no leapfrogging, is handled by timing, boosts and diffusion. |
| Capstone techs | A required tech is a no-brainer, and it still leaves techs unresearched. |
| Simply cheaper research | The best tested setting missed the targets in every cohort, and it adds no tanks, only the same era faster. |
| Stellaris size tax on research | It punishes expansion, the core loop, and makes tall viable (pillar 10). |
| Golden and Dark ages, debt | The user chose overreach as the rise-and-fall mechanic. Debt is a whole new system. |
| Random crises (Civ 7, Stellaris) | Random punishment feels like cheating; overreach is a visible, chosen cost. |
| Secret objectives (TI4) | They hide the race (pillar 4). Ours are public. |
| Exclusive tech forks | They leave techs unresearched, against the user's gate. Faction items give the variety instead. |
| A world-era clock set by the leader | It lets the leader end eras for everyone. |
| Per-capita titles | Hard to read and weakens the reward for expanding. |
| A fourth land class (anti-cavalry, swordsmen) | The user wants exactly three. |
| A warmonger penalty that grows each era (Civ 6) | The AI already attacks too little; holding should cost, not fighting. |

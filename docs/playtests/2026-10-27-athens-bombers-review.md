# Coach's review: the 8.9.13 Athens bomber win (2026-10-27)

Save: `~/LegatumRuns/bug-saves/athens-win-bombers-20261027.legatum`. Engine `a58e80856` (tag `playtest-8.9.13`), 5 empires, every AI Hard. Replayed at the tag with 0 hash mismatches in 177 turns. The AI plan trace (`p881_trace`) matches all 696 recorded AI order lists. The what-ifs are 59 branches of 32 runs, run locally (runner and data in `~/LegatumRuns/coach-tools/coach-8913-athens/`).

VP race and army size (archived outside the repository)

## Coach's verdict

You played the Athens plan the design intends: a science lead, Walls everywhere, and a late weapon. The AIs never made you pay for its weakness. You fielded 3 or fewer military units for 177 turns. At the start of your hold (T168), 14 of your 16 cities had no unit inside. Sparta, next door with the strongest army in the game (987 strength at T150 against your 492, 2,391 at the end), was at War with you for 55 turns. It never attacked one of your cities. In the whole game the AIs hit your cities once and killed one of your units.

The win itself came from one Bomber and two Infantry. They took six cities in 18 turns, two of them with Great Works. A Bomber strike did 204 to 261 damage to cities with 200 HP, because aircraft ignore Walls. No AI could answer: none had Flight until your own Share the Learning gave it to them on T169.

The game was over by T150. Hard playing your seat from T150 wins 31 of 32. "It was easy" is fair. It is an AI failure first: Sparta's army never left its stale campaign. The air rule is second: one Bomber wipes out a city's HP every turn.

## 1. Result and margin

- **Result.** You won on T177 by VP: 13 to Byzantium's 5 and Sparta's 4. You held 10 or more VP for 10 turns from T168.
- **Who led.** Byzantium led from T131 to T158 (6 to your 3 at T150). You went from 5 VP to 13 between T159 and T177: two capture objectives, two captured Great Works (+4), Share the Learning (+2), a territory objective and Largest Economy.
- **Pillar 11.** The outcome was clear by about T165 (8 VP to 5), but the position had been decided earlier. Hard in your seat wins 69% from T100 and 97% from T150.
- **Play time.** 27 active minutes in one session, 0.1 to 0.3 minutes a turn in every era.
  - You gave about 350 orders in the whole game, 62 of them automation.
  - Each AI gave 1,300 to 2,000.
  - This was a light-touch game.

## 2. Turning points

| Turn | What happened | Swing |
|---|---|---|
| T13–T17 | Sparta planned a campaign on Athens. Its capture forecast rose from 0% to 22% while your Walls went up (finished T14). It declared on T16 with 4 Spearmen against your 1, then switched the campaign to the Celts on T17 ("forced expedition", value 0). | No attack; Peace T42 |
| T101–T104 | Sparta declared again. You had 7 cities and 2 military units. Sparta's own view showed it 0 of your military units. Athens Colony 7 was founded that turn: pop 1, no Walls, 5 tiles from Sellasia. Sparta aimed at Phaleron with another value-0 expedition, then on T104 switched to Mongol Shangdu (it took an "abandons a declared war" penalty of −300 to do it). | 29 turns at War, 0 Spartan attacks on you; Peace T130 |
| T133 | The Mongols declared on you ("Contain" the VP race, value +129) with 6–8 old units. They ran one 2-turn campaign (Eleusis, T138). On T146 they opened a second war, on Byzantium. | Gave you the war you needed |
| T147–T148 | You were the first empire to research Flight. No AI was even in the Modern era yet: Byzantium entered it T156, Sparta T163, the Mongols never. You bought a Bomber for 920 Gold. | The air weapon arrives uncontested |
| T150–T167 | Bomber plus two Infantry took Karakorum, Almaliq, Bukhara, Khanbaliq, Sarai (Great Work, +2) and Shangdu. The Mongols were eliminated on T167. | 3 → 8 VP |
| T168–T171 | Share the Learning (+2) took you to 10 VP and started your hold. On T171 you declared on the Celts and took Bibracte (Great Work, +2) the same turn. Byzantium declared on T166 and made Peace on T172 without one attack. | Hold to 13 VP |

## 3. Questions answered

### Q1. Is the strategy the intended Athens identity, fair and counterable, or a dominant line?

**The identity is right (pillar 8).**
- **Science.** Your Science was 77 to 120 a turn against at most 58 for any AI. You were first to 27 of the 31 techs anyone learned.
- **The weakness.** It was real and you leaned into it: an army of 1 to 3 units all game.
- **The broadcast.** Share the Learning handed every rival 3 techs on T169, including Flight for Byzantium and Sparta.

The faction sheet names the counter: "press before its Science lead compounds". It was available to Sparta the whole game and was never used.

**The air stage is close to a dominant line for whoever reaches Flight first.**
- **The damage.** `combat::wall_damage_percent` returns 100% for aircraft, so a Bomber's strike (114 strength, plus 36 against city HP) deleted a whole 200-HP city at strengths 39–67: 204 to 261 damage.
- **The capture.** Any melee unit then walks in. You took a city every 2–3 turns with one plane.
- **The counter.** There is no anti-air ground unit. The Fighter (intercepts within 4 tiles of its base) needs the same Flight tech. Your victim, the Mongols, never reached the Modern era, so for them the counter didn't exist.

The ground counters did exist, and none was used:
- killing your only two capture units (two Infantry);
- retaking Karakorum, your Bomber's base from T156 (capturing a base destroys its aircraft);
- the Spartan army attacking your empty home cities.

**Verdict.** A fair plan with a real counter in the rules (punish the tiny army). The AIs didn't use it. One rule makes the air step decisive for an empire one era ahead: a Bomber zeroes a walled city in one strike.

**Design question for you:** should Walls or Castles reduce air damage to city HP, or should one strike be unable to take a city from full HP to 0?

### Q2. Why Sparta failed next door

Sparta was at War with you for 55 of 177 turns (T16–T42, T101–T130) and never attacked a city of yours.

**First war (T16–T42).**
- It declared two turns after your Walls finished.
- It had 4 Spearmen (strength 65) against your 1 Spearman and a walled city.
- On T17 it moved its campaign to the Celts' Bibracte. Its only contact was one Hoplite shot by your city (T17), plus your Scout killed on T35.

**Second war (T101–T130).**
- Sparta was a Renaissance-era empire with 9–10 units (at T100: 6 Ancient, 1 Classical, 2 Medieval). Your units were Renaissance: a Musketman and an Arquebusier.
- The window was T101–T104. Colony 7 had just been founded: pop 1, no Walls, no unit, 5 tiles from Sellasia. Your home cities were walled but empty, and Sparta's view showed 0 Athenian military.
- It switched to a Mongol city on T104. From T112 it sat on a campaign against Byzantine Pergamon for 65 turns: paused, with its next plan review set for T182. It proposed Peace to you 4 turns into the war (T105) and again on T129, and you accepted the second.

**After T130.**
- Sparta named you its top threat on T160 (score 72, rising to 556 by T177).
- It never declared again.
- In the Scenario Lab cut from T150 (below), Hard Sparta declares in only 11 of 50 runs, median T167, and takes Athens in 0 of 50.

**Did it have a winning window?** Yes, in the sense that matters:
- In every one of my branches where your finish slipped past about T180, Sparta won 32 of 32. It took 4 to 7 of your empty cities, starting between T187 and T208.
- Its army could beat you; it acted about 20 turns too late.
- Forced to stay at War with you from T133, T140 or T150 with no Peace, it still hit your cities only 0.4 times on average before T177.

### Q3. The air stage

- **No AI built or ordered a Fighter.** That was 0 orders in the whole game. No AI completed any aircraft.
- **Flight.** No AI researched Flight on its own. Byzantium was in the Modern era from T156 and chose Railways, Replaceable Parts and Combustion. Byzantium and Sparta got Flight only from your broadcast on T169. They then queued Bombers (Sparta 4 cities from T170, Byzantium 3 from T172), and none finished by T177.
- **How the Bomber took VP cities.** You bought it on T148 at Eleusis and rebased it to Laurion (T149), Karakorum (T156) and Bibracte (T175). It made 22 strikes (7 on cities, 15 on units) and was never damaged. In each capture the Bomber took the city's HP to 0 and an Infantry walked in the same turn or the next: Karakorum T152–153, Almaliq T159, Bukhara T160, Khanbaliq T163, Sarai T165, Shangdu T167. Bibracte fell on T171 to your declaration and capture in the same turn.
- **Response time.** From your first Bomber (T148) to the first AI air response (a Spartan Bomber queued on T170) was 22 turns. It was a Bomber, not a Fighter, and it was made possible by your gift.

### Q4. Army quality: did 8.9.13's modernization show?

8.9.13 has only the valuation gene on (`modernization_value` 100); the upgrade and return genes are off. The armies stayed obsolete.

| AI | Tech era at T160 | Military units at T160, by unit era | Units added after T100, mean eras behind | Upgrades all game |
|---|---|---|---|---|
| Byzantium | 6 (Modern) | 12 Ancient, 2 Renaissance | 35, **4.3 behind** (mostly Spearmen hired from Celtic Mercenary Companies) | 2 (122 Gold) |
| Sparta | 5 | 3 Ancient, 2 Classical, 1 Medieval, 7 Renaissance, 1 Industrial | 10, 3.1 behind (6 Archers or Spearmen) | 32 (1,984 Gold) |
| Mongols | 5 | 1 Ancient, 1 Classical, 2 Medieval, 2 Renaissance, 1 Industrial | 10, 1.9 behind | 14 (710 Gold) |
| Celts | 4 | 8 Ancient | 10, 3.2 behind | 0 |

Measured on his position (32 runs each):
- **Hard plays Athens** (it adapts, so it is the honest measure). Every AI with `modernization_upgrade` and `modernization_return` at 100:
  - from T150, Athens wins 31 → 26 of 32;
  - from T140, 22 → 18 of 32, with Sparta's wins 9 → 13.
- **The active part** is the upgrade gene. The return gene alone changes nothing.
- **With your recorded orders** the same genes drop you from 32 to 0 of 32. That number is fragile, not a real effect size. The Mongol defence of Sarai changes, your T165 capture misses, you never reach 10, and Sparta wins later. Neutral perturbations keep 31–32 of 32, but a Celtic plan reset does the same damage. Treat it as an upper bound.

### Q5. Who had the best chance to hurt you, and why it didn't

**Sparta, clearly.**
- **Its edge.** The strongest army from T1, 4 to 6 tiles from your capital, and your cities mostly empty.
- **Plan.** Each war switched away from you within 1–3 turns, to value-0 "forced expeditions". Then a paused campaign on Pergamon held its plan for 65 turns. Clearing that memory at T150 (with a forced war and no Peace) still gave 0 captures before T177.
- **Money.** It had 20–150 Gold all game, and its upgrades took all of it.
- **Diplomacy.** It offered Peace 4 turns into its second war and signed on T130.

**The Mongols** had the right instinct ("contain the leader", T133) but no force: 6–8 old units and 12–65 Gold. They then opened a second front on Byzantium (T146).

**Byzantium**, the VP leader until T158, declared on you at T166 while "busy elsewhere". It made Peace six turns later without an attack. Its army was 11–12 Ancient Spearmen.

**Bands or decisions.**
- **Decadence.** You paid 15% on average after T100 (23% at the end). Byzantium paid 16%, Sparta 3%.
- **Bands off from T133** speeds up Hard Athens (22 → 28 of 32): the bands slowed the leader.
- **Diffusion.** The AIs learned 28–29 of their 30 techs after you knew them. That kept their tech counts close (30 to your 31) but not their timing: they reached the Modern era 15–22 turns after you.
- **Verdict.** What little resistance there was came from the bands. AI decisions contributed nothing measurable: no city taken from you, one city attack all game.

### Q6. New war fixtures (specified, not added)

The states are extracted at the release engine (0 hash mismatches) in `~/LegatumRuns/coach-tools/coach-8913-athens/states/`. I did not add them to the suite. Each needs:
- a `war_build.rs` recipe;
- recorded orders for the Athens seat (`extract-war-states.sh` with `0:first-last`);
- embedding;
- predictions;
- 64 runs with the planner both off and on, on `claude/ai-operation-planner`.

That isn't quick. Predictions, written now and before any run:

| Id | Seat | Start | Goal and deadline | His game's AI result | Prediction (plain Hard / planner on) |
|---|---|---|---|---|---|
| `war-sparta-next-door-t101` | Sparta (2), Hard; Athens replays his recorded orders T101–T120 | T101, the turn Sparta declares: 6 cities, 9–10 units; Athens 7 cities, 2 military units, Colony 7 pop 1 unwalled 5 tiles from Sellasia | Capture any Athenian city by T120 (20 turns) | 0 captures; the campaign switched to the Mongols on T104; Peace T130 | 5–15% / 20–40% |
| `war-mongol-bombers-t150` | Mongols (4), Hard defender; Athens replays T150–T167 | T150: 6 cities, 9 units (Medieval–Renaissance), 17 Gold, at War with Athens since T133 | Keep at least 4 of the 6 cities through T167 (18 turns) | All 6 lost (T153–T167), eliminated | 0–5% / 5–15%; 10–30% with `modernization_upgrade` |
| `war-break-the-hold-t168` | Byzantium (1), Hard, at War with Athens since T166; Athens replays T168–T177 | T168, the hold's first turn. Athens holds Sarai (Great Work, 60 HP, no unit), Bukhara (120 HP, no unit) and Shangdu (20 HP, one Infantry) | Capture any Athenian city by T177 (break the hold) | 0 attacks; Peace T172 | 0–10% / 10–25% |

A Sparta twin of the third case (at Peace at T168, so it must declare) would test the leader read rather than war execution.

### Q7. The single change most likely to make this game hard for you

**Make the strongest neighbour's war on the leader real, and about 20 turns sooner.**
- **What the change is.** When Sparta is at War with the public VP leader, or can declare, and that leader's nearby cities stand visibly empty, its plan should commit to the softest reachable one. The current behaviour has three faults:
  - it drifts to a value-0 expedition elsewhere (T17, T104);
  - it keeps a paused campaign on Pergamon for 65 turns;
  - it offers Peace 4 turns into its own war.
- **The evidence.**
  - Sparta was at War with you for 55 turns and made 0 attacks on your cities.
  - Forced at War from T133–T150 with no Peace, it still made 0.4.
  - In every branch where your finish slipped past T180, Sparta beat you 32 of 32 with captures from T187 on.
  - The army was enough; the decision and the commitment were missing.
- **Where it lives.** `crates/ai/src/plan.rs`: `campaign_candidates`, `contain_candidates`, the review clock, and `plan_abort_reason` for paused campaigns. Also the T101-style forced expedition in `plan.rs`, and `diplomacy.rs::wants_peace` early in its own war. The W1 operation planner (commit/gather/withdraw) is the natural place.
- **How to measure.** On the new fixtures `war-sparta-next-door-t101` and `war-break-the-hold-t168` with `operation_planner` off and on, then the frozen human-seat gate.

**Second: `modernization_upgrade` on.** This is the only lever I could measure in the adaptive seat: −12 to −16 pp for Athens from T140–T150, and Sparta's wins up from 9 to 13 of 32.

**Third: the air stage.**
- Value Flight once a rival is in the Modern era and bombing within reach. Byzantium sat 13 Modern turns without it.
- Turn on `air_defense` once it passes its gate.
- Defend cities that hold a Great Work by their VP. Sarai and Bibracte were the two that decided your hold.

Rule question for you: air damage against walled cities (Q1).

## Scenario for you

**`sparta-next-door`** (Scenario Lab, T150, you play Sparta): "Athens has nine cities and three military units. Can Sparta take its capital before its Bomber decides the game?" Take Athens within 25 turns. The Hard AI baseline is 0 of 50 (it declares in 11 runs, median T167).

This is the one to play first. How a human Sparta breaks an empty, walled Athens is exactly what the AI needs to learn. If you find it easy, that confirms Q2 and Q7.

No new Scenario Lab runs or feedback since 2026-10-22.

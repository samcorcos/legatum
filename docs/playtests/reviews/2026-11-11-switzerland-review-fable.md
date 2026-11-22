# Coach review (independent QA pass): Switzerland, "Dev after 8.9.29" (6f594cf), 2026-11-11

Save: `~/LegatumRuns/bug-saves/swiss-win-2026-11-11.legatum` (copy; the live save was not touched). Engine `playtest-8.9.30` = `6f594cfb272e`, rules 25. Replay: 139 turns, 0 hash mismatches, 0 of 626 AI seat-turns differ. Working files: `~/LegatumRuns/coach-tools/coach-8930-swiss-fable/`.

This review was written without reading the other review of the same game. It changes no code, skill or playbook; proposed playbook changes are a section at the end.

## Coach's verdict

He won on T139 by holding 11 of 10 VP for ten turns (hold began T130). The game was decided between T51 and T67, and by nothing after it. He declared on the Mongols, the map's strongest AI, while half their army stood ten tiles away guarding a population-2 Aztec capture, and he did not stop after the first city: four cities and the capital in 16 turns with seven units, one unit lost. Hard put in his seat agrees with the declaration (it wants it at T48, three turns before him) and takes the first city, then **plans Peace with the Mongols after each capture**. That one difference is the game: Hard in his seat wins 15% from T54 (Shangdu already taken), 73% from T60, 100% from T70.

After T70 four AIs were at War with him for 70 turns and it cost him four military units. He ran 4 to 7 military units for 12 to 13 cities, bought one item all game, and banked 5,151 Gold by T130. The AIs' one "break the hold" campaign went for Lucerne, the only one of his cities whose capture could not break the Reserve; on that same turn a capture of any other city would have reset his hold, and Bern stood walled but empty with four Aztec units 2 to 5 tiles away.

It was a fair win: nothing he did used a rules hole, and every piece is learnable from information the AIs had. Pillar 11 failed: from T70 the winner was known (Hard in his seat 48 of 48 from every later branch).

## 1. Result and margin

| | |
| --- | --- |
| Result | Switzerland wins T139, "held 11 of 10 VP for 10 turns" (hold started T130) |
| Final VP | Switzerland 11, India 4, Celts 3, Aztecs 2, Germany 1, Mongols 1 (eliminated T70) |
| His 11 VP | 3 crowns, Reserve 2 (T130), Pyramids 2 (T63, Biel), Khanbaliq capital 1 (T67), era objectives 3 (T59 capture 2 cities, T91 Mills, T123 Universities) |
| VP race | 2 at T54, 3 at T60, 5 at T64, 7 at T70, 7 at T100, 9 at T123, 11 at T130. No rival passed 4 |
| Outcome clear | T70 (Hard in his seat 48/48 from T70 on). Pillar 11 breach by 30 turns |
| Play time | 34 minutes, 139 turns, one session (about 15 seconds a turn) |

Hard in his seat from his recorded positions, 48 runs each to T240 (current master playing his seat, tag rivals; `takeover-gap/curve`):

| Branch turn | 2 | 20 | 30 | 40 | 50 | 54 | 60 | 66 | 70 to 130 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Seat wins of 48 | 3 | 3 | 0 | 4 | 8 | 7 | 35 | 45 | 48 |

Who wins when he does not, from T30 and T40: the Mongols in 30 and 31 of 48, India in the rest. He removed the empire that wins this map.

## 2. Turning points

1. **T51 to T53, the strike on Shangdu.** At T50 four Soldiers and a Ranger stood 1 to 2 tiles from Shangdu (population 5, no garrison, no Walls, building a Settler) before any declaration. The Mongols held Strongest Army (122 to his 92), but their Soldier and Horseman were 10+ tiles south holding Tenochtitlan (population 2) and their plan read "Peace with the Aztecs" until T60. Captured T53.
2. **T54 to T69, he kept going.** Almaliq T59, Khanbaliq (capital) T67, Bukhara T69; 6 upgrades to Phalangites at T54 to T55 (his main Gold spend of the game) and Roadworks bought in Shangdu at T59. VP 2 to 7. The Mongols first queued Walls at T60. This is where Hard in his seat stops (section 4c).
3. **T63, the Pyramids in Biel (2 VP)** while the army was away. No AI contested the wonder or the city.
4. **T65 to T129, the coalition that did not bite.** Aztecs T65, Celts T66, India T72, Germany T93. In 65 turns they made 20 attacks on his cities, 16 of them on Lucerne (population 1). Ten of his cities were never attacked. His Gold went 225 to 4,969.
5. **T130 to T132, the hold nobody contested where it could be broken.** Reserve margin 151 Gold at T130 and 338 at T131 (table in section 4b). Thirteen coalition units sat around Lucerne. From T133 only the capital could break it; from T138 nothing could.

## 3. The human's edge (measured)

| What | Him | Hard in his seat / the AIs here |
| --- | --- | --- |
| Finish the neighbour | 4 cities and a capital in 16 turns after the first capture | Plan "Peace with the Mongols" at T54 to T56, T60, T64 to T68; campaign on Bukhara valued 6 to 207 against 1,145 to 2,204 for Shangdu; gate "our army is holding a captured city" T54 to T60. From T54: 7/48, Mongols survive with a median 6 VP |
| Gold | One purchase (Roadworks, 245) and about ten upgrades (six of them at T54 to T55). 5,151 at T130 | Wants a unit purchase or an upgrade on 68 of the 70 turns T70 to T139 (a unit purchase on 39 of them). Not buying the T77 Phalangite alone is worth +22.9 points of win chance [+9.6, +36.3] at the T140 horizon |
| Army | 4 to 7 military units for 12 to 13 cities; weakest army in the game at T110 (175) at War with four | Hard's production picks differ toward a military unit 184 times where his were 79 buildings (14 Markets, 12 Walls) and 0 military units in those slots; Hard's continuations carry a median 11 military units at T100 |
| Settlers under war | 6 cities founded T82 to T119 with the coalition at War | Close to Hard: 16 against 12 Settler picks. Not an edge this game |
| Declaring | T24 on the Celts for a Settler (captured the same turn); T51 on the Mongols | Hard would not declare at T24 (his choice is not better at T140: 0 win points, -0.7 VP). Hard wants the Mongol war at T48, T49 and T50. "He declares, Hard waits" is **not** supported here |
| Diplomacy | Refused the Aztec and Celtic Alliance offers against the Mongols (T48, T50), then fought them alone; refused India's Peace at T114 | no measured effect |

His opening was not the edge: Hard from T2 and from his T40 or T50 positions wins 6 to 17%. His position at T50 was about as good as Hard's own.

## 4. Strategic review of every AI

Army sizes all game were tiny: at T50 the rivals had 1 to 4 military units each, at T100 5 to 7, at T139 5 to 10. Economies at T100: his 340, India 191, Celts 142, Germany 102, Aztecs 61.

**Mongols (eliminated T70).** Effective strategy: early conquest of the weak neighbour (Tenochtitlan T47), which worked, then nothing. Opening A-: 5 cities and Strongest Army at T50, the best AI on the map (wins 30/48 from T30 when Hard has his seat). T47 to T60 F: the army stayed on a population-2 capture; no Walls anywhere; Gold 22 to 58 so nothing could be bought; five foreign units massing beside an empty population-5 city for at least two turns did not change the plan; Almaliq was building the Pyramids while Shangdu fell. Walls came at T60, after two cities were gone. T70: it razed its own last city (Tenochtitlan, population 1, HP 0) by order and eliminated itself. Idle-army cost: 2 of 4 units 10+ tiles from the war for its first ten turns.

**Aztecs (2 VP).** Lost their capital at T47, 2 cities to T70, 4 at the end. Asked him for an Alliance against the Mongols (T48), declared on him T65 once he was the runaway: a correct read with no means (economy 61). Two attacks on Khanbaliq (T73 to T74), then 55 turns of nothing; four units stood 2 to 5 tiles from empty Bern at T130 and did not move on it. RelayUplink T88 was its one good decision. Grades: opening D, mid C (right enemy, no force), late D.

**Celts (3 VP).** Effective strategy: harass. Lost a Settler to him at T24, Peace T40. Rejoined T66. Campaign on Khanbaliq T73 to T88: roster 8, arrived 2, attacks 0, ended "walls before arrival". 12 of the coalition's 20 city attacks were Celtic, all on Lucerne; sacked it T131 for 60 Gold and an era objective ("Take 1 city from the VP leader", +1 VP for the Celts), which was real value for the Celts and zero against his hold. Closest thing to an effective rival tactically (31 attacks on his units), no strategy that reached his VP. Grades: opening C, mid C, late C+.

**India (4 VP, second).** Effective strategy: wide then economy, at a distance of 18 tiles from him. 11 cities and the largest AI army by the end (1,363 strength at T138). Joined the coalition T71 to T72. Four attacks on Lucerne T96 to T97; campaign on Lucerne T127 to T132: roster 7, arrived 2, attacks 0. Flipped Contain and Peace six times T113 to T121 and offered him Peace at T114. Hagia Sophia T100. Its one act that touched his hold was by accident of army growth: it held Strongest Army T133 and T135 to T138, which put him at exactly 10 VP; it never knew, and six Indian units stood 2 to 4 tiles from unwalled Aarau at T138. The AI closest to beating him, and it never had a plan to. Grades: opening B, mid C, late D (largest unused army: by upkeep the most wasted force in the game).

**Germany (1 VP).** 8 cities, no VP path. Declared T93 after four refused Alliance rounds with the Celts (T73, T80, T87, T91). Captured Lucerne T107 (73 Gold) and razed it T114; campaign on Lausanne T116 to T132, two attacks. Lost 7 units to him, the worst exchange of the coalition. Grades: opening C, mid D, late D.

**Coherence.** Every AI named him top threat by T55 and all four were at War with him by T93. The read was right and early. The failure is entirely from read to march: the wars were Contain plans with short campaigns on the nearest weak colony.

## 4b. Tactics and battles

- **Exchange ratio.** Mongol war: he lost 1 Phalangite and took 5 cities, killing 5 Mongol units. Coalition war (T65 to T139): he lost 4 military units (Marksman T92, two Assault Troopers at Lucerne T107 to T108, Rifleman T131) and killed 22 (Celts 8, Germany 7, Aztecs 4, India 3).
- **City attacks on him, whole game:** Celts on Lucerne 12, India on Lucerne 4, Aztecs on Khanbaliq 2, Germany on Lausanne 2. Never attacked: Thun, Biel (Pyramids), Aarau, Bern, Shangdu, Almaliq, Bukhara, Zürich, Basel, Geneva.
- **Stragglers.** Celts to Khanbaliq: 2 of 8 arrived. India to Lucerne: 2 of 7 arrived. Same piecemeal pattern as past reviews.
- **Sieges that worked:** Lucerne twice (T107 Germany, T131 Celts), a population-1 colony founded inside the coalition's reach. The AIs did buy siege conversions (13 between T101 and T133, 77 to 160 Gold each), so the spending existed; the target did not matter.
- **The Reserve-break table** (Gold above 5,000 against each city's capture bounty; a bounty larger than the margin drops the goal and resets the hold):

| Turn | Margin | Cities whose capture breaks the Reserve |
| ---: | ---: | --- |
| 130 | 151 | every city except Lucerne (Aarau 245, unwalled; Bern 429, walled, no unit) |
| 131 | 338 | all except Lucerne, Aarau (260), Lausanne (260). Bern 455 still empty; Geneva 390 unwalled and empty |
| 133 | 720 | Thun only (capital, doubled, 1,197) |
| 135 | 1,124 | Thun only (1,267) |
| 138 | 1,747 | none |

   On T133 and T135 to T138 he stood at exactly 10 VP, so removing any one VP (the Pyramids in Biel, a crown) would also have broken the hold.
- **Known incidents (being fixed, not analysed here):** the Bukhara T67 bounce (relief chain rule) and the Lucerne rival-attacker standoff (tie-breaker rule).
- **Cheating-adjacent ideas rejected:** giving the AIs his true treasury or his full city list. Neither is needed; see recommendation 2.

## 4c. Decision diff

Report: `~/LegatumRuns/coach-tools/coach-8930-swiss-fable/diff/swiss-2026-11-11-decision-diff.md` (30 valued candidates, 2,880 paired runs to T140). Event windows: `diff/swiss-2026-11-11-decision-diff-windows.md`. Rival seats: `diff/swiss-2026-11-11-decision-diff-rivals.md`.

Event windows (his army orders against Hard's over five turns, then Hard continues; his win chance at T140):

| Window | Event | His orders minus Hard's |
| --- | --- | ---: |
| T48 to T52 | siege of Shangdu begins | +29.3 points [+20.6, +38.0], +1.3 VP |
| T52 to T56 | siege of Almaliq begins | +8.6 [+1.3, +15.9], +0.8 VP |
| T63 to T67 | Khanbaliq captured | +10.1 [+6.6, +13.6], +1.4 VP |
| T65 to T69 | Bukhara captured | +5.3 [+3.5, +7.1], +0.8 VP |
| T59 to T63 | siege of Khanbaliq begins | -10.3 [-16.2, -4.4] (Hard's moves were better here) |
| T20 to T24 | Celtic Settler taken | -0.4 [-7.1, +6.3] (no effect) |

So even where Hard agrees on the war, his five turns of army orders before and at the declaration are worth 29 points: he walked the whole army next to the city before declaring, and Hard declares first (T48) and marches after.

Top disagreements, classified:

| Turn | Disagreement | Paired value (his minus Hard's) | Class | Where |
| --- | --- | --- | --- | --- |
| 54 to 68 | He continues the Mongol war; Hard plans Peace and gates new campaigns | not a single-turn swap; takeover curve 15% (T54) to 73% (T60) to 100% (T70) | **valuation error** | `plan.rs` campaign candidates after a capture (gate "our army is holding a captured city"), the value of the next city of a broken enemy, `diplomacy.rs::wants_peace` |
| 70 to 139 | He keeps Gold; Hard buys units and upgrades almost every turn | T77 +22.9 win points [+9.6, +36.3], +0.6 VP; T76 and T81 within noise | **valuation error** (Hard Swiss) | `swiss_playbook`, `decisive_buy.rs`, purchase passes; Hard's own trace says `saving_for_goal true` and still lists buys |
| 119 | He refounds Lucerne; Hard walks the Settler on | +35.4 win points [+21.7, +49.1], -1.1 VP | unclear: a horizon effect I could not attribute (win chance up, VP down) | `civilians.rs` site choice |
| 122, 123 | His escort moves; a Settler in Thun where Hard builds a Rifleman | -31.2 and -29.2 win points | probably his slips at this horizon (he lost Strongest Army and fell to exactly 10 VP on five turns of the hold; a Rifleman would have helped), not verified | none |
| 24 | He declares on the Celts for a Settler | 0 win points, -0.7 VP [-1.1, -0.2] | not an edge | none |
| 48 to 50 | Hard declares on the Mongols; he waits to T51 | 0, within noise | agreement in substance | `plan.rs` Campaign value 1,145 to 2,204 |

Closure of recent fixes:

- `runaway_coalition` 55: fired (all four at War, Alliances T71 and T92) and achieved nothing; Alliance offers were still refused 3 and 4 times before acceptance.
- "break the hold" (`vp_hold_break`) and `capture_goal_swing`: **fires on the decisive turn (T131, value 1,785 of 1,946) and aims at the wrong city.** Open, see recommendation 2.
- `swiss_playbook` level 3: Hard Switzerland still spends where he saves. Level 4 (idea #50) is supported.
- `war_reaim`, `hollow_read`, `persist_hollow`: no campaign ever re-aimed at an empty city (Bern, Geneva, Basel, Zürich). Not closed.
- "He declares war; Hard waits" (pattern 7): weakened by this game. "He refuses Peace / Hard seeks it" (pattern 8) and "Hard buys; he keeps Gold" (pattern 14, so far -2.00 and not significant) are strengthened; pattern 14 should be split by faction, since for Switzerland the sign is now clearly positive.

## 4d. His Switzerland playbook, and what Hard Switzerland does

AI-only Swiss win share: 6.9% of seats in the 11-03 census (best faction 20.6%), 7.87% with `swiss_playbook` level 3.

| Part | What he did | Hard Switzerland |
| --- | --- | --- |
| Opening | 4 cities by T37, one from a stolen Celtic Settler; 5 military units at T50 | comparable (T50 branch 17%) |
| War | One war of conquest on the neighbour whose army was away, pressed to elimination: 5 cities, +5 VP (capital, objective, and the crowns the land carried) | declares it, stops after one city. **Missing** |
| Gold | Upgrades for the war army (T54 to T55), then nothing: Markets in every city, 100 Gold a turn by T120 and 190 by T135, Reserve at T130 | buys units and upgrades most turns (`swiss_playbook` does not stop it). **Missing** (idea #50b) |
| Garrison | Walls in most cities (Basel, Geneva and Lausanne still unwalled at T110), 4 to 7 units in total, most cities empty | median 11 units at T100 at triple upkeep. **Missing** (idea #50a) |
| Expansion | 6 Settlers T82 to T119 into the dead Mongol land | does it |
| Wonder | Pyramids T63 during the war | not measured |
| Skipped | Alliances, defensive purchases, a field army after T70 | |

Recommendation: `swiss_playbook` level 4 as a committed line: (a) after a first capture from a neighbour that has lost its field army, keep the campaign until its capital falls; (b) no unit purchases while saving for the Reserve unless a city is forecast to fall; (c) garrison only cities with a rival army in reach. Measure by Swiss AI-only share on both census seeds and Hard in his seat from T54 of this game (now 7/48).

## 5. Was it fair?

Yes. Skill: timing the strike on an overextended neighbour, finishing it, and reading that the coalition would not come. Luck: the Mongols had just spent their army on the Aztecs; that is a readable opportunity, not a gift. One rules question, low impact: the Mongols razed their own last city and eliminated themselves (T70); see design questions.

## 5a. Smarter AI or weaker leader

Not measured with band-off counterfactuals in this pass (stated, not hidden). The direct evidence says the bands were irrelevant to the outcome: Hard in his seat wins 48/48 from every branch from T70, with the bands on, so no band the rivals received made the game contestable. My estimate is that the closeness of this game was about 0% from the bands and about 0% from AI decisions, because it was not close. The two turns he fell to 10 VP came from India's army growth, not from a band.

## 5b. What-ifs

Run: the takeover curve above (Hard in his seat from 15 positions). Not run in this pass: targeted rival changes. The branches worth running, in order, each 48 runs with his recorded orders and with Hard in his seat:

1. T50, Mongols only: army home and Walls first in Shangdu (existing garrison and Walls genes forced). Expect the largest effect; the T50 to T60 swing is 17% to 73%.
2. T130, coalition aims at Bern or Aarau instead of Lucerne (forced campaign target). Tests recommendation 2.
3. T93, all four AIs on one city that is not Lucerne (forced target Basel or Zürich). Tests recommendation 3.

**Rival decisions that would most have cut his win chance** (each rival's own-view alternatives, 40 valued across the five seats, 48 paired runs each, his recorded later orders kept; `diff/swiss-2026-11-11-decision-diff-rivals.md`). Only the Mongol seat had any that mattered:

| Turn | Mongol alternative | His win chance | Mongol win chance |
| --- | --- | ---: | ---: |
| T59 | a forecast attack back toward Shangdu (unit 70 to 8,7) instead of standing | -25.1 points [14.1, 36.1], -1.6 VP | +3.1 |
| T60 | the same a turn later | -11.4 [0.7, 22.1] | +0.6 |
| T47 | **accept the Peace the Aztecs offered** the turn after Tenochtitlan fell | -5.8 [1.1, 10.5], -1.5 VP | **+35.3 [23.6, 47.1], +3.8 VP** |

The Mongols refused that Peace at T47, then asked the Aztecs for it at T50 and T57 and were refused, and asked him at T59 (he refused). Taking the T47 Peace, with the capital already in hand, was worth more to the Mongols than anything else any rival could have done all game. Nothing India, Germany, the Aztecs or the Celts had among their own scored alternatives moved his chance beyond noise (best +2.5 points). Caveat: 95 of 96 paired runs had at least one of his later recorded orders become invalid after the branch, so these arms approximate him.

## 6. Recommendations, ranked by expected effect on how often he wins on Hard

1. **Defend against the visible strike (rivals; new, extends #37 and #43).** Mechanism: when a neighbour's military units outnumber the local garrison within 2 tiles of one of our cities, at Peace or not, the city switches to Walls or a unit, the army comes home, and a far capture of low value is abandoned. Evidence: T50, five Swiss units 1 to 2 tiles from Shangdu, garrison 0, half the Mongol army 10+ tiles away; plan unchanged for 9 turns; his win chance through this window goes 17% to 73% to 100%. Two measured Mongol alternatives: accepting the Aztec Peace at T47 (bank the captured capital and bring the army home) is +35 points of Mongol win chance; one counterattack order at T59 cuts his chance 25 points. So add: accept Peace from an enemy whose capital we have just taken when a third army is on our border (`diplomacy.rs` peace response). Where: `threat.rs::plan_threat_defense`, `hard_defense.rs`, `production.rs::choose_item`. All of it is in the Mongols' own view (the units stood in their borders). Gates: fires at T50 of this tape; a Scenario Lab cut (below); benchmark opponents-only.
2. **Aim a hold-break by the bounty, and fix the fog estimate (rivals; extends #18, #44).** Mechanism: `victory.rs::hoard_denial_value` recovers the Swiss treasury as bounty × (population of the Swiss cities the viewer knows) / share. With a partial view the estimate is far below 5,000 while the goal is publicly live, so every Swiss city reads as a Reserve-breaker and the campaign goes to the easiest one. Use the public fact instead: on the turn the goal scores the treasury is about 5,000, so the margin is about (turns since) × income, and income can be read from the same city's bounty changing turn to turn. Rank targets by bounty minus estimated margin, capital doubled. Second part: "contest the hold" is a term on the passive Contain plan and it shrank as the win approached (India 480 at T132 to 0 at T139); it should open a campaign and grow with turns elapsed. Evidence: T131 Celtic campaign on Lucerne valued 1,946, of which "break the hold" 1,785; Lucerne's bounty was 61 against a margin of 151; his VP did not move. Bern (429) was empty. Gates: fires at T130 to T131 of this tape with Bern or Aarau as the target; scenario below; benchmark opponents-only.
3. **Hard finishes a broken neighbour (seat and every AI; new, relates to pattern 8).** Mechanism: after a capture, value the enemy's next city with the enemy's remaining field army, Walls and Gold, and drop the "holding a captured city" gate when no enemy unit is in reach of the capture; no Peace plan against an enemy whose capital is forecast to fall. Evidence: plan Peace at T54 to T56, T60, T64 to T68 against his four further captures; Hard from T54 7/48 against 35/48 from T60. Where (current master, from the takeover trace across T54 to T65, `takeover-gap/README.md`): Campaign candidates gated by "stale city sighting" (`plan.rs:3801`, 15 of 18 seat-turns), "funded forecast unavailable" (`plan.rs:3806`, 15), "held capture" (`plan.rs:3799`), and `decisive_buy.rs:288` "city in resistance"; the plan then falls to Peace. He had the same sightings and marched anyway: a known city of an enemy with no field army does not go stale. This makes Hard stronger in every seat, including the rival that gets a first capture off him. Gates: Hard in his seat from T54 of this tape; war-scenario suite; benchmark seat-only and opponents-only.
4. **Hard Switzerland stops buying (idea #50b, now measured).** Evidence above (T77 +22.9 points). Caveat: the T140 horizon rewards the exact turn the Reserve lands, so the size is overstated; the sign is not in doubt, since Hard's own trace has `saving_for_goal true`. Hard also wanted an Assault Trooper in Lucerne at T129 and T130 (treasury 4,969 at the start of T130, so the Reserve and the hold start later) and a Soldier at T131 (140 Gold against a margin of 151). Add "a purchase that delays or drops a scoring Reserve costs hold turns" to the buy passes. Where: `swiss_playbook`, `decisive_buy.rs`, purchase passes in `production.rs`.
5. **A war on the leader goes to a city that matters, and stays there (ideas #35, #37a, #52a; evidence only).** 16 of 20 city attacks in 65 turns on a population-1 colony; ten cities never attacked; Bern, Geneva, Basel and Zürich empty for most of the war; he never had to spend. Also force a spend: at T130 to T131 an era unit bought at the Swiss 175% price would have taken his treasury below 5,000 and reset the hold, so a credible threat on one empty city paid off whether he answered it or not.

Lower: India's six Contain and Peace flips in nine turns and its Peace offer to a leader at 7 VP (T114); Alliance refusals that delayed the coalition to T71 and T92 (#53c, fee).

**Design and rule questions for him, not AI fixes:**

- Should an empire be allowed to raze its last city? The Mongols did at T70 and eliminated themselves.
- The Swiss treasury is private while the bounty per city is public. Either is fine, but the AI then has to infer the margin (recommendation 2); a public Reserve margin during a hold would also serve pillars 4 and 5 for human rivals.
- Pillar 11: a single 16-turn war produced a known winner at T70. The rubber bands did nothing about it. Whether a conquered capital plus an era objective plus the crowns that come with the land should all arrive together is a design question.

## Scenarios (not created in this pass; commands for the lead)

I was limited to one file, so nothing was cut. Recommended, first one first. Check exact turn numbers with `cut.sh SAVE --inspect`.

1. `tools/scenarios/cut.sh ~/LegatumRuns/bug-saves/swiss-win-2026-11-11.legatum --turn 49 --seat 4 --id mongols-hold-shangdu --title "Hold Shangdu" --question "Five Swiss units are at the gate and your army is in the south. Can you keep your cities?" --objective hold --turns 20`. Why first: it tests whether the T51 strike is defendable at all by a human from the Mongol seat. If he cannot hold it either, recommendation 1 is a rules or balance question, not an AI one.
2. `tools/scenarios/cut.sh … --turn 129 --seat 3 --id aztecs-break-the-reserve-2 --title "Break the Reserve" --question "Switzerland starts its hold next turn. Which city do you take?" --objective score --turns 10` (Aztec seat, units by Bern). An existing scenario, `celts-break-the-reserve` (Hard 0/50), covers the same idea from the 8.9.26 game; cut this one only if that one is stale. Past runs of `swiss-hold-biel` were rated "too easy" (Fun 1), which fits: nothing threatens a Swiss hold.

## Proposed playbook changes (not applied)

- New pattern: **"Hard stops after the first capture; he finishes the neighbour."** Evidence this game; check older tapes (8.9.26 Sparta, Golden Horde) before calling it recurring.
- New pattern: **"The strike is visible for turns before it lands."** Units massed inside a rival's borders at Peace.
- Strengthen "nobody contests the hold" with: the break term now fires and mis-targets; add the Reserve-break table to the checklist for every Swiss game.
- Mark "He declares war; Hard waits" as not seen here (Hard wanted the T51 war earlier than he did).
- Verdicts table row: 2026-11-11, Switzerland, win T139, decided T54 to T67, fair.
- `human-reasoning.md`: nothing to add until he answers the questions below.

## Method: what I did differently, and what the usual method can miss

1. **Outcome-check every denial forecast.** The plan trace said "break the hold 1,785". The usual reading is "the AI contested the hold and the fix fired". The VP track says the sack removed nothing. Any term that claims a VP swing should be checked against the VP actually moved the next turn.
2. **Compute the break table, not the anecdote.** "Geneva was unwalled near the Celts" is the old playbook line. Per city, per turn, bounty against margin shows the window was every city but one at T130 and closed in three turns. That turns a pattern into a testable target.
3. **Use the takeover curve to locate the game before reading the diff.** The single-turn diff ranks T119 to T123 Settler moves first, 60 turns after the game was decided, because its horizon is T140 and those turns sit next to the finish. The curve puts the decision at T54 to T60, and the per-turn plan notes (Hard planning Peace) explain it. A multi-turn commitment never shows up as a one-turn swap. I would run the curve first in every review and treat one-turn swap values near the end of a won game as horizon effects.
4. **Do not restate a pattern without checking its sign.** "He declares; Hard waits" and "he expands under war; Hard does not" both read true from the tape alone and are both false against Hard in his seat this game.
5. **Fog check on every "should have".** Recommendation 1 uses only units inside Mongol borders. Recommendation 2 exists because of fog: the Celts knew 4 of his 12 cities, and the right fix uses public facts, not the true treasury.
6. **Wrong-seat numbers.** The curve's seat plays current master while rivals play the tag; the diff plays the tag on both sides. I kept them apart above. Snapshot turn t holds state turn t+1; all turns here are event turns.
7. Deviations from the written method, forced by this pass's limits: no chart, no scenario files, no playbook, ideas, learnings or human-reasoning edits, no band-off counterfactuals, no rival what-if branches. Kill attribution by empire is by last combat contact within a turn, so 26 AI unit losses with no recent contact (barbarians, disbands, upgrades) are left out of the exchange ratio.

## Intent questions for him (only what the tape cannot answer)

1. You refused the Mongol Peace offer at T59 with two of their cities taken. What told you to take all of it rather than bank the gain?
2. From T70 you ran 4 to 7 units for 12 cities at War with four empires. Was that a read that they would not come, and what would have made you buy a defender?
3. At T130 to T131 your Reserve was 151 to 338 Gold above 5,000. Did you know that losing almost any city, or buying one unit, would have reset the hold?

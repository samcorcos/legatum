# Coach's review: the 8.9.15 Swiss win (2026-10-28)

Save: `~/LegatumRuns/bug-saves/swiss-win-2026-10-28/autosave.legatum`. Engine `060f78ef4` (tag `playtest-8.9.15`; 8.9.15.1 has the same AI). Four empires (Switzerland, Japan, Sparta, the Mongols) on a 40x26 map, every AI on Hard. Replayed at the tag with 0 hash mismatches in 180 turns; the AI plan trace (`p881_trace`) matches all 513 recorded AI order lists. A logging copy of the door defense printed every front it saw. What-ifs: 40 branches, 32 or 64 runs each, on the box (about 6 minutes). Tools and data: `~/LegatumRuns/coach-tools/coach-8915-swiss/`.

VP race and army size (archived outside the repository)

## Coach's verdict

You won by playing the same way you won as Athens and as Sparta: almost no army, Walls everywhere, a big treasury, and one short war at the right moment. For 107 turns (T30 to T137) you had one or two military units for up to eight cities. No AI made you pay for it. The AIs hit your cities 15 times in 180 turns, 11 of them Spartan Soldiers and Rangers against Walls.

The game turned between T137 and T150. The Mongols, the strongest AI, declared on you at T137. You bought one Arquebusier in the threatened town, their campaign ended two turns later, and they offered Peace. The same thing happened at T158 with a Field Cannon. Meanwhile three Field Cannons and three Riflemen took all five Spartan cities (T153 to T171). Sparta's capital held its Great Work, which was worth +2 VP to whoever held it. That capture took you from 7 to 10 VP and started the hold.

"Pretty easily" is fair, and you're right that the cause isn't Switzerland. The failures are the same ones the Athens and Sparta reviews found:
- the AIs don't read an empty, rich leader as a threat;
- their wars on you end on the first defender you buy;
- they sign Peace with the leader while their own plan says to fight them.

In the what-ifs, the one change that hurt you was the Mongols keeping their T137 war. Hard playing your seat then wins 25% of games instead of 44%, and your own recorded orders stop winning altogether.

## 1. Result and margin

- **Result.** You won on T180 by holding 10 VP for 10 turns (T171 to T180). Japan and the Mongols ended on 2 VP; Sparta was eliminated on T171.
- **Your 10 VP:**
  - 3 era objectives: Gain 7 cities (T136), Mills in 3 more cities (T155), Capture 1 enemy city (T171);
  - the 5,000-Gold reserve (T166, +2);
  - Sparta's captured Great Work (+2);
  - three crowns: Largest Economy (359 to 184), Strongest Army (1,328 to 612), and Most Advanced (two techs ahead of the Mongols).
- **The race.** Nobody had more than 3 VP until T158. You had 2 at T100, 4 at T150, 5 at T158, 7 at T166 and 10 at T171. Pillar 11 held on the VP track: nobody could have called it at T100.
- **When it was decided.** Hard playing your seat (64 runs from each turn) wins 6% of games from T120, 44% from T137, 80% from T146 and 88% from T158. Your own recorded orders win 77% from T137 and 100% from T150. So it was decided between T137 and T150, while your cannons were staging on Sparta.
- **Play time.**
  - 28 active minutes over 2 sessions.
  - 0.1 minutes a turn in eras 1 to 3, 0.2 in era 4, 0.3 in eras 5 and 6.
  - 405 orders (34 of them automation), against 1,154 to 1,767 for each AI.

## 2. Turning points

| Turn | What happened | Swing |
|---|---|---|
| T40–T44 | Sparta declared on you (a "forced expedition", value 0) with 4 Soldiers and a Ranger. It hit Basel 4 times; the campaign ended `insufficient_army` on T44. Sparta then stayed at War with you for 131 turns, because its faction rule makes regiments desert at Peace. Its campaigns were valued −388 to −176 and took one city (Solothurn, T110, razed T148). | A 131-turn war that fed you 59 kills and cost Sparta its economy |
| T118 | You declared on Japan and captured an unescorted Japanese Settler the same turn (Peace T142). | +1 city for free |
| T137–T141 | The Mongols (11 cities, 12 military units) declared and aimed at Chur (pop 1, no Walls, valued +3,696, mostly the "supply" term). You bought an Arquebusier in Chur on T138. The campaign ended `insufficient_army` on T139. Their fallback was Peace, valued −300 ("abandons a declared war"); they offered it T140 and you accepted T141. | **The decisive AI decision** (section 5b) |
| T144–T149 | You built 2 Field Cannons and paid for Siege on three units (T149, 314 Gold). Your Strongest Army went from 180 at T140 to 612 at T150. | Your army now outclassed Sparta's Ancient units |
| T153–T171 | You took Amyclae (T153), Therapne and Sellasia (T161), Sparta (T168) and Pylos (T171, the capital, with the Great Work). Field Cannon shots did 140 to 259 damage to 200-HP cities. Sparta asked for Peace on T148, T157 and T166; you refused each time. Japan and the Mongols stayed at Peace with you. | 4 → 10 VP |
| T158–T162 | The Mongols declared again and aimed at Solothurn (pop 1, 51%). You bought a Field Cannon there on T159 and the campaign ended on T160. On T161 the Mongols planned a campaign on Lausanne (value +826) and in the same turn proposed Peace to you. You accepted on T162, and on T163 their plan failed as "war_illegal". | Your back stayed free for the Spartan war |
| T171–T180 | The hold. Japan (T172) and the Mongols (T174) declared, with "contest the hold" valued +460 to +680. They attacked no city, and the Mongols proposed Peace again on T175. You took Matsue from them on T178. | Win |

## 3. Your edge (what you did that the AI doesn't)

- **Army size and timing:**
  - 1 or 2 military units from T30 to T137, while the AIs kept 5 to 15 each;
  - every city got Walls first in its build queue;
  - you built an army only for the war you chose: 2 Field Cannons by T145 and 6 units at most.

  This is the third game in a row where you trusted the AI not to punish an empty empire (Athens: 3 units for 16 cities; Sparta: 2 units from T70). It never has.
- **Buying the defender at the door.** Twice you bought one unit in the one town an AI army was walking to (Chur T138, Solothurn T159), and twice the AI's campaign ended within two turns. A human answers the army they see with one purchase; the AI reads that one unit as "insufficient army" and gives up the whole war.
- **Target choice.** You picked the weakest neighbour, the one that held a public +2 VP Great Work in its capital, and took it in 18 turns. You refused its three Peace offers.
- **Economy.**
  - Gold income: 53 a turn at T100, 104 at T150, 177 at T180. The best AI earned 20, 43 and 57 at the same turns; Japan and Sparta earned 0 to 17.
  - Treasury: 1,847 Gold at T100, 4,062 at T150, 7,129 at the end. No AI had more than 190.
- **Expansion was not your edge.** You had one city until T31 and 5 at T90. The Mongols had 7 at T90 and 11 to 12 from T120. You overtook them only through conquest (15 cities at the end).
- **Faction signature:** you used it fully. Interest, the hoard reached on T166, and Gold as an instant defence.
- **Diplomacy:**
  - you accepted every Mongol Peace (T141, T162) and refused every Spartan one;
  - you declared once, on Japan, to steal an unescorted Settler (T118).

## 4. The AI's mistakes

### The Mongols (the strongest AI: 11–12 cities from T120, army 250–800, the strongest until T144)

**Strategic:**
- **Busy elsewhere for 90 turns.** They fought Japan from T87 to T174: six campaigns, three captures, two of them razed later. Their own win-chance estimate read 74–99% from T87 to T158, so in their own model they were winning.
- **They didn't see you until T158.** At T136–T141 their threat score for you was 30–44, with no VP-race term. You had 2–3 VP, tied with them, and the runaway signs only count for the strict VP leader. Your 1 military unit, 3,600 Gold and two crowns counted for nothing. You became their top threat at T158 (score 87), when you were already taking Sparta.
- **Decadence taxed them, not you.** They paid 15–22% from T50 onward: 15.6% of their Production, Gold and Science (3,927), against your 8.4% (2,289).

**Tactical:**
- **Two wars ended on one purchase each.** The Chur campaign (37%, T137) ended on T139 and the Solothurn campaign (51%, T158) on T160. Each time you had 7 other empty cities in reach.
- **Peace against their own plan.** They proposed Peace to you on T161 while their plan was a campaign on Lausanne (+826), and again on T175 during your hold ("contest the hold" +680). The code path: in `diplomacy.rs::wants_peace`, the rule "Hard's plan decides" returns true whenever `endangered()` holds, even for the team the plan is fighting. `endangered()` counts pressure from any enemy at War, so Japanese units near Mongol towns were enough.

### Sparta (your neighbour and prey)

- **Strategic:** it was at War with you for 131 turns with an army that couldn't open a walled city: Soldiers, Rangers and Horsemen into era 4 and 5. Every campaign ended `insufficient_army` or `infeasible` within 2 to 6 turns. Its Gold income was 0–17 a turn, and the regiments-desert rule kept it fighting anyway.
- **Tactical:**
  - Its Gold went on tile improvements while at War: 8 purchases (Quarries, Farms, Saltpeter Works) between T60 and T145.
  - When your cannons came (T146–T171) it had 14–174 Gold and bought one Soldier.
  - It razed Solothurn when your army arrived (T148, by design: denial under counterattack).

### Japan

- **Strategic:** a Contain plan against the Mongols from T39 to T172 (valued +421 to +1,388, as the "leader's race"). It never treated you as a threat until T145 and never helped Sparta.
- **Tactical:** it lost two Settlers to unescorted moves (T37 and T118, to the Mongols and to you). Its Gold income was 0–6 a turn all game.

### Nobody stopped the conquest

Sparta had a public +2 VP Great Work in its capital. While you took five Spartan cities, Japan and the Mongols were at Peace with you for most of it, and the Mongols signed a second Peace mid-conquest. A coalition forced from T150 (Japan and the Mongols at War with you, no Peace) changed nothing: Hard in your seat won 18/32 without it and 20/32 with it. By T150 the cannons were already in place.

No cheating-adjacent idea is needed for any of this. Everything above is public: your units in sight, your treasury, your crowns, Sparta's Great Work.

## 5. Was the win fair?

**Mostly skill the AI can learn, plus a friendly neighbour.**
- **Skill:** the cheap-defence-plus-one-war pattern is learnable, and the AI's counter is to attack where you are empty and keep the war going.
- **Exploit:** the "buy one defender and the campaign quits" response is close to an exploit of a brittle forecast. It isn't a rules hole: the purchase is legal and costs Gold.
- **Luck:** Sparta, poor and obsolete and holding a Great Work, sat next door.

**Switzerland's balance** (the designer's call, 2026-10-28): not the problem; you win this way with every faction. No balance work follows from this game.

## 5a. Smarter AI or a weaker leader?

| | You | Mongols | Japan | Sparta |
|---|---|---|---|---|
| Decadence, share of Production + Gold + Science | 8.4% (2,289) | 15.6% (3,927) | 0% | 0% |
| Rate at T100 / T150 / T180 | 3 / 10 / 16% | 17 / 16 / 8% | 0 | 0 |

- **Crown upkeep:** no longer exists (removed 2026-10-20).
- **Tech diffusion:**
  - the AIs learned 54 techs after you knew them (Japan 19, Sparta 19, Mongols 16);
  - your share of each tech's discount is 10% of its cost;
  - that is roughly five techs' worth spread over three empires, small next to the decadence they paid.
- **Counterfactual (Hard in your seat), decadence off:**
  - from T100: 5/32 → 1/32, and the Mongols win 31 of 32;
  - from T137: 44% → 47%.
- **Verdict:**
  - the bands didn't make this game close; they mostly taxed the challenger;
  - the designer's win owes about 0% to the bands and about 100% to AI decisions, give or take about 10 points for the T100 sample (32 runs).
  - The AI-smartness benchmark wasn't rerun: no gene changed since the W2.1 gate. Door defense there: seat −1.2 pp, rivals +1.0 pp, both noise.

## 5b. What would have beaten the designer

The what-ifs branch from states at the release tag. Your seat was played two ways:
- **Hard**, which approximates you, with reseeded dice;
- **your recorded orders**.

Neutral controls show how fragile your recorded line is. Changing a gene that does nothing in this game (Fighter pricing, with nobody at Flight) leaves it at 49/64 from T137 and T146, and 64/64 from T150. Any change in the Sparta or Mongol theatre, though, breaks it to 0/32 with about 240 failed orders. Read the recorded-orders column as an upper bound on the harm, and the Hard column for size.

| Change (AIs only) | From | Hard in the designer's seat | The designer's recorded orders |
|---|---|---|---|
| **Control** | T137 | 44% (28/64) | 77% (49/64) |
| **The Mongols never make Peace with the designer** | T137 | **25% (16/64)** | **0/32** (Mongols win 32/32) |
| Japan and the Mongols at War with the designer, no Peace | T137 | 10/32 (control 12/32) | 0/32 |
| Commitment genes on (`leader_commit` 2, `contain_after_campaign`, `coalition_own_read`) | T137 | 48% (31/64) | |
| Decadence off | T137 | 47% (30/64) | |
| Control | T120 | 6% (4/64) | |
| The Mongols declare at T120 and never make Peace | T120 | 16% (10/64) | |
| Commitment genes on | T120 | 16% (10/64) | |
| Control | T146 | 80% (51/64) | 77% |
| Door defense off in every AI | T146 | 78% (50/64) | 0/32 (fragility) |
| Coalition at War, no Peace | T150 | 20/32 (control 18/32) | 0/32 |
| Control | T158 | 88% (56/64) | 100% |
| The Mongols never make Peace with the designer | T158 | 84% (54/64) | 32/32 |
| Japan and the Mongols refuse Peace during the hold | T171 | | 32/32 |

**Ranked:**
1. **The Mongols keep the T137 war** (refuse the Peace their ended campaign fell back to): −19 points for the Hard seat, and the designer's recorded line no longer wins.
   - Learnable from the AI's own information: yes. The empty cities were in sight, and the treasury and crowns are public.
   - It maps to two fixes: don't end a war on one bought defender, and no Peace with the leader the plan is fighting (recs 1 and 3).
2. **Nothing else moved it:**
   - The same war started earlier (T120) helped the designer: 6% → 16%, because it wasted the Mongol army before the designer was hollow.
   - The existing commitment genes changed nothing.
   - Coalitions came too late.
   - The bands and door defense made no difference.
   - Once the designer's cannons stood at Sparta (T146), only a change in the Sparta theatre broke the designer's recorded line, and the Hard seat still wins 80% there.

## 6. Did the 8.9.15 changes show up?

- **Door defense: it fired, and changed nothing.**
  - Fronts opened on 24 Spartan turns (T146–T171), with gaps of 100 to 650 squared-strength. The Mongols had fronts on 65 turns and Japan on about 100, mostly against each other's armies.
  - **It bought 0 units in the whole game.** Sparta's spendable Gold was 14–156 and the Mongols' 41–146. Every AI ran its treasury near zero.
  - It switched production 8 times, mostly to Horsemen and Soldiers (Sparta's best options), moved 11 garrisons and made 2 focused strikes (both Japanese).
  - With door defense off from T146, Hard in the designer's seat wins 78% against 80% with it.
  - The feature works as written; it has nothing to spend.
- **Fighters: untested.**
  - No AI reached Flight. The Mongols were researching it at the end.
  - The designer got Flight around T173 and queued Bombers from T174; none was finished.
  - Nobody faced a Bomber.
- **Bugs or self-sabotage:**
  - **Peace offers against the plan** (T161, T175): see section 4. This is the clearest bug.
  - **Campaigns abandoned in 2 turns** after one purchase (T139, T160). `campaign_persist` keeps a campaign only while the committed force at the city still matches the local defence, so one bought unit flips it, and the plan falls back to Peace rather than to another city of the same enemy.
  - **No idle Gold.** The opposite: no war chest.
  - **No disbanding** at War.
  - **Razes:**
    - The Mongols razed two Japanese cities they had held for 16 and 71 turns (Fukuoka T132, Matsue T162), going from 12 cities to 10.
    - Sparta razed Solothurn.
    - The raze rule fires under counterattack pressure, by design. It cost the strongest AI two cities.

## 7. Coach's recommendations (ranked by expected effect on how often the designer wins with any faction)

| # | Mechanism | Where | How to measure | Evidence across games |
|---|---|---|---|---|
| 1 | **A war on the leader survives one bought defender.** When a campaign's target gains a defender, re-aim at the same enemy's next empty city in reach instead of ending the war; price the public treasury as one purchase per threatened city per turn, not as a full garrison, when choosing the target; only a forecast with every reachable city closed ends the war. | `campaign.rs` (end on `insufficient_army`), `plan.rs` fallback after a campaign ends, `military.rs::attack_targets` | New war fixtures from this save (Mongols at T137 and T158 against his recorded orders and against a Hard Swiss); war suite; human-seat benchmark in both roles | This game (T137, T158; −19 pp), 8.9.4 Swiss (Japan T142, two Marksmen), 8.9.13 Athens (Sparta's wars switched away in 1–3 turns), Golden Horde regression |
| 2 | **Read the hollow leader.** The threat model's runaway signs count without a strict VP lead: public military strength per city (Strongest Army against city count), live crowns, the public treasury, an era lead. A leader with 1 unit for 8 cities and two crowns is the best target on the map, whatever the VP tie. | `threat.rs::runaway_signs`, `plan.rs::contain_leader`, `military.rs` target value | Benchmark; the T137 branch with the change in the Mongols; the threat log's first-named turn in the war fixtures | This game (threat 30–44 until T158), 8.9.13 Athens (14 of 16 cities empty, one attack), 8.9.13 Sparta (0 attacks on empty cities), 8.9.2 Swiss (named at T132) |
| 3 | **Peace that follows the plan.** In `wants_peace`, `endangered()` should count only the enemy we'd make Peace with, and never override a running campaign or Contain against that same team, nor a hold contest. | `diplomacy.rs::wants_peace` ("Hard's plan decides" branch), `diplomacy.rs::endangered` | `peace_flip_5/10` counters in `hard_vs_human`; the T158 branch; war suite | This game (T161 mid-campaign, T175 mid-hold), 8.9.2 Swiss (Mongol flip T168–T172), 8.9.4 Swiss (Athens Peace as the hold began) |
| 4 | **Contest the leader's conquest of a VP holder.** When the leader is at War with a weak neighbour whose capital or Great Work is a public VP source, add that VP to the threat and to a Contain war's value, early, while the leader's army is staging. | `threat.rs`, `plan.rs::contain_candidates` (merges ideas #27 and #33) | What-if from T146 with the change; benchmark | This game (Pylos Great Work +2 decided the hold), 8.9.5 Athens, 8.9.13 Athens (Great Work cities) |
| 5 | **A war chest the door can spend.** While a visible enemy army is within march, no Gold on tile improvements; keep a reserve sized to the door's gap. | `production.rs` improvement purchases, `door.rs::door_gold_floor` | War suite defence fixtures; the adaptive Sparta check (bigger standing defence backfired before, 2026-10-27) | This game (0 door buys; Sparta bought 8 improvements at War), 8.9.13 Sparta (3 units bought in 80 turns of war) |

**Design questions for the designer (not AI fixes):**
- **Sparta's regiments-desert rule** kept an AI Sparta in a hopeless 131-turn war with its strongest neighbour. That fed the human 59 kills and an easy conquest. Is that the intended cost of the faction?

## Scenarios

- **`mongols-rich-and-empty` (show first).**
  - **The setup:** T137, you play the Mongols. Switzerland has 3,600 Gold and one military unit for eight cities.
  - **The goal:** take Lausanne and St. Gallen within 20 turns.
  - **Hard's baseline:** 0/50. It declares on T137 in every run and takes neither city (median score 1,250; 6 runs also lost a city to Japan).
  - **The question:** how a human attacks a rich leader who can buy a defender anywhere, and whether it's possible. A win here is the recipe for rec 1.
- **Dropped: a Sparta defence at T158.** Hard held 50/50 because the Hard Swiss in the designer's seat never attacked, so it tests nothing.
- **Earlier scenarios:** no Scenario Lab runs since 2026-10-22.

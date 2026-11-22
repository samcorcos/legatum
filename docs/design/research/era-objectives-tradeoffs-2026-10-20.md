# Era objectives as trade-offs: a new pool for eras 2 to 7

Design research, 2026-10-20. A proposal for the user to approve; nothing here is built. Decisions go to the design doc's decisions log and the `legatum-design` skill. Every rule below is checked against `.claude/skills/legatum-rules/references/game-rules.md` §6 (VP v2) and `crates/core/src/objectives.rs`.

**The prompt (user, 2026-10-20):** "the era scoring events are sometimes too connected. e.g. in one era it's 3 universities and the next era it's 5 or something. it feels like the ways you score shouldn't be so linear... like in TI4, every point feels like a tradeoff you have to make. it never felt like I could just get all the points by accident if I just played the game normally."

**Already decided (decisions log, same day):** no linear chains; nothing a normal game completes by accident; one event scores at most one objective. Measure "objectives met without being pursued" in sims.

**Kept:** the shape players know. Every era after the first shows three public objectives (one Build, one Expand, one Fight), the same for everyone; meeting any one earns that era's single permanent VP; unscored eras stay open. Crowns, faction goals, Great Works, the 10 VP target and the 10-turn hold are untouched.

Contents:
1. What is wrong today, measured
2. Principles, in TI4 terms, tested against our rules and pillars
3. The new pool for eras 2 to 7
4. Rules changes, AI work, measurement plan
5. Questions for the user

---

## 1. What is wrong today, measured

**Method.** Two cohorts of 256 all-AI games on the cloud box, 8 Hard empires, Standard speed and map size, the same seeds (9100 onward) in both, master `60adbddd3`: **Hard** as shipped, and **objectives off**, Hard with `--ai-params` setting `objective_bonus_percent` to 0 and `objective_horizon` to its minimum (10). No knob turns pursuit off completely (`objective_capture_value` still weighs a capture that would meet a live Fight objective at 1, through `weight.max(1)`, and `objective_item_points` still counts), so the second cohort is the closest proxy, and the Hard cohort's undrawn objectives are a cleaner one: an objective that was not drawn in a game is never pursued by anyone. A tape replay (a throwaway `crates/sim/examples` program written for this, not committed) checked every objective of every era pool for every empire that entered the era, after every turn, at the lobby's real counts. "Met" means at any time before the game ended (median 313 and 310 turns; eras stay open). Raw output: `~/LegatumRuns/box/20261020T231721Z-2f8f57/artifacts/oa-full/{hard,noobj}/accident.txt`.

**Finding 1: today every era VP is scored by accident.** Aiming at an objective changes nothing measurable. The two cohorts scored 5,673 and 5,670 era objectives; per era, 59/57/52/43/46/39% of entrants scored with pursuit and 60/59/52/45/43/38% without. For every one of the 50 objectives, the share of entrants meeting it is within a few points whether it was drawn (and so pursued), not drawn, or pursued by nobody (largest gaps: Castles in 3 cities 24% drawn against 15% not drawn, a Research Lab in 3 cities 17 against 13, a city from the leader in the Modern era 23 against 18). The objectives are measuring how a game goes, not choices. (The AI's pursuit is also weak, which section 4 addresses; but the user's point is the same: a normal game produces the points.)

| Era | Objective (8 empires, counts at 150%) | Hard: met when drawn | Hard: met when not drawn (never pursued) | Objectives off: met |
|---|---|---:|---:|---:|
| 2 | Have a Library in 6 of your cities | 23% | 24% | 25% |
| 2 | Have a Market in 6 of your cities | 15% | 12% | 14% |
| 2 | Have a Workshop in 8 of your cities | 12% | 10% | 11% |
| 2 | Own 9 cities | 22% | 24% | 24% |
| 2 | Reach 66 total population | 33% | 32% | 31% |
| 2 | Own 117 tiles of territory | 29% | 29% | 28% |
| 2 | Capture 2 enemy cities | 56% | 56% | 55% |
| 2 | Destroy 36 enemy military units | 49% | 51% | 50% |
| 3 | Have a Library in 8 of your cities | 7% | 8% | 8% |
| 3 | Have a Walls in 8 of your cities | 19% | 17% | 17% |
| 3 | Have a Castle in 3 of your cities | 24% | 15% | 18% |
| 3 | Own 11 cities | 13% | 11% | 11% |
| 3 | Reach 75 total population | 21% | 24% | 23% |
| 3 | Own 132 tiles of territory | 19% | 21% | 21% |
| 3 | Capture 1 city from an empire ahead of you on VP | 52% | 53% | 54% |
| 3 | Destroy 44 enemy military units | 41% | 41% | 42% |
| 4 | Have a University in 6 of your cities | 8% | 7% | 7% |
| 4 | Have a Bank in 5 of your cities | 4% | 2% | 2% |
| 4 | Have a Mill in 5 of your cities | 2% | 2% | 2% |
| 4 | Own 11 cities | 12% | 12% | 12% |
| 4 | Reach 81 total population | 17% | 18% | 18% |
| 4 | Own 144 tiles of territory | 16% | 16% | 16% |
| 4 | Capture 1 city from an empire ahead of you on VP | 53% | 47% | 50% |
| 4 | Destroy 44 enemy military units | 38% | 37% | 38% |
| 5 | Have a Forge in 5 of your cities | 4% | 2% | 3% |
| 5 | Have a Factory in 3 of your cities | 11% | 10% | 11% |
| 5 | Have a University in 6 of your cities | 8% | 7% | 7% |
| 5 | Own 11 cities | 12% | 12% | 12% |
| 5 | Reach 80 total population | 19% | 19% | 20% |
| 5 | Own 143 tiles of territory | 16% | 16% | 17% |
| 5 | Take 1 city from the VP leader | 21% | 22% | 20% |
| 5 | Destroy 33 enemy military units | 40% | 44% | 44% |
| 6 | Have a Factory in 3 of your cities | 11% | 10% | 11% |
| 6 | Have a Research Lab in 3 of your cities | 17% | 13% | 13% |
| 6 | Have a Bank in 6 of your cities | 1% | 0% | 0% |
| 6 | Own 12 cities | 8% | 8% | 9% |
| 6 | Reach 89 total population | 14% | 12% | 13% |
| 6 | Own 156 tiles of territory | 11% | 12% | 11% |
| 6 | Capture 2 enemy cities | 43% | 43% | 41% |
| 6 | Take 1 city from the VP leader | 23% | 18% | 18% |
| 6 | Destroy 30 enemy military units | 40% | 37% | 39% |
| 7 | Have a Research Lab in 5 of your cities | 2% | 1% | 1% |
| 7 | Have a Star Fort in 5 of your cities | 0% | 0% | 1% |
| 7 | Have a Factory in 5 of your cities | 0% | 1% | 1% |
| 7 | Own 12 cities | 8% | 9% | 9% |
| 7 | Reach 99 total population | 8% | 8% | 8% |
| 7 | Own 174 tiles of territory | 9% | 7% | 8% |
| 7 | Capture 1 enemy city | 57% | 54% | 53% |
| 7 | Take 1 city from the VP leader | 18% | 17% | 16% |
| 7 | Destroy 29 enemy military units | 30% | 32% | 31% |

**Finding 2: Fight objectives are most of the points, and they come with any war.** Of the Hard cohort's era scores, 72% were Fight (destroy N units 28%, capture N cities 19%, a city from an empire ahead 17%, a city from the leader 7%), 16% Expand (cities 4%, population 6%, territory 6%) and 12% Build. "Capture 2 cities", "capture 1 city from an empire ahead" and "capture 1 city" are met by 50 to 56% of entrants who never aimed at them; "destroy N units" by 30 to 50%.

**Finding 3: one event scores several eras.** A third of all era scores (33%; 35% with objectives off) land on a turn when the same empire scores another era too: 819 such turns, and in 545 of them two eras were scored by the same kind of objective. 63% of the "city from an empire ahead" scores and 78% of the "city from the leader" scores were in such turns: the Medieval and Renaissance pools both hold "capture 1 city from an empire ahead", and an unscored Medieval era counts every capture since it was entered, so one capture scores both. Scores on the very turn an era is entered are rare (1%), because the 150% counts (2026-10-18) put every threshold above what an empire carries in; the stacking happens later, through open eras.

**Finding 4: the pool is a set of linear chains.** One building family feeds four eras: a Library in 6 cities (Classical), in 8 (Medieval), a University in 6 (Renaissance and again Industrial), a Research Lab in 3 then 5 (Modern, Future), and a higher tier counts as the lower. Own 9, 11, 11, 11, 12, 12 cities; population 66 to 99 and territory 117 to 174 in every era; destroy 29 to 44 units in every era; a Factory in 3, 3, then 5 cities. This is exactly "3 universities, then 5".

**Finding 5: some options are dead.** Seven of the 50 are met by at most 4% of entrants, pursued or not: a Bank in 5 and in 6 cities, a Mill in 5, a Forge in 5, a Star Fort in 5, a Factory in 5 and a Research Lab in 5 cities. When one is drawn, that family's slot is empty for the era: a dead option, which the design skill calls a design bug.

**Finding 6: strategic diversity is already low.** Wide empires won 70% of these games (179 of 256; Rush 20%, Turtle/Wonder 9%, Economic and Science none), winner entropy 0.46 against the 0.80 guardrail. Objectives that reward size feed the archetype that is already winning.

---

## 2. Principles, in TI4 terms

TI4's public objectives feel like trade-offs for four reasons (the scoring numbers are in `docs/research/ti4-objective-scoring.md`; the card rules are from memory, so approximate). Each is tested against Legatum below.

**P1. Scoring costs something you could have used elsewhere.** TI4's most common Stage I cards are "spend 8 resources", "spend 8 influence", "spend 5 trade goods": the point is bought with what would otherwise have built ships. Other cards ask for a position that costs: ships parked next to Mecatol Rex or in a rival's home are ships not defending yours.
*Legatum test:* holds. Production, Gold and units are already the currencies (influence was removed so that "every trade-off is priced in Gold"). An objective can cost Production (a project that does nothing else), Gold (a purchase), units (a garrison held far from home) or strategic-resource supply (a building that uses Iron is Iron a Pikeman cannot use). Pillar 5 holds as long as the price is shown before you commit: "Triumphal Arch: 240 Production, 1 VP".

**P2. The thresholds sit outside what a normal game does.** TI4 never asks "have 6 planets" when everyone has 6 planets by round 3; it asks for planets of one trait, or in non-home systems, or adjacent to a rival. The condition is a deliberate shape, not a size.
*Legatum test:* fails today (section 1). Population, territory, city count and "a Library in N cities" are size measures, and size is what every empire grows anyway. The fix is to ask for shapes: where a city is (near a rival, far from home, on a contested site), what it runs (a concentration of resource-using buildings), what you refrained from (no war), what you took (a walled city, a resource city, the leader's city).

**P3. Objectives compete with each other.** The spend cards draw on the same pool as your fleet; "control Mecatol" and "keep 3 systems with no rival ships" pull your ships in opposite directions. You choose which points to chase.
*Legatum test:* holds, with one VP per era. The three drawn objectives of an era should pull in different directions, so that the plan that scores one makes the others harder: the Build options are peaceful spending (a monument, a peace, a city of industry), the Expand options are exposure (a city where it can be taken), the Fight options are taking things. Peace and Fight in the same era are mutually exclusive by construction. Pillar 10's rock-paper-scissors gets a VP edge: defending against a rush can score (Fight "destroy units inside your borders"), so attacking a turtle feeds it.

**P4. A limit per round, and nothing counts twice.** TI4 lets you score at most one public objective per status phase; a card scored is done, and what scored it does not score the next card.
*Legatum test:* fails today. A capture made after entering two eras counts toward both (the Fight counters run from each era's entry, and both eras stay open), a city count met once satisfies every later "Own N cities" at or below it, and one empire can score several eras in a single turn (section 1). The fix is one rule (R1 below): progress counts from your last era score, so what scored one era cannot score another.

**Pillars.** Pillar 4 (visible race): every objective shows progress and is public, and the new ones are things rivals can see coming (a monument's progress, a forward city, a peace streak). Pillar 10 (no dominant strategy): each family has a peaceful, an exposed and a violent path, and each objective names its counter. Pillar 11 (competitive to the end): two Fight kinds aim at the leader, and "count from your last score" stops a leader from banking accidental points. Pillar 8 (asymmetry): checked per faction in section 3.

**The design rules that follow.**
- Every objective is one sentence, names what you give up, and names how a rival stops you.
- No objective kind appears in two consecutive eras, and when a kind recurs it asks for a different thing (another building, resource, currency or target), never the same thing with a bigger number.
- No raw size measures (population, territory, city count, technology count, a common building in N cities).
- Resources enter through use and taking, never "own N deposits": run buildings that need the supply, concentrate several in one city, gain a city on a contested deposit, or take a rival's industrial city.
- No exceptions for edge cases (user, 2026-10-20): retaking your own city is a capture; a faction that cannot do one option of an era has the other two.

---

## 3. The new pool, eras 2 to 7

Twelve kinds, four per family. Each kind is in every other era, so no kind appears in two consecutive eras, and each recurrence asks for something different. An era draws one Build, one Expand and one Fight from its two options per family (the current draw, `Rng::derive(seed, [era, OBJECTIVES, slot])`).

"Gain a city" means found or capture, after the counting start (R1). "Running" means supplied under the resource-supply rule (decided 2026-10-19: each deposit supplies about 2, +1 with a Mine; each user uses 1; the newest users go idle when short). Counts are for 8 empires and starting points to calibrate (section 4); costs are at Standard.

### The twelve kinds

| Kind | Family | Rule (one sentence) | What you give up | Counter |
|---|---|---|---|---|
| Monument | Build | Complete the era's monument, a city project that does nothing else. | Its Production (or Gold): two or three units or a building | Take or besiege the city; the progress is public and stays with the city |
| Industry | Build | Have N of the era's resource buildings running at once. | Supply that units could use; deposits you must win | Take or cut a deposit city, and the newest buildings go idle |
| Great city | Build | Have one city running all of a set of resource buildings. | All eggs in one city, and supply of three or four resources | Take that one city |
| Peace | Build | Sign peace with N empires you were at war with, then stay at Peace with everyone for 20 turns. | Every war gain, the era's Fight objective, conquest | Declare war on them before the 20 turns run out |
| Frontier | Expand | Gain cities close to a rival. | A safe site; you provoke a neighbour | Settle the gap first, or take the city |
| Far city | Expand | Gain a city far from your capital. | A defensible core; a city hard to reinforce | Take it |
| Contested site | Expand | Gain a city that owns a scarce site (a natural wonder, a Silver or Coal deposit). | The site everyone wants, usually at a border | Get there first, or take it |
| Hold | Expand | Hold captured cities for 20 turns. | Garrisons; the decadence a bigger empire pays | Retake one before the 20 turns |
| Siege | Fight | Capture a city with a given wall tier (or a capital). | An army and Engineers, against the hardest target | Build the next wall tier |
| Topple | Fight | Take a city from an empire ahead of you, or from the VP leader. | A war you choose for position, not for land | Stay level, or be the leader with a strong border |
| Raid industry | Fight | Capture or raze a city running a given resource building. | A war aimed at a rich city, often deep in enemy land | Garrison and wall your industrial cities |
| Defend | Fight | Destroy N enemy units inside your own borders. | An army kept at home instead of in the field; it pays only if a rival attacks you | Don't attack them |

### Eras 2 to 7

| Era | Build (two options) | Expand (two options) | Fight (two options) |
|---|---|---|---|
| **2 Classical** | **Monument:** the Triumphal Arch, 240 Production, Production only. **Industry:** 3 Forges running (Iron). | **Frontier:** gain 2 cities within 5 hexes of another empire's city. **Contested site:** gain a city that owns a natural wonder. | **Topple:** take a city from the VP leader. **Raid industry:** capture or raze a city running a Forge. |
| **3 Medieval** | **Great city:** one city running a Forge and a University (Iron, Copper). **Peace:** sign peace with 1 empire you were at war with, then 20 turns at Peace with everyone. | **Far city:** gain a city at least 12 hexes from your capital. **Hold:** hold 2 cities you captured for 20 turns. | **Siege:** capture a city with a Castle. **Defend:** destroy 8 enemy units inside your borders. |
| **4 Renaissance** | **Monument:** the Cathedral, 500 Gold, bought with Gold only. **Industry:** 3 Banks running (Silver). | **Frontier:** gain a city within 6 hexes of a rival's capital. **Contested site:** gain a city that owns a Silver deposit. | **Topple:** take 2 cities from empires ahead of you on VP. **Raid industry:** capture or raze a city running a Bank. |
| **5 Industrial** | **Great city:** one city running a Forge, a Bank and a Factory (Iron, Silver, Iron + Coal). **Peace:** sign peace with 2 empires you were at war with, then 20 turns at Peace with everyone. | **Far city:** gain 2 cities at least 10 hexes from your capital. **Hold:** hold a captured city running a Forge or better for 20 turns. | **Siege:** capture a city with a Star Fort. **Defend:** destroy 10 enemy units inside your borders. |
| **6 Modern** | **Monument:** the World's Fair, 600 Production, in a city running a Factory. **Industry:** 3 Research Labs running (Copper + Coal). | **Frontier:** gain a city within 4 hexes of a city of the VP leader. **Contested site:** gain a city that owns a Coal deposit a rival was supplying from. | **Topple:** end a rival's victory hold by taking one of its cities, or take 2 cities from the VP leader. **Raid industry:** capture or raze a city running a Factory. |
| **7 Future** | **Great city:** one city running a Factory, a Bank and a Research Lab. **Peace:** sign peace with 2 empires you were at war with, then 25 turns at Peace with everyone. | **Far city:** gain 3 cities. **Hold:** hold a captured capital for 20 turns. | **Siege:** capture a rival's capital. **Defend:** destroy 12 enemy units inside your borders. |

Notes on the table:
- **Monument currencies differ by era:** Production you cannot buy (Classical, the rush window, so it competes with Walls and the army), Gold only (Renaissance, the age of patronage, so it competes with buying units and buildings), Production in an industrial city (Modern). The Monument is the cleanest TI4 spend card: it is never met by accident, and its price is the calibration lever (scorers should be 40 to 60% of entrants).
- **Resources:** Industry, Great city, Contested site and Raid industry are the resource ties, one per family. None counts deposits: Industry and Great city count buildings that need supply to run (so they need more deposits than an ordinary empire holds, which pushes expansion or war over deposits, as the 2026-10-19 direction wants), Contested site asks for one city on a scarce site, and Raid industry takes the buildings from a rival. "A Coal deposit a rival was supplying from" (Modern) is the one place a deposit is named, and it is a taking, not a holding.
- **Peace requires wars first,** so a closed Japan (Sakoku: no wars) cannot meet it by its vow, and the goal and the objective never score from one condition. Byzantium's bought peace counts: it is a peace signed with an empire it was at war with, and that is in character.
- **Celts** keep no captured cities (they sack them): Hold is out of reach for them, and Raid industry, Topple and Siege count razing as today. They keep Frontier, Far city and Contested site by founding. No exception (user rule).
- **Mongols** build no Walls and get no Engineers until they capture a Workshop city: Siege is harder for them, Topple and Raid industry are not, and their speed suits Far city and Frontier.
- **Switzerland** leans to the Gold Cathedral, Germany to Industry and Great city, Sparta to Topple and Defend, Athens to the Monument; each faction has an objective in every family it can do in every era.

### Why these replace the current kinds

| Current kind | Fate | Reason |
|---|---|---|
| Population, Territory, Cities (own N) | Removed | Size measures, 8 to 33% by accident, and they feed Wide, which already wins 70% |
| Techs | Removed (already out of every era pool) | Everyone researches |
| Buildings in N cities (Library, Market, Workshop, Walls, Castle, University, Bank, Mill, Forge, Factory, Research Lab, Star Fort) | Replaced by Industry (supply-limited, alternate eras, a different building each time) and Great city | "3 Universities, then 5" is the chain the user named; a higher tier counts as the lower, so one Library line feeds four eras today |
| Kills (destroy N anywhere) | Replaced by Defend (inside your borders) | 28% of all era scores, 30 to 50% by accident; inside your borders they reward the defender, a counter to rushing |
| Captures (N enemy cities) | Folded into Siege, Raid industry and Hold | A conqueror captures anyway; a capture of a particular city is a choice |
| Captures from ahead, from the leader, break a hold | Kept as Topple | A counter-leader choice (pillar 11), but today 16 to 22% (from the leader) and about 50% (from an empire ahead) by accident; so the new pool uses the leader version early and asks for 2 cities in the Renaissance; R1 cuts the stacking. Topple is the kind to watch in the first measurement |

---

## 4. Rules changes, AI work, measurement plan

### Rules changes

- **R1. Progress counts from your last era score.** "Era objectives count only what you gain after entering the era and after your last era score." This is what "one event scores at most one objective" means in practice, and it replaces every multi-era double count today: a capture, a city, a building or a kill that scored one era starts nothing for another, and an open old era needs fresh work. (Implementation: the `EraRecord.base` snapshot that Fight objectives already use is extended to every counter and re-taken for all open eras when any era scores; positional objectives such as Industry count buildings completed after the counting start.)
- **R2. At most one era scored per turn,** the oldest first (TI4's one public per status phase). With R1 a second era could not score from the same turn's events anyway; this makes it explicit for the UI and the event log.
- **R3. Pool rules:** no kind in two consecutive eras; a recurrence changes what is asked; no size measures. A unit test checks the first two over `era_pool`.
- **R4. One new production item, the era Monument** (`ProductionItem::Project`, like Germany's projects): one per era per empire, available while that era is open and unscored, its progress public in the race panel; completing it scores the era. The Cathedral is Gold-only (a purchase order), the others Production-only (like Walls, user 2026-10-18).
- **R5. New counters:** wars ended by treaty and the Peace streak (from the diplomacy events), kills inside your borders (the Battle event's tile and the owner), captures of cities by wall tier and by running buildings (at capture), cities gained and their distance to your capital and to rival capitals and leader cities.
- **R6. Revisit `objective-scale=150`.** It was a pacing lever on the old size measures. The new counts are set directly, so the scale knob should apply only to N in Industry, Far city, Hold and Defend, or retire; pacing is re-measured (median game about 300 turns, as now).
- **Dependencies:** Industry, Great city, the Silver and Coal Contested sites and Raid industry need the supply rule and Silver. Until supply lands, those slots fall back to the other option of their family (each era still has an option per family that needs no supply: Monument or Peace, Frontier/Far city/Hold, Topple/Siege/Defend).

**Skateboard order:** (1) R1 and R2 on the current pool, measured (it alone removes the double counts and most "met at entry"); (2) the non-resource kinds (Monument, Peace, Frontier, Far city, natural-wonder site, Hold, Siege, Topple, Defend) with AI support, measured; (3) the resource kinds as supply and Silver land.

### AI work

- **A real "objectives off" switch.** Today `objective_bonus_percent=0` and `objective_horizon=10` leave a leak (`objective_capture_value` uses `weight.max(1)`, and `objective_item_points` still counts), so the accident cohort in section 1 is a proxy. Add one gene, `objective_seek` (0 = never pursue), read by every objective path, for measurement.
- **Effort and pursuit per kind** (`ai::crowns::objective_effort`, `objective_item_bonus`): Monument as a priced VP like the Great Work valuation (VP against Production and the risk to the city); Industry and Great city through the supply planner (want deposits, rank deposit cities as campaign targets); Peace through `ai::diplomacy` (offer peace, refuse new wars for the streak, weighed against the war's expected gains); Frontier, Far city and Contested site as settle-site and campaign-target bonuses; Hold as garrison priority for captured cities; Siege, Raid industry and Topple as campaign-target value; Defend needs no pursuit (it rewards a defensive stance the AI already takes).
- **Denial:** rivals read the public progress and act: besiege a city whose Monument is near done, declare war on an empire 5 turns from its Peace, retake a forward city. This is the counter half of every objective, and it is what makes them trade-offs for the AI too.

### Measurement plan (cloud box, Standard, 8 empires, paired seeds)

| Measure | Today (section 1) | Target |
|---|---|---|
| Met by accident: share of entrants meeting each objective with objective seeking off | 0 to 56%; median objective 16%; 26 of 50 above 15% | under 15% for every objective; under 10% averaged over the pool |
| Pursuit gap: met with seeking on minus met with seeking off | about 0 for every objective | at least 25 points for every objective (it is something you choose) |
| Already met the turn the era is entered | about 1% | 0% (R1 makes it impossible) |
| Era scores in a turn when the same empire scores another era | 33% | 0% (R2) |
| Entrants scoring each era (seeking on) | about 49% pooled | 40 to 60% pooled, falling from about 60% (era 2) to 35% (era 7), as calibrated now |
| Share of all era scores by one kind | Kills 28%; Fight family 72% | no kind above 25%; each family between 20 and 45% |
| Strategic diversity (pillar 10) | Wide wins 70%; winner entropy 0.46 (guardrail 0.80) | the winning strategies stay varied, no archetype beats all others; winners' scored kinds vary by faction and map |
| Pacing and pillar 11 | median about 308 turns; T100 leader wins 43% | median 280 to 330; T100 leader wins at most 45% |
| Per faction | — | every faction scores from at least two families; best-to-worst era-score share at most 1.5x |

Tools: a tape replay like the one in section 1 (to be committed with the work) evaluates every pool objective for every entrant each turn; the sim report already carries era scores per faction.

---

## 5. Questions for the user

1. **Should progress count from your last era score (R1), so an old open era needs fresh work?** Today an empire that skipped the Medieval objective can still score it later from what it does anyway; under R1 it scores it only from what it does after its last score. *Recommendation: yes.* It is the one-sentence form of "one event scores at most one objective", it removes the double counts measured in section 1, and unscored eras still stay open, so catching up stays possible, just never free.

2. **Is a Monument (a city project that does nothing but score the era) the right "spend" objective, given the Great Work already exists?** It is the clearest TI4 spend card and is never met by accident, but it is a second "build a thing for VP". *Recommendation: yes, in three eras with three different currencies* (Production in the Classical rush window, Gold in the Renaissance, Production in an industrial city in the Modern era), priced so 40 to 60% of entrants score it; the Great Work stays the 2 VP prize.

3. **Peace as a Build objective: should a peaceful path score VP at all, and is "make peace with empires you were at war with, then 20 turns of peace" the right shape?** It gives the diplomatic archetype (pillar 10) a VP and makes Peace and Fight exclusive in the same era; requiring wars first stops a closed Japan from scoring it by its vow. *Recommendation: yes, as proposed.* The counter is cheap (declare war), so it may be hard to score; the sims will say whether it needs to be 15 turns.

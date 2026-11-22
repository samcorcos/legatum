# Switzerland, 8.9.30 build: the Mongol war decided it by T65

Coach review of your Swiss game on the build after 8.9.29 (engine `6f594cfb2`, tagged `playtest-8.9.30`; rules version 25; six empires on the Standard 48×32 map: you as Switzerland against Hard India, Germany, the Aztecs, the Mongols and the Celts; seed 4199048207413126). **The game is over: you won on turn 140 by holding 11 of 10 VP for 10 turns (hold from T130), 11 VP to India's 4.** The save says `finished: true`, `winner: 0`, outcome `vp`.

**Evidence.** Save copy `~/LegatumRuns/bug-saves/swiss-win-2026-11-11.legatum` (the live save was not touched); scripts and raw output in `~/LegatumRuns/coach-tools/coach-8929-swiss/`. The tape replays at the tag with 0 hash mismatches and every AI order reproduced (626 of 626 AI seat-turns). Every sim and replay for this review ran on the cloud box at 4 threads. Play time: **34 active minutes in one session** (0.1 minutes a turn in the Ancient era, 0.2 in the Classical, 0.3 from the Medieval era on).

Chart: `2026-11-11-switzerland-review.png` (VP and army strength, T1–T139, portrait).

The two incidents already being fixed (the Bukhara bounce on T67 and the Lucerne standoff between rival attackers, T128–T130) are not reviewed again here.

## Coach's verdict

"Won easily" is right, and the easy part has a date: **T51 to T65, when you took the Mongols apart.** Hard playing your seat wins 1 of 24 from T2, 0 of 24 from T40, 1 of 24 from T50, then 6 of 24 from T55, 17 from T60 and 24 of 24 from T65. In the takeovers before your war, the most frequent winner is not India but **the Mongols (15 to 16 of 24 from T30 to T45)**: the strongest AI on this map, and you removed it in 19 turns.

**What you did that Hard doesn't.** You picked the Mongols while their army was in the south fighting the Aztecs at Tenochtitlan, went from 2 military units at T44 to 5 at T50 and parked four Spearmen and an Archer two and three tiles from Shangdu at Peace, refused the Aztec and Celtic Alliances against the same target (T48, T50), declared alone on T51, upgraded six Spearmen to Phalangites for 360 Gold the turn after the first capture (T54–T55), and did not stop: Shangdu T53, Almaliq T59, the capital Khanbaliq T67, Bukhara T69, the Mongols eliminated T70. That war was worth 4 of your VP (Strongest Army, Most Advanced, "Capture 2 enemy cities" and the captured capital) and four cities. Hard in your seat plans the same war (its campaign on Shangdu starts T48, three turns before yours), but between captures its plan falls back to "Peace with the Mongols" at value 0 (T54–T56, T60, T64–T68) because the next campaign is blocked while the army "is holding a captured city". The event-window diff values your army orders T48–T52 at **+33 percentage points** of win chance over Hard's.

**What the AIs never did.** The Mongols watched your army build up at their border for seven turns at Peace (threat score 34 to 41, "army 40–80 near us", never their top threat) and kept building a Settler in Shangdu, the Pyramids in Almaliq and a Cataphract in Khanbaliq, with no Walls in any of their five cities. After you declared, their plan was "Peace with Switzerland" at value 0 for eight turns while they lost two cities. Then four AIs went to War with you (Aztecs T65, Celts T66, India T72, Germany T93: the earliest full coalition in any of your games) and in 47 to 74 turns of War they hit your cities 20 times: 16 on Lucerne, a population-1 frontier town, 2 on Khanbaliq and 2 on Lausanne. None of your core cities was ever attacked.

**The hold was breakable for three turns, and nobody aimed at the right city.** Your Reserve stood 151 Gold above 5,000 at T130. Capturing almost any Swiss city of population 3 or more on T130–T132 would have dropped the treasury under 5,000 through the capture bounty, cost 2 VP and ended the hold; Bern (walled, no garrison) had Aztec units within 3 tiles. All three rival operations that week went to Lucerne instead, whose capture could not break anything (its bounty was 61 Gold), and two of them priced it as "break the hold" (Aztecs +525, Celts +1,785).

**Was it fair?** Skill the AI can learn, on a map where the start was poor for Switzerland (Hard from T2 wins 1 of 24). No exploit or rules hole decided it.

**Smarter AI or weaker leader:** about 0% rubber bands (decadence is off by default and there is no crown upkeep; your techs saved each AI 217–304 Science through diffusion, 5–9% of its research bill, and you got 234, 4% of yours). All of it was decisions.

## 1. Result and margin

| | You | India | Germany | Aztecs | Mongols | Celts |
|---|---|---|---|---|---|---|
| Cities T50 / T100 / T139 | 4 / **12** / 12 | 4 / 9 / 11 | 4 / 6 / 8 | 2 / 3 / 4 | 5 / out T70 | 3 / 7 / 8 |
| VP T50 / T100 / T139 | 0 / **7** / **11** | 0 / 3 / 4 | 1 / 1 / 1 | 0 / 2 / 2 | 2 / – / 1 | 0 / 1 / 3 |
| Military units T50 / T100 / T130 | 5 / 6 / 7 | 3 / 6 / 7 | 1 / 6 / 4 | 2 / 5 / 6 | 4 / – / – | 1 / 7 / 9 |
| Treasury T50 / T100 / T139 | 550 / 1,712 / 6,956 | 123 / 106 / 52 | 88 / 265 / 50 | 56 / 119 / 73 | 28 | 203 / 214 / 61 |

- **Your 11 VP:** Largest Economy, Strongest Army and Most Advanced (+3); the Pyramids in Biel (+2, built with Production T54–T63); the Swiss Reserve (+2, from T130); Khanbaliq, the Mongol capital (+1, T67); three era objectives ("Capture 2 enemy cities" T59, Mills in 2 more cities T91, Universities in 4 more cities T123).
- **The VP race:** 0–2 until T52, 3 at T60, **7 at T70**, then 6 to 8 for fifty turns, 9 at T125, 11 at T130. No AI passed 5 all game.
- **When it became clear:** for a Hard player in your seat, T55–T65 (6, 17, then 24 of 24). On VP, at T70 (7 to 1). Pillar 11 ("nobody should know the winner by T100") failed by about 35 turns.
- **The finish:** India took Strongest Army on T133 and again T135–T138 (10 VP each time; the hold only needs 10). You had queued Tanks at T129 and took the crown back on T139.

## 2. Turning points

1. **T24: a Scout takes a Celtic Settler.** The Celtic Settler left Lugdunum with its Spearman two tiles away; your Scout stood next to it. You declared War, took it and founded Aarau with it on T31 (your third city), then accepted the Celtic Peace on T40. The five-turn window around it is worth +0.4 points [−8.9, +9.7]: unlike India 8.9.25 and Celts 8.9.26, this snipe did not decide the game. The Scout was killed the next turn.
2. **T40–T51: the staging.** Four Spearmen and an Archer (two of them new from Thun and Aarau), parked two and three tiles from Shangdu at Peace. The Mongols had the Strongest Army crown and 2 VP, but their army was at Tenochtitlan (taken from the Aztecs T47). Hard in your seat from T50 wins 1 of 24 (the Mongols 7, India 13).
3. **T51–T70: the Mongol war.** 48 attacks (25 on cities), 12 kills, 1 unit lost. Shangdu fell after 7 hits in two turns (T52–T53); the six Phalangite upgrades came the next turn; Almaliq fell to your Archer's fire plus a melee finish (T56–T59); Khanbaliq and Bukhara were besieged together (T63–T69). VP 2 → 7. **Hard in your seat: 1/24 at T50, 6/24 at T55, 17/24 at T60, 24/24 at T65.** Window diff: your army orders T48–T52 +33.4 points [+20.7, +46.0] and +1.4 VP over Hard's; the Khanbaliq window T63–T67 +11.8 points [+6.5, +17.0]; Bukhara T65–T69 +4.3 [+1.9, +6.7]. One window went Hard's way: the siege moves T59–T63, −14.3 [−23.2, −5.3].
4. **T65–T93: the coalition that never reached your core.** The Aztecs declared T65 (a "second front" while you were busy with the Mongols), the Celts joined on T66 through their Aztec Alliance, India on T72 through a coalition Alliance with the Celts (signed T71), Germany on T93 the day after its own coalition Alliance. Two of them came close once: the Aztecs hit Khanbaliq twice (T73–T74, 120 → 37 HP, no Walls), then 9 to 11 Aztec and Celtic units stood 3 to 6 tiles from it for twelve turns with no attack until its Walls were done (T80).
5. **T107–T131: Lucerne, the buffer.** You founded Lucerne forward on T88; Germany took it on T107 (bounty 73 Gold) and razed it on its own order on T114, offering you Peace the same turn (refused); you founded it again on T119 on the same tile; the Celts burned it on T131. Seven of the coalition's thirteen campaigns targeted Lucerne. The one-turn diff values your T119 refounding over Hard's Settler route at +41.7 points [+21.5, +61.8] by T140 (24 of 24 wins against 14 of 24), a measured hint that the decoy worked; the mechanism is not traced.

## 3. Your play against Hard's

| | You | Hard in your seat from T2 (24 runs) | Hard from T50 |
|---|---|---|---|
| Cities T100 / T150 | 12 / 12 (T139) | 5.8 / 5.9 | 7.3 / 7.1 |
| Military units T100 / T150 | 6 / 7 (T139) | 9.1 / 14.8 | 9.9 / 13.9 |
| VP T100 / T150 | 7 / 11 (T139) | 2.0 / 3.4 | 3.2 / 4.5 |
| Wins | T140 | 1 of 24 | 1 of 24 |

- **Expansion:** 13 Settlers trained plus one captured, 4 cities taken. 4 cities at T50 (the best AI 5), 8 at T70, 12 at T100.
- **Army:** two units until T44, 5 to 8 for the Mongol war, then 6 to 7 through four wars. Walls in 10 of 12 cities from T115. Where you built a Market, Mill, Library, University, Factory or Settler, Hard in your seat builds a Spearman, Phalangite, Crossbowman, Rifleman or Cavalry: across the game its city choices differ from yours 244 times, 183 of them a military unit against your 51.
- **Gold:** one purchase all game (Roadworks in Shangdu, T59, 245 Gold) and 15 upgrades for 1,554 Gold, six of them at the gate (T54–T55, 360 Gold). Hard in your seat would buy 50 items, mostly Quarries, Farms and Phalangites, and runs 230 Gold lower a turn on average. Your treasury: 1,712 at T100, 5,151 at T130, 6,956 at the end; interest about 60 Gold a turn at the end.
- **Diplomacy:** you refused the two Alliances aimed at your own target (Aztecs T48, Celts T50) and fought alone; refused the Mongols' Peace twice (T59, T69: Hard would refuse too, "peace would be broken") and Germany's (T114). You never signed with a coalition member.
- **Your answers from 2026-11-11 match the tape:** buffer cities with Walls (Lucerne, refounded), units near enough to rush in, a reserve spent almost only on units (upgrades), and a second front only against a walkover (you fought nobody else while the Mongols lasted).

## 4. Every AI, strategically

| AI | Opening to T50 | T50–T100 | T100–T130 | T130–end | Strategy and grade |
|---|---|---|---|---|---|
| **Mongols** | 5 cities, Strongest Army from T27, takes Tenochtitlan T47 (campaign valued 971, "supply" 792) | Plan "Peace with the Aztecs" from T48 while at War; ignores your army 2 tiles away for 7 turns (threat 34–41); keeps building a Settler, the Pyramids and Cataphracts; no Walls in 5 cities until T60; "Peace with Switzerland" at value 0 T52–T59 while losing Shangdu and Almaliq; offers Peace T58, T68; eliminated T70 | – | – | Conquer the weak neighbour, then none. B opening, F defence. **The AI most likely to win this map** (it wins 15–16 of 24 Hard-seat continuations from T30–T45) |
| **India** | 4 cities, economy | Joins the coalition T72 by Alliance; plan "Peace with Switzerland" while at War T72–T81; campaign on Lucerne T90–T108 (4 hits) | Plan flips Peace and Contain six times T113–T121 (−161 to +5); strike on Lucerne T127–T132 (2 arrived, 0 attacks) | Strongest Army T133–T138 (10 units, army 1,348–1,420) | Wide then economy, a dead war. C. **Closest on VP (4–5)** and the only one to take a crown during your hold |
| **Celts** | 3 cities, economy | Joins on T66 through the Aztec Alliance; builds the coalition (India T71, Germany T92 after four refusals); Khanbaliq strike T73–T88 (8 units, 2 arrived, 0 attacks); Bern campaign T89–T101 `insufficient_army` | Lucerne T103–T104 (`negative_value`), Aarau T122–T124 (`insufficient_army`) | Burns Lucerne T131 (+1 VP objective, "break the hold" +1,785) | Defend and counterattack, the only AI that fought: 43 attacks on you, 12 city hits, 5 of your units killed for 11 lost. C+ |
| **Germany** | 4 cities, 1 military unit at T50 | Economy to T92; declares T93 (coalition Alliance, "busy elsewhere") | Takes Lucerne T107, razes it T114 and offers Peace the same turn; Lausanne campaign valued −33 T116–T132 (2 hits) | Contain, nothing | Wide then economy; its one capture thrown away. C− |
| **Aztecs** | 2 cities, lose Tenochtitlan T47 | Declare on you T65 as a second front at plan value −257 while their war with the Mongols runs; retake and lose Tenochtitlan (T64, T67); Khanbaliq campaign T77–T92 ends `walls_before_arrival` (2 hits); Great Library T88 | – | "Break the hold" campaign on Lucerne T131 | None; a war declared for nothing. D |

- **The runaway:** every AI named you top threat on T55 (score 90–100, "captured 1 recently"), two turns after Shangdu, and 250+ by T72. That read is the best yet. What it caused: four declarations and two coalition Alliances, and 20 city hits in 74 turns, none on a city that mattered.
- **Army follows intent:** none of the four had more than 10 military units while at War with you; Germany had 1 at T50 and 4–6 at War. Wars without armies, again.
- **Dead wars:** India (plan "Peace" while at War T72–T81, and six plan flips T113–T121), the Aztecs (2 attacks in 74 turns), Germany (7 attacks in 46 turns).

## 5. Battles

- **Exchange, whole game:** you killed 40 units and lost 10 (3 of them Scouts). Mongols 12 killed for 1 Swiss loss; Celts 11 for 5; Germany 9 for 2; Aztecs 5 for 0; India 3 for 0.
- **Your sieges:** Shangdu (pop 5, no Walls, no garrison) in two turns with four Spearmen; Almaliq with the Archer's fire then a melee hit; Khanbaliq and Bukhara in parallel.
- **The AIs' sieges:** Khanbaliq T73–T79 is the clearest miss: unwalled, 37 HP on T74 with two Aztec Phalangites adjacent, then up to 11 Aztec and Celtic units within 6 tiles for twelve turns and no attack after T74 (they stood at 3 tiles, the siege stance's staging ring, the same picture as St. Gallen in 8.9.29). Lucerne: 16 hits for one capture.
- **Air:** none on either side.

## 6. The Swiss rules in this game

- **Triple upkeep:** 6–7 military units cost you 12–15 Gold a turn; it did not stop you fighting.
- **Interest and the Reserve:** the hoard grew from 1,712 (T100) to 6,956; the Reserve was the closer (T130) and, for three turns, the weak point (§7).
- **Redoubt (Walls) not buyable:** you built Walls first in 10 of 12 cities.
- **Hard Switzerland:** the AI-only census has Swiss at 6.84% of Swiss seats (rules 19 control) and 7.87% with `swiss_playbook` 3, against a fair 12.5% (`docs/sim-results/ai-quality/swiss-gap-1107/README.md`). The playbook below (§9) is what this game adds.

## 7. What would most likely have beaten you

Measured from the rival seats' own views (decision diff `--rival`, 16 paired runs each, every rival continuing as Hard and your seat on your recorded orders) and from the takeover curve:

- **The Mongols, T42–T48:** their best alternatives were all in Khanbaliq: a Phalangite instead of the Cataphract on T48 cuts your estimated win chance by 10.9 points [−2.5, +24.2] and your VP by 1.8 [0.6, 3.1], and raises the Mongols' own chance by 27.7 points [13.6, 41.7]; buying a Quarry there on T44 (Production toward defenders) is similar. Your recorded orders drift in 31 of 32 of these runs, so read them as upper bounds. The generator offered nothing for Shangdu, the city you took first.
- **The Celts, T122–T132:** none of their generated alternatives (Quarry and Farm buys, two attack moves, an Arquebusier) moved your chance at all. The generator does not offer a strike on Bern or Khanbaliq, the captures that would have broken your Reserve.
- **Hard in your seat (takeover curve, 24 runs a point, every rival Hard):** 1/24 from T2, 0 from T10, 1 from T20, 0 from T30–T45, 1 from T50, 6 from T55, 17 from T60, 24 of 24 from T65 on. From T30–T45 the Mongols win 15–16 of 24.

From the evidence, ranked by how likely each was to break the win:

1. **The Mongols defending at their door (T44–T51).** Walls or two defenders in Shangdu instead of the Settler, and the army called home from Tenochtitlan. Your swing is T50–T65; without the first two captures you are back near Hard's 1 in 24. Learnable: your units were visible at their border for seven turns.
2. **Any capture of a real Swiss city on T130–T132.** Bern on T130–T131 (walled, no garrison, Aztec units within 3 tiles), Khanbaliq on T130–T132, or any city of population 3 or more on T130 would have pushed the Reserve under 5,000. Learnable: the treasury and every bounty are public.
3. **The coalition hitting Khanbaliq at 37 HP (T74–T79).** It held your captured-capital VP and its Walls were six turns away.

## 8. Decision diff

Full report: `docs/playtests/diffs/swiss-2026-11-11-8.9.30-decision-diff.md` (20 valued turns, 24 paired salts, horizon T140); event windows: `swiss-2026-11-11-8.9.30-decision-diff-windows.md`; rivals: `swiss-2026-11-11-8.9.30-decision-diff-rivals.md`.

- **Top disagreements:** the T119 refounding of Lucerne (+41.7 points, see §2); purchases at T76/T77 (Hard buys a Phalangite in Aarau; ±12.5 points, opposite signs, noise); Hard would have declared on the Mongols at T48 (−0.5 VP for your three-turn wait, inside noise).
- **Classification:** the T48–T52 and T63–T69 army windows are an **execution and follow-through gap** (`plan.rs` campaign choice after a capture, `military.rs::campaign_against` "our army is holding a captured city"); the Lucerne refounding is **him exploiting an AI weakness** (campaign target choice: `plan.rs` campaign pricing, "break the hold"); the economy-over-troops production gap is a **valuation error** in Hard's war quota (`production.rs::desired_army` while at War with distant enemies behind Walls).
- **Hard across the steepest rise (T55 → T60, +45.8 points; one traced run from each end, `takeover_gap.py`):** its most frequent blocks were city production left running without a comparison (17 seat-turns), Khanbaliq "not currently visible" for a capture (10), decisive buys blocked for lack of Gold (10; Hard had spent it on improvements), campaign targets dropped as "stale city sighting" (12) or "funded forecast unavailable" (12). Gates are counted, not proven causes.
- **Cross-game patterns:** strengthens "army routes and targets" (+33.4 and +11.8 points here), "Hard buys; he keeps Gold" (50 against 1), and "the coalition targets the smallest city"; weakens "he declares war; Hard waits" (Hard would have declared first) and the Settler snipe (no measurable value here).

**Closure since the last review (8.9.29, 2026-11-08):** the AI shipped with this build includes the siege stance and calibrated forecast, threatened-city defence and the standing coalition bloc. The coalition did form (earliest yet), so that part of "Leaders aren't checked" is closed in effect; the stance's staging ring still left Khanbaliq untouched at 37 HP (T74–T79), as St. Gallen in 8.9.29; threatened-city defence could not act for the Mongols before your declaration (an army at Peace is not counted, rec 1), and after it they had 22 to 53 Gold to buy with. The decisive-turn fixtures were not rerun (the closure check runs a local cargo test; the ledger keeps its 11-08 column).

## 9. Your Swiss playbook for Hard

| Part of his plan | What he did | Does Hard Switzerland do it? |
|---|---|---|
| Opening | Spearman, Walls in Thun at T9, two Scouts on Explore, Settlers from T19 | Partly: `swiss_playbook` 3 (`params.rs`) starts the Settler chain earlier |
| Opportunistic war | The busy, unwalled neighbour with the strongest army, staged at Peace, declared alone | Partly: plans the same campaign (T48) but drops to Peace between captures (`plan.rs`, `military.rs::campaign_against`) |
| Upgrades at the gate | Six upgrades the turn after the first capture | No: Hard's upgrade budget in your seat was 60–72 Gold a turn (`production.rs` upgrade reserve) |
| Troops | 6–7 units for 12 walled cities while at War with four empires | No: Hard keeps 9–15 and builds units where you built Markets and Mills (`production.rs::desired_army`) |
| Gold | One purchase, 15 upgrades, hoard to the Reserve | No: about 50 purchases in your seat, mostly tile improvements (`production.rs::plan_buy`) |
| Buffer town | A walled frontier town, refounded after it fell | No such valuation (`civilians.rs::choose_site_scored`) |
| Closer | Tanks queued at T129 for Strongest Army | Partly: `decisive_buy` exists, but not "build the crown" |

**Recommendation:** a `swiss_playbook` level 4 that adds (a) follow-through on a walkover war (keep campaigning while the target has no army at home), (b) Gold for upgrades at the gate instead of improvements, (c) the troop count of a walled empire; measured by the Swiss AI-only win share on vs off (both census seeds) and Hard in his seat from T2/T40/T50 on this map.

## 10. Recommendations for the AI

Ranked by expected effect on how often you win on Hard, with any faction:

1. **Defend at the door, even at Peace** (Mongols here; Aztecs and Switzerland in 8.9.29, Sparta's victims in 8.9.26). When a neighbour's visible military near an unwalled city exceeds its defence, put Walls or a defender before a Settler or wonder there, call units home, and read the build-up as a threat before the declaration. The cause found here: `analysis.rs::Ctx::build` marks only empires at War as hostile, so `threat_units`, `door_defense` and the production defence passes cannot see an army at Peace; only the threat read does ("army 40–80 near us" for seven turns). Feed visible non-allied military within two turns' march with a rising trend into the same exposure test (public units only, no cheating). Code: `analysis.rs`, `door.rs`, `threat.rs::exposure_to`, `production.rs::choose_item`. Gates: your tape seat from T45 on this game (does Shangdu survive past T53?), the war suite, the human-seat benchmark (rivals-only). The Lab can't measure it: Hard in your seat never takes the cities (§11).
2. **Price "break the hold" on the city that actually breaks it.** The term fired for pop-1 Lucerne (+525, +1,785) whose bounty was 61 Gold, and nobody priced Bern (T130–T131) or Khanbaliq (T130–T132), whose capture would have broken the Reserve. Compute the hold break from the public treasury, bounty and VP sources per city (`plan.rs` campaign terms near "break the hold", `victory.rs::hoard_denial_value`, `hold.rs::capture_title_losses_with_model`; check whether `sack_vp_denied`'s crowns are counted again by `winchance.rs::projected_vp`). Gate: this save's T128 state with your recorded orders (does any rival take a Reserve-breaking city?), the `celts-break-the-reserve` card, the human-seat benchmark.
3. **Follow through on a walkover war.** Hard in your seat plans the Shangdu campaign at T48 but returns to "Peace with the Mongols" at value 0 between captures (`military.rs::campaign_against` returns "our army is holding a captured city"; Contain carries a −133 to −274 "retaliation" term). Gate: the takeover curve from T55 (6 of 24 now) and the window T48–T52; AI-only census (captures per war, eliminations).
4. **Coalition sieges that strike at the staging ring.** Khanbaliq T74–T79 (37 HP, no Walls, 9–11 rival units within 6) and St. Gallen in 8.9.29: the siege stance waits at 3 tiles. Code: `city_assault.rs`, the siege stance genes. Gate: a war scenario cut from Khanbaliq T74, the combat lab.
5. **Hard Switzerland's war quota.** At War with four distant enemies behind Walls, Hard builds 183 military units where you built economy; level 4 of `swiss_playbook` (§9).

Design or rule questions for you, not AI fixes: none new. (Germany razing its only captured city and offering Peace the same turn is an AI choice, covered by rec 2's target valuation.)

## 11. Scenarios for you

- **`swiss-strike-the-busy-horde` (The Busy Horde; show this one):** Switzerland at T40, at Peace with the Mongols, four cities, two military units, 313 Gold; the Mongol army is in the south at War with the Aztecs. Take Shangdu and Almaliq within 20 turns. **Hard: 0 of 50** (median score 430): it declares War in 21 of 50 runs (median T48) and takes neither city in any. You took both by T59. The card asks whether a person finds this line, and it is the cleanest test of rec 3.
- **Dropped: `mongols-swiss-at-the-door`** (the Mongols at T45 keeping their cities). Hard Mongols held 50 of 50, because the Lab seats Hard in the Swiss seat and Hard Switzerland never takes the cities (the card above). A defence card needs an attacker that attacks.

## 12. Questions for you

1. T40–T51: did you choose the Mongols because their army was busy with the Aztecs, because their cities had no Walls, or because they were the strongest AI and the likely winner?
2. T48 and T50: why refuse the Aztec and Celtic Alliances against the Mongols when you were about to attack them anyway?
3. T88 and T119: was Lucerne a buffer you expected to lose, and why found it again on the same tile?

# Coach review: "Japan's Century" (Japan from T100 of the 8.9.6 Swiss game, played to T250 on 8.9.7)

Run file: `~/LegatumData/scenarios/japan-from-100-2026-10-22T22-50-35Z.json` (scenario `scenarios/lab/japan-from-100.json` r1, engine `5637d8115` = `playtest-8.9.7`). Replayed on the box at the tag: 150 turns, 0 hash mismatches. Chart: `2026-10-22-japans-century.png`.

Score 1240: +6 VP, 12 cities, all 8 starting cities held. You said: "I was definitely about to win. I just had a couple units to kill, get my 2 VP, then I would win the game." The feedback card was left empty, so that sentence is the free text.

## Coach's verdict

You played Japan better than the AI does, and you were one turn from 10 VP, but not one turn from winning.
- **The "2 VP" was Sakoku.** You closed the country at T217, the 30 turns were done at T247, and six enemy military units in your eastern land were blocking the vow: three German (a Trooper, a Field Cannon, a Ranger) and three Athenian (a Trooper, a Cataphract, a Rifleman), with seven Military Engineers beside them.
- **Reaching 10 is the easy part.** From your exact T250 position, with Hard playing your seat, Japan reached 10 VP on the very next turn in 100 of 100 games. Then the 10-turn hold broke within 1 to 2 turns in 100 of 100, because four empires were at War with you and one enemy unit ending a turn in your land ends the vow. Japan won 14 of 100 of those games in the end, a median 70 turns later, after three to five tries. Athens won 41 and Switzerland 38.
- **On the same start, you beat Hard by about one VP.** Hard Japan from T100 scores a median 1010 (you 1240, which 10 of 85 runs matched). It has 7 VP at T240 (you 8) and wins 11 of 100 games, never by T250.
- **Your edge was keeping and growing, not fighting:**
  - you lost 2 cities in 150 turns, where Hard Japan loses 7.3;
  - you grew to 12 cities and 109 population, where Hard has 9 and 85;
  - you entered the Modern and Future eras with their objectives already met, scoring each the turn you arrived;
  - you never declared a war.

**On decadence:** it shaped your game rather than stopping it. You paid the highest rate anyone has paid, 44 to 52%, which took 49% of your Production, Gold and Science (14,256 Production). Every rival paid 11% or less, and Hard Japan in the same seat paid 37%. It cost you every crown and kept your army at half Switzerland's. But your VP came from objectives decadence doesn't touch.
- **The two fixes, tested:** "Power without Food" would have helped (Hard Japan wins 41 of 100 from T100 instead of 11). Scaling by VP standing makes Japan worse (2 of 100).
- **My recommendation is to keep the rule.** In balanced 8-empire games "without Food" lifts Japan only from 3.1% to 4.2% and lifts Sparta from 10.4% to 17.6%. Even with no decadence at all, Japan wins just 9.1%. Japan's real problem as a closer is the vow, not the tax.

## 1. Result and margin
- **The window ended at T250 with no winner.** The VP stood at:

  | Empire | VP |
  |---|---|
  | You | 8 |
  | Switzerland (AI) | 7 |
  | Athens | 6 |
  | Mongols, Sparta | 4 each |
  | Byzantium, Germany | 2 each |
  | Celts | gone since T180 |

  Switzerland held Largest Economy, Strongest Army and its Reserve goal.
- **The VP race:**

  | Turn | You | Best rival |
  |---|---|---|
  | T100 | 2 | 1 |
  | T150 | 4 | 3 |
  | T200 | 5 | 6 (Athens) |
  | T225 | 7 | 6 |
  | T250 | 8 | 7 |

  You led most of the way, but never by more than 2. Pillar 11 held: nobody had won by T250.
- **Your VP sources at T250:**
  - all six era objectives: Classical T107, Renaissance and Industrial T166, Modern T194, Future T239 (one each);
  - the Great Work, T202 (2).

  No crown at the end: you held Largest Economy on and off (it changed hands with Switzerland 15 times) and Most Advanced from T136 to T164.
- **Play time:** about 51 minutes of active play for 148 turns (median 14 s a turn; 0.33, 0.40 and 0.31 minutes a turn for T100–149, T150–199 and T200–249). It was three sittings, with an overnight break at T173 and a 31-minute one at T188. `play-time-report.py` can't read a scenario run (`turn_times` is empty), so this comes from the per-turn `ms` in the run file.

## 2. Turning points
| Turn | What happened | Swing |
|---|---|---|
| T107–T120 | You found Kanazawa (9 cities: Classical +1), take Phaleron from Athens (T111), lose Kanazawa to two Phalangites (T112) and retake it (T120) | 10 cities, kept from here on |
| T166 | 144 tiles of territory meets the Renaissance and the Industrial objectives in one turn | +2 VP. The AI Japan of the real game sat at 141 of 143 tiles at T199 |
| T171–T193 | You take the Athenian capital (20 HP left) with a Rifleman and a Musketman, and accept Athens's peace at T178. Switzerland (T184) and Athens (T187) declare, and Athens retakes its capital at T193 | Athens, the usual winner (35 of 100 Hard runs), is held back: 6 VP at T250 |
| T194, T202 | You enter the Modern era with 93 population (Modern +1 at once), then build the Great Work | 5 → 7 VP |
| T211–T217 | You found Hiroshima and Himeji (one bought Settler, 12 cities), then close the country | The Future objective (12 cities) scores the turn you enter the era, T239: 8 VP |
| T232–T250 | Switzerland declares again (T232). German and Athenian units with Engineers stand in your land when the vow's 30 turns end (T247) | The vow is blocked: 8 VP, not 10 |

## 3. Your Japan against the AI's Japan
**(a) The AI Japan of the real 8.9.6 game, from the same T100 position**

| | You (T100–T250) | AI Japan, real game (T100–T245) |
|---|---|---|
| Cities | 8 → 12. Founded 3, captured 3 (Phaleron, Kanazawa back, the Athenian capital), lost 2 | 8 → 10. Founded 4, captured 1 (Laurion), lost 3 |
| Wars | Declared none. 10 peace offers, 4 peaces | 6 campaigns on Athens, most ended `insufficient_army` |
| Sakoku | Closed T217, countdown done T247, blocked by intruders | Closed T165, scored T198–T201, lost T202, reopened T205 |
| Great Work | T202 | T159 |
| Era VP after T100 | 5 (Classical, Renaissance, Industrial, Modern, Future) | 3 (Classical T126, Renaissance T195, Modern T196) |
| Peak VP | 8 (T240–T250) | 9 (T199–T202), then 7 |
| Decadence T100 / T150 / T200 / T245 | 47 / 48 / 50 / 50% | 48 / 41 / 35 / 35% |
| Army (crown metric) at T200 / T240 | 126 / 383 | 398 / 830 |

**(b) Hard as Japan from the same start** (100 runs on your engine, dice reseeded per run, played to the end of the game; `~/LegatumRuns/coach-tools/coach-japan/`)

| | You | Hard Japan (median, or runs) |
|---|---|---|
| Score at T250 | 1240 | 1010 (p10 750, p90 1240, max 1340; 10 of 85 runs at 1240 or more) |
| VP at T150 / T200 / T240 | 4 / 5 / 8 | 4 / 6 / 7 |
| Cities at T240, population, Food | 12, 109, 343 | 9, 85, 227 |
| Cities lost T100–T250 | 2 | 7.3 (all 8 starting cities kept in 32 of 100) |
| Wars declared | 0 | 0.95 (63 of 100 runs; first at a median T143). Runs that declared have 6 VP at T250, runs that didn't have 7 |
| Wars declared on Japan | 4 | 5.4 |
| Era entered: Industrial / Modern / Future | T145 / T194 / T239 | T155 / T201 / T240 |
| Era scored on the turn of entry | Modern and Future both | Modern 13 runs, Future 8 |
| Great Work | T202 | T137 (98 runs) |
| Sakoku closed | T217 | 88 runs, median T179. Scored by T250 in 74, held at T250 in 22 |
| Decadence at T150 / T200 / T240 | 48 / 50 / 50% | 42 / 35 / 27% |
| Game won | (window ended) | 11 of 100, median T310, never by T250. Athens 35, the Mongols 34, Switzerland 16 |

**What you did that the AI doesn't:**
1. **You kept every city that mattered.**
   - Walls stood in 9 of your 12 cities at T250 (4 built in the window), plus 2 Castles.
   - A small army was kept current: 19 unit upgrades, Artillery and Field Cannons, and 38 ranged attacks.
   - You lost 2 cities in 150 turns. Hard Japan trades cities back and forth (5.0 taken, 7.3 lost).
   - In Japan's network each city adds 7% Food and Production to its neighbours, so a lost city costs more than its own yields.
2. **You grew, and decadence doesn't cut Food.**
   - Your Food went from 189 to 347, where Hard Japan's stays flat at about 230.
   - You built Rice Storehouses, Granaries and Hydroponics, and founded two late cities with a bought Settler.
   - Decadence only slows growth, so Food was the part of your economy it barely touched. That size then paid twice: Modern (89 population) and Future (12 cities) scored the turn you entered each era. Your research was only 7 to 10 turns ahead of Hard's.
3. **One territory count, two eras.** At T166, 144 tiles met both the Renaissance and the Industrial objectives (`EraObjectiveScored` for eras 4 and 5 in the same turn). In the real game the AI Japan never got there.
4. **You fought only the war you were dealt.**
   - You declared nothing and offered peace 10 times.
   - You struck the strongest rival when it was weak: Athens's capital at 20 HP.
   - You captured 7 Settlers. Hard Japan also captures about 5, so that one isn't an edge.

**Where the AI was better:**
- **The Great Work came 65 turns earlier** (T137 against your T202).
- **The vow came earlier too.** Hard closes at a median T179, and 74 of 100 runs score it by T250. Your T217 close was late: closing before about T207 would have had the vow's 30 turns done by your T239 Future score. You declared no war all game, so a closed country cost you almost nothing but tech diffusion.

## 4. Was he really about to win?
- **Where it stood at T250:**
  - you had 8 VP and 33 turns closed;
  - `goals::sakoku_intruders` listed six enemy military units, the Germans and Athenians above;
  - you were at War with Switzerland, Athens, Byzantium and Germany.
- **What winning needed:** clear the six (+2 VP, which takes you to 10), then end ten turns in a row with no enemy military unit in your land.
- **Continuations from your T250 state** (every seat Hard with fresh memory, 100 runs; fresh AI memory may make the first turn look easier than it was):

  | Your seat | 10 VP after T250 | First hold broken | Japan wins | Others |
  |---|---|---|---|---|
  | Hard | 100/100 | T251–T252 in 100/100 | 14/100 (median T321) | Athens 41, Switzerland 38, Byzantium 6 |
  | Hard, `vow_hold` gene on | 100/100 | T251 in 100/100 | 12/100 | |
  | Hard, bands off | 100/100 (11 VP) | median T257, 96/100 | 30/100 | Athens 34, Switzerland 31 |

- **On master:** the new Scenario Lab cut `japan-finish-the-vow` reads the same. Hard reaches 10 at T251, but by T280 41 of 50 runs are back at 8 VP and lose 1 to 4 cities; none wins.
- **The answer:** you were about to reach 10 VP, which nobody else was near. Winning needed a ten-turn border defence against four armies, while Switzerland sat at 7 VP with twice your army. Hard can't do it. You might, and that is the scenario below.

## 5. The decadence question
**What it cost you:**

| | You | Hard Japan, same seat | AI Japan, real game | Rivals in your run |
|---|---|---|---|---|
| Share of Production, Gold and Science lost, T100–T250 | **49.0%** (14,256 Production, 3,678 Gold, 4,804 Science) | 37.3% | 39% (whole game) | Mongols 11.4%, Athens 5.5%, Swiss 4.8%, Germany 0.9%, others 0 |
| Rate at T250 | 50% | 26% | 35% | |

You paid more than the AI because you grew more: your Power went from 234 to 516.

**Did it stop you? No; it steered you.**
- **What it took:** every crown. Largest Economy is measured on net Gold plus Production, and Switzerland took it back each time. It also kept your army at 380 against Switzerland's 846.
- **What it left:** the objectives (territory, population, cities, era entry) and the Great Work, which decadence doesn't touch. You built a permanent-VP empire and reached the target's doorstep anyway.
- **What it means:** what stood between you and the win was the vow's fragility under four wars. With the bands off from T250, Hard's hold still broke first in 96 of 100.

**The options, measured:**

| Rule | Hard Japan from T100 (100 runs) | Japan's decadence T150 / T200 | Balanced 8-empire games (2,048, Japan's win share) | Side effects in the balanced games |
|---|---|---|---|---|
| (c) Keep (control) | 11 wins, none by T250; score 1010 | 42 / 35% | **3.1%** | Byzantium 29.2%, Celts 2.4% |
| (a) Power without Food | **41 wins** (13 by T250); score **1220** | 29 / 28% | **4.2%** (86 wins against 63; about 2 standard errors) | Sparta 10.4 → **17.6%**, Switzerland 13.8 → 11.3%, Germany 8.1 → 6.2%, Celts 1.9% |
| (b) Half rate unless you lead on VP | **2 wins** | 44 / 35% | 2.8% | Switzerland 15.0%, otherwise unchanged |
| (a) + (b) | 24 wins | 29 / 27% | not run | |
| No decadence (and no crown upkeep) | **100 wins, median T176** | 0 | **9.1%** (decadence only) | Mongols 23.3%, Sparta 15.9%; game 241 turns |

The balanced control reproduced the published 8.9.7 run exactly (63 Japanese wins), so the two new switches change nothing when off.

**My recommendation: (c), keep decadence as it is.**
- **(b) is out.** A Japan that leads on VP pays the full rate while everyone else pays half, so it hurts the leader it was meant to spare: 11 → 2 of 100.
- **(a) is the only option that helps Japan, but it is the wrong size in both directions.**
  - In this position it turns an 11% AI Japan into a 41% one, and gives Hard about your score (1220 against your 1240). Taking Food out is worth about what your skill was worth here.
  - In balanced games it adds 1.1 points to Japan and 7.2 to Sparta, which pillar 8 can't accept as a Japan fix.
- **Decadence isn't why Japan is at 3%.** With no decadence at all, balanced Japan is still at 9.1%, below the 12.5% fair share.
- **The lever for Japan is its vow (a rule question for you).** As the closer, Sakoku breaks to a single unit of any one of the empires at War with it:
  - Hard's first hold broke within 2 turns in every one of 100 runs;
  - Hard Japan closes in 88 of 100 runs but holds the vow at T250 in only 22.

  The faction design already names a fallback: an intruder must stand inside 2 turn ends in a row. A second option would count only intruders within a few tiles of a Japanese city. Either one makes the counter "occupy, don't tag", which is closer to the vow's story.
- **If you still want Food out of Power,** take it as a faction-balance change with Sparta's jump looked at first. It isn't the answer to "should decadence stop Japan".

## 5a. Smarter AI or a weaker leader?
- **AI Japan's results come almost all from the bands:**
  - from T100, Hard Japan wins 11 of 100 with the bands and 100 of 100 without;
  - the decision levers I tried moved nothing: `vow_hold` 11 → 10 wins, `era_entry_percent` at its maximum 11 (no change), and the objective genes (horizon 200, bonus 200) 6. The 8.9.6 review's five forced strategies moved 0–4 of 30.
  - For Japan's non-conversion in this position: **about 90% bands, 10% decisions.**
- **Your own result was mostly your decisions.**
  - You scored 230 points above Hard's median, with a higher tax than Hard paid.
  - From T250, the bands are the smaller part of the last step: switching them off lifts a Hard seat from 14 to 30 of 100, while the vow still breaks first in 96 of 100.
- **No human-seat benchmark arm was run for this scenario review.** The 8.9.7 baseline (`docs/sim-results/hard-vs-human/baseline-8.9.7.md`) stands.

## 5b. What would make the AI play Japan your way
The question this time is not what would have beaten you, but what would make Hard's Japan score like you. It lacks three things no existing gene supplies, and each is learnable from its own view (no cheating):
1. **Defend the cluster instead of trading cities.**
   - **The evidence:** 7.3 cities lost against your 2. Runs that declare a war end a VP lower (6 against 7). Hard keeps all 8 starting cities in 32 of 100.
   - **The change:** in `crates/ai/src/threat.rs::plan_threat_defense` and `exposure_to`, and the Walls and garrison priorities in `production.rs::choose_item`, weigh a Japanese city by its network links (its neighbours' 7% Food and Production), not its own yields. And for Japan, a war it would start needs a forecast that beats losing a linked city.
   - **Measure:** this scenario's baseline (cities lost, starting cities held, score), the war-suite defence scenarios, and the human-seat benchmark seat-only and opponents-only.
2. **Grow into the next era's objective before entering it.**
   - **The evidence:** your Food and population turned Modern and Future into same-turn scores. Hard reaches the Future era at T240, as you did, but with 9 cities; it scores era 7 in 13 of 100.
   - **The change:** `crowns.rs::objective_effort` and `chosen_objective` should rank the next era's Expand objectives (cities, population, tiles) while the current era is still being researched. Production should value Rice Storehouses, Granaries, Hydroponics and late Settlers by the objective they complete, not by yield alone. Decadence slows growth but not Food, so growth is the cheap path for a taxed leader.
   - **Measure:** as above, plus era-7 scoring in all-AI games.
3. **Close the country only with the borders quiet, and as the closer.**
   - **The evidence:** Hard closes at T179 and keeps the vow at T250 in 22 of 100. You closed too late.
   - **The change:** in `goal_sakoku.rs`, value the vow by the turn its 2 VP takes Japan to the target. Price each rival at War with units able to reach Japanese land as a near-certain break (the public `sakoku_denial_value` logic; `vow_denial_percent` is a start). Seek peace before closing, not after.
   - **Measure:** this scenario, `japan-big-empire`, and `japan-finish-the-vow` with Hard in the seat.

## 6. Recommendations (ranked by how much they would make Hard's Japan like yours)
1. **Defend the network** (5b.1). The AI's biggest measured gap: 5 more cities lost than you. Code: `threat.rs`, `production.rs`, `plan.rs` (the Japan war gate).
2. **Grow into era objectives** (5b.2). It closes the 12-against-9 cities and 109-against-85 population gaps that scored your last two eras. Code: `crowns.rs`, `production.rs`, `civilians.rs` (late sites at 3-hex spacing).
3. **A rule question for you: the vow as a closer** (section 5). Make an intruder count only after 2 turn ends in a row, or only within a few tiles of a city, and measure Japan's balanced share and `japan-finish-the-vow`. This is a rule, not an AI change, and it is yours to decide.
4. **The vow's timing for the AI** (5b.3), after the rule question, since its value depends on the answer.
5. **Decadence: keep it** (section 5). If Japan still needs help after the vow, "Power without Food" is the faction-balance lever to look at, with Sparta checked.

## Scenario for you
- **`japan-finish-the-vow`, "Finish the Vow"** (new, put it in front of him first). It starts from your own T250:
  - 12 cities, 8 VP, the vow's 30 turns done;
  - six enemy military units in your land;
  - at War with Switzerland, Athens, Byzantium and Germany.

  The task: clear them, take 10 VP and hold them for 10 turns, scored as VP gained over 30 turns. Hard's baseline (50 runs, master engine) has a median score of 770, a p90 of 840 and no wins. Hard reaches 10 VP at T251 every time and is back at 8 in 41 of 50 by T280. It answers your "I was about to win" directly, and tells us whether a human can hold the vow, which decides recommendation 3.

## Scenario Lab feedback
This is the first lab run since 2026-10-04 and the card's Fun, Fair and text fields are empty. Your words above came by chat.

**Baseline note:**
- **Re-measured on your engine.** The recorded baseline (median 1060) was measured on `72a0fd9bc`, before 8.9.7. On your engine Hard's median is 1010, with p90 1240.
- **compare.py's warning is expected.** It says the run and the baseline "started from different states" because the human-prepared start sets the human flag in the hash. The position is the same.

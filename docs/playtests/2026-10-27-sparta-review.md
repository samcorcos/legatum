# Coach's review: the 8.9.13 Sparta conquest (2026-10-27, stopped at T126)

Save: `~/LegatumRuns/bug-saves/sparta-win-20261027.legatum`. Engine `a58e80856` (tag `playtest-8.9.13`), 48x32 map, 5 empires, every AI Hard. Replayed at the tag with 0 hash mismatches in 125 turns. The AI plan trace matches all 284 recorded AI order lists. The what-ifs are 28 branches of 32 runs, run locally (runner and data in `~/LegatumRuns/coach-tools/coach-8913-sparta/`).

VP race and cities owned (archived outside the repository)

## Coach's verdict

You played textbook Sparta. Six Hoplites queued on turn 2, no second city until a conquest, then one war after another. The defenders were not armies. They were 1 to 5 Ancient or Classical units each, with 8 to 150 Gold, and 3 unit purchases between them in 80 turns of war.

Every one of the 14 cities you took fell 1 to 4 turns after your first shot. From T70 on, your whole army was two Hoplites, later two Pikemen with Siege, plus an Arquebusier from T95. Every one of your 8 to 16 cities was empty. Germany and Athens were at War with you all that time and never attacked a city of yours. They offered each other an Alliance five times and turned each other down every time.

Hard in your seat from T40 wins 22%, from T60 75%: the Japan war (T42–T56) decided the game. "The AI still sucks at fighting wars" is right, on the defending side this time.

## 1. Result and margin

- **Where it stopped.** T126: you had 7 VP and 20 cities. Athens had 1 VP and 3 cities; everyone else was eliminated. You were 3 VP short, with Athens collapsing.
- **Your VP.** Largest Economy, Strongest Army, Most Advanced, and four era objectives: "Capture 2 enemy cities" T36, Walls in 5 cities T89, "Gain 4 cities" T106, 48 tiles T125.
- **Play time.** 24 active minutes in one session, 0.1 to 0.3 minutes a turn, 460 orders.
- **Rubber bands.** Only you paid decadence, about 10% after T60. The AIs paid 0 to 3%. Nothing competitive came from AI decisions, so I didn't run a bands-off arm.

## 2. Your campaigns and what each defender had

All AIs were tiny from the start. At T30 you had 1 city and each AI 1 or 2. At T40 you had 7 Hoplites against their 1 or 2 units each.

| War | You brought | Cities, siege to fall | Defender at the first shot (units within 3 tiles; Walls; Gold) | What the defender did |
|---|---|---|---|---|
| Switzerland, you declared T30 | 5–6 Hoplites | Schaffhausen T33 (4 turns), St. Gallen T36 (1 turn). Eliminated T36 | 1 Spearman, then none; no Walls; 139 Gold, then 6 | Bought nothing. Asked Athens and Japan for an Alliance (T31–T32), refused |
| Japan, you declared T42 | 5–6 Hoplites | Matsue T44 (3 turns), Matsuyama T49 (3), Takamatsu T56 (4, walled). Eliminated T56 | 1 Spearman each; 26–37 Gold | 14 shots at your units; Walls in two cities (T39). Bought nothing |
| Germany, it declared on you T45 ("Contain the leader", +378) | 4 Hoplites at Berlin, 2 Siege Pikemen later | Berlin T67 (3 turns), Essen T86 (3), Cologne T92 (3). Eliminated T92 | 3 Spearmen and an Archer within 3 tiles of Berlin (walled, 149 Gold), 2 units at each later city, 8–11 Gold | Killed 3 of your Hoplites around Berlin and Essen (T63–T70) with city fire and an Archer. Bought 1 Spearman (T67). One recapture campaign (T72–T88) that never attacked. Its Contain value fell from +556 to +13 the turn Berlin fell |
| Athens, it declared on you T46 (Contain, up to +1,370) | 2 Siege Pikemen and an Arquebusier, Cuirassiers from T116 | Phaleron T99, Athens T106, Colonies 8 and 9 T111 and T121, Eleusis T118, Marathon T125 (1–4 turns each) | 1–2 units per city from 3–6 in all; Walls in most; 8–112 Gold | 22 attacks on your units. Bought 2 Spearmen (T70, T87). Built 5 Settlers while at war. One 2-turn campaign (T110) |

**The pattern.**
- **Units.** The AIs fielded Ancient or Classical units against your Medieval Pikemen.
- **Walls didn't help.** Your Siege upgrade (5 bought, T50–T63) cancels the Walls halving.
- **Cities never changed back.** No AI retook a city.
- **No coalition.** Germany offered Athens an Alliance on T65, T76 and T83, and Athens offered Germany one on T87 and T90. All five were refused.

## 3. The best moment an AI had to stop you

**T68 to T85, Germany and Athens together against your two units.**
- **Your army.** After Berlin your Hoplites were down to two (one at 1 HP on T69).
- **Your cities.** All 8 to 10 of them were empty.
- **Berlin.** It stood at 0 to 40 HP from T67 to T70.
- **Germany.** It had 4 Spearmen and an Archer 1 to 5 tiles away.
- **Athens.** It had 2 to 4 units 4 to 7 tiles from your cities.

Germany's plan priced the war at almost nothing from the moment Berlin fell (+13, then 0 to 6). It started a "campaign" to retake Berlin on T72 that never fired a shot and ended `insufficient_army` on T88. Athens held a Contain plan worth +1,000 to +1,400 for 50 turns with 1 to 6 military units. It turned down the Alliance with Germany. Killing two units, or walking into an empty city, would have stopped the whole conquest.

## 4. Army quality, and whether the upgrade gene helps

**Eras behind.**
- **Germany.** Medieval era from T61 with Ancient and Classical units only; 2 upgrades all game.
- **Athens.** Medieval era with 3 or 4 Ancient units of 4 to 6 at T80–T95; 9 upgrades (480 Gold).
- **Japan and Switzerland.** Nothing but Spearmen.

**Your army, for comparison.** You upgraded your two veteran Hoplites all the way to Riflemen (9 upgrades, 680 Gold).

**The upgrade gene as a what-if** (`modernization_upgrade` and `modernization_return` at 100 in every AI, 32 runs each):

| Your seat | Branch turn | Measure | Control | With the genes |
|---|---|---|---|---|
| Hard (adaptive, the honest measure) | T1 | Your wins | 7 of 32 | 2 of 32 |
| Hard | T40 | Your wins | 7 of 32 | 7 of 32 |
| Your recorded orders | T30 | Cities you take by T125 | 13.6 | 9.0 |
| Your recorded orders | T45 | Cities you take by T125 | 10.6 | 6.0 |
| Your recorded orders | T60 | Cities you take by T125 | 8.6 | 4.0 |

- **Reading it.** The gene helps, and the earlier the better. The recorded-orders numbers are upper bounds, because your orders can't adapt.
- **A bigger army quota doesn't help.** I tried `army_war_extra` 10 with `army_item_bonus` 300. Against your recorded orders it cuts your captures to 3 to 5. Against a Hard Sparta it backfires: from T40 your seat's wins go up, 7 → 16 of 32.
- **A scripted Walls-plus-shooters floor has the same problem.** Your captures fall to 0 to 2 against your orders, but a Hard Sparta seat wins 31 of 32. More units at the cost of the economy is not the answer. Better units, and defence where the army is, is.

## 5. Was it decided early?

Yes: between T40 and T60, by the Japan war.

| Hard in your seat from | Wins (of 32) | Median win turn | Cities captured by T125 |
|---|---|---|---|
| T1 | 7 | T282 | 4.3 |
| T40 | 7 | T226 | 4.4 |
| T60 | 24 | T211 | 2.3 |
| T80 | 26 | T197 | 1.1 |

You did it much faster: 14 captures by T125, and the win was near at T126. Nobody should know the winner by T100, but here Hard itself wins 75% of the time from T60 (pillar 11).

## 6. Defence fixtures (specified, not added)

The states are extracted at the release engine (0 hash mismatches) in `~/LegatumRuns/coach-tools/coach-8913-sparta/states/` (T30, T40, T42, T60, T65, T80, T97). As with the Athens fixtures, they need a `war_build.rs` recipe and his orders recorded as a script. Predictions are written now:

| Id | AI seat | Start | Goal and deadline | His game | Prediction (Hard / Hard with upgrade gene) |
|---|---|---|---|---|---|
| `war-japan-hoplites-t42` | Japan (2), against his recorded orders T42–T56 | T42: 3 cities, 2 Spearmen, 42 Gold; 5–6 Hoplites at the border | Keep at least 1 city through T56 (15 turns) | All 3 lost, eliminated T56 | 5–15% / 10–20% |
| `war-germany-berlin-t65` | Germany (1), his orders T65–T92 | T65: 3 walled cities, 5 units, 156 Gold; 4 Hoplites at Berlin | Keep at least 2 of 3 cities through T92, or retake Berlin by T80 | All 3 lost; the Berlin recapture campaign never attacked | 10–25% / 20–35% |
| `war-athens-two-pikemen-t97` | Athens (4), his orders T97–T125 | T97: 8 cities, 6 units; his army is 2 Siege Pikemen and 1 Arquebusier | Keep at least 5 of 8 cities through T125, and kill both Pikemen | 6 lost, both Pikemen alive | 10–20% / 15–30% |

## 7. The single change most likely to make this game hard for you

**Defend where the army is, with units of the era.**
- **What the change is.** When a visible field army is inside the defender's march window, it should do three things:
  - put its best current-era unit, built or bought, in the threatened cities;
  - spend its Gold on upgrades of the units already there;
  - attack the attacker's few, wounded units with everything in reach, instead of a recapture campaign that never fires.
- **Where it lives.** This is idea #23 ("defence that counts the army at the door") together with the upgrade gene: `threat.rs::plan_threat_defense`, `production.rs::plan_emergency_production`, and `military.rs` counterattacks.
- **The evidence.**
  - Every city fell in 1 to 4 turns to 2 to 6 units.
  - The defenders bought 3 units in 80 turns of war.
  - The upgrade gene alone cuts your captures by 4 to 5 cities against your orders, and from T1 takes a Hard Sparta's wins from 7 to 2 of 32.
  - The crude defence floor shows the ceiling (0 to 2 captures) but costs too much economy.
- **How to measure.** On the three fixtures, then the human-seat gate.

The coalition side is second. Two AIs at War with the same leader should accept each other's Alliance: five refusals here.

**Air:** not reached. Nobody had Flight by T126.

## Compared with tonight's Athens game

Same root causes, seen from the other side:
- No AI attacks a leader's empty cities: 0 attacks here, 1 in the Athens game.
- Wars on the leader are valued but never fought: Athens's +1,370 Contain here, Sparta's paused campaign there.
- Coalitions don't form.
- Armies stay obsolete, and in both games the upgrade gene is the one measured lever.

What's new here: the defenders were also tiny. They had 1 to 5 units each, so a light army of 2 to 6 units was enough.

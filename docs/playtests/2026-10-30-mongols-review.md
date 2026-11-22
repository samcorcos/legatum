# Coach's review: the 8.9.17 Mongols win (2026-10-30)

Save: `~/LegatumRuns/bug-saves/mongols-win-2026-10-30/autosave.legatum`, plus the T96 save of the same game (`mongols-hollow-8917-20261030.legatum`), which holds turns 1 to 95. Five empires on a 48x32 map: you (Mongols), Byzantium, the Celts, Athens and Switzerland, every AI on Hard.

**Which AI played.** The T96 save's archive says it was recorded on the 8.9.16 engine. That label is stale: it records the engine the game started on. The AI was 8.9.17 all game. The 8.9.17 AI reproduces every recorded AI decision in turns 1 to 95 (380 of 380); the 8.9.16 AI departs from them 20 times from T63, the first turn the new hollow read changed a decision. The rules are identical in both builds. So the hollow read (`hollow_read`, `persist_hollow`) was live from the start.

**Method.** The whole game replays at the 8.9.17 tag with 0 hash mismatches over 172 turns, and every AI order list matches (600 of 600). What-ifs: 30 branches × 32 runs from T60, T96, T133 and T150, on the box (about 4 minutes). Tools and data: `~/LegatumRuns/coach-tools/coach-8917-mongols/`.

VP race and army size (archived outside the repository)

## Coach's verdict

"Ran away with it easily" is right. **No AI attacked one of your cities in 172 turns.** Three AIs were at War with you for a combined 215 turns:
- Switzerland: T56 to the end, after you declared;
- Byzantium: from T86;
- Athens: T66 to T79.

Two of them ran a plan called "contain the leader" against you for 70 to 80 turns, and neither launched a single campaign until you were already taking Swiss cities. You had 3 to 6 military units for 10 to 14 cities, and 6 to 10 of those cities had nobody inside.

8.9.17's new hollow read did fire. It is why Athens declared on you at T66, and why the Swiss and Byzantine plans pointed at you. But wanting the war never became marching on a city. The agent "hollow-attack" is tracing why from your T96 position, so this review stays on the whole game.

**The best army on the map never came for you.** The Celts had the strongest army from T100 (1,767 at the end) and sat four tiles from your colony. They spent 125 turns in a war with Athens that neither side fought and neither side would end. They were at Peace with you all game.

**The only brake was a rubber band.** Decadence took 16.9% of your Production, Gold and Science and almost nothing from anyone else. With decadence off from T96, Hard playing your seat wins 32 of 32 games instead of 22, and about 35 turns sooner.

**What would most likely have beaten you:** the Celts at War with you from T96 and never making Peace. With that change, Hard in your seat wins 47% instead of 69%, and your own recorded orders win 0 of 32 instead of 13.

## 1. Result and margin

- **Result:** you won on T172 by holding 10+ VP for 10 turns (T163 to T172). The final VP was 13 to Byzantium's 5, the Celts' 5, Athens's 2 and Switzerland's 1.
- **Your 13 VP:**
  - two crowns: Largest Economy (held since T28) and Most Advanced (since T85);
  - five era objectives:
    - Markets in 5 cities, T95;
    - Libraries in 5 cities, T115;
    - Banks in 3 cities, T133;
    - capture a city, T157;
    - grow by 34 population, T168;
  - the Mongol goal, T133 (+2);
  - the Swiss Great Library you captured with Schaffhausen, T163 (+2: your tenth point);
  - your own Great Library, T168 (+2).
- **The VP race.** Byzantium led 5 to your 3–4 from T102 to T132. You jumped to 7 on T133 and nobody passed 5 after that. On the VP track, pillar 11 held at T100.
- **When it was decided.** Hard playing your seat (32 runs from each turn, its own dice) wins:
  - 17/32 from T60;
  - 22/32 from T96;
  - 22/32 from T133;
  - 32/32 from T150.

  With five empires, an even game is 20% a seat. You were already a 2-to-1 favourite at T96, though the track didn't show it. It was settled between T133 and T150, when your army went from 296 to 670 while the Swiss, your target, held 3,257 Gold and an Ancient army.
- **Your pace beat Hard's.** From your T96 position, Hard's median win comes on T229 and it has 6 VP at T172. You won on T172 with 13.
- **Play time:**
  - 43 active minutes over 3 sessions;
  - minutes a turn by era: 0.1 (era 1), 0.3 (era 2), 0.2 (eras 3 to 5), 0.5 (era 6);
  - about 570 orders in 172 turns.

## 2. Turning points

| Turn | What happened | Swing |
|---|---|---|
| T1–T95 | **Expansion.** You had 4 cities at T50 (AIs 2–4), 7 at T75 (3–7) and 10 at T95 (4–6). You queued Granaries, Settlers, Farms and Markets in the first 60 turns. By T95 you had Markets in 5 cities (an era objective) and Largest Economy. | The economy everything else rests on |
| T56–T86 | **Three wars, no attacks.** You declared on Switzerland (T56). Athens declared (T66), which the 8.9.16 AI would not have done: the hollow read's first visible act. Athens made Peace at T79 without a single battle. Byzantium declared (T86). | 0 city attacks in any of them |
| T133 | **+3 VP in one turn:** Banks in 3 cities and the Mongol goal take you from 4 to 7, against Byzantium's 5. | You lead for good |
| T145–T161 | **The army for the one war you chose.** Army strength rose from 296 (T140) to 1,061 (T160): Field Cannon, Riflemen, upgrades, and 2 bought Bombers (T156, T161; 18 air strikes in all). | A Swiss army of Phalangites can't answer it |
| T157–T168 | **The Swiss.** You took Chur (T157, a 652-Gold bounty from their hoard), Sion (T161), Schaffhausen (T163: their capital and Great Library, your tenth VP; the hold starts) and St. Gallen (T168). | 7 → 13 VP. Nobody attacked your cities during the hold |

## 3. Your edge (what you did that the AI doesn't)

The designer asked: "how does my build order compare to the AIs? ... I win on population, science, gold ... I don't understand how that's possible." In this game:

- **The army wasn't the early difference; expansion was.** In the first 60 turns you trained 2 military units and the AIs 2–4, and everyone trained 3–4 Settlers. The AIs also built 2–3 Walls each (the Mongols can't) and Quarries; you built Markets, Farms and Mines. The lead's five-game pattern (0–1 units against 6–15) is weaker here.
- **The gap opened after T50, and it's cities:**

  | Turn | Your cities | AI cities | Your Gold a turn | AI Gold a turn |
  |---|---|---|---|---|
  | T50 | 4 | 2–4 | 11 | 2–10 |
  | T75 | 7 | 3–7 | 27 | 0–16 |
  | T100 | 10 | 4–6 | 51 | 4–44 |
  | T125 | 12 | 4–8 | 68 | 7–56 |

  You trained 12 Settlers and bought 3 more. Byzantium trained 5 and Switzerland 6, and every AI city count was flat from about T100. Population and Science follow the cities: 83 population and 63 Science at T150, against the AIs' 44–53 and 36–55.
- **You paid a third of their upkeep.** Unit upkeep T1–T100 was 76 Gold for you, against 225 (Byzantium), 225 (Celts) and 405 (Switzerland, which pays 3 a unit). The Celts' net income over those 100 turns was 236 Gold in all. Each AI paid for a standing army of 5–9 units that never attacked you. The lead's reading holds: you win the economy partly *because* the AI pays for an army it doesn't use.
- **Building objectives scale with cities.** Markets, Libraries and Banks in N more cities gave you 3 of your 13 VP, and those objectives favour the empire with the most cities.
- **Hollow and safe.** You had 4 Horsemen at T96, 3 Horsemen at T125, and 6 units at T150 for 14 cities, 10 of them empty. This is the fourth game in a row where you trusted the AI not to punish an empty empire (Athens, Sparta and Switzerland before). It didn't.
- **Target choice:** the same as the Swiss game, the weakest neighbour holding a public +2 Great Work. This time it was an AI Switzerland playing your own Swiss strategy: 3,769 Gold and Phalangites in era 6.
- **Diplomacy:** you declared once (T56). You offered Peace to Byzantium and Switzerland at T132 (neither accepted), took Athens's Peace at T79, and refused the Swiss Peace at T167.
- **Signature:** the Mongol goal (+2 at T133). The Mongols' weakness (no Walls before the Medieval era) cost you nothing, because nobody came.

## 4. The AI's mistakes

### Faction-independent (rank these first)

1. **Read the leader, never march (Switzerland and Byzantium).**
   - The Swiss Contain plan targeted you from T78 to T158: 80 turns, valued +124 to +566, with your threat score 104–173 ("VP race +51 to +124").
   - The Byzantine plan did the same from T86 to T159 (73 turns, up to +786).
   - Not one campaign started in either war until you took Chur. Then the Swiss started one (T158) and Byzantium too (T159). Both ended `infeasible` or `insufficient_army` within 1–3 turns.
   - The Swiss had 4 of their 6–7 units within 6 tiles of your empty cities all game, at War. Almaliq (Walls, one Horseman) and Mongols Colony 8 (Walls, empty) were 4–5 tiles from Chur.
   - Byzantium's cities are 10 tiles from yours, and it saw none of your units in reach until T140.
2. **The strongest army fought a war it didn't want, and so never joined the war that mattered (the Celts).**
   - The Celts named you their top threat at T81, then switched to Byzantium at T87.
   - From T87 to the end their plan was "Peace with Athens". Yet they refused all 11 of Athens's Peace offers (T66–T169).
   - The war between them saw 28 unit battles after T96 and no city attacked.
   - In the what-ifs, a Peace proposal forced from each side (Celts T96, Athens T97) was refused both times, so the war is locked in by both peace evaluators, not by one stubborn AI.
   - Their army was the strongest on the map (1,052 at T140, modern Cuirassiers and Assault Troopers), four tiles from your empty Colony 10, at Peace with you all game.
3. **The coalition was priced out.**
   - 19 Alliance offers were refused. 11 of them failed on "Signing costs 60 Gold; you have 13–36": Byzantium's treasury sat at 13–90 all game.
   - The richest AI, Switzerland (768–3,769 Gold), refused Byzantium's offers on the fee's score (−12) at T108 and T119.
   - The one Alliance against you (Byzantium and Switzerland, T136) changed nothing: no attacks followed.
4. **The rich AI hoarded and stayed obsolete (Switzerland).**
   - It held 768 Gold at T96, 3,257 at T150 and 3,769 at T157, at War with a hollow neighbour 4 tiles away.
   - Its army at T150 was 3 Phalangites, 2 Rangers, a Horseman and an Arquebusier.
   - When your Bombers came it bought units (9 after T96) but never an answer to air.
5. **Decadence and the AIs' own armies taxed them, not you.** See section 5a.

### Tactical

- **The Swiss campaigns:** after Chur fell (T157), the recapture campaign on Chur (T158, +893) and then on Sion (T164, +3,808) ended `insufficient_army` on T165. Their Contain then sat at value 0 to 500 through the hold.
- **Byzantium's two campaigns** (T159 on Chur, T165 on Sion) ended `infeasible` on T161 and T168.
- **No Peace mistakes this time:** no AI signed Peace with you after T79. The 8.9.17 Peace genes held. Switzerland offered Peace at T167, as its cities fell, and you refused.

No cheating-adjacent idea is needed: your empty cities, your army strength, your crowns and the Swiss Great Work were all public.

## 5. Was the win fair?

**Skill the AI can learn, against an AI that doesn't punish greed.**
- Fast settling, Markets, then one short war on the weakest neighbour with a public Great Work is a learnable plan. Its counter, a neighbour attacking the empty empire, exists in the rules: the Mongols can't wall before the Medieval era, and you had 6–10 empty cities. Nobody used it.
- **No exploit or rules hole** is involved. The Bombers mattered only at the end, and the Swiss had nothing to shoot them with, as in the Athens game.
- **Luck:** a rich, obsolete AI Switzerland next door with a Great Work. The Celts, the one AI that could have hurt you, were locked into a war with Athens.

## 5a. Smarter AI or a weaker leader?

| | You | Byzantium | Celts | Athens | Switzerland |
|---|---|---|---|---|---|
| Decadence, share of Production + Gold + Science lost | **16.9%** (5,820) | 0% | 0% | 1.1% | 0.6% |
| Rate at T75 / T100 / T125 / T150 / T172 | 8 / 15 / 21 / 23 / 27% | 0 | 0 | 10 / 2 / 0 / 0 / 0% | 4 / 2 / 2 / 0 / 0% |

- **Tech diffusion:** the AIs learned 50 techs after you knew them (11–14 each). Your marginal share of each tech's discount is 10% of its cost, about 5 techs' worth spread over four empires. That is small next to your decadence.
- **Crown upkeep** no longer exists.
- **Counterfactual (Hard in your seat), decadence off:**
  - from T60: 17 → 25 of 32;
  - from T96: 22 → 32 of 32 (median win T229 → T194);
  - from T133: 22 → 31 of 32.
- **AI-smartness side:** no AI gene changed since the W2.3 gate (rivals with the hollow read: +0.34 pp [−1.38, +2.07], neutral). The benchmark wasn't rerun.
- **Verdict:**
  - In this game the brake on you came about **100% from the bands and 0% from AI decisions.** The AIs took no city and made no attack on one, so their decisions cost you nothing. Decadence cost you about a sixth of your output and roughly 35 turns of tempo.
  - The AI lever that exists is worth about as much as the band: the Celts at War from T96 took 22 points off Hard's win rate in your seat. Decadence off adds 31.
  - Uncertainty is about ±15 points per branch at 32 runs.

## 5b. What would have beaten you

The branches start from the release-tag states at T60, T96, T133 and T150. AI memory is restored exactly at T96, and the shadow replay matches every AI order. Your seat was played two ways:
- **Hard**, with its own dice, which approximates you;
- **your recorded orders** (the tape). It is fragile here: from T96 it wins only 13/32 even unchanged, with about 200 orders failing as the AIs' salted seeds drift. A neutral gene (`vow_hold`, which can't act without Japan) gives exactly the control result, 13/32 and 5/32. Read the tape column as an upper bound on harm and the Hard column for size.

| Change (AIs only) | From | Hard in your seat | Your recorded orders |
|---|---|---|---|
| **Control** | T60 | 17/32 | 13/32 |
| Celts at War with you, no Peace | T60 | 16/32 | 0/32 |
| Decadence off | T60 | 25/32 | |
| **Control** | T96 | **22/32** | **13/32** |
| **Celts at War with you, no Peace** | T96 | **15/32** | **0/32** |
| Celts and Athens join Byzantium and Switzerland, no Peace | T96 | 15/32 | 4/32 |
| Celts and Athens forced to propose Peace to each other | T96 | 22/32 (both refused; nothing changed) | 13/32 |
| Decadence off | T96 | 32/32 | |
| **Control** | T133 | 22/32 | 5/32 |
| Celts at War with you, no Peace | T133 | 22/32 | 5/32 |
| Coalition of all four, no Peace | T133 | 20/32 | 3/32 |
| Decadence off | T133 | 31/32 | |
| **Control** | T150 | 32/32 | 32/32 |
| Coalition of all four, no Peace | T150 | 32/32 | 0/32 (fragility: 297 failed orders against 4) |

**Ranked:**
1. **The Celts at War with you from T96, never making Peace: −22 points for the Hard seat, and your recorded orders stop winning.**
   - In those runs, hits on your cities double (8 → 18.5 a game) and Switzerland wins 8 of 32.
   - Learnable from the AI's own information: yes. The Celts saw you as top threat at T81, your cities four tiles away were empty, and your crowns and VP were public.
   - It maps to rec 2 (end the phony war, take the war that matters) plus rec 1 (march once at War).
2. **A full coalition from T96:** about the same (−22 points). Adding Athens adds nothing to the Celts alone.
3. **Too early or too late does nothing.** The Celts at War from T60 cost you a point (17 → 16): your T60 empire was still small and their army not yet modern. From T133 it changed nothing (22 → 22). The window was T96 to about T130, when their army was the best on the map and yours was three Horsemen.
4. **From T150 nothing beats you.** Even a four-empire coalition leaves Hard in your seat at 32 of 32.

## 6. Did the 8.9.17 changes show up?

- **The hollow read fired:**
  - from T63 it changed decisions;
  - Athens declared war on you at T66 because of it;
  - the Swiss (T63, then T78) and Byzantium (T86) aimed Contain at you;
  - by T96 every AI's memory listed you as hollow (58);
  - your threat score jumped from 42–56 to 102–105 at T96 with "VP race +51".
- **It never became an attack.**
  - 0 campaigns against you before T158.
  - 0 attacks on your cities all game.
  - The Contain value stayed positive (+124 to +786) for 70–80 turns with no march.
- **War persistence (`war_reaim`, `peace_own_danger`) had nothing to persist:** no campaign existed to re-aim. Peace discipline held: no AI offered you Peace mid-war until the Swiss collapse.
- **Fighters:** no AI faced your two Bombers with one; nobody else had Flight.
- **The runner's "wars on seat 0" counter reads 0 for every branch:** a matching-order bug in the throwaway runner. Forced wars are confirmed in each branch's log instead.

## 7. Coach's recommendations (ranked by expected effect on how often you win, with any faction)

| # | Mechanism | Where | How to measure | Evidence across games |
|---|---|---|---|---|
| 1 | **From read to march.** A Contain or war plan against a hollow leader must produce a campaign on his softest reachable city within a few turns, or the plan is re-priced. 70–80 turns of Contain with 0 campaigns is the failure. The hollow-attack trace will name the gate (forecast odds, march window, second-front veto, the army's era); fix that gate, not the read. | `plan.rs` (Contain → campaign candidates), `campaign.rs::campaign_forecast`, `military.rs::campaign_target_for` | New Scenario Lab `swiss-strike-the-horde` (Hard 0/50) and `mongols-rich-and-empty` (0/50) as fixtures; war suite; human-seat benchmark both ways | This game (Swiss 80 turns, Byzantium 73, 0 city attacks in 172 turns); 8.9.15 Swiss (15 hits in 180 turns); 8.9.13 Athens (1 attack); 8.9.13 Sparta (0 on empty cities) |
| 2 | **End a phony war to fight the real one.** When a war has gone N turns without a city attack on either side and the enemy offers Peace, accept it, above all when the top threat is someone else. Then let the freed army price the leader. The forced-Peace branch shows both sides' evaluators refuse today. | `diplomacy.rs::wants_peace` (`leader_peace_continuation_value`, `rival_redeclare_permille`) and the response path; `plan.rs` second-front pricing | The T96 Celts branch (−22 pp in his seat); count phony-war turns per game in `hard_vs_human`; benchmark | This game (Celts–Athens, 125 turns, 11 refused offers); 8.9.15 Swiss (Mongols fought Japan 87 turns); 8.9.13 Athens (Sparta's 65-turn paused campaign) |
| 3 | **Spend the war chest on the war.** At War with a hollow neighbour in reach, turn Gold into an attack force (upgrades to the era's units, siege), not a hoard; cap the reserve at what the faction goal needs. | `production.rs` purchases, `military.rs::hold_or_upgrade`, the Swiss hoard logic in `victory.rs`/goals | `swiss-strike-the-horde`; war suite; Swiss AI win rate in the benchmark | This game (Swiss 3,769 Gold, Phalangites in era 6); 8.9.15 (door defense had nothing to spend) |
| 4 | **Price standing army against economy, or use it.** The AIs paid 3× your upkeep (225–405 Gold against 76 by T100) for armies that never attacked. If recs 1–3 don't make those armies pay, the step 3 evaluator should learn the army size from outcomes (the lead's levers). | `production.rs::desired_army`, step 3 evaluator | Upkeep share and Gold at T100 in the benchmark; his-seat milestones | This game; the lead's five-game comparison (2026-10-30 learnings) |
| 5 | **Coalitions the poor can afford** (AI half): the richer partner offers to cover the fee, or an Alliance against the formal coalition target is valued above its fee. | `diplomacy.rs` Alliance scoring | Alliance acceptance rate against the leader; T96 coalition branch | This game (11 of 19 refusals on the 60-Gold fee; one Alliance at T136 that did nothing); 8.9.13 Sparta and 8.9.5 Athens (alliances against the leader refused) |

**Design questions for the designer (not AI fixes):**
- **The Alliance fee.** Should an Alliance against the public coalition target be free, or paid by the proposer? The 60-Gold fee blocked most of this game's coalition attempts by broke AIs.
- **Decadence is the only brake on you, and it works.** It taxed you 16.9% and nobody else, and it changed your result more than any AI decision did. That's pillar 11 doing its job; it's reported here for balance, not as a problem.
- **Faction balance:** one line. Nothing here is Mongol-specific: you won with the same plan as Switzerland and Athens.

## Scenarios

- **`swiss-strike-the-horde` (show first).**
  - **The setup:** turn 140, you play Switzerland, at War with the Mongols (your empire in this game) for 84 turns. They lead on VP with 13 cities, 5 military units and most cities empty. You have 2,587 Gold and seven obsolete units.
  - **The goal:** take Almaliq (Walls, one Horseman, 4 tiles from Chur) within 15 turns without losing a city.
  - **Hard's baseline:** 0/50. It never declares a new war, and 22 of 50 runs lose a Swiss city.
  - **The question:** can a human, in the AI's shoes, turn a long war and a hoard into a capture? A win here is the recipe for recs 1 and 3. It is the mirror of `mongols-rich-and-empty`, which has no runs yet.
- **Not cut: a Celtic attack** from the same turn. A Celtic capture always sacks the city, so a "take" objective can't be scored. The T96 Celts branch above answers the AI side.
- **Earlier scenarios:** no Scenario Lab runs since 2026-10-22 (`runs-report.py`), so there is no new feedback to quote.

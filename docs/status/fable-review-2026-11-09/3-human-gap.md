# Review 3: the evidence from real play — where and why he beats Hard

Master `a9eb46864`, 2026-11-09. Read-only review; no sims run. Sources, abbreviated below:

- **AZ** = `docs/playtests/reviews/2026-11-08-8.9.29-aztecs.md`
- **COACH** = `.claude/skills/legatum-coach/SKILL.md` (Recurring patterns and Verdicts tables; it summarises every review)
- **HR** = `docs/playtests/human-reasoning.md` (his own words)
- **DS** = `docs/status/designer-seat.md`
- **SG** = `docs/sim-results/ai-quality/swiss-gap-1107/README.md`
- **TG** = `docs/sim-results/ai-quality/takeover-gap-1107/README.md`
- **PROG** = `plans/ai-war-progress.txt`
- **LAB** = `scenarios/lab/*.json` stored Hard baselines (50 runs each)

Limits: I worked from the coach reviews and their tables, not by re-replaying tapes. T25 and T75 figures exist for only a few games (the reviews tabulate T50 and T100), so those rows are thinner. Anything marked *(inference)* is mine.

## 1. Timeline of a game he wins

He has won 11+ games in a row with every faction; win turns T116, 122, 135, 144, 153, 153, 159, 161, 172, 177, 177, 180, 181 (COACH, Verdicts). Active play is 22 to 62 minutes a game (COACH).

| By | Him | Hard rivals | Gap | Source |
|---|---|---|---|---|
| **T25** | 1-4 cities; a 5-unit single-type army already marching (Aztecs: 5 Spearmen, 1 city, 2 VP, declares T26). Germany: 3rd and 4th city at T23 and T32. A scout parked on the neighbour's Settler route. | 2 cities, 1-3 military units; Switzerland had 2 cities and 1 unit when he declared. Germany's rivals found city 3 and 4 at T44 and T57. | Cities about even; the gap is army concentration and intent. Hard in his seat from T20 wins 0-2 of 48 in five of eight tapes. | AZ; COACH (under-expansion row); DS |
| **T50** | 4 cities (India, Swiss 8.9.24, Mongols 8.9.17, Aztecs 8.9.29), 3 in Aztecs-2; one neighbour dead or crippled (Swiss eliminated T42, 8 Spearmen alive); treasury 160-421 Gold (835 as Switzerland). | 2-4 cities; treasury median 21-64 Gold. | +0 to +2 cities, 3-10x the Gold, one fewer rival. | COACH; SG; AZ |
| **T75** | 7-9 cities (Mongols 8.9.17: 7; Aztecs 8.9.29: 8 by T69, all walled; Germany: 9 at T81); army 6-9 veterans, upgraded once; second war started (Sparta T63). | 3-7 (Mongols game), 4 (Germany). | About 2x cities. | COACH; AZ |
| **T100** | 9-11 cities, 7-8 VP (Aztecs 8.9.29: 11 cities, 8 VP at T101; Swiss 8.9.24: 9; India: 11; Mongols: 10); army still 6-8 units; upkeep about 76 Gold. | 4-8 cities, 2-4 VP; upkeep 225-405 Gold on armies that made 0-10 city attacks all game. | 1.5-2.5x cities, 2-4x VP, a third of the military bill. | AZ; COACH (rows "under-expands", "pays for an army it never uses") |
| **End** | Aztecs 8.9.29: 11 VP at T116, 23 techs, lost 4 units. | Byzantium 2, Celts 4, India 3, Sparta 0 VP; Sparta 15 techs, lost 23 units. | | AZ |

**Where the gap opens.** The takeover curves (Hard plays his seat from turn N, 48 reseeded runs; DS) date it:

| Tape | T2 | T20 | T40 | T60 | T80 | Coach's "decided by" |
|---|---:|---:|---:|---:|---:|---|
| Aztecs 8.9.29 | 0 | 0 | 1 | ~11 | 44 | T55-70 |
| Swiss 8.9.24 | 0 | 0 | 20 | 33 | 44 | T41-90 |
| Mongols 8.9.25 | 11 | 2 | 2 | 42 | 48 | T55-70 |
| Sparta 8.9.26 | 17 | 20 | 37 | 48 | 48 | by T50 |
| Swiss 8.9.26 | 12 | 12 | 29 | 34 | 47 | by T80 |
| Germany 8.9.27 | 0 | 1 | 3 | 5 | 4 | not until T144 |
| India 8.9.25 | 2 | 1 | 0 | 4 | 9 | T149 Bomber war |

Two shapes. In five of seven the game is decided between **T40 and T70**: one early conquest or Settler capture plus the walled colony wave turns a 0-40% position into an 70-100% one. In the other two (Germany, India) Hard cannot convert his position even from T80, meaning his lead there rests on things Hard does not do at all (air war, VP conversion, veteran army), not on the board. So the gap opens at T25-T45 (one decisive act), becomes structural by T70, and nothing a rival does afterwards changes it. In the Aztecs game Hard given his T55 position still loses its young cities (holds 7.1-7.7 cities and 3.3 VP at T100 against his 10 and 7; AZ): he also defends width better than Hard does.

## 2. Recurring reasons, ranked

Ranked by games in which the pattern appears and how decisive the coach measured it. Status: **landed** (on master, default on), **partial**, **tried, not landed**, **nobody**.

| # | Reason | Games | Evidence | Status |
|---|---|---|---|---|
| 1 | **Rivals do not attack him, even at war and next to empty cities.** | Nearly every game | 8.9.29: 10 AI city hits in 116 turns, 8-10 of his 11 cities empty; Celts kept 7 of 8 units within 3 tiles of unwalled St. Gallen (1 Spearman) T99-T114 and attacked 0 times. India: 537 empty-city-turns unpunished, he lost 2 units and killed 112. 8.9.26 Celts: 406 empty-city-turns with an at-war unit within 6 tiles. 8.9.17 Mongols: 0 attacks in 172 turns (AZ, COACH). | Partial. `siege_stance` 6 and `runaway_coalition` 55 landed 11-08 (PROG); both were live in 8.9.29 and the result was still "didn't really do much". Operation executor: executed strikes lose on average (-15 pp), kill check 3/38 (PROG). |
| 2 | **Nobody builds a real coalition or contests the hold.** | 8+ games; 4 named for the hold (8.9.25 Celts, 8.9.26 Celts, 8.9.26 Swiss, 8.9.29) | Wariness 48-82 from T50 in 8.9.29, yet India never joined (alliance upkeep fee; 4-20 Gold), Byzantium spent its war on two 0-VP colonies. What-ifs: Sparta turning on the leader moves his win 88% to 1%; Celts at war from T96 moves his hold orders from 13/32 to 0/32 (COACH). LAB: `byzantium-break-the-fire`, `athens-break-the-hold`, `celts-strike-the-leader`, `india-strike-the-leader` all 0/50. | Partial (bits landed), core open. Hollow-runaway declare-and-strike was named "NEXT" on 11-07; Geneva 0/64 in every arm (PROG). Alliance fee is a design question nobody owns. |
| 3 | **Settlers walk unescorted and he takes them.** | 4 games, decisive in each | India: Hard seat 8% from T16, 56% from T22. Swiss 8.9.24: T37-T41 Settler capture worth +54.4 pp [46.8, 62.0] and +3.6 VP (SG). Celts 8.9.26 T17, Germany 8.9.23. HR: "AI is terrible at defending its settlers". LAB `celts-steal-a-settler`, `india-take-the-settler` 0/50. | Tried, gated off: idea #43 (`settler_safety` + `peace_settler_capture`) saved Germany 48/48 but was switched off 10-24 (COACH). Nobody now. |
| 4 | **Victims do not defend against an army at the door.** | Most conquest games | Switzerland: 1 unit against 5 Spearmen at T26; Sparta ungarrisoned Therapne T84-88 and queued Wonders under siege, 3-4 Science a turn (AZ). AI treasuries 14-190 Gold so the door-defence buy buys nothing (COACH). | Partial: `hard-city-defense-tactics` landed 11-08 (Hold Biel 0/50 to 21/50; PROG). The upstream valuation (stop building Settlers and Wonders, keep Gold for the door) is open. |
| 5 | **Wrong targets: nearest small city, never the VP city.** | 6+ games | 8.9.29: Solothurn (walled, empty, 3 VP) never targeted; Byzantium hit two 0-VP colonies 9 times. His closing VP is usually a rival's Great Work or goal unit that its owner left undefended (COACH). | Nobody (`victory.rs::hoard_denial_value`, `conquest.rs` named by the coach). |
| 6 | **Obsolete, unused armies.** | Most games | Celts' strike read "capture at 6%, era deficit -80" and parked; AI upkeep 225-405 Gold vs his 76; era resources go to buildings; `modernization_value` loop (COACH). Dropped 8.9.29 card: Hard storms Therapne 49/50 once upgraded, so the storm is not the problem, the upgrade is (AZ). | Open: plan W2 upgrade gene (`plans/ai-war.md` line 49: "army quality comes before operations"). Not landed. |
| 7 | **AI under-expands in the T20-T60 window.** | 6 games; 1 counter-case | Table in section 1. Germany: his cities 3 and 4 came 21-25 turns earlier. | Partial: `swiss_playbook` 3, `site_denial` landed 11-07; the coach notes the expansion genes changed no rival's T100 count (COACH). |
| 8 | **Air stage is uncontested.** | 10+ games, decisive late in 3 | 0 AI Fighters in 10+ games; he reaches Flight 17+ turns first; one bought Bomber takes 4-8 cities; air ignores Walls; `decisive_buy.rs::could_intercept` blocks Hard's own Bomber buy (COACH). | Nobody. Matters less while he wins by T116-T150 on land. |
| 9 | **No working rubber band.** | Every recent game | Bands 0% (decadence off since 10-31, crown upkeep removed, diffusion 7-9% of the tech bill). When decadence was on it was "the only brake" (16.9% of output in Mongols 8.9.17; restoring slope 45 in Aztecs 8.9.22 cut Hard-in-his-seat from 24 to 13 of 32) (COACH). | Rules, frozen. His decision. |
| 10 | **Veteran unique-unit army and faction kit.** | Sparta, Mongols, Aztecs | He kept 5 Hoplites alive T29-T97: 13 upgrades, 18 captures, 1 lost; Hard in the seat loses 7-14 and captures 2.1 (COACH). Mongols T34-T38 orders worth +18.6 pp; Hard's Keshigs move instead of shooting (`military.rs` `press_strike` runs before `dual_fire`) (TG). | Ideas #47/#48 open; `mongol_horde` 3 landed; the pass-order gap is not fixed. |

## 3. His playbook, as valuation errors in Hard

Each line is "he values X at about Y; Hard values it near Z". His words are from HR.

1. **A weak neighbour's city is the cheapest city in the game.** "Conquering these cities is the fastest way to grow my economy." He compares a capture with a Settler and takes the capture when the defender shows 1-2 units. Hard prices early conquest as a war (risk, army parity), not as expansion.
2. **Concentration beats size.** Five identical units on one city at T26, "pick one and commit". Hard spreads a larger army over guard duty and several half-wars.
3. **A veteran is worth several recruits; never trade one.** He retreats and heals, then upgrades the same units for 90 turns. Hard prices units at build cost.
4. **Gold in the bank is a defence.** He keeps 160-1,280 Gold and buys two Archers on the turn the raid arrives (removing either buy loses Lugano in 32/32 replays; SG). Hard spends to zero, then cannot buy at the door.
5. **Walls plus width instead of garrisons.** "I had buffer"; he bets empty walled colonies will not be attacked, and is right. Hard pays upkeep for garrisons everywhere and still loses its young cities.
6. **A rival's Settler is two cities** (one gained, one denied) and worth a declaration of war. Hard values a Settler capture as a unit capture.
7. **Information is worth a unit.** Scouts on chokepoints for 5 turns' warning; that is what lets him run with empty cities.
8. **Target by VP per capture and by visible defence**, not distance: wonder cities, capitals, Great Work cities.
9. **Tempo over tidiness.** He settles 5 to 8 cities in ten turns (T59-T69) once the first war pays; reveal tech first; Science project toward Flight from a finished capital.
10. **Wars are tools.** He closes fronts he does not need near a win; Hard keeps dead wars (India refused Peace 9 times in 8.9.29) and so cannot afford the war that matters.
11. **VP is the score.** He converts every lead to VP; Hard leaves crowns and objectives on the table (`crowns.rs::objective_effort`).

One edge Hard never gets: he re-rolls starts for production (HR). Balanced starts (rules 24, PROG) reduce this.

## 4. Why winning feels easy and dull

His own words: "I'm bored... press return and wait"; "my cities were basically undefended... nobody attacked with a serious force"; "conquering the world shouldn't be this easy"; the coalition "didn't really do much"; "the aztecs should have beaten me... didn't use [their army]" (HR).

**It is mostly the AI, but the rules now do nothing to slow a snowball, and two rules actively protect it.**

AI side (the larger share, *inference* from the what-ifs): one committed rival changes the result completely (88% to 1%; 13/32 to 0/32 holds). The games he remembers as good all involved an AI that attacked: Athens 8.6, Mongols 8.9.2, Sparta 8.9.5, the Swiss retake in 8.9.21, the Celts second front in 8.9.22 (11-8 finish), the T30 loss to Sparta in 8.9.26, which he called a "good outcome" (HR). So pressure is what makes it fun, and when Hard applies it the rules allow a contest.

Rules side:
- **No cost to wide conquest.** A captured city pays immediately, no unrest, decadence off. His 11 cities cost him 6-8 units and 76 Gold of upkeep. Bands are 0% of output in every recent review (COACH).
- **Walls make empty cities safe from the armies Hard builds.** Era deficit plus Walls gives a forecast of 6%, so the correct AI move really is not to attack. The AI is partly right to sit still; the rule asks for an upgraded siege army Hard never builds.
- **Blocs cost Gold that poor empires lack.** The alliance upkeep fee kept India out at 4-20 Gold (AZ).
- **VP arrives from conquest and stacks with wonder and hold VP**, so a win lands at T116-T180 of 300. Pillar 11 says nobody should know the winner by T100; by the takeover curves the winner is known by T60-T80 in five of seven tapes, and the game ends before the late eras exist ("tech feels stuck in first age", HR).
- **Air ignores Walls** and nobody builds Fighters, so the one late brake is also his.

His own question, "how much from AI getting smarter vs making the leader weaker?" (HR): the evidence says AI first. Decadence at slope 45 was the largest single lever the coach measured (24 to 13 of 32) but it makes the leader weaker without making the game more contested; a rival that takes Solothurn does both.

## 5. The five changes most likely to make him lose

Ordered by expected effect on his next game. Estimates are wall-clock for agent work; the bottleneck in each is the measurement loop (designer-seat run 14 min on the Mac, human-seat gate and 4,096-game census on the box, about 1-2 hours a round) and landing reviews.

**1. AI: a coalition strike that goes for his VP cities and his hold, with an army that can win.** One operation: when the runaway read fires, the strongest wary neighbour ends or shelves its side war, upgrades, and marches on the leader's highest-VP reachable city (capital, Great Work city, hold city), not the nearest colony. Combines reasons 1, 2, 5. Evidence: 88% to 1% and 13 to 0 what-ifs; Solothurn never targeted; LAB 0/50 on four strike-the-leader cards. Existing pieces: `runaway.rs::second_front_why`, `operation.rs`, `siege_stance`. The gate should be those four cards plus the designer-seat rivals read, not the AI-only census (the lead already found the gate "can't see it", PROG). About 2-3 days, 3-4 measurement rounds; risk: executed strikes currently lose on average, so this depends on change 2.

**2. AI: an army that is parked must upgrade, merge or retarget; modernise before marching.** The Celts' 8 units at 3 tiles for 15 turns are the single clearest picture of the gap. Price upgrades and era resources for units ahead of buildings when a war is planned (W2 upgrade gene, `modernization_value`), and when a campaign forecast fails, spend Gold on upgrades or pick a target it can take (unwalled St. Gallen was right there). Evidence: Hard storms Therapne 49/50 once upgraded (AZ). About 1-2 days. This is the precondition for change 1.

**3. AI: protect Settlers and react to a visible conqueror in the first 45 turns.** Re-open idea #43 (it saved Germany 48/48) and add an early alarm: a neighbour with 5 units on the border and no Settlers means build defenders and Walls now, not Settlers and Wonders. This attacks the T25-T45 act that decides five of seven tapes; the Swiss Settler capture alone was +54 pp. About 1 day to re-measure #43 against current master, 1-2 more for the alarm (the earlier `conquest-alarm` branch measured no gain in the census, PROG; it needs testing on `aztecs-swiss-spearmen` and `sparta-next-door`, both 0/50, not on AI-only games).

**4. AI: Hard adopts his opening economy.** Early conquest of a 1-unit neighbour priced as expansion; walled colony wave at T45-T70; a Gold reserve held for door buys instead of garrison upkeep. This makes rivals arrive at T75 with 6-8 cities instead of 3-5, which is what makes changes 1-3 bite. Evidence: section 1 table; `aztecs-eight-by-seventy` 8%. About 2-3 days; highest regression risk for faction balance (pillar 8), so it needs the census ratio check.

**5. Rules proposals (frozen; for his decision at the AI checkpoint).** In order of my confidence *(inference)*:
   - **Alliance against the runaway costs no upkeep** (or the fee scales with treasury). Smallest change, directly unblocks blocs.
   - **A cost to fast conquest**: restore decadence at a slope that only bites above the median city count, or occupation unrest for N turns on captured cities. The measured lever exists (24 to 13 of 32).
   - **Victory pacing**: a hold that lengthens while a coalition is at war with the holder, or a VP threshold that scales with era, so a T116 finish needs a contested hold.
   - **Fighters or Walls interact with Bombers** before the air stage decides games again.
   Each is 0.5-1 day to build and one census plus human-seat round to measure. I would not ship any before changes 1-2: a weaker leader facing passive rivals is a longer dull game, not a closer one.

**What would not help next game** *(inference)*: more tactical search or breach work. Tactics cards moved (lab capture 29.8 to 41.7%) while the designer-seat score stayed flat and his experience did not change (PROG, 11-08 entries). The gap is in what Hard chooses to do with its army between T25 and T100, not in how it fights once committed.

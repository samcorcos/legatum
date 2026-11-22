# Why the Mongol seat wins about 3% of the human-seat benchmark (2026-10-21)

The number is one map, not a faction rate: `playtest-8.5-mongols-win` (4 empires, Standard speed; the seat is the user's Mongols, the rivals are a second Mongol AI, Sparta and Athens). Eight maps make the benchmark headline, so this map moves the overall score by about 0.4 points. Everything below is Hard in the seat, 128 salts unless stated, reseeded dice (`LEGATUM_HVH_RESEED_DICE=1`) unless stated, run on the cloud box. `mongol_probe.rs.txt` is the scratch probe (it replays the map and records the seat's wars, sieges, captures, losses, Engineers, Hall and VP per salt; copy it to `crates/sim/examples/mongol_probe.rs` to run it); `probe-runs.tar.gz` holds its per-salt output for every table here. No rule or AI code was changed.

## 1. The drop did not come with the Military Engineer

| State (commit) | Reseeded dice | Shared dice |
| --- | --- | --- |
| master just before the Engineer landed (`05431dfde`, 10-19) | 1/256 (0.4%) | 0/128 |
| Engineer and era curve merged (`61af75520`, 10-20) | 3/256 (1.2%) | 6/128 (4.7%) |
| allies (`762d41dd3`) / accelerating curve (`3c37f8359`) / before the Mongol Walls change (`7e08777d5^`) | 4, 1, 2 of 128 | |
| master today (`47f5aaf20`) | 6/128 (4.7%) | |
| land/athens merge (`d88af5ded`) | 5/128 (3.9%) | **23/128 (18.0%)** |
| land/transit merge (`88ea86d09`) | 4/128 (3.1%) | **27/128 (21.1%)** |

- The seat has been at 0 to 5% on this map in every reseeded run since VP v2 and the stronger Sparta AI on 2026-10-10 (ledger: 93.8% at `ab436dca`, 0.0% at `800db602`; `docs/learnings.md` "the stronger Sparta opponent took 8.5 Mongols from 60 to 0/64").
- The 16 to 22% readings (ledger rows `athens-ruins-1020`, `transit-1020`) exist only with shared dice, where all 128 salts of a map play one combat and movement sequence. The same commits read 3 to 4% with reseeded dice. I could not reproduce 21.9% on any master state, so the Engineer is not the cause and the Walls and Engineer change (about 1.5 points) is not the lever. Treat any single-map shared-dice reading as noise of about 20 points.

## 2. What happens to the seat (master, reseeded, 128 salts)

- **Sparta declares war on the seat in 128 of 128 games, on turn 18 to 20 (median 19).** The Spartan capital is 2 to 3 hexes from the Mongol capital; a Spartan Scout is beside it on turn 12 and a Ranger and Soldiers by turn 19. The seat has 1.5 units at T30 (Sparta 3.4), and its first city is lost on turn 22 to 26 in every game (median 24).
- Cities at T30 / T50 / T100 / T150: 1.2 / 1.1 / 1.4 / 1.6 (the user had 2 / 5 at T30 / T50). The seat loses 4.4 cities a game (498 of the 569 lost cities go to Sparta) and is eliminated in 73 of 128 games. Sparta wins 93, Athens 23, the other Mongol AI 12, the seat 6.
- **Captures:** 1.9 a game, 65 games with at least one, first at a median of T113; 190 of the 247 are from Sparta, most likely retakes of cities it lost to Sparta. Its first assault on a city comes at a median of T81. 91 of 128 games make any assault.
- **Walls:** 59% of Sparta's cities and 66% of Athens's are walled at T50 and 85% or more by T250, while the Mongols raise Walls only from the Medieval era (median T78). The seat's city battles are 49% against walled cities.
- **Engineers:** 2.8 built a game (76 of 128 games), the first at a median of T115, none lost.
- **The goal:** the Courier Hall (Open Road, 2 VP) is built in 6 of 128 games, at a median of T244; VP at T100 is 0.2 and at T150 0.6.

Cross-check from the 8.9.7 baseline (`docs/sim-results/hard-vs-human/baseline-8.9.7.md`, landed while this ran): the Mongols win 16.2% of balanced 8-player games (1.30x a fair share) and 76% of the 8.3 Switzerland duel, and their 3.9% seat rate on this map is the Spartan neighbour's doing. The Mongols are not a weak faction; this is a map the default Mongol AI plays badly.

## 3. Rule or AI play?

Same seat, same map, every faction played by Hard (64 salts, reseeded):

| Seat faction | Wins | | Seat opening (Mongols) | Wins |
| --- | --- | --- | --- | --- |
| Sparta | 23 (36%) | | Default Hard | 3 (5%) |
| Athens | 20 (31%) | | **Rush** | **19 (30%)** |
| Japan | 18 (28%) | | GreedyExpand | 3 (5%) |
| Germany | 18 (28%) | | Snowball | 0 |
| Byzantium | 15 (23%) | | Economic | 3 (5%) |
| Celts | 8 (13%) | | the user's Mongol playbook proxy (v8) | 0 |
| Mongols | 3 (5%) | | | |
| Switzerland | 2 (3%) | | | |

- Another faction in the same seat survives the same turn-19 war far better (Germany and Japan are at war at T19 and T25 and still win 28%). Only Switzerland does as badly. So the map is hard, but the Mongols are 5 to 6 times worse than the median faction on it.
- **The decisive AI-play cause is the opening.** The Mongol seat on the default Hard opening has 1.5 units at T30, builds a Settler for 17 turns beside a Spartan army and loses its cities. The `Rush` opening wins 30% (15 of the 64 games capture a Spartan city by T60 despite 59% of them being walled), the same as Germany, Japan and Athens in the seat. Every expansion or economy opening stays at 0 to 5%. Rush is the line the Mongol rules are built for (+1 move, slip past enemy units, the Keshig's charge and bow), and the AI does not play it: nothing in the default Hard policy chooses a rush for the Mongols when a rival capital is three hexes away.
- **The Walls and Engineer gate is a small rule cause.** With Walls and Engineers from the start (`MONGOL_WALLS_ERA` 0, scratch) the seat wins 12 of 128 (9.4%) against 6 of 128 (4.7%): +4.7 points, not significant, and it loses more cities (5.8 against 4.4), because the first city falls on T24 before any Wall is built. The early gate costs the Mongols at most about 5 points here.
- **A gene tweak has not moved it:** the ledger (Hard and Hard + Search) has never shown the Mongol seat above 8% under reseeded dice since 10-10 (7.8% once, on 64 salts), so this is not something a gene tweak fixes (the Hall-phase, capital-guard and contested-site genes from 10-10 and 10-11 all read 0 to 3%).

## 4. Recommendations (pillar 8: deepen drawbacks or make a weak identity pay off, never weaken strengths)

1. **AI, not a rule (first):** have the Hard Mongol AI open with `Rush` when a rival capital is within about 8 hexes at contact, and keep the default opening otherwise (gene on `hard_v2::policy`, measured through the AI-impact gate: seat-only and opponents-only, plus the war suite). On this map it is worth about +25 points on one of eight maps (about +3 on the headline); it must also be measured on maps without a close neighbour (not run here) so that the Mongols do not rush an empty field. It also serves pillar 10 (a real rush opening, with a counter in Walls and an early defender).
2. **No rule change is justified yet.** The Mongol drawback (no terrain defence, no Walls before Medieval) already pushes the faction toward the rush, which is exactly the pillar-8 shape; the data say the AI refuses the push, not that the drawback is too heavy. Do not remove or shorten it, and re-measure the faction after the AI change.
3. **If a rule is still wanted after (1), pay off the weak identity, not the drawback:** the Courier Hall goal is almost dead (6 of 128 games, median T244) because a 300 Production building cannot start in a one-city empire. Candidate for the user: let a captured city or a held Market count toward an earlier, cheaper Hall (the 10-10 gene that moved the Hall phase earlier measured 0). Estimate unmeasured; the size is bounded by section 3, since a seat that survives to T100 has only 0.2 VP today.
4. **Benchmark hygiene:** report the Mongol (and any single-map) reading only with reseeded dice and at least 256 salts, and put the shared-dice number in the footnote; the shared-dice 18 to 21% rows in the ledger are a dice artefact, not a recovery.

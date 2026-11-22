# Aztec playbook for Hard (gene `aztecs_playbook`, 2026-11-05)

Standing rule (user, 2026-11-05): every win yields the counter and his faction playbook for Hard. His two Playtest 8.9.22 Aztec wins (game 1: T181, 11 VP to the Celts' 8; game 2: T153, 11 to 4) taught rivals counters and fixed generic Hard valuations; nothing turned his Aztec plan into how the AI Aztecs play. This folder is that playbook, what the AI Aztecs already do, the gene, and its receipts.

**Result.** Gene `aztecs_playbook` 7 (Walls first in frontier cities, the New Fire as the closer, a lean war army) takes the AI Aztecs from 13.9% to 16.5% of eight-empire Hard AI-only games (paired +2.66 pp [+1.32, +4.00], held-out seed). Nearly all of it is the lean war army. His opening (Spearmen first, eat the nearest neighbour) made the AI Aztecs weaker in every form tried. Hard playing his seat still can't follow his game from the opening (0-3 of 64). Default stays 0 for the lead's decision.

Sources: the coach reviews (`docs/playtests/reviews/2026-11-01-8.9.22-aztecs.md`, `-aztecs-2.md`), `docs/playtests/human-reasoning.md`, and his recorded orders (both tapes replayed hash-exact at `playtest-8.9.22`, 181 and 153 turns, 0 mismatches; states exported with `export-states.sh`).

## Step 1: his Aztec plan, game by game

| | Game 1 (8 empires, his neighbours: Mongols 7 hexes, Sparta 10, Germany 11) | Game 2 (his neighbours: India 7 hexes, Sparta 8) |
|---|---|---|
| Opening | T2: queue of Spearmen (`Soldier`) in the capital; no Settler until T46; declares on Switzerland T19 and Germany T23 and takes a Settler of each (T21, T23); takes Sarai, the Mongols' only city, with four Spearmen T36 | T2: queue of Spearmen; declares on Switzerland T18 and India T23; India eliminated T40; Spearmen again from T27 to T47 |
| Cities (his / Hard in his seat from T2, median of 64) | T50 3 / 3, T100 7 / 3, T150 12 / 3, T175 21 | T50 3 / 2, T100 9 / 2.5, T140 14 / 2 |
| New cities | Walls first in every new or captured city (T30, T37, T54, T63, T70, T81, T94, T106: 14 Walls orders), then Market, Granary, Laboratory, Workshop | Walls first again (12 orders), then Market, Granary, Laboratory, Workshop |
| Expansion | Settler waves once the first conquest is held: queues of 2-3 Settlers from T46 (12 Settler orders) | first Settler T67, then waves (10 Settler orders); 9 cities at T100 |
| Army | 4 to 9 ground units all game for up to 21 cities; always the current era (Spearmen, Phalangites and Composite Archers, Troopers and Marksmen, Arquebusiers and Assault Troopers, Riflemen and Field Cannons; 19 upgrades); first to Flight (T153) and four Bombers | 3 to 6 military units; Riflemen from T137 |
| Gold | banks it (643 at T100, 1,469 at T160) and buys 4 things all game (a Rifleman, a Field Cannon, two Bombers) | buys one Rifleman all game |
| Wonders | none | none |
| War targets | the weakest neighbour each time: one-city Mongols (T36), Sparta down to two cities (T122-T124), Switzerland with Spearmen and a public Great Work (T147-T166), Byzantium's brand-new Great Work city (T172) | India (T40), Switzerland (T112), Byzantium's Heir in Corinth (T130, +2 VP) |
| Peace | takes it when it frees the army (Germany T63, Sparta T66 and T102, Switzerland T77 and T119); refuses it when the enemy is dying (Switzerland T135, T150, T160) | accepts Byzantium T104, refuses Switzerland three times |
| Flower Wars | 16 events, 16 population: nearly neutral (coach what-ifs) | 13 events, 13 population; off from T90 his own orders still win on T153 |
| The New Fire | never used | the closer: T144 takes him to 10 VP and starts the hold |
| VP | six era objectives, Largest Economy, two captured Great Works | four era objectives (capture 2 cities, gain 4 cities, gain 49 tiles, grow 25), three crowns, Byzantium's goal (the Heir), the New Fire |

**The plan in one line:** Spearmen first and eat the nearest one-city or weak neighbour by T40 (its capital is a city, a permanent capital VP since 2026-11-01, and Flower Wars population); Walls in every new city; then Settler waves behind a small current-era army; then the next weakest neighbour, and public VP objects (Great Works, the Heir) for the closing points; the New Fire only as the closer.

**What he skipped:** an early Settler before the army, a Granary or Archers opening, wonders, a big standing army, horses (the Aztecs can't), early purchases.

### Rules changed since 8.9.22 (which parts still apply)

- **Capitals are worth +1 permanent VP on first capture** (`capital-vp`, 2026-11-01): his early capital captures (Sarai, Agra) are now worth more than when he played. Applies, stronger.
- **Celts:** barbarians never attack them, no melee or cost penalty, sack Gold 30 per population (RULES_VERSION 11-12): changes the rival that broke his first hold, not his plan.
- **Walled ranged garrisons fire over terrain; aircraft rebase at any distance; Future-era tech tree:** sieges of walled cities are harder; his Bombers are easier to base. Applies.
- **Flower Wars (melee only, shared and defensive kills count) and the New Fire (30 population for 2 VP) are unchanged** since 8.9.22. Every part of the plan still applies.

## What the AI Aztecs do now (current master, `crates/ai`)

Measured three ways: the census (4,096 eight-empire Hard AI-only games, held-out seed 9311003, master 3b984418a, engine identical to this branch's base), Hard playing his seat on the current engine from his recorded positions (`seat-run.sh`, 64 salts), and per-turn traces of those runs (`seat_run --trace`: production diagnosis, Settler gates, plans).

| Playbook part | AI Aztecs today | Where |
|---|---|---|
| Spearmen-first opening against the nearest weak neighbour | **Missing.** Hard opens Granary, then Archers (game 1) or a Soldier and an early Settler (game 2); no early war of its own. The early-prize opening (melee first, an assault wing sized by the siege forecast, a valued conquest nominee with one city) exists but is switched on only for Sparta | `production.rs::choose_item_inner` (early-prize branch, `early_prize_opening`), `campaign.rs::wing_target`, `assault_wing_ready`, `sparta_opening_byzantium`, `conquest.rs::value_first_target` (1 = Sparta only), `params.rs::BUILTIN_FACTION_DECISIONS` |
| Conquest as expansion when free land runs out | **Missing.** In his seat the Settler gate `sites_ok` fails from T40-T60 (no free site within reach) and Hard sits at 2-3 cities to T150; Aztec AI-only seats average 4.5 cities at T100 | `production.rs` `SettlerGates` / `count_good_sites`; `knobs.rs` (expansion 75: target 6 cities) |
| Snipe exposed rival Settlers | Partly: the generic Snipe plan exists (`plan.rs` Snipe, genes `stalk_contact`, `snipe_read`) but never fired in the traces | `plan.rs::snipe_candidates` |
| Walls first in every new city | Partly: Walls only when threatened, at a border (8 hexes) or beside an aggressive neighbour | `production.rs::item_base_value` (`BuildingKind::Walls`) |
| Settler waves after the first conquest | Generic Settler gates and `colony_compound`; nothing ties them to a first capture | `production.rs::choose_item_inner`, gene `colony_compound` |
| Small current-era army, upgrades | Partly: army quota is cities + 2 at peace (Hard); in his seat 7-10 units for 3 cities at T100-T150; upgrades exist | `production.rs::uncapped_army`, `military.rs::hold_or_upgrade`, `modernization.rs` |
| No wonders | **Opposite:** Hard starts the Pyramids in a fresh pop-1 city at T44 with two cities (game 1) and Hagia Sophia; AI winners do take 2.2 VP a game from Great Works | `production.rs` wonder branch (`work_plan`, `missing_great_work`) |
| Weakest neighbour as the target | Partly: `value_first_target` scores public softness (last city, empty city, other wars) for Sparta only; the Aztecs use the default nominee | `conquest.rs::public_softness`, `conquest_city` |
| Peace when it frees the army; refuse it when the enemy is dying | Generic (`treaty_score`, gene `peace_siege_value` off) | `diplomacy.rs::treaty_score`, `wants_peace_why` |
| Public VP objects (Great Works, the Heir) as late targets | Generic capture value (`conquest_victory_value`); `defend-goal-objects` was rejected 11-01 | `victory.rs::conquest_victory_value` |
| Flower Wars | Already valued: melee kills, melee production near rivals | `tactics.rs::flower_wars_value`, `flower_wars_campaign_value`, `production.rs::flower_wars_build_value` (gene `flower_wars_value`) |
| The New Fire as the closer | Already: kindled when the 2 VP cross the target with time to hold, or late (75% of the game); never into an enemy army; "right not to kindle at 7 VP" (learnings 11-03) | `goal_actions.rs::swarm_now`, `rite_pays` |

## Step 2: which part of his plan is the lever

### Why Hard in his seat stays small

Per-turn traces of Hard in his seat (`seat_run --trace`, 4 salts a game) show three things his orders avoid:
- **No free land.** The Settler gate `sites_ok` fails from T40 (game 1) and T60 (game 2): no good site within reach. His growth came from rival cities and Settlers (game 1: two Settlers sniped T21 and T23, Sarai T36; game 2: India by T40) and then from the land those conquests opened.
- **Fresh colonies fall.** In game 2 from T40, Hard founds Tlacopan T69 and Xochimilco T118 without Walls; Sparta takes the first at T77, Athens the second at T123. The stock rule builds a garrisoned shooter first in a city that sees any rival city within 12 hexes (`production.rs::choose_item_inner`, deadline "rival city distance / 2 + 3" in `threat.rs::walls_beat_arrival`), and Walls first only after the opening's hand-off. He queued Walls first in every new city.
- **Production into a standing army and wonders.** From T40 Hard holds 9-12 units for 6 cities; from T2 it starts the Pyramids with two cities (game 1 T44) and Hagia Sophia. He held 3-9 units for up to 21 cities and built no wonder.

### AI-only screening (one arm at a time, 4,096 games, seed 9311003, paired with the 11-04 census at an identical engine)

Each arm changes the Aztec seats only (`faction_overrides` or the gene); every other seat plays stock Hard. Paired difference in Aztec win rate per seat, 95% interval over 3,623 Aztec seats.

| Arm | What his plan says | Change | Aztec win rate | Behaviour |
|---|---|---|---|---|
| A | Spearmen first, eat the nearest open neighbour | Sparta's early-prize bundle on the Aztecs (`early_prize_opening`, `assault_wing`, `open_followup`, `lone_city_strike`, `early_campaign_value` 160, `early_campaign_muster`, `conquest_min_cities` 0, `value_first_target` 2) | -1.32 pp [-2.72, +0.07] | capture by T60 9% → 36% of seats, first capture T112 → T81, but cities T50 3.11 → 2.48, T100 4.52 → 4.25 |
| B | the same, harder | A + `chain_press`, `walled_prize`, `sparta_force_campaign` | -1.55 [-2.96, -0.13] | as A |
| N | the same, Settler waves after T45 | A with `early_prize_turn` 45 | -1.08 [-2.49, +0.33] | cities T100 4.28 |
| C | the weakest neighbour as the target | `value_first_target` 2 | -0.11 [-1.25, +1.02] | no change |
| D | target plus a one-city strike, no production change | C + `conquest_min_cities` 0, `lone_city_strike`, `early_campaign_value` 160 | +0.06 [-1.12, +1.23] | no change |
| F | small army at peace | `army_hard_extra` 1 | 0.00 [-0.62, +0.62] | Aztecs are rarely at peace with no threat |
| G | Settler waves | `production_settler_percent` 130 | -0.41 [-1.44, +0.61] | cities unchanged: land, not production, binds |
| H | claim sites first | `settler_open_cities` 6, `colony_compound` 200 | +0.03 [-1.00, +1.05] | no change |
| M | settle marginal land | `settler_min_site_score` 40, `settler_max_site_distance` 14 | -0.08 [-1.16, +1.00] | no change |
| K | wars only on the weakest | aggression knob 65 → 30 | -0.39 [-1.57, +0.80] | no change |
| **I** | **a small field army at war** | **`army_war_extra` 5 → 2** | **+1.63 [+0.26, +3.00]** | early military share 42% → 36%, Great Works VP 0.89 → 1.01, era objectives 2.71 → 2.78, final VP 4.68 → 4.93, cities lost 2.00 → 1.91 |

Twelve arms on one seed: one interval excluding zero is what chance alone gives about half the time, so the winning lever was re-measured on a held-out seed with the gene (below).

## The gene `aztecs_playbook`

`crates/ai/src/params.rs`, default 0 while gating; bit flags, each on the Hard Aztec seat only, each a valuation from the seat's own `PlayerView` (no scripted orders, integer maths, deterministic):

| Bit | His plan | What changes | Code |
|---|---|---|---|
| 1 | Walls first in every new city | A city without Walls that sees an unallied rival empire's city within `open_prize_radius` (10) builds Walls before anything else (a Settler under way keeps its slot). Under the anticipatory defense, Walls come before the shooter unless a **visible** army arrives first: the stock deadline also assumes an army from the rival's nearest city ("distance / 2 + 3"), which no pop-1 colony's Walls can beat, and waited for the opening's hand-off | `production.rs::aztec_frontier_walls`, `aztec_walls_playbook`; `threat.rs::walls_beat_arrival` |
| 2 | The New Fire as the closer (game 2, T144, at War with four empires) | When the rite starts the VP hold, only the cities it shrinks must be clear of an enemy army; the stock rite waits while any city has one within three hexes | `goal_actions.rs::swarm_now`, `cities_under_threat` |
| 4 | A small current-era army | A conquest or containment war wants one unit per city plus 1, not plus `army_war_extra` (5) | `production.rs::uncapped_army` (`AZTEC_WAR_EXTRA`) |

Level 7 is the whole playbook. A fourth bit (a lean guard under a visible threat, one per city plus the same field unit instead of the knobs' 1.7 a city) added nothing over bit 4 on the screening seed (+2.65 pp both) and was removed.

### Screening of each bit (seed 9311003, 4,096 games, paired with the 11-04 census)

| Arm | Aztec win rate per seat | Paired difference [95%] | Cities lost | Final VP | Early military share |
|---|---|---|---|---|---|
| control (census 11-04) | 15.21% | | 2.00 | 4.68 | 42.3% |
| bit 1, Walls first | 16.23% | +1.02 [-0.22, +2.26] | 1.90 | 4.89 | 41.3% |
| bit 2, the closer | 15.32% | +0.11 [-0.12, +0.34] (fires in 18 games; New Fire 19.0% → 20.5% of seats) | 2.00 | 4.68 | |
| war extra 2 (`army_war_extra` override) | 16.84% | +1.63 [+0.26, +3.00] | 1.91 | 4.93 | 36.4% |
| war extra 1 (= bit 4) | 18.08% | +2.87 [+1.51, +4.23] | 1.91 | 5.03 | 34.3% |
| war extra 0 | 17.86% | +2.65 [+1.31, +3.99] | 1.92 | 5.01 | 34.2% |

Arms W1c, X2 and Y12 ran the gene at the commit of their day (`screening/`); the others are `faction_overrides` on master's AI.

### Primary: AI Aztec win share, gene 7 against 0 (held-out seed 8241001, after merging master's Swiss and Celts playbooks)

4,096 eight-empire Hard AI-only games per arm, the same seeds, `legatum-sim --games 4096 --seed 8241001 --difficulty hard` with and without `{"decisions": {"aztecs_playbook": 7}}` (filler batches `az-hold-ctlm-1105`, `az-hold-g7m-1105`; `heldout-8241001.md`, the two reports `heldout-*-report.md`). This seed was never used in the screening.

**Aztec win rate per seat: 13.89% (506 of 3,644) → 16.55% (603 of 3,644), paired +2.66 pp, 95% interval [+1.32, +4.00]** (Aztecs won 360 games only with the gene, 263 only without). The screening seed gave +2.87 for bit 4 alone and +1.02 for bit 1 alone.

Other factions, control → gene (pp): Athens 19.1 → 19.4 (+0.30), Mongols 15.9 → 13.8 (-2.10), Sparta 13.3 → 13.2, Celts 11.8 → 11.7, Byzantium 11.4 → 10.7 (-0.74), Switzerland 10.5 → 10.3, India 9.3 → 9.5, Germany 7.4 → 7.5. The Aztecs move from fourth to second; the best/worst ratio stays 19.4 / 7.5 = 2.6.

Behaviour of the Aztec seats (mean per seat, control → gene 7):

| Measure | Control | Gene 7 |
|---|---|---|
| Early military share of production (to T120) | 42.2% | 33.6% |
| Cities lost | 1.89 | 1.72 |
| Cities T100 / T150 / peak | 4.46 / 4.53 / 6.34 | 4.55 / 4.69 / 6.57 |
| Cities captured | 2.15 | 2.22 |
| Eliminated | 11.8% | 10.5% |
| The New Fire kindled | 17.5% of seats | 20.7% |
| Great Works VP / era objective VP | 0.89 / 2.63 | 1.03 / 2.84 |
| Final population | 40.9 | 43.2 |
| Final VP | 4.50 | 5.02 |

So the AI Aztecs win by what his small army bought him: production into buildings, objectives and Great Works, cities kept behind Walls, and the population to kindle the New Fire.

### Hard from his position (`seat-run.sh`, 64 paired salts a branch, cap T260)

Hard plays his Aztec seat from his recorded state at the branch turn; every rival is Hard with its real memory (`seat_run.rs`). Wins of his seat, control against gene 7, on identical salts:

| Branch | Before the merge (`seat-gene7/`) | After the merge (`seat-gene7-merged/`, Swiss and Celts playbooks on) |
|---|---|---|
| Game 1 T2 | 0 → 3 | 2 → 0 |
| Game 1 T60 | 16 → 10 | 18 → 29 |
| Game 1 T100 | 36 → 38 (median win T201 → T185) | 30 → 40 (T203 → T192) |
| Game 2 T2 | 0 → 0 | 1 → 2 |
| Game 2 T40 | 3 → 2 | 0 → 3 |
| Game 2 T68 | 5 → 4 | 4 → 2 |
| **All six** | **60 → 57 of 384** (McNemar p 0.81) | **55 → 76 of 384** (p 0.014) |

- **From his opening Hard still can't play his game:** 0 to 3 of 64 from T2 in both games, with or without the gene; 2 to 3 cities at T100 and T150 against his 7 to 9 and 12 to 14. The gene does not fix the opening: on his crowded maps his first cities came from neighbours, and the AI-only screening says a conquest opening costs the AI more than it brings (arms A, B, N).
- **From his mid-game positions the result moved with the rivals.** Before the merge the gene was neutral (60 → 57); after it, with the Celtic and Swiss playbooks on, it won 21 more of 384 (p 0.014, mostly game 1 from T60 and T100, where his Celtic rival is the one that broke his hold). Two paired runs that disagree this much say the in-seat effect is small and position-dependent; the AI-only census is the measure to trust.
- Control (no gene, before the merge) for T2/T40/T68 is in `control/`.

### Safety: the human-seat gate

`gate.sh` (seed 310001, 256 reseeded salts x 8 maps, plain Hard rivals; arms `gate-arms.json`, comparison `gate-comparison.json`, batch `aztecs-gate-1105-1104-235808`): control 470, rivals-only 470, seat-only 470 of 2,048, delta 0.00 [0.00, 0.00] in both arms. None of the eight benchmark maps seats the Aztecs, so a gene that acts only on an Aztec seat cannot change the benchmark; it measures nothing about this gene and is reported as the gate requires.

### Cost

About 20 census arms of 4,096 games on the box filler, a dozen seat runs (64 salts x up to 6 branches x 2 arms) and one spot server for the gate ($0.67 ceiling; it ran about 10 minutes): about 4 box-hours and under $5 in all.

## Next

- **Turn it on for the Aztecs** (`BUILTIN_FACTION_DECISIONS`, as the Swiss and Celtic playbooks were): +2.66 pp on a held-out seed, the human-seat gate cannot see it. The lead's call.
- **The lean war army may be generic.** The quota `cities + army_war_extra` (5) is the shared default; the Aztec screening found +2.65 to +2.87 pp at 0 to 1. The Swiss playbook found the same direction from lean garrisons. Worth one census arm per faction with `army_war_extra` 1, then the human-seat gate (rivals-only would show whether leaner rivals are weaker against him).
- **His opening stays a human edge.** On his maps the land was gone by T40-T60 and his first cities came from neighbours; the AI's conquest opening (Sparta's bundle) quadruples early captures but costs more cities than it takes. A better AI opening here needs the early land race the 11-02 work left open, not a rush.
- **Tools:** `crates/sim/examples/seat_run.rs` and `seat-run.sh` run Hard (with any params) in the designer's seat from any recorded turn of any decodable save, paired on salts, with optional per-turn production and plan traces; `tools/` holds the census extractor and paired comparison used here.

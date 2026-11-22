# The designer's Swiss playbook for Hard (2026-11-05, gene `swiss_playbook`)

Standing rule (user, 2026-11-05): every win yields the counter **and** that faction's playbook for Hard. AI Switzerland is near the bottom (6.9% win share in 4,096 eight-empire Hard AI-only games, held-out seed 9311003, census of 11-03; Athens 20.6%). The designer won easily with the Swiss in Playtest 8.9.24 (T161, 12 VP to 4, 40 active minutes), and earlier on 8.9.4 and 8.9.6. Before this, his Swiss games fed rivals' counters and generic Hard valuations (`decisive_buy`, `hoard_at_war`, `found_now`, `snipe_read`, `escort_buy`, `peace_siege_value`); nothing turned his Swiss plan into how the AI Swiss play.

## Step 1: his Swiss plan, part by part, and whether AI Swiss already do it

Sources: the 8.9.24 tape (`~/LegatumRuns/coach-tools/coach-8924-swiss/st/summary.jsonl`, every order he gave; review `docs/playtests/reviews/2026-11-02-8.9.24-swiss.md`, decision diff `docs/playtests/diffs/swiss-8.9.24-decision-diff*.md`). The older games are used only where today's Swiss rules match: **8.9.4** (10-18) and **8.9.6** (10-20) had the Redoubt, 3 Gold unit upkeep and uncapped interest as now, but a 1,000 Gold treasury goal (now the 5,000 Gold Reserve) and decadence on (off since 10-31). So from them I take only the army and walls habits (few units, Walls everywhere, Gold bought defenders at the door, 15 to 21 purchases a game, nearly all in wars), not the hoard timing.

AI numbers below: `swiss_playbook_probe` (this folder), 512 eight-empire Hard AI-only games, seed 9311003 on this branch's base (`90c9f995a`), Switzerland seated in 455; "other factions" is the mean of the other eight.

| Part of his plan | What he did (8.9.24 unless noted) | AI Swiss today (code path) | Verdict |
|---|---|---|---|
| **Opening** | Granary, a Ranger, then a Settler pipeline in Lugano (queues "Settler, Ranger" T10 and T21, "Settler, Settler" T34). Research Masonry, Currency, Writing, Irrigation, Bronze Working. 2 military units (the free ones) through T57. | Capital Settler chain exists (`production.rs::item_base_value`, `settler_chain`), but the climb (round 1, 10-24) set the Swiss `settler_chain_priority` to 0, `settler_min_site_score` to 84 (shared 60) and `production_settler_percent` to 85 (`params.rs::BUILTIN_FACTION_DECISIONS`). Masonry has no Swiss value (`research.rs`, about 70 like everyone). | **Partly missing**: the Settler chain is switched off for the Swiss |
| **Expansion** | 16 Settlers, in waves of 2 to 3 from developed cities (T34, T45, T84, T93, T123, T127, T138). 4 cities at T50, 9 at T100, 14 at T130, 22 at the end. | 2.85 cities at T50, 3.96 at T100, 4.24 at T150 (other factions 3.02 / 4.50); 3.7 Settlers trained by T100. | **Missing**: the Swiss expand slower than the average AI |
| **Walls and garrisons** | Every founded city's first order: Alpine Redoubt, then Market (then Granary, Workshop, Laboratory). Thun walled T40, Winterthur T45, Biel T50; 14 Redoubts. Lugano, the capital, only at T59, during the Mongol raid. 2 to 5 military units across 4 to 22 cities; 12 trained in 160 turns. | In founded cities the first completed item is a military unit 31%, a Granary 28%, the Redoubt 19% (median 29 turns after founding). The garrison floor wants one troop per city plus one at 3+ cities (`production.rs::item_base_value`, "garrison minimum", value 2,200) and the peacetime quota one per city (`uncapped_army`, `army_banker`). 6.0 military units at T100 (12 Gold a turn of Swiss upkeep beyond the two free). The Redoubt is worth 20 in a quiet interior city, 100+150 near a rival (`Walls` arm of `item_base_value`, `walls_near_enemy_bonus`). | **Missing**: the AI garrisons with troops; he garrisons with Redoubts |
| **The hoard** | Treasury never below about 400 after T26: 835 at T50, 1,280 at T100, 5,000 Reserve (+2 VP) at T146, 7,387 at the end; interest +61 a turn at T150. | 236 Gold at T50, 540 at T100; the Reserve banked in 11.4% of games, median T161 (`victory.rs::saving_for_goal`: the 5,000 is the purchase reserve from two cities on). | **Partly**: the AI does save, but its income is smaller (fewer cities, more upkeep) |
| **Gold as the army** | 7 purchases all game: two Composite Archers T58/T59 that broke the Mongol siege of unwalled Lugano, two Assault Troopers T113 (after the T107 declarations), an Arquebusier T122, a Bomber T140 (the turn after Flight), a Tank T157. 8.9.4: 4 emergency units T163 to T190; 8.9.6: 21 purchases, most in one war. | Door and forecast rescues spend the reserve (`door.rs::door_gold_floor`, `production.rs::buy_emergency_defender`); `hoard_at_war` 2 and `decisive_buy` 1 spend it at war (shipped from this game). 1.7 military purchases a seat by T100 (304 Gold). | **Done** (earlier genes); this branch leaves it alone |
| **Signature and goal** | Markets second in every city (tolls +1 Gold per 2 population, so Granaries too); Reserve held from T146 (+2 VP, 2 of his last 8 points). | `swiss_market_priority` 300 and `toll_market_bonus` 60 (`production.rs`), the Reserve as the reserve (`victory.rs`). | **Done** |
| **War targets and timing** | One opportunistic declaration (T41, an unescorted Mongol Settler: it became Biel), Peace taken T43. Defended the T54 to T61 Mongol raid with Gold. Then no war of his own until he had Flight (T139): one bought Bomber, a Bomber and Tank wave from city queues (T142 to T156), the Mongol capital (+1 VP) and a captured Great Work (+2). Refused Mongol Peace while besieging (T147). | `settler_stalk`/`snipe_read` (snipes), `decisive_buy` (the Bomber), `peace_siege_value` (opt-in) all shipped; research and air are generic. | **Done or opt-in** |
| **What he skipped** | No standing army, no early Walls in the capital, no Wonders, no improvement purchases (the AI's T11 Quarry buy), no declarations he could not finish. | 0.6 improvement purchases a seat by T100 (34 Gold); troops trained to a per-city quota. | Covered by the garrison change |

**The biggest missing lever** is the Redoubt-as-garrison: his cities were walled and empty, the AI's cities are garrisoned by troops that cost the Swiss 3 Gold a turn each and whose Production he spent on Settlers and Markets. It also explains the slow expansion (a troop per new city before the next Settler) and the thin treasury. The expansion genes the climb lowered are the second.

## Step 2: gene `swiss_playbook` (Hard Switzerland only; default 0 while gating)

| Level | What changes | Code |
|---|---|---|
| 1 | **The Redoubt is a city's garrison.** A city with a Redoubt standing, under way or buildable needs no troop of its own: the garrison floor (value 2,200 a missing troop) and the army quotas (peacetime `army_banker`, and the threatened-empire formula) count only the other cities, never below the two free units. A founded city's Redoubt is priced as the troop it replaces (the same 2,200 tier). A city facing a border threat raises its Redoubt before a shooter (Gold buys the shooter if an attack comes: `buy_emergency_defender`, door defense). Masonry gets the foundation value (100, inside the faction's military research weight). The capital keeps its usual Walls price (he walled Lugano only under attack). | `production.rs::redoubt_garrison`, `troop_garrison_cities`, `uncapped_army`, `choose_item_inner` (anticipatory defense), `item_base_value` (garrison minimum; `Walls` arm), `research.rs` (Masonry) |
| 2 | Level 1, and the Swiss expansion genes the climb set below the shared defaults play the defaults: Settler chain priority 0 to 500, site floor 84 to 60, Settler weight 85 to 100. A walled colony no longer costs a troop, so expanding is cheaper than when the climb tuned them. | `params.rs::DecisionParams::for_faction` |

All value scoring; deterministic integer math; read-only `PlayerView` (a city's `walls` flag is public).

## Measurements

Binaries built on the box from this branch (levels 1 and 2 are the same code in every run below; the tried-and-removed variants are listed at the end). Spend: about 0.9 spot server-hours and 40 box-minutes, under $2.

### Primary: AI Swiss win rate, eight-empire Hard AI-only (held-out seed 9311003, 4,096 games, paired seeds)

`legatum-sim --players 8 --difficulty hard --games 4096 --seed 9311003 --ai-params params-N.json` (batches `census-v2-{0,1,2}-1104-225049`, binary `e3df2e206da5124e`, 0 panics). Win rate per seat; the interval is a bootstrap over the 3,610 games with a Swiss seat ([census-comparison.json](census-comparison.json), [census.py](census.py)).

| Faction | Off | Level 1 | Level 2 | Paired Δ at 1 (pp) | Paired Δ at 2 (pp) |
|---|---:|---:|---:|---:|---:|
| **Switzerland** | **6.37%** | **10.22%** | **11.58%** | **+3.85 [+2.74, +4.96]** | **+5.21 [+4.02, +6.34]** |
| Athens | 20.86% | 20.58% | 20.50% | -0.27 | -0.36 |
| Mongols | 15.57% | 15.08% | 14.45% | -0.49 | -1.12 |
| Aztecs | 15.21% | 14.44% | 13.69% | -0.77 | -1.52 [-2.73, -0.25] |
| Sparta | 14.14% | 13.98% | 14.04% | -0.16 | -0.11 |
| Byzantium | 12.75% | 12.56% | 12.09% | -0.19 | -0.66 |
| India | 10.37% | 9.50% | 8.95% | -0.87 | -1.42 [-2.48, -0.38] |
| Celts | 9.67% | 9.42% | 9.69% | -0.25 | +0.03 |
| Germany | 7.55% | 6.76% | 7.55% | -0.79 | 0.00 |
| Best / worst | 3.27 | 3.04 | 2.71 | | |

The Swiss go from second-last to sixth of nine at level 2; the share they gain comes a little from everyone (no other faction moves by more than 1.5 points). Swiss cities 2.83 / 4.03 at T50 / T100 off, 3.05 / 4.41 at 2; first Redoubt median T38 to T32; Redoubts by T100 3.6 to 4.5; eliminated 12.7% to 7.7%; treasury peak 2,108 to 2,977.

### Behaviour: does it play his plan? (`swiss_playbook_probe`, 1,024 AI-only games, seed 9311003; Swiss seated in 910)

[probe-ai-only-l1.json](probe-ai-only-l1.json), [probe-ai-only-l2.json](probe-ai-only-l2.json), [analyze.py](analyze.py). His 8.9.24 numbers for reference (5-empire map, so city counts are not comparable one to one).

| Swiss seat | Off | Level 1 | Level 2 | His 8.9.24 |
|---|---:|---:|---:|---:|
| First thing a founded city completes: Redoubt / troop | 18% / 31% | **47% / 18%** | **46% / 19%** | every city: Redoubt |
| Redoubt this many turns after founding (median) | 29 | **16** | **16** | 5 to 10 |
| Cities walled at T60 / T100 | 61% / 83% | 81% / 94% | 79% / 94% | 3 of 4 at T50 |
| Military units at T50 / T100 | 2.70 / 6.10 | 2.26 / 5.29 | 2.14 / 5.19 | 2 / 2 to 5 |
| Military trained by T100 | 6.9 | 5.6 | 5.4 | about 6 |
| Treasury at T50 / T100 | 237 / 560 | 257 / 810 | 259 / 849 | 835 / 1,280 |
| Cities at T50 / T100 | 2.90 / 4.04 | 3.03 / 4.29 | 3.02 / 4.42 | 4 / 9 |
| Settlers trained by T100 | 3.77 | 3.94 | 4.23 | about 8 |
| Reserve (+2 VP) banked; median turn | 13% ; T158 | 25% ; T150 | 26% ; T149 | T146 |
| Eliminated | 12.2% | 7.6% | 6.8% | no |
| Win share in this probe | 6.5% | 13.2% | 13.5% | won |
| Paired Δ win share (pp) | | +6.7 [+4.3, +9.1] | +7.0 [+4.6, +9.3] | |

The garrison lever fires as intended: Redoubts first and twice as fast, one troop fewer at T100 (3 Gold a turn), 250 to 290 more Gold at T100, half the eliminations, twice the Reserve. Expansion moves only a little (+0.4 cities at T100 with level 2): on a crowded eight-empire map the land, not the troop, is the limit (as found for Germany on 11-02).

### Hard from his 8.9.24 opening position

**His exact start, on the recording engine.** The gene ported onto `playtest-8.9.24` (the same diff, plus the gene line), the decision-audit tool (`tools/decision-audit` steps run by hand on the box: tape from his save, 160 tape turns, 0 hash mismatches, 0 of 633 AI order mismatches), Hard (or Hard with the gene) in his seat from the branch turn, every rival Hard, 64 reseeded runs per row, scored at T161 by the win-chance estimate (nobody finishes by T161; positive = better for his seat). Receipt: [his-seat-8924.json](his-seat-8924.json). "His T1 orders" keeps his turn-1 orders, then the brain plays; "Hard's T1 orders" is Hard throughout.

| Branch | Hard | Level 1 | Level 2 | Paired Δ at 1 | Paired Δ at 2 |
|---|---:|---:|---:|---:|---:|
| T1, Hard's T1 orders | 12.4% (23 of 64 lose every city) | 11.6% (21) | 13.0% (17) | -0.8 [-6.5, +5.2] | +0.5 [-5.2, +6.7] |
| T1, his T1 orders | 16.6% (3) | 28.4% (4) | 36.4% (3) | +11.8 [+1.6, +21.9] | **+19.8 [+9.1, +30.5]** |
| T40, Hard's orders | 33.2% | 36.2% | 43.9% | +3.1 [-5.8, +12.0] | **+10.7 [+1.1, +20.3]** |
| T50, Hard's orders | 87.9% | 94.1% | 92.3% | **+6.2 [+2.0, +10.8]** | +4.4 [+0.0, +9.1] |

Cities at T161 with his T1 orders: 5.0 (Hard), 5.6 (1), 6.3 (2). **The whole T1 difference is his research queue.** A category swap at T1 (same runs) moves the result with research and not at all with the unit moves: his queue (Masonry, Currency, Writing, Irrigation, Bronze Working) against Hard's (Irrigation, Currency, Bronze Working, Writing, Masonry) is 12.4% → 16.6% for Hard and 13.0% → 36.4% with level 2, and it cuts total losses from 17-23 of 64 to 3. The queue persists until it is spent, so a T1 queue decides the first five techs. On his map the playbook pays only with the Redoubt researched first.

**His map on the current engine** (`swiss_playbook_probe --setup 8924`: his setup, every seat Hard, 512 reseeded runs; today's start placement puts the Swiss on another tile of the same map, so this is his map, not his start): Swiss win share 0.6% → 1.6% (+1.0 [-0.2, +2.3]) at both levels; losing every city 32% → 21%; cities at T100 4.5 → 5.1; Mongol wins 62% → 56%. Military still comes first in 31% of founded cities here: the Mongols are at the door, and the threat path trains the defender ([probe-his-map-l1.json](probe-his-map-l1.json), [probe-his-map-l2.json](probe-his-map-l2.json)).

### Safety: the human-seat benchmark gate

`hard_vs_human --arms`, seed 310001, 256 reseeded salts × 8 maps (2,048 games an arm), plain Hard rivals, one spot server (batch `swiss-playbook-hvh-1104-225924`, 0.23 server-hours, binary `8906b43af40b8d03`; [arms.json](arms.json), [comparison.json](comparison.json) by `tools/sim/quality_gate.py::compare` via [compare.py](compare.py)). Control: gene 0 in every seat, 470 seat wins.

| Arm | Seat wins | Δ seat win rate (pp) [95%] | quality_gate |
|---|---:|---:|---|
| Level 1, rivals only | 467 | -0.15 [-1.53, +1.24] | noninferior |
| Level 1, seat only | 492 | **+1.07 [+0.38, +1.77]** | noninferior (stronger seat) |
| Level 2, rivals only | 450 | -0.98 [-2.32, +0.37] | noninferior (rivals a little stronger) |
| Level 2, seat only | 485 | +0.73 [+0.08, +1.38] | noninferior |

Only the two Swiss maps move in the seat-only arms (8.3 Switzerland 8 → 15 at 1, 8 → 12 at 2; 8.5 Switzerland 11 → 26 and 11 → 22); in the rivals-only arms only the maps with a Swiss rival move (8.4 Japan 55 → 53 at 1, 55 → 41 at 2). Neither level makes the rivals weaker.

### Tried and removed (measured, not shipped)

- **Masonry first outright** (its playbook value outside the Swiss military research weight, 200): Masonry was the first tech in 62% of Swiss games instead of 18%, but AI-only win share fell 2.5 points against level 1 [-4.7, -0.4] (`probe-ai-only-masonry-first.json`).
- **His whole research opening** (Masonry then Currency first, unconditional): -3.7 pp [-6.3, -1.3] against level 2 AI-only, -0.2 on his map. **Only with a rival city within 10 hexes**: +0.4 [-1.1, +2.0] AI-only, -0.2 [-0.6, 0.0] on his map. Neither helps on today's engine; the his-seat result above stays the open question.
- **A walled city of 3 counts as developed for a Settler** (his Thun, walled T40, two Settlers T45): no change (cities at T100 4.42 → 4.43, win share 13.5% → 13.1%).

## Decision and recommendation

Ship level 2 as the Hard Swiss default (`BUILTIN_FACTION_DECISIONS`: `(Faction::Switzerland, "swiss_playbook", 2)`, or the shared default, which only the Swiss read): +5.2 points of AI-only win share, his-seat +19.8 points with his opening and +10.7 from T40, rivals not weaker in the benchmark, and the seat stronger on both Swiss maps. Left at 0 on this branch pending the lead's call; flipping it needs `tools/sim/repin.sh` (Swiss AI games in the pinned tapes change).

## What's next

1. **The research queue.** On his map his T1 queue is worth 4 points for Hard and 24 with the playbook, yet Masonry-first and his whole opening do not help AI-only on today's engine. Find out why: when Hard re-plans a standing queue, and whether his map's Mongols (7 hexes) are what makes Masonry first pay. A neighbour-distance or threat-timed Masonry rule is the candidate.
2. **Expansion on open land.** Level 2 adds +0.4 cities at T100 AI-only; his 9 cities at T100 came on a 5-empire map. Measure on 5- and 6-empire maps (the app's solo lobby is 6) before tuning further.
3. **Military still first in 31% of founded cities on his map**: the threatened-city path trains a shooter when an enemy is near. He bought archers only when Lugano was actually shot (T58). A Gold-first door defense for the Swiss is the next lever there.

# Settler capture priced by what the AI can see (2026-11-03, branch `claude/fix-settler-safety`)

From the coach's 8.9.25 India start review (`docs/playtests/reviews/2026-11-03-8.9.25-india-start.md`): three of the designer's five cities came from AIs in the first 22 turns. Germany walked its first Settler out of Leipzig with its only Soldier; India's two Soldiers killed the escort, took the Settler at T17 and walked into the empty capital at T22. The Celts sent their Settler out alone the turn after India's surprise war and lost it to his Scout at T18.

## What Hard did at the decisive turns

Fixtures cut from his tape (`crates/ai/src/fixtures/india-1103/`: each AI seat's recorded view and its Hard memory, warmed by shadowing the tape with the app's seeds; all reproduce the recorded orders with the genes off).

| Turn | Seat | What it saw | What it did | Why |
|---|---|---|---|---|
| T12 | Germany | Spartan Soldier 2 hexes from Leipzig | Settler east, the only Soldier with it | escort path: a garrison may escort while the city has no threat at war |
| T13, T14 | Germany | Spartan, then one Indian Soldier 3 hexes off | trip south-west to (11,27), escorted | one escort holds a tile against one attacker; this was reasonable on what Germany could see |
| T15 | Germany | two Indian Soldiers, one beside the Settler | Settler turns home, **escort stands still** | the escort pass skips a Settler whose own tile is unsafe |
| T17 | Celts | no Indian unit or city in sight, India not yet met; India declared on Germany at T16 | Settler walks on alone to (18,9) | a surprise war was priced at the flat 10% prior, and nothing in sight |
| T18 | Celts | India's Scout beside the Settler | founds there (captured first) | site search counts a Peace Scout as harmless |
| T2-T4 | Germany | nothing (no rival unit seen until T8) | bought a Pasture and a Mine with 140 Gold | a second troop (160 Gold) needs a visible army to be worth more than the Settler risk |

## Genes

| Gene | Default | What it does |
|---|---|---|
| `snipe_read` | 90 (on, lead decision after the single-gene what-if) | A rival whose team alone declared a running war within 20 turns is a known sniper: surprise-war chance 90 (founding in place, trip risk), its Scouts mark settler danger; a trip past unseen tiles is threatened while such a rival is not known to be far (12 hexes; never seen counts), so it takes an escort or waits up to two turns for one; a Settler waits for an escort that cannot reach its stopping tile; two or more rival troops in reach of an escorted Settler's stopping tile send it home; a Settler falling back into a city keeps its escort beside it. |
| `capital_guard` | 0 (off), tested at 1 | While a rival army (two troops of one empire that can declare or is at war, or one of an empire at war or with a recent surprise war; no barbarians) stands within 3 of the capital, the lone defender does not escort and a threatened Settler without escort stays home; an empty capital calls Settlers within 4 home. |
| `escort_buy` | 90 (on) | With at most one troop, a Settler out or in production and a rival troop within 6, buy the cheapest melee troop worth at most 90% of the Settler's quote, before routine purchases; hold its price if it is 5 turns of income away. |

## Gate 1: decisive turns

`crates/ai/src/india_settler_tests.rs` (genes on): T12-T14 Germany's Soldier walks on the Settler's path (escorted, as recorded); T15 the escort walks home beside the Settler (recorded: it stood still); T14 with a second Indian Soldier added two hexes from Leipzig, Settler and Soldier go home; T17 the Celtic Settler waits (`SkipUnit`) while Lugdunum's quiet Soldier joins it, so India's Scout meets an escorted Settler at T18; at T17 India reads as a sniper (90), the others 10, and at T16 India is 10; at T13 with 300 Gold Germany buys a second Soldier, with its real 70 nothing.

The first version of `capital_guard` (any single troop near the capital) sent Germany's Settler home at T12-T14. It failed the Lab check below, so it was narrowed; T12-T14 now keep the recorded escorted trip, which is what Germany could justify from its view.

The coach's what-if ([whatif-summary.txt](whatif-summary.txt), branches [whatif-branches.json](whatif-branches.json), 48 runs each, `coach_india_whatif` on this branch):

| Branch | His cities / Production at T37 | Germany alive T37 | Notes |
|---|---|---|---|
| His orders from T12, master genes | 4 / 59 | no (Leipzig to India T22) | reproduces the game |
| His orders from T12, new genes in every AI | **1 / 14** | yes | German and Celtic Settlers kept (both still on the map at T22), Leipzig not taken by India (Sparta takes it at T35; Germany lives on its second city). His tape breaks at T15, so his later orders are not his reply to the new AI: directional. All 48 runs are identical (the tape branch has no dice to vary). |
| Hard in his seat from T1, master | wins 12/48; AI cities T37/T50/T100 2.32/2.83/4.80 | 48/48 | |
| Hard in his seat from T1, new genes everywhere | wins 12/48; AI cities 2.28/2.73/4.70 | 46/48 | slightly fewer AI cities (noise-sized) |

Single genes on his orders from T12 ([whatif-summary-single.txt](whatif-summary-single.txt), [whatif-branches-single.json](whatif-branches-single.json); run after the rebase on master `fc8a229dd`, whose replay already differs from the tape at T12 in the Celtic seat, so the reference reads 46 Production, not 59):

| AI genes (with `escort_buy` 90 in each) | His cities / Production T37 | German Settler | Celtic Settler | Leipzig |
|---|---|---|---|---|
| `escort_buy` only (the shipped baseline before this decision) | 4 / 46 | taken T17 | taken | India takes it T22, Germany eliminated |
| `snipe_read` 90 alone | **1 / 14** | kept (on the map T22) | kept (on the map T22) | Sparta takes it T35; Germany lives on its second city |
| `capital_guard` 1 alone | 4 / 46 | taken | taken | as the baseline: identical game |

`snipe_read` alone does all the work on his game; `capital_guard` never fires there (one Indian troop at a time near Leipzig until the war).

## Gate 2: Scenario Lab `india-take-the-settler`

Seat objective (take Leipzig in 12 turns), Hard in India's seat: 0/200 with master genes, 0/200 with the new genes (Hard never declares). The AI's side ([scenario-ai-side.txt](scenario-ai-side.txt); every seat Hard, 200 runs, 24 turns, temporary example `india_scenario_side`):

| Arm | Germany lost a Settler | Leipzig lost | Germany cities T36 |
|---|---:|---:|---:|
| master genes | 2 | 0 | 2.90 |
| `snipe_read` only | 3 | 0 | 2.88 |
| `capital_guard` only | 2 | 0 | 2.89 |
| `escort_buy` only | 2 | 0 | 2.90 |
| all three | 3 | 0 | 2.87 |
| first `capital_guard` (single troop) | 54 | 0 | 1.87 ([first run](scenario-ai-side-first-guard.txt)) |

The first guard froze Germany's Settler for twenty turns (a Spartan Soldier parked three hexes from Leipzig, the Settler stepped out and back), and the second Settler then walked out alone and was taken.

## Gate 3: human-seat benchmark

`hard_vs_human`, seed 310001, 256 reseeded salts x 8 maps (2,048 games per arm), plain Hard rivals, one spot server. Control `ctl`: all three genes 0, `settler_safety` 0, `peace_settler_capture` 0 in every seat; other arms set the remaining seats to the control (`other_params`). Delta: change in the designer's-seat win rate (rivals-only: negative means stronger rivals). [compare.py](compare.py).

Batch `settler-gate2-1103-225015` ([arms-2.json](arms-2.json), [comparison-2.json](comparison-2.json)), 0.30 server-hours:

| Arm | Seat wins | Delta pp [95%] | Rival cities T50 / T100 | Rival Settlers lost a game | Seat cities T50 / T100 |
|---|---:|---:|---:|---:|---:|
| ctl | 468 | | 3.27 / 5.35 | 0.481 | 2.85 / 5.28 |
| all three, rivals only | 492 | +1.17 [-0.98, +3.27] | 3.23 / 5.27 | 0.472 | 2.86 / 5.30 |
| all three, seat only | 462 | -0.29 [-2.00, +1.42] | 3.26 / 5.34 | 0.487 | 2.83 / 5.29 |
| `snipe_read`, rivals only | 481 | +0.63 [-1.51, +2.69] | 3.24 / 5.28 | 0.459 | 2.86 / 5.32 |
| `capital_guard`, rivals only | 484 | +0.78 [-0.73, +2.29] | 3.25 / 5.34 | 0.502 | 2.85 / 5.27 |
| `escort_buy`, rivals only | 468 | 0.00 [-0.49, +0.54] | 3.27 / 5.35 | 0.479 | 2.85 / 5.28 |

Re-gate of the existing genes, batch `settler-gate1-1103-222233` ([arms.json](arms.json), [comparison-1.json](comparison-1.json); same control, 468 wins; its "new" arms ran the first `capital_guard` and are superseded):

| Arm | Seat wins | Delta pp [95%] | Rival cities T50 / T100 | Rival Settlers lost a game |
|---|---:|---:|---:|---:|
| `settler_safety` 1 + `peace_settler_capture` 100, rivals only | 532 | **+3.12 [+0.83, +5.42]** | 3.15 / 5.13 | 0.546 |
| same, seat only | 378 | **-4.39 [-6.35, -2.54]** | (seat 2.71 / 4.96) | (seat 0.375) |

## Decision

- `escort_buy` 90 on: rivals not weaker (0.00), and it is right on the fixture; it rarely fires.
- `snipe_read` 90 on (lead decision, 2026-11-03): its benchmark cost (rivals-only +0.63 pp [-1.51, +2.69], T100 cities 5.35 to 5.28) is within noise, and the benchmark cannot see snipe defence because its Hard seat rarely snipes, while the designer sniped in most of his games (Germany 8.9.23 T16, Swiss 8.9.24 T41, India 8.9.25 T16-T18). Alone it keeps both Settlers on his T12 orders.
- `capital_guard` stays off: rivals +0.78 pp softer, and alone it changes nothing in his game.
- The flat pair stays off and is now clearly harmful: with `settler_stalk` in the benchmark seat, rivals lose more Settlers with it, not fewer (0.48 to 0.55 a game), expand less, and Hard in the seat is 4.4 pp weaker. The 10-24 verdict holds more strongly.

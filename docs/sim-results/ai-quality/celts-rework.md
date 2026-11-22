# The Celts rework: no homeland damage, no hidden hunters, a bigger sack (2026-10-23)

Branch `claude/celts-rework`, user decision of 2026-10-23 (decisions log; `legatum-factions` skill). Rules text: `game-rules.md` sections 4, 7, 8 and 10.

## What changed

1. **Deleted** (rules, AI and app, not commented out): Celtic homeland damage (enemy military on Celtic land lost up to 12 HP a turn, floor 10; `TerritoryAttrition`, `CELTS_TERRITORY_*`, the forecast term, the camp exemption `is_wild_unit`) and hunter concealment (`visibility::is_hunter`, `unit_visible_to`, `reveal_hunter`, `rehide_hunters`, `Unit::revealed_to`, `hunter_sightings`, the bump and reveal movement rules and `HunterBumped`, the Seen and Spotted badges and ghost markers, the AI's `reveal_cost`, last-seen memory and Celts-land avoidance, the gene `prey_skips_celts`). Celtic archers keep +3 ranged and +20 Production. Old saves load: removed fields are ignored on load.
2. **The sack grows**: it also steals one technology (the cheapest, for the Celts, of the technologies the victim knows that the Celts could research now; nothing if none) and raises a Celtic warband camp within 3 hexes of the ruins (a Call the Wild camp; it counts toward Raise the Warbands when it stands in a Celtic city's territory, first choice of tile when that city has no camp yet). `CityRazed` carries `tech` (private to the two empires) and `camp` (public).
3. **AI**: the stolen technology and the camp are priced in the sack value (`opportunity::sack_take_value`, `victory.rs::sack_camp_vp`, `price_campaign`, the strike value, `conquest.rs`); a rival clears a Celtic camp when its empire is within one camp of Raise the Warbands (`goal_sparta_wild::wild_camp_denial`). Every brain drops a move that ends in an ally's city (a latent bug a changed game exposed in `mixed_opening_orders_validate_in_real_fogged_games`).

## Measured (control: master, Celts already with the priced sack)

Balanced 8-empire Hard league, 2,048 games a seed (the Celts, 1 seat in each game):

| | Control | Rework | Paired |
|---|---|---|---|
| Seed 9210001 | 7.0% (144) | 7.6% (156) | +0.6 pp +-1.5 |
| Seed 9310001 | 8.2% (167) | 6.4% (132) | -1.7 pp +-1.4 |
| Both | 7.6% | 7.0% | -0.6 pp +-1.0 |

Celtic seats on seed 9210001 (1,024 tape replays): cities at the end 4.39 -> 3.92, eliminated 4% -> 9%, cities lost 0.99 -> 1.16 (1.21 in the replay), era-objective VP 2.83 -> 2.51, sacks per seat 2.0 (unchanged), technologies stolen 0.29 a seat (17% of sacks: the victim is rarely ahead of the Celts in their own era), warband camps raised 1.74 a seat. Other factions move by at most 1.4 pp.

Human-seat benchmark (8 maps x 128 salts, reseeded dice, binary against binary): the Celtic seat's map 26.6% -> 15.6% (-10.9 pp +-9.0); the seven maps without a Celtic seat move -0.4 pp over the eight-map mean [-3.0, +2.3], which includes the ally-city move fix. There is no Celtic rival on any benchmark map, so opponents-only is not separable. War suite: 47% -> 47% (7 of 14 at human parity both), the Golden Horde scenarios identical.

Reading: removing the Celts' land and hidden archers costs the seat what the sack's technology and camp do not give back; the league share moves by about a point (inside noise); on the human's map the seat is weaker. The user's decision stands, and the remaining Celtic lever is war selection (`celts-sack.md`).

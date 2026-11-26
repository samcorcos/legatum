# City upkeep is the catch-up lever that works against a runaway conqueror

2026-11-26. `rush_ladder` with rule experiments per arm (`x.name=value`). The arms below ran under the experiment names of that day (`city_upkeep` = each city costs cities-2, before the rename to `no_city_upkeep`, `city_upkeep_percent` and `city_upkeep_free`); on master the rule is on and the same arms are `x.city_upkeep_free=2` and `x.city_upkeep_percent=50`.

Why rivals lose to a runaway conqueror (two recorded Rome games, 10 cities at T80, replayed to T80 then all seats Hard): the map is full by T80, so cities change hands only by conquest; the leader runs up to two eras ahead (era 8 vs 6 by T180) and its attacks average 4.3x the defender's strength. Rivals' city counts stay flat at 2-8.

From T80 (Rome wins of 32 runs): no change 32; `city_research` (+5% research per city beyond two) 32; `city_upkeep` (each city costs cities-2 Gold) 22; both 18; upkeep at 50% 26, at 25% 32; first 6 cities free 20; 6 free at 50% 30.

From turn 1, 200 all-AI games (seed 7001): game end median 227 (none), 274 (full upkeep), 254 (6 free). Cities at T50/T100 unchanged (median 3/4). Research cost per city does nothing: a big empire out-researches anyway.

Faction balance (1,000 all-Hard 8-seat games each, seed 20261126; `docs/sim-results/2026-11-26-city-upkeep-{rule,off}.md`): game length median 224 -> 243. Win rate per seat without -> with: Celts 14.0 -> 20.4%, Germany 17.8 -> 18.9, Athens 16.3 -> 16.0, Aztecs 12.4 -> 13.5, Switzerland 10.6 -> 12.5, Byzantium 9.7 -> 11.5, Mongols 12.8 -> 9.4, Sparta 14.0 -> 9.3, India 5.0 -> 1.1 (ends with 2.5 cities instead of 3.7: the most cities to pay for, and non-capital cities remit no Gold before an Office). Shipped as is; India is a separate follow-up. Rome and Egypt are not in this runner's default faction pool, so this check does not cover them.

# City upkeep is the catch-up lever that works against a runaway conqueror

2026-11-26. `rush_ladder` with rule experiments per arm (`x.city_upkeep=1` etc.).

Why rivals lose to a runaway conqueror (two recorded Rome games, 10 cities at T80, replayed to T80 then all seats Hard): the map is full by T80, so cities change hands only by conquest; the leader runs up to two eras ahead (era 8 vs 6 by T180) and its attacks average 4.3x the defender's strength. Rivals' city counts stay flat at 2-8.

From T80 (Rome wins of 32 runs): no change 32; `city_research` (+5% research per city beyond two) 32; `city_upkeep` (each city costs cities-2 Gold) 22; both 18; upkeep at 50% 26, at 25% 32; first 6 cities free 20; 6 free at 50% 30.

From turn 1, 200 all-AI games (seed 7001): game end median 227 (none), 274 (full upkeep), 254 (6 free). Cities at T50/T100 unchanged (median 3/4). Research cost per city does nothing: a big empire out-researches anyway.

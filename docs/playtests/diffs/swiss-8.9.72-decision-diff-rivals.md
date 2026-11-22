# The five rival choices that most let him win

Each alternative comes from the rival's own view: its next scored production choices, legal buys, forecast-driven attacks, safe Settler route or escort, or a peace/war response. The designer follows his recorded later orders in both arms; rival seats continue with Hard. Positive designer win-chance Δ means the rival alternative cut his estimated chance at the recorded horizon; finished games use the actual outcome. The top five require a positive mean rival win-chance gain too, so they are plausible choices from that rival's own objective. Rival VP gain is a secondary check. Drift counts runs in which later recorded designer orders became invalid.

| Rank | Rival seat | Turn | Rival alternative | His win chance Δ pp [95%] | His VP Δ [95%] | Rival win chance gain pp [95%] | Rival VP gain [95%] | Drift runs |
|---:|---:|---:|---|---:|---:|---:|---:|---:|
| 1 | 1 | 69 | scored_production: SetProduction { city: CityId(2), item: Unit { unit: Soldier } } | +3.1 [-0.9, +7.1] | +1.0 [+0.6, +1.5] | +0.1 [-0.1, +0.4] | +0.4 [+0.0, +0.7] | 96/96 |
| 2 | 4 | 54 | forecast_attack: MoveUnit { unit: UnitId(10), to: Hex { q: 28, r: 20 } } | +0.4 [-0.3, +1.1] | -0.0 [-0.1, +0.1] | +1.5 [-4.9, +7.9] | -0.0 [-0.3, +0.3] | 96/96 |
| 3 | 4 | 53 | forecast_attack: MoveUnit { unit: UnitId(10), to: Hex { q: 27, r: 21 } } | +0.3 [-2.5, +3.1] | -0.3 [-0.6, +0.1] | +2.6 [-11.2, +16.4] | +0.4 [-0.2, +1.1] | 96/96 |
| 4 | 2 | 29 | settler_safety: MoveUnit { unit: UnitId(22), to: Hex { q: 7, r: 19 } } | +0.2 [-0.1, +0.5] | +0.7 [+0.4, +1.1] | +4.4 [-1.8, +10.6] | +0.6 [-0.1, +1.3] | 96/96 |
| 5 | 1 | 14 | buy: BuyItem { city: CityId(2), item: Improvement { improvement: Quarry, tile: Hex { q: 16, r: 7 } } } | +0.2 [+0.0, +0.3] | +0.5 [+0.3, +0.8] | +21.4 [+11.9, +30.9] | +1.4 [+0.7, +2.0] | 96/96 |

40 rival alternatives across 5 rival seats; 18 have positive mean rival win-chance gain. All options and raw paired results are in the JSON file.

# The five rival choices that most let him win

Each alternative comes from the rival's own view: its next scored production choices, legal buys, forecast-driven attacks, safe Settler route or escort, or a peace/war response. The designer follows his recorded later orders in both arms; rival seats continue with Hard. Positive designer win-chance Δ means the rival alternative cut his estimated chance at the recorded horizon; finished games use the actual outcome. The top five require a positive mean rival win-chance gain too, so they are plausible choices from that rival's own objective. Rival VP gain is a secondary check. Drift counts runs in which later recorded designer orders became invalid.

| Rank | Rival seat | Turn | Rival alternative | His win chance Δ pp [95%] | His VP Δ [95%] | Rival win chance gain pp [95%] | Rival VP gain [95%] | Drift runs |
|---:|---:|---:|---|---:|---:|---:|---:|---:|
| 1 | 3 | 59 | forecast_attack: MoveUnit { unit: UnitId(59), to: Hex { q: 27, r: 18 } } | +13.4 [+3.5, +23.3] | +0.8 [+0.1, +1.5] | +21.4 [+9.5, +33.4] | +0.8 [+0.0, +1.7] | 95/96 |
| 2 | 4 | 92 | scored_production: SetProduction { city: CityId(5), item: Unit { unit: Soldier } } | +2.8 [-5.6, +11.2] | +0.3 [-0.1, +0.7] | +2.6 [-0.1, +5.3] | -0.1 [-0.3, +0.1] | 95/96 |
| 3 | 3 | 37 | settler_safety: MoveUnit { unit: UnitId(38), to: Hex { q: 23, r: 14 } } | +2.3 [-1.7, +6.3] | +0.3 [-0.2, +0.7] | +3.9 [-2.4, +10.1] | +0.4 [-0.3, +1.2] | 95/96 |
| 4 | 2 | 40 | scored_production: SetProduction { city: CityId(6), item: Building { building: Workshop } } | +2.2 [-1.7, +6.2] | +0.4 [-0.2, +0.9] | +1.9 [-11.9, +15.6] | +0.1 [-0.6, +0.9] | 95/96 |
| 5 | 3 | 38 | settler_safety: MoveUnit { unit: UnitId(38), to: Hex { q: 25, r: 16 } } | +2.2 [-1.8, +6.2] | +0.2 [-0.2, +0.7] | +0.5 [-4.5, +5.5] | -0.4 [-1.1, +0.3] | 95/96 |

32 rival alternatives across 4 rival seats; 17 have positive mean rival win-chance gain. All options and raw paired results are in the JSON file.

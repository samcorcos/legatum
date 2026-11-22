# Egypt's Settler values land by its Nile Food (2026-11-22)

**Result.** Neutral on wins, safer on survival. With the change, Egypt wins 18.8% of eight-seat Hard games over two pools (400 Egypt seats), the same as master (18.8%; even is 12.5%). Egypt is eliminated less often: 5.8% against 9.0%. Rules 49, `legatum-sim`, scripted Hard in every seat, medium map, `--rotate-factions`.

**The change.** `settlement.rs::site_tile_yields` scores a site's land for an Egyptian Settler with the Nile applied: double Food and no Production. It covers the site score the AI plans with, its quality band and the Settler card's numbers. Before this, Egypt's Settler counted Production it never gets (Food ×4 + Production ×3), so Hills and Forest sites looked better than they are for Egypt, and Food land looked worse.

| Arm | Egypt seats | Egypt win rate | Eliminated | Cities T50 | Cities T100 | Cities lost / game |
|---|---|---|---|---|---|---|
| master (5f27e85 + #3, #5) | 400 | 18.8% | 9.0% | 2.69 | 4.87 | 2.29 |
| Nile site score | 400 | 18.8% | 5.8% | 2.71 | 4.83 | 2.23 |

By pool: pool 1 22.0% → 21.5%, pool 2 15.5% → 16.0%. Both moves are within noise.

Pools are the same as `egypt-nile-2026-11-21.md`:
- Pool 1: Egypt, Switzerland, India, Byzantium, Germany, Celts, Mongols, Sparta; seeds 7000-7199.
- Pool 2: Egypt, Athens, Aztecs, Rome, Switzerland, Germany, Mongols, Sparta; seeds 9000-9199.

Other factions in pool 1:
- Switzerland 10.5% → 5.5%.
- Sparta 4.5% → 12.0%.

Those are single-pool, 200-seat swings from different land being taken. They are not a direct effect of the change.

**Scale.** Egypt's doubled Food keeps the fixed Food ×4 weight. Averaged over Grassland, Plains, Hills and Forest, a tile scores 8.5 for other factions and 8 for Egypt, so the bands stay comparable on mixed land. An all-Grassland map rates higher for Egypt, which is true: Egypt gets more from it. The UI review saw most of a Grassland-heavy tour map in the top band. If Egypt's lens looks too flat in play, revisit the weight with sims.

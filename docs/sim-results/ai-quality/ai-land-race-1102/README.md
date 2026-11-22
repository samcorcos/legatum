# AI land race (2026-11-02, branch `claude/ai-land-race`)

Round 2 (`../ai-expansion-2-1102/`) found that Hard's remaining city gap on the designer's 8.9.23 Germany map (5.2 cities at T81 against his 9) is land: sites it does not know, and the land toward Byzantium that 4 of his 9 cities stand on. This round priced the land race behind genes and measured each one. None ships; both new genes stay at 0.

## Genes

| Gene | What it does |
|---|---|
| `land_contest` (new) | A preferred site also carries what founding there denies the nearest known rival city's empire: the gene's percent of the site's value, times how close the site lies to that city inside a rival Settler's reach (twice its city gap plus 2) (`civilians.rs::land_contest_value`). |
| `camp_land` (new) | A known barbarian camp is valued by the land behind it: the best known legal site within 3 and 3 points per unexplored tile within 3, this percent. Camps are cleared in that order, and up to 4 hexes farther out when the land is worth a site (`barbarians.rs::camp_land_value`). |
| `sentry` 2 (existing) | A Scout stands sentry by the nearest city of every bordering rival, not only rushers: exploration aimed at the neighbour whose Settlers matter. |

Existing genes tried for early pressure on his map (`expansion_probe`, 128 games): `early_campaign_value` 200 and 400 changed nothing (T81 cities 5.18 and 5.21; base 5.18).

## What the probe shows on his map (`expansion_probe`, all seats Hard)

- **The land toward Byzantium is gone by T20.** Byzantium founds its second city at T11 to T17 (median T17) and has 5.0 cities at T81. Of Hard Germany's foundings to T81, 0.02 a game lie closer to Byzantium's capital than to its own; the designer had 4.
- **Hard cannot see Byzantium's Settlers.** To T40 a Byzantine Settler is outside a city on about 9.3 turns a game. The seat sees it on 1.1 of them and 0.66 unescorted; on 0.9 turns a game it is unescorted within 6 of the seat's capital and unseen. The seat takes 0.07 rival Settlers a game by T40. His T16 snipe needed a unit watching that land.
- **War alone does not freeze a neighbour.** In the 16 of 128 games where the seat was at war with Byzantium by T30, Byzantium founded more cities (6.2 against 4.8 at Peace).
- **Few camps are known.** Within 10 hexes of the seat's cities: 0.27 known and 0.70 unknown camps on an average 10th turn. `camp_land` at 100 and 300 did not change a single game on his map.

| Arm (256 games, to T100) | Cities T50 / T81 / T100 (mean) | Seat Settlers lost a game | Foundings: mean distance to nearest rival city; share within 8 |
|---|---|---:|---|
| base | 3.58 / 5.17 / 5.84 | 0.42 | 7.25; 0.69 |
| `land_contest` 100 | 3.55 / 5.19 / 5.87 | 0.38 | 7.01; 0.73 |
| `camp_land` 100 | 3.58 / 5.17 / 5.86 | 0.42 | |
| `sentry` 2 | 3.78 / 5.33 / 5.90 | 0.28 | |
| all three | 3.79 / 5.35 / 5.98 | 0.29 | 6.78; 0.79 |

The behaviour check for `land_contest`: Hard's foundings move toward its neighbours, as meant (land_contest 200: 6.90; 0.75), but the cities it founds there are the same number. `sentry` 2's gain comes from the rivals' Scouts: they sentry instead of raiding, so the seat loses fewer Settlers.

## Human-seat gate

`hard_vs_human`, seed 310001, 256 reseeded salts x 8 maps (2,048 games per arm), plain Hard rivals, one spot server (0.69 server-hours), binary at `28373ce2`. Control: master defaults (`land_contest` 0, `camp_land` 0, `sentry` 1) in every seat. `_riv` and `_seat` set the other seats to the control through `other_params`. Delta: change in the designer's-seat win rate ([compare.py](compare.py), [comparison.json](comparison.json), [arms.json](arms.json)); on a rivals-only arm negative means stronger rivals. Batch `~/LegatumRuns/cloud/landrace-gate-1102-153155`.

| Arm | Seat wins | Delta pp [95%] | Rival cities T50 / T100 | Rival Settlers lost a game |
|---|---:|---:|---:|---:|
| ctl | 449 | | 3.06 / 5.31 | 0.53 |
| land_contest, rivals only | 435 | -0.68 [-2.54, +1.22] | 3.04 / 5.25 | 0.56 |
| land_contest, all seats | 420 | -1.42 [-3.52, +0.73] | 3.04 / 5.23 | 0.55 |
| land_contest, seat only | 440 | -0.44 [-2.15, +1.27] | 3.07 / 5.30 | 0.54 |
| camp_land, rivals only | 456 | +0.34 [-0.29, +0.98] | 3.06 / 5.31 | 0.54 |
| camp_land, all seats | 459 | +0.49 [-0.20, +1.17] | 3.06 / 5.31 | 0.53 |
| camp_land, seat only | 451 | +0.10 [-0.20, +0.44] | 3.06 / 5.31 | 0.53 |
| sentry 2, rivals only | 463 | +0.68 [-1.51, +2.93] | 3.06 / 5.18 | 0.50 |
| sentry 2, all seats | 452 | +0.15 [-2.05, +2.34] | 3.02 / 5.19 | 0.50 |
| sentry 2, seat only | 437 | -0.59 [-2.49, +1.22] | 3.05 / 5.36 | 0.57 |
| all three, rivals only | 442 | -0.34 [-2.49, +1.81] | 3.05 / 5.17 | 0.50 |
| all three, all seats | 465 | +0.78 [-1.46, +2.93] | 3.02 / 5.14 | 0.49 |
| all three, seat only | 437 | -0.59 [-2.54, +1.37] | 3.05 / 5.36 | 0.55 |

Every interval spans zero. `land_contest` points the right way for rivals (-0.68) but the wrong way in the seat (-0.44: Hard there wins less), so it is noise, not strength; `camp_land` is near a no-op (it acts on few games) and leans weaker; `sentry` 2 makes the rivals weaker. Nothing ships.

## The lever that remains

The designer's gain was one Settler taken at T16 from a neighbour Hard never watches. The open AI change is vision of a neighbour's settling lane in the first 20 turns (a unit placed where the neighbour's Settlers walk, priced by the expected snipe), measured on games where a neighbour founds within 8 of the capital. Generic sentries (`sentry` 2) and pricing land (`land_contest`) do not reach it.

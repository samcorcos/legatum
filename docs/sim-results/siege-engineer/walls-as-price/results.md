# Walls as a price: `walls_as_cost` and `engineer_for_walls` on (2026-10-20)

Branch `ai/walls-as-price`, from the coach's review of the user's 8.9.6 Athens win (`docs/playtests/reviews/2026-10-20-8.9.6-athens.md`, recommendation 1). Raw runs: `~/LegatumRuns/cloud/arms{4,7,8}-*` (human-seat benchmark arms), `~/LegatumRuns/box/*walls-whatif*` and the box job logs (tape branches).

## What changed

- Defaults: `walls_as_cost` 1, `engineer_for_walls` 1, `sally_engineer` stays 0.
- `engineers.rs`: an Engineer we would build is cover to plan on only if the faction can build it, the war builds them (`engineer_for_walls`) and the cities finish the whole force's cover within `COVER_PATIENCE` = 10 turns (`plannable_engineers`, all or nothing). `engineer_build_turns` prices the build time into the capture time (`forecast_siege`). `engineers_wanted` builds none for a force that is not there and no more than are plannable. `walls_are_a_price` (which lifts the early-game walled-target rule in `assessed_campaign_allowed` and the `WallsBeforeArrival` end) needs real Engineers or two plannable.
- Why each piece (found by measuring the genes alone on the 8.9.6 engine and the war suite):
  - Virtual cover for a faction that cannot build Engineers (the Mongols): `war-golden-horde-thin` fell from 95% to 51%.
  - Virtual cover with no build time and for small economies: the 8.4 Sparta map's seat fell from 88 to 29 wins of 128 (`walls_as_cost` alone 44; `engineer_for_walls` alone 80): the campaign kept marching on a walled city whose Engineers were 16 turns away and missed the open city next door (first Settler at T72 instead of T112, 1.0 cities at T50 instead of 1.8).
  - Partial cover (two Engineers of six) brought the same failure back: 88 -> 29 again; all or nothing gives 82.

## Measured

Coach's tape branches of his Athens game, his own orders, 30 runs each, 8.9.6 engine (`coach896_whatif`; the era landing changed the engine, so these cannot be replayed on master). Genes on = `walls_as_cost` + `engineer_for_walls`.

| Branch | Control | Genes on, 8.9.6 code | Genes on, this branch |
|---|---|---|---|
| T187, his orders | 30/30 | 0/30 (Sparta 27) | 0/30 (Sparta 11, Swiss 10, none 5); he loses 5.1 cities, the AIs build 24.8 Engineers |
| T200, his orders | 30/30 | 0/30 (Sparta 29) | 0/30 (Swiss 24, Sparta 5, Celts 1); he loses 7.9 cities (control 2.0), 23.7 Engineers |
| T200, Hard in his seat | 29/30 | 30/30 | 30/30 |
| T187 with `sally_engineer` too | | 0/30 | 15/30 (the AIs get weaker) |

Human-seat benchmark (8 maps x 128 salts, 1,024 games per arm, paired by map and salt, 95% bootstrap interval; seat wins):

| | Control | Seat-only | Opponents-only |
|---|---|---|---|
| Before the era landing (branch alone) | 41.1% | 39.3% (-1.9 pp [-5.2, +1.4]) | 36.8% (-4.3 pp [-7.7, -0.9]) |
| On merged master (era curve) | 27.1% | 24.0% (-3.0 pp [-5.9, -0.1]) | 26.5% (-0.6 pp [-3.9, +2.6]) |

On merged master the seat builds 8 to 12 Engineers a game and captures no more cities with them (8.4 Sparta: 9.7 -> 8.0 captured), and the five AI empires together build 20 to 22 with the seat losing no more cities than before. The benchmark's Hard seat does not play a walled thin-army line, so it prices the Engineers as a cost; the benefit shows only against the line the coach replayed.

War suite (`tools/sim/war-scenarios.sh`, 64 runs, seat params on; merged master, genes on vs off in every seat):

| Scenario | Off | On |
|---|---|---|
| `war-golden-horde-thin` (the attack check, floor 30%) | 67% | 67% |
| `war-golden-horde` | 0% | 3% |
| `war-sally-engineer` (the Mongols arrive with 11 Engineers) | 100% | 0% (a German city falls on turn 4: the defender has one Spearman; the scenario predicted holds about 30% of the time) |
| `war-walled-engineer`, `war-sparta-walls` | 0% | 0% |
| Suite mean | 54% | 47% |

Before the era landing the thin check read 95% off and 95% on (51% with the first version, phantom cover). The drop to 67% is the era landing's, not the genes'.

Scenario Lab `swiss-walls-athens` (50 runs): committed baseline 0/50 (mean 697, damage to the target 0.3 points); branch before the era landing 1/50 (damage 6.7); merged master 0/50 either way (genes off 650 median, on 640; no damage dealt).

Known gap: `war-walled-engineer` stays at 0%. The Sparta army builds its six Engineers and walks them in, but the city assault counts only units that can reach a ring tile this turn: Keystone sits behind an impassable ridge, so three of six arrive and the forecast fails each turn (units at distance 3 and 4 have no reachable ring tile). Staging by path, not distance, would fix it.

# A war chest the door can spend: genes `war_chest`, `war_chest_standing` (W2, 8.9.15 Swiss review rec 5)

Measured 2026-10-30 on branch `claude/w2-war-chest` (master `0fa889678`). Everything ran on the cloud box (one slot, 64 threads). Design: `crates/ai/src/war_chest.rs`. **Verdict: keep both genes off.** The door's Gold problem is income, not spending order.

## Baseline: how often the door wants a buy it can't afford

Method: a measurement-only stderr trace in `plan_door_defense` (not landed). It logs every seat-turn with a door front: treasury, Gold left after the earlier rescue buys, the cheapest combat unit buyable at an open-gap front, and the door's buys. Search rollouts are excluded. "Unaffordable" means the door bought nothing and Gold left, less the cheapest buy, falls under the door's floor.

**All-AI census (512 eight-empire Hard games, Standard map, seeds 7300001+):**

| Measure | Value |
| --- | --- |
| Seat-turns with an open gap at the door | 83,598 |
| Turns the door bought a unit | 3,376 (4.0%) |
| Turns it wanted one and couldn't afford it | 95.3% |
| Treasury at an open gap: median / 75th percentile | 74 / 115 |
| Gold left after the earlier rescue buys (median) | 66 |
| Cheapest combat buy at the front (median) | 160 |
| Income on those turns: median (deciles 2 to 36) | 13 |

Every faction is at 98% to 99.8% unaffordable, except Switzerland at 63% (it buys on 2,735 of its 8,225 turns, from its hoard).

**War suite (gene on the seat, 64 runs per case).** Every defence case reads 98% to 100% unaffordable, with a median treasury of 32 to 156 against a cheapest buy of 160:
- Japan T43: treasury 32, 0 buys.
- Japan T142: treasury 55, 0 buys.
- German stand: treasury 64, 0 buys.
- Mongol bombers T150: treasury 37, 0 buys.

The one exception is the Golden Horde hold: 2,072 Gold, a buy on every one of its 289 open-gap turns.

**Where Hard's Gold goes on threatened turns.** A tape audit (`wc_gold_spend`, not landed) of 256 games counted the turns when an enemy unit at war stood within 8 hexes of a city. It reads the whole board (omniscient), so it's a measuring tool only:

| Use of Gold | Gold per seat-turn |
| --- | --- |
| Income (not a spend) | 15.6 |
| Military purchases | 6.1 |
| Unit upgrades | 5.7 |
| Other purchases | 1.5 |
| Buildings | 1.4 |
| Siege | 1.2 |
| Companies | 1.1 |

Mean treasury on those turns: 194.

## Round 1: the marching need only (`war_chest` 100, `war_chest_standing` 0)

The chest is sized by the door's own gap model. It looks out to 8 turns of march (a unit counts in full within 3 turns, then less the further it is) and prices each open gap at the city's best strength-per-Gold buy, at most 4 units. Each city is capped at 30 turns of its yields, and income that arrives before the door opens counts toward the need. The chest joins `gold_reserve`. The door, rescue buys and the frontline defender may spend it.

The chest was held (median 580 Gold at an open gap, above the treasury on 89% of those turns), but almost nothing changed:

| Measure | Off | On (100) | On (200) |
| --- | --- | --- | --- |
| War suite, seat (30 cases) | 47% | 47% (every case identical) | 47% |
| War suite, opponents | 47% | 47% | 47% |
| Census: captures per game | 16.8 | 16.9 | 16.9 |
| Census: door buys | 3,376 | 3,483 | 3,451 |
| Census: unaffordable | 95.3% | 95.2% | 95.2% |
| Census: treasury at an open gap (median) | 74 | 80 | 80 |
| Adaptive Sparta, gene in the defenders (Sparta wins of 32) | 26 | 23 | |

**Why it does nothing:** armies come out of the fog only three to five turns from the door. At 13 Gold a turn, the treasury can't reach a 160-Gold unit in that window.

## Round 2: plus the standing need (`war_chest_standing` 1)

Each empire we are at war with that has a known city within 12 hexes keeps one door purchase in hand at our nearest city. The amount is weighted by its public army (Strongest Army values) against ours. The chest is the larger of this and the marching need.

| Measure | Off | Chest 100 | Chest 200 |
| --- | --- | --- | --- |
| War suite, seat | 47% (13 at parity) | **40%** (11) | 40% (11) |
| War suite, opponents | 47% | 48% | 48% |
| Census: captures per game | 16.8 | **15.0** | 15.4 |
| Census: game length (turns) | 247 | 268 | 273 |
| Census: Sparta wins / Switzerland wins (of 512) | 114 / 37 | 66 / 85 | 69 / 67 |
| Census: door buys | 3,376 | 2,869 | |
| Census: treasury at an open gap (median) | 74 | 91 | |

**Gold on threatened turns** (audit, per seat-turn):

| Measure | Off | Chest 100 |
| --- | --- | --- |
| Mean treasury | 194 | 263 |
| Military purchases | 6.1 | 10.8 |
| Unit upgrades | 5.7 | 1.5 |

**War suite, seat losses** (no other case moved more than 5 points):

| Case | Off | Chest 100 |
| --- | --- | --- |
| Mongols at Solothurn (attack) | 100% | 10% |
| Mongols at Chur (attack) | 34% | 15% |
| German air T214 (defence) | 100% | 0% |
| Japan v Sparta T142 (defence) | 82% | 62% |

With the gene in the opponents, Chur rises from 34% to 51%: the Japanese and Swiss defenders that hold it are weaker.

**Adaptive Sparta** (Hard in the designer's 8.9.13 seat from T40, 32 salts):

| Arm | Sparta wins | Cities Sparta took |
| --- | --- | --- |
| Control | 26 | 12.4 |
| Gene in the other AIs | 25 | 13.3 |
| Chest 200 in the other AIs | 28 | 14.0 |
| Gene in every seat, Sparta's too | **12** | 9.2 |

The treasury rises, but the Gold is spent by the rescue and threat purchases that run before the door (`buy_emergency_defender`, `buy_ready_defenders`, `plan_threat_defense`), not by the door. Upgrades are what it gives up. An empire at war holds Gold instead of refitting, so attackers stall: the hollow-leader cases, Sparta and every AI attack in the census. Defenders gain nothing measurable.

## Cost

Seat planning time per turn (native, `war_trace` runs 0 to 2, six cases): unchanged within noise. For example, Golden Horde hold is 3.71 / 3.68 ms off / on, Japan T142 is 3.38 to 4.41 / 3.49 to 4.17 ms, and German stand is 1.45 / 1.41 ms.

## Implications

- The door has nothing to spend because Hard earns 13 Gold a turn, not because it spends in the wrong order. The designer earned 53 to 177 a turn in the same game.
- More door buys need more Gold income, cheaper defence, or a reason to keep the treasury that pays off (the Swiss hoard does).
- Reserving Gold at war costs attack power. It recreates the standing-army failure of 2026-10-27, with Gold in place of units.

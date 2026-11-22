# AI expansion genes (2026-11-02, branch `claude/ai-expansion`)

From the 8.9.23 Germany coach review (`docs/playtests/reviews/2026-11-01-8.9.23-germany-start.md`): the designer had 3 cities at T23, 4 at T32 and 9 at T81; Hard on his own map founds its 3rd and 4th city around T42 and T56 and has 5 at T81.

## Diagnosis (his start, 48 to 64 Hard games, `expansion_probe`)

`crates/sim/examples/expansion_probe.rs` plays every seat of his save's setup as Hard, reseeding the dice per salt, and records the human seat's Settler gates for each idle city (`ScriptedBrain::production_diagnosis`) and every Settler capture with its circumstances.

- **Production allocation, not site scoring or population.** To T60, 918 of the capital's choices passed every Settler gate, and 641 of them went to a troop: mostly the capital-garrison quota (`2200 +` the unit bonus, times Germany's 123% military weight) because the starting Soldier was out exploring, which outbid the Settler's flat `~1,700 x 121%`. The capital reached population 3 on T6 and spent T8 to T17 on a Horseman it never finished.
- **The two-troop gate** (`army_ok`: before T80 a second Settler needs two troops) then stopped the second Settler for 5 to 8 turns in most games, and the pipeline cap (two Settlers alive or building) and the `developed` gate followed.
- **Settler safety: Scouts at Peace.** In 48 games rivals took 106 Settlers (2.2 a game); 96 were taken by a Scout, 88 by a surprise war from Peace, 105 with no escort beside them, a third of the victims on one city. The escort search skipped every unit standing in a city, the only troop a young empire has.
- **The snipe** fires when a capturer is already within one move; Hard in his seat took 0.3 Settlers a game. Exposed Settlers 3 to 4 hexes from its troops were left alone.

## Genes (default on; 0 is the old behaviour)

| Gene | What it does |
|---|---|
| `colony_compound` 100 | A Settler adds a new city's compounding return per Production spent (`production.rs::colony_compound_value`): the best reachable site's early worked yield over 40 turns less the citizen it takes, raised by 1/cities; one troop instead of two lets an early Settler start. Not while a conquest target is set. |
| `settler_escort` 1 | A city's garrison may escort a threatened Settler when the city has no threat and no enemy at war within 3 (`hard_defense.rs::garrison_can_escort`). |
| `settler_stalk` 3 | A Scout or field troop closes on an exposed rival Settler at Peace up to 3 turns away, without declaring, while no own city is threatened (`plan.rs::stalk_settlers`); the Snipe plan prices the war once it is in reach. |

## Human-seat benchmark

`hard_vs_human`, seed 310001, 256 reseeded salts on the 8 maps (2,048 games per arm), plain Hard rivals, one c8g.16xlarge spot server (0.49 server-hours). [arms.json](arms.json): `ctl` has all three genes 0 in every seat; `_all` turns the gene on everywhere; `_riv` (mode `opponents`) and `_seat` (mode `seat`) set the genes off in the other seats through the new `other_params` override. Delta = change in the designer's-seat win rate, paired salt bootstrap ([compare.py](compare.py), [comparison.json](comparison.json)); on a rivals-only arm negative means stronger rivals. Batch `~/LegatumRuns/cloud/aiexp-gate-1102-111429`.

| Arm | Seat wins | Delta pp [95%] | Seat cities T50 / T100 | Rival cities T50 / T100 | Settlers lost per empire-game (seat / rivals) |
|---|---:|---:|---:|---:|---:|
| ctl | 513 | | 2.62 / 5.04 | 2.64 / 5.09 | 0.41 / 0.58 |
| colony, rivals only | 489 | -1.17 [-3.42, +0.98] | 2.65 / 5.03 | 2.98 / 5.24 | 0.40 / 0.59 |
| colony, all seats | 449 | -3.12 [-5.42, -0.88] | 2.92 / 5.05 | 3.03 / 5.26 | 0.44 / 0.59 |
| escort, rivals only | 501 | -0.59 [-2.49, +1.37] | 2.56 / 4.97 | 2.67 / 5.17 | 0.42 / 0.50 |
| escort, all seats | 516 | +0.15 [-1.86, +2.25] | 2.61 / 5.04 | 2.66 / 5.16 | 0.38 / 0.51 |
| stalk, rivals only | 510 | -0.15 [-1.32, +1.03] | 2.62 / 5.04 | 2.64 / 5.08 | 0.40 / 0.58 |
| stalk, all seats | 531 | +0.88 [-0.49, +2.20] | 2.62 / 5.06 | 2.64 / 5.08 | 0.41 / 0.60 |
| **all three, rivals only** | **444** | **-3.37 [-5.62, -1.12]** | 2.63 / 4.96 | 3.01 / 5.28 | 0.39 / 0.52 |
| all three, all seats | 425 | -4.30 [-6.49, -2.15] | 2.93 / 5.06 | 3.06 / 5.32 | 0.41 / 0.50 |
| all three, seat only | 549 | +1.76 [-0.34, +3.76] | 2.91 / 5.21 | 2.66 / 5.08 | 0.42 / 0.59 |

Intervals are per arm (no family correction). The combination makes the rivals clearly stronger against the designer's seat; the colony gene carries most of it, the escort halves Settler losses on its own, and the stalk alone is neutral.

## Hard in his seat on his Germany map (`expansion_probe`, 64 games per arm, all seats on the arm's genes)

| Arm | Cities T50 / T81 / T100 (median; mean) | 3rd / 4th city (median turn) | His Settlers lost a game |
|---|---|---|---:|
| off | 3 / 5 / 5 (2.83 / 4.70 / 5.31) | T42 / T56 | 0.34 |
| colony | 4 / 5 / 6 (3.56 / 5.11 / 5.67) | T30 / T47 | 0.39 |
| escort | 3 / 5 / 6 (3.00 / 4.83 / 5.50) | T43 / T54 | 0.28 |
| stalk | 3 / 5 / 5 (2.78 / 4.66 / 5.31) | T43 / T57 | 0.38 |
| all three | 4 / 5 / 6 (3.58 / 5.20 / 5.86) | T30 / T47 | 0.36 |

The designer: 4 cities at T50, 9 at T81, 3rd and 4th city at T22 and T31. With `settler_pipeline` 3 on top of all three genes: 3.73 / 5.36 / 6.02 (not shipped; a separate gene the climb can take). The remaining gap: Settlers take about 12 Settler-turns per founding (his are similar), so the next limits are the pipeline cap, the `developed` gate for cities 5 and up, and the garrison and ranged quotas in new cities.

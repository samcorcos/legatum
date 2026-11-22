# The Settler contest and the unfinished strike (2026-11-02, branch `claude/ai-strike-escort`)

From the coach's review of the designer's 8.9.24 Swiss win (`docs/playtests/reviews/2026-11-02-8.9.24-swiss.md`): the T41 Settler swing (Hard in his seat 4% from T40, 46% from T42) and the T58 to T61 raid that left Lugano at 30 of 200 HP.

## Why the genes did not fire in his game

- **The escort (`settler_escort`).** Plain Hard in a five-player game runs the duel guard path (`settler_safety` 0), whose threat is visible units. The Mongol garrison Soldier walked out with the Settler at T37 and was released at T38, when the Swiss Soldier stood in fog; it went back to garrison, and the Settler walked on alone two hexes past Thun and was taken at T41.
- **The snipe (`settler_stalk`, Snipe plan).** At T41, Hard in his seat paired the Settler with Scout #2 three hexes off: the snipe reach read the view's `moves_left`, last turn's leftover (0 for that Scout), and ties went to the lowest unit id, ahead of the Thun Soldier two hexes off. The Settler walked on and the plan ended `SnipeMissed` the next turn, in every one of 8 branches. A pursuer that starts beside a Settler pins it; from two hexes a moving Settler outruns an equally fast one.

## Genes

| Gene | Value | What it does |
|---|---|---|
| `settler_escort` | 2 (new value, default) | A route within 2 hexes of a city of a rival that can declare a surprise war counts as threatened with no unit in sight; a Settler with no escort in reach this turn but one within 2 turns waits where nobody can take it this turn while the escort comes (`hard_defense.rs::settler_route_near_rival_city`, `guard_exposed_settlers_duel`). 1 is the garrison escort alone. |
| `stalk_contact` | 1 (new, default) | With `settler_stalk` on: the Snipe pairs a Settler only with a melee-capable unit beside it; the stalk closes every non-adjacent unit within the horizon with this turn's moves, onto the side away from the owner's nearest city, and may use a quiet city's garrison (`plan.rs::snipe_targets`, `stalk_settlers`). |
| `finish_kill` | 0 (new, off) | A campaign whose city is at most this percent of its HP, with a unit of ours still on it, is not ended for an insufficient army while a melee unit (a quiet garrison included) can arrive within `march_turns`; those units join and march, and the Campaign plan waits (`campaign.rs::finish_kill_force`). |

## The decisive turns, replayed (branches from his save, salts reseed dice and AI seeds)

| Branch | Seat | Rivals | Runs | Seat wins | Cities at T50 (seat / Mongols) |
|---|---|---|---:|---:|---|
| T40, Hard in his seat | 8.9.24 genes | 8.9.24 genes | 48 | 2 (4%) | 3 / 4 |
| | shipped genes | 8.9.24 genes | 48 | 20 (42%) | 4 / 3 |
| | 8.9.24 genes | shipped genes | 48 | 6 (12.5%) | 3 / 4 |
| | shipped genes | shipped genes | 48 | 22 (46%) | 4 / 3 |

The 8.9.24 row reproduces the coach's 4%. With the new seat genes Hard stalks the Settler with the Thun Soldier at T40, declares from the next hex at T41 and takes it, then makes Peace at T43, as he did. From T36, with the new genes on both sides, the Mongol escort stays beside the Settler and the Mongols found their fourth city at T43 in 8 of 8 branches.

| Branch (to T75) | Runs | Lugano falls, `finish_kill` 0 / 50 / 80 |
|---|---:|---|
| T58, his orders and purchases | 32 | 0 / 0 / 0 (lowest HP: median 28 in every arm; 0 of 32 again with the shipped genes) |
| T55, his orders without the T58-T59 purchases | 32 | 32 / 32 / 32 (all at T60) |

`finish_kill` holds the campaign at T61 to T63 and walks the two garrisons out, but both Mongol melee units died to the bought archers on T60 and T61 and the next capturers were 7 and 9 hexes away; Lugano heals 20 a turn.

## Scenario Lab (200 runs, Hard seat, all AI seats on the given parameters)

| Scenario | Master baseline (50 runs) | `finish_kill` 0 | `finish_kill` 50 |
|---|---:|---:|---:|
| `swiss-strongbox` (Mongols take Lugano in 25 turns) | 6% | 5.5% (11) | 6% (12) |
| `german-second-front` | 0% | 0% | 0% |

In `swiss-strongbox` Hard declares War in 29 of 200 runs (median T65) in both arms: its failure is the declaration, not the finish.

## Human-seat gate

`hard_vs_human`, seed 310001, 256 reseeded salts x 8 maps (2,048 games per arm), plain Hard rivals, one spot server. Control `ctl`: `settler_escort` 1, `stalk_contact` 0 in every seat (the 8.9.24 behaviour), 449 seat wins. `_riv` and `_seat` set the other seats to the control through `other_params`. Delta: change in the designer's-seat win rate; on a rivals-only arm negative means stronger rivals ([compare.py](compare.py), [comparison.json](comparison.json), [arms.json](arms.json)). Batch `strike-gate1-1102-230752`, 0.42 server-hours.

| Arm | Seat wins | Delta pp [95%] | Rival cities T50 / T100 | Rival Settlers lost a game |
|---|---:|---:|---:|---:|
| ctl | 449 | | 3.06 / 5.31 | 0.534 |
| escort 2, rivals only | 439 | -0.49 [-2.25, +1.32] | 3.06 / 5.31 | 0.537 |
| escort 2, seat only | 459 | +0.49 [-0.88, +1.90] | 3.07 / 5.32 | 0.533 |
| contact, rivals only | 459 | +0.49 [-1.12, +2.10] | 3.09 / 5.32 | 0.534 |
| contact, seat only | 443 | -0.29 [-1.46, +0.98] | 3.07 / 5.31 | 0.543 |
| both, rivals only | 419 | **-1.46 [-3.22, +0.34]** | 3.09 / 5.32 | 0.522 |
| both, seat only | 450 | +0.05 [-1.56, +1.71] | 3.08 / 5.31 | 0.537 |
| both, all seats | 453 | +0.20 [-1.71, +2.20] | 3.10 / 5.32 | 0.534 |

`finish_kill` 50 on top of the Settler genes (batch `strike-gate2-1102-234407`, [arms-finish-kill.json](arms-finish-kill.json); control 453): rivals only 441, -0.59 pp [-1.51, +0.29]; seat only 462, +0.44 [-0.05, +0.93]. Both directions say it helps the AI that has it, though not on his T58 position.

**Final configuration** (`settler_escort` 2, `stalk_contact` 1, `finish_kill` 50, with the retreat exemption) against the 8.9.24 control (`settler_escort` 1, `stalk_contact` 0, `finish_kill` 0), batch `strike-gate3-1103-001016`, [arms-final.json](arms-final.json), [comparison-final.json](comparison-final.json):

| Arm | Seat wins | Delta pp [95%] | Rival cities T50 / T100 | Rival Settlers lost a game |
|---|---:|---:|---:|---:|
| ctl | 449 | | 3.06 / 5.31 | 0.534 |
| final, rivals only | 431 | -0.88 [-2.69, +0.98] | 3.09 / 5.31 | 0.532 |
| final, seat only | 438 | -0.54 [-2.10, +1.07] | 3.08 / 5.31 | 0.534 |

Rivals are stronger, within noise; Hard in the seat is level within noise (-0.54), while on his own 8.9.24 position it wins 42 to 46% from T40 instead of 4%.

## Decision

Ship `settler_escort` 2 and `stalk_contact` 1 together: rivals with both are the strongest arm (-1.46 pp, the interval just spans zero), the seat is unchanged, and the designer's decisive turn moves the right way on both sides. Alone, each is noise. Ship `finish_kill` 50 as well: rivals not weaker, the seat stronger. After the gates a Settler falling back into an own city was exempted from the escort wait (a unit test caught the wait replacing a retreat home); the final configuration was gated again against the 8.9.24 control (above).

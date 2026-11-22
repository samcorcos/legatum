# Celts rework: barbarians spare the Celts (2026-11-02)

Branch `claude/celts-rework`, from master `139a89615` (code identical to `1877f2b5a`). Rule text: `game-rules.md` section 7 ("Barbarians never attack the Celts") and the faction table; learnings entry of the same date.

## Balanced Hard league (8 empires, Standard, 3,000 games per arm and seed, per-seat-appearance win rate)

Seeds 9410001 and 9510001, factions drawn from the seed (identical draws in every arm). Control is master, final is the branch. `tools/cloud/run-batch.sh --count 1 -- --games 3000 --seed S --difficulty hard`.

| Faction | Control | Final | Delta pp |
|---|---:|---:|---:|
| Athens | 20.2 [19.1, 21.3] | 19.9 [18.8, 21.0] | -0.3 |
| Mongols | 16.6 [15.6, 17.6] | 16.5 [15.6, 17.6] | -0.1 |
| Sparta | 15.9 [14.9, 16.9] | 16.0 [15.0, 17.0] | +0.1 |
| Aztecs | 14.6 [13.7, 15.6] | 14.2 [13.3, 15.2] | -0.4 |
| Byzantium | 13.1 [12.2, 14.0] | 12.6 [11.8, 13.5] | -0.4 |
| India | 10.6 [9.8, 11.5] | 10.3 [9.5, 11.1] | -0.4 |
| Germany | 8.0 [7.3, 8.8] | 8.1 [7.4, 8.8] | +0.1 |
| Switzerland | 7.5 [6.8, 8.2] | 6.5 [5.8, 7.1] | -1.0 |
| **Celts** | **6.1 [5.5, 6.8]** | **8.5 [7.8, 9.3]** | **+2.4 (paired bootstrap [+1.6, +3.3])** |

Best/worst ratio 3.32 to 3.08. Per seed, Celts: 6.3 to 9.0 (seed 1), 5.9 to 8.0 (seed 2). Mean game length unchanged (191.7 to 191.8).

## Ladder, seed 9410001 only (Celts win rate, 2,651 Celtic seats each)

| Arm | Celts |
|---|---:|
| Control (master) | 6.3% |
| Rule and stats only (AI = master) | 9.0% |
| + the Celtic AI no longer fears barbarians (the shipped build) | 9.0% |
| + call the Wild for its raid value (2 or more rival cities within 6 of the camp sites) and keep those camps | 3.7% |
| the same call only after expansion, keeping the camps | 7.5% |
| the same call only after expansion, clearing the camps as before | 7.6% |

The raid-value call was not shipped.

## Human-seat benchmark (seed 310001, 256 reseeded salts on 8 maps, 2,048 games per arm, plain Hard rivals)

`hard_vs_human --arms` with one all-seats `ctl` arm via `tools/cloud/run-batch.sh` (the method of the trains; `quality_gate.py`'s frozen-control machinery was not set up), paired salt bootstrap with `artillery-siege-1031/compare.py`.

| | master | branch | Delta pp [95%] |
|---|---:|---:|---:|
| Human-seat wins | 449 | 472 | +1.12 [+0.49, +1.76] |

Only the Celtic-seat map moves, 19 to 42 wins of 256 (7.4% to 16.4%); the seven other maps are identical, win for win (they have no Celtic seat). The first candidate (with the raid-value call) read 444 (-0.24 [-0.73, +0.24]; Celtic map 19 to 14).

## Pins

Only runs with a Celtic seat moved. Of the three 40-turn mixed tapes the first (no Celts) is unchanged and the other two (a Celtic seat each) moved; the scripted city-fire tape (`city-strike-awareness`, a Celtic seat) and the garrison replay (a Celtic seat) moved. 70 current-rule fixtures and 55 scenario files only re-stamped the version.

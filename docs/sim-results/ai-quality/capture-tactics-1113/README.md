# Capture tactics from the Fable seat (2026-11-13, `claude/capture-tactics`, rules 30)

**Question.** The Fable seat won After the Riflemen 3 of 3 while Hard wins 21 of 50. Do its five tactical lessons, written as Hard genes, win more?

**Answer.** No. None of the genes beats the 21/50 baseline by more than noise. Hard mostly loses this card before tactics matter: in 24 of its 29 losses it never makes a melee attack on Tenochtitlan, and in 21 of those it fires 2 shots in all. Its campaign value collapses and the plan returns to Peace (`funded forecast unavailable` from about T142). All genes stay off.

## How it was run

`legatum-sim scenario scenarios/lab/athens-after-the-riflemen.json --runs 50 --threads 4 --out <arm dir> [--ai-params params/<arm>.json --params-seat-only]`. Runs 0-49 use the runner's salts (run 0 keeps the card's seed, run N derives `Rng::derive(seed, [0x5CE4_A210, N])`), the same salts as the card's recorded 21/50 baseline, which this branch reproduces exactly. Every arm gives the genes to the Athens seat only; the Aztecs and everyone else play default Hard. The per-arm `.md` reports are in the arm directories; the per-run JSON is not committed (`docs/sim-results` keeps JSON out of git).

## After the Riflemen, 50 runs per arm

| Arm | Genes | Wins/50 | Median score | Runs won that baseline lost / lost that baseline won |
| --- | --- | --- | --- | --- |
| baseline | none | 21 | 801 | |
| relief | `relief_forecast` 1 | 18 | 770 | 0 / 3 |
| preview | `charge_preview` 1 | 23 | 797 | 4 / 2 |
| relief-kill | `relief_kill` 50 | 21 | 811 | 3 / 3 |
| capturer-upgrade | `capturer_upgrade` 100 | 24 | 891 | 22 / 19 |
| regen-strike | `regen_strike` 100 | 21 | 801 | 0 / 0 (no run changed) |
| relief-preview | lessons 1 and 3 | 19 | 780 | 3 / 5 |
| capturer-relief-kill | `capturer_upgrade` 100, `relief_kill` 50 | 25 | 1140 | |
| all | all five | 2 | 780 | |

Pairs measured to explain "all": `capturer_upgrade` + `relief_forecast` 7/50, `capturer_upgrade` + `charge_preview` 3/50, `relief_forecast` + `charge_preview` + `relief_kill` 18/50.

With 50 runs the standard error is about 3.5 wins, so a difference under about 7 wins is noise. The best arm (capturer-relief-kill, 25) is 4 above baseline.

## Regression cards (baseline / capturer-upgrade / capturer-relief-kill / all, wins of 50 and median)

| Card | Baseline | capturer-upgrade | capturer-relief-kill | all |
| --- | --- | --- | --- | --- |
| Stop the Snowball (`athens-stop-the-snowball`) | 4, 110 | 4, 110 | 4, 110 | 4, 110 |
| The Courier King (`runaway-conqueror`) | 39, 200 | 39, 200 | 39, 200 | 40, 200 |
| Athens against two Pikemen, T112 (`war-athens-two-pikemen-t112`, Piraeus and Marathon) | 0, 630 | 0, 630 | 0, 630 | 0, 630 |

Other capture cards (baseline / capturer-upgrade / capturer-relief-kill): Take Gergovia 0/0/0, Walled Nicomedia 4 (970) / 3 (920) / 3 (920), Ride to Dyrrachium 2/2/2, Two Cities in One Turn 0/0/0, Before the Walls 0/0/0.

## What each gene does, and what it showed

1. **`relief_forecast` (lesson 1, `opportunity.rs::relief_units`).** The siege and assault forecasts model the core's relief chain: the city owner's melee units that can step in are fought one by one after the garrison. Alone: 18/50, no run gained. The forecast plays several rounds, so a lone attacker's capture is only delayed a round and the storm still charges with one unit. A first version also let the relief units skip their attacks on our units; the storm then charged at T133 with three mounted units and won 1/50.
2. **`charge_preview` (lesson 3, `city_assault.rs::charge_loses`).** A melee charge that does not capture this turn is held when the engine's preview has it dying or taking more than it deals. It won exactly the four runs where baseline Hard made losing lone charges (5, 11, 13, 19), but it breaks sustained storms: with `capturer_upgrade` Hard takes Tenochtitlan on T136 by charging with two melee units three turns running, each charge a bad trade on its own; the preview holds the second unit and that arm falls to 3/50.
3. **`relief_kill` (lesson 2, `tactics.rs::relief_stake`).** A unit that can relieve a city our melee can reach is worth part of the city's value to kill. Neutral alone (21/50).
4. **`capturer_upgrade` (lesson 4, `modernization.rs::capturer`).** Refits of melee units near an enemy city score higher, and Gold is held for the best one. Hard then upgrades the Assault Trooper on T133 instead of the Marksman, and in 20 runs takes the city on T136 with two melee attackers a turn (lesson 1 emerges from the stronger unit). The Soldier the teacher upgraded still waits until T146: the Assault Trooper's refit scores higher per Gold. 24/50, median 891, but the win set reshuffles (22 gained, 19 lost) and Walled Nicomedia loses one win.
5. **`regen_strike` (lesson 5, `tactics.rs::regen_strike_at`).** Tiles within 2 of a walled city at 0 HP that will heal are charged its next strike. No run on any card changed: Hard rarely parks there with no unit beside the city. Mounted units leading a charge are left to `charge_preview`, since the preview already carries their -33%.

## Recommendation

Leave all five genes off. `capturer_upgrade` (alone or with `relief_kill`) is the only lean, and it is inside the noise with one regression card slightly worse. The bigger lever on this card is strategic: Hard drops the campaign in most runs. That is the next thing to look at (why `forecast_siege` turns the funded forecast unavailable around T142 while the city is weak).

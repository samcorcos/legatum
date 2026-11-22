# Egypt: the Nile against the current rule (2026-11-21)

**Result.** As specified (double Food, no Production, 1 progress per surplus Food), Egypt wins 17.5% of eight-seat Hard games over two pools (400 Egypt seats), where even is 12.5%, against 6.5% under the current rule. That ties it for the top of the spread with the Mongols and Aztecs. At 85% conversion it wins 10.5%. The designer chose 1:1 (2026-11-21), the default (`EGYPT_NILE_BUILD_PERCENT = 100`). Among six rushers, Nile Egypt is eliminated less often than the old Egypt (1.0% against 4.5%) and loses fewer cities. Rules 48 (47 when run) on this branch, `legatum-sim`, scripted Hard in every seat, medium map, seeds as listed.

**These numbers replace a first sweep that was wrong.** The player view applied the build-dependent part of a city's yields twice (`view.rs::city_detail`, fixed on this branch), so the AI planned with Food ×4 on the Nile and with decadence taken twice. Resolution was right, but the AI's choices were not. That sweep measured 30% at 1:1 and 14.5% at 85%, and the designer first picked 85% on it. Everything below is from after the fix. The current-rule arm came out the same (6.5%).

## Both pools, Egypt rotated through all eight seats (200 games per pool per arm)

Pool 1: Egypt, Switzerland, India, Byzantium, Germany, Celts, Mongols, Sparta; seeds 7000-7199. Pool 2: Egypt, Athens, Aztecs, Rome, Switzerland, Germany, Mongols, Sparta; seeds 9000-9199. `--rotate-factions`. The current rule is `--rule-off egypt_nile`; other rates use `--rules-experiment egypt-nile-build=N`. Eliminated counts any elimination; cities lost is per Egypt seat.

| Arm | Egypt seats | Egypt win rate [95% CI] | Eliminated | Cities T50 | Cities T100 | Cities lost / game | Buildings |
|---|---|---|---|---|---|---|---|
| Current rule (Food-funded buildings) | 400 | 6.5% [4.1%, 8.9%] | 15.0% | 3.11 | 4.46 | 2.08 | 36.9 |
| Nile, 85% | 400 | 10.5% [7.5%, 13.5%] | 15.2% | 2.26 | 4.10 | 2.27 | 37.3 |
| Nile, 90% | 400 | 9.2% [6.4%, 12.1%] | 13.2% | 2.32 | 4.25 | 2.18 | 39.2 |
| **Nile, 1:1 (chosen)** | 400 | **17.5% [13.8%, 21.2%]** | 10.8% | 2.73 | 4.92 | 2.17 | 50.3 |

By pool: 1:1 is 21.0% in pool 1 and 14.0% in pool 2; 85% is 14.0% and 7.0%. 85% and 90% are within noise of each other; the step to 1:1 is large.

A 40-game check of the new default without the flag gives the same winners as `egypt-nile-build=100` on the same seeds.

Faction win rates over both pools:

| Arm | Best to worst |
|---|---|
| Current rule | Mongols 20.8%, Germany 19.2%, Athens 18.0%, Byzantium 15.5%, Celts 15.5%, Aztecs 14.0%, India 10.0%, Sparta 8.5%, Switzerland 7.0%, **Egypt 6.5%**, Rome 3.0% |
| Nile 85% | Mongols 21.8%, Germany 19.2%, Athens 19.0%, Celts 14.5%, Aztecs 14.0%, Byzantium 11.5%, **Egypt 10.5%**, India 10.0%, Switzerland 7.8%, Sparta 6.2%, Rome 0.0% |
| Nile 1:1 | **Egypt 17.5%**, Mongols 17.5%, Aztecs 17.5%, Germany 17.2%, Celts 16.5%, Athens 16.5%, Byzantium 11.0%, Sparta 7.5%, India 7.0%, Switzerland 5.8%, Rome 0.5% |

Egypt moves from second-worst to tied top, below the Mongols' 20.8% under the current rule. The pool's spread is set by Rome, not Egypt.

## Early conquest (pool 1, Egypt in seat 0, 200 games each)

`--seat-opening all=Rush --seat-opening 0=Default`, seeds 11000-11199: Egypt plays its normal opening while the other seven rush.

| Arm | Egypt seats | Egypt win rate [95% CI] | Eliminated | Cities T50 | Cities T100 | Cities lost / game | Buildings |
|---|---|---|---|---|---|---|---|
| Current rule, among seven rushers | 200 | 75.0% [69.0%, 81.0%] | 4.5% | 3.40 | 5.49 | 1.34 | 69.9 |
| Nile 1:1, among seven rushers | 200 | 87.0% [82.3%, 91.7%] | 1.0% | 2.65 | 5.95 | 0.76 | 78.7 |

AI Egypt on the Nile holds up against early conquerors: it is eliminated less and loses fewer cities. The scripted Rush opening is weak for every faction (the first sweep's "Egypt rushes" arm won 0% under both rules), so this says nothing about a human Egypt rushing. That needs the human-seat benchmark or a playtest.

## Caveats

- The AI's Grow choice (`production.rs::egypt_nile_choice`, gene `egypt_grow_payback` 30) is a first heuristic and is not tuned.
- Nile Egypt founds fewer cities by T50 (2.7 against 3.1) and catches up by T100: a building city doesn't grow, so its early Settlers come later.
- Human Egypt is unmeasured; check the designer's next Egypt game.

# Era objectives, step 1: count gains from the last era score (R1), one era a turn (R2), `objective_seek`

2026-10-21, branch `era1/seek-r1-r2`. The user approved the redesign on 2026-10-20 (decisions log); this is step 1 of `docs/design/research/era-objectives-tradeoffs-2026-10-20.md`: the three rule and AI changes, on today's pool (no new objective kinds).

## What changed

- **`objective_seek` gene** (`crates/ai/src/params.rs`, default 1, range 0 to 1). At 0 the AI ignores era objectives entirely: `crowns.rs::sought_objectives` is empty, so there is no production bonus, no effort estimate, no chosen objective, no `objective_close` victory pass and no capture value for a live objective (the leak through `weight.max(1)` is closed, because `objective_capture_value` returns 0 first). `live_objectives()` stays the plain fact (the endgame planner reads "nothing left to score" from it), so the gene turns off pursuit, not knowledge. `docs/sim-results/era-objectives/params/seek-0.json` is the params file.
- **R1.** Every era objective counts only what the empire gains after it entered the era and after its last era score. Each `EraRecord` keeps two snapshots from when counting began: `base` (fight counters, as before) and the new `levels` (cities, population, tiles, technologies, and for each building the cities holding it or a later step). Progress is the count now minus the snapshot, never below zero. Scoring an era takes a fresh snapshot for every era the empire has entered and not scored (`objectives::restart_counting`). An era not yet entered counts nothing. The texts follow: "Gain 5 cities", "Grow your population by 50", "Gain 80 tiles of territory", "Add a Library to 4 more cities".
- **R2.** At most one era scores per empire per turn, the oldest first.
- **The pool is rewritten as gains** (`era_pool`), because the old counts were holdings ("Own 11 cities") and a gain of 11 cities is out of reach. `DEFAULT_OBJECTIVE_SCALE` went from 150 to 100 (the pool is written at the final counts; `objective-scale=N` still multiplies them). Fight counts are set the same way.
- UI: the objectives sheet has a one-line note under "Era objectives", each count's tooltip carries the rule, an era you have not entered shows 0, the Guide's Era objectives section explains it; game-rules.md section 6; the design, UI and coach skills.
- Tools: `tools/sim/era-objectives.sh` (all-AI Hard cohorts per arm and the report), `tools/sim/era-objectives-report.py`, `tools/sim/era-objectives-benchmark.sh`. `vp_race` telemetry now records the whole pool (at the game's counts) as met every turn, which pool objectives were drawn and the pool's text.

## Calibration

Four passes of 256 games each (8 Hard empires, Standard, seeds 9100 on the box, about 3 minutes a pass), then a final pass after master landed the crown-upkeep removal. Findings:

1. **R1 and R2 alone, on the old level counts, break the game.** Median 479 turns, 46 of 256 games at the 600-turn safety cap (master then: 303 and 0). Era scores per empire barely moved (2.52 against 2.61), but the winners' stacking is gone: in master 310 of 2,048 empires scored all six eras and the best empire of a game scored six in 188 of 256 games; with R1 (on the level counts) it was 100 and 87. Gains of 9 or 11 cities were nearly never met (Expand was 1% of scores, against 17% in master).
2. **Thresholds were set at the 70th percentile of the best gain** each entrant made in each era (`peaks` telemetry), by measure and era, then eased in the late eras for pace.
3. **The calibration is coupled.** Scoring an era restarts the count of every older open era, so easing a late era makes empires score it earlier and lowers early-era scoring (era 2 went from 51% to 46% when only eras 5 to 7 changed). Retune all eras together.

Final pool (counts for 8 empires; 12% more per missing empire):

| Era | Build (three options) | Expand | Fight |
|---|---|---|---|
| 2 Classical | Library, Market, Workshop in 4 more cities | Gain 5 cities, grow by 50 population, gain 80 tiles | Capture 2 cities, destroy 30 units |
| 3 Medieval | Library 4, Walls 4, Castle 2 | 4 cities, 42 population, 62 tiles | A city from an empire ahead, destroy 38 |
| 4 Renaissance | University 4, Bank 2, Mill 2 | 3 cities, 34 population, 49 tiles | A city from an empire ahead, destroy 38 |
| 5 Industrial | Forge 2, Factory 2, University 3 | 2 cities, 25 population, 35 tiles | A city from the VP leader, destroy 29 |
| 6 Modern | Factory 2, Research Lab 2, Bank 3 | 2 cities, 23 population, 31 tiles | Capture 1 city, a city from the leader, destroy 30 |
| 7 Future | Research Lab 3, Star Fort 2, Factory 2 | 2 cities, 25 population, 33 tiles | Capture 1 city, a city from the leader, destroy 26 |

## Results (256 games per arm, seeds 9100 to 9355, 8 Hard empires)

The control is master `36ad0b462` (crown upkeep and the speed layer already removed), same seeds and flags. `default` is the branch merged onto it with `objective_seek` 1, `seek0` is every seat at 0. Full tables, every pool objective by era, drawn or not: `step1-report.md`.

| | master | R1 + R2, default | R1 + R2, `objective_seek` 0 |
|---|---:|---:|---:|
| Median game length (p10 / p90) | 298 (246 / 383) | **318** (246 / 445) | 336 (253 / 449) |
| Games at the 600-turn cap | 0 | 2 | 3 |
| Median turn the first team reaches the target | 281 | 297.5 | 309 |
| Era scores per empire | 2.54 | 2.69 | 2.66 |
| Entrants scoring era 2 / 3 / 4 / 5 / 6 / 7 | 58 / 57 / 47 / 40 / 37 / 35% | 58 / 54 / 53 / 46 / 43 / 34% | 60 / 51 / 47 / 46 / 44 / 38% |
| Empires that scored all six eras | 296 of 2,048 | 144 | 131 |
| Empire-turns that scored two or more eras | 795 (15% of scores) | **0** | **0** |
| Era scores on the turn the era was entered | 36 | **0** | **0** |
| Era scores by Build / Expand / Fight | 11 / 17 / 72% | 37 / 18 / 45% | see report |
| Median turns from entering an era to scoring it, eras 2 to 7 | 175, 147, 127, 103, 70, 38 | 125, 121, 99, 88, 69, 52 | 129, 114, 100, 88, 76, 65 |

- **Pace holds:** the median is 318 turns, inside the 280 to 330 target (master 298, 20 turns shorter: its leaders bank several eras in one turn); the first team reaches the target at the median in 297.5 (master 281).
- **Double-scoring is gone:** no empire scored two eras in a turn and none scored on the turn it entered an era. In master 795 empire-turns scored two or more, and 296 of 2,048 empires scored all six eras (now 144).
- **Stalls:** 2 of 256 games hit the cap (master 0; `objective_seek` 0 has 3). The capped games of an earlier cohort were hold wars (every empire had 3 to 6 era VP, the leader's hold was broken 4 to 6 times and none completed), not an objective deadlock. Watch it in step 2.
- **Met by accident** (share of entrants meeting each pool objective since entry or the last era score, `objective_seek` 0, objectives not drawn, so nobody aimed at them): Build 26% on average (5 to 49%), Expand 24% (14 to 35%), Fight 34% (8 to 55%); 39 of the 50 pool objectives are met by more than 15% of entrants by accident, mean 27% (the old pool, measured on 2026-10-20: median objective 16%, 26 of 50 above 15%; the new counts were set to hit the era-scoring targets and the pace, so they are easier to meet than the old ones). Accident is the baseline step 2 has to push down, kind by kind.
- **Pursuit gap:** drawn and pursued (`default`) minus drawn and not pursued (`seek0`): +0.6 points for Build, -0.7 for Expand, -0.9 for Fight; the largest single gap is Castle in 2 more cities (+11: 45% pursued against 34%). The AI's pursuit is weak (as the research doc said), so the objectives still measure how a game goes more than what an empire chose. Pursuit does speed the game: 318 against 336 turns.
- **Human-seat benchmark** (Hard in the user's recorded seat on the eight fixed maps, 64 salts per map, reseeded dice, every seat on the same AI, spot server): master 36.5% [33.0, 40.0], branch default **40.2% [36.5, 43.9]** (+3.7 pp, inside the intervals; the rule change makes the recorded seat no weaker), `objective_seek` 0 in every seat 43.6% [39.6, 47.3] (+3.4 pp over default: when no seat chases objectives, the recorded seat wins more often; the pursuing AIs are the harder opponents). Per map: `benchmark-master.json`, `benchmark-default.json`, `benchmark-seek0.json` (measured with master at `99918a751`, before the speed-layer removal, which leaves the default game unchanged; an earlier pass before the crown-upkeep removal read 37.9, 39.8 and 43.4%). Single maps move up to 17 points on 64 salts (8.6 Germany 29.7 to 37.5%, 8.4 Japan 51.6 to 65.6% for seek0), so read only the overall; the all-seats mode cannot isolate the seat from the opponents because the rule applies to every seat.
- **Scoring is slow in every era** (table above): the median scorer needs 125 turns after entering Classical, and a leader needs several fresh gains in a row, so eras are mostly won after the next era opens.

## What to know for step 2

- Gains make every old "linear chain" look different but not less linear: the pool is still the same kinds (Library, then University, then Research Lab; cities, population, tiles in every era). It is calibrated, not redesigned.
- Fight is 44% of scores (master 74%) only because the Build gains became reachable; the "city from an empire ahead" and "capture N cities" kinds are still met by 40 to 57% of entrants without aiming.
- The era-scoring calibration must be redone whenever one era's counts change (coupling above), and after any change that moves pace (master's crown-upkeep removal moved master's own median from 303 to 294, and the speed removal to 298). `tools/sim/era-objectives.sh` is the loop: about 3 minutes on the box.

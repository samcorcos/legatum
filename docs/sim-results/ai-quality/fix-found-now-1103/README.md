# Found now when delay loses the site (2026-11-03, branch `claude/fix-found-now`)

Fix 1 of the decision-diff patterns (`docs/playtests/diffs/patterns-2026-11.md`, pattern 2): on 62 turns in 20 of the designer's games he founded on the Settler's tile while Hard routed it on, retreated or went home (+5.79 VP [+1.28, +10.30] summed). Hard's own Settler routing beats his on average (pattern 18), so the change is "found now when delay loses the site", not "found sooner".

## What Hard got wrong at the decisive turns

Traced on each game's own tag (`playtest-8.3`, `playtest-8.9.5`, both no longer decode on master: retired faction, title and building names) with a temporary trace of the own-tile verdict, and on master's engine for the 8.9.23 and 8.9.24 games (`decision_audit --states`, positions from `audit_states` on the recording tag, hash-checked).

| Game, turn | His order | Hard's order | Why Hard left the tile |
|---|---|---|---|
| Tollkeepers 8.3 T36 | found | step to (9,5) | exposed: a barbarian **Scout** beside it; a camp's Scout only wanders and reports (`rules/barbarians.rs::scout_goto`) |
| Tollkeepers 8.3 T44 | found | home | settler danger: a Ranger of the rival at **Peace** beside it, counted as certain capture |
| Signalwrights 8.9.5 T58 | found | step to (12,23) | exposed: a barbarian Soldier two tiles off standing on its **camp** (the guard holds the camp tile) |
| Signalwrights 8.9.5 T118 | found | step to (4,29) | two barbarian raiders two tiles off (by the rules they take a lone civilian in reach), visible-border risk, quality 58 |
| Signalwrights 8.9.5 T137 | found | home | visible foreign border that could hide a city, quality 31 (not preferred) |
| Germany 8.9.23 T64 (master) | found | step to (30,9) | exposed: a barbarian Scout three tiles off |
| Switzerland 8.9.24 T145 (master) | found | wait | remembered foreign border (his arm did worse: -12.5 pp, -0.25 VP) |

## Gene `found_now` (default 10)

`crates/ai/src/civilians.rs`: `found_here_value` (the founding-in-place value of gene `found_here`), with `found_now_capture_percent`, `found_now_border_risk`, `found_now_alternative` and `site_claim_percent`. Hard only.

- The capture chance on the Settler's own tile before the found order resolves is priced per unit that can reach it this turn instead of ruling the tile out: an empire at war or a raiding barbarian 100 (still a veto, the 8.3 Switzerland lesson of `found_here`), a barbarian Scout 5, a barbarian on its camp 0, a rival at Peace that can declare its `peace_settler_capture_chance` (10 with `peace_settler_capture` 0). Founding is worth the site's value less twice the chance of losing it (both sides of the swing), with the fresh city's loss counting only empires at war (barbarian raiders never assault a city).
- The tile must pass the search's border verdicts: a remembered foreign border no known city explains, or a visible one (unexplained, with `border_explained`). This keeps the Swiss T145 rejection.
- The search's choice is discounted by the chance its land is gone before the Settler founds there (`site_claim_percent`): 50 for a visible foreign Settler that can get within its city gap of the site by then, plus, for each other empire whose nearest known city lies within a Settler's reach (twice its city gap plus 2), the gene's percent a turn times `(reach + 1 - d) / reach` over the travel turns plus one. With no alternative (retreat, home, wait) any positive value founds.

## Gate 1: Hard's choice at the decisive turns

`decision_audit --diff-only`, the seat's Hard with the gene at 10 against 0 (on the old tags a port of the same logic behind `FOUND_NOW=10`, `found_now` itself not existing there).

| Turn | Gene off | `found_now` 10 |
|---|---|---|
| Tollkeepers 8.3 T36 | step to (9,5) | **found** (capture 5%) |
| Tollkeepers 8.3 T44 | home | **found** (capture 10%) |
| Signalwrights 8.9.5 T58 | step to (12,23) | **found** (capture 0%) |
| Signalwrights 8.9.5 T118 | step | step (raiders in reach: veto) |
| Signalwrights 8.9.5 T137 | home | home (border risk; also with master's `border_explained` ported) |
| Switzerland 8.9.24 T145 | wait | **wait** (border risk) |
| Switzerland 8.9.24 T142 | found | found |
| Switzerland 8.9.24 T61, T111 | wait, retreat | found (his arms: -0.04, +0.10 VP) |
| Germany 8.9.23 T64 | step to (30,9) | found (his arm: +0.00 VP) |
| Germany 8.9.23 T31 | home | found (his arm: **-0.54** VP [-1.02, -0.06]: at war, a Byzantine army 6 tiles off) |

## Gate 2: Scenario Lab `germany-found-by-the-scout`

The 8.9.23 Germany position at T64 (`scenarios/lab/germany-found-by-the-scout.json`): six cities, the Settler on a legal site with a barbarian Scout three tiles off. Objective: seven cities by T72. 200 runs each, Hard in every seat (`legatum-sim scenario --ai-params`), [gene off](scenario-gene-off.md), [gene on](scenario-gene-on.md).

| Arm | T64 Settler order | Objective met | Median score |
|---|---|---:|---:|
| `found_now` 0 | move to (30,9), 200 of 200 | 111 of 200 (55%) | 1270 |
| `found_now` 10 | found, 200 of 200 | **200 of 200** | 1390 |

## Gate 3: human-seat benchmark

`hard_vs_human`, seed 310001, 256 reseeded salts x 8 maps (2,048 games per arm), plain Hard rivals, one spot server, batch `found-gate1-1103-115014` (0.15 server-hours). Control `ctl`: `found_now` 0 in every seat; the rivals-only and seat-only arms set the other seats to the control through `other_params` ([arms.json](arms.json), [comparison.json](comparison.json), [compare.py](compare.py)). Delta: change in the designer's-seat win rate; on a rivals-only arm negative means stronger rivals.

| Arm | Seat wins | Delta pp [95%] | Rival cities T50 / T100 | Rival Settlers lost a game | Seat cities T50 / T100 |
|---|---:|---:|---:|---:|---:|
| ctl | 453 | | 3.25 / 5.31 | 0.523 | 2.80 / 5.22 |
| `found_now` 10, rivals only | 445 | -0.39 [-2.00, +1.27] | 3.27 / 5.35 | 0.481 | 2.81 / 5.21 |
| `found_now` 10, seat only | 460 | +0.34 [-0.98, +1.66] | 3.25 / 5.31 | 0.516 | 2.84 / 5.27 |

Both arms point the right way (rivals a little stronger, the seat a little stronger, more cities and fewer Settlers lost where the gene plays) and neither interval excludes zero.

## Decision

Ship `found_now` 10 by default: the rivals are not weaker (the brief's bar), the seat is not weaker, the scenario goes from 55% to 100%, and three of the five cited Tollkeepers/Signalwrights turns change while Swiss T145 still waits. Not clearly stronger: the benchmark effect is within noise, as `found_here` was. Known cost: on master's engine Germany 8.9.23 T31 (at war, a Byzantine army six tiles off, his found -0.54 VP) now founds too; the fresh-city loss looks only one turn ahead. Repin: only the scripted Hard city-fire tape moved.

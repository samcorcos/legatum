# AI fixes 3: three fixes from the coach reviews (2026-10-21)

Branch `ai/ai-fixes-3`, from the coach reviews of the user's 8.9.5 Athens loss and 8.9.6 Swiss win (`docs/playtests/reviews/`). One gene each, landed one at a time. Raw runs: `~/LegatumRuns/cloud/arms9-1020-195609` (human-seat benchmark arms, all three genes; `arms-report.txt` here), `~/LegatumRuns/box/*flip*` and the box job logs (all-AI flip metrics), `~/LegatumRuns/coach-tools/coach-896/` plus `coach-job3.sh` in the throwaway worktree at `playtest-8.9.6` (Swiss branches).

Tools added: `tools/sim/flip-metrics.sh` (N eight-player Hard games with tapes, then `war_metrics`: wars, Peace-then-war flips by the declarer's faction, city churn, wins and Japan's vow by faction), `crates/sim/examples/flip_trace.rs` (the plans at each flip), `tools/sim/scenario-genes.sh` and `legatum-sim scenario --ai-params` (a Scenario Lab scenario with a gene), `tools/cloud/remote.sh ... war_suite` as before.

## 1. Sparta's peace-then-war cycle: `peace_redeclare` (default 2)

What changed: `plan.rs::peace_would_be_broken` and the hook in `diplomacy.rs::wants_peace`. See `game-rules.md` section 10 for the rule.

All-AI flips (`tools/sim/flip-metrics.sh --games 256 --seed 7400001`, Hard, random factions, Standard speed):

| | Control | Hold only (1, 192 games) | Hold and any war fought in the last 8 turns (2) |
|---|---|---|---|
| Wars per game | 47.2 | 46.5 | 36.3 |
| Peaces per game | 55.7 | 54.2 | 38.2 |
| Declarations within 10 turns of a Peace with the target, per game | 13.4 | 13.0 | 8.1 |
| ...as a share of wars | 28.4% | 28.0% | 22.3% |
| ...Sparta | 30.3% | | 22.7% |
| Repeat declarations on the same team within 30 turns | 30.4% | 31.1% | 20.6% |
| Captures per game / per war | 26.9 / 0.57 | 25.9 / 0.56 | 24.7 / 0.68 |
| Captures of a city captured in the previous 20 turns | 36.1% | 35.9% | 35.1% |
| Captures lost back to the old owner within 10 turns | 25.2% | 26.2% | 25.6% |

Human-seat benchmark (8 maps x 128 salts, reseeded dice, 1,024 games per arm; `arms-report.txt`): control 30.1% seat wins; seat-only -0.6 pp [-3.2, +2.1], opponents-only -1.2 pp [-4.5, +2.1]. Neither interval excludes zero.

War suite (`tools/sim/war-scenarios.sh`, 64 runs, genes on in every seat): suite mean 47% -> 48%; `war-golden-horde-thin` (attack check, floor 30%) 67% -> 85%.

Reading: the hold-only version does nothing (a fifth of flips). Extending the same price to any war the AI is still fighting removes 40% of the flips and a quarter of the wars, and the wars that remain capture more per war. The city churn (a capture retaken within 10 turns by its old owner, 25%) did not move: it is not only a Peace-then-war effect.

## 2. Convert a lead: `capture_completes_count` (default 1)

What changed: `crowns.rs::objective_capture_value`. See `game-rules.md` section 10 for the rule.

Human-seat benchmark (same arms batch as section 1): seat-only -0.4 pp [-1.0, +0.1], opponents-only -1.0 pp [-1.9, -0.1] (control 30.1%). The seat's Hard brain with the gene plays as before; the AI opponents with it beat the Hard seat a little more.

8.9.6 Swiss branches (`coach896_whatif` on the throwaway worktree at `playtest-8.9.6` with this branch's `crates/ai` diff applied; his orders (`tape`) in seat 0, 60 runs each; seat 4 is Japan, seat 2 the AI that wins most branches):

| Branch | Control: you / seat 2 | Gene on Japan only | Gene on every AI |
|---|---|---|---|
| T160 | 46 / 14 | 46 / 14 (Japan 1.4 -> 1.6 captures) | **0 / 60** |
| T175 | 50 / 10 | 50 / 10 | **0 / 40** (seat 1: 19, Japan 1) |
| T190 | 53 / 7 | 53 / 7 | 53 / 7 |
| T196 | 49 / 11 | 49 / 11 | 49 / 11 |

Japan alone does not move because a closed Japan declares no war and its campaigns stay near zero (0.1 captures from T196). The other AIs, free to fight, convert: from T160 and T175 the AI that wins these games anyway wins every run.

War suite (genes on in every seat, 64 runs): 47% -> 47%, thin attack check 67% -> 67%.
Scenario Lab (100 runs, every seat on the gene): `japan-big-empire` mean 713.4 both ways, `byzantium-last-point` 15% both ways, `athens-next-points` 643.8 both ways: the positions are 30-turn windows with no count-completing capture inside them.
All-AI (256 games): captures per game 26.9 -> 27.5, wars unchanged (47.2 -> 47.0).

## 3. Hold the vow you close: `vow_hold`, `vow_denial_percent` (landed off)

What changed: `goal_sakoku.rs` (`vow_at_stake`, `vow_closes_the_race`, `sakoku_denial_cost`), `diplomacy.rs::wants_peace` and `treaty_score`, `production.rs::desired_army`, `goal_actions.rs` / `scripted.rs` (the denial percent reaches `sakoku_close_now`). See `game-rules.md` section 10.

A first version gated Peace and the garrison on "the vow takes the team to the target". From the T190 and T196 states of the 8.9.6 Swiss game it never fired: Japan had 4 to 5 VP at T192 (the 9 at T198 was 7 plus the vow), 7 + 2 is still short of 10. Peace now follows the vow itself (closed, holds or scores within 15 turns); the garrison keeps the "within one point of the target" gate.

8.9.6 Swiss branches (Japan seat 4 only, his orders in seat 0, 60 runs each; `jan.py`-style summary of `vp_line`, `caps`, `peace`):

| Branch | Peaces Japan signs | Cities Japan loses | Japan's peak VP | His seat wins |
|---|---|---|---|---|
| T160 control / `vow_hold` | 5.4 / 11.1 | 1.97 / 1.58 | 8.3 / 10.0 | 46 / 47 |
| T175 | 4.4 / 10.0 | 1.98 / 1.40 | 8.2 / 10.0 | 50 / 50 |
| T190 | 4.0 / 11.0 | 2.00 / 1.30 | 8.0 / 10.1 | 53 / 51 |
| T196 | 4.2 / 4.3 | 2.00 / 2.00 | 9.0 / 9.0 | 49 / 49 |

`vow_denial_percent` 20 changes nothing in these branches (Japan has closed by then); 100 at T160 makes Japan stay open, take 9.6 cities and his seat win 2/60.

All-AI (768 games, seed 7500001): Japan wins 24 (3.1%) -> 21 (2.7%); closes 407 -> 406; cities lost while closed 1,232 -> 1,164. With `vow_denial_percent` 20 (256 games): closes 141 -> 39, Japan wins 10 -> 5.
Benchmark arms (vow_hold plus denial 20; Japan is rarely at the table): seat-only -0.7 pp [-1.8, +0.4], opponents-only +0.3 [-1.3, +1.8]. War suite 48% (the same as master with `peace_redeclare`), thin check 85%.

Why it stays off: the intended effect shows (more peaces, fewer cities lost, VP peaks at the target) but nothing downstream moves, and the all-AI games are neutral. The denial price at 20 removes most closes and Japan's wins with them, so it stays 0 too. Open: the countdown's defence (the vow flickers about 6 completions per close and Japan loses 3 cities per close in the all-AI games), and a denial price that uses the chance a rival marches in.

## Reproducing the Swiss branches

The branches replay the user's 8.9.6 Swiss game, so they run on the 8.9.6 engine with this branch's `crates/ai` diff applied: a throwaway worktree at tag `playtest-8.9.6`, `git apply --3way` of `git diff <base> -- crates/ai`, the save's `archive.tape` as `tape.json` and `tapeB.json`, `~/LegatumRuns/coach-tools/coach-896/coach896_whatif.rs` in `crates/sim/examples/`, then `tools/cloud/remote.sh <worktree> --with-uncommitted -- bash coach-job.sh` with `cargo build --release -p legatum-sim --example coach896_whatif` and `coach896_whatif --tape tapeB.json --ai-params CONFIG --games 60 --seed 1 --out-dir OUT`. Configs: `swiss-branches-count.json`, `swiss-branches-vow.json`; summary: `swiss-branches-japan.py OUT/branches.json.gz` (Japan is seat 4).

# Wartime objective assignment: structural test (2026-11-09)

**The gene code is not on master:** it lives only on branch `astra/objective-assignment` (first working slice `65eddfea3`), kept there because the experiment failed. One new gene `objective_assignment` (default 0). This tests F1 from
[`1-ai-code.md`](../../../status/fable-review-2026-11-09/1-ai-code.md), with the F2 limitation
left visible: the plan still prices one city over ten turns. No core rule or default changed;
`RULES_VERSION` did not move.

## Method

The gene-on path runs only for Hard during War. The still-free land units after the emergency
passes are assigned greedily to three objectives: the routed Campaign's city, a visible enemy
field unit, or an own city with positive threat. Each unit/objective pair is priced in an
integer Production-cost proxy. The siege coverage term is convex until sufficient force is
assembled; crossing `campaign_can_take` adds a second prize. Field pairs use the focus forecast;
defence pairs use the existing city threat and defence power. Ordered maps and unit IDs break
ties. The assigned groups run scoped assault/siege/campaign, field-fire or city-defence
executors. If there is no eligible objective, the legacy passes still run. The legacy pass block
is intact at gene 0.

The `focus::best_focus_plan` pass selects its own target, so the field price calls its
target-specific forecast (`focus_plan_on`) through `focus_value_for`. `threat::score` is an
empire-level rival score, not a forecast of attacks on a particular own city; the hold price
uses the existing per-city `Ctx::threat` instead. That is a narrower interpretation of the
proposed threat price and a limitation of this test.

The paired comparisons use the same source and seeds with `off.json` versus `on.json`.
The 19-case war suite uses 50 runs per case, with only the scenario seat on the gene. The two
Lab cards use 50 runs each. The proper Lab comparison keeps every rival on built-in Hard via
`--params-seat-only`; the first all-seats run is retained under `lab/` to show the confound.
The held-out designer-seat check uses Mongols 8.9.25, Celts 8.9.26 and Swiss 8.9.24, five
starts and 48 shared salts per tape. Paired intervals are normal approximations over 50 seed
blocks (war suite), 50 paired card runs, or 144 tape/salt blocks (designer seat); each block
keeps its correlated cases or starts together. `compare.py` recomputes them from the raw runs.

## Results

| Measure | Gene off | Gene on | Paired change, 95% interval |
|---|---:|---:|---:|
| War-suite goals met, mean of 19 cases | 23% | 12% | **−10.8 points [−12.8, −8.9]** |
| Lab Hold Biel, seat only | 35/50 | 4/50 | −62 [−78.7, −45.3] points |
| Lab walled Nicomedia, seat only | 3/50 | 1/50 | −4 [−11.8, +3.8] points |
| Designer-seat held out | 58.1% | 48.9% | **−9.2 [−12.4, −5.9]** points |
| Training round 2 capture, 21 fixed cells × 8 dice | 74/168 = 44.0% [36.8, 51.6] | 74/168 = 44.0% [36.8, 51.6] | 0 points [−10.6, +10.6] (unpaired approximation); every cell's aggregate was identical |

**Fail.** The pass bar required at least twice the war-suite goals met with no loss on
designer-seat held-out. The measured goal rate fell to about half of control and the held-out
score fell clearly. Keep the gene off.

The all-seats Lab run gave Hold Biel 35/50 → 50/50 and Nicomedia 3/50 → 7/50. That is **not**
an isolated improvement: it also changed the opposing Hard AI. With the rivals held fixed,
both cards lose as shown above. The plain-Hard fixed combat comparison was exactly 96/252
captures in both arms; it does not measure the Training policy.
The Training round 2 run used the historical 2,048-rollout budget and eight dice per cell;
both arms had the same 74 captures, 65 plain-Hard captures, 11 fights only Training won,
and two only Hard won. Its baseline differs from the 2026-11-08 42.9% because this is a
same-commit rerun on current code. The intervals in the table are Wilson binomial intervals,
not a paired interval; the saved Training summary has cell counts, not per-fight paired
outcomes across arms.

## Where the assignment loses

| War case | Off → on | Evidence and likely cause |
|---|---:|---|
| Hollow Swiss T76 | 94% → 0% | Mean cities captured 0.94 → 0; attack groups 4.14 → 1.04. The new city objective exists only for a routed `Campaign`, while the old cascade can press through other war plans and conquest nomination. A missing city objective is the likely cause; the case was not turn-traced. |
| Tepid Swiss T220 | 94% → 0% | Mean captures 0.94 → 0 in a three-turn race for Athens. The scoped executors skip `seize_open_city` and several late attack passes, so the army may miss a short capture window. The exact lost order was not turn-traced. |
| Hold Biel | 42% → 12% | Mean cities lost 0.58 → 0.88; enemy units killed 12.02 → 9.58. The assigned hold executor does not reproduce the old sequence of counterattack, ranged fire, wound recovery and late defensive passes. |
| India vs Sparta T142 | 58% → 32% | Mean cities lost 2.98 → 3.62 and isolated losses 1.26 → 2.68. The simplified hold execution leaves a weaker defence. |
| Mongols at Chur T137 | 6% → 38% | Mean captures 0.30 → 0.56 and fewer units lost (8.38 → 6.80). Here a concentrated city group beats the old cascade. |

One gene-on war run reported an illegal shared tile at T163 in India at Chur. The cause was
not traced; the suite counted one bug report and zero with gene off. The test does not change
core movement rules.

The failure is informative for F2: a unit allocator cannot make a `Contain` plan name a city,
and replacing the entire middle pass block exposes the protective work those passes do. A
next version would need a target for every active war plan and objective-scoped tactical
executors that preserve defence and short-window capture behavior. This branch does not turn
on the gene or change the plan horizon.

## Files in this directory

The gene's code (`crates/ai/src/objective_assignment.rs`, hooks in `military.rs` and `focus.rs`, the `objective_assignment` param, the `scenario.rs` harness change and the `designer_seat.py` change) was not merged to master and lives only on branch `astra/objective-assignment`.

- `README.md`, `compare.py` (recomputes `comparison.json` from the saved raw runs), `comparison.json`
- `off.json`, `on.json` (the AI params of the two arms)
- `war-off.json.gz`, `war-on.json.gz` (19-case war suite)
- `designer-heldout.json`, `designer-seat-out/` (designer-seat held-out runs)
- `lab/` (the confounded all-seats Lab run), `lab-seat/` (the seat-only Lab runs, off and on)
- `training-off.json.gz`, `training-on.json.gz` (Training round 2 spot check)

## What was run and its result

- Focused `legatum-ai` test `siege_marginal_value_rises_before_threshold_and_stops_after`: 1 passed.
- Spot war suite, 19 cases × 50 matched seeds per arm: completed; 23% off, 12% on, paired change −10.84 points [−12.76, −8.92].
- Two Lab cards, 50 seeds per arm with the candidate seat isolated: completed; Hold Biel 35 → 4 wins, Nicomedia 3 → 1 win. The first all-seats probe is separately recorded as a confounded run.
- Designer-seat held-out, 3 tapes × 5 starts × 48 salts per arm: completed; 58.1% off, 48.9% on, paired change −9.17 points [−12.42, −5.91].
- Spot Training round 2, 21 fixed cells × 8 dice per arm, with 2,048 rollouts: completed; both arms captured 74/168.
- `python3 docs/sim-results/ai-quality/objective-assignment-1109/compare.py`: passed and reproduced `comparison.json` from saved raw runs.
- `python3 -m py_compile` on `compare.py` and `tools/decision-audit/designer_seat.py`: passed.
- `cargo fmt --all` before the merge and `cargo fmt --all -- --check` after it: both passed.
- `git diff --check`: passed.
- Initial `tools/sim/build-slot.sh -- cargo clippy --workspace --all-targets -- -D warnings`: failed on a collapsible `if` lint; the code was corrected and formatted. Reruns did not submit a box job before its repeated stop cycles, so there is no final result.
- `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked`: attempted, but no box job was submitted before the repeated stop cycles; no result.
- Post-merge `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked -- pinned` and `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked --test determinism`: attempted, then stopped before box submission to limit further spend; no result.

## Not verified

- The final full clippy, release-profile workspace tests, and post-merge determinism/pin gate were not completed because the cloud build box stopped during job startup. Do not treat them as passed. In particular, the default-off hash identity is supported by the unchanged gene-off branch and old pin values, but has not been verified on this merged commit.
- The specific lost order on the worst war cases and the cause of one illegal shared tile in the gene-on suite were not turn-traced. The causes in the case table are supported inferences from the aggregate telemetry.
- The experiment has not been enabled by default, released, or played through in the Mac UI. There is no authoritative AWS cost readback; the spot batches were bounded to 30 minutes each and the total spend was intended to stay below $5.

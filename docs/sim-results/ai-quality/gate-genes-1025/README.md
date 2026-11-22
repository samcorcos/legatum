# AI-gene default gate, 2026-10-25

Integration branch: `astra/gate-ai-genes-1025`, merging the Athens and Sparta analysis branches into master `acc7ddfc0`. The four candidate AI decisions are `raid_arrival_weight=100`, `founding_survival_weight=200`, `local_mongol_guard=1`, and `capture_vp_accounting=1`; all four started at default 0. This changes AI choices, not game rules. The benchmark binary was built before later master documentation-only commits; the merged Rust code and Cargo inputs are byte-identical.

## Method

`arms.json` runs a single merged-tree `hard_vs_human` binary with an all-off control, one seat-only and one all-seats arm per gene, plus both modes with all four on. The benchmark uses the last gate's eight fixed human-seat maps, salts 1–128 per map (1,024 games per arm), Hard, and `LEGATUM_HVH_RESEED_DICE=1`. The master comparison is the already completed `gate-landable-1025-master-1025-202818` control on the same seeds. Its source checkout, commit `0a73cfc5`, has the same `crates/`, `Cargo.toml`, and `Cargo.lock` as current master. The analysis requires the merged branch's all-off outcomes to match master game by game. `analyze.py` then resamples the same salt index across all eight maps 10,000 times for paired 95% intervals. Intervals describe variability on these maps, not unseen maps.

A gene passes when its seat-only point estimate is no worse than −1 percentage point and its all-seats point estimate is no worse than −1.5 percentage points against off/master. The all-four arms check interaction. Raw candidate batch results are retained in `~/LegatumRuns/cloud/`; `arm-deltas.json` is the reproducible summary.

## Results

The merged-tree all-off control reproduced master **game by game**: 399/1,024 human-seat wins. The table shows candidate minus master in percentage points, with paired 95% salt-bootstrap intervals. These are point-estimate gates; wide intervals still indicate uncertainty on other seeds and maps.

| Gene at candidate value | Seat only, pp [95%] | All seats, pp [95%] | Individual gate |
| --- | ---: | ---: | --- |
| `raid_arrival_weight=100` | +0.49 [−1.95, +2.93] | −1.37 [−4.49, +1.86] | Pass, close to all-seats floor |
| `founding_survival_weight=200` | −1.07 [−3.52, +1.37] | +0.49 [−2.93, +3.91] | **Fail seat floor** |
| `local_mongol_guard=1` | −0.68 [−1.56, +0.20] | −0.20 [−1.17, +0.78] | Pass |
| `capture_vp_accounting=1` | +0.20 [−0.49, +0.88] | +0.00 [−0.88, +0.88] | Pass |
| All four | +0.20 [−2.54, +3.03] | −1.66 [−4.69, +1.47] | **Fail all-seats floor** |

Because the four-gene bundle fails and three genes individually pass, `subset-arms.json` measures every pair of those three and their three-gene combination on the same 1,024 seeds. Its all-off control again reproduces master game by game. Complete paired results are in `subset-deltas.json`.

| Combination | Seat only, pp [95%] | All seats, pp [95%] | Gate |
| --- | ---: | ---: | --- |
| Raid + guard | −0.59 [−2.93, +1.66] | −1.37 [−4.49, +1.66] | Pass |
| Raid + capture VP | +0.59 [−1.86, +3.03] | −1.27 [−4.40, +1.95] | Pass |
| Guard + capture VP | −0.59 [−1.66, +0.49] | −0.29 [−1.56, +0.98] | Pass |
| **Raid + guard + capture VP** | **−0.49 [−2.93, +1.86]** | **−1.37 [−4.40, +1.66]** | **Pass** |

**Default decision:** enable `raid_arrival_weight=100`, `local_mongol_guard=1`, and `capture_vp_accounting=1`; keep `founding_survival_weight=0`. The selected three-gene all-seats arm is only 0.13 pp above the −1.5 pp floor. These are same-seed point-estimate gates, and the intervals are wide; the selected combination has not been tested on fresh maps or seeds. The all-four arm misses its all-seats floor by 0.16 pp, also within sampling noise. The branch records the predeclared threshold outcome rather than claiming a reliable win-rate gain.

Candidate raw batches: `~/LegatumRuns/cloud/gate-ai-genes-1025-fast-1025-221629` and `~/LegatumRuns/cloud/gate-ai-genes-1025-subsets-1025-234328`. Master raw batch: `~/LegatumRuns/cloud/gate-landable-1025-master-1025-202818`.

The all-four-on war regression check passed: Golden Horde thin-treasury captured Aarau in **40/64 runs (62%)**, above the **30%** floor, with **0 bugs**. Its complete per-run result is `war-all-on.json`.
The selected three-gene default also passed the same check at **40/64 (62%)**, with **0 bugs**; its per-run result is `war-selected-defaults.json`.

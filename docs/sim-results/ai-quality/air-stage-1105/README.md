# Air-stage public read — 2026-11-05 India review

Gene `air_stage_read` compares the old Modern-era Fighter certainty (`0`) with public aircraft sightings and reported air battles plus prize-city Fighter guard value (`1`). Both arms used the same `cff0a6b47a66ddf696a7c8962293dda8463495b4` sim binary (SHA-256 `0d013fa13644be133ab71557648eb2bb0be13855596eb56c0377ad857e3eb133`). `params-0.json` and `params-1.json` are the explicit AI overrides. The gene default is **1**: the old certainty prevents a decisive Bomber buy even when no Fighter has been observed, while the new estimate permits one without assuming a hidden hangar is empty.

## AI-only census

`tools/cloud/run-batch.sh` ran 4,096 eight-player Hard games per arm, seeds 9311003–9315098, on one c8g.4xlarge spot server per arm. Both completed with **0 panics**. Results: `~/LegatumRuns/cloud/air-stage-corrected-ctl-1105-234232` and `~/LegatumRuns/cloud/air-stage-corrected-gene1-1105-234311`. `census-comparison.json` comes from `docs/sim-results/ai-quality/swiss-playbook-1105/census.py` with control then candidate.

The per-seat win rates barely moved: India 8.70% → 8.51% (paired difference −0.19 percentage points, bootstrap interval −0.49 to +0.11); Switzerland 10.75% → 11.16% (+0.42, 0.00 to +0.83). The other seven faction differences range from −0.16 to +0.11 points, all with intervals crossing zero. The strongest-to-weakest win-rate ratio moved from 3.15 to 3.12. This census checks broad balance, not whether Hard makes the desired Bomber or guard decision in the specific reviewed save.

## Human-seat benchmark

`tools/cloud/run-batch.sh --bin hard_vs_human` ran `arms.json` with 256 paired salts on each of eight maps (2,048 games per arm, `LEGATUM_HVH_RESEED_DICE=1`). The binary SHA-256 was `abd0e44042166adf5e3469f69cd6294db039ac24d9f4aa5a000c9ae0b5e72802`. Results: `~/LegatumRuns/cloud/air-stage-hvh-1105-234354`. `comparison.json` uses `tools/sim/quality_gate.py::compare` via the Swiss gate's `compare.py`.

| Arm | Human wins | Hard strength effect | Gate decision |
| --- | ---: | ---: | --- |
| Gene off, all seats | 472 / 2,048 | control | — |
| Gene on, rivals only | 473 / 2,048 | −0.05 pp; simultaneous interval −0.73 to +0.63 pp | **noninferior** versus −1.5 pp floor |
| Gene on, human seat only | 476 / 2,048 | +0.20 pp observed | **inconclusive**; sparse paired discordance leaves the interval unresolved |

The seat arm does not establish a gain. The rivals arm rules out the gate's harm floor, and the AI-only census shows no material faction shift. Together with the known hard veto bug, that supports defaulting the gene on while retaining gene 0 for a clean ablation. A focused AI test checks that a Modern rival without a sighted Fighter carries 25% estimated interception risk, and a core test checks that a hidden attacking aircraft's battle report does not disclose its kind.

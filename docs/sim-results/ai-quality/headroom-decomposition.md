# Hard seat headroom: RNG versus policy

## Result

Hard's archived 8.8 control won **142/512 games (27.7%)**. Resampling only the recorded seat's AI stream across offsets 0–7 raised the per-salt best-of-eight to **287/512 (56.1%)**. The archived best-of-eight policy arms won **338/512 (66.0%)**. Policy headroom over the RNG best was therefore **+10.0 percentage points [95% paired salt-bootstrap CI: +5.7, +14.3]**.

The bootstrap resamples the 64 salt slots independently within each map, keeping the eight arms paired at every sampled salt, then averages the eight maps equally. The interval covers salt variation for these fixed maps and arms; it does not cover new maps or uncertainty from choosing a different set of policy arms.

| Map | Hard, offset 0 | RNG best of 8 | Policy best of 8 | Policy headroom [95% CI] |
| --- | ---: | ---: | ---: | ---: |
| 8.3 Switzerland | 1/64 (1.6%) | 3/64 (4.7%) | 24/64 (37.5%) | +32.8 pp [+20.3, +46.9] |
| 8.4 Sparta | 30/64 (46.9%) | 46/64 (71.9%) | 61/64 (95.3%) | +23.4 pp [+10.9, +35.9] |
| 8.4 Japan | 31/64 (48.4%) | 51/64 (79.7%) | 61/64 (95.3%) | +15.6 pp [+6.3, +26.6] |
| 8.4 Celts | 22/64 (34.4%) | 47/64 (73.4%) | 48/64 (75.0%) | +1.6 pp [−12.5, +15.6] |
| 8.5 Mongols | 0/64 (0.0%) | 1/64 (1.6%) | 3/64 (4.7%) | +3.1 pp [−3.1, +9.4] |
| 8.5 Switzerland | 28/64 (43.8%) | 59/64 (92.2%) | 56/64 (87.5%) | −4.7 pp [−15.6, +6.3] |
| 8.6 Germany | 19/64 (29.7%) | 39/64 (60.9%) | 44/64 (68.8%) | +7.8 pp [−6.3, +21.9] |
| 8.6 Sparta | 11/64 (17.2%) | 41/64 (64.1%) | 41/64 (64.1%) | +0.0 pp [−15.6, +15.6] |
| **Overall, equal map weight** | **142/512 (27.7%)** | **287/512 (56.1%)** | **338/512 (66.0%)** | **+10.0 pp [+5.7, +14.3]** |

Across the 28 pairs of RNG-offset arms, the mean pair changed **102.2 of 512 outcomes (20.0%)** and had mean win-set Jaccard **0.465**. This is substantial chaos: the best offset for a salt wins more than the fixed control on many slots. But the policy best-of-eight still beats the RNG best-of-eight, with the overall interval above zero. The next AI effort should prioritize **plan choice**, since policy variants retain measurable headroom beyond RNG luck. Execution and variance reduction still matter: RNG offsets alone add 28.3 pp over the control, and they erase most of the policy gain on 8.5 Switzerland.

## Arms and method

The eight RNG arms are the Hard seat with `--seat-ai-rng-offset K`, K = 0…7, on all eight checked-in maps and salts 1–64. Each ran as a separate AWS spot batch with one Linux ARM `hard_vs_human` binary. The offset is included only in the recorded seat's AI seed derivation. Offset zero uses the exact historical seed derivation. Its report and complete per-game JSON compare equal to the archived 8.8 Hard control (`rebaseline-88-hard`), not merely in win count.

The eight archived 8.8 policy arms are Hard and Hard + Search capital guard; Hard and Hard + Search Sparta war genes; Hard committed advance; Hard + Search peace-follows-plan; Hard + Search snipe-payback; and Hard Japan Settler safety. The two Sparta gene files are the archived all-seats runs; the other candidate files apply their changed policy to the recorded seat. The decomposition and intervals were computed with `tools/sim/hard-vs-human-compare.py headroom`, using the same 8-map/salt pairing and equal map weight. The complete command inputs and calculated values are saved in [`runs/headroom-decomposition/decomposition.json`](../hard-vs-human/runs/headroom-decomposition/decomposition.json).

Each offset's report, matched game outcomes, AWS batch record, and post-batch status are in [`hard-vs-human/runs/headroom-decomposition/`](../hard-vs-human/runs/headroom-decomposition/). The batches used `AWS_PROFILE=<launcher-profile> tools/cloud/run-batch.sh --max-hours 1 --bin hard_vs_human`; `tools/cloud/status.sh` ran after each. The built binary SHA-256 is `fc0003d080d4e37248d31377be92a3ba9aa37b1459d05b40c96095418a139202` on commit `e04bc968` plus this worktree's changes.

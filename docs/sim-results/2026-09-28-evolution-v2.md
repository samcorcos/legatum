# Evolution v2: noise calibration and smoke evidence (2026-09-28)

Branch: `astra/evolution-v2`, based on master `ff48dcc67bcfe3b41c8c4615c91966d0efdb985d`.
The simulator reports that revision with `worktree_dirty: true` because these decision-parameter
changes were under test. No core rules changed. Proxy v2 explicitly started at War; ordinary
league games started at Peace, with current diplomacy.

## Before: all 16 overnight generation-zero candidates

Source: `~/LegatumData/evolve/2026-09-28-0313/gens/g0/c*.json.gz`, referenced by the original
`docs/sim-results/evolve-2026-09-28-0313/run.json` in the main checkout. The original run used
128 Proxy and 32 league games per candidate. We recomputed whole-fitness uncertainty with 256
whole-game bootstrap replicates, preserving correlated seats and stratifying by league opponent.
The fitness weights are unchanged. No new simulations were run for this retrospective analysis.

| Candidate | Recorded fitness | Measured SE, 128+32 | Projected SE, 512+128 |
| --- | ---: | ---: | ---: |
| cand-g0-c0 | 0.7497 | 0.06535 | 0.04150 |
| cand-g0-c1 | 0.9133 | 0.07637 | 0.04041 |
| cand-g0-c2 | 0.7557 | 0.06638 | 0.03261 |
| cand-g0-c3 | 0.7968 | 0.06833 | 0.03465 |
| cand-g0-c4 | 0.7814 | 0.07252 | 0.03891 |
| cand-g0-c5 | 0.7968 | 0.06551 | 0.03369 |
| cand-g0-c6 | 0.8100 | 0.06692 | 0.04007 |
| cand-g0-c7 | 0.7927 | 0.06410 | 0.03436 |
| cand-g0-c8 | 0.8478 | 0.07409 | 0.04166 |
| cand-g0-c9 | 0.8572 | 0.06966 | 0.04695 |
| cand-g0-c10 | 0.7988 | 0.06487 | 0.03834 |
| cand-g0-c11 | 0.8467 | 0.06551 | 0.03999 |
| cand-g0-c12 | 0.7781 | 0.06483 | 0.03609 |
| cand-g0-c13 | 0.8710 | 0.06875 | 0.04739 |
| cand-g0-c14 | 0.7431 | 0.06402 | 0.03382 |
| cand-g0-c15 | 0.9627 | 0.07304 | 0.04554 |

Median measured SE: **0.06665**. Median projected SE: **0.03945**.
The after column is a projection from the same observed distributions, not independent new
candidate games. Its reduction is less than a square-root-of-four factor because the full
fitness includes discontinuous dominance and capped faction penalties.

The old 64-game checkpoint's Proxy-only binomial SE at 59% wins is about 0.062; 128 games near
50% give 0.044, not the previously documented 0.035. The 0.59 → 0.50 → 0.64 checkpoint sequence
therefore needs uncertainty intervals before being interpreted as learning or regression.

Reproduction:

```sh
python3 tools/evolve/calibrate.py \
  ~/LegatumData/evolve/2026-09-28-0313/gens/g0 \
  --proxy-games 512 --league-games 128 --bootstrap 256 --out /tmp/evolution-noise.json
```

## After: actual current-rules stock-AI calibration

A single full-size candidate evaluation ran **512 war-start Proxy games (100001–100512)** and
**128 Peace-start stock-versus-stock league games (500001–500128)**. Its first 128 Proxy and 32
league games also provide a same-rules before estimate; these samples overlap and are not two
independent experiments. No held-out seeds were used to choose the sampling budget.

| Measurement | Fitness SE |
| --- | ---: |
| Current rules, first 128+32 games | 0.07227 |
| Current rules, all 512+128 games | 0.04173 |

Full-size fitness: 0.9797; 95% bootstrap-normal interval
[0.8979, 1.0615].
Proxy win rate: 0.5020; candidate-side league win rate:
0.5938. This measures one stock candidate, not every future genotype.

The calibration deliberately used the same stock file on both sides of the league. Both sides
therefore had the same report label; candidate ownership was reconstructed using the simulator's
exact `(seat + game index) % 2 == 0` allocation before computing fitness/SE. Labels do not affect
the brain or outcomes. Production harness candidates and incumbents have distinct names.

Raw measurement cache: `/tmp/astra-evolution-v2-calibration/data/current-calibration/gens/g0/c0.json.gz`.
Corrected compact measurement: `/tmp/astra-evolution-v2-current-noise.json`; attribution and the
same-rules before/after estimates are recorded there. Large reports/tapes remain outside git.

The new default is **six candidates + one fresh incumbent**, each at 512+128. A 0.042 marginal SE
means an unpaired difference near 0.12 is needed for a conventional 95% separation. Pairing can
reduce that error; each actual paired SE is logged. This budget does **not** resolve 1–3 point
fitness gains. CMA-ES ranks drive search, while promotion requires an adjusted paired interval
above zero. The final independent held-out comparison remains mandatory.

## CPU budget on this 16-core Mac

The full-size calibration used **1710.8 child CPU-seconds** and **950.1 wall-seconds**
with eight simulator threads. This includes actual simulator CPU but excludes Python bootstrap
CPU; the wall time includes bootstrap. Other jobs drove load averages above 270–300 during the
measurement, so that wall time is not an idle-machine benchmark.

- Current-rules simulation budget per default generation: 7 × 1710.8 =
  **11976 CPU-seconds**. At 14 effective cores the ideal upper estimate is
  **4.21 generations/hour**.
- Seven evaluations on two simulator workers leave one final partial wave. Allowing for that
  scheduling effect and bootstrap/checkpoint overhead gives **about 3.5–4 generations/hour**
  on an otherwise available 16-core machine. Periodic coaching and final validation also consume
  the overnight budget.
- On the same per-game CPU cost, the old 16 × (128+32) budget would be about
  **7.36 generations/hour** before overhead. The historical older-rules
  measurement estimated about 9.8/hour; do not mix those rules/performance estimates.
- `--nice` requests OS priority 10. This restricted session denied `setpriority`; the harness
  recorded the error and used its half-core fallback. If limited to eight otherwise available
  cores, expect roughly half the full-machine rate; shared load can make it much slower.

These are throughput estimates, not guarantees during interactive play. `cpu_s`, `wall_s`,
actual niceness, thread budget, per-candidate SE, and paired confidence bounds are in the run log.

## Real smoke and checks

The two-generation smoke completed with exit **0**. It used four candidates plus a fresh
incumbent, 32 Proxy + 16 league games per evaluation, 64 bootstrap draws, and four simulator
threads. It ran from 08:43:16 to 09:27:34 local time under heavy shared load; no long evolution
run, model call, queue submission, or film generation was started.

The selected genome is `cand-g0-c1`. It replaced stock in generation zero with paired delta
0.3335 and adjusted CI [0.0583, 0.6087]. Its freshly measured fitness fell from 1.1527 to 1.0374
on generation one's new seed block; the historical score was not retained. No challenger
qualified for replacement in generation one. The exported file was checked against that
incumbent's exact personality and decision values, not the CMA mean.

Search, genes, starts, and final validation were exercised by this real smoke. Runtime binary
freezing and cache-collision hardening added during the smoke were additionally checked by the
24-test Python suite and a separate **two-game real frozen-binary launch**. That launch printed
69 genes and recorded Proxy v2 with the War start. All simulator code was unchanged between
those runtime harness hardening steps.

### Held-out result: don't-adopt

Each speed used 32 games for each Proxy arm, 32 Peace head-to-head games, and 32 self-play games
for each AI (320 validation games total). These samples are smoke evidence, not the default
2,000-game-per-speed faction qualification.

| Speed | Stock Proxy win rate (95% CI) | Evolved Proxy win rate (95% CI) | Paired reduction (95% CI) | Evolved Peace league win rate (95% CI) |
| --- | --- | --- | --- | --- |
| quick | 0.562 [0.393, 0.718] | 0.500 [0.336, 0.664] | 0.062 [-0.185, 0.293] | 0.531 [0.364, 0.691] |
| standard | 0.656 [0.483, 0.796] | 0.562 [0.393, 0.718] | 0.094 [-0.190, 0.352] | 0.562 [0.393, 0.718] |

Both improvement intervals cross zero. Evolved self-play had a raw best/worst faction ratio
of **3.5 at both speeds**, with a conservative 95% upper bound of **40.56** at these small
sample sizes. Winner entropy was 0.531 at both speeds; its intervals were [0.326, 0.633] at
Quick and [0.408, 0.619] at Standard. **Wide remained dominant** in both evolved self-play
samples. There were **no stalls or panics**. Improvement, faction balance, and strategic
counterplay therefore remain unestablished; the digest correctly says **don't-adopt**.

Small committed evidence:

- [Noise measurements and frozen-binary receipt](evolution-v2-2026-09-28/noise.json)
- [Smoke settings and incumbent comparisons](evolution-v2-2026-09-28/smoke.json)
- [Exact selected genome, not approved for adoption](evolution-v2-2026-09-28/smoke-params.json)
- [Full held-out intervals, faction counts, and matchup matrices](evolution-v2-2026-09-28/validation.json)
- [Harness-written digest with the adoption verdict](evolution-v2-2026-09-28/smoke-digest.md)

All requested checks completed:

| Command | Result | Exit code |
| --- | --- | --- |
| `cargo fmt --all` | Passed | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed | 0 |
| `cargo test --workspace --release --no-fail-fast` | 831 passed, 0 failed, 4 ignored | 0 |
| `cargo test --workspace` | 831 passed, 0 failed, 4 ignored | 0 |
| `cargo test -p legatum-sim --test determinism` | 11 passed | 0 |
| `python3 -m unittest discover -s tools/evolve -p 'test_*.py'` | 24 passed | 0 |

The debug workspace test used `CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4` to leave CPU headroom.

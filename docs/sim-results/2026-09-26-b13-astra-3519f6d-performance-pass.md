# B13 performance measurements — source 3519f6d983fd3af34e5bac9bd1cb48e34fbb8dab

Measurements from the fixed public-API harness; six games, seeds 1 and 7, reused for every run. The 48×32 cases cover separate teams and shared seats 0/1. The 72×44 case covers the older PRD dimensions. All seats use the unchanged scripted brain. These are representative workloads, not the historical 200-game campaign.

Full metadata, raw samples, distributions, corpus checksums and exit receipts: [2026-09-26-b13-astra-3519f6d-performance-pass.json](2026-09-26-b13-astra-3519f6d-performance-pass.json). Method and commands: [performance-pass.md](../analysis/performance-pass.md).

The shorter alternating series has six process runs per binary, 11 samples per snapshot/metric/run, 3 operations per timed sample, and 5 discarded warm-up samples. That is 66 retained batch samples (198 operations) per snapshot/metric/binary. Every snapshot receives equal weight in the geometric mean below. Negative percentages mean less elapsed time.

| Metric | Short series: change | Snapshot range | Long series: change |
| --- | ---: | ---: | ---: |
| resolve | -24.4% | -32.0% to -16.7% | -20.2% |
| views | -2.5% | -4.0% to -0.1% | +1.5% |
| ai | +0.4% | -3.0% to +7.7% | +8.6% |
| total | -5.5% | -10.3% to -1.6% | +4.3% |
| observe | -66.2% | -69.5% to -62.4% | -63.2% |
| visible | -78.7% | -81.2% to -76.1% | -78.4% |
| clone | +1.4% | -1.9% to +4.2% | +4.1% |
| titles | +1.7% | -2.4% to +4.1% | +3.3% |

The earlier long series used three runs per binary, 31 samples/run, 5 operations/sample, and the same five warm-up samples (93 retained batch samples per snapshot/metric/binary). Its total result reversed sign while unchanged AI/clone/title code also slowed. Short-series aggregate total changes across the six pairs were -1.2%, -4.2%, -4.9%, -3.9%, -7.4%, -9.9%. Observation and visibility improvements are large in both series. The smaller total/view effects remain sensitive to machine conditions; the worst short-series AI snapshot is +7.7%, with both positive and negative changes across its repeated runs. This is not proof that every component or workload gets faster.

Per-snapshot medians below are microseconds per operation, baseline → optimized. Early is turn 5, middle is turn 50, and late is the last active turn before the recorded game finishes.

| Case / stage (turn; units/cities) | Resolve | All-seat views | AI on ready views | Total |
| --- | ---: | ---: | ---: | ---: |
| prd72-seed1 / early (5; 16/8) | 220.3 → 149.8 | 217.3 → 212.5 | 985.2 → 1003.5 | 1461.2 → 1415.9 |
| prd72-seed1 / middle (50; 19/25) | 329.7 → 239.5 | 411.6 → 407.4 | 1431.8 → 1455.9 | 2257.8 → 2179.6 |
| prd72-seed1 / late (139; 66/43) | 616.6 → 508.5 | 924.0 → 906.2 | 952.1 → 991.3 | 2563.0 → 2462.8 |
| prd72-seed7 / early (5; 16/8) | 193.1 → 139.4 | 214.7 → 214.5 | 986.6 → 987.9 | 1421.1 → 1387.4 |
| prd72-seed7 / middle (50; 24/21) | 278.5 → 212.8 | 359.9 → 358.7 | 1537.3 → 1551.1 | 2226.9 → 2158.9 |
| prd72-seed7 / late (108; 59/43) | 641.2 → 483.9 | 825.3 → 794.4 | 814.4 → 790.2 | 2334.2 → 2168.3 |
| sim48-seed1 / early (5; 16/8) | 126.3 → 91.4 | 148.7 → 143.9 | 472.3 → 469.2 | 757.6 → 713.8 |
| sim48-seed1 / middle (50; 19/20) | 232.1 → 174.0 | 291.5 → 280.6 | 634.5 → 619.5 | 1200.8 → 1093.9 |
| sim48-seed1 / late (140; 55/35) | 453.0 → 360.3 | 658.2 → 634.8 | 597.3 → 587.6 | 1785.4 → 1600.9 |
| sim48-seed7 / early (5; 16/8) | 125.9 → 92.8 | 147.8 → 144.4 | 482.7 → 481.6 | 773.5 → 728.1 |
| sim48-seed7 / middle (50; 17/17) | 188.2 → 132.5 | 255.2 → 248.8 | 381.6 → 378.8 | 844.0 → 784.5 |
| sim48-seed7 / late (141; 46/29) | 335.9 → 273.0 | 526.1 → 508.3 | 744.9 → 738.2 | 1630.3 → 1542.8 |
| team48-seed1 / early (5; 16/8) | 126.3 → 93.2 | 150.9 → 144.8 | 502.6 → 489.5 | 795.5 → 740.3 |
| team48-seed1 / middle (50; 18/20) | 219.1 → 165.7 | 289.4 → 281.6 | 601.5 → 628.5 | 1146.8 → 1115.6 |
| team48-seed1 / late (123; 56/35) | 403.5 → 336.2 | 639.6 → 613.7 | 500.2 → 497.2 | 1587.3 → 1477.3 |
| team48-seed7 / early (5; 16/8) | 127.5 → 94.0 | 149.5 → 144.8 | 508.9 → 504.5 | 787.9 → 775.1 |
| team48-seed7 / middle (50; 18/17) | 191.1 → 139.7 | 257.9 → 250.6 | 519.1 → 522.7 | 998.0 → 929.9 |
| team48-seed7 / late (132; 43/29) | 326.4 → 271.5 | 515.4 → 510.4 | 542.8 → 584.4 | 1429.4 → 1335.9 |

Observation distributions in µs per update, median [P10, P90], n=66 batch averages on each side:

| Case / stage | Baseline | Optimized |
| --- | ---: | ---: |
| prd72-seed1 / early | 26.45 [25.47, 29.32] | 8.06 [7.38, 8.88] |
| prd72-seed1 / middle | 34.89 [33.14, 38.10] | 11.43 [10.96, 13.22] |
| prd72-seed1 / late | 49.81 [48.21, 54.93] | 16.49 [15.33, 19.85] |
| prd72-seed7 / early | 26.78 [25.85, 29.18] | 8.42 [7.82, 9.65] |
| prd72-seed7 / middle | 34.17 [32.69, 36.24] | 11.03 [10.22, 12.76] |
| prd72-seed7 / late | 48.94 [47.38, 55.67] | 15.98 [15.03, 17.26] |
| sim48-seed1 / early | 16.37 [15.60, 18.31] | 5.46 [5.00, 6.06] |
| sim48-seed1 / middle | 22.18 [20.83, 24.04] | 7.56 [6.75, 8.56] |
| sim48-seed1 / late | 34.58 [33.19, 37.99] | 11.01 [10.26, 12.60] |
| sim48-seed7 / early | 16.40 [15.72, 19.19] | 5.53 [4.94, 6.10] |
| sim48-seed7 / middle | 19.85 [18.85, 23.21] | 6.76 [6.07, 7.81] |
| sim48-seed7 / late | 30.20 [28.93, 33.00] | 9.90 [9.15, 11.18] |
| team48-seed1 / early | 16.42 [15.74, 17.79] | 5.85 [5.33, 6.39] |
| team48-seed1 / middle | 21.75 [20.88, 23.40] | 8.18 [7.28, 9.36] |
| team48-seed1 / late | 34.44 [33.49, 37.61] | 12.51 [11.64, 13.65] |
| team48-seed7 / early | 16.29 [15.72, 17.64] | 5.85 [5.32, 7.07] |
| team48-seed7 / middle | 20.18 [19.32, 21.78] | 7.20 [6.64, 8.57] |
| team48-seed7 / late | 29.73 [28.92, 32.75] | 10.63 [10.00, 12.01] |

Optimized `legatum_wasm_bg.wasm`, with identical release/wasm-bindgen/wasm-opt `-O` settings:

| Bytes | Baseline | Optimized | Difference |
| --- | ---: | ---: | ---: |
| Raw | 1,045,161 | 1,045,220 | +59 (+0.01%) |
| gzip level 9, mtime 0 | 367,128 | 367,238 | +110 (+0.03%) |

All 783 recorded turn hashes and 789 complete state/all-seat-view JSON snapshots match. All three committed 40-turn native/WASM tapes pass unchanged, including save/load. No fixture, policy, balance, serialized-state, rule, view, app or other-worktree files were changed.

| Check | Exit code | Result |
| --- | ---: | --- |
| Formatting | 0 | Pass |
| Strict all-target Clippy | 0 | Pass |
| Workspace tests | 101 | Same five socket-bind permission failures as baseline |
| Visibility differential tests | 0 | Pass |
| Complete cross-build JSON comparison | 0 | Pass |
| Native debug tape tests | 0 | Pass |
| Native release tape tests | 0 | Pass |
| Workspace excluding sandbox-blocked server | 0 | Pass |
| Optimized WASM build | 0 | Pass |
| Node tape check | 0 | Pass |
| Git staging | 128 | Git index.lock creation denied; commit blocked |

Baseline failures are separate: `cargo test --workspace` exited 101 for the same five `crates/server/tests/protocol.rs:27` socket-bind failures. The first `wasm-pack` attempt exited 1 because its default cache was unwritable; both compared builds passed after using installed tools and a cache under target/. Initial harness lint errors (two warnings promoted to errors, exit 101) were fixed before any timing and are not pre-existing repository failures. Git staging exited 128; no commit or merge occurred. The scoped patch and requested commit message are in `target/performance-pass/` for the lead.

# Test gate timings and worker counts (measured 2026-10-18)

Measurements behind the three test gates (D13), the Playwright worker counts and the Rust "extended" list.
Raw reports: `tools/ops/pw-sweep.sh` (Playwright), `tools/ops/rust-test-durations.py` (Rust), both rerunnable.

## Playwright worker sweep (CL19)

Unchanged suites on the 64-core box, commit `3d3a1d06a`, run with `tools/ops/pw-sweep.sh`. **The box was never quiet:** the load
average was 50 to 400 on 64 cores for the whole sweep (other agents' verifies, lanes and combat-lab sweeps; see the load columns), so
these numbers say how the suites behave under contention, which is what a real gate meets, not what an idle box would do. Rerun
the sweep when `box.sh util` shows a quiet hour.

Chromium, all `game` specs (655 tests, 488 screen-tour shots):

| Workers | Wall | Load before / after | Passed | Failed | Failure classes |
| --- | --- | --- | --- | --- | --- |
| 12 | 1115 s | 315 / 250 | 641 | 6 | assertion 4, timeout 1, other 1 |
| 24 | 1096 s | 104 / 315 | 629 | 18 | assertion 9, timeout 6, other 2, transport 1 |
| 32 | 910 s | 250 / 57 | 634 | 13 | assertion 8, transport 3, timeout 1, other 1 |
| 48 | 896 s | 57 / 47 | 607 | 40 | assertion 15, other 23, transport 2 |

WebKit, screen tour plus exit-to-title (the release check's set):

| Workers | Wall | Load before / after | Passed | Failed | Shots |
| --- | --- | --- | --- | --- | --- |
| 4 | 1505 s | 104 / 193 | 115 | 14 | 444 |
| 8 | 647 s | 193 / 148 | 124 | 5 | 465 |
| 16 | 435 s | 148 / 344 | 100 | 29 | 383 |

Reading: Chromium's wall time is flat from 12 to 48 workers (the cores, not the workers, are the limit) while failures grow with
the worker count, so more workers only add timeouts; the same suite summed 2664 s of test time for the tour at 12 workers against
1400 s in earlier lightly loaded runs. WebKit scales to 8 workers (2.3x faster than 4) and breaks down at 16. The previous release
check ran Chromium at 6 workers (720 s in the 8.9 check) and WebKit in a Mac container at 1 worker (3340 s), so WebKit was the
critical path of the release check. Defaults: WebKit 8 workers (`LEGATUM_WEBKIT_WORKERS`, `tools/cloud/box/webkit.sh`); Chromium
follows the job's core budget (`tools/cloud/box/budget.sh`, `LEGATUM_PW_WORKERS`), which stays in the stable region at every
occupancy. Failures that appeared at every worker count (12, 24 and 32) are not worker-count effects:
`contrast.spec.ts` "Atlas: the tech unlock card..." (CDP `CSS.forcePseudoState` node lost) and `keys.spec.ts` "arrows pan and
Return selects decisions..." (see CH26) failed in all three Chromium runs.

## Rust tests per binary (CL23)

Median of six box verifies (`python3 tools/ops/rust-test-durations.py <logs> --top 30`): 73 binaries, 1871 tests, 570 s of summed
test time. One test, `legatum-sim` `runner::tests::opening_league_twenty_standard_games_finish_without_panics_or_stalls`, took
272 to 291 s on one thread: the test step's wall time was that one test. Next: `legatum-wasm::difficulty_fairness` 31 s,
`legatum-server::protocol` heartbeat deadline 30 s (a real 30 s wait), `legatum-sim::hold_metrics` 15 to 21 s, `legatum-core::map_sizes`
10 to 12 s. Everything else is under 10 s. The tests listed in `tools/astra/extended-rust-tests.txt` are skipped by the fast profile
(about 570 s down to about 130 s of work); nothing is deleted. The opening-league test now plays its 20 games side by side.

## Verify step durations before the change (box, medians of 89 to 105 runs, 2026-10-10 to 10-18)

app-playwright 552 s (max 2940 s), app-wasm 198 s, repin-merge 159 s, test 147 s, sim-build-branch 114 s, app-vitest 65 s,
app-server 64 s, clippy 59 s, smoke-branch 38 s, app-build 18 s, app-tsc 17 s. A full `--app` verify was therefore about
wasm 198 + build 18 + Playwright 552 = 13 minutes on the critical path, 17.7 minutes median wall including waits.

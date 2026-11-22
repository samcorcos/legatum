# Cloud box remote flakiness review

Ledger window: 2026-10-09 07:22 EDT through 2026-10-11 07:22 EDT (48 hours, local timestamps). Source: `~/LegatumRuns/timings/*.tsv` plus matching `~/LegatumRuns/box/<job>/output.raw` and `ssh.err`. 43 failed `box:remote` rows were present.

## Cause counts

| Primary class | Failed runs | Evidence / limit |
|---|---:|---|
| Remote command / test failure | 33 | Exit 1; verifier/test output or command error is present. These are not retried. |
| Daily-cap stop (strongly indicated; AWS stop event not retained) | 3 | Exit 143 with the container terminated after more than eight hours from the 20:00 EDT reset. Cap classification is an inference; the historical Lambda log was not retained here. |
| Box stopped: spot vs daily cap unresolved | 3 | Exit 143; retained output does not identify whether AWS stopped the persistent spot box or the cap did. |
| Git slot checkout / missing object | 4 | Exit 95; `git checkout` reports the requested commit is not a tree in the slot. No rsync error was recorded. |
| SSH connect/start race | 0 | No standalone failed run was attributable to connect timeout. One overlapping log has an overlong local ControlMaster socket bind error; shortened socket path addresses this. |
| rsync | 0 | No `rsync` error line in retained logs. |

The two `build-sim` exits with `LEGATUM_CLOUD_EXAMPLES: unbound variable` are included under remote command failures. The current `tools/cloud/box/build-sim.sh` already defaults that optional variable to empty; they do not indicate an SSH or box-start problem.

Daily-cap and spot-stop exits are now normalized to exit 3 (`BOX_UNAVAILABLE`) when the runner is terminated and the instance is no longer running. The box state does not prove whether an individual historical stop was the daily cap or spot interruption; ambiguous cases stay labeled as unresolved here. Missing-object slot checkouts now repeat the idempotent fetch/reset/checkout sequence up to three times; if the object remains absent, the job still fails visibly with exit 95.

## Failed `box:remote` runs

First error means the first retained failure line from the job output (or `ssh.err`). A missing line is stated explicitly.

| Local timestamp | Label | Exit | First error line | Class |
|---|---|---:|---|---|
| 2026-10-10 18:32:14 | `verify:astra/ui-batch-1010` | 143 | `Terminated` | Box stopped: spot vs daily cap unresolved |
| 2026-10-10 19:04:08 | `verify:astra/ui-batch-1010` | 1 | `VERIFY FAILED:app-vitest` | Remote command / test failure |
| 2026-10-10 19:09:42 | `build-sim` | 1 | `/data/bin/build-sim.sh: line 7: LEGATUM_CLOUD_EXAMPLES: unbound variable` | Remote command / test failure |
| 2026-10-10 19:09:54 | `build-sim` | 1 | `/data/bin/build-sim.sh: line 7: LEGATUM_CLOUD_EXAMPLES: unbound variable` | Remote command / test failure |
| 2026-10-10 19:44:13 | `verify:claude/bought-peace` | 1 | `VERIFY FAILED: merge` | Remote command / test failure |
| 2026-10-10 20:06:01 | `verify:claude/verify-speed` | 143 | `Terminated` | Box stopped: spot vs daily cap unresolved |
| 2026-10-10 20:54:21 | `verify-new:claude/verify-speed` | 1 | `VERIFY FAILED:sim-build-master smoke-master smoke-master-report` | Remote command / test failure |
| 2026-10-10 21:04:28 | `verify:claude/era-takeover` | 1 | `VERIFY FAILED:app-playwright` | Remote command / test failure |
| 2026-10-10 23:55:10 | `build-sim` | 95 | `fatal: reference is not a tree: c48b0b724c8953f6d608da871e4b2286517333eb` | Git slot checkout / missing object |
| 2026-10-11 00:02:45 | `verify:claude/era-units` | 1 | `VERIFY FAILED: merge` | Remote command / test failure |
| 2026-10-11 00:11:28 | `verify:claude/era-units` | 1 | `VERIFY FAILED: merge` | Remote command / test failure |
| 2026-10-11 00:18:53 | `verify:claude/era-units` | 1 | `VERIFY FAILED:clippy test app-playwright` | Remote command / test failure |
| 2026-10-11 00:28:18 | `dock-debug` | 1 | `Error: faction sheet for Celts scrolls at 1280×800` | Remote command / test failure |
| 2026-10-11 00:28:41 | `verify:claude/overreach-v3` | 1 | `VERIFY FAILED: merge` | Remote command / test failure |
| 2026-10-11 00:28:55 | `dock-debug-master` | 95 | `fatal: reference is not a tree: 4b3f0a872a98ac7be08df3b77553c031418a21ab` | Git slot checkout / missing object |
| 2026-10-11 00:41:35 | `dock-debug-branch` | 1 | `Error: no turn dock row is sliced` | Remote command / test failure |
| 2026-10-11 00:43:08 | `verify:claude/era-units` | 1 | `VERIFY FAILED: merge` | Remote command / test failure |
| 2026-10-11 00:57:41 | `verify:claude/era-units` | 1 | `VERIFY FAILED:app-playwright` | Remote command / test failure |
| 2026-10-11 01:04:27 | `pw-master-baseline` | 1 | `Error: expect(locator).toContainText(expected) failed` | Remote command / test failure |
| 2026-10-11 01:12:18 | `pw-branch-check` | 1 | `Error: research: button.research-salvage-camp "+8 Science 2 camps near Bukhara Epoc" default label: #7b796d on #7b796d` | Remote command / test failure |
| 2026-10-11 01:15:50 | `pw-research-contrast` | 1 | `Error: research: button.research-salvage-camp "+8 Science 2 camps near Bukhara Epoc" default label: #7b796d on #7b796d` | Remote command / test failure |
| 2026-10-11 01:21:22 | `pw-research-shot` | 1 | `Error: research: button.research-salvage-camp "+8 Science 2 camps near Bukhara Epoc" default label: #7b796d on #7b796d` | Remote command / test failure |
| 2026-10-11 01:26:04 | `pw-research-fix` | 1 | `Error: research: button.research-salvage-camp "+8 Science 2 camps near Bukhara Epoc" default label: #7b796d on #7b796d` | Remote command / test failure |
| 2026-10-11 01:29:53 | `pw-debug` | 1 | `[box 01:29:49] command exited 1 after 200s` | Remote command / test failure |
| 2026-10-11 01:33:29 | `pw-debug` | 1 | `[box 01:33:21] command exited 1 after 194s` | Remote command / test failure |
| 2026-10-11 01:37:22 | `pw-debug` | 1 | `[box 01:37:12] command exited 1 after 198s` | Remote command / test failure |
| 2026-10-11 01:40:30 | `pw-debug` | 1 | `[box 01:40:19] command exited 1 after 164s` | Remote command / test failure |
| 2026-10-11 01:44:37 | `pw-research-fix2` | 1 | `Error: research: button.research-salvage-camp "+8 Science 2 camps near Bukhara Epoc" default label: #7b796d on #7b796d` | Remote command / test failure |
| 2026-10-11 01:49:47 | `build-sim` | 95 | `fatal: reference is not a tree: ed186db5d7d230d86d630dc50f243d84e43b0ef9` | Git slot checkout / missing object |
| 2026-10-11 01:56:24 | `verify:claude/era-units` | 1 | `VERIFY FAILED:app-playwright` | Remote command / test failure |
| 2026-10-11 02:05:40 | `verify:claude/era-units` | 1 | `VERIFY FAILED:app-playwright` | Remote command / test failure |
| 2026-10-11 02:12:58 | `verify:claude/bought-peace` | 1 | `VERIFY FAILED:app-playwright` | Remote command / test failure |
| 2026-10-11 02:18:25 | `fix-e2e-baseline` | 1 | `Error: expect(locator).toContainText(expected) failed` | Remote command / test failure |
| 2026-10-11 02:21:08 | `verify:master` | 1 | `VERIFY FAILED:app-playwright` | Remote command / test failure |
| 2026-10-11 02:35:44 | `build-sim` | 95 | `fatal: reference is not a tree: 8da2b2ac58a1cde490d9d8529b256e464c963682` | Git slot checkout / missing object |
| 2026-10-11 02:55:23 | `verify:claude/era-units` | 1 | `VERIFY FAILED:app-build app-tsc app-vitest` | Remote command / test failure |
| 2026-10-11 03:35:54 | `verify:astra/ui-batch-1011a` | 1 | `VERIFY FAILED:app-playwright` | Remote command / test failure |
| 2026-10-11 03:36:47 | `verify:astra/map-art-legibility-polish` | 1 | `VERIFY FAILED: merge` | Remote command / test failure |
| 2026-10-11 04:32:14 | `verify:astra/sakoku-show-intruder` | 1 | `VERIFY FAILED:app-playwright` | Remote command / test failure |
| 2026-10-11 04:56:58 | `build-sim` | 143 | `Terminated` | Daily-cap stop (strongly indicated; AWS stop event not retained) |
| 2026-10-11 05:12:08 | `verify:claude/release-8.8.1` | 143 | `Terminated` | Daily-cap stop (strongly indicated; AWS stop event not retained) |
| 2026-10-11 05:12:14 | `build-sim` | 143 | `Terminated` | Daily-cap stop (strongly indicated; AWS stop event not retained) |
| 2026-10-11 07:02:08 | `build-sim` | 143 | `[no matching retained job log]` | Box stopped: spot vs daily cap unresolved |

## `box-verify:fmt`

There were 3 failing timing rows across 2 refs. All have `wall_s=0`; matching refs also have zero-duration pass rows from `tools/astra/test-verify-cache.sh` runs that set `VERIFY_STEP_STUB=1` and deliberately inject a failed `fmt`. That is verifier-harness instrumentation, not a stale rustfmt toolchain. `tools/ops/timed.py` now suppresses ledger writes under that stub flag; no rustfmt/toolchain changes were made.

| Local timestamp | Ref | Exit | Wall seconds |
|---|---|---:|---:|
| 2026-10-10 20:06:38 | `e31138203dc6` | 1 | 0 |
| 2026-10-10 20:48:40 | `d26db371a24a` | 1 | 0 |
| 2026-10-10 20:56:56 | `d26db371a24a` | 1 | 0 |

## Fixes and stub reproductions

- SSH startup now uses a short per-user ControlMaster socket path and a bounded retry loop with a 10-second connect timeout. `tools/cloud/test-box-retries.sh` uses a stub `ssh` that fails twice then succeeds, checks that retry exhaustion stops at three attempts, and checks unavailable exit mapping while preserving ordinary test failures.
- Slot ref setup retries only the idempotent fetch/reset/checkout sequence. `tools/cloud/test-box-slot-checkout.sh` uses a local bare repository and a stub `git` that fails the first fetch; the next fetch checks out the requested commit.
- The idempotent data-volume mount retries at most three times. Non-idempotent job submission and test commands are not retried.
- When a job is terminated with the box stopped, `remote.sh` reports exit 3 unavailable; a normal test exit remains unchanged. `tools/ops/test_timings_report.py` verifies exit 3 does not count as a flaky test result.
- Stubbed verifier steps no longer write timing rows. `tools/ops/test_timings_report.py` verifies a deliberately failed stubbed `fmt` record leaves the ledger empty.

Run the stubs with:

```sh
bash tools/cloud/test-box-retries.sh
bash tools/cloud/test-box-slot-checkout.sh
python3 tools/ops/test_timings_report.py
```

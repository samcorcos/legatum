# B13: visibility performance pass

The scoped change reduces observation time by **66.2%**, `resolve_turn` time by **24.4%**, and total view/AI/resolution time by **5.5%** in the shorter alternating experiment. These are geometric means of the before/after median ratios for 18 fixed snapshots. Visibility masks take 78.7% less time; all-seat views take 2.5% less time. AI decisions are approximately unchanged (+0.4% overall, with substantial variation). All recorded hashes, complete serialized states and views, and committed replay tapes match.

The large visibility improvement repeats in both measurement series. The smaller end-to-end gain is less certain: the earlier, longer series showed a 4.3% total slowdown alongside 8.6% slower unchanged AI code and 4.1% slower cloning. Shortening the alternation interval produced aggregate total improvements in all six pairs and lower pooled median total time in all 18 snapshots. Both series, every retained sample, per-snapshot distributions, and the remaining AI variability are reported. This does not establish how much of the historical **7.4s → 10.6s** difference was caused by fog memory; those reports used different source revisions and outcomes.

The [measurement report](../sim-results/2026-09-26-b13-astra-3519f6d-performance-pass.md) contains the tables and exact check results. Its [JSON companion](../sim-results/2026-09-26-b13-astra-3519f6d-performance-pass.json) contains raw batch durations, sample counts, P10/median/P90/P95/min/max/mean, source and corpus checksums, and command exit receipts. Generated binaries, corpus snapshots, CSV measurements, build output, logs, and handoff files remain under this worktree's `target/performance-pass/` (the requested final WASM package is `target/performance-wasm-node/`).

## Scope and source

Baseline source: **`3519f6d983fd3af34e5bac9bd1cb48e34fbb8dab`**, branch **`astra/performance-pass`**, worktree `.claude/worktrees/astra-performance-pass`. The worktree was clean before the new harness and tests were added. No upstream changes were pulled during the experiment. The optimized source is that SHA plus the two production-file changes in this patch; the JSON report records their complete file hashes and the production diff SHA-256. Git metadata permissions prevented a new commit, so the optimized work is not represented as a committed SHA.

Only `crates/core/src/visibility.rs` and `crates/core/src/bitset.rs` change production code:

- `visible_tiles` walks the same axial `dq`/`dr` ranges directly instead of allocating and sorting `Hex::within`'s temporary vector for every unit/city. The original order is already ascending `(q,r)`. Source order, clipping, radii, union results, and integer arithmetic stay the same.
- `BitSet::iter_ones` skips zero bits with `trailing_zeros` and clearing the lowest set bit. Indices remain in ascending map order; padding beyond logical length is excluded. The bitset's fields, serialization, existing operations, and hash representation do not change.
- `update_explored` visits those indices instead of scanning every map tile for every player. It still resizes old memory, writes every currently visible observation, preserves every unseen observation, and rebuilds city observations on each call. There is no cached state, changed observation frequency, RNG draw, or rule change.

The baseline direct measurements established that `update_explored` and `visible_tiles` were material costs before these edits: pooled early/middle/late medians were about 17/21/35 µs per observation and 5/9/20 µs for the all-team visibility pass. Observation is repeated during movement and at the end of resolution, so one update's time is not the entire turn's fog cost.

The new files are the AI measurement example, shell driver, differential integration test, and these reports. Existing tests, manifests/lockfiles, `view.rs`, `hex.rs`, rules, balance data/economy, AI policy, fixtures, plans, shared learnings, the app, and other worktrees were not edited. The newly added iterator test lives in `bitset.rs`'s test module so the final integration-test file can also compile against the original baseline. Moving that test after profiling produced a **byte-identical release profiler executable**, recorded in the JSON report.

## Fixed inputs and timing method

The required planning/specification files, visibility/bitset/view code, AI public API, fixture README, and both historical simulation reports were read. The planning documents disagree about map size: the live runner's `GameSetup::all_ai` and the first-playable `GameSetup::standard` both use **48×32**, the engineering spec also says 48×32, and the PRD still says **72×44**. The experiment covers all of them explicitly:

| Case | Dimensions | Layout | Seeds | Early / middle / late turns |
| --- | --- | --- | --- | --- |
| sim48 | 48×32, 1,536 tiles | Eight separate teams | 1, 7 | 5 / 50 / 140 or 141 |
| team48 | 48×32, 1,536 tiles | Seats 0/1 share vision; six other teams | 1, 7 | 5 / 50 / 123 or 132 |
| prd72 | 72×44, 3,168 tiles | Eight separate teams | 1, 7 | 5 / 50 / 139 or 108 |

There are **six distinct games**, **783 recorded turns**, **18 measured snapshots**, and **789 complete JSON snapshots** including each initial and final state. Every game has turn limit 150 and all seats use the existing `ScriptedBrain`, including the shared-vision layout. The harness calls `ScriptedBrain::new(faction, game_seed, player)` directly. The production `Session` adds a per-seat seed derivation before that call; therefore these are deliberately fixed public-API workloads, not reproductions of the runner's historical seed-1 campaign. No policy or knob code changes occur between binaries; all AI source checksums are verified against the baseline.

Before production edits, `scripts/perf-profile.sh record` wrote exclusive-create order tapes, every state hash, and full debug snapshots of the measured states/views/orders. The optional core test wrote full canonical JSON state-and-all-seat-view snapshots using core's existing `serde_json` dependency. This keeps the AI example within its existing dependencies and avoids manifest changes. A small, versioned numeric order codec is independently decoded by the JSON test; each replayed hash must match before a JSON snapshot is recorded or accepted.

Every benchmark launch first replays the **recorded** orders. It verifies every state hash, compares newly generated AI decisions to those orders, and compares the complete measured snapshots. It aborts on a changed policy/setup/output. The cross-build core check additionally compares every canonical JSON snapshot **byte for byte**, not just by a view hash. Setup generation, snapshot construction/checking, AI equivalence assertions, JSON, hashing, and disk I/O occur outside the timers.

Each measured operation uses the same snapshot input:

| Metric | Timed work |
| --- | --- |
| resolve | One `resolve_turn` on recorded orders, including its normal internal clones and returned-state/event drop |
| views | `player_view` for every seat, including eliminated seats, and returned-view drops |
| ai | All live-seat decisions on prebuilt views, including the normal stateless brain construction/order collection and drop |
| total | All-seat views, live-seat decisions, then resolution; retains normal allocation/drop work |
| observe | One `update_explored` on a fresh independent scratch state; scratch clone/drop excluded |
| visible | `visible_tiles` once per team, including team enumeration and result drop |
| clone | One full `GameState::clone` and drop |
| titles | One `scoring::title_values` call and drop |

The profiler uses `Instant` and `black_box`. It has one execution thread. `RAYON_NUM_THREADS=1`, `RUST_TEST_THREADS=1`, and `CARGO_BUILD_JOBS=4` are fixed by the driver; no project build/test runs overlapped the timed series. Warm-up is five samples per snapshot/metric. Metric order rotates each sample. Timings are batch wall times divided by the batch size, so distribution tails describe batch averages, not individual-call latency.

The first pre-edit baseline used 31 samples with 5 operations/sample. Then the saved baseline and optimized binaries ran in order **B, O, O, B, B, O**: three process runs per binary, 93 retained samples and 465 operations per snapshot/metric/binary. Because unchanged components also moved substantially, a second experiment shortened each process run to 11 samples of 3 operations, alternating **B,O / O,B** for six pairs. That gives 66 retained samples and 198 operations per snapshot/metric/binary. Both comparisons use identical settings on their two sides; no samples were trimmed. There are no timing assertions in tests.

The machine was not exclusively reserved. CPU affinity and thermal/frequency state were not controlled; process sampling and some `sysctl` queries were denied by the sandbox. Run-pair timing variation is observable, but attribution to a particular concurrent workload is unavailable. Correlated batch samples do not justify a tight statistical confidence interval. The observation reduction is large and consistent; the exact total gain and small changes to unchanged code should be rechecked on the lead's machine if they matter to a release decision. The worst short-series AI snapshot is +7.7%, with its repeated run comparisons changing sign; overall AI is +0.4%. There is no repeated aggregate total-work regression in the shorter series.

## Toolchain, hardware, and commands

Hardware: **a 16-core Apple-silicon laptop (12 performance + 4 efficiency cores)**, aarch64, on a current macOS. Tools:

- `rustc 1.98.1 (48a229cea 2026-09-01)`, full compiler commit `48a229ceaefd4985c50990b14116b6d856af0985`, LLVM 22.1.8, host `aarch64-apple-darwin`.
- `cargo 1.98.1 (797e8a9bc 2026-08-05)`; native `cargo build --release -p legatum-ai --example perf_profile`; existing release profile, `debug=false`, no RUSTFLAGS or release-profile environment overrides.
- `wasm-pack 0.15.0`, `wasm-bindgen 0.2.129`, `wasm-opt 117`, target `wasm32-unknown-unknown`; release build and wasm-pack's default `wasm-opt -O`, unchanged on both sides.
- `node v26.10.0`; size compression with Python 3.14.7, zlib 1.2.12, `gzip.compress(bytes, compresslevel=9, mtime=0)`.

To reproduce, start at the baseline SHA with the **same new example, shell script, and integration-test file**, while leaving the two production files at the baseline. Keep the corpus and binaries under that worktree's `target/`. The final integration file has no dependency on the newly added bitset method, so it works on the baseline too. Record first, then apply only this patch's two production-file changes; keep AI/rules/data/manifests pinned throughout.

```sh
export CARGO_TARGET_DIR="$PWD/target"
export CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1
bash scripts/perf-profile.sh record
bash scripts/perf-profile.sh measure baseline-initial target/performance-pass/baseline-perf_profile

# After applying only the visibility/bitset changes:
bash scripts/perf-profile.sh verify-json
cargo build --release -p legatum-ai --example perf_profile
cp target/release/examples/perf_profile target/performance-pass/optimized-perf_profile

# Long series, in this exact order:
bash scripts/perf-profile.sh measure baseline-paired-1 target/performance-pass/baseline-perf_profile
bash scripts/perf-profile.sh measure optimized-paired-1 target/performance-pass/optimized-perf_profile
bash scripts/perf-profile.sh measure optimized-paired-2 target/performance-pass/optimized-perf_profile
bash scripts/perf-profile.sh measure baseline-paired-2 target/performance-pass/baseline-perf_profile
bash scripts/perf-profile.sh measure baseline-paired-3 target/performance-pass/baseline-perf_profile
bash scripts/perf-profile.sh measure optimized-paired-3 target/performance-pass/optimized-perf_profile

# Short series: run odd pairs baseline then optimized, even pairs in reverse.
for n in 1 2 3 4 5 6; do
  if (( n % 2 )); then
    variants=(baseline optimized)
  else
    variants=(optimized baseline)
  fi
  for variant in "${variants[@]}"; do
    PERF_SAMPLES=11 PERF_BATCH=3 bash scripts/perf-profile.sh measure \
      "$variant-short-$n" "target/performance-pass/$variant-perf_profile"
  done
done
```

The driver refuses to overwrite the original corpus or an existing sample label. Defaults are 31 samples, 5 warm-ups, 5 operations/sample. Raw CSV schema is `case,stage,turn,tiles,units,cities,metric,sample,batch,ns`. Percentiles use ascending sort and index `floor((n-1)*p)`; the median averages the middle pair when n is even. The summary geometric mean weights each snapshot once, avoiding domination by the larger map. The JSON report also preserves each run's raw integer batch durations so other summaries can be recomputed.

For each WASM build, the default cache initially caused an `Operation not permitted` failure. Reuse the already installed matching tools with a writable cache; no tool installation, optimization flag change, or manifest edit was needed:

```sh
export WASM_PACK_CACHE="$PWD/target/performance-pass/wasm-pack-cache"
export PATH="$HOME/Library/Caches/.wasm-pack/wasm-bindgen-cargo-install-0.2.129:$HOME/Library/Caches/.wasm-pack/wasm-opt-50385c9e73ccee70/bin:$PATH"
wasm-pack build crates/wasm --target nodejs --out-dir ../../target/performance-wasm-node
node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/performance-wasm-node
```

The compared WASM files are the **post-wasm-bindgen, post-wasm-opt** `legatum_wasm_bg.wasm` modules. Baseline: **1,045,161 raw / 367,128 gzip bytes**. Optimized: **1,045,220 raw / 367,238 gzip bytes**. The changes are **+59 raw bytes (+0.006%)** and **+110 gzip bytes (+0.030%)**. This pass improves the measured native hot path; it does not claim a bundle-size reduction. These sizes exclude JS glue, app assets, and debug/custom-profile builds.

## Equivalence and checks

The integration oracle preserves the original `visible_tiles` and `update_explored` algorithms, including their full map scan and `Hex::within`. It compares complete `GameState` equality, state hashes, serialized state, each `PlayerView`, and serialized views, then repeats the update to check idempotence. The deterministic generator uses core's existing RNG and valid bounded orders: six games, 24 turns each, both map sizes, separate/shared teams, and a seed at `u64::MAX`. Hand-built traces cover unseen founding, capture and city-field changes, remembered razed cities, removal/regaining of teammate-provided sight, changed ruin/ownership, no remaining sources, and missing/empty/short/oversized tile memory. Razing is modeled by removing the city, as in the existing regression, since there is no public raze order. The bitset unit test covers empty/sparse/dense sets, word boundaries and dirty serialized padding without changing representation. No extreme-Hex fix or test weakening is included.

The optional corpus integration test is ignored in normal test discovery because it requires the pre-edit artifacts; it was explicitly executed in record mode before production edits and verify mode afterward. All **789 state + all-seat JSON snapshots** matched exactly, as did **783 per-turn hashes and recorded AI-order maps**. Normal differential tests run without these artifacts.

The following required commands were executed in this worktree. The report and `target/performance-pass/*.exit.json` record their exit codes and logs:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p legatum-core --test visibility_equivalence
cargo test -p legatum-sim --test determinism
cargo test --release -p legatum-sim --test determinism
wasm-pack build crates/wasm --target nodejs --out-dir ../../target/performance-wasm-node
node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/performance-wasm-node

# Supplemental coverage because the full workspace command stops at server protocol failures:
cargo test --workspace --exclude legatum-server
bash scripts/perf-profile.sh verify-json
```

Formatting, strict all-target Clippy, visibility equivalence, both native determinism suites (6 tests each), the optimized WASM build, all three Node tapes, the JSON comparison, and workspace tests excluding the server **exit 0**. The full workspace command **exits 101** both before and after the optimization: all five `crates/server/tests/protocol.rs` tests fail at line 27 because the sandbox prohibits binding the listener (`Os { code: 1, kind: PermissionDenied }`). The server's six library tests pass. This is a baseline environmental block, not a resolved or ignored green check. Fixture file SHA-256s remain exactly unchanged; no fixture was regenerated. The initial harness had two Clippy errors which were corrected before timing; those were introduced during harness development, not baseline repository failures.

Git staging **exits 128** because `.git/worktrees/astra-performance-pass/index.lock` cannot be created. The commit is blocked and was not attempted after that denial. No permissions were bypassed, and nothing was merged. The exact patch is `target/performance-pass/b13-performance-pass.patch`, based on the SHA above; `target/performance-pass/commit-message.txt` has the requested co-author trailer. The lead can stage and commit the preserved changes from an environment allowed to write the Git metadata, or apply that patch to the same source revision. `target/performance-pass/handoff.txt` lists the files, patch checksum, and exact commands.

## Read-only follow-up findings

AI decisions and all-seat views now dominate these representative turns. For the larger 72×44 middle-game snapshots, AI decisions alone are roughly 1.4–1.5 ms on the baseline, compared with roughly 0.36–0.41 ms for views. This API-level profile does not identify an internal AI heuristic as the culprit; ai-v2 owns policy and should profile its own fixed-view inputs before changing it.

`player_view` computes the complete public `scoring::title_values` table for each seat. The baseline late-game pooled medians are about **46 µs for one title-table call** and **650 µs for all eight views**. Title computation itself revisits city yields and scans territory. Measuring an immutable, per-call public-summary reuse path is a useful follow-up for the view/economy owners. Eight times the standalone title timer is only an estimate, not an attributed share of the view timer; allocation and cache behavior differ.

Cloning costs roughly 30–70 µs in these snapshots and remains part of resolution. Hash/JSON costs and the runner's second determinism replay are deliberately excluded from the operation timers, so these totals should not be multiplied into a prediction of the historical 16-thread 200-game wall times. Only six distinct fixed games were generated; no balance campaign was run. Later balance/rule changes can intentionally change hashes and require their owner's reviewed fixture updates; they should not be mixed into this optimization's equivalence comparison.

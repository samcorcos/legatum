# Playtest build profile: thin LTO against the shipped fat LTO, 2026-11-15

Question: the release app build is slow because of whole-program optimisation (fat LTO, one codegen
unit). Is a cheaper profile worth using for playtest builds, and what does it cost? Measured on the
designer's 16-core Apple M3 Max laptop from the `claude/calendar` tree (playtest-8.9.41 plus later commits).

## Result

Times are wall-clock seconds. Each build used 7 cargo jobs (what `tools/ops/free-threads.sh` printed), sccache
off (`RUSTC_WRAPPER=`), and its own target directory. The machine was not idle: load average was 15-20 from other
agents throughout, so absolute numbers are somewhat inflated and noisy. "Change" is what a developer
sees after a one-line comment edit in `crates/ai/src/lib.rs`, rebuilding the WASM engine and the Tauri app (both
steps include `npm run build` and bundling). "From clean" is an empty target directory for both steps.

| | Current (fat LTO, 1 unit) | Thin (thin LTO, 16 units) | No LTO (16 units) |
| --- | ---: | ---: | ---: |
| **Rebuild after a one-line `crates/ai` change** | **516 s (8.6 min)** | **205 s (3.4 min)** | **171 s (2.9 min)** |
| of which WASM / app | 216 / 300 | 98 / 107 | 63 / 108 |
| From clean, WASM + app | 595 s (9.9 min) | 239 s (4.0 min) | 195 s (3.3 min) |
| of which WASM / app | 215 / 380 | 81 / 158 | 71 / 124 |
| App binary (`legatum-desktop`) | 132,175,616 B | 137,112,576 B (+3.7%) | 139,718,992 B (+5.7%) |
| WASM (`legatum_bg.wasm`, after wasm-opt) | 9,081,229 B | 9,588,105 B (+5.6%) | 8,574,422 B (-5.6%) |
| Native AI speed, 32 Hard games, median of 3 | 9.59 s | 10.75 s (+12%) | 16.81 s (+75%) |
| WASM AI turn, mean ms for turns 101-177, median of 3 | 30.9 | 33.4 (+8%) | 46.4 (+50%) |
| `wasm-opt -O3` alone on the unoptimised module | 42.5 s | 37.3 s | 30.2 s |
| Unoptimised WASM size (skipping `wasm-opt`) | 11,674,319 B | 14,204,085 B | 16,385,980 B |

Engine output is identical in all three: the 32 order tapes (every per-turn state hash) are byte-for-byte equal
across the three native builds and across repeats, and the WASM replay's per-turn view hash matched on all 177
turns for every variant and repeat.

The 13 to 25 minute figure was not reproduced: on this machine the shipped profile took 8.6 to 9.9 minutes.
The longer times are presumably heavier load (a build competing with other agents' builds and sims) or a
slower bundle step; this run does not say which.

## What the opt-in is

`LEGATUM_PLAYTEST_PROFILE=1` switches to the Thin column:

- `scripts/build-wasm.sh` builds with the new `wasm-playtest` profile (root `Cargo.toml`: inherits `wasm-release`,
  `lto = "thin"`, `codegen-units = 16`); `wasm-opt -O3` still runs.
- `scripts/build-mac-app.sh` exports `CARGO_PROFILE_RELEASE_LTO=thin` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`,
  which cargo applies over the Tauri crate's `[profile.release]` (opt-level `s`, with `legatum-ai` and
  `legatum-core` at 3, are unchanged). The bundle path stays `app/src-tauri/target/release/bundle/macos/Legatum.app`.
  For the explicit recipe (`scripts/build-wasm.sh`, then `npm run tauri:build -- --bundles app`), export the same
  two variables plus `LEGATUM_PLAYTEST_PROFILE=1` before both steps.
- With the variable unset nothing changes: the shipped profiles are untouched.

Proof the opt-in path builds a bundle: `LEGATUM_PLAYTEST_PROFILE=1 scripts/build-mac-app.sh` from an empty target
directory finished in 266 s with exit 0 and produced `Legatum.app` with a Mach-O arm64 `legatum-desktop`
(137,112,576 B, ad-hoc signature as read by `codesign -dv`) and a 9,588,105 B WASM, the same sizes as the Thin
measurement. It was not launched.

## How each number was measured

- Build times: a script ran each variant twice in a row (clean, then after appending a comment line to
  `crates/ai/src/lib.rs`), each step timed with `date +%s`: `scripts/build-wasm.sh` (including its timing wrapper),
  then `npm run tauri:build -- --bundles app` (tsc, vite, cargo, bundle). Variants were switched with cargo's
  `CARGO_PROFILE_*` environment overrides, which are the same settings as editing the profile tables. One run each:
  there is no repeat of the build timings, so treat differences under about 10% as noise.
- App binary and WASM sizes: `ls -l` of the built files.
- Native AI speed: `legatum-sim` built with the same lto and codegen settings per variant
  (`CARGO_PROFILE_RELEASE_*`; the sim uses opt-level 3 throughout, so it is a proxy for the app's native AI), then
  `legatum-sim --games 32 --seed 7 --turn-limit 120 --difficulty hard --threads 4`, three alternating runs, median
  wall-clock. (The task's `--brain hard` does not exist; Hard is `--difficulty hard`, and `--brain` is
  scripted/random/mixed.) An 8-game version gave the same ordering: 2.35 s, 2.62 s, 3.90 s. Sim builds from clean:
  361 s fat, 100 s thin, 84 s no LTO.
- WASM AI speed: `scripts/perf-wasm-turns.mjs` (copied with one extra output column, a sha1 of each turn's view),
  replaying 7000001's game (the first 200 requested turns; it ended at turn 177) through each variant's
  `wasm-release`-style build in Node 23.6, three runs each. Plain Hard whole lobby is about 31 ms p95 at turn 250
  in the October budget, so +8% is a few milliseconds a turn.
  The medians above are noisy because of the machine load: the fastest single runs were 28.2, 29.0 and 41.5 ms.
- `wasm-opt`: the cached `wasm-opt -O3 --all-features` run directly on the module `wasm-pack --no-opt` produces for
  each variant, on a loaded machine.

## Not measured

- AI turn time inside the running desktop app (it was not launched); the Node replay and the native sim stand in.
- The speed of an unoptimised (`LEGATUM_WASM_NO_OPT=1`) WASM; only its size and the saved time.
- Repeats of the build timings, and a build after a change in `crates/core` (more crates rebuild; the fat profile
  suffers more from that than thin, so the gap should not shrink).
- Whether a human playtester feels any difference; at 31 ms for a whole lobby it is not expected to be visible.

## Recommendation

Use the Thin profile for playtest builds. A fix build drops from about 8.6 to 3.4 minutes (roughly 5 minutes
saved per build, 60%), and from clean from 9.9 to 4.0. The price is 8 to 12% slower AI turns (a few
milliseconds on a turn that costs about 30), a 5 MB larger app (+3.7%) and 0.5 MB more WASM, with
identical game results. No LTO saves only another 30 seconds per build and makes AI turns 50 to 75% slower, so it
is not worth it. Keep `wasm-opt`: skipping it saves 30 to 40 seconds but adds 2.6 MB of WASM on the current profile and 4.6 MB (+48%) on thin, and its AI speed is unmeasured. Keep the fat profile for any build that goes to public players or
for AI speed measurements. The designer left it to the lead (2026-11-15: "i trust your judgment. i just dont like waiting so long before i can do play tests"): playtest builds use the Thin profile from 8.9.44; the shipped profile is unchanged.

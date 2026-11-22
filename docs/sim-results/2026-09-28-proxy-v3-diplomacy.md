# Proxy v3 diplomacy calibration (2026-09-28)

`PROXY_VERSION = 3`, frozen policy SHA-256
`2a49bf8bc98d01035382731e43b6e35b29b6723f8a808f5c3b4d6b3792498233`.
Engine and opponent baseline: `ad2e9c546bbc54c1e2c3ef75bf17178c7c775956`.
The new policy, version selection, observers and tests were uncommitted when measured; the
source manifest below pins all Rust sources and Cargo files used by the final binary.
No core rules or scripted-opponent decisions changed.

The requested Peace calibration target is **met** on this 300-game sample:
at least 70% wins against seven Hard AIs and median winning turn about 200–220.

| Start | Proxy wins | Median winning turn | Captures/game | Full deterministic replays |
| --- | ---: | ---: | ---: | --- |
| Peace | 225/300 (75.0%) | 211 | 6.18 | 300/300; 0 failures |
| Legacy War | 180/300 (60.0%) | 208.5 | 5.50 | 300/300; 0 failures |

Both simulation commands exited **0**. Both runs have **0 panics and 0 stalls**. Each report
contains 300 first plays; replayed games do not increase metric denominators. Captures include
recaptures, and winning-turn medians use wins only. All 600 games ended by the engine's VP condition. These are calibration seeds used during development;
results depend on the seed distribution and engine revision.

A separate validation after freezing the policy used seeds 910001–910064, covering all 64
seat/faction pairs: **46/64 (71.9%)**, median winning turn **209**, 5.41 captures/game. All 64
full deterministic replays passed; there were no panics or stalls. These seeds were not used to
choose or change this policy. The validation command exited 0. Its receipt, report and tapes are
in `proxy-v3-peace-heldout`; report SHA-256
`f162345d104d77d33833f2500aa393eb3f31c184c79d54c6d7a7e6c4f131ee7d`.

V3 delegates directly to frozen v2 under legacy all-war rules. Every turn hash in the final
300-game War run matches the earlier War reference, which also delegated to frozen v2.
The reference is retained in `final-war/2026-09-28-final-war.json`. A full game and replay
using the final binary with `--proxy-version 2` and no start flag also matches seed 700001's
legacy hashes, confirming the v2 default remains War. V3 defaults to Peace. Both versions
also accept either explicit `--proxy-start` value.

The current-AI [early-defense report](2026-09-28-early-defense-economy.md) records v2 at
60.0% in War and 16.7% in Peace on the same 300-seed protocol. The older 83% War result in the
[v2 calibration](2026-09-28-proxy-v2-calibration.md) predates the garrison and early-defense
changes. The new War result reproduces the current 60% baseline.

## Policy and calibration

The policy takes only `PlayerView`, has no RNG or persistent private state, and never calls the
opponent's diplomacy planner. It ranks known reachable neighbors by public army strength and
visible defense, requires healthy capture infantry plus ranged support and favorable public
combat forecasts, and budgets reserves against all visible unprotected flanks. Pending pact
offers do not count as protection. Treaty quotes, guarantees, cooldowns and notice periods are
respected; it never deliberately breaches a pact.

It assembles a campaign outside peaceful borders, requests or accepts peace when defenses
are overwhelmed or a war has no useful reachable front, and accepts useful flank pacts.
It reserves one conquest route and can give lawful notice to reopen a fully protected route
when a favorable campaign is ready. Public notice history keeps that campaign focused.
Declarations take effect at the engine boundary; projected permissions never authorize a
premature attack. Units caught in closed territory follow known legal exit steps.

Peace also changes the opening's resource needs. After two cities, one city's extra settler
build can reinforce toward two infantry and three ranged units. A nearly empty treasury with
nonpositive income triggers Exchange at the next research choice and Markets in place of unit
builds, while existing infrastructure and the immediate home-defense check take precedence.
This prevents repeatedly producing troops that disband for unpaid upkeep.

In the matched 64-game development sample, lawful route-opening reached 43 wins with median
turn 211. Adding upkeep recovery reached 48 wins (75%) with median 212.5; upkeep disbands fell
from 121 to 35. Seven previous losses became wins and two previous wins became losses.
Seed 700025 changed from a one-city bankruptcy loss with no captures to a turn-237 win with
eight captures. The notice-only full sample was 200/300 (66.7%), median 211. A wider neighbor
radius performed worse (39/64, median 222) and was rejected. The final 300-game results above
are the acceptance measurements; the small screens are development evidence.

Resolved Peace-run events involving the proxy: 1502 wars declared,
1137 wars received, 1851 peace agreements,
1373 pacts activated and 195 notices given.
These counts come from engine events, not attempted orders. Public army-title totals can
include unseen forces but reveal no hidden location or composition. Future moves, unseen
attacks and future pact breaches remain uncertain.

Snapshots are conditional on reaching the stated turn. City counts include captures; the
technology gap compares the proxy with the strongest original AI seat, including retained
knowledge of eliminated opponents. The broader guide's early expansion and technology goals
remain unmet: Peace has median 3 cities at turn 100 and mean
tech-count gap -0.94 at turn 150. Normal difficulty was not recalibrated here.

| Start | Turn | Proxy cities: mean / median | Mean tech gap vs strongest AI | Games reaching turn |
| --- | ---: | ---: | ---: | ---: |
| Peace | 100 | 3.28 / 3 | -0.96 | 300 |
| Peace | 150 | 5.88 / 6 | -0.94 | 285 |
| Peace | 200 | 7.11 / 7 | -1.86 | 210 |
| Legacy War | 100 | 4.00 / 4 | -0.40 | 299 |
| Legacy War | 150 | 6.10 / 6 | -0.94 | 293 |
| Legacy War | 200 | 6.72 / 6 | -1.35 | 237 |

## Reproduction and artifacts

The protocol uses Standard speed, the default 48×32 map, eight separate teams, seven Hard AIs,
seeds 700001–700300, and balanced seat/faction rotation. Each 64-game block covers all 64
seat/faction pairs. The unmodified Standard safety cap is 1,200 turns; a cap stop counts as a
stall, never a win. All tapes were also hash-verified by the analysis observer.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
for proxy_start in peace war; do
  ./target/release/legatum-sim --proxy 0 --proxy-version 3 \
    --proxy-start "$proxy_start" --rotate-proxy --difficulty hard \
    --games 300 --seed 700001 --speed standard --threads 8 --determinism \
    --archive --archive-dir "/tmp/proxy-v3-reproduction/$proxy_start/archive" \
    --out-dir "/tmp/proxy-v3-reproduction/$proxy_start" --label "proxy-v3-$proxy_start"
done
```

Large artifacts are outside Git under `/tmp/legatum-proxy-v3`. Each run folder contains
`receipt.json` with the exact command, binary/policy hashes, exit code and elapsed time,
`run.log`, `status.json`, and the archived tapes. Final checks are in `checks-final`;
`final-verification.json` records source verification and legacy hash equivalence.
`v2-default-final` contains the final-binary version-2 compatibility probe. Earlier trial
folders are development evidence and are not substituted for the final runs.

| Artifact relative to that directory | SHA-256 |
| --- | --- |
| `proxy-v3-peace/2026-09-28-proxy-v3-peace.json` | `e0646933661b3cddc4c71f687c113721570056f7709d10a4ff4da4ed3125d1c4` |
| `proxy-v3-war/2026-09-28-proxy-v3-war.json` | `fcfc3764985e72fce4dd0a7b5d421e0af7cbd03f81c40ea4518642335972f818` |
| `candidate-15-main/source-manifest.json` | `e0ae2dbf2e0260c3eb9b1920dabb16744d7b0e40dfdcfd2d559371c0297232df` |
| `bin/candidate-15-main` | `993283ec075097bba2744ebc6dc20ba7c0b7a2440d2a970c3594f16a7a587b01` |

## Checks

Final frozen source, `CARGO_BUILD_JOBS=4`, `RUST_TEST_THREADS=4`:

| Command | Exit code |
| --- | ---: |
| `cargo fmt --all` | 0 |
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo clippy --workspace --all-targets --release -- -D warnings` | 0 |
| `cargo test --workspace --release --no-fail-fast` | 0 |
| `cargo test --workspace` | 0 |
| `cargo build --release -p legatum-sim` | 0 |
| `git diff --check` | 0 |

Workspace test totals: release: 851 passed, 4 ignored; debug: 851 passed, 4 ignored. The four
ignored tests are existing opt-in performance, paired-evaluation, multi-game and
cross-build-corpus checks.

The focused diplomacy suite has 22 tests, including fog invariance, save/load repeatability,
weakest-target selection, favorable forecasts, real declaration/peace boundaries, visible
coalitions, pending versus signed pacts, treaty cooldowns, staging/reserves, lawful notice,
closed-border egress, legacy equivalence, reinforcement replacement and upkeep recovery.
The CLI tests cover both versions crossed with both start modes and their recorded metadata.
No conflict markers were found. `CLAUDE.md` and `docs/learnings.md` are unchanged.

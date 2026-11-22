# Endgame planning corrections: paired Quick comparison

Building-title sequences now reserve core's empire construction slots across the
first building, every later step, existing buildings and effective queues,
including orders already issued this turn. Core data supplies both the limit and
the competing building kinds: one Reactor, or one shared Great Work construction
slot. Core validation and Reactor capture enforcement consume the same data.
There are no world-wide building limits in this revision. Great Work completion
remains a permanent player flag; owning a captured Work is distinct from having
completed one. Resistance does not free an existing Reactor slot, and teammates
have independent slots.

Capture estimates now add `max(city.population - 1, 1)` to the attacker for both
Most Populous and Twenty Population, while subtracting the city's full population
from the defender. An attacker at 10 capturing a population-6 city from an
incumbent at 21 now correctly predicts 15–15, which leaves the title with the
incumbent. Territory estimates are unchanged.

The control is **master `fcdb97df20c3eb28f400dfe089b325bc73d6c061`**, captured at the
start of this task. The branch was fast-forwarded to that revision before editing.
The two frozen executables each ran 200 Quick games, seeds **810000–810199**, with
eight Normal scripted AIs, one of each faction, separate teams, 48×32 maps, four
workers, no victory turn limit, and a 600-turn non-awarding safety cap. Pairing
checks verified every seed and faction/seat assignment. Master subsequently
advanced with the ruin/celebration merge; that later rules revision is outside
this experiment.

| Metric | Master | Corrected planner | Change |
| --- | ---: | ---: | ---: |
| Median turns | 145.5 | 146 | +0.5 |
| P90 turns | 176 | 179 | +3 |
| Maximum turns | 260 | 301 | +41 |
| Games over 250 turns | 1 (0.5%) | 3 (1.5%) | +2 |
| Mean turns | 149.64 | 150.65 | +1.01 |
| VP victories | 200 | 200 | 0 |
| Capped games / panics | 0 / 0 | 0 / 0 | 0 / 0 |

Median averages the two central values. The simulator's upper-middle median is
146 for both runs. P90 follows its sorted-rank convention:
`sorted_turns[(90 * (N - 1) + 50) / 100]`, using integer division; here that is the
180th of 200 sorted observations. Every game is included, and “over 250” means
strictly greater than 250 resolved turns.

| Faction | Master wins | Corrected wins | Change |
| --- | ---: | ---: | ---: |
| Switzerland | 38 (19.0%) | 41 (20.5%) | +3 |
| Japan | 16 (8.0%) | 18 (9.0%) | +2 |
| Byzantium | 15 (7.5%) | 14 (7.0%) | −1 |
| Germany | 40 (20.0%) | 37 (18.5%) | −3 |
| Celts | 34 (17.0%) | 34 (17.0%) | 0 |
| Mongols | 44 (22.0%) | 42 (21.0%) | −2 |
| Sparta | 6 (3.0%) | 4 (2.0%) | −2 |
| Athens | 7 (3.5%) | 10 (5.0%) | +3 |

Each faction has 200 seats in each run. Across matched seeds, 21 games became
shorter, 145 retained their length, and 34 became longer; 10 changed winner.
These correctness fixes did **not** improve the length tail in this cohort.
The combined comparison does not identify each fix's separate contribution.
This is Quick Normal self-play evidence, with no claim of human-play strength,
other-speed performance, faction balance, or a maximum completion time.

| Seed exceeding 250 in either run | Master turns | Corrected turns | Corrected winner |
| --- | ---: | ---: | --- |
| 810048 | 260 | 247 | Switzerland |
| 810053 | 230 | 256 | Mongols |
| 810115 | 209 | 255 | Athens |
| 810178 | 233 | 301 | Sparta |

Four added regressions failed against the unmodified planner: the Reactor as
first step, duplicate Reactors among later steps, the capture-title tie, and the
capture-population milestone. That run exited 101 with 30 passing and four failing
endgame tests. With the fixes, all four pass. Two additional tests cover existing
and ordered commitments, continuing the reserved queue, resistance, teammates,
and the shared Great Work slot. All 36 endgame tests pass.

| Check | Result |
| --- | --- |
| `cargo fmt --all` | Exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --workspace --release` | Exit 0; 641 passed, 0 failed, 4 ignored |
| `cargo test --workspace` | Exit 0; 641 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | Exit 0; 8 passed, 0 failed, 0 ignored |
| Optimized Node-target WASM build | Exit 0 |
| WASM fixture replay | Exit 0; 5 tapes, 248 turns; initial, every-turn and save/load hashes match |
| `git diff --check` and conflict-marker scan | Exit 0; no conflict markers |

The [check summary](2026-09-27-endgame-planning-fixes/checks.json) preserves exact
commands, exit codes and test totals. The initial release attempt exited 1 in
server doctests (`E0460`) after 641 passing tests: concurrent fixture generation
replaced an un-hashed Wasm dependency artifact. The full serial rerun above
passed. Fixture generation and native workspace tests must run serially when
sharing a Cargo target directory; the additional WASM build used a separate
`target/wasm-verification` directory.

The documented three-game fixture regeneration command was run. MAX-2 and MAX-1
are byte-identical; MAX keeps its initial hash but changes four turns of orders
and ten hashes from turn 31 onward. Its first changed order is player 4's unit 25
moving to `(20, 14)` instead of `(20, 15)`. Independently regenerating from the
frozen master executable produces exactly the same three tapes as the candidate.
This refresh catches up with master's existing campaign-memory behavior. The
research-queue and movement-path fixtures are unchanged.

The [small summary](2026-09-27-endgame-planning-fixes/summary.json) records source,
binary and full-report SHA-256 hashes, exact argv, fixture comparisons and metric
definitions. The [200 paired records](2026-09-27-endgame-planning-fixes/paired.jsonl)
retain every seed's length and winner. These summaries and this report are the
only simulation results committed; the required test fixture is tracked with
the code. Rust/Cargo: 1.98.1, arm64 macOS.

Full reports, binaries, regeneration tapes and logs are staged outside git at
`/tmp/legatum-sim-artifacts/2026-09-27-endgame-planning-fixes/`. The requested
`~/LegatumData/sim-artifacts/2026-09-27-endgame-planning-fixes/` destination rejected
the write with `Operation not permitted` under this session's sandbox. The two
full JSON reports are approximately 8.1 MB each.

The control executable was built and copied before any source edits. Simulator
provenance samples the checkout at run start: both runs report `fcdb97d` plus a
dirty worktree because regression tests had been added before starting the
control run, and the candidate fixes were uncommitted during its run. The frozen
binary hashes and per-file source hashes distinguish the two policies.

To reproduce, build each revision separately and preserve its executable:

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
# Use a distinct LABEL and ARTIFACT_ROOT for each revision.
./target/release/legatum-sim --games 200 --seed 810000 --speed quick \
  --safety-cap 600 --threads 4 --label "$LABEL" --out-dir "$ARTIFACT_ROOT"

# Exact documented fixture regeneration command:
cargo run --release -p legatum-sim -- --games 3 --seed 18446744073709551613 \
  --turn-limit 40 --brain mixed --label determinism-fixtures \
  --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports
```

# Human playbook benchmark — 2026-10-01

Proxy v5 wins **69/400 games: 17.25% (95% Wilson CI 13.86–21.26%)**.
The requested human calibration is **not achieved**: it falls well below 40%,
and never reaches the human's eight-city position at the turn-136 observation.
The implementation is opt-in, deterministic and restricted to `PlayerView`.
Default Proxy v4, the seven scripted Hard opponents, engine rules and app code
are unchanged. [Usage and policy](../archive/human-playbook-benchmark.md).

## Matched results

Each policy played 400 complete games, seeds **3600001–3600400**, Standard speed,
48×32 map, eight independent seats, Peace start, barbarians on, current city fire
enabled. The benchmark seat rotates; each seat and faction occurs 50 times.
Every game's seed, seat, faction, opponent factions, teams and difficulties were
matched across all three policies. All 1,200 games ended in VP victories, with
**zero panics and zero stalls**. There was no artificial turn limit; the safety
cap was 1,200. The GreedyExpand comparator uses the checked-in Hard v2 policy and
its behavior primitives, against seven **scripted** Hard opponents.

| Brain | Wins / 400 | Win rate | 95% Wilson CI | Winning turn, mean / median (wins only) |
|---|---:|---:|---:|---:|
| Proxy v4 | 7 | 1.75% | 0.85–3.57% | 290.29 / 295 (n=7) |
| GreedyExpand | 51 | 12.75% | 9.83–16.38% | 256.80 / 255 (n=51) |
| Human playbook / v5 | 69 | 17.25% | 13.86–21.26% | 208.77 / 210 (n=69) |

| Brain | Cities T100, mean / median (n) | Cities T136, mean / median (n) | Captures through T136, mean (n) | Captures over complete game, mean / median |
|---|---:|---:|---:|---:|
| Proxy v4 | 1.93 / 2 (400) | 2.41 / 2 (400) | 0.355 (400) | 0.980 / 1 |
| GreedyExpand | 2.39 / 2 (400) | 2.84 / 3 (398) | 0.000 (398) | 0.030 / 0 |
| Human playbook / v5 | 2.82 / 3 (400) | 3.46 / 3 (398) | 0.389 (398) | 0.708 / 1 |

Complete-game capture totals are 392, 12 and 283, respectively, across 400 games
each. Captures include recaptures; they exclude stolen settlers. Snapshots are
after resolving the named turn. Games ending earlier are excluded from that
snapshot, with no zero filling or carried-forward states. Thus the T136 means
describe games still observable then. Win-rate denominators remain 400.
V5's two earlier games were wins at T129 and T134, with five cities each.

V5's gain is 15.50 percentage points over v4 and 4.50 points over GreedyExpand.
On matched seeds, v5 alone wins 66 games versus v4 alone winning 4; both win 3.
Against GreedyExpand, the corresponding counts are 53, 35 and 16. The latter
advantage remains uncertain (two-sided exact McNemar p=0.069). The Wilson intervals
describe this seed schedule; they do not estimate Sam's personal win rate.

## What the candidate still needs

The human reference is `~/LegatumData/analysis/playtest3/human-play.md`, supported
by `ai-failures.md`, `benchmark.md` and `civ6-openings-and-ai-playbooks.md` in the
same directory. Its source is engine `29a5b2a`, seed 211025479670853, Switzerland.
That game predates automatic city fire. The save shows eight cities and 7/10 VP
after resolving turn 135, displayed to the player as turn 136; it does not supply
a completed human victory turn. These are behavioral reference points from one
older game, rather than a matched control for today's rules.

| Behavior | Human reference | V5 observation |
|---|---|---|
| Reach three founded cities | T65, including a recovered settler | Median T70 among 376 games that ever do; 134/400 by T65 and 312/400 by T100 |
| Start the conquest | War T102; first capture T113 | Own declaration median T110 (n=344); first capture median T129 (n=206) |
| Chain captures | Three cities by resolved T135 | 123/398 have any capture at T136; only 1/398 has three |
| Turn gold into a concentrated army | Four Rangers bought for 700 gold during T106–107 | Median first military purchase T107 (n=388), but no game buys four military units within two consecutive turns by T136 |
| Recover or steal settlers | Two by T103 | 307 over complete games; 210/400 games steal at least one, 77/400 steal at least two; 143 settlers are lost |
| Reach the eight-city position | Eight at displayed T136 | At resolved T136: 59/398 reach five, 4/398 reach six, 0/398 reach eight |
| Turn expansion into science | Science/turn 8 at T100 and 15 at T135 | Means 5.49 at T100 and 9.70 at T136 |

The following are measured priorities for another candidate, not predictions
that any single change will produce a 40% win rate:

1. Finish sieges and keep capture infantry alive. Of 567 distinct city/turn
   pairs where v5's attacks reported zero city HP, 318 (56.1%) ended with that city
   still enemy-owned. Repeated turns against one city count repeatedly; this is
   not a count of 318 distinct failed conquests. V5 suffers 9,000 city-fire hits
   and loses 4,914 units overall. Its 2,429 city-shot events with a dead defender
   can include simultaneous unit damage or multiple shots at one victim, so they
   are not an exclusive count of kills caused by cities. The tactical tests prove
   selected volley/capture and preemptive-retreat cases; full games still expose
   failures in positioning, reinforcement and finishing an attack. A stronger
   planner needs to coordinate the melee arrival with the zero-HP turn and
   preserve the ranged group through city fire.
2. Fund a complete attack wave and field siege technology on time. V5 spends
   a mean 863.69 gold on 3.96 military purchases over the complete game, but only
   63/400 games make even two purchases within two consecutive turns by T136.
   Across entire games, 101/400 achieve a two-purchase burst and only 3/400 a
   four-purchase burst. Purchase timing and composition need to be tied to the
   campaign rather than filling individual vacancies. No game researches
   SiegeEngineering by T200; only 17 ever do, at median T307. The current early
   siege therefore relies on Rangers. The policy can build Siege Engines, but
   does not yet deliver a timely siege-technology plan against current city fire.
3. Convert the opening into enough cities. The fixed transition to
   consolidation after two distinct held captures or five cities is more modest
   than the reference's five foundings plus three captures. Continuation should
   depend on available targets, healthy troops, income and a funded next settler
   or replacement wave. The current brain wins 36/70 games with at least two
   complete-game captures and 32/59 games with five cities at T136. Those are
   associations, not causal estimates; they identify useful cohorts to inspect.
4. Improve consolidation and faction victory execution. The median tech
   gap to the best original opponent goes from 0 at T100 to −1 at T136 and −3 at
   T200. Only 81/348 T200 observations match or exceed that opponent's tech count.
   Also, 47 games complete 1,119 Scouts combined: when useful production options
   run out, the priority fallback can choose a cheap Scout despite its negative
   priority. This needs an explicit idle/project choice rather than uncontrolled
   replacement production. Faction wins range from 0/50 for Sparta and 2/50
   for Germany to 15/50 each for Mongols and Byzantium; Switzerland wins 12/50.
   A reliable general benchmark must handle those faction objectives as well as
   the shared expansion plan.

V5's faster opening and lower recorded unit losses are useful diagnostic progress. It
still has fewer total captures than v4, and its results do not establish that it
represents a strong human. Reaching 40% would require 160 wins in this schedule,
91 more than observed. No measured intervention in this work establishes that
gain.

## Reproduction and evidence

The implementation and checks were based on master commit
`7730da41295ad4681819079820ed1ba79a25a14a`, using rustc and Cargo 1.98.1.
Six development screens used separate
seeds starting at 3500001. After those screens and correctness fixes, v5 was
frozen before the 400-game acceptance run. The deficiencies above were recorded
without tuning the policy against acceptance outcomes.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
CARGO_BUILD_JOBS=4 nice -n 10 cargo build --release -p legatum-sim
nice -n 10 ./target/release/legatum-sim \
  --proxy 0 --proxy-version 5 --proxy-start peace --rotate-proxy \
  --legacy-hard all --difficulty hard --barbarians true \
  --games 400 --seed 3600001 --speed standard --threads 4 \
  --out-dir /tmp/legatum-human-playbook/v5 --label v5
```

Repeat with `--proxy-version 4`, or replace the version selector with
`--proxy-opening GreedyExpand`, keeping every setup flag and seed unchanged.
The new opening selector is needed because the standalone challenger schedule
does not use this exact seat/faction assignment. `--proxy` alone remains v4.

The committed [compressed evidence](2026-10-01-human-playbook-benchmark.evidence.json.gz)
contains the selected raw per-game proxy telemetry for all 1,200 games, final
player summaries, aggregate metrics, paired outcomes, raw-report SHA-256 hashes,
binary hashes and a 230-file Rust/Cargo source manifest. It retains every diagnostic
used above; unrelated per-turn scout histories are omitted. Full original reports
and check logs are under `/tmp/legatum-human-playbook/` on the measuring machine.
The source manifest identifies the measured dirty worktree precisely.

V4 and GreedyExpand were measured using the frozen baseline binary, and v5 using
the final binary. The later observer fields for zero-HP city/turn pairs exist only
in v5's acceptance report; baseline absence is not interpreted as zero. Eight full
games per policy were rerun using the final binary with `--determinism`: all 24
repeat pairs matched, and every turn hash matched the corresponding acceptance
game. These repeats are excluded from the 400-game counts. All 1,200 acceptance
games also had successful hash-checked observer replay, with complete AI and
proxy incident coverage.

## Checks

| Command | Result |
|---|---|
| `cargo fmt --all` | Exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --workspace --release --no-fail-fast` | Exit 0; 1,153 passed, 0 failed, 4 existing ignored tests |
| `cargo test --workspace` | Exit 0; 1,153 passed, 0 failed, 4 existing ignored tests |
| `cargo test -p legatum-sim --test determinism` | Exit 0; 14 passed, 0 failed |
| `git diff --check`, `git diff --cached --check` and conflict-marker scan | Exit 0; no markers |

The first release run exited 101 because the new Wilson test's expected midpoint
bound was slightly mistyped. The formula was correct; the reference constant was
corrected, then the entire release suite passed. Tests cover real fog isolation,
closed-border war staging, simultaneous volley/capture, keeping a settler raider
in position, and retreat before the next unavoidable city shot, as well as the
purchase reserve and phase memory.

Builds used four Cargo jobs, tests used one test-harness thread, and each benchmark
batch used four workers, with at most two heavy jobs alongside each other.
Commands were invoked with `nice -n 10`, but macOS sandboxing rejected
`setpriority` with `Operation not permitted`; the commands continued and their
actual exit codes were checked. Niceness was therefore not applied. No app,
`docs/learnings.md` or `CLAUDE.md` changes were made.

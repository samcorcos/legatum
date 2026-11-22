# Quick endgame pursuit

Control: current master at task start, `83ff4b1bedfd2b61278832af87273f73023670eb`.
Both sides use that engine, map generator, faction assignment and simulator. Only
`crates/ai` decision policy changes. Eight Normal scripted AIs, one of each faction,
Quick speed, no turn limit, runner safety cap 600. The control executable was copied
before editing and used for every control rerun. Candidate source and executable
hashes are in the evidence manifest.

Master advanced to `625eff57d9cfc7c012d5ef67684d696a45736dd3` during the
investigation, including separate movement and remembered-threat changes. The
control remains the master revision captured at task start; these paired results
do not measure the later merged rules. This branch has not merged master.

## Lengths

Quantiles use the simulator's rounded index, `floor((N - 1) * p + 0.5)`, over all
observed lengths. A cap stop would remain in the denominator and be censored, not
a victory. Seed 700347 was also run independently, without double-counting it in
the 2,000-game distribution.

| Seeds | Policy | P50 | P90 | P99 | Max | Mean | >250 | >300 | Safety caps / panics |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 700001–702000 | Master | 147 | 185 | 258 | 470 | 152.233 | 28 | 9 | 0 / 0 |
| 700001–702000 | After | 145 | 177 | 234 | 308 | 148.596 | 13 | 2 | 0 / 0 |
| 810000–810199 | Master | 147 | 192 | 311 | 389 | 157.450 | 6 | 4 | 0 / 0 |
| 810000–810199 | After | 148 | 184 | 240 | 264 | 151.720 | 1 | 0 | 0 / 0 |

Every game in all four runs ended by VP. The main cohort's median is near Quick's
150-turn pace; the worst case falls 162 turns, while some individual games lengthen.

The separate 200-game comparison is seeds **810000–810199**. Its median is
147 → 148; games over 250 turns are 6 → 1; longest is 389 → 264. The remaining
game over 250 is 810034, at 264 turns. These are development-seed self-play
comparisons, not held-out or human-opponent evidence.

## Every control game over 300 turns

All thirteen were recorded again using the preserved control executable. Their
turn counts, outcomes, players, captures, eliminations and entire per-game telemetry
match the original batch records exactly. Every archive was checked with
`legatum-sim --replay <archive.tape.json.zst> --analyze`; all per-turn state hashes
matched. The saved archives and compact diagnostics accompany this report.

| Cohort | Seed | Before | After | Battles before | Captures before | Eliminations before | Main missed scoring routes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 2,000 | 700212 | 332 | 192 | 3,427 | 4 | 0 | Cash goal repeatedly spent on reinforcements; empire-wide investment suppression |
| 2,000 | 700273 | 386 | 173 | 4,060 | 5 | 0 | Cash goal repeatedly spent; multi-building title pursuit |
| 2,000 | 700634 | 301 | 186 | 2,423 | 7 | 0 | Celts geography goal and title pursuit despite a large treasury |
| 2,000 | 701204 | 327 | 267 | 2,240 | 20 | 4 | Title race and useful capture selection after several eliminations |
| 2,000 | 701249 | 377 | 224 | 3,406 | 16 | 1 | Celts at nine VP without enough qualifying hill cities; useful conquest |
| 2,000 | 701397 | 304 | 180 | 2,109 | 9 | 0 | Cash goal delayed by repeated military spending; title investments |
| 2,000 | 701433 | 381 | 197 | 3,122 | 7 | 0 | Cash spending loop plus Celts's missing geography goal |
| 2,000 | 701699 | 470 | 245 | 4,150 | 22 | 2 | Split titles and geography constraints; capture selection |
| 2,000 | 701966 | 363 | 145 | 3,075 | 16 | 2 | Cash goal unbanked even at eight VP; title competition |
| 200 | 810043 | 389 | 184 | 4,554 | 13 | 1 | Multi-building economic/science deficits and useful captures |
| 200 | 810085 | 303 | 167 | 2,458 | 3 | 1 | Cash goal delayed by purchases; title competition |
| 200 | 810195 | 311 | 219 | 2,685 | 12 | 1 | Switzerland at nine VP, still spending the cash goal |
| 200 | 810197 | 349 | 246 | 2,647 | 9 | 1 | Split titles, investment suppression and capture selection |

The routes overlap; these labels describe observed decision failures, not mutually
exclusive experimental treatments. Changing one empire's orders changes its
opponents' subsequent views and decisions, so shorter reruns do not isolate a
single causal effect for each seed.

## Every final game over 300 turns

The final main cohort has two different outliers. Both were independently recorded,
matched against the batch, analyzed, and replayed through native and WASM. There
are no final paired-cohort games over 300.

| Seed | Before | After | Battles after | Captures after | Eliminations after | Diagnosis |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| 700269 | 267 | 307 | 2,455 | 20 | 2 | Contested titles during conquest; Mongols eventually combines Economy, Territory and Army |
| 701311 | 221 | 308 | 3,101 | 5 | 0 | Split titles and a low-income early leader; Mongols builds toward Economy while Army changes hands |

In 700269, Switzerland has already banked its cash goal by turn 200. Mongols's
Economy value rises from 98 at turn 200 to 194 at 300, overtaking the rival lead;
its territory grows from 147 to 207. It finishes by retaking Army on turn 307.
There are 1,351 production completions, 378 purchases and 141 title changes. The
44 funded-but-unfinished unit city-turns include local crowding, but do not stop
empire production or scoring. This is residual attrition and title competition,
not the cash-goal spending loop or a global queue deadlock.

In 701311, Byzantium holds nine VP at turns 175–225 but has only 1–7 net gold/turn
and loses Science to Athens. Mongols's Economy value progresses
82 → 102 → 127 → 141 → 177 at turns 200/225/250/275/300 while the rival lead
reaches 180. At turn 308 it buys the Workshop queued in city 34, takes Economy
at 181, and wins with Economy, Territory and Army. The game has 1,513 production
completions, 455 purchases, 77 title changes and two funded-but-unfinished unit
city-turns. The scoring plan is making progress, but competing empires delay its
payoff. Neither residual tail shows a peace lock, an eliminated-title holder or
a treasury that could immediately buy victory and remains unused. These two
long games remain a limitation; this change does not guarantee a maximum length.

The historical named repros no longer run long on this master: **700347 = 111**,
**810042 = 124**, and **810312 = 136** turns, before and after. The older
600/555/596-turn observations are not treated as this experiment's baseline;
810312's historical observation was also from a different branch.

## Diagnosis and changes

### Spending and threat gates suppressed scoring

In control 700212, Switzerland had eight VP at turn 200, 88 gold and 43 gold/turn,
but bought another Soldier in its full-health border city. At turn 300 it had nine
VP, 73 gold and 53 gold/turn, still without its two-point cash goal. Control 700273
shows the same loop: at turn 300, eight VP and 218 gold, without the goal.
The emergency buy ran before victory planning and skipped that planning pass for
the entire empire. Ordinary purchase/upgrade reserves did not protect the cash
goal from this earlier pass.

The AI now reserves the cash goal during endgame pursuit when nearby troops plus
city defense can absorb the visible threat. A force exceeding that defense still
permits emergency spending. It always evaluates victory investments in other
cities after an emergency buy, using the remaining treasury and reserving that
city's orders. A newly ordered emergency unit also counts toward Army, preventing
a second city from buying the same missing title twice. Defended border cities
can invest in a scoring building; an undefended city still reinforces.

### Title pursuit stopped at one building

The existing planner knows public title standings, strict leads and team ownership,
but only Army had a multi-step investment estimate. Economy and Science required
one building to cover the entire gap. For example, 810043's Sparta had nine VP
at turn 250 but an economic deficit of 40 and a science-history deficit of 374;
its broad empire had infrastructure still to build. Routine priorities continued
funding units and unrelated construction while no single building received title
credit.

Economy and Science now compare a sequence of legal building investments against
other scoring routes. Every later building is charged in full, with queue time;
the first step alone can consume current stored production and quoted gold.
Science includes its rolling-history delay. The estimate rejects a deficit that
the available buildings cannot cover and never invents research, population or
new cities. It is recalculated from the public standings after each turn.

### A nearby opponent is not necessarily a useful conquest

Celts can hold four titles and five milestone points while lacking the geography
needed for its two-point faction goal. In the diagnostic follow-up of 701249,
only one of five cities had two own or claimable hills; there were no settlers.
More buildings in the other four cities cannot repair that geography. By the late
game, useful hill sites may already be occupied by other empires.

Endgame Celts can exceed its ordinary settlement quota for a known, claimable hill
goal, budgeting settlers, founding, infrastructure and growth. Those settlers
prefer the goal sites and do not count foreign-owned hills as free resources.
Campaigns also value progress toward their own population/territory titles,
bankable milestones and relevant faction goals, instead of ranking only enemy
strength, distance, the rival VP leader and threats at home. Capture estimates
include the rival's loss of population/territory and respect teammate-held titles.
Combat strength and safety thresholds are unchanged.

The campaign correction shortened the remaining diagnostic follow-ups 701249
from 379 to 224 turns and 810034 from 383 to 264. These are intermediate-policy
comparisons; the table above always compares untouched master with the final
policy. Final 701249 wins via all five titles without its faction goal; 700634
completes Celts's goal and wins with 11 VP. The policy supports both routes.

### Other suspected causes

- **No impossible VP target:** the rules offer five milestone points, two faction
  goal points and five live titles. Research completion does not remove Science
  title competition: it compares recent science income, not technologies left to
  research. Geography can make a particular faction goal unavailable without
  expansion or conquest; the AI must change its plan.
- **No production/queue liveness deadlock in these thirteen controls:** they
  continue producing, buying and fighting. Fully funded but unfinished unit queues
  account for only 0–15 city-turns per game, across all cities and items, far below
  the duration of the scoring plateau. The simulator recorded 372–751 purchases
  and 2,109–4,554 battles per game. No production or queue rule was changed.
- **Movement incidents are present:** the analyzer reports oscillation, wandering
  settlers and repeated bounces. These observations are not automatically causes
  of the scoring stalls: affordable scoring routes were being suppressed while
  production and combat continued. The fix changes strategic priorities, not
  pathfinding or collision rules.
- **Eliminations are not trapping the score:** the controls have 0–4 eliminations;
  all still finish by VP. Live titles exclude eliminated players. The replay and
  telemetry do not show a dead player holding a required title.
- **No diplomacy peace lock:** this revision has permanent hostility between
  distinct teams and no diplomacy/peace orders.

## Regressions and checks

Eight added endgame tests cover cash under bombardment (and the true-emergency
exception), concurrent defense and scoring with a shared treasury, defended-city
Science investment, multi-building title completion, full follow-on costs and
Science delay, Celts expansion beyond its quota with foreign hills excluded,
goal-aware conquest, and title transfer accounting with teammate ownership.
The existing threatened-city test now distinguishes a defended border from an
undefended one. The endgame module has 30 passing tests.

- `cargo fmt --all`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace --release`: 587 passed, 0 failed, 4 ignored.
- `cargo test --workspace`: 587 passed, 0 failed, 4 ignored.
- `cargo test -p legatum-sim --test determinism`: 7 passed, 0 failed.
- `scripts/build-wasm.sh`: exit 0; Node-target `wasm-pack` build also passed.
- WASM tape checks: all four fixtures (240 turns) and all eighteen final selected
  games passed initial-state, every-turn and save/load hash verification.
- `cd app && npx vitest run`: 64 files passed; 716 tests passed, 1 skipped.

The documented mixed-brain regeneration command updated the three 40-turn tapes.
Their initial hashes are unchanged; their first changed orders/hashes are turns
3, 6 and 4, respectively. A separate regeneration with the preserved master
executable produces byte-identical new tapes: the committed fixtures already
lagged current master's AI orders. This task's endgame policy does not cause those
opening differences. The 120-turn research-queue tape remains unchanged. Native
debug and release determinism checks were repeated after regeneration, seven
tests passing in each mode.

## Reproduction and evidence

At each source revision, build and preserve the executable before switching code:

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
./target/release/legatum-sim --games 2000 --seed 700001 --speed quick --safety-cap 600 --threads 8 --label quick-long-2000
./target/release/legatum-sim --games 200 --seed 810000 --speed quick --safety-cap 600 --threads 4 --label quick-long-paired
./target/release/legatum-sim --games 1 --seed 700347 --speed quick --safety-cap 600 --threads 1 --label quick-long-700347
```

For the other named repros substitute 810042 or 810312. To record an outlier, add
`--archive --archive-dir /tmp/legatum-archives --tape-out target/quick-tapes`.
To regenerate the diagnostic narrative from the saved evidence:

```sh
./target/release/legatum-sim --replay docs/sim-results/2026-09-27-quick-long-games/before/seed-700212.tape.json.zst --analyze --out target/quick-analysis
```

The [evidence manifest](2026-09-27-quick-long-games/manifest.json) links compressed
full batch reports, before/after archives for every control and final outlier,
the three historical repro seeds,
diagnostic snapshots/counters, checks, and source/binary SHA-256 provenance.
Exact commands, exit codes and compressed check logs are in
[checks.json](2026-09-27-quick-long-games/checks.json).
The archive rules fingerprint is unchanged because no core or WASM rules changed;
the binary was built before the task commit, so its embedded HEAD is the base
revision. The source hashes distinguish the candidate AI from that control.

No turn limit or automatic victory was added. The estimates remain heuristics:
opponents can grow, contest titles and change the available routes. This study
measures Quick Normal self-play on the requested seeds; other speeds, difficulty
levels, human play and the project's faction-balance/diversity targets are not
established by these results. In the main cohort, Switzerland wins rise from
321/2,000 (16.05%) to 363/2,000 (18.15%); all faction counts are retained in the
manifest. Improving its previously suppressed faction goal changes competitive
balance as well as game length.

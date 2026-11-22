# Human playbook v6 — 2026-10-01

Proxy v6 wins **125/400 games: 31.25% (95% Wilson CI 26.90–35.95%)**, versus
**60/400, 15.00%**, for the current v5 control. Captures increase from 271 to 580.
**The 40% target is not met:** it would require another 35 wins in this schedule.
V6 still never reaches eight cities at T136. It is a stronger benchmark candidate
on these seeds, not a completed calibration to the human reference.

## Matched comparison

The primary comparison reruns both policies against the scripted Hard opponents
at base commit `0639267762f8c8021fcd150b0dc650ef54ae8568`. V5's implementation is
byte-for-byte unchanged. The historical v5 result was **69/400, 17.25% (95% Wilson
CI 13.86–21.26%)**; the fresh control is **60/400, 15.00% (11.83–18.83%)**.
Several opponent files, including `scripted.rs`, `civilians.rs`, `military.rs`,
`production.rs` and `victory.rs`, changed after the original measurement. Core
source hashes still match the original evidence. The old 17.25% therefore remains
historical context, rather than the control for an isolated v6 policy comparison.

| Policy | Wins / 400 | Win rate | 95% Wilson CI | Winning turn, mean / median (n) |
|---|---:|---:|---:|---:|
| V5, current opponents | 60 | 15.00% | 11.83–18.83% | 215.53 / 208 (60) |
| V6, same opponents | 125 | 31.25% | 26.90–35.95% | 221.24 / 223 (125) |

| Policy | Cities T100, mean / median (n) | Cities T136, mean / median (n) | Captures through T136, mean (n) | Complete-run captures, mean / median; total |
|---|---:|---:|---:|---:|
| V5 | 2.8425 / 3 (400) | 3.4369 / 3 (396) | 0.3586 (396) | 0.6775 / 0; 271 |
| V6 | 2.8525 / 3 (400) | 3.6947 / 3 (393) | 0.5394 (393) | 1.4500 / 1; 580 |

Each row uses 400 requested games, seeds **3600001–3600400**, eight independent
players, Standard speed, 48×32 maps, Peace start, barbarians on, and normal city
fire. Every seat and faction appears 50 times. Seeds, proxy seats, factions, teams
and all seven opponents' difficulties match between policies; every opponent is
recorded as scripted Hard. There is no turn limit; the safety cap is 1,200. For each
policy, 399 games end in a VP victory and one reaches the safety cap; there are
zero panics. V5 stalls on seed 3600185 and v6 on 3600026; both stalls count as
non-wins. The four v5 games and seven v6 games
ending before T136 are excluded only from that snapshot.

Win-rate denominators include all requested games. The intervals are two-sided
95% Wilson intervals for this seed schedule, not estimates of Sam's personal win
rate. Winning turns include wins only. City snapshots are after resolving the
named turn; games ending earlier are excluded, with no zero filling or carried
states. Captures include recaptures and exclude stolen settlers. The old human
reference had eight cities after resolved T135, displayed as T136, and predates
automatic city fire; it is a behavioral reference rather than a matched control.

V6 gains **16.25 percentage points** over the matched v5 control: 90 seeds are
won only by v6, 25 only by v5, 35 by both, and 250 by neither (two-sided exact
McNemar p=8.37e-10). More wins and captures do not mean faster victories: the
median winning turn increases from 208 to 223.

## What v6 changes

V6 is an independent opt-in fork, selected with `--proxy-version 6`. The default
remains v4, v5 stays unchanged, and there are no engine, opponent-policy or app
changes. All planning and persistent memory derive from `PlayerView`. Ordered
collections and explicit tie breaks keep decisions deterministic; there is no
RNG, seed inspection, full-state access or observer feedback in the policy.

- Three cities unlock a small initial force before further
  workshops and expansion. Research explicitly reaches SiegeEngineering, and
  settlers value revealed missing strategic resources. Settler production
  continues toward six cities. Three distinct held captures or seven total cities trigger
  consolidation; campaigns also have a time limit.
- Reserve a surviving melee finisher after a conservative
  ranged volley. Project unavoidable current fire before assigning that unit,
  allow an arrival from two tiles only when the path completes this turn, and
  combine adjacent melee attacks in the same movement round. Garrison overkill
  does not become fictitious city damage. Movement assigns an infantry screen
  before the ranged line, accounting for a city's single selected shot.
- Rotate a unit while it can survive the current shot.
  Moving away cannot cancel city fire that resolves before movement. These are
  forecasts from visible positions; hidden orders and reinforcements remain
  unknown.
- Save a wave reserve based on visible buy quotes, release it across
  front cities, or replenish an assembled force at the start of conquest. Preserve
  upkeep reserves and existing economic production. The real-engine fixture buys
  four reinforcements over two turns; full-game results below show how often that
  opportunity actually materializes.
- Reinvest spare treasury in markets, science and production,
  start a victory project in the best available city, handle faction infrastructure
  and Sparta home guards, and stop selecting negative-priority Scout production.
- Stage near visible exposed settlers for a
  legal declaration, capture them with safe reachable units, and probe remembered
  weak cities with an adequate force. Two healthy veterans can attack a suitable
  unwalled city without waiting for Rangers. A remembered lack of guards is not
  treated as knowledge of hidden units. Settlers seek safe known frontiers when
  no known legal founding site remains.

## Development measurements

Nine 64-game revisions used separate seeds **3700001–3700064**. These seeds were
reused during development, so the screens are selection evidence, not nine
independent samples or a substitute for the acceptance run. The final policy was
frozen before seeing any v6 acceptance outcomes; no policy tuning followed them.

| Revision | Wins / 64 | Total captures | Cities T136, mean | First capture, median turn (n) |
|---|---:|---:|---:|---:|
| Current v5 | 7 | 31 | 3.26 | 130 (23) |
| 1 | 14 | 64 | 3.20 | 156 (33) |
| 2 | 12 | 41 | 3.19 | 167 (27) |
| 3 | 10 | 41 | 3.09 | 182 (26) |
| 4 | 10 | 39 | 3.81 | 163 (25) |
| 5 | 10 | 30 | 3.94 | 178 (21) |
| 6 | 7 | 38 | 3.80 | 161 (18) |
| 7 | 12 | 31 | 4.02 | 159 (19) |
| 8 | 9 | 73 | 3.14 | 156 (37) |
| 9 | 17 | 80 | 3.44 | 133 (40) |

The last two controlled comparisons help identify the size of the remaining
timing problem. Revision 6 → 7 changed remembered-city probing and settler frontier
exploration: wins rose 7 → 12, with five newly won seeds and no lost wins.
Hash-checked development traces had shown a force waiting at radius three while
ordinary sight reached only two, plus settlers holding for dozens of turns when
their known settlement sites ran out.

Revision 8 → 9 prioritized completing the first small force and relaxed the
healthy-veteran threshold for mature unwalled cities: wins rose 9 → 17, captures
73 → 80, captures through T136 rose 9 → 31, and median first capture moved
T156 → T133. Nine seeds became wins and one stopped being a win. These are effects
of the combined changes on reused development seeds; they do not isolate an
individual heuristic or establish a general population effect.

## Remaining gaps, ranked by measured impact

This priority order uses the development interventions, the frequency of failed
positions, and their association with winning. It is not a set of additive or
causal estimates of recoverable wins; no measured intervention here establishes
the additional 35 wins needed for 40%.

1. Bring the first effective force and follow-up captures forward. This has the
   largest measured development gain: revision 8 → 9 adds eight wins in 64 seeds.
   In acceptance, only **18/393** observations have three captures by T136 (v5:
   0/396), and neither policy reaches eight cities then. V6's median first capture
   is T134 among 262 games that capture at all, versus T126.5 among 188 for v5.
   At T136, v6 wins 48/59 games with at least two captures, versus 30/258 with none.
   Those cohorts are associations and include different maps and faction outcomes.
   Research is much earlier—SiegeEngineering median **T163 (n=363)** versus
   **T310 (n=18)**, and 353/400 versus 0/400 research it by T200—but only seven v6
   games do so by T136, and **no v6 observation has a live Siege Engine at T136**.
   Only 106/316 have one at T200. Research alone does not deliver the siege force.
   Further work should measure local assembly, resource supply, movement and
   reinforcement timing, including probes of remembered weak targets.
2. Hold conquests and protect the settlers that extend them. V6 loses **137
   cities across 98 games**, versus v5's 52 across 47; 81 of those 98 v6 games are
   non-wins. This is partly greater exposure from fighting more, not proof that
   preventing each loss creates a win. A hash-checked loss replay (seed 3600235,
   Athens) captures city 11 at T126, loses it at T127, recaptures it at T130,
   and loses it again at T133. That game records six captures and six city losses
   and ends at 8 VP. Captures therefore need healthy relief units and a defensible
   stopping point, not just a surviving finisher. Settler sniping improves from
   **257 to 391 captures**, but settler losses also rise **132 → 238** across
   **85 → 149 games**. Safer founding routes and replacement escorts remain gaps.
3. Convert the larger empire into science, faction objectives and a finished
   victory. At T200, median technology deficit to the best original opponent is
   **−4 for v6 (n=316)** versus **−3 for v5 (n=350)**, despite mean city counts
   of 4.87 versus 3.90. Sparta completes its goal in 23/50 games but wins only
   six; Germany completes its goal in just 6/50 and wins seven. The faction table
   below shows how uneven the policy remains. The Celts stall, seed 3600026, ends
   T1200 at 8 VP, with seven cities, all technologies and its faction goal still
   incomplete. V6 produces no unwanted Scouts (v5: 1,367), but 5,168 of its 7,102
   production-idle incidents come from that stalled game. Avoiding pointless
   production still needs an adaptive plan to claim the remaining public VP.
4. Make treasury release coincide with an actual attack window. V6 makes a
   four-unit purchase burst within two consecutive turns by T136 in **1/400**
   games, versus 0/400 for v5; over complete runs the counts are **6 versus 1**.
   Two-unit bursts by T136 improve only **58 → 70**. Median first military
   purchase moves later, **T107 → T123**. V6 spends mean 867.11 gold on 3.775
   military purchases, versus 812.08 on 3.865 for v5. The burst mechanism works
   in controlled engine tests, but the economy, upgrades and travel schedule
   seldom create the human's four-Ranger moment. Increasing a fixed reserve
   alone was not sufficient in development.
5. Finish the remaining sieges without another city-fire cycle. Of city/turn
   pairs attacked down to zero HP, **205/733 (28.0%)** still end enemy-owned for
   v6, down from **341/567 (60.1%)** for v5. These are repeated city/turn pairs,
   not distinct failed conquests. This is substantial coordination progress, but
   still leaves repeated failures to occupy a disabled city. City-fire hits rise
   **9,264 → 12,087**, and total recorded military-unit losses **4,973 → 5,165**;
   exposure is greater, so these counts do not isolate the effect of city fire.
   Forecasts must continue to account for its pre-movement shot, incoming enemy
   units and a fresh melee replacement rather than relying on a late retreat.

| Faction | V5 wins / 50 | V6 wins / 50 | V6 faction goals completed / 50 |
|---|---:|---:|---:|
| Japan | 3 | 9 | 30 |
| Germany | 3 | 7 | 6 |
| Mongols | 12 | 25 | 30 |
| Sparta | 1 | 6 | 23 |
| Byzantium | 13 | 27 | 50 |
| Athens | 5 | 12 | 37 |
| Switzerland | 11 | 21 | 27 |
| Celts | 12 | 18 | 20 |

## Reproduction and retained evidence

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
CARGO_BUILD_JOBS=2 nice -n 10 cargo build --release -p legatum-sim
nice -n 10 ./target/release/legatum-sim \
  --proxy 0 --proxy-version 6 --proxy-start peace --rotate-proxy \
  --legacy-hard all --difficulty hard --barbarians true \
  --games 400 --seed 3600001 --speed standard --threads 4 \
  --out-dir /tmp/legatum-human-playbook-v6/v6-final --label v6-final
```

Repeat with `--proxy-version 5` and a separate output directory for the control.
The measured v6 run additionally archived its tapes; archiving does not affect
orders. [Compressed evidence](2026-10-01-human-playbook-v6.evidence.json.gz)
retains selected per-game proxy telemetry, final player summaries, goal and VP
observations, every acceptance turn hash, matched outcomes, development summaries,
hash-checked trace excerpts, raw-report and binary hashes, the Rust/Cargo source
manifest and validation receipts.
Full raw reports, tapes and check logs remain under
`/tmp/legatum-human-playbook-v6/` on the measuring machine.

The independent analysis helper recomputes the main statistics and checks the
matched game setups from the committed evidence:

```sh
python3 tools/analysis/human_playbook_v6.py \
  docs/sim-results/2026-10-01-human-playbook-v6.evidence.json.gz \
  docs/sim-results/2026-10-01-human-playbook-v6.evidence.json.gz
```

Eight complete acceptance games per policy were rerun with `--determinism` using
the final standalone release build. All 16 repeat pairs match, and every turn
hash matches the corresponding acceptance game. Eight development games for the
selected revision were also repeated: all eight pairs and their original turn
hashes match. These repeats cover all eight seats and factions and are excluded
from the 400-game counts. The measured and standalone-build binary hashes are
both retained. All 800 requested benchmark games have successful independent
hash-checked observer replay and complete AI/proxy incident coverage.

## Checks

| Command | Result |
|---|---|
| `cargo fmt --all` | Exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --workspace --release --no-fail-fast` | Exit 0; 1,205 passed, 0 failed, 4 ignored |
| `cargo test --workspace` | Exit 0; 1,205 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | Exit 0; 14 passed, 0 failed, 0 ignored |
| `git diff --check`, `git diff --cached --check`, conflict-marker scan | Exit 0; no markers |

The checks include seven focused v6 unit tests, fourteen real-engine v6 integration tests, and the CLI version/start-mode repeat matrix. Fog invariance, pre-movement city fire, simultaneous captures, coordinated purchase bursts, remembered-city probes and blocked-settler exploration are covered.

The initial Clippy run exited 101 because an existing simulator test fixture
omitted `opening` from its `Config` initializer. Adding `opening: None` fixed that
test-only compilation error. A new settler fixture initially ended immediately
because the opponent had no city or units; adding a distant opponent city made
the intended multi-turn scenario valid. The first full release suite ran 1,205
tests successfully but exited 101 at server doctests (`E0460`): an overlapping
standalone build replaced the shared Wasm library artifact. The workspace checks
were rerun sequentially, with no overlapping Cargo build. These failures are not
counted as successful checks.

Heavy commands were invoked with `nice -n 10`. The macOS sandbox rejected
`setpriority` with `Operation not permitted`, so niceness was not applied; the
commands continued and their actual exit codes were captured. Development batches
used up to six workers; acceptance used four, and checks used two Cargo jobs and
one test-harness thread. The accidental build/test overlap described above could
briefly have permitted seven active workers (four simulation, two build, one
test). Subsequent Cargo commands were serialized to keep the concurrent budget
within six. No `docs/learnings.md` or `CLAUDE.md` edits were made.

# Eight factions: implementation and simulation

Implemented B6 + B7 on `astra/factions-implement`, based on both faction design
files. All eight rules, replacements, goals (+2 permanent VP once) and weaknesses
are active in the Rust engine and reflected in the scripted AI and UI content.

The final requested 1,000-game run has **every faction within 6–20% per seat**.
An independent 1,000-game run on seeds 1001–2000 also keeps every point estimate
in that band. This is a viability screen for this AI and map configuration,
not an attempt to make the factions equally strong.

## Method and evidence

```sh
cargo run --release -p legatum-sim -- --games 1000 --seed 1 --label factions-8 --status-file target/factions-8-status.json
cargo run --release -p legatum-sim -- --games 1000 --seed 1001 --label factions-8-holdout --out-dir target/factions-holdout
```

Both runs use 48×32 maps, eight separate teams, Normal scripted AI, 150-turn
limits and 10 VP targets. Every game has one seat of every faction, assigned by
a deterministic Fisher–Yates shuffle using a separate setup RNG stream. Seeds
and hashes remain integers/decimal strings across native and WASM. Each faction
therefore has exactly 1,000 seat observations per run. Draws count as no win.

[Final raw game records](../sim-results/2026-09-26-factions-8.json),
[runner report](../sim-results/2026-09-26-factions-8.md), and
[compact records of every tuning run and the holdout](factions-8-runs.json)
retain the evidence, parameters, report SHA-256 hashes and source revision.
The raw exploratory/holdout reports remain in the indicated ignored `target/`
directories; the final raw report is committed. Runs were made from parent
`d1e0c11007b203c6f6885e1460b62767cca46dad` plus this implementation, with a dirty
checkout as recorded by the runner. Formatting and regression-test updates
subsequently changed source text without changing the simulated rules.

CIs below are nominal 95% Wilson score intervals. The first batch was used for
tuning, so its intervals do not account for model selection; the untouched
second batch is reported separately. Results do not yet establish team-format,
human-play or terrain-stratified balance. No faction-specific map bonuses or
AI resource cheats were introduced.

## Final per-faction wins

| Faction | Wins / seats | Win rate | 95% CI | Independent seed win rate | Independent 95% CI |
| --- | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 161 / 1000 | 16.1% | 14.0–18.5% | 14.6% | 12.5–16.9% |
| Japan | 116 / 1000 | 11.6% | 9.8–13.7% | 13.2% | 11.2–15.4% |
| Byzantium | 162 / 1000 | 16.2% | 14.0–18.6% | 15.2% | 13.1–17.6% |
| Germany Compact | 76 / 1000 | 7.6% | 6.1–9.4% | 7.3% | 5.8–9.1% |
| Celts | 75 / 1000 | 7.5% | 6.0–9.3% | 8.8% | 7.2–10.7% |
| Mongols Kin | 125 / 1000 | 12.5% | 10.6–14.7% | 12.3% | 10.4–14.5% |
| Sparta | 67 / 1000 | 6.7% | 5.3–8.4% | 6.3% | 5.0–8.0% |
| Athens | 96 / 1000 | 9.6% | 7.9–11.6% | 9.2% | 7.6–11.2% |

No final point estimate is outside the requested band. Sparta is closest to
the lower edge; its uncertainty interval crosses 6% in both runs. None of the
upper intervals reaches a dominant 20% rate. Preserve the present identities
and collect more matchup/team/human evidence before further tuning.

## Goals and completion turns

Each rate includes every initial seat, including eliminated empires. A goal is
observed only if awarded before that game ends; incomplete goals are censored
by victory or the turn limit, not assigned an invented completion turn. Turns
below describe successful completions only. The collector records the first
resolved scoring turn and never overwrites it.

| Faction | Goals / seats | Completion rate (95% CI) | Mean turn | Median turn | P10–P90 turns |
| --- | ---: | ---: | ---: | ---: | ---: |
| Switzerland | 514 / 1000 | 51.4% (48.3–54.5%) | 94.6 | 90 | 78–121 |
| Japan | 298 / 1000 | 29.8% (27.0–32.7%) | 87.0 | 81 | 66–117 |
| Byzantium | 973 / 1000 | 97.3% (96.1–98.1%) | 83.3 | 81 | 71–98 |
| Germany Compact | 682 / 1000 | 68.2% (65.2–71.0%) | 97.9 | 92 | 79–126 |
| Celts | 366 / 1000 | 36.6% (33.7–39.6%) | 105.0 | 103 | 86–129 |
| Mongols Kin | 393 / 1000 | 39.3% (36.3–42.4%) | 87.3 | 84 | 72–108 |
| Sparta | 128 / 1000 | 12.8% (10.9–15.0%) | 129.5 | 130.5 | 111–146 |
| Athens | 793 / 1000 | 79.3% (76.7–81.7%) | 86.3 | 84 | 72–102 |

Sparta's simultaneous healthy-garrison/incursion goal remains substantially
later and rarer than the others; Byzantium's exploration goal is much more reliable.
These are observed consequences of the distinct goal designs and current AI.
The AI actively plans both; Sparta goal pacing remains a useful next target
for human playtesting. A low completion rate alone does not establish that
the AI ignores its goal.

## Outcome mix

| Outcome | Seed 1–1000 games | Share | Independent games | Independent share |
| --- | ---: | ---: | ---: | ---: |
| VP target | 655 | 65.5% | 633 | 63.3% |
| Conquest | 0 | 0.0% | 0 | 0.0% |
| Turn limit | 223 | 22.3% | 236 | 23.6% |
| Draw | 122 | 12.2% | 131 | 13.1% |
| Unfinished | 0 | 0.0% | 0 | 0.0% |
| Panic | 0 | 0.0% | 0 | 0.0% |

Mean / median game length: **134.1 / 138 turns**; independent batch **135.1 / 139**. Zero panics and no runner-stopped games in either batch.

## Adjustments, without homogenizing

The first run used the proposed numbers: Germany buildings at 80%, Foundry +3,
Mongols military-only movement with 2-move Settlers, and the original-three
rebalance. Germany (0.8%) and Celts (3.1%) were the clear outliers.

Celts's automatic border expansion still valued hills at ordinary yields. Using
Celts's own yields for border selection fixed that missing rule path and raised
its win rate without changing its 2 Food / 1 Production hills or Cairn Singer.
Germany's initial three-city AI target forfeited the four-city milestone; it
now permits four cities and reserves the population-8 growth focus for its
capital. Distinct personalities remain intact.

| Stage (1,000 games each, same seeds) | Germany wins | Celts wins | Sparta wins |
| --- | ---: | ---: | ---: |
| Initial implementation | 0.8% | 3.1% | 7.6% |
| Faction-aware hill claims and capital planning | 2.1% | 7.4% | 6.3% |
| Foundry +4 | 3.6% | 6.9% | 6.6% |
| Foundry +5 | 4.9% | 7.3% | 5.4% |
| Foundry +6; Sparta garrison rotation | 7.8% | 7.6% | 5.4% |
| Final: Foundry +6, Oathbearer 24 | 7.6% | 7.5% | 6.7% |

Only two design numbers differ from the supplied starting values:

- **Foundry output: +3 → +6 Production**, still costing 48. +4 and +5 did not
  bring Germany into the band. Its 80% building costs and +20 cost for every
  unit remain; this strengthens its industrial advantage instead of removing
  its expensive-army weakness.
- **Oathbearer melee: 22 → 24**, still costing 50 and moving 2. Infantry wound
  immunity, 40-Gold upgrades, and 5/10 military healing remain. The AI can also
  vacate a Ranger garrison when healthy Troopers are ready to fill its goal roles.

The live engine already had a 75-Production base Settler, despite the older
faction draft's 80-based examples. That existing base is preserved: Germany
Settlers cost **95, 115, 135…**, exactly the required +20 faction surcharge.
Other factions' goals, weaknesses and numerical strengths are unchanged from
the original-three document's final proposals.

## Implementation and compatibility

- Stable faction serde names are `Switzerland`, `Japan`, `Byzantium`, `Germany`,
  `Celts`, `Mongols`, `Sparta`, `Athens`. Unique pieces retain base
  item IDs in orders/saves; faction-aware data supplies names and stats. There
  is no second base item to build alongside its replacement.
- Production, purchases, upgrade differences, tile work/claims, upkeep,
  spawn/reset movement, healing, combat/forecasts, remembered-city strength,
  title metrics and replay displays use the appropriate faction data. All core
  arithmetic remains integer-only. Goals reject cities in resistance and use
  the empire's own cities, techs and units; allied territory does not satisfy
  the Sparta incursion.
- Each faction has distinct knobs and tested goal/unique-piece decisions.
  `AiDifficulty` implementation is unchanged. Frozen AI-v1 test code received
  only a neutral fallback for the five enum variants it never knew.
- UI data lists all eight names, mottos, abilities, unique pieces, goals,
  weaknesses and heraldry. Existing picker/join/city/unit content bindings use
  those names; no component layout or CSS was restyled. The development mock's
  roster/research/name tables accept all eight; authoritative rules and AI are
  in Rust/WASM, not a new parallel implementation of every rule in the mock.
- An actual save generated by the unmodified parent core, and the first turn
  of an original three-faction tape, remain frozen compatibility fixtures.
  They deserialize successfully. Historical tape hashes intentionally do not
  match changed rules; replay validation still rejects stale/corrupted hashes.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all` | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, no warnings |
| `cargo test --workspace` | **311 passed, 0 failed, 4 pre-existing ignored** |
| `cargo test -p legatum-sim --test determinism` | **6 passed** |
| `cargo test --release -p legatum-sim --test determinism` | **6 passed** |
| `wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node` | Passed, optimized release |
| `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node` | **3 tapes / 120 turns passed**, including initial hashes and save/load |
| `scripts/build-wasm.sh` | Passed, optimized release web package |
| `cd app && npm test` | **126 passed in 15 files** |
| `cd app && npm run build` | Passed, TypeScript and Vite production build |
| Requested 1,000 games, seed 1 | 1,000 finished; 0 panics; every faction 6–20% |
| Independent 1,000 games, seed 1001 | 1,000 finished; 0 panics; every faction 6–20% |

The four ignored tests are existing performance/large-evaluation checks:
`midgame_speed`, `paired_evaluation`, `many_games_summary`, and the explicit
cross-build profiling corpus comparison. None of the new faction tests is
ignored. Opening-pacing regressions retain the ordinary-faction targets and
now explicitly allow the designed higher Germany/Oathbearer unit costs.

The fixture diff was reviewed as an intentional rules/setup/AI change: three
mixed-brain games, the same high-u64 seeds and 40 turns apiece, newly pinned
seat permutations, and changed initial/per-turn hashes. No fixture is
regenerated by tests. The native and WASM replay checks cover uninterrupted
replay and save/load at every boundary.

WASM commands use `WASM_PACK_CACHE=/tmp/legatum-factions-wasm-cache` and
`XDG_CACHE_HOME=/tmp/legatum-factions-xdg` because the default cache location is
not writable in this worktree's sandbox. The first default-cache attempt
failed; both release packages subsequently built and passed with the writable
cache. No repository optimization settings or permission policy were changed.

`docs/learnings.md` was deliberately left untouched for the parallel work.

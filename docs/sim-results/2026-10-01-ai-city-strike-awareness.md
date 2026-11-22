# AI city-strike awareness — 2026-10-01

Historical pre-V9 experiment from `e6a961d`, rejected for reduced offensive conversion.
The current-master revision is documented in [the v2 report](2026-10-01-ai-city-strike-awareness-v2.md).

The shared scripted planner now budgets city fire for Default, Easy/Normal/Hard,
every opening and Hard v2. Tile danger includes known hostile cities with positive
HP. Unit-specific exposure uses core forecasts with destination terrain and lost
fortification. Recovery anticipates the pre-movement ranged phase and the next
volley. Coordinated assaults charge one shot per city at its lowest-HP/ID target,
recompute melee forecasts at post-shot HP, and credit committed ranged support.
Guaranteed ranged damage cannot spill through a garrison into city HP. The mock
AI uses its existing combat mirror and only PlayerView. Versioned Proxy policies
and the deliberately random test opponent remain fixed controls.

City-fire losses fell **54.5%**
in the 400-game Standard cohort, while captures fell
**61.6%**. Neither final Standard nor final Proxy
cohort stalled. Hard won 125/128 final Proxy lobbies versus
127/128 before. These measurements establish better survival from city
fire, **not stronger play against a human-like opponent**; reduced offensive
conversion remains a material limitation for review.

The paired Standard results below use **scripted Hard**, eight independent players,
default 48×32 map, Standard speed, default barbarians, no explicit turn limit and
the default 1,200-turn safety cap. There are 400 games per executable on seeds
3200001–3200400. These are paired before/after lobbies, not mixed-version seats in
one game. Every seed, initial seat/faction/difficulty assignment and first setup
was checked for agreement. No seeds were discarded.

| Metric | Master baseline | Final branch |
| --- | ---: | ---: |
| Player-owned units lost in city-fire phases / game | 12.925 | 5.875 |
| Total player-owned city-fire losses | 5170 | 2350 |
| City captures / game | 0.215 | 0.0825 |
| Total captures, including recaptures | 86 | 33 |
| Mean observed turns | 260.8725 | 262.4550 |
| P90 observed turns | 305 | 312 |
| Safety-cap stalls | 0/400 | 0/400 |
| Panics | 0 | 0 |
| Faction win-rate spread, max minus min | 15.75 pp | 13.00 pp |

Observed turn statistics include cap stops, which are censored observations and
never victories. City-fire losses count unique player-owned military units at
zero HP after a ranged phase in which a city hit them. Simultaneous unit fire may
also contribute; this is **not an exclusive last-hit attribution**. Multiple cities
hitting the same destroyed unit count once. Barbarian losses are stored separately
by owner in JSON and excluded from these player-loss figures. The same observer-only
telemetry patch was used in both executables; it does not affect decisions or hashes.

| Faction | Baseline wins / 400 | Final wins / 400 | Baseline rate | Final rate |
| --- | ---: | ---: | ---: | ---: |
| Switzerland | 89 | 85 | 22.25% | 21.25% |
| Japan | 45 | 40 | 11.25% | 10.00% |
| Byzantium | 31 | 34 | 7.75% | 8.50% |
| Germany | 50 | 57 | 12.50% | 14.25% |
| Celts | 88 | 78 | 22.00% | 19.50% |
| Mongols | 30 | 36 | 7.50% | 9.00% |
| Sparta | 26 | 33 | 6.50% | 8.25% |
| Athens | 41 | 37 | 10.25% | 9.25% |

The Proxy cohort follows `docs/archive/ai-proxy-benchmark.md`: Proxy v4, Peace start,
rotating seat and faction, Standard, seven scripted Hard opponents, barbarians off,
no turn limit, 128 games per executable on seeds 700001–700128. Each seat and faction
appears 16 times; every seat/faction pair appears twice. All 128 games on each side
were played twice with matching hashes, plus the automatic analysis replay.

| Proxy benchmark metric | Master baseline | Final branch |
| --- | ---: | ---: |
| Games won by a Hard opponent (seven opponents combined) | 127/128 (99.22%) | 125/128 (97.66%) |
| Proxy wins | 1/128 (0.78%) | 3/128 (2.34%) |
| Hard city-fire losses / game, all seven seats | 10.28125 | 5.140625 |
| Hard captures / game, all seven seats | 0.4375 | 0.09375 |
| Proxy captures / game | 0.867188 | 0.882812 |
| Hard cities at turn 150, all seven seats | 23.102 | 23.070 |
| Hard techs at turn 150, per original seat | 9.269 | 9.277 |
| Surviving Hard empires at turn 150 / 7 | 6.922 | 6.930 |
| Mean / P90 observed turns | 248.49 / 286 | 250.97 / 285 |
| Stalls / panics | 0 / 0 | 0 / 0 |
| Determinism repeat checks | 128/128, 0 failures | 128/128, 0 failures |

Hard already wins almost every baseline Proxy lobby. This cohort therefore has
little ability to establish increased strength against human-like play; a small
change in wins is not evidence of human-level improvement. Read the capture,
survival and stall tradeoffs alongside win rate. Proxy is deliberately frozen so
opponent-policy drift cannot explain a change.

The first candidate, retained under `branch/`, added fire awareness before shared
volley coordination. It reduced Standard city-fire losses to 5.970/game but captures
to 0.0475/game, with one stall; Hard won 125/128 Proxy games. That exposed the existing
Default accounting gap: already-fired ranged units supported Rush assaults only.
The final implementation shares that support and uses core forecasts for the
follow-up. No decision parameters, game rules or seed assignments were tuned. Because the
implementation was revised after viewing this cohort, these seeds are development
evidence rather than an independent held-out evaluation.

The baseline was built from `bac090c4aa1ca782c091b78ac7458890205639a2` with only the
observer patch. Master at the start was `4f40d2143a9b9927ad2fb6e125c8700584674709`;
the sole difference was three documentation lines in `docs/learnings.md`, so its
engine, AI and simulator code were identical. The preserved executable reproduced
all three old 80-turn golden hashes exactly. Run metadata reports the checkout
observed at runtime (`bac090c`, dirty), not a compile-time embedded revision. The
saved observer patch and binary checksums disambiguate the before executable.

Artifacts: `/tmp/ai-city-strike/` (the sandbox denied the requested
`~/LegatumData/sim-artifacts/ai-city-strike/` directory). Headline cohorts are in
`baseline/` and `final/`; `branch/` is the intermediate candidate. Raw JSON, Markdown,
binaries, logs, Rust source manifests, fixture comparisons and the aggregation script
remain there. Standard and Proxy simulation commands requested `nice -n 10` and
used at most eight worker threads. **The macOS sandbox rejected `setpriority`, so
the requested lower scheduling priority could not be enforced.**

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
# Run once with each preserved baseline/final executable and matching output directory.
nice -n 10 "$binary" --difficulty hard --brain scripted --players 8 --speed standard \
  --games 400 --seed 3200001 --threads 8 --out-dir "$output" --label hard-standard
nice -n 10 "$binary" --proxy 0 --proxy-version 4 --proxy-start peace --rotate-proxy \
  --difficulty hard --players 8 --speed standard --games 128 --seed 700001 \
  --threads 8 --determinism --out-dir "$output" --label proxy-v4-hard
python3 /tmp/ai-city-strike/summarize.py
```

The [fixture receipt](../../crates/sim/tests/fixtures/README.md#ai-city-fire-awareness-2026-10-01)
explains each re-record. The three 40-turn tapes and three 80-turn Normal/mixed
recordings remain byte-identical. A new 160-turn scripted Hard tape changes at
turn 105 and pins the new policy; its initial state remains identical. All eleven
tapes, 615 turns, pass native/WASM replay and every-turn save/load checks.

| Required check | Final result |
| --- | --- |
| `cargo fmt --all` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace --release --no-fail-fast` | exit 0; 1100 passed, 0 failed, 4 ignored |
| `cargo test --workspace` | exit 0; 1100 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | exit 0; 14 passed, 0 failed, 0 ignored |
| `scripts/build-wasm.sh` and Node WASM build | both exit 0 |
| `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node` | exit 0; 11 tapes / 615 turns |
| `cd app && npx tsc --noEmit` | exit 0 |
| `cd app && npx vitest run --maxWorkers=4` | exit 0; 1,345 passed, 1 skipped; 105 files passed |
| `git diff --check`; conflict-marker scan | both clean |

The initial app run had one five-second timeout in the existing 40-turn mock-game
test while builds and simulations overlapped; its other 1,343 tests passed. The
final full run passed with the unchanged timeout. Earlier Rust failures were an
obsolete scout expectation beside a live city, the fixture-count assertion after
adding the eleventh tape, and transitional helper renames; all final suite exit
codes above were read explicitly.

WASM builds used `WASM_PACK_CACHE=/tmp/ai-city-strike/wasm-pack-cache` and
`CARGO_TARGET_DIR=target/wasm-check`. Cargo compilation and test concurrency were
limited to eight; app tests used four workers. No merge or deployment was performed.

| Artifact | SHA-256 |
| --- | --- |
| `baseline/legatum-sim` | `14394cd6766ae82b9d06f03c72a160eebd0efe67c80e7768518558f7e3756ec0` |
| `final/legatum-sim` | `c05c411978dbd3988e47f3230d4c7c6994724465bbc561f2673ab36cfd173734` |
| `baseline/2026-10-01-hard-standard.json` | `9a7f7baa9985cda913fb0bb4208199cdee249518d9262d95f734dec333b2ada4` |
| `final/2026-10-01-hard-standard.json` | `bb4c8866cbc8f67251d177c4d832b5192073cf050d67bba0da0ce2abc53971a7` |
| `baseline/2026-10-01-proxy-v4-hard.json` | `bd5ec9c344241d21171d39b5909eed521d1d402bb4a5a8e747cac00a7ba14b9f` |
| `final/2026-10-01-proxy-v4-hard.json` | `9cd1bb26a146c6701fd6af36caf850feef81154d75b163cc4442828fa7beb670` |

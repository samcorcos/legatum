# Coordinated city assaults — current-master revision, 2026-10-01

This revision starts from master `a9f6224340bc95325a3d19e095bf225f59f7a4da`,
including V9 faction balance, real Rush, and the opening identity fixes, and
cherry-picks `e6a961d` as `e5f0530`. The conflicts retain master's V9 golden
hashes and both historical fixture receipts. The earlier experiment's numbers
are not the baseline for this revision.

The AI retains city-aware exposure, scouting, retreat and healing for troops
outside a viable assault. A separate city-assault pass runs before recovery and
reserves the participating melee and ranged units together. It forecasts up to
four turns of ranged suppression, simultaneous city fire, observed supporting
defenders and sequential melee attrition, including the approach turn. Each city
charges one shot to the lowest-HP/ID participant. A city suppressed in the ranged
phase still fires; garrison overkill cannot spill into city HP. Movement removes
fortification. Wounded survivors do not leave a forecast capture to heal.

The army gathers outside the two-hex firing ring and chooses distinct reachable
staging tiles. Once assembled, it can buy a survivable sighting at range two;
otherwise infantry's two-hex vision can leave it permanently blind at range three.
Ranged and siege support enter with the group. Movement reconciliation cancels a
blocked group together and recognizes reaching a battle as an approach, without
pretending the attacker has already occupied the city. Guaranteed ranged defeat
reserves exactly one occupier to avoid friendly collisions. Rush retains its
existing sustained siege approach when the short capture forecast is not yet ready.

Forecasts use only PlayerView. They are bounded expectations, not an oracle for
future enemy orders or reinforcements. The mock city-fire helper now skips safe tiles before constructing defense and
uses 41 shot rolls instead of 1,681 melee roll pairs, with an exact-odds regression.
No faction knobs, difficulty bonuses,
opening genomes, combat rules or victory rules are changed. The frozen Proxy
policies retain their original geography.

## Paired Standard acceptance

Both preserved release executables run 400 games on seeds 3200001–3200400,
scripted Hard, eight independent players, Standard speed, default 48×32 map and
barbarians, with no turn limit and the default 1,200-turn safety cap:

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
"$binary" --difficulty hard --brain scripted --players 8 --speed standard \
  --games 400 --seed 3200001 --threads 6 --out-dir "$output" --label hard-standard
```

The baseline is an archived current-master source tree with only the same
observer-only city-loss telemetry patch. Its preserved executable independently
reproduces all three current-master 80-turn tape digests. Neither telemetry nor
source-checkout metadata participates in orders, RNG, resolution or state hashes.
Simulator Git metadata reports the checkout at runtime, not the executable's
compile-time revision; the archived source, observer patch and executable hashes
disambiguate the baseline.

All prescribed numerical gates pass on the final executable. The game seeds,
settings and per-seat factions/difficulties match between the two cohorts.

| Standard measure, 400 games | Master | Revision | Change |
| --- | ---: | ---: | ---: |
| Captures | 77 | 106 | +29 |
| Captures per game | 0.1925 | 0.2650 | +37.66% |
| City-fire losses | 4,496 | 2,283 | −2,213 |
| City-fire losses per game | 11.2400 | 5.7075 | −49.22% |
| Mean observed turns | 250.8800 | 253.1725 | +0.91% |
| 90th percentile turns | 295 | 300 | +5 |
| Stalls / panics / turn-limit exits | 0 / 0 / 0 | 0 / 0 / 0 | Unchanged |
| VP victories | 400 | 400 | Unchanged |

The final command exits 0. Its 400 game records also match the earlier replay-
inspection candidate apart from wall-clock timings; the later changes restored
Rush behavior, while this cohort uses Default. The final duel below is rerun
separately because it exercises Rush.

The master reference moved to `3719f23` during the checks. Its intervening changes
touch documentation and the duel evolution/identity tooling; the engine, AI,
simulator and `tools/duel/duel.py` are unchanged from the starting master baseline.

City-fire losses count distinct player-owned units at zero HP in a ranged phase
where a city hit them. Simultaneous unit fire may also contribute; this is not
exclusive last-hit attribution. Multiple city hits count once. Barbarian losses
are excluded. Captures include recaptures. Turn means include any capped games as
censored observations, never victories. The prescribed seeds are development
evidence, not an independent held-out test or a human playtest.

## Coordinated attacks in replay

A matched 40-game subset (3200001–3200040, the same settings as the acceptance
cohort) was re-recorded as complete tapes and replayed independently. Every turn
hash matched. A city attack turn groups distinct player-owned attackers by turn,
attacking player and city tile, including fire into a garrison. City automatic
shots and barbarian attackers are excluded. A coordinated turn has at least two
attackers; this is a combat-event measure, not a count of declared wars or entire
siege episodes.

| Replay measure, 40 games | Master | Revision |
| --- | ---: | ---: |
| Coordinated city attack turns | 36 | 36 |
| Attackers per coordinated turn | 2.139 | 2.722 |
| Largest attacking group | 3 | 5 |
| Solo city attack turns | 215 | 52 |
| All city attack turns | 251 | 88 |
| Distinct player/city targets attacked, summed per game | 130 | 37 |

The sample establishes larger coordinated attacks with the same number of
coordinated attack turns. It also shows substantially fewer solo attacks
and fewer distinct targets, so it must not be described as an increase in all
attack attempts. The full cohort's capture increase is the offensive-conversion
gate. The targeted regression also checks a five-unit infantry/ranged/siege
approach and a wounded capturer retaining its orders through reconciliation.

## Duel matrix

The exact requested command is run for both executables, with the tool's unchanged
defaults: seven openings, 20 mirrored maps per unordered pairing, Hard, Standard,
small map, barbarians on, seeds beginning at 1930001, six workers, 1,120 games each.

```sh
python3 tools/duel/duel.py --sim "$binary" --run "$scratch" --seeds 20 --workers 6
```

Default's mean is the unweighted mean of the six off-diagonal scores, excluding
self-play. The final executable exits 0 and **Default's mean does not fall**:

| Duel measure | Master | Revision |
| --- | ---: | ---: |
| Default mean score | 0.795833 | 0.804167 |
| Rush mean score | 0.495833 | 0.483333 |
| Mean observed turns | 247.2107 | 250.3616 |
| Stalled games | 5 / 1120 | 5 / 1120 |
| Rush early-pressure identity passes | 65 / 320 | 58 / 320 |

Both full matrices are `NOT VALID` under the tool's broader diversity criteria:
Tall and Economic lack significant wins and have low mean scores. The revision
also has no significant Rush win over another opening. Rush's identity gate is
below its 60% threshold on both sides, and declined by seven seats. Thus the
Default guard passes, but this is not a claim that every opening improved.
Default's score against Rush decreases from 0.800 to 0.775; its aggregate gain
comes from other matchups. Twenty mirrored seeds give wide uncertainty bounds.

An intermediate candidate disabled the old Rush siege approach once city fire
was enabled. That cut its identity passes to 8/320. The final code retains that
approach, respects earlier capture commitments, and restores most early pressure.
It does not change opening genomes or their identity thresholds.

## Fixtures and checks

The [fixture receipt](../../crates/sim/tests/fixtures/README.md#coordinated-city-assaults-on-current-master-2026-10-01)
records all generation commands and first differences. The three 40-turn tapes,
three 80-turn golden recordings and seven manual scenario tapes regenerate
byte-identically. The 160-turn Hard tape changes against current master at turn
132; its initial hash/setup match and its 29 later order maps/hashes differ.
The pre-V9 cherry-picked tape cannot be reused under V9: its first changed
resolution hash is turn 1. All eleven committed tapes contain 615 turns.
The short Hard tape pins a changed movement decision; it contains no city-attack
events. Coordinated combat is covered by the focused regressions and the full-game
replay sample above.

The initial focused test runs exposed integration mistakes in movement
reconciliation and zero-HP occupation; these were corrected. Restoring the Rush
fallback exposed a second-occupier collision, caught by the existing regression;
the fallback now respects the earlier capture commitment.

Three full Vitest attempts hit the old five-second limit on the existing
40-turn/eight-empire mock-game test during concurrent builds and simulations.
Isolated measurements were 2.293 seconds on current master and 1.972 seconds on
the revision. The functional test now allows 15 seconds, retaining every
assertion; the final full four-worker run passes. This timeout adjustment is
separate from the exact-mean optimization above.

One initial full release run exited 101 after all test binaries passed: another
Cargo invocation had replaced the unhashed native WASM rlib before rustdoc. The
inspection helper was moved to its own target directory and the complete release
suite was rerun without concurrent writes to its build artifacts. A later run
correctly exited 101 on the Rush second-occupier regression; that fix was tested
before the final complete run. The first full debug run likewise passed all test
binaries but exited 1 at rustdoc after a shared-artifact mismatch; the complete
debug suite was rerun with no competing builds and exited 0. Exit codes, not just the last
passing test line, are recorded in the receipts.

| Final check | Result |
| --- | --- |
| `cargo fmt --all` | Exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --workspace --release --no-fail-fast` | Exit 0; 1,115 passed, 4 ignored |
| `cargo test --workspace` | Exit 0; 1,115 passed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | Exit 0; 14 passed |
| `scripts/build-wasm.sh` | Exit 0 |
| Node WASM build and `scripts/wasm-tape-check.mjs` | Exit 0; 11 tapes / 615 turns, including save boundaries |
| `cd app && npx tsc --noEmit` | Exit 0 |
| `cd app && npx vitest run --maxWorkers=4` | Exit 0; 1,377 passed, 1 skipped, 105 files |
| `git diff --check`, conflict-marker scan | Exit 0; no markers |

Frozen executable SHA-256 values:

```text
c4ef33ec71d73de245eeba546a0f213130940fe3701f0d408224b2d7e788f78d  master/legatum-sim
c4f96c71e1d7ad11bf632ede1ff51009a402ad4d18b8655e2f24c1bc24633ca3  accepted/legatum-sim
```

Raw Standard report SHA-256 values:

```text
053e0ff8ab90ff70bedf5cec4792b5a3dfd39fb6f73cc16fd4b6b9e90a38c819  master/2026-10-01-hard-standard.json
068cc4d335dd3bc19485a3a1253e55bccd98a7a8347d03a235adf8838988fbc5  accepted/2026-10-01-hard-standard.json
```

Artifacts are under `/tmp/ai-city-strike-v2/`: frozen executables, the observer
patch, source manifests, raw reports, duel caches/matrices, comparison scripts,
fixture recordings and check logs/exit-code receipts. No merge or deployment
was performed. `docs/learnings.md` and `CLAUDE.md` are unchanged.

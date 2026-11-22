> Archived 2026-11-22: kept for history; it no longer describes the project.

# Human playbook benchmark (Proxy v5)

For the separate opt-in v6 policy, use `--proxy-version 6` and see the
[v6 comparison and reproduction commands](../sim-results/2026-10-01-human-playbook-v6.md).
V5 and the default version selector retain their existing behavior.

V5 is an opt-in simulator opponent built from the playtest-3 plan: expand to about
three cities, assemble a ranged army, take a nearby weak city's territory, then
consolidate and pursue science and victory projects. It is a calibration candidate,
not an assertion that a simulated player is as strong as the designer. See the
[400-game measurement](../sim-results/2026-10-01-human-playbook-benchmark.md) for its
strength and remaining limitations on the current engine.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
CARGO_BUILD_JOBS=8 nice -n 10 cargo build --release -p legatum-sim
nice -n 10 ./target/release/legatum-sim \
  --proxy 0 --proxy-version 5 --rotate-proxy --proxy-start peace \
  --legacy-hard all --difficulty hard --barbarians true \
  --games 400 --seed 3600001 --threads 8 \
  --out-dir /tmp/human-playbook --label human-playbook
```

The eight independent seats use 48×32 maps and seeds 3600001–3600400. Every seat
appears 50 times; the existing 64-game rotation balances factions and covers all
seat/faction combinations. All seven opponents use scripted Hard. No economic,
vision or rule bonuses are given to v5. Default rules and the normal safety cap
remain in force; no turn-limit winner is substituted for an actual victory.

Repeat the command with `--proxy-version 4` for the frozen v4 baseline. For the
tuned GreedyExpand challenger, replace that version argument with
`--proxy-opening GreedyExpand`. This uses the same setup, faction rotation and
telemetry, unlike the standalone `--challenger` schedule. Reports identify the
opening by name and leave its Proxy version null. The opening uses the checked-in
Hard v2 policy, including its behavior primitives; its seven opponents remain
scripted Hard. Explicit version and opening selectors cannot be combined.

`--proxy` still defaults to version 4. The new selector exists only in the simulator;
no app or default AI behavior changes. Barbarians are explicit above because the
older Proxy harness defaults to no barbarians. Use `--determinism` to play each game
twice and compare every turn hash; only the first play counts. Every Proxy-harness
game also receives the existing independent tape replay and hash verification.

The policy takes only `PlayerView`. Its persistent state consists of a phase, a
known campaign city, previously observed city owners, captures inferred from those
observations, the campaign's starting turn and whether its purchase reserve has
been released. It uses ordered collections and
deterministic tie breaks, with no RNG, opponent planner, observer feedback or access
to `GameState`. Economy and combat forecasts use the public rules. Hidden enemies
cannot become attack targets. War planning may consider the intended enemy's known
territory as traversable after a declaration; actual movement obeys today's borders.

The expansion phase trains settlers without requiring a standing escort, but uses
known safe routes. Markets fund a reserved treasury; purchases buy military units
at the front without replacing city production. The campaign stages several
shooters and capture infantry at one target, focuses ranged attacks before melee,
and takes exposed hostile settlers. Two distinct captured cities still held, or
five cities, trigger consolidation; an unsuccessful campaign also has a time limit.
Consolidation seeks peace, replenishes defense
and funds science buildings and victory projects. Captured settlers can found cities
even after the opening's trained-settler limit.

The observer reports cities and technologies after resolving turns 100, 136, 150
and 200. Games ending earlier do not contribute a fabricated snapshot; each metric
has an observation count. Win rates include all requested games, treating panics,
stalls and draws as non-wins, and include a two-sided 95% Wilson interval. Winning
turns include wins only. City captures include recaptures and are separate from
settler captures. Diagnostics record founding/capture turns, declarations, purchase
turns and gold, unit losses, failed orders, treasury peaks and city-fire hits/kills.
City-fire damage includes rolled overkill. `city_fire_kills` counts city-shot
battles whose defender ends the simultaneous volley dead: unit fire may also have
contributed, and multiple city shots can share one victim. It does not establish
that cities alone caused those losses. V5's final run also records distinct
city/turn pairs attacked to zero HP, and how many remained enemy-owned after that
turn. Repeat sieges can contribute on multiple turns. These measurements never
enter the brain.

The original human evidence is under `~/LegatumData/analysis/playtest3/`. It comes
from an older engine where cities did not fire automatically. Current-engine
calibration keeps city fire enabled; the old human run is evidence for a plan, not
a matched control or proof of today's expected win rate.

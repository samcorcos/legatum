# Measuring losses after newly visible city attacks

Issue #366 asks whether late defense reduces city losses after a newly visible army attacks. `rush_ladder` now adds `late_city_assaults` to each game row. This is an observational acceptance tool, not a new AI policy. It never feeds its observations back into orders.

## Metric, version 1

- Observe the measured seat's planning `PlayerView` from the first simulated turn, including recorded-order warmup. Remember the first available sighting of each military unit, keyed by team and unit ID. Units remembered far from the later city still count as previously known; disappearing into fog does not reset their first sighting.
- An assault starts with a hostile, non-barbarian unit's first `Battle` on the measured seat's city tile. Fighting the garrison on that tile counts; city-fire attacks do not start an assault. Track one assault per city and rival team until peace, ownership loss, or the run ends. Reinforcements, quiet turns and replacement units do not restart it.
- Classify the opening turn's actual striking units together. `visible_within_two` requires **every** opening attacker to have appeared in a pre-order planning view and the earliest such sighting to be at most two turns before the attack. One older attacker makes it `visible_longer`; any attacker absent from all preceding planning views makes it `unseen_at_start`. Seeing an attacker only after resolution does not retroactively create warning. These are attacking-cohort definitions, not a claim to reconstruct every nearby supporting unit or human concept of an army.
- Keep pre-T120 assaults in memory so an ongoing siege is not mistaken for a fresh T120 attack, but report only assaults starting at `max(120, handover)`. A unit already visible in the first observation of a simulation starting after T1 has an unknown earlier history: report `left_censored` separately.
- `lost_to_attacker` means the city was later captured or automatically razed by that same attacking team during the assault. Another team's capture, an ordered raze, or another ownership change closes it as `other_ownership_end`; a new assault after recapture has a separate record. Peace closes the assault as `peace`. An unfinished assault at the end of observation is `right_censored`, not a successful defense. `start`/`end` preserve the elapsed siege time; there is no arbitrary capture horizon.
- Some losses, including undefended captures, have no preceding unit `Battle` for the capturing team. Report them separately as `losses_without_assault` tuples `[turn, city, captor_team]`. Do not infer a unit's warning from the capturing empire's identity or silently drop these losses. This bucket may also contain a competing team's capture or non-unit attack. Its visibility cohort is **unknown**.

The AI itself still receives only its normal view. Full-state city ownership and resolver events are used solely to score the outcome. Events are processed in order so attacks after a capture are not assigned to the previous owner.

## Paired comparison

Freeze one engine revision and change only the measured seat's `invasion_defense` gene; opponents retain shipped Hard. Use the same seeds and factions in both arms. The existing runner resumes by arm/seed, so use a **new output file** for this metric/version, engine and configuration; do not mix old rows or runs.

```sh
CARGO_TARGET_DIR=/path/to/private/target tools/sim/build-slot.sh -- \
  cargo build --release -p legatum-sim --example rush_ladder
RAYON_NUM_THREADS=2 /path/to/private/target/release/examples/rush_ladder \
  --arm baseline:invasion_defense=0 --arm candidate:invasion_defense=1 \
  --mode duel --players 6 --games 2 --seed 7101 --turns 180 \
  --out /path/to/private/new-run.jsonl
```

This two-pair T180 command is a pipeline smoke test, **not** the acceptance benchmark. For a later adequately sized comparison, run full games with a predeclared seed block and report completed-game win share, all city losses per game, all assault counts, each visibility cohort's opportunities and attributed losses, losses with unknown attribution, and censoring. Compare paired **per-game** differences as well as conditional loss rates: a policy can change whether attacks happen, so the arms need not have identical assault opportunities. Bootstrap uncertainty by paired game/seed, not individual battles. Do not interpret a truncated game's `won: false` as a completed loss.

The published 300-pair defense result predates the final #394 integration and lacks this cohort. A smoke run cannot establish fewer short-warning losses or no win-share regression. Keep #366 open until the larger evidence supports its acceptance criteria; unexplained missing-attribution losses are a measurement limitation, not a success.

## Validation and smoke result

Eight focused tests passed, including the two-turn boundary, unseen opening participants, replacement units, competing captures/recaptures, pre-window sieges, censoring, and a real event-adapter test for an undefended capture followed by an attack after ownership changed. `cargo clippy -p legatum-sim --example rush_ladder -- -D warnings` and formatting passed.

The command above ran on engine base `e15aa2da59628b8a41538082fc27e2c6feb0620c`, with this observer added, in about seven seconds on two threads. [Sanitized per-game output](smoke-7101-180.json) retains the episode records. All four games reached the T180 cap without finishing; no win-share conclusion is possible.

| Across two seeds | Defense off | Defense on |
| --- | ---: | ---: |
| All city losses (including before T120) | 3 | 4 |
| Reported late assaults | 5 | 3 |
| Observed-within-two assaults | 2 | 1 |
| Attributed losses in that cohort | 0 | 0 |
| Right-censored assaults | 2 | 0 |
| Late losses without an assault record | 0 | 0 |

This validates the output pipeline and exercises different assault opportunities. It does **not** show fewer short-warning losses: both baseline short-warning assaults are unfinished, and the candidate's one ended with another ownership change. Two seeds are insufficient for an effectiveness claim, and the raw city-loss count is not improved here. Keep #366 open for an independently reviewed metric and a larger full-game paired comparison. No private recordings were needed.

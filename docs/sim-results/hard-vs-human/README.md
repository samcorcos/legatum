# Hard = human

Fixed-map benchmark of Hard in the human's recorded seat. `maps.json` stores tape-verified setups and human milestones. Each salt changes only AI RNG; map generation, starts, combat and barbarians stay fixed.

For an AI behavior change, use one binary and a candidate `AiParams` file. The built-in genes are the baseline. `--seat-brain` applies the candidate only to the recorded human seat; every rival gets baseline params. This is the **north-star skill measure**. `--opponent-brain` applies the candidate only to rivals; the human seat gets baseline params. A lower human-seat win rate here means the rivals became harder. `--all-seats` applies the candidate everywhere and is the old mixed measure; it is the default when a params file is supplied. The modes keep map, salt, difficulty, search settings and candidate code identical. **Rivals play plain Hard (as the app's default lobby does) unless `--rival-search` is given; the maps' `search: true` flag is ignored otherwise (2026-10-28; earlier ledger rows used Search rivals that never bombed, see `docs/sim-results/ai-war/baseline.md` W2.0).** A code change must have an off-by-default gene that reproduces baseline behavior before its seat-split result can be interpreted.

For a **rule change** (the rules apply to every seat, so seat-split arms cannot isolate it), `hard_vs_human --rules-experiment FLAGS` applies comma-separated rules experiments to every map's setup, so the control is the same binary with the old rule restored (e.g. `crown-upkeep-after=0`) and an AI gene that switches the AI's knowledge of the rule off (`params/crown-free-aware-off.json`). Arms for the crown-upkeep grace: control, new rule with unaware AI, new rule with aware AI, and the two seat-split arms (see `docs/sim-results/overreach-grace/README.md`). Also build the true master binary in a separate worktree as the final control.

Run locally or on Graviton cloud:

```sh
tools/sim/hard-vs-human.sh --local --brain hard --salts 64
tools/sim/hard-vs-human.sh --local --brain hard+search --salts 64
tools/sim/hard-vs-human.sh --cloud --brain hard --salts 64
tools/sim/hard-vs-human.sh --cloud --brain hard+search --salts 64 --ai-params candidate.json --seat-brain
tools/sim/hard-vs-human.sh --cloud --brain hard+search --salts 64 --ai-params candidate.json --opponent-brain
tools/sim/hard-vs-human.sh --cloud --brain hard+search --salts 64 --ai-params candidate.json --all-seats
```

First run the baseline params (or no params) on the same salts and maps. Report three **candidate minus baseline** human-seat win-rate deltas: seat-only, opponent-only and all-seats. Negative is bad for seat-only, good for opponent-only; all-seats mixes the two effects. Use paired per-salt outcomes and an interval when judging whether a change exceeds noise. `params/settler-safety.json` and `params/sparta-war.json` encode the two candidate branches; `params/baseline.json` is the all-default control.

For saved game files, `python3 tools/sim/hard-vs-human-compare.py BASELINE-games.json.gz SEAT-games.json.gz OPPONENT-games.json.gz ALL-games.json.gz` reports the three paired deltas and 95% salt-bootstrap intervals. The interval reflects AI RNG on these fixed maps; it does not measure uncertainty across new maps.

Each wrapper run appends one JSON row to `ledger.jsonl` and redraws `chart.png`. Direct `hard_vs_human` runs write a JSON report and, with `--out-dir`, matched per-game outcomes. Overall score is mean map win rate; milestone ratio is the median of the available city and VP AI/human ratios across maps. A checkpoint after the recorded game ends is `null`. For a recorded human loss, `human.win_turn` and `human.win_type` are `null`, and `recorded_winner` keeps the AI winner's seat, titles, and milestones; no human-versus-AI win-turn ratio is computed for that map. Cloud runs build with `LEGATUM_CLOUD_EXAMPLES=hard_vs_human` and `run-batch.sh --bin hard_vs_human`.

For faster T30/T50/T100 experiments, pass `--until-turn N` to stop after resolving turn N. Games that end before N have no N snapshot. The report is marked `truncated` and omits win/loss fields and win-rate summaries. Per-game results retain eligible city checkpoints and include the seat's city count, capital-held state, VP, crowns, and public VP-leading team(s) at N. Map summaries give medians for numeric measures, the median 0/1 capital-held value, and leader-team counts. Truncated reports cannot be used for win-rate comparisons. Full runs without this flag retain the legacy report format.

CPU timing on 2026-10-11: one Mongols map, Hard, salts 1–16, four Rayon workers, measured with `tools/ops/timed.py` on the same release binary. Full runs used 755.4 CPU seconds; `--until-turn 100` used 43.6 CPU seconds (5.8% of full, about 17.3× less CPU), below the 40% limit.

For several arms in one cloud batch, pass `--arms` with an arms file as the `--ai-params` file: `{"arms": [{"label", "brain" (hard, hard+search, params, params+search), "mode" (seat, opponents, all), "params" (an inline AiParams), "rules" (rule-experiment flags, diagnostics only)}]}`. Every (arm, map, salt) runs in one flat parallel pass and each arm writes `<label>-games.json`. Per-game results also carry `diag` (the seat's units built by class and their Production, Settlers and first-Settler turn, disbands, desertions, losses, captures, Gold spent buying), `standings` at T60/100/150/200/250 and `final` (every empire's VP by source, era, cities, army and economy) and `crowns` (per empire from T50: turns crown upkeep exceeded gross city Gold, or half of it, empty-treasury turns). `runs/hard-88/tools/arms_report.py DIR [CONTROL]` prints the paired comparison; `merge_slices.py` joins a two-server batch. `LEGATUM_HVH_TRACE=<map>:<salt>` adds the seat's Gold, upkeep lines, crowns, decadence, era and research to its per-turn trace.

`LEGATUM_HVH_RESEED_DICE=1` (cloud: `run-batch.sh --env LEGATUM_HVH_RESEED_DICE=1`) gives every salt its own combat, movement and barbarian dice, as the war suite's `reseed` does. Without it all salts of a map share one set of dice, so a map's win rate mostly reflects how that dice sequence treats the seat's opening, and a change to exploration or opening order can flip a whole map (2026-10-18: one gene read -0.4 to -10.7 pp over four weights; reseeded it read +1.5 pp [-1.7, +4.5] and a 9,000-game fresh-seed league gave +7 pp). Prefer the reseeded run, or a league (`run-batch.sh --opponent-params`), for such changes. The reseeded control scores lower (26.9% against 35.2%), so never compare across the two modes.

For a fixed-map chaos-floor experiment, pass `--seat-ai-rng-offset K` directly to the binary. Its default is 0 and preserves the historical AI stream exactly. A nonzero K changes only the recorded seat's AI seed; map/setup seeds and opponent AI seeds keep their original derivation. Example: `hard_vs_human --brain hard --salts 64 --seat-ai-rng-offset 3`.

## War scenarios

Nine short campaigns cut from the user's playtests (the start state replayed exactly from his saves, then moved to today's rules), each with his own result as the target: `war-scenarios/README.md`, chart `war-scenarios/chart.png`, ledger `war-scenarios/ledger.jsonl`. Score an AI change with `tools/sim/war-scenarios.sh --brain hard --runs 64 --ai-params CAND.json --params-seats seat` (one spot server, about a minute). Two of them (added 2026-10-17, from the first AI offensive that beat the user) have the AI as the **attacker**: `tools/sim/war-scenarios.sh --check` is the attack regression check and exits 1 below 30%.

Baseline 2026-10-17 (64 runs each, Hard on the merged master 2e364c6e, Hard + Search on 06d93758 one merge earlier; share of runs that met the goal the user met, or for The Golden Horde the AI's own result in his game):

| Scenario | User | Hard | Hard + Search |
|---|---|---|---|
| The Mongol Storm: wipe out Byzantium and Athens (27 turns) | 26 turns | 0% | 0% |
| Sparta's First Blood: take Germany's two cities (12) | 8 turns | 100% (9 turns) | 100% (9 turns) |
| Sparta Against All: wipe out Japan and Byzantium (20) | 20 turns | 0% | 0% |
| Walls of Keystone: take Anvil and walled Keystone (12) | 11 turns | 0% | 0% |
| The Swiss Poachers: seven cities, stealing Settlers (14) | 10 turns | 0% | 1% |
| **The Golden Horde comes: the Mongol AI takes Aarau (14)** | AI: 12 turns | **43%** (8 turns) | 1% |
| Behind the Walls: hold five cities against Sparta (20) | held | 90% | 90% |
| Germany's Last Stand: hold five cities against the Mongols (24) | held | 98% | 98% |
| The Golden Horde comes, hold: Switzerland holds five cities (14) | lost Aarau | 56% | 54% |
| Suite (mean goal success) | | 43% | 38% |

The 2026-10-11 baseline had Sparta's First Blood at 0% for plain Hard (fixed since by `sparta_search_only` 0) and seven scenarios; the earlier numbers are in `ledger.jsonl`.

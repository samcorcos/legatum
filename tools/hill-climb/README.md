# Hard AI hill-climb measurement suite

This suite compares the Hard AI from two engine builds at recorded decision points. The Rust runner (`crates/sim/examples/hill_suite.rs`) verifies each original state hash, warms each AI brain on the recorded turns, then writes one JSON line per salt and arm. `suite.py` runs a cell list and summarizes outcomes. Keep actual tapes, states, and cell lists outside Git; `cells.example.json` shows the schema with placeholder relative paths.

## Cells and modes

A cell supplies a tape, hash-verified state stream, branch and end turns, the designer seat, and a success target: a named city's actual capture deadline (`target`) or the actual victory deadline (`win_by`). An early cell can set `report: 60` for the opening score. `target_name` follows a city if its runtime ID drifts. `set` is `train` or `heldout`; permanently held-out games yield numbers only and must never be traced or tuned on.

- `tape`: the designer's recorded orders. Orders rejected after a counterfactual are misfires. A tape result is a screen, not a gate.
- `hybrid`: recorded orders mapped to live units and cities, with shipped Hard filling orders that cannot apply. This is the designer-seat stand-in and the primary gate. `--hybrid-defend on|off` controls its defence against wars absent from the recording; a cell-level `hybrid_defend` overrides the command line.
- `hard`: shipped Hard plays the designer seat.
- `storm`: Hard plus a scripted rusher against the specified victim from `war_turn`.
- `quiet`: Hard in the designer seat, with war declarations stripped. Use its AI founded-city and Science reports as the neutral guardrail.

The **designer-seat score** is the percentage of runs where that seat fails to capture the target by its actual capture turn or fails to win by its actual win turn. Higher means the AI seats resisted the designer more often. The score of a set is the mean of its cell scores, not a pooled fraction. For opening cells, score only through T60. The runner reports capture timing relative to the real deadline so the hybrid seat's conservativeness remains visible.

Salt 0 uses recorded brain seeds and dice. Every higher salt reseeds each AI brain and the dice deterministically. Compare the same salt in both engines: `suite.py` prints discordant paired outcomes and an exact two-sided McNemar p-value. Tape summaries include low and high misfire halves; a cell is marked drift-heavy when their success rates differ by at least 40 points, and collapsed when an engine averages at least 100 misfires. Only usable tape cells enter the tape screen score.

## Run and gate

Build on the cloud box, with the shared build slot:

```sh
tools/sim/build-slot.sh -- cargo build --release -p legatum-sim --example hill_suite
python3 tools/hill-climb/suite.py \
  --engine champion=/path/to/champion/hill_suite \
  --engine candidate=target/release/examples/hill_suite \
  --cells /path/to/private/cells.json --out /path/to/private/runs \
  --salts 96 --threads 16 --set train
```

`--only` selects comma-separated cell names. `--reuse champion=/path/to/prior/run` reads raw JSONL from that run, preserving pair salts. `--set train|heldout|all` selects cells. The gate is 96 paired hybrid runs per cell, with no late-cell regression. Tape is a screen. The quiet guardrail at T60 allows no more than 0.3 fewer AI cities founded per seat and 10% less AI Science than the champion. A proposed opening-score gain also needs a held-out early-cell confirmation and an all-Hard pillar-8 check against measured A/A noise. Use `pillar8.py` for faction win-rate comparisons and `all_arms.py` to index prior run summaries; neither script changes game data.

`HILL_TRACE=<salt>:<arm>:<city>` and `HILL_ONLY_SALT=<salt>` support training-cell diagnosis. `HS_TECH` is a tech-parity screen; `HS_VP=<salt>` prints per-source VP for diagnosis. Never use these diagnostics on held-out games. Results are local artifacts. The checked-in tables in `results/` are sanitized historical measurements: `genes-table.md` and `border-army-score-of-record.md`.

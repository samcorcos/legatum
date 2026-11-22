# Card loop

One command, run from any directory, on this Mac (no cloud):

```sh
tools/sim/card-loop.sh                 # score every Lab card, search each failing one, write tasks
tools/sim/card-loop.sh --score-only    # the headline only (about 2 minutes)
tools/sim/card-loop.sh --cards sparta-walled-nicomedia --max-evals 120
```

Results land in `docs/sim-results/ai-quality/card-loop-queue/`: `README.md` (headline and
per-card table), `history.jsonl`, `runs/<date>/<card>/report.md` and `result.json`, and
`tasks/<date>/<card>-t<turn>.md`, one task per first divergence on a training card. Tasks are
never written to `.progress/astra-queue`; the lead triages them.

**Headline.** Hard's mean card score over the held-out Lab cards, and the count of cards where
Hard's mean is at least the designer's score from a run on the card's current revision and state
hash. The split is the scenario gate's, `docs/sim-results/scenario-gate/split.json` (one split for
every tool; add new cards with `tools/sim/scenario_gate_split.py --add`). Held-out cards are only
scored: `legatum-sim card-loop` refuses to search a held-out or unlisted card.

**One card by hand.**

```sh
cargo run --release -p legatum-sim -- card-loop scenarios/lab/<id>.json --out <dir> \
  [--run ~/LegatumData/scenarios/<id>-<time>.json] [--max-evals 300] [--score-only]
```

Hard and the line are measured on 50 held-out dice seeds (indices 10000-10049); the search
selects on training seeds 1-6 only. A line is a set of overrides on top of Hard, per unit, city,
research or diplomacy, keyed by ids so it plays on any dice. An override the engine refuses in a
position falls back to Hard's order. Candidates each turn: no new order, the battle search's
orders (`legatum_ai::training::search_turn`), the breach on each target city, marching every
nearby unit on a target city, and garrisoning a threatened held city. The search is greedy, turn
by turn, keeping a candidate only when the training-seed score rises.

**Designer seeds.** `--run` is refused unless the run's card revision and state hash match the
card: orders from another position are not a valid seed (the Hold Biel run of 2026-11-08 was on
revision 1; the card is now revision 2). Without a seed the search starts from Hard's orders, so
its result is not the kill check on his line.

**First-divergence report.** The first turn where the line's orders differ from Hard's, with
each changed actor, both orders, the card-score value of the override (the line against the line
without it), and Hard's planning passes for that unit, from a shadow Hard brain that re-runs the
turn with `ScriptedBrain::order_diagnosis`.

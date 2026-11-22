# Decision diffs from designer games

The decision diff is part of every designer review (`.claude/agents/legatum-game-coach.md` §4c). Run the recording tag against the save:

```sh
tools/decision-audit/designer-diff.sh \
  ~/LegatumRuns/bug-saves/swiss-win-8924-20261102/autosave.legatum \
  playtest-8.9.24 \
  docs/playtests/diffs/swiss-8.9.24-decision-diff.md
```

For the Germany validation game, replace the save with `~/LegatumRuns/bug-saves/germany-8923-20261101/autosave.legatum`, the tag with `playtest-8.9.23`, and the report stem with `germany-2026-11-01-8.9.23-decision-diff`.

The command writes the original one-turn report and JSON, plus `-windows.md/.json` and `-rivals.md/.json`. Only the Markdown reports are kept in the repository; the raw JSON (paired runs and diagnostics, up to 4 MB each) was removed before the public release (2026-11-21) and can be regenerated with these scripts. `patterns.py` needs the JSON reports present. To run only the newer analyses on a game whose one-turn report already exists:

```sh
tools/decision-audit/designer-diff-v2.sh SAVE.legatum playtest-8.9.24 docs/playtests/diffs/GAME-decision-diff
```

The script checks the save's engine commit against the tag. `build_tape.py` extracts the orders into a temporary directory. `decision_audit` replays the tape and verifies each state hash before valuing branches. Every branch uses the recording engine. The original report asks recording-version Hard to plan the designer's seat from its `player_view`, chooses up to 30 turn/category disagreements by an outcome-independent heuristic, and swaps one category on one turn. Purchases and city production are separate categories. Forty-eight paired salts reseed game dice and AI randomness while the map seed and every era's objectives stay fixed. Both arms continue with Hard through the recorded end turn. For unfinished branches, the win-chance column uses the engine's end-position estimate; finished branches use the actual outcome. Raw JSON retains paired outcomes, replay checks and scorer diagnostics. The table orders candidates by absolute win-chance change plus ten times absolute VP change; that is a display order, not an AI score. `legacy_adapter.py` handles old tags lacking the audit example. The runner applies the dice-stream compatibility patch to the recording engine in its temporary worktree; it does not alter the tag or recorded hashes.

**Swiss T140 recheck (2026-11-03).** With objectives fixed and 48 paired salts, buying the Bomber instead of Hard's no-purchase order gains **+2.21 VP [1.64, 2.78]** by T161, compared with +2.38 [1.93, 2.82] in the original report. The estimated end-position win chance rises by 25.1 percentage points [18.7, 31.4] instead of the original −2.1-point actual-win result; neither new arm finished by T161, so both record zero actual wins. The recording-engine replay had **0/160 hash mismatches** and 0/633 AI strategic-order mismatches. This isolated recheck leaves the other rows and the pattern ledger at their original values until those branches are rerun. Raw results: `~/LegatumRuns/diff-reseed-swiss-t140/`.

## Multi-turn events

`--event-windows 5` selects windows from the replay's resolved event log: Settler captures, city captures and the first city attack after a gap of at least two turns. Settler windows swap Settler, army and nearby garrison orders; city captures swap army and garrison; siege starts swap army orders. Each window runs the recorded category orders against live Hard choices for those categories for up to five turns, then both arms continue with Hard. The event selector prioritizes Settler snipes and captures over siege starts and values up to 12 windows per game. A capture near T41, for example, selects T37–T41; a siege window can include preparation before the first hit. The JSON records the event turn, window start, swapped categories and every paired run.

```sh
tools/decision-audit/run.sh --out /tmp/swiss-windows --engine playtest-8.9.24 SAVE.legatum -- \
  --seat 0 --event-windows 5 --salts 48 --end-turn 161
python3 tools/decision-audit/v2_report.py /tmp/swiss-windows --mode window --out /tmp/swiss-windows.md
```

## Rival choices

`--rival --seat N --target-seat 0 --top 8` evaluates each rival separately from its own visible `PlayerView`. Candidates are its top scored alternative city production, legal purchases shown in its city options, positive forecast-driven attacks against visible targets, changed peace or war orders, and safe Settler routes or escorts. The event log prioritizes the safety candidates in the five turns before a recorded Settler loss; it supplies selection, never hidden information to the rival's order. The recorded rival orders are the baseline; one rival candidate is substituted for one turn. The designer follows his recorded orders in both arms afterward, while every rival continues with Hard. This holds his subsequent choices fixed for the opponent question; drifted orders can become invalid, and the report counts those runs. The generator uses the full state only to reject illegal candidate orders. `-rivals.md` ranks the five candidates across all rival seats that most reduced the designer's paired end-position win-chance estimate **and** raised that rival's own estimated win chance on average. It shows rival VP too, since a defensive move can help its chance while sacrificing short-run VP. The JSON retains all candidates, including self-harming alternatives. This searches the generated options, not every legal move or every possible search plan. A zero or negative result does not prove that a rival had no stronger move.

```sh
tools/decision-audit/run.sh --out /tmp/swiss-rival-1 --engine playtest-8.9.24 SAVE.legatum -- \
  --seat 1 --target-seat 0 --rival --top 8 --salts 48 --end-turn 161
python3 tools/decision-audit/v2_report.py /tmp/swiss-rival-1 --mode rival --out /tmp/swiss-rival-1.md
```

## Pattern ledger and decisive turns

`patterns.jsonl` holds stable pattern IDs, category, candidate function or gene, contributing games and turns, summed paired VP delta and its approximate interval, fixture IDs and closure state. [The ledger table](patterns-ledger.md) displays an open/closed column. The function is a diagnostic attribution, not proof that every turn followed the same branch. Rebuild it after adding a game:

```sh
python3 tools/decision-audit/patterns.py
python3 tools/decision-audit/closure.py --engine master --write
```

`closure.py PATTERN_ID` checks one pattern. It checks out the requested revision in a temporary worktree, adds the known-gap fixtures, and runs the ignored sim test on that engine. Its open/closed table means **Hard now chooses the specified move at those recorded views**. Closure is a decision gate; it does not establish a paired win-rate improvement. The decisive turn states were exported by the hash-verified recording engine; the saves and tapes remain outside Git. To run the tests on this branch after an AI change:

```sh
tools/decision-audit/run-decisive.sh                     # all ignored known gaps
tools/decision-audit/run-decisive.sh swiss_t140_bomber    # one choice
```

The fixtures intentionally fail while their gaps remain. They are ignored in ordinary `cargo test`. The pattern's broad interval can be positive while an individual fixture measures a different local mechanism; each fixture manifest records its exact turn and expected order.

## Cost and limits

The original Swiss run was 30 candidates × 2 arms × 48 salts = 2,880 continuations and took **864 seconds of box work (0.24 box hours)**, plus 89 seconds of transfer and queue time. The 12-window cap adds at most 1,152 continuations. Four rivals at eight candidates each add at most 3,072 continuations, for at most 7,104 continuations including the original report. Each continuation reaches at most the remaining turns of one game.

The November 3 validation used 48 salts. Remote wall seconds below come from `~/LegatumRuns/timings/2026-11-03.tsv` and include box execution, transfer and queue; overlapping jobs are added as box time, not elapsed clock time.

| Game | Windows | Rival seats | V2 continuations | Remote wall | Box hours |
|---|---:|---:|---:|---:|---:|
| Swiss 8.9.24 | 12 | 4 | 4,224 | 1,245 s | 0.35 h |
| Germany 8.9.23 | 1 | 4 | 3,168 | 907 s | 0.25 h |
| Both | 13 | 8 | 7,392 | 2,152 s | 0.60 h |

Actual box time varies with game length and load; use `tools/ops/timings-report.py` or the timing ledger for later games. The temporary raw workspace path is printed; commit only reports, ledger and scenario states, never a save or tape.

These are causal comparisons for the swapped order bundles under the stated continuations. Categories interact; a recorded order can misfire after a reseeded branch drifts from the original game; standing orders persist into handover; AI opponent memory is reconstructed from the tape. The normal-approximation 95% intervals describe paired-salt sampling only. They do not cover event or candidate selection, reconstructed memory, engine-version differences or model bias. A nonzero tape hash mismatch stops valuation. Production scorer traces quantify some gaps on newer tags; legacy reports may have no comparable score. A multi-turn category value is not additive with its one-turn components, and a rival alternative that hurts its own VP is a spoiler, not necessarily a better rival policy. Rival results with many invalid recorded designer orders require particular caution. This tooling changes no AI behavior.

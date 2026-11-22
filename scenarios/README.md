# Scenario Lab

Short, playable scenarios for questions only a human can answer: how a person plays a situation,
and how it feels. They are listed on the title screen under **Scenarios** (the Scenario Lab). Each
ends with the player's result next to the Hard AI's from the same start, then a short feedback card
(Fun 1-5, Fair 1-5, "It felt": too easy, about right, too hard or confusing, and free text).

Make one whenever a design question needs a person's play or feel, after every coach review, and when
you are stuck on something the sims can't settle (`legatum-workflow` skill, "Scenario Lab").

**Scenarios are for learning, not permanent fixtures (2026-11-07).** Delete a scenario when it depends on a removed mechanic or faction, when Hard saturates it (45 or more of 50), when it duplicates another's gap, when its AI idea has landed or been dropped, or when it starts after the decisive choice (a lost position). Delete the file, its baseline and every reference in the same change, and list the reason in a dated `docs/status/scenario-sweep-*.md`. The first sweep: [`docs/status/scenario-sweep-2026-11-07.md`](../docs/status/scenario-sweep-2026-11-07.md).

## What is here

| Folder | What | In the app |
| --- | --- | --- |
| `lab/` | Scenarios cut from saves with `tools/scenarios/cut.sh` | yes |
| `war/` | War scenarios cut from playtest games (`tools/scenarios/extract-war-states.sh`, `legatum-sim war-build`), run by the AI in `tools/sim/war-scenarios.sh` | only those with a Lab card |
| (this folder) | The first four skateboard scenarios from 2026-10-03, kept as fixtures for the simulator and engine tests | no |

A scenario file is `legatum-scenario` JSON: a briefing, an objective, a score, the start state and the
Hard AI's baseline. The Lab lists a file only when it has a `lab` card (`question`, a two-line `blurb`,
an optional `ask` for the feedback box, `era` for the painting, `order` for the list position).

## Cut a scenario in one command

```sh
# 1. Find the turn: one line per turn with wars, cities and military units of each empire.
tools/scenarios/cut.sh path/to/SAVE.legatum --inspect --from 5 --to 40

# 2. Cut it, add it to the list and measure the Hard AI from the same start (about 5 minutes on the box).
tools/scenarios/cut.sh path/to/SAVE.legatum \
  --turn 10 --seat Mongols --id mongol-hold-the-line \
  --title "Hold the Line" \
  --question "Can you survive a Spartan rush on a small map?" \
  --blurb "Turn 10. Sparta has a spear line ten tiles from your capital and you have one Archer. Walls are two technologies away." \
  --briefing "Turn 10. You are the Mongols ... (plain language: who, where, what you have, what to do)" \
  --objective hold --turns 20
```

What the options mean:

| Option | Meaning |
| --- | --- |
| `SAVE` | A `.legatum` save recorded on the current engine. It is only read. Older saves need `--allow-mismatch`, and the position may then differ from the original game. |
| `--turn N` | The position before turn N's orders. |
| `--seat K` | The empire the human plays: player number, faction (`Sparta`) or country (`Sparta`). Every other empire becomes a Hard AI. |
| `--id ID` | File name `scenarios/lab/ID.json` (lowercase, digits, dashes). |
| `--title`, `--question`, `--blurb` | The card: a short title, the question in plain words, two plain lines of scene. |
| `--briefing` | The full briefing. Say where things are and what the player has. Write it for the player, with no code words (see the plain-language guide in `.claude/skills/legatum-ui/SKILL.md`). |
| `--objective` | `hold`, `take:City[+City]`, `eliminate:Empire[+Empire]`, `cities:N` or `score`. The objective sentence and the score are written for you. |
| `--turns N` | Turns in the window. |
| `--ask TEXT` | What the feedback box asks when it differs from the question (optional). |
| `--order K`, `--minutes M` | List position and minutes to play (optional). |
| `--rusher EMPIRE --war EMPIRE@TURN ... [--storm N] [--rush-target EMPIRE] --opponent TEXT` | Make an AI empire rush on a script, for scenarios about defending a rush (the Hard AI does not rush well). From the listed turns it declares war, never accepts peace, trains Spearmen, marches its army on the nearest enemy city (or the `--rush-target`'s) and attacks together once N Spearmen (default 3) stand next to it. `--opponent` is the sentence the briefing shows ("Sparta follows a fixed rush plan..."). The Hard AI baseline plays the seat against the same script. |
| `--requires TEXT` | Build and measure the scenario but keep it off the title screen until a change lands (say what it waits for). Delete the `requires` field in the JSON when it does. |
| `--replace` | Overwrite an existing id. The revision goes up so old results are not compared with the new one. |
| `--runs N`, `--no-baseline` | Hard AI baseline runs (default 50), or skip it. |
| `--local` | Run on this Mac instead of the box (slow; only if the box is unavailable). |

The command writes `scenarios/lab/ID.json` and `docs/sim-results/scenarios/ID-baseline.{json,md}`.
Commit both. The scenario appears in the Scenarios screen once the branch is merged to `master` and the site is rebuilt
(scenarios ship by merging; there are no drop-in folders).
Put an existing scenario file on the list (for example one in `scenarios/war/`) and measure its baseline with
`tools/scenarios/cut.sh --card scenarios/war/FILE.json --question ... --blurb ... --order K`.

Save from an older engine? `tools/scenarios/extract-state.sh SAVE --out DIR --turns 150,160` replays it on the
engine it was recorded on (checked turn by turn) and dumps the states and a `summary.jsonl` of wars, cities and
the coalition target per turn; then `tools/scenarios/cut.sh --state DIR/T160.json --seat K ...` cuts from one.
For positions recorded before rules 25, `scenario-cut` restores each empire's old strategic-resource
reveals (Niter, Coal, Oil and Silicon) by granting the corresponding new reveal technology.
If that was the empire's current research, it advances to its next queued technology or the
Hard AI's next choice with its stored Science intact. Existing cards can be repaired with
`legatum-sim scenario-reveal-migrate scenarios/lab/*.json scenarios/war/*.json`; it skips
unchanged cards, clears the changed cards' baselines, and must be followed by fresh Hard runs.
Use `legatum-sim scenario SCENARIO.json --runs 50 --difficulty hard --record` for a Lab card and
`tools/sim/war-scenarios.sh --brain hard --runs 50` for the war suite.

A good scenario has one question, a window of 8-25 turns, a position the player can read in a minute,
an objective a person can fail, and a Hard AI result worth comparing to. Check the baseline: a scenario
the AI always wins or always fails in the first turn is a weak test. Say in the review which scenario
to put in front of a player first.

## Ship it without a release

A scenario is data, so it needs no full test run (2026-11-07). Commit the file to `scenarios/lab/` on its own branch
(below). The web build bundles every file in `scenarios/lab/`, so the scenario is playable as soon as the branch is
merged and the site is redeployed.

### Landing scenario files in the repo

A branch that changes only `scenarios/lab/*.json` (and the docs beside them, such as the baseline in
`docs/sim-results/scenarios/`) skips the extended verify and the docs review. The merge gate runs a fast scenario
check instead (`tools/gates/scenario.py`, a few seconds): the state loads on master's engine (hash, legal position,
objective), its `save_version` is master's `RULES_VERSION`, the baseline `.json` and `.md` exist and the copy lint passes
on the title, briefing, objective and card text. Any other change in the branch brings back the full flow.

## Where runs and feedback are saved

The app keeps each run as one JSON file and downloads it when the run ends (the web build has no folder to write to).

File name: `<id>-<UTC start time>.json` (one file per run; each save replaces the last), format `legatum-scenario-run`. It holds the player's orders for every
turn with the time spent on each (`ms`), the score and outcome, the turn-by-turn report lines that
mattered, the **feedback** (`fun`, `fair`, `felt`, `text`) and `archive`: the engine's full history of the
run (the tape of every empire's orders and state hashes), so the coach can replay it. The feedback is
written into the same file when the player saves it. A run left before the end, or still unfinished when the tab closed, has `abandoned: true`.

Read them with:

```sh
tools/scenarios/runs-report.py                      # every run, grouped by scenario, with the feedback
tools/scenarios/runs-report.py --scenario ID --since 2026-10-18
python3 tools/scenarios/compare.py RUN.json          # decisions against the Hard AI's baseline
```

Quote the player's free text in a review; do not summarise it away.

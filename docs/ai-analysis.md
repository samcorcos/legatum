# Automatic AI behavior analysis

F3 turns a finished game into verified evidence, an Astra review of what worked and what
failed, machine-readable proposals for the evolution coach and lead, and (only for
high-confidence incident findings) concrete pending AI tasks. No game rules or AI orders change.
This is an owner-side tool: archive data is private; running the review sends a compact
excerpt to Codex/Astra. The detector and batch statistics run entirely locally.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"

# Local detector: compressed archive envelope (zstd, gzip, or JSON).
cargo run --release -p legatum-analysis -- ~/LegatumData/archive/2026/09/<game_id>.tape.json.zst
# Same detector through the existing replay command:
cargo run --release -p legatum-sim -- --replay <archive-file> --analyze --out .reviews/ai-analysis

# Replay, review with Astra, write report, enqueue only high-confidence proposals:
tools/astra/analyze-game.sh <archive-file>
# Review for the lead, without queuing work:
tools/astra/analyze-game.sh <archive-file> --no-queue
# Supply an opposite-outcome game for the coach (up to four --compare flags):
tools/astra/analyze-game.sh <winning-archive> --compare <losing-archive> --no-queue
# Detector only; no LLM call or queue writes:
tools/astra/analyze-game.sh <archive-file> --detector-only

# Per-faction behavior and attacker-faction rates added to the batch JSON and Markdown:
cargo run --release -p legatum-sim -- --games 100 --analyze --archive --label ai-baseline
```

The standalone detector accepts `--out <directory>` and `--settler-turns <N>` (default 12).
Its stdout is a JSON manifest giving `game_id`, `incidents`, `narrative`, and `incident_count`.
It uses the archive's actual orders, not today's AI to reconstruct yesterday's choices.
No incident report is published unless the initial hash and **every** turn hash match.
Schema, setup, turn sequence, and rules fingerprint are checked before analysis. An engine
mismatch fails clearly; use the matching engine revision, rather than bypassing the gate.
Legacy saves replay from their stored anchor, retain original turn numbers, and report
`complete_history: false`; earlier history and settler age are never invented.

## Incident definitions and rate denominators

A flag is a behavior worth examining, not an automatic verdict. Each incident identifies an
AI player, faction, resolved turn, unit/city where applicable, consecutive-turn count,
evidence, and local context from the player's **pre-order view**. Context includes HP,
submitted orders or persistent destination, visible nearby units/cities, a known safe step,
and available combat forecasts. Names and free-form archive text are not included in the
LLM narrative. Enemy factions from actual battle events are outcome attribution; the local
context identifies what was visible when the AI decided.

Only seats marked AI and alive at the beginning of a turn are scored. Hostility uses the
engine's relationship rules, not an assumption that every other player is an enemy.
Movement, initiating combat, melee retaliation, a valid movement/ranged/upgrade order, or an
ongoing movement destination count as a response. Attempted responses that fail in resolution
are not called intentional passivity. Their bounce pattern can still be flagged separately.

All thresholds are explicit:

| Behavior key | Flag | Opportunity denominator |
| --- | --- | --- |
| `unanswered_attack` | Unit attacked in 2+ consecutive turns with no response | Attacked AI unit-turns, including the first hit |
| `scout_not_fleeing` | Scout has visible adjacent hostile military, a known passable unoccupied adjacent step beyond enemy adjacency, and no response | Scout-turns with that threat and escape option |
| `garrison_under_ranged_fire` | Military on its own city tile absorbs, or its city receives, ranged attacks in 2+ consecutive turns without response | Garrison unit-turns under ranged fire |
| `idle_military_threatened_city` | Military other than Scout, at least 70 HP, outside a city, remains without response for 2+ turns within 3 hexes of an own city with visible hostile military within 3 hexes | Such healthy nearby military unit-turns |
| `repeated_bounce` | Actual engine `Bounce` events in 2+ consecutive turns for the same unit | Unit-turns containing at least one Bounce (multiple movement rounds count once) |
| `movement_oscillation` | Four successive boundary positions are A-B-A-B, A != B | Surviving AI unit-turns with four observed positions |
| `gold_hoarded_under_attack` | Own city/garrison attacked for 2+ turns, treasury >= 400, affordable legal military/walls purchase, no successful purchase or upgrade | Player-turns with an attacked own city and an affordable defensive purchase |
| `settler_wandering` | Surviving unfounded Settler for more than N observed turns, including stationary turns | Surviving owned settler-turns |
| `research_idle` | Research empty both before and after resolution, positive science income, available technology, no research completed this turn | Player-turns with positive science income and an available technology |
| `production_idle` | Production empty before and after resolution, legal build options, no resistance, no build completed this turn | Own city-turns with build options and no resistance |

Consecutive rules reset on a gap or a response. New or just-completed production/research
is not falsely flagged at the end of its completion turn. Human seats are never accused.
Healing units and stationary garrisons are excluded from the generic idle-military rule.
A fortifying garrison may nevertheless receive a bombardment flag; whether a sortie was
wise belongs to the review. Scouts surrounded by impassable terrain or units do not receive
an escape flag. A distant hidden threat cannot make a nearby unit appear negligent.

`ai_analysis` in batch JSON contains `games`, seat/win counts by faction, and `rates`.
Each rate has `faction`, `behavior`, `attacker_faction` (null for all opponents),
`opportunities`, `incidents`, and `incident_percent` (null when there were no opportunities).
Numerators count incident-bearing entity-turns, not unique bugs; overlapping rules are not
additive. Counts sum before division. Opponent rows may overlap when multiple factions
attack in the same turn. Zero-opportunity rows remain visible. Compare matching seeds,
speeds and difficulties; win counts and incident rates do **not** demonstrate causation.
For example, `garrison_under_ranged_fire` against Celts quantifies unanswered bombardment
exposure, but does not prove why Celts won. `--analyze` adds a verified core replay to each
primary batch game; determinism re-plays are not additional samples. Analysis failures
make the batch fail rather than silently count missing games as zero-incident games.

## What worked and comparisons

Every LLM review includes effective AI behaviors, why they succeeded, confidence, and exact
game/player/turn citations. This includes successful decisions by losing seats. Neither winning
nor the absence of incident flags proves good play. If the evidence supports no success, the
review must explicitly explain the limitation instead of inventing praise.

Detector v2 adds per-AI-player outcomes and city, population, technology, VP, gold and survival
snapshots to every resolved turn, including turns without incidents. Founding, captures, completed
production and research, purchases, battles, losses and scoring events are attributed to players.
These facts are reconstructed during hash-verified replay; unverified stored telemetry is not
used to substantiate successes. Names and free-form game text remain excluded. Outcome facts are
not automatic judgments of decision quality or proof that hidden information was visible.

The model receives up to 12 KB of these timeline frames per game, sampling early, middle and
late play with explicit omitted-turn counts and paths to the full verified reports. Up to four
comparison games accompany the reviewed game. Matching cached detector-v2 reports are reused;
otherwise, for archives under an index.jsonl, the wrapper shortlists opposite outcomes from the
most recent 1,000 index entries and verifies up to four companion archives. Explicit --compare
archives are always replayed and must provide a matching opposite-outcome AI seat. A failed
automatic companion replay is disclosed as unavailable; it is never counted as a loss.

Win/loss pairs must come from different games and match archived engine commit, rules fingerprint,
faction, difficulty, speed and map dimensions. Unknown revisions, human seats, incomplete histories,
draws and unfinished games cannot form pairs. Each comparison cites observed turns for both seats,
explains behavior differences, and discusses confounders such as seeds, opponents and luck.
An available pair must be discussed; no available pair is reported as a limitation.

The archive currently records an engine commit, not a distinct brain-policy or tuning-parameter
identity. Therefore these are **revision-level comparisons**, not proof that mixed, Proxy, biased
or tuned seats ran identical policies. The report and proposal envelope disclose this basis.
The evolution coach must verify its own candidate/policy/parameter identity before using a
comparison to evaluate a candidate. Today's checkout revision is recorded separately and is
never substituted for the archived version.

## Machine-readable proposals

Each successful LLM review also writes .reviews/ai-proposals/YYYY-MM-DD-GAME_ID.json, using
the UTC review date. The markdown links to this file. The checked-in
[JSON Schema](../tools/astra/ai-proposals.schema.json) is the interchange contract for the
ai-evolution coach loop and the lead; schema_version is currently 1. The envelope records the
reviewed game, timestamp, reviewed code/source digest, comparison basis, and archived engine
commit/rules fingerprint for every evidence game.

Each proposal has exactly these fields:

~~~json
{
  "kind": "tunable_parameter",
  "description": "Expose defensive reserve turns (1-4), trading early spending for readiness.",
  "evidence": [{"game_id": "example-game", "player": 0, "turns": [42, 43]}],
  "expected_effect": "Preserve productive research while buying emergency defenders earlier.",
  "confidence": "medium"
}
~~~

The example illustrates the contract; it is not a finding about an actual game. Kinds are:

| kind | Intended use |
| --- | --- |
| new_behavior | Code changes in crates/ai, including improvements that preserve demonstrated strengths |
| tunable_parameter | New parameters worth exposing to the evolution harness, with a proposed range and tradeoff |
| exploit | Exploits or degenerate strategies to guard against, with a proposed guard and regression |

Evidence must name a verified context game, an AI player and nonempty, unique, observed turn
numbers. Confidence is high, medium or low. Descriptions and expected effects must be nonempty.
Unknown fields/kinds, unsupported schema versions, invented games/players/turns, and malformed
values fail local validation before any report, proposals or task is published. An empty
proposals list is valid when nothing actionable is supported; categories are not quotas.
Existing incident-backed findings are automatically exported as new_behavior proposals too.

The new proposal list goes to the coach and lead at every confidence level. It does not
automatically change code, add harness parameters or adopt a strategy. The existing task queue
still promotes only high-confidence incident-backed findings with checked code causes and
regression tests. Consumers should reproduce claims and use matched A/B and held-out evaluation
before adoption.

The success receipt is the publication marker: it names the proposal file and its SHA-256.
Consumers should follow the latest .reviews/ai-analysis-GAME_ID.receipt.json rather than treating
every dated file as a new experiment. Both artifacts are written before queue promotion, and
the receipt is written last. Missing or modified proposals invalidate cached completion, including
in the watcher. Pipeline v2 reruns older reviews so the additions are not silently skipped.
Direct reruns also refresh when the selected comparison evidence or reviewed AI/core sources
change. A later dated forced review can leave an older proposal file; the receipt identifies
the current one. --no-queue still produces both artifacts; --detector-only produces neither LLM
artifact. No live model call is required by the tests.

## Reports, review, and task promotion

The local detector writes `<game_id>.incidents.json` (complete incidents, rates, per-turn
summaries) and `<game_id>.narrative.md`. The narrative contains key event counts and up to
two local incident examples per turn, bounded to 64 KB with explicit omission counts.
Astra receives up to 48 KB of narrative plus 28 KB of JSON examples, selected round-robin
by behavior and player so long idle episodes cannot crowd out a rare combat problem.
The full verified files remain available for follow-up inspection.

`tools/astra/ai_analysis.py` calls `codex exec -m gpt-6-astra -s read-only` with a JSON
output schema. This uses the installed CLI's existing authentication, like `runner.sh`.
The [official structured-evaluation example](https://developers.openai.com/blog/eval-skills)
describes `--output-schema`; local validation independently checks the returned evidence.
The model inspects `crates/ai`, distinguishes current code from the archived engine revision,
and ranks findings with likely file/line causes, proposals and regression tests.

Final reports are `.reviews/ai-analysis-<game_id>.md`; raw review JSON, detector output,
and Codex logs are under `.reviews/ai-analysis/`. A success receipt records the input digest,
detector version, rules fingerprint, reviewed code revision and AI/core source digest.
Source changes during the model call reject the review before publication or task promotion;
the watcher retries against stable code. `--force` repeats a review.
A `--no-queue` review can subsequently be promoted by rerunning without that option.
The model itself runs read-only. The wrapper, after validating all findings and publishing
the report, writes pending tasks only for findings explicitly marked `high` confidence.
They must cite existing incident IDs of the stated behavior and valid `crates/ai/*.rs` lines,
a rationale, concrete fix, and a regression test. Confidence is an LLM assessment, not a
statistical confidence interval or proof that the proposed fix is correct.

Medium/low confidence, rational defense possibilities, engine bugs, and already-fixed
behaviors stay with the lead. Constructed passive-order tapes must not become high-confidence
claims about the current AI without independently checking that it makes the same decision.
Tasks tell the worker to reproduce before editing, obey pillar 6, check for overlapping AI
work, run checks, and leave its branch for the lead. No branch is automatically merged.
Task identities use behavior, stable cause key and source paths. A lock and an atomic file
rename deduplicate across games and all pending/running/done stages. Model failures or
invalid responses produce no success receipt and no new tasks; rerunning safely retries.
The review process has a 30-minute timeout. `LEGATUM_CODEX`, `LEGATUM_ANALYZER` and
`LEGATUM_SIM` accept executable paths for testing or pinned builds.

## Automatic processing

```sh
# Start with the other Astra services, from the integration checkout after review:
tools/astra/runner.sh analysis
# Equivalent standalone daemon:
tools/astra/watch-games.sh
# One polling pass (also suitable for a scheduler):
tools/astra/watch-games.sh --once
# Human archives only:
tools/astra/watch-games.sh --once --no-daily
```

The watcher polls every 30 seconds (`--interval`), catches up finished human games from
`index.jsonl`, and only reads final immutable archive files. This covers native local,
online-hosted human seats through the existing archive writer; checkpoints are never
reviewed. It respects `--archive-dir` / `LEGATUM_ARCHIVE` / `~/LegatumData/archive`, follows
the writer's file lock, and retries a torn last index line on the next pass. No server turn
waits for a model. Browser-only games enter this flow after delivery to the native archive.

Once per UTC day, select 20 finished non-panic sim archives matching the compiled rules
fingerprint (queried through `legatum-analyze --engine-info`) from the most recent 1,000,
ordered by a deterministic hash of date plus game ID. Exclude games the watcher already
successfully reviewed on previous days. If fewer remain, run a seeded batch
to fill the sample using `--archive --analyze`. Freeze the selected IDs in
`.progress/ai-analysis/daily-YYYY-MM-DD.json` before reviews so a restart resumes the same
sample. Completed reviews for that frozen sample are reused on restart. This is monitoring, not a statistically
balanced faction benchmark; use seeded matched batches for before/after evaluation.

Watcher receipts, failures, and next retry times live in `.progress/ai-analysis/watcher.json`.
Failed games retry with backoff up to one hour; one failure does not discard later games.
An OS lock prevents two watchers in the same checkout. Nothing is silently marked done for
incompatible rules, corrupt tapes, or failed Codex calls. Keep a matching engine for historical
archives (see [game archive](game-archive.md)); the watcher does not rewrite archives.
Start the daemon from the chosen integration checkout when enabling the service. Adding this
branch does not install a login service or alter the user's machine-wide scheduler.

## Reproducing the user's report

```sh
cargo run -p legatum-analysis -- --write-repro /tmp/ranger-passivity.tape.json --out /tmp/ranger-analysis
cargo run -p legatum-sim -- --replay /tmp/ranger-passivity.tape.json --analyze --out /tmp/ranger-analysis
tools/astra/analyze-game.sh /tmp/ranger-passivity.tape.json --no-queue
```

`crates/analysis/src/scenario.rs` constructs an anchored grassland scenario and records
four real resolved turns: two enemy Rangers bombard a soldier and a city-garrison soldier,
while an AI Scout holds beside enemy military with a visible escape tile. The soldiers hold
until killed. It creates `ranger-passivity-repro` with the currently compiled engine's
fingerprint and turn hashes. This proves detector sensitivity; it deliberately supplies
passive orders and does not claim today's AI still chooses them. Keeping the generator
avoids checking in a tape that every unrelated core source edit makes incompatible.

Rust tests replay small constructed tapes for every rule, plus response/escape/ally/healing
and completion exclusions, resumed anchors, deterministic output, aggregation and corruption
rejection. CLI tests verify compressed input and no report overwrite on divergence. The sim
integration test compares `--analyze` output with one and two worker threads. Python tests
exercise confidence validation, queue deduplication, failures/retries, daily sampling,
bounded evidence selection, and the review-to-task flow with a controlled model stand-in.

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 -m unittest discover -s tools/astra -p 'test_*.py'
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 -m unittest discover -s tools/analysis -p 'test_*.py'
```

## Validation snapshot (2026-09-27)

A 20-game Quick smoke run at seeds 270900–270919, two workers, `--archive --analyze`,
finished in 113.5 seconds after compilation: 20 VP victories, zero panics/stalls, 20 unique
final archive entries, and 20 hash-verified analyses. It produced 14 unanswered-attack
incidents / 7,702 attacked unit-turns, 2 passive-garrison incidents / 409 bombardment
unit-turns, and 0 scout non-escape incidents / 671 eligible scout-turns. This small sample
checks the pipeline, not balance or the validity of every heuristic flag.

The four-turn reproduction produced 6 unanswered-attack incidents / 8 attacked unit-turns,
3 passive-garrison incidents / 4 bombardment turns, and 4 scout non-escape incidents / 4
opportunities; both soldiers died on turn 4. The standalone detector and simulator replay
entrypoint emitted identical JSON.

The live Codex smoke invocation stopped before model execution because this restricted
Codex environment denied app-server initialization (`Operation not permitted`). No success
receipt or task was created. The structured-review and queue path is covered by the Python
stand-in tests; a successful live Astra call remains to be verified from the host checkout.

Final branch checks: `cargo fmt --all` passed; `cargo clippy --workspace --all-targets -- -D warnings`
passed; `cargo test --workspace --config 'profile.test.package.legatum-core.opt-level=2'`
passed all 36 suites (545 tests passed, 0 failed, 4 existing tests ignored). The ordinary
unoptimized workspace run was interrupted during exhaustive map generation; the full rerun
optimized only the core package's test build, retaining test-profile debug assertions and
all test cases. Python checks passed: 10 Astra orchestration tests, 5 script tests and 3
analysis-tool tests. Bash syntax checks passed for all three Astra entrypoint scripts.

## Validation snapshot (2026-09-28)

After merging master at 0c7c032, cargo fmt --all and
cargo clippy --workspace --all-targets -- -D warnings passed. The unmodified
cargo test --workspace command passed all 46 suites: 698 tests passed, 0 failed,
4 existing tests ignored, using the ordinary unoptimized test profile.

Python checks passed: 19 Astra orchestration/schema tests, 5 script tests and 3 analysis-tool
tests. Coverage includes all three proposal kinds, invalid schemas and invented citations,
matched win/loss comparisons and rejected mismatches, incident-free successes, companion
replay failures, cache invalidation and missing/corrupt proposal recovery. The Rust analysis
tests verify that successful outcome evidence comes from replayed state rather than stored
telemetry. Bash syntax checks passed for all three Astra entrypoint scripts.

A smoke run with the actual compiled Rust detector and a controlled model stand-in produced
the markdown, schema-valid proposal file and matching success receipt with queuing disabled.
No live LLM call was made in this validation; assessment quality and live structured-output
acceptance remain unverified.

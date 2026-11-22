# Learnings (digest)

This file is the short list of lessons that are still true and still change what an agent should do. Read it at session start and after compaction. The full record, every entry verbatim with its evidence, lives in the monthly archive under `docs/learnings/` (`2026-09.md`, `2026-10.md`, `2026-11.md`, ...).

**How to add a learning.** Append a dated entry (`## YYYY-MM-DD: title`, then bullets: cause, evidence, what changed) at the top of the current month's archive file, `docs/learnings/<YYYY-MM>.md`; start the file if the month is new. Promote it here, as one or two lines with the archive month in brackets, only if it is durable: a rule an agent must follow from now on. Correct or delete digest lines that stop being true; the archive keeps the history. Lessons about removed systems (the Tauri desktop app, the Mac app, the LAN server, iOS, Quick speed, crowns, decadence, the TS mock engine, the Codex lanes on the Mac) stay in the archive only.

To find the full entry for a line, search its month file for the words in the line.

## Engine and determinism

- Integer math only, and never cast a `u64` seed or hash to `usize` (32-bit on wasm32, so native and WASM diverge). Apply a percentage to the total, not the delta (`total * 75 / 100`), or small values escape it. [2026-09]
- On wasm32 `catch_unwind` catches nothing: a panic reaches JS as `RuntimeError: unreachable`; the UI recovers by rebuilding the `Game` from its last save. Treat save JSON as opaque in JS (u64 bitset words lose precision if re-parsed). [2026-09]
- Saves and tapes from another `RULES_VERSION` are refused; there are no migrations (designer's "no grandfathering", 2026-10-31). Any change to the save shape or to resolution bumps `RULES_VERSION` (except optional, default-empty client telemetry that no hash reads, such as `turn_times` and `frame_times`) and moves every pinned hash and embedded scenario state. [2026-10]
- Parallel branches silently pick the same `RULES_VERSION`; git merges the shared number with no conflict. Re-check the number against master after every merge, and re-encode scenarios cut on a branch when master bumped first. [2026-11, 2026-11 "scenario cut on a branch"]
- New state fields must skip serializing their default (`skip_serializing_if`), or every archived fixture re-serializes and archive-hash pins fail. Removing or renaming an enum variant breaks every reader of old records, including the archive index. [2026-10, 2026-11]
- `GameState::hash` covers player names; anonymizing a recording needs a replay (`Recording::anonymized`). [2026-11]
- Every new `Order` variant needs a tape round-trip test in `crates/wasm/src/tape.rs`; a replay that reports hash mismatches is a bug, not noise. [2026-11]
- Planning-time validation must apply the same phase order as resolution (surprise war in phase 0 makes a same-turn shot legal). A phase-1 order must not change anything the same turn's combat forecasts read. [2026-10, 2026-09]
- A public fact that feeds passage or legality must be set in both `PlayerView` constructors (`player_view_cached` and `order_check_view`). [2026-10]
- The AI reads some human-facing strings (cost labels, blocker text); do not drop labels from AI views until those reads use structured fields. [2026-10]
- Two cargo processes sharing one `target/` produce E0460 and stale builds; give watchers their own target dir. Never share one `CARGO_TARGET_DIR` between git worktrees, even serially: cargo judges freshness by mtime, so a worktree with older files runs another branch's binaries ("Finished in 0.05s", no "Compiling" lines); serial CI uses one worktree and `git checkout --detach` per branch. A fresh container can corrupt the pinned toolchain if two cargo commands race its first install (`rustup toolchain uninstall <ver>`, then one cargo command). [2026-09, 2026-11]
- Egypt's Nile applies to city totals in `economy::nile`; per-tile numbers stay raw because the AI reads them, so the app wraps display yields (`resources.ts::nileYields`). [2026-11]

## Rules and the rules reference

- The rules reference drifts (about 18% of sampled claims were stale after one busy week). Cite `file::function`, never line numbers, and run a sampled claim audit after a train with many data changes. [2026-11]
- Agents paraphrase code into rule text ("+1 Gold per complete pair of population"). Write the rule as a player would say it; the plain-language reviewer checks player text. [2026-10]
- Never present a rule or preference from a memo or brief as the designer's own; quote him or say whose idea it is. A review brief with a false premise (an old skill copy in context) produces a confident wrong verdict: brief reviewers from current master. [2026-11]
- "The AI cheated" reports have twice been resolution-order gaps (a shot on a city that fell this turn, a charge whose target died first). Check phase order before suspecting the AI. [2026-11]
- A rule that needs a hidden number must not publish it; private rewards and reveals leak through side channels (events, previews, counts). [2026-10, 2026-09]
- A tech with nothing for a faction starts known (rules 49); `factions.rs::no_faction_ever_researches_a_tech_that_gives_it_nothing` pins the skip list. Prefer giving the tech something real. [2026-11]
- `start-settlers=N` means N extra Settlers. [2026-10]
- Rules 50 (2026-11-22): World Ages is the only era scoring, a rival's original capital is 1 VP only while held (the Celtic sack keeps its point), Alliance upkeep is waived at War with a near-winner. Grep player text and the Guide for the retired "era objectives" wording when touching scoring. [2026-11]
- Before building a "new" idea, search the archive and the code: the designer's training-mode idea had been built a month earlier (`crates/ai/src/search/tactics.rs`). [2026-11]

## AI

- AI changes interact: gate each gene alone and in the planned combination, in landing order, on binaries built from the commit it will land on. Pre-merge gains often vanish after merging; trust post-merge numbers and record both. [2026-10, 2026-11]
- A gain carried by one old position must show its mechanism on a second position before it counts; fixes validated on one replay have failed on the next map. [2026-11, 2026-10]
- Turning a faction gene on by default breaks tests that use that faction as a generic fixture; grep the tests for the faction and pin plain-path tests to the gene off. A params file cannot switch a built-in faction gene off through `decisions`. [2026-11]
- An agent's "full suite passed" is not verify: run the repo's verify on the merged branch. Codex/agent reports can misstate defaults; check `params.rs` defaults and pin moves. [2026-11, 2026-10]
- Captures per game is not an AI quality metric; a better win predictor did not make a better player; search cannot defend against an opponent it does not model. [2026-10]
- Different thresholds for starting and ending a relationship (war, Peace, Alliance) make AIs oscillate; use hysteresis on both sides. An AI that sues for Peace and attacks again is usually a candidate-list gap. [2026-09, 2026-10]
- An AI-only ablation can call a faction drawback "backwards" when the AI overspends; read it against human play. [2026-11]
- Describe what the AI actually does from a recorded tape, not from its configured priority list. [2026-10]
- A gate keyed on a state the action itself changes closes mid-action: Rome's one-city war gate keyed on "no veteran" shut as soon as the siege promoted its units, and the army walked home. Key gates on the campaign, not on its side effects. [2026-11]

## Simulations and measurement

- The census and nightly `win_share_by_faction` is wins per game, not per seat (each faction sits in about 8/9 of games); convert before comparing factions. [2026-11]
- A combined multi-map gate can hide large opposite per-map effects; read the per-map table (`HETEROGENEOUS` flag in `tools/sim/hard-vs-human-compare.py`) before calling a change noise. 128-game screens overstate levers. [2026-10, 2026-09]
- Short scenarios paired by dice seed halve the noise of a gene A/B (`tools/sim/scenario-gate.sh`); use pairing for any change measured on short positions. The hill-climb suite (`tools/hill-climb/`) is the designer-seat gate. [2026-11]
- Compare against current master, not the base the branch started from. A rule or opponent landing moves the benchmark without any seat-AI change. [2026-09, 2026-10]
- A benchmark opponent must first reproduce the result it stands in for. [2026-09]
- Cloud-container timings vary about 6% run to run; compare base and head built side by side, interleaved, best of three (`tools/perf/lean_check.py`). [2026-11]
- Sim JSON reports are committed only as `.md` plus `gzip -9` JSON; raw `docs/sim-results/*.json` is gitignored. Run logs have leaked account details into git: scrub before committing. [2026-09, 2026-11]
- Measure faction identity per faction, never in aggregate (pillar 8 is about the best and worst faction). [2026-10]
- Winner's curse: the best value on sweep seeds, or a sequential test stopped at a boundary, overstates the effect; confirm on fresh seeds before landing. Judge on end-of-game VP share, VP and wins: VP share at T100 inflates tiny edges because seats hold about 1.2 VP then. [2026-11]

## UI

- Tooltips: hover is for reading, click opens breakdowns; a hoverable bubble must never cover its trigger. [2026-09]
- Never close a popup on a blur whose `relatedTarget` is null (Safari never focuses a clicked button); close on Escape, outside pointerdown, or blur to a known outside target. [2026-10]
- "Possible" is not "recommended": an offer or primary action must be the right move now, not merely legal. [2026-11]
- Never build rule text by regex-stripping prose (punctuation fragments ship); a changed default or a faction rule leaves stale copy elsewhere: grep for the old name and number. [2026-10, 2026-11]
- Rewrites silently drop established interactions (shortcuts, click targets): list them before rewriting and pin them with tests. [2026-10]
- Do not override a stated user preference for a general guideline (e.g. a colour-blind palette replacing his chosen colour); raise it instead. [2026-10]
- Copy has a pixel budget: check every faction and screen at 1000x600; a display-name rename or art that changes a card's tile count needs the screen tour, not just Vitest. [2026-10, 2026-11]
- Splitting JSX text runs moves glyphs; amounts need the sans, not the display italic. [2026-09, 2026-10]
- A solo lobby defaults to 6 empires, so e2e specs that start from defaults get that map. [2026-11]
- A recolour passes the distinctness score and is still a placeholder (see `legatum-art`). [2026-11]
- Art goes stale the way docs do: a rename, a new kind or new art wiring can leave a picture wrong or missing, and the lookalike check (`tools/art/distinctness.py`) judges neither style nor staleness. `legatum-art-reviewer` and `tools/art/art-review.py impact` cover both. Map-graded art must not be reused on cards. [2026-11]

## Web build, tests and deploy

- The game is web-only (2026-11-22): the engine runs as WASM in the browser, served from Cloudflare Pages; playtests are Pages previews. Native speedups do not reach players. [2026-10, 2026-11]
- The hosted page's CSP allows `connect-src 'self'` only; any off-site endpoint goes in `app/hosted/_headers` (`scripts/build-web.sh` adds the games upload Worker when `LEGATUM_GAMES_URL` is set). Games upload to R2 through `infra/games`. [2026-11]
- The web build must work with no backend: `npm run e2e:hosted` plays `app/dist` from static files. Some browsers expose no `indexedDB` at all; guard every storage call. [2026-11, 2026-09]
- `scripts/build-wasm.sh` keys its cache on everything under `crates/`; a stale `app/src/wasm-pkg` fails WASM parity tests, so rebuild it after engine changes. [2026-10]
- Run Playwright from `app/`, not with `--prefix`; any Playwright run wipes `app/test-results/` (including screen-tour shots); JSON imported by modules that e2e specs load needs `with { type: 'json' }`. [2026-09, 2026-10]
- Vitest once matched only `.test.ts`; check that a new test file pattern actually runs. [2026-09]
- Autosave keeps rolling slots per game in IndexedDB with the old localStorage slot as fallback; a failed write must show a notice, never fail silently. A render error, a trapped engine or a failed chunk shows the crash card, never a blank page. [2026-11]
- Fake timers freeze `performance.now()`, so a time-capped loop never ends under `vi.useFakeTimers()`, and `runAllTimersAsync` never ends while an interval runs: use real timers or advance a fixed time. [2026-11]
- `npm run e2e:hosted` is not in `tools/ci/local-ci.sh`; run it by hand after changing the title screen or online flows. [2026-11]
- Online co-op runs the host in the host's browser tab (WASM) through the Cloudflare relay. Browsers cannot send WebSocket pings or see why an upgrade failed, and background tabs throttle timers to about once a minute: liveness uses app-level ping frames and generous timeouts. Relay limits and the `LOBBY_OPEN=0` kill switch live in the Worker. [2026-11]
- The public tree is exported; an exporter must be checked by building and testing its output, not by scanning it. Scrub lists hold generic patterns only: never write a value to hide in a tracked file, test, comment or commit message. [2026-11, 2026-10]
- Public copy (README counts of peoples, eras, techs) is checked by `tools/gates/public-claims.py`; universal claims ("every number", "never cheats") need a stated scope. [2026-11]

## CI and gates

- Judge a run by its exit code, never by the tail of its output; check a status file's mtime is newer than the run's start. A verify that never started has read as green, and a release check once exited 0 with a failed step. [2026-09, 2026-10, 2026-11]
- Never "fix the expectation and merge" on a red verify; re-run verify after the fix. Never land on red because the machine was loaded. [2026-11, 2026-10]
- Clean git merges can still break the build: compile and test before committing a merge. Resolving a conflict by taking one side drops the other side's additions (attributes, fields). Branches have carried committed conflict markers: grep for them. [2026-09]
- Long chained verification in one background shell gets killed (exit 144); split the bar into separate steps. [2026-09]
- Reviewer verdicts are PR comments that name the reviewer and the head commit SHA; GitHub merges never fired the old local marker hook (removed 2026-11-22). `tools/gates/check.py <branch>` names the reviewers the changed paths need. [2026-11]

## Process

- In a shared checkout, commit with explicit paths, never `-a` or `add -A`; agents work in their own worktree. `git stash` is shared across worktrees: use a WIP commit instead. [2026-10]
- Never `pkill -f` or `killall` shared patterns (a hook blocks them); kill by PID. [2026-09, 2026-10]
- Sub-agents outlive the agent that spawned them, and a finished agent can wake with stale advice; check what is running before re-spawning or acting on a late report. [2026-10, 2026-11]
- Never append to a queued task file that may already have started; write a new task. [2026-09]
- Decisions from conversation slip through: record a decision or preference in the matching skill or the design doc in the same turn. [2026-10]
- A check buried in a long checklist with heavier duties gets dropped first; give recurring checks their own owner (`legatum-perf-supervisor`). [2026-11]
- A release note written by an agent can describe cut rules as new features; check "what's new" against the rules diff. [2026-11]
- A hand-maintained progress plan goes stale; derive progress from the living queue or the PR list. [2026-09]

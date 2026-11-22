> Archived 2026-11-22: kept for history; it no longer describes the project.

# Review plan prompts, 2026-10-18

Copies of the Codex queue prompts for `docs/status/review-plan-2026-10-18.md`, so the work is not lost if `.progress/astra-queue/` is cleaned. To requeue one: write it to `.progress/astra-queue/pending/<name>.md` with the header `mode: write`, `name:`, `model:`, `effort:`, a blank line, then the prompt.

## CQ1 · review-vitest-vite-bump · Codex Luna · effort medium · pending

**Bump Vitest and Vite past their published advisories**

Security fix in `app/`. The locked Vitest 5.0.2 is inside advisories published 2026-10-16 (file access and code injection through its API server; fixed in 5.0.5) and Vite 8.3.1 is inside an advisory fixed in 8.3.3. Both are dev dependencies that run on contributors' machines.

Do:
1. In `app/package.json` raise `vitest` to `^5.0.5` and `vite` to `^8.3.4`. Regenerate `app/package-lock.json` (`npm install --package-lock-only`, then `npm ci` to confirm). Keep any `@vitest/*` packages in step with vitest.
2. Leave `app/vite.config.ts` as it is (default SPA setup).
3. Verify: `cd app && npx vitest run` (all tests), `npx tsc --noEmit`, `npm run build`, `npm audit` (no vitest or vite finding), and run two short Playwright specs (for example `npx playwright test e2e/exit-to-title.spec.ts`) to prove the dev server still works.
4. If a test breaks because of a Vitest 5.0.x behaviour change, fix the test minimally and say what changed.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ1 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ2 · review-evolve-gene-parity · Codex Luna · effort medium · pending

**Evolution harness: add the 46 missing decision-gene ranges and a parity test**

`tools/evolve/genome.py` has a hand-written `DECISION_RANGES` allowlist with 68 entries, but `crates/ai/src/params.rs` (the `decision_params!` macro) now declares 114 decision genes. `builtin_catalog()` raises "Rust/Python decision schema mismatch" for any gene missing from the allowlist, so a new evolution run cannot start. 46 genes are missing (for example `settler_safety`, `crown_income_percent`, `decadence_weight`, `troop_runway`, `siege_window`, `scout_find`, `learned_macro`, and the `sparta_*` family).

Do:
1. Add every missing gene to `DECISION_RANGES` with exactly the `min, max` written in `params.rs`. Do not edit `params.rs`.
2. Add a test in `tools/evolve/test_evolve.py` that parses the `decision_params! { ... }` block of `crates/ai/src/params.rs` (lines of the form `name: default, min, max, "description";`) and asserts that the set of names and every (min, max) pair equals `DECISION_RANGES`. It must read the file at test time and not hard-code a count, so any future gene added in Rust fails this test until the allowlist is updated. Print the missing names in the failure message.
3. Run the evolve tests. Do not build or run the simulator.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ2 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ3 · review-verify-lock-recovery · Codex Sol · effort medium · pending

**Verifier: a killed run must not leave a cache lock that stalls later runs for hours**

Problem: in `tools/astra/verify-branch.sh` (the `run_step` cache-lock section, around lines 255-310) a step takes a lock by `mkdir "$cache_lock"` and writes its PID to `$cache_lock/pid` (the script's own `$$`). The lock is removed only when the step returns normally. If the verifier is killed (a box job killed, a dropped terminal), the lock stays, and every later verifier of the same code tree waits up to four hours (see docs/learnings.md around the 2026-10-18 entries on a hung box job and the release check killed in the queue).

Do:
1. Cleanup on exit: once a step holds a lock, record it in a list; an EXIT/INT/TERM/HUP trap removes the pid file and the lock directory for each lock this process holds. Chain with any trap the script already installs. Never remove a lock whose pid file names a different live process.
2. Stale-lock recovery in the wait loop: write `<pid> <hostname>` into the pid file. When `mkdir` fails, read it. If the host matches this machine and `kill -0 <pid>` fails (owner dead), or the lock directory has had no pid file for more than 30 seconds, remove the pid file and directory and retry the `mkdir`. If the host differs or the owner is alive, keep waiting as today. Keep the four-hour timeout.
3. Tests: extend `tools/astra/test-verify-cache.sh` (use the existing `VERIFY_STEP_STUB` mode) with: a lock owned by a dead PID is reclaimed in under 10 seconds; a lock owned by a live process is not touched; sending SIGTERM to a verifier that holds a lock leaves no lock behind.
4. Do not change what a step runs or the cache key. Do not run a real verify.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ3 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ4 · review-air-order-fixes · Codex Sol · effort medium · pending

**Air rules: an invalid Fighter strike must not use up its interception, and a duplicate rebase must not reserve a hangar slot**

Two bugs in `crates/core/src/rules/air.rs`:

1. `resolve_ranged_phase` (around lines 215-275): `used_fighters` is built from every ordered Fighter strike before the strikes are validated, so a Fighter whose strike order is invalid (out of range, no target) is marked as having used its order and can no longer intercept. Build `used_fighters` only from strikes that pass `validate_strike` (keep the first-order-per-plane rule, and keep Fighters that successfully intercepted marked as used).
2. `validate_orders` (around lines 95-130) calls `validate_rebase`, which increments the `rebased_into` hangar reservation, before it checks whether the same aircraft already has an order this turn. A duplicate rebase order for one plane therefore reserves a slot it never uses (resolution keeps only the first order per plane), so a third plane's legal rebase into that hangar can be rejected. Check the duplicate first and reserve the slot only when the order is accepted.

Add tests (find the existing air tests under `crates/core/tests` or the `air` module and follow their style):
- an enemy Bomber strike with a defending Fighter that was ordered to strike out of range the same turn: the Fighter still intercepts;
- one valid rebase A to X, a duplicate rebase for plane A to X, and an independent rebase of plane B to X where X has room for exactly one more plane: `validate_orders` accepts A's first order and B's order, rejects the duplicate, and `resolve_turn` ends with the same planes in X.

Do NOT change whether a strike is legal on the turn war is declared; that is a separate item (CH2). Run `cargo test -p legatum-core` and then the workspace tests. If any pinned hash or tape changes, stop and report exactly which one and why instead of repinning.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ4 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ5 · review-mock-fallback-guard · Codex Luna · effort high · pending

**Mock retirement stage 1: a missing WASM package is an error, not a silent switch to the TypeScript engine**

Part of retiring the former TypeScript client engine (one-engine rule). Today `createClient` in `app/src/client.ts` silently falls back to the former TypeScript client when the WASM package is missing, so a broken or missing build starts a different game without any sign of failure.

Do:
1. Change `createClient(kind)`: when `kind` is null and `wasmAvailable()` is false, throw an `Error` whose message says what is wrong and how to fix it (for example: "The game engine (WASM) is missing from this build. Run scripts/build-wasm.sh, or open the app with retired client selector for the development fixture."). `kind === 'mock'` (from the former client selector) still returns the former TypeScript client. `kind === 'wasm'` is unchanged.
2. Find every caller (start with `grep -rn "createClient" app/src app/e2e`) and make a rejected `createClient` show that message on a simple full-screen error state (no new design; reuse an existing error or toast component) instead of a blank screen.
3. Existing vitest tests or e2e fixtures that relied on the implicit fallback must request the mock explicitly (`createClient('mock')` or the former client selector). Do not delete the mock or change any save format in this task.
4. Add tests in `app/src/client.test.ts` (create it if missing, follow `requestedClientKind` tests if any) for the three cases above, stubbing `wasmAvailable`.
5. Run the whole vitest suite and `npx tsc --noEmit`.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ5 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- Any text a player reads follows the plain-language guide in `.claude/skills/legatum-ui/SKILL.md` (Game text: plain language, with its glossary), and `npx vitest run src/ui/textArtifacts.test.tsx src/ui/copySource.test.ts` must pass.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CQ6 · review-land-ff-script · Codex Luna · effort medium · pending

**A safe landing command that fast-forwards master from the main checkout**

On 2026-10-06 and again later, moving `refs/heads/master` with `git update-ref` from another worktree left the main checkout's index and files stale, so the next commit there reverted the landing (docs/learnings.md, search "update-ref"). The rule is `git merge --ff-only` run in the main checkout, but no tool enforces it.

Write `tools/ops/land.sh <branch> [--dry-run]` (bash 3.2 compatible: no associative arrays, no `source <(...)`):
1. Find the main checkout (the first entry of `git worktree list --porcelain`) and run everything with `git -C` there, whatever the caller's directory is.
2. Refuse unless the main checkout has `master` checked out, the branch exists, and master is an ancestor of the branch (otherwise print "merge master into <branch> first").
3. Do not require a clean tree (the main checkout often has an unrelated modified file such as `app/src-tauri/Cargo.lock`); `git merge --ff-only` already refuses when the merge would touch a dirty path. Let it refuse and print its message.
4. After merging, verify: `HEAD` equals the branch tip, and `git diff --quiet HEAD -- <paths changed by the merge>` and `git diff --cached --quiet` for those paths. Exit non-zero with a clear message otherwise.
5. Never call `update-ref`, `reset`, `stash` or `commit --amend`.
6. `--dry-run` prints what it would do and checks the preconditions only.

Add `tools/ops/tests/test-land.sh` that builds temporary repos with a main checkout and a second worktree and covers: a clean fast-forward; a non-fast-forward branch refused; a dirty unrelated file in the main checkout allowed; a dirty file the merge touches refused with master unchanged; master not checked out refused. Do not edit any skill file (the lead will point the workflow skill at the script).

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ6 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ7 · review-docs-truth-infra-tooling · Codex Luna · effort low · pending

**Fix docs that contradict the code: Terraform defaults, runner header, Tauri README, perf README**

Documentation fixes only (no behaviour changes). Check each claim against the code before editing.
1. `infra/cloud-sims/README.md`: it states a stale monthly budget (lines near 15, 34, 87); `infra/cloud-sims/variables.tf` defaults `monthly_budget_usd` to 200. Correct every mention, and recompute or reword the "about 70 server-hours: roughly 60 million Quick duels" line (Quick speed no longer exists; say "duels").
2. `infra/cloud-box/README.md`: it says `daily_cap_hours` defaults to 8 (lines near 20 and 46) but `variables.tf` defaults it to 24 (no cap; idle auto-stop and the monthly budget are the guards). Fix the text and the worst-case cost lines.
3. `tools/astra/runner.sh` header comment (lines 9-12): it says `astra` means gpt-6-astra. The `model_slug` function maps every key except `luna` to gpt-6-sol and `astra` also means high default effort. Rewrite the comment only; change no code (the script is running live).
4. `app/README.md` line 27 says "There are no Rust commands". `app/src-tauri/src/lib.rs` registers `#[tauri::command]` functions (autosave/archive/scenario). List them in one short sentence each, from the code. Fix the links to `plans/` (around lines 43 and 51) and the link at `crates/server/README.md:33`: point to a maintained doc under `docs/` or drop the reference.
5. `docs/sim-results/perf/README.md` (around line 23) calls a Standard duel "Quick"; fix the label.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ7 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ8 · review-docs-superseded-drafts · Codex Luna · effort low · pending

**Mark two stale design drafts as superseded and make their links portable**

Two design drafts describe removed systems as if current and are confusing for contributors.
1. `docs/design/diplomacy-design.md`: lines about a single Influence pool (around 76-94) and pact / security treaties and Quick-speed timing (around 241-245) describe removed systems. Current diplomacy: Peace, Alliance or War with visible betrayal; borders grow per city; treaties cost Gold (see "Influence and diplomacy" in `.claude/skills/legatum-rules/references/game-rules.md`).
2. `docs/design/tech-tree-design.md` line 34 says eras impose "no additional research quota"; the current rule is a strict all-technologies-in-era gate (see CLAUDE.md and the rules reference). Line 10 links to a local absolute path.

For each: add a short status note at the very top ("Superseded 2026-10-18. Kept as history. Current rules: <relative link>."), list in two or three bullets which ideas in the draft no longer apply, and replace every `/Users/...` link with a repo-relative link (or remove it). Keep the original text below the note. Read the rules reference sections before writing so the note is accurate. Change nothing else.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ8 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ9 · review-error-toast-a11y · Codex Luna · effort medium · pending

**The game error message gets a live-region role and a keyboard-reachable close button**

`app/src/screens/GameScreen.tsx` around line 137 renders the error as a clickable `<div className="toast bad" onClick=...>`. Keyboard and screen-reader users cannot dismiss it, and it is not announced.

Do:
1. Give the toast `role="alert"` and a native `<button type="button" className="toast-close" aria-label="Dismiss">` that clears the error (same `set({ error: null })`). Keep the message text selectable; remove the whole-toast `onClick` only if no existing e2e or vitest test depends on clicking the toast body (`grep -rn "toast" app/e2e app/src`). If one does, update that test to click the button.
2. Style the close button in `app/src/styles.css` next to the existing `.toast` rules, using existing colour tokens, a 24 px minimum hit area, and a visible focus ring. Keep the toast within the layout at 1000x600.
3. Add a vitest + Testing Library test (follow the nearest existing GameScreen or toast test) covering the role, the accessible name, and keyboard activation.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ9 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- Any text a player reads follows the plain-language guide in `.claude/skills/legatum-ui/SKILL.md` (Game text: plain language, with its glossary), and `npx vitest run src/ui/textArtifacts.test.tsx src/ui/copySource.test.ts` must pass.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CQ10 · review-app-small-cleanups · Codex Luna · effort low · pending

**Three small app cleanups: one view call less on load, internal-only exports, no wall-clock assertion**

Three independent small changes.
1. `app/src/store.ts`: `afterLoad()` calls `humanIds(client)`, which calls `client.view(0)` for the whole view, and then `readViews(client, humans)` builds every human's view, so seat 0's full view is built and JSON-parsed twice on every game start and load. Read seat 0 once, derive the human ids from it, reuse it as `views[0]`, and build only the remaining human views. Check solo, hot-seat (two humans), save-load and LAN code paths that use `humanIds` or `readViews` (`grep -n "humanIds\|readViews" app/src`). Add a vitest that spies on `client.view` and asserts one call per human seat.
2. Remove `export` from `BUILD_RULE` and `OWN_BOUNTY_SHOWN` in `app/src/ui/CityPanel.tsx` and from `PromotionPips` and `UnitBadge` in `app/src/ui/UnitPanel.tsx`, after `grep -rn` (including tests) shows no importer. Leave any that something imports.
3. the former TypeScript engine (around line 105) asserts `view()` averages under 5 ms. It fails under machine load (docs/learnings.md). Replace it with a semantic check: two consecutive `view()` calls return deeply equal results and do not advance the turn or change the game state.
Keep each change minimal; other branches are editing UnitPanel and the mock.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ10 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ11 · review-cities-overview-tests · Codex Luna · effort medium · pending

**Behaviour tests for the Cities overview and Where to build**

The Cities overview (`app/src/ui/CitiesOverview.tsx`) and Where to build (`app/src/ui/WhereToBuild.tsx`) landed on 2026-10-17 with a screen-tour shot but no behaviour tests (`whereToBuild.wasm.test.ts` covers only the build logic).

Add `app/src/ui/CitiesOverview.test.tsx` and `app/src/ui/WhereToBuild.test.tsx` (Testing Library; follow the fixture helpers used by `CityPanel.test.tsx` and the neighbouring tests). Cases:
- Cities overview: clicking a column header sorts ascending, clicking again reverses; selecting a row selects and focuses that city and closes the overview; a city that is blocked shows its reason; row order is stable after a production choice in another row.
- Where to build: the fastest city is listed first for an item; a blocked city shows a readable reason; a city already building the item elsewhere is marked; choosing "Build here" and "Queue here" sets the production order for that city and leaves the list order and the displayed turns for the other rows unchanged.
Test only. If you need a stable hook, add an accessible name (`aria-label`) to the existing element and say so. If the real behaviour differs from the list above, test the real behaviour and report the difference.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ11 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- Any text a player reads follows the plain-language guide in `.claude/skills/legatum-ui/SKILL.md` (Game text: plain language, with its glossary), and `npx vitest run src/ui/textArtifacts.test.tsx src/ui/copySource.test.ts` must pass.


## CQ12 · review-release-check-workers · Codex Luna · effort medium · pending

**Release check: load-aware Chromium workers and a configurable WebKit worker count**

Release verification spends about 2,400 s in Chromium and 3,000 s in WebKit (timings ledger, 2026-10-18). `tools/release/check.sh` fixes Chromium at `--workers=6` (line ~103) and `tools/release/webkit-docker.sh` (4 CPUs) passes no worker count, so Playwright uses its default of one. `tools/astra/verify-branch.sh` (around lines 76-90) already sizes workers from the core count and the load.

Do (do not edit `verify-branch.sh`; another agent is working there):
1. Create `tools/ops/pw-workers.sh`, a small sourceable bash 3.2 function `pw_workers [cores] [load]` that reproduces verify-branch's formula (cores/3, capped at 24, reduced when the 1-minute load is high; read the comment and code at those lines and copy the rule exactly). Add `tools/ops/tests/test-pw-workers.sh` with table-driven cases.
2. In `tools/release/check.sh` use it for the Chromium `--workers` (keep `LEGATUM_PW_WORKERS` as an override).
3. In `tools/release/webkit-docker.sh` pass `--workers=${LEGATUM_WEBKIT_WORKERS:-1}` to Playwright (default stays 1, so behaviour is unchanged until someone measures), and print the value used.
4. Do not run a release check, a container or Playwright. Choosing the WebKit default is a later measured task.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ12 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ13 · review-server-scenario-run-cap · Codex Sol · effort medium · pending

**LAN server: bound scenario-run uploads (size and total retention) and say plainly that LAN play is for trusted networks**

`POST /api/scenario-runs` in `crates/server/src/lib.rs` (route near line 102, handler near 274, `save_scenario_run` in `host.rs` near 964) accepts any syntactically valid run from any LAN client and writes a client-named file under `<saves>/scenarios/` with no aggregate limit; the only guard is the global 16 MiB request limit and a Host/Origin check that a LAN client can forge. Decision (user, 2026-10-18): keep uploads working from LAN browsers (a second player on the LAN can play scenarios), but bound them.

Do:
1. Give this route its own body limit of 2 MiB (axum `DefaultBodyLimit` on the route) and check the largest real run in `scenarios/` fixtures or test data fits comfortably; say what you measured.
2. Before writing a run, keep the scenarios directory to at most 500 files and 100 MiB by deleting the oldest files first (by modified time). Make both limits named constants.
3. Tests in `crates/server/tests/`: an upload over 2 MiB is rejected with a clear error; after 501 uploads only 500 files remain and the oldest is gone; a run with a path-like id is still rejected (existing behaviour).
4. In `crates/server/README.md` add a short "Network trust" section: the server speaks plain HTTP and WebSocket and sends seat tokens in the clear, so it is meant for networks you trust (a home LAN), not public Wi-Fi; it listens on all interfaces; list the limits above.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ13 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ14 · review-perf-baseline-refresh · Codex Luna · effort medium · pending

**Fresh simulator speed baseline on the cloud box, written into the perf ledger**

The simulator speed ledger in `docs/sim-results/perf/README.md` was last updated 2026-10-06 and the AI has changed a lot since (scouting, war follow-through, territory sight). Before anyone optimises routes again we need a current baseline.

Do (no code changes; run on the cloud box, never the Mac: `tools/cloud/remote.sh . -- <command>` with `--artifact` for output files; see `.claude/skills/legatum-workflow/SKILL.md` for the rules):
1. Read the header of `crates/sim/examples/sim_bench.rs` and the README for its modes (whole duel, eight-player game, search, rollout, latency, view digests) and the exact commands the previous ledger used.
2. Run each mode with the same fixed seeds as the previous ledger, building with `--profile sim`. Record commit hash, toolchain, box load, and the numbers (and digests where printed).
3. Append a dated section to the README with a table comparing against the 10-06 numbers, and fix the "Quick" duel label to Standard.
4. If `sim_bench` already prints route-search counts by caller, include them; do not add instrumentation.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ14 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ15 · review-heraldry-concept-sheet · Codex Luna · effort high · pending

**Heraldry: concept sheet of eight historically grounded charges for the user to approve**

Decision (user, 2026-10-18): the faction crests still show charges from the retired fictional setting (for example a keyhole city and crossed keys for the Switzerland, a brood-cell design for the Japan). Replace them with historically grounded charges. The shared relay-star rim and the gold-on-navy embossed crest treatment stay.

Read `.claude/skills/legatum-art/SKILL.md` and `app/src/assets/heraldry/README.md` first. The internal faction ids map to the historical peoples as follows (check against `.claude/skills/legatum-factions/SKILL.md`): Switzerland = Switzerland, Japan = Japan, Byzantium = Byzantium, Germany = Germany, Celts = Celts, Mongols = Mongols, Sparta = Sparta, Athens = Athens.

Produce a CONCEPT SHEET only, for the user to approve:
- one proposed charge per faction, drawn from objects, architecture or motifs that really belong to that people (a different real object from the ones other factions use), each readable as a silhouette at 24 px;
- `docs/design/heraldry/historical-charges-2026-10-18.png`: all eight crests side by side at large size, then again at 48 px and 24 px, labelled with the display name;
- `docs/design/heraldry/historical-charges-2026-10-18.md`: for each faction, one sentence naming the real object and why it fits, and the image prompt you used.
Do not touch `app/src/assets/heraldry`, any code, or any other art. Finalising the crests, seals and glyphs is a later task after the user approves.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ15 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CQ16 · review-random-faction-glyph · Codex Luna · effort medium · pending

**A real icon for the Random faction instead of a text question mark**

`app/src/ui/FactionPicker.tsx` (around line 161) draws the Random faction's hero portrait as a literal `?` character. The icon rules (`app/src/assets/icons/README.md`, `.claude/skills/legatum-art/SKILL.md`) say every concept gets its own glyph, not a text character.

Do: draw `app/src/assets/icons/ui-random.svg`, a single-colour tintable glyph on the 24-unit grid, one clear symbol for chance or selection (for example a die or crossed arrows), solid silhouette with knocked-out detail, readable at 16 px. Register it the way the other `ui-*` icons are (find the icon registry and any preview or distinctness list and follow them), and use it in the Random portrait instead of the `?`. Keep the existing circular portrait treatment. Run the icon tests, `npx tsc --noEmit` and the vitest suite.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CQ16 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CH1 · review-plan-rs-dead-code-notes · Codex Luna · effort low · hold

**Remove four stale dead-code allowances and the "currently unused" comments in the AI campaign forecast**

Release when: the AI war follow-through work (plan.rs, military.rs) has landed on master.

In `crates/ai/src/plan.rs`, `CampaignOutlook` (line ~385), `MIN_SEAT_WAR_ODDS` (~419), `campaign_against` (~940) and `campaign_outlook` (~982) carry `#[allow(dead_code)]`, and the doc comment at line ~383 says "currently unused; kept for the search AI". They are in use: `campaign_outlook` is called from plan.rs (several places) and `military.rs` (2727, 2790), and `campaign_against` from plan.rs (~1234, ~1779). Remove the four allowances, rewrite the comment from the real call sites, and update the matching sentence in `docs/learnings.md` (around line 1293: "awaiting use"). Run `cargo clippy --workspace --all-targets` and the `legatum-ai` tests.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH1 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH2 · review-air-surprise-strike · Codex Sol · effort medium · hold

**Air strikes on the turn war is declared: make validation agree with resolution (allowed, like ground attacks)**

Release when: review-air-order-fixes (CQ4) has landed (same file).

Decision (user, 2026-10-18): an air strike on the turn you declare war is LEGAL, the same as a ground attack (ground validation already counts same-turn declarations through `declared_wars(state, orders)` in `crates/core/src/rules/units.rs`). Today it is inconsistent: `crates/core/src/rules/air.rs` `validate_orders` calls `validate_strike` against the planning state, where `battle::target_at` requires War, so the order is rejected; but `resolve_turn` starts a surprise war before the air phase and validates the strike again against the now-hostile state, so a strike submitted without validation resolves.

Do:
1. Make air validation treat wars declared in the same order list as ground validation does (reuse the same helper; do not duplicate its logic).
2. Rebases stay sequential and ID-ordered. State that in the air section of `.claude/skills/legatum-rules/references/game-rules.md` (near line 194: "a strike needs an existing War" becomes "or a War declared this turn"; add "rebases apply in aircraft id order, so two aircraft cannot swap between full hangars").
3. Check the client's order availability for air strikes (`grep -rn "AirStrike" app/src`) and the former TypeScript client the former TypeScript engine air validation for the same rule; keep them consistent.
4. Tests: `DeclareWar` + `AirStrike` against a Peace neighbour is accepted by `validate_orders` and the strike resolves in `resolve_turn`; the same strike without a declaration is rejected by both.
Run the workspace tests; if a pin changes, report exactly which and why.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH2 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH3 · review-docs-status-reconcile · Codex Luna · effort medium · hold

**Reconcile docs/status/next-tasks.md and tracker.md with what has landed**

Release when: the Military Engineer has landed on master and the lead has no pending edits to the status files.

`docs/status/next-tasks.md` and `docs/status/tracker.md` still list finished work as pending. Reconcile each item against current master (`git log`, `git branch --merged master`, `docs/learnings.md`, the rules reference) and the release tags.

Known stale items: the "Overnight plan 2026-10-17 to 18" and "Paused at the usage limit" sections (remove-boosts and no-stockpile-cap landed; the Keshig landed; decadence landed on the mean); "Mongols: no siege until they capture a city with a Workshop" (landed as the Mongol siege unlock); section 1 "Finish what is parked" (the six parked branches landed 2026-10-11); the tracker's "Next playtest build (8.9)" and "Waiting on the user" items that were answered; the "Next AI change: real coalitions" bullet 2 "Commitment: declare only with a force staged for a named city" (docs/learnings.md 2026-10-18 found that gate weakened pressure on the leader; replace it with "price commitment through the plan, time strikes together, open extra fronts").

Do:
1. Move every completed or superseded item, with its date, into a new `docs/status/archive/2026-10.md` (history, not deleted).
2. Leave only genuinely open work, and keep every promise to the user and every waiting-on-user item that is still open.
3. Do NOT remove or reword the "Review plan (approved 2026-10-18)" sections or the link to `docs/status/review-plan-2026-10-18.md` at the top of both files.
Mark anything you are unsure about "check with the lead" instead of deleting it.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH3 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH4 · review-rules-reference-reconcile · Codex Sol · effort high · hold

**Make the rules reference match the code: siege units, bought Walls, decadence on the mean, order summary**

Release when: claude/military-engineer and astra/walls-no-buy have both landed on master.

`.claude/skills/legatum-rules/references/game-rules.md` is the "code-verified" reference but contradicts newer decisions. Verify each point against the code on current master, then fix the reference (and the two skill lines named) so it describes what the engine does. Where the code disagrees with a settled decision, do not change the code; list it in your final message.
1. Siege units: the roster and tech unlock lines (around 133-135 and 185-192) list Catapult and Bombard; the Military Engineer decision (legatum-design skill, Military Engineer entries) replaces them. Describe only what is live. In `.claude/skills/legatum-factions/SKILL.md` (lines ~33 and ~183) the Mongol line should match what landed (siege unlocks after capturing a city with a Workshop).
2. Walls: around lines 1508 and 1625-1640 the AI "buys Walls at the door" and a `price_bought_walls` gene exists, but the decision is that Walls are production-only (legatum-design skill). Check whether any purchase path for Walls remains in `crates/core/src/rules/cities.rs` and `crates/ai`; describe the real behaviour.
3. Decadence: reference around 939-961 versus the design skill lines 73, 89 and 263 (saturating curve, "mean", "median"). Keep one current statement in the reference; mark the older entries in the design skill "superseded by <date entry>", do not delete them.
4. The order summary at line 7 says "one per unit/city/research slot per turn" but city validation allows `SetProduction` followed by `BuyItem`; reword.
5. Add under save compatibility: a save made before `title_since` and `vp_holds` existed loads with empty maps, so every incumbent crown's upkeep clock restarts and a team's victory hold restarts at one turn. This is accepted behaviour (user, 2026-10-18), documented only.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH4 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH5 · review-rules-reference-split · Codex Luna · effort medium · hold

**Split the 2,100-line rules reference into topic files with an index**

Release when: review-rules-reference-reconcile (CH4) has landed.

`.claude/skills/legatum-rules/references/game-rules.md` is about 2,100 lines and 256 KB. Split it by its existing top-level sections (turn flow, economy, combat and sieges, diplomacy, victory and crowns, map and terrain, factions, systems that do NOT exist, AI behaviour) into files under `.claude/skills/legatum-rules/references/` and make `game-rules.md` a short index with one line per topic. Keep headings and code pointers unchanged; update the `SKILL.md` text that tells agents which file to read; update links inside the files. No wording changes. Verify with a script that the concatenated word count equals the original and every original `##` heading appears once.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH5 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH6 · review-learnings-split · Codex Luna · effort medium · hold

**Archive old learnings by month and keep docs/learnings.md short**

Release when: the lead has no open edits to docs/learnings.md (check with the lead; many agents append to it).

`docs/learnings.md` is 2,600 lines and 669 KB. Keep the newest entries (2026-10-12 onward) plus a short index of topics in `docs/learnings.md`, and move older entries, unchanged, into `docs/learnings/2026-09.md` and `docs/learnings/2026-10-early.md` (split on the date headings). Update links in CLAUDE.md, skills and docs that point into old line numbers or anchors (`grep -rn "learnings.md" .claude docs CLAUDE.md tools`). Check `.claude/hooks/learnings-check.sh` still detects an updated learnings file and widen it to the new directory if needed. Verify no entry is lost (count of dated headings before and after).

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH6 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH7 · review-skills-trim · Codex Luna · effort medium · hold

**Shrink the five largest skills to short entry points with focused reference files**

Release when: the Military Engineer, Germany and scenario-lab branches have landed (they edit these skills) and the lead agrees nobody is mid-edit.

`legatum-ui` (443 lines, 133 KB), `legatum-design` (267 lines), `legatum-factions` (213), `legatum-art` (197) and `legatum-workflow` (182) are too long for an agent to read at the start of a task. For each, keep in `SKILL.md` the description, the principles, and a one-line pointer per reference; move long dated entries, examples and checklists, unchanged, into `.claude/skills/<skill>/references/<topic>.md`. Do not reword or drop any decision. Update `CLAUDE.md` pointers if they name sections. Check no hook or script greps these files by line (`grep -rn "SKILL.md" .claude tools`).

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH7 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH8 · review-split-tests-production-view · Codex Luna · effort low · hold

**Move the embedded test modules out of production.rs and view.rs**

Release when: claude/military-engineer, eu4-wars and the war follow-through have landed (they edit view.rs, production.rs and the AI crate).

`crates/ai/src/production.rs` (3,840 lines; tests start near line 3282) and `crates/core/src/view.rs` (3,108 lines; tests start near line 2165) embed large `#[cfg(test)] mod tests`. Move each into a sibling file (`production_tests.rs`, `view_tests.rs`) declared with `#[cfg(test)] mod tests;` so private access still works. Pure move: no behaviour or test changes; count the tests before and after (`cargo test -p legatum-ai -- --list`, same for core).

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH8 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH9 · review-split-styles-css · Codex Luna · effort medium · hold

**Split app/src/styles.css by feature, keeping the cascade identical**

Release when: no open UI branch edits styles.css (Military Engineer, Germany, scenario-lab, play-time landed).

`app/src/styles.css` is 4,232 lines. Split it along its existing section comments (title and setup, game shell and HUD, panels, reports, modals and overlays, responsive overrides at lines near 361, 846, 2364, 3082, 3196) into files imported in exactly the same order, keeping shared tokens and base rules in `styles.css`. Do not reorder, merge or delete a rule. Acceptance: build `app/` before the change and after, minify both CSS bundles and diff them (whitespace and comments aside they must be identical); run the screen tour and `layout.spec.ts`.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH9 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CH10 · review-split-format-ts · Codex Luna · effort low · hold

**Split app/src/format.ts by responsibility without changing any wording**

Release when: Military Engineer and Germany branches have landed (they edit format.ts).

`app/src/format.ts` (1,549 lines) combines event translation with report filtering. Split into: shared report context and types; combat and siege lines (around line 397); faction-goal lines (~532); the general event dispatcher (~609); and a separate `reportFilters.ts` for filtering, grouping and movement-path extraction (~1490-1531). Keep `format.ts` as a barrel that re-exports every current export so no importer changes. No wording, ordering or category change. Run the full vitest suite.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH10 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- Any text a player reads follows the plain-language guide in `.claude/skills/legatum-ui/SKILL.md` (Game text: plain language, with its glossary), and `npx vitest run src/ui/textArtifacts.test.tsx src/ui/copySource.test.ts` must pass.


## CH11 · review-split-types-ts · Codex Luna · effort low · hold

**Split app/src/types.ts into view, order, event and forecast modules behind a barrel**

Release when: Military Engineer and Germany branches have landed (both edit types.ts).

`app/src/types.ts` (1,284 lines) mirrors the Rust wire types. Split it into `types/view.ts`, `types/orders.ts`, `types/events.ts`, `types/forecast.ts` (and keep UI-only types in their own file) with `app/src/types.ts` re-exporting everything so no importer changes. Do not rename a field or tag; do not touch decimal-string `u64` handling. Run `npx tsc --noEmit` and the vitest suite.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH11 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH12 · review-split-lobby · Codex Luna · effort low · hold

**Extract the seat table and settings controls from LobbyScreen**

Release when: no open branch edits the lobby.

`app/src/screens/LobbyScreen.tsx` (651 lines) holds the screen, the host hook (`useStagingHost`, ~190), `LobbyFrame` (~231), `SeatsTable` (~286) and `Settings` (~555). Move `SeatsTable` and its row components, and `Settings`, into their own files under `app/src/screens/lobby/`. Keep the host hook and screen assembly where they are. No behaviour or markup change. Run lobby-related vitest and Playwright specs and `layout.spec.ts`.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH12 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CH13 · review-split-store · Codex Sol · effort high · hold

**Split the Zustand store implementation into slices while keeping one exported hook**

Release when: astra/play-time and claude/scenario-lab have landed (both edit store.ts) and format/types splits are done.

`app/src/store.ts` (1,231 lines) is one state interface and one creator covering orders, reports, map selection, LAN, staging, autosave and scenarios. Split the implementation into action modules or slices (session and gameplay, reports, LAN and staging, scenarios and recording) under `app/src/store/`, composed back into one `useGame` hook with the same state shape and action names. Keep turn resolution and load/reset lifecycle ownership in one place and explain it in a header comment. Extract pure helpers (save parsing at ~332, turn-result projection) as plain functions. No component call site changes in this task. Run the whole vitest suite, tsc, and the Playwright specs for solo, hot-seat, save/load and LAN.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH13 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CH14 · review-split-citypanel · Codex Sol · effort high · hold

**Split CityPanel into production, territory and air-wing sections**

Release when: the CityPanel is not being edited by an open branch and the store split (CH13) has landed.

`app/src/ui/CityPanel.tsx` (787 lines) owns production previews, queue interactions, purchases, unlocks, territory and improvement selection, scroll anchoring, visibility and the air-wing picker. Extract the production list and queue, the territory and improvement controls, and the air-wing picker into components or hooks under `app/src/ui/city/`. Keep shared decisions (order edits, production forecasts) in their existing helpers, and keep scroll anchoring, the sticky header and map-picker cleanup behaving exactly as now. Do not change visible text or rules. Run vitest, `layout.spec.ts`, the city-panel and production-picker specs, and the screen tour.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH14 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- Any text a player reads follows the plain-language guide in `.claude/skills/legatum-ui/SKILL.md` (Game text: plain language, with its glossary), and `npx vitest run src/ui/textArtifacts.test.tsx src/ui/copySource.test.ts` must pass.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CH15 · review-split-mapcanvas · Codex Sol · effort high · hold

**Separate MapCanvas input, camera and frame-scheduling concerns**

Release when: the CityPanel split (CH14) has landed and the selection-pulse work (CL17) has not started in MapCanvas.

`app/src/map/MapCanvas.tsx` (1,208 lines) mixes pointer and keyboard input, camera state and the animation-frame loop. Extract narrow hooks or modules: input handling, camera registration and updates, and frame scheduling. Keep a single owner of camera state, keep the `draw` contract and render timing, and make sure listeners are still cleaned up (no duplicates after remount). Do not move the renderer (`render.ts`) in this task. Run the map, keys, minimap and selection specs plus the screen tour.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH15 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CH16 · review-combat-lab-rerun · Codex Luna · effort medium · hold

**Re-run the combat lab on the current rules and record how wrong the siege forecast is now**

Release when: the Military Engineer has landed (it changes siege rules).

The AI review found the siege forecast calls 23% of winnable situations hopeless and 34% of its "capture" calls had no winning policy (`docs/sim-results/combat-lab/results.md`), but the rules changed after that sweep (phalanx, Engineer, walls). Re-run the sweep on the box (`tools/sim/combat-lab.sh sweep`, never on the Mac) on current master, write a dated section in `docs/sim-results/combat-lab/results.md` with the same tables, and add a comparison of forecast windows of 4, 5, 6 and 8 rounds against the real outcomes (the `siege_window` gene; see `crates/ai/src/opportunity.rs` and `city_assault.rs`). Report only numbers; do not change AI code.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH16 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH17 · review-scripted-diagnosis-transform · Codex Luna · effort low · hold

**Make production_diagnosis use the same final order transform as the live AI**

Release when: claude/eu4-wars has landed (it edits scripted.rs).

`crates/ai/src/scripted.rs`: `production_diagnosis` (lines ~374-420) finishes orders through a different path than the live logged planner (~493), which rewrites an illegal same-turn ranged attack on a neutral city into Hold. The diagnostic presents itself as exact orders but is not. Factor the final transform into one function used by both and add a test with a same-turn war declaration where both paths give identical orders.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH17 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH18 · review-runner-lane-setup-errors · Codex Sol · effort medium · hold

**Astra runner: a failed lane setup parks the task with a clear message instead of failing later**

Release when: the offload-lanes agent has finished (it edits runner.sh).

`tools/astra/runner.sh` uses `set -uo pipefail`; `prepare_lane` (lines ~74-94) runs `git worktree add` and symlink setup whose failures are not checked, so a broken lane shows up later as an opaque worker failure. Make `prepare_lane` return a checked status and have the queue loop park the task (leave the file in `pending/`, or move to `held/` with a note) and log a one-line diagnostic. Do not add a blanket `set -e`: this script collects statuses on purpose. Add a small shell test with a stubbed failing git. IMPORTANT: this script runs live. Do not edit it in place in the running checkout; your lane worktree copy is fine, and the lead installs the result by writing a new file and `mv`.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH18 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH19 · review-verify-remote-timeout-budget · Codex Luna · effort medium · hold

**Remote checks: one timeout budget that covers queueing and running, with a final status always printed**

Release when: the verify-speed agent has finished (it edits the verification scripts) and CQ3 has landed.

`docs/learnings.md` (2026-10-18) records a release check killed by its own client-side `timeout 7000` while waiting in the box queue, with no final status. `tools/cloud/verify-remote.sh` defaults its remote timeout to eight hours (line ~19) but callers wrap their own shorter timeout. Make `verify-remote.sh` and `tools/cloud/remote.sh` take one `--budget` covering queue plus run, print the budget and queue position at start, and on expiry or cancellation print the job id, the last known state and how to `--attach`. Document in the script header that wrapping them in an outer `timeout` is wrong. Test with a stub box.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH19 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH20 · review-verify-dedupe-tsc-server · Codex Luna · effort low · hold

**Verifier: stop type-checking twice and reuse an unchanged release server**

Release when: CQ3 (verify lock recovery) has landed (same file).

`tools/astra/verify-branch.sh` runs `tsc --noEmit` (line ~598) and then `npm run build`, whose script starts with `tsc -b` (app/package.json), and it rebuilds the release server for Playwright (line ~569, about 62 s) even when no server source changed. Make the build the only TypeScript check (verify that `tsc -b` covers the same files, including e2e, as the standalone run; if not, keep the standalone run for the uncovered files only), and cache the release server executable keyed by the server-relevant Rust inputs and toolchain (restore and hash-check it before Playwright). Use the existing gate cache. Prove with the step timings the stubbed tests print.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH20 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH21 · review-worktree-lifecycle-doc · Codex Luna · effort low · hold

**One page on the three worktree ownership modes (lanes, agent pool, verifier trees)**

Release when: the worktree-pool ownership fix (CL5) has landed.

Three tools create and reuse worktrees with different rules: Astra lanes (`tools/astra/runner.sh` ~74), the agent pool (`tools/ops/agent-worktree.sh` ~94) and verifier trees (`tools/astra/verify-branch.sh` ~331). Write `docs/ops/worktrees.md`: for each mode, who owns the lock, when reset or clean is allowed, how to release, how stale locks are recovered. Quote the actual script lines. Link it from `.claude/skills/legatum-ops/SKILL.md`. Describe the new ownership rules from the worktree-pool fix (CL5) only if it has landed.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH21 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH22 · review-tools-index · Codex Luna · effort low · hold

**An index of tools/ and scripts/ that separates supported commands from historical one-offs**

Release when: claude/oss-readme has landed (it adds README files).

Write `tools/README.md` (one table: path, what it does, status: supported / test helper / historical) covering `tools/{astra,cloud,sim,ops,release,art,evolve,league,scenarios,analysis}` and the top-level files of `scripts/`. For the `scripts/summarize-tech-*.py` family and `tools/league/analyze_hard_handoff*.py`, mark them historical and link the report each one produced (find it from their headers or `docs/sim-results`). Do not move or delete anything. Read each script's header before describing it.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH22 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH23 · review-slot-queue-bash5 · Codex Luna · effort low · hold

**Run the box queue regression test under the box's Bash 5 and print the version**

Release when: the lane is idle and the box is not busy with verifies.

The 2026-10-11 box-queue PID bug passed locally under macOS Bash 3.2 and failed on the box's Bash 5 (`docs/learnings.md`). `tools/cloud/box/tests/test-slot-queue.sh` should print `$BASH_VERSION` and, when run through `tools/cloud/remote.sh . -- bash tools/cloud/box/tests/test-slot-queue.sh`, assert the major version is at least 5 when `LEGATUM_EXPECT_BASH5=1`. Add a short note in `tools/cloud/box/README` (or the script header) telling people to run it on the box after touching `slot-queue.sh`.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH23 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH24 · review-audit-deps-script · Codex Luna · effort low · hold

**A repeatable dependency advisory check for both Rust lockfiles and the npm lockfile**

Release when: claude/oss-readme has landed and the lead has OK for installing cargo-audit on the box.

Write `tools/ops/audit-deps.sh`: run `cargo audit` against `Cargo.lock` and `app/src-tauri/Cargo.lock`, and `npm audit --omit=dev=false` in `app/` (against the registry, not the offline cache), print vulnerable and "unmaintained" findings in separate sections (the known `proc-macro-error` entry from the Linux GTK dependencies is informational), and exit non-zero only for vulnerabilities. Run it through `tools/cloud/remote.sh` (install `cargo-audit` on the box if missing; if you cannot, write the script and say so). Add a line to `CONTRIBUTING.md` only if that file exists.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH24 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## CH25 · review-tour-split · Codex Luna · effort high · hold

**Split screen-tour.spec.ts into selectable domains with separate screenshot directories**

Release when: the fast Playwright profile design (CL23) has landed.

`app/e2e/screen-tour.spec.ts` (118 tests across two sizes, about 89% of Playwright time) must be split so each part runs alone and in parallel without overwriting another's screenshots. Split into domains: title and lobby, turn-one game, diplomacy and war, research and unlocks, post-game. Each file creates its own fixture state and writes to `test-results/screen-tour/<domain>/`. Keep every shot and assertion; verify the old and new shot counts match. Copy-on-run rule: the lead copies `app/test-results/screen-tour` after a full run, so keep that parent directory.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH25 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CH26 · review-keys-minimap-specs · Codex Luna · effort low · hold

**Keyboard and minimap specs assert what the player sees, not camera internals**

Release when: the map input split (CH15) has landed.

`app/e2e/keys.spec.ts` (line ~17-20) reads `legatumMap.camera.x` and waits a fixed 250 ms; `app/e2e/minimap.spec.ts` (~56, ~152) reads camera internals and compares `canvas.toDataURL()` bytes. Rewrite to user-visible outcomes: after an arrow key a known city's on-screen position moves in the expected direction (use `expect.poll`); clicking the minimap brings a known tile into view; dragging moves the viewport frame. Remove fixed sleeps. Keep the tablet and phone cases only where they protect a distinct layout.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH26 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## CH27 · review-save-size-limit-ui · Codex Luna · effort low · hold

**Reject oversized save files before reading them in the desktop picker and autosave**

Release when: CL1 (save integrity at load) has chosen the limit and landed.

`app/src/platform/files.ts` (~33-56) reads a chosen save fully into a string; `app/src/store.ts` (~975-985) passes it to the client; autosave is read at startup. Add a size check using the limit chosen in item CL1 (exported by the Rust/WASM side as a constant or documented in `docs/`): files over the limit are rejected with "This save file is too large to open" before any read. Test with a fake `File`. Plain-language guide applies to the message.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH27 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- Any text a player reads follows the plain-language guide in `.claude/skills/legatum-ui/SKILL.md` (Game text: plain language, with its glossary), and `npx vitest run src/ui/textArtifacts.test.tsx src/ui/copySource.test.ts` must pass.


## CH28 · review-quick-save-notice · Codex Sol · effort medium · hold

**Opening an old Quick-speed save tells the player the game now plays at Standard**

Release when: CL1 (save integrity at load) has landed (same parse path).

Decision (user, 2026-10-18): do not convert old Quick saves; show a one-line notice. `crates/core/src/state.rs:24` deserializes `"Quick"` as `Standard` silently. In `crates/wasm/src/json.rs` `parse_state` detect `setup.speed == "Quick"` before conversion and return it as a load notice (extend the load result the WASM `Session` already returns for other notices, or add a field with `#[serde(default)]`), and show one line in the UI after loading: "This save was made at Quick speed, which no longer exists. It now plays at Standard speed, so costs are higher than before." Add a fixture under `crates/wasm/tests/fixtures/saves/` (a minimal Quick save) and make the old-saves regression test assert the notice. Follow the plain-language guide in `.claude/skills/legatum-ui/SKILL.md`; `npx vitest run src/ui/textArtifacts.test.tsx src/ui/copySource.test.ts` must pass.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item CH28 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.

- Any text a player reads follows the plain-language guide in `.claude/skills/legatum-ui/SKILL.md` (Game text: plain language, with its glossary), and `npx vitest run src/ui/textArtifacts.test.tsx src/ui/copySource.test.ts` must pass.

- This changes UI you cannot see (no browser in your sandbox). Say so in your final message; Claude will take screenshots at 1000x600 and 1440x900 before it lands.


## M3b · review-mock-fixtures-migrate · Codex Luna · effort high · hold

**Mock retirement stage 3b: migrate UI tests from the mock to core-generated fixtures, one batch per task**

Release when: M3a (fixture format and generator) has landed. Re-queue until no test outside retired TypeScript engine imports the mock.

Part of retiring the former TypeScript client engine. After stage 3a (a Rust-generated fixture format and generator, item M3a) exists, migrate UI tests that currently build their state through the former TypeScript engine or the former TypeScript client. This task: pick the next 25 test files that import from the former TypeScript engine (list them with `grep -rln "from '.*mock" app/src --include=*.test.*`; skip files under the former TypeScript engine itself), regenerate each one's state with the fixture generator (read its README), keep every assertion, and make the tests pass. Delete nothing in the former TypeScript engine yet. Report how many files remain. Re-queue yourself by listing the remaining count in your final message.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item M3b of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## M4 · review-mock-delete · Codex Luna · effort high · hold

**Mock retirement stage 4: delete retired TypeScript engine and refuse old mock-format saves with a clear message**

Release when: stages M1, M2a-M2d, M3a and M3b are all done and the lead has confirmed.

Final stage of retiring the former TypeScript client engine (one-engine rule; open source). Only start if `grep -rn "from '.*mock" app/src app/e2e --include=*.ts --include=*.tsx` outside the former TypeScript engine finds nothing.
1. Delete the former TypeScript engine completely (resolver, AI, map generation, save path, parity tests). Remove the former TypeScript client, the the former client selector option and `ClientKind 'mock'` from `app/src/client.ts`, and any build flag that only the mock used.
2. Old autosaves written by the mock have a different shape (`app/src/store.ts` around line 332 parses and accepts them). Detect a mock-format save and refuse with: "This save was made by the old test engine and cannot be opened." Add a vitest with a mock-format fixture (copy a tiny one from git history before deleting).
3. Replace the WASM parity tests with nothing (their job is done by the Rust tests); remove references from `tools/` and `docs/` (`grep -rn "mock" tools docs/*.md`; keep dated learnings). Update `app/README.md` and `ARCHITECTURE.md` text if they mention a mock.
4. Run vitest, `npx tsc --noEmit`, `npm run build`, and the full Playwright suite.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item M4 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## A1 · review-art-future-era · Codex Luna · effort high · hold

**Repaint the Future-era painting without humanoid robots**

Release when: queue has room (fewer than 6 pending tasks).

Read `.claude/skills/legatum-art/SKILL.md` and the README beside the target assets first, and follow the series technique and delivery sizes exactly. Provide the prompt you used next to the source image. Do not edit TypeScript; wiring is a separate task.

Decision (user, 2026-10-18): keep the Future era but make it read as a grounded near future, not science fiction. Repaint `app/src/assets/eras/future-1280.png` (and the portrait derivatives that exist for the other eras): same oil-painted, parchment-compatible series, same wide composition and sizes. Replace the humanoid robots, drones and gleaming speculative city with a plausible continuation of the world: recognisable civic architecture, solar fields and power lines, rail and harbours, cultivated land, people. Provide a comparison sheet of the new image beside the Modern and Industrial era paintings.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item A1 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## A2 · review-art-greek-portraits · Codex Luna · effort high · hold

**Sparta and Athens portraits that differ at 48 px**

Release when: queue has room (fewer than 6 pending tasks).

Read `.claude/skills/legatum-art/SKILL.md` and the README beside the target assets first, and follow the series technique and delivery sizes exactly. Provide the prompt you used next to the source image. Do not edit TypeScript; wiring is a separate task.

Decision (user, 2026-10-18): the paired portraits for Sparta and Athens look alike at the faction-picker size (both are a Greek man and woman against classical settings). Make new paired portraits (exactly two unnamed adults each, period dress, refined oil technique) that differ in pose, silhouette, setting and dominant value pattern: Sparta martial and disciplined (a Spartan setting, hard light, red and bronze), Athens civic and intellectual (agora or stoa, marble, blue and ochre). Targets: the files under `docs/design/historical-factions/` and the app portrait assets that `app/src/brand/ornaments.tsx` loads (find them); deliver in the same sizes. Include a comparison image of old and new at 48 px (portraits-greek-48px.png in docs/design/historical-factions, archived outside the repository), comparing old and new at 48 px.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item A2 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## A3 · review-art-swiss-german-portraits · Codex Luna · effort high · hold

**Switzerland and Germany portraits that differ at 48 px**

Release when: queue has room (fewer than 6 pending tasks).

Read `.claude/skills/legatum-art/SKILL.md` and the README beside the target assets first, and follow the series technique and delivery sizes exactly. Provide the prompt you used next to the source image. Do not edit TypeScript; wiring is a separate task.

Decision (user, 2026-10-18): the Switzerland and Germany portraits both show a pair leaning over an indoor table at the picker size. Make new paired portraits (exactly two unnamed adults each) that differ in pose, setting and value: Switzerland, a Confederacy civic-finance scene (a counting house or toll bridge with a mountain window, cool light, red and white accents); Germany, a 1870s-80s industrial workshop (iron machinery, forge glow, standing figures, warm dark values). Same files and sizes as the current portraits (find them via `app/src/brand/ornaments.tsx`). Include `docs/design/historical-factions/portraits-swiss-german-48px.png` comparing old and new.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item A3 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## A4 · review-art-celebration-vignettes · Codex Luna · effort medium · hold

**Prepare the nine celebration vignettes for the small reward-card slot**

Release when: queue has room (fewer than 6 pending tasks).

Read `.claude/skills/legatum-art/SKILL.md` and the README beside the target assets first, and follow the series technique and delivery sizes exactly. Provide the prompt you used next to the source image. Do not edit TypeScript; wiring is a separate task.

Decision (user, 2026-10-18): wire the prepared celebration paintings in two stages; stage 1 is the small vignettes in the reward cards. The paintings already exist (see `app/src/assets/art/README.md` line ~60 and `docs/design/moment-vignettes/`). Prepare the nine square paintings for the card's picture slot (`app/src/celebrate/MomentLane.tsx` ~142-146: check the slot size): crop and resize so the subject stays legible at that size, no text, same technique. Write `app/src/assets/art/celebrations/README.md` with a table of which moment kind should use which image (read the moment kinds in `app/src/celebrate/`); do not edit TypeScript.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item A4 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## A5 · review-art-takeover-paintings · Codex Luna · effort low · hold

**Prepare three 16:9 takeover paintings for founded city, captured city and Great Work**

Release when: the small vignettes (A4) are wired and approved by the user.

Read `.claude/skills/legatum-art/SKILL.md` and the README beside the target assets first, and follow the series technique and delivery sizes exactly. Provide the prompt you used next to the source image. Do not edit TypeScript; wiring is a separate task.

Stage 2 of the celebration art (user decision, 2026-10-18). Prepare the three selected 16:9 takeover paintings (`docs/design/moment-vignettes/`) for the founded-city, captured-city and Great-Work moments: crop to 16:9, leave the text area readable (compose a mock card over each at 1000x600 and 1440x900 and save the comps in `docs/design/moment-vignettes/takeover-comps/`). No text in the paintings. Do not edit TypeScript.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item A5 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## A6 · review-art-terrain-increment · Codex Luna · effort medium · hold

**Terrain art increment: single trees, wood-edge strips, connected massif peaks**

Release when: .progress/resume/terrain-adopt.md work has landed and queue has room.

Read `.claude/skills/legatum-art/SKILL.md` and the README beside the target assets first, and follow the series technique and delivery sizes exactly. Provide the prompt you used next to the source image. Do not edit TypeScript; wiring is a separate task.

The adopted S2 organic terrain's queued next step (art skill, around line 164-169): single trees, edge-of-wood strips that keep a forest read at tile boundaries, and connected multi-hex massif peaks with saddles. Transparent, muted-earth pigments matching the S2 sprite palette and line weight, no sticker outlines, normal and far-zoom variants, with composites against plains and hills so boundaries can be checked. Put them in the folder the terrain README names. Renderer integration is a separate task.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item A6 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## A7 · review-art-keshig-base · Codex Luna · effort medium · hold

**Keshig: hooves planted on an integrated stone base**

Release when: astra/military-engineer-art and the other unit art branches have landed (unit art is under active work).

Read `.claude/skills/legatum-art/SKILL.md` and the README beside the target assets first, and follow the series technique and delivery sizes exactly. Provide the prompt you used next to the source image. Do not edit TypeScript; wiring is a separate task.

Unit art. The Mongol Keshig (`app/src/assets/units/keshig-256.png` and its derivatives) floats above a detached stone oval. Repaint it so at least one hoof touches a full-size stone plinth integrated under the horse, no detached foreground disc, subject centred and grounded, same series technique. Read the "Common rules for every figure" section of `.progress/astra-queue/done/unit-repaint-infantry.md` and follow it (deliverables: source, 256, 64, 48, panel, prompt; real alpha; check sheet at 45 px in colour and greyscale). Touch no other unit.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item A7 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.


## A8 · review-art-unit-sources · Codex Luna · effort low · hold

**Regenerate full-resolution sources for Spearman, Tank and Drone Swarm**

Release when: astra/military-engineer-art and the other unit art branches have landed.

Read `.claude/skills/legatum-art/SKILL.md` and the README beside the target assets first, and follow the series technique and delivery sizes exactly. Provide the prompt you used next to the source image. Do not edit TypeScript; wiring is a separate task.

Unit art. The Spearman, Tank and Drone Swarm sources are 256x256 while most unit sources are 1254x1254, so their 512 px panel derivatives (used by unlock hover above 80 px) are upscaled. Regenerate transparent full-resolution sources in the existing miniature style, keeping subject, pose and neutral stone base; make 1024 px or larger sources and regenerate the 256, 64, 48, card and 512 panel derivatives. Check no silhouette or base placement changes unintentionally. Follow the "Common rules for every figure" section of `.progress/astra-queue/done/unit-repaint-infantry.md`.

Rules for this task:
- Start from master. Make intermediate commits on your branch; do not merge or push.
- Run cargo, vitest, tsc and Playwright with the normal commands: the lane's shims send them to the cloud box. Never set a custom runs root.
- Do not touch files other agents are working in: crates/core battle and walls code, the AI siege genes, crates/ai/src/plan.rs and military.rs, and unit art under app/src/assets/units. Keep to the files named here. Mention anything else you notice in your final message instead of fixing it.
- This task is item A8 of docs/status/review-plan-2026-10-18.md. In your final message, say whether its "done when" line is met, with the command output that shows it.



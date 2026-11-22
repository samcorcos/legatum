> Archived 2026-11-22: kept for history; it no longer describes the project.

# Review plan, 2026-10-18

Thirteen Codex code reviews of the whole repo (`.reviews/astra-review-*.md`, read at master `bc68d6f79`) were triaged on 2026-10-18. This file is the durable checklist. **Tick a box only when the "done when" line is true, and write the landing commit next to it.** The user approved every group C recommendation on 2026-10-18 ("I accept all of your recommendations"), so section 1 is decided, not proposed.

- Codex items live in `.progress/astra-queue/pending/` (queued now) or `.progress/astra-queue/hold/` (waiting on a condition, listed in `hold/_INDEX-review-plan.txt`). The `.progress` folder is not in git, so the full prompts are also kept in `docs/status/review-plan-2026-10-18-prompts.md` (copy one back into `pending/` to requeue it).
- Claude items live in `docs/status/next-tasks.md` under "Review plan (approved 2026-10-18)".
- Owner types: **Codex Luna**, **Codex Sol**, **Claude Sonnet**, **Claude Opus**. Pace: at most about 4 Claude agents at once; hold files are released as their conditions are met, queue order follows file mtime.
- If you find this file after a context reset: run `ls .progress/astra-queue/pending .progress/astra-queue/hold`, check `.progress/astra.log` for finished `review-*` tasks, and continue from the first unticked item that is not blocked.

## 0. The findings that matter most (plain words)

1. Opening a damaged or very old save can crash the game later, because the solo/WASM load path never checks the save's structure (the LAN server does). CL1-CL3, D9.
1. The AI often picks a good plan and then does not carry it out: the chosen Campaign target is not the city the army marches to or the production funds, and Sparta rules overwrite the searched plan, sometimes with a zero-value one. CL7-CL9.
1. The AI's siege forecast is badly wrong in the lab: 23% of winnable fights are called hopeless and 34% of its "capture" calls had no winning policy. Re-measure after the Engineer lands, then recalibrate. CH16, CL10.
1. The app contains a second 18,900-line TypeScript copy of the rules and silently falls back to it when the Rust engine is missing. D1, stages M1-M4.
1. The AI evolution tool cannot start: 46 of the 114 decision genes are missing from its Python allowlist. CQ2.
1. Security and speed housekeeping: Vitest 5.0.2 sits inside published advisories (CQ1); scenario uploads to the LAN server have no size or count limit (CQ13); a killed verifier leaves a lock that stalls others for hours (CQ3); the worktree pool can hand one slot to two agents (CL5); the release check takes about 90 minutes of browser time with fixed worker counts (CQ12, CL19); the narrower WASM cache key sat unlanded on a branch for a week (CL6).
1. Docs contradict the code in places that mislead agents: the rules reference still has Catapults, bought Walls and an old decadence wording; Terraform READMEs state the wrong budget and cap; next-tasks lists finished work. CQ7, CQ8, CH3, CH4.
1. Open-source readiness: the review mostly re-found existing checklist items (no README, licence, architecture guide); new facts are in CL34.

## 1. Decisions (approved by the user, 2026-10-18)

Written into `legatum-design` (D1-D3, D4, D6-D11), `legatum-factions` (D5, heraldry part of D12), `legatum-art` (D12) and `legatum-workflow` (D13) the same day. Design-doc decisions-log rows are CL36.

### D1. Retire the former TypeScript client engine, in four stages

The app carries a second, 18,900-line copy of the game rules in TypeScript, and a broken build silently runs it instead of the real Rust engine. That breaks the one-engine rule. Stage 1 makes a missing engine an error. Stage 2 moves every rule the screens still compute (Found City blockers, attack forecasts, building facts) to the Rust engine. Stage 3 replaces the mock-built test data with data made by the Rust engine. Stage 4 deletes the mock and refuses to open old mock saves with a clear message.

Tasks: M1 (CQ5), M2a-M2d, M3a, M3b, M4.

### D2. Public release: curated export, fresh history, MIT OR Apache-2.0 for code

Publish from an allow-listed copy of the repo as a single new commit under a public identity; keep this repository private as the archive. Code is dual MIT OR Apache-2.0; art, music and fonts get a separate assets licence and a third-party notices file. The Civ 6 reference image is left out. The other open items (project name, art disclosure and the rest of that list) remain open.

Tasks: CL34, then the readiness plan.

### D3. Agent tooling does not ship publicly

Leave `.claude/`, `tools/astra`, `tools/ops`, `.progress/`, `.reviews/` and the private parts of CLAUDE.md out of the public export. Ship a short generic AGENTS.md (commands and code rules) and publish the code-verified rules reference as `docs/rules/`.

Tasks: CL34, readiness plan.

### D4. Cloud infrastructure stays out of the first public release

`infra/` and `tools/cloud/` add AWS cost and IAM work for contributors who only want to build and play. They stay in the private archive; a later release can add them as an optional example with configurable profile names and a no-AWS local path.

Tasks: CL34.

### D5. Every faction ability passes a one-sentence test

Each ability and each drawback must be one idea in about 30 words. Bundles are split or cut. The review found the worst bundles in Sakoku (Japan), Sparta (promotions, desertion, healing, slow rest), Switzerland (tolls plus interest, upkeep plus treasury loss), the Celts (concealment, ambush, territory damage) and the eight faction goals. Japan, Byzantium, the Mongols and Germany are already being reworked; the test applies to them when they land. Opus writes one page per faction; nothing changes in the rules without the user's playtest sign-off.

Tasks: CL31.

### D6. Every missing Civ 6 system gets a recorded reason

Trade routes, districts, religion, Great People, espionage, Builders, pillaging, naval units, map-based air play, tech trading, moving cities, optional razing, promotion trees and tile assignment are absent. Each gets a status (deliberate, planned or scope cut) and a one-sentence reason tied to a pillar. Where no reason was ever recorded, the question goes to the user instead of being invented.

Tasks: CL32.

### D7. An air strike on the turn you declare war is legal; rebases stay in order

Ground attacks already work on the turn war is declared; air strikes now match. Two aircraft still cannot swap between two full hangars (rebases apply in aircraft id order); the rules reference and the order text say so.

Tasks: CH2.

### D8. Old saves: Quick gets a notice, missing crown and hold clocks restart

A save made at the removed Quick speed opens at Standard with one plain line saying so; it is not converted. A save from before `title_since` and `vp_holds` existed restarts every crown clock and victory hold; this is accepted and written down in the rules reference.

Tasks: CH28, CH4.

### D9. A loaded save is checked, and rejected when it is broken

Opening a save validates its structure (map size, owners, references, counters) and its size; a broken one is refused with a message naming the field, and the running game and autosave are left alone. It is never silently repaired.

Tasks: CL1, CL2, CL3, CH27.

### D10. LAN play is for networks you trust; scenario uploads are capped, not locked

The LAN server uses plain HTTP and WebSocket, so seat tokens can be read by anyone on the network path. No TLS for now; the README and the join screen say so. Scenario-run uploads from other LAN devices keep working but are limited to 2 MiB each and 500 files or 100 MiB in total.

Tasks: CQ13.

### D11. A new unit order clears the unit's old standing order even if the new one fails

The review called this a bug. It is intentional (the code comment says so): a new order always replaces the old intent. No change.

Tasks: none.

### D12. Art decisions

Faction crests drop the fictional charges for historically grounded ones (the user approves the eight-symbol sheet first). The Future-era painting stays but loses the humanoid robots and drones. The Sparta/Athens and Switzerland/Germany portrait pairs are redone so they differ at 48 px. The prepared celebration paintings go into reward cards in two stages: small vignettes first, full-width takeovers later. The Keshig gets a planted base, and Spearman, Tank and Drone Swarm get full-resolution sources.

Tasks: CQ15, A1-A8, CL33.

### D13. Three test gates: fast change, extended, release

A normal change runs types, unit tests, the build and a small set of core Playwright flows. Extended adds the full Rust workspace and wider browser tests. The release gate adds the full two-size screen tour and WebKit. The tour is split into domains that can run alone.

Tasks: CL23, CL24, CH25.

## 2. Codex queue, running now (`.progress/astra-queue/pending/`, highest value first)

- [x] **CQ1** · Codex Luna · `pending/review-vitest-vite-bump.md` · Bump Vitest and Vite past their published advisories (landed ddb6c4822, 2026-10-20)
  - Done when: `npm audit` in app/ shows no vitest or vite finding; vitest 5.0.5 or later and vite 8.3.4 or later are locked; vitest, tsc and build pass.
- [x] **CQ2** · Codex Luna · `pending/review-evolve-gene-parity.md` · Evolution harness: add the 46 missing decision-gene ranges and a parity test (landed 62afc5936)
  - Done when: `python3 -m pytest tools/evolve` (or its unittest runner) passes with a new test that fails when `crates/ai/src/params.rs` and `tools/evolve/genome.py` disagree on any decision gene name or range.
- [x] **CQ3** · Codex Sol · `pending/review-verify-lock-recovery.md` · Verifier: a killed run must not leave a cache lock that stalls later runs for hours (closed: superseded by master's tools/cloud lockdir.sh, dead-holder-safe locks, 9aa5eafcb; the Codex branch is not landed)
  - Done when: `tools/astra/test-verify-cache.sh` has new cases (dead owner reclaimed within seconds, live owner left alone, SIGTERM of the owner removes its lock) and passes.
- [x] **CQ4** · Codex Sol · `pending/review-air-order-fixes.md` · Air rules: an invalid Fighter strike must not use up its interception, and a duplicate rebase must not reserve a hangar slot (landed by another agent, dca8532ff)
  - Done when: New tests in the core crate pass: (1) an enemy strike is still intercepted by a Fighter whose own strike order was invalid; (2) validation and resolution agree on the three-order rebase case. `cargo test --workspace` pins unchanged.
- [x] **CQ5** · Codex Luna · `pending/review-mock-fallback-guard.md` · Mock retirement stage 1: a missing WASM package is an error, not a silent switch to the TypeScript engine (landed 01a964687, 2026-10-20)
  - Done when: New vitest cases pass: no WASM and no the former client selector rejects with an actionable message; the former client selector still returns the mock; `?client=wasm` unchanged. The app shows the message on a plain error screen. The full vitest suite passes.
- [x] **CQ6** · Codex Luna · `pending/review-land-ff-script.md` · A safe landing command that fast-forwards master from the main checkout (landed 01a964687, 2026-10-20)
  - Done when: `bash tools/ops/tests/test-land.sh` passes (run under macOS /bin/bash 3.2).
- [x] **CQ7** · Codex Luna · `pending/review-docs-truth-infra-tooling.md` · Fix docs that contradict the code: Terraform defaults, runner header, Tauri README, perf README (landed 01a964687, 2026-10-20)
  - Done when: `git diff` shows only documentation and comment changes; each statement below matches the code it describes.
- [x] **CQ8** · Codex Luna · `pending/review-docs-superseded-drafts.md` · Mark two stale design drafts as superseded and make their links portable (landed 01a964687, 2026-10-20)
  - Done when: Both files open with a status note naming what is superseded and linking the current rules; no `/Users/` path remains in either.
- [x] **CQ9** · Codex Luna · `pending/review-error-toast-a11y.md` · The game error message gets a live-region role and a keyboard-reachable close button (landed c6b2cdb87, 2026-10-20)
  - Done when: A vitest test finds the message by `role="alert"`, finds a button named "Dismiss", and activating it with Enter clears the error. Existing Playwright specs that dismiss the toast still pass.
- [x] **CQ10** · Codex Luna · `pending/review-app-small-cleanups.md` · Three small app cleanups: one view call less on load, internal-only exports, no wall-clock assertion (landed 01a964687, 2026-10-20)
  - Done when: New vitest asserts `client.view` is called once per human seat on load (not once more); the four exports are no longer exported; the 5 ms assertion is gone; vitest and tsc pass.
- [x] **CQ11** · Codex Luna · `pending/review-cities-overview-tests.md` · Behaviour tests for the Cities overview and Where to build (landed 01a964687, 2026-10-20)
  - Done when: New test files pass and cover every case listed; no production code changed except accessible names where a test needs them.
- [ ] **CQ12** · Codex Luna · `pending/review-release-check-workers.md` · Release check: load-aware Chromium workers and a configurable WebKit worker count
  - Done when: `bash tools/ops/tests/test-pw-workers.sh` passes; `tools/release/check.sh` and `webkit-docker.sh` use the helper and the new env knob (read the diff; do not run a release check).
- [x] **CQ13** · Codex Sol · `pending/review-server-scenario-run-cap.md` · LAN server: bound scenario-run uploads (size and total retention) and say plainly that LAN play is for trusted networks (landed 01a964687, 2026-10-20)
  - Done when: New server tests pass: an oversized upload is rejected; the 501st stored run evicts the oldest; the README states the trusted-network assumption.
- [ ] **CQ14** · Codex Luna · `pending/review-perf-baseline-refresh.md` · Fresh simulator speed baseline on the cloud box, written into the perf ledger
  - Done when: `docs/sim-results/perf/README.md` has a dated 2026-10-18 section with commit, toolchain, load and numbers for each mode, and the "Quick" label is fixed.
- [x] **CQ15** · Codex Luna · `pending/review-heraldry-concept-sheet.md` · Heraldry: concept sheet of eight historically grounded charges for the user to approve (landed 01a964687, 2026-10-20)
  - Done when: A labelled contact sheet and a one-line rationale per faction exist under `docs/design/heraldry/`; no app files changed. The user approves the eight symbols before anything is finalised.
- [x] **CQ16** · Codex Luna · `pending/review-random-faction-glyph.md` · A real icon for the Random faction instead of a text question mark (landed 9b463cea1)
  - Done when: The icon exists as `ui-random.svg`, is registered like the other icons, the picker uses it, the icon tests and distinctness check pass.

## 3. Codex queue, held (`.progress/astra-queue/hold/`, each with the condition that releases it)

- [ ] **CH1** · Codex Luna · `hold/review-plan-rs-dead-code-notes.md` · Remove four stale dead-code allowances and the "currently unused" comments in the AI campaign forecast
  - Release when: the AI war follow-through work (plan.rs, military.rs) has landed on master.
  - Done when: `cargo clippy --workspace` is clean with the allowances removed; the comment and the learnings note describe the current callers.
- [ ] **CH2** · Codex Sol · `hold/review-air-surprise-strike.md` · Air strikes on the turn war is declared: make validation agree with resolution (allowed, like ground attacks)
  - Release when: review-air-order-fixes (CQ4) has landed (same file).
  - Done when: A `DeclareWar` plus `AirStrike` at Peace test passes through both `validate_orders` and `resolve_turn`; the rules reference air section and the Fighter/Bomber order availability say the same.
- [x] **CH3** · Codex Luna · `hold/review-docs-status-reconcile.md` · Reconcile docs/status/next-tasks.md and tracker.md with what has landed (landed 2026-10-21; still-open era, user-question and worktree-pool items kept in next-tasks)
  - Release when: the Military Engineer has landed on master and the lead has no pending edits to the status files.
  - Done when: No completed item remains in the active lists; removed history is in `docs/status/archive/2026-10.md`; the review-plan link is still at the top of both files.
- [ ] **CH4** · Codex Sol · `hold/review-rules-reference-reconcile.md` · Make the rules reference match the code: siege units, bought Walls, decadence on the mean, order summary
  - Release when: claude/military-engineer and astra/walls-no-buy have both landed on master.
  - Done when: Every item below is checked against code and the reference matches it; the diff cites the code location for each change.
- [ ] **CH5** · Codex Luna · `hold/review-rules-reference-split.md` · Split the 2,100-line rules reference into topic files with an index
  - Release when: review-rules-reference-reconcile (CH4) has landed.
  - Done when: The legatum-rules skill points at an index; every old section exists in exactly one topic file; no text is lost (word count before and after matches).
- [ ] **CH6** · Codex Luna · `hold/review-learnings-split.md` · Archive old learnings by month and keep docs/learnings.md short
  - Release when: the lead has no open edits to docs/learnings.md (check with the lead; many agents append to it).
  - Done when: `docs/learnings.md` holds the current month and an index; older months are in `docs/learnings/YYYY-MM.md`; the learnings Stop hook still works.
- [ ] **CH7** · Codex Luna · `hold/review-skills-trim.md` · Shrink the five largest skills to short entry points with focused reference files
  - Release when: the Military Engineer, Germany and scenario-lab branches have landed (they edit these skills) and the lead agrees nobody is mid-edit.
  - Done when: Each SKILL.md is under 150 lines; every moved section lives in a `references/` file the SKILL.md links; no content deleted.
- [ ] **CH8** · Codex Luna · `hold/review-split-tests-production-view.md` · Move the embedded test modules out of production.rs and view.rs
  - Release when: claude/military-engineer, eu4-wars and the war follow-through have landed (they edit view.rs, production.rs and the AI crate).
  - Done when: `cargo test --workspace` passes with the same test count; the two source files are shorter by the size of their test modules.
- [ ] **CH9** · Codex Luna · `hold/review-split-styles-css.md` · Split app/src/styles.css by feature, keeping the cascade identical
  - Release when: no open UI branch edits styles.css (Military Engineer, Germany, scenario-lab, play-time landed).
  - Done when: The built CSS after minification is byte-identical (or differs only in whitespace and comments) before and after; the screen tour passes.
- [ ] **CH10** · Codex Luna · `hold/review-split-format-ts.md` · Split app/src/format.ts by responsibility without changing any wording
  - Release when: Military Engineer and Germany branches have landed (they edit format.ts).
  - Done when: The existing format tests pass unchanged; `format.ts` re-exports so no importer changes.
- [ ] **CH11** · Codex Luna · `hold/review-split-types-ts.md` · Split app/src/types.ts into view, order, event and forecast modules behind a barrel
  - Release when: Military Engineer and Germany branches have landed (both edit types.ts).
  - Done when: tsc and vitest pass; no importer changed; serialized field names untouched.
- [ ] **CH12** · Codex Luna · `hold/review-split-lobby.md` · Extract the seat table and settings controls from LobbyScreen
  - Release when: no open branch edits the lobby.
  - Done when: Lobby specs and layout specs pass; `LobbyScreen.tsx` is under 350 lines.
- [ ] **CH13** · Codex Sol · `hold/review-split-store.md` · Split the Zustand store implementation into slices while keeping one exported hook
  - Release when: astra/play-time and claude/scenario-lab have landed (both edit store.ts) and format/types splits are done.
  - Done when: The `useGame` API is unchanged; the full vitest and Playwright suites pass; `store.ts` is under 400 lines.
- [ ] **CH14** · Codex Sol · `hold/review-split-citypanel.md` · Split CityPanel into production, territory and air-wing sections
  - Release when: the CityPanel is not being edited by an open branch and the store split (CH13) has landed.
  - Done when: No visible change: city-panel specs, layout spec and the screen tour pass; `CityPanel.tsx` is under 350 lines.
- [ ] **CH15** · Codex Sol · `hold/review-split-mapcanvas.md` · Separate MapCanvas input, camera and frame-scheduling concerns
  - Release when: the CityPanel split (CH14) has landed and the selection-pulse work (CL17) has not started in MapCanvas.
  - Done when: All map specs (pan, zoom, keys, minimap, selection) pass; `MapCanvas.tsx` is under 600 lines; one camera owner.
- [ ] **CH16** · Codex Luna · `hold/review-combat-lab-rerun.md` · Re-run the combat lab on the current rules and record how wrong the siege forecast is now
  - Release when: the Military Engineer has landed (it changes siege rules).
  - Done when: `docs/sim-results/combat-lab/` has a dated results section for current master with forecast-versus-actual tables, including 4, 5, 6 and 8 round windows.
- [ ] **CH17** · Codex Luna · `hold/review-scripted-diagnosis-transform.md` · Make production_diagnosis use the same final order transform as the live AI
  - Release when: claude/eu4-wars has landed (it edits scripted.rs).
  - Done when: A same-turn declaration fixture shows diagnostic and live orders identical; `cargo test -p legatum-ai` passes.
- [ ] **CH18** · Codex Sol · `hold/review-runner-lane-setup-errors.md` · Astra runner: a failed lane setup parks the task with a clear message instead of failing later
  - Release when: the offload-lanes agent has finished (it edits runner.sh).
  - Done when: A test with a stubbed failing `git worktree add` shows the task parked with the diagnostic; `bash -n tools/astra/runner.sh` passes.
- [x] **CH19** · Codex Luna · `hold/review-verify-remote-timeout-budget.md` · Remote checks: one timeout budget that covers queueing and running, with a final status always printed (landed 2026-10-21, merged as 0fafa698c)
  - Release when: the verify-speed agent has finished (it edits the verification scripts) and CQ3 has landed.
  - Done when: A test with a short budget shows a clear "timed out after queueing/running" line with the job id; the client never times out before the remote job reports.
- [ ] **CH20** · Codex Luna · `hold/review-verify-dedupe-tsc-server.md` · Verifier: stop type-checking twice and reuse an unchanged release server
  - Release when: CQ3 (verify lock recovery) has landed (same file).
  - Done when: Timing on an app-only branch shows the second TypeScript pass gone and the server step a cache hit; type errors are still caught.
- [ ] **CH21** · Codex Luna · `hold/review-worktree-lifecycle-doc.md` · One page on the three worktree ownership modes (lanes, agent pool, verifier trees)
  - Release when: the worktree-pool ownership fix (CL5) has landed.
  - Done when: `docs/ops/worktrees.md` exists and is linked from the legatum-ops skill; each statement is checked against the scripts.
- [x] **CH22** · Codex Luna · `hold/review-tools-index.md` · An index of tools/ and scripts/ that separates supported commands from historical one-offs (landed 2026-10-21; eight newer tool directories added to the index)
  - Release when: claude/oss-readme has landed (it adds README files).
  - Done when: `tools/README.md` lists every directory and top-level script with one line and a status; historical scripts link their source report.
- [x] **CH23** · Codex Luna · `hold/review-slot-queue-bash5.md` · Run the box queue regression test under the box's Bash 5 and print the version (landed aadbe33b1, 2026-10-20)
  - Release when: the lane is idle and the box is not busy with verifies.
  - Done when: The test prints its Bash version and passes on the box.
- [x] **CH24** · Codex Luna · `hold/review-audit-deps-script.md` · A repeatable dependency advisory check for both Rust lockfiles and the npm lockfile (landed 2026-10-21; fixed to fail when cargo-audit is missing and to find vulnerable markers; tested on the box, cargo-audit installed per job, not on the box image)
  - Release when: claude/oss-readme has landed and the lead has OK for installing cargo-audit on the box.
  - Done when: `tools/ops/audit-deps.sh` exits non-zero on a known-vulnerable crate or package and prints unmaintained crates separately.
- [x] **CH25** · Codex Luna · `hold/review-tour-split.md` · Split screen-tour.spec.ts into selectable domains with separate screenshot directories (landed aadbe33b1, 2026-10-20)
  - Release when: the fast Playwright profile design (CL23) has landed.
  - Done when: Each domain runs on its own (`npx playwright test e2e/tour/<domain>`), screenshots land in distinct directories, and the union of shots equals the old tour (same count).
- [ ] **CH26** · Codex Luna · `hold/review-keys-minimap-specs.md` · Keyboard and minimap specs assert what the player sees, not camera internals
  - Release when: the map input split (CH15) has landed.
  - Done when: Both specs pass 10 times in a row on the box with no fixed sleeps.
- [ ] **CH27** · Codex Luna · `hold/review-save-size-limit-ui.md` · Reject oversized save files before reading them in the desktop picker and autosave
  - Release when: CL1 (save integrity at load) has chosen the limit and landed.
  - Done when: Vitest shows a clear "save too large" error before `FileReader`/read; the limit constant is shared with the Rust parser (CL1).
- [ ] **CH28** · Codex Sol · `hold/review-quick-save-notice.md` · Opening an old Quick-speed save tells the player the game now plays at Standard
  - Release when: CL1 (save integrity at load) has landed (same parse path).
  - Done when: A fixture save with `"Quick"` loads and the load result carries a flag; the UI shows one plain line; costs follow Standard (unchanged).
- [ ] **M3b** · Codex Luna · `hold/review-mock-fixtures-migrate.md` · Mock retirement stage 3b: migrate UI tests from the mock to core-generated fixtures, one batch per task
  - Release when: M3a (fixture format and generator) has landed. Re-queue until no test outside retired TypeScript engine imports the mock.
  - Done when: The listed tests pass reading fixtures made by the Rust core; no test imports retired TypeScript engine; the migrated tests are deleted from the mock suite.
- [ ] **M4** · Codex Luna · `hold/review-mock-delete.md` · Mock retirement stage 4: delete retired TypeScript engine and refuse old mock-format saves with a clear message
  - Release when: stages M1, M2a-M2d, M3a and M3b are all done and the lead has confirmed.
  - Done when: the duplicate engine is gone; source imports point to WASM-backed helpers; opening a save from the retired engine shows a plain refusal; vitest, tsc, build and Playwright pass.
- [ ] **A1** · Codex Luna · `hold/review-art-future-era.md` · Repaint the Future-era painting without humanoid robots
  - Release when: queue has room (fewer than 6 pending tasks).
  - Done when: New `future-1280.png` and derivatives in the series style; no robots, drones or megastructures; contact sheet next to the neighbouring eras.
- [x] **A2** · Codex Luna · `hold/review-art-greek-portraits.md` · Sparta and Athens portraits that differ at 48 px (landed 2026-10-21)
  - Release when: queue has room (fewer than 6 pending tasks).
  - Done when: A thumbnail comparison sheet at 48 px and 96 px shows the two pairs are distinguishable by silhouette and value pattern.
- [x] **A3** · Codex Luna · `hold/review-art-swiss-german-portraits.md` · Switzerland and Germany portraits that differ at 48 px (landed 2026-10-21; the two new masters were in each other's files, swapped to match their prompts)
  - Release when: queue has room (fewer than 6 pending tasks).
  - Done when: A thumbnail comparison sheet at 48 px and 96 px shows the two pairs are distinguishable.
- [ ] **A4** · Codex Luna · `hold/review-art-celebration-vignettes.md` · Prepare the nine celebration vignettes for the small reward-card slot
  - Release when: queue has room (fewer than 6 pending tasks).
  - Done when: Nine square images at card-slot sizes with a crop sheet at the real slot size, and a mapping table of moment kind to image.
- [ ] **A5** · Codex Luna · `hold/review-art-takeover-paintings.md` · Prepare three 16:9 takeover paintings for founded city, captured city and Great Work
  - Release when: the small vignettes (A4) are wired and approved by the user.
  - Done when: Three 16:9 images with a text-safe area and screen comps at 1000x600 and 1440x900.
- [ ] **A6** · Codex Luna · `hold/review-art-terrain-increment.md` · Terrain art increment: single trees, wood-edge strips, connected massif peaks
  - Release when: .progress/resume/terrain-adopt.md work has landed and queue has room.
  - Done when: Transparent sprites in normal and far-zoom variants plus composites over plains and hills for review.
- [ ] **A7** · Codex Luna · `hold/review-art-keshig-base.md` · Keshig: hooves planted on an integrated stone base
  - Release when: astra/military-engineer-art and the other unit art branches have landed (unit art is under active work).
  - Done when: 45 px crops in colour and greyscale on grass, desert and hills show the horse grounded; no detached disc.
- [ ] **A8** · Codex Luna · `hold/review-art-unit-sources.md` · Regenerate full-resolution sources for Spearman, Tank and Drone Swarm
  - Release when: astra/military-engineer-art and the other unit art branches have landed.
  - Done when: Each has a 1024 px or larger source and all derivatives regenerated; side-by-side 512 px panels with the old ones.

## 4. Claude items (also in `docs/status/next-tasks.md` under "Review plan (approved 2026-10-18)")

### Retire the former TypeScript client engine (staged)

- [ ] **M2a** · Claude Sonnet · next-tasks.md > Review plan > Retire the former TypeScript client engine · Stage 2a: Found City blocker and settlement scoring come from the core
  - What: Route map hover, UnitPanel, command enablement and report wording for Found City through one core-backed blocker (WASM and LAN server query; the pattern is `crates/wasm/src/lib.rs` ~288). Remove the TS blocker in `app/src/commands.ts` ~238 and the production use of the former TypeScript engine (MapCanvas ~171). Keep the distinction between known blockers and a hidden city that can still defeat an order.
  - Done when: No production file imports the retired settlement module; LAN and local games show the same blocker text; vitest and settle-lens Playwright specs pass.
  - Start after: M1 has landed
- [ ] **M2b** · Claude Sonnet · next-tasks.md > Review plan > Retire the former TypeScript client engine · Stage 2b: attack forecasts and strength cues come from the core
  - What: Replace `app/src/combat.ts` (mirrors Rust damage and strength), its callers `aftermath.ts`, `siege.ts` ~152 and `map/unitEra.ts` ~25 with core-provided forecast/strength fields on `PlayerView` (or a core query). Presentation geometry may stay in TS; rule maths may not.
  - Done when: `combat.ts` rule maths is deleted; forecast card, aftermath and unit chips read core numbers; WASM parity tests no longer needed for these.
  - Start after: Military Engineer has landed (it edits combat.ts)
- [ ] **M2c** · Claude Sonnet · next-tasks.md > Review plan > Retire the former TypeScript client engine · Stage 2c: building facts come from the generated rules snapshot
  - What: The repo already checks `rules.generated.json` against Rust (`crates/ai/tests/ui_rules.rs`) but `app/src/data.ts` ~189 and `buildingStatsFor` ~968 hand-maintain cost, yields, requirements, strength and faction overrides. Read mechanical fields from `RULES.buildingStats` (extend the export with predecessor and population requirements and faction overrides); keep prose in TS. Add a test that every building id the UI uses exists in the export.
  - Done when: No mechanical building number is hand-written in `data.ts`; research timing and unlock cards read the export; export-parity test passes.
  - Start after: Military Engineer and Germany have landed (both edit data.ts)
- [ ] **M2d** · Claude Sonnet · next-tasks.md > Review plan > Retire the former TypeScript client engine · Stage 2d: inventory and replace the remaining production imports of retired TypeScript engine
  - What: Write the checked list of every production import from the former TypeScript engine (settlement scoring, terrain yields, threat estimates, title backdrop generation, auto-explore, others), mark fixture-only imports, then replace each with a core-backed view field or move genuinely UI-only code (title backdrop) out of the retired engine directory.
  - Done when: `grep -rn "from .*mock" app/src --include=*.ts --include=*.tsx` outside the former TypeScript engine and test files returns nothing.
  - Start after: M2a-M2c
- [ ] **M3a** · Claude Sonnet · next-tasks.md > Review plan > Retire the former TypeScript client engine · Stage 3a: core-generated UI fixtures (format and generator)
  - What: Design a small deterministic fixture format built from a Rust `PlayerView`, events and optional orders, with a Rust generator (an example or `legatum-sim` subcommand) and a check that committed fixtures match current core serialization. Migrate a representative set of map, combat, diplomacy and turn-flow tests as proof, then write the README the Luna migration task (M3b) follows.
  - Done when: Fixtures regenerate byte-identically; 10 representative tests migrated; README written; M3b released.
  - Start after: M2d

### AI execution coherence (from the AI review; each gated by the AI-impact gate: seat-only and opponents-only vs master, war suite)

- [ ] **CL7** · Claude Opus (design) then Sonnet · next-tasks.md > Review plan > AI execution coherence · Route the chosen Campaign into orders: one shared objective
  - What: The planned city comes from the routed Campaign only for Sparta with `planned_stage`; for others military staging falls through to a conquest nominee while the plan names another city, and production funds only `conquest_city` or containment (`military.rs` ~2911-2962, `production.rs` ~699). Define one `CampaignObjective` (target, opponent, stage, required force, funding need) consumed by military, production and diplomacy, with a log of target to staging to builds to declaration to first attack. Regression position: a non-Sparta Campaign that differs from the routine nominee; orders must pursue the chosen city.
  - Done when: Regression test passes; paired benchmark shows target-to-first-attack delay down and no seat-only loss.
  - Start after: AI war follow-through has landed (plan.rs, military.rs)
- [ ] **CL8** · Claude Opus (design) then Sonnet · next-tasks.md > Review plan > AI execution coherence · Sparta overrides: price them in the plan instead of replacing it after search
  - What: `sparta_force_campaign` picks a Campaign regardless of value after search, a follow-up rule can replace it again, both logged as "Better"; the generator can synthesise a zero-value "forced expedition" (`plan.rs` ~1615-1700, ~2070-2110). Also `restless_far_war` declares a distant war under a Peace plan (`diplomacy.rs` ~1347). First instrument searched-vs-executed plan divergence; then price capture healing, desertion avoidance and chain value inside candidate valuation with a feasibility floor.
  - Done when: Divergence telemetry in place; zero-value Campaign switches near zero; Sparta war scenarios (Nicaea Strike) keep their success; paired runs gate passes.
  - Start after: CL7
- [ ] **CL9** · Claude Opus (design) then Sonnet · next-tasks.md > Review plan > AI execution coherence · Conquest target shortlist priced with the same terms as plans
  - What: The routine nominee is unavailable to Germany and to non-Sparta factions with fewer than two cities, early Sparta absolutely excludes walled cities and Celts owners, and Switzerland reject optional conquest below a 45% VP threshold (`conquest.rs` 53-110, `params.rs` ~179). Build a shortlist of legal reachable cities, price the best few with the capture, economy, title and risk terms plans use; keep only impossible actions as hard filters.
  - Done when: Candidate recall and captures-by-turn-50 measured against successful human/best-policy attacks and improved; faction regressions checked.
  - Start after: CL7, CL8
- [ ] **CL10** · Claude Opus · next-tasks.md > Review plan > AI execution coherence · Recalibrate siege and assault forecasts from the combat lab
  - What: Use the CH16 results to fit capture probability, expected losses and time to capture by walls, composition, terrain and era; compare 4/5/6/8-round windows jointly with the assault policy. A wider window alone adds false positives.
  - Done when: Held-out calibration improves; captures per war up without more failed assaults; seat-only benchmark not worse.
  - Start after: CH16 results exist
- [ ] **CL11** · Claude Opus · next-tasks.md > Review plan > AI execution coherence · Give Contain an executable outcome
  - What: Containment can award value for coalition strength with no credible attack (`plan.rs` ~1224-1280, ~2780; 16.8% of wars have no battle). Each Contain candidate names how it lowers the leader's win chance (funded capture, viable raid, measurable denial) and is reviewed after declaration. The blanket funded-route gate is a control, not the answer (it helped the leader in a scenario).
  - Done when: Wars without a battle, battle within 10 turns, leader cities captured and coalition timing measured against the control; leader win rate not up.
  - Start after: CL7-CL9 (merges with the "real coalitions" brief below)
- [ ] **CL12** · Claude Opus · next-tasks.md > Review plan > AI execution coherence · Peace/redeclare loops: compare peace with continuing the named war
  - What: On Campaign termination keep the named war as a priced short option against peace using remaining targets, army condition and retaliation (36% of examined peaces reversed within 10 turns; Mongol peace then redeclare 4 turns apart). Related existing item: "Honest peace pricing" in section 3.
  - Done when: Peace-then-war within 10 turns down; futile-war duration not up; paired win rate not down.
  - Start after: CL11
- [ ] **CL13** · Claude Sonnet · next-tasks.md > Review plan > AI execution coherence · Production provenance: log which gate chose each item and the runner-up
  - What: Emergency, opening, goal and faction rules return items before `item_value` and its genes (`production.rs` 442, 1215, 1456; Sparta assault wing 1335; Switzerland Market bonus 99). Add the log first, then turn faction bonuses and plan commitments into comparable scores keeping hard survival gates.
  - Done when: Provenance log for every build decision; a new Campaign visibly changes the required army; no rise in abandoned near-complete queues.
  - Start after: CL7
- [ ] **CL14** · Claude Sonnet · next-tasks.md > Review plan > AI execution coherence · Gene activation telemetry for all decision genes
  - What: Count, per gene, eligible branch, actual branch entry and changed winner in a controlled ablation, so dormant, masked and influential genes can be told apart before the evolution harness runs.
  - Done when: A ledger table of all decision genes with the three counts exists in `docs/sim-results/ai-quality/`.
  - Start after: CQ2 has landed
- [ ] **CL15** · Claude Opus · next-tasks.md > Review plan > AI execution coherence · Search: measure candidate recall, refit win chance and hold hazard on current rules
  - What: Search only sees three candidates after the analytic generator filters; the win-chance weights were fit on 10-04/10-06 rules (`winchance.rs` 43-61). Measure whether the winning human/best-policy objective is in the top three, refit on held-out games, test adaptive sample allocation under the WASM turn budget.
  - Done when: Plan-choice regret on saved decisive positions reported; paired seat-only/opponents-only results; latency budget kept.
  - Start after: CL7-CL12

### Core correctness at the save boundary

- [ ] **CL1** · Claude Sonnet · next-tasks.md > Review plan > Core correctness at the save boundary · Validate a loaded save: integrity, size limit and hostile-save fixtures
  - What: `parse_state` (`crates/wasm/src/json.rs` 68) checks the version only; the server already has a structural check (`crates/server/src/host.rs` `check_state` ~997) but the solo/WASM path does not, so a shortened tile vector or bad owner id loads and panics later. Lift `check_state` into core (`GameState::validate_structure`, field-specific errors), call it from `parse_state` and the server; add a documented maximum save size enforced in Rust (UI half is CH27); hostile-save fixtures (shortened tiles, bad owner, oversized fields, malformed embedded archive and AI memory) asserting rejection before expensive parsing and that a failed load keeps the live game and the committed autosave.
  - Done when: All committed `.legatum` fixtures still load (WASM old-saves regression test); each hostile fixture is rejected with a field-specific message and no panic; pins unchanged.
  - Start after: Military Engineer has landed (touches state)
- [ ] **CL2** · Claude Sonnet · next-tasks.md > Review plan > Core correctness at the save boundary · Checked arithmetic on persisted numbers
  - What: Audit treasury (`cities.rs` `pay_gold` ~1043), production progress (~1017), tribute quote (`diplomacy/tribute.rs` ~130), `overreach_dry_in` negation (`view.rs` ~1429), turn counters. Use wider intermediates where legitimate totals can be large, reject impossible saved values at load (with CL1). Boundary tests at `i32::MAX`, `i32::MIN`, `u32::MAX`.
  - Done when: Boundary tests pass; no panic or wrap on a save with extreme values; pins unchanged.
  - Start after: CL1
- [ ] **CL3** · Claude Sonnet · next-tasks.md > Review plan > Core correctness at the save boundary · Fallible new-game setup
  - What: `new_game` (`game.rs` ~133) calls map generation that panics after failed attempts and `Map::new` multiplies unvalidated dimensions. Validate player count and dimensions, expose a `Result`, keep the panic only as an internal invariant. Update callers (sim, server, wasm).
  - Done when: Zero, negative, excessive and infeasible setups return errors in tests; no caller panics.
  - Start after: CL1

### Tooling and ops

- [ ] **CL5** · Claude Sonnet · next-tasks.md > Review plan > Tooling and ops · Worktree pool: real ownership instead of age-based expiry
  - What: `tools/ops/agent-worktree.sh` frees a lock after 12 hours regardless of owner and handed `pool-1` to two agents on 2026-10-18. PID liveness is NOT an option (it wiped two agents' work on 10-09: the PID is a one-shot shell). Design: an owner token returned by `acquire` and required by `release`; a slot with unmerged commits or uncommitted changes never expires; stale detection by branch merged-or-gone plus age. Regression: an owner active past the old timeout keeps its slot; release by a wrong token fails; simultaneous acquire gets distinct slots.
  - Done when: Tests for those cases pass; the learnings entry for the double assignment links the fix.
- [x] **CL6** · Claude Sonnet (landing) · next-tasks.md > Review plan > Tooling and ops · Land astra/wasm-cache-key-narrow and make cache hits restore the WASM package (landed aadbe33b1, 2026-10-20)
  - What: The branch (1 commit, e4f977998, 7 days old) narrows the WASM cache key to the compiled crates but never landed; master still hashes all of `crates/`. Merge master into it, gate it, land it. Then make a verifier cache hit restore and hash-check `app/src/wasm-pkg` (a PASS record alone does not give a fresh worktree the package).
  - Done when: A sim-only edit hits the cache with a byte-identical package; a core/ai edit misses.
- [x] **CL19** · Claude Sonnet · next-tasks.md > Review plan > Tooling and ops · Measure and choose release browser worker counts (landed aadbe33b1, 2026-10-20)
  - What: After CQ12 lands, run the unchanged Chromium and WebKit release suites on the box at several worker settings (WebKit 1, 2, 3 inside its 4 CPUs); record wall time, CPU time, test and screenshot counts and failures; set the fastest stable defaults.
  - Done when: Defaults chosen from a recorded table; release check wall time down with identical coverage.
  - Start after: CQ12
- [x] **CL23** · Claude Sonnet · next-tasks.md > Review plan > Tooling and ops · Three-level test gate: fast PR, extended, release/visual (landed aadbe33b1, 2026-10-20)
  - What: Decision approved 2026-10-18. Define the profiles: fast PR (tsc, vitest, build, a small Playwright smoke set: launch/continue, a production choice, end turn, research, save/reload), extended (full Rust workspace and broader Playwright), release/visual (full two-size screen tour and WebKit). Wire the remote verifier and document triggers and failure policy in `legatum-workflow`. Measure per-binary Rust durations and label long AI/sim/property suites as extended without deleting tests.
  - Done when: Verifier takes `--profile`; app-only changes run the fast profile by default; policy documented.
- [x] **CL24** · Claude Sonnet · next-tasks.md > Review plan > Tooling and ops · Playwright worker cap and one bounded rerun for browser transport errors (landed aadbe33b1, 2026-10-20)
  - What: Archived remote runs use 21 workers and show `ERR_NETWORK_CHANGED` and timing failures across unrelated branches. Set a measured cap, record error class and worker count in the summary, and rerun only the failed shard once for transport errors, reporting the first failure and the rerun.
  - Done when: Summary shows error classes; assertion failures are never rerun silently.
  - Start after: CL23
- [x] **CL25** · Claude Sonnet · next-tasks.md > Review plan > Tooling and ops · Triage the reward-emblem spec failures (landed aadbe33b1, 2026-10-20)
  - What: Archived runs fail `reward-emblems.spec.ts` ("milestone card", "ruin-gold card"). Separate fixture-import and transport failures from assertion failures, stabilise the fixture loading through a helper, keep one layout shot per card family plus text/action/dismiss assertions.
  - Done when: The spec passes 5 times in a row on the box.
- [ ] **CL37** · Claude Sonnet (landing) · next-tasks.md > Review plan > Tooling and ops · Land the Codex outputs, with screenshots for UI ones
  - What: Every `astra/review-*` branch is reviewed by a Sonnet landing agent, never merged on Codex's word. UI items (CQ5, CQ9, CQ16, CH9, CH12-CH15, CH25, CH26) need Playwright screenshots at 1000x600 and 1440x900 first. Land with `tools/ops/land.sh` once CQ6 exists, otherwise `git merge --ff-only` in the main checkout.
  - Done when: Each finished branch is landed or sent back with a reason; the checkbox in the plan is ticked with the landing commit.
  - Start after: each Codex task finishing

### Performance

- [ ] **CL17** · Claude Sonnet · next-tasks.md > Review plan > Performance · Selection pulse on its own transparent layer
  - What: A selection redraws the whole map every 33 ms (`MapCanvas.tsx` ~699, 737) at device pixel ratio 2 (about 5 million pixels). Measure selected-idle frame work at 1000x600 and 1440x900, DPR 1 and 2 on the Retina Mac (`window.legatumMap.perf`), move the pulse to a separate layer, repeat, compare screenshots for selection, movement, siege and celebrations.
  - Done when: Full-map draw frequency falls while idle with identical screenshots.
  - Start after: CH15 has landed
- [ ] **CL18** · Claude Sonnet · next-tasks.md > Review plan > Performance · Terrain update: reuse regions when only the view object changed
  - What: `TerrainCache.update()` computes organic regions before checking tile signatures (`terrain.ts` ~247) and rebuilds fields on any dirty tile (~285-327), about 45-65 ms (terrain README). Instrument phases, reuse unchanged inputs, prototype dirty-region updates for soft fields, keep fog correct.
  - Done when: Phase timings before/after on three fixed saves; screenshots identical; no stale art after a turn.
- [ ] **CL21** · Claude Sonnet · next-tasks.md > Review plan > Performance · Terrain cache memory budget across seats and fade modes
  - What: Each cache keeps 110+ chunks of 512x512 (about 1 MiB each) and there are separate caches per seat and variant, so two seats hold about 220 MiB. Put one byte budget across all caches; compare repaint cost against keeping a second fade variant.
  - Done when: Measured memory during hot-seat switches and lens toggles stays under the budget; no old fog or borders appear.
  - Start after: CL18
- [ ] **CL22** · Claude Opus · next-tasks.md > Review plan > Performance · Route-search hotspot in the AI, after the fresh baseline
  - What: After CQ14 gives a current profile (70% of an 8-player turn was scripted planning in the 10-06 ledger), find equivalent repeated route searches by caller in `movement.rs` and `campaign.rs`, and share a per-seat route field where passability is equivalent. Digests must not change.
  - Done when: Game, search-decision and view digests identical; measured speed-up reported.
  - Start after: CQ14
- [ ] **CL20** · Claude Sonnet · next-tasks.md > Review plan > Performance · Cold-load trace on the Mac app
  - What: Time title paint, terrain sprite readiness, WASM fetch/compile, first game creation and first view, cold and warm, in a current Tauri build (the 6.4 MB WASM and the sprite wait before mount in `main.tsx` ~17 are the suspects). Needs the user's Mac: ask first, build off-Mac.
  - Done when: Numbers recorded in `docs/sim-results/perf/`; a fix only if the click-to-game delay is material.
  - Start after: user available

### Architecture, docs and public release

- [ ] **CL26** · Claude Sonnet · next-tasks.md > Review plan > Architecture, docs and public release · Target-neutral session crate
  - What: Native crates (sim, server, archive, analysis) depend on `legatum-wasm` for `Session`, tape and recording code. Write the target Cargo graph first, move those modules into `legatum-session` without changing serialized bytes or hashes, repoint crates one at a time, leave `legatum-wasm` as the thin browser binding.
  - Done when: Cross-target hash checks and old-saves test pass; the dependency graph is documented.
  - Start after: Military Engineer has landed
- [ ] **CL27** · Claude Sonnet · next-tasks.md > Review plan > Architecture, docs and public release · TypeScript wire contract generated or checked from Rust
  - What: Start with stable enum unions (faction ids etc.) and one representative `PlayerView`, `Order` and `Event` payload check against the TS types; keep decimal-string u64 handling. Expand only after the first slice is reliable.
  - Done when: A test fails when a Rust enum variant or field is added without the TS side.
  - Start after: CH11
- [ ] **CL30** · Claude Sonnet · next-tasks.md > Review plan > Architecture, docs and public release · CLAUDE.md "Architecture at a glance" and a legatum-engineering skill
  - What: The section says "planned" and omits the WASM bridge and LAN server. Update it, and add a short `legatum-engineering` skill (where behaviour belongs, deterministic core rules, adapter parity, verification paths) without repeating mechanics. Check the 150-line CLAUDE.md limit first.
  - Done when: CLAUDE.md under the size limit; skill exists.
- [ ] **CL32** · Claude Sonnet · next-tasks.md > Review plan > Architecture, docs and public release · Civ 6 departures: a reason for each absent system
  - What: Decision approved 2026-10-18. For every row of "Systems that do NOT exist" in the rules reference (trade routes, districts, religion, Great People, espionage, Builders, pillaging, naval, air movement, tech trading, moving cities, razing choice, promotion trees, tile assignment) record deliberate / planned / scope cut and a one-sentence reason tied to a pillar, from the design doc, learnings and skills. Where no reason is recorded, write "reason not recorded: ask the user" and list them for him.
  - Done when: Every row has a status and reason or an explicit question to the user; the Guide text matches.
  - Start after: CH4 has landed
- [ ] **CL33** · Claude Sonnet · next-tasks.md > Review plan > Architecture, docs and public release · Wire the finished art: portraits, celebration vignettes, then heraldry
  - What: After the user approves each image set (A2/A3 portraits, A4 vignettes, the heraldry concept sheet CQ15), wire them: portrait swaps, vignettes into `MomentLane` by moment kind (stage 1), crest/seal/glyph tiers for heraldry after a finalisation task. Take screenshots at both sizes, run the UI reviewer.
  - Done when: Each set is live, the UI reviewer reports no blockers.
  - Start after: user approval per set
- [ ] **CL34** · Claude Sonnet · next-tasks.md > Review plan > Architecture, docs and public release · Hand the new open-source facts to the readiness work
  - What: The OSS review found items missing from the checklist: 12 tracked `.reviews/` files and 8 tracked `batch.log` files (the audit counted 13), the tracked Civ 6 reference image `docs/design/terrain-art/reference/civ6-reference-user.webp` (exclude), the home LAN address in `docs/status/tracker.md` (~119), no `docs/README.md`, cloud infra excluded for the first release (decision D4). Pass them to the `claude/oss-readme` / oss-readiness owner and update the checklist.
  - Done when: the checklist reflects them; the export script has checks for these classes.
- [ ] **CL35** · Claude Sonnet · next-tasks.md > Review plan > Architecture, docs and public release · Tauri release build with the style CSP exception removed
  - What: `app/src-tauri/tauri.conf.json` ~26-35 disables Tauri's style CSP modification. Find why, build a release bundle (off-Mac) with it enabled and see what breaks; keep the exception only with a recorded reason. Needs a Mac Tauri build: ask first.
  - Done when: Reason recorded in `docs/learnings.md` or the exception removed with the bundle verified.
  - Start after: user available

### Rules simplicity (approved 2026-10-18: one-sentence test)

- [ ] **CL31** · Claude Opus · next-tasks.md > Review plan > Rules simplicity · Faction one-sentence pass: Sakoku, Sparta, Switzerland, Celts, then goals
  - What: Decision approved 2026-10-18: each faction ability and price must be one idea in about 30 words; bundles are split or cut. Japan (Tokyo-dense), Byzantium (Mercenary Companies), Mongols (Workshop siege unlock) and Germany (move rule) are already in flight; apply the test to them when they land. For Sakoku, Sparta (promotions, desertion, healing, slow rest), Switzerland (tolls+interest, upkeep+treasury loss), Celts/Celts (concealment vs ambush, territory damage) and the eight faction goals, write for each: current promise list, shortest honest sentence, what must be cut. One page per faction for the user; no mechanic changes land without his playtest sign-off. Copy edits after he decides are Luna tasks. The Keshig sheet line (charge or shoot, not both) can go with the Mongol pass.
  - Done when: Eight one-page proposals delivered and either decided or parked in the tracker; sheet text matches the rules.
  - Start after: Germany, Military Engineer landed (they edit factionSheets.ts)
- [ ] **CL36** · Claude Opus · next-tasks.md > Review plan > Rules simplicity · Add the 2026-10-18 decisions to the design doc decisions log
  - What: CLAUDE.md requires every design decision in the Claude Doc decisions log in the same turn. Add rows for D1-D13 in `docs/status/review-plan-2026-10-18.md` through the docs connector.
  - Done when: One row per decision exists in the design doc.

## 5. The mock-engine retirement as one sequence (decision D1)

Each stage is its own task; do not start a stage until the previous one is ticked.

1. **M1** = CQ5 (Codex Luna, queued): a missing WASM package is an error, the former client selector stays as an explicit dev fixture.
2. **M2a-M2d** (Claude Sonnet, next-tasks): Found City blocker and settlement scoring; attack forecasts and strength cues; building facts from `rules.generated.json`; inventory and replace every remaining production import of the former TypeScript engine.
3. **M3a** (Claude Sonnet): Rust-generated UI fixtures and their generator; **M3b** (Codex Luna, held, re-queued in batches of 25 test files).
4. **M4** (Codex Luna, held): delete the former TypeScript engine, remove the former client selector, refuse old mock-format saves with a plain message.

## 6. How accurate were the reviews?

Spot-checked against master `08d13cfe6` (73 commits after the reviews' base).

**Confirmed in code** (all of these are queued): the silent mock fallback (`client.ts`); the 46 missing evolution genes (checked with a script: 114 genes, 68 in the allowlist); `used_fighters` built before validation and the duplicate-rebase reservation (`air.rs`); no structural check in `parse_state` (but see below); Vitest 5.0.2 and Vite 8.3.1 locked while 5.0.5 and 8.3.4 exist on npm; the four stale `allow(dead_code)` markers (the functions are called); the 5 ms wall-clock assertion in `mockClient.test.ts`; the error toast with a click-only handler; the duplicate `view(0)` on load; Terraform README budget (stale figure) and cap (8 vs 24); the runner header; the "no Rust commands" claim in `app/README.md`; the verifier lock cleanup gap; fixed worker counts in `release/check.sh`; no behaviour tests for the Cities overview or Where to build; no root README, LICENSE or ARCHITECTURE.

**Stale or already handled:**
- The WASM cache key narrowing was finished on 2026-10-11 (`astra/wasm-cache-key-narrow`, one commit) and never landed. Queued as CL6, not as new work.
- Open-source findings repeat the 2026-10-11 checklist and the `claude/oss-readme` branch; only the new facts went to CL34. The review's sizes are higher than the audit's (it counted 8 tracked batch logs against the audit's 13 earlier).
- Bought Walls in the rules reference: `astra/walls-no-buy` is in flight; Catapult/Bombard vs Engineer: `claude/military-engineer` is in flight. Both doc fixes are held (CH4).
- "No validation of saved games": the LAN server has `check_state`; only the solo/WASM path lacks it, so CL1 lifts the existing check rather than inventing one.
- Gene count 113 in the AI review; master has 114 (scouting genes landed since).

**Wrong or overstated:**
- "An invalid unit order erases a standing order" is intentional: the code comment says every new order replaces the old intent even if it fails (D11). Not queued.
- The tooling review suggests freeing a pool worktree when "the owner PID is dead". That is exactly what wiped two agents' work on 2026-10-09 (the PID is a one-shot shell). CL5 uses an owner token instead. The same idea is correct for the verifier lock (CQ3), where the PID is the long-running verifier itself.
- The skills review calls the "No balance sims" workflow rule stale. It is the user's standing instruction (2026-10-04, kept in memory), so it was left alone.
- The art review says the Future-era painting clashes with "no sci-fi". The game has a Future era with drones and fusion power, so the painting matches the content; the user decided to keep the era and remove only the humanoid robots (D12).
- The dead-code review found nothing safe to delete; that is a static-scan result (no `cargo udeps`, no Knip), not proof. Not queued beyond the four markers.
- Performance numbers for Retina frame time and WASM load were never measured by the review; they are measurement tasks (CL17, CL20), not findings.

## 7. Not done on purpose

Splitting the oversized UI and Rust files all at once (held one at a time behind the UI gate), TLS for the LAN server (D10), converting old Quick saves (D8), deleting compatibility code for retired features (the reviews agree it is intentional), and any faction rule change without the user's playtest sign-off (D5).

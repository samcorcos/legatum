# Tools and scripts

Helper programs around the engine. Everything here is optional for building and playing. "Invoked by" lists what calls the directory from outside itself (grep across the repo, ignoring `docs/learnings.md`, on 2026-11-22). "Nothing" means no caller; run by hand or leave for history.

| Directory | What it does | Invoked by | Status |
| --- | --- | --- | --- |
| `tools/ai-plan/` | Builds the local AI-war plan dashboard HTML from `plans/ai-war-status.json`. | `plans/ai-war.md`, the legatum-workflow skill | active |
| `tools/ai-quality/` | Charts, climb targets and gate scripts for measuring AI quality. | `crates/sim/examples/milestone_suite.rs`, AI fixtures READMEs, the legatum-workflow skill | active |
| `tools/analysis/` | Balance probes and strategy-diversity analysis of sim reports. | `tools/cloud/merge.py`, docs (`strategy-diversity.md`, `ai-analysis.md`), `crates/sim` telemetry | active |
| `tools/games/` | Analyses uploaded web games on the developer's Mac: polls the games Worker, keeps a ledger, replays at the game's engine, cheap-model review. | `infra/games` read API, `crates/analysis`, `crates/sim` `game_dump`, launchd | active |
| `tools/art/` | Unit-art distinctness checks, art plans and derivative generation; `fetch-sources.py` fetches the art masters kept outside git (`sources-manifest.json`). | App art tests (`app/src/*Art*.test.ts`), art provenance docs | active |
| `tools/astra/` | Codex/Astra work queue runner, reviewers and AI analysis helpers. | `CLAUDE.md`, `.config/nextest.toml`, `tools/ops/`, `tools/sim/repin.sh`, `tools/release/check.sh` | active |
| `tools/audio/` | Builds the shipped audio from raw downloads: trim, normalise, encode, credits. | Sound docs, `app/src/assets/PROVENANCE.md` | active |
| `tools/ci/` | `local-ci.sh` (the CI steps run outside GitHub Actions) and test-group splitting. | `ci.yml`, `CLAUDE.md`, workflow skill, `tools/perf/lean_check.py` | active |
| `tools/cleanup/` | Disk cleanup, disk watch and stale lock removal for build machines. | Ops and workflow skills, ops agent | active |
| `tools/cloud/` | Remote build and batch-run boxes: sync, run sim batches, merge and verify results. | Sim examples, `tools/ops/`, `tools/astra/`, many docs | active |
| `tools/dashboard/` | Evolution and replay dashboard (HTML and JS). | `tools/film/`, `.claude/launch.json`, `docs/archive/ai-evolution-log.md` | historical (serves the retired evolution runs) |
| `tools/decision-audit/` | Designer-seat decision audits, tape building and diff reports. | `tools/sim/nightly.sh`, `crates/sim` examples and tests, playtest reviews | active |
| `tools/design/` | Brand tokens, app icon and mockups. | App styles and art README, `tools/gates/hygiene.py`, `tools/film/` | active |
| `tools/dev/` | One-off script that renamed legacy faction identifiers. | `tools/analysis/summarize.py` only | historical |
| `tools/evolve/` | CMA-ES evolution of the scripted AI's parameters. | `crates/ai/src/params.rs`, `.private.example/cloud.env`, archived evolution docs | historical |
| `tools/film/` | Renders storyboard films and reports from runs. | `tools/evolve/`, `tools/dashboard/` | historical |
| `tools/gates/` | The merge gates: static checks, art, engine invariants, hygiene, markers. | `ci.yml`, `CLAUDE.md`, `CONTRIBUTING.md`, `tools/ci/local-ci.sh`, scenario lint tests | active |
| `tools/hill-climb/` | Hard AI hill-climb measurement suite: two engine builds compared on recorded decision points. | Run by hand by the AI hill-climb work | active |
| `tools/league/` | Tuned-opening export (`export_tuned.py`) for Hard v2. | `crates/ai` (`hard_v2.rs`, `tuned_openings.rs`) | historical |
| `tools/ops/` | Agent worktrees, background jobs, landing, test timing and Playwright worker helpers. | `scripts/build-wasm.sh`, `.config/nextest.toml`, `tools/astra/`, `tools/cloud/`, `tools/release/check.sh` | active |
| `tools/perf/` | Performance budgets and profiles for AI turns and app load. | `app/e2e/perf-app.spec.ts`, `scripts/perf-wasm-*.mjs`, perf agent and workflow skill | active |
| `tools/release/` | Public export, release checks, link checker and license texts. | `ci.yml`, `release-mac.yml`, `THIRD_PARTY_NOTICES.md`, `app/` | active |
| `tools/scenarios/` | Extract, cut, compare and publish saved game states and scenarios. | App scenario tests and loaders, `app/e2e/scenarios.spec.ts` | active |
| `tools/sim/` | Simulation, combat and pacing runs, nightly runs, and `repin.sh` (regenerates pinned determinism tapes and hashes after an intended rule change). | `README.md`, `AGENTS.md`, `CONTRIBUTING.md`, `plans/`, `crates/ai` | active |
| `tools/training/` | Copies Training-mode answers from the desktop app into the repo. | The legatum-rules reference only | historical (desktop app is being removed) |
| `tools/ui/` | Playwright bridge helpers and WebKit tooling for UI checks. | `docs/pw-bridge.md`, `app/e2e/`, workflow skill | active |

## Main scripts

| Path | What it does |
| --- | --- |
| `scripts/build-wasm.sh` | Build the engine as WASM for the web UI. |
| `scripts/archive-stats` | Summarise or export the game-archive index. |
| `scripts/wasm-tape-check.mjs`, `scripts/wasm-search-check.mjs` | Check that WASM and native builds produce identical per-turn state hashes. |
| `scripts/perf-profile.sh` | Record a reproducible single-threaded performance profile. |
| `scripts/build-mac-app.sh`, `scripts/build-ios-sim.sh` | Desktop and iOS builds; being removed with the web-only change. |

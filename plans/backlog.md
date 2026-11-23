# Backlog

> 2026-11-22: the desktop (Tauri) app, the native `legatum-server` and LAN play were dropped; the game is web-only. Rows below that mention them are history. Online play: plans/online-play.md.

**Balance philosophy (user, 2026-09-26):** asymmetry over balance, TI4 not Eclipse. Keep factions bold and even lopsided; balance only prevents a faction that dominates (>~20% of its seats) or never matters (<~6%). Never homogenize factions to even out win rates.

Prioritized work toward the first playable, grouped by increment. The Ops supervisor keeps the Astra queue fed from items marked `ready`; the lead (Claude) owns this file, reviews all results and merges. Statuses: `ready` (can start now), `blocked (<why>)`, `doing (<who>)`, `done`.

Owner hints: **astra** = gpt-6-astra via `tools/astra/runner.sh` (write tasks get their own worktree), **claude** = a Claude subagent (UI/visual work that needs a browser), **lead** = me.

## v0.2 Scooter: LAN play, real war, forecasts

| ID | Item | Owner | Status |
| --- | --- | --- | --- |
| B1 | LAN game server `crates/server` per plans/lan-protocol.md | astra | done |
| B2 | LAN UI: host/join screens, RemoteClient, single-seat play | claude | done |
| B3 | Fix the 11 rule bugs from `.reviews/astra-review-rules.md` | astra | done |
| B18 | Integrate LAN server + UI and play-test two browsers | lead | done |

## v0.3 Bicycle: full rules, factions, save/load, desktop app

| ID | Item | Owner | Status |
| --- | --- | --- | --- |
| B22 | Standard game flow: main menu with Single Player and Multiplayer (Host / Join online), then a Civ-style lobby: seats (Human/AI/Closed, faction, team, AI difficulty), map size, seed, game length, generic presets. No bespoke 'partner' wording anywhere | claude | done (network staging room server side = B23) |
| B23 | LAN staging room on the server (host configures, players join seats, ready, start) per plans/lan-protocol.md; accept 1-8 humans | astra | implemented on astra/lan-staging-server; awaiting review (browser verification blocked by sandbox) |
| B4 | Tauri desktop app: `.app` bundle, app icon from the logo, embeds nothing yet (browser mode) | claude | done |
| B5 | Property/fuzz tests: random valid and invalid orders through `resolve_turn` never panic and keep invariants (≤1 military + ≤1 civilian per tile, no two teams on a tile, HP bounds, gold ≥ 0 after upkeep, determinism) | astra | doing (astra property-fuzz) |
| B6 | Implement unique units/buildings for Switzerland, Japan, Byzantium | astra | doing (astra factions-implement) |
| B7 | Implement the five new factions | astra | doing (astra factions-implement) |
| B10 | Sim report: battles, captures, eliminations, VP-source breakdown (titles vs milestones vs goals), turn-50 leader win rate | astra | ready |
| B17 | UI polish: unit move animations, battle forecast tooltip, turn report polish | claude | done (first pass) |
| B20 | Implement the chosen art direction across the whole UI (tokens + components + map rendering + title screen). Default: C "Heir's Court" chrome + B's map lettering until the user picks | claude | done (polish follow-ups: key-art title, loading, fog, Tauri perf = polish agent) |

## v0.4 Motorcycle: balance, AI, determinism

| ID | Item | Owner | Status |
| --- | --- | --- | --- |
| B8 | Balance tuning from `docs/analysis/balance-probe-1.md` (constants in data.rs, AI knobs), verified by 1,000-game runs. **Hard targets for game feel (from play-testing):** a typical new capital makes 3–4 Production on turn 1 (today ~1), so the first Soldier takes 10–14 turns and the first Settler 18–25; early turns must feel eventful like Civ 6. | astra | blocked (balance-probe) |
| B9 | AI v2: conquest pressure late game, better research/expansion choices, uses forecast_attack_mode; measured vs AI v1 in head-to-head sims | astra | queued (astra ai-v2) |
| B13 | Performance pass: profile resolve_turn, player_view, AI orders and WASM size; optimize hot paths without changing results (hashes must match). Note: fog memory (B3) made 200-game sims 7.4s → 10.6s; update_explored rebuilds every tile memory each movement round, a likely hot spot | astra | ready |
| S8 | Cross-target determinism check + committed replay fixtures | astra | done |

## After v1.0: next major features

| ID | Item | Owner | Status |
| --- | --- | --- | --- |
| B25 | Full tech tree per docs/design/tech-tree-design.md (skateboard first: readable tree + forecasts) | astra | done: all four eras, Great Works, tall milestone (2026-09-27) |
| B24 | Diplomacy (Stellaris-style): peace by default, treaties with minimum terms and public notice to exit, trust/reputation, war declarations that fit simultaneous turns, trade agreements with mutual economic benefit, AI diplomacy incl. coalitions vs the leader | astra | design queued (diplomacy-design); build after v1.0 unless the user pulls it into the Bicycle |

## v1.0 Car: the first playable

| ID | Item | Owner | Status |
| --- | --- | --- | --- |
| B11 | Browser end-to-end tests (Playwright) for new game, a few turns, save/load, LAN join | astra + claude | merged; spec drift fix in progress (claude) |
| B12 | In-game How to play (from the guide) and first-turn hints | claude | blocked (how-to-play guide) |
| B14 | Security review of online play (the relay and the in-browser host) | astra | queued |
| B16 | Colorblind-safe player colors and legibility review | astra | queued: a11y-atlas |
| B19 | Final review: Astra + Claude code reviews of everything, fix confirmed findings | astra + lead | blocked (all) |

## Mobile (plans/mobile.md)

| ID | Item | Owner | Status |
| --- | --- | --- | --- |
| M1 | Skateboard: touch-playable in iPhone Safari (touch map controls, tap breakdowns, on-screen turn controls) | astra + claude | in progress |
| M2 | Scooter: iPhone portrait layout, safe areas, autosave when hidden, motion and sound | astra | planned (next) |
| M3 | Bicycle: an online game between two phones, reconnect on return, home-screen icon and manifest | claude + astra | planned |

## Presentation and data (2026-09-27)

| ID | Item | Owner | Status |
| --- | --- | --- | --- |
| P1 | Game archive: every human and sim game stored as a compressed tape plus telemetry in ~/LegatumData/archive, outside the repo | astra | queued: game-archive |
| P2 | New icon family (favicon, Mac, iOS and dashboard) in the Atlas style | claude | in progress |
| P3 | Design consistency audit: recurring screenshot sweep of every screen, reviewed against the brand rubric | claude | after the QA harness lands |
| P4 | Sound: SFX mapped to feel.* events, ambient music, volume and silent-mode handling; technical audio QA automated, taste judged by the user through a sound board | claude + astra | planned; needs source/licensing decision |

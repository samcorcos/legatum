# Review 5: the app as played, and engineering health

Master `a9eb46864`, 2026-11-09. Read-only review: code, docs and his archived game tapes (`~/LegatumData/archive`); the app was not launched. Items marked (inferred) were not verified by running anything.

## Headline

The app is in better shape for a solo expert than the open problems list suggests, and worse for co-op than the feature list suggests.

- **Orders per turn are already low.** In his last 12 long games he issued a median of 2 to 5 orders per turn after turn 80, with 10 to 30 cities. The 10-04 "war micromanagement fatigue" problem (`legatum-design` skill, line 184) no longer shows in the tapes.
- **His games are short.** 116 to 181 turns, 22 to 62 minutes of active play, every one a VP win. The design target is 3 to 6 hours.
- **A co-op game cannot survive an install.** Saves are refused on any rules-version mismatch and the rules version changed 20 times in the last 9 days.
- **Hosting needs a terminal.** The desktop app does not embed the LAN server.
- **Opponents are mute.** The AI builds reasons, plans and campaigns internally; the player sees almost none of it.
- **The engine is fast; the UI is the latency.** Plain Hard resolves a whole late-game turn in about 31 ms; the click-to-paint End Turn is about 700 ms, nearly all UI work.

## 1. Player experience

### What the tapes say (measured)

Twelve most recent human games of 100+ turns, orders by seat 0 from turn 81 onward:

| Game | Turns | Active min | Late median orders/turn | p90 | Final cities |
| --- | --- | --- | --- | --- | --- |
| 10-27 | 125 | 24 | 3 | 5 | 20 |
| 10-29 | 180 | 28 | 3 | 5 | 15 |
| 11-01 | 181 | 62 | 5 | 10 | 20 |
| 11-01 | 153 | 35 | 3 | 5 | 14 |
| 11-03 | 160 | 40 | 4 | 7 | 22 |
| 11-04 | 159 | 29 | 2 | 4 | 10 |
| 11-06 | 177 | 48 | 4 | 7 | 30 |
| 11-07 | 144 | 22 | 2 | 5 | 12 |
| 11-07 | 122 | 35 | 5 | 10 | 21 |
| 11-07 | 135 | 25 | 3 | 6 | 16 |
| 11-07 | 153 | 30 | 2 | 6 | 13 |
| 11-09 | 116 | 29 | 5 | 8 | 10 |

Median seconds per turn rise from 3–7 s (T1–40) to 18–24 s (T121+). The slowest single turns are 30–95 s. `active_seconds` may exclude idle time; wall clock for the 11-07 Germany game was 66 minutes for 153 turns.

Late-game order mix (11-06 India game, T121+): MoveUnitPath 72, SetProductionQueue 48, AirStrike 36, SleepUnit 26, MoveUnit 21, RangedAttack 17. So the remaining manual work is, in order: moving units, refilling city queues, and air strikes.

Caveat: orders are not clicks. Each unit that needs orders costs one Return to reach it plus one click or key, and decisions that end in no order (opening a siege, reading an offer) are not on the tape. A fair estimate is 2 to 3 inputs per order (inferred), so 10 to 15 inputs on a typical late turn and about 30 on the worst.

### How the Return flow works

- `app/src/turnFlow.ts` builds one list of pending decisions in this order: diplomacy offers, calls to arms, cities under siege, goal actions, free techs, bankruptcy, cities with no automation choice, units needing orders (one entry per unit, sorted by unit id), aircraft, idle cities, empty research.
- `app/src/screens/gameKeyHandler.ts`: Enter jumps to the next pending decision, or readies the turn when none are left. Space skips, F fortifies, H heals, E auto-explores, I toggles city automation, U upgrades.
- A unit needs orders unless it has automation, a goto or a standing order (`app/src/orders.ts:170`).

### Delegation that exists

Auto-explore (the only `Automation` variant, `crates/core/src/types.rs:658`), city automation with four focuses, production and research queues, persistent goto, Fortify (wakes once per newly arrived enemy), Heal until full, garrison AutoFire, automatic tile improvements (no Builders).

### Delegation that does not exist

No army grouping or multi-select, no rally points, no queue templates, no "upgrade all", no standing air-strike target, no attack-move. Grep for `rally`, `multi-select` and `group` in `app/src` finds nothing; the `Order` enum in `crates/core/src/orders.rs` has no such variant.

### The five cheapest order cutters

Ranked by orders removed, using the late-game mix above. All are UI-side batching of existing orders except the last, so none needs a rules-version bump except where noted.

1. **Shift-click to add units to a selection, then one move target.** MoveUnit plus MoveUnitPath are 40–50% of late orders, and armies move together. The UI emits one existing `MoveUnit` per unit. One evening of agent work (about 3 hours, bottleneck: UI reviewer pass).
2. **Rally point per city.** New units get a goto to the rally tile on spawn. Removes the "walk the reinforcement to the front" order for every unit built. Needs one small field on the city and a rules bump. About 3 hours.
3. **Repeat air strike.** Aircraft keep striking the same target until it is dead or out of range. Air strikes were 36 orders in one late band. About 2 hours plus a rules bump.
4. **Order the Return queue by distance from the last unit handled, not by unit id** (`turnFlow.ts`). No orders saved, but it stops the camera jumping across the map between fronts, which is where the seconds go (inferred). About 1 hour.
5. **"Upgrade all" in the units list and a queue template ("copy this queue to all new cities").** UpgradeUnit and SetProductionQueue are the next largest groups. About 2 hours each, UI only.

Before building any of them, note the measured ceiling: they would take a 4-order turn to about 2. That matters far more for co-op with a new player than for him.

## 2. How the AI is presented

### What the player sees

- Race panel: VP and its sources, city count, era, power, lead percent, decadence (`PlayerPublic`, `crates/core/src/view.rs:736`).
- Diplomacy panel: relations, treaties, proposals, the coalition target with its members and a reason, betrayals, and "Recent diplomatic replies" with the AI's score reasons (`app/src/ui/DiplomacyPanel.tsx:652`).
- Turn report and chronicle: "X declared war on Y" with no reason (`app/src/format.ts:721`).
- Post-game report: history charts and the chronicle (`app/src/ui/PostGameReport.tsx`).

### What the engine has and the UI does not show

- **Reasons for war.** `Event::WarDeclared` carries no reason (`crates/core/src/events.rs:184`), but the AI already produces `DecisionExplanation{score, reasons}` strings such as "Shared enemy: +25" and "They broke an alliance with us" (`crates/ai/src/diplomacy.rs:1035-1122`). They are only attached to treaty replies.
- **Plans and campaigns.** Postures, campaign targets, `OperationLog` (`crates/ai/src/operation.rs:87`), `ProductionTrace` (`crates/ai/src/production.rs:17`). None reach the player, in play or after the game.
- **Observed threats.** `view.observed_threats` exists for the AI. His 11-07 loss "came from information he never saw" (human-reasoning notes, line 396): Spartan Hoplites near his border and an ally city at 0 HP.

### What would make opponents feel alive, cheapest first

1. **A reason on every war declaration and peace offer**, taken from the existing `reasons` list, in the turn report and the diplomacy row. Pillar 5 (no hidden math) already asks for this.
2. **One faction-voiced line per diplomatic event** (declaration, refusal, betrayal, coalition joining). A data table of about 10 lines per faction keyed by event and top reason; no engine change beyond item 1.
3. **Grievance list per opponent** in the diplomacy row: the dated events that moved their attitude. The relation scoring already has the inputs.
4. **Post-game "what they were thinking"**: replay each AI's posture and campaign target on the history timeline. This is fair after the game ends and gives him a direct read on AI quality, which he already studies through tapes.
5. **Threat alerts in the turn report** from `observed_threats`: "Sparta has 6 units within 3 tiles of Zurich", "Your ally's city Thebes is at 0 HP".

Not proposed: a visible coalition threshold meter. The 2026-11-08 decision keeps coalition wariness hidden.

## 3. Co-op readiness

### What works

- `crates/server` is a real LAN host: staging lobby, per-seat random tokens, join/take/release seats, orders and ready per turn, forecast and assault queries, autosave and resume, and a 2,505-line protocol test.
- `app/src/remote.ts` reconnects with backoff and pings. A disconnected seat does not hold up a ready teammate.
- Teammate units are labelled "Commanded by X".

### What is missing or fragile

- **Saves do not survive installs.** `RULES_VERSION = 25` (`crates/core/src/lib.rs:38`); `crates/wasm/src/json.rs:92` refuses any mismatch, by design (decision 2026-10-31, no migrations). Git log shows 20 bumps between 10-31 and 11-08 and 14 playtest releases in 10 days. His solo games fit in one sitting, so he has not felt this. A co-op game with a new player will take several evenings.
- **Hosting needs a terminal.** The Tauri app does not embed the server (`app/README.md:51`). He runs `legatum-server --static <dist> --port 7878` and both play in browsers. Stale server binaries have already broken tours (`legatum-workflow`, line 262).
- **No way to talk on the map.** No pings, no chat, no shared markers.
- **No gifting** of gold or units between teammates.
- **Path collisions.** The Guide tells players that same-class allied units entering one tile both stop, "so coordinate routes with your teammate" (`app/src/ui/GuideContent.tsx:246`).
- **Conflict rule is invisible.** When teammates give conflicting diplomatic answers the lower seat wins (`DiplomacyPanel.tsx:277`).
- **Pace mismatch (inferred).** He plays 10–20 s turns. Simultaneous turns help, but nothing shows the waiting player what the teammate is still doing.

### Shortest path to the two of them finishing a game

1. Pin a co-op build: install it side by side under its own name and do not replace it until the game ends. A release-script change, about 1 hour.
2. "Host game" in the desktop app that starts the bundled server as a sidecar and shows the join address. About 4 hours, bottleneck: one packaged-build test on both laptops.
3. Map ping (one key, visible to the team for the turn). About 2 hours.
4. Play. Record what the second player asks out loud; those questions are the UI backlog.

Steps 1 and 2 can land by 2026-11-10; a first full co-op game is then limited only by their evenings.

## 4. Performance

Source: `docs/sim-results/perf/turn-latency.md` and `app.md`.

| Measure | Value |
| --- | --- |
| Plain Hard, whole-lobby End Turn in WASM, on the laptop | 8/9 ms (T50), 19/24 ms (T150), 26/31 ms (T250), median/p95 |
| Share of the 1 s p95 budget | 3% |
| Hard+Search p95 | 2.0 s (T150), 4.9 s (T250); over budget on 24% and 52% of turns |
| In-app click-to-paint End Turn | 425 ms early, about 700 ms mean and 864 ms p95 late, about 620 ms of it main thread blocked |
| WASM bundle | 7.57 MB; main JS chunk 2.3 MB; `app/dist` 88 MB |
| Save size | 176 KB (T1), 1.4 MB (T201), 2.1 MB (T301) |
| Memory | WASM 40 MB at T301, JS heap about 18 MB, terrain cache about 220 MiB for two seats (CL21) |

Conclusions:

- **The live AI budget is not a constraint for plain Hard.** About 950 ms per turn is unused.
- **The budget binds only for search**, and only because the engine runs on the main thread at End Turn. His turns last 18–24 s late game. Moving the engine to a Worker and letting AI seats think during the human's turn gives search roughly 20 times the current budget with no added wait. `training_begin_json` already computes AI orders at turn start (inferred from `crates/wasm/src/lib.rs:559`), so the split exists in the bridge.
- **End Turn feels slow because of the UI**: map redraw, terrain rebuild (45–65 ms) and race-panel layout. Open items CL17 (the selection pulse redraws the whole map every 33 ms at DPR 2, `app/src/map/MapCanvas.tsx` near 699) and CL18 are the right fixes.
- Bundle and save sizes are fine for desktop.

## 5. Engineering health and risk

### Size

- App: about 52,000 lines of TypeScript outside tests. Largest: `map/terrain.ts` 1,738, `format.ts` 1,681, `map/render.ts` 1,509, `store.ts` 1,417, `map/MapCanvas.tsx` 1,412, `types.ts` 1,383.
- Rust: about 251,000 lines. Largest: `crates/ai/src/production.rs` 6,656, `plan.rs` 5,061, `military.rs` 4,302, `crates/core/src/view.rs` 3,622.

### Two sources of truth still in the app

- `app/src/types.ts` is a hand-maintained copy of the Rust wire types (CL27 open).
- `app/src/data.ts` (1,008 lines) hand-maintains building stats, and `app/src/combat.ts` mirrors Rust damage maths (M2a–M3a open). This breaks the one-engine rule and is the known source of forecast bugs.

### Tests

- 287 vitest files (about 3,100 tests), 111 Playwright specs, 133 skip/fixme/todo markers in app tests, 10 `#[ignore]` in crates.
- Flaky under load: a remote verify failed 2 vitest and 12 Playwright tests from timeouts on a saturated box; CL23–CL25 (tiered gate, worker cap, one known flake) are open. Playwright runs with one worker and a 45 s timeout. WebKit cannot run on this Mac.

### Saves

- The WASM load path checks the version only (`parse_state`); the server validates structure (`check_state`, `crates/server/src/host.rs` near 997). CL1 is open. A corrupt save can reach the engine in the desktop app.

### Build and release

- 14 releases in 10 days works, but the build label depends on tag order (learnings entry `300a11ca8`), and stale server binaries are a recurring trap.

### LAN server security

Sound for a home LAN: Host and Origin checks, host actions loopback only, per-seat tokens, 16 MB body and 64 KB WebSocket limits. It binds `0.0.0.0` over plain HTTP with no password. Acceptable at home; the README should say not to expose it to the internet.

### What would embarrass an open-source launch

- `app/src-tauri/tauri.conf.json`: `"shortDescription": "A sci-fi 4X on a hex map"`, empty copyright, version 0.1.0, and `minWidth 1100 / minHeight 700` while the review size is 1000x600.
- Sci-fi internal ids in live code and asset names: `BroodHall` (the Rice Storehouse), `RelayUplink`, `PlanetaryFoundry`, `WorldExchange` (`app/src/data.ts:50-70`, 151 Rust occurrences).
- Phone and iOS code in a desktop-only game: `app/src/mobile/`, `touch.css`, `PhoneSheet.tsx`, `tauri.ios.conf.json`, `playwright.phone.config.ts`, a haptics plugin used in 21 files.
- Japan and Sakoku remnants (`sakoku.ts`, `goal_sakoku.rs`) for a faction that is not playable.
- `dangerousDisableAssetCspModification` (CL35), the stale "Architecture (planned)" section in `CLAUDE.md` (CL30), and the items already listed in the oss-readiness doc (459 dangling doc links, 817 MB of sim results, personal references in skills).

## 6. Top 8 recommendations, by player benefit per cost

Estimates are agent wall-clock time; the bottleneck is named.

1. **Pin a co-op build so saves survive.** Evidence: `lib.rs:38`, `json.rs:92`, 20 bumps in 9 days. Change: versioned side-by-side install, never auto-replaced while an unfinished save exists on its rules version. About 1 hour; done 11-09.
2. **"Host game" button in the desktop app.** Evidence: `app/README.md:51`. Change: bundle `legatum-server` as a Tauri sidecar, show the join address. About 4 hours; bottleneck is one packaged test on two laptops; 11-10.
3. **Reasons and a voice for the AI.** Evidence: `events.rs:184`, `ai/src/diplomacy.rs:1035-1122`, `format.ts:721`. Change: carry the top reasons on war and peace events, show them in the turn report with one faction-voiced line. About 4 hours plus a rules bump and docs review; 11-10.
4. **Threat alerts in the turn report.** Evidence: human-reasoning line 396, `view.observed_threats`. Change: list enemy stacks near his and his ally's cities and ally cities in danger. About 3 hours; 11-10.
5. **Multi-select move, rally points, repeat air strike.** Evidence: tape order mix above; no such orders in `orders.rs`. About 8 hours total across three parallel agents; bottleneck is the UI reviewer; 11-11.
6. **Think during the human's turn.** Evidence: `turn-latency.md` (31 ms used of 1 s; search over budget on 52% of late turns). Change: engine in a Worker, AI orders computed while he plays. About 1 day including determinism checks; 11-12. This is the enabler for a stronger Hard, which the 22-to-62-minute wins say is the real gap.
7. **Fix End Turn paint cost.** Evidence: `app.md` (620 ms blocked), CL17, CL18. Change: stop full-map redraw for the selection pulse, defer race-panel layout. About 4 hours; bottleneck is re-running the perf capture; 11-11.
8. **Open-source hygiene pass.** Evidence: `tauri.conf.json`, `data.ts:50-70`, `app/src/mobile/`. Change: fix the description and window minimum, rename sci-fi ids (rules bump), delete phone and Japan code, validate saves on load (CL1), generate `types.ts` from Rust (CL27). About 1.5 days across parallel agents; bottleneck is the full test gate on a busy box; 11-13. Schedule the id rename after the co-op game starts on its pinned build.

## Open question for him

Is war micromanagement still tedious? The tapes say 2 to 5 orders a turn. If it is, the cost is in something the tape does not record (camera travel, re-woken units, reading the map), and recommendation 5 should start with the Return-queue ordering instead.

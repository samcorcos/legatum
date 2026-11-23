# Game setup and start

Part of the Legatum rules reference (index: `index.md`). Answers: How are factions, AI levels, seats and the calendar set up, and what does an empire start with on turn 1?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

## New-game faction setup

In the app lobby, explicit faction choices are kept and every Random seat is resolved in player
seat order from factions no seat has chosen or already received. The seeded draw is deterministic;
once all factions are in use, a repeated draw is limited to the least-used factions
(`app/src/lobby.ts::resolveFactions`, `app/src/gameSetup.ts::factionChoiceIndex`). Core
`GameSetup::all_ai` seats eight distinct factions from the original nine-faction pool, rotating the omitted faction with the seed; Egypt is selected explicitly in the app or test setups so established seeded tapes do not change
(`crates/core/src/state.rs::GameSetup::all_ai`).

New app lobbies default every AI seat to **Hard**. The single game-wide AI level is stored under the release-independent `legatum.lobby.difficulty` key; Normal remains an explicit choice. The game view
exposes each seat's recorded difficulty and Search (Training) flag, shown in Diplomacy and the victory race
and score ledger (`app/src/lobby.ts::newLobby,rememberDifficulty,buildSetup`,
`crates/core/src/view.rs::public_view`, `app/src/ui/DiplomacyPanel.tsx`, `app/src/ui/VpTrack.tsx`,
`app/src/ui/ScoreLedger.tsx`).

**AI levels (2026-11-21):** players are offered **Easy, Normal and Hard**, and all three plan only
from the seat's own `PlayerView`. **Training is a research tool, not a level a player can choose:**
its battle study reads the true state, including units the seat cannot see, so no production build
lists it in any picker (solo lobby and online staging). A development build (`npm run dev`)
offers it with `?training=1` in the URL (remembered under `localStorage['legatum.dev.training']`;
`?training=0` forgets it; `app/src/lobby.ts::trainingSwitch,offeredLevels`), and there its
description says it uses the true positions of nearby units. With the switch off, a stored lobby
preference of `Training` or the older `HardSearch`, and a lobby or online room whose seats carry the
`search` flag, read as Hard and start a game without the flag (`levelOf`, `levelFields`,
`readDifficultyPreference`, `app/src/staging.ts`). Nothing changed in the core, the save shape or
the hashes: a save or tape recorded with Training still loads, is labelled Training in Diplomacy
and the race, and keeps running its battle study. The online host relays the `search` flag unchanged
(`crates/wasm/src/host/staging.rs`); it offers no picker of its own.

**Training (2026-11-07):** replaced "Hard + Search" and is saved as the same `PlayerSetup::search`
flag on a Hard AI seat (no rules change, no save-shape change). A Training seat
plays plain Hard (`crates/wasm/src/session.rs::make_brain`), except in its battles: each turn its
orders are computed at the start of the turn (`Session::pending_ai_orders`), and every battle in
scope (an enemy city with its army within 2, its own city with an enemy army within 2, or armies
within 2 of each other; at most 3 per seat, every own unit within 3 of the centre searched) is cut
into an arena from the **true state** (the teacher may see everything) and searched by coordinate
descent over every legal per-unit order (hold, rest or heal, every move tile in reach, every
ranged and melee target; melee under 10% to win and moves away from the fight other than one
retreat tile are pruned, except that a melee charge on an enemy city is never pruned: the shots may
zero the city first, then the charge walks in, `training.rs` `SUICIDE` and `city_at`, 2026-11-08), restarted from Hard's orders and the lab's storm, focus and wait,
scored by rollouts of up to 20 turns (Hard continues for the seat; the enemy alternates Hard and a
sortie policy; common dice for every candidate), `crates/ai/src/training.rs`. The searched orders
replace Hard's only for the battle's units; nothing else changes, and the human seat is never
searched live (his battle positions are logged with his orders instead). The search runs while
the human plans (in WASM slices in the browser) and is capped at 15 s a turn
(`localStorage['legatum.training.capMs']`); at the cap the best orders found so far are played.
The UI: a top-bar chip while it runs, a progress card if End Turn has to wait, and a turn-report
line on what changed (`app/src/training.ts`, `app/src/ui/Training.tsx`). Every searched battle,
every human battle position and each one's realised outcome become JSON log lines (schema 1, with
the arena embedded so any line can be searched again: `legatum-sim --retrain LOG`), kept in memory
for the session as `globalThis.legatumTrainingLog` (the latest 5,000, `app/src/training.ts`); a Training game's archive is marked `mode: training`
(`crates/wasm/src/history.rs::Metadata`). Headless: `legatum-sim --training DIR` (every Hard seat),
`combat_lab --training-compare OUT`, `war_suite --brain training`; summarise with
`tools/sim/training-report.py`.

In a new solo free-for-all lobby, Standard starts six empires (you and five AI seats); Small starts four and Large eight (`app/src/lobby.ts::MAP_SIZES,newLobby`). Changing map size resets the count to that size's default while every AI seat still has a default name and Random faction. Once any AI seat has a custom name or chosen faction, an in-range count stays as chosen; an out-of-range count is adjusted without removing that customised seat (`app/src/lobby.ts::setMapSize`).

**Co-op AI alliances (2026-11-22, review finding; no rules bump).** `GameSetup::ai_alliances` (default off, omitted from setups, saves, tapes and hashes when off) starts the teams with no human seat in Alliances of two: `GameSetup::ai_alliance_pairs` puts those teams in a seeded order (`rng::seeded_order`, purpose `AI_ALLIANCES`; the same seed gives the same pairs whatever the seat order) and takes them two at a time, an odd team out staying alone, never the human team; `blank_game` signs each pair with `DiplomacyState::start_alliance` (met, a normal Alliance dated turn 1, the usual 10-turn protected window, no coalition target). From then on they are ordinary Alliances (upkeep, calls to arms, leaving or breaking them, no shared VP). `GameSetup::standard` and `all_ai` leave it off, so every pin and tape is unchanged. In the app the **Co-op preset** (hosted rooms start in it) turns it on (`lobby.ts::applyPreset`, `Lobby.aiAlliances`); the lobby's Game settings show **AI alliances On/Off** whenever two or more AI-only teams exist, and the hosted room carries it as `ai_alliances` (`staging.ts`, `crates/wasm/src/host/staging.rs::StagingConfig`). Solo, Teams and Free-for-all start with it off. The view carries a public setup fact, `PlayerView.ai_alliances` (true when `ai_alliance_pairs()` is non-empty, omitted when false; `crates/core/src/view.rs`), so a loaded game can say it too: the Diplomacy Relations note reads it, and the Start button's sub-line uses `lobby.ts::aiAlliancesOn`. Tests: `crates/core/tests/ai_alliances.rs`, `app/src/screens/lobby/AiAlliances.test.tsx`.

## Calendar (display only, designer 2026-11-14)

The UI shows a calendar year beside the turn number ("Turn 81 · 1130 AD" in the top bar, the turn report header and the Resume and Continue buttons), never in place of it. The year is a pure function of the turn number: no game state, no save shape, no RULES_VERSION bump, and no rule reads it. The anchors (turn to year, BC negative, no year 0, linear interpolation in integer maths, one year per turn after turn 300) are in `crates/core/src/calendar.json`; `crates/core/src/calendar.rs` interpolates (`calendar_year`, `calendar_label`) and the app reads the labels exported to `app/src/rules.generated.json` (`calendarLabels`, refreshed with `UPDATE_UI_RULES=1 cargo test -p legatum-ai --test ui_rules`), so there is one implementation. The target is a game ending between turn 250 and 300 (the turn limit is 300). The sim report's "Tech pace against the calendar" table sets the median year each tech is learned beside `calendar::tech_real_year` (Flight 1903, Armoured Warfare 1916) as a pacing gauge for the designers.

## 2. Game start

Every empire starts with a population-1 capital founded on its balanced start hex, its centre and first ring claimed, and no production selected. A Scout and Spearman stand on the first two passable neighbors in hex order; no faction, including Rome, starts with a Settler (`crates/core/src/game.rs::try_new_game`, `rules/cities.rs::found_city_at`). The capital gets the ordinary founding name, capital and original-capital flags, Palace yield, and a turn-1 `Chronicle::Founded` entry. It does not train a Settler or raise its price. Extra `rule_experiments.start_settlers` Settlers are placed around the capital. `blank_game` remains empty for tests. Mapgen balances the sites with `balance_founding_sites`. **Every empire starts with a treasury of `data::START_GOLD` = 60 Gold** (`game.rs::try_new_game`; `blank_game`, used by tests, still starts at 0; `rules.generated.json` `startGold`). It is below Swiss interest's first step (100 Gold) and far under the 5,000-Gold hoard goal.

**Turn-1 legality**: an item needs its tech prereq met (`economy.rs::production_blocker`). At turn 1 no techs are
known, so the already-founded capital can choose Scout, Spearman, Archer, Horseman or the Granary (+2 Food, 50 Production); a
**Settler needs population 2** (below). A "Workshop turn 1" opening is impossible — Workshop
needs Bronze Working.

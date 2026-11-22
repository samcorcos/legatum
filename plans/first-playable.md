# First playable: engineering spec

The contract every piece of the first playable is built against. Design intent lives in the design doc (see CLAUDE.md); this file holds the concrete rules, numbers and interfaces for this milestone. Numbers are starting values for simulation tuning, not final balance. Revised after an independent review by gpt-6-astra (`.reviews/astra-spec-review.md`, 2026-09-26).

**Done means:** two humans on one computer play a full game in the browser against 6 scripted AIs: explore, found cities, grow, research, fight, and race on a live victory-point track, with simultaneous turns, save/resume and autosave.

**Out of scope for this milestone:** LAN, delegation/governors, fronts and army templates, diplomacy, internal pressure, coalitions, navies, rivers, map wrapping.

## Architecture

```
crates/core   legatum-core   rules, map generation, views, forecasts. Pure + deterministic. No floats, no I/O.
crates/ai     legatum-ai     AI brains. Reads only PlayerView (fog applied). Never sees full state.
crates/sim    legatum-sim    headless runner CLI: parallel games (rayon), determinism check, writes docs/sim-results/
crates/wasm   legatum-wasm   wasm-bindgen API over core + ai for the UI
app/                         Vite + React + TypeScript UI, canvas hex renderer; served as a static web app (the Mac/Tauri app was dropped 2026-11-22)
```

The UI runs the whole game locally through WASM, so the whole game runs in the browser.

### Determinism rules (core)

- `#![deny(clippy::float_arithmetic)]` in core. All math is fixed-width integer (`i32` game values, `u64` seeds/hashes). Division truncates toward zero. Percentages are integers out of 100; multiply before dividing. Overflow is a bug (debug builds panic); hashes and RNG use explicit `wrapping_*`.
- One RNG (`rng::Rng`, SplitMix64). Every draw derives from `(game_seed, turn, purpose, round, entity ids…, draw index)`, never from call order across systems. `range(lo, hi)` = `lo + next_u64() % (hi - lo + 1)`.
- Iteration is always in a defined order (`BTreeMap`/`BTreeSet`/sorted `Vec`, never `HashMap` iteration). Every tie-break is explicit, ending in the lowest id / hex order.
- `GameState::hash()` = FNV-1a 64 over the serde_json serialization (fixed field order, integers only). Same seed + same orders ⇒ same hash every turn, in native debug, native release and WASM, including after save/load. 64-bit seeds and hashes cross into JavaScript as decimal strings, never JS numbers.
- Cross-target check: replay fixed order tapes natively and in WASM (Node) and compare per-turn hashes (story S8).

### Core API (stable contract)

```rust
pub fn new_game(setup: &GameSetup) -> GameState;
pub fn player_view(state: &GameState, player: PlayerId) -> PlayerView;   // fog applied, team shares vision
pub fn validate_orders(state: &GameState, player: PlayerId, orders: &[Order]) -> Vec<OrderError>;
pub fn resolve_turn(state: &GameState, orders: &BTreeMap<PlayerId, Vec<Order>>) -> (GameState, Vec<Event>);
pub fn forecast_attack(view: &PlayerView, attacker: UnitId, target: HexCoord) -> Option<BattleForecast>;
```

Orders: `MoveUnit { unit, to }` (persistent goto; core pathfinds), `MoveUnitPath { unit, to, path }` (confirmed client route, excluding the starting hex; persists across turns), `HoldUnit`, `FoundCity`, `DisbandUnit`, `RangedAttack { unit, target }`, `UpgradeUnit`, `SetProduction { city, item }`, `BuyItem { city }`, `SetFocus { city, focus }`, `SetResearch { tech }`. At most one unit-action order per unit per turn (a second one for the same unit is rejected by validation and ignored at resolution). At most one `BuyItem` per city per turn.

`Event`s are the turn report: every movement result, battle (with the numbers that decided it), city founded/grown/captured, tech done, title change, VP scored, failed order. The UI shows them as the resolution log.

## Map

- Pointy-top hexes, axial `(q, r)`, rectangular, no wrap. **48 × 32** (1,536 tiles) for 8 empires.
- Terrain (food/production/gold): Grassland 2/0/0 · Plains 1/1/0 · Forest 1/2/0 (move 2, +3 defense) · Hills 0/2/0 (move 2, +5 defense) · Desert 0/0/1 · Tundra 1/0/0 · Coast 1/0/1 · Ocean 1/0/0 · Mountains impassable. Coast and ocean are impassable to land units (no navy yet).
- **One connected landmass** (≈40–50% land) holding all 8 starts in the same passable component; small separate islands allowed only if they hold no start. Latitude bands give tundra near the top/bottom and deserts in dry mid-latitudes; forests and hills by noise; mountain ridges that never cut the landmass apart.
- About 1 Astrarchy ruin per 60 land tiles (≈10–12 ruins).
- 8 starts on grassland/plains, pairwise distance ≥ 7, each with **≥ 12 workable land tiles (non-mountain land) within radius 2**. Retry with a derived seed until valid.

## Empires and factions

- 8 empires. Seats 0 and 1 are humans on a permanent team (shared vision, cannot attack each other, shared victory). Seats 2–7 are AIs, each its own team. Sims may use any team layout.
- All empires are at war with every non-teammate (no diplomacy yet).
- Each empire starts with 1 Settler, 1 Scout, 1 Soldier.

| Faction | Rule it bends (simplified for this milestone) | Faction goal (+2 VP, permanent, once) |
| --- | --- | --- |
| Switzerland | +1 Gold per city per 2 population; buy items at 1.5× instead of 2× Gold per Production | Hold 400 Gold at once |
| Japan | +2 Influence per city (borders grow fast); cities more than 8 hexes from the capital yield −25% | Own 6 cities |
| Byzantium | Ruins give double rewards | Know Deep Survey and have explored 100 land tiles |

## Units

One military and one civilian unit per tile.

| Unit | Cost | Moves | Melee | Ranged (range) | Needs | Upgrades to |
| --- | --- | --- | --- | --- | --- | --- |
| Scout | 30 | 3 | 5 | — | — | — |
| Settler (civilian) | 80 + 20 per settler already trained; city pop ≥ 2, −1 pop | 2 | — | — | — | — |
| Soldier | 40 | 2 | 20 | — | — | Trooper |
| Ranger | 50 | 2 | 8 | 15 (2) | — | — |
| Trooper | 70 | 2 | 32 | — | Ballistics | — |

- Units produced, bought, upgraded or captured this turn act **next turn** (spawn with 0 moves).
- **Healing:** a unit that neither moved nor fought this turn heals 10 HP (20 in own territory).
- **Fortify:** each full turn a military unit neither moves nor fights, its fortify bonus rises by 3, max 6. Moving or attacking clears it.
- **Upgrade:** in own territory, with the tech, pay 2 × (cost difference) Gold. Keeps HP; acts next turn.
- **Upkeep:** 1 Gold per military unit beyond **2 free per empire**. A negative treasury disbands the costliest military unit (one per turn) and resets gold to 0.
- Settlers are captured (become the attacker's, 0 moves) when an enemy military unit enters their tile.

## Simultaneous turn resolution

Phases, in order:

1. **Orders:** apply Hold/Disband/Upgrade/Focus/Production/Research/Buy orders; set gotos from MoveUnit. Reset moves at the start of resolution.
2. **Ranged attacks:** all resolved from the same snapshot, damage applied together. No retaliation. Only units that haven't moved this turn; the attacking unit then cannot move.
3. **Movement in rounds** (simultaneous intent resolution). Each round:
   - Snapshot. Every unit with a goto and enough moves declares one step (its next path step, computed with the land pathfinder). A unit may always take one step at full moves; otherwise it needs moves ≥ the step cost. A unit that can't step stops for the turn.
   - **Friendly conflicts:** same-owner units of the same class route around reserved intermediate steps where possible, otherwise take turns through the contested hex by lowest unit id. Waiting units retry within the turn only while another move succeeds; every progressing round spends movement, so this terminates. Different owners on the same team still bounce when contesting the same tile (stay, goto kept). Moving into a tile a friendly unit is leaving succeeds only if that unit's move succeeds (chains and cycles resolve together; a blocked link blocks everything depending on it).
   - **Hostile contact:** a step into a tile held by an enemy unit or city is an attack on it. Two hostile units stepping into the same empty tile, or swapping tiles, fight a **meeting engagement**: both use `combat::unit_meeting_strength` (no terrain/fortify bonuses). Contact pins: an attacked unit that was trying to leave stays.
   - All engagements in a round are computed from the snapshot and applied together (mutual kills possible). For three-way contests, every hostile pair fights once.
   - After damage: a unique surviving claimant of an emptied or contested tile advances into it; otherwise everyone stays. Attacking ends the attacker's movement for the turn.
   - Newly seen tiles are revealed after each round.
4. **Combat math** (Civ 6-style): HP 100. Wounds weaken: units lose 1 strength per 10 HP missing, cities 1 per 20 HP missing. Damage to the other side = `combat::damage(own, other, roll)` with a seeded roll in 80–120% per side. Defender uses terrain and fortify bonuses (except meeting engagements). **A garrison absorbs attacks on its city**; the city takes damage only when it has no military garrison. Cities: strength 10 + 2×pop (+10 walls), 200 HP. A surviving melee attacker can enter and capture a 0-HP city once its hostile military defenders are cleared. Killing a garrison on an already 0-HP city permits capture immediately. Hostile units simultaneously entering an ungarrisoned 0-HP city fight meeting engagements against each other, without additional attacks on the ruined city. Survivors on the same team (the same owner or teammates) capture for their team: the entrant with the most HP after resolution takes the city, and a tie goes to a seeded coin keyed on (game seed, turn, tile), never unit id or seat (decisions log, 2026-09-27). Ranged units (ranged strength > 0) never capture a city, by moving in, winning a meeting engagement or killing a garrison; they are blocked with a stated reason. Forecasts use visible current defenders; they cannot predict secret incoming orders. Captured city: −1 pop (min 1), production and food stored reset, loses capital status, **3 turns of resistance (no output)**, its territory changes owner. An empire with no cities and no settlers is eliminated (units removed).
5. **Found cities:** land tile, not owned by another empire, distance ≥ 4 from every city. **Conflicting found-city orders within 3 hexes of each other all fail without consuming the settlers.** Radius-1 tiles not owned by others become the city's territory. The first city is the capital.
6. **Cities:** citizens work the best territory tiles within radius 2 by the city's focus (Balanced / Growth / Production), ties by hex order. The city center is always worked for free and yields its terrain **plus a flat 2 Food, 1 Production, 1 Gold** (like Civ 6's city-center minimum). Science = pop/2 + buildings. Influence = 1 per city + buildings. Growth threshold = 15 + 8 × (pop − 1); surplus = food − 2 × pop; excess carries over; starvation shrinks the city (min 1). Completed units spawn on the city tile (or the first free adjacent passable tile in hex order); if none is free, production waits. Settler completion waits until pop ≥ 2. Idle production (nothing selected) accumulates up to 30.
7. **Gold and upkeep.** Buying: any item except the megaproject, for 2× its remaining Production in Gold (Switzerland 1.5×); it completes this turn.
8. **Research:** science accumulates; stored science carries over when switching. Tech diffusion: −10% cost per other empire that knows the tech, up to −30%.
9. **Borders:** each city's influence accumulates; claiming the next tile costs 10 + 5 × (tiles already bought by that city), deducted on claim. The city claims the best unowned tile adjacent to its territory within radius 3.
10. **City healing:** +10 HP only if no enemy military unit is adjacent.
11. **Scoring**, then turn + 1.

### Buildings and techs

| Building | Cost | Effect | Needs |
| --- | --- | --- | --- |
| Monument | 40 | +2 Influence | — |
| Granary | 50 | +2 Food | Soil Reclamation |
| Workshop | 60 | +2 Production | Fabrication |
| Market | 60 | +3 Gold | Exchange |
| Laboratory | 80 | +3 Science | Signal Theory |
| Walls | 60 | +10 city strength | Fortification |
| Relay Uplink (megaproject) | 400, cannot be bought | milestone (+2 VP); announced to all when started | Relay Restoration |

Tech tree (cost in Science): Soil Reclamation 20 · Fabrication 25 · Fortification 30 · Exchange 35 (needs Soil Reclamation) · Signal Theory 40 (needs Fabrication) · Ballistics 60 (needs Fortification + Fabrication) · Deep Survey 70 (needs Signal Theory; reveals ruins on the map) · Civic Networks 80 (needs Exchange; +1 Influence per city) · Relay Restoration 150 (needs Signal Theory + Civic Networks).

## Ruins

The first unit to enter a ruin explores it (clears it). Reward from the seeded RNG, one of: +40 Gold, +25 stored Science, +1 population in the nearest own city (Gold if none). Byzantium gets double.

## Victory (live, public)

- **Titles** (1 VP each, held live): Largest Economy (Gold + Production per turn), Most Territory (owned tiles), Strongest Army (Σ strength × HP/100), Most Advanced (science generated over the last 5 turns), Most Populous (total population). A title is held by the empire with the strictly highest value > 0; the incumbent keeps it on a tie; a tie among non-incumbents leaves it unclaimed.
- **Milestones** (permanent, **each empire can reach each once**): 4 cities (1), 6 techs (1), 20 total population (1), complete the Relay Uplink (2).
- **Faction goal:** +2 VP, permanent.
- **Win:** a team whose total VP reaches its target at the end of a turn wins: **10 VP for a one-empire team, 17 for a two-empire team** (+5 per extra member). Several teams reaching it on the same turn: highest VP wins, then most population; an exact tie is a draw. Last team standing wins. **Turn limit 150:** highest VP wins; an exact tie is a draw.
- The VP track and every title's current values are public in every PlayerView.

## Legibility requirements

- Every city yield in a PlayerView carries a breakdown (`economy::CityYields`).
- Production, growth and research show turns remaining.
- `forecast_attack` returns win chance, loss chance and expected damage for both sides (exact, over all roll pairs).
- Events explain battles with the numbers used.

## AI v1 (scripted, knobs)

Knobs: expansion, military, science, economy, aggression (0–100). Scouts explore toward the nearest unexplored or ruin tiles. Settlers go to the best-scoring site (food + production within radius 2, ≥ 4 from known cities, not foreign territory). Production and research pick by knob-weighted scores. Military defends threatened cities and attacks when its local strength ratio (accounting for HP, terrain and walls) exceeds a threshold between 90% (aggression 100) and 130% (aggression 0). Knobs per faction; deterministic given the view and seed. Also a `RandomBrain` baseline.

## Simulation runner

`legatum-sim --games N --seed S --threads T --label L` runs AI-only games in parallel and writes `docs/sim-results/<date>-<label>.md` (+ `.json`): win rate per faction, VP distribution, game length, share decided by VP vs turn limit vs conquest vs draw, average cities and techs, panics. `--determinism` replays each game twice and compares per-turn hashes; `--tape` writes/reads order tapes for the native-vs-WASM comparison.

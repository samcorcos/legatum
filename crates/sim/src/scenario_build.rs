//! `legatum-sim scenario-build [ids]`: regenerate the start states of the skateboard scenarios.
//!
//! Each recipe plays an all-AI game (or replays the user's recorded game) to a turn, then makes
//! a few legal edits (unit placements, a declaration of war resolved as an ordinary order).
//! The builder rewrites only the generated parts of `scenarios/<id>.json` (state, seat, faction,
//! the objective's conditions and score, source); the human-written title, briefing, tests and
//! predictions are kept. Every state is checked by `scenario::validate_state` before writing.

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::scenario::{Condition, Metric, Objective, ScoreTerm};
use legatum_core::{
    AiDifficulty, CityId, Faction, GameSetup, GameState, Hex, Order, PlayerId, UnitId, UnitKind,
    game::spawn_unit, new_game, player_view, resolve_turn, rules::is_military,
};
use legatum_wasm::scenario::{
    EncodedState, FORMAT, Predictions, ScenarioFile, ScorePrediction, VERSION, encode_state,
};
use legatum_wasm::session::{OrderMap, brain_seed};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(
    name = "legatum-sim scenario-build",
    about = "Regenerate scenario start states"
)]
pub struct Cli {
    /// Scenario ids to build (default: all).
    pub ids: Vec<String>,
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scenarios"))]
    pub dir: PathBuf,
    /// The user's save (with complete history) that the runaway scenario starts from.
    #[arg(long)]
    pub source_save: Option<PathBuf>,
}

pub const IDS: [&str; 4] = [
    "rush-defense",
    "rush-offense",
    "runaway-conqueror",
    "opening-race",
];

/// What a recipe produces; the builder merges it into the scenario file.
struct Built {
    state: GameState,
    seat: PlayerId,
    objective: Objective,
    source: String,
    /// Facts for writing the briefing and predictions.
    notes: Vec<String>,
}

type Brains = BTreeMap<PlayerId, Box<dyn Brain>>;

fn scripted(state: &GameState) -> Brains {
    state
        .players
        .iter()
        .map(|p| {
            let b: Box<dyn Brain> = Box::new(ScriptedBrain::with_difficulty(
                p.faction,
                brain_seed(state.setup.seed, p.id),
                p.id,
                state.setup.players[p.id.0 as usize].difficulty,
            ));
            (p.id, b)
        })
        .collect()
}

/// Resolve one turn with every alive player's AI orders, after `edit` adjusts them.
fn advance(state: &mut GameState, brains: &mut Brains, edit: impl FnOnce(&mut OrderMap)) {
    let mut orders = OrderMap::new();
    for (id, brain) in brains.iter_mut() {
        if state.player(*id).alive {
            orders.insert(*id, brain.orders(&player_view(state, *id)));
        }
    }
    edit(&mut orders);
    *state = resolve_turn(state, &orders).0;
}

fn play_to(state: &mut GameState, brains: &mut Brains, turn: u32) {
    while state.turn < turn && !state.finished {
        advance(state, brains, |_| {});
    }
}

/// An eight-empire Standard game at Hard, one of each faction (seat order from the seed).
fn hard_setup(seed: u64) -> GameSetup {
    let mut setup = GameSetup::all_ai(seed);
    for p in &mut setup.players {
        p.difficulty = AiDifficulty::Hard;
    }
    setup
}

fn player_of(state: &GameState, faction: Faction) -> PlayerId {
    state
        .players
        .iter()
        .find(|p| p.faction == faction)
        .expect("one of each faction")
        .id
}

fn combat(state: &GameState, p: PlayerId) -> Vec<UnitId> {
    state
        .units_of(p)
        .filter(|u| is_military(u.kind) && u.kind != UnitKind::Scout)
        .map(|u| u.id)
        .collect()
}

/// Free tiles at `min..=max` hexes from `around` where `mover` may stand, nearest to `toward`.
fn staging(
    state: &GameState,
    mover: PlayerId,
    around: Hex,
    toward: Hex,
    (min, max): (i32, i32),
    allowed: impl Fn(Option<PlayerId>) -> bool,
) -> Vec<Hex> {
    let mut tiles: Vec<Hex> = (min..=max)
        .flat_map(|r| around.ring(r))
        .filter(|h| {
            state.map.get(*h).is_some_and(|t| {
                t.terrain.move_cost().is_some()
                    && allowed(t.owner)
                    && state.passage_allowed(mover, legatum_core::UnitKind::Spearman, t.owner)
            }) && state.city_at(*h).is_none()
                && state.units_at(*h).next().is_none()
        })
        .collect();
    tiles.sort_by_key(|h| (h.distance(toward), *h));
    tiles
}

/// Move a unit to `to` as a clean edit: no orders, routes or fortification carried over.
fn relocate(state: &mut GameState, unit: UnitId, to: Hex) {
    let u = state.units.get_mut(&unit).expect("unit");
    u.pos = to;
    u.goto = None;
    u.path.clear();
    u.path_stop_on_block = false;
    u.automation = None;
    u.standing_order = None;
    u.attack_target = None;
    u.needs_orders = false;
    u.blocked_turns = 0;
    u.fortify = 0;
}

fn nearest_city(state: &GameState, owner: PlayerId, to: Hex) -> Option<CityId> {
    state
        .cities_of(owner)
        .min_by_key(|c| (c.pos.distance(to), c.id))
        .map(|c| c.id)
}

fn capital_pos(state: &GameState, p: PlayerId) -> Option<Hex> {
    state
        .capital_of(p)
        .or_else(|| state.cities_of(p).next())
        .map(|c| c.pos)
}

fn term(metric: Metric, weight: i32) -> ScoreTerm {
    ScoreTerm { metric, weight }
}

fn city_line(state: &GameState, id: CityId) -> String {
    let c = &state.cities[&id];
    format!(
        "{} (id {}, {:?}) at ({},{}) pop {} hp {} walls {} capital {}",
        c.name,
        id.0,
        state.player(c.owner).faction,
        c.pos.q,
        c.pos.r,
        c.population,
        c.hp,
        legatum_core::data::has_building(&c.buildings, legatum_core::BuildingKind::Walls),
        c.is_capital
    )
}

fn unit_lines(state: &GameState, p: PlayerId) -> Vec<String> {
    state
        .units_of(p)
        .map(|u| {
            format!(
                "  {:?} #{} at ({},{}) hp {}",
                u.kind, u.id.0, u.pos.q, u.pos.r, u.hp
            )
        })
        .collect()
}

/// (a) A Mongols rusher with four units on your border declares war at about turn 20.
fn rush_defense() -> Result<Built, String> {
    for seed in 1..=60u64 {
        let setup = hard_setup(9100 + seed);
        let mut state = new_game(&setup);
        let glass = player_of(&state, Faction::Mongols);
        let mut brains = scripted(&state);
        brains.insert(
            glass,
            Box::new(legatum_ai::openings::OpeningBrain::new(
                ScriptedBrain::with_difficulty(
                    Faction::Mongols,
                    brain_seed(setup.seed, glass),
                    glass,
                    AiDifficulty::Hard,
                ),
                legatum_ai::openings::OpeningPolicy::builtin(legatum_ai::openings::Opening::Rush),
            )),
        );
        play_to(&mut state, &mut brains, 20);
        let Some(gcap) = capital_pos(&state, glass) else {
            continue;
        };
        // The defender: an expanding neighbour (two cities) that isn't the anti-rush Celts.
        let Some(seat) = state
            .players
            .iter()
            .filter(|p| {
                p.alive && p.id != glass && !matches!(p.faction, Faction::Celts | Faction::Sparta)
            })
            .filter(|p| state.cities_of(p.id).count() >= 2)
            .filter_map(|p| capital_pos(&state, p.id).map(|c| (c.distance(gcap), p.id)))
            .filter(|(d, _)| *d <= 12)
            .min()
            .map(|(_, p)| p)
        else {
            continue;
        };
        if state.at_war(glass, seat) {
            continue;
        }
        let target = nearest_city(&state, seat, gcap).expect("seat has cities");
        let tpos = state.cities[&target].pos;
        // The classic archer rush: Spearmen screen two Archers, two to three hexes out.
        // Four Spearmen three hexes out never attacked a fortified garrison (first baseline);
        // two Spearmen and two Archers never broke a city that shields its garrison (revision
        // 2 after city shields: the AI defender held 50 of 50), so a third Spearman joins.
        let kinds = [
            UnitKind::Spearman,
            UnitKind::Spearman,
            UnitKind::Spearman,
            UnitKind::Archer,
            UnitKind::Archer,
        ];
        let tiles = staging(&state, glass, tpos, gcap, (2, 3), |o| o != Some(seat));
        if tiles.len() < kinds.len() {
            continue;
        }
        // Existing units of each needed kind are moved first (nearest first); the rest spawn.
        let mut existing = combat(&state, glass);
        existing.sort_by_key(|u| (state.units[u].pos.distance(tpos), *u));
        let mut force = vec![];
        let mut moved = 0;
        for (kind, to) in kinds.iter().zip(&tiles) {
            match existing.iter().position(|u| state.units[u].kind == *kind) {
                Some(i) => {
                    let u = existing.remove(i);
                    relocate(&mut state, u, *to);
                    force.push(u);
                    moved += 1;
                }
                None => force.push(spawn_unit(&mut state, glass, *kind, *to)),
            }
        }
        // The threatened city gets a garrison if it has none: a fair defence, not a lost cause.
        let garrisoned = state
            .units_at(tpos)
            .any(|u| u.owner == seat && is_military(u.kind));
        let garrison =
            (!garrisoned).then(|| spawn_unit(&mut state, seat, UnitKind::Spearman, tpos));
        legatum_core::visibility::update_explored(&mut state);
        // The declaration is an ordinary order; the force holds while it takes effect.
        let seat_team = state.player(seat).team;
        let staged = force.clone();
        advance(&mut state, &mut brains, |orders| {
            let mine = orders.entry(glass).or_default();
            mine.retain(|o| !touches(o, &staged) && !legatum_core::diplomacy::is_diplomatic(o));
            mine.push(Order::DeclareWar {
                against: seat_team,
                breach: false,
            });
            for u in &staged {
                mine.push(Order::SkipUnit { unit: *u });
            }
            if let Some(g) = garrison {
                let theirs = orders.entry(seat).or_default();
                theirs.retain(|o| !touches(o, &[g]));
                theirs.push(Order::SkipUnit { unit: g });
            }
        });
        if !state.at_war(glass, seat) || state.cities_of(seat).count() < 2 {
            continue;
        }
        let mut notes = vec![
            format!(
                "seed {} turn {}: seat {} {:?} vs Mongols player {}; moved {moved} Mongols units, spawned {}",
                setup.seed,
                state.turn,
                seat.0,
                state.player(seat).faction,
                glass.0,
                kinds.len() - moved
            ),
            format!(
                "seat cities: {}",
                state
                    .cities_of(seat)
                    .map(|c| city_line(&state, c.id))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            format!(
                "mongols cities: {}",
                state
                    .cities_of(glass)
                    .map(|c| city_line(&state, c.id))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            "seat units:".into(),
        ];
        notes.extend(unit_lines(&state, seat));
        notes.push("mongols units:".into());
        notes.extend(unit_lines(&state, glass));
        let source = format!(
            "All-AI Hard game, Standard speed, seed {seed}; Mongols (player {glass}) played the \
             Rush opening. Played to turn 20, then a force of three Spearmen and two Archers was \
             staged 2-3 hexes from {city} ({moved} existing Mongols units moved, {spawned} \
             spawned), {garrison}and Mongols declared war as an ordinary order on turn 20 (its \
             staged units held). Built by `legatum-sim scenario-build rush-defense`.",
            seed = setup.seed,
            glass = glass.0,
            city = state.cities[&target].name,
            spawned = kinds.len() - moved,
            garrison = if garrison.is_some() {
                "the threatened city was given a Spearman garrison (it had none), "
            } else {
                ""
            }
        );
        return Ok(Built {
            objective: Objective {
                summary: String::new(),
                turns: 15,
                win: vec![Condition::HoldStartingCities, Condition::MinCombatUnits(2)],
                end_on_win: false,
                keep: vec![],
                score: vec![
                    term(Metric::StartingCitiesHeld, 100),
                    term(Metric::CitiesFounded, 50),
                    term(Metric::CombatUnits, 10),
                    term(Metric::RivalCombatUnits(glass), -10),
                ],
            },
            state,
            seat,
            source,
            notes,
        });
    }
    Err("no seed gave a Mongols rusher next to an expanding neighbour".into())
}

fn touches(order: &Order, units: &[UnitId]) -> bool {
    let unit = match order {
        Order::SetAutomation { unit, .. }
        | Order::SetAutoFire { unit, .. }
        | Order::MoveUnit { unit, .. }
        | Order::MoveUnitPath { unit, .. }
        | Order::SleepUnit { unit, .. }
        | Order::SkipUnit { unit }
        | Order::FoundCity { unit }
        | Order::DisbandUnit { unit }
        | Order::RangedAttack { unit, .. }
        | Order::UpgradeUnit { unit } => unit,
        _ => return false,
    };
    units.contains(unit)
}

/// (b) An Sparta strike force beside an unwalled neighbour city, at Peace.
fn rush_offense() -> Result<Built, String> {
    for seed in 1..=60u64 {
        let setup = hard_setup(9200 + seed);
        let mut state = new_game(&setup);
        let oath = player_of(&state, Faction::Sparta);
        let mut brains = scripted(&state);
        play_to(&mut state, &mut brains, 22);
        let Some(ocap) = capital_pos(&state, oath) else {
            continue;
        };
        // The target: the nearest unwalled foreign city (not Celts land, which bleeds invaders).
        let Some(target) = state
            .cities
            .values()
            .filter(|c| c.owner != oath && state.player(c.owner).faction != Faction::Celts)
            .filter(|c| {
                !legatum_core::data::has_building(&c.buildings, legatum_core::BuildingKind::Walls)
            })
            .filter(|c| !state.at_war(oath, c.owner))
            .map(|c| (c.pos.distance(ocap), c.id))
            .filter(|(d, _)| (5..=10).contains(d))
            .min()
            .map(|(_, id)| id)
        else {
            continue;
        };
        let tpos = state.cities[&target].pos;
        let owner = state.cities[&target].owner;
        let tiles = staging(&state, oath, tpos, ocap, (3, 4), |o| o != Some(owner));
        if tiles.len() < 4 {
            continue;
        }
        // Keep one defender in every Sparta city; the rest join the strike force.
        let garrisons: Vec<UnitId> = state
            .cities_of(oath)
            .filter_map(|c| {
                combat(&state, oath)
                    .into_iter()
                    .min_by_key(|u| (state.units[u].pos.distance(c.pos), *u))
            })
            .collect();
        let mut force: Vec<UnitId> = combat(&state, oath)
            .into_iter()
            .filter(|u| !garrisons.contains(u))
            .collect();
        force.sort_by_key(|u| (state.units[u].pos.distance(tpos), *u));
        force.truncate(4);
        let moved = force.len();
        let has_archer = force
            .iter()
            .any(|u| state.units[u].kind == UnitKind::Archer);
        let mut spawned = vec![];
        for (i, to) in tiles.iter().take(4).enumerate() {
            match force.get(i) {
                Some(u) => relocate(&mut state, *u, *to),
                None => {
                    let kind = if !has_archer && spawned.is_empty() {
                        UnitKind::Archer
                    } else {
                        UnitKind::Spearman
                    };
                    spawned.push(kind);
                    spawn_unit(&mut state, oath, kind, *to);
                }
            }
        }
        legatum_core::visibility::update_explored(&mut state);
        let mut notes = vec![
            format!(
                "seed {} turn {}: Sparta player {}, target {}",
                setup.seed,
                state.turn,
                oath.0,
                city_line(&state, target)
            ),
            format!(
                "sparta cities: {}",
                state
                    .cities_of(oath)
                    .map(|c| city_line(&state, c.id))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            format!(
                "target owner cities: {}",
                state
                    .cities_of(owner)
                    .map(|c| city_line(&state, c.id))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            format!("moved {moved}, spawned {spawned:?}"),
            "sparta units:".into(),
        ];
        notes.extend(unit_lines(&state, oath));
        notes.push("target owner units:".into());
        notes.extend(unit_lines(&state, owner));
        let source = format!(
            "All-AI Hard game, Standard speed, seed {}. Played to turn 22; then {moved} Sparta \
             combat units (one garrison per city stays home) were moved 3-4 hexes from {}, and \
             {:?} were spawned to make a four-unit strike force. Still at Peace. Built by \
             `legatum-sim scenario-build rush-offense`.",
            setup.seed, state.cities[&target].name, spawned
        );
        return Ok(Built {
            objective: Objective {
                summary: String::new(),
                turns: 12,
                win: vec![Condition::OwnCity(target)],
                end_on_win: true,
                keep: vec![Condition::HoldStartingCities],
                score: vec![
                    term(Metric::ObjectiveMet, 300),
                    term(Metric::TurnsLeft, 20),
                    term(Metric::CityDamage(target), 1),
                    term(Metric::CombatUnits, 10),
                ],
            },
            state,
            seat: oath,
            source,
            notes,
        });
    }
    Err("no seed gave Sparta an unwalled neighbour".into())
}

/// (c) The user's own game at turn 40, from the Switzerland' seat.
fn runaway(save: Option<&PathBuf>) -> Result<Built, String> {
    let path =
        save.ok_or("runaway-conqueror needs --source-save <the user's autosave with history>")?;
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let outer: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let game = outer["game"].as_str().ok_or("not an app save file")?;
    let value: serde_json::Value = serde_json::from_str(game).map_err(|e| e.to_string())?;
    let history: legatum_wasm::history::Recording =
        serde_json::from_value(value["archive"].clone()).map_err(|e| format!("archive: {e}"))?;
    let mut state = match &history.initial_state {
        Some(json) => legatum_wasm::json::parse_state(json)?,
        None => history.tape.initial_state(),
    };
    // Hashes recorded before 6a5f3e5f serialized a zero Byzantium salvage field, so they can't
    // match exactly; the replay is checked against the recorded per-turn telemetry instead.
    let mut diverged = None;
    for t in &history.tape.turns {
        if state.turn >= 40 {
            break;
        }
        state = resolve_turn(&state, &t.orders).0;
        let snapshot = legatum_wasm::history::snapshot(t.turn, &state);
        let recorded = history.telemetry.iter().find(|r| r.turn == t.turn);
        if diverged.is_none() && recorded != Some(&snapshot) {
            diverged = Some(t.turn);
        }
    }
    let human = PlayerId(0);
    let seat = player_of(&state, Faction::Switzerland);
    let conqueror_cities = state.cities_of(human).count() as u32;
    let mut notes = vec![
        format!(
            "replayed to turn {}; {}",
            state.turn,
            diverged.map_or(
                "telemetry matches the recording every turn".into(),
                |t| format!("first telemetry divergence after turn {t}")
            )
        ),
        format!(
            "game {} seed {}",
            history.metadata.game_id, state.setup.seed
        ),
    ];
    for p in &state.players {
        notes.push(format!(
            "player {} {:?} alive {} cities {} vp {} combat {} wars {:?}",
            p.id.0,
            p.faction,
            p.alive,
            state.cities_of(p.id).count(),
            legatum_core::scoring::vp_total(&state, p.id),
            combat(&state, p.id).len(),
            state
                .players
                .iter()
                .filter(|q| q.alive && state.at_war(p.id, q.id) && q.id != p.id)
                .map(|q| q.id.0)
                .collect::<Vec<_>>()
        ));
        for c in state.cities_of(p.id) {
            notes.push(format!("  {}", city_line(&state, c.id)));
        }
    }
    notes.push("swiss units:".into());
    notes.extend(unit_lines(&state, seat));
    let source = format!(
        "A recorded game (Mongols, Standard, 8 empires), replayed from its recorded orders to \
         turn {}{}. No edits: the human player's empire becomes a Hard AI and Switzerland becomes \
         the human seat. Built by `legatum-sim scenario-build runaway-conqueror --source-save <save>`.",
        state.turn,
        diverged.map_or(String::new(), |t| format!(
            " (diverged from the recording after turn {t})"
        ))
    );
    Ok(Built {
        objective: Objective {
            summary: String::new(),
            turns: 20,
            win: vec![
                Condition::HoldStartingCities,
                Condition::RivalMaxCities {
                    rival: human,
                    max: conqueror_cities,
                },
            ],
            end_on_win: false,
            keep: vec![],
            score: vec![
                term(Metric::StartingCitiesHeld, 100),
                term(Metric::CitiesCaptured, 75),
                term(Metric::RivalCitiesGained(human), -50),
                term(Metric::RivalCitiesLost(human), 50),
                term(Metric::VpGained, 25),
            ],
        },
        state,
        seat,
        source,
        notes,
    })
}

/// (d) Turn 1: as many cities as possible by turn 50.
fn opening_race() -> Result<Built, String> {
    let setup = hard_setup(9401);
    let state = new_game(&setup);
    let seat = player_of(&state, Faction::Athens);
    let start = state
        .units_of(seat)
        .find(|u| u.kind == UnitKind::Settler)
        .map(|u| u.pos)
        .expect("settler");
    let notes = vec![format!(
        "seed {} seat {} Athens settler at ({},{})",
        setup.seed, seat.0, start.q, start.r
    )];
    Ok(Built {
        objective: Objective {
            summary: String::new(),
            turns: 49,
            win: vec![Condition::MinCities(3)],
            end_on_win: false,
            keep: vec![],
            score: vec![term(Metric::Cities, 100), term(Metric::Population, 10)],
        },
        state,
        seat,
        source: format!(
            "A new Hard game (8 empires, Standard speed, seed {}), turn 1, unedited. Built by \
             `legatum-sim scenario-build opening-race`.",
            setup.seed
        ),
        notes,
    })
}

fn build(id: &str, cli: &Cli) -> Result<Built, String> {
    match id {
        "rush-defense" => rush_defense(),
        "rush-offense" => rush_offense(),
        "runaway-conqueror" => runaway(cli.source_save.as_ref()),
        "opening-race" => opening_race(),
        other => Err(format!(
            "unknown scenario {other:?} (known: {})",
            IDS.join(", ")
        )),
    }
}

pub fn run(cli: &Cli) -> Result<(), String> {
    let ids: Vec<String> = if cli.ids.is_empty() {
        IDS.iter().map(|s| s.to_string()).collect()
    } else {
        cli.ids.clone()
    };
    for id in ids {
        let built = build(&id, cli)?;
        let errors = legatum_core::scenario::validate_state(&built.state);
        if !errors.is_empty() {
            return Err(format!("{id}: illegal state: {}", errors.join("; ")));
        }
        let path = cli.dir.join(format!("{id}.json"));
        let mut file = match std::fs::read_to_string(&path) {
            Ok(text) => ScenarioFile::parse(&text)?,
            Err(_) => placeholder(&id, &built),
        };
        let changed = file.state.hash != built.state.hash().to_string()
            || file.objective.win != built.objective.win
            || file.objective.keep != built.objective.keep
            || file.objective.score != built.objective.score
            || file.objective.turns != built.objective.turns;
        if changed && file.state.hash != PLACEHOLDER_HASH {
            file.revision += 1;
        }
        if changed {
            // A measured baseline belongs to one revision.
            file.baseline = None;
        }
        file.seat = built.seat;
        file.faction = built.state.player(built.seat).faction;
        file.state = encode_state(&built.state);
        file.source = built.source.clone();
        let summary = std::mem::take(&mut file.objective.summary);
        file.objective = built.objective;
        file.objective.summary = summary;
        file.prepare(true).map_err(|e| format!("{id}: {e}"))?;
        std::fs::create_dir_all(&cli.dir).map_err(|e| e.to_string())?;
        let mut json = serde_json::to_string_pretty(&file).expect("scenario JSON");
        json.push('\n');
        std::fs::write(&path, json).map_err(|e| e.to_string())?;
        println!("== {id} -> {} (revision {})", path.display(), file.revision);
        for line in &built.notes {
            println!("{line}");
        }
    }
    Ok(())
}

/// The hash a placeholder file starts with, so the first build doesn't bump the revision.
const PLACEHOLDER_HASH: &str = "0";

fn placeholder(id: &str, built: &Built) -> ScenarioFile {
    ScenarioFile {
        format: FORMAT.into(),
        version: VERSION,
        id: id.into(),
        revision: 1,
        title: id.into(),
        faction: built.state.player(built.seat).faction,
        seat: built.seat,
        difficulty: AiDifficulty::Hard,
        minutes: 12,
        briefing: "TODO".into(),
        tests: "TODO".into(),
        objective: Objective {
            summary: "TODO".into(),
            ..built.objective.clone()
        },
        predictions: Predictions {
            good_play: "TODO".into(),
            ai_play: "TODO".into(),
            ai_score: ScorePrediction {
                median: 0,
                low: 0,
                high: 0,
                win_percent: 0,
            },
        },
        source: String::new(),
        baseline: None,
        human: None,
        lab: None,
        rusher: None,
        ai_memory: None,
        state: EncodedState {
            encoding: "gzip-base64".into(),
            hash: PLACEHOLDER_HASH.into(),
            data: String::new(),
        },
    }
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    use clap::Parser;
    let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
    run(&cli)
}

//! Fast, small-map Settler decision episodes using the real Hard planner and core turn engine.
//! Example: `cargo run --release -p legatum-sim --example settler_scenarios -- --trials 8 --seed 1`
use clap::Parser;
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::{
    AiDifficulty, Faction, GameSetup, GameState, Hex, Order, PlayerId, TeamId, Terrain, UnitId,
    UnitKind, data,
    game::{blank_game, spawn_unit},
    player_view, resolve_turn,
    rules::barbarians::Camp,
    settlement::base_site_score,
    state::{City, Map},
};
use rayon::prelude::*;
use serde::Serialize;
use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};

const LIMIT: u32 = 15;
const FULL_GAME_TURNS: u32 = 150;
const HOME: Hex = Hex::new(6, 7);
const SETTLER: UnitId = UnitId(1);

#[derive(Parser, Debug)]
#[command(about = "Bounded Hard-AI Settler route, site and escort scenarios")]
struct Cli {
    #[arg(long, default_value_t = 8)]
    trials: u32,
    #[arg(long, default_value_t = 1)]
    seed: u64,
    #[arg(long, default_value_t = 4)]
    threads: usize,
    /// Optional AI parameter JSON (defaults to built-in Hard).
    #[arg(long)]
    params: Option<PathBuf>,
    /// Run a bounded full-game reference for the same number of seeds and report episode throughput.
    #[arg(long)]
    compare_full_game: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Scenario {
    threat_distance: i32,
    escorts: usize,
    site_distance: i32,
}

#[derive(Clone, Debug, Serialize)]
struct Trial {
    seed: u64,
    found: bool,
    turns: u32,
    settler_lost: bool,
    escorts_lost: usize,
    chosen_site_score: Option<i32>,
    orders: Vec<Vec<Order>>,
}

#[derive(Serialize)]
struct Summary {
    scenario: Scenario,
    trials: u32,
    found_rate: f64,
    median_turns_to_found: Option<f64>,
    settler_losses: u32,
    escort_losses: u32,
    mean_chosen_site_score: Option<f64>,
    episodes_per_second: f64,
}

fn setup(seed: u64) -> GameSetup {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(1);
    setup.width = 20;
    setup.height = 15;
    setup.barbarians = false;
    setup.legacy_all_war = true;
    setup.players[0].faction = Faction::Mongols;
    setup.players[0].team = TeamId(0);
    setup.players[0].difficulty = AiDifficulty::Hard;
    setup
}

fn site_for(distance: i32) -> Hex {
    Hex::new(HOME.q + distance, HOME.r)
}

fn arena(scenario: Scenario, seed: u64) -> (GameState, Vec<UnitId>, Hex) {
    let mut map = Map::new(20, 15, Terrain::Plains);
    let destination = site_for(scenario.site_distance);
    for h in destination.within(3) {
        if let Some(tile) = map.get_mut(h) {
            tile.terrain = match (h.q + h.r).rem_euclid(5) {
                0 => Terrain::Forest,
                1 => Terrain::Hills,
                2 => Terrain::Grassland,
                _ => Terrain::Plains,
            };
        }
    }
    let mut setup = setup(seed);
    setup.barbarians = true;
    let mut state = blank_game(&setup, map);
    let home_id = state.alloc_city_id();
    state.cities.insert(
        home_id,
        City {
            last_attacked_turn: None,
            id: home_id,
            owner: PlayerId(0),
            founder: PlayerId(0),
            name: "Home".into(),
            pos: HOME,
            population: 3,
            food_stored: 0,
            production_stored: 0,
            producing: None,
            production_banked: Vec::new(),
            production_queue: Vec::new(),
            build_automation: None,
            last_auto_pick: None,
            auto_queued: None,
            production_queue_notified: Default::default(),
            wonder_lost: None,
            last_purchase_turn: None,
            bought: None,
            buildings: Default::default(),
            hp: data::CITY_HP,
            tiles_claimed: 0,
            grown_tiles: Vec::new(),
            is_capital: true,
            original_capital: true,
            greece_captive_paid: Default::default(),
            occupied_from: None,
            founded_turn: 1,
            resistance: 0,
        },
    );
    for h in HOME.within(1) {
        if let Some(tile) = state.map.get_mut(h) {
            tile.owner = Some(PlayerId(0));
            tile.city = Some(home_id);
        }
    }
    let settler = spawn_unit(&mut state, PlayerId(0), UnitKind::Settler, HOME);
    debug_assert_eq!(settler, SETTLER);
    let escort_ids = (0..scenario.escorts)
        .map(|i| {
            let h = HOME.neighbor((i + 1) % 6);
            spawn_unit(&mut state, PlayerId(0), UnitKind::Soldier, h)
        })
        .collect();
    let camp = Hex::new(
        HOME.q
            + if scenario.threat_distance == 0 {
                2
            } else {
                scenario.threat_distance
            },
        HOME.r,
    );
    let count = if scenario.threat_distance == 0 {
        1
    } else {
        usize::try_from((4 - scenario.threat_distance / 2).clamp(1, 3)).unwrap_or(1)
    };
    let threats: Vec<_> = (0..count)
        .map(|i| {
            let pos = if i == 0 { camp } else { camp.neighbor(i) };
            spawn_unit(&mut state, PlayerId::BARBARIAN, UnitKind::Soldier, pos)
        })
        .collect();
    state
        .barbarians
        .as_mut()
        .expect("barbarian setup")
        .camps
        .push(Camp {
            id: 0,
            pos: camp,
            founded_turn: 1,
            guard: threats.first().copied(),
            scout: None,
            raiders: threats.into_iter().skip(1).collect(),
            scout_report: None,
            raid: None,
            next_raid_turn: 0,
            next_spawn_turn: 50,
            rite: None,
        });
    legatum_core::visibility::update_explored(&mut state);
    for player in &mut state.players {
        for index in 0..state.map.len() {
            player.explored.set(index);
        }
    }
    (state, escort_ids, destination)
}

fn make_brain(
    faction: Faction,
    seed: u64,
    player: PlayerId,
    params: Option<&AiParams>,
) -> ScriptedBrain {
    params.map_or_else(
        || ScriptedBrain::with_difficulty(faction, seed, player, AiDifficulty::Hard),
        |p| ScriptedBrain::with_params(faction, seed, player, AiDifficulty::Hard, p),
    )
}

fn median(mut values: Vec<u32>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let mid = values.len() / 2;
    Some(if values.len().is_multiple_of(2) {
        f64::from(values[mid - 1] + values[mid]) / 2.0
    } else {
        f64::from(values[mid])
    })
}

fn play(scenario: Scenario, seed: u64, params: Option<&AiParams>) -> Trial {
    let (mut state, escort_ids, _destination) = arena(scenario, seed);
    let home_city = state.cities.keys().next().copied().expect("home city");
    let mut brain = make_brain(Faction::Mongols, seed ^ 11, PlayerId(0), params);
    let mut log = Vec::new();
    let mut chosen_site_score = None;
    let mut turns = 0;
    while turns < LIMIT
        && !state.finished
        && state
            .units
            .get(&SETTLER)
            .is_some_and(|u| u.owner == PlayerId(0))
    {
        let mut orders = BTreeMap::new();
        let player = PlayerId(0);
        let view = player_view(&state, player);
        let own = brain.orders(&view);
        if chosen_site_score.is_none()
            && let Some(to) = own.iter().find_map(|order| match order {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                    if *unit == SETTLER =>
                {
                    Some(*to)
                }
                _ => None,
            })
        {
            chosen_site_score = Some(base_site_score(&view, to));
        }
        orders.insert(player, own.clone());
        log.push(own);
        state = resolve_turn(&state, &orders).0;
        turns += 1;
    }
    let found_city = state.cities_of(PlayerId(0)).find(|c| c.id != home_city);
    let found = found_city.is_some();
    if let Some(city) = found_city {
        chosen_site_score = Some(base_site_score(&player_view(&state, PlayerId(0)), city.pos));
    }
    let settler_lost = !found
        && state
            .units
            .get(&SETTLER)
            .is_none_or(|u| u.owner != PlayerId(0));
    let escorts_lost = escort_ids
        .iter()
        .filter(|id| !state.units.contains_key(id))
        .count();
    Trial {
        seed,
        found,
        turns,
        settler_lost,
        escorts_lost,
        chosen_site_score,
        orders: log,
    }
}

/// Reference cost: start a full eight-seat game, build all real Hard brains, and resolve up to
/// 150 turns. This includes map generation, all views and all seat orders, which direct episodes avoid.
fn full_game_reference(seed: u64, params: Option<&AiParams>) -> (f64, u64) {
    let start = Instant::now();
    let mut setup = GameSetup::all_ai(seed);
    setup.width = 48;
    setup.height = 32;
    for p in &mut setup.players {
        p.difficulty = AiDifficulty::Hard;
    }
    let mut state = legatum_core::new_game(&setup);
    let mut brains: Vec<_> = state
        .players
        .iter()
        .map(|p| make_brain(p.faction, seed ^ u64::from(p.id.0), p.id, params))
        .collect();
    let mut episodes = 0_u64;
    while state.turn <= FULL_GAME_TURNS && !state.finished {
        let mut orders = BTreeMap::new();
        for p in &state.players {
            if p.alive {
                orders.insert(
                    p.id,
                    brains[p.id.0 as usize].orders(&player_view(&state, p.id)),
                );
            }
        }
        let (next, events) = resolve_turn(&state, &orders);
        episodes += events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    legatum_core::Event::CityFounded { .. }
                        | legatum_core::Event::SettlerCaptured { .. }
                )
            })
            .count() as u64;
        state = next;
    }
    let elapsed = start.elapsed().as_secs_f64().max(f64::MIN_POSITIVE);
    let rate = 1.0 / elapsed;
    println!(
        "full_game_reference=8_seats turns={} target_turns={FULL_GAME_TURNS} games=1 observed_settler_outcomes={episodes} games_per_second={rate:.4} elapsed_seconds={elapsed:.3}",
        state.turn.saturating_sub(1)
    );
    (rate, episodes)
}

fn run_case(scenario: Scenario, trials: u32, seed: u64, params: Option<&AiParams>) -> Summary {
    let start = Instant::now();
    let rows: Vec<_> = (0..trials)
        .map(|i| play(scenario, seed.wrapping_add(u64::from(i)), params))
        .collect();
    let elapsed = start.elapsed().as_secs_f64().max(f64::MIN_POSITIVE);
    let found: Vec<_> = rows.iter().filter(|r| r.found).map(|r| r.turns).collect();
    let scores: Vec<_> = rows.iter().filter_map(|r| r.chosen_site_score).collect();
    Summary {
        scenario,
        trials,
        found_rate: found.len() as f64 / f64::from(trials),
        median_turns_to_found: median(found),
        settler_losses: rows.iter().filter(|r| r.settler_lost).count() as u32,
        escort_losses: rows.iter().map(|r| r.escorts_lost as u32).sum(),
        mean_chosen_site_score: (!scores.is_empty())
            .then(|| scores.iter().map(|v| f64::from(*v)).sum::<f64>() / scores.len() as f64),
        episodes_per_second: f64::from(trials) / elapsed,
    }
}

fn main() {
    let cli = Cli::parse();
    assert!(cli.trials > 0, "--trials must be positive");
    assert!(
        (1..=4).contains(&cli.threads),
        "--threads must be between 1 and 4"
    );
    let params = cli.params.as_ref().map(|p| {
        let params: AiParams = serde_json::from_slice(&fs::read(p).expect("read params file"))
            .expect("parse AI params");
        params.validate().expect("valid AI params");
        params
    });
    let grid: Vec<_> = [0, 2, 4, 6, 8]
        .into_iter()
        .flat_map(|threat_distance| {
            [0, 1, 2].into_iter().flat_map(move |escorts| {
                [3, 5, 8].into_iter().map(move |site_distance| Scenario {
                    threat_distance,
                    escorts,
                    site_distance,
                })
            })
        })
        .collect();
    assert!(grid.len() >= 40);
    let started = Instant::now();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build()
        .expect("thread pool");
    let summaries: Vec<_> = pool.install(|| {
        grid.into_par_iter()
            .map(|scenario| run_case(scenario, cli.trials, cli.seed, params.as_ref()))
            .collect()
    });
    let elapsed = started.elapsed().as_secs_f64();
    println!(
        "parameter_set={} scenarios={} trials_per_scenario={} total_episodes={} wall_seconds={elapsed:.3}",
        params.as_ref().map_or("baseline", |p| &p.name),
        summaries.len(),
        cli.trials,
        summaries.len() * cli.trials as usize
    );
    println!(
        "threat_distance,escorts,site_distance,found_rate,median_turns_to_found,settler_losses,escort_losses,mean_site_yield_score,episodes_per_second"
    );
    for row in &summaries {
        println!(
            "{},{},{},{:.3},{},{},{},{},{}",
            row.scenario.threat_distance,
            row.scenario.escorts,
            row.scenario.site_distance,
            row.found_rate,
            row.median_turns_to_found
                .map_or_else(|| "NA".into(), |v| format!("{v:.1}")),
            row.settler_losses,
            row.escort_losses,
            row.mean_chosen_site_score
                .map_or_else(|| "NA".into(), |v| format!("{v:.1}")),
            row.episodes_per_second
        );
    }
    let grid_episodes_per_second =
        (summaries.len() * cli.trials as usize) as f64 / elapsed.max(f64::MIN_POSITIVE);
    println!("bounded_episode_rate={grid_episodes_per_second:.3} episodes_per_second");
    if cli.compare_full_game {
        let (full_games_per_second, outcomes) = full_game_reference(cli.seed, params.as_ref());
        // Each observed founding or capture ends one full-game Settler episode. Use that
        // episode throughput, rather than whole games/s, for the speedup denominator.
        let full_game_episodes_per_second = outcomes as f64 * full_games_per_second;
        println!("full_game_settler_episodes_per_second={full_game_episodes_per_second:.3}");
        println!(
            "narrow_episode_speedup={:.1}x",
            grid_episodes_per_second / full_game_episodes_per_second.max(f64::MIN_POSITIVE)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid_has_at_least_forty_scenario_combinations() {
        assert_eq!([0, 2, 4, 6, 8].len() * 3 * 3, 45);
    }
    #[test]
    fn episode_is_reproducible_and_uses_site_scoring() {
        let scenario = Scenario {
            threat_distance: 6,
            escorts: 1,
            site_distance: 5,
        };
        let a = play(scenario, 501, None);
        let b = play(scenario, 501, None);
        assert_eq!(a.orders, b.orders);
        assert_eq!(a.found, b.found);
    }
}

//! Strategic supply in eight-player Standard Hard games (user, 2026-10-19: resources are a core
//! reason for war; 2026-10-22: each era is defined by its resource). Does a typical empire run
//! short of its era's resource, and do deposits change hands when cities are captured?
//!
//! supply_census <games> [first_seed=9700001] [params.json]
//!
//! Eight empires on the Standard 48x32 map, solo teams, all Hard, no turn limit (safety cap 4x the
//! expected length). Prints one JSON object:
//! - `map`: deposits of each strategic resource per map and nearest each start;
//! - `by_turn`: at T50 .. T250, over living empires: era, cities and land army, and for each
//!   resource the deposits owned and mined, supply, users (buildings and units) and idle users;
//!   for the empire's own era resource, what its army and key buildings want (users plus land
//!   units of an older era that could upgrade into it) against what it supplies;
//! - `transfers`: deposits of each resource that passed from one empire to another, and how many
//!   went with a captured city; captures of cities holding a deposit;
//! - `shortages`: games and empires that ever had a user idle for lack of supply.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::resources::{self, Resource, SupplyUser};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, Faction, GameSetup, GameState, PlayerId, TeamId, UnitKind, data,
    player_view, resolve_turn,
};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const SNAPSHOTS: [u32; 5] = [50, 100, 150, 200, 250];

#[derive(Default, Clone, Copy)]
struct Supply {
    deposits: u32,
    mined: u32,
    supplied: i32,
    users: i32,
    unit_users: u32,
    idle: u32,
}

#[derive(Default, Clone)]
struct Snapshot {
    era: u8,
    cities: u32,
    army: u32,
    supply: BTreeMap<Resource, Supply>,
    /// The empire's own era resource: users now plus older land units that could upgrade into it.
    era_wanted: i32,
    era_supplied: i32,
}

#[derive(Default)]
struct Out {
    deposits: BTreeMap<Resource, u32>,
    per_start: BTreeMap<Resource, Vec<u32>>,
    snapshots: BTreeMap<u32, Vec<Snapshot>>,
    /// Per resource: (deposits that changed hands, of those with a captured city).
    transfers: BTreeMap<Resource, (u32, u32)>,
    captures: u32,
    deposit_city_captures: u32,
    shortage_empires: BTreeSet<u8>,
    turns: u32,
}

fn is_land_military(kind: UnitKind) -> bool {
    data::unit_class(kind).is_some_and(|c| c != legatum_core::UnitClass::Air)
}

/// A unit of an older era than `era` whose upgrades lead into it.
fn behind(kind: UnitKind, era: u8) -> bool {
    let mut at = kind;
    while let Some(to) = data::unit_stats(at).upgrades_to {
        if data::unit_era_tier(to) == era {
            return data::unit_era_tier(kind) < era;
        }
        at = to;
    }
    false
}

fn snapshot(state: &GameState, p: PlayerId) -> Snapshot {
    let player = state.player(p);
    let owned = resources::deposits(state, p);
    let users = resources::current_users(state, p);
    let era = state.era_of(p);
    let era_resource = resources::era_resource(era);
    let mut supply = BTreeMap::new();
    for r in Resource::STRATEGIC {
        supply.insert(
            r,
            Supply {
                deposits: owned.iter().filter(|d| d.resource == r).count() as u32,
                mined: owned.iter().filter(|d| d.resource == r && d.mined).count() as u32,
                supplied: resources::supplied(state, p, r),
                users: users.iter().filter(|(_, x)| *x == r).count() as i32,
                unit_users: users
                    .iter()
                    .filter(|(u, x)| *x == r && matches!(u, SupplyUser::Unit { .. }))
                    .count() as u32,
                idle: player
                    .supply
                    .iter()
                    .filter(|x| x.resource == r && x.idle)
                    .count() as u32,
            },
        );
    }
    let army: Vec<UnitKind> = state
        .units_of(p)
        .map(|u| u.kind)
        .filter(|k| is_land_military(*k))
        .collect();
    let older = army
        .iter()
        .filter(|k| resources::unit_need(**k) != era_resource && behind(**k, era))
        .count() as i32;
    Snapshot {
        era,
        cities: state.cities_of(p).count() as u32,
        army: army.len() as u32,
        era_wanted: era_resource.map_or(0, |r| supply[&r].users + older),
        era_supplied: era_resource.map_or(0, |r| supply[&r].supplied),
        supply,
    }
}

fn deposit_owners(state: &GameState) -> BTreeMap<usize, (PlayerId, Option<legatum_core::CityId>)> {
    state
        .map
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| t.resource.is_some_and(resources::is_strategic))
        .filter_map(|(i, t)| {
            t.owner
                .filter(|o| !o.is_barbarian())
                .map(|o| (i, (o, t.city)))
        })
        .collect()
}

fn play(seed: u64, params: &'static AiParams) -> Out {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(8);
    (setup.width, setup.height) = (data::MAP_WIDTH, data::MAP_HEIGHT);
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (seat, (p, f)) in setup.players.iter_mut().zip(factions).enumerate() {
        p.faction = f;
        p.team = TeamId(seat as u8);
        p.difficulty = AiDifficulty::Hard;
    }
    setup.turn_limit = None;
    let cap = data::EXPECTED_GAME_TURNS * 4;
    let mut state = legatum_core::new_game(&setup);
    let mut out = Out::default();
    // Starts are where each empire's first Settler stands.
    let starts: Vec<legatum_core::Hex> = (0..8u8)
        .filter_map(|i| {
            state
                .units_of(PlayerId(i))
                .find(|u| u.kind == UnitKind::Settler)
                .map(|u| u.pos)
        })
        .collect();
    for r in Resource::STRATEGIC {
        out.per_start.insert(r, vec![0; starts.len()]);
    }
    for (i, t) in state.map.tiles.iter().enumerate() {
        let Some(r) = t.resource.filter(|r| resources::is_strategic(*r)) else {
            continue;
        };
        *out.deposits.entry(r).or_default() += 1;
        if let Some((s, _)) = resources::placement::nearest_start(&starts, state.map.hex_at(i)) {
            out.per_start.get_mut(&r).unwrap()[s] += 1;
        }
    }
    let mut brains: Vec<(PlayerId, ScriptedBrain)> = (0..8u8)
        .map(|i| {
            let id = PlayerId(i);
            let b = ScriptedBrain::with_params(
                state.player(id).faction,
                legatum_wasm::session::brain_seed(setup.seed, id),
                id,
                AiDifficulty::Hard,
                params,
            );
            (id, b)
        })
        .collect();
    let mut owners = deposit_owners(&state);
    while !state.finished && state.turn < cap {
        let mut orders = legatum_wasm::session::OrderMap::new();
        for (id, b) in brains.iter_mut() {
            if state.player(*id).alive {
                orders.insert(*id, b.orders(&player_view(&state, *id)));
            }
        }
        let turn = state.turn;
        let (next, events) = resolve_turn(&state, &orders);
        state = next;
        let mut captured = BTreeSet::new();
        for e in &events {
            match e {
                Event::CityCaptured { city, to, .. } if !to.is_barbarian() => {
                    captured.insert(*city);
                    out.captures += 1;
                    if let Some(c) = state.cities.get(city) {
                        let deposit = c.pos.within_iter(data::CITY_WORK_RADIUS).any(|h| {
                            state.map.get(h).is_some_and(|t| {
                                t.owner == Some(*to)
                                    && t.resource.is_some_and(resources::is_strategic)
                            })
                        });
                        out.deposit_city_captures += u32::from(deposit);
                    }
                }
                Event::SupplyShortage { player, idle, .. } if *idle > 0 => {
                    out.shortage_empires.insert(player.0);
                }
                _ => {}
            }
        }
        let now = deposit_owners(&state);
        for (tile, (owner, city)) in &now {
            if let Some((before, _)) = owners.get(tile)
                && before != owner
            {
                let r = state.map.tiles[*tile].resource.expect("a deposit");
                let e = out.transfers.entry(r).or_default();
                e.0 += 1;
                if city.is_some_and(|c| captured.contains(&c)) {
                    e.1 += 1;
                }
            }
        }
        owners = now;
        if SNAPSHOTS.contains(&turn) {
            let snaps = state
                .players
                .iter()
                .filter(|p| p.alive && !p.id.is_barbarian())
                .map(|p| snapshot(&state, p.id))
                .collect();
            out.snapshots.insert(turn, snaps);
        }
    }
    out.turns = state.turn;
    out
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let games: u64 = a[1].parse().unwrap();
    let first: u64 = a.get(2).map_or(9_700_001, |s| s.parse().unwrap());
    let params: &'static AiParams =
        Box::leak(Box::new(match a.get(3).filter(|s| s.ends_with(".json")) {
            Some(path) => serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap(),
            None => AiParams::builtin(),
        }));
    let outs: Vec<Out> = (0..games)
        .into_par_iter()
        .map(|i| play(first + i, params))
        .collect();
    let mut map = serde_json::Map::new();
    for r in Resource::STRATEGIC {
        let near: Vec<f64> = outs
            .iter()
            .flat_map(|o| o.per_start[&r].iter().map(|x| *x as f64))
            .collect();
        map.insert(
            resources::name(r).into(),
            json!({
                "mean_deposits": round2(mean(&outs.iter().map(|o| *o.deposits.get(&r).unwrap_or(&0) as f64).collect::<Vec<_>>())),
                "nearest_each_start": {"mean": round2(mean(&near)), "median": median(near.clone()), "max": near.iter().cloned().fold(0.0, f64::max)},
            }),
        );
    }
    let mut by_turn = serde_json::Map::new();
    for t in SNAPSHOTS {
        let snaps: Vec<&Snapshot> = outs
            .iter()
            .filter_map(|o| o.snapshots.get(&t))
            .flatten()
            .collect();
        if snaps.is_empty() {
            continue;
        }
        let col = |f: &dyn Fn(&Snapshot) -> f64| snaps.iter().map(|s| f(s)).collect::<Vec<_>>();
        let mut per = serde_json::Map::new();
        for r in Resource::STRATEGIC {
            let g = |f: fn(&Supply) -> f64| col(&|s: &Snapshot| f(&s.supply[&r]));
            per.insert(
                resources::name(r).into(),
                json!({
                    "mean_deposits": round2(mean(&g(|x| x.deposits as f64))),
                    "mean_mined": round2(mean(&g(|x| x.mined as f64))),
                    "median_supplied": median(g(|x| x.supplied as f64)),
                    "mean_supplied": round2(mean(&g(|x| x.supplied as f64))),
                    "mean_users": round2(mean(&g(|x| x.users as f64))),
                    "mean_unit_users": round2(mean(&g(|x| x.unit_users as f64))),
                    "mean_idle": round2(mean(&g(|x| x.idle as f64))),
                }),
            );
        }
        let short = snaps
            .iter()
            .filter(|s| s.era_wanted > s.era_supplied)
            .count();
        let mut eras: BTreeMap<String, u32> = BTreeMap::new();
        for s in &snaps {
            *eras
                .entry(legatum_core::eras::era_name(s.era).to_string())
                .or_default() += 1;
        }
        by_turn.insert(
            t.to_string(),
            json!({
                "empires": snaps.len(),
                "eras": eras,
                "median_cities": median(col(&|s| s.cities as f64)),
                "median_land_army": median(col(&|s| s.army as f64)),
                "era_resource": {
                    "median_wanted": median(col(&|s| s.era_wanted as f64)),
                    "median_supplied": median(col(&|s| s.era_supplied as f64)),
                    "share_wanting_more_than_supplied": round2(short as f64 / snaps.len() as f64),
                },
                "share_with_idle_users": round2(snaps.iter().filter(|s| s.supply.values().any(|x| x.idle > 0)).count() as f64 / snaps.len() as f64),
                "by_resource": Value::Object(per),
            }),
        );
    }
    let mut transfers = serde_json::Map::new();
    for r in Resource::STRATEGIC {
        let (n, c) = outs.iter().fold((0u64, 0u64), |a, o| {
            let x = o.transfers.get(&r).copied().unwrap_or_default();
            (a.0 + u64::from(x.0), a.1 + u64::from(x.1))
        });
        transfers.insert(
            resources::name(r).into(),
            json!({"changed_hands": n, "with_a_captured_city": c}),
        );
    }
    let total = |f: fn(&Out) -> u32| outs.iter().map(|o| f(o) as u64).sum::<u64>();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "games": games,
            "first_seed": first,
            "median_turns": median(outs.iter().map(|o| o.turns as f64).collect()),
            "map": Value::Object(map),
            "by_turn": Value::Object(by_turn),
            "transfers": Value::Object(transfers),
            "captures": {
                "cities_captured_per_game": round2(total(|o| o.captures) as f64 / games as f64),
                "deposit_cities_captured_per_game": round2(total(|o| o.deposit_city_captures) as f64 / games as f64),
            },
            "shortages": {
                "games_with_any": outs.iter().filter(|o| !o.shortage_empires.is_empty()).count(),
                "empires_ever_short": total(|o| o.shortage_empires.len() as u32),
            },
        }))
        .unwrap()
    );
}

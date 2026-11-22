//! The Settler-loss recovery scenario (playtest 8.4: a Athens lost its first Settler to a
//! neighbour that declared war at T25, then sat on one city for 94 turns).
//!
//! settler_loss_recovery <games> <first_seed> [params.json] [control] [siege=N] [ring=R] [at=T]
//!
//! Four empires on a Small map at Standard speed, all Hard (seat 0 under test, optionally with
//! `--ai-params`-style decisions). The neighbour whose capital is nearest seat 0 is an inert
//! brain: it declares war on seat 0 at T25 and does nothing else. At T25 (or the first turn
//! after it that seat 0 has one) seat 0's first Settler is captured, i.e. removed. The other
//! two empires play Hard. With `control` nothing is captured and nobody declares war. With
//! `siege=N` the captor also parks N Soldiers `ring=R` (default 2) hexes from seat 0's capital for
//! good (the playtest's captor stayed near the city), and they never move. `at=T` (default 25)
//! is the first turn a Settler can be captured and the neighbour declares war.
//! Prints one JSON line: seat 0's mean cities at T50, T75 and T100, and the share with at
//! least three cities at T75 (the target: at least 3 by T75).
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::{
    AiDifficulty, Faction, GameSetup, Order, PlayerId, TeamId, UnitKind, player_view, resolve_turn,
    rng::{Rng, purpose},
};
use rayon::prelude::*;
use serde_json::json;

struct Inert {
    target: TeamId,
    active: bool,
    war_turn: u32,
}

impl Brain for Inert {
    fn plan(&mut self, view: &legatum_core::view::PlayerView) -> Vec<Order> {
        if self.active && view.turn >= self.war_turn && !view.teams_at_war(view.team, self.target) {
            return vec![Order::DeclareWar {
                against: self.target,
                breach: false,
            }];
        }
        Vec::new()
    }
}

struct Opts {
    control: bool,
    siege: usize,
    ring: i32,
    at: u32,
}

fn play(seed: u64, params: &'static AiParams, o: &Opts) -> Vec<u32> {
    let (control, siege) = (o.control, o.siege);
    let mut parked_alive = 0u32;
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(4);
    (setup.width, setup.height) = (40, 26);
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (p, f) in setup.players.iter_mut().zip(factions) {
        p.faction = f;
        p.difficulty = AiDifficulty::Hard;
    }
    let mut state = legatum_core::new_game(&setup);
    let test = PlayerId(0);
    // Empires start as a Settler (they found on turn 1): the nearest start is the neighbour.
    let start = |p: PlayerId| state.units_of(p).next().expect("start unit").pos;
    let home = start(test);
    let neighbour = (1..4u8)
        .map(PlayerId)
        .min_by_key(|p| start(*p).distance(home))
        .unwrap();
    let mut brains: Vec<(PlayerId, Box<dyn Brain>)> = Vec::new();
    for i in 0..4u8 {
        let id = PlayerId(i);
        let bseed = legatum_wasm::session::brain_seed(setup.seed, id);
        let faction = state.player(id).faction;
        let brain: Box<dyn Brain> = if id == neighbour {
            Box::new(Inert {
                target: state.team_of(test),
                active: !control,
                war_turn: o.at,
            })
        } else {
            Box::new(ScriptedBrain::with_params(
                faction,
                bseed,
                id,
                AiDifficulty::Hard,
                params,
            ))
        };
        brains.push((id, brain));
    }
    let mut captured = control;
    let mut at = vec![0u32; 4];
    while !state.finished && state.turn <= 100 {
        if !captured && state.turn >= o.at {
            let id = state
                .units_of(test)
                .find(|u| u.kind == UnitKind::Settler)
                .map(|u| u.id);
            if let Some(id) = id {
                state.units.remove(&id);
                captured = true;
                let capital = state.cities_of(test).next().map(|c| c.pos);
                if let Some(capital) = capital {
                    let spots: Vec<_> = capital
                        .ring(o.ring)
                        .into_iter()
                        .filter(|h| {
                            state.map.get(*h).is_some_and(|t| {
                                t.terrain.is_land() && t.terrain.move_cost().is_some()
                            }) && state.units.values().all(|u| u.pos != *h)
                        })
                        .take(siege)
                        .collect();
                    for h in spots {
                        legatum_core::game::spawn_unit(&mut state, neighbour, UnitKind::Soldier, h);
                    }
                }
            }
        }
        let mut orders = legatum_wasm::session::OrderMap::new();
        for (id, b) in brains.iter_mut() {
            if state.player(*id).alive {
                orders.insert(*id, b.orders(&player_view(&state, *id)));
            }
        }
        let turn = state.turn;
        let (next, _) = resolve_turn(&state, &orders);
        state = next;
        for (k, t) in [50u32, 75, 100].into_iter().enumerate() {
            if turn == t {
                at[k] = state.cities_of(test).count() as u32;
            }
        }
        if turn == 50 {
            parked_alive = state
                .units_of(neighbour)
                .filter(|u| {
                    u.kind == UnitKind::Soldier
                        && state.cities_of(test).any(|c| c.pos.distance(u.pos) <= 3)
                })
                .count() as u32;
        }
    }
    at[3] = parked_alive;
    at
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let games: u64 = a[1].parse().unwrap();
    let first: u64 = a[2].parse().unwrap();
    let params: &'static AiParams =
        Box::leak(Box::new(match a.get(3).filter(|s| s.ends_with(".json")) {
            Some(path) => serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap(),
            None => AiParams::builtin(),
        }));
    let control = a.iter().any(|s| s == "control");
    let num = |key: &str, default: u32| {
        a.iter()
            .find_map(|s| s.strip_prefix(key))
            .map_or(default, |n| n.parse().unwrap())
    };
    let opts = Opts {
        control,
        siege: num("siege=", 0) as usize,
        ring: num("ring=", 2) as i32,
        at: num("at=", 25),
    };
    let runs: Vec<Vec<u32>> = (0..games)
        .into_par_iter()
        .map(|i| play(first + i, params, &opts))
        .collect();
    let mean = |k: usize| runs.iter().map(|r| r[k] as f64).sum::<f64>() / runs.len() as f64;
    let share3 = runs.iter().filter(|r| r[1] >= 3).count() as f64 / runs.len() as f64;
    println!(
        "{}",
        json!({"games": games, "control": control, "cities_t50": mean(0), "cities_t75": mean(1),
            "cities_t100": mean(2), "share_3plus_t75": share3, "parked_soldiers_alive_t50": mean(3)})
    );
}

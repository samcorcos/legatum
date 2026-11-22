//! Four Hard + Search seats replaying the archived playtest 8.5 T14 army edge.
//! `sparta_snapshot <uncompressed-fixture.json> [games=32] [first-seed=874654782136188]`
//! Only the future game/brain RNG seeds vary; the starting map, armies and AI memories are fixed.
use legatum_ai::search::{SearchBrain, SearchConfig};
use legatum_ai::{Brain, OpponentMemory, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::scoring::team_vp;
use legatum_core::{
    AiDifficulty, Event, GameState, Order, PlayerId, UnitKind, player_view, resolve_turn,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Fixture {
    state: GameState,
    memories: Vec<(PlayerId, Option<OpponentMemory>)>,
}

fn snapshot(state: &GameState, me: PlayerId) -> Value {
    let cities: Vec<_> = state.cities_of(me).collect();
    let yields: i32 = cities
        .iter()
        .map(|c| {
            let y = legatum_core::economy::city_yields(state, c).total;
            y.food + y.production + y.gold + y.science
        })
        .sum();
    json!({"turn":state.turn,"finished":state.finished,"cities":cities.len(),
        "population":cities.iter().map(|c| c.population).sum::<i32>(),
        "yields":yields,"vp":team_vp(state,state.team_of(me))})
}

fn run(f: &Fixture, seed: u64) -> Value {
    let me = PlayerId(2);
    let target = legatum_core::CityId(1);
    let mut state = f.state.clone();
    state.setup.seed = seed;
    for p in &mut state.players {
        p.is_human = false;
    }
    for p in &mut state.setup.players {
        p.is_human = false;
        p.difficulty = AiDifficulty::Hard;
        p.search = true;
    }
    let mut brains: BTreeMap<PlayerId, Box<dyn Brain>> = state
        .players
        .iter()
        .map(|p| {
            let bseed = Rng::derive(seed, &[purpose::AI, u64::from(p.id.0)]).next_u64();
            let mut base =
                ScriptedBrain::with_difficulty(p.faction, bseed, p.id, AiDifficulty::Hard);
            if let Some(m) = f
                .memories
                .iter()
                .find(|(id, _)| *id == p.id)
                .and_then(|(_, m)| m.clone())
            {
                base.restore_opponent_memory(m);
            }
            (
                p.id,
                Box::new(SearchBrain::new(base, SearchConfig::in_app(4))) as Box<dyn Brain>,
            )
        })
        .collect();
    let mut declaration = None;
    let mut capture = None;
    let mut first_walls = None;
    let mut losses = 0;
    let mut at60 = Value::Null;
    let mut at100 = Value::Null;
    let mut at150 = Value::Null;
    let mut t14_search = Value::Null;
    while !state.finished && state.turn <= 150 {
        let turn = state.turn;
        if first_walls.is_none()
            && state.cities.get(&target).is_some_and(|c| {
                c.owner == PlayerId(0) && c.buildings.contains(&legatum_core::BuildingKind::Walls)
            })
        {
            first_walls = Some(turn);
        }
        let orders: BTreeMap<_, _> = brains
            .iter_mut()
            .filter(|(id, _)| state.player(**id).alive)
            .map(|(id, b)| (*id, b.orders(&player_view(&state, *id))))
            .collect();
        if turn == 14 {
            t14_search = json!(brains.get(&me).and_then(|b| b.search_events().last()));
        }
        if declaration.is_none()
            && orders.get(&me).is_some_and(|list| {
                list.iter().any(|o| {
                    matches!(
                        o,
                        Order::DeclareWar {
                            against: legatum_core::TeamId(0),
                            ..
                        }
                    )
                })
            })
        {
            declaration = Some(turn);
        }
        let (next, events) = resolve_turn(&state, &orders);
        for e in events {
            match e {
                Event::CityCaptured { city, to, .. } if city == target && to == me => {
                    capture = Some(turn)
                }
                Event::UnitDestroyed {
                    owner, unit_kind, ..
                } if owner == me && unit_kind != UnitKind::Scout => losses += 1,
                _ => {}
            }
        }
        state = next;
        match state.turn {
            61 => at60 = snapshot(&state, me),
            101 => at100 = snapshot(&state, me),
            151 => at150 = snapshot(&state, me),
            _ => {}
        }
    }
    if at60.is_null() && state.turn <= 60 {
        at60 = snapshot(&state, me);
    }
    if at100.is_null() && state.turn <= 100 {
        at100 = snapshot(&state, me);
    }
    if at150.is_null() {
        at150 = snapshot(&state, me);
    }
    json!({"seed":seed,"declaration":declaration,"capture":capture,"first_walls":first_walls,
        "attacked_before_walls":declaration.is_some_and(|at| first_walls.is_none_or(|w| at < w)),
        "losses":losses,"t14_search":t14_search,"t60":at60,"t100":at100,"t150":at150})
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let f: Fixture =
        serde_json::from_slice(&std::fs::read(&args[1]).expect("fixture")).expect("json");
    assert_eq!(f.state.turn, 14);
    let games: u64 = args.get(2).map_or(32, |s| s.parse().expect("games"));
    let seed: u64 = args
        .get(3)
        .map_or(874_654_782_136_188, |s| s.parse().expect("seed"));
    for i in 0..games {
        println!("{}", run(&f, seed + i));
    }
}

//! Idea 24 diagnostic: load the user's 8.9.4 state at T120 and let the AIs
//! play on with the given params; report the turn the Mongols (P2) declare war on the user's team
//! and what their units suffer. `lugano_probe STATE.json PARAMS.json [--seen-lugano TURN] [--turns N]`
use legatum_ai::{AiParams, Brain, OpponentMemory, ScriptedBrain};
use legatum_core::{AiDifficulty, CityId, GameState, Order, PlayerId, player_view, resolve_turn};
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let state: GameState =
        legatum_wasm::json::parse_state(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    let params: AiParams =
        serde_json::from_str(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let mut seen: Option<u32> = None;
    let mut turns = 8;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--seen-lugano" => {
                seen = Some(args[i + 1].parse().unwrap());
                i += 2;
            }
            "--turns" => {
                turns = args[i + 1].parse().unwrap();
                i += 2;
            }
            _ => panic!("arg"),
        }
    }
    let mongol = PlayerId(2);
    let lugano = CityId(14);
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .filter(|p| !p.is_human && p.alive)
        .map(|p| {
            (
                p.id,
                ScriptedBrain::with_params(p.faction, 7, p.id, AiDifficulty::Hard, &params),
            )
        })
        .collect();
    if let Some(t) = seen {
        let b = brains.get_mut(&mongol).unwrap();
        let mut m: OpponentMemory = b.opponent_memory().unwrap();
        m.campaigns.city_seen.insert(lugano, t);
        b.restore_opponent_memory(m);
    }
    let mut state = state;
    let user_team = state.player(PlayerId(0)).team;
    let units_at = |s: &GameState| {
        s.units
            .values()
            .filter(|u| u.owner == mongol && legatum_core::rules::is_military(u.kind))
            .count()
    };
    let start_units = units_at(&state);
    for _ in 0..turns {
        let mut orders: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
        for (id, b) in brains.iter_mut() {
            orders.insert(*id, b.orders(&player_view(&state, *id)));
        }
        if let Some(o) = orders.get(&mongol)
            && o.iter()
                .any(|x| matches!(x, Order::DeclareWar { against, .. } if *against == user_team))
        {
            println!("T{}: Mongols declare war on the user's team", state.turn);
        }
        state = resolve_turn(&state, &orders).0;
    }
    let at_war = state.player(mongol).team != user_team
        && legatum_core::player_view(&state, mongol).at_war(mongol, PlayerId(0));
    println!(
        "after {turns} turns: Mongol military units {start_units} -> {}, at war with the user: {at_war}",
        units_at(&state)
    );
}

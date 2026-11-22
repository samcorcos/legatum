//! Measure Hard expansion milestones, Settler captures and stalls.
//! hard_expansion <games> [first_seed=9100001] [small4|standard8] [baseline|chain|raid|candidate]
//! `baseline` disables both genes; `chain` and `raid` isolate their effects.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, Event, GameSetup, PlayerId, new_game, player_view, resolve_turn};
use legatum_wasm::session::{OrderMap, brain_seed};
use serde_json::json;

fn play(seed: u64, standard: bool, policy: &str) -> serde_json::Value {
    let mut setup = GameSetup::all_ai(seed);
    let players = if standard { 8 } else { 4 };
    setup.players.truncate(players);
    for (i, player) in setup.players.iter_mut().enumerate() {
        player.difficulty = AiDifficulty::Hard;
        player.team = legatum_core::TeamId(i as u8);
    }
    (setup.width, setup.height) = if standard { (48, 32) } else { (24, 16) };
    setup.turn_limit = None;
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
    let mut params = AiParams::builtin();
    params.name = policy.into();
    if policy == "baseline" || policy == "raid" {
        params.decisions.settler_chain = 0;
        params.decisions.settler_chain_priority = 0;
    }
    if policy == "baseline" || policy == "chain" {
        params.decisions.settler_raid_range = 0;
    }
    let params: &'static AiParams = Box::leak(Box::new(params));
    let mut state = new_game(&setup);
    let mut brains: Vec<(PlayerId, Box<dyn Brain>)> = (0..players)
        .map(|i| {
            let id = PlayerId(i as u8);
            (
                id,
                Box::new(ScriptedBrain::with_params(
                    state.player(id).faction,
                    brain_seed(setup.seed, id),
                    id,
                    AiDifficulty::Hard,
                    params,
                )) as Box<dyn Brain>,
            )
        })
        .collect();
    let (mut cities30, mut cities50) = (0, 0);
    let mut settler_captures = 0;
    let mut capital_queue = Vec::new();
    let mut settler_gates = Vec::new();
    while !state.finished && state.turn < cap {
        if seed == 9_100_001 {
            let view0 = player_view(&state, PlayerId(0));
            let diagnostic = ScriptedBrain::with_params(
                state.player(PlayerId(0)).faction,
                brain_seed(setup.seed, PlayerId(0)),
                PlayerId(0),
                AiDifficulty::Hard,
                params,
            );
            let (_, trace) = diagnostic.production_diagnosis(&view0);
            if let Some(capital) = state.cities_of(PlayerId(0)).find(|city| city.is_capital)
                && let Some(entry) = trace.iter().find(|entry| entry.city == capital.id)
                && state.turn <= 35
            {
                settler_gates.push(json!([state.turn, entry]));
            }
        }
        let orders: OrderMap = brains
            .iter_mut()
            .filter(|(id, _)| state.player(*id).alive)
            .map(|(id, brain)| {
                let id = *id;
                (id, brain.orders(&player_view(&state, id)))
            })
            .collect();
        if let Some(capital) = state.cities_of(PlayerId(0)).find(|city| city.is_capital) {
            for order in orders.get(&PlayerId(0)).into_iter().flatten() {
                if let legatum_core::Order::SetProduction { city, item } = order
                    && *city == capital.id
                    && state.turn <= 35
                {
                    capital_queue.push(json!([state.turn, item]));
                }
            }
        }
        let (next, events) = resolve_turn(&state, &orders);
        settler_captures += events
            .iter()
            .filter(|event| matches!(event, Event::SettlerCaptured { .. }))
            .count();
        if state.turn == 30 {
            cities30 = next.cities.len();
        }
        if state.turn == 50 {
            cities50 = next.cities.len();
        }
        state = next;
    }
    json!({
        "seed": seed,
        "players": players,
        "cities_t30": cities30,
        "cities_t50": cities50,
        "settler_captures": settler_captures,
        "capital_queue": capital_queue,
        "capital_production_trace": settler_gates,
        "stalled": !state.finished,
        "turns": state.turn,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: u64 = args.get(1).map_or(16, |x| x.parse().unwrap());
    let first_seed: u64 = args.get(2).map_or(9_100_001, |x| x.parse().unwrap());
    let standard = args.get(3).is_some_and(|x| x == "standard8");
    let policy = args.get(4).map_or("baseline", String::as_str);
    let rows: Vec<_> = (0..games)
        .map(|i| play(first_seed + i, standard, policy))
        .collect();
    let mean = |key: &str| {
        rows.iter()
            .map(|row| row[key].as_f64().unwrap())
            .sum::<f64>()
            / games as f64
    };
    let stalls = rows.iter().filter(|row| row["stalled"] == true).count();
    let captures: u64 = rows
        .iter()
        .map(|row| row["settler_captures"].as_u64().unwrap())
        .sum();
    println!(
        "{}",
        json!({
            "policy": policy,
            "map": if standard { "standard8" } else { "small4" },
            "games": games,
            "mean_cities_t30_per_game": mean("cities_t30"),
            "mean_cities_t50_per_game": mean("cities_t50"),
            "settler_captures_per_game": captures as f64 / games as f64,
            "stalls": stalls,
            "rows": rows,
        })
    );
}

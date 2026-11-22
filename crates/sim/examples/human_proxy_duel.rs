//! Four-seat playtest 8.5 counterfactual. Only Hard/Search brain seeds vary between salts;
//! game setup, map, starts, combat and barbarian RNG all keep the playtest seed.
use legatum_ai::{Brain, HumanMongolsBrain, HumanPlaybookV7Brain, HumanSpartaBrain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, BuildingKind, Event, Faction, GameSetup, GameState, PlayerId, ProductionItem,
    UnitKind, new_game, player_view, resolve_turn, scoring,
};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Write;

fn setup(which: &str) -> GameSetup {
    let raw = if which == "84" {
        include_str!("../data/human-proxy-8.4-setup.json")
    } else {
        include_str!("../data/human-proxy-8.5-setup.json")
    };
    legatum_wasm::json::setup_from_value(serde_json::from_str(raw).unwrap()).unwrap()
}

fn brain_seed(game_seed: u64, id: PlayerId, salt: u64) -> u64 {
    let seed = legatum_wasm::session::brain_seed(game_seed, id);
    if salt == 0 {
        seed
    } else {
        Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
    }
}

fn play(which: &str, mirror: bool, validate: bool, legacy_v7: bool, salt: u64) -> Value {
    let original = new_game(&setup("85"));
    let mut setup = setup(which);
    if mirror {
        setup.players[0].faction = Faction::Sparta;
        setup.players[2].faction = Faction::Mongols;
    }
    let mut state = new_game(&setup);
    if which == "85" {
        for id in [PlayerId(0), PlayerId(1), PlayerId(2), PlayerId(3)] {
            assert_eq!(
                state.player(id).is_human,
                original.player(id).is_human,
                "mirror changed the player control flag"
            );
        }
        assert_eq!(
            serde_json::to_value(&state.map).unwrap(),
            serde_json::to_value(&original.map).unwrap(),
            "mirror changed the map"
        );
        for id in [PlayerId(0), PlayerId(2)] {
            let start = |s: &GameState| s.capital_of(id).expect("starting capital").pos;
            assert_eq!(start(&state), start(&original), "mirror changed a start");
        }
    }
    let (glass, oath) = if which == "84" {
        (0, 0)
    } else if mirror {
        (2, 0)
    } else {
        (0, 2)
    };
    let mut brains: Vec<Box<dyn Brain>> = setup
        .players
        .iter()
        .enumerate()
        .map(|(seat, p)| {
            let id = PlayerId(seat as u8);
            if seat == glass && which == "85" {
                Box::new(HumanMongolsBrain::default()) as Box<dyn Brain>
            } else if seat == oath && (!validate || which == "84") {
                if legacy_v7 && which == "84" {
                    Box::new(HumanPlaybookV7Brain::default()) as Box<dyn Brain>
                } else {
                    Box::new(HumanSpartaBrain::default()) as Box<dyn Brain>
                }
            } else {
                let hard = ScriptedBrain::with_difficulty(
                    p.faction,
                    brain_seed(setup.seed, id, salt),
                    id,
                    AiDifficulty::Hard,
                );
                if p.search {
                    Box::new(legatum_ai::search::SearchBrain::new(
                        hard,
                        legatum_ai::search::SearchConfig::in_app(setup.players.len()),
                    )) as Box<dyn Brain>
                } else {
                    Box::new(hard)
                }
            }
        })
        .collect();
    let mut war = [None; 4];
    let mut walls = [None; 4];
    let mut hall = [None; 4];
    let mut captures: [Vec<u32>; 4] = std::array::from_fn(|_| Vec::new());
    let mut war_events = Vec::new();
    let mut capture_events = Vec::new();
    let mut war_details = Vec::new();
    let mut capture_details = Vec::new();
    let mut settler_captures = Vec::new();
    let mut foundings: [Vec<u32>; 4] = std::array::from_fn(|_| Vec::new());
    let mut founding_sites = Vec::new();
    let mut settlers: [Vec<u32>; 4] = std::array::from_fn(|_| Vec::new());
    let mut snapshots = BTreeMap::new();
    let mut scout_trace = Vec::new();
    let mut decision_trace = Vec::new();
    let mut opening_opponent_orders = None;
    let mut win_reason = None;
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 2;
    while !state.finished && state.turn <= cap {
        let turn = state.turn;
        if validate && which == "85" && turn <= 35 {
            let view = player_view(&state, PlayerId(0));
            scout_trace.push(json!({
                "turn": turn,
                "scouts": state.units_of(PlayerId(0)).filter(|u| u.kind == UnitKind::Scout).map(|u| u.pos).collect::<Vec<_>>(),
                "seen_oath_city": view.cities.iter().filter(|c| c.owner == PlayerId(2)).map(|c| c.pos).collect::<Vec<_>>(),
                "seen_oath_settler": view.units.iter().filter(|u| u.owner == PlayerId(2) && u.kind == UnitKind::Settler).map(|u| u.pos).collect::<Vec<_>>(),
            }));
        }
        let orders: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let orders = brains[p.id.0 as usize].orders(&player_view(&state, p.id));
                (p.id, orders)
            })
            .collect();
        if validate && turn <= 150 {
            let seat = if which == "84" {
                PlayerId(0)
            } else {
                PlayerId(glass as u8)
            };
            let view = player_view(&state, seat);
            decision_trace.push(json!({
                "turn": turn,
                "gold": view.economy.gold,
                "capital_wall_quote": view.cities.iter().find(|c| c.owner == seat && c.is_capital)
                    .and_then(|c| c.detail.as_ref())
                    .and_then(|d| d.options.iter().find(|o| o.item == ProductionItem::Building {
                        building: BuildingKind::Walls,
                    }))
                    .map(|o| json!({"cost": o.buy_cost, "block": o.buy_block})),
                "orders": orders.get(&seat),
                "cities": view.cities.iter().filter(|c| c.owner == seat).map(|c| json!({
                    "id": c.id, "at": c.pos, "production": c.detail.as_ref().and_then(|d| d.producing),
                    "walls": c.walls,
                })).collect::<Vec<_>>(),
                "units": view.units.iter().filter(|u| u.owner == seat).map(|u| json!({
                    "id": u.id, "kind": u.kind, "at": u.pos, "hp": u.hp, "xp": u.xp,
                })).collect::<Vec<_>>(),
                "known_cities": view.cities.iter().filter(|c| c.owner != seat).map(|c| json!({
                    "owner": c.owner, "at": c.pos, "walls": c.walls, "hp": c.hp,
                })).collect::<Vec<_>>(),
                "seen_settlers": view.units.iter().filter(|u| u.owner != seat && u.kind == UnitKind::Settler).map(|u| json!({
                    "owner": u.owner, "at": u.pos,
                })).collect::<Vec<_>>(),
                "visible_military": view.units.iter().filter(|u| u.owner != seat && u.military).map(|u| json!({
                    "owner": u.owner, "at": u.pos, "kind": u.kind, "hp": u.hp,
                })).collect::<Vec<_>>(),
            }));
        }
        if validate && turn == 1 {
            let opponents: BTreeMap<_, _> = orders
                .iter()
                .filter(|(id, _)| **id != PlayerId(0))
                .map(|(id, orders)| (id.0.to_string(), orders.clone()))
                .collect();
            opening_opponent_orders = Some(serde_json::to_value(opponents).unwrap());
        }
        let walls_before: BTreeMap<_, _> = state
            .cities
            .iter()
            .map(|(id, city)| (*id, city.buildings.contains(&BuildingKind::Walls)))
            .collect();
        let glass_cities_before = state.cities_of(PlayerId(glass as u8)).count();
        let glass_walled_before = state
            .cities_of(PlayerId(glass as u8))
            .filter(|city| city.buildings.contains(&BuildingKind::Walls))
            .count();
        let (next, events) = resolve_turn(&state, &orders);
        state = next;
        for event in events {
            match event {
                Event::WarDeclared { relation } => {
                    for team in relation.declarers {
                        let target = if team == relation.a {
                            relation.b
                        } else {
                            relation.a
                        };
                        war_events.push((turn, team.0, target.0));
                        if team.0 as usize == oath && target.0 as usize == glass {
                            war_details.push(json!({
                                "turn": turn,
                                "glass_cities": glass_cities_before,
                                "glass_walled_cities": glass_walled_before,
                            }));
                        }
                        if let Some(w) = war.get_mut(team.0 as usize) {
                            w.get_or_insert(turn);
                        }
                    }
                }
                Event::ProductionCompleted {
                    owner,
                    item: ProductionItem::Building { building },
                    ..
                }
                | Event::ItemBought {
                    owner,
                    item: ProductionItem::Building { building },
                    ..
                } => {
                    if building == BuildingKind::Walls {
                        walls[owner.0 as usize].get_or_insert(turn);
                    }
                    if building == BuildingKind::YamRelayStation {
                        hall[owner.0 as usize].get_or_insert(turn);
                    }
                }
                Event::CityCaptured { city, from, to, .. } => {
                    captures[to.0 as usize].push(turn);
                    capture_events.push((turn, from.0, to.0));
                    capture_details.push(json!({
                        "turn": turn, "city": city.0, "from": from.0, "to": to.0,
                        "walled_before": walls_before.get(&city).copied().unwrap_or(false),
                        "walled_after": state.cities.get(&city).is_some_and(|c| c.buildings.contains(&BuildingKind::Walls)),
                    }));
                }
                Event::SettlerCaptured { from, to, .. } => {
                    settler_captures.push((turn, from.0, to.0));
                }
                Event::CityFounded { owner, at, .. } => {
                    foundings[owner.0 as usize].push(turn);
                    founding_sites.push(json!({"turn": turn, "owner": owner.0, "at": at}));
                }
                Event::ProductionCompleted {
                    owner,
                    item:
                        ProductionItem::Unit {
                            unit: UnitKind::Settler,
                        },
                    ..
                } => settlers[owner.0 as usize].push(turn),
                Event::GameWon { reason, .. } => win_reason = Some(reason),
                _ => {}
            }
        }
        if [30, 50, 60, 62, 95, 100, 114, 123, 150].contains(&turn) {
            snapshots.insert(
                turn,
                [glass, oath].map(|seat| {
                    let id = PlayerId(seat as u8);
                    json!({
                        "cities": state.cities_of(id).count(),
                        "army": state.units_of(id).filter(|u| u.kind != UnitKind::Scout && u.kind != UnitKind::Settler).count(),
                        "vp": scoring::vp_total(&state, id),
                        "vp_sources": scoring::vp(&state, id),
                        "population": state.cities_of(id).map(|c| c.population).sum::<i32>(),
                        "veterans_level3": state.units_of(id).filter(|u| u.kind != UnitKind::Scout && u.kind != UnitKind::Settler && legatum_core::rules::promotion::level_for(state.player(id).faction, u.xp) >= 3).count(),
                    })
                }),
            );
        }
    }
    json!({
        "salt": salt, "mirror": mirror, "validate": validate, "legacy_v7": legacy_v7, "source": which,
        "opponent_config": "in_app_session",
        "turn": state.turn, "winner": state.winner.map(|t| t.0),
        "reason": win_reason, "glass_seat": glass, "oath_seat": oath,
        "war": war, "walls": walls, "hall": hall, "captures": captures,
        "war_events": war_events, "capture_events": capture_events,
        "war_details": war_details, "capture_details": capture_details,
        "settler_captures": settler_captures,
        "foundings": foundings, "settlers": settlers,
        "founding_sites": founding_sites,
        "scout_trace": scout_trace,
        "decision_trace": decision_trace,
        "opening_opponent_orders": opening_opponent_orders,
        "snapshots": snapshots,
        "glass_cities": state.cities_of(PlayerId(glass as u8)).count(),
        "oath_cities": state.cities_of(PlayerId(oath as u8)).count(),
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let value = |flag: &str| -> Option<&str> {
        args.windows(2)
            .find(|pair| pair[0] == flag)
            .map(|pair| pair[1].as_str())
    };
    let which = value("--source").unwrap_or("85");
    let mirror = value("--orientation") == Some("mirror");
    let validate = args.iter().any(|arg| arg == "--validate");
    let legacy_v7 = args.iter().any(|arg| arg == "--legacy-v7");
    let games: u64 = value("--games").unwrap_or("1").parse().unwrap();
    let first: u64 = value("--seed").unwrap_or("0").parse().unwrap();
    let out_dir = value("--out-dir");
    let rows: Vec<_> = (first..first + games)
        .into_par_iter()
        .map(|salt| play(which, mirror, validate, legacy_v7, salt))
        .collect();
    if let Some(dir) = out_dir {
        std::fs::create_dir_all(dir).unwrap();
        let path = format!(
            "{dir}/human_proxy_{}_{}.jsonl",
            which,
            if mirror { "mirror" } else { "original" }
        );
        let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
        for row in rows {
            writeln!(file, "{row}").unwrap();
        }
    } else {
        for row in rows {
            println!("{row}");
        }
    }
}

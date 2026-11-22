//! Hash-checked, per-turn Germany production and supply facts from recorded smoke tapes.
//! germany_foundry_audit TAPE_DIR OUT.jsonl

use legatum_core::resources::{Resource, supply_views};
use legatum_core::view::player_view;
use legatum_core::{BuildingKind, Faction, Order, ProductionItem, TechId, data, resolve_turn};
use legatum_wasm::tape::Tape;
use serde_json::json;
use std::io::Write;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "germany_foundry_audit TAPE_DIR OUT.jsonl");
    let mut files: Vec<_> = std::fs::read_dir(&args[1])
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    for file in files {
        let tape = Tape::from_json(&std::fs::read_to_string(&file).unwrap()).unwrap();
        let mut state = tape.initial_state();
        assert_eq!(state.hash(), tape.initial_hash);
        for turn in &tape.turns {
            assert_eq!(state.turn, turn.turn);
            for player in state
                .players
                .iter()
                .filter(|p| p.faction == Faction::Germany)
            {
                let view = player_view(&state, player.id);
                let supplies = supply_views(&state, player.id);
                let supply = |resource| {
                    let s = supplies.iter().find(|s| s.resource == resource).unwrap();
                    json!({"capacity": s.supplied.total, "used": s.used.total,
                        "held": s.held.len(), "spare": s.spare, "deposits": s.deposits.len()})
                };
                let orders = turn.orders.get(&player.id);
                let city_rows: Vec<_> = state
                    .cities_of(player.id)
                    .map(|city| {
                        let detail = view
                            .cities
                            .iter()
                            .find(|c| c.id == city.id)
                            .and_then(|c| c.detail.as_ref())
                            .unwrap();
                        let option =
                            |building| {
                                detail.options.iter().find(|o| o.item ==
                        ProductionItem::Building { building }).map(|o| json!({
                            "turns": o.turns, "resource_blocker": o.resource_blocker,
                            "buy_block": o.buy_block, "buy_cost": o.buy_cost,
                        }))
                            };
                        let city_orders: Vec<_> = orders
                            .into_iter()
                            .flatten()
                            .filter(|o| {
                                matches!(o,
                        Order::SetProduction { city: id, .. } | Order::BuyItem { city: id, .. }
                        if *id == city.id)
                            })
                            .collect();
                        json!({
                            "id": city.id.0, "pop": city.population,
                            "founded": city.founded_turn, "resistance": city.resistance,
                            "workshop": data::has_building(&city.buildings, BuildingKind::Workshop),
                            "forge": data::has_building(&city.buildings, BuildingKind::MachineShop),
                            "foundry": data::has_building(&city.buildings, BuildingKind::Autoworks),
                            "producing": city.producing, "queue": city.production_queue,
                            "orders": city_orders,
                            "workshop_option": option(BuildingKind::Workshop),
                            "forge_option": option(BuildingKind::MachineShop),
                            "foundry_option": option(BuildingKind::Autoworks),
                        })
                    })
                    .collect();
                writeln!(out, "{}", json!({
                    "seed": tape.setup.seed, "turn": turn.turn, "seat": player.id.0,
                    "alive": player.alive, "industrialization": player.techs.contains(&TechId::Industrialization),
                    "gold": player.gold, "goal": player.faction_goal_done,
                    "iron": supply(Resource::Iron), "coal": supply(Resource::Coal),
                    "cities": city_rows,
                })).unwrap();
            }
            state = resolve_turn(&state, &turn.orders).0;
            assert_eq!(
                state.hash().to_string(),
                turn.hash,
                "replay drift: {} T{}",
                file.display(),
                turn.turn
            );
        }
    }
    out.flush().unwrap();
}

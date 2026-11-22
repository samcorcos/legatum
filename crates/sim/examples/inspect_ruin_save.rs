//! Inspect AI orders around a saved ruin without modifying the `.legatum` file.
use legatum_core::player_view;
use legatum_wasm::session::{BrainKind, Session};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("save path");
    let turns: u32 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    let wrapper: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let save = wrapper["game"].as_str().expect("game string");
    let mut session = Session::from_save_json(save, |_| BrainKind::Scripted)?;
    let india = session
        .state
        .players
        .iter()
        .find(|p| p.faction == legatum_core::Faction::India)
        .map(|p| p.id)
        .expect("India");
    for _ in 0..turns {
        let view = player_view(&session.state, india);
        let ruins: Vec<_> = session
            .state
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.ruin && t.owner == Some(india))
            .map(|(i, _)| session.state.map.hex_at(i))
            .collect();
        println!("turn {} India {:?} own ruins {ruins:?}", view.turn, india);
        for u in view.units.iter().filter(|u| u.owner == india) {
            println!(
                "  unit {:?} {:?} {:?} hp {} distance {:?}",
                u.id,
                u.kind,
                u.pos,
                u.hp,
                ruins.iter().map(|h| h.distance(u.pos)).collect::<Vec<_>>()
            );
        }
        let orders = session.ai_orders();
        println!("  orders {:#?}", orders.get(&india));
        session.step(&orders);
    }
    Ok(())
}

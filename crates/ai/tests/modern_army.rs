// Views and warmed decision memory from the recorded turn 141/150 replay on engine a0f44d9.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, CityId, Faction, Order, PlayerView, ProductionItem, UnitKind};

fn view(turn: u32) -> PlayerView {
    let bytes: &[u8] = match turn {
        141 => include_bytes!("fixtures/modern-army/t141-p5.json.gz"),
        150 => include_bytes!("fixtures/modern-army/t150-p5.json.gz"),
        _ => panic!("unsupported turn"),
    };
    serde_json::from_reader(flate2::read::GzDecoder::new(bytes)).unwrap()
}

fn orders(view: &PlayerView, turn: u32, gene: i32) -> Vec<Order> {
    let mut params = AiParams::builtin();
    params.decisions.modern_army = gene;
    let mut brain = ScriptedBrain::with_params(
        Faction::Switzerland,
        1,
        view.me,
        AiDifficulty::Hard,
        &params,
    );
    let bytes: &[u8] = match (turn, gene) {
        (141, 0) => include_bytes!("fixtures/modern-army/t141-p5-base.memory.json.gz"),
        (141, 1) => include_bytes!("fixtures/modern-army/t141-p5-new.memory.json.gz"),
        (150, 0) => include_bytes!("fixtures/modern-army/t150-p5-base.memory.json.gz"),
        (150, 1) => include_bytes!("fixtures/modern-army/t150-p5-new.memory.json.gz"),
        _ => panic!("unsupported turn or gene"),
    };
    let saved: serde_json::Value =
        serde_json::from_reader(flate2::read::GzDecoder::new(bytes)).unwrap();
    brain.strategy = serde_json::from_value(saved["strategy"].clone()).unwrap();
    brain.restore_opponent_memory(serde_json::from_value(saved["memory"].clone()).unwrap());
    brain.orders(view)
}

fn spearman_cities(orders: &[Order]) -> Vec<CityId> {
    let mut cities: Vec<_> = orders
        .iter()
        .filter_map(|order| match order {
            Order::SetProduction {
                city,
                item:
                    ProductionItem::Unit {
                        unit: UnitKind::Spearman,
                    },
            } => Some(*city),
            _ => None,
        })
        .collect();
    cities.sort();
    cities
}

#[test]
fn swiss_t141_stops_ordering_spearmen_with_modern_army() {
    let view = view(141);
    let legacy = orders(&view, 141, 0);
    let modern = orders(&view, 141, 1);
    assert_eq!(
        spearman_cities(&legacy),
        [6, 10, 14, 21, 26, 30, 37].map(CityId).to_vec(),
        "legacy orders: {legacy:?}"
    );
    assert!(
        spearman_cities(&modern).is_empty(),
        "modern orders: {modern:?}"
    );
    assert!(modern.iter().any(|order| matches!(
        order,
        Order::SetProduction {
            item: ProductionItem::Unit {
                unit: UnitKind::Musketman | UnitKind::Rifleman
            },
            ..
        }
    )));
}

#[test]
fn swiss_t150_refits_obsolete_spearmen_from_reserved_gold() {
    let view = view(150);
    let legacy = orders(&view, 150, 0);
    let modern = orders(&view, 150, 1);
    assert!(
        !legacy
            .iter()
            .any(|order| matches!(order, Order::UpgradeUnit { .. }))
    );
    assert!(modern.iter().any(|order| {
        match order {
            Order::UpgradeUnit { unit } => view
                .units
                .iter()
                .any(|u| u.id == *unit && u.kind == UnitKind::Spearman),
            _ => false,
        }
    }));
    assert_eq!(spearman_cities(&legacy), [6, 26, 30].map(CityId).to_vec());
    assert!(spearman_cities(&modern).is_empty());
}

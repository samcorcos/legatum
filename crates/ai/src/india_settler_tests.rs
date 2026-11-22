//! The 8.9.25 India start (coach review 2026-11-03): Germany's and the Celts' first Settlers.
use crate::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, Hex, Order, PlayerView, ProductionItem, UnitId, UnitKind};

fn load<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    serde_json::from_reader(flate2::read::GzDecoder::new(bytes)).unwrap()
}

macro_rules! fixture {
    ($name:literal) => {
        (
            include_bytes!(concat!("fixtures/india-1103/", $name, ".view.json.gz")).as_slice(),
            include_bytes!(concat!("fixtures/india-1103/", $name, ".memory.json.gz")).as_slice(),
            include_bytes!(concat!("fixtures/india-1103/", $name, ".recorded.json.gz")).as_slice(),
        )
    };
}

type Fixture = (&'static [u8], &'static [u8], &'static [u8]);

/// The seat's recorded view and its Hard memory warmed over the earlier turns (the brain
/// shadowed the tape with the app's seeds), plus the orders it gave. The brain plays
/// `snipe_read` 90 (the default) and `capital_guard` 1 (off by default since the 2026-11-03
/// benchmark, `docs/sim-results/ai-quality/settler-safety-1103/`).
fn seat(f: Fixture) -> (PlayerView, ScriptedBrain, serde_json::Value) {
    let view: PlayerView = load(f.0);
    let recorded: serde_json::Value = load(f.2);
    let seed = recorded["seed"].as_u64().unwrap();
    let mut brain =
        ScriptedBrain::with_difficulty(view.faction_of(view.me), seed, view.me, AiDifficulty::Hard);
    let memory: Option<crate::OpponentMemory> = load(f.1);
    brain.restore_opponent_memory(memory.unwrap());
    brain.decisions.snipe_read = 90;
    brain.decisions.capital_guard = 1;
    // Recorded before the capitals victory: the escort tests read `capital_guard` alone.
    brain.decisions.capitals = 0;
    (view, brain, recorded)
}

fn germany(f: Fixture) -> (PlayerView, ScriptedBrain) {
    let (view, brain, _) = seat(f);
    (view, brain)
}

fn genes_off(brain: &mut ScriptedBrain) {
    brain.decisions.snipe_read = 0;
    brain.decisions.capital_guard = 0;
    brain.decisions.escort_buy = 0;
    brain.decisions.peace_army_sites = 0;
    brain.decisions.supported_defence = 0;
    // The fixtures were recorded before Walls counted as the garrison (shared since 2026-11-06).
    brain.decisions.walls_garrison = 0;
}

fn unit_order(orders: &[Order], unit: UnitId) -> Option<&Order> {
    orders
        .iter()
        .find(|o| legatum_core::rules::units::order_unit(o) == Some(unit))
}

const LEIPZIG: Hex = Hex { q: 14, r: 23 };
const GERMAN_SETTLER: UnitId = UnitId(21);
const GERMAN_SOLDIER: UnitId = UnitId(18);

/// Gene off, the fixture reproduces the recorded trip: T12 east with the garrison, T13 and
/// T14 south-west to (11,27) with the Soldier beside the Settler.
#[test]
fn the_fixtures_reproduce_the_recorded_trip_with_the_genes_off() {
    for f in [
        fixture!("T12-p5"),
        fixture!("T13-p5"),
        fixture!("T14-p5"),
        fixture!("T17-p4"),
    ] {
        let (view, mut brain, recorded) = seat(f);
        genes_off(&mut brain);
        let orders = brain.orders(&view);
        assert_eq!(
            serde_json::to_value(&orders).unwrap(),
            recorded["shadow"],
            "T{}",
            view.turn
        );
    }
}

fn path_of(order: Option<&Order>) -> Vec<Hex> {
    match order {
        Some(Order::MoveUnitPath { path, .. }) => path.clone(),
        Some(Order::MoveUnit { to, .. }) => vec![*to],
        _ => Vec::new(),
    }
}

fn destination(order: Option<&Order>) -> Option<Hex> {
    match order {
        Some(Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }) => Some(*to),
        _ => None,
    }
}

/// T12-T14: Germany's Settler leaves with its only Soldier beside it, and at T13 and T14 a
/// single troop of a neighbour without a surprise war (a Spartan, then an Indian Soldier)
/// does not hold the garrison home (`capital_guard` needs two troops, a war or a recent
/// surprise war: in the Lab position a Spartan Soldier parked by Leipzig for twenty turns).
/// The genes keep the recorded escort: the Soldier walks onto the Settler's path.
#[test]
fn germany_t12_to_t14_escorts_its_settler() {
    for f in [fixture!("T12-p5"), fixture!("T13-p5"), fixture!("T14-p5")] {
        let (view, mut brain) = germany(f);
        let orders = brain.orders(&view);
        let path = path_of(unit_order(&orders, GERMAN_SETTLER));
        let soldier = destination(unit_order(&orders, GERMAN_SOLDIER));
        assert!(
            soldier.is_some_and(|to| path.contains(&to)),
            "T{}: {orders:?}",
            view.turn
        );
    }
}

/// T15 (two Indian Soldiers in sight, one beside the Settler): the Settler turns home and
/// its escort, which stood still in the recorded game (its tile was unsafe, so the escort
/// pass skipped it), now walks back beside it (`snipe_read`).
#[test]
fn germany_t15_escort_walks_home_with_the_settler() {
    let (view, mut brain) = germany(fixture!("T15-p5"));
    let orders = brain.orders(&view);
    assert_eq!(
        destination(unit_order(&orders, GERMAN_SETTLER)),
        Some(LEIPZIG),
        "{orders:?}"
    );
    assert_eq!(
        unit_order(&orders, GERMAN_SOLDIER),
        Some(&Order::MoveUnit {
            unit: GERMAN_SOLDIER,
            to: Hex { q: 12, r: 25 }
        }),
        "{orders:?}"
    );
    let (view, mut brain) = germany(fixture!("T15-p5"));
    genes_off(&mut brain);
    let orders = brain.orders(&view);
    assert_eq!(
        unit_order(&orders, GERMAN_SOLDIER),
        Some(&Order::SkipUnit {
            unit: GERMAN_SOLDIER
        }),
        "{orders:?}"
    );
}

/// `capital_guard`: T14 with a second Indian Soldier two hexes from the empty capital. Two
/// troops of one rival in reach of Leipzig call the Settler home, and its Soldier with it.
#[test]
fn germany_t14_goes_home_when_an_army_reaches_the_empty_capital() {
    let (mut view, mut brain) = germany(fixture!("T14-p5"));
    let mut second = view
        .units
        .iter()
        .find(|u| u.owner == legatum_core::PlayerId(0) && u.kind == UnitKind::Soldier)
        .unwrap()
        .clone();
    second.id = UnitId(900);
    second.pos = Hex { q: 12, r: 24 };
    view.units.push(second);
    let soldier = view
        .units
        .iter()
        .find(|u| u.id == GERMAN_SOLDIER)
        .unwrap()
        .pos;
    let orders = brain.orders(&view);
    assert_eq!(
        destination(unit_order(&orders, GERMAN_SETTLER)),
        Some(LEIPZIG),
        "{orders:?}"
    );
    let to = destination(unit_order(&orders, GERMAN_SOLDIER)).expect("the Soldier moves");
    assert!(
        to.distance(LEIPZIG) < soldier.distance(LEIPZIG),
        "{orders:?}"
    );
    let (_, mut off) = germany(fixture!("T14-p5"));
    off.decisions.capital_guard = 0;
    let orders = off.orders(&view);
    assert_ne!(
        destination(unit_order(&orders, GERMAN_SETTLER)),
        Some(LEIPZIG),
        "{orders:?}"
    );
}

const CELTIC_SETTLER: UnitId = UnitId(27);
const CELTIC_SOLDIER: UnitId = UnitId(15);

/// T17: India declared on Germany at T16. The Celts have not met India and see none of its
/// units; their Settler walked on alone to (18,9) and India's Scout took it at T18. With
/// `snipe_read` the trip past unseen tiles takes Lugdunum's quiet garrison as escort, and the
/// Settler waits for it this turn instead of walking ahead of it.
#[test]
fn celts_t17_settler_waits_for_its_escort() {
    let (view, mut brain, _) = seat(fixture!("T17-p4"));
    let settler = view
        .units
        .iter()
        .find(|u| u.id == CELTIC_SETTLER)
        .unwrap()
        .pos;
    let orders = brain.orders(&view);
    assert_eq!(
        unit_order(&orders, CELTIC_SETTLER),
        Some(&Order::SkipUnit {
            unit: CELTIC_SETTLER
        }),
        "{orders:?}"
    );
    assert_eq!(
        unit_order(&orders, CELTIC_SOLDIER),
        Some(&Order::MoveUnit {
            unit: CELTIC_SOLDIER,
            to: settler
        }),
        "{orders:?}"
    );
}

/// India's war on Germany marks India as a sniper for the Celts (`snipe_read`); before it,
/// and for the genes off, the flat prior stands.
#[test]
fn a_recent_surprise_war_raises_the_capture_chance() {
    let (view, brain, _) = seat(fixture!("T17-p4"));
    let p = brain.planner(&view);
    assert!(p.recent_surprise_war(legatum_core::PlayerId(0)));
    assert_eq!(
        p.peace_settler_capture_chance(legatum_core::PlayerId(0)),
        90
    );
    assert_eq!(
        p.peace_settler_capture_chance(legatum_core::PlayerId(1)),
        10
    );
    let (view, brain, _) = seat(fixture!("T16-p4"));
    let p = brain.planner(&view);
    assert_eq!(
        p.peace_settler_capture_chance(legatum_core::PlayerId(0)),
        10
    );
}

/// `escort_buy`: at T13 Germany has one Soldier, its Settler beside Leipzig and a Spartan
/// Soldier two hexes off. With 300 Gold the fight-aware scorer buys a second Soldier
/// against the Spearman's anti-mounted bonus; gene 0 keeps the old Horseman order.
/// With its real 70 Gold, it buys nothing.
#[test]
fn germany_t13_buys_a_second_troop_when_it_has_the_gold() {
    let buys = |orders: &[Order]| {
        orders
            .iter()
            .filter_map(|o| match o {
                Order::BuyItem { item, .. } => Some(*item),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let escort = ProductionItem::Unit {
        unit: UnitKind::Horseman,
    };
    let guard = ProductionItem::Unit {
        unit: UnitKind::Soldier,
    };
    let rich = |view: &mut PlayerView| {
        view.economy.gold = 300;
        for c in view.cities.iter_mut().filter(|c| c.owner == view.me) {
            for o in &mut c.detail.as_mut().unwrap().options {
                if o.buy_cost.is_some_and(|q| q <= 300) {
                    o.buy_block = None;
                }
            }
        }
    };
    let (mut view, mut brain) = germany(fixture!("T13-p5"));
    rich(&mut view);
    assert_eq!(buys(&brain.orders(&view)), vec![guard]);
    let (mut view, mut brain) = germany(fixture!("T13-p5"));
    rich(&mut view);
    brain.decisions.hard_purchase_scorer = 0;
    brain.decisions.named_fund = 1;
    assert_eq!(buys(&brain.orders(&view)), vec![escort]);
    let (mut view, mut brain) = germany(fixture!("T13-p5"));
    rich(&mut view);
    brain.decisions.named_fund = 1;
    brain.decisions.escort_buy = 0;
    assert!(!buys(&brain.orders(&view)).contains(&escort));
    let (view, mut brain) = germany(fixture!("T13-p5"));
    assert!(buys(&brain.orders(&view)).is_empty());
}

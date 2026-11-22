//! Gene `door_defense` (`door.rs`): defence sized to the visible army at the door.
use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid};
use crate::scripted::Planner;
use legatum_core::{
    AiDifficulty, CityId, Faction, Hex, Order, PlayerView, ProductionItem, TechId, UnitId, UnitKind,
};

const CITY: Hex = Hex { q: 6, r: 6 };

/// Germany's city at (6,6) with Gold, every unit option buyable at twice its Production.
fn view(gold: i32, enemies: &[(u32, UnitKind, Hex, i32)]) -> PlayerView {
    let mut b = ViewBuilder::new(24, 14)
        .faction(Faction::Germany)
        .known_techs(&[TechId::Mathematics])
        .gold(gold)
        .city(1, ME, CITY, 4)
        .city(2, ENEMY, Hex::new(20, 6), 3);
    for &(id, kind, pos, hp) in enemies {
        b = b.unit_hp(id, ENEMY, kind, pos, hp);
    }
    let mut v = b.build();
    v.economy.gold_per_turn.total = 10;
    for c in v.cities.iter_mut().filter(|c| c.owner == ME) {
        let d = c.detail.as_mut().unwrap();
        for o in &mut d.options {
            if matches!(o.item, ProductionItem::Unit { .. }) {
                o.buy_cost = Some(o.cost * 2);
            }
        }
    }
    v
}

fn planner(v: &PlayerView, door: i32) -> Planner<'_> {
    let mut p = crate::fixture::planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.door_defense = door;
    p
}

fn army(n: u32, at: Hex) -> Vec<(u32, UnitKind, Hex, i32)> {
    (0..n)
        .map(|i| {
            (
                50 + i,
                UnitKind::Phalangite,
                Hex::new(at.q + i as i32 % 2, at.r + i as i32 / 2),
                100,
            )
        })
        .collect()
}

fn buys(orders: &[Order]) -> Vec<ProductionItem> {
    orders
        .iter()
        .filter_map(|o| match o {
            Order::BuyItem {
                city: CityId(1),
                item,
            } => Some(*item),
            _ => None,
        })
        .collect()
}

#[test]
fn a_breached_city_swaps_its_shooter_for_a_surviving_line_garrison() {
    let mut v = ViewBuilder::new(24, 14)
        .faction(Faction::Switzerland)
        .city(1, ME, CITY, 6)
        .unit(1, ME, UnitKind::FieldCannon, CITY)
        .unit(2, ME, UnitKind::Rifleman, Hex::new(5, 6))
        .unit(50, ENEMY, UnitKind::Cavalry, Hex::new(6, 7))
        .unit(51, ENEMY, UnitKind::Cavalry, Hex::new(7, 6))
        .build();
    v.cities[0].hp = 51;
    let mut p = planner(&v, 100);
    // This older fixture isolates immediate garrison rotation.
    p.decisions.siege_survival_clock = 0;
    let army = p.ctx.own_units.clone();
    p.door_military(&army);
    assert!(
        p.orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit: UnitId(1), to } if *to == Hex::new(5, 6)))
            && p.orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { unit: UnitId(2), to } if *to == CITY)),
        "{:?}",
        p.orders
    );
    // One Cavalry can zero the shield but cannot capture through it this turn. The
    // stronger line garrison still needs to take over before the next attack.
    v.units.retain(|u| u.id != UnitId(51));
    let mut p = planner(&v, 100);
    p.decisions.siege_survival_clock = 0;
    let army = p.ctx.own_units.clone();
    p.door_military(&army);
    assert!(
        p.orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit: UnitId(2), to } if *to == CITY)),
        "{:?}",
        p.orders
    );
}

#[test]
fn cannon_in_a_nearly_breached_city_values_adjacent_cavalry() {
    let mut v = ViewBuilder::new(24, 14)
        .faction(Faction::Switzerland)
        .city(1, ME, CITY, 6)
        .unit(1, ME, UnitKind::FieldCannon, CITY)
        .unit_hp(50, ENEMY, UnitKind::Cavalry, Hex::new(6, 7), 66)
        .unit_hp(51, ENEMY, UnitKind::Cavalry, Hex::new(7, 6), 93)
        .unit(52, ENEMY, UnitKind::Soldier, Hex::new(6, 8))
        .build();
    v.cities[0].hp = 51;
    let p = planner(&v, 100);
    let cavalry = p.ctx.enemy_units[0];
    let soldier = p.ctx.enemy_units[2];
    assert!(
        p.enemy_threat(cavalry, cavalry.hp) > p.enemy_threat(soldier, soldier.hp),
        "adjacent city attackers should be worth more than distant infantry"
    );
}

#[test]
fn door_gene_off_sees_no_front_and_changes_nothing() {
    let v = view(800, &army(3, Hex::new(9, 6)));
    let mut p = planner(&v, 0);
    assert!(p.door_fronts().is_empty());
    p.plan_door_defense();
    assert!(p.orders.is_empty(), "{:?}", p.orders);
    assert_eq!(p.gold_left, 800);
}

#[test]
fn an_army_within_its_march_is_a_front_and_a_far_one_is_not() {
    let near = view(0, &army(3, Hex::new(9, 6)));
    let p = planner(&near, 100);
    let fronts = p.door_fronts();
    assert_eq!(fronts.len(), 1);
    assert_eq!(fronts[0].city, CityId(1));
    assert!(fronts[0].gap() > 0, "{fronts:?}");
    assert!(fronts[0].arrival <= 1);
    // Twelve hexes away, a two-move army needs more than three turns: no front, no spending.
    let far = view(800, &army(3, Hex::new(18, 6)));
    let mut p = planner(&far, 100);
    assert!(p.door_fronts().is_empty());
    p.plan_door_defense();
    assert!(p.orders.is_empty(), "{:?}", p.orders);
}

#[test]
fn threat_counts_strength_squared_so_an_era_lead_outweighs_numbers() {
    // One Phalangite (25) outweighs one Spearman (20) by more than its linear 25%.
    let one_new = view(0, &[(50, UnitKind::Phalangite, Hex::new(9, 6), 100)]);
    let one_old = view(0, &[(50, UnitKind::Soldier, Hex::new(9, 6), 100)]);
    let new = planner(&one_new, 100).door_fronts()[0].threat;
    let old = planner(&one_old, 100).door_fronts()[0].threat;
    assert!(new * 100 > old * 150, "{new} vs {old}");
    // The weight scales the threat.
    assert_eq!(planner(&one_new, 200).door_fronts()[0].threat, 2 * new);
}

#[test]
fn the_door_buys_the_best_unit_once_and_only_while_short() {
    let v = view(800, &army(3, Hex::new(9, 6)));
    let mut old = planner(&v, 100);
    old.decisions.siege_survival_clock = 0;
    old.decisions.production_threat_agreement = 1;
    old.decisions.walled_defense_percent = 100;
    old.plan_door_defense();
    assert_eq!(
        buys(&old.orders),
        vec![ProductionItem::Unit {
            unit: UnitKind::Phalangite
        }]
    );
    let mut p = planner(&v, 100);
    p.decisions.siege_survival_clock = 1;
    p.decisions.production_threat_agreement = 1;
    p.decisions.walled_defense_percent = 100;
    p.plan_door_defense();
    let bought = buys(&p.orders);
    assert_eq!(
        bought.len(),
        1,
        "one purchase per city per turn: {:?}",
        p.orders
    );
    assert_eq!(
        bought[0],
        ProductionItem::Unit {
            unit: UnitKind::Phalangite
        },
        "the era's best viable unit, not the cheapest"
    );
    assert!(p.gold_left < 800);
    assert_valid(&v, &p.orders);

    // A city already holding the visible attack spends nothing.
    let mut v = view(800, &[(50, UnitKind::Soldier, Hex::new(9, 6), 30)]);
    v.units.extend(
        ViewBuilder::new(24, 14)
            .unit(1, ME, UnitKind::Phalangite, CITY)
            .build()
            .units,
    );
    let mut p = planner(&v, 100);
    assert!(p.door_fronts().iter().all(|f| f.gap() <= 0));
    p.plan_door_defense();
    assert!(buys(&p.orders).is_empty(), "{:?}", p.orders);
    assert_eq!(p.gold_left, 800);
}

#[test]
fn the_door_keeps_solvency_gold() {
    let mut v = view(300, &army(3, Hex::new(9, 6)));
    // Losing 20 a turn: ten turns of runway (reserve_deficit_turns) is 200 Gold kept.
    v.economy.gold_per_turn.total = -20;
    let mut p = planner(&v, 100);
    p.plan_door_defense();
    assert!(p.gold_left >= 200, "spent into the runway: {}", p.gold_left);
}

#[test]
fn a_unit_at_the_door_with_no_target_upgrades_with_gold() {
    // The army is three turns out; no purchase is affordable, an upgrade is.
    let mut v = view(70, &army(3, Hex::new(12, 6)));
    v.units.extend(
        ViewBuilder::new(24, 14)
            .unit(1, ME, UnitKind::Soldier, CITY)
            .build()
            .units,
    );
    v.units
        .iter_mut()
        .find(|u| u.id == UnitId(1))
        .unwrap()
        .upgrade_cost = Some(60);
    // Upgrades are off in the default part mask (`door_parts` 29); they act with bit 2.
    let mut default_parts = planner(&v, 100);
    default_parts.plan_door_defense();
    assert!(
        !default_parts
            .orders
            .iter()
            .any(|o| matches!(o, Order::UpgradeUnit { .. })),
        "{:?}",
        default_parts.orders
    );
    let mut p = planner(&v, 100);
    p.decisions.door_parts = 63;
    assert!(!p.door_fronts().is_empty());
    p.plan_door_defense();
    assert!(
        p.orders
            .iter()
            .any(|o| matches!(o, Order::UpgradeUnit { unit: UnitId(1) })),
        "{:?}",
        p.orders
    );
    assert_eq!(p.gold_left, 10);
}

#[test]
fn each_door_part_can_be_switched_off() {
    let v = view(800, &army(3, Hex::new(9, 6)));
    let mut p = planner(&v, 100);
    p.decisions.door_parts = 63 & !crate::door::DOOR_BUY & !crate::door::DOOR_PRODUCTION;
    p.plan_door_defense();
    assert!(p.orders.is_empty(), "{:?}", p.orders);
}

#[test]
fn a_wounded_attacker_beside_the_city_is_struck_by_everything_in_reach() {
    let mut v = view(0, &[(50, UnitKind::Soldier, Hex::new(7, 6), 30)]);
    v.units.extend(
        ViewBuilder::new(24, 14)
            .unit(1, ME, UnitKind::Ranger, CITY)
            .unit(2, ME, UnitKind::Phalangite, Hex::new(6, 7))
            .build()
            .units,
    );
    let mut p = planner(&v, 100);
    let units: Vec<_> = v.units.iter().filter(|u| u.owner == ME).collect();
    p.door_military(&units);
    let aimed = p
        .orders
        .iter()
        .filter(|o| match o {
            Order::RangedAttack { target, .. } => *target == Hex::new(7, 6),
            Order::MoveUnit { to, .. } => *to == Hex::new(7, 6),
            _ => false,
        })
        .count();
    assert!(aimed >= 1, "{:?}", p.orders);
    assert_valid(&v, &p.orders);
    // With the gene off nothing is planned here.
    let mut off = planner(&v, 0);
    off.door_military(&units);
    assert!(off.orders.is_empty(), "{:?}", off.orders);
}

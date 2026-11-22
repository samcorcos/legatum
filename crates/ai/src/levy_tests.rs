//! Thin levies (`levy.rs`): the Byzantium AI stays inside its cap and rivals read it.

use crate::Knobs;
use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use crate::scripted::Planner;
use legatum_core::view::{LevyView, UnavailableOption};
use legatum_core::{Faction, Order, PlayerView, ProductionItem, UnitKind};

fn byzantium(units: u32, cap: u32) -> PlayerView {
    let mut v = ViewBuilder::new(24, 16)
        .faction(Faction::Byzantium)
        .city_with_detail(1, ME, off(4, 6), 3)
        .city(2, ME, off(8, 6), 3)
        .city(3, ME, off(4, 10), 3)
        .build();
    v.players.iter_mut().find(|p| p.id == ME).unwrap().levy = Some(LevyView { units, cap });
    v
}

fn planner(v: &PlayerView) -> Planner<'_> {
    Planner::new(
        v,
        Knobs::for_faction(Faction::Byzantium),
        Faction::Byzantium,
        3,
    )
}

/// The core moves armed units from a city's options to its unavailable list at the cap.
fn move_units_to_unavailable(v: &mut PlayerView) {
    let (units, cap) = v.players[0].levy.map(|l| (l.units, l.cap)).unwrap();
    for c in &mut v.cities {
        let Some(d) = c.detail.as_mut() else { continue };
        let (armed, rest): (Vec<_>, Vec<_>) = d.options.drain(..).partition(|o| {
            matches!(o.item, ProductionItem::Unit { unit } if legatum_core::data::pays_upkeep(unit))
        });
        d.options = rest;
        d.unavailable = armed
            .into_iter()
            .map(|o| UnavailableOption {
                item: o.item,
                cost: o.cost,
                reason: format!("Levies full: {units} of {cap}"),
                prerequisite_building: None,
                production_banked: 0,
            })
            .collect();
    }
}

#[test]
fn the_army_it_wants_never_exceeds_the_cap() {
    let mut v = byzantium(2, 4);
    let wanted = planner(&v).desired_army();
    assert!(wanted <= 4, "{wanted}");
    // Without a cap the same empire wants more, so the clamp is what bounds it.
    v.players.iter_mut().find(|p| p.id == ME).unwrap().levy = None;
    assert!(planner(&v).desired_army() > 4);
}

#[test]
fn a_city_building_an_armed_unit_at_the_cap_switches_to_something_else() {
    let spearman = ProductionItem::Unit {
        unit: UnitKind::Spearman,
    };
    let mut v = byzantium(4, 4);
    for c in v.cities.iter_mut().filter(|c| c.detail.is_some()) {
        let d = c.detail.as_mut().unwrap();
        d.producing = Some(spearman);
    }
    move_units_to_unavailable(&mut v);
    let mut p = planner(&v);
    p.plan_cities();
    let set: Vec<_> = p
        .orders
        .iter()
        .filter_map(|o| match o {
            Order::SetProduction { city, item } if city.0 == 1 => Some(*item),
            _ => None,
        })
        .collect();
    assert_eq!(set.len(), 1, "{:?}", p.orders);
    assert!(
        !matches!(set[0], ProductionItem::Unit { unit } if legatum_core::data::pays_upkeep(unit)),
        "{set:?}"
    );
    // Under the cap the same city keeps an armed build: its Spearman or, with no enemy in
    // sight, a Horseman (riders stand above the melee line since 2026-10-31).
    let mut v = byzantium(3, 4);
    v.cities[0].detail.as_mut().unwrap().producing = Some(spearman);
    let mut p = planner(&v);
    p.plan_cities();
    assert!(
        p.orders
            .iter()
            .all(|o| !matches!(o, Order::SetProduction { city, item }
            if city.0 == 1 && !matches!(item, ProductionItem::Unit { unit }
                if legatum_core::data::pays_upkeep(*unit)))),
        "{:?}",
        p.orders
    );
}

#[test]
fn it_disbands_a_weak_unit_for_a_clearly_better_one_only_in_quiet_peace() {
    let waiting = |unit| UnavailableOption {
        item: ProductionItem::Unit { unit },
        cost: 60,
        reason: "Levies full: 2 of 2".into(),
        prerequisite_building: None,
        production_banked: 0,
    };
    let build = |better: bool, upgrade: Option<i32>, war: bool| {
        let mut v = ViewBuilder::new(24, 16)
            .faction(Faction::Byzantium)
            .city_with_detail(1, ME, off(4, 6), 3)
            .unit(1, ME, UnitKind::Spearman, off(4, 6))
            .unit(2, ME, UnitKind::Spearman, off(5, 6))
            .build();
        v.players.iter_mut().find(|p| p.id == ME).unwrap().levy =
            Some(LevyView { units: 2, cap: 2 });
        for u in &mut v.units {
            u.upgrade_cost = upgrade;
        }
        move_units_to_unavailable(&mut v);
        // Only the Spearman, or a Pikeman (strength 32 against 20: clearly better), is waiting.
        v.cities[0].detail.as_mut().unwrap().unavailable = vec![waiting(if better {
            UnitKind::Pikeman
        } else {
            UnitKind::Spearman
        })];
        let state = if war {
            legatum_core::diplomacy::RelationState::War
        } else {
            legatum_core::diplomacy::RelationState::Peace
        };
        v.diplomacy.legacy = false;
        let teams: Vec<_> = v.players.iter().map(|p| p.team).collect();
        for t in teams.into_iter().filter(|t| *t != v.team) {
            if v.diplomacy.relations.iter().any(|r| r.between(v.team, t)) {
                continue;
            }
            v.diplomacy
                .relations
                .push(legatum_core::diplomacy::Relation {
                    a: v.team,
                    b: t,
                    state,
                    effective_turn: 1,
                    guarantee_until: 0,
                    version: 1,
                    security: None,
                    war_id: None,
                    declarers: Vec::new(),
                });
        }
        v
    };
    let disbands = |v: &PlayerView| {
        let mut p = planner(v);
        p.plan_levy_turnover();
        p.orders
            .iter()
            .any(|o| matches!(o, Order::DisbandUnit { .. }))
    };
    assert!(disbands(&build(true, None, false)));
    assert!(
        !disbands(&build(false, None, false)),
        "an equal unit is no reason"
    );
    assert!(
        !disbands(&build(true, Some(30), false)),
        "an upgrade offer is used instead"
    );
    assert!(!disbands(&build(true, None, true)), "never at war");
}

#[test]
fn a_rival_counts_the_units_it_has_beyond_a_byzantines_public_cap() {
    let mut v = ViewBuilder::new(24, 16)
        .city_with_detail(1, ME, off(4, 6), 3)
        .city(2, ENEMY, off(14, 6), 3)
        .build();
    for i in 0..6 {
        v = {
            let mut b = ViewBuilder { view: v };
            b = b.unit(10 + i, ME, UnitKind::Spearman, off(5 + i as i32, 8));
            b.build()
        };
    }
    v.players
        .iter_mut()
        .find(|p| p.id == ENEMY)
        .unwrap()
        .faction = Faction::Byzantium;
    v.players.iter_mut().find(|p| p.id == ENEMY).unwrap().levy =
        Some(LevyView { units: 3, cap: 3 });
    let p = Planner::new(
        &v,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        3,
    );
    let enemy_team = v.team_of(ENEMY);
    assert_eq!(p.levy_edge_odds(enemy_team), 12);
    // No bonus against a team without a cap.
    v.players.iter_mut().find(|p| p.id == ENEMY).unwrap().levy = None;
    let p = Planner::new(
        &v,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        3,
    );
    assert_eq!(p.levy_edge_odds(enemy_team), 0);
}

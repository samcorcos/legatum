use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use legatum_core::diplomacy::{DiplomaticAction, TreatyKind};
use legatum_core::{CityId, TeamId, UnitId};

const OTHER: legatum_core::PlayerId = legatum_core::PlayerId(3);

#[test]
fn movement_will_not_stage_units_inside_an_allied_city() {
    let city_pos = off(8, 6);
    let mut view = ViewBuilder::new(18, 12)
        .city(1, ENEMY, city_pos, 3)
        .unit(1, ME, UnitKind::Scout, off(7, 6))
        .unit(2, ME, UnitKind::Spearman, off(7, 7))
        .build();
    let ours = view.team_of(ME);
    let theirs = view.team_of(ENEMY);
    view.diplomacy
        .relations
        .push(legatum_core::diplomacy::Relation {
            a: ours.min(theirs),
            b: ours.max(theirs),
            state: legatum_core::diplomacy::RelationState::Alliance,
            effective_turn: view.turn,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: vec![],
        });
    let mut plan = HumanPlaybookV7Brain::default().build_plan(&view);
    for unit in view.units.iter().filter(|u| u.owner == ME) {
        assert!(
            !plan.go(unit, city_pos, false),
            "{:?} was staged in the allied city",
            unit.kind
        );
    }
}

fn economy() -> PlayerView {
    ViewBuilder::new(24, 20)
        .known_techs(&[TechId::Irrigation, TechId::Currency, TechId::BronzeWorking])
        .city(1, ME, off(4, 5), 3)
        .city(2, ME, off(4, 11), 3)
        .city(3, ME, off(10, 11), 3)
        .city(4, ENEMY, off(11, 5), 2)
        .city(5, ENEMY, off(17, 5), 2)
        .unit(1, ME, UnitKind::Spearman, off(7, 5))
        .build()
}

/// The same view with the rivals at Peace (a missing relation is Peace) and every war
/// declaration, treaty offer and acceptance legal.
fn at_peace(mut view: PlayerView) -> PlayerView {
    view.diplomacy.legacy = false;
    for against in [TeamId(2), TeamId(3)] {
        for order in [
            Order::DeclareWar {
                against,
                breach: false,
            },
            Order::ProposeTreaty {
                to: against,
                kind: TreatyKind::Peace,
            },
        ] {
            view.diplomacy.actions.push(DiplomaticAction {
                order,
                effective_turn: view.turn,
                cost: 0,
                upkeep: 0,
                betrayal: false,
                new_wars: Vec::new(),
                calls: Vec::new(),
                also_agree: Vec::new(),
                blocked_reason: None,
            });
        }
    }
    view
}

fn soldier_production(city: u32) -> Order {
    Order::SetProduction {
        city: CityId(city),
        item: ProductionItem::Unit {
            unit: UnitKind::Spearman,
        },
    }
}

fn is_settler_order(order: &Order) -> bool {
    matches!(
        order,
        Order::SetProduction {
            item: ProductionItem::Unit {
                unit: UnitKind::Settler
            },
            ..
        } | Order::BuyItem {
            item: ProductionItem::Unit {
                unit: UnitKind::Settler
            },
            ..
        }
    )
}

#[test]
fn trains_soldiers_and_queues_no_early_settler() {
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(4, 5), 2)
        .unit(1, ME, UnitKind::Spearman, off(5, 5))
        .gold(400)
        .build();
    let orders = HumanPlaybookV7Brain::default().orders(&view);
    assert!(orders.contains(&soldier_production(1)), "{orders:?}");
    assert!(!orders.iter().any(is_settler_order), "{orders:?}");
}

#[test]
fn a_settler_waits_for_the_first_wave_of_soldiers() {
    let mut view = ViewBuilder::new(20, 16)
        .city(1, ME, off(4, 5), 3)
        .unit(1, ME, UnitKind::Spearman, off(5, 5))
        .unit(2, ME, UnitKind::Spearman, off(5, 6))
        .gold(50)
        .build();
    let early = HumanPlaybookV7Brain::default().orders(&view);
    assert!(!early.iter().any(is_settler_order), "{early:?}");
    view.units.extend(
        ViewBuilder::new(20, 16)
            .unit(3, ME, UnitKind::Spearman, off(6, 5))
            .build()
            .units
            .iter()
            .cloned(),
    );
    let later = HumanPlaybookV7Brain::default().orders(&view);
    assert!(later.iter().any(is_settler_order), "{later:?}");
}

#[test]
fn buys_soldiers_toward_the_goal_and_not_a_firing_line_first() {
    let mut view = ViewBuilder::new(20, 16)
        .city(1, ME, off(4, 5), 2)
        .gold(400)
        .build();
    for o in &mut view.cities[0].detail.as_mut().unwrap().options {
        if matches!(
            o.item,
            ProductionItem::Unit {
                unit: UnitKind::Archer | UnitKind::Spearman
            }
        ) {
            o.buy_cost = Some(70);
        }
    }
    let orders = HumanPlaybookV7Brain::default().orders(&view);
    assert!(
        orders.contains(&Order::BuyItem {
            city: CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman
            },
        }),
        "{orders:?}"
    );
}

/// Four Spearmen two hexes from a city held by a Spearman and an Archer, at Peace.
fn assault_view() -> PlayerView {
    let target = off(10, 5);
    at_peace(
        ViewBuilder::new(24, 16)
            .city(1, ME, off(3, 5), 3)
            .city(2, ENEMY, target, 3)
            .unit(1, ME, UnitKind::Spearman, off(8, 5))
            .unit(2, ME, UnitKind::Spearman, off(8, 4))
            .unit(3, ME, UnitKind::Spearman, off(8, 6))
            .unit(4, ME, UnitKind::Spearman, off(9, 3))
            .unit(5, ENEMY, UnitKind::Spearman, target)
            .unit(6, ENEMY, UnitKind::Archer, off(11, 5))
            .build(),
    )
}

fn conquering() -> HumanPlaybookV7Brain {
    HumanPlaybookV7Brain {
        phase: Phase::Conquer,
        ..Default::default()
    }
}

#[test]
fn declares_war_and_strikes_a_defended_city_with_soldiers_in_the_same_turn() {
    let view = assault_view();
    let target = off(10, 5);
    let orders = conquering().orders(&view);
    assert!(
        orders.contains(&Order::DeclareWar {
            against: TeamId(2),
            breach: false
        }),
        "{orders:?}"
    );
    let strikers = orders
        .iter()
        .filter(|o| matches!(o, Order::MoveUnitPath { to, .. } if *to == target))
        .count();
    // Two reach the city this turn; the rest close to its walls.
    assert!(strikers >= 2, "{orders:?}");
}

#[test]
fn waits_at_peace_for_the_third_soldier() {
    let mut view = assault_view();
    view.units
        .retain(|u| !matches!(u.id, UnitId(3) | UnitId(4)));
    let orders = conquering().orders(&view);
    assert!(
        !orders.iter().any(|o| matches!(o, Order::DeclareWar { .. })),
        "{orders:?}"
    );
}

#[test]
fn takes_exposed_settler_and_declares_war_to_do_it() {
    let target = off(7, 5);
    let view = at_peace(
        ViewBuilder::new(20, 16)
            .city(1, ME, off(3, 5), 3)
            .unit(1, ME, UnitKind::Spearman, off(6, 5))
            .unit(2, ENEMY, UnitKind::Settler, target)
            .build(),
    );
    let orders = HumanPlaybookV7Brain::default().orders(&view);
    assert!(
        orders.contains(&Order::DeclareWar {
            against: TeamId(2),
            breach: false
        }),
        "{orders:?}"
    );
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnitPath { unit: UnitId(1), to, .. } if *to == target)),
        "{orders:?}"
    );
}

#[test]
fn volley_then_melee_captures_instead_of_attacking_independently() {
    let target = off(8, 5);
    let mut view = ViewBuilder::new(20, 16)
        .city(1, ME, off(3, 5), 3)
        .city(2, ENEMY, target, 2)
        .unit(1, ME, UnitKind::Archer, off(6, 5))
        .unit(2, ME, UnitKind::Spearman, off(7, 5))
        .build();
    view.cities
        .iter_mut()
        .find(|c| c.id == CityId(2))
        .unwrap()
        .hp = 5;
    let orders = HumanPlaybookV7Brain::default().orders(&view);
    assert!(orders.contains(&Order::RangedAttack {
        unit: UnitId(1),
        target
    }));
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnitPath { unit: UnitId(2), to, .. } if *to == target)),
        "{orders:?}"
    );
}

#[test]
fn never_consolidates_after_captures_and_rolls_on_to_the_next_city() {
    let view = economy();
    let mut brain = conquering();
    brain.orders(&view);
    assert!(brain.target.is_some());
    let mut captured = view.clone();
    captured.cities[3].owner = ME;
    captured.cities[3].detail = captured.cities[0].detail.clone();
    captured.turn += 1;
    brain.orders(&captured);
    assert_eq!(brain.captured_cities.len(), 1);
    // The army's next target is the rival's other city.
    assert_eq!(brain.target, Some(CityId(5)));
    captured.cities[4].owner = ME;
    captured.cities[4].detail = captured.cities[0].detail.clone();
    let orders = brain.orders(&captured);
    assert_eq!(brain.phase, Phase::Conquer);
    // Late in the game, with five cities and three captures, there is still no peace.
    captured.turn = 230 + 40;
    let orders_late = brain.orders(&captured);
    assert_eq!(brain.phase, Phase::Conquer);
    for orders in [orders, orders_late] {
        assert!(
            !orders
                .iter()
                .any(|o| matches!(o, Order::ProposeTreaty { .. })),
            "{orders:?}"
        );
    }
    captured.finished = true;
    assert!(brain.orders(&captured).is_empty());
}

#[test]
fn view_memory_is_repeatable() {
    let view = at_peace(economy());
    let mut a = HumanPlaybookV7Brain::default();
    let mut b = HumanPlaybookV7Brain::default();
    assert_eq!(a.orders(&view), b.orders(&view));
    let mut later = view.clone();
    later.turn += 1;
    assert_eq!(a.orders(&later), b.orders(&later));
}

#[test]
fn a_stalled_war_asks_for_peace_and_moves_the_army_to_another_front() {
    let view = ViewBuilder::new(30, 16)
        .city(1, ME, off(4, 5), 3)
        .city(2, ENEMY, off(10, 5), 2)
        .city(3, ENEMY, off(12, 8), 2)
        .city(4, OTHER, off(10, 12), 2)
        .unit(1, ME, UnitKind::Spearman, off(7, 5))
        .build();
    let mut view = at_peace(view);
    // War with the first rival only.
    view.diplomacy
        .relations
        .push(legatum_core::diplomacy::Relation {
            a: TeamId(0),
            b: TeamId(2),
            state: legatum_core::diplomacy::RelationState::War,
            effective_turn: 1,
            guarantee_until: 0,
            version: 0,
            security: None,
            war_id: None,
            declarers: vec![TeamId(0)],
        });
    let mut brain = conquering();
    let first = brain.orders(&view);
    assert!(
        !first
            .iter()
            .any(|o| matches!(o, Order::ProposeTreaty { .. })),
        "{first:?}"
    );
    let owner_of_target = |brain: &HumanPlaybookV7Brain| {
        brain
            .target
            .and_then(|id| view.cities.iter().find(|c| c.id == id))
            .map(|c| c.owner)
    };
    assert_eq!(owner_of_target(&brain), Some(ENEMY));
    // Many turns without a gain, then the offer, and a target on the other front.
    view.turn += 16 + 1;
    let stalled = brain.orders(&view);
    assert!(
        stalled.contains(&Order::ProposeTreaty {
            to: TeamId(2),
            kind: TreatyKind::Peace
        }),
        "{stalled:?}"
    );
    assert_eq!(owner_of_target(&brain), Some(OTHER));
}

#[test]
fn only_sparta_fights_on_and_walks_wounded() {
    let build = |faction| {
        ViewBuilder::new(24, 16)
            .faction(faction)
            .city(1, ME, off(4, 5), 3)
            .city(2, ENEMY, off(20, 5), 3)
            .unit_hp(1, ME, UnitKind::Spearman, off(12, 5), 55)
            .build()
    };
    let moves = |orders: &[Order]| {
        orders.iter().any(|o| {
            matches!(
                o,
                Order::MoveUnitPath {
                    unit: UnitId(1),
                    ..
                }
            )
        })
    };
    assert!(moves(&conquering().orders(&build(Faction::Sparta))));
    assert!(!moves(&conquering().orders(&build(Faction::Byzantium))));
}

#[test]
fn researches_siege_as_soon_as_the_campaign_prerequisites_are_complete() {
    let view = ViewBuilder::new(24, 20)
        .known_techs(&[
            TechId::Irrigation,
            TechId::BronzeWorking,
            TechId::Currency,
            TechId::Writing,
            TechId::Masonry,
            TechId::Feudalism,
            TechId::Engineering,
            TechId::IronWorking,
        ])
        .city(1, ME, off(4, 5), 3)
        .build();
    assert!(
        HumanPlaybookV7Brain::default()
            .orders(&view)
            .contains(&Order::SetResearch {
                tech: TechId::Mathematics,
            })
    );
}

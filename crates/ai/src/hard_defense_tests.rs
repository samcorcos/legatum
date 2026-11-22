//! Exercise readiness through the public brain: diplomacy, production, civilians,
//! military planning and final movement reconciliation must agree on the orders.
use crate::{
    Brain, ScriptedBrain,
    fixture::{ENEMY, ME, ViewBuilder, assert_valid, off},
};
use legatum_core::{
    AiDifficulty, BuildingKind, Hex, Order, PlayerId, PlayerView, ProductionItem, TeamId, UnitId,
    UnitKind,
    diplomacy::{DiplomaticAction, Relation, RelationState, TreatyKind},
    view::TeamStanding,
};

fn plan(view: &PlayerView, difficulty: AiDifficulty) -> Vec<Order> {
    let orders = crate::fixture::brain(view, 7, difficulty).plan(view);
    assert_valid(view, &orders);
    orders
}

/// `plan` with gene `buy_hold` pinned to 0: this test is about readiness funding and the treasury
/// reserve, which buy_hold 3 (default since 2026-10-31) holds Gold from; the gene has its own tests.
fn plan_without_hold(view: &PlayerView, difficulty: AiDifficulty) -> Vec<Order> {
    let mut brain = crate::fixture::brain(view, 7, difficulty);
    brain.decisions.buy_hold = 0;
    let orders = brain.plan(view);
    assert_valid(view, &orders);
    orders
}

/// `plan` with gene `runaway_coalition` pinned to 0: these tests are about the ordinary Peace
/// gates; the runaway gate refuses Peace with a runaway even when home is overrun.
fn plan_without_runaway(view: &PlayerView, difficulty: AiDifficulty) -> Vec<Order> {
    let mut brain = crate::fixture::brain(view, 7, difficulty);
    brain.decisions.runaway_coalition = 0;
    let orders = brain.plan(view);
    assert_valid(view, &orders);
    orders
}

fn relation(view: &mut PlayerView, state: RelationState) {
    view.diplomacy.legacy = false;
    let (a, b) = (view.team, view.team_of(ENEMY));
    view.diplomacy.relations.push(Relation {
        a,
        b,
        state,
        effective_turn: 1,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: (state == RelationState::War).then_some(1),
        declarers: vec![b],
    });
}

#[test]
fn threatened_city_keeps_garrison_and_calls_a_melee_supporter() {
    let city = Hex::new(7, 7);
    let mut view = ViewBuilder::new(20, 16)
        .city(1, ME, city, 3)
        .city(2, ENEMY, Hex::new(16, 7), 3)
        .unit(1, ME, UnitKind::Horseman, city)
        .unit(2, ME, UnitKind::Spearman, Hex::new(6, 7))
        .unit(3, ME, UnitKind::Archer, Hex::new(6, 6))
        .unit(50, ENEMY, UnitKind::Phalangite, Hex::new(8, 7))
        .unit(51, ENEMY, UnitKind::Phalangite, Hex::new(8, 6))
        .unit(52, ENEMY, UnitKind::Phalangite, Hex::new(7, 8))
        .build();
    relation(&mut view, RelationState::War);
    view.turn = 40;
    view.cities[0].hp = 10;
    view.units
        .iter_mut()
        .find(|u| u.id == UnitId(1))
        .unwrap()
        .hp = 20;
    let mut brain = crate::fixture::brain(&view, 7, AiDifficulty::Hard);
    let orders = brain.plan(&view);
    assert_valid(&view, &orders);
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit: UnitId(2), to } if *to == city)),
        "{orders:?}"
    );
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit: UnitId(1), to } if *to == Hex::new(6, 7))),
        "{orders:?}"
    );
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit: UnitId(3), to } if *to == city)),
        "{orders:?}"
    );
}

#[test]
fn athens_t40_visible_line_orders_spearman_46_into_city() {
    // Hand-made PlayerView cut from the rules-39 tape's T40 state: city 11 at (7,14)
    // has 116 HP, #35 is the full-HP garrison, #46 is one tile away, #3 is wounded.
    let city = Hex::new(7, 14);
    let mut view = ViewBuilder::new(22, 25)
        .faction(legatum_core::Faction::Athens)
        .city(11, ME, city, 4)
        .city(6, ME, Hex::new(6, 18), 3)
        .city(4, ENEMY, Hex::new(12, 7), 5)
        .unit(35, ME, UnitKind::Spearman, city)
        .unit(46, ME, UnitKind::Spearman, Hex::new(7, 15))
        .unit(3, ME, UnitKind::Phalangite, Hex::new(6, 14))
        .unit(19, ENEMY, UnitKind::Spearman, Hex::new(7, 13))
        .unit(39, ENEMY, UnitKind::Spearman, Hex::new(8, 13))
        .unit(42, ENEMY, UnitKind::Spearman, Hex::new(9, 13))
        .build();
    relation(&mut view, RelationState::War);
    view.turn = 40;
    view.players
        .iter_mut()
        .find(|p| p.id == ENEMY)
        .unwrap()
        .faction = legatum_core::Faction::Sparta;
    view.cities
        .iter_mut()
        .find(|c| c.id == legatum_core::CityId(11))
        .unwrap()
        .hp = 116;
    for (id, hp) in [(3, 22), (19, 23), (39, 92), (42, 57)] {
        view.units
            .iter_mut()
            .find(|u| u.id == UnitId(id))
            .unwrap()
            .hp = hp;
    }
    let mut brain = crate::fixture::brain(&view, 7, AiDifficulty::Hard);
    // The cut-down view chooses Contain, while the taped T40 seat was on Economy.
    // Keep this test on the tactical pass; the tape replay checks full planning.
    brain.plan_layer = false;
    let orders = brain.plan(&view);
    assert_valid(&view, &orders);
    assert!(
        orders.iter().any(|o| matches!(o,
        Order::MoveUnit { unit: UnitId(46), to } if *to == city)),
        "{orders:?}"
    );
    assert!(
        !orders.iter().any(|o| matches!(
            o,
            Order::MoveUnit {
                unit: UnitId(35),
                ..
            } | Order::MoveUnitPath {
                unit: UnitId(35),
                ..
            }
        )),
        "{orders:?}"
    );
}

#[test]
fn athens_t36_garrisons_before_a_legal_spartan_declaration() {
    let city = Hex::new(7, 14);
    let mut view = ViewBuilder::new(22, 25)
        .faction(legatum_core::Faction::Athens)
        .city(11, ME, city, 4)
        .city(6, ME, Hex::new(6, 18), 3)
        .city(4, ENEMY, Hex::new(12, 7), 4)
        .unit(3, ME, UnitKind::Spearman, Hex::new(6, 14))
        .unit(35, ME, UnitKind::Spearman, Hex::new(8, 17))
        .unit(19, ENEMY, UnitKind::Spearman, Hex::new(8, 11))
        .unit(39, ENEMY, UnitKind::Spearman, Hex::new(6, 12))
        .unit(42, ENEMY, UnitKind::Spearman, Hex::new(9, 9))
        .build();
    view.turn = 36;
    view.diplomacy.legacy = false;
    let enemy_team = view.team_of(ENEMY);
    view.diplomacy.actions.push(DiplomaticAction {
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        order: Order::DeclareWar {
            against: enemy_team,
            breach: false,
        },
        effective_turn: view.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    });
    let orders = plan(&view, AiDifficulty::Hard);
    assert!(
        orders.iter().any(|o| matches!(o,
        Order::MoveUnit { unit: UnitId(3), to } if *to == city)),
        "{orders:?}"
    );
}

#[test]
fn scout_can_capture_broken_city_so_a_free_spearman_garrisons_it() {
    let city = Hex::new(6, 6);
    let mut view = ViewBuilder::new(20, 14)
        .city(1, ME, city, 3)
        .city(2, ENEMY, Hex::new(16, 6), 3)
        .unit(1, ME, UnitKind::Spearman, Hex::new(4, 6))
        .unit(50, ENEMY, UnitKind::Scout, Hex::new(9, 6))
        .build();
    relation(&mut view, RelationState::War);
    view.cities
        .iter_mut()
        .find(|c| c.id == legatum_core::CityId(1))
        .unwrap()
        .hp = 0;
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    p.prepare_hard_defense();
    assert!(
        p.ctx.threat[0] > 0,
        "a capturing Scout has nonzero threat power"
    );
    let scout = view.units.iter().find(|u| u.id == UnitId(50)).unwrap();
    assert!(
        p.enemy_threat(scout, scout.hp) >= legatum_core::data::CITY_HP,
        "a Scout that reaches a broken, ungarrisoned city carries its capture stake"
    );
    let orders = plan(&view, AiDifficulty::Hard);
    assert!(
        orders.iter().any(|o| matches!(o,
        Order::MoveUnit { unit: UnitId(1), to } if *to == city)),
        "the free Spearman should protect the city: {orders:?}"
    );
    let mut legacy = crate::fixture::planner(&view, 7);
    legacy.difficulty = AiDifficulty::Hard;
    legacy.decisions.scout_threat = 0;
    legacy.prepare_hard_defense();
    assert_eq!(
        legacy.ctx.threat[0], 0,
        "zero gene restores the old Scout exclusion"
    );
}

#[test]
fn walled_city_forecast_to_fall_continues_defender_production() {
    let city = Hex::new(6, 6);
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, city, 3)
        .city(2, ENEMY, Hex::new(18, 6), 3)
        .unit(1, ME, UnitKind::Archer, city)
        .unit(50, ENEMY, UnitKind::Archer, Hex::new(8, 6))
        .unit(51, ENEMY, UnitKind::Archer, Hex::new(8, 5))
        .unit(52, ENEMY, UnitKind::Archer, Hex::new(8, 7))
        .unit(53, ENEMY, UnitKind::Spearman, Hex::new(7, 6))
        .unit(54, ENEMY, UnitKind::Spearman, Hex::new(7, 5))
        .unit(55, ENEMY, UnitKind::Spearman, Hex::new(7, 7))
        .gold(300)
        .build();
    relation(&mut view, RelationState::War);
    view.cities[0].walls = true;
    view.cities[0].hp = 0;
    view.units
        .iter_mut()
        .find(|u| u.id == UnitId(1))
        .unwrap()
        .hp = 1;
    view.units
        .iter_mut()
        .find(|u| u.id == UnitId(53))
        .unwrap()
        .siege = true;
    let mut model = crate::opponents::OpponentAssessments::default();
    model.update(&view);
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    p.opponents = Some(&model);
    p.prepare_hard_defense();
    let home = p.ctx.own_cities[0];
    assert!(
        p.defense_risk(home, None).falls,
        "the siege forecast must put the city at risk"
    );
    p.plan_threat_defense();
    assert!(
        p.orders.iter().any(|o| matches!(o,
        Order::SetProduction { city: legatum_core::CityId(1), item: ProductionItem::Unit { unit } }
            if legatum_core::data::unit_stats(*unit).military)),
        "Walls should not end defence while the city is forecast to fall: {:?}",
        p.orders
    );
    let mut legacy = crate::fixture::planner(&view, 7);
    legacy.difficulty = AiDifficulty::Hard;
    legacy.opponents = Some(&model);
    legacy.decisions.walled_defense_percent = 0;
    legacy.decisions.army_door_production = 0;
    legacy.plan_threat_defense();
    assert!(
        legacy.orders.is_empty(),
        "zero gene restores the Walls stopping point"
    );
}

#[test]
fn army_at_war_replaces_a_front_city_build_until_its_gap_closes() {
    let city = Hex::new(6, 6);
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, city, 4)
        .city(2, ENEMY, Hex::new(18, 6), 3)
        .unit(50, ENEMY, UnitKind::Archer, Hex::new(8, 6))
        .unit(51, ENEMY, UnitKind::Archer, Hex::new(8, 5))
        .unit(52, ENEMY, UnitKind::Archer, Hex::new(8, 7))
        .unit(53, ENEMY, UnitKind::Archer, Hex::new(9, 6))
        .unit(54, ENEMY, UnitKind::Archer, Hex::new(9, 5))
        .build();
    relation(&mut view, RelationState::War);
    view.cities[0].walls = true;
    view.cities[0].detail.as_mut().unwrap().producing = Some(ProductionItem::Building {
        building: BuildingKind::Granary,
    });
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    assert!(p.door_fronts().iter().any(|f| f.gap() > 0));
    p.plan_threat_defense();
    assert!(
        p.orders.iter().any(|o| matches!(o,
        Order::SetProduction { city: legatum_core::CityId(1), item: ProductionItem::Unit { unit } }
            if crate::composition::is_combat(*unit))),
        "{:?}",
        p.orders
    );

    let mut off = crate::fixture::planner(&view, 7);
    off.difficulty = AiDifficulty::Hard;
    off.decisions.army_door_production = 0;
    off.plan_threat_defense();
    assert!(off.orders.is_empty(), "{:?}", off.orders);

    view.diplomacy.relations[0].state = RelationState::Peace;
    let mut peace = crate::fixture::planner(&view, 7);
    peace.difficulty = AiDifficulty::Hard;
    peace.plan_threat_defense();
    assert!(peace.orders.is_empty(), "{:?}", peace.orders);
}

#[test]
fn nearest_city_builds_a_second_defender_for_the_same_front() {
    let front = Hex::new(6, 6);
    let reserve = Hex::new(10, 6);
    let mut builder = ViewBuilder::new(24, 16)
        .city(1, ME, front, 4)
        .city(3, ME, reserve, 4)
        .city(2, ENEMY, Hex::new(18, 6), 3);
    for (id, at) in [
        Hex::new(5, 6),
        Hex::new(5, 5),
        Hex::new(4, 6),
        Hex::new(5, 7),
        Hex::new(4, 5),
        Hex::new(4, 7),
        Hex::new(3, 6),
        Hex::new(3, 5),
    ]
    .into_iter()
    .enumerate()
    {
        builder = builder.unit(50 + id as u32, ENEMY, UnitKind::Archer, at);
    }
    let mut view = builder.build();
    relation(&mut view, RelationState::War);
    for city in view.cities.iter_mut().filter(|c| c.owner == ME) {
        city.walls = true;
        city.detail.as_mut().unwrap().producing = Some(ProductionItem::Building {
            building: BuildingKind::Granary,
        });
    }
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    assert!(
        p.door_fronts()
            .iter()
            .any(|f| f.city == legatum_core::CityId(1) && f.gap() > 0)
    );
    assert!(
        p.door_fronts()
            .iter()
            .all(|f| f.city != legatum_core::CityId(3))
    );
    p.plan_threat_defense();
    assert!(
        p.orders.iter().any(|o| matches!(o,
        Order::SetProduction { city: legatum_core::CityId(3), item: ProductionItem::Unit { unit } }
            if crate::composition::is_combat(*unit))),
        "{:?}",
        p.orders
    );
}

#[test]
fn war_front_trains_the_newest_available_line_tier() {
    let city = Hex::new(6, 6);
    let mut view = ViewBuilder::new(24, 16)
        .known_techs(&[
            legatum_core::TechId::Feudalism,
            legatum_core::TechId::Gunpowder,
        ])
        .city(1, ME, city, 4)
        .city(2, ENEMY, Hex::new(18, 6), 3)
        .unit(50, ENEMY, UnitKind::Cavalry, Hex::new(8, 6))
        .unit(51, ENEMY, UnitKind::Cavalry, Hex::new(8, 5))
        .unit(52, ENEMY, UnitKind::Cavalry, Hex::new(8, 7))
        .build();
    relation(&mut view, RelationState::War);
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    assert!(p.door_fronts().iter().any(|f| f.gap() > 0));
    p.plan_threat_defense();
    assert!(
        p.orders.iter().any(|o| matches!(
            o,
            Order::SetProduction {
                city: legatum_core::CityId(1),
                item: ProductionItem::Unit {
                    unit: UnitKind::Musketman
                }
            }
        )),
        "{:?}",
        p.orders
    );
}

#[test]
fn nearly_finished_defender_completes_before_retooling() {
    let city = Hex::new(6, 6);
    let mut view = ViewBuilder::new(24, 16)
        .known_techs(&[legatum_core::TechId::Feudalism])
        .city(1, ME, city, 4)
        .city(2, ENEMY, Hex::new(18, 6), 3)
        .unit(50, ENEMY, UnitKind::Cavalry, Hex::new(8, 6))
        .unit(51, ENEMY, UnitKind::Cavalry, Hex::new(8, 5))
        .unit(52, ENEMY, UnitKind::Cavalry, Hex::new(8, 7))
        .build();
    relation(&mut view, RelationState::War);
    let detail = view.cities[0].detail.as_mut().unwrap();
    detail.producing = Some(ProductionItem::Unit {
        unit: UnitKind::Spearman,
    });
    detail
        .options
        .iter_mut()
        .find(|o| {
            o.item
                == ProductionItem::Unit {
                    unit: UnitKind::Spearman,
                }
        })
        .unwrap()
        .turns = 1;
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    assert!(p.door_fronts().iter().any(|f| f.gap() > 0));
    p.plan_threat_defense();
    assert!(
        !p.orders.iter().any(|o| matches!(
            o,
            Order::SetProduction {
                city: legatum_core::CityId(1),
                ..
            }
        )),
        "{:?}",
        p.orders
    );
}

fn action(view: &mut PlayerView, order: Order) {
    view.diplomacy.actions.push(DiplomaticAction {
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        order,
        effective_turn: view.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    });
}

#[test]
fn peaceful_army_at_the_gate_has_tactical_exposure() {
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, Hex::new(8, 8), 3)
        .unit(1, ME, UnitKind::Archer, Hex::new(7, 8))
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(9, 8))
        .unit(51, ENEMY, UnitKind::Archer, Hex::new(9, 7))
        .build();
    relation(&mut view, RelationState::Peace);
    action(
        &mut view,
        Order::DeclareWar {
            against: TeamId(2),
            breach: false,
        },
    );
    let mut p = crate::fixture::planner(&view, 7);
    assert!(p.enemy_exposure_key(Hex::new(7, 8)).0 > 0);
    let archer = p.ctx.own_units[0];
    p.avoid_free_damage(&[archer]);
    assert!(
        p.orders.iter().any(|order| matches!(order,
            Order::MoveUnit { unit: UnitId(1), to }
                if *to == Hex::new(8, 8) || to.distance(Hex::new(9, 8)) > 2
        )),
        "a field archer should take city cover or leave their reach: {:?}",
        p.orders
    );
    let mut legacy = crate::fixture::planner(&view, 7);
    legacy.decisions.peace_exposure_percent = 0;
    assert_eq!(legacy.enemy_exposure_key(Hex::new(7, 8)).0, 0);
}

#[test]
fn fast_army_two_turns_away_threatens_a_peaceful_city() {
    let city = Hex::new(6, 6);
    let mut view = ViewBuilder::new(28, 16)
        .city(1, ME, city, 3)
        .unit(50, ENEMY, UnitKind::Horseman, Hex::new(13, 6))
        .unit(51, ENEMY, UnitKind::Horseman, Hex::new(13, 7))
        .unit(52, ENEMY, UnitKind::Horseman, Hex::new(13, 5))
        .build();
    relation(&mut view, RelationState::Peace);
    action(
        &mut view,
        Order::DeclareWar {
            against: TeamId(2),
            breach: false,
        },
    );
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    p.prepare_hard_defense();
    assert!(
        p.ctx.threat[0] > 0,
        "mounted army can arrive within two turns"
    );
    let mut legacy = crate::fixture::planner(&view, 7);
    legacy.difficulty = AiDifficulty::Hard;
    legacy.decisions.threat_horizon_turns = 0;
    legacy.prepare_hard_defense();
    assert_eq!(legacy.ctx.threat[0], 0);
}

#[test]
fn nearby_army_refreshes_opponent_assessment_next_turn() {
    let mut view = ViewBuilder::new(28, 16)
        .city(1, ME, Hex::new(6, 6), 3)
        .unit(50, ENEMY, UnitKind::Horseman, Hex::new(13, 6))
        .build();
    relation(&mut view, RelationState::Peace);
    let mut brain = crate::fixture::brain(&view, 7, AiDifficulty::Hard);
    let mut old_view = view.clone();
    brain.plan(&view);
    assert_eq!(brain.opponents.rivals[&ENEMY].assessed_turn, 20);
    view.turn = 21;
    view.units[0].pos = Hex::new(12, 6);
    brain.plan(&view);
    assert_eq!(brain.opponents.rivals[&ENEMY].assessed_turn, 21);
    let mut legacy = crate::fixture::brain(&old_view, 7, AiDifficulty::Hard);
    legacy.decisions.assessment_interval = 0;
    legacy.plan(&old_view);
    old_view.turn = 21;
    old_view.units[0].pos = Hex::new(12, 6);
    legacy.plan(&old_view);
    assert_eq!(legacy.opponents.rivals[&ENEMY].assessed_turn, 20);
}

#[test]
fn unseen_approach_and_rival_border_call_for_a_settler_escort() {
    let from = Hex::new(4, 6);
    let site = Hex::new(11, 6);
    let mut view = ViewBuilder::new(30, 14)
        .city(1, ME, Hex::new(2, 6), 3)
        .unit(1, ME, UnitKind::Settler, from)
        .unit(2, ME, UnitKind::Spearman, Hex::new(3, 6))
        .owner(Hex::new(10, 9), ENEMY)
        .build();
    relation(&mut view, RelationState::Peace);
    action(
        &mut view,
        Order::DeclareWar {
            against: TeamId(2),
            breach: false,
        },
    );
    for h in [Hex::new(7, 5), Hex::new(8, 5), Hex::new(9, 5)] {
        let i = view.index(h).unwrap();
        view.tiles[i].visible = false;
        view.tiles[i].explored = false;
        view.tiles[i].terrain = None;
    }
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    assert!(p.settler_trip_risk(from, site) >= p.decisions.settler_risk_escort);
    p.orders.push(Order::MoveUnit {
        unit: UnitId(1),
        to: site,
    });
    let guard = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect::<Vec<_>>();
    p.guard_exposed_settlers(&guard);
    assert!(p.orders.iter().any(|o| matches!(
        o,
        Order::MoveUnit {
            unit: UnitId(2),
            ..
        }
    )));
    let mut legacy = crate::fixture::planner(&view, 7);
    legacy.difficulty = AiDifficulty::Hard;
    legacy.decisions.fog_escort_percent = 0;
    assert_eq!(legacy.settler_trip_risk(from, site), 0);
    legacy.orders.push(Order::MoveUnit {
        unit: UnitId(1),
        to: site,
    });
    let guard = legacy
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect::<Vec<_>>();
    legacy.guard_exposed_settlers(&guard);
    assert!(!legacy.orders.iter().any(|o| matches!(
        o,
        Order::MoveUnit {
            unit: UnitId(2),
            ..
        }
    )));

    let mut alone = view.clone();
    alone.units.retain(|u| u.id == UnitId(1));
    let mut p = crate::fixture::planner(&alone, 7);
    p.difficulty = AiDifficulty::Hard;
    p.orders.push(Order::MoveUnit {
        unit: UnitId(1),
        to: site,
    });
    p.guard_exposed_settlers(&[]);
    // It does not cross the unseen flank alone, and it does not wait for a guard on open
    // ground either: it goes back to its city.
    assert!(
        p.orders.contains(&Order::MoveUnit {
            unit: UnitId(1),
            to: Hex::new(2, 6)
        }),
        "{:?}",
        p.orders
    );
}

#[test]
fn dispatch_funds_visible_peacetime_fronts_from_one_treasury_only_when_war_is_legal() {
    let mut view = ViewBuilder::new(36, 20)
        .city(1, ME, Hex::new(6, 6), 3)
        .city(2, ME, Hex::new(16, 6), 3)
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
        .unit(51, ENEMY, UnitKind::Archer, Hex::new(10, 5))
        .unit(52, ENEMY, UnitKind::Spearman, Hex::new(20, 6))
        .unit(53, ENEMY, UnitKind::Archer, Hex::new(20, 5))
        .build();
    relation(&mut view, RelationState::Peace);
    action(
        &mut view,
        Order::DeclareWar {
            against: TeamId(2),
            breach: false,
        },
    );
    for city in &mut view.cities {
        let detail = city.detail.as_mut().unwrap();
        detail.producing = Some(ProductionItem::Building {
            building: BuildingKind::Market,
        });
        detail.options.retain(|o| {
            o.item
                == (ProductionItem::Unit {
                    unit: UnitKind::Spearman,
                })
        });
        detail.options[0].buy_cost = Some(200);
    }
    // Readiness is defense and can spend the ordinary reserve once war is legal.
    for (gold, expected) in [(300, 1), (500, 2)] {
        view.economy.gold = gold;
        let orders = plan_without_hold(&view, AiDifficulty::Hard);
        let purchases: Vec<_> = orders
            .iter()
            .filter(|o| matches!(o, Order::BuyItem { .. }))
            .collect();
        assert_eq!(purchases.len(), expected, "{orders:?}");
        assert!(purchases.iter().all(|o| matches!(
            o,
            Order::BuyItem {
                item: ProductionItem::Unit {
                    unit: UnitKind::Spearman
                },
                ..
            }
        )));
        assert!(100 * purchases.len() as i32 <= gold);
        assert!(
            !orders
                .iter()
                .any(|o| matches!(o, Order::DeclareWar { .. } | Order::RangedAttack { .. }))
        );
    }
    view.diplomacy.actions[0].blocked_reason = Some("Protected treaty".into());
    let purchases = |difficulty| {
        plan_without_hold(&view, difficulty)
            .into_iter()
            .filter(|o| matches!(o, Order::BuyItem { .. }))
            .collect::<Vec<_>>()
    };
    // Ordinary army spending can still buy one unit above the smaller reserve;
    // the protected treaty suppresses the extra readiness purchase.
    let protected = purchases(AiDifficulty::Hard);
    assert_eq!(protected.len(), 1, "{protected:?}");
    assert_eq!(protected, purchases(AiDifficulty::Normal));
    view.diplomacy.actions[0].blocked_reason = None;
    assert_eq!(
        plan_without_hold(&view, AiDifficulty::Normal)
            .iter()
            .filter(|o| matches!(o, Order::BuyItem { .. }))
            .count(),
        1
    );
}

#[test]
fn dispatch_reserves_upkeep_across_two_readiness_purchases() {
    let mut view = ViewBuilder::new(36, 20)
        .city(1, ME, Hex::new(6, 6), 3)
        .city(2, ME, Hex::new(16, 6), 3)
        .unit(1, ME, UnitKind::Spearman, Hex::new(3, 14))
        .unit(2, ME, UnitKind::Spearman, Hex::new(4, 14))
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
        .unit(51, ENEMY, UnitKind::Archer, Hex::new(10, 5))
        .unit(52, ENEMY, UnitKind::Spearman, Hex::new(20, 6))
        .unit(53, ENEMY, UnitKind::Archer, Hex::new(20, 5))
        .build();
    relation(&mut view, RelationState::Peace);
    action(
        &mut view,
        Order::DeclareWar {
            against: TeamId(2),
            breach: false,
        },
    );
    view.economy.gold = 420;
    view.economy.gold_per_turn.total = 1;
    for city in &mut view.cities {
        let detail = city.detail.as_mut().unwrap();
        detail.producing = Some(ProductionItem::Building {
            building: BuildingKind::Market,
        });
        detail.options.retain(|o| {
            o.item
                == ProductionItem::Unit {
                    unit: UnitKind::Spearman,
                }
        });
        detail.options[0].buy_cost = Some(200);
    }
    let orders = plan(&view, AiDifficulty::Hard);
    assert_eq!(
        orders
            .iter()
            .filter(|o| matches!(o, Order::BuyItem { .. }))
            .count(),
        1,
        "planned purchases must not exceed next turn's upkeep: {orders:?}"
    );
}

#[test]
fn dispatch_keeps_a_surviving_melee_garrison_against_an_immediate_capture() {
    let home = Hex::new(6, 6);
    let view = ViewBuilder::new(24, 16)
        .city(1, ME, home, 3)
        .unit(1, ME, UnitKind::Pikeman, home)
        .unit(2, ME, UnitKind::Pikeman, Hex::new(5, 6))
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(8, 6))
        .unit(51, ENEMY, UnitKind::Archer, Hex::new(9, 6))
        .build();
    let orders = plan(&view, AiDifficulty::Hard);
    assert!(
        orders.contains(&Order::SkipUnit { unit: UnitId(1) }),
        "{orders:?}"
    );
    assert_eq!(
        orders
            .iter()
            .filter(|o| legatum_core::rules::units::order_unit(o) == Some(UnitId(1)))
            .count(),
        1
    );
}

#[test]
fn dispatch_does_not_send_a_ranged_unit_to_capture_an_empty_city() {
    let target = Hex::new(10, 8);
    let mut view = ViewBuilder::new(26, 18)
        .city(1, ME, Hex::new(5, 8), 3)
        .city(9, ENEMY, target, 3)
        .unit(1, ME, UnitKind::Archer, Hex::new(9, 8))
        .build();
    view.cities
        .iter_mut()
        .find(|c| c.owner == ENEMY)
        .unwrap()
        .hp = 0;
    let orders = plan(&view, AiDifficulty::Hard);
    assert!(!orders.iter().any(|o| matches!(o, Order::MoveUnit { unit: UnitId(1), to } | Order::MoveUnitPath { unit: UnitId(1), to, .. } if *to == target)), "{orders:?}");
}

#[test]
fn settler_takes_a_safe_route_away_from_war_without_an_escort() {
    let home = Hex::new(6, 6);
    let view = ViewBuilder::new(28, 20)
        .city(1, ME, home, 3)
        .unit(1, ME, UnitKind::Settler, home)
        .unit(2, ME, UnitKind::Spearman, home)
        .unit(3, ME, UnitKind::Spearman, Hex::new(5, 6))
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
        .build();
    assert!(view.at_war(ME, ENEMY));
    let mut params = crate::AiParams::builtin();
    params.decisions.settler_safety = 1;
    let orders =
        ScriptedBrain::with_params(view.faction_of(ME), 7, ME, AiDifficulty::Hard, &params)
            .plan(&view);
    assert!(
        orders.iter().any(|o| matches!(o,
            Order::MoveUnit { unit: UnitId(1), to }
                | Order::MoveUnitPath { unit: UnitId(1), to, .. }
                if *to == Hex::new(6, 2)
        )),
        "{orders:?}"
    );
    assert!(!orders.iter().any(|o| matches!(o, Order::MoveUnit { unit: UnitId(3), to } | Order::MoveUnitPath { unit: UnitId(3), to, .. } if *to == home)), "{orders:?}");
}

#[test]
fn dispatch_keeps_the_settler_moving_when_a_garrison_blocks_its_escort() {
    let at = Hex::new(8, 6);
    let mut view = ViewBuilder::new(28, 20)
        .city(1, ME, Hex::new(4, 4), 3)
        .unit(1, ME, UnitKind::Settler, at)
        .unit(2, ME, UnitKind::Spearman, at)
        .unit(3, ME, UnitKind::Spearman, Hex::new(8, 5))
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(12, 6))
        .build();
    view.failed_city_sites.insert(at);
    let orders = plan(&view, AiDifficulty::Hard);
    assert!(
        orders.iter().any(|o| matches!(
            o,
            Order::MoveUnit {
                unit: UnitId(1),
                ..
            } | Order::MoveUnitPath {
                unit: UnitId(1),
                ..
            }
        )),
        "{orders:?}"
    );
}

#[test]
fn dispatch_preserves_the_escorted_settlers_founding_destination() {
    let at = Hex::new(8, 6);
    let mut view = ViewBuilder::new(28, 20)
        .city(1, ME, Hex::new(4, 4), 3)
        .unit(1, ME, UnitKind::Settler, at)
        .unit(2, ME, UnitKind::Spearman, at)
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(12, 6))
        .build();
    view.failed_city_sites.insert(at);
    let orders = plan(&view, AiDifficulty::Hard);
    let destination = orders
        .iter()
        .find_map(|o| match o {
            Order::MoveUnit {
                unit: UnitId(1),
                to,
            }
            | Order::MoveUnitPath {
                unit: UnitId(1),
                to,
                ..
            } => Some(*to),
            _ => None,
        })
        .unwrap_or_else(|| panic!("settler should keep a founding goal: {orders:?}"));
    assert!(at.distance(destination) > 1, "{orders:?}");
    assert!(
        orders.iter().any(|o| matches!(
            o,
            Order::MoveUnit {
                unit: UnitId(2),
                ..
            } | Order::MoveUnitPath {
                unit: UnitId(2),
                ..
            }
        )),
        "{orders:?}"
    );
}

#[test]
fn dispatch_maintains_containment_but_allows_peace_when_home_is_overrun() {
    let mut view = ViewBuilder::new(28, 20)
        .city(1, ME, off(4, 6), 3)
        .unit(1, ME, UnitKind::Spearman, off(4, 6))
        .build();
    relation(&mut view, RelationState::War);
    view.standings = vec![
        TeamStanding {
            team: view.team,
            members: vec![ME, PlayerId(1)],
            vp: 4,
            target: 20,
            victory_hold: None,
        },
        TeamStanding {
            team: TeamId(2),
            members: vec![ENEMY],
            vp: 8,
            target: 10,
            victory_hold: None,
        },
    ]
    .into();
    crate::fixture::set_armies(&mut view, &[(ME, 20), (ENEMY, 200)]);
    let peace = Order::ProposeTreaty {
        to: TeamId(2),
        kind: TreatyKind::Peace,
    };
    action(&mut view, peace.clone());
    view.turn = (10..50)
        .find(|turn| {
            let mut at = view.clone();
            at.turn = *turn;
            crate::offer_slot_open(&at, TeamId(2))
        })
        .unwrap();
    let normal = plan_without_runaway(&view, AiDifficulty::Normal);
    assert!(normal.contains(&peace), "{normal:?}");
    assert!(!plan_without_runaway(&view, AiDifficulty::Hard).contains(&peace));
    let overrun = ViewBuilder { view }
        .unit(50, ENEMY, UnitKind::Pikeman, off(7, 6))
        .unit(51, ENEMY, UnitKind::Pikeman, off(7, 5))
        .unit(52, ENEMY, UnitKind::Pikeman, off(7, 7))
        .build();
    let orders = plan_without_runaway(&overrun, AiDifficulty::Hard);
    assert!(orders.contains(&peace), "{orders:?}");
}

#[test]
fn dispatch_requires_ranged_support_before_declaring_an_optional_siege() {
    let mut view = ViewBuilder::new(28, 20)
        .city(1, ME, off(4, 6), 3)
        .city(2, ENEMY, off(11, 6), 3)
        .walled(2)
        .unit(1, ME, UnitKind::Phalangite, off(4, 6))
        .unit(2, ME, UnitKind::Phalangite, off(8, 6))
        .unit(3, ME, UnitKind::Phalangite, off(8, 5))
        .unit(4, ME, UnitKind::Phalangite, off(8, 4))
        .build();
    // Early assessment intentionally avoids Walls even for Normal. Test siege
    // readiness after that opening caution has expired.
    view.turn = 200;
    relation(&mut view, RelationState::Peace);
    let declaration = Order::DeclareWar {
        against: TeamId(2),
        breach: false,
    };
    action(&mut view, declaration.clone());
    let declares = |view: &PlayerView, difficulty| {
        let mut brain = crate::fixture::brain(view, 7, difficulty);
        brain.knobs = crate::Knobs::new(50, 80, 50, 50, 100);
        brain.base = brain.knobs;
        let orders = brain.plan(view);
        assert_valid(view, &orders);
        orders.contains(&declaration)
    };
    assert!(declares(&view, AiDifficulty::Normal));
    assert!(!declares(&view, AiDifficulty::Hard));
    let supported = ViewBuilder { view }
        .unit(5, ME, UnitKind::Archer, off(8, 7))
        .unit(6, ME, UnitKind::Archer, off(9, 7))
        .unit(7, ME, UnitKind::Archer, off(9, 6))
        .unit(8, ME, UnitKind::Archer, off(9, 5))
        .unit(9, ME, UnitKind::Archer, off(8, 8))
        .build();
    assert!(declares(&supported, AiDifficulty::Hard));
}

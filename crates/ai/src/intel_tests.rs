//! Early warning from sentries, the value of a fresh sighting, and composition counters.

use crate::Brain;
use crate::fixture::{ENEMY, ME, ViewBuilder};
use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
use legatum_core::{
    AiDifficulty, Hex, Order, PlayerView, ProductionItem, TeamId, UnitId, UnitKind,
};

fn approach() -> PlayerView {
    let mut view = ViewBuilder::new(32, 16)
        .city(1, ME, Hex::new(6, 6), 3)
        .city(2, ENEMY, Hex::new(20, 6), 3)
        .unit(1, ME, UnitKind::Scout, Hex::new(9, 6))
        .unit(2, ME, UnitKind::Spearman, Hex::new(5, 6))
        .terrain(Hex::new(11, 6), legatum_core::Terrain::Hills)
        .build();
    view.players[ME.0 as usize].cities = 1;
    view.players[ENEMY.0 as usize].cities = 1;
    crate::fixture::set_armies(&mut view, &[(ME, 20), (ENEMY, 200)]);
    view.diplomacy.legacy = false;
    view.diplomacy.relations.push(Relation {
        a: TeamId(0),
        b: TeamId(2),
        state: RelationState::Peace,
        effective_turn: 1,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: None,
        declarers: vec![],
    });
    view.diplomacy.actions.push(DiplomaticAction {
        new_wars: vec![],
        calls: vec![],
        also_agree: vec![],
        order: Order::DeclareWar {
            against: TeamId(2),
            breach: false,
        },
        effective_turn: view.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    });
    view
}

#[test]
fn sentry_sees_four_turn_approach_and_triggers_early_defense() {
    let hidden = approach();
    let mut brain = crate::fixture::brain(&hidden, 7, AiDifficulty::Hard);
    let orders = brain.plan(&hidden);
    let post = orders
        .iter()
        .find_map(|o| match o {
            Order::MoveUnit {
                unit: UnitId(1),
                to,
            } => Some(*to),
            _ => None,
        })
        .expect("a Scout should be posted on the high-value approach");
    let marching = Hex::new(14, 6);
    assert!(
        post.distance(marching)
            <= legatum_core::data::UNIT_SIGHT
                + i32::from(
                    hidden.tile(post).and_then(|t| t.terrain) == Some(legatum_core::Terrain::Hills)
                ),
        "the sentry post must see the army at least four turns away: {post:?}"
    );
    assert!(Hex::new(6, 6).distance(marching) >= 4 * 2);

    let mut seen = ViewBuilder {
        view: hidden.clone(),
    }
    .unit(50, ENEMY, UnitKind::Archer, marching)
    .unit(51, ENEMY, UnitKind::Archer, Hex::new(14, 5))
    .unit(52, ENEMY, UnitKind::Archer, Hex::new(14, 7))
    .build();
    seen.units
        .iter_mut()
        .find(|u| u.id == UnitId(1))
        .unwrap()
        .pos = post;
    let mut p = crate::fixture::planner(&seen, 7);
    p.difficulty = AiDifficulty::Hard;
    p.prepare_hard_defense();
    assert!(
        p.ctx.threat[0] > 0,
        "fresh sight of a four-turn approach must raise readiness"
    );
    let alert = crate::fixture::brain(&seen, 7, AiDifficulty::Hard).plan(&seen);
    assert!(
        alert.iter().any(|o| matches!(
            o,
            Order::SetProduction {
                city: legatum_core::CityId(1),
                item: ProductionItem::Building {
                    building: legatum_core::BuildingKind::Walls
                } | ProductionItem::Unit {
                    unit: UnitKind::Archer
                }
            }
        )),
        "early sight should start Walls or a shooter: {alert:?}"
    );
    let mut blind = hidden.clone();
    blind.units.retain(|u| u.id != UnitId(1));
    let mut p = crate::fixture::planner(&blind, 7);
    p.difficulty = AiDifficulty::Hard;
    p.prepare_hard_defense();
    assert_eq!(
        p.ctx.threat[0], 0,
        "without a sentry this army remains unseen"
    );
    let mut no_new_sentry = crate::fixture::brain(&hidden, 7, AiDifficulty::Hard);
    no_new_sentry.decisions.sentry_posts = 0;
    no_new_sentry.decisions.sentry = 0;
    let old_orders = no_new_sentry.plan(&hidden);
    assert!(
        !old_orders.iter().any(|o| matches!(
            o,
            Order::MoveUnit {
                unit: UnitId(1),
                ..
            }
        )),
        "zero sentry genes keep the approach unwatched: {old_orders:?}"
    );
    let mut no_warning = crate::fixture::planner(&seen, 7);
    no_warning.difficulty = AiDifficulty::Hard;
    no_warning.decisions.intel_value = 0;
    no_warning.prepare_hard_defense();
    assert_eq!(
        no_warning.ctx.threat[0], 0,
        "zero information value restores the two-turn emergency window"
    );
}

#[test]
fn stale_army_sightings_have_more_information_value_than_fresh_sight() {
    use legatum_core::state::ThreatObservation;
    let mut stale = approach();
    stale.observed_threats.push(ThreatObservation {
        unit: UnitId(50),
        owner: ENEMY,
        kind: UnitKind::Archer,
        pos: Hex::new(14, 6),
        seen_turn: stale.turn - 7,
    });
    let mut fresh = stale.clone();
    fresh.observed_threats[0].seen_turn = fresh.turn;
    let old = crate::fixture::planner(&stale, 7);
    let now = crate::fixture::planner(&fresh, 7);
    let mut old = old;
    let mut now = now;
    old.difficulty = AiDifficulty::Hard;
    now.difficulty = AiDifficulty::Hard;
    assert!(old.information_value(ENEMY) > now.information_value(ENEMY));
    assert!(now.information_value(ENEMY) > 0);
    old.decisions.intel_value = 0;
    assert_eq!(old.information_value(ENEMY), 0);
    let orders = crate::fixture::brain(&stale, 7, AiDifficulty::Hard).plan(&stale);
    assert!(
        orders.iter().any(|o| matches!(o,
        Order::MoveUnit { unit: UnitId(1), to }
            if to.distance(Hex::new(14, 6)) <= legatum_core::data::UNIT_SIGHT
                + i32::from(stale.tile(*to).and_then(|t| t.terrain)
                    == Some(legatum_core::Terrain::Hills)))),
        "a stale sighting should send the sentry to re-observe the last known approach: {orders:?}"
    );
}

#[test]
fn observed_archers_call_for_cavalry_and_observed_cavalry_for_spears() {
    let mut view = approach();
    view.cities[0].walls = true;
    for (id, at) in [
        (50, Hex::new(14, 6)),
        (51, Hex::new(14, 5)),
        (52, Hex::new(14, 7)),
    ] {
        view = ViewBuilder { view }
            .unit(id, ENEMY, UnitKind::Archer, at)
            .build();
    }
    let archer_orders = crate::fixture::brain(&view, 7, AiDifficulty::Hard).plan(&view);
    assert!(
        archer_orders.contains(&Order::SetProduction {
            city: legatum_core::CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Horseman
            },
        }),
        "archers should raise cavalry's production value: {archer_orders:?}"
    );
    let mut valuation = crate::fixture::planner(&view, 7);
    valuation.difficulty = AiDifficulty::Hard;
    assert!(valuation.early_counter_value(valuation.ctx.own_cities[0], UnitKind::Horseman) > 0);
    valuation.decisions.counter_composition = 0;
    assert_eq!(
        valuation.early_counter_value(valuation.ctx.own_cities[0], UnitKind::Horseman),
        0
    );
    for u in view.units.iter_mut().filter(|u| u.owner == ENEMY) {
        u.kind = UnitKind::Horseman;
        let stats = legatum_core::data::unit_stats(u.kind);
        u.strength = stats.strength;
        u.ranged = stats.ranged;
        u.range = stats.range;
        u.max_moves = stats.moves;
        u.moves_left = stats.moves;
    }
    let mounted_orders = crate::fixture::brain(&view, 7, AiDifficulty::Hard).plan(&view);
    assert!(
        mounted_orders.contains(&Order::SetProduction {
            city: legatum_core::CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman
            },
        }),
        "cavalry should raise spear production value: {mounted_orders:?}"
    );
}

#[test]
fn held_shot_stepin_keeps_a_doomed_unit_out_of_its_fallen_city() {
    let mut view = ViewBuilder::new(32, 16)
        .city(1, ME, Hex::new(6, 6), 3)
        .unit(1, ME, UnitKind::Horseman, Hex::new(7, 6))
        .unit(2, ENEMY, UnitKind::Archer, Hex::new(8, 6))
        .unit(3, ENEMY, UnitKind::Archer, Hex::new(8, 7))
        .unit(4, ENEMY, UnitKind::Archer, Hex::new(8, 5))
        .unit(5, ENEMY, UnitKind::Horseman, Hex::new(9, 6))
        .unit(6, ENEMY, UnitKind::Horseman, Hex::new(9, 7))
        .build();
    view.cities.iter_mut().find(|c| c.id.0 == 1).unwrap().hp = 0;
    let city = view.cities.iter().find(|c| c.id.0 == 1).unwrap().clone();
    let u = view.units.iter().find(|u| u.id == UnitId(1)).unwrap();
    let mut p = crate::fixture::planner(&view, 7);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.held_shot_stepin = 1;
    assert!(
        !p.can_defend_at(u, city.pos),
        "the fixture must be a death trap"
    );
    assert!(!p.may_close_into(u, &city, city.pos));
    assert!(
        p.may_close_into(u, &city, Hex::new(7, 6)),
        "other tiles stay open"
    );
    p.decisions.held_shot_stepin = 0;
    assert!(p.may_close_into(u, &city, city.pos), "gene off");
}

/// `approach()` with the land beyond column 16 unexplored and every explored tile more than
/// one hex from home turned to Desert (the sentry hill stays), so no good city site is known.
fn approach_without_sites() -> PlayerView {
    let mut view = ViewBuilder { view: approach() }
        .unexplored_from_col(16)
        .build();
    let home = Hex::new(6, 6);
    for i in 0..view.tiles.len() {
        let h = Hex::from_offset(i as i32 % view.width, i as i32 / view.width);
        let t = &mut view.tiles[i];
        if t.explored && h.distance(home) > 1 && h != Hex::new(11, 6) {
            t.terrain = Some(legatum_core::Terrain::Desert);
        }
    }
    view
}

/// Gene `sentry_needs_sites` (Mongols 8.9.73.1 review): the Swiss sent their only Scout to
/// stand by his capital on T7 and kept it there to T63; nobody else explored, no city site was
/// ever seen and no Settler was ever trained. A small empire that knows no good site sends its
/// Scout to the frontier first; once a site is known, the sentry goes back on duty.
#[test]
fn a_scout_explores_for_sites_before_standing_sentry() {
    let barren = approach_without_sites();
    let fertile = ViewBuilder { view: approach() }
        .unexplored_from_col(16)
        .build();
    let scout_move = |view: &PlayerView, needs_sites: i32| {
        let mut b = crate::fixture::brain(view, 7, AiDifficulty::Hard);
        b.decisions.sentry_needs_sites = needs_sites;
        b.plan(view).into_iter().find_map(|o| match o {
            Order::MoveUnit {
                unit: UnitId(1),
                to,
            } => Some(to),
            _ => None,
        })
    };
    let mut p = crate::fixture::planner(&barren, 7);
    p.difficulty = AiDifficulty::Hard;
    assert_eq!(
        p.count_good_sites(1),
        0,
        "the fixture must know no good site"
    );
    let post = scout_move(&barren, 0).expect("with the gene off the Scout takes a sentry post");
    let explore = scout_move(&barren, 1).expect("with no known site the Scout explores");
    assert_ne!(explore, post, "the Scout should not take the sentry post");
    // With good sites known, the sentry stands as before.
    assert_eq!(scout_move(&fertile, 1), scout_move(&fertile, 0));
}

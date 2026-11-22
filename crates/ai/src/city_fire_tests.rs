use super::*;
use crate::Brain;
use crate::fixture::{ENEMY, ME, ViewBuilder};
use legatum_core::{AiDifficulty, PlayerView, UnitId};

const CITY: Hex = Hex::new(10, 8);
const EDGE: Hex = Hex::new(8, 8);

fn view(hp: i32) -> PlayerView {
    ViewBuilder::new(26, 18)
        .city(1, ME, Hex::new(3, 8), 2)
        .city(9, ENEMY, CITY, 3)
        .walled(9)
        .unit_hp(1, ME, UnitKind::Spearman, EDGE, hp)
        .build()
}

fn planner(view: &PlayerView) -> Planner<'_> {
    crate::fixture::planner(view, 7)
}

#[test]
fn default_and_every_difficulty_retreat_before_city_fire_makes_escape_fatal() {
    let view = view(55);
    for difficulty in [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard] {
        let mut brain = crate::fixture::brain(&view, 7, difficulty);
        let orders = brain.orders(&view);
        assert!(orders.iter().any(|o| matches!(o,
            Order::MoveUnit { unit: UnitId(1), to } | Order::MoveUnitPath { unit: UnitId(1), to, .. }
                if to.distance(CITY) > 2)), "{difficulty:?}: {orders:?}");
        assert_eq!(
            orders,
            brain.orders(&view),
            "stateless, deterministic decisions"
        );
    }
}

#[test]
fn danger_includes_known_city_fire_without_a_garrison_but_not_dead_or_peaceful_cities() {
    let mut view = view(100);
    // City knowledge is public even when its tile is outside current vision.
    view.cities[1].visible = false;
    let p = planner(&view);
    assert!(p.ctx.in_danger(EDGE));
    assert!(p.exposure_at(EDGE) > 0);
    assert!(p.city_damage_at(p.ctx.own_units[0], EDGE) > 0);
    assert_eq!(p.exposure_at(Hex::new(7, 8)), 0);
    view.cities[1].hp = 0;
    assert_eq!(planner(&view).exposure_at(EDGE), 0);
    view.cities[1].hp = 1;
    assert!(planner(&view).exposure_at(EDGE) > 0);
    // Only walled cities fire: without Walls the same city is no danger.
    view.cities[1].walls = false;
    assert_eq!(planner(&view).exposure_at(EDGE), 0);
    assert!(!planner(&view).ctx.in_danger(EDGE));
    view.cities[1].walls = true;
    view.diplomacy.legacy = false;
    assert_eq!(planner(&view).exposure_at(EDGE), 0);
}

#[test]
fn city_fire_exposure_ignores_forest_and_hills_between_city_and_unit() {
    for blocker in [legatum_core::Terrain::Forest, legatum_core::Terrain::Hills] {
        let view = ViewBuilder { view: view(100) }
            .terrain(Hex::new(9, 8), blocker)
            .build();
        let p = planner(&view);
        assert!(p.ctx.in_danger(EDGE), "{blocker:?}");
        assert!(p.exposure_at(EDGE) > 0, "{blocker:?}");
        assert!(
            p.city_damage_at(p.ctx.own_units[0], EDGE) > 0,
            "{blocker:?}"
        );
    }
}

#[test]
fn city_shot_is_charged_once_to_lowest_hp_then_id_and_forecasts_recompute_wounded_melee() {
    let view = ViewBuilder { view: view(65) }
        .unit_hp(2, ME, UnitKind::Spearman, Hex::new(9, 7), 65)
        .build();
    let p = planner(&view);
    let first = p.ctx.own_units[0];
    let second = p.ctx.own_units[1];
    assert!(p.city_damage_now(first) > 0);
    assert_eq!(p.city_damage_now(second), 0);
    let before = forecast_attack_mode(&view, first.id, CITY, AttackMode::Melee).unwrap();
    assert!(before.city_strike.is_some());
    let after = p.melee_forecast(first, CITY).unwrap();
    assert_eq!(
        after.attacker_hp,
        first.hp - p.attack_city_damage(first, &before)
    );
    assert!(after.loss_chance >= before.loss_chance);
    let mut reversed = view.clone();
    reversed.units.reverse();
    assert_eq!(
        planner(&reversed).city_damage_now(first),
        p.city_damage_now(first)
    );
}

#[test]
fn staging_pays_one_shot_per_city_and_uses_arrival_terrain_without_fortification() {
    let mut view = ViewBuilder::new(26, 18)
        .city(9, ENEMY, CITY, 1)
        .walled(9)
        .unit(1, ME, UnitKind::Spearman, Hex::new(7, 8))
        .unit(2, ME, UnitKind::Spearman, Hex::new(7, 9))
        .build();
    view.units[0].fortify = 6;
    let p = planner(&view);
    let arrivals = [
        (p.ctx.own_units[0], EDGE),
        (p.ctx.own_units[1], Hex::new(9, 7)),
    ];
    let shot = forecast_city_strike(&view, &view.cities[0], arrivals[0].0, EDGE).unwrap();
    let mut wounded = arrivals[0].0.clone();
    wounded.hp -= shot.expected_damage(wounded.hp);
    let expected = offense_power(&view, &wounded) + offense_power(&view, arrivals[1].0);
    assert!(wounded.hp >= MARCH_MIN_HP);
    assert_eq!(p.assault_force_after_exposure(&arrivals), expected);
    let bare = shot.damage;
    let i = view.index(EDGE).unwrap();
    view.tiles[i].terrain = Some(Terrain::Hills);
    assert!(
        forecast_city_strike(&view, &view.cities[0], &view.units[0], EDGE)
            .unwrap()
            .damage
            < bare
    );
}

#[test]
fn a_wounded_unit_heals_outside_the_ring_and_an_undefended_zero_hp_city_is_still_capturable() {
    let mut view = view(35);
    view.units[0].pos = Hex::new(7, 8);
    let mut p = planner(&view);
    let army = p.ctx.own_units.clone();
    p.recover_wounded(&army);
    assert!(
        p.orders
            .iter()
            .any(|o| matches!(o, Order::SkipUnit { unit: UnitId(1) }))
    );
    view.units[0].pos = EDGE;
    view.cities[1].hp = 0;
    assert!(!planner(&view).too_risky(&view.units[0], CITY));
}

#[test]
fn proxy_geography_stays_frozen_for_the_paired_benchmark() {
    let view = view(100);
    assert!(crate::analysis::Ctx::new(&view).in_danger(EDGE));
    assert!(!crate::analysis::Ctx::for_proxy(&view).in_danger(EDGE));
}

#[test]
fn default_credits_committed_volley_but_a_defeated_city_still_fires_in_that_phase() {
    let mut view = ViewBuilder { view: view(80) }
        .unit(2, ME, UnitKind::Archer, Hex::new(9, 7))
        .unit(3, ME, UnitKind::Archer, Hex::new(10, 6))
        .build();
    // Walls add strength against the volley and halve its damage: leave less HP for it to finish.
    view.cities[1].hp = 12;
    let mut p = planner(&view);
    for unit in [UnitId(2), UnitId(3)] {
        p.orders.push(Order::RangedAttack { unit, target: CITY });
    }
    assert!(
        p.opening.is_none(),
        "exercise Default, not an opening policy"
    );
    assert!(p.committed_fire_support(&p.ctx.own_units, CITY) > 0);
    let f = p.melee_forecast(p.ctx.own_units[0], CITY).unwrap();
    assert_eq!(f.defender_hp, 0, "the guaranteed volley defeats the city");
    assert_eq!(
        f.expected_damage_to_attacker, 0,
        "no melee retaliation at zero city HP"
    );
    assert!(
        f.attacker_hp < 80,
        "the simultaneous city shot still happens before capture"
    );
    assert!(f.city_strike.is_some());
    let mut coordinated = planner(&view);
    coordinated.plan_military();
    assert!(
        coordinated.orders.iter().any(|o| matches!(o,
        Order::MoveUnit { unit: UnitId(1), to } if *to == CITY)),
        "{:?}",
        coordinated.orders
    );
}

#[test]
fn archers_wear_down_a_garrisoned_city_and_shoot_its_garrison_once_it_falls() {
    // Unwalled, so the city cannot fire back. Two Archers in range of a garrisoned city.
    let mut view = ViewBuilder::new(26, 18)
        .city(1, ME, Hex::new(3, 8), 2)
        .city(9, ENEMY, CITY, 3)
        .unit(1, ME, UnitKind::Archer, EDGE)
        .unit(2, ME, UnitKind::Archer, Hex::new(9, 7))
        .unit(50, ENEMY, UnitKind::Spearman, CITY)
        .build();
    let shots = |view: &PlayerView| {
        let mut p = planner(view);
        let army = p.ctx.own_units.clone();
        p.ranged_fire(&army);
        p.orders
    };
    let fire = |unit| Order::RangedAttack {
        unit: UnitId(unit),
        target: CITY,
    };
    // The city shields its garrison but takes the shots: no garrison skips the city.
    assert_eq!(shots(&view), vec![fire(1), fire(2)]);
    // A nearly broken city takes one shot; its excess can reach the garrison.
    view.cities[1].hp = 3;
    assert_eq!(shots(&view), vec![fire(1)]);
    // A fallen city's garrison is an ordinary target.
    view.cities[1].hp = 0;
    assert_eq!(shots(&view), vec![fire(1), fire(2)]);
}

#[test]
fn committed_fire_hits_the_city_first_then_spills_into_its_garrison() {
    let mut view = ViewBuilder { view: view(100) }
        .unit(2, ME, UnitKind::Crossbowman, Hex::new(9, 7))
        .unit_hp(50, ENEMY, UnitKind::Spearman, CITY, 1)
        .build();
    let volley = |view: &PlayerView| {
        let shot = forecast_attack_mode(view, UnitId(2), CITY, AttackMode::Ranged).unwrap();
        assert!(shot.garrison.is_some_and(|g| g.shielded));
        legatum_core::combat::damage(
            shot.attacker_strength.total,
            shot.defender_strength.total,
            legatum_core::combat::ROLL_MIN,
        )
    };
    let forecast = |view: &PlayerView| {
        let mut p = planner(view);
        p.orders.push(Order::RangedAttack {
            unit: UnitId(2),
            target: CITY,
        });
        p.melee_forecast(p.ctx.own_units[0], CITY).unwrap()
    };
    // The city absorbs the volley; its 1-HP garrison is untouched and still shielded.
    let f = forecast(&view);
    assert_eq!(f.defender_hp, view.cities[1].hp - volley(&view));
    assert!(f.garrison.is_some_and(|g| g.shielded && g.hp == 1));
    // A volley that breaks the city kills the 1-HP garrison through overflow.
    view.cities[1].hp = 5;
    assert!(volley(&view) > 5);
    let f = forecast(&view);
    assert_eq!(f.defender_hp, 0);
    assert!(f.garrison.is_none());
}

#[test]
fn queued_melee_ally_reduces_projected_counterattack_cost() {
    let view = ViewBuilder { view: view(100) }
        .unit(2, ME, UnitKind::Spearman, Hex::new(9, 7))
        .build();
    let mut p = planner(&view);
    let attacker = p.ctx.own_units[0];
    let alone = p.melee_forecast(attacker, CITY).unwrap();
    p.orders.push(Order::MoveUnit {
        unit: UnitId(2),
        to: CITY,
    });
    let together = p.melee_forecast(attacker, CITY).unwrap();
    assert!(together.expected_damage_to_attacker < alone.expected_damage_to_attacker);
    assert_eq!(
        together.expected_damage_to_defender,
        alone.expected_damage_to_defender
    );
}

use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid, off};
use crate::{Brain, Knobs, ScriptedBrain};
use legatum_core::AiDifficulty;
use legatum_core::view::PlayerView;
use legatum_core::{CityId, PlayerId, UnitId};

const FOUNDRY: ProductionItem = ProductionItem::Building {
    building: BuildingKind::Forge,
};
const HEIR: ProductionItem = ProductionItem::Unit {
    unit: UnitKind::HeirToTheThrone,
};

fn planner(v: &PlayerView) -> Planner<'_> {
    Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 7)
}

fn brain(v: &PlayerView) -> Vec<Order> {
    let out = ScriptedBrain::with_difficulty(v.faction_of(ME), 7, ME, AiDifficulty::Hard).orders(v);
    assert_valid(v, &out);
    out
}

/// A Germany with `n` cities, every one but the last holding a Foundry.
fn germany(n: u32, last_has_factory: bool) -> PlayerView {
    let mut b = ViewBuilder::new(40, 24)
        .faction(Faction::Germany)
        .known_techs(&[legatum_core::TechId::IronWorking]);
    for i in 0..n {
        b = b.city(
            i + 1,
            ME,
            off(4 + 6 * (i as i32 % 4), 4 + 6 * (i as i32 / 4)),
            4,
        );
        b = b.unit(
            i + 1,
            ME,
            UnitKind::Spearman,
            off(4 + 6 * (i as i32 % 4), 4 + 6 * (i as i32 / 4)),
        );
    }
    let mut v = b.build();
    v.turn = 200;
    v.players[0].faction_goal_done = false;
    v.economy.gold = 5000;
    for resource in [
        legatum_core::resources::Resource::Iron,
        legatum_core::resources::Resource::Iron,
    ] {
        v.supply.push(legatum_core::resources::SupplyView {
            resource,
            revealed_by: None,
            revealed: true,
            supplied: Default::default(),
            deposits: vec![],
            used: Default::default(),
            idle: vec![],
            held: vec![],
            spare: 4,
        });
    }
    let last = n as usize - 1;
    for (i, c) in v.cities.iter_mut().enumerate() {
        let d = c.detail.as_mut().unwrap();
        d.buildings = vec![BuildingKind::Granary, BuildingKind::Workshop];
        for o in &mut d.options {
            if o.item == FOUNDRY {
                o.turns = 25;
            }
        }
        if i != last || last_has_factory {
            d.buildings.push(BuildingKind::Forge);
            d.options.retain(|o| o.item != FOUNDRY);
        }
    }
    v.players[0].cities = n;
    v
}

#[test]
fn factories_gain_goal_value_before_the_fifth_city() {
    let v = germany(3, false);
    let p = planner(&v);
    assert!(p.forge_open());
    assert_eq!(p.factory_goal_value(&v.cities[2], 40), 1000);
}

#[test]
fn the_last_missing_factory_is_worth_the_whole_goal_and_gets_started() {
    let v = germany(5, false);
    let p = planner(&v);
    assert!(p.forge_open());
    assert_eq!(p.factories_missing(), 1);
    assert!(p.factory_goal_value(&v.cities[4], 40) >= 1500 * 2);
    assert!(p.factory_completes_goal(&v.cities[4], BuildingKind::Forge));
    // Only that city can use it.
    assert_eq!(p.factory_goal_value(&v.cities[0], 40), 0);
    // Several missing: each receives a share, even if Iron Working arrives early.
    let mut many = germany(5, false);
    for c in &mut many.cities {
        let d = c.detail.as_mut().unwrap();
        d.buildings = vec![BuildingKind::Granary, BuildingKind::Workshop];
        for o in &mut d.options {
            if o.item == FOUNDRY {
                o.turns = 25;
            }
        }
    }
    many.turn = 10;
    let early = planner(&many);
    assert!(early.forge_open());
    assert_eq!(early.factory_goal_value(&many.cities[0], 40), 600);
    many.turn = 240;
    let late = planner(&many);
    assert!(late.forge_open());
    assert_eq!(late.factories_missing(), 5);
    assert!(late.factory_goal_value(&many.cities[0], 40) < p.factory_goal_value(&v.cities[4], 40));
    assert!(late.factory_goal_value(&many.cities[0], 40) >= 600);
    // The brain starts a Foundry in the city that lacks one.
    let orders = brain(&v);
    assert!(
        orders.contains(&Order::SetProduction {
            city: CityId(5),
            item: FOUNDRY
        }) || orders.contains(&Order::BuyItem {
            city: CityId(5),
            item: FOUNDRY
        }),
        "{orders:?}"
    );
}

#[test]
fn a_nearly_complete_forge_values_the_fifth_city_and_each_step() {
    let mut v = germany(4, true);
    v.economy.gold = 40;
    let poor = planner(&v);
    assert_eq!(poor.factory_goal_expansion_cost(), -3000);
    v.economy.gold = 5000;
    assert_eq!(planner(&v).factory_goal_expansion_cost(), -3000);
    // The value does not require spare Gold or supply before founding.
    v.supply
        .iter_mut()
        .find(|s| s.resource == legatum_core::resources::Resource::Iron)
        .unwrap()
        .spare = 0;
    assert_eq!(planner(&v).factory_goal_expansion_cost(), -3000);
    let last = v.cities.len() - 1;
    v.cities[last].detail.as_mut().unwrap().buildings = vec![BuildingKind::Granary];
    v.players[0].faction_goal_done = false;
    let p = planner(&v);
    assert_eq!(p.factory_goal_expansion_cost(), -2400);
    assert!(p.factory_goal_stage_value(&v.cities[last], BuildingKind::Workshop, 10) > 0);
    assert!(p.factory_goal_stage_value(&v.cities[last], BuildingKind::Forge, 10) > 0);

    assert!(p.factory_goal_supply_value(legatum_core::resources::Resource::Iron) > 0);
    // Earlier expansion gets a smaller benefit, proportional to held Foundries.
    let mut young = germany(3, false);
    young.economy.gold = 0;
    assert_eq!(planner(&young).factory_goal_expansion_cost(), -1800);
    let mut other = germany(4, true);
    other.players[0].faction = Faction::India;
    other.economy.gold = 0;
    assert_eq!(planner(&other).factory_goal_expansion_cost(), 0);
}

#[test]
fn a_sixth_city_does_not_create_another_foundry_obligation() {
    let v = germany(6, false);
    let p = planner(&v);
    assert_eq!(p.factory_goal_expansion_cost(), 0);
    assert_eq!(p.factory_goal_value(&v.cities[5], 40), 0);
    assert_eq!(
        p.factory_goal_stage_value(&v.cities[5], BuildingKind::Forge, 40),
        0
    );
    assert_eq!(
        p.factory_goal_supply_value(legatum_core::resources::Resource::Iron),
        0
    );
    assert!(!p.factory_completes_goal(&v.cities[5], BuildingKind::Forge));

    let mut four = v.clone();
    four.cities[4]
        .detail
        .as_mut()
        .unwrap()
        .buildings
        .retain(|b| *b != BuildingKind::Forge);
    let p = planner(&four);
    assert_eq!(p.factory_goal_value(&four.cities[5], 40), 3000);
    assert!(p.factory_completes_goal(&four.cities[5], BuildingKind::Forge));
}

#[test]
fn capturing_a_city_of_a_held_forge_with_five_cities_is_worth_the_goal() {
    let mut v = ViewBuilder::new(40, 24)
        .city(1, ME, off(4, 4), 3)
        .city(2, ENEMY, off(14, 4), 4)
        .city(3, ENEMY, off(20, 4), 4)
        .city(4, ENEMY, off(26, 4), 4)
        .city(5, ENEMY, off(32, 4), 4)
        .city(6, ENEMY, off(8, 12), 4)
        .build();
    let enemy = v.players.iter_mut().find(|p| p.id == ENEMY).unwrap();
    enemy.faction = Faction::Germany;
    enemy.cities = 5;
    enemy.vp_sources = vec![(VpSource::FactionGoal { of: ENEMY }, 2)];
    let p = planner(&v);
    assert!(p.forge_denial_value(&v.cities[1]) > 0);
    assert!(p.hoard_denial_value(&v.cities[1]) > 0);
    // A sixth city might have a spare Foundry: keep a discounted denial value.
    let mut spare = v.clone();
    spare
        .players
        .iter_mut()
        .find(|p| p.id == ENEMY)
        .unwrap()
        .cities = 6;
    let spare_value = planner(&spare).forge_denial_value(&spare.cities[1]);
    assert!(spare_value > 0 && spare_value < p.forge_denial_value(&v.cities[1]));
    // Not holding it: no denial.
    let mut idle = v.clone();
    idle.players
        .iter_mut()
        .find(|p| p.id == ENEMY)
        .unwrap()
        .vp_sources
        .clear();
    assert_eq!(planner(&idle).forge_denial_value(&idle.cities[1]), 0);
}

// ---------- the Heir ----------

fn byzantium(threatened: bool) -> PlayerView {
    let mut b = ViewBuilder::new(40, 24)
        .faction(Faction::Byzantium)
        .city(1, ME, off(4, 4), 4)
        .city(2, ME, off(10, 4), 4)
        .city(3, ME, off(4, 10), 4)
        .unit(1, ME, UnitKind::Spearman, off(4, 4))
        .unit(2, ME, UnitKind::Spearman, off(10, 4))
        .unit(3, ME, UnitKind::Spearman, off(4, 10));
    if threatened {
        b = b.unit(9, ENEMY, UnitKind::Pikeman, off(6, 4));
    }
    let mut v = b.build();
    v.turn = 60;
    v.players[0].faction_goal_done = false;
    v
}

#[test]
fn the_byzantium_trains_an_heir_when_it_has_cities_a_garrison_and_time() {
    let mut v = byzantium(false);
    for c in &mut v.cities {
        let d = c.detail.as_mut().unwrap();
        // The fixture's 4 Production would take the Heir twice as long as the cheap Granary is
        // worth waiting for; these Byzantium cities make 8, so he takes HEIR_COST / 8 turns.
        for o in &mut d.options {
            if o.item == HEIR {
                o.turns = (o.cost + 7) / 8;
            }
        }
        d.options.retain(|o| {
            o.item == HEIR
                || o.item
                    == ProductionItem::Building {
                        building: BuildingKind::Granary,
                    }
        });
    }
    assert!(planner(&v).heir_value(&v.cities[0], 40, false, false) > 0);
    // Pin the production comparison under test. The banked Byzantium personality now values
    // Granaries more highly in this position; this test is about the Heir's availability and
    // one-at-a-time assignment when the Heir wins that comparison.
    let heir_brain = |view: &PlayerView| {
        let mut b =
            ScriptedBrain::with_knobs(view.faction_of(ME), 7, ME, Knobs::new(55, 70, 75, 45, 45));
        b.difficulty = AiDifficulty::Hard;
        b.decisions = crate::DecisionParams::default();
        b
    };
    let orders = heir_brain(&v).orders(&v);
    let heir_orders = orders
        .iter()
        .filter(|o| matches!(o, Order::SetProduction { item, .. } if *item == HEIR))
        .count();
    assert_eq!(
        heir_orders, 1,
        "three cities saw the same pre-turn Heir availability; assign at most one: {orders:?}"
    );
    // Under threat, with too few cities, or too late, he waits.
    assert_eq!(planner(&v).heir_value(&v.cities[0], 40, true, false), 0);
    assert_eq!(planner(&v).heir_value(&v.cities[0], 40, false, true), 0);
    assert_eq!(planner(&v).heir_value(&v.cities[0], 500, false, false), 0);
    let mut small = v.clone();
    small.cities.truncate(2);
    assert_eq!(
        planner(&small).heir_value(&small.cities[0], 40, false, false),
        0
    );
    // A India never sees the option, and would value it at nothing.
    let mut other = v.clone();
    other.players[0].faction = Faction::India;
    assert_eq!(
        planner(&other).heir_value(&other.cities[0], 40, false, false),
        0
    );
    assert!(
        brain(&other)
            .iter()
            .all(|o| !matches!(o, Order::SetProduction { item, .. } if *item == HEIR)),
        "a faction-locked stale candidate must never be queued"
    );
}

fn with_heir(
    mut v: PlayerView,
    owner: PlayerId,
    of: PlayerId,
    pos: legatum_core::Hex,
) -> PlayerView {
    let b = ViewBuilder::new(40, 24)
        .unit(77, owner, UnitKind::HeirToTheThrone, pos)
        .build();
    let mut u = b.units.into_iter().next().unwrap();
    u.heir_of = Some(of);
    v.units.push(u);
    v
}

#[test]
fn the_heir_shelters_in_a_walled_garrisoned_city_and_never_wanders() {
    // In the field, two cities in reach: the walled, garrisoned one wins over the nearer.
    let mut v = byzantium(false);
    v.cities[1].walls = true;
    let v = with_heir(v, ME, ME, off(7, 4));
    let orders = brain(&v);
    assert!(
        orders.contains(&Order::MoveUnit {
            unit: UnitId(77),
            to: off(10, 4)
        }),
        "{orders:?}"
    );
    // Already sheltered: no order at all for him.
    let sheltered = with_heir(byzantium(false), ME, ME, off(4, 4));
    let orders = brain(&sheltered);
    assert!(!orders.iter().any(|o| matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. } if *unit == UnitId(77))), "{orders:?}");
    // A captured Heir of another empire is kept just as safe.
    let held = with_heir(byzantium(false), ME, ENEMY, off(7, 4));
    assert!(
        brain(&held)
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit, .. } if *unit == UnitId(77)))
    );
}

#[test]
fn heir_waits_in_shelter_when_the_refuge_route_crosses_an_interceptor() {
    let v = ViewBuilder::new(40, 24)
        .faction(Faction::Byzantium)
        .city(1, ME, off(4, 4), 4)
        .city(2, ME, off(12, 4), 4)
        .walled(2)
        .unit(1, ME, UnitKind::Spearman, off(4, 4))
        .unit(2, ME, UnitKind::Spearman, off(12, 4))
        .unit(90, ENEMY, UnitKind::Horseman, off(7, 5))
        .build();
    let v = with_heir(v, ME, ME, off(4, 4));
    let mut p = planner(&v);
    p.protect_heirs();
    assert!(
        !p.orders.iter().any(|o| matches!(o,
        Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. } if *unit == UnitId(77))),
        "the safer destination has an exposed journey: {:?}",
        p.orders
    );
}

#[test]
fn an_unguarded_heir_city_buys_a_garrison_and_walls_first() {
    let mut v = byzantium(false);
    v.units.retain(|u| u.pos != off(4, 4));
    let v = with_heir(v, ME, ME, off(4, 4));
    let p = planner(&v);
    assert!(p.heir_unguarded(&v.cities[0]));
    assert!(!p.heir_unguarded(&v.cities[1]));
}

#[test]
fn rivals_hunt_a_visible_heir_and_price_the_war_that_takes_it() {
    // A rival holds Byzantium's Heir beside our soldier: we capture him.
    let byzantium_id = PlayerId(3);
    let mut v = ViewBuilder::new(40, 24)
        .city(1, ME, off(4, 4), 4)
        .unit(2, ME, UnitKind::Spearman, off(4, 4))
        .unit(3, ME, UnitKind::Spearman, off(4, 5))
        .unit(1, ME, UnitKind::Spearman, off(8, 4))
        .build();
    v.players
        .iter_mut()
        .find(|p| p.id == byzantium_id)
        .unwrap()
        .faction = Faction::Byzantium;
    let v = with_heir(v, ENEMY, byzantium_id, off(9, 4));
    let p = planner(&v);
    let heir = v
        .units
        .iter()
        .find(|u| u.kind == UnitKind::HeirToTheThrone)
        .unwrap();
    assert!(p.heir_capture_value(heir) > 0);
    let orders = brain(&v);
    assert!(
        orders.contains(&Order::MoveUnit {
            unit: UnitId(1),
            to: off(9, 4)
        }),
        "{orders:?}"
    );
    // Our own Heir is not a target, and a team-mate's neither.
    let ours = with_heir(
        ViewBuilder::new(40, 24).city(1, ME, off(4, 4), 4).build(),
        ME,
        ME,
        off(4, 4),
    );
    assert_eq!(planner(&ours).heir_capture_value(&ours.units[0]), 0);
    // The holder's team is a juicier war target: public `vp_sources` names the held goal.
    let mut war = v.clone();
    war.players
        .iter_mut()
        .find(|p| p.id == ENEMY)
        .unwrap()
        .vp_sources = vec![(VpSource::FactionGoal { of: byzantium_id }, 2)];
    let enemy_team = war.team_of(ENEMY);
    assert!(planner(&war).held_goal_denial(enemy_team) > 0);
    assert_eq!(planner(&v).held_goal_denial(enemy_team), 0);
}

#[test]
fn a_city_sheltering_an_enemy_heir_is_worth_taking() {
    let byzantium_id = PlayerId(3);
    let mut v = ViewBuilder::new(40, 24)
        .city(1, ME, off(4, 4), 4)
        .city(2, ENEMY, off(14, 4), 4)
        .build();
    v.players
        .iter_mut()
        .find(|p| p.id == byzantium_id)
        .unwrap()
        .faction = Faction::Byzantium;
    let v = with_heir(v, ENEMY, byzantium_id, off(14, 4));
    let p = planner(&v);
    assert!(p.heir_in_city_value(&v.cities[1]) > 0);
    assert!(p.hoard_denial_value(&v.cities[1]) > 0);
    assert_eq!(p.capture_goal_swing(&v.cities[1]), (2, 2));
}

#[test]
fn a_switched_off_goal_is_never_planned_around() {
    let mut v = germany(5, false);
    assert!(planner(&v).goal_pending());
    v.rule_experiments
        .faction_rules_off
        .insert(legatum_core::experiments::FactionRule::GermanyGoal);
    let p = planner(&v);
    assert!(!p.goal_pending());
    assert!(!p.forge_open());
    assert_eq!(p.factory_goal_value(&v.cities[4], 40), 0);
    assert_eq!(p.factory_goal_expansion_cost(), 0);
    assert_eq!(
        p.factory_goal_stage_value(&v.cities[4], BuildingKind::Forge, 10),
        0
    );
    assert_eq!(
        p.factory_goal_supply_value(legatum_core::resources::Resource::Iron),
        0
    );
    assert!(!p.factory_completes_goal(&v.cities[4], BuildingKind::Forge));
    assert_valid(&v, &brain(&v));
}

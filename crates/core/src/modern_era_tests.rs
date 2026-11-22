//! The Modern era (user, 2026-11-14, 1950 to 2010): the ninth era, between the World War era (the
//! era first called Modern) and the Future. Five techs, the Mechanised Infantry, Main Battle Tank,
//! Rocket Artillery, Jet Fighter and Jet Bomber, and the Supermarket and the Air Defence. The Air
//! Defence's duel is tested with the air rules (`rules::air`).
use crate::data;
use crate::economy;
use crate::eras;
use crate::orders::Order;
use crate::rules::test_support::{city, game, hex, orders, resolve_units, unit};
use crate::*;

const P0: PlayerId = PlayerId(0);

#[test]
fn a_nine_era_game_researches_all_forty_five_techs_in_order() {
    let mut s = game(&[0, 1]);
    assert_eq!(eras::ERA_COUNT, 9);
    assert_eq!(TechId::ALL.len(), 45);
    for tech in TechId::ALL {
        let p = s.player(P0);
        assert!(
            eras::researchable(p, tech),
            "{tech:?} is open when the earlier techs are known"
        );
        let era_before = s.era_of(P0);
        assert_eq!(era_before, eras::era_of_tech(tech));
        s.player_mut(P0).techs.insert(tech);
    }
    assert_eq!(s.player(P0).techs.len(), 45);
    assert_eq!(s.era_of(P0), eras::LAST_ERA);
    assert_eq!(eras::era_name(7), "Modern");
    assert_eq!(eras::era_name(6), "World War");
    // The strict gate: the new era's five techs are shut until the World War era is done.
    let mut early = game(&[0]);
    for tech in TechId::ALL
        .into_iter()
        .filter(|t| eras::era_of_tech(*t) < 6)
    {
        early.player_mut(P0).techs.insert(tech);
    }
    assert!(!eras::researchable(
        early.player(P0),
        TechId::MechanisedWarfare
    ));
    early.player_mut(P0).techs.extend([
        TechId::MassMobilisation,
        TechId::Rocketry,
        TechId::ArmouredWarfare,
        TechId::AirPower,
        TechId::Electronics,
    ]);
    for tech in [
        TechId::MechanisedWarfare,
        TechId::CompositeArmour,
        TechId::GuidedMissiles,
        TechId::JetEngines,
        TechId::GreenRevolution,
    ] {
        assert_eq!(eras::era_of_tech(tech), 7, "{tech:?}");
        assert!(eras::researchable(early.player(P0), tech), "{tech:?}");
    }
    assert!(!eras::researchable(
        early.player(P0),
        TechId::AutonomousSystems
    ));
}

#[test]
fn the_modern_tech_costs_are_the_futures_old_ones_and_the_future_moved_up_a_step() {
    let cost = |t: TechId| data::tech_stats(t).cost;
    assert_eq!(
        [
            TechId::MechanisedWarfare,
            TechId::CompositeArmour,
            TechId::GuidedMissiles,
            TechId::JetEngines,
            TechId::GreenRevolution
        ]
        .map(cost),
        [654, 670, 686, 702, 720]
    );
    assert_eq!(
        [
            TechId::AutonomousSystems,
            TechId::Robotics,
            TechId::FusionPower,
            TechId::StealthTechnology,
            TechId::RecursiveAi
        ]
        .map(cost),
        [840, 862, 884, 906, 930]
    );
}

#[test]
fn each_modern_unit_upgrades_from_its_world_war_unit_and_on_to_the_future_or_stealth_one() {
    use UnitKind as U;
    for (from, modern, next, tech) in [
        (
            U::Infantry,
            U::MechanisedInfantry,
            U::HunterKiller,
            TechId::MechanisedWarfare,
        ),
        (
            U::Tank,
            U::MainBattleTank,
            U::AutonomousArmour,
            TechId::CompositeArmour,
        ),
        (
            U::Artillery,
            U::RocketArtillery,
            U::Drone,
            TechId::GuidedMissiles,
        ),
        (
            U::Fighter,
            U::JetFighter,
            U::StealthFighter,
            TechId::JetEngines,
        ),
        (
            U::Bomber,
            U::JetBomber,
            U::StealthBomber,
            TechId::JetEngines,
        ),
    ] {
        assert_eq!(data::unit_stats(from).upgrades_to, Some(modern), "{from:?}");
        assert_eq!(
            data::unit_stats(modern).upgrades_to,
            Some(next),
            "{modern:?}"
        );
        assert_eq!(data::unit_stats(modern).requires, Some(tech), "{modern:?}");
        assert_eq!(data::unit_class(modern), data::unit_class(from));
        assert_eq!(data::unit_era_tier(modern), 7);
        assert!(data::unit_stats(modern).cost > data::unit_stats(from).cost);
        assert!(data::unit_stats(next).cost > data::unit_stats(modern).cost);
    }
    // Germany's Panzer is a Tank, so it takes the Tank's upgrade like every faction piece: a Main
    // Battle Tank, which is not a Panzer.
    assert_eq!(
        data::unit_stats_for(Faction::Germany, U::Tank).upgrades_to,
        Some(U::MainBattleTank)
    );
    assert!(!data::is_panzer_rules(
        Default::default(),
        Faction::Germany,
        U::MainBattleTank
    ));
}

#[test]
fn the_modern_units_stand_about_a_fifth_above_the_world_war_ones() {
    use UnitKind as U;
    let s = data::unit_stats;
    assert_eq!(s(U::MechanisedInfantry).strength, 107);
    assert_eq!(s(U::Infantry).strength, 89);
    assert_eq!(s(U::MainBattleTank).strength, 113);
    assert_eq!(
        (s(U::RocketArtillery).strength, s(U::RocketArtillery).ranged),
        (92, 118)
    );
    assert_eq!(s(U::JetFighter).strength, 124);
    assert_eq!(s(U::JetFighter).ranged, 124);
    assert_eq!(s(U::JetBomber).ranged, 137);
    for (modern, old) in [
        (U::MechanisedInfantry, U::Infantry),
        (U::MainBattleTank, U::Tank),
        (U::RocketArtillery, U::Artillery),
        (U::JetFighter, U::Fighter),
        (U::JetBomber, U::Bomber),
    ] {
        let power = |k: U| s(k).strength.max(s(k).ranged);
        let percent = power(modern) * 100 / power(old);
        assert!((118..=122).contains(&percent), "{modern:?} {percent}%");
    }
    // The Future moved up one era step with its era: 50% over the Modern line, not over the World War one.
    assert_eq!(s(U::HunterKiller).strength, 161);
}

#[test]
fn mechanised_infantry_has_three_moves_and_the_main_battle_tank_five() {
    assert_eq!(
        data::unit_stats(UnitKind::MechanisedInfantry).moves,
        data::MECHANISED_INFANTRY_MOVES
    );
    assert_eq!(data::MECHANISED_INFANTRY_MOVES, 3);
    assert_eq!(
        data::unit_stats(UnitKind::MainBattleTank).moves,
        data::TANK_MOVES
    );
    assert_eq!(data::unit_stats(UnitKind::Infantry).moves, 2);
    assert!(!data::is_horse_mounted(UnitKind::MainBattleTank));
}

#[test]
fn jets_reach_one_tile_farther_and_stealth_still_reaches_farther_than_jets() {
    let range = |k: UnitKind| data::unit_stats(k).range;
    assert_eq!(range(UnitKind::JetFighter), 6);
    assert_eq!(range(UnitKind::Fighter), 5);
    assert_eq!(range(UnitKind::JetBomber), 9);
    assert_eq!(range(UnitKind::Bomber), 8);
    assert!(range(UnitKind::StealthFighter) > range(UnitKind::JetFighter));
    assert!(range(UnitKind::StealthBomber) > range(UnitKind::JetBomber));
    assert!(data::is_fighter(UnitKind::JetFighter) && !data::is_bomber(UnitKind::JetFighter));
    assert!(data::is_bomber(UnitKind::JetBomber) && !data::is_fighter(UnitKind::JetBomber));
    assert_eq!(
        data::fighter_guard_radius(UnitKind::JetFighter),
        data::FIGHTER_RANGE
    );
}

#[test]
fn rocket_artillery_shoots_three_tiles_and_artillery_two() {
    assert_eq!(data::unit_stats(UnitKind::RocketArtillery).range, 3);
    assert_eq!(data::unit_stats(UnitKind::Artillery).range, 2);
    assert!(data::indirect_fire(UnitKind::RocketArtillery, true));
    assert!(data::built_in_siege(UnitKind::RocketArtillery));
    let mut s = game(&[0, 1]);
    let rocket = unit(&mut s, 0, UnitKind::RocketArtillery, hex(4, 4));
    let gun = unit(&mut s, 0, UnitKind::Artillery, hex(4, 6));
    let near = unit(&mut s, 1, UnitKind::Soldier, hex(7, 4));
    unit(&mut s, 1, UnitKind::Soldier, hex(4, 9));
    // Scouts so that both targets are seen: only the range is in question.
    unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
    unit(&mut s, 0, UnitKind::Scout, hex(4, 8));
    assert_eq!(hex(4, 4).distance(hex(7, 4)), 3);
    let (next, events) = resolve_units(
        &s,
        &orders(
            0,
            vec![Order::RangedAttack {
                unit: rocket,
                target: hex(7, 4),
            }],
        ),
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, crate::Event::OrderFailed { .. })),
        "{events:?}"
    );
    assert!(
        next.units.get(&near).is_none_or(|u| u.hp < 100),
        "the rocket hit at 3 tiles"
    );
    assert_eq!(next.units[&rocket].hp, 100, "no retaliation");
    // The Artillery, three tiles from its target, is refused with the range in the reason.
    let (_, events) = resolve_units(
        &s,
        &orders(
            0,
            vec![Order::RangedAttack {
                unit: gun,
                target: hex(4, 9),
            }],
        ),
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            crate::Event::OrderFailed { reason, .. } if reason.contains("range")
        )),
        "{events:?}"
    );
}

#[test]
fn the_supermarket_is_the_third_food_building_and_adds_four_food() {
    use BuildingKind::{Granary, Hydroponics, Supermarket};
    assert_eq!(data::building_predecessor(Supermarket), Some(Hydroponics));
    assert_eq!(data::building_family(Supermarket), Granary);
    assert_eq!(
        data::building_stats(Supermarket).requires,
        Some(TechId::GreenRevolution)
    );
    assert_eq!(data::building_name(Supermarket), "Supermarket");
    assert_eq!(
        data::building_family_yields(Faction::Switzerland, Supermarket).food,
        2 + 3 + 4
    );
    let mut s = game(&[0, 1]);
    s.player_mut(P0).faction = Faction::Switzerland;
    let id = city(&mut s, 0, hex(3, 3), 4);
    let food = |s: &GameState| economy::city_yields(s, &s.cities[&id]).total.food;
    economy::install_building(s.cities.get_mut(&id).unwrap(), Granary);
    let granary = food(&s);
    economy::install_building(s.cities.get_mut(&id).unwrap(), Hydroponics);
    let mill = food(&s);
    economy::install_building(s.cities.get_mut(&id).unwrap(), Supermarket);
    let supermarket = food(&s);
    assert_eq!(mill - granary, 3);
    assert_eq!(supermarket - mill, 4, "the Supermarket's step is +4 Food");
    assert_eq!(
        s.cities[&id].buildings,
        std::collections::BTreeSet::from([Supermarket]),
        "it replaces the Mill in the city's list, as the Mill replaces the Granary"
    );
}

#[test]
fn the_air_defence_is_a_yieldless_city_building_that_guided_missiles_unlock() {
    let stats = data::building_stats(BuildingKind::AirDefence);
    assert_eq!(stats.requires, Some(TechId::GuidedMissiles));
    assert_eq!(stats.yields, Yields::ZERO);
    assert!(data::building_predecessor(BuildingKind::AirDefence).is_none());
    assert!(data::available_to_faction(
        Faction::Germany,
        ProductionItem::Building {
            building: BuildingKind::AirDefence
        }
    ));
    assert_eq!(data::building_name(BuildingKind::AirDefence), "Air Defence");
}

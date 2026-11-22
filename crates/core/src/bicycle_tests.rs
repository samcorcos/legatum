//! Rules-level acceptance of the era tech table and the building families; no Great Work or
//! diplomacy placeholders.
use crate::rules::test_support::{city, game as fixture, hex, unit};
use crate::{
    data, economy, game, research,
    rules::{battle, cities, units},
    visibility, *,
};
use std::collections::BTreeMap;

const P: PlayerId = PlayerId(0);
fn building(b: BuildingKind) -> ProductionItem {
    ProductionItem::Building { building: b }
}

/// Iron and Coal deposits beside the city: the Forge and Factory use 1 supply each.
fn supply(s: &mut GameState, city: CityId) {
    let pos = s.cities[&city].pos;
    for (h, r) in pos
        .neighbors()
        .into_iter()
        .zip([Resource::Iron, Resource::Coal])
    {
        s.map.get_mut(h).unwrap().resource = Some(r);
    }
}

#[test]
fn the_tech_table_has_seven_eras_with_rising_costs_and_a_strict_gate() {
    // Five techs per era (Future is the living era and may grow), and every era costs more than
    // the one before. Exact costs are tuned elsewhere and deliberately not pinned here.
    let mut per_era: BTreeMap<u8, (usize, i32, i32)> = BTreeMap::new();
    for tech in TechId::ALL {
        let cost = data::tech_stats(tech).cost;
        let entry = per_era
            .entry(crate::eras::era_of_tech(tech))
            .or_insert((0, i32::MAX, 0));
        entry.0 += 1;
        entry.1 = entry.1.min(cost);
        entry.2 = entry.2.max(cost);
    }
    assert_eq!(per_era.len(), crate::eras::ERA_COUNT);
    let mut previous_max = 0;
    for (era, (count, min, max)) in per_era {
        assert_eq!(count, 5, "era {era}");
        assert!(
            min > previous_max,
            "era {era} must cost more than the one before"
        );
        previous_max = max;
    }
    let mut s = fixture(&[0, 1]);
    // Nothing from a later era is researchable, whatever else is known.
    assert!(!economy::tech_available(&s, P, TechId::Mathematics));
    assert!(!economy::tech_available(&s, P, TechId::Industrialization));
    assert!(economy::tech_available(&s, P, TechId::Masonry));
    // Finishing the Ancient era opens the Classical one, and only that one.
    crate::rules::test_support::know_eras_before(&mut s, P, 1);
    assert!(economy::tech_available(&s, P, TechId::Mathematics));
    assert!(
        !economy::tech_available(&s, P, TechId::Masonry),
        "already known"
    );
    assert!(!economy::tech_available(&s, P, TechId::Feudalism));
    assert!(!economy::tech_available(&s, P, TechId::Industrialization));
}

#[test]
fn paid_upgrade_requires_exact_predecessor_and_population_but_survives_loss() {
    let mut s = fixture(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 3);
    s.player_mut(P).techs.extend(
        TechId::ALL
            .into_iter()
            .filter(|t| crate::eras::era_of_tech(*t) < crate::eras::LAST_ERA),
    );
    supply(&mut s, id);
    let item = building(BuildingKind::Forge);
    assert!(!economy::can_produce(&s, &s.cities[&id], item));
    s.cities
        .get_mut(&id)
        .unwrap()
        .buildings
        .insert(BuildingKind::Workshop);
    assert!(!economy::can_produce(&s, &s.cities[&id], item));
    s.cities.get_mut(&id).unwrap().population = 4;
    assert!(economy::can_produce(&s, &s.cities[&id], item));
    cities::apply_city_orders(
        &mut s,
        P,
        &[Order::SetProduction { city: id, item }],
        &mut Vec::new(),
    );
    s.cities.get_mut(&id).unwrap().population = 2;
    assert_eq!(
        economy::buy_cost(&s, &s.cities[&id]),
        None,
        "purchase rechecks population"
    );
    cities::produce(&mut s, id, 320, &mut Vec::new());
    assert_eq!(
        s.cities[&id].buildings.iter().copied().collect::<Vec<_>>(),
        [BuildingKind::Forge]
    );
    assert_eq!(
        data::building_family_yields(Faction::Switzerland, BuildingKind::Forge).production,
        5
    );
    assert!(!economy::can_produce(
        &s,
        &s.cities[&id],
        building(BuildingKind::Workshop)
    ));
    assert!(data::has_building(
        &s.cities[&id].buildings,
        BuildingKind::Workshop
    ));
}

#[test]
fn relay_forecast_requires_the_full_paid_project() {
    for faction in Faction::ALL {
        let mut s = fixture(&[0, 1]);
        s.player_mut(P).faction = faction;
        s.player_mut(P).techs.insert(TechId::Philosophy);
        s.player_mut(P).gold = 10_000;
        let id = city(&mut s, 0, hex(3, 3), 3);
        s.cities.get_mut(&id).unwrap().production_stored = 0;
        s.cities
            .get_mut(&id)
            .unwrap()
            .buildings
            .insert(BuildingKind::Library);
        let item = building(BuildingKind::GreatLibrary);
        let quote = research::tech_unlocks(&s, P)
            .into_iter()
            .find(|u| u.item == item)
            .unwrap();
        let cost = if faction == Faction::Sparta { 600 } else { 400 };
        assert_eq!(quote.cost, cost);
        assert_eq!(quote.cost_breakdown.total, cost);
        assert_eq!(quote.buy_cost, None);
        let production = economy::city_yields_for(&s, &s.cities[&id], Some(item))
            .total
            .production;
        assert_eq!(
            quote.cities[0].build_turns,
            Some((cost + production - 1) / production)
        );
        let placed =
            crate::landmarks::candidates(&s, &s.cities[&id], BuildingKind::GreatLibrary)[0];
        cities::apply_city_orders(
            &mut s,
            P,
            &[Order::SetProduction {
                city: id,
                item: placed,
            }],
            &mut Vec::new(),
        );
        assert_eq!(economy::buy_cost(&s, &s.cities[&id]), None);
        cities::produce(&mut s, id, cost - 1, &mut Vec::new());
        assert!(
            !s.cities[&id]
                .buildings
                .contains(&BuildingKind::GreatLibrary)
        );
        cities::produce(&mut s, id, 1, &mut Vec::new());
        scoring::update(&mut s, &mut Vec::new());
        assert!(
            scoring::vp(&s, P)
                .iter()
                .any(|(source, vp)| matches!(source, VpSource::GreatWork { of: P }) && *vp == 2)
        );
    }
}

#[test]
fn family_inheritance_preserves_faction_identity() {
    assert_eq!(
        data::building_family_yields(Faction::Germany, BuildingKind::Factory).production,
        13
    );
    assert_eq!(
        data::building_family_yields(Faction::Athens, BuildingKind::University).science,
        10
    );
    assert_eq!(
        data::building_family_yields(Faction::Athens, BuildingKind::Bank).gold,
        5
    );
    assert_eq!(
        data::building_family_yields(Faction::India, BuildingKind::Mill).food,
        5
    );
    assert_eq!(
        data::building_family_yields(Faction::Celts, BuildingKind::Mill).food,
        5
    );
}

#[test]
fn building_upgrade_quotes_match_paid_completion() {
    // Egypt's buildings use Food and cannot be bought; its own tests cover that route.
    for faction in Faction::ALL.into_iter().filter(|f| *f != Faction::Egypt) {
        for tier in [BuildingKind::Forge, BuildingKind::Factory] {
            let mut s = fixture(&[0, 1]);
            s.player_mut(P).faction = faction;
            s.player_mut(P).techs.extend(
                TechId::ALL
                    .into_iter()
                    .filter(|t| crate::eras::era_of_tech(*t) < crate::eras::LAST_ERA),
            );
            s.player_mut(P).gold = 10_000;
            let id = city(&mut s, 0, hex(3, 3), 6);
            supply(&mut s, id);
            s.cities.get_mut(&id).unwrap().production_stored = 0;
            s.cities
                .get_mut(&id)
                .unwrap()
                .buildings
                .insert(BuildingKind::Library);
            s.cities
                .get_mut(&id)
                .unwrap()
                .buildings
                .insert(data::building_predecessor(tier).unwrap());
            let hash = s.hash();
            let unlock = research::tech_unlocks(&s, P)
                .into_iter()
                .find(|u| u.item == building(tier))
                .unwrap();
            assert_eq!(s.hash(), hash);
            let base_cost = if tier == BuildingKind::Forge {
                320
            } else {
                440
            };
            let expected = match faction {
                Faction::Sparta => base_cost * 3 / 2,
                _ => base_cost,
            };
            assert_eq!(unlock.cost, expected, "{faction:?} {tier:?}");
            assert_eq!(unlock.cost_breakdown.total, expected);
            assert_eq!(
                unlock
                    .cost_breakdown
                    .parts
                    .iter()
                    .map(|(_, v)| v)
                    .sum::<i32>(),
                expected
            );
            let income_after = unlock
                .cities
                .iter()
                .find(|c| c.city == id)
                .unwrap()
                .yields_after
                .clone();
            let mut events = Vec::new();
            cities::apply_city_orders(
                &mut s,
                P,
                &[
                    Order::SetProduction {
                        city: id,
                        item: building(tier),
                    },
                    Order::BuyItem {
                        city: id,
                        item: building(tier),
                    },
                ],
                &mut events,
            );
            assert_eq!(s.player(P).gold, 10_000 - unlock.buy_cost.unwrap());
            cities::produce(&mut s, id, 0, &mut events);
            assert!(events.iter().any(
                |e| matches!(e, Event::ProductionCompleted { city, item, .. }
                    if *city == id && *item == building(tier))
            ));
            assert_eq!(economy::city_yields(&s, &s.cities[&id]), income_after);
        }
    }
    assert_eq!(
        data::building_stats_for(Faction::Germany, BuildingKind::Workshop).cost,
        60
    );
    assert_eq!(
        data::building_stats_for(Faction::Germany, BuildingKind::Workshop)
            .yields
            .production,
        2
    );
}

#[test]
fn research_quotes_diffusion_at_its_cap_and_completes_with_the_leftover_science() {
    let mut s = fixture(&[0, 1, 2, 3, 4, 5, 6]);
    crate::rules::test_support::know_eras_before(&mut s, P, 1);
    // Five other empires knowing Philosophy reaches the 50% cap; a sixth adds nothing.
    let base = data::tech_stats(TechId::Philosophy).cost;
    for n in 1..=6u8 {
        s.player_mut(PlayerId(n)).techs.insert(TechId::Philosophy);
        let quote = economy::tech_cost(&s, P, TechId::Philosophy);
        let percent = (10 * i32::from(n)).min(50);
        assert_eq!(quote.total, base * (100 - percent) / 100);
    }
    assert!(economy::tech_available(&s, P, TechId::Philosophy));
    let quote = economy::tech_cost(&s, P, TechId::Philosophy);
    assert_eq!(quote.total, base / 2); // maximum diffusion
    s.player_mut(P).science_stored = quote.total + 17;
    s.player_mut(P).researching = Some(TechId::Philosophy);
    cities::process_research(&mut s, &mut Vec::new());
    assert!(s.player(P).techs.contains(&TechId::Philosophy));
    assert_eq!(s.player(P).science_stored, 17);
    assert!(!s.player(P).techs.contains(&TechId::IronWorking));
}

#[test]
fn forecasts_gate_upgrades_and_match_research_and_building_yields() {
    let mut s = fixture(&[0, 1]);
    s.player_mut(P).faction = Faction::Athens;
    let id = city(&mut s, 0, hex(3, 3), 4);
    s.cities
        .get_mut(&id)
        .unwrap()
        .buildings
        .insert(BuildingKind::Library);
    let unlocks = research::tech_unlocks(&s, P);
    let college = unlocks
        .iter()
        .find(|u| u.item == building(BuildingKind::University))
        .unwrap();
    assert!(
        college.cities[0]
            .blocker
            .as_ref()
            .unwrap()
            .contains("population 5")
    );
    assert_eq!(college.cost_breakdown.total, college.cost);
    s.cities.get_mut(&id).unwrap().population = 5;
    let before = economy::city_yields(&s, &s.cities[&id]).total;
    let effect = research::income_change(&s, P, TechId::PrintingPress);
    s.player_mut(P).techs.insert(TechId::PrintingPress);
    let after = economy::city_yields(&s, &s.cities[&id]).total;
    assert_eq!(effect[0].yields, after - before);
    s.player_mut(P).techs.insert(TechId::Education);
    let college = research::tech_unlocks(&s, P)
        .into_iter()
        .find(|u| u.item == building(BuildingKind::University))
        .unwrap();
    assert_eq!(college.cities[0].blocker, None);
    economy::install_building(s.cities.get_mut(&id).unwrap(), BuildingKind::University);
    assert_eq!(
        college.cities[0].yields_after,
        economy::city_yields_for(&s, &s.cities[&id], None)
    );
}

#[test]
fn siege_forecast_and_resolution_hit_the_city_while_it_shields_its_garrison() {
    let mut s = fixture(&[0, 1]);
    let target = hex(5, 3);
    let c = city(&mut s, 1, target, 4);
    s.cities
        .get_mut(&c)
        .unwrap()
        .buildings
        .insert(BuildingKind::Castle);
    let a = unit(&mut s, 0, UnitKind::Crossbowman, hex(3, 3));
    let g = unit(&mut s, 1, UnitKind::Spearman, target);
    visibility::update_explored(&mut s);
    // Civ 6: the city absorbs the shot and its garrison is safe.
    let f = forecast_attack_mode(&player_view(&s, P), a, target, AttackMode::Ranged).unwrap();
    assert_eq!(f.attacker_strength.total, 31);
    assert_eq!(
        f.defender_strength.total, 38,
        "10 + 8 population + 15 Bastion + 5 Castle"
    );
    assert!(f.garrison.as_ref().is_some_and(|g| g.shielded));
    let fight = battle::plan_fight(&s, 0, a, battle::Target::City(c), battle::FightKind::Ranged);
    assert_eq!(f.attacker_strength, fight.attacker_strength);
    assert_eq!(f.defender_strength, fight.defender_strength);
    // At 0 HP the garrison is the target.
    s.cities.get_mut(&c).unwrap().hp = 0;
    let f = forecast_attack_mode(&player_view(&s, P), a, target, AttackMode::Ranged).unwrap();
    assert_eq!(f.attacker_strength.total, 31);
    assert!(f.garrison.as_ref().is_some_and(|g| !g.shielded));
    let at_garrison =
        battle::plan_fight(&s, 0, a, battle::Target::Unit(g), battle::FightKind::Ranged);
    assert_eq!(f.attacker_strength, at_garrison.attacker_strength);
    assert_eq!(f.defender_strength, at_garrison.defender_strength);
    s.units.remove(&g);
    s.cities.get_mut(&c).unwrap().hp = 1;
    battle::apply_fights(&mut s, &[fight], &mut Vec::new());
    assert_eq!(
        s.cities[&c].owner,
        PlayerId(1),
        "ranged fire cannot capture"
    );
}

#[test]
fn marksmen_keep_celts_stats_and_paid_upgrades_do_not_move_or_fire() {
    let mut s = fixture(&[0, 1]);
    s.player_mut(P).faction = Faction::Celts;
    city(&mut s, 0, hex(3, 3), 3);
    let a = unit(&mut s, 0, UnitKind::Archer, hex(3, 3));
    s.player_mut(P).techs.insert(TechId::Engineering);
    s.player_mut(P).gold = 200;
    // A Composite Archer uses 1 Iron (the Classical era's resource).
    s.player_mut(P)
        .techs
        .extend([TechId::BronzeWorking, TechId::IronWorking]);
    let iron = s.map.get_mut(hex(4, 3)).unwrap();
    iron.resource = Some(crate::Resource::Iron);
    iron.owner = Some(P);
    s.units.get_mut(&a).unwrap().goto = Some(hex(5, 3));
    units::apply_unit_orders(
        &mut s,
        &BTreeMap::from([(P, &[Order::UpgradeUnit { unit: a }][..])]),
        &mut Vec::new(),
    );
    assert_eq!(s.units[&a].kind, UnitKind::CompositeArcher);
    assert_eq!(s.units[&a].moves_left, 0);
    // 2 x (168 - 132) Gold: the Celts have no cost surcharge any more (removed 2026-11-02).
    assert_eq!(s.player(P).gold, 128);
    let st = data::unit_stats_for(Faction::Celts, UnitKind::CompositeArcher);
    assert_eq!(
        (st.cost, st.strength, st.ranged, st.range, st.moves),
        (168, 14, 22, 2, 2)
    );
    // The Celts use the ordinary archery line.
    let st = data::unit_stats_for(Faction::Celts, UnitKind::Crossbowman);
    assert_eq!(
        (st.cost, st.ranged, st.moves),
        (214, data::era_scaled(31, 2), 2)
    );
}

#[test]
fn no_technology_reveals_distant_ruins_or_foreign_state_or_extends_scout_sight() {
    let mut s = fixture(&[0, 1]);
    let scout = game::spawn_unit(&mut s, P, UnitKind::Scout, hex(2, 4));
    let edge = hex(5, 4);
    let distant = hex(10, 4);
    let enemy = city(&mut s, 1, distant, 3);
    s.map.get_mut(distant).unwrap().ruin = true;
    assert!(!player_view(&s, P).tile(edge).unwrap().visible);
    s.player_mut(P).techs.extend(TechId::ALL);
    let view = player_view(&s, P);
    assert!(
        !view.tile(edge).unwrap().visible,
        "no tech adds Scout sight"
    );
    assert_eq!(
        view.units.iter().find(|u| u.id == scout).unwrap().max_moves,
        3
    );
    assert!(!view.tile(distant).unwrap().ruin);
    assert!(!view.tile(distant).unwrap().visible);
    assert_eq!(view.tile(distant).unwrap().terrain, None);
    assert!(!view.cities.iter().any(|c| c.id == enemy));
}

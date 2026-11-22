//! Default factions must keep their identity strengths and quote what resolves.
use legatum_core::{data, economy, game, goals, rules, scoring, state::Map, *};
use std::collections::BTreeMap;

const P: PlayerId = PlayerId(0);
fn state(faction: Faction) -> GameState {
    let mut setup = GameSetup::all_ai(3);
    setup.players.truncate(2);
    setup.players[0].faction = faction;
    game::blank_game(&setup, Map::new(32, 24, Terrain::Grassland))
}
fn city(s: &mut GameState, col: i32, row: i32) -> CityId {
    let unit = game::spawn_unit(s, P, UnitKind::Settler, Hex::from_offset(col, row));
    rules::cities::found_city(s, unit, &mut vec![]).unwrap()
}

#[test]
fn signatures_survive_current_defaults() {
    let office = data::building_stats_for(Faction::India, BuildingKind::Market);
    assert_eq!((office.cost, office.yields.gold), (60, 3));
    assert_eq!(goals::spec(Faction::India).vp, 2);
    let foundry = data::building_stats_for(Faction::Germany, BuildingKind::MachineShop);
    assert_eq!((foundry.cost, foundry.yields.production), (320, 6));
    let legion = data::unit_stats_for(Faction::Byzantium, UnitKind::Trooper);
    assert_eq!((legion.cost, legion.strength), (162, 36));
    assert_eq!(
        data::unit_stats_for(Faction::Germany, UnitKind::Settler).cost,
        70
    );
    assert_eq!(
        data::building_stats_for(Faction::Byzantium, BuildingKind::Workshop).cost,
        60
    );
    assert_eq!(
        data::terrain_yields(Faction::Celts, Terrain::Hills),
        Yields::new(0, 2, 0, 0)
    );
    let archer = data::unit_stats_for(Faction::Celts, UnitKind::Ranger);
    assert_eq!((archer.ranged, archer.moves, archer.cost), (15, 2, 132));
    let scout = data::unit_stats_for(Faction::Mongols, UnitKind::Scout);
    assert_eq!((scout.cost, scout.strength, scout.moves), (60, 5, 4));
    let keshig = data::unit_stats_for(Faction::Mongols, data::KESHIG_KIND);
    let cataphract = data::unit_stats(UnitKind::Cataphract);
    assert_eq!(keshig.strength, cataphract.strength);
    assert!(keshig.ranged > 0 && keshig.range >= 1);
    assert_eq!(
        data::unit_stats_for(Faction::Mongols, UnitKind::Settler).moves,
        2
    );
    assert_eq!(
        data::unit_stats_for(Faction::Sparta, UnitKind::Soldier).strength,
        20
    );
    assert_eq!(
        data::terrain_yields(Faction::Celts, Terrain::Grassland).food,
        2
    );
    assert_eq!(
        data::unit_stats_for(Faction::Celts, UnitKind::Soldier).strength,
        20
    );
}

#[test]
fn germany_forge_goal_needs_five_foundry_cities_and_scores_two_vp_while_it_holds() {
    let mut s = state(Faction::Germany);
    let ids: Vec<_> = [(4, 4), (12, 10), (20, 4), (28, 10), (4, 16)]
        .into_iter()
        .map(|(col, row)| city(&mut s, col, row))
        .collect();
    for (n, id) in ids.iter().enumerate() {
        assert!(!goals::reached(&s, P), "{n} Foundries");
        s.cities
            .get_mut(id)
            .unwrap()
            .buildings
            .insert(BuildingKind::MachineShop);
    }
    assert!(goals::reached(&s, P));
    // Foundries count even in resistance.
    s.cities.get_mut(&ids[0]).unwrap().resistance = 1;
    assert!(goals::reached(&s, P));
    s.cities.get_mut(&ids[0]).unwrap().resistance = 0;
    let mut events = vec![];
    scoring::update(&mut s, &mut events);
    scoring::update(&mut s, &mut events);
    let expected = 2;
    assert_eq!(events.iter().filter(|event| matches!(event, Event::FactionGoalCompleted { player, vp, .. } if *player == P && *vp == expected)).count(), 1);
    assert_eq!(
        scoring::vp(&s, P)
            .iter()
            .find(|(src, _)| matches!(src, VpSource::FactionGoal { .. }))
            .unwrap()
            .1,
        expected
    );
    // A sixth city without a Foundry leaves the live VP in place.
    city(&mut s, 16, 18);
    let mut events = vec![];
    scoring::update(&mut s, &mut events);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::FactionGoalLost { player, .. } if *player == P))
    );
    assert!(
        scoring::vp(&s, P)
            .iter()
            .any(|(src, _)| matches!(src, VpSource::FactionGoal { .. }))
    );
    // Losing a Foundry city drops the count below five.
    s.cities.remove(&ids[0]);
    let mut events = vec![];
    scoring::update(&mut s, &mut events);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::FactionGoalLost { player, .. } if *player == P))
    );
    assert!(
        !scoring::vp(&s, P)
            .iter()
            .any(|(src, _)| matches!(src, VpSource::FactionGoal { .. }))
    );
}

#[test]
fn india_hive_bonus_and_capital_distance_are_quoted() {
    assert_eq!(data::INDIA_CAPITAL_RADIUS, 4);
    for (distance, loss) in [
        (4, 0),
        (5, 10),
        (6, 20),
        (7, 30),
        (8, 40),
        (9, 50),
        (14, 100),
    ] {
        let mut s = state(Faction::India);
        let capital = city(&mut s, 4, 4);
        let remote = city(&mut s, 4 + distance, 4);
        let yields = economy::city_yields(&s, &s.cities[&remote]);
        let distance_source = yields
            .sources
            .iter()
            .find(|source| source.label.starts_with("Corruption:"));
        assert_eq!(distance_source.is_some(), distance > 4);
        if let Some(source) = distance_source {
            assert_eq!(
                source.label,
                format!("Corruption: {distance} tiles from the capital (-{loss}%)")
            );
            let before = yields
                .sources
                .iter()
                .take_while(|part| !part.label.starts_with("Corruption:"))
                .fold(Yields::ZERO, |sum, part| sum + part.yields);
            assert_eq!(source.yields, before.percent(100 - loss) - before);
            if distance == 6 {
                assert_eq!(before.gold, 1);
                assert_eq!(source.yields.gold, -1, "80% of one Gold truncates to zero");
            }
        }
        assert_eq!(
            yields
                .sources
                .iter()
                .any(|source| source.label.starts_with("India: local network")),
            distance <= 5
        );
        if let Some(network) = yields
            .sources
            .iter()
            .find(|source| source.label.starts_with("India: local network"))
        {
            assert_eq!((network.yields.gold, network.yields.science), (0, 0));
        }
        assert!(
            !economy::city_yields(&s, &s.cities[&capital])
                .sources
                .iter()
                .any(|source| source.label.starts_with("Corruption:"))
        );
        assert!(
            economy::city_yields(&s, &s.cities[&remote])
                .sources
                .iter()
                .any(|s| s.label == "India network" && s.yields.food == 1)
        );
        assert!(yields.total.production >= 3);
        if distance >= 14 {
            assert_eq!(yields.total.production, 3);
            assert!(
                yields
                    .sources
                    .iter()
                    .any(|source| source.label == "Minimum city production"
                        && source.yields.production == 3)
            );
        }
    }
}

#[test]
fn india_distance_loss_is_linear_capped_and_safe_at_extremes() {
    for (beyond, loss) in [
        (i32::MIN, 0),
        (-1, 0),
        (0, 0),
        (1, 10),
        (2, 20),
        (3, 30),
        (4, 40),
        (5, 50),
        (10, 100),
        (i32::MAX, 100),
    ] {
        assert_eq!(data::india_distance_loss_percent(beyond), loss);
    }
}

#[test]
fn india_capital_capture_removes_distance_loss_until_a_new_capital_is_founded() {
    let mut s = state(Faction::India);
    s.player_mut(PlayerId(1)).faction = Faction::Byzantium;
    let old_capital = city(&mut s, 4, 4);
    let remote = city(&mut s, 10, 4);
    let loss = |s: &GameState| {
        economy::city_yields(s, &s.cities[&remote])
            .sources
            .into_iter()
            .find(|source| source.label.starts_with("Corruption:"))
            .map(|source| source.label)
    };
    assert!(loss(&s).unwrap().contains("(-20%)"));
    rules::battle::capture_city(&mut s, old_capital, PlayerId(1), &mut Vec::new());
    s.cities.get_mut(&old_capital).unwrap().resistance = 0;
    assert!(s.capital_of(P).is_none());
    assert_eq!(loss(&s), None);
    city(&mut s, 18, 4);
    assert!(loss(&s).unwrap().contains("(-40%)"));
}

#[test]
fn india_small_network_yields_round_to_nearest_whole_food_and_production() {
    let mut s = state(Faction::India);
    let capital = city(&mut s, 4, 4);
    s.cities.get_mut(&capital).unwrap().population = 8;
    let before = economy::city_yields(&s, &s.cities[&capital]).total;
    city(&mut s, 9, 4);
    let after = economy::city_yields(&s, &s.cities[&capital]);
    let network = after
        .sources
        .iter()
        .find(|source| source.label.starts_with("India: local network"))
        .unwrap();
    assert_eq!(network.yields.food, (before.food * 7 + 50) / 100);
    assert_eq!(
        network.yields.production,
        (before.production * 7 + 50) / 100
    );
    assert_eq!(network.yields.gold, 0);
    assert_eq!(network.yields.science, 0);
}

#[test]
fn remote_commerce_requires_an_active_market_and_owned_capital_at_eight_hexes() {
    let mut s = state(Faction::Mongols);
    let capital = city(&mut s, 4, 4);
    let remote = city(&mut s, 12, 4);
    s.cities
        .get_mut(&remote)
        .unwrap()
        .buildings
        .insert(BuildingKind::Clearinghouse);
    let bonus = |s: &GameState| {
        economy::city_yields(s, &s.cities[&remote])
            .sources
            .iter()
            .find(|source| source.label == "Mongols: long-road commerce")
            .map_or(0, |source| source.yields.gold)
    };
    let expected = 2;
    assert_eq!(bonus(&s), expected);
    s.cities.get_mut(&capital).unwrap().pos = Hex::from_offset(5, 4);
    assert_eq!(bonus(&s), 0);
    s.cities.get_mut(&capital).unwrap().pos = Hex::from_offset(4, 4);
    s.cities.get_mut(&remote).unwrap().resistance = 1;
    assert_eq!(bonus(&s), 0);
    s.cities.get_mut(&remote).unwrap().resistance = 0;
    s.cities.get_mut(&capital).unwrap().is_capital = false;
    assert_eq!(bonus(&s), 0);
}

#[test]
fn sparta_regiments_rest_heal_normally_at_home_in_field_or_fortified() {
    let mut s = state(Faction::Sparta);
    let home = city(&mut s, 4, 4);
    let soldier = game::spawn_unit(&mut s, P, UnitKind::Soldier, Hex::from_offset(4, 4));
    for (fortify, pos) in [
        (0, Hex::from_offset(4, 4)),
        (0, Hex::from_offset(14, 4)),
        (6, Hex::from_offset(4, 4)),
    ] {
        // Spartan military uses the normal home or field rate while fortified too.
        s.cities.get_mut(&home).unwrap().buildings.clear();
        let unit = s.units.get_mut(&soldier).unwrap();
        unit.pos = pos;
        unit.hp = 40;
        unit.acted = false;
        unit.fortify = fortify;
        let forecast = rules::units::resting_heal(&s, &s.units[&soldier]);
        assert_eq!(
            forecast.total,
            if pos == Hex::from_offset(4, 4) {
                20
            } else {
                10
            }
        );
        assert_eq!(
            forecast.parts.iter().map(|(_, value)| value).sum::<i32>(),
            if pos == Hex::from_offset(4, 4) {
                20
            } else {
                10
            }
        );
        rules::units::heal_and_fortify(&mut s);
        assert_eq!(
            s.units[&soldier].hp,
            40 + if pos == Hex::from_offset(4, 4) {
                20
            } else {
                10
            }
        );
    }
}

#[test]
fn sparta_scouts_and_regiments_rest_heal_alike() {
    let mut s = state(Faction::Sparta);
    let home = city(&mut s, 4, 4);
    s.cities.get_mut(&home).unwrap().buildings.clear();
    let scout = game::spawn_unit(&mut s, P, UnitKind::Scout, Hex::from_offset(4, 4));
    let hoplite = game::spawn_unit(&mut s, P, UnitKind::Soldier, Hex::from_offset(4, 4));
    let field = Hex::from_offset(14, 4);
    for (pos, scout_heal) in [(Hex::from_offset(4, 4), 20), (field, 10)] {
        for id in [scout, hoplite] {
            let unit = s.units.get_mut(&id).unwrap();
            unit.pos = pos;
            unit.hp = 40;
            unit.acted = false;
        }
        assert_eq!(
            rules::units::resting_heal(&s, &s.units[&scout]).total,
            scout_heal
        );
        assert_eq!(
            rules::units::resting_heal(&s, &s.units[&hoplite]).total,
            if pos == Hex::from_offset(4, 4) {
                20
            } else {
                10
            }
        );
        rules::units::heal_and_fortify(&mut s);
        assert_eq!(s.units[&scout].hp, 40 + scout_heal);
        assert_eq!(
            s.units[&hoplite].hp,
            40 + if pos == Hex::from_offset(4, 4) {
                20
            } else {
                10
            }
        );
    }
}

#[test]
fn the_sparta_goal_scores_only_once_the_third_veteran_has_retired() {
    let mut s = state(Faction::Sparta);
    city(&mut s, 4, 4);
    let faction_goal = |s: &GameState| {
        scoring::vp(s, P)
            .iter()
            .find(|(src, _)| matches!(src, VpSource::FactionGoal { .. }))
            .map(|(_, vp)| *vp)
    };
    // Armies, walls and enemy land earn nothing: only retirement does.
    let mut events = vec![];
    scoring::update(&mut s, &mut events);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::FactionGoalCompleted { .. }))
            && faction_goal(&s).is_none()
    );
    s.player_mut(P).veterans_retired = 2;
    scoring::update(&mut s, &mut events);
    assert!(faction_goal(&s).is_none());
    // The action completes it (`rules::actions`); scoring then keeps it for good.
    s.player_mut(P).veterans_retired = 3;
    s.player_mut(P).faction_goal_done = true;
    scoring::update(&mut s, &mut events);
    assert_eq!(faction_goal(&s), Some(goals::spec(Faction::Sparta).vp));
    assert!(s.player(P).faction_goal_done);
}

#[test]
fn upkeep_quote_matches_resolution_after_free_slots() {
    // No technology changes upkeep any more (Strategic Logistics is gone), whatever is known.
    for all_techs in [false, true] {
        for military in 0..7 {
            let mut s = state(Faction::Switzerland);
            city(&mut s, 4, 4);
            s.player_mut(P).gold = 1_000;
            if all_techs {
                s.player_mut(P).techs.extend(TechId::ALL);
            }
            for i in 0..military {
                game::spawn_unit(&mut s, P, UnitKind::Soldier, Hex::from_offset(4 + i, 5));
            }
            let paid = (military - 2).max(0);
            let expected = 3 * paid;
            assert_eq!(economy::upkeep_breakdown(&s, P).total, expected);
            let before = s.player(P).gold;
            let gold = economy::city_yields(&s, s.cities_of(P).next().unwrap())
                .total
                .gold;
            let research_gold = if all_techs {
                economy::player_income(&s, P).science.total / 2
            } else {
                0
            };
            let (next, _) = resolve_turn(&s, &BTreeMap::new());
            assert_eq!(
                next.player(P).gold,
                before + gold + research_gold + economy::swiss_interest(&s, P) - expected
            );
        }
    }
}

#[test]
fn swiss_interest_and_capture_bounty_are_public_and_conserved() {
    let mut s = state(Faction::Switzerland);
    // A Celts captor razes instead of taking the city, so use a conventional captor.
    s.player_mut(PlayerId(1)).faction = Faction::Mongols;
    let capital = city(&mut s, 4, 4);
    let colony = city(&mut s, 9, 4);
    let enemy = game::spawn_unit(
        &mut s,
        PlayerId(1),
        UnitKind::Settler,
        Hex::from_offset(20, 15),
    );
    rules::cities::found_city(&mut s, enemy, &mut vec![]).unwrap();
    s.cities.get_mut(&capital).unwrap().population = 5;
    s.cities.get_mut(&colony).unwrap().population = 3;
    s.player_mut(P).gold = 450;
    assert_eq!(economy::swiss_interest(&s, P), 4);
    assert_eq!(
        economy::player_income(&s, P)
            .gold
            .parts
            .iter()
            .find(|(label, _)| label == "Switzerland: interest")
            .unwrap()
            .1,
        4
    );
    assert_eq!(economy::swiss_capture_bounty(&s, &s.cities[&colony]), 168);
    assert_eq!(economy::swiss_capture_bounty(&s, &s.cities[&capital]), 450);
    assert_eq!(
        view::player_view(&s, PlayerId(1))
            .cities
            .iter()
            .find(|c| c.id == colony)
            .and_then(|c| c.capture_bounty),
        None
    ); // unseen before contact
    let before = s.player(PlayerId(1)).gold;
    let mut events = vec![];
    rules::battle::capture_city(&mut s, colony, PlayerId(1), &mut events);
    assert_eq!(s.player(P).gold, 282);
    assert_eq!(s.player(PlayerId(1)).gold, before + 168);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::CityCaptured {
            bounty: Some(168),
            ..
        }
    )));
}

#[test]
fn swiss_interest_has_no_maximum_and_decadence_cuts_it_like_any_income() {
    let mut s = state(Faction::Switzerland);
    city(&mut s, 4, 4);
    for (gold, interest) in [(99, 0), (100, 1), (1_234, 12), (50_000, 500)] {
        s.player_mut(P).gold = gold;
        assert_eq!(economy::swiss_interest(&s, P), interest, "{gold} Gold");
        assert_eq!(economy::swiss_interest_cut(&s, P), 0);
    }
    // Decadence takes its rate from the interest too (user, 2026-10-17: everything that feels like
    // income), to the nearest Gold, and the breakdown shows the cut as its own line.
    s.setup.rule_experiments.decadence_slope = Some(10);
    s.player_mut(P).decadence_percent = 30;
    assert_eq!(economy::swiss_interest_gross(&s, P), 500);
    assert_eq!(economy::swiss_interest_cut(&s, P), 150);
    assert_eq!(economy::swiss_interest(&s, P), 350);
    let income = economy::player_income(&s, P);
    let part = |label: &str| {
        income
            .gold
            .parts
            .iter()
            .find(|(l, _)| l == label)
            .map(|p| p.1)
    };
    assert_eq!(part("Switzerland: interest"), Some(500));
    assert_eq!(
        part("Decadence \u{2212}30% of interest: luxury"),
        Some(-150)
    );
    // The forecast and resolution agree: the treasury grows by the taxed interest.
    let before = s.player(P).gold;
    let city_gold = economy::city_yields(&s, s.cities_of(P).next().unwrap())
        .total
        .gold;
    assert_eq!(income.gold.total, city_gold + 350 - economy::upkeep(&s, P));
    let (next, _) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(
        next.player(P).gold,
        before + city_gold + 350 - economy::upkeep(&s, P)
    );
    // A small hoard's interest rounds like a city's yield: 3 Gold at 12% loses nothing.
    s.player_mut(P).decadence_percent = 12;
    s.player_mut(P).gold = 300;
    assert_eq!(economy::swiss_interest_cut(&s, P), 0);
    s.player_mut(P).gold = 1_000;
    assert_eq!(economy::swiss_interest_cut(&s, P), 1);
    assert!(
        economy::player_income(&s, P)
            .gold
            .parts
            .iter()
            .any(|(l, v)| l.starts_with("Decadence") && *v == -1)
    );
    // No decadence, no line.
    s.player_mut(P).decadence_percent = 0;
    assert!(
        economy::player_income(&s, P)
            .gold
            .parts
            .iter()
            .all(|(l, _)| !l.starts_with("Decadence"))
    );
    // Other factions have no interest to cut.
    s.player_mut(P).faction = Faction::Mongols;
    s.player_mut(P).decadence_percent = 30;
    assert_eq!(economy::swiss_interest_gross(&s, P), 0);
    assert_eq!(economy::swiss_interest_cut(&s, P), 0);
    s.player_mut(P).faction = Faction::Switzerland;
    s.player_mut(P).decadence_percent = 0;
    s.player_mut(P).gold = 50_000;
    assert_eq!(economy::swiss_interest(&s, P), 500);
    s.player_mut(P).gold = 250;
    assert_eq!(economy::swiss_interest(&s, P), 2);
}

#[test]
fn scouts_cost_no_upkeep_and_use_no_free_slot() {
    for faction in [Faction::Byzantium, Faction::Switzerland, Faction::Mongols] {
        let mut s = state(faction);
        for i in 0..5 {
            game::spawn_unit(&mut s, P, UnitKind::Scout, Hex::from_offset(4 + i, 5));
        }
        game::spawn_unit(&mut s, P, UnitKind::Settler, Hex::from_offset(4, 8));
        assert_eq!(economy::upkeep_breakdown(&s, P).total, 0, "{faction:?}");
        // Three armed units: only the third pays, however many Scouts stand beside them.
        for i in 0..3 {
            game::spawn_unit(&mut s, P, UnitKind::Soldier, Hex::from_offset(4 + i, 6));
        }
        assert_eq!(
            economy::upkeep_breakdown(&s, P).total,
            data::military_upkeep(faction),
            "{faction:?}"
        );
    }
}

#[test]
fn switzerland_with_ranger_soldier_and_two_scouts_are_not_bankrupt() {
    // The 2026-10-06 playtest: turn 6, 0 Gold, a Ranger, a Soldier and two Scouts.
    let mut s = state(Faction::Switzerland);
    city(&mut s, 4, 4);
    game::spawn_unit(&mut s, P, UnitKind::Ranger, Hex::from_offset(5, 4));
    game::spawn_unit(&mut s, P, UnitKind::Soldier, Hex::from_offset(3, 4));
    game::spawn_unit(&mut s, P, UnitKind::Scout, Hex::from_offset(6, 6));
    game::spawn_unit(&mut s, P, UnitKind::Scout, Hex::from_offset(7, 6));
    s.player_mut(P).gold = 0;
    assert_eq!(economy::upkeep_breakdown(&s, P).total, 0);
    let before = s.units_of(P).count();
    let (next, events) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(next.units_of(P).count(), before, "nothing disbanded");
    assert!(next.player(P).gold >= 0);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::UnitDisbanded { .. }))
    );
}

#[test]
fn bankruptcy_never_disbands_a_scout() {
    let mut s = state(Faction::Switzerland);
    city(&mut s, 4, 4);
    let scout = game::spawn_unit(&mut s, P, UnitKind::Scout, Hex::from_offset(6, 6));
    for i in 0..3 {
        game::spawn_unit(&mut s, P, UnitKind::Soldier, Hex::from_offset(3 + i, 7));
    }
    s.player_mut(P).gold = 0;
    let (next, _) = resolve_turn(&s, &BTreeMap::new());
    assert!(next.units.contains_key(&scout));
}

#[test]
fn every_empire_starts_with_standard_gold() {
    let setup = GameSetup::all_ai(3);
    let s = game::new_game(&setup);
    assert!(!s.players.is_empty());
    for p in &s.players {
        assert_eq!(p.gold, data::START_GOLD, "{:?}", p.faction);
    }
    assert_eq!(data::START_GOLD, 60);
}

#[test]
fn starting_gold_does_not_trivialise_swiss_interest_or_goal() {
    let mut setup = GameSetup::all_ai(3);
    setup.players[0].faction = Faction::Switzerland;
    let s = game::new_game(&setup);
    assert_eq!(economy::swiss_interest(&s, P), 0);
    assert!(!goals::reached(&s, P));
}

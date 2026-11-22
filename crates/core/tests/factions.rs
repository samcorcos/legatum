//! Faction contracts tested through the same public economy, resolution and view paths as play.
use legatum_core::{combat, data, economy, game, goals, rules, scoring, state::Map, visibility, *};

const P: PlayerId = PlayerId(0);
fn at(x: i32, y: i32) -> Hex {
    Hex::from_offset(x, y)
}
fn state(f: Faction) -> GameState {
    let mut setup = GameSetup::all_ai(3);
    setup.players.truncate(2);
    setup.players[0].faction = f;
    setup.players[1].faction = Faction::India;
    {
        let mut state = game::blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        // These pre-diplomacy tactical fixtures assume every foreign team is at War.
        for relation in &mut state.diplomacy.as_mut().unwrap().relations {
            relation.state = legatum_core::diplomacy::RelationState::War;
        }
        state
    }
}
/// Egypt under the rule the Nile replaced (Food-funded ordinary buildings), for its ablation arm.
fn classic_egypt() -> GameState {
    let mut s = state(Faction::Egypt);
    s.setup
        .rule_experiments
        .faction_rules_off
        .insert(legatum_core::experiments::FactionRule::EgyptNile);
    s
}
fn city(s: &mut GameState, h: Hex) -> CityId {
    let u = game::spawn_unit(s, P, UnitKind::Settler, h);
    rules::cities::found_city(s, u, &mut vec![]).unwrap()
}
fn building(b: BuildingKind) -> ProductionItem {
    ProductionItem::Building { building: b }
}
fn unit(u: UnitKind) -> ProductionItem {
    ProductionItem::Unit { unit: u }
}

#[test]
fn egypt_food_construction_and_switches_keep_the_two_banks_separate() {
    let mut s = classic_egypt();
    let id = city(&mut s, at(4, 4));
    let granary = building(BuildingKind::Granary);
    let soldier = unit(UnitKind::Spearman);
    let food = economy::city_totals(&s, &s.cities[&id]).food;
    let surplus = (food - 2).max(0);
    assert!(surplus > 0);
    let production = economy::city_totals(&s, &s.cities[&id]).production;
    s.cities.get_mut(&id).unwrap().production_stored = 30;
    s.cities
        .get_mut(&id)
        .unwrap()
        .switch_production_for(Some(granary), true);
    assert_eq!(s.cities[&id].production_stored, 0); // idle Production cannot enter a building
    let before_food = s.cities[&id].food_stored;
    rules::cities::process_cities(&mut s, &mut vec![]);
    let progress = s.cities[&id].production_stored;
    assert_eq!(progress, 2 * surplus);
    assert_eq!(s.cities[&id].food_stored, before_food);
    assert_eq!(s.cities[&id].population, 1);
    assert!(production > 0); // it was ignored, not silently added
    let v = player_view(&s, P);
    let d = v
        .cities
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .detail
        .as_ref()
        .unwrap();
    assert_eq!(d.turns_to_grow, None);
    assert_eq!(
        d.turns_to_complete,
        Some((50 - progress + 2 * surplus - 1) / (2 * surplus))
    );
    assert_eq!(
        d.options.iter().find(|o| o.item == granary).unwrap().turns,
        d.turns_to_complete.unwrap()
    );
    s.cities
        .get_mut(&id)
        .unwrap()
        .switch_production_for(Some(soldier), false);
    assert_eq!(s.cities[&id].banked(granary), progress);
    assert_eq!(s.cities[&id].production_stored, 0);
    s.cities.get_mut(&id).unwrap().food_stored = data::FIRST_GROWTH - 1;
    rules::cities::process_cities(&mut s, &mut vec![]);
    assert_eq!(s.cities[&id].population, 2); // units use Production and allow growth
    s.cities
        .get_mut(&id)
        .unwrap()
        .switch_production_for(Some(granary), true);
    assert_eq!(s.cities[&id].production_stored, progress);
    assert!(
        economy::buy_blocker(&s, &s.cities[&id], granary)
            .unwrap()
            .contains("Gold cannot buy")
    );
    assert_eq!(
        data::building_stats_for(Faction::Egypt, BuildingKind::Granary)
            .yields
            .food,
        3
    );
    let saved = serde_json::to_string(&s).unwrap();
    let loaded: GameState = serde_json::from_str(&saved).unwrap();
    assert_eq!(s, loaded);
}

#[test]
fn egypt_completion_drops_food_overflow_and_population_gate_pauses() {
    let mut s = classic_egypt();
    let id = city(&mut s, at(4, 4));
    let granary = building(BuildingKind::Granary);
    s.cities
        .get_mut(&id)
        .unwrap()
        .switch_production_for(Some(granary), true);
    s.cities.get_mut(&id).unwrap().production_stored = 49;
    s.cities
        .get_mut(&id)
        .unwrap()
        .production_queue
        .push(unit(UnitKind::Spearman));
    rules::cities::process_cities(&mut s, &mut vec![]);
    assert!(s.cities[&id].buildings.contains(&BuildingKind::Granary));
    assert_eq!(s.cities[&id].producing, Some(unit(UnitKind::Spearman)));
    assert_eq!(s.cities[&id].production_stored, 0);

    let factory = building(BuildingKind::Forge);
    s.player_mut(P).techs.insert(TechId::IronWorking);
    let c = s.cities.get_mut(&id).unwrap();
    c.population = 4;
    c.buildings.insert(BuildingKind::Workshop);
    c.switch_production_for(Some(factory), true);
    c.production_stored = 17;
    c.population = 3;
    c.food_stored = 0;
    let before = c.production_stored;
    let food = economy::city_totals(&s, &s.cities[&id]).food;
    rules::cities::process_cities(&mut s, &mut vec![]);
    assert_eq!(s.cities[&id].production_stored, before);
    assert_eq!(s.cities[&id].food_stored, (food - 6).max(0));
    assert_eq!(s.cities[&id].producing, Some(factory));
    assert!(
        economy::production_blocker(&s, &s.cities[&id], factory, false)
            .unwrap()
            .contains("population 4")
    );
}

fn grow() -> ProductionItem {
    ProductionItem::Project {
        project: IndustryProject::Grow,
    }
}

#[test]
fn nile_surplus_food_builds_everything_and_a_building_city_does_not_grow() {
    let mut s = state(Faction::Egypt);
    let id = city(&mut s, at(4, 4));
    let soldier = unit(UnitKind::Spearman);
    let base = {
        let mut classic = s.clone();
        classic
            .setup
            .rule_experiments
            .faction_rules_off
            .insert(legatum_core::experiments::FactionRule::EgyptNile);
        economy::city_yields_for(&classic, &classic.cities[&id], Some(grow())).total
    };
    assert!(base.production > 0);
    let growing = economy::city_yields_for(&s, &s.cities[&id], Some(grow())).total;
    assert_eq!(growing.food, 2 * base.food);
    assert_eq!(growing.production, 0);
    let surplus = growing.food - data::FOOD_PER_POP;
    let progress = surplus * i32::from(legatum_core::experiments::EGYPT_NILE_BUILD_PERCENT) / 100;
    for item in [
        soldier,
        unit(UnitKind::Settler),
        building(BuildingKind::Granary),
    ] {
        let building = economy::city_yields_for(&s, &s.cities[&id], Some(item)).total;
        assert_eq!(building.food, data::FOOD_PER_POP, "{item:?}");
        assert_eq!(building.production, progress, "{item:?}");
    }
    let c = s.cities.get_mut(&id).unwrap();
    c.switch_production(Some(soldier));
    c.food_stored = data::FIRST_GROWTH - 1;
    // The view's cached forecasts key on whether the city builds (a debug assertion checks).
    let d = player_view(&s, P)
        .cities
        .into_iter()
        .find(|c| c.id == id)
        .unwrap();
    assert_eq!(d.detail.unwrap().turns_to_grow, None);
    rules::cities::process_cities(&mut s, &mut vec![]);
    assert_eq!(s.cities[&id].production_stored, progress);
    assert_eq!(s.cities[&id].population, 1);
    assert_eq!(s.cities[&id].food_stored, data::FIRST_GROWTH - 1);
    // Grow: the same surplus grows the city, and the Spearman keeps its progress.
    s.cities
        .get_mut(&id)
        .unwrap()
        .switch_production(Some(grow()));
    // Forecasts price builds and growth from one cached city (a debug assertion checks).
    for producing in [Some(grow()), None, Some(soldier)] {
        let mut s = s.clone();
        s.cities.get_mut(&id).unwrap().producing = producing;
        let _ = player_view(&s, P);
        let _ = legatum_core::research::tech_unlocks(&s, P);
    }
    rules::cities::process_cities(&mut s, &mut vec![]);
    assert_eq!(s.cities[&id].population, 2);
    assert_eq!(s.cities[&id].banked(soldier), progress);
    // Gold buys as usual, and nothing about the rule leaves the save.
    assert!(
        economy::buy_blocker(&s, &s.cities[&id], building(BuildingKind::Granary))
            .is_none_or(|why| !why.contains("Food"))
    );
    let saved = serde_json::to_string(&s).unwrap();
    let loaded: GameState = serde_json::from_str(&saved).unwrap();
    assert_eq!(s, loaded);
}

#[test]
fn nile_grow_is_egypts_and_the_workshop_line_is_not() {
    let rules = legatum_core::experiments::RuleExperiments::default();
    assert!(rules.available_to_player(Faction::Egypt, 0, grow()));
    for f in Faction::ALL.into_iter().filter(|f| *f != Faction::Egypt) {
        assert!(!rules.available_to_player(f, 0, grow()), "{f:?}");
    }
    for b in [
        BuildingKind::Workshop,
        BuildingKind::Forge,
        BuildingKind::Factory,
    ] {
        assert!(!data::available_to_faction(Faction::Egypt, building(b)));
        assert!(data::available_to_faction(
            Faction::Switzerland,
            building(b)
        ));
    }
    let mut classic = rules;
    classic
        .faction_rules_off
        .insert(legatum_core::experiments::FactionRule::EgyptNile);
    assert!(!classic.available_to_player(Faction::Egypt, 0, grow()));
    assert!(data::available_to_faction_rules(
        classic.faction_rules_off,
        Faction::Egypt,
        building(BuildingKind::Workshop)
    ));
}

#[test]
fn medjay_sight_uses_visibility_and_unit_view() {
    let mut s = state(Faction::Egypt);
    let medjay = game::spawn_unit(&mut s, P, UnitKind::Spearman, at(10, 10));
    let v = player_view(&s, P);
    assert_eq!(
        v.units.iter().find(|u| u.id == medjay).unwrap().sight,
        Some(data::UNIT_SIGHT + 1)
    );
    let edge = at(10 + data::UNIT_SIGHT + 1, 10);
    let seen = visibility::visible_tiles(&s, s.team_of(P));
    assert!(seen.get(s.map.index(edge).unwrap()));
    s.player_mut(P).faction = Faction::India;
    let seen = visibility::visible_tiles(&s, s.team_of(P));
    assert!(!seen.get(s.map.index(edge).unwrap()));
    // A Company keeps the seller's unit version when a different empire hires it.
    s.units.get_mut(&medjay).unwrap().origin = Some(Faction::Egypt);
    let seen = visibility::visible_tiles(&s, s.team_of(P));
    assert!(seen.get(s.map.index(edge).unwrap()));
    assert_eq!(
        player_view(&s, P)
            .units
            .iter()
            .find(|u| u.id == medjay)
            .unwrap()
            .sight,
        Some(data::UNIT_SIGHT + 1)
    );
}

#[test]
fn egypt_food_rule_applies_only_to_ordinary_buildings() {
    let mut s = classic_egypt();
    let id = city(&mut s, at(4, 4));
    let c = &s.cities[&id];
    for item in [
        unit(UnitKind::Spearman),
        unit(UnitKind::Settler),
        ProductionItem::Improvement {
            improvement: ImprovementKind::Farm,
            tile: at(4, 5),
        },
        ProductionItem::Wonder {
            building: BuildingKind::Pyramids,
            tile: at(4, 5),
        },
        ProductionItem::Project {
            project: IndustryProject::Gold,
        },
    ] {
        assert!(!rules::cities::egypt_building(&s, c, item), "{item:?}");
    }
    assert!(rules::cities::egypt_building(
        &s,
        c,
        building(BuildingKind::Granary)
    ));
    assert!(
        economy::buy_blocker(&s, c, building(BuildingKind::Granary))
            .unwrap()
            .contains("Gold cannot buy")
    );
    // A stalled building cannot borrow this city's ordinary Production or its stored Food.
    let c = s.cities.get_mut(&id).unwrap();
    c.switch_production_for(Some(building(BuildingKind::Granary)), true);
    c.production_stored = 4;
    c.food_stored = 20;
    let food = economy::city_totals(&s, &s.cities[&id]).food;
    s.cities.get_mut(&id).unwrap().population = ((food + 1) / 2).max(1);
    let before = s.cities[&id].production_stored;
    rules::cities::process_cities(&mut s, &mut vec![]);
    assert_eq!(s.cities[&id].production_stored, before);
    let v = player_view(&s, P);
    let d = v
        .cities
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .detail
        .as_ref()
        .unwrap();
    assert_eq!(d.turns_to_complete, None);
    assert_eq!(
        d.options
            .iter()
            .find(|o| o.item == building(BuildingKind::Granary))
            .unwrap()
            .turns,
        -1
    );
}

#[test]
fn replacements_use_base_ids_with_faction_costs_names_and_unlocks() {
    for (f, b, cost, name) in [
        (
            Faction::Switzerland,
            BuildingKind::Walls,
            120,
            "Alpine Redoubt",
        ),
        (Faction::India, BuildingKind::Market, 60, "Revenue Office"),
        (Faction::Athens, BuildingKind::Library, 70, "Lyceum"),
    ] {
        let mut s = state(f);
        let c = city(&mut s, at(4, 4));
        assert_eq!(economy::production_cost(&s, P, building(b)), cost);
        assert_eq!(data::building_name_for(f, b), name);
        if let Some(tech) = data::building_stats(b).requires {
            assert!(!economy::can_produce(&s, &s.cities[&c], building(b)));
            s.player_mut(P).techs.insert(tech);
        }
        assert!(economy::can_produce(&s, &s.cities[&c], building(b)));
        s.cities.get_mut(&c).unwrap().buildings.insert(b);
        assert!(!economy::can_produce(&s, &s.cities[&c], building(b)));
    }
    for (f, u, cost, moves, melee, ranged, name) in [
        (
            Faction::Byzantium,
            UnitKind::Pikeman,
            162,
            2,
            36,
            0,
            "Tagmatic Soldier",
        ),
        (
            Faction::Mongols,
            data::KESHIG_KIND,
            data::KESHIG_COST,
            data::MOUNTED_MOVES + 1,
            data::unit_stats(data::KESHIG_KIND).strength,
            data::era_scaled(data::KESHIG_RANGED, 1),
            "Keshig",
        ),
        (Faction::Mongols, UnitKind::Scout, 60, 4, 5, 0, "Scout"),
        (
            Faction::Sparta,
            UnitKind::Spearman,
            80,
            2,
            20,
            0,
            "Spartan Hoplite",
        ),
    ] {
        let mut s = state(f);
        let id = game::spawn_unit(&mut s, P, u, at(4, 4));
        visibility::update_explored(&mut s);
        let v = player_view(&s, P);
        let v = v.units.iter().find(|v| v.id == id).unwrap();
        assert_eq!((v.max_moves, v.strength, v.ranged), (moves, melee, ranged));
        assert_eq!(economy::production_cost(&s, P, unit(u)), cost);
        assert_eq!(data::unit_name_for(f, u), name);
    }
}

#[test]
fn ruhr_foundry_replaces_the_forge_and_factory_is_ordinary() {
    let workshop = data::building_stats_for(Faction::Germany, BuildingKind::Workshop);
    assert_eq!((workshop.cost, workshop.yields.production), (60, 2));
    assert_eq!(
        data::building_name_for(Faction::Germany, BuildingKind::Workshop),
        "Workshop"
    );
    let factory = data::building_stats_for(Faction::Germany, BuildingKind::Forge);
    let ordinary = data::building_stats_for(Faction::India, BuildingKind::Forge);
    assert_eq!(
        (factory.cost, factory.yields.production),
        (ordinary.cost, 6)
    );
    assert_eq!(ordinary.yields.production, 3);
    assert_eq!(factory.yields.food, 0);
    assert_eq!(factory.requires, Some(TechId::IronWorking));
    assert_eq!(
        data::building_stats_for(Faction::Germany, BuildingKind::Factory)
            .yields
            .production,
        5
    );
    assert_eq!(
        legatum_core::resources::building_need(BuildingKind::Forge),
        Some(Resource::Iron)
    );
    assert_eq!(
        legatum_core::resources::building_need(BuildingKind::Factory),
        Some(Resource::Coal)
    );
    assert_eq!(
        data::building_name_for(Faction::Germany, BuildingKind::Forge),
        "Ruhr Foundry"
    );
    assert!(data::building_satisfies(
        BuildingKind::Factory,
        BuildingKind::Workshop
    ));
    assert_eq!(
        data::building_predecessor(BuildingKind::Factory),
        Some(BuildingKind::Forge)
    );
}

#[test]
fn building_prices_purchases_and_settler_sequence_apply_once() {
    for (f, costs) in [(Faction::Germany, [50, 50, 60, 60, 80, 100, 400])] {
        let mut s = state(f);
        let c = city(&mut s, at(4, 4));
        s.player_mut(P).techs.extend(TechId::ALL);
        for (b, cost) in BuildingKind::ALL.into_iter().zip(costs) {
            assert_eq!(economy::production_cost(&s, P, building(b)), cost);
            let city = s.cities.get_mut(&c).unwrap();
            city.producing = Some(building(b));
            city.production_stored = 7;
            assert_eq!(
                economy::buy_cost(&s, &s.cities[&c]),
                (b != BuildingKind::GreatLibrary
                    && data::building_family(b) != BuildingKind::Walls
                    && data::available_to_faction(f, building(b)))
                .then_some(cost * 2)
            );
        }
    }
    let mut s = state(Faction::Germany);
    for n in 0..3 {
        s.player_mut(P).settlers_trained = n;
        assert_eq!(
            economy::production_cost(&s, P, unit(UnitKind::Settler)),
            // The first Settler costs FIRST_SETTLER_PERCENT (65%).
            if n == 0 {
                70 * data::FIRST_SETTLER_PERCENT / 100
            } else {
                70 + 50 * n
            }
        );
    }
    for u in UnitKind::ALL {
        assert_eq!(
            data::unit_stats_for(Faction::Germany, u).cost,
            data::unit_stats(u).cost
        );
        // The Panzer is the one German unit with its own move.
        assert_eq!(
            data::unit_stats_for(Faction::Germany, u).moves,
            data::unit_stats(u).moves + i32::from(u == UnitKind::Tank)
        );
    }
}

#[test]
fn swiss_rounding_upkeep_and_bastion_strength() {
    let mut s = state(Faction::Switzerland);
    let c = city(&mut s, at(4, 4));
    let city = s.cities.get_mut(&c).unwrap();
    city.population = 5;
    city.producing = Some(unit(UnitKind::Spearman));
    city.production_stored = 1;
    assert_eq!(economy::buy_cost(&s, &s.cities[&c]), Some(140)); // Full 80 * 1.75, regardless of stored production.
    assert_eq!(economy::city_yields(&s, &s.cities[&c]).total.gold, 3);
    for i in 0..4 {
        game::spawn_unit(&mut s, P, UnitKind::Spearman, at(4 + i, 4));
    }
    assert_eq!(economy::upkeep(&s, P), 6);
    assert_eq!(
        combat::city_defense_strength_for(Faction::Switzerland, 5, true, 200, 0).total,
        35
    );
}

#[test]
// Population is territory: a city works all six grassland tiles around it (centre 4 Food, tiles
// 12 Food).
fn replacement_yields_and_india_food_penalty_precede_consumption() {
    let mut india = state(Faction::India);
    let capital = city(&mut india, at(4, 4));
    assert!(!data::available_to_faction(
        Faction::India,
        building(BuildingKind::RiceStorehouse)
    ));
    let sources = economy::city_yields(&india, &india.cities[&capital]).sources;
    assert!(
        sources
            .iter()
            .any(|s| s.label == "India network" && s.yields.food == 1)
    );
    for (f, b, expected) in [
        (
            Faction::Germany,
            BuildingKind::Workshop,
            Yields::new(15, 7, 1, 2),
        ),
        (
            Faction::Athens,
            BuildingKind::Library,
            Yields::new(16, 5, 1, 8),
        ),
        (
            Faction::Athens,
            BuildingKind::Market,
            Yields::new(16, 5, 2, 2),
        ),
    ] {
        let mut s = state(f);
        let c = city(&mut s, at(4, 4));
        s.cities.get_mut(&c).unwrap().buildings.insert(b);
        let y = economy::city_yields(&s, &s.cities[&c]);
        assert_eq!(y.total, expected);
        assert!(
            y.sources
                .iter()
                .any(|s| s.label == data::building_name_for(f, b))
        );
    }
    let mut s = state(Faction::India);
    city(&mut s, at(4, 4));
    let far = city(&mut s, at(14, 4));
    assert_eq!(
        economy::city_yields(&s, &s.cities[&far]).total,
        // Six hexes beyond the capital radius remove 60% of Food.
        // Production keeps its floor of 3.
        Yields::new(6, 3, 0, 0)
    );
}

#[test]
fn india_revenue_offices_remit_half_gross_gold_and_unlock_a_transferable_diwan() {
    let mut s = state(Faction::India);
    let capital = city(&mut s, at(4, 4));
    let province = city(&mut s, at(7, 4));
    s.player_mut(P).techs.insert(TechId::Currency);
    let before = economy::player_income(&s, P).gold.total;
    let gross = economy::city_yields(&s, &s.cities[&province]).total.gold;
    assert!(gross > 0);
    assert!(!economy::can_produce(
        &s,
        &s.cities[&capital],
        building(BuildingKind::ImperialDiwan)
    ));
    s.cities
        .get_mut(&province)
        .unwrap()
        .buildings
        .insert(BuildingKind::Market);
    assert_eq!(
        economy::player_income(&s, P).gold.total - before,
        (gross + 3) / 2
    );
    let mut turn = s.clone();
    turn.player_mut(P).gold = 1_000;
    let remitted_income = economy::player_income(&turn, P).gold.total;
    rules::cities::process_cities(&mut turn, &mut vec![]);
    assert_eq!(turn.player(P).gold, 1_000 + remitted_income);
    assert!(!economy::can_produce(
        &s,
        &s.cities[&capital],
        building(BuildingKind::ImperialDiwan)
    ));
    s.cities
        .get_mut(&capital)
        .unwrap()
        .buildings
        .insert(BuildingKind::Market);
    assert!(economy::can_produce(
        &s,
        &s.cities[&capital],
        building(BuildingKind::ImperialDiwan)
    ));
    s.cities.get_mut(&province).unwrap().producing = Some(building(BuildingKind::ImperialDiwan));
    s.cities.get_mut(&province).unwrap().production_stored = data::IMPERIAL_DIWAN_COST;
    rules::cities::process_cities(&mut s, &mut vec![]);
    assert_eq!(s.player(P).diwan_city, Some(province));
    assert_eq!(goals::diwan_holder(&s, P), Some(P));
    assert_eq!(player_view(&s, P).imperial_diwans[0].city, province);
    rules::battle::capture_city(&mut s, province, PlayerId(1), &mut vec![]);
    assert_eq!(goals::diwan_holder(&s, P), Some(PlayerId(1)));
    assert!(
        s.cities[&province]
            .buildings
            .contains(&BuildingKind::ImperialDiwan)
    );
    assert!(scoring::vp(&s, PlayerId(1)).iter().any(
        |(source, vp)| matches!(source, VpSource::FactionGoal { of } if *of == P) && *vp == 2
    ));
}

#[test]
fn celts_has_no_hill_yield_or_border_preference() {
    let mut s = state(Faction::Celts);
    s.map.get_mut(at(4, 4)).unwrap().terrain = Terrain::Hills;
    let c = city(&mut s, at(4, 4));
    let hill = at(5, 4);
    s.map.get_mut(hill).unwrap().terrain = Terrain::Hills;
    let edge = at(6, 4);
    s.map.get_mut(edge).unwrap().terrain = Terrain::Hills;
    let mut ordinary = s.clone();
    ordinary.player_mut(P).faction = Faction::Mongols;
    assert_eq!(
        economy::city_yields(&s, &s.cities[&c]),
        economy::city_yields(&ordinary, &ordinary.cities[&c])
    );
    assert_eq!(
        rules::cities::next_claim(&s, &s.cities[&c]),
        rules::cities::next_claim(&ordinary, &ordinary.cities[&c])
    );
    assert_eq!(
        data::terrain_yields(Faction::Celts, Terrain::Hills),
        data::terrain_yields(Faction::Mongols, Terrain::Hills)
    );
}

#[test]
fn mongols_military_moves_only_and_sparta_regiments_rest_heal_normally() {
    let mut s = state(Faction::Mongols);
    for kind in UnitKind::ALL {
        let id = game::spawn_unit(&mut s, P, kind, at(4, 4));
        s.units.get_mut(&id).unwrap().moves_left = 0;
    }
    rules::units::reset_turn(&mut s);
    for u in s.units.values() {
        assert_eq!(
            u.moves_left,
            if u.kind == UnitKind::Settler || !data::unit_stats(u.kind).military {
                // Settlers and the Heir are civilians: Mongols's +1 is for military only.
                data::unit_stats(u.kind).moves
            } else {
                data::unit_stats(u.kind).moves + 1
            }
        );
    }
    s.player_mut(P).faction = Faction::Sparta;
    for u in s.units.values_mut() {
        u.hp = 40;
        u.acted = false;
    }
    rules::units::heal_and_fortify(&mut s);
    for u in s.units.values() {
        assert_eq!(u.hp, 50);
    }
    s.map.get_mut(at(4, 4)).unwrap().owner = Some(P);
    rules::units::heal_and_fortify(&mut s);
    for u in s.units.values() {
        assert_eq!(u.hp, 70);
    }
}

#[test]
fn faction_combat_matches_forecast_and_keeps_garrison_priority() {
    for f in Faction::ALL {
        let mut s = state(f);
        let a = game::spawn_unit(&mut s, P, UnitKind::Spearman, at(4, 4));
        s.units.get_mut(&a).unwrap().hp = 50;
        s.player_mut(PlayerId(1)).faction = Faction::Mongols;
        let d = game::spawn_unit(&mut s, PlayerId(1), UnitKind::Pikeman, at(5, 4));
        s.map.get_mut(at(5, 4)).unwrap().terrain = Terrain::Hills;
        s.units.get_mut(&d).unwrap().fortify = 6;
        visibility::update_explored(&mut s);
        let forecast = forecast_attack(&player_view(&s, P), a, at(5, 4)).unwrap();
        assert_eq!(
            forecast.attacker_strength.total,
            match f {
                Faction::Sparta => 22,
                // The Eagle Warrior is 4 stronger than the Spearman (24 less 5 for wounds).
                Faction::Aztecs => 19,
                _ => 15,
            }
        );
        assert_eq!(forecast.defender_strength.total, 38); // terrain absent, fortification retained
        s.units.get_mut(&a).unwrap().goto = Some(at(5, 4));
        let mut events = vec![];
        rules::movement::resolve_movement(&mut s, &mut events);
        let event = events
            .iter()
            .find(|e| matches!(e, Event::Battle { .. }))
            .unwrap();
        if let Event::Battle {
            attacker_strength,
            defender_strength,
            ..
        } = event
        {
            assert_eq!(attacker_strength, &forecast.attacker_strength);
            assert_eq!(defender_strength, &forecast.defender_strength);
        }
    }
    assert_eq!(
        combat::unit_meeting_strength_for(Faction::Sparta, UnitKind::Pikeman, 10, 0).total,
        32
    );
    assert_eq!(
        combat::unit_attack_strength_for(Faction::Byzantium, UnitKind::Pikeman, 50, 0).total,
        31
    );
    assert_eq!(
        combat::unit_ranged_strength_for(Faction::Celts, UnitKind::Archer, 50, 0).total,
        10
    );
    assert_eq!(
        combat::unit_attack_strength_for(Faction::Sparta, UnitKind::Scout, 50, 0).total,
        0
    );
}

/// The attacker's strength breakdown for P's Spearman at (4,4) hitting a rival Spearman at (5,4),
/// with `friend` (owner, kind) standing on a neighbouring tile of the attacker.
fn hoplite_attack(
    faction: Faction,
    friend: Option<(PlayerId, UnitKind)>,
) -> (GameState, UnitId, Breakdown) {
    let mut s = state(faction);
    let a = game::spawn_unit(&mut s, P, UnitKind::Spearman, at(4, 4));
    game::spawn_unit(&mut s, PlayerId(1), UnitKind::Spearman, at(5, 4));
    if let Some((owner, kind)) = friend {
        let tile = at(4, 4)
            .neighbors()
            .into_iter()
            .find(|h| h.distance(at(5, 4)) == 2)
            .unwrap();
        game::spawn_unit(&mut s, owner, kind, tile);
    }
    visibility::update_explored(&mut s);
    let forecast = forecast_attack(&player_view(&s, P), a, at(5, 4)).unwrap();
    (s, a, forecast.attacker_strength)
}

#[test]
fn a_hoplite_next_to_another_hoplite_fights_with_the_phalanx_bonus() {
    let line = (
        combat::PHALANX_LABEL.to_string(),
        data::HOPLITE_PHALANX_STRENGTH,
    );
    let (_, _, alone) = hoplite_attack(Faction::Sparta, None);
    assert!(!alone.parts.contains(&line));

    // One Spartan Spearman next to it adds the bonus, and a second neighbour adds nothing more.
    let (s, a, paired) = hoplite_attack(Faction::Sparta, Some((P, UnitKind::Spearman)));
    assert!(paired.parts.contains(&line));
    assert_eq!(paired.total, alone.total + data::HOPLITE_PHALANX_STRENGTH);
    let mut crowd = s.clone();
    let tile = at(4, 4)
        .neighbors()
        .into_iter()
        .filter(|h| h.distance(at(5, 4)) == 2)
        .nth(1)
        .unwrap();
    game::spawn_unit(&mut crowd, P, UnitKind::Spearman, tile);
    let line_of_three = forecast_attack(&player_view(&crowd, P), a, at(5, 4)).unwrap();
    assert_eq!(line_of_three.attacker_strength.total, paired.total);

    // Not next to it, not a Hoplite, not its own empire, not a Spartan: no bonus.
    for (faction, friend) in [
        (Faction::Sparta, Some((P, UnitKind::Pikeman))),
        (Faction::Sparta, Some((P, UnitKind::Phalangite))),
        (Faction::Sparta, Some((PlayerId(1), UnitKind::Spearman))),
        (Faction::India, Some((P, UnitKind::Spearman))),
    ] {
        let (_, _, b) = hoplite_attack(faction, friend);
        assert!(!b.parts.contains(&line), "{faction:?} {friend:?}");
    }
    let mut far = s.clone();
    let friend = far
        .units
        .values()
        .find(|u| u.id != a && u.owner == P)
        .map(|u| u.id)
        .unwrap();
    far.units.get_mut(&friend).unwrap().pos = at(4, 8);
    visibility::update_explored(&mut far);
    let apart = forecast_attack(&player_view(&far, P), a, at(5, 4)).unwrap();
    assert_eq!(apart.attacker_strength.total, alone.total);

    // The bonus is lost on upgrade: a Phalangite beside a Hoplite is an ordinary Phalangite.
    let mut upgraded = s.clone();
    upgraded.units.get_mut(&a).unwrap().kind = UnitKind::Phalangite;
    let f = forecast_attack(&player_view(&upgraded, P), a, at(5, 4)).unwrap();
    assert!(!f.attacker_strength.parts.contains(&line));
}

#[test]
fn the_phalanx_bonus_applies_in_defence_and_resolution_matches_the_forecast() {
    let line = (
        combat::PHALANX_LABEL.to_string(),
        data::HOPLITE_PHALANX_STRENGTH,
    );
    let mut s = state(Faction::Sparta);
    let hoplite = game::spawn_unit(&mut s, P, UnitKind::Spearman, at(4, 4));
    let friend_tile = at(4, 4)
        .neighbors()
        .into_iter()
        .find(|h| h.distance(at(5, 4)) == 2)
        .unwrap();
    let friend = game::spawn_unit(&mut s, P, UnitKind::Spearman, friend_tile);
    let raider = game::spawn_unit(&mut s, PlayerId(1), UnitKind::Spearman, at(5, 4));
    visibility::update_explored(&mut s);

    // Defending: the rival's forecast against the Hoplite names the phalanx.
    let forecast = forecast_attack(&player_view(&s, PlayerId(1)), raider, at(4, 4)).unwrap();
    assert!(forecast.defender_strength.parts.contains(&line));
    let mut alone = s.clone();
    alone.units.remove(&friend);
    let lone = forecast_attack(&player_view(&alone, PlayerId(1)), raider, at(4, 4)).unwrap();
    assert!(!lone.defender_strength.parts.contains(&line));
    assert_eq!(
        forecast.defender_strength.total,
        lone.defender_strength.total + data::HOPLITE_PHALANX_STRENGTH
    );

    // Resolution plans the same fight from the stage's snapshot.
    let mut events = vec![];
    s.units.get_mut(&raider).unwrap().goto = Some(at(4, 4));
    rules::movement::resolve_movement(&mut s, &mut events);
    let Some(Event::Battle {
        attacker_strength,
        defender_strength,
        ..
    }) = events.iter().find(|e| matches!(e, Event::Battle { .. }))
    else {
        panic!("no battle: {events:?}");
    };
    assert_eq!(attacker_strength, &forecast.attacker_strength);
    assert_eq!(defender_strength, &forecast.defender_strength);
    assert!(s.units.contains_key(&hoplite));
}

#[test]
fn upgrades_follow_replacement_cost_difference_and_preserve_hp() {
    // Spearman to Phalangite: Byzantium's Phalangite is one of its later melee units (+5 Gold),
    // Sparta's Hoplite is the replaced Spearman (no change).
    for (f, cost) in [
        (Faction::Byzantium, 80),
        (Faction::Sparta, 60),
        (Faction::Germany, 60),
    ] {
        let mut s = state(f);
        city(&mut s, at(4, 4));
        s.player_mut(P).techs.insert(TechId::Mathematics);
        s.player_mut(P)
            .techs
            .extend([TechId::BronzeWorking, TechId::IronWorking]);
        s.player_mut(P).gold = 300;
        // The Phalangite uses 1 Iron (the Classical era's resource).
        let tile = s.map.get_mut(at(5, 4)).unwrap();
        tile.resource = Some(legatum_core::Resource::Iron);
        tile.owner = Some(P);
        let u = game::spawn_unit(&mut s, P, UnitKind::Spearman, at(4, 4));
        s.units.get_mut(&u).unwrap().hp = 55;
        assert_eq!(
            rules::units::check_upgrade(&s, P, u, 300),
            Ok((UnitKind::Phalangite, cost))
        );
        rules::units::apply_unit_orders(
            &mut s,
            &std::collections::BTreeMap::from([(P, &[Order::UpgradeUnit { unit: u }][..])]),
            &mut vec![],
        );
        assert_eq!(
            (s.units[&u].kind, s.units[&u].hp, s.player(P).gold),
            (UnitKind::Phalangite, 55, 300 - cost)
        );
    }
}

#[test]
fn athenian_ruins_keep_the_normal_reward_and_add_science() {
    for turn in 1..40 {
        let mut athens = state(Faction::Athens);
        athens.turn = turn;
        let h = at(4, 4);
        athens.map.get_mut(h).unwrap().ruin = true;
        let u = game::spawn_unit(&mut athens, P, UnitKind::Scout, h);
        let mut athens_events = vec![];
        rules::units::explore_ruin(&mut athens, u, &mut athens_events);
        let mut ordinary = state(Faction::Byzantium);
        ordinary.turn = turn;
        ordinary.map.get_mut(h).unwrap().ruin = true;
        let u = game::spawn_unit(&mut ordinary, P, UnitKind::Scout, h);
        let mut ordinary_events = vec![];
        rules::units::explore_ruin(&mut ordinary, u, &mut ordinary_events);
        let athens_reward = athens_events
            .iter()
            .find_map(|e| match e {
                Event::RuinExplored { reward, .. } => Some(reward.clone()),
                _ => None,
            })
            .unwrap();
        let ordinary_reward = ordinary_events
            .iter()
            .find_map(|e| match e {
                Event::RuinExplored { reward, .. } => Some(reward.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(athens_reward, ordinary_reward);
        assert_eq!(athens.player(P).gold, ordinary.player(P).gold);
        assert_eq!(
            athens.player(P).science_stored,
            ordinary.player(P).science_stored + 20
        );
        assert!(
            [(40, 0), (0, 25), (0, 0)]
                .contains(&(ordinary.player(P).gold, ordinary.player(P).science_stored))
        );
    }
}

fn fulfill_goal(s: &mut GameState) {
    match s.player(P).faction {
        Faction::Egypt => (),
        Faction::Switzerland => {
            let goal = goals::SWISS_HOARD_GOLD;
            s.player_mut(P).gold = goal;
        }
        Faction::Byzantium => {
            let h = at(4, 4);
            let u = game::spawn_unit(s, P, UnitKind::HeirToTheThrone, h);
            s.units.get_mut(&u).unwrap().heir_of = Some(P);
        }
        Faction::Mongols => {
            let c = city(s, at(4, 4));
            s.cities
                .get_mut(&c)
                .unwrap()
                .buildings
                .insert(BuildingKind::YamRelayStation);
            s.player_mut(P).hall_city = Some(c);
        }
        // The Diwan is held by the owner of its city.
        Faction::India => {
            let c = city(s, at(4, 4));
            s.cities
                .get_mut(&c)
                .unwrap()
                .buildings
                .insert(BuildingKind::ImperialDiwan);
            s.player_mut(P).diwan_city = Some(c);
        }
        // The broadcast and the retirement of veterans are actions: the action marks the goal done
        // (see `rules::actions` tests for what they require).
        Faction::Athens | Faction::Sparta | Faction::Aztecs => {
            s.player_mut(P).faction_goal_done = true
        }
        f => {
            let n = if f == Faction::Germany { 5 } else { 3 };
            for i in 0..n {
                let c = city(s, at(4 + (i % 3) * 10, 4 + (i / 3) * 10));
                let city = s.cities.get_mut(&c).unwrap();
                city.population = 8;
                city.buildings = BuildingKind::ALL
                    .into_iter()
                    .filter(|b| *b != BuildingKind::GreatLibrary)
                    .collect();
                let h = city.pos;
                for t in h.within(1).iter().take(2) {
                    s.map.get_mut(*t).unwrap().terrain = Terrain::Hills;
                }
            }
            if f == Faction::Celts {
                // Territory three hexes out, then the rite: a camp in every city's land.
                let ids: Vec<_> = s.cities.keys().copied().collect();
                for id in ids {
                    let pos = s.cities[&id].pos;
                    for h in pos.within(3) {
                        if let Some(t) = s.map.get_mut(h)
                            && (t.owner.is_none() || t.city == Some(id))
                        {
                            t.owner = Some(P);
                            t.city = Some(id);
                        }
                    }
                }
                rules::barbarians::call_the_wild(s, P, &mut vec![]);
            }
        }
    }
}

#[test]
fn every_goal_requires_own_state_out_of_resistance_and_awards_its_vp_once() {
    for f in Faction::ALL {
        if matches!(f, Faction::Egypt | Faction::Rome) {
            assert_eq!(goals::spec(f).vp, 0);
            continue;
        }
        let mut s = state(f);
        assert!(!goals::reached(&s, P));
        fulfill_goal(&mut s);
        assert!(goals::reached(&s, P), "{f:?}");
        let action_goal = matches!(f, Faction::Athens | Faction::Sparta | Faction::Aztecs);
        if !matches!(
            f,
            Faction::Switzerland | Faction::Byzantium | Faction::Mongols | Faction::India
        ) && !action_goal
        {
            // A city in resistance still counts for Germany's Foundry total and
            // the Celts' wild-camp rule.
            if !matches!(f, Faction::Germany | Faction::Celts) {
                for c in s.cities.values_mut() {
                    c.resistance = 1;
                }
                assert!(!goals::reached(&s, P), "{f:?}: resistance");
            }
            for c in s.cities.values_mut() {
                c.resistance = 0;
                c.owner = PlayerId(1);
            }
            assert!(!goals::reached(&s, P), "{f:?}: ownership");
            for c in s.cities.values_mut() {
                c.owner = P;
            }
        }
        let mut events = vec![];
        scoring::update(&mut s, &mut events);
        scoring::update(&mut s, &mut events);
        // An action goal's completion event comes from the action, not from scoring.
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e,Event::FactionGoalCompleted{player,vp,..} if *player==P && *vp == goals::spec(f).vp))
                .count(),
            usize::from(!action_goal),
            "{f:?}"
        );
        s.player_mut(P).gold = 0;
        s.player_mut(P).techs.clear();
        s.cities.clear();
        // The Heir's and the Hall's VP are held, not kept: they follow the unit or the city.
        if f != Faction::Byzantium {
            s.units.clear();
        }
        if f == Faction::Mongols {
            let c = city(&mut s, at(4, 4));
            s.cities
                .get_mut(&c)
                .unwrap()
                .buildings
                .insert(BuildingKind::YamRelayStation);
            s.player_mut(P).hall_city = Some(c);
        }
        if f == Faction::India {
            let c = city(&mut s, at(4, 4));
            s.cities
                .get_mut(&c)
                .unwrap()
                .buildings
                .insert(BuildingKind::ImperialDiwan);
            s.player_mut(P).diwan_city = Some(c);
        }
        assert_eq!(
            scoring::vp(&s, P)
                .iter()
                .filter(|(src, _)| matches!(src, VpSource::FactionGoal { .. }))
                .map(|(_, n)| *n)
                .sum::<u32>(),
            goals::spec(f).vp
        );
    }
}

#[test]
fn goal_thresholds_do_not_accept_nearly_complete_empires_or_allied_territory() {
    for f in Faction::ALL {
        if f == Faction::Egypt {
            continue;
        }
        let mut s = state(f);
        fulfill_goal(&mut s);
        match f {
            Faction::Switzerland => {
                let goal = goals::SWISS_HOARD_GOLD;
                s.player_mut(P).gold = goal - 1;
            }
            // Action goals have no threshold: their requirements are validated in `rules::actions`.
            Faction::Athens | Faction::Sparta | Faction::Aztecs | Faction::Rome => continue,
            Faction::India => {
                s.cities.values_mut().next().unwrap().owner = PlayerId(1);
            }
            Faction::Byzantium => {
                // A rival holds the Heir: the Byzantium itself no longer scores.
                s.units.values_mut().next().unwrap().owner = PlayerId(1);
            }
            Faction::Germany => {
                // Removing a Foundry city leaves Germany one short of five.
                let id = *s.cities.keys().last().unwrap();
                s.cities.remove(&id);
            }
            Faction::Celts => {
                // A camp cleared: one city lacks one.
                s.barbarians.as_mut().unwrap().camps.remove(0);
            }
            Faction::Mongols => {
                // A rival holds the Hall's city: Mongols itself no longer scores.
                s.cities.values_mut().next().unwrap().owner = PlayerId(1);
            }
            Faction::Egypt => unreachable!("Egypt has no faction goal"),
        }
        assert!(!goals::reached(&s, P), "{f:?}");
    }
}

#[test]
fn original_serde_names_and_base_piece_ids_still_load() {
    for (name, f) in [
        ("Switzerland", Faction::Switzerland),
        ("India", Faction::India),
        ("Japan", Faction::India),
        ("japan", Faction::India),
        ("Byzantium", Faction::Byzantium),
    ] {
        assert_eq!(
            serde_json::from_str::<Faction>(&format!("\"{name}\"")).unwrap(),
            f
        );
        let mut s = state(f);
        city(&mut s, at(4, 4));
        game::spawn_unit(&mut s, P, UnitKind::Pikeman, at(4, 4));
        let saved = serde_json::to_string(&s).unwrap();
        let loaded: GameState = serde_json::from_str(&saved).unwrap();
        assert_eq!(s, loaded);
    }
}

#[test]
fn faction_names_are_stable_as_json_object_keys() {
    let overrides = [(Faction::Sparta, 7), (Faction::India, 4)]
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let json = serde_json::to_value(&overrides).unwrap();
    assert_eq!(json["Sparta"], 7);
    assert_eq!(json["India"], 4);
    assert_eq!(
        serde_json::from_value::<std::collections::BTreeMap<Faction, i32>>(json).unwrap(),
        overrides
    );
}

#[test]
fn original_three_save_preserves_faction_names() {
    let raw = include_str!("fixtures/original-three-save.json");
    let saved: serde_json::Value = serde_json::from_str(raw).unwrap();
    let factions: Vec<_> = saved["players"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["faction"].as_str().unwrap())
        .collect();
    assert_eq!(factions, ["Switzerland", "Japan", "Byzantium"]);
}

#[test]
fn germany_industry_runs_each_turn_without_completion_or_purchase() {
    let mut s = state(Faction::Germany);
    let c = city(&mut s, at(4, 4));
    for project in [IndustryProject::Gold, IndustryProject::Science] {
        let item = ProductionItem::Project { project };
        let preview = economy::city_yields_for(&s, &s.cities[&c], Some(item));
        let production = preview.total.production;
        let amount = match project {
            IndustryProject::Gold => {
                preview.total.gold - economy::city_yields_for(&s, &s.cities[&c], None).total.gold
            }
            IndustryProject::Science => {
                preview.total.science
                    - economy::city_yields_for(&s, &s.cities[&c], None)
                        .total
                        .science
            }
            IndustryProject::Grow => unreachable!("Germany has no Grow project"),
        };
        assert_eq!(
            amount,
            production
                * if project == IndustryProject::Gold {
                    50
                } else {
                    25
                }
                / 100
        );
        assert!(economy::can_produce(&s, &s.cities[&c], item));
        assert_eq!(economy::buy_cost_for(&s, &s.cities[&c], item), None);
        let before = s.cities[&c].production_stored;
        s.cities.get_mut(&c).unwrap().producing = Some(item);
        let (next, events) = resolve_turn(&s, &std::collections::BTreeMap::new());
        assert_eq!(next.cities[&c].production_stored, before);
        assert_eq!(next.cities[&c].producing, Some(item));
        assert!(!events.iter().any(
            |e| matches!(e, legatum_core::Event::ProductionCompleted { city, .. } if *city == c)
        ));
        s = next;
    }
    s.player_mut(P).faction = Faction::India;
    assert!(!economy::can_produce(
        &s,
        &s.cities[&c],
        ProductionItem::Project {
            project: IndustryProject::Gold
        }
    ));
}

/// The gate behind the user's rule (2026-11-22): "every tech card that you get should have at
/// least something". A faction researches only technologies that give it something in the era it
/// researches them; any other technology is skipped (known from the start). A new skip must be a
/// reviewed design choice, so the list of skips is pinned here.
#[test]
fn no_faction_ever_researches_a_tech_that_gives_it_nothing() {
    let rules = Default::default();
    let mut skips = Vec::new();
    for f in Faction::ALL {
        for tech in TechId::ALL {
            if data::tech_skipped(rules, f, tech) {
                skips.push((f, tech));
                continue;
            }
            // Researched in its own era: the state-aware test (the Mongols' Walls wait for the
            // Medieval era) must still find something.
            let era = legatum_core::eras::era_of_tech(tech);
            assert!(
                data::tech_has_player_benefit(rules, f, era, tech),
                "{f:?} would research {tech:?} for nothing in era {era}: give it a benefit or skip it"
            );
        }
    }
    assert_eq!(skips, vec![(Faction::Egypt, TechId::BronzeWorking)]);
    // The Nile takes all Production, so Egypt's cards drop the lines that only add Production
    // (a Deep Mine's Gold and cheaper land units still help it).
    for tech in TechId::ALL {
        for line in data::tech_card_effects_for(Faction::Egypt, tech) {
            assert!(
                !line.contains("Production") || line.contains("cost") || line.contains("Gold"),
                "{tech:?}: Egypt sees {line:?}"
            );
        }
    }
    assert_eq!(
        data::tech_card_effects_for(Faction::Egypt, TechId::Masonry),
        Vec::<&str>::new()
    );
    assert_eq!(
        data::tech_card_effects_for(Faction::India, TechId::Masonry),
        vec!["Quarries upgrade: Deep Quarry (+1 Production)"]
    );
}

#[test]
fn a_new_game_gives_each_faction_its_skipped_techs() {
    let mut setup = GameSetup::all_ai(5);
    setup.players.truncate(3);
    setup.players[0].faction = Faction::Egypt;
    setup.players[1].faction = Faction::India;
    setup.players[2].faction = Faction::Mongols;
    let s = game::try_new_game(&setup).unwrap();
    assert_eq!(
        s.player(P).techs.iter().copied().collect::<Vec<_>>(),
        vec![TechId::BronzeWorking]
    );
    assert!(s.player(PlayerId(1)).techs.is_empty());
    assert!(s.player(PlayerId(2)).techs.is_empty());
    // Egypt's research screen counts four Ancient technologies, without Bronze Working.
    let v = legatum_core::view::player_view(&s, P);
    assert_eq!(v.research.skipped, vec![TechId::BronzeWorking]);
    assert_eq!(
        (v.research.era, v.research.era_total, v.research.era_known),
        (0, 4, 0)
    );
    assert!(
        !v.research
            .available
            .iter()
            .any(|o| o.tech == TechId::BronzeWorking)
    );
    let india = legatum_core::view::player_view(&s, PlayerId(1));
    assert_eq!((india.research.era_total, india.research.era_known), (5, 0));
    // Egypt never researched it, so it gives no one a diffusion discount.
    assert_eq!(
        economy::tech_cost(&s, PlayerId(1), TechId::BronzeWorking).total,
        data::tech_stats(TechId::BronzeWorking).cost
    );
    // Off the Nile, Egypt builds Workshops and researches Bronze Working like everyone else.
    let mut classic = setup.clone();
    classic
        .rule_experiments
        .faction_rules_off
        .insert(legatum_core::experiments::FactionRule::EgyptNile);
    assert!(
        game::try_new_game(&classic)
            .unwrap()
            .player(P)
            .techs
            .is_empty()
    );
}

#[test]
fn research_cards_show_only_what_helps_the_faction() {
    let rules = Default::default();
    for f in Faction::ALL {
        for tech in TechId::ALL {
            // Chivalry now gives Castles and Military Science discounts land units, so even
            // the Aztecs benefit despite having no mounted line.
            let has = data::tech_has_benefit(rules, f, tech);
            // The Nile: Egypt has no Production, so Bronze Working (Workshops) gives it nothing.
            // Industrialization still reveals Coal, which its Industrial-era units need.
            let skipped = f == Faction::Egypt && tech == TechId::BronzeWorking;
            assert_eq!(has, !skipped, "{f:?} {tech:?}");
        }
    }
    assert_eq!(
        data::tech_card_effects(TechId::PrintingPress),
        vec!["+1 Science per Library family", "+1 Gold per Market family"]
    );
    assert_eq!(
        data::tech_card_effects(TechId::FusionPower),
        vec![
            "+5 Production in every city",
            "Factories and Research Labs need no Coal or Oil"
        ]
    );

    let mut s = state(Faction::Byzantium);
    city(&mut s, at(4, 4));
    let unlocks = legatum_core::research::tech_unlocks(&s, P);
    let lab = unlocks
        .iter()
        .find(|u| u.item == building(BuildingKind::Library))
        .unwrap();
    // The Byzantium researches like everyone else: an ordinary Library with its Science.
    assert_eq!(lab.name.as_str(), "Library");
    assert_eq!(lab.yields.science, 3);

    let mut t = state(Faction::Switzerland);
    city(&mut t, at(4, 4));
    let walls = legatum_core::research::tech_unlocks(&t, P)
        .into_iter()
        .find(|u| u.item == building(BuildingKind::Walls))
        .unwrap();
    assert_eq!(
        (walls.name.as_str(), walls.city_strength),
        ("Alpine Redoubt", 15)
    );
    assert!(!data::available_to_faction(
        Faction::India,
        ProductionItem::Project {
            project: IndustryProject::Gold
        }
    ));
}

#[test]
fn the_byzantium_builds_a_library_and_its_relay_uplink_needs_one_like_everyone() {
    let relay = |tile| ProductionItem::Wonder {
        building: BuildingKind::GreatLibrary,
        tile,
    };
    let library = building(BuildingKind::Library);
    assert!(data::available_to_faction(Faction::Byzantium, library));
    assert_eq!(
        data::great_work_requirement(BuildingKind::GreatLibrary),
        Some((BuildingKind::Library, 1))
    );
    for faction in [Faction::Byzantium, Faction::India] {
        let mut s = state(faction);
        let c = city(&mut s, at(4, 4));
        s.player_mut(P).techs.extend(TechId::ALL);
        let blocker = economy::production_blocker(&s, &s.cities[&c], relay(at(5, 4)), true);
        assert_eq!(
            blocker.as_deref(),
            Some("Build in a city with Library"),
            "{faction:?}"
        );
    }
    // The one-per-empire rule still binds the Byzantium (it hung off the requirement once).
    let mut s = state(Faction::Byzantium);
    let first = city(&mut s, at(4, 4));
    let second = city(&mut s, at(12, 4));
    s.player_mut(P).techs.extend(TechId::ALL);
    s.cities.get_mut(&first).unwrap().producing = Some(relay(at(5, 4)));
    let blocker =
        economy::production_blocker(&s, &s.cities[&second], relay(at(13, 4)), true).unwrap();
    assert!(blocker.contains("only one per empire"), "{blocker}");
}

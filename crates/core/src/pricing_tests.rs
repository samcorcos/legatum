//! Prices, growth thresholds, storage caps and timers read straight from the data tables, and old
//! saves and setups that still carry a game speed keep loading.
use crate::data;
use crate::economy;
use crate::game::spawn_unit;
use crate::rules::cities::tests::{P0, P1, at, found, test_game};
use crate::rules::{battle, cities, units};
use crate::*;
use std::collections::BTreeMap;

#[test]
fn walls_price_experiment_reaches_production_and_city_quotes() {
    let mut s = test_game(&[(Faction::India, 0), (Faction::Byzantium, 1)]);
    let city = found(&mut s, P0, at(3, 3));
    s.player_mut(P0).techs.insert(TechId::Masonry);
    let walls = ProductionItem::Building {
        building: BuildingKind::Walls,
    };
    assert_eq!(economy::production_cost(&s, P0, walls), 100);
    assert_eq!(
        s.setup
            .rule_experiments
            .base_production_cost(Faction::Switzerland, walls),
        120
    );
    assert_eq!(
        s.setup.rule_experiments.base_production_cost(
            Faction::India,
            ProductionItem::Building {
                building: BuildingKind::Castle
            }
        ),
        240
    );
    s.setup.rule_experiments.apply("walls-cost=100").unwrap();
    assert!(s.setup.rule_experiments.walls_cost.is_none());
    s.setup.rule_experiments.apply("walls-cost=80").unwrap();
    assert_eq!(economy::production_cost(&s, P0, walls), 80);
    s.cities.get_mut(&city).unwrap().producing = Some(walls);
    let view = crate::view::player_view(&s, P0);
    let detail = view
        .cities
        .iter()
        .find(|c| c.id == city)
        .unwrap()
        .detail
        .as_ref()
        .unwrap();
    let quote = detail.options.iter().find(|o| o.item == walls).unwrap();
    assert_eq!(quote.cost, 80);
    assert_eq!(quote.buy_cost, None); // Walls remain Production-only.
    assert_eq!(economy::buy_cost(&s, &s.cities[&city]), None);
    s.setup.rule_experiments.apply("walls-cost=120").unwrap();
    assert_eq!(economy::production_cost(&s, P0, walls), 120);
    assert_eq!(
        s.setup
            .rule_experiments
            .base_production_cost(Faction::Switzerland, walls),
        140
    );
}

#[test]
fn every_buildable_buys_at_its_faction_percent_of_the_production_cost() {
    for faction in Faction::ALL {
        let mut s = test_game(&[(faction, 0), (Faction::Byzantium, 1)]);
        let city = found(&mut s, P0, at(3, 3));
        s.player_mut(P0).settlers_trained = 3;
        s.player_mut(P0).techs.extend(TechId::ALL);
        s.cities.get_mut(&city).unwrap().population = 8;
        let items = UnitKind::ALL
            .map(|unit| ProductionItem::Unit { unit })
            .into_iter()
            .chain(BuildingKind::ALL.map(|building| ProductionItem::Building { building }));
        for item in items.filter(|item| {
            data::available_to_faction(faction, *item)
                && !matches!(item, ProductionItem::Building { building } if crate::landmarks::is_landmark(*building))
        }) {
            let c = s.cities.get_mut(&city).unwrap();
            c.buildings.clear();
            if let ProductionItem::Building { building } = item
                && let Some(previous) = data::building_predecessor(building)
            {
                c.buildings.insert(previous);
            }
            if item == (ProductionItem::Building {
                building: BuildingKind::ImperialDiwan,
            }) {
                c.buildings.insert(BuildingKind::Market);
            }
            s.cities.get_mut(&city).unwrap().producing = Some(item);
            assert!(economy::production_cost(&s, P0, item) > 0);
            let pct = if faction == Faction::Switzerland {
                175
            } else {
                200
            };
            let buy = economy::buy_cost(&s, &s.cities[&city]);
            if data::is_great_work_item(item) || economy::is_unbuyable(item) {
                assert_eq!(buy, None);
            } else {
                assert_eq!(
                    buy,
                    Some(economy::production_cost(&s, P0, item) * pct / 100),
                    "{faction:?} {item:?}"
                );
            }
        }
    }
}

#[test]
fn research_cost_is_the_discounted_listed_cost_and_the_breakdown_sums_to_it() {
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::India, 1)]);
    s.player_mut(P1).techs.extend(TechId::ALL);
    for tech in TechId::ALL {
        let listed = data::tech_stats(tech).cost;
        let cost = economy::tech_cost(&s, P0, tech);
        assert_eq!(cost.total, listed * 90 / 100, "{tech:?}");
        assert_eq!(cost.parts[0], ("Listed cost".into(), listed));
        assert_eq!(
            cost.parts.iter().map(|part| part.1).sum::<i32>(),
            cost.total
        );
    }
}

#[test]
fn experimental_research_percent_applies_after_diffusion() {
    let mut s = test_game(&[(Faction::India, 0), (Faction::Byzantium, 1)]);
    found(&mut s, P0, at(3, 3));
    s.player_mut(P1).techs.extend(TechId::ALL);
    for tech in TechId::ALL {
        let diffused = data::tech_stats(tech).cost * 90 / 100;
        s.setup.rule_experiments.tech_cost_percent = None;
        assert_eq!(economy::tech_cost(&s, P0, tech).total, diffused);
        s.setup.rule_experiments.tech_cost_percent = Some(100);
        assert_eq!(economy::tech_cost(&s, P0, tech).total, diffused);
        s.setup.rule_experiments.tech_cost_percent = Some(65);
        let cost = economy::tech_cost(&s, P0, tech);
        assert_eq!(cost.total, diffused * 65 / 100, "{tech:?}");
        assert_eq!(
            cost.parts.iter().map(|part| part.1).sum::<i32>(),
            cost.total
        );
    }
}

#[test]
fn growth_borders_idle_storage_and_views_use_the_table_thresholds() {
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::India, 1)]);
    s.setup.turn_limit = None;
    let id = found(&mut s, P0, at(3, 3));
    let yields = economy::city_yields(&s, &s.cities[&id]).total;
    let city = s.cities.get_mut(&id).unwrap();
    // A Settler needs population 2, so the first growth has its own price.
    city.food_stored = data::FIRST_GROWTH - (yields.food - 2) - 1;
    let view = player_view(&s, P0);
    assert_eq!(view.turn_limit, None);
    let detail = view
        .cities
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .detail
        .as_ref()
        .unwrap();
    assert_eq!(detail.growth_threshold, data::FIRST_GROWTH);
    assert_eq!(data::growth_threshold(1), data::FIRST_GROWTH);
    assert_eq!(
        data::growth_threshold(2),
        data::GROWTH_BASE + data::GROWTH_PER_POP
    );
    cities::process_cities(&mut s, &mut Vec::new());
    cities::process_borders(&mut s, &mut Vec::new());
    assert_eq!(s.cities[&id].population, 1);
    assert_eq!(s.cities[&id].tiles_claimed, 0);
    cities::process_cities(&mut s, &mut Vec::new());
    cities::process_borders(&mut s, &mut Vec::new());
    assert_eq!(s.cities[&id].population, 2);
    assert_eq!(s.cities[&id].tiles_claimed, 1);
    for _ in 0..100 {
        cities::process_cities(&mut s, &mut Vec::new());
    }
    assert_eq!(s.cities[&id].production_stored, cities::IDLE_PRODUCTION_CAP);
    assert_eq!(cities::IDLE_PRODUCTION_CAP, 60);
    assert_eq!(
        s.player(P0).science_history.len(),
        data::SCIENCE_HISTORY_TURNS
    );
    assert_eq!(data::SCIENCE_HISTORY_TURNS, 10);
}

#[test]
fn swiss_hoard_upgrade_price_and_capture_resistance_read_the_tables() {
    let mut s = test_game(&[(Faction::Switzerland, 0), (Faction::Byzantium, 1)]);
    let city = found(&mut s, P0, at(3, 3));
    let enemy = found(&mut s, P1, at(10, 10));
    let pos = s.cities[&city].pos;
    let soldier = spawn_unit(&mut s, P0, UnitKind::Phalangite, pos);
    s.player_mut(P0).techs.insert(TechId::Feudalism);
    // Phalangite→Pikeman also uses 1 Iron supply: an Iron deposit inside the border.
    s.player_mut(P0)
        .techs
        .extend([TechId::BronzeWorking, TechId::IronWorking]);
    s.map.get_mut(pos.neighbors()[0]).unwrap().resource = Some(crate::Resource::Iron);
    assert_eq!(units::check_upgrade(&s, P0, soldier, 10000).unwrap().1, 84);
    s.player_mut(P0).gold = crate::goals::SWISS_HOARD_GOLD - 1;
    assert!(!goals::reached(&s, P0));
    s.player_mut(P0).gold += 1;
    assert!(goals::reached(&s, P0));
    battle::capture_city(&mut s, enemy, P0, &mut Vec::new());
    assert_eq!(s.cities[&enemy].resistance, data::CAPTURE_RESISTANCE_TURNS);
    assert_eq!(data::CAPTURE_RESISTANCE_TURNS, 6);
    for _ in 0..data::CAPTURE_RESISTANCE_TURNS {
        assert_eq!(
            economy::city_yields(&s, &s.cities[&enemy]).total,
            Yields::ZERO
        );
        cities::process_borders(&mut s, &mut Vec::new());
    }
    assert_eq!(s.cities[&enemy].resistance, 0);
}

#[test]
fn research_upgrade_preview_matches_gold_charged() {
    for faction in Faction::ALL {
        let mut s = test_game(&[(faction, 0), (Faction::Byzantium, 1)]);
        let pos = at(3, 3);
        found(&mut s, P0, pos);
        let soldier = spawn_unit(&mut s, P0, UnitKind::Phalangite, pos);
        s.player_mut(P0).techs.insert(TechId::Feudalism);
        s.player_mut(P0).gold = 10_000;
        s.player_mut(P0)
            .techs
            .extend([TechId::BronzeWorking, TechId::IronWorking]);
        s.map.get_mut(pos.neighbors()[0]).unwrap().resource = Some(crate::Resource::Iron);
        let advertised = research::tech_unlocks(&s, P0)
            .into_iter()
            .find(|unlock| {
                unlock.item
                    == (ProductionItem::Unit {
                        unit: UnitKind::Pikeman,
                    })
            })
            .unwrap()
            .upgrade_cost
            .unwrap();
        let before = s.player(P0).gold;
        let orders = [Order::UpgradeUnit { unit: soldier }];
        let mut events = Vec::new();
        units::apply_unit_orders(
            &mut s,
            &BTreeMap::from([(P0, orders.as_slice())]),
            &mut events,
        );
        assert_eq!(s.units[&soldier].kind, UnitKind::Pikeman);
        assert_eq!(advertised, before - s.player(P0).gold, "{faction:?}");
        assert!(events.iter().any(|event| matches!(
            event,
            Event::UnitUpgraded { unit, gold, .. }
                if *unit == soldier && *gold == advertised
        )));
    }
}

#[test]
fn no_limit_never_finishes_from_turn_count_but_explicit_limit_still_does() {
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::India, 1)]);
    s.setup.turn_limit = None;
    found(&mut s, P0, at(3, 3));
    found(&mut s, P1, at(10, 10));
    s.cities
        .values_mut()
        .find(|c| c.owner == P0)
        .unwrap()
        .population = 2;
    for turn in [150, 300, 900, 10_000] {
        s.turn = turn;
        scoring::update(&mut s, &mut Vec::new());
        assert!(!s.finished, "ended at {turn}");
    }
    s.setup.turn_limit = Some(10_001);
    scoring::update(&mut s, &mut Vec::new());
    assert!(!s.finished);
    s.turn = 10_001;
    scoring::update(&mut s, &mut Vec::new());
    assert!(s.finished);
    assert_eq!(s.winner, None);
    assert_eq!(scoring::team_target(&s, TeamId(0)), 10);
}

#[test]
fn setup_and_save_serde_accept_legacy_numeric_limits_and_missing_new_fields() {
    let setup = GameSetup::all_ai(42);
    assert_eq!(setup.turn_limit, None);
    assert_eq!(crate::data::EXPECTED_GAME_TURNS, 300);
    let mut json = serde_json::to_value(&setup).unwrap();
    json["turn_limit"] = 150.into();
    let legacy: GameSetup = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(legacy.turn_limit, Some(150));
    json.as_object_mut().unwrap().remove("turn_limit");
    assert_eq!(serde_json::from_value::<GameSetup>(json).unwrap(), setup);
    let state = test_game(&[(Faction::Byzantium, 0), (Faction::India, 1)]);
    let saved = serde_json::to_value(&state).unwrap();
    let loaded: GameState = serde_json::from_value(saved).unwrap();
    assert_eq!(loaded.setup.turn_limit, Some(150));
    let roundtrip: GameState =
        serde_json::from_str(&serde_json::to_string(&loaded).unwrap()).unwrap();
    assert_eq!(loaded, roundtrip);
}

#[test]
fn old_setups_and_saves_with_a_game_speed_still_load_and_drop_it() {
    for speed in ["Standard", "Quick"] {
        let setup = GameSetup::all_ai(42);
        let mut json = serde_json::to_value(&setup).unwrap();
        assert!(
            json.get("speed").is_none(),
            "a setup no longer writes a speed"
        );
        json["speed"] = speed.into();
        let loaded: GameSetup = serde_json::from_value(json).unwrap();
        assert_eq!(loaded, setup, "{speed}");
        assert!(
            serde_json::to_value(&loaded)
                .unwrap()
                .get("speed")
                .is_none()
        );

        let state = test_game(&[(Faction::Byzantium, 0), (Faction::India, 1)]);
        let mut saved = serde_json::to_value(&state).unwrap();
        assert!(saved["setup"].get("speed").is_none());
        saved["setup"]["speed"] = speed.into();
        let loaded: GameState = serde_json::from_value(saved).unwrap();
        assert_eq!(loaded, state, "{speed}");
        let text = serde_json::to_string(&loaded).unwrap();
        assert!(!text.contains("\"speed\""), "{speed}");
        let view = serde_json::to_value(player_view(&loaded, P0)).unwrap();
        assert!(view.get("speed").is_none());
    }
}

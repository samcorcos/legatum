//! Optional balance overrides retain identity requirements and expose what resolves.
use legatum_core::{data, economy, experiments::RuleExperiments, game, rules, state::Map, *};
use std::collections::BTreeMap;

const P: PlayerId = PlayerId(0);

fn state(faction: Faction) -> GameState {
    let mut setup = GameSetup::all_ai(3);
    setup.players.truncate(2);
    setup.players[0].faction = faction;
    game::blank_game(&setup, Map::new(32, 24, Terrain::Grassland))
}

fn city(s: &mut GameState, col: i32, row: i32) -> CityId {
    let settler = game::spawn_unit(s, P, UnitKind::Settler, Hex::from_offset(col, row));
    rules::cities::found_city(s, settler, &mut vec![]).unwrap()
}

#[test]
fn candidate_flags_validate_round_trip_and_leave_default_encoding_and_prices_unchanged() {
    let default = RuleExperiments::default();
    assert_eq!(
        serde_json::to_string(&default).unwrap(),
        r#"{"frontier_cities":false,"market_plunder":false,"wall_masonry":false}"#
    );
    for f in Faction::ALL {
        for unit in UnitKind::ALL {
            assert_eq!(
                default.base_production_cost(f, ProductionItem::Unit { unit }),
                data::unit_stats_for(f, unit).cost
            );
        }
        for building in BuildingKind::ALL {
            assert_eq!(
                default.base_production_cost(f, ProductionItem::Building { building }),
                data::building_stats_for(f, building).cost
            );
        }
    }
    let mut rules = default;
    rules.apply("mongols-commerce-gold=4").unwrap();
    assert_ne!(rules, default);
    assert_eq!(
        serde_json::from_str::<RuleExperiments>(&serde_json::to_string(&rules).unwrap()).unwrap(),
        rules
    );
    for flag in ["mongols-commerce-gold=1", "mongols-commerce-gold=9"] {
        assert!(RuleExperiments::default().apply(flag).is_err(), "{flag}");
    }
}

#[test]
fn sparta_nonmilitary_costs_include_settler_escalation_and_great_works() {
    let mut s = state(Faction::Sparta);
    let home = city(&mut s, 4, 4);
    // A Settler needs population 2 to be quoted, chosen or bought.
    s.cities.get_mut(&home).unwrap().population = 2;
    let settler = ProductionItem::Unit {
        unit: UnitKind::Settler,
    };
    assert_eq!(
        economy::production_cost(&s, P, settler),
        (70 * 3 + 1) / 2 * data::FIRST_SETTLER_PERCENT / 100,
        "the first trained Settler pays the surcharge, then gets the empire discount"
    );
    let first_quote = economy::buy_quote(&s, &s.cities[&home], settler).unwrap();
    assert_eq!(
        first_quote.production.total,
        (70 * 3 + 1) / 2 * data::FIRST_SETTLER_PERCENT / 100
    );
    assert_eq!(
        first_quote.additional_settler_production,
        (70 + 50) * 3 / 2 - first_quote.production.total,
        "the pending price includes both the end of the discount and Sparta's premium"
    );
    s.player_mut(P).settlers_trained = 1;
    assert_eq!(
        economy::production_cost(&s, P, settler),
        (70 + 50) * 3 / 2,
        "the second trained Settler pays the home-economy premium"
    );
    s.player_mut(P).settlers_trained = 2;
    let cost = (70 + 2 * 50) * 3 / 2;
    assert_eq!(economy::production_cost(&s, P, settler), cost);
    for (building, base) in [
        (BuildingKind::Walls, 100),
        (BuildingKind::Castle, 240),
        (BuildingKind::WarShrine, 160),
        (BuildingKind::Market, 90),
    ] {
        assert_eq!(
            economy::production_cost(&s, P, ProductionItem::Building { building }),
            base
        );
    }
    assert!(data::available_to_faction(
        Faction::Sparta,
        ProductionItem::Building {
            building: BuildingKind::WarShrine
        }
    ));
    assert_eq!(
        economy::buy_quote(&s, &s.cities[&home], settler)
            .unwrap()
            .production
            .total,
        cost
    );
    assert_eq!(
        economy::production_cost(
            &s,
            P,
            ProductionItem::Unit {
                unit: UnitKind::Soldier
            }
        ),
        80
    );
    assert_eq!(
        economy::production_cost(
            &s,
            P,
            ProductionItem::Building {
                building: BuildingKind::RelayUplink
            }
        ),
        600
    );
    let c = s.cities.get_mut(&home).unwrap();
    c.population = 5;
    c.producing = Some(settler);
    c.production_stored = cost - 1;
    let (next, _) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(
        next.units_of(P)
            .filter(|u| u.kind == UnitKind::Settler)
            .count(),
        1
    );
    assert_eq!(next.player(P).settlers_trained, 3);
}

#[test]
fn remote_commerce_payoff_resolves_only_with_active_remote_market_and_owned_capital() {
    let mut s = state(Faction::Mongols);
    let home = city(&mut s, 4, 4);
    let remote = city(&mut s, 12, 4);
    s.setup
        .rule_experiments
        .apply("mongols-commerce-gold=4")
        .unwrap();
    let bonus = |s: &GameState| {
        economy::city_yields(s, &s.cities[&remote])
            .sources
            .iter()
            .find(|s| s.label == "Mongols: long-road commerce")
            .map_or(0, |s| s.yields.gold)
    };
    assert_eq!(bonus(&s), 0);
    s.cities
        .get_mut(&remote)
        .unwrap()
        .buildings
        .insert(BuildingKind::Clearinghouse);
    assert_eq!(bonus(&s), 4);
    let income = economy::player_income(&s, P).gold.total;
    let (next, _) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(next.player(P).gold - s.player(P).gold, income);
    s.cities.get_mut(&remote).unwrap().resistance = 1;
    assert_eq!(bonus(&s), 0);
    s.cities.get_mut(&remote).unwrap().resistance = 0;
    s.cities.get_mut(&remote).unwrap().pos = Hex::from_offset(11, 4);
    assert_eq!(bonus(&s), 0);
    s.cities.get_mut(&remote).unwrap().pos = Hex::from_offset(12, 4);
    s.cities.get_mut(&home).unwrap().owner = PlayerId(1);
    assert_eq!(bonus(&s), 0);
}

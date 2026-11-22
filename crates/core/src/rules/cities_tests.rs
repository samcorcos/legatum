use super::*;
use crate::game::{blank_game, spawn_unit};
use crate::state::{GameSetup, Map, PlayerSetup};

/// A 20x20 grassland game with the given (faction, team) per player.
pub(crate) fn test_game(players: &[(Faction, u8)]) -> GameState {
    let setup = GameSetup {
        seed: 7,
        width: 20,
        height: 20,
        players: players
            .iter()
            .enumerate()
            .map(|(i, (faction, team))| PlayerSetup {
                name: format!("P{i}"),
                faction: *faction,
                team: TeamId(*team),
                is_human: false,
                difficulty: Default::default(),
                search: false,
            })
            .collect(),
        turn_limit: Some(150),
        legacy_all_war: false,
        ai_alliances: false,
        barbarians: false,
        rule_experiments: Default::default(),
    };
    let mut state = blank_game(&setup, Map::new(20, 20, Terrain::Grassland));
    // These city/combat fixtures predate diplomacy and explicitly start at War.
    for r in &mut state.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::War;
    }
    state
}

/// Two Athens empires (no terrain bonuses, no unit cap) on separate teams.
pub(crate) fn two_players() -> GameState {
    test_game(&[(Faction::Athens, 0), (Faction::Athens, 1)])
}

pub(crate) fn at(col: i32, row: i32) -> Hex {
    Hex::from_offset(col, row)
}

pub(crate) const P0: PlayerId = PlayerId(0);
pub(crate) const P1: PlayerId = PlayerId(1);
pub(crate) const P2: PlayerId = PlayerId(2);

#[test]
fn gold_income_caps_without_overflow_after_many_turns() {
    let mut state = two_players();
    state.player_mut(P0).gold = crate::validation::MAX_GOLD - 1;
    for _ in 0..1_000 {
        pay_gold(&mut state, P0, i64::from(i32::MAX), &mut Vec::new());
        assert_eq!(state.player(P0).gold, crate::validation::MAX_GOLD);
    }
}

/// Found a city through the real rule.
pub(crate) fn found(state: &mut GameState, owner: PlayerId, pos: Hex) -> CityId {
    let u = spawn_unit(state, owner, UnitKind::Settler, pos);
    found_city(state, u, &mut Vec::new()).expect("legal city site")
}

fn spearman() -> ProductionItem {
    ProductionItem::Unit {
        unit: UnitKind::Spearman,
    }
}

fn settler() -> ProductionItem {
    ProductionItem::Unit {
        unit: UnitKind::Settler,
    }
}

fn building(b: BuildingKind) -> ProductionItem {
    ProductionItem::Building { building: b }
}

/// An online guest's order: the pick is stored on the city unchecked, and a tile off the map there made
/// every later save fail `check_state` (security review, 2026-11-22).
#[test]
fn an_automation_pick_off_the_map_is_refused_and_the_state_stays_loadable() {
    let mut state = test_game(&[(Faction::Athens, 0)]);
    let id = found(&mut state, P0, at(5, 5));
    for picked in [
        ProductionItem::Wonder {
            building: BuildingKind::Pyramids,
            tile: at(-9, 0),
        },
        ProductionItem::Improvement {
            improvement: ImprovementKind::Farm,
            tile: at(500, 500),
        },
    ] {
        let order = Order::SetCityAutomation {
            city: id,
            focus: Some(CityBuildFocus::Military),
            picked: Some(picked),
        };
        let errors = validate_city_orders(&state, P0, std::slice::from_ref(&order));
        assert_eq!(errors.len(), 1, "{picked:?}");
        let (next, _) = crate::resolve_turn(&state, &BTreeMap::from([(P0, vec![order])]));
        assert_eq!(city(&next, id).last_auto_pick, None);
        assert_eq!(crate::validation::check_state(&next), Ok(()));
    }
}

#[test]
fn stopping_city_automation_keeps_current_build_and_drops_its_future_pick() {
    let mut state = test_game(&[(Faction::Athens, 0)]);
    let id = found(&mut state, P0, at(5, 5));
    let scout = ProductionItem::Unit {
        unit: UnitKind::Scout,
    };
    let orders = [
        Order::SetProductionQueue {
            city: id,
            queue: vec![spearman(), scout],
        },
        Order::SetCityAutomation {
            city: id,
            focus: Some(CityBuildFocus::Military),
            picked: Some(scout),
        },
    ];
    assert!(validate_city_orders(&state, P0, &orders).is_empty());
    apply_city_orders(&mut state, P0, &orders, &mut Vec::new());
    assert_eq!(city(&state, id).production_queue, vec![scout]);
    assert_eq!(city(&state, id).auto_queued, Some(scout));
    let mut changed = state.clone();
    apply_city_orders(
        &mut changed,
        P0,
        &[Order::SetCityAutomation {
            city: id,
            focus: Some(CityBuildFocus::Gold),
            picked: None,
        }],
        &mut Vec::new(),
    );
    assert_eq!(city(&changed, id).producing, Some(spearman()));
    assert!(
        city(&changed, id).production_queue.is_empty(),
        "a new focus replaces the old automatic tail"
    );
    apply_city_orders(
        &mut state,
        P0,
        &[Order::SetCityAutomation {
            city: id,
            focus: None,
            picked: None,
        }],
        &mut Vec::new(),
    );
    assert_eq!(city(&state, id).producing, Some(spearman()));
    assert!(city(&state, id).production_queue.is_empty());
    assert_eq!(city(&state, id).build_automation, None);
    assert_eq!(city(&state, id).last_auto_pick, None);
    apply_city_orders(
        &mut state,
        P0,
        &[
            Order::SetCityAutomation {
                city: id,
                focus: Some(CityBuildFocus::Balanced),
                picked: None,
            },
            Order::SetProductionQueue {
                city: id,
                queue: vec![spearman(), scout],
            },
        ],
        &mut Vec::new(),
    );
    apply_city_orders(
        &mut state,
        P0,
        &[Order::SetCityAutomation {
            city: id,
            focus: None,
            picked: None,
        }],
        &mut Vec::new(),
    );
    assert_eq!(
        city(&state, id).production_queue,
        vec![scout],
        "a player's queued item stays"
    );
}

#[test]
fn walls_line_cannot_be_bought_but_finishes_with_production() {
    let cases = [
        (Faction::India, BuildingKind::Walls),
        (Faction::India, BuildingKind::Castle),
        (Faction::India, BuildingKind::StarFort),
        // The Swiss Alpine Redoubt is the Switzerland' Walls replacement.
        (Faction::Switzerland, BuildingKind::Walls),
    ];
    for (faction, kind) in cases {
        let mut s = test_game(&[(faction, 0)]);
        let id = found(&mut s, P0, at(5, 5));
        s.player_mut(P0).techs.extend([
            TechId::Masonry,
            TechId::Feudalism,
            TechId::MilitaryEngineering,
        ]);
        s.player_mut(P0).gold = 10_000;
        let item = building(kind);
        assert_eq!(economy::buy_cost_for(&s, city(&s, id), item), None);
        assert_eq!(
            economy::buy_block(&s, city(&s, id), item),
            Some(economy::BuyBlock::WallsNeedProduction)
        );
        assert_eq!(
            economy::buy_blocker(&s, city(&s, id), item).as_deref(),
            Some("Walls must be built with Production; they can't be bought.")
        );
        assert!(
            validate_city_order(&s, P0, &Order::BuyItem { city: id, item })
                .unwrap_err()
                .contains("Walls must be built with Production; they can't be bought.")
        );
        assert!(economy::production_cost(&s, P0, item) > 0);
    }

    let mut s = test_game(&[(Faction::India, 0)]);
    let id = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).techs.insert(TechId::Masonry);
    let walls = building(BuildingKind::Walls);
    let mut events = Vec::new();
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetProduction {
            city: id,
            item: walls,
        }],
        &mut events,
    );
    city_mut(&mut s, id).production_stored = 10;
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetProduction {
            city: id,
            item: spearman(),
        }],
        &mut events,
    );
    assert_eq!(
        city(&s, id)
            .production_banked
            .iter()
            .find(|bank| bank.item == walls)
            .unwrap()
            .progress,
        10
    );
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetProduction {
            city: id,
            item: walls,
        }],
        &mut events,
    );
    assert_eq!(city(&s, id).production_stored, 10);
    let cost = economy::production_cost(&s, P0, walls);
    city_mut(&mut s, id).production_stored = cost;
    city_turn(&mut s);
    assert!(data::has_building(
        city(&s, id).buildings.iter(),
        BuildingKind::Walls
    ));
}

#[test]
fn mongols_cannot_produce_engineers_walls_castles_or_star_forts_before_the_medieval_era() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::India, 1)]);
    let city = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).techs.extend([
        TechId::Masonry,
        TechId::Feudalism,
        TechId::Chivalry,
        TechId::Mathematics,
        TechId::Machinery,
    ]);
    for item in [
        building(BuildingKind::Walls),
        building(BuildingKind::Castle),
        building(BuildingKind::StarFort),
    ] {
        assert!(data::available_to_faction(Faction::Mongols, item));
        assert!(!data::available_to_player(
            Faction::Mongols,
            s.era_of(P0),
            item
        ));
        assert!(!economy::can_produce(&s, &s.cities[&city], item));
        let view = crate::view::player_view(&s, P0);
        let detail = view
            .cities
            .iter()
            .find(|c| c.id == city)
            .unwrap()
            .detail
            .as_ref()
            .unwrap();
        assert!(!detail.options.iter().any(|o| o.item == item));
        let unavailable = detail
            .unavailable
            .iter()
            .find(|option| option.item == item)
            .unwrap();
        assert!(unavailable.reason.contains("Mongol"));
    }
}

#[test]
fn india_may_found_three_hexes_from_another_city_but_other_factions_may_not() {
    let site = at(8, 5);
    let mut india = test_game(&[(Faction::India, 0)]);
    found(&mut india, P0, at(5, 5));
    let settler = spawn_unit(&mut india, P0, UnitKind::Settler, site);
    assert!(check_found_city(&india, settler).is_ok());

    let mut other = test_game(&[(Faction::Athens, 0)]);
    found(&mut other, P0, at(5, 5));
    let settler = spawn_unit(&mut other, P0, UnitKind::Settler, site);
    assert!(
        check_found_city(&other, settler)
            .unwrap_err()
            .contains("cities must be at least 4 tiles apart")
    );
}

#[test]
fn sparta_agoge_promotes_trained_military() {
    let mut s = test_game(&[(Faction::Sparta, 0), (Faction::India, 1)]);
    let c = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).techs.insert(TechId::Chivalry);
    assert!(economy::can_produce(
        &s,
        &s.cities[&c],
        building(BuildingKind::Agoge)
    ));
    let mut events = Vec::new();
    assert!(complete_item(
        &mut s,
        c,
        building(BuildingKind::Agoge),
        &mut events
    ));
    assert!(complete_item(&mut s, c, spearman(), &mut events));
    let trained = s
        .units_of(P0)
        .find(|u| u.kind == UnitKind::Spearman)
        .unwrap();
    assert_eq!(
        crate::rules::promotion::level_for(Faction::Sparta, trained.xp),
        2
    );
    assert_eq!(trained.xp, crate::rules::promotion::threshold(2).unwrap());
}

fn city(state: &GameState, id: CityId) -> &City {
    &state.cities[&id]
}

fn city_mut(state: &mut GameState, id: CityId) -> &mut City {
    state.cities.get_mut(&id).unwrap()
}

/// One full city cycle: phases 6-9 in order.
fn city_turn(state: &mut GameState) -> Vec<Event> {
    let mut ev = Vec::new();
    process_cities(state, &mut ev);
    process_research(state, &mut ev);
    process_borders(state, &mut ev);
    ev
}

#[test]
fn a_foreign_unit_does_not_block_a_worked_tile() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::Byzantium, 1)]);
    for relation in &mut s.diplomacy.as_mut().unwrap().relations {
        relation.state = crate::diplomacy::RelationState::War;
    }
    let city = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, city).population = 2;
    let tile = at(6, 5);
    s.map.get_mut(tile).unwrap().resource = Some(crate::resources::Resource::Wheat);
    let before = economy::city_yields(&s, &s.cities[&city]);
    assert!(before.worked.contains(&tile));
    spawn_unit(&mut s, P1, UnitKind::Spearman, tile);
    let after = economy::city_yields(&s, &s.cities[&city]);
    assert!(after.worked.contains(&tile));
    assert_eq!(before.total, after.total);
}

#[test]
fn celts_raze_removes_conquered_territory() {
    let mut s = test_game(&[(Faction::Celts, 0), (Faction::Mongols, 1)]);
    let home = found(&mut s, P0, at(5, 5));
    let foreign = found(&mut s, P1, at(12, 5));
    let intruder = spawn_unit(&mut s, P1, UnitKind::Spearman, at(6, 5));
    s.units.get_mut(&intruder).unwrap().hp = 15;
    let (mut next, _) = crate::resolve_turn(&s, &BTreeMap::new());
    assert!(next.units[&intruder].hp >= 15);
    assert!(next.cities.contains_key(&home));
    let at = next.cities[&foreign].pos;
    let mut report = Vec::new();
    crate::rules::battle::capture_city(&mut next, foreign, P0, &mut report);
    assert!(!next.cities.contains_key(&foreign));
    assert_eq!(next.map.get(at).unwrap().owner, None);
    assert!(
        report
            .iter()
            .any(|event| matches!(event, Event::CityRazed { city, .. } if *city == foreign))
    );
}

#[test]
fn captured_city_raze_pays_loot_releases_land_and_preserves_celtic_sack() {
    let mut s = two_players();
    found(&mut s, P0, at(3, 3));
    found(&mut s, P1, at(16, 16));
    let target = found(&mut s, P1, at(11, 5));
    let tile = s.cities[&target].pos;
    s.cities.get_mut(&target).unwrap().population = 4;
    s.map.get_mut(tile).unwrap().improved = true;
    crate::rules::battle::capture_city(&mut s, target, P0, &mut Vec::new());
    let order = Order::Raze { city: target };
    assert!(raze_blocker(&s, P0, target).unwrap().contains("turn after"));
    s.cities.get_mut(&target).unwrap().resistance = 5;
    let pop = s.cities[&target].population;
    assert!(validate_city_order(&s, P0, &order).is_ok());
    let view = crate::view::player_view(&s, P0);
    let detail = view
        .cities
        .iter()
        .find(|c| c.id == target)
        .unwrap()
        .detail
        .as_ref()
        .unwrap();
    assert!(detail.captured);
    assert!(detail.raze_blocker.is_none());
    assert_eq!(
        validate_city_orders(
            &s,
            P0,
            &[
                Order::SetProduction {
                    city: target,
                    item: spearman(),
                },
                order.clone()
            ]
        )
        .iter()
        .map(|e| e.index)
        .collect::<Vec<_>>(),
        vec![1]
    );
    let (next, events) = crate::resolve_turn(&s, &BTreeMap::from([(P0, vec![order])]));
    let (control, _) = crate::resolve_turn(&s, &BTreeMap::new());
    assert!(!next.cities.contains_key(&target));
    assert_eq!(next.player(P0).gold, control.player(P0).gold + pop * 10);
    assert_eq!(next.map.get(tile).unwrap().owner, None);
    assert!(!next.map.get(tile).unwrap().improved);
    assert!(events.iter().any(|e| matches!(e, Event::CityRazed {
        city, sack_gold: Some(g), ordered: true, ..
    } if *city == target && *g == pop * 10)));
}

#[test]
fn raze_allows_original_capitals_and_goal_buildings() {
    let mut s = two_players();
    let home = found(&mut s, P0, at(3, 3));
    let capital = found(&mut s, P1, at(16, 16));
    let target = found(&mut s, P1, at(11, 5));
    assert!(
        raze_blocker(&s, P0, home)
            .unwrap()
            .contains("founded by you")
    );
    crate::rules::battle::capture_city(&mut s, capital, P0, &mut Vec::new());
    s.cities.get_mut(&capital).unwrap().resistance = 0;
    assert!(raze_blocker(&s, P0, capital).is_none());
    crate::rules::battle::capture_city(&mut s, target, P0, &mut Vec::new());
    s.cities.get_mut(&target).unwrap().resistance = 0;
    s.cities
        .get_mut(&target)
        .unwrap()
        .buildings
        .insert(BuildingKind::GreatLibrary);
    assert!(raze_blocker(&s, P0, target).is_none());
    s.cities.get_mut(&target).unwrap().buildings.clear();
    s.cities
        .get_mut(&target)
        .unwrap()
        .buildings
        .insert(BuildingKind::YamRelayStation);
    assert!(raze_blocker(&s, P0, target).is_none());
    // The captor holds the capital's point until it razes the city: razing gives it up.
    assert_eq!(crate::scoring::capitals_held(&s, P0).len(), 1);
    let (next, events) = crate::resolve_turn(
        &s,
        &BTreeMap::from([(P0, vec![Order::Raze { city: capital }])]),
    );
    assert!(!next.cities.contains_key(&capital));
    assert!(crate::scoring::capitals_held(&next, P0).is_empty());
    assert!(events.iter().any(|e| matches!(e,
        Event::CapitalLost { player, capital: id, .. } if *player == P0 && *id == capital)));
}

#[test]
fn razing_a_great_work_city_destroys_its_site() {
    let mut s = two_players();
    found(&mut s, P0, at(3, 3));
    found(&mut s, P1, at(16, 16));
    let target = found(&mut s, P1, at(10, 5));
    let site = at(11, 5);
    s.cities
        .get_mut(&target)
        .unwrap()
        .buildings
        .insert(BuildingKind::GreatLibrary);
    crate::landmarks::complete(&mut s, target, BuildingKind::GreatLibrary, site);
    crate::rules::battle::capture_city(&mut s, target, P0, &mut Vec::new());
    s.cities.get_mut(&target).unwrap().resistance = 0;
    assert_eq!(s.map.get(site).unwrap().owner, Some(P0));
    assert!(raze_blocker(&s, P0, target).is_none());
    let (next, _) = crate::resolve_turn(
        &s,
        &BTreeMap::from([(P0, vec![Order::Raze { city: target }])]),
    );
    assert!(!next.cities.contains_key(&target));
    assert_eq!(next.map.get(site).unwrap().landmark, None);
    assert_eq!(next.map.get(site).unwrap().owner, None);
}

#[test]
fn replacement_after_original_capital_is_destroyed_is_not_original() {
    let mut s = two_players();
    let original = found(&mut s, P0, at(3, 3));
    let name = s.cities[&original].name.clone();
    s.history
        .chronicle
        .push(crate::history::Chronicle::Founded {
            turn: s.cities[&original].founded_turn,
            player: P0,
            city: name,
        });
    s.cities.remove(&original);
    let replacement = found(&mut s, P0, at(10, 10));
    assert!(s.cities[&replacement].is_capital);
    assert!(!s.cities[&replacement].original_capital);
}

// ---------- orders ----------

#[test]
fn set_production_switch_banks_progress_and_checks_tech() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).producing = Some(spearman());
    city_mut(&mut s, c).production_stored = 12;

    let mut ev = Vec::new();
    let scout = ProductionItem::Unit {
        unit: UnitKind::Scout,
    };
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetProduction {
            city: c,
            item: scout,
        }],
        &mut ev,
    );
    assert_eq!(city(&s, c).producing, Some(scout));
    // The Spearman's progress stays banked with it; the Scout starts from its own (none).
    assert_eq!(city(&s, c).production_stored, 0);
    assert_eq!(city(&s, c).banked(spearman()), 12);

    // A Library needs Writing: refused (and an order for it leaves the Scout building).
    let market = Order::SetProduction {
        city: c,
        item: building(BuildingKind::Library),
    };
    assert!(validate_city_order(&s, P0, &market).is_err());
    apply_city_orders(&mut s, P0, std::slice::from_ref(&market), &mut ev);
    assert_eq!(city(&s, c).producing, Some(scout));
    assert!(matches!(ev.last(), Some(Event::OrderFailed { .. })));

    // The Granary needs no tech any more.
    let granary = Order::SetProduction {
        city: c,
        item: building(BuildingKind::Granary),
    };
    assert!(validate_city_order(&s, P0, &granary).is_ok());
    s.player_mut(P0).techs.insert(TechId::Writing);
    assert!(validate_city_order(&s, P0, &market).is_ok());
    // Another empire can't touch this city.
    assert!(validate_city_order(&s, P1, &market).is_err());
}

#[test]
fn megaproject_start_is_announced_once() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).techs.insert(TechId::Philosophy);
    city_mut(&mut s, c).buildings.insert(BuildingKind::Library);
    let order = Order::SetProduction {
        city: c,
        item: ProductionItem::Wonder {
            building: BuildingKind::GreatLibrary,
            tile: at(6, 5),
        },
    };
    let mut ev = Vec::new();
    apply_city_orders(&mut s, P0, std::slice::from_ref(&order), &mut ev);
    apply_city_orders(&mut s, P0, std::slice::from_ref(&order), &mut ev);
    let started = ev
        .iter()
        .filter(|e| matches!(e, Event::MegaprojectStarted { .. }))
        .count();
    assert_eq!(started, 1);
}

#[test]
fn buy_any_item_preserves_construction_and_spends_full_price_once() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    let market = building(BuildingKind::Market);
    s.player_mut(P0).techs.extend(TechId::ALL);
    s.player_mut(P0).gold = 400;
    city_mut(&mut s, c).producing = Some(market);
    city_mut(&mut s, c).production_stored = 10;
    let buy = Order::BuyItem {
        city: c,
        item: spearman(),
    };
    let mut events = Vec::new();
    apply_city_orders(&mut s, P0, &[buy.clone(), buy.clone()], &mut events);
    assert_eq!(s.player(P0).gold, 256);
    assert_eq!(city(&s, c).producing, Some(market));
    assert_eq!(city(&s, c).production_stored, 10);
    assert_eq!(s.units_at(at(5, 5)).next().unwrap().moves_left, 0);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::ItemBought { gold: 144, .. }))
            .count(),
        1
    );
    assert!(matches!(events.last(), Some(Event::OrderFailed { .. })));
    // Also enforced across calls and save/load, not only within one order list.
    let saved = serde_json::to_string(&s).unwrap();
    let mut s: GameState = serde_json::from_str(&saved).unwrap();
    assert!(
        validate_city_order(&s, P0, &buy)
            .unwrap_err()
            .contains("already bought")
    );
    city_turn(&mut s);
    assert_eq!(city(&s, c).producing, Some(market));
    assert!(city(&s, c).production_stored > 10);
}

#[test]
fn buying_buildings_installs_now_and_keeps_other_builds_and_progress() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).gold = 1000;
    let monument = building(BuildingKind::Granary);
    city_mut(&mut s, c).producing = Some(spearman());
    city_mut(&mut s, c).production_stored = 100;
    apply_city_orders(
        &mut s,
        P0,
        &[Order::BuyItem {
            city: c,
            item: monument,
        }],
        &mut Vec::new(),
    );
    assert!(city(&s, c).buildings.contains(&BuildingKind::Granary));
    assert_eq!(city(&s, c).producing, Some(spearman()));
    assert_eq!(city(&s, c).production_stored, 100);
    assert_eq!(
        s.player(P0).gold,
        1000 - 2 * economy::production_cost(&s, P0, monument)
    );
}

#[test]
fn buy_requires_gold_population_and_spawn_room_before_movement() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).gold = 1000;
    let buy = Order::BuyItem {
        city: c,
        item: settler(),
    };
    assert!(
        validate_city_order(&s, P0, &buy)
            .unwrap_err()
            .contains("population")
    );
    city_mut(&mut s, c).population = 2;
    for pos in at(5, 5).within(1) {
        spawn_unit(&mut s, P0, UnitKind::Settler, pos);
    }
    assert!(
        validate_city_order(&s, P0, &buy)
            .unwrap_err()
            .contains("spawn")
    );
    let mut events = Vec::new();
    apply_city_orders(&mut s, P0, &[buy], &mut events);
    assert_eq!(s.player(P0).gold, 1000);
    s.units.clear();
    s.player_mut(P0).gold = 159;
    assert!(
        validate_city_order(
            &s,
            P0,
            &Order::BuyItem {
                city: c,
                item: spearman()
            }
        )
        .unwrap_err()
        .contains("160")
    );
    s.player_mut(P0).gold = 160;
    assert!(
        validate_city_order(
            &s,
            P0,
            &Order::BuyItem {
                city: c,
                item: spearman()
            }
        )
        .is_ok(),
        "idle cities can buy"
    );
}

#[test]
fn purchased_settlers_escalate_sequentially_and_do_not_fund_production() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    let b = found(&mut s, P0, at(12, 12));
    s.player_mut(P0).gold = 70 * 2 + 120 * 2; // the first Settler is full price
    for id in [a, b] {
        city_mut(&mut s, id).population = 2;
    }
    let orders = [
        Order::BuyItem {
            city: a,
            item: settler(),
        },
        Order::BuyItem {
            city: b,
            item: settler(),
        },
    ];
    assert!(validate_city_orders(&s, P0, &orders).is_empty());
    apply_city_orders(&mut s, P0, &orders, &mut Vec::new());
    assert_eq!(s.player(P0).gold, 0); // 70*2 + 120*2
    assert_eq!(s.player(P0).settlers_trained, 2);
    for id in [a, b] {
        assert_eq!(city(&s, id).population, 1);
        assert_eq!(city(&s, id).production_stored, 0);
    }
}

#[test]
fn queues_advance_with_overflow_and_roundtrip_and_old_saves_default_empty() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    let monument = building(BuildingKind::Granary);
    let queue = vec![monument, spearman(), spearman()];
    let order = Order::SetProductionQueue { city: c, queue };
    assert!(validate_city_order(&s, P0, &order).is_ok());
    apply_city_orders(&mut s, P0, &[order], &mut Vec::new());
    let cost = economy::production_cost(&s, P0, monument);
    city_mut(&mut s, c).production_stored = cost + 50;
    let mut events = Vec::new();
    produce(&mut s, c, 3, &mut events);
    assert_eq!(city(&s, c).producing, Some(spearman()));
    assert_eq!(city(&s, c).production_queue, vec![spearman()]);
    assert_eq!(city(&s, c).production_stored, 53);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::ProductionCompleted { .. }))
            .count(),
        1
    );
    let save = serde_json::to_string(&s).unwrap();
    let restored: GameState = serde_json::from_str(&save).unwrap();
    assert_eq!(s, restored);
    let mut old = serde_json::to_value(city(&s, c)).unwrap();
    old.as_object_mut().unwrap().remove("production_queue");
    old.as_object_mut().unwrap().remove("last_purchase_turn");
    let old: City = serde_json::from_value(old).unwrap();
    assert!(old.production_queue.is_empty());
    assert_eq!(old.last_purchase_turn, None);
    city_mut(&mut s, c).production_stored += 40;
    produce(&mut s, c, 3, &mut events);
    assert_eq!(city(&s, c).producing, Some(spearman()));
    assert!(city(&s, c).production_queue.is_empty());
    assert_eq!(city(&s, c).production_stored, 16);
}

#[test]
fn queue_validation_reorder_remove_duplicates_unlocks_and_ownership() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    let monument = building(BuildingKind::Granary);
    let queue = |items| Order::SetProductionQueue {
        city: c,
        queue: items,
    };
    assert!(validate_city_order(&s, P1, &queue(vec![spearman()])).is_err());
    assert!(validate_city_order(&s, P0, &queue(vec![monument, monument])).is_err());
    assert!(validate_city_order(&s, P0, &queue(vec![spearman(); 51])).is_err());
    city_mut(&mut s, c).production_stored = 12;
    apply_city_orders(
        &mut s,
        P0,
        &[
            queue(vec![spearman(), monument]),
            queue(vec![monument, spearman()]),
        ],
        &mut Vec::new(),
    );
    assert_eq!(city(&s, c).producing, Some(monument));
    assert_eq!(city(&s, c).production_queue, vec![spearman()]);
    // Unassigned Production joined the first item started; the switch banked it there.
    assert_eq!(city(&s, c).production_stored, 0);
    assert_eq!(city(&s, c).banked(spearman()), 12);
    apply_city_orders(&mut s, P0, &[queue(vec![])], &mut Vec::new());
    assert_eq!(city(&s, c).producing, None);
    assert_eq!(city(&s, c).production_stored, 0);
    assert_eq!(city(&s, c).banked(spearman()), 12);
}

#[test]
fn accepted_purchase_items_and_prices_match_resolution_across_order_permutations() {
    fn permutations(orders: &[Order]) -> Vec<Vec<Order>> {
        if orders.is_empty() {
            return vec![Vec::new()];
        }
        let mut result = Vec::new();
        for (i, order) in orders.iter().enumerate() {
            let mut rest = orders.to_vec();
            rest.remove(i);
            for mut tail in permutations(&rest) {
                tail.insert(0, order.clone());
                result.push(tail);
            }
        }
        result
    }

    // Read the validator's quote at its affordability boundary, without using the
    // resolver or duplicating the implementation of purchase projection.
    fn minimum_gold(s: &GameState, orders: &[Order]) -> i32 {
        let mut probe = s.clone();
        let (mut low, mut high) = (0, s.player(P0).gold);
        while low < high {
            let gold = (low + high) / 2;
            probe.player_mut(P0).gold = gold;
            if crate::rules::validate_orders(&probe, P0, orders).is_empty() {
                high = gold;
            } else {
                low = gold + 1;
            }
        }
        low
    }

    let mut accepted_buys = 0;
    let mut rejected_orders = 0;
    for faction in [Faction::Athens, Faction::Switzerland, Faction::Byzantium] {
        for item in [
            spearman(),
            settler(),
            building(BuildingKind::Workshop),
            building(BuildingKind::Granary),
        ] {
            let mut s = test_game(&[(faction, 0), (Faction::India, 1)]);
            s.player_mut(P0).techs.extend(TechId::ALL);
            s.player_mut(P0).gold = 10_000;
            let a = found(&mut s, P0, at(5, 5));
            let b = found(&mut s, P0, at(12, 12));
            for id in [a, b] {
                let c = city_mut(&mut s, id);
                c.population = 5;
                c.food_stored = 100;
                c.production_stored = 15;
                c.producing = Some(item);
            }
            let candidates = [
                Order::BuyItem { city: a, item },
                Order::SetProduction {
                    city: a,
                    item: spearman(),
                },
                Order::SetProduction { city: b, item },
                Order::BuyItem { city: b, item },
            ];
            for orders in permutations(&candidates) {
                let context = format!("{faction:?} {orders:?}");
                let before = s.clone();
                let errors = crate::rules::validate_orders(&s, P0, &orders);
                assert_eq!(errors, validate_city_orders(&s, P0, &orders), "{context}");
                assert_eq!(s, before, "{context}: validation is read-only");
                let accepted: Vec<_> = orders
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| errors.iter().all(|e| e.index != *i))
                    .map(|(_, o)| o.clone())
                    .collect();
                assert!(
                    crate::rules::validate_orders(&s, P0, &accepted).is_empty(),
                    "{context}"
                );
                let mut producing: BTreeMap<_, _> =
                    s.cities.iter().map(|(id, c)| (*id, c.producing)).collect();
                let mut expected = Vec::new();
                let mut spent = 0;
                for (i, order) in accepted.iter().enumerate() {
                    match *order {
                        Order::SetProduction { city, item } => {
                            producing.insert(city, Some(item));
                        }
                        Order::BuyItem { city, item } => {
                            let total = minimum_gold(&s, &accepted[..=i]);
                            expected.push(Event::ItemBought {
                                city,
                                owner: P0,
                                item,
                                gold: total - spent,
                            });
                            spent = total;
                        }
                        _ => unreachable!(),
                    }
                }
                let (_, events) = crate::rules::resolve_turn(&s, &BTreeMap::from([(P0, orders)]));
                assert_eq!(
                    events
                        .iter()
                        .filter(|e| matches!(e, Event::ItemBought { .. }))
                        .cloned()
                        .collect::<Vec<_>>(),
                    expected,
                    "{context}"
                );
                assert_eq!(
                    events
                        .iter()
                        .filter_map(|e| match e {
                            Event::OrderFailed { reason, .. } => Some(reason),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    errors.iter().map(|e| &e.message).collect::<Vec<_>>(),
                    "{context}"
                );
                accepted_buys += expected.len();
                rejected_orders += errors.len();
            }
        }
    }
    assert!(accepted_buys > 300);
    assert!(rejected_orders > 0);
}

#[test]
fn set_research() {
    let mut s = two_players();
    found(&mut s, P0, at(5, 5));
    let mut ev = Vec::new();
    // A later era's technology is gated while the current era has unknown techs.
    assert!(
        validate_city_order(
            &s,
            P0,
            &Order::SetResearch {
                tech: TechId::Mathematics
            }
        )
        .is_err()
    );
    assert!(
        validate_city_order(
            &s,
            P0,
            &Order::SetResearch {
                tech: TechId::Currency
            }
        )
        .is_ok(),
        "no prerequisites inside the era"
    );
    apply_city_orders(
        &mut s,
        P0,
        &[
            Order::SetResearch {
                tech: TechId::Mathematics,
            },
            Order::SetResearch {
                tech: TechId::BronzeWorking,
            },
        ],
        &mut ev,
    );
    assert_eq!(s.player(P0).researching, Some(TechId::BronzeWorking));
    assert_eq!(ev.len(), 1, "only the Mathematics order failed: {ev:?}");
}

// ---------- founding ----------

#[test]
fn found_city_creates_capital_and_claims_territory() {
    let mut s = two_players();
    s.turn = 4;
    let u = spawn_unit(&mut s, P0, UnitKind::Settler, at(5, 5));
    let mut ev = Vec::new();
    let c = found_city(&mut s, u, &mut ev).unwrap();
    assert!(!s.units.contains_key(&u), "settler consumed");
    let city = city(&s, c);
    assert_eq!(city.population, 1);
    assert_eq!(city.hp, CITY_HP);
    assert!(city.is_capital);
    assert_eq!(city.founder, P0);
    assert_eq!(city.founded_turn, 4);
    assert_eq!(
        economy::auto_focus(city, city.producing),
        CityFocus::Balanced
    );
    for h in at(5, 5).within(1) {
        assert_eq!(s.map.get(h).unwrap().owner, Some(P0));
        assert_eq!(s.map.get(h).unwrap().city, Some(c));
    }
    assert!(matches!(ev[0], Event::CityFounded { city, .. } if city == c));

    let second = found(&mut s, P0, at(10, 5));
    assert!(!s.cities[&second].is_capital);
    // Another empire's first city is its own capital.
    let other = found(&mut s, P1, at(5, 12));
    assert!(s.cities[&other].is_capital);
}

#[test]
fn founding_a_city_at_war_moves_no_unit() {
    // Playtest 8.9.6, T155: "i built a city while at war, and it evicted the enemy units and
    // also moved my unit". Founding displaces nothing: every unit that moved had an order.
    let mut s = two_players();
    found(&mut s, P0, at(2, 2));
    found(&mut s, P1, at(16, 16));
    let settler = spawn_unit(&mut s, P0, UnitKind::Settler, at(8, 8));
    let escort = spawn_unit(&mut s, P0, UnitKind::Spearman, at(8, 8));
    let foes: Vec<_> = at(8, 8)
        .neighbors()
        .iter()
        .take(2)
        .map(|h| spawn_unit(&mut s, P1, UnitKind::Spearman, *h))
        .collect();
    let before: Vec<_> = [escort, foes[0], foes[1]]
        .iter()
        .map(|u| (*u, s.units[u].pos))
        .collect();
    let orders = BTreeMap::from([(P0, vec![Order::FoundCity { unit: settler }])]);
    let (next, _) = crate::resolve_turn(&s, &orders);
    assert!(
        next.city_at(at(8, 8)).is_some(),
        "the city is founded at War"
    );
    for (u, pos) in before {
        assert_eq!(next.units[&u].pos, pos, "unit {} stays put", u.0);
    }
}

#[test]
fn capital_science_is_visible_unscaled_and_paid_on_the_founding_turn() {
    for faction in Faction::ALL {
        let mut s = test_game(&[(faction, 0), (Faction::Byzantium, 1)]);
        let settler = spawn_unit(&mut s, P0, UnitKind::Settler, at(3, 3));
        spawn_unit(&mut s, P1, UnitKind::Settler, at(15, 15));
        let orders = BTreeMap::from([(P0, vec![Order::FoundCity { unit: settler }])]);
        let (s, _) = crate::resolve_turn(&s, &orders);
        let capital = s.capital_of(P0).unwrap();
        let yields = economy::city_yields(&s, capital);
        assert_eq!(capital.population, 1);
        assert_eq!(yields.total.science, 2, "{faction:?}");
        assert_eq!(s.player(P0).science_stored, 2);
        assert_eq!(s.player(P0).science_history, vec![2]);
        let line = yields
            .sources
            .iter()
            .find(|source| source.label == "Capital")
            .unwrap();
        assert_eq!(line.yields, Yields::new(0, 2, 0, 2));
        assert_eq!(
            yields
                .sources
                .iter()
                .fold(Yields::ZERO, |sum, source| sum + source.yields),
            yields.total
        );
        let view = crate::player_view(&s, P0);
        assert_eq!(view.economy.science_per_turn.total, 2);
        assert_eq!(
            view.cities
                .iter()
                .find(|c| c.id == capital.id)
                .unwrap()
                .detail
                .as_ref()
                .unwrap()
                .yields,
            yields
        );
    }
}

#[test]
fn capital_science_follows_capture_and_refounding_without_stacking() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::Mongols, 1)]);
    let old = found(&mut s, P0, at(3, 3));
    let second = found(&mut s, P0, at(9, 3));
    let enemy = found(&mut s, P1, at(15, 15));
    let has_bonus = |s: &GameState, id| {
        economy::city_yields(s, &s.cities[&id])
            .sources
            .iter()
            .any(|source| source.label == "Capital")
    };
    assert!(has_bonus(&s, old));
    assert!(!has_bonus(&s, second));
    assert!(has_bonus(&s, enemy));
    city_mut(&mut s, old).resistance = 1;
    assert_eq!(economy::city_yields(&s, city(&s, old)).total, Yields::ZERO);
    assert!(!has_bonus(&s, old));
    city_mut(&mut s, old).resistance = 0;

    crate::rules::battle::capture_city(&mut s, old, P1, &mut Vec::new());
    assert!(s.capital_of(P0).is_none());
    assert!(!has_bonus(&s, second), "existing cities are not promoted");
    assert!(!has_bonus(&s, old));
    city_mut(&mut s, old).resistance = 0;
    assert!(
        !has_bonus(&s, old),
        "ending resistance does not restore capital status"
    );
    assert_eq!(economy::player_income(&s, P1).science.total, 2);

    let replacement = found(&mut s, P0, at(3, 10));
    assert_eq!(s.capital_of(P0).unwrap().id, replacement);
    assert!(
        !s.cities[&replacement].original_capital,
        "replacement is not the original capital"
    );
    assert!(has_bonus(&s, replacement));
    crate::rules::battle::capture_city(&mut s, old, P0, &mut Vec::new());
    city_mut(&mut s, old).resistance = 0;
    assert!(
        !has_bonus(&s, old),
        "recapture does not restore the old capital"
    );
    assert_eq!(economy::player_income(&s, P0).science.total, 2);
}

#[test]
fn found_city_rules() {
    let mut s = two_players();
    found(&mut s, P0, at(5, 5));
    let mut ev = Vec::new();

    // Distance 3 is too close, distance 4 is fine.
    let near = spawn_unit(&mut s, P1, UnitKind::Settler, at(8, 5));
    assert!(found_city(&mut s, near, &mut ev).is_err());
    assert!(
        s.units.contains_key(&near),
        "failed founding keeps the settler"
    );
    let ok = spawn_unit(&mut s, P1, UnitKind::Settler, at(9, 5));
    assert!(found_city(&mut s, ok, &mut ev).is_ok());

    // Not a settler.
    let spearman = spawn_unit(&mut s, P0, UnitKind::Spearman, at(5, 15));
    assert!(found_city(&mut s, spearman, &mut ev).is_err());

    // Not land.
    s.map.get_mut(at(15, 15)).unwrap().terrain = Terrain::Coast;
    let wet = spawn_unit(&mut s, P0, UnitKind::Settler, at(15, 15));
    assert!(found_city(&mut s, wet, &mut ev).is_err());

    // Owned by another empire.
    s.map.get_mut(at(15, 10)).unwrap().owner = Some(P1);
    let foreign = spawn_unit(&mut s, P0, UnitKind::Settler, at(15, 10));
    assert!(found_city(&mut s, foreign, &mut ev).is_err());
}

#[test]
fn found_city_leaves_foreign_tiles_alone() {
    let mut s = two_players();
    let n = at(5, 5).neighbors()[0];
    s.map.get_mut(n).unwrap().owner = Some(P1);
    let c = found(&mut s, P0, at(5, 5));
    assert_eq!(s.map.get(n).unwrap().owner, Some(P1));
    assert_ne!(s.map.get(n).unwrap().city, Some(c));
}

#[test]
fn city_names_are_unique_deterministic_and_fall_back() {
    let mut a = two_players();
    let mut b = two_players();
    let mut names = BTreeSet::new();
    for i in 0..4 {
        let ca = found(&mut a, P0, at(2 + 4 * i, 2));
        let cb = found(&mut b, P0, at(2 + 4 * i, 2));
        assert_eq!(a.cities[&ca].name, b.cities[&cb].name);
        assert!(names.insert(a.cities[&ca].name.clone()));
        assert!(city_names(Faction::Athens).contains(&a.cities[&ca].name.as_str()));
    }
    // Same-faction rival doesn't reuse a taken name.
    let rival = found(&mut a, P1, at(2, 10));
    assert!(names.insert(a.cities[&rival].name.clone()));

    // Exhaust the list: rename existing cities to cover every name.
    let mut s = two_players();
    for (i, name) in city_names(Faction::Athens).iter().enumerate() {
        let col = 2 + 4 * (i as i32 % 4);
        let row = 2 + 4 * (i as i32 / 4);
        let c = found(&mut s, P1, at(col, row));
        city_mut(&mut s, c).name = name.to_string();
    }
    assert_eq!(next_city_name(&s, P0), "Athens Colony 1");
}

// ---------- growth ----------

#[test]
fn growth_carries_excess_food() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    // Centre 2+2 food plus six worked grassland (2 each), eating 2: +14 a turn. The first
    // growth costs data::FIRST_GROWTH.
    let net = economy::city_yields(&s, city(&s, c)).total.food - 2;
    assert_eq!(net, 14);
    city_mut(&mut s, c).food_stored = data::FIRST_GROWTH - 4;
    let ev = city_turn(&mut s);
    assert_eq!(city(&s, c).population, 2);
    assert_eq!(city(&s, c).food_stored, net - 4);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::CityGrew { population: 2, .. }))
    );
}

#[test]
fn starvation_shrinks_to_minimum_one() {
    let mut s = two_players();
    for t in s.map.tiles.iter_mut() {
        t.terrain = Terrain::Desert;
    }
    let c = found(&mut s, P0, at(5, 5));
    // Center gives 2 food; pop 3 eats 6.
    city_mut(&mut s, c).population = 3;
    let ev = city_turn(&mut s);
    assert_eq!(city(&s, c).population, 2);
    assert_eq!(city(&s, c).food_stored, 0);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::CityStarved { population: 2, .. }))
    );
    city_mut(&mut s, c).population = 1;
    city_turn(&mut s);
    city_turn(&mut s);
    assert_eq!(city(&s, c).population, 1);
}

// ---------- production ----------

#[test]
fn production_floor_preserves_workshops_rich_tiles_and_india_penalty() {
    let mut s = test_game(&[(Faction::India, 0), (Faction::Byzantium, 1)]);
    let capital = found(&mut s, P0, at(3, 3));
    let far = found(&mut s, P0, at(15, 15));
    let production = |s: &GameState, id| economy::city_yields(s, city(s, id)).total.production;
    assert_eq!(production(&s, capital), 5);
    assert_eq!(
        production(&s, far),
        3,
        "India penalty cannot erase minimum city production"
    );
    for id in [capital, far] {
        city_mut(&mut s, id)
            .buildings
            .insert(BuildingKind::Workshop);
    }
    assert_eq!(
        production(&s, capital),
        7,
        "Workshop adds its full two production"
    );
    assert_eq!(
        production(&s, far),
        3,
        "the distant city's production remains at its floor"
    );

    city_mut(&mut s, capital).population = 2;
    for pos in at(3, 3).neighbors() {
        s.map.get_mut(pos).unwrap().terrain = Terrain::Forest;
    }
    let forests = economy::worked_tiles(&s, city(&s, capital))
        .iter()
        .filter(|h| s.map.get(**h).unwrap().terrain == Terrain::Forest)
        .count() as i32;
    assert!(forests >= 5, "{forests}");
    assert_eq!(
        production(&s, capital),
        5 + 2 * forests,
        "rich tiles retain their full output"
    );
    city_mut(&mut s, capital).resistance = 1;
    assert_eq!(
        production(&s, capital),
        0,
        "resistance suppresses the floor too"
    );
}

#[test]
fn production_completes_unit_with_overflow() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).producing = Some(spearman());
    city_mut(&mut s, c).production_stored = 79;
    city_turn(&mut s);
    assert_eq!(s.units_of(P0).count(), 1);
    assert_eq!(city(&s, c).production_stored, 4);
    assert_eq!(city(&s, c).producing, None);

    // Buildings go into the city; overflow carries.
    city_mut(&mut s, c).producing = Some(building(BuildingKind::Granary));
    city_mut(&mut s, c).production_stored = 59;
    city_turn(&mut s);
    assert!(city(&s, c).buildings.contains(&BuildingKind::Granary));
    assert_eq!(city(&s, c).production_stored, 14); // A Granary costs 50.
}

#[test]
fn spawn_placement_respects_stacking_and_waits_when_blocked() {
    let mut s = two_players();
    let pos = at(5, 5);
    let c = found(&mut s, P0, pos);
    let mut adj = pos.neighbors().to_vec();
    adj.sort();
    // Enough gold that upkeep never disbands anything here.
    s.player_mut(P0).gold = 1000;

    // Own soldier on the city tile: the new soldier goes to the first adjacent tile in hex order,
    // skipping water.
    spawn_unit(&mut s, P0, UnitKind::Spearman, pos);
    s.map.get_mut(adj[0]).unwrap().terrain = Terrain::Ocean;
    city_mut(&mut s, c).producing = Some(spearman());
    city_mut(&mut s, c).production_stored = 80;
    city_turn(&mut s);
    assert_eq!(s.units_at(adj[1]).count(), 1);
    assert_eq!(s.units_at(adj[1]).next().unwrap().moves_left, 0);

    // Block everything else: enemy scout, own military units.
    spawn_unit(&mut s, P1, UnitKind::Scout, adj[2]);
    for h in &adj[3..] {
        spawn_unit(&mut s, P0, UnitKind::Spearman, *h);
    }
    let before = s.units.len();
    city_mut(&mut s, c).producing = Some(spearman());
    city_mut(&mut s, c).production_stored = 80;
    let ev = city_turn(&mut s);
    assert_eq!(s.units.len(), before, "waits for a free tile");
    assert_eq!(city(&s, c).production_stored, 85);
    assert!(
        !ev.iter()
            .any(|e| matches!(e, Event::ProductionCompleted { .. }))
    );

    // A civilian still fits on the city tile next to the garrison.
    city_mut(&mut s, c).producing = Some(settler());
    city_mut(&mut s, c).production_stored = 70;
    city_mut(&mut s, c).population = 2;
    city_turn(&mut s);
    assert!(s.units_at(pos).any(|u| u.kind == UnitKind::Settler));
}

#[test]
fn settler_needs_pop_two_and_raises_the_next_cost() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    let item = settler();
    // The first Settler costs FIRST_SETTLER_PERCENT (100%) of 70.
    assert_eq!(economy::production_cost(&s, P0, item), 70);
    // As in Civ 6, a population-1 city can't choose, queue or buy a Settler.
    for order in [
        Order::SetProduction { city: c, item },
        Order::SetProductionQueue {
            city: c,
            queue: vec![spearman(), item],
        },
        Order::BuyItem { city: c, item },
    ] {
        let err = validate_city_order(&s, P0, &order).unwrap_err();
        assert!(err.contains("Settler needs population 2"), "{err}");
    }
    // A Settler whose city drops to population 1 mid-build keeps its progress banked, and
    // the city asks for a new choice.
    city_mut(&mut s, c).producing = Some(item);
    city_mut(&mut s, c).production_stored = 70;
    let ev = city_turn(&mut s);
    assert_eq!(s.units_of(P0).count(), 0, "pop 1 city can't train it");
    assert_eq!(city(&s, c).producing, None);
    assert_eq!(city(&s, c).banked(item), 70);
    assert!(ev.iter().any(|e| matches!(e, Event::OrderFailed { reason, .. }
        if reason.contains("can no longer produce Settler: Settler needs population 2 (currently 1); its 70 Production stays banked"))));
    let idle = city(&s, c).production_stored;

    city_mut(&mut s, c).population = 2;
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetProduction { city: c, item }],
        &mut Vec::new(),
    );
    assert_eq!(city(&s, c).production_stored, 70 + idle, "resumes its bank");
    city_mut(&mut s, c).food_stored = 0;
    city_turn(&mut s);
    assert_eq!(s.units_of(P0).count(), 1);
    assert_eq!(city(&s, c).population, 1);
    assert_eq!(s.player(P0).settlers_trained, 1);
    assert_eq!(economy::production_cost(&s, P0, item), 120);
}

#[test]
fn idle_production_caps_at_thirty() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).production_stored = 58;
    city_turn(&mut s);
    assert_eq!(city(&s, c).production_stored, 60);
    city_turn(&mut s);
    assert_eq!(city(&s, c).production_stored, 60);
}

#[test]
fn resistance_blocks_all_output_for_three_turns() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).resistance = CAPTURE_RESISTANCE_TURNS;
    city_mut(&mut s, c).producing = Some(spearman());
    for _ in 0..CAPTURE_RESISTANCE_TURNS {
        city_turn(&mut s);
        let city = city(&s, c);
        assert_eq!((city.food_stored, city.production_stored), (0, 0));
        assert_eq!(s.player(P0).gold, 0);
    }
    assert_eq!(city(&s, c).resistance, 0);
    city_turn(&mut s);
    let city = city(&s, c);
    // Centre 4 plus six grassland (12) Food, minus 2 eaten.
    assert_eq!((city.food_stored, city.production_stored), (14, 5));
    assert_eq!(s.player(P0).gold, 1);
}

// ---------- gold ----------

#[test]
fn income_adds_gold() {
    let mut s = test_game(&[(Faction::Switzerland, 0)]);
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).population = 2;
    // Center 1 + tolls (pop 2 / 2) 1.
    city_turn(&mut s);
    assert_eq!(s.player(P0).gold, 2);
}

#[test]
fn newly_completed_military_pays_upkeep_using_the_city_yield_snapshot() {
    for faction in [Faction::Athens, Faction::Switzerland] {
        let mut s = test_game(&[(faction, 0), (Faction::Byzantium, 1)]);
        let c = found(&mut s, P0, at(5, 5));
        spawn_unit(&mut s, P0, UnitKind::Archer, at(4, 5));
        spawn_unit(&mut s, P0, UnitKind::Spearman, at(6, 5));
        city_mut(&mut s, c).producing = Some(spearman());
        city_mut(&mut s, c).production_stored = 79;
        s.player_mut(P0).gold = 10;
        // Growth changes citizen science and Swiss gold, but this turn uses the old yields.
        city_mut(&mut s, c).food_stored = data::FIRST_GROWTH;
        assert_eq!(economy::player_income(&s, P0).gold.total, 1);
        city_turn(&mut s);
        assert_eq!(s.units_of(P0).count(), 3);
        assert_eq!(
            s.player(P0).gold,
            11 - data::military_upkeep(faction),
            "new units pay faction upkeep from the pre-growth yield snapshot"
        );
        assert_eq!(city(&s, c).population, 2);
        assert_eq!(s.player(P0).science_history, vec![2]);
    }
}

#[test]
fn negative_treasury_disbands_costliest_unit_one_per_turn() {
    let mut s = two_players();
    for i in 0..3 {
        spawn_unit(&mut s, P0, UnitKind::Spearman, at(2 + i, 2));
    }
    let t1 = spawn_unit(&mut s, P0, UnitKind::Pikeman, at(2, 4));
    let t2 = spawn_unit(&mut s, P0, UnitKind::Pikeman, at(3, 4));
    spawn_unit(&mut s, P0, UnitKind::Settler, at(4, 4));
    // 5 military, 2 free: upkeep 3.
    assert_eq!(economy::upkeep(&s, P0), 3);
    s.player_mut(P0).gold = 1;
    let ev = city_turn(&mut s);
    assert!(!s.units.contains_key(&t2), "tie on cost: highest id goes");
    assert!(s.units.contains_key(&t1));
    assert_eq!(s.player(P0).gold, 0);
    assert!(matches!(ev[0], Event::UnitDisbanded { unit, .. } if unit == t2));

    // Next turn: 4 military, upkeep 2, still broke: the other trooper goes.
    city_turn(&mut s);
    assert!(!s.units.contains_key(&t1));
    assert_eq!(s.units_of(P0).count(), 4);

    // Enough gold: nothing disbanded.
    s.player_mut(P0).gold = 5;
    city_turn(&mut s);
    assert_eq!(s.player(P0).gold, 4);
    assert_eq!(s.units_of(P0).count(), 4);
}

#[test]
fn science_history_keeps_last_five() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::Mongols, 1)]);
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).population = 4;
    s.player_mut(P0).science_history = vec![1, 2, 3, 4, 5];
    city_turn(&mut s);
    assert_eq!(s.player(P0).science_history, vec![1, 2, 3, 4, 5, 4]);
    assert_eq!(s.player(P1).science_history, vec![0]);
}

#[test]
fn science_history_discards_an_oversized_prefix_in_order() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::Mongols, 1)]);
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).population = 4;
    s.player_mut(P0).science_history = (0..1024).collect();
    city_turn(&mut s);
    assert_eq!(
        s.player(P0).science_history,
        vec![1015, 1016, 1017, 1018, 1019, 1020, 1021, 1022, 1023, 4]
    );
}

#[test]
fn an_undamaged_unthreatened_city_heals_twenty_and_caps_at_full() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    let b = found(&mut s, P0, at(12, 12));
    city_mut(&mut s, a).hp = CITY_HP - 50;
    city_mut(&mut s, b).hp = CITY_HP - 2;
    // Its own garrison next door changes nothing.
    spawn_unit(&mut s, P0, UnitKind::Spearman, at(6, 5));
    city_turn(&mut s);
    assert_eq!(CITY_HEAL, 20);
    assert_eq!(city(&s, a).hp, CITY_HP - 50 + 20);
    assert_eq!(city(&s, b).hp, CITY_HP);
    // Ten quiet turns take an empty city to full.
    city_mut(&mut s, a).hp = 0;
    for turn in 1..=10 {
        city_turn(&mut s);
        assert_eq!(city(&s, a).hp, (20 * turn).min(CITY_HP), "turn {turn}");
    }
}

#[test]
fn a_city_with_an_at_war_military_unit_adjacent_does_not_heal() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, a).hp = CITY_HP - 50;
    spawn_unit(&mut s, P1, UnitKind::Scout, at(4, 5));
    city_turn(&mut s);
    assert_eq!(city(&s, a).hp, CITY_HP - 50);
    // Two tiles away is not adjacent.
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, a).hp = CITY_HP - 50;
    spawn_unit(&mut s, P1, UnitKind::Spearman, at(3, 5));
    city_turn(&mut s);
    assert_eq!(city(&s, a).hp, CITY_HP - 50 + CITY_HEAL);
}

#[test]
fn a_city_damaged_this_turn_does_not_heal_but_one_damaged_last_turn_does() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, a).hp = CITY_HP - 50;
    city_mut(&mut s, a).last_attacked_turn = Some(s.turn);
    city_turn(&mut s);
    assert_eq!(city(&s, a).hp, CITY_HP - 50);
    city_mut(&mut s, a).last_attacked_turn = Some(s.turn.saturating_sub(1));
    s.turn += 1;
    city_turn(&mut s);
    assert_eq!(city(&s, a).hp, CITY_HP - 50 + CITY_HEAL);
}

// ---------- research ----------

#[test]
fn research_completes_and_carries_excess() {
    let mut s = two_players();
    s.player_mut(P0).researching = Some(TechId::Irrigation);
    // 11 arrives this turn on top of the bank; 3 is left over after paying the cost.
    let cost = economy::tech_cost(&s, P0, TechId::Irrigation).total;
    s.player_mut(P0).science_stored = cost - 8;
    s.player_mut(P0).science_history = vec![11];
    let mut ev = Vec::new();
    process_research(&mut s, &mut ev);
    assert!(s.player(P0).techs.contains(&TechId::Irrigation));
    assert_eq!(s.player(P0).science_stored, 3);
    assert_eq!(s.player(P0).researching, None);
    assert!(matches!(
        ev[0],
        Event::TechResearched {
            player: P0,
            tech: TechId::Irrigation
        }
    ));

    // With nothing selected, science still accumulates.
    s.player_mut(P0).science_history = vec![5];
    process_research(&mut s, &mut ev);
    assert_eq!(s.player(P0).science_stored, 3 + 5);
}

#[test]
fn tech_diffusion_lowers_cost() {
    let mut s = test_game(&[
        (Faction::Byzantium, 0),
        (Faction::Byzantium, 1),
        (Faction::Byzantium, 2),
        (Faction::Byzantium, 3),
        (Faction::Byzantium, 4),
        (Faction::Byzantium, 5),
        (Faction::Byzantium, 6),
    ]);
    let tech = TechId::Currency;
    let base = crate::data::tech_stats(tech).cost;
    let at_percent_off = |percent: i32| base * (100 - percent) / 100;
    s.player_mut(P0).techs.insert(TechId::Irrigation);
    assert_eq!(economy::tech_cost(&s, P0, tech).total, at_percent_off(0));
    s.player_mut(P1).techs.insert(tech);
    s.player_mut(P2).techs.insert(tech);
    // Two empires: -20%.
    assert_eq!(economy::tech_cost(&s, P0, tech).total, at_percent_off(20));
    for p in 3..=4 {
        s.player_mut(PlayerId(p)).techs.insert(tech);
    }
    // Four empires: -40%.
    assert_eq!(economy::tech_cost(&s, P0, tech).total, at_percent_off(40));
    s.player_mut(PlayerId(5)).techs.insert(tech);
    // Capped at -50%.
    let capped = at_percent_off(50);
    assert_eq!(economy::tech_cost(&s, P0, tech).total, capped);
    s.player_mut(PlayerId(6)).techs.insert(tech);
    assert_eq!(
        economy::tech_cost(&s, P0, tech).total,
        capped,
        "no more than 50%"
    );

    s.player_mut(P0).researching = Some(tech);
    s.player_mut(P0).science_history = vec![capped - 1];
    process_research(&mut s, &mut Vec::new());
    assert!(!s.player(P0).techs.contains(&tech));
    s.player_mut(P0).science_history = vec![1];
    process_research(&mut s, &mut Vec::new());
    assert!(s.player(P0).techs.contains(&tech));
    assert_eq!(s.player(P0).science_stored, 0);
}

#[test]
fn research_costs_are_fixed_before_anyone_completes() {
    let mut s = test_game(&[
        (Faction::Byzantium, 0),
        (Faction::Byzantium, 1),
        (Faction::Byzantium, 2),
    ]);
    // Both finish Bronze Working this turn; P1 must not get diffusion from P0 finishing first.
    let cost = economy::tech_cost(&s, P0, TechId::BronzeWorking).total;
    assert_eq!(
        cost,
        economy::tech_cost(&s, P1, TechId::BronzeWorking).total
    );
    for p in [P0, P1] {
        s.player_mut(p).researching = Some(TechId::BronzeWorking);
        s.player_mut(p).science_history = vec![cost];
    }
    process_research(&mut s, &mut Vec::new());
    assert_eq!(s.player(P0).science_stored, 0);
    assert_eq!(s.player(P1).science_stored, 0);
}

// ---------- territory (population is territory) ----------

fn own(s: &mut GameState, c: CityId, h: Hex) {
    let owner = s.cities[&c].owner;
    let t = s.map.get_mut(h).unwrap();
    t.owner = Some(owner);
    t.city = Some(c);
}

fn growth_tile(s: &mut GameState, c: CityId, h: Hex) {
    own(s, c, h);
    let city = city_mut(s, c);
    city.grown_tiles.push(h);
    city.tiles_claimed += 1;
    city.population += 1;
}

#[test]
fn actual_turn_186_save_reassigns_colony_11_tile_and_removes_all_workable_stripes() {
    // Derived from bug-saves/unworked-claim-colony11-20261025-0845.legatum:
    // the game state is unchanged; the unrelated replay archive and AI memory are omitted.
    let mut s: GameState = serde_json::from_str(include_str!(
        "../../tests/fixtures/sparta-colony11-t186.json"
    ))
    .unwrap();
    let tile = Hex::new(12, 11);
    let shizuoka = CityId(11);
    let colony = CityId(27);
    assert_eq!(s.cities[&shizuoka].name, "Shizuoka");
    assert_eq!(s.cities[&colony].name, "Sparta Colony 11");
    assert_eq!(s.map.get(tile).unwrap().city, Some(shizuoka));
    assert_eq!(s.map.get(tile).unwrap().owner, Some(P0));
    let workable_stripes = |state: &GameState| {
        state
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                let Some(source) = t.city.and_then(|id| state.cities.get(&id)) else {
                    return false;
                };
                let h = state.map.hex_at(*i);
                h.distance(source.pos) > CITY_WORK_RADIUS
                    && state.cities.values().any(|other| {
                        other.owner == source.owner
                            && other.id != source.id
                            && h.distance(other.pos) <= CITY_WORK_RADIUS
                    })
            })
            .count()
    };
    assert_eq!(workable_stripes(&s), 7);
    let mut events = Vec::new();
    process_borders(&mut s, &mut events);
    assert_eq!(s.map.get(tile).unwrap().city, Some(colony));
    assert!(economy::worked_tiles(&s, &s.cities[&colony]).contains(&tile));
    let view = crate::view::player_view(&s, P0);
    let colony_view = view.cities.iter().find(|c| c.id == colony).unwrap();
    assert!(
        colony_view
            .detail
            .as_ref()
            .unwrap()
            .territory
            .contains(&tile)
    );
    assert!(
        colony_view
            .detail
            .as_ref()
            .unwrap()
            .yields
            .worked
            .contains(&tile)
    );
    assert!(events.iter().any(
        |e| matches!(e, Event::TilesReassigned { from, to, tiles, .. }
        if *from == shizuoka && *to == colony && tiles.contains(&tile))
    ));
    assert_eq!(workable_stripes(&s), 0);
}

#[test]
fn saved_colony_11_tile_passes_from_captured_shizuoka_to_the_city_that_can_work_it() {
    // The 2026-10-25 T186 save: Shizuoka #11 at (13,14) claimed (12,11)
    // before Colony 11 #27 at (12,10) was founded. Sparta captured it at T175.
    let mut s = test_game(&[(Faction::Sparta, 0), (Faction::India, 1)]);
    s.map = Map::new(40, 26, Terrain::Grassland);
    let shizuoka = found(&mut s, P1, Hex::new(13, 14));
    let tile = Hex::new(12, 11);
    growth_tile(&mut s, shizuoka, tile);
    let colony = found(&mut s, P0, Hex::new(12, 10));
    let released = Hex::new(17, 11);
    s.map.get_mut(released).unwrap().terrain = Terrain::Desert;
    growth_tile(&mut s, shizuoka, released);
    assert_eq!(tile.distance(s.cities[&shizuoka].pos), 4);
    assert_eq!(tile.distance(s.cities[&colony].pos), 1);
    // Four people, so the halved city keeps the two it had when this save was captured.
    city_mut(&mut s, shizuoka).population = 4;
    crate::rules::battle::capture_city(&mut s, shizuoka, P0, &mut Vec::new());
    assert_eq!(next_release(&s, city(&s, shizuoka)), Some(released));
    let mut events = Vec::new();
    process_borders(&mut s, &mut events);
    assert_eq!(s.map.get(released).unwrap().owner, None);
    assert_eq!(s.map.get(tile).unwrap().city, Some(colony));
    assert_eq!(s.map.get(tile).unwrap().owner, Some(P0));
    assert!(!s.cities[&shizuoka].grown_tiles.contains(&tile));
    assert!(events.iter().any(
        |e| matches!(e, Event::TilesReassigned { from, to, owner, tiles }
        if *from == shizuoka && *to == colony && *owner == P0 && tiles.contains(&tile))
    ));
}

#[test]
fn dense_india_claims_choose_nearest_then_lower_city_id_without_crossing_empires() {
    let mut s = test_game(&[(Faction::India, 0), (Faction::Athens, 1)]);
    let source = found(&mut s, P0, Hex::new(4, 4));
    let lower = found(&mut s, P0, Hex::new(8, 4));
    let nearer = found(&mut s, P0, Hex::new(6, 7));
    let rival = found(&mut s, P1, Hex::new(12, 8));
    let tied = Hex::new(8, 6);
    let close = Hex::new(7, 6);
    let kept = Hex::new(6, 5); // Worked by its current city.
    let foreign = Hex::new(10, 8); // Workable by a rival, but never transferred there.
    for tile in [tied, close, kept, foreign] {
        growth_tile(&mut s, source, tile);
    }
    let mut events = Vec::new();
    reassign_workable_territory(&mut s, &mut events);
    assert_eq!(s.map.get(tied).unwrap().city, Some(lower));
    assert_eq!(s.map.get(close).unwrap().city, Some(nearer));
    assert_eq!(s.map.get(kept).unwrap().city, Some(source));
    assert_eq!(s.map.get(foreign).unwrap().city, Some(source));
    assert_ne!(s.map.get(foreign).unwrap().city, Some(rival));
    assert_eq!(s.cities[&source].grown_tiles, vec![kept, foreign]);
    assert_eq!(s.cities[&source].tiles_claimed, 2);
}

#[test]
fn territory_handoffs_are_idempotent_and_independent_of_growth_claim_order() {
    let mut s = test_game(&[(Faction::India, 0)]);
    let source = found(&mut s, P0, Hex::new(4, 4));
    let _first = found(&mut s, P0, Hex::new(8, 4));
    let _second = found(&mut s, P0, Hex::new(6, 7));
    for tile in [Hex::new(8, 6), Hex::new(7, 6)] {
        growth_tile(&mut s, source, tile);
    }
    let mut reverse = s.clone();
    reverse
        .cities
        .get_mut(&source)
        .unwrap()
        .grown_tiles
        .reverse();
    let mut a = Vec::new();
    let mut b = Vec::new();
    reassign_workable_territory(&mut s, &mut a);
    reassign_workable_territory(&mut reverse, &mut b);
    assert_eq!(a, b);
    assert_eq!(s.map.tiles, reverse.map.tiles);
    reassign_workable_territory(&mut s, &mut a);
    assert_eq!(a, b);
}

#[test]
fn a_new_city_owns_and_works_its_centre_and_ring_one() {
    let mut s = two_players();
    let pos = at(5, 5);
    // A rival already holds one ring-1 tile; founding leaves it alone.
    let taken = pos.neighbors()[0];
    s.map.get_mut(taken).unwrap().owner = Some(P1);
    let c = found(&mut s, P0, pos);
    let worked = economy::worked_tiles(&s, city(&s, c));
    assert_eq!(worked.len(), 5);
    assert!(!worked.contains(&taken));
    assert_eq!(s.map.get(taken).unwrap().owner, Some(P1));
    // Five grassland tiles plus the centre: 2 + 2 (grassland centre) + 5 x 2 Food.
    assert_eq!(economy::city_yields(&s, city(&s, c)).total.food, 14);
}

#[test]
fn each_growth_claims_the_best_adjacent_tile_by_focus() {
    let mut s = two_players();
    let pos = at(5, 5);
    let c = found(&mut s, P0, pos);
    let ring2 = pos.ring(2);
    // All ring-2 grassland ties: the lowest hex wins.
    assert_eq!(next_claim(&s, city(&s, c)), Some(ring2[0]));
    // A forest (1 Food, 2 Production) beats grassland (2 Food) under the Balanced focus...
    let forest = ring2[5];
    s.map.get_mut(forest).unwrap().terrain = Terrain::Forest;
    assert_eq!(next_claim(&s, city(&s, c)), Some(forest));
    // ...but not when the city wants Food (a Settler at population 1).
    city_mut(&mut s, c).producing = Some(settler());
    assert_eq!(next_claim(&s, city(&s, c)), Some(ring2[0]));
    city_mut(&mut s, c).producing = Some(spearman());
    assert_eq!(next_claim(&s, city(&s, c)), Some(forest));

    let mut ev = Vec::new();
    process_borders(&mut s, &mut ev);
    assert!(ev.is_empty(), "population 1 claims nothing: {ev:?}");
    city_mut(&mut s, c).population = 2;
    process_borders(&mut s, &mut ev);
    assert!(matches!(ev[..], [Event::TileClaimed { tile, .. }] if tile == forest));
    assert_eq!(s.map.get(forest).unwrap().city, Some(c));
    assert_eq!(city(&s, c).grown_tiles, vec![forest]);
    assert_eq!(city(&s, c).tiles_claimed, 1);
    assert!(economy::worked_tiles(&s, city(&s, c)).contains(&forest));
}

#[test]
fn growth_fills_each_ring_before_the_next_and_picks_quality_within_a_ring() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    // A Forest in ring 3 (best by the Balanced focus) must still wait for all of ring 2.
    let forest = pos.ring(3)[4];
    s.map.get_mut(forest).unwrap().terrain = Terrain::Forest;
    city_mut(&mut s, c).producing = Some(spearman());
    // Ring 2 has 12 tiles: 12 growths claim exactly those.
    city_mut(&mut s, c).population = 13;
    process_borders(&mut s, &mut Vec::new());
    let mut claimed = city(&s, c).grown_tiles.clone();
    assert_eq!(claimed.len(), 12);
    assert!(claimed.iter().all(|h| h.distance(pos) == 2), "{claimed:?}");
    // The next growth moves to ring 3 and takes its best tile.
    city_mut(&mut s, c).population = 14;
    process_borders(&mut s, &mut Vec::new());
    claimed = city(&s, c).grown_tiles.clone();
    assert_eq!(claimed.last(), Some(&forest));
    // And once ring 3 is full, ring 4 (claimed, unworked) follows, again in ring order.
    city_mut(&mut s, c).population = 1 + 12 + 18 + 1;
    process_borders(&mut s, &mut Vec::new());
    let grown = &city(&s, c).grown_tiles;
    assert_eq!(grown.len(), 12 + 18 + 1);
    assert!(grown[..30].iter().all(|h| h.distance(pos) <= 3));
    assert_eq!(grown[30].distance(pos), 4);
}

#[test]
fn growth_ranks_a_ring_beyond_the_work_radius_by_quality_too() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    for h in pos.within(CITY_WORK_RADIUS) {
        own(&mut s, c, h);
    }
    let forest = pos.ring(4)[7];
    s.map.get_mut(forest).unwrap().terrain = Terrain::Forest;
    city_mut(&mut s, c).producing = Some(spearman());
    assert_eq!(next_claim(&s, city(&s, c)), Some(forest));
    // A Food-hungry city prefers the grassland of the same ring.
    city_mut(&mut s, c).producing = Some(settler());
    assert_ne!(next_claim(&s, city(&s, c)), Some(forest));
}

#[test]
fn growth_reaches_past_the_work_radius_but_those_tiles_are_not_worked() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    for h in pos.within(CITY_WORK_RADIUS) {
        own(&mut s, c, h);
    }
    assert_eq!(economy::worked_tiles(&s, city(&s, c)).len(), 36);
    city_mut(&mut s, c).population = 2;
    let ev = {
        let mut ev = Vec::new();
        process_borders(&mut s, &mut ev);
        ev
    };
    let Event::TileClaimed { tile, .. } = ev[0] else {
        panic!("{ev:?}")
    };
    assert_eq!(tile.distance(pos), CITY_WORK_RADIUS + 1);
    assert_eq!(s.map.get(tile).unwrap().owner, Some(P0));
    assert!(!economy::worked_tiles(&s, city(&s, c)).contains(&tile));
    assert_eq!(economy::worked_tiles(&s, city(&s, c)).len(), 36);
}

#[test]
fn claims_skip_owned_tiles_and_wait_when_boxed_in() {
    let mut s = two_players();
    let pos = at(5, 5);
    let c = found(&mut s, P0, pos);
    let ring2 = pos.ring(2);
    s.map.get_mut(ring2[0]).unwrap().owner = Some(P1);
    assert_eq!(next_claim(&s, city(&s, c)), Some(ring2[1]));
    for h in &ring2 {
        s.map.get_mut(*h).unwrap().owner = Some(P1);
    }
    city_mut(&mut s, c).population = 2;
    let mut ev = Vec::new();
    process_borders(&mut s, &mut ev);
    assert!(ev.is_empty());
    assert!(city(&s, c).grown_tiles.is_empty());
    // Land frees up: the owed claim is made at the next territory phase.
    s.map.get_mut(ring2[2]).unwrap().owner = None;
    process_borders(&mut s, &mut ev);
    assert_eq!(city(&s, c).grown_tiles, vec![ring2[2]]);
}

#[test]
fn losing_population_releases_an_unworked_tile_first_then_the_latest_claim() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    let (a, b) = (pos.ring(2)[0], pos.ring(2)[1]);
    let far = pos.ring(4)[0];
    own(&mut s, c, a);
    own(&mut s, c, far);
    own(&mut s, c, b);
    let city_now = city_mut(&mut s, c);
    city_now.grown_tiles = vec![a, far, b];
    city_now.tiles_claimed = 3;
    city_now.population = 3;
    let mut ev = Vec::new();
    process_borders(&mut s, &mut ev);
    assert!(matches!(ev[..], [Event::TileReleased { tile, .. }] if tile == far));
    assert_eq!(s.map.get(far).unwrap().owner, None);
    assert_eq!(city(&s, c).tiles_claimed, 2);
    city_mut(&mut s, c).population = 2;
    ev.clear();
    process_borders(&mut s, &mut ev);
    assert!(matches!(ev[..], [Event::TileReleased { tile, .. }] if tile == b));
    assert_eq!(city(&s, c).grown_tiles, vec![a]);
}

#[test]
fn a_settler_costs_its_city_a_tile() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).population = 2;
    process_borders(&mut s, &mut Vec::new());
    let claimed = city(&s, c).grown_tiles[0];
    s.map.get_mut(claimed).unwrap().improved = true;
    city_mut(&mut s, c).producing = Some(settler());
    city_mut(&mut s, c).production_stored = 500;
    let ev = city_turn(&mut s);
    assert_eq!(city(&s, c).population, 1);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::TileReleased { tile, .. } if *tile == claimed))
    );
    assert_eq!(s.map.get(claimed).unwrap().owner, None);
    assert!(s.map.get(claimed).unwrap().improved);
}

#[test]
fn a_completed_great_work_tile_survives_population_loss() {
    let mut s = two_players();
    let c = found(&mut s, P0, at(5, 5));
    city_mut(&mut s, c).population = 2;
    process_borders(&mut s, &mut Vec::new());
    let site = city(&s, c).grown_tiles[0];
    city_mut(&mut s, c)
        .buildings
        .insert(BuildingKind::GreatLibrary);
    crate::landmarks::complete(&mut s, c, BuildingKind::GreatLibrary, site);
    process_borders(&mut s, &mut Vec::new());
    assert!(!city(&s, c).grown_tiles.contains(&site));
    assert_eq!(city(&s, c).grown_tiles.len(), 1);

    city_mut(&mut s, c).population = 1;
    process_borders(&mut s, &mut Vec::new());
    let tile = s.map.get(site).unwrap();
    assert_eq!((tile.owner, tile.city), (Some(P0), Some(c)));
    assert_eq!(tile.landmark.unwrap().city, c);
    assert!(city(&s, c).buildings.contains(&BuildingKind::GreatLibrary));
}

#[test]
fn code_of_laws_claims_ring_two_for_existing_cities() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    let b = found(&mut s, P0, at(12, 12));
    let mut events = Vec::new();
    crate::eras::learn(&mut s, P0, TechId::CodeOfLaws, &mut events);
    for id in [a, b] {
        let city = &s.cities[&id];
        let claims: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::TileClaimed { city, tile, .. } if *city == id => Some(*tile),
                _ => None,
            })
            .collect();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].distance(city.pos), 2);
        assert_eq!(s.map.get(claims[0]).unwrap().city, Some(id));
        assert!(!city.grown_tiles.contains(&claims[0]));
    }
    events.clear();
    crate::eras::learn(&mut s, P0, TechId::CodeOfLaws, &mut events);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::TileClaimed { .. }))
    );
}

#[test]
fn fusion_power_adds_five_production_to_every_city() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    let b = found(&mut s, P0, at(12, 12));
    let before = [a, b].map(|id| economy::city_yields(&s, &s.cities[&id]).total.production);
    crate::eras::learn(&mut s, P0, TechId::FusionPower, &mut Vec::new());
    for (id, old) in [a, b].into_iter().zip(before) {
        let yields = economy::city_yields(&s, &s.cities[&id]);
        assert_eq!(yields.total.production, old + 5);
        assert!(
            yields
                .sources
                .iter()
                .any(|source| source.label == "Fusion Power")
        );
    }
}

#[test]
fn food_sources_after_the_border_rework() {
    let mut s = test_game(&[(Faction::India, 0)]);
    let c = found(&mut s, P0, at(5, 5));
    let food = |s: &GameState| economy::city_yields(s, city(s, c)).total.food;
    let base = food(&s);
    city_mut(&mut s, c).buildings.insert(BuildingKind::Granary);
    // The Granary is +2 Food, buildable from turn 1.
    assert_eq!(food(&s), base + 2);
    // Soil Reclamation: +1 Food in every city. Civic Networks no longer gives Food.
    s.player_mut(P0).techs.insert(TechId::Irrigation);
    assert_eq!(food(&s), base + 3);
    s.player_mut(P0).techs.insert(TechId::CodeOfLaws);
    assert_eq!(food(&s), base + 3);
    let sources = economy::city_yields(&s, city(&s, c)).sources;
    assert!(
        sources
            .iter()
            .any(|y| y.label == "India network" && y.yields.food == 1)
    );
}

#[test]
fn city_phases_are_deterministic() {
    let build = || {
        let mut s = test_game(&[(Faction::India, 0), (Faction::Switzerland, 1)]);
        let a = found(&mut s, P0, at(4, 4));
        let b = found(&mut s, P1, at(12, 12));
        city_mut(&mut s, a).producing = Some(settler());
        city_mut(&mut s, b).producing = Some(spearman());
        s.player_mut(P0).researching = Some(TechId::BronzeWorking);
        s
    };
    let (mut x, mut y) = (build(), build());
    for _ in 0..40 {
        let ex = city_turn(&mut x);
        let ey = city_turn(&mut y);
        assert_eq!(ex, ey);
        assert_eq!(x.hash(), y.hash());
    }
    // Something actually happened.
    assert!(x.player(P0).techs.contains(&TechId::BronzeWorking));
    assert!(x.cities.values().all(|c| c.tiles_claimed > 0));
}

#[test]
fn research_queue_validates_order_and_rejects_known_duplicate_and_out_of_era_techs() {
    use TechId::*;
    let mut s = two_players();
    let queue = |techs| Order::SetResearchQueue { techs };
    for techs in [
        vec![Mathematics],
        vec![BronzeWorking, Mathematics],
        vec![BronzeWorking, BronzeWorking],
    ] {
        let before = s.clone();
        let order = queue(techs);
        assert_eq!(
            validate_city_orders(&s, P0, std::slice::from_ref(&order)).len(),
            1
        );
        let mut events = Vec::new();
        apply_city_orders(&mut s, P0, &[order], &mut events);
        assert_eq!(s, before, "invalid plans are atomic");
        assert!(matches!(events.as_slice(), [Event::OrderFailed { .. }]));
    }
    // Any order within the current era is fine: there are no prerequisites inside an era.
    let valid = queue(vec![Currency, BronzeWorking, Writing, Irrigation]);
    assert!(validate_city_order(&s, P0, &valid).is_ok());
    apply_city_orders(&mut s, P0, &[valid], &mut Vec::new());
    assert_eq!(s.player(P0).researching, Some(Currency));
    s.player_mut(P0).techs.insert(Masonry);
    assert!(validate_city_order(&s, P0, &queue(vec![Masonry])).is_err());
    // Queueing the rest of the era does not open the next one: Classical techs stay refused
    // until the whole Ancient era is known.
    assert!(validate_city_order(&s, P0, &queue(vec![Mathematics])).is_err());
    s.player_mut(P0).alive = false;
    assert!(validate_city_order(&s, P0, &queue(vec![])).is_err());
}

#[test]
fn research_queue_advances_once_per_turn_carries_science_and_emits_finished() {
    use TechId::*;
    let mut s = two_players();
    let path = vec![BronzeWorking, Writing, Irrigation, Currency];
    s.player_mut(P0).science_stored = 1000;
    s.player_mut(P0).science_history.push(7);
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetResearchQueue {
            techs: path.clone(),
        }],
        &mut Vec::new(),
    );
    let mut expected_science = 1000;
    for (i, tech) in path.iter().enumerate() {
        expected_science += 7 - economy::tech_cost(&s, P0, *tech).total;
        let mut events = Vec::new();
        process_research(&mut s, &mut events);
        assert_eq!(s.player(P0).techs.len(), i + 1);
        assert_eq!(s.player(P0).science_stored, expected_science);
        assert_eq!(s.player(P0).research_queue, path[i + 1..]);
        assert_eq!(s.player(P0).researching, path.get(i + 1).copied());
        assert_eq!(
            events,
            vec![
                Event::TechResearched {
                    player: P0,
                    tech: *tech
                },
                Event::ResearchQueueAdvanced {
                    player: P0,
                    next: path.get(i + 1).copied()
                }
            ]
        );
        let saved = serde_json::to_string(&s).unwrap();
        let loaded: GameState = serde_json::from_str(&saved).unwrap();
        assert_eq!(loaded, s);
        assert_eq!(loaded.hash(), s.hash());
        s = loaded;
    }
    let mut events = Vec::new();
    process_research(&mut s, &mut events);
    assert!(events.is_empty(), "finished is emitted only at transition");
}

#[test]
fn research_queue_stalls_clears_and_legacy_research_replaces_the_plan() {
    use TechId::*;
    let mut s = two_players();
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetResearchQueue {
            techs: vec![BronzeWorking, Writing],
        }],
        &mut Vec::new(),
    );
    let mut events = Vec::new();
    process_research(&mut s, &mut events);
    assert!(events.is_empty());
    assert_eq!(s.player(P0).research_queue, vec![BronzeWorking, Writing]);
    s.player_mut(P0).science_stored = 15;
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetResearch { tech: Irrigation }],
        &mut events,
    );
    assert_eq!(s.player(P0).research_queue, vec![Irrigation]);
    apply_city_orders(
        &mut s,
        P0,
        &[Order::SetResearchQueue { techs: vec![] }],
        &mut events,
    );
    assert_eq!(s.player(P0).researching, None);
    assert!(s.player(P0).research_queue.is_empty());
    assert_eq!(s.player(P0).science_stored, 15);
}

#[test]
fn old_save_without_queue_keeps_current_research_then_finishes_normally() {
    let mut s = two_players();
    s.player_mut(P0).researching = Some(TechId::BronzeWorking);
    s.player_mut(P0).science_stored = 100;
    let mut saved = serde_json::to_value(&s).unwrap();
    for p in saved["players"].as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("research_queue");
    }
    let mut loaded: GameState = serde_json::from_value(saved).unwrap();
    assert!(loaded.player(P0).research_queue.is_empty());
    assert_eq!(
        crate::player_view(&loaded, P0).research.queue,
        vec![TechId::BronzeWorking]
    );
    let mut events = Vec::new();
    process_research(&mut loaded, &mut events);
    assert!(loaded.player(P0).techs.contains(&TechId::BronzeWorking));
    assert!(events.contains(&Event::ResearchQueueAdvanced {
        player: P0,
        next: None
    }));
}

#[test]
fn blocked_queue_head_is_retained_while_later_work_proceeds() {
    let mut s = two_players();
    let id = found(&mut s, P0, at(5, 5));
    let blocked = spearman();
    for h in at(5, 5).within(1) {
        spawn_unit(&mut s, P0, UnitKind::Spearman, h);
    }
    let monument = building(BuildingKind::Granary);
    let c = city_mut(&mut s, id);
    c.production_stored = 17;
    c.producing = Some(blocked);
    c.production_queue = vec![monument];
    let mut events = Vec::new();
    produce(&mut s, id, 3, &mut events);
    assert_eq!(city(&s, id).producing, Some(monument));
    assert_eq!(city(&s, id).production_queue, vec![blocked]);
    // The waiting Spearman keeps its own progress; the Monument starts from its own.
    assert_eq!(city(&s, id).production_stored, 3);
    assert_eq!(city(&s, id).banked(blocked), 17);
    assert!(events.iter().any(|e| matches!(e, Event::OrderFailed { reason: message, .. } if message.contains("No free spawn tile") && message.contains("kept in queue"))));
    produce(&mut s, id, 200, &mut events);
    assert!(city(&s, id).buildings.contains(&BuildingKind::Granary));
    assert_eq!(city(&s, id).production_queue, vec![blocked]);
    assert_eq!(city(&s, id).producing, None);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::OrderFailed { .. }))
            .count(),
        1,
        "finishing the alternative must not repeat the earlier skip"
    );
    let overflow = city(&s, id).production_stored;
    s.units.clear();
    produce(&mut s, id, 3, &mut events);
    assert!(city(&s, id).production_queue.is_empty());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ProductionCompleted { item, .. } if *item == blocked))
    );
    // Overflow and the Spearman's own bank both went into it.
    let cost = economy::city_production_cost(&s, city(&s, id), blocked);
    assert_eq!(city(&s, id).production_stored, overflow + 17 + 3 - cost);
    assert!(city(&s, id).production_banked.is_empty());
}

#[test]
fn a_queued_settler_that_cannot_start_is_dropped_with_its_reason() {
    let mut s = two_players();
    let id = found(&mut s, P0, at(5, 5));
    let c = city_mut(&mut s, id);
    c.population = 2;
    c.production_queue = vec![settler()];
    // Population fell after it was queued.
    c.population = 1;
    let mut events = Vec::new();
    produce(&mut s, id, 4, &mut events);
    assert_eq!(city(&s, id).producing, None);
    assert!(city(&s, id).production_queue.is_empty());
    assert!(matches!(&events[..], [Event::OrderFailed { reason, .. }]
        if reason.contains("skipped queued Settler: Settler needs population 2 (currently 1)")
            && !reason.contains("kept in queue")));
}

#[test]
fn blocked_only_queue_banks_production_and_reports_once_across_save_load() {
    // Spawn space is the only temporary blocker now that a Settler needs population 2
    // to be queued at all.
    let mut s = two_players();
    let id = found(&mut s, P0, at(5, 5));
    let item = spearman();
    for h in at(5, 5).within(1) {
        spawn_unit(&mut s, P0, UnitKind::Spearman, h);
    }
    let cap = IDLE_PRODUCTION_CAP;
    let item_cost = economy::city_production_cost(&s, city(&s, id), item);
    let c = city_mut(&mut s, id);
    c.production_queue = vec![item];
    c.production_stored = cap - 1;
    // Legacy saves have no notification history.
    let saved = serde_json::to_value(&s).unwrap();
    assert!(
        saved["cities"][id.0.to_string()]
            .get("production_queue_notified")
            .is_none()
    );
    s = serde_json::from_value(saved).unwrap();
    for turn in 0..3 {
        let mut events = Vec::new();
        produce(&mut s, id, 4, &mut events);
        assert_eq!(city(&s, id).producing, None);
        assert_eq!(city(&s, id).production_queue, vec![item]);
        // Banking while waiting stops at the queued item's cost.
        let stored = if cap > item_cost {
            cap - 1
        } else {
            (cap - 1 + 4 * (turn + 1)).min(item_cost)
        };
        assert_eq!(city(&s, id).production_stored, stored);
        assert_eq!(events.len(), if turn == 0 { 1 } else { 0 });
        if turn == 0 {
            assert!(
                matches!(&events[0], Event::OrderFailed { reason, .. } if reason.contains("kept in queue"))
            );
        }
        let detail = crate::player_view(&s, P0)
            .cities
            .into_iter()
            .find(|c| c.id == id)
            .unwrap()
            .detail
            .unwrap();
        assert_eq!(detail.idle_production_cap, item_cost);
        assert_eq!(
            detail.idle_production_waste,
            (detail.yields.total.production - (item_cost - stored))
                .clamp(0, detail.yields.total.production)
        );
        let loaded: GameState = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(loaded.hash(), s.hash());
        s = loaded;
    }
    let banked = city(&s, id).production_stored;
    s.units.clear();
    let cost = economy::city_production_cost(&s, city(&s, id), item);
    let mut events = Vec::new();
    produce(&mut s, id, cost, &mut events);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ProductionCompleted { item: built, .. } if *built == item))
    );
    assert_eq!(city(&s, id).production_stored, banked);
    assert!(city(&s, id).production_queue_notified.is_empty());
}

#[test]
fn waiting_queue_banks_only_up_to_the_first_queued_item_cost() {
    // An item waiting for spawn space must not bank indefinitely, then pay for whatever
    // comes after it (a Great Work) from the overflow.
    let mut s = two_players();
    let id = found(&mut s, P0, at(5, 5));
    for h in at(5, 5).within(1) {
        spawn_unit(&mut s, P0, UnitKind::Spearman, h);
    }
    let cost = economy::city_production_cost(&s, city(&s, id), spearman());
    let c = city_mut(&mut s, id);
    c.production_queue = vec![spearman()];
    c.production_stored = cost - 3;
    for _ in 0..3 {
        let mut events = Vec::new();
        produce(&mut s, id, 5, &mut events);
        assert_eq!(city(&s, id).producing, None);
        assert_eq!(city(&s, id).production_stored, cost);
    }
    let detail = crate::player_view(&s, P0)
        .cities
        .into_iter()
        .find(|c| c.id == id)
        .unwrap()
        .detail
        .unwrap();
    assert_eq!(detail.idle_production_cap, cost);
    assert_eq!(detail.idle_production_waste, detail.yields.total.production);
    // Stored progress above the cap (older saves) is kept, never cut.
    city_mut(&mut s, id).production_stored = cost + 50;
    produce(&mut s, id, 5, &mut Vec::new());
    assert_eq!(city(&s, id).production_stored, cost + 50);
}

#[test]
fn invalid_active_build_banks_its_progress_and_the_city_still_banks_this_turn() {
    let mut s = two_players();
    let id = found(&mut s, P0, at(5, 5));
    // Needs Fabrication, which the player lacks: invalid, so its 10 stays banked.
    let archive = building(BuildingKind::Workshop);
    let c = city_mut(&mut s, id);
    c.producing = Some(archive);
    c.production_queue = vec![settler()];
    c.production_stored = 10;
    let mut events = Vec::new();
    produce(&mut s, id, 7, &mut events);
    // The Workshop keeps its 10 for later; the population-1 Settler is dropped from the
    // queue; this turn's 7 is unassigned idle Production.
    assert_eq!(city(&s, id).banked(archive), 10);
    assert_eq!(city(&s, id).production_stored, 7);
    assert_eq!(city(&s, id).producing, None);
    assert!(city(&s, id).production_queue.is_empty());
    assert_eq!(events.len(), 2);
}

#[test]
fn player_facing_messages_never_contain_camel_case_identifiers() {
    let mut s = two_players();
    let id = found(&mut s, P0, at(5, 5));
    let mut messages = Vec::new();
    for tech in &TechId::ALL {
        let name = data::tech_name(*tech);
        let era = crate::eras::era_of_tech(*tech);
        let mut earlier = BTreeSet::new();
        for t in TechId::ALL {
            if crate::eras::era_of_tech(t) < era {
                earlier.insert(t);
            }
        }
        let cases = [
            // Already known.
            (BTreeSet::from([*tech]), Order::SetResearch { tech: *tech }),
            (
                BTreeSet::from([*tech]),
                Order::SetResearchQueue { techs: vec![*tech] },
            ),
            // In the current era, but twice in the plan.
            (
                earlier.clone(),
                Order::SetResearchQueue {
                    techs: vec![*tech, *tech],
                },
            ),
        ];
        for (known, order) in cases {
            s.player_mut(P0).techs = known;
            let message = validate_city_order(&s, P0, &order).unwrap_err();
            assert!(message.contains(name));
            let mut events = Vec::new();
            apply_city_orders(&mut s, P0, &[order], &mut events);
            assert_eq!(
                events,
                vec![Event::OrderFailed {
                    player: P0,
                    reason: message.clone()
                }]
            );
            messages.push(message);
        }
        // Outside the current era: nothing known, so every later era is gated.
        s.player_mut(P0).techs.clear();
        if era > 0 {
            for order in [
                Order::SetResearch { tech: *tech },
                Order::SetResearchQueue { techs: vec![*tech] },
            ] {
                let message = validate_city_order(&s, P0, &order).unwrap_err();
                assert!(message.contains(name));
                assert!(message.contains("finish the Ancient era first"));
                messages.push(message);
            }
        }
    }
    s.player_mut(P0).techs.clear();
    for item in UnitKind::ALL
        .iter()
        .map(|unit| ProductionItem::Unit { unit: *unit })
        .chain(
            BuildingKind::ALL
                .iter()
                .map(|building| ProductionItem::Building {
                    building: *building,
                }),
        )
    {
        if let Some(tech) = data::item_requires(item)
            .filter(|_| data::available_to_faction(s.player(P0).faction, item))
        {
            let message = economy::production_blocker(&s, city(&s, id), item, true).unwrap();
            assert_eq!(message, format!("Requires {}", data::tech_name(tech)));
            messages.push(message.clone());
            let mut queued = s.clone();
            city_mut(&mut queued, id).production_queue = vec![item];
            let mut events = Vec::new();
            produce(&mut queued, id, 0, &mut events);
            let Event::OrderFailed { reason, .. } = events.remove(0) else {
                panic!("expected skip");
            };
            assert!(reason.ends_with(&message));
            messages.push(reason);
        }
    }
    for kind in &UnitKind::ALL {
        let unit = spawn_unit(&mut s, P0, *kind, at(5, 5));
        if let Err(message) = crate::rules::units::check_upgrade(&s, P0, unit, 1000) {
            messages.push(message);
        }
    }
    for message in messages {
        assert!(
            !message
                .as_bytes()
                .windows(2)
                .any(|pair| pair[0].is_ascii_lowercase() && pair[1].is_ascii_uppercase()),
            "raw identifier in player-facing message: {message}"
        );
    }
}

#[test]
fn invalid_queue_item_reports_its_specific_building_requirement() {
    let mut s = two_players();
    let id = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).techs.extend(TechId::ALL);
    let c = city_mut(&mut s, id);
    c.population = 8;
    c.production_queue = vec![building(BuildingKind::Factory), spearman()];
    let mut events = Vec::new();
    produce(&mut s, id, 3, &mut events);
    assert_eq!(city(&s, id).producing, Some(spearman()));
    assert!(events.iter().any(
        |e| matches!(e, Event::OrderFailed { reason, .. } if reason.ends_with("Requires Forge"))
    ));
}

#[test]
fn great_work_reservation_names_city_and_distinguishes_queued_from_started() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    let b = found(&mut s, P0, at(12, 12));
    let work = ProductionItem::Wonder {
        building: BuildingKind::RoyalArsenal,
        tile: at(6, 5),
    };
    city_mut(&mut s, a).name = "Northstar".into();
    city_mut(&mut s, a).production_queue = vec![work];
    assert_eq!(
        economy::building_blocker(&s, city(&s, b), BuildingKind::GrandBazaar, true).as_deref(),
        Some("World wonder queued in Northstar; only one per empire")
    );
    city_mut(&mut s, a).production_queue.clear();
    city_mut(&mut s, a).producing = Some(work);
    assert_eq!(
        economy::building_blocker(&s, city(&s, b), BuildingKind::GrandBazaar, true).as_deref(),
        Some("World wonder under construction in Northstar; only one per empire")
    );
}

#[test]
fn engine_purchase_breakdown_quotes_pending_settlers_without_reverse_rounding() {
    let mut s = test_game(&[(Faction::Switzerland, 0)]);
    let a = found(&mut s, P0, at(5, 5));
    let b = found(&mut s, P0, at(12, 12));
    city_mut(&mut s, a).population = 2;
    city_mut(&mut s, b).population = 2;
    s.player_mut(P0).gold = 10_000;
    let quote = economy::buy_quote(&s, city(&s, b), settler()).unwrap();
    assert_eq!(quote.gold_percent, 175);
    // The first Settler costs FIRST_SETTLER_PERCENT; the next pays full price + 50.
    assert_eq!(
        quote.production.total,
        70 * data::FIRST_SETTLER_PERCENT / 100
    );
    assert_eq!(
        quote.additional_settler_production,
        70 + 50 - quote.production.total
    );
    apply_city_orders(
        &mut s,
        P0,
        &[Order::BuyItem {
            city: a,
            item: settler(),
        }],
        &mut Vec::new(),
    );
    assert_eq!(
        economy::buy_cost_for(&s, city(&s, b), settler()),
        Some(
            (quote.production.total + quote.additional_settler_production) * quote.gold_percent
                / 100
        )
    );
}

#[test]
fn experimental_settler_escalation_prices_sequential_purchases() {
    let mut s = test_game(&[(Faction::Switzerland, 0)]);
    s.setup.rule_experiments.settler_escalation = true;
    let a = found(&mut s, P0, at(5, 5));
    let b = found(&mut s, P0, at(12, 12));
    city_mut(&mut s, a).population = 4;
    city_mut(&mut s, b).population = 4;
    s.player_mut(P0).gold = 10_000;
    s.player_mut(P0).settlers_trained = 2;
    let quote = economy::buy_quote(&s, city(&s, a), settler()).unwrap();
    assert_eq!(quote.production.total, 370);
    // The next price is the listed price, 150 further along.
    assert_eq!(
        quote.additional_settler_production,
        520 - 370,
        "the pending price increment is the difference of the two prices"
    );
    apply_city_orders(
        &mut s,
        P0,
        &[Order::BuyItem {
            city: a,
            item: settler(),
        }],
        &mut Vec::new(),
    );
    assert_eq!(s.player(P0).settlers_trained, 3);
    assert_eq!(economy::production_cost(&s, P0, settler()), 520);
    assert_eq!(s.player(P0).gold, 10_000 - 370 * 175 / 100);
    assert_eq!(
        economy::buy_quote(&s, city(&s, b), settler())
            .unwrap()
            .production
            .total,
        520
    );
    let v = crate::view::player_view(&s, P0);
    let option = v
        .cities
        .iter()
        .find(|c| c.id == b)
        .unwrap()
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|o| o.item == settler())
        .unwrap();
    assert_eq!(option.cost, 520);
    assert_eq!(option.buy_cost, Some(520 * 175 / 100));
}

#[test]
fn experimental_city_research_tracks_ownership_diffusion_and_resolution() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::Mongols, 1)]);
    let ids: Vec<_> = [(5, 5), (12, 5), (5, 12), (12, 12)]
        .into_iter()
        .map(|(x, y)| found(&mut s, P0, at(x, y)))
        .collect();
    s.player_mut(P1).techs.insert(TechId::BronzeWorking);
    // One other empire knows it: -10%. Each city beyond two adds 5% of that, rounded up.
    let diffused = crate::data::tech_stats(TechId::BronzeWorking).cost * 90 / 100;
    let with_extra = |extra: i32| diffused + (diffused * extra * 5 + 99) / 100;
    assert_eq!(
        economy::tech_cost(&s, P0, TechId::BronzeWorking).total,
        diffused
    );
    s.setup.rule_experiments.city_research = true;
    assert_eq!(
        economy::tech_cost(&s, P0, TechId::BronzeWorking).total,
        with_extra(2)
    );
    city_mut(&mut s, ids[3]).owner = P1;
    assert_eq!(
        economy::tech_cost(&s, P0, TechId::BronzeWorking).total,
        with_extra(1)
    );
    city_mut(&mut s, ids[2]).owner = P1;
    assert_eq!(
        economy::tech_cost(&s, P0, TechId::BronzeWorking).total,
        diffused
    );
    city_mut(&mut s, ids[2]).owner = P0;
    city_mut(&mut s, ids[3]).owner = P0;
    let income = economy::player_income(&s, P0).science.total;
    s.player_mut(P0).science_history.push(income);
    s.player_mut(P0).researching = Some(TechId::BronzeWorking);
    s.player_mut(P0).science_stored = with_extra(2) - income - 1;
    process_research(&mut s, &mut Vec::new());
    assert!(!s.player(P0).techs.contains(&TechId::BronzeWorking));
    let v = crate::view::player_view(&s, P0);
    assert_eq!(v.research.cost, Some(with_extra(2)));
    process_research(&mut s, &mut Vec::new());
    assert!(s.player(P0).techs.contains(&TechId::BronzeWorking));
}

#[test]
fn experimental_city_upkeep_is_paid_and_forecast_and_no_tech_discounts_it() {
    let mut s = two_players();
    for (x, y) in [(5, 5), (12, 5), (5, 12), (12, 12)] {
        found(&mut s, P0, at(x, y));
    }
    s.setup.rule_experiments.city_upkeep = true;
    s.player_mut(P0).gold = 100;
    s.player_mut(P0).techs.extend(TechId::ALL);
    assert_eq!(economy::upkeep(&s, P0), 8);
    let income = economy::player_income(&s, P0);
    assert_eq!(
        income.gold.total,
        income.cities.gold - 8 + income.science.total / 2
    );
    let v = crate::view::player_view(&s, P0);
    assert_eq!(v.economy.upkeep, 8);
    assert_eq!(v.economy.gold_per_turn, income.gold);
    process_cities(&mut s, &mut Vec::new());
    assert_eq!(s.player(P0).gold, 100 + income.gold.total);
    // Administration still applies during resistance, when the city's income is absent.
    for c in s.cities.values_mut() {
        c.resistance = 2;
    }
    s.player_mut(P0).gold = 0;
    let v = crate::view::player_view(&s, P0);
    assert_eq!(v.economy.bankruptcy.clone().unwrap().gold.total, -8);
}

#[test]
fn default_opening_pace_adds_capital_output_and_costs_stay_at_the_listed_price() {
    let mut s = test_game(&[(Faction::Byzantium, 0)]);
    let capital = found(&mut s, P0, at(5, 5));
    let colony = found(&mut s, P0, at(12, 12));
    let before = economy::city_yields(&s, city(&s, capital)).total.production;
    let colony_before = economy::city_yields(&s, city(&s, colony)).total.production;
    assert_eq!(
        economy::city_yields(&s, city(&s, capital)).total.production,
        before
    );
    assert_eq!(
        economy::city_yields(&s, city(&s, colony)).total.production,
        colony_before
    );
    for turn in [0, 25, 100, 400] {
        s.turn = turn;
        assert_eq!(
            economy::production_cost(&s, P0, spearman()),
            80,
            "turn {turn}"
        );
    }
    s.turn = 50;
    city_mut(&mut s, capital).population = 2;
    let quote = economy::buy_quote(&s, city(&s, capital), settler()).unwrap();
    assert_eq!(
        quote.production.total,
        economy::production_cost(&s, P0, settler())
    );
    // The first Settler is discounted (FIRST_SETTLER_PERCENT); the next one is not.
    let first = economy::production_cost(&s, P0, settler());
    s.player_mut(P0).settlers_trained = 1;
    assert_eq!(
        quote.additional_settler_production,
        economy::production_cost(&s, P0, settler()) - first
    );
}

fn buy_tile(c: CityId, tile: Hex) -> Order {
    Order::BuyTile { city: c, tile }
}

#[test]
fn tile_price_rises_with_ring_and_tiles_gained() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    let price = |s: &GameState, ring: i32| tile_price(city(s, c), pos.ring(ring)[0]);
    // 40 Gold base: ring 1 100%, ring 2 125%, ring 3 150%.
    assert_eq!((price(&s, 1), price(&s, 2), price(&s, 3)), (40, 50, 60));
    assert_eq!(tile_ring_percent(1), 100);
    assert_eq!(tile_ring_percent(3), 150);
    // Each tile gained adds 20 to the base, so every ring costs more.
    city_mut(&mut s, c).tiles_claimed = 2;
    assert_eq!(tile_base_price(city(&s, c)), 80);
    assert_eq!((price(&s, 1), price(&s, 2), price(&s, 3)), (80, 100, 120));
    // The ring percent is applied once and rounds down.
    city_mut(&mut s, c).tiles_claimed = 1;
    assert_eq!(tile_base_price(city(&s, c)), 60);
    assert_eq!((price(&s, 2), price(&s, 3)), (75, 90));
    city_mut(&mut s, c).tiles_claimed = 3;
    assert_eq!((price(&s, 2), price(&s, 3)), (125, 150));
}

#[test]
fn buying_a_tile_claims_it_for_gold_and_uses_the_purchase_slot() {
    let mut s = two_players();
    let pos = at(5, 5);
    let c = found(&mut s, P0, pos);
    crate::visibility::update_explored(&mut s);
    let tile = pos.ring(2)[3];
    assert_eq!(tile_price(city(&s, c), tile), 50);
    s.player_mut(P0).gold = 24;
    let buy = buy_tile(c, tile);
    assert_eq!(
        validate_city_order(&s, P0, &buy).unwrap_err(),
        "Not enough Gold: the tile costs 50 Gold; you have 24 (need 26 more)"
    );
    s.player_mut(P0).gold = 100;
    let far = pos.ring(3)[0];
    assert!(
        validate_city_order(&s, P0, &buy_tile(c, far))
            .unwrap_err()
            .contains("next to its borders")
    );
    let mut ev = Vec::new();
    apply_city_orders(
        &mut s,
        P0,
        &[
            buy.clone(),
            Order::BuyItem {
                city: c,
                item: spearman(),
            },
        ],
        &mut ev,
    );
    assert_eq!(s.player(P0).gold, 50);
    assert_eq!(s.map.get(tile).unwrap().city, Some(c));
    assert_eq!(s.map.get(tile).unwrap().owner, Some(P0));
    assert_eq!(city(&s, c).tiles_claimed, 1);
    assert_eq!(city(&s, c).last_purchase_turn, Some(s.turn));
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::TileBought { gold: 50, tile: t, owner, .. } if *t == tile && *owner == P0
    )));
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::OrderFailed { reason, .. } if reason.contains("already bought something this turn")
    )));
    // The next tile costs more: base 60, ring 2 is 125%.
    assert_eq!(tile_base_price(city(&s, c)), 60);
    assert_eq!(tile_price(city(&s, c), pos.ring(2)[7]), 75);
}

#[test]
fn a_bought_tile_is_not_a_growth_claim_and_survives_population_loss() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 500;
    let bought = pos.ring(2)[0];
    apply_city_orders(&mut s, P0, &[buy_tile(c, bought)], &mut Vec::new());
    // Population 1 keeps the bought tile and records no growth claim.
    process_borders(&mut s, &mut Vec::new());
    assert_eq!(s.map.get(bought).unwrap().city, Some(c));
    assert!(city(&s, c).grown_tiles.is_empty());
    // Growth still claims one tile per population above 1, on top of the bought one.
    city_mut(&mut s, c).population = 3;
    process_borders(&mut s, &mut Vec::new());
    assert_eq!(city(&s, c).grown_tiles.len(), 2);
    assert!(!city(&s, c).grown_tiles.contains(&bought));
    assert_eq!(city(&s, c).tiles_claimed, 3);
    // Losing every population releases the grown tiles, never the bought one.
    city_mut(&mut s, c).population = 1;
    let mut ev = Vec::new();
    process_borders(&mut s, &mut ev);
    assert!(city(&s, c).grown_tiles.is_empty());
    assert_eq!(s.map.get(bought).unwrap().city, Some(c));
    assert_eq!(s.map.get(bought).unwrap().owner, Some(P0));
    assert!(
        !ev.iter()
            .any(|e| matches!(e, Event::TileReleased { tile, .. } if *tile == bought))
    );
    assert_eq!(city(&s, c).tiles_claimed, 1);
}

#[test]
fn purchases_may_skip_rings_but_never_reach_past_the_work_radius() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 1000;
    let buy = |s: &GameState, tile: Hex| validate_city_order(s, P0, &buy_tile(c, tile));
    // Ring 3 isn't next to the territory yet.
    let ring2 = pos.ring(2)[0];
    let ring3 = pos
        .ring(3)
        .into_iter()
        .find(|h| h.neighbors().contains(&ring2))
        .unwrap();
    assert!(buy(&s, ring3).is_err());
    // Owning one ring-2 tile makes its ring-3 neighbour buyable, though the rest of ring 2 is
    // still free: purchases may snake around out of ring order.
    own(&mut s, c, ring2);
    assert!(buy(&s, ring3).is_ok());
    assert_eq!(tile_price(city(&s, c), ring3), 60);
    // A ring-4 tile next to a ring-3 one is beyond reach, whatever the Gold.
    own(&mut s, c, ring3);
    let ring4 = pos
        .ring(4)
        .into_iter()
        .find(|h| h.neighbors().contains(&ring3))
        .unwrap();
    // (Seen, so only the reach rule applies.)
    let i = s.map.index(ring4).unwrap();
    s.player_mut(P0).explored.set(i);
    assert!(
        buy(&s, ring4)
            .unwrap_err()
            .contains("farther tiles come only from growth")
    );
    assert!(!buy_candidates(&s, city(&s, c)).contains(&ring4));
    assert!(claim_candidates(&s, city(&s, c)).contains(&ring4));
}

#[test]
fn a_tile_must_be_explored_and_unowned_and_the_city_yours() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    s.player_mut(P0).gold = 1000;
    let tile = pos.ring(2)[0];
    // Unexplored.
    let blank = crate::bitset::BitSet::new(s.map.len());
    s.player_mut(P0).explored = blank;
    assert!(
        validate_city_order(&s, P0, &buy_tile(c, tile))
            .unwrap_err()
            .contains("Explore a tile")
    );
    crate::visibility::update_explored(&mut s);
    assert!(validate_city_order(&s, P0, &buy_tile(c, tile)).is_ok());
    // Already owned by someone.
    s.map.get_mut(tile).unwrap().owner = Some(P1);
    assert!(
        validate_city_order(&s, P0, &buy_tile(c, tile))
            .unwrap_err()
            .contains("already owned")
    );
    // Not your city.
    assert!(validate_city_order(&s, P1, &buy_tile(c, pos.ring(2)[1])).is_err());
    // An eliminated player buys nothing.
    s.player_mut(P0).alive = false;
    assert!(validate_city_order(&s, P0, &buy_tile(c, pos.ring(2)[1])).is_err());
}

#[test]
fn a_tile_and_an_item_share_the_city_purchase_slot_both_ways() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 2000;
    let tile = pos.ring(2)[0];
    let item = Order::BuyItem {
        city: c,
        item: spearman(),
    };
    // Item first, then tile: the tile is refused.
    let mut ev = Vec::new();
    apply_city_orders(&mut s, P0, &[item.clone(), buy_tile(c, tile)], &mut ev);
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::OrderFailed { reason, .. } if reason.contains("already bought something this turn")
    )));
    assert_eq!(s.map.get(tile).unwrap().city, None);
    assert!(
        validate_city_orders(&s, P0, &[buy_tile(c, tile)])
            .first()
            .is_some_and(|e| e.message.contains("already bought something this turn"))
    );
    // Tile first, then item: validation reports the item.
    let mut s = two_players();
    let c = found(&mut s, P0, pos);
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 2000;
    let errors = validate_city_orders(&s, P0, &[buy_tile(c, tile), item.clone()]);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].index, 1);
    // Another city of the same empire still has its own slot.
    let d = found(&mut s, P0, at(14, 14));
    crate::visibility::update_explored(&mut s);
    let other = pos_ring_tile_for(&s, d);
    assert!(validate_city_orders(&s, P0, &[buy_tile(c, tile), buy_tile(d, other)]).is_empty());
}

fn pos_ring_tile_for(s: &GameState, c: CityId) -> Hex {
    buy_candidates(s, city(s, c))[0]
}

#[test]
fn an_occupied_or_resisting_city_buys_no_tile() {
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 1000;
    let tile = pos.ring(2)[0];
    city_mut(&mut s, c).occupied_from = Some(P1);
    let reason = validate_city_order(&s, P0, &buy_tile(c, tile)).unwrap_err();
    assert!(reason.starts_with("Occupied:"), "{reason}");
    // The same words the item purchase gives.
    assert_eq!(
        Some(reason),
        economy::buy_blocker(&s, city(&s, c), spearman())
    );
    city_mut(&mut s, c).occupied_from = None;
    city_mut(&mut s, c).resistance = 2;
    assert!(
        validate_city_order(&s, P0, &buy_tile(c, tile))
            .unwrap_err()
            .contains("resisting occupation")
    );
}

#[test]
fn two_cities_cannot_buy_the_same_tile_in_one_turn() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(8, 8));
    let b = found(&mut s, P0, at(8, 12));
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 1000;
    let shared = buy_candidates(&s, city(&s, a))
        .into_iter()
        .find(|h| buy_candidates(&s, city(&s, b)).contains(h))
        .expect("the two cities border a common free tile");
    let orders = [buy_tile(a, shared), buy_tile(b, shared)];
    let errors = validate_city_orders(&s, P0, &orders);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].index, 1);
    assert!(errors[0].message.contains("already owned"));
    let mut ev = Vec::new();
    apply_city_orders(&mut s, P0, &orders, &mut ev);
    assert_eq!(s.map.get(shared).unwrap().city, Some(a));
    assert_eq!(
        ev.iter()
            .filter(|e| matches!(e, Event::TileBought { .. }))
            .count(),
        1
    );
    assert_eq!(city(&s, b).last_purchase_turn, None);
    // Two empires: the order of resolution decides, the later one fails.
    let mut s = two_players();
    let a = found(&mut s, P0, at(8, 8));
    let b = found(&mut s, P1, at(8, 12));
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 1000;
    s.player_mut(P1).gold = 1000;
    let shared = buy_candidates(&s, city(&s, a))
        .into_iter()
        .find(|h| buy_candidates(&s, city(&s, b)).contains(h))
        .expect("the two cities border a common free tile");
    let mut ev = Vec::new();
    apply_city_orders(&mut s, P0, &[buy_tile(a, shared)], &mut ev);
    apply_city_orders(&mut s, P1, &[buy_tile(b, shared)], &mut ev);
    assert_eq!(s.map.get(shared).unwrap().owner, Some(P0));
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::OrderFailed { player, .. } if *player == P1))
    );
    assert_eq!(s.player(P1).gold, 1000);
}

#[test]
fn a_city_with_no_free_tile_in_reach_says_so_in_one_sentence() {
    use crate::view::{ViewDetail, player_view_detail, public_view};
    let mut s = two_players();
    let c = found(&mut s, P0, at(8, 8));
    s.player_mut(P0).explored = crate::bitset::BitSet::new(s.map.len());
    let full = player_view_detail(&s, P0, &public_view(&s), ViewDetail::Full);
    let d = full
        .cities
        .iter()
        .find(|v| v.id == c)
        .and_then(|v| v.detail.as_ref())
        .unwrap();
    assert!(d.buyable_tiles.is_empty());
    let want = format!(
        "{} has no free tile to buy within 3 tiles of the city.",
        city(&s, c).name
    );
    assert_eq!(d.buy_tile_blocker.as_deref(), Some(want.as_str()));
}

#[test]
fn the_view_lists_buyable_tiles_nearest_ring_first_for_a_human_only() {
    use crate::view::{ViewDetail, player_view_detail, public_view};
    let mut s = two_players();
    let pos = at(8, 8);
    let c = found(&mut s, P0, pos);
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 10;
    let public = public_view(&s);
    let full = player_view_detail(&s, P0, &public, ViewDetail::Full);
    let d = full
        .cities
        .iter()
        .find(|v| v.id == c)
        .and_then(|v| v.detail.as_ref())
        .unwrap();
    assert_eq!(d.tile_base_price, 40);
    assert_eq!(d.buy_tile_blocker, None);
    assert!(!d.buyable_tiles.is_empty());
    let rings: Vec<i32> = d.buyable_tiles.iter().map(|b| b.ring).collect();
    assert!(rings.windows(2).all(|w| w[0] <= w[1]), "{rings:?}");
    assert_eq!(d.buyable_tiles[0].ring, 2);
    assert_eq!(d.buyable_tiles[0].price, 50);
    let decisions = player_view_detail(&s, P0, &public, ViewDetail::Decisions);
    let d = decisions
        .cities
        .iter()
        .find(|v| v.id == c)
        .and_then(|v| v.detail.as_ref())
        .unwrap();
    assert!(d.buyable_tiles.is_empty());
    assert_eq!(d.tile_base_price, 40);
    // After a purchase the view says why the city can buy no more.
    s.player_mut(P0).gold = 500;
    apply_city_orders(&mut s, P0, &[buy_tile(c, pos.ring(2)[0])], &mut Vec::new());
    let full = player_view_detail(&s, P0, &public_view(&s), ViewDetail::Full);
    let d = full
        .cities
        .iter()
        .find(|v| v.id == c)
        .and_then(|v| v.detail.as_ref())
        .unwrap();
    assert!(
        d.buy_tile_blocker
            .as_deref()
            .unwrap()
            .contains("already bought something this turn")
    );
}

#[test]
fn no_purchase_under_threat_stops_military_buys_near_an_enemy_and_is_off_by_default() {
    let mut s = two_players();
    let id = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).gold = 10_000;
    let spearman = ProductionItem::Unit {
        unit: UnitKind::Spearman,
    };
    let settler_free = ProductionItem::Building {
        building: BuildingKind::Granary,
    };
    // An enemy military unit two tiles from the city.
    let near = s.cities[&id]
        .pos
        .within(2)
        .into_iter()
        .find(|h| h.distance(s.cities[&id].pos) == 2)
        .unwrap();
    let enemy = spawn_unit(&mut s, P1, UnitKind::Spearman, near);
    // Off by default: nothing changes.
    assert_eq!(economy::buy_block(&s, city(&s, id), spearman), None);
    s.setup
        .rule_experiments
        .apply("no-purchase-under-threat")
        .unwrap();
    assert_eq!(
        economy::buy_block(&s, city(&s, id), spearman),
        Some(economy::BuyBlock::UnderThreat)
    );
    let reason = validate_city_order(
        &s,
        P0,
        &Order::BuyItem {
            city: id,
            item: spearman,
        },
    )
    .unwrap_err();
    assert!(
        reason.contains("can't buy military units while an enemy unit is within 2 tiles"),
        "{reason}"
    );
    // Buildings are still for sale.
    assert_eq!(economy::buy_block(&s, city(&s, id), settler_free), None);
    // Three tiles out, or at Peace, it no longer blocks.
    let far = s.cities[&id]
        .pos
        .within(3)
        .into_iter()
        .find(|h| h.distance(s.cities[&id].pos) == 3)
        .unwrap();
    s.units.get_mut(&enemy).unwrap().pos = far;
    assert_eq!(economy::buy_block(&s, city(&s, id), spearman), None);
    s.units.get_mut(&enemy).unwrap().pos = near;
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::Peace;
    }
    assert_eq!(economy::buy_block(&s, city(&s, id), spearman), None);
}

#[test]
fn two_empires_buying_one_tile_the_same_turn_is_drawn_by_seed_not_seat() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(8, 8));
    let b = found(&mut s, P1, at(8, 12));
    crate::visibility::update_explored(&mut s);
    s.player_mut(P0).gold = 1000;
    s.player_mut(P1).gold = 1000;
    let shared = buy_candidates(&s, city(&s, a))
        .into_iter()
        .find(|h| buy_candidates(&s, city(&s, b)).contains(h))
        .expect("the two cities border a common free tile");
    let orders = BTreeMap::from([
        (P0, vec![buy_tile(a, shared)]),
        (P1, vec![buy_tile(b, shared)]),
    ]);
    let mut winners = BTreeSet::new();
    for seed in 0..40 {
        s.dice_seed = Some(seed);
        let (control, _) = crate::resolve_turn(&s, &BTreeMap::new());
        let (next, events) = crate::resolve_turn(&s, &orders);
        let winner = next
            .map
            .get(shared)
            .unwrap()
            .owner
            .expect("someone buys it");
        let loser = if winner == P0 { P1 } else { P0 };
        winners.insert(winner);
        // The loser fails the ordinary way and keeps its Gold.
        assert!(events.iter().any(|e| matches!(e,
            Event::OrderFailed { player, reason } if *player == loser && reason.contains("already owned"))));
        assert_eq!(next.player(loser).gold, control.player(loser).gold);
        assert!(next.player(winner).gold < control.player(winner).gold);
        // The same seed gives the same result.
        assert_eq!(
            crate::resolve_turn(&s, &orders)
                .0
                .map
                .get(shared)
                .unwrap()
                .owner,
            Some(winner)
        );
    }
    assert_eq!(winners.len(), 2, "either seat can win the tile");
}

#[test]
fn a_same_turn_world_wonder_race_goes_to_more_overflow_then_to_a_seeded_draw() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(5, 5));
    let b = found(&mut s, P1, at(14, 14));
    let pyramids = ProductionItem::Building {
        building: BuildingKind::Pyramids,
    };
    let cost_a = economy::city_production_cost(&s, city(&s, a), pyramids);
    let cost_b = economy::city_production_cost(&s, city(&s, b), pyramids);
    for (id, cost) in [(a, cost_a), (b, cost_b)] {
        let c = city_mut(&mut s, id);
        c.producing = Some(pyramids);
        c.production_stored = cost - 10;
    }
    let yields = |pa: i32, pb: i32| {
        BTreeMap::from([(a, Yields::new(0, pa, 0, 0)), (b, Yields::new(0, pb, 0, 0))])
    };
    // The higher city id finishes with more overflow: it goes first, whatever the seed.
    for seed in 0..10 {
        s.dice_seed = Some(seed);
        assert_eq!(wonder_race_order(&s, &yields(12, 20)), vec![b, a]);
        assert_eq!(wonder_race_order(&s, &yields(20, 12)), vec![a, b]);
        // Only one city finishes: no race, id order.
        assert_eq!(wonder_race_order(&s, &yields(5, 20)), vec![a, b]);
    }
    // Exactly equal overflow: a seeded draw, so either city can win.
    let firsts: BTreeSet<CityId> = (0..40)
        .map(|seed| {
            s.dice_seed = Some(seed);
            wonder_race_order(&s, &yields(15, 15))[0]
        })
        .collect();
    assert_eq!(firsts, BTreeSet::from([a, b]));
    // `resolve_wonder_race` then hands the losers their refund, as before.
}

//! Macro regressions use real prices, fogged views, validation and turn resolution.
use legatum_ai::{Brain, DecisionParams, Knobs, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::state::Map;
use legatum_core::visibility::update_explored;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);

fn at(col: i32, row: i32) -> Hex {
    Hex::from_offset(col, row)
}

fn scenario() -> (GameState, CityId) {
    let mut setup = GameSetup::all_ai(817);
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Byzantium;
    // A rival with no special unit that makes it a better rusher (`opponents::rush_capable`).
    setup.players[1].faction = Faction::Athens;
    setup.width = 24;
    setup.height = 16;
    let mut s = blank_game(&setup, Map::new(24, 16, Terrain::Plains));
    let settler = spawn_unit(&mut s, ME, UnitKind::Settler, at(6, 8));
    let city = found_city(&mut s, settler, &mut Vec::new()).unwrap();
    let enemy = spawn_unit(&mut s, ENEMY, UnitKind::Settler, at(20, 8));
    found_city(&mut s, enemy, &mut Vec::new()).unwrap();
    s.cities.get_mut(&city).unwrap().population = 3;
    s.player_mut(ME).gold = 0;
    s.turn = 100;
    (s, city)
}

/// Siege fixtures need a hostile neighbour; new games start at Peace.
fn at_war(s: &mut GameState) {
    for relation in &mut s.diplomacy.as_mut().unwrap().relations {
        relation.state = legatum_core::diplomacy::RelationState::War;
    }
}

fn decide(s: &mut GameState) -> Vec<Order> {
    decide_with(s, DecisionParams::default())
}

fn decide_with(s: &mut GameState, decisions: DecisionParams) -> Vec<Order> {
    update_explored(s);
    let v = player_view(s, ME);
    let test_brain = || {
        // The scenario tests the macro choice, not Byzantium's climbed personality.
        let mut brain = ScriptedBrain::with_knobs(
            v.faction_of(ME),
            s.setup.seed,
            ME,
            Knobs::new(55, 70, 75, 45, 45),
        );
        brain.difficulty = AiDifficulty::Hard;
        brain.decisions = decisions;
        brain
    };
    let orders = test_brain().orders(&v);
    assert!(validate_orders(s, ME, &orders).is_empty(), "{orders:?}");
    let loaded: GameState = serde_json::from_str(&serde_json::to_string(s).unwrap()).unwrap();
    assert_eq!(orders, test_brain().orders(&player_view(&loaded, ME)));
    orders
}

fn bought(orders: &[Order], id: CityId) -> bool {
    orders
        .iter()
        .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == id))
}

fn resolve(s: &GameState, orders: Vec<Order>) -> GameState {
    let (next, events) = resolve_turn(s, &BTreeMap::from([(ME, orders)]));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { player, .. } if *player == ME)),
        "{events:?}"
    );
    next
}

#[test]
fn opening_research_funds_a_lab_after_food_and_production() {
    let (mut s, _) = scenario();
    s.player_mut(ME)
        .techs
        .extend([TechId::Irrigation, TechId::BronzeWorking]);
    let orders = decide(&mut s);
    assert_eq!(
        orders.iter().find_map(|o| match o {
            Order::SetResearchQueue { techs } => techs.first().copied(),
            _ => None,
        }),
        Some(TechId::Writing)
    );
}

#[test]
fn buys_an_overdue_lab_at_the_real_price_without_queuing_a_duplicate() {
    let (mut s, city) = scenario();
    // Keep this purchasing regression independent of India's tighter city spacing, which makes
    // early Settler purchases more attractive on the open test map.
    s.player_mut(ME).faction = Faction::Athens;
    s.player_mut(ME)
        .techs
        .extend([TechId::Irrigation, TechId::BronzeWorking, TechId::Writing]);
    s.cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .extend([BuildingKind::Granary, BuildingKind::Workshop]);
    spawn_unit(&mut s, ME, UnitKind::Spearman, at(6, 8));
    spawn_unit(&mut s, ME, UnitKind::Archer, at(6, 9));
    update_explored(&mut s);
    let lab = ProductionItem::Building {
        building: BuildingKind::Library,
    };
    let quote = player_view(&s, ME)
        .cities
        .iter()
        .find(|c| c.id == city)
        .unwrap()
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|o| o.item == lab)
        .unwrap()
        .buy_cost
        .unwrap();
    s.player_mut(ME).gold = quote;
    // About the real price of an overdue Library, not the held Gold: buy_hold 3 (default
    // since 2026-10-31) keeps the treasury for the army spike and has its own tests.
    let orders = decide_with(
        &mut s,
        DecisionParams {
            buy_hold: 0,
            ..DecisionParams::default()
        },
    );
    assert!(
        !orders.contains(&Order::SetProduction { city, item: lab }),
        "{orders:?}"
    );
    assert!(
        orders.contains(&Order::BuyItem { city, item: lab }),
        "{orders:?}"
    );
    let next = resolve(&s, orders);
    assert!(
        next.cities[&city]
            .buildings
            .contains(&BuildingKind::Library)
    );
    assert!(next.player(ME).gold < quote);
    s.player_mut(ME).gold = 0;
    assert!(!bought(&decide(&mut s), city));
}

#[test]
fn does_not_buy_production_that_will_finish_this_turn() {
    let (mut s, city) = scenario();
    s.player_mut(ME).techs.insert(TechId::BronzeWorking);
    s.player_mut(ME).gold = 200;
    let c = s.cities.get_mut(&city).unwrap();
    c.producing = Some(ProductionItem::Building {
        building: BuildingKind::Workshop,
    });
    c.production_stored =
        (data::building_stats_for(Faction::Byzantium, BuildingKind::Workshop).cost) - 1;
    let orders = decide(&mut s);
    assert!(
        !orders.contains(&Order::BuyItem {
            city,
            item: ProductionItem::Building {
                building: BuildingKind::Workshop
            }
        }),
        "{orders:?}"
    );
    assert!(
        resolve(&s, orders).cities[&city]
            .buildings
            .contains(&BuildingKind::Workshop)
    );
}

#[test]
fn a_bankrupt_city_finishes_its_market_instead_of_training_an_unpayable_defender() {
    let (mut s, city) = scenario();
    s.player_mut(ME).techs.insert(TechId::Currency);
    spawn_unit(&mut s, ME, UnitKind::Spearman, at(6, 8));
    for row in 0..6 {
        spawn_unit(&mut s, ME, UnitKind::Spearman, at(1, row));
    }
    spawn_unit(&mut s, ENEMY, UnitKind::Archer, at(8, 8));
    let market = ProductionItem::Building {
        building: BuildingKind::Market,
    };
    let cost = legatum_core::economy::production_cost(&s, ME, market);
    let c = s.cities.get_mut(&city).unwrap();
    c.producing = Some(market);
    c.production_stored = cost - 30;
    update_explored(&mut s);
    assert!(player_view(&s, ME).economy.gold_per_turn.total < 0);
    let orders = decide(&mut s);
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::SetProduction { city: id, .. } if *id == city)),
        "{orders:?}"
    );
    assert_eq!(resolve(&s, orders).cities[&city].producing, Some(market));
}

#[test]
fn siege_cannot_buy_walls_when_all_defender_spawn_tiles_are_occupied() {
    let (mut s, city) = scenario();
    at_war(&mut s);
    s.player_mut(ME).techs.insert(TechId::Masonry);
    s.player_mut(ME).gold = (data::building_stats_for(Faction::Byzantium, BuildingKind::Walls)
        .cost)
        * data::BUY_COST_PERCENT
        / 100;
    for pos in at(6, 8).within(1) {
        spawn_unit(&mut s, ME, UnitKind::Scout, pos);
    }
    spawn_unit(&mut s, ENEMY, UnitKind::Archer, at(8, 8));
    let orders = decide(&mut s);
    assert!(
        !orders.iter().any(|o| matches!(o,
            Order::BuyItem { item: ProductionItem::Building { building }, .. }
                if data::building_family(*building) == BuildingKind::Walls
        )),
        "{orders:?}"
    );
    assert!(
        !resolve(&s, orders).cities[&city]
            .buildings
            .contains(&BuildingKind::Walls)
    );
}

#[test]
fn a_frontier_garrison_gets_its_quoted_upgrade_before_a_routine_rush_purchase() {
    let (mut s, city) = scenario();
    s.player_mut(ME).techs.extend([
        TechId::BronzeWorking,
        TechId::IronWorking,
        TechId::Masonry,
        TechId::Feudalism,
    ]);
    let soldier = spawn_unit(&mut s, ME, UnitKind::Phalangite, at(6, 8));
    spawn_unit(&mut s, ME, UnitKind::Archer, at(5, 8));
    spawn_unit(&mut s, ENEMY, UnitKind::Scout, at(9, 8));
    let item = ProductionItem::Building {
        building: BuildingKind::Workshop,
    };
    let cost = legatum_core::economy::production_cost(&s, ME, item);
    let c = s.cities.get_mut(&city).unwrap();
    c.producing = Some(item);
    c.production_stored = cost - 30;
    update_explored(&mut s);
    let upgrade = player_view(&s, ME)
        .units
        .iter()
        .find(|u| u.id == soldier)
        .unwrap()
        .upgrade_breakdown
        .as_ref()
        .unwrap()
        .total;
    s.player_mut(ME).gold = upgrade;
    // The Phalangite→Pikeman upgrade also uses 1 Iron supply; this test is about Gold priorities.
    let deposit = s.cities[&city].pos.neighbors()[0];
    // The deposit is already mined: a Mine is half the old price and would otherwise be a cheap
    // routine purchase competing with the upgrade for the same Gold.
    let tile = s.map.get_mut(deposit).unwrap();
    tile.resource = Some(legatum_core::Resource::Iron);
    tile.improved = true;
    let orders = decide(&mut s);
    assert!(!bought(&orders, city), "{orders:?}");
    assert!(
        orders.contains(&Order::UpgradeUnit { unit: soldier }),
        "{orders:?}"
    );
    assert_eq!(resolve(&s, orders).units[&soldier].kind, UnitKind::Pikeman);
}

#[test]
fn a_new_bombardment_interrupts_a_settler_and_banks_its_progress() {
    let (mut s, city) = scenario();
    at_war(&mut s);
    spawn_unit(&mut s, ME, UnitKind::Spearman, at(6, 8));
    spawn_unit(&mut s, ENEMY, UnitKind::Archer, at(8, 8));
    let settler = ProductionItem::Unit {
        unit: UnitKind::Settler,
    };
    let progress = legatum_core::economy::production_cost(&s, ME, settler) / 2;
    let c = s.cities.get_mut(&city).unwrap();
    c.producing = Some(settler);
    c.production_stored = progress;
    let orders = decide(&mut s);
    assert!(
        orders.contains(&Order::SetProduction {
            city,
            item: ProductionItem::Unit {
                unit: UnitKind::Archer
            },
        }),
        "{orders:?}"
    );
    let next = resolve(&s, orders);
    // Per-item banking: the Settler's progress waits for the next Settler; the Archer is built
    // from its own progress rather than popping out of the Settler's.
    assert_eq!(next.cities[&city].banked(settler), progress);
    assert_eq!(
        next.cities[&city].producing,
        Some(ProductionItem::Unit {
            unit: UnitKind::Archer
        })
    );
    assert!(next.cities[&city].population >= 3);
}

#[test]
fn visible_enemy_troops_near_a_thin_city_buy_a_second_defender_before_buildings() {
    let (mut s, city) = scenario();
    at_war(&mut s);
    s.player_mut(ME).techs.insert(TechId::BronzeWorking);
    spawn_unit(&mut s, ME, UnitKind::Spearman, at(6, 8));
    // Not yet a forecast capture: a visible Spearman approaching, three tiles out.
    spawn_unit(&mut s, ENEMY, UnitKind::Spearman, at(9, 8));
    s.player_mut(ME).gold = 2000;
    let orders = decide(&mut s);
    assert!(
        orders.iter().any(|o| matches!(o,
            Order::BuyItem { city: id, item: ProductionItem::Unit { unit } }
                if *id == city && legatum_core::data::unit_stats(*unit).military)),
        "{orders:?}"
    );
    assert!(
        !orders.iter().any(|o| matches!(
            o,
            Order::BuyItem {
                item: ProductionItem::Building { .. },
                ..
            }
        )),
        "{orders:?}"
    );
    resolve(&s, orders);

    // Without the visible enemy, the same treasury buys the overdue Workshop instead.
    s.units.retain(|_, u| u.owner == ME);
    let orders = decide(&mut s);
    assert!(
        orders.contains(&Order::BuyItem {
            city,
            item: ProductionItem::Building {
                building: BuildingKind::Workshop
            }
        }),
        "{orders:?}"
    );
}

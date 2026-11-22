use super::placement::{self, CLEAR_RADIUS, IRON_MAX_DISTANCE, IRON_MIN_DISTANCE, nearest_start};
use super::*;
use crate::game::spawn_unit;
use crate::mapgen;
use crate::orders::Order;
use crate::rules::cities::tests::test_game;
use crate::state::GameSetup;
use crate::view::player_view;
use crate::{resolve_turn, validate_orders};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const TROOPER: ProductionItem = ProductionItem::Unit {
    unit: UnitKind::Pikeman,
};
const SOLDIER: ProductionItem = ProductionItem::Unit {
    unit: UnitKind::Spearman,
};
const WALLS: ProductionItem = ProductionItem::Building {
    building: BuildingKind::Walls,
};

fn at(col: i32, row: i32) -> Hex {
    Hex::from_offset(col, row)
}

fn orders(p: PlayerId, os: Vec<Order>) -> BTreeMap<PlayerId, Vec<Order>> {
    BTreeMap::from([(p, os)])
}

/// P0 (Byzantium, no yield bonuses) with a city at (5,5) that owns an Iron tile, knows
/// Ballistics (and so Fabrication), and has gold for upkeep.
fn iron_game() -> (GameState, CityId, Hex) {
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::Byzantium, 1)]);
    let id = crate::rules::test_support::city(&mut s, 0, at(5, 5), 3);
    // A rival city keeps the game going (an empty empire is eliminated on the first turn).
    crate::rules::test_support::city(&mut s, 1, at(16, 2), 1);
    let iron = at(6, 5);
    s.map.get_mut(iron).unwrap().resource = Some(Resource::Iron);
    let p = s.player_mut(P0);
    p.techs.extend([
        TechId::BronzeWorking,
        TechId::IronWorking,
        TechId::Masonry,
        TechId::Feudalism,
    ]);
    p.gold = 1000;
    (s, id, iron)
}

/// `iron_game` for Switzerland: Byzantium's levy cap of 2 would stop a third Pikeman.
fn swiss_iron_game() -> (GameState, CityId, Hex) {
    let (mut s, id, iron) = iron_game();
    s.player_mut(P0).faction = Faction::Switzerland;
    (s, id, iron)
}

fn iron_view(s: &GameState) -> SupplyView {
    player_view(s, P0)
        .supply
        .into_iter()
        .find(|x| x.resource == Resource::Iron)
        .unwrap()
}

fn give(s: &mut GameState, city: CityId, building: BuildingKind) {
    crate::economy::install_building(s.cities.get_mut(&city).unwrap(), building);
}

#[test]
fn direct_supply_counts_match_the_deposit_and_user_lists() {
    let (mut state, city, iron) = iron_game();
    state.map.get_mut(iron).unwrap().improved = true;
    give(&mut state, city, BuildingKind::Forge);
    spawn_unit(&mut state, P0, UnitKind::Pikeman, at(5, 4));
    for player in [P0, P1] {
        for resource in Resource::ALL {
            let from_deposits: i32 = deposits(&state, player)
                .iter()
                .filter(|deposit| deposit.resource == resource)
                .map(Deposit::supply)
                .sum();
            let from_users = current_users(&state, player)
                .iter()
                .filter(|(_, used)| *used == resource)
                .count() as i32;
            assert_eq!(supplied(&state, player, resource), from_deposits);
            assert_eq!(in_use(&state, player, resource), from_users);
        }
    }
}

const FORGE: BuildingKind = BuildingKind::Forge;
const FACTORY: BuildingKind = BuildingKind::Factory;

#[test]
fn supply_needs_the_tech_and_the_border_and_a_mine_adds_one() {
    let (mut s, _, iron) = iron_game();
    s.player_mut(P0).techs.clear();
    assert_eq!(
        supplied(&s, P0, Resource::Iron),
        0,
        "hidden until Iron Working"
    );
    assert!(!iron_view(&s).revealed);

    s.player_mut(P0).techs.insert(TechId::IronWorking);
    let chip = iron_view(&s);
    assert!(chip.revealed);
    assert_eq!(chip.supplied.total, SUPPLY_PER_DEPOSIT);
    assert_eq!(chip.supplied.parts[0].0, "Iron near City 1");
    assert_eq!((chip.used.total, chip.spare), (0, 2));

    s.map.get_mut(iron).unwrap().improved = true;
    let chip = iron_view(&s);
    assert_eq!(chip.supplied.total, SUPPLY_PER_DEPOSIT + SUPPLY_PER_MINE);
    assert_eq!(chip.supplied.parts[0].0, "Iron near City 1, with a Mine");

    // Outside the border: no supply even with the tech.
    s.map.get_mut(iron).unwrap().owner = None;
    assert_eq!(supplied(&s, P0, Resource::Iron), 0);
    s.map.get_mut(iron).unwrap().owner = Some(P0);
    assert_eq!(supplied(&s, P0, Resource::Iron), 3);
}

#[test]
fn a_pikeman_starts_only_with_spare_iron_and_says_why_not() {
    let (mut s, id, _) = swiss_iron_game();
    spawn_unit(&mut s, P0, UnitKind::Pikeman, at(5, 4));
    spawn_unit(&mut s, P0, UnitKind::Pikeman, at(4, 5));
    let reason = start_blocker(&s, &s.cities[&id], TROOPER).unwrap();
    assert_eq!(
        reason,
        "No Iron to spare: all 2 of your Iron are in use. Bring another Iron deposit inside your borders, or build a Mine on one for +1 Iron."
    );
    let v = player_view(&s, P0);
    let option = v.cities[0]
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|o| o.item == TROOPER)
        .expect("still listed, with its blocker");
    assert_eq!(option.resource_blocker.as_deref(), Some(reason.as_str()));
    assert_eq!(option.supply_need, Some(Resource::Iron));
    let chip = iron_view(&s);
    assert_eq!(chip.used.parts, vec![("Reisläufer ×2".to_string(), 2)]);
    assert_eq!(chip.spare, 0);
    let errors = validate_orders(
        &s,
        P0,
        &[Order::SetProduction {
            city: id,
            item: TROOPER,
        }],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message.contains("No Iron to spare"));

    // A Mine makes room for one more.
    s.map.get_mut(at(6, 5)).unwrap().improved = true;
    assert_eq!(start_blocker(&s, &s.cities[&id], TROOPER), None);

    // No deposit at all: say how to get one.
    s.map.get_mut(at(6, 5)).unwrap().resource = None;
    assert_eq!(
        start_blocker(&s, &s.cities[&id], TROOPER).unwrap(),
        "Needs 1 Iron: you have none yet. Bring an Iron deposit inside your borders, or build a Mine on one for +1 Iron."
    );
    // Not yet revealed.
    s.player_mut(P0).techs.remove(&TechId::IronWorking);
    assert_eq!(
        start_blocker(&s, &s.cities[&id], TROOPER).unwrap(),
        "Needs Iron: Iron Working reveals Iron on the map"
    );
}

#[test]
fn a_build_in_progress_holds_its_supply_and_gives_it_back_when_replaced() {
    let (mut s, a, _) = swiss_iron_game();
    let b = crate::rules::test_support::city(&mut s, 0, at(12, 12), 3);
    let c = crate::rules::test_support::city(&mut s, 0, at(12, 6), 3);
    let start = |city| Order::SetProduction {
        city,
        item: TROOPER,
    };
    let errors = validate_orders(&s, P0, &[start(a), start(b), start(c)]);
    assert_eq!(errors.iter().map(|e| e.index).collect::<Vec<_>>(), vec![2]);
    assert!(
        errors[0].message.contains(
            "2 are held for the Pikeman being built in City 1 and the Pikeman being built in City"
        ),
        "{errors:?}"
    );
    let (next, _) = resolve_turn(&s, &orders(P0, vec![start(a), start(b)]));
    let chip = iron_view(&next);
    assert_eq!(chip.held.len(), 2);
    assert_eq!(chip.spare, 0);
    // The city's own build doesn't block itself, and replacing it frees its supply.
    assert_eq!(start_blocker(&next, &next.cities[&a], TROOPER), None);
    let swap = vec![
        Order::SetProduction {
            city: a,
            item: WALLS,
        },
        start(c),
    ];
    assert!(validate_orders(&next, P0, &swap).is_empty());
}

#[test]
fn the_forge_uses_iron_and_the_factory_adds_coal() {
    let (mut s, id, _) = iron_game();
    let coal = at(4, 5);
    s.map.get_mut(coal).unwrap().resource = Some(Resource::Coal);
    s.player_mut(P0)
        .techs
        .extend([TechId::IronWorking, TechId::Industrialization]);
    s.cities.get_mut(&id).unwrap().population = 8;
    give(&mut s, id, BuildingKind::Workshop);
    assert_eq!(
        crate::view::player_view(&s, P0).cities[0]
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .find(|o| o.item == ProductionItem::Building { building: FORGE })
            .and_then(|o| o.supply_need),
        Some(Resource::Iron)
    );
    give(&mut s, id, FORGE);
    give(&mut s, id, FACTORY);
    assert_eq!(
        current_users(&s, P0),
        vec![
            (
                SupplyUser::Building {
                    city: id,
                    building: FORGE
                },
                Resource::Iron
            ),
            (
                SupplyUser::Building {
                    city: id,
                    building: FACTORY
                },
                Resource::Coal
            ),
        ]
    );
    assert_eq!(
        item_need(ProductionItem::Building { building: FACTORY }),
        Some(Resource::Coal)
    );
    assert_eq!(item_need(WALLS), None);
    assert_eq!(
        item_need(ProductionItem::Building {
            building: BuildingKind::Workshop
        }),
        None
    );
    assert_eq!(
        item_need(ProductionItem::Building {
            building: BuildingKind::Granary
        }),
        None
    );
    assert_eq!(
        iron_view(&s).used.parts,
        vec![("Forge in City 1".to_string(), 1)]
    );
}

#[test]
fn short_supply_idles_the_newest_users_until_it_returns() {
    let (mut s, a, iron) = swiss_iron_game();
    s.player_mut(P0).techs.insert(TechId::IronWorking);
    let b = crate::rules::test_support::city(&mut s, 0, at(12, 12), 4);
    s.cities.get_mut(&a).unwrap().population = 4;
    give(&mut s, a, FORGE);
    let (mut s, events) = resolve_turn(&s, &BTreeMap::new());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SupplyShortage { .. }))
    );
    // A turn later b gets a Forge and a Pikeman stands: three users for 2 Iron.
    give(&mut s, b, FORGE);
    let pike = spawn_unit(&mut s, P0, UnitKind::Pikeman, at(12, 11));
    let (s, events) = resolve_turn(&s, &BTreeMap::new());
    assert!(events.contains(&Event::SupplyShortage {
        player: P0,
        resource: Resource::Iron,
        idle: 1,
        supplied: 2,
        used: 3,
    }));
    // The newest go idle first; on the same turn a building outranks a unit.
    assert_eq!(unit_unsupplied(&s, &s.units[&pike]), Some(Resource::Iron));
    assert_eq!(idle_tier(s.player(P0), b, FORGE), None);
    let view = player_view(&s, P0);
    assert_eq!(
        view.units.iter().find(|u| u.id == pike).unwrap().unsupplied,
        Some(Resource::Iron)
    );
    assert_eq!(iron_view(&s).idle, vec!["Reisläufer".to_string()]);
    // A rival sees the unit is short too: its fights read it.
    let rival = player_view(&s, P1);
    assert!(
        rival
            .units
            .iter()
            .filter(|u| u.id == pike)
            .all(|u| u.unsupplied == Some(Resource::Iron))
    );

    // Lose the deposit: every user is idle, and both Forges yield nothing.
    let mut lost = s.clone();
    lost.map.get_mut(iron).unwrap().owner = None;
    lost.map.get_mut(iron).unwrap().city = None;
    let (lost, events) = resolve_turn(&lost, &BTreeMap::new());
    assert!(events.iter().any(|e| matches!(
        e,
        Event::SupplyShortage {
            idle: 3,
            supplied: 0,
            ..
        }
    )));
    let y = crate::economy::city_yields(&lost, &lost.cities[&a]);
    let idle = y
        .sources
        .iter()
        .find(|x| x.label == "Forge idle: no Iron")
        .expect("the idle line explains the loss");
    assert_eq!(idle.yields, Yields::new(0, -3, 0, 0));
    let detail = player_view(&lost, P0).cities[0].detail.clone().unwrap();
    assert_eq!(
        detail.idle_buildings,
        vec![crate::view::IdleBuilding {
            building: FORGE,
            resource: Resource::Iron
        }]
    );

    // Supply back: everyone works again, and the report says so.
    let mut back = lost.clone();
    back.map.get_mut(iron).unwrap().owner = Some(P0);
    back.map.get_mut(iron).unwrap().city = Some(a);
    back.map.get_mut(iron).unwrap().improved = true;
    let (back, events) = resolve_turn(&back, &BTreeMap::new());
    assert!(events.iter().any(|e| matches!(
        e,
        Event::SupplyShortage {
            idle: 0,
            supplied: 3,
            used: 3,
            ..
        }
    )));
    assert!(back.player(P0).supply.iter().all(|r| !r.idle));
}

#[test]
fn an_idle_forge_does_not_count_toward_a_building_objective() {
    let (mut s, a, iron) = swiss_iron_game();
    s.player_mut(P0).techs.insert(TechId::IronWorking);
    s.cities.get_mut(&a).unwrap().population = 4;
    give(&mut s, a, FORGE);
    // Counting began for the era before the Forge was built.
    s.player_mut(P0).eras.insert(1, Default::default());
    let objective = crate::objectives::Objective::Buildings {
        building: FORGE,
        cities: 1,
    };
    let (s, _) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(objective.progress(&s, P0, 1), 1);
    // Lose the deposit: the Forge stands but is idle, so it counts for nothing.
    let mut lost = s.clone();
    lost.map.get_mut(iron).unwrap().owner = None;
    lost.map.get_mut(iron).unwrap().city = None;
    let (lost, _) = resolve_turn(&lost, &BTreeMap::new());
    assert_eq!(idle_tier(lost.player(P0), a, FORGE), Some(Resource::Iron));
    assert_eq!(objective.progress(&lost, P0, 1), 0);
}

#[test]
fn an_unsupplied_unit_fights_a_quarter_weaker() {
    let (mut s, _, iron) = iron_game();
    let pike = spawn_unit(&mut s, P0, UnitKind::Pikeman, at(5, 4));
    s.map.get_mut(iron).unwrap().resource = None;
    let (s, _) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(unit_unsupplied(&s, &s.units[&pike]), Some(Resource::Iron));
    let mut b = Breakdown::default();
    add_unsupplied(
        &mut b,
        Faction::Byzantium,
        UnitKind::Pikeman,
        false,
        Some(Resource::Iron),
    );
    let base = data::unit_stats_for(Faction::Byzantium, UnitKind::Pikeman).strength;
    assert_eq!(b.parts, vec![("No Iron supply".to_string(), -(base / 4))]);
}

#[test]
fn a_queued_pikeman_waits_for_supply_and_starts_when_it_is_spare() {
    let (mut s, id, iron) = swiss_iron_game();
    spawn_unit(&mut s, P0, UnitKind::Pikeman, at(5, 4));
    spawn_unit(&mut s, P0, UnitKind::Pikeman, at(4, 5));
    // Queuing behind other work is allowed even with no Iron to spare yet.
    let os = orders(
        P0,
        vec![Order::SetProductionQueue {
            city: id,
            queue: vec![SOLDIER, TROOPER],
        }],
    );
    assert!(validate_orders(&s, P0, &os[&P0]).is_empty());
    let (mut next, _) = resolve_turn(&s, &os);
    assert_eq!(next.cities[&id].producing, Some(SOLDIER));
    next.cities.get_mut(&id).unwrap().production_stored = 100;
    let (after, events) = resolve_turn(&next, &BTreeMap::new());
    assert_eq!(after.cities[&id].producing, None);
    assert_eq!(after.cities[&id].production_queue, vec![TROOPER]);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::OrderFailed { reason, .. } if reason.contains("No Iron to spare") && reason.contains("kept in queue")
    )));
    // A Mine adds the supply: the queue starts it.
    let mut mined = after.clone();
    mined.map.get_mut(iron).unwrap().improved = true;
    let (started, _) = resolve_turn(&mined, &BTreeMap::new());
    assert_eq!(started.cities[&id].producing, Some(TROOPER));
}

#[test]
fn buying_a_pikeman_needs_spare_iron() {
    let (mut s, id, _) = swiss_iron_game();
    spawn_unit(&mut s, P0, UnitKind::Pikeman, at(5, 4));
    spawn_unit(&mut s, P0, UnitKind::Pikeman, at(4, 5));
    let reason = crate::economy::buy_blocker(&s, &s.cities[&id], TROOPER).unwrap();
    assert!(reason.starts_with("No Iron to spare"), "{reason}");
    s.map.get_mut(at(6, 5)).unwrap().improved = true;
    let buy = orders(
        P0,
        vec![Order::BuyItem {
            city: id,
            item: TROOPER,
        }],
    );
    assert!(validate_orders(&s, P0, &buy[&P0]).is_empty());
    let (next, _) = resolve_turn(&s, &buy);
    assert_eq!(in_use(&next, P0, Resource::Iron), 3);
}

#[test]
fn upgrades_into_classical_units_take_iron_supply_in_list_order() {
    let (mut s, id, _) = swiss_iron_game();
    // Keep the line at its Classical form; Feudalism would make one order reach Pikeman.
    s.player_mut(P0).techs.remove(&TechId::Feudalism);
    s.player_mut(P0).techs.insert(TechId::Mathematics);
    let a = spawn_unit(&mut s, P0, UnitKind::Spearman, at(5, 4));
    let b = spawn_unit(&mut s, P0, UnitKind::Spearman, at(4, 5));
    let c = spawn_unit(&mut s, P0, UnitKind::Spearman, at(5, 6));
    let v = player_view(&s, P0);
    let unit = v.units.iter().find(|u| u.id == a).unwrap();
    assert_eq!(unit.upgrade_need, Some(Resource::Iron));
    let os = vec![
        Order::UpgradeUnit { unit: a },
        Order::UpgradeUnit { unit: b },
        Order::UpgradeUnit { unit: c },
    ];
    let errors = validate_orders(&s, P0, &os);
    assert_eq!(errors.iter().map(|e| e.index).collect::<Vec<_>>(), vec![2]);
    assert!(errors[0].message.contains("no Iron to spare"), "{errors:?}");
    // Production sees the supply the upgrades took.
    let mixed = vec![
        Order::UpgradeUnit { unit: a },
        Order::UpgradeUnit { unit: b },
        Order::SetProduction {
            city: id,
            item: TROOPER,
        },
    ];
    let errors = validate_orders(&s, P0, &mixed);
    assert_eq!(errors.iter().map(|e| e.index).collect::<Vec<_>>(), vec![2]);
    let (next, _) = resolve_turn(&s, &orders(P0, os[..2].to_vec()));
    assert_eq!(next.units[&a].kind, UnitKind::Phalangite);
    assert_eq!(next.units[&b].kind, UnitKind::Phalangite);
    let err = crate::rules::units::check_upgrade(&next, P0, c, 1000).unwrap_err();
    assert!(
        err.starts_with("upgrading to Phalangite: no Iron to spare"),
        "{err}"
    );
}

#[test]
fn worked_resources_add_their_bonus_and_strategic_ones_only_once_known() {
    let (mut s, id, iron) = iron_game();
    // Everything around the Iron tile is ocean (a city works all its owned tiles in range).
    s.cities.get_mut(&id).unwrap().population = 1;
    for h in at(5, 5).within(2) {
        if h != at(5, 5) && h != iron {
            s.map.get_mut(h).unwrap().terrain = Terrain::Ocean;
        }
    }
    s.map.get_mut(iron).unwrap().terrain = Terrain::Hills;
    let y = crate::economy::city_yields(&s, &s.cities[&id]);
    assert!(y.worked.contains(&iron), "{:?}", y.worked);
    // A resource yields nothing until the tile is improved.
    assert!(y.sources.iter().all(|x| !x.label.starts_with("Resources")));
    s.map.get_mut(iron).unwrap().improved = true;
    let y = crate::economy::city_yields(&s, &s.cities[&id]);
    let part = y
        .sources
        .iter()
        .find(|x| x.label == "Resources: Iron (Deep Mine)")
        .unwrap();
    assert_eq!(part.yields, Yields::new(0, 2, 0, 0));
    s.player_mut(P0).techs.clear();
    let y = crate::economy::city_yields(&s, &s.cities[&id]);
    assert!(y.sources.iter().all(|x| !x.label.starts_with("Resources")));
    // A bonus resource works without any technology.
    s.map.get_mut(iron).unwrap().resource = Some(Resource::Copper);
    let y = crate::economy::city_yields(&s, &s.cities[&id]);
    let part = y
        .sources
        .iter()
        .find(|x| x.label == "Resources: Copper (Mine)")
        .unwrap();
    assert_eq!(part.yields, Yields::new(0, 0, 2, 0));
}

#[test]
fn views_show_strategic_resources_only_to_empires_that_know_them() {
    let (mut s, _, iron) = iron_game();
    let wheat = at(4, 5);
    s.map.get_mut(wheat).unwrap().resource = Some(Resource::Wheat);
    let far = at(15, 15);
    s.map.get_mut(far).unwrap().resource = Some(Resource::Coal);
    crate::visibility::update_explored(&mut s);
    let v0 = player_view(&s, P0);
    assert_eq!(v0.tile(iron).unwrap().resource, Some(Resource::Iron));
    assert_eq!(v0.tile(wheat).unwrap().resource, Some(Resource::Wheat));
    assert_eq!(v0.tile(far).unwrap().resource, None, "unexplored");
    // P1 has explored the same tiles but not Fabrication: Iron stays hidden, Wheat doesn't.
    for i in 0..s.map.len() {
        s.player_mut(P1).explored.set(i);
    }
    let v1 = player_view(&s, P1);
    assert!(v1.tile(iron).unwrap().explored);
    assert_eq!(v1.tile(iron).unwrap().resource, None);
    assert_eq!(v1.tile(wheat).unwrap().resource, Some(Resource::Wheat));
    assert_eq!(
        v1.tile(far).unwrap().resource,
        None,
        "Coal needs Industrialization"
    );
    s.player_mut(P1).techs.insert(TechId::IronWorking);
    s.player_mut(P1).techs.insert(TechId::Industrialization);
    let v1 = player_view(&s, P1);
    assert!(!v1.tile(far).unwrap().visible, "remembered like terrain");
    assert_eq!(v1.tile(far).unwrap().resource, Some(Resource::Coal));
    assert_eq!(v1.tile(iron).unwrap().resource, Some(Resource::Iron));
}

#[test]
fn a_new_source_is_announced_to_its_team_only() {
    let (mut s, id, _) = iron_game();
    s.player_mut(P0).techs.extend(crate::eras::era_techs(0));
    s.player_mut(P0).techs.remove(&TechId::IronWorking);
    s.player_mut(P0).researching = Some(TechId::IronWorking);
    s.player_mut(P0).research_queue = vec![TechId::IronWorking];
    s.player_mut(P0).science_stored = 1000;
    let (next, events) = resolve_turn(&s, &BTreeMap::new());
    assert!(next.player(P0).techs.contains(&TechId::IronWorking));
    let found: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::ResourcesRevealed { .. }))
        .collect();
    assert_eq!(
        found,
        vec![&Event::ResourcesRevealed {
            player: P0,
            resource: Resource::Iron,
            city: Some(id),
            sources: 1,
            supply: 2,
        }]
    );
    let before = player_view(&s, P1);
    let after = player_view(&next, P1);
    let seen = crate::event_visibility::visible_events(&events, &before, &after);
    assert!(
        !serde_json::to_string(&seen)
            .unwrap()
            .contains("ResourcesRevealed")
    );

    // A reveal with nothing inside the borders still says so.
    s.map.get_mut(at(6, 5)).unwrap().resource = None;
    let (_, events) = resolve_turn(&s, &BTreeMap::new());
    assert!(events.contains(&Event::ResourcesRevealed {
        player: P0,
        resource: Resource::Iron,
        city: None,
        sources: 0,
        supply: 0,
    }));
}

#[test]
fn resource_free_states_serialize_as_before() {
    let (mut s, _, iron) = iron_game();
    s.map.get_mut(iron).unwrap().resource = None;
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.contains("\"resource\""));
    assert!(!json.contains("\"supply\""));
    // A state without strategic users round-trips.
    let back: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(back, s);
    assert!(back.player(P0).supply.is_empty());
}

fn setup(seed: u64) -> GameSetup {
    GameSetup::standard(seed, [Faction::Sparta, Faction::Byzantium])
}

#[test]
fn mapgen_places_fair_guarantees_and_borderland_era_resources() {
    for seed in 1..=60 {
        let s = setup(seed);
        let (map, starts) = mapgen::generate(&s);
        mapgen::validate(&map, &starts).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        placement::validate_rivals(&map, &starts, &s)
            .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        let land = map.tiles.iter().filter(|t| t.terrain.is_land()).count();
        let count = |r: Resource| map.tiles.iter().filter(|t| t.resource == Some(r)).count();
        let bonus: usize = [Resource::Wheat, Resource::Cattle, Resource::Stone]
            .into_iter()
            .map(count)
            .sum();
        let target = (land / placement::LAND_TILES_PER_BONUS).max(2 * starts.len());
        // Bonus resources never touch another deposit, and the era resources take room: the
        // fill reaches about three quarters of its target, and never less than 60% (Wheat, Cattle
        // and Stone; Copper has a separate placement pass and is not one of the start's food/production bonuses).
        assert!(
            bonus <= target && bonus * 5 + 10 >= target * 3,
            "seed {seed}: {bonus} bonus for {land} land"
        );
        assert!(count(Resource::Iron) >= starts.len(), "seed {seed}");
        assert!(count(Resource::Copper) >= starts.len(), "seed {seed}");
        for r in placement::BORDERLAND {
            assert_eq!(
                count(r),
                placement::border_target(land, starts.len()),
                "seed {seed}: {r:?}"
            );
        }
        for (seat, &start) in starts.iter().enumerate() {
            let iron = map
                .hexes()
                .filter(|&h| map.get(h).unwrap().resource == Some(Resource::Iron))
                .filter(|&h| nearest_start(&starts, h).unwrap().0 == seat)
                .map(|h| start.distance(h))
                .min()
                .unwrap();
            assert!(
                (IRON_MIN_DISTANCE..=IRON_MAX_DISTANCE).contains(&iron),
                "seed {seed}"
            );
        }
        // Nothing but a start's own guarantees lies within the clear radius of a start.
        for h in map.hexes() {
            let t = map.get(h).unwrap();
            let Some(r) = t.resource else { continue };
            let (_, d) = nearest_start(&starts, h).unwrap();
            if d <= CLEAR_RADIUS && r != Resource::Iron && r != Resource::Copper {
                assert!(d <= 2 && !is_strategic(r), "seed {seed}: {r:?} at {d}");
            }
        }
    }
}

#[test]
fn every_start_gets_the_same_quality_from_its_bonus_resources() {
    let mut fallbacks = 0;
    for seed in 1..=30 {
        let s = setup(seed);
        let (mut map, starts) = mapgen::generate(&s);
        let original: Vec<_> = map.tiles.iter().map(|t| t.resource).collect();
        let with: Vec<i32> = (0..starts.len())
            .map(|i| mapgen::start_quality(&map, &starts, &s, i))
            .collect();
        for t in &mut map.tiles {
            t.resource = None;
        }
        for (i, q) in with.iter().enumerate() {
            let without = mapgen::start_quality(&map, &starts, &s, i);
            // Every start gets two inner-ring Food/Production bonuses. Their quality gain
            // now depends on whether they improve the local Food/Production balance.
            assert!(
                (3..=9).contains(&(q - without)),
                "seed {seed}, seat {i}: {}",
                q - without
            );
            fallbacks += starts[i]
                .within(2)
                .into_iter()
                .filter(|h| original[map.index(*h).unwrap()] == Some(Resource::Copper))
                .count();
        }
    }
    assert!(fallbacks >= 30, "every start has its Copper 2 hexes out");
}

#[test]
fn placement_is_deterministic() {
    let s = setup(77);
    assert_eq!(mapgen::generate(&s), mapgen::generate(&s));
}

#[test]
fn each_era_uses_its_own_resource() {
    use UnitKind::*;
    // Turn-1 units and civilians are free; from the Classical era on a unit uses its era's.
    for free in [Spearman, Archer, Horseman, Scout, Settler] {
        assert_eq!(unit_need(free), None, "{free:?}");
    }
    for (kind, r) in [
        (Phalangite, Resource::Iron),
        (Cataphract, Resource::Iron),
        (Pikeman, Resource::Iron),
        (Knight, Resource::Iron),
        (Musketman, Resource::Niter),
        (Arquebusier, Resource::Niter),
        (Rifleman, Resource::Coal),
        (Infantry, Resource::Oil),
        (Tank, Resource::Oil),
        (Fighter, Resource::Oil),
        (Bomber, Resource::Oil),
        (HunterKiller, Resource::Silicon),
        (CombatDrone, Resource::Silicon),
    ] {
        assert_eq!(unit_need(kind), Some(r), "{kind:?}");
    }
    use BuildingKind::*;
    for (b, r) in [
        (Forge, Resource::Iron),
        (University, Resource::Iron),
        (Bank, Resource::Niter),
        (Factory, Resource::Coal),
        (ResearchLab, Resource::Oil),
    ] {
        assert_eq!(building_need(b), Some(r), "{b:?}");
    }
    for free in [
        Granary, Market, Workshop, Library, Walls, Castle, StarFort, Mill, Roadworks,
    ] {
        assert_eq!(building_need(free), None, "{free:?}");
    }
    // An upgrade into another era's resource needs that one; within Iron it needs nothing new.
    assert_eq!(upgrade_need(Spearman), Some(Resource::Iron));
    assert_eq!(upgrade_need(Phalangite), None);
    assert_eq!(upgrade_need(Pikeman), Some(Resource::Niter));
    // Copper is a visible bonus tile with no supply users.
    assert!(known_to(Resource::Copper, &BTreeSet::new()));
    assert!(!is_strategic(Resource::Copper));
}

#[test]
fn an_upgrade_frees_its_old_resource_and_is_the_newest_user_of_the_new_one() {
    let (mut s, _, _) = swiss_iron_game();
    let niter = at(4, 5);
    s.map.get_mut(niter).unwrap().resource = Some(Resource::Niter);
    s.player_mut(P0)
        .techs
        .extend([TechId::Education, TechId::Gunpowder]);
    let a = spawn_unit(&mut s, P0, UnitKind::Pikeman, at(5, 4));
    let (s, _) = resolve_turn(&s, &orders(P0, vec![]));
    assert_eq!(in_use(&s, P0, Resource::Iron), 1);
    let since = s.player(P0).supply[0].since;
    let (s, _) = resolve_turn(&s, &orders(P0, vec![Order::UpgradeUnit { unit: a }]));
    assert_eq!(s.units[&a].kind, UnitKind::Musketman);
    assert_eq!(in_use(&s, P0, Resource::Iron), 0);
    assert_eq!(in_use(&s, P0, Resource::Niter), 1);
    let record = s.player(P0).supply[0];
    assert_eq!(record.resource, Resource::Niter);
    assert!(record.since > since, "a new Niter user");
}

#[test]
fn a_market_and_bank_do_not_need_copper() {
    let (mut s, id, _) = swiss_iron_game();
    s.player_mut(P0)
        .techs
        .extend([TechId::Currency, TechId::Gunpowder, TechId::Banking]);
    let niter = at(4, 5);
    s.map.get_mut(niter).unwrap().resource = Some(Resource::Niter);
    give(&mut s, id, BuildingKind::Bank);
    // Copper has no supply role. Niter supplies the Bank; the Market needs nothing.
    let (s, _) = resolve_turn(&s, &orders(P0, vec![]));
    let player = s.player(P0);
    assert_eq!(idle_tier(player, id, BuildingKind::Bank), None);
    assert_eq!(idle_tiers(player, &s.cities[&id]), vec![]);
}

#[test]
fn later_resources_have_their_own_improvements() {
    use crate::improvements::{kind_for, name};
    assert_eq!(name(kind_for(Resource::Oil), true), "Oil Well");
    assert_eq!(name(kind_for(Resource::Niter), true), "Saltpetre Works");
    assert_eq!(name(kind_for(Resource::Silicon), true), "Quartz Mine");
    assert_eq!(
        serde_json::from_str::<Resource>("\"Lithium\"").unwrap(),
        Resource::Silicon
    );
    assert_eq!(
        serde_json::from_str::<ImprovementKind>("\"LithiumMine\"").unwrap(),
        ImprovementKind::QuartzMine
    );
    assert_eq!(name(kind_for(Resource::Iron), false), "Mine");
}

#[test]
fn hangar_aircraft_reserve_oil_and_appear_in_supply_breakdowns() {
    let (mut s, city, _) = swiss_iron_game();
    s.player_mut(P0)
        .techs
        .extend([TechId::Combustion, TechId::Flight]);
    let oil = at(4, 5);
    s.map.get_mut(oil).unwrap().resource = Some(Resource::Oil);
    crate::rules::air::base_new(&mut s, city, UnitKind::Fighter);
    let fighter = s.aircraft[0].id;
    assert_eq!(in_use(&s, P0, Resource::Oil), 1);
    assert_eq!(
        current_users(&s, P0),
        vec![(SupplyUser::Unit { unit: fighter }, Resource::Oil)]
    );
    let bomber = ProductionItem::Unit {
        unit: UnitKind::Bomber,
    };
    assert_eq!(start_blocker(&s, &s.cities[&city], bomber), None);
    s.cities.get_mut(&city).unwrap().producing = Some(bomber);
    assert_eq!(reserved(&s, P0, Resource::Oil, None), vec![(city, bomber)]);
    let oil_view = player_view(&s, P0)
        .supply
        .into_iter()
        .find(|v| v.resource == Resource::Oil)
        .unwrap();
    assert_eq!(oil_view.used.parts, vec![("Fighter".into(), 1)]);
    assert_eq!(oil_view.held, vec!["Bomber in City 1".to_string()]);
    crate::rules::air::base_new(&mut s, city, UnitKind::Bomber);
    assert!(start_blocker(&s, &s.cities[&city], bomber).is_some());
    assert_eq!(in_use(&s, P0, Resource::Oil), 2);
}

// ---- Siege (user decision 2026-10-22): a Gold upgrade that uses 1 Iron ----

/// `iron_game` (2 Iron supply) with Mathematics and two Spearmen in own land.
fn siege_game() -> (GameState, UnitId, UnitId) {
    let (mut s, _, _) = iron_game();
    s.player_mut(P0).techs.insert(TechId::Mathematics);
    assert_eq!(supplied(&s, P0, Resource::Iron), 2);
    let a = spawn_unit(&mut s, P0, UnitKind::Spearman, at(5, 4));
    let b = spawn_unit(&mut s, P0, UnitKind::Spearman, at(4, 5));
    (s, a, b)
}

fn siege(unit: UnitId) -> Order {
    Order::UpgradeSiege { unit }
}

#[test]
fn siege_costs_gold_and_one_iron_and_the_unit_acts_next_turn() {
    let (s, a, _) = siege_game();
    let price = crate::rules::siege::price(&s, P0, UnitKind::Spearman).total;
    assert_eq!(
        price,
        data::SIEGE_BASE_GOLD,
        "an Ancient unit pays the base price"
    );
    let os = orders(P0, vec![siege(a)]);
    assert!(validate_orders(&s, P0, &os[&P0]).is_empty());
    let (next, events) = resolve_turn(&s, &os);
    let unit = &next.units[&a];
    assert!(unit.siege);
    assert_eq!(unit.moves_left, 0);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::SiegeBought { unit, owner, gold } if *unit == a && *owner == P0 && *gold == price
    )));
    assert_eq!(in_use(&next, P0, Resource::Iron), 1);
    assert_eq!(spare(&next, P0, Resource::Iron, None), 1);
    // Permanent: it keeps Siege through an upgrade.
    let mut upgraded = next.clone();
    upgraded.units.get_mut(&a).unwrap().kind = UnitKind::Phalangite;
    assert!(upgraded.units[&a].siege);
}

#[test]
fn siege_takes_iron_supply_in_list_order_and_refuses_the_one_that_does_not_fit() {
    let (mut s, a, b) = siege_game();
    let c = spawn_unit(&mut s, P0, UnitKind::Spearman, at(5, 6));
    let errors = validate_orders(&s, P0, &[siege(a), siege(b), siege(c)]);
    assert_eq!(errors.iter().map(|e| e.index).collect::<Vec<_>>(), vec![2]);
    assert!(errors[0].message.contains("no Iron to spare"), "{errors:?}");
    // A Mine on the deposit supplies a third one.
    let mut mined = s.clone();
    mined.map.get_mut(at(6, 5)).unwrap().improved = true;
    assert!(validate_orders(&mined, P0, &[siege(a), siege(b), siege(c)]).is_empty());
}

#[test]
fn newest_siege_units_beyond_the_supply_go_idle_and_fight_weaker() {
    let (mut s, a, b) = siege_game();
    let c = spawn_unit(&mut s, P0, UnitKind::Spearman, at(5, 6));
    let mut events = Vec::new();
    // Two units use the 2 Iron from turn 5; a third arrives on turn 9 and is the newest.
    s.turn = 5;
    for id in [a, b] {
        s.units.get_mut(&id).unwrap().siege = true;
    }
    update_supply(&mut s, &mut events);
    s.turn = 9;
    s.units.get_mut(&c).unwrap().siege = true;
    update_supply(&mut s, &mut events);
    assert_eq!(unit_unsupplied(&s, &s.units[&a]), None);
    assert_eq!(unit_unsupplied(&s, &s.units[&b]), None);
    assert_eq!(unit_unsupplied(&s, &s.units[&c]), Some(Resource::Iron));
    let view = player_view(&s, P0);
    let idle = view.units.iter().find(|u| u.id == c).unwrap();
    assert_eq!(idle.unsupplied, Some(Resource::Iron));
    assert!(idle.unsupplied_loss.unwrap() > 0);
}

#[test]
fn siege_asks_for_mathematics_territory_gold_a_land_unit_and_only_once() {
    let (mut s, a, _) = siege_game();
    let refused = |s: &GameState, unit: UnitId| {
        validate_orders(s, P0, &[siege(unit)])
            .first()
            .map(|e| e.message.clone())
    };
    assert_eq!(refused(&s, a), None);
    // No Mathematics: no Siege, and the card says why.
    let mut early = s.clone();
    early.player_mut(P0).techs.remove(&TechId::Mathematics);
    assert!(refused(&early, a).unwrap().contains("Mathematics"));
    // Outside own land.
    let mut abroad = s.clone();
    abroad.units.get_mut(&a).unwrap().pos = at(12, 8);
    assert!(refused(&abroad, a).unwrap().contains("own territory"));
    // Gold.
    let mut poor = s.clone();
    poor.player_mut(P0).gold = 10;
    assert!(refused(&poor, a).unwrap().contains("Gold"));
    // Scouts and Settlers take none; aircraft are not land units.
    let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(5, 6));
    assert!(refused(&s, scout).unwrap().contains("cannot have Siege"));
    // Once.
    s.units.get_mut(&a).unwrap().siege = true;
    assert!(refused(&s, a).unwrap().contains("already has Siege"));
}

#[test]
fn siege_is_built_in_from_the_industrial_era_never_bought_and_dropped_on_upgrade() {
    let (mut s, a, _) = siege_game();
    s.map.get_mut(at(4, 5)).unwrap().resource = Some(Resource::Coal);
    s.player_mut(P0).techs.extend([
        TechId::Industrialization,
        TechId::Gunpowder,
        TechId::Rifling,
    ]);
    // A Rifleman is refused in plain words, and its card offers nothing.
    let rifle = spawn_unit(&mut s, P0, UnitKind::Rifleman, at(5, 6));
    let reason = validate_orders(&s, P0, &[siege(rifle)])
        .first()
        .map(|e| e.message.clone())
        .unwrap();
    assert!(
        reason.contains(
            "Rifleman needs no Siege: it ignores Walls, Castles and Star Forts (Industrial-era guns)"
        ),
        "{reason}"
    );
    let view = player_view(&s, P0);
    let card = view.units.iter().find(|u| u.id == rifle).unwrap();
    assert!(card.siege_offer.is_none());
    assert!(!card.siege);
    // A Musketman that bought Siege upgrades into a Rifleman: the bought Siege goes, and with
    // it the Iron it used.
    {
        let u = s.units.get_mut(&a).unwrap();
        u.kind = UnitKind::Musketman;
        u.siege = true;
    }
    let mut events = Vec::new();
    update_supply(&mut s, &mut events);
    assert_eq!(in_use(&s, P0, Resource::Iron), 1);
    let os = orders(P0, vec![Order::UpgradeUnit { unit: a }]);
    assert!(validate_orders(&s, P0, &os[&P0]).is_empty());
    let (next, _) = resolve_turn(&s, &os);
    assert_eq!(next.units[&a].kind, UnitKind::Rifleman);
    assert!(!next.units[&a].siege, "built in now");
    assert_eq!(in_use(&next, P0, Resource::Iron), 0);
    assert!(
        next.player(P0)
            .supply
            .iter()
            .all(|r| r.user != SupplyUser::Siege { unit: a })
    );
}

#[test]
fn military_engineering_halves_the_siege_price_and_later_units_pay_more() {
    let (mut s, _, _) = siege_game();
    let ancient = crate::rules::siege::price(&s, P0, UnitKind::Spearman).total;
    s.player_mut(P0).techs.insert(TechId::MilitaryEngineering);
    assert_eq!(
        crate::rules::siege::price(&s, P0, UnitKind::Spearman).total,
        ancient / 2
    );
    // Era-scaled on the units' own curve: about flat to the Medieval era, dearer after.
    assert!(data::siege_gold(UnitKind::Musketman) > data::siege_gold(UnitKind::Spearman));
    assert!(data::siege_gold(UnitKind::Infantry) > data::siege_gold(UnitKind::Musketman));
}

#[test]
fn the_mongols_buy_siege_only_from_the_medieval_era() {
    let (mut s, a, _) = siege_game();
    s.player_mut(P0).faction = Faction::Mongols;
    let reason = validate_orders(&s, P0, &[siege(a)])
        .first()
        .map(|e| e.message.clone())
        .unwrap();
    assert!(reason.contains("Medieval era"), "{reason}");
    for tech in [TechId::BronzeWorking, TechId::Mathematics] {
        s.player_mut(P0).techs.insert(tech);
    }
    let workshop_city = s.cities_of(P0).next().map(|c| c.id).unwrap();
    s.cities
        .get_mut(&workshop_city)
        .unwrap()
        .buildings
        .insert(BuildingKind::Workshop);
    assert!(!validate_orders(&s, P0, &[siege(a)]).is_empty());
    s.player_mut(P0)
        .techs
        .extend((0..2).flat_map(crate::eras::era_techs));
    assert!(validate_orders(&s, P0, &[siege(a)]).is_empty());
}

#[test]
fn the_unit_card_offers_siege_with_its_price_and_the_reason_it_cannot_be_bought() {
    let (mut s, a, _) = siege_game();
    let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(5, 6));
    let view = player_view(&s, P0);
    let offer = view
        .units
        .iter()
        .find(|u| u.id == a)
        .unwrap()
        .siege_offer
        .clone()
        .unwrap();
    assert_eq!(offer.price.total, data::SIEGE_BASE_GOLD);
    assert_eq!(offer.blocker, None);
    assert!(
        view.units
            .iter()
            .find(|u| u.id == scout)
            .unwrap()
            .siege_offer
            .is_none()
    );
    s.player_mut(P0).gold = 3;
    let poor = player_view(&s, P0)
        .units
        .iter()
        .find(|u| u.id == a)
        .unwrap()
        .siege_offer
        .clone()
        .unwrap();
    assert!(poor.blocker.unwrap().contains("Gold"));
    // A rival sees the siege gear, never the offer.
    s.units.get_mut(&a).unwrap().siege = true;
    let rival = player_view(&s, P1);
    if let Some(u) = rival.units.iter().find(|u| u.id == a) {
        assert!(u.siege);
        assert!(u.siege_offer.is_none());
    }
}

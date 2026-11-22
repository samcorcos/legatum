//! Proxy decisions must survive real turn resolution, including ranged-before-melee timing.
use legatum_ai::{Brain, ProxyV2Brain as ProxyBrain};
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
    let mut setup = GameSetup::standard(19, [Faction::Byzantium, Faction::India]);
    setup.width = 24;
    setup.height = 16;
    // Authored before cities fired back; city strike has its own tests in core.
    setup.players.truncate(2);
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.team = TeamId(i as u8);
    }
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    // Combat fixtures explicitly choose War under the integrated Peace default.
    // Keep Proxy v2's tactical assertions unchanged.
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = diplomacy::RelationState::War;
    }
    for (owner, h) in [(ME, at(4, 8)), (ENEMY, at(14, 8))] {
        let settler = spawn_unit(&mut state, owner, UnitKind::Settler, h);
        found_city(&mut state, settler, &mut Vec::new()).unwrap();
    }
    state.turn = 50;
    let city = state.city_at(at(14, 8)).unwrap().id;
    (state, city)
}

fn decide(state: &mut GameState) -> Vec<Order> {
    update_explored(state);
    let view = player_view(state, ME);
    let orders = ProxyBrain.orders(&view);
    assert!(validate_orders(state, ME, &orders).is_empty(), "{orders:?}");
    let loaded: GameState = serde_json::from_str(&serde_json::to_string(state).unwrap()).unwrap();
    assert_eq!(orders, ProxyBrain.orders(&player_view(&loaded, ME)));
    orders
}

#[test]
fn a_wounded_shooter_removes_the_garrison_and_infantry_captures_in_the_same_turn() {
    let (mut state, city) = scenario();
    state.cities.get_mut(&city).unwrap().hp = 15;
    let archer = spawn_unit(&mut state, ME, UnitKind::Archer, at(12, 8));
    state.units.get_mut(&archer).unwrap().hp = 45;
    let infantry = spawn_unit(&mut state, ME, UnitKind::Spearman, at(13, 8));
    let defender = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(14, 8));
    state.units.get_mut(&defender).unwrap().hp = 8;
    let ours = decide(&mut state);
    assert!(ours.contains(&Order::RangedAttack {
        unit: archer,
        target: at(14, 8)
    }));
    assert!(ours.iter().any(|o| matches!(o, Order::MoveUnitPath { unit, to, .. } if *unit == infantry && *to == at(14, 8))));
    let (next, events) = resolve_turn(
        &state,
        &BTreeMap::from([
            (ME, ours),
            (
                ENEMY,
                vec![Order::MoveUnit {
                    unit: defender,
                    to: at(12, 8),
                }],
            ),
        ]),
    );
    assert_eq!(next.cities[&city].owner, ME, "{events:?}");
    assert_eq!(next.units[&archer].hp, 45);
    assert!(!next.units.contains_key(&defender));
}

#[test]
fn a_doomed_shooter_returns_fire_instead_of_trying_to_outrun_the_ranged_phase() {
    let (mut state, _) = scenario();
    let ours = spawn_unit(&mut state, ME, UnitKind::Archer, at(11, 8));
    let theirs = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(13, 8));
    state.units.get_mut(&ours).unwrap().hp = 15;
    state.units.get_mut(&theirs).unwrap().hp = 10;
    let orders = decide(&mut state);
    assert!(orders.contains(&Order::RangedAttack {
        unit: ours,
        target: at(13, 8)
    }));
    let (next, events) = resolve_turn(
        &state,
        &BTreeMap::from([
            (ME, orders),
            (
                ENEMY,
                vec![Order::RangedAttack {
                    unit: theirs,
                    target: at(11, 8),
                }],
            ),
        ]),
    );
    assert!(!next.units.contains_key(&ours), "{events:?}");
    assert!(!next.units.contains_key(&theirs), "{events:?}");
}

#[test]
fn supporting_infantry_join_a_defended_city_assault() {
    let (mut state, city) = scenario();
    let a = spawn_unit(&mut state, ME, UnitKind::Pikeman, at(13, 8));
    let b = spawn_unit(&mut state, ME, UnitKind::Pikeman, at(14, 7));
    let ours = decide(&mut state);
    for unit in [a, b] {
        assert!(ours.iter().any(|o| matches!(o, Order::MoveUnitPath { unit: id, to, .. } if *id == unit && *to == at(14, 8))), "{ours:?}");
    }
    let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, ours)]));
    assert!(next.cities[&city].hp < 100, "{events:?}");
    assert!(next.units.contains_key(&a) && next.units.contains_key(&b));
}

#[test]
fn a_scout_occupies_an_exposed_defeated_city() {
    let (mut state, city) = scenario();
    state.cities.get_mut(&city).unwrap().hp = 0;
    let scout = spawn_unit(&mut state, ME, UnitKind::Scout, at(12, 8));
    let ours = decide(&mut state);
    let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, ours)]));
    assert_eq!(next.cities[&city].owner, ME, "{events:?}");
    assert_eq!(next.units[&scout].pos, at(14, 8));
}

#[test]
fn wounded_infantry_capture_before_stopping_to_heal() {
    let (mut state, city) = scenario();
    state.cities.get_mut(&city).unwrap().hp = 5;
    let infantry = spawn_unit(&mut state, ME, UnitKind::Spearman, at(13, 8));
    state.units.get_mut(&infantry).unwrap().hp = 40;
    let ours = decide(&mut state);
    let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, ours)]));
    assert_eq!(next.cities[&city].owner, ME, "{events:?}");
    assert!(next.units[&infantry].hp > 0);
}

#[test]
fn an_exposed_settler_is_captured_using_only_visible_units() {
    let (mut state, _) = scenario();
    let scout = spawn_unit(&mut state, ME, UnitKind::Scout, at(9, 8));
    let settler = spawn_unit(&mut state, ENEMY, UnitKind::Settler, at(11, 8));
    let ours = decide(&mut state);
    let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, ours)]));
    assert_eq!(next.units[&settler].owner, ME, "{events:?}");
    assert_eq!(next.units[&scout].pos, at(11, 8));
}

#[test]
fn an_affordable_defender_interrupts_construction_under_bombardment() {
    let (mut state, _) = scenario();
    let home = state.city_at(at(4, 8)).unwrap().id;
    state.player_mut(ME).techs.extend([
        TechId::BronzeWorking,
        TechId::IronWorking,
        TechId::Writing,
        TechId::Masonry,
        TechId::Feudalism,
    ]);
    state.player_mut(ME).gold = 500;
    // Pikemen also use Iron supply: a mined deposit supplies one to start and one to buy.
    let deposit = at(4, 8).neighbors()[0];
    state.map.get_mut(deposit).unwrap().resource = Some(Resource::Iron);
    state.map.get_mut(deposit).unwrap().improved = true;
    state.cities.get_mut(&home).unwrap().producing = Some(ProductionItem::Building {
        building: BuildingKind::Library,
    });
    state.cities.get_mut(&home).unwrap().production_stored = 40;
    let enemy = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(6, 8));
    let ours = decide(&mut state);
    assert!(
        ours.contains(&Order::SetProduction {
            city: home,
            item: ProductionItem::Unit {
                unit: UnitKind::Pikeman
            }
        }),
        "{ours:?}"
    );
    assert!(ours.contains(&Order::BuyItem {
        city: home,
        item: ProductionItem::Unit {
            unit: UnitKind::Pikeman
        }
    }));
    let (next, events) = resolve_turn(
        &state,
        &BTreeMap::from([
            (ME, ours),
            (
                ENEMY,
                vec![Order::RangedAttack {
                    unit: enemy,
                    target: at(4, 8),
                }],
            ),
        ]),
    );
    assert_eq!(next.cities[&home].owner, ME);
    assert!(
        next.units_of(ME).any(|u| u.kind == UnitKind::Pikeman),
        "{events:?}"
    );
}

#[test]
fn a_local_reinforcement_intercepts_a_threat_to_home() {
    let (mut state, _) = scenario();
    let guard = spawn_unit(&mut state, ME, UnitKind::Spearman, at(4, 11));
    spawn_unit(&mut state, ME, UnitKind::Archer, at(12, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(6, 8));
    let ours = decide(&mut state);
    let (next, _) = resolve_turn(&state, &BTreeMap::from([(ME, ours)]));
    assert!(next.units[&guard].pos.distance(at(4, 8)) <= 2);
}

#[test]
fn unseen_enemy_positions_cannot_change_proxy_orders() {
    let (mut state, _) = scenario();
    spawn_unit(&mut state, ME, UnitKind::Archer, at(12, 8));
    spawn_unit(&mut state, ME, UnitKind::Spearman, at(13, 8));
    let hidden = at(22, 1);
    let unit = spawn_unit(&mut state, ENEMY, UnitKind::Musketman, hidden);
    update_explored(&mut state);
    let before = player_view(&state, ME);
    assert!(!before.tile(hidden).unwrap().visible);
    let destination = at(22, 2);
    assert!(!before.tile(destination).unwrap().visible);
    // Army strength is public. Moving an existing hidden unit keeps that public
    // information constant while changing the inaccessible tactical state.
    state.units.get_mut(&unit).unwrap().pos = destination;
    update_explored(&mut state);
    let after = player_view(&state, ME);
    assert!(before == after, "hidden movement changed the public view");
    assert_eq!(ProxyBrain.orders(&before), ProxyBrain.orders(&after));
}

#[test]
fn a_great_work_uses_the_fast_city_even_when_it_is_busy() {
    let (mut state, _) = scenario();
    let slow = state.city_at(at(4, 8)).unwrap().id;
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, at(4, 13));
    found_city(&mut state, settler, &mut Vec::new()).unwrap();
    let fast = state.city_at(at(4, 13)).unwrap().id;
    state.player_mut(ME).techs.extend([
        TechId::Irrigation,
        TechId::BronzeWorking,
        TechId::Writing,
        TechId::Philosophy,
        TechId::IronWorking,
        TechId::Electricity,
        TechId::Currency,
        TechId::CodeOfLaws,
        TechId::Philosophy,
    ]);
    for id in [fast, slow] {
        state
            .cities
            .get_mut(&id)
            .unwrap()
            .buildings
            .insert(BuildingKind::Library);
    }
    let city = state.cities.get_mut(&fast).unwrap();
    city.buildings.insert(BuildingKind::Workshop);
    // Busy with a valid Settler (it needs population 2). The 40 stays banked with the
    // Settler, so its Forge (not transferable progress) makes this city faster.
    city.buildings.insert(BuildingKind::Forge);
    city.population = 4;
    city.production_stored = 40;
    city.producing = Some(ProductionItem::Unit {
        unit: UnitKind::Settler,
    });
    let ours = decide(&mut state);
    let work = ours.iter().find_map(|order| match order {
        Order::SetProduction {
            city,
            item:
                item @ ProductionItem::Wonder {
                    building: BuildingKind::GreatLibrary,
                    ..
                },
        } if *city == fast => Some(*item),
        _ => None,
    });
    assert!(work.is_some(), "{ours:?}");
    assert!(!ours.iter().any(|order| matches!(order,
        Order::SetProduction { city, item: ProductionItem::Wonder { building: BuildingKind::GreatLibrary, .. } }
        if *city == slow
    )));
    let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, ours)]));
    assert_eq!(next.cities[&fast].producing, work, "{events:?}");
    // The Settler's 40 waits for the next Settler; the Great Work starts from its own.
    let settler = ProductionItem::Unit {
        unit: UnitKind::Settler,
    };
    assert_eq!(next.cities[&fast].banked(settler), 40);
    assert!(next.cities[&fast].production_stored > 0);
}

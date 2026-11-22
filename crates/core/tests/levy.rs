//! Thin levies: the Byzantium (Byzantium) fields at most one armed unit per city, plus one.
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::{Aircraft, Map};
use legatum_core::*;

const P: PlayerId = PlayerId(0);

fn state(faction: Faction) -> GameState {
    let mut setup = GameSetup::all_ai(5);
    setup.players.truncate(2);
    setup.players[0].faction = faction;
    let mut s = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    s.player_mut(P).techs.extend(TechId::ALL);
    s.player_mut(P).gold = 1000;
    s
}

fn found(s: &mut GameState, col: i32, row: i32) -> CityId {
    let settler = spawn_unit(s, P, UnitKind::Settler, Hex::from_offset(col, row));
    rules::cities::found_city(s, settler, &mut vec![]).unwrap()
}

fn spearman() -> ProductionItem {
    ProductionItem::Unit {
        unit: UnitKind::Spearman,
    }
}

#[test]
fn the_cap_is_one_armed_unit_per_city_plus_one_and_scouts_do_not_count() {
    let mut s = state(Faction::Byzantium);
    let home = found(&mut s, 4, 4);
    assert_eq!(economy::levy(&s, P), Some((0, 2)));
    spawn_unit(&mut s, P, UnitKind::Scout, Hex::from_offset(6, 6));
    spawn_unit(&mut s, P, UnitKind::Spearman, Hex::from_offset(5, 4));
    assert_eq!(economy::levy(&s, P), Some((1, 2)), "the Scout is free");
    assert_eq!(
        economy::production_blocker(&s, &s.cities[&home], spearman(), true),
        None
    );
    spawn_unit(&mut s, P, UnitKind::Archer, Hex::from_offset(3, 4));
    let blocker = economy::production_blocker(&s, &s.cities[&home], spearman(), true);
    assert_eq!(blocker.as_deref(), Some("Levies full: 2 of 2"));
    // Civilians and Scouts are never blocked.
    for unit in [UnitKind::Scout, UnitKind::Settler] {
        let item = ProductionItem::Unit { unit };
        let reason = economy::production_blocker(&s, &s.cities[&home], item, true);
        assert!(!reason.is_some_and(|r| r.contains("Levies")), "{unit:?}");
    }
    // A second city lifts the cap by one.
    found(&mut s, 12, 8);
    assert_eq!(economy::levy(&s, P), Some((2, 3)));
    assert_eq!(
        economy::production_blocker(&s, &s.cities[&home], spearman(), true),
        None
    );
}

#[test]
fn losing_a_city_never_disbands_units_it_only_blocks_new_ones() {
    let mut s = state(Faction::Byzantium);
    let home = found(&mut s, 4, 4);
    let second = found(&mut s, 12, 8);
    for i in 0..3 {
        spawn_unit(&mut s, P, UnitKind::Spearman, Hex::from_offset(5 + i, 4));
    }
    assert_eq!(economy::levy(&s, P), Some((3, 3)));
    s.cities.remove(&second);
    assert_eq!(economy::levy(&s, P), Some((3, 2)));
    let (next, _) = resolve_turn(&s, &Default::default());
    assert_eq!(economy::levy_count(&next, P), 3, "no unit is disbanded");
    assert!(economy::production_blocker(&next, &next.cities[&home], spearman(), false).is_some());
}

#[test]
fn a_full_levy_blocks_buying_and_finishing_but_other_factions_are_uncapped() {
    let mut s = state(Faction::Byzantium);
    let home = found(&mut s, 4, 4);
    spawn_unit(&mut s, P, UnitKind::Spearman, Hex::from_offset(5, 4));
    spawn_unit(&mut s, P, UnitKind::Spearman, Hex::from_offset(3, 4));
    let errors = validate_orders(
        &s,
        P,
        &[Order::BuyItem {
            city: home,
            item: spearman(),
        }],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        economy::production_blocker(&s, &s.cities[&home], spearman(), true).as_deref(),
        Some("Levies full: 2 of 2")
    );
    let view = player_view(&s, P);
    let public = view.players.iter().find(|p| p.id == P).unwrap();
    assert_eq!(public.levy.map(|l| (l.units, l.cap)), Some((2, 2)));

    let mut t = state(Faction::Switzerland);
    let home = found(&mut t, 4, 4);
    for i in 0..5 {
        spawn_unit(&mut t, P, UnitKind::Spearman, Hex::from_offset(5 + i, 5));
    }
    assert_eq!(economy::levy(&t, P), None);
    assert_eq!(
        economy::production_blocker(&t, &t.cities[&home], spearman(), true),
        None
    );
}

#[test]
fn aircraft_count_toward_thin_levies_and_cannot_be_built_at_the_cap() {
    let mut s = state(Faction::Byzantium);
    let home = found(&mut s, 4, 4);
    spawn_unit(&mut s, P, UnitKind::Spearman, Hex::from_offset(5, 4));
    s.aircraft.push(Aircraft {
        id: UnitId(s.next_unit_id + 1),
        kind: UnitKind::Fighter,
        city: home,
        hp: 100,
        xp: 0,
    });

    assert_eq!(economy::levy(&s, P), Some((2, 2)));
    assert_eq!(economy::upkeep_unit_count(&s, P), 1);
    assert_eq!(
        economy::upkeep(&s, P),
        0,
        "the Fighter uses a levy slot but no free upkeep slot"
    );
    assert_eq!(
        economy::production_blocker(&s, &s.cities[&home], spearman(), true).as_deref(),
        Some("Levies full: 2 of 2")
    );
}

#[test]
fn the_byzantines_buildings_cost_the_ordinary_price_again() {
    let s = state(Faction::Byzantium);
    for building in BuildingKind::ALL {
        assert_eq!(
            data::building_stats_for(Faction::Byzantium, building).cost,
            data::building_stats_for(Faction::India, building).cost,
            "{building:?}"
        );
    }
    assert_eq!(
        economy::production_cost(
            &s,
            P,
            ProductionItem::Building {
                building: BuildingKind::Workshop
            }
        ),
        60
    );
}

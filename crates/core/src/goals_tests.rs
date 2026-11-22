use super::*;
use crate::economy;
use crate::events::Event;
use crate::game::spawn_unit;
use crate::hex::Hex;
use crate::rules::cities::tests::{P0, P1, P2, at, found, test_game};
use crate::scoring;

fn factory() -> ProductionItem {
    ProductionItem::Building {
        building: BuildingKind::MachineShop,
    }
}

fn heir_item() -> ProductionItem {
    ProductionItem::Unit {
        unit: UnitKind::Heir,
    }
}

fn run(state: &mut GameState) -> Vec<Event> {
    let mut ev = Vec::new();
    scoring::update(state, &mut ev);
    ev
}

/// The viewer's own meter: `(value, target)`.
fn pv(state: &GameState) -> (i32, i32) {
    let v = view(state, P0).expect("a meter");
    (v.value, v.target)
}

fn give_factory(state: &mut GameState, city: CityId) {
    state
        .cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::MachineShop);
}

/// Five Germany cities, each with a Ruhr Foundry, as P0.
fn forge(state: &mut GameState) -> Vec<CityId> {
    let ids: Vec<CityId> = (0..5)
        .map(|i| found(state, P0, at(2 + 4 * (i % 4), 2 + 8 * (i / 4))))
        .collect();
    for id in &ids {
        give_factory(state, *id);
    }
    ids
}

fn goal_vp(state: &GameState, p: PlayerId) -> Vec<PlayerId> {
    scoring::vp(state, p)
        .into_iter()
        .filter_map(|(src, _)| match src {
            VpSource::FactionGoal { of } => Some(of),
            _ => None,
        })
        .collect()
}

/// An Heir of `of`, held by `owner`.
fn heir_for(state: &mut GameState, of: PlayerId, owner: PlayerId, pos: Hex) -> UnitId {
    let u = spawn_unit(state, owner, UnitKind::Heir, pos);
    state.units.get_mut(&u).unwrap().heir_of = Some(of);
    u
}

// ---------- Germany's Five Ruhr Foundries goal ----------

#[test]
fn the_ruhr_foundry_is_germanys_forge_replacement() {
    for f in Faction::ALL {
        // The Nile: Egypt has no Production, so the Workshop line, Factory included, is not its.
        assert_eq!(
            data::available_to_faction(f, factory()),
            f != Faction::Egypt,
            "{f:?}"
        );
    }
    assert_eq!(
        data::building_stats_for(Faction::Germany, BuildingKind::MachineShop)
            .yields
            .production,
        6
    );
    assert_eq!(
        data::building_stats_for(Faction::India, BuildingKind::MachineShop)
            .yields
            .production,
        3
    );
}

#[test]
fn germany_goal_needs_five_foundries_and_ignores_unbuilt_cities() {
    let mut s = test_game(&[(Faction::Germany, 0), (Faction::India, 1)]);
    let ids = forge(&mut s);
    let ev = run(&mut s);
    assert!(s.player(P0).faction_goal_done);
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted {
            player: P0,
            faction: Faction::Germany,
            vp: 2,
            again: false
        }
    )));
    assert_eq!(goal_vp(&s, P0), vec![P0]);
    assert_eq!(pv(&s), (5, 5));

    // A new city without a Foundry leaves the held goal intact.
    let sixth = found(&mut s, P0, at(6, 10));
    assert_eq!(pv(&s), (5, 5));
    assert!(
        !run(&mut s)
            .iter()
            .any(|e| matches!(e, Event::FactionGoalLost { player: P0, .. }))
    );
    assert!(s.player(P0).faction_goal_done);

    // Capturing an unbuilt city also leaves the goal intact.
    // P1 keeps its own original capital, so the capitals victory does not end the game here.
    found(&mut s, P1, at(14, 14));
    let captured = found(&mut s, P1, at(10, 10));
    s.cities.get_mut(&captured).unwrap().owner = P0;
    assert_eq!(pv(&s), (5, 5));
    run(&mut s);
    assert!(s.player(P0).faction_goal_done);

    // Losing a Foundry city drops the count below five, even with other cities (not P0's
    // original capital, whose loss would hand P1 the capitals victory).
    s.cities.remove(&ids[1]);
    let ev = run(&mut s);
    assert!(!s.player(P0).faction_goal_done);
    assert_eq!(pv(&s), (4, 5));
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::FactionGoalLost { player: P0, .. }))
    );
    assert!(goal_vp(&s, P0).is_empty());

    give_factory(&mut s, sixth);
    let ev = run(&mut s);
    assert!(s.player(P0).faction_goal_done);
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted {
            player: P0,
            again: true,
            ..
        }
    )));
}

#[test]
fn a_captured_factory_counts_only_for_a_germany() {
    let mut s = test_game(&[
        (Faction::Germany, 0),
        (Faction::Germany, 1),
        (Faction::India, 2),
    ]);
    let ids = forge(&mut s);
    run(&mut s);
    assert!(s.player(P0).faction_goal_done);
    // A rival Germany takes one: it keeps the Foundry, and the victim drops to four.
    s.cities.get_mut(&ids[0]).unwrap().owner = P1;
    for i in 0..4 {
        let c = found(&mut s, P1, at(2 + 4 * i, 17));
        give_factory(&mut s, c);
    }
    run(&mut s);
    assert!(!s.player(P0).faction_goal_done);
    assert!(
        s.player(P1).faction_goal_done,
        "the captured Ruhr Foundry counts"
    );
    // In a India's hands it does nothing: they have no Germany goal to satisfy.
    s.cities.get_mut(&ids[1]).unwrap().owner = P2;
    assert!(!reached(&s, P2));
}

// ---------- the Heir ----------

#[test]
fn only_the_byzantium_trains_an_heir_and_never_buys_one() {
    for f in Faction::ALL {
        assert_eq!(
            data::available_to_faction(f, heir_item()),
            f == Faction::Byzantium,
            "{f:?}"
        );
    }
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::India, 1)]);
    let a = found(&mut s, P0, at(3, 3));
    s.player_mut(P0).gold = 100_000;
    assert!(economy::can_produce(&s, &s.cities[&a], heir_item()));
    assert_eq!(economy::buy_cost_for(&s, &s.cities[&a], heir_item()), None);
    assert_eq!(
        economy::buy_block(&s, &s.cities[&a], heir_item()),
        Some(economy::BuyBlock::NotBuyable)
    );
    assert!(data::unit_stats(UnitKind::Heir).cost >= 2 * data::unit_stats(UnitKind::Marksman).cost);
    assert!(!data::unit_stats(UnitKind::Heir).military);
}

#[test]
fn one_heir_at_a_time_and_retrained_only_once_he_is_gone() {
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::India, 1)]);
    let a = found(&mut s, P0, at(3, 3));
    let b = found(&mut s, P0, at(10, 10));
    // Another city already building one blocks the second.
    s.cities.get_mut(&a).unwrap().producing = Some(heir_item());
    assert!(!economy::can_produce(&s, &s.cities[&b], heir_item()));
    assert!(economy::can_produce(&s, &s.cities[&a], heir_item()));
    // Completing him marks whose Heir he is.
    let mut ev = Vec::new();
    crate::rules::cities::produce(&mut s, a, 10_000, &mut ev);
    let heir = s
        .units
        .values()
        .find(|u| u.kind == UnitKind::Heir)
        .expect("trained");
    assert_eq!((heir.owner, heir.heir_of), (P0, Some(P0)));
    let id = heir.id;
    assert!(!economy::can_produce(&s, &s.cities[&b], heir_item()));
    // A rival holding him still blocks a new one: retake him.
    s.units.get_mut(&id).unwrap().owner = P1;
    let why = economy::production_blocker(&s, &s.cities[&b], heir_item(), true).unwrap();
    assert!(why.contains("retake"), "{why}");
    // Only once he no longer exists (disbanded, razed) can another be trained.
    s.units.remove(&id);
    assert!(economy::can_produce(&s, &s.cities[&b], heir_item()));
}

#[test]
fn the_heir_scores_for_whoever_holds_him() {
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::India, 1)]);
    found(&mut s, P0, at(3, 3));
    found(&mut s, P1, at(12, 12));
    let id = heir_for(&mut s, P0, P0, at(3, 4));
    let ev = run(&mut s);
    assert_eq!(goal_vp(&s, P0), vec![P0]);
    assert!(s.player(P0).faction_goal_done);
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted {
            player: P0,
            faction: Faction::Byzantium,
            vp: 2,
            ..
        }
    )));
    // A rival captures him: the points go with him, whoever's Heir he is.
    s.units.get_mut(&id).unwrap().owner = P1;
    let ev = run(&mut s);
    assert!(goal_vp(&s, P0).is_empty());
    assert_eq!(goal_vp(&s, P1), vec![P0]);
    assert_eq!(scoring::vp_total(&s, P1), 2);
    assert!(!s.player(P0).faction_goal_done);
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalLost {
            player: P0,
            faction: Faction::Byzantium,
            ..
        }
    )));
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted {
            player: P1,
            faction: Faction::Byzantium,
            again: true,
            ..
        }
    )));
    // Barbarians hold him: nobody scores.
    s.units.get_mut(&id).unwrap().owner = PlayerId::BARBARIAN;
    run(&mut s);
    assert!(goal_vp(&s, P0).is_empty() && goal_vp(&s, P1).is_empty());
    // Freed to the clearer, then retaken by his own empire.
    s.units.get_mut(&id).unwrap().owner = P1;
    run(&mut s);
    assert_eq!(goal_vp(&s, P1), vec![P0]);
    s.units.get_mut(&id).unwrap().owner = P0;
    run(&mut s);
    assert_eq!(goal_vp(&s, P0), vec![P0]);
    assert!(goal_vp(&s, P1).is_empty());
    // Disbanding him forfeits the VP.
    s.units.remove(&id);
    run(&mut s);
    assert!(goal_vp(&s, P0).is_empty());
    assert_eq!(pv(&s), (0, 1));
}

#[test]
fn each_byzantines_heir_scores_for_its_own_empire() {
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::Byzantium, 1)]);
    heir_for(&mut s, P0, P1, at(3, 3));
    heir_for(&mut s, P1, P1, at(8, 8));
    run(&mut s);
    assert_eq!(goal_vp(&s, P1), vec![P0, P1]);
    assert!(goal_vp(&s, P0).is_empty());
}

// ---------- the Courier Hall ----------

fn hall() -> ProductionItem {
    ProductionItem::Building {
        building: BuildingKind::CourierHall,
    }
}

fn hall_at(state: &GameState, city: CityId) -> ProductionItem {
    ProductionItem::Wonder {
        building: BuildingKind::CourierHall,
        tile: state.cities[&city].pos.neighbors()[0],
    }
}

fn transfer_hall_city(state: &mut GameState, city: CityId, owner: PlayerId) {
    state.cities.get_mut(&city).unwrap().owner = owner;
    for tile in &mut state.map.tiles {
        if tile.city == Some(city) {
            tile.owner = Some(owner);
        }
    }
}

/// Build P0's Hall in `city` the way production completes it.
fn raise_hall(state: &mut GameState, city: CityId) {
    let item = hall_at(state, city);
    state.cities.get_mut(&city).unwrap().producing = Some(item);
    let mut ev = Vec::new();
    crate::rules::cities::produce(state, city, 10_000, &mut ev);
}

#[test]
fn only_mongols_sees_or_builds_the_hall_and_it_is_pricey() {
    for f in Faction::ALL {
        assert_eq!(
            data::available_to_faction(f, hall()),
            f == Faction::Mongols,
            "{f:?}"
        );
    }
    assert!(
        data::building_stats_for(Faction::Mongols, BuildingKind::CourierHall).cost
            > data::building_stats(BuildingKind::MachineShop).cost
    );
    // No faction discount or surcharge touches the price.
    assert_eq!(
        data::building_stats_for(Faction::Byzantium, BuildingKind::CourierHall).cost,
        data::building_stats(BuildingKind::CourierHall).cost
    );
}

#[test]
fn one_hall_per_mongols_empire_rebuilt_only_if_it_no_longer_exists() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::India, 1)]);
    let a = found(&mut s, P0, at(3, 3));
    let b = found(&mut s, P0, at(10, 10));
    found(&mut s, P1, at(12, 3));
    assert!(economy::can_produce(&s, &s.cities[&a], hall_at(&s, a)));
    // Another city already building one blocks the second.
    let a_hall = hall_at(&s, a);
    s.cities.get_mut(&a).unwrap().producing = Some(a_hall);
    assert!(!economy::can_produce(&s, &s.cities[&b], hall_at(&s, b)));
    raise_hall(&mut s, a);
    assert_eq!(s.player(P0).hall_city, Some(a));
    assert!(data::has_building(
        &s.cities[&a].buildings,
        BuildingKind::CourierHall
    ));
    assert!(!economy::can_produce(&s, &s.cities[&b], hall_at(&s, b)));
    // A rival captures the Hall's city: it stays standing there, so retake it.
    transfer_hall_city(&mut s, a, P1);
    let why = economy::production_blocker(&s, &s.cities[&b], hall_at(&s, b), true).unwrap();
    assert!(why.contains("retake"), "{why}");
    // Razed (a Celts capture removes the city): the Hall is gone and Mongols may rebuild.
    s.cities.remove(&a);
    for tile in &mut s.map.tiles {
        if tile.city == Some(a) {
            tile.city = None;
            tile.owner = None;
            tile.landmark = None;
        }
    }
    assert!(hall_city(&s, P0).is_none());
    assert!(economy::can_produce(&s, &s.cities[&b], hall_at(&s, b)));
    raise_hall(&mut s, b);
    assert_eq!(hall_city(&s, P0).map(|c| c.id), Some(b));
}

#[test]
fn the_hall_scores_for_the_owner_of_its_city_and_moves_with_it() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::India, 1)]);
    let a = found(&mut s, P0, at(3, 3));
    found(&mut s, P1, at(12, 12));
    assert_eq!(pv(&s), (0, 1));
    let ev = run(&mut s);
    assert!(goal_vp(&s, P0).is_empty() && ev.is_empty());
    raise_hall(&mut s, a);
    let ev = run(&mut s);
    assert_eq!(goal_vp(&s, P0), vec![P0]);
    assert_eq!(scoring::vp_total(&s, P0), 2);
    assert_eq!(pv(&s), (1, 1));
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted {
            player: P0,
            faction: Faction::Mongols,
            vp: 2,
            again: false
        }
    )));
    // A rival takes the city: both the points and the open road go with it.
    transfer_hall_city(&mut s, a, P1);
    let ev = run(&mut s);
    assert!(goal_vp(&s, P0).is_empty());
    assert_eq!(goal_vp(&s, P1), vec![P0]);
    assert!(!s.player(P0).faction_goal_done);
    assert_eq!(pv(&s), (0, 1));
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalLost {
            player: P0,
            faction: Faction::Mongols,
            ..
        }
    )));
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted {
            player: P1,
            faction: Faction::Mongols,
            again: true,
            ..
        }
    )));
    assert!(holds_open_road(&s, P1) && !holds_open_road(&s, P0));
    // Retaken.
    transfer_hall_city(&mut s, a, P0);
    run(&mut s);
    assert_eq!(goal_vp(&s, P0), vec![P0]);
    assert!(goal_vp(&s, P1).is_empty());
    // Razed: nobody scores until Mongols builds another.
    s.cities.remove(&a);
    for tile in &mut s.map.tiles {
        if tile.city == Some(a) {
            tile.city = None;
            tile.owner = None;
            tile.landmark = None;
        }
    }
    run(&mut s);
    assert!(goal_vp(&s, P0).is_empty() && goal_vp(&s, P1).is_empty());
    assert!(!holds_open_road(&s, P0));
}

#[test]
fn the_hall_opens_its_holders_land_to_every_unit_at_peace() {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::India, 1)]);
    let a = found(&mut s, P0, at(3, 3));
    found(&mut s, P1, at(12, 12));
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::Peace;
    }
    for kind in UnitKind::ALL {
        let military = data::unit_stats(kind).military;
        // Before the Hall, every unit is closed out of foreign land.
        assert!(
            !s.passage_allowed(P1, kind, Some(P0)),
            "{kind:?} {military}"
        );
    }
    raise_hall(&mut s, a);
    for kind in UnitKind::ALL {
        assert!(s.passage_allowed(P1, kind, Some(P0)), "{kind:?}");
        assert!(!s.passage_allowed(P0, kind, Some(P1)), "{kind:?}");
    }
    // The view agrees with the factory, and lists the Hall publicly.
    let v = crate::view::player_view(&s, P1);
    assert_eq!(v.courier_halls.len(), 1);
    assert_eq!(
        (v.courier_halls[0].holder, v.courier_halls[0].city),
        (P0, a)
    );
    for kind in UnitKind::ALL {
        assert_eq!(
            v.passage_allowed(P1, kind, Some(P0)),
            s.passage_allowed(P1, kind, Some(P0))
        );
    }
    // Captured: the road moves to the captor's land, and the old holder's closes.
    transfer_hall_city(&mut s, a, P1);
    assert!(s.passage_allowed(P0, UnitKind::Soldier, Some(P1)));
    assert!(!s.passage_allowed(P1, UnitKind::Soldier, Some(P0)));
}

//! Peace closes foreign borders to every unit kind unless a separate rule opens them.
//! War, Alliance and the Courier Hall open passage.
use legatum_core::diplomacy::RelationState;
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::Map;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

fn h(c: i32, r: i32) -> Hex {
    Hex::from_offset(c, r)
}

/// Two teams at Peace on a 12x8 plain; `THEM` owns columns 4 and 5.
fn world(mine: Faction) -> GameState {
    let mut setup = GameSetup::all_ai(5);
    setup.width = 12;
    setup.height = 8;
    setup.players.truncate(2);
    setup.players[0].faction = mine;
    setup.players[1].faction = Faction::India;
    setup.players[1].team = TeamId(1);
    let mut s = blank_game(&setup, Map::new(12, 8, Terrain::Grassland));
    let teams = s.teams();
    s.diplomacy.as_mut().unwrap().meet_all(&teams);
    for r in 0..8 {
        for c in 4..=5 {
            s.map.get_mut(h(c, r)).unwrap().owner = Some(THEM);
        }
    }
    // Keep both teams alive.
    spawn_unit(&mut s, ME, UnitKind::Settler, h(0, 7));
    spawn_unit(&mut s, THEM, UnitKind::Settler, h(11, 7));
    s
}

fn set_relation(s: &mut GameState, state: RelationState) {
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = state;
    }
}

const ALL: [UnitKind; 37] = UnitKind::ALL;

#[test]
fn mongols_units_cannot_cross_at_peace() {
    let s = world(Faction::Mongols);
    for kind in ALL {
        assert!(!s.passage_allowed(ME, kind, Some(THEM)), "{kind:?}");
        // Own land and unowned land stay open to everyone.
        assert!(s.passage_allowed(ME, kind, Some(ME)));
        assert!(s.passage_allowed(ME, kind, None));
    }
}

#[test]
fn no_faction_has_open_peace_borders_by_default() {
    for faction in Faction::ALL {
        let s = world(faction);
        for kind in ALL {
            assert!(
                !s.passage_allowed(ME, kind, Some(THEM)),
                "{faction:?} {kind:?}"
            );
        }
    }
}

#[test]
fn war_and_alliance_open_every_unit_kind() {
    for state in [RelationState::War, RelationState::Alliance] {
        let mut s = world(Faction::Mongols);
        set_relation(&mut s, state);
        for kind in ALL {
            assert!(
                s.passage_allowed(ME, kind, Some(THEM)),
                "{state:?} {kind:?}"
            );
            assert!(
                s.passage_allowed(THEM, kind, Some(ME)),
                "{state:?} {kind:?}"
            );
        }
    }
}

#[test]
fn the_player_view_agrees_with_the_engine() {
    for faction in [Faction::Mongols, Faction::Byzantium, Faction::Celts] {
        for state in [
            RelationState::Peace,
            RelationState::War,
            RelationState::Alliance,
        ] {
            let mut s = world(faction);
            set_relation(&mut s, state);
            let v = player_view(&s, ME);
            for kind in ALL {
                for owner in [None, Some(ME), Some(THEM)] {
                    assert_eq!(
                        v.passage_allowed(ME, kind, owner),
                        s.passage_allowed(ME, kind, owner),
                        "{faction:?} {state:?} {kind:?} {owner:?}"
                    );
                    assert_eq!(
                        v.entry_table(ME, kind).can_enter(&v, owner),
                        s.passage_allowed(ME, kind, owner)
                    );
                }
            }
        }
    }
}

fn move_order(unit: UnitId, to: Hex) -> BTreeMap<PlayerId, Vec<Order>> {
    BTreeMap::from([(ME, vec![Order::MoveUnit { unit, to }])])
}

#[test]
fn mongols_soldiers_scouts_and_settlers_cannot_cross_at_peace() {
    let mut s = world(Faction::Mongols);
    for (kind, row) in [
        (UnitKind::Soldier, 2),
        (UnitKind::Scout, 4),
        (UnitKind::Settler, 6),
    ] {
        let unit = spawn_unit(&mut s, ME, kind, h(3, row));
        let errors = rules::validate_orders(
            &s,
            ME,
            &[Order::MoveUnit {
                unit,
                to: h(4, row),
            }],
        );
        assert!(!errors.is_empty(), "{kind:?}");
        let (next, _) = resolve_turn(&s, &move_order(unit, h(5, row)));
        assert_ne!(
            next.map.get(next.units[&unit].pos).unwrap().owner,
            Some(THEM)
        );
    }
}

#[test]
fn a_soldier_ordered_across_the_border_by_a_stale_client_never_enters() {
    let mut s = world(Faction::Mongols);
    let soldier = spawn_unit(&mut s, ME, UnitKind::Soldier, h(3, 2));
    let (next, _) = resolve_turn(&s, &move_order(soldier, h(5, 2)));
    let at = next.units[&soldier].pos;
    assert_ne!(
        next.map.get(at).unwrap().owner,
        Some(THEM),
        "stood at {at:?}"
    );
}

#[test]
fn the_same_soldier_crosses_once_at_war() {
    let mut s = world(Faction::Mongols);
    set_relation(&mut s, RelationState::War);
    let soldier = spawn_unit(&mut s, ME, UnitKind::Soldier, h(3, 2));
    assert!(
        rules::validate_orders(
            &s,
            ME,
            &[Order::MoveUnit {
                unit: soldier,
                to: h(5, 2)
            }]
        )
        .is_empty()
    );
}

#[test]
fn mongols_keeps_its_extra_move() {
    let plain = data::unit_stats_for(Faction::Celts, UnitKind::Trooper).moves;
    let glass = data::unit_stats_for(Faction::Mongols, UnitKind::Trooper).moves;
    assert_eq!(glass, plain + 1);
}

/// `world(Sparta)` with THEM turned into a Mongols empire that owns a city at (11, 1).
/// Its two border columns (4 and 5) stay owned by it.
fn hall_world() -> (GameState, CityId) {
    let mut s = world(Faction::Sparta);
    s.players[1].faction = Faction::Mongols;
    let settler = spawn_unit(&mut s, THEM, UnitKind::Settler, h(11, 1));
    let city = rules::cities::found_city(&mut s, settler, &mut vec![]).unwrap();
    for r in 0..8 {
        for c in 4..=5 {
            s.map.get_mut(h(c, r)).unwrap().owner = Some(THEM);
        }
    }
    (s, city)
}

#[test]
fn the_courier_hall_opens_its_holders_land_to_a_soldier_at_peace_and_closes_when_it_falls() {
    let (mut s, city) = hall_world();
    let soldier = spawn_unit(&mut s, ME, UnitKind::Soldier, h(3, 2));
    let go = |s: &GameState| {
        rules::validate_orders(
            s,
            ME,
            &[Order::MoveUnit {
                unit: soldier,
                to: h(5, 2),
            }],
        )
    };
    assert!(!go(&s).is_empty(), "closed before the Hall exists");
    s.cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::CourierHall);
    s.player_mut(THEM).hall_city = Some(city);
    assert!(go(&s).is_empty(), "{:?}", go(&s));
    // Resolution agrees: the army walks into the holder's land, then stands there at Peace.
    let (next, _) = resolve_turn(&s, &move_order(soldier, h(5, 2)));
    assert_eq!(
        next.map.get(next.units[&soldier].pos).unwrap().owner,
        Some(THEM)
    );
    // The public view agrees with the engine for every kind.
    let v = player_view(&s, ME);
    assert_eq!(v.courier_halls.len(), 1);
    for kind in ALL {
        assert_eq!(
            v.entry_table(ME, kind).can_enter(&v, Some(THEM)),
            s.passage_allowed(ME, kind, Some(THEM)),
            "{kind:?}"
        );
    }
    // The city falls (razed): the road is gone and the soldier, stranded inside, is closed out again.
    s.cities.remove(&city);
    assert!(!s.passage_allowed(ME, UnitKind::Soldier, Some(THEM)));
    assert!(!go(&s).is_empty());
}

use super::*;
use crate::orders::Order;
use crate::rules::test_support::*;
use crate::rules::{hostile, resolve_turn};
use crate::state::GameSetup;

/// Two empires at Peace on the 12x10 grassland test map, with barbarians on.
fn peaceful() -> GameState {
    let mut s = game(&[0, 1]);
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::Peace;
    }
    s.setup.barbarians = true;
    s.barbarians = Some(BarbarianState::default());
    s.players[0].gold = 500;
    s.players[1].gold = 500;
    // Home cities in far corners keep both empires alive (no elimination).
    let a = city(&mut s, 0, hex(0, 9), 1);
    let b = city(&mut s, 1, hex(11, 9), 1);
    for c in [a, b] {
        s.cities.get_mut(&c).unwrap().is_capital = true;
    }
    s
}

/// A camp with its guard and scout, placed directly.
fn add_camp(s: &mut GameState, pos: Hex) -> u32 {
    let mut barb = s.barbarians.take().unwrap();
    let id = barb.next_camp_id;
    barb.next_camp_id += 1;
    barb.last_camp_turn = s.turn;
    let guard = spawn(s, UnitKind::Soldier, pos);
    let scout = place_near(s, pos, true).map(|h| spawn(s, UnitKind::Scout, h));
    barb.camps.push(Camp {
        id,
        pos,
        founded_turn: s.turn,
        guard: Some(guard),
        scout,
        raiders: Vec::new(),
        scout_report: None,
        raid: None,
        next_raid_turn: 0,
        next_spawn_turn: s.turn + 100,
        rite: None,
    });
    s.barbarians = Some(barb);
    id
}

fn camp(s: &GameState, id: u32) -> Option<&Camp> {
    s.barbarians
        .as_ref()
        .unwrap()
        .camps
        .iter()
        .find(|c| c.id == id)
}

fn barbarian_units(s: &GameState) -> Vec<&crate::state::Unit> {
    s.units
        .values()
        .filter(|u| u.owner.is_barbarian())
        .collect()
}

#[test]
fn barbarian_ranger_skips_a_visible_target_behind_a_mountain() {
    let mut s = peaceful();
    let ranger = spawn(&mut s, UnitKind::Ranger, hex(4, 4));
    unit(&mut s, 0, UnitKind::Soldier, hex(6, 4));
    let mut sight = BitSet::new(s.map.tiles.len());
    sight.set(s.map.index(hex(6, 4)).unwrap());
    s.map.get_mut(hex(5, 4)).unwrap().terrain = Terrain::Mountains;
    assert_eq!(
        raider_plan(&s, &sight, ranger, None, None, &mut BTreeSet::new()).1,
        None
    );
    s.map.get_mut(hex(5, 4)).unwrap().terrain = Terrain::Grassland;
    assert_eq!(
        raider_plan(&s, &sight, ranger, None, None, &mut BTreeSet::new()).1,
        Some(hex(6, 4))
    );
}

fn run(s: &GameState, orders: &BTreeMap<PlayerId, Vec<Order>>) -> (GameState, Vec<Event>) {
    resolve_turn(s, orders)
}

#[test]
fn barbarians_are_hostile_to_every_team_even_at_peace_and_never_an_empire() {
    let mut s = peaceful();
    add_camp(&mut s, hex(9, 7));
    assert!(
        !hostile(&s, PlayerId(0), PlayerId(1)),
        "empires start at Peace"
    );
    for p in [PlayerId(0), PlayerId(1)] {
        assert!(hostile(&s, p, PlayerId::BARBARIAN));
        assert!(hostile(&s, PlayerId::BARBARIAN, p));
        assert!(s.passage_allowed(PlayerId::BARBARIAN, crate::UnitKind::Soldier, Some(p)));
    }
    assert!(!hostile(&s, PlayerId::BARBARIAN, PlayerId::BARBARIAN));
    assert_eq!(s.team_of(PlayerId::BARBARIAN), TeamId::BARBARIAN);
    assert_eq!(s.players.len(), 2);
    assert!(!s.teams().contains(&TeamId::BARBARIAN));
    let view = crate::player_view(&s, PlayerId(0));
    assert!(view.players.iter().all(|p| !p.id.is_barbarian()));
    assert!(view.standings.iter().all(|t| t.team != TeamId::BARBARIAN));
    assert_eq!(view.team_of(PlayerId::BARBARIAN), TeamId::BARBARIAN);
    assert!(view.at_war(PlayerId(0), PlayerId::BARBARIAN));
    assert!(!view.at_war(PlayerId(0), PlayerId(1)));
}

#[test]
fn barbarians_off_never_spawn_and_old_setups_load_off() {
    let mut setup = GameSetup::all_ai(5);
    assert!(setup.barbarians, "new games default to barbarians on");
    setup.barbarians = false;
    let mut s = crate::new_game(&setup);
    assert!(s.barbarians.is_none());
    let json = serde_json::to_string(&s.setup).unwrap();
    assert!(
        !json.contains("barbarians"),
        "an Off setup serializes as before"
    );
    for _ in 0..30 {
        s = run(&s, &no_orders()).0;
    }
    assert!(barbarian_units(&s).is_empty());
    let mut v: serde_json::Value = serde_json::to_value(&setup).unwrap();
    v.as_object_mut().unwrap().remove("barbarians");
    let old: GameSetup = serde_json::from_value(v).unwrap();
    assert!(
        !old.barbarians,
        "setups from before barbarians load with them off"
    );
}

#[test]
fn first_camps_appear_at_the_grace_turn_in_fog_away_from_cities_and_units() {
    let setup = GameSetup::all_ai(9);
    let mut s = crate::new_game(&setup);
    let first = FIRST_CAMP_TURN;
    let mut spawned_turn = None;
    while s.turn <= first {
        let before = s.clone();
        let (next, events) = run(&s, &no_orders());
        let spawned: Vec<Hex> = events
            .iter()
            .filter_map(|e| match e {
                Event::BarbarianCampSpawned { at, .. } => Some(*at),
                _ => None,
            })
            .collect();
        if !spawned.is_empty() {
            spawned_turn = Some(before.turn);
            assert_eq!(spawned.len(), camp_target(&next).div_ceil(3));
            for at in spawned {
                let i = next.map.index(at).unwrap();
                for team in next.teams() {
                    assert!(!visibility::native_visible_tiles(&next, team).get(i));
                }
                assert!(next.map.get(at).unwrap().owner.is_none());
                for c in next.cities.values() {
                    assert!(c.pos.distance(at) >= CAMP_MIN_CITY_DISTANCE);
                }
                for u in next.units.values().filter(|u| !u.owner.is_barbarian()) {
                    assert!(u.pos.distance(at) >= CAMP_MIN_UNIT_DISTANCE);
                }
                // Guard on the camp, scout beside it.
                let here: Vec<_> = next.units_at(at).collect();
                assert_eq!(here.len(), 1);
                assert!(here[0].owner.is_barbarian() && here[0].kind == UnitKind::Soldier);
            }
        }
        s = next;
    }
    assert_eq!(spawned_turn, Some(first));
}

#[test]
fn rite_camps_stand_outside_the_natural_camp_target() {
    let setup = GameSetup::all_ai(9);
    let mut s = crate::new_game(&setup);
    let first = FIRST_CAMP_TURN;
    let target = camp_target(&s);
    // As many rite camps as the target allows: natural camps still appear at the grace turn.
    let mut barb = s.barbarians.take().unwrap();
    for i in 0..target as u32 {
        barb.camps.push(Camp {
            id: 100 + i,
            pos: hex(0, i as i32),
            founded_turn: 1,
            guard: None,
            scout: None,
            raiders: Vec::new(),
            scout_report: None,
            raid: None,
            next_raid_turn: 0,
            next_spawn_turn: 999,
            rite: Some(PlayerId(0)),
        });
    }
    barb.next_camp_id = 200;
    s.barbarians = Some(barb);
    assert_eq!(s.barbarians.as_ref().unwrap().natural_camps(), 0);
    while s.turn <= first {
        s = run(&s, &no_orders()).0;
    }
    assert_eq!(
        s.barbarians.as_ref().unwrap().natural_camps(),
        target.div_ceil(3)
    );
}

#[test]
fn a_scout_that_sees_a_city_and_gets_home_launches_a_raid() {
    let mut s = peaceful();
    let city_id = city(&mut s, 0, hex(2, 2), 2);
    let camp_id = add_camp(&mut s, hex(8, 2));
    let scout = camp(&s, camp_id).unwrap().scout.unwrap();
    s.units.get_mut(&scout).unwrap().pos = hex(4, 2);
    let (s, events) = run(&s, &no_orders());
    assert!(events.iter().any(|e| matches!(e,
        Event::BarbarianScoutSpotted { city, owner: PlayerId(0), .. } if *city == city_id)));
    assert_eq!(camp(&s, camp_id).unwrap().scout_report, Some(city_id));
    // The scout walks home; the turn after it arrives the raid is launched.
    let mut s = s;
    let mut launched = None;
    for _ in 0..6 {
        let (next, events) = run(&s, &no_orders());
        if let Some(e) = events
            .iter()
            .find(|e| matches!(e, Event::BarbarianRaidLaunched { .. }))
        {
            launched = Some(e.clone());
        }
        s = next;
        if launched.is_some() {
            break;
        }
    }
    let Some(Event::BarbarianRaidLaunched {
        city,
        owner,
        direction,
        ..
    }) = launched
    else {
        panic!("raid launched")
    };
    assert_eq!((city, owner), (city_id, PlayerId(0)));
    assert_eq!(direction, "east");
    // Raiders muster one per turn in the fixed Standard pace: melee, ranged, melee.
    let mut kinds = Vec::new();
    for _ in 0..5 {
        s = run(&s, &no_orders()).0;
    }
    for id in &camp(&s, camp_id).unwrap().raiders {
        kinds.push(s.units[id].kind);
    }
    kinds.sort();
    assert_eq!(
        kinds,
        [UnitKind::Soldier, UnitKind::Soldier, UnitKind::Ranger]
    );
    // The raid is visible to the city's owner only.
    let before = crate::player_view(&s, PlayerId(0));
    let raid = Event::BarbarianRaidLaunched {
        camp: camp_id,
        city: city_id,
        owner: PlayerId(0),
        direction: "east".into(),
    };
    let seen = |p: u8| {
        let v = crate::player_view(&s, PlayerId(p));
        !crate::event_visibility::visible_events(std::slice::from_ref(&raid), &v, &v).is_empty()
    };
    assert!(seen(0) && !seen(1));
    drop(before);
}

#[test]
fn killing_the_scout_before_it_gets_home_prevents_the_raid() {
    let mut s = peaceful();
    city(&mut s, 0, hex(2, 2), 2);
    let camp_id = add_camp(&mut s, hex(10, 2));
    let scout = camp(&s, camp_id).unwrap().scout.unwrap();
    s.units.get_mut(&scout).unwrap().pos = hex(4, 2);
    let (mut s, _) = run(&s, &no_orders());
    assert!(camp(&s, camp_id).unwrap().scout_report.is_some());
    let at = s.units[&scout].pos;
    let hunter = unit(&mut s, 0, UnitKind::Soldier, at.neighbors()[0]);
    s.units.get_mut(&scout).unwrap().hp = 1;
    let (mut s, _) = run(
        &s,
        &orders(
            0,
            vec![Order::MoveUnit {
                unit: hunter,
                to: at,
            }],
        ),
    );
    assert!(!s.units.contains_key(&scout), "the scout died");
    for _ in 0..6 {
        let (next, events) = run(&s, &no_orders());
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::BarbarianRaidLaunched { .. }))
        );
        s = next;
    }
    assert!(camp(&s, camp_id).unwrap().raid.is_none());
}

#[test]
fn entering_an_undefended_camp_clears_it_for_standard_gold() {
    let mut s = peaceful();
    let camp_id = add_camp(&mut s, hex(6, 5));
    let guard = camp(&s, camp_id).unwrap().guard.unwrap();
    let scout = camp(&s, camp_id).unwrap().scout.unwrap();
    s.units.remove(&guard);
    s.units.remove(&scout);
    let soldier = unit(&mut s, 1, UnitKind::Soldier, hex(4, 5));
    let gold = s.players[1].gold;
    let (next, events) = run(
        &s,
        &orders(
            1,
            vec![Order::MoveUnit {
                unit: soldier,
                to: hex(6, 5),
            }],
        ),
    );
    assert!(camp(&next, camp_id).is_none());
    let reward = CAMP_CLEAR_GOLD;
    assert_eq!(reward, 50);
    assert!(events.contains(&Event::BarbarianCampCleared {
        camp: camp_id,
        at: hex(6, 5),
        by: PlayerId(1),
        unit: Some(soldier),
        gold: reward,
    }));
    let paid: i32 = events
        .iter()
        .filter_map(|e| match e {
            Event::BarbarianCampCleared { gold, .. } => Some(*gold),
            _ => None,
        })
        .sum();
    assert_eq!(paid, reward);
    assert!(next.players[1].gold >= gold, "the reward is paid");
    // The clearing team sees the report; the other team cannot see the tile.
    let view = crate::player_view(&next, PlayerId(1));
    assert!(
        view.barbarian_camps.is_empty(),
        "a cleared camp is forgotten"
    );
}

#[test]
fn a_guarded_camp_must_be_fought_for() {
    let mut s = peaceful();
    let camp_id = add_camp(&mut s, hex(6, 5));
    let soldier = unit(&mut s, 1, UnitKind::Soldier, hex(5, 5));
    let (next, events) = run(
        &s,
        &orders(
            1,
            vec![Order::MoveUnit {
                unit: soldier,
                to: hex(6, 5),
            }],
        ),
    );
    assert!(events.iter().any(|e| matches!(e, Event::Battle { .. })));
    assert!(
        camp(&next, camp_id).is_some(),
        "the guard survived one attack"
    );
    assert_ne!(next.units[&soldier].pos, hex(6, 5));
}

#[test]
fn borders_growing_over_a_camp_disperse_it_without_gold() {
    let mut s = peaceful();
    let camp_id = add_camp(&mut s, hex(6, 5));
    s.map.get_mut(hex(6, 5)).unwrap().owner = Some(PlayerId(0));
    let gold = s.players[0].gold;
    let (next, events) = run(&s, &no_orders());
    assert!(camp(&next, camp_id).is_none());
    assert!(events.iter().any(|e| matches!(
        e,
        Event::BarbarianCampCleared {
            by: PlayerId(0),
            unit: None,
            gold: 0,
            ..
        }
    )));
    assert!(next.players[0].gold <= gold + 20, "no clearing reward");
}

#[test]
fn raiders_prowl_but_never_assault_or_capture_a_city() {
    let mut s = peaceful();
    let c = city(&mut s, 0, hex(5, 5), 1);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let camp_id = add_camp(&mut s, hex(10, 8));
    let raider = spawn(&mut s, UnitKind::Soldier, hex(6, 5));
    let mut barb = s.barbarians.take().unwrap();
    barb.camps[0].raiders.push(raider);
    barb.camps[0].raid = Some(Raid {
        city: c,
        to_spawn: 0,
        next_spawn_turn: 0,
        until_turn: 100,
    });
    s.barbarians = Some(barb);
    // Force the step into the city, as a player could order.
    let mut t = s.clone();
    barbarians_goto(&mut t, raider, hex(5, 5));
    let mut events = Vec::new();
    crate::rules::movement::resolve_movement(&mut t, &mut events);
    assert_eq!(t.cities[&c].owner, PlayerId(0));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::CityCaptured { .. }))
    );
    assert_ne!(t.units[&raider].pos, hex(5, 5));
    // Their own plan prowls near the city instead of assaulting it.
    let (next, events) = run(&s, &no_orders());
    assert_eq!(next.cities[&c].owner, PlayerId(0));
    assert_eq!(next.units[&raider].pos.distance(hex(5, 5)), RAID_PROWL);
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::Battle {
            defender: crate::events::Defender::City { .. },
            ..
        }
    )));
    assert!(camp(&next, camp_id).is_some());
}

#[test]
fn a_raider_does_not_choose_its_own_guard_as_a_prowl_destination() {
    let mut s = peaceful();
    let target = city(&mut s, 0, hex(5, 5), 1);
    let camp_id = add_camp(&mut s, hex(7, 5));
    let guard = camp(&s, camp_id).unwrap().guard.unwrap();
    let raider = spawn(&mut s, UnitKind::Soldier, hex(8, 5));
    let c = &mut s.barbarians.as_mut().unwrap().camps[0];
    c.raiders.push(raider);
    c.raid = Some(Raid {
        city: target,
        to_spawn: 0,
        next_spawn_turn: 0,
        until_turn: 100,
    });
    let sight = BitSet::new(s.map.tiles.len());
    let (destination, shot) = raider_plan(
        &s,
        &sight,
        raider,
        Some(hex(7, 5)),
        Some(target),
        &mut BTreeSet::new(),
    );
    assert_eq!(shot, None);
    assert!(destination.is_some());
    assert_ne!(destination, Some(s.units[&guard].pos));
    let (next, events) = run(&s, &no_orders());
    assert_ne!(next.units[&raider].pos, hex(8, 5));
    assert!(!events.iter().any(
        |event| matches!(event, Event::MoveBlocked { unit, reason, .. }
        if *unit == raider && reason.contains("blocked by Barbarians"))
    ));
}

fn barbarians_goto(s: &mut GameState, id: UnitId, to: Hex) {
    s.units.get_mut(&id).unwrap().goto = Some(to);
}

#[test]
fn raiders_capture_lone_settlers_and_clearing_the_camp_frees_them() {
    let mut s = peaceful();
    let camp_id = add_camp(&mut s, hex(8, 5));
    let raider = spawn(&mut s, UnitKind::Soldier, hex(5, 5));
    s.barbarians.as_mut().unwrap().camps[0].raiders.push(raider);
    let settler = unit(&mut s, 0, UnitKind::Settler, hex(4, 5));
    let (mut s, events) = run(&s, &no_orders());
    assert!(events.iter().any(|e| matches!(
        e,
        Event::SettlerCaptured {
            from: PlayerId(0),
            to: PlayerId::BARBARIAN,
            ..
        }
    )));
    assert!(s.units[&settler].owner.is_barbarian());
    // The captive walks to the camp.
    for _ in 0..6 {
        s = run(&s, &no_orders()).0;
    }
    assert_eq!(s.units[&settler].pos, hex(8, 5));
    // Clear the camp: the guard is gone and a Soldier walks in, freeing the captive.
    if let Some(g) = camp(&s, camp_id).unwrap().guard {
        s.units.remove(&g);
    }
    for id in camp(&s, camp_id).unwrap().raiders.clone() {
        s.units.remove(&id);
    }
    let rescuer = unit(&mut s, 1, UnitKind::Soldier, hex(7, 5));
    let (next, events) = run(
        &s,
        &orders(
            1,
            vec![Order::MoveUnit {
                unit: rescuer,
                to: hex(8, 5),
            }],
        ),
    );
    assert!(camp(&next, camp_id).is_none());
    assert_eq!(next.units[&settler].owner, PlayerId(1));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::SettlerCaptured {
            from: PlayerId::BARBARIAN,
            to: PlayerId(1),
            ..
        }
    )));
}

#[test]
fn camps_show_only_where_seen_and_are_remembered_until_seen_gone() {
    let mut s = peaceful();
    let camp_id = add_camp(&mut s, hex(9, 5));
    crate::visibility::update_explored(&mut s);
    assert!(
        crate::player_view(&s, PlayerId(0))
            .barbarian_camps
            .is_empty()
    );
    let scout = unit(&mut s, 0, UnitKind::Scout, hex(7, 5));
    let before = s.clone();
    crate::visibility::update_explored(&mut s);
    let view = crate::player_view(&s, PlayerId(0));
    assert_eq!(
        view.barbarian_camps,
        [crate::view::BarbarianCampView {
            pos: hex(9, 5),
            visible: true
        }]
    );
    assert!(
        crate::player_view(&s, PlayerId(1))
            .barbarian_camps
            .is_empty()
    );
    let mut events = Vec::new();
    end_of_turn(&before, &mut s, &mut events);
    assert!(events.contains(&Event::BarbarianCampDiscovered {
        camp: camp_id,
        team: TeamId(0),
        at: hex(9, 5),
    }));
    // Walk away: remembered, not visible.
    s.units.get_mut(&scout).unwrap().pos = hex(1, 1);
    crate::visibility::update_explored(&mut s);
    let view = crate::player_view(&s, PlayerId(0));
    assert_eq!(view.barbarian_camps.len(), 1);
    assert!(!view.barbarian_camps[0].visible);
    // Gone while unseen: still remembered. Seen gone: forgotten.
    s.barbarians.as_mut().unwrap().camps.clear();
    crate::visibility::update_explored(&mut s);
    assert_eq!(crate::player_view(&s, PlayerId(0)).barbarian_camps.len(), 1);
    s.units.get_mut(&scout).unwrap().pos = hex(7, 5);
    crate::visibility::update_explored(&mut s);
    assert!(
        crate::player_view(&s, PlayerId(0))
            .barbarian_camps
            .is_empty()
    );
}

#[test]
fn barbarians_never_heal_or_explore_ruins() {
    let mut s = peaceful();
    let id = add_camp(&mut s, hex(6, 5));
    let guard = camp(&s, id).unwrap().guard.unwrap();
    s.units.get_mut(&guard).unwrap().hp = 40;
    s.map.get_mut(hex(3, 3)).unwrap().ruin = true;
    let roamer = spawn(&mut s, UnitKind::Soldier, hex(3, 4));
    barbarians_goto(&mut s, roamer, hex(3, 3));
    let mut events = Vec::new();
    crate::rules::movement::resolve_movement(&mut s, &mut events);
    assert!(
        s.map.get(hex(3, 3)).unwrap().ruin,
        "barbarians leave ruins alone"
    );
    crate::rules::units::heal_and_fortify(&mut s);
    assert_eq!(s.units[&guard].hp, 40);
}

#[test]
fn units_by_era_follow_half_the_living_empires() {
    let mut s = peaceful();
    assert_eq!(unit_kinds(&s), (UnitKind::Soldier, UnitKind::Ranger));
    s.players[0].techs.insert(TechId::Feudalism);
    assert_eq!(
        unit_kinds(&s).0,
        UnitKind::Trooper,
        "one of two empires is half"
    );
    s.players[1].alive = false;
    s.players[0].techs.insert(TechId::Machinery);
    assert_eq!(unit_kinds(&s), (UnitKind::Trooper, UnitKind::Marksman));
}

#[test]
fn xp_from_barbarians_stops_at_the_first_promotion() {
    assert_eq!(cap_barbarian_xp(0, 5, 30), 5);
    assert_eq!(cap_barbarian_xp(28, 5, 30), 2);
    assert_eq!(cap_barbarian_xp(30, 5, 30), 0);
    assert_eq!(cap_barbarian_xp(45, 5, 30), 0);
}

#[test]
fn raid_directions_name_the_compass_point_toward_the_camp() {
    let c = hex(10, 10);
    assert_eq!(direction(c, hex(14, 10)), "east");
    assert_eq!(direction(c, hex(6, 10)), "west");
    assert_eq!(direction(c, hex(10, 4)), "north");
    assert_eq!(direction(c, hex(10, 16)), "south");
    assert_eq!(direction(c, hex(14, 6)), "north-east");
    assert_eq!(direction(c, hex(6, 14)), "south-west");
}

#[test]
fn full_games_with_barbarians_are_deterministic_and_keep_invariants() {
    let setup = GameSetup::all_ai(21);
    let play = || {
        let mut s = crate::new_game(&setup);
        let mut hashes = Vec::new();
        let mut spawned = 0;
        for _ in 0..45 {
            let (next, events) = run(&s, &no_orders());
            spawned += events
                .iter()
                .filter(|e| matches!(e, Event::BarbarianCampSpawned { .. }))
                .count();
            let mut seen = BTreeSet::new();
            for u in next.units.values() {
                assert!(
                    seen.insert((u.pos, is_military(u.kind))),
                    "stacked {:?}",
                    u.pos
                );
                for v in next.units_at(u.pos) {
                    assert!(
                        !hostile(&next, u.owner, v.owner),
                        "enemies share {:?}",
                        u.pos
                    );
                }
            }
            assert!(next.cities.values().all(|c| !c.owner.is_barbarian()));
            hashes.push(next.hash());
            s = next;
        }
        (hashes, spawned, s)
    };
    let (a, spawned, end) = play();
    let (b, _, _) = play();
    assert_eq!(a, b);
    assert!(spawned > 0);
    assert!(!barbarian_units(&end).is_empty());
    let json = serde_json::to_string(&end).unwrap();
    let back: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(back, end);
}

#[test]
fn fighting_barbarians_earns_xp_only_up_to_the_first_promotion() {
    let first = crate::rules::promotion::threshold(BARBARIAN_XP_MAX_LEVEL).unwrap();
    for (xp_before, expect_gain) in [(0, true), (first, false)] {
        let mut s = peaceful();
        add_camp(&mut s, hex(10, 2));
        let soldier = unit(&mut s, 0, UnitKind::Soldier, hex(4, 5));
        s.units.get_mut(&soldier).unwrap().xp = xp_before;
        let raider = spawn(&mut s, UnitKind::Soldier, hex(5, 5));
        s.units.get_mut(&raider).unwrap().hp = 30;
        let (next, events) = run(
            &s,
            &orders(
                0,
                vec![Order::MoveUnit {
                    unit: soldier,
                    to: hex(5, 5),
                }],
            ),
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Battle { attacker, .. } if *attacker == soldier))
        );
        let after = next.units[&soldier].xp;
        if expect_gain {
            assert!(
                after > 0 && after <= first,
                "capped at the first promotion: {after}"
            );
        } else {
            assert_eq!(
                after, xp_before,
                "no XP from barbarians after the first promotion"
            );
        }
        assert!(
            next.units
                .values()
                .filter(|u| u.owner.is_barbarian())
                .all(|u| u.xp == 0)
        );
    }
}

/// `peaceful()` with player 0 as the Celts.
fn celtic() -> GameState {
    let mut s = peaceful();
    s.players[0].faction = Faction::Celts;
    s
}

#[test]
fn raiders_leave_a_celtic_settler_alone_but_the_rule_is_the_celtic_empire_only() {
    for (faction, captured) in [(Faction::Celts, false), (Faction::Switzerland, true)] {
        let mut s = peaceful();
        s.players[0].faction = faction;
        add_camp(&mut s, hex(8, 5));
        let raider = spawn(&mut s, UnitKind::Soldier, hex(5, 5));
        s.barbarians.as_mut().unwrap().camps[0].raiders.push(raider);
        let settler = unit(&mut s, 0, UnitKind::Settler, hex(4, 5));
        let (next, events) = run(&s, &no_orders());
        assert_eq!(
            next.units[&settler].owner.is_barbarian(),
            captured,
            "{faction:?}"
        );
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e, Event::SettlerCaptured { .. })),
            captured
        );
    }
}

#[test]
fn barbarian_units_never_step_onto_or_shoot_celtic_units() {
    let mut s = celtic();
    add_camp(&mut s, hex(9, 8));
    let raider = spawn(&mut s, UnitKind::Soldier, hex(5, 5));
    let ranger = spawn(&mut s, UnitKind::Ranger, hex(5, 3));
    let soldier = unit(&mut s, 0, UnitKind::Soldier, hex(6, 5));
    let archer = unit(&mut s, 0, UnitKind::Ranger, hex(5, 2));
    let mut sight = BitSet::new(s.map.tiles.len());
    for h in [hex(6, 5), hex(5, 2)] {
        sight.set(s.map.index(h).unwrap());
    }
    let (goto, shot) = raider_plan(&s, &sight, raider, None, None, &mut BTreeSet::new());
    assert_eq!(shot, None);
    assert!(
        goto.is_none_or(|g| g != hex(6, 5)),
        "no attack on the soldier"
    );
    assert_eq!(
        raider_plan(&s, &sight, ranger, None, None, &mut BTreeSet::new()).1,
        None,
        "no shot at a Celtic unit in reach"
    );
    // A forced step into the Celtic unit's tile is blocked, with no fight.
    let mut t = s.clone();
    barbarians_goto(&mut t, raider, hex(6, 5));
    let mut events = Vec::new();
    crate::rules::movement::resolve_movement(&mut t, &mut events);
    assert_eq!(t.units[&raider].pos, hex(5, 5));
    assert!(t.units.contains_key(&soldier) && t.units.contains_key(&archer));
    assert!(!events.iter().any(|e| matches!(e, Event::Battle { .. })));
    // A Celtic unit stepping onto the tile a barbarian also steps onto is no meeting either:
    // the barbarian stays put.
    let mut t = s.clone();
    barbarians_goto(&mut t, raider, hex(6, 4));
    t.units.get_mut(&soldier).unwrap().goto = Some(hex(6, 4));
    let mut events = Vec::new();
    crate::rules::movement::resolve_movement(&mut t, &mut events);
    assert!(!events.iter().any(|e| matches!(e, Event::Battle { .. })));
    assert_eq!(t.units[&raider].pos, hex(5, 5));
    assert_eq!(t.units[&soldier].pos, hex(6, 4));
}

#[test]
fn scouts_report_no_celtic_city_and_raids_never_target_one() {
    let mut s = celtic();
    city(&mut s, 0, hex(2, 2), 2);
    let camp_id = add_camp(&mut s, hex(8, 2));
    let scout = camp(&s, camp_id).unwrap().scout.unwrap();
    s.units.get_mut(&scout).unwrap().pos = hex(4, 2);
    let (mut s, events) = run(&s, &no_orders());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::BarbarianScoutSpotted { .. }))
    );
    for _ in 0..10 {
        let (next, events) = run(&s, &no_orders());
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::BarbarianRaidLaunched { .. }))
        );
        s = next;
    }
    assert!(camp(&s, camp_id).unwrap().raid.is_none());
}

#[test]
fn the_celts_still_fight_camps_and_collect_the_gold() {
    let mut s = celtic();
    let camp_id = add_camp(&mut s, hex(6, 5));
    let guard = camp(&s, camp_id).unwrap().guard.unwrap();
    let soldier = unit(&mut s, 0, UnitKind::Soldier, hex(5, 5));
    let go = orders(
        0,
        vec![Order::MoveUnit {
            unit: soldier,
            to: hex(6, 5),
        }],
    );
    // The Celts attack the guard (the rule is one-sided).
    let (_, events) = run(&s, &go);
    assert!(events.iter().any(|e| matches!(e, Event::Battle { .. })));
    // An undefended natural camp pays the usual Gold to the Celts.
    s.units.remove(&guard);
    if let Some(sc) = camp(&s, camp_id).unwrap().scout {
        s.units.remove(&sc);
    }
    let (next, events) = run(&s, &go);
    assert!(camp(&next, camp_id).is_none());
    assert!(events.contains(&Event::BarbarianCampCleared {
        camp: camp_id,
        at: hex(6, 5),
        by: PlayerId(0),
        unit: Some(soldier),
        gold: CAMP_CLEAR_GOLD,
    }));
}

/// `peaceful()` with player 0 as `faction` and a walled city of theirs at (5, 5).
fn walled(faction: Faction) -> (GameState, crate::CityId) {
    let mut s = peaceful();
    s.players[0].faction = faction;
    let c = city(&mut s, 0, hex(5, 5), 2);
    s.cities.get_mut(&c).unwrap().buildings = [crate::BuildingKind::Walls].into_iter().collect();
    (s, c)
}

fn city_fired(events: &[Event]) -> bool {
    events.iter().any(|e| {
        matches!(
            e,
            Event::Battle {
                attacker: crate::events::Attacker::City { .. },
                ..
            }
        )
    })
}

#[test]
fn barbarians_are_hostile_to_the_celts_but_no_threat_to_them() {
    let barb = PlayerId::BARBARIAN;
    for (faction, threat) in [(Faction::Celts, false), (Faction::Switzerland, true)] {
        let mut s = peaceful();
        s.players[0].faction = faction;
        assert!(
            hostile(&s, PlayerId(0), barb),
            "{faction:?}: attacks stay legal"
        );
        assert_eq!(
            crate::rules::is_threat(&s, PlayerId(0), barb),
            threat,
            "{faction:?}"
        );
        let view = crate::view::player_view(&s, PlayerId(0));
        assert_eq!(
            view.is_threat(PlayerId(0), barb),
            threat,
            "{faction:?}: view agrees"
        );
        assert!(view.at_war(PlayerId(0), barb));
    }
    // The rule is about barbarians only: a rival at War is a threat to the Celts, and the
    // barbarians' own view of the Celts is unchanged.
    let mut s = celtic();
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::War;
    }
    assert!(crate::rules::is_threat(&s, PlayerId(0), PlayerId(1)));
    assert!(crate::rules::is_threat(&s, barb, PlayerId(0)));
}

#[test]
fn a_celtic_walled_city_does_not_fire_at_barbarians_but_other_cities_do() {
    for (faction, fires) in [(Faction::Celts, false), (Faction::Switzerland, true)] {
        let (mut s, c) = walled(faction);
        let guard = spawn(&mut s, UnitKind::Soldier, hex(6, 5));
        let (_, events) = run(&s, &no_orders());
        assert_eq!(city_fired(&events), fires, "{faction:?}");
        // The shot's forecast (what the UI and AI read) agrees with resolution.
        let view = crate::view::player_view(&s, PlayerId(0));
        let cv = view.cities.iter().find(|v| v.id == c).unwrap();
        assert_eq!(
            crate::forecast::city_strike_target(&view, cv),
            fires.then_some(guard),
            "{faction:?}"
        );
    }
}

#[test]
fn a_celtic_walled_city_still_fires_at_a_rival_beside_a_barbarian() {
    let (mut s, _) = walled(Faction::Celts);
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::War;
    }
    spawn(&mut s, UnitKind::Soldier, hex(6, 5));
    let rival = unit(&mut s, 1, UnitKind::Soldier, hex(4, 5));
    let (next, events) = run(&s, &no_orders());
    assert!(city_fired(&events));
    assert!(next.units.get(&rival).is_none_or(|u| u.hp < 100));
}

#[test]
fn barbarians_do_not_wake_a_celtic_sleeper_but_do_wake_everyone_else() {
    for (faction, wakes) in [(Faction::Celts, false), (Faction::Switzerland, true)] {
        let mut s = peaceful();
        s.players[0].faction = faction;
        let sleeper = unit(&mut s, 0, UnitKind::Soldier, hex(5, 5));
        s.units.get_mut(&sleeper).unwrap().standing_order = Some(crate::StandingOrder::Sleep);
        let before = crate::rules::units::sleeping_threats(&s);
        assert!(before[&sleeper].is_empty());
        spawn(&mut s, UnitKind::Soldier, hex(7, 5));
        let mut now = crate::rules::units::sleeping_threats(&s);
        assert_eq!(!now[&sleeper].is_empty(), wakes, "{faction:?}");
        let mut events = Vec::new();
        let mut previous = before;
        crate::rules::units::wake_threatened_sleepers(&mut s, &mut previous, &mut events);
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e, Event::UnitAwakened { unit, .. } if *unit == sleeper)),
            wakes,
            "{faction:?}"
        );
        now.clear();
    }
}

#[test]
fn a_celtic_sleeper_still_wakes_for_a_rival_at_war() {
    let mut s = celtic();
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::War;
    }
    let sleeper = unit(&mut s, 0, UnitKind::Soldier, hex(5, 5));
    s.units.get_mut(&sleeper).unwrap().standing_order = Some(crate::StandingOrder::Sleep);
    let mut previous = crate::rules::units::sleeping_threats(&s);
    unit(&mut s, 1, UnitKind::Soldier, hex(7, 5));
    let mut events = Vec::new();
    crate::rules::units::wake_threatened_sleepers(&mut s, &mut previous, &mut events);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::UnitAwakened { unit, .. } if *unit == sleeper))
    );
}

#[test]
fn a_manual_celtic_ranged_strike_on_a_barbarian_still_resolves() {
    let mut s = celtic();
    let archer = unit(&mut s, 0, UnitKind::Ranger, hex(5, 5));
    let guard = spawn(&mut s, UnitKind::Soldier, hex(6, 5));
    let (next, events) = run(
        &s,
        &orders(
            0,
            vec![Order::RangedAttack {
                unit: archer,
                target: hex(6, 5),
            }],
        ),
    );
    assert!(events.iter().any(|e| matches!(e, Event::Battle { .. })));
    assert!(next.units.get(&guard).is_none_or(|u| u.hp < 100));
}

#[test]
fn known_threats_leave_out_barbarians_for_the_celts() {
    for (faction, marked) in [(Faction::Celts, false), (Faction::Switzerland, true)] {
        let mut s = peaceful();
        s.players[0].faction = faction;
        unit(&mut s, 0, UnitKind::Soldier, hex(4, 5));
        spawn(&mut s, UnitKind::Soldier, hex(6, 5));
        let view = crate::view::player_view(&s, PlayerId(0));
        let threats = crate::settlement::known_threats(&view);
        assert_eq!(threats.iter().any(|n| *n > 0), marked, "{faction:?}");
    }
}

#[test]
fn a_barbarian_beside_a_celtic_city_does_not_stop_it_healing() {
    for (faction, heals) in [(Faction::Celts, true), (Faction::Switzerland, false)] {
        let mut s = peaceful();
        s.players[0].faction = faction;
        let c = city(&mut s, 0, hex(5, 5), 2);
        s.cities.get_mut(&c).unwrap().hp = 100;
        spawn(&mut s, UnitKind::Soldier, hex(6, 5));
        let view = crate::view::player_view(&s, PlayerId(0));
        let detail = view
            .cities
            .iter()
            .find(|v| v.id == c)
            .unwrap()
            .detail
            .as_ref();
        assert_eq!(
            detail.is_some_and(|d| d.heal_blocked),
            !heals,
            "{faction:?}: the card's forecast"
        );
        assert_eq!(
            crate::rules::cities::heal_blocked_by_adjacent_enemy(&s, &s.cities[&c]),
            !heals,
            "{faction:?}: resolution"
        );
    }
}

//! Founding beside a never-explored city must lead to reconnaissance or a new site,
//! including when the brain is recreated from a saved game after every turn.

use legatum_ai::{Brain, RandomBrain, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::state::Map;
use legatum_core::visibility::update_explored;
use legatum_core::*;
use std::collections::{BTreeMap, BTreeSet};

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);

fn border_game(trapped: bool) -> (GameState, UnitId, Hex, Hex) {
    let mut setup = GameSetup::standard(7, [Faction::Switzerland, Faction::Byzantium]);
    setup.width = 24;
    setup.height = 16;
    setup.turn_limit = None;
    setup.players.truncate(2);
    for (i, player) in setup.players.iter_mut().enumerate() {
        player.team = TeamId(i as u8);
    }
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    for (owner, pos) in [
        (ME, Hex::from_offset(4, 8)),
        (ENEMY, Hex::from_offset(13, 8)),
    ] {
        let unit = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, unit, &mut Vec::new()).unwrap();
    }
    let site = Hex::from_offset(10, 8);
    let hidden = Hex::from_offset(13, 8);
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, site);
    if trapped {
        for h in site.neighbors() {
            state.map.get_mut(h).unwrap().terrain = Terrain::Mountains;
        }
        // Preserve the visible foreign border through an impassable sight lane.
        state.map.get_mut(Hex::from_offset(11, 8)).unwrap().terrain = Terrain::Ocean;
    }
    state.turn = 10;
    update_explored(&mut state);
    let view = player_view(&state, ME);
    assert_eq!(site.distance(hidden), legatum_core::data::MIN_CITY_GAP);
    assert!(!view.tile(hidden).unwrap().explored);
    assert_eq!(view.tile(hidden).unwrap().terrain, None);
    let border = view.tile(Hex::from_offset(12, 8)).unwrap();
    assert!(border.visible);
    assert_eq!(border.owner, Some(ENEMY));
    assert!(view.cities.iter().all(|c| c.owner == ME));
    (state, settler, site, hidden)
}

fn settler_orders(state: &GameState, settler: UnitId) -> Vec<Order> {
    let view = player_view(state, ME);
    let mut brain = ScriptedBrain::new(state.player(ME).faction, state.setup.seed, ME);
    let orders: Vec<_> = brain
        .orders(&view)
        .into_iter()
        .filter(|order| legatum_core::rules::units::order_unit(order) == Some(settler))
        .collect();
    assert!(validate_orders(state, ME, &orders).is_empty());
    orders
}

#[test]
fn stationary_settler_avoids_a_never_explored_city_using_visible_borders() {
    let (state, settler, site, hidden) = border_game(false);
    let orders = settler_orders(&state, settler);
    let [Order::MoveUnit { unit, to }] = orders.as_slice() else {
        panic!("expected a new site, got {orders:?}");
    };
    assert_eq!(*unit, settler);
    assert_ne!(*to, site);
    let view = player_view(&state, ME);
    let step = view.tile(*to).unwrap();
    assert!(step.explored && step.terrain.unwrap().move_cost().is_some());
    assert!(to.distance(hidden) > legatum_core::data::MIN_CITY_GAP);
    let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, orders)]));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { .. }))
    );
    assert_ne!(next.units[&settler].pos, site);
}

#[test]
fn failed_founding_is_not_repeated_even_after_waits_and_save_load() {
    for trapped in [false, true] {
        let (mut state, settler, site, _) = border_game(trapped);
        // Fresh arrivals now avoid the risky site. Force a legacy/player order to test
        // durable recovery from a rejection independently of that prevention.
        state.units.get_mut(&settler).unwrap().moves_left = 0;
        assert!(!settler_orders(&state, settler).contains(&Order::FoundCity { unit: settler }));
        let orders = vec![Order::FoundCity { unit: settler }];
        let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, orders)]));
        assert!(events.iter().any(|e| matches!(e,
            Event::OrderFailed { player, reason }
                if *player == ME && reason.contains("Too close to")
        )));
        state = next;
        assert_eq!(
            player_view(&state, ME).failed_city_sites,
            BTreeSet::from([site])
        );
        assert!(player_view(&state, ENEMY).failed_city_sites.is_empty());
        let mut attempted = BTreeSet::from([site]);
        for _ in 0..16 {
            // A one-turn cooldown or brain-local memory must not pass this regression.
            state = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
            let Some(unit) = state.units.get(&settler) else {
                break;
            };
            let pos = unit.pos;
            let orders = settler_orders(&state, settler);
            if trapped {
                let random =
                    RandomBrain::new(state.setup.seed, ME).orders(&player_view(&state, ME));
                assert!(!random.contains(&Order::FoundCity { unit: settler }));
            }
            if orders.contains(&Order::FoundCity { unit: settler }) {
                assert!(
                    attempted.insert(pos),
                    "repeated FoundCity at {pos:?}, trapped={trapped}, turn={}",
                    state.turn
                );
            }
            let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, orders)]));
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, Event::OrderFailed { player, .. } if *player == ME)),
                "{events:?}"
            );
            state = next;
        }
        if trapped {
            assert_eq!(state.units[&settler].pos, site);
        } else {
            assert_eq!(
                state.cities_of(ME).count(),
                2,
                "settler must recover and found elsewhere"
            );
        }
    }
}

#[test]
fn simultaneous_founding_failures_are_remembered_without_revealing_a_rival() {
    let mut setup = GameSetup::all_ai(7);
    setup.players.truncate(2);
    let mut state = blank_game(
        &setup,
        Map::new(setup.width, setup.height, Terrain::Grassland),
    );
    let a = Hex::from_offset(10, 8);
    let b = Hex::from_offset(13, 8);
    let first = spawn_unit(&mut state, ME, UnitKind::Settler, a);
    let second = spawn_unit(&mut state, ENEMY, UnitKind::Settler, b);
    update_explored(&mut state);
    let (state, events) = resolve_turn(
        &state,
        &BTreeMap::from([
            (ME, vec![Order::FoundCity { unit: first }]),
            (ENEMY, vec![Order::FoundCity { unit: second }]),
        ]),
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::OrderFailed { .. }))
            .count(),
        2
    );
    for (player, unit, site) in [(ME, first, a), (ENEMY, second, b)] {
        let view = player_view(&state, player);
        assert_eq!(view.failed_city_sites, BTreeSet::from([site]));
        assert!(view.units.iter().all(|u| u.owner == player));
        let orders = ScriptedBrain::new(view.faction_of(player), setup.seed, player).orders(&view);
        assert!(!orders.contains(&Order::FoundCity { unit }));
        assert!(
            orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { unit: id, .. } if *id == unit))
        );
    }
}

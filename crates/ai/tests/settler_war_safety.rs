//! Settlers stay out of territory and threat zones of teams they are fighting.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::rules::units::order_unit;
use legatum_core::state::Map;
use legatum_core::visibility::update_explored;
use legatum_core::*;

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);
const HOME: Hex = Hex::new(2, 6);
const WAR_BORDER_Q: i32 = 8;
const BEHIND_BORDER: Hex = Hex::new(11, 6);
const SAFE_SITE: Hex = Hex::new(6, 6);

fn scenario(war: bool, sites: &[Hex]) -> (GameState, UnitId) {
    let mut setup = GameSetup::all_ai(31);
    setup.width = 20;
    setup.height = 12;
    setup.barbarians = false;
    setup.players.truncate(2);
    setup.players[ME.0 as usize].faction = Faction::India;
    setup.players[ENEMY.0 as usize].faction = Faction::Switzerland;
    let mut state = blank_game(&setup, Map::new(20, 12, Terrain::Plains));

    let founder = spawn_unit(&mut state, ME, UnitKind::Settler, HOME);
    found_city(&mut state, founder, &mut Vec::new()).unwrap();
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, HOME);

    if war {
        let diplomacy = state.diplomacy.as_mut().unwrap();
        for relation in &mut diplomacy.relations {
            relation.state = legatum_core::diplomacy::RelationState::War;
        }
    }
    // A known, full-height enemy border makes every route to the eastern site cross enemy land.
    for r in 0..state.map.height {
        state.map.get_mut(Hex::new(WAR_BORDER_Q, r)).unwrap().owner = Some(ENEMY);
    }
    state.players[ME.0 as usize]
        .tile_memory
        .resize(state.map.len(), None);
    for i in 0..state.map.len() {
        let h = state.map.hex_at(i);
        state.player_mut(ME).explored.set(i);
        if h.q == WAR_BORDER_Q {
            state.players[ME.0 as usize].tile_memory[i] = Some(legatum_core::state::TileMemory {
                owner: Some(ENEMY),
                ruin: false,
                improved: false,
                city: None,
            });
        }
        if !sites.contains(&h) {
            state.player_mut(ME).failed_city_sites.insert(h);
        }
    }
    state.turn = 20;
    update_explored(&mut state);
    if war {
        assert!(state.at_war(ME, ENEMY));
        assert_eq!(
            state.map.get(Hex::new(WAR_BORDER_Q, 6)).unwrap().owner,
            Some(ENEMY)
        );
    }
    (state, settler)
}

fn settler_order(state: &GameState, settler: UnitId) -> Order {
    let view = player_view(state, ME);
    let orders = ScriptedBrain::new(view.faction_of(ME), state.setup.seed, ME).orders(&view);
    assert!(validate_orders(state, ME, &orders).is_empty(), "{orders:?}");
    orders
        .iter()
        .find(|order| order_unit(order) == Some(settler))
        .cloned()
        .unwrap_or(Order::SkipUnit { unit: settler })
}

fn destination(order: &Order) -> Option<Hex> {
    match order {
        Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => Some(*to),
        _ => None,
    }
}

#[test]
fn at_war_a_site_behind_the_enemy_border_is_rejected_and_the_settler_waits_at_home() {
    let (state, settler) = scenario(true, &[BEHIND_BORDER]);
    let view = player_view(&state, ME);
    assert!(view.at_war(ME, ENEMY));
    assert_eq!(
        view.tile(Hex::new(WAR_BORDER_Q, 6)).unwrap().owner,
        Some(ENEMY)
    );
    let order = settler_order(&state, settler);
    assert_eq!(state.units[&settler].pos, HOME);
    assert_ne!(destination(&order), Some(BEHIND_BORDER), "{order:?}");
    assert!(matches!(order, Order::SkipUnit { .. }), "{order:?}");
}

#[test]
fn at_war_a_safe_site_wins_over_one_requiring_a_route_through_enemy_land() {
    let (state, settler) = scenario(true, &[BEHIND_BORDER, SAFE_SITE]);
    let order = settler_order(&state, settler);
    assert_eq!(destination(&order), Some(SAFE_SITE), "{order:?}");
    let view = player_view(&state, ME);
    let Order::MoveUnitPath {
        path,
        stop_on_block,
        ..
    } = order
    else {
        panic!("wartime routes stay confirmed and stop if blocked: {order:?}");
    };
    assert!(stop_on_block);
    assert!(
        path.iter()
            .all(|h| view.tile(*h).unwrap().owner != Some(ENEMY)),
        "{path:?}"
    );
}

#[test]
fn at_war_a_visible_enemy_army_closes_sites_within_the_threat_radius() {
    let (mut state, settler) = scenario(true, &[SAFE_SITE]);
    spawn_unit(&mut state, ME, UnitKind::Scout, Hex::new(7, 6));
    spawn_unit(
        &mut state,
        ENEMY,
        UnitKind::Soldier,
        Hex::new(SAFE_SITE.q + 3, SAFE_SITE.r),
    );
    update_explored(&mut state);
    assert!(player_view(&state, ME).units.iter().any(|u| {
        u.owner == ENEMY && u.kind == UnitKind::Soldier && u.pos.distance(SAFE_SITE) == 3
    }));
    let order = settler_order(&state, settler);
    assert_ne!(destination(&order), Some(SAFE_SITE), "{order:?}");
}

//! V4 uses only public diplomacy and fogged combat evidence; resolve the real boundaries.
use legatum_ai::{Brain, PROXY_VERSION, ProxyBrain, ProxyV2Brain};
use legatum_core::diplomacy::{RelationState, TreatyKind};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::Map;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
fn h(c: i32, r: i32) -> Hex {
    Hex::from_offset(c, r)
}

fn situation() -> GameState {
    let mut setup = GameSetup::all_ai(44);
    setup.width = 30;
    setup.height = 22;
    setup.players.truncate(4);
    for p in &mut setup.players {
        p.faction = Faction::India;
    }
    let mut s = blank_game(&setup, Map::new(30, 22, Terrain::Grassland));
    for (p, pos) in [(0, h(5, 8)), (1, h(12, 8)), (2, h(5, 15)), (3, h(27, 19))] {
        let settler = spawn_unit(&mut s, PlayerId(p), UnitKind::Settler, pos);
        rules::cities::found_city(&mut s, settler, &mut vec![]).unwrap();
    }
    for (kind, pos) in [
        (UnitKind::Trooper, h(8, 8)),
        (UnitKind::Ranger, h(7, 8)),
        (UnitKind::Ranger, h(7, 7)),
        (UnitKind::Scout, h(10, 8)),
        (UnitKind::Scout, h(5, 13)),
    ] {
        spawn_unit(&mut s, ME, kind, pos);
    }
    spawn_unit(&mut s, PlayerId(2), UnitKind::Trooper, h(5, 15));
    s.turn = 50;
    for player in &mut s.players {
        player.gold = 1000;
    }
    visibility::update_explored(&mut s);
    // Alliance choices are tested apart from first contact (core tests the contact rule).
    let teams = s.teams();
    s.diplomacy.as_mut().unwrap().meet_all(&teams);
    s
}

fn choices(s: &GameState) -> Vec<Order> {
    let view = player_view(s, ME);
    let orders = ProxyBrain.orders(&view);
    assert!(
        validate_orders(s, ME, &orders).is_empty(),
        "{:?}",
        validate_orders(s, ME, &orders)
    );
    let saved: GameState = serde_json::from_str(&serde_json::to_string(s).unwrap()).unwrap();
    assert_eq!(orders, ProxyBrain.orders(&player_view(&saved, ME)));
    orders
        .into_iter()
        .filter(diplomacy::is_diplomatic)
        .collect()
}

fn offer(s: &GameState, from: u8, to: u8, kind: TreatyKind) -> GameState {
    resolve_turn(
        s,
        &BTreeMap::from([(
            PlayerId(from),
            vec![Order::ProposeTreaty {
                to: TeamId(to),
                kind,
            }],
        )]),
    )
    .0
}

fn sign(s: &GameState, from: u8, to: u8, kind: TreatyKind) -> GameState {
    let s = offer(s, from, to, kind);
    let proposal = s
        .diplomacy
        .as_ref()
        .unwrap()
        .proposals
        .iter()
        .find(|p| p.from == TeamId(from) && p.to == TeamId(to))
        .unwrap()
        .id;
    let (s, events) = resolve_turn(
        &s,
        &BTreeMap::from([(
            PlayerId(to),
            vec![Order::RespondToProposal {
                proposal,
                accept: true,
                explanation: None,
            }],
        )]),
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TreatyActivated { .. })),
        "{events:?}"
    );
    s
}

#[test]
fn useful_flank_alliance_is_accepted_and_old_proxy_stays_frozen() {
    assert_eq!(PROXY_VERSION, 4);
    let s = situation();
    let s = offer(&s, 2, 0, TreatyKind::Alliance);
    let p = s.diplomacy.as_ref().unwrap().proposals[0].id;
    let accept = Order::RespondToProposal {
        proposal: p,
        accept: true,
        explanation: None,
    };
    assert!(choices(&s).contains(&accept), "orders={:?}", choices(&s));
    let (s, events) = resolve_turn(&s, &BTreeMap::from([(ME, choices(&s))]));
    assert!(events.iter().any(|e| matches!(e, Event::TreatyActivated { relation, .. } if relation.state == RelationState::Alliance)));
    assert!(
        !choices(&s)
            .iter()
            .any(|o| matches!(o, Order::BreakTreaty { .. }))
    );
}

#[test]
fn an_alliance_does_not_protect_a_rival_about_to_win() {
    let s = situation();
    let s = offer(&s, 2, 0, TreatyKind::Alliance);
    let mut v = player_view(&s, ME);
    let p = v.diplomacy.proposals[0].id;
    let rival = v
        .standings
        .iter_mut()
        .find(|s| s.team == TeamId(2))
        .unwrap();
    rival.vp = 8;
    assert!(ProxyBrain.orders(&v).contains(&Order::RespondToProposal {
        proposal: p,
        accept: false,
        explanation: None
    }));
}

#[test]
fn a_war_the_proxy_declares_calls_its_ally_to_arms() {
    let s = sign(&situation(), 2, 0, TreatyKind::Alliance);
    let (s, events) = resolve_turn(
        &s,
        &BTreeMap::from([(
            ME,
            vec![Order::DeclareWar {
                against: TeamId(1),
                breach: false,
            }],
        )]),
    );
    assert!(s.at_war(ME, PlayerId(1)));
    assert!(
        !s.at_war(PlayerId(2), PlayerId(1)),
        "the ally is asked, not dragged in"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CallToArms { call } if call.team == TeamId(2) && call.ally == TeamId(0))),
        "{events:?}"
    );
}

#[test]
fn a_war_on_the_proxys_ally_brings_the_proxy_in() {
    let s = sign(&situation(), 2, 0, TreatyKind::Alliance);
    let (s, events) = resolve_turn(
        &s,
        &BTreeMap::from([(
            PlayerId(1),
            vec![Order::DeclareWar {
                against: TeamId(2),
                breach: false,
            }],
        )]),
    );
    assert!(s.at_war(PlayerId(1), PlayerId(2)));
    assert!(s.at_war(ME, PlayerId(1)), "the ally defends at once");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::WarJoined { ally, .. } if *ally == TeamId(2))),
        "{events:?}"
    );
}

#[test]
fn an_enemys_local_ally_counts_even_when_its_army_is_out_of_sight() {
    let mut s = situation();
    for x in 1..6 {
        spawn_unit(&mut s, PlayerId(2), UnitKind::AssaultTrooper, h(x, 20));
    }
    let s = sign(&s, 1, 2, TreatyKind::Alliance);
    let v = player_view(&s, ME);
    let war = Order::DeclareWar {
        against: TeamId(1),
        breach: false,
    };
    assert!(
        !ProxyBrain.orders(&v).contains(&war),
        "dip={:?}",
        v.diplomacy.relations
    );
}

#[test]
fn legacy_orders_and_fog_invariance_are_preserved() {
    let s = situation();
    let before = player_view(&s, ME);
    let mut hidden = s.clone();
    hidden.player_mut(PlayerId(3)).science_stored += 100;
    let city = hidden.cities_of(PlayerId(3)).next().unwrap().id;
    hidden.cities.get_mut(&city).unwrap().hp = 1;
    let after = player_view(&hidden, ME);
    assert_eq!(before, after);
    assert_eq!(ProxyBrain.orders(&before), ProxyBrain.orders(&after));
    hidden.diplomacy = None;
    let legacy = player_view(&hidden, ME);
    assert_eq!(ProxyBrain.orders(&legacy), ProxyV2Brain.orders(&legacy));
}

#[test]
fn a_vulnerable_lead_waits_on_a_nonwinning_great_work_but_never_delays_victory() {
    let mut s = situation();
    s.player_mut(ME).techs.extend([
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
    let city = s.cities_of(ME).next().unwrap().id;
    s.cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::Laboratory);
    for x in 1..8 {
        spawn_unit(&mut s, PlayerId(2), UnitKind::AssaultTrooper, h(x, 20));
    }
    visibility::update_explored(&mut s);
    let mut v = player_view(&s, ME);
    for standing in &mut v.standings {
        standing.vp = if standing.team == v.team { 5 } else { 4 };
    }
    let work = |o: &Order| matches!(o, Order::SetProduction { item, .. } | Order::BuyItem { item, .. } if legatum_core::data::is_great_work_item(*item));
    assert!(!ProxyBrain.orders(&v).iter().any(work));
    // The same military situation must never block a project that ends the game.
    v.standings
        .iter_mut()
        .find(|s| s.team == v.team)
        .unwrap()
        .vp = 8;
    assert!(ProxyBrain.orders(&v).iter().any(work));
    // A signed flank guarantee removes the reason to defer the earlier project.
    v.standings
        .iter_mut()
        .find(|s| s.team == v.team)
        .unwrap()
        .vp = 5;
    v.diplomacy
        .relations
        .iter_mut()
        .find(|r| r.between(v.team, TeamId(2)))
        .unwrap()
        .guarantee_until = v.turn + 20;
    assert!(ProxyBrain.orders(&v).iter().any(work));
}

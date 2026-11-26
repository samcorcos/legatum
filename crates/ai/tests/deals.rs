//! Deals (rules 61, `GameSetup::deals`): how the AI uses Gold in offers, coalition Gold and Open
//! Borders (`crates/ai/src/deals.rs`).
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::diplomacy::{Agreement, ProposalId, RelationState, TreatyKind};
use legatum_core::{
    game::{blank_game, spawn_unit},
    state::Map,
    *,
};
use std::collections::BTreeMap;

fn h(c: i32, r: i32) -> Hex {
    Hex::from_offset(c, r)
}

/// Three empires, all met, 500 Gold each: Sparta (0) at h(4,6) with five Pikemen, an empire
/// at h(11,6) and one far away at h(24,15). `deals` sets the switch.
fn situation(deals: bool) -> GameState {
    situation_at(deals, h(24, 15))
}

/// `situation` with the third empire's city at `far`.
fn situation_at(deals: bool, far: Hex) -> GameState {
    let mut setup = GameSetup::all_ai(44);
    setup.width = 28;
    setup.height = 20;
    setup.players.truncate(3);
    setup.players[0].faction = Faction::Sparta;
    setup.deals = deals;
    let mut s = blank_game(&setup, Map::new(28, 20, Terrain::Grassland));
    for (player, pos) in [(0, h(4, 6)), (1, h(11, 6)), (2, far)] {
        let settler = spawn_unit(&mut s, PlayerId(player), UnitKind::Settler, pos);
        rules::cities::found_city(&mut s, settler, &mut vec![]).unwrap();
    }
    for pos in [h(5, 6), h(6, 6), h(5, 7), h(6, 7), h(7, 7)] {
        spawn_unit(&mut s, PlayerId(0), UnitKind::Pikeman, pos);
    }
    visibility::update_explored(&mut s);
    // Every city is known to Sparta (remembered, not in sight).
    visibility::reveal_all_to_team(&mut s, PlayerId(0));
    let teams = s.teams();
    s.diplomacy.as_mut().unwrap().meet_all(&teams);
    for p in &mut s.players {
        p.gold = 500;
    }
    s
}

fn set_relation(s: &mut GameState, a: u8, b: u8, state: RelationState, since: u32) {
    let r = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(a), TeamId(b)))
        .unwrap();
    r.state = state;
    r.effective_turn = since;
}

fn set_armies(v: &mut PlayerView, values: &[(PlayerId, i32)]) {
    for p in v.players.iter_mut() {
        p.army = values
            .iter()
            .find(|(id, _)| *id == p.id)
            .map_or(0, |(_, n)| *n);
    }
}

/// Plain-Hard Sparta policy (`sparta_search_only` = 1): Peace judged on strength and danger.
fn brain() -> ScriptedBrain {
    let mut brain = ScriptedBrain::new(Faction::Sparta, 99, PlayerId(0));
    brain.decisions.sparta_search_only = 1;
    brain
}

fn choices(view: &PlayerView) -> Vec<Order> {
    brain()
        .orders(view)
        .into_iter()
        .filter(|o| !(!(diplomacy::is_diplomatic(o)) && !(matches!(o, Order::GiveGold { .. }))))
        .collect()
}

/// Team 2 offers Team 0 a Peace carrying `gold` (resolved, so the offer is on the table).
fn offered(s: &GameState, from: u8, kind: TreatyKind, gold: i32) -> GameState {
    let (mut s, _) = resolve_turn(
        s,
        &BTreeMap::from([(
            PlayerId(from),
            vec![Order::ProposeTreaty {
                to: TeamId(0),
                kind,
                gold,
            }],
        )]),
    );
    assert_eq!(s.diplomacy.as_ref().unwrap().proposals.len(), 1);
    for p in &mut s.players {
        p.gold = 500;
    }
    s
}

fn at_war(turn: u32, deals: bool) -> GameState {
    let mut s = situation(deals);
    s.turn = turn;
    set_relation(&mut s, 0, 2, RelationState::War, 1);
    s
}

fn view_with(s: &GameState, armies: &[(PlayerId, i32)]) -> PlayerView {
    let mut v = player_view(s, PlayerId(0));
    set_armies(&mut v, armies);
    v
}

fn answer(v: &PlayerView) -> bool {
    choices(v)
        .into_iter()
        .find_map(|o| match o {
            Order::RespondToProposal {
                proposal: ProposalId(1),
                accept,
                ..
            } => Some(accept),
            _ => None,
        })
        .expect("an answer to the offer")
}

#[test]
fn a_losing_ai_pays_a_fifth_of_its_gold_for_a_truce_and_only_with_deals_on() {
    let losing = [(PlayerId(0), 40), (PlayerId(2), 100)];
    let even = [(PlayerId(0), 100), (PlayerId(2), 100)];
    let peace = |gold| Order::ProposeTreaty {
        to: TeamId(2),
        kind: TreatyKind::Peace,
        gold,
    };
    // Without deals the offer is plain, as before.
    assert_eq!(choices(&view_with(&at_war(50, false), &losing)), [peace(0)]);
    // Losing (our army 40% of theirs): 20% of 500 Gold.
    assert_eq!(
        choices(&view_with(&at_war(50, true), &losing)),
        [peace(100)]
    );
    // An even war: plain Peace.
    assert_eq!(choices(&view_with(&at_war(50, true), &even)), [peace(0)]);
    // A poor treasury pays nothing rather than a pittance: 20% of 60 is under 20.
    let mut s = at_war(50, true);
    s.players[0].gold = 60;
    assert_eq!(choices(&view_with(&s, &losing)), [peace(0)]);
    // Rich: capped at 200.
    s.players[0].gold = 5000;
    assert_eq!(choices(&view_with(&s, &losing)), [peace(200)]);
}

#[test]
fn a_losing_ai_pays_a_reasonable_price_for_peace_and_an_even_one_pays_nothing() {
    let losing = [(PlayerId(0), 40), (PlayerId(2), 100)];
    let even = [(PlayerId(0), 100), (PlayerId(2), 100)];
    let asked = |gold| offered(&at_war(50, true), 2, TreatyKind::Peace, gold);
    // Losing, 100 of its 500 Gold (the cap is a third, 165): it pays.
    assert!(answer(&view_with(&asked(-100), &losing)));
    // 200 is more than a third of its Gold: refused.
    assert!(!answer(&view_with(&asked(-200), &losing)));
    // Not losing: it would take a free Peace, but pays nothing for one.
    assert!(answer(&view_with(&asked(0), &even)));
    assert!(!answer(&view_with(&asked(-100), &even)));
}

#[test]
fn an_offer_that_asks_for_an_alliance_fee_is_refused() {
    let mut s = situation(true);
    s.turn = 30;
    let s = offered(&s, 1, TreatyKind::Alliance, -50);
    assert!(!answer(&view_with(&s, &[(PlayerId(0), 100)])));
}

/// Sparta at h(4,6), its neighbour (Team 1) at h(11,6) and Team 2 at h(18,6), beyond it.
fn borders(war_beyond: bool) -> GameState {
    let mut s = situation_at(true, h(18, 6));
    s.turn = 50;
    if !(war_beyond) {
    } else {
        set_relation(&mut s, 0, 2, RelationState::War, 1);
    }
    s
}

const CALM: [(PlayerId, i32); 3] = [(PlayerId(0), 100), (PlayerId(1), 50), (PlayerId(2), 50)];

fn open_borders_offer(to: u8) -> Order {
    Order::ProposeTreaty {
        to: TeamId(to),
        kind: TreatyKind::OpenBorders,
        gold: 0,
    }
}

#[test]
fn open_borders_go_to_a_harmless_neighbour_whose_land_lies_on_the_road_to_war() {
    // At War with Team 2 beyond Team 1: offered, and accepted when Team 1 asks.
    let s = borders(true);
    assert!(choices(&view_with(&s, &CALM)).contains(&open_borders_offer(1)));
    assert!(answer(&view_with(
        &offered(&s, 1, TreatyKind::OpenBorders, 0),
        &CALM
    )));
    // No war beyond: the road leads nowhere, so neither offered nor accepted...
    let s = borders(false);
    assert!(!choices(&view_with(&s, &CALM)).contains(&open_borders_offer(1)));
    assert!(!answer(&view_with(
        &offered(&s, 1, TreatyKind::OpenBorders, 0),
        &CALM
    )));
    // ...unless the neighbour pays for passage.
    assert!(answer(&view_with(
        &offered(&s, 1, TreatyKind::OpenBorders, 50),
        &CALM
    )));
    // Never paid for by us.
    assert!(!answer(&view_with(
        &offered(&borders(true), 1, TreatyKind::OpenBorders, -50),
        &CALM
    )));
    // Without deals the AI never offers them (and the view lists no such order).
    let mut s = borders(true);
    s.setup.deals = false;
    assert!(!choices(&view_with(&s, &CALM)).iter().any(|o| matches!(
        o,
        Order::ProposeTreaty {
            kind: TreatyKind::OpenBorders,
            ..
        }
    )));
}

#[test]
fn open_borders_are_refused_to_a_stronger_neighbour_the_leader_or_an_army_at_the_border() {
    let offer = |s: &GameState| offered(s, 1, TreatyKind::OpenBorders, 0);
    // A stronger army next door.
    let stronger = [(PlayerId(0), 100), (PlayerId(1), 120), (PlayerId(2), 50)];
    assert!(!answer(&view_with(&offer(&borders(true)), &stronger)));
    assert!(!choices(&view_with(&borders(true), &stronger)).contains(&open_borders_offer(1)));
    // The public leader.
    let mut v = view_with(&offer(&borders(true)), &CALM);
    let mut standings = v.standings.to_vec();
    for st in &mut standings {
        st.vp = if !(st.team == TeamId(1)) { 1 } else { 3 };
        st.target = 10;
    }
    v.standings = standings.into();
    assert!(!answer(&v));
    // An army within five hexes of our city.
    let mut s = borders(true);
    spawn_unit(&mut s, PlayerId(1), UnitKind::Pikeman, h(8, 6));
    visibility::update_explored(&mut s);
    assert!(!answer(&view_with(&offer(&s), &CALM)));
}

fn with_open_borders(s: &mut GameState) {
    s.diplomacy
        .as_mut()
        .unwrap()
        .open_borders
        .insert((TeamId(0), TeamId(1)));
}

#[test]
fn open_borders_end_when_the_partner_becomes_a_threat_with_a_margin() {
    let close = Order::BreakTreaty {
        with: TeamId(1),
        agreement: Agreement::OpenBorders,
    };
    let mut s = borders(true);
    with_open_borders(&mut s);
    assert!(
        view_with(&s, &CALM)
            .diplomacy
            .open_borders
            .contains(&(TeamId(0), TeamId(1)))
    );
    assert!(!choices(&view_with(&s, &CALM)).contains(&close));
    // A little stronger than us: no longer offered, but not worth ending (hysteresis).
    let edge = [(PlayerId(0), 100), (PlayerId(1), 120), (PlayerId(2), 50)];
    assert!(!choices(&view_with(&s, &edge)).contains(&close));
    // Well stronger: ended.
    let strong = [(PlayerId(0), 100), (PlayerId(1), 130), (PlayerId(2), 50)];
    assert!(choices(&view_with(&s, &strong)).contains(&close));
    // Three of its units within three hexes of our city: ended.
    for at in [h(5, 5), h(6, 5), h(7, 6)] {
        spawn_unit(&mut s, PlayerId(1), UnitKind::Pikeman, at);
    }
    visibility::update_explored(&mut s);
    assert!(choices(&view_with(&s, &CALM)).contains(&close));
}

#[test]
fn hard_gives_a_little_gold_to_a_partner_fighting_the_leader_it_fights() {
    let gifts = |deals: bool, turn: u32| {
        let mut s = situation(deals);
        s.turn = turn;
        set_relation(&mut s, 0, 2, RelationState::War, 1);
        set_relation(&mut s, 1, 2, RelationState::War, 1);
        let mut v = view_with(
            &s,
            &[(PlayerId(0), 100), (PlayerId(1), 80), (PlayerId(2), 100)],
        );
        v.economy.gold_per_turn.total = 10;
        v.economy.bankruptcy = None;
        let mut standings = v.standings.to_vec();
        for st in &mut standings {
            st.vp = if !(st.team == TeamId(2)) { 2 } else { 7 };
            st.target = 10;
        }
        v.standings = standings.into();
        let mut brain = ScriptedBrain::with_difficulty(
            Faction::Sparta,
            99,
            PlayerId(0),
            legatum_core::AiDifficulty::Hard,
        );
        brain.decisions.sparta_search_only = 1;
        brain
            .orders(&v)
            .into_iter()
            .filter(|o| matches!(o, Order::GiveGold { .. }))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        gifts(true, 50),
        [Order::GiveGold {
            to: PlayerId(1),
            amount: 50
        }]
    );
    // Only every fifth turn, and never without deals.
    assert!(gifts(true, 51).is_empty());
    assert!(gifts(false, 50).is_empty());
}

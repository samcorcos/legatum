//! Japan's Sakoku vow (`GoalRule::Sakoku`): the action, what a closed country blocks, the
//! countdown, and holding and losing the live VP.

use super::*;
use crate::diplomacy::{RelationState, TreatyKind};
use crate::economy;
use crate::events::Event;
use crate::game::spawn_unit;
use crate::hex::Hex;
use crate::orders::{Action, Order};
use crate::rules::actions::{sakoku_blocker, validate_actions};
use crate::rules::cities::tests::{P0, P1, P2, at, found, test_game as base_test_game};
use crate::scoring;

// Keep the reserved Japan rule exercised through an explicit scenario override.
fn test_game(players: &[(Faction, u8)]) -> GameState {
    let mut state = base_test_game(players);
    for p in &mut state.players {
        if p.faction == Faction::India {
            p.goal_rule = Some(GoalRule::Sakoku);
        }
    }
    state
}

fn close() -> Order {
    Order::Action {
        action: Action::Sakoku { close: true },
    }
}

fn reopen() -> Order {
    Order::Action {
        action: Action::Sakoku { close: false },
    }
}

fn relate(state: &mut GameState, a: PlayerId, b: PlayerId, to: RelationState) {
    let (ta, tb) = (state.team_of(a), state.team_of(b));
    let d = state.diplomacy.as_mut().unwrap();
    d.meet(ta, tb);
    for r in &mut d.relations {
        if r.between(ta, tb) {
            r.state = to;
        }
    }
}

/// India with the legacy Sakoku rule (P0), a Byzantium (P1) and a Mongols (P2), all at Peace, each with a city.
fn world() -> GameState {
    let mut s = test_game(&[
        (Faction::India, 0),
        (Faction::Byzantium, 1),
        (Faction::Mongols, 2),
    ]);
    found(&mut s, P0, at(3, 3));
    found(&mut s, P1, at(14, 14));
    found(&mut s, P2, at(3, 15));
    for (a, b) in [(P0, P1), (P0, P2), (P1, P2)] {
        relate(&mut s, a, b, RelationState::Peace);
    }
    for p in [P0, P1, P2] {
        s.player_mut(p).gold = 1000;
    }
    s
}

fn close_now(s: &mut GameState) -> Vec<Event> {
    let mut ev = Vec::new();
    crate::rules::actions::apply_actions(s, P0, &[close()], &mut ev);
    ev
}

fn run(s: &mut GameState) -> Vec<Event> {
    let mut ev = Vec::new();
    scoring::update(s, &mut ev);
    ev
}

/// A land tile of India's territory that is not its city.
fn india_tile(s: &GameState) -> Hex {
    let city = s.cities_of(P0).next().unwrap().pos;
    city.within(1)
        .into_iter()
        .find(|h| *h != city && s.map.get(*h).is_some_and(|t| t.owner == Some(P0)))
        .expect("an owned tile beside the city")
}

fn goal_vp(s: &GameState, p: PlayerId) -> bool {
    scoring::vp(s, p)
        .iter()
        .any(|(src, _)| matches!(src, VpSource::FactionGoal { of } if *of == p))
}

#[test]
fn india_follows_diwan_and_aztecs_follow_the_swarm() {
    for f in Faction::ALL {
        assert_eq!(default_rule(f) == GoalRule::Swarm, f == Faction::Aztecs);
        assert_eq!(default_rule(f) == GoalRule::Diwan, f == Faction::India);
    }
    let spec = spec(Faction::India);
    assert_eq!(
        (spec.kind, spec.vp, spec.rule),
        (GoalKind::Held, 2, GoalRule::Diwan)
    );
    // Sakoku is fixed at 30 turn ends.
    assert_eq!(sakoku_target(), 30);
}

#[test]
fn the_vow_closes_and_reopens_and_is_public() {
    let mut s = world();
    assert!(!is_closed(&s, P0));
    let ev = close_now(&mut s);
    assert!(is_closed(&s, P0));
    assert_eq!(s.player(P0).sakoku_turns, Some(0));
    assert!(ev.contains(&Event::SakokuClosed { player: P0 }));
    // The closed state and its countdown are on every empire's view.
    let rival = crate::view::player_view(&s, P1);
    assert_eq!(rival.players[0].sakoku_turns, Some(0));
    assert!(rival.is_closed(P0) && !rival.is_closed(P1));
    // A second close is refused; reopening works once.
    assert!(
        sakoku_blocker(&s, P0, true)
            .unwrap()
            .contains("already closed")
    );
    let mut ev = Vec::new();
    s.player_mut(P0).sakoku_turns = Some(7);
    crate::rules::actions::apply_actions(&mut s, P0, &[reopen()], &mut ev);
    assert!(!is_closed(&s, P0));
    assert!(ev.contains(&Event::SakokuReopened {
        player: P0,
        turns: 7
    }));
    assert!(
        sakoku_blocker(&s, P0, false)
            .unwrap()
            .contains("already open")
    );
    // Only an empire assigned the reserved Sakoku rule has the vow.
    assert!(
        sakoku_blocker(&s, P1, true)
            .unwrap()
            .contains("with Sakoku")
    );
}

#[test]
fn closing_needs_no_alliance_and_no_courier_hall() {
    let mut s = world();
    relate(&mut s, P0, P1, RelationState::Alliance);
    assert!(sakoku_blocker(&s, P0, true).unwrap().contains("Alliance"));
    relate(&mut s, P0, P1, RelationState::Peace);
    assert!(sakoku_blocker(&s, P0, true).is_none());
    // A captured Courier Hall opens its holder's road: the vow contradicts it.
    let hall_city = s.cities_of(P0).next().unwrap().id;
    s.cities
        .get_mut(&hall_city)
        .unwrap()
        .buildings
        .insert(BuildingKind::YamRelayStation);
    s.player_mut(P2).hall_city = Some(hall_city);
    assert!(holds_open_road(&s, P0));
    assert!(
        sakoku_blocker(&s, P0, true)
            .unwrap()
            .contains("Courier Hall")
    );
}

#[test]
fn a_vow_cannot_ride_with_a_declaration_or_a_signature() {
    let s = world();
    let war = Order::DeclareWar {
        against: s.team_of(P1),
        breach: false,
    };
    let errs = validate_actions(&s, P0, &[war, close()]);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].message.contains("can't declare war"));
    let ally = Order::ProposeTreaty {
        to: s.team_of(P1),
        kind: TreatyKind::Alliance,
    };
    assert_eq!(validate_actions(&s, P0, &[ally, close()]).len(), 1);
    // A lone vow is fine, and so is accepting something that is not an Alliance (here, a proposal
    // that does not exist as an Alliance: Peace acceptances ride with the vow).
    assert!(validate_actions(&s, P0, &[close()]).is_empty());
    let accept = Order::RespondToProposal {
        proposal: crate::diplomacy::ProposalId(99),
        accept: true,
        explanation: None,
    };
    assert!(validate_actions(&s, P0, &[accept, close()]).is_empty());
}

#[test]
fn a_closed_country_cannot_declare_war_or_ally_but_can_make_peace() {
    let mut s = world();
    let war = Order::DeclareWar {
        against: s.team_of(P1),
        breach: false,
    };
    let ally = |to| Order::ProposeTreaty {
        to,
        kind: TreatyKind::Alliance,
    };
    // Open: both are legal.
    assert!(crate::diplomacy::check_order(&s, P0, &war).is_ok());
    assert!(crate::diplomacy::check_order(&s, P0, &ally(s.team_of(P1))).is_ok());
    close_now(&mut s);
    let err = crate::diplomacy::check_order(&s, P0, &war).unwrap_err();
    assert!(err.contains("closed the country"), "{err}");
    let err = crate::diplomacy::check_order(&s, P0, &ally(s.team_of(P1))).unwrap_err();
    assert!(err.contains("closed the country"), "{err}");
    // A rival can't ally with India either.
    let err = crate::diplomacy::check_order(&s, P1, &ally(s.team_of(P0))).unwrap_err();
    assert!(err.contains("closed the country"), "{err}");
    // But rivals may ally with each other, and India may still end a war.
    assert!(crate::diplomacy::check_order(&s, P1, &ally(s.team_of(P2))).is_ok());
    relate(&mut s, P0, P1, RelationState::War);
    let peace = Order::ProposeTreaty {
        to: s.team_of(P1),
        kind: TreatyKind::Peace,
    };
    assert!(crate::diplomacy::check_order(&s, P0, &peace).is_ok());
    // A rival declaring war on India is legal: wars others declare are fought normally.
    let at_india = Order::DeclareWar {
        against: s.team_of(P0),
        breach: false,
    };
    assert!(crate::diplomacy::check_order(&s, P2, &at_india).is_ok());
}

#[test]
fn a_pending_alliance_is_refused_once_the_country_is_closed() {
    let s = world();
    // P1 proposes an Alliance to India while it is open; India then closes the country.
    let propose = Order::ProposeTreaty {
        to: s.team_of(P0),
        kind: TreatyKind::Alliance,
    };
    let mut orders = std::collections::BTreeMap::new();
    orders.insert(P1, vec![propose]);
    let (mut s, _) = crate::rules::resolve_turn(&s, &orders);
    let proposal = s.diplomacy.as_ref().unwrap().proposals[0].id;
    close_now(&mut s);
    let accept = Order::RespondToProposal {
        proposal,
        accept: true,
        explanation: None,
    };
    let err = crate::diplomacy::check_order(&s, P0, &accept).unwrap_err();
    assert!(err.contains("closed the country"), "{err}");
}

#[test]
fn a_closed_country_lets_nobody_in_at_peace_but_its_own_team() {
    let mut s = test_game(&[
        (Faction::India, 0),
        (Faction::India, 0),
        (Faction::Byzantium, 1),
        (Faction::Mongols, 2),
        (Faction::Switzerland, 3),
    ]);
    let (mate, byzantium, glass, toll) = (PlayerId(1), PlayerId(2), PlayerId(3), PlayerId(4));
    for (a, b) in [(P0, byzantium), (P0, glass), (P0, toll)] {
        relate(&mut s, a, b, RelationState::Peace);
    }
    // Before the vow: foreign units cannot cross at Peace.
    assert!(!s.passage_allowed(byzantium, UnitKind::Spearman, Some(P0)));
    assert!(!s.passage_allowed(glass, UnitKind::Scout, Some(P0)));
    assert!(!s.passage_allowed(toll, UnitKind::Spearman, Some(P0)));
    s.player_mut(P0).sakoku_turns = Some(0);
    for kind in UnitKind::ALL {
        assert!(!s.passage_allowed(byzantium, kind, Some(P0)), "{kind:?}");
        assert!(!s.passage_allowed(glass, kind, Some(P0)), "{kind:?}");
        assert!(!s.passage_allowed(toll, kind, Some(P0)), "{kind:?}");
        // Its permanent teammate is unaffected (teams are not Alliances), and India itself goes
        // wherever it could before.
        assert!(s.passage_allowed(mate, kind, Some(P0)), "{kind:?}");
        assert!(s.passage_allowed(P0, kind, Some(mate)), "{kind:?}");
        assert!(!s.passage_allowed(P0, kind, Some(byzantium)));
    }
    // The player view agrees with the engine.
    let v = crate::view::player_view(&s, byzantium);
    for kind in UnitKind::ALL {
        for who in [byzantium, glass, toll, mate] {
            assert_eq!(
                v.passage_allowed(who, kind, Some(P0)),
                s.passage_allowed(who, kind, Some(P0)),
                "{who:?} {kind:?}"
            );
        }
    }
    // At war an army still walks in: closing blocks the peaceful ways in.
    relate(&mut s, P0, toll, RelationState::War);
    assert!(s.passage_allowed(toll, UnitKind::Spearman, Some(P0)));
    // Reopening restores the old rule.
    s.player_mut(P0).sakoku_turns = None;
    assert!(!s.passage_allowed(glass, UnitKind::Scout, Some(P0)));
    assert!(!s.passage_allowed(byzantium, UnitKind::Spearman, Some(P0)));
}

#[test]
fn a_closed_country_gets_no_tech_diffusion() {
    let mut s = world();
    let tech = TechId::ALL[0];
    for q in [P1, P2] {
        s.player_mut(q).techs.insert(tech);
    }
    let open = economy::tech_cost(&s, P0, tech);
    s.player_mut(P0).sakoku_turns = Some(3);
    let closed = economy::tech_cost(&s, P0, tech);
    assert!(
        closed.total > open.total,
        "two empires know it: open {} closed {}",
        open.total,
        closed.total
    );
    // The full price is the undiscounted one.
    let base = data::tech_stats(tech).cost;
    assert_eq!(closed.total, base);
}

#[test]
fn the_vp_is_held_after_the_countdown_while_no_enemy_stands_in_the_land() {
    let mut s = world();
    close_now(&mut s);
    let target = sakoku_target();
    // The countdown counts turn ends.
    for n in 1..target {
        run(&mut s);
        assert_eq!(s.player(P0).sakoku_turns, Some(n));
        assert!(!s.player(P0).faction_goal_done, "turn {n}");
    }
    let ev = run(&mut s);
    assert_eq!(s.player(P0).sakoku_turns, Some(target));
    assert!(s.player(P0).faction_goal_done && goal_vp(&s, P0));
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted {
            player: P0,
            vp: 2,
            again: false,
            ..
        }
    )));
    // Holding, it keeps scoring each turn without repeating the event.
    let ev = run(&mut s);
    assert!(!ev.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted { .. } | Event::FactionGoalLost { .. }
    )));
    assert!(goal_vp(&s, P0));
}

#[test]
fn enemy_units_in_indian_land_do_not_break_the_vow() {
    let mut s = world();
    close_now(&mut s);
    s.player_mut(P0).sakoku_turns = Some(sakoku_target());
    let tile = india_tile(&s);
    relate(&mut s, P0, P1, RelationState::War);
    spawn_unit(&mut s, P1, UnitKind::Spearman, tile);
    let ev = run(&mut s);
    assert!(goal_vp(&s, P0));
    assert!(s.player(P0).faction_goal_done);
    assert!(
        !ev.iter()
            .any(|e| matches!(e, Event::FactionGoalLost { player: P0, .. }))
    );
}

#[test]
fn reopening_loses_the_vp_and_restarts_the_count() {
    let mut s = world();
    close_now(&mut s);
    s.player_mut(P0).sakoku_turns = Some(sakoku_target() + 4);
    run(&mut s);
    assert!(goal_vp(&s, P0));
    let mut ev = Vec::new();
    crate::rules::actions::apply_actions(&mut s, P0, &[reopen()], &mut ev);
    let ev2 = run(&mut s);
    assert!(!goal_vp(&s, P0));
    assert!(
        ev2.iter()
            .any(|e| matches!(e, Event::FactionGoalLost { player: P0, .. }))
    );
    // Closing again starts from zero.
    close_now(&mut s);
    assert_eq!(s.player(P0).sakoku_turns, Some(0));
    run(&mut s);
    assert!(!goal_vp(&s, P0));
    assert_eq!(s.player(P0).sakoku_turns, Some(1));
}

#[test]
fn the_meter_counts_turns_then_reports_holding() {
    let mut s = world();
    let t = sakoku_target() as i32;
    let v = view(&s, P0).unwrap();
    assert_eq!((v.value, v.target, v.live, v.done), (0, t, true, false));
    assert_eq!(v.label, "Not closed yet · 0 of 30 turns");
    close_now(&mut s);
    s.player_mut(P0).sakoku_turns = Some(5);
    let v = view(&s, P0).unwrap();
    assert_eq!((v.value, v.label.as_str()), (5, "5 of 30 turns closed"));
    s.player_mut(P0).sakoku_turns = Some(t as u32);
    run(&mut s);
    let v = view(&s, P0).unwrap();
    assert_eq!((v.value, v.target, v.done), (t, t, true));
}

#[test]
fn the_action_view_lists_what_india_sees() {
    let mut s = world();
    let v = crate::rules::actions::view(&s, P0).sakoku.unwrap();
    assert!(!v.closed && v.blocker.is_none());
    assert!(crate::rules::actions::view(&s, P1).sakoku.is_none());
    close_now(&mut s);
    s.player_mut(P0).sakoku_turns = Some(sakoku_target());
    relate(&mut s, P0, P1, RelationState::War);
    let tile = india_tile(&s);
    spawn_unit(&mut s, P1, UnitKind::Spearman, tile);
    let v = crate::rules::actions::view(&s, P0).sakoku.unwrap();
    assert!(v.closed && v.holding);
    assert!(v.blocker.is_none(), "reopening is always possible");
}

#[test]
fn closing_through_the_turn_is_deterministic() {
    let s = world();
    let mut orders = std::collections::BTreeMap::new();
    orders.insert(P0, vec![close()]);
    let (a, ea) = crate::rules::resolve_turn(&s, &orders);
    let (b, eb) = crate::rules::resolve_turn(&s, &orders);
    assert_eq!(a, b);
    assert_eq!(ea, eb);
    assert_eq!(a.player(P0).sakoku_turns, Some(1));
}

#[test]
fn a_teammate_cannot_sign_an_alliance_on_the_turn_india_closes() {
    let mut s = test_game(&[
        (Faction::India, 0),
        (Faction::Switzerland, 0),
        (Faction::Byzantium, 1),
    ]);
    let (mate, rival) = (PlayerId(1), PlayerId(2));
    found(&mut s, P0, at(3, 3));
    found(&mut s, mate, at(3, 10));
    found(&mut s, rival, at(14, 14));
    relate(&mut s, P0, rival, RelationState::Peace);
    for p in [P0, mate, rival] {
        s.player_mut(p).gold = 1000;
    }
    let mut orders = std::collections::BTreeMap::new();
    orders.insert(
        rival,
        vec![Order::ProposeTreaty {
            to: s.team_of(P0),
            kind: TreatyKind::Alliance,
        }],
    );
    let (s, _) = crate::rules::resolve_turn(&s, &orders);
    let proposal = s.diplomacy.as_ref().unwrap().proposals[0].id;
    // The teammate accepts for the team while India closes the country, in one turn.
    let mut orders = std::collections::BTreeMap::new();
    orders.insert(
        mate,
        vec![Order::RespondToProposal {
            proposal,
            accept: true,
            explanation: None,
        }],
    );
    orders.insert(P0, vec![close()]);
    let (after, events) = crate::rules::resolve_turn(&s, &orders);
    assert!(is_closed(&after, P0));
    assert!(crate::diplomacy::allied_teams(&after, after.team_of(P0)).is_empty());
    assert!(events.iter().any(|e| matches!(
        e,
        Event::OrderFailed { reason, .. } if reason.contains("closed the country")
    )));
}

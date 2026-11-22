use legatum_ai::{Brain, DecisionParams, Knobs, ScriptedBrain};
use legatum_core::diplomacy::{ProposalId, TreatyKind};
use legatum_core::{
    game::{blank_game, spawn_unit},
    state::Map,
    *,
};
use std::collections::BTreeMap;

fn h(c: i32, r: i32) -> Hex {
    Hex::from_offset(c, r)
}
fn situation() -> GameState {
    let mut setup = GameSetup::all_ai(44);
    setup.width = 28;
    setup.height = 20;
    setup.players.truncate(3);
    // Authored before cities fired back; city strike has its own tests in core.
    setup.players[0].faction = Faction::Sparta;
    let mut s = blank_game(&setup, Map::new(28, 20, Terrain::Grassland));
    for (player, pos) in [(0, h(4, 6)), (1, h(11, 6)), (2, h(24, 15))] {
        let settler = spawn_unit(&mut s, PlayerId(player), UnitKind::Settler, pos);
        rules::cities::found_city(&mut s, settler, &mut vec![]).unwrap();
    }
    for pos in [h(5, 6), h(6, 6), h(5, 7), h(6, 7), h(7, 7)] {
        spawn_unit(&mut s, PlayerId(0), UnitKind::Pikeman, pos);
    }
    spawn_unit(&mut s, PlayerId(0), UnitKind::Scout, h(9, 6));
    visibility::update_explored(&mut s);
    // Treaty scoring is tested apart from first contact (core tests the contact rule).
    let teams = s.teams();
    s.diplomacy.as_mut().unwrap().meet_all(&teams);
    s
}
/// The situation's army stands two turns from the rival's empty last city, which an
/// opportunistic strike (`opportunity.rs`) rightly takes. Tests of the campaign score and of
/// treaty answers move it back out of strike reach (still within campaign range).
fn beyond_strike_reach(s: &mut GameState) {
    let spots = [h(1, 10), h(2, 10), h(1, 11), h(2, 11), h(3, 11)];
    let pikemen: Vec<UnitId> = s
        .units
        .values()
        .filter(|u| u.owner == PlayerId(0) && u.kind == UnitKind::Pikeman)
        .map(|u| u.id)
        .collect();
    for (id, pos) in pikemen.into_iter().zip(spots) {
        s.units.get_mut(&id).unwrap().pos = pos;
    }
    visibility::update_explored(s);
}
fn choices(view: &PlayerView) -> Vec<Order> {
    ScriptedBrain::new(Faction::Sparta, 99, PlayerId(0))
        .orders(view)
        .into_iter()
        .filter(diplomacy::is_diplomatic)
        .collect()
}

#[test]
fn campaign_ai_does_not_declare_during_a_saved_peace_truce() {
    let mut s = situation();
    s.turn = 204;
    let r = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(1)))
        .unwrap();
    r.state = legatum_core::diplomacy::RelationState::Peace;
    r.effective_turn = 202;
    r.guarantee_until = 202 + legatum_core::diplomacy::PEACE_TRUCE_TURNS;
    let view = player_view(&s, PlayerId(0));
    assert!(view.diplomacy.actions.iter().any(|a| {
        a.order
            == Order::DeclareWar {
                against: TeamId(1),
                breach: false,
            }
            && a.blocked_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("through turn 211"))
    }));
    assert!(
        !choices(&view)
            .iter()
            .any(|o| matches!(o, Order::DeclareWar { against, .. } if *against == TeamId(1)))
    );
}

/// The pre-8.8.1 plain-Hard Sparta policy (`sparta_search_only` = 1): no forced
/// campaign or follow-up targets, so peace is judged on strength and danger alone.
fn legacy_choices(view: &PlayerView) -> Vec<Order> {
    let mut brain = ScriptedBrain::new(Faction::Sparta, 99, PlayerId(0));
    brain.decisions.sparta_search_only = 1;
    brain
        .orders(view)
        .into_iter()
        .filter(diplomacy::is_diplomatic)
        .collect()
}

#[test]
fn identical_fogged_views_ignore_unseen_world_and_produce_identical_declarations() {
    let s = situation();
    let v = player_view(&s, PlayerId(0));
    assert!(!v.tile(h(24, 15)).unwrap().visible);
    let mut unseen = s.clone();
    unseen.player_mut(PlayerId(2)).science_stored = 777;
    let hidden_city = unseen.cities_of(PlayerId(2)).next().unwrap().id;
    unseen.cities.get_mut(&hidden_city).unwrap().hp = 1;
    unseen.map.get_mut(h(26, 18)).unwrap().terrain = Terrain::Mountains;
    let other = player_view(&unseen, PlayerId(0));
    assert_eq!(v, other);
    let expected = choices(&v);
    assert_eq!(
        expected,
        [Order::DeclareWar {
            against: TeamId(1),
            breach: false
        }]
    );
    assert_eq!(expected, choices(&other));
    assert_eq!(expected, choices(&v.clone()));
    let all = ScriptedBrain::new(Faction::Sparta, 99, PlayerId(0)).orders(&v);
    assert!(rules::validate_orders(&s, PlayerId(0), &all).is_empty());
}

#[test]
fn surprise_war_archer_shoots_on_the_declaration_turn() {
    let mut s = situation();
    s.player_mut(PlayerId(0)).gold = 2_000;
    let archer = spawn_unit(&mut s, PlayerId(0), UnitKind::Archer, h(9, 7));
    visibility::update_explored(&mut s);
    let target = h(11, 6);
    let view = player_view(&s, PlayerId(0));
    let mut brain = ScriptedBrain::new(Faction::Sparta, 99, PlayerId(0));
    let orders = brain.orders(&view);
    assert!(
        orders.contains(&Order::DeclareWar {
            against: TeamId(1),
            breach: false
        }),
        "{orders:?}"
    );
    assert!(
        orders.contains(&Order::RangedAttack {
            unit: archer,
            target
        }),
        "{orders:?}"
    );
    assert!(
        rules::validate_orders(&s, PlayerId(0), &orders).is_empty(),
        "{orders:?}"
    );
    let hp = s.cities_of(PlayerId(1)).next().unwrap().hp;
    let all = BTreeMap::from([(PlayerId(0), orders)]);
    let (after, _) = rules::resolve_turn(&s, &all);
    assert!(after.cities_of(PlayerId(1)).next().unwrap().hp < hp);

    let mut old = ScriptedBrain::new(Faction::Sparta, 99, PlayerId(0));
    old.decisions.declare_and_strike = 0;
    let control = old.orders(&view);
    assert!(
        !control
            .iter()
            .any(|o| matches!(o, Order::RangedAttack { unit, .. } if *unit == archer)),
        "{control:?}"
    );
}

#[test]
fn no_declaration_without_a_known_reachable_viable_campaign() {
    let mut s = situation();
    // Remove the forward scout and remembered foreign city; the public roster is insufficient.
    s.units
        .retain(|_, u| u.owner != PlayerId(0) || u.kind != UnitKind::Scout);
    s.player_mut(PlayerId(0)).explored = bitset::BitSet::new(s.map.len());
    s.player_mut(PlayerId(0)).tile_memory.fill(None);
    let v = player_view(&s, PlayerId(0));
    assert!(v.cities.iter().all(|c| c.owner == PlayerId(0)));
    assert!(choices(&v).is_empty());
}

#[test]
fn eliminated_opponent_does_not_block_a_new_offensive_war() {
    let mut s = situation();
    let old_war = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(2)))
        .unwrap();
    old_war.state = diplomacy::RelationState::War;
    assert!(choices(&player_view(&s, PlayerId(0))).is_empty());

    s.player_mut(PlayerId(2)).alive = false;
    s.cities.retain(|_, c| c.owner != PlayerId(2));
    s.units.retain(|_, u| u.owner != PlayerId(2));
    assert_eq!(
        choices(&player_view(&s, PlayerId(0))),
        [Order::DeclareWar {
            against: TeamId(1),
            breach: false
        }]
    );
}

#[test]
fn peace_acceptance_is_deterministic_from_public_strength_and_own_city_danger() {
    let s = situation();
    let (s, _) = resolve_turn(
        &s,
        &BTreeMap::from([(
            PlayerId(1),
            vec![Order::DeclareWar {
                against: TeamId(0),
                breach: false,
            }],
        )]),
    );
    let (mut s, _) = resolve_turn(
        &s,
        &BTreeMap::from([(
            PlayerId(1),
            vec![Order::ProposeTreaty {
                to: TeamId(0),
                kind: TreatyKind::Peace,
            }],
        )]),
    );
    let own = s.cities_of(PlayerId(0)).next().unwrap().id;
    s.cities.get_mut(&own).unwrap().hp = 50;
    // A weak city is in danger when the force beside it outweighs what holds it: its low HP
    // alone, against one Pikeman and our three, is not.
    for at in [h(3, 6), h(3, 7), h(4, 6), h(4, 7)] {
        spawn_unit(&mut s, PlayerId(1), UnitKind::Pikeman, at);
    }
    s.turn += 1;
    let mut v = player_view(&s, PlayerId(0));
    set_armies(&mut v, &[(PlayerId(0), 100), (PlayerId(1), 100)]);
    let expected = vec![Order::RespondToProposal {
        proposal: ProposalId(1),
        accept: true,
        explanation: None,
    }];
    assert_eq!(legacy_choices(&v), expected);
    assert_eq!(legacy_choices(&v.clone()), expected);
    let (s, _) = resolve_turn(&s, &BTreeMap::from([(PlayerId(0), expected)]));
    assert!(
        !legacy_choices(&player_view(&s, PlayerId(0)))
            .iter()
            .any(|o| matches!(
                o,
                Order::DeclareWar {
                    against: TeamId(1),
                    breach: false
                }
            ))
    );
}

fn war(s: &mut GameState, a: u8, b: u8, since: u32) {
    let r = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(a), TeamId(b)))
        .unwrap();
    r.state = diplomacy::RelationState::War;
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

fn set_economies(v: &mut PlayerView, values: &[(PlayerId, i32)]) {
    for p in v.players.iter_mut() {
        p.economy = values
            .iter()
            .find(|(id, _)| *id == p.id)
            .map_or(0, |(_, n)| *n);
    }
}

fn leader(v: &mut PlayerView, team: u8, vp: u32) {
    for standing in &mut v.standings {
        standing.vp = if standing.team == TeamId(team) { vp } else { 4 };
        standing.target = 10;
    }
}

fn declares(v: &PlayerView, team: u8) -> bool {
    choices(v).contains(&Order::DeclareWar {
        against: TeamId(team),
        breach: false,
    })
}

#[test]
fn nearby_land_or_coalition_value_can_justify_an_economic_war() {
    let mut s = situation();
    beyond_strike_reach(&mut s);
    s.player_mut(PlayerId(0)).faction = Faction::Switzerland;
    let mut v = player_view(&s, PlayerId(0));
    let test_brain = || {
        let mut brain = ScriptedBrain::with_knobs(
            Faction::Switzerland,
            99,
            PlayerId(0),
            Knobs::new(55, 40, 45, 80, 35),
        );
        brain.decisions = DecisionParams::default();
        brain
    };
    assert!(test_brain().orders(&v).contains(&Order::DeclareWar {
        against: TeamId(1),
        breach: false,
    }));
    leader(&mut v, 1, 8);
    let orders = test_brain().orders(&v);
    assert!(orders.contains(&Order::DeclareWar {
        against: TeamId(1),
        breach: false
    }));
    assert!(rules::validate_orders(&s, PlayerId(0), &orders).is_empty());
    // Even with a declaration queued, this turn's movements use published Peace.
    assert!(orders.iter().all(|o| match o {
        Order::MoveUnit { to, .. } => v.passage_allowed(
            v.me,
            legatum_core::UnitKind::Spearman,
            v.tile(*to).and_then(|t| t.owner)
        ),
        Order::RangedAttack { .. } => false,
        _ => true,
    }));
    let (next, events) = resolve_turn(&s, &BTreeMap::from([(PlayerId(0), orders)]));
    assert!(next.at_war(PlayerId(0), PlayerId(1)));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::WarDeclared { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Battle { .. } | Event::CityCaptured { .. }))
    );
}

#[test]
fn known_target_across_neutral_closed_borders_is_not_a_viable_war() {
    let mut v = player_view(&situation(), PlayerId(0));
    for row in 0..v.height {
        let i = v.index(h(8, row)).unwrap();
        v.tiles[i].owner = Some(PlayerId(2));
    }
    assert!(!declares(&v, 1));
}

#[test]
fn wounded_or_home_bound_armies_do_not_start_paper_campaigns() {
    let mut s = situation();
    beyond_strike_reach(&mut s);
    for u in s
        .units
        .values_mut()
        .filter(|u| u.owner == PlayerId(0) && u.kind != UnitKind::Scout)
    {
        // Sparta never rest-heal, so their march and recovery floor is 20 HP, not 60
        // (plain Hard plays the Sparta war policy since playtest 8.8.1).
        u.hp = 19;
    }
    assert!(!declares(&player_view(&s, PlayerId(0)), 1));
    let mut s = situation();
    beyond_strike_reach(&mut s);
    let keep = s
        .units
        .values()
        .find(|u| u.owner == PlayerId(0) && u.kind == UnitKind::Pikeman)
        .unwrap()
        .id;
    s.units
        .retain(|_, u| u.owner != PlayerId(0) || u.kind == UnitKind::Scout || u.id == keep);
    assert!(!declares(&player_view(&s, PlayerId(0)), 1));
}

#[test]
fn public_victory_threat_can_open_one_extra_front_but_unknown_leader_cannot() {
    let mut s = situation();
    s.turn = 50;
    war(&mut s, 0, 2, 1);
    let mut v = player_view(&s, PlayerId(0));
    assert!(!declares(&v, 1));
    leader(&mut v, 1, 8);
    assert!(declares(&v, 1));
    // The roster, army totals and VP cannot supply an unseen city's location.
    v.cities.retain(|c| c.owner != PlayerId(1));
    assert!(!declares(&v, 1));
}

#[test]
fn coalition_uses_team_progress_and_ignores_human_flag() {
    let mut s = situation();
    s.turn = 50;
    war(&mut s, 0, 2, 1);
    let mut v = player_view(&s, PlayerId(0));
    leader(&mut v, 1, 6);
    assert!(declares(&v, 1)); // 60%, a 20-point lead.
    v.standings
        .iter_mut()
        .find(|s| s.team == TeamId(0))
        .unwrap()
        .vp = 5;
    assert!(!declares(&v, 1)); // Only a 10-point lead.
    leader(&mut v, 1, 8);
    v.standings
        .iter_mut()
        .find(|s| s.team == TeamId(1))
        .unwrap()
        .target = 17;
    assert!(!declares(&v, 1)); // 8/17 is not 80%.
    leader(&mut v, 1, 8);
    v.players
        .iter_mut()
        .find(|p| p.id == PlayerId(1))
        .unwrap()
        .is_human = true;
    assert!(declares(&v, 1));
}

#[test]
fn only_already_fighting_teams_supply_coalition_strength() {
    let mut s = situation();
    beyond_strike_reach(&mut s);
    s.turn = 50;
    war(&mut s, 0, 2, 1);
    let mut v = player_view(&s, PlayerId(0));
    leader(&mut v, 1, 8);
    set_armies(
        &mut v,
        &[(PlayerId(0), 100), (PlayerId(1), 250), (PlayerId(2), 200)],
    );
    assert!(!declares(&v, 1));
    v.diplomacy
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(1), TeamId(2)))
        .unwrap()
        .state = diplomacy::RelationState::War;
    assert!(declares(&v, 1));
}

#[test]
fn declarations_respect_guarantee_and_legal_actions_without_hidden_rebuilding_lock() {
    let mut s = situation();
    s.turn = 100;
    let r = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(1)))
        .unwrap();
    r.effective_turn = 100;
    r.guarantee_until = 103;
    assert!(!declares(&player_view(&s, PlayerId(0)), 1));
    s.turn = 103;
    assert!(declares(&player_view(&s, PlayerId(0)), 1));
    s.turn = 124;
    let mut v = player_view(&s, PlayerId(0));
    assert!(declares(&v, 1));
    v.diplomacy
        .actions
        .iter_mut()
        .filter(|a| matches!(a.order, Order::DeclareWar { .. }))
        .for_each(|a| a.blocked_reason = Some("protected".into()));
    assert!(!declares(&v, 1));
}

#[test]
fn peace_uses_eight_turn_age_ratio_gate_and_five_turn_kind_cooldown() {
    let mut s = situation();
    war(&mut s, 0, 2, 1);
    let view = |s: &GameState| {
        let mut v = player_view(s, PlayerId(0));
        set_armies(&mut v, &[(PlayerId(0), 100), (PlayerId(2), 100)]);
        v
    };
    let offer = Order::ProposeTreaty {
        to: TeamId(2),
        kind: TreatyKind::Peace,
    };
    s.turn = 8;
    assert!(!choices(&view(&s)).contains(&offer));
    s.turn = 9;
    assert!(choices(&view(&s)).contains(&offer));
    s.diplomacy
        .as_mut()
        .unwrap()
        .peace_cooldowns
        .push(diplomacy::PeaceCooldown {
            from: TeamId(0),
            to: TeamId(2),
            retry_turn: 14,
            kind: TreatyKind::Peace,
        });
    assert!(!choices(&view(&s)).contains(&offer));
    s.turn = 14;
    assert!(choices(&view(&s)).contains(&offer));
}

#[test]
fn peace_ratio_is_assessed_separately_for_each_opponent() {
    let mut s = situation();
    s.turn = 50;
    war(&mut s, 0, 1, 1);
    war(&mut s, 0, 2, 1);
    let mut v = player_view(&s, PlayerId(0));
    set_armies(
        &mut v,
        &[(PlayerId(0), 100), (PlayerId(1), 50), (PlayerId(2), 100)],
    );
    assert_eq!(
        choices(&v),
        vec![Order::ProposeTreaty {
            to: TeamId(2),
            kind: TreatyKind::Peace
        }]
    );
}

#[test]
fn healthy_active_siege_does_not_sign_a_timed_truce() {
    let mut s = situation();
    s.turn = 100;
    war(&mut s, 0, 1, 1);
    let mut v = player_view(&s, PlayerId(0));
    let offer = Order::ProposeTreaty {
        to: TeamId(1),
        kind: TreatyKind::Peace,
    };
    assert!(!choices(&v).contains(&offer)); // A dominant army does not qualify for the 130% peace gate.
    v.cities
        .iter_mut()
        .find(|c| c.owner == PlayerId(1))
        .unwrap()
        .hp = 150;
    assert!(!choices(&v).contains(&offer));
}

#[test]
fn viable_campaign_can_declare_without_a_guaranteed_title_flip() {
    let mut s = situation();
    for pos in [h(4, 12), h(13, 14)] {
        let u = spawn_unit(&mut s, PlayerId(0), UnitKind::Settler, pos);
        rules::cities::found_city(&mut s, u, &mut vec![]).unwrap();
    }
    for pos in [h(5, 8), h(6, 8), h(7, 8), h(5, 9)] {
        spawn_unit(&mut s, PlayerId(0), UnitKind::Pikeman, pos);
    }
    let city = s.cities_of(PlayerId(1)).next().unwrap().id;
    s.cities.get_mut(&city).unwrap().population = 6;
    visibility::update_explored(&mut s);
    let mut v = player_view(&s, PlayerId(0));
    for standing in &mut v.standings {
        standing.vp = 0;
    }
    set_economies(
        &mut v,
        &[(PlayerId(0), 12), (PlayerId(1), 22), (PlayerId(2), 12)],
    );
    set_armies(
        &mut v,
        &[(PlayerId(0), 1000), (PlayerId(1), 100), (PlayerId(2), 100)],
    );
    let decisions = |v: &PlayerView| {
        ScriptedBrain::with_knobs(
            Faction::Sparta,
            99,
            PlayerId(0),
            legatum_ai::Knobs::new(0, 80, 35, 40, 85),
        )
        .orders(v)
    };
    assert!(decisions(&v).contains(&Order::DeclareWar {
        against: TeamId(1),
        breach: false
    }));
    set_economies(
        &mut v,
        &[(PlayerId(0), 12), (PlayerId(1), 22), (PlayerId(2), 18)],
    );
    assert!(
        decisions(&v)
            .iter()
            .any(|o| matches!(o, Order::DeclareWar { .. }))
    );
}

#[test]
fn one_declaration_per_turn_even_with_multiple_weak_neighbors() {
    let mut s = situation();
    let city = s.cities_of(PlayerId(2)).next().unwrap().id;
    s.cities.remove(&city);
    let u = spawn_unit(&mut s, PlayerId(2), UnitKind::Settler, h(8, 12));
    rules::cities::found_city(&mut s, u, &mut vec![]).unwrap();
    spawn_unit(&mut s, PlayerId(0), UnitKind::Scout, h(8, 10));
    visibility::update_explored(&mut s);
    let v = player_view(&s, PlayerId(0));
    assert_eq!(
        choices(&v)
            .iter()
            .filter(|o| matches!(o, Order::DeclareWar { .. }))
            .count(),
        1
    );
}

#[test]
fn coalition_extra_front_has_no_hidden_recent_declaration_cooldown() {
    let mut s = situation();
    s.turn = 50;
    war(&mut s, 0, 2, 49);
    s.diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(2)))
        .unwrap()
        .declarers = vec![TeamId(0)];
    let mut v = player_view(&s, PlayerId(0));
    leader(&mut v, 1, 8);
    assert!(declares(&v, 1));
    v.turn = 57;
    assert!(declares(&v, 1));
}

fn rich(s: &mut GameState) {
    for p in &mut s.players {
        p.gold = 500;
    }
}
/// The Sparta army's upkeep would otherwise trip the treasury gate, tested on its own.
fn solvent(v: &mut PlayerView) {
    v.economy.gold_per_turn.total = 5;
    v.economy.bankruptcy = None;
}
fn alliance_view() -> PlayerView {
    let mut s = situation();
    beyond_strike_reach(&mut s);
    s.turn = 30;
    rich(&mut s);
    let (s, _) = resolve_turn(
        &s,
        &BTreeMap::from([(
            PlayerId(1),
            vec![Order::ProposeTreaty {
                to: TeamId(0),
                kind: TreatyKind::Alliance,
            }],
        )]),
    );
    let mut v = player_view(&s, PlayerId(0));
    solvent(&mut v);
    // A distant proposer: no army to keep off us and no war to forego, so the score is the
    // shared interests against the Gold.
    v.cities
        .iter_mut()
        .find(|c| c.owner == PlayerId(1))
        .unwrap()
        .pos = h(27, 1);
    v
}
fn response(v: &PlayerView) -> Order {
    choices(v)
        .into_iter()
        .find(|o| matches!(o, Order::RespondToProposal { .. }))
        .unwrap()
}
fn accepts(v: &PlayerView) -> bool {
    matches!(response(v), Order::RespondToProposal { accept: true, .. })
}

#[test]
fn alliances_need_a_shared_enemy_or_coalition_and_price_upkeep() {
    let mut v = alliance_view();
    assert!(
        !accepts(&v),
        "no shared enemy, no coalition: the upkeep isn't worth it"
    );
    leader(&mut v, 2, 8);
    let expected = response(&v);
    assert!(
        matches!(&expected, Order::RespondToProposal { accept: true, explanation: Some(e), .. }
            if e.score.is_some_and(|s| s >= 0)
                && e.reasons.iter().any(|r| r.contains("same coalition target"))
                && e.reasons.iter().any(|r| r.contains("Gold/turn upkeep"))),
        "{expected:?}"
    );
    // Humans and AIs get the same score.
    v.players[1].is_human = true;
    assert_eq!(expected, response(&v));
    let mut ahead = v.clone();
    ahead
        .standings
        .iter_mut()
        .find(|s| s.team == TeamId(1))
        .unwrap()
        .vp = 6;
    assert!(
        matches!(response(&ahead), Order::RespondToProposal { accept: false, explanation: Some(e), .. }
            if e.reasons.iter().any(|r| r.contains("60%")))
    );
    let mut betrayer = v.clone();
    betrayer.diplomacy.betrayals.push(diplomacy::Betrayal {
        by: TeamId(1),
        against: TeamId(0),
        turn: 25,
        until: 45,
    });
    assert!(!accepts(&betrayer));
    // The treasury must cover 20 turns of any deficit the upkeep leaves.
    let mut poor = v.clone();
    poor.economy.gold = 50;
    poor.economy.gold_per_turn.total = -1;
    assert!(accepts(&poor));
    poor.economy.gold_per_turn.total = -2;
    assert!(!accepts(&poor));
    poor.economy.gold_per_turn.total = 5;
    poor.economy.gold = 0;
    assert!(accepts(&poor));
}

fn allied(protected_until: u32, coalition_target: Option<TeamId>) -> GameState {
    let mut s = situation();
    s.turn = 30;
    rich(&mut s);
    let d = s.diplomacy.as_mut().unwrap();
    let r = d
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(1)))
        .unwrap();
    r.state = diplomacy::RelationState::Alliance;
    r.security = Some(diplomacy::SecurityTreaty {
        id: 1,
        started_turn: 20,
        protected_until,
        coalition_target,
    });
    s
}
fn leaves(v: &PlayerView) -> bool {
    choices(v).iter().any(|o| {
        matches!(
            o,
            Order::BreakTreaty {
                with: TeamId(1),
                ..
            }
        )
    })
}

#[test]
fn emergency_betrayal_requires_ninety_percent_and_honest_exits_wait_for_the_window() {
    let emergency = Order::DeclareWar {
        against: TeamId(1),
        breach: true,
    };
    // Inside the protected window, leaving costs Gold: an ally near victory is not enough.
    let s = allied(40, None);
    let mut v = player_view(&s, PlayerId(0));
    leader(&mut v, 1, 8);
    assert!(!choices(&v).contains(&emergency));
    assert!(!leaves(&v));
    leader(&mut v, 1, 9);
    let os = choices(&v);
    assert!(os.contains(&emergency), "{os:?}");
    assert!(validate_orders(&s, PlayerId(0), &os).is_empty());
    // After the window, leaving is free and honest.
    let s = allied(30, None);
    let mut v = player_view(&s, PlayerId(0));
    leader(&mut v, 1, 8);
    assert!(leaves(&v));
    assert!(!choices(&v).contains(&emergency));
}

#[test]
fn coalition_ai_offers_alliances_and_leaves_when_the_target_falls_or_the_ally_leads() {
    let mut v = alliance_view();
    leader(&mut v, 2, 8);
    v.diplomacy.proposals.clear();
    v.diplomacy.actions.retain(|a| {
        matches!(
            a.order,
            Order::ProposeTreaty {
                to: TeamId(1),
                kind: TreatyKind::Alliance
            }
        )
    });
    for a in &mut v.diplomacy.actions {
        a.blocked_reason = None;
    }
    assert!(choices(&v).iter().any(|o| matches!(
        o,
        Order::ProposeTreaty {
            kind: TreatyKind::Alliance,
            to: TeamId(1)
        }
    )));
    let s = allied(25, Some(TeamId(2)));
    let v = player_view(&s, PlayerId(0));
    assert!(leaves(&v), "the coalition target is not ahead");
    let mut eliminated = v.clone();
    eliminated
        .players
        .iter_mut()
        .filter(|p| p.team == TeamId(2))
        .for_each(|p| p.alive = false);
    eliminated
        .standings
        .iter_mut()
        .find(|s| s.team == TeamId(2))
        .unwrap()
        .vp = 9;
    assert!(
        leaves(&eliminated),
        "An eliminated leader's permanent VP must not preserve yesterday's coalition"
    );
    let mut ally_leads = v.clone();
    ally_leads
        .standings
        .iter_mut()
        .find(|s| s.team == TeamId(1))
        .unwrap()
        .vp = 8;
    assert!(leaves(&ally_leads));
    assert!(
        !choices(&ally_leads)
            .iter()
            .any(|o| matches!(o, Order::DeclareWar { breach: true, .. }))
    );
}

#[test]
fn an_alliance_that_joins_a_war_on_the_visible_leader_needs_a_reachable_target() {
    use diplomacy::RelationState;
    let mut s = situation();
    s.turn = 30;
    rich(&mut s);
    // Team 2 is at war with team 1 and offers team 0 an Alliance: signing joins that war.
    let war = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(2), TeamId(1)))
        .unwrap();
    war.state = RelationState::War;
    war.war_id = Some(1);
    war.declarers = vec![TeamId(1)];
    let (s, _) = resolve_turn(
        &s,
        &BTreeMap::from([(
            PlayerId(2),
            vec![Order::ProposeTreaty {
                to: TeamId(0),
                kind: TreatyKind::Alliance,
            }],
        )]),
    );
    let mut v = player_view(&s, PlayerId(0));
    v.standings
        .iter_mut()
        .find(|s| s.team == TeamId(1))
        .unwrap()
        .vp = 8;
    let action = v
        .diplomacy
        .actions
        .iter()
        .find(|a| matches!(a.order, Order::RespondToProposal { accept: true, .. }))
        .unwrap();
    assert_eq!(
        action.new_wars,
        [diplomacy::NewWar {
            team: TeamId(0),
            enemy: TeamId(1)
        }],
        "the offer says which war it joins"
    );
    assert!(accepts(&v));
    let mut hidden = v.clone();
    hidden.cities.retain(|c| c.owner != PlayerId(1));
    assert!(!accepts(&hidden));
    assert_eq!(response(&v), response(&v.clone()));
    let mut outmatched = v.clone();
    set_armies(&mut outmatched, &[(PlayerId(1), 10000)]);
    assert!(!accepts(&outmatched));
    let s = resolve_turn(&s, &BTreeMap::from([(PlayerId(0), vec![response(&v)])])).0;
    assert!(s.teams_at_war(TeamId(0), TeamId(1)));
    assert!(s.allied(PlayerId(0), PlayerId(2)));
    assert!(
        !choices(&player_view(&s, PlayerId(0)))
            .iter()
            .any(|o| matches!(
                o,
                Order::ProposeTreaty {
                    to: TeamId(1),
                    kind: TreatyKind::Peace
                }
            ))
    );
}

#[test]
fn an_alliance_into_a_war_nobody_wants_is_refused() {
    use diplomacy::RelationState;
    let mut s = situation();
    s.turn = 30;
    rich(&mut s);
    // Team 2 fights team 1, which is no leader: joining is a cost with nothing to show for it.
    let war = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(2), TeamId(1)))
        .unwrap();
    war.state = RelationState::War;
    war.war_id = Some(1);
    war.declarers = vec![TeamId(2)];
    let (s, _) = resolve_turn(
        &s,
        &BTreeMap::from([(
            PlayerId(2),
            vec![Order::ProposeTreaty {
                to: TeamId(0),
                kind: TreatyKind::Alliance,
            }],
        )]),
    );
    let mut v = player_view(&s, PlayerId(0));
    for standing in &mut v.standings {
        standing.vp = 2;
    }
    assert!(!accepts(&v));
}

#[test]
fn an_alliance_offer_to_a_peaceful_empire_says_it_would_join_our_war() {
    use diplomacy::RelationState;
    let mut s = situation();
    beyond_strike_reach(&mut s);
    s.turn = 30;
    rich(&mut s);
    // Team 0 fights team 1; team 2 is at peace with everyone and strong.
    let war = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(1)))
        .unwrap();
    war.state = RelationState::War;
    war.war_id = Some(1);
    war.declarers = vec![TeamId(1)];
    let v = player_view(&s, PlayerId(0));
    let offer = Order::ProposeTreaty {
        to: TeamId(2),
        kind: TreatyKind::Alliance,
    };
    let action = v
        .diplomacy
        .actions
        .iter()
        .find(|a| a.order == offer)
        .unwrap();
    assert_eq!(
        action.new_wars,
        [diplomacy::NewWar {
            team: TeamId(2),
            enemy: TeamId(1)
        }],
        "the ally would join our war"
    );
}

#[test]
fn joined_coalition_wars_offer_peace_after_the_old_target_falls_behind() {
    let mut s = situation();
    beyond_strike_reach(&mut s);
    s.turn = 30;
    let d = s.diplomacy.as_mut().unwrap();
    let r = d
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(1)))
        .unwrap();
    r.state = diplomacy::RelationState::War;
    r.war_id = Some(42);
    r.effective_turn = 20;
    // Team 2, our ally, fought team 1 first and brought us into the same war.
    r.declarers = vec![TeamId(2)];
    for (a, b, state) in [
        (0, 2, diplomacy::RelationState::Alliance),
        (1, 2, diplomacy::RelationState::War),
    ] {
        let rel = d
            .relations
            .iter_mut()
            .find(|r| r.between(TeamId(a), TeamId(b)))
            .unwrap();
        rel.state = state;
        if state == diplomacy::RelationState::War {
            rel.war_id = Some(42);
            rel.declarers = vec![TeamId(2)];
            rel.effective_turn = 18;
        } else {
            rel.security = Some(diplomacy::SecurityTreaty {
                id: 1,
                started_turn: 19,
                protected_until: 29,
                coalition_target: Some(TeamId(1)),
            });
        }
    }
    let mut v = player_view(&s, PlayerId(0));
    for standing in &mut v.standings {
        standing.vp = if standing.team == TeamId(2) { 8 } else { 3 };
    }
    let peace = |v: &PlayerView| {
        choices(v).iter().any(|o| {
            matches!(
                o,
                Order::ProposeTreaty {
                    to: TeamId(1),
                    kind: TreatyKind::Peace
                }
            )
        })
    };
    assert!(
        peace(&v),
        "A coalition intervention must allow the old leader to recover"
    );
    let mut still_leading = v.clone();
    still_leading
        .standings
        .iter_mut()
        .find(|s| s.team == TeamId(1))
        .unwrap()
        .vp = 9;
    assert!(!peace(&still_leading));
    // A war of its own, not the ally's, is not disengaged from on that ground.
    for r in &mut v.diplomacy.relations {
        if r.between(TeamId(0), TeamId(1)) {
            r.war_id = Some(41);
            r.declarers = vec![TeamId(0)];
        }
    }
    assert!(!peace(&v), "Only a war an ally brought us into is let go");
}

use legatum_core::diplomacy::{DiplomacyState, ProposalId, RelationState, TreatyKind};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::{City, Map};
use legatum_core::*;

fn h(c: i32, r: i32) -> Hex {
    Hex::from_offset(c, r)
}
fn game(teams: &[u8]) -> GameState {
    let mut setup = GameSetup::all_ai(19);
    // These tests isolate treaty rules from unit upkeep in their empty economies.
    setup.width = 16;
    setup.height = 12;
    setup.players.truncate(teams.len());
    for (p, team) in setup.players.iter_mut().zip(teams) {
        p.team = TeamId(*team);
    }
    let mut s = blank_game(&setup, Map::new(16, 12, Terrain::Grassland));
    // Independent settlers keep all parties alive, so tests isolate diplomacy from conquest.
    for i in 0..teams.len() {
        spawn_unit(
            &mut s,
            PlayerId(i as u8),
            UnitKind::Settler,
            h(i as i32 * 3, 11),
        );
    }
    // These tests isolate treaty rules from first contact (tested on its own below).
    let teams = s.teams();
    s.diplomacy.as_mut().unwrap().meet_all(&teams);
    s
}
fn war(to: u8) -> Order {
    Order::DeclareWar {
        against: TeamId(to),
        breach: false,
    }
}
fn offer(to: u8) -> Order {
    Order::ProposeTreaty {
        to: TeamId(to),
        kind: TreatyKind::Peace,
    }
}
fn response(id: u32, accept: bool) -> Order {
    Order::RespondToProposal {
        proposal: ProposalId(id),
        accept,
        explanation: None,
    }
}
fn step(s: &GameState, orders: &[(u8, Vec<Order>)]) -> (GameState, Vec<Event>) {
    resolve_turn(
        s,
        &orders
            .iter()
            .map(|(p, os)| (PlayerId(*p), os.clone()))
            .collect(),
    )
}
fn establish_war(s: &GameState) -> GameState {
    step(s, &[(0, vec![war(1)])]).0
}
fn relation(s: &GameState) -> &legatum_core::diplomacy::Relation {
    s.diplomacy
        .as_ref()
        .unwrap()
        .relation(TeamId(0), TeamId(1))
        .unwrap()
}
fn city(s: &mut GameState, p: u8, at: Hex) -> CityId {
    let id = s.alloc_city_id();
    s.cities.insert(
        id,
        City {
            last_attacked_turn: None,
            id,
            owner: PlayerId(p),
            founder: PlayerId(p),
            name: "Border city".into(),
            pos: at,
            population: 2,
            food_stored: 0,
            production_stored: 0,
            producing: None,
            bought: None,
            production_queue: Vec::new(),
            build_automation: None,
            last_auto_pick: None,
            auto_queued: None,
            production_banked: Vec::new(),
            production_queue_notified: Default::default(),
            wonder_lost: None,
            last_purchase_turn: None,
            buildings: Default::default(),
            hp: data::CITY_HP,
            grown_tiles: Vec::new(),
            tiles_claimed: 0,
            is_capital: false,
            original_capital: false,
            greece_captive_paid: Default::default(),
            occupied_from: None,
            founded_turn: 1,
            resistance: 0,
        },
    );
    s.map.get_mut(at).unwrap().owner = Some(PlayerId(p));
    s.map.get_mut(at).unwrap().city = Some(id);
    id
}
#[test]
fn a_declaration_at_plain_peace_is_a_surprise_war_that_attacks_this_turn() {
    // G20 (Civ 6 surprise war): with no Alliance, War starts at the start of this
    // turn's resolution, so the attack ordered with it resolves in the same turn.
    let mut s = game(&[0, 1]);
    let a = spawn_unit(&mut s, PlayerId(0), UnitKind::Soldier, h(4, 4));
    let b = spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, h(5, 4));
    assert!(!s.at_war(PlayerId(0), PlayerId(1)));
    let view = player_view(&s, PlayerId(0));
    assert!(forecast_attack(&view, a, h(5, 4)).is_none());
    let forecast = view
        .diplomacy
        .actions
        .iter()
        .find(|a| a.order == war(1))
        .unwrap();
    assert_eq!(
        forecast.effective_turn, 1,
        "War starts in the turn it is declared"
    );
    assert_eq!(forecast.blocked_reason, None);
    let (next, events) = step(
        &s,
        &[(
            0,
            vec![
                war(1),
                Order::MoveUnit {
                    unit: a,
                    to: h(5, 4),
                },
            ],
        )],
    );
    assert!(next.units[&b].hp < 100);
    assert!(events.iter().any(|e| matches!(e, Event::Battle { .. })));
    assert_eq!(next.turn, 2);
    assert!(next.at_war(PlayerId(0), PlayerId(1)));
    assert_eq!(
        events
            .iter()
            .filter(
                |e| matches!(e, Event::WarDeclared { relation } if relation.effective_turn == 1)
            )
            .count(),
        1,
        "declared once, effective this turn"
    );
    assert_eq!(relation(&next).state, RelationState::War);
}

#[test]
fn declaration_wakes_sleepers_for_an_existing_peaceful_army_only_once() {
    for kind in [UnitKind::Settler, UnitKind::Soldier] {
        for declarer in [0, 1] {
            let mut s = game(&[0, 1]);
            // This test exercises diplomacy wakeups with ordinary movement range.
            s.player_mut(PlayerId(0)).faction = Faction::Switzerland;
            let sleeper = spawn_unit(&mut s, PlayerId(0), kind, h(6, 4));
            let enemy = spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, h(8, 4));
            let arrival = spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, h(6, 8));
            let sleep = vec![(
                0,
                vec![Order::SleepUnit {
                    unit: sleeper,
                    until_healed: false,
                }],
            )];
            let wake = Event::UnitAwakened {
                unit: sleeper,
                owner: PlayerId(0),
                unit_kind: kind,
                at: h(6, 4),
            };
            let (s, events) = step(&s, &sleep);
            assert_eq!(s.units[&sleeper].standing_order, Some(StandingOrder::Sleep));
            assert!(!events.contains(&wake));
            assert!(
                player_view(&s, PlayerId(0))
                    .units
                    .iter()
                    .any(|u| u.id == enemy)
            );
            let s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();

            let (mut s, events) = step(&s, &[(declarer, vec![war(1 - declarer)])]);
            assert!(s.at_war(PlayerId(0), PlayerId(1)));
            assert_eq!(s.units[&sleeper].standing_order, None);
            assert!(s.units[&sleeper].needs_orders);
            assert_eq!(events.iter().filter(|e| **e == wake).count(), 1);

            // Re-sleeping accepts this army, including after a save/load boundary.
            for orders in [&sleep[..], &[]] {
                let (next, events) = step(&s, orders);
                assert_eq!(
                    next.units[&sleeper].standing_order,
                    Some(StandingOrder::Sleep)
                );
                assert!(!next.units[&sleeper].needs_orders);
                assert!(!events.contains(&wake));
                s = serde_json::from_str(&serde_json::to_string(&next).unwrap()).unwrap();
            }
            let (s, events) = step(
                &s,
                &[(
                    1,
                    vec![Order::MoveUnit {
                        unit: arrival,
                        to: h(6, 6),
                    }],
                )],
            );
            assert_eq!(s.units[&sleeper].standing_order, None);
            assert!(s.units[&sleeper].needs_orders);
            assert_eq!(events.iter().filter(|e| **e == wake).count(), 1);
        }
    }
}

#[test]
fn simultaneous_declarations_have_one_war_id_and_no_defender() {
    let s = game(&[0, 1]);
    let (s, events) = step(&s, &[(1, vec![war(0)]), (0, vec![war(1)])]);
    assert_eq!(relation(&s).declarers, [TeamId(0), TeamId(1)]);
    assert_eq!(relation(&s).war_id, Some(1));
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::WarDeclared { .. }))
            .count(),
        1
    );
    assert!(
        validate_orders(&s, PlayerId(0), &[war(1)])[0]
            .message
            .contains("Already")
    );
}

#[test]
fn offers_have_three_response_turns_and_peace_has_ten_guaranteed_resolutions() {
    let s = establish_war(&game(&[0, 1])); // P2
    let (s, _) = step(&s, &[(0, vec![offer(1)])]); // P3, reply through P5
    let p = &s.diplomacy.as_ref().unwrap().proposals[0];
    assert_eq!((p.created_turn, p.respond_by), (3, 5));
    let (s, _) = step(&s, &[]); // P4
    let (s, _) = step(&s, &[]); // P5: last legal response
    assert!(validate_orders(&s, PlayerId(1), &[response(1, true)]).is_empty());
    let (s, events) = step(&s, &[(1, vec![response(1, true)])]); // P6 peace
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::PeaceEstablished { .. }))
    );
    assert_eq!(relation(&s).guarantee_until, 16);
    assert!(!validate_orders(&s, PlayerId(0), &[war(1)]).is_empty()); // R6 protected
    assert_eq!(
        player_view(&s, PlayerId(0))
            .diplomacy
            .actions
            .iter()
            .find(|a| a.order == war(1))
            .unwrap()
            .effective_turn,
        16
    );
    let (s, _) = step(&s, &[]); // P7, R7 protected
    assert!(!validate_orders(&s, PlayerId(0), &[war(1)]).is_empty());
    let mut s = step(&s, &[]).0; // P8
    while s.turn < 16 {
        assert!(!validate_orders(&s, PlayerId(0), &[war(1)]).is_empty());
        s = step(&s, &[]).0;
    }
    // A declaration cannot be queued on the last protected planning turn.
    assert!(validate_orders(&s, PlayerId(0), &[war(1)]).is_empty());
    assert!(
        step(&s, &[(0, vec![war(1)])])
            .0
            .at_war(PlayerId(0), PlayerId(1))
    );
}

#[test]
fn athens_sparta_save_sequence_cannot_restart_war_at_turns_199_or_204() {
    // Extracted from peace-flipflop-athens-20261025-0915.legatum: Athens declared
    // on 193, offered Peace on 194, and Sparta accepted on 195 (Peace effective
    // 196, old guarantee_until 199). Athens declared on 199, offered again on
    // 200, Sparta accepted on 201 (Peace effective 202, until 205), and Athens
    // queued War on 204 for the 205 boundary. Chronicle turns label resolution;
    // Relation.effective_turn labels when the new state begins.
    let mut s = game(&[0, 3]);
    s.players[0].faction = Faction::Sparta;
    s.players[1].faction = Faction::Athens;
    s.turn = 196;
    let r = s.diplomacy.as_mut().unwrap().relations.get_mut(0).unwrap();
    r.state = RelationState::Peace;
    r.effective_turn = 196;
    r.guarantee_until = 199;
    s.turn = 199;
    assert!(validate_orders(&s, PlayerId(1), &[war(0)]).is_empty());
    s.diplomacy.as_mut().unwrap().relations[0].guarantee_until =
        196 + legatum_core::diplomacy::PEACE_TRUCE_TURNS;
    assert!(
        !validate_orders(&s, PlayerId(1), &[war(0)]).is_empty(),
        "a new treaty would still protect turn 199"
    );

    s.turn = 202;
    let r = s.diplomacy.as_mut().unwrap().relations.get_mut(0).unwrap();
    r.effective_turn = 202;
    r.guarantee_until = 205;
    s.turn = 204;
    assert!(!validate_orders(&s, PlayerId(1), &[war(0)]).is_empty());
    s.turn = 205;
    assert!(validate_orders(&s, PlayerId(1), &[war(0)]).is_empty());
    let r = s.diplomacy.as_mut().unwrap().relations.get_mut(0).unwrap();
    r.guarantee_until = 202 + legatum_core::diplomacy::PEACE_TRUCE_TURNS;
    for turn in [202, 204, 205, 210, 211] {
        s.turn = turn;
        let errors = validate_orders(&s, PlayerId(1), &[war(0)]);
        assert!(!errors.is_empty(), "turn {turn} must be protected");
        assert!(errors[0].message.contains("Peace with Sparta"));
        assert!(errors[0].message.contains("through turn 211"));
    }
    assert!(!legatum_core::diplomacy::surprise_war(
        s.diplomacy
            .as_ref()
            .unwrap()
            .relation(TeamId(0), TeamId(3))
            .unwrap(),
        s.turn
    ));
    s.turn = 212;
    assert!(validate_orders(&s, PlayerId(1), &[war(0)]).is_empty());
    assert!(legatum_core::diplomacy::surprise_war(
        s.diplomacy
            .as_ref()
            .unwrap()
            .relation(TeamId(0), TeamId(3))
            .unwrap(),
        s.turn
    ));
}

#[test]
fn unanswered_offers_expire_only_after_last_response_and_rejection_is_reported() {
    let s = establish_war(&game(&[0, 1]));
    let (s, _) = step(&s, &[(0, vec![offer(1)])]);
    let (rejected, events) = step(&s, &[(1, vec![response(1, false)])]);
    assert!(rejected.diplomacy.unwrap().proposals.is_empty());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ProposalRejected { .. }))
    );
    let (s, _) = step(&s, &[]);
    let (s, _) = step(&s, &[]);
    assert_eq!(s.diplomacy.as_ref().unwrap().proposals.len(), 1);
    let (s, events) = step(&s, &[]);
    assert!(s.diplomacy.as_ref().unwrap().proposals.is_empty());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ProposalExpired { .. }))
    );
}

#[test]
fn teammate_lower_id_wins_and_all_teammates_see_the_superseded_order() {
    let s = establish_war(&game(&[0, 0, 1]));
    let (s, events) = step(&s, &[(1, vec![offer(1)]), (0, vec![offer(1)])]);
    assert_eq!(
        s.diplomacy.as_ref().unwrap().proposals[0].proposer,
        PlayerId(0)
    );
    assert!(events.iter().any(|e| matches!(e, Event::OrderSuperseded { decision } if decision.player == PlayerId(1) && decision.winner == PlayerId(0))));
    assert_eq!(
        player_view(&s, PlayerId(0)).diplomacy.superseded,
        player_view(&s, PlayerId(1)).diplomacy.superseded
    );
    assert!(player_view(&s, PlayerId(2)).diplomacy.superseded.is_empty());
    let s = step(&s, &[(2, vec![response(1, true)])]).0;
    assert!(!s.at_war(PlayerId(1), PlayerId(2)));
    assert!(!validate_orders(&s, PlayerId(0), &[war(0)]).is_empty());
}

#[test]
fn competing_proposals_and_withdrawal_before_acceptance_are_deterministic() {
    let s = establish_war(&game(&[0, 1]));
    let (s, events) = step(&s, &[(1, vec![offer(0)]), (0, vec![offer(1)])]);
    assert_eq!(s.diplomacy.as_ref().unwrap().proposals.len(), 1);
    assert_eq!(s.diplomacy.as_ref().unwrap().proposals[0].from, TeamId(0));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::OrderFailed {
            player: PlayerId(1),
            ..
        }
    )));
    let (s, events) = step(
        &s,
        &[
            (
                0,
                vec![Order::WithdrawProposal {
                    proposal: ProposalId(1),
                }],
            ),
            (1, vec![response(1, true)]),
        ],
    );
    assert_eq!(relation(&s).state, RelationState::War);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ProposalWithdrawn { .. }))
    );
    assert!(events.iter().any(|e| matches!(
        e,
        Event::OrderFailed {
            player: PlayerId(1),
            ..
        }
    )));
}

#[test]
fn accepting_peace_does_not_cancel_the_current_city_assault() {
    let mut s = establish_war(&game(&[0, 1]));
    let c = city(&mut s, 1, h(6, 4));
    // P1 keeps its own original capital elsewhere, so the capitals victory does not end the game
    // with the assault.
    let home = city(&mut s, 1, h(13, 9));
    s.cities.get_mut(&home).unwrap().original_capital = true;
    let a = spawn_unit(&mut s, PlayerId(0), UnitKind::Soldier, h(5, 4));
    let (mut s, _) = step(&s, &[(0, vec![offer(1)])]);
    s.cities.get_mut(&c).unwrap().hp = 1;
    let (s, events) = step(
        &s,
        &[
            (
                0,
                vec![Order::MoveUnit {
                    unit: a,
                    to: h(6, 4),
                }],
            ),
            (1, vec![response(1, true)]),
        ],
    );
    assert_eq!(s.cities[&c].owner, PlayerId(0));
    assert_eq!(relation(&s).state, RelationState::Peace);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CityCaptured { .. }))
    );
}

#[test]
fn three_way_neutral_claimant_is_undamaged_and_blocks_occupation() {
    let mut s = establish_war(&game(&[0, 1, 2]));
    let target = h(6, 5);
    let positions = target.neighbors();
    let mut ids = Vec::new();
    for i in 0..3 {
        ids.push(spawn_unit(
            &mut s,
            PlayerId(i),
            UnitKind::Soldier,
            positions[i as usize],
        ));
    }
    let orders: Vec<_> = ids
        .iter()
        .enumerate()
        .map(|(p, id)| {
            (
                p as u8,
                vec![Order::MoveUnit {
                    unit: *id,
                    to: target,
                }],
            )
        })
        .collect();
    let (s, events) = step(&s, &orders);
    assert_eq!(s.units[&ids[2]].hp, 100);
    assert!(!s.units.values().any(|u| u.pos == target));
    let battles: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::Battle { .. }))
        .collect();
    assert_eq!(battles.len(), 1);
    assert!(matches!(
        battles[0],
        Event::Battle {
            attacker_owner: PlayerId(0),
            defender: events::Defender::Unit {
                owner: PlayerId(1),
                ..
            },
            ..
        }
    ));
}

#[test]
fn peaceful_claimants_never_fight_the_stronger_enters_and_neutral_occupants_block() {
    // Units of teams at Peace claiming one tile: the stronger moves in, the other stays put,
    // and nobody fights or captures (Sam, 2026-11-22). Equal strength is drawn by seed.
    for civilian in [false, true] {
        let mut s = game(&[0, 1]);
        let a = spawn_unit(&mut s, PlayerId(0), UnitKind::Soldier, h(4, 4));
        let b = spawn_unit(
            &mut s,
            PlayerId(1),
            if civilian {
                UnitKind::Settler
            } else {
                UnitKind::Soldier
            },
            h(6, 4),
        );
        let (next, events) = step(
            &s,
            &[
                (
                    0,
                    vec![Order::MoveUnit {
                        unit: a,
                        to: h(5, 4),
                    }],
                ),
                (
                    1,
                    vec![Order::MoveUnit {
                        unit: b,
                        to: h(5, 4),
                    }],
                ),
            ],
        );
        let entered: Vec<_> = [a, b]
            .into_iter()
            .filter(|u| next.units[u].pos == h(5, 4))
            .collect();
        assert_eq!(entered.len(), 1, "exactly one enters");
        if civilian {
            assert_eq!(entered, [a], "the Spearman is stronger than the Settler");
            assert_eq!(next.units[&b].owner, PlayerId(1), "never captured");
        }
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Battle { .. } | Event::SettlerCaptured { .. }))
        );
        s.units.get_mut(&b).unwrap().pos = h(5, 4);
        let next = step(
            &s,
            &[(
                0,
                vec![Order::MoveUnit {
                    unit: a,
                    to: h(5, 4),
                }],
            )],
        )
        .0;
        assert_eq!(next.units[&a].pos, s.units[&a].pos);
    }
}

#[test]
fn closed_borders_require_progress_to_exit_and_forbid_healing_fortifying_or_attacking() {
    let mut s = game(&[0, 1, 2]);
    // Byzantium may peacefully camp foreign land, and Celts land causes attrition.
    // This fixture isolates closed-border egress from both faction rules.
    s.player_mut(PlayerId(0)).faction = Faction::Switzerland;
    s.player_mut(PlayerId(1)).faction = Faction::Switzerland;
    let a = spawn_unit(&mut s, PlayerId(0), UnitKind::Ranger, h(6, 5));
    for tile in h(6, 5).within(2) {
        s.map.get_mut(tile).unwrap().owner = Some(PlayerId(1));
    }
    s.units.get_mut(&a).unwrap().hp = 60;
    s.units.get_mut(&a).unwrap().fortify = 6;
    let view = player_view(&s, PlayerId(0));
    let healing = view
        .units
        .iter()
        .find(|u| u.id == a)
        .unwrap()
        .resting_heal
        .as_ref()
        .unwrap();
    assert_eq!(healing.total, 0);
    assert_eq!(healing.parts, vec![("Closed foreign territory".into(), 0)]);
    let (s, _) = step(&s, &[(0, vec![Order::SkipUnit { unit: a }])]);
    assert_eq!((s.units[&a].hp, s.units[&a].fortify), (60, 0));
    let toward_exit = h(6, 5).neighbors()[0];
    let (s, _) = step(
        &s,
        &[(
            0,
            vec![Order::MoveUnit {
                unit: a,
                to: toward_exit,
            }],
        )],
    );
    assert_eq!(s.units[&a].pos, toward_exit);
    let (next, _) = step(
        &s,
        &[(
            0,
            vec![Order::MoveUnit {
                unit: a,
                to: h(6, 5),
            }],
        )],
    );
    assert_eq!(next.units[&a].pos, toward_exit, "cannot move deeper");
    let mut s = s;
    spawn_unit(
        &mut s,
        PlayerId(2),
        UnitKind::Soldier,
        toward_exit.neighbors()[0],
    );
    let (s, _) = step(&s, &[(0, vec![war(2)])]);
    let order = Order::RangedAttack {
        unit: a,
        target: toward_exit.neighbors()[0],
    };
    assert!(!validate_orders(&s, PlayerId(0), &[order]).is_empty());
    assert!(
        forecast_attack(&player_view(&s, PlayerId(0)), a, toward_exit.neighbors()[0]).is_none()
    );
}

#[test]
fn explicit_attack_forecasts_reject_closed_territory_like_validation() {
    let (mut s, _) = step(&game(&[0, 1, 2, 0]), &[(0, vec![war(2)])]);
    let origin = h(6, 5);
    let target = h(7, 5);
    let ranger = spawn_unit(&mut s, PlayerId(0), UnitKind::Ranger, origin);
    spawn_unit(&mut s, PlayerId(2), UnitKind::Soldier, target);
    let shot = Order::RangedAttack {
        unit: ranger,
        target,
    };
    for owner in [
        Some(PlayerId(1)),
        None,
        Some(PlayerId(0)),
        Some(PlayerId(2)),
        Some(PlayerId(3)),
    ] {
        s.map.get_mut(origin).unwrap().owner = owner;
        let view = player_view(&s, PlayerId(0));
        let closed = owner == Some(PlayerId(1));
        for mode in [AttackMode::Melee, AttackMode::Ranged] {
            assert_eq!(
                forecast_attack_mode(&view, ranger, target, mode).is_none(),
                closed
            );
        }
        assert_eq!(forecast_attack(&view, ranger, target).is_none(), closed);
        let errors = validate_orders(&s, PlayerId(0), std::slice::from_ref(&shot));
        if closed {
            assert_eq!(
                errors,
                vec![OrderError {
                    index: 0,
                    message: "Cannot attack from closed foreign territory".into()
                }]
            );
        } else {
            assert!(errors.is_empty(), "{errors:?}");
        }
    }
}

#[test]
fn neutral_units_do_not_block_city_healing() {
    let mut s = game(&[0, 1]);
    let c = city(&mut s, 0, h(5, 5));
    s.cities.get_mut(&c).unwrap().hp = 100;
    spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, h(6, 5));
    let (s, _) = step(&s, &[]);
    assert!(s.cities[&c].hp > 100);
}

#[test]
fn save_load_continuation_preserves_pending_offers_versions_timers_and_events() {
    let s = establish_war(&game(&[0, 1]));
    let (s, _) = step(&s, &[(0, vec![offer(1)])]);
    let mut live = s;
    let mut restored: GameState =
        serde_json::from_str(&serde_json::to_string(&live).unwrap()).unwrap();
    let mut orders_by_turn = vec![vec![(1, vec![response(1, true)])]];
    orders_by_turn.extend((0..legatum_core::diplomacy::PEACE_TRUCE_TURNS).map(|_| vec![]));
    orders_by_turn.push(vec![(0, vec![war(1)])]);
    for orders in orders_by_turn {
        let (next, events) = step(&live, &orders);
        let (loaded_next, loaded_events) = step(&restored, &orders);
        assert_eq!(next.hash(), loaded_next.hash());
        assert_eq!(events, loaded_events);
        live = next;
        restored = serde_json::from_str(&serde_json::to_string(&loaded_next).unwrap()).unwrap();
    }
    assert!(live.at_war(PlayerId(0), PlayerId(1)));
}

#[test]
fn proposals_and_superseded_orders_are_private_but_declarations_are_public() {
    let s = establish_war(&game(&[0, 1, 2]));
    let (after, events) = step(&s, &[(0, vec![offer(1)])]);
    assert!(
        player_view(&after, PlayerId(2))
            .diplomacy
            .proposals
            .is_empty()
    );
    for p in 0..3 {
        let before_view = player_view(&s, PlayerId(p));
        let after_view = player_view(&after, PlayerId(p));
        let visible = event_visibility::visible_events(&events, &before_view, &after_view);
        assert_eq!(!visible.is_empty(), p != 2);
    }
    assert_eq!(
        validate_orders(&after, PlayerId(2), &[response(1, true)]),
        validate_orders(&after, PlayerId(2), &[response(999, true)])
    );
}

#[test]
fn legacy_saves_keep_all_war_and_new_games_start_peace() {
    let s = game(&[0, 1]);
    let mut json = serde_json::to_value(&s).unwrap();
    json.as_object_mut().unwrap().remove("diplomacy");
    let legacy: GameState = serde_json::from_value(json).unwrap();
    assert!(legacy.at_war(PlayerId(0), PlayerId(1)));
    assert!(legacy.diplomacy.is_none());
    assert!(player_view(&legacy, PlayerId(0)).diplomacy.legacy);
    assert!(!s.at_war(PlayerId(0), PlayerId(1)));
    let mut fresh = DiplomacyState::new([TeamId(0), TeamId(1)].into_iter());
    fresh.meet(TeamId(0), TeamId(1)); // game() starts every pair in contact.
    assert_eq!(s.diplomacy, Some(fresh));
}

#[test]
fn eliminated_team_cannot_accept_and_scoring_precedes_diplomacy() {
    let s = establish_war(&game(&[0, 1, 2]));
    let (mut s, _) = step(&s, &[(0, vec![offer(1)])]);
    // Last settler disbands in the same resolution as acceptance.
    let id = s.units_of(PlayerId(1)).next().unwrap().id;
    let (after, events) = step(
        &s,
        &[(1, vec![response(1, true), Order::DisbandUnit { unit: id }])],
    );
    assert!(after.diplomacy.as_ref().unwrap().proposals.is_empty());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::PeaceEstablished { .. }))
    );
    s.setup.turn_limit = Some(s.turn);
    let (after, events) = step(&s, &[(1, vec![response(1, true)])]);
    assert!(after.finished);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::PeaceEstablished { .. }))
    );
}

#[test]
fn pending_team_diplomacy_never_exposes_other_teams_or_military_orders() {
    let s = game(&[0, 0, 1]);
    let mut v = player_view(&s, PlayerId(0));
    let orders = [
        (
            PlayerId(1),
            vec![war(1), Order::SkipUnit { unit: UnitId(2) }],
        ),
        (PlayerId(2), vec![war(0)]),
    ];
    diplomacy::with_pending(&mut v, orders.iter().map(|(p, os)| (*p, os.as_slice())));
    assert_eq!(v.diplomacy.queued.len(), 1);
    assert_eq!(v.diplomacy.queued[0].player, PlayerId(1));
    assert_eq!(v.diplomacy.queued[0].order, war(1));
}

#[test]
fn stale_proposal_is_invalidated_instead_of_accepted() {
    let s = establish_war(&game(&[0, 1]));
    let (mut s, _) = step(&s, &[(0, vec![offer(1)])]);
    s.diplomacy.as_mut().unwrap().relations[0].version += 1;
    let (s, events) = step(&s, &[(1, vec![response(1, true)])]);
    assert!(s.diplomacy.as_ref().unwrap().proposals.is_empty());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ProposalInvalidated { .. }))
    );
    assert_eq!(relation(&s).state, RelationState::War);
}

fn rich(mut s: GameState) -> GameState {
    for p in &mut s.players {
        p.gold = 500;
    }
    s
}
fn leave(to: u8) -> Order {
    Order::BreakTreaty {
        with: TeamId(to),
        agreement: legatum_core::diplomacy::Agreement::Security,
    }
}
fn betray(to: u8) -> Order {
    Order::DeclareWar {
        against: TeamId(to),
        breach: true,
    }
}
fn ally_offer(to: u8) -> Order {
    Order::ProposeTreaty {
        to: TeamId(to),
        kind: TreatyKind::Alliance,
    }
}
fn alliance(s: &GameState, from: u8, to: u8) -> GameState {
    let (s, _) = step(s, &[(from, vec![ally_offer(to)])]);
    let id = s
        .diplomacy
        .as_ref()
        .unwrap()
        .proposals
        .iter()
        .find(|p| p.from == TeamId(from) && p.to == TeamId(to))
        .unwrap()
        .id
        .0;
    step(&s, &[(to, vec![response(id, true)])]).0
}
fn alliance_relation(s: &GameState, a: u8, b: u8) -> &legatum_core::diplomacy::Relation {
    s.diplomacy
        .as_ref()
        .unwrap()
        .relation(TeamId(a), TeamId(b))
        .unwrap()
}

#[test]
fn alliance_activates_with_empty_treasuries_and_no_payment() {
    let s = game(&[0, 0, 1]);
    assert!(s.players.iter().all(|p| p.gold == 0));
    assert!(validate_orders(&s, PlayerId(0), &[ally_offer(1)]).is_empty());
    let (s, _) = step(&s, &[(0, vec![ally_offer(1)])]);
    let id = s.diplomacy.as_ref().unwrap().proposals[0].id.0;
    assert!(validate_orders(&s, PlayerId(2), &[response(id, true)]).is_empty());
    let (s, events) = step(&s, &[(2, vec![response(id, true)])]);
    assert_eq!(alliance_relation(&s, 0, 1).state, RelationState::Alliance);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TreatyActivated { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::TreatyPayment { .. }))
    );
}

#[test]
fn former_ally_stays_in_closed_land_then_can_attack_after_declaration() {
    let mut s = alliance(&rich(game(&[0, 1, 2])), 0, 1);
    let turn = s.turn;
    s.diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(1)))
        .unwrap()
        .security
        .as_mut()
        .unwrap()
        .protected_until = turn;
    for tile in h(6, 5).within(2) {
        s.map.get_mut(tile).unwrap().owner = Some(PlayerId(1));
    }
    let ranger = spawn_unit(&mut s, PlayerId(0), UnitKind::Ranger, h(6, 5));
    let target = spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, h(7, 5));
    let (s, _) = step(&s, &[(0, vec![leave(1)])]);
    assert_eq!(alliance_relation(&s, 0, 1).state, RelationState::Peace);
    assert_eq!(s.units[&ranger].pos, h(6, 5));
    assert!(!s.unit_in_open_land(&s.units[&ranger]));
    let shot = Order::RangedAttack {
        unit: ranger,
        target: h(7, 5),
    };
    assert!(!validate_orders(&s, PlayerId(0), std::slice::from_ref(&shot)).is_empty());
    let (s, _) = step(&s, &[(0, vec![war(1)])]);
    assert_eq!(s.units[&ranger].pos, h(6, 5));
    assert!(s.unit_in_open_land(&s.units[&ranger]));
    assert!(s.units.contains_key(&target));
    assert!(validate_orders(&s, PlayerId(0), &[shot]).is_empty());
}

#[test]
fn alliance_upkeep_is_a_gold_cost_in_income_and_bankruptcy_forecasts() {
    let s = alliance(&rich(game(&[0, 1, 2])), 0, 1);
    let income = economy::player_income(&s, PlayerId(0));
    let upkeep = income
        .gold
        .parts
        .iter()
        .find(|(label, _)| label.starts_with("Treaty upkeep: Alliance with"))
        .unwrap();
    assert_eq!(upkeep.1, -TreatyKind::Alliance.upkeep());
    assert_eq!(economy::upkeep(&s, PlayerId(0)), 1);
    assert_eq!(economy::upkeep(&s, PlayerId(2)), 0);
    let mut grown = s.clone();
    city(&mut grown, 0, h(4, 4));
    city(&mut grown, 0, h(8, 4));
    let larger_income = economy::player_income(&grown, PlayerId(0));
    assert!(
        larger_income.gold.parts.iter().any(|(label, amount)| label
            .starts_with("Treaty upkeep: Alliance with")
            && *amount == -2)
    );
    assert_eq!(economy::upkeep(&grown, PlayerId(0)), 2);
    let quote = player_view(&grown, PlayerId(0))
        .diplomacy
        .actions
        .clone()
        .into_iter()
        .find(|a| {
            matches!(
                a.order,
                Order::ProposeTreaty {
                    to: TeamId(2),
                    kind: TreatyKind::Alliance
                }
            )
        })
        .unwrap();
    assert_eq!(
        quote.upkeep, 2,
        "the offer quotes this empire's current city count"
    );
    let gold = s.player(PlayerId(0)).gold;
    let (mut s, _) = step(&s, &[]);
    assert_eq!(s.player(PlayerId(0)).gold, gold - 1);
    // A treasury that the upkeep would tip below zero is forecast as bankrupt.
    s.player_mut(PlayerId(0)).gold = 0;
    let forecast = player_view(&s, PlayerId(0))
        .economy
        .bankruptcy
        .clone()
        .unwrap();
    assert!(
        forecast
            .gold
            .parts
            .iter()
            .any(|(label, n)| label.starts_with("Treaty upkeep") && *n == -1)
    );
}

#[test]
fn alliance_upkeep_is_waived_while_at_war_with_a_team_close_to_winning() {
    let s = alliance(&rich(game(&[0, 1, 2])), 0, 1);
    // Team 0 goes to War with team 2; nobody is close to winning yet, so the Alliance pays.
    let (mut s, _) = step(&s, &[(0, vec![war(2)])]);
    assert!(s.teams_at_war(TeamId(0), TeamId(2)));
    let target = scoring::team_target(&s, TeamId(2));
    assert!(target > scoring::NEAR_VICTORY_VP);
    assert!(!scoring::near_victory(&s, TeamId(2)));
    assert_eq!(economy::upkeep(&s, PlayerId(0)), 1);
    assert_eq!(diplomacy::upkeep_waived_by(&s, TeamId(0), TeamId(1)), None);
    // Within 2 VP of its target: the Alliance pays nothing, for both allies, and the Gold
    // breakdown shows the waiver as its own line.
    s.setup.rule_experiments.vp_target = Some(scoring::NEAR_VICTORY_VP as u8);
    assert!(scoring::near_victory(&s, TeamId(2)));
    assert_eq!(
        diplomacy::upkeep_waived_by(&s, TeamId(0), TeamId(1)),
        Some(TeamId(2))
    );
    assert_eq!(economy::upkeep(&s, PlayerId(0)), 0);
    assert_eq!(economy::upkeep(&s, PlayerId(1)), 0);
    let gold = economy::player_income(&s, PlayerId(0)).gold;
    assert!(gold.parts.iter().any(|(label, n)| *n == 1
        && label.starts_with("Treaty upkeep: Alliance with")
        && label.contains("free while at War with")));
    let before = s.player(PlayerId(0)).gold;
    let (next, _) = step(&s, &[]);
    let (control, _) = {
        let mut paying = s.clone();
        paying.setup.rule_experiments.vp_target = None;
        step(&paying, &[])
    };
    assert_eq!(
        next.player(PlayerId(0)).gold - before,
        control.player(PlayerId(0)).gold - before + 1,
        "the waived Gold stays in the treasury"
    );
    // On a victory hold counts too, whatever the VP.
    s.setup.rule_experiments.vp_target = None;
    assert!(!scoring::near_victory(&s, TeamId(2)));
    s.vp_holds.insert(TeamId(2), 1);
    assert_eq!(economy::upkeep(&s, PlayerId(0)), 0);
    // A near-winner the Alliance is not at War with waives nothing.
    s.vp_holds.clear();
    s.vp_holds.insert(TeamId(1), 1);
    assert_eq!(economy::upkeep(&s, PlayerId(0)), 1);
}

#[test]
fn leaving_inside_the_window_costs_gold_and_is_a_visible_betrayal() {
    use legatum_core::diplomacy::BETRAYAL_TURNS;
    let s = alliance(&rich(game(&[0, 1])), 0, 1);
    let breach = TreatyKind::Alliance.breach_cost();
    let view = player_view(&s, PlayerId(0));
    let action = view
        .diplomacy
        .actions
        .iter()
        .find(|a| a.order == leave(1))
        .unwrap();
    assert_eq!((action.cost, action.betrayal), (breach, true));
    assert_eq!(action.effective_turn, s.turn + 1);
    let gold = s.player(PlayerId(0)).gold;
    let (s, events) = step(&s, &[(0, vec![leave(1)])]);
    assert_eq!(alliance_relation(&s, 0, 1).state, RelationState::Peace);
    assert_eq!(
        s.player(PlayerId(0)).gold,
        gold - breach - 1,
        "breach and upkeep"
    );
    let betrayal = s.diplomacy.as_ref().unwrap().betrayals[0].clone();
    assert_eq!(
        (betrayal.by, betrayal.against, betrayal.until),
        (TeamId(0), TeamId(1), s.turn + BETRAYAL_TURNS)
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::AllianceBetrayed { .. }))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TreatyBreached { cost, .. } if *cost == breach))
    );
    // Everyone sees the record; the betrayer can't offer an Alliance until it expires.
    for p in 0..2 {
        assert_eq!(
            player_view(&s, PlayerId(p)).diplomacy.betrayals,
            vec![betrayal.clone()]
        );
    }
    let message = validate_orders(&s, PlayerId(0), &[ally_offer(1)])[0]
        .message
        .clone();
    assert!(
        message.contains(&format!("until turn {}", betrayal.until)),
        "{message}"
    );
    assert!(
        validate_orders(&s, PlayerId(1), &[ally_offer(0)]).is_empty(),
        "the betrayed side may still forgive"
    );
    let mut s = s;
    while s.turn < betrayal.until - 1 {
        s = step(&s, &[]).0;
    }
    assert!(validate_orders(&s, PlayerId(0), &[ally_offer(1)]).is_empty());
    assert!(player_view(&s, PlayerId(1)).diplomacy.betrayals.is_empty());
    let s = step(&s, &[]).0;
    assert!(s.diplomacy.as_ref().unwrap().betrayals.is_empty());
}

#[test]
fn leaving_after_the_window_is_free_takes_effect_next_turn_and_is_no_betrayal() {
    let mut s = alliance(&rich(game(&[0, 1])), 0, 1);
    let until = alliance_relation(&s, 0, 1)
        .security
        .as_ref()
        .unwrap()
        .protected_until;
    while s.turn + 1 < until {
        s = step(&s, &[]).0;
    }
    let gold = s.player(PlayerId(1)).gold;
    let (s, events) = step(&s, &[(1, vec![leave(0)])]);
    assert_eq!(alliance_relation(&s, 0, 1).state, RelationState::Peace);
    assert_eq!(
        s.player(PlayerId(1)).gold,
        gold - 1,
        "only this turn's upkeep"
    );
    assert!(s.diplomacy.as_ref().unwrap().betrayals.is_empty());
    assert!(events.iter().any(|e| matches!(e,
        Event::TreatyEnded { reason, .. } if reason.ends_with("left the Alliance"))));
    // At plain Peace a declaration is a surprise war again.
    let (s, _) = step(&s, &[(0, vec![war(1)])]);
    assert!(s.teams_at_war(TeamId(0), TeamId(1)));
}

#[test]
fn war_on_an_ally_needs_an_explicit_betrayal_and_starts_next_turn() {
    let mut s = alliance(&rich(game(&[0, 1])), 0, 1);
    while alliance_relation(&s, 0, 1)
        .security
        .as_ref()
        .unwrap()
        .protected_until
        > s.turn
    {
        s = step(&s, &[]).0;
    }
    assert!(
        validate_orders(&s, PlayerId(0), &[war(1)])[0]
            .message
            .contains("leave the Alliance first")
    );
    let gold = s.player(PlayerId(0)).gold;
    let (s, events) = step(&s, &[(0, vec![betray(1)])]);
    assert!(s.teams_at_war(TeamId(0), TeamId(1)));
    assert_eq!(relation(&s).effective_turn, s.turn, "effective next turn");
    // Outside the window it costs no Gold but is still a betrayal.
    assert_eq!(s.player(PlayerId(0)).gold, gold - 1);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::AllianceBetrayed { .. }))
    );
}

#[test]
fn simultaneous_wars_on_allies_charge_and_record_both_betrayers() {
    let s = alliance(&rich(game(&[0, 1])), 0, 1);
    let cost = TreatyKind::Alliance.breach_cost();
    let gold = [s.player(PlayerId(0)).gold, s.player(PlayerId(1)).gold];
    assert!(validate_orders(&s, PlayerId(0), &[betray(1)]).is_empty());
    assert!(validate_orders(&s, PlayerId(1), &[betray(0)]).is_empty());
    let (next, events) = step(&s, &[(0, vec![betray(1)]), (1, vec![betray(0)])]);
    assert!(at_war(&next, 0, 1));
    for p in 0..2 {
        assert_eq!(next.player(PlayerId(p)).gold, gold[p as usize] - cost - 1);
    }
    let d = next.diplomacy.as_ref().unwrap();
    assert_eq!(d.betrayals.len(), 2);
    for (a, b) in [(TeamId(0), TeamId(1)), (TeamId(1), TeamId(0))] {
        assert!(
            d.betrayals
                .iter()
                .any(|record| record.by == a && record.against == b)
        );
    }
    assert_eq!(relation(&next).declarers, [TeamId(0), TeamId(1)]);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::TreatyBreached { .. }))
            .count(),
        2
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::AllianceBetrayed { .. }))
            .count(),
        2
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::TreatyEnded { .. }))
            .count(),
        1
    );
}

#[test]
fn allies_may_enter_each_others_territory_but_not_attack() {
    let mut s = rich(game(&[0, 1, 2]));
    for p in &mut s.players {
        p.faction = Faction::Switzerland;
    }
    city(&mut s, 1, h(8, 5));
    assert!(!s.passage_allowed(
        PlayerId(0),
        legatum_core::UnitKind::Soldier,
        Some(PlayerId(1))
    ));
    let s = alliance(&s, 0, 1);
    assert!(s.passage_allowed(
        PlayerId(0),
        legatum_core::UnitKind::Soldier,
        Some(PlayerId(1))
    ));
    assert!(s.passage_allowed(
        PlayerId(1),
        legatum_core::UnitKind::Soldier,
        Some(PlayerId(0))
    ));
    assert!(!s.passage_allowed(
        PlayerId(2),
        legatum_core::UnitKind::Soldier,
        Some(PlayerId(1))
    ));
    assert!(!s.at_war(PlayerId(0), PlayerId(1)));
    assert!(s.shares_vision(PlayerId(0), PlayerId(1)));
}

#[test]
fn newly_allied_sight_is_remembered_when_its_source_disbands_next_turn() {
    let mut s = rich(game(&[0, 1, 2]));
    let scout = spawn_unit(&mut s, PlayerId(1), UnitKind::Scout, h(12, 2));
    visibility::update_explored(&mut s);
    let native = scoring::explored_land_tiles(&s, PlayerId(0));
    assert!(
        !player_view(&s, PlayerId(0))
            .tile(h(12, 2))
            .unwrap()
            .explored
    );
    let s = alliance(&s, 0, 1);
    let borrowed: Vec<_> = player_view(&s, PlayerId(0))
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, tile)| tile.visible)
        .map(|(i, _)| i)
        .collect();
    assert!(player_view(&s, PlayerId(0)).tile(h(12, 2)).unwrap().visible);
    let s: GameState = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    let (s, _) = step(&s, &[(1, vec![Order::DisbandUnit { unit: scout }])]);
    let view = player_view(&s, PlayerId(0));
    assert!(!view.tile(h(12, 2)).unwrap().visible);
    for i in borrowed {
        assert!(
            view.tiles[i].explored,
            "forgot tile {i} shared on activation"
        );
        assert_eq!(view.tiles[i].terrain, Some(Terrain::Grassland));
    }
    assert_eq!(scoring::explored_land_tiles(&s, PlayerId(0)), native);
    assert!(
        !s.player(PlayerId(0))
            .explored
            .get(s.map.index(h(12, 2)).unwrap())
    );
}

#[test]
fn shared_sight_is_one_hop_observations_survive_exit_and_byzantium_earns_only_native_exploration() {
    let mut setup = GameSetup::all_ai(123);
    setup.width = 60;
    setup.height = 14;
    setup.players.truncate(3);
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.team = TeamId(i as u8);
    }
    let mut s = blank_game(&setup, Map::new(60, 14, Terrain::Grassland));
    for i in 0..3 {
        spawn_unit(
            &mut s,
            PlayerId(i),
            UnitKind::Settler,
            h(2 + i as i32 * 20, 2),
        );
    }
    let mut s = rich(s);
    visibility::update_explored(&mut s);
    // Twenty hexes apart: these empires meet by fixture, not by sight.
    s.diplomacy
        .as_mut()
        .unwrap()
        .meet_all(&[TeamId(0), TeamId(1), TeamId(2)]);
    let native = scoring::explored_land_tiles(&s, PlayerId(0));
    let s = alliance(&alliance(&s, 0, 1), 1, 2);
    let a = player_view(&s, PlayerId(0));
    assert!(a.tile(h(22, 2)).unwrap().visible);
    assert!(!a.tile(h(42, 2)).unwrap().visible);
    // Every treasury is public (rules 31), allied or not.
    assert_eq!(a.players[1].gold, s.player(PlayerId(1)).gold);
    assert_eq!(scoring::explored_land_tiles(&s, PlayerId(0)), native);
    let s = step(&s, &[]).0; // Save borrowed observations without claiming native exploration.
    let s: GameState = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    let s = step(&s, &[(0, vec![leave(1)])]).0;
    let a = player_view(&s, PlayerId(0));
    assert!(!a.tile(h(22, 2)).unwrap().visible);
    assert!(a.tile(h(22, 2)).unwrap().explored);
    assert!(!a.units.iter().any(|u| u.owner == PlayerId(1)));
    assert_eq!(scoring::explored_land_tiles(&s, PlayerId(0)), native);
}

fn at_war(s: &GameState, a: u8, b: u8) -> bool {
    s.teams_at_war(TeamId(a), TeamId(b))
}
fn joined_events(events: &[Event]) -> Vec<(u8, u8, u8)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::WarJoined { relation, ally, .. } => Some((relation.a.0, relation.b.0, ally.0)),
            _ => None,
        })
        .collect()
}
fn proposal_to(s: &GameState, to: u8) -> legatum_core::diplomacy::PeaceProposal {
    s.diplomacy
        .as_ref()
        .unwrap()
        .proposals
        .iter()
        .find(|p| p.to == TeamId(to))
        .cloned()
        .expect("a proposal to that team")
}
fn blocked(s: &GameState, player: u8, order: &Order) -> Option<String> {
    player_view(s, PlayerId(player))
        .diplomacy
        .actions
        .iter()
        .find(|a| a.order == *order)
        .and_then(|a| a.blocked_reason.clone())
}

#[test]
fn an_alliance_with_an_enemy_or_an_enemys_ally_is_refused_with_a_reason() {
    // 1 is allied with 2, so 0's war on 2 is a war on 1 too: no Alliance with either.
    let s = alliance(&rich(game(&[0, 1, 2])), 1, 2);
    let s = step(&s, &[(0, vec![war(2)])]).0;
    assert!(at_war(&s, 0, 1) && at_war(&s, 0, 2));
    for to in [1, 2] {
        let reason = blocked(&s, 0, &ally_offer(to)).expect("blocked");
        assert!(reason.contains("at war"), "{reason}");
        assert!(!validate_orders(&s, PlayerId(0), &[ally_offer(to)]).is_empty());
    }
}

#[test]
fn declaring_war_on_a_team_tied_to_you_by_another_alliance_is_refused() {
    // 0 and 2 are both allied with 1: they are one side, so 0 cannot declare on 2.
    let s = alliance(&alliance(&rich(game(&[0, 1, 2])), 0, 1), 1, 2);
    let errors = validate_orders(&s, PlayerId(0), &[war(2)]);
    assert!(errors[0].message.contains("another Alliance"), "{errors:?}");
    // A war on a direct ally whose Alliance is the only link is the betrayal, as before.
    let s = alliance(&rich(game(&[0, 1])), 0, 1);
    assert!(validate_orders(&s, PlayerId(0), &[betray(1)]).is_empty());
}

#[test]
fn surprise_war_cannot_pull_defensive_ally_in_on_last_truce_turn() {
    let mut s = alliance(&rich(game(&[0, 1, 2])), 1, 2);
    let turn = s.turn;
    let r = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(1)))
        .unwrap();
    r.guarantee_until = turn + 1;
    // The direct declaration on 2 is a surprise war now. The 0-1 treaty protects
    // this resolution even though a declaration for the next boundary would be legal.
    let reason = blocked(&s, 0, &war(2)).expect("defensive join must be blocked");
    assert!(reason.contains("guaranteed through turn"), "{reason}");
    let (next, events) = step(&s, &[(0, vec![war(2)])]);
    assert!(!next.at_war(PlayerId(0), PlayerId(1)));
    assert!(!next.at_war(PlayerId(0), PlayerId(2)));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::WarDeclared { .. }))
    );
}

#[test]
fn alliance_offer_cannot_join_a_war_on_the_last_truce_turn() {
    let mut s = rich(game(&[0, 1, 2]));
    s = step(&s, &[(2, vec![war(0)])]).0;
    let turn = s.turn;
    s.diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(1), TeamId(2)))
        .unwrap()
        .guarantee_until = turn + 1;
    let reason = blocked(&s, 1, &ally_offer(0)).expect("the truce blocks this Alliance");
    assert!(reason.contains("guaranteed through turn"), "{reason}");
    assert!(!validate_orders(&s, PlayerId(1), &[ally_offer(0)]).is_empty());
}

#[test]
fn alliance_acceptance_at_boundary_cannot_pull_a_truced_ally_into_war() {
    let mut s = alliance(&rich(game(&[0, 1, 2])), 0, 2);
    let turn = s.turn;
    s.diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(1), TeamId(2)))
        .unwrap()
        .guarantee_until = turn + 10;
    let (s, _) = step(&s, &[(0, vec![ally_offer(1)])]);
    let offer = proposal_to(&s, 1);
    let (next, events) = step(
        &s,
        &[(1, vec![response(offer.id.0, true)]), (2, vec![betray(0)])],
    );
    assert!(!next.allied(PlayerId(0), PlayerId(1)));
    assert!(at_war(&next, 0, 2));
    assert!(!at_war(&next, 1, 2));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ProposalInvalidated { reason, .. }
        if reason.contains("guaranteed through turn")))
    );
}

#[test]
fn signing_and_leaving_an_alliance_cannot_erase_a_peace_truce() {
    let s = establish_war(&rich(game(&[0, 1])));
    let s = step(&s, &[(0, vec![offer(1)])]).0;
    let peace = step(&s, &[(1, vec![response(1, true)])]).0;
    let until = relation(&peace).guarantee_until;
    let allied = alliance(&peace, 0, 1);
    assert_eq!(relation(&allied).guarantee_until, until);
    assert!(!validate_orders(&allied, PlayerId(0), &[betray(1)]).is_empty());
    let left = step(&allied, &[(0, vec![leave(1)])]).0;
    assert_eq!(relation(&left).guarantee_until, until);
    assert!(!validate_orders(&left, PlayerId(0), &[war(1)]).is_empty());
}

#[test]
fn alliances_need_first_contact_and_sight_records_it_for_both_sides() {
    let mut s = game(&[0, 1]);
    s.diplomacy.as_mut().unwrap().contacts = Some(Default::default());
    let mut s = rich(s);
    let alliance = ally_offer(1);
    let errors = validate_orders(&s, PlayerId(0), std::slice::from_ref(&alliance));
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, diplomacy::NOT_MET);
    // Peace and War don't need contact.
    assert!(validate_orders(&s, PlayerId(0), &[war(1)]).is_empty());
    let view = player_view(&s, PlayerId(0));
    assert!(view.diplomacy.met.is_empty());
    assert!(
        view.diplomacy
            .actions
            .iter()
            .any(|a| a.order == alliance && a.blocked_reason.as_deref() == Some(diplomacy::NOT_MET))
    );
    // An offer made before contact can't be accepted either (e.g. one created by a fixture).
    let mut offered = s.clone();
    offered
        .diplomacy
        .as_mut()
        .unwrap()
        .meet(TeamId(0), TeamId(1));
    let (mut offered, _) = step(&offered, &[(0, vec![alliance.clone()])]);
    offered.diplomacy.as_mut().unwrap().contacts = Some(Default::default());
    let errors = validate_orders(&offered, PlayerId(1), &[response(1, true)]);
    assert_eq!(errors[0].message, diplomacy::NOT_MET);
    assert!(validate_orders(&offered, PlayerId(1), &[response(1, false)]).is_empty());
    // A unit of team 1 coming into team 0's sight is contact for both, and it is permanent.
    let scout = spawn_unit(&mut s, PlayerId(1), UnitKind::Scout, h(1, 10));
    visibility::update_explored(&mut s);
    assert!(s.diplomacy.as_ref().unwrap().met(TeamId(0), TeamId(1)));
    assert_eq!(player_view(&s, PlayerId(1)).diplomacy.met, vec![TeamId(0)]);
    assert!(validate_orders(&s, PlayerId(0), std::slice::from_ref(&alliance)).is_empty());
    assert!(validate_orders(&s, PlayerId(1), &[ally_offer(0)]).is_empty());
    s.units.remove(&scout);
    visibility::update_explored(&mut s);
    assert!(validate_orders(&s, PlayerId(0), &[alliance]).is_empty());
    // Saves from before the rule have no contact record: every pair counts as met.
    let mut legacy = serde_json::to_value(game(&[0, 1])).unwrap();
    legacy["diplomacy"]
        .as_object_mut()
        .unwrap()
        .remove("contacts");
    let legacy: GameState = serde_json::from_value(legacy).unwrap();
    assert!(legacy.diplomacy.as_ref().unwrap().contacts.is_none());
    assert!(legacy.diplomacy.as_ref().unwrap().met(TeamId(0), TeamId(1)));
}

// ---- Calls to arms: defenders join at once, an ally's own war is a call it may decline ----

fn calls_for(s: &GameState, p: u8) -> Vec<legatum_core::diplomacy::WarCall> {
    player_view(s, PlayerId(p)).diplomacy.calls.clone()
}
fn join(call: &legatum_core::diplomacy::WarCall) -> Order {
    response(call.id.0, true)
}
fn decline(call: &legatum_core::diplomacy::WarCall) -> Order {
    response(call.id.0, false)
}
fn rel(s: &GameState, a: u8, b: u8) -> &legatum_core::diplomacy::Relation {
    s.diplomacy
        .as_ref()
        .unwrap()
        .relation(TeamId(a), TeamId(b))
        .unwrap()
}

#[test]
fn a_team_attacked_in_a_war_is_defended_by_its_allies_in_the_same_turn() {
    // 1 is 2's ally; 0 declares on 2 (a surprise war that starts this turn).
    let s = alliance(&rich(game(&[0, 1, 2])), 1, 2);
    let (s, events) = step(&s, &[(0, vec![war(2)])]);
    assert!(at_war(&s, 0, 2) && at_war(&s, 1, 0), "the defender joins");
    assert!(!at_war(&s, 1, 2), "allies fight on the same side");
    assert_eq!(joined_events(&events), [(0, 1, 2)]);
    let joined = rel(&s, 0, 1);
    assert_eq!(joined.declarers, [TeamId(0)], "the declarer started both");
    assert_eq!(joined.war_id, rel(&s, 0, 2).war_id);
    assert_eq!(joined.effective_turn, s.turn - 1, "effective the same turn");
    assert!(calls_for(&s, 1).is_empty() && calls_for(&s, 2).is_empty());
}

#[test]
fn an_allys_war_is_a_call_to_arms_and_joining_is_an_order() {
    use legatum_core::diplomacy::CALL_WINDOW_TURNS;
    let s = alliance(&rich(game(&[0, 1, 2])), 0, 1);
    let (s, events) = step(&s, &[(0, vec![war(2)])]);
    assert!(at_war(&s, 0, 2));
    assert!(!at_war(&s, 1, 2), "the ally is asked, not dragged in");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CallToArms { call } if call.team == TeamId(1)))
    );
    let calls = calls_for(&s, 1);
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!((call.ally, call.target), (TeamId(0), TeamId(2)));
    assert_eq!(call.created_turn, s.turn);
    assert_eq!(call.respond_by, s.turn + CALL_WINDOW_TURNS - 1);
    assert!(calls_for(&s, 0).is_empty() && calls_for(&s, 2).is_empty());
    // The inbox offers Join and Decline, both open, free and no betrayal.
    let view = player_view(&s, PlayerId(1));
    for accept in [true, false] {
        let a = view
            .diplomacy
            .actions
            .iter()
            .find(|a| a.order == response(call.id.0, accept))
            .expect("an action for the call");
        assert_eq!(
            (a.blocked_reason.clone(), a.cost, a.betrayal),
            (None, 0, false)
        );
        assert_eq!(a.effective_turn, s.turn + 1);
    }
    let (s, events) = step(&s, &[(1, vec![join(call)])]);
    assert!(at_war(&s, 1, 2), "joining puts it at War with the target");
    assert!(!at_war(&s, 0, 1));
    assert!(calls_for(&s, 1).is_empty());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CallAnswered { joined: true, .. }))
    );
    assert_eq!(joined_events(&events), [(1, 2, 0)]);
    let r = rel(&s, 1, 2);
    assert_eq!(r.declarers, [TeamId(1)], "the joiner chose this war");
    assert_eq!(r.war_id, rel(&s, 0, 2).war_id);
}

#[test]
fn declining_a_call_keeps_the_alliance_and_the_peace_with_no_penalty() {
    let s = alliance(&rich(game(&[0, 1, 2])), 0, 1);
    let (s, _) = step(&s, &[(0, vec![war(2)])]);
    let call = calls_for(&s, 1).remove(0);
    let gold = s.player(PlayerId(1)).gold;
    let (s, events) = step(&s, &[(1, vec![decline(&call)])]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CallAnswered { joined: false, .. }))
    );
    assert_eq!(alliance_relation(&s, 0, 1).state, RelationState::Alliance);
    assert!(!at_war(&s, 1, 2));
    assert!(s.diplomacy.as_ref().unwrap().betrayals.is_empty());
    assert!(calls_for(&s, 1).is_empty());
    assert_eq!(s.player(PlayerId(1)).gold, gold - 1, "only the upkeep");
    // The ally can still call: nothing remembers the refusal, and 2 is not at war with 1.
    assert!(
        validate_orders(&s, PlayerId(1), &[war(2)]).is_empty(),
        "1 may still choose its own war later"
    );
}

#[test]
fn an_unanswered_call_lapses_and_the_peace_stands() {
    use legatum_core::diplomacy::CALL_WINDOW_TURNS;
    let s = alliance(&rich(game(&[0, 1, 2])), 0, 1);
    let (mut s, _) = step(&s, &[(0, vec![war(2)])]);
    assert_eq!(calls_for(&s, 1).len(), 1);
    let mut lapsed = false;
    for _ in 0..CALL_WINDOW_TURNS {
        let (next, events) = step(&s, &[]);
        lapsed |= events.iter().any(|e| matches!(e, Event::CallLapsed { .. }));
        s = next;
    }
    assert!(lapsed && calls_for(&s, 1).is_empty());
    assert!(!at_war(&s, 1, 2));
    assert_eq!(alliance_relation(&s, 0, 1).state, RelationState::Alliance);
}

#[test]
fn a_call_ends_when_the_war_does_or_the_alliance_does() {
    let s = alliance(&rich(game(&[0, 1, 2, 3])), 0, 1);
    let (s, _) = step(&s, &[(0, vec![war(2)])]);
    let call = calls_for(&s, 1).remove(0);
    // The war ends in Peace: the call is void.
    let (s2, _) = step(&s, &[(2, vec![offer(0)])]);
    let p = proposal_to(&s2, 0);
    let (s2, _) = step(&s2, &[(0, vec![response(p.id.0, true)])]);
    assert!(!at_war(&s2, 0, 2));
    assert!(calls_for(&s2, 1).is_empty());
    assert!(validate_orders(&s2, PlayerId(1), &[join(&call)]).len() == 1);
    // The ally leaves the Alliance: the call is void too.
    let (s3, events) = step(&s, &[(1, vec![leave(0)])]);
    assert!(calls_for(&s3, 1).is_empty());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CallLapsed { reason, .. } if reason.contains("Alliance")))
    );
}

#[test]
fn a_call_to_arms_names_the_ally_the_target_and_that_declining_changes_nothing_in_the_offer() {
    let s = alliance(&rich(game(&[0, 1, 2, 3])), 0, 1);
    let view = player_view(&s, PlayerId(0));
    let action = view
        .diplomacy
        .actions
        .iter()
        .find(|a| a.order == war(2))
        .unwrap();
    use legatum_core::diplomacy::NewWar;
    let new = |team, enemy| NewWar {
        team: TeamId(team),
        enemy: TeamId(enemy),
    };
    assert!(action.new_wars.is_empty(), "nobody defends 2");
    assert_eq!(action.calls, [new(1, 2)], "the ally would be asked");
    // With an ally of the target, the defender is a certain war.
    let s = alliance(&s, 2, 3);
    let view = player_view(&s, PlayerId(0));
    let action = view
        .diplomacy
        .actions
        .iter()
        .find(|a| a.order == war(2))
        .unwrap();
    assert_eq!(action.new_wars, [new(3, 0)], "2's ally fights us");
    assert_eq!(action.calls, [new(1, 2)]);
    let (s, _) = step(&s, &[(0, vec![war(2)])]);
    assert!(at_war(&s, 0, 2) && at_war(&s, 3, 0));
    assert!(!at_war(&s, 1, 2) && !at_war(&s, 1, 3), "1 is only asked");
    // 1 joins: 2's ally 3 now defends against 1 too, at once.
    let call = calls_for(&s, 1).remove(0);
    let (s, _) = step(&s, &[(1, vec![join(&call)])]);
    assert!(at_war(&s, 1, 2) && at_war(&s, 1, 3));
    assert!(calls_for(&s, 1).is_empty());
}

#[test]
fn signing_an_alliance_during_a_war_joins_defended_wars_and_calls_for_declared_ones() {
    use legatum_core::diplomacy::NewWar;
    let new = |team, enemy| NewWar {
        team: TeamId(team),
        enemy: TeamId(enemy),
    };
    // 0 declared on 2 (an aggressor); 3 declared on 1 (1 is the defender).
    let s = rich(game(&[0, 1, 2, 3]));
    let s = step(&s, &[(0, vec![war(2)])]).0;
    let s = step(&s, &[(3, vec![war(1)])]).0;
    let quote = player_view(&s, PlayerId(0))
        .diplomacy
        .actions
        .iter()
        .find(|a| a.order == ally_offer(1))
        .cloned()
        .unwrap();
    assert_eq!(quote.blocked_reason, None);
    assert_eq!(quote.new_wars, [new(0, 3)], "0 joins the war 1 defends");
    assert_eq!(quote.calls, [new(1, 2)], "1 is asked to join 0's war");
    let s = alliance(&s, 0, 1);
    assert!(at_war(&s, 0, 3), "the defended war is joined at once");
    assert!(!at_war(&s, 1, 2), "the declared war is a call");
    let calls = calls_for(&s, 1);
    assert_eq!(calls.len(), 1);
    assert_eq!((calls[0].ally, calls[0].target), (TeamId(0), TeamId(2)));
    assert!(calls_for(&s, 0).is_empty(), "1 declared nothing");
}

#[test]
fn a_peace_does_not_need_the_ally_that_declined() {
    let s = alliance(&rich(game(&[0, 1, 2])), 0, 1);
    let s = step(&s, &[(0, vec![war(2)])]).0;
    let call = calls_for(&s, 1).remove(0);
    let s = step(&s, &[(1, vec![decline(&call)])]).0;
    let (s, _) = step(&s, &[(2, vec![offer(0)])]);
    let p = proposal_to(&s, 0);
    assert_eq!(
        player_view(&s, PlayerId(0)).diplomacy.proposals[0].needs,
        [TeamId(0)]
    );
    assert_eq!(
        player_view(&s, PlayerId(0)).diplomacy.proposals[0].required_teams,
        [TeamId(0), TeamId(2)]
    );
    assert!(player_view(&s, PlayerId(1)).diplomacy.proposals.is_empty());
    let (s, _) = step(&s, &[(0, vec![response(p.id.0, true)])]);
    assert!(!at_war(&s, 0, 2));
}

#[test]
fn separate_peace_preserves_the_alliance_and_its_allys_war() {
    let s = alliance(&rich(game(&[0, 1, 2])), 1, 2);
    let (s, _) = step(&s, &[(0, vec![war(1)])]);
    assert!(at_war(&s, 0, 1) && at_war(&s, 0, 2));
    let before_ally = player_view(&s, PlayerId(2));
    let (s, created) = step(&s, &[(1, vec![offer(0)])]);
    let p = proposal_to(&s, 0);
    let parties = [TeamId(0), TeamId(1)];
    assert!(created.iter().any(|event| matches!(event,
        Event::ProposalCreated { proposal } if proposal.required_teams.as_slice() == parties)));
    assert!(
        event_visibility::visible_events(&created, &before_ally, &player_view(&s, PlayerId(2)))
            .iter()
            .all(|event| !matches!(
                event,
                event_visibility::VisibleEvent::Complete(Event::ProposalCreated { .. })
            ))
    );
    assert_eq!(
        player_view(&s, PlayerId(0)).diplomacy.proposals[0].required_teams,
        parties
    );
    assert!(player_view(&s, PlayerId(2)).diplomacy.proposals.is_empty());
    let (s, accepted) = step(&s, &[(0, vec![response(p.id.0, true)])]);
    assert!(
        accepted.iter().any(|event| matches!(event,
        Event::ProposalAccepted { proposal, .. } if proposal.required_teams.as_slice() == parties))
    );
    assert!(!at_war(&s, 0, 1) && at_war(&s, 0, 2));
    assert_eq!(alliance_relation(&s, 1, 2).state, RelationState::Alliance);
    assert_eq!(rel(&s, 0, 1).guarantee_until, s.turn + 10);
    assert_eq!(rel(&s, 0, 2).guarantee_until, 0);
}

#[test]
fn a_new_call_after_separate_peace_cannot_break_that_pairs_truce() {
    let s = rich(game(&[0, 1, 2]));
    let s = step(&s, &[(0, vec![war(1)]), (2, vec![war(0)])]).0;
    assert!(at_war(&s, 0, 1) && at_war(&s, 0, 2));
    let s = step(&s, &[(1, vec![offer(0)])]).0;
    let p = proposal_to(&s, 0);
    let s = step(&s, &[(0, vec![response(p.id.0, true)])]).0;
    assert!(!at_war(&s, 0, 1) && at_war(&s, 0, 2));
    let s = alliance(&s, 1, 2);
    assert_eq!(alliance_relation(&s, 1, 2).state, RelationState::Alliance);
    let call = calls_for(&s, 1).remove(0);
    assert_eq!(call.target, TeamId(0));
    assert!(blocked(&s, 1, &join(&call)).unwrap().contains("guaranteed"));
    assert_eq!(blocked(&s, 1, &decline(&call)), None);
    assert!(at_war(&s, 0, 2));
}

#[test]
fn a_team_with_two_permanent_teammates_makes_one_pairwise_peace() {
    let s = rich(game(&[0, 0, 1, 2]));
    let s = step(&s, &[(0, vec![war(1)]), (3, vec![war(0)])]).0;
    assert!(at_war(&s, 0, 1) && at_war(&s, 0, 2));
    let s = step(&s, &[(2, vec![offer(0)])]).0;
    let p = proposal_to(&s, 0);
    assert_eq!(
        player_view(&s, PlayerId(1)).diplomacy.proposals[0].required_teams,
        [TeamId(0), TeamId(1)]
    );
    let s = step(&s, &[(1, vec![response(p.id.0, true)])]).0;
    assert!(!at_war(&s, 0, 1) && at_war(&s, 0, 2));
    assert!(!s.at_war(PlayerId(0), PlayerId(2)) && !s.at_war(PlayerId(1), PlayerId(2)));
}

#[test]
fn a_guaranteed_peace_blocks_the_defender_who_would_have_to_break_it() {
    // 1 defends 2; 1 and 0 hold a guaranteed Peace, so 0 cannot declare on 2 yet.
    let mut s = alliance(&rich(game(&[0, 1, 2])), 1, 2);
    let turn = s.turn;
    s.diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(0), TeamId(1)))
        .unwrap()
        .guarantee_until = turn + 5;
    let errors = validate_orders(&s, PlayerId(0), &[war(2)]);
    assert!(errors[0].message.contains("guaranteed"), "{errors:?}");
}

#[test]
fn a_call_to_join_is_blocked_by_a_guaranteed_peace_with_the_target() {
    let mut s = alliance(&rich(game(&[0, 1, 2])), 0, 1);
    let (next, _) = step(&s, &[(0, vec![war(2)])]);
    s = next;
    let call = calls_for(&s, 1).remove(0);
    let turn = s.turn;
    s.diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(1), TeamId(2)))
        .unwrap()
        .guarantee_until = turn + 1;
    let reason = blocked(&s, 1, &join(&call)).expect("join is blocked");
    assert!(reason.contains("guaranteed"), "{reason}");
    assert_eq!(blocked(&s, 1, &decline(&call)), None, "declining is open");
}

#[test]
fn a_call_to_join_cannot_break_peace_with_the_targets_defender() {
    // 0-1 and 2-3 are allied. Joining 0's war on 2 would make 3 defend against 1.
    let mut s = alliance(&alliance(&rich(game(&[0, 1, 2, 3])), 0, 1), 2, 3);
    let (next, _) = step(&s, &[(0, vec![war(2)])]);
    s = next;
    let call = calls_for(&s, 1).remove(0);
    let until = s.turn + 5;
    s.diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(1), TeamId(3)))
        .unwrap()
        .guarantee_until = until;
    let reason = blocked(&s, 1, &join(&call)).expect("the defender's truce blocks joining");
    assert!(reason.contains("would have to join the war on"), "{reason}");
    assert!(
        reason.contains(&format!("through turn {}", until - 1)),
        "{reason}"
    );
    assert_eq!(blocked(&s, 1, &decline(&call)), None);
    let (next, events) = step(&s, &[(1, vec![join(&call)])]);
    assert!(!at_war(&next, 1, 2) && !at_war(&next, 1, 3));
    assert_eq!(rel(&next, 1, 3).guarantee_until, until);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { reason: failed, .. } if failed == &reason))
    );
}

#[test]
fn a_chain_of_alliances_defends_and_calls_only_direct_allies() {
    // 0-1-2 are chained; 3 declares on 0: only 1 (0's direct ally) defends.
    let s = alliance(&alliance(&rich(game(&[0, 1, 2, 3])), 0, 1), 1, 2);
    let (s, _) = step(&s, &[(3, vec![war(0)])]);
    assert!(at_war(&s, 3, 0) && at_war(&s, 3, 1));
    assert!(!at_war(&s, 3, 2), "2 is only the ally of an ally");
}

#[test]
fn a_heal_order_is_refused_in_closed_foreign_land_where_nothing_heals() {
    let mut s = game(&[0, 1, 2]);
    s.player_mut(PlayerId(0)).faction = Faction::Switzerland;
    s.player_mut(PlayerId(1)).faction = Faction::Switzerland;
    let a = spawn_unit(&mut s, PlayerId(0), UnitKind::Ranger, h(6, 5));
    for tile in h(6, 5).within(2) {
        s.map.get_mut(tile).unwrap().owner = Some(PlayerId(1));
    }
    s.units.get_mut(&a).unwrap().hp = 60;
    let errs = legatum_core::validate_orders(
        &s,
        PlayerId(0),
        &[Order::SleepUnit {
            unit: a,
            until_healed: true,
        }],
    );
    assert_eq!(errs.len(), 1);
    assert!(
        errs[0].message.contains("closed foreign"),
        "{}",
        errs[0].message
    );
}

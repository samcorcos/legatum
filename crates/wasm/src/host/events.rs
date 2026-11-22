//! Online and local reports share the same core visibility policy.
pub(crate) use legatum_core::event_visibility::{VisibleEvent, visible_events};

// Diplomacy is deliberately covered by the same policy as local play: war/peace announcements
// are public, proposals belong to their two teams, superseded orders to the ordering team.
#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::diplomacy::TreatyKind;
    use legatum_core::{GameSetup, Order, PlayerId, TeamId, new_game, player_view, resolve_turn};
    use std::collections::BTreeMap;

    #[test]
    fn lan_reports_publish_war_but_do_not_publish_private_offers() {
        let s = new_game(&GameSetup::all_ai(15));
        let (war, declaration) = resolve_turn(
            &s,
            &BTreeMap::from([(
                PlayerId(0),
                vec![Order::DeclareWar {
                    against: TeamId(1),
                    breach: false,
                }],
            )]),
        );
        let third = PlayerId(2);
        assert!(
            serde_json::to_string(&visible_events(
                &declaration,
                &player_view(&s, third),
                &player_view(&war, third)
            ))
            .unwrap()
            .contains("WarDeclared")
        );
        let (offered, events) = resolve_turn(
            &war,
            &BTreeMap::from([(
                PlayerId(0),
                vec![Order::ProposeTreaty {
                    to: TeamId(1),
                    kind: TreatyKind::Peace,
                }],
            )]),
        );
        for p in [PlayerId(0), PlayerId(1), third] {
            let visible = serde_json::to_string(&visible_events(
                &events,
                &player_view(&war, p),
                &player_view(&offered, p),
            ))
            .unwrap();
            assert_eq!(visible.contains("ProposalCreated"), p != third);
        }
    }
}

#[cfg(test)]
mod alliance_terms_visibility {
    use super::*;
    use legatum_core::{
        diplomacy::{ProposalId, TreatyKind},
        *,
    };
    use std::collections::BTreeMap;
    #[test]
    fn lan_alliances_and_betrayals_are_public_but_breach_payments_and_offers_are_private() {
        let mut s = new_game(&GameSetup::all_ai(19));
        for p in &mut s.players {
            p.gold = 500;
        }
        // Visibility of treaty events, not first contact, is under test here.
        let teams = s.teams();
        s.diplomacy.as_mut().unwrap().meet_all(&teams);
        let step = |s: &GameState, p, order| {
            resolve_turn(s, &BTreeMap::from([(PlayerId(p), vec![order])]))
        };
        s = step(
            &s,
            0,
            Order::ProposeTreaty {
                to: TeamId(1),
                kind: TreatyKind::Alliance,
            },
        )
        .0;
        let (next, events) = step(
            &s,
            1,
            Order::RespondToProposal {
                proposal: ProposalId(1),
                accept: true,
                explanation: None,
            },
        );
        for p in [PlayerId(0), PlayerId(1), PlayerId(2)] {
            let json = serde_json::to_string(&visible_events(
                &events,
                &player_view(&s, p),
                &player_view(&next, p),
            ))
            .unwrap();
            assert!(json.contains("TreatyActivated"));
            assert_eq!(json.contains("ProposalAccepted"), p != PlayerId(2));
            assert!(!json.contains("TreatyPayment"));
        }
        s = next;
        while s.turn < 5 {
            s = resolve_turn(&s, &BTreeMap::new()).0;
        }
        let (next, events) = step(
            &s,
            0,
            Order::DeclareWar {
                against: TeamId(1),
                breach: true,
            },
        );
        let outsider = serde_json::to_string(&visible_events(
            &events,
            &player_view(&s, PlayerId(2)),
            &player_view(&next, PlayerId(2)),
        ))
        .unwrap();
        for name in ["TreatyBreached", "AllianceBetrayed", "WarDeclared"] {
            assert!(outsider.contains(name));
        }
        assert!(!outsider.contains("TreatyPayment"));
        let payer = serde_json::to_string(&visible_events(
            &events,
            &player_view(&s, PlayerId(0)),
            &player_view(&next, PlayerId(0)),
        ))
        .unwrap();
        assert!(payer.contains("TreatyPayment"));
    }
}

#[cfg(test)]
mod alliance_visibility {
    use super::*;
    use legatum_core::{
        diplomacy::{DecisionExplanation, ProposalId, TreatyKind},
        *,
    };
    use std::collections::BTreeMap;
    #[test]
    fn lan_wars_are_public_but_proposals_replies_and_private_plans_stay_with_the_pair() {
        let mut s = new_game(&GameSetup::all_ai(22));
        for p in &mut s.players {
            p.gold = 500;
        }
        // Visibility of treaty events, not first contact, is under test here.
        let teams = s.teams();
        s.diplomacy.as_mut().unwrap().meet_all(&teams);
        let step = |s: &GameState, p, order| {
            resolve_turn(s, &BTreeMap::from([(PlayerId(p), vec![order])]))
        };
        s = step(
            &s,
            0,
            Order::ProposeTreaty {
                to: TeamId(1),
                kind: TreatyKind::Alliance,
            },
        )
        .0;
        s = step(
            &s,
            1,
            Order::RespondToProposal {
                proposal: ProposalId(1),
                accept: true,
                explanation: Some(DecisionExplanation {
                    score: Some(10),
                    reasons: vec!["Shared coalition target".into()],
                }),
            },
        )
        .0;
        assert_eq!(player_view(&s, PlayerId(0)).diplomacy.replies.len(), 1);
        assert!(player_view(&s, PlayerId(3)).diplomacy.replies.is_empty());
        // A war on one ally brings the other in at once: the declaration and the joining are
        // public, the proposals that signed the Alliance stayed with the pair.
        let (joined, events) = step(
            &s,
            2,
            Order::DeclareWar {
                against: TeamId(0),
                breach: false,
            },
        );
        for p in [PlayerId(0), PlayerId(1), PlayerId(2), PlayerId(3)] {
            let json = serde_json::to_string(&visible_events(
                &events,
                &player_view(&s, p),
                &player_view(&joined, p),
            ))
            .unwrap();
            assert!(json.contains("WarDeclared"));
            assert!(json.contains("WarJoined"));
            assert!(!json.contains("ProposalAccepted"));
        }
    }
}

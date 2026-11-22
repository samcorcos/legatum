//! Player reports use the union of the team's pre- and post-turn sight. Explored-but-fogged
//! cities are deliberately excluded; an empire having a visible unit does not expose its
//! private research or failed orders.

use crate::{Event, GameState, Hex, PlayerId, PlayerView, UnitId, events::Defender};
use serde::Serialize;
use std::collections::BTreeSet;

// Keep the deterministic, complete event log unchanged. The player-visible representation
// permits null endpoints when an enemy crosses the team's sight boundary, except for an
// ordinary attacker the defending team currently has revealed.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum VisibleEvent {
    Complete(Event),
    Spatial(SpatialEvent),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SpatialEvent {
    UnitMoved {
        unit: UnitId,
        owner: PlayerId,
        from: Option<Hex>,
        to: Option<Hex>,
    },
    MoveBlocked {
        unit: UnitId,
        owner: PlayerId,
        at: Option<Hex>,
        toward: Option<Hex>,
        reason: String,
    },
}

/// The pre-turn fields used by `visible_events`: sight, units, cities, and team identities.
/// The order-check map view contains these without building city forecasts or economy detail.
pub fn event_visibility_view(state: &GameState, player: PlayerId) -> PlayerView {
    crate::view::order_check_view(state, player)
}

pub fn visible_events(
    events: &[Event],
    before: &PlayerView,
    after: &PlayerView,
) -> Vec<VisibleEvent> {
    let teammates: BTreeSet<_> = before
        .players
        .iter()
        .filter(|p| p.team == before.team)
        .map(|p| p.id)
        .collect();
    let own_units: BTreeSet<_> = before
        .units
        .iter()
        .chain(&after.units)
        .filter(|u| teammates.contains(&u.owner))
        .map(|u| u.id)
        .collect();
    let observed_units: BTreeSet<_> = before
        .units
        .iter()
        .chain(&after.units)
        .map(|u| u.id)
        .collect();
    let cities: BTreeSet<_> = before
        .cities
        .iter()
        .chain(&after.cities)
        .filter(|c| c.visible || teammates.contains(&c.owner))
        .map(|c| c.id)
        .collect();
    let at =
        |h| before.tile(h).is_some_and(|t| t.visible) || after.tile(h).is_some_and(|t| t.visible);
    events
        .iter()
        .filter_map(|event| {
            // Actor identity is not permission to disclose a later location.
            match event {
                Event::UnitMoved {
                    unit,
                    owner,
                    from,
                    to,
                } if !teammates.contains(owner) => {
                    if !observed_units.contains(unit) {
                        return None;
                    }
                    return (at(*from) || at(*to)).then(|| {
                        VisibleEvent::Spatial(SpatialEvent::UnitMoved {
                            unit: *unit,
                            owner: *owner,
                            from: at(*from).then_some(*from),
                            to: at(*to).then_some(*to),
                        })
                    });
                }
                Event::MoveBlocked {
                    unit,
                    owner,
                    at: pos,
                    toward,
                    reason,
                } if !teammates.contains(owner) => {
                    if !observed_units.contains(unit) {
                        return None;
                    }
                    return (at(*pos) || at(*toward)).then(|| {
                        VisibleEvent::Spatial(SpatialEvent::MoveBlocked {
                            unit: *unit,
                            owner: *owner,
                            at: at(*pos).then_some(*pos),
                            toward: at(*toward).then_some(*toward),
                            reason: if at(*pos) && at(*toward) {
                                reason.clone()
                            } else {
                                "movement blocked".into()
                            },
                        })
                    });
                }
                _ => {}
            }
            let visible = match event {
                Event::WarDeclared { .. }
                | Event::HeirBorn { .. }
                | Event::WarJoined { .. }
                | Event::PeaceEstablished { .. }
                | Event::TreatyActivated { .. }
                | Event::TreatyEnded { .. }
                | Event::TreatyBreached { .. }
                | Event::AllianceBetrayed { .. } => true,
                Event::TreatyPayment { player, .. } => teammates.contains(player),
                // Gifts only ever move inside a team.
                Event::GoldGiven { from, .. } => teammates.contains(from),
                // The Gold is private to the two empires.
                Event::CompanyHired { owner, seller, .. } => {
                    teammates.contains(owner) || teammates.contains(seller)
                }
                Event::CompanyDeparted { owner, pos, .. }
                | Event::CompanyTurned { owner, pos, .. } => teammates.contains(owner) || at(*pos),
                Event::WonderDiscovered { owner, .. } => teammates.contains(owner),
                Event::ProposalCreated { proposal }
                | Event::ProposalAccepted { proposal, .. }
                | Event::ProposalRejected { proposal, .. }
                | Event::ProposalWithdrawn { proposal, .. }
                | Event::ProposalExpired { proposal, .. }
                | Event::ProposalInvalidated { proposal, .. } => {
                    proposal.from == before.team
                        || proposal.to == before.team
                        || proposal.required_teams.contains(&before.team)
                }
                Event::CallToArms { call }
                | Event::CallAnswered { call, .. }
                | Event::CallLapsed { call, .. } => {
                    call.team == before.team || call.ally == before.team
                }
                Event::OrderSuperseded { decision } => decision.team == before.team,
                // A World Age is revealed to every empire, so its scores are public.
                Event::EraObjectiveScored { .. } | Event::AgeBegan { .. } => true,
                Event::GreatWorkCompleted { .. }
                | Event::GreatWorkLost { .. }
                | Event::CapitalTaken { .. }
                | Event::CapitalSacked { .. }
                | Event::CapitalLost { .. }
                | Event::AthensCityCaptured { .. }
                | Event::EraEntered { .. }
                | Event::FactionGoalCompleted { .. }
                | Event::FactionGoalLost { .. }
                | Event::SwarmUnleashed { .. }
                | Event::SakokuClosed { .. }
                | Event::SakokuReopened { .. }
                | Event::TombStarted { .. }
                | Event::TombPaused { .. }
                | Event::TombRaised { .. }
                | Event::SignalBroadcast { .. }
                | Event::RoadsLaid { .. }
                | Event::WildCalled { .. }
                | Event::MegaprojectStarted { .. }
                | Event::ProductionCompleted {
                    item:
                        crate::ProductionItem::Building {
                            building:
                                crate::BuildingKind::GreatLibrary
                                | crate::BuildingKind::RoyalArsenal
                                | crate::BuildingKind::GrandBazaar
                                | crate::BuildingKind::Pyramids
                                | crate::BuildingKind::HagiaSophia
                                | crate::BuildingKind::EiffelTower
                                | crate::BuildingKind::IterFusionReactor,
                        },
                    ..
                }
                | Event::PlayerEliminated { .. }
                | Event::VictoryHoldStarted { .. }
                | Event::VictoryHoldBroken { .. }
                | Event::GameWon { .. }
                | Event::GameDrawn { .. } => true,
                Event::UnitMoved { .. } | Event::MoveBlocked { .. } => true, // teammates; enemies handled above
                Event::CityReinforced { owner, city, .. } => {
                    teammates.contains(owner) || cities.contains(city)
                }
                Event::Bounce { units, tile } => {
                    at(*tile) || units.iter().any(|id| own_units.contains(id))
                }
                Event::Battle {
                    attacker_owner,
                    defender,
                    at: pos,
                    ..
                } => {
                    at(*pos)
                        || teammates.contains(attacker_owner)
                        || match defender {
                            Defender::Unit { owner, .. } | Defender::City { owner, .. } => {
                                teammates.contains(owner)
                            }
                        }
                }
                Event::UnitDestroyed { owner, at: pos, .. }
                | Event::CityFounded { owner, at: pos, .. } => {
                    teammates.contains(owner) || at(*pos)
                }
                // Ruin rewards expose private economy/research even if the finder is visible, and
                // Waking and a finished heal reveal a private standing order. Other unit events have no
                // location to establish enemy sight.
                Event::RuinExplored { owner, .. }
                | Event::UnitAwakened { owner, .. }
                | Event::UnitHealed { owner, .. }
                | Event::AutomationStopped { owner, .. }
                | Event::UnitDisbanded { owner, .. }
                | Event::UnitUpgraded { owner, .. }
                | Event::SiegeBought { owner, .. }
                | Event::WonderRaceLost { owner, .. }
                | Event::AttackPursuit { owner, .. }
                | Event::UnitPromoted { owner, .. } => teammates.contains(owner),
                Event::SettlerCaptured {
                    from, to, at: pos, ..
                } => teammates.contains(from) || teammates.contains(to) || at(*pos),
                Event::FlowerWars { city, owner, .. }
                | Event::CityGrew { city, owner, .. }
                | Event::CityStarved { city, owner, .. }
                | Event::BuildingDismantled { city, owner, .. }
                | Event::ItemBought { city, owner, .. }
                | Event::ProductionCompleted { city, owner, .. } => {
                    teammates.contains(owner) || cities.contains(city)
                }
                Event::CityCaptured { city, from, to, .. } => {
                    teammates.contains(from) || teammates.contains(to) || cities.contains(city)
                }
                Event::SpartaConquest { player, .. } | Event::SpartaKillHeal { player, .. } => {
                    teammates.contains(player)
                }
                Event::CityRazed {
                    from, by, at: pos, ..
                } => teammates.contains(from) || teammates.contains(by) || at(*pos),
                Event::AircraftRebased { owner, .. } => teammates.contains(owner),
                Event::TileClaimed { owner, tile, .. }
                | Event::TileReleased { owner, tile, .. }
                | Event::TileBought { owner, tile, .. } => teammates.contains(owner) || at(*tile),
                Event::TilesReassigned { owner, .. } => teammates.contains(owner),
                Event::BarbarianCampSpawned { .. } => false,
                Event::BarbarianCampDiscovered { team, .. } => *team == before.team,
                Event::BarbarianScoutSpotted { owner, at: pos, .. } => {
                    teammates.contains(owner) && at(*pos)
                }
                Event::BarbarianRaidLaunched { owner, .. } => teammates.contains(owner),
                Event::BarbarianCampCleared { by, at: pos, .. } => {
                    teammates.contains(by) || at(*pos)
                }
                Event::TechResearched { player, .. }
                | Event::FreeTechsChosen { player, .. }
                | Event::VeteranRetired { player, .. }
                | Event::ResourcesRevealed { player, .. }
                | Event::SupplyShortage { player, .. }
                | Event::ResearchQueueAdvanced { player, .. }
                | Event::OrderFailed { player, .. } => teammates.contains(player),
            };
            if !visible {
                return None;
            }
            let mut event = event.clone();
            // "From where?" only as far as the team saw: a shot from the fog keeps its attacker
            // but not the tile it came from.
            if let Event::Battle {
                attacker,
                attacker_owner,
                attacker_at,
                ..
            } = &mut event
                && !teammates.contains(attacker_owner)
                && !attacker
                    .unit()
                    .is_some_and(|unit| observed_units.contains(&unit))
                && attacker_at.is_some_and(|h| !at(h))
            {
                *attacker_at = None;
            }
            Some(VisibleEvent::Complete(event))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BuildingKind, CityId, Faction, GameSetup, Hex, PlayerId, TechId, Terrain, UnitKind,
        events::Defender,
        game::{blank_game, spawn_unit},
        player_view,
        state::Map,
        visibility::update_explored,
    };

    #[test]
    fn enemy_movement_redacts_hidden_endpoints_and_drops_steps_behind_fog() {
        let setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
        let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        spawn_unit(&mut state, PlayerId(0), UnitKind::Settler, Hex::new(20, 20));
        spawn_unit(&mut state, PlayerId(2), UnitKind::Settler, Hex::new(30, 5));
        spawn_unit(&mut state, PlayerId(0), UnitKind::Spearman, Hex::new(5, 5));
        let enemy = spawn_unit(&mut state, PlayerId(2), UnitKind::Scout, Hex::new(7, 5));
        update_explored(&mut state);
        let before = player_view(&state, PlayerId(0));
        let (state, events) = crate::resolve_turn(
            &state,
            &[(
                PlayerId(2),
                vec![crate::Order::MoveUnit {
                    unit: enemy,
                    to: Hex::new(10, 5),
                }],
            )]
            .into(),
        );
        let after = player_view(&state, PlayerId(0));
        assert_eq!(state.units[&enemy].pos, Hex::new(10, 5));
        assert!(!after.units.iter().any(|u| u.id == enemy));
        let raw = serde_json::to_value(visible_events(&events, &before, &after)).unwrap();
        let moves: Vec<_> = raw
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["type"] == "UnitMoved")
            .collect();
        assert_eq!(
            moves.len(),
            1,
            "a previously seen actor cannot reveal later hidden steps"
        );
        assert_eq!(moves[0]["from"], serde_json::json!({"q": 7, "r": 5}));
        assert!(moves[0]["to"].is_null());

        let entering = Event::UnitMoved {
            unit: enemy,
            owner: PlayerId(2),
            from: Hex::new(8, 5),
            to: Hex::new(7, 5),
        };
        let blocked = Event::MoveBlocked {
            unit: enemy,
            owner: PlayerId(2),
            at: Hex::new(7, 5),
            toward: Hex::new(10, 5),
            reason: "enemy in the way".into(),
        };
        let raw =
            serde_json::to_value(visible_events(&[entering, blocked], &before, &after)).unwrap();
        assert!(raw[0]["from"].is_null());
        assert_eq!(raw[0]["to"], serde_json::json!({"q": 7, "r": 5}));
        assert!(raw[1]["toward"].is_null());
    }

    #[test]
    fn a_sacks_gold_is_public_to_every_empire_that_sees_the_city_fall() {
        let setup = GameSetup::standard(7, [Faction::India, Faction::Celts]);
        let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        let pos = Hex::new(5, 5);
        spawn_unit(&mut state, PlayerId(0), UnitKind::Spearman, pos);
        update_explored(&mut state);
        let view = player_view(&state, PlayerId(0));
        let sack = |from: u8, by: u8, ordered: bool| Event::CityRazed {
            city: CityId(9),
            from: PlayerId(from),
            by: PlayerId(by),
            at: pos,
            original_capital: false,
            sack_gold: Some(1_230),
            camp: Some(Hex::new(6, 5)),
            ordered,
            prize: false,
            walled: false,
        };
        let field = |events: &Vec<VisibleEvent>, name: &str| {
            serde_json::to_value(events).unwrap()[0].get(name).cloned()
        };
        // The victim, the captor and a third empire that sees the city fall all read the same
        // Gold (population times 30 is public math) and the camp's tile.
        for (from, by, ordered) in [(0, 2, false), (2, 0, false), (2, 3, false), (2, 3, true)] {
            let events = visible_events(&[sack(from, by, ordered)], &view, &view);
            assert_eq!(events.len(), 1);
            assert_eq!(field(&events, "sack_gold"), Some(serde_json::json!(1_230)));
            assert!(field(&events, "camp").is_some());
            assert_eq!(field(&events, "tech"), None);
        }
    }

    #[test]
    fn wake_notices_are_private_even_when_an_enemy_can_see_the_sleeper() {
        let setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
        let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        let pos = Hex::new(5, 5);
        let sleeper = spawn_unit(&mut state, PlayerId(0), UnitKind::Spearman, pos);
        spawn_unit(&mut state, PlayerId(2), UnitKind::Scout, Hex::new(6, 5));
        update_explored(&mut state);
        let events = vec![Event::UnitAwakened {
            unit: sleeper,
            owner: PlayerId(0),
            unit_kind: UnitKind::Spearman,
            at: pos,
        }];
        for player in [PlayerId(0), PlayerId(1), PlayerId(2)] {
            let view = player_view(&state, player);
            assert_eq!(
                visible_events(&events, &view, &view).len(),
                usize::from(player != PlayerId(2))
            );
        }
    }

    #[test]
    fn heal_notices_reach_only_the_owners_team() {
        let setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
        let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        let pos = Hex::new(5, 5);
        let healer = spawn_unit(&mut state, PlayerId(0), UnitKind::Spearman, pos);
        spawn_unit(&mut state, PlayerId(2), UnitKind::Scout, Hex::new(6, 5));
        update_explored(&mut state);
        let events = vec![Event::UnitHealed {
            unit: healer,
            owner: PlayerId(0),
            unit_kind: UnitKind::Spearman,
            at: pos,
        }];
        for player in [PlayerId(0), PlayerId(1), PlayerId(2)] {
            let view = player_view(&state, player);
            assert_eq!(
                visible_events(&events, &view, &view).len(),
                usize::from(player != PlayerId(2)),
                "{player:?}"
            );
        }
    }

    #[test]
    fn a_previously_seen_enemy_does_not_reveal_hidden_encounters() {
        let setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
        let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        spawn_unit(&mut state, PlayerId(0), UnitKind::Spearman, Hex::new(5, 5));
        let enemy = spawn_unit(&mut state, PlayerId(2), UnitKind::Scout, Hex::new(7, 5));
        update_explored(&mut state);
        let before = player_view(&state, PlayerId(0));
        let hidden = Hex::new(10, 5);
        state.units.get_mut(&enemy).unwrap().pos = hidden;
        update_explored(&mut state);
        let after = player_view(&state, PlayerId(0));
        let owner = PlayerId(2);
        let events = vec![
            Event::MoveBlocked {
                unit: enemy,
                owner,
                at: hidden,
                toward: Hex::new(11, 5),
                reason: "enemy in the way".into(),
            },
            Event::Bounce {
                units: vec![enemy],
                tile: hidden,
            },
            Event::RuinExplored {
                unit: enemy,
                owner,
                at: hidden,
                reward: crate::events::RuinReward::Science {
                    amount: 25,
                    research: None,
                },
            },
            Event::UnitDestroyed {
                unit: enemy,
                owner,
                unit_kind: UnitKind::Scout,
                at: hidden,
            },
            Event::SettlerCaptured {
                unit: enemy,
                from: owner,
                to: PlayerId(3),
                at: hidden,
            },
            Event::Battle {
                attacker: enemy.into(),
                attacker_owner: owner,
                defender: Defender::City {
                    city: CityId(90),
                    owner: PlayerId(3),
                },
                at: hidden,
                attacker_at: Some(hidden),
                attacker_strength: Default::default(),
                defender_strength: Default::default(),
                damage_to_attacker: 10,
                damage_to_defender: 10,
                attacker_roll: 100,
                defender_roll: 100,
                attacker_hp_after: 90,
                defender_hp_after: 90,
                ranged: false,
                meeting: false,
                air_strike: false,
                defender_leaving: false,
            },
        ];
        assert!(visible_events(&events, &before, &after).is_empty());
    }

    #[test]
    fn includes_pre_and_post_sight_team_events_and_globals() {
        let setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
        let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        let origin = Hex::from_offset(5, 5);
        let destination = Hex::from_offset(25, 20);
        let own = spawn_unit(&mut state, PlayerId(0), UnitKind::Scout, origin);
        let enemy = spawn_unit(&mut state, PlayerId(2), UnitKind::Spearman, origin);
        update_explored(&mut state);
        let before = player_view(&state, PlayerId(0));
        state.units.get_mut(&own).unwrap().pos = destination;
        state.units.remove(&enemy);
        update_explored(&mut state);
        let after = player_view(&state, PlayerId(0));
        assert!(!after.tile(origin).unwrap().visible);
        assert!(!before.tile(destination).unwrap().visible);

        let pre = Event::UnitDestroyed {
            unit: enemy,
            owner: PlayerId(2),
            unit_kind: UnitKind::Spearman,
            at: origin,
        };
        let post = Event::CityFounded {
            city: CityId(90),
            owner: PlayerId(2),
            name: "Seen after".into(),
            at: destination,
            veteran: false,
        };
        let actor = Event::UnitDisbanded {
            unit: enemy,
            owner: PlayerId(2),
            reason: "seen before".into(),
        };
        let battle = Event::Battle {
            attacker: own.into(),
            attacker_owner: PlayerId(0),
            defender: Defender::Unit {
                unit: enemy,
                owner: PlayerId(2),
                unit_kind: UnitKind::Spearman,
            },
            at: origin,
            attacker_at: Some(destination),
            attacker_strength: Default::default(),
            defender_strength: Default::default(),
            damage_to_attacker: 10,
            damage_to_defender: 10,
            attacker_roll: 100,
            defender_roll: 100,
            attacker_hp_after: 90,
            defender_hp_after: 90,
            air_strike: false,
            defender_leaving: false,
            ranged: false,
            meeting: false,
        };
        let global = Event::GreatWorkLost {
            player: PlayerId(7),
            of: PlayerId(6),
            building: BuildingKind::GreatLibrary,
            vp: 2,
        };
        let teammate = Event::TechResearched {
            player: PlayerId(1),
            tech: TechId::BronzeWorking,
        };
        let hidden = Event::CityFounded {
            city: CityId(91),
            owner: PlayerId(3),
            name: "Hidden".into(),
            at: Hex::from_offset(40, 2),
            veteran: false,
        };
        let research = Event::TechResearched {
            player: PlayerId(2),
            tech: TechId::BronzeWorking,
        };
        let expected = vec![
            pre,
            post,
            battle,
            global,
            teammate,
            Event::ResearchQueueAdvanced {
                player: PlayerId(1),
                next: Some(TechId::Writing),
            },
            Event::ResearchQueueAdvanced {
                player: PlayerId(0),
                next: None,
            },
        ];
        let mut events = expected.clone();
        events.extend([
            hidden,
            research,
            actor,
            Event::ResearchQueueAdvanced {
                player: PlayerId(2),
                next: None,
            },
        ]);
        assert_eq!(
            serde_json::to_value(visible_events(&events, &before, &after)).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }

    #[test]
    fn a_shot_from_the_fog_keeps_its_attacker_but_not_where_it_stood() {
        let setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
        let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        let target_at = Hex::from_offset(5, 5);
        let own = spawn_unit(&mut state, PlayerId(0), UnitKind::Spearman, target_at);
        update_explored(&mut state);
        let view = player_view(&state, PlayerId(0));
        let near = Hex::from_offset(6, 5);
        let fog = Hex::from_offset(30, 20);
        assert!(view.tile(near).unwrap().visible);
        assert!(!view.tile(fog).unwrap().visible);
        let shot = |owner: PlayerId, from: Hex| Event::Battle {
            attacker: crate::UnitId(900).into(),
            attacker_owner: owner,
            defender: Defender::Unit {
                unit: own,
                owner: PlayerId(0),
                unit_kind: UnitKind::Spearman,
            },
            at: target_at,
            attacker_at: Some(from),
            attacker_strength: Default::default(),
            defender_strength: Default::default(),
            damage_to_attacker: 0,
            damage_to_defender: 20,
            attacker_roll: 100,
            defender_roll: 0,
            attacker_hp_after: 100,
            defender_hp_after: 80,
            ranged: true,
            meeting: false,
            air_strike: false,
            defender_leaving: false,
        };
        let events = [
            shot(PlayerId(2), fog),
            shot(PlayerId(2), near),
            shot(PlayerId(1), fog),
        ];
        let raw = serde_json::to_value(visible_events(&events, &view, &view)).unwrap();
        assert_eq!(
            raw.as_array().unwrap().len(),
            3,
            "the hit itself is always reported"
        );
        assert!(
            raw[0].get("attacker_at").is_none(),
            "an enemy firing from unseen tiles: no position"
        );
        assert_eq!(raw[0]["attacker_owner"], serde_json::json!(2));
        assert_eq!(
            raw[1]["attacker_at"],
            serde_json::to_value(near).unwrap(),
            "a seen attacker tile is kept"
        );
        assert_eq!(
            raw[2]["attacker_at"],
            serde_json::to_value(fog).unwrap(),
            "a teammate's position is always known"
        );
    }

    #[test]
    fn explored_enemy_cities_do_not_make_growth_public() {
        let state = crate::new_game(&GameSetup::standard(
            7,
            [Faction::India, Faction::Byzantium],
        ));
        let mut state = state;
        let city = state.cities_of(PlayerId(2)).next().unwrap().clone();
        // Player 0 saw this city once (fog memory), but has no unit in sight of it now.
        let idx = state.map.index(city.pos).unwrap();
        state.players[0].explored.set(idx);
        let len = state.map.len();
        let tile_owner = state.map.tiles[idx].owner;
        let memory = &mut state.players[0].tile_memory;
        memory.resize(len, None);
        memory[idx] = Some(crate::state::TileMemory {
            owner: tile_owner,
            ruin: false,
            improved: false,
            city: Some(crate::state::CityMemory::from(&city)),
        });
        state
            .units
            .retain(|_, u| u.owner != PlayerId(0) && u.owner != PlayerId(1));
        let view = player_view(&state, PlayerId(0));
        assert!(view.cities.iter().any(|c| c.id == city.id && !c.visible));
        let event = Event::CityGrew {
            city: city.id,
            owner: city.owner,
            population: 2,
        };
        assert!(visible_events(&[event], &view, &view).is_empty());
    }

    #[test]
    fn a_bought_tile_is_visible_to_its_owner_but_not_to_a_rival_who_cannot_see_it() {
        let state = crate::new_game(&GameSetup::standard(
            7,
            [Faction::India, Faction::Byzantium],
        ));
        let city = state.cities_of(PlayerId(0)).next().unwrap().clone();
        let tile = city.pos.ring(2)[0];
        let event = Event::TileBought {
            city: city.id,
            owner: PlayerId(0),
            tile,
            gold: 50,
        };
        let mine = player_view(&state, PlayerId(0));
        let theirs = player_view(&state, PlayerId(2));
        assert_eq!(
            visible_events(std::slice::from_ref(&event), &mine, &mine).len(),
            1
        );
        assert!(visible_events(&[event], &theirs, &theirs).is_empty());
    }

    #[test]
    fn visible_rival_city_does_not_reveal_its_wonder_refund() {
        let state = crate::new_game(&GameSetup::standard(
            7,
            [Faction::India, Faction::Byzantium],
        ));
        let owner_view = player_view(&state, PlayerId(2));
        let city = owner_view
            .cities
            .iter()
            .find(|c| c.owner == PlayerId(2))
            .unwrap();
        let event = Event::WonderRaceLost {
            city: city.id,
            owner: PlayerId(2),
            winner: PlayerId(0),
            building: BuildingKind::Pyramids,
            production: 140,
            gold: 70,
            active: true,
        };
        let mut rival_view = player_view(&state, PlayerId(0));
        rival_view.cities.push(city.clone());
        assert!(visible_events(std::slice::from_ref(&event), &rival_view, &rival_view).is_empty());
        let own = visible_events(&[event], &owner_view, &owner_view);
        assert_eq!(own.len(), 1);
        assert_eq!(serde_json::to_value(&own[0]).unwrap()["gold"], 70);
    }
}

use super::*;
use crate::winchance::WinFacts;
use legatum_core::{GameSetup, player_view, resolve_turn};
use serde::Deserialize;

#[derive(Deserialize)]
struct PlaytestFixture {
    state: GameState,
    memories: Vec<(PlayerId, Option<crate::OpponentMemory>)>,
    #[serde(default)]
    future_orders: Vec<BTreeMap<PlayerId, Vec<legatum_core::Order>>>,
}

fn playtest_fixture(bytes: &[u8]) -> PlaytestFixture {
    serde_json::from_slice(&zstd::decode_all(bytes).expect("compressed playtest fixture"))
        .expect("playtest fixture")
}

fn playtest_brain(fixture: &PlaytestFixture, player: PlayerId) -> ScriptedBrain {
    let seed = Rng::derive(
        fixture.state.setup.seed,
        &[legatum_core::rng::purpose::AI, u64::from(player.0)],
    )
    .next_u64();
    let mut brain = ScriptedBrain::with_difficulty(
        fixture.state.player(player).faction,
        seed,
        player,
        AiDifficulty::Hard,
    );
    if let Some(memory) = fixture
        .memories
        .iter()
        .find(|(id, _)| *id == player)
        .and_then(|(_, m)| m.clone())
    {
        brain.restore_opponent_memory(memory);
    }
    brain
}

#[test]
fn mongols_partial_reviews_campaign_counterfactual() {
    let f = playtest_fixture(include_bytes!("fixtures/mongols-partial-t51.json.zst"));
    assert_eq!(f.state.turn, 51);
    assert_eq!(f.future_orders.len(), 4);
    let india = PlayerId(2);
    let byzantium = PlayerId(3);
    // This archived order sequence predates the new diplomacy valuations. Keep the
    // replay on its recorded model; current prices have separate position tests.
    let archived = |player| {
        let mut brain = playtest_brain(&f, player);
        brain.decisions.second_front_siege_price = 0;
        brain.decisions.rival_redeclare_percent = 0;
        brain.decisions.peace_withdraw_price = 0;
        brain.decisions.ally_defense_percent = 0;
        brain.decisions.ally_vision_cost = 0;
        brain.decisions.ally_passage_cost = 0;
        brain.decisions.ally_exit_discount_percent = 0;
        brain.decisions.leader_peace_horizon = 0;
        brain.decisions.ally_commitment_model = 0;
        brain.decisions.modernization_value = 0;
        brain
    };
    let mut strike = SearchBrain::new(archived(india), SearchConfig::in_app(4));
    let mut ally = SearchBrain::new(archived(byzantium), SearchConfig::in_app(4));
    let mut campaign = archived(india);
    campaign.campaigns.plan.forced = Some(ForcedPlan {
        kind: PlanKind::Campaign {
            against: legatum_core::TeamId(0),
            target: legatum_core::CityId(9),
        },
        until: 55,
    });
    let mut state = f.state.clone();
    for (i, recorded) in f.future_orders.iter().enumerate() {
        let orders = strike.orders(&player_view(&state, india));
        if i == 0 {
            let verdict = strike.search_events().last().expect("T51 search review");
            assert!(!verdict.omniscient);
            assert!(
                verdict
                    .options
                    .iter()
                    .any(|o| matches!(&o.kind, PlanKind::Economy))
            );
            assert!(verdict.options.iter().any(|o| matches!(
                &o.kind,
                PlanKind::Campaign {
                    target: legatum_core::CityId(9),
                    ..
                }
            )));
            // Which plan wins depends on how well the rollouts' default Hard rivals
            // defend (Strongest Army, door defense), so only both being compared is
            // pinned here; the forced campaign below plays the campaign either way.
        }
        assert!(legatum_core::validate_orders(&state, india, &orders).is_empty());
        let campaign_orders = campaign.orders(&player_view(&state, india));
        if i == 0 {
            assert!(campaign_orders.iter().any(|o| matches!(
                o,
                legatum_core::Order::DeclareWar {
                    against: legatum_core::TeamId(0),
                    ..
                }
            )));
        }
        if i == 1 {
            assert!(
                campaign
                    .campaign_events()
                    .iter()
                    .any(|e| e.target == legatum_core::CityId(9)
                        && e.turn <= 52
                        && e.end.is_none())
            );
        }
        let byzantium_view = player_view(&state, byzantium);
        let proposal = ally.orders(&byzantium_view);
        if i == 3 {
            let coalition_offer = byzantium_view.diplomacy.proposals.iter().find(|p| {
                p.from == legatum_core::TeamId(2)
                    && p.kind == legatum_core::diplomacy::TreatyKind::Alliance
            });
            if let Some(coalition_offer) = coalition_offer {
                assert!(
                    proposal.iter().any(|o| matches!(
                        o,
                        legatum_core::Order::RespondToProposal { proposal, accept: true, .. }
                            if *proposal == coalition_offer.id
                    )),
                    "coalition Alliance orders: {proposal:?}"
                );
            }
        }
        // Force the campaign throughout the replay to exercise its coalition reply
        // and capture, regardless of later search reviews.
        let mut together = recorded.clone();
        together.insert(india, campaign_orders);
        state = resolve_turn(&state, &together).0;
    }
    assert_eq!(
        state.cities.get(&legatum_core::CityId(9)).unwrap().owner,
        india
    );
}

#[test]
fn mongols_partial_byzantium_queues_walls_under_attack() {
    for (bytes, turn, city) in [
        (
            include_bytes!("fixtures/mongols-partial-t27.json.zst").as_slice(),
            27,
            legatum_core::CityId(4),
        ),
        (
            include_bytes!("fixtures/mongols-partial-t29.json.zst").as_slice(),
            29,
            legatum_core::CityId(5),
        ),
    ] {
        let f = playtest_fixture(bytes);
        assert_eq!(f.state.turn, turn);
        let player = PlayerId(3);
        let mut brain = SearchBrain::new(playtest_brain(&f, player), SearchConfig::in_app(4));
        let orders = brain.orders(&player_view(&f.state, player));
        assert!(orders.iter().any(|o| matches!(
            o,
            legatum_core::Order::SetProduction {
                city: id,
                item: legatum_core::ProductionItem::Building {
                    building: legatum_core::BuildingKind::Walls
                }
            } if *id == city
        )));
    }
}

#[test]
fn mongols_win_t14_search_stages_and_presses_attack_after_early_defense() {
    let f = playtest_fixture(include_bytes!("fixtures/mongols-win-t14.json.zst"));
    let me = PlayerId(2);
    let mut state = f.state.clone();
    state.players[0].is_human = false;
    state.setup.players[0].is_human = false;
    state.setup.players.iter_mut().for_each(|p| {
        p.difficulty = AiDifficulty::Hard;
        p.search = true;
    });
    let mut brains: BTreeMap<_, SearchBrain> = state
        .players
        .iter()
        .map(|p| {
            (
                p.id,
                SearchBrain::new(playtest_brain(&f, p.id), SearchConfig::in_app(4)),
            )
        })
        .collect();
    let mut declared = None;
    while state.turn <= 32 && !state.finished {
        let turn = state.turn;
        let orders: BTreeMap<_, _> = brains
            .iter_mut()
            .filter(|(id, _)| state.player(**id).alive)
            .map(|(id, brain)| (*id, brain.orders(&player_view(&state, *id))))
            .collect();
        if turn == 14 {
            brains[&me].search_events().last().expect("T14 search");
            assert!(matches!(
                brains[&me]
                    .hard()
                    .campaigns
                    .plan
                    .active
                    .as_ref()
                    .map(|p| &p.kind),
                Some(PlanKind::Campaign {
                    target: legatum_core::CityId(1),
                    ..
                })
            ));
            assert!(
                orders[&me]
                    .iter()
                    .filter(|o| matches!(o, legatum_core::Order::MoveUnit { .. }))
                    .count()
                    >= 2
            );
            assert!(
                !orders[&me]
                    .iter()
                    .any(|o| matches!(o, legatum_core::Order::DeclareWar { .. }))
            );
        }
        if orders[&me].iter().any(|o| {
            matches!(
                o,
                legatum_core::Order::DeclareWar {
                    against: legatum_core::TeamId(0),
                    ..
                }
            )
        }) {
            declared = Some(turn);
        }
        state = resolve_turn(&state, &orders).0;
    }
    // With the audited defaults, the rival can complete Walls before the capture. The
    // campaign must still declare promptly and take the city.
    assert!(declared.is_some_and(|turn| turn <= 20), "{declared:?}");
    let target = state
        .cities
        .get(&legatum_core::CityId(1))
        .expect("capital survives capture");
    assert_eq!(target.owner, me);
}

#[test]
fn mongols_win_contain_never_starts_an_immediate_failed_siege() {
    let f = playtest_fixture(include_bytes!("fixtures/mongols-win-t53.json.zst"));
    assert_eq!(f.state.turn, 53);
    assert_eq!(f.future_orders.len(), 3);
    let sparta = PlayerId(2);
    let mut brain = SearchBrain::new(playtest_brain(&f, sparta), SearchConfig::in_app(4));
    let mut state = f.state;
    let mut declared = false;
    for recorded in f.future_orders {
        let view = player_view(&state, sparta);
        let mine = brain.orders(&view);
        // The T53 forecast was 15% before the Spartan phalanx (2026-10-17): a Hoplite beside another
        // fights five stronger, which lifts the capture odds of this army past the Contain threshold.
        // The war is still judged by its outcome below: it must not start and then abort for want of army.
        declared |= mine.iter().any(|o| {
            matches!(
                o,
                legatum_core::Order::DeclareWar {
                    against: legatum_core::TeamId(0),
                    ..
                }
            )
        });
        let mut together = recorded;
        together.insert(sparta, mine);
        state = resolve_turn(&state, &together).0;
    }
    let abort = brain.campaign_events().iter().any(|e| {
        e.turn >= 53
            && e.turn <= 55
            && e.end == Some(crate::campaign::CampaignEnd::InsufficientArmy)
    });
    assert!(
        !declared || !abort,
        "new Contain war aborted with insufficient army"
    );
}

#[test]
#[ignore = "economic counterfactual diagnostic"]
fn mongols_win_t14_forced_plan_trajectories() {
    let f = playtest_fixture(include_bytes!("fixtures/mongols-win-t14.json.zst"));
    let me = PlayerId(2);
    let view = player_view(&f.state, me);
    let mut search = SearchBrain::new(playtest_brain(&f, me), SearchConfig::in_app(4));
    let _ = search.orders(&view);
    eprintln!("T14 search: {:?}", search.search_events().last());
    let target = view
        .cities
        .iter()
        .find(|c| c.id.0 == 1)
        .expect("human capital");
    {
        let mut state = f.state.clone();
        let mut seat = playtest_brain(&f, me);
        seat.campaigns.plan.forced = Some(ForcedPlan {
            kind: PlanKind::Campaign {
                against: view.team_of(target.owner),
                target: target.id,
            },
            until: 34,
        });
        let mut rivals: BTreeMap<PlayerId, Box<dyn Brain>> = state
            .players
            .iter()
            .filter(|p| p.id != me)
            .map(|p| {
                let seed = Rng::derive(77, &[STREAM, u64::from(p.id.0)]).next_u64();
                (p.id, rival_brain(p.faction, seed, p.id, 0, 14))
            })
            .collect();
        for _ in 0..6 {
            let mut orders: BTreeMap<_, _> = rivals
                .iter_mut()
                .map(|(id, brain)| (*id, brain.orders(&player_view(&state, *id))))
                .collect();
            let mine = seat.orders(&player_view(&state, me));
            eprintln!(
                "trace T{} plan {:?} campaign {:?} orders {:?}",
                state.turn,
                seat.plan_events().last(),
                seat.campaign_events().last(),
                mine
            );
            orders.insert(me, mine);
            state = resolve_turn(&state, &orders).0;
        }
    }
    for plan in [
        PlanKind::Economy,
        PlanKind::Campaign {
            against: view.team_of(target.owner),
            target: target.id,
        },
    ] {
        for horizon in [1, 4, 10, 16, 26, 36, 46, 86, 136] {
            let mut brain = playtest_brain(&f, me);
            brain.campaigns.plan.forced = Some(ForcedPlan {
                kind: plan.clone(),
                until: f.state.turn + 20,
            });
            eprintln!("forced {plan:?}, to T{}:", f.state.turn + horizon);
            let _ = rollout_with(
                f.state.clone(),
                me,
                rollout_seat(brain, 0, f.state.turn),
                horizon,
                77,
                0,
                |state, me| {
                    let target = state.cities.get(&legatum_core::CityId(1));
                    eprintln!(
                        "  actual T{} finished {} war {} target {:?} field {:?}",
                        state.turn,
                        state.finished,
                        state.at_war(me, PlayerId(0)),
                        target.map(|c| (c.owner.0, c.hp, &c.buildings)),
                        state
                            .units_of(me)
                            .filter(|u| u.kind != legatum_core::UnitKind::Scout)
                            .map(|u| (u.id.0, u.kind, u.hp, u.pos.distance(target.unwrap().pos)))
                            .collect::<Vec<_>>()
                    );
                    for player in [me, PlayerId(0)] {
                        let cities: Vec<_> = state.cities_of(player).collect();
                        let yields: i32 = cities
                            .iter()
                            .map(|c| {
                                let y = legatum_core::economy::city_yields(state, c).total;
                                y.food + y.production + y.gold + y.science
                            })
                            .sum();
                        eprintln!(
                            "  P{}: cities {} pop {} yields {} VP {}",
                            player.0,
                            cities.len(),
                            cities.iter().map(|c| c.population).sum::<i32>(),
                            yields,
                            legatum_core::scoring::team_vp(state, state.team_of(player)),
                        );
                    }
                    chance_in(state, me)
                },
            );
        }
    }
}

/// An 8-seat Standard Hard game played by scripted brains to `turn`.
fn game_at(seed: u64, turn: u32) -> GameState {
    let mut setup = GameSetup::all_ai(seed);
    for p in &mut setup.players {
        p.difficulty = AiDifficulty::Hard;
    }
    let mut state = legatum_core::new_game(&setup);
    let mut brains: Vec<ScriptedBrain> = state
        .players
        .iter()
        .map(|p| ScriptedBrain::with_difficulty(p.faction, 3, p.id, AiDifficulty::Hard))
        .collect();
    while state.turn < turn && !state.finished {
        let orders = brains
            .iter_mut()
            .filter(|b| state.player(b.player).alive)
            .map(|b| (b.player, b.orders(&player_view(&state, b.player))))
            .collect();
        state = resolve_turn(&state, &orders).0;
    }
    state
}

#[test]
fn a_determinized_state_shows_the_seat_exactly_what_it_knows() {
    let state = game_at(3_600_001, 30);
    let me = PlayerId(0);
    let view = player_view(&state, me);
    let prior = prior_map(&view, 9);
    let det = determinize(&view, &prior, 17);
    let again = player_view(&det, me);
    let own = |v: &PlayerView| {
        let mut u: Vec<_> = v
            .units
            .iter()
            .filter(|u| u.owner == me)
            .map(|u| (u.id, u.kind, u.pos, u.hp, u.moves_left))
            .collect();
        u.sort();
        u
    };
    assert_eq!(own(&view), own(&again));
    assert_eq!(view.economy.gold, again.economy.gold);
    assert_eq!(view.research.known, again.research.known);
    assert_eq!(view.research.current, again.research.current);
    let cities = |v: &PlayerView| {
        v.cities
            .iter()
            .map(|c| (c.id, c.owner, c.pos, c.population))
            .collect::<Vec<_>>()
    };
    // Every city the seat knows is where it knew it; unseen rival cities are added.
    for c in cities(&view) {
        assert!(
            det.cities
                .values()
                .any(|d| (d.id, d.owner, d.pos, d.population) == c)
        );
    }
    for p in &view.players {
        assert_eq!(
            det.cities_of(p.id).count() as u32,
            p.cities,
            "public city count of {:?}",
            p.id
        );
    }
    // Same seed, same state.
    assert_eq!(determinize(&view, &prior, 17).hash(), det.hash());
}

#[test]
fn a_determinized_state_never_depends_on_hidden_state() {
    // Two true states that differ only where seat 0 can't see give the same determinization.
    let state = game_at(3_600_002, 25);
    let me = PlayerId(0);
    let view = player_view(&state, me);
    let mut hidden = state.clone();
    let rival = hidden
        .players
        .iter()
        .find(|p| p.team != view.team)
        .map(|p| p.id)
        .expect("rival");
    hidden.player_mut(rival).science_stored += 50;
    for c in hidden.cities.values_mut().filter(|c| c.owner == rival) {
        c.production_stored += 30;
    }
    assert_eq!(player_view(&hidden, me), view);
    let prior = prior_map(&view, 1);
    assert_eq!(
        determinize(&player_view(&hidden, me), &prior, 5).hash(),
        determinize(&view, &prior, 5).hash()
    );
}

#[test]
fn unseen_wartime_army_is_sampled_on_the_approach() {
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    let home = legatum_core::Hex::new(5, 8);
    let mut view = ViewBuilder::new(28, 18)
        .city(1, ME, home, 3)
        .city(2, ENEMY, legatum_core::Hex::new(18, 8), 3)
        .build();
    view.players[ME.0 as usize].cities = 1;
    view.players[ENEMY.0 as usize].cities = 1;
    crate::fixture::set_armies(&mut view, &[(ME, 0), (ENEMY, 80)]);
    for i in 0..view.tiles.len() {
        if i as i32 % view.width >= 8 {
            view.tiles[i].visible = false;
        }
    }
    let det = determinize(&view, &prior_map(&view, 17), 17);
    assert!(
        det.units
            .values()
            .any(|u| u.owner == ENEMY && u.pos.distance(home) <= 4),
        "an army already at war should have a forward fog prior"
    );
    let prior = prior_map(&view, 17);
    let legacy = determinize::determinize_with(&view, &prior, 17, 0, None);
    assert!(
        !legacy
            .units
            .values()
            .any(|u| u.owner == ENEMY && u.pos.distance(home) <= 4)
    );
    view.diplomacy.legacy = false;
    let watched = determinize::determinize_with(&view, &prior, 17, 70, Some(ENEMY));
    assert!(
        watched
            .units
            .values()
            .any(|u| u.owner == ENEMY && u.pos.distance(home) <= 4),
        "the top threat gets a forward prior even during Peace"
    );
}

#[test]
fn rollouts_replay_exactly_and_score_the_seat() {
    let state = game_at(3_600_003, 20);
    let me = PlayerId(1);
    let view = player_view(&state, me);
    let det = determinize(&view, &prior_map(&view, 0), 8);
    let brain =
        || ScriptedBrain::with_difficulty(state.player(me).faction, 4, me, AiDifficulty::Hard);
    let a = rollout(det.clone(), me, Box::new(brain()), 6, 99, 0);
    let b = rollout(det, me, Box::new(brain()), 6, 99, 0);
    assert_eq!(a, b);
    assert!((0..=10_000).contains(&a));
    // Before any turn is played, the score is the view's own estimate.
    assert_eq!(
        chance_in(&state, me),
        WinFacts::of(&view)
            .with_relative(&crate::DecisionParams::default())
            .search_chance()
    );
}

#[test]
fn search_options_parse() {
    let c = SearchConfig::parse("k=4,h=10,d=3").unwrap();
    assert_eq!(
        (c.candidates, c.horizon, c.samples, c.omniscient),
        (4, 10, 3, false)
    );
    assert!(SearchConfig::parse("omniscient").unwrap().omniscient);
    assert!(SearchConfig::parse("zz=1").is_err());
    let c = SearchConfig::parse("w,g,f,s,sh=6,x,xh=5,xc").unwrap();
    assert_eq!(c.label(), "k3h20d4-w-g-f-s6-x5c");
    assert_eq!(SearchConfig::default().label(), "k3h20d4");
    let c = SearchConfig::parse("h=end,o,oh=80,v=3,t").unwrap();
    assert_eq!(c.label(), "k3hendd4-o80-v3-t");
    assert_eq!(SearchConfig::parse("a=2").unwrap().label(), "k3h20d4-a2");
    assert_eq!(
        SearchConfig::parse("c=12,y").unwrap().label(),
        "k3h20d4-c12-y"
    );
}

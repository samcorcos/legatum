//! Goal-driven plans (`plan.rs`) and Settler risk, end to end where the rule lives in core.

use crate::fixture::{ENEMY, ME, ViewBuilder};
use crate::plan::{BlockReason, Plan, PlanEvent, PlanKind};
use crate::{Brain, Knobs, ScriptedBrain};
use legatum_core::state::GameSetup;
use legatum_core::{
    AiDifficulty, CityId, Event, GameState, Hex, Order, PlayerId, ProductionItem, UnitKind,
    game::spawn_unit, new_game, player_view, resolve_turn,
};
use std::collections::BTreeMap;

#[test]
fn hard_strikes_a_massing_field_army_before_it_declares() {
    use legatum_core::TeamId;
    use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
    let mut view = ViewBuilder::new(30, 18)
        .city(1, ME, Hex::new(5, 8), 3)
        .city(2, ENEMY, Hex::new(20, 8), 3)
        .unit(1, ME, UnitKind::Soldier, Hex::new(7, 8))
        .unit(2, ME, UnitKind::Soldier, Hex::new(7, 9))
        .unit(3, ME, UnitKind::Soldier, Hex::new(7, 7))
        .unit(50, ENEMY, UnitKind::Ranger, Hex::new(8, 8))
        .unit(51, ENEMY, UnitKind::Ranger, Hex::new(8, 9))
        .build();
    view.players[ME.0 as usize].cities = 1;
    view.players[ENEMY.0 as usize].cities = 1;
    view.diplomacy.legacy = false;
    view.diplomacy.relations.push(Relation {
        a: TeamId(0),
        b: TeamId(2),
        state: RelationState::Peace,
        effective_turn: 1,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: None,
        declarers: vec![],
    });
    view.diplomacy.actions.push(DiplomaticAction {
        new_wars: vec![],
        calls: vec![],
        also_agree: vec![],
        order: Order::DeclareWar {
            against: TeamId(2),
            breach: false,
        },
        effective_turn: view.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    });
    let mut brain = crate::fixture::brain(&view, 7, AiDifficulty::Hard);
    let orders = brain.plan(&view);
    assert!(
        orders.contains(&Order::DeclareWar {
            against: TeamId(2),
            breach: false
        }),
        "a locally superior army should preempt a massing force: {orders:?}"
    );
    assert!(
        orders.iter().any(|o| matches!(o, Order::MoveUnit { to, .. }
        if *to == Hex::new(8, 8) || *to == Hex::new(8, 9))),
        "the declaration must include a field attack: {orders:?}"
    );
    let mut legacy = crate::fixture::brain(&view, 7, AiDifficulty::Hard);
    legacy.decisions.field_strike = 0;
    let old = legacy.plan(&view);
    assert!(
        !old.iter()
            .any(|o| matches!(o, Order::DeclareWar { against, .. }
        if *against == TeamId(2))),
        "zero gene restores city-only plans: {old:?}"
    );
}

const RIVAL: PlayerId = PlayerId(1);

/// A few turns of an all-AI game, so capitals stand.
fn early_game(seed: u64) -> GameState {
    let mut state = {
        let mut setup = GameSetup::all_ai(seed);
        // These tactical fixtures keep their original roster as new factions join the lobby.
        let roster = [
            legatum_core::Faction::Switzerland,
            legatum_core::Faction::India,
            legatum_core::Faction::Byzantium,
            legatum_core::Faction::Germany,
            legatum_core::Faction::Celts,
            legatum_core::Faction::Mongols,
            legatum_core::Faction::Sparta,
            legatum_core::Faction::Athens,
        ];
        for (i, player) in setup.players.iter_mut().enumerate() {
            player.faction = roster[(seed as usize % roster.len() + i) % roster.len()];
        }
        new_game(&setup)
    };
    let mut brains: Vec<ScriptedBrain> = state
        .players
        .iter()
        .map(|p| ScriptedBrain::with_difficulty(p.faction, seed, p.id, AiDifficulty::Normal))
        .collect();
    for _ in 0..4 {
        let orders: BTreeMap<_, _> = brains
            .iter_mut()
            .map(|b| (b.player, b.orders(&player_view(&state, b.player))))
            .collect();
        state = resolve_turn(&state, &orders).0;
    }
    state
}

fn give_current_vp(state: &mut GameState, player: PlayerId, eras: u8) {
    let turn = state.turn;
    let p = state.player_mut(player);
    p.faction_goal_done = true;
    for era in 2..2 + eras {
        p.eras.insert(
            era,
            legatum_core::objectives::EraRecord {
                entered: turn,
                scored: Some(legatum_core::objectives::EraScore { turn, slot: 0 }),
                ..Default::default()
            },
        );
    }
    // Great Work VP is held by an owned landmark tile.
    let city = state.cities_of(player).next().expect("capital").id;
    state
        .map
        .tiles
        .iter_mut()
        .find(|tile| tile.owner == Some(player) && tile.landmark.is_none())
        .expect("owned tile")
        .landmark = Some(legatum_core::landmarks::Landmark {
        building: legatum_core::BuildingKind::RelayUplink,
        city,
        of: player,
    });
    state.player_mut(player).great_work_completed = Some(legatum_core::BuildingKind::RelayUplink);
}

/// An unowned land tile and a neighbour, far from every city and unit.
fn open_ground(state: &GameState) -> (Hex, Hex) {
    let clear = |h: Hex| {
        state.map.get(h).is_some_and(|t| {
            t.owner.is_none() && t.terrain.is_land() && t.terrain.move_cost().is_some()
        }) && state.cities.values().all(|c| c.pos.distance(h) > 4)
            && state.units.values().all(|u| u.pos.distance(h) > 4)
    };
    state
        .map
        .hexes()
        .filter(|h| clear(*h))
        .find_map(|h| {
            h.neighbors()
                .into_iter()
                .find(|n| clear(*n))
                .map(|n| (h, n))
        })
        .expect("open ground")
}

fn hard(state: &GameState, player: PlayerId) -> ScriptedBrain {
    ScriptedBrain::with_difficulty(
        state.player(player).faction,
        711,
        player,
        AiDifficulty::Hard,
    )
}

#[test]
fn hard_snipes_an_exposed_settler_with_a_surprise_war_and_then_seeks_peace() {
    let mut state = early_game(711);
    let (site, beside) = open_ground(&state);
    let settler = spawn_unit(&mut state, RIVAL, UnitKind::Settler, site);
    let soldier = spawn_unit(&mut state, ME, UnitKind::Soldier, beside);
    let rival = state.player(RIVAL).team;
    let view = player_view(&state, ME);
    assert!(!view.at_war(ME, RIVAL));
    let mut brain = hard(&state, ME);
    let orders = brain.orders(&view);
    assert!(
        orders.contains(&Order::DeclareWar {
            against: rival,
            breach: false
        }),
        "{orders:?}"
    );
    assert!(orders.contains(&Order::MoveUnit {
        unit: soldier,
        to: site
    }));
    let snipe = brain.plan_events().iter().rev().find_map(|e| match e {
        PlanEvent::Switch { to, terms, .. } => Some((to.clone(), terms.clone())),
        PlanEvent::BlockedExpense { .. } => None,
    });
    let (kind, terms) = snipe.expect("a plan switch");
    assert!(matches!(kind, PlanKind::Snipe { settler: s, .. } if s == settler));
    assert!(
        terms
            .iter()
            .any(|(name, v)| name == "a city and its denial" && *v > 0)
    );
    // Core resolves the surprise war before movement: the step pins and captures it.
    let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, orders)]));
    assert!(events.iter().any(
        |e| matches!(e, Event::SettlerCaptured { unit, to, .. } if *unit == settler && *to == ME)
    ));
    assert_eq!(next.units[&settler].owner, ME);
    // The next plan ends the snipe for its reason and wants peace with the victim.
    let view = player_view(&next, ME);
    let _ = brain.orders(&view);
    let last = brain.plan_events().last().cloned();
    assert!(
        matches!(
            last,
            Some(PlanEvent::Switch {
                reason: crate::plan::PlanReason::SettlerTaken,
                ..
            })
        ),
        "{last:?}"
    );
}

#[test]
fn a_guarded_settler_is_not_sniped_and_normal_difficulty_never_plans() {
    let mut state = early_game(711);
    let (site, beside) = open_ground(&state);
    spawn_unit(&mut state, RIVAL, UnitKind::Settler, site);
    spawn_unit(&mut state, ME, UnitKind::Soldier, beside);
    let guard = site
        .neighbors()
        .into_iter()
        .find(|h| *h != beside && state.map.get(*h).is_some_and(|t| t.terrain.is_land()))
        .unwrap();
    let unguarded = state.clone();
    spawn_unit(&mut state, RIVAL, UnitKind::Soldier, guard);
    let orders = hard(&state, ME).orders(&player_view(&state, ME));
    assert!(!orders.iter().any(|o| matches!(o, Order::DeclareWar { .. })));
    let mut normal =
        ScriptedBrain::with_difficulty(unguarded.player(ME).faction, 711, ME, AiDifficulty::Normal);
    let orders = normal.orders(&player_view(&unguarded, ME));
    assert!(!orders.iter().any(|o| matches!(o, Order::DeclareWar { .. })));
    assert!(normal.plan_events().is_empty());
}

#[test]
fn plan_memory_defaults_in_old_saves_and_round_trips() {
    let old: crate::CampaignMemory =
        serde_json::from_str(r#"{"active":null,"paused":null,"held_capture":null,"events":[]}"#)
            .unwrap();
    assert!(old.plan.active.is_none() && old.plan.events.is_empty());
    let mut memory = crate::CampaignMemory::default();
    memory.plan.active = Some(Plan {
        kind: PlanKind::Campaign {
            against: legatum_core::TeamId(2),
            target: CityId(9),
        },
        start_turn: 4,
        review_turn: 14,
        reserve: 30,
        value: 120,
        terms: vec![("capture at 70%".into(), 150)],
        breach: false,
        misses: 0,
    });
    memory.plan.events.push(PlanEvent::BlockedExpense {
        turn: 4,
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Soldier,
        },
        cost: 40,
        reason: BlockReason::PlanReserve,
    });
    let json = serde_json::to_string(&memory).unwrap();
    assert!(json.contains(r#""kind":"blocked_expense""#));
    assert_eq!(
        serde_json::from_str::<crate::CampaignMemory>(&json).unwrap(),
        memory
    );
}

#[test]
fn the_plan_reserve_withdraws_discretionary_purchases_and_logs_them() {
    let item = ProductionItem::Unit {
        unit: UnitKind::Soldier,
    };
    let view = ViewBuilder::new(20, 12)
        .city_with_detail(1, ME, Hex::new(4, 5), 3)
        .producing(1, item, 40)
        .gold(60)
        .build();
    let mut p = crate::scripted::Planner::new(
        &view,
        Knobs::for_faction(view.faction_of(ME)),
        view.faction_of(ME),
        7,
    );
    p.plan_routing = true;
    p.campaigns.plan.active = Some(Plan {
        kind: PlanKind::Economy,
        start_turn: 1,
        review_turn: 10,
        reserve: 50,
        value: 0,
        terms: Vec::new(),
        breach: false,
        misses: 0,
    });
    let buy = Order::BuyItem {
        city: CityId(1),
        item,
    };
    p.orders.push(buy.clone());
    p.gold_left = 20;
    // Emergency defense is protected; an ordinary purchase is not.
    p.veto_plan_reserve(std::slice::from_ref(&buy));
    assert!(p.orders.contains(&buy));
    p.veto_plan_reserve(&[]);
    assert!(!p.orders.contains(&buy));
    assert_eq!(p.gold_left, 60);
    assert_eq!(p.campaigns.plan.blocked_expenses, 1);
    assert!(matches!(
        p.campaigns.plan.events[..],
        [PlanEvent::BlockedExpense {
            reason: BlockReason::PlanReserve,
            cost: 40,
            ..
        }]
    ));
}

#[test]
fn settler_trip_risk_prices_who_can_reach_the_route() {
    let from = Hex::new(4, 6);
    let to = Hex::new(12, 6);
    let base = ViewBuilder::new(30, 14)
        .city_with_detail(1, ME, Hex::new(2, 6), 3)
        .unit(1, ME, UnitKind::Settler, from);
    let quiet = base.build();
    let mut p = crate::scripted::Planner::new(
        &quiet,
        Knobs::for_faction(quiet.faction_of(ME)),
        quiet.faction_of(ME),
        7,
    );
    p.difficulty = AiDifficulty::Hard;
    assert_eq!(p.settler_trip_risk(from, to), 0);
    // An enemy (legacy fixtures are at war with everyone) beside the route prices it high;
    // the same unit far off the route doesn't count.
    let near = ViewBuilder::new(30, 14)
        .city_with_detail(1, ME, Hex::new(2, 6), 3)
        .unit(1, ME, UnitKind::Settler, from)
        .unit(50, ENEMY, UnitKind::Soldier, Hex::new(8, 9))
        .build();
    let mut p = crate::scripted::Planner::new(
        &near,
        Knobs::for_faction(near.faction_of(ME)),
        near.faction_of(ME),
        7,
    );
    p.difficulty = AiDifficulty::Hard;
    assert!(p.settler_trip_risk(from, to) >= p.decisions.settler_risk_escort);
    let far = ViewBuilder::new(30, 14)
        .city_with_detail(1, ME, Hex::new(2, 6), 3)
        .unit(1, ME, UnitKind::Settler, from)
        .unit(50, ENEMY, UnitKind::Soldier, Hex::new(25, 12))
        .build();
    let mut p = crate::scripted::Planner::new(
        &far,
        Knobs::for_faction(far.faction_of(ME)),
        far.faction_of(ME),
        7,
    );
    p.difficulty = AiDifficulty::Hard;
    assert_eq!(p.settler_trip_risk(from, to), 0);
}

#[test]
fn settler_trip_risk_counts_a_distant_army_that_can_intercept_before_founding() {
    let from = Hex::new(4, 6);
    let to = Hex::new(16, 6);
    let view = ViewBuilder::new(30, 20)
        .city_with_detail(1, ME, Hex::new(2, 6), 3)
        .unit(1, ME, UnitKind::Settler, from)
        .unit(50, ENEMY, UnitKind::Soldier, Hex::new(16, 13))
        .build();
    let mut p = crate::scripted::Planner::new(
        &view,
        Knobs::for_faction(view.faction_of(ME)),
        view.faction_of(ME),
        7,
    );
    p.difficulty = AiDifficulty::Hard;
    p.decisions.settler_safety = 1;
    assert!(p.settler_trip_risk(from, to) > 0);
    assert_eq!(p.settler_trip_risk(from, from), 0);
}

#[test]
fn remote_peaceful_scout_makes_an_unescorted_settler_detour() {
    use legatum_core::diplomacy::DiplomaticAction;

    let from = Hex::new(4, 12);
    let to = Hex::new(10, 12);
    let raider = Hex::new(9, 12);
    let mut view = ViewBuilder::new(32, 24)
        .city(1, ME, Hex::new(2, 8), 3)
        .city(2, ENEMY, Hex::new(26, 18), 3)
        .unit(1, ME, UnitKind::Settler, from)
        .unit(50, ENEMY, UnitKind::Scout, raider)
        .build();
    view.diplomacy.legacy = false;
    view.players
        .iter_mut()
        .find(|p| p.id == ENEMY)
        .unwrap()
        .is_human = true;
    let rival_team = view.team_of(ENEMY);
    view.diplomacy.actions.push(DiplomaticAction {
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        order: Order::DeclareWar {
            against: rival_team,
            breach: false,
        },
        effective_turn: view.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    });
    let mut planner = crate::scripted::Planner::new(
        &view,
        Knobs::for_faction(view.faction_of(ME)),
        view.faction_of(ME),
        7,
    );
    planner.difficulty = AiDifficulty::Hard;
    planner.decisions.peace_settler_capture = 100;
    assert!(planner.settler_safety());
    let remote_chance = planner.peace_settler_capture_chance(ENEMY);
    assert!(remote_chance >= planner.decisions.settler_risk_escort);
    assert!(planner.settler_trip_risk(from, to) >= planner.decisions.settler_risk_escort);
    planner.prepare_hard_defense();
    planner.move_settler(&view.units[0], to);
    let arrival = planner.settler_arrival(&view.units[0]).unwrap_or(from);
    assert!(
        !planner.settler_capturable_this_turn(arrival),
        "unescorted arrival {arrival:?} in capture reach: {:?}",
        planner.orders
    );

    planner.decisions.peace_settler_capture = 0;
    assert!(!planner.settler_safety());
    assert_eq!(planner.settler_trip_risk(from, to), 50);

    let mut neighbor = view.clone();
    neighbor
        .cities
        .iter_mut()
        .find(|c| c.owner == ENEMY)
        .unwrap()
        .pos = Hex::new(8, 8);
    let neighbor_planner = crate::scripted::Planner::new(
        &neighbor,
        Knobs::for_faction(neighbor.faction_of(ME)),
        neighbor.faction_of(ME),
        7,
    );
    assert!(neighbor_planner.peace_settler_capture_chance(ENEMY) < remote_chance);
}

#[test]
fn settler_trip_risk_reads_known_rival_border_without_a_visible_unit() {
    let from = Hex::new(4, 6);
    let view = ViewBuilder::new(30, 20)
        .city_with_detail(1, ME, Hex::new(2, 6), 3)
        .owner(Hex::new(10, 8), ENEMY)
        .unit(1, ME, UnitKind::Settler, from)
        .build();
    let mut p = crate::scripted::Planner::new(
        &view,
        Knobs::for_faction(view.faction_of(ME)),
        view.faction_of(ME),
        7,
    );
    p.difficulty = AiDifficulty::Hard;
    p.decisions.settler_safety = 1;
    assert_eq!(p.settler_trip_risk(from, Hex::new(12, 6)), 4);
    assert_eq!(p.settler_trip_risk(from, Hex::new(4, 2)), 0);
}

/// A Hold plan (Hard+Search) on an empty, threatened city: the nearest unit that can arrive
/// this turn moves in; with none in reach, a defender is bought.
#[test]
fn a_hold_plan_garrisons_its_city_by_moving_or_buying() {
    let home = Hex::new(8, 6);
    let hold = |v: &legatum_core::PlayerView| {
        let mut p = crate::scripted::Planner::new(
            v,
            Knobs::for_faction(v.faction_of(ME)),
            v.faction_of(ME),
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.plan_routing = true;
        p.campaigns.plan.active = Some(Plan {
            kind: PlanKind::Hold { city: CityId(1) },
            start_turn: v.turn,
            review_turn: v.turn + 10,
            reserve: 0,
            value: 0,
            terms: Vec::new(),
            breach: false,
            misses: 0,
        });
        p.execute_hold();
        p.orders
    };
    let v = ViewBuilder::new(20, 14)
        .city(1, ME, home, 3)
        .unit(5, ME, UnitKind::Soldier, Hex::new(9, 6))
        .unit(9, ENEMY, UnitKind::Soldier, Hex::new(11, 6))
        .build();
    assert!(
        hold(&v)
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit, to } if unit.0 == 5 && *to == home)),
        "{:?}",
        hold(&v)
    );
    let mut far = ViewBuilder::new(20, 14)
        .city(1, ME, home, 3)
        .unit(9, ENEMY, UnitKind::Soldier, Hex::new(11, 6))
        .build();
    far.economy.gold = 500;
    for o in &mut far.cities[0].detail.as_mut().unwrap().options {
        if matches!(o.item, ProductionItem::Unit { .. }) {
            o.buy_cost = Some(2 * o.cost);
        }
    }
    let orders = hold(&far);
    assert!(
        orders.iter().any(|o| matches!(
            o,
            Order::BuyItem {
                city: CityId(1),
                item: ProductionItem::Unit { .. }
            }
        )),
        "{orders:?}"
    );
}

#[test]
fn hard_goes_to_war_against_a_rival_counting_down_to_victory() {
    let mut state = early_game(711);
    let rival = state.player(RIVAL).team;
    // Our army stands near the rival's capital, so a war can reach it.
    let capital = state.cities_of(RIVAL).next().expect("rival capital").pos;
    let near: Vec<Hex> = capital
        .ring(2)
        .into_iter()
        .filter(|h| {
            state
                .map
                .get(*h)
                .is_some_and(|t| t.terrain.is_land() && t.terrain.move_cost().is_some())
                && state.units.values().all(|u| u.pos != *h)
        })
        .take(3)
        .collect();
    for h in &near {
        spawn_unit(&mut state, ME, UnitKind::Soldier, *h);
    }
    // One quiet turn, so our soldiers have seen the capital.
    state = resolve_turn(&state, &BTreeMap::new()).0;
    let declares = |state: &GameState| {
        let view = player_view(state, ME);
        let mut brain = hard(state, ME);
        let orders = brain.orders(&view);
        orders.contains(&Order::DeclareWar {
            against: rival,
            breach: false,
        })
    };
    assert!(!declares(&state), "no reason for war at turn 5");
    // Six era points, the Great Work and its goal reach the current target.
    give_current_vp(&mut state, RIVAL, 6);
    assert_eq!(legatum_core::scoring::team_vp(&state, rival), 10);
    state.vp_holds.insert(rival, 2);
    assert!(
        declares(&state),
        "a holder two turns into its countdown is fought"
    );
}

/// Playtest 8.4: a 7-VP leader was never fought before its hold, and a containment war did
/// nothing. Hard now contains the strict leader at 70% of its target and marches on it.
#[test]
fn hard_contains_a_runaway_leader_before_its_hold_and_marches_on_it() {
    let mut state = early_game(715);
    let rival = state.player(RIVAL).team;
    let capital = state.cities_of(RIVAL).next().expect("rival capital").pos;
    let open = |state: &GameState, h: Hex| {
        state.map.get(h).is_some_and(|t| {
            t.terrain.is_land() && t.terrain.move_cost().is_some() && t.owner.is_none()
        }) && state.units.values().all(|u| u.pos != h)
            && state.cities.values().all(|c| c.pos.distance(h) > 1)
    };
    // A Scout that has seen the capital, and a field army six hexes out.
    let scout_at = capital
        .ring(2)
        .into_iter()
        .find(|h| open(&state, *h))
        .expect("scout tile");
    spawn_unit(&mut state, ME, UnitKind::Scout, scout_at);
    let army: Vec<Hex> = capital
        .ring(7)
        .into_iter()
        .filter(|h| open(&state, *h))
        .take(3)
        .collect();
    assert_eq!(army.len(), 3);
    for h in &army {
        spawn_unit(&mut state, ME, UnitKind::Soldier, *h);
    }
    state = resolve_turn(&state, &BTreeMap::new()).0;
    // The rival leads on 7 of 10: three era points, Great Work and its goal.
    give_current_vp(&mut state, RIVAL, 3);
    assert_eq!(legatum_core::scoring::team_vp(&state, rival), 7);
    // A treasury that funds the war.
    state.player_mut(ME).gold = 400;
    let declaration = Order::DeclareWar {
        against: rival,
        breach: false,
    };
    let mut off = hard(&state, ME);
    off.decisions.contain_leader_percent = 0;
    off.decisions.contain_share = 0;
    off.decisions.vp_race = 0;
    off.decisions.vp_denial = 0;
    off.decisions.vp_hold_break = 0;
    off.decisions.vp_peace = 0;
    off.decisions.vp_two_step = 0;
    assert!(
        !off.orders(&player_view(&state, ME)).contains(&declaration),
        "the old pricing leaves a 7-VP leader alone"
    );
    let mut brain = hard(&state, ME);
    // The fixture's Scout is the army's eyes on the leader's capital. With `scout_find` it would
    // go exploring for the three rivals this four-empire lobby has not met, and the army would
    // march on a city it no longer sees; this test is about the containment war.
    brain.decisions.scout_find = 0;
    let orders = brain.orders(&player_view(&state, ME));
    assert!(
        orders.contains(&declaration),
        "{orders:?} {:?}",
        brain.plan_events()
    );
    assert!(matches!(
        brain.plan_events().last(),
        Some(PlanEvent::Switch {
            to: PlanKind::Contain { against },
            ..
        }) if *against == rival
    ));
    // At war, the field army marches on the leader's city.
    let mut orders_map = BTreeMap::new();
    orders_map.insert(ME, orders);
    state = resolve_turn(&state, &orders_map).0;
    assert!(state.teams_at_war(state.player(ME).team, rival));
    let orders = brain.orders(&player_view(&state, ME));
    let closer = orders
        .iter()
        .filter(|o| match o {
            Order::MoveUnit { unit, to } => state.units.get(unit).is_some_and(|u| {
                u.kind == UnitKind::Soldier && to.distance(capital) < u.pos.distance(capital)
            }),
            _ => false,
        })
        .count();
    // The new seeded map can put two Soldiers behind terrain while one still advances.
    assert!(closer >= 1, "the army marches: {orders:?}");
}

#[test]
fn era_terms_reward_a_lead_and_charge_a_deficit_toward_a_war_target() {
    use legatum_core::TeamId;
    let rival_team = TeamId(2);
    let terms = |my_era: u8, their_era: u8, first: bool, deficit: bool| {
        let mut view = ViewBuilder::new(16, 16)
            .city(1, ME, Hex::from_offset(4, 4), 3)
            .era_of(2, their_era)
            .build();
        view.research.era = my_era;
        if first {
            view.research.era_firsts.push((my_era, ME));
        }
        let p = crate::scripted::Planner::new(
            &view,
            Knobs::for_faction(legatum_core::Faction::Switzerland),
            legatum_core::Faction::Switzerland,
            1,
        );
        let mut terms = Vec::new();
        p.era_terms(rival_team, deficit, &mut terms);
        terms
    };
    // Level: nothing.
    assert!(terms(2, 2, false, true).is_empty());
    // One era ahead is a window; the first into the era is worth a little more.
    let ahead = terms(2, 1, false, true);
    let first = terms(2, 1, true, true);
    assert_eq!(ahead[0].0, "era lead");
    assert!(
        ahead[0].1 > 0 && first[0].1 > ahead[0].1,
        "{ahead:?} {first:?}"
    );
    // The lead is capped at two eras.
    assert_eq!(terms(4, 0, false, true)[0].1, terms(2, 0, false, true)[0].1);
    // Behind is a threat for a campaign but not for containment.
    let behind = terms(1, 3, false, true);
    assert_eq!(behind[0].0, "era deficit");
    assert!(behind[0].1 < 0);
    assert!(terms(1, 3, false, false).is_empty());
}

/// Idea 24d: under VP v2 population and territory are 0 for every team, so a projection that
/// moves them for the capturer alone handed it a 100% share of both. With `extent_guard` a
/// lost city moves only the city count, and the swing shrinks accordingly.
#[test]
fn extent_guard_stops_a_lost_city_swinging_unread_facts() {
    use legatum_core::{Faction, TeamId};
    let view = ViewBuilder::new(16, 16)
        .city(1, ME, Hex::from_offset(4, 4), 3)
        .city(2, ME, Hex::from_offset(6, 4), 2)
        .city(3, ENEMY, Hex::from_offset(12, 4), 4)
        .build();
    // Isolate the guard's behavior on the older public-information view, where neither
    // population nor territory could be read. Current WinFacts reconstructs both from
    // observed cities and tiles.
    let mut facts = crate::winchance::WinFacts::of(&view).refined();
    for team in &mut facts.teams {
        team.population = 0;
        team.territory = 0;
    }
    assert!(!facts.extent_known());
    let swing = |guard: i32| {
        let mut p = crate::scripted::Planner::new(
            &view,
            Knobs::for_faction(Faction::Mongols),
            Faction::Mongols,
            1,
        );
        p.decisions.extent_guard = guard;
        let lost = p.city_lost(&facts, TeamId(2));
        facts.chance() - lost.chance()
    };
    let (old, guarded) = (swing(0), swing(1));
    assert!(
        guarded > 0,
        "losing a city still costs something: {guarded}"
    );
    assert!(
        guarded < old,
        "the unread facts no longer inflate the loss: {old} -> {guarded}"
    );
}

/// Idea 24a, 24b: a rival city out of sight for longer than the gene's turns is stale (and
/// priced aged); one in sight, one not yet old, and one first noted just now are not.
#[test]
fn a_rival_city_not_seen_for_a_long_time_is_stale_and_priced_aged() {
    use legatum_core::Faction;
    let mut view = ViewBuilder::new(16, 16)
        .city(1, ME, Hex::from_offset(4, 4), 3)
        .city(2, ENEMY, Hex::from_offset(12, 4), 3)
        .city(3, ENEMY, Hex::from_offset(12, 8), 3)
        .city(4, ENEMY, Hex::from_offset(12, 12), 3)
        .era_of(2, 4)
        .build();
    view.turn = 120;
    for id in [2, 3] {
        view.cities
            .iter_mut()
            .find(|c| c.id == CityId(id))
            .unwrap()
            .visible = false;
    }
    let mut p = crate::scripted::Planner::new(
        &view,
        Knobs::for_faction(Faction::Mongols),
        Faction::Mongols,
        1,
    );
    p.decisions.stale_intel = 15;
    p.campaigns.city_seen.insert(CityId(2), 48);
    p.campaigns.city_seen.insert(CityId(3), 112);
    let city = |id: u32| view.cities.iter().find(|c| c.id == CityId(id)).unwrap();
    assert!(p.stale_city(city(2)), "seen 72 turns ago");
    assert!(!p.stale_city(city(3)), "seen 8 turns ago");
    assert!(!p.stale_city(city(4)), "in sight");
    let aged = p.aged_city(city(2)).expect("stale");
    assert!(aged.walls && aged.population > 3 && aged.strength > city(2).strength);
    assert!(p.aged_city(city(3)).is_none());
    p.decisions.stale_intel = 0;
    assert!(!p.stale_city(city(2)), "the gene is off");
}

/// Idea 24a: a new war on a stale picture is not offered; the same city in sight is.
#[test]
fn no_new_campaign_is_offered_on_a_stale_picture() {
    use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
    use legatum_core::{Faction, TeamId};
    let near = Hex::from_offset(8, 4);
    let build = |visible: bool| {
        let mut view = ViewBuilder::new(16, 16)
            .city(1, ME, Hex::from_offset(4, 4), 3)
            .city(2, ENEMY, near, 3)
            .unit(10, ME, UnitKind::Soldier, Hex::from_offset(6, 4))
            .unit(11, ME, UnitKind::Soldier, Hex::from_offset(6, 5))
            .unit(12, ME, UnitKind::Ranger, Hex::from_offset(5, 4))
            .build();
        view.turn = 120;
        view.diplomacy.legacy = false;
        view.diplomacy.relations.push(Relation {
            a: TeamId(0),
            b: TeamId(2),
            state: RelationState::Peace,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: Vec::new(),
        });
        view.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against: TeamId(2),
                breach: false,
            },
            effective_turn: view.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        view.cities
            .iter_mut()
            .find(|c| c.id == CityId(2))
            .unwrap()
            .visible = visible;
        view
    };
    let outlook = |view: &legatum_core::PlayerView, gene: i32| {
        let mut p = crate::scripted::Planner::new(
            view,
            Knobs::for_faction(Faction::Mongols),
            Faction::Mongols,
            1,
        );
        p.decisions.stale_intel = gene;
        p.campaigns.city_seen.insert(CityId(2), 48);
        let city = view.cities.iter().find(|c| c.id == CityId(2)).unwrap();
        let _ = TeamId(2);
        p.campaign_outlook(city).is_some()
    };
    let hidden = build(false);
    let seen = build(true);
    assert!(outlook(&seen, 0), "the fixture offers a campaign in sight");
    assert_eq!(
        outlook(&seen, 15),
        outlook(&seen, 0),
        "in sight it is priced as before"
    );
    assert!(
        !outlook(&hidden, 15),
        "a 72-turn-old picture cannot open a war"
    );
}

/// Gene `settler_stalk`: a Scout closes on an exposed rival Settler at Peace that is beyond
/// this turn's snipe reach, without declaring; at 0 it does not.
#[test]
fn a_scout_stalks_an_exposed_rival_settler_without_declaring() {
    use legatum_core::diplomacy::DiplomaticAction;

    let mut view = ViewBuilder::new(32, 24)
        .city(1, ME, Hex::new(2, 8), 3)
        .city(2, ENEMY, Hex::new(26, 18), 3)
        .unit(1, ME, UnitKind::Scout, Hex::new(6, 8))
        .unit(50, ENEMY, UnitKind::Settler, Hex::new(13, 8))
        .build();
    view.diplomacy.legacy = false;
    let rival_team = view.team_of(ENEMY);
    view.diplomacy.actions.push(DiplomaticAction {
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        order: Order::DeclareWar {
            against: rival_team,
            breach: false,
        },
        effective_turn: view.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    });
    for gene in [0, 3] {
        let mut p = crate::scripted::Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(ME)),
            view.faction_of(ME),
            1,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.settler_stalk = gene;
        p.stalk_settlers();
        let moved = p.orders.iter().find_map(|o| match o {
            Order::MoveUnit { unit, to } if *unit == legatum_core::UnitId(1) => Some(*to),
            _ => None,
        });
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::DeclareWar { .. }))
        );
        if gene == 0 {
            assert_eq!(moved, None);
        } else {
            assert_eq!(moved.map(|to| to.distance(Hex::new(13, 8))), Some(1));
        }
    }
}

/// A view where ME may declare a surprise war on ENEMY (the snipe and stalk tests).
fn declarable(mut view: legatum_core::PlayerView) -> legatum_core::PlayerView {
    use legatum_core::diplomacy::DiplomaticAction;
    view.diplomacy.legacy = false;
    let rival_team = view.team_of(ENEMY);
    view.diplomacy.actions.push(DiplomaticAction {
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        order: Order::DeclareWar {
            against: rival_team,
            breach: false,
        },
        effective_turn: view.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    });
    view
}

/// Gene `stalk_contact` (8.9.24 Swiss review: Hard declared with a Scout three hexes off and
/// took nothing): the Snipe pairs a Settler only with a unit beside it; at 0 the Scout two
/// moves away is paired.
#[test]
fn a_snipe_declares_only_from_the_next_hex() {
    let far = declarable(
        ViewBuilder::new(32, 24)
            .city(1, ME, Hex::new(2, 8), 3)
            .city(2, ENEMY, Hex::new(26, 18), 3)
            .unit(1, ME, UnitKind::Scout, Hex::new(10, 8))
            .unit(50, ENEMY, UnitKind::Settler, Hex::new(13, 8))
            .build(),
    );
    let near = declarable(
        ViewBuilder::new(32, 24)
            .city(1, ME, Hex::new(2, 8), 3)
            .city(2, ENEMY, Hex::new(26, 18), 3)
            .unit(1, ME, UnitKind::Scout, Hex::new(10, 8))
            .unit(2, ME, UnitKind::Soldier, Hex::new(12, 8))
            .unit(50, ENEMY, UnitKind::Settler, Hex::new(13, 8))
            .build(),
    );
    let pair = |view: &legatum_core::PlayerView, contact: i32| {
        let mut p = crate::scripted::Planner::new(
            view,
            Knobs::for_faction(view.faction_of(ME)),
            view.faction_of(ME),
            1,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.stalk_contact = contact;
        p.snipe_targets().first().map(|(u, _)| u.id.0)
    };
    assert_eq!(pair(&far, 0), Some(1));
    assert_eq!(pair(&far, 1), None);
    assert_eq!(pair(&near, 1), Some(2));
}

/// Gene `stalk_contact`: a quiet city's garrison stalks a Settler it reaches this turn, onto
/// the side of it away from the owner's city; at 0 the garrison stays.
#[test]
fn a_quiet_garrison_stalks_a_settler_from_ahead() {
    let settler = Hex::new(12, 8);
    let view = declarable(
        ViewBuilder::new(32, 24)
            .city(1, ME, Hex::new(12, 11), 3)
            .city(2, ENEMY, Hex::new(6, 8), 3)
            .unit(1, ME, UnitKind::Soldier, Hex::new(12, 11))
            .unit(50, ENEMY, UnitKind::Settler, settler)
            .build(),
    );
    for contact in [0, 1] {
        let mut p = crate::scripted::Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(ME)),
            view.faction_of(ME),
            1,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.stalk_contact = contact;
        p.stalk_settlers();
        let moved = p.orders.iter().find_map(|o| match o {
            Order::MoveUnit { unit, to } if *unit == legatum_core::UnitId(1) => Some(*to),
            _ => None,
        });
        if contact == 0 {
            assert_eq!(moved, None);
        } else {
            let to = moved.expect("the garrison stalks");
            assert_eq!(to.distance(settler), 1);
            assert!(
                to.distance(Hex::new(6, 8)) > settler.distance(Hex::new(6, 8)),
                "{to:?} is ahead of the Settler"
            );
        }
    }
}

#[test]
fn capture_horizon_slows_only_a_walled_siege_without_siege() {
    // Gene `capture_horizon` (Lab Ride to Dyrrachium): a walled city's siege rounds count at
    // the gene's percent when the force has no Siege; an open city's forecast stands.
    let view = ViewBuilder::new(20, 12)
        .city(1, ME, Hex::new(2, 5), 3)
        .city(2, ENEMY, Hex::new(10, 5), 3)
        .city(3, ENEMY, Hex::new(14, 5), 3)
        .walled(3)
        .unit(10, ME, UnitKind::Horseman, Hex::new(9, 5))
        .build();
    let f = crate::plan::Forecast {
        turns: 5,
        losses: 0,
        force: vec![legatum_core::UnitId(10)],
        march: 1,
    };
    let open = view.cities.iter().find(|c| c.id == CityId(2)).unwrap();
    let walled = view.cities.iter().find(|c| c.id == CityId(3)).unwrap();
    let mut p = crate::fixture::planner(&view, 1);
    p.decisions.siege_for_walls = 0;
    assert_eq!(
        p.capture_turns(walled, &f),
        5,
        "gene off: the forecast's turns"
    );
    p.decisions.capture_horizon = 200;
    assert_eq!(
        p.capture_turns(open, &f),
        5,
        "an open city keeps its forecast"
    );
    assert_eq!(
        p.capture_turns(walled, &f),
        1 + 4 * 2,
        "the march stands, the four siege rounds count double"
    );
}

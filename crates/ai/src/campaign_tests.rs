use super::*;
use crate::Knobs;
use crate::campaign::{Campaign, CampaignEnd};
use crate::fixture::{ENEMY, ME, ViewBuilder};
use legatum_core::state::ThreatObservation;
use legatum_core::{AiDifficulty, CityId, PlayerView, UnitId};

const TARGET: Hex = Hex::new(14, 8);

fn fogged_campaign() -> PlayerView {
    let mut view = ViewBuilder::new(30, 16)
        .city(1, ME, Hex::new(3, 8), 3)
        .city(9, ENEMY, TARGET, 2)
        .unit(1, ME, UnitKind::Soldier, Hex::new(6, 8))
        .unit(2, ME, UnitKind::Soldier, Hex::new(6, 9))
        .build();
    for h in TARGET.within(3) {
        if let Some(i) = view.index(h) {
            view.tiles[i].visible = false;
        }
    }
    view.cities
        .iter_mut()
        .find(|c| c.pos == TARGET)
        .unwrap()
        .visible = false;
    view
}

fn sighting(view: &PlayerView, pos: Hex, kind: UnitKind) -> ThreatObservation {
    ThreatObservation {
        unit: UnitId(50),
        owner: ENEMY,
        pos,
        kind,
        seen_turn: view.turn - 1,
    }
}

fn planner(view: &PlayerView) -> Planner<'_> {
    let mut p = crate::fixture::planner(view, 7);
    p.knobs = Knobs::new(50, 50, 50, 50, 100);
    p.personality_aggression = 100;
    if view.faction_of(ME) == Faction::Sparta {
        p.decisions.walled_prize = 1;
        // The legacy all-war fixture has a third, cityless opponent; that dead war must not
        // veto a forecast against the city these tests actually put on the map.
        p.decisions.march_ignores_dead_war = 1;
    }
    p
}

fn sparta_wall_force(count: usize) -> PlayerView {
    let city = Hex::new(14, 8);
    let mut b = ViewBuilder::new(30, 16)
        .faction(Faction::Sparta)
        .city(1, ME, Hex::new(7, 8), 3)
        .city(9, ENEMY, city, 3)
        .walled(9);
    for (i, pos) in city
        .ring(1)
        .into_iter()
        .chain(city.ring(2))
        .take(count)
        .enumerate()
    {
        b = b.unit(i as u32 + 1, ME, UnitKind::Soldier, pos);
    }
    b.build()
}

#[test]
fn hard_sparta_forecasts_a_concentrated_melee_wall_assault() {
    let small = sparta_wall_force(2);
    let massed = sparta_wall_force(7);
    let mut few = planner(&small);
    few.difficulty = AiDifficulty::Hard;
    let mut many = planner(&massed);
    many.difficulty = AiDifficulty::Hard;
    many.decisions.sparta_capture_heal_forecast = 0;
    let city = &massed.cities[1];
    assert!(many.sparta_walled_prize(city));
    assert!(
        few.campaign_siege_estimate(&small.cities[1], &few.campaign_force(&small.cities[1]))
            .is_none()
    );
    let ids = many.campaign_force(city);
    assert_eq!(
        ids.len(),
        6,
        "the six assault hexes are filled; the seventh stays in reserve"
    );
    let (_, expected_losses) = many
        .campaign_siege_estimate(city, &ids)
        .expect("a massed melee force can pay through Walls");
    assert!(
        expected_losses > 0,
        "the capture forecast must price attrition"
    );
}

#[test]
fn hard_sparta_counts_a_two_turn_reinforcement_before_declaring_on_walls() {
    let far = Hex::new(19, 8);
    let mut view = ViewBuilder::new(30, 16)
        .faction(Faction::Sparta)
        .city(1, ME, Hex::new(6, 8), 3)
        .city(9, ENEMY, TARGET, 3)
        .walled(9)
        .unit(1, ME, UnitKind::Soldier, TARGET.neighbor(3))
        .unit(2, ME, UnitKind::Soldier, far)
        .build();
    view.diplomacy.legacy = false;
    let enemy_team = view.team_of(ENEMY);
    view.diplomacy
        .relations
        .push(legatum_core::diplomacy::Relation {
            a: view.team,
            b: enemy_team,
            state: legatum_core::diplomacy::RelationState::Peace,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: Vec::new(),
        });
    let mut p = planner(&view);
    p.difficulty = AiDifficulty::Hard;
    assert!(
        p.campaign_force(&view.cities[1]).contains(&UnitId(2)),
        "a peace-time wall forecast includes the reachable reserve outside four hexes"
    );
}

#[test]
fn hard_sparta_prices_the_committed_march_instead_of_rejecting_a_distant_wall() {
    let mut b = ViewBuilder::new(30, 16)
        .faction(Faction::Sparta)
        .city(1, ME, Hex::new(6, 8), 3)
        .city(9, ENEMY, TARGET, 3)
        .walled(9);
    for (i, pos) in TARGET.ring(4).into_iter().take(7).enumerate() {
        b = b.unit(i as u32 + 1, ME, UnitKind::Soldier, pos);
    }
    let view = b.build();
    let mut p = planner(&view);
    p.difficulty = AiDifficulty::Hard;
    let city = &view.cities[1];
    let ids = p.campaign_force(city);
    assert!(ids.len() >= 6, "the march can concentrate the field army");
    assert!(
        p.campaign_siege_estimate(city, &ids).is_some(),
        "the campaign gate uses the field-force siege forecast"
    );
}

#[test]
fn hard_sparta_aborts_when_early_walls_break_the_siege_forecast() {
    let city = Hex::new(14, 8);
    let mut view = sparta_wall_force(2);
    for (u, pos) in view
        .units
        .iter_mut()
        .filter(|u| u.owner == ME)
        .zip(city.ring(2))
    {
        u.pos = pos;
    }
    let mut p = planner(&view);
    p.difficulty = AiDifficulty::Hard;
    p.campaigns.active = Some(Campaign {
        target: CityId(9),
        start_turn: view.turn - 1,
        committed: vec![UnitId(1), UnitId(2)],
        deadline: None,
        expected_value: 100,
        target_was_unwalled: true,
    });
    p.reconcile_campaign();
    assert!(p.campaigns.active.is_none());
    assert_eq!(
        p.campaigns.events.last().unwrap().end,
        Some(CampaignEnd::WallsBeforeArrival)
    );
}

fn target(view: &PlayerView) -> Option<Hex> {
    let p = planner(view);
    p.campaign_target(&p.ctx.own_units)
}

fn visible_assault() -> PlayerView {
    let view = ViewBuilder::new(30, 16)
        .city(1, ME, Hex::new(3, 8), 3)
        .city(9, ENEMY, TARGET, 2)
        .unit(1, ME, UnitKind::Soldier, Hex::new(11, 8))
        .unit(2, ME, UnitKind::Soldier, Hex::new(11, 9))
        .unit(50, ENEMY, UnitKind::Soldier, TARGET)
        .build();
    // City fire has dedicated group tests.
    view
}

#[test]
fn recent_retake_hold_requires_ownership_force_and_a_live_window() {
    let mut view = visible_assault();
    view.turn = 40;
    let city = &view.cities[1];
    let active = Campaign {
        target: city.id,
        start_turn: 38,
        committed: vec![UnitId(1), UnitId(2)],
        deadline: None,
        expected_value: 100,
        target_was_unwalled: false,
    };
    let mut p = planner(&view);
    p.decisions.retake_hold = 5;
    assert!(
        !p.retake_hold(city, &active),
        "no evidence we owned the city"
    );
    p.campaigns.last_owned.insert(city.id, 39);
    assert!(p.retake_hold(city, &active));
    p.campaigns.last_owned.insert(city.id, 19);
    assert!(!p.retake_hold(city, &active), "loss older than 20 turns");
    p.campaigns.last_owned.insert(city.id, 39);
    assert!(!p.retake_hold(
        city,
        &Campaign {
            committed: vec![],
            ..active.clone()
        }
    ));
    assert!(!p.retake_hold(
        city,
        &Campaign {
            start_turn: 35,
            ..active.clone()
        }
    ));
    p.decisions.retake_hold = 0;
    assert!(!p.retake_hold(city, &active));
}

#[test]
fn visible_assault_spends_only_the_forecast_force_margin() {
    let view = visible_assault();
    let mut p = planner(&view);
    p.knobs.aggression = 0;
    let army = p.ctx.own_units.clone();
    let t = p
        .targets(false)
        .into_iter()
        .find(|t| t.pos == TARGET)
        .unwrap();
    let force: i32 = army.iter().map(|u| offense_power(&view, u)).sum();
    assert!(force * 100 >= p.knobs.attack_threshold() * t.their);
    assert!(army.iter().all(|u| !p.reaches_this_turn(u, TARGET)));
    assert!(
        !p.stage_assault(&army, &t),
        "incoming damage exceeds the margin"
    );
    assert!(p.orders.is_empty());
    assert!(p.mil_dest.is_empty(), "failed plans cannot reserve tiles");
    p.knobs.aggression = 100;
    assert!(p.stage_assault(&army, &t));
    p.dedupe_first_steps();
    assert_eq!(p.orders.len(), 2);
    for order in p.orders {
        assert!(matches!(order, Order::MoveUnit { to, .. } if to.distance(TARGET) == 1));
    }
}

#[test]
fn assault_exposure_uses_arrival_terrain_and_clears_marching_fortification() {
    let mut view = visible_assault();
    let spots = [TARGET.neighbor(3), TARGET.neighbor(4)];
    let remaining = |view: &PlayerView| {
        let p = planner(view);
        p.assault_force_after_exposure(&[
            (p.ctx.own_units[0], spots[0]),
            (p.ctx.own_units[1], spots[1]),
        ])
    };
    // One defender gets one attack, not one attack per staged Soldier. The engine's
    // expected 29 damage leaves one Soldier at 71 HP (12 power), the other at 20.
    assert_eq!(remaining(&view), 32);
    for u in view.units.iter_mut().filter(|u| u.owner == ME) {
        u.fortify = data::FORTIFY_MAX;
    }
    assert_eq!(remaining(&view), 32, "movement clears fortification");
    for spot in spots {
        let i = view.index(spot).unwrap();
        view.tiles[i].terrain = Some(Terrain::Hills);
    }
    assert!(
        remaining(&view) > 32,
        "forecast the destination's defense bonus"
    );
}

#[test]
fn exposed_staging_does_not_count_troops_that_cannot_arrive_this_turn() {
    let mut view = visible_assault();
    let slow = view.units.iter_mut().find(|u| u.id == UnitId(2)).unwrap();
    // Separate approaches keep the expensive terrain off the first Soldier's route.
    slow.pos = TARGET.neighbor(0).neighbor(0).neighbor(0);
    let slow_pos = slow.pos;
    for pos in slow_pos.neighbors() {
        let i = view.index(pos).unwrap();
        view.tiles[i].terrain = Some(Terrain::Hills);
    }
    let mut p = planner(&view);
    let army = p.ctx.own_units.clone();
    let t = p
        .targets(false)
        .into_iter()
        .find(|t| t.pos == TARGET)
        .unwrap();
    assert!(
        TARGET
            .neighbors()
            .iter()
            .any(|h| p.reaches_this_turn(army[0], *h))
    );
    assert!(
        TARGET
            .neighbors()
            .iter()
            .all(|h| !p.reaches_this_turn(army[1], *h))
    );
    assert!(!p.stage_assault(&army, &t));
    assert!(
        p.orders.is_empty(),
        "the faster Soldier must not advance alone"
    );
}

#[test]
fn visible_assault_does_not_spend_a_margin_on_unseen_defenders() {
    let mut view = visible_assault();
    let hidden = TARGET.neighbor(3);
    let i = view.index(hidden).unwrap();
    view.tiles[i].visible = false;
    view.observed_threats
        .push(sighting(&view, hidden, UnitKind::Scout));
    // Use a distinct observation ID so the visible garrison does not supersede it.
    view.observed_threats[0].unit = UnitId(51);
    let mut p = planner(&view);
    let army = p.ctx.own_units.clone();
    let t = p
        .targets(false)
        .into_iter()
        .find(|t| t.pos == TARGET)
        .unwrap();
    let arrivals = [(army[0], TARGET.neighbor(3)), (army[1], TARGET.neighbor(4))];
    assert!(
        p.assault_force_after_exposure(&arrivals) * 100 >= p.knobs.attack_threshold() * t.their,
        "visible exposure fits the margin; the remembered reach must still veto staging"
    );
    assert!(!p.stage_assault(&army, &t));
    assert!(p.orders.is_empty());
}

#[test]
fn movement_reconciliation_cancels_an_exposed_group_together() {
    // These routes collide on their second step. Ordinary traffic yields one unit;
    // an exposed assault must yield both, preserving the force used for approval.
    let middle = Hex::new(6, 6);
    let left = middle.neighbor(3).neighbor(3);
    let right = middle.neighbor(0).neighbor(0);
    let mut b = ViewBuilder::new(20, 14).patch(middle, 20, Terrain::Mountains);
    for pos in [left, middle.neighbor(3), middle, middle.neighbor(0), right] {
        b = b.terrain(pos, Terrain::Plains);
    }
    let view = b
        .unit(1, ME, UnitKind::Soldier, left)
        .unit(2, ME, UnitKind::Soldier, right)
        .build();
    let mut p = planner(&view);
    p.orders = vec![
        Order::MoveUnit {
            unit: UnitId(1),
            to: right,
        },
        Order::MoveUnit {
            unit: UnitId(2),
            to: left,
        },
    ];
    p.assault_groups.push(vec![UnitId(1), UnitId(2)]);
    p.dedupe_first_steps();
    assert_eq!(
        p.orders,
        vec![
            Order::SkipUnit { unit: UnitId(1) },
            Order::SkipUnit { unit: UnitId(2) }
        ]
    );
}

#[test]
fn movement_reconciliation_requires_every_assault_member_to_finish_its_route() {
    let view = ViewBuilder::new(24, 16)
        .unit(1, ME, UnitKind::Soldier, Hex::new(6, 6))
        .unit(2, ME, UnitKind::Soldier, Hex::new(6, 10))
        .build();
    let mut p = planner(&view);
    // No traffic collision: one arrival is merely beyond this turn's movement budget.
    p.orders = vec![
        Order::MoveUnit {
            unit: UnitId(1),
            to: Hex::new(8, 6),
        },
        Order::MoveUnit {
            unit: UnitId(2),
            to: Hex::new(9, 10),
        },
    ];
    p.assault_groups.push(vec![UnitId(1), UnitId(2)]);
    p.dedupe_first_steps();
    assert_eq!(
        p.orders,
        vec![
            Order::SkipUnit { unit: UnitId(1) },
            Order::SkipUnit { unit: UnitId(2) }
        ]
    );
}

#[test]
fn remembered_garrison_and_support_affect_campaign_estimates_until_expiry() {
    for pos in [TARGET, TARGET.neighbor(0)] {
        let mut view = fogged_campaign();
        assert_eq!(target(&view), Some(TARGET));
        view.observed_threats
            .push(sighting(&view, pos, UnitKind::AssaultTrooper));
        assert_eq!(target(&view), None, "recent defender at {pos:?}");
        assert!(
            planner(&view).targets(false).is_empty(),
            "a sighting is not an attack target"
        );
        assert!(!planner(&view).safe_campaign_tile(pos.neighbor(0)));

        let expiry =
            view.observed_threats[0].seen_turn + legatum_core::visibility::THREAT_MEMORY_TURNS;
        view.turn = expiry - 1;
        assert_eq!(target(&view), None);
        view.turn = expiry;
        assert_eq!(target(&view), Some(TARGET));
        let mut walled = view.clone();
        walled
            .cities
            .iter_mut()
            .find(|c| c.pos == TARGET)
            .unwrap()
            .walls = true;
        assert!(
            !planner(&walled).safe_campaign_tile(pos.neighbor(0)),
            "the known walled city still fires after unit memory expires"
        );
    }
}

#[test]
fn live_units_and_reconnaissance_supersede_sightings_without_double_counting() {
    let mut view = fogged_campaign();
    let old = sighting(&view, TARGET.neighbor(0), UnitKind::AssaultTrooper);
    view.units.extend(
        ViewBuilder::new(30, 16)
            .unit(50, ENEMY, UnitKind::Soldier, TARGET.neighbor(3))
            .build()
            .units,
    );
    let estimate = |v: &PlayerView| {
        planner(v)
            .targets(true)
            .into_iter()
            .find(|t| t.pos == TARGET)
            .unwrap()
            .their
    };
    let live_only = estimate(&view);
    view.observed_threats.push(old.clone());
    assert_eq!(estimate(&view), live_only);
    assert!(planner(&view).ctx.remembered_threats.is_empty());

    view.units.retain(|u| u.id != old.unit);
    assert_eq!(target(&view), None);
    let i = view.index(old.pos).unwrap();
    view.tiles[i].visible = true;
    assert_eq!(
        target(&view),
        Some(TARGET),
        "visible empty terrain clears the uncertainty"
    );
}

#[test]
fn repeated_sightings_count_once_and_friendly_or_future_sightings_do_not_count() {
    let mut view = fogged_campaign();
    let newest = sighting(&view, TARGET, UnitKind::Soldier);
    let older = ThreatObservation {
        pos: TARGET.neighbor(0),
        seen_turn: newest.seen_turn - 1,
        ..newest.clone()
    };
    for sightings in [
        vec![newest.clone(), older.clone()],
        vec![older, newest.clone(), newest.clone()],
    ] {
        view.observed_threats = sightings;
        let p = planner(&view);
        assert_eq!(p.ctx.remembered_threats, vec![&newest]);
    }
    view.observed_threats = vec![
        ThreatObservation {
            owner: ME,
            ..newest.clone()
        },
        ThreatObservation {
            seen_turn: view.turn + 1,
            ..newest
        },
    ];
    assert!(planner(&view).ctx.remembered_threats.is_empty());
}

#[test]
fn peace_removes_foreign_sightings_from_campaign_danger() {
    let mut view = fogged_campaign();
    view.observed_threats
        .push(sighting(&view, TARGET, UnitKind::Soldier));
    assert!(!planner(&view).safe_campaign_tile(TARGET.neighbor(3)));

    // Legacy tactical fixtures start at War. A modern view without a War
    // relation is at Peace; an old sighting must not remain a military threat.
    view.diplomacy.legacy = false;
    let p = planner(&view);
    assert!(!view.at_war(ME, ENEMY));
    assert!(p.ctx.remembered_threats.is_empty());
    assert!(p.safe_campaign_tile(TARGET.neighbor(3)));
    assert!(p.targets(true).is_empty());
}

#[test]
fn reachable_reinforcements_reopen_campaign_but_do_not_erase_memory() {
    let mut view = fogged_campaign();
    view.observed_threats
        .push(sighting(&view, TARGET, UnitKind::AssaultTrooper));
    assert_eq!(target(&view), None);
    view.units.extend(
        ViewBuilder::new(30, 16)
            .unit(3, ME, UnitKind::Trooper, Hex::new(7, 8))
            .unit(4, ME, UnitKind::Trooper, Hex::new(7, 9))
            .build()
            .units,
    );
    assert_eq!(target(&view), Some(TARGET));
    assert!(!planner(&view).safe_campaign_tile(TARGET.neighbor(3)));
}

#[test]
fn fogged_campaign_stages_outside_remembered_reach() {
    let mut view = fogged_campaign();
    view.observed_threats
        .push(sighting(&view, TARGET, UnitKind::Soldier));
    let mut p = planner(&view);
    let army = p.ctx.own_units.clone();
    p.campaign(&army);
    assert_eq!(p.orders.len(), 2);
    for order in &p.orders {
        let Order::MoveUnit { to, .. } = order else {
            panic!("{order:?}")
        };
        assert_eq!(
            to.distance(TARGET),
            3,
            "stage outside the remembered two-move reach"
        );
        assert!(p.safe_campaign_tile(*to));
    }
}

#[test]
fn a_safe_destination_cannot_hide_an_exposed_arrival_on_expensive_terrain() {
    let start = Hex::new(6, 8);
    let end = Hex::new(11, 8);
    let mut b = ViewBuilder::new(24, 16)
        .patch(start, 20, Terrain::Mountains)
        .unit(1, ME, UnitKind::Soldier, start);
    for q in 6..=11 {
        b = b.terrain(Hex::new(q, 8), Terrain::Plains);
    }
    b = b.terrain(Hex::new(8, 8), Terrain::Hills);
    let mut view = b.build();
    for tile in &mut view.tiles {
        tile.visible = false;
    }
    view.observed_threats
        .push(sighting(&view, Hex::new(8, 8), UnitKind::Soldier));
    let p = planner(&view);
    assert!(p.safe_campaign_tile(end));
    assert_eq!(p.campaign_approach(p.ctx.own_units[0], end), None);
}

#[test]
fn favorable_melee_forecast_does_not_approve_an_unreachable_attack() {
    let target = Hex::new(10, 8);
    let view = ViewBuilder::new(24, 16)
        .unit(1, ME, UnitKind::Soldier, Hex::new(8, 8))
        .unit_hp(50, ENEMY, UnitKind::Ranger, target, 40)
        .terrain(Hex::new(9, 8), Terrain::Hills)
        .build();
    let mut p = planner(&view);
    let u = p.ctx.own_units[0];
    assert!(!p.too_risky(u, target));
    assert!(!p.reaches_this_turn(u, target));
    p.strike(u, target, false);
    assert_eq!(p.orders, vec![Order::SkipUnit { unit: u.id }]);
}

#[test]
fn visible_targets_still_include_recent_unseen_support_in_the_force_estimate() {
    let mut view = fogged_campaign();
    view.cities
        .iter_mut()
        .find(|c| c.pos == TARGET)
        .unwrap()
        .visible = true;
    let i = view.index(TARGET).unwrap();
    view.tiles[i].visible = true;
    let estimate = |v: &PlayerView| {
        planner(v)
            .targets(false)
            .into_iter()
            .find(|t| t.pos == TARGET)
            .unwrap()
            .their
    };
    let baseline = estimate(&view);
    view.observed_threats
        .push(sighting(&view, TARGET.neighbor(0), UnitKind::Soldier));
    assert!(estimate(&view) > baseline);
    assert_eq!(
        planner(&view).targets(false).len(),
        1,
        "support is not a fogged attack target"
    );
}

#[test]
fn campaign_commitment_ratio_gene_preserves_default_and_can_require_more_power() {
    let view = fogged_campaign();
    let mut p = planner(&view);
    assert_eq!(p.decisions.war_commit_ratio, 100);
    assert_eq!(p.campaign_target(&p.ctx.own_units), Some(TARGET));
    p.decisions.war_commit_ratio = 250;
    assert_eq!(p.campaign_target(&p.ctx.own_units), None);
}

#[test]
fn rush_campaign_keeps_the_declared_city_even_with_low_aggression() {
    use crate::openings::{Opening, OpeningPlan, OpeningPolicy, Phase};
    let view = ViewBuilder::new(32, 20)
        .city(1, ME, Hex::new(5, 5), 3)
        .city(9, ENEMY, TARGET, 2)
        .city(10, legatum_core::PlayerId(3), Hex::new(7, 6), 1)
        .unit(1, ME, UnitKind::Soldier, Hex::new(6, 8))
        .unit(2, ME, UnitKind::Ranger, Hex::new(6, 9))
        .build();
    let mut p = planner(&view);
    p.knobs.aggression = 0;
    assert_eq!(p.campaign_target(&p.ctx.own_units), None);
    let policy = OpeningPolicy::builtin(Opening::Rush);
    for phase in [Phase::Opening, Phase::Conquer] {
        p.opening = Some(OpeningPlan {
            investment: None,
            policy: &policy,
            phase,
            next_build: None,
        });
        p.strike_target = Some(view.team_of(ENEMY));
        p.strike_city = Some(legatum_core::CityId(9));
        assert_eq!(p.campaign_target(&p.ctx.own_units), Some(TARGET));
    }
}

#[test]
fn wounded_rush_troops_leave_city_fire_before_healing() {
    use crate::openings::{Opening, OpeningPlan, OpeningPolicy, Phase};
    let view = ViewBuilder::new(30, 20)
        .city(1, ME, Hex::new(5, 5), 3)
        .city(9, ENEMY, TARGET, 3)
        .walled(9)
        .unit_hp(1, ME, UnitKind::Soldier, Hex::new(13, 8), 55)
        .build();
    let policy = OpeningPolicy::builtin(Opening::Rush);
    let mut p = planner(&view);
    p.opening = Some(OpeningPlan {
        investment: None,
        policy: &policy,
        phase: Phase::Conquer,
        next_build: None,
    });
    p.recover_city_fire(&p.ctx.own_units.clone());
    assert!(p.retreats.contains(&UnitId(1)));
    p.dedupe_first_steps();
    assert!(
        p.orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit, to }
        if *unit == UnitId(1) && to.distance(TARGET) > 2))
    );
}

#[test]
fn rush_rotates_an_archer_before_two_city_volleys_can_kill_it() {
    use crate::openings::{Opening, OpeningPlan, OpeningPolicy, Phase};
    let mut view = ViewBuilder::new(30, 20)
        .city(1, ME, Hex::new(5, 5), 3)
        .city(9, ENEMY, TARGET, 1)
        .walled(9)
        .unit(1, ME, UnitKind::Ranger, Hex::new(12, 8))
        .build();
    // A worn Customs Bastion (pop 1, 60 HP) shoots with the old unwalled pop-5 strength of 20.
    view.cities[1].hp = 60;
    let first = forecast_attack_mode(&view, UnitId(1), TARGET, AttackMode::Ranged)
        .unwrap()
        .city_strike
        .unwrap();
    assert!(
        first.damage * 2 < 100,
        "a doubled mean misses the wound penalty"
    );
    let policy = OpeningPolicy::builtin(Opening::Rush);
    let mut p = planner(&view);
    p.opening = Some(OpeningPlan {
        investment: None,
        policy: &policy,
        phase: Phase::Conquer,
        next_build: None,
    });
    p.recover_city_fire(&p.ctx.own_units.clone());
    assert!(p.retreats.contains(&UnitId(1)));
    p.dedupe_first_steps();
    assert!(p.orders.iter().any(|o| matches!(o,
        Order::MoveUnit { unit, to } if *unit == UnitId(1) && to.distance(TARGET) > 2)));
}

#[test]
fn siege_retreats_override_traffic_holds_and_leave_rough_terrain_in_stages() {
    use crate::openings::{Opening, OpeningPlan, OpeningPolicy, Phase};
    let mut builder = ViewBuilder::new(30, 20)
        .city(9, ENEMY, TARGET, 3)
        .walled(9)
        .unit(1, ME, UnitKind::Soldier, Hex::new(13, 8))
        .unit_hp(2, ME, UnitKind::Soldier, Hex::new(13, 9), 50);
    for pos in TARGET.within(4) {
        builder = builder.terrain(pos, Terrain::Hills);
    }
    let mut view = builder.build();
    for unit in &mut view.units {
        unit.goto = Some(TARGET);
        unit.moves_left = unit.max_moves;
    }
    let policy = OpeningPolicy::builtin(Opening::Rush);
    let mut p = planner(&view);
    p.opening = Some(OpeningPlan {
        investment: None,
        policy: &policy,
        phase: Phase::Conquer,
        next_build: None,
    });
    p.plan_yields();
    assert!(!p.has_acted(UnitId(2)), "city fire outranks a traffic hold");
    p.recover_city_fire(&p.ctx.own_units.clone());
    p.dedupe_first_steps();
    assert!(
        p.orders.iter().any(|o| matches!(o,
        Order::MoveUnit { unit, to } if *unit == UnitId(2) && to.distance(TARGET) == 2)),
        "a partial escape must survive movement reconciliation: {:?}",
        p.orders
    );
}

#[test]
fn rush_bombards_then_commits_one_capturer_with_archer_support() {
    use crate::openings::{Opening, OpeningPlan, OpeningPolicy, Phase};
    let mut view = ViewBuilder::new(30, 20)
        .city(1, ME, Hex::new(5, 5), 3)
        .city(9, ENEMY, TARGET, 2)
        .unit(1, ME, UnitKind::Soldier, Hex::new(13, 8))
        .unit(2, ME, UnitKind::Soldier, Hex::new(13, 9))
        .unit(3, ME, UnitKind::Ranger, Hex::new(12, 8))
        .unit(4, ME, UnitKind::Ranger, Hex::new(12, 9))
        .build();
    let policy = OpeningPolicy::builtin(Opening::Rush);
    for (hp, capturers) in [(200, 2), (30, 1)] {
        view.cities[1].hp = hp;
        let mut p = planner(&view);
        p.opening = Some(OpeningPlan {
            investment: None,
            policy: &policy,
            phase: Phase::Conquer,
            next_build: None,
        });
        p.strike_target = Some(view.team_of(ENEMY));
        p.strike_city = Some(legatum_core::CityId(9));
        p.plan_military();
        p.dedupe_first_steps();
        assert_eq!(
            p.orders
                .iter()
                .filter(|o| matches!(o, Order::RangedAttack { .. }))
                .count(),
            2
        );
        assert!(p.committed_fire_support(&p.ctx.own_units, TARGET) > 0);
        assert_eq!(
            p.orders
                .iter()
                .filter(|o| matches!(o, Order::MoveUnit { to, .. } if *to == TARGET))
                .count(),
            capturers,
            "city HP {hp}: {:?}",
            p.orders
        );
    }
}

#[test]
fn rush_occupies_a_defeated_city_with_one_survivor() {
    use crate::openings::{Opening, OpeningPlan, OpeningPolicy, Phase};
    let mut view = ViewBuilder::new(30, 20)
        .city(1, ME, Hex::new(5, 5), 3)
        .city(9, ENEMY, TARGET, 2)
        .unit_hp(1, ME, UnitKind::Soldier, Hex::new(13, 7), 30)
        .unit(2, ME, UnitKind::Ranger, Hex::new(13, 8))
        .unit(3, ME, UnitKind::Ranger, Hex::new(12, 8))
        .unit(4, ME, UnitKind::Ranger, Hex::new(12, 9))
        .build();
    view.cities[1].hp = 0;
    let policy = OpeningPolicy::builtin(Opening::Rush);
    let mut p = planner(&view);
    p.opening = Some(OpeningPlan {
        investment: None,
        policy: &policy,
        phase: Phase::Conquer,
        next_build: None,
    });
    p.strike_target = Some(view.team_of(ENEMY));
    p.strike_city = Some(legatum_core::CityId(9));
    p.plan_military();
    assert!(
        p.orders.contains(&Order::MoveUnit {
            unit: UnitId(1),
            to: TARGET
        }),
        "a ranged unit cannot capture; take the defeated city before resting: {:?}",
        p.orders
    );
    assert_eq!(
        p.orders
            .iter()
            .filter(|o| matches!(o, Order::MoveUnit { to, .. } if *to == TARGET))
            .count(),
        1,
        "{:?}",
        p.orders
    );
}

/// Idea 24c: with no plan attacking a firing city, a unit standing in its two-hex strike
/// range walks out of it; while a campaign attacks that city it stays.
#[test]
fn units_leave_a_firing_citys_range_once_no_plan_attacks_it() {
    let view = ViewBuilder::new(30, 16)
        .city(1, ME, Hex::new(6, 8), 3)
        .city(9, ENEMY, TARGET, 3)
        .walled(9)
        .unit(1, ME, UnitKind::Soldier, TARGET.neighbor(0).neighbor(0))
        .build();
    let run = |gene: i32, campaigning: bool| {
        let mut p = planner(&view);
        p.decisions.strike_range_exit = gene;
        if campaigning {
            p.campaigns.active = Some(Campaign {
                target: CityId(9),
                committed: vec![UnitId(1)],
                start_turn: view.turn,
                deadline: None,
                expected_value: 0,
                target_was_unwalled: false,
            });
        }
        let army: Vec<&legatum_core::view::UnitView> = p
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military)
            .collect();
        p.leave_strike_range(&army);
        p.orders.iter().find_map(|o| match o {
            Order::MoveUnit { unit, to, .. } if *unit == UnitId(1) => Some(*to),
            _ => None,
        })
    };
    assert_eq!(run(0, false), None, "gene off");
    let to = run(1, false).expect("walks out");
    assert!(
        to.distance(TARGET) > 2,
        "{to:?} is outside the strike range"
    );
    assert_eq!(
        run(1, true),
        None,
        "a running campaign keeps its units there"
    );
}

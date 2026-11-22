use super::*;
use crate::rules::test_support::*;
use crate::rules::{resolve_turn, validate_orders};

const A: PlayerId = PlayerId(0);
const B: PlayerId = PlayerId(1);
/// The population the New Fire sacrifices and the promotion level a retiring veteran needs.
const SWARM: i32 = goals::AZTEC_NEW_FIRE_POPULATION;
const RETIRE_LEVEL: u8 = goals::SPARTA_RETIRE_LEVEL;

fn act(action: Action) -> Order {
    Order::Action { action }
}

fn aztecs_game(pops: &[i32]) -> (GameState, Vec<CityId>) {
    let mut s = game(&[0, 1]);
    s.player_mut(A).faction = Faction::Aztecs;
    // Aztecs use the Swarm rule by default.
    let ids = pops
        .iter()
        .enumerate()
        .map(|(i, p)| city(&mut s, 0, hex(1 + 3 * i as i32, 2), *p))
        .collect();
    // A rival city, so neither empire is eliminated by the turn.
    city(&mut s, 1, hex(10, 8), 3);
    (s, ids)
}

#[test]
fn swarm_plan_takes_from_the_biggest_city_one_at_a_time() {
    // Two big cities and a smaller one 6 below them: the big ones level down to it (6 taken each),
    // then all three give an equal share of what is left of the sacrifice.
    let (s, ids) = aztecs_game(&[21, 21, 15]);
    let share = (SWARM - 12) / 3;
    assert_eq!(
        (SWARM - 12) % 3,
        0,
        "the rest of the sacrifice splits three ways"
    );
    assert_eq!(
        goals::swarm_plan(&s, A),
        vec![(ids[0], 6 + share), (ids[1], 6 + share), (ids[2], share)]
    );
}

#[test]
fn swarm_plan_levels_ties_toward_the_lower_city_id_and_keeps_one() {
    let (s, ids) = aztecs_game(&[10, 10, 1]);
    // Spare is 18: short of the sacrifice, and the one-population city is untouched.
    let plan = goals::swarm_plan(&s, A);
    assert_eq!(plan.iter().map(|(_, l)| l).sum::<i32>(), 18);
    assert!(plan.iter().all(|(c, _)| *c != ids[2]));
    assert_eq!(plan, vec![(ids[0], 9), (ids[1], 9)]);
    // Odd split: the lower id keeps its extra population taken first.
    let (s, ids) = aztecs_game(&[SWARM + 2, 1]);
    assert_eq!(goals::swarm_plan(&s, A), vec![(ids[0], SWARM)]);
    let (s, ids) = aztecs_game(&[SWARM, SWARM]);
    // Plenty spare: both levelled down together, evenly.
    let plan = goals::swarm_plan(&s, A);
    assert_eq!(plan.iter().map(|(_, l)| l).sum::<i32>(), SWARM);
    assert_eq!(plan, vec![(ids[0], SWARM / 2), (ids[1], SWARM / 2)]);
}

#[test]
fn swarm_needs_enough_spare_population_and_the_aztecs() {
    // Two cities that together spare 2 short of the sacrifice.
    let (s, _) = aztecs_game(&[SWARM / 2, SWARM / 2]);
    let errs = validate_orders(&s, A, &[act(Action::Swarm)]);
    assert_eq!(errs.len(), 1);
    assert!(
        errs[0]
            .message
            .contains(&format!("can spare {}", SWARM - 2)),
        "{}",
        errs[0].message
    );
    // Exactly the sacrifice spare is enough.
    let (s, _) = aztecs_game(&[SWARM / 2 + 1, SWARM / 2 + 1]);
    assert!(validate_orders(&s, A, &[act(Action::Swarm)]).is_empty());
    // Another faction can't.
    let mut other = s.clone();
    other.player_mut(A).faction = Faction::India;
    other.player_mut(A).goal_rule = None;
    assert!(
        validate_orders(&other, A, &[act(Action::Swarm)])[0]
            .message
            .contains("goal is the New Fire")
    );
    // A second Swarm is refused once the goal is done.
    let mut done = s.clone();
    done.player_mut(A).faction_goal_done = true;
    assert!(
        validate_orders(&done, A, &[act(Action::Swarm)])[0]
            .message
            .contains("already")
    );
    // Two in one turn: the second is flagged.
    let errs = validate_orders(&s, A, &[act(Action::Swarm), act(Action::Swarm)]);
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].index, 1);
}

/// Two big Aztec cities (and a small one) with the territory their population claims: one
/// grown tile per population above 1.
fn grown_aztecs_game() -> (GameState, Vec<CityId>) {
    let mut s = game(&[0, 1]);
    s.player_mut(A).faction = Faction::Aztecs;
    let spots = [
        (hex(2, 2), SWARM / 2 + 1),
        (hex(9, 7), SWARM / 2 + 1),
        (hex(5, 8), 1),
    ];
    let mut ids = Vec::new();
    for (at, pop) in spots {
        ids.push(city(&mut s, 0, at, pop));
    }
    city(&mut s, 1, hex(10, 1), 3);
    for id in &ids {
        let (pos, pop) = (s.cities[id].pos, s.cities[id].population);
        for h in pos.within(8) {
            if s.cities[id].grown_tiles.len() >= (pop - 1) as usize {
                break;
            }
            if let Some(t) = s.map.get_mut(h)
                && t.owner.is_none()
            {
                t.owner = Some(A);
                t.city = Some(*id);
                s.cities.get_mut(id).unwrap().grown_tiles.push(h);
            }
        }
    }
    (s, ids)
}

#[test]
fn swarm_resolution_takes_population_releases_territory_and_scores_for_good() {
    let (s, ids) = grown_aztecs_game();
    for id in &ids {
        assert_eq!(
            s.cities[id].grown_tiles.len(),
            (s.cities[id].population - 1) as usize
        );
    }
    let before_tiles: usize = ids.iter().map(|id| s.cities[id].grown_tiles.len()).sum();
    let mut events = Vec::new();
    let mut now = s.clone();
    apply_actions(&mut now, A, &[act(Action::Swarm)], &mut events);
    let plan = goals::swarm_plan(&s, A);
    for (id, lost) in &plan {
        assert_eq!(now.cities[id].population, s.cities[id].population - lost);
        // Population is territory: grown tiles follow population, borders shrink at once.
        assert_eq!(
            now.cities[id].grown_tiles.len(),
            (now.cities[id].population - 1) as usize
        );
    }
    assert_eq!(
        now.cities[&ids[2]].population, 1,
        "a one-population city is untouched"
    );
    assert!(now.cities[&ids[0]].population == 1 && now.cities[&ids[1]].population == 1);
    let after_tiles: usize = ids.iter().map(|id| now.cities[id].grown_tiles.len()).sum();
    assert_eq!(before_tiles - after_tiles, SWARM as usize);
    let swarm = events
        .iter()
        .find_map(|e| match e {
            Event::SwarmUnleashed { player, cities } if *player == A => Some(cities.clone()),
            _ => None,
        })
        .expect("swarm event");
    assert_eq!(swarm.iter().map(|c| c.lost).sum::<i32>(), SWARM);
    assert_eq!(
        swarm.iter().map(|c| c.tiles).sum::<u32>() as usize,
        SWARM as usize
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TileReleased { .. }))
    );
    // Through a whole turn: the goal scores 2 VP, for good.
    let (next, turn_events) = resolve_turn(&s, &orders(0, vec![act(Action::Swarm)]));
    assert!(next.player(A).faction_goal_done);
    assert!(
        turn_events.iter().any(
            |e| matches!(e, Event::FactionGoalCompleted { player, vp: 2, .. } if *player == A)
        )
    );
    let goal_vp = |st: &GameState| -> u32 {
        crate::scoring::vp(st, A)
            .iter()
            .filter(|(src, _)| matches!(src, crate::VpSource::FactionGoal { .. }))
            .map(|(_, v)| v)
            .sum()
    };
    assert_eq!((goal_vp(&s), goal_vp(&next)), (0, 2));
    let mut later = next.clone();
    for c in later.cities.values_mut().filter(|c| c.owner == A) {
        c.population = 40;
    }
    let (later, _) = resolve_turn(&later, &no_orders());
    assert!(later.player(A).faction_goal_done);
    assert_eq!(goal_vp(&later), 2);
}

#[test]
fn swarm_view_previews_each_city_and_matches_resolution() {
    let (s, _) = grown_aztecs_game();
    let swarm = view(&s, A).swarm.expect("swarm view");
    assert!(swarm.blocker.is_none());
    assert_eq!(swarm.spare, SWARM);
    assert_eq!(swarm.cities.len(), 2);
    let mut next = s.clone();
    apply_actions(&mut next, A, &[act(Action::Swarm)], &mut Vec::new());
    for c in &swarm.cities {
        assert_eq!(next.cities[&c.city].population, c.population);
        let released = s.cities[&c.city].grown_tiles.len() - next.cities[&c.city].grown_tiles.len();
        assert_eq!(released, c.tiles.len());
        for h in &c.tiles {
            assert!(
                next.map.get(*h).unwrap().owner.is_none(),
                "preview tile is released"
            );
        }
    }
    assert!(view(&next, A).swarm.is_none(), "done: nothing to preview");
}

fn signal_game() -> GameState {
    let mut s = game(&[0, 1, 2, 2]);
    s.player_mut(A).faction = Faction::Athens;
    city(&mut s, 0, hex(1, 1), 3);
    city(&mut s, 1, hex(10, 1), 3);
    city(&mut s, 2, hex(1, 8), 3);
    city(&mut s, 3, hex(10, 8), 3);
    s
}

#[test]
fn broadcast_gives_every_living_rival_outside_the_team_three_picks() {
    let mut s = signal_game();
    // An eliminated rival receives nothing; a teammate of the broadcaster would not either.
    s.player_mut(PlayerId(3)).alive = false;
    let (next, events) = resolve_turn(&s, &orders(0, vec![act(Action::Broadcast)]));
    assert_eq!(next.player(B).free_techs_pending, 3);
    assert_eq!(next.player(PlayerId(2)).free_techs_pending, 3);
    assert_eq!(next.player(PlayerId(3)).free_techs_pending, 0);
    assert_eq!(next.player(A).free_techs_pending, 0);
    assert_eq!(next.player(B).free_techs_from, Some(A));
    assert!(next.player(A).faction_goal_done);
    let (receivers, techs_each) = events
        .iter()
        .find_map(|e| match e {
            Event::SignalBroadcast {
                from,
                receivers,
                techs_each,
            } if *from == A => Some((receivers.clone(), *techs_each)),
            _ => None,
        })
        .expect("broadcast event");
    assert_eq!(receivers, vec![B, PlayerId(2)]);
    assert_eq!(techs_each, 3);
    // Permanent, and a second broadcast is refused.
    assert!(
        validate_orders(&next, A, &[act(Action::Broadcast)])[0]
            .message
            .contains("already")
    );
}

#[test]
fn broadcast_needs_a_rival_and_the_athens() {
    let mut s = game(&[0, 0]);
    s.player_mut(A).faction = Faction::Athens;
    city(&mut s, 0, hex(1, 1), 3);
    let errs = validate_orders(&s, A, &[act(Action::Broadcast)]);
    assert!(errs[0].message.contains("No rival"), "{}", errs[0].message);
    let mut s = signal_game();
    s.player_mut(A).faction = Faction::Switzerland;
    assert!(
        validate_orders(&s, A, &[act(Action::Broadcast)])[0]
            .message
            .contains("Only Athens")
    );
}

#[test]
fn free_tech_choices_are_limited_to_the_current_era_and_persist_until_spent() {
    let mut s = signal_game();
    s.player_mut(B).free_techs_pending = 3;
    s.player_mut(B).free_techs_from = Some(A);
    s.player_mut(B).techs.insert(TechId::BronzeWorking);
    // No orders for several turns: nothing is auto-picked.
    let mut s2 = s.clone();
    for _ in 0..3 {
        s2 = resolve_turn(&s2, &no_orders()).0;
    }
    assert_eq!(s2.player(B).free_techs_pending, 3);
    let known = s.player(B).techs.clone();
    let open: Vec<TechId> = free_tech_choices(&s, B);
    assert_eq!(
        open,
        vec![
            TechId::Irrigation,
            TechId::Masonry,
            TechId::Currency,
            TechId::Writing
        ],
        "the unknown Ancient techs, nothing from later eras"
    );
    let (first, follow) = (open[0], open[1]);
    let choose =
        |techs: Vec<TechId>| validate_orders(&s, B, &[act(Action::ChooseFreeTechs { techs })]);
    // A later era's technology is refused; two of the current era are fine, in any order.
    assert!(
        choose(vec![TechId::Mathematics])[0]
            .message
            .contains("not in your era")
    );
    assert!(
        choose(vec![first, TechId::Mathematics])[0]
            .message
            .contains("not in your era")
    );
    assert!(choose(vec![first, follow]).is_empty());
    assert!(choose(vec![follow, first]).is_empty());
    assert!(choose(vec![]).len() == 1);
    assert!(
        choose(vec![first, first])[0]
            .message
            .contains("more than once")
    );
    // A technology the empire already knows.
    let known_tech = *known.iter().next().expect("a known technology");
    assert!(choose(vec![known_tech])[0].message.contains("already"));
    // More than the three pending.
    let four: Vec<TechId> = open.iter().copied().cycle().take(4).collect();
    assert!(!choose(four).is_empty());
    // Spending two keeps one pending.
    let (next, events) = resolve_turn(
        &s,
        &orders(
            1,
            vec![act(Action::ChooseFreeTechs {
                techs: vec![first, follow],
            })],
        ),
    );
    assert!(next.player(B).techs.contains(&first) && next.player(B).techs.contains(&follow));
    assert_eq!(next.player(B).free_techs_pending, 1);
    assert_eq!(next.player(B).free_techs_from, Some(A));
    assert!(events.iter().any(|e| matches!(e, Event::FreeTechsChosen { player, from: Some(f), techs } if *player == B && *f == A && techs == &vec![first, follow])));
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::TechResearched { player, .. } if *player == B))
            .count(),
        2
    );
    // Without a pending grant the choice is refused.
    assert!(
        validate_orders(
            &next,
            A,
            &[act(Action::ChooseFreeTechs { techs: vec![first] })]
        )[0]
        .message
        .contains("no free technologies")
    );
}

#[test]
fn a_free_pick_that_finishes_the_era_enters_the_next_one_and_stops_there() {
    let mut s = signal_game();
    crate::rules::test_support::know_eras_before(&mut s, B, 1);
    s.player_mut(B).techs.extend([
        TechId::Mathematics,
        TechId::Engineering,
        TechId::IronWorking,
        TechId::CodeOfLaws,
    ]);
    s.player_mut(B).free_techs_pending = 3;
    s.player_mut(B).free_techs_from = Some(A);
    assert_eq!(free_tech_choices(&s, B), vec![TechId::Philosophy]);
    let choose = |s: &GameState, techs: Vec<TechId>| {
        validate_orders(s, B, &[act(Action::ChooseFreeTechs { techs })])
    };
    // The next era's technology cannot ride along: eligibility is the era at the time of choosing.
    assert!(
        choose(&s, vec![TechId::Philosophy, TechId::Feudalism])[0]
            .message
            .contains("not in your era")
    );
    let (next, events) = resolve_turn(
        &s,
        &orders(
            1,
            vec![act(Action::ChooseFreeTechs {
                techs: vec![TechId::Philosophy],
            })],
        ),
    );
    assert_eq!(next.era_of(B), 2);
    assert_eq!(next.era_firsts.get(&2), Some(&B));
    assert!(events.contains(&Event::EraEntered {
        player: B,
        era: 3,
        first: true
    }));
    assert_eq!(
        next.player(B).free_techs_pending,
        2,
        "the rest wait for the new era"
    );
    assert_eq!(free_tech_choices(&next, B).len(), 5);
    assert!(choose(&next, vec![TechId::Feudalism, TechId::Machinery]).is_empty());
}

#[test]
fn a_broadcast_carries_three_picks_into_the_next_era() {
    let mut s = signal_game();
    crate::rules::test_support::know_eras_before(&mut s, B, 1);
    s.player_mut(B).techs.extend([
        TechId::Mathematics,
        TechId::Engineering,
        TechId::IronWorking,
    ]);
    let (next, _) = resolve_turn(&s, &orders(0, vec![act(Action::Broadcast)]));
    assert_eq!(
        next.player(B).free_techs_pending,
        3,
        "the third pick remains available in the Medieval era"
    );
    assert_eq!(next.player(PlayerId(2)).free_techs_pending, 3);
    let (after_classical, _) = resolve_turn(
        &next,
        &orders(
            1,
            vec![act(Action::ChooseFreeTechs {
                techs: vec![TechId::CodeOfLaws, TechId::Philosophy],
            })],
        ),
    );
    assert_eq!(after_classical.player(B).free_techs_pending, 1);
    assert_eq!(after_classical.era_of(B), 2);
    assert!(free_tech_choices(&after_classical, B).contains(&TechId::Feudalism));
}

#[test]
fn the_view_shows_the_broadcast_and_the_prompt() {
    let s = signal_game();
    let v = view(&s, A);
    let b = v.broadcast.expect("broadcast view");
    assert!(b.blocker.is_none());
    assert_eq!(b.receivers, vec![B, PlayerId(2), PlayerId(3)]);
    assert!(v.free_techs.is_none() && v.swarm.is_none());
    let (next, _) = resolve_turn(&s, &orders(0, vec![act(Action::Broadcast)]));
    assert!(view(&next, A).broadcast.is_none());
    let prompt = view(&next, B).free_techs.expect("prompt");
    assert_eq!((prompt.pending, prompt.from), (3, Some(A)));
    assert!(!prompt.choices.is_empty());
}

// ---------- Sparta: Honour the Fallen ----------

/// An Sparta empire with a War Shrine city and a second city without one, plus a rival.
fn sparta_game() -> (GameState, CityId, CityId) {
    let mut s = game(&[0, 1]);
    s.player_mut(A).faction = Faction::Sparta;
    let shrine = city(&mut s, 0, hex(2, 2), 3);
    s.cities
        .get_mut(&shrine)
        .unwrap()
        .buildings
        .insert(BuildingKind::Agoge);
    let plain = city(&mut s, 0, hex(8, 2), 3);
    city(&mut s, 1, hex(10, 8), 3);
    (s, shrine, plain)
}

/// A soldier of the given promotion level (Sparta ladder) standing at `at`.
fn veteran(s: &mut GameState, at: Hex, level: u8) -> UnitId {
    let id = unit(s, 0, UnitKind::Spearman, at);
    let xp = if level == 0 {
        0
    } else {
        crate::rules::promotion::threshold_for(Faction::Sparta, level).unwrap()
    };
    s.units.get_mut(&id).unwrap().xp = xp;
    id
}

fn retire(unit: UnitId) -> Order {
    act(Action::RetireVeteran { unit })
}

#[test]
fn retiring_needs_a_veteran_of_the_retire_level_standing_in_a_war_shrine_city() {
    let (mut s, shrine, plain) = sparta_game();
    let (shrine_pos, plain_pos) = (s.cities[&shrine].pos, s.cities[&plain].pos);
    let ok = veteran(&mut s, shrine_pos, RETIRE_LEVEL);
    let top = veteran(&mut s, shrine_pos, RETIRE_LEVEL + 2);
    let green = veteran(&mut s, shrine_pos, RETIRE_LEVEL - 1);
    let wrong_city = veteran(&mut s, plain_pos, RETIRE_LEVEL);
    let in_field = veteran(&mut s, hex(5, 5), RETIRE_LEVEL + 1);
    let scout = unit(&mut s, 0, UnitKind::Scout, shrine_pos);
    assert!(retire_blocker(&s, A, ok).is_none());
    assert!(retire_blocker(&s, A, top).is_none());
    assert!(
        retire_blocker(&s, A, green)
            .unwrap()
            .contains(&format!("promotion level {RETIRE_LEVEL}"))
    );
    assert!(
        retire_blocker(&s, A, wrong_city)
            .unwrap()
            .contains("no Agoge")
    );
    assert!(
        retire_blocker(&s, A, in_field)
            .unwrap()
            .contains("your cities with an Agoge")
    );
    assert!(retire_blocker(&s, A, scout).is_some());
    // Another empire's unit, another faction.
    let rival = unit(&mut s, 1, UnitKind::Spearman, hex(10, 8));
    assert!(retire_blocker(&s, A, rival).unwrap().contains("not yours"));
    let mut other = s.clone();
    other.player_mut(A).faction = Faction::Celts;
    assert!(
        retire_blocker(&other, A, ok)
            .unwrap()
            .contains("Only Sparta")
    );
    // The same unit twice in one turn is flagged; two units are two orders.
    let errs = validate_orders(&s, A, &[retire(ok), retire(ok), retire(top)]);
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].index, 1);
}

#[test]
fn three_retirements_complete_the_goal_for_good() {
    let (mut s, shrine, _) = sparta_game();
    let at = s.cities[&shrine].pos;
    let vets: Vec<_> = (0..4).map(|i| veteran(&mut s, at, 4 + (i % 2))).collect();
    let healthy = veteran(&mut s, at, 0);
    s.units.get_mut(&vets[0]).unwrap().hp = 40;
    // Two retire: progress, no VP yet, the units are gone, nothing heals the survivors.
    let (s2, events) = resolve_turn(&s, &orders(0, vec![retire(vets[0]), retire(vets[1])]));
    assert!(!s2.units.contains_key(&vets[0]) && !s2.units.contains_key(&vets[1]));
    assert!(s2.units.contains_key(&vets[2]) && s2.units.contains_key(&healthy));
    assert_eq!(s2.player(A).veterans_retired, 2);
    assert!(!s2.player(A).faction_goal_done);
    let retired: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::VeteranRetired {
                player,
                unit,
                city,
                retired,
                of,
                ..
            } if *player == A => Some((*unit, *city, *retired, *of)),
            _ => None,
        })
        .collect();
    assert_eq!(
        retired,
        vec![(vets[0], shrine, 1, 3), (vets[1], shrine, 2, 3)]
    );
    let g = goals::view(&s2, A).expect("progress");
    assert_eq!((g.value, g.target, g.done), (2, 3, false));
    assert_eq!(g.label, "2 of 3 veterans retired");
    // The third completes it, permanently.
    let (s3, events) = resolve_turn(&s2, &orders(0, vec![retire(vets[2])]));
    assert_eq!(s3.player(A).veterans_retired, 3);
    assert!(s3.player(A).faction_goal_done);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted { player, again: false, .. } if *player == A
    )));
    assert!(
        goals::view(&s3, A).is_none(),
        "a permanent goal has no meter once done"
    );
    // Nothing more to retire, and it stays done as the turns pass.
    assert!(
        retire_blocker(&s3, A, vets[3])
            .unwrap()
            .contains("already complete")
    );
    let (s4, _) = resolve_turn(&s3, &no_orders());
    assert!(s4.player(A).faction_goal_done);
}

#[test]
fn retirements_past_what_completes_the_goal_are_refused() {
    let (mut s, shrine, _) = sparta_game();
    let at = s.cities[&shrine].pos;
    let vets: Vec<_> = (0..4).map(|_| veteran(&mut s, at, 4)).collect();
    let orders_list: Vec<_> = vets.iter().map(|u| retire(*u)).collect();
    let errs = validate_orders(&s, A, &orders_list);
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].index, 3);
    // Earlier retirements count toward the limit.
    s.player_mut(A).veterans_retired = 2;
    let errs = validate_orders(&s, A, &orders_list);
    assert_eq!(
        errs.iter().map(|e| e.index).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn a_retiring_veteran_with_other_orders_leaves_cleanly() {
    let (mut s, shrine, _) = sparta_game();
    let at = s.cities[&shrine].pos;
    let a = veteran(&mut s, at, 4);
    let b = veteran(&mut s, at, 4);
    let os = vec![
        Order::MoveUnit {
            unit: a,
            to: hex(5, 5),
        },
        retire(a),
        Order::SkipUnit { unit: b },
        retire(b),
    ];
    let (s2, _) = resolve_turn(&s, &orders(0, os));
    assert!(!s2.units.contains_key(&a) && !s2.units.contains_key(&b));
    assert_eq!(s2.player(A).veterans_retired, 2);
}

#[test]
fn the_view_lists_shrines_and_veterans_with_why_not() {
    let (mut s, shrine, plain) = sparta_game();
    let (shrine_pos, plain_pos) = (s.cities[&shrine].pos, s.cities[&plain].pos);
    let ready = veteran(&mut s, shrine_pos, RETIRE_LEVEL);
    let away = veteran(&mut s, plain_pos, RETIRE_LEVEL + 1);
    let _green = veteran(&mut s, shrine_pos, RETIRE_LEVEL - 1);
    let r = view(&s, A).retire.expect("retire view");
    assert_eq!((r.retired, r.target, r.level), (0, 3, RETIRE_LEVEL));
    assert_eq!(r.shrines, vec![shrine]);
    assert_eq!(
        r.veterans
            .iter()
            .map(|v| (v.unit, v.blocker.is_none()))
            .collect::<Vec<_>>(),
        vec![(ready, true), (away, false)]
    );
    assert!(view(&s, B).retire.is_none());
    let mut done = s.clone();
    done.player_mut(A).faction_goal_done = true;
    assert!(view(&done, A).retire.is_none());
}

// ---------- Celts: Call the warbands ----------

/// A Celts empire with two cities that own territory out to radius 3, and a rival.
fn celts_game() -> (GameState, Vec<CityId>) {
    let mut s = game(&[0, 1]);
    s.player_mut(A).faction = Faction::Celts;
    let ids = vec![city(&mut s, 0, hex(3, 3), 4), city(&mut s, 0, hex(8, 6), 4)];
    for id in &ids {
        let pos = s.cities[id].pos;
        for h in pos.within(3) {
            if let Some(t) = s.map.get_mut(h)
                && t.owner.is_none_or(|o| o == A)
            {
                t.owner = Some(A);
                t.city = Some(*id);
            }
        }
    }
    city(&mut s, 1, hex(10, 1), 3);
    (s, ids)
}

fn camps(s: &GameState) -> Vec<(Hex, Option<PlayerId>)> {
    s.barbarians
        .as_ref()
        .map(|b| b.camps.iter().map(|c| (c.pos, c.rite)).collect())
        .unwrap_or_default()
}

#[test]
fn calling_the_wild_raises_a_camp_in_every_cities_territory_even_with_barbarians_off() {
    let (s, ids) = celts_game();
    assert!(s.barbarians.is_none() && !s.setup.barbarians);
    let plan = crate::rules::barbarians::rite_plan(&s, A);
    assert_eq!(plan.len(), 2);
    for (id, at) in &plan {
        let c = &s.cities[id];
        assert_eq!(c.pos.distance(*at), 3, "the farthest tile of its territory");
        assert_eq!(s.map.get(*at).unwrap().city, Some(*id));
    }
    assert_eq!(plan.iter().map(|(c, _)| *c).collect::<Vec<_>>(), ids);
    let mut now = s.clone();
    let mut events = Vec::new();
    apply_actions(&mut now, A, &[act(Action::CallTheWild)], &mut events);
    let raised = camps(&now);
    assert_eq!(raised.len(), 2);
    assert!(raised.iter().all(|(_, rite)| *rite == Some(A)));
    // Ordinary camps: a guard on the tile and a scout beside it.
    for camp in &now.barbarians.as_ref().unwrap().camps {
        let guard = camp.guard.expect("guard");
        assert_eq!(now.units[&guard].pos, camp.pos);
        assert!(now.units[&guard].owner.is_barbarian());
        assert!(
            camp.scout
                .is_some_and(|u| now.units[&u].kind == UnitKind::Scout)
        );
    }
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::WildCalled { player, camps: 2 } if *player == A))
    );
    // Barbarians stay off: no natural camp ever joins the rite's, however long the game runs.
    let mut later = now.clone();
    for _ in 0..30 {
        later = resolve_turn(&later, &no_orders()).0;
    }
    assert!(camps(&later).iter().all(|(_, rite)| rite.is_some()));
}

#[test]
fn a_second_call_fills_only_cities_missing_a_camp() {
    let (mut s, ids) = celts_game();
    apply_actions(&mut s, A, &[act(Action::CallTheWild)], &mut Vec::new());
    assert_eq!(camps(&s).len(), 2);
    assert!(crate::rules::barbarians::rite_plan(&s, A).is_empty());
    let removed = s.barbarians.as_ref().unwrap().camps[0].pos;
    s.barbarians
        .as_mut()
        .unwrap()
        .camps
        .retain(|camp| camp.pos != removed);
    assert_eq!(crate::rules::barbarians::rite_plan(&s, A).len(), 1);
    assert_eq!(crate::rules::barbarians::rite_plan(&s, A)[0].0, ids[0]);
    apply_actions(&mut s, A, &[act(Action::CallTheWild)], &mut Vec::new());
    assert_eq!(
        camps(&s).len(),
        2,
        "the city with a camp receives no duplicate"
    );
    let (with, owned) = goals::wild_camp_cities(&s, A);
    assert_eq!((with, owned), (2, 2));
    // A city whose territory has no valid tile gets none, and the call is refused if no city does.
    let mut tight = s.clone();
    for t in &mut tight.map.tiles {
        if t.city == Some(ids[1]) {
            t.owner = None;
            t.city = None;
        }
    }
    tight.barbarians = None;
    let plan = crate::rules::barbarians::rite_plan(&tight, A);
    assert_eq!(
        plan.iter().map(|(c, _)| *c).collect::<Vec<_>>(),
        vec![ids[0]]
    );
    for t in &mut tight.map.tiles {
        if t.city == Some(ids[0]) {
            t.owner = None;
            t.city = None;
        }
    }
    let errs = validate_orders(&tight, A, &[act(Action::CallTheWild)]);
    assert_eq!(errs.len(), 1);
    assert!(
        errs[0]
            .message
            .contains("No city needs a wild camp or has room")
    );
    // Twice in one turn is flagged; another faction can't.
    let (fresh, _) = celts_game();
    let errs = validate_orders(
        &fresh,
        A,
        &[act(Action::CallTheWild), act(Action::CallTheWild)],
    );
    assert_eq!(errs.iter().map(|e| e.index).collect::<Vec<_>>(), vec![1]);
    assert!(
        validate_orders(&s, B, &[act(Action::CallTheWild)])[0]
            .message
            .contains("Only the Celts")
    );
}

#[test]
fn a_rite_camp_survives_borders_and_stays_outside_the_natural_target() {
    let (mut s, _) = celts_game();
    s.setup.barbarians = true;
    s.barbarians = Some(Default::default());
    apply_actions(&mut s, A, &[act(Action::CallTheWild)], &mut Vec::new());
    assert_eq!(camps(&s).len(), 2);
    assert_eq!(s.barbarians.as_ref().unwrap().natural_camps(), 0);
    // Borders sit over both camps, and a turn ends: nobody disperses them.
    let (next, events) = resolve_turn(&s, &no_orders());
    assert_eq!(camps(&next).iter().filter(|(_, r)| r.is_some()).count(), 2);
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::BarbarianCampCleared { by, unit: None, .. } if *by == A
    )));
}

#[test]
fn the_celts_hold_two_vp_while_every_city_has_a_camp_and_lose_them_when_one_is_cleared() {
    let (mut s, ids) = celts_game();
    assert!(!goals::reached(&s, A));
    let (called, events) = resolve_turn(&s, &orders(0, vec![act(Action::CallTheWild)]));
    assert!(called.player(A).faction_goal_done, "held at the turn's end");
    assert!(events.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted { player, again: false, .. } if *player == A
    )));
    let g = goals::view(&called, A).expect("a live goal always has a meter");
    assert_eq!((g.value, g.target, g.live, g.done), (2, 2, true, true));
    // A rival's soldier clears one camp: the rival gets the Gold, the Celts lose the VP.
    s = called;
    let camp = s.barbarians.as_ref().unwrap().camps[0].clone();
    let gold = s.player(B).gold;
    let clearer = unit(&mut s, 1, UnitKind::Spearman, camp.pos);
    s.units.get_mut(&camp.guard.unwrap()).unwrap().hp = 0;
    s.units.remove(&camp.guard.unwrap());
    let _ = clearer;
    let (after, events) = resolve_turn(&s, &no_orders());
    assert_eq!(camps(&after).len(), 1);
    let reward = crate::rules::barbarians::CAMP_CLEAR_GOLD;
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::BarbarianCampCleared { by, gold: g, .. } if *by == B && *g == reward
        )),
        "the rival gets the normal reward"
    );
    assert!(after.player(B).gold >= gold + reward);
    assert!(!after.player(A).faction_goal_done);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::FactionGoalLost { player, .. } if *player == A
    )));
    let g = goals::view(&after, A).unwrap();
    assert_eq!((g.value, g.target, g.done), (1, 2, false));
    // Calling again raises one in every city and the goal is held again ("again").
    let (again, events) = resolve_turn(&after, &orders(0, vec![act(Action::CallTheWild)]));
    assert!(again.player(A).faction_goal_done);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::FactionGoalCompleted { player, again: true, .. } if *player == A
    )));
    let _ = ids;
}

#[test]
fn the_celts_get_no_gold_for_clearing_their_own_rite_camp() {
    let (mut s, _) = celts_game();
    apply_actions(&mut s, A, &[act(Action::CallTheWild)], &mut Vec::new());
    let camp = s.barbarians.as_ref().unwrap().camps[0].clone();
    s.units.remove(&camp.guard.unwrap());
    let gold = s.player(A).gold;
    unit(&mut s, 0, UnitKind::Spearman, camp.pos);
    let mut events = Vec::new();
    crate::rules::barbarians::clear_camps(&mut s, &mut events);
    assert_eq!(s.player(A).gold, gold, "no reward from the rite's own camp");
    assert!(events.iter().any(|e| matches!(
        e,
        Event::BarbarianCampCleared { by, gold: 0, unit: Some(_), .. } if *by == A
    )));
    assert_eq!(camps(&s).len(), 1);
}

#[test]
fn a_celts_empire_with_one_city_can_hold_the_goal_and_one_without_cities_cannot() {
    let mut s = game(&[0, 1]);
    s.player_mut(A).faction = Faction::Celts;
    let id = city(&mut s, 0, hex(3, 3), 4);
    for h in s.cities[&id].pos.within(3) {
        let t = s.map.get_mut(h).unwrap();
        t.owner = Some(A);
        t.city = Some(id);
    }
    city(&mut s, 1, hex(10, 1), 3);
    assert!(!goals::reached(&s, A));
    apply_actions(&mut s, A, &[act(Action::CallTheWild)], &mut Vec::new());
    assert!(goals::reached(&s, A), "no minimum city count");
    s.cities.remove(&id);
    assert!(!goals::reached(&s, A), "no city, no goal");
}

#[test]
fn the_view_previews_the_camps_the_call_would_raise() {
    let (s, ids) = celts_game();
    let w = view(&s, A).wild.expect("wild view");
    assert!(w.blocker.is_none());
    assert_eq!((w.with_camp, w.cities), (0, 2));
    assert_eq!(w.sites.iter().map(|x| x.city).collect::<Vec<_>>(), ids);
    assert_eq!(
        w.sites.iter().map(|x| x.at).collect::<Vec<_>>(),
        crate::rules::barbarians::rite_plan(&s, A)
            .iter()
            .map(|(_, h)| *h)
            .collect::<Vec<_>>()
    );
    assert!(view(&s, B).wild.is_none());
    let (called, _) = resolve_turn(&s, &orders(0, vec![act(Action::CallTheWild)]));
    let w = view(&called, A).wild.unwrap();
    assert_eq!((w.with_camp, w.cities), (2, 2));
}

#[test]
fn a_young_city_with_only_its_ring_still_gets_a_camp_next_to_it() {
    // One population owns the city and its ring of six tiles: the camp stands on the farthest free
    // one (distance 1), so a freshly founded city can hold the goal too.
    let mut s = game(&[0, 1]);
    s.player_mut(A).faction = Faction::Celts;
    let id = city(&mut s, 0, hex(5, 5), 1);
    city(&mut s, 1, hex(10, 1), 3);
    let plan = crate::rules::barbarians::rite_plan(&s, A);
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].0, id);
    assert_eq!(s.cities[&id].pos.distance(plan[0].1), 1);
    assert_ne!(plan[0].1, s.cities[&id].pos);
    apply_actions(&mut s, A, &[act(Action::CallTheWild)], &mut Vec::new());
    assert!(goals::reached(&s, A));
    // The second call adds no redundant camp to the same young city.
    apply_actions(&mut s, A, &[act(Action::CallTheWild)], &mut Vec::new());
    assert_eq!(camps(&s).len(), 1);
}

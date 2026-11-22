use super::*;
use crate::events::{RuinResearch, RuinReward};
use crate::rules::test_support::*;

fn find(seed: u64, faction: Faction) -> (GameState, UnitId, Vec<Event>) {
    let mut s = game(&[0, 0, 1]);
    s.setup.seed = seed;
    s.players[0].faction = faction;
    city(&mut s, 0, hex(1, 1), 2);
    let u = unit(&mut s, 0, UnitKind::Scout, hex(5, 5));
    s.map.get_mut(hex(5, 5)).unwrap().ruin = true;
    let mut events = vec![];
    explore_ruin(&mut s, u, &mut events);
    (s, u, events)
}

#[test]
fn pinned_ruin_rolls_match_state_and_serialize_structurally() {
    for (seed, expected) in [
        (0, "Science"),
        (5, "Gold"),
        (18, "Population"),
        (9, "FreeScout"),
    ] {
        let (mut state, finder, events) = find(seed, Faction::India);
        // A Population reward claims its tile first (population is territory).
        let found = events
            .iter()
            .find(|e| matches!(e, Event::RuinExplored { .. }))
            .expect("typed find");
        let Event::RuinExplored {
            unit,
            owner,
            at,
            reward,
        } = found
        else {
            panic!("typed find")
        };
        assert_eq!((*unit, *owner, *at), (finder, PlayerId(0), hex(5, 5)));
        assert_eq!(reward.kind_name(), expected, "seed {seed}");
        match reward {
            RuinReward::Gold {
                amount,
                balance,
                affordable,
            } => {
                assert_eq!(*amount, 40);
                assert_eq!(*balance, state.players[0].gold);
                assert!(affordable.is_none());
            }
            RuinReward::Science { amount, .. } => {
                assert_eq!((*amount, state.players[0].science_stored), (25, 25))
            }
            RuinReward::Population {
                city,
                amount,
                population,
                yields,
            } => {
                assert_eq!((*amount, *population), (1, 3));
                assert_eq!(state.cities[city].population, 3);
                assert!(yields.food > 0);
            }
            RuinReward::FreeScout { unit } => {
                let scout = &state.units[unit];
                assert_eq!(
                    (
                        scout.owner,
                        scout.kind,
                        scout.pos,
                        scout.moves_left,
                        scout.goto
                    ),
                    (
                        PlayerId(0),
                        UnitKind::Scout,
                        at.neighbors().into_iter().min().unwrap(),
                        0,
                        None
                    )
                );
                assert_ne!(*unit, finder);
            }
        }
        assert_eq!(
            serde_json::to_value(found).unwrap()["reward"]["kind"],
            expected
        );
        let (again, _, again_events) = find(seed, Faction::India);
        assert_eq!(
            serde_json::to_string(&state).unwrap(),
            serde_json::to_string(&again).unwrap()
        );
        assert_eq!(events, again_events);
        let mut duplicate = vec![];
        explore_ruin(&mut state, finder, &mut duplicate);
        assert!(duplicate.is_empty());
        assert_eq!(state.players[0].ruins_explored, 1);
    }
}

#[test]
fn faction_identity_and_no_city_fallback() {
    for (seed, expected) in [
        (0, "Science"),
        (5, "Gold"),
        (18, "Population"),
        (9, "FreeScout"),
    ] {
        let (athens, _, events) = find(seed, Faction::Athens);
        let (ordinary, _, ordinary_events) = find(seed, Faction::Switzerland);
        let Event::RuinExplored { reward, .. } = events
            .iter()
            .find(|e| matches!(e, Event::RuinExplored { .. }))
            .expect("Athens reward")
        else {
            panic!("Athens reward")
        };
        let Event::RuinExplored {
            reward: ordinary_reward,
            ..
        } = ordinary_events
            .iter()
            .find(|e| matches!(e, Event::RuinExplored { .. }))
            .expect("ordinary reward")
        else {
            panic!("ordinary reward")
        };
        assert_eq!(reward.kind_name(), expected, "seed {seed}");
        assert_eq!(
            reward.kind_name(),
            ordinary_reward.kind_name(),
            "seed {seed}"
        );
        assert_eq!(
            reward, ordinary_reward,
            "normal reward changed for seed {seed}"
        );
        assert_eq!(athens.players[0].gold, ordinary.players[0].gold);
        assert_eq!(
            athens.cities.values().map(|c| c.population).sum::<i32>(),
            ordinary.cities.values().map(|c| c.population).sum::<i32>()
        );
        assert_eq!(
            athens.players[0].science_stored,
            ordinary.players[0].science_stored + 20
        );
        if let RuinReward::Science { amount, .. } = reward {
            assert_eq!(*amount, 25);
        }
    }
    for (seed, amount) in [(0, 25), (5, 40)] {
        let (_, _, events) = find(seed, Faction::Byzantium);
        match &events[0] {
            Event::RuinExplored {
                reward: RuinReward::Gold { amount: n, .. } | RuinReward::Science { amount: n, .. },
                ..
            } => assert_eq!(*n, amount),
            _ => panic!("resource reward"),
        }
    }
    let mut s = game(&[0, 1]);
    s.setup.seed = 18;
    let u = unit(&mut s, 0, UnitKind::Scout, hex(5, 5));
    s.map.get_mut(hex(5, 5)).unwrap().ruin = true;
    let mut events = vec![];
    explore_ruin(&mut s, u, &mut events);
    assert!(matches!(
        events[0],
        Event::RuinExplored {
            reward: RuinReward::Gold { amount: 40, .. },
            ..
        }
    ));
}

#[test]
fn gold_credits_only_a_newly_affordable_current_buy() {
    let mut s = game(&[0, 1]);
    s.setup.seed = 5;
    let c = city(&mut s, 0, hex(1, 1), 2);
    s.cities.get_mut(&c).unwrap().producing = Some(ProductionItem::Unit {
        unit: UnitKind::Scout,
    });
    let cost = crate::economy::buy_cost(&s, &s.cities[&c]).unwrap();
    let u = unit(&mut s, 0, UnitKind::Scout, hex(5, 5));
    for (gold, pivotal) in [(cost - 1, true), (cost, false), (cost - 41, false)] {
        s.players[0].gold = gold;
        s.map.get_mut(hex(5, 5)).unwrap().ruin = true;
        let mut es = vec![];
        explore_ruin(&mut s, u, &mut es);
        let Event::RuinExplored {
            reward: RuinReward::Gold { affordable, .. },
            ..
        } = &es[0]
        else {
            panic!("Gold")
        };
        assert_eq!(affordable.is_some(), pivotal);
        if let Some(buy) = affordable {
            assert_eq!((buy.city, buy.cost), (c, cost));
        }
    }
}

#[test]
fn science_credits_the_threshold_find_with_actual_income_not_an_already_funded_tech() {
    let tech = TechId::BronzeWorking;
    let cost = crate::economy::tech_cost(&game(&[0, 1]), PlayerId(0), tech).total;
    // Two finds of half the cost each. Cases: (banked Science, income, which find completes it).
    // The first find completes it when the bank plus income is within one find of the cost; an
    // already funded tech (bank at the cost) has no pivotal find.
    let find = (cost + 1) / 2;
    assert!(
        5 + find < cost,
        "income 5 plus one find must not fund the tech"
    );
    for (base, income, expected) in [
        (0, 5, Some(1)),
        (cost - find - 5, 5, Some(0)),
        (cost + 15, 5, None),
        (0, 0, Some(1)),
    ] {
        let mut s = game(&[0, 1]);
        s.players[0].researching = Some(tech);
        s.players[0].science_history = vec![income];
        // Two finds collectively fund the tech. Income is already included when attributing it.
        let amount = (cost + 1) / 2;
        s.players[0].science_stored = base + 2 * amount;
        let mut es: Vec<_> = (0..2)
            .map(|n| Event::RuinExplored {
                unit: UnitId(n),
                owner: PlayerId(0),
                at: hex(5, 5),
                reward: RuinReward::Science {
                    amount,
                    research: None,
                },
            })
            .collect();
        super::super::cities::process_research(&mut s, &mut es);
        let pivotal: Vec<_> = es
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match e {
                Event::RuinExplored {
                    reward:
                        RuinReward::Science {
                            research:
                                Some(RuinResearch {
                                    completes: true, ..
                                }),
                            ..
                        },
                    ..
                } => Some(i),
                _ => None,
            })
            .collect();
        assert_eq!(
            pivotal,
            expected.into_iter().collect::<Vec<_>>(),
            "base {base}, income {income}, cost {cost}"
        );
        assert!(es.iter().any(|e| matches!(e, Event::TechResearched { .. })));
    }
}

#[test]
fn free_scout_uses_a_free_passable_neighbour_or_falls_back_to_gold() {
    for faction in [Faction::India, Faction::Byzantium] {
        for blocked in 0..=6 {
            let mut s = game(&[0, 0, 1]);
            s.setup.seed = 9;
            s.players[0].faction = faction;
            let at = hex(5, 5);
            let finder = unit(&mut s, 0, UnitKind::Scout, at);
            let mut neighbours = at.neighbors();
            neighbours.sort();
            for (i, &h) in neighbours.iter().take(blocked).enumerate() {
                match i {
                    0 => s.map.get_mut(h).unwrap().terrain = Terrain::Ocean,
                    1 => s.map.get_mut(h).unwrap().terrain = Terrain::Mountains,
                    2 => {
                        unit(&mut s, 1, UnitKind::Scout, h);
                    }
                    3 => {
                        unit(&mut s, 2, UnitKind::Settler, h);
                    }
                    4 => {
                        city(&mut s, 2, h, 2);
                    }
                    _ => {
                        unit(&mut s, 0, UnitKind::Spearman, h);
                    }
                }
            }
            s.map.get_mut(at).unwrap().ruin = true;
            let gold = s.players[0].gold;
            let mut events = vec![];
            explore_ruin(&mut s, finder, &mut events);
            match &events[0] {
                Event::RuinExplored {
                    reward: RuinReward::FreeScout { unit },
                    ..
                } => {
                    assert!(blocked < 6);
                    assert_eq!(s.units[unit].pos, neighbours[blocked]);
                    assert_eq!(s.units[unit].moves_left, 0);
                }
                Event::RuinExplored {
                    reward:
                        RuinReward::Gold {
                            amount, balance, ..
                        },
                    ..
                } => {
                    assert_eq!(blocked, 6);
                    assert_eq!(*amount, 40);
                    assert_eq!(*balance, gold + amount);
                    assert_eq!(s.players[0].gold, *balance);
                }
                e => panic!("unexpected reward: {e:?}"),
            }
            let mut occupied = std::collections::BTreeSet::new();
            assert!(
                s.units
                    .values()
                    .filter(|u| data::unit_stats(u.kind).military)
                    .all(|u| occupied.insert(u.pos))
            );
        }
    }
}

#[test]
fn free_scout_cannot_spawn_across_peace_borders_or_on_peaceful_civilians() {
    for closed_border in [false, true] {
        let mut s = game(&[0, 1]);
        s.setup.seed = 9;
        s.diplomacy.as_mut().unwrap().relations[0].state = crate::diplomacy::RelationState::Peace;
        let at = hex(5, 5);
        let finder = unit(&mut s, 0, UnitKind::Scout, at);
        for h in at.neighbors() {
            if closed_border {
                s.map.get_mut(h).unwrap().owner = Some(PlayerId(1));
            } else {
                unit(&mut s, 1, UnitKind::Settler, h);
            }
        }
        s.map.get_mut(at).unwrap().ruin = true;
        let count = s.units.len();
        let mut events = vec![];
        explore_ruin(&mut s, finder, &mut events);
        assert!(matches!(
            events[0],
            Event::RuinExplored {
                reward: RuinReward::Gold { amount: 40, .. },
                ..
            }
        ));
        assert_eq!(s.units.len(), count);
    }
}

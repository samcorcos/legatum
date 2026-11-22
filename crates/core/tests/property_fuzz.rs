//! B5: deterministic property coverage through the public core API; no external fuzz runtime.
//!
//! Default: 8 seeds x 40 active resolved turns (each resolved twice), plus boundary fixtures.
//! Larger run, using the same assertions and generator:
//! LEGATUM_FUZZ_CASES=256 LEGATUM_FUZZ_TURNS=100 \
//!   cargo test --release -p legatum-core --test property_fuzz -- --nocapture
//! Single-seed replay: LEGATUM_FUZZ_START_SEED=<seed> LEGATUM_FUZZ_CASES=1 \
//!   LEGATUM_FUZZ_TURNS=<step+1> cargo test -p legatum-core --test property_fuzz \
//!   seeded_sequences -- --exact --nocapture
//!
//! Constants follow the current eight-faction engine and fair-start rules. City HP may
//! be zero pending capture; surviving units may not. Unknown/dead players' entire order
//! lists are dropped silently by live_orders, unlike invalid orders from a living player.
//! Isolated arenas below are explicit boundary fixtures, never fed into the reachable-state
//! campaign. Engine failures stay failing; catch_unwind only adds replay context/collects cases.

mod property_fuzz_support;

use legatum_core::data;
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rng::Rng;
use legatum_core::state::Map;
use legatum_core::*;
use property_fuzz_support::*;

fn env_number(name: &str, default: u64) -> u64 {
    match std::env::var(name) {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|_| panic!("{name} must be a u64, got {value:?}")),
        Err(std::env::VarError::NotPresent) => default,
        Err(error) => panic!("{name}: {error}"),
    }
}

fn fresh(seed: u64) -> GameState {
    let setup = if seed.is_multiple_of(2) {
        let seat = (seed % Faction::ALL.len() as u64) as usize;
        GameSetup::standard(
            seed,
            [
                Faction::ALL[seat],
                Faction::ALL[(seat + 1) % Faction::ALL.len()],
            ],
        )
    } else {
        GameSetup::all_ai(seed)
    };
    new_game(&setup)
}

fn context<'a>(name: &'a str, state: &'a GameState, orders: &'a Orders) -> Case<'a> {
    Case {
        name,
        seed: state.setup.seed,
        step: state.turn as usize - 1,
        state,
        orders,
    }
}

fn sequence(seed: u64, turns: usize, coverage: &mut Coverage) -> Result<(), String> {
    let mut state = attempt(|| fresh(seed)).map_err(|why| {
        format!("seed={seed} turn=0 input orders={{}} invariant=new_game never panics: {why}")
    })?;
    let empty = Orders::new();
    let initial = Case {
        name: "seeded_sequences",
        seed,
        step: 0,
        state: &state,
        orders: &empty,
    };
    let replay = attempt(|| fresh(seed)).map_err(|why| initial.failure(why))?;
    if state != replay || state.hash() != replay.hash() {
        return Err(initial.failure("new_game state/hash determinism"));
    }
    invariants(&state).map_err(|why| initial.failure(why))?;
    start_invariants(&state).map_err(|why| initial.failure(why))?;
    let mut episode = 0;
    for step in 0..turns {
        // Do not pad the quota with finished-game no-ops. If a team wins early, first
        // exercise the terminal state, then start another untouched generated game.
        if state.finished {
            let (_, raw) = generate(&state, seed, step);
            let terminal = Case {
                name: "seeded_sequences",
                seed,
                step,
                state: &state,
                orders: &raw,
            };
            exercise(&terminal)?;
            episode += 1;
            let next_seed = Rng::derive(seed, &[0x0052_4553_5441_5254, episode]).next_u64();
            let next = attempt(|| fresh(next_seed)).map_err(|why| terminal.failure(why))?;
            start_invariants(&next).map_err(|why| {
                terminal.failure(format!("restarted game seed={next_seed}: {why}"))
            })?;
            state = next;
        }
        let (legal, raw) = generate(&state, seed, step);
        let legal_case = Case {
            name: "seeded_sequences",
            seed,
            step,
            state: &state,
            orders: &legal,
        };
        let errors = validation(&legal_case)?;
        if errors.values().any(|errors| !errors.is_empty()) {
            return Err(
                legal_case.failure(format!("generated legal backbone validates: {errors:?}"))
            );
        }
        coverage.confirmed_paths += legal
            .values()
            .flatten()
            .filter(|o| matches!(o, Order::MoveUnitPath { .. }))
            .count();
        coverage.legal_orders += legal.values().map(Vec::len).sum::<usize>();
        let case = Case {
            orders: &raw,
            ..legal_case
        };
        let validated = validation(&case);
        // Even a validator panic must not prevent the resolver seeing the original input.
        let resolved = resolution(&case);
        let errors = validated?;
        coverage.rejected_orders += errors.values().map(Vec::len).sum::<usize>();
        let (next, events) = resolved?;
        coverage.observe(&state, &events);
        state = next;
    }
    Ok(())
}

#[test]
fn seeded_sequences() {
    let start = env_number("LEGATUM_FUZZ_START_SEED", 0);
    let seeds = env_number("LEGATUM_FUZZ_CASES", 8);
    let turns =
        usize::try_from(env_number("LEGATUM_FUZZ_TURNS", 40)).expect("turn count fits usize");
    assert!(
        seeds > 0 && turns > 0,
        "seed and turn budgets must be positive"
    );
    let mut coverage = Coverage::default();
    let mut failures = Vec::new();
    for index in 0..seeds {
        let seed = start.wrapping_add(index);
        if let Err(failure) = sequence(seed, turns, &mut coverage) {
            failures.push(failure);
        }
    }
    eprintln!("B5 start_seed={start} seeds={seeds} turns_per_seed={turns} {coverage:?}");
    assert!(
        failures.is_empty(),
        "{} seeded failures:\n{}",
        failures.len(),
        failures.join("\n\n")
    );
    assert_eq!(coverage.active_turns as u64, seeds * turns as u64);
    // Prevent a passing campaign that quietly exercises only rejected orders or empty turns.
    // Smaller explicit replay budgets need not encounter every mechanic.
    if seeds >= 8 && turns >= 40 {
        assert!(
            coverage.legal_orders > 0
                && coverage.rejected_orders > 0
                && coverage.confirmed_paths > 0,
            "{coverage:?}"
        );
        assert!(
            coverage.failures > 0 && coverage.moves > 0 && coverage.battles > 0,
            "{coverage:?}"
        );
        assert!(
            coverage.contests > 0 && coverage.founded > 0 && coverage.produced > 0,
            "{coverage:?}"
        );
        assert!(
            coverage.upkeep_turns > 0 && coverage.buys > 0,
            "{coverage:?}"
        );
    }
}

/// Small valid occupancy fixtures isolate boundaries; reserve settlers keep every player
/// alive during full resolve_turn, rather than bypassing elimination with a phase helper.
fn arena(seed: u64, teams: &[u8]) -> GameState {
    let setup = GameSetup {
        seed,
        width: 18,
        height: 12,
        turn_limit: Some(150),
        legacy_all_war: false,
        barbarians: false,
        rule_experiments: Default::default(),
        players: teams
            .iter()
            .enumerate()
            .map(|(i, &team)| PlayerSetup {
                name: format!("P{i}"),
                faction: Faction::ALL
                    [((seed % Faction::ALL.len() as u64) as usize + i) % Faction::ALL.len()],
                team: TeamId(team),
                is_human: false,
                difficulty: AiDifficulty::Normal,
                search: false,
            })
            .collect(),
    };
    let mut state = blank_game(
        &setup,
        Map::new(setup.width, setup.height, Terrain::Grassland),
    );
    // Existing battle-boundary arenas explicitly start at War; new-game campaigns below
    // submit declarations and exercise the peaceful opening boundary as well.
    for r in &mut state.diplomacy.as_mut().unwrap().relations {
        r.state = diplomacy::RelationState::War;
    }
    for (i, _) in teams.iter().enumerate() {
        spawn_unit(
            &mut state,
            PlayerId(i as u8),
            UnitKind::Settler,
            Hex::from_offset(1 + i as i32 * 6, 10),
        );
    }
    state
}

fn add_city(state: &mut GameState, player: PlayerId, pos: Hex) -> CityId {
    let settler = spawn_unit(state, player, UnitKind::Settler, pos);
    legatum_core::rules::cities::found_city(state, settler, &mut Vec::new())
        .expect("fixture city site")
}

#[test]
fn invariant_checker_rejects_corrupted_states() {
    let name = "invariant_checker_rejects_corrupted_states";
    let mut valid = arena(101, &[0, 0, 1]);
    let pos = Hex::new(3, 3);
    let military = spawn_unit(&mut valid, PlayerId(0), UnitKind::Soldier, pos);
    let civilian = spawn_unit(&mut valid, PlayerId(1), UnitKind::Settler, pos);
    let city = add_city(&mut valid, PlayerId(0), Hex::new(7, 3));
    let orders = Orders::new();
    let case = context(name, &valid, &orders);
    case.require(
        invariants(&valid).is_ok(),
        "same-team military + civilian sharing is valid",
    );
    let mut corruptions = Vec::new();
    for hp in [i32::MIN, -1, 0, data::UNIT_HP + 1, i32::MAX] {
        let mut bad = valid.clone();
        bad.units.get_mut(&military).unwrap().hp = hp;
        corruptions.push(("unit HP", bad));
    }
    for hp in [i32::MIN, -1, data::CITY_HP + 1, i32::MAX] {
        let mut bad = valid.clone();
        bad.cities.get_mut(&city).unwrap().hp = hp;
        corruptions.push(("city HP", bad));
    }
    for (kind, expected) in [
        (UnitKind::Ranger, "military stacking"),
        (UnitKind::Scout, "military stacking"),
        (UnitKind::Settler, "civilian stacking"),
    ] {
        let mut bad = valid.clone();
        spawn_unit(&mut bad, PlayerId(0), kind, pos);
        corruptions.push((expected, bad));
    }
    let mut bad = valid.clone();
    bad.units.get_mut(&civilian).unwrap().owner = PlayerId(2);
    corruptions.push(("team sharing", bad));
    let mut bad = valid.clone();
    bad.units.get_mut(&military).unwrap().pos = valid.cities[&city].pos;
    bad.units.get_mut(&military).unwrap().owner = PlayerId(2);
    corruptions.push(("team sharing", bad));
    for gold in [-1, i32::MIN] {
        let mut bad = valid.clone();
        bad.player_mut(PlayerId(0)).gold = gold;
        corruptions.push(("nonnegative gold", bad));
    }
    for (expected, bad) in corruptions {
        let result = invariants(&bad);
        case.require(
            result.as_ref().is_err_and(|why| why.starts_with(expected)),
            format!("checker rejects deliberately corrupted {expected}: {result:?}"),
        );
    }
    for hp in [1, data::UNIT_HP] {
        let mut edge = valid.clone();
        edge.units.get_mut(&military).unwrap().hp = hp;
        edge.cities.get_mut(&city).unwrap().hp = 0;
        case.require(invariants(&edge).is_ok(), "inclusive valid HP endpoints");
    }
}

#[test]
fn boundary_missing_foreign_and_duplicate_orders() {
    let name = "boundary_missing_foreign_and_duplicate_orders";
    let initial = fresh(u64::MAX);
    let founding: Orders = initial
        .players
        .iter()
        .map(|p| {
            (
                p.id,
                initial
                    .units_of(p.id)
                    .filter(|u| u.kind == UnitKind::Settler)
                    .map(|u| Order::FoundCity { unit: u.id })
                    .collect(),
            )
        })
        .collect();
    let (state, _) = checked(&context(name, &initial, &founding));
    let p = PlayerId(0);
    let own = state.units_of(p).next().unwrap();
    let foreign = state.units_of(PlayerId(1)).next().unwrap();
    let own_city = state.cities_of(p).next().unwrap().id;
    let foreign_city = state.cities_of(PlayerId(1)).next().unwrap().id;
    let good = Orders::from([(p, vec![Order::SkipUnit { unit: own.id }])]);
    let raw = Orders::from([
        (
            p,
            vec![
                Order::SkipUnit { unit: own.id },
                Order::DisbandUnit { unit: own.id },
                Order::MoveUnit {
                    unit: own.id,
                    to: foreign.pos,
                },
                Order::DisbandUnit { unit: foreign.id },
                Order::FoundCity { unit: UnitId(0) },
                Order::UpgradeUnit {
                    unit: UnitId(u32::MAX),
                },
                Order::SetProduction {
                    city: foreign_city,
                    item: ProductionItem::Unit {
                        unit: UnitKind::Settler,
                    },
                },
                Order::BuyItem {
                    city: CityId(0),
                    item: ProductionItem::Unit {
                        unit: UnitKind::Soldier,
                    },
                },
                Order::SetProduction {
                    city: CityId(u32::MAX),
                    item: ProductionItem::Unit {
                        unit: UnitKind::Soldier,
                    },
                },
                Order::BuyItem {
                    city: own_city,
                    item: ProductionItem::Unit {
                        unit: UnitKind::Soldier,
                    },
                }, // idle city
                Order::SetResearch {
                    tech: TechId::Philosophy,
                },
            ],
        ),
        (PlayerId(u8::MAX), vec![Order::DisbandUnit { unit: own.id }]),
    ]);
    let case = context(name, &state, &raw);
    let errors = validation(&case).unwrap();
    case.require(
        errors[&p].iter().map(|e| e.index).eq(1..raw[&p].len()),
        "all invalid order indices reported",
    );
    case.require(
        errors[&PlayerId(u8::MAX)].len() == 1,
        "unknown player rejected",
    );
    let (next, events) = checked(&case);
    let (expected, _) = checked(&context(name, &state, &good));
    case.require(
        next == expected,
        "invalid/missing/foreign/duplicate orders have no state effects",
    );
    case.require(
        events
            .iter()
            .filter(|e| matches!(e, Event::OrderFailed { player, .. } if *player == p))
            .count()
            == raw[&p].len() - 1,
        "each invalid live-player order emits failure",
    );
}

fn coordinate_boundaries(state: &GameState) -> Vec<Hex> {
    let mut out = vec![
        Hex::from_offset(-1, 0),
        Hex::from_offset(state.map.width, 0),
        Hex::from_offset(0, -1),
        Hex::from_offset(0, state.map.height),
        Hex::new(i32::MAX, 2),
        Hex::new(i32::MIN, -2),
    ];
    for q in [i32::MIN, 0, i32::MAX] {
        for r in [i32::MIN, 0, i32::MAX] {
            if q != 0 || r != 0 {
                out.push(Hex::new(q, r));
            }
        }
    }
    out
}

#[test]
fn hex_distance_and_offset_match_wide_reference() {
    fn check(a: Hex, b: Hex) {
        // Independent cube-coordinate oracle: the largest absolute delta equals distance.
        let dq = i64::from(a.q) - i64::from(b.q);
        let dr = i64::from(a.r) - i64::from(b.r);
        let distance = dq.abs().max(dr.abs()).max((dq + dr).abs());
        let expected = distance.min(i64::from(i32::MAX)) as i32;
        assert_eq!(a.distance(b), expected, "distance {a:?} -> {b:?}");
        assert_eq!(b.distance(a), expected, "reverse distance {a:?} -> {b:?}");
        assert_eq!(a.distance(a), 0, "self distance {a:?}");

        let col = i64::from(a.q) + i64::from(a.r).div_euclid(2);
        let expected_col = col.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
        assert_eq!(a.to_offset(), (expected_col, a.r), "offset {a:?}");
        // Saturation deliberately loses information; only representable offsets round-trip.
        if i32::try_from(col).is_ok() {
            assert_eq!(Hex::from_offset(expected_col, a.r), a, "round trip {a:?}");
        }
    }

    let edges = [
        i32::MIN,
        i32::MIN + 1,
        -2,
        -1,
        0,
        1,
        2,
        i32::MAX - 1,
        i32::MAX,
    ];
    let points: Vec<_> = edges
        .into_iter()
        .flat_map(|q| edges.into_iter().map(move |r| Hex::new(q, r)))
        .collect();
    for &a in &points {
        for &b in &points {
            check(a, b);
        }
    }
    let mut rng = Rng::derive(
        env_number("LEGATUM_FUZZ_START_SEED", 0),
        &[0x4845_585f_4d41_5448],
    );
    for _ in 0..env_number("LEGATUM_FUZZ_CASES", 8) {
        for _ in 0..128 {
            let a = Hex::new(rng.next_u64() as i32, rng.next_u64() as i32);
            let b = Hex::new(rng.next_u64() as i32, rng.next_u64() as i32);
            check(a, b);
        }
    }
}

fn reject_coordinates(name: &str, state: &GameState, unit: UnitId, ranged: bool) {
    let player = state.units[&unit].owner;
    let mut failures = Vec::new();
    for target in coordinate_boundaries(state) {
        let order = if ranged {
            Order::RangedAttack { unit, target }
        } else {
            Order::MoveUnit { unit, to: target }
        };
        let orders = Orders::from([(player, vec![order])]);
        let case = context(name, state, &orders);
        // Both APIs are probed even if one panics. Do not prevalidate/filter the order.
        let validated = validation(&case);
        let resolved = resolution(&case);
        match validated {
            Ok(errors) if errors[&player].len() == 1 => {}
            Ok(errors) => {
                failures.push(case.failure(format!("out-of-bounds order rejected: {errors:?}")))
            }
            Err(failure) => failures.push(failure),
        }
        match resolved {
            Ok((_, events))
                if events
                    .iter()
                    .any(|e| matches!(e, Event::OrderFailed { .. })) => {}
            Ok(_) => failures.push(case.failure("out-of-bounds order emits OrderFailed")),
            Err(failure) => failures.push(failure),
        }
    }
    assert!(
        failures.is_empty(),
        "{} boundary failures:\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

#[test]
fn boundary_extreme_move_coordinates_are_nonfatal() {
    let state = fresh(0);
    let unit = state
        .units_of(PlayerId(0))
        .find(|u| u.kind == UnitKind::Soldier)
        .unwrap()
        .id;
    reject_coordinates(
        "boundary_extreme_move_coordinates_are_nonfatal",
        &state,
        unit,
        false,
    );
}

#[test]
fn boundary_extreme_ranged_coordinates_are_nonfatal() {
    let mut state = arena(103, &[0, 1]);
    let unit = spawn_unit(&mut state, PlayerId(0), UnitKind::Ranger, Hex::new(3, 3));
    reject_coordinates(
        "boundary_extreme_ranged_coordinates_are_nonfatal",
        &state,
        unit,
        true,
    );
}

#[test]
fn boundary_simultaneous_contests() {
    let name = "boundary_simultaneous_contests";
    for seed in 0..32 {
        for teams in [[0, 0, 0], [0, 0, 1], [0, 1, 2]] {
            for civilian_mask in 0..8 {
                let mut state = arena(seed, &teams);
                // This invariant covers ordinary meeting combat. Mongols claimants
                // exercise their separate pass-through rule in movement parity tests.
                for player in &mut state.players {
                    player.faction = Faction::Switzerland;
                }
                let target = Hex::new(5, 4);
                let mut orders = Orders::new();
                for i in 0..3 {
                    let kind = if civilian_mask & (1 << i) == 0 {
                        UnitKind::Soldier
                    } else {
                        UnitKind::Settler
                    };
                    let id = spawn_unit(
                        &mut state,
                        PlayerId(i),
                        kind,
                        target.neighbor(usize::from(i) * 2),
                    );
                    // Cover surviving claimants and deaths, without corrupting health bounds.
                    state.units.get_mut(&id).unwrap().hp =
                        [1, 35, data::UNIT_HP][(seed as usize + usize::from(i)) % 3];
                    orders.insert(
                        PlayerId(i),
                        vec![Order::MoveUnit {
                            unit: id,
                            to: target,
                        }],
                    );
                }
                let case = context(name, &state, &orders);
                let errors = validation(&case).unwrap();
                case.require(
                    errors.values().all(Vec::is_empty),
                    "each simultaneous claimant has a legal order",
                );
                let (_, events) = checked(&case);
                if civilian_mask == 0 && teams == [0, 1, 2] {
                    case.require(
                        events
                            .iter()
                            .filter(|e| matches!(e, Event::Battle { meeting: true, .. }))
                            .count()
                            == 3,
                        "three hostile claimants fight each pair once from a snapshot",
                    );
                }
                if civilian_mask == 0 && teams == [0, 0, 0] {
                    case.require(
                        events.iter().any(|e| matches!(e, Event::Bounce { .. })),
                        "friendly same-class claimants bounce",
                    );
                }
            }
        }
    }
}

#[test]
fn boundary_simultaneous_founding() {
    let name = "boundary_simultaneous_founding";
    for gap in [data::MIN_CITY_GAP, data::MIN_CITY_GAP + 1] {
        for teams in [[0, 0], [0, 1]] {
            let mut state = arena(104, &teams);
            let a = spawn_unit(&mut state, PlayerId(0), UnitKind::Settler, Hex::new(2, 3));
            let b = spawn_unit(
                &mut state,
                PlayerId(1),
                UnitKind::Settler,
                Hex::new(2 + gap, 3),
            );
            let orders = Orders::from([
                (PlayerId(0), vec![Order::FoundCity { unit: a }]),
                (PlayerId(1), vec![Order::FoundCity { unit: b }]),
            ]);
            let case = context(name, &state, &orders);
            let (next, events) = checked(&case);
            let founded = events
                .iter()
                .filter(|e| matches!(e, Event::CityFounded { .. }))
                .count();
            if gap <= data::MIN_CITY_GAP {
                case.require(
                    founded == 0 && next.units.contains_key(&a) && next.units.contains_key(&b),
                    "conflicting founders all survive and neither city is founded",
                );
            } else {
                case.require(
                    founded == 2 && !next.units.contains_key(&a) && !next.units.contains_key(&b),
                    "minimum legal city distance permits both founders",
                );
            }
        }
    }
}

#[test]
fn boundary_unaffordable_buys_and_upgrades() {
    let name = "boundary_unaffordable_buys_and_upgrades";
    let p = PlayerId(0);
    let mut base = arena(105, &[0, 1]);
    let city = add_city(&mut base, p, Hex::new(3, 3));
    let city_pos = base.cities[&city].pos;
    let soldier = spawn_unit(&mut base, p, UnitKind::Soldier, city_pos);
    base.player_mut(p).techs.extend([
        TechId::BronzeWorking,
        TechId::IronWorking,
        TechId::Masonry,
        TechId::Mathematics,
    ]);
    // The Phalangite uses 1 Iron (the Classical era's resource): an owned deposit supplies it.
    let iron = Hex::new(4, 3);
    let tile = base.map.get_mut(iron).unwrap();
    tile.resource = Some(legatum_core::Resource::Iron);
    tile.owner = Some(p);
    let upgrade_cost =
        legatum_core::rules::units::upgrade_cost_for(base.player(p).faction, UnitKind::Soldier)
            .unwrap();
    let buy_cost = economy::buy_cost_for(
        &base,
        &base.cities[&city],
        ProductionItem::Unit {
            unit: UnitKind::Soldier,
        },
    )
    .unwrap();
    for gold in [0, upgrade_cost, upgrade_cost + buy_cost - 1] {
        let mut state = base.clone();
        state.player_mut(p).gold = gold;
        let orders = Orders::from([(
            p,
            vec![
                Order::SetProduction {
                    city,
                    item: ProductionItem::Unit {
                        unit: UnitKind::Settler,
                    },
                },
                Order::BuyItem {
                    city,
                    item: ProductionItem::Unit {
                        unit: UnitKind::Soldier,
                    },
                }, // appears first, but unit upgrades are charged first
                Order::UpgradeUnit { unit: soldier },
                Order::BuyItem {
                    city,
                    item: ProductionItem::Unit {
                        unit: UnitKind::Soldier,
                    },
                },
            ],
        )]);
        let case = context(name, &state, &orders);
        let errors = validation(&case).unwrap();
        case.require(
            errors[&p].iter().any(|e| e.index == 1),
            "buy is unaffordable after unit-phase spending",
        );
        case.require(
            errors[&p].iter().any(|e| e.index == 2) == (gold < upgrade_cost),
            "upgrade affordability boundary",
        );
        let (next, events) = checked(&case);
        case.require(
            !events.iter().any(|e| matches!(e, Event::ItemBought { .. })),
            "unaffordable buys never complete",
        );
        case.require(
            next.units[&soldier].kind
                == if gold >= upgrade_cost {
                    UnitKind::Phalangite
                } else {
                    UnitKind::Soldier
                },
            "only affordable upgrades change kind",
        );
    }
}

#[test]
fn boundary_production_and_upkeep() {
    let name = "boundary_production_and_upkeep";
    let mut state = arena(106, &[0, 1]);
    let p = PlayerId(0);
    let city = add_city(&mut state, p, Hex::new(4, 4));
    // The Byzantium's levy cap would keep production blocked after the upkeep thins the army.
    state.player_mut(p).faction = legatum_core::Faction::Switzerland;
    state.cities.get_mut(&city).unwrap().production_stored =
        data::unit_stats_for(state.player(p).faction, UnitKind::Soldier).cost;
    // Occupy all seven spawn tiles, making production wait until upkeep creates space.
    for pos in state.cities[&city].pos.within(1) {
        spawn_unit(&mut state, p, UnitKind::Soldier, pos);
    }
    let mut coverage = Coverage::default();
    for _ in 0..12 {
        let orders = Orders::from([(
            p,
            vec![Order::SetProduction {
                city,
                item: ProductionItem::Unit {
                    unit: UnitKind::Soldier,
                },
            }],
        )]);
        let case = context(name, &state, &orders);
        let (next, events) = checked(&case);
        if state.turn == 1 {
            case.require(
                !events
                    .iter()
                    .any(|e| matches!(e, Event::ProductionCompleted { .. })),
                "blocked production does not overstack units",
            );
            case.require(
                next.player(p).gold == 0,
                "upkeep resets an insolvent treasury to zero",
            );
        }
        coverage.observe(&state, &events);
        state = next;
    }
    let orders = Orders::new();
    let case = context(name, &state, &orders);
    case.require(
        coverage.produced > 0 && coverage.upkeep_disbands > 0,
        format!("production and upkeep both exercised: {coverage:?}"),
    );
}

#[test]
fn boundary_zero_hp_city_is_valid_until_capture() {
    let name = "boundary_zero_hp_city_is_valid_until_capture";
    let mut state = arena(107, &[0, 1]);
    let pos = Hex::new(4, 4);
    let city = add_city(&mut state, PlayerId(1), pos);
    state.cities.get_mut(&city).unwrap().hp = 1;
    let ranger = spawn_unit(&mut state, PlayerId(0), UnitKind::Ranger, pos.neighbor(0));
    let orders = Orders::from([(
        PlayerId(0),
        vec![Order::RangedAttack {
            unit: ranger,
            target: pos,
        }],
    )]);
    let case = context(name, &state, &orders);
    let (next, _) = checked(&case);
    case.require(
        next.cities[&city].hp == 0 && next.cities[&city].owner == PlayerId(1),
        "ranged-damaged city can remain at zero HP without capture",
    );
}

#[test]
fn boundary_finished_and_eliminated_players() {
    let name = "boundary_finished_and_eliminated_players";
    let mut setup = GameSetup::all_ai(u64::MAX - 1);
    setup.turn_limit = Some(2);
    let mut state = new_game(&setup);
    let former_unit = state.units_of(PlayerId(0)).next().unwrap().id;
    let capital = state.capital_of(PlayerId(0)).unwrap().id;
    state.units.retain(|_, unit| unit.owner != PlayerId(0));
    state.cities.remove(&capital);
    for tile in &mut state.map.tiles {
        if tile.city == Some(capital) {
            tile.city = None;
            tile.owner = None;
        }
    }
    let orders = Orders::new();
    let (next, _) = checked(&context(name, &state, &orders));
    state = next;
    let raw = Orders::from([(
        PlayerId(0),
        vec![
            Order::SkipUnit { unit: former_unit },
            Order::SetResearch {
                tech: TechId::Irrigation,
            },
        ],
    )]);
    let case = context(name, &state, &raw);
    case.require(
        !state.player(PlayerId(0)).alive,
        "disbanding last settler eliminates a cityless empire",
    );
    case.require(
        validation(&case).unwrap()[&PlayerId(0)].len() == 2,
        "every eliminated-player order rejected",
    );
    let (next, _) = checked(&case);
    let (empty_next, _) = checked(&context(name, &state, &Orders::new()));
    case.require(
        next == empty_next,
        "eliminated-player orders are ignored at resolution",
    );
    state = next;
    let case = context(name, &state, &raw);
    case.require(
        state.finished,
        "turn-limit fixture reaches a finished game through resolution",
    );
    for _ in 0..3 {
        let (next, events) = checked(&case);
        case.require(
            next == state && next.hash() == state.hash() && events.is_empty(),
            "finished game is an immutable no-op",
        );
    }
}

#[test]
fn boundary_ruin_scout_spawn_preserves_occupancy() {
    let mut gifts = 0;
    // Exercise ruin rewards through movement and the same invariant checker used after
    // every fuzz turn, including the next round's occupancy map and subsequent turns.
    for seed in 0..64 {
        let mut state = arena(seed, &[0, 1]);
        state.players[0].faction = Faction::India;
        state.players[0].gold = 10000;
        let at = Hex::new(5, 4);
        let finder = spawn_unit(&mut state, PlayerId(0), UnitKind::Scout, at.neighbor(0));
        state.map.get_mut(at).unwrap().ruin = true;
        let orders = Orders::from([(
            PlayerId(0),
            vec![Order::MoveUnit {
                unit: finder,
                to: at,
            }],
        )]);
        let (next, events) = checked(&context("boundary_ruin_scout_spawn", &state, &orders));
        if let Some(unit) = events.iter().find_map(|e| match e {
            Event::RuinExplored {
                reward: legatum_core::events::RuinReward::FreeScout { unit },
                ..
            } => Some(*unit),
            _ => None,
        }) {
            gifts += 1;
            assert_eq!(next.units[&unit].pos.distance(at), 1);
            assert_ne!(next.units[&unit].pos, next.units[&finder].pos);
        }
        checked(&context(
            "boundary_ruin_scout_next_turn",
            &next,
            &Orders::new(),
        ));
    }
    assert!(gifts > 0, "campaign exercises at least one Scout gift");
}

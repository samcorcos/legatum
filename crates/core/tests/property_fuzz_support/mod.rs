//! Test-only generators and oracles. Campaign states come exclusively from new_game/resolve_turn.

use legatum_core::data;
use legatum_core::economy;
use legatum_core::mapgen;
use legatum_core::rng::Rng;
use legatum_core::rules::units::order_unit;
use legatum_core::state::Unit;
use legatum_core::*;
use std::collections::BTreeMap;
use std::panic::{UnwindSafe, catch_unwind};

pub type Orders = BTreeMap<PlayerId, Vec<Order>>;

// Separate from the engine's own random streams; never seeded from a clock or thread id.
const FUZZ_STREAM: u64 = 0x4235_4655_5a5a;

pub fn pick<T: Copy>(rng: &mut Rng, values: &[T]) -> T {
    values[rng.range(0, values.len() as i64 - 1) as usize]
}

/// Independent occupancy oracle: no movement/spawn helper is reused to decide validity.
/// Territory ownership is deliberately not occupancy: armies may enter enemy territory.
pub fn invariants(state: &GameState) -> Result<(), String> {
    let mut classes = BTreeMap::new();
    let mut teams = BTreeMap::new();
    for unit in state.units.values() {
        if state.map.move_cost(unit.pos).is_none() {
            return Err(format!(
                "unit position: {:?} at {:?} is not passable",
                unit.id, unit.pos
            ));
        }
        if !(1..=data::UNIT_HP).contains(&unit.hp) {
            return Err(format!("unit HP: {:?} has {}", unit.id, unit.hp));
        }
        let military = data::unit_stats(unit.kind).military;
        if let Some(other) = classes.insert((unit.pos, military), unit.id) {
            return Err(format!(
                "{} stacking: {other:?} and {:?} at {:?}",
                if military { "military" } else { "civilian" },
                unit.id,
                unit.pos
            ));
        }
        occupy(state, &mut teams, unit.pos, unit.owner)?;
    }
    for city in state.cities.values() {
        if state.map.move_cost(city.pos).is_none() {
            return Err(format!(
                "city position: {:?} at {:?} is not passable",
                city.id, city.pos
            ));
        }
        // battle::apply_fights clamps cities to zero. Ranged fire can leave an uncaptured
        // city at zero, and adjacent enemies suppress healing; only units must have HP > 0.
        if !(0..=data::CITY_HP).contains(&city.hp) {
            return Err(format!("city HP: {:?} has {}", city.id, city.hp));
        }
        if city.owner.is_barbarian() {
            return Err(format!("barbarian city: {:?}", city.id));
        }
        occupy(state, &mut teams, city.pos, city.owner)?;
    }
    for player in &state.players {
        if player.gold < 0 {
            return Err(format!(
                "nonnegative gold: {:?} has {}",
                player.id, player.gold
            ));
        }
    }
    Ok(())
}

/// Check fair starts using the initial capitals, without rerunning map generation.
pub fn start_invariants(state: &GameState) -> Result<(), String> {
    let starts: Vec<_> = state
        .players
        .iter()
        .map(|p| {
            let units: Vec<_> = state.units_of(p.id).collect();
            let mut kinds: Vec<_> = units.iter().map(|u| u.kind).collect();
            kinds.sort();
            if kinds != [UnitKind::Scout, UnitKind::Soldier] {
                return Err(format!("starting units: {:?} has {kinds:?}", p.id));
            }
            let start = state
                .capital_of(p.id)
                .ok_or(format!("starting capital: {:?}", p.id))?
                .pos;
            if units.iter().any(|u| {
                u.hp != data::UNIT_HP
                    || u.moves_left != data::unit_stats_for(p.faction, u.kind).moves
                    || u.pos.distance(start) != 1
            }) {
                return Err(format!("starting escorts/HP/faction moves: {:?}", p.id));
            }
            Ok(start)
        })
        .collect::<Result<_, _>>()?;
    for (i, &start) in starts.iter().enumerate() {
        if starts[i + 1..]
            .iter()
            .any(|&h| start.distance(h) < mapgen::START_MIN_DISTANCE)
        {
            return Err(format!("start spacing: seat {i} at {start:?}"));
        }
        let workable = start
            .within(mapgen::START_RADIUS)
            .into_iter()
            .filter(|&h| state.map.move_cost(h).is_some())
            .count();
        if workable < mapgen::START_MIN_WORKABLE {
            return Err(format!("start workable land: seat {i} has {workable}"));
        }
        // Standard setups have one allied pair; all-AI setups have only rivals.
        for (j, &ally) in starts
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i && state.players[j].team == state.players[i].team)
        {
            if starts.iter().enumerate().any(|(k, &rival)| {
                state.players[k].team != state.players[i].team
                    && (start.distance(ally) >= start.distance(rival)
                        || start.distance(ally) >= ally.distance(rival))
            }) {
                return Err(format!("start teammate proximity: seats {i} and {j}"));
            }
        }
    }
    let qualities: Vec<_> = (0..starts.len())
        .map(|i| mapgen::start_quality(&state.map, &starts, &state.setup, i))
        .collect();
    let low = *qualities.iter().min().ok_or("no starts")?;
    let high = *qualities.iter().max().unwrap();
    if low < mapgen::START_MIN_QUALITY || high - low > mapgen::START_MAX_QUALITY_SPREAD {
        return Err(format!("fair start quality: {qualities:?}"));
    }
    Ok(())
}

fn occupy(
    state: &GameState,
    teams: &mut BTreeMap<Hex, TeamId>,
    pos: Hex,
    owner: PlayerId,
) -> Result<(), String> {
    // Barbarian units exist only in games with barbarians on, on their own team.
    let team = if owner.is_barbarian() && state.barbarians.is_some() {
        TeamId::BARBARIAN
    } else {
        state
            .players
            .get(owner.0 as usize)
            .ok_or_else(|| format!("occupant owner: unknown {owner:?}"))?
            .team
    };
    if let Some(other) = teams.insert(pos, team)
        && other != team
    {
        return Err(format!("team sharing: {other:?} and {team:?} at {pos:?}"));
    }
    Ok(())
}

pub struct Case<'a> {
    pub name: &'a str,
    pub seed: u64,
    pub step: usize,
    pub state: &'a GameState,
    pub orders: &'a Orders,
}

impl Case<'_> {
    pub fn failure(&self, invariant: impl std::fmt::Display) -> String {
        format!(
            "case={} seed={} step={} game_seed={} turn={} input_hash={}\n\
             invariant: {invariant}\ninput orders: {}\n\
             replay: LEGATUM_FUZZ_START_SEED={} LEGATUM_FUZZ_CASES=1 \
             LEGATUM_FUZZ_TURNS={} cargo test -p legatum-core --test property_fuzz {} \
             -- --exact --nocapture",
            self.name,
            self.seed,
            self.step,
            self.state.setup.seed,
            self.state.turn,
            self.state.hash(),
            serde_json::to_string(self.orders).expect("orders serialize"),
            self.seed,
            self.step + 1,
            self.name,
        )
    }

    pub fn require(&self, condition: bool, invariant: impl std::fmt::Display) {
        assert!(condition, "{}", self.failure(invariant));
    }
}

pub fn attempt<T>(f: impl FnOnce() -> T + UnwindSafe) -> Result<T, String> {
    catch_unwind(f).map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
            .unwrap_or_else(|| "non-string panic".to_owned())
    })
}

pub fn validation(case: &Case<'_>) -> Result<BTreeMap<PlayerId, Vec<OrderError>>, String> {
    let mut results = BTreeMap::new();
    for (&player, orders) in case.orders {
        let a = attempt(|| validate_orders(case.state, player, orders));
        let b = attempt(|| validate_orders(case.state, player, orders));
        let (a, b) = a.and_then(|a| b.map(|b| (a, b))).map_err(|panic| {
            case.failure(format!(
                "validate_orders never panics for {player:?}: {panic}"
            ))
        })?;
        if a != b {
            return Err(case.failure(format!("validation determinism for {player:?}")));
        }
        if a.iter().any(|error| error.index >= orders.len())
            || a.windows(2).any(|pair| pair[0].index >= pair[1].index)
        {
            return Err(case.failure("validation error indices are unique, ordered and in bounds"));
        }
        results.insert(player, a);
    }
    Ok(results)
}

pub fn resolution(case: &Case<'_>) -> Result<(GameState, Vec<Event>), String> {
    invariants(case.state).map_err(|why| case.failure(format!("input {why}")))?;
    let before = case.state.clone();
    // Run BOTH calls even if the first panics. A caught panic is an error, never a passing test.
    let a = attempt(|| resolve_turn(case.state, case.orders));
    let b = attempt(|| resolve_turn(case.state, case.orders));
    let (a, b) = a.and_then(|a| b.map(|b| (a, b))).map_err(|panic| {
        case.failure(format!("resolve_turn never panics on raw orders: {panic}"))
    })?;
    if a.0 != b.0 || a.0.hash() != b.0.hash() || a.1 != b.1 {
        return Err(case.failure("resolution state/hash/event determinism"));
    }
    if *case.state != before {
        return Err(case.failure("resolution preserves the input state"));
    }
    if case.state.finished {
        if a.0 != before || !a.1.is_empty() {
            return Err(case.failure("finished games return the unchanged state and no events"));
        }
    } else if a.0.turn != before.turn + 1 {
        return Err(case.failure("one active resolution advances exactly one turn"));
    }
    invariants(&a.0).map_err(|why| case.failure(why))?;
    Ok(a)
}

/// Validation does not gate resolution: adversarial orders go into the resolver unchanged,
/// including when the validator panics. This also gives each boundary two independent probes.
pub fn exercise(case: &Case<'_>) -> Result<(GameState, Vec<Event>), String> {
    let checked = validation(case);
    let resolved = resolution(case);
    match (checked, resolved) {
        (Ok(_), Ok(result)) => Ok(result),
        (Err(a), Err(b)) => Err(format!("{a}\n{b}")),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}

pub fn checked(case: &Case<'_>) -> (GameState, Vec<Event>) {
    exercise(case).unwrap_or_else(|failure| panic!("{failure}"))
}

#[derive(Default, Debug)]
pub struct Coverage {
    pub active_turns: usize,
    pub legal_orders: usize,
    pub confirmed_paths: usize,
    pub rejected_orders: usize,
    pub failures: usize,
    pub moves: usize,
    pub battles: usize,
    pub contests: usize,
    pub founded: usize,
    pub produced: usize,
    pub buys: usize,
    pub upkeep_turns: usize,
    pub upkeep_disbands: usize,
    pub finishes: usize,
}

impl Coverage {
    pub fn observe(&mut self, before: &GameState, events: &[Event]) {
        self.active_turns += usize::from(!before.finished);
        self.upkeep_turns += usize::from(before.players.iter().any(|p| {
            before
                .units_of(p.id)
                .filter(|u| data::pays_upkeep(u.kind))
                .count()
                > data::FREE_MILITARY_UNITS as usize
        }));
        for event in events {
            match event {
                Event::OrderFailed { .. } => self.failures += 1,
                Event::UnitMoved { .. } => self.moves += 1,
                Event::Battle { meeting, .. } => {
                    self.battles += 1;
                    self.contests += usize::from(*meeting);
                }
                Event::Bounce { .. } => self.contests += 1,
                Event::CityFounded { .. } => self.founded += 1,
                Event::ProductionCompleted { .. } => self.produced += 1,
                Event::ItemBought { .. } => self.buys += 1,
                Event::UnitDisbanded { reason, .. } if reason.contains("upkeep") => {
                    self.upkeep_disbands += 1;
                }
                Event::GameWon { .. } | Event::GameDrawn { .. } => self.finishes += 1,
                _ => {}
            }
        }
    }
}

fn can_found(state: &GameState, owner: PlayerId, pos: Hex) -> bool {
    state.map.move_cost(pos).is_some()
        && state
            .map
            .get(pos)
            .is_some_and(|t| t.owner.is_none_or(|p| p == owner))
        && state
            .cities
            .values()
            .all(|c| c.pos.distance(pos) > data::MIN_CITY_GAP)
}

fn unit_order(state: &GameState, unit: &Unit, rng: &mut Rng) -> Order {
    let id = unit.id;
    let view = player_view(state, unit.owner);
    if unit.kind == UnitKind::Heir {
        // A civilian with no order of its own: he never founds, attacks or marches.
        return Order::SkipUnit { unit: id };
    }
    if unit.kind == UnitKind::Settler {
        if can_found(state, unit.owner, unit.pos) {
            return Order::FoundCity { unit: id };
        }
        let sites: Vec<_> = unit
            .pos
            .within(5)
            .into_iter()
            .filter(|&h| can_found(state, unit.owner, h))
            .filter(|&h| {
                view.movement_path(unit.owner, unit.kind, unit.pos, h)
                    .is_some()
            })
            .collect();
        return sites
            .first()
            .map_or(Order::SkipUnit { unit: id }, |_| Order::MoveUnit {
                unit: id,
                to: pick(rng, &sites),
            });
    }
    if unit.kind == UnitKind::Ranger
        && state.passage_allowed(
            unit.owner,
            unit.kind,
            state.map.get(unit.pos).and_then(|t| t.owner),
        )
    {
        let targets: Vec<_> = view
            .units
            .iter()
            .filter(|u| u.military && state.at_war(unit.owner, u.owner))
            .map(|u| u.pos)
            .chain(
                view.cities
                    .iter()
                    .filter(|c| state.at_war(unit.owner, c.owner))
                    .map(|c| c.pos),
            )
            .filter(|h| {
                (1..=data::unit_stats_for(state.player(unit.owner).faction, unit.kind).range)
                    .contains(&unit.pos.distance(*h))
                    && view.tile(*h).is_some_and(|t| t.visible)
                    && los::visible(unit.pos, *h, |p| view.tile(p).and_then(|t| t.terrain))
            })
            .collect();
        if !targets.is_empty() {
            return Order::RangedAttack {
                unit: id,
                target: pick(rng, &targets),
            };
        }
    }
    if rng.chance(8) {
        return Order::SkipUnit { unit: id };
    }
    if rng.chance(2) && state.units_of(unit.owner).count() > 5 {
        return Order::DisbandUnit { unit: id };
    }
    // Frequently gather toward the same enemy. Other draws use local destinations; both
    // friendly bounces and hostile contacts arise without writing positions into the state.
    let enemy = state
        .units
        .values()
        .filter(|u| state.at_war(unit.owner, u.owner))
        // Every hostile unit on the tile counts: a civilian can share it with a mounted unit.
        .filter(|u| {
            !state.units.values().any(|o| {
                o.pos == u.pos
                    && state.at_war(unit.owner, o.owner)
                    && legatum_core::combat::outrider_blocks(
                        unit.kind,
                        o.kind,
                        unit.pos.distance(o.pos),
                    )
            })
        })
        .min_by_key(|u| (unit.pos.distance(u.pos), u.id));
    if rng.chance(65)
        && let Some(enemy) = enemy
        && view
            .movement_path(unit.owner, unit.kind, unit.pos, enemy.pos)
            .is_some()
    {
        return Order::MoveUnit {
            unit: id,
            to: enemy.pos,
        };
    }
    let nearby: Vec<_> = unit
        .pos
        .within(2)
        .into_iter()
        .filter(|&h| state.map.move_cost(h).is_some())
        .filter(|&h| {
            view.movement_path(unit.owner, unit.kind, unit.pos, h)
                .is_some()
        })
        .filter(|&h| {
            !state.units.values().any(|u| {
                u.pos == h
                    && state.at_war(unit.owner, u.owner)
                    && legatum_core::combat::outrider_blocks(
                        unit.kind,
                        u.kind,
                        unit.pos.distance(h),
                    )
            })
        })
        .collect();
    if nearby.is_empty() {
        return Order::SkipUnit { unit: id };
    }
    Order::MoveUnit {
        unit: id,
        to: pick(rng, &nearby),
    }
}

/// Build a legal backbone without repairing or filtering the adversarial list. The caller
/// separately asserts this backbone validates, then submits the mixed list verbatim.
pub fn generate(state: &GameState, seed: u64, step: usize) -> (Orders, Orders) {
    let mut legal = Orders::new();
    for player in state.players.iter().filter(|p| p.alive) {
        let p = player.id;
        let mut rng = Rng::derive(seed, &[FUZZ_STREAM, step as u64, u64::from(p.0)]);
        let mut list: Vec<_> = state
            .units_of(p)
            // An occupation-locked unit may not leave its city, so the legal backbone only skips it.
            .map(|u| {
                if legatum_core::rules::occupation::lock(state, u).is_some() {
                    Order::SkipUnit { unit: u.id }
                } else {
                    unit_order(state, u, &mut rng)
                }
            })
            .collect();
        let view = player_view(state, p);
        for (index, order) in list.iter_mut().enumerate() {
            if (step + index).is_multiple_of(2)
                && let Order::MoveUnit { unit, to } = order
                && let Some(path) =
                    legatum_core::path::find_path(&state.map, state.units[unit].pos, *to, |h| {
                        view.tile(h)?.terrain.and_then(Terrain::move_cost)
                    })
                && !path.is_empty()
            {
                let candidate = Order::MoveUnitPath {
                    unit: *unit,
                    to: *to,
                    path,
                    stop_on_block: false,
                };
                if validate_orders(state, p, std::slice::from_ref(&candidate)).is_empty() {
                    *order = candidate;
                }
            }
        }
        for city in state.cities_of(p) {
            if city.producing.is_none() {
                let candidates: Vec<_> = UnitKind::ALL
                    .into_iter()
                    .map(|unit| ProductionItem::Unit { unit })
                    .chain(
                        BuildingKind::ALL
                            .into_iter()
                            .map(|building| ProductionItem::Building { building }),
                    )
                    .filter(|&item| economy::can_produce(state, city, item))
                    // A strategic item also needs spare Iron or Coal supply
                    // source: territory now follows population, so a city often lacks one.
                    .filter(|&item| {
                        legatum_core::resources::start_blocker(state, city, item).is_none()
                    })
                    .collect();
                if !candidates.is_empty() {
                    list.push(Order::SetProduction {
                        city: city.id,
                        item: pick(&mut rng, &candidates),
                    });
                }
            }
        }
        let techs: Vec<_> = TechId::ALL
            .into_iter()
            .filter(|&t| economy::tech_available(state, p, t))
            .collect();
        if !techs.is_empty() && (player.researching.is_none() || rng.chance(10)) {
            list.push(Order::SetResearch {
                tech: pick(&mut rng, &techs),
            });
        }
        if state.turn == 1 {
            for against in state.teams().into_iter().filter(|t| *t != player.team) {
                list.push(Order::DeclareWar {
                    against,
                    breach: false,
                });
            }
        }
        legal.insert(p, list);
    }
    let mut mixed = legal.clone();
    for (&p, list) in &mut mixed {
        let mut rng = Rng::derive(seed, &[FUZZ_STREAM + 1, step as u64, u64::from(p.0)]);
        list.push(Order::SkipUnit {
            unit: UnitId(u32::MAX),
        });
        list.push(Order::BuyItem {
            city: CityId(u32::MAX),
            item: ProductionItem::Unit {
                unit: UnitKind::Soldier,
            },
        });
        if let Some(foreign) = state.units.values().find(|u| u.owner != p) {
            list.push(Order::DisbandUnit { unit: foreign.id });
        }
        if let Some(foreign) = state.cities.values().find(|c| c.owner != p) {
            list.push(Order::SetProduction {
                city: foreign.id,
                item: ProductionItem::Unit {
                    unit: UnitKind::Settler,
                },
            });
        }
        // Reserve a unit for a bad FIRST action so this does not merely test duplicate
        // rejection. Most other units retain their generated legal action.
        if state.turn > 1
            && let Some(unit) = state.units_of(p).find(|u| u.kind != UnitKind::Settler)
        {
            let bad = match (step + usize::from(p.0)) % 9 {
                0 => Order::FoundCity { unit: unit.id },
                1 => Order::UpgradeUnit { unit: unit.id },
                2 => Order::MoveUnit {
                    unit: unit.id,
                    to: Hex::new(-1, -1),
                },
                3 => Order::RangedAttack {
                    unit: unit.id,
                    target: unit.pos,
                },
                6 => Order::MoveUnitPath {
                    unit: unit.id,
                    to: unit.pos,
                    path: vec![],
                    stop_on_block: false,
                },
                7 => Order::MoveUnitPath {
                    unit: unit.id,
                    to: Hex::new(i32::MAX, i32::MIN),
                    path: vec![Hex::new(i32::MAX, i32::MIN)],
                    stop_on_block: false,
                },
                8 => Order::MoveUnitPath {
                    unit: unit.id,
                    to: unit.pos,
                    path: vec![unit.pos, unit.pos],
                    stop_on_block: false,
                },
                4 => Order::MoveUnit {
                    unit: unit.id,
                    to: Hex::from_offset(state.map.width, state.map.height),
                },
                _ => Order::MoveUnit {
                    unit: unit.id,
                    to: state
                        .map
                        .hexes()
                        .find(|&h| state.map.move_cost(h).is_none())
                        .expect("generated map has water"),
                },
            };
            // Small-map out-of-bounds here; the entire i32 domain is exercised in dedicated
            // boundary tests, so an overflow cannot prevent all long campaigns from running.
            if let Some(index) = list.iter().position(|o| order_unit(o) == Some(unit.id)) {
                list[index] = bad;
            }
            list.push(Order::SkipUnit { unit: unit.id });
            list.push(Order::DisbandUnit { unit: unit.id });
        }
        for city in state.cities_of(p) {
            // Exercise full-price purchases even in this short opening campaign.
            // No treasury is injected: choose the cheapest currently available item,
            // and still submit both copies verbatim to test affordability/duplicates.
            let item = UnitKind::ALL
                .into_iter()
                .map(|unit| ProductionItem::Unit { unit })
                .chain(
                    BuildingKind::ALL
                        .into_iter()
                        .map(|building| ProductionItem::Building { building }),
                )
                .filter_map(|item| {
                    economy::buy_cost_for(state, city, item).map(|cost| (cost, item))
                })
                .min_by_key(|(cost, _)| *cost)
                .map_or(
                    ProductionItem::Unit {
                        unit: UnitKind::Soldier,
                    },
                    |(_, item)| item,
                );
            list.push(Order::BuyItem {
                city: city.id,
                item,
            });
            list.push(Order::BuyItem {
                city: city.id,
                item,
            });
            if rng.chance(25) {
                list.push(Order::SetProduction {
                    city: city.id,
                    item: ProductionItem::Building {
                        building: BuildingKind::RelayUplink,
                    },
                });
            }
        }
        list.push(Order::SetResearch {
            tech: TechId::Philosophy,
        });
    }
    mixed.insert(
        PlayerId(u8::MAX),
        vec![
            Order::DisbandUnit { unit: UnitId(1) },
            Order::BuyItem {
                city: CityId(1),
                item: ProductionItem::Unit {
                    unit: UnitKind::Soldier,
                },
            },
        ],
    );
    (legal, mixed)
}

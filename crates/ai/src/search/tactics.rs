//! Move-level tactical search, skateboard (Hard+Search option `x`): an attack on an enemy city
//! with a visible garrison.
//!
//! When some of our military stands within two hexes of such a city (at war with its owner),
//! the units there get four candidate order sets for this turn; the scripted brain's orders
//! stand for every other unit:
//! - **default**: the scripted orders unchanged;
//! - **focus**: every unit that legally can attacks the city (ranged fire; melee only at even
//!   or better odds of winning);
//! - **spread**: ranged units shoot the nearest enemy unit in range other than the garrison
//!   (the city if none); melee holds;
//! - **wait**: every unit there holds.
//!
//! Each set is resolved this turn on the determinized states, then the scripted policy plays
//! `tactic_horizon` turns, and the set with the best mean `material` (army, cities, city HP,
//! captures) is played. Ties keep the scripted default. Every decision is logged, so how often
//! the choice differs from the default is visible.

use super::*;
use legatum_core::forecast::{AttackMode, forecast_attack_mode};
use legatum_core::view::UnitView;
use legatum_core::{Hex, Order, UnitId};

/// The candidate order sets, in this order; 0 is the scripted default.
pub const TACTICS: [&str; 4] = ["default", "focus", "spread", "wait"];

/// The unit an order acts for, if it is a unit action.
fn unit_of(o: &Order) -> Option<UnitId> {
    match o {
        Order::MoveUnit { unit, .. }
        | Order::MoveUnitPath { unit, .. }
        | Order::SleepUnit { unit, .. }
        | Order::SkipUnit { unit }
        | Order::RangedAttack { unit, .. }
        | Order::UpgradeUnit { unit }
        | Order::SetAutomation { unit, .. } => Some(*unit),
        _ => None,
    }
}

/// The engagement this turn, if any: the enemy city at war with a visible garrison that has
/// the most of our military within two hexes, and those units.
fn engagement(view: &PlayerView) -> Option<(Hex, Vec<&UnitView>)> {
    view.cities
        .iter()
        .filter(|c| {
            c.visible
                && !c.owner.is_barbarian()
                && view.at_war(view.me, c.owner)
                && view
                    .units
                    .iter()
                    .any(|u| u.pos == c.pos && u.military && u.owner == c.owner)
        })
        .map(|c| {
            let near: Vec<&UnitView> = view
                .units
                .iter()
                .filter(|u| {
                    u.owner == view.me
                        && u.military
                        && u.kind != legatum_core::UnitKind::Scout
                        && u.pos.distance(c.pos) <= 2
                })
                .collect();
            (c.pos, near)
        })
        .filter(|(_, near)| !near.is_empty())
        .max_by_key(|(pos, near)| (near.len(), std::cmp::Reverse(*pos)))
}

/// The order set `tactic` for the engaged units, replacing their scripted orders.
fn variant(
    view: &PlayerView,
    scripted: &[Order],
    city: Hex,
    units: &[&UnitView],
    tactic: usize,
) -> Vec<Order> {
    if tactic == 0 {
        return scripted.to_vec();
    }
    let ids: Vec<UnitId> = units.iter().map(|u| u.id).collect();
    let mut out: Vec<Order> = scripted
        .iter()
        .filter(|o| unit_of(o).is_none_or(|u| !ids.contains(&u)))
        .cloned()
        .collect();
    let ranged_at = |u: &UnitView, target: Hex| {
        forecast_attack_mode(view, u.id, target, AttackMode::Ranged).is_some()
    };
    for u in units {
        let order = match TACTICS[tactic] {
            "focus" => {
                let charge = crate::composition::charges(u.kind)
                    && u.pos.distance(city) == 1
                    && forecast_attack_mode(view, u.id, city, AttackMode::Melee)
                        .is_some_and(|f| f.win_chance >= 50);
                // A unit with both attacks (the Keshig) charges when that wins, else shoots.
                if u.ranged > 0
                    && ranged_at(u, city)
                    && !(charge && crate::composition::dual(u.kind, u.ranged))
                {
                    Order::RangedAttack {
                        unit: u.id,
                        target: city,
                    }
                } else if charge {
                    Order::MoveUnit {
                        unit: u.id,
                        to: city,
                    }
                } else {
                    Order::SkipUnit { unit: u.id }
                }
            }
            "spread" => {
                let target = view
                    .units
                    .iter()
                    .filter(|e| {
                        e.military
                            && e.pos != city
                            && view.at_war(view.me, e.owner)
                            && u.ranged > 0
                            && ranged_at(u, e.pos)
                    })
                    .min_by_key(|e| (e.pos.distance(u.pos), e.hp, e.id))
                    .map(|e| e.pos)
                    .or_else(|| (u.ranged > 0 && ranged_at(u, city)).then_some(city));
                match target {
                    Some(target) => Order::RangedAttack { unit: u.id, target },
                    None => Order::SkipUnit { unit: u.id },
                }
            }
            _ => Order::SkipUnit { unit: u.id },
        };
        out.push(order);
    }
    out
}

/// Choose this turn's order set for the engagement (if any) by rollouts. Returns the orders
/// to play and the logged decision.
pub(crate) fn choose(
    brain: &ScriptedBrain,
    config: &SearchConfig,
    view: &PlayerView,
    scripted: Vec<Order>,
) -> (Vec<Order>, Option<SearchEvent>) {
    let Some((city, units)) = engagement(view) else {
        return (scripted, None);
    };
    let mut sets: Vec<Vec<Order>> = (0..TACTICS.len())
        .map(|t| variant(view, &scripted, city, &units, t))
        .collect();
    let oracle = brain.oracle.as_deref().filter(|_| config.omniscient);
    let starts = starts(config, brain, oracle, view);
    let horizon = SearchConfig::turns(config.tactic_horizon.max(1));
    let jobs: Vec<(usize, usize)> = (0..sets.len())
        .flat_map(|k| (0..starts.len()).map(move |s| (k, s)))
        .collect();
    let results = map_jobs(&jobs, |(k, s)| {
        let (state, seed, variant) = &starts[*s];
        rollout_first(
            state.clone(),
            view.me,
            &sets[*k],
            rollout_seat(
                brain.rollout_copy_with(config.rollout_policy),
                config.rollout_policy,
                view.turn,
            ),
            horizon,
            *seed,
            *variant,
            if config.tactic_captures {
                material_captures
            } else {
                material
            },
        )
    });
    let mut per = vec![Vec::new(); sets.len()];
    for ((k, _), r) in jobs.iter().zip(results) {
        per[*k].push(r);
    }
    let means: Vec<i32> = per
        .iter()
        .map(|r| r.iter().map(|x| *x as i64).sum::<i64>() / r.len().max(1) as i64)
        .map(|m| m.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
        .collect();
    // Ties keep the scripted default (index 0), then the list order.
    let chosen = (0..sets.len())
        .max_by_key(|i| (means[*i], std::cmp::Reverse(*i)))
        .expect("tactics");
    // Sets that issue the same orders as the default are recorded as such.
    let same_as_default: Vec<bool> = sets.iter().map(|s| *s == sets[0]).collect();
    let event = SearchEvent {
        turn: view.turn,
        chance: crate::winchance::WinFacts::of(view).chance(),
        options: (0..sets.len())
            .map(|i| SearchOption {
                kind: PlanKind::Economy,
                opening: None,
                posture: Some(format!(
                    "tactic:{}{}",
                    TACTICS[i],
                    if i > 0 && same_as_default[i] {
                        "=default"
                    } else {
                        ""
                    }
                )),
                value: units.len() as i32,
                mean: means[i],
                samples: per[i].clone(),
                beyond: 0,
            })
            .collect(),
        chosen,
        analytic: 0,
        omniscient: oracle.is_some(),
        trigger: "tactics".into(),
        horizon,
    };
    (sets.swap_remove(chosen), Some(event))
}

/// `rollout_with` scored by `eval`, with the seat's orders this turn fixed to `first`.
#[allow(clippy::too_many_arguments)]
fn rollout_first(
    mut state: GameState,
    me: PlayerId,
    first: &[Order],
    seat: Box<dyn Brain>,
    horizon: u32,
    seed: u64,
    variant: u8,
    eval: fn(&GameState, PlayerId) -> i32,
) -> i32 {
    let start = state.turn;
    let mut brains: BTreeMap<PlayerId, Box<dyn Brain>> = state
        .players
        .iter()
        .filter(|p| p.alive && p.id != me)
        .map(|p| {
            let s = Rng::derive(seed, &[STREAM, p.id.0 as u64]).next_u64();
            (p.id, rival_brain(p.faction, s, p.id, variant, start))
        })
        .collect();
    brains.insert(me, seat);
    let end = state.turn + horizon;
    let mut turn0 = true;
    super::with_rollout_views(|cache| {
        while !state.finished && state.turn < end {
            let public = legatum_core::public_view(&state);
            let orders: BTreeMap<PlayerId, Vec<Order>> = brains
                .iter_mut()
                .filter(|(id, _)| state.player(**id).alive)
                .map(|(id, b)| {
                    if turn0 && *id == me {
                        return (*id, first.to_vec());
                    }
                    if b.rests(state.turn) {
                        return (*id, Vec::new());
                    }
                    let view = legatum_core::player_view_cached(
                        &state,
                        *id,
                        &public,
                        legatum_core::ViewDetail::Decisions,
                        cache,
                    );
                    let o = b.orders(&view);

                    (*id, o)
                })
                .collect();
            turn0 = false;
            state = legatum_core::resolve_turn(&state, &orders).0;
        }
    });
    eval(&state, me)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use legatum_core::UnitKind;

    #[test]
    fn a_garrisoned_city_attack_offers_focus_spread_and_wait_for_the_engaged_units_only() {
        let city = Hex::new(10, 8);
        let mut v = ViewBuilder::new(24, 16)
            .city(9, ENEMY, city, 3)
            .unit(20, ENEMY, UnitKind::Spearman, city)
            .unit(21, ENEMY, UnitKind::Spearman, Hex::new(12, 8))
            .unit(1, ME, UnitKind::Archer, Hex::new(9, 7))
            .unit(2, ME, UnitKind::Pikeman, Hex::new(9, 8))
            .unit(3, ME, UnitKind::Spearman, Hex::new(2, 2))
            .build();
        v.diplomacy.legacy = true; // every rival at war
        let (at, units) = engagement(&v).expect("engagement");
        assert_eq!(at, city);
        assert_eq!(units.len(), 2, "the far Spearman isn't engaged");
        let far = Order::MoveUnit {
            unit: UnitId(3),
            to: Hex::new(3, 2),
        };
        let scripted = vec![
            Order::SkipUnit { unit: UnitId(1) },
            Order::SkipUnit { unit: UnitId(2) },
            far.clone(),
        ];
        for (t, name) in TACTICS.iter().enumerate() {
            let set = variant(&v, &scripted, city, &units, t);
            assert!(set.contains(&far), "{name}: other units keep their orders");
            assert_eq!(
                set.iter().filter(|o| unit_of(o) == Some(UnitId(1))).count(),
                1,
                "{name}: one order per engaged unit"
            );
        }
        let focus = variant(&v, &scripted, city, &units, 1);
        assert!(focus.contains(&Order::RangedAttack {
            unit: UnitId(1),
            target: city
        }));
        let wait = variant(&v, &scripted, city, &units, 3);
        assert!(wait.contains(&Order::SkipUnit { unit: UnitId(2) }));
    }
}

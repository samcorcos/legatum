//! Client-side scout automation. Only PlayerView is accepted: no hidden state or RNG.
use crate::analysis::{Ctx, UNREACHABLE};
use legatum_core::view::UnitView;
use legatum_core::{Automation, ExploreStopReason, Hex, Order, PlayerView, UnitId};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const EXPLORE_MIN_HP: i32 = 60;

/// Shared with the empire AI's civilian/scout planner. Lower is better.
pub(crate) fn frontier_score(ctx: &Ctx<'_>, h: Hex, distance: i32, bonus: i32) -> i32 {
    distance * 10
        - ctx.unexplored_near(h) * 3
        - if ctx.view.tile(h).is_some_and(|t| t.ruin) {
            bonus
        } else {
            0
        }
}

/// One turn's route, or an explicit stop order (with an optional final retreat).
/// None means this is not an owned unit that can auto-explore (a Scout or a land military
/// unit) or the game has finished.
/// Routes use known land and avoid known armies. Retreats can cross out of a threat
/// radius, but only while reducing danger and reaching safe ground this turn.
pub fn explore_step(view: &PlayerView, unit: UnitId) -> Option<Order> {
    let scout = view.units.iter().find(|u| {
        u.id == unit && u.owner == view.me && legatum_core::data::can_auto_explore(u.kind)
    })?;
    if view.finished {
        return None;
    }
    let ctx = Ctx::new(view);
    let danger_at = |h: Hex| {
        ctx.threat_units
            .iter()
            .filter(|e| e.military)
            .map(|e| (e.range.max(e.max_moves).max(1) + 1 - h.distance(e.pos)).max(0))
            .chain(
                ctx.enemy_cities
                    .iter()
                    .filter(|c| legatum_core::city_fires(c))
                    .map(|c| (3 - h.distance(c.pos)).max(0)),
            )
            .max()
            .unwrap_or(0)
    };
    let unsafe_at = |h: Hex| danger_at(h) > 0;
    let blocked = |h: Hex| {
        view.index(h).is_some_and(|i| ctx.neutral_at[i])
            || ctx.enemy_cities.iter().any(|c| c.pos == h)
            || view.units.iter().any(|u| {
                u.id != unit
                    && u.pos == h
                    && (u.military || ctx.enemy_units.iter().any(|e| e.id == u.id))
            })
    };
    // One stable Dijkstra field supplies both reachability and the exact safe route.
    let mut dist = vec![UNREACHABLE; view.tiles.len()];
    let mut prev = vec![None; view.tiles.len()];
    let start = view.index(scout.pos)?;
    dist[start] = 0;
    let mut queue = BinaryHeap::from([Reverse((0, scout.pos))]);
    while let Some(Reverse((cost, from))) = queue.pop() {
        if dist[view.index(from)?] != cost {
            continue;
        }
        for to in from.neighbors() {
            let Some(i) = view.index(to) else { continue };
            let Some(step) = ctx.move_cost_light(to) else {
                continue;
            };
            if (unsafe_at(to) && danger_at(to) >= danger_at(from)) || blocked(to) {
                continue;
            }
            if cost + step < dist[i] {
                dist[i] = cost + step;
                prev[i] = Some(from);
                queue.push(Reverse((cost + step, to)));
            }
        }
    }
    let route = |goal: Hex| -> Vec<Hex> {
        let mut at = goal;
        let mut path = Vec::new();
        while at != scout.pos {
            path.push(at);
            let Some(from) = view.index(at).and_then(|i| prev[i]) else {
                return Vec::new();
            };
            at = from;
        }
        path.reverse();
        let mut moves = scout.max_moves;
        path.into_iter()
            .take_while(|h| {
                let cost = ctx.move_cost_light(*h).expect("known route");
                let can = legatum_core::rules::movement::can_step(moves, scout.max_moves, cost);
                moves = (moves - cost).max(0);
                can
            })
            .collect()
    };
    let threatened = unsafe_at(scout.pos);
    let stop = if scout.hp < EXPLORE_MIN_HP {
        Some(ExploreStopReason::Damaged)
    } else if threatened {
        Some(ExploreStopReason::Threatened)
    } else {
        None
    };
    if scout.hp < 100 || threatened {
        // Rest in safe friendly territory, otherwise move toward the nearest safe home.
        // Without a city, prefer safer ground; when already safe, resting heals in place.
        let home = |h: Hex| {
            ctx.own_cities
                .iter()
                .map(|c| c.pos.distance(h))
                .min()
                .unwrap_or(0)
        };
        let mut choices: Vec<_> = dist
            .iter()
            .enumerate()
            .filter(|(_, d)| **d != UNREACHABLE)
            .map(|(i, d)| (ctx.hex_at(i), *d))
            .filter(|(h, _)| !unsafe_at(*h))
            .filter(|(h, _)| !unsafe_at(route(*h).last().copied().unwrap_or(scout.pos)))
            .collect();
        choices.sort_by_key(|(h, d)| {
            (
                !view.tile(*h).is_some_and(|t| t.owner == Some(view.me)),
                home(*h),
                *d,
                *h,
            )
        });
        let path = choices.first().map(|(h, _)| route(*h)).unwrap_or_default();
        return Some(automation_order(scout, path, stop));
    }
    let faction = view.players.iter().find(|p| p.id == view.me)?.faction;
    let ruin_bonus = crate::DecisionParams::builtin_for(faction).ruin_bonus;
    let target = ctx
        .frontier
        .iter()
        .copied()
        .filter(|h| *h != scout.pos && !unsafe_at(*h))
        .filter_map(|h| {
            let d = dist[view.index(h)?];
            (d != UNREACHABLE).then(|| (frontier_score(&ctx, h, d, ruin_bonus), h))
        })
        .min();
    Some(match target {
        Some((_, h)) => automation_order(scout, route(h), None),
        None => automation_order(scout, Vec::new(), Some(ExploreStopReason::Complete)),
    })
}

fn automation_order(
    unit: &UnitView,
    path: Vec<Hex>,
    stop_reason: Option<ExploreStopReason>,
) -> Order {
    Order::SetAutomation {
        unit: unit.id,
        automation: stop_reason.is_none().then_some(Automation::Explore),
        path,
        stop_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, MATE, ME, ViewBuilder, off};
    use legatum_core::{Terrain, UnitKind};

    fn plan(v: &PlayerView) -> (Vec<Hex>, Option<ExploreStopReason>) {
        match explore_step(v, UnitId(1)).unwrap() {
            Order::SetAutomation {
                path, stop_reason, ..
            } => (path, stop_reason),
            _ => panic!("automation order expected"),
        }
    }

    #[test]
    fn explore_prefers_visible_and_remembered_ruins_and_is_deterministic() {
        let mut v = ViewBuilder::new(16, 10)
            .unexplored_from_col(11)
            .unit(1, ME, UnitKind::Scout, off(7, 5))
            .ruin(off(6, 5))
            .build();
        for visible in [true, false] {
            let i = v.index(off(6, 5)).unwrap();
            v.tiles[i].visible = visible;
            assert_eq!(plan(&v).0.last(), Some(&off(6, 5)));
            for _ in 0..10 {
                assert_eq!(explore_step(&v, UnitId(1)), explore_step(&v, UnitId(1)));
            }
        }
    }

    #[test]
    fn military_units_explore_like_scouts_and_civilians_do_not() {
        let build = |kind| {
            ViewBuilder::new(20, 10)
                .unexplored_from_col(14)
                .unit(1, ME, kind, off(6, 5))
                .build()
        };
        for kind in [UnitKind::Spearman, UnitKind::Horseman, UnitKind::Archer] {
            let v = build(kind);
            let (path, stop) = plan(&v);
            assert!(stop.is_none(), "{kind:?}");
            assert_eq!(path.len(), v.units[0].max_moves as usize, "{kind:?}");
        }
        for kind in [UnitKind::Settler, UnitKind::HeirToTheThrone] {
            assert_eq!(explore_step(&build(kind), UnitId(1)), None, "{kind:?}");
        }
        // A damaged soldier still rests instead of walking, as a Scout does.
        let hurt = ViewBuilder::new(16, 10)
            .unexplored_from_col(13)
            .unit_hp(1, ME, UnitKind::Spearman, off(6, 5), 59)
            .build();
        assert_eq!(plan(&hurt).1, Some(ExploreStopReason::Damaged));
    }

    #[test]
    fn explore_heads_for_frontier_and_spends_full_movement_budget() {
        let v = ViewBuilder::new(20, 10)
            .unexplored_from_col(14)
            .unit(1, ME, UnitKind::Scout, off(6, 5))
            .build();
        let (path, stop) = plan(&v);
        assert!(stop.is_none());
        assert_eq!(path.len(), v.units[0].max_moves as usize);
        assert!(path.last().unwrap().to_offset().0 > 6);
    }

    #[test]
    fn explore_avoids_enemy_range_along_the_whole_route() {
        let mut v = ViewBuilder::new(20, 12)
            .unexplored_from_col(15)
            .unit(1, ME, UnitKind::Scout, off(5, 5))
            .unit(2, ENEMY, UnitKind::Archer, off(10, 5))
            .ruin(off(9, 5))
            .build();
        v.units[1].range = 3;
        let (path, stop) = plan(&v);
        assert!(stop.is_none());
        assert!(!path.is_empty());
        assert!(path.iter().all(|h| h.distance(off(10, 5)) > 3));
    }

    #[test]
    fn threatened_or_badly_damaged_scouts_stop_with_a_safe_retreat() {
        for hp in [100, 59] {
            let v = ViewBuilder::new(16, 10)
                .unexplored_from_col(13)
                .unit_hp(1, ME, UnitKind::Scout, off(6, 5), hp)
                .unit(2, ENEMY, UnitKind::Spearman, off(8, 5))
                .build();
            let (path, reason) = plan(&v);
            assert_eq!(
                reason,
                Some(if hp < 60 {
                    ExploreStopReason::Damaged
                } else {
                    ExploreStopReason::Threatened
                })
            );
            assert!(!path.is_empty());
            assert!(
                path.iter()
                    .all(|h| h.distance(off(8, 5)) > v.units[1].max_moves)
            );
        }
    }

    /// Barbarians never attack the Celts: a Celtic scout is not "threatened" by one and walks
    /// past it; every other empire's scout still retreats.
    #[test]
    fn a_celtic_scout_is_not_threatened_by_barbarians() {
        for (faction, stops) in [
            (legatum_core::Faction::Celts, false),
            (legatum_core::Faction::Switzerland, true),
        ] {
            let v = ViewBuilder::new(16, 10)
                .faction(faction)
                .unexplored_from_col(13)
                .unit(1, ME, UnitKind::Scout, off(6, 5))
                .unit(
                    2,
                    legatum_core::PlayerId::BARBARIAN,
                    UnitKind::Spearman,
                    off(8, 5),
                )
                .build();
            let (_, reason) = plan(&v);
            assert_eq!(
                reason == Some(ExploreStopReason::Threatened),
                stops,
                "{faction:?}"
            );
        }
    }

    #[test]
    fn damaged_scout_rests_safely_without_chasing_ruins() {
        let v = ViewBuilder::new(16, 10)
            .unexplored_from_col(13)
            .unit_hp(1, ME, UnitKind::Scout, off(6, 5), 80)
            .ruin(off(7, 5))
            .build();
        assert_eq!(plan(&v), (vec![], None));
    }

    #[test]
    fn retreat_can_leave_adjacent_threats_but_must_reach_safety_this_turn() {
        for hp in [100, 59] {
            let mut v = ViewBuilder::new(16, 10)
                .unit_hp(1, ME, UnitKind::Scout, off(6, 5), hp)
                .unit(2, ENEMY, UnitKind::Spearman, off(7, 5))
                .build();
            let (path, reason) = plan(&v);
            assert_eq!(
                reason,
                Some(if hp < 60 {
                    ExploreStopReason::Damaged
                } else {
                    ExploreStopReason::Threatened
                })
            );
            assert!(path.len() >= 2);
            assert!(path.last().unwrap().distance(off(7, 5)) > v.units[1].max_moves);
            let mut distance = v.units[0].pos.distance(off(7, 5));
            for h in path {
                assert!(h.distance(off(7, 5)) > distance);
                distance = h.distance(off(7, 5));
            }
            v.units[0].max_moves = 1;
            assert!(plan(&v).0.is_empty(), "cannot escape within one move");
        }
    }

    #[test]
    fn explore_stops_on_fully_explored_or_unreachable_frontier() {
        let v = ViewBuilder::new(16, 10)
            .unit(1, ME, UnitKind::Scout, off(5, 5))
            .build();
        assert_eq!(plan(&v), (vec![], Some(ExploreStopReason::Complete)));
        let mut b = ViewBuilder::new(16, 10).unexplored_from_col(13).unit(
            1,
            ME,
            UnitKind::Scout,
            off(5, 5),
        );
        for row in 0..10 {
            b = b.terrain(off(8, row), Terrain::Mountains);
        }
        assert_eq!(
            plan(&b.build()),
            (vec![], Some(ExploreStopReason::Complete))
        );
    }

    #[test]
    fn explore_never_commands_a_teammate() {
        let v = ViewBuilder::new(16, 10)
            .unexplored_from_col(13)
            .unit(1, MATE, UnitKind::Scout, off(5, 5))
            .build();
        assert_eq!(explore_step(&v, UnitId(1)), None);
    }
}

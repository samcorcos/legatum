//! Reserve this turn's movement in tactical order, using only the fogged view.
//! Replan after each yield: a newly parked friend changes the engine's routes.

use crate::scripted::Planner;
#[cfg(test)]
use legatum_core::Terrain;
use legatum_core::path::{border_step, find_path, hop, route_around_friends};
use legatum_core::rules::movement::can_step;
use legatum_core::state::Map;
use legatum_core::{Hex, Order, UnitId};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

impl Planner<'_> {
    /// Find the participant that would invalidate a proposed siege before committing
    /// the force. The caller removes it and forecasts the smaller force again.
    pub(crate) fn assault_traffic_conflict(
        &mut self,
        arrivals: &[(&legatum_core::view::UnitView, Hex)],
        target: Hex,
    ) -> Option<UnitId> {
        let mut movers = BTreeMap::new();
        let mut paths = BTreeMap::new();
        let mut priority = Vec::new();
        for order in &self.orders {
            match order {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => {
                    movers.insert(*unit, *to);
                    priority.push(*unit);
                    if let Order::MoveUnitPath { path, .. } = order {
                        paths.insert(*unit, path.clone());
                    }
                }
                _ => {}
            }
        }
        let attacks = self.assault_attacks.clone();
        let mut group = Vec::new();
        for &(unit, to) in arrivals {
            if to != unit.pos {
                movers.insert(unit.id, to);
                priority.push(unit.id);
                group.push(unit.id);
                if to == target {
                    self.assault_attacks.insert(unit.id);
                }
            }
        }
        priority.sort_by_key(|id| {
            let u = self
                .ctx
                .own_units
                .iter()
                .find(|u| u.id == *id)
                .expect("own mover");
            if self.retreats.contains(id) {
                (false, u.hp, Reverse(self.unit_exposure_key(u, u.pos).0))
            } else {
                (true, 0, Reverse(0))
            }
        });
        self.assault_groups.push(group);
        let conflict = self.movement_conflict(self.movement_map(), &movers, &paths, &priority);
        self.assault_groups.pop();
        self.assault_attacks = attacks;
        conflict
    }

    pub fn dedupe_first_steps(&mut self) {
        let mut movers: BTreeMap<UnitId, Hex> = self
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => {
                    Some((*unit, *to))
                }
                _ => None,
            })
            .collect();
        if movers.is_empty() {
            return;
        }
        let mut paths: BTreeMap<_, _> = self
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::MoveUnitPath { unit, path, .. } => Some((*unit, path.clone())),
                _ => None,
            })
            .collect();
        let mut priority: Vec<_> = self
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. } => {
                    self.ctx.own_units.iter().find(|u| u.id == *unit)
                }
                _ => None,
            })
            .copied()
            .collect();
        // Stable sorting retains the planner's phase order for equally urgent moves.
        // Unit IDs can break tactical ties, but cannot put a routine march before an escape.
        priority.sort_by_key(|u| {
            if self.retreats.contains(&u.id) {
                (false, u.hp, Reverse(self.unit_exposure_key(u, u.pos).0))
            } else {
                (true, 0, Reverse(0))
            }
        });
        let priority: Vec<_> = priority.into_iter().map(|u| u.id).collect();
        let view = self.view();
        let map = self.movement_map();
        let declared: BTreeSet<_> = self
            .orders
            .iter()
            .filter_map(|order| match order {
                Order::DeclareWar {
                    against,
                    breach: false,
                } => Some(*against),
                _ => None,
            })
            .collect();
        let mut tried: BTreeSet<_> = movers.iter().map(|(&id, &to)| (id, to)).collect();
        while let Some(id) = self.movement_conflict(map, &movers, &paths, &priority) {
            let u = self.ctx.own_units.iter().find(|u| u.id == id).unwrap();
            // A yield can close the chosen route. Try other known, safer arrivals before
            // parking an escape. Every attempt is new, and every other failure removes a
            // mover, so this reconciliation terminates even in an enclosed traffic jam.
            let mut alternatives: Vec<_> = if self.retreats.contains(&id) {
                u.pos
                    .within(u.max_moves.max(1))
                    .into_iter()
                    .filter(|h| {
                        !tried.contains(&(id, *h))
                            && self.ctx.reachable(u.pos, *h)
                            && self.unit_exposure_key(u, *h) < self.unit_exposure_key(u, u.pos)
                            && view
                                .index(*h)
                                .is_some_and(|i| !self.ctx.hostile_at[i] && !self.ctx.neutral_at[i])
                            && !view
                                .units
                                .iter()
                                .any(|other| other.pos == *h && other.military == u.military)
                    })
                    .collect()
            } else {
                Vec::new()
            };
            alternatives.sort_by_key(|h| self.recovery_key(u, *h));
            if let Some(&to) = alternatives.first() {
                tried.insert((id, to));
                if u.kind == legatum_core::UnitKind::Settler {
                    let route = find_path(map, u.pos, to, |h| {
                        map.move_cost(h).filter(|_| {
                            view.index(h).is_some_and(|i| {
                                !view.tile(h).is_some_and(|tile| {
                                    tile.owner.is_some_and(|owner| {
                                        view.at_war(u.owner, owner)
                                            || declared.contains(&view.team_of(owner))
                                    })
                                }) && !self.ctx.hostile_at[i]
                                    && !self.ctx.neutral_at[i]
                            })
                        })
                    });
                    if let Some(route) = route {
                        movers.insert(id, to);
                        paths.insert(id, route);
                    } else {
                        movers.remove(&id);
                        paths.remove(&id);
                    }
                } else {
                    movers.insert(id, to);
                    paths.remove(&id);
                }
            } else {
                movers.remove(&id);
            }
            for group in &self.assault_groups {
                if group.iter().any(|id| !movers.contains_key(id)) {
                    for id in group {
                        movers.remove(id);
                    }
                }
            }
        }
        for o in &mut self.orders {
            if let Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } = o {
                if let Some(&target) = movers.get(unit) {
                    *to = target;
                    if let Some(path) = paths.get(unit) {
                        match o {
                            Order::MoveUnitPath { path: current, .. } => *current = path.clone(),
                            Order::MoveUnit { unit, .. } => {
                                *o = Order::MoveUnitPath {
                                    unit: *unit,
                                    to: target,
                                    path: path.clone(),
                                    stop_on_block: true,
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        *o = Order::MoveUnit {
                            unit: *unit,
                            to: target,
                        };
                    }
                } else {
                    *o = Order::SkipUnit { unit: *unit };
                }
            }
        }
    }

    /// Predict friendly occupancy one movement round at a time. The engine repaths each
    /// round as units arrive or exhaust their budget; a single start-of-turn path is stale.
    /// Return the lower-priority mover in a conflict. Stationary occupants cannot yield.
    fn movement_conflict(
        &self,
        map: &Map,
        movers: &BTreeMap<UnitId, Hex>,
        paths: &BTreeMap<UnitId, Vec<Hex>>,
        priority: &[UnitId],
    ) -> Option<UnitId> {
        let view = self.view();
        let rank: BTreeMap<_, _> = priority
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();
        let friendly: BTreeMap<_, _> = view
            .units
            .iter()
            .filter(|u| {
                view.players
                    .iter()
                    .any(|p| p.id == u.owner && p.team == view.team)
                    || view.allied(view.me, u.owner)
            })
            .map(|u| (u.id, u))
            .collect();
        let mut positions: BTreeMap<_, _> = friendly.iter().map(|(&id, u)| (id, u.pos)).collect();
        let mut budgets: BTreeMap<_, _> =
            friendly.iter().map(|(&id, u)| (id, u.max_moves)).collect();
        let mut hostile_any: BTreeSet<_> = self.ctx.enemy_units.iter().map(|u| u.pos).collect();
        let mut hostile_military: BTreeSet<_> = self
            .ctx
            .enemy_units
            .iter()
            .filter(|u| u.military)
            .map(|u| u.pos)
            .collect();
        let hostile_city: BTreeSet<_> = self.ctx.enemy_cities.iter().map(|c| c.pos).collect();
        let declared: BTreeSet<legatum_core::TeamId> = self
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::DeclareWar {
                    against,
                    breach: false,
                } => Some(*against),
                _ => None,
            })
            .collect();
        // A surprise war declared in this submission makes that team's units hostile before
        // movement (G20): a snipe steps onto its Settler.
        let declared_at: BTreeSet<Hex> = view
            .units
            .iter()
            .filter(|u| declared.contains(&view.team_of(u.owner)))
            .map(|u| u.pos)
            .collect();
        for u in view
            .units
            .iter()
            .filter(|u| declared.contains(&view.team_of(u.owner)))
        {
            hostile_any.insert(u.pos);
            if u.military {
                hostile_military.insert(u.pos);
            }
        }
        // Bitmaps (by map index) for the sets the per-step route costs read: same answers,
        // without a tree lookup per edge.
        let bits = |set: &BTreeSet<Hex>| -> Vec<bool> {
            let mut b = vec![false; map.len()];
            for h in set {
                if let Some(i) = map.index(*h) {
                    b[i] = true;
                }
            }
            b
        };
        let on = |b: &[bool], h: Hex| map.index(h).is_some_and(|i| b[i]);
        let hostile_any_at = bits(&hostile_any);
        let hostile_military_at = bits(&hostile_military);
        let hostile_city_at = bits(&hostile_city);
        let declared_at_bits = bits(&declared_at);
        let mut open_by_owner: BTreeMap<
            (legatum_core::PlayerId, legatum_core::UnitKind),
            Vec<bool>,
        > = BTreeMap::new();
        let rounds = friendly.values().map(|u| u.max_moves).max().unwrap_or(0);
        let mut attacked = BTreeSet::new();
        // A round in which a mover waited its turn (below) adds a round: the waiter still has
        // its moves, and the resolver goes on while anyone advances.
        let mut waits = 0;
        for round in 0..rounds * 2 {
            if round >= rounds + waits {
                break;
            }
            let active: BTreeSet<_> = movers
                .iter()
                .filter(|(id, to)| positions[id] != **to && budgets[id] > 0)
                .map(|(&id, _)| id)
                .collect();
            if active.is_empty() {
                break;
            }
            let parked: BTreeSet<_> = friendly
                .iter()
                .filter(|(id, _)| !active.contains(id))
                .map(|(id, u)| (u.military, positions[id]))
                .collect();
            // The resolver's `attacking_in_place`: a mover aimed at an adjacent enemy stays.
            let staying: BTreeSet<_> = active
                .iter()
                .filter(|id| {
                    let (pos, goto) = (positions[*id], movers[*id]);
                    pos.distance(goto) == 1
                        && (hostile_city.contains(&goto)
                            || hostile_military.contains(&goto)
                            || (!friendly[*id].military && hostile_any.contains(&goto)))
                })
                .map(|id| (friendly[id].military, positions[id]))
                .collect();
            let leaving: BTreeSet<_> = active
                .iter()
                .map(|id| (friendly[id].military, positions[id]))
                .filter(|k| !staying.contains(k))
                .collect();
            // Parked friends that still have a destination and clear on a later turn: out of
            // moves for now (`path::route_around_friends`: waited for, not circled).
            let waiting: BTreeSet<_> = friendly
                .iter()
                .filter(|(id, _)| {
                    !active.contains(id)
                        && movers.get(id).is_some_and(|goto| *goto != positions[id])
                })
                .map(|(id, u)| (u.military, positions[id]))
                .collect();
            let split = |set: &BTreeSet<(bool, Hex)>| -> [Vec<bool>; 2] {
                let mut b = [vec![false; map.len()], vec![false; map.len()]];
                for (military, h) in set {
                    if let Some(i) = map.index(*h) {
                        b[usize::from(*military)][i] = true;
                    }
                }
                b
            };
            let (parked_at, staying_at, leaving_at, waiting_at) = (
                split(&parked),
                split(&staying),
                split(&leaving),
                split(&waiting),
            );
            let mut next = positions.clone();
            let budgets_before = budgets.clone();
            let mut reservations: BTreeMap<(legatum_core::PlayerId, bool), BTreeMap<Hex, bool>> =
                BTreeMap::new();
            // The resolver declares claims in unit-id order. Tactical priority only decides
            // which order to drop if a conflict cannot be reconciled.
            for &id in &active {
                let u = friendly[&id];
                let goto = movers[&id];
                let m = usize::from(u.military);
                // Answers for the goal too: a crossing cannot land on a hostile goal. The
                // route searches exempt the goal themselves.
                let avoid = |h: Hex| {
                    on(&hostile_city_at, h)
                        || on(&hostile_military_at, h)
                        || (!u.military && on(&hostile_any_at, h))
                        || (positions[&id].distance(h) == 1
                            && reservations
                                .get(&(u.owner, u.military))
                                .and_then(|steps| steps.get(&h))
                                == Some(&false))
                };
                // Friends that stay this round are crossed, never ended on (`path::hop`).
                let passable = |h: Hex| on(&parked_at[m], h) || on(&staying_at[m], h);
                let waiting = |h: Hex| on(&waiting_at[m], h);
                let risky = |h: Hex| h != positions[&id] && on(&leaving_at[m], h);
                let reserved = |h: Hex| {
                    positions[&id].distance(h) == 1
                        && reservations
                            .get(&(u.owner, u.military))
                            .and_then(|steps| steps.get(&h))
                            == Some(&true)
                };
                // Every simulated round obeys the same closed-border egress rule as
                // the real mover. Occupancy fallback cannot enter a neutral unit's tile.
                // A surprise war declared in this submission opens its borders before
                // movement resolves.
                let open_at = open_by_owner.entry((u.owner, u.kind)).or_insert_with(|| {
                    view.tiles
                        .iter()
                        .map(|t| {
                            view.passage_allowed(u.owner, u.kind, t.owner)
                                || t.owner.is_some_and(|o| declared.contains(&view.team_of(o)))
                        })
                        .collect()
                });
                let open_at: &[bool] = open_at;
                let open = |h| view.index(h).is_some_and(|i| open_at[i]);
                let exits = (!open(positions[&id])).then(|| view.exit_distances(map, open));
                let cost = |a, b| {
                    map.move_cost(b).filter(|_| {
                        border_step(map, a, b, open, exits.as_deref())
                            && (u.kind != legatum_core::UnitKind::Settler
                                || !view.tile(b).is_some_and(|tile| {
                                    tile.owner.is_some_and(|owner| {
                                        view.at_war(u.owner, owner)
                                            || declared.contains(&view.team_of(owner))
                                    })
                                }))
                            && (!self.ctx.neutral_at[view.index(b).expect("on map")]
                                || on(&declared_at_bits, b))
                    })
                };
                let route = if let Some(path) = paths.get(&id) {
                    // Confirmed routes do not repath around parked units. Match the
                    // resolver's cursor, including a mover that waited a round.
                    let pos = positions[&id];
                    let cursor = path.iter().position(|h| *h == pos).map_or(0, |i| i + 1);
                    let rest = &path[cursor.min(path.len())..];
                    let step = rest
                        .first()
                        .copied()
                        .filter(|h| pos.distance(*h) == 1 && cost(pos, *h).is_some());
                    // A confirmed step into a friend that stays crosses it when the rest of
                    // the route lands (the resolver's `next_step`).
                    step.map(|_| {
                        if rest.len() > 1 && passable(rest[0]) && rest[0] != goto {
                            rest.to_vec()
                        } else {
                            vec![rest[0]]
                        }
                    })
                } else {
                    // The resolver's costing (`rules::movement::next_step`).
                    route_around_friends(
                        map,
                        positions[&id],
                        goto,
                        cost,
                        avoid,
                        passable,
                        waiting,
                        risky,
                        (budgets[&id], u.max_moves),
                        reserved,
                    )
                };
                let Some(route) = route else {
                    return Some(id);
                };
                let mut step = route[0];
                let mut cost = map.move_cost(step).expect("path is passable");
                // The resolver's hop across friends that stay: it lands beyond them when it
                // can pay for the crossing, waits a turn when it cannot, and otherwise steps
                // into the friend (a conflict below).
                if step != goto
                    && route.len() > 1
                    && let Some((landing, total)) =
                        hop(&route, positions[&id], |_, h| map.move_cost(h), passable)
                {
                    let foe = hostile_any.contains(&landing)
                        || hostile_city.contains(&landing)
                        || self.ctx.neutral_at[view.index(landing).expect("on map")];
                    if !foe && total <= budgets[&id] {
                        step = landing;
                        cost = total;
                    } else if !foe && total <= u.max_moves {
                        budgets.insert(id, 0);
                        continue;
                    }
                }
                if !can_step(budgets[&id], u.max_moves, cost) {
                    budgets.insert(id, 0);
                    continue;
                }
                let attacks_from_next = step.distance(goto) == 1
                    && (hostile_city.contains(&goto)
                        || hostile_military.contains(&goto)
                        || !u.military && hostile_any.contains(&goto));
                reservations
                    .entry((u.owner, u.military))
                    .or_default()
                    .entry(step)
                    .or_insert(step != goto && budgets_before[&id] > cost && !attacks_from_next);
                if hostile_any.contains(&step) || hostile_city.contains(&step) {
                    // Do not invent combat outcomes. A favorable attack may be coordinated
                    // with other attackers, but followers cannot rely on the attacker leaving.
                    if !u.military
                        || ((hostile_military.contains(&step) || hostile_city.contains(&step))
                            && self.too_risky(u, step)
                            && !self.assault_attacks.contains(&id))
                    {
                        return Some(id);
                    }
                    budgets.insert(id, 0);
                    if step == goto {
                        attacked.insert(id);
                    }
                    continue;
                }
                next.insert(id, step);
                budgets.insert(id, (budgets[&id] - cost).max(0));
            }
            // Holds, exhausted movers and combatants occupy their tile before any incoming
            // claims. Friendly chains and swaps remain possible when every link can leave.
            // Two movers that claim one tile in the same round take turns the way the resolver
            // has them (`rules::movement`, step 5: claimants that share an owner advance by
            // unit id): the lower id advances and the other waits this round with its moves
            // intact. A mover that waits holds its tile, so a mover entering that tile waits
            // too. Neither is a conflict: the resolver does the same and both still arrive (gene
            // `movement_wait`; at 0 the lower-priority mover loses its whole turn).
            let mut waiting_now: BTreeSet<UnitId> = BTreeSet::new();
            loop {
                let mut claimed: BTreeMap<_, _> = friendly
                    .iter()
                    .filter(|(id, _)| positions[id] == next[id])
                    .map(|(&id, u)| ((u.military, next[&id]), id))
                    .collect();
                let mut clash = None;
                for &id in priority {
                    if positions[&id] != next[&id]
                        && let Some(other) = claimed.insert((friendly[&id].military, next[&id]), id)
                    {
                        clash = Some((id, other));
                        break;
                    }
                }
                let Some((id, other)) = clash else {
                    break;
                };
                let take_turns = self.decisions.movement_wait > 0;
                let loser = if take_turns && positions[&other] != next[&other] {
                    // Both arrive this round.
                    id.max(other)
                } else if take_turns && waiting_now.contains(&other) {
                    // The occupant is a mover that waits this round: the chain behind it waits.
                    id
                } else {
                    // A lower-priority mover that arrived earlier can still yield its
                    // whole turn. An originally parked occupant cannot be displaced.
                    return Some(if movers.contains_key(&other) && rank[&other] > rank[&id] {
                        other
                    } else {
                        id
                    });
                };
                next.insert(loser, positions[&loser]);
                budgets.insert(loser, budgets_before[&loser]);
                waiting_now.insert(loser);
            }
            if !waiting_now.is_empty() {
                waits += 1;
            }
            positions = next;
        }
        // A newly parked unit can lengthen an escape without directly colliding. Check the
        // actual arrival this turn, not just the safety of the order's distant destination.
        priority.iter().copied().find(|id| {
            movers.contains_key(id)
                && ((self.retreats.contains(id)
                    && self.unit_exposure_key(friendly[id], positions[id])
                        >= self.unit_exposure_key(friendly[id], friendly[id].pos))
                    || (self.assault_groups.iter().any(|group| group.contains(id))
                        && positions[id] != movers[id]
                        && !attacked.contains(id)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::{Faction, PlayerView, UnitKind};

    fn corridor() -> PlayerView {
        let mut builder = ViewBuilder::new(12, 7).unit(1, ME, UnitKind::Soldier, off(3, 3));
        for row in 0..7 {
            for col in 0..12 {
                builder = builder.terrain(
                    off(col, row),
                    if row == 3 && (3..=8).contains(&col) {
                        Terrain::Grassland
                    } else {
                        Terrain::Mountains
                    },
                );
            }
        }
        let mut view = builder.build();
        view.diplomacy.legacy = false; // Missing foreign relations are Peace in this view.
        view
    }

    #[test]
    fn preflight_accepts_the_same_reserved_pass_as_core_and_preview() {
        let mut builder = ViewBuilder::new(12, 10)
            .unit(1, ME, UnitKind::Soldier, off(4, 3))
            .unit(2, ME, UnitKind::Soldier, off(4, 4));
        for row in 0..10 {
            for col in 0..12 {
                let open = (1..=10).contains(&col) && (row == 4 || row == 8)
                    || (5..=7).contains(&row) && (col == 1 || col == 10)
                    || (col, row) == (4, 3);
                builder = builder.terrain(
                    off(col, row),
                    if open {
                        Terrain::Grassland
                    } else {
                        Terrain::Mountains
                    },
                );
            }
        }
        let mut view = builder.build();
        view.diplomacy.legacy = false;
        let planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            1,
        );
        let movers = BTreeMap::from([(UnitId(1), off(9, 4)), (UnitId(2), off(8, 4))]);
        assert_eq!(
            planner.movement_conflict(
                planner.movement_map(),
                &movers,
                &BTreeMap::new(),
                &[UnitId(1), UnitId(2)]
            ),
            None
        );
    }

    fn reserve(view: &PlayerView) -> Vec<Order> {
        reserve_to(view, off(7, 3))
    }

    fn reserve_to(view: &PlayerView, to: Hex) -> Vec<Order> {
        let mut planner = Planner::new(
            view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            1,
        );
        planner.orders.push(Order::MoveUnit {
            unit: UnitId(1),
            to,
        });
        planner.dedupe_first_steps();
        planner.orders
    }

    #[test]
    fn settler_retreat_reconciliation_keeps_the_alternative_out_of_enemy_territory() {
        let start = Hex::new(7, 8);
        let mut view = ViewBuilder::new(26, 18)
            .city(9, ENEMY, Hex::new(10, 8), 3)
            .unit(1, ME, UnitKind::Settler, start)
            .unit(50, ENEMY, UnitKind::Scout, Hex::new(5, 8))
            .build();
        for i in 0..view.tiles.len() {
            let h = Hex::from_offset(i as i32 % view.width, i as i32 / view.width);
            view.tiles[i].terrain = Some(if h.r == 8 && (5..=10).contains(&h.q) {
                Terrain::Grassland
            } else {
                Terrain::Mountains
            });
        }
        for blocked_route in [false, true] {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            p.plan_settlers();
            assert_eq!(p.orders, [Order::SkipUnit { unit: UnitId(1) }]);
            assert!(p.retreats.contains(&UnitId(1)));
            if blocked_route {
                // A stale confirmed route cannot replace the settler's safe hold decision.
                p.orders[0] = Order::MoveUnitPath {
                    unit: UnitId(1),
                    to: Hex::new(7, 7),
                    path: vec![Hex::new(7, 7)],
                    stop_on_block: false,
                };
            }
            p.dedupe_first_steps();
            assert_eq!(p.orders, [Order::SkipUnit { unit: UnitId(1) }]);
        }
    }

    #[test]
    fn reservations_keep_closed_border_egress_but_cannot_cross_a_peaceful_border() {
        let mut view = corridor();
        let movement = reserve(&view);
        assert!(matches!(movement[0], Order::MoveUnit { .. }));
        let blocked = view.index(off(5, 3)).unwrap();
        view.tiles[blocked].owner = Some(ENEMY);
        assert_eq!(reserve(&view), [Order::SkipUnit { unit: UnitId(1) }]);
        for col in 3..5 {
            let i = view.index(off(col, 3)).unwrap();
            view.tiles[i].owner = Some(ENEMY);
        }
        assert_eq!(reserve(&view), movement); // Each simulated round approaches the exit.
    }

    #[test]
    fn reservations_do_not_route_a_military_unit_through_a_peaceful_civilian() {
        let mut view = corridor();
        assert!(matches!(reserve(&view)[0], Order::MoveUnit { .. }));
        let foreign = ViewBuilder::new(12, 7)
            .unit(2, ENEMY, UnitKind::Settler, off(5, 3))
            .build();
        view.units.extend(foreign.units);
        assert_eq!(reserve(&view), [Order::SkipUnit { unit: UnitId(1) }]);
    }

    #[test]
    fn confirmed_settler_routes_cross_a_parked_civilian_but_cannot_end_on_it() {
        let view = ViewBuilder::new(12, 10)
            .unit(1, ME, UnitKind::Settler, off(3, 4))
            .unit(2, ME, UnitKind::Settler, off(5, 4))
            .build();
        let plan = |to: Hex, path: Vec<Hex>| {
            let mut planner = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            let first = Order::MoveUnitPath {
                unit: UnitId(1),
                to,
                path,
                stop_on_block: false,
            };
            planner.orders = vec![first.clone(), Order::SkipUnit { unit: UnitId(2) }];
            planner.dedupe_first_steps();
            (first, planner.orders)
        };
        // The route passes the stationary settler and lands beyond it: kept as submitted.
        let (first, orders) = plan(off(7, 4), (4..=7).map(|col| off(col, 4)).collect());
        assert_eq!(orders, vec![first, Order::SkipUnit { unit: UnitId(2) }]);
        // A route that would end its move on the settler is a conflict: the mover is parked.
        let (_, orders) = plan(off(5, 4), (4..=5).map(|col| off(col, 4)).collect());
        assert_eq!(
            orders,
            vec![
                Order::SkipUnit { unit: UnitId(1) },
                Order::SkipUnit { unit: UnitId(2) },
            ]
        );
    }

    #[test]
    fn a_move_through_a_parked_friend_is_kept_and_one_that_cannot_land_is_not() {
        // A one-tile corridor with a parked soldier in it: the mover crosses it and lands
        // beyond (the resolver's hop). Two parked soldiers in a row cost more to cross than
        // a Soldier has moves, and a parked soldier on the goal cannot be ended on: both
        // are conflicts, as they are when the resolver steps into the friend.
        let corridor = |parked: &[i32]| {
            let mut builder = ViewBuilder::new(12, 7).unit(1, ME, UnitKind::Soldier, off(3, 3));
            for (i, col) in parked.iter().enumerate() {
                builder = builder.unit(10 + i as u32, ME, UnitKind::Soldier, off(*col, 3));
            }
            for row in 0..7 {
                for col in 0..12 {
                    builder = builder.terrain(
                        off(col, row),
                        if row == 3 && (2..=9).contains(&col) {
                            Terrain::Grassland
                        } else {
                            Terrain::Mountains
                        },
                    );
                }
            }
            let mut view = builder.build();
            view.diplomacy.legacy = false;
            view
        };
        let kept = |view: &PlayerView, to: i32| reserve_to(view, off(to, 3));
        let moved = |orders: &[Order]| matches!(orders[0], Order::MoveUnit { .. });
        assert!(moved(&kept(&corridor(&[4]), 8)));
        assert!(!moved(&kept(&corridor(&[4, 5]), 8)));
        assert!(!moved(&kept(&corridor(&[5]), 5)));
    }

    #[test]
    fn confirmed_settler_routes_reserve_later_round_arrivals() {
        let view = ViewBuilder::new(12, 10)
            .unit(1, ME, UnitKind::Settler, off(3, 4))
            .unit(2, ME, UnitKind::Settler, off(6, 4))
            .build();
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            1,
        );
        let first = Order::MoveUnitPath {
            unit: UnitId(1),
            to: off(5, 4),
            path: vec![off(4, 4), off(5, 4)],
            stop_on_block: false,
        };
        planner.orders = vec![
            first.clone(),
            Order::MoveUnit {
                unit: UnitId(2),
                to: off(5, 4),
            },
        ];
        planner.dedupe_first_steps();
        assert_eq!(
            planner.orders,
            vec![first, Order::SkipUnit { unit: UnitId(2) }]
        );
    }
    #[test]
    fn movers_that_claim_one_tile_in_one_round_take_turns_instead_of_losing_the_turn() {
        // Two soldiers cross the same tile on their first step: unit 1 moves east through it,
        // unit 2 south. The resolver gives the tile to the lower id and the other waits a round
        // with its moves, then both arrive (`rules::movement` step 5), so neither order is
        // cancelled, whichever the planner ranked first.
        let view = ViewBuilder::new(12, 8)
            .unit(1, ME, UnitKind::Soldier, Hex::new(4, 5))
            .unit(2, ME, UnitKind::Soldier, Hex::new(5, 4))
            .build();
        let east = Order::MoveUnit {
            unit: UnitId(1),
            to: Hex::new(6, 5),
        };
        let south = Order::MoveUnit {
            unit: UnitId(2),
            to: Hex::new(5, 6),
        };
        for orders in [
            vec![east.clone(), south.clone()],
            vec![south.clone(), east.clone()],
        ] {
            let mut planner = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            planner.orders = orders.clone();
            planner.dedupe_first_steps();
            assert_eq!(planner.orders, orders);
            // Gene at 0: the lower-priority mover loses its turn.
            let mut old = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            old.orders = orders.clone();
            old.decisions.movement_wait = 0;
            old.dedupe_first_steps();
            assert!(
                old.orders
                    .iter()
                    .any(|o| matches!(o, Order::SkipUnit { .. }))
            );
        }
    }
}

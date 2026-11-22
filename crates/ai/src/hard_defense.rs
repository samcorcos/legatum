//! Readiness from visible armies, including neighbours that can legally declare war.
//! Kept out of Ctx's shared constructor so versioned benchmark policies stay frozen.
use crate::{analysis::threat_power, scripted::Planner};
use legatum_core::rules::movement::can_step;
use legatum_core::{
    AiDifficulty, AlongsideAttack, AttackMode, Order, UnitId, UnitKind,
    view::{CityView, UnitView},
};
#[cfg(test)]
use std::collections::BTreeSet;
use std::collections::{BTreeMap, VecDeque};

/// Earliest arrival after a possible declaration, through known terrain and legal
/// third-party borders. A fresh turn refills movement; the engine also permits one
/// expensive step when a unit still has its full allowance.
pub(crate) fn declaration_arrival_turns(
    v: &legatum_core::PlayerView,
    u: &UnitView,
    to: legatum_core::Hex,
    horizon: u32,
) -> Option<u32> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    if u.pos.distance(to) > u.max_moves.max(1) * horizon as i32 {
        return None;
    }
    let entry = v.entry_table(u.owner, u.kind);
    let mut best = vec![(u32::MAX, -1); v.tiles.len()];
    let start = v.index(u.pos)?;
    best[start] = (1, u.max_moves);
    let mut queue = BinaryHeap::from([Reverse((1u32, -u.max_moves, u.pos))]);
    while let Some(Reverse((turn, neg_moves, pos))) = queue.pop() {
        let moves = -neg_moves;
        let Some(index) = v.index(pos) else { continue };
        if best[index] != (turn, moves) {
            continue;
        }
        if pos == to {
            return Some(turn);
        }
        for next in pos.neighbors() {
            let Some(i) = v.index(next) else { continue };
            let tile = &v.tiles[i];
            let cost = if tile.terrain.is_none() {
                Some(1)
            } else {
                tile.move_cost()
            };
            let Some(cost) = cost else { continue };
            if !tile.owner.is_some_and(|owner| v.team_of(owner) == v.team)
                && !entry.can_enter(v, tile.owner)
            {
                continue;
            }
            let next_turn = if can_step(moves, u.max_moves, cost) {
                turn
            } else {
                turn + 1
            };
            if next_turn > horizon {
                continue;
            }
            let allowance = if next_turn == turn {
                moves
            } else {
                u.max_moves
            };
            let left = (allowance - cost).max(0);
            if (next_turn, -left) < (best[i].0, -best[i].1) {
                best[i] = (next_turn, left);
                queue.push(Reverse((next_turn, -left, next)));
            }
        }
    }
    None
}

// Ordinary Peace does not let our pathfinder cross the rival's border. Readiness
// instead asks where this visible unit could walk after a legal declaration,
// retaining known terrain and every third party's current border permissions.
#[cfg(test)]
fn declaration_routes(v: &legatum_core::PlayerView, u: &UnitView) -> BTreeSet<legatum_core::Hex> {
    let seen = declaration_reach(v, u, &v.entry_table(u.owner, u.kind));
    (0..seen.len())
        .filter(|i| seen[*i])
        .map(|i| {
            let w = v.width as usize;
            legatum_core::Hex::from_offset((i % w) as i32, (i / w) as i32)
        })
        .collect()
}

/// Tiles (by map index) a rival unit could step through within five moves if it declared war
/// now, using its owner's entry rule (`entry`).
fn declaration_reach(
    v: &legatum_core::PlayerView,
    u: &UnitView,
    entry: &legatum_core::view::EntryTable,
) -> Vec<bool> {
    let mine = v.team_of(v.me);
    let mut seen = vec![false; v.tiles.len()];
    if let Some(i) = v.index(u.pos) {
        seen[i] = true;
    }
    let mut queue = VecDeque::from([(u.pos, 0)]);
    while let Some((pos, steps)) = queue.pop_front() {
        if steps >= 5 {
            continue;
        }
        for h in pos.neighbors() {
            // Off-map neighbours never pass the tile test below, so only on-map ones are marked.
            let Some(i) = v.index(h) else { continue };
            if seen[i] {
                continue;
            }
            let t = &v.tiles[i];
            if t.terrain.and_then(|terrain| terrain.move_cost()).is_some()
                && (t.owner.is_some_and(|owner| v.team_of(owner) == mine)
                    || entry.can_enter(v, t.owner))
            {
                seen[i] = true;
                queue.push_back((h, steps + 1));
            }
        }
    }
    seen
}

/// Gene `capital_guard`: Settlers this near an empty capital in a rival army's reach go home.
const CAPITAL_RECALL: i32 = 4;
/// Gene `escort_buy`: how near a Settler (or the city training one) a rival army must stand.
const SECOND_TROOP_RADIUS: i32 = 6;
/// Gene `escort_buy`: routine purchases wait for the second troop only this many turns of income.
const SECOND_TROOP_SAVE_TURNS: i32 = 5;

impl Planner<'_> {
    fn defender_soak(
        &self,
        u: &UnitView,
        attackers: &[&UnitView],
        terrain: legatum_core::Terrain,
    ) -> i64 {
        let worst = attackers
            .iter()
            .map(|e| {
                let attack = crate::supply::attack_total(self.view().faction_of(e.owner), e, e.hp);
                let defense = crate::supply::defense_total(self.faction, u, u.hp, terrain, 0);
                let (bonus, against) = crate::tactics::fight_bonuses(e.kind, u.kind, false);
                legatum_core::combat::damage(attack + bonus, defense + against, 100)
            })
            .max()
            .unwrap_or(1);
        i64::from(u.hp) * 1000 / i64::from(worst.max(1))
    }

    /// Bring a garrison and reserves to the threatened city before the enemy can hit it.
    /// No forecast assumes a peaceful rival's unsubmitted orders; this only stages visible
    /// defenders against a legal declaration or an army already at war.
    fn stage_supported_defence(&mut self, city: &CityView, attackers: &[&UnitView]) {
        let terrain = self
            .view()
            .tile(city.pos)
            .and_then(|t| t.terrain)
            .unwrap_or(legatum_core::Terrain::Plains);
        let garrison = self
            .ctx
            .own_units
            .iter()
            .copied()
            .find(|u| u.military && u.pos == city.pos);
        let mut assigned = garrison.map(|u| u.id);
        if let Some(u) = garrison
            && self.orders.iter().any(|o| {
                legatum_core::rules::units::order_unit(o) == Some(u.id)
                    && matches!(o, Order::MoveUnit { .. } | Order::MoveUnitPath { .. })
            })
        {
            let order = (u.ranged > 0)
                .then(|| self.best_shot_automatic(u, &BTreeMap::new()))
                .flatten()
                .map(|target| Order::RangedAttack { unit: u.id, target })
                .unwrap_or(Order::SkipUnit { unit: u.id });
            self.replace_supported_order(u.id, order);
        }
        if assigned.is_none() {
            let candidate = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| u.military && !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
                // Gene `roman_found_first`: a stand staged against a neighbour's visible
                // units does not recall the veteran founding a city this turn (or walking
                // to its site); the site's own danger gates already priced them.
                .filter(|u| Some(u.id) != self.roman_founder && Some(u.id) != self.colonia_founder)
                .filter(|u| self.support_path_this_turn(u, city.pos).is_some())
                .max_by_key(|u| {
                    (
                        u.ranged == 0,
                        self.defender_soak(u, attackers, terrain),
                        std::cmp::Reverse(u.id),
                    )
                });
            if let Some(u) = candidate {
                self.replace_supported_order(
                    u.id,
                    Order::MoveUnit {
                        unit: u.id,
                        to: city.pos,
                    },
                );
                assigned = Some(u.id);
            }
        }
        let mut staging = Vec::new();
        for _ in 0..2 {
            let choice = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| u.military && u.ranged == 0 && u.pos != city.pos)
                .filter(|u| Some(u.id) != assigned && !staging.contains(&u.id))
                .filter(|u| Some(u.id) != self.roman_founder && Some(u.id) != self.colonia_founder)
                .filter(|u| !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
                .filter(|u| u.pos.distance(city.pos) > 1 && u.pos.distance(city.pos) <= 5)
                .filter_map(|u| {
                    let dest = u
                        .pos
                        .within(u.max_moves.max(1))
                        .into_iter()
                        .filter(|h| h.distance(city.pos) < u.pos.distance(city.pos))
                        .filter(|h| h.distance(city.pos) >= 1)
                        .filter(|h| {
                            self.view()
                                .index(*h)
                                .is_some_and(|i| !self.ctx.occupied[i] && !self.ctx.hostile_at[i])
                        })
                        .filter(|h| self.support_path_this_turn(u, *h).is_some())
                        .min_by_key(|h| (h.distance(city.pos), self.exposure_at(*h), *h))?;
                    Some((u, dest))
                })
                .max_by_key(|(u, h)| {
                    (
                        u.pos.distance(city.pos) - h.distance(city.pos),
                        self.defender_soak(u, attackers, terrain),
                        std::cmp::Reverse(u.id),
                    )
                });
            let Some((u, dest)) = choice else { break };
            staging.push(u.id);
            self.replace_supported_order(
                u.id,
                Order::MoveUnit {
                    unit: u.id,
                    to: dest,
                },
            );
        }
    }

    /// Forecast the visible melee assault from our fogged view, with the opponent temporarily
    /// selected as the forecasting side. This changes no information in the view: the only
    /// units present are still the ones Hard can see. A distant reserve is placed at its last
    /// step before the city so the engine's adjacent-support forecast can price its arrival.
    fn supported_forecast(
        &self,
        city: &CityView,
        attackers: &[&UnitView],
        supporters: &[UnitId],
        swap: Option<(UnitId, UnitId)>,
    ) -> Option<legatum_core::CityAssaultForecast> {
        let first = *attackers.first()?;
        let mut view = self.view().clone();
        view.me = first.owner;
        view.team = view.team_of(first.owner);
        if let Some((old, new)) = swap {
            let from = view.units.iter().find(|u| u.id == new)?.pos;
            view.units.iter_mut().find(|u| u.id == old)?.pos = from;
            view.units.iter_mut().find(|u| u.id == new)?.pos = city.pos;
        }
        for id in supporters {
            let unit = self.ctx.own_units.iter().copied().find(|u| u.id == *id)?;
            if unit.pos.distance(city.pos) > 1 {
                let path = self.support_path_this_turn(unit, city.pos)?;
                let approach = *path.get(path.len().checked_sub(2)?)?;
                view.units.iter_mut().find(|u| u.id == *id)?.pos = approach;
            }
        }
        let alongside: Vec<_> = attackers
            .iter()
            .skip(1)
            .filter(|e| e.owner == first.owner)
            .map(|e| AlongsideAttack {
                unit: e.id,
                mode: AttackMode::Melee,
            })
            .collect();
        legatum_core::forecast::forecast_city_assault_supported(
            &view,
            first.id,
            city.pos,
            AttackMode::Melee,
            &alongside,
            supporters,
        )
    }

    fn supported_score(f: &legatum_core::CityAssaultForecast) -> i64 {
        i64::from(100 - f.capture_chance) * 100
            + i64::from(f.city_hp_after) * 2
            + i64::from(f.garrison.as_ref().map_or(0, |g| g.hp_after))
            + f.supporters
                .iter()
                .map(|s| i64::from(s.hp_after))
                .sum::<i64>()
            + i64::from(f.reply) * 3
    }

    /// A threatened city's current occupant stays unless a stronger adjacent defender can
    /// exchange places. Add up to two supporters only when the engine's shared-damage and pooled
    /// reply forecast gains more than the unit loses by leaving its field position.
    pub(crate) fn supported_defence_orders(&mut self) {
        if !self.hard_defense() || self.decisions.supported_defence == 0 {
            return;
        }
        for city in self.ctx.own_cities.clone() {
            let imminent: Vec<_> = self
                .view()
                .units
                .iter()
                .filter(|e| {
                    e.military
                        && e.ranged == 0
                        && !e.owner.is_barbarian()
                        && !self.view().same_team(self.view().me, e.owner)
                        && (self.view().at_war(self.view().me, e.owner)
                            || self.view().diplomacy.actions.iter().any(|a| {
                                a.order
                                    == Order::DeclareWar {
                                        against: self.view().team_of(e.owner),
                                        breach: false,
                                    }
                                    && a.blocked_reason.is_none()
                            }))
                        && declaration_arrival_turns(self.view(), e, city.pos, 2).is_some()
                })
                .collect();
            if imminent.is_empty() {
                continue;
            }
            let mut attackers: Vec<_> = self
                .ctx
                .threat_units
                .iter()
                .copied()
                .filter(|e| {
                    e.military
                        && e.ranged == 0
                        && self.view().at_war(self.view().me, e.owner)
                        && self.threat_reaches_this_turn(e, city.pos)
                })
                .collect();
            if attackers.is_empty() {
                if imminent
                    .iter()
                    .all(|e| !self.view().at_war(self.view().me, e.owner))
                    && self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|u| u.military && u.ranged == 0 && u.kind != UnitKind::Scout)
                        .take(2)
                        .count()
                        >= 2
                    // A lone Scout near the last city cannot recall a routed operation.
                    && !(matches!(
                        self.routed_plan().map(|p| &p.kind),
                        Some(
                            crate::plan::PlanKind::Campaign { .. }
                                | crate::plan::PlanKind::Contain { .. }
                        )
                    ) && (self.ctx.own_cities.len() > 1
                        || imminent.iter().all(|e| e.kind == UnitKind::Scout)))
                {
                    self.stage_supported_defence(city, &imminent);
                }
                continue;
            }
            if attackers.len() < 3 {
                continue;
            }
            // Campaign and Contain already assigned these field troops; an immediate
            // last-city stand still outranks the operation.
            if self.ctx.own_cities.len() > 1
                && matches!(
                    self.routed_plan().map(|p| &p.kind),
                    Some(
                        crate::plan::PlanKind::Campaign { .. }
                            | crate::plan::PlanKind::Contain { .. }
                    )
                )
            {
                continue;
            }
            // A single opposing empire is the forecast's attacking side. The strongest visible
            // army gets the city's defence priority when several wars reach it at once.
            let owner = attackers
                .iter()
                .map(|e| e.owner)
                .max_by_key(|owner| {
                    (
                        attackers
                            .iter()
                            .filter(|e| e.owner == *owner)
                            .map(|e| {
                                crate::supply::attack_total(
                                    self.view().faction_of(e.owner),
                                    e,
                                    e.hp,
                                )
                            })
                            .sum::<i32>(),
                        std::cmp::Reverse(*owner),
                    )
                })
                .unwrap();
            attackers.retain(|e| e.owner == owner);
            if attackers.len() < 3 {
                continue;
            }
            attackers.sort_by_key(|e| e.id);
            let Some(mut forecast) = self.supported_forecast(city, &attackers, &[], None) else {
                continue;
            };
            let mut garrison = self
                .ctx
                .own_units
                .iter()
                .copied()
                .find(|u| u.military && u.pos == city.pos);
            if garrison.is_some_and(|u| u.ranged > 0) {
                continue;
            }
            if let Some(g) = garrison {
                let leaves = self.orders.iter().any(|o| {
                    legatum_core::rules::units::order_unit(o) == Some(g.id)
                        && matches!(o, Order::MoveUnit { .. } | Order::MoveUnitPath { .. })
                });
                if leaves {
                    let order = if g.ranged > 0 {
                        self.best_shot_automatic(g, &BTreeMap::new())
                            .map(|target| Order::RangedAttack { unit: g.id, target })
                    } else {
                        None
                    }
                    .unwrap_or(Order::SkipUnit { unit: g.id });
                    self.replace_supported_order(g.id, order);
                }
            }
            if garrison.is_none() {
                let melee_reaches = self.ctx.own_units.iter().any(|u| {
                    u.military
                        && u.ranged == 0
                        && self.support_path_this_turn(u, city.pos).is_some()
                });
                if !melee_reaches
                    && let Some(archer) = self
                        .ctx
                        .own_units
                        .iter()
                        .copied()
                        .filter(|u| u.military && u.ranged > 0)
                        .filter(|u| self.support_path_this_turn(u, city.pos).is_some())
                        .min_by_key(|u| (u.pos.distance(city.pos), u.id))
                {
                    self.replace_supported_order(
                        archer.id,
                        Order::MoveUnit {
                            unit: archer.id,
                            to: city.pos,
                        },
                    );
                }
            }
            let mut swap = None;
            // Backfill ranking uses the same HP / worst average hit as battle::soak. A better
            // defender arriving beside a mounted garrison can take its place before the strike.
            if let Some(old) = garrison {
                let terrain = self
                    .view()
                    .tile(city.pos)
                    .and_then(|t| t.terrain)
                    .unwrap_or(legatum_core::Terrain::Plains);
                let better = self
                    .ctx
                    .own_units
                    .iter()
                    .copied()
                    .filter(|u| u.military && u.ranged == 0 && u.pos.distance(city.pos) == 1)
                    .filter(|u| !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
                    .filter(|u| self.support_path_this_turn(u, city.pos).is_some())
                    .filter(|u| {
                        self.defender_soak(u, &attackers, terrain)
                            > self.defender_soak(old, &attackers, terrain)
                    })
                    .max_by_key(|u| {
                        (
                            self.defender_soak(u, &attackers, terrain),
                            std::cmp::Reverse(u.id),
                        )
                    });
                if let Some(new) = better {
                    self.replace_supported_order(
                        old.id,
                        Order::MoveUnit {
                            unit: old.id,
                            to: new.pos,
                        },
                    );
                    self.replace_supported_order(
                        new.id,
                        Order::MoveUnit {
                            unit: new.id,
                            to: city.pos,
                        },
                    );
                    garrison = Some(new);
                    swap = Some((old.id, new.id));
                }
            }
            if swap.is_some() {
                let Some(next) = self.supported_forecast(city, &attackers, &[], swap) else {
                    continue;
                };
                forecast = next;
            }
            let mut supporters = Vec::new();
            for _ in 0..2 {
                let chosen = self
                    .ctx
                    .own_units
                    .iter()
                    .copied()
                    .filter(|u| u.military && u.ranged == 0 && u.pos != city.pos)
                    .filter(|u| {
                        !supporters.contains(&u.id) && garrison.is_none_or(|g| g.id != u.id)
                    })
                    .filter(|u| !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
                    .filter(|u| {
                        u.pos.distance(city.pos)
                            <= attackers
                                .iter()
                                .map(|e| e.pos.distance(city.pos))
                                .min()
                                .unwrap_or(1)
                    })
                    .filter(|u| self.support_path_this_turn(u, city.pos).is_some())
                    .filter_map(|u| {
                        let mut proposed = supporters.clone();
                        proposed.push(u.id);
                        let next = self.supported_forecast(city, &attackers, &proposed, swap)?;
                        let outside = i64::from(self.incoming_at(u, u.pos, u.hp, None));
                        let forgone = self
                            .orders
                            .iter()
                            .find_map(|o| {
                                if legatum_core::rules::units::order_unit(o) != Some(u.id) {
                                    return None;
                                }
                                let to = match o {
                                    Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => {
                                        *to
                                    }
                                    _ => return None,
                                };
                                self.view()
                                    .units
                                    .iter()
                                    .any(|e| e.pos == to && self.view().at_war(u.owner, e.owner))
                                    .then(|| {
                                        legatum_core::forecast_attack_mode(
                                            self.view(),
                                            u.id,
                                            to,
                                            AttackMode::Melee,
                                        )
                                    })
                                    .flatten()
                                    .map(|f| i64::from(f.expected_damage_to_defender) * 2)
                            })
                            .unwrap_or(0);
                        let gain = Self::supported_score(&next)
                            - Self::supported_score(&forecast)
                            - i64::from(u.hp)
                            + outside
                            - forgone;
                        (gain > 0).then_some((u, gain, next))
                    })
                    .max_by_key(|(u, gain, _)| (*gain, std::cmp::Reverse(u.id)));
                let Some((unit, _, next)) = chosen else { break };
                supporters.push(unit.id);
                self.replace_supported_order(
                    unit.id,
                    Order::MoveUnit {
                        unit: unit.id,
                        to: city.pos,
                    },
                );
                forecast = next;
            }
        }
    }

    fn replace_supported_order(&mut self, unit: legatum_core::UnitId, order: Order) {
        if let Some(old) = self
            .orders
            .iter_mut()
            .find(|o| legatum_core::rules::units::order_unit(o) == Some(unit))
        {
            *old = order;
        } else {
            self.unit_order(unit, order);
        }
    }

    pub(crate) fn settler_safety(&self) -> bool {
        (self.decisions.settler_safety > 0 || self.decisions.peace_settler_capture > 0)
            && self.difficulty == AiDifficulty::Hard
            && self.view().players.len() > 2
    }

    pub(crate) fn hard_defense(&self) -> bool {
        self.difficulty == AiDifficulty::Hard && self.opening.is_none()
    }

    pub(crate) fn prepare_hard_defense(&mut self) {
        if !self.hard_defense() {
            return;
        }
        let v = self.view();
        let front_ready = self
            .runaway_read()
            .is_some_and(|leader| self.busy_leader_front(leader.team));
        let rivals: Vec<_> = v
            .units
            .iter()
            .filter(|u| {
                u.military
                    && !v.same_team(v.me, u.owner)
                    && (v.at_war(v.me, u.owner)
                        || v.diplomacy.actions.iter().any(|a| {
                            a.order
                                == (Order::DeclareWar {
                                    against: v.team_of(u.owner),
                                    breach: false,
                                })
                                && a.blocked_reason.is_none()
                        }))
            })
            .collect();
        let mut entries: BTreeMap<
            (legatum_core::PlayerId, legatum_core::UnitKind),
            legatum_core::view::EntryTable,
        > = BTreeMap::new();
        // A rival's reach depends on its own entry rule and the map; a prospective planner
        // (our relation with one other team changed) shares the main planner's.
        let shared = self.shared_routes.unwrap_or(&self.declaration_routes);
        let routes: BTreeMap<_, _> = rivals
            .iter()
            .filter(|u| !v.at_war(v.me, u.owner))
            .map(|u| {
                if let Some(r) = shared.borrow().get(&u.id) {
                    return (u.id, r.clone());
                }
                let entry = *entries
                    .entry((u.owner, u.kind))
                    .or_insert_with(|| v.entry_table(u.owner, u.kind));
                let r = std::rc::Rc::new(declaration_reach(v, u, &entry));
                shared.borrow_mut().insert(u.id, r.clone());
                (u.id, r)
            })
            .collect();
        let reachable = |u: &UnitView, h| {
            routes.get(&u.id).map_or_else(
                || self.ctx.reachable(u.pos, h),
                |r| v.index(h).is_some_and(|i| r[i]),
            )
        };
        let mut danger = self.ctx.settler_danger.clone();
        let mut threats = self.ctx.threat.clone();
        // A peaceful border is not a license to walk an unguarded settler through
        // another army. This is avoidance, never permission to attack a neutral unit.
        for u in &rivals {
            // The old policy ignored peaceful Scouts; the new prior includes them only
            // when a declaration is plausible enough to warrant an escort. The separate
            // Scout gene can still force caution for every peaceful Scout.
            if !v.at_war(v.me, u.owner)
                && !(u.kind == UnitKind::Scout && self.decisions.peace_scout_danger > 0)
                // Gene `snipe_read`: a known sniper's Scout takes a Settler like any troop.
                && !self.recent_surprise_war(u.owner)
                && (self.decisions.peace_settler_capture > 0
                    && self.peace_settler_capture_chance(u.owner)
                        < self.decisions.settler_risk_escort
                    || self.decisions.peace_settler_capture == 0 && u.kind == UnitKind::Scout)
            {
                continue;
            }
            for h in u.pos.within(u.max_moves.max(u.range)) {
                if let Some(i) = v.index(h)
                    && (u.ranged > 0 && u.pos.distance(h) <= u.range || reachable(u, h))
                {
                    danger[i] = true;
                }
            }
        }
        // An army threatens the city it stands nearest to (and any within the slack of it), so
        // a stack between two cities doesn't split their defenders.
        let slack = self.decisions.threat_aim_slack;
        let aimed = |u: &UnitView, c: legatum_core::Hex| {
            let nearest = self
                .ctx
                .own_cities
                .iter()
                .map(|o| u.pos.distance(o.pos))
                .min()
                .unwrap_or(0);
            u.pos.distance(c) <= nearest + slack
        };
        for (i, c) in self.ctx.own_cities.iter().enumerate() {
            threats[i] = self
                .ctx
                .enemy_units
                .iter()
                .filter(|u| {
                    u.military
                        && (u.kind != UnitKind::Scout || self.decisions.scout_threat > 0)
                        && u.pos.distance(c.pos) <= crate::analysis::THREAT_RADIUS
                        && !(u.owner.is_barbarian() && self.view().barbarians_spare_me())
                        && (u.owner.is_barbarian() || aimed(u, c.pos))
                })
                .map(|u| {
                    let weight = if u.kind == UnitKind::Scout {
                        self.decisions.scout_threat
                    } else {
                        100
                    };
                    threat_power(v, u) * weight / 100
                })
                .sum();
        }
        for (i, c) in self.ctx.own_cities.iter().enumerate() {
            let approaching: Vec<_> = rivals
                .iter()
                .filter(|u| {
                    aimed(u, c.pos)
                        && !u.owner.is_barbarian()
                        && (u.kind != UnitKind::Scout || self.decisions.scout_threat > 0)
                        && if self.decisions.threat_horizon_turns > 0 {
                            declaration_arrival_turns(
                                v,
                                u,
                                c.pos,
                                self.decisions.threat_horizon_turns as u32,
                            )
                            .is_some()
                        } else {
                            u.pos.distance(c.pos) <= 5 && reachable(u, c.pos)
                        }
                })
                .collect();
            let pressure: i32 = approaching
                .iter()
                .map(|u| {
                    let chance = if v.at_war(v.me, u.owner) {
                        100
                    } else {
                        self.peace_declaration_percent(u.owner, u.pos)
                            * self.decisions.peace_exposure_percent
                            / 100
                    };
                    let weight = if u.kind == UnitKind::Scout {
                        self.decisions.scout_threat
                    } else {
                        100
                    };
                    threat_power(v, u) * chance * weight / 10_000
                })
                .sum();
            threats[i] = threats[i]
                .max(pressure + self.ctx.barbarian_threat[i])
                .max(self.early_warning_power(c))
                .max(if front_ready && self.local_defense(c) > 0 {
                    0
                } else {
                    self.neighbour_fall_power(c) / 2
                });
        }
        if self.decisions.relieve_radius > 0 {
            self.relief = self
                .ctx
                .own_cities
                .iter()
                .enumerate()
                .filter(|(i, _)| threats[*i] > self.ctx.barbarian_threat[*i])
                .map(|(_, c)| c.pos)
                .collect();
        }
        self.ctx.settler_danger = danger;
        self.ctx.threat = threats;
    }

    /// Do not send the city's only healthy melee guard chasing a shooter while a
    /// second attacker can take the vacated city. Other units still counterattack.
    pub(crate) fn reserve_front_guards(&mut self, army: &[&UnitView]) {
        if !self.hard_defense() {
            return;
        }
        for c in self.ctx.own_cities.clone() {
            let network_city = self.faction == legatum_core::Faction::India
                && self.decisions.india_network_guard > 0
                && self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|other| other.id != c.id && other.pos.distance(c.pos) <= 5);
            let attackers = self
                .ctx
                .enemy_units
                .iter()
                .filter(|u| {
                    u.military
                        && !u.owner.is_barbarian()
                        && (u.kind != UnitKind::Scout || self.decisions.scout_threat > 0)
                        && u.pos.distance(c.pos) <= 4
                })
                .count();
            if attackers < if network_city { 1 } else { 2 } {
                continue;
            }
            if !self.ctx.enemy_units.iter().any(|u| {
                u.military
                    && crate::composition::charges(u.kind)
                    && !u.owner.is_barbarian()
                    && (u.kind != UnitKind::Scout || self.decisions.scout_threat > 0)
                    && self.arrives(u, c.pos)
            }) {
                continue;
            }
            if let Some(u) = self.free(army).into_iter().find(|u| {
                u.pos == c.pos
                    && crate::composition::charges(u.kind)
                    && self.can_defend_at(u, c.pos)
            }) {
                self.hold_or_upgrade(u);
            }
        }
    }

    /// Where a civilian's move order this turn leaves it, after its whole movement budget.
    pub(crate) fn settler_arrival(&self, s: &UnitView) -> Option<legatum_core::Hex> {
        let path = self.orders.iter().find_map(|o| match o {
            Order::MoveUnitPath { unit, path, .. } if *unit == s.id => Some(path.clone()),
            Order::MoveUnit { unit, to } if *unit == s.id => {
                self.view()
                    .movement_path(self.view().me, s.kind, s.pos, *to)
            }
            _ => None,
        })?;
        let mut arrival = s.pos;
        let budget = if self.settler_safety() {
            self.turn_moves(s)
        } else {
            s.max_moves
        };
        let mut moves = budget;
        for h in path {
            let Some(cost) = self.ctx.move_cost(h) else {
                break;
            };
            if !can_step(moves, budget, cost) {
                break;
            }
            arrival = h;
            moves = (moves - cost).max(0);
        }
        Some(arrival)
    }

    /// A fast raider can intercept an intermediate step even when the Settler
    /// finishes the movement phase outside its reach.
    fn settler_route_exposed(&self, s: &UnitView) -> bool {
        let path = self.orders.iter().find_map(|o| match o {
            Order::MoveUnitPath { unit, path, .. } if *unit == s.id => Some(path.clone()),
            Order::MoveUnit { unit, to } if *unit == s.id => {
                self.view()
                    .movement_path(self.view().me, s.kind, s.pos, *to)
            }
            _ => None,
        });
        let Some(path) = path else { return false };
        let budget = self.turn_moves(s);
        let mut moves = budget;
        for h in path {
            let Some(cost) = self.ctx.move_cost(h) else {
                return false;
            };
            if !can_step(moves, budget, cost) {
                break;
            }
            if self.settler_capturable_this_turn(h) {
                return true;
            }
            moves = (moves - cost).max(0);
        }
        false
    }

    /// Whether a rival that could attack a Settler could actually enter this tile during the
    /// current movement phase. `snipe_exposed` adds a step to warn about next-turn danger.
    pub(crate) fn settler_capturable_this_turn(&self, tile: legatum_core::Hex) -> bool {
        let v = self.view();
        v.units.iter().any(|u| {
            u.military
                && !v.same_team(v.me, u.owner)
                && (u.owner.is_barbarian() && !v.barbarians_spare_me()
                    || !u.owner.is_barbarian()
                        && (v.at_war(v.me, u.owner)
                            || crate::opportunity::legal_declaration(v, v.team_of(u.owner))))
                && (self.decisions.peace_settler_capture == 0
                    || self.peace_settler_capture_chance(u.owner)
                        >= self.decisions.settler_risk_escort)
                && u.pos.distance(tile)
                    <= if self.settler_safety() {
                        self.turn_moves(u)
                    } else {
                        u.max_moves
                    }
                && self.ctx.reachable(u.pos, tile)
        })
    }

    /// When an army appears on a trip, use the Settler's own movement to get back toward
    /// its nearest city. Confirm the whole escape path so the engine cannot repath through
    /// the raider after orders resolve.
    fn recall_exposed_settler(&mut self, s: &UnitView) -> bool {
        let Some(home) = self.ctx.nearest_own_city(s.pos).map(|c| c.pos) else {
            return false;
        };
        let mut best = None;
        for h in s.pos.within(self.turn_moves(s)) {
            if h == s.pos
                || self.settler_capturable_this_turn(h)
                || self
                    .view()
                    .tile(h)
                    .is_some_and(|tile| tile.owner.is_some_and(|o| self.view().at_war(s.owner, o)))
            {
                continue;
            }
            let Some(path) = self.view().movement_path(s.owner, s.kind, s.pos, h) else {
                continue;
            };
            let mut remaining = self.turn_moves(s);
            if path.iter().any(|step| {
                let cost = self.ctx.move_cost(*step).unwrap_or(i32::MAX);
                if !can_step(remaining, self.turn_moves(s), cost)
                    || self.settler_capturable_this_turn(*step)
                    || self
                        .view()
                        .index(*step)
                        .is_none_or(|i| self.ctx.hostile_at[i] || self.ctx.neutral_at[i])
                {
                    return true;
                }
                remaining = (remaining - cost).max(0);
                false
            }) {
                continue;
            }
            // Leave the opponent's next-turn reach if possible. A move merely closer to
            // home can still be captured before the following planning pass.
            let key = (
                self.snipe_exposed(h),
                self.settler_trip_risk(h, h),
                h.distance(home),
                s.pos.distance(h),
                h,
            );
            if best.as_ref().is_none_or(|(old, _)| key < *old) {
                best = Some((key, path));
            }
        }
        let Some(((_, _, _, _, to), path)) = best else {
            return false;
        };
        let recall = Order::MoveUnitPath {
            unit: s.id,
            to,
            path,
            stop_on_block: true,
        };
        if let Some(existing) = self
            .orders
            .iter_mut()
            .find(|order| legatum_core::rules::units::order_unit(order) == Some(s.id))
        {
            *existing = recall;
            true
        } else {
            self.unit_order(s.id, recall)
        }
    }

    /// Keep a local escort beside its civilian through the next movement phase.
    /// A distant founding destination is not the settler's actual position this turn.
    pub(crate) fn guard_exposed_settlers(&mut self, army: &[&UnitView]) {
        if !self.settler_safety() {
            if self.hard_defense() {
                self.guard_exposed_settlers_duel(army);
            }
            return;
        }
        for s in self
            .ctx
            .own_units
            .clone()
            .into_iter()
            .filter(|u| u.kind == UnitKind::Settler)
        {
            let found = self
                .orders
                .iter()
                .any(|o| matches!(o,Order::FoundCity {unit} if *unit==s.id));
            // Only commit a guard for exposure in the upcoming movement phase. Distant
            // threats affect site choice, then are re-evaluated when the Settler nears them.
            let arrival = self.settler_arrival(s).unwrap_or(s.pos);
            let route_exposed = self.settler_route_exposed(s);
            let origin_guarded = self
                .free(army)
                .iter()
                .any(|u| u.pos == s.pos && u.hp >= self.decisions.recover_hp);
            let origin_capturable = self.settler_capturable_this_turn(s.pos) && !origin_guarded;
            let threatened = route_exposed
                || origin_capturable
                || self.settler_trip_risk(s.pos, arrival) >= self.decisions.settler_risk_escort
                || self.settler_capturable_this_turn(arrival);
            if !threatened {
                continue;
            }
            let escort = self
                .free(army)
                .into_iter()
                .filter(|u| {
                    u.hp >= self.decisions.recover_hp
                        && (!self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                            || u.pos == s.pos
                                && self
                                    .ctx
                                    .own_cities
                                    .iter()
                                    .position(|c| c.pos == u.pos)
                                    .is_some_and(|i| {
                                        self.ctx.threat[i] == 0
                                            || (self.ctx.own_cities[i].hp > 0
                                                && self.ctx.threat_units.iter().all(|enemy| {
                                                    !enemy.military || enemy.kind == UnitKind::Scout
                                                })
                                                && self.ctx.remembered_threats.is_empty())
                                    }))
                        && self.free_tile_for(u, s.pos)
                        && self.reaches_this_turn(u, s.pos)
                })
                .min_by_key(|u| {
                    (
                        crate::composition::shooter_only(u.kind, u.ranged),
                        u.pos.distance(s.pos),
                        u.id,
                    )
                });
            let Some(u) = escort else {
                if (self.snipe_exposed(s.pos)
                    || self.settler_trip_risk(s.pos, s.pos) >= self.decisions.settler_risk_escort)
                    && self.recall_exposed_settler(s)
                {
                    continue;
                }
                // No escort this turn: rather than end the move where a rival could stand
                // beside it (and take it next turn), wait on a safe tile for one.
                // The exposure radius includes next-turn reach, so the current tile may
                // already be flagged even though a peaceful rival cannot take it this turn.
                if !found
                    && !self.settler_capturable_this_turn(s.pos)
                    && (route_exposed
                        || self.settler_arrival(s).is_some_and(|a| {
                            self.settler_trip_risk(a, a) >= self.decisions.settler_risk_escort
                                || self.settler_capturable_this_turn(a)
                        }))
                    && let Some(order) = self.orders.iter_mut().find(|o| {
                        matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. }
                            if *unit == s.id)
                    })
                {
                    *order = Order::SkipUnit { unit: s.id };
                }
                continue;
            };
            if found || origin_capturable {
                self.go_to(u, s.pos);
                continue;
            }
            // Predict the civilian's position after its whole movement budget, while
            // leaving its founding destination and confirmed detour intact.
            if arrival != s.pos
                && self.free_tile_for(u, arrival)
                && self.reaches_this_turn(u, arrival)
                && self.safe_campaign_tile(arrival)
                && self
                    .view()
                    .index(arrival)
                    .is_some_and(|i| !self.ctx.settler_danger[i])
            {
                self.go_to(u, arrival);
            } else {
                // An exposed transit step is only safe while the guard can follow.
                if route_exposed
                    && let Some(order) = self.orders.iter_mut().find(|o| {
                        matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. }
                            if *unit == s.id)
                    })
                {
                    *order = Order::SkipUnit { unit: s.id };
                }
                self.go_to(u, s.pos);
            }
        }
    }

    /// The existing two-player policy is the paired proxy7 control. Keep its
    /// escort timing while the broader safety policy is measured in multiplayer.
    /// Gene `settler_escort`: a garrison may leave a city to escort a threatened Settler
    /// when the city itself has nothing to fear this turn. The Settler is a city in waiting,
    /// worth more than an empty town that no enemy can reach (a city is taken only at 0 HP,
    /// never by a Scout walking in). No threat on the city (`ctx.threat`, visible military
    /// aimed at it) and no enemy at war in sight within the threat radius.
    fn garrison_can_escort(&self, city: usize) -> bool {
        self.decisions.settler_escort > 0 && self.quiet_city(city)
    }

    /// No visible military aimed at the city (`ctx.threat`) and no enemy at war in sight
    /// within the threat radius: its garrison may step out for a turn or two.
    pub(crate) fn quiet_city(&self, city: usize) -> bool {
        let v = self.view();
        self.ctx.threat[city] == 0
            && !self.ctx.enemy_units.iter().any(|e| {
                e.military
                    && (e.owner.is_barbarian() && !v.barbarians_spare_me()
                        || !e.owner.is_barbarian() && v.at_war(v.me, e.owner))
                    && e.pos.distance(self.ctx.own_cities[city].pos)
                        <= crate::analysis::THREAT_RADIUS
            })
    }

    /// Gene `settler_escort`: the route passes within two hexes of a city of a rival that can
    /// declare a surprise war. A city is where its troops stand, seen or not; the visible-unit
    /// risk alone let the 8.9.24 Mongol escort turn back when the Swiss Spearman stood in fog.
    fn settler_route_near_rival_city(
        &self,
        from: legatum_core::Hex,
        to: legatum_core::Hex,
    ) -> bool {
        if self.decisions.settler_escort < 2 {
            return false;
        }
        let v = self.view();
        let mut route = v
            .movement_path(v.me, UnitKind::Settler, from, to)
            .unwrap_or_default();
        route.push(from);
        v.cities.iter().any(|c| {
            !c.owner.is_barbarian()
                && !v.same_team(v.me, c.owner)
                && !v.allied(v.me, c.owner)
                && !v.at_war(v.me, c.owner)
                && crate::opportunity::legal_declaration(v, v.team_of(c.owner))
                && route.iter().any(|h| h.distance(c.pos) <= 2)
        })
    }

    /// Gene `capital_guard`: a rival army stands within the threat radius of the capital and
    /// can walk to it: two or more troops (not Scouts) of one empire at war or at Peace with a
    /// legal declaration, or one of an empire at war or with a recent surprise war
    /// (`recent_surprise_war`). Barbarians never assault a city. A single troop of a quiet
    /// neighbour is not enough: in the `india-take-the-settler` Lab position a Spartan Spearman
    /// parked three hexes from Leipzig for twenty turns, and holding the garrison for it froze
    /// Germany's first Settler (Germany's cities at T36 2.90 to 1.87 over 200 runs).
    fn capital_in_rival_reach(&self, capital: legatum_core::Hex) -> bool {
        let v = self.view();
        let mut troops: BTreeMap<legatum_core::PlayerId, usize> = BTreeMap::new();
        for u in v.units.iter().filter(|u| {
            u.military
                && u.kind != UnitKind::Scout
                && !u.owner.is_barbarian()
                && !v.same_team(v.me, u.owner)
                && !v.allied(v.me, u.owner)
                && (v.at_war(v.me, u.owner)
                    || crate::opportunity::legal_declaration(v, v.team_of(u.owner)))
                && u.pos.distance(capital) <= crate::analysis::THREAT_RADIUS
                && self.ctx.reachable(u.pos, capital)
        }) {
            *troops.entry(u.owner).or_default() += 1;
        }
        troops
            .into_iter()
            .any(|(owner, n)| n >= 2 || v.at_war(v.me, owner) || self.recent_surprise_war(owner))
    }

    /// Gene `capital_guard` (8.9.25 India, T12-T22: Germany's only Spearman walked its first
    /// Settler out of Leipzig with a Spartan and then an Indian Spearman in reach of the empty
    /// capital; India killed the escort, took the Settler and walked into Leipzig): while a
    /// rival army is in reach of the capital, its last troop stays home. A lone defender in it
    /// is held back from escorts. An empty capital calls its Settlers within
    /// `CAPITAL_RECALL` hexes home (their escorts walk back with them), and the nearest troop
    /// beside a Settler that gets home this turn goes home with it. Returns the troop held.
    fn hold_capital_guard<'u>(&mut self, army: &[&'u UnitView]) -> Option<&'u UnitView> {
        let city = self.ctx.capital()?;
        let capital = city.pos;
        // The capitals victory (rules 37) makes the capital's fall a loss of the game: a known
        // rival unit in reach holds the guard whatever the gene says.
        let pressed = self.capital_pressed(city);
        if (self.decisions.capital_guard == 0 && !pressed)
            || !(pressed || self.capital_in_rival_reach(capital))
        {
            return None;
        }
        let troop = |u: &UnitView| u.military && u.kind != UnitKind::Scout;
        let inside: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| troop(u) && u.pos == capital)
            .collect();
        if inside.len() > 1 {
            return None;
        }
        if let Some(&only) = inside.first() {
            return army.iter().copied().find(|u| u.id == only.id);
        }
        let cities: Vec<legatum_core::Hex> = self.ctx.own_cities.iter().map(|c| c.pos).collect();
        let settlers: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| {
                u.kind == UnitKind::Settler
                    && !cities.contains(&u.pos)
                    && u.pos.distance(capital) <= CAPITAL_RECALL
                    && !self
                        .orders
                        .iter()
                        .any(|o| matches!(o, Order::FoundCity { unit } if *unit == u.id))
            })
            .collect();
        for s in &settlers {
            let home = Order::MoveUnit {
                unit: s.id,
                to: capital,
            };
            if let Some(existing) = self
                .orders
                .iter_mut()
                .find(|o| legatum_core::rules::units::order_unit(o) == Some(s.id))
            {
                *existing = home;
            } else {
                self.unit_order(s.id, home);
            }
        }
        let guard = self
            .free(army)
            .into_iter()
            .filter(|u| {
                crate::composition::charges(u.kind)
                    && u.hp >= self.decisions.recover_hp
                    && self.reaches_this_turn(u, capital)
                    && settlers
                        .iter()
                        .any(|s| s.pos.distance(u.pos) <= 2 && self.reaches_this_turn(s, capital))
            })
            .min_by_key(|u| (u.pos.distance(capital), u.id))?;
        self.go_to(guard, capital);
        Some(guard)
    }

    /// Gene `capital_guard`: a threatened Settler whose escort holds the capital stays in, or
    /// goes back to, the nearest city it reaches this turn instead of travelling alone.
    fn keep_settler_home(&mut self, s: &UnitView) -> bool {
        let Some(home) = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| c.pos == s.pos || self.reaches_this_turn(s, c.pos))
            .min_by_key(|c| (c.pos.distance(s.pos), c.id))
            .map(|c| c.pos)
        else {
            return false;
        };
        let order = if home == s.pos {
            Order::SkipUnit { unit: s.id }
        } else {
            Order::MoveUnit {
                unit: s.id,
                to: home,
            }
        };
        if let Some(existing) = self
            .orders
            .iter_mut()
            .find(|o| legatum_core::rules::units::order_unit(o) == Some(s.id))
        {
            *existing = order;
            true
        } else {
            self.unit_order(s.id, order)
        }
    }

    /// Gene `snipe_read`: rival troops other than Scouts that could stand on `tile` after
    /// their next move. One escort holds a Settler's tile against one attacker for a turn;
    /// two or more break it (8.9.25 India T16-T17: two Spearmen killed Germany's escort and
    /// took the Settler).
    fn troops_in_reach(&self, tile: legatum_core::Hex) -> usize {
        let v = self.view();
        v.units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && !v.same_team(v.me, u.owner)
                    && !v.allied(v.me, u.owner)
                    && if u.owner.is_barbarian() {
                        !v.barbarians_spare_me()
                    } else {
                        v.at_war(v.me, u.owner)
                            || crate::opportunity::legal_declaration(v, v.team_of(u.owner))
                    }
                    && u.pos.distance(tile) <= self.turn_moves(u) + 1
                    && self.ctx.reachable(u.pos, tile)
            })
            .count()
    }

    /// Gene `escort_buy` (8.9.25 India: Germany bought a Pasture and a Mine with its 140
    /// Gold of ruin and wonder rewards at T2-T4 and sent its first Settler out with its only
    /// Spearman, which then had to choose between the Settler and the capital): while we have
    /// at most one troop, a Settler is out or in production, and a rival army (a troop other
    /// than a Scout of an empire at war, a raiding barbarian or a rival at Peace that can
    /// declare) stands within `SECOND_TROOP_RADIUS` of it or of the city training it, a second
    /// troop is worth the Settler's own quote times the gene's percent (the chance a visible
    /// army takes a Settler nobody guards). `plan_buy` buys a melee troop that is
    /// worth that, before any routine purchase, from the city nearest the Settler; when it is
    /// not yet affordable but will be within `SECOND_TROOP_SAVE_TURNS` turns of income,
    /// routine purchases keep its price back (`Err`, the Gold to leave).
    pub(crate) fn second_troop_buy(
        &self,
    ) -> Option<Result<(legatum_core::CityId, legatum_core::ProductionItem, i32), i32>> {
        if self.decisions.escort_buy == 0 || self.difficulty != AiDifficulty::Hard {
            return None;
        }
        let v = self.view();
        let troops = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .count();
        if troops >= 2
            || self.orders.iter().any(|o| {
                matches!(o, Order::BuyItem { item, .. } if crate::production::is_military_unit(*item))
            })
        {
            return None;
        }
        let settler = legatum_core::ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        let anchors: Vec<legatum_core::Hex> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.kind == UnitKind::Settler)
            .map(|u| u.pos)
            .chain(
                self.ctx
                    .own_cities
                    .iter()
                    .filter(|c| {
                        c.detail
                            .as_ref()
                            .is_some_and(|d| d.producing == Some(settler))
                    })
                    .map(|c| c.pos),
            )
            .collect();
        let army_near = v.units.iter().any(|u| {
            u.military
                && u.kind != UnitKind::Scout
                && !v.same_team(v.me, u.owner)
                && !v.allied(v.me, u.owner)
                && if u.owner.is_barbarian() {
                    !v.barbarians_spare_me()
                } else {
                    v.at_war(v.me, u.owner)
                        || crate::opportunity::legal_declaration(v, v.team_of(u.owner))
                }
                && anchors.iter().any(|a| {
                    a.distance(u.pos) <= SECOND_TROOP_RADIUS && self.ctx.reachable(u.pos, *a)
                })
        });
        if !army_near {
            return None;
        }
        let anchor = anchors[0];
        let city = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| {
                c.detail.as_ref().is_some_and(|d| d.resistance == 0)
                    && !self
                        .orders
                        .iter()
                        .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
            })
            .min_by_key(|c| (c.pos.distance(anchor), c.id))?;
        let d = city.detail.as_ref()?;
        let settler_quote = d
            .options
            .iter()
            .find(|o| o.item == settler)
            .and_then(|o| o.buy_cost.or(Some(2 * o.cost)))
            .unwrap_or(2 * legatum_core::data::unit_stats(UnitKind::Settler).cost);
        let worth = settler_quote * self.decisions.escort_buy / 100;
        let choices: Vec<_> = d
            .options
            .iter()
            .filter_map(|o| {
                let legatum_core::ProductionItem::Unit { unit } = o.item else {
                    return None;
                };
                if !crate::composition::charges(unit)
                    || unit == UnitKind::Scout
                    || !crate::production::is_military_unit(o.item)
                    || o.buy_block.is_some_and(|b| {
                        !matches!(b, legatum_core::economy::BuyBlock::NotEnoughGold)
                    })
                {
                    return None;
                }
                let cost = o.buy_cost.filter(|c| *c > 0)?;
                Some((o.item, cost, o.buy_block.is_none()))
            })
            .filter(|(_, cost, _)| self.decisions.named_fund > 0 || *cost <= worth)
            .filter(|(_, cost, _)| {
                self.decisions.named_fund == 0
                    || *cost <= self.gold_left - self.ordinary_gold_reserve()
            })
            .filter(|(item, _, _)| {
                if self.decisions.named_fund == 0 {
                    return true;
                }
                let legatum_core::ProductionItem::Unit { unit } = item else {
                    return false;
                };
                let strength = v
                    .rule_experiments
                    .unit_stats_for(self.faction, *unit)
                    .strength;
                v.units
                    .iter()
                    .filter(|enemy| {
                        enemy.military
                            && !v.same_team(v.me, enemy.owner)
                            && enemy.pos.distance(anchor) <= SECOND_TROOP_RADIUS
                    })
                    .all(|enemy| {
                        legatum_core::combat::base_damage(enemy.strength, strength)
                            < legatum_core::data::UNIT_HP
                    })
            })
            .collect();
        let (item, cost, now) = if self.decisions.hard_purchase_scorer > 0 {
            choices.into_iter().max_by_key(|(item, cost, _)| {
                let legatum_core::ProductionItem::Unit { unit } = item else {
                    unreachable!()
                };
                (self.military_purchase_score(*unit, *cost, city.pos), -*cost)
            })?
        } else if self.decisions.named_fund > 0 {
            choices.into_iter().min_by_key(|(item, cost, _)| {
                let legatum_core::ProductionItem::Unit { unit } = item else {
                    unreachable!()
                };
                let stats = v.rule_experiments.unit_stats_for(self.faction, *unit);
                (
                    -i32::from(legatum_core::data::unit_era_tier(*unit)),
                    -stats.strength,
                    *cost,
                )
            })?
        } else {
            choices.into_iter().min_by_key(|(_, cost, _)| *cost)?
        };
        if self.military_upgrade_hold(city) {
            return None;
        }
        if let legatum_core::ProductionItem::Unit { unit } = item
            && !self.military_purchase_viable(city, unit, cost)
        {
            return None;
        }
        let keep = self.ordinary_gold_reserve();
        if now && self.gold_left - cost >= keep {
            return Some(Ok((city.id, item, cost)));
        }
        let income = v.economy.gold_per_turn.total;
        let gap = keep + cost - self.gold_left;
        (income > 0 && gap <= income * SECOND_TROOP_SAVE_TURNS).then_some(Err(keep + cost))
    }

    /// A troop that may walk with Settler `s` on plain Hard's escort path.
    fn duel_can_escort(&self, u: &UnitView, s: &UnitView, held_home: Option<&UnitView>) -> bool {
        crate::composition::charges(u.kind)
            && held_home.is_none_or(|g| g.id != u.id)
            && u.hp >= self.decisions.recover_hp
            && self
                .ctx
                .own_cities
                .iter()
                .position(|c| c.pos == u.pos)
                .is_none_or(|i| self.garrison_can_escort(i))
            && self.free_tile_for(u, s.pos)
    }

    /// A Settler held back for an unseen threat (fog, a rival's border) may wait only in one of
    /// our cities or on its escort's tile; on open ground it goes back to the nearest city
    /// instead (8.9.40 Byzantium T16: the Settler waited two tiles from Piraeus for an escort
    /// while his Horseman stood three tiles off in fog). False when it is already in a city.
    fn fog_wait_home(&mut self, s: &UnitView) -> bool {
        if self.ctx.own_cities.iter().any(|c| c.pos == s.pos) {
            return false;
        }
        let Some(home) = self.ctx.nearest_own_city(s.pos).map(|c| c.pos) else {
            return false;
        };
        let order = Order::MoveUnit {
            unit: s.id,
            to: home,
        };
        if let Some(existing) = self
            .orders
            .iter_mut()
            .find(|o| legatum_core::rules::units::order_unit(o) == Some(s.id))
        {
            *existing = order;
            true
        } else {
            self.unit_order(s.id, order)
        }
    }

    fn guard_exposed_settlers_duel(&mut self, army: &[&UnitView]) {
        let held_home = self.hold_capital_guard(army);
        // An escort stays with its Settler until the city is founded or the Settler is back in
        // a city (8.9.40 Byzantium T89: the Horseman called to the Settler was released the
        // turn the threat read fell to 0). Pairs whose Settler or escort is gone end here.
        let alive: std::collections::BTreeSet<legatum_core::UnitId> =
            self.ctx.own_units.iter().map(|u| u.id).collect();
        self.campaigns
            .settler_escorts
            .retain(|settler, escort| alive.contains(settler) && alive.contains(escort));
        for s in self
            .ctx
            .own_units
            .clone()
            .into_iter()
            .filter(|u| u.kind == UnitKind::Settler)
        {
            let found = self
                .orders
                .iter()
                .any(|o| matches!(o, Order::FoundCity { unit } if *unit == s.id));
            let destination = self.orders.iter().find_map(|o| match o {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                    if *unit == s.id =>
                {
                    Some(*to)
                }
                _ => None,
            });
            if !found && destination.is_none() {
                self.campaigns.settler_escorts.remove(&s.id);
                continue;
            }
            let fog = self.snipe_fog_risk(s.pos, destination.unwrap_or(s.pos));
            let near_rival_city =
                self.settler_route_near_rival_city(s.pos, destination.unwrap_or(s.pos));
            let threatened = self.settler_trip_risk(s.pos, destination.unwrap_or(s.pos))
                >= self.decisions.settler_risk_escort
                || near_rival_city
                || fog >= self.decisions.settler_risk_escort;
            // The threat comes from what we cannot see (fog, a rival's border): the wait tile
            // cannot be judged safe by the units in sight.
            let fog_driven = near_rival_city
                || fog >= self.decisions.settler_risk_escort
                || self.fog_route_risk(s.pos, destination.unwrap_or(s.pos))
                    >= self.decisions.settler_risk_escort;
            let paired = self
                .campaigns
                .settler_escorts
                .get(&s.id)
                .copied()
                .and_then(|id| self.free(army).into_iter().find(|u| u.id == id))
                .filter(|u| self.duel_can_escort(u, s, held_home));
            // Gene `snipe_read`: a Settler falling back into one of our cities keeps its
            // escort beside it even from a tile a rival could strike (8.9.25 India T15: the
            // German Settler turned home beside an Indian Spearman and its escort stood still).
            let heading_home = self
                .ctx
                .own_cities
                .iter()
                .any(|c| Some(c.pos) == destination);
            let retreat_escort = self.decisions.snipe_read > 0 && heading_home;
            if !threatened && paired.is_none()
                || !self.safe_campaign_tile(s.pos) && !retreat_escort && paired.is_none()
            {
                continue;
            }
            let escort = paired
                .filter(|u| self.reaches_this_turn(u, s.pos))
                .or_else(|| {
                    self.free(army)
                        .into_iter()
                        .filter(|u| {
                            self.duel_can_escort(u, s, held_home)
                                && self.reaches_this_turn(u, s.pos)
                        })
                        .min_by_key(|u| (u.pos.distance(s.pos), u.id))
                });
            // No threat now and the escort is not beside it: the escort walks to the Settler.
            if !threatened && escort.is_none() {
                if let Some(u) = paired {
                    self.go_to(u, s.pos);
                }
                continue;
            }
            // Gene `capital_guard`: the troop that would walk with it is holding the capital.
            if escort.is_none() && held_home.is_some() && !found && self.keep_settler_home(s) {
                continue;
            }
            // Gene `settler_escort`: no escort reaches it this turn, but one reaches it next
            // turn. The Settler waits where nobody can take it this turn and the escort comes
            // (8.9.24: the Mongol Settler walked on alone from the escort that turned back,
            // two hexes past Thun, and was taken). A turn's wait costs less than the city.
            if escort.is_none()
                && self.decisions.settler_escort >= 2
                && !found
                // A Settler already falling back into one of our cities keeps going.
                && !self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|c| Some(c.pos) == destination)
                && !self.settler_capturable_this_turn(s.pos)
                && let Some(target) = self.view().index(s.pos)
                && let Some(u) = self
                    .free(army)
                    .into_iter()
                    .filter(|u| self.duel_can_escort(u, s, held_home))
                    .map(|u| (self.ctx.distance_field(u.pos, false)[target], u))
                    .filter(|(cost, u)| *cost <= 2 * self.turn_moves(u))
                    .min_by_key(|(cost, u)| (*cost, u.id))
                    .map(|(_, u)| u)
            {
                if fog_driven && self.fog_wait_home(s) {
                    continue;
                }
                if let Some(order) = self.orders.iter_mut().find(|o| {
                    matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. }
                        if *unit == s.id)
                }) {
                    *order = Order::SkipUnit { unit: s.id };
                }
                self.go_to(u, s.pos);
                self.campaigns.settler_escorts.insert(s.id, u.id);
                continue;
            }
            let Some(u) = escort else {
                let unsafe_next = self
                    .settler_arrival(s)
                    .is_some_and(|a| self.snipe_exposed(a))
                    || self.fog_route_risk(s.pos, destination.unwrap_or(s.pos))
                        >= self.decisions.settler_risk_escort;
                if !found
                    && !self.settler_capturable_this_turn(s.pos)
                    && unsafe_next
                    && fog_driven
                    && self.fog_wait_home(s)
                {
                    continue;
                }
                if !found
                    && !self.settler_capturable_this_turn(s.pos)
                    && unsafe_next
                    && let Some(order) = self.orders.iter_mut().find(|o| {
                        matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. }
                            if *unit == s.id)
                    })
                {
                    *order = Order::SkipUnit { unit: s.id };
                    // The wait needs an end: a troop too far for the two-turn rendezvous
                    // above still walks to the Settler, and the escort steps with it next
                    // turn (8.9.37: a Settler one step from its site skipped 11 turns while
                    // its Spearman stood three turns away). Any free troop that can reach it is
                    // called, nearest first, however far (8.9.40 Byzantium T92: the nearest
                    // troop stood seven hexes off, past `escort_search_distance`).
                    if let Some(u) = self
                        .free(army)
                        .into_iter()
                        .filter(|u| {
                            self.duel_can_escort(u, s, held_home)
                                && self.ctx.reachable(u.pos, s.pos)
                        })
                        .min_by_key(|u| (u.pos.distance(s.pos), u.id))
                    {
                        self.go_to(u, s.pos);
                        self.campaigns.settler_escorts.insert(s.id, u.id);
                    }
                }
                continue;
            };
            if found {
                self.go_to(u, s.pos);
                self.campaigns.settler_escorts.remove(&s.id);
                continue;
            }
            self.campaigns.settler_escorts.insert(s.id, u.id);
            // An escort kept from an earlier threat follows the Settler wherever it stops.
            let keeps_pair = !threatened || paired.is_some_and(|p| p.id == u.id);
            let arrival = self.settler_arrival(s).unwrap_or(s.pos);
            // Gene `snipe_read`: one escort cannot hold the arrival against two or more troops
            // in reach of it; the Settler goes back to a city it reaches this turn instead.
            if self.decisions.snipe_read > 0
                && arrival != s.pos
                && self.troops_in_reach(arrival) >= 2
                && !self.ctx.own_cities.iter().any(|c| c.pos == arrival)
                && self.keep_settler_home(s)
            {
                self.go_to(u, s.pos);
                continue;
            }
            if arrival != s.pos
                && self.free_tile_for(u, arrival)
                && self.reaches_this_turn(u, arrival)
                && (retreat_escort
                    || keeps_pair
                    || self.safe_campaign_tile(arrival)
                        && self
                            .view()
                            .index(arrival)
                            .is_some_and(|i| !self.ctx.settler_danger[i]))
            {
                self.go_to(u, arrival);
            } else {
                // Gene `snipe_read`: the escort is too far behind to reach where the Settler
                // would stop, and something could take it there (an unseen sniper's unit, or a
                // rival in reach next turn): the Settler waits for its escort this turn rather
                // than walk on alone (a Settler two hexes ahead of its escort is taken before
                // the escort arrives). A tile merely occupied keeps the Settler moving.
                if self.decisions.snipe_read > 0
                    && !heading_home
                    && arrival != s.pos
                    && !self.reaches_this_turn(u, arrival)
                    && (fog >= self.decisions.settler_risk_escort || self.snipe_exposed(arrival))
                    && !self.ctx.own_cities.iter().any(|c| c.pos == arrival)
                    && !self.settler_capturable_this_turn(s.pos)
                    && let Some(order) = self.orders.iter_mut().find(|o| {
                        matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. }
                            if *unit == s.id)
                    })
                {
                    *order = Order::SkipUnit { unit: s.id };
                } else if keeps_pair
                    && arrival != s.pos
                    && let Some(order) = self.orders.iter_mut().find(|o| {
                        matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. }
                            if *unit == s.id)
                    })
                {
                    // The pair moves together or not at all.
                    *order = Order::SkipUnit { unit: s.id };
                }
                self.go_to(u, s.pos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder},
    };
    use legatum_core::{Hex, PlayerView, TechId, diplomacy::DiplomaticAction};

    #[test]
    fn hard_named_fund_second_troop_prefers_a_surviving_current_era_escort() {
        let mut v = ViewBuilder::new(24, 16)
            .known_techs(&TechId::ALL)
            .gold(10000)
            .city(1, ME, Hex::new(6, 6), 4)
            .unit(1, ME, UnitKind::Settler, Hex::new(7, 6))
            .unit(2, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
            .build();
        for option in &mut v.cities[0].detail.as_mut().unwrap().options {
            if matches!(option.item, legatum_core::ProductionItem::Unit { .. }) {
                option.buy_cost = Some(2 * option.cost);
            }
        }
        let mut p = planner(&v);
        p.decisions.named_fund = 0;
        let old = p.second_troop_buy();
        let mut p = planner(&v);
        p.decisions.named_fund = 1;
        let new = p
            .second_troop_buy()
            .expect("escort risk")
            .expect("affordable");
        let legatum_core::ProductionItem::Unit { unit: new_unit } = new.1 else {
            panic!("troop")
        };
        assert!(old.is_none_or(|old| old.ok().is_none_or(|buy| buy.1 != new.1)));
        assert!(legatum_core::data::unit_era_tier(new_unit) > 0);
        assert!(new.2 <= v.economy.gold);
    }

    fn planner(v: &PlayerView) -> Planner<'_> {
        let mut p = Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 1);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.settler_safety = 1;
        p.decisions.peace_exposure_percent = 100;
        p.decisions.threat_horizon_turns = 2;
        p
    }

    #[test]
    fn visible_approach_reserves_defense_before_the_army_reaches_firing_range() {
        let v = ViewBuilder::new(24, 16)
            .city(1, ME, Hex::new(6, 6), 3)
            .unit(5, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
            .build();
        let mut p = planner(&v);
        assert_eq!(p.ctx.threat[0], 0);
        p.prepare_hard_defense();
        assert!(p.ctx.threat[0] > 0);
        // The shared context used by the benchmark does not gain this policy.
        assert_eq!(crate::analysis::Ctx::for_proxy(&v).threat[0], 0);
    }

    #[test]
    fn peaceful_raider_changes_settler_safety_only_when_war_is_legal() {
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, Hex::new(6, 6), 3)
            .owner(Hex::new(10, 6), ENEMY)
            .unit(5, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
            .build();
        v.diplomacy.legacy = false;
        let at = v.index(Hex::new(9, 6)).unwrap();
        let against = v.team_of(ENEMY);
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against,
                breach: false,
            },
            effective_turn: v.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        let mut p = planner(&v);
        assert!(!p.ctx.settler_danger[at]);
        p.prepare_hard_defense();
        assert!(p.ctx.settler_danger[at]);
        assert!(
            p.ctx.enemy_units.is_empty(),
            "a neutral is not an attack target"
        );
        assert!(
            p.ctx.threat[0] > 0,
            "one peaceful soldier carries weighted declaration risk"
        );
        let mut legacy = planner(&v);
        legacy.decisions.peace_exposure_percent = 0;
        legacy.prepare_hard_defense();
        assert_eq!(legacy.ctx.threat[0], 0);
        v = ViewBuilder { view: v }
            .unit(6, ENEMY, UnitKind::Archer, Hex::new(10, 5))
            .build();
        let mut p = planner(&v);
        p.prepare_hard_defense();
        assert!(
            p.ctx.threat[0] > 0,
            "an approaching group warrants readiness"
        );
        v = ViewBuilder { view: v }
            .unit(7, ENEMY, UnitKind::Archer, Hex::new(9, 7))
            .build();
        let mut p = planner(&v);
        p.prepare_hard_defense();
        assert!(p.ctx.threat[0] > 0, "a visible army warrants readiness");
        v.diplomacy.actions[0].blocked_reason = Some("Protected treaty".into());
        let mut p = planner(&v);
        p.prepare_hard_defense();
        assert!(!p.ctx.settler_danger[at]);
    }

    #[test]
    fn a_peaceful_scout_endangers_settlers_only_by_gene_or_at_war() {
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, Hex::new(6, 6), 3)
            .owner(Hex::new(10, 6), ENEMY)
            .unit(5, ENEMY, UnitKind::Scout, Hex::new(10, 6))
            .build();
        v.diplomacy.legacy = false;
        let at = v.index(Hex::new(8, 6)).unwrap();
        let against = v.team_of(ENEMY);
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against,
                breach: false,
            },
            effective_turn: v.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        let danger = |v: &PlayerView, scout_gene: i32, capture_gene: i32| {
            let mut p = planner(v);
            p.decisions.peace_scout_danger = scout_gene;
            p.decisions.peace_settler_capture = capture_gene;
            p.prepare_hard_defense();
            p.ctx.settler_danger[at]
        };
        assert!(!danger(&v, 0, 0), "the old policy ignored a Scout at Peace");
        assert!(danger(&v, 1, 0), "the scout gene restores the old caution");
        assert!(
            danger(&v, 0, 100),
            "a cheap surprise war makes the Scout a capture risk"
        );
        // An armed rival at Peace still counts.
        let armed = ViewBuilder { view: v.clone() }
            .unit(6, ENEMY, UnitKind::Spearman, Hex::new(10, 5))
            .build();
        assert!(danger(&armed, 0, 0));
    }

    #[test]
    fn an_army_threatens_the_city_it_stands_nearest() {
        let (near, far) = (Hex::new(6, 6), Hex::new(11, 6));
        let v = ViewBuilder::new(24, 16)
            .city(1, ME, near, 3)
            .city(2, ME, far, 3)
            .unit(5, ENEMY, UnitKind::Spearman, Hex::new(7, 6))
            .unit(6, ENEMY, UnitKind::Archer, Hex::new(7, 7))
            .build();
        let threat = |slack: i32, city: Hex| {
            let mut p = planner(&v);
            p.decisions.threat_aim_slack = slack;
            p.prepare_hard_defense();
            let i = p.ctx.own_cities.iter().position(|c| c.pos == city).unwrap();
            p.ctx.threat[i]
        };
        assert!(threat(1, near) > 0);
        assert_eq!(threat(1, far), 0, "the next city over keeps its defenders");
        assert!(threat(5, far) > 0, "slack 5 counts every city in reach");
    }

    #[test]
    fn defenders_close_in_on_an_attacked_city_when_no_safe_tile_is_left() {
        let home = Hex::new(6, 6);
        let reserve = Hex::new(11, 6);
        let mut b = ViewBuilder::new(24, 16)
            .city(1, ME, home, 3)
            .unit(1, ME, UnitKind::Spearman, home)
            .unit(2, ME, UnitKind::Spearman, reserve);
        for (i, h) in home.neighbors().into_iter().enumerate() {
            b = b.unit(10 + i as u32, ENEMY, UnitKind::Spearman, h);
        }
        let v = b.build();
        let order = |radius: i32| {
            let mut p = planner(&v);
            p.decisions.relieve_radius = radius;
            p.prepare_hard_defense();
            p.plan_military();
            p.orders
                .iter()
                .find(|o| {
                    matches!(o, Order::MoveUnit { unit, .. } | Order::SkipUnit { unit }
                        if *unit == legatum_core::UnitId(2))
                })
                .cloned()
        };
        assert!(
            matches!(order(3), Some(Order::MoveUnit { to, .. }) if to.distance(home) < reserve.distance(home)),
            "{:?}",
            order(3)
        );
        assert!(
            !matches!(order(0), Some(Order::MoveUnit { to, .. }) if to.distance(home) < reserve.distance(home)),
            "without relief the reserve keeps out of reach: {:?}",
            order(0)
        );
    }

    #[test]
    fn free_units_go_for_an_unguarded_enemy_city_at_war() {
        let (home, theirs) = (Hex::new(4, 6), Hex::new(10, 6));
        let v = ViewBuilder::new(24, 16)
            .city(1, ME, home, 3)
            .city(2, ENEMY, theirs, 3)
            .unit(1, ME, UnitKind::Spearman, home)
            .unit(2, ME, UnitKind::Spearman, Hex::new(7, 6))
            .unit(3, ME, UnitKind::Spearman, Hex::new(7, 7))
            .build();
        let closes_in = |radius: i32| {
            let mut p = planner(&v);
            p.decisions.open_city_radius = radius;
            p.prepare_hard_defense();
            let army = p.ctx.own_units.clone();
            p.seize_open_city(&army);
            p.orders.iter().any(|o| {
                matches!(o, Order::MoveUnit { unit, to }
                    if *unit == legatum_core::UnitId(2) && to.distance(theirs) <= 1)
            })
        };
        assert!(closes_in(6));
        assert!(
            !closes_in(0),
            "without the opening the field units stay home"
        );
    }

    #[test]
    fn units_defending_an_attacked_city_are_marked_for_upkeep() {
        let home = Hex::new(6, 6);
        let v = ViewBuilder::new(24, 16)
            .city(1, ME, home, 3)
            .unit(5, ENEMY, UnitKind::Spearman, Hex::new(8, 6))
            .build();
        let mut p = planner(&v);
        assert!(p.defending(Hex::new(10, 6)));
        assert!(
            !p.defending(Hex::new(14, 6)),
            "too far from the attacked city"
        );
        p.decisions.protect_defenders = 0;
        assert!(!p.defending(Hex::new(10, 6)));
    }

    #[test]
    fn a_rush_capable_neighbour_triggers_border_defence_before_it_attacks() {
        use crate::fixture::off;
        let view = |faction: legatum_core::Faction| {
            let mut v = ViewBuilder::new(18, 8)
                .city(0, ME, off(2, 3), 3)
                .city(1, ENEMY, off(9, 3), 3)
                .build();
            v.players
                .iter_mut()
                .find(|p| p.id == ENEMY)
                .unwrap()
                .faction = faction;
            v
        };
        let threat = |faction, prior: i32| {
            let v = view(faction);
            let mut model = crate::opponents::OpponentAssessments::default();
            model.update(&v);
            let mut p = planner(&v);
            p.decisions.faction_threat_prior = prior;
            // The faction prior alone; `rush_walls` (tested below) would also fire here.
            p.decisions.rush_walls = 0;
            p.opponents = Some(&model);
            p.assessed_border_threat(off(2, 3))
        };
        assert!(threat(legatum_core::Faction::Sparta, 1));
        assert!(threat(legatum_core::Faction::Mongols, 1));
        assert!(!threat(legatum_core::Faction::Sparta, 0));
        assert!(
            !threat(legatum_core::Faction::Germany, 1),
            "a slow neighbour at peace leaves us economic"
        );
    }

    #[test]
    fn a_rush_capable_rival_within_reach_brings_walls_first() {
        use crate::fixture::off;
        let threat = |faction: legatum_core::Faction, gene: i32, far: bool| {
            let mut v = ViewBuilder::new(30, 8)
                .city(0, ME, off(2, 3), 3)
                .city(1, ENEMY, off(if far { 28 } else { 9 }, 3), 3)
                .build();
            v.players
                .iter_mut()
                .find(|p| p.id == ENEMY)
                .unwrap()
                .faction = faction;
            let mut p = planner(&v);
            p.decisions.rush_walls = gene;
            p.rush_walls_threat(off(2, 3))
        };
        assert!(threat(legatum_core::Faction::Mongols, 1, false));
        assert!(!threat(legatum_core::Faction::Mongols, 0, false));
        assert!(
            !threat(legatum_core::Faction::Mongols, 1, true),
            "beyond 12 hexes"
        );
        assert!(!threat(legatum_core::Faction::Germany, 1, false));
    }

    #[test]
    fn threatened_garrison_stays_while_field_army_remains_available() {
        let home = Hex::new(6, 6);
        let v = ViewBuilder::new(24, 16)
            .city(1, ME, home, 3)
            .unit(1, ME, UnitKind::Pikeman, home)
            .unit(2, ME, UnitKind::Pikeman, Hex::new(5, 6))
            .unit(5, ENEMY, UnitKind::Spearman, Hex::new(8, 6))
            .unit(6, ENEMY, UnitKind::Archer, Hex::new(9, 6))
            .build();
        let mut p = planner(&v);
        let army = p.ctx.own_units.clone();
        p.reserve_front_guards(&army);
        assert!(p.orders.contains(&Order::SkipUnit {
            unit: legatum_core::UnitId(1)
        }));
        assert_eq!(p.free(&army).len(), 1);
    }

    #[test]
    fn india_keeps_a_linked_city_guard_against_one_arriving_attacker() {
        let home = Hex::new(6, 6);
        let v = ViewBuilder::new(24, 16)
            .faction(legatum_core::Faction::India)
            .city(1, ME, home, 3)
            .city(2, ME, Hex::new(9, 6), 3)
            .unit(1, ME, UnitKind::Pikeman, home)
            .unit(5, ENEMY, UnitKind::Spearman, Hex::new(8, 6))
            .build();
        let mut off = planner(&v);
        off.decisions.india_network_guard = 0;
        let army = off.ctx.own_units.clone();
        off.reserve_front_guards(&army);
        assert!(off.orders.is_empty());

        let mut on = planner(&v);
        on.decisions.india_network_guard = 1;
        let army = on.ctx.own_units.clone();
        on.reserve_front_guards(&army);
        assert!(on.orders.contains(&Order::SkipUnit {
            unit: legatum_core::UnitId(1)
        }));
    }

    #[test]
    fn distant_melee_does_not_reserve_a_guard_before_its_counterattack() {
        let home = Hex::new(6, 6);
        let v = ViewBuilder::new(24, 16)
            .city(1, ME, home, 3)
            .unit(1, ME, UnitKind::Pikeman, home)
            .unit(5, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
            .unit(6, ENEMY, UnitKind::Archer, Hex::new(9, 6))
            .build();
        let mut p = planner(&v);
        let army = p.ctx.own_units.clone();
        p.reserve_front_guards(&army);
        assert!(
            p.orders.is_empty(),
            "distant infantry cannot exploit a sortie this turn"
        );
    }

    #[test]
    fn hypothetical_declaration_still_respects_known_impassable_terrain() {
        let pos = Hex::new(10, 6);
        let mut v = ViewBuilder::new(24, 16)
            .unit(5, ENEMY, UnitKind::Spearman, pos)
            .build();
        for h in pos.neighbors() {
            let i = v.index(h).unwrap();
            v.tiles[i].terrain = Some(legatum_core::Terrain::Mountains);
        }
        assert_eq!(declaration_routes(&v, &v.units[0]), BTreeSet::from([pos]));
    }

    #[test]
    fn a_quiet_city_garrison_escorts_a_settler_a_peaceful_scout_could_take() {
        let home = Hex::new(6, 6);
        let at = Hex::new(7, 6);
        let v = ViewBuilder::new(28, 20)
            .city(1, ME, home, 3)
            .unit(1, ME, UnitKind::Settler, at)
            .unit(2, ME, UnitKind::Spearman, home)
            .unit(50, ENEMY, UnitKind::Scout, Hex::new(11, 6))
            .build();
        for gene in [0, 1] {
            let mut p = planner(&v);
            p.decisions.settler_safety = 0;
            p.decisions.settler_escort = gene;
            p.unit_order(
                legatum_core::UnitId(1),
                Order::MoveUnit {
                    unit: legatum_core::UnitId(1),
                    to: Hex::new(8, 6),
                },
            );
            let army = vec![p.ctx.own_units[1]];
            p.guard_exposed_settlers(&army);
            let escorted = p.orders.iter().any(|order| {
                legatum_core::rules::units::order_unit(order) == Some(legatum_core::UnitId(2))
            });
            assert_eq!(escorted, gene == 1, "gene {gene}");
        }
    }

    /// ME may declare a surprise war on ENEMY, and so (in these fixtures) the other way.
    fn declarable(mut v: PlayerView) -> PlayerView {
        v.diplomacy.legacy = false;
        let team = v.team_of(ENEMY);
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against: team,
                breach: false,
            },
            effective_turn: v.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        v
    }

    /// Gene `settler_escort` 2 (8.9.24 Swiss review, the Mongol Settler past Thun): a route two
    /// hexes from a rival city calls for an escort with no rival unit in sight; at 1 it does not.
    #[test]
    fn a_route_beside_a_rival_city_takes_an_escort_without_a_visible_unit() {
        let home = Hex::new(6, 6);
        let v = declarable(
            ViewBuilder::new(28, 20)
                .city(1, ME, home, 3)
                .city(2, ENEMY, Hex::new(12, 6), 3)
                .unit(1, ME, UnitKind::Settler, Hex::new(7, 6))
                .unit(2, ME, UnitKind::Spearman, home)
                .build(),
        );
        for gene in [1, 2] {
            let mut p = planner(&v);
            p.decisions.settler_safety = 0;
            p.decisions.settler_escort = gene;
            p.unit_order(
                legatum_core::UnitId(1),
                Order::MoveUnit {
                    unit: legatum_core::UnitId(1),
                    to: Hex::new(10, 6),
                },
            );
            let army = vec![p.ctx.own_units[1]];
            p.guard_exposed_settlers(&army);
            let escorted = p.orders.iter().any(|order| {
                legatum_core::rules::units::order_unit(order) == Some(legatum_core::UnitId(2))
            });
            assert_eq!(escorted, gene == 2, "gene {gene}");
        }
    }

    /// Gene `settler_escort` 2: with no escort in reach this turn but one two turns away, an
    /// unseen threat sends the Settler back to its city (it no longer waits on open ground).
    #[test]
    fn a_settler_with_an_escort_two_turns_away_goes_home_under_an_unseen_threat() {
        let home = Hex::new(4, 6);
        let at = Hex::new(8, 6);
        let v = declarable(
            ViewBuilder::new(28, 20)
                .city(1, ME, home, 3)
                .city(2, ENEMY, Hex::new(14, 6), 3)
                .unit(1, ME, UnitKind::Settler, at)
                .unit(2, ME, UnitKind::Spearman, home)
                .build(),
        );
        for gene in [1, 2] {
            let mut p = planner(&v);
            p.decisions.settler_safety = 0;
            p.decisions.settler_escort = gene;
            p.unit_order(
                legatum_core::UnitId(1),
                Order::MoveUnit {
                    unit: legatum_core::UnitId(1),
                    to: Hex::new(12, 6),
                },
            );
            let army = vec![p.ctx.own_units[1]];
            p.guard_exposed_settlers(&army);
            // Gene 2 reads the rival city beside the route as an unseen threat. The Settler
            // does not wait for the escort on open ground: it goes back to its city.
            let home_order = Order::MoveUnit {
                unit: legatum_core::UnitId(1),
                to: home,
            };
            let goes_home = p.orders.contains(&home_order);
            let waits = p.orders.iter().any(|order| {
                matches!(order, Order::SkipUnit { unit } if *unit == legatum_core::UnitId(1))
            });
            assert_eq!((goes_home, waits), (gene == 2, false), "gene {gene}");
        }
    }

    /// 8.9.37 (his Byzantium game T69-T79): a Settler one step from its site, a peaceful
    /// Cataphract that could declare three hexes from the site, its Spearman three turns away in a city.
    /// The Settler waits for an escort, so the escort must be on its way; before, it was too
    /// far for the two-turn rendezvous and the Settler skipped 11 turns.
    #[test]
    fn a_waiting_settler_calls_an_escort_more_than_two_turns_away() {
        let site = Hex::new(11, 19);
        let settler = Hex::new(12, 18);
        let v = declarable(
            ViewBuilder::new(30, 30)
                .city(1, ME, Hex::new(12, 22), 3)
                .city(3, ME, Hex::new(8, 25), 3)
                .unit(1, ME, UnitKind::Settler, settler)
                .unit(2, ME, UnitKind::Spearman, Hex::new(12, 22))
                .unit(50, ENEMY, UnitKind::Cataphract, Hex::new(9, 24))
                .build(),
        );
        let mut p = planner(&v);
        p.decisions.settler_safety = 0;
        p.decisions.capitals = 0;
        p.prepare_hard_defense();
        p.unit_order(
            legatum_core::UnitId(1),
            Order::MoveUnitPath {
                unit: legatum_core::UnitId(1),
                to: site,
                path: vec![site],
                stop_on_block: true,
            },
        );
        let army = vec![p.ctx.own_units[1]];
        p.guard_exposed_settlers(&army);
        let order_of = |p: &Planner<'_>, id| {
            p.orders
                .iter()
                .find(|o| {
                    legatum_core::rules::units::order_unit(o) == Some(legatum_core::UnitId(id))
                })
                .cloned()
        };
        assert_eq!(
            order_of(&p, 1),
            Some(Order::SkipUnit {
                unit: legatum_core::UnitId(1)
            }),
            "{:?}",
            p.orders
        );
        let walks = matches!(
            order_of(&p, 2),
            Some(Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }) if to == settler
        );
        assert!(
            walks,
            "the Spearman must walk to the Settler: {:?}",
            p.orders
        );
    }

    #[test]
    fn settler_moving_away_from_a_distant_threat_does_not_take_an_escort() {
        let at = Hex::new(8, 6);
        let v = ViewBuilder::new(28, 20)
            .city(1, ME, Hex::new(4, 4), 3)
            .unit(1, ME, UnitKind::Settler, at)
            .unit(2, ME, UnitKind::Spearman, at)
            .unit(50, ENEMY, UnitKind::Spearman, Hex::new(12, 6))
            .build();
        let mut p = planner(&v);
        p.unit_order(
            legatum_core::UnitId(1),
            Order::MoveUnit {
                unit: legatum_core::UnitId(1),
                to: Hex::new(6, 6),
            },
        );
        let army = vec![p.ctx.own_units[1]];
        p.guard_exposed_settlers(&army);
        assert!(p.orders.contains(&Order::MoveUnit {
            unit: legatum_core::UnitId(1),
            to: Hex::new(6, 6)
        }));
        assert!(!p.orders.iter().any(|order| {
            legatum_core::rules::units::order_unit(order) == Some(legatum_core::UnitId(2))
        }));
    }
    #[test]
    fn occupied_escort_rendezvous_does_not_cancel_a_settlers_safe_path() {
        let home = Hex::new(6, 6);
        let v = ViewBuilder::new(28, 20)
            .city(1, ME, home, 3)
            .unit(1, ME, UnitKind::Settler, home)
            .unit(2, ME, UnitKind::Spearman, home)
            .unit(3, ME, UnitKind::Spearman, Hex::new(5, 6))
            .unit(50, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
            .build();
        let mut p = planner(&v);
        let movement = Order::MoveUnit {
            unit: legatum_core::UnitId(1),
            to: Hex::new(3, 6),
        };
        p.unit_order(legatum_core::UnitId(1), movement.clone());
        let army = vec![p.ctx.own_units[1], p.ctx.own_units[2]];
        p.guard_exposed_settlers(&army);
        assert!(p.orders.contains(&movement));
        assert!(!p.orders.contains(&Order::MoveUnit {
            unit: legatum_core::UnitId(3),
            to: home,
        }));
    }

    #[test]
    fn a_same_turn_surprise_war_makes_the_settler_wait_when_no_escort_can_follow() {
        let from = Hex::new(23, 10);
        let destination = Hex::new(20, 14);
        let mut v = ViewBuilder::new(28, 20)
            .faction(legatum_core::Faction::Sparta)
            .city(1, ME, Hex::new(11, 10), 3)
            .unit(1, ME, UnitKind::Settler, from)
            .unit(50, ENEMY, UnitKind::Scout, Hex::new(19, 11))
            .build();
        v.turn = 27;
        v.diplomacy.legacy = false;
        let enemy_team = v.team_of(ENEMY);
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against: enemy_team,
                breach: false,
            },
            effective_turn: v.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });

        let mut p = planner(&v);
        p.unit_order(
            legatum_core::UnitId(1),
            Order::MoveUnitPath {
                unit: legatum_core::UnitId(1),
                path: vec![
                    Hex::new(22, 11),
                    Hex::new(21, 12),
                    Hex::new(20, 13),
                    destination,
                ],
                to: destination,
                stop_on_block: true,
            },
        );
        let arrival = p.settler_arrival(p.ctx.own_units[0]).unwrap();
        assert!(from.distance(Hex::new(19, 11)) > 3);
        assert_eq!(arrival, Hex::new(21, 12));
        assert!(p.settler_capturable_this_turn(arrival));
        assert!(p.snipe_exposed(arrival));
        assert!(!p.snipe_exposed(from));
        assert!(p.safe_campaign_tile(from));

        p.guard_exposed_settlers(&[]);

        assert!(p.orders.contains(&Order::SkipUnit {
            unit: legatum_core::UnitId(1),
        }));
    }

    #[test]
    fn settler_recalls_toward_home_when_a_raider_reaches_its_current_tile() {
        let from = Hex::new(8, 6);
        let v = ViewBuilder::new(28, 20)
            .city(1, ME, Hex::new(4, 6), 3)
            .unit(1, ME, UnitKind::Settler, from)
            .unit(50, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
            .build();
        let mut p = planner(&v);
        p.unit_order(
            legatum_core::UnitId(1),
            Order::MoveUnit {
                unit: legatum_core::UnitId(1),
                to: Hex::new(12, 6),
            },
        );
        assert!(p.settler_capturable_this_turn(from));
        p.guard_exposed_settlers(&[]);
        assert!(
            p.orders.iter().any(|o| matches!(o,
                Order::MoveUnitPath { unit: legatum_core::UnitId(1), to, stop_on_block: true, .. }
                    if *to == Hex::new(6, 6)
            )),
            "{:?}",
            p.orders
        );
    }

    #[test]
    fn settler_founds_before_a_distant_scout_can_capture_it() {
        let from = Hex::new(7, 6);
        let mut v = ViewBuilder::new(28, 20)
            .city(1, ME, Hex::new(4, 6), 3)
            .unit(1, ME, UnitKind::Settler, from)
            .unit(50, ENEMY, UnitKind::Scout, Hex::new(11, 6))
            .build();
        v.diplomacy.legacy = false;
        let enemy_team = v.team_of(ENEMY);
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against: enemy_team,
                breach: false,
            },
            effective_turn: v.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        let mut p = planner(&v);
        p.unit_order(
            legatum_core::UnitId(1),
            Order::FoundCity {
                unit: legatum_core::UnitId(1),
            },
        );
        assert!(!p.settler_capturable_this_turn(from));
        assert!(p.snipe_exposed(from));
        p.guard_exposed_settlers(&[]);
        assert!(p.orders.contains(&Order::FoundCity {
            unit: legatum_core::UnitId(1),
        }));
    }

    #[test]
    fn road_start_extends_visible_scout_capture_reach() {
        let scout = Hex::new(11, 6);
        let settler = Hex::new(7, 6);
        let mut v = ViewBuilder::new(28, 20)
            .city(1, ME, Hex::new(4, 6), 3)
            .unit(1, ME, UnitKind::Settler, settler)
            .unit(50, ENEMY, UnitKind::Scout, scout)
            .build();
        let at = v.index(scout).unwrap();
        v.tiles[at].road = 1;
        let p = planner(&v);
        assert_eq!(
            p.turn_moves(v.units.iter().find(|u| u.id.0 == 50).unwrap()),
            4
        );
        assert!(p.settler_capturable_this_turn(settler));
    }

    #[test]
    fn capital_guard_keeps_settler_safe_near_a_peaceful_scout() {
        let home = Hex::new(8, 6);
        for kind in [UnitKind::Spearman, UnitKind::Archer] {
            let mut v = ViewBuilder::new(24, 16)
                .faction(legatum_core::Faction::Mongols)
                .city(1, ME, home, 3)
                .unit(1, ME, UnitKind::Settler, home)
                .unit(2, ME, kind, home)
                .unit(50, ENEMY, UnitKind::Scout, Hex::new(10, 6))
                .build();
            v.diplomacy.legacy = false;
            let enemy_team = v.team_of(ENEMY);
            v.diplomacy.actions.push(DiplomaticAction {
                new_wars: Vec::new(),
                calls: Vec::new(),
                also_agree: Vec::new(),
                order: Order::DeclareWar {
                    against: enemy_team,
                    breach: false,
                },
                effective_turn: v.turn,
                cost: 0,
                blocked_reason: None,
                upkeep: 0,
                betrayal: false,
            });
            let mut p = planner(&v);
            p.prepare_hard_defense();
            let settler = (*p.ctx.own_units.iter().find(|u| u.id.0 == 1).unwrap()).clone();
            let guard = (*p.ctx.own_units.iter().find(|u| u.id.0 == 2).unwrap()).clone();
            p.move_settler(&settler, Hex::new(4, 6));
            p.guard_exposed_settlers(&[&guard]);
            match p.settler_arrival(&settler) {
                Some(arrival) => {
                    assert_eq!(p.settler_arrival(&guard), Some(arrival), "{kind:?}");
                }
                None => {
                    assert_eq!(settler.pos, home, "{kind:?}");
                    assert_eq!(guard.pos, home, "{kind:?}");
                    assert_eq!(p.settler_arrival(&guard), None, "{kind:?}");
                }
            }
        }
    }

    #[test]
    fn settler_advances_safely_then_replans_before_a_distant_scout() {
        let from = Hex::new(4, 6);
        let to = Hex::new(14, 6);
        let v = ViewBuilder::new(24, 16)
            .city(1, ME, Hex::new(2, 6), 3)
            .unit(1, ME, UnitKind::Settler, from)
            .unit(50, ENEMY, UnitKind::Scout, Hex::new(9, 8))
            .build();
        let mut p = planner(&v);
        let settler = p.ctx.own_units[0];
        p.move_settler(settler, to);
        let path = p
            .orders
            .iter()
            .find_map(|o| match o {
                Order::MoveUnitPath { unit, path, .. } if *unit == settler.id => Some(path.clone()),
                Order::MoveUnit { unit, to } if *unit == settler.id => {
                    v.movement_path(ME, settler.kind, from, *to)
                }
                _ => None,
            })
            .expect("detour remains reachable");
        let mut moves = p.turn_moves(settler);
        for h in &path {
            let cost = p.ctx.move_cost(*h).unwrap();
            if !can_step(moves, p.turn_moves(settler), cost) {
                break;
            }
            assert!(!p.snipe_exposed(*h), "{:?}", p.orders);
            moves -= cost;
        }
        assert!(path.iter().any(|h| p.snipe_exposed(*h)));
    }

    #[test]
    fn holding_or_founding_settler_reacts_to_a_new_raider() {
        let from = Hex::new(8, 6);
        let v = ViewBuilder::new(28, 20)
            .city(1, ME, Hex::new(4, 6), 3)
            .unit(1, ME, UnitKind::Settler, from)
            .unit(50, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
            .build();
        for order in [
            Order::SkipUnit {
                unit: legatum_core::UnitId(1),
            },
            Order::FoundCity {
                unit: legatum_core::UnitId(1),
            },
        ] {
            let mut p = planner(&v);
            let policy =
                crate::openings::OpeningPolicy::builtin(crate::openings::Opening::GreedyExpand);
            p.opening = Some(crate::openings::OpeningPlan {
                policy: &policy,
                phase: crate::openings::Phase::Opening,
                next_build: None,
                investment: None,
            });
            assert!(!p.hard_defense());
            assert!(p.settler_safety());
            p.unit_order(legatum_core::UnitId(1), order);
            p.guard_exposed_settlers(&[]);
            assert!(
                p.orders.iter().any(|o| matches!(o,
                    Order::MoveUnitPath { unit: legatum_core::UnitId(1), to, .. }
                        if *to == Hex::new(6, 6)
                )),
                "{:?}",
                p.orders
            );
        }
    }

    #[test]
    fn sixteen_seeded_surprise_war_trips_avoid_immediate_capture() {
        for seed in 0..16 {
            let from = Hex::new(19 + seed % 4, 10 + seed / 4);
            let scout = Hex::new(from.q - 4, from.r);
            let destination = Hex::new(from.q - 3, from.r);
            let mut v = ViewBuilder::new(28, 20)
                .faction(legatum_core::Faction::Sparta)
                .city(1, ME, Hex::new(6, 6), 3)
                .unit(1, ME, UnitKind::Settler, from)
                .unit(50, ENEMY, UnitKind::Scout, scout)
                .build();
            v.turn = 27;
            v.diplomacy.legacy = false;
            let enemy_team = v.team_of(ENEMY);
            v.diplomacy.actions.push(DiplomaticAction {
                new_wars: Vec::new(),
                calls: Vec::new(),
                also_agree: Vec::new(),
                order: Order::DeclareWar {
                    against: enemy_team,
                    breach: false,
                },
                effective_turn: v.turn,
                cost: 0,
                blocked_reason: None,
                upkeep: 0,
                betrayal: false,
            });

            let mut p = planner(&v);
            p.decisions.peace_settler_capture = 100;
            p.unit_order(
                legatum_core::UnitId(1),
                Order::MoveUnitPath {
                    unit: legatum_core::UnitId(1),
                    path: vec![
                        Hex::new(from.q - 1, from.r),
                        Hex::new(from.q - 2, from.r),
                        destination,
                    ],
                    to: destination,
                    stop_on_block: true,
                },
            );
            assert!(
                p.settler_trip_risk(from, destination) >= p.decisions.settler_risk_escort,
                "seed {seed}"
            );
            assert!(!p.settler_capturable_this_turn(from), "seed {seed}");
            p.guard_exposed_settlers(&[]);
            let arrival = p.settler_arrival(p.ctx.own_units[0]).unwrap_or(from);
            assert!(
                !p.settler_capturable_this_turn(arrival),
                "seed {seed}: {:?}",
                p.orders
            );
            assert_ne!(arrival, destination, "seed {seed}");
        }
    }
    fn order_of(p: &Planner<'_>, id: u32) -> Option<Order> {
        p.orders
            .iter()
            .find(|o| legatum_core::rules::units::order_unit(o) == Some(legatum_core::UnitId(id)))
            .cloned()
    }

    /// A rival with a recent surprise war on a third empire, one unseen tile beside the route,
    /// no unit of it in sight: the 8.9.40 Byzantium T16 position (a fog-only threat).
    fn fog_threat(mut v: PlayerView, unseen: Hex) -> PlayerView {
        let (a, b) = (v.team_of(ENEMY), v.team_of(legatum_core::PlayerId(3)));
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: a.min(b),
                b: a.max(b),
                state: legatum_core::diplomacy::RelationState::War,
                effective_turn: v.turn,
                guarantee_until: 0,
                version: 0,
                security: None,
                war_id: Some(1),
                declarers: vec![a],
            });
        let i = v.index(unseen).unwrap();
        v.tiles[i].visible = false;
        v
    }

    /// Orders Settler 1 to `to`: east along its row one step per hex, else straight to it.
    fn settler_trip(p: &mut Planner<'_>, to: Hex) {
        let from = p.ctx.own_units[0].pos;
        p.unit_order(
            legatum_core::UnitId(1),
            Order::MoveUnitPath {
                unit: legatum_core::UnitId(1),
                to,
                path: if to.q > from.q {
                    (from.q + 1..=to.q).map(|q| Hex::new(q, from.r)).collect()
                } else {
                    vec![to]
                },
                stop_on_block: true,
            },
        );
    }

    /// 8.9.40 Byzantium T16: a fog-only threat made the Settler wait in the open, two tiles from
    /// its city, for an escort three tiles away; his Horseman stood in the fog. It goes back to
    /// the city instead and never waits on open ground alone.
    #[test]
    fn a_fog_threat_never_makes_a_settler_wait_in_the_open() {
        let home = Hex::new(4, 6);
        let at = Hex::new(8, 6);
        let v = fog_threat(
            declarable(
                ViewBuilder::new(28, 20)
                    .city(1, ME, home, 3)
                    .unit(1, ME, UnitKind::Settler, at)
                    .unit(2, ME, UnitKind::Spearman, home)
                    .build(),
            ),
            Hex::new(10, 5),
        );
        let mut p = planner(&v);
        p.decisions.settler_safety = 0;
        p.prepare_hard_defense();
        settler_trip(&mut p, Hex::new(12, 6));
        assert!(p.snipe_fog_risk(at, Hex::new(12, 6)) >= p.decisions.settler_risk_escort);
        let army = vec![p.ctx.own_units[1]];
        p.guard_exposed_settlers(&army);
        assert_eq!(
            order_of(&p, 1),
            Some(Order::MoveUnit {
                unit: legatum_core::UnitId(1),
                to: home
            }),
            "{:?}",
            p.orders
        );
    }

    /// Inside one of our cities the same Settler may wait for its escort.
    #[test]
    fn a_fog_threat_lets_a_settler_wait_inside_a_city() {
        let home = Hex::new(4, 6);
        let v = fog_threat(
            declarable(
                ViewBuilder::new(28, 20)
                    .city(1, ME, home, 3)
                    .city(3, ME, Hex::new(1, 6), 3)
                    .unit(1, ME, UnitKind::Settler, home)
                    .unit(2, ME, UnitKind::Spearman, Hex::new(1, 6))
                    .build(),
            ),
            Hex::new(8, 5),
        );
        let mut p = planner(&v);
        p.decisions.settler_safety = 0;
        p.prepare_hard_defense();
        settler_trip(&mut p, Hex::new(10, 6));
        let army = vec![p.ctx.own_units[1]];
        p.guard_exposed_settlers(&army);
        assert_eq!(
            order_of(&p, 1),
            Some(Order::SkipUnit {
                unit: legatum_core::UnitId(1)
            }),
            "{:?}",
            p.orders
        );
    }

    /// 8.9.40 Byzantium T89: the escort called to a Settler was released the turn nothing was
    /// in sight. A kept escort walks to where the Settler stops, never to the tile it leaves.
    #[test]
    fn an_escort_stays_with_its_settler_when_the_threat_read_falls_to_zero() {
        let home = Hex::new(6, 6);
        let v = ViewBuilder::new(28, 20)
            .city(1, ME, home, 3)
            .unit(1, ME, UnitKind::Settler, home)
            .unit(2, ME, UnitKind::Spearman, Hex::new(7, 6))
            .build();
        let mut p = planner(&v);
        p.decisions.settler_safety = 0;
        p.campaigns
            .settler_escorts
            .insert(legatum_core::UnitId(1), legatum_core::UnitId(2));
        settler_trip(&mut p, Hex::new(10, 6));
        let arrival = p.settler_arrival(p.ctx.own_units[0]).unwrap();
        assert_ne!(arrival, home);
        let army = vec![p.ctx.own_units[1]];
        p.guard_exposed_settlers(&army);
        assert!(matches!(
            order_of(&p, 2),
            Some(Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }) if to == arrival
        ));
        assert_eq!(
            p.campaigns.settler_escorts.get(&legatum_core::UnitId(1)),
            Some(&legatum_core::UnitId(2))
        );
    }

    /// An escort one step too far behind to keep up: the Settler waits in its city and the
    /// escort steps onto the city's tile; they leave together next turn.
    #[test]
    fn a_kept_escort_that_cannot_keep_up_holds_the_settler_in_its_city() {
        let home = Hex::new(6, 6);
        let v = ViewBuilder::new(28, 20)
            .city(1, ME, home, 3)
            .unit(1, ME, UnitKind::Settler, home)
            .unit(2, ME, UnitKind::Spearman, Hex::new(4, 6))
            .build();
        let mut p = planner(&v);
        p.decisions.settler_safety = 0;
        p.campaigns
            .settler_escorts
            .insert(legatum_core::UnitId(1), legatum_core::UnitId(2));
        settler_trip(&mut p, Hex::new(10, 6));
        let army = vec![p.ctx.own_units[1]];
        p.guard_exposed_settlers(&army);
        assert_eq!(
            order_of(&p, 1),
            Some(Order::SkipUnit {
                unit: legatum_core::UnitId(1)
            })
        );
        assert!(matches!(
            order_of(&p, 2),
            Some(Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }) if to == home
        ));
    }

    /// The pair ends when the Settler is back in a city with no trip, or has founded.
    #[test]
    fn an_escort_is_released_when_its_settler_is_home_or_founds() {
        let home = Hex::new(6, 6);
        let v = ViewBuilder::new(28, 20)
            .city(1, ME, home, 3)
            .unit(1, ME, UnitKind::Settler, home)
            .unit(2, ME, UnitKind::Spearman, Hex::new(5, 6))
            .build();
        let mut p = planner(&v);
        p.decisions.settler_safety = 0;
        p.campaigns
            .settler_escorts
            .insert(legatum_core::UnitId(1), legatum_core::UnitId(2));
        let army = vec![p.ctx.own_units[1]];
        p.guard_exposed_settlers(&army);
        assert!(p.campaigns.settler_escorts.is_empty());
        assert_eq!(order_of(&p, 2), None);

        let mut p = planner(&v);
        p.decisions.settler_safety = 0;
        p.campaigns
            .settler_escorts
            .insert(legatum_core::UnitId(1), legatum_core::UnitId(2));
        p.unit_order(
            legatum_core::UnitId(1),
            Order::FoundCity {
                unit: legatum_core::UnitId(1),
            },
        );
        p.guard_exposed_settlers(&army);
        assert!(p.campaigns.settler_escorts.is_empty());
    }

    /// 8.9.40 Byzantium T92 to T94: the nearest troop stood seven hexes off, past
    /// `escort_search_distance` 6, so the waiting Settler called nobody. It calls the nearest
    /// free troop that can reach it, however far.
    #[test]
    fn a_waiting_settler_calls_an_escort_beyond_the_search_distance() {
        let site = Hex::new(11, 19);
        let settler = Hex::new(12, 18);
        let v = declarable(
            ViewBuilder::new(30, 30)
                .city(1, ME, Hex::new(12, 28), 3)
                .city(3, ME, Hex::new(8, 25), 3)
                .unit(1, ME, UnitKind::Settler, settler)
                .unit(2, ME, UnitKind::Spearman, Hex::new(12, 28))
                .unit(50, ENEMY, UnitKind::Cataphract, Hex::new(9, 24))
                .build(),
        );
        let mut p = planner(&v);
        p.decisions.settler_safety = 0;
        p.prepare_hard_defense();
        settler_trip(&mut p, site);
        assert!(Hex::new(12, 28).distance(settler) > p.decisions.escort_search_distance);
        let army = vec![p.ctx.own_units[1]];
        p.guard_exposed_settlers(&army);
        assert_eq!(
            order_of(&p, 1),
            Some(Order::SkipUnit {
                unit: legatum_core::UnitId(1)
            }),
            "{:?}",
            p.orders
        );
        assert!(matches!(
            order_of(&p, 2),
            Some(Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }) if to == settler
        ));
    }

    /// 8.9.40 Byzantium T96: he took Athens's Settler on T17 and Athens forgot after 20 turns.
    /// A rival that took one of our Settlers stays a known sniper, read from our own view only.
    #[test]
    fn a_rival_that_took_our_settler_stays_a_known_sniper() {
        let base = |turn: u32| {
            let mut v = ViewBuilder::new(28, 20)
                .city(1, ME, Hex::new(6, 6), 3)
                .unit(1, ME, UnitKind::Settler, Hex::new(8, 6))
                .build();
            v.turn = turn;
            v
        };
        let mut assessments = crate::opponents::OpponentAssessments::default();
        assessments.update(&base(16));
        let mut taken = base(17);
        taken.units[0].owner = ENEMY;
        assessments.update(&taken);
        let mut late = base(120);
        late.units.clear();
        let p = Planner::new(
            &late,
            Knobs::for_faction(late.faction_of(ME)),
            late.faction_of(ME),
            1,
        );
        assert!(!p.recent_surprise_war(ENEMY));
        let mut p = p;
        p.opponents = Some(&assessments);
        assert!(p.recent_surprise_war(ENEMY));
        assert!(!p.recent_surprise_war(legatum_core::PlayerId(3)));
    }

    /// Gene `roman_found_first`: the peace-time supported-defence stand does not recall the
    /// Roman veteran founding a city this turn (rome-founders trace, seed 930027 T44: staging
    /// replaced its FoundCity with a step back beside the capital).
    #[test]
    fn speculative_defence_leaves_a_roman_foundation_alone() {
        let site = Hex::new(10, 8);
        let mut v = ViewBuilder::new(28, 16)
            .faction(legatum_core::Faction::Rome)
            .city(1, ME, Hex::new(6, 8), 3)
            .unit(1, ME, UnitKind::Spearman, site)
            .unit(2, ME, UnitKind::Spearman, Hex::new(6, 8))
            .unit(3, ENEMY, UnitKind::Spearman, Hex::new(24, 8))
            .gold(0)
            .build();
        v.units[0].level = 1;
        let city = v.cities[0].clone();
        let enemy = v.units[2].clone();
        let run = |found_first: i32| {
            let mut p = planner(&v);
            p.decisions.roman_founders = 1;
            p.decisions.roman_found_first = found_first;
            p.decisions.settler_min_site_score = 1000;
            p.plan_settlers();
            assert!(
                p.orders.contains(&Order::FoundCity { unit: UnitId(1) }),
                "{:?}",
                p.orders
            );
            p.stage_supported_defence(&city, &[&enemy]);
            p.orders.contains(&Order::FoundCity { unit: UnitId(1) })
        };
        assert!(!run(0), "the control arm reproduces the recall");
        assert!(run(1));
    }
}

//! Commit a local army before individual recovery can dissolve a viable city assault.
//! Forecasts use the observed defenders, never hidden orders or future reinforcements.

use super::*;
use legatum_core::{PlayerView, UnitId};
use std::collections::BTreeMap;

impl Planner<'_> {
    pub(super) fn coordinate_city_assaults(&mut self, army: &[&UnitView]) {
        self.coordinate_city_assaults_at(army, None);
    }

    /// The assault planner for every visible enemy city, or only the city at `only`.
    pub(super) fn coordinate_city_assaults_at(&mut self, army: &[&UnitView], only: Option<Hex>) {
        self.coordinate_city_assaults_on(army, only, false);
    }

    /// As `coordinate_city_assaults_at`; `any_campaign` also assaults a city that is not the
    /// running campaign's target (gene `finish_fallen`: a broken city next to our units).
    pub(super) fn coordinate_city_assaults_on(
        &mut self,
        army: &[&UnitView],
        only: Option<Hex>,
        any_campaign: bool,
    ) {
        if self.takes_no_cities() {
            return;
        }
        let mut targets: Vec<_> = self
            .targets(false)
            .into_iter()
            .filter(|t| only.is_none_or(|pos| t.pos == pos))
            .filter(|t| {
                any_campaign
                    || self.campaigns.active.as_ref().is_none_or(|campaign| {
                        self.ctx
                            .enemy_cities
                            .iter()
                            .any(|c| c.pos == t.pos && c.id == campaign.target)
                    })
            })
            .filter(|t| {
                self.ctx
                    .enemy_cities
                    .iter()
                    .any(|c| c.pos == t.pos && c.visible)
            })
            .collect();
        targets.sort_by_key(|t| (t.their, t.pos));
        for target in targets {
            if self.mil_dest.contains(&target.pos) {
                continue;
            }
            let city = self
                .ctx
                .enemy_cities
                .iter()
                .find(|c| c.pos == target.pos)
                .unwrap();
            let garrison = self
                .ctx
                .enemy_units
                .iter()
                .any(|u| u.military && u.pos == target.pos);
            // Gene `relief_forecast`: a lone walker only takes the empty fallen city when no
            // owner's unit stands by to step in first; otherwise the full forecast decides.
            let relief = self.decisions.relief_forecast > 0
                && !crate::opportunity::relief_units(self.view(), target.pos).is_empty();
            if city.hp == 0 && !garrison && !relief {
                if let Some(u) = self
                    .free(army)
                    .into_iter()
                    .filter(|u| {
                        (!self.hard_defense() || crate::composition::charges(u.kind))
                            && self.reaches_this_turn(u, target.pos)
                    })
                    .filter(|u| {
                        self.melee_forecast(u, target.pos)
                            .is_some_and(|f| f.loss_chance < 25)
                    })
                    .min_by_key(|u| {
                        (
                            crate::composition::shooter_only(u.kind, u.ranged),
                            u.pos.distance(target.pos),
                            std::cmp::Reverse(u.hp),
                            u.id,
                        )
                    })
                {
                    self.unit_order(
                        u.id,
                        Order::MoveUnit {
                            unit: u.id,
                            to: target.pos,
                        },
                    );
                    self.mil_dest.insert(target.pos);
                }
                continue;
            }
            let mut near: Vec<_> = self
                .free(army)
                .into_iter()
                .filter(|u| u.pos.distance(target.pos) <= STRIKE_RADIUS + 1)
                .filter(|u| self.ctx.reachable(u.pos, target.pos))
                .filter(|u| self.rushing() || !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
                .collect();
            // The Keshig is the melee wing here, as the Horseman it replaces: it can take the city.
            near.sort_by_key(|u| {
                (
                    crate::composition::shooter_only(u.kind, u.ranged),
                    u.pos.distance(target.pos),
                    u.id,
                )
            });
            if !near.iter().any(|u| crate::composition::charges(u.kind)) {
                continue;
            }
            let mut reserved = self.mil_dest.clone();
            let mut arrivals = Vec::new();
            for &u in &near {
                let bow_siege = self.mongol_horde()
                    && self.decisions.mongol_horde >= 3
                    && city.walls
                    && city.hp > 0
                    && crate::composition::dual(u.kind, u.ranged);
                let firing_tile = bow_siege
                    .then(|| {
                        target
                            .pos
                            .ring(2)
                            .into_iter()
                            .filter(|h| {
                                !reserved.contains(h)
                                    && (*h == u.pos || self.free_tile_for(u, *h))
                                    && (*h == u.pos || self.reaches_this_turn(u, *h))
                                    && self.ranged_unit_clear(u, *h, target.pos)
                                    && self.outside_remembered_reach(*h)
                            })
                            .min_by_key(|h| (self.enemy_exposure_key(*h), h.distance(u.pos), *h))
                    })
                    .flatten();
                let to = if let Some(tile) = firing_tile {
                    Some(tile)
                } else if bow_siege {
                    None
                } else if crate::composition::charges(u.kind)
                    && self.reaches_this_turn(u, target.pos)
                {
                    Some(target.pos)
                } else if crate::composition::shooter_only(u.kind, u.ranged)
                    && u.pos.distance(target.pos) <= u.range
                    && self.ranged_unit_clear(u, u.pos, target.pos)
                {
                    Some(u.pos)
                } else {
                    let radius = if crate::composition::shooter_only(u.kind, u.ranged) {
                        u.range
                    } else {
                        1
                    };
                    let outer = if self.hard_defense() && crate::composition::charges(u.kind) {
                        2
                    } else {
                        radius
                    };
                    let staged = (radius..=outer)
                        .flat_map(|r| target.pos.ring(r))
                        .filter(|h| {
                            !reserved.contains(h)
                                && self.free_tile_for(u, *h)
                                && (*h == u.pos || self.reaches_this_turn(u, *h))
                                && self.outside_remembered_reach(*h)
                                && (crate::composition::charges(u.kind)
                                    || self.ranged_unit_clear(u, *h, target.pos))
                        })
                        .min_by_key(|h| {
                            (
                                h.distance(target.pos),
                                self.enemy_exposure_key(*h),
                                self.city_damage_at(u, *h),
                                std::cmp::Reverse(
                                    crate::composition::shooter_only(u.kind, u.ranged)
                                        && self.view().tile(*h).and_then(|t| t.terrain)
                                            == Some(Terrain::Hills),
                                ),
                                std::cmp::Reverse(
                                    self.view().tile(*h).and_then(|t| t.terrain)
                                        == Some(Terrain::Forest),
                                ),
                                h.distance(u.pos),
                                *h,
                            )
                        });
                    // A fallen city: a charging unit with no tile beside it this turn walks to
                    // the storm ring (even round the friends already in it) instead of parking
                    // a tile out, when its own charge passes the forecast (`storm_ring_spot`).
                    match staged {
                        Some(h) if h.distance(target.pos) <= 1 => Some(h),
                        staged if crate::composition::charges(u.kind) => {
                            self.storm_ring_spot(u, target.pos, &reserved).or(staged)
                        }
                        staged => staged,
                    }
                };
                if let Some(to) = to {
                    reserved.insert(to);
                    arrivals.push((u, to));
                }
            }
            let guaranteed_volley: i32 = arrivals
                .iter()
                .filter(|(u, to)| fires_from(u, *to, target.pos))
                .filter_map(|(u, _)| {
                    forecast_attack_mode(&self.attack_view(), u.id, target.pos, AttackMode::Ranged)
                })
                .map(|f| {
                    legatum_core::combat::damage(
                        f.attacker_strength.total,
                        f.defender_strength.total,
                        legatum_core::combat::ROLL_MIN,
                    )
                })
                .sum();
            if !garrison && !relief && guaranteed_volley >= city.hp {
                // Simultaneous ranged fire leaves an empty tile. Several melee
                // arrivals would bounce instead of occupying it; reserve one survivor.
                let capturer = arrivals
                    .iter()
                    .filter(|(u, to)| crate::composition::charges(u.kind) && *to == target.pos)
                    .filter(|(u, _)| u.hp > self.city_damage_now(u))
                    .min_by_key(|(u, _)| {
                        (u.pos.distance(target.pos), std::cmp::Reverse(u.hp), u.id)
                    })
                    .map(|(u, _)| u.id);
                arrivals.retain(|(u, to)| {
                    crate::composition::shooter_only(u.kind, u.ranged)
                        || fires_from(u, *to, target.pos)
                        || Some(u.id) == capturer
                });
            }
            if self.hard_defense() {
                while let Some(blocked) = self.assault_traffic_conflict(&arrivals, target.pos) {
                    let before = arrivals.len();
                    arrivals.retain(|(unit, _)| unit.id != blocked);
                    if arrivals.len() == before {
                        // An earlier civilian escape or defensive move has priority.
                        arrivals.clear();
                        break;
                    }
                }
            }
            if arrivals.is_empty() {
                continue;
            }
            let Some(capture_turn) = self.city_assault_turn(&arrivals, target.pos) else {
                continue;
            };
            // Gene `charge_preview`: the shots go first, so a charge's preview counts them.
            if self.decisions.charge_preview > 0 {
                arrivals.sort_by_key(|(u, to)| !fires_from(u, *to, target.pos));
            }
            // Moving participants enter together. A traffic conflict must not leave a
            // lone front rank exposed while its supporting army stays behind.
            self.assault_groups.push(
                arrivals
                    .iter()
                    .filter(|(u, to)| u.pos != *to)
                    .map(|(u, _)| u.id)
                    .collect(),
            );
            for (u, to) in arrivals {
                if fires_from(u, to, target.pos) {
                    self.mil_dest.insert(u.pos);
                    self.unit_order(
                        u.id,
                        Order::RangedAttack {
                            unit: u.id,
                            target: target.pos,
                        },
                    );
                } else if to == target.pos && capture_turn > 0 && self.charge_loses(u, target.pos) {
                    self.siege_hold(u, target.pos);
                } else if to == target.pos {
                    self.assault_attacks.insert(u.id);
                    self.unit_order(u.id, Order::MoveUnit { unit: u.id, to });
                } else {
                    self.go_to(u, to);
                }
            }
        }
    }

    /// A bounded capture forecast, including the approach turn. It deliberately keeps
    /// wounded survivors in the army: healing thresholds are not combat failure tests.
    #[cfg(test)]
    fn city_assault_succeeds(&self, arrivals: &[(&UnitView, Hex)], target: Hex) -> bool {
        self.city_assault_turn(arrivals, target).is_some()
    }

    /// The turn (0: this one) the bounded capture forecast takes the city, if it does.
    fn city_assault_turn(&self, arrivals: &[(&UnitView, Hex)], target: Hex) -> Option<i32> {
        let mut projected = self.attack_view().into_owned();
        let ids: Vec<_> = arrivals.iter().map(|(u, _)| u.id).collect();
        // A Keshig in the Mongol bow siege keeps shooting until the city reaches zero HP.
        let first_volley: Vec<_> = arrivals
            .iter()
            .filter(|(u, to)| fires_from(u, *to, target))
            .map(|(u, _)| u.id)
            .collect();
        let bow_siege = self.mongol_horde() && self.decisions.mongol_horde >= 3;
        let shoots = |u: &UnitView, turn: i32, city_hp: i32| {
            crate::composition::shooter_only(u.kind, u.ranged)
                || bow_siege
                    && crate::composition::dual(u.kind, u.ranged)
                    && city_hp > 0
                    && u.pos.distance(target) <= u.range
                || turn == 0 && first_volley.contains(&u.id)
        };
        // Do not rely on an uncommitted ally to absorb any future shot.
        projected
            .units
            .retain(|u| u.owner != projected.me || ids.contains(&u.id));
        // Walls halve every attack from a unit without Siege: the units that have it fight as if the
        // city had none (gene `siege_for_walls`).
        if self.hard_defense()
            && self.decisions.siege_for_walls > 0
            && let Some(city) = self
                .ctx
                .enemy_cities
                .iter()
                .copied()
                .find(|c| c.pos == target)
        {
            self.add_siege_cover(&mut projected, city, &ids, false);
        }
        // Gene `siege_window` (combat lab 2026-10-17): the same window as the siege forecast.
        let window = if self.hard_defense() && self.decisions.siege_window > 0 {
            self.decisions.siege_window
        } else {
            4
        };
        // Gene `relief_forecast`: the owner's melee units that can step in relieve the city (and
        // still count as attackers: the defender's orders are unknown).
        let mut relief = if self.decisions.relief_forecast > 0 {
            crate::opportunity::relief_units(&projected, target)
        } else {
            Vec::new()
        };
        for turn in 0..window {
            let city_hp = projected
                .cities
                .iter()
                .find(|c| c.pos == target)
                .map_or(0, |c| c.hp);
            let mut incoming = BTreeMap::<UnitId, i32>::new();
            for city in &projected.cities {
                if city.hp <= 0 || !projected.at_war(projected.me, city.owner) {
                    continue;
                }
                let victim = projected
                    .units
                    .iter()
                    .filter(|u| ids.contains(&u.id) && (1..=2).contains(&u.pos.distance(city.pos)))
                    .filter_map(|u| {
                        forecast_attack_mode(&projected, u.id, city.pos, AttackMode::Melee)
                            .and_then(|f| f.city_strike)
                            .map(|shot| (u, shot))
                    })
                    .min_by_key(|(u, _)| (u.hp, u.id));
                if let Some((u, shot)) = victim {
                    *incoming.entry(u.id).or_default() += shot.expected_damage(u.hp);
                }
            }
            // Visible ranged defenders share the ranged snapshot with city fire.
            // Melee responses belong to movement, after ranged casualties.
            let enemies: Vec<_> = projected
                .units
                .iter()
                .filter(|e| e.military && projected.at_war(projected.me, e.owner))
                .cloned()
                .collect();
            let me = projected.me;
            let team = projected.team;
            // One order each: an enemy with both attacks (the Keshig) that shoots doesn't charge.
            let mut fired = Vec::new();
            for enemy in enemies.iter().filter(|e| e.ranged > 0) {
                projected.me = enemy.owner;
                projected.team = projected.team_of(enemy.owner);
                let shot = projected
                    .units
                    .iter()
                    .filter(|u| ids.contains(&u.id))
                    .filter_map(|u| {
                        forecast_attack_mode(&projected, enemy.id, u.pos, AttackMode::Ranged)
                            .map(|f| (f.expected_damage_to_defender, std::cmp::Reverse(u.id)))
                    })
                    .max();
                if let Some((damage, std::cmp::Reverse(id))) = shot {
                    *incoming.entry(id).or_default() += damage;
                    fired.push(enemy.id);
                }
            }
            projected.me = me;
            projected.team = team;
            // Ranged support and city fire share the original snapshot; excess city damage
            // reaches the garrison at the defense ratio.
            let volley: i32 = projected
                .units
                .iter()
                .filter(|u| ids.contains(&u.id) && u.hp > 0 && shoots(u, turn, city_hp))
                .filter_map(|u| forecast_attack_mode(&projected, u.id, target, AttackMode::Ranged))
                .map(|f| f.expected_damage_to_defender)
                .sum();
            crate::analysis::damage_tile_defender(&mut projected, target, volley);
            for u in &mut projected.units {
                u.hp -= incoming.get(&u.id).copied().unwrap_or(0);
                if ids.contains(&u.id)
                    && shoots(u, turn, city_hp)
                    && u.pos.distance(target) <= u.range
                {
                    u.fortify = 0;
                }
            }
            projected.units.retain(|u| u.hp > 0);
            for &(unit, to) in arrivals {
                if let Some(u) = projected.units.iter_mut().find(|u| u.id == unit.id)
                    && u.pos != to
                {
                    u.fortify = 0;
                }
            }
            // Budget one possible melee response per visible supporting defender.
            // The tile's defending side retaliates once across these movement rounds.
            let mut responses = BTreeMap::<usize, Vec<(UnitId, UnitId)>>::new();
            for enemy in enemies.iter().filter(|e| {
                crate::composition::charges(e.kind) && e.pos != target && !fired.contains(&e.id)
            }) {
                let Some(enemy) = projected.units.iter().find(|u| u.id == enemy.id) else {
                    continue;
                };
                projected.me = enemy.owner;
                projected.team = projected.team_of(enemy.owner);
                let attack = projected
                    .units
                    .iter()
                    .filter(|u| ids.contains(&u.id))
                    .filter_map(|u| {
                        let round = response_round(&projected, enemy, u.pos)?;
                        let f =
                            forecast_attack_mode(&projected, enemy.id, u.pos, AttackMode::Melee)?;
                        Some((
                            f.expected_damage_to_defender,
                            std::cmp::Reverse(u.id),
                            round,
                        ))
                    })
                    .max_by_key(|&(damage, id, _)| (damage, id));
                if let Some((_, std::cmp::Reverse(id), round)) = attack {
                    responses.entry(round).or_default().push((enemy.id, id));
                }
            }
            projected.me = me;
            projected.team = team;
            // Movement rounds are steps, not whole turns. A later arrival can fight a
            // garrison exposed, or capture a city emptied, in an earlier movement round.
            let mut melee_rounds = BTreeMap::<usize, Vec<(UnitId, Hex)>>::new();
            for &(unit, to) in arrivals {
                let Some(u) = projected.units.iter().find(|u| u.id == unit.id) else {
                    continue;
                };
                if shoots(u, turn, city_hp) || turn == 0 && to != target {
                    continue;
                }
                let Some(path) = self.known_path_this_turn(u, target) else {
                    continue;
                };
                if path.is_empty() {
                    continue;
                }
                let from = path.iter().rev().nth(1).copied().unwrap_or(u.pos);
                melee_rounds
                    .entry(path.len())
                    .or_default()
                    .push((u.id, from));
            }
            let rounds: std::collections::BTreeSet<_> = melee_rounds
                .keys()
                .chain(responses.keys())
                .copied()
                .collect();
            let mut counter_spent = false;
            for round in rounds {
                let attackers = melee_rounds.get(&round).map(Vec::as_slice).unwrap_or(&[]);
                for &(id, from) in attackers {
                    let Some(u) = projected.units.iter_mut().find(|u| u.id == id) else {
                        continue;
                    };
                    if u.pos != from {
                        u.pos = from;
                        u.fortify = 0;
                    }
                }
                // Like core's movement combat, every fight in this round sees the same
                // defender and HP: the city while it has HP, else its garrison.
                let mut unit_damage = BTreeMap::<UnitId, i32>::new();
                let mut capturers = Vec::new();
                let mut melee_damage = 0;
                let mut counter_group = Vec::new();
                for &(id, _) in attackers {
                    if let Some(f) = forecast_attack_mode(&projected, id, target, AttackMode::Melee)
                    {
                        melee_damage += f.expected_damage_to_defender;
                        counter_group.push((
                            id,
                            f.attacker_strength.total,
                            f.expected_damage_to_attacker,
                        ));
                        capturers.push(id);
                    }
                }
                if !counter_group.is_empty() && !counter_spent {
                    let &(strongest, _, total) = counter_group
                        .iter()
                        .max_by_key(|&&(id, strength, _)| (strength, std::cmp::Reverse(id)))
                        .expect("counterattack group");
                    let share = total / counter_group.len() as i32;
                    for (id, _, _) in &counter_group {
                        *unit_damage.entry(*id).or_default() += share
                            + if *id == strongest {
                                total % counter_group.len() as i32
                            } else {
                                0
                            };
                    }
                    counter_spent = true;
                }
                for &(enemy_id, id) in responses.get(&round).map(Vec::as_slice).unwrap_or(&[]) {
                    let Some(enemy) = projected.units.iter().find(|u| u.id == enemy_id) else {
                        continue;
                    };
                    let Some(victim) = projected.units.iter().find(|u| u.id == id) else {
                        continue;
                    };
                    projected.me = enemy.owner;
                    projected.team = projected.team_of(enemy.owner);
                    if let Some(f) =
                        forecast_attack_mode(&projected, enemy_id, victim.pos, AttackMode::Melee)
                    {
                        *unit_damage.entry(id).or_default() += f.expected_damage_to_defender;
                        *unit_damage.entry(enemy_id).or_default() += f.expected_damage_to_attacker;
                    }
                }
                projected.me = me;
                projected.team = team;
                let cleared = |v: &PlayerView| {
                    v.cities.iter().any(|c| c.pos == target && c.hp == 0)
                        && !v
                            .units
                            .iter()
                            .any(|e| e.pos == target && e.military && v.at_war(v.me, e.owner))
                };
                if relief.is_empty() {
                    crate::analysis::damage_tile_defender(&mut projected, target, melee_damage);
                } else {
                    // Blow by blow: a cleared city takes the next relief unit, which the
                    // following blows fight; it is taken only when no relief is left.
                    for &id in &capturers {
                        if cleared(&projected)
                            && !crate::opportunity::relieve(&mut projected, &mut relief, target)
                        {
                            break;
                        }
                        if let Some(f) =
                            forecast_attack_mode(&projected, id, target, AttackMode::Melee)
                        {
                            crate::analysis::damage_tile_defender(
                                &mut projected,
                                target,
                                f.expected_damage_to_defender,
                            );
                        }
                    }
                    if cleared(&projected) {
                        crate::opportunity::relieve(&mut projected, &mut relief, target);
                    }
                }
                for u in &mut projected.units {
                    if let Some(damage) = unit_damage.get(&u.id) {
                        u.hp -= damage;
                        u.fortify = 0;
                    }
                }
                projected.units.retain(|u| u.hp > 0);
                if projected.units.iter().any(|u| capturers.contains(&u.id))
                    && projected
                        .cities
                        .iter()
                        .any(|c| c.pos == target && c.hp == 0)
                    && !projected.units.iter().any(|e| {
                        e.pos == target && e.military && projected.at_war(projected.me, e.owner)
                    })
                {
                    return Some(turn);
                }
            }
            if !projected
                .units
                .iter()
                .any(|u| ids.contains(&u.id) && crate::composition::charges(u.kind))
            {
                return None;
            }
            if turn == 0 {
                for &(unit, to) in arrivals {
                    if let Some(u) = projected.units.iter_mut().find(|u| u.id == unit.id)
                        && to != target
                        && to != u.pos
                    {
                        u.pos = to;
                        u.fortify = 0;
                    }
                }
            }
        }
        None
    }

    /// Gene `charge_preview`: the engine's preview (this turn's ordered shots counted) has a melee
    /// charge at `city` dying more often than not, or taking more damage than it deals.
    pub(super) fn charge_loses(&self, u: &UnitView, city: Hex) -> bool {
        self.decisions.charge_preview > 0
            && self.melee_forecast(u, city).is_none_or(|f| {
                f.loss_chance > 50 || f.expected_damage_to_attacker > f.expected_damage_to_defender
            })
    }
}

/// Whether an assault arrival fires this turn: it stays where it is with the city in range (a
/// shooter's place, or a Keshig that cannot reach the city and holds within its bow's range).
fn fires_from(u: &UnitView, to: Hex, target: Hex) -> bool {
    u.ranged > 0 && to == u.pos && u.pos.distance(target) <= u.range
}

fn response_round(view: &PlayerView, unit: &UnitView, target: Hex) -> Option<usize> {
    if unit.pos.distance(target) > unit.max_moves.max(1) {
        return None;
    }
    let path = view.movement_path(unit.owner, unit.kind, unit.pos, target)?;
    let mut moves = unit.max_moves;
    for &h in &path {
        let cost = view.tile(h)?.move_cost()?;
        if !legatum_core::rules::movement::can_step(moves, unit.max_moves, cost) {
            return None;
        }
        moves = (moves - cost).max(0);
    }
    (!path.is_empty()).then_some(path.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder};

    const CITY: Hex = Hex::new(10, 8);

    fn planner(view: &PlayerView) -> Planner<'_> {
        Planner::new(
            view,
            Knobs::for_faction(view.faction_of(ME)),
            view.faction_of(ME),
            7,
        )
    }

    #[test]
    fn hard_reforecasts_without_a_blocked_follower_instead_of_canceling_the_siege() {
        let view = ViewBuilder::new(26, 18)
            .city(9, ENEMY, CITY, 3)
            .unit(1, ME, UnitKind::Pikeman, Hex::new(8, 8))
            .unit(2, ME, UnitKind::Pikeman, Hex::new(7, 9))
            .unit(3, ME, UnitKind::Archer, Hex::new(10, 6))
            .unit(4, ME, UnitKind::Archer, Hex::new(11, 6))
            .unit(5, ME, UnitKind::Archer, Hex::new(12, 6))
            .build();
        let mut p = planner(&view);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        let army = p.ctx.own_units.clone();
        p.coordinate_city_assaults(&army);
        p.dedupe_first_steps();
        assert!(
            p.orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { unit: UnitId(1), to } if *to == CITY)),
            "{:?}",
            p.orders
        );
        assert!(
            !p.assault_groups.iter().any(|g| g.contains(&UnitId(2))),
            "the follower cannot occupy the attacker's melee tile"
        );
        assert_eq!(
            p.orders
                .iter()
                .filter(|o| matches!(o, Order::RangedAttack { .. }))
                .count(),
            3
        );
    }

    #[test]
    fn hard_does_not_assign_a_ranged_unit_to_capture_an_empty_city() {
        let mut view = ViewBuilder::new(26, 18)
            .city(9, ENEMY, CITY, 3)
            .unit(1, ME, UnitKind::Archer, Hex::new(9, 8))
            .build();
        view.cities[0].hp = 0;
        let mut p = planner(&view);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        let army = p.ctx.own_units.clone();
        p.coordinate_city_assaults(&army);
        assert!(
            p.orders.is_empty(),
            "ranged units cannot capture: {:?}",
            p.orders
        );
    }

    #[test]
    fn mongol_horde_shoots_walls_from_two_hexes_and_rides_into_a_fallen_city() {
        let mut view = ViewBuilder::new(26, 18)
            .faction(legatum_core::Faction::Mongols)
            .city(9, ENEMY, CITY, 3)
            .walled(9)
            .unit(1, ME, UnitKind::Cataphract, Hex::new(10, 6))
            .unit(2, ME, UnitKind::Cataphract, Hex::new(12, 8))
            .build();
        view.cities[0].hp = 35;
        let mut p = planner(&view);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p.decisions.mongol_horde = 3;
        let army = p.ctx.own_units.clone();
        p.coordinate_city_assaults(&army);
        assert!(
            p.orders.iter().any(|o| matches!(o,
            Order::RangedAttack { unit: UnitId(1), target } if *target == CITY)),
            "{:?}",
            p.orders
        );
        assert!(
            !p.orders.iter().any(|o| matches!(o,
            Order::MoveUnit { to, .. } if *to == CITY)),
            "{:?}",
            p.orders
        );
        let mut fallen = view;
        fallen.cities[0].hp = 0;
        let mut p = planner(&fallen);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p.decisions.mongol_horde = 3;
        let army = p.ctx.own_units.clone();
        p.coordinate_city_assaults(&army);
        assert!(
            p.orders.iter().any(|o| matches!(o,
            Order::MoveUnit { to, .. } if *to == CITY)),
            "{:?}",
            p.orders
        );
    }

    #[test]
    fn relief_forecast_needs_a_second_attacker_for_the_unit_that_steps_in() {
        // A fallen city with a nearly dead garrison and a fresh Spearman beside it that can
        // relieve it (Fable seat, After the Riflemen: the swap-in beat every lone walk-in).
        let mut view = ViewBuilder::new(26, 18)
            .city(9, ENEMY, CITY, 3)
            .unit(1, ME, UnitKind::Rifleman, Hex::new(9, 8))
            .unit(2, ME, UnitKind::Rifleman, Hex::new(10, 7))
            .unit_hp(50, ENEMY, UnitKind::Spearman, CITY, 5)
            .unit(51, ENEMY, UnitKind::Spearman, Hex::new(11, 8))
            .build();
        view.cities[0].hp = 0;
        let mut p = planner(&view);
        let army = p.ctx.own_units.clone();
        let lone = [(army[0], CITY)];
        let both: Vec<_> = army.iter().map(|&u| (u, CITY)).collect();
        assert_eq!(
            p.city_assault_turn(&lone, CITY),
            Some(0),
            "gene off: no relief seen"
        );
        p.decisions.relief_forecast = 1;
        assert_ne!(
            p.city_assault_turn(&lone, CITY),
            Some(0),
            "the relief steps in"
        );
        assert_eq!(
            p.city_assault_turn(&both, CITY),
            Some(0),
            "the second kills the relief"
        );
    }

    #[test]
    fn split_garrison_retaliation_allows_wounded_attackers_to_capture() {
        let mut view = ViewBuilder::new(26, 18)
            .city(9, ENEMY, CITY, 3)
            .unit_hp(1, ME, UnitKind::Spearman, Hex::new(9, 8), 40)
            .unit_hp(2, ME, UnitKind::Spearman, Hex::new(10, 7), 40)
            .unit_hp(50, ENEMY, UnitKind::Pikeman, CITY, 1)
            .build();
        // A fallen city: its garrison defends (core `combat::city_tile_defender`).
        view.cities[0].hp = 0;
        view.units[2].fortify = 6;
        let mut p = planner(&view);
        let army = p.ctx.own_units.clone();
        let arrivals: Vec<_> = army.iter().map(|&u| (u, CITY)).collect();
        for u in &army {
            let f = forecast_attack_mode(&view, u.id, CITY, AttackMode::Melee).unwrap();
            assert_eq!(f.loss_chance, 100, "the garrison kills either Spearman");
        }
        assert!(p.city_assault_succeeds(&arrivals, CITY));
        let mut reversed = arrivals.clone();
        reversed.reverse();
        assert!(p.city_assault_succeeds(&reversed, CITY));
        p.coordinate_city_assaults(&army);
        assert!(!p.orders.is_empty(), "commit the shared assault");
    }

    /// Round one breaks the 1-HP city (its garrison is shielded until then); the Spearman
    /// arriving a round later fights the exposed garrison and captures.
    #[test]
    fn later_movement_round_can_capture_after_the_garrison_dies() {
        let mut view = ViewBuilder::new(26, 18)
            .city(9, ENEMY, CITY, 3)
            .unit_hp(1, ME, UnitKind::Spearman, Hex::new(9, 8), 40)
            .unit_hp(2, ME, UnitKind::Spearman, Hex::new(10, 6), 40)
            .unit_hp(50, ENEMY, UnitKind::Pikeman, CITY, 1)
            .build();
        view.cities[0].hp = 1;
        view.units[2].fortify = 6;
        let mut p = planner(&view);
        let army = p.ctx.own_units.clone();
        let arrivals: Vec<_> = army.iter().map(|&u| (u, CITY)).collect();
        assert!(p.city_assault_succeeds(&arrivals, CITY));
        p.coordinate_city_assaults(&army);
        p.dedupe_first_steps();
        assert_eq!(p.orders.len(), 2);
        assert!(p.orders.iter().all(|o| matches!(o,
            Order::MoveUnit { to, .. } if *to == CITY)));
    }

    #[test]
    fn supporting_melee_defender_takes_retaliation_in_the_same_round() {
        let view = ViewBuilder::new(26, 18)
            .city(9, ENEMY, CITY, 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(9, 8))
            .unit(2, ME, UnitKind::Spearman, Hex::new(10, 7))
            .unit(3, ME, UnitKind::Archer, Hex::new(9, 7))
            .unit(4, ME, UnitKind::Archer, Hex::new(10, 6))
            .unit_hp(50, ENEMY, UnitKind::Pikeman, Hex::new(8, 8), 1)
            .build();
        let p = planner(&view);
        let arrivals: Vec<_> = p
            .ctx
            .own_units
            .iter()
            .map(|&u| (u, if u.ranged > 0 { u.pos } else { CITY }))
            .collect();
        assert!(p.city_assault_succeeds(&arrivals, CITY));
    }

    #[test]
    fn wounded_capturer_stays_with_a_winning_volley_instead_of_healing() {
        let mut view = ViewBuilder::new(26, 18)
            .city(1, ME, Hex::new(3, 8), 2)
            .city(9, ENEMY, CITY, 3)
            .unit_hp(1, ME, UnitKind::Spearman, Hex::new(8, 8), 55)
            .unit(2, ME, UnitKind::Archer, Hex::new(9, 7))
            .unit(3, ME, UnitKind::Archer, Hex::new(10, 6))
            .build();
        view.cities[1].hp = 40;
        let mut p = planner(&view);
        p.plan_military();
        p.dedupe_first_steps();
        assert!(
            p.orders.iter().any(|o| matches!(o,
            Order::MoveUnit { unit: UnitId(1), to } if *to == CITY)),
            "{:?}",
            p.orders
        );
        for id in [2, 3] {
            assert!(p.orders.iter().any(|o| matches!(o,
                Order::RangedAttack { unit, target } if *unit == UnitId(id) && *target == CITY)));
        }
        assert!(!p.retreats.contains(&UnitId(1)));
    }

    #[test]
    fn massed_infantry_enters_with_ranged_support_and_survives_traffic_reconciliation() {
        let ring = CITY.ring(3);
        let view = ViewBuilder::new(26, 18)
            .city(9, ENEMY, CITY, 3)
            .unit(1, ME, UnitKind::Spearman, ring[0])
            .unit(2, ME, UnitKind::Spearman, ring[3])
            .unit(3, ME, UnitKind::Spearman, ring[6])
            .unit(4, ME, UnitKind::Archer, ring[9])
            .unit(5, ME, UnitKind::Archer, ring[12])
            .build();
        let mut p = planner(&view);
        let army = p.ctx.own_units.clone();
        p.coordinate_city_assaults(&army);
        p.dedupe_first_steps();
        assert_eq!(p.orders.len(), 5, "{:?}", p.orders);
        assert!(
            p.orders.iter().all(|o| matches!(o,
            Order::MoveUnit { to, .. } if (1..=2).contains(&to.distance(CITY)))),
            "{:?}",
            p.orders
        );
        assert_eq!(p.assault_groups[0].len(), 5);
    }

    /// Gene `finish_fallen` (8.9.25 Celts T93: Celts Colony 7 at 0 HP, its Crossbowman wounded,
    /// the Mongol campaign withdrawn): a broken city beside our units is assaulted even when the
    /// running campaign aims elsewhere.
    #[test]
    fn finish_fallen_assaults_a_broken_city_outside_the_campaign() {
        let mut view = ViewBuilder::new(26, 18)
            .city(1, ME, Hex::new(3, 8), 2)
            .city(9, ENEMY, CITY, 3)
            .walled(9)
            .city(10, ENEMY, Hex::new(20, 14), 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(9, 8))
            .unit(2, ME, UnitKind::Archer, Hex::new(10, 6))
            .unit(3, ME, UnitKind::Archer, Hex::new(12, 8))
            .unit_hp(50, ENEMY, UnitKind::Archer, CITY, 40)
            .build();
        view.cities[1].hp = 0;
        let mut p = planner(&view);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p.campaigns.active = Some(crate::campaign::Campaign {
            target: legatum_core::CityId(10),
            start_turn: view.turn,
            committed: Vec::new(),
            deadline: None,
            expected_value: 100,
            target_was_unwalled: false,
        });
        let army = p.ctx.own_units.clone();
        p.coordinate_city_assaults(&army);
        assert!(p.orders.is_empty(), "the campaign filter: {:?}", p.orders);
        let mut p = planner(&view);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p.decisions.finish_fallen = 50;
        p.campaigns.active = Some(crate::campaign::Campaign {
            target: legatum_core::CityId(10),
            start_turn: view.turn,
            committed: Vec::new(),
            deadline: None,
            expected_value: 100,
            target_was_unwalled: false,
        });
        p.finish_fallen_cities(&army);
        assert!(
            p.orders.iter().any(|o| matches!(o,
                Order::MoveUnit { unit: UnitId(1), to } if *to == CITY)),
            "{:?}",
            p.orders
        );
    }

    #[test]
    fn failed_assault_releases_wounded_units_for_recovery() {
        let view = ViewBuilder::new(26, 18)
            .city(1, ME, Hex::new(3, 8), 2)
            .city(9, ENEMY, CITY, 3)
            .walled(9)
            .unit_hp(1, ME, UnitKind::Spearman, Hex::new(8, 8), 55)
            .build();
        let mut p = planner(&view);
        p.plan_military();
        assert!(
            p.orders.iter().any(|o| matches!(o,
            Order::MoveUnit { unit: UnitId(1), to } if to.distance(CITY) > 2)),
            "{:?}",
            p.orders
        );
        assert!(p.retreats.contains(&UnitId(1)));
    }
}

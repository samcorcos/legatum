//! Campaign and border movement decisions use the same fogged snapshot as combat.
use super::*;
use legatum_core::diplomacy::TreatyKind;

/// Spearmen that take a lightly held city, and the Archers a walled one needs outside it.
const ASSAULT_SOLDIERS: usize = 3;
const WALL_SIEGE_RANGERS: usize = 2;

impl Plan<'_> {
    /// Healthy Spearmen within a march of the city: the ones that take it.
    pub(super) fn assault_force(&self, at: Hex) -> usize {
        self.ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.ranged == 0
                    && u.kind != UnitKind::Scout
                    && u.hp >= self.assault_hp()
                    && u.pos.distance(at) <= 4
            })
            .count()
    }

    fn assault_hp(&self) -> i32 {
        if self.fights_wounded() { 40 } else { 60 }
    }

    /// Three or more Spearmen beat a defended city: the human took cities against two or three
    /// defenders with no ranged support. Defenders count by their melee strength, the city by
    /// double its own, and our wounded count for less.
    pub(super) fn assault_ready(&self, city: &CityView) -> bool {
        let defenders: Vec<_> = self
            .ctx
            .view
            .units
            .iter()
            .filter(|u| {
                u.military
                    && self.ctx.view.same_team(u.owner, city.owner)
                    && u.pos.distance(city.pos) <= 3
            })
            .collect();
        // Every defender in reach is another Spearman needed: a garrison behind a broken wall
        // still has to be killed.
        let need = (ASSAULT_SOLDIERS + defenders.len().saturating_sub(1)).min(6);
        if self.assault_force(city.pos) < need {
            return false;
        }
        // A walled city shoots back at everything near it, so it is shot from outside first.
        if city.walls
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.ranged > 0 && u.hp >= 60 && u.pos.distance(city.pos) <= 4)
                .count()
                < WALL_SIEGE_RANGERS
        {
            return false;
        }
        let power: i32 = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(city.pos) <= 4)
            .map(|u| u.strength.max(u.ranged / 2) * u.hp / 100)
            .sum();
        let defense: i32 = city.strength * 2
            + defenders
                .iter()
                .map(|u| u.strength.max(u.ranged / 2) * u.hp / 100)
                .sum::<i32>();
        power * 10 >= defense * 8
    }

    pub(super) fn legal(&self, order: &Order) -> bool {
        self.ctx
            .view
            .diplomacy
            .actions
            .iter()
            .any(|a| a.order == *order && a.blocked_reason.is_none())
    }

    pub(super) fn select_campaign(&mut self) {
        let v = self.ctx.view;
        if self.target.is_some_and(|id| {
            v.cities.iter().any(|c| {
                c.id == id
                    && c.owner != v.me
                    && !v.same_team(v.me, c.owner)
                    && !self.stalled.contains(&v.team_of(c.owner))
                    && (!v.turn.is_multiple_of(12) || c.visible && c.hp < c.max_hp)
            })
        }) {
            return;
        }
        let candidates = |skip_stalled: bool| {
            v.cities
                .iter()
                .filter(|c| !v.same_team(v.me, c.owner))
                .filter(|c| !skip_stalled || !self.stalled.contains(&v.team_of(c.owner)))
                .filter(|c| v.players.iter().any(|p| p.id == c.owner && p.alive))
                .filter(|c| {
                    self.ctx.own_cities.iter().any(|home| {
                        home.pos.distance(c.pos) <= 24
                            && find_path(&self.map, home.pos, c.pos, |h| {
                                let tile = v.tile(h)?;
                                (tile.explored
                                    && (v.passage_allowed(
                                        v.me,
                                        legatum_core::UnitKind::Spearman,
                                        tile.owner,
                                    ) || tile
                                        .owner
                                        .is_some_and(|owner| v.same_team(owner, c.owner))))
                                .then(|| tile.terrain?.move_cost())
                                .flatten()
                            })
                            .is_some()
                    })
                })
                .filter(|c| {
                    v.at_war(v.me, c.owner)
                        || self.legal(&Order::DeclareWar {
                            against: v.team_of(c.owner),
                            breach: false,
                        })
                })
                .min_by_key(|c| {
                    let home_distance = self
                        .ctx
                        .own_cities
                        .iter()
                        .map(|h| h.pos.distance(c.pos))
                        .min()
                        .unwrap_or(100);
                    // The army rolls on from where it stands: the nearest or weakest next.
                    let army_distance = self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|u| u.military && u.kind != UnitKind::Scout && u.hp >= 40)
                        .map(|u| u.pos.distance(c.pos))
                        .min()
                        .unwrap_or(home_distance);
                    let defenders: i32 = v
                        .units
                        .iter()
                        .filter(|u| {
                            u.military
                                && v.same_team(u.owner, c.owner)
                                && u.pos.distance(c.pos) <= 4
                        })
                        .map(|u| u.strength.max(u.ranged) * u.hp / 100)
                        .sum();
                    (
                        home_distance * 4
                            + army_distance * 10
                            + c.strength * 2
                            + c.hp / 5
                            + defenders * 2
                            + i32::from(c.walls) * 100
                            - i32::from(c.visible && defenders == 0) * 80
                            - i32::from(v.at_war(v.me, c.owner)) * 100,
                        c.id,
                    )
                })
                .map(|c| c.id)
        };
        // A war that stalled moves the army to another front; with no other front it stays.
        self.target = candidates(true).or_else(|| candidates(false));
    }

    pub(super) fn diplomacy(&mut self) {
        let v = self.ctx.view;
        // Peace is a tool: take it from a rival whose war stalled, refuse it otherwise.
        for p in &v.diplomacy.proposals {
            if p.to != v.team {
                continue;
            }
            let order = Order::RespondToProposal {
                proposal: p.id,
                accept: p.kind == TreatyKind::Peace && self.stalled.contains(&p.from),
                explanation: None,
            };
            if self.legal(&order) {
                self.orders.push(order);
            }
        }
        for p in &v.players {
            let order = Order::ProposeTreaty {
                to: p.team,
                kind: TreatyKind::Peace,
            };
            if p.alive
                && p.team != v.team
                && self.stalled.contains(&p.team)
                && v.at_war(v.me, p.id)
                && self.legal(&order)
            {
                self.orders.push(order);
                self.peace_offers.push(p.team);
            }
        }
        // A Settler without an escort is the cheapest city there is: declare war the turn a
        // Spearman stands beside it and take it.
        if !self.ctx.own_cities.is_empty()
            && let Some(settler) =
                v.units
                    .iter()
                    .filter(|e| {
                        e.kind == UnitKind::Settler
                            && !v.same_team(v.me, e.owner)
                            && !v.at_war(v.me, e.owner)
                            && !v.units.iter().any(|guard| {
                                guard.military
                                    && v.same_team(guard.owner, e.owner)
                                    && guard.pos.distance(e.pos) <= 2
                            })
                            && !v.cities.iter().any(|c| {
                                v.same_team(c.owner, e.owner) && c.pos.distance(e.pos) <= 2
                            })
                            && v.tile(e.pos).is_some_and(|t| {
                                t.owner.is_none_or(|owner| {
                                    v.same_team(owner, e.owner)
                                        || v.passage_allowed(
                                            v.me,
                                            legatum_core::UnitKind::Spearman,
                                            Some(owner),
                                        )
                                })
                            })
                            && self
                                .ctx
                                .own_units
                                .iter()
                                .any(|u| u.military && u.hp >= 60 && u.pos.distance(e.pos) == 1)
                            && self.legal(&Order::DeclareWar {
                                against: v.team_of(e.owner),
                                breach: false,
                            })
                    })
                    .min_by_key(|e| e.id)
        {
            self.orders.push(Order::DeclareWar {
                against: v.team_of(settler.owner),
                breach: false,
            });
            return;
        }
        let Some(city) = self
            .target
            .and_then(|id| v.cities.iter().find(|c| c.id == id))
        else {
            return;
        };
        if v.at_war(v.me, city.owner) {
            return;
        }
        // Staged 2-3 hexes out at peace, the army declares and strikes in the same turn.
        let order = Order::DeclareWar {
            against: v.team_of(city.owner),
            breach: false,
        };
        if self.phase == Phase::Conquer && self.assault_ready(city) && self.legal(&order) {
            self.orders.push(order);
        }
    }

    pub(super) fn city_incoming(&self, u: &UnitView, pos: Hex) -> i32 {
        self.ctx
            .enemy_cities
            .iter()
            .filter(|c| {
                if pos == u.pos {
                    return legatum_core::city_strike_target(self.ctx.view, c) == Some(u.id);
                }
                // Only one city shot. Include the already assigned infantry screen
                // in the next-turn forecast, without relying on unseen enemy orders.
                !self.ctx.view.units.iter().any(|v| {
                    if v.id == u.id
                        || !v.military
                        || !self.ctx.view.at_war(c.owner, v.owner)
                        || (v.hp, v.id) >= (u.hp, u.id)
                    {
                        return false;
                    }
                    let end = self
                        .orders
                        .iter()
                        .find_map(|o| match o {
                            Order::MoveUnitPath { unit, to, .. } if *unit == v.id => Some(*to),
                            _ => None,
                        })
                        .unwrap_or(v.pos);
                    (1..=2).contains(&c.pos.distance(end)) && self.ranged_incoming(v) < v.hp
                })
            })
            .filter_map(|c| legatum_core::forecast_city_strike(self.ctx.view, c, u, pos))
            .map(|s| s.damage_at_roll(legatum_core::combat::ROLL_MAX))
            .sum()
    }

    /// City fire precedes movement. Leave while the current shot is survivable,
    /// instead of discovering next turn that retreat can no longer save the unit.
    /// A guaranteed volley that disables the city keeps its finishing shot.
    pub(super) fn needs_rotation(&self, u: &UnitView) -> bool {
        let today = self.city_incoming(u, u.pos);
        if today == 0 || today >= u.hp {
            return false;
        }
        let mut wounded = u.clone();
        wounded.hp -= today;
        let tomorrow: i32 = self
            .ctx
            .enemy_cities
            .iter()
            .filter_map(|city| {
                // The volley hits the city even when it is garrisoned (core
                // `combat::city_tile_defender`), and a city at 0 HP stops firing.
                let damage = self.damage.get(&city.pos).copied().unwrap_or(0);
                let occupier = self.ctx.own_units.iter().any(|melee| {
                    melee.ranged == 0
                        && melee.military
                        && melee.kind != UnitKind::Scout
                        && melee.hp >= 60
                        && melee.pos.distance(city.pos) == 1
                });
                if damage >= city.hp && occupier {
                    return None;
                }
                // A disabled city heals and fires again unless infantry actually occupies it.
                let mut projected = (*city).clone();
                projected.hp = (city.hp - damage)
                    .max(0)
                    .saturating_add(legatum_core::data::CITY_HEAL)
                    .min(city.max_hp);
                legatum_core::forecast_city_strike(self.ctx.view, &projected, &wounded, u.pos)
            })
            .map(|shot| shot.damage_at_roll(legatum_core::combat::ROLL_MAX))
            .sum();
        tomorrow > 0 && tomorrow >= wounded.hp - 10
    }

    pub(super) fn march(&mut self, u: &UnitView) -> bool {
        let target = self
            .target
            .and_then(|id| self.ctx.view.cities.iter().find(|c| c.id == id));
        let (center, radius) = if let Some(c) = target.filter(|_| self.phase == Phase::Conquer) {
            let at_war = self.ctx.view.at_war(u.owner, c.owner);
            let exposed = c.visible
                && c.hp <= 60
                && !self
                    .ctx
                    .enemy_units
                    .iter()
                    .any(|e| e.military && e.pos.distance(c.pos) <= 2);
            let engaged = self
                .ctx
                .own_units
                .iter()
                .any(|v| v.military && v.kind != UnitKind::Scout && v.pos.distance(c.pos) <= 2);
            // At war, close on the city once enough Spearmen are in reach; otherwise gather
            // 2-3 hexes out (at Peace, outside its borders) and wait for them.
            let radius = if at_war && (self.assault_ready(c) || exposed || engaged) {
                if u.ranged > 0 { u.range } else { 1 }
            } else {
                2
            };
            (c.pos, radius)
        } else if let Some(c) = self.ctx.nearest_own_city(u.pos) {
            // Leave the city tile clear for purchases and new production.
            (c.pos, 1)
        } else {
            return false;
        };
        let field = self.ctx.distance_field(u.pos, false);
        // A crowded or foreign-owned staging ring spills outward: at Peace the borders of a
        // grown city can cover the first rings, and the army waits just outside them.
        let rings = if radius == 2 { 5 } else { 1 };
        let mut spots: Vec<_> =
            (0..rings)
                .flat_map(|extra| {
                    center
                        .ring(radius + extra)
                        .into_iter()
                        .map(move |h| (extra, h))
                })
                .filter(|(_, h)| {
                    self.free(u, *h)
                        && self.ctx.view.tile(*h).is_some_and(|t| {
                            self.ctx.view.passage_allowed(u.owner, u.kind, t.owner)
                        })
                })
                .filter_map(|(extra, h)| {
                    let distance = field[self.ctx.view.index(h)?];
                    (distance != UNREACHABLE)
                        .then(|| (self.incoming(u, h) * 3 + distance * 8 + extra * 40, h))
                })
                .collect();
        spots.sort();
        for (_, h) in spots {
            if h == u.pos {
                self.hold(u);
                return true;
            }
            if self.go(u, h, false) {
                return true;
            }
        }
        // The explored map does not reach the target yet: open the road to it.
        if target.is_some() && self.phase == Phase::Conquer {
            let mut frontier: Vec<_> = self
                .ctx
                .frontier
                .iter()
                .copied()
                .filter(|h| *h != u.pos)
                .filter_map(|h| {
                    let d = field[self.ctx.view.index(h)?];
                    (d != UNREACHABLE && self.exposure(h) == 0)
                        .then(|| (h.distance(center) * 10 + d * 3, h))
                })
                .collect();
            frontier.sort();
            for (_, h) in frontier.into_iter().take(6) {
                if self.go(u, h, false) {
                    return true;
                }
            }
        }
        false
    }
}

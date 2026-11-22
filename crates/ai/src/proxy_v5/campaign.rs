//! Campaign and border movement decisions use the same fogged snapshot as combat.
use super::*;
use legatum_core::diplomacy::TreatyKind;

impl Plan<'_> {
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
        if matches!(self.phase, Phase::Expand | Phase::Consolidate) {
            self.target = None;
            return;
        }
        if self.target.is_some_and(|id| {
            v.cities.iter().any(|c| {
                c.id == id
                    && c.owner != v.me
                    && (v.at_war(v.me, c.owner) || !v.turn.is_multiple_of(6))
            })
        }) {
            return;
        }
        self.target = v
            .cities
            .iter()
            .filter(|c| !v.same_team(v.me, c.owner))
            .filter(|c| v.players.iter().any(|p| p.id == c.owner && p.alive))
            .filter(|c| {
                self.ctx.own_cities.iter().any(|home| {
                    home.pos.distance(c.pos) <= 14
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
                let distance = self
                    .ctx
                    .own_cities
                    .iter()
                    .map(|h| h.pos.distance(c.pos))
                    .min()
                    .unwrap_or(100);
                let defenders: i32 = v
                    .units
                    .iter()
                    .filter(|u| {
                        u.military && v.same_team(u.owner, c.owner) && u.pos.distance(c.pos) <= 4
                    })
                    .map(|u| u.strength.max(u.ranged) * u.hp / 100)
                    .sum();
                (
                    distance * 15 + c.strength + c.hp / 10 + defenders * 2
                        - i32::from(v.at_war(v.me, c.owner)) * 100,
                    c.id,
                )
            })
            .map(|c| c.id);
    }

    pub(super) fn diplomacy(&mut self) {
        let v = self.ctx.view;
        for p in &v.diplomacy.proposals {
            if p.to != v.team {
                continue;
            }
            let order = Order::RespondToProposal {
                proposal: p.id,
                accept: p.kind == TreatyKind::Peace && self.phase == Phase::Consolidate,
                explanation: None,
            };
            if self.legal(&order) {
                self.orders.push(order);
            }
        }
        if self.phase == Phase::Consolidate {
            for p in &v.players {
                let order = Order::ProposeTreaty {
                    to: p.team,
                    kind: TreatyKind::Peace,
                };
                if p.alive && p.team != v.team && v.at_war(v.me, p.id) && self.legal(&order) {
                    self.orders.push(order);
                }
            }
            return;
        }
        if self.ctx.own_cities.len() >= 2
            && !v.players.iter().any(|p| p.alive && v.at_war(v.me, p.id))
            && let Some(settler) =
                v.units
                    .iter()
                    .filter(|e| {
                        e.kind == UnitKind::Settler
                            && !v.same_team(v.me, e.owner)
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
                                .any(|u| u.military && u.hp >= 75 && u.pos.distance(e.pos) == 1)
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
            self.stage_raider(settler.pos);
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
        let ready = self.phase == Phase::Conquer
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.ranged > 0 && u.hp >= 60 && u.pos.distance(city.pos) <= 4)
                .count()
                >= 3
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    u.military
                        && u.ranged == 0
                        && u.kind != UnitKind::Scout
                        && u.hp >= 60
                        && u.pos.distance(city.pos) <= 4
                })
                .count()
                >= 2;
        // A visible unguarded settler can be stolen on the next turn. Never assume its
        // future position, and never open a second front for this opportunity.
        let settler = self.phase != Phase::Expand
            && v.units.iter().any(|e| {
                e.owner == city.owner
                    && e.kind == UnitKind::Settler
                    && !v.units.iter().any(|guard| {
                        guard.military
                            && v.same_team(guard.owner, e.owner)
                            && guard.pos.distance(e.pos) <= 1
                    })
                    && self
                        .ctx
                        .own_units
                        .iter()
                        .any(|u| u.military && u.hp >= 75 && u.pos.distance(e.pos) <= 1)
            });
        let order = Order::DeclareWar {
            against: v.team_of(city.owner),
            breach: false,
        };
        if (ready || settler) && self.legal(&order) {
            self.orders.push(order);
            if settler
                && let Some(pos) = v
                    .units
                    .iter()
                    .filter(|e| e.owner == city.owner && e.kind == UnitKind::Settler)
                    .map(|e| e.pos)
                    .find(|pos| {
                        self.ctx
                            .own_units
                            .iter()
                            .any(|u| u.military && u.hp >= 75 && u.pos.distance(*pos) == 1)
                    })
            {
                self.stage_raider(pos);
            }
        }
    }

    fn stage_raider(&mut self, settler: Hex) {
        if let Some(u) = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.hp >= 75 && u.pos.distance(settler) == 1)
            .max_by_key(|u| (u.max_moves, Reverse(u.id)))
        {
            self.orders.push(Order::SkipUnit { unit: u.id });
            self.reserved.insert((true, u.pos));
            self.committed.insert(u.id);
        }
    }

    pub(super) fn city_incoming(&self, u: &UnitView, pos: Hex) -> i32 {
        self.ctx
            .enemy_cities
            .iter()
            .filter(|c| {
                pos != u.pos || legatum_core::city_strike_target(self.ctx.view, c) == Some(u.id)
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
                let defended = self
                    .ctx
                    .enemy_units
                    .iter()
                    .any(|e| e.military && e.pos == city.pos);
                let damage = if defended {
                    0
                } else {
                    self.damage.get(&city.pos).copied().unwrap_or(0)
                };
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
                // A city that took no damage this turn heals CITY_HEAL and fires again; a struck
                // one heals nothing (core `heal_cities`), so the volley's HP stays gone.
                let mut projected = (*city).clone();
                projected.hp = (city.hp - damage).max(0);
                if damage == 0 {
                    projected.hp = projected
                        .hp
                        .saturating_add(legatum_core::data::CITY_HEAL)
                        .min(city.max_hp);
                }
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
        let (center, radius) = if let Some(c) = target.filter(|_| self.phase != Phase::Expand) {
            let at_war = self.ctx.view.at_war(u.owner, c.owner);
            let massed = self
                .ctx
                .own_units
                .iter()
                .filter(|v| v.ranged > 0 && v.hp >= 50 && v.pos.distance(c.pos) <= 5)
                .count()
                >= 3;
            let radius = if !at_war || !massed {
                3
            } else if u.ranged > 0 {
                u.range
            } else if c.hp - self.damage.get(&c.pos).copied().unwrap_or(0) > 120 {
                2
            } else {
                1
            };
            (c.pos, radius)
        } else if let Some(c) = self.ctx.nearest_own_city(u.pos) {
            // Leave the city tile clear for purchases and new production.
            (c.pos, 1)
        } else {
            return false;
        };
        let field = self.ctx.distance_field(u.pos, false);
        let mut spots: Vec<_> = center
            .ring(radius)
            .into_iter()
            .filter(|h| self.free(u, *h))
            .filter_map(|h| {
                let distance = field[self.ctx.view.index(h)?];
                (distance != UNREACHABLE).then(|| (self.incoming(u, h) * 3 + distance * 8, h))
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
        false
    }
}

//! Campaign and border movement decisions use the same fogged snapshot as combat.
use super::*;
use legatum_core::diplomacy::TreatyKind;

impl Plan<'_> {
    // Staging at radius three cannot see the center with ordinary two-tile sight.
    // A known weak city may justify a probe; its remembered defenses are not a
    // promise that hidden reinforcements are absent. Reassess after moving in.
    fn lightly_defended(&self, city: &CityView) -> bool {
        !city.walls
            && city.strength <= 26
            && !self.ctx.view.units.iter().any(|u| {
                u.military
                    && self.ctx.view.same_team(u.owner, city.owner)
                    && u.pos.distance(city.pos) <= 4
            })
    }

    fn infantry_assault(&self, city: &CityView) -> bool {
        let infantry: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.ranged == 0
                    && u.kind != UnitKind::Scout
                    && u.hp >= 80
                    && u.pos.distance(city.pos) <= 4
            })
            .collect();
        self.lightly_defended(city)
            && infantry.len() >= 2
            && infantry
                .iter()
                .map(|u| u.strength * u.hp / 100)
                .sum::<i32>()
                >= city.strength * 2 + 20
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
        if matches!(self.phase, Phase::Expand | Phase::Consolidate) {
            self.target = None;
            return;
        }
        if self.target.is_some_and(|id| {
            v.cities.iter().any(|c| {
                c.id == id
                    && c.owner != v.me
                    && !v.same_team(v.me, c.owner)
                    && (!v.turn.is_multiple_of(12) || c.visible && c.hp < c.max_hp)
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
                    // Neighbours within a march in a crowded lobby; any known city in a duel.
                    (self.duel || home.pos.distance(c.pos) <= 14)
                        && find_path(&self.map, home.pos, c.pos, |h| {
                            let tile = v.tile(h)?;
                            (tile.explored
                                && (v.passage_allowed(
                                    v.me,
                                    legatum_core::UnitKind::Soldier,
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
                let army_distance = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| u.military && u.kind != UnitKind::Scout && u.hp >= 60)
                    .map(|u| u.pos.distance(c.pos))
                    .min()
                    .unwrap_or(home_distance);
                let defenders: i32 = v
                    .units
                    .iter()
                    .filter(|u| {
                        u.military && v.same_team(u.owner, c.owner) && u.pos.distance(c.pos) <= 4
                    })
                    .map(|u| u.strength.max(u.ranged) * u.hp / 100)
                    .sum();
                (
                    home_distance * 6
                        + army_distance * 10
                        + c.strength * 2
                        + c.hp / 5
                        + defenders * 2
                        - i32::from(c.visible && defenders == 0) * 80
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
                accept: p.kind == TreatyKind::Peace
                    && self.phase == Phase::Consolidate
                    && !self.goal_pending(Faction::Sparta),
                explanation: None,
            };
            if self.legal(&order) {
                self.orders.push(order);
            }
        }
        if self.phase == Phase::Consolidate {
            if self.goal_pending(Faction::Sparta) {
                return;
            }
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
                                            legatum_core::UnitKind::Soldier,
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
        let ready = (self.phase == Phase::Conquer || self.lightly_defended(city))
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.ranged > 0 && u.hp >= 60 && u.pos.distance(city.pos) <= 4)
                .count()
                >= if self.lightly_defended(city) { 2 } else { 3 }
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
        // The Sparta objective needs two home guards plus a healthy veteran
        // abroad. Do not wait for those home guards to join the expedition.
        let expedition = self.goal_pending(Faction::Sparta)
            && self.ctx.own_cities.iter().filter(|c| c.walls).count() >= 2
            && self.ctx.own_units.iter().any(|u| {
                legatum_core::data::is_veteran_infantry(u.kind)
                    && u.hp >= 80
                    && u.pos.distance(city.pos) <= 5
            });
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
        if (ready || settler || expedition || self.infantry_assault(city)) && self.legal(&order) {
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
            let shooters = self
                .ctx
                .own_units
                .iter()
                .filter(|v| v.ranged > 0 && v.hp >= 65 && v.pos.distance(c.pos) <= 4)
                .count();
            let infantry = self
                .ctx
                .own_units
                .iter()
                .filter(|v| {
                    v.ranged == 0
                        && v.military
                        && v.kind != UnitKind::Scout
                        && v.hp >= 65
                        && v.pos.distance(c.pos) <= 4
                })
                .count();
            let massed = (shooters >= if self.lightly_defended(c) { 2 } else { 3 }
                && infantry >= 2)
                || self.infantry_assault(c);
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
            let radius =
                if !at_war || !(massed || exposed || engaged && shooters >= 2 && infantry >= 1) {
                    3
                } else if u.ranged > 0 {
                    u.range
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
        // In a duel the one rival's city is the only target, so a crowded staging ring must
        // not strand the rest of the army: spill onto the next ring, still within four.
        let rings = if self.duel && radius == 3 { 2 } else { 1 };
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
        false
    }
}

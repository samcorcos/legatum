//! Resolve the public phase ordering before assigning capture and movement orders.
use super::*;

impl Plan<'_> {
    /// An upgraded capture unit is part of funding the wave. Do not save the
    /// entire purchase reserve while sending obsolete infantry into city fire.
    pub(super) fn upgrade(&mut self, u: &UnitView) -> bool {
        let Some(cost) = u.upgrade_cost else {
            return false;
        };
        let reserve = 30 + (-self.ctx.view.economy.gold_per_turn.total).max(0) * 12;
        if self.phase == Phase::Expand || self.gold < cost + reserve {
            return false;
        }
        self.gold -= cost;
        self.reserved.insert((true, u.pos));
        self.orders.push(Order::UpgradeUnit { unit: u.id });
        true
    }

    /// Scout raids follow only visible settlers; a remembered sighting never
    /// becomes a live target. At Peace, stage outside borders for a legal war.
    pub(super) fn stalk_settler(&mut self, u: &UnitView) -> bool {
        if u.hp < 75 {
            return false;
        }
        let v = self.ctx.view;
        let mut targets: Vec<_> = v
            .units
            .iter()
            .filter(|e| {
                e.kind == UnitKind::Settler
                    && !v.same_team(u.owner, e.owner)
                    && e.pos.distance(u.pos) <= 6
                    && (v.at_war(u.owner, e.owner)
                        || self.ctx.own_cities.len() >= 2
                            && !v.players.iter().any(|p| p.alive && v.at_war(u.owner, p.id))
                            && self.legal(&Order::DeclareWar {
                                against: v.team_of(e.owner),
                                breach: false,
                            }))
                    && !v.units.iter().any(|guard| {
                        guard.military
                            && v.same_team(guard.owner, e.owner)
                            && guard.pos.distance(e.pos) <= 2
                    })
                    && !v
                        .cities
                        .iter()
                        .any(|c| v.same_team(c.owner, e.owner) && c.pos.distance(e.pos) <= 2)
            })
            .map(|e| e.pos)
            .collect();
        targets.sort_by_key(|h| (u.pos.distance(*h), *h));
        for target in targets {
            let mut staging: Vec<_> = target
                .neighbors()
                .into_iter()
                .filter(|h| self.free(u, *h) && self.exposure(*h) == 0)
                .collect();
            staging.sort_by_key(|h| (u.pos.distance(*h), *h));
            for h in staging {
                if h == u.pos {
                    self.hold(u);
                    return true;
                }
                if self.go(u, h, false) {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn one_turn_path(&self, u: &UnitView, to: Hex, attack: bool) -> bool {
        self.path(u, to, attack).is_some_and(|path| {
            let mut moves = u.max_moves;
            path.iter().all(|h| {
                let cost = self.ctx.move_cost(*h).expect("known path");
                let legal = can_step(moves, u.max_moves, cost);
                moves = (moves - cost).max(0);
                legal
            })
        })
    }

    /// All shots hit the same initial defender (core `combat::city_tile_defender`): a city
    /// with HP absorbs the volley and none of it spills into its garrison; at 0 HP the
    /// garrison takes it. Our minimum rolls and incoming maximum rolls bound the
    /// follow-up; enemy orders, reinforcements and fog remain unknown.
    pub(super) fn after_volley(&self) -> PlayerView {
        let mut projected = self.ctx.view.clone();
        for u in &mut projected.units {
            if u.owner == self.ctx.me && u.military {
                u.hp -= self.ranged_incoming(u);
            }
        }
        for (h, damage) in &self.damage {
            crate::analysis::damage_tile_defender(&mut projected, *h, *damage);
        }
        projected.units.retain(|u| u.hp > 0);
        projected
    }

    pub(super) fn finish_captures(&mut self) {
        let projected = self.after_volley();
        let mut captures = Vec::new();
        for c in self.ctx.enemy_cities.iter().filter(|c| c.visible) {
            // A sure melee win captures only against the city's last layer: an empty
            // city, or the exposed garrison of a fallen one.
            let shielded = projected.units.iter().any(|u| {
                u.military && u.pos == c.pos && self.ctx.view.at_war(self.ctx.me, u.owner)
            }) && projected
                .cities
                .iter()
                .any(|city| city.id == c.id && city.hp > 0);
            if shielded {
                continue;
            }
            for u in self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.ranged == 0)
            {
                if self.committed.contains(&u.id) || !self.one_turn_path(u, c.pos, true) {
                    continue;
                }
                let Some(f) = forecast_attack_mode(&projected, u.id, c.pos, AttackMode::Melee)
                else {
                    continue;
                };
                if f.win_chance == 100 && f.loss_chance == 0 {
                    captures.push((
                        c.id,
                        Reverse(f.attacker_hp - f.expected_damage_to_attacker),
                        u.id,
                        c.pos,
                    ));
                }
            }
        }
        captures.sort();
        let mut assigned = BTreeSet::new();
        for (city, _, unit, pos) in captures {
            if assigned.contains(&city) || self.committed.contains(&unit) {
                continue;
            }
            let u = self
                .ctx
                .own_units
                .iter()
                .find(|u| u.id == unit)
                .expect("own infantry");
            if self.go(u, pos, true) {
                self.committed.insert(unit);
                assigned.insert(city);
            }
        }
        // Adjacent infantry attacks resolve in the same movement round. Combine
        // their guaranteed damage when no single unit can finish the city alone.
        // Longer paths are excluded: arriving in different rounds is not a volley.
        for c in self
            .ctx
            .enemy_cities
            .clone()
            .into_iter()
            .filter(|c| c.visible)
        {
            if assigned.contains(&c.id)
                || projected.units.iter().any(|u| {
                    u.military && u.pos == c.pos && self.ctx.view.at_war(self.ctx.me, u.owner)
                })
            {
                continue;
            }
            let mut wave = Vec::new();
            for u in self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.ranged == 0 && u.pos.distance(c.pos) == 1)
            {
                if self.committed.contains(&u.id) || !self.one_turn_path(u, c.pos, true) {
                    continue;
                }
                let Some(f) = forecast_attack_mode(&projected, u.id, c.pos, AttackMode::Melee)
                else {
                    continue;
                };
                let loss = legatum_core::combat::damage(
                    f.defender_strength.total,
                    f.attacker_strength.total,
                    120,
                );
                if loss + 10 >= f.attacker_hp {
                    continue;
                }
                let dealt = legatum_core::combat::damage(
                    f.attacker_strength.total,
                    f.defender_strength.total,
                    80,
                );
                wave.push((Reverse(dealt), u.id));
            }
            wave.sort();
            let hp = projected
                .cities
                .iter()
                .find(|city| city.id == c.id)
                .expect("known city")
                .hp;
            if wave.iter().map(|(Reverse(d), _)| *d).sum::<i32>() < hp {
                continue;
            }
            let mut dealt = 0;
            for (Reverse(damage), id) in wave {
                let u = self
                    .ctx
                    .own_units
                    .iter()
                    .find(|u| u.id == id)
                    .expect("own infantry");
                if self.go(u, c.pos, true) {
                    self.committed.insert(id);
                    dealt += damage;
                }
                if dealt >= hp {
                    break;
                }
            }
        }
    }
}

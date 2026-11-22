//! The army: ranged fire, defense of threatened cities, garrisons, attacks when the local
//! strength ratio beats the aggression threshold, settler escorts, campaigns on enemy cities,
//! and resting.

use crate::ai_v1_baseline::analysis::{
    attack_power, city_power, defense_power, offense_power, threat_power,
};
use crate::ai_v1_baseline::scripted::Planner;
use legatum_core::combat;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{Hex, Order, UnitKind, forecast_attack};

/// Army units this far from a threatened city are called back to defend it.
const DEFEND_RADIUS: i32 = 6;
/// Units within this distance of a target count toward the local strength ratio.
const STRIKE_RADIUS: i32 = 3;
/// Idle units this far from a city come back to it.
const HOME_RADIUS: i32 = 2;
/// Wounded units below this HP don't march on enemy cities.
const MARCH_MIN_HP: i32 = 60;
/// Empires below this aggression never start campaigns (they still strike nearby targets).
pub(crate) const CAMPAIGN_MIN_AGGRESSION: i32 = 30;
/// Enemy cities farther than this from all our cities are not campaign targets.
const CAMPAIGN_RANGE: i32 = 14;

struct Target {
    pos: Hex,
    /// Defensive weight at the target, including nearby support.
    their: i32,
    /// A lone civilian: capture it with one melee unit.
    civilian: bool,
}

impl Planner<'_> {
    pub fn plan_military(&mut self) {
        let army: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .collect();
        self.ranged_fire(&army);
        self.defend_cities(&army);
        self.garrison_cities(&army);
        self.attack_targets(&army);
        self.escort_settlers(&army);
        self.campaign(&army);
        self.rest(&army);
    }

    fn free<'u>(&self, army: &[&'u UnitView]) -> Vec<&'u UnitView> {
        army.iter()
            .copied()
            .filter(|u| !self.has_acted(u.id))
            .collect()
    }

    /// A tile the unit may stand on: not reserved, and not held by another friendly military
    /// unit or anything hostile.
    fn free_tile_for(&self, u: &UnitView, h: Hex) -> bool {
        if h == u.pos {
            return !self.mil_dest.contains(&h);
        }
        let Some(i) = self.ctx.view.index(h) else {
            return false;
        };
        self.ctx.passable_known(h)
            && !self.mil_dest.contains(&h)
            && !self.ctx.friendly_military_at[i]
            && !self.ctx.hostile_at[i]
    }

    fn hold_or_upgrade(&mut self, u: &UnitView) {
        self.mil_dest.insert(u.pos);
        let reserve = self.gold_reserve();
        if let Some(cost) = u.upgrade_cost
            && self.gold_left - cost >= reserve
        {
            self.gold_left -= cost;
            self.unit_order(u.id, Order::UpgradeUnit { unit: u.id });
        } else {
            self.unit_order(u.id, Order::SkipUnit { unit: u.id });
        }
    }

    fn go_to(&mut self, u: &UnitView, to: Hex) {
        if to == u.pos {
            self.hold_or_upgrade(u);
        } else {
            self.mil_dest.insert(to);
            self.move_or_hold(u, to);
        }
    }

    /// Rangers shoot the best visible target in range: a likely kill first, then military
    /// units, then the most damage relative to the target's HP.
    fn ranged_fire(&mut self, army: &[&UnitView]) {
        for u in self.free(army) {
            if u.ranged <= 0 || u.range <= 0 {
                continue;
            }
            let own = combat::unit_ranged_strength(u.kind, u.hp).total;
            let mut best: Option<((i32, i32, i32), Hex)> = None;
            let mut consider = |key: (i32, i32, i32), pos: Hex| {
                if best.is_none_or(|(b, bp)| key > b || (key == b && pos < bp)) {
                    best = Some((key, pos));
                }
            };
            // Ranged attacks hit military units and cities only (not civilians).
            for e in &self.ctx.enemy_units {
                let d = u.pos.distance(e.pos);
                if !e.military || d < 1 || d > u.range {
                    continue;
                }
                let terrain = self
                    .ctx
                    .view
                    .tile(e.pos)
                    .and_then(|t| t.terrain)
                    .unwrap_or(legatum_core::Terrain::Grassland);
                let def = combat::unit_defense_strength(e.kind, e.hp, terrain, e.fortify).total;
                let dmg = combat::base_damage(own, def);
                let kill = (dmg >= e.hp) as i32;
                consider((kill, e.military as i32, dmg * 100 / e.hp.max(1)), e.pos);
            }
            for c in &self.ctx.enemy_cities {
                let d = u.pos.distance(c.pos);
                if !c.visible
                    || d < 1
                    || d > u.range
                    || self.ctx.enemy_units.iter().any(|e| e.pos == c.pos)
                {
                    continue;
                }
                let def = combat::city_defense_strength(c.population, c.walls, c.hp).total;
                let dmg = combat::base_damage(own, def);
                consider((0, 0, dmg * 50 / c.hp.max(1)), c.pos);
            }
            if let Some((_, target)) = best {
                self.mil_dest.insert(u.pos);
                self.unit_order(u.id, Order::RangedAttack { unit: u.id, target });
            }
        }
    }

    /// The best adjacent enemy this unit can profitably attack on its own, if any.
    fn best_strike(&self, u: &UnitView) -> Option<Hex> {
        let threshold = self.knobs.attack_threshold();
        let mine = attack_power(u);
        let mut best: Option<(i32, Hex)> = None;
        for e in &self.ctx.enemy_units {
            if u.pos.distance(e.pos) != 1 || self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos) {
                continue;
            }
            let their = if e.military {
                defense_power(self.ctx.view, e)
            } else {
                0
            };
            let ratio = mine * 100 / their.max(1);
            if ratio >= threshold
                && best.is_none_or(|(b, bp)| ratio > b || (ratio == b && e.pos < bp))
            {
                best = Some((ratio, e.pos));
            }
        }
        best.map(|(_, h)| h)
    }

    /// Threatened cities (most threatened first) pull in the nearest free units until the
    /// defenders' weight matches the threat.
    fn defend_cities(&mut self, army: &[&UnitView]) {
        let mut order: Vec<(i32, &CityView)> = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .filter(|(i, _)| self.ctx.threat[*i] > 0)
            .map(|(i, c)| (self.ctx.threat[i], *c))
            .collect();
        order.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.id.cmp(&b.1.id)));
        for (threat, city) in order {
            let mut have = city_power(city) / 2
                + army
                    .iter()
                    .filter(|u| self.has_acted(u.id) && u.pos.distance(city.pos) <= 2)
                    .map(|u| defense_power(self.ctx.view, u))
                    .sum::<i32>();
            let mut near: Vec<&UnitView> = self
                .free(army)
                .into_iter()
                .filter(|u| u.pos.distance(city.pos) <= DEFEND_RADIUS)
                .collect();
            near.sort_by_key(|u| (u.pos.distance(city.pos), u.id));
            for u in near {
                if have >= threat {
                    break;
                }
                have += defense_power(self.ctx.view, u).max(attack_power(u));
                self.defend(u, city);
            }
        }
    }

    fn defend(&mut self, u: &UnitView, city: &CityView) {
        if u.pos != city.pos
            && let Some(target) = self.best_strike(u)
        {
            self.unit_order(
                u.id,
                Order::MoveUnit {
                    unit: u.id,
                    to: target,
                },
            );
            return;
        }
        if self.free_tile_for(u, city.pos) {
            self.go_to(u, city.pos);
            return;
        }
        if u.pos.distance(city.pos) == 1 && self.free_tile_for(u, u.pos) {
            self.hold_or_upgrade(u);
            return;
        }
        let mut spots: Vec<Hex> = city
            .pos
            .neighbors()
            .into_iter()
            .filter(|h| self.free_tile_for(u, *h))
            .collect();
        spots.sort_by_key(|h| {
            let bonus = self
                .ctx
                .view
                .tile(*h)
                .and_then(|t| t.terrain)
                .map_or(0, |t| t.defense_bonus());
            (-bonus, h.distance(u.pos), *h)
        });
        match spots.first() {
            Some(&h) => self.go_to(u, h),
            None => self.hold_or_upgrade(u),
        }
    }

    /// Every city keeps one unit on its tile: the one already there, else the nearest free one.
    fn garrison_cities(&mut self, army: &[&UnitView]) {
        let cities: Vec<&CityView> = self.ctx.own_cities.clone();
        for city in cities {
            if self.mil_dest.contains(&city.pos) {
                continue;
            }
            if let Some(u) = self.free(army).into_iter().find(|u| u.pos == city.pos) {
                self.hold_or_upgrade(u);
                continue;
            }
            let occupied = self
                .ctx
                .view
                .index(city.pos)
                .is_some_and(|i| self.ctx.friendly_military_at[i]);
            if occupied {
                continue;
            }
            let nearest = self
                .free(army)
                .into_iter()
                .filter(|u| u.pos.distance(city.pos) <= 5)
                .min_by_key(|u| (u.pos.distance(city.pos), u.id));
            if let Some(u) = nearest {
                self.go_to(u, city.pos);
            }
        }
    }

    fn targets(&self) -> Vec<Target> {
        let ctx = &self.ctx;
        let support = |pos: Hex| -> i32 {
            ctx.enemy_units
                .iter()
                .filter(|o| o.military && o.pos != pos && o.pos.distance(pos) <= 2)
                .map(|o| threat_power(o) / 2)
                .sum()
        };
        let mut out = Vec::new();
        for e in &ctx.enemy_units {
            if ctx.enemy_cities.iter().any(|c| c.pos == e.pos) {
                continue;
            }
            let own = if e.military {
                defense_power(ctx.view, e)
            } else {
                0
            };
            let civilian = !e.military
                && ctx
                    .enemy_units
                    .iter()
                    .all(|o| o.pos != e.pos || !o.military);
            out.push(Target {
                pos: e.pos,
                their: own + support(e.pos),
                civilian,
            });
        }
        for c in &ctx.enemy_cities {
            let garrison = ctx
                .enemy_units
                .iter()
                .find(|e| e.pos == c.pos && e.military);
            let own = match garrison {
                Some(g) => defense_power(ctx.view, g) + city_power(c) / 4,
                None => city_power(c),
            };
            out.push(Target {
                pos: c.pos,
                their: own + support(c.pos),
                civilian: false,
            });
        }
        out
    }

    /// Attack wherever the free units near a target outweigh it by the aggression threshold.
    fn attack_targets(&mut self, army: &[&UnitView]) {
        let threshold = self.knobs.attack_threshold();
        let mut candidates: Vec<(i32, Target)> = Vec::new();
        for t in self.targets() {
            let mine: i32 = self
                .free(army)
                .iter()
                .filter(|u| u.pos.distance(t.pos) <= STRIKE_RADIUS)
                .map(|u| offense_power(u))
                .sum();
            if mine == 0 {
                continue;
            }
            let ratio = mine * 100 / t.their.max(1);
            if ratio >= threshold {
                candidates.push((ratio, t));
            }
        }
        candidates.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.pos.cmp(&b.1.pos)));
        for (_, t) in candidates {
            let mut near: Vec<&UnitView> = self
                .free(army)
                .into_iter()
                .filter(|u| u.pos.distance(t.pos) <= STRIKE_RADIUS)
                .collect();
            near.sort_by_key(|u| (u.pos.distance(t.pos), u.id));
            let mine: i32 = near.iter().map(|u| offense_power(u)).sum();
            if mine == 0 || mine * 100 / t.their.max(1) < threshold {
                continue;
            }
            if t.civilian {
                if let Some(u) = near.into_iter().find(|u| u.ranged == 0) {
                    self.unit_order(
                        u.id,
                        Order::MoveUnit {
                            unit: u.id,
                            to: t.pos,
                        },
                    );
                }
                continue;
            }
            for u in near {
                self.strike(u, t.pos);
            }
        }
    }

    /// Whether this unit attacking `target` alone would likely just die (exact forecast).
    fn too_risky(&self, u: &UnitView, target: Hex) -> bool {
        let max_loss = 40 + self.knobs.aggression / 3;
        forecast_attack(self.ctx.view, u.id, target)
            .is_some_and(|f| f.loss_chance as i32 > max_loss && f.win_chance < 50)
    }

    fn strike(&mut self, u: &UnitView, target: Hex) {
        if u.ranged <= 0 {
            let d = u.pos.distance(target);
            if d <= 2 {
                if d == 1 && self.too_risky(u, target) {
                    self.hold_or_upgrade(u);
                } else {
                    self.unit_order(
                        u.id,
                        Order::MoveUnit {
                            unit: u.id,
                            to: target,
                        },
                    );
                }
                return;
            }
            // Close in on a free tile next to the target rather than all converging on the
            // target's own tile (friendly units stepping into one tile all bounce).
            let spot = target
                .ring(1)
                .into_iter()
                .filter(|h| self.free_tile_for(u, *h))
                .min_by_key(|h| (h.distance(u.pos), *h));
            match spot {
                Some(h) => self.go_to(u, h),
                None => self.go_to(u, target),
            }
            return;
        }
        let d = u.pos.distance(target);
        let visible = self.ctx.view.tile(target).is_some_and(|t| t.visible);
        if d >= 1 && d <= u.range && visible {
            self.mil_dest.insert(u.pos);
            self.unit_order(u.id, Order::RangedAttack { unit: u.id, target });
            return;
        }
        // Get in range without walking into melee.
        let spot = target
            .ring(u.range)
            .into_iter()
            .filter(|h| self.free_tile_for(u, *h))
            .min_by_key(|h| (h.distance(u.pos), *h));
        match spot {
            Some(h) => self.go_to(u, h),
            None => self.hold_or_upgrade(u),
        }
    }

    /// A free unit accompanies each settler headed away from home.
    fn escort_settlers(&mut self, army: &[&UnitView]) {
        let sites = self.escort_sites.clone();
        for site in sites {
            let far = self
                .ctx
                .nearest_own_city(site)
                .is_none_or(|c| c.pos.distance(site) > 2);
            if !far || !self.free_tile_for_any(site) {
                continue;
            }
            let escort = self
                .free(army)
                .into_iter()
                .filter(|u| u.pos.distance(site) <= 6)
                .min_by_key(|u| (u.pos.distance(site), u.id));
            if let Some(u) = escort {
                self.go_to(u, site);
            }
        }
    }

    fn free_tile_for_any(&self, h: Hex) -> bool {
        self.ctx.view.index(h).is_some_and(|i| {
            !self.mil_dest.contains(&h)
                && !self.ctx.friendly_military_at[i]
                && !self.ctx.hostile_at[i]
        })
    }

    /// The enemy city to campaign against: the weakest one (defense weight, including its
    /// garrison and nearby support) close to our cities, if our whole free army could take it.
    pub(crate) fn campaign_target(&self, force: i32) -> Option<Hex> {
        if self.knobs.aggression < CAMPAIGN_MIN_AGGRESSION || self.ctx.own_cities.is_empty() {
            return None;
        }
        let threshold = self.knobs.attack_threshold();
        let cities: Vec<Hex> = self.ctx.enemy_cities.iter().map(|c| c.pos).collect();
        self.targets()
            .into_iter()
            .filter(|t| cities.contains(&t.pos))
            .filter_map(|t| {
                let d = self
                    .ctx
                    .own_cities
                    .iter()
                    .map(|c| c.pos.distance(t.pos))
                    .min()?;
                let winnable = force * 100 >= threshold * t.their.max(1);
                (d <= CAMPAIGN_RANGE && winnable).then_some((t.their * (8 + d), t.pos))
            })
            .min()
            .map(|(_, h)| h)
    }

    /// With enough aggression and no city left undefended, healthy free units gather on
    /// distinct tiles just outside strike range of the weakest nearby enemy city they could
    /// take together; `attack_targets` storms it once enough of them have arrived.
    fn campaign(&mut self, army: &[&UnitView]) {
        let undefended = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .any(|(i, c)| self.ctx.threat[i] > city_power(c));
        if undefended {
            return;
        }
        let free: Vec<&UnitView> = self
            .free(army)
            .into_iter()
            .filter(|u| u.hp >= MARCH_MIN_HP)
            .collect();
        // Units holding this turn only to break a bounce still belong to the group.
        let yielding: Vec<&UnitView> = army
            .iter()
            .copied()
            .filter(|u| self.has_acted(u.id) && Self::stuck(u) && u.hp >= MARCH_MIN_HP)
            .collect();
        let force: i32 = free.iter().chain(&yielding).map(|u| offense_power(u)).sum();
        if free.is_empty() || free.len() + yielding.len() < 2 {
            return;
        }
        let Some(target) = self.campaign_target(force) else {
            return;
        };
        for u in free {
            if u.pos.distance(target) <= STRIKE_RADIUS {
                if self.free_tile_for(u, u.pos) {
                    self.hold_or_upgrade(u);
                }
                continue;
            }
            let spot = target
                .ring(STRIKE_RADIUS)
                .into_iter()
                .filter(|h| self.free_tile_for(u, *h))
                .min_by_key(|h| (h.distance(u.pos), *h));
            match spot {
                Some(h) => self.go_to(u, h),
                None => self.go_to(u, target),
            }
        }
    }

    /// Everything left rests near home (healing and fortifying).
    fn rest(&mut self, army: &[&UnitView]) {
        for u in self.free(army) {
            let Some(city) = self.ctx.nearest_own_city(u.pos) else {
                self.hold_or_upgrade(u);
                continue;
            };
            if u.pos.distance(city.pos) <= HOME_RADIUS && self.free_tile_for(u, u.pos) {
                self.hold_or_upgrade(u);
                continue;
            }
            let spot = city
                .pos
                .within(1)
                .into_iter()
                .filter(|h| self.free_tile_for(u, *h))
                .min_by_key(|h| (h.distance(u.pos), *h));
            match spot {
                Some(h) => self.go_to(u, h),
                None => self.hold_or_upgrade(u),
            }
        }
    }
}

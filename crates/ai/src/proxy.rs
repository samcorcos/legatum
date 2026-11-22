//! Fixed human-proxy benchmark policy, deliberately separate from ScriptedBrain's knobs and
//! planner. Expand to seven cities, build a science/production base, then take cities with
//! ranged fire and melee follow-up. No difficulty bonuses, hidden state or persistent planner state.
//! Shared helpers derive geography from PlayerView and forecast the public combat rules only.

use crate::Brain;
use crate::analysis::{Ctx, UNREACHABLE};
use legatum_core::data::{MIN_CITY_GAP, unit_stats};
use legatum_core::path::find_path;
use legatum_core::rules::movement::can_step;
use legatum_core::state::Map;
use legatum_core::view::{CityView, ProductionOption, UnitView};
use legatum_core::{
    AttackMode, BuildingKind, Faction, Hex, Order, PlayerView, ProductionItem, TechId, Terrain,
    UnitKind, forecast_attack_mode,
};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod tests;

/// Bump whenever the fixed benchmark policy changes; observers record this in every run.
pub const PROXY_V2_VERSION: u32 = 2;

#[derive(Default)]
pub struct ProxyV2Brain;

impl Brain for ProxyV2Brain {
    fn plan(&mut self, view: &PlayerView) -> Vec<Order> {
        if view.finished {
            return Vec::new();
        }
        let mut p = Plan {
            ctx: v2_context(view),
            map: Map::new(view.width, view.height, Terrain::Grassland),
            orders: Vec::new(),
            reserved: BTreeSet::new(),
            sites: Vec::new(),
            damage: BTreeMap::new(),
            incoming_cache: RefCell::new(BTreeMap::new()),
            fired: BTreeSet::new(),
            gold: view.economy.gold,
        };
        p.research();
        p.volley();
        // Fire before melee: the latter can see this turn's committed volleys.
        let mut units = p.ctx.own_units.clone();
        units.sort_by_key(|u| (Reverse(u.ranged > 0), u.id));
        for u in units {
            if p.fired.contains(&u.id) {
                continue;
            }
            match u.kind {
                UnitKind::Settler => p.settler(u),
                UnitKind::Scout => p.explore(u),
                _ => p.army(u),
            }
        }
        p.cities();
        p.orders
    }
}

/// Keep the calibrated v2 geography frozen when ScriptedBrain adopts the shared
/// settlement advisor. In particular v2 did not infer unseen cities from borders.
/// This is the original Ctx site calculation, copied without changing its arithmetic.
fn v2_context(view: &PlayerView) -> Ctx<'_> {
    let mut ctx = Ctx::for_proxy(view);
    let scores: Vec<Option<i32>> = view
        .tiles
        .iter()
        .enumerate()
        .map(|(i, tile)| {
            let terrain = tile.terrain?;
            let pos = ctx.hex_at(i);
            if !tile.explored
                || !terrain.is_land()
                || terrain.move_cost().is_none()
                || tile.owner.is_some_and(|owner| owner != view.me)
                || view.failed_city_sites.contains(&pos)
                || view
                    .cities
                    .iter()
                    .any(|c| c.pos.distance(pos) <= MIN_CITY_GAP)
            {
                return None;
            }
            let mut score = 0;
            for h in pos.within(legatum_core::data::CITY_WORK_RADIUS) {
                let Some(t) = view.tile(h) else { continue };
                if t.owner.is_some_and(|owner| owner != view.me) {
                    score -= 8;
                    continue;
                }
                if let Some(terrain) = t.terrain
                    && t.explored
                    && terrain.is_land()
                {
                    score += legatum_core::economy::tile_score(
                        legatum_core::data::terrain_yields(view.faction_of(view.me), terrain),
                        legatum_core::CityFocus::Balanced,
                    );
                }
            }
            Some(score)
        })
        .collect();
    ctx.site_score.set(scores);
    ctx
}

struct Plan<'a> {
    ctx: Ctx<'a>,
    // Only dimensions: all path costs come from the fogged view.
    map: Map,
    orders: Vec<Order>,
    reserved: BTreeSet<(bool, Hex)>,
    sites: Vec<Hex>,
    damage: BTreeMap<Hex, i32>,
    // Turn-local memoization, cleared whenever the committed ranged damage changes.
    incoming_cache: RefCell<BTreeMap<(legatum_core::UnitId, Hex), i32>>,
    fired: BTreeSet<legatum_core::UnitId>,
    gold: i32,
}

impl Plan<'_> {
    /// Coordinate the whole ranged phase before choosing retreats. A wounded shooter
    /// need not flee from a counterattack that the combined volley will prevent.
    fn volley(&mut self) {
        let targets: BTreeSet<_> = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| e.military)
            .map(|e| e.pos)
            .chain(
                self.ctx
                    .enemy_cities
                    .iter()
                    .filter(|c| c.visible)
                    .map(|c| c.pos),
            )
            .collect();
        let mut shots = BTreeMap::new();
        for u in self.ctx.own_units.iter().filter(|u| u.ranged > 0) {
            let best = targets
                .iter()
                .filter_map(|h| {
                    let f = forecast_attack_mode(self.ctx.view, u.id, *h, AttackMode::Ranged)?;
                    let remaining = f.defender_hp - self.damage.get(h).copied().unwrap_or(0);
                    if remaining <= 0 {
                        return None;
                    }
                    let military = self
                        .ctx
                        .enemy_units
                        .iter()
                        .any(|e| e.pos == *h && e.military);
                    let score =
                        i32::from(military) * 100 + f.expected_damage_to_defender * 100 / remaining;
                    let damage = legatum_core::combat::damage(
                        f.attacker_strength.total,
                        f.defender_strength.total,
                        80,
                    );
                    Some(((score, Reverse(*h)), *h, damage))
                })
                .max_by_key(|x| x.0);
            if let Some((_, h, damage)) = best {
                shots.insert(u.id, (h, damage));
                *self.damage.entry(h).or_default() += damage;
                self.incoming_cache.borrow_mut().clear();
            }
        }
        loop {
            let retreat = self.ctx.own_units.iter().find(|u| {
                if !shots.contains_key(&u.id) {
                    return false;
                }
                let unavoidable = self.ranged_incoming(u);
                let total = self.incoming(u, u.pos);
                // Moving follows simultaneous fire. Do not waste a finishing
                // shot trying to outrun an unavoidable volley.
                unavoidable < u.hp && total > unavoidable && total >= u.hp - 15
            });
            let Some(u) = retreat else {
                break;
            };
            let (h, damage) = shots.remove(&u.id).expect("planned shot");
            *self.damage.get_mut(&h).expect("volley damage") -= damage;
            self.incoming_cache.borrow_mut().clear();
        }
        for (unit, (target, _)) in shots {
            let u = self
                .ctx
                .own_units
                .iter()
                .find(|u| u.id == unit)
                .expect("own shooter");
            self.reserved.insert((true, u.pos));
            self.fired.insert(unit);
            self.orders.push(Order::RangedAttack { unit, target });
        }
    }

    fn goal_pending(&self, faction: Faction) -> bool {
        self.ctx
            .view
            .players
            .iter()
            .any(|p| p.id == self.ctx.me && p.faction == faction && !p.faction_goal_done)
    }

    fn research(&mut self) {
        if self.ctx.view.research.current.is_some() {
            return;
        }
        use TechId::*;
        let mut priorities = vec![
            Irrigation,
            BronzeWorking,
            Writing,
            Masonry,
            Feudalism,
            Currency,
            Philosophy,
            IronWorking,
            CodeOfLaws,
            Electricity,
            Philosophy,
            Engineering,
            Machinery,
            Education,
            CropRotation,
            Banking,
            Industrialization,
            Gunpowder,
        ];
        if self.goal_pending(Faction::Byzantium) || self.goal_pending(Faction::Athens) {
            let science_goal = if self.goal_pending(Faction::Athens) {
                vec![Philosophy, IronWorking, Electricity, CodeOfLaws, Philosophy]
            } else {
                vec![Philosophy]
            };
            priorities.splice(5..5, science_goal);
        }
        if let Some(t) = self.ctx.view.research.available.iter().min_by_key(|t| {
            (
                priorities.iter().position(|p| *p == t.tech).unwrap_or(100),
                t.cost.total,
                t.tech,
            )
        }) {
            self.orders.push(Order::SetResearch { tech: t.tech });
        }
    }

    fn cities(&mut self) {
        let mut pipeline = BTreeMap::<UnitKind, usize>::new();
        for u in &self.ctx.own_units {
            *pipeline.entry(u.kind).or_default() += 1;
        }
        for c in &self.ctx.own_cities {
            if let Some(ProductionItem::Unit { unit }) = c.detail.as_ref().and_then(|d| d.producing)
            {
                *pipeline.entry(unit).or_default() += 1;
            }
        }
        let room = self.ctx.site_score.iter().enumerate().any(|(i, score)| {
            score.is_some_and(|s| s >= 60)
                && !self.ctx.in_danger(self.ctx.hex_at(i))
                && self.ctx.own_cities.iter().any(|c| {
                    c.pos.distance(self.ctx.hex_at(i)) <= 9
                        && self.ctx.reachable(c.pos, self.ctx.hex_at(i))
                })
        });
        for c in self.ctx.own_cities.clone() {
            let d = c.detail.as_ref().expect("own detail");
            if d.resistance > 0 {
                continue;
            }
            // Finish investments already started instead of resetting the plan every turn.
            let emergency = self.emergency_defender(c);
            let work = self.great_work(c);
            let option = if emergency.is_some() {
                emergency
            } else if work.is_some() {
                work
            } else if let Some(item) = d.producing {
                d.options.iter().find(|o| o.item == item)
            } else {
                d.options.iter().filter(|o| o.turns > 0).max_by_key(|o| {
                    (
                        self.production_priority(c, o, &pipeline, room),
                        Reverse(o.cost),
                        Reverse(o.item),
                    )
                })
            };
            let Some(o) = option else { continue };
            if d.producing != Some(o.item) {
                if let Some(ProductionItem::Unit { unit }) = d.producing {
                    *pipeline.entry(unit).or_default() -= 1;
                }
                self.orders.push(Order::SetProduction {
                    city: c.id,
                    item: o.item,
                });
                if let ProductionItem::Unit { unit } = o.item {
                    *pipeline.entry(unit).or_default() += 1;
                }
            }
            // Work the treasury, but leave enough for several turns of existing upkeep.
            let reserve = (-self.ctx.view.economy.gold_per_turn.total).max(0) * 5
                + if emergency.is_some() { 10 } else { 50 };
            let saving_wins = self.goal_pending(Faction::Switzerland)
                && self
                    .ctx
                    .view
                    .standings
                    .iter()
                    .any(|s| s.team == self.ctx.view.team && s.vp + 2 >= s.target);
            if (o.turns > 2 || (emergency.is_some() && o.turns > 1))
                && !saving_wins
                && o.buy_block.is_none()
                && let Some(cost) = o.buy_cost
                && cost <= self.gold - reserve
            {
                self.orders.push(Order::BuyItem {
                    city: c.id,
                    item: o.item,
                });
                self.gold -= cost;
            }
        }
    }

    fn great_work<'c>(&self, city: &'c CityView) -> Option<&'c ProductionOption> {
        if self.ctx.own_cities.iter().any(|c| c.detail.as_ref().and_then(|d| d.producing).is_some_and(legatum_core::data::is_great_work_item))
            || self.orders.iter().any(|o| matches!(o, Order::SetProduction { item, .. } if legatum_core::data::is_great_work_item(*item)))
        { return None; }
        let host = self
            .ctx
            .own_cities
            .iter()
            .filter_map(|c| {
                let d = c.detail.as_ref()?;
                if d.resistance > 0 {
                    return None;
                }
                let option = d
                    .options
                    .iter()
                    .filter(|o| legatum_core::data::is_great_work_item(o.item) && o.turns > 0)
                    .min_by_key(|o| (o.turns, o.item))?;
                Some((option.turns, c.id, option.item))
            })
            .min()?;
        if host.1 != city.id {
            return None;
        }
        city.detail
            .as_ref()?
            .options
            .iter()
            .find(|o| o.item == host.2)
    }

    fn emergency_defender<'c>(&self, city: &'c CityView) -> Option<&'c ProductionOption> {
        let enemy: i32 = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| e.military && e.kind != UnitKind::Scout && e.pos.distance(city.pos) <= 3)
            .map(|e| e.strength.max(e.ranged) * e.hp / 100)
            .sum();
        let friendly: i32 = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(city.pos) <= 3)
            .map(|u| u.strength.max(u.ranged) * u.hp / 100)
            .sum();
        if enemy <= friendly {
            return None;
        }
        city.detail.as_ref()?.options.iter().filter(|o| o.buy_cost.is_some_and(|cost| cost <= self.gold - 10))
            .filter(|o| matches!(o.item, ProductionItem::Unit { unit } if unit != UnitKind::Scout && unit != UnitKind::Settler))
            .max_by_key(|o| {
                let ProductionItem::Unit { unit } = o.item else { unreachable!() };
                let stats = legatum_core::data::unit_stats_for(self.ctx.view.faction_of(self.ctx.me), unit);
                (stats.strength.max(stats.ranged), Reverse(o.buy_cost), Reverse(unit))
            })
    }

    fn production_priority(
        &self,
        c: &CityView,
        o: &ProductionOption,
        counts: &BTreeMap<UnitKind, usize>,
        room: bool,
    ) -> i32 {
        use BuildingKind::*;
        use UnitKind::*;
        let count = |u| counts.get(&u).copied().unwrap_or(0);
        let cities = self.ctx.own_cities.len();
        let ranged = count(Archer) + count(Crossbowman);
        let melee = count(Spearman) + count(Pikeman) + count(Musketman);
        let threatened = self
            .ctx
            .enemy_units
            .iter()
            .any(|e| e.military && e.pos.distance(c.pos) <= 4);
        let army_needed = ranged + melee < (cities + 3).clamp(6, 14);
        match o.item {
            ProductionItem::Project { .. } => 0,
            // A human fills improvements in around the main plan.
            ProductionItem::Improvement { .. } => 160,
            ProductionItem::Unit { unit: Settler } => {
                if room
                    && (c.population >= 2 || o.yields.total.food > 2)
                    && cities + count(Settler) < 7
                    && count(Settler) < 3
                {
                    if cities + count(Settler) < 4 {
                        450
                    } else {
                        180
                    }
                } else {
                    -100
                }
            }
            ProductionItem::Unit { unit: Scout } => {
                if count(Scout) == 0 && !self.ctx.frontier.is_empty() {
                    65
                } else {
                    -100
                }
            }
            ProductionItem::Unit { unit } => {
                if self.goal_pending(Faction::Sparta)
                    && cities >= 4
                    && count(Pikeman) + count(Musketman) < 3
                    && matches!(unit, Pikeman | Musketman)
                {
                    return 240;
                }
                if !army_needed || self.ctx.view.economy.gold_per_turn.total < -2 {
                    return -50;
                }
                let want_ranged = ranged < melee * 2;
                let correct = (unit_stats(unit).ranged > 0) == want_ranged;
                80 + i32::from(correct) * 25
                    + i32::from(ranged < 2 && unit_stats(unit).ranged > 0) * 420
                    + i32::from(melee == 0 && unit_stats(unit).ranged == 0) * 420
                    + i32::from(threatened) * 70
                    + match unit {
                        Crossbowman | Musketman => 6,
                        Pikeman => 3,
                        _ => 0,
                    }
            }
            ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                match building {
                    GreatLibrary | RoyalArsenal | GrandBazaar => -100,
                    Workshop => 350,
                    Granary => {
                        if c.is_capital && self.goal_pending(Faction::Germany) {
                            220
                        } else {
                            400
                        }
                    }
                    Library => 340,
                    Market => 330,
                    University => 110,
                    Mill => {
                        if c.is_capital && self.goal_pending(Faction::Germany) {
                            190
                        } else {
                            70
                        }
                    }
                    Forge => 68,
                    Bank => 62,
                    Walls => {
                        if (c.is_capital && self.goal_pending(Faction::Germany))
                            || (cities >= 4 && self.goal_pending(Faction::Sparta))
                        {
                            175
                        } else if threatened {
                            85
                        } else {
                            10
                        }
                    }
                    RiceStorehouse => {
                        if c.is_capital && self.goal_pending(Faction::Germany) {
                            170
                        } else {
                            30
                        }
                    }
                    _ => 5,
                }
            }
        }
    }

    fn exposure(&self, h: Hex) -> i32 {
        self.ctx
            .enemy_units
            .iter()
            .filter(|e| e.military && e.kind != UnitKind::Scout)
            .map(|e| {
                let d = h.distance(e.pos);
                if e.ranged > 0 && d <= e.range {
                    e.ranged
                } else if d <= e.max_moves && self.ctx.reachable(e.pos, h) {
                    e.strength
                } else {
                    0
                }
            })
            .sum()
    }

    /// Worst-roll damage from visible enemies able to attack this arrival. Friendly
    /// screens block routes; unseen troops and hidden orders are deliberately unknown.
    fn incoming(&self, u: &UnitView, h: Hex) -> i32 {
        let key = (u.id, h);
        if let Some(damage) = self.incoming_cache.borrow().get(&key) {
            return *damage;
        }
        let damage = self.incoming_uncached(u, h);
        self.incoming_cache.borrow_mut().insert(key, damage);
        damage
    }

    fn ranged_incoming(&self, u: &UnitView) -> i32 {
        let terrain = self
            .ctx
            .view
            .tile(u.pos)
            .and_then(|t| t.terrain)
            .unwrap_or(Terrain::Grassland);
        let defense = legatum_core::combat::unit_defense_total(
            self.ctx.view.faction_of(u.owner),
            u.kind,
            u.hp,
            terrain,
            u.fortify,
            u.level,
        );
        self.ctx
            .enemy_units
            .iter()
            .filter(|e| e.ranged > 0 && e.pos.distance(u.pos) <= e.range)
            .map(|e| {
                legatum_core::combat::damage(
                    legatum_core::combat::unit_ranged_total(
                        self.ctx.view.faction_of(e.owner),
                        e.kind,
                        e.hp,
                        e.level,
                    ),
                    defense,
                    120,
                )
            })
            .sum()
    }

    fn incoming_uncached(&self, u: &UnitView, h: Hex) -> i32 {
        let terrain = self
            .ctx
            .view
            .tile(h)
            .and_then(|t| t.terrain)
            .unwrap_or(Terrain::Grassland);
        let defense = legatum_core::combat::unit_defense_total(
            self.ctx.view.faction_of(u.owner),
            u.kind,
            u.hp,
            terrain,
            if h == u.pos { u.fortify } else { 0 },
            u.level,
        );
        self.ctx
            .enemy_units
            .iter()
            .filter(|e| e.military)
            .map(|e| {
                let hp = e.hp - self.damage.get(&e.pos).copied().unwrap_or(0);
                let faction = self.ctx.view.faction_of(e.owner);
                if e.ranged > 0 && e.pos.distance(h) <= e.range && (hp > 0 || h == u.pos) {
                    return legatum_core::combat::damage(
                        legatum_core::combat::unit_ranged_total(
                            faction,
                            e.kind,
                            if h == u.pos { e.hp } else { hp },
                            e.level,
                        ),
                        defense,
                        120,
                    );
                }
                if hp <= 0 || e.pos.distance(h) > e.max_moves {
                    return 0;
                }
                let path = find_path(&self.map, e.pos, h, |p| {
                    if p != h
                        && self
                            .ctx
                            .view
                            .units
                            .iter()
                            .any(|v| v.military && v.pos == p && v.id != u.id)
                    {
                        return None;
                    }
                    self.ctx.move_cost(p)
                });
                let reachable = path.is_some_and(|p| {
                    let mut moves = e.max_moves;
                    p.iter().all(|p| {
                        let cost = self.ctx.move_cost(*p).unwrap_or(100);
                        let ok = can_step(moves, e.max_moves, cost);
                        moves = (moves - cost).max(0);
                        ok
                    })
                });
                if reachable {
                    legatum_core::combat::damage(
                        legatum_core::combat::unit_attack_total(faction, e.kind, hp, e.level),
                        defense,
                        120,
                    )
                } else {
                    0
                }
            })
            .sum()
    }

    fn retreat(&mut self, u: &UnitView) -> bool {
        let incoming = self.incoming(u, u.pos);
        let mut spots: Vec<_> = u
            .pos
            .within(u.max_moves)
            .into_iter()
            .filter(|h| self.free(u, *h))
            .filter_map(|h| {
                let damage = self.incoming(u, h);
                let path = self.path(u, h, false)?;
                let mut moves = u.max_moves;
                let reachable = path.iter().all(|p| {
                    let cost = self.ctx.move_cost(*p).unwrap_or(100);
                    let ok = can_step(moves, u.max_moves, cost);
                    moves = (moves - cost).max(0);
                    ok
                });
                (reachable && damage < incoming).then_some((
                    damage,
                    Reverse(
                        self.ctx
                            .view
                            .tile(h)
                            .is_some_and(|t| t.owner == Some(u.owner)),
                    ),
                    h,
                ))
            })
            .collect();
        spots.sort();
        for (_, _, h) in spots {
            if self.go(u, h, false) {
                return true;
            }
        }
        false
    }

    fn free(&self, u: &UnitView, h: Hex) -> bool {
        self.ctx.move_cost(h).is_some()
            && !self.reserved.contains(&(u.military, h))
            && !self
                .ctx
                .view
                .units
                .iter()
                .any(|v| v.id != u.id && v.pos == h && v.military == u.military)
    }

    fn path(&self, u: &UnitView, to: Hex, attack: bool) -> Option<Vec<Hex>> {
        find_path(&self.map, u.pos, to, |h| {
            let i = self.ctx.view.index(h)?;
            let hostile = self.ctx.hostile_at[i];
            let defended = self
                .ctx
                .enemy_units
                .iter()
                .any(|e| e.military && e.pos == h)
                || self.ctx.enemy_cities.iter().any(|c| c.pos == h && c.hp > 0);
            if attack && h == to && (!self.reserved.contains(&(u.military, h)) || defended) {
                return self.ctx.move_cost(h);
            }
            self.ctx
                .move_cost(h)
                .filter(|_| self.free(u, h) && !hostile && (u.military || !self.ctx.in_danger(h)))
        })
    }

    fn go(&mut self, u: &UnitView, to: Hex, attack: bool) -> bool {
        if !attack
            && self.ctx.view.cities.iter().any(|c| {
                c.pos == to
                    && !self.ctx.view.same_team(u.owner, c.owner)
                    && self.ctx.view.allied(u.owner, c.owner)
            })
        {
            return false;
        }
        let Some(path) = self.path(u, to, attack) else {
            return false;
        };
        let mut moves = u.max_moves;
        let mut end = u.pos;
        let mut steps = Vec::new();
        for h in path {
            let cost = self.ctx.move_cost(h).expect("known path");
            if !can_step(moves, u.max_moves, cost) {
                break;
            }
            moves = (moves - cost).max(0);
            steps.push(h);
            end = h;
        }
        if end == u.pos {
            return false;
        }
        if !attack && u.military && u.kind != UnitKind::Scout {
            let incoming = self.incoming(u, end);
            if incoming >= u.hp - 25 && incoming >= self.incoming(u, u.pos) {
                return false;
            }
        }
        self.reserved.insert((u.military, end));
        self.orders.push(Order::MoveUnitPath {
            unit: u.id,
            to: end,
            path: steps,
            stop_on_block: false,
        });
        true
    }

    fn hold(&mut self, u: &UnitView) {
        self.reserved.insert((u.military, u.pos));
        self.orders.push(Order::SkipUnit { unit: u.id });
    }

    fn recover(&mut self, u: &UnitView) -> bool {
        if u.hp >= 50 {
            return false;
        }
        if self.exposure(u.pos) == 0 {
            self.hold(u);
            return true;
        }
        let mut spots = u
            .pos
            .within(u.max_moves)
            .into_iter()
            .filter(|h| self.free(u, *h) && self.exposure(*h) < self.exposure(u.pos))
            .collect::<Vec<_>>();
        spots.sort_by_key(|h| {
            (
                self.exposure(*h),
                Reverse(
                    self.ctx
                        .view
                        .tile(*h)
                        .is_some_and(|t| t.owner == Some(u.owner)),
                ),
                *h,
            )
        });
        for h in spots {
            if self.go(u, h, false) {
                return true;
            }
        }
        false
    }

    fn settler(&mut self, u: &UnitView) {
        let field = self.ctx.distance_field(u.pos, true);
        let mut sites: Vec<_> = self
            .ctx
            .site_score
            .iter()
            .enumerate()
            .filter_map(|(i, score)| {
                let score = (*score)?;
                let h = self.ctx.hex_at(i);
                if field[i] == UNREACHABLE
                    || self.ctx.in_danger(h)
                    || self.sites.iter().any(|s| s.distance(h) <= MIN_CITY_GAP)
                    || self
                        .ctx
                        .enemy_units
                        .iter()
                        .any(|e| e.kind == UnitKind::Settler && e.pos.distance(h) <= MIN_CITY_GAP)
                {
                    return None;
                }
                // The original radius sum preferred distant future tiles over the
                // first citizens' actual output. Value the center and first ring:
                // productive, growing cities repay their settler much sooner.
                let faction = self.ctx.view.faction_of(u.owner);
                let mut worked: Vec<_> = h
                    .neighbors()
                    .into_iter()
                    .filter_map(|n| self.ctx.view.tile(n))
                    .filter(|t| t.owner.is_none() || t.owner == Some(u.owner))
                    .filter_map(|t| t.terrain)
                    .map(|t| legatum_core::data::terrain_yields(faction, t))
                    .collect();
                worked.sort_by_key(|y| {
                    Reverse(legatum_core::economy::tile_score(
                        *y,
                        legatum_core::CityFocus::Balanced,
                    ))
                });
                let center = legatum_core::data::CITY_CENTER_YIELDS
                    + legatum_core::data::terrain_yields(faction, self.ctx.view.tile(h)?.terrain?);
                let first = worked.iter().take(2).fold(center, |sum, y| sum + *y);
                let value =
                    score / 3 + first.production.max(3) * 45 + (first.food - 4).clamp(-2, 5) * 18;
                let first_city = self.ctx.own_cities.is_empty();
                if first_city && field[i] > u.max_moves * 3 {
                    return None;
                }
                let value = if first_city {
                    value - 20 * ((field[i] + u.max_moves - 1) / u.max_moves)
                } else {
                    value * 100 / (100 + 10 * field[i])
                };
                let value = value
                    + if first_city && self.ctx.view.turn >= 4 && h == u.pos {
                        10000
                    } else {
                        0
                    };
                Some((Reverse(value + i32::from(h == u.pos) * 15), h))
            })
            .collect();
        sites.sort();
        for (_, h) in sites {
            if h == u.pos {
                self.sites.push(h);
                self.orders.push(Order::FoundCity { unit: u.id });
                return;
            }
            if self.go(u, h, false) {
                self.sites.push(h);
                return;
            }
        }
        self.hold(u);
    }

    fn explore(&mut self, u: &UnitView) {
        if u.kind == UnitKind::Scout {
            let mut cities: Vec<_> = self
                .ctx
                .enemy_cities
                .iter()
                .filter(|c| c.visible && c.hp <= 20 && u.pos.distance(c.pos) <= u.max_moves)
                .map(|c| c.pos)
                .collect();
            cities.sort_by_key(|h| (u.pos.distance(*h), *h));
            for h in cities {
                if forecast_attack_mode(self.ctx.view, u.id, h, AttackMode::Melee)
                    .is_some_and(|f| f.win_chance == 100 && f.loss_chance == 0)
                    && self.incoming(u, h) < u.hp - 25
                    && self.go(u, h, true)
                {
                    return;
                }
            }
        }
        if self.capture_settler(u) {
            return;
        }
        if u.kind == UnitKind::Scout && self.exposure(u.pos) > 0 {
            let mut safe: Vec<_> = u
                .pos
                .within(u.max_moves)
                .into_iter()
                .filter(|h| self.free(u, *h) && self.exposure(*h) == 0)
                .collect();
            safe.sort_by_key(|h| (u.pos.distance(*h), *h));
            for h in safe {
                if self.path(u, h, false).is_some_and(|p| {
                    p.iter()
                        .map(|h| self.ctx.move_cost(*h).unwrap_or(100))
                        .sum::<i32>()
                        <= u.max_moves
                }) && self.go(u, h, false)
                {
                    return;
                }
            }
        }
        if self.recover(u) {
            return;
        }
        let field = self.ctx.distance_field(u.pos, true);
        let mut frontier: Vec<_> = self
            .ctx
            .frontier
            .iter()
            .copied()
            .filter(|h| *h != u.pos && self.exposure(*h) == 0)
            .filter_map(|h| {
                let d = field[self.ctx.view.index(h)?];
                let ruin = self.ctx.view.tile(h)?.ruin;
                (d != UNREACHABLE).then(|| {
                    (
                        d * 10 - self.ctx.unexplored_near(h) * 4 - i32::from(ruin) * 40,
                        h,
                    )
                })
            })
            .collect();
        frontier.sort();
        for (_, h) in frontier {
            if self.go(u, h, false) {
                return;
            }
        }
        self.hold(u);
    }

    fn army(&mut self, u: &UnitView) {
        // Preserve veterans instead of firing from a tile on which the visible
        // counterattack kills them. A retreat must actually arrive safely this turn.
        let danger = self.incoming(u, u.pos);
        if danger >= u.hp - 15 && self.retreat(u) {
            return;
        }
        if let Some(cost) = u.upgrade_cost
            && self.gold >= cost + 20
        {
            self.gold -= cost;
            self.reserved.insert((true, u.pos));
            self.orders.push(Order::UpgradeUnit { unit: u.id });
            return;
        }
        let mut projected = self.ctx.view.clone();
        if u.ranged == 0 {
            // Ranged fire resolves before movement, all volleys from one snapshot.
            // Only guaranteed minimum damage is credited to the melee follow-up.
            for (h, damage) in &self.damage {
                if let Some(enemy) = projected
                    .units
                    .iter_mut()
                    .find(|e| e.pos == *h && e.military && e.owner != self.ctx.me)
                {
                    enemy.hp -= damage;
                } else if let Some(city) = projected.cities.iter_mut().find(|c| c.pos == *h) {
                    city.hp = (city.hp - damage).max(0);
                }
            }
            projected.units.retain(|e| e.hp > 0);
        }
        let mut shots = Vec::new();
        let targets: BTreeSet<_> = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| e.military)
            .map(|e| e.pos)
            .chain(
                self.ctx
                    .enemy_cities
                    .iter()
                    .filter(|c| c.visible)
                    .map(|c| c.pos),
            )
            .collect();
        for h in targets {
            let mode = if u.ranged > 0 {
                AttackMode::Ranged
            } else {
                AttackMode::Melee
            };
            if u.pos.distance(h) > if u.ranged > 0 { u.range } else { u.max_moves } {
                continue;
            }
            let Some(f) = forecast_attack_mode(&projected, u.id, h, mode) else {
                continue;
            };
            let garrison = projected
                .units
                .iter()
                .any(|e| e.military && e.owner != u.owner && e.pos == h);
            let city = projected
                .cities
                .iter()
                .find(|c| c.owner != u.owner && c.pos == h);
            let remaining = f.defender_hp
                - if u.ranged > 0 {
                    self.damage.get(&h).copied().unwrap_or(0)
                } else {
                    0
                };
            if u.ranged > 0 {
                if remaining <= 0 {
                    continue;
                }
            } else {
                let reserve = if f.win_chance >= 90 { 10 } else { 30 };
                if f.loss_chance > 10
                    || f.expected_damage_to_attacker >= u.hp - reserve
                    || (f.expected_damage_to_attacker > f.expected_damage_to_defender
                        && remaining > 30)
                {
                    continue;
                }
                let Some(path) = self.path(u, h, true) else {
                    continue;
                };
                let mut moves = u.max_moves;
                if !path.iter().all(|p| {
                    let cost = self.ctx.move_cost(*p).expect("path");
                    let ok = can_step(moves, u.max_moves, cost);
                    moves = (moves - cost).max(0);
                    ok
                }) {
                    continue;
                }
            }
            let key = (
                i32::from(u.ranged == 0 && city.is_some() && !garrison && f.win_chance >= 90)
                    * 1000
                    + i32::from(garrison) * 100
                    + i32::from(self.ctx.own_cities.iter().any(|c| c.pos.distance(h) <= 3)) * 100
                    + f.expected_damage_to_defender * 100 / remaining.max(1),
                Reverse(h),
            );
            let guaranteed = legatum_core::combat::damage(
                f.attacker_strength.total,
                f.defender_strength.total,
                80,
            );
            shots.push((key, h, guaranteed));
        }
        shots.sort_by_key(|s| Reverse(s.0));
        for (_, h, damage) in shots {
            if u.ranged > 0 {
                self.reserved.insert((true, u.pos));
                self.orders.push(Order::RangedAttack {
                    unit: u.id,
                    target: h,
                });
            } else if !self.go(u, h, true) {
                continue;
            }
            if u.ranged > 0 {
                *self.damage.entry(h).or_default() += damage;
                self.incoming_cache.borrow_mut().clear();
            }
            return;
        }
        if u.hp < 75 && danger == 0 {
            self.hold(u);
            return;
        }
        if self.recover(u) {
            return;
        }
        if self.capture_settler(u) {
            return;
        }
        if self.goal_garrison(u) {
            return;
        }
        if self.intercept(u) {
            return;
        }
        // Hold a fresh capture against visible counterattacks until the city recovers.
        if u.ranged == 0
            && self.ctx.own_cities.iter().any(|c| {
                c.pos == u.pos
                    && c.hp < c.max_hp
                    && self
                        .ctx
                        .enemy_units
                        .iter()
                        .any(|e| e.military && e.pos.distance(c.pos) <= 4)
            })
        {
            self.hold(u);
            return;
        }
        // The starting melee unit is a capture escort, not a solo siege. Keep complementary
        // troops together while marching; an available shot/occupation above always wins.
        let partner = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|v| {
                v.kind != UnitKind::Scout
                    && v.military
                    && v.hp >= 50
                    && (v.ranged > 0) != (u.ranged > 0)
            })
            .min_by_key(|v| (u.pos.distance(v.pos), v.id));
        // Reinforcements join the front; an established pair never walks back to fetch
        // a newly produced partner at home.
        let gathering = self.ctx.own_units.iter().filter(|v| v.ranged > 0).count() < 2
            && self.ctx.view.turn < 80;
        let gather = match partner {
            _ if gathering => self.ctx.nearest_own_city(u.pos).map(|c| c.pos),
            None => self.ctx.nearest_own_city(u.pos).map(|c| c.pos),
            _ => None,
        };
        if let Some(home) = gather {
            if u.pos.distance(home) <= 1 {
                self.hold(u);
                return;
            }
            let mut spots = home.within(1);
            spots.sort_by_key(|h| (u.pos.distance(*h), *h));
            for h in spots {
                if self.free(u, h) && self.go(u, h, false) {
                    return;
                }
            }
            self.hold(u);
            return;
        }
        // One nearby weak city concentrates the army. Re-evaluate only from known cities;
        // remembered cities can be approached, but only visible targets can be fired upon.
        let target = self
            .ctx
            .enemy_cities
            .iter()
            .filter(|c| self.ctx.reachable(u.pos, c.pos))
            .min_by_key(|c| {
                (
                    self.ctx
                        .own_units
                        .iter()
                        .filter(|v| v.ranged > 0)
                        .map(|v| v.pos.distance(c.pos))
                        .min()
                        .unwrap_or(u.pos.distance(c.pos))
                        * 20
                        + self
                            .ctx
                            .own_units
                            .iter()
                            .filter(|v| v.military && v.ranged == 0 && v.kind != UnitKind::Scout)
                            .map(|v| v.pos.distance(c.pos))
                            .min()
                            .unwrap_or(u.pos.distance(c.pos))
                            * 20
                        + c.hp / 5
                        + c.strength
                        - self
                            .ctx
                            .own_units
                            .iter()
                            .filter(|v| {
                                v.military
                                    && v.kind != UnitKind::Scout
                                    && v.pos.distance(c.pos) <= 3
                            })
                            .count() as i32
                            * 15,
                    c.id,
                )
            });
        if let Some(c) = target {
            if let Some(partner) = partner
                && u.pos.distance(partner.pos) > 4
                && u.pos.distance(c.pos) < partner.pos.distance(c.pos)
                && self.ctx.reachable(u.pos, partner.pos)
            {
                self.hold(u);
                return;
            }
            let radius = if u.ranged > 0 { u.range } else { 1 };
            let field = self.ctx.distance_field(u.pos, false);
            let mut spots: Vec<_> = c
                .pos
                .ring(radius)
                .into_iter()
                .filter(|h| self.free(u, *h))
                .filter_map(|h| {
                    let d = field[self.ctx.view.index(h)?];
                    (d != UNREACHABLE).then(|| (self.incoming(u, h) * 3 + d * 8, h))
                })
                .collect();
            spots.sort();
            for (_, h) in spots {
                if h == u.pos {
                    self.hold(u);
                    return;
                }
                if self.go(u, h, false) {
                    return;
                }
            }
        }
        self.explore(u);
    }

    fn capture_settler(&mut self, u: &UnitView) -> bool {
        let mut targets: Vec<_> = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| e.kind == UnitKind::Settler && u.pos.distance(e.pos) <= u.max_moves)
            .map(|e| e.pos)
            .filter(|h| {
                !self.ctx.enemy_cities.iter().any(|c| c.pos == *h)
                    && !self
                        .ctx
                        .enemy_units
                        .iter()
                        .any(|e| e.military && e.pos == *h)
                    && self.incoming(u, *h) < u.hp - 25
            })
            .collect();
        targets.sort_by_key(|h| (u.pos.distance(*h), *h));
        for h in targets {
            if self.path(u, h, true).is_some_and(|path| {
                let mut moves = u.max_moves;
                path.iter().all(|p| {
                    let cost = self.ctx.move_cost(*p).unwrap_or(100);
                    let ok = can_step(moves, u.max_moves, cost);
                    moves = (moves - cost).max(0);
                    ok
                })
            }) && self.go(u, h, true)
            {
                return true;
            }
        }
        false
    }

    fn intercept(&mut self, u: &UnitView) -> bool {
        let threat = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| e.military && e.kind != UnitKind::Scout && e.pos.distance(u.pos) <= 6)
            .filter(|e| {
                self.ctx
                    .own_cities
                    .iter()
                    .any(|c| c.pos.distance(e.pos) <= 3)
            })
            .min_by_key(|e| (u.pos.distance(e.pos), e.id));
        let Some(enemy) = threat else {
            return false;
        };
        let radius = if u.ranged > 0 { u.range } else { 1 };
        let field = self.ctx.distance_field(u.pos, false);
        let mut positions: Vec<_> = enemy
            .pos
            .ring(radius)
            .into_iter()
            .filter(|h| self.free(u, *h))
            .filter_map(|h| {
                let distance = field[self.ctx.view.index(h)?];
                (distance != UNREACHABLE).then(|| (distance * 8 + self.incoming(u, h) * 3, h))
            })
            .collect();
        positions.sort();
        for (_, h) in positions {
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

    fn goal_garrison(&mut self, u: &UnitView) -> bool {
        if !self.goal_pending(Faction::Sparta) || !legatum_core::data::is_veteran_infantry(u.kind) {
            return false;
        }
        let mut assigned = BTreeSet::new();
        for city in self
            .ctx
            .own_cities
            .clone()
            .into_iter()
            .filter(|c| c.walls)
            .take(2)
        {
            let nearest = self
                .ctx
                .own_units
                .iter()
                .filter(|v| {
                    legatum_core::data::is_veteran_infantry(v.kind) && !assigned.contains(&v.id)
                })
                .min_by_key(|v| (v.pos.distance(city.pos), v.id));
            if let Some(v) = nearest {
                assigned.insert(v.id);
                if v.id == u.id {
                    if u.pos == city.pos {
                        self.hold(u);
                        return true;
                    }
                    return self.go(u, city.pos, false);
                }
            }
        }
        false
    }
}

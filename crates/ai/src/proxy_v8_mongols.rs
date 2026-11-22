//! Opt-in Mongols playtest 8.5 proxy: a brief mobile raid, rapid settlement,
//! early Walls, and a protected Courier Hall for the VP race.
//! Forked from frozen v7. All decisions and memory derive exclusively from PlayerView;
//! no engine state, observer feedback, seed inspection or random choices.

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

mod campaign;
mod economy;
mod tactics;

pub const PROXY_V8_MONGOLS_VERSION: u32 = 8;

/// Spearmen wanted: a first wave, then more for every city the army holds.
const GOAL_BASE: usize = 1;
const GOAL_PER_CITY: usize = 0;
const GOAL_MAX: usize = 4;
/// Archers kept per Soldier-goal divisor, and their ceiling; more while the target has Walls.
const RANGED_DIV: usize = 2;
const RANGED_MAX: usize = 2;
const WALL_RANGED: usize = 2;
/// A Settler wave starts once this many Spearmen exist, up to this many cities.
const SETTLE_CITIES: usize = 9;
const SETTLE_PRIORITY: i32 = 900;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    /// Spearmen are still being trained; the army gathers at home.
    #[default]
    Muster,
    /// The army marches on the nearest or weakest rival city, forever.
    Conquer,
    /// The one early raid is over; settlement and held VP decide the game.
    Settle,
}

/// What this brain has seen of one war, from its own view only.
#[derive(Clone, Debug, Default)]
struct WarLog {
    /// The last turn the war showed a gain: a capture, a damaged city or a kill.
    progress: u32,
    /// Own units lost near this rival since the last progress.
    losses: u32,
}

#[derive(Default)]
pub struct HumanMongolsBrain {
    phase: Phase,
    target: Option<legatum_core::CityId>,
    owners: BTreeMap<legatum_core::CityId, legatum_core::PlayerId>,
    city_hp: BTreeMap<legatum_core::CityId, i32>,
    captured_cities: BTreeSet<legatum_core::CityId>,
    original_homes: BTreeSet<legatum_core::CityId>,
    wars: BTreeMap<legatum_core::TeamId, WarLog>,
    /// Rivals we asked for peace: their cities are not targets until the turn given.
    avoid: BTreeMap<legatum_core::TeamId, u32>,
    military: usize,
    captures: usize,
    /// Visible rival military units last turn: where, and whose.
    seen: BTreeMap<legatum_core::UnitId, (Hex, legatum_core::TeamId)>,
}

impl HumanMongolsBrain {
    fn observe(&mut self, view: &PlayerView) {
        for city in &view.cities {
            if city.owner == view.me {
                match self.owners.get(&city.id) {
                    None => {
                        self.original_homes.insert(city.id);
                    }
                    Some(owner) if *owner != view.me && !self.original_homes.contains(&city.id) => {
                        self.captured_cities.insert(city.id);
                    }
                    _ => {}
                }
            }
            self.owners.insert(city.id, city.owner);
        }
        let military = view
            .units
            .iter()
            .filter(|u| u.owner == view.me && u.military && u.kind != UnitKind::Scout)
            .count();
        let lost = self.military.saturating_sub(military) as u32;
        self.military = military;
        let captured = self.captured_cities.len() > self.captures;
        self.captures = self.captured_cities.len();
        let at_war: BTreeSet<_> = view
            .players
            .iter()
            .filter(|p| p.alive && !view.same_team(view.me, p.id) && view.at_war(view.me, p.id))
            .map(|p| p.team)
            .collect();
        // A rival unit that vanished from the middle of our sight was killed, not moved away.
        let killed: Vec<_> = self
            .seen
            .iter()
            .filter(|(id, (pos, _))| {
                !view.units.iter().any(|u| u.id == **id)
                    && pos
                        .within(2)
                        .into_iter()
                        .all(|h| view.tile(h).is_none_or(|t| t.visible))
            })
            .map(|(_, (_, team))| *team)
            .collect();
        self.seen = view
            .units
            .iter()
            .filter(|u| u.military && !view.same_team(view.me, u.owner) && !u.owner.is_barbarian())
            .map(|u| (u.id, (u.pos, view.team_of(u.owner))))
            .collect();
        self.wars.retain(|team, _| at_war.contains(team));
        self.avoid.retain(|_, until| *until > view.turn);
        for team in at_war {
            let log = self.wars.entry(team).or_insert_with(|| WarLog {
                progress: view.turn,
                ..Default::default()
            });
            let enemies: Vec<_> = view
                .units
                .iter()
                .filter(|u| u.military && view.team_of(u.owner) == team)
                .collect();
            let hit = view.cities.iter().any(|c| {
                view.team_of(c.owner) == team
                    && c.visible
                    && self.city_hp.get(&c.id).is_some_and(|hp| c.hp < *hp)
            });
            if hit || captured || killed.contains(&team) {
                log.progress = view.turn;
                log.losses = 0;
            }
            if lost > 0
                && view.units.iter().any(|u| {
                    u.owner == view.me && enemies.iter().any(|e| e.pos.distance(u.pos) <= 4)
                })
            {
                log.losses += lost;
            }
        }
        self.city_hp = view
            .cities
            .iter()
            .filter(|c| c.visible)
            .map(|c| (c.id, c.hp))
            .collect();
    }

    /// A war that has stopped paying: no gain for many turns, or units lost without one.
    fn stalled(&self, view: &PlayerView) -> BTreeSet<legatum_core::TeamId> {
        let patience = 16_u32;
        self.wars
            .iter()
            .filter(|(_, log)| {
                let idle = view.turn.saturating_sub(log.progress);
                idle >= patience || (log.losses >= 2 && idle >= patience / 2)
            })
            .map(|(team, _)| *team)
            .collect()
    }

    fn build_plan<'a>(&self, view: &'a PlayerView) -> Plan<'a> {
        let duel = view.players.len() == 2;
        let mut stalled = if duel {
            BTreeSet::new()
        } else {
            self.stalled(view)
        };
        stalled.extend(self.avoid.keys().copied());
        let mut p = Plan {
            ctx: Ctx::new(view),
            map: Map::new(view.width, view.height, Terrain::Mountains),
            orders: Vec::new(),
            reserved: BTreeSet::new(),
            sites: Vec::new(),
            damage: BTreeMap::new(),
            incoming_cache: RefCell::new(BTreeMap::new()),
            committed: BTreeSet::new(),
            rotating: BTreeSet::new(),
            gold: view.economy.gold,
            phase: self.phase,
            stalled,
            peace_offers: Vec::new(),
            target: self.target,
        };
        for (tile, known) in p.map.tiles.iter_mut().zip(&view.tiles) {
            if let Some(terrain) = known.terrain {
                tile.terrain = terrain;
            }
        }
        p
    }
}

impl Brain for HumanMongolsBrain {
    fn plan(&mut self, view: &PlayerView) -> Vec<Order> {
        if view.finished {
            return Vec::new();
        }
        self.observe(view);
        let infantry = view
            .units
            .iter()
            .filter(|u| {
                u.owner == view.me && u.military && u.ranged == 0 && u.kind != UnitKind::Scout
            })
            .count();
        if self.phase == Phase::Muster && infantry >= 2 {
            self.phase = Phase::Conquer;
        }
        if view.turn >= 35 || !self.captured_cities.is_empty() {
            self.phase = Phase::Settle;
            self.target = None;
        }
        let mut first = self.build_plan(view);
        first.select_campaign();
        first.diplomacy();
        // A surprise war at plain Peace takes effect before movement: plan the strike against
        // the war it opens, so the army declares and attacks in the same turn.
        let declared: Vec<_> = first
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
        let mut war_view;
        let mut p = if declared.is_empty() {
            first
        } else {
            war_view = view.clone();
            for team in &declared {
                use legatum_core::diplomacy::{Relation, RelationState};
                match war_view
                    .diplomacy
                    .relations
                    .iter_mut()
                    .find(|r| r.between(view.team, *team))
                {
                    Some(r) if r.state == RelationState::Peace => r.state = RelationState::War,
                    Some(_) => {}
                    // A missing pair is Peace in a view; the strike plans against its war.
                    None => {
                        let (a, b) = legatum_core::diplomacy::pair(view.team, *team);
                        war_view.diplomacy.relations.push(Relation {
                            a,
                            b,
                            state: RelationState::War,
                            effective_turn: view.turn,
                            guarantee_until: 0,
                            version: 0,
                            security: None,
                            war_id: None,
                            declarers: vec![view.team],
                        });
                    }
                }
            }
            let mut second = self.build_plan(&war_view);
            second.orders = first.orders.clone();
            second.committed = first.committed.clone();
            second.reserved = first.reserved.clone();
            second.target = first.target;
            second.peace_offers = first.peace_offers.clone();
            second
        };
        self.target = p.target;
        for team in p.peace_offers.clone() {
            self.avoid.insert(team, view.turn + 24);
        }
        p.research();
        p.volley();
        p.finish_captures();
        let mut units = p.ctx.own_units.clone();
        // Move capture infantry first, then bring the firing line into its screen.
        units.sort_by_key(|u| (u.ranged > 0, u.id));
        for u in units {
            if p.committed.contains(&u.id) {
                continue;
            }
            match u.kind {
                UnitKind::Settler => p.settler(u),
                UnitKind::Scout => {
                    if !p.capture_settler(u) {
                        p.explore(u);
                    }
                }
                _ => p.army(u),
            }
        }
        p.cities();
        p.buy_infrastructure();
        p.orders
    }
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
    committed: BTreeSet<legatum_core::UnitId>,
    rotating: BTreeSet<legatum_core::UnitId>,
    gold: i32,
    phase: Phase,
    /// Rivals whose war has stalled or whom we have asked for peace.
    stalled: BTreeSet<legatum_core::TeamId>,
    /// Peace offers made this turn.
    peace_offers: Vec<legatum_core::TeamId>,
    target: Option<legatum_core::CityId>,
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
        for u in self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.ranged > 0 && !self.committed.contains(&u.id))
        {
            let best = targets
                .iter()
                .filter_map(|h| {
                    let f = forecast_attack_mode(self.ctx.view, u.id, *h, AttackMode::Ranged)?;
                    let remaining = f.defender_hp - self.damage.get(h).copied().unwrap_or(0);
                    if remaining <= 0 {
                        return None;
                    }
                    // A city with HP shields its garrison: such a shot hits the city.
                    let military = self
                        .ctx
                        .enemy_units
                        .iter()
                        .any(|e| e.pos == *h && e.military)
                        && f.garrison.as_ref().is_none_or(|g| !g.shielded);
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
                unavoidable < u.hp
                    && ((total > unavoidable && total >= u.hp - 15) || self.needs_rotation(u))
            });
            let Some(u) = retreat else {
                break;
            };
            let (h, damage) = shots.remove(&u.id).expect("planned shot");
            self.rotating.insert(u.id);
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
            self.committed.insert(unit);
            self.orders.push(Order::RangedAttack { unit, target });
        }
    }

    /// The playbook is Spearmen. Sparta refill their ranks by killing, so they keep one Archer;
    /// every other faction needs a firing line (an Archer per two Spearmen) to trade fairly.
    fn ranged_goal(&self) -> usize {
        let base = if self.fights_wounded() {
            1
        } else {
            (self.infantry_goal() / RANGED_DIV).clamp(1, RANGED_MAX)
        };
        // Walls make a city fire at whoever stands near it: shoot it from outside.
        if self.walled_target() {
            base.max(WALL_RANGED)
        } else {
            base
        }
    }

    fn walled_target(&self) -> bool {
        self.target
            .is_some_and(|id| self.ctx.view.cities.iter().any(|c| c.id == id && c.walls))
    }

    /// Spearmen grow with the empire: captured cities train them too.
    fn infantry_goal(&self) -> usize {
        if self.ctx.view.turn >= 65 && self.ctx.own_cities.len() >= 5 {
            4
        } else {
            (GOAL_BASE + GOAL_PER_CITY * self.ctx.own_cities.len().saturating_sub(1)).min(GOAL_MAX)
        }
    }

    /// Kills and captures refill an Sparta army, so it fights on while wounded.
    fn fights_wounded(&self) -> bool {
        self.goal_faction(Faction::Sparta)
    }

    fn goal_faction(&self, faction: Faction) -> bool {
        self.ctx
            .view
            .players
            .iter()
            .any(|p| p.id == self.ctx.me && p.faction == faction)
    }

    /// A treasury that cannot carry another unit's upkeep for a dozen turns disbands units at the
    /// front, so the army never outgrows what its income pays for.
    fn can_support_unit(&self) -> bool {
        let military = self.ctx.own_units.iter().filter(|u| u.military).count() as i32;
        let extra = if military >= legatum_core::data::FREE_MILITARY_UNITS {
            legatum_core::data::military_upkeep(self.ctx.view.faction_of(self.ctx.me))
        } else {
            0
        };
        self.gold + (self.ctx.view.economy.gold_per_turn.total - extra) * 8 >= 0
    }

    /// Damage margin below which a veteran retreats.
    fn retreat_margin(&self) -> i32 {
        if self.fights_wounded() { 5 } else { 15 }
    }

    /// Hit points below which a unit rests instead of attacking.
    fn rest_hp(&self) -> i32 {
        if self.fights_wounded() { 30 } else { 75 }
    }

    /// No city of the rival is known, so there is nothing to march on.
    fn rival_unknown(&self) -> bool {
        !self
            .ctx
            .view
            .cities
            .iter()
            .any(|c| !self.ctx.view.same_team(self.ctx.me, c.owner))
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
        let priorities = [
            Masonry,
            Irrigation,
            Currency,
            BronzeWorking,
            Writing,
            CropRotation,
            IronWorking,
            Education,
            Banking,
            Feudalism,
            Engineering,
            Mathematics,
            Philosophy,
            CodeOfLaws,
            Electricity,
            Philosophy,
            Machinery,
            Industrialization,
            Gunpowder,
        ];
        if let Some(t) = self.ctx.view.research.available.iter().min_by_key(|t| {
            let rank = priorities.iter().position(|p| *p == t.tech).unwrap_or(100);
            let rank = if self.goal_pending(Faction::Byzantium) && t.tech == Philosophy {
                rank.min(5)
            } else {
                rank
            };
            (rank, t.cost.total, t.tech)
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
                && self.ctx.own_cities.iter().any(|c| {
                    c.pos.distance(self.ctx.hex_at(i)) <= 15
                        && self.ctx.reachable(c.pos, self.ctx.hex_at(i))
                })
        });
        let count = |ranged: bool| {
            self.ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.kind != UnitKind::Scout && (u.ranged > 0) == ranged)
                .count()
        };
        let mut ranged = count(true);
        let mut melee = count(false);
        let reserve = (-self.ctx.view.economy.gold_per_turn.total).max(0) * 12 + 30;
        let saving_wins = self.goal_pending(Faction::Switzerland)
            && self
                .ctx
                .view
                .standings
                .iter()
                .any(|s| s.team == self.ctx.view.team && s.vp + 2 >= s.target);
        let target = self
            .target
            .and_then(|id| self.ctx.view.cities.iter().find(|c| c.id == id))
            .map(|c| c.pos);
        // Fill the cities nearest the front first.
        let mut cities = self.ctx.own_cities.clone();
        cities.sort_by_key(|c| (target.map_or(0, |h| c.pos.distance(h)), c.id));
        for c in cities {
            let d = c.detail.as_ref().expect("own detail");
            if d.resistance > 0 {
                continue;
            }
            // The source opening banks a Quarry and Granary, then commits the capital to
            // a Farm and five Settlers. Queueing preserves that decision through pop dips.
            if c.is_capital && self.ctx.view.turn == 2 {
                let quarry = d.options.iter().find(|o| {
                    matches!(
                        o.item,
                        ProductionItem::Improvement {
                            improvement: legatum_core::ImprovementKind::Quarry,
                            ..
                        }
                    )
                });
                let granary = d.options.iter().find(|o| {
                    o.item
                        == ProductionItem::Building {
                            building: BuildingKind::Granary,
                        }
                });
                let queue: Vec<_> = quarry.into_iter().chain(granary).map(|o| o.item).collect();
                if !queue.is_empty() {
                    self.orders
                        .push(Order::SetProductionQueue { city: c.id, queue });
                    continue;
                }
            }
            if c.is_capital && self.ctx.view.turn == 12 {
                let farm = d.options.iter().find(|o| {
                    matches!(
                        o.item,
                        ProductionItem::Improvement {
                            improvement: legatum_core::ImprovementKind::Farm,
                            ..
                        }
                    )
                });
                let settler = d.options.iter().find(|o| {
                    o.item
                        == ProductionItem::Unit {
                            unit: UnitKind::Settler,
                        }
                });
                if let Some(settler) = settler {
                    let mut queue: Vec<_> = farm.into_iter().map(|o| o.item).collect();
                    queue.extend(std::iter::repeat_n(settler.item, 5));
                    self.orders
                        .push(Order::SetProductionQueue { city: c.id, queue });
                    continue;
                }
            }
            if c.is_capital
                && self.ctx.view.turn < 70
                && d.production_queue.iter().any(|item| {
                    matches!(
                        item,
                        ProductionItem::Unit {
                            unit: UnitKind::Settler
                        }
                    )
                })
            {
                continue;
            }
            let emergency = self.emergency_defender(c);
            let option = emergency.or_else(|| {
                d.options
                    .iter()
                    .filter(|o| o.turns > 0 && self.production_priority(c, o, &pipeline, room) > 0)
                    .max_by_key(|o| {
                        (
                            self.production_priority(c, o, &pipeline, room)
                                + i32::from(d.producing == Some(o.item)) * 60,
                            Reverse(o.cost),
                            Reverse(o.item),
                        )
                    })
            });
            if let Some(o) = option
                && d.producing != Some(o.item)
            {
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
            if saving_wins || !self.can_support_unit() || self.ctx.view.turn < 85 {
                continue;
            }
            // Gold is spent on Spearmen as fast as the army is short of them.
            let want_ranged = melee >= 2 && ranged < self.ranged_goal();
            if emergency.is_none() && !want_ranged && melee >= self.infantry_goal() {
                continue;
            }
            let buy = d
                .options
                .iter()
                .filter(|o| {
                    o.buy_block.is_none()
                        && o.buy_cost.is_some_and(|cost| cost <= self.gold - reserve)
                })
                .filter(|o| {
                    matches!(o.item, ProductionItem::Unit { unit }
                    if unit != UnitKind::Scout && unit_stats(unit).military
                    && (unit_stats(unit).ranged > 0) == want_ranged)
                })
                .max_by_key(|o| {
                    let ProductionItem::Unit { unit } = o.item else {
                        unreachable!()
                    };
                    let stats = legatum_core::data::unit_stats_for(
                        self.ctx.view.faction_of(self.ctx.me),
                        unit,
                    );
                    (
                        stats.strength.max(stats.ranged)
                            + legatum_core::data::siege_city_bonus(unit),
                        Reverse(o.buy_cost),
                        Reverse(o.item),
                    )
                });
            if let Some(o) = buy {
                self.orders.push(Order::BuyItem {
                    city: c.id,
                    item: o.item,
                });
                self.gold -= o.buy_cost.expect("affordable");
                if want_ranged {
                    ranged += 1;
                } else {
                    melee += 1;
                }
            }
        }
    }

    fn emergency_defender<'c>(&self, city: &'c CityView) -> Option<&'c ProductionOption> {
        let enemy: i32 = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| {
                e.military
                    && !e.owner.is_barbarian()
                    && e.kind != UnitKind::Scout
                    && e.pos.distance(city.pos) <= 3
            })
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
            .any(|e| e.military && !e.owner.is_barbarian() && e.pos.distance(c.pos) <= 4);
        match o.item {
            ProductionItem::Project { .. } => 0,
            // A human fills improvements in around the main plan.
            ProductionItem::Improvement { .. } => 200,
            // A Settler only once rich and safe, if at all: the army is the expansion.
            ProductionItem::Unit { unit: Settler } => {
                let already_building = c
                    .detail
                    .as_ref()
                    .is_some_and(|d| d.producing == Some(o.item));
                if room
                    && cities + count(Settler) < SETTLE_CITIES
                    && (count(Settler) < 3 || already_building)
                    && (c.is_capital || cities >= 4)
                {
                    SETTLE_PRIORITY
                } else {
                    -200
                }
            }
            // A scout finds rivals and unescorted Settlers while the Spearmen train.
            ProductionItem::Unit { unit: Scout } => {
                if count(Scout) == 0 && self.rival_unknown() && melee >= 2 {
                    650
                } else {
                    -100
                }
            }
            ProductionItem::Unit { unit } => {
                let ranged_unit = unit_stats(unit).ranged > 0;
                let needed = if ranged_unit {
                    ranged < self.ranged_goal() && melee >= 2
                } else {
                    melee < self.infantry_goal()
                };
                if !needed || !self.can_support_unit() || (35..65).contains(&self.ctx.view.turn) {
                    return -100;
                }
                let base = if ranged_unit {
                    720
                } else if melee < 3 {
                    800
                } else {
                    740
                };
                base + match unit {
                    Pikeman => 10,
                    Musketman => 15,
                    Crossbowman => 15,
                    _ => 0,
                }
            }
            ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                match building {
                    Market => {
                        if self.ctx.view.economy.gold_per_turn.total < 6 {
                            790
                        } else {
                            700
                        }
                    }
                    Workshop => 660,
                    Library => 710,
                    Granary => {
                        if self.goal_pending(Faction::Celts)
                            || c.is_capital && self.goal_pending(Faction::Germany)
                        {
                            710
                        } else {
                            740
                        }
                    }
                    University => 530,
                    Forge => 450,
                    Mill | Supermarket => 420,
                    Bank | StockExchange => 430,
                    RiceStorehouse => {
                        if c.is_capital && self.goal_pending(Faction::Germany) {
                            705
                        } else {
                            300
                        }
                    }
                    Walls => {
                        if self.ctx.view.turn < 36 {
                            -100
                        } else if !c.walls {
                            if threatened { 980 } else { 830 }
                        } else if c.is_capital && self.goal_pending(Faction::Germany) {
                            695
                        } else if threatened {
                            580
                        } else {
                            100
                        }
                    }
                    Castle => {
                        if threatened {
                            480
                        } else {
                            50
                        }
                    }
                    YamRelayStation => {
                        let first_expansion = self
                            .ctx
                            .own_cities
                            .iter()
                            .filter(|city| !city.is_capital)
                            .map(|city| city.id)
                            .min();
                        if cities >= 5 && self.ctx.view.turn >= 60 && first_expansion == Some(c.id)
                        {
                            980
                        } else {
                            -200
                        }
                    }
                    StarFort | GreatLibrary | Pyramids | HagiaSophia | EiffelTower
                    | IterFusionReactor | RoyalArsenal | GrandBazaar | ImperialDiwan
                    | Roadworks | AirDefence => -200,
                    ResearchLab => 520,
                    Factory => 440,
                    Agoge => 80,
                }
            }
        }
    }

    fn exposure(&self, h: Hex) -> i32 {
        let units: i32 = self
            .ctx
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
            .sum();
        let cities: i32 = self
            .ctx
            .enemy_cities
            .iter()
            .filter(|c| legatum_core::city_fires(c) && c.pos.distance(h) <= 2)
            .map(|c| c.strength)
            .sum();
        units + cities
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

    /// Our city at `h` with HP left, which takes every attack on its tile (core
    /// `combat::city_tile_defender`), and its defense strength.
    fn shielding_city(&self, h: Hex) -> Option<(i32, i32)> {
        let c = self
            .ctx
            .own_cities
            .iter()
            .find(|c| c.pos == h && c.hp > 0)?;
        Some((c.hp, c.strength))
    }

    fn ranged_incoming(&self, u: &UnitView) -> i32 {
        // Ranged fire on a city with HP hits the city, never its garrison.
        if u.military && self.shielding_city(u.pos).is_some() {
            return self.city_incoming(u, u.pos);
        }
        let terrain = self
            .ctx
            .view
            .tile(u.pos)
            .and_then(|t| t.terrain)
            .unwrap_or(Terrain::Grassland);
        let defense = legatum_core::combat::unit_defense_strength_for(
            self.ctx.view.faction_of(u.owner),
            u.kind,
            u.hp,
            terrain,
            u.fortify,
            u.level,
        )
        .total;
        let units: i32 = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| e.ranged > 0 && e.pos.distance(u.pos) <= e.range)
            .map(|e| {
                legatum_core::combat::damage(
                    legatum_core::combat::unit_ranged_strength_for(
                        self.ctx.view.faction_of(e.owner),
                        e.kind,
                        e.hp,
                        e.level,
                    )
                    .total,
                    defense,
                    120,
                )
            })
            .sum();
        units + self.city_incoming(u, u.pos)
    }

    fn incoming_uncached(&self, u: &UnitView, h: Hex) -> i32 {
        let terrain = self
            .ctx
            .view
            .tile(h)
            .and_then(|t| t.terrain)
            .unwrap_or(Terrain::Grassland);
        let defense = legatum_core::combat::unit_defense_strength_for(
            self.ctx.view.faction_of(u.owner),
            u.kind,
            u.hp,
            terrain,
            if h == u.pos { u.fortify } else { 0 },
            u.level,
        )
        .total;
        let shield = self.shielding_city(h).filter(|_| u.military);
        let worst = |attack: i32, vs_city: i32| {
            (
                legatum_core::combat::damage(attack, defense, 120),
                shield.map_or(0, |(_, city)| {
                    legatum_core::combat::damage(vs_city, city, 120)
                }),
            )
        };
        // (damage to the unit, damage to a shielding city, ranged) per visible enemy.
        let attacks: Vec<(i32, i32, bool)> = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| e.military)
            .map(|e| {
                let hp = e.hp - self.damage.get(&e.pos).copied().unwrap_or(0);
                let faction = self.ctx.view.faction_of(e.owner);
                if e.ranged > 0 && e.pos.distance(h) <= e.range && (hp > 0 || h == u.pos) {
                    let attack = legatum_core::combat::unit_ranged_strength_for(
                        faction,
                        e.kind,
                        if h == u.pos { e.hp } else { hp },
                        e.level,
                    )
                    .total;
                    let (to_unit, to_city) = worst(
                        attack,
                        attack + legatum_core::data::siege_city_bonus(e.kind),
                    );
                    return (to_unit, to_city, true);
                }
                if hp <= 0 || e.pos.distance(h) > e.max_moves {
                    return (0, 0, false);
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
                    let attack = legatum_core::combat::unit_attack_strength_for(
                        faction, e.kind, hp, e.level,
                    )
                    .total;
                    let (to_unit, to_city) = worst(attack, attack);
                    (to_unit, to_city, false)
                } else {
                    (0, 0, false)
                }
            })
            .collect();
        // Our city absorbs every attack while it has HP: the garrison is exposed only if
        // the worst case breaks the city this turn, and then only to melee.
        let units: i32 = match shield {
            Some((city_hp, _)) if attacks.iter().map(|a| a.1).sum::<i32>() < city_hp => 0,
            Some(_) => attacks.iter().filter(|a| !a.2).map(|a| a.0).sum(),
            None => attacks.iter().map(|a| a.0).sum(),
        };
        units + self.city_incoming(u, h)
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
        let open = |h| {
            self.ctx
                .view
                .tile(h)
                .is_some_and(|t| self.ctx.view.passage_allowed(u.owner, u.kind, t.owner))
        };
        let exits = (!open(u.pos)).then(|| legatum_core::path::exit_distances(&self.map, open));
        legatum_core::path::find_path_by_step(&self.map, u.pos, to, |from, h| {
            if !legatum_core::path::border_step(&self.map, from, h, open, exits.as_deref()) {
                return None;
            }
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
            self.ctx.move_cost(h).filter(|_| {
                self.free(u, h)
                    && !hostile
                    && !self.ctx.neutral_at[i]
                    && (u.military || !self.ctx.settler_danger[i])
            })
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
        if !attack && u.military {
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
        self.incoming_cache.borrow_mut().clear();
        true
    }

    fn hold(&mut self, u: &UnitView) {
        self.reserved.insert((u.military, u.pos));
        self.orders.push(Order::SkipUnit { unit: u.id });
    }

    fn recover(&mut self, u: &UnitView) -> bool {
        if u.hp >= if self.fights_wounded() { 25 } else { 50 } {
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
        let field = self.ctx.settler_distance_field(u.pos);
        let mut sites: Vec<_> = self
            .ctx
            .site_score
            .iter()
            .enumerate()
            .filter_map(|(i, score)| {
                let score = (*score)?;
                let h = self.ctx.hex_at(i);
                if field[i] == UNREACHABLE
                    || self.ctx.settler_danger[i]
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
                    value * 100 / (100 + 50 * field[i])
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
        // A dead scout or closed border can exhaust the known settlement sites.
        // Reveal more land by walking a known safe route, instead of holding a
        // trained settler at home for the rest of the game.
        let mut frontier: Vec<_> = self
            .ctx
            .frontier
            .iter()
            .copied()
            .filter_map(|h| {
                let i = self.ctx.view.index(h)?;
                let d = field[i];
                (h != u.pos && d != UNREACHABLE && !self.ctx.settler_danger[i])
                    .then(|| (d * 10 - self.ctx.unexplored_near(h) * 3, h))
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
        if u.kind == UnitKind::Scout && self.stalk_settler(u) {
            return;
        }
        // Keep one mobile Scout near a known rival's settlement corridor during the
        // early Settler wave. Fogged frontier exploration otherwise walks every Scout
        // away just before the rival's first Settler leaves home.
        if u.kind == UnitKind::Scout
            && (22..=30).contains(&self.ctx.view.turn)
            && self
                .ctx
                .own_units
                .iter()
                .filter(|v| v.kind == UnitKind::Scout)
                .map(|v| v.id)
                .nth(1)
                .or_else(|| {
                    self.ctx
                        .own_units
                        .iter()
                        .filter(|v| v.kind == UnitKind::Scout)
                        .map(|v| v.id)
                        .min()
                })
                == Some(u.id)
            && let Some(home) = self.ctx.own_cities.iter().find(|c| c.is_capital)
            && let Some(rival) = self
                .ctx
                .view
                .cities
                .iter()
                .filter(|c| !self.ctx.view.same_team(c.owner, home.owner))
                .filter(|c| (5..=12).contains(&home.pos.distance(c.pos)))
                .min_by_key(|c| (u.pos.distance(c.pos), c.id))
        {
            let mut watch: Vec<_> = rival
                .pos
                .within(2)
                .into_iter()
                .filter(|h| (1..=2).contains(&h.distance(rival.pos)))
                .filter(|h| h.distance(home.pos) < rival.pos.distance(home.pos))
                .filter(|h| self.free(u, *h) && self.exposure(*h) == 0)
                .collect();
            watch.sort_by_key(|h| (h.distance(rival.pos), Reverse(h.r), u.pos.distance(*h), *h));
            for h in watch {
                if h == u.pos {
                    self.hold(u);
                    return;
                }
                if self.go(u, h, false) {
                    return;
                }
            }
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
        if self.capture_settler(u) {
            return;
        }
        // Preserve veterans instead of firing from a tile on which the visible
        // counterattack kills them. A retreat must actually arrive safely this turn.
        let danger = self.incoming(u, u.pos);
        if (danger >= u.hp - self.retreat_margin()
            || self.rotating.contains(&u.id)
            || self.needs_rotation(u))
            && self.retreat(u)
        {
            return;
        }
        if self.upgrade(u) {
            return;
        }
        let projected = if u.ranged == 0 {
            self.after_volley()
        } else {
            self.ctx.view.clone()
        };
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
            // The defender is a unit (a field unit, or the garrison of a fallen city);
            // a city with HP shields its garrison (core `combat::city_tile_defender`).
            let garrison = projected
                .units
                .iter()
                .any(|e| e.military && e.owner != u.owner && e.pos == h)
                && f.garrison.as_ref().is_none_or(|g| !g.shielded);
            // A melee win captures when it beats the city's last layer.
            let last_layer = f.garrison.as_ref().is_none_or(|g| !g.shielded);
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
                // A city assault trades hit points for the city; an Sparta army that
                // fights on while wounded accepts any attack it survives.
                let assault = city.is_some() && self.assault_force(h) >= 2;
                let reserve = if self.fights_wounded() {
                    0
                } else if f.win_chance >= 90 {
                    10
                } else {
                    30
                };
                let loss_cap = if assault || self.fights_wounded() {
                    25
                } else {
                    10
                };
                if f.loss_chance > loss_cap
                    || f.expected_damage_to_attacker >= u.hp - reserve
                    || (!assault
                        && f.expected_damage_to_attacker > f.expected_damage_to_defender
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
                i32::from(u.ranged == 0 && city.is_some() && last_layer && f.win_chance >= 90)
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
        let besieging = self.target.is_some_and(|id| {
            self.ctx.view.cities.iter().any(|c| {
                c.id == id && self.ctx.view.at_war(u.owner, c.owner) && c.pos.distance(u.pos) <= 2
            })
        });
        if u.hp < self.rest_hp() && danger == 0 && !(besieging && u.hp >= 40) {
            self.hold(u);
            return;
        }
        if self.recover(u) {
            return;
        }
        if self.capture_settler(u) {
            return;
        }
        if self.guard_city(u) {
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
        if self.seek_rival(u) {
            self.explore(u);
            return;
        }
        if u.ranged == 0 && self.stalk_settler(u) {
            return;
        }
        if self.march(u) {
            return;
        }
        self.explore(u);
    }

    /// With no rival city known (the scouts are dead or never found it), one infantryman
    /// looks for it while the rest of the army waits: a campaign needs a target.
    fn seek_rival(&self, u: &UnitView) -> bool {
        let mut infantry: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|v| v.military && v.ranged == 0 && v.kind != UnitKind::Scout)
            .map(|v| v.id)
            .collect();
        infantry.sort();
        let has_scout = self.ctx.own_units.iter().any(|v| v.kind == UnitKind::Scout);
        self.target.is_none()
            && self.rival_unknown()
            && u.ranged == 0
            && u.kind != UnitKind::Scout
            && infantry
                .iter()
                .take(if has_scout { 1 } else { 2 })
                .any(|id| *id == u.id)
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

    /// Home cities stay lightly guarded: one unit, and only where a rival army is in sight.
    fn guard_city(&mut self, u: &UnitView) -> bool {
        let busy = |v: &UnitView| {
            self.target.is_some_and(|id| {
                self.ctx
                    .view
                    .cities
                    .iter()
                    .any(|c| c.id == id && c.pos.distance(v.pos) <= 3)
            })
        };
        for city in &self.ctx.own_cities {
            let threatened = self.ctx.enemy_units.iter().any(|e| {
                e.military
                    && !e.owner.is_barbarian()
                    && e.kind != UnitKind::Scout
                    && e.pos.distance(city.pos) <= 3
            });
            if !threatened {
                continue;
            }
            if self
                .ctx
                .own_units
                .iter()
                .any(|v| v.military && v.id != u.id && v.pos == city.pos)
            {
                continue;
            }
            let nearest = self
                .ctx
                .own_units
                .iter()
                .filter(|v| {
                    v.military
                        && v.kind != UnitKind::Scout
                        && !self.committed.contains(&v.id)
                        && !busy(v)
                        && v.pos.distance(city.pos) <= 8
                })
                .min_by_key(|v| (v.pos.distance(city.pos), v.id));
            if nearest.is_some_and(|v| v.id == u.id) {
                if u.pos == city.pos {
                    self.hold(u);
                    return true;
                }
                return self.go(u, city.pos, false);
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
}

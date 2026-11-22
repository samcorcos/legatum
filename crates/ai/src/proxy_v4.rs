//! Alliance-aware diplomacy for the fixed human proxy. Only PlayerView enters this module; there is no
//! random source, opponent planner, remembered private state or observer feedback.
use crate::analysis::{Ctx, UNREACHABLE};
use crate::{Brain, ProxyV2Brain};
use legatum_core::diplomacy::{Agreement, RelationState, TreatyKind};
use legatum_core::state::Map;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{
    AttackMode, Order, PlayerView, ProductionItem, TeamId, Terrain, UnitKind, forecast_attack_mode,
};
use std::cmp::Reverse;
use std::collections::BTreeSet;

pub const PROXY_VERSION: u32 = 4;

// The declaration budget and the movement reserve must describe the same flank.
const FLANK_RADIUS: i32 = 6;
const RESERVE_RADIUS: i32 = 5;

#[derive(Default)]
pub struct ProxyBrain;

impl Brain for ProxyBrain {
    fn plan(&mut self, view: &PlayerView) -> Vec<Order> {
        let mut orders = ProxyV2Brain.orders(view);
        if view.finished || view.diplomacy.legacy {
            return orders;
        }
        let diplomat = Diplomat { view };
        orders.extend(diplomat.orders());
        diplomat.mobilize(&mut orders);
        diplomat.fund_army(&mut orders);
        diplomat.prepare_finish(&mut orders);
        diplomat.stage_army(&mut orders);
        diplomat.guard_flanks(&mut orders);
        egress(view, &mut orders);
        orders
    }
}

struct Diplomat<'a> {
    view: &'a PlayerView,
}

#[derive(Clone, Copy)]
struct Campaign {
    team: TeamId,
    resistance: i32,
    distance: i32,
    city: legatum_core::CityId,
}

impl Diplomat<'_> {
    /// Do not rush a non-winning two-point project that would make a vulnerable
    /// empire the coalition target. Finish an existing investment (or build a
    /// defender) first. Projects already underway and an immediate victory proceed.
    fn prepare_finish(&self, orders: &mut Vec<Order>) {
        let v = self.view;
        let Some(own) = v.standings.iter().find(|s| s.team == v.team) else {
            return;
        };
        if own.vp + 2 >= own.target
            || legatum_core::diplomacy::coalition::target_view(v) == Some(v.team)
            || legatum_core::diplomacy::coalition::target(
                v.standings.iter().filter(|s| self.alive(s.team)).map(|s| {
                    (
                        s.team,
                        s.vp + if s.team == v.team { 2 } else { 0 },
                        s.target,
                    )
                }),
            )
            .is_none_or(|t| t.0 != v.team)
        {
            return;
        }
        let opposition: i32 = v
            .players
            .iter()
            .filter(|p| p.alive && p.team != v.team)
            .map(|p| p.team)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|t| self.neighbour(*t) && !self.protected(*t))
            .map(|t| self.strength(t))
            .sum();
        if self.strength(v.team) * 100 >= opposition * 85 {
            return;
        }
        for city in self.homes() {
            let Some(d) = &city.detail else {
                continue;
            };
            if d.producing.is_some_and(legatum_core::data::is_great_work_item)
                || !orders.iter().any(|o| matches!(o, Order::SetProduction { city: id, item } if *id == city.id && legatum_core::data::is_great_work_item(*item)))
            { continue; }
            let alternative = d.producing.or_else(|| d.options.iter()
                .filter(|o| matches!(o.item, ProductionItem::Unit { unit } if unit != UnitKind::Scout && legatum_core::data::unit_stats(unit).military) && o.turns > 0)
                .max_by_key(|o| {
                    let ProductionItem::Unit { unit } = o.item else { unreachable!() };
                    let stats = legatum_core::data::unit_stats_for(v.faction_of(v.me), unit);
                    (stats.strength.max(stats.ranged) * 100 / (o.turns + 5), Reverse(o.item))
                }).map(|o| o.item));
            if let Some(item) = alternative {
                orders.retain(|o| !matches!(o, Order::SetProduction { city: id, .. } | Order::BuyItem { city: id, .. } if *id == city.id));
                if d.producing != Some(item) {
                    orders.push(Order::SetProduction {
                        city: city.id,
                        item,
                    });
                }
            }
        }
    }

    /// Peace removes the early conquest income that kept v2's opening sustainable.
    /// Break a visible upkeep spiral before producing replacements that immediately disband.
    /// Only interrupt unit builds; an attacked home keeps its emergency defender.
    fn fund_army(&self, orders: &mut Vec<Order>) {
        let v = self.view;
        let income = v.economy.gold_per_turn.total;
        if income > 0 || v.economy.gold > (-income).max(1) * 15 || self.home_danger() {
            return;
        }
        if v.research.current.is_none()
            && v.research
                .available
                .iter()
                .any(|t| t.tech == legatum_core::TechId::Currency)
        {
            orders.retain(|o| !matches!(o, Order::SetResearch { .. }));
            orders.push(Order::SetResearch {
                tech: legatum_core::TechId::Currency,
            });
        }
        let market = ProductionItem::Building {
            building: legatum_core::BuildingKind::Market,
        };
        for c in self.homes() {
            let Some(d) = &c.detail else {
                continue;
            };
            let scheduled = orders
                .iter()
                .find_map(|o| match o {
                    Order::SetProduction { city, item } if *city == c.id => Some(*item),
                    _ => None,
                })
                .or(d.producing);
            if d.resistance > 0
                || !matches!(scheduled, Some(ProductionItem::Unit { .. }))
                || !matches!(d.producing, None | Some(ProductionItem::Unit { .. }))
                || !d.options.iter().any(|o| o.item == market && o.turns > 0)
            {
                continue;
            }
            orders.retain(|o| !matches!(o, Order::SetProduction { city, .. } | Order::BuyItem { city, .. } if *city == c.id));
            orders.push(Order::SetProduction {
                city: c.id,
                item: market,
            });
        }
    }

    /// Peace lets rivals retain their garrisons and buy reinforcements after a declaration.
    /// Sustain a small campaign force (two infantry, three shooters) rather than repeatedly
    /// replacing just v2's opening trio. One city mobilizes; funded purchases, infrastructure
    /// already underway, Great Works and the rest of the economy retain their existing plan.
    fn mobilize(&self, orders: &mut Vec<Order>) {
        let v = self.view;
        let exposed = self.progress(v.team) >= 50
            && v.players.iter().any(|p| {
                p.alive && p.team != v.team && self.neighbour(p.team) && !self.protected(p.team)
            });
        if self.homes().count() < 2
            || v.economy.gold_per_turn.total < -2
            || (!exposed && self.active_wars().is_empty() && self.expansion_neighbour().is_none())
        {
            return;
        }
        let scheduled = |c: &CityView| {
            orders
                .iter()
                .find_map(|o| match o {
                    Order::SetProduction { city, item } if *city == c.id => Some(*item),
                    _ => None,
                })
                .or(c.detail.as_ref().and_then(|d| d.producing))
        };
        let mut melee = 0;
        let mut ranged = 0;
        for kind in v
            .units
            .iter()
            .filter(|u| u.owner == v.me)
            .map(|u| u.kind)
            .chain(self.homes().filter_map(|c| match scheduled(c) {
                Some(ProductionItem::Unit { unit }) => Some(unit),
                _ => None,
            }))
        {
            let stats = legatum_core::data::unit_stats(kind);
            if stats.military && kind != UnitKind::Scout {
                if stats.ranged > 0 {
                    ranged += 1;
                } else {
                    melee += 1;
                }
            }
        }
        let (melee_goal, ranged_goal) = if exposed { (3, 4) } else { (2, 3) };
        if melee >= melee_goal && ranged >= ranged_goal {
            return;
        }
        let want_ranged = melee > 0 && (ranged < ranged_goal - 1 || melee >= melee_goal);
        let choice = self
            .homes()
            .filter(|c| {
                c.detail.as_ref().is_some_and(|d| d.resistance == 0)
                    && matches!(
                        scheduled(c),
                        Some(ProductionItem::Unit {
                            unit: UnitKind::Settler
                        })
                    )
                    && matches!(
                        c.detail.as_ref().and_then(|d| d.producing),
                        None | Some(ProductionItem::Unit {
                            unit: UnitKind::Settler
                        })
                    )
                    && !scheduled(c).is_some_and(legatum_core::data::is_great_work_item)
                    && !orders
                        .iter()
                        .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
            })
            .flat_map(|c| {
                c.detail
                    .as_ref()
                    .expect("own detail")
                    .options
                    .iter()
                    .filter_map(move |o| {
                        let ProductionItem::Unit { unit } = o.item else {
                            return None;
                        };
                        let stats = legatum_core::data::unit_stats_for(v.faction_of(v.me), unit);
                        (stats.military
                            && unit != UnitKind::Scout
                            && (stats.ranged > 0) == want_ranged
                            && o.turns > 0)
                            .then_some((
                                Reverse(stats.strength.max(stats.ranged) * 100 / (o.turns + 5)),
                                c.id,
                                o.item,
                            ))
                    })
            })
            .min();
        if let Some((_, city, item)) = choice {
            orders.retain(|o| !matches!(o, Order::SetProduction { city: id, .. } if *id == city));
            orders.push(Order::SetProduction { city, item });
        }
    }

    fn alive(&self, team: TeamId) -> bool {
        self.view.players.iter().any(|p| p.team == team && p.alive)
    }

    fn strength(&self, team: TeamId) -> i32 {
        // These army values are public, including armies outside our vision. They do
        // not reveal locations, health, composition or orders of unseen units.
        crate::diplomacy::strength(self.view, team)
    }

    fn progress(&self, team: TeamId) -> u32 {
        self.view
            .standings
            .iter()
            .find(|s| s.team == team)
            .map_or(0, |s| 100 * s.vp / s.target.max(1))
    }

    fn allied(&self, a: TeamId, b: TeamId) -> bool {
        self.view
            .diplomacy
            .relations
            .iter()
            .any(|r| r.between(a, b) && r.state == RelationState::Alliance)
    }

    /// A weak garrison does not make its owner's allies disappear. Budget for existing
    /// enemies and unprotected local allies that would fight beside the target.
    /// When we lead the public race, include the other unprotected local flanks too.
    fn war_affordable(&self, target: TeamId) -> bool {
        let v = self.view;
        let leading = legatum_core::diplomacy::coalition::target_view(v) == Some(v.team);
        let enemies: BTreeSet<_> = v
            .players
            .iter()
            .filter(|p| {
                p.alive
                    && p.team != v.team
                    && (p.team == target
                        || (v.teams_at_war(v.team, p.team) && self.has_front(p.team))
                        || (!self.protected(p.team)
                            && self.neighbour(p.team)
                            && (leading || self.allied(target, p.team))))
            })
            .map(|p| p.team)
            .collect();
        // An isolated opponent is assessed by the local siege forecast below. Its
        // distant army alone is not a reason to pass up a reachable, weakly held city.
        if enemies.len() == 1 {
            return !self.home_danger();
        }
        let hostile: i32 = enemies.iter().map(|t| self.strength(*t)).sum();
        // Credit only allies already fighting this enemy, at half strength: they have
        // their own fronts. A pending treaty supplies no troops.
        let support: i32 = v
            .players
            .iter()
            .filter(|p| p.alive && p.team != v.team && !enemies.contains(&p.team))
            .map(|p| p.team)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|t| self.allied(v.team, *t) && v.teams_at_war(*t, target))
            .map(|t| self.strength(t) / 2)
            .sum();
        !self.home_danger() && (self.strength(v.team) + support) * 100 >= hostile * 85
    }

    fn alliance_useful(&self, other: TeamId, target: Option<TeamId>) -> bool {
        let v = self.view;
        target != Some(other)
            && self.progress(other) < 70
            && !v
                .diplomacy
                .betrayals
                .iter()
                .any(|b| b.by == other && b.against == v.team)
            && (self.neighbour(other) || self.wars().iter().any(|t| v.teams_at_war(other, *t)))
    }

    fn legal(&self, order: &Order) -> bool {
        self.view
            .diplomacy
            .actions
            .iter()
            .any(|a| a.order == *order && a.blocked_reason.is_none())
    }

    fn homes(&self) -> impl Iterator<Item = &CityView> {
        self.view.cities.iter().filter(|c| c.owner == self.view.me)
    }

    fn neighbour(&self, other: TeamId) -> bool {
        self.view.cities.iter().any(|c| {
            self.view.team_of(c.owner) == other
                && self.homes().any(|home| home.pos.distance(c.pos) <= 10)
        }) || self.view.units.iter().any(|u| {
            self.view.team_of(u.owner) == other
                && self.homes().any(|home| home.pos.distance(u.pos) <= 6)
        })
    }

    fn protected(&self, other: TeamId) -> bool {
        self.view.diplomacy.relations.iter().any(|r| {
            r.between(self.view.team, other)
                && (r.guarantee_until > self.view.turn || r.state == RelationState::Alliance)
        })
    }

    fn wars(&self) -> Vec<TeamId> {
        self.view
            .players
            .iter()
            .filter(|p| p.alive && self.view.teams_at_war(self.view.team, p.team))
            .map(|p| p.team)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn reachable_city(&self, c: &CityView) -> bool {
        let v = self.view;
        let map = Map::new(v.width, v.height, Terrain::Grassland);
        self.homes().any(|home| {
            home.pos.distance(c.pos) <= 10
                && legatum_core::path::find_path(&map, home.pos, c.pos, |h| {
                    let t = v.tile(h)?;
                    (t.explored
                        && (v.passage_allowed(v.me, legatum_core::UnitKind::Soldier, t.owner)
                            || t.owner.is_some_and(|o| v.team_of(o) == v.team_of(c.owner))))
                    .then(|| t.terrain?.move_cost())
                    .flatten()
                })
                .is_some()
        })
    }

    fn has_front(&self, team: TeamId) -> bool {
        let v = self.view;
        v.units.iter().any(|u| {
            v.team_of(u.owner) == team
                && power(u) > 0
                && self.homes().any(|c| c.pos.distance(u.pos) <= FLANK_RADIUS)
        }) || v
            .cities
            .iter()
            .any(|c| v.team_of(c.owner) == team && self.reachable_city(c))
    }

    fn active_wars(&self) -> Vec<TeamId> {
        self.wars()
            .into_iter()
            .filter(|t| self.has_front(*t))
            .collect()
    }

    fn home_danger(&self) -> bool {
        self.homes().any(|c| {
            let attackers: i32 = self
                .view
                .units
                .iter()
                .filter(|u| self.view.at_war(self.view.me, u.owner) && u.pos.distance(c.pos) <= 4)
                .map(power)
                .sum();
            let defenders: i32 = self
                .view
                .units
                .iter()
                .filter(|u| u.owner == self.view.me && u.pos.distance(c.pos) <= 3)
                .map(power)
                .sum();
            // A freshly captured city has little HP even in a winning campaign. Its
            // low HP alone is not evidence that the war is going badly.
            let settled = c.detail.as_ref().is_some_and(|d| d.resistance == 0);
            settled
                && attackers > defenders + c.strength
                && (c.hp * 3 < c.max_hp || attackers > defenders * 2 + c.strength)
        })
    }

    fn wants_peace(&self, other: TeamId) -> bool {
        let v = self.view;
        let age = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, other))
            .map_or(0, |r| v.turn.saturating_sub(r.effective_turn));
        let enemies: i32 = self.wars().iter().map(|t| self.strength(*t)).sum();
        let viable = self.campaign(other).is_some();
        self.home_danger()
            || (age >= 12 && !self.has_front(other))
            || (age >= 8 && !viable && self.strength(v.team) * 100 < enemies * 60)
            || (age >= 40 && !viable && self.strength(v.team) < self.strength(other))
    }

    /// Reserve enough strength for every unprotected, visible flank together. A
    /// pending offer is not protection. Unknown forces cannot be made safe by inference.
    fn safe_flanks(&self, target: TeamId, committed: i32) -> bool {
        let v = self.view;
        let flank_power: i32 = v
            .units
            .iter()
            .filter(|u| {
                let team = v.team_of(u.owner);
                team != v.team
                    && team != target
                    && !self.protected(team)
                    && self.homes().any(|c| c.pos.distance(u.pos) <= FLANK_RADIUS)
            })
            .map(power)
            .sum();
        let own: i32 = v.units.iter().filter(|u| u.owner == v.me).map(power).sum();
        own >= committed + flank_power * 5 / 4
            && !self.home_danger()
            && self.homes().all(|c| {
                let threat: i32 = v
                    .units
                    .iter()
                    .filter(|u| {
                        let team = v.team_of(u.owner);
                        team != v.team
                            && team != target
                            && !self.protected(team)
                            && u.pos.distance(c.pos) <= FLANK_RADIUS
                    })
                    .map(power)
                    .sum();
                let local: i32 = v
                    .units
                    .iter()
                    .filter(|u| u.owner == v.me && u.pos.distance(c.pos) <= RESERVE_RADIUS)
                    .map(power)
                    .sum();
                threat <= local + c.strength / 2
            })
    }

    /// Keep a local reserve against unprotected neutral armies on another flank.
    /// Tactical combat still uses the actual relations; neutral units are never targets.
    fn guard_flanks(&self, orders: &mut [Order]) {
        let v = self.view;
        let mut reserved = BTreeSet::new();
        for c in self.homes() {
            let threat: i32 = v
                .units
                .iter()
                .filter(|u| {
                    let team = v.team_of(u.owner);
                    team != v.team
                        && !v.teams_at_war(v.team, team)
                        && !self.protected(team)
                        && u.pos.distance(c.pos) <= FLANK_RADIUS
                })
                .map(power)
                .sum();
            let mut needed = threat * 5 / 4 - c.strength / 2;
            let mut defenders: Vec<_> = v
                .units
                .iter()
                .filter(|u| {
                    u.owner == v.me && power(u) > 0 && u.pos.distance(c.pos) <= RESERVE_RADIUS
                })
                .collect();
            defenders.sort_by_key(|u| (u.pos.distance(c.pos), u.id));
            for u in defenders {
                if needed <= 0 {
                    break;
                }
                if !reserved.insert(u.id) {
                    continue;
                }
                needed -= power(u);
                for order in orders.iter_mut() {
                    if matches!(order, Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                        if *unit == u.id && to.distance(c.pos) > RESERVE_RADIUS)
                    {
                        *order = Order::SkipUnit { unit: u.id };
                    }
                }
            }
        }
    }

    /// At Peace the v2 fallback sends infantry and archers exploring separately.
    /// Assemble them outside the intended neighbor's borders instead; scouts still
    /// explore. Remembered cities guide travel, but only a visible defense can justify War.
    fn stage_army(&self, orders: &mut [Order]) {
        let v = self.view;
        if !self.active_wars().is_empty() {
            return;
        }
        let Some(team) = orders
            .iter()
            .find_map(|order| match order {
                Order::DeclareWar { against, .. } => Some(*against),
                Order::BreakTreaty { with, .. } => Some(*with),
                _ => None,
            })
            .or_else(|| self.expansion_neighbour())
        else {
            return;
        };
        let Some(city) = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == team)
            .min_by_key(|c| (self.homes().map(|h| h.pos.distance(c.pos)).min(), c.id))
        else {
            return;
        };
        let ctx = Ctx::for_proxy(v);
        let map = Map::new(v.width, v.height, Terrain::Grassland);
        let mut reserved = BTreeSet::new();
        for u in &ctx.own_units {
            if power(u) == 0 || u.hp < 75 {
                continue;
            }
            let Some(order) = orders.iter_mut().find(|o| matches!(o,
                Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. } | Order::SkipUnit { unit } if *unit == u.id)) else { continue; };
            let field = ctx.distance_field(u.pos, true);
            let mut spots: Vec<_> = city
                .pos
                .within(4)
                .into_iter()
                .filter(|h| {
                    ctx.move_cost(*h).is_some()
                        && !reserved.contains(h)
                        && !ctx.neutral_at[v.index(*h).expect("known tile")]
                        && !v
                            .units
                            .iter()
                            .any(|e| e.id != u.id && e.military && e.pos == *h)
                })
                .filter_map(|h| {
                    let d = field[v.index(h)?];
                    (d != UNREACHABLE).then_some((h.distance(city.pos), d, h))
                })
                .collect();
            spots.sort();
            for (_, _, to) in spots {
                if to == u.pos {
                    reserved.insert(to);
                    *order = Order::SkipUnit { unit: u.id };
                    break;
                }
                let Some(path) = legatum_core::path::find_path(&map, u.pos, to, |h| {
                    let i = v.index(h)?;
                    if reserved.contains(&h)
                        || ctx.neutral_at[i]
                        || v.units
                            .iter()
                            .any(|e| e.id != u.id && e.military && e.pos == h)
                    {
                        return None;
                    }
                    ctx.move_cost(h)
                }) else {
                    continue;
                };
                let mut moves = u.max_moves;
                let steps: Vec<_> = path
                    .into_iter()
                    .take_while(|h| {
                        let cost = ctx.move_cost(*h).expect("known route");
                        let legal =
                            legatum_core::rules::movement::can_step(moves, u.max_moves, cost);
                        moves = (moves - cost).max(0);
                        legal
                    })
                    .collect();
                if let Some(to) = steps.last().copied() {
                    reserved.insert(to);
                    *order = Order::MoveUnitPath {
                        unit: u.id,
                        to,
                        path: steps,
                        stop_on_block: false,
                    };
                    break;
                }
            }
        }
    }

    /// A hypothetical permission change, never a tactical order against a peaceful
    /// player. All terrain, troops and defenses stay exactly as the fogged view knows them.
    fn campaign(&self, other: TeamId) -> Option<Campaign> {
        let v = self.view;
        if !self.neighbour(other) {
            return None;
        }
        let mut projected = v.clone();
        projected
            .diplomacy
            .relations
            .iter_mut()
            .find(|r| r.between(v.team, other))?
            .state = RelationState::War;
        let ctx = Ctx::for_proxy(&projected);
        v.cities
            .iter()
            // Memory is useful for exploration, never evidence of a current weak defense.
            .filter(|c| c.visible && v.team_of(c.owner) == other)
            .filter_map(|c| {
                let field = ctx.distance_field(c.pos, false);
                let troops: Vec<_> = v
                    .units
                    .iter()
                    .filter(|u| u.owner == v.me && power(u) > 0 && u.hp >= 60)
                    .filter(|u| {
                        v.index(u.pos)
                            .is_some_and(|i| field[i] != UNREACHABLE && field[i] <= u.max_moves * 8)
                    })
                    .collect();
                let melee = troops.iter().filter(|u| u.ranged == 0).count();
                let ranged = troops.iter().filter(|u| u.ranged > 0).count();
                if melee == 0 || (ranged < 2 && melee < 3 && c.hp > 30) {
                    return None;
                }
                let army: i32 = troops.iter().map(|u| power(u)).sum();
                let guards: i32 = v
                    .units
                    .iter()
                    .filter(|u| v.team_of(u.owner) == other && u.pos.distance(c.pos) <= 5)
                    .map(power)
                    .sum();
                let required = guards * 5 / 4 + c.strength;
                if army < required || !self.safe_flanks(other, required) {
                    return None;
                }
                // Forecast a coordinated volley from distinct known firing tiles, then
                // the infantry follow-up. Fire hits the original garrison; excess damage
                // never spills through it into city HP. This changes the campaign forecast,
                // not this turn's tactical permissions or orders.
                let mut volley = 0;
                let mut firing_tiles = BTreeSet::new();
                for u in troops.iter().filter(|u| u.ranged > 0) {
                    let mut forecast_view = projected.clone();
                    let Some(firing) = c.pos.ring(u.range).into_iter().find(|h| {
                        !firing_tiles.contains(h)
                            && ctx.reachable(u.pos, *h)
                            && !v.units.iter().any(|e| e.pos == *h && e.owner != v.me)
                    }) else {
                        continue;
                    };
                    firing_tiles.insert(firing);
                    forecast_view.units.iter_mut().find(|a| a.id == u.id)?.pos = firing;
                    let f = forecast_attack_mode(&forecast_view, u.id, c.pos, AttackMode::Ranged)?;
                    volley += legatum_core::combat::damage(
                        f.attacker_strength.total,
                        f.defender_strength.total,
                        80,
                    );
                }
                let mut after_volley = projected.clone();
                if let Some(garrison) = after_volley
                    .units
                    .iter_mut()
                    .find(|u| u.military && u.pos == c.pos && v.team_of(u.owner) == other)
                {
                    garrison.hp -= volley;
                    after_volley.units.retain(|u| u.hp > 0);
                } else {
                    after_volley.cities.iter_mut().find(|a| a.id == c.id)?.hp =
                        (c.hp - volley).max(0);
                }
                let safe_melee = troops.iter().filter(|u| u.ranged == 0).any(|u| {
                    forecast_attack_mode(&after_volley, u.id, c.pos, AttackMode::Melee).is_some_and(
                        |f| f.loss_chance == 0 && f.expected_damage_to_attacker * 2 < u.hp,
                    )
                });
                if !safe_melee || (volley < 20 && c.hp > 30) {
                    return None;
                }
                Some(Campaign {
                    team: other,
                    resistance: self.strength(other).max(guards) + c.strength,
                    distance: self.homes().map(|h| h.pos.distance(c.pos)).min()?,
                    city: c.id,
                })
            })
            .min_by_key(|c| (c.resistance, c.distance, c.city))
    }

    /// Keep an expansion route open while assembling the opening army. Signing with
    /// every neighbour before a siege is ready otherwise traps the proxy in permanent peace.
    fn expansion_neighbour(&self) -> Option<TeamId> {
        let v = self.view;
        v.cities
            .iter()
            .filter(|c| {
                let team = v.team_of(c.owner);
                team != v.team && self.alive(team) && self.neighbour(team) && !self.protected(team)
            })
            .filter(|c| self.reachable_city(c))
            .min_by_key(|c| {
                (
                    self.strength(v.team_of(c.owner)),
                    self.homes().map(|h| h.pos.distance(c.pos)).min(),
                    c.id,
                )
            })
            .map(|c| v.team_of(c.owner))
    }

    fn orders(&self) -> Vec<Order> {
        let v = self.view;
        let teams: BTreeSet<_> = v
            .players
            .iter()
            .filter(|p| p.alive && p.team != v.team)
            .map(|p| p.team)
            .collect();
        let target = if self.active_wars().is_empty() {
            teams
                .iter()
                .filter_map(|t| {
                    self.legal(&Order::DeclareWar {
                        against: *t,
                        breach: false,
                    })
                    .then(|| self.war_affordable(*t).then(|| self.campaign(*t)).flatten())
                    .flatten()
                })
                .min_by_key(|c| (c.resistance, c.distance, c.team, c.city))
                .map(|c| c.team)
        } else {
            None
        };
        let leave = |team: TeamId| Order::BreakTreaty {
            with: team,
            agreement: Agreement::Security,
        };
        let free = |order: &Order| {
            v.diplomacy.actions.iter().any(|a| {
                a.order == *order && a.blocked_reason.is_none() && a.cost == 0 && !a.betrayal
            })
        };
        let leave_target = if target.is_none() && self.active_wars().is_empty() {
            let closed = self.expansion_neighbour().is_none();
            teams
                .iter()
                .filter_map(|team| {
                    (((closed && !self.allied(v.team, *team)) || self.progress(*team) >= 80)
                        && self.war_affordable(*team)
                        && free(&leave(*team)))
                    .then(|| self.campaign(*team))
                    .flatten()
                })
                .min_by_key(|c| (c.resistance, c.distance, c.team, c.city))
                .map(|c| c.team)
        } else {
            None
        };
        let reserve = target.or(leave_target).or_else(|| {
            self.active_wars()
                .is_empty()
                .then(|| self.expansion_neighbour())
                .flatten()
        });
        let mut out = Vec::new();
        let mut handled = BTreeSet::new();
        for p in &v.diplomacy.proposals {
            if p.to != v.team || !self.alive(p.from) || handled.contains(&p.from) {
                continue;
            }
            let accept = match p.kind {
                TreatyKind::Peace => self.wants_peace(p.from),
                TreatyKind::Alliance => self.alliance_useful(p.from, reserve),
            };
            let yes = Order::RespondToProposal {
                proposal: p.id,
                accept: true,
                explanation: None,
            };
            let order = Order::RespondToProposal {
                proposal: p.id,
                accept: accept && self.legal(&yes),
                explanation: None,
            };
            if self.legal(&order) {
                out.push(order);
                handled.insert(p.from);
            }
        }
        // An ally's war is a call to arms: join when the target is about to win or a campaign
        // against it is affordable and can reach it; decline otherwise (the Alliance stands).
        for call in &v.diplomacy.calls {
            if handled.contains(&call.ally) || !self.alive(call.target) {
                continue;
            }
            let yes = Order::RespondToProposal {
                proposal: call.id,
                accept: true,
                explanation: None,
            };
            let join = self.legal(&yes)
                && (self.progress(call.target) >= 80
                    || (self.war_affordable(call.target) && self.campaign(call.target).is_some()));
            let order = Order::RespondToProposal {
                proposal: call.id,
                accept: join,
                explanation: None,
            };
            if self.legal(&order) {
                out.push(order);
                handled.insert(call.ally);
            }
        }
        for team in teams {
            if handled.contains(&team) {
                continue;
            }
            let alliance = Order::ProposeTreaty {
                to: team,
                kind: TreatyKind::Alliance,
            };
            let order = if target == Some(team) {
                Order::DeclareWar {
                    against: team,
                    breach: false,
                }
            } else if v.teams_at_war(v.team, team) && self.wants_peace(team) {
                Order::ProposeTreaty {
                    to: team,
                    kind: TreatyKind::Peace,
                }
            } else if leave_target == Some(team) {
                leave(team)
            } else if self.alliance_useful(team, reserve)
                && self.wars().iter().any(|enemy| v.teams_at_war(team, *enemy))
                && self.legal(&alliance)
            {
                alliance
            } else {
                continue;
            };
            if let Order::ProposeTreaty { to, kind } = &order
                && v.diplomacy.peace_cooldowns.iter().any(|c| {
                    c.from == v.team && c.to == *to && c.kind == *kind && v.turn < c.retry_turn
                })
            {
                continue;
            }
            if self.legal(&order) {
                out.push(order);
            }
        }
        out
    }
}

fn power(u: &UnitView) -> i32 {
    if !u.military || u.kind == UnitKind::Scout {
        0
    } else {
        u.strength.max(u.ranged) * u.hp / 100
    }
}

/// Peace closes borders without teleporting units. Find a legal exit on *known* terrain;
/// replace that unit's old tactical order instead of issuing two competing orders.
fn egress(v: &PlayerView, orders: &mut Vec<Order>) {
    let trapped: Vec<_> = v
        .units
        .iter()
        .filter(|u| {
            u.owner == v.me && !v.passage_allowed(v.me, u.kind, v.tile(u.pos).and_then(|t| t.owner))
        })
        .collect();
    if trapped.is_empty() {
        return;
    }
    let mut map = Map::new(v.width, v.height, Terrain::Mountains);
    for (i, t) in v.tiles.iter().enumerate() {
        if t.explored
            && let Some(terrain) = t.terrain
        {
            map.tiles[i].terrain = terrain;
        }
    }
    let mut reserved = BTreeSet::new();
    for u in trapped {
        let open = |h| {
            v.tile(h)
                .is_some_and(|t| v.passage_allowed(v.me, u.kind, t.owner))
        };
        let exits = legatum_core::path::exit_distances(&map, open);
        let destination = u
            .pos
            .neighbors()
            .into_iter()
            .filter(|h| map.move_cost(*h).is_some())
            .filter(|h| legatum_core::path::border_step(&map, u.pos, *h, open, Some(&exits)))
            .filter(|h| !reserved.contains(&(u.military, *h)))
            .filter(|h| {
                !v.units.iter().any(|e| {
                    e.pos == *h
                        && e.id != u.id
                        && (!v.same_team(v.me, e.owner) || e.military == u.military)
                })
            })
            .min_by_key(|h| {
                (
                    exits[map.index(*h).expect("known tile")],
                    Reverse(open(*h)),
                    *h,
                )
            });
        orders.retain(|o| {
            !matches!(o,
            Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. } |
            Order::RangedAttack { unit, .. } | Order::SkipUnit { unit } |
            Order::UpgradeUnit { unit } | Order::FoundCity { unit } if *unit == u.id)
        });
        if let Some(to) = destination {
            reserved.insert((u.military, to));
            orders.push(Order::MoveUnit { unit: u.id, to });
        } else {
            orders.push(Order::SkipUnit { unit: u.id });
        }
    }
}

//! Composable opt-in decisions. Every input is an owner's PlayerView or memory of one.
use super::{EscortPolicy, Gene, Numbers, unit};
use crate::analysis::{defense_power, offense_power, threat_power};
use crate::scripted::Planner;
use legatum_core::diplomacy::RelationState;
use legatum_core::view::{CityView, ProductionOption, UnitView};
use legatum_core::{
    CityId, Hex, Order, PlayerView, ProductionItem, TeamId, UnitId, UnitKind, data,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! genes {
    ($($name:ident: $default:expr, $min:expr, $max:expr;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct PrimitiveParams { $(pub $name: i32,)* }
        impl Default for PrimitiveParams {
            fn default() -> Self { Self { $($name: $default,)* } }
        }
        impl PrimitiveParams {
            pub fn genes() -> Vec<Gene> { vec![$(Gene {
                name: concat!("primitives.", stringify!($name)).into(), min: $min, max: $max,
            },)*] }
            pub fn genome(self) -> Vec<i32> { vec![$(self.$name,)*] }
            pub(super) fn decode(values: &mut impl Iterator<Item = i32>) -> Self {
                Self { $($name: values.next().expect("validated primitive genome"),)* }
            }
        }
    }
}

// Appended after the original 25 genes. Zero weight disables that primitive;
// 100 is nominal, larger weights increase its urgency without changing any rule.
genes! {
    escort_civilians_weight: 100, 0, 300;
    escort_civilians_threat: 20, 0, 200;
    escort_civilians_wait: 8, 0, 60;
    snipe_settlers_weight: 100, 0, 300;
    snipe_settlers_radius: 5, 1, 15;
    snipe_settlers_war_score: 60, 0, 200;
    scout_upkeep_weight: 100, 0, 300;
    scout_upkeep_minimum: 1, 0, 5;
    scout_upkeep_until: 200, 0, 600;
    scout_upkeep_intel_age: 40, 2, 300;
    save_and_burst_weight: 100, 0, 300;
    save_and_burst_reserve_percent: 100, 0, 300;
    weakest_target_weight: 100, 0, 300;
    weakest_target_distance: 8, 1, 25;
    coalition_march_weight: 100, 0, 300;
    coalition_march_leader_percent: 60, 1, 100;
    coalition_march_force_percent: 50, 1, 100;
    siege_persistence_weight: 100, 0, 300;
    siege_persistence_hp_percent: 60, 1, 100;
    barbarian_settler_guard_weight: 100, 0, 300;
    barbarian_settler_guard_threat: 1, 0, 200;
    market_first_economy_weight: 100, 0, 300;
    market_first_economy_horizon: 120, 2, 300;
}

#[derive(Clone, Debug, Default)]
pub struct PrimitiveMemory {
    /// Last direct sighting of a rival's city or unit. Public standings do not refresh intel.
    last_seen: BTreeMap<TeamId, u32>,
    /// Start of the current continuous stay in an own city; leaving resets the wait.
    waiting_since: BTreeMap<UnitId, u32>,
}
impl PrimitiveMemory {
    pub fn observe(&mut self, v: &PlayerView) {
        for c in v.cities.iter().filter(|c| !v.same_team(v.me, c.owner)) {
            let seen = self.last_seen.entry(v.team_of(c.owner)).or_insert(0);
            if c.visible {
                *seen = v.turn;
            }
        }
        for u in v
            .units
            .iter()
            .filter(|u| !v.same_team(v.me, u.owner) && !u.owner.is_barbarian())
        {
            self.last_seen.insert(v.team_of(u.owner), v.turn);
        }
        // A public leader with no sighting has unknown, hence conservatively old, intel.
        if let Some(team) = leader(v) {
            self.last_seen.entry(team).or_insert(0);
        }
        self.waiting_since.retain(|id, _| {
            v.units.iter().any(|s| {
                s.id == *id
                    && s.owner == v.me
                    && s.kind == UnitKind::Settler
                    && v.cities.iter().any(|c| c.owner == v.me && c.pos == s.pos)
            })
        });
        for s in v.units.iter().filter(|s| {
            s.owner == v.me
                && s.kind == UnitKind::Settler
                && v.cities.iter().any(|c| c.owner == v.me && c.pos == s.pos)
        }) {
            self.waiting_since.entry(s.id).or_insert(v.turn);
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PrimitivePlan<'a> {
    pub params: &'a PrimitiveParams,
    pub memory: &'a PrimitiveMemory,
}

/// A dispatch intent. The observer counts it only if the minimum force actually advances.
#[derive(Clone, Debug)]
pub struct CoalitionMarch {
    pub city: CityId,
    pub target: Hex,
    pub minimum: usize,
    pub units: Vec<(UnitId, Hex)>,
}

fn leader(v: &PlayerView) -> Option<TeamId> {
    let mut standings: Vec<_> = v
        .standings
        .iter()
        .filter(|s| s.target > 0 && v.players.iter().any(|p| p.team == s.team && p.alive))
        .map(|s| (s.vp * 100 / s.target, s.team))
        .collect();
    standings.sort_by(|a, b| b.cmp(a));
    let first = standings.first()?;
    (first.1 != v.team && standings.get(1).is_none_or(|second| first.0 > second.0))
        .then_some(first.1)
}

impl<'a> Planner<'a> {
    fn primitive_numbers(&self) -> Numbers {
        self.opening
            .map_or_else(Numbers::default, |o| o.policy.numbers)
    }

    /// Defender/city power divided by OUR LOCAL force, in hundredths. Unseen cities
    /// never become undefended cities; remembered cities remain useful march destinations.
    pub(crate) fn primitive_weakest_city(
        &self,
        team: Option<TeamId>,
    ) -> Option<(&'a CityView, i32)> {
        let g = self.primitives?.params;
        if g.weakest_target_weight == 0 {
            return None;
        }
        let v = self.view();
        v.cities
            .iter()
            .filter(|c| {
                c.visible
                    && self.assessed_campaign_allowed(c)
                    && !v.same_team(v.me, c.owner)
                    && !c.owner.is_barbarian()
                    && team.is_none_or(|t| v.team_of(c.owner) == t)
                    && v.players.iter().any(|p| p.id == c.owner && p.alive)
            })
            .filter_map(|c| {
                let ours: i32 = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| {
                        u.military
                            && u.kind != UnitKind::Scout
                            && u.pos.distance(c.pos) <= g.weakest_target_distance
                            && self.opening_reachable(u.pos, c.pos)
                    })
                    .map(|u| offense_power(v, u))
                    .sum();
                if ours <= 0 {
                    return None;
                }
                let defenders: i32 = v
                    .units
                    .iter()
                    .filter(|u| {
                        u.military
                            && v.same_team(c.owner, u.owner)
                            && u.pos.distance(c.pos) <= g.weakest_target_distance
                    })
                    .map(|u| defense_power(v, u))
                    .sum();
                let ratio = (defenders + c.strength).max(1) * 100 / ours;
                Some((c, ratio))
            })
            .min_by_key(|(c, ratio)| (*ratio, c.id))
    }

    fn snipes(
        &self,
        team: Option<TeamId>,
        hostile_only: bool,
    ) -> Vec<(&'a UnitView, &'a UnitView)> {
        let Some(p) = self.primitives else {
            return Vec::new();
        };
        let g = p.params;
        if g.snipe_settlers_weight == 0 {
            return Vec::new();
        }
        let v = self.view();
        let mut pairs = Vec::new();
        for civilian in v.units.iter().filter(|s| {
            !s.military
                && !v.same_team(v.me, s.owner)
                && team.is_none_or(|t| v.team_of(s.owner) == t)
                && (!hostile_only || s.owner.is_barbarian() || v.at_war(v.me, s.owner))
        }) {
            if v.cities.iter().any(|c| c.pos == civilian.pos)
                || v.units.iter().any(|u| {
                    u.military
                        && v.same_team(u.owner, civilian.owner)
                        && u.pos.distance(civilian.pos) <= 1
                })
            {
                continue;
            }
            for u in self.ctx.own_units.iter().copied().filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && !self.has_acted(u.id)
                    && u.pos.distance(civilian.pos) <= g.snipe_settlers_radius
                    && self.ctx.reachable(u.pos, civilian.pos)
            }) {
                pairs.push((u, civilian));
            }
        }
        pairs.sort_by_key(|(u, s)| (u.pos.distance(s.pos), s.id, u.id));
        pairs
    }

    pub(crate) fn primitive_snipe_target(&self, team: TeamId) -> Option<Hex> {
        self.snipes(Some(team), false).first().map(|(_, s)| s.pos)
    }

    pub(crate) fn primitive_coalition_target(&self) -> Option<&'a CityView> {
        let g = self.primitives?.params;
        if g.coalition_march_weight == 0 {
            return None;
        }
        let v = self.view();
        let team = leader(v)?;
        let standing = v.standings.iter().find(|s| s.team == team)?;
        if standing.vp * 100 / standing.target <= g.coalition_march_leader_percent as u32 {
            return None;
        }
        let allied = v.diplomacy.relations.iter().any(|r| {
            r.state == RelationState::Alliance
                && (r.a == v.team || r.b == v.team)
                && r.a != team
                && r.b != team
        });
        if !allied {
            return None;
        }
        v.cities
            .iter()
            .filter(|c| v.team_of(c.owner) == team)
            .filter_map(|c| {
                let distance = self
                    .ctx
                    .own_cities
                    .iter()
                    .filter(|h| self.ctx.reachable(h.pos, c.pos))
                    .map(|h| h.pos.distance(c.pos))
                    .min()?;
                Some((distance, c.id, c))
            })
            .min_by_key(|(d, id, _)| (*d, *id))
            .map(|(_, _, c)| c)
    }

    /// Primitives nominate a target. Non-default openings apply their force gate afterwards.
    pub(crate) fn plan_primitive_war(&mut self) -> bool {
        let Some(p) = self.primitives else {
            return false;
        };
        let g = p.params;
        if self.rushing()
            && self.strike_city.is_some()
            && self
                .strike_target
                .is_some_and(|team| self.view().teams_at_war(self.view().team, team))
        {
            return true;
        }
        if g.weakest_target_weight == 0
            && g.snipe_settlers_weight == 0
            && g.coalition_march_weight == 0
        {
            return false;
        }
        self.orders
            .retain(|o| !matches!(o, Order::DeclareWar { .. }));
        let v = self.view();
        let coalition = self
            .primitive_coalition_target()
            .map(|c| v.team_of(c.owner));
        let n = self.primitive_numbers();
        let mut candidates = Vec::new();
        for team in v
            .players
            .iter()
            .filter(|p| p.alive && p.team != v.team)
            .map(|p| p.team)
        {
            let local = self.primitive_weakest_city(Some(team));
            let ratio = local.map(|(_, ratio)| ratio);
            let snipe = !self.snipes(Some(team), false).is_empty();
            let local_score = ratio.map_or(0, |r| {
                (100 - r).clamp(-100, 100) * g.weakest_target_weight / 100
            });
            let score = self.knobs.aggression
                + local_score
                + i32::from(snipe) * g.snipe_settlers_war_score * g.snipe_settlers_weight / 100;
            let weak = ratio.is_some_and(|r| r < n.weak_percent * g.weakest_target_weight / 100);
            let coalition = coalition == Some(team);
            if (ratio.is_some() || snipe || coalition) && (weak || score >= 70 || coalition) {
                candidates.push((
                    !coalition,
                    ratio.unwrap_or(i32::MAX),
                    std::cmp::Reverse(score),
                    team,
                ));
            }
        }
        candidates.sort();
        candidates.dedup();
        for (_, _, _, against) in candidates {
            let already_at_war = v.teams_at_war(v.team, against);
            let order = Order::DeclareWar {
                against,
                breach: false,
            };
            if !already_at_war
                && !v
                    .diplomacy
                    .actions
                    .iter()
                    .any(|a| a.order == order && a.blocked_reason.is_none())
            {
                continue;
            }
            if self.opening.is_some() {
                self.strike_target = Some(against);
                self.strike_city = self
                    .primitive_weakest_city(Some(against))
                    .map(|(c, _)| c.id);
                break;
            }
            self.orders.retain(|o| {
                !matches!(o, Order::ProposeTreaty { to, .. } if *to == against)
                    && !matches!(o, Order::RespondToProposal { proposal, accept: true, .. }
                    if v.diplomacy.proposals.iter().any(|p| p.id == *proposal && p.from == against))
            });
            if !already_at_war {
                self.orders.push(order);
            }
            self.strike_target = Some(against);
            break;
        }
        true
    }

    pub(crate) fn plan_primitive_snipes(&mut self) {
        let pairs = self.snipes(None, true);
        let mut captured = Vec::new();
        for (u, s) in pairs {
            if self.has_acted(u.id) || captured.contains(&s.id) {
                continue;
            }
            if self.unit_order(
                u.id,
                Order::MoveUnit {
                    unit: u.id,
                    to: s.pos,
                },
            ) {
                self.mil_dest.insert(s.pos);
                captured.push(s.id);
            }
        }
    }

    pub(crate) fn plan_primitive_escorts(&mut self) -> bool {
        let Some(p) = self.primitives else {
            return false;
        };
        if let Some(o) = self.opening {
            match o.policy.escort {
                EscortPolicy::None => return true,
                EscortPolicy::WaitForEscort => {
                    self.plan_opening_escorts();
                    return true;
                }
                EscortPolicy::EscortIfThreat => {}
            }
        }
        let g = p.params;
        if g.escort_civilians_weight == 0 {
            return false;
        }
        let radius = self.primitive_numbers().escort_radius;
        for s in self
            .ctx
            .own_units
            .clone()
            .into_iter()
            .filter(|u| u.kind == UnitKind::Settler)
        {
            if self.ctx.own_cities.is_empty() || self.retreats.contains(&s.id) {
                continue;
            }
            if self
                .orders
                .iter()
                .any(|o| matches!(o, Order::FoundCity { unit } if *unit == s.id))
            {
                // Movement resolves before founding; reserve the arrived escort even
                // when the last leg is now safe and no city exists here yet.
                if let Some(escort) = self.ctx.own_units.iter().copied().find(|u| {
                    u.military
                        && u.kind != UnitKind::Scout
                        && u.pos == s.pos
                        && !self.has_acted(u.id)
                }) {
                    self.go_to(escort, s.pos);
                }
                continue;
            }
            let destination = self.orders.iter().find_map(|o| match o {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                    if *unit == s.id =>
                {
                    Some(*to)
                }
                _ => None,
            });
            let Some(destination) = destination else {
                continue;
            };
            // Ignore journeys wholly within our borders. Foreign troops at plain Peace
            // are perceived risks too: a surprise declaration can capture this turn.
            let Some(path) = self
                .view()
                .movement_path(self.ctx.me, s.kind, s.pos, destination)
            else {
                continue;
            };
            let leaves_borders = path.iter().any(|h| {
                self.view()
                    .tile(*h)
                    .is_none_or(|t| t.owner != Some(self.ctx.me))
            });
            if !leaves_borders {
                continue;
            }
            let visible: i32 = self
                .view()
                .units
                .iter()
                .filter(|u| {
                    u.military
                        && !self.view().same_team(self.ctx.me, u.owner)
                        && (u.pos.distance(s.pos) <= radius
                            || path.iter().any(|h| u.pos.distance(*h) <= radius))
                })
                .map(|u| threat_power(self.view(), u))
                .sum();
            let remembered: i32 = self
                .ctx
                .remembered_threats
                .iter()
                .filter(|u| {
                    u.pos.distance(s.pos) <= radius
                        || path.iter().any(|h| u.pos.distance(*h) <= radius)
                })
                .map(|u| {
                    self.view()
                        .rule_experiments
                        .unit_stats_for(self.view().faction_of(u.owner), u.kind)
                        .strength
                        / 2
                })
                .sum();
            if (visible + remembered) * g.escort_civilians_weight / 100 < g.escort_civilians_threat
            {
                continue;
            }
            let escort = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| {
                    u.military
                        && u.kind != UnitKind::Scout
                        && !self.has_acted(u.id)
                        && u.hp >= self.decisions.recover_hp
                        && u.pos.distance(s.pos) <= radius
                        && self.ctx.reachable(u.pos, s.pos)
                })
                .min_by_key(|u| (u.pos.distance(s.pos), u.id));
            let at_home = self.ctx.own_cities.iter().any(|c| c.pos == s.pos);
            let waited = self.view().turn.saturating_sub(
                p.memory
                    .waiting_since
                    .get(&s.id)
                    .copied()
                    .unwrap_or(self.view().turn),
            );
            if escort.is_none() && at_home && waited >= g.escort_civilians_wait as u32 {
                continue;
            }
            let step = escort.filter(|u| u.pos == s.pos).and_then(|u| {
                path.first()
                    .copied()
                    .filter(|h| self.free_tile_for(u, *h) && self.safe_campaign_tile(*h))
            });
            let shelter = if escort.is_none() && !at_home {
                self.ctx
                    .nearest_own_city(s.pos)
                    .and_then(|c| self.campaign_approach(s, c.pos))
            } else {
                None
            };
            self.orders
                .retain(|o| legatum_core::rules::units::order_unit(o) != Some(s.id));
            self.orders.push(
                step.or(shelter)
                    .map_or(Order::SkipUnit { unit: s.id }, |to| Order::MoveUnit {
                        unit: s.id,
                        to,
                    }),
            );
            if let Some(u) = escort {
                self.go_to(u, step.unwrap_or(s.pos));
            }
        }
        true
    }

    fn intel_stale(&self) -> bool {
        let Some(p) = self.primitives else {
            return false;
        };
        p.memory.last_seen.iter().any(|(team, seen)| {
            self.view()
                .players
                .iter()
                .any(|x| x.team == *team && x.alive)
                && self.view().turn.saturating_sub(*seen) > p.params.scout_upkeep_intel_age as u32
        })
    }
    pub(crate) fn primitive_scout_target_count(&self) -> i32 {
        let Some(p) = self.primitives else { return 0 };
        if self.rushing()
            && self.strike_city.is_some()
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.kind != UnitKind::Scout)
                .count()
                >= 2
        {
            return 0;
        }
        if p.params.scout_upkeep_weight == 0 {
            return 0;
        }
        let early = self.view().turn <= p.params.scout_upkeep_until as u32;
        (if early {
            p.params.scout_upkeep_minimum
        } else {
            0
        })
        .max(i32::from(self.intel_stale()))
    }
    pub(crate) fn primitive_scout_target(&self, s: &UnitView) -> Option<Hex> {
        if !self.intel_stale() || self.primitive_scout_target_count() == 0 {
            return None;
        }
        let p = self.primitives?;
        let mut cities: Vec<_> = self
            .view()
            .cities
            .iter()
            .filter(|c| !c.visible && !self.view().same_team(self.ctx.me, c.owner))
            .collect();
        cities.sort_by_key(|c| {
            (
                p.memory
                    .last_seen
                    .get(&self.view().team_of(c.owner))
                    .copied()
                    .unwrap_or(0),
                c.id,
            )
        });
        for c in cities {
            let mut spots = c.pos.within(s.sight.unwrap_or(data::UNIT_SIGHT));
            spots.sort_by_key(|h| (s.pos.distance(*h), *h));
            if let Some(to) = spots
                .into_iter()
                .filter(|h| *h != s.pos && self.free_tile_for(s, *h))
                .find_map(|h| self.campaign_approach(s, h))
            {
                return Some(to);
            }
        }
        None
    }
    fn scouts_committed(&self) -> i32 {
        self.ctx.own_units.iter().filter(|u| u.kind == UnitKind::Scout).count() as i32
            + self.ctx.own_cities.iter().filter(|c| c.detail.as_ref().is_some_and(|d| d.producing == Some(unit(UnitKind::Scout))
                || d.production_queue.contains(&unit(UnitKind::Scout)))) .count() as i32
            + self.orders.iter().filter(|o| matches!(o, Order::SetProduction { item, .. } | Order::BuyItem { item, .. } if *item == unit(UnitKind::Scout))).count() as i32
    }
    pub(crate) fn primitive_production_bonus(&self, _c: &CityView, opt: &ProductionOption) -> i32 {
        let Some(p) = self.primitives else { return 0 };
        let g = p.params;
        if opt.item == unit(UnitKind::Scout)
            && self.scouts_committed() < self.primitive_scout_target_count()
        {
            return 120 * g.scout_upkeep_weight;
        }
        let ProductionItem::Building { building: b } = opt.item else {
            return 0;
        };
        if g.market_first_economy_weight == 0 {
            return 0;
        }
        // Core stacks every tier with its predecessors; an upgrade adds its full
        // listed yield, rather than replacing the preceding tier's contribution.
        let gain = self
            .view()
            .rule_experiments
            .building_stats_for(self.faction, b)
            .yields
            .gold
            .max(0);
        let horizon = g.market_first_economy_horizon;
        let payoff = gain * (horizon - opt.turns.max(0)).max(0);
        // Gold-valued opportunity cost, from the same quoted price used by the purchaser.
        let cost = opt.buy_cost.unwrap_or(opt.cost * 2).max(1);
        if payoff < cost {
            return 0;
        }
        (payoff * 2000 / cost).min(12000) * g.market_first_economy_weight / 100
    }

    pub(crate) fn primitive_priority_production(&self, c: &CityView) -> Option<ProductionItem> {
        c.detail
            .as_ref()?
            .options
            .iter()
            .filter_map(|o| {
                let score = self.primitive_production_bonus(c, o);
                (score > 10000).then_some((score, o.item))
            })
            .max()
            .map(|(_, item)| item)
    }

    pub(crate) fn primitive_gold_reserve(&self) -> Option<i32> {
        let p = self.primitives?;
        let g = p.params;
        if g.save_and_burst_weight == 0 {
            return None;
        }
        let price = self
            .ctx
            .own_cities
            .iter()
            .filter_map(|c| c.detail.as_ref())
            .flat_map(|d| &d.options)
            .filter(|o| o.item == unit(UnitKind::Ranger))
            .filter_map(|o| o.buy_cost)
            .min()
            .unwrap_or(0);
        let mut missing = self.primitive_numbers().war_army;
        if self.rushing() {
            missing = (missing
                - self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| u.military && u.kind != UnitKind::Scout)
                    .count() as i32)
                .max(0);
        }
        let reserve = price * missing * g.save_and_burst_reserve_percent / 100;
        Some((reserve * g.save_and_burst_weight / 100).max(
            self.decisions.reserve_deficit_turns
                * (-self.view().economy.gold_per_turn.total).max(0),
        ))
    }

    pub(crate) fn plan_primitive_buy(&mut self) -> bool {
        let Some(p) = self.primitives else {
            return false;
        };
        if p.params.save_and_burst_weight == 0 {
            return false;
        }
        let n = self.primitive_numbers();
        let army: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .collect();
        let war = self.strike_target.is_some() || self.rushing();
        let field_settlers = self
            .ctx
            .own_units
            .iter()
            .any(|u| u.kind == UnitKind::Settler)
            || self.ctx.own_cities.iter().any(|c| {
                c.detail.as_ref().is_some_and(|d| {
                    d.producing == Some(unit(UnitKind::Settler))
                        || d.production_queue.contains(&unit(UnitKind::Settler))
                })
            });
        let expand = !war
            && !field_settlers
            && self.opening_admits_settler()
            && self.count_good_sites(1) > 0;
        if !war && !expand {
            return false;
        }
        let count = if war {
            (n.war_army - army.len() as i32)
                .max(0)
                .min((n.burst_count * p.params.save_and_burst_weight / 100).max(1))
        } else {
            1
        };
        let mut need_melee = army
            .iter()
            .filter(|u| crate::composition::charges(u.kind))
            .count()
            < if self.rushing() {
                (n.war_army as usize * 3 / 5).max(1)
            } else {
                1
            };
        let mut cities = self.ctx.own_cities.clone();
        let target = self
            .strike_target
            .and_then(|t| {
                self.view()
                    .cities
                    .iter()
                    .find(|c| self.view().team_of(c.owner) == t)
            })
            .map(|c| c.pos);
        cities.sort_by_key(|c| (target.map_or(0, |h| c.pos.distance(h)), c.id));
        let runway = self.decisions.reserve_deficit_turns
            * (-self.view().economy.gold_per_turn.total).max(0);
        let mut bought = 0;
        for c in cities {
            if bought >= count {
                break;
            }
            if self
                .orders
                .iter()
                .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
            {
                continue;
            }
            let item = unit(if expand {
                UnitKind::Settler
            } else if need_melee {
                UnitKind::Soldier
            } else {
                UnitKind::Ranger
            });
            if !self.opening_allows_optional_military(item)
                || expand && !self.opening_admits_settler()
            {
                continue;
            }
            let Some(opt) = c.detail.as_ref().and_then(|d| {
                d.options
                    .iter()
                    .find(|o| o.item == item && o.buy_block.is_none())
            }) else {
                continue;
            };
            let Some(cost) = opt
                .buy_cost
                .filter(|cost| *cost > 0 && self.gold_left - *cost >= runway)
            else {
                continue;
            };
            self.orders.push(Order::BuyItem { city: c.id, item });
            self.gold_left -= cost;
            need_melee = false;
            bought += 1;
        }
        bought > 0
    }

    pub(crate) fn primitive_persist_siege(&self, team: TeamId) -> bool {
        let Some(p) = self.primitives else {
            return false;
        };
        let g = p.params;
        if g.siege_persistence_weight == 0 {
            return false;
        }
        let hp = (g.siege_persistence_hp_percent * g.siege_persistence_weight / 100).min(100);
        self.ctx.enemy_cities.iter().any(|c| {
            c.visible
                && self.view().team_of(c.owner) == team
                && c.hp * 100 < c.max_hp * hp
                && self.ctx.own_units.iter().any(|u| {
                    u.military
                        && u.kind != UnitKind::Scout
                        && u.pos.distance(c.pos) <= u.range.max(1)
                })
        })
    }

    pub(crate) fn primitive_barbarian_guard(&self, ci: usize) -> bool {
        self.primitives.is_some_and(|p| {
            p.params.barbarian_settler_guard_weight > 0
                && self.ctx.barbarian_threat[ci] > 0
                && self.ctx.barbarian_threat[ci] * p.params.barbarian_settler_guard_weight / 100
                    >= p.params.barbarian_settler_guard_threat
                && self.ctx.threat[ci] == self.ctx.barbarian_threat[ci]
        })
    }
    pub(crate) fn plan_primitive_barbarian_guards(&mut self) {
        for (ci, c) in self.ctx.own_cities.clone().into_iter().enumerate() {
            if !self.primitive_barbarian_guard(ci)
                || c.detail
                    .as_ref()
                    .is_none_or(|d| d.producing != Some(unit(UnitKind::Settler)))
            {
                continue;
            }
            let guard = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| {
                    u.military
                        && u.kind != UnitKind::Scout
                        && !self.has_acted(u.id)
                        && self.ctx.reachable(u.pos, c.pos)
                })
                .min_by_key(|u| (u.pos.distance(c.pos), u.id));
            if let Some(u) = guard
                && self.free_tile_for(u, c.pos)
            {
                self.go_to(u, c.pos);
            }
        }
    }

    pub(crate) fn plan_primitive_coalition_march(&mut self) {
        let Some(target) = self.primitive_coalition_target() else {
            return;
        };
        let (city, destination, team) = (target.id, target.pos, self.view().team_of(target.owner));
        if !self.view().teams_at_war(self.view().team, team) {
            return;
        }
        let g = self.primitives.expect("coalition primitive").params;
        let army: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .collect();
        let minimum = (army.len() * g.coalition_march_force_percent as usize)
            .div_ceil(100)
            .max(1);
        let desired = (minimum * g.coalition_march_weight as usize)
            .div_ceil(100)
            .max(minimum)
            .min(army.len());
        let mut free: Vec<_> = army
            .into_iter()
            .filter(|u| {
                !self.has_acted(u.id)
                    && u.hp >= self.decisions.recover_hp
                    && self.ctx.reachable(u.pos, destination)
            })
            .collect();
        free.sort_by_key(|u| (u.pos.distance(destination), u.id));
        let mut moves = Vec::new();
        for u in free {
            if moves.len() >= desired {
                break;
            }
            // Already fighting at the objective is not a new march and must keep its attack.
            if u.pos.distance(destination) <= u.range.max(1) {
                continue;
            }
            let mut spots = destination.within(3);
            spots.sort_by_key(|h| (u.pos.distance(*h), h.distance(destination), *h));
            if let Some(to) = spots
                .into_iter()
                .filter(|h| {
                    h.distance(destination) >= 1
                        && self.free_tile_for(u, *h)
                        && !moves.iter().any(|(_, _, to)| *to == *h)
                })
                .find_map(|h| self.campaign_approach(u, h))
                .filter(|h| h.distance(destination) < u.pos.distance(destination))
            {
                moves.push((u, u.pos, to));
            }
        }
        if moves.len() < minimum {
            return;
        }
        let units = moves.iter().map(|(u, from, _)| (u.id, *from)).collect();
        for (u, _, to) in moves {
            self.go_to(u, to);
        }
        self.primitive_march = Some(CoalitionMarch {
            city,
            target: destination,
            minimum,
            units,
        });
    }
}

#[cfg(test)]
mod tests;

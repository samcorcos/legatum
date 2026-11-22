//! World Age cards for the scripted AI (Hard, and Hard + Search through the same planner).
//!
//! Each Age's card is public and worth 1 permanent VP. Each turn the AI picks the live (begun,
//! unscored) card with the least estimated effort and biases production, tile buying and
//! campaign targets towards it. Old unscored Ages are worth the same as new ones.

use crate::DecisionParams;
use crate::scripted::Planner;
use legatum_core::objectives::Objective;
use legatum_core::view::CityView;
use legatum_core::{PlayerView, ProductionItem, UnitKind, data};

/// An objective of an era the AI has entered and not yet scored.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LiveObjective {
    pub era: u8,
    pub slot: u8,
    pub objective: Objective,
    pub current: u32,
    pub target: u32,
}

impl LiveObjective {
    pub fn remaining(&self) -> u32 {
        self.target.saturating_sub(self.current)
    }
}

/// How heavy a rival's lead penalty is: its decadence (the cut from every yield)
/// weighted by `decadence_weight`.
pub(crate) fn burden(
    view: &PlayerView,
    rival: legatum_core::PlayerId,
    genes: &DecisionParams,
) -> i32 {
    if view.rule_experiments.decadence_slope() == 0 {
        return 0;
    }
    view.players
        .iter()
        .find(|p| p.id == rival)
        .map_or(0, |p| p.decadence_percent * genes.decadence_weight / 100)
}

/// Whether an opted-in decadence rule cuts a rival's yields enough to meet the strain threshold.
pub(crate) fn strained(
    view: &PlayerView,
    rival: legatum_core::PlayerId,
    genes: &DecisionParams,
) -> bool {
    genes.strain_percent > 0
        && view
            .players
            .iter()
            .any(|p| p.id == rival && p.team != view.team && p.alive)
        && burden(view, rival, genes) >= genes.strain_percent
}

/// How hard a rival's strain pulls a campaign towards its cities: its burden percent times
/// `strain_priority` per hundred.
pub(crate) fn strain_value(
    view: &PlayerView,
    rival: legatum_core::PlayerId,
    genes: &DecisionParams,
) -> i32 {
    if !strained(view, rival, genes) {
        return 0;
    }
    burden(view, rival, genes) * genes.strain_priority / 100
}

impl Planner<'_> {
    /// Objectives we could score now, with our progress.
    pub(crate) fn live_objectives(&self) -> Vec<LiveObjective> {
        let v = self.view();
        v.objective_progress
            .iter()
            .filter(|p| p.entered && !p.era_scored)
            .filter_map(|p| {
                let era = v.eras.iter().find(|e| e.era == p.era)?;
                let o = era.objectives.iter().find(|o| o.slot == p.slot)?;
                Some(LiveObjective {
                    era: p.era,
                    slot: p.slot,
                    objective: o.objective,
                    current: p.current,
                    target: p.target,
                })
            })
            .collect()
    }

    /// The live objectives the AI works towards: none when the `objective_seek` gene is 0.
    fn sought_objectives(&self) -> Vec<LiveObjective> {
        if self.decisions.objective_seek == 0 {
            return Vec::new();
        }
        self.live_objectives()
    }

    /// Estimated turns to meet an objective, or None when it is out of reach for now.
    fn objective_effort(&self, o: &LiveObjective) -> Option<i32> {
        if self.difficulty == legatum_core::AiDifficulty::Hard
            && self.decisions.cheapest_objective > 0
        {
            return self.costed_objective_effort(o);
        }
        self.legacy_objective_effort(o)
    }

    fn legacy_objective_effort(&self, o: &LiveObjective) -> Option<i32> {
        let v = self.view();
        let rem = o.remaining() as i32;
        let cities = self.ctx.own_cities.len().max(1) as i32;
        let at_war = self.ctx.hostile_at.iter().any(|h| *h) && !self.ctx.enemy_cities.is_empty();
        let armed = self.ctx.own_units.iter().filter(|u| u.military).count() >= 3;
        let hard = self.difficulty == legatum_core::AiDifficulty::Hard;
        let rival_vp = |f: &dyn Fn(u32) -> bool| {
            self.ctx
                .enemy_cities
                .iter()
                .any(|c| v.players.iter().any(|p| p.id == c.owner && f(p.vp)))
        };
        let my_vp = v.players.iter().find(|p| p.id == v.me).map_or(0, |p| p.vp);
        match o.objective {
            Objective::HumbleTheMighty => {
                (hard && at_war && armed && rival_vp(&|vp| vp > my_vp)).then_some(120)
            }
            Objective::Breach => (hard
                && at_war
                && armed
                && self
                    .ctx
                    .enemy_cities
                    .iter()
                    .any(|c| c.walls && !c.is_capital))
            .then_some(130),
            Objective::Cities { .. } => {
                let sites = self.count_good_sites(rem + 1);
                if sites >= rem {
                    Some(rem * 24)
                } else if self.faction == legatum_core::Faction::India
                    && self.decisions.india_objective_reach > 0
                    && self.conquest_city().is_some()
                {
                    Some(sites * 24 + (rem - sites) * 35)
                } else if self.decisions.vp_two_step > 0
                    && self.ctx.enemy_cities.len() as i32 >= rem - sites
                {
                    let readiness = 25 * i32::from(!at_war) + 25 * i32::from(!armed);
                    Some(sites * 24 + (rem - sites) * (45 + readiness))
                } else {
                    None
                }
            }
            Objective::Population { .. } => Some(rem * 20 / cities),
            Objective::Territory { .. } => Some(rem * 6 / cities),
            Objective::Buildings { building, .. } => {
                if data::faction_excluded_building_at_rules(
                    self.view().rule_experiments.faction_rules_off,
                    self.faction,
                    building,
                    self.view().era_of(self.view().me),
                )
                .is_some()
                {
                    return None;
                }
                let lacking = self
                    .ctx
                    .own_cities
                    .iter()
                    .filter(|c| {
                        c.detail
                            .as_ref()
                            .is_some_and(|d| !data::has_building(&d.buildings, building))
                    })
                    .count() as i32;
                let expansion = if lacking < rem {
                    if self.count_good_sites(rem - lacking) < rem - lacking {
                        return None;
                    }
                    (rem - lacking) * 24
                } else {
                    0
                };
                let tech_wait = data::building_stats(building)
                    .requires
                    .filter(|t| !v.research.known.contains(t))
                    .map_or(0, |_| 50);
                Some(tech_wait + rem * 18 + expansion)
            }
            // A promotion takes a few won fights by one unit: dearer than a kill, only at war.
            Objective::Veterans { .. } => (hard && at_war && armed).then_some(rem * 25),
        }
    }

    /// Turns of work for one permanent VP. Production is priced in empire turns as well as
    /// elapsed turns: four builds in parallel still divert four cities from their other plans.
    /// Only our city options, buildings, income and visible enemy cities enter the estimate.
    fn costed_objective_effort(&self, o: &LiveObjective) -> Option<i32> {
        let rem = o.remaining() as i32;
        let v = self.view();
        let cities = self.ctx.own_cities.len().max(1) as i32;
        match o.objective {
            Objective::Buildings { building, .. } => {
                if data::faction_excluded_building_at_rules(
                    v.rule_experiments.faction_rules_off,
                    self.faction,
                    building,
                    v.era_of(v.me),
                )
                .is_some()
                {
                    return None;
                }
                let mut choices = Vec::new();
                let mut empire_rate = 0;
                for city in &self.ctx.own_cities {
                    let Some(d) = &city.detail else { continue };
                    empire_rate += d.yields.total.production.max(1);
                    if data::has_building(&d.buildings, building) {
                        continue;
                    }
                    let rules = self.view().rule_experiments;
                    let mut cost = rules.building_stats_for(self.faction, building).cost;
                    if let Some(predecessor) = data::building_predecessor(building)
                        && !data::has_building(&d.buildings, predecessor)
                    {
                        cost += rules.building_stats_for(self.faction, predecessor).cost;
                    }
                    let wait_for_population =
                        (data::building_min_population(building) - city.population).max(0)
                            * d.turns_to_grow.unwrap_or(18).max(4);
                    let wait_for_tech = data::building_stats(building)
                        .requires
                        .filter(|tech| !v.research.known.contains(tech))
                        .map_or(0, |_| 30);
                    let option = d
                        .options
                        .iter()
                        .find(|opt| opt.item == ProductionItem::Building { building });
                    let turns = option
                        .filter(|opt| opt.resource_blocker.is_none() && opt.turns >= 0)
                        .map_or_else(
                            || {
                                (cost + d.yields.total.production.max(1) - 1)
                                    / d.yields.total.production.max(1)
                            },
                            |opt| opt.turns.max(1),
                        );
                    let supply_wait = if option.is_some_and(|opt| opt.resource_blocker.is_some()) {
                        25
                    } else {
                        0
                    };
                    choices.push((
                        turns + wait_for_population + wait_for_tech + supply_wait,
                        cost,
                    ));
                }
                if choices.len() < rem as usize {
                    return None;
                }
                choices.sort_by_key(|(turns, cost)| (*turns, *cost));
                let selected = &choices[..rem as usize];
                let elapsed = selected.iter().map(|(turns, _)| *turns).max().unwrap_or(1);
                let work: i32 = selected.iter().map(|(_, cost)| *cost).sum();
                let supply_wait = legatum_core::resources::item_need(
                    ProductionItem::Building { building },
                )
                .map_or(0, |resource| {
                    let spare = v.supply.iter().find(|s| s.resource == resource).map_or(0, |s| s.spare);
                    let already_held = self.ctx.own_cities.iter().filter(|city| {
                        city.detail.as_ref().is_some_and(|d| matches!(d.producing,
                            Some(ProductionItem::Building { building: b }) if data::building_satisfies(b, building)))
                    }).count() as i32;
                    (rem - already_held - spare).max(0) * 40
                });
                Some(elapsed + work / empire_rate.max(1) / 2 + supply_wait)
            }
            Objective::Territory { .. } => {
                // Border growth is spread across cities, but a newly acquired tile is not
                // free each turn. The old six/city made a long land count look instant.
                Some((rem * 24 + cities - 1) / cities + 8)
            }
            Objective::HumbleTheMighty | Objective::Breach => {
                let my_vp = v.players.iter().find(|p| p.id == v.me).map_or(0, |p| p.vp);
                let target = self
                    .ctx
                    .enemy_cities
                    .iter()
                    .filter(|city| {
                        let owner_vp = v
                            .players
                            .iter()
                            .find(|p| p.id == city.owner)
                            .map_or(0, |p| p.vp);
                        match o.objective {
                            Objective::HumbleTheMighty => owner_vp > my_vp && !city.is_capital,
                            Objective::Breach => city.walls && !city.is_capital,
                            _ => true,
                        }
                    })
                    .filter_map(|city| {
                        let march = self
                            .ctx
                            .own_units
                            .iter()
                            .filter(|u| u.military)
                            .map(|u| u.pos.distance(city.pos))
                            .min()?;
                        let strongest = self
                            .ctx
                            .own_units
                            .iter()
                            .filter(|u| u.military)
                            .map(|u| u.strength.max(u.ranged))
                            .max()
                            .unwrap_or(0);
                        let visible_guards = self
                            .ctx
                            .enemy_units
                            .iter()
                            .filter(|u| {
                                u.owner == city.owner && u.military && u.pos.distance(city.pos) <= 1
                            })
                            .count() as i32;
                        Some(
                            march * 2
                                + if city.walls { 32 } else { 8 }
                                + (city.strength - strongest).max(0) * 2
                                + visible_guards * 12
                                + if v.at_war(v.me, city.owner) { 0 } else { 16 },
                        )
                    })
                    .min();
                target.map(|turns| turns + (rem - 1).max(0) * 25)
            }
            _ => self.legacy_objective_effort(o),
        }
    }

    /// The live objective the AI works towards: the least effort, the oldest era on a tie.
    pub(crate) fn chosen_objective(&self) -> Option<(LiveObjective, i32)> {
        let horizon = self.decisions.objective_horizon;
        let options: Vec<_> = self
            .sought_objectives()
            .into_iter()
            .filter_map(|o| self.objective_effort(&o).map(|e| (o, e)))
            .filter(|(_, e)| *e <= horizon)
            .collect();
        let own = self
            .view()
            .standings
            .iter()
            .find(|s| s.team == self.view().team);
        let needs_second = own.is_some_and(|s| s.vp + 1 < s.target);
        if self.difficulty == legatum_core::AiDifficulty::Hard
            && self.decisions.cheapest_objective > 0
        {
            // Every open era card pays one VP. The lowest expected time plus diverted
            // production therefore has the highest VP per unit of work.
            return options
                .into_iter()
                .min_by_key(|(o, effort)| (*effort, o.era, o.slot));
        }
        options
            .iter()
            .min_by_key(|(o, e)| {
                let next = if self.decisions.vp_two_step == 0 || !needs_second {
                    0
                } else {
                    options
                        .iter()
                        .filter(|(other, _)| other.era != o.era)
                        .map(|(other, effort)| self.followup_effort(o, other, *effort))
                        .min()
                        .unwrap_or(horizon)
                };
                (*e + next * self.decisions.vp_two_step / 100, o.era, o.slot)
            })
            .copied()
    }

    /// One objective can create progress toward a second one. A city gained on the way to
    /// the first point also contributes its visible population and at least its inner ring;
    /// a capture can do the same if the city survives. These are conservative floors, not
    /// assumed scores from undisclosed rival progress.
    fn followup_effort(&self, first: &LiveObjective, next: &LiveObjective, effort: i32) -> i32 {
        let gained_cities = match first.objective {
            Objective::Cities { .. } => first.remaining() as i32,
            Objective::HumbleTheMighty | Objective::Breach if !self.sacks() => {
                first.remaining() as i32
            }
            _ => 0,
        };
        let cities = self.ctx.own_cities.len().max(1) as i32;
        let discount = match next.objective {
            Objective::Cities { .. } => gained_cities * 24,
            Objective::Population { .. } => gained_cities * 2 * 20 / cities,
            Objective::Territory { .. } => gained_cities * 7 * 6 / cities,
            _ => 0,
        };
        (effort - discount).max(1)
    }

    /// The chosen objective is one step from done: worth a victory-planner pass.
    pub(crate) fn objective_close(&self) -> bool {
        self.chosen_objective()
            .is_some_and(|(o, _)| o.remaining() <= 2 || o.current * 100 >= o.target * 70)
    }

    /// Extra percent on a production item's value when it advances the chosen objective.
    pub(crate) fn objective_item_bonus(&self, c: &CityView, item: ProductionItem) -> i32 {
        let Some((o, _)) = self.chosen_objective() else {
            return 0;
        };
        if self.difficulty == legatum_core::AiDifficulty::Hard
            && self.decisions.cheapest_objective > 0
            && matches!(o.objective, Objective::Buildings { .. })
        {
            return 0;
        }
        let pct = self.decisions.objective_bonus_percent.max(500);
        match (o.objective, item) {
            (
                Objective::Cities { .. },
                ProductionItem::Unit {
                    unit: UnitKind::Settler,
                },
            ) => pct,
            (Objective::Buildings { building, .. }, ProductionItem::Building { building: b })
                if data::building_satisfies(b, building)
                    && c.detail
                        .as_ref()
                        .is_some_and(|d| !data::has_building(&d.buildings, building)) =>
            {
                pct
            }
            (
                Objective::Buildings { cities, .. },
                ProductionItem::Unit {
                    unit: UnitKind::Settler,
                },
            ) if self.ctx.own_cities.len() < cities as usize => pct,
            (Objective::Population { .. }, ProductionItem::Building { building: b })
                if self
                    .view()
                    .rule_experiments
                    .building_stats_for(self.faction, b)
                    .yields
                    .food
                    > 0 =>
            {
                pct / 2
            }
            _ => 0,
        }
    }

    /// Production value of the next step toward a chosen one-VP building card. A prerequisite
    /// gets the same share of the point, discounted by its extra build step. Count existing
    /// queues and orders so two needed Castles produce two orders, not twelve.
    pub(crate) fn objective_build_value(&self, c: &CityView, item: ProductionItem) -> i32 {
        if self.difficulty != legatum_core::AiDifficulty::Hard
            || self.decisions.cheapest_objective == 0
        {
            return 0;
        }
        let Some((o, effort)) = self.chosen_objective() else {
            return 0;
        };
        if o.remaining() == 0 {
            return 0;
        }
        let Objective::Buildings { building, .. } = o.objective else {
            return 0;
        };
        let ProductionItem::Building {
            building: candidate,
        } = item
        else {
            return 0;
        };
        let Some(detail) = &c.detail else { return 0 };
        if detail
            .options
            .iter()
            .find(|opt| opt.item == item)
            .is_some_and(|opt| opt.resource_blocker.is_some())
        {
            return 0;
        }
        if data::has_building(&detail.buildings, building) {
            return 0;
        }
        let direct = data::building_satisfies(candidate, building);
        let prerequisite = data::building_predecessor(building) == Some(candidate)
            && !data::has_building(&detail.buildings, candidate);
        if !direct && !prerequisite {
            return 0;
        }
        let matching = |other: &CityView| {
            let Some(d) = &other.detail else { return false };
            let planned = self
                .orders
                .iter()
                .find_map(|order| match order {
                    legatum_core::Order::SetProduction { city, item } if *city == other.id => {
                        Some(*item)
                    }
                    _ => None,
                })
                .or(d.producing);
            matches!(planned, Some(ProductionItem::Building { building: b })
                if data::building_satisfies(b, building)
                    || data::building_predecessor(building) == Some(b))
        };
        let assigned = self
            .ctx
            .own_cities
            .iter()
            .filter(|other| other.id != c.id && matching(other))
            .count();
        if assigned >= o.remaining() as usize {
            return 0;
        }
        let share = 1400 / (o.remaining() as i32).max(1);
        let urgency = (self.decisions.objective_horizon - effort).max(0);
        let value = share * (100 + urgency * 50 / self.decisions.objective_horizon.max(1)) / 100;
        if prerequisite { value * 2 / 3 } else { value }
    }

    /// Points (VP) an item would add towards the chosen objective by completing it.
    pub(crate) fn objective_item_points(&self, c: &CityView, item: ProductionItem) -> u32 {
        let Some((o, _)) = self.chosen_objective() else {
            return 0;
        };
        match (o.objective, item) {
            (
                Objective::Cities { .. },
                ProductionItem::Unit {
                    unit: UnitKind::Settler,
                },
            ) => {
                let pending = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| u.kind == UnitKind::Settler)
                    .count() as u32;
                u32::from(o.current + pending + 1 >= o.target)
            }
            (Objective::Buildings { building, .. }, ProductionItem::Building { building: b }) => {
                u32::from(
                    o.remaining() <= 1
                        && data::building_satisfies(b, building)
                        && c.detail
                            .as_ref()
                            .is_some_and(|d| !data::has_building(&d.buildings, building)),
                )
            }
            _ => 0,
        }
    }

    /// 1 when taking `city` meets a live Fight objective at once; the AI weighs this in
    /// campaign targets next to the VP the capture denies.
    pub(crate) fn objective_capture_value(&self, city: &CityView) -> i32 {
        let v = self.view();
        let live = self.sought_objectives();
        if city.owner.is_barbarian() || v.team_of(city.owner) == v.team {
            return 0;
        }
        if live.is_empty() {
            return 0;
        }
        let owner_vp = v
            .players
            .iter()
            .find(|p| p.id == city.owner)
            .map_or(0, |p| p.vp);
        let my_vp = v.players.iter().find(|p| p.id == v.me).map_or(0, |p| p.vp);
        // Gene `capture_completes_count`: the captured city is one more city and brings its
        // owner's tiles around it (the ring is a floor: the city's whole border comes with it).
        let counts = self.decisions.capture_completes_count > 0 && !self.sacks_city(city);
        let land = city
            .pos
            .within(1)
            .iter()
            .filter(|h| v.tile(**h).is_some_and(|t| t.owner == Some(city.owner)))
            .count() as i32;
        let worth = |o: &LiveObjective| match o.objective {
            Objective::Cities { .. } => counts && o.remaining() == 1,
            Objective::Territory { .. } => counts && land >= 1 && o.remaining() as i32 <= land,
            Objective::HumbleTheMighty => {
                o.remaining() == 1 && owner_vp > my_vp && !city.is_capital
            }
            Objective::Breach => o.remaining() == 1 && city.walls && !city.is_capital,
            _ => false,
        };
        let point_value = if self.decisions.vp_race > 0 {
            1
        } else {
            (self.decisions.objective_bonus_percent / 20).max(1)
        };
        i32::from(live.iter().any(worth)) * point_value
    }

    /// Completing the last missing technology opens the next era's objectives.
    pub(crate) fn era_entry_bonus(&self, tech: legatum_core::TechId) -> i32 {
        let v = self.view();
        let mine = v.players.iter().find(|p| p.id == v.me).map_or(1, |p| p.era);
        let enters = if self.decisions.era_transition_scoring == 0 {
            data::era_index(tech) > mine
        } else {
            v.research.era < legatum_core::eras::LAST_ERA
                && legatum_core::eras::era_of_tech(tech) == v.research.era
                && !v.research.known.contains(&tech)
                && v.research.era_known + 1 == v.research.era_total
        };
        if enters {
            self.decisions.era_entry_percent
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};

    #[test]
    fn final_technology_of_each_era_receives_the_transition_bonus() {
        for era in 0..legatum_core::eras::LAST_ERA {
            let last = legatum_core::eras::era_techs(era).last().unwrap();
            let known: Vec<_> = legatum_core::TechId::ALL
                .into_iter()
                .filter(|t| {
                    legatum_core::eras::era_of_tech(*t) < era
                        || (legatum_core::eras::era_of_tech(*t) == era && *t != last)
                })
                .collect();
            let mut view = ViewBuilder::new(16, 16)
                .known_techs(&known)
                .city(1, ME, off(4, 4), 4)
                .build();
            // The fixture stores zero-based public eras; the resolver exposes one-based eras.
            view.players[0].era = era + 1;
            let mut planner = crate::fixture::planner(&view, 7);
            planner.decisions.era_entry_percent = 37;
            assert_eq!(planner.era_entry_bonus(last), 37, "era {era}");
            planner.decisions.era_entry_percent = 0;
            assert_eq!(planner.era_entry_bonus(last), 0, "era {era}");
        }
    }

    fn count_view(objective: Objective, current: u32, target: u32) -> PlayerView {
        use legatum_core::objectives::Family;
        use legatum_core::view::{EraView, ObjectiveProgress, ObjectiveView};
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(4, 8), 6)
            .city(2, ENEMY, off(14, 8), 5)
            .build();
        v.eras = vec![EraView {
            era: 4,
            name: "Industrial".into(),
            objectives: vec![ObjectiveView {
                slot: 1,
                family: Family::Expand,
                text: String::new(),
                target,
                objective,
            }],
            entered_by: vec![ME],
            scored_by: vec![],
        }]
        .into();
        v.objective_progress = vec![ObjectiveProgress {
            era: 4,
            slot: 1,
            current,
            target,
            entered: true,
            era_scored: false,
            entered_turn: None,
            ready: None,
        }];
        v
    }

    fn capture_value(v: &PlayerView, gene: i32) -> i32 {
        let faction = v.faction_of(ME);
        let mut p = Planner::new(v, crate::Knobs::for_faction(faction), faction, 7);
        p.decisions.capture_completes_count = gene;
        p.objective_capture_value(&v.cities[1])
    }

    #[test]
    fn cheapest_card_prices_real_build_work_and_stops_at_the_needed_queues() {
        use legatum_core::BuildingKind;
        use legatum_core::objectives::Family;
        use legatum_core::view::{ObjectiveProgress, ObjectiveView};
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(4, 8), 6)
            .city(2, ME, off(8, 8), 6)
            .city(3, ME, off(12, 8), 6)
            .city(4, ENEMY, off(19, 8), 5)
            .build();
        let objective = Objective::Buildings {
            building: BuildingKind::Castle,
            cities: 2,
        };
        v.eras = vec![legatum_core::view::EraView {
            era: 3,
            name: "Medieval".into(),
            objectives: vec![
                ObjectiveView {
                    slot: 0,
                    family: Family::Build,
                    text: String::new(),
                    target: 2,
                    objective,
                },
                ObjectiveView {
                    slot: 1,
                    family: Family::Expand,
                    text: String::new(),
                    target: 30,
                    objective: Objective::Territory { n: 30 },
                },
            ],
            entered_by: vec![ME],
            scored_by: vec![],
        }]
        .into();
        v.objective_progress = vec![
            ObjectiveProgress {
                era: 3,
                slot: 0,
                current: 0,
                target: 2,
                entered: true,
                era_scored: false,
                entered_turn: None,
                ready: None,
            },
            ObjectiveProgress {
                era: 3,
                slot: 1,
                current: 0,
                target: 30,
                entered: true,
                era_scored: false,
                entered_turn: None,
                ready: None,
            },
        ];
        let faction = v.faction_of(ME);
        let mut p = Planner::new(&v, crate::Knobs::for_faction(faction), faction, 7);
        p.decisions.cheapest_objective = 1;
        p.difficulty = legatum_core::AiDifficulty::Hard;
        assert!(matches!(
            p.chosen_objective().unwrap().0.objective,
            Objective::Buildings { .. }
        ));
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        assert!(p.objective_build_value(&v.cities[0], walls) > 0);
        // Other cities' already assigned prerequisite work consumes the two slots.
        let mut v = v;
        v.cities[1].detail.as_mut().unwrap().producing = Some(walls);
        v.cities[2].detail.as_mut().unwrap().producing = Some(walls);
        let mut p = Planner::new(&v, crate::Knobs::for_faction(faction), faction, 7);
        p.decisions.cheapest_objective = 1;
        p.difficulty = legatum_core::AiDifficulty::Hard;
        assert_eq!(p.objective_build_value(&v.cities[0], walls), 0);
    }

    #[test]
    fn a_capture_that_completes_a_territory_or_city_count_is_worth_its_point() {
        // 141 of 143 tiles: any city's ring (seven tiles) covers the two still needed.
        let v = count_view(Objective::Territory { n: 143 }, 141, 143);
        assert_eq!(capture_value(&v, 0), 0);
        assert!(capture_value(&v, 1) > 0);
        // 100 tiles short: one city cannot close it.
        let far = count_view(Objective::Territory { n: 143 }, 40, 143);
        assert_eq!(capture_value(&far, 1), 0);
        // 11 of 12 cities: one capture is the twelfth; two short it is not.
        let one = count_view(Objective::Cities { n: 12 }, 11, 12);
        assert!(capture_value(&one, 1) > 0);
        assert_eq!(capture_value(&one, 0), 0);
        let two = count_view(Objective::Cities { n: 12 }, 10, 12);
        assert_eq!(capture_value(&two, 1), 0);
    }

    #[test]
    fn normal_two_step_choice_prefers_a_city_that_advances_the_next_era() {
        use legatum_core::objectives::Family;
        use legatum_core::view::{EraView, ObjectiveProgress, ObjectiveView, TeamStanding};
        let mut v = count_view(Objective::Population { n: 7 }, 6, 7);
        let specs = [
            (4, Objective::Population { n: 7 }, 6, 7),
            (5, Objective::Cities { n: 2 }, 1, 2),
            (6, Objective::Territory { n: 10 }, 0, 10),
        ];
        v.eras = specs
            .iter()
            .map(|(era, objective, _, target)| EraView {
                era: *era,
                name: format!("era {era}"),
                objectives: vec![ObjectiveView {
                    slot: 1,
                    family: Family::Expand,
                    text: String::new(),
                    target: *target,
                    objective: *objective,
                }],
                entered_by: vec![ME],
                scored_by: vec![],
            })
            .collect::<Vec<_>>()
            .into();
        v.objective_progress = specs
            .iter()
            .map(|(era, _, current, target)| ObjectiveProgress {
                era: *era,
                slot: 1,
                current: *current,
                target: *target,
                entered: true,
                era_scored: false,
                entered_turn: None,
                ready: None,
            })
            .collect();
        v.standings = vec![TeamStanding {
            team: v.team,
            members: vec![ME],
            vp: 5,
            target: 10,
            victory_hold: None,
        }]
        .into();
        let faction = v.faction_of(ME);
        let chosen = |difficulty, gene| {
            let mut p = Planner::new(&v, crate::Knobs::for_faction(faction), faction, 7);
            p.difficulty = difficulty;
            p.decisions.vp_two_step = gene;
            p.chosen_objective().map(|(o, _)| o.era)
        };
        use legatum_core::AiDifficulty::{Hard, Normal};
        assert_eq!(chosen(Normal, 0), Some(4));
        assert_eq!(chosen(Normal, 100), Some(5));
        // Hard's default cheapest-objective branch returns before the two-step ranking.
        assert_eq!(chosen(Hard, 0), Some(4));
        assert_eq!(chosen(Hard, 100), Some(4));
    }

    #[test]
    fn a_rivals_public_work_city_and_count_objective_have_denial_value() {
        use legatum_core::projects::GreatWorkView;
        let mut v = count_view(Objective::Cities { n: 2 }, 0, 2);
        let mut eras = v.eras.to_vec();
        eras[0].entered_by.push(ENEMY);
        v.eras = eras.into();
        v.great_works = vec![GreatWorkView {
            player: ENEMY,
            completed: None,
            building: Some(legatum_core::BuildingKind::GreatLibrary),
            city: Some(v.cities[1].id),
            progress: 50,
            cost: 100,
            production_per_turn: 10,
            turns: None,
            blocker: None,
        }];
        let faction = v.faction_of(ME);
        let mut p = Planner::new(&v, crate::Knobs::for_faction(faction), faction, 7);
        assert!(p.rival_objective_denial(&v.cities[1]) >= 200);
        p.decisions.vp_denial = 0;
        assert_eq!(p.rival_objective_denial(&v.cities[1]), 0);
    }

    #[test]
    fn with_objective_seek_zero_no_era_objective_steers_the_ai() {
        let v = count_view(Objective::Cities { n: 12 }, 11, 12);
        let faction = v.faction_of(ME);
        let planner = |seek: i32| {
            let mut p = Planner::new(&v, crate::Knobs::for_faction(faction), faction, 7);
            p.decisions.objective_seek = seek;
            p
        };
        let pursuing = planner(1);
        assert!(!pursuing.sought_objectives().is_empty());
        assert!(pursuing.objective_capture_value(&v.cities[1]) > 0);
        // The facts stay visible (the endgame planner reads them), but nothing is sought.
        let ignoring = planner(0);
        assert!(!ignoring.live_objectives().is_empty());
        assert!(ignoring.sought_objectives().is_empty());
        assert!(ignoring.chosen_objective().is_none());
        assert_eq!(ignoring.objective_capture_value(&v.cities[1]), 0);
        assert!(!ignoring.objective_close());
    }

    #[test]
    fn a_rival_with_heavy_decadence_is_strained_too() {
        let genes = DecisionParams::default();
        let mut v = ViewBuilder::new(6, 6).view;
        v.rule_experiments.decadence_slope = Some(45);
        let mut players = v.players.to_vec();
        let enemy = players.iter().position(|p| p.id == ENEMY).unwrap();
        // Decadence is weighted: 9% reads as an 18% burden, 10% as 20%.
        players[enemy].decadence_percent = 9;
        v.players = players.clone().into();
        assert!(!strained(&v, ENEMY, &genes));
        v.rule_experiments.decadence_slope = None;
        assert_eq!(burden(&v, ENEMY, &genes), 0);
        v.rule_experiments.decadence_slope = Some(45);
        players[enemy].decadence_percent = 10;
        v.players = players.clone().into();
        assert!(strained(&v, ENEMY, &genes));
        assert_eq!(burden(&v, ENEMY, &genes), 20);
        assert_eq!(
            strain_value(&v, ENEMY, &genes),
            20 * genes.strain_priority / 100
        );
    }
}

//! Research under the strict era gate.
//!
//! An empire may research only the unknown technologies of its current era (at most five),
//! with no prerequisites inside the era, and enters the next era the turn it finishes the last
//! one. So the choice is not *what* but *in what order*, and the AI orders the whole era at
//! once: it fills `Order::SetResearchQueue` with every researchable technology, best first, and
//! re-evaluates each turn because threats and rivals change. The queue also means no
//! turn is wasted between two technologies.
//!
//! The order, from most to least decisive:
//! 1. **Threat tier.** When the empire is threatened, has an aggressive rival within reach of
//!    a city (at war, or massing units) or a neighbour an era ahead of it, the era's unit and defence technologies (Walls' Masonry,
//!    Mathematics' siege, Feudalism's Trooper and Castle, Machinery, Gunpowder, Military
//!    Engineering's Star Fort and the later unit lineage) come first, whatever the strategy.
//! 2. **Value per Science.** A technology's value (knobs, faction priorities, the strategy
//!    bundle's weight, the goal and Great Work it unlocks, a war or peace plan's lean) divided
//!    by its price *now*. The price already carries the diffusion discount (-10% per empire
//!    that knows it), so cheaper technologies finish sooner.
//! 3. **Hysteresis.** The current technology stays first unless another beats its score by
//!    a quarter, so the queue does not churn.

use crate::scripted::Planner;
use legatum_core::data;
use legatum_core::{Order, ScienceConversion, TechId};

/// Value every technology has for opening the next era, so no technology is worth nothing.
pub(crate) const ERA_STEP_VALUE: i32 = 25;
/// Added to Irrigation, Bronze Working and Writing while the empire is young (fewer than six
/// technologies, no strategy): food, production and a Library come first.
const FOUNDATION_VALUE: i32 = 100;
/// Gene `swiss_playbook`: Masonry as a foundation of a Swiss empire whose cities' garrison is the
/// Alpine Redoubt, inside the military research weight (the designer queued it first in 8.9.24;
/// first outright, outside the weight, cost the AI Swiss 2.5 points of win share, 2026-11-05).
const SWISS_MASONRY_VALUE: i32 = FOUNDATION_VALUE;
/// The technologies that give units or defence, the ones a threatened empire wants first.
pub(crate) fn is_military_tech(tech: TechId) -> bool {
    use TechId::*;
    matches!(
        tech,
        Masonry
            | Mathematics
            | Feudalism
            | Machinery
            | Chivalry
            | Gunpowder
            | Metallurgy
            | MilitaryEngineering
            | Rifling
            | MilitaryScience
            | Cannonry
            | ReplaceableParts
            | Combustion
            | Steel
            | Flight
            | Robotics
            | StealthTechnology
    )
}

/// Whether the technology reveals a strategic resource (Iron Working, Gunpowder, Industrialization,
/// Combustion, Autonomous Systems).
fn reveals_resource(tech: TechId) -> bool {
    legatum_core::resources::Resource::STRATEGIC
        .into_iter()
        .any(|r| legatum_core::resources::stats(r).revealed_by == Some(tech))
}

/// The technologies that grow the economy or science, wanted first while at peace.
fn is_growth_tech(tech: TechId) -> bool {
    use TechId::*;
    matches!(
        tech,
        Irrigation
            | Currency
            | Writing
            | Philosophy
            | Education
            | CropRotation
            | Banking
            | PrintingPress
            | Industrialization
            | Electricity
            | FusionPower
            | RecursiveAi
    )
}

/// What this turn's research choice reads from the view, computed once for the whole era.
struct Signals {
    threatened: bool,
    /// Threatened, at war, bordered by an aggressive rival or behind a neighbour's era.
    urgent: bool,
    arms_race: bool,
    war_plan: bool,
    peace_plan: bool,
    stalled_growth: i32,
    slow_production: i32,
    broke: bool,
    slow_science: bool,
}

impl Planner<'_> {
    pub fn plan_research(&mut self) {
        if self.threat_research() {
            return;
        }
        let r = &self.view().research;
        if r.available.is_empty() {
            if r.known.len() == r.catalog.len() {
                self.plan_science_conversion();
            }
            return;
        }
        // An opening's own research pick stands, and an opening keeps its technology until
        // it is done (the opening policies pick one at a time).
        if self.opening.is_some()
            && (r.current.is_some()
                || self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::SetResearch { .. })))
        {
            return;
        }
        if r.current.is_none() && self.prepare_border_defense() {
            return;
        }
        let order = self.research_order();
        if order.is_empty() || order == self.view().research.queue {
            return;
        }
        self.orders.push(Order::SetResearchQueue { techs: order });
    }

    fn plan_science_conversion(&mut self) {
        let v = self.view();
        let science = v.economy.science_per_turn.total;
        let local: i32 = v
            .economy
            .science_per_turn
            .parts
            .iter()
            .filter(|(name, _)| v.cities.iter().any(|c| c.owner == v.me && c.name == *name))
            .map(|(_, amount)| amount / 2)
            .sum();
        let production = local * (100 + self.knobs.military.max(0) / 2) / 100;
        let food = local
            * (if self.research_signals().stalled_growth > 0 {
                130
            } else {
                70
            })
            / 100;
        let gold = (science / 2)
            * (if v.economy.gold_per_turn.total <= 0 {
                150
            } else {
                85
            })
            / 100;
        let choice = if production >= food && production >= gold {
            ScienceConversion::Production
        } else if food >= gold {
            ScienceConversion::Food
        } else {
            ScienceConversion::Gold
        };
        if choice != v.research.science_conversion {
            self.orders.push(Order::SetScienceConversion { to: choice });
        }
    }

    /// The full ordered plan for the current era: every researchable technology that can
    /// finish before an explicit turn limit, best first. Never holds an invalid pick: the
    /// candidates are exactly `research.available` (unknown, current era), without repeats.
    pub(crate) fn research_order(&self) -> Vec<TechId> {
        let r = &self.view().research;
        let signals = self.research_signals();
        struct Row {
            tech: TechId,
            military_first: bool,
            cost: i32,
            score: i32,
        }
        let mut rows: Vec<Row> = r
            .available
            .iter()
            .filter(|o| o.turns.is_none_or(|turns| turns <= self.turns_remaining()))
            .map(|o| {
                let military = is_military_tech(o.tech)
                    || o.tech == TechId::IronWorking && self.mongol_iron_push();
                let mut value = self.tech_value_with(o.tech, &signals);
                if signals.war_plan && military {
                    value += self.decisions.war_lean;
                }
                if signals.peace_plan && is_growth_tech(o.tech) {
                    value += self.decisions.peace_lean;
                }
                let cost = o.cost.total.max(1);
                let score = value.max(1) * 2000 / cost + self.stable_jitter(o.tech);
                Row {
                    tech: o.tech,
                    military_first: signals.urgent
                        && (military
                            || self.decisions.reveal_tech_first == 1 && reveals_resource(o.tech)),
                    cost: o.cost.total,
                    score,
                }
            })
            .collect();
        // Military first (when urgent), then by score;
        // equal scores keep the cheaper, then the earlier technology.
        rows.sort_by_key(|row| {
            (
                std::cmp::Reverse(row.military_first),
                std::cmp::Reverse(row.score),
                row.cost,
                row.tech,
            )
        });
        let mut order: Vec<TechId> = rows.iter().map(|row| row.tech).collect();
        // Rush: income before the expensive upgrades, in its fixed opening sequence.
        if self.strategy == Some(crate::Strategy::Rush) && !signals.urgent {
            let sequence = crate::Strategy::Rush.opening_sequence(r.era);
            let mut head: Vec<TechId> = sequence
                .iter()
                .copied()
                .filter(|t| order.contains(t))
                .collect();
            let rest: Vec<TechId> = order
                .iter()
                .copied()
                .filter(|t| !head.contains(t))
                .collect();
            head.extend(rest);
            order = head;
        }
        // Hysteresis: keep the current technology first unless the leader clearly beats it.
        if let Some(current) = r.current
            && let Some(at) = order.iter().position(|t| *t == current)
            && at > 0
        {
            let score_of = |tech: TechId| rows.iter().find(|row| row.tech == tech);
            if let (Some(cur), Some(top)) = (score_of(current), score_of(order[0]))
                && cur.military_first == top.military_first
                && top.score * 100 < cur.score * self.decisions.research_hysteresis
            {
                order.remove(at);
                order.insert(0, current);
            }
        }
        order
    }

    /// The tech's own value, scaled by the committed strategy's research weight.
    pub fn tech_value(&self, tech: TechId) -> i32 {
        self.tech_value_with(tech, &self.research_signals())
    }

    fn mongol_iron_push(&self) -> bool {
        self.mongol_horde()
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && crate::composition::is_mounted(u.kind))
                .count()
                >= 3
    }

    fn tech_value_with(&self, tech: TechId, s: &Signals) -> i32 {
        self.tech_base_value(tech, s) * self.strategy.map_or(100, |st| st.research_weight(tech))
            / 100
            * (100 + self.era_entry_bonus(tech))
            / 100
    }

    /// Fixed per technology and seat, so a re-evaluation never flips equal technologies.
    fn stable_jitter(&self, tech: TechId) -> i32 {
        let mut r = legatum_core::rng::Rng::derive(
            self.seed,
            &[
                legatum_core::rng::purpose::AI,
                self.ctx.me.0 as u64,
                crate::scripted::tag::RESEARCH,
                tech as u64,
            ],
        );
        r.range(0, 40) as i32
    }

    fn research_signals(&self) -> Signals {
        let v = self.view();
        let threatened = self.ctx.threat.iter().any(|t| *t > 0);
        let cities = self.ctx.own_cities.len() as i32;
        let border_exposed = self
            .ctx
            .own_cities
            .iter()
            .any(|c| !c.walls && self.assessed_border_threat(c.pos));
        // At war with (or bordered by) a rival that can reach us; a distant declared war
        // is the plan's lean, not an emergency.
        let rival_at_the_door = self
            .ctx
            .own_cities
            .iter()
            .any(|c| self.aggressive_neighbour(c));
        // A rival an era ahead within reach has better units than we can build: a threat.
        let mine = if self.decisions.era_transition_scoring == 0 {
            v.research.era
        } else {
            v.players
                .iter()
                .find(|p| p.id == v.me)
                .map_or(v.research.era + 1, |p| p.era)
        };
        let behind_neighbour = v.players.iter().any(|p| {
            p.alive
                && !v.same_team(v.me, p.id)
                && p.era > mine
                && crate::diplomacy::nearest_known(v, p.team).is_some_and(|d| d <= 12)
        });
        let arms_race =
            threatened || (self.conquest_pressure() > 0 && self.has_reachable_enemy_city());
        let plan = self
            .campaigns
            .plan
            .active
            .as_ref()
            .map(|p| p.kind.war_target().is_some());
        let stalled_growth = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| {
                c.detail
                    .as_ref()
                    .is_some_and(|d| d.yields.total.food <= 2 * c.population)
            })
            .count() as i32;
        let slow_production = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| {
                c.detail
                    .as_ref()
                    .is_some_and(|d| d.yields.total.production < 4)
            })
            .count() as i32;
        let urgent = threatened || border_exposed || rival_at_the_door || behind_neighbour;
        Signals {
            threatened,
            urgent,
            arms_race,
            war_plan: plan == Some(true),
            peace_plan: plan == Some(false) && !urgent,
            stalled_growth,
            slow_production,
            broke: v.economy.gold_per_turn.total <= 0,
            slow_science: cities > 0 && v.economy.science_per_turn.total <= cities * 2,
        }
    }

    fn tech_base_value(&self, tech: TechId, s: &Signals) -> i32 {
        let weight = match tech {
            TechId::Engineering
            | TechId::Feudalism
            | TechId::Machinery
            | TechId::Mathematics
            | TechId::Masonry => self.decisions.research_military_percent,
            TechId::Education | TechId::PrintingPress | TechId::Writing | TechId::Philosophy => {
                self.decisions.research_science_percent
            }
            TechId::CropRotation
            | TechId::IronWorking
            | TechId::Industrialization
            | TechId::Banking
            | TechId::Irrigation
            | TechId::BronzeWorking
            | TechId::Currency => self.decisions.research_economy_percent,
            _ => 100,
        };
        // Every technology opens the way to the next era, so none is worth nothing.
        self.tech_benefit_value(tech, s) * weight / 100 + ERA_STEP_VALUE
    }

    /// Gene `reveal_tech_bonus`: the technology that reveals a strategic resource is worth more,
    /// until the empire knows it, because every deposit decision waits on it.
    fn reveal_tech_value(&self, tech: TechId) -> i32 {
        if reveals_resource(tech) {
            self.decisions.reveal_tech_bonus
        } else {
            0
        }
    }

    /// What the technology itself does for this empire, from the knobs and the situation.
    /// Nothing when it gives this faction no effect and no item it can use.
    fn tech_benefit_value(&self, tech: TechId, s: &Signals) -> i32 {
        if !data::tech_has_player_benefit(
            self.view().rule_experiments,
            self.faction,
            self.view().era_of(self.view().me),
            tech,
        ) {
            return 0;
        }
        let k = self.knobs;
        let Signals {
            threatened,
            arms_race,
            stalled_growth,
            slow_production,
            broke,
            slow_science,
            ..
        } = *s;
        let d = &self.decisions;
        let faction_priority = match tech {
            TechId::BronzeWorking => d.bronze_bias,
            TechId::Irrigation => d.irrigation_bias,
            TechId::Currency => d.currency_bias,
            TechId::Banking => d.banking_bias,
            TechId::Feudalism => d.feudalism_bias,
            TechId::Chivalry => d.chivalry_bias,
            TechId::Writing | TechId::CodeOfLaws | TechId::Philosophy => d.scribe_bias,
            _ => 0,
        };
        let work_priority = if self.view().turn >= self.decisions.work_plan_turn as u32
            && self
                .work_plan
                .as_ref()
                .is_some_and(|p| p.research.contains(&tech) && p.turns <= self.turns_remaining())
        {
            350
        } else {
            0
        };
        // The opening foundation (food, production, a Library) before the rest of the Ancient
        // era, for an empire that follows no committed strategy.
        let foundation = if self.strategy.is_none()
            && self.view().research.known.len() < 6
            && matches!(
                tech,
                TechId::Irrigation | TechId::BronzeWorking | TechId::Writing
            ) {
            FOUNDATION_VALUE
        } else {
            0
        };
        let great_work = self.missing_great_work() > 0;
        faction_priority
            + foundation
            + self.goal_tech_priority(tech)
            + work_priority
            + self.reveal_tech_value(tech)
            + match tech {
                TechId::Irrigation => {
                    40 + k.economy / 2 + k.expansion / 4 + 60 * stalled_growth.min(3)
                }
                TechId::BronzeWorking => {
                    40 + k.economy / 2 + k.military / 4 + 60 * slow_production.min(3)
                }
                TechId::Masonry => {
                    // Walls make a city fire back, the counter to a neighbour's rush.
                    let wary = self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|c| self.aggressive_neighbour(c));
                    30 + k.military / 2
                        + k.economy / 3
                        + if threatened { 100 } else { 0 }
                        + if wary { 80 } else { 0 }
                        // Gene `swiss_playbook`: every founded city's garrison is a Redoubt.
                        + if self.redoubt_garrison() {
                            SWISS_MASONRY_VALUE
                        } else {
                            0
                        }
                }
                TechId::Currency => {
                    30 + k.economy * 3 / 4
                        + self.decisions.currency_tech_bonus
                        + if broke { 160 } else { 0 }
                }
                TechId::Writing => 30 + k.science * 3 / 4 + if slow_science { 100 } else { 0 },
                TechId::Mathematics => {
                    if self.has_reachable_enemy_city() {
                        100 + k.aggression * 2 + k.military
                    } else {
                        20
                    }
                }
                TechId::Engineering => 60 + k.military + if arms_race { 80 } else { 0 },
                TechId::IronWorking => {
                    // This technology is also the first chance to see Iron. Price that
                    // information when our cities and army are ready to use Iron supply.
                    let iron_users = self
                        .view()
                        .cities
                        .iter()
                        .filter(|city| city.owner == self.view().me && city.population >= 4)
                        .count() as i32
                        + self
                            .view()
                            .units
                            .iter()
                            .filter(|unit| {
                                unit.owner == self.view().me
                                    && unit.upgrade_need
                                        == Some(legatum_core::resources::Resource::Iron)
                            })
                            .count() as i32;
                    100 + k.economy
                        + k.military / 2
                        + self.decisions.industry_tech_bonus
                        + 50 * iron_users.min(4)
                        + if self.mongol_iron_push() { 250 } else { 0 }
                }
                TechId::CodeOfLaws => {
                    40 + k.expansion + k.economy / 4 + k.science / 4 + self.decisions.law_tech_bonus
                }
                TechId::Philosophy => {
                    // The Great Library (Relay Uplink) is worth 2 VP.
                    20 + k.science / 2
                        + self.ctx.own_cities.len() as i32 * 12
                        + if great_work { 40 + k.science / 2 } else { 0 }
                }
                TechId::Feudalism => {
                    20 + k.military / 2
                        + k.aggression / 2
                        + if threatened { 100 } else { 0 }
                        + if arms_race { 100 } else { 0 }
                }
                TechId::Machinery => {
                    100 + k.military
                        + k.economy / 2
                        + if arms_race { 120 } else { 0 }
                        + if great_work { 35 } else { 0 }
                }
                TechId::Chivalry => {
                    20 + k.military / 2 + if arms_race || threatened { 90 } else { 0 }
                }
                TechId::Education => 80 + k.science * 2,
                TechId::PrintingPress => 80 + k.science * 2 + k.economy / 2,
                TechId::CropRotation => {
                    70 + k.economy + 60 * stalled_growth.min(3) + self.decisions.crop_tech_bonus
                }
                TechId::Gunpowder => 60 + k.military + if arms_race { 120 } else { 0 },
                TechId::MilitaryEngineering => {
                    20 + k.military / 2 + if threatened { 120 } else { 0 }
                }
                TechId::Banking => {
                    // Commerce has its own paid Great Work finish (the Grand Bazaar).
                    80 + k.economy
                        + if broke { 120 } else { 0 }
                        + if great_work { 40 + k.economy / 2 } else { 0 }
                }
                TechId::Industrialization => {
                    100 + k.economy + k.military / 2 + self.decisions.industry_tech_bonus
                }
                TechId::Railways => 35 + k.aggression / 2 + k.economy / 2,
                TechId::MilitaryScience => {
                    45 + k.military
                        + k.aggression / 2
                        + self
                            .view()
                            .units
                            .iter()
                            .filter(|u| {
                                u.owner == self.view().me
                                    && u.military
                                    && legatum_core::data::unit_class(u.kind)
                                        != Some(legatum_core::UnitClass::Air)
                            })
                            .count() as i32
                            * 8
                }
                TechId::ReplaceableParts => {
                    45 + k.military / 2 + k.economy + self.ctx.own_cities.len() as i32 * 8
                }
                TechId::Combustion => 45 + k.military / 2 + k.economy / 2,
                // The Stock Exchange, the Gold line's third building.
                TechId::Electricity => 80 + k.economy + if broke { 120 } else { 0 },
                // The Research Lab.
                TechId::Electronics => 100 + k.economy + k.science,
                // The Supermarket, the Food line's third building.
                TechId::GreenRevolution => 70 + k.economy + 60 * stalled_growth.min(3),
                TechId::FusionPower => 100 + k.economy + self.ctx.own_cities.len() as i32 * 12,
                TechId::StealthTechnology => {
                    40 + k.military
                        + self
                            .view()
                            .aircraft
                            .iter()
                            .filter(|a| a.owner == self.view().me)
                            .count() as i32
                            * 55
                        + if arms_race { 100 } else { 0 }
                }
                TechId::RecursiveAi => {
                    80 + k.economy
                        + self.view().tiles.iter().filter(|t| !t.explored).count() as i32 / 15
                        + self
                            .ctx
                            .own_cities
                            .iter()
                            .filter_map(|c| c.detail.as_ref())
                            .map(|d| d.yields.total.production / 5)
                            .sum::<i32>()
                }
                // Unit lineages that arrive with the units update: wanted for the army.
                TechId::Metallurgy
                | TechId::Rifling
                | TechId::Cannonry
                | TechId::Steel
                | TechId::Flight
                | TechId::MassMobilisation
                | TechId::Rocketry
                | TechId::ArmouredWarfare
                | TechId::AirPower
                | TechId::MechanisedWarfare
                | TechId::CompositeArmour
                | TechId::GuidedMissiles
                | TechId::JetEngines => 20 + k.military / 2 + if arms_race { 60 } else { 0 },
                TechId::AutonomousSystems | TechId::Robotics => 20 + k.military / 4 + k.economy / 4,
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid, off};
    use crate::{Brain, Knobs, ScriptedBrain};
    use legatum_core::eras;
    use legatum_core::{Faction, PlayerView, UnitKind};

    fn planner(view: &PlayerView) -> Planner<'_> {
        let mut p = Planner::new(
            view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.decisions.era_transition_scoring = 1;
        p
    }

    fn queue_of(p: &Planner<'_>) -> Vec<TechId> {
        p.orders
            .iter()
            .find_map(|o| match o {
                Order::SetResearchQueue { techs } => Some(techs.clone()),
                _ => None,
            })
            .expect("a research queue was ordered")
    }

    fn view_in_era(era: u8) -> PlayerView {
        let known: Vec<TechId> = TechId::ALL
            .into_iter()
            .filter(|t| eras::era_of_tech(*t) < era)
            .collect();
        let mut view = ViewBuilder::new(20, 20)
            .known_techs(&known)
            .city(1, ME, off(5, 5), 3)
            .build();
        view.turn_limit = None;
        view
    }

    #[test]
    fn every_era_queues_all_of_its_technologies_once_and_only_those() {
        for era in 0..eras::ERA_COUNT as u8 {
            let view = view_in_era(era);
            let mut p = planner(&view);
            p.plan_research();
            let queue = queue_of(&p);
            let mut expected: Vec<TechId> = eras::era_techs(era).collect();
            let mut sorted = queue.clone();
            sorted.sort();
            expected.sort();
            assert_eq!(sorted, expected, "era {era}");
            assert_valid(&view, &p.orders);
        }
    }

    #[test]
    fn a_threatened_empire_researches_defence_and_units_first() {
        // Medieval: Feudalism and Machinery before the economy technologies.
        let mut view = view_in_era(2);
        view = ViewBuilder { view }
            .city(2, ENEMY, off(9, 5), 3)
            .unit(20, ENEMY, UnitKind::Soldier, off(7, 5))
            .unit(21, ENEMY, UnitKind::Soldier, off(7, 6))
            .build();
        view.turn_limit = None;
        let mut p = planner(&view);
        p.plan_research();
        let queue = queue_of(&p);
        let military: Vec<_> = queue.iter().filter(|t| is_military_tech(**t)).collect();
        assert!(!military.is_empty());
        for (i, t) in queue.iter().enumerate() {
            if is_military_tech(*t) {
                assert!(i < military.len(), "{queue:?}: {t:?} is not in the front");
            }
        }
    }

    #[test]
    fn a_peaceful_isolated_empire_does_not_front_load_the_military() {
        let view = view_in_era(2);
        let mut p = planner(&view);
        p.plan_research();
        let queue = queue_of(&p);
        assert!(
            !(is_military_tech(queue[0]) && is_military_tech(queue[1])),
            "{queue:?}"
        );
    }

    #[test]
    fn iron_working_is_more_valuable_when_an_upgrade_needs_iron() {
        let mut view = ViewBuilder {
            view: view_in_era(1),
        }
        .unit(20, ME, UnitKind::Soldier, off(6, 5))
        .build();
        let without = planner(&view).tech_value(TechId::IronWorking);
        view.units
            .iter_mut()
            .find(|unit| unit.id == legatum_core::UnitId(20))
            .unwrap()
            .upgrade_need = Some(legatum_core::resources::Resource::Iron);
        let with = planner(&view).tech_value(TechId::IronWorking);
        assert!(with > without, "{without} vs {with}");
    }

    #[test]
    fn a_neighbour_an_era_ahead_is_a_threat() {
        let view = view_in_era(1);
        let mut v = ViewBuilder { view }.city(2, ENEMY, off(12, 5), 4).build();
        v.turn_limit = None;
        let quiet = {
            let mut p = planner(&v);
            p.plan_research();
            queue_of(&p)
        };
        let mut ahead = ViewBuilder { view: v }.era_of(2, 2).build();
        ahead.turn_limit = None;
        let mut p = planner(&ahead);
        p.plan_research();
        let queue = queue_of(&p);
        // A threatened empire's first tier is the unit and defence technologies plus the one
        // that reveals the era's resource (gene `reveal_tech_first`): units need the deposits.
        let first_tier = |t: &TechId| is_military_tech(*t) || reveals_resource(*t);
        let military_first = queue.iter().take_while(|t| first_tier(t)).count();
        let quiet_first = quiet.iter().take_while(|t| first_tier(t)).count();
        assert!(military_first >= quiet_first, "{quiet:?} vs {queue:?}");
        assert!(first_tier(&queue[0]), "{queue:?}");
    }

    #[test]
    fn the_current_technology_keeps_its_place_without_a_clear_better_choice() {
        let view = view_in_era(0);
        let mut p = planner(&view);
        p.plan_research();
        let order = queue_of(&p);
        let mut running = view;
        running.research.current = Some(order[1]);
        running.research.queue = order.clone();
        let mut p = planner(&running);
        p.plan_research();
        // Either unchanged (no order) or still led by the running technology.
        if !p.orders.is_empty() {
            assert_eq!(queue_of(&p)[0], order[1]);
        }
    }

    #[test]
    fn research_with_nothing_fitting_the_deadline_orders_nothing() {
        let mut view = view_in_era(0);
        view.turn_limit = Some(view.turn);
        let mut p = planner(&view);
        p.plan_research();
        assert!(p.orders.is_empty());
    }

    #[test]
    fn an_unchanged_queue_is_not_reordered_every_turn() {
        let view = view_in_era(1);
        let mut p = planner(&view);
        p.plan_research();
        let order = queue_of(&p);
        let mut queued = view;
        queued.research.current = Some(order[0]);
        queued.research.queue = order;
        let mut p = planner(&queued);
        p.plan_research();
        assert!(p.orders.is_empty(), "{:?}", p.orders);
    }

    #[test]
    fn equal_public_era_does_not_trigger_an_arms_emergency() {
        for era in 0..legatum_core::eras::LAST_ERA {
            let mut v = ViewBuilder {
                view: view_in_era(era),
            }
            .city(2, ENEMY, off(12, 5), 4)
            .era_of(0, era + 1)
            .era_of(2, era + 1)
            .build();
            v.diplomacy.legacy = false;
            assert!(!planner(&v).research_signals().urgent, "equal era {era}");
            v.players[2].era += 1;
            assert!(planner(&v).research_signals().urgent, "ahead era {era}");
        }
    }

    #[test]
    fn a_brain_never_orders_an_invalid_research_queue() {
        for era in 0..eras::ERA_COUNT as u8 {
            let view = view_in_era(era);
            for faction in Faction::ALL {
                let mut brain = ScriptedBrain::new(faction, 7, ME);
                let orders = brain.orders(&view);
                assert_valid(&view, &orders);
            }
        }
    }
}

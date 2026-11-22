//! View-derived strategic adjustments shared by scripted and opening planners.
use crate::opponents::StrategySwitch;
use crate::scripted::{Planner, ScriptedBrain};
use legatum_core::view::CityView;
use legatum_core::{Faction, Hex, PlayerView, UnitKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Adaptation {
    #[default]
    Balanced,
    Expand,
    DefendedExpand,
}

impl Adaptation {
    fn name(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Expand => "expand",
            Self::DefendedExpand => "defended_expansion",
        }
    }
}

pub(crate) fn early(v: &PlayerView) -> bool {
    v.turn < 120
}

impl ScriptedBrain {
    pub(crate) fn assess_opponents(&mut self, v: &PlayerView) {
        self.opponents.threats.runaway_read = self.decisions.runaway_read;
        self.opponents.threats.hollow_read = self.decisions.hollow_read;
        self.opponents.threats.army_threat_weight = self.decisions.army_threat_weight;
        self.opponents.threats.threat_horizon_turns = self.decisions.threat_horizon_turns;
        self.opponents
            .update_with_risk(v, self.decisions.assessment_interval > 0);
        let threat = self.opponents.threatening_neighbor();
        let known_targets = self.opponents.rivals.values().any(|r| !r.cities.is_empty());
        let no_targets = known_targets && !self.opponents.has_viable_target(early(v));
        let (next, rival, reason) = if no_targets
            && (self.knobs.aggression >= 60 || self.strategy == Some(crate::Strategy::Rush))
        {
            // Walls and garrisons still respond locally to danger while the offensive
            // production budget pivots; a threatening Celts must not keep a rush alive.
            (
                Adaptation::Expand,
                self.opponents.rivals.keys().next().copied(),
                "no_viable_targets",
            )
        } else if let Some(r) = threat {
            (
                Adaptation::DefendedExpand,
                Some(r.id),
                "aggressive_neighbor",
            )
        } else {
            (
                Adaptation::Balanced,
                None,
                "targets_or_border_conditions_changed",
            )
        };
        if next != self.adaptation {
            self.strategy_switches.push(StrategySwitch {
                turn: v.turn,
                player: v.me,
                rival,
                from: self.adaptation.name().into(),
                to: next.name().into(),
                reason: reason.into(),
            });
            self.adaptation = next;
        }
    }

    pub(crate) fn adapted_knobs(&self) -> crate::Knobs {
        let mut k = self.knobs;
        match self.adaptation {
            Adaptation::Balanced => {}
            Adaptation::Expand => {
                k.expansion = k.expansion.max(70);
                k.aggression = k.aggression.min(25);
                k.economy = k.economy.max(60);
            }
            Adaptation::DefendedExpand => {
                k.military = k.military.max(65);
            }
        }
        k
    }
}

impl Planner<'_> {
    /// Restrict anticipatory spending to exposed cities, preserving interior expansion.
    /// Walls and a ranged garrison answer observed aggression (an army massing or a war
    /// declared). Hard also reads the faction (user, 2026-10-06, Civ 6's "Babylon next door
    /// means walls and archers now"): a rush-capable neighbour by its public rules (more
    /// moves, cheaper strength, or infantry that ignore wounds) counts before it attacks.
    pub(crate) fn assessed_border_threat(&self, pos: Hex) -> bool {
        let prior = self.hard_defense() && self.decisions.faction_threat_prior > 0;
        if self.rush_walls_threat(pos) {
            return true;
        }
        // Gene `prior_needs_army`: the faction prior alone (no army seen massing, no war
        // declared) counts only when the rival's public army is at least `prior_needs_army`
        // percent of ours; a neighbour we already out-muscle cannot rush us (playtest 8.9.2:
        // AI Sparta, three units to the Mongols' one, spent turns 15-25 on a Ranger and Walls
        // against them instead of its own rush).
        let v = self.view();
        let army = |p: legatum_core::PlayerId| v.army_of(p);
        let ours = army(v.me);
        let pct = self.decisions.prior_needs_army;
        let able =
            |r: &crate::opponents::OpponentAssessment| pct == 0 || army(r.id) * 100 >= ours * pct;
        self.opponents.is_some_and(|model| {
            model.rivals.values().any(|r| {
                (r.observed_aggression() || prior && r.aggressive_neighbor() && able(r))
                    && (r.cities.iter().any(|c| c.pos.distance(pos) <= 10)
                        || self
                            .view()
                            .units
                            .iter()
                            .any(|u| u.owner == r.id && u.military && u.pos.distance(pos) <= 6))
            })
        })
    }

    /// Hard with `rush_walls`, before any contact: a rush-capable rival (public faction rules)
    /// that we haven't yet placed beyond striking range makes the capital a border city, so
    /// the Walls tech, a shooter and Walls come before the second Settler (playtest proxy:
    /// a Mongols conqueror declared on turns 12-35 and Hard's first Walls came on 34-59).
    /// Once its cities are known, only our cities within 12 of them stay on the list.
    pub(crate) fn rush_walls_threat(&self, pos: Hex) -> bool {
        if !self.hard_defense() {
            return false;
        }
        let v = self.view();
        // Research item 2: read the rush from public numbers before the border is crossed; and
        // at any time a rival army `army_ratio_walls` percent of ours (playtest 8.4: walls
        // and a shooter when a rival's public army is twice ours).
        let early = v.turn < self.decisions.rush_read_turn as u32;
        let pct = match (
            early,
            self.decisions.army_growth_trigger,
            self.decisions.army_ratio_walls,
        ) {
            (true, a, b) if a > 0 && b > 0 => a.min(b),
            (true, a, _) if a > 0 => a,
            (_, _, b) => b,
        };
        if pct > 0 {
            let army = |p: legatum_core::PlayerId| v.army_of(p);
            let ours = army(v.me);
            if v.players.iter().any(|p| {
                p.alive
                    && !v.same_team(v.me, p.id)
                    && army(p.id) * 100 > ours.max(1) * pct
                    && v.cities.iter().any(|c| {
                        c.owner == p.id && c.pos.distance(pos) <= self.decisions.rival_radius
                    })
            }) {
                return true;
            }
        }
        if self.decisions.rush_walls == 0 {
            return false;
        }
        v.players.iter().any(|p| {
            p.alive && !v.same_team(v.me, p.id) && crate::opponents::rush_capable(p.faction) && {
                let known: Vec<_> = v.cities.iter().filter(|c| c.owner == p.id).collect();
                if known.is_empty() {
                    // Unplaced: only the capital, and only in a small lobby where every
                    // rival is a neighbour.
                    v.players.len() <= 4 && self.ctx.capital().is_some_and(|cap| cap.pos == pos)
                } else {
                    known
                        .iter()
                        .any(|c| c.pos.distance(pos) <= self.decisions.rival_radius)
                }
            }
        })
    }

    pub(crate) fn prepare_border_defense(&mut self) -> bool {
        if self.faction == Faction::Mongols
            || self.view().research.current.is_some()
            || !self
                .ctx
                .own_cities
                .iter()
                .any(|c| !c.walls && self.assessed_border_threat(c.pos))
        {
            return false;
        }
        // Follow the actual unlock prerequisite graph; do not assume Walls are turn-one legal.
        let walls = legatum_core::data::building_stats(legatum_core::BuildingKind::Walls);
        let Some(tech) = walls.requires else {
            return false;
        };
        if self.view().research.known.contains(&tech) {
            return false;
        }
        let mut needed = std::collections::BTreeSet::new();
        crate::investment::prerequisites(tech, &self.view().research.known, &mut needed);
        needed.insert(tech);
        if let Some(opt) = self
            .view()
            .research
            .available
            .iter()
            .find(|o| needed.contains(&o.tech))
        {
            self.orders
                .push(legatum_core::Order::SetResearch { tech: opt.tech });
            return true;
        }
        false
    }

    /// Strategic caution is a preference, not a ban on finishing a won siege.
    pub(crate) fn assessed_campaign_allowed(&self, city: &CityView) -> bool {
        if self
            .opponents
            .and_then(|m| m.rivals.get(&city.owner))
            .is_none()
        {
            return true;
        }
        if city.visible && city.hp <= city.max_hp / 3 {
            return true;
        }
        if early(self.view())
            && city.walls
            && !self.walls_are_a_price()
            && self.campaign_forecast(city, None).is_none()
        {
            return false;
        }
        true
    }

    pub(crate) fn assessed_target_value(&self, city: &CityView) -> i32 {
        let Some(r) = self.opponents.and_then(|m| m.rivals.get(&city.owner)) else {
            return 0;
        };
        let Some(c) = r.cities.iter().find(|c| c.id == city.id) else {
            return 0;
        };
        let soft = i32::from(
            city.visible
                && !city.walls
                && c.garrison_strength.is_some_and(|s| {
                    s <= legatum_core::data::unit_stats(UnitKind::Soldier).strength
                }),
        );
        50 * soft
            + 15 * c.india_links as i32
            + 40 * i32::from(self.recently_captured_by_threat(city))
            - 30 * i32::from(city.walls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid, off};
    use crate::openings::{Opening, OpeningBrain, OpeningPolicy, Phase};
    use crate::{Brain, ScriptedBrain};
    use legatum_core::{AiDifficulty, BuildingKind, Faction, Order, ProductionItem, TechId};

    fn neighbor(faction: Faction, techs: &[TechId]) -> PlayerView {
        let mut v = ViewBuilder::new(24, 16)
            .faction(Faction::Athens)
            .known_techs(techs)
            .city(1, ME, off(5, 6), 3)
            .city(2, ENEMY, off(11, 6), 3)
            .unit(1, ME, UnitKind::Soldier, off(5, 6))
            .build();
        v.players
            .iter_mut()
            .find(|p| p.id == ENEMY)
            .unwrap()
            .faction = faction;
        v.economy.gold = 0;
        v
    }

    #[test]
    fn greedy_expansion_walls_against_observed_aggression_not_a_faction_prior() {
        // A rush-capable neighbour at peace earns more units (DefendedExpand), not Walls.
        let mut brain = OpeningBrain::new(
            ScriptedBrain::with_difficulty(Faction::Athens, 17, ME, AiDifficulty::Hard),
            OpeningPolicy::builtin(Opening::GreedyExpand),
        );
        let v = neighbor(Faction::Mongols, &[]);
        let orders = brain.orders(&v);
        assert_valid(&v, &orders);
        assert!(!orders.contains(&Order::SetResearch {
            tech: TechId::Masonry
        }));
        assert!(brain.strategy_switches().iter().any(|s| s.reason
            == "aggressive_neighbor_defended_expansion"
            && s.rival == Some(ENEMY)));
        // Its army massing at our border is observed aggression: Walls' tech, a garrisoned
        // shooter, then Walls.
        let mut brain = OpeningBrain::new(
            ScriptedBrain::with_difficulty(Faction::Athens, 17, ME, AiDifficulty::Hard),
            OpeningPolicy::builtin(Opening::GreedyExpand),
        );
        let massing = |techs: &[TechId], shooter: bool| {
            let mut b = ViewBuilder {
                view: neighbor(Faction::Mongols, techs),
            }
            .unit(10, ENEMY, UnitKind::Soldier, off(8, 6))
            .unit(11, ENEMY, UnitKind::Soldier, off(8, 7));
            if shooter {
                b = b.unit(12, ME, UnitKind::Ranger, off(5, 6));
            }
            b.build()
        };
        let v = massing(&[], false);
        let orders = brain.orders(&v);
        assert_valid(&v, &orders);
        assert!(orders.contains(&Order::SetResearch {
            tech: TechId::Masonry
        }));
        let mut fortified = massing(&[TechId::Masonry], true);
        fortified.turn += 5;
        let orders = brain.orders(&fortified);
        assert_valid(&fortified, &orders);
        assert!(orders.iter().any(|o| matches!(
            o,
            Order::SetProduction {
                item: ProductionItem::Building {
                    building: BuildingKind::Walls
                },
                ..
            }
        )));
        let mut calm = OpeningBrain::new(
            ScriptedBrain::new(Faction::Athens, 17, ME),
            OpeningPolicy::builtin(Opening::GreedyExpand),
        );
        let quiet = neighbor(Faction::Athens, &[TechId::Masonry]);
        assert!(!calm.orders(&quiet).iter().any(|o| matches!(
            o,
            Order::SetProduction {
                item: ProductionItem::Building {
                    building: BuildingKind::Walls
                },
                ..
            }
        )));
    }

    #[test]
    fn mongols_rush_pivots_on_a_walled_neighbor() {
        let mut v = neighbor(Faction::Athens, &[]);
        v.cities[1].walls = true;
        let mut brain = OpeningBrain::new(
            ScriptedBrain::new(Faction::Mongols, 17, ME),
            OpeningPolicy::builtin(Opening::Rush),
        );
        let orders = brain.orders(&v);
        assert_valid(&v, &orders);
        assert_eq!(brain.phase, Phase::Expand);
        assert!(
            brain
                .strategy_switches()
                .iter()
                .any(|s| s.reason == "no_viable_targets")
        );
        let count = brain.strategy_switches().len();
        assert_eq!(orders, brain.orders(&v));
        assert_eq!(count, brain.strategy_switches().len());
    }

    #[test]
    fn scripted_hard_adapts_and_soft_targets_remain_eligible() {
        let mut v = neighbor(Faction::Athens, &[]);
        v.cities[1].walls = true;
        let mut brain =
            ScriptedBrain::with_strategy(Faction::Mongols, 17, ME, crate::Strategy::Rush);
        brain.difficulty = AiDifficulty::Hard;
        brain.orders(&v);
        assert_eq!(brain.adaptation, Adaptation::Expand);
        v.turn += 5;
        v.cities[1].walls = false;
        brain.orders(&v);
        assert_eq!(brain.adaptation, Adaptation::Balanced);
        assert_eq!(brain.strategy_switches().len(), 2);
    }
}

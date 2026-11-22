//! Faction-goal decisions for Germany's Ruhr Foundries and Byzantium's Heir.
//! Both are valued against their VP and opportunity cost; rivals price denial from public facts.

use crate::scripted::Planner;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{
    BuildingKind, Faction, Hex, Order, ProductionItem, UnitKind, VpSource, data, goals,
};

const LAST_FACTORY_VALUE_PER_VP: i32 = 1500;
const HEIR_SWING_VALUE_PER_VP: i32 = 100;

impl Planner<'_> {
    fn city_has_factory(&self, c: &CityView) -> bool {
        c.detail.as_ref().is_some_and(|d| data::has_building(&d.buildings, BuildingKind::Forge))
            || self.orders.iter().any(|o| matches!(o,
                Order::BuyItem { city, item: ProductionItem::Building { building: BuildingKind::Forge } }
                if *city == c.id))
    }

    pub(crate) fn factories_missing(&self) -> usize {
        self.ctx
            .own_cities
            .iter()
            .filter(|c| !self.city_has_factory(c))
            .count()
    }

    fn foundries_held_or_bought(&self) -> i32 {
        self.ctx.own_cities.len() as i32 - self.factories_missing() as i32
    }

    fn foundries_to_goal(&self) -> i32 {
        (data::GERMANY_GOAL_CITIES - self.foundries_held_or_bought()).max(0)
    }

    pub(crate) fn expansion_room(&self) -> bool {
        self.ctx.site_score.iter().enumerate().any(|(i, score)| {
            let h = self.ctx.hex_at(i);
            score.is_some_and(|s| s >= self.decisions.settler_min_site_score)
                && self.ctx.own_cities.iter().any(|home| {
                    home.pos.distance(h) <= self.decisions.settler_max_site_distance
                        && self.ctx.reachable(home.pos, h)
                })
                && !self.ctx.in_danger(h)
        })
    }

    /// Germany's Five Ruhr Foundries goal scores (an ablation can switch it off).
    pub(crate) fn germany_goal_on(&self) -> bool {
        !goals::rule_switched_off(self.view().rule_experiments, goals::GoalRule::Factories)
    }

    fn forge_planning(&self) -> bool {
        self.faction == Faction::Germany
            && self.germany_goal_on()
            && self
                .view()
                .research
                .known
                .contains(&legatum_core::TechId::IronWorking)
    }

    pub(crate) fn forge_open(&self) -> bool {
        self.forge_planning()
            && self
                .view()
                .research
                .known
                .contains(&legatum_core::TechId::IronWorking)
    }

    /// The goal value of expansion before five Foundry cities, as a negative cost.
    /// It grows with the completed share; ordinary city value remains in the caller.
    pub(crate) fn factory_goal_expansion_cost(&self) -> i32 {
        if !self.forge_open() || self.ctx.own_cities.len() as i32 >= data::GERMANY_GOAL_CITIES {
            return 0;
        }
        -LAST_FACTORY_VALUE_PER_VP
            * goals::spec_in(self.view().rule_experiments, self.faction).vp as i32
            * (1 + self.foundries_held_or_bought())
            / data::GERMANY_GOAL_CITIES
    }

    /// Each needed Foundry earns a share of the held 2 VP. A prerequisite carries part of
    /// that share because it makes the final Foundry possible; ordinary yields are priced
    /// separately. The Foundry gets its whole share, including in the last city.
    pub(crate) fn factory_goal_stage_value(
        &self,
        c: &CityView,
        building: BuildingKind,
        turns: i32,
    ) -> i32 {
        if !self.forge_planning()
            || self.foundries_to_goal() == 0
            || self.city_has_factory(c)
            || turns < 0
            || turns + 10 > self.turns_remaining()
        {
            return 0;
        }
        let d = c.detail.as_ref().expect("own city detail");
        let percent = match building {
            BuildingKind::Workshop if !data::has_building(&d.buildings, BuildingKind::Workshop) => {
                35
            }
            BuildingKind::Forge if self.forge_open() => 100,
            _ => 0,
        };
        let share = if self.forge_open() {
            self.factory_goal_value(c, turns)
        } else {
            LAST_FACTORY_VALUE_PER_VP
                * goals::spec_in(self.view().rule_experiments, self.faction).vp as i32
                / self.foundries_to_goal()
        };
        share * percent / 100
    }

    pub(crate) fn factory_goal_value(&self, c: &CityView, turns: i32) -> i32 {
        if !self.forge_open()
            || self.city_has_factory(c)
            || self.foundries_to_goal() == 0
            || turns < 0
            || turns + 10 > self.turns_remaining()
        {
            return 0;
        }
        let vp = goals::spec_in(self.view().rule_experiments, self.faction).vp as i32;
        let missing = self.foundries_to_goal();
        LAST_FACTORY_VALUE_PER_VP * vp / missing
    }

    /// One spare Iron has a goal payoff when the remaining Foundries need it.
    /// The same share is the opportunity cost of spending that slot on another item.
    pub(crate) fn factory_goal_supply_value(
        &self,
        resource: legatum_core::resources::Resource,
    ) -> i32 {
        use legatum_core::resources::Resource::Iron;
        if !self.forge_planning() || self.foundries_to_goal() == 0 || resource != Iron {
            return 0;
        }
        let need = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| !self.city_has_factory(c))
            .count() as i32;
        let need = need.min(self.foundries_to_goal());
        let spare = self
            .view()
            .supply
            .iter()
            .find(|s| s.resource == resource)
            .map_or(0, |s| s.spare);
        if need <= spare {
            0
        } else {
            LAST_FACTORY_VALUE_PER_VP
                * goals::spec_in(self.view().rule_experiments, self.faction).vp as i32
                / self.foundries_to_goal()
        }
    }

    pub(crate) fn factory_completes_goal(&self, city: &CityView, building: BuildingKind) -> bool {
        self.faction == Faction::Germany
            && self.germany_goal_on()
            && building == BuildingKind::Forge
            && self.foundries_held_or_bought() == data::GERMANY_GOAL_CITIES - 1
            && self
                .ctx
                .own_cities
                .iter()
                .any(|c| c.id == city.id && !self.city_has_factory(c))
    }

    /// A held goal is certain to fall if Germany has exactly five cities. With more
    /// cities, price the smaller chance that the target is one of exactly five Foundries.
    pub(crate) fn forge_denial_value(&self, city: &CityView) -> i32 {
        if city
            .detail
            .as_ref()
            .is_some_and(|d| !data::has_building(&d.buildings, BuildingKind::Forge))
        {
            return 0;
        }
        let holds = self.view().players.iter().any(|p| {
            p.id == city.owner
                && p.faction == Faction::Germany
                && holds_goal(p.vp_sources.as_slice(), city.owner)
        });
        if !holds {
            return 0;
        }
        let cities = self
            .view()
            .players
            .iter()
            .find(|p| p.id == city.owner)
            .unwrap()
            .cities as i32;
        let target = data::GERMANY_GOAL_CITIES;
        50 * goals::spec_in(self.view().rule_experiments, Faction::Germany).vp as i32
            * target
            * target
            / cities.max(target).pow(2)
    }

    // ---------- Byzantium: the Heir ----------

    /// Every Heir we hold: ours, or a rival's we captured.
    pub(crate) fn own_heirs(&self) -> impl Iterator<Item = &UnitView> {
        self.ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.kind == UnitKind::HeirToTheThrone)
    }

    /// The Heir is worth training here: the Byzantium has the cities and the time, and the city
    /// is not under threat. His price is steep, so he waits for an economy that can carry it.
    pub(crate) fn heir_value(
        &self,
        c: &CityView,
        turns: i32,
        threatened: bool,
        undefended: bool,
    ) -> i32 {
        if self.faction != Faction::Byzantium
            || threatened
            || undefended
            || self.ctx.own_cities.len() < 3
            || turns < 0
            || turns + 20 > self.turns_remaining()
        {
            return 0;
        }
        // Not while a garrison is missing in his future home, nor with a better Heir around.
        let garrisoned = self
            .ctx
            .own_units
            .iter()
            .any(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 1);
        if !garrisoned {
            return 0;
        }
        goals::spec(Faction::Byzantium).vp as i32 * 1500
    }

    /// An own Heir in `c` with no garrison beside him.
    pub(crate) fn heir_unguarded(&self, c: &CityView) -> bool {
        self.own_heirs().any(|h| h.pos == c.pos)
            && !self
                .ctx
                .own_units
                .iter()
                .any(|u| u.military && u.kind != UnitKind::Scout && u.pos == c.pos)
    }

    /// Where the Heir shelters: the safest own city in reach, preferring walls and a garrison.
    fn heir_refuge(&self, heir: &UnitView) -> Option<(Hex, bool)> {
        self.ctx
            .own_cities
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.detail.as_ref().is_some_and(|d| d.resistance <= 0)
                    && c.pos.distance(heir.pos) <= 8
                    && self.ctx.reachable(heir.pos, c.pos)
            })
            .max_by_key(|(i, c)| {
                let garrison = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| {
                        u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 1
                    })
                    .count() as i32;
                (
                    -(self.ctx.threat[*i]).min(1000) - self.heir_journey_risk(heir, c.pos),
                    i32::from(c.walls),
                    garrison.min(3),
                    i32::from(c.pos == heir.pos),
                    -c.pos.distance(heir.pos),
                    c.id,
                )
            })
            .map(|(_, c)| (c.pos, c.pos == heir.pos))
    }

    fn heir_journey_risk(&self, heir: &UnitView, to: Hex) -> i32 {
        if to == heir.pos || self.decisions.heir_route_risk_weight == 0 {
            return 0;
        }
        let Some(path) = self
            .view()
            .movement_path(heir.owner, heir.kind, heir.pos, to)
        else {
            return 1000;
        };
        let mut exposed = 0;
        for h in path.into_iter().filter(|h| *h != heir.pos && *h != to) {
            let danger = self
                .view()
                .units
                .iter()
                .filter(|u| u.military && !self.view().same_team(u.owner, heir.owner))
                .filter(|u| {
                    u.pos.distance(h) <= u.max_moves.max(1)
                        || (u.ranged > 0 && u.pos.distance(h) <= u.range)
                })
                .map(|u| {
                    if self.view().at_war(heir.owner, u.owner) {
                        200
                    } else {
                        100
                    }
                })
                .max()
                .unwrap_or(0);
            exposed += danger;
        }
        exposed.min(1000) * self.decisions.heir_route_risk_weight / 100
    }

    /// Keep every Heir we hold (ours, or a rival's we captured) inside the safest city: he never
    /// wanders, and moves only to shelter.
    pub(crate) fn protect_heirs(&mut self) {
        let heirs: Vec<UnitView> = self.own_heirs().cloned().collect();
        for heir in heirs {
            let Some((to, here)) = self.heir_refuge(&heir) else {
                continue;
            };
            if here || self.has_acted(heir.id) {
                continue;
            }
            self.unit_order(heir.id, Order::MoveUnit { unit: heir.id, to });
        }
    }

    /// VP-swing value of capturing an Heir: the captor gains the goal's VP and its holder loses
    /// them. Zero for an Heir we already hold.
    pub(crate) fn heir_capture_value(&self, heir: &UnitView) -> i32 {
        if heir.kind != UnitKind::HeirToTheThrone
            || self.view().team_of(heir.owner) == self.view().team
        {
            return 0;
        }
        // Barbarian-held: nobody scores, but a cleared camp frees him to us, and he is ours to
        // protect then. Capturing him at once is worth the VP he will give.
        goals::spec(Faction::Byzantium).vp as i32 * HEIR_SWING_VALUE_PER_VP
    }

    /// Value of the Heir a city shelters: taking the city takes him, and his points.
    pub(crate) fn heir_in_city_value(&self, city: &CityView) -> i32 {
        self.view()
            .units
            .iter()
            .filter(|u| {
                u.kind == UnitKind::HeirToTheThrone && u.pos == city.pos && u.owner == city.owner
            })
            .map(|u| self.heir_capture_value(u))
            .max()
            .unwrap_or(0)
    }

    /// Denial value of making war on `team`: it scores a held faction goal (the Heir or the
    /// Yam Relay Station) that capture would take. Public: `vp_sources` carries `FactionGoal { of }`
    /// for the holder.
    pub(crate) fn held_goal_denial(&self, against: legatum_core::TeamId) -> i32 {
        let v = self.view();
        v.players
            .iter()
            .filter(|p| v.team_of(p.id) == against)
            .flat_map(|p| p.vp_sources.iter())
            .map(|(src, _)| match src {
                VpSource::FactionGoal { of }
                    if v.players
                        .iter()
                        .any(|q| q.id == *of && q.faction == Faction::Byzantium) =>
                {
                    goals::spec(Faction::Byzantium).vp as i32 * HEIR_SWING_VALUE_PER_VP
                }
                VpSource::FactionGoal { of } => {
                    self.hall_goal_denial(*of) + self.goal_city_goal_denial(*of)
                }
                _ => 0,
            })
            .sum()
    }
}

fn holds_goal(sources: &[(VpSource, u32)], of: legatum_core::PlayerId) -> bool {
    sources
        .iter()
        .any(|(src, _)| matches!(src, VpSource::FactionGoal { of: o } if *o == of))
}

#[cfg(test)]
#[path = "goal_forge_heir_tests.rs"]
mod tests;

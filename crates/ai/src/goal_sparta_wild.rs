//! Faction-goal decisions for Sparta's Honour the Fallen and the Celts' Call the Wild. Both
//! goals are a cost the faction chooses to pay, so each is a value question rather than a quota:
//! when giving up three veterans or living among camps is worth 2 VP, and how rivals profit from
//! it. The one-time decisions themselves (retire, call) are in `goal_actions.rs`; this is what
//! the rest of the planner does around them: the War Shrine, gathering the veterans, keeping the
//! Celts' camps, not settling after the call, and rivals clearing a Celt's camps or punishing a
//! retired army. Everything reads the fogged view alone.

use crate::goal_actions;
use crate::scripted::Planner;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{BuildingKind, CityId, Faction, Hex, Order, VpSource, data, goals};

/// Garrison a retired army's city is expected to hold, and what each missing defender is worth to
/// a rival weighing its capture (public: Sparta's goal is done once its veterans retired).
const RETIRED_FULL_GARRISON: i32 = 3;
const RETIRED_VALUE_PER_MISSING_UNIT: i32 = 45;
/// Production preference for a military unit in a War Shrine city while Honour the Fallen is open.
const SHRINE_TRAINING_BONUS: i32 = 60;
/// A rival goes this much farther for a camp whose clearing takes 2 VP from a Celts empire.
pub(crate) const DENIAL_CAMP_RANGE: i32 = 4;
/// The longest walk to a Celts's camp, and the health a unit must have to go.
const WILD_APPROACH_STEPS: i32 = 8;
const WILD_ARRIVAL_MIN_HP: i32 = 40;

impl Planner<'_> {
    // ---------- Sparta: Honour the Fallen ----------

    fn city_has_shrine(c: &CityView) -> bool {
        c.detail
            .as_ref()
            .is_some_and(|d| data::has_building(&d.buildings, BuildingKind::WarShrine))
    }

    /// Own cities with a War Shrine standing.
    pub(crate) fn shrine_cities(&self) -> Vec<&CityView> {
        self.ctx
            .own_cities
            .iter()
            .copied()
            .filter(|c| Self::city_has_shrine(c))
            .collect()
    }

    /// Where Sparta's one War Shrine should go while they have none: the city with the
    /// most Production out of resistance (veterans stay near the capital's army). None once one
    /// stands or is being built, or when the goal is done.
    pub(crate) fn shrine_site(&self) -> Option<CityId> {
        if self.faction != Faction::Sparta
            || self.view().actions.retire.is_none()
            || self.shrine_planned()
        {
            return None;
        }
        self.ctx
            .own_cities
            .iter()
            .filter(|c| c.detail.as_ref().is_some_and(|d| d.resistance <= 0))
            .max_by_key(|c| {
                (
                    c.detail.as_ref().map_or(0, |d| d.yields.total.production),
                    std::cmp::Reverse(c.id),
                )
            })
            .map(|c| c.id)
    }

    /// A War Shrine stands or is being built anywhere in the empire.
    fn shrine_planned(&self) -> bool {
        let shrine = legatum_core::ProductionItem::Building {
            building: BuildingKind::WarShrine,
        };
        self.ctx.own_cities.iter().any(|c| {
            Self::city_has_shrine(c)
                || c.detail
                    .as_ref()
                    .is_some_and(|d| d.producing == Some(shrine))
                || self.orders.iter().any(|o| {
                    matches!(o, Order::SetProduction { city, item } | Order::BuyItem { city, item }
                        if *city == c.id && *item == shrine)
                })
        })
    }

    /// Production value of the War Shrine in `c`: the goal needs one, and every Sparta unit
    /// trained there starts a level higher, so veterans come sooner. Only in the chosen city, and
    /// only once the empire has the economy to spare it (a third of the game is gone, or a veteran
    /// already waits).
    pub(crate) fn shrine_value(&self, c: &CityView) -> i32 {
        if self.shrine_site() != Some(c.id) {
            return 0;
        }
        let early = self.view().turn * 100 < legatum_core::data::EXPECTED_GAME_TURNS * 25;
        let veteran = self
            .ctx
            .own_units
            .iter()
            .any(|u| u.military && u.level >= goals::SPARTA_RETIRE_LEVEL);
        if early && !veteran {
            return 0;
        }
        goals::spec(Faction::Sparta).vp as i32 * 150
    }

    /// Sparta military trained in a War Shrine city start a level higher, closer to the level
    /// the goal needs: a small preference for training there while the goal is open.
    pub(crate) fn shrine_training_bonus(&self, c: &CityView, unit: legatum_core::UnitKind) -> i32 {
        if self.faction == Faction::Sparta
            && self.view().actions.retire.is_some()
            && Self::city_has_shrine(c)
            && legatum_core::data::unit_stats(unit).military
        {
            SHRINE_TRAINING_BONUS
        } else {
            0
        }
    }

    /// In the retirement window, bring every veteran to the nearest War Shrine city and hold him
    /// there, so the retirement can happen in one turn. Only with enough veterans to complete the
    /// goal (a lone veteran gathers nothing), and never leaving a city that needs him.
    pub(crate) fn gather_veterans(&mut self) {
        let Some(r) = self.view().actions.retire.clone() else {
            return;
        };
        let need = usize::from(r.target.saturating_sub(r.retired));
        let veterans: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.level >= r.level && u.kind != legatum_core::UnitKind::Scout)
            .collect();
        if need == 0
            || veterans.len() < need
            || !goal_actions::retire_window(self.view(), self.decisions.goal_open_end > 0)
        {
            return;
        }
        let shrines: Vec<Hex> = self.shrine_cities().iter().map(|c| c.pos).collect();
        for u in veterans {
            if self.has_acted(u.id) {
                continue;
            }
            let Some(to) = shrines
                .iter()
                .copied()
                .filter(|h| self.ctx.reachable(u.pos, *h))
                .min_by_key(|h| (u.pos.distance(*h), *h))
            else {
                continue;
            };
            if to == u.pos {
                self.mil_dest.insert(to);
                self.unit_order(u.id, Order::SkipUnit { unit: u.id });
            } else {
                self.go_to(u, to);
            }
        }
    }

    // ---------- rivals: punishing a retired army ----------

    /// Value of capturing a city of an Sparta empire whose goal is done (public: its three
    /// level-3 veterans are gone) while the city's visible garrison is thin: each missing
    /// defender below `RETIRED_FULL_GARRISON` is worth `RETIRED_VALUE_PER_MISSING_UNIT`.
    pub(crate) fn retired_army_weakness_value(&self, city: &CityView) -> i32 {
        let v = self.view();
        if v.team_of(city.owner) == v.team
            || !v
                .players
                .iter()
                .any(|p| p.id == city.owner && p.faction == Faction::Sparta && p.faction_goal_done)
        {
            return 0;
        }
        let garrison = v
            .units
            .iter()
            .filter(|u| {
                u.owner == city.owner
                    && u.military
                    && u.kind != legatum_core::UnitKind::Scout
                    && u.pos.distance(city.pos) <= 1
            })
            .count() as i32;
        (RETIRED_FULL_GARRISON - garrison).max(0) * RETIRED_VALUE_PER_MISSING_UNIT
    }

    // ---------- Celts: Call the Wild ----------

    /// The Celts want their camps now: the Wild's 2 VP decide the game (or it is late). While true
    /// they keep one camp in every city and settle nothing new.
    pub(crate) fn wild_wanted(&self) -> bool {
        self.faction == Faction::Celts
            && goal_actions::wild_window(
                self.view(),
                self.decisions.goal_open_end > 0,
                &self.decisions,
            )
    }

    /// Whether this empire may found another city. A Celts that has called the Wild and still wants
    /// it founds none: the new city has no camp, and only another call (doubling the camps in
    /// every other city) would hold the goal again.
    pub(crate) fn wild_allows_settling(&self) -> bool {
        // Gene `celts_playbook` part 2: the next call covers a new city, so founding only waits
        // while the goal decides the hold (the new city would lapse it below the target).
        if self.celts_playbook(crate::celts_playbook::WILD) {
            let v = self.view();
            let held = v.faction_goal.as_ref().is_some_and(|g| g.done);
            return !(held && goal_actions::standing(v).is_some_and(|(vp, target)| vp >= target));
        }
        !(self.wild_wanted()
            && self
                .view()
                .actions
                .wild
                .as_ref()
                .is_some_and(|w| w.with_camp > 0))
    }

    /// The Celts empire (outside our team) in whose territory a known camp stands, when it scores
    /// Call the Wild right now (public: its `vp_sources` carry its `FactionGoal`), or is within
    /// one camp of scoring it (the known camps in its land number at least its cities less one):
    /// clearing the camp takes 2 VP from it or keeps them from it.
    pub(crate) fn wild_camp_denial(&self, camp: Hex) -> bool {
        let v = self.view();
        let Some(owner) = v.tile(camp).and_then(|t| t.owner) else {
            return false;
        };
        v.team_of(owner) != v.team
            && v.players.iter().any(|p| {
                p.id == owner
                    && p.faction == Faction::Celts
                    && (p.vp_sources.iter().any(
                        |(src, _)| matches!(src, VpSource::FactionGoal { of } if *of == owner),
                    ) || self.wild_goal_near(p))
            })
    }

    /// A Celts empire one camp short of Call the Wild: the camps we know in its territory number at
    /// least its cities less one. Public information only (its cities, tile owners, camps seen).
    fn wild_goal_near(&self, celts: &legatum_core::view::PlayerPublic) -> bool {
        let v = self.view();
        let known = v
            .barbarian_camps
            .iter()
            .filter(|c| v.tile(c.pos).and_then(|t| t.owner) == Some(celts.id))
            .count() as i64;
        celts.cities > 0 && known >= i64::from(celts.cities) - 1
    }

    /// The next step toward a camp in a Celts's land, for a rival taking its 2 VP away: a short
    /// walk of at most `WILD_APPROACH_STEPS` steps by a unit that is healthy enough to fight there.
    pub(crate) fn wild_camp_approach(&self, u: &UnitView, to: Hex) -> Option<Hex> {
        let path = legatum_core::path::find_path(self.movement_map(), u.pos, to, |h| {
            self.ctx
                .move_cost(h)
                .filter(|_| h == to || self.free_tile_for(u, h))
        })?;
        if path.len() as i32 > WILD_APPROACH_STEPS || u.hp < WILD_ARRIVAL_MIN_HP {
            return None;
        }
        path.first().copied()
    }

    /// The city of ours whose territory holds `at`: the city the goal counts a camp there for
    /// (`goals::wild_camp_cities`). The nearest city is not the same: territory is claimed, not
    /// drawn by distance, and clearing a camp on that guess left cities with none.
    pub(crate) fn territory_city(&self, at: Hex) -> Option<legatum_core::CityId> {
        self.ctx
            .own_cities
            .iter()
            .find(|c| c.detail.as_ref().is_some_and(|d| d.territory.contains(&at)))
            .map(|c| c.id)
    }
}

#[cfg(test)]
#[path = "goal_sparta_wild_tests.rs"]
mod tests;

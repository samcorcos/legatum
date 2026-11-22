//! Mongols's goal, "The Open Road": the Courier Hall (2 VP to whoever owns its city). While it
//! stands every empire has Peace passage through its holder's land, so the Hall is a cost the
//! Mongols chooses to pay: its price and the exposure of its open land against 2 VP, which also
//! change hands with the city. The decisions here, all from the fogged view:
//! - **build**: when expansion is done, the land can be defended and the 2 VP decide something
//!   (the same rite test as the broadcast), in the safest city;
//! - **defend**: whoever holds a Hall city (the Mongols or a captor) garrisons, walls and
//!   defends it first;
//! - **capture**: rivals price the Hall city by the 2 VP swing; the Mongols prices retaking it
//!   higher (`hall_in_city_value`, folded into `hoard_denial_value`, and `held_goal_denial`).
//!
//! The same public facts (`PlayerView::courier_halls`, `vp_sources`) serve every empire.

use crate::goal_actions::{hold_turns, is_late, last_turn, rite_pays, standing};
use crate::scripted::Planner;
use legatum_core::view::{CityView, CourierHallView};
use legatum_core::{BuildingKind, Faction, ProductionItem, UnitKind, goals};

/// Cities the Mongols has before the Hall is worth its exposure (its opening is expansion).
const HALL_MIN_CITIES: usize = 3;
/// Share of the game (percent of its expected length) after which expansion is over even where
/// land is left; before it, a Mongols with room keeps settling.
const HALL_PHASE_PERCENT: u32 = 45;
/// Share of the game after which the Hall's 2 VP are banked whatever the race looks like: its
/// exposure then lasts few turns.
const HALL_LATE_PERCENT: u32 = 60;
/// VP the team is expected to earn while the Hall is under construction: the Hall pays when it
/// would reach the target within this lead.
const HALL_LEAD_VP: u32 = 2;
/// Production value of the Hall, per goal VP, in the city chosen to hold it.
const HALL_VALUE_PER_VP: i32 = 1500;
/// The Hall's value per goal VP when those points take the team to its target (`hall_decides_the_game`):
/// the game itself. A build of 45 turns is discounted against a garrison unit that takes five, so
/// at the ordinary 1,500 per VP the Hall city built units for 400 turns and never the Hall
/// (seed 22229, 2026-10-23).
const HALL_DECISIVE_VALUE_PER_VP: i32 = 4000;
/// A held Hall city is worth this much per VP when capturing it (the swing is double: the
/// holder loses the points as the captor gains them).
const HALL_SWING_VALUE_PER_VP: i32 = 100;
/// The Mongols retaking its own Hall: its passage and its points both come back.
const HALL_RETAKE_VALUE_PER_VP: i32 = 250;
/// Military units at or beside a Hall city that count as a garrison for its open land.
const HALL_GARRISON: usize = 2;
/// A threatened Hall city is defended against this percent of the visible attack.
pub(crate) const HALL_DEFENSE_MARGIN_PERCENT: i32 = 150;

/// One turn's Hall decision (`Planner::hall_plan`).
pub(crate) struct HallPlan {
    site: Option<legatum_core::CityId>,
    wanted: bool,
    ready: bool,
}

impl Planner<'_> {
    fn is_hall(item: ProductionItem) -> bool {
        matches!(
            item,
            ProductionItem::Wonder {
                building: BuildingKind::CourierHall,
                ..
            }
        )
    }

    fn hall_item(&self, c: &CityView) -> Option<ProductionItem> {
        c.detail
            .as_ref()?
            .options
            .iter()
            .filter(|o| Self::is_hall(o.item))
            .min_by_key(|o| self.wonder_tile_penalty(o))
            .map(|o| o.item)
    }

    /// Our own Courier Hall, wherever it stands and whoever holds it.
    fn own_hall(&self) -> Option<&CourierHallView> {
        self.view()
            .courier_halls
            .iter()
            .find(|h| h.of == self.ctx.me)
    }

    /// The Hall city we hold (our own Hall, or one we captured).
    pub(crate) fn holds_hall_city(&self, c: &CityView) -> bool {
        self.view()
            .courier_halls
            .iter()
            .any(|h| h.city == c.id && h.holder == self.ctx.me)
    }

    /// Military units standing at `c` or beside it.
    fn hall_garrison(&self, c: &CityView) -> usize {
        self.ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 1)
            .count()
    }

    /// A unit holds the city tile itself and the garrison is complete.
    fn hall_garrisoned(&self, c: &CityView) -> bool {
        self.hall_garrison(c) >= HALL_GARRISON
            && self
                .ctx
                .own_units
                .iter()
                .any(|u| u.military && u.kind != UnitKind::Scout && u.pos == c.pos)
    }

    /// Expansion is the core loop: the Hall waits until the planned expansion is done (the land
    /// is full, or nearly half the game is gone).
    fn hall_expansion_done(&self) -> bool {
        self.ctx.own_cities.len() >= HALL_MIN_CITIES
            && (self.view().turn * 100
                >= legatum_core::data::EXPECTED_GAME_TURNS * HALL_PHASE_PERCENT
                || !self.expansion_room())
    }

    /// Turns to build the Hall in `c`: its quote when startable, else its price over the city's
    /// Production.
    fn hall_build_turns(&self, c: &CityView) -> i32 {
        let d = c.detail.as_ref().expect("own city");
        if let Some(opt) = d.options.iter().find(|o| Self::is_hall(o.item)) {
            return opt.turns.max(1);
        }
        let cost = self
            .view()
            .rule_experiments
            .building_stats_for(self.faction, BuildingKind::CourierHall)
            .cost;
        (cost + d.yields.total.production.max(1) - 1) / d.yields.total.production.max(1)
    }

    /// The 2 VP decide something: with the lead the Hall earns while it is built, they take the
    /// team to the target with time to hold it, or the game is late enough to bank them.
    fn hall_pays(&self, build: i32) -> bool {
        let v = self.view();
        let Some((vp, _)) = standing(v) else {
            return false;
        };
        let points = goals::spec(Faction::Mongols).vp;
        let open_end = self.decisions.goal_open_end > 0;
        let late = is_late(v, HALL_LATE_PERCENT, open_end);
        let hold = if late { 0 } else { hold_turns(v) as i32 };
        rite_pays(v, vp + points + HALL_LEAD_VP, HALL_LATE_PERCENT, open_end)
            && i64::from(v.turn) + i64::from(build) + i64::from(hold)
                <= i64::from(last_turn(v, open_end))
    }

    /// The Hall's 2 VP take the team to its target. Then no check of the open land's exposure
    /// holds the build back: with the points the team wins, without them it sits one rite short
    /// (2026-10-23: a 9-VP Mongols with 18 cities never started its Hall in 900 turns, because
    /// "any city of ours under fire" is true all game at war with six empires, and the garrison
    /// the check asked for is built by the Hall city's own priority once the Hall is wanted).
    fn hall_decides_the_game(&self) -> bool {
        standing(self.view()).is_some_and(|(vp, target)| {
            vp < target && vp + goals::spec(Faction::Mongols).vp >= target
        })
    }

    /// Visible armies that could take the open land: a rival team whose public army is much
    /// larger than ours, or any city of ours already under more fire than it answers.
    fn hall_outmatched(&self) -> bool {
        let v = self.view();
        let mine = crate::diplomacy::strength(v, v.team);
        let strongest_neighbour = v
            .players
            .iter()
            .filter(|p| p.alive && !p.id.is_barbarian() && !v.same_team(v.me, p.id))
            .filter(|p| !v.allied(v.me, p.id))
            .filter(|p| {
                v.cities.iter().any(|c| {
                    c.owner == p.id
                        && self
                            .ctx
                            .own_cities
                            .iter()
                            .any(|home| home.pos.distance(c.pos) <= 14)
                })
            })
            .map(|p| crate::diplomacy::strength(v, p.team))
            .max()
            .unwrap_or(0);
        strongest_neighbour > mine * 2
            || self
                .ctx
                .own_cities
                .iter()
                .enumerate()
                .any(|(i, c)| self.ctx.threat[i] > self.local_defense(c))
    }

    /// How safe `c` is to hold the Hall: far from rival cities, walled, garrisoned and not a
    /// frontier. Higher is safer.
    fn hall_site_score(&self, i: usize, c: &CityView) -> i32 {
        let v = self.view();
        let rival_distance = v
            .cities
            .iter()
            .filter(|o| !v.same_team(v.me, o.owner) && !o.owner.is_barbarian())
            .map(|o| o.pos.distance(c.pos))
            .min()
            .unwrap_or(12)
            .min(12);
        3 * rival_distance
            + if c.walls { 8 } else { 0 }
            + 3 * self.hall_garrison(c).min(3) as i32
            + if c.is_capital { 6 } else { 0 }
            - if self.exposed_city(c) { 10 } else { 0 }
            - 5 * i32::from(self.threat_exposure(c))
            - self.ctx.threat[i].min(100) / 10
    }

    /// The city that should hold the Hall: the safest one that can start it.
    fn choose_hall_site(&self) -> Option<&CityView> {
        self.ctx
            .own_cities
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.detail.as_ref().is_some_and(|d| {
                    d.resistance <= 0 && d.options.iter().any(|o| Self::is_hall(o.item))
                })
            })
            .max_by_key(|(i, c)| (self.hall_site_score(*i, c), std::cmp::Reverse(c.id)))
            .map(|(_, c)| *c)
    }

    /// The Hall's site and whether it is wanted and ready this turn (one evaluation per turn).
    fn hall_plan(&self) -> &HallPlan {
        self.hall_plan.get_or_init(|| {
            let none = HallPlan {
                site: None,
                wanted: false,
                ready: false,
            };
            if self.faction != Faction::Mongols
                || self.own_hall().is_some()
                || self.ctx.own_cities.is_empty()
                || !self.hall_expansion_done()
            {
                return none;
            }
            // A Hall already under construction stays the plan while it can still finish: the
            // race tightening or loosening mid-build never strands its Production.
            let building = self.ctx.own_cities.iter().copied().find(|c| {
                c.detail.as_ref().is_some_and(|d| {
                    d.resistance <= 0
                        && (d.producing.is_some_and(Self::is_hall)
                            || d.production_queue.iter().copied().any(Self::is_hall))
                })
            });
            let Some(site) = building.or_else(|| self.choose_hall_site()) else {
                return none;
            };
            let build = self.hall_build_turns(site);
            let wanted = if building.is_some() {
                i64::from(self.view().turn) + i64::from(build)
                    <= i64::from(last_turn(self.view(), self.decisions.goal_open_end > 0))
            } else {
                self.hall_pays(build)
            };
            HallPlan {
                site: Some(site.id),
                wanted,
                ready: wanted
                    && ((self.decisions.hall_decisive > 0 && self.hall_decides_the_game())
                        || (self.hall_garrisoned(site) && !self.hall_outmatched())),
            }
        })
    }

    /// The city chosen to hold the Hall.
    pub(crate) fn hall_site(&self) -> Option<&CityView> {
        let id = self.hall_plan().site?;
        self.ctx.own_cities.iter().copied().find(|c| c.id == id)
    }

    /// The Mongols should hold a Hall soon: it has none, expansion is done and its 2 VP pay.
    /// A Hall standing anywhere, even in a rival's captured city, rules out a second one.
    pub(crate) fn hall_wanted(&self) -> bool {
        self.hall_plan().wanted
    }

    /// The Hall is started now: wanted, its city garrisoned and nothing outmatches us.
    pub(crate) fn hall_ready(&self) -> bool {
        self.hall_plan().ready
    }

    /// Production value of the Hall in `c`: nothing unless it is the chosen city and the Hall is
    /// ready; then the whole of the goal.
    pub(crate) fn hall_value(&self, c: &CityView, turns: i32) -> i32 {
        if turns < 0 || !self.hall_ready() || self.hall_plan().site != Some(c.id) {
            return 0;
        }
        let per_vp = if self.decisions.hall_decisive > 0 && self.hall_decides_the_game() {
            HALL_DECISIVE_VALUE_PER_VP
        } else {
            HALL_VALUE_PER_VP
        };
        goals::spec(Faction::Mongols).vp as i32 * per_vp
    }

    /// The Hall's 2 VP win the game and `city` is the one that should build it: it drops what it is
    /// producing for the Hall. Cities repeat a unit they are building, so without this a busy
    /// Hall city never became idle and the goal pass never reached it (seed 22229, 2026-10-23: a
    /// 9-VP Mongols built Drones in all 14 cities for 250 turns).
    pub(crate) fn hall_takes_over(&self, city: legatum_core::CityId, item: ProductionItem) -> bool {
        self.decisions.hall_decisive > 0
            && Self::is_hall(item)
            && self.hall_plan().site == Some(city)
            && self.hall_ready()
            && self.hall_decides_the_game()
    }

    /// The Hall for the goal pass, which may buy it with Gold (the pass pays reserve, upkeep and
    /// the garrison quota first).
    pub(crate) fn hall_steps(&self, steps: &mut Vec<(legatum_core::CityId, ProductionItem)>) {
        if !self.hall_ready() {
            return;
        }
        if let Some(site) = self.hall_site()
            && !steps.iter().any(|(id, _)| *id == site.id)
            && let Some(item) = self.hall_item(site)
        {
            steps.push((site.id, item));
        }
    }

    /// `c` holds the Hall, or is the city that will: the first place to garrison and wall.
    pub(crate) fn hall_guard_city(&self, c: &CityView) -> bool {
        self.holds_hall_city(c) || (self.hall_wanted() && self.hall_plan().site == Some(c.id))
    }

    /// A Hall city without its garrison: units are the first thing it builds.
    pub(crate) fn hall_unguarded(&self, c: &CityView) -> bool {
        self.hall_guard_city(c) && !self.hall_garrisoned(c)
    }

    /// Percent of a visible attack the defenders of `c` must cover: a Hall city is defended with
    /// a wider margin than any other.
    pub(crate) fn hall_defense_percent(&self, c: &CityView) -> i32 {
        if self.holds_hall_city(c) {
            HALL_DEFENSE_MARGIN_PERCENT
        } else {
            100
        }
    }

    // ---------- rivals: capturing the Hall city ----------

    /// Value of capturing `city` if a Courier Hall stands in it and its holder is not on our team:
    /// the captor scores its 2 VP and the holder loses them. The Mongols whose Hall it is values
    /// retaking it higher. A Celts capture razes the city and the Hall with it, so it only denies.
    pub(crate) fn hall_in_city_value(&self, city: &CityView) -> i32 {
        let v = self.view();
        let Some(hall) = v.courier_halls.iter().find(|h| h.city == city.id) else {
            return 0;
        };
        if v.same_team(v.me, hall.holder) {
            return 0;
        }
        let vp = goals::spec(Faction::Mongols).vp as i32;
        if hall.of == v.me {
            vp * HALL_RETAKE_VALUE_PER_VP
        } else if self.faction == Faction::Celts {
            vp * HALL_SWING_VALUE_PER_VP / 2
        } else {
            vp * HALL_SWING_VALUE_PER_VP
        }
    }

    /// Denial value of a holder scoring `of`'s goal when `of` is a Mongols: the Hall's swing,
    /// more when it is ours to retake. Zero for any other faction.
    pub(crate) fn hall_goal_denial(&self, of: legatum_core::PlayerId) -> i32 {
        let v = self.view();
        if !v
            .players
            .iter()
            .any(|q| q.id == of && q.faction == Faction::Mongols)
        {
            return 0;
        }
        let vp = goals::spec(Faction::Mongols).vp as i32;
        vp * if of == v.me {
            HALL_RETAKE_VALUE_PER_VP
        } else {
            HALL_SWING_VALUE_PER_VP
        }
    }
}

#[cfg(test)]
#[path = "goal_hall_tests.rs"]
mod tests;

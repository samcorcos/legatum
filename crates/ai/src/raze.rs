//! Gene `raze_value`: keep or raze a captured city by what it is worth (decision audit
//! 2026-10-31: Hard razed captured cities in four of the designer's five wins; he kept every
//! one, worth +0.11 cities per 10-turn window). From public information and our own cities only.
//!
//! Keeping is worth the city's output over a horizon (estimated from our settled cities' yield
//! per population, since a captured city yields nothing during resistance), its growth and the
//! live "gain cities" objectives it counts for, less the extra Decadence its output adds to our
//! lead. The keep arm then either pays to defend it (Gold for the defenders the visible pressure
//! asks for) or risks the forecast loss, in which case its former owner regains it. Razing pays
//! population x 10 Gold and denies that recapture. Celtic sacks are a capture rule, not this
//! order, and stay as they are.
use crate::analysis::defense_power;
use crate::purchase_defense::purchased_defender;
use crate::scripted::Planner;
use legatum_core::view::CityView;
use legatum_core::{BuildingKind, ProductionItem};

/// Turns of a kept city's output the decision counts.
const HORIZON: i32 = 40;
/// Turns per population a growing city adds.
const GROWTH_TURNS: i32 = 15;
/// Gold-equivalent of one VP (an objective the city counts for).
pub(crate) const VP_GOLD: i32 = 300;
/// A rival's regained city counts at this percent of its value to us (one rival among several).
const RIVAL_WEIGHT: i32 = 50;
/// Chance of losing a city the visible attack is forecast to take this turn, in percent.
const FALL_FORECAST: i32 = 90;
/// Loss chance, in percent, of a city facing only visible pressure it cannot match, scaled by
/// the unmatched share: an attack can reach it this turn, or not yet (troops can still come).
const REACH_NOW: i32 = 75;
const REACH_LATER: i32 = 40;

/// VP that would be restored to a former holder if this captured city were retaken.
pub(crate) fn held_vp_at_city(city: &CityView) -> i32 {
    let goals = city.detail.as_ref().map_or(0, |detail| {
        i32::from(detail.buildings.contains(&BuildingKind::CourierHall))
    });
    (city.great_works.len() as i32 + goals) * 2
}

impl Planner<'_> {
    /// The 1 VP an original capital we hold would give a rival that takes it from us: 1 while
    /// some other empire, neither us nor the capital's founder's team, could hold it (rules 50:
    /// the point is held, so whoever takes it next scores it). Razing the city removes it from
    /// the pool (designer ruling 2026-11-01: every city can be razed, original capitals included).
    pub(crate) fn rival_capital_vp(&self, city: &CityView) -> i32 {
        if !city.original_capital {
            return 0;
        }
        let me = self.ctx.me;
        let my_team = self
            .view()
            .players
            .iter()
            .find(|p| p.id == me)
            .map(|p| p.team);
        let founder_team = city.original_capital_of.and_then(|f| {
            self.view()
                .players
                .iter()
                .find(|p| p.id == f)
                .map(|p| p.team)
        });
        let open = self.view().players.iter().any(|p| {
            p.alive
                && p.id != me
                && Some(p.team) != my_team
                && Some(p.team) != founder_team
                && city.original_capital_of != Some(p.id)
        });
        i32::from(open)
    }

    /// The 1 VP we score while we hold `city`, a rival's original capital (rules 50: held, not
    /// kept for good; mirrors `legatum_core::scoring::capitals_held`).
    pub(crate) fn own_capital_vp(&self, city: &CityView) -> i32 {
        let v = self.view();
        let team_of = |p| v.players.iter().find(|q| q.id == p).map(|q| q.team);
        i32::from(
            city.original_capital
                && city.owner == self.ctx.me
                && city
                    .original_capital_of
                    .is_some_and(|f| team_of(f) != team_of(self.ctx.me)),
        )
    }

    /// Whether to raze captured city `i` (`ctx.own_cities[i]`) under `raze_value`.
    pub(crate) fn raze_by_value(&self, i: usize, city: &CityView) -> bool {
        let Some(detail) = &city.detail else {
            return false;
        };
        let v = self.view();
        let loot = city.population.saturating_mul(10);
        let horizon = (self.turns_remaining().min(HORIZON) - detail.resistance.max(0)).max(0);
        // Yield per population in hundredths, from our cities past resistance.
        let (out, pop) = self
            .ctx
            .own_cities
            .iter()
            .filter_map(|c| c.detail.as_ref().map(|d| (c, d)))
            .filter(|(_, d)| d.resistance <= 0)
            .fold((0i64, 0i64), |(o, p), (c, d)| {
                let y = d.yields.total;
                (
                    o + i64::from(y.food + y.production + y.gold + y.science),
                    p + i64::from(c.population),
                )
            });
        let per_pop = if pop > 0 {
            (out * 100 / pop) as i32
        } else {
            300
        };
        let base = city.population * per_pop / 100;
        let output = base * horizon + per_pop * horizon * horizon / (2 * GROWTH_TURNS * 100);
        // Decadence: the city's output raises our Power, our lead over the mean and the rate
        // that cuts every yield.
        let alive: Vec<_> = v.players.iter().filter(|p| p.alive).collect();
        let decadence = match alive.iter().find(|p| p.id == v.me) {
            _ if v.rule_experiments.decadence_slope() == 0 => 0,
            // Core waits for a full Power window before decadence can begin. A captured city
            // early in the game cannot make its owner pay a rate on turns that have no history.
            _ if v.turn <= u32::from(legatum_core::experiments::POWER_WINDOW_TURNS) => 0,
            Some(me) if !alive.is_empty() => {
                let n = alive.len() as i64;
                let total: i64 = alive.iter().map(|p| i64::from(p.power)).sum();
                let lead = |mine: i64, total: i64| {
                    let mean = (total / n).max(1);
                    ((mine - mean) * 100 / mean) as i32
                };
                let power = i64::from(me.power);
                let before = v.rule_experiments.decadence_percent(lead(power, total));
                let after = v
                    .rule_experiments
                    .decadence_percent(lead(power + i64::from(base), total + i64::from(base)));
                (after - before).max(0) * (me.power + base) / 100 * horizon
            }
            _ => 0,
        };
        let vp = self
            .live_objectives()
            .iter()
            .filter(|o| {
                matches!(
                    o.objective,
                    legatum_core::objectives::Objective::Cities { .. }
                ) && o.remaining() > 0
            })
            .count() as i32;
        let site_vp = held_vp_at_city(city) * VP_GOLD;
        // Rules 50: a rival's original capital scores for us only while we hold it, so razing
        // it gives that point up.
        let worth =
            output - decadence + (vp * VP_GOLD) + site_vp + self.own_capital_vp(city) * VP_GOLD;
        // A rival that takes an original capital from us scores its point; razing denies it.
        let rival_point = (self.rival_capital_vp(city) + self.capital_restore_vp(city)) * VP_GOLD;
        // Defence: the visible pressure on it against the troops standing by.
        let pressure = (self.ctx.threat[i] - self.ctx.barbarian_threat[i]).max(0);
        let defense = self.local_defense(city);
        let risk = self.defense_risk(city, None);
        let fall = if risk.falls {
            FALL_FORECAST
        } else if pressure > defense {
            let reach = if risk.threatened {
                REACH_NOW
            } else {
                REACH_LATER
            };
            (pressure - defense) * reach / pressure
        } else {
            0
        };
        let regain = if detail.former_owner.is_some() {
            (output + site_vp) * RIVAL_WEIGHT / 100
        } else {
            0
        };
        let risked = worth * (100 - fall) / 100 - (regain + rival_point) * fall / 100;
        // Defenders must be affordable within a few turns of income to count.
        let budget = self.gold_left + v.economy.gold_per_turn.total.max(0) * 5;
        let defended = self
            .defend_cost(city, pressure - defense)
            .filter(|cost| *cost <= budget)
            .map_or(i32::MIN, |cost| worth - cost);
        loot > risked.max(defended)
    }

    /// Gold for enough of our cheapest buyable defender to cover `gap` power, or None
    /// when no settled city can buy one.
    fn defend_cost(&self, city: &CityView, gap: i32) -> Option<i32> {
        if gap <= 0 {
            return Some(0);
        }
        let v = self.view();
        self.ctx
            .own_cities
            .iter()
            .filter_map(|c| c.detail.as_ref().map(|d| (c, d)))
            .filter(|(_, d)| d.resistance <= 0)
            .flat_map(|(_, d)| d.options.iter())
            .filter_map(|o| {
                let ProductionItem::Unit { unit } = o.item else {
                    return None;
                };
                if !crate::production::is_military_unit(o.item) || o.buy_block.is_some() {
                    return None;
                }
                let cost = o.buy_cost.filter(|c| *c > 0)?;
                let power = defense_power(v, &purchased_defender(v, city, unit)).max(1);
                Some(cost.saturating_mul((gap + power - 1) / power))
            })
            .min()
    }
}

#[cfg(test)]
mod tests {
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use legatum_core::{Hex, Order, PlayerView, UnitKind};

    /// A captured population-3 city at (8,8) and a settled population-4 home city yielding 12.
    fn view(enemies: &[(u32, Hex)]) -> PlayerView {
        let mut b = ViewBuilder::new(24, 18)
            .city(1, ME, Hex::new(8, 8), 3)
            .city(2, ME, Hex::new(16, 8), 4);
        for (id, at) in enemies {
            b = b.unit(*id, ENEMY, UnitKind::Soldier, *at);
        }
        let mut view = b.build();
        view.cities[0].is_capital = false;
        let d = view.cities[0].detail.as_mut().unwrap();
        d.captured = true;
        d.former_owner = Some(ENEMY);
        d.raze_blocker = None;
        let home = view.cities[1].detail.as_mut().unwrap();
        home.yields.total.food = 5;
        home.yields.total.production = 4;
        home.yields.total.gold = 2;
        home.yields.total.science = 1;
        view
    }

    fn razes(view: &PlayerView, gene: i32) -> bool {
        let mut p = crate::fixture::planner(view, 7);
        p.decisions.raze_value = gene;
        p.plan_raze();
        p.orders.iter().any(|o| matches!(o, Order::Raze { .. }))
    }

    #[test]
    fn an_undefended_city_the_army_can_attack_this_turn_is_still_razed() {
        let v = view(&[
            (20, Hex::new(9, 8)),
            (21, Hex::new(9, 9)),
            (22, Hex::new(8, 9)),
        ]);
        assert!(razes(&v, 0));
        assert!(razes(&v, 1));
    }

    #[test]
    fn pressure_that_cannot_take_the_city_this_turn_no_longer_burns_it() {
        // Three Soldiers three hexes out: the old pressure gate razes, the value keeps it.
        let v = view(&[
            (20, Hex::new(11, 8)),
            (21, Hex::new(11, 7)),
            (22, Hex::new(10, 10)),
        ]);
        assert!(razes(&v, 0));
        assert!(!razes(&v, 1));
    }

    #[test]
    fn a_safe_city_is_kept_and_the_last_turns_take_the_loot() {
        let mut v = view(&[]);
        assert!(!razes(&v, 1));
        // Nothing left to earn from it: the Gold wins.
        v.turn_limit = Some(v.turn);
        assert!(razes(&v, 1));
    }

    #[test]
    fn raze_prices_the_vp_denied_by_destroying_a_work_site() {
        let mut v = view(&[]);
        v.cities[0]
            .great_works
            .push(legatum_core::BuildingKind::RelayUplink);
        assert_eq!(super::held_vp_at_city(&v.cities[0]), 2);
        assert!(!razes(&v, 1), "a safe held Great Work is worth keeping");
        let mut threatened = view(&[
            (20, Hex::new(9, 8)),
            (21, Hex::new(9, 9)),
            (22, Hex::new(8, 9)),
        ]);
        threatened.cities[0].great_works = v.cities[0].great_works.clone();
        assert!(
            razes(&threatened, 1),
            "raze denies the imminent recapture its VP"
        );
    }

    #[test]
    fn an_original_capital_holds_a_point_for_whoever_holds_it_until_razed() {
        let mut v = view(&[]);
        fn planner(v: &PlayerView) -> crate::scripted::Planner<'_> {
            let mut p = crate::fixture::planner(v, 7);
            p.decisions.raze_value = 1;
            p
        }
        v.cities[0].original_capital = false;
        assert_eq!(
            planner(&v).rival_capital_vp(&v.cities[0]),
            0,
            "not a capital"
        );
        v.cities[0].original_capital = true;
        v.cities[0].original_capital_of = Some(ENEMY);
        let others = v
            .players
            .iter()
            .filter(|p| p.id != ME && p.id != ENEMY && p.team != v.players[0].team)
            .count();
        assert_eq!(
            planner(&v).rival_capital_vp(&v.cities[0]),
            i32::from(others > 0),
            "only an empire that is neither us nor the founder can still score it"
        );
        // While we hold it, the point is ours; held, not kept, so razing it gives it up.
        v.cities[0].owner = ME;
        assert_eq!(
            planner(&v).own_capital_vp(&v.cities[0]),
            1,
            "we hold a rival's capital"
        );
        v.cities[0].original_capital_of = Some(ME);
        assert_eq!(
            planner(&v).own_capital_vp(&v.cities[0]),
            0,
            "our own capital scores nothing"
        );
    }
}

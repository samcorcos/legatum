//! Gene `breach` (Training round 2, distilled into plain Hard, 2026-11-08): shooters and aircraft
//! take an enemy city to 0 HP and the cheapest melee unit walks in the same turn. Ranged fire,
//! air strikes included, resolves before movement (core `rules/battle.rs`), so a city zeroed in
//! the ranged stage is entered by the first melee step. The designer's 8.9.27 Germany T144 took
//! Xochimilco and Chalco this way, two captures in one turn.
//!
//! The breach is priced, not scripted: the chance the shots zero the city (exact over the
//! uniform rolls), times the city's value, kept against the visible enemy power that can reach
//! it, against the walker's expected loss when the city keeps HP.

use super::*;
use legatum_core::combat;
use legatum_core::{AiDifficulty, UnitId};

/// Stop adding shots once the city falls this often (percent).
const ENOUGH: i64 = 95;
/// At most this many shots priced on one city (the exact distribution grows with each).
const MAX_SHOTS: usize = 16;

/// One shot's damage to the city's HP at each roll (uniform, `ROLL_MIN..=ROLL_MAX`).
type Shot = Vec<i32>;

/// The chance (0-100) that `shots`, each rolled independently, take `hp` to 0.
pub(crate) fn zero_chance(shots: &[Shot], hp: i32) -> i64 {
    if hp <= 0 {
        return 100;
    }
    let cap = hp as usize;
    // Ways to reach each damage total, capped at `hp`; 41 rolls a shot, at most 16 shots.
    let mut ways = vec![0u128; cap + 1];
    ways[0] = 1;
    let mut total: u128 = 1;
    for shot in shots {
        let mut next = vec![0u128; cap + 1];
        for (done, &n) in ways.iter().enumerate() {
            if n == 0 {
                continue;
            }
            for &d in shot {
                next[(done + d.max(0) as usize).min(cap)] += n;
            }
        }
        ways = next;
        total *= shot.len() as u128;
    }
    (ways[cap] * 100 / total) as i64
}

/// A priced breach on one city.
struct Breach {
    city: Hex,
    shooters: Vec<UnitId>,
    planes: Vec<UnitId>,
    walker: UnitId,
    value: i64,
}

impl Planner<'_> {
    /// Gene `breach`: take every city the shots can zero this turn, best value first.
    pub(super) fn breach_cities(&mut self, army: &[&UnitView]) {
        if self.decisions.breach == 0
            || self.difficulty != AiDifficulty::Hard
            || self.takes_no_cities()
        {
            return;
        }
        let mut done = Vec::new();
        while let Some(b) = self.best_breach(army, &done) {
            done.push(b.city);
            for id in &b.shooters {
                if let Some(u) = self.ctx.own_units.iter().find(|u| u.id == *id) {
                    self.mil_dest.insert(u.pos);
                }
                self.unit_order(
                    *id,
                    Order::RangedAttack {
                        unit: *id,
                        target: b.city,
                    },
                );
            }
            for id in &b.planes {
                self.unit_order(
                    *id,
                    Order::AirStrike {
                        plane: *id,
                        target: b.city,
                    },
                );
            }
            self.assault_attacks.insert(b.walker);
            // The walker's route is reserved: a later move that stops on it blocks the walker
            // (war suite, tepid T223: a Trooper stepped onto the Soldier's path and the city
            // stood empty at the deadline).
            if let Some(u) = self
                .ctx
                .own_units
                .iter()
                .copied()
                .find(|u| u.id == b.walker)
                && let Some(path) = self.known_path_this_turn(u, b.city)
            {
                for h in path {
                    self.mil_dest.insert(h);
                }
            }
            self.mil_dest.insert(b.city);
            self.unit_order(
                b.walker,
                Order::MoveUnit {
                    unit: b.walker,
                    to: b.city,
                },
            );
        }
    }

    fn best_breach(&self, army: &[&UnitView], done: &[Hex]) -> Option<Breach> {
        let v = self.view();
        let av = self.attack_view();
        let free = self.free(army);
        let faction = self.faction;
        let mut best: Option<Breach> = None;
        for c in self.ctx.enemy_cities.iter().copied() {
            if !c.visible
                || c.owner.is_barbarian()
                || !av.at_war(v.me, c.owner)
                || done.contains(&c.pos)
                || self.mil_dest.contains(&c.pos)
            {
                continue;
            }
            let garrison: Vec<&UnitView> = self
                .ctx
                .enemy_units
                .iter()
                .copied()
                .filter(|e| e.military && e.pos == c.pos)
                .collect();
            // Its owner's melee units that can step in first: an owner reinforcing its own
            // attacked city moves before our walker (core `rules/movement.rs::reserves`),
            // and the walker then fights it (war suite, tepid T223: the Bomber hit the city, not
            // the Spearman beside it, and the Spearman walked in).
            let reinforcers: Vec<&UnitView> = self
                .ctx
                .enemy_units
                .iter()
                .copied()
                .filter(|e| {
                    e.military
                        && e.owner == c.owner
                        && e.pos != c.pos
                        && e.ranged == 0
                        && e.pos.distance(c.pos) <= e.max_moves.max(1)
                })
                .collect();
            // The walker: a free melee unit (or a Keshig) that reaches the city this turn and
            // survives the city's shot before movement. The nearest first (an adjacent unit
            // steps straight in; a longer path can be blocked by our own moves), then the
            // cheapest.
            let Some(walker) = free
                .iter()
                .copied()
                .filter(|u| crate::composition::charges(u.kind))
                .filter(|u| u.hp > self.city_damage_now(u))
                .filter(|u| self.reaches_this_turn(u, c.pos))
                .filter(|u| forecast_attack_mode(&av, u.id, c.pos, AttackMode::Melee).is_some())
                .min_by_key(|u| {
                    (
                        u.pos.distance(c.pos),
                        data::unit_stats_for(faction, u.kind).cost,
                        u.id,
                    )
                })
            else {
                continue;
            };
            // Every shot that can reach the city now: ground shooters first, then aircraft,
            // each group by expected damage.
            let mut ground: Vec<(i32, UnitId, Shot)> = Vec::new();
            let mut air: Vec<(i32, UnitId, Shot)> = Vec::new();
            if c.hp > 0 {
                for u in free.iter().copied().filter(|u| {
                    u.id != walker.id
                        && u.ranged > 0
                        && u.range > 0
                        && u.pos.distance(c.pos) <= u.range
                        && self.ranged_unit_clear(u, u.pos, c.pos)
                }) {
                    if let Some(f) = forecast_attack_mode(&av, u.id, c.pos, AttackMode::Ranged) {
                        ground.push((f.expected_damage_to_defender, u.id, shot_rolls(&f)));
                    }
                }
                for a in v.aircraft.iter().filter(|a| {
                    a.owner == v.me
                        && !self.has_acted(a.id)
                        && (!data::is_fighter(a.kind) || self.decisions.fighter_strike != 0)
                }) {
                    let stats = data::unit_stats(a.kind);
                    let in_range = v
                        .cities
                        .iter()
                        .find(|b| b.id == a.city)
                        .is_some_and(|b| b.pos.distance(c.pos) <= stats.range);
                    if stats.ranged <= 0 || !in_range {
                        continue;
                    }
                    if let Some(f) = forecast_attack_mode(&av, a.id, c.pos, AttackMode::Ranged) {
                        air.push((f.expected_damage_to_defender, a.id, shot_rolls(&f)));
                    }
                }
                ground.sort_by_key(|(d, id, _)| (std::cmp::Reverse(*d), *id));
                air.sort_by_key(|(d, id, _)| (std::cmp::Reverse(*d), *id));
            }
            // Shots other passes already ordered at the city count first.
            let mut shots: Vec<Shot> = self
                .orders
                .iter()
                .filter_map(|o| match o {
                    Order::RangedAttack { unit, target }
                    | Order::AirStrike {
                        plane: unit,
                        target,
                    } if *target == c.pos && c.hp > 0 => {
                        forecast_attack_mode(&av, *unit, c.pos, AttackMode::Ranged)
                    }
                    _ => None,
                })
                .map(|f| shot_rolls(&f))
                .collect();
            let mut shooters = Vec::new();
            let mut planes = Vec::new();
            let mut zero = zero_chance(&shots, c.hp);
            for (is_plane, (_, id, shot)) in ground
                .into_iter()
                .map(|s| (false, s))
                .chain(air.into_iter().map(|s| (true, s)))
            {
                if zero >= ENOUGH || shots.len() >= MAX_SHOTS {
                    break;
                }
                shots.push(shot);
                if is_plane {
                    planes.push(id);
                } else {
                    shooters.push(id);
                }
                zero = zero_chance(&shots, c.hp);
            }
            if zero == 0 {
                continue;
            }
            // A garrison, or a unit that reinforces it, defends the fallen city: the walker must
            // beat the strongest of them (overflow ignored).
            let win = garrison
                .iter()
                .chain(reinforcers.iter())
                .map(|e| {
                    let mut fallen = v.clone();
                    if let Some(fc) = fallen.cities.iter_mut().find(|x| x.id == c.id) {
                        fc.hp = 0;
                    }
                    fallen.units.retain(|x| x.pos != c.pos || x.id == e.id);
                    if let Some(x) = fallen.units.iter_mut().find(|x| x.id == e.id) {
                        x.pos = c.pos;
                    }
                    forecast_attack_mode(&fallen, walker.id, c.pos, AttackMode::Melee)
                        .map_or(0, |f| i64::from(f.win_chance))
                })
                .min()
                .unwrap_or(100);
            let capture = zero * win / 100;
            if capture == 0 {
                continue;
            }
            let cost = i64::from(data::unit_stats_for(faction, walker.kind).cost);
            // Kept: the walker in the taken city against the enemy power that can reach it.
            let own = i64::from(crate::analysis::defense_power(v, walker).max(1));
            let threat = i64::from(self.enemy_exposure_key(c.pos).0.max(0));
            let hold = own * 100 / (own + threat);
            let worth =
                (400 + 80 * i64::from(c.population)) * i64::from(self.decisions.breach) / 100;
            let gain = capture * worth * hold / 10_000;
            // The walker charges a city that kept HP, or loses the fallen city's garrison
            // fight, or is caught in the city it took.
            let fail_loss = forecast_attack_mode(&av, walker.id, c.pos, AttackMode::Melee)
                .map_or(100, |f| i64::from(f.loss_chance));
            let risk = cost * ((100 - zero) * fail_loss + zero * (100 - win)) / 10_000
                + cost * capture * (100 - hold) / 10_000;
            let value = gain - risk;
            if value <= 0 {
                continue;
            }
            if best
                .as_ref()
                .is_none_or(|b| (value, c.pos) > (b.value, b.city))
            {
                best = Some(Breach {
                    city: c.pos,
                    shooters,
                    planes,
                    walker: walker.id,
                    value,
                });
            }
        }
        best
    }

    /// Gene `siege_stance`, the round-2 stances as a valuation. For each visible enemy city at
    /// war, with our free units near it:
    /// - **storm** when the strike forecast (`forecast_siege`, the free units only) takes the
    ///   city and its expected Production lost is below the city's value (400 + 80 per
    ///   population): every unit of the force hits the city, shooters from range, melee
    ///   charging it when they reach it and closing in otherwise;
    /// - else **siege** when those shooters and our aircraft zero it within the gene's turns
    ///   at their expected damage: the shooters take firing tiles (`siege_city`, which keeps
    ///   them out of the city's killing strike) and the melee hold out of its strike, within a
    ///   move of it, for the breach (gene `breach`).
    ///
    /// The running campaign's city first, then a storm over a siege, then the best value.
    pub(super) fn siege_stance_cities(&mut self, army: &[&UnitView]) {
        if self.decisions.siege_stance == 0
            || self.difficulty != AiDifficulty::Hard
            || self.takes_no_cities()
        {
            return;
        }
        let v = self.view();
        // Gene `stance_home_guard`: units this close to a home under real threat (more than
        // half its strength, as `siege_war` reads it) stay home unless the city is that close
        // to the home too; the stance doesn't take them elsewhere
        // (Mongols 8.9.25 T60 in the designer seat: the Cataphracts left the front to storm a
        // city and a young city behind them fell).
        let threatened: Vec<Hex> = if self.decisions.stance_home_guard > 0 {
            self.ctx
                .own_cities
                .iter()
                .enumerate()
                .filter(|(i, c)| {
                    self.ctx.threat[*i] > crate::analysis::city_power(self.view(), c) / 2
                })
                .map(|(_, c)| c.pos)
                .collect()
        } else {
            Vec::new()
        };
        let all_free: Vec<&UnitView> = self.free(army);
        let guard = self.decisions.stance_home_guard;
        let target = self.campaigns.active.as_ref().map(|c| c.target).or(self
            .conquest_city
            .get()
            .copied()
            .flatten());
        let av = self.attack_view().into_owned();
        // The key, the units with their tiles, and whether the storm takes the city this turn.
        type Pick<'u> = ((bool, bool, i64, Hex), Vec<(&'u UnitView, Hex)>, bool);
        let mut best: Option<Pick> = None;
        // Gene `stance_plan_war`: only the war the plan fights (its Campaign or Contain enemy)
        // or the running campaign's city. A stance on another war's city keeps a war alive the
        // plan was letting die: on India 8.9.25 from T80 the storm turned a dead war "pressing",
        // Hard refused the Peace it signed without the stance, and lost its army.
        let plan_enemy = match self.routed_plan().map(|p| &p.kind) {
            Some(crate::plan::PlanKind::Campaign { against, .. })
            | Some(crate::plan::PlanKind::Contain { against }) => Some(*against),
            _ => None,
        };
        for c in self.ctx.enemy_cities.iter().copied() {
            if self.decisions.stance_plan_war > 0
                && Some(c.id) != target
                && plan_enemy != Some(v.team_of(c.owner))
            {
                continue;
            }
            if !c.visible
                || c.hp <= 0
                || c.owner.is_barbarian()
                || !av.at_war(v.me, c.owner)
                || self.mil_dest.contains(&c.pos)
            {
                continue;
            }
            // A unit guarding a threatened home may still take on a city near that home, but
            // not one that would take it away from it.
            let free: Vec<&UnitView> = all_free
                .iter()
                .copied()
                .filter(|u| {
                    !self.last_garrison_under_attack(u)
                        && threatened
                            .iter()
                            .all(|h| h.distance(u.pos) > guard || h.distance(c.pos) <= guard)
                })
                .collect();
            let worth = 400 + 80 * i64::from(c.population);
            // Storm: the strike forecast on the free units alone. The storm orders only the
            // units that hit the city this turn (shooters in range, melee that reach it); the
            // rest stay with the other passes (a field unit in their way, say).
            let force: Vec<(&UnitView, Hex, i32)> = self
                .strike_force(&av, c)
                .into_iter()
                .filter_map(|(u, h, t)| free.iter().find(|f| f.id == u.id).map(|f| (*f, h, t)))
                .collect();
            let now: Vec<(&UnitView, Hex, i32)> = force
                .iter()
                .copied()
                .filter(|(u, _, _)| {
                    if crate::composition::shooter_only(u.kind, u.ranged) {
                        u.pos.distance(c.pos) <= u.range && self.ranged_unit_clear(u, u.pos, c.pos)
                    } else {
                        self.reaches_this_turn(u, c.pos)
                    }
                })
                .collect();
            if now
                .iter()
                .any(|(u, _, _)| crate::composition::charges(u.kind))
                && let Some(siege) = {
                    self.stance_forecast.set(true);
                    let siege = self.forecast_siege_rounds(
                        &av,
                        c,
                        &force,
                        &[],
                        Some(self.decisions.siege_stance),
                    );
                    self.stance_forecast.set(false);
                    siege
                }
                && i64::from(siege.losses) < worth
            {
                let key = (
                    Some(c.id) != target,
                    false,
                    -(worth - i64::from(siege.losses)) / i64::from(siege.turns.max(1)),
                    c.pos,
                );
                if best.as_ref().is_none_or(|(k, _, _)| key < *k) {
                    // The rest close in on their assault tiles, unless an enemy field unit
                    // stands within two tiles of them (it would block or catch the approach;
                    // the other passes deal with it).
                    let field_near = |u: &UnitView| {
                        self.ctx
                            .enemy_units
                            .iter()
                            .any(|e| e.military && e.pos != c.pos && e.pos.distance(u.pos) <= 2)
                    };
                    let approach = force
                        .iter()
                        .filter(|(u, _, _)| !now.iter().any(|(n, _, _)| n.id == u.id))
                        .filter(|(u, _, _)| {
                            self.decisions.forecast_garrison_holds > 0 && !field_near(u)
                        });
                    best = Some((
                        key,
                        now.iter()
                            .chain(approach)
                            .map(|(u, h, _)| (*u, *h))
                            .collect(),
                        siege.turns <= 1,
                    ));
                }
                continue;
            }
            // Siege: the shooters wear the city down.
            let near: Vec<&UnitView> = free
                .iter()
                .copied()
                .filter(|u| {
                    u.pos.distance(c.pos) <= self.decisions.siege_reach
                        && self.ctx.reachable(u.pos, c.pos)
                })
                .collect();
            if !near.iter().any(|u| crate::composition::charges(u.kind)) {
                continue;
            }
            let Some(turns) = self.turns_to_zero(c, &near) else {
                continue;
            };
            if turns > self.decisions.siege_stance {
                continue;
            }
            let key = (Some(c.id) != target, true, i64::from(turns), c.pos);
            if best.as_ref().is_none_or(|(k, _, _)| key < *k) {
                best = Some((key, near.iter().map(|u| (*u, c.pos)).collect(), false));
            }
        }
        let Some(((_, siege, _, city), mut units, captures_now)) = best else {
            return;
        };
        if siege {
            let near: Vec<&UnitView> = units.iter().map(|(u, _)| *u).collect();
            self.siege_city(&near, city, false);
            for u in near {
                if !self.has_acted(u.id) && crate::composition::charges(u.kind) {
                    self.siege_hold(u, city);
                }
            }
            return;
        }
        // Gene `charge_preview`: the shots go first, so a charge's preview counts them.
        if self.decisions.charge_preview > 0 {
            units.sort_by_key(|(u, _)| !crate::composition::shooter_only(u.kind, u.ranged));
        }
        for (u, tile) in units {
            if self.has_acted(u.id) {
                continue;
            }
            let shooter = crate::composition::shooter_only(u.kind, u.ranged);
            if shooter
                && u.pos.distance(city) <= u.range
                && self.ranged_unit_clear(u, u.pos, city)
                && forecast_attack_mode(&av, u.id, city, AttackMode::Ranged).is_some()
            {
                self.mil_dest.insert(u.pos);
                self.unit_order(
                    u.id,
                    Order::RangedAttack {
                        unit: u.id,
                        target: city,
                    },
                );
            } else if !shooter
                && self.reaches_this_turn(u, city)
                && !captures_now
                && self.charge_loses(u, city)
            {
                self.siege_hold(u, city);
            } else if !shooter && self.reaches_this_turn(u, city) {
                self.assault_attacks.insert(u.id);
                self.unit_order(
                    u.id,
                    Order::MoveUnit {
                        unit: u.id,
                        to: city,
                    },
                );
            } else if tile == u.pos {
                self.hold_or_upgrade(u);
            } else {
                self.go_to(u, tile);
            }
        }
    }

    /// Turns our shooters near `city` (each as if at a firing tile) and our aircraft in range
    /// need to take it to 0 HP at their expected damage; a city hit this turn does not heal.
    fn turns_to_zero(&self, city: &CityView, near: &[&UnitView]) -> Option<i32> {
        let per_turn = self.siege_damage(city, near);
        (per_turn > 0).then(|| (city.hp + per_turn - 1) / per_turn)
    }

    fn siege_damage(&self, city: &CityView, near: &[&UnitView]) -> i32 {
        let v = self.view();
        let mut av = self.attack_view().into_owned();
        // Each shooter as if beside the city: a firing tile it can take (line of sight is clear
        // at range 1), on the city's own defence.
        let beside = city.pos.neighbors()[0];
        let ids: Vec<UnitId> = near
            .iter()
            .filter(|u| u.ranged > 0 && u.range > 0)
            .map(|u| u.id)
            .collect();
        for u in av.units.iter_mut().filter(|u| ids.contains(&u.id)) {
            u.pos = beside;
        }
        let ground: i32 = ids
            .iter()
            .filter_map(|id| forecast_attack_mode(&av, *id, city.pos, AttackMode::Ranged))
            .map(|f| f.expected_damage_to_defender)
            .sum();
        let air: i32 = v
            .aircraft
            .iter()
            .filter(|a| a.owner == v.me)
            .filter(|a| !data::is_fighter(a.kind) || self.decisions.fighter_strike != 0)
            .filter(|a| {
                v.cities
                    .iter()
                    .find(|b| b.id == a.city)
                    .is_some_and(|b| b.pos.distance(city.pos) <= data::unit_stats(a.kind).range)
            })
            .filter_map(|a| forecast_attack_mode(&av, a.id, city.pos, AttackMode::Ranged))
            .map(|f| f.expected_damage_to_defender)
            .sum();
        ground + air
    }

    /// Gene `siege_stance` in the open-city seizure: whether the shooters `near` the city and
    /// our aircraft in range zero it within the gene's turns.
    pub(super) fn siege_within_reach(&self, city: &CityView, near: &[&UnitView]) -> bool {
        self.turns_to_zero(city, near)
            .is_some_and(|t| t <= self.decisions.siege_stance)
    }

    /// Gene `siege_stance`: a melee unit holds where `city`'s strike can't reach it but it can
    /// still reach the city next turn; failing that, two tiles out (the city shoots one unit).
    pub(super) fn siege_hold(&mut self, u: &UnitView, city: Hex) {
        let moves = u.max_moves.max(1);
        let spot = (2..=moves.max(2))
            .flat_map(|r| city.ring(r))
            .filter(|h| {
                self.free_tile_for(u, *h)
                    && (*h == u.pos || self.reaches_this_turn(u, *h))
                    && self.ctx.reachable(*h, city)
            })
            .min_by_key(|h| {
                (
                    self.city_damage_at(u, *h) > 0,
                    self.enemy_exposure_key(*h),
                    h.distance(city) > moves,
                    h.distance(u.pos),
                    *h,
                )
            });
        match spot {
            Some(to) if to == u.pos => self.hold_or_upgrade(u),
            Some(to) => self.go_to(u, to),
            None => self.strike(u, city, false),
        }
    }
}

/// A forecast shot's damage to the city's HP at every roll, as resolution computes it.
fn shot_rolls(f: &legatum_core::BattleForecast) -> Shot {
    let percent = f.walls.map_or(100, |w| w.percent);
    (combat::ROLL_MIN..=combat::ROLL_MAX)
        .map(|r| {
            combat::damage_to_city_hp(
                percent,
                combat::damage(f.attacker_strength.total, f.defender_strength.total, r),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder};

    const CITY: Hex = Hex::new(10, 8);

    #[test]
    fn zero_chance_is_exact_over_uniform_rolls() {
        let shot: Shot = (0..41).collect();
        assert_eq!(zero_chance(&[], 0), 100);
        assert_eq!(zero_chance(&[], 5), 0);
        // One shot of 0..=40: at least 30 in 11 of 41 rolls.
        assert_eq!(zero_chance(std::slice::from_ref(&shot), 30), 11 * 100 / 41);
        assert_eq!(zero_chance(&[shot.clone(), shot], 81), 0);
    }

    #[test]
    fn shooters_zero_an_open_city_and_the_cheapest_melee_walks_in() {
        let mut view = ViewBuilder::new(26, 18)
            .city(1, ME, Hex::new(3, 8), 2)
            .city(9, ENEMY, CITY, 3)
            .unit(1, ME, UnitKind::Soldier, Hex::new(9, 8))
            .unit(2, ME, UnitKind::Ranger, Hex::new(9, 7))
            .unit(3, ME, UnitKind::Ranger, Hex::new(10, 6))
            .unit(4, ME, UnitKind::Ranger, Hex::new(11, 7))
            .build();
        view.cities[1].hp = 30;
        let mut p = crate::fixture::planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        let army = p.ctx.own_units.clone();
        p.breach_cities(&army);
        assert!(p.orders.is_empty(), "gene off: {:?}", p.orders);
        let mut p = crate::fixture::planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.breach = 100;
        p.breach_cities(&army);
        assert!(
            p.orders.iter().any(|o| matches!(o,
                Order::MoveUnit { unit: UnitId(1), to } if *to == CITY)),
            "{:?}",
            p.orders
        );
        assert!(
            p.orders
                .iter()
                .any(|o| matches!(o, Order::RangedAttack { target, .. } if *target == CITY)),
            "{:?}",
            p.orders
        );
    }

    /// The owner reinforces its attacked city before our walker moves: a Rifleman beside the
    /// city would step in and beat the Soldier, so the breach is not worth it.
    #[test]
    fn no_breach_when_a_stronger_defender_can_step_into_the_city_first() {
        let mut view = ViewBuilder::new(26, 18)
            .city(1, ME, Hex::new(3, 8), 2)
            .city(9, ENEMY, CITY, 3)
            .unit(1, ME, UnitKind::Soldier, Hex::new(9, 8))
            .unit(2, ME, UnitKind::Ranger, Hex::new(9, 7))
            .unit(3, ME, UnitKind::Ranger, Hex::new(10, 6))
            .unit(4, ME, UnitKind::Ranger, Hex::new(11, 7))
            .unit(50, ENEMY, UnitKind::Rifleman, Hex::new(11, 8))
            .build();
        view.cities[1].hp = 30;
        let mut p = crate::fixture::planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.breach = 100;
        let army = p.ctx.own_units.clone();
        p.breach_cities(&army);
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { to, .. } if *to == CITY)),
            "{:?}",
            p.orders
        );
    }

    #[test]
    fn no_breach_when_the_shots_cannot_zero_a_full_walled_city() {
        let view = ViewBuilder::new(26, 18)
            .city(1, ME, Hex::new(3, 8), 2)
            .city(9, ENEMY, CITY, 3)
            .walled(9)
            .unit(1, ME, UnitKind::Soldier, Hex::new(9, 8))
            .unit(2, ME, UnitKind::Ranger, Hex::new(9, 7))
            .build();
        let mut p = crate::fixture::planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.breach = 100;
        let army = p.ctx.own_units.clone();
        p.breach_cities(&army);
        assert!(p.orders.is_empty(), "{:?}", p.orders);
    }
}

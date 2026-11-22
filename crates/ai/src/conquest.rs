//! A funded, local campaign for shipped Hard. All evidence comes from PlayerView.
//! Observed troop positions and city damage provide continuity without hidden state.
use crate::{analysis::threat_power, scripted::Planner};
use legatum_core::{Faction, Order, TeamId, UnitKind, path::find_path, view::CityView};

impl Planner<'_> {
    pub(crate) fn conquest_city(&self) -> Option<&CityView> {
        let id = *self
            .conquest_city
            .get_or_init(|| self.choose_conquest_city(self.decisions.stale_nominee > 0));
        self.view().cities.iter().find(|c| Some(c.id) == id)
    }

    /// Funding, staging and peace persistence all use this same nomination. Early Walls
    /// require a viable capture forecast unless the city is already visibly breached.
    /// Strong or unseen garrisons are costs in the city's score (`choose_conquest_city`).
    fn assessed_conquest_allowed(&self, city: &CityView) -> bool {
        if self.decisions.conquest_walled_forecast > 0
            && city.walls
            && city.hp > city.max_hp / 3
            && self.campaign_forecast(city, None).is_none()
        {
            return false;
        }
        if self.opponents.is_none() || !(crate::adaptation::early(self.view()) && city.walls) {
            return true;
        }
        // A forecasted capture can pay the Wall cost even before the siege era.
        if self.campaign_forecast(city, None).is_some() {
            return true;
        }
        if !city.visible {
            return false;
        }
        // The common campaign policy permits finishing a visibly weakened city.
        // Keep that exception local to a war already underway: it cannot restart a
        // distant invasion merely because an old city snapshot had low HP.
        if city.hp > city.max_hp / 3 || !self.view().at_war(self.view().me, city.owner) {
            return false;
        }
        let mut capture_units = 0;
        let mut local_units = 0;
        for unit in self.ctx.own_units.iter().filter(|unit| {
            unit.military
                && unit.kind != UnitKind::Scout
                && unit.hp >= 70
                && unit.pos.distance(city.pos) <= 5
                && !self.ctx.own_cities.iter().any(|home| home.pos == unit.pos)
        }) {
            local_units += 1;
            capture_units += usize::from(crate::composition::charges(unit.kind));
        }
        local_units >= 2 && capture_units >= 1
    }

    pub(crate) fn choose_conquest_city(&self, prefer_fresh: bool) -> Option<legatum_core::CityId> {
        if !self.hard_defense()
            || self.strategy.is_some()
            || self.decisions.conquest_enabled == 0
            || (self.ctx.own_cities.len() as i32) < self.conquest_min_cities()
        {
            return None;
        }
        let v = self.view();
        // Switzerland bank for their treasury goal before any optional war, except against a
        // runaway: the public VP leader past 45% of its target (The Courier King, user run
        // 2026-10-04: the human Switzerland retook Resonator from the runaway conqueror and
        // won; the AI banked and never declared in 50 of 50 runs).
        let runaway = v
            .standings
            .iter()
            .filter(|s| s.target > 0)
            .max_by_key(|s| (s.vp * 100 / s.target, std::cmp::Reverse(s.team)))
            .filter(|s| {
                s.team != v.team
                    && s.vp * 100 >= s.target * self.decisions.conquest_runaway_percent as u32
            })
            .map(|s| s.team);
        let banking = self.decisions.conquest_bank_goal > 0 && self.goal_pending();
        if banking && runaway.is_none() {
            return None;
        }
        // find_path reads only the map's geometry; costs come from the closure.
        let map = self.movement_map();
        let front = self.active_front();
        let strike_radius = self.decisions.lone_city_strike_radius;
        let nearby_open_byzantium = self.decisions.lone_city_strike > 0
            && self.ctx.own_cities.len() == 1
            && v.cities.iter().any(|city| {
                !city.owner.is_barbarian()
                    && !v.same_team(v.me, city.owner)
                    && v.faction_of(city.owner) == Faction::Byzantium
                    && !city.walls
                    && self.ctx.own_cities[0].pos.distance(city.pos) <= strike_radius
            });
        // Gene `value_first_target`: every reachable rival city competes on value; softness
        // comes from public information (playtest 8.9.2 Sparta: the faction filter below
        // ruled out Byzantium's Nicaea at 8 hexes and the Swiss, the two softest neighbours).
        let by_value = self.value_first_target();
        let mut best = None;
        for c in &v.cities {
            // Before its first settlement, Sparta can fund a local strike on the
            // India's exposed growth opening. Other neighbours need the ordinary
            // two-city economy before a conquest war is considered.
            if !by_value
                && self.decisions.lone_city_strike > 0
                && self.ctx.own_cities.len() == 1
                && if nearby_open_byzantium {
                    v.faction_of(c.owner) != Faction::Byzantium
                        || self.ctx.own_cities[0].pos.distance(c.pos) > strike_radius
                } else {
                    v.faction_of(c.owner) != Faction::India
                }
            {
                continue;
            }
            if self.decisions.early_walled_skip > 0
                && v.turn <= legatum_core::data::FIRST_THIRD_TURNS
                && c.walls
            {
                continue;
            }
            let legal_war = v.diplomacy.actions.iter().any(|a| {
                matches!(a.order, Order::DeclareWar { against, breach: false }
                    if against == v.team_of(c.owner))
                    && a.blocked_reason.is_none()
            });
            if v.same_team(v.me, c.owner)
                || !(v.at_war(v.me, c.owner) || legal_war)
                || !self.assessed_conquest_allowed(c)
            {
                continue;
            }
            if banking && runaway != Some(v.team_of(c.owner)) {
                continue;
            }
            // A war still being fought near home is the front; staging against a neutral
            // city would strip it (rush-defense, 2026-10-04: Mongols left a city it had
            // just taken to muster against a neutral neighbour, and lost it).
            if front && !v.at_war(v.me, c.owner) {
                continue;
            }
            let home = self
                .ctx
                .own_cities
                .iter()
                .min_by_key(|h| h.pos.distance(c.pos))?;
            if home.pos.distance(c.pos) > self.decisions.conquest_home_distance {
                continue;
            }
            // Only this prospective enemy's borders open; third parties and fog remain real.
            let path = find_path(map, home.pos, c.pos, |h| {
                let tile = v.tile(h)?;
                (tile.explored
                    && (v.passage_allowed(v.me, legatum_core::UnitKind::Soldier, tile.owner)
                        || tile.owner.is_some_and(|p| v.same_team(p, c.owner))))
                .then(|| tile.move_cost())
                .flatten()
            });
            let Some(path) = path.filter(|p| p.len() <= self.decisions.conquest_path_len as usize)
            else {
                continue;
            };
            let defenders: i32 = v
                .units
                .iter()
                .filter(|u| {
                    u.military && v.same_team(u.owner, c.owner) && u.pos.distance(c.pos) <= 3
                })
                .map(|u| threat_power(v, u))
                .sum();
            let nearby = self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 5)
                .count() as i32;
            let progress = v
                .standings
                .iter()
                .find(|s| s.team == v.team_of(c.owner))
                .map_or(0, |s| s.vp as i32 * 100 / s.target.max(1) as i32);
            let hubs = if v.faction_of(c.owner) == Faction::India {
                v.cities
                    .iter()
                    .filter(|h| h.owner == c.owner && h.id != c.id && h.pos.distance(c.pos) <= 5)
                    .count() as i32
            } else {
                0
            };
            let (softness, reinforcement) = if by_value {
                self.public_softness(c)
            } else {
                (0, 0)
            };
            let value = 100
                + softness
                // Gene `celts_playbook` part 4: no Walls, no garrison, a weaker army.
                + self.celtic_soft_target(c)
                + (progress - 40).max(0) * 3
                + c.capture_bounty.unwrap_or(0) / 4
                + crate::opportunity::sack_gold_value(
                    self.view().rule_experiments,
                    c,
                    self.faction,
                    self.ctx.me,
                )
                    / if self.faction == Faction::Celts && self.decisions.celt_sack_target > 0 {
                        2
                    } else {
                        4
                    }
                + self.hoard_denial_value(c)
                + self.greece_captive_tech_value(c)
                + 100 * (self.capital_capture_vp(c) + self.capital_race_vp(c))
                + self.swarm_weakness_value(c)
                + self.flower_wars_campaign_value(c)
                + hubs * 25
                + nearby * self.decisions.conquest_nearby_weight
                + if c.visible && c.hp < c.max_hp {
                    self.decisions.conquest_wounded_bonus
                } else {
                    0
                }
                + if v.at_war(v.me, c.owner) {
                    self.decisions.conquest_war_bonus
                } else {
                    0
                }
                // A team counting down to victory: the cities that take its VP.
                + 3 * crate::hold::denial_priority(v, c.owner, || 0)
                // Containing a conqueror: its newly taken cities are thinly held.
                + if self.top_threat_team() == Some(v.team_of(c.owner)) {
                    80 + if self.recently_captured_by_threat(c) { 80 } else { 0 }
                } else {
                    0
                };
            let celtic = self.faction == Faction::Celts && self.decisions.celt_sack_target > 0;
            let cost = c.strength * 3
                + defenders * if celtic { 6 } else { 3 }
                + reinforcement
                + c.hp / 4
                + path.len() as i32 * if celtic { 18 } else { 12 }
                + if celtic && c.walls { 250 } else { 0 }
                + if c.walls {
                    self.decisions.target_wall_cost
                } else {
                    0
                };
            // A neutral objective cannot displace the front we are legally fighting.
            let candidate = (
                prefer_fresh && self.stale_city(c),
                !v.at_war(v.me, c.owner),
                cost * 100 / value,
                c.id,
            );
            if best.is_none_or(|old| candidate < old) {
                best = Some(candidate);
            }
        }
        best.map(|(_, _, _, id)| id)
    }

    /// Gene `value_first_target` is on for this faction.
    /// Gene `roman_finish_siege`: Rome has a campaign under way. The siege's own XP promotes
    /// its units, and that first veteran must not close the one-city gate on the city they
    /// are about to take.
    fn roman_campaign_live(&self) -> bool {
        self.decisions.roman_finish_siege > 0
            && (self.campaigns.active.is_some()
                || matches!(
                    self.routed_plan().map(|p| &p.kind),
                    Some(crate::plan::PlanKind::Campaign { .. })
                ))
    }

    /// Own cities needed before a conquest city is nominated. Gene `roman_early_war`: Rome
    /// founds only with veterans, and its surest first promotions are a war; without a
    /// veteran in hand, its first city is enough (owner, 2026-11-21: the rule "forces you
    /// into early war").
    fn conquest_min_cities(&self) -> i32 {
        let needed = self.decisions.conquest_min_cities;
        if self.faction == Faction::Rome
            && self.decisions.roman_founders > 0
            && self.decisions.roman_early_war > 0
            && (!self.ctx.own_units.iter().any(|u| {
                u.level >= 1
                    && legatum_core::data::unit_class(u.kind)
                        == Some(legatum_core::UnitClass::Melee)
            }) || self.roman_campaign_live())
        {
            needed.min(1)
        } else {
            needed
        }
    }

    pub(crate) fn value_first_target(&self) -> bool {
        match self.decisions.value_first_target {
            1 => self.faction == Faction::Sparta,
            2 => true,
            _ => false,
        }
    }

    /// What makes a rival city soft, from public information only (gene `value_first_target`):
    /// the value of taking it beyond its yields, and the cost of the owner's visible field army
    /// that could relieve it. A last city ends a rival's whole future economy (winning is
    /// zero-sum); a seen-empty city or a levy-capped owner can't be reinforced in time; every
    /// other war the owner fights splits its defence.
    pub(crate) fn public_softness(&self, c: &CityView) -> (i32, i32) {
        let v = self.view();
        let owner = v.players.iter().find(|p| p.id == c.owner);
        let last_city = owner.is_some_and(|p| p.cities == 1);
        let levy_capped = owner.and_then(|p| p.levy).is_some_and(|l| l.units >= l.cap);
        let garrisoned = v
            .units
            .iter()
            .any(|u| u.military && u.pos == c.pos && v.same_team(u.owner, c.owner));
        let empty = c.visible && !garrisoned;
        let owner_team = v.team_of(c.owner);
        let other_wars = v
            .players
            .iter()
            .filter(|p| {
                p.alive
                    && p.team != owner_team
                    && p.team != v.team
                    && v.teams_at_war(owner_team, p.team)
            })
            .count()
            .min(3) as i32;
        let value = if last_city { 120 } else { 0 }
            + if levy_capped { 60 } else { 0 }
            + if empty { 60 } else { 0 }
            + other_wars * 30;
        let relief: i32 = v
            .units
            .iter()
            .filter(|u| {
                u.military
                    && v.same_team(u.owner, c.owner)
                    && (4..=8).contains(&u.pos.distance(c.pos))
            })
            .map(|u| threat_power(v, u))
            .sum();
        (value, relief)
    }

    /// At war with an empire whose troops, cities or land we have seen within six hexes of
    /// one of our cities: that war is an unresolved front.
    pub(crate) fn active_front(&self) -> bool {
        let v = self.view();
        let near_home =
            |h: legatum_core::Hex| self.ctx.own_cities.iter().any(|c| c.pos.distance(h) <= 6);
        self.ctx
            .enemy_units
            .iter()
            .any(|e| e.military && !e.owner.is_barbarian() && near_home(e.pos))
            || self
                .ctx
                .remembered_threats
                .iter()
                .any(|o| !o.owner.is_barbarian() && near_home(o.pos))
            || self
                .ctx
                .enemy_cities
                .iter()
                .any(|c| !c.owner.is_barbarian() && near_home(c.pos))
            || self.ctx.own_cities.iter().any(|home| {
                home.pos.within(6).into_iter().any(|h| {
                    v.tile(h).is_some_and(|t| {
                        t.explored
                            && t.owner
                                .is_some_and(|o| !o.is_barbarian() && v.at_war(v.me, o))
                    })
                })
            })
    }

    pub(crate) fn conquest_ready(&self, against: TeamId) -> bool {
        let Some(c) = self
            .conquest_city()
            .filter(|c| self.view().team_of(c.owner) == against)
        else {
            return false;
        };
        let force: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && u.hp >= 70
                    && u.pos.distance(c.pos) <= 5
                    && !self.ctx.own_cities.iter().any(|h| h.pos == u.pos)
            })
            .collect();
        // Only firing cities require a ranged siege wing. Keep the two healthy,
        // local capture units and the home-garrison exclusion for either target;
        // available_campaign and tactical forecasts still check winning strength.
        force
            .iter()
            .filter(|u| crate::composition::charges(u.kind))
            .count()
            >= 2
            && (self.sparta_walled_prize(c) && self.campaign_outlook(c).is_some()
                || !legatum_core::city_fires(c)
                || force
                    .iter()
                    .filter(|u| crate::composition::shooter_only(u.kind, u.ranged))
                    .count()
                    >= 2)
    }

    pub(crate) fn persist_conquest(&self, against: TeamId) -> bool {
        let Some(city) = self
            .conquest_city()
            .filter(|c| self.view().team_of(c.owner) == against)
        else {
            return false;
        };
        let v = self.view();
        let age = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, against))
            .map_or(0, |r| v.turn.saturating_sub(r.effective_turn));
        let near: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && u.hp >= 40
                    && u.pos.distance(city.pos) <= 5
            })
            .collect();
        age < 32
            || (city.visible
                && city.hp < city.max_hp
                && near.len() >= 3
                && near.iter().any(|u| crate::composition::charges(u.kind)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::{AiDifficulty, CityId, PlayerView};

    fn hard(v: &PlayerView) -> Planner<'_> {
        let mut p = crate::fixture::planner(v, 7);
        if v.faction_of(ME) == Faction::Sparta {
            // These tests cover Sparta's one-city opening and its caution around early Walls.
            p.knobs = crate::Knobs::new(40, 80, 35, 40, 85);
            p.personality_aggression = 85;
            p.decisions.conquest_min_cities = 0;
            p.decisions.early_walled_skip = 1;
        }
        p.difficulty = AiDifficulty::Hard;
        p
    }

    fn cities() -> ViewBuilder {
        ViewBuilder::new(32, 20)
            .city(1, ME, off(3, 6), 3)
            .city(2, ME, off(3, 12), 3)
            .city(3, ENEMY, off(11, 6), 3)
            .city(4, ENEMY, off(11, 12), 3)
    }

    #[test]
    fn stale_best_target_yields_to_fresh_city_and_gets_a_scout() {
        let mut v = cities()
            .unit(10, ME, UnitKind::Scout, off(6, 6))
            .unit(11, ME, UnitKind::Soldier, off(5, 7))
            .build();
        v.turn = 61;
        let stale = v.cities.iter_mut().find(|c| c.id == CityId(3)).unwrap();
        stale.visible = false;
        stale.capture_bounty = Some(2000);
        let mut p = hard(&v);
        p.campaigns.city_seen.insert(CityId(3), 44);
        assert_eq!(p.choose_conquest_city(false), Some(CityId(3)));
        assert_eq!(p.conquest_city().map(|c| c.id), Some(CityId(4)));
        p.refresh_stale_nominee();
        let to = p.orders.iter().find_map(|o| match o {
            Order::MoveUnit { unit, to } if *unit == legatum_core::UnitId(10) => Some(*to),
            _ => None,
        });
        assert!(to.is_some_and(|h| h.distance(off(11, 6)) <= legatum_core::data::UNIT_SIGHT));

        let mut old = hard(&v);
        old.campaigns.city_seen.insert(CityId(3), 44);
        old.decisions.stale_nominee = 0;
        assert_eq!(old.conquest_city().map(|c| c.id), Some(CityId(3)));
        old.refresh_stale_nominee();
        assert!(old.orders.is_empty());
    }

    #[test]
    fn stale_target_uses_fast_idle_army_when_no_scout_can_go() {
        let mut v = cities()
            .unit(10, ME, UnitKind::Soldier, off(6, 6))
            .unit(11, ME, UnitKind::Horseman, off(5, 7))
            .build();
        v.turn = 61;
        let stale = v.cities.iter_mut().find(|c| c.id == CityId(3)).unwrap();
        stale.visible = false;
        stale.capture_bounty = Some(2000);
        let mut p = hard(&v);
        p.campaigns.city_seen.insert(CityId(3), 44);
        p.refresh_stale_nominee();
        assert!(p.orders.iter().any(|o| matches!(o,
            Order::MoveUnit { unit, to }
                if *unit == legatum_core::UnitId(11)
                    && to.distance(off(11, 6)) <= legatum_core::data::UNIT_SIGHT
        )));
    }

    #[test]
    fn germany_can_name_a_conquest_target_and_the_gene_can_disable_it() {
        let mut v = cities()
            .faction(Faction::Germany)
            .city(5, ME, off(3, 16), 3)
            .build();
        v.diplomacy.legacy = false;
        let rival = v.team_of(ENEMY);
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: rival,
                state: legatum_core::diplomacy::RelationState::War,
                effective_turn: 1,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![v.team],
            });
        let mut p = hard(&v);
        p.decisions = crate::DecisionParams::builtin_for(Faction::Germany);
        assert_eq!(p.decisions.conquest_enabled, 1);
        assert!(
            p.conquest_city().is_some(),
            "Germany can name a reachable rival city"
        );
        let mut disabled = hard(&v);
        disabled.decisions = crate::DecisionParams::builtin_for(Faction::Germany);
        disabled.decisions.conquest_enabled = 0;
        assert!(disabled.conquest_city().is_none());
    }

    #[test]
    fn conquest_values_a_celtic_sack_at_the_forecast_gold() {
        let v = cities().build();
        let city = v.cities[2].clone();
        // 3 population x 60: the amount a sack forecast shows, nothing estimated.
        assert_eq!(
            crate::opportunity::sack_gold_value(v.rule_experiments, &city, Faction::Celts, ME),
            3 * 60
        );
        assert_eq!(
            crate::opportunity::sack_gold_value(v.rule_experiments, &city, Faction::Sparta, ME),
            0
        );
    }

    #[test]
    fn conquest_values_retaking_our_own_founded_city_as_a_kept_city_not_a_sack() {
        let v = cities().build();
        let mut city = v.cities[2].clone();
        city.founder = Some(ME);
        assert_eq!(
            crate::opportunity::sack_gold_value(v.rule_experiments, &city, Faction::Celts, ME),
            0
        );
    }

    #[test]
    fn weak_city_and_public_bounty_change_the_objective() {
        let mut v = cities().build();
        // Compare ordinary cities so the new original-capital point does not swamp this test.
        v.cities[2].original_capital = false;
        v.cities[2].original_capital_of = None;
        v.cities[2].strength = 50;
        v.cities[3].strength = 15;
        assert_eq!(hard(&v).conquest_city().map(|c| c.id), Some(CityId(4)));
        v.cities[2].capture_bounty = Some(2000);
        assert_eq!(hard(&v).conquest_city().map(|c| c.id), Some(CityId(3)));
        let mut normal = hard(&v);
        normal.difficulty = AiDifficulty::Normal;
        assert!(normal.conquest_city().is_none());
    }

    #[test]
    fn a_one_city_sparta_picks_the_softest_rival_by_value_not_faction() {
        use legatum_core::view::LevyView;
        const THIRD: legatum_core::PlayerId = legatum_core::PlayerId(3);
        let mut v = ViewBuilder::new(32, 24)
            .faction(Faction::Sparta)
            .city(1, ME, off(3, 6), 3)
            .city(3, ENEMY, off(9, 6), 3)
            .city(5, ENEMY, off(15, 6), 3)
            .city(4, THIRD, off(3, 14), 2)
            .build();
        for p in v.players.iter_mut() {
            p.cities = v.cities.iter().filter(|c| c.owner == p.id).count() as u32;
            match p.id {
                ENEMY => p.faction = Faction::India,
                THIRD => {
                    p.faction = Faction::Byzantium;
                    p.levy = Some(LevyView { units: 2, cap: 2 });
                }
                _ => {}
            }
        }
        // The faction filter: a one-city Sparta may only take the India city.
        let mut off = hard(&v);
        off.decisions.value_first_target = 0;
        assert_eq!(off.conquest_city().map(|c| c.id), Some(CityId(3)));
        // By value: Byzantium's last city, levy-capped, two hexes farther, is the softer prize.
        let mut p = hard(&v);
        p.decisions.value_first_target = 1;
        assert_eq!(p.conquest_city().map(|c| c.id), Some(CityId(4)));
        // Gene 1 is Sparta only.
        let mut v2 = v.clone();
        v2.players[0].faction = Faction::Switzerland;
        let mut other = hard(&v2);
        other.decisions.value_first_target = 1;
        assert!(!other.value_first_target());
    }

    #[test]
    fn a_bounty_that_breaks_a_held_hoard_adds_vp_denial_value() {
        let mut v = cities().build();
        v.cities[2].original_capital = false;
        v.cities[2].original_capital_of = None;
        v.cities[2].strength = 50;
        v.cities[3].strength = 15;
        // A small bounty on the stronger city: the weaker one is still the objective.
        v.cities[2].capture_bounty = Some(20);
        assert_eq!(hard(&v).conquest_city().map(|c| c.id), Some(CityId(4)));
        // The owner now scores the live hoard, and the bounty drops its treasury below it.
        v.players
            .iter_mut()
            .find(|p| p.id == ENEMY)
            .unwrap()
            .vp_sources = vec![(legatum_core::VpSource::FactionGoal { of: ENEMY }, 2)];
        assert!(hard(&v).hoard_denial_value(&v.cities[2]) > 0);
        assert_eq!(hard(&v).conquest_city().map(|c| c.id), Some(CityId(3)));
        // Not a hoard holder, no denial.
        v.players
            .iter_mut()
            .find(|p| p.id == ENEMY)
            .unwrap()
            .vp_sources
            .clear();
        assert_eq!(hard(&v).hoard_denial_value(&v.cities[2]), 0);
        // A huge treasury stays above the hoard after the bounty: nothing to deny.
        v.players
            .iter_mut()
            .find(|p| p.id == ENEMY)
            .unwrap()
            .vp_sources = vec![(legatum_core::VpSource::FactionGoal { of: ENEMY }, 2)];
        v.cities[2].capture_bounty = Some(5000);
        v.cities[2].population = 1;
        assert_eq!(hard(&v).hoard_denial_value(&v.cities[2]), 0);
    }

    #[test]
    fn firing_city_needs_healthy_local_melee_and_ranged_not_global_strength() {
        let v = cities()
            .walled(3)
            .walled(4)
            .unit(1, ME, UnitKind::Soldier, off(9, 6))
            .unit(2, ME, UnitKind::Soldier, off(8, 7))
            .unit(3, ME, UnitKind::Ranger, off(8, 5))
            .unit(4, ME, UnitKind::Ranger, off(7, 6))
            .build();
        assert!(hard(&v).conquest_ready(TeamId(2)));
        let mut remote = v.clone();
        remote.units[3].pos = off(1, 18);
        assert!(!hard(&remote).conquest_ready(TeamId(2)));
        remote.units[3].pos = v.units[3].pos;
        remote.units[3].hp = 30;
        assert!(!hard(&remote).conquest_ready(TeamId(2)));
    }

    #[test]
    fn unwalled_infantry_campaign_is_ready_but_firing_walls_need_shooters() {
        let v = cities()
            .unit(1, ME, UnitKind::Soldier, off(9, 6))
            .unit(2, ME, UnitKind::Soldier, off(8, 7))
            .build();
        assert!(hard(&v).conquest_ready(TeamId(2)));

        let walled = ViewBuilder { view: v.clone() }.walled(3).walled(4).build();
        assert!(!hard(&walled).conquest_ready(TeamId(2)));
        let supported = ViewBuilder { view: walled }
            .unit(3, ME, UnitKind::Ranger, off(8, 5))
            .unit(4, ME, UnitKind::Ranger, off(7, 6))
            .build();
        assert!(hard(&supported).conquest_ready(TeamId(2)));

        let mut unhealthy = v.clone();
        unhealthy.units[1].hp = 30;
        assert!(!hard(&unhealthy).conquest_ready(TeamId(2)));
        let mut remote = v.clone();
        remote.units[1].pos = off(1, 18);
        assert!(!hard(&remote).conquest_ready(TeamId(2)));

        // A nearby home guard is not one of the two capture units, even when its
        // position is within the campaign's five-hex staging radius.
        let mut guarded = ViewBuilder { view: v }.city(5, ME, off(7, 6), 3).build();
        guarded.units[1].pos = off(7, 6);
        assert!(!hard(&guarded).conquest_ready(TeamId(2)));
    }

    fn assessed_hard<'a>(
        view: &'a PlayerView,
        model: &'a crate::opponents::OpponentAssessments,
    ) -> Planner<'a> {
        let mut planner = hard(view);
        assert!(
            planner.conquest_city.get().is_none(),
            "selection must remain lazy until the model is attached"
        );
        planner.opponents = Some(model);
        planner
    }

    #[test]
    fn unsuitable_assessed_targets_disable_conquest_funding_and_persistence() {
        let view = ViewBuilder {
            view: cities().build(),
        }
        .walled(3)
        .walled(4)
        .build();
        let mut model = crate::opponents::OpponentAssessments::default();
        model.update(&view);
        assert!(!model.has_viable_target(crate::adaptation::early(&view)));
        assert!(
            hard(&view).conquest_city().is_some(),
            "no-model prerequisite behavior is unchanged"
        );
        let planner = assessed_hard(&view, &model);
        // This shared nomination is the production quota and staging gate too.
        assert!(planner.conquest_city().is_none());
        assert!(!planner.conquest_ready(TeamId(2)));
        assert!(!planner.persist_conquest(TeamId(2)));
    }

    #[test]
    fn finishing_a_damaged_siege_requires_a_visible_target_and_local_capture_force() {
        {
            let mut view = cities()
                .walled(3)
                .walled(4)
                .unit(1, ME, UnitKind::Soldier, off(9, 6))
                .unit(2, ME, UnitKind::Ranger, off(8, 7))
                .build();
            view.cities
                .iter_mut()
                .find(|c| c.id == CityId(3))
                .unwrap()
                .hp = 50;
            let mut model = crate::opponents::OpponentAssessments::default();
            model.update(&view);
            assert!(!model.has_viable_target(crate::adaptation::early(&view)));
            assert_eq!(
                assessed_hard(&view, &model).conquest_city().map(|c| c.id),
                Some(CityId(3))
            );
            view.units[0].hp = 30;
            assert!(assessed_hard(&view, &model).conquest_city().is_none());
            view.units[0].hp = 100;
            view.cities
                .iter_mut()
                .find(|c| c.id == CityId(3))
                .unwrap()
                .visible = false;
            assert!(assessed_hard(&view, &model).conquest_city().is_none());
        }
    }

    #[test]
    fn unknown_terrain_cannot_be_a_campaign_route() {
        let mut v = cities().build();
        for row in 0..v.height {
            let i = v.index(off(7, row)).unwrap();
            v.tiles[i].explored = false;
            v.tiles[i].visible = false;
            v.tiles[i].terrain = None;
        }
        assert!(hard(&v).conquest_city().is_none());
    }

    #[test]
    fn sparta_opening_skips_walls_even_in_funded_conquest() {
        let mut v = cities()
            .faction(Faction::Sparta)
            .walled(3)
            .walled(4)
            .build();
        assert!(hard(&v).conquest_city().is_none());
        v.turn = legatum_core::data::FIRST_THIRD_TURNS + 1;
        assert!(hard(&v).conquest_city().is_some());
    }

    /// Gene `roman_early_war`: Rome with one city and no veteran names a conquest city (its
    /// second city needs a veteran, and a war is the surest first promotion); with the gene
    /// off, or a veteran in hand and no campaign under way, it keeps the shared two-city gate.
    /// Gene `roman_finish_siege`: a campaign's own promotions do not close the gate.
    #[test]
    fn a_one_city_rome_without_a_veteran_names_a_conquest_city() {
        let mut v = ViewBuilder::new(32, 20)
            .faction(Faction::Rome)
            .city(1, ME, off(3, 6), 3)
            .city(3, ENEMY, off(11, 6), 3)
            .unit(11, ME, UnitKind::Soldier, off(4, 6))
            .unit(12, ME, UnitKind::Soldier, off(3, 7))
            .build();
        v.diplomacy.legacy = false;
        let rival = v.team_of(ENEMY);
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: rival,
                state: legatum_core::diplomacy::RelationState::War,
                effective_turn: 1,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![v.team],
            });
        let rome = |v: &PlayerView, early_war: i32, campaign: bool| {
            let mut p = hard(v);
            p.decisions = crate::DecisionParams::builtin_for(Faction::Rome);
            p.decisions.conquest_min_cities = 2;
            p.decisions.roman_founders = 1;
            p.decisions.roman_early_war = early_war;
            p.decisions.roman_finish_siege = 1;
            if campaign {
                p.campaigns.active = Some(crate::campaign::Campaign {
                    target: CityId(3),
                    start_turn: 1,
                    committed: vec![legatum_core::UnitId(11), legatum_core::UnitId(12)],
                    deadline: None,
                    expected_value: 100,
                    target_was_unwalled: true,
                });
            }
            p.conquest_city().map(|c| c.id)
        };
        assert_eq!(rome(&v, 1, false), Some(CityId(3)));
        assert_eq!(rome(&v, 0, false), None);
        v.units.iter_mut().find(|u| u.id.0 == 11).unwrap().level = 1;
        // With no campaign under way, a veteran in hand founds the second city first.
        assert_eq!(rome(&v, 1, false), None);
        // Gene `roman_finish_siege`: a promotion earned mid-campaign keeps the target named
        // (the owner's Laurion siege: the first veteran closed the gate on a falling city).
        assert_eq!(rome(&v, 1, true), Some(CityId(3)));
    }
}

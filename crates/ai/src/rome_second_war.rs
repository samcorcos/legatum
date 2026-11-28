//! Hard Rome's next open-city war after its first victim.
//! The choice and the assault use only the acting seat's player view.
use crate::plan::{Plan, PlanKind};
use crate::scripted::Planner;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{AiDifficulty, Faction, Order, TeamId, UnitClass, data};

const FIRST_TURN: u32 = 51;
const LAST_TURN: u32 = 100;
const FIT_HP: i32 = 50;
const NEIGHBOUR_RANGE: i32 = 10;
const SOFT_DEFENDERS: usize = 1;
const HIT_HP_AFTER: i32 = 20;
const HIT_MAX_LOSS: u32 = 10;
const STORM_MAX_LOSS: u32 = 25;
const STORM_MIN_DAMAGE: i32 = 40;

impl Planner<'_> {
    pub(crate) fn rome_second_on(&self) -> bool {
        self.conquest_opening_faction()
            && self.difficulty == AiDifficulty::Hard
            && self.decisions.rome_second_war > 0
            && self.view().turn <= LAST_TURN
            && !(self.faction == Faction::Sparta
                && self.decisions.sparta_finish_60 > 0
                && self.rome_opening_active())
            && (self.view().turn >= FIRST_TURN
                || (self.faction == Faction::Sparta
                    && self.decisions.sparta_second_war_from > 0
                    && self.view().turn >= self.decisions.sparta_second_war_from as u32
                    && !self.rome_opening_active())
                || (self.decisions.rome_walled_peace > 0
                    && self.campaigns.rome_walled_peace_window))
    }

    fn rome_second_fit(&self, unit: &UnitView) -> bool {
        unit.owner == self.view().me
            && data::unit_class(unit.kind) == Some(UnitClass::Melee)
            && unit.hp >= FIT_HP
            && unit.occupation_lock.is_none()
            && self.roman_unit_can_leave(unit)
    }

    fn rome_second_reaches(&self, city: &CityView) -> bool {
        let at_war = self.view().at_war(self.view().me, city.owner);
        self.ctx.own_units.iter().any(|unit| {
            unit.owner == self.view().me
                && data::unit_class(unit.kind) == Some(UnitClass::Melee)
                && unit.occupation_lock.is_none()
                && self.roman_unit_can_leave(unit)
                && (self.ctx.reachable(unit.pos, city.pos)
                    || city
                        .pos
                        .ring(if at_war { 1 } else { 3 })
                        .iter()
                        .any(|tile| {
                            self.ctx.reachable(unit.pos, *tile)
                                && self.view().tile(*tile).is_some_and(|t| {
                                    t.terrain.is_some()
                                        && self.view().passage_allowed(
                                            self.view().me,
                                            unit.kind,
                                            t.owner,
                                        )
                                })
                        }))
        })
    }

    fn rome_second_visible_defenders(&self, city: &CityView) -> usize {
        self.view()
            .units
            .iter()
            .filter(|unit| {
                unit.owner == city.owner && unit.military && unit.pos.distance(city.pos) <= 4
            })
            .count()
    }

    fn rome_second_soft(&self, city: &CityView) -> bool {
        let v = self.view();
        let at_war = v.at_war(v.me, city.owner);
        let fresh_neighbour = self
            .ctx
            .own_units
            .iter()
            .filter(|unit| self.rome_second_fit(unit))
            .any(|unit| unit.pos.distance(city.pos) <= NEIGHBOUR_RANGE)
            && self.rome_second_visible_defenders(city) <= SOFT_DEFENDERS;
        !city.owner.is_barbarian()
            && !v.same_team(v.me, city.owner)
            && !city.walls
            && (at_war
                || (fresh_neighbour
                    && crate::opportunity::legal_declaration(v, v.team_of(city.owner))))
            && self.rome_second_reaches(city)
    }

    fn rome_second_score(&self, city: &CityView) -> (bool, bool, i32, legatum_core::CityId) {
        let v = self.view();
        let faction = v.faction_of(city.owner);
        let visible_military = self.rome_second_visible_defenders(city) as i32;
        let known_cities = v
            .cities
            .iter()
            .filter(|other| other.owner == city.owner)
            .count() as i32;
        let distance = self
            .ctx
            .own_units
            .iter()
            .filter(|unit| self.rome_second_fit(unit))
            .map(|unit| unit.pos.distance(city.pos))
            .min()
            .unwrap_or(i32::MAX / 4);
        let preferred = matches!(
            faction,
            Faction::Athens | Faction::India | Faction::Switzerland
        );
        (
            faction == Faction::Sparta,
            faction == Faction::Mongols,
            distance + 4 * visible_military + 2 * known_cities + 5 * i32::from(city.walls_building)
                - 3 * i32::from(preferred),
            city.id,
        )
    }

    pub(crate) fn choose_rome_second_war(&mut self) {
        if let Some(victim) = self.campaigns.rome_victim
            && self.rome_walled_peace(self.view().team_of(victim))
        {
            self.campaigns.rome_walled_peace_window = true;
        }
        if !self.rome_second_on() || self.takes_no_cities() {
            return;
        }
        let v = self.view();
        let fit = self
            .ctx
            .own_units
            .iter()
            .filter(|unit| self.rome_second_fit(unit))
            .count();
        let soft: Vec<_> = v
            .cities
            .iter()
            .filter(|city| self.rome_second_soft(city))
            .collect();
        // Finish a reachable open city of the opening victim before choosing a new war.
        // While the opening is still active it owns that siege; once the opening window ends
        // (turn 51+), hand the living victim's open city straight to the second-war pass
        // instead of dropping it.
        let unfinished_victim = self.campaigns.rome_victim.and_then(|victim| {
            v.players
                .iter()
                .any(|p| p.id == victim && p.alive)
                .then(|| {
                    soft.iter()
                        .copied()
                        .filter(|city| city.owner == victim)
                        .min_by_key(|city| self.rome_second_score(city))
                })
                .flatten()
        });
        if unfinished_victim.is_some() && self.rome_opening_active() {
            self.campaigns.rome_second_target = None;
            self.campaigns.rome_second_enemy = None;
            return;
        }
        if fit < 4 && self.campaigns.rome_second_enemy.is_none() && unfinished_victim.is_none() {
            return;
        }
        let prior_enemy = self.campaigns.rome_second_enemy;
        let committed = unfinished_victim.or_else(|| {
            soft.iter()
                .copied()
                .filter(|city| prior_enemy == Some(v.team_of(city.owner)))
                .min_by_key(|city| self.rome_second_score(city))
        });
        let chosen = if fit >= 4 {
            committed
                .or_else(|| {
                    soft.iter()
                        .copied()
                        .filter(|city| v.at_war(v.me, city.owner))
                        .min_by_key(|city| self.rome_second_score(city))
                })
                .or_else(|| {
                    soft.iter()
                        .copied()
                        .min_by_key(|city| self.rome_second_score(city))
                })
        } else {
            committed
        };
        let Some(city) = chosen else {
            self.campaigns.rome_second_target = None;
            self.campaigns.rome_second_enemy = None;
            return;
        };
        let enemy = v.team_of(city.owner);
        let kind = PlanKind::Campaign {
            against: enemy,
            target: city.id,
        };
        let start = self
            .campaigns
            .plan
            .active
            .as_ref()
            .filter(|plan| plan.kind == kind)
            .map_or(v.turn, |plan| plan.start_turn);
        self.campaigns.rome_second_target = Some(city.id);
        self.campaigns.rome_second_enemy = Some(enemy);
        if self
            .campaigns
            .active
            .as_ref()
            .is_some_and(|active| active.target != city.id)
        {
            self.campaigns.active = None;
        }
        self.campaign_withdraw.clear();
        self.campaigns.plan.active = Some(Plan {
            kind,
            start_turn: start,
            review_turn: v.turn + 1,
            reserve: 0,
            value: 0,
            terms: vec![("Roman second war".into(), 0)],
            breach: false,
            misses: 0,
        });
    }

    fn rome_second_city(&self) -> Option<&CityView> {
        let target = self.campaigns.rome_second_target?;
        self.view().cities.iter().find(|city| {
            city.id == target
                && !city.walls
                && Some(self.view().team_of(city.owner)) == self.campaigns.rome_second_enemy
        })
    }

    pub(crate) fn rome_second_declaration(&self) -> Option<Order> {
        let city = self.rome_second_city()?;
        let v = self.view();
        let against = v.team_of(city.owner);
        if v.teams_at_war(v.team, against) {
            return None;
        }
        if self.faction == Faction::Rome {
            let staged = self
                .ctx
                .own_units
                .iter()
                .filter(|unit| self.rome_second_fit(unit) && unit.pos.distance(city.pos) <= 4)
                .count();
            if staged < 2 {
                return None;
            }
        }
        v.diplomacy
            .actions
            .iter()
            .find(|action| {
                action.blocked_reason.is_none()
                    && action.order
                        == Order::DeclareWar {
                            against,
                            breach: false,
                        }
            })
            .map(|action| action.order.clone())
    }

    pub(crate) fn rome_second_refuses_peace(&self, against: TeamId) -> bool {
        self.rome_second_on()
            && self.campaigns.rome_second_enemy == Some(against)
            && self
                .rome_second_city()
                .is_some_and(|city| self.rome_second_reaches(city))
    }

    pub(crate) fn rome_second_assault(&mut self, army: &[&UnitView]) {
        if !self.rome_second_on() {
            return;
        }
        let Some(city) = self.rome_second_city() else {
            return;
        };
        let (pos, hp, owner, visible) = (city.pos, city.hp, city.owner, city.visible);
        if !visible || !self.view().at_war(self.view().me, owner) {
            return;
        }
        let mut wing: Vec<_> = self
            .free(army)
            .into_iter()
            .filter(|unit| {
                data::unit_class(unit.kind) == Some(UnitClass::Melee)
                    && unit.occupation_lock.is_none()
                    && self.roman_unit_can_leave(unit)
                    && !self.last_garrison_under_attack(unit)
                    && unit.pos.distance(pos) <= unit.max_moves.max(1)
                    && self.reaches_this_turn(unit, pos)
            })
            .filter_map(|unit| {
                let forecast = self.melee_forecast(unit, pos)?;
                let fit = if hp > 0 {
                    forecast.loss_chance <= HIT_MAX_LOSS
                        && unit.hp - forecast.expected_damage_to_attacker >= HIT_HP_AFTER
                } else {
                    forecast.loss_chance <= STORM_MAX_LOSS
                };
                fit.then_some((
                    unit.id,
                    forecast.win_chance,
                    forecast.loss_chance,
                    forecast.expected_damage_to_defender,
                ))
            })
            .collect();
        let garrisoned = self
            .view()
            .units
            .iter()
            .any(|unit| unit.pos == pos && unit.military && unit.owner == owner);
        if hp == 0
            && garrisoned
            && wing.iter().map(|(_, _, _, damage)| damage).sum::<i32>() < STORM_MIN_DAMAGE
            && !wing.iter().any(|(_, win, _, _)| *win >= 50)
        {
            return;
        }
        wing.sort_by_key(|(id, win, loss, _)| (std::cmp::Reverse(*win), *loss, *id));
        for (id, _, _, _) in wing {
            self.unit_order(id, Order::MoveUnit { unit: id, to: pos });
        }
    }

    pub(crate) fn march_rome_second_war(&mut self, army: &[&UnitView]) {
        if !self.rome_second_on() {
            return;
        }
        let Some(city) = self.rome_second_city() else {
            return;
        };
        let target = city.pos;
        let radius = if self.view().at_war(self.view().me, city.owner) {
            1
        } else {
            3
        };
        for unit in self.free(army) {
            if !self.rome_second_fit(unit) || self.last_garrison_under_attack(unit) {
                continue;
            }
            if unit.pos.distance(target) <= radius {
                continue;
            }
            let spot = target
                .ring(radius)
                .into_iter()
                .filter(|tile| {
                    self.ctx.reachable(unit.pos, *tile)
                        && self.free_tile_for(unit, *tile)
                        && self.view().tile(*tile).is_some_and(|t| {
                            t.terrain.is_some()
                                && self
                                    .view()
                                    .passage_allowed(self.view().me, unit.kind, t.owner)
                        })
                })
                .min_by_key(|tile| (unit.pos.distance(*tile), *tile));
            if let Some(spot) = spot {
                self.go_to(unit, spot);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off, planner};
    use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
    use legatum_core::{CityId, PlayerId, PlayerView, TeamId, UnitKind};

    fn view() -> PlayerView {
        let mut v = ViewBuilder::new(30, 20)
            .faction(Faction::Rome)
            .city(1, ME, off(5, 8), 3)
            .city(2, PlayerId(2), off(12, 8), 3)
            .city(3, PlayerId(3), off(10, 8), 3)
            .unit(1, ME, UnitKind::Spearman, off(7, 7))
            .unit(2, ME, UnitKind::Spearman, off(7, 8))
            .unit(3, ME, UnitKind::Spearman, off(7, 9))
            .unit(4, ME, UnitKind::Spearman, off(6, 8))
            .build();
        v.turn = 60;
        v.players[2].faction = Faction::Switzerland;
        v.players[3].faction = Faction::Sparta;
        v.diplomacy.legacy = false;
        for team in [TeamId(2), TeamId(3)] {
            v.diplomacy.actions.push(DiplomaticAction {
                new_wars: vec![],
                calls: vec![],
                also_agree: vec![],
                order: Order::DeclareWar {
                    against: team,
                    breach: false,
                },
                effective_turn: v.turn,
                cost: 0,
                blocked_reason: None,
                upkeep: 0,
                betrayal: false,
            });
        }
        v
    }

    #[test]
    fn second_war_uses_four_fit_melee_and_defers_sparta() {
        let mut v = view();
        let mut p = planner(&v, 9);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.rome_second_war = 1;
        p.choose_rome_second_war();
        assert_eq!(p.campaigns.rome_second_target, Some(CityId(2)));
        // Units are 5 hexes away: Rome stages before declaring war.
        assert_eq!(p.rome_second_declaration(), None);
        for id in [1, 2] {
            v.units.iter_mut().find(|u| u.id.0 == id).unwrap().pos = off(9, 8);
        }
        let mut p = planner(&v, 9);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.rome_second_war = 1;
        p.choose_rome_second_war();
        assert_eq!(p.campaigns.rome_second_target, Some(CityId(2)));
        assert_eq!(
            p.rome_second_declaration(),
            Some(Order::DeclareWar {
                against: TeamId(2),
                breach: false,
            })
        );
        let mut v = view();
        v.units.retain(|unit| unit.id.0 != 4);
        let mut p = planner(&v, 9);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.rome_second_war = 1;
        p.choose_rome_second_war();
        assert_eq!(p.campaigns.rome_second_target, None);
    }

    #[test]
    fn a_fresh_war_requires_a_lightly_defended_city() {
        let mut v = view();
        for (id, pos) in [(5, off(11, 8)), (6, off(12, 9))] {
            let mut unit = v.units[0].clone();
            unit.id = legatum_core::UnitId(id);
            unit.owner = PlayerId(2);
            unit.pos = pos;
            v.units.push(unit);
        }
        let mut p = planner(&v, 9);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.rome_second_war = 1;
        p.choose_rome_second_war();
        assert_eq!(p.campaigns.rome_second_target, Some(CityId(3)));
    }

    #[test]
    fn an_open_city_of_the_second_enemy_blocks_peace() {
        let mut v = view();
        v.diplomacy.relations.push(Relation {
            a: TeamId(0),
            b: TeamId(2),
            state: RelationState::War,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: vec![],
        });
        let mut p = planner(&v, 9);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.rome_second_war = 1;
        p.choose_rome_second_war();
        assert!(p.rome_second_refuses_peace(TeamId(2)));
        assert!(!p.rome_second_refuses_peace(TeamId(3)));
        let memory = p.campaigns.clone();
        v.units.retain(|unit| unit.id.0 != 4);
        let mut continued = planner(&v, 9);
        continued.difficulty = AiDifficulty::Hard;
        continued.decisions.rome_second_war = 1;
        continued.campaigns = memory;
        continued.choose_rome_second_war();
        assert!(continued.rome_second_refuses_peace(TeamId(2)));
        v.cities
            .iter_mut()
            .find(|c| c.id == CityId(2))
            .unwrap()
            .walls = true;
        let mut p = planner(&v, 9);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.rome_second_war = 1;
        p.choose_rome_second_war();
        assert!(!p.rome_second_refuses_peace(TeamId(2)));
    }

    #[test]
    fn turn_51_hands_unfinished_opening_victim_to_second_war() {
        let mut v = view();
        v.turn = 51;
        // Leave only 2 fit melee (below the 4-unit fresh-war threshold) at war with the opening victim.
        v.units.retain(|u| u.id.0 <= 2);
        v.diplomacy.relations.push(Relation {
            a: TeamId(0),
            b: TeamId(2),
            state: RelationState::War,
            effective_turn: 20,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: vec![],
        });
        let mut p = planner(&v, 9);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.rome_opening_march = 1;
        p.decisions.rome_opening_finish = 1;
        p.decisions.rome_second_war = 1;
        p.campaigns.rome_victim = Some(PlayerId(2));
        p.choose_rome_opening();
        p.choose_rome_second_war();
        assert_eq!(p.campaigns.rome_second_target, Some(CityId(2)));
        assert_eq!(p.campaigns.rome_second_enemy, Some(TeamId(2)));
        assert!(p.rome_second_refuses_peace(TeamId(2)));
    }
}

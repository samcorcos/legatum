//! A committed Roman opening, using only the seat's fogged PlayerView.
use crate::analysis::city_power;
use crate::plan::{Plan, PlanKind};
use crate::scripted::Planner;
use legatum_core::view::CityView;
use legatum_core::{AiDifficulty, Faction, Hex, ProductionItem, UnitClass, UnitKind, data};

impl Planner<'_> {
    /// Rome, or Sparta with gene `sparta_rome_opening`: the seat plays the committed
    /// conquest opening (march, finish, second war).
    pub(crate) fn conquest_opening_faction(&self) -> bool {
        self.faction == Faction::Rome
            || (self.faction == Faction::Sparta && self.decisions.sparta_rome_opening > 0)
    }

    pub(crate) fn rome_opening_active(&self) -> bool {
        self.conquest_opening_faction()
            && self.difficulty == AiDifficulty::Hard
            && self.decisions.rome_opening_march > 0
            && self.view().turn <= self.rome_opening_last_turn()
            && self.campaigns.rome_opening_target.is_some()
    }

    pub(crate) fn rome_opening_city(&self) -> Option<&CityView> {
        self.rome_opening_active()
            .then_some(self.campaigns.rome_opening_target)
            .flatten()
            .and_then(|id| {
                self.view().cities.iter().find(|city| {
                    city.id == id && !self.view().same_team(self.view().me, city.owner)
                })
            })
    }

    fn rome_opening_melee(&self) -> Vec<&legatum_core::view::UnitView> {
        self.ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| data::unit_class(u.kind) == Some(UnitClass::Melee) && u.hp > 0)
            .collect()
    }

    pub(crate) fn rome_opening_ready(&self, city: &CityView) -> bool {
        let melee = self.rome_opening_melee();
        if self.faction == Faction::Sparta
            && self.decisions.sparta_scout_declare > 0
            && city.visible
            && !city.walls
            && self
                .view()
                .units
                .iter()
                .filter(|u| u.owner == city.owner && u.military && u.pos == city.pos)
                .count()
                == 1
            && melee.iter().any(|u| u.pos.distance(city.pos) == 1)
            && self
                .ctx
                .own_units
                .iter()
                .any(|u| u.kind == UnitKind::Scout && u.hp > 0 && u.pos.distance(city.pos) == 1)
        {
            return true;
        }
        // Gene `sparta_opening_ready`: Sparta's veteran Hoplites may strike with fewer.
        // They declare from four hexes: a Hoplite pair one march out still strikes first.
        let (need, reach) = match self.decisions.sparta_opening_ready {
            n if n > 0 && self.faction == Faction::Sparta => (n as usize, 4),
            _ => (3, 3),
        };
        melee.len() >= need
            && melee
                .iter()
                .filter(|u| u.pos.distance(city.pos) <= reach)
                .count()
                >= need.min(2)
    }

    /// Gene `sparta_opening_search`: Sparta's opening has no victim in sight yet.
    pub(crate) fn sparta_opening_seeking(&self) -> bool {
        self.faction == Faction::Sparta
            && self.conquest_opening_faction()
            && self.difficulty == AiDifficulty::Hard
            && self.decisions.rome_opening_march > 0
            && self.decisions.sparta_opening_search > 0
            && self.campaigns.rome_opening_target.is_none()
            && self.view().turn <= crate::rome_finish::OPENING_LAST_TURN
    }

    pub(crate) fn rome_opening_build(&self, city: &CityView) -> Option<ProductionItem> {
        // Gene `sparta_opening_search`: the wing is trained while its victim is still sought.
        let seeking = self.sparta_opening_seeking() && city.is_capital;
        // Gene `sparta_war_army`: Sparta keeps training melee for its opening and second war
        // up to this many units.
        let army = (self.faction == Faction::Sparta
            && self.conquest_opening_faction()
            && self.decisions.sparta_war_army > 0)
            .then_some(self.decisions.sparta_war_army as usize);
        let second = army.is_some() && self.campaigns.rome_second_target.is_some();
        let goal = army.map_or(self.rome_opening_melee_goal(), |n| {
            n.max(self.rome_opening_melee_goal())
        });
        if !(self.rome_opening_active() || seeking || second)
            || (!second && self.view().turn > self.rome_opening_last_turn() - 10)
            || self.rome_opening_melee().len() >= goal
        {
            return None;
        }
        city.detail
            .as_ref()?
            .options
            .iter()
            .filter_map(|option| match option.item {
                ProductionItem::Unit { unit }
                    if data::unit_class(unit) == Some(UnitClass::Melee)
                        && option.resource_blocker.is_none()
                        && option.turns > 0 =>
                {
                    Some((unit, option.item))
                }
                _ => None,
            })
            .max_by_key(|(unit, _)| data::unit_stats(*unit).strength)
            .map(|(_, item)| item)
    }

    pub(crate) fn choose_rome_opening(&mut self) {
        if !self.conquest_opening_faction()
            || self.difficulty != AiDifficulty::Hard
            || self.decisions.rome_opening_march == 0
        {
            return;
        }
        // Gene `rome_opening_finish`: the war on the opening's city names the victim.
        self.note_rome_victim();
        if self.view().turn > self.rome_opening_last_turn() {
            return;
        }
        let v = self.view();
        let candidates: Vec<_> = v
            .cities
            .iter()
            .filter(|city| {
                !city.owner.is_barbarian()
                    && !v.same_team(v.me, city.owner)
                    && (v.at_war(v.me, city.owner)
                        || crate::opportunity::legal_declaration(v, v.team_of(city.owner)))
                    && self.rome_opening_melee().iter().any(|u| {
                        self.ctx.reachable(u.pos, city.pos)
                            || city
                                .pos
                                .ring(3)
                                .iter()
                                .any(|staging| self.ctx.reachable(u.pos, *staging))
                    })
            })
            .collect();
        let prior = self
            .campaigns
            .rome_opening_target
            .and_then(|id| candidates.iter().find(|c| c.id == id).copied());
        let selected = if self.rome_finish() {
            let scored: Vec<_> = candidates
                .iter()
                .map(|city| (*city, self.rome_opening_distance(city.pos)))
                .collect();
            self.rome_finish_target(&scored, prior)
        } else if prior.is_some_and(|city| !city.walls) {
            prior
        } else if prior.is_some() {
            candidates
                .iter()
                .copied()
                .filter(|city| !city.walls)
                .min_by_key(|city| {
                    (
                        city_power(v, city),
                        self.rome_opening_distance(city.pos),
                        city.id,
                    )
                })
                .or(prior)
        } else {
            candidates
                .iter()
                .copied()
                .filter(|city| !city.walls)
                .min_by_key(|city| (self.rome_opening_distance(city.pos), city.id))
        };
        let Some(city) = selected else {
            if self.rome_finish() {
                // No rival worth the opening yet: the march waits for one to be found.
                self.campaigns.rome_opening_target = None;
            }
            return;
        };
        self.campaigns.rome_opening_target = Some(city.id);
        let kind = PlanKind::Campaign {
            against: v.team_of(city.owner),
            target: city.id,
        };
        let prior_plan = self.campaigns.plan.active.as_ref();
        if self
            .campaigns
            .active
            .as_ref()
            .is_some_and(|active| active.target != city.id)
        {
            self.campaigns.active = None;
        }
        self.campaign_withdraw.clear();
        let start = prior_plan
            .filter(|plan| plan.kind == kind)
            .map_or(v.turn, |plan| plan.start_turn);
        self.campaigns.plan.active = Some(Plan {
            kind,
            start_turn: start,
            review_turn: v.turn + 1,
            reserve: 0,
            value: 0,
            terms: vec![("Roman opening march".into(), 0)],
            breach: false,
            misses: 0,
        });
    }

    /// Gene `sparta_opening_search`: with no rival city known, up to two melee units that are
    /// not a city's only defender walk to the nearest unexplored land within the gene's radius
    /// of the capital, so the opening finds its victim.
    pub(crate) fn sparta_opening_search(&mut self, army: &[&legatum_core::view::UnitView]) {
        if !self.sparta_opening_seeking() {
            return;
        }
        let Some(home) = self.ctx.capital().map(|c| c.pos) else {
            return;
        };
        let radius = self.decisions.sparta_opening_search;
        let centre = Hex::from_offset(self.view().width / 2, self.view().height / 2);
        let direct = home.distance(centre);
        let land: Vec<Hex> = self
            .ctx
            .frontier
            .iter()
            .copied()
            .filter(|h| h.distance(home) <= radius && self.ctx.unexplored_near(*h) > 0)
            .collect();
        let alone = |p: &Self, u: &legatum_core::view::UnitView| {
            p.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                && p.ctx
                    .own_units
                    .iter()
                    .filter(|o| o.pos == u.pos && o.id != u.id && o.military)
                    .count()
                    == 0
        };
        let searchers: Vec<_> = self
            .free(army)
            .into_iter()
            .filter(|u| {
                data::unit_class(u.kind) == Some(UnitClass::Melee) && u.hp >= 50 && !alone(self, u)
            })
            .take(2)
            .collect();
        let mut taken: Vec<Hex> = Vec::new();
        for u in searchers {
            if self.has_acted(u.id) {
                continue;
            }
            let goal = land
                .iter()
                .copied()
                .filter(|h| {
                    *h != u.pos
                        && taken.iter().all(|t| t.distance(*h) > 3)
                        && self.free_tile_for(u, *h)
                        && self.ctx.reachable(u.pos, *h)
                })
                .min_by_key(|h| {
                    let cost = u.pos.distance(*h) + h.distance(home);
                    let toward_centre = self.decisions.sparta_center_search > 0
                        && h.distance(home) + h.distance(centre) <= direct + 4
                        && h.distance(centre) < direct;
                    (cost - if toward_centre { 12 } else { 0 }, *h)
                });
            if let Some(goal) = goal {
                taken.push(goal);
                self.go_to(u, goal);
            }
        }
    }

    fn rome_opening_distance(&self, target: Hex) -> i32 {
        self.ctx
            .own_cities
            .iter()
            .map(|home| home.pos.distance(target))
            .min()
            .unwrap_or(i32::MAX)
    }

    /// Keep spare melee on the shortest direct approach. Tactical assault and emergency
    /// combat passes have already taken any unit that can profitably strike this turn.
    pub(crate) fn march_rome_opening(&mut self, army: &[&legatum_core::view::UnitView]) {
        let Some(city) = self.rome_opening_city() else {
            return;
        };
        let target = city.pos;
        let at_war = self.view().at_war(self.view().me, city.owner);
        let finish = self.rome_finish();
        // Gene `rome_opening_finish`: out of a walled city's fire, and at peace two hexes out
        // where the land is open, so the declaration turn lands hits.
        let radius = if finish && at_war && city.walls && city.hp > 0 {
            3
        } else if at_war {
            1
        } else {
            3
        };
        let open_at = |p: &Self, unit: &legatum_core::view::UnitView, h: Hex| {
            p.free_tile_for(unit, h)
                && p.ctx.reachable(unit.pos, h)
                && p.view().tile(h).is_some_and(|tile| {
                    tile.terrain.is_some()
                        && p.view().passage_allowed(p.view().me, unit.kind, tile.owner)
                })
        };
        let melee: Vec<_> = self
            .free(army)
            .into_iter()
            .filter(|u| data::unit_class(u.kind) == Some(UnitClass::Melee))
            .collect();
        for unit in melee {
            if self.has_acted(unit.id) {
                continue;
            }
            if finish && !at_war {
                let near = target
                    .ring(2)
                    .into_iter()
                    .filter(|h| *h == unit.pos || open_at(self, unit, *h))
                    .min_by_key(|h| (unit.pos.distance(*h), *h));
                if let Some(near) = near {
                    self.go_to(unit, near);
                    continue;
                }
            }
            if unit.pos.distance(target) <= radius && self.free_tile_for(unit, unit.pos) {
                self.go_to(unit, unit.pos);
                continue;
            }
            let spot = target
                .ring(radius)
                .into_iter()
                .filter(|h| {
                    self.free_tile_for(unit, *h)
                        && self.ctx.reachable(unit.pos, *h)
                        && self.view().tile(*h).is_some_and(|tile| {
                            tile.terrain.is_some()
                                && self.view().passage_allowed(
                                    self.view().me,
                                    unit.kind,
                                    tile.owner,
                                )
                        })
                })
                .min_by_key(|h| (unit.pos.distance(*h), *h));
            if let Some(spot) = spot {
                self.go_to(unit, spot);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
    use legatum_core::{CityId, Order, PlayerId, PlayerView, TeamId, UnitId, UnitKind};

    fn opening_view() -> PlayerView {
        let mut view = ViewBuilder::new(28, 18)
            .faction(Faction::Rome)
            .city(1, ME, off(4, 8), 3)
            .city(2, ENEMY, off(10, 8), 3)
            .city(3, ENEMY, off(14, 8), 3)
            .unit(1, ME, UnitKind::Spearman, off(8, 8))
            .unit(2, ME, UnitKind::Spearman, off(8, 9))
            .unit(3, ME, UnitKind::Spearman, off(5, 8))
            .build();
        view.turn = 19;
        view.diplomacy.legacy = false;
        view.diplomacy.relations.push(Relation {
            a: TeamId(0),
            b: TeamId(2),
            state: RelationState::War,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: Vec::new(),
        });
        view
    }

    #[test]
    fn opening_keeps_the_nearest_target_until_walls_require_a_softer_one() {
        let mut view = opening_view();
        let mut planner = Planner::new(&view, Knobs::for_faction(Faction::Rome), Faction::Rome, 1);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        planner.choose_rome_opening();
        assert_eq!(planner.campaigns.rome_opening_target, Some(CityId(2)));
        assert!(planner.rome_opening_ready(planner.rome_opening_city().unwrap()));
        let memory = planner.campaigns.clone();
        view.cities
            .iter_mut()
            .find(|city| city.id == CityId(2))
            .unwrap()
            .walls = true;
        let mut planner = Planner::new(&view, Knobs::for_faction(Faction::Rome), Faction::Rome, 1);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        planner.campaigns = memory;
        planner.choose_rome_opening();
        assert_eq!(planner.campaigns.rome_opening_target, Some(CityId(3)));
        assert!(matches!(
            planner.campaigns.plan.active.as_ref().unwrap().kind,
            PlanKind::Campaign {
                target: CityId(3),
                ..
            }
        ));
    }

    #[test]
    fn opening_declaration_waits_for_three_melee_and_two_close() {
        let mut view = opening_view();
        let mut planner = Planner::new(&view, Knobs::for_faction(Faction::Rome), Faction::Rome, 1);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        planner.choose_rome_opening();
        assert!(planner.rome_opening_ready(planner.rome_opening_city().unwrap()));
        view.units.retain(|u| u.id.0 != 3);
        let mut planner = Planner::new(&view, Knobs::for_faction(Faction::Rome), Faction::Rome, 1);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        planner.choose_rome_opening();
        assert!(!planner.rome_opening_ready(planner.rome_opening_city().unwrap()));
    }

    #[test]
    fn opening_targets_a_city_reachable_from_peace_time_staging_tiles() {
        let mut view = opening_view();
        view.diplomacy.relations[0].state = RelationState::Peace;
        view.diplomacy.actions.push(DiplomaticAction {
            new_wars: vec![],
            calls: vec![],
            also_agree: vec![],
            order: Order::DeclareWar {
                against: TeamId(2),
                breach: false,
            },
            effective_turn: view.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        let mut planner = Planner::new(&view, Knobs::for_faction(Faction::Rome), Faction::Rome, 1);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        assert!(!planner.ctx.reachable(off(8, 8), off(10, 8)));
        planner.choose_rome_opening();
        assert_eq!(planner.campaigns.rome_opening_target, Some(CityId(2)));
        assert!(planner.rome_opening_ready(planner.rome_opening_city().unwrap()));
    }

    #[test]
    fn sparta_plays_the_opening_only_with_its_gene_and_may_strike_with_two() {
        let mut view = opening_view();
        view.units.retain(|u| u.id.0 != 3);
        let knobs = Knobs::for_faction(Faction::Sparta);
        let mut planner = Planner::new(&view, knobs, Faction::Sparta, 1);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        planner.decisions.sparta_rome_opening = 0;
        planner.choose_rome_opening();
        assert_eq!(planner.campaigns.rome_opening_target, None);

        let mut planner = Planner::new(&view, knobs, Faction::Sparta, 1);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        planner.decisions.sparta_rome_opening = 1;
        planner.decisions.sparta_opening_ready = 0;
        planner.choose_rome_opening();
        assert_eq!(planner.campaigns.rome_opening_target, Some(CityId(2)));
        // Rome's rule: three melee. Two Hoplites are not enough without the ready gene.
        assert!(!planner.rome_opening_ready(planner.rome_opening_city().unwrap()));
        planner.decisions.sparta_opening_ready = 2;
        assert!(planner.rome_opening_ready(planner.rome_opening_city().unwrap()));
    }

    #[test]
    fn sparta_scout_declaration_needs_one_visible_garrison_and_two_adjacent_units() {
        let mut view = opening_view();
        view.units.retain(|u| u.id.0 <= 2);
        view.units[0].pos = off(9, 8);
        view.units[1].kind = UnitKind::Scout;
        view.units[1].pos = off(9, 9);
        let mut guard = view.units[0].clone();
        guard.id = UnitId(4);
        guard.owner = ENEMY;
        guard.pos = off(10, 8);
        view.units.push(guard);
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Sparta),
            Faction::Sparta,
            1,
        );
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.sparta_rome_opening = 1;
        planner.decisions.sparta_scout_declare = 1;
        planner.choose_rome_opening();
        assert!(planner.rome_opening_ready(planner.rome_opening_city().unwrap()));
        planner.decisions.sparta_scout_declare = 0;
        assert!(!planner.rome_opening_ready(planner.rome_opening_city().unwrap()));
    }

    #[test]
    fn sparta_chains_to_a_capital_after_its_victim_is_gone() {
        let mut view = opening_view();
        view.turn = 45;
        view.cities
            .iter_mut()
            .find(|c| c.id == CityId(2))
            .unwrap()
            .is_capital = true;
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Sparta),
            Faction::Sparta,
            1,
        );
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.sparta_rome_opening = 1;
        planner.decisions.sparta_chain_opening = 1;
        planner.campaigns.rome_victim = Some(PlayerId(99));
        planner.choose_rome_opening();
        assert_eq!(planner.campaigns.rome_opening_target, Some(CityId(2)));
        planner.decisions.sparta_chain_opening = 0;
        assert!(!planner.rome_opening_active());
    }

    #[test]
    fn sparta_does_not_chain_after_peace_with_a_living_victim() {
        let mut view = opening_view();
        view.turn = 45;
        view.diplomacy.relations[0].state = RelationState::Peace;
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Sparta),
            Faction::Sparta,
            1,
        );
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.sparta_rome_opening = 1;
        planner.decisions.sparta_chain_opening = 1;
        planner.campaigns.rome_victim = Some(ENEMY);
        planner.campaigns.rome_walled_peace_window = true;
        planner.choose_rome_opening();
        assert_eq!(planner.campaigns.rome_opening_target, None);
    }

    #[test]
    fn sparta_living_victim_can_keep_the_opening_through_turn_sixty() {
        let mut view = opening_view();
        view.turn = 55;
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Sparta),
            Faction::Sparta,
            1,
        );
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.sparta_rome_opening = 1;
        planner.campaigns.rome_victim = Some(ENEMY);
        planner.decisions.sparta_finish_60 = 1;
        assert_eq!(planner.rome_victim(), Some(ENEMY));
        assert_eq!(planner.rome_opening_last_turn(), 60);
        planner.decisions.sparta_finish_60 = 0;
        assert_eq!(planner.rome_victim(), None);
    }

    #[test]
    fn sparta_seeks_a_victim_only_while_none_is_known() {
        let mut view = opening_view();
        view.cities.retain(|c| c.owner == ME);
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Sparta),
            Faction::Sparta,
            1,
        );
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        planner.decisions.sparta_rome_opening = 1;
        planner.decisions.sparta_opening_search = 10;
        planner.choose_rome_opening();
        assert!(planner.sparta_opening_seeking());
        let known = opening_view();
        let mut planner = Planner::new(
            &known,
            Knobs::for_faction(Faction::Sparta),
            Faction::Sparta,
            1,
        );
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.rome_opening_march = 1;
        planner.decisions.sparta_rome_opening = 1;
        planner.decisions.sparta_opening_search = 10;
        planner.choose_rome_opening();
        assert!(!planner.sparta_opening_seeking());
    }
}

//! Macro commitments derived only from the owner's fogged view. Schedules complement the
//! shared settlement scorer without changing the frozen Proxy policy.

use crate::scripted::Planner;
use legatum_core::view::CityView;
use legatum_core::{AiDifficulty, BuildingKind, CityFocus, Hex, data, economy};

impl Planner<'_> {
    /// Estimate an initial military build from the known founding ring. Production
    /// focus can select very different workers from the balanced settlement score.
    pub(crate) fn rush_capital_production(&self, pos: Hex, travel: i32) -> i32 {
        let yields = |h| {
            let tile = self.view().tile(h)?;
            Some(
                data::terrain_yields(self.faction, tile.terrain?)
                    + legatum_core::resources::tile_bonus(tile.resource, |tech| {
                        self.view().research.known.contains(&tech)
                    }),
            )
        };
        let center = data::CITY_CENTER_YIELDS + yields(pos).unwrap_or(legatum_core::Yields::ZERO);
        let mut workers: Vec<_> = pos
            .neighbors()
            .into_iter()
            .filter(|h| {
                self.view()
                    .tile(*h)
                    .is_some_and(|t| t.explored && t.owner.is_none_or(|o| o == self.ctx.me))
            })
            .filter_map(|h| yields(h).map(|y| (h, y)))
            .collect();
        workers.sort_by_key(|(h, y)| {
            (
                std::cmp::Reverse(economy::tile_score(*y, CityFocus::Production)),
                *h,
            )
        });
        let mut population = 1;
        let mut food = 0;
        let mut production = 0;
        for _ in travel..40 {
            let y = workers
                .iter()
                .take(population as usize)
                .fold(center, |total, (_, y)| total + *y);
            production += y.production.max(data::CITY_MIN_PRODUCTION);
            food += y.food - data::FOOD_PER_POP * population;
            let growth = data::growth_threshold(population);
            if food >= growth {
                food -= growth;
                population += 1;
            } else if food < 0 {
                population = (population - 1).max(1);
                food = 0;
            }
        }
        production
    }

    /// Hard adds a short-term productivity preference to the shared suitability score.
    /// Only the initial ring can support the first citizens; a rich eventual radius-two
    /// territory does not repay a colony that starts on three production indefinitely.
    pub(crate) fn settlement_productivity(&self, pos: Hex) -> i32 {
        if self.difficulty != AiDifficulty::Hard {
            return 100;
        }
        let Some(center) = self.view().tile(pos).and_then(|t| t.terrain) else {
            return 100;
        };
        let mut workers: Vec<_> = pos
            .neighbors()
            .into_iter()
            .filter_map(|h| self.view().tile(h).map(|t| (h, t)))
            .filter(|(_, t)| t.explored && t.owner.is_none_or(|owner| owner == self.ctx.me))
            .filter_map(|(h, t)| {
                t.terrain
                    .map(|terrain| (h, data::terrain_yields(self.faction, terrain)))
            })
            .collect();
        workers.sort_by_key(|(h, y)| {
            (
                std::cmp::Reverse(economy::tile_score(*y, CityFocus::Balanced)),
                *h,
            )
        });
        let yields = workers.iter().take(2).fold(
            data::CITY_CENTER_YIELDS + data::terrain_yields(self.faction, center),
            |sum, (_, y)| sum + *y,
        );
        100 + (yields.production - data::CITY_MIN_PRODUCTION).max(0) * 20
            + (yields.food - 4).clamp(-2, 4) * 5
    }

    /// Do not consume every small income payment finishing buildings while a frontier
    /// garrison waits forever for its already researched upgrade. Budget one quoted
    /// upgrade only when it is affordable soon; emergency purchases still go first.
    pub(crate) fn frontline_upgrade_reserve(&self) -> i32 {
        self.ctx
            .own_units
            .iter()
            .filter(|u| u.hp >= 60)
            .filter_map(|u| {
                self.ctx
                    .own_cities
                    .iter()
                    .find(|c| c.pos == u.pos && self.exposed_city(c))?;
                let next = data::unit_stats(u.kind).upgrades_to?;
                let tech = data::unit_stats(next).requires?;
                if !self.view().research.known.contains(&tech) {
                    return None;
                }
                let cost = u.upgrade_breakdown.as_ref()?.total;
                (cost > 0
                    && self.gold_left + self.view().economy.gold_per_turn.total.max(0) * 12 >= cost)
                    .then_some(cost)
            })
            .min()
            .unwrap_or(0)
            .max(self.modernization_reserve())
    }

    /// A known approach matters; a rival on the other side of an unscouted sea does not.
    pub(crate) fn exposed_city(&self, c: &CityView) -> bool {
        let near = |h: Hex, radius| c.pos.distance(h) <= radius && self.ctx.reachable(c.pos, h);
        self.ctx.enemy_cities.iter().any(|e| near(e.pos, 8))
            || self
                .ctx
                .threat_units
                .iter()
                .any(|e| e.military && near(e.pos, 5))
    }

    /// Walls are the counter to a rush: only a walled city fires (`legatum_core::city_fires`).
    /// A rival with a city within eight reachable hexes is aggressive when it is at war with
    /// us, has declared any war, or has two or more military units within six hexes of this
    /// city while no Alliance binds it. Public relations and visible units only.
    pub(crate) fn aggressive_neighbour(&self, c: &CityView) -> bool {
        use legatum_core::diplomacy::RelationState;
        // A conqueror on the war path (threat.rs) within reach is the strongest reason to wall.
        if self.threat_exposure(c) > 0 || self.watched_exposure(c) {
            return true;
        }
        let v = self.view();
        let near = |h: Hex, radius| c.pos.distance(h) <= radius && self.ctx.reachable(c.pos, h);
        let mut rivals: Vec<_> = v
            .cities
            .iter()
            .filter(|e| !e.owner.is_barbarian() && !v.same_team(v.me, e.owner) && near(e.pos, 8))
            .map(|e| v.team_of(e.owner))
            .collect();
        rivals.sort();
        rivals.dedup();
        rivals.into_iter().any(|team| {
            let bound = v
                .diplomacy
                .relations
                .iter()
                .any(|r| r.between(v.team, team) && r.state == RelationState::Alliance);
            let declared = v
                .diplomacy
                .relations
                .iter()
                .any(|r| r.state == RelationState::War && r.declarers.contains(&team));
            let massed = v
                .units
                .iter()
                .filter(|u| u.military && v.team_of(u.owner) == team && near(u.pos, 6))
                .count()
                >= 2;
            let allied_exposure = bound
                && self.decisions.ally_defense_percent > 0
                && massed
                && crate::diplomacy::strength(v, team) * self.decisions.ally_defense_percent
                    >= crate::diplomacy::strength(v, v.team).max(1) * 150;
            v.teams_at_war(v.team, team) || allied_exposure || !bound && (declared || massed)
        })
    }

    /// These are deadlines to start investment, not guaranteed completions. Hard anticipates
    /// the next bottleneck earlier; Normal waits longer with identical costs and information.
    pub(crate) fn infrastructure_bonus(&self, ci: usize, c: &CityView, b: BuildingKind) -> i32 {
        let d = c.detail.as_ref().expect("own city");
        let has = |b| data::has_building(&d.buildings, b);
        let delay = if self.difficulty == AiDifficulty::Hard {
            0
        } else {
            24
        };
        let due = |turn| self.view().turn >= (turn + delay) as u32;
        let threatened = self.ctx.threat[ci] > self.local_defense(c);
        if b == BuildingKind::Walls && self.exposed_city(c) && due(36) {
            return if threatened {
                1800
            } else if self.decisions.ally_defense_percent > 0 && self.aggressive_neighbour(c) {
                600
            } else {
                250
            };
        }
        // Walls are what make a city fire: build them before a neighbour's rush lands.
        if b == BuildingKind::Walls && self.aggressive_neighbour(c) {
            return if threatened { 1800 } else { 600 };
        }
        if threatened {
            return 0;
        }
        match b {
            BuildingKind::Granary if due(30) && d.yields.total.food <= 2 * c.population + 2 => 1100,
            BuildingKind::Workshop if due(40) => 1000,
            BuildingKind::Laboratory if due(64) && has(BuildingKind::Workshop) => {
                let first_lab = self.difficulty == AiDifficulty::Hard
                    && !self.ctx.own_cities.iter().any(|city| {
                        city.detail.as_ref().is_some_and(|d| {
                            data::has_building(&d.buildings, BuildingKind::Laboratory)
                                || d.producing
                                    == Some(legatum_core::ProductionItem::Building {
                                        building: BuildingKind::Laboratory,
                                    })
                        })
                    })
                    && !self.orders.iter().any(|o| {
                        matches!(
                            o,
                            legatum_core::Order::SetProduction {
                                item: legatum_core::ProductionItem::Building {
                                    building: BuildingKind::Laboratory
                                },
                                ..
                            } | legatum_core::Order::BuyItem {
                                item: legatum_core::ProductionItem::Building {
                                    building: BuildingKind::Laboratory
                                },
                                ..
                            }
                        )
                    });
                if first_lab || self.ctx.own_cities.len() >= 3 {
                    1400
                } else {
                    600
                }
            }
            BuildingKind::Laboratory
                if due(110)
                    && !d.options.iter().any(|o| {
                        o.item
                            == (legatum_core::ProductionItem::Building {
                                building: BuildingKind::Workshop,
                            })
                    }) =>
            {
                1100
            }
            BuildingKind::Market
                if due(70)
                    && self.view().economy.gold_per_turn.total
                        < 4 * self.ctx.own_cities.len() as i32 =>
            {
                900
            }
            BuildingKind::Granary if due(70) => 800,
            BuildingKind::College if due(130) => 800,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use crate::{Brain, ScriptedBrain};
    use legatum_core::{BuildingKind, Faction, Order, ProductionItem, TechId, UnitKind};

    #[test]
    fn stronger_allied_army_near_city_keeps_walls_valuable() {
        use crate::scripted::Planner;
        use legatum_core::diplomacy::{Relation, RelationState, SecurityTreaty};
        let mut v = ViewBuilder::new(24, 16)
            .known_techs(&[TechId::Masonry])
            .city(1, ME, off(4, 6), 4)
            .city(2, ENEMY, off(10, 6), 4)
            .unit(20, ENEMY, UnitKind::Soldier, off(8, 6))
            .unit(21, ENEMY, UnitKind::Soldier, off(8, 7))
            .build();
        v.turn = 40;
        v.diplomacy.legacy = false;
        crate::fixture::set_armies(&mut v, &[(ME, 100), (ENEMY, 200)]);
        let rival = v.team_of(ENEMY);
        v.diplomacy.relations.push(Relation {
            a: v.team,
            b: rival,
            state: RelationState::Alliance,
            effective_turn: 20,
            guarantee_until: 0,
            version: 1,
            security: Some(SecurityTreaty {
                id: 1,
                started_turn: 20,
                protected_until: 30,
                coalition_target: None,
            }),
            war_id: None,
            declarers: Vec::new(),
        });
        let value = |gene| {
            let mut p = Planner::new(
                &v,
                crate::Knobs::for_faction(Faction::India),
                Faction::India,
                1,
            );
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.decisions.ally_defense_percent = gene;
            p.infrastructure_bonus(0, &v.cities[0], BuildingKind::Walls)
        };
        assert!(value(0) < 600, "baseline {}", value(0));
        assert!(value(100) >= 600, "priced {}", value(100));
    }

    #[test]
    fn settling_continues_past_the_personality_target_while_land_and_time_remain() {
        // Germany's knobs once capped it at four cities (3 + expansion / 20) with open
        // land all around. Land, time and price now decide; timid Easy keeps its target.
        let mut b = ViewBuilder::new(40, 24).faction(Faction::Germany);
        for (i, (col, row)) in [(4, 4), (4, 10), (10, 4), (10, 10), (16, 4)]
            .into_iter()
            .enumerate()
        {
            let id = i as u32 + 1;
            b = b.city(id, ME, off(col, row), 3).unit(
                10 + id,
                ME,
                UnitKind::Soldier,
                off(col, row),
            );
        }
        let mut v = b.unit(30, ME, UnitKind::Soldier, off(5, 5)).build();
        v.turn = 160;
        // A Germany with five Foundries has no further city obligation from its goal.
        v.economy.gold = 5000;
        for resource in [
            legatum_core::resources::Resource::Iron,
            legatum_core::resources::Resource::Coal,
        ] {
            v.supply.push(legatum_core::resources::SupplyView {
                resource,
                revealed_by: None,
                revealed: true,
                supplied: Default::default(),
                deposits: vec![],
                used: Default::default(),
                idle: vec![],
                held: vec![],
                spare: 4,
            });
        }
        for c in &mut v.cities {
            let buildings = &mut c.detail.as_mut().unwrap().buildings;
            buildings.push(BuildingKind::Workshop);
            buildings.push(BuildingKind::MachineShop);
        }
        let gates = |v: &legatum_core::PlayerView, difficulty| {
            ScriptedBrain::with_difficulty(Faction::Germany, 7, ME, difficulty)
                .production_diagnosis(v)
                .1
                .into_iter()
                .find_map(|t| t.settler)
                .expect("an idle city weighed a Settler")
        };
        let normal = gates(&v, legatum_core::AiDifficulty::Normal);
        assert!(normal.want && normal.value > 0, "{normal:?}");
        let hard = gates(&v, legatum_core::AiDifficulty::Hard);
        assert!(hard.want && hard.value > 0, "{hard:?}");
        let easy = gates(&v, legatum_core::AiDifficulty::Easy);
        assert!(!easy.want && easy.value == 0, "{easy:?}");
        // Under a turn limit, a colony that cannot be founded and repay itself in time is
        // worth nothing; uncapped games have no deadline.
        v.turn_limit = Some(v.turn + 10);
        let late = gates(&v, legatum_core::AiDifficulty::Hard);
        assert!(late.value == 0 && hard.value > 0, "{late:?} vs {hard:?}");
    }

    #[test]
    fn a_small_city_develops_then_exports_a_citizen_and_counts_the_pipeline() {
        let mut v = ViewBuilder::new(28, 18)
            .faction(Faction::India)
            .known_techs(&[TechId::Irrigation, TechId::BronzeWorking, TechId::Writing])
            .city(1, ME, off(5, 8), 2)
            .unit(1, ME, UnitKind::Soldier, off(5, 8))
            .unit(2, ME, UnitKind::Soldier, off(5, 9))
            .unit(3, ME, UnitKind::Ranger, off(4, 8))
            .gold(0)
            .build();
        v.turn = 65;
        let settler = ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        for o in &mut v.cities[0].detail.as_mut().unwrap().options {
            if o.item == settler {
                o.cost += 20;
            }
        }
        let choose = |v: &legatum_core::PlayerView| {
            ScriptedBrain::new(Faction::India, 7, ME)
                .orders(v)
                .into_iter()
                .find_map(|o| match o {
                    Order::SetProduction { item, .. } => Some(item),
                    _ => None,
                })
        };
        assert_eq!(
            choose(&v),
            Some(ProductionItem::Building {
                building: BuildingKind::Granary
            })
        );
        v.cities[0].population = 3;
        v.cities[0].detail.as_mut().unwrap().buildings.extend([
            BuildingKind::Granary,
            BuildingKind::Workshop,
            BuildingKind::Laboratory,
        ]);
        assert_eq!(choose(&v), Some(settler));
        v.units.extend(
            ViewBuilder::new(28, 18)
                .unit(10, ME, UnitKind::Settler, off(9, 8))
                .build()
                .units,
        );
        assert_ne!(
            choose(&v),
            Some(settler),
            "finish the colony already on its way"
        );
    }
}

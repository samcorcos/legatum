//! Compare the remaining paid commitments for world wonders using only PlayerView.
use crate::{analysis::Ctx, scripted::Planner};
use legatum_core::{BuildingKind, CityId, ProductionItem, TechId, data};
use std::collections::BTreeSet;

pub(crate) struct WorkPlan {
    pub building: BuildingKind,
    pub host: CityId,
    pub steps: Vec<(CityId, BuildingKind)>,
    pub research: BTreeSet<TechId>,
    pub turns: i32,
}

/// Every technology that must be known before `tech` can be researched under the strict gate:
/// all of the earlier eras (there are no prerequisites inside an era).
pub(crate) fn gate_prereqs(tech: TechId) -> impl Iterator<Item = TechId> {
    let era = legatum_core::eras::era_of_tech(tech);
    TechId::ALL
        .into_iter()
        .filter(move |t| legatum_core::eras::era_of_tech(*t) < era)
}

/// `tech` and what the gate makes it wait for, minus what is known, added to `missing`.
pub(crate) fn prerequisites(tech: TechId, known: &[TechId], missing: &mut BTreeSet<TechId>) {
    for t in std::iter::once(tech).chain(gate_prereqs(tech)) {
        if !known.contains(&t) {
            missing.insert(t);
        }
    }
}

/// Earliest completion of each prerequisite at today's science, retaining overflow and
/// the active research commitment. Construction can run alongside this schedule.
pub(crate) fn research_schedule(
    ctx: &Ctx<'_>,
    required: &BTreeSet<TechId>,
) -> std::collections::BTreeMap<TechId, i32> {
    let r = &ctx.view.research;
    let mut pending = required.clone();
    if let Some(current) = r.current {
        prerequisites(current, &r.known, &mut pending);
    }
    let mut ready = std::collections::BTreeMap::new();
    let mut spent = 0;
    let mut turn = 0;
    let rate = ctx.view.economy.science_per_turn.total.max(1);
    while !pending.is_empty() {
        let available =
            |t: &TechId| gate_prereqs(*t).all(|p| r.known.contains(&p) || ready.contains_key(&p));
        let next = r
            .current
            .filter(|t| pending.contains(t) && available(t))
            .or_else(|| pending.iter().copied().find(available))
            .expect("acyclic technology tree");
        spent += r
            .catalog
            .iter()
            .find(|o| o.tech == next)
            .map_or(data::tech_stats(next).cost, |o| o.cost.total);
        turn = (turn + 1).max(((spent - r.progress).max(0) + rate - 1) / rate);
        ready.insert(next, turn);
        pending.remove(&next);
    }
    ready
}

pub(crate) fn work_plan(ctx: &Ctx<'_>) -> Option<WorkPlan> {
    let view = ctx.view;
    if view
        .great_works
        .iter()
        .any(|work| work.player == ctx.me && work.completed.is_some())
    {
        return None;
    }
    let committed = ctx.own_cities.iter().find_map(|c| {
        let i = c
            .detail
            .as_ref()?
            .producing
            .filter(|i| data::is_great_work_item(*i))?;
        Some((c.id, i))
    });
    let faction = view.faction_of(ctx.me);
    let met = &view.diplomacy.met;
    let mut best: Option<WorkPlan> = None;
    let mut best_score = i32::MIN;
    for building in data::GREAT_WORKS {
        if view
            .great_works
            .iter()
            .any(|w| w.completed == Some(building))
        {
            continue;
        }
        let item = ProductionItem::Building { building };
        if committed.is_some_and(|(_, i)| {
            !matches!(i,
            ProductionItem::Building { building: b } | ProductionItem::Wonder { building: b, .. }
            if b == building)
        }) {
            continue;
        }
        let requirement = data::great_work_requirement(building);
        let count = requirement.map_or(1, |(_, count)| count);
        let mut research = BTreeSet::new();
        prerequisites(
            data::item_requires(item)?,
            &view.research.known,
            &mut research,
        );
        let mut chain = Vec::new();
        let mut next = requirement.map(|(required, _)| required);
        while let Some(b) = next {
            chain.push(b);
            if let Some(t) = data::building_stats(b).requires {
                prerequisites(t, &view.research.known, &mut research);
            }
            next = data::building_predecessor(b);
        }
        chain.reverse();
        let schedule = research_schedule(ctx, &research);
        let tech_ready = |b| {
            data::building_stats(b)
                .requires
                .and_then(|t| schedule.get(&t))
                .copied()
                .unwrap_or(0)
        };
        let mut candidates = Vec::new();
        for (ci, city) in ctx.own_cities.iter().enumerate() {
            let d = city.detail.as_ref()?;
            let steps: Vec<_> = chain
                .iter()
                .copied()
                .filter(|b| !data::has_building(&d.buildings, *b))
                .collect();
            // With the technology and support already in place, a missing production option
            // means this city cannot start the wonder (for example, the world race is over).
            if research.is_empty()
                && steps.is_empty()
                && !d.options.iter().any(|o| {
                    matches!(o.item,
                        ProductionItem::Building { building: b }
                            | ProductionItem::Wonder { building: b, .. } if b == building)
                })
            {
                continue;
            }
            // Supporting infrastructure remains owned even in resistance or under threat.
            if !steps.is_empty() && (ctx.threat[ci] > 0 || d.resistance > 0) {
                continue;
            }
            let gate = steps
                .iter()
                .map(|b| data::building_min_population(*b))
                .max()
                .unwrap_or(0);
            let growth = if city.population >= gate {
                0
            } else {
                // Reaching six is a conservative bound for all current (<=6) gates.
                let Some(turns) = view
                    .development
                    .as_ref()
                    .and_then(|d| d.growth.iter().find(|g| g.city == city.id))
                    .and_then(|g| g.turns_to_six.or(g.turns_to_eight))
                else {
                    continue;
                };
                turns
            };
            let mut delay = 0;
            // Progress belongs to its item: the first step's own bank, plus unassigned
            // Production when it would start now.
            let first = d.options.iter().find(|o| {
                steps
                    .first()
                    .is_some_and(|b| o.item == ProductionItem::Building { building: *b })
            });
            let mut stored = first.map_or(d.production_stored, |o| o.production_stored);
            if d.producing.is_some_and(|i| {
                i != item
                    && !steps
                        .iter()
                        .any(|b| i == ProductionItem::Building { building: *b })
            }) {
                delay = d.turns_to_complete.unwrap_or(60);
                // That build spends its own production first; only this step's bank waits.
                stored = first.map_or(0, |o| o.production_banked);
            }
            let mut rate = d
                .options
                .iter()
                .find(|o| o.yields.focus == legatum_core::CityFocus::Balanced)
                .map_or(d.yields.total.production, |o| o.yields.total.production)
                .max(1);
            let mut work_rate = d
                .options
                .iter()
                .find(|o| o.yields.focus == legatum_core::CityFocus::Production)
                .map_or(rate, |o| o.yields.total.production)
                .max(1);
            for b in &steps {
                let stats = view.rule_experiments.building_stats_for(faction, *b);
                let cost = view
                    .rule_experiments
                    .base_production_cost(faction, ProductionItem::Building { building: *b });
                delay = delay.max(tech_ready(*b));
                if city.population < data::building_min_population(*b) {
                    delay = delay.max(growth);
                }
                delay += ((cost - stored).max(1) + rate - 1) / rate;
                stored = 0;
                let gain = if faction == legatum_core::Faction::India {
                    let nearby = ctx
                        .own_cities
                        .iter()
                        .filter(|c| {
                            c.id != city.id
                                && c.pos.distance(city.pos) <= data::INDIA_NETWORK_RADIUS
                        })
                        .count() as i32;
                    let off = view.rule_experiments;
                    let nearby =
                        if off.rule_off(legatum_core::experiments::FactionRule::IndiaNetwork) {
                            0
                        } else {
                            nearby
                        };
                    let network = stats.yields.production
                        * (100 + nearby * data::INDIA_NETWORK_PERCENT_PER_CITY)
                        / 100;
                    let beyond = ctx
                        .capital()
                        .filter(|_| {
                            !off.rule_off(legatum_core::experiments::FactionRule::IndiaCorruption)
                        })
                        .map_or(0, |c| {
                            (c.pos.distance(city.pos) - data::INDIA_CAPITAL_RADIUS).max(0)
                        });
                    network * (100 - data::india_distance_loss_percent(beyond)) / 100
                } else {
                    stats.yields.production
                };
                // A far India city's network gain can be negative; a rate never drops below 1.
                rate = (rate + gain).max(1);
                work_rate = (work_rate + gain).max(1);
            }
            let cost = data::building_stats(building).cost;
            let finish = if steps.is_empty() && delay == 0 {
                d.options
                    .iter()
                    .find(|o| o.item == item && o.turns >= 1)
                    .map_or(((cost - stored).max(1) + work_rate - 1) / work_rate, |o| {
                        o.turns
                    })
            } else {
                ((cost - stored).max(1) + work_rate - 1) / work_rate
            };
            let host_ok = ctx.threat[ci] <= 0
                && d.resistance <= 0
                && !data::has_building(&d.buildings, building)
                && committed.is_none_or(|(id, _)| id == city.id);
            candidates.push((city.id, delay, finish, steps, host_ok));
        }
        for host in candidates.iter().filter(|c| c.4) {
            let mut support: Vec<_> = candidates.iter().filter(|c| c.0 != host.0).collect();
            support.sort_by_key(|c| (c.1, c.0));
            if support.len() + 1 < count {
                continue;
            }
            support.truncate(count - 1);
            let ready = support.iter().map(|c| c.1).max().unwrap_or(0).max(host.1);
            let turns = tech_ready(building).max(ready) + host.2;
            let rival_eta = view
                .great_works
                .iter()
                .filter(|w| w.player != ctx.me && w.building == Some(building))
                .filter(|w| {
                    view.players
                        .iter()
                        .any(|p| p.id == w.player && met.contains(&p.team))
                })
                .map(|w| (w.cost - w.progress).max(0) / w.production_per_turn.max(1))
                .min();
            // A known rival's faster public Production makes this commitment a bad bet.
            if rival_eta.is_some_and(|eta| eta + 2 < turns) {
                continue;
            }
            let effect = data::building_stats(building).yields;
            let value = 200 + 12 * (effect.food + effect.production + effect.gold + effect.science);
            let race_cost = rival_eta.map_or(0, |eta| 80 / (eta - turns).max(1));
            let score = value - turns * 2 - race_cost;
            if score > best_score {
                best_score = score;
                let steps = std::iter::once(host)
                    .chain(support)
                    .flat_map(|c| c.3.iter().map(|b| (c.0, *b)))
                    .collect();
                best = Some(WorkPlan {
                    building,
                    host: host.0,
                    steps,
                    research: research.clone(),
                    turns,
                });
            }
        }
    }
    best
}

impl Planner<'_> {
    pub(crate) fn work_bonus(&self, city: CityId, building: BuildingKind) -> i32 {
        if self.view().turn < self.decisions.work_plan_turn as u32 {
            return 0;
        }
        self.work_plan.as_ref().map_or(0, |plan| {
            if plan.turns > self.turns_remaining() {
                return 0;
            }
            if (plan.host == city && plan.building == building)
                || plan.steps.contains(&(city, building))
            {
                1200
            } else {
                0
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder};
    use legatum_core::{Faction, Hex, projects::GreatWorkView};

    #[test]
    fn chooses_each_route_when_existing_investments_make_it_the_fastest() {
        for building in data::GREAT_WORKS {
            let mut v = ViewBuilder::new(30, 20)
                .known_techs(&TechId::ALL)
                .city(1, ME, Hex::from_offset(5, 5), 8)
                .city(2, ME, Hex::from_offset(12, 5), 8)
                .build();
            let required = data::great_work_requirement(building).map(|(required, _)| required);
            for city in &mut v.cities {
                let d = city.detail.as_mut().unwrap();
                d.buildings = required.into_iter().collect();
                d.options
                    .retain(|o| o.item == ProductionItem::Building { building });
                for o in &mut d.options {
                    o.turns = 10;
                    o.buy_cost = None;
                }
                d.yields.total.production = 20;
                d.producing = None;
            }
            let ctx = Ctx::new(&v);
            let plan = work_plan(&ctx).unwrap();
            assert_eq!(plan.building, building);
            assert!(plan.research.is_empty());
            assert!(plan.steps.is_empty());
            let mut planner = Planner::new(
                &v,
                crate::Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            planner.plan_cities();
            assert!(planner.orders.iter().any(|o| matches!(o,legatum_core::Order::SetProduction{item:ProductionItem::Building{building:b},..} if *b==building)));
            v.great_works.push(GreatWorkView {
                player: ME,
                completed: Some(building),
                building: None,
                city: None,
                progress: 0,
                cost: 0,
                production_per_turn: 0,
                turns: None,
                blocker: None,
            });
            assert!(work_plan(&Ctx::new(&v)).is_none());
        }
    }
    #[test]
    fn research_schedule_preserves_the_active_queue_and_at_most_one_unlock_per_turn() {
        let mut v = ViewBuilder::new(24, 16)
            .researching(TechId::Irrigation)
            .build();
        v.research.progress = 1000;
        let mut missing = BTreeSet::new();
        // Mathematics waits for the whole Ancient era under the strict gate.
        prerequisites(TechId::Mathematics, &[], &mut missing);
        let schedule = research_schedule(&Ctx::new(&v), &missing);
        assert_eq!(schedule[&TechId::Irrigation], 1);
        assert_eq!(schedule[&TechId::BronzeWorking], 2);
        assert_eq!(schedule[&TechId::Masonry], 3);
        assert_eq!(schedule[&TechId::Currency], 4);
        assert_eq!(schedule[&TechId::Writing], 5);
        assert_eq!(schedule[&TechId::Mathematics], 6);
    }

    #[test]
    fn committed_work_keeps_its_host_and_counts_support_in_resistance() {
        let mut v = ViewBuilder::new(30, 20)
            .known_techs(&TechId::ALL)
            .city(1, ME, Hex::from_offset(5, 5), 8)
            .city(2, ME, Hex::from_offset(12, 5), 8)
            .build();
        for city in &mut v.cities {
            city.detail.as_mut().unwrap().buildings = vec![BuildingKind::Bank];
        }
        v.cities[0].detail.as_mut().unwrap().producing = Some(ProductionItem::Building {
            building: BuildingKind::GrandBazaar,
        });
        v.cities[1].detail.as_mut().unwrap().resistance = 3;
        let plan = work_plan(&Ctx::new(&v)).unwrap();
        assert_eq!(plan.building, BuildingKind::GrandBazaar);
        assert_eq!(plan.host, CityId(1));
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn an_existing_machine_shop_can_win_the_remaining_research_and_construction_race() {
        let mut v = ViewBuilder::new(24, 16)
            .faction(Faction::Germany)
            .known_techs(
                &TechId::ALL
                    .into_iter()
                    .filter(|t| legatum_core::eras::era_of_tech(*t) < 4)
                    .collect::<Vec<_>>(),
            )
            .city(1, ME, Hex::from_offset(5, 5), 6)
            .build();
        let d = v.cities[0].detail.as_mut().unwrap();
        d.buildings = vec![BuildingKind::Forge];
        for o in &mut d.options {
            o.yields.total.production = 12;
        }
        d.yields.total.production = 12;
        // With the Great Library's Philosophy already known (the strict gate makes it a
        // prerequisite of the Industrial era), commit to the Royal Arsenal route: the Forge
        // already built leaves the Factory, Industrialization and Ballistics (Cannonry).
        d.producing = Some(ProductionItem::Building {
            building: BuildingKind::RoyalArsenal,
        });
        v.economy.science_per_turn.total = 12;
        let plan = work_plan(&Ctx::new(&v)).unwrap();
        assert_eq!(plan.building, BuildingKind::RoyalArsenal);
        assert_eq!(plan.steps, vec![(CityId(1), BuildingKind::Factory)]);
        assert_eq!(
            plan.research,
            BTreeSet::from([TechId::Cannonry, TechId::Industrialization])
        );
    }
}
